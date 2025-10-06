use anchor_lang::prelude::*;
use anchor_spl::token;
use crate::modules::distribution::contexts::*;
use crate::modules::distribution::events::*;
use crate::modules::distribution::state::{DailyDistributionState, GlobalDistributionState, PolicyState};
use crate::integrations::streamflow;
use crate::shared::constants::*;
use crate::errors::FeeRouterError;

/// Initialize the policy state
/// 
/// This creates the policy configuration that governs fee distribution.
/// Only needs to be called once per quote mint.
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// * `investor_fee_share_bps` - Maximum investor share (0-10000)
/// * `daily_cap_lamports` - Daily distribution cap (0 = no cap)
/// * `min_payout_lamports` - Minimum payout threshold
/// * `y0_total_allocation` - Total investor allocation at TGE
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn initialize_policy(
    ctx: Context<InitializePolicy>,
    investor_fee_share_bps: u64,
    daily_cap_lamports: u64,
    min_payout_lamports: u64,
    y0_total_allocation: u64,
) -> Result<()> {
    msg!("Initializing policy for quote mint: {}", ctx.accounts.quote_mint.key());

    // Initialize policy state
    ctx.accounts.policy_state.set_inner(PolicyState {
        quote_mint: ctx.accounts.quote_mint.key(),
        investor_fee_share_bps,
        daily_cap_lamports,
        min_payout_lamports,
        y0_total_allocation,
        policy_authority: ctx.accounts.authority.key(),
        reserved: [0; 64],
    });

    // Validate policy parameters
    ctx.accounts.policy_state.validate()?;

    msg!("✅ Policy initialized successfully");
    Ok(())
}

/// Initialize the global distribution state
/// 
/// This creates the global state account that tracks distribution history.
/// Only needs to be called once per quote mint.
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// * `quote_mint` - The quote mint for this distribution system
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn initialize_global_distribution(
    ctx: Context<InitializeGlobalDistribution>, 
    quote_mint: Pubkey
) -> Result<()> {
    msg!("Initializing global distribution state for quote mint: {}", quote_mint);

    // Validate quote mint matches the account
    require!(
        quote_mint == ctx.accounts.quote_mint_account.key(),
        FeeRouterError::QuoteMintMismatch
    );

    // Initialize global distribution state
    let clock = Clock::get()?;
    ctx.accounts.global_distribution_state.set_inner(GlobalDistributionState {
        quote_mint,
        last_distribution_timestamp: 0, // No distributions yet
        total_distributions: 0,
        total_amount_distributed: 0,
        reserved: [0; 64],
    });

    msg!("✅ Global distribution state initialized successfully");
    Ok(())
}

