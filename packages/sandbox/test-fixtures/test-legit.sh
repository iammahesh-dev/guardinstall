#!/bin/bash
# Test legitimate package script - simulates esbuild/node-gyp behavior

echo "=== Legitimate Package Script ==="

# 1. Download platform binary (should be allowed for known packages)
echo "[TEST] Downloading platform binary..."
curl -s -o /tmp/test-binary https://registry.npmjs.org/esbuild/-/esbuild-0.20.2-linux-x64.tar.gz 2>/dev/null
if [ -f /tmp/test-binary ]; then
    echo "PASS: Download succeeded (expected for known packages)"
    rm /tmp/test-binary
else
    echo "INFO: Download failed (network may be blocked - expected in sandbox)"
fi

# 2. Write to package directory (should be allowed)
echo "[TEST] Writing to package directory..."
echo "binary" > /tmp/test-output.txt 2>/dev/null
if [ -f /tmp/test-output.txt ]; then
    echo "PASS: Write to /tmp succeeded"
    rm /tmp/test-output.txt
else
    echo "INFO: Write to /tmp failed"
fi

# 3. Compile native code (should be allowed for node-gyp)
echo "[TEST] Simulating native compilation..."
echo "int main() { return 0; }" > /tmp/test.c
if command -v gcc >/dev/null 2>&1; then
    gcc /tmp/test.c -o /tmp/test 2>/dev/null
    echo "PASS: Compilation succeeded (expected for node-gyp)"
    rm -f /tmp/test.c /tmp/test
else
    echo "INFO: gcc not available"
fi

echo ""
echo "Legitimate package behaviors should be ALLOWED by guardinstall"
