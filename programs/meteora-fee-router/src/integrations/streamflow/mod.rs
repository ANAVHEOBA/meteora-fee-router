// Streamflow Integration Module
// Purpose: Read locked token amounts from Streamflow streams

pub mod accounts;
pub mod cpi;
pub mod calculations;

// Re-export public API
pub use accounts::*;
pub use cpi::*;
pub use calculations::*;

// Streamflow program ID (mainnet)
use anchor_lang::prelude::*;

declare_id!("strmRqUCoQUgGUan5YhzUZa6KqdzwX5L6FpUxfmKg5m");

pub const STREAMFLOW_PROGRAM_ID: Pubkey = crate::integrations::streamflow::ID;
