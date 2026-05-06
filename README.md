# guardinstall

A kernel-level behavioral sandbox for npm/pnpm/bun install scripts.  
Catches supply chain attacks at install time — before they execute.

[![CI](https://github.com/guardinstall/guardinstall/actions/workflows/ci.yml/badge.svg)](https://github.com/guardinstall/guardinstall/actions/workflows/ci.yml)
[![npm version](https://badge.fury.io/js/guardinstall.svg)](https://www.npmjs.com/package/guardinstall)

## Why guardinstall?

Every time you run `npm install`, **postinstall scripts execute with your full user permissions**. Malicious packages can:
- Steal your SSH keys, AWS credentials, and .env files
- Exfiltrate data to remote servers
- Install persistent backdoors
- Mine cryptocurrency

**guardinstall sandboxes each install script** using Linux kernel primitives (seccomp-BPF, namespaces, Landlock) to block dangerous behavior before it happens.

## Installation

```bash
npm install -g guardinstall
# or
pnpm add -g guardinstall
```

## Usage

### Replace npm install

```bash
# Instead of: npm install
guardinstall install

# Instead of: npm add express
guardinstall add express

# Use with pnpm
guardinstall --pm pnpm add lodash

# Use with bun
guardinstall --pm bun install
```

### CI Mode (fail-fast)

```bash
guardinstall --ci install
```

### Audit existing node_modules

```bash
guardinstall audit
```

## How It Works

```
Developer runs: $ npx guardinstall add express
                        │
                        ▼
┌───────────────────────────────────────────┐
│  LAYER 1 — CLI WRAPPER (TypeScript)      │
│  • Intercepts install command             │
│  • Resolves dependency tree via arborist │
│  • Identifies packages with install scripts│
└──────────────┬────────────────────────────┘
               │ spawns sandbox per script
               ▼
┌───────────────────────────────────────────┐
│  LAYER 2 — BEHAVIORAL SANDBOX (Rust)   │
│  Linux: seccomp-BPF + namespaces + Landlock│
│  macOS: sandbox-exec (Seatbelt)          │
│  Windows: Job Objects + restricted tokens  │
└──────────────┬────────────────────────────┘
               │ emits JSON event stream
               ▼
┌───────────────────────────────────────────┐
│  LAYER 3 — POLICY ENGINE (TypeScript)    │
│  • Scores anomalies by severity           │
│  • Applies community behavior profiles    │
│  • Interactive prompt or CI fail-fast     │
└───────────────────────────────────────────┘
```

## Supported Platforms

| Platform | Minimum Version | Sandbox Technology |
|-----------|-----------------|-------------------|
| Linux     | Kernel 5.13+    | seccomp-BPF + namespaces + Landlock |
| macOS     | 10.15+          | Seatbelt (sandbox-exec) |
| Windows   | 10+             | Job Objects (detect mode) |

## Configuration

Create a `.guardinstallrc` file in your project root:

```json
{
  "allowlist": ["esbuild", "sharp"],
  "ci": true,
  "severity_threshold": "HIGH"
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to add policy profiles for popular packages.

## License

MIT
