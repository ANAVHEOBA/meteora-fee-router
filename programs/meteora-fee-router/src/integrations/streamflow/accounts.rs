use anchor_lang::prelude::*;

/// Streamflow Stream Account Structure
/// 
/// This represents the on-chain data structure for a Streamflow stream.
/// Based on the Streamflow protocol specification.
#[account]
pub struct StreamflowStream {
    /// Magic number to identify stream accounts
    pub magic: u64,
    
    /// Version of the stream account
    pub version: u64,
    
    /// Timestamp when the stream was created
    pub created_at: u64,
    
    /// Timestamp when the stream becomes active
    pub start_time: u64,
    
    /// Timestamp when the stream ends
    pub end_time: u64,
    
    /// Total amount deposited in the stream
    pub deposited_amount: u64,
    
    /// Amount already withdrawn from the stream
    pub withdrawn_amount: u64,
    
    /// The recipient of the stream (investor)
    pub recipient: Pubkey,
    
    /// The sender/creator of the stream
    pub sender: Pubkey,
    
    /// The mint of the token being streamed
    pub mint: Pubkey,
    
    /// The escrow token account holding the funds
    pub escrow_tokens: Pubkey,
    
    /// Stream name/identifier
    pub name: [u8; 64],
    
    /// Whether the stream can be cancelled
    pub can_cancel: bool,
    
    /// Whether the stream can be transferred
    pub can_transfer: bool,
    
    /// Whether the stream has been cancelled
    pub cancelled: bool,
    
    /// Additional metadata
    pub metadata: [u8; 128],
}

impl StreamflowStream {
    /// Calculate the amount that should be unlocked at a given timestamp
    pub fn unlocked_amount(&self, current_timestamp: u64) -> u64 {
        if current_timestamp < self.start_time {
            // Stream hasn't started yet
            return 0;
        }
        
        if current_timestamp >= self.end_time {
            // Stream has fully vested
            return self.deposited_amount;
        }
        
        // Linear vesting calculation
        let elapsed_time = current_timestamp - self.start_time;
        let total_duration = self.end_time - self.start_time;
        
        if total_duration == 0 {
            return self.deposited_amount;
        }
        
        // Calculate proportional unlock
        let unlocked = (self.deposited_amount as u128 * elapsed_time as u128) / total_duration as u128;
        unlocked as u64
    }
    
    /// Calculate the amount still locked at a given timestamp
    pub fn locked_amount(&self, current_timestamp: u64) -> u64 {
        let unlocked = self.unlocked_amount(current_timestamp);
        self.deposited_amount.saturating_sub(unlocked)
    }
    
    /// Calculate the amount available for withdrawal (unlocked - withdrawn)
    pub fn withdrawable_amount(&self, current_timestamp: u64) -> u64 {
        let unlocked = self.unlocked_amount(current_timestamp);
        unlocked.saturating_sub(self.withdrawn_amount)
    }
    
    /// Check if the stream is active at a given timestamp
    pub fn is_active(&self, current_timestamp: u64) -> bool {
        !self.cancelled && 
        current_timestamp >= self.start_time && 
        current_timestamp < self.end_time
    }
    
    /// Check if the stream has fully vested
    pub fn is_fully_vested(&self, current_timestamp: u64) -> bool {
        current_timestamp >= self.end_time
    }
}

/// Helper struct for investor stream data
#[derive(Debug, Clone)]
pub struct InvestorStreamData {
    /// The investor's wallet address
    pub investor: Pubkey,
    
    /// The stream account address
    pub stream_account: Pubkey,
    
    /// Amount still locked in the stream
    pub locked_amount: u64,
    
    /// Total deposited amount (for reference)
    pub total_deposited: u64,
    
    /// The investor's ATA for receiving payouts
    pub investor_ata: Pubkey,
}

impl InvestorStreamData {
    /// Calculate the investor's weight in the distribution
    pub fn calculate_weight(&self, total_locked: u64) -> u64 {
        if total_locked == 0 {
            return 0;
        }
        
        // Weight as basis points (out of 10000)
        // weight = (locked_amount / total_locked) * 10000
        ((self.locked_amount as u128 * 10000u128) / total_locked as u128) as u64
    }
    
    /// Calculate payout amount based on weight and total investor fees
    pub fn calculate_payout(&self, total_locked: u64, investor_fee_quote: u64) -> u64 {
        if total_locked == 0 || investor_fee_quote == 0 {
            return 0;
        }
        
        // payout = floor(investor_fee_quote * locked_amount / total_locked)
        ((investor_fee_quote as u128 * self.locked_amount as u128) / total_locked as u128) as u64
    }
}
