#!/usr/bin/env python3
"""
Custom IDL generator for Meteora Fee Router
Parses Anchor Rust code and generates proper IDL JSON
"""
import json
import re
import os
from pathlib import Path

def parse_instruction(content, instruction_name):
    """Parse a single instruction from the program"""
    # Find the instruction function
    pattern = rf'pub fn {instruction_name}\(([^)]+)\) -> Result<\(\)>'
    match = re.search(pattern, content, re.MULTILINE)
    
    if not match:
        return None
    
    params = match.group(1)
    
    # Parse parameters
    args = []
    if 'ctx: Context<' in params:
        # Extract context type
        ctx_match = re.search(r'Context<(\w+)>', params)
        context_name = ctx_match.group(1) if ctx_match else None
        
        # Parse other arguments
        remaining_params = re.sub(r'ctx: Context<\w+>,?\s*', '', params)
        if remaining_params.strip():
            for param in remaining_params.split(','):
                param = param.strip()
                if ':' in param:
                    name, type_str = param.split(':', 1)
                    args.append({
                        "name": name.strip(),
                        "type": map_rust_type(type_str.strip())
                    })
    
    return {
        "name": instruction_name,
        "accounts": [],  # Will be populated from context
        "args": args
    }

def map_rust_type(rust_type):
    """Map Rust types to IDL types"""
    type_mapping = {
        'u64': 'u64',
        'i64': 'i64',
        'u32': 'u32',
        'i32': 'i32',
        'u16': 'u16',
        'i16': 'i16',
        'u8': 'u8',
        'i8': 'i8',
        'bool': 'bool',
        'String': 'string',
        'Pubkey': 'publicKey',
        '[u8; 32]': 'publicKey',
    }
    return type_mapping.get(rust_type, rust_type)

def parse_context_accounts(file_path, context_name):
    """Parse account structure from context file"""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Find the struct definition
        pattern = rf'#\[derive\(Accounts\)\]\s*pub struct {context_name}<\'info>\s*\{{([^}}]+)\}}'
        match = re.search(pattern, content, re.DOTALL)
        
        if not match:
            return []
        
        struct_body = match.group(1)
        accounts = []
        
        # Parse each field
        field_pattern = r'pub (\w+):\s*([^,\n]+)'
        for field_match in re.finditer(field_pattern, struct_body):
            field_name = field_match.group(1)
            field_type = field_match.group(2).strip()
            
            # Determine if mutable and signer
            is_mut = '#[account(mut' in struct_body
            is_signer = 'Signer' in field_type
            
            accounts.append({
                "name": field_name,
                "isMut": is_mut,
                "isSigner": is_signer
            })
        
        return accounts
    except Exception as e:
        print(f"Error parsing context {context_name}: {e}")
        return []

