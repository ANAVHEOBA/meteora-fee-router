programs/meteora-fee-router/src/
├── lib.rs                              # Entry point, routes to modules
│
├── modules/                            # Feature modules
│   ├── mod.rs                          # Export all modules
│   │
│   ├── position/                       # Position management module
│   │   ├── mod.rs                      # Module exports
│   │   ├── instructions.rs             # initialize_position
│   │   ├── contexts.rs                 # InitializePosition context
│   │   ├── state.rs                    # PositionMetadata (if needed)
│   │   ├── validation.rs               # Quote-only validation
│   │   └── events.rs                   # PositionInitialized event
│   │
│   ├── policy/                         # Policy management module
│   │   ├── mod.rs
│   │   ├── instructions.rs             # initialize_policy, update_policy
│   │   ├── contexts.rs                 # Policy contexts
│   │   ├── state.rs                    # PolicyState account
│   │   └── validation.rs               # Policy validation
│   │
│   ├── distribution/                   # Fee distribution module
│   │   ├── mod.rs
│   │   ├── instructions.rs             # distribute_fees (main crank)
│   │   ├── contexts.rs                 # DistributeFees context
│   │   ├── state.rs                    # ProgressState account
│   │   ├── calculator.rs               # Fee share calculations
│   │   ├── pagination.rs               # Page management
│   │   └── events.rs                   # Distribution events
│   │
│   └── claiming/                       # Fee claiming module
│       ├── mod.rs
│       ├── instructions.rs             # claim_fees_from_position
│       ├── contexts.rs                 # ClaimFees context
│       └── treasury.rs                 # Treasury management
│
├── integrations/                       # External program interfaces
│   ├── mod.rs
│   ├── meteora/                        # Meteora integration
│   │   ├── mod.rs
│   │   ├── cpi.rs                      # CPI calls
│   │   ├── accounts.rs                 # Account structures
│   │   └── validation.rs               # Pool validation
│   │
│   └── streamflow/                     # Streamflow integration
│       ├── mod.rs
│       ├── reader.rs                   # Read stream data
│       └── accounts.rs                 # Stream account structures
│
├── shared/                             # Shared utilities
│   ├── mod.rs
│   ├── math.rs                         # Math functions
│   ├── time.rs                         # Time utilities
│   ├── pda.rs                          # PDA helpers
│   └── constants.rs                    # Global constants
│
└── errors.rs                           # All error codes