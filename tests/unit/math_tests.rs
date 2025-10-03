use meteora_fee_router::integrations::streamflow::calculations::*;
use meteora_fee_router::integrations::streamflow::accounts::InvestorStreamData;
use meteora_fee_router::integrations::streamflow::cpi::calculate_locked_fraction;
use anchor_lang::prelude::*;

#[cfg(test)]
mod mathematical_tests {
    use super::*;

    #[test]
    fn test_locked_fraction_calculation() {
        // Test normal case
        let locked_total = 500_000u64;
        let initial_total_deposit = 1_000_000u64;
        let fraction = calculate_locked_fraction(locked_total, initial_total_deposit);
        assert_eq!(fraction, 5000); // 50% = 5000 basis points

        // Test 100% locked
        let fraction_full = calculate_locked_fraction(1_000_000, 1_000_000);
        assert_eq!(fraction_full, 10000); // 100% = 10000 basis points

        // Test 0% locked
        let fraction_zero = calculate_locked_fraction(0, 1_000_000);
        assert_eq!(fraction_zero, 0); // 0% = 0 basis points

        // Test edge case: zero initial deposit
        let fraction_edge = calculate_locked_fraction(100, 0);
        assert_eq!(fraction_edge, 0); // Should handle division by zero
    }

    #[test]
    fn test_pro_rata_distribution_calculation() {
        // Create test investor data
        let investor1 = Pubkey::new_unique();
        let investor2 = Pubkey::new_unique();
        let investor3 = Pubkey::new_unique();
        
        let investors = vec![
            InvestorStreamData {
                investor: investor1,
                stream_account: Pubkey::new_unique(),
                locked_amount: 300_000, // 30% of total
                total_deposited: 500_000,
                investor_ata: Pubkey::new_unique(),
            },
            InvestorStreamData {
                investor: investor2,
                stream_account: Pubkey::new_unique(),
                locked_amount: 500_000, // 50% of total
                total_deposited: 800_000,
                investor_ata: Pubkey::new_unique(),
            },
            InvestorStreamData {
                investor: investor3,
                stream_account: Pubkey::new_unique(),
                locked_amount: 200_000, // 20% of total
                total_deposited: 300_000,
                investor_ata: Pubkey::new_unique(),
            },
        ];

        let total_locked = 1_000_000u64;
        let claimed_quote = 10_000u64;
        let initial_total_deposit = 2_000_000u64;
        let investor_fee_share_bps = 5000u64; // 50%
        let min_payout_lamports = 100u64;

        // Calculate distribution
        let result = calculate_distribution(
            claimed_quote,
            &investors,
            total_locked,
            initial_total_deposit,
            investor_fee_share_bps,
            min_payout_lamports,
        ).unwrap();

        // Verify locked fraction: 1M / 2M = 50% = 5000 bps
        // Eligible share: min(5000, 5000) = 5000 bps
        // Investor fee: 10000 * 5000 / 10000 = 5000
        assert_eq!(result.investor_fee_quote, 5000);

        // Verify individual payouts are proportional
        let payout1 = result.investor_payouts.iter().find(|p| p.investor == investor1).unwrap();
        let payout2 = result.investor_payouts.iter().find(|p| p.investor == investor2).unwrap();
        let payout3 = result.investor_payouts.iter().find(|p| p.investor == investor3).unwrap();

        // Investor 1: 300k/1M * 5000 = 1500
        assert_eq!(payout1.payout_amount, 1500);
        assert_eq!(payout1.weight_bps, 3000); // 30% = 3000 bps

        // Investor 2: 500k/1M * 5000 = 2500  
        assert_eq!(payout2.payout_amount, 2500);
        assert_eq!(payout2.weight_bps, 5000); // 50% = 5000 bps

        // Investor 3: 200k/1M * 5000 = 1000
        assert_eq!(payout3.payout_amount, 1000);
        assert_eq!(payout3.weight_bps, 2000); // 20% = 2000 bps

        // Verify total distributed
        assert_eq!(result.total_distributed, 5000);

        // Verify creator remainder
        assert_eq!(result.creator_remainder, 5000); // 10000 - 5000
    }

