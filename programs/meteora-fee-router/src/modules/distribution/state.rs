use anchor_lang::prelude::*;

/// Policy configuration for fee distribution
#[account]
pub struct PolicyState {
    /// Quote mint this policy applies to
    pub quote_mint: Pubkey,
    
    /// Maximum investor share in basis points (0-10000)
    pub investor_fee_share_bps: u64,
    
    /// Daily distribution cap in lamports (0 = no cap)
    pub daily_cap_lamports: u64,
    
    /// Minimum payout threshold in lamports
    pub min_payout_lamports: u64,
    
    /// Total investor allocation at TGE (Y0)
    pub y0_total_allocation: u64,
    
    /// Authority that can update this policy
    pub policy_authority: Pubkey,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl PolicyState {
    pub const INIT_SPACE: usize = 8 +   // discriminator
                                   32 +  // quote_mint
                                   8 +   // investor_fee_share_bps
                                   8 +   // daily_cap_lamports
                                   8 +   // min_payout_lamports
                                   8 +   // y0_total_allocation
                                   32 +  // policy_authority
                                   64;   // reserved

    /// Derive the PDA for policy state
    pub fn derive_pda(quote_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"policy",
                quote_mint.as_ref(),
            ],
            program_id,
        )
    }

    /// Validate policy parameters
    pub fn validate(&self) -> Result<()> {
        require!(
            self.investor_fee_share_bps <= 10000,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        require!(
            self.y0_total_allocation > 0,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
        Ok(())
    }
}

/// Daily distribution state to track progress within a 24-hour period
#[account]
pub struct DailyDistributionState {
    /// The distribution day (Unix timestamp of day start)
    pub distribution_day: i64,
    
    /// Quote mint being distributed
    pub quote_mint: Pubkey,
    
    /// Treasury ATA being distributed from
    pub treasury_ata: Pubkey,
    
    /// Total amount available for distribution this day
    pub total_amount_to_distribute: u64,
    
    /// Amount distributed so far
    pub amount_distributed: u64,
    
    /// Current pagination cursor (investor index)
    pub current_cursor: u32,
    
    /// Total number of investors to process
    pub total_investors: u32,
    
    /// Number of investors processed so far
    pub investors_processed: u32,
    
    /// Whether this day's distribution is complete
    pub is_complete: bool,
    
    /// Timestamp when distribution started
    pub started_at: i64,
    
    /// Timestamp when distribution completed (0 if not complete)
    pub completed_at: i64,
    
    /// Dust amount carried over from previous pages/days
    pub dust_carried_over: u64,
    
    /// Daily distribution cap (max amount that can be distributed per day)
    pub daily_cap_total: u64,
    
    /// Remaining daily cap for this day
    pub daily_cap_remaining: u64,
    
    /// Minimum payout threshold in lamports
    pub min_payout_threshold: u64,
    
    /// Initial total deposit amount (Y0) for locked fraction calculation
    pub initial_total_deposit: u64,
    
    /// Investor fee share in basis points (max share for investors)
    pub investor_fee_share_bps: u64,
    
    /// Hash of the last processed page (for idempotency)
    pub last_page_hash: [u8; 32],
    
    /// Number of pages processed so far
    pub pages_processed: u32,
    
    /// Number of failed payouts (for retry tracking)
    pub failed_payouts_count: u32,
    
    /// Reserved for future use
    pub reserved: [u8; 20],
}

impl DailyDistributionState {
    pub const INIT_SPACE: usize = 8 +   // distribution_day
                                   32 +  // quote_mint
                                   32 +  // treasury_ata
                                   8 +   // total_amount_to_distribute
                                   8 +   // amount_distributed
                                   4 +   // current_cursor
                                   4 +   // total_investors
                                   4 +   // investors_processed
                                   1 +   // is_complete
                                   8 +   // started_at
                                   8 +   // completed_at
                                   8 +   // dust_carried_over
                                   8 +   // daily_cap_total
                                   8 +   // daily_cap_remaining
                                   8 +   // min_payout_threshold
                                   8 +   // initial_total_deposit
                                   8 +   // investor_fee_share_bps
                                   32 +  // last_page_hash
                                   4 +   // pages_processed
                                   4 +   // failed_payouts_count
                                   20;   // reserved

    /// Derive the PDA for daily distribution state
    pub fn derive_pda(distribution_day: i64, quote_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"daily_distribution",
                distribution_day.to_string().as_bytes(),
                quote_mint.as_ref(),
            ],
            program_id,
        )
    }

    /// Get the day timestamp (start of day) from a given timestamp
    pub fn get_day_start(timestamp: i64) -> i64 {
        // Round down to start of day (86400 seconds = 24 hours)
        (timestamp / 86400) * 86400
    }

    /// Check if enough time has passed since last distribution
    pub fn can_start_new_distribution(last_distribution_day: i64, current_timestamp: i64) -> bool {
        let current_day = Self::get_day_start(current_timestamp);
        current_day > last_distribution_day
    }

    /// Update progress after processing a page of investors
    pub fn update_progress(&mut self, investors_processed: u32, amount_distributed: u64, new_cursor: u32) {
        self.investors_processed = self.investors_processed.saturating_add(investors_processed);
        self.amount_distributed = self.amount_distributed.saturating_add(amount_distributed);
        self.current_cursor = new_cursor;
    }

    /// Mark distribution as complete
    pub fn mark_complete(&mut self, timestamp: i64) {
        self.is_complete = true;
        self.completed_at = timestamp;
    }

    /// Calculate remaining amount to distribute
    pub fn remaining_amount(&self) -> u64 {
        self.total_amount_to_distribute.saturating_sub(self.amount_distributed)
    }

    /// Check if there are more investors to process
    pub fn has_more_investors(&self) -> bool {
        self.investors_processed < self.total_investors
    }

    /// Update daily cap after distribution
    pub fn update_daily_cap(&mut self, amount_distributed: u64) {
        self.daily_cap_remaining = self.daily_cap_remaining.saturating_sub(amount_distributed);
    }

    /// Add dust to carry over
    pub fn add_dust(&mut self, dust_amount: u64) {
        self.dust_carried_over = self.dust_carried_over.saturating_add(dust_amount);
    }

    /// Check if daily cap allows for distribution
    pub fn can_distribute(&self, amount: u64) -> bool {
        amount <= self.daily_cap_remaining
    }

    /// Get effective distribution amount (including carried over dust)
    pub fn get_effective_distribution_amount(&self) -> u64 {
        self.total_amount_to_distribute.saturating_add(self.dust_carried_over)
    }

    /// Calculate hash for a page of investor accounts (for idempotency)
    pub fn calculate_page_hash(investor_accounts: &[Pubkey]) -> [u8; 32] {
        use anchor_lang::solana_program::hash::hash;
        
        let mut data = Vec::new();
        for account in investor_accounts {
            data.extend_from_slice(account.as_ref());
        }
        
        let hash_result = hash(&data);
        hash_result.to_bytes()
    }

    /// Check if this page has already been processed (idempotency check)
    pub fn is_page_already_processed(&self, page_hash: &[u8; 32]) -> bool {
        self.last_page_hash == *page_hash
    }

    /// Update page processing state
    pub fn update_page_state(&mut self, page_hash: [u8; 32], investors_in_page: u32, amount_distributed: u64) {
        self.last_page_hash = page_hash;
        self.pages_processed = self.pages_processed.saturating_add(1);
        self.update_progress(investors_in_page, amount_distributed, self.current_cursor + investors_in_page);
    }

    /// Track failed payouts for retry purposes
    pub fn add_failed_payouts(&mut self, failed_count: u32) {
        self.failed_payouts_count = self.failed_payouts_count.saturating_add(failed_count);
    }

    /// Check if there were any failed payouts that might need retry
    pub fn has_failed_payouts(&self) -> bool {
        self.failed_payouts_count > 0
    }

    /// Validate page for retry safety
    pub fn validate_page_for_retry(&self, investor_accounts: &[Pubkey]) -> Result<()> {
        // Calculate hash for this page
        let page_hash = Self::calculate_page_hash(investor_accounts);
        
        // If this exact page was already processed, it's a retry attempt
        if self.is_page_already_processed(&page_hash) {
            msg!("Page already processed - idempotency violation detected");
            return Err(anchor_lang::error::ErrorCode::ConstraintRaw.into());
        }
        
        Ok(())
    }
}

/// Global distribution state to track the last distribution day
#[account]
pub struct GlobalDistributionState {
    /// Quote mint this global state tracks
    pub quote_mint: Pubkey,
    
    /// Last distribution day (Unix timestamp of day start)
    pub last_distribution_day: i64,
    
    /// Total number of distributions completed
    pub total_distributions: u64,
    
    /// Total amount distributed across all time
    pub total_amount_distributed: u64,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl GlobalDistributionState {
    pub const INIT_SPACE: usize = 32 +  // quote_mint
                                   8 +   // last_distribution_day
                                   8 +   // total_distributions
                                   8 +   // total_amount_distributed
                                   64;   // reserved

    /// Derive the PDA for global distribution state
    pub fn derive_pda(quote_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"global_distribution", quote_mint.as_ref()],
            program_id,
        )
    }

    /// Update after completing a daily distribution
    pub fn update_after_distribution(&mut self, distribution_day: i64, amount_distributed: u64) {
        self.last_distribution_day = distribution_day;
        self.total_distributions = self.total_distributions.saturating_add(1);
        self.total_amount_distributed = self.total_amount_distributed.saturating_add(amount_distributed);
    }
}
