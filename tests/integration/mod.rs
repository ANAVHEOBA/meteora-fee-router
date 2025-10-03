pub mod test_scenarios;
pub mod test_helpers;

use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

/// Test context for integration tests
pub struct TestContext {
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub quote_mint: Keypair,
    pub creator: Keypair,
    pub investors: Vec<Keypair>,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            program_id: meteora_fee_router::id(),
            payer: Keypair::new(),
            quote_mint: Keypair::new(),
            creator: Keypair::new(),
            investors: (0..10).map(|_| Keypair::new()).collect(),
        }
    }
}