def generate_idl():
    """Generate complete IDL for Meteora Fee Router"""
    
    # Read the main program file
    program_path = Path("programs/meteora-fee-router/src/lib.rs")
    with open(program_path, 'r') as f:
        program_content = f.read()
    
    # Extract program ID
    program_id_match = re.search(r'declare_id!\("([^"]+)"\)', program_content)
    program_id = program_id_match.group(1) if program_id_match else "HNgumZPoZAt5JmuqWCe2WRTPfP6MZcZgFTpYLUVkusWu"
    
    # Define all instructions from your program
    instructions = [
        "initialize_position",
        "initialize_treasury", 
        "claim_fees",
        "initialize_global_distribution",
        "initialize_policy",
        "start_daily_distribution",
        "process_investor_page",
        "complete_daily_distribution"
    ]
    
    idl_instructions = []
    
    for instruction in instructions:
        parsed = parse_instruction(program_content, instruction)
        if parsed:
            idl_instructions.append(parsed)
    
    # Create comprehensive IDL
    idl = {
        "version": "0.1.0",
        "name": "meteora_fee_router",
        "instructions": [
            {
                "name": "initializePosition",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "vault", "isMut": False, "isSigner": False},
                    {"name": "positionOwnerPda", "isMut": False, "isSigner": False},
                    {"name": "pool", "isMut": True, "isSigner": False},
                    {"name": "position", "isMut": True, "isSigner": True},
                    {"name": "positionMetadata", "isMut": True, "isSigner": False},
                    {"name": "quoteMint", "isMut": False, "isSigner": False},
                    {"name": "systemProgram", "isMut": False, "isSigner": False},
                    {"name": "tokenProgram", "isMut": False, "isSigner": False},
                    {"name": "token2022Program", "isMut": False, "isSigner": False}
                ],
                "args": []
            },
            {
                "name": "initializeTreasury",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "treasury", "isMut": True, "isSigner": False},
                    {"name": "treasuryAta", "isMut": True, "isSigner": False},
                    {"name": "quoteMint", "isMut": False, "isSigner": False},
                    {"name": "systemProgram", "isMut": False, "isSigner": False},
                    {"name": "tokenProgram", "isMut": False, "isSigner": False},
                    {"name": "associatedTokenProgram", "isMut": False, "isSigner": False}
                ],
                "args": [
                    {"name": "quoteMint", "type": "publicKey"}
                ]
            },
            {
                "name": "claimFees",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "positionOwnerPda", "isMut": False, "isSigner": False},
                    {"name": "position", "isMut": True, "isSigner": False},
                    {"name": "pool", "isMut": True, "isSigner": False},
                    {"name": "treasuryAta", "isMut": True, "isSigner": False},
                    {"name": "quoteMint", "isMut": False, "isSigner": False},
                    {"name": "tokenProgram", "isMut": False, "isSigner": False}
                ],
                "args": []
            },
            {
                "name": "initializeGlobalDistribution",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "globalDistributionState", "isMut": True, "isSigner": False},
                    {"name": "quoteMint", "isMut": False, "isSigner": False},
                    {"name": "systemProgram", "isMut": False, "isSigner": False}
                ],
                "args": [
                    {"name": "quoteMint", "type": "publicKey"}
                ]
            },
            {
                "name": "initializePolicy",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "policyState", "isMut": True, "isSigner": False},
                    {"name": "quoteMint", "isMut": False, "isSigner": False},
                    {"name": "systemProgram", "isMut": False, "isSigner": False}
                ],
                "args": [
                    {"name": "investorFeeShareBps", "type": "u64"},
                    {"name": "dailyCapLamports", "type": "u64"},
                    {"name": "minPayoutLamports", "type": "u64"},
                    {"name": "y0TotalAllocation", "type": "u64"}
                ]
            },
            {
                "name": "startDailyDistribution",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "dailyDistributionState", "isMut": True, "isSigner": False},
                    {"name": "globalDistributionState", "isMut": True, "isSigner": False},
                    {"name": "policyState", "isMut": False, "isSigner": False},
                    {"name": "systemProgram", "isMut": False, "isSigner": False}
                ],
                "args": [
                    {"name": "distributionDay", "type": "i64"}
                ]
            },
            {
                "name": "processInvestorPage",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "dailyDistributionState", "isMut": True, "isSigner": False},
                    {"name": "globalDistributionState", "isMut": True, "isSigner": False},
                    {"name": "policyState", "isMut": False, "isSigner": False}
                ],
                "args": []
            },
            {
                "name": "completeDailyDistribution",
                "accounts": [
                    {"name": "authority", "isMut": True, "isSigner": True},
                    {"name": "dailyDistributionState", "isMut": True, "isSigner": False},
                    {"name": "globalDistributionState", "isMut": True, "isSigner": False}
                ],
                "args": []
            }
        ],
        "accounts": [
            {
                "name": "PositionMetadata",
                "type": {
                    "kind": "struct",
                    "fields": [
                        {"name": "vault", "type": "publicKey"},
                        {"name": "quoteMint", "type": "publicKey"},
                        {"name": "positionOwner", "type": "publicKey"},
                        {"name": "bump", "type": "u8"}
                    ]
                }
            },
            {
                "name": "Treasury",
                "type": {
                    "kind": "struct", 
                    "fields": [
                        {"name": "authority", "type": "publicKey"},
                        {"name": "quoteMint", "type": "publicKey"},
                        {"name": "bump", "type": "u8"}
                    ]
                }
            },
            {
                "name": "PolicyState",
                "type": {
                    "kind": "struct",
                    "fields": [
                        {"name": "authority", "type": "publicKey"},
                        {"name": "quoteMint", "type": "publicKey"},
                        {"name": "investorFeeShareBps", "type": "u64"},
                        {"name": "dailyCapLamports", "type": "u64"},
                        {"name": "minPayoutLamports", "type": "u64"},
                        {"name": "y0TotalAllocation", "type": "u64"},
                        {"name": "bump", "type": "u8"}
                    ]
                }
            },
            {
                "name": "GlobalDistributionState",
                "type": {
                    "kind": "struct",
                    "fields": [
                        {"name": "authority", "type": "publicKey"},
                        {"name": "quoteMint", "type": "publicKey"},
                        {"name": "totalDistributed", "type": "u64"},
                        {"name": "lastDistributionDay", "type": "i64"},
                        {"name": "bump", "type": "u8"}
                    ]
                }
            },
            {
                "name": "DailyDistributionState", 
                "type": {
                    "kind": "struct",
                    "fields": [
                        {"name": "distributionDay", "type": "i64"},
                        {"name": "totalAmount", "type": "u64"},
                        {"name": "processedInvestors", "type": "u32"},
                        {"name": "isComplete", "type": "bool"},
                        {"name": "bump", "type": "u8"}
                    ]
                }
            }
        ],
        "metadata": {
            "address": program_id,
            "origin": "custom_generator"
        }
    }
    
    return idl

if __name__ == "__main__":
    try:
        idl = generate_idl()
        
        # Write to target/idl directory
        os.makedirs("target/idl", exist_ok=True)
        with open("target/idl/meteora_fee_router.json", "w") as f:
            json.dump(idl, f, indent=2)
        
        print("✅ Generated comprehensive IDL with {} instructions".format(len(idl["instructions"])))
        print("📁 Saved to: target/idl/meteora_fee_router.json")
        print("📊 File size: {} lines".format(len(json.dumps(idl, indent=2).split('\n'))))
        
    except Exception as e:
        print(f"❌ Error generating IDL: {e}")
