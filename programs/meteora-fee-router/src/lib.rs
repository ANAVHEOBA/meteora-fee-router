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

#[program]
pub mod meteora_fee_router {
    use super::*;

    /// Initialize the honorary fee position for quote-only fee accrual
    pub fn initialize_position(ctx: Context<InitializePosition>) -> Result<()> {
        instructions::initialize_position(ctx)
    }

    // TODO: Add other instructions as modules are built
    // pub fn initialize_policy(ctx: Context<policy::InitializePolicy>, ...) -> Result<()
    // pub fn distribute_fees(ctx: Context<distribution::DistributeFees>, ...) -> Result<()
    // pub fn claim_fees(ctx: Context<claiming::ClaimFees>) -> Result<()
}
