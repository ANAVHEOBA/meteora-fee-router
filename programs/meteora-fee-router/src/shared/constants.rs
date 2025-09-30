// Global constants for the Meteora Fee Router program

/// Seed for vault-related PDAs
pub const VAULT_SEED: &[u8] = b"vault";

/// Seed for the position owner PDA
pub const POSITION_OWNER_SEED: &[u8] = b"investor_fee_pos_owner";

/// Seed for policy PDA
pub const POLICY_SEED: &[u8] = b"policy";

/// Seed for progress tracking PDA
pub const PROGRESS_SEED: &[u8] = b"progress";

/// Seed for treasury ATA
pub const TREASURY_SEED: &[u8] = b"treasury";

/// 24 hours in seconds
pub const DISTRIBUTION_INTERVAL: i64 = 86_400;

/// Maximum investors per page (to avoid compute limits)
pub const MAX_INVESTORS_PER_PAGE: u32 = 10;

/// Basis points denominator (10000 = 100%)
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Minimum payout threshold in lamports (to avoid dust)
pub const MIN_PAYOUT_LAMPORTS: u64 = 1_000;
