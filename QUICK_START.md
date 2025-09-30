# Quick Start Guide - Position Module

## 🎯 What We Built

The **Position Management Module** - the first feature of the Meteora Fee Router that creates and maintains a quote-only LP position for fee accrual.

## 📂 File Structure Created

```
programs/meteora-fee-router/src/
├── lib.rs                          # Main entry point
├── errors.rs                       # Error codes
├── modules/
│   ├── mod.rs
│   └── position/                   # ← POSITION MODULE
│       ├── mod.rs                  # Module exports
│       ├── instructions.rs         # initialize_position()
│       ├── contexts.rs             # Account validation
│       ├── state.rs                # PositionMetadata
│       ├── validation.rs           # Quote-only checks
│       └── events.rs               # Events
├── integrations/
│   ├── mod.rs
│   └── meteora/
│       └── mod.rs                  # Meteora integration (TODO)
└── shared/
    ├── mod.rs
    └── constants.rs                # Seeds, limits
```

## 🔑 Key Components

### 1. Main Instruction
**File**: `modules/position/instructions.rs`
```rust
pub fn initialize_position(ctx: Context<InitializePosition>) -> Result<()>
```
Creates the honorary DAMM V2 position owned by program PDA.

### 2. Account Context
**File**: `modules/position/contexts.rs`
```rust
pub struct InitializePosition<'info> {
    pub authority: Signer<'info>,
    pub position_owner_pda: UncheckedAccount<'info>,
    pub pool: UncheckedAccount<'info>,
    // ... more accounts
}
```
Defines all required accounts with validation.

### 3. Validation
**File**: `modules/position/validation.rs`
- `validate_quote_only_pool()` - Ensure quote-only fees
- `identify_quote_mint()` - Find quote token
- `preflight_validation()` - Pre-creation checks

### 4. State
**File**: `modules/position/state.rs`
```rust
pub struct PositionMetadata {
    pub position: Pubkey,
    pub pool: Pubkey,
    pub quote_mint: Pubkey,
    // ...
}
```
Optional metadata storage.

### 5. Events
**File**: `modules/position/events.rs`
```rust
pub struct HonoraryPositionInitialized {
    pub position: Pubkey,
    pub pool: Pubkey,
    // ...
}
```
Emitted on successful initialization.

## 🚧 What Needs Implementation

### Critical TODOs (marked in code)

1. **Meteora Integration** (`integrations/meteora/`)
   - [ ] Define Pool and Position account structures
   - [ ] Implement CPI for position creation
   - [ ] Implement pool validation

2. **Validation Logic** (`modules/position/validation.rs`)
   - [ ] Implement `validate_quote_only_pool()`
   - [ ] Implement `identify_quote_mint()`
   - [ ] Complete `validate_token_order()`

3. **Instruction Handler** (`modules/position/instructions.rs`)
   - [ ] Wire up validation calls
   - [ ] Implement Meteora CPI
   - [ ] Complete position creation flow
   - [ ] Emit events

## 🎓 How to Work on This

### Step 1: Research Meteora (2-3 days)
```bash
# Find Meteora DAMM V2 documentation
# Questions to answer:
# - How to create a position?
# - What are the account structures?
# - How to configure quote-only fees?
# - What are tick ranges?
```

### Step 2: Implement Meteora Integration (2-3 days)
```bash
# Create these files:
programs/meteora-fee-router/src/integrations/meteora/
├── accounts.rs      # Pool, Position structs
├── cpi.rs          # CPI calls
└── validation.rs   # Pool validation
```

### Step 3: Complete Position Module (1-2 days)
```bash
# Fill in the TODOs in:
# - modules/position/validation.rs
# - modules/position/instructions.rs
# - modules/position/contexts.rs (if needed)
```

### Step 4: Test (1-2 days)
```bash
# Create test file:
tests/test_position_initialization.rs

# Test scenarios:
# - Successful position creation
# - Quote-only validation
# - Base fee rejection
# - Invalid pool config
```

## 🔍 Code Navigation Tips

### To add a new validation:
1. Add function to `modules/position/validation.rs`
2. Call it from `modules/position/instructions.rs`
3. Add error code to `errors.rs` if needed

### To add a new account:
1. Add to `modules/position/contexts.rs`
2. Add constraints with `#[account(...)]`
3. Use in `modules/position/instructions.rs`

### To add a new event:
1. Define in `modules/position/events.rs`
2. Emit in `modules/position/instructions.rs` with `emit!(...)`

### To add a constant:
1. Add to `shared/constants.rs`
2. Use anywhere with `use crate::shared::constants::*;`

## 🧪 Testing Strategy

### Unit Tests (in each module file)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation() {
        // Test validation logic
    }
}
```

### Integration Tests (in `tests/` directory)
```rust
// tests/test_position_initialization.rs
#[tokio::test]
async fn test_initialize_position() {
    // Full end-to-end test
}
```

## 📋 Checklist

### Structure ✅
- [x] All files created
- [x] Module exports configured
- [x] lib.rs wired up
- [x] Error codes defined
- [x] Constants defined

### Implementation ⚠️
- [ ] Meteora integration
- [ ] Validation logic
- [ ] Instruction handler
- [ ] CPI calls
- [ ] Event emission

### Testing ❌
- [ ] Unit tests
- [ ] Integration tests
- [ ] Error case tests
- [ ] Local validator tests

## 🎯 Success Criteria

When this module is complete, you should be able to:

1. ✅ Call `initialize_position` instruction
2. ✅ Create a DAMM V2 position owned by program PDA
3. ✅ Validate pool only accrues quote fees
4. ✅ Reject pools that would accrue base fees
5. ✅ Store position metadata
6. ✅ Emit initialization event
7. ✅ Handle all error cases gracefully

## 💡 Pro Tips

1. **Start with Meteora Research**: Don't code until you understand their API
2. **Use TODO Comments**: Mark what needs implementation
3. **Test Early**: Write tests as you implement
4. **One File at a Time**: Complete one file before moving to next
5. **Ask for Help**: If stuck on Meteora, ask in their Discord/docs

## 📞 Next Steps

1. Read `POSITION_MODULE_SUMMARY.md` for detailed info
2. Research Meteora DAMM V2 documentation
3. Implement `integrations/meteora/` files
4. Fill in TODOs in position module
5. Write tests
6. Move to next module (Policy)

## 🎉 You're Ready!

The structure is solid. Now it's time to fill in the Meteora-specific details and bring this module to life!

**Estimated Time to Complete**: 4-6 days
**Current Progress**: 30% (structure done, implementation pending)
