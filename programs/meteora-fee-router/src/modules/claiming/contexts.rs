use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use anchor_spl::associated_token::AssociatedToken;
use crate::modules::claiming::state::TreasuryState;
use crate::modules::position::state::PositionMetadata;
use crate::integrations::meteora::POOL_AUTHORITY;
use crate::shared::constants::*;

/// Accounts required to initialize the treasury for fee claiming
#[derive(Accounts)]
#[instruction(quote_mint: Pubkey)]
pub struct InitializeTreasury<'info> {
    /// The authority initializing the treasury (pays for creation)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The quote mint for this treasury
    pub quote_mint_account: Account<'info, Mint>,

    /// Treasury state account
    #[account(
        init,
        payer = authority,
        space = 8 + TreasuryState::INIT_SPACE,
        seeds = [b"treasury_state", quote_mint.as_ref()],
        bump,
    )]
    pub treasury_state: Account<'info, TreasuryState>,

    /// Treasury ATA to hold claimed fees
    #[account(
        init,
        payer = authority,
        associated_token::mint = quote_mint_account,
        associated_token::authority = treasury_authority,
    )]
    pub treasury_ata: Account<'info, TokenAccount>,

    /// Treasury authority PDA (owns the ATA)
    /// Seeds: [b"treasury_authority", quote_mint]
    #[account(
        seeds = [b"treasury_authority", quote_mint.as_ref()],
        bump,
    )]
    /// CHECK: PDA authority for treasury ATA
    pub treasury_authority: UncheckedAccount<'info>,

    /// Position owner PDA (will be the claim authority)
    /// Seeds: [VAULT_SEED, vault, POSITION_OWNER_SEED]
    /// CHECK: Validated by seeds
    pub position_owner_pda: UncheckedAccount<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts required to claim fees from the position
#[derive(Accounts)]
pub struct ClaimFees<'info> {
    /// The position metadata account
    #[account(
        seeds = [b"position_metadata", position_nft_mint.key().as_ref()],
        bump,
    )]
    pub position_metadata: Account<'info, PositionMetadata>,

    /// Position NFT mint
    pub position_nft_mint: Account<'info, Mint>,

    /// The Meteora pool
    /// CHECK: Validated against position metadata
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// The Meteora position account
    /// CHECK: Validated against position metadata
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// Position NFT account
    /// CHECK: Derived by Meteora program
    pub position_nft_account: UncheckedAccount<'info>,

    /// Position owner PDA (authority to claim fees)
    #[account(
        seeds = [VAULT_SEED, vault.key().as_ref(), POSITION_OWNER_SEED],
        bump,
    )]
    /// CHECK: PDA owner of the position
    pub position_owner_pda: UncheckedAccount<'info>,

    /// The vault account (used for PDA derivation)
    /// CHECK: Used as seed for PDA derivation
    pub vault: UncheckedAccount<'info>,

    /// Treasury state account
    #[account(
        mut,
        seeds = [b"treasury_state", quote_mint.key().as_ref()],
        bump,
        constraint = treasury_state.quote_mint == quote_mint.key(),
        constraint = treasury_state.claim_authority == position_owner_pda.key(),
    )]
    pub treasury_state: Account<'info, TreasuryState>,

    /// Quote mint
    pub quote_mint: Account<'info, Mint>,

    /// Base mint
    pub base_mint: Account<'info, Mint>,

    /// Treasury ATA to receive claimed fees
    #[account(
        mut,
        constraint = treasury_ata.key() == treasury_state.treasury_ata,
        constraint = treasury_ata.mint == quote_mint.key(),
    )]
    pub treasury_ata: Account<'info, TokenAccount>,

    /// Position owner's quote token account (temporary holder)
    #[account(
        mut,
        constraint = position_owner_quote_ata.mint == quote_mint.key(),
        constraint = position_owner_quote_ata.owner == position_owner_pda.key(),
    )]
    pub position_owner_quote_ata: Account<'info, TokenAccount>,

    /// Position owner's base token account (should remain empty)
    #[account(
        mut,
        constraint = position_owner_base_ata.mint == base_mint.key(),
        constraint = position_owner_base_ata.owner == position_owner_pda.key(),
    )]
    pub position_owner_base_ata: Account<'info, TokenAccount>,

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

    /// Token program
    pub token_program: Program<'info, Token>,
}
