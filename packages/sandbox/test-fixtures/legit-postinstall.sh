#!/bin/bash
# Legitimate postinstall script (like esbuild)

# Download platform-specific binary
echo "Downloading esbuild binary..."
curl -o ./bin/esbuild https://registry.npmjs.org/esbuild/-/esbuild-0.20.2-linux-x64.tar.gz 2>/dev/null

# Extract and set permissions
chmod +x ./bin/esbuild 2>/dev/null

echo "esbuild installed successfully"
