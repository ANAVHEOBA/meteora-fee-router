// Constants for the Meteora Fee Router program

// PDA seeds
pub const VAULT_SEED: &[u8] = b"vault";
pub const POSITION_OWNER_SEED: &[u8] = b"position_owner";
pub const POLICY_SEED: &[u8] = b"policy";
pub const TREASURY_SEED: &[u8] = b"treasury";

// Program limits
pub const MAX_INVESTORS_PER_PAGE: u32 = 50;

// Distribution constants
pub const DEFAULT_MIN_PAYOUT_LAMPORTS: u64 = 1000; // 0.001 SOL equivalent
pub const DEFAULT_DAILY_CAP_LAMPORTS: u64 = 1_000_000_000; // 1 SOL equivalent
pub const DEFAULT_INVESTOR_FEE_SHARE_BPS: u64 = 5000; // 50% max to investors
pub const MAX_BASIS_POINTS: u64 = 10000; // 100%

// Time constants
pub const SECONDS_PER_DAY: i64 = 86400;

/// Basis points denominator (10000 = 100%)
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Minimum payout threshold in lamports (to avoid dust)
pub const MIN_PAYOUT_LAMPORTS: u64 = 1_000;
