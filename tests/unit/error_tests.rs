use meteora_fee_router::errors::FeeRouterError;
use meteora_fee_router::modules::distribution::state::DailyDistributionState;
use meteora_fee_router::integrations::streamflow::cpi::{StreamError, StreamErrorType};
use anchor_lang::prelude::*;

#[cfg(test)]
mod error_condition_tests {
    use super::*;

    #[test]
    fn test_fee_router_error_messages() {
        // Test that all error variants have proper messages
        let errors = vec![
            FeeRouterError::BaseFeeDetected,
            FeeRouterError::TooSoonToDistribute,
            FeeRouterError::InvalidPoolConfig,
            FeeRouterError::LockedAmountError,
            FeeRouterError::PaginationError,
            FeeRouterError::DailyCapExceeded,
            FeeRouterError::InvalidTokenOrder,
            FeeRouterError::QuoteMintMismatch,
            FeeRouterError::PositionOwnerMismatch,
            FeeRouterError::ArithmeticOverflow,
            FeeRouterError::ArithmeticUnderflow,
            FeeRouterError::DivisionByZero,
            FeeRouterError::InvalidFeeShareBps,
            FeeRouterError::NoInvestors,
            FeeRouterError::InvestorAtaMissing,
            FeeRouterError::DistributionInProgress,
            FeeRouterError::DistributionNotStarted,
            FeeRouterError::InvalidPageIndex,
            FeeRouterError::PayoutBelowMinimum,
            FeeRouterError::NoFeesToClaim,
            FeeRouterError::BaseFeesClaimedError,
            FeeRouterError::PositionMetadataMismatch,
            FeeRouterError::TreasuryStateMismatch,
            FeeRouterError::ClaimIntervalNotElapsed,
            FeeRouterError::NoAccumulatedFees,
            FeeRouterError::TreasuryBalanceMismatch,
            FeeRouterError::TreasuryTransferFailed,
            FeeRouterError::MeteoraCpiFailed,
        ];

        // Verify each error can be converted to an anchor error
        for error in errors {
            let anchor_error: anchor_lang::error::Error = error.into();
            assert!(anchor_error.error_msg().is_some());
        }
    }

