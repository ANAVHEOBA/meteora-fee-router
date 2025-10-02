use anchor_lang::prelude::*;
use crate::integrations::streamflow::accounts::InvestorStreamData;

/// Distribution calculation results
#[derive(Debug, Clone)]
pub struct DistributionCalculation {
    /// Total amount available for investor distribution
    pub investor_fee_quote: u64,
    
    /// Individual investor payouts
    pub investor_payouts: Vec<InvestorPayout>,
    
    /// Total amount distributed to investors
    pub total_distributed: u64,
    
    /// Dust amount (due to floor division)
    pub dust_amount: u64,
    
    /// Amount going to creator (remainder)
    pub creator_remainder: u64,
}

/// Individual investor payout information
#[derive(Debug, Clone)]
pub struct InvestorPayout {
    /// The investor's wallet address
    pub investor: Pubkey,
    
    /// The investor's ATA for receiving tokens
    pub investor_ata: Pubkey,
    
    /// Amount to pay out to this investor
    pub payout_amount: u64,
    
    /// The investor's weight in basis points
    pub weight_bps: u64,
    
    /// Whether this payout meets the minimum threshold
    pub meets_minimum: bool,
}

/// Calculate complete distribution for a page of investors
/// 
/// This is the main calculation function that implements the formulas
/// from Section 4 of the specification.
/// 
/// # Arguments
/// * `claimed_quote` - Total quote tokens available for distribution
/// * `investor_data` - Vector of investor stream data
/// * `total_locked` - Total locked amount across all investors
/// * `initial_total_deposit` - Y0 - initial total deposit amount
/// * `investor_fee_share_bps` - Maximum investor fee share in basis points
/// * `min_payout_lamports` - Minimum payout threshold
/// 
/// # Returns
/// * `Result<DistributionCalculation>` - Complete distribution calculation
pub fn calculate_distribution(
    claimed_quote: u64,
    investor_data: &[InvestorStreamData],
    total_locked: u64,
    initial_total_deposit: u64,
    investor_fee_share_bps: u64,
    min_payout_lamports: u64,
) -> Result<DistributionCalculation> {
    msg!("Calculating distribution for {} investors", investor_data.len());
    
    // Step 1: Calculate locked fraction
    // f_locked(t) = locked_total(t) / Y0
    let locked_fraction_bps = if initial_total_deposit == 0 {
        0
    } else {
        ((total_locked as u128 * 10000u128) / initial_total_deposit as u128) as u64
    };
    
    msg!("Locked fraction: {} bps", locked_fraction_bps);
    
    // Step 2: Calculate eligible investor share
    // eligible_investor_share_bps = min(investor_fee_share_bps, floor(f_locked(t) * 10000))
    let eligible_investor_share_bps = std::cmp::min(investor_fee_share_bps, locked_fraction_bps);
    
    msg!("Eligible investor share: {} bps", eligible_investor_share_bps);
    
    // Step 3: Calculate total investor fee amount
    // investor_fee_quote = floor(claimed_quote * eligible_investor_share_bps / 10000)
    let investor_fee_quote = ((claimed_quote as u128 * eligible_investor_share_bps as u128) / 10000u128) as u64;
    
    msg!("Total investor fee amount: {} tokens", investor_fee_quote);
    
    // Handle edge case: all unlocked = 100% to creator
    if total_locked == 0 || investor_fee_quote == 0 {
        return Ok(DistributionCalculation {
            investor_fee_quote: 0,
            investor_payouts: vec![],
            total_distributed: 0,
            dust_amount: 0,
            creator_remainder: claimed_quote,
        });
    }
    
    // Step 4: Calculate individual payouts
    let mut investor_payouts = Vec::new();
    let mut total_distributed = 0u64;
    
    for investor in investor_data {
        // Calculate weight: weight_i(t) = locked_i(t) / locked_total(t)
        let weight_bps = if total_locked == 0 {
            0
        } else {
            ((investor.locked_amount as u128 * 10000u128) / total_locked as u128) as u64
        };
        
        // Calculate payout: payout_i = floor(investor_fee_quote * weight_i(t))
        let payout_amount = ((investor_fee_quote as u128 * investor.locked_amount as u128) / total_locked as u128) as u64;
        
        // Check if payout meets minimum threshold
        let meets_minimum = payout_amount >= min_payout_lamports;
        
        // Only include payouts that meet the minimum
        let final_payout = if meets_minimum { payout_amount } else { 0 };
        
        investor_payouts.push(InvestorPayout {
            investor: investor.investor,
            investor_ata: investor.investor_ata,
            payout_amount: final_payout,
            weight_bps,
            meets_minimum,
        });
        
        total_distributed = total_distributed.saturating_add(final_payout);
    }
    
    // Step 5: Calculate dust and creator remainder
    let dust_amount = investor_fee_quote.saturating_sub(total_distributed);
    let creator_remainder = claimed_quote.saturating_sub(investor_fee_quote);
    
    msg!("Distribution complete: {} distributed, {} dust, {} to creator", 
         total_distributed, dust_amount, creator_remainder);
    
    Ok(DistributionCalculation {
        investor_fee_quote,
        investor_payouts,
        total_distributed,
        dust_amount,
        creator_remainder,
    })
}

/// Apply daily cap to distribution amounts
/// 
/// # Arguments
/// * `calculation` - The distribution calculation
/// * `daily_cap_remaining` - Remaining daily cap
/// 
/// # Returns
/// * `DistributionCalculation` - Capped distribution calculation
pub fn apply_daily_cap(
    mut calculation: DistributionCalculation,
    daily_cap_remaining: u64,
) -> DistributionCalculation {
    if calculation.total_distributed <= daily_cap_remaining {
        // No capping needed
        return calculation;
    }
    
    msg!("Applying daily cap: {} remaining", daily_cap_remaining);
    
    // Scale down all payouts proportionally
    let scale_factor = if calculation.total_distributed == 0 {
        0
    } else {
        ((daily_cap_remaining as u128 * 10000u128) / calculation.total_distributed as u128) as u64
    };
    
    let mut new_total_distributed = 0u64;
    
    for payout in &mut calculation.investor_payouts {
        if payout.payout_amount > 0 {
            let scaled_amount = ((payout.payout_amount as u128 * scale_factor as u128) / 10000u128) as u64;
            payout.payout_amount = scaled_amount;
            new_total_distributed = new_total_distributed.saturating_add(scaled_amount);
        }
    }
    
    calculation.total_distributed = new_total_distributed;
    calculation.dust_amount = daily_cap_remaining.saturating_sub(new_total_distributed);
    
    msg!("After capping: {} distributed", new_total_distributed);
    
    calculation
}

/// Validate distribution calculation
/// 
/// # Arguments
/// * `calculation` - The distribution calculation to validate
/// * `claimed_quote` - Original claimed quote amount
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn validate_distribution(
    calculation: &DistributionCalculation,
    claimed_quote: u64,
) -> Result<()> {
    // Validate that total doesn't exceed claimed amount
    let total_accounted = calculation.total_distributed
        .saturating_add(calculation.dust_amount)
        .saturating_add(calculation.creator_remainder);
    
    require!(
        total_accounted <= claimed_quote,
        anchor_lang::error::ErrorCode::ConstraintRaw
    );
    
    // Validate individual payouts are non-negative
    for payout in &calculation.investor_payouts {
        require!(
            payout.weight_bps <= 10000,
            anchor_lang::error::ErrorCode::ConstraintRaw
        );
    }
    
    Ok(())
}