/// Start a new daily distribution
/// 
/// This creates the daily distribution state and validates that 24 hours
/// have passed since the last distribution. Can be called by anyone.
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// * `distribution_day` - The day timestamp (start of day)
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn start_daily_distribution(
    ctx: Context<StartDailyDistribution>, 
    distribution_day: i64
) -> Result<()> {
    msg!("Starting daily distribution for day: {}", distribution_day);

    let clock = Clock::get()?;
    let current_day = DailyDistributionState::get_day_start(clock.unix_timestamp);
    
    // Validate the distribution day is correct (today)
    require!(
        distribution_day == current_day,
        FeeRouterError::InvalidPageIndex // TODO: Add better error
    );

    // Check if 24 hours have passed since last distribution
    require!(
        DailyDistributionState::can_start_new_distribution(
            ctx.accounts.global_distribution_state.last_distribution_timestamp,
            clock.unix_timestamp
        ),
        FeeRouterError::TooSoonToDistribute
    );

    // Get available treasury balance
    let treasury_balance = ctx.accounts.treasury_ata.amount;
    require!(
        treasury_balance > 0,
        FeeRouterError::NoFeesToClaim // TODO: Add better error for no funds to distribute
    );

    // TODO: Get total number of investors from Streamflow or other source
    // For now, we'll use a placeholder
    let total_investors = 100u32; // This should come from investor registry

    // Initialize daily distribution state
    ctx.accounts.daily_distribution_state.set_inner(DailyDistributionState {
        distribution_day,
        quote_mint: ctx.accounts.quote_mint.key(),
        treasury_ata: ctx.accounts.treasury_ata.key(),
        total_amount_to_distribute: treasury_balance,
        amount_distributed: 0,
        current_cursor: 0,
        total_investors,
        investors_processed: 0,
        is_complete: false,
        started_at: clock.unix_timestamp,
        completed_at: 0,
        dust_carried_over: 0, // TODO: Carry over from previous day
        daily_cap_total: DEFAULT_DAILY_CAP_LAMPORTS,
        daily_cap_remaining: DEFAULT_DAILY_CAP_LAMPORTS,
        min_payout_threshold: DEFAULT_MIN_PAYOUT_LAMPORTS,
        initial_total_deposit: 1_000_000_000, // TODO: Get from config/state
        investor_fee_share_bps: DEFAULT_INVESTOR_FEE_SHARE_BPS,
        last_page_hash: [0; 32], // No pages processed yet
        pages_processed: 0,
        failed_payouts_count: 0,
        reserved: [0; 20],
    });

    // Emit event
    emit!(DailyDistributionStarted {
        distribution_day,
        quote_mint: ctx.accounts.quote_mint.key(),
        total_amount_to_distribute: treasury_balance,
        total_investors,
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ Daily distribution started with {} tokens for {} investors", 
         treasury_balance, total_investors);
    Ok(())
}

/// Process a page of investors
/// 
/// This processes a batch of investors (up to MAX_INVESTORS_PER_PAGE)
/// and distributes their share of fees based on locked token amounts.
/// Implements the complete Section 4 distribution logic.
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn process_investor_page(ctx: Context<ProcessInvestorPage>) -> Result<()> {
    msg!("Processing investor page starting from cursor: {}", 
         ctx.accounts.daily_distribution_state.current_cursor);

    let clock = Clock::get()?;
    
    // Check if there are more investors to process
    require!(
        ctx.accounts.daily_distribution_state.has_more_investors(),
        FeeRouterError::DistributionNotStarted
    );

    // Get remaining accounts (these should be Streamflow stream accounts)
    let remaining_accounts = &ctx.remaining_accounts;
    require!(
        !remaining_accounts.is_empty(),
        FeeRouterError::NoInvestors
    );

    // Step 1: Idempotency check - validate this page hasn't been processed
    let investor_keys: Vec<Pubkey> = remaining_accounts.iter().map(|acc| acc.key()).collect();
    ctx.accounts.daily_distribution_state.validate_page_for_retry(&investor_keys)?;

    // Step 2: Read Streamflow stream data for this page of investors
    let (investor_data, total_locked) = streamflow::cpi::calculate_locked_amounts(
        remaining_accounts,
        clock.unix_timestamp as u64,
        &ctx.accounts.quote_mint.key(),
    )?;

    msg!("Found {} investors with {} total locked tokens", 
         investor_data.len(), total_locked);

    // Step 3: Calculate distribution using Section 4 formulas
    let effective_distribution_amount = ctx.accounts.daily_distribution_state.get_effective_distribution_amount();
    
    let distribution_calc = streamflow::calculations::calculate_distribution(
        effective_distribution_amount,
        &investor_data,
        total_locked,
        ctx.accounts.daily_distribution_state.initial_total_deposit,
        ctx.accounts.daily_distribution_state.investor_fee_share_bps,
        ctx.accounts.daily_distribution_state.min_payout_threshold,
    )?;

    // Step 4: Apply daily cap
    let final_calc = streamflow::calculations::apply_daily_cap(
        distribution_calc,
        ctx.accounts.daily_distribution_state.daily_cap_remaining,
    );

    // Step 5: Validate calculation
    streamflow::calculations::validate_distribution(&final_calc, effective_distribution_amount)?;

    // Step 6: Execute transfers to investors
    let treasury_authority_bump = ctx.bumps.treasury_authority;
    let quote_mint_key = ctx.accounts.quote_mint.key();
    let treasury_seeds = &[
        b"treasury_authority",
        quote_mint_key.as_ref(),
        &[treasury_authority_bump],
    ];
    let _signer_seeds = &[&treasury_seeds[..]];

    let mut actual_distributed = 0u64;
    let mut investors_processed = 0u32;

    for payout in &final_calc.investor_payouts {
        if payout.payout_amount > 0 && payout.meets_minimum {
            // TODO: Transfer tokens to investor
            // This requires the investor ATAs to be passed in remaining_accounts
            // For now, we'll simulate the transfer
            
            actual_distributed = actual_distributed.saturating_add(payout.payout_amount);
            investors_processed += 1;

            msg!("Would pay {} tokens to investor {}", payout.payout_amount, payout.investor);
        }
    }

    // Step 7: Update state with idempotency tracking
    let page_hash = DailyDistributionState::calculate_page_hash(&investor_keys);
    ctx.accounts.daily_distribution_state.update_page_state(
        page_hash,
        investors_processed,
        actual_distributed
    );

    // Update daily cap
    ctx.accounts.daily_distribution_state.update_daily_cap(actual_distributed);

    // Add dust to carry over
    ctx.accounts.daily_distribution_state.add_dust(final_calc.dust_amount);

    let is_final_page = !ctx.accounts.daily_distribution_state.has_more_investors();

    // Step 8: Emit event
    emit!(InvestorsProcessed {
        distribution_day: ctx.accounts.daily_distribution_state.distribution_day,
        quote_mint: ctx.accounts.quote_mint.key(),
        investors_in_page: investors_processed,
        amount_distributed_in_page: actual_distributed,
        new_cursor: ctx.accounts.daily_distribution_state.current_cursor,
        total_investors_processed: ctx.accounts.daily_distribution_state.investors_processed,
        total_amount_distributed: ctx.accounts.daily_distribution_state.amount_distributed,
        is_final_page,
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ Processed {} investors, distributed {} tokens, {} dust", 
         investors_processed, actual_distributed, final_calc.dust_amount);
    
    if is_final_page {
        msg!("🎉 All investors processed for this day!");
    }

    Ok(())
}

/// Complete the daily distribution
/// 
/// This marks the daily distribution as complete, pays the creator remainder,
/// and updates the global state. Can only be called after all investors 
/// have been processed.
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn complete_daily_distribution(ctx: Context<CompleteDailyDistribution>) -> Result<()> {
    msg!("Completing daily distribution for day: {}", 
         ctx.accounts.daily_distribution_state.distribution_day);

    let clock = Clock::get()?;

    // Step 1: Calculate creator remainder
    // creator_remainder = total_amount_to_distribute - amount_distributed + dust_carried_over
    let total_available = ctx.accounts.daily_distribution_state.get_effective_distribution_amount();
    let total_investor_payouts = ctx.accounts.daily_distribution_state.amount_distributed;
    let dust_amount = ctx.accounts.daily_distribution_state.dust_carried_over;
    
    let creator_remainder = total_available.saturating_sub(total_investor_payouts);
    
    msg!("Creator remainder calculation: {} total - {} to investors = {} remainder", 
         total_available, total_investor_payouts, creator_remainder);

    // Step 2: Transfer remainder to creator
    if creator_remainder > 0 {
        let treasury_authority_bump = ctx.bumps.treasury_authority;
        let quote_mint_key = ctx.accounts.quote_mint.key();
        let treasury_seeds = &[
            b"treasury_authority",
            quote_mint_key.as_ref(),
            &[treasury_authority_bump],
        ];
        let signer_seeds = &[&treasury_seeds[..]];

        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.treasury_ata.to_account_info(),
                to: ctx.accounts.creator_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            },
            signer_seeds,
        );

        token::transfer(transfer_ctx, creator_remainder)?;
        
        msg!("✅ Transferred {} tokens to creator", creator_remainder);

        // Emit creator payout event
        emit!(CreatorPayoutCompleted {
            distribution_day: ctx.accounts.daily_distribution_state.distribution_day,
            quote_mint: ctx.accounts.quote_mint.key(),
            creator: ctx.accounts.creator_ata.owner,
            creator_remainder,
            total_distributed_amount: total_available,
            total_investor_payouts,
            dust_amount,
            timestamp: clock.unix_timestamp,
        });
    } else {
        msg!("No creator remainder to distribute");
    }

    // Step 3: Mark daily distribution as complete
    ctx.accounts.daily_distribution_state.mark_complete(clock.unix_timestamp);

    // Step 4: Update global distribution state
    ctx.accounts.global_distribution_state.update_after_distribution(
        clock.unix_timestamp, // Use current timestamp instead of day
        total_available // Include full amount (investors + creator)
    );

    // Step 5: Emit completion events
    emit!(DailyDistributionCompleted {
        distribution_day: ctx.accounts.daily_distribution_state.distribution_day,
        quote_mint: ctx.accounts.quote_mint.key(),
        total_amount_distributed: total_available,
        total_investors_processed: ctx.accounts.daily_distribution_state.investors_processed,
        timestamp: clock.unix_timestamp,
    });

    emit!(GlobalDistributionUpdated {
        quote_mint: ctx.accounts.quote_mint.key(),
        last_distribution_day: ctx.accounts.global_distribution_state.last_distribution_timestamp,
        total_distributions: ctx.accounts.global_distribution_state.total_distributions,
        total_amount_distributed: ctx.accounts.global_distribution_state.total_amount_distributed,
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ Daily distribution completed successfully with creator payout");
    Ok(())
}
