# 🚀 Meteora Fee Router - Integration Guide

## 📋 Overview

The **Meteora Fee Router** is a sophisticated DeFi protocol that manages honorary LP positions, automates fee claiming, and distributes fees to investors based on their locked token allocations via Streamflow integration.

**🔗 Deployed Program ID:** `Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE`
**🌐 Network:** Solana Devnet
**⚡ Binary Size:** 459,584 bytes
**📦 Repository:** [ANAVHEOBA/meteora-fee-router](https://github.com/ANAVHEOBA/meteora-fee-router)

## 🏗️ Architecture

### Core Modules
- **Position Management**: Honorary LP position creation and management
- **Fee Claiming**: Automated fee claiming from Meteora DLMM pools
- **Distribution System**: 24-hour gated distribution with pagination
- **Investor Logic**: Pro-rata distribution based on Streamflow locked tokens

### Key Features
- ✅ Quote-only fee positions (no impermanent loss)
- ✅ Permissionless 24-hour distribution cycles
- ✅ Mathematical precision in pro-rata calculations
- ✅ Complete Streamflow integration
- ✅ Comprehensive event system

## 🔧 Integration Requirements

### Prerequisites
- Node.js 16+ or Python 3.8+
- `@coral-xyz/anchor` or `anchorpy` package
- Solana wallet (Phantom, Solflare, etc.)
- IDL file: `meteora_fee_router.json`

### Installation

```bash
# Node.js
npm install @coral-xyz/anchor @solana/web3.js @solana/wallet-adapter-react

# Python
pip install anchorpy solana-py
```

## 📡 Network Configuration

```typescript
// TypeScript/JavaScript
import { Connection, PublicKey } from '@solana/web3.js';

const PROGRAM_ID = new PublicKey('Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE');
const DEVNET_RPC = 'https://api.devnet.solana.com';
const connection = new Connection(DEVNET_RPC);
```

```python
# Python
from solana.rpc.api import Client
from solana.publickey import PublicKey

PROGRAM_ID = PublicKey('Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE')
DEVNET_RPC = 'https://api.devnet.solana.com'
client = Client(DEVNET_RPC)
```

## 🏦 Account Structures

### Policy State
```typescript
interface PolicyState {
  authority: PublicKey;           // Policy authority
  quoteMint: PublicKey;           // Quote token mint
  investorFeeShareBps: number;    // Fee share for investors (basis points)
  dailyCapLamports: number;       // Daily distribution cap
  minPayoutLamports: number;      // Minimum payout amount
  y0TotalAllocation: number;      // Total allocation for calculations
  bump: number;                   // PDA bump
}
```

### Position Account
```typescript
interface Position {
  owner: PublicKey;               // Position owner
  pool: PublicKey;                // Meteora DLMM pool
  quoteMint: PublicKey;           // Quote token mint
  positionOwnerPda: PublicKey;    // PDA for position ownership
  claimedFees: number;            // Total claimed fees
  bump: number;                   // PDA bump
}
```

### Daily Distribution State
```typescript
interface DailyDistributionState {
  authority: PublicKey;           // Distribution authority
  distributionDay: number;        // Current distribution day (Unix timestamp)
  investorsProcessed: number;     // Number of investors processed
  totalDistributedAmount: number;  // Total amount distributed
  creatorRemainder: number;       // Remaining amount for creator
  isComplete: boolean;            // Distribution completion status
  bump: number;                   // PDA bump
}
```

## ⚡ Instructions

### 1. Initialize Policy
Creates the main policy configuration.

```typescript
await program.methods
  .initializePolicy(
    new BN(500),           // investorFeeShareBps (5%)
    new BN(1000000),       // dailyCapLamports (1M lamports)
    new BN(10000),         // minPayoutLamports (10K lamports)
    new BN(100000000)      // y0TotalAllocation (100M)
  )
  .accounts({
    authority: wallet.publicKey,
    quoteMint: quoteMintPubkey,
    policyState: policyStatePda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

**Accounts Required:**
- `authority` (signer): Policy creator
- `quoteMint`: Quote token mint address
- `policyState` (PDA): Policy configuration account
- `systemProgram`: Solana system program

### 2. Initialize Position
Creates a new honorary LP position.

```typescript
await program.methods
  .initializePosition()
  .accounts({
    authority: wallet.publicKey,
    positionOwnerPda: positionOwnerPda,
    position: positionPubkey,
    pool: meteoraPoolPubkey,
    quoteMint: quoteMintPubkey,
  })
  .rpc();
```

**Accounts Required:**
- `authority` (signer): Position creator
- `positionOwnerPda` (PDA): Position ownership account
- `position` (PDA): Position account
- `pool`: Meteora DLMM pool address
- `quoteMint`: Quote token mint

### 3. Claim Fees
Claims accumulated fees from the position.

```typescript
await program.methods
  .claimFees()
  .accounts({
    authority: wallet.publicKey,
    position: positionPubkey,
    treasuryAta: treasuryAtaPubkey,
  })
  .rpc();
```

**Accounts Required:**
- `authority` (signer): Fee claimer
- `position`: Position account
- `treasuryAta`: Treasury token account

### 4. Start Daily Distribution
Initiates a new 24-hour distribution cycle.

```typescript
await program.methods
  .startDailyDistribution(
    new BN(currentTimestamp)  // distributionDay
  )
  .accounts({
    authority: wallet.publicKey,
    dailyDistributionState: dailyDistributionStatePda,
  })
  .rpc();
```

**Accounts Required:**
- `authority` (signer): Distribution initiator
- `dailyDistributionState` (PDA): Distribution state account

### 5. Process Investor Page
Processes a page of investors for distribution.

```typescript
await program.methods
  .processInvestorPage()
  .accounts({
    authority: wallet.publicKey,
    dailyDistributionState: dailyDistributionStatePda,
  })
  .rpc();
```

**Accounts Required:**
- `authority` (signer): Page processor
- `dailyDistributionState`: Current distribution state

### 6. Complete Daily Distribution
Finalizes the daily distribution cycle.

```typescript
await program.methods
  .completeDailyDistribution()
  .accounts({
    authority: wallet.publicKey,
    creatorAta: creatorAtaPubkey,
  })
  .rpc();
```

**Accounts Required:**
- `authority` (signer): Distribution completer
- `creatorAta`: Creator's token account for remainder

## 🎯 Events

### HonoraryPositionInitialized
```typescript
{
  position: PublicKey,     // Position account
  quoteMint: PublicKey     // Quote token mint
}
```

### QuoteFeesClaimed
```typescript
{
  amount: number,          // Amount of fees claimed
  timestamp: number        // Claim timestamp
}
```

### InvestorsProcessed
```typescript
{
  investorsInPage: number,        // Number of investors processed
  amountDistributedInPage: number // Total amount distributed
}
```

### CreatorPayoutCompleted
```typescript
{
  creatorRemainder: number,       // Remaining amount for creator
  totalDistributedAmount: number  // Total distributed in cycle
}
```

## ❌ Error Codes

| Code | Name | Description |
|------|------|-------------|
| 6000 | BaseFeeDetected | Base fees detected - only quote fees allowed |
| 6001 | TooSoonToDistribute | 24 hour period not elapsed |
| 6002 | DailyCapExceeded | Daily cap exceeded |

## 💻 Frontend Integration Example

```typescript
import { Program, AnchorProvider, web3, BN } from '@coral-xyz/anchor';
import { useWallet } from '@solana/wallet-adapter-react';
import idl from '../idl/meteora_fee_router.json';

const PROGRAM_ID = new web3.PublicKey('Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE');

export class MeteoraFeeRouter {
  private program: Program;

  constructor(provider: AnchorProvider) {
    this.program = new Program(idl, PROGRAM_ID, provider);
  }

  async initializePolicy(params: {
    investorFeeShareBps: number;
    dailyCapLamports: number;
    minPayoutLamports: number;
    y0TotalAllocation: number;
    quoteMint: PublicKey;
  }) {
    const [policyStatePda] = await web3.PublicKey.findProgramAddress(
      [Buffer.from('policy'), params.quoteMint.toBuffer()],
      PROGRAM_ID
    );

    return this.program.methods
      .initializePolicy(
        new BN(params.investorFeeShareBps),
        new BN(params.dailyCapLamports),
        new BN(params.minPayoutLamports),
        new BN(params.y0TotalAllocation)
      )
      .accounts({
        authority: this.program.provider.publicKey,
        quoteMint: params.quoteMint,
        policyState: policyStatePda,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
  }

  async claimFees(positionPubkey: PublicKey, treasuryAta: PublicKey) {
    return this.program.methods
      .claimFees()
      .accounts({
        authority: this.program.provider.publicKey,
        position: positionPubkey,
        treasuryAta: treasuryAta,
      })
      .rpc();
  }

  // Event listener example
  async subscribeToEvents(callback: (event: any) => void) {
    return this.program.addEventListener('QuoteFeesClaimed', callback);
  }
}
```

## 🖥️ Backend Integration Example

```python
from anchorpy import Program, Provider, Context
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.system_program import SYS_PROGRAM_ID
import json

class MeteoraFeeRouter:
    def __init__(self, provider: Provider):
        with open('meteora_fee_router.json') as f:
            idl = json.load(f)

        program_id = PublicKey('Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE')
        self.program = Program(idl, program_id, provider)

    async def initialize_policy(
        self,
        authority: Keypair,
        quote_mint: PublicKey,
        investor_fee_share_bps: int,
        daily_cap_lamports: int,
        min_payout_lamports: int,
        y0_total_allocation: int
    ):
        policy_state_pda, _ = PublicKey.find_program_address(
            [b'policy', bytes(quote_mint)],
            self.program.program_id
        )

        return await self.program.rpc["initialize_policy"](
            investor_fee_share_bps,
            daily_cap_lamports,
            min_payout_lamports,
            y0_total_allocation,
            ctx=Context(
                accounts={
                    "authority": authority.public_key,
                    "quote_mint": quote_mint,
                    "policy_state": policy_state_pda,
                    "system_program": SYS_PROGRAM_ID,
                },
                signers=[authority]
            )
        )

    async def claim_fees(
        self,
        authority: Keypair,
        position: PublicKey,
        treasury_ata: PublicKey
    ):
        return await self.program.rpc["claim_fees"](
            ctx=Context(
                accounts={
                    "authority": authority.public_key,
                    "position": position,
                    "treasury_ata": treasury_ata,
                },
                signers=[authority]
            )
        )
```

## 🔍 Testing Instructions

### Local Development
```bash
# Start local validator
solana-test-validator

# Deploy locally
anchor localnet
anchor deploy

# Run tests
anchor test
```

### Devnet Testing
```typescript
// Use devnet RPC for testing
const connection = new Connection('https://api.devnet.solana.com');

// Test with real program ID
const programId = new PublicKey('Dr4sAJ3wJoy9DKjrEoCwJW7axJmQweWMcBS36UB1y6KE');
```

## 📚 Additional Resources

- **📖 Full Documentation**: See `README.md` for complete technical details
- **🧪 Test Suite**: 100+ tests in `tests/` directory
- **🔧 IDL File**: Complete interface definition in `meteora_fee_router.json`
- **📦 Source Code**: [GitHub Repository](https://github.com/ANAVHEOBA/meteora-fee-router)

## 🚨 Important Notes

- **Network**: Currently deployed on **Solana Devnet** for testing
- **Production**: Deploy to mainnet for production use
- **IDL**: Use the provided `meteora_fee_router.json` for type safety
- **Events**: Subscribe to events for real-time updates
- **PDA Calculation**: Use consistent seeds for PDA generation

## 💬 Support

For integration questions or issues:
- Check the [GitHub Issues](https://github.com/ANAVHEOBA/meteora-fee-router/issues)
- Review the comprehensive test suite
- Examine the IDL file for complete interface documentation

---

**🎊 Ready for Integration! The Meteora Fee Router is fully deployed and documented for seamless frontend/backend integration.**
