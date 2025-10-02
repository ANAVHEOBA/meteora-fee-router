use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;

/// Meteora CP-AMM Program ID: cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG
pub const METEORA_CP_AMM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0xc9, 0x8a, 0x6d, 0xd0, 0x5c, 0x47, 0x4b, 0x59,
    0x5f, 0x58, 0xb4, 0xdc, 0x09, 0x4f, 0xc1, 0x47,
    0x1c, 0x1f, 0x4e, 0xbb, 0xc6, 0xce, 0xf5, 0xb4,
    0xcd, 0x35, 0x6e, 0x1f, 0x53, 0x47, 0x6d, 0x1c,
]);

/// Pool authority (from IDL): HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC
pub const POOL_AUTHORITY: Pubkey = Pubkey::new_from_array([
    0xf4, 0x5e, 0x8f, 0x7a, 0x25, 0xf5, 0x3b, 0x3e,
    0x89, 0x5f, 0x43, 0x72, 0x6e, 0x53, 0x44, 0x39,
    0x72, 0x6e, 0x53, 0x44, 0x39, 0x7a, 0x42, 0x37,
    0x42, 0x38, 0x68, 0x51, 0x4d, 0x4b, 0x6d, 0x44,
]);

/// Seeds for position PDA
pub const POSITION_SEED: &[u8] = b"position";

/// Seeds for position NFT account PDA
pub const POSITION_NFT_ACCOUNT_SEED: &[u8] = b"position_nft_account";

/// Seeds for event authority PDA
pub const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";

/// Create a new position in a Meteora pool
/// 
/// This creates a position NFT owned by the specified owner (can be a PDA).
/// The position is empty initially - liquidity is added separately.
/// 
/// # Arguments
/// * `ctx` - The CPI context
/// * `owner_seeds` - Optional seeds if owner is a PDA (for signing)
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn create_position<'info>(
    owner: AccountInfo<'info>,
    position_nft_mint: AccountInfo<'info>,
    position_nft_account: AccountInfo<'info>,
    pool: AccountInfo<'info>,
    position: AccountInfo<'info>,
    pool_authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    event_authority: AccountInfo<'info>,
    meteora_program: AccountInfo<'info>,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    msg!("Creating Meteora position via CPI");

    // Instruction discriminator for create_position (from IDL)
    let discriminator: [u8; 8] = [48, 215, 197, 153, 96, 203, 180, 133];

    // Build instruction data (discriminator + no args)
    let mut instruction_data = Vec::with_capacity(8);
    instruction_data.extend_from_slice(&discriminator);

    // Build accounts for the instruction
    let accounts = vec![
        AccountMeta::new_readonly(owner.key(), false), // owner (not signer here, we sign below)
        AccountMeta::new(position_nft_mint.key(), true), // position_nft_mint (signer)
        AccountMeta::new(position_nft_account.key(), false), // position_nft_account (PDA)
        AccountMeta::new(pool.key(), false), // pool
        AccountMeta::new(position.key(), false), // position (PDA)
        AccountMeta::new_readonly(pool_authority.key(), false), // pool_authority
        AccountMeta::new(payer.key(), true), // payer (signer)
        AccountMeta::new_readonly(token_program.key(), false), // token_program
        AccountMeta::new_readonly(system_program.key(), false), // system_program
        AccountMeta::new_readonly(event_authority.key(), false), // event_authority (PDA)
        AccountMeta::new_readonly(meteora_program.key(), false), // program
    ];

    let instruction = anchor_lang::solana_program::instruction::Instruction {
        program_id: METEORA_CP_AMM_PROGRAM_ID,
        accounts,
        data: instruction_data,
    };

    // Invoke with optional PDA signing
    if let Some(seeds) = owner_seeds {
        invoke_signed(
            &instruction,
            &[
                owner,
                position_nft_mint,
                position_nft_account,
                pool,
                position,
                pool_authority,
                payer,
                token_program,
                system_program,
                event_authority,
                meteora_program,
            ],
            seeds,
        )?;
    } else {
        anchor_lang::solana_program::program::invoke(
            &instruction,
            &[
                owner,
                position_nft_mint,
                position_nft_account,
                pool,
                position,
                pool_authority,
                payer,
                token_program,
                system_program,
                event_authority,
                meteora_program,
            ],
        )?;
    }

    msg!("Position created successfully");
    Ok(())
}

/// Derive the position PDA from the position NFT mint
pub fn derive_position_pda(position_nft_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[POSITION_SEED, position_nft_mint.as_ref()],
        &METEORA_CP_AMM_PROGRAM_ID,
    )
}

