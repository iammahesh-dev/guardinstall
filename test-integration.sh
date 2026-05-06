#!/bin/bash
# Integration test for guardinstall

set -e

echo "🔒 guardinstall Integration Test"
echo "================================"

# Test 1: CLI builds
echo -e "\n✓ Test 1: Building all packages..."
pnpm build
echo "  -> Build successful"

# Test 2: Run policy-engine tests
echo -e "\n✓ Test 2: Policy Engine tests..."
cd packages/policy-engine
pnpm test
cd ../..

# Test 3: Run CLI tests
echo -e "\n✓ Test 3: CLI tests..."
cd packages/cli
pnpm test
cd ../..

# Test 4: Run Rust tests
echo -e "\n✓ Test 4: Rust sandbox tests..."
cd packages/sandbox
source ~/.cargo/env
cargo test
cd ../..

# Test 5: Check all policy profiles are valid JSON
echo -e "\n✓ Test 5: Validating policy profiles..."
for f in packages/policies/*.json; do
  node -e "JSON.parse(require('fs').readFileSync('$f'))"
  echo "  -> $f is valid"
done

echo -e "\n✓ All tests passed! guardinstall is ready."
echo "================================"
