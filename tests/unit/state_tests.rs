use meteora_fee_router::modules::distribution::state::{DailyDistributionState, PolicyState};
use meteora_fee_router::integrations::streamflow::accounts::StreamflowStream;
use anchor_lang::prelude::*;

#[cfg(test)]
mod state_transition_tests {
    use super::*;

    #[test]
    fn test_daily_distribution_state_initialization() {
        let quote_mint = Pubkey::new_unique();
        let treasury_ata = Pubkey::new_unique();
        let distribution_day = 1672531200i64;
        
        let state = DailyDistributionState {
            distribution_day,
            quote_mint,
            treasury_ata,
            total_amount_to_distribute: 100_000,
            amount_distributed: 0,
            current_cursor: 0,
            total_investors: 50,
            investors_processed: 0,
            is_complete: false,
            started_at: distribution_day,
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
        };

        // Test initial state
        assert!(!state.is_complete);
        assert_eq!(state.amount_distributed, 0);
        assert_eq!(state.current_cursor, 0);
        assert_eq!(state.investors_processed, 0);
        assert!(state.has_more_investors());
        assert_eq!(state.remaining_amount(), 100_000);
    }

    #[test]
    fn test_distribution_progress_updates() {
        let mut state = create_test_daily_state();
        
        // Test progress update
        let page_hash = [1u8; 32];
        state.update_page_state(page_hash, 10, 5_000);
        
        assert_eq!(state.investors_processed, 10);
        assert_eq!(state.amount_distributed, 5_000);
        assert_eq!(state.current_cursor, 10);
        assert_eq!(state.pages_processed, 1);
        assert_eq!(state.last_page_hash, page_hash);
        assert_eq!(state.remaining_amount(), 95_000);
        
        // Test second page
        let page_hash2 = [2u8; 32];
        state.update_page_state(page_hash2, 15, 7_500);
        
        assert_eq!(state.investors_processed, 25);
        assert_eq!(state.amount_distributed, 12_500);
        assert_eq!(state.current_cursor, 25);
        assert_eq!(state.pages_processed, 2);
        assert_eq!(state.last_page_hash, page_hash2);
    }

    #[test]
    fn test_daily_cap_management() {
        let mut state = create_test_daily_state();
        
        // Test cap checking
        assert!(state.can_distribute(500_000));
        assert!(!state.can_distribute(1_500_000)); // Exceeds cap
        
        // Test cap updates
        state.update_daily_cap(300_000);
        assert_eq!(state.daily_cap_remaining, 700_000);
        
        state.update_daily_cap(700_000);
        assert_eq!(state.daily_cap_remaining, 0);
        assert!(!state.can_distribute(1)); // No cap remaining
    }

    #[test]
    fn test_dust_management() {
        let mut state = create_test_daily_state();
        
        // Test dust accumulation
        state.add_dust(150);
        assert_eq!(state.dust_carried_over, 150);
        
        state.add_dust(75);
        assert_eq!(state.dust_carried_over, 225);
        
        // Test effective distribution amount includes dust
        let effective = state.get_effective_distribution_amount();
        assert_eq!(effective, 100_000 + 225);
    }

    #[test]
    fn test_failed_payout_tracking() {
        let mut state = create_test_daily_state();
        
        // Test failed payout tracking
        assert!(!state.has_failed_payouts());
        
        state.add_failed_payouts(3);
        assert!(state.has_failed_payouts());
        assert_eq!(state.failed_payouts_count, 3);
        
        state.add_failed_payouts(2);
        assert_eq!(state.failed_payouts_count, 5);
    }

    #[test]
    fn test_completion_state_transitions() {
        let mut state = create_test_daily_state();
        let completion_time = 1672617600i64;
        
        // Process all investors
        state.update_page_state([1u8; 32], 50, 50_000);
        assert!(!state.has_more_investors());
        
        // Mark as complete
        state.mark_complete(completion_time);
        assert!(state.is_complete);
        assert_eq!(state.completed_at, completion_time);
    }

    #[test]
    fn test_idempotency_page_validation() {
        let state = create_test_daily_state();
        let investor_accounts = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        
        // Test page hash calculation
        let hash1 = DailyDistributionState::calculate_page_hash(&investor_accounts);
        let hash2 = DailyDistributionState::calculate_page_hash(&investor_accounts);
        assert_eq!(hash1, hash2); // Should be deterministic
        
        // Test different accounts produce different hashes
        let different_accounts = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let hash3 = DailyDistributionState::calculate_page_hash(&different_accounts);
        assert_ne!(hash1, hash3);
        
        // Test page not already processed
        assert!(!state.is_page_already_processed(&hash1));
        
        // Test validation passes for new page
        assert!(state.validate_page_for_retry(&investor_accounts).is_ok());
    }

    #[test]
    fn test_policy_state_validation() {
        let mut policy = PolicyState {
            quote_mint: Pubkey::new_unique(),
            investor_fee_share_bps: 5000,
            daily_cap_lamports: 1_000_000,
            min_payout_lamports: 1000,
            y0_total_allocation: 2_000_000,
            policy_authority: Pubkey::new_unique(),
            reserved: [0; 64],
        };
        
        // Test valid policy
        assert!(policy.validate().is_ok());
        
        // Test invalid fee share (> 10000)
        policy.investor_fee_share_bps = 15000;
        assert!(policy.validate().is_err());
        
        // Reset and test zero allocation
        policy.investor_fee_share_bps = 5000;
        policy.y0_total_allocation = 0;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_streamflow_stream_calculations() {
        let current_time = 1672531200u64; // Jan 1, 2023
        let stream = StreamflowStream {
            magic: 0,
            version: 1,
            created_at: current_time - 86400, // Created 1 day ago
            start_time: current_time - 3600,  // Started 1 hour ago
            end_time: current_time + 86400,   // Ends in 1 day
            deposited_amount: 100_000,
            withdrawn_amount: 0,
            recipient: Pubkey::new_unique(),
            sender: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            escrow_tokens: Pubkey::new_unique(),
            name: [0; 64],
            can_cancel: true,
            can_transfer: false,
            cancelled: false,
            metadata: [0; 128],
        };
        
        // Test unlocked amount calculation
        // Stream duration: 86400 + 3600 = 90000 seconds
        // Elapsed: 3600 seconds
        // Unlocked: 100000 * 3600 / 90000 = 4000
        let unlocked = stream.unlocked_amount(current_time);
        assert_eq!(unlocked, 4000);
        
        // Test locked amount
        let locked = stream.locked_amount(current_time);
        assert_eq!(locked, 96_000);
        
        // Test withdrawable amount
        let withdrawable = stream.withdrawable_amount(current_time);
        assert_eq!(withdrawable, 4000); // No withdrawals yet
        
        // Test stream is active
        assert!(stream.is_active(current_time));
        assert!(!stream.is_fully_vested(current_time));
        
        // Test fully vested scenario
        let future_time = current_time + 90000;
        assert_eq!(stream.unlocked_amount(future_time), 100_000);
        assert_eq!(stream.locked_amount(future_time), 0);
        assert!(stream.is_fully_vested(future_time));
    }

    // Helper function to create test state
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