/// Derive the position NFT account PDA from the position NFT mint
pub fn derive_position_nft_account_pda(position_nft_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[POSITION_NFT_ACCOUNT_SEED, position_nft_mint.as_ref()],
        &METEORA_CP_AMM_PROGRAM_ID,
    )
}

/// Derive the event authority PDA
pub fn derive_event_authority_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[EVENT_AUTHORITY_SEED],
        &METEORA_CP_AMM_PROGRAM_ID,
    )
}

/// Parameters for adding liquidity to a position
#[derive(Debug, Clone, Copy)]
pub struct AddLiquidityParameters {
    /// Delta liquidity to add
    pub liquidity_delta: u128,
    /// Maximum token A amount to spend
    pub token_a_amount_threshold: u64,
    /// Maximum token B amount to spend  
    pub token_b_amount_threshold: u64,
}

impl AddLiquidityParameters {
    /// Create parameters for minimal liquidity addition (quote-only)
    /// This adds just enough liquidity to activate fee collection
    pub fn minimal_quote_only(quote_amount: u64) -> Self {
        Self {
            liquidity_delta: 1_000_000, // Minimal liquidity (1M units)
            token_a_amount_threshold: quote_amount,
            token_b_amount_threshold: quote_amount,
        }
    }
}

/// Add liquidity to a Meteora position
/// 
/// This adds liquidity to an existing position to enable fee collection.
/// For quote-only positions, we add minimal liquidity across the full range.
/// 
/// # Arguments
/// * All the required accounts for add_liquidity instruction
/// * `params` - Liquidity parameters
/// * `owner_seeds` - Optional seeds if owner is a PDA
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn add_liquidity<'info>(
    pool: AccountInfo<'info>,
    position: AccountInfo<'info>,
    token_a_account: AccountInfo<'info>,
    token_b_account: AccountInfo<'info>,
    token_a_vault: AccountInfo<'info>,
    token_b_vault: AccountInfo<'info>,
    token_a_mint: AccountInfo<'info>,
    token_b_mint: AccountInfo<'info>,
    position_nft_account: AccountInfo<'info>,
    owner: AccountInfo<'info>,
    token_a_program: AccountInfo<'info>,
    token_b_program: AccountInfo<'info>,
    event_authority: AccountInfo<'info>,
    meteora_program: AccountInfo<'info>,
    params: AddLiquidityParameters,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    msg!("Adding liquidity to Meteora position via CPI");

    // Instruction discriminator for add_liquidity (from IDL)
    let discriminator: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72];

    // Serialize parameters
    let mut param_data = Vec::new();
    param_data.extend_from_slice(&params.liquidity_delta.to_le_bytes());
    param_data.extend_from_slice(&params.token_a_amount_threshold.to_le_bytes());
    param_data.extend_from_slice(&params.token_b_amount_threshold.to_le_bytes());

    // Build instruction data (discriminator + params)
    let mut instruction_data = Vec::with_capacity(8 + param_data.len());
    instruction_data.extend_from_slice(&discriminator);
    instruction_data.extend_from_slice(&param_data);

    // Build accounts for the instruction
    let accounts = vec![
        AccountMeta::new(pool.key(), false), // pool
        AccountMeta::new(position.key(), false), // position
        AccountMeta::new(token_a_account.key(), false), // token_a_account
        AccountMeta::new(token_b_account.key(), false), // token_b_account
        AccountMeta::new(token_a_vault.key(), false), // token_a_vault
        AccountMeta::new(token_b_vault.key(), false), // token_b_vault
        AccountMeta::new_readonly(token_a_mint.key(), false), // token_a_mint
        AccountMeta::new_readonly(token_b_mint.key(), false), // token_b_mint
        AccountMeta::new_readonly(position_nft_account.key(), false), // position_nft_account
        AccountMeta::new_readonly(owner.key(), true), // owner (signer)
        AccountMeta::new_readonly(token_a_program.key(), false), // token_a_program
        AccountMeta::new_readonly(token_b_program.key(), false), // token_b_program
        AccountMeta::new_readonly(event_authority.key(), false), // event_authority
        AccountMeta::new_readonly(meteora_program.key(), false), // program
    ];

    let instruction = anchor_lang::solana_program::instruction::Instruction {
        program_id: METEORA_CP_AMM_PROGRAM_ID,
        accounts,
        data: instruction_data,
    };

    // Invoke with optional PDA signing
    if let Some(seeds) = owner_seeds {
        invoke_signed(
            &instruction,
            &[
                pool,
                position,
                token_a_account,
                token_b_account,
                token_a_vault,
                token_b_vault,
                token_a_mint,
                token_b_mint,
                position_nft_account,
                owner,
                token_a_program,
                token_b_program,
                event_authority,
                meteora_program,
            ],
            seeds,
        )?;
    } else {
        anchor_lang::solana_program::program::invoke(
            &instruction,
            &[
                pool,
                position,
                token_a_account,
                token_b_account,
                token_a_vault,
                token_b_vault,
                token_a_mint,
                token_b_mint,
                position_nft_account,
                owner,
                token_a_program,
                token_b_program,
                event_authority,
                meteora_program,
            ],
        )?;
    }

    msg!("Liquidity added successfully");
    Ok(())
}

