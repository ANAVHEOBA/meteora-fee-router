use anchor_lang::prelude::*;

#[error_code]
pub enum FeeRouterError {
    #[msg("Base fees detected - only quote fees are allowed")]
    BaseFeeDetected,
    
    #[msg("24 hour period has not elapsed since last distribution")]
    TooSoonToDistribute,
    
    #[msg("Invalid pool configuration for quote-only fees")]
    InvalidPoolConfig,
    
    #[msg("Failed to calculate locked amount from Streamflow")]
    LockedAmountError,
    
    #[msg("Pagination cursor mismatch")]
    PaginationError,
    
    #[msg("Daily distribution cap exceeded")]
    DailyCapExceeded,
    
    #[msg("Invalid token order in pool")]
    InvalidTokenOrder,
    
    #[msg("Quote mint mismatch")]
    QuoteMintMismatch,
    
    #[msg("Position owner PDA mismatch")]
    PositionOwnerMismatch,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    #[msg("Invalid fee share basis points (must be 0-10000)")]
    InvalidFeeShareBps,
    
    #[msg("No investors to distribute to")]
    NoInvestors,
    
    #[msg("Investor ATA does not exist")]
    InvestorAtaMissing,
    
    #[msg("Distribution already in progress for this day")]
    DistributionInProgress,
    
    #[msg("Distribution not started for this day")]
    DistributionNotStarted,
    
    #[msg("Invalid page index")]
    InvalidPageIndex,
    
    #[msg("Payout below minimum threshold")]
    PayoutBelowMinimum,
    
    // Fee Claiming Errors
    #[msg("No fees available to claim from position")]
    NoFeesToClaim,
    
    #[msg("Base token fees detected during claim (should be quote-only)")]
    BaseFeesClaimedError,
    
    #[msg("Position metadata mismatch with provided accounts")]
    PositionMetadataMismatch,
    
    #[msg("Treasury state mismatch with provided accounts")]
    TreasuryStateMismatch,
    
    #[msg("Claim interval not elapsed - too soon to claim again")]
    ClaimIntervalNotElapsed,
    
    #[msg("Position has no accumulated fees")]
    NoAccumulatedFees,
    
    #[msg("Treasury ATA balance mismatch")]
    TreasuryBalanceMismatch,
    
    #[msg("Failed to transfer claimed fees to treasury")]
    TreasuryTransferFailed,
    
    #[msg("Meteora CPI call failed")]
    MeteoraCpiFailed,
}
