use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token_2022::Token2022;
use crate::integrations::meteora::POOL_AUTHORITY;
use crate::modules::position::state::PositionMetadata;
use crate::shared::constants::*;

/// Accounts required to initialize the honorary fee position
#[derive(Accounts)]
pub struct InitializePosition<'info> {
    /// The authority initializing the position (pays for creation)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The vault account (used for PDA derivation)
    /// CHECK: Used as seed for PDA derivation
    pub vault: UncheckedAccount<'info>,

    /// PDA that will own the honorary position
    /// Seeds: [VAULT_SEED, vault, "investor_fee_pos_owner"]
    #[account(
        seeds = [VAULT_SEED, vault.key().as_ref(), POSITION_OWNER_SEED],
        bump,
    )]
    /// CHECK: PDA owner of the position
    pub position_owner_pda: UncheckedAccount<'info>,

    /// The Meteora pool account
    /// CHECK: Validated in instruction
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// The pool's base token mint
    pub base_mint: Account<'info, Mint>,

    /// The pool's quote token mint
    pub quote_mint: Account<'info, Mint>,

    /// Position NFT mint (must be a signer, will be created)
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,

    /// Position NFT account (PDA derived by Meteora)
    /// CHECK: Derived by Meteora program
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,

    /// The position account (PDA derived by Meteora)
    /// CHECK: Derived by Meteora program
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// Meteora pool authority
    /// CHECK: Verified by address constraint
    #[account(address = POOL_AUTHORITY)]
    pub pool_authority: UncheckedAccount<'info>,

    /// Event authority PDA (required by Meteora)
    /// CHECK: Derived by Meteora program
    pub event_authority: UncheckedAccount<'info>,

    /// Meteora CP-AMM program
    /// CHECK: Meteora program ID
    pub meteora_program: UncheckedAccount<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Token program (Token2022 for Meteora)
    pub token_program: Program<'info, Token2022>,

    /// Token A program (for add_liquidity)
    pub token_a_program: Program<'info, Token2022>,

    /// Token B program (for add_liquidity) 
    pub token_b_program: Program<'info, Token2022>,

    /// Authority's token A account (for providing liquidity)
    #[account(mut)]
    pub authority_token_a: UncheckedAccount<'info>,

    /// Authority's token B account (for providing liquidity)
    #[account(mut)]
    pub authority_token_b: UncheckedAccount<'info>,

    /// Pool's token A vault
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,

    /// Pool's token B vault
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,

    /// Position metadata account to store position information
    #[account(
        init,
        payer = authority,
        space = 8 + PositionMetadata::INIT_SPACE,
        seeds = [b"position_metadata", position_nft_mint.key().as_ref()],
        bump,
    )]
    pub position_metadata: Account<'info, PositionMetadata>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}