/// Claim fees from a Meteora position
/// 
/// This claims accumulated fees from the position to the owner's token accounts.
/// For quote-only positions, only quote token fees should be claimed.
/// 
/// # Arguments
/// * All the required accounts for claim_position_fee instruction
/// * `owner_seeds` - Optional seeds if owner is a PDA
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn claim_position_fee<'info>(
    pool_authority: AccountInfo<'info>,
    pool: AccountInfo<'info>,
    position: AccountInfo<'info>,
    token_a_account: AccountInfo<'info>,
    token_b_account: AccountInfo<'info>,
    token_a_vault: AccountInfo<'info>,
    token_b_vault: AccountInfo<'info>,
    token_a_mint: AccountInfo<'info>,
    token_b_mint: AccountInfo<'info>,
    position_nft_account: AccountInfo<'info>,
    owner: AccountInfo<'info>,
    token_a_program: AccountInfo<'info>,
    token_b_program: AccountInfo<'info>,
    event_authority: AccountInfo<'info>,
    meteora_program: AccountInfo<'info>,
    owner_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    msg!("Claiming position fees via CPI");

    // Instruction discriminator for claim_position_fee (from IDL)
    let discriminator: [u8; 8] = [180, 38, 154, 17, 133, 33, 162, 211];

    // Build instruction data (discriminator only, no args)
    let instruction_data = discriminator.to_vec();

    // Build accounts for the instruction
    let accounts = vec![
        AccountMeta::new_readonly(pool_authority.key(), false), // pool_authority
        AccountMeta::new_readonly(pool.key(), false), // pool
        AccountMeta::new(position.key(), false), // position
        AccountMeta::new(token_a_account.key(), false), // token_a_account
        AccountMeta::new(token_b_account.key(), false), // token_b_account
        AccountMeta::new(token_a_vault.key(), false), // token_a_vault
        AccountMeta::new(token_b_vault.key(), false), // token_b_vault
        AccountMeta::new_readonly(token_a_mint.key(), false), // token_a_mint
        AccountMeta::new_readonly(token_b_mint.key(), false), // token_b_mint
        AccountMeta::new_readonly(position_nft_account.key(), false), // position_nft_account
        AccountMeta::new_readonly(owner.key(), true), // owner (signer)
        AccountMeta::new_readonly(token_a_program.key(), false), // token_a_program
        AccountMeta::new_readonly(token_b_program.key(), false), // token_b_program
        AccountMeta::new_readonly(event_authority.key(), false), // event_authority
        AccountMeta::new_readonly(meteora_program.key(), false), // program
    ];

    let instruction = anchor_lang::solana_program::instruction::Instruction {
        program_id: METEORA_CP_AMM_PROGRAM_ID,
        accounts,
        data: instruction_data,
    };

    // Invoke with optional PDA signing
    if let Some(seeds) = owner_seeds {
        invoke_signed(
            &instruction,
            &[
                pool_authority,
                pool,
                position,
                token_a_account,
                token_b_account,
                token_a_vault,
                token_b_vault,
                token_a_mint,
                token_b_mint,
                position_nft_account,
                owner,
                token_a_program,
                token_b_program,
                event_authority,
                meteora_program,
            ],
            seeds,
        )?;
    } else {
        anchor_lang::solana_program::program::invoke(
            &instruction,
            &[
                pool_authority,
                pool,
                position,
                token_a_account,
                token_b_account,
                token_a_vault,
                token_b_vault,
                token_a_mint,
                token_b_mint,
                position_nft_account,
                owner,
                token_a_program,
                token_b_program,
                event_authority,
                meteora_program,
            ],
        )?;
    }

    msg!("Position fees claimed successfully");
    Ok(())
}
