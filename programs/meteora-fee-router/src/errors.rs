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
}
