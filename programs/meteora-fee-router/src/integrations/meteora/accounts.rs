use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

/// Meteora Pool account structure
/// Using bytemuck for zero-copy deserialization (as specified in IDL)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Pool {
    pub pool_fees: PoolFeesStruct,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub whitelisted_vault: Pubkey,
    pub partner: Pubkey,
    pub liquidity: u128,
    pub _padding: u128,
    pub protocol_a_fee: u64,
    pub protocol_b_fee: u64,
    pub partner_a_fee: u64,
    pub partner_b_fee: u64,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub sqrt_price: u128,
    pub activation_point: u64,
    pub activation_type: u8,
    pub pool_status: u8,
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    /// CRITICAL: 0 = both tokens, 1 = only token A, 2 = only token B
    pub collect_fee_mode: u8,
    pub pool_type: u8,
    pub _padding_0: [u8; 2],
    pub fee_a_per_liquidity: [u8; 32],
    pub fee_b_per_liquidity: [u8; 32],
    // ... rest of fields as padding
    pub _padding_rest: [u8; 256],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PoolFeesStruct {
    pub trade_fee_bps: u64,
    pub protocol_trade_fee_bps: u64,
    pub fund_trade_fee_bps: u64,
}

/// Fee collection modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectFeeMode {
    Both = 0,
    OnlyTokenA = 1,
    OnlyTokenB = 2,
}

impl CollectFeeMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(CollectFeeMode::Both),
            1 => Some(CollectFeeMode::OnlyTokenA),
            2 => Some(CollectFeeMode::OnlyTokenB),
            _ => None,
        }
    }
}

impl Pool {
    /// Get the collect fee mode
    pub fn get_collect_fee_mode(&self) -> Option<CollectFeeMode> {
        CollectFeeMode::from_u8(self.collect_fee_mode)
    }

    /// Check if pool only collects fees in token A
    pub fn is_token_a_only(&self) -> bool {
        self.collect_fee_mode == CollectFeeMode::OnlyTokenA as u8
    }

    /// Check if pool only collects fees in token B
    pub fn is_token_b_only(&self) -> bool {
        self.collect_fee_mode == CollectFeeMode::OnlyTokenB as u8
    }

    /// Check if pool collects fees in both tokens (not allowed for us)
    pub fn is_both_tokens(&self) -> bool {
        self.collect_fee_mode == CollectFeeMode::Both as u8
    }
}

/// Position account (we don't need to deserialize this, just reference it)
/// The position is managed by Meteora program
#[derive(Debug)]
pub struct Position;

/// Pool status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolStatus {
    Enabled = 0,
    Disabled = 1,
}

impl Pool {
    pub fn is_enabled(&self) -> bool {
        self.pool_status == PoolStatus::Enabled as u8
    }
}

// Implement AccountDeserialize for Pool to work with Account<'info, Pool>
impl anchor_lang::AccountDeserialize for Pool {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        if buf.len() < std::mem::size_of::<Pool>() {
            return Err(anchor_lang::error::ErrorCode::AccountDidNotDeserialize.into());
        }
        Ok(*bytemuck::from_bytes(&buf[..std::mem::size_of::<Pool>()]))
    }
}

// Implement Owner for Pool (required by Account)
impl anchor_lang::Owner for Pool {
    fn owner() -> Pubkey {
        crate::integrations::meteora::METEORA_CP_AMM_PROGRAM_ID
    }
}