    #[test]
    fn test_floor_division_dust_calculation() {
        // Test case where division creates dust
        let investors = vec![
            InvestorStreamData {
                investor: Pubkey::new_unique(),
                stream_account: Pubkey::new_unique(),
                locked_amount: 333_333, // 1/3 of total
                total_deposited: 333_333,
                investor_ata: Pubkey::new_unique(),
            },
            InvestorStreamData {
                investor: Pubkey::new_unique(),
                stream_account: Pubkey::new_unique(),
                locked_amount: 333_333, // 1/3 of total
                total_deposited: 333_333,
                investor_ata: Pubkey::new_unique(),
            },
            InvestorStreamData {
                investor: Pubkey::new_unique(),
                stream_account: Pubkey::new_unique(),
                locked_amount: 333_334, // 1/3 of total (with remainder)
                total_deposited: 333_334,
                investor_ata: Pubkey::new_unique(),
            },
        ];

        let total_locked = 1_000_000u64;
        let claimed_quote = 100u64; // Small amount to create dust
        let result = calculate_distribution(
            claimed_quote,
            &investors,
            total_locked,
            1_000_000,
            10000, // 100% to investors
            1,
        ).unwrap();

        // With 100 tokens and 3 equal investors, each should get 33 (floor division)
        // This creates 1 token of dust (100 - 33*3 = 1)
        let total_payouts: u64 = result.investor_payouts.iter().map(|p| p.payout_amount).sum();
        assert_eq!(result.dust_amount, 100 - total_payouts);
        assert!(result.dust_amount > 0); // Should have some dust
    }

    #[test]
    fn test_minimum_payout_threshold() {
        // Test case where some payouts are below minimum
        let investors = vec![
            InvestorStreamData {
                investor: Pubkey::new_unique(),
                stream_account: Pubkey::new_unique(),
                locked_amount: 1, // Very small amount
                total_deposited: 1,
                investor_ata: Pubkey::new_unique(),
            },
            InvestorStreamData {
                investor: Pubkey::new_unique(),
                stream_account: Pubkey::new_unique(),
                locked_amount: 999_999, // Most of the total
                total_deposited: 999_999,
                investor_ata: Pubkey::new_unique(),
            },
        ];

        let result = calculate_distribution(
            1000,
            &investors,
            1_000_000,
            1_000_000,
            10000,
            100, // High minimum threshold
        ).unwrap();

        // First investor should not meet minimum
        let small_payout = result.investor_payouts.iter().find(|p| p.payout_amount < 100).unwrap();
        assert!(!small_payout.meets_minimum);

        // Second investor should meet minimum
        let large_payout = result.investor_payouts.iter().find(|p| p.payout_amount >= 100).unwrap();
        assert!(large_payout.meets_minimum);
    }

    #[test]
    fn test_edge_case_all_unlocked() {
        // Test scenario where all tokens are unlocked (100% to creator)
        let investors = vec![
            InvestorStreamData {
                investor: Pubkey::new_unique(),
                stream_account: Pubkey::new_unique(),
                locked_amount: 0, // All unlocked
                total_deposited: 1_000_000,
                investor_ata: Pubkey::new_unique(),
            },
        ];

        let result = calculate_distribution(
            10_000,
            &investors,
            0, // No locked tokens
            1_000_000,
            5000, // 50% max to investors
            100,
        ).unwrap();

        // Should be 0 to investors, all to creator
        assert_eq!(result.investor_fee_quote, 0);
        assert_eq!(result.total_distributed, 0);
        assert_eq!(result.creator_remainder, 10_000);
    }

    #[test]
    fn test_weight_calculation() {
        let investor_data = InvestorStreamData {
            investor: Pubkey::new_unique(),
            stream_account: Pubkey::new_unique(),
            locked_amount: 250_000,
            total_deposited: 500_000,
            investor_ata: Pubkey::new_unique(),
        };

        let total_locked = 1_000_000u64;
        let weight = investor_data.calculate_weight(total_locked);
        
        // 250k / 1M = 25% = 2500 basis points
        assert_eq!(weight, 2500);

        // Test edge case: zero total locked
        let weight_zero = investor_data.calculate_weight(0);
        assert_eq!(weight_zero, 0);
    }

    #[test]
    fn test_payout_calculation() {
        let investor_data = InvestorStreamData {
            investor: Pubkey::new_unique(),
            stream_account: Pubkey::new_unique(),
            locked_amount: 300_000,
            total_deposited: 500_000,
            investor_ata: Pubkey::new_unique(),
        };

        let total_locked = 1_000_000u64;
        let investor_fee_quote = 5_000u64;
        
        let payout = investor_data.calculate_payout(total_locked, investor_fee_quote);
        
        // 300k / 1M * 5000 = 1500
        assert_eq!(payout, 1500);

        // Test edge cases
        assert_eq!(investor_data.calculate_payout(0, investor_fee_quote), 0);
        assert_eq!(investor_data.calculate_payout(total_locked, 0), 0);
    }
}