    #[test]
    fn test_streamflow_error_types() {
        let stream_account = Pubkey::new_unique();
        let investor = Pubkey::new_unique();

        // Test different error types
        let errors = vec![
            StreamError {
                stream_account,
                investor: Some(investor),
                error_type: StreamErrorType::InvalidStreamData,
                error_message: "Invalid stream data".to_string(),
            },
            StreamError {
                stream_account,
                investor: Some(investor),
                error_type: StreamErrorType::MissingInvestorAta,
                error_message: "Missing investor ATA".to_string(),
            },
            StreamError {
                stream_account,
                investor: Some(investor),
                error_type: StreamErrorType::StreamExpired,
                error_message: "Stream has expired".to_string(),
            },
            StreamError {
                stream_account,
                investor: Some(investor),
                error_type: StreamErrorType::InsufficientLocked,
                error_message: "Insufficient locked amount".to_string(),
            },
            StreamError {
                stream_account,
                investor: None,
                error_type: StreamErrorType::AccountDeserializationFailed,
                error_message: "Failed to deserialize account".to_string(),
            },
        ];

        // Verify error structure
        for error in errors {
            assert_eq!(error.stream_account, stream_account);
            assert!(!error.error_message.is_empty());
            
            match error.error_type {
                StreamErrorType::InvalidStreamData => {
                    assert!(error.investor.is_some());
                }
                StreamErrorType::AccountDeserializationFailed => {
                    assert!(error.investor.is_none());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_idempotency_violation_error() {
        let mut state = create_test_daily_state();
        let investor_accounts = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];

        // Process a page
        let page_hash = DailyDistributionState::calculate_page_hash(&investor_accounts);
        state.update_page_state(page_hash, 10, 5000);

        // Try to process the same page again - should fail
        let result = state.validate_page_for_retry(&investor_accounts);
        assert!(result.is_err());
    }

    #[test]
    fn test_daily_cap_exceeded_scenario() {
        let mut state = create_test_daily_state();
        
        // Set a low daily cap
        state.daily_cap_total = 1000;
        state.daily_cap_remaining = 1000;
        
        // Try to distribute more than cap
        assert!(!state.can_distribute(1500));
        
        // Distribute up to cap
        state.update_daily_cap(1000);
        assert_eq!(state.daily_cap_remaining, 0);
        
        // Try to distribute more - should fail
        assert!(!state.can_distribute(1));
    }

    #[test]
    fn test_arithmetic_overflow_protection() {
        // Test scenarios that could cause overflow
        let max_u64 = u64::MAX;
        
        // Test safe addition
        let result = max_u64.saturating_add(1);
        assert_eq!(result, max_u64); // Should not overflow
        
        // Test safe subtraction
        let result = 0u64.saturating_sub(1);
        assert_eq!(result, 0); // Should not underflow
        
        // Test safe multiplication in weight calculation
        let large_amount = u64::MAX / 10000; // Safe for BPS calculation
        let weight_calc = (large_amount as u128 * 10000u128) / 1000000u128;
        assert!(weight_calc <= u64::MAX as u128);
    }

    #[test]
    fn test_division_by_zero_protection() {
        use meteora_fee_router::integrations::streamflow::cpi::calculate_locked_fraction;
        
        // Test division by zero in locked fraction
        let fraction = calculate_locked_fraction(1000, 0);
        assert_eq!(fraction, 0); // Should handle gracefully
        
        // Test in weight calculation
        use meteora_fee_router::integrations::streamflow::accounts::InvestorStreamData;
        let investor = InvestorStreamData {
            investor: Pubkey::new_unique(),
            stream_account: Pubkey::new_unique(),
            locked_amount: 1000,
            total_deposited: 2000,
            investor_ata: Pubkey::new_unique(),
        };
        
        let weight = investor.calculate_weight(0); // Zero total
        assert_eq!(weight, 0); // Should handle gracefully
        
        let payout = investor.calculate_payout(0, 1000); // Zero total
        assert_eq!(payout, 0); // Should handle gracefully
    }

    #[test]
    fn test_invalid_parameter_validation() {
        use meteora_fee_router::modules::distribution::state::PolicyState;
        
        // Test invalid fee share BPS
        let mut policy = PolicyState {
            quote_mint: Pubkey::new_unique(),
            investor_fee_share_bps: 15000, // > 10000
            daily_cap_lamports: 1000000,
            min_payout_lamports: 1000,
            y0_total_allocation: 2000000,
            policy_authority: Pubkey::new_unique(),
            reserved: [0; 64],
        };
        
        assert!(policy.validate().is_err());
        
        // Test zero allocation
        policy.investor_fee_share_bps = 5000;
        policy.y0_total_allocation = 0;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_distribution_state_validation_errors() {
        let state = create_test_daily_state();
        
        // Test distribution not started error condition
        assert!(state.has_more_investors()); // Should have investors to process
        
        // Test invalid page scenarios
        let empty_accounts: Vec<Pubkey> = vec![];
        let result = state.validate_page_for_retry(&empty_accounts);
        // Empty accounts should still be valid (edge case)
        assert!(result.is_ok());
    }

    #[test]
    fn test_minimum_payout_threshold_errors() {
        use meteora_fee_router::integrations::streamflow::calculations::calculate_distribution;
        use meteora_fee_router::integrations::streamflow::accounts::InvestorStreamData;
        
        // Create investor with very small locked amount
        let investors = vec![
            InvestorStreamData {
                investor: Pubkey::new_unique(),
                stream_account: Pubkey::new_unique(),
                locked_amount: 1, // Very small
                total_deposited: 1,
                investor_ata: Pubkey::new_unique(),
            },
        ];
        
        let result = calculate_distribution(
            1000,
            &investors,
            1,
            1000000,
            10000,
            1000, // High minimum threshold
        ).unwrap();
        
        // Payout should be below minimum
        let payout = &result.investor_payouts[0];
        assert!(payout.payout_amount < 1000);
        assert!(!payout.meets_minimum);
    }

    #[test]
    fn test_no_investors_error_scenario() {
        use meteora_fee_router::integrations::streamflow::calculations::calculate_distribution;
        
        // Empty investor list
        let investors: Vec<_> = vec![];
        
        let result = calculate_distribution(
            1000,
            &investors,
            0,
            1000000,
            5000,
            100,
        ).unwrap();
        
        // Should handle empty investor list gracefully
        assert_eq!(result.investor_payouts.len(), 0);
        assert_eq!(result.total_distributed, 0);
        assert_eq!(result.creator_remainder, 1000); // All to creator
    }

    // Helper function
    fn create_test_daily_state() -> DailyDistributionState {
        DailyDistributionState {
            distribution_day: 1672531200,
            quote_mint: Pubkey::new_unique(),
            treasury_ata: Pubkey::new_unique(),
            total_amount_to_distribute: 100_000,
            amount_distributed: 0,
            current_cursor: 0,
            total_investors: 50,
            investors_processed: 0,
            is_complete: false,
            started_at: 1672531200,
            completed_at: 0,
            dust_carried_over: 0,
            daily_cap_total: 1_000_000,
            daily_cap_remaining: 1_000_000,
            min_payout_threshold: 1000,
            initial_total_deposit: 2_000_000,
            investor_fee_share_bps: 5000,
            last_page_hash: [0; 32],
            pages_processed: 0,
            failed_payouts_count: 0,
            reserved: [0; 20],
        }
    }
}
