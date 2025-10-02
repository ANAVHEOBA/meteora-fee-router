use anchor_lang::prelude::*;
use crate::modules::position::contexts::*;
use crate::modules::position::events::*;
use crate::modules::position::state::PositionMetadata;
use crate::integrations::meteora;
use crate::shared::constants::*;

/// Initialize the honorary fee position
/// 
/// This creates a DAMM V2 LP position owned by our program PDA that:
/// - Accrues fees exclusively in the quote mint
/// - Is owned by the InvestorFeePositionOwnerPda
/// - Validates pool configuration for quote-only fees
/// 
/// # Arguments
/// * `ctx` - The context containing all required accounts
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn initialize_position(ctx: Context<InitializePosition>) -> Result<()> {
    msg!("Initializing honorary fee position");

    // Step 1 - Deserialize and validate pool
    let pool_data = ctx.accounts.pool.try_borrow_data()?;
    let pool = bytemuck::from_bytes::<meteora::Pool>(&pool_data[8..]); // Skip 8-byte discriminator
    
    meteora::validation::preflight_validation(
        pool,
        &ctx.accounts.base_mint.key(),
        &ctx.accounts.quote_mint.key(),
    )?;

    // Step 2 - Create DAMM V2 position via CPI
    // The position will be owned by our position_owner_pda
    let vault_key = ctx.accounts.vault.key();
    let bump = ctx.bumps["position_owner_pda"];
    let owner_seeds = &[
        VAULT_SEED,
        vault_key.as_ref(),
        POSITION_OWNER_SEED,
        &[bump],
    ];
    let signer_seeds = &[&owner_seeds[..]];

    meteora::cpi::create_position(
        ctx.accounts.position_owner_pda.to_account_info(),
        ctx.accounts.position_nft_mint.to_account_info(),
        ctx.accounts.position_nft_account.to_account_info(),
        ctx.accounts.pool.to_account_info(),
        ctx.accounts.position.to_account_info(),
        ctx.accounts.pool_authority.to_account_info(),
        ctx.accounts.authority.to_account_info(), // payer
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.event_authority.to_account_info(),
        ctx.accounts.meteora_program.to_account_info(),
        Some(signer_seeds),
    )?;

    // Step 3 - Add minimal liquidity to activate fee collection
    msg!("Adding minimal liquidity to activate position");
    
    // Determine quote amount for minimal liquidity (e.g., 1000 units)
    let minimal_quote_amount = 1000u64;
    let liquidity_params = meteora::AddLiquidityParameters::minimal_quote_only(minimal_quote_amount);

    meteora::cpi::add_liquidity(
        ctx.accounts.pool.to_account_info(),
        ctx.accounts.position.to_account_info(),
        ctx.accounts.authority_token_a.to_account_info(),
        ctx.accounts.authority_token_b.to_account_info(),
        ctx.accounts.token_a_vault.to_account_info(),
        ctx.accounts.token_b_vault.to_account_info(),
        ctx.accounts.base_mint.to_account_info(),
        ctx.accounts.quote_mint.to_account_info(),
        ctx.accounts.position_nft_account.to_account_info(),
        ctx.accounts.position_owner_pda.to_account_info(),
        ctx.accounts.token_a_program.to_account_info(),
        ctx.accounts.token_b_program.to_account_info(),
        ctx.accounts.event_authority.to_account_info(),
        ctx.accounts.meteora_program.to_account_info(),
        liquidity_params,
        Some(signer_seeds),
    )?;

    // Step 4 - Initialize position metadata
    msg!("Storing position metadata");
    
    let clock = Clock::get()?;
    ctx.accounts.position_metadata.set_inner(PositionMetadata {
        position: ctx.accounts.position.key(),
        pool: ctx.accounts.pool.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_mint: ctx.accounts.base_mint.key(),
        created_at: clock.unix_timestamp,
        position_owner_bump: bump,
        reserved: [0; 64],
    });

    // Step 5 - Emit event
    emit!(HonoraryPositionInitialized {
        position: ctx.accounts.position.key(),
        pool: ctx.accounts.pool.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_mint: ctx.accounts.base_mint.key(),
        position_owner: ctx.accounts.position_owner_pda.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ Honorary position initialized with liquidity and metadata successfully");
    Ok(())
}
