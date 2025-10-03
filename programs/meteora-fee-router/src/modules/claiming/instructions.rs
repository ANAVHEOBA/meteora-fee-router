use anchor_lang::prelude::*;
use anchor_spl::token;
use crate::modules::claiming::contexts::*;
use crate::modules::claiming::events::*;
use crate::modules::claiming::state::TreasuryState;
use crate::integrations::meteora;
use crate::shared::constants::*;
use crate::errors::FeeRouterError;

/// Initialize the treasury for fee claiming
/// 
/// This creates the treasury state and ATA to receive claimed fees.
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// * `quote_mint` - The quote mint for this treasury
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn initialize_treasury(ctx: Context<InitializeTreasury>, quote_mint: Pubkey) -> Result<()> {
    msg!("Initializing treasury for quote mint: {}", quote_mint);

    // Validate quote mint matches the account
    require!(
        quote_mint == ctx.accounts.quote_mint_account.key(),
        anchor_lang::error::ErrorCode::ConstraintRaw
    );

    // Initialize treasury state
    let clock = Clock::get()?;
    ctx.accounts.treasury_state.set_inner(TreasuryState {
        quote_mint,
        treasury_ata: ctx.accounts.treasury_ata.key(),
        total_fees_claimed: 0,
        last_claim_timestamp: 0,
        claim_count: 0,
        claim_authority: ctx.accounts.position_owner_pda.key(),
        reserved: [0; 64],
    });

    // Emit event
    emit!(TreasuryInitialized {
        quote_mint,
        treasury_ata: ctx.accounts.treasury_ata.key(),
        claim_authority: ctx.accounts.position_owner_pda.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ Treasury initialized successfully");
    Ok(())
}

/// Claim fees from the honorary position
/// 
/// This claims accumulated fees from the Meteora position and transfers
/// them to the treasury. Validates that only quote tokens are claimed.
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
    msg!("Claiming fees from honorary position");

    // Validate position metadata matches accounts
    require!(
        ctx.accounts.position_metadata.position == ctx.accounts.position.key(),
        FeeRouterError::PositionMetadataMismatch
    );
    require!(
        ctx.accounts.position_metadata.pool == ctx.accounts.pool.key(),
        FeeRouterError::PositionMetadataMismatch
    );
    require!(
        ctx.accounts.position_metadata.quote_mint == ctx.accounts.quote_mint.key(),
        FeeRouterError::PositionMetadataMismatch
    );

    // Check if enough time has passed since last claim (optional cooldown)
    let clock = Clock::get()?;
    let min_claim_interval = 3600; // 1 hour minimum between claims
    require!(
        ctx.accounts.treasury_state.can_claim(clock.unix_timestamp, min_claim_interval),
        FeeRouterError::ClaimIntervalNotElapsed
    );

    // Get balances before claiming
    let quote_balance_before = ctx.accounts.position_owner_quote_ata.amount;
    let base_balance_before = ctx.accounts.position_owner_base_ata.amount;

    // Step 1 - Claim fees from Meteora position via CPI with error handling
    let vault_key = ctx.accounts.vault.key();
    let bump = ctx.bumps.position_owner_pda;
    let owner_seeds = &[
        VAULT_SEED,
        vault_key.as_ref(),
        POSITION_OWNER_SEED,
        &[bump],
    ];
    let signer_seeds = &[&owner_seeds[..]];

    // Attempt the Meteora CPI call with error wrapping
    meteora::cpi::claim_position_fee(
        ctx.accounts.pool_authority.to_account_info(),
        ctx.accounts.pool.to_account_info(),
        ctx.accounts.position.to_account_info(),
        ctx.accounts.position_owner_quote_ata.to_account_info(), // token_a_account
        ctx.accounts.position_owner_base_ata.to_account_info(),  // token_b_account
        // Note: We need to determine which vault is A and which is B based on mint order
        ctx.accounts.pool.to_account_info(), // token_a_vault (placeholder)
        ctx.accounts.pool.to_account_info(), // token_b_vault (placeholder)
        ctx.accounts.quote_mint.to_account_info(), // token_a_mint (placeholder)
        ctx.accounts.base_mint.to_account_info(),  // token_b_mint (placeholder)
        ctx.accounts.position_nft_account.to_account_info(),
        ctx.accounts.position_owner_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(), // token_a_program
        ctx.accounts.token_program.to_account_info(), // token_b_program
        ctx.accounts.event_authority.to_account_info(),
        ctx.accounts.meteora_program.to_account_info(),
        Some(signer_seeds),
    ).map_err(|_| FeeRouterError::MeteoraCpiFailed)?;

    // Refresh account data to get updated balances
    ctx.accounts.position_owner_quote_ata.reload()?;
    ctx.accounts.position_owner_base_ata.reload()?;

    // Calculate claimed amounts
    let quote_amount_claimed = ctx.accounts.position_owner_quote_ata.amount
        .saturating_sub(quote_balance_before);
    let base_amount_claimed = ctx.accounts.position_owner_base_ata.amount
        .saturating_sub(base_balance_before);

    msg!("Quote claimed: {}, Base claimed: {}", quote_amount_claimed, base_amount_claimed);

    // Step 2 - Verify only quote tokens were claimed (base should be 0)
    require!(
        base_amount_claimed == 0,
        FeeRouterError::BaseFeesClaimedError
    );

    // Check if any fees were actually claimed
    require!(
        quote_amount_claimed > 0,
        FeeRouterError::NoFeesToClaim
    );

    // Step 3 - Transfer claimed quote tokens to treasury with error handling
    let treasury_balance_before = ctx.accounts.treasury_ata.amount;
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.position_owner_quote_ata.to_account_info(),
            to: ctx.accounts.treasury_ata.to_account_info(),
            authority: ctx.accounts.position_owner_pda.to_account_info(),
        },
        signer_seeds,
    );

    token::transfer(transfer_ctx, quote_amount_claimed)
        .map_err(|_| FeeRouterError::TreasuryTransferFailed)?;

    // Verify the transfer succeeded by checking treasury balance
    ctx.accounts.treasury_ata.reload()?;
    let expected_balance = treasury_balance_before.saturating_add(quote_amount_claimed);
    require!(
        ctx.accounts.treasury_ata.amount == expected_balance,
        FeeRouterError::TreasuryBalanceMismatch
    );

    // Step 4 - Update treasury state with overflow protection
    ctx.accounts.treasury_state.record_claim(quote_amount_claimed, clock.unix_timestamp);

    // Step 5 - Emit event
    emit!(FeesClaimedFromPosition {
        position: ctx.accounts.position.key(),
        pool: ctx.accounts.pool.key(),
        quote_amount_claimed,
        base_amount_claimed,
        treasury_ata: ctx.accounts.treasury_ata.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        timestamp: clock.unix_timestamp,
        total_fees_claimed: ctx.accounts.treasury_state.total_fees_claimed,
    });

    msg!("✅ Fees claimed successfully: {} quote tokens", quote_amount_claimed);
    Ok(())
}
