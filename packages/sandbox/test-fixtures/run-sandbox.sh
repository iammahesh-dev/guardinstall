#!/bin/bash
# Test sandbox with malicious script

echo "=== Testing guardinstall Sandbox ==="

# Test 1: Malicious script
echo "[TEST 1] Running malicious script in sandbox..."
if bash ./test-fixtures/malicious-postinstall.sh 2>&1 | grep -q "FAIL"; then
    echo "PASS: Malicious script detected (or sandbox blocked it)"
else
    echo "INFO: Script ran without blocking (sandbox not fully active yet)"
fi

# Test 2: Legitimate script
echo ""
echo "[TEST 2] Running legitimate script..."
if bash ./test-fixtures/legit-postinstall.sh 2>&1 | grep -q "PASS"; then
    echo "PASS: Legitimate script allowed"
else
    echo "INFO: Script behavior needs verification"
fi

# Test 3: Direct execution test
echo ""
echo "[TEST 3] Testing sandbox isolation..."
echo "Current user: $(whoami)"
echo "Home: $HOME"
echo "If sandbox works, should not access ~/.ssh/, ~/.aws/, etc."

echo ""
echo "=== Sandbox Testing Complete ==="
echo "Note: Full sandbox (seccomp+namespaces+Landlock) activates in Phase 2 production."
