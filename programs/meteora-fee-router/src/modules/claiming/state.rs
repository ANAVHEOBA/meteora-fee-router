use anchor_lang::prelude::*;

/// Treasury state account to track fee claiming
/// 
/// This account tracks the total fees claimed and provides
/// accounting for the treasury balance.
#[account]
pub struct TreasuryState {
    /// The quote mint this treasury manages
    pub quote_mint: Pubkey,
    
    /// The treasury ATA (Associated Token Account)
    pub treasury_ata: Pubkey,
    
    /// Total fees claimed since inception
    pub total_fees_claimed: u64,
    
    /// Last claim timestamp
    pub last_claim_timestamp: i64,
    
    /// Number of successful claims
    pub claim_count: u64,
    
    /// Authority that can claim fees (should be position owner PDA)
    pub claim_authority: Pubkey,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl TreasuryState {
    pub const INIT_SPACE: usize = 32 + // quote_mint
                                   32 + // treasury_ata
                                   8 +  // total_fees_claimed
                                   8 +  // last_claim_timestamp
                                   8 +  // claim_count
                                   32 + // claim_authority
                                   64;  // reserved

    /// Derive the PDA for treasury state
    pub fn derive_pda(quote_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"treasury_state", quote_mint.as_ref()],
            program_id,
        )
    }

    /// Update state after a successful claim
    pub fn record_claim(&mut self, amount_claimed: u64, timestamp: i64) {
        self.total_fees_claimed = self.total_fees_claimed.saturating_add(amount_claimed);
        self.last_claim_timestamp = timestamp;
        self.claim_count = self.claim_count.saturating_add(1);
    }

    /// Check if enough time has passed since last claim
    pub fn can_claim(&self, current_timestamp: i64, min_interval_seconds: i64) -> bool {
        current_timestamp >= self.last_claim_timestamp + min_interval_seconds
    }
}
