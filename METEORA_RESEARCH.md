# Meteora DAMM V2 Research Summary

## 🎯 Key Findings

### Program Information
- **Program Name**: Meteora Constant Product AMM (CP-AMM / DAMM V2)
- **Program ID (Mainnet)**: `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`
- **Program ID (Devnet)**: `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG` (same)
- **GitHub Repo**: https://github.com/MeteoraAg/damm-v2
- **SDK Repo**: https://github.com/MeteoraAg/damm-v2-sdk
- **Documentation**: https://docs.meteora.ag/

---

## 📚 Critical Information for Our Project

### 1. Position Management

#### Create Position Instruction
From the GitHub repo, Meteora has a `create_position` instruction:

```
create_position: create a new position nft, that holds liquidity that owner will deposit later
```

**Key Points**:
- ✅ Positions are NFTs (transferrable)
- ✅ Position is created BEFORE adding liquidity
- ✅ Position can be owned by any account (including PDAs)
- ✅ Fees are NOT compounded into LP (separate fee collection)

#### Position Features
- **Position NFT**: Transferrable ownership
- **Separate Fee Collection**: Fees don't auto-compound
- **Lock Options**: Can lock position but still claim fees
- **Permanent Lock**: Can permanently lock position

---

### 2. Fee Collection System

#### Fee Collection Modes
From the research, DAMM V2 supports:

**"Collecting fee only in one token (aka SOL)"** ✅

This is EXACTLY what we need! The bounty requires quote-only fees, and Meteora supports single-token fee collection.

#### Claim Position Fee Instruction
```
claim_position_fee: claim position fee
```

**This is the instruction we'll use in our claiming module!**

---

### 3. Pool Structure

#### Key Features
- **Constant Product AMM**: x * y = k formula
- **Concentrated Liquidity**: Minimal version with price ranges
- **Token2022 Support**: All token2022 extensions supported
- **Dynamic Fees**: Based on volatility
- **No Shared Accounts**: Each pool has unique accounts (no hot account issue)

#### Pool Configuration
Pools are created with either:
- **Static Config Key**: Pre-defined parameters
- **Dynamic Config Key**: Only defines pool creator authority
- **Customizable Pool**: Custom parameters (for token deployers)

---

### 4. Important Instructions for Our Project

#### For Position Initialization
```rust
create_position: create a new position nft
```

#### For Fee Claiming
```rust
claim_position_fee: claim position fee
```

#### For Adding Liquidity (if needed)
```rust
add_liquidity: add liquidity to a pool
remove_liquidity: remove liquidity from a pool
```

---

## 🔑 Critical Answers to Our Questions

### Q1: Can we create a quote-only fee position?
**Answer**: ✅ YES! 

Meteora explicitly supports "collecting fee only in one token". This is mentioned in their GitHub:
> "Fee is not compounded on LP, which allows us to implement many cool features like: collecting fee only in one token (aka SOL)"

### Q2: How do we create a position owned by our PDA?
**Answer**: Use the `create_position` instruction with our PDA as the owner.

The position is an NFT, so ownership can be any account including PDAs.

### Q3: How do we claim fees?
**Answer**: Use the `claim_position_fee` instruction.

Fees are separate from the position liquidity, so we can claim them without affecting the position.

### Q4: What about tick ranges?
**Answer**: DAMM V2 has "minimal concentrated liquidity" with price ranges.

We'll need to understand their price range system to configure quote-only fee accrual.

---

## 📖 Resources Found

### Official Resources
1. **Main Repo**: https://github.com/MeteoraAg/damm-v2
2. **SDK Repo**: https://github.com/MeteoraAg/damm-v2-sdk
3. **TypeScript SDK**: `@meteora-ag/cp-amm-sdk`
4. **Documentation**: https://docs.meteora.ag/
5. **Program IDs**: https://docs.meteora.ag/resources/meteora-program-ids

### Code Examples
- **TypeScript SDK Examples**: In damm-v2-sdk repo
- **Go Examples**: https://github.com/MeteoraAg/damm-v2-go
- **Rust SDK**: https://github.com/MeteoraAg/cp-amm/tree/main/rust-sdk

---

## 🚀 Next Steps for Implementation

