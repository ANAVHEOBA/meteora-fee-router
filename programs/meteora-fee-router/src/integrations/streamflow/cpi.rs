use anchor_lang::prelude::*;
use crate::integrations::streamflow::accounts::{StreamflowStream, InvestorStreamData};

/// Data for a single investor
#[derive(Debug, Clone)]
pub struct InvestorData {
    pub investor: Pubkey,
    pub locked_amount: u64,
    pub investor_ata: Pubkey, // This would need to be derived or passed
}

/// Error information for failed stream processing
#[derive(Debug, Clone)]
pub struct StreamError {
    pub stream_account: Pubkey,
    pub investor: Option<Pubkey>,
    pub error_type: StreamErrorType,
    pub error_message: String,
}

/// Types of stream processing errors
#[derive(Debug, Clone)]
pub enum StreamErrorType {
    InvalidStreamData,
    MissingInvestorAta,
    StreamExpired,
    InsufficientLocked,
    AccountDeserializationFailed,
}

/// Read stream data from a Streamflow stream account
/// 
/// This function deserializes the stream account data to get
/// information about locked amounts and vesting schedules.
/// 
/// # Arguments
/// * `stream_account_info` - The AccountInfo for the stream account
/// 
/// # Returns
/// * `Result<StreamData>` - The deserialized stream data
/// * `Result<StreamflowStream>` - The deserialized stream data
pub fn read_stream_data(stream_account_info: &AccountInfo) -> Result<StreamflowStream> {
    // Deserialize the account data
    let stream_data = StreamflowStream::try_deserialize(&mut stream_account_info.data.borrow().as_ref())?;
    
    // This would need to be updated based on actual Streamflow implementation
    
    Ok(stream_data)
}

/// Calculate locked amounts for multiple investors with error handling
/// 
/// This function processes multiple stream accounts and calculates
/// the total locked amount across all investors, with comprehensive
/// error handling and retry tracking.
/// 
/// # Arguments
/// * `stream_accounts` - Array of stream account infos
/// * `current_timestamp` - Current Unix timestamp
/// * `quote_mint` - The quote mint being distributed
/// 
/// # Returns
/// * `Result<(Vec<InvestorStreamData>, u64, Vec<StreamError>)>` - investor data, total locked, and errors
pub fn calculate_locked_amounts_with_errors(
    stream_accounts: &[AccountInfo],
    current_timestamp: u64,
    quote_mint: &Pubkey,
) -> Result<(Vec<InvestorStreamData>, u64, Vec<StreamError>)> {
    let mut investor_data = Vec::new();
    let mut total_locked = 0u64;
    let mut errors = Vec::new();

    for stream_account in stream_accounts {
        match process_single_stream(stream_account, current_timestamp, quote_mint) {
            Ok(Some(data)) => {
                total_locked = total_locked.saturating_add(data.locked_amount);
                investor_data.push(data);
            }
            Ok(None) => {
                // Stream has no locked amount - not an error
                msg!("Stream {} has no locked amount", stream_account.key());
            }
            Err(error) => {
                msg!("Error processing stream {}: {}", stream_account.key(), error.error_message);
                errors.push(error);
            }
        }
    }

    msg!("Processed {} streams: {} successful, {} errors", 
         stream_accounts.len(), investor_data.len(), errors.len());

    Ok((investor_data, total_locked, errors))
}

/// Process a single stream account with error handling
fn process_single_stream(
    stream_account: &AccountInfo,
    current_timestamp: u64,
    quote_mint: &Pubkey,
) -> std::result::Result<Option<InvestorStreamData>, StreamError> {
    // Try to read stream data
    let stream = match read_stream_data(stream_account) {
        Ok(stream) => stream,
        Err(_) => {
            return Err(StreamError {
                stream_account: stream_account.key(),
                investor: None,
                error_type: StreamErrorType::AccountDeserializationFailed,
                error_message: "Failed to deserialize stream account".to_string(),
            });
        }
    };

    // Validate stream is not expired
    if stream.end_time < current_timestamp {
        return Err(StreamError {
            stream_account: stream_account.key(),
            investor: Some(stream.recipient),
            error_type: StreamErrorType::StreamExpired,
            error_message: "Stream has expired".to_string(),
        });
    }

    // Calculate locked amount using the existing method
    let locked_amount = stream.locked_amount(current_timestamp);
    
    if locked_amount == 0 {
        return Ok(None); // No locked amount, but not an error
    }

    // TODO: Validate investor ATA exists
    let investor_ata = stream.recipient; // Placeholder - would need proper ATA derivation
    
    Ok(Some(InvestorStreamData {
        investor: stream.recipient,
        stream_account: stream_account.key(),
        locked_amount,
        total_deposited: stream.deposited_amount,
        investor_ata,
    }))
}

