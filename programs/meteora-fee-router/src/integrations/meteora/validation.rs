use anchor_lang::prelude::*;
use crate::integrations::meteora::accounts::{Pool, CollectFeeMode};
use crate::errors::FeeRouterError;

/// Validate that the pool is configured for quote-only fee collection
/// 
/// This checks the pool's `collect_fee_mode` to ensure it only collects
/// fees in one token (not both). We need to know which token is the quote
/// token to validate this correctly.
/// 
/// # Arguments
/// * `pool` - The Meteora pool account
/// * `quote_mint` - The quote token mint
/// 
/// # Returns
/// * `Result<()>` - Success if pool is valid for quote-only fees
pub fn validate_quote_only_pool(pool: &Pool, quote_mint: &Pubkey) -> Result<()> {
    msg!("Validating pool for quote-only fee collection");
    
    // Check pool is enabled
    require!(
        pool.is_enabled(),
        FeeRouterError::InvalidPoolConfig
    );

    // Get the collect fee mode
    let fee_mode = pool.get_collect_fee_mode()
        .ok_or(FeeRouterError::InvalidPoolConfig)?;

    // Pool must NOT collect fees in both tokens
    require!(
        fee_mode != CollectFeeMode::Both,
        FeeRouterError::BaseFeeDetected
    );

    // Determine which token is A and which is B
    let quote_is_token_a = pool.token_a_mint == *quote_mint;
    let quote_is_token_b = pool.token_b_mint == *quote_mint;

    // Quote mint must be either token A or token B
    require!(
        quote_is_token_a || quote_is_token_b,
        FeeRouterError::QuoteMintMismatch
    );

    // Validate fee collection matches quote token
    if quote_is_token_a {
        // Quote is token A, so pool must collect fees only in token A
        require!(
            fee_mode == CollectFeeMode::OnlyTokenA,
            FeeRouterError::BaseFeeDetected
        );
        msg!("✅ Pool collects fees only in token A (quote token)");
    } else {
        // Quote is token B, so pool must collect fees only in token B
        require!(
            fee_mode == CollectFeeMode::OnlyTokenB,
            FeeRouterError::BaseFeeDetected
        );
        msg!("✅ Pool collects fees only in token B (quote token)");
    }

    msg!("Pool validation passed - quote-only fees confirmed");
    Ok(())
}

/// Identify which token is the quote token based on pool configuration
/// 
/// In Meteora pools, the quote token is typically the second token (token B),
/// but we should verify based on the fee collection mode and common conventions.
/// 
/// # Arguments
/// * `pool` - The Meteora pool account
/// 
/// # Returns
/// * `Result<Pubkey>` - The quote mint pubkey
pub fn identify_quote_mint(pool: &Pool) -> Result<Pubkey> {
    msg!("Identifying quote mint from pool");
    
    let fee_mode = pool.get_collect_fee_mode()
        .ok_or(FeeRouterError::InvalidPoolConfig)?;

    // If pool collects fees in only one token, that's likely the quote token
    // This is a heuristic - in practice, the quote token should be provided by the user
    match fee_mode {
        CollectFeeMode::OnlyTokenA => {
            msg!("Pool collects fees in token A - likely the quote token");
            Ok(pool.token_a_mint)
        },
        CollectFeeMode::OnlyTokenB => {
            msg!("Pool collects fees in token B - likely the quote token");
            Ok(pool.token_b_mint)
        },
        CollectFeeMode::Both => {
            msg!("❌ Pool collects fees in both tokens - cannot determine quote token");
            Err(FeeRouterError::InvalidPoolConfig.into())
        }
    }
}

/// Validate token order in the pool
/// 
/// Ensure the provided base and quote mints match the pool's tokens
/// 
/// # Arguments
/// * `pool` - The pool account
/// * `base_mint` - The base token mint
/// * `quote_mint` - The quote token mint
/// 
/// # Returns
/// * `Result<()>` - Success if token order is correct
pub fn validate_token_order(
    pool: &Pool,
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
) -> Result<()> {
    msg!("Validating token order");
    
    // Check that both mints are in the pool
    let has_base = pool.token_a_mint == *base_mint || pool.token_b_mint == *base_mint;
    let has_quote = pool.token_a_mint == *quote_mint || pool.token_b_mint == *quote_mint;

    require!(has_base, FeeRouterError::InvalidTokenOrder);
    require!(has_quote, FeeRouterError::QuoteMintMismatch);

    // Ensure base and quote are different
    require!(
        base_mint != quote_mint,
        FeeRouterError::InvalidTokenOrder
    );

    msg!("Token order validated");
    Ok(())
}

/// Preflight validation before position creation
/// 
/// This should be called before attempting to create the position
/// to catch any configuration issues early.
/// 
/// # Arguments
/// * `pool` - The pool account
/// * `base_mint` - The base mint
/// * `quote_mint` - The quote mint
/// 
/// # Returns
/// * `Result<()>` - Success if all validations pass
pub fn preflight_validation(
    pool: &Pool,
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
) -> Result<()> {
    msg!("Running preflight validation");
    
    // Validate pool is enabled
    require!(
        pool.is_enabled(),
        FeeRouterError::InvalidPoolConfig
    );
    
    // Validate token order
    validate_token_order(pool, base_mint, quote_mint)?;
    
    // Validate quote-only fee collection
    validate_quote_only_pool(pool, quote_mint)?;
    
    msg!("✅ Preflight validation passed");
    Ok(())
}