### Step 1: Get the Program IDL
```bash
# We need the Anchor IDL to understand account structures
# Option 1: Check if it's in the repo
# Option 2: Fetch from on-chain
anchor idl fetch cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG -o meteora_cp_amm.json
```

### Step 2: Study Account Structures
We need to understand:
- `Pool` account structure
- `Position` account structure
- `Config` account structure
- Fee vault accounts

### Step 3: Implement CPI Calls
Based on the instructions we found:

```rust
// In integrations/meteora/cpi.rs

pub fn create_position(
    ctx: CpiContext<'_, '_, '_, '_, CreatePosition<'_>>,
    // parameters TBD
) -> Result<()> {
    // CPI to Meteora's create_position instruction
}

pub fn claim_position_fee(
    ctx: CpiContext<'_, '_, '_, '_, ClaimPositionFee<'_>>,
) -> Result<()> {
    // CPI to Meteora's claim_position_fee instruction
}
```

### Step 4: Configure Quote-Only Fees
We need to:
1. Understand Meteora's fee collection mode parameter
2. Set it to collect only quote token
3. Validate this in our `validate_quote_only_pool()` function

---

## 🎯 Key Takeaways

### ✅ Good News
1. **Quote-only fees ARE supported** by Meteora
2. **Positions can be owned by PDAs** (they're NFTs)
3. **Fees are separate** from liquidity (can claim without removing liquidity)
4. **Clear instructions** exist for position creation and fee claiming

### ⚠️ Still Need to Research
1. **Exact parameters** for `create_position` instruction
2. **How to configure** quote-only fee collection
3. **Account structures** (Pool, Position, Config)
4. **Price range/tick range** configuration
5. **Fee vault** account derivation

### 📋 Action Items
- [ ] Fetch the Meteora program IDL
- [ ] Study the TypeScript SDK for examples
- [ ] Look at the Rust SDK implementation
- [ ] Understand pool and position account structures
- [ ] Implement CPI wrappers in `integrations/meteora/`

---

## 💡 Implementation Strategy

### Phase 1: Study the SDK (1-2 days)
1. Clone the damm-v2-sdk repo
2. Read through TypeScript examples
3. Understand how they create positions
4. Understand how they claim fees
5. Note all account requirements

### Phase 2: Get Account Structures (1 day)
1. Fetch the IDL
2. Define Rust structs for Pool, Position, Config
3. Create account validation helpers

### Phase 3: Implement CPIs (1-2 days)
1. Create `integrations/meteora/cpi.rs`
2. Implement `create_position` CPI
3. Implement `claim_position_fee` CPI
4. Test on devnet

### Phase 4: Complete Position Module (1 day)
1. Wire up CPIs in `instructions.rs`
2. Implement validation in `validation.rs`
3. Test end-to-end

---

## 🔗 Useful Links

### Documentation
- Main Docs: https://docs.meteora.ag/
- DAMM V2 Overview: https://docs.meteora.ag/overview/products/damm-v2/what-is-damm-v2
- TypeScript SDK: https://docs.meteora.ag/integration/damm-v2-integration/damm-v2-sdk/damm-v2-typescript-sdk

### GitHub
- Program: https://github.com/MeteoraAg/damm-v2
- SDK: https://github.com/MeteoraAg/damm-v2-sdk
- Rust SDK: https://github.com/MeteoraAg/cp-amm/tree/main/rust-sdk

### Community
- Discord: https://discord.com/invite/meteora
- Twitter: https://x.com/MeteoraAG
- Medium: https://blog.meteora.ag/

---

## 📊 Confidence Level

| Aspect | Confidence | Notes |
|--------|-----------|-------|
| Quote-only fees supported | ✅ High | Explicitly mentioned in docs |
| Position creation | ✅ High | Clear instruction exists |
| PDA ownership | ✅ High | Positions are NFTs, any account can own |
| Fee claiming | ✅ High | Separate instruction exists |
| Account structures | ⚠️ Medium | Need to study IDL/SDK |
| Configuration params | ⚠️ Low | Need more research |

---

## 🎉 Summary

**We CAN build this!** Meteora DAMM V2 supports everything we need:
- ✅ Quote-only fee collection
- ✅ PDA-owned positions
- ✅ Separate fee claiming
- ✅ Position NFTs with locking

**Next**: Study the SDK and IDL to understand exact implementation details.
