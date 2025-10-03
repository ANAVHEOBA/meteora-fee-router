use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::modules::distribution::state::{DailyDistributionState, GlobalDistributionState, PolicyState};
use crate::modules::claiming::state::TreasuryState;

/// Accounts required to initialize policy state
#[derive(Accounts)]
pub struct InitializePolicy<'info> {
    /// The authority initializing the policy (pays for creation)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Quote mint this policy applies to
    pub quote_mint: Account<'info, Mint>,

    /// Policy state PDA to create
    #[account(
        init,
        payer = authority,
        space = PolicyState::INIT_SPACE,
        seeds = [b"policy", quote_mint.key().as_ref()],
        bump,
    )]
    pub policy_state: Account<'info, PolicyState>,

    /// System program
    pub system_program: Program<'info, System>,
}

/// Accounts required to initialize global distribution state
#[derive(Accounts)]
#[instruction(quote_mint: Pubkey)]
pub struct InitializeGlobalDistribution<'info> {
    /// The authority initializing the global state (pays for creation)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The quote mint for this distribution system
    pub quote_mint_account: Account<'info, Mint>,

    /// Global distribution state account
    #[account(
        init,
        payer = authority,
        space = 8 + GlobalDistributionState::INIT_SPACE,
        seeds = [b"global_distribution", quote_mint.as_ref()],
        bump,
    )]
    pub global_distribution_state: Account<'info, GlobalDistributionState>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts required to start a new daily distribution
#[derive(Accounts)]
#[instruction(distribution_day: i64)]
pub struct StartDailyDistribution<'info> {
    /// The authority starting the distribution (can be anyone - permissionless)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Quote mint being distributed
    pub quote_mint: Account<'info, Mint>,

    /// Global distribution state
    #[account(
        mut,
        seeds = [b"global_distribution", quote_mint.key().as_ref()],
        bump,
        constraint = global_distribution_state.quote_mint == quote_mint.key(),
    )]
    pub global_distribution_state: Account<'info, GlobalDistributionState>,

    /// Daily distribution state account (created for this day)
    #[account(
        init,
        payer = authority,
        space = 8 + DailyDistributionState::INIT_SPACE,
        seeds = [
            b"daily_distribution",
            distribution_day.to_string().as_bytes(),
            quote_mint.key().as_ref(),
        ],
        bump,
    )]
    pub daily_distribution_state: Account<'info, DailyDistributionState>,

    /// Treasury state to get available balance
    #[account(
        seeds = [b"treasury_state", quote_mint.key().as_ref()],
        bump,
        constraint = treasury_state.quote_mint == quote_mint.key(),
    )]
    pub treasury_state: Account<'info, TreasuryState>,

    /// Treasury ATA to distribute from
    #[account(
        constraint = treasury_ata.key() == treasury_state.treasury_ata,
        constraint = treasury_ata.mint == quote_mint.key(),
    )]
    pub treasury_ata: Account<'info, TokenAccount>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts required to process a page of investors
#[derive(Accounts)]
pub struct ProcessInvestorPage<'info> {
    /// The authority processing this page (can be anyone - permissionless)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Quote mint being distributed
    pub quote_mint: Account<'info, Mint>,

    /// Daily distribution state for the current day
    #[account(
        mut,
        seeds = [
            b"daily_distribution",
            daily_distribution_state.distribution_day.to_string().as_bytes(),
            quote_mint.key().as_ref(),
        ],
        bump,
        constraint = daily_distribution_state.quote_mint == quote_mint.key(),
        constraint = !daily_distribution_state.is_complete,
    )]
    pub daily_distribution_state: Account<'info, DailyDistributionState>,

    /// Treasury ATA to distribute from
    #[account(
        mut,
        constraint = treasury_ata.key() == daily_distribution_state.treasury_ata,
        constraint = treasury_ata.mint == quote_mint.key(),
    )]
    pub treasury_ata: Account<'info, TokenAccount>,

    /// Treasury authority PDA (owns the treasury ATA)
    #[account(
        seeds = [b"treasury_authority", quote_mint.key().as_ref()],
        bump,
    )]
    /// CHECK: PDA authority for treasury ATA
    pub treasury_authority: UncheckedAccount<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,

    // Note: Investor accounts will be passed as remaining_accounts
    // Each investor needs their ATA for receiving tokens
}

/// Accounts required to complete a daily distribution
#[derive(Accounts)]
pub struct CompleteDailyDistribution<'info> {
    /// The authority completing the distribution (can be anyone - permissionless)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Quote mint that was distributed
    pub quote_mint: Account<'info, Mint>,

    /// Global distribution state to update
    #[account(
        mut,
        seeds = [b"global_distribution", quote_mint.key().as_ref()],
        bump,
        constraint = global_distribution_state.quote_mint == quote_mint.key(),
    )]
    pub global_distribution_state: Account<'info, GlobalDistributionState>,

    /// Daily distribution state to mark as complete
    #[account(
        mut,
        seeds = [
            b"daily_distribution",
            daily_distribution_state.distribution_day.to_string().as_bytes(),
            quote_mint.key().as_ref(),
        ],
        bump,
        constraint = daily_distribution_state.quote_mint == quote_mint.key(),
        constraint = !daily_distribution_state.has_more_investors(),
        constraint = !daily_distribution_state.is_complete,
    )]
    pub daily_distribution_state: Account<'info, DailyDistributionState>,

    /// Treasury ATA to transfer creator remainder from
    #[account(
        mut,
        constraint = treasury_ata.key() == daily_distribution_state.treasury_ata,
        constraint = treasury_ata.mint == quote_mint.key(),
    )]
    pub treasury_ata: Account<'info, TokenAccount>,

    /// Treasury authority PDA (owns the treasury ATA)
    #[account(
        seeds = [b"treasury_authority", quote_mint.key().as_ref()],
        bump,
    )]
    /// CHECK: PDA authority for treasury ATA
    pub treasury_authority: UncheckedAccount<'info>,

    /// Creator's ATA for receiving remainder
    #[account(
        mut,
        constraint = creator_ata.mint == quote_mint.key(),
    )]
    pub creator_ata: Account<'info, TokenAccount>,

    /// Token program
    pub token_program: Program<'info, Token>,
}
