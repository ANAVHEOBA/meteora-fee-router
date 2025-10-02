use anchor_lang::prelude::*;

/// Event emitted when a new daily distribution is started
#[event]
pub struct DailyDistributionStarted {
    /// The day this distribution represents
    pub distribution_day: i64,
    
    /// Quote mint being distributed
    pub quote_mint: Pubkey,
    
    /// Total amount available for distribution
    pub total_amount_to_distribute: u64,
    
    /// Total number of investors to process
    pub total_investors: u32,
    
    /// Timestamp when started
    pub timestamp: i64,
}

/// Event emitted when a page of investors is processed
#[event]
pub struct InvestorsProcessed {
    /// The distribution day
    pub distribution_day: i64,
    
    /// Quote mint being distributed
    pub quote_mint: Pubkey,
    
    /// Number of investors processed in this page
    pub investors_in_page: u32,
    
    /// Amount distributed in this page
    pub amount_distributed_in_page: u64,
    
    /// New cursor position
    pub new_cursor: u32,
    
    /// Total investors processed so far
    pub total_investors_processed: u32,
    
    /// Total amount distributed so far
    pub total_amount_distributed: u64,
    
    /// Whether this was the final page
    pub is_final_page: bool,
    
    /// Timestamp
    pub timestamp: i64,
}

/// Event emitted when daily distribution is completed
#[event]
pub struct DailyDistributionCompleted {
    /// The distribution day
    pub distribution_day: i64,
    
    /// Quote mint that was distributed
    pub quote_mint: Pubkey,
    
    /// Total amount distributed
    pub total_amount_distributed: u64,
    
    /// Total investors processed
    pub total_investors_processed: u32,
    
    /// Timestamp when completed
    pub timestamp: i64,
}

/// Event emitted when global distribution state is updated
#[event]
pub struct GlobalDistributionUpdated {
    /// Quote mint
    pub quote_mint: Pubkey,
    
    /// New last distribution day
    pub last_distribution_day: i64,
    
    /// Total distributions completed
    pub total_distributions: u64,
    
    /// Total amount distributed across all time
    pub total_amount_distributed: u64,
    
    /// Timestamp
    pub timestamp: i64,
}

/// Event emitted for individual investor payout
#[event]
pub struct InvestorPayout {
    /// The distribution day
    pub distribution_day: i64,
    
    /// Quote mint being distributed
    pub quote_mint: Pubkey,
    
    /// The investor who received the payout
    pub investor: Pubkey,
    
    /// Amount paid out to this investor
    pub payout_amount: u64,
    
    /// The investor's weight in basis points
    pub weight_bps: u64,
    
    /// The investor's locked amount
    pub locked_amount: u64,
    
    /// Timestamp
    pub timestamp: i64,
}

/// Event emitted with distribution calculation details
#[event]
pub struct DistributionCalculationComplete {
    /// The distribution day
    pub distribution_day: i64,
    
    /// Quote mint being distributed
    pub quote_mint: Pubkey,
    
    /// Total locked amount across all investors
    pub total_locked: u64,
    
    /// Locked fraction in basis points
    pub locked_fraction_bps: u64,
    
    /// Eligible investor share in basis points
    pub eligible_investor_share_bps: u64,
    
    /// Total investor fee amount
    pub investor_fee_quote: u64,
    
    /// Total amount distributed to investors
    pub total_distributed: u64,
    
    /// Dust amount carried over
    pub dust_amount: u64,
    
    /// Creator remainder amount
    pub creator_remainder: u64,
    
    /// Timestamp
    pub timestamp: i64,
}

/// Event emitted when creator receives remainder payout
#[event]
pub struct CreatorPayoutCompleted {
    /// The distribution day
    pub distribution_day: i64,
    
    /// Quote mint that was distributed
    pub quote_mint: Pubkey,
    
    /// Creator who received the payout
    pub creator: Pubkey,
    
    /// Amount paid to creator (remainder after investor payouts)
    pub creator_remainder: u64,
    
    /// Total amount that was available for distribution
    pub total_distributed_amount: u64,
    
    /// Total amount paid to investors
    pub total_investor_payouts: u64,
    
    /// Dust amount included in creator remainder
    pub dust_amount: u64,
    
    /// Timestamp when payout completed
    pub timestamp: i64,
}
