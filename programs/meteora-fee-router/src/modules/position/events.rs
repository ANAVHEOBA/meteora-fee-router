use anchor_lang::prelude::*;

/// Event emitted when the honorary position is successfully initialized
#[event]
pub struct HonoraryPositionInitialized {
    /// The position account pubkey
    pub position: Pubkey,
    
    /// The pool this position belongs to
    pub pool: Pubkey,
    
    /// The quote mint (only mint that will accrue fees)
    pub quote_mint: Pubkey,
    
    /// The base mint (should not accrue fees)
    pub base_mint: Pubkey,
    
    /// The PDA that owns this position
    pub position_owner: Pubkey,
    
    /// Timestamp of initialization
    pub timestamp: i64,
}

/// Event emitted if position initialization fails validation
#[event]
pub struct PositionInitializationFailed {
    /// The pool that failed validation
    pub pool: Pubkey,
    
    /// Reason for failure
    pub reason: String,
    
    /// Timestamp of failure
    pub timestamp: i64,
}
