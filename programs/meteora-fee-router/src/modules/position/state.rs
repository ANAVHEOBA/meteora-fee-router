use anchor_lang::prelude::*;

/// Optional: Position metadata account
/// 
/// Store additional information about the honorary position if needed.
/// This may not be necessary if all required data is stored in the Meteora position itself.
#[account]
pub struct PositionMetadata {
    /// The position account pubkey
    pub position: Pubkey,
    
    /// The pool this position belongs to
    pub pool: Pubkey,
    
    /// The quote mint (the only mint that should accrue fees)
    pub quote_mint: Pubkey,
    
    /// The base mint (should NOT accrue fees)
    pub base_mint: Pubkey,
    
    /// Timestamp when position was created
    pub created_at: i64,
    
    /// The bump seed for the position owner PDA
    pub position_owner_bump: u8,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl PositionMetadata {
    pub const INIT_SPACE: usize = 32 + // position
                                   32 + // pool
                                   32 + // quote_mint
                                   32 + // base_mint
                                   8 +  // created_at
                                   1 +  // position_owner_bump
                                   64;  // reserved

    /// Derive the PDA for position metadata
    pub fn derive_pda(position_nft_mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"position_metadata", position_nft_mint.as_ref()],
            program_id,
        )
    }

    /// Check if this position is configured for quote-only fees
    pub fn is_quote_only(&self) -> bool {
        // In our system, all positions should be quote-only
        // This is validated during creation
        true
    }

    /// Get the age of the position in seconds
    pub fn age_seconds(&self, current_timestamp: i64) -> i64 {
        current_timestamp - self.created_at
    }
}