/// Calculate locked amounts for multiple investors (backward compatibility)
/// 
/// This function maintains backward compatibility while using the enhanced
/// error handling internally.
/// 
/// # Arguments
/// * `stream_accounts` - Array of stream account infos
/// * `current_timestamp` - Current Unix timestamp
/// * `quote_mint` - The quote mint being distributed
/// 
/// # Returns
/// * `Result<(Vec<InvestorStreamData>, u64)>` - Investor data and total locked
pub fn calculate_locked_amounts(
    stream_accounts: &[AccountInfo],
    current_timestamp: u64,
    quote_mint: &Pubkey,
) -> Result<(Vec<InvestorStreamData>, u64)> {
    let (investor_data, total_locked, errors) = calculate_locked_amounts_with_errors(
        stream_accounts,
        current_timestamp,
        quote_mint,
    )?;

    // Log errors but don't fail the entire operation
    if !errors.is_empty() {
        msg!("Encountered {} stream processing errors (continuing with valid streams)", errors.len());
    }

    Ok((investor_data, total_locked))
}

/// Calculate the locked fraction for fee distribution
/// 
/// This implements the formula: f_locked(t) = locked_total(t) / Y0
/// where Y0 is the initial total deposit amount.
/// 
/// # Arguments
/// * `locked_total` - Total amount currently locked across all streams
/// * `initial_total_deposit` - Y0 - the initial total deposit amount
/// 
/// # Returns
/// * `u64` - The locked fraction as basis points (out of 10000)
pub fn calculate_locked_fraction(locked_total: u64, initial_total_deposit: u64) -> u64 {
    if initial_total_deposit == 0 {
        return 0;
    }
    
    // f_locked(t) = locked_total(t) / Y0
    // Return as basis points (multiply by 10000)
    ((locked_total as u128 * 10000u128) / initial_total_deposit as u128) as u64
}

/// Calculate eligible investor share based on locked fraction
/// 
/// This implements: eligible_investor_share_bps = min(investor_fee_share_bps, floor(f_locked(t) * 10000))
/// 
/// # Arguments
/// * `investor_fee_share_bps` - Maximum investor fee share in basis points
/// * `locked_fraction_bps` - Current locked fraction in basis points
/// 
/// # Returns
/// * `u64` - Eligible investor share in basis points
pub fn calculate_eligible_investor_share(
    investor_fee_share_bps: u64,
    locked_fraction_bps: u64,
) -> u64 {
    std::cmp::min(investor_fee_share_bps, locked_fraction_bps)
}

/// Calculate total investor fee amount in quote tokens
/// 
/// This implements: investor_fee_quote = floor(claimed_quote * eligible_investor_share_bps / 10000)
/// 
/// # Arguments
/// * `claimed_quote` - Total quote tokens claimed from fees
/// * `eligible_investor_share_bps` - Eligible investor share in basis points
/// 
/// # Returns
/// * `u64` - Total investor fee amount in quote tokens
pub fn calculate_investor_fee_amount(
    claimed_quote: u64,
    eligible_investor_share_bps: u64,
) -> u64 {
    ((claimed_quote as u128 * eligible_investor_share_bps as u128) / 10000u128) as u64
}

/// Validate stream account ownership and program
/// 
/// # Arguments
/// * `stream_account_info` - The stream account to validate
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn validate_stream_account(stream_account_info: &AccountInfo) -> Result<()> {
    // Validate that the account is owned by the Streamflow program
    require!(
        stream_account_info.owner == &crate::integrations::streamflow::STREAMFLOW_PROGRAM_ID,
        anchor_lang::error::ErrorCode::ConstraintOwner
    );
    
    // Additional validations could go here
    // - Check account discriminator
    // - Validate account size
    // - Check magic numbers
    
    Ok(())
}
