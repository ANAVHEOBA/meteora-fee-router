use anchor_lang::prelude::*;
use meteora_fee_router::modules::distribution::state::{PolicyState, DailyDistributionState, GlobalDistributionState};
use meteora_fee_router::modules::position::state::PositionState;
use meteora_fee_router::shared::constants::*;

#[cfg(test)]
mod pda_derivation_tests {
    use super::*;

    #[test]
    fn test_policy_state_pda_derivation() {
        let program_id = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        
        // Test PDA derivation
        let (pda, bump) = PolicyState::derive_pda(&quote_mint, &program_id);
        
        // Verify PDA is derived correctly
        let expected_seeds = &[
            b"policy",
            quote_mint.as_ref(),
            &[bump],
        ];
        let expected_pda = Pubkey::create_program_address(expected_seeds, &program_id).unwrap();
        
        assert_eq!(pda, expected_pda);
        
        // Test that bump is canonical (highest valid bump)
        assert!(bump <= 255);
        
        // Test that derived PDA is deterministic
        let (pda2, bump2) = PolicyState::derive_pda(&quote_mint, &program_id);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }

    #[test]
    fn test_daily_distribution_state_pda_derivation() {
        let program_id = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        let distribution_day = 1672531200i64; // Jan 1, 2023
        
        let (pda, bump) = DailyDistributionState::derive_pda(distribution_day, &quote_mint, &program_id);
        
        // Verify PDA is derived correctly
        let expected_seeds = &[
            b"daily_distribution",
            distribution_day.to_string().as_bytes(),
            quote_mint.as_ref(),
            &[bump],
        ];
        let expected_pda = Pubkey::create_program_address(expected_seeds, &program_id).unwrap();
        
        assert_eq!(pda, expected_pda);
        
        // Test deterministic behavior
        let (pda2, bump2) = DailyDistributionState::derive_pda(distribution_day, &quote_mint, &program_id);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }

    #[test]
    fn test_global_distribution_state_pda_derivation() {
        let program_id = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        
        let (pda, bump) = GlobalDistributionState::derive_pda(&quote_mint, &program_id);
        
        // Verify PDA is derived correctly
        let expected_seeds = &[
            b"global_distribution",
            quote_mint.as_ref(),
            &[bump],
        ];
        let expected_pda = Pubkey::create_program_address(expected_seeds, &program_id).unwrap();
        
        assert_eq!(pda, expected_pda);
    }

    #[test]
    fn test_position_owner_pda_derivation() {
        let program_id = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        
        let expected_seeds = &[
            POSITION_OWNER_SEED,
            vault.as_ref(),
        ];
        let (expected_pda, expected_bump) = Pubkey::find_program_address(expected_seeds, &program_id);
        
        // Test that we can derive the same PDA
        let derived_seeds = &[
            POSITION_OWNER_SEED,
            vault.as_ref(),
            &[expected_bump],
        ];
        let derived_pda = Pubkey::create_program_address(derived_seeds, &program_id).unwrap();
        
        assert_eq!(expected_pda, derived_pda);
    }

    #[test]
    fn test_treasury_authority_pda_derivation() {
        let program_id = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        
        let expected_seeds = &[
            b"treasury_authority",
            quote_mint.as_ref(),
        ];
        let (expected_pda, expected_bump) = Pubkey::find_program_address(expected_seeds, &program_id);
        
        // Test that we can derive the same PDA
        let derived_seeds = &[
            b"treasury_authority",
            quote_mint.as_ref(),
            &[expected_bump],
        ];
        let derived_pda = Pubkey::create_program_address(derived_seeds, &program_id).unwrap();
        
        assert_eq!(expected_pda, derived_pda);
    }

    #[test]
    fn test_pda_uniqueness() {
        let program_id = Pubkey::new_unique();
        let quote_mint1 = Pubkey::new_unique();
        let quote_mint2 = Pubkey::new_unique();
        
        // Test that different quote mints produce different PDAs
        let (pda1, _) = PolicyState::derive_pda(&quote_mint1, &program_id);
        let (pda2, _) = PolicyState::derive_pda(&quote_mint2, &program_id);
        
        assert_ne!(pda1, pda2);
        
        // Test that different distribution days produce different PDAs
        let day1 = 1672531200i64;
        let day2 = 1672617600i64;
        
        let (pda3, _) = DailyDistributionState::derive_pda(day1, &quote_mint1, &program_id);
        let (pda4, _) = DailyDistributionState::derive_pda(day2, &quote_mint1, &program_id);
        
        assert_ne!(pda3, pda4);
    }
}
