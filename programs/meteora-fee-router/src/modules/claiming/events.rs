use anchor_lang::prelude::*;

/// Event emitted when fees are successfully claimed from the position
#[event]
pub struct FeesClaimedFromPosition {
    /// The position that fees were claimed from
    pub position: Pubkey,
    
    /// The pool the position belongs to
    pub pool: Pubkey,
    
    /// Amount of quote tokens claimed
    pub quote_amount_claimed: u64,
    
    /// Amount of base tokens claimed (should be 0 for quote-only)
    pub base_amount_claimed: u64,
    
    /// The treasury ATA that received the fees
    pub treasury_ata: Pubkey,
    
    /// Quote mint
    pub quote_mint: Pubkey,
    
    /// Timestamp of the claim
    pub timestamp: i64,
    
    /// Total fees claimed to date
    pub total_fees_claimed: u64,
}

/// Event emitted when treasury state is initialized
#[event]
pub struct TreasuryInitialized {
    /// The quote mint this treasury manages
    pub quote_mint: Pubkey,
    
    /// The treasury ATA address
    pub treasury_ata: Pubkey,
    
    /// The claim authority (position owner PDA)
    pub claim_authority: Pubkey,
    
    /// Timestamp of initialization
    pub timestamp: i64,
}
