#!/bin/bash

# Meteora Fee Router Build Script
# Builds the program and generates IDL using custom generator

set -e

echo "🔨 Building Meteora Fee Router..."

# Build the program binary (skips IDL generation)
echo "📦 Building program binary..."
anchor build --no-idl

# Generate IDL using custom generator
echo "🔧 Generating IDL with custom generator..."
python3 generate_idl.py

echo "✅ Build complete!"
echo "📁 Program binary: target/deploy/meteora_fee_router.so"
echo "📄 IDL file: target/idl/meteora_fee_router.json"
echo ""
echo "🚀 Ready to deploy:"
echo "   solana program deploy target/deploy/meteora_fee_router.so"
