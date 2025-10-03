use anchor_lang::prelude::*;

declare_id!("5c7hSgUxDM1NKAr6nTVpcBpLypdeh6RX2paQueS2Z3Lc");

// Module declarations
pub mod modules;
pub mod integrations;
pub mod shared;
pub mod errors;

// Import what we need
use modules::position::contexts::InitializePosition;
use modules::position::contexts::__client_accounts_initialize_position;
use modules::position::instructions;
use modules::claiming::contexts::{InitializeTreasury, ClaimFees};
use modules::claiming::contexts::{__client_accounts_initialize_treasury, __client_accounts_claim_fees};
use modules::claiming::instructions as claiming_instructions;
use modules::distribution::contexts::{InitializePolicy, InitializeGlobalDistribution, StartDailyDistribution, ProcessInvestorPage, CompleteDailyDistribution};
use modules::distribution::contexts::{__client_accounts_initialize_policy, __client_accounts_initialize_global_distribution, __client_accounts_start_daily_distribution, __client_accounts_process_investor_page, __client_accounts_complete_daily_distribution};
use modules::distribution::instructions as distribution_instructions;

#[program]
pub mod meteora_fee_router {
    use super::*;

    /// Initialize the honorary fee position for quote-only fee accrual
    pub fn initialize_position(ctx: Context<InitializePosition>) -> Result<()> {
        instructions::initialize_position(ctx)
    }

    /// Initialize the treasury for fee claiming
    pub fn initialize_treasury(ctx: Context<InitializeTreasury>, quote_mint: Pubkey) -> Result<()> {
        claiming_instructions::initialize_treasury(ctx, quote_mint)
    }
    /// Claim fees from the honorary position
    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        claiming_instructions::claim_fees(ctx)
    }

    /// Initialize global distribution state
    pub fn initialize_global_distribution(ctx: Context<InitializeGlobalDistribution>, quote_mint: Pubkey) -> Result<()> {
        distribution_instructions::initialize_global_distribution(ctx, quote_mint)
    }

    /// Initialize policy parameters
    pub fn initialize_policy(
        ctx: Context<InitializePolicy>,
        investor_fee_share_bps: u64,
        daily_cap_lamports: u64,
        min_payout_lamports: u64,
        y0_total_allocation: u64,
    ) -> Result<()> {
        distribution_instructions::initialize_policy(
            ctx,
            investor_fee_share_bps,
            daily_cap_lamports,
            min_payout_lamports,
            y0_total_allocation,
        )
    }

    /// Start a new daily distribution (24-hour crank)
    pub fn start_daily_distribution(ctx: Context<StartDailyDistribution>, distribution_day: i64) -> Result<()> {
        distribution_instructions::start_daily_distribution(ctx, distribution_day)
    }

    /// Process a page of investors in the current distribution
    pub fn process_investor_page(ctx: Context<ProcessInvestorPage>) -> Result<()> {
        distribution_instructions::process_investor_page(ctx)
    }

    /// Complete the daily distribution
    pub fn complete_daily_distribution(ctx: Context<CompleteDailyDistribution>) -> Result<()> {
        distribution_instructions::complete_daily_distribution(ctx)
    }

    // TODO: Add other instructions as modules are built
    // pub fn initialize_policy(ctx: Context<policy::InitializePolicy>, ...) -> Result<()>
}
