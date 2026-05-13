# @guardinstall/cli

A kernel-level behavioral sandbox for npm/pnpm/bun install scripts. Catches supply chain attacks at install time — before they execute.

## Why?

Every time you run `npm install`, **postinstall scripts execute with your full user permissions**. Malicious packages can:
- Steal your SSH keys, AWS credentials, and .env files
- Exfiltrate data to remote servers
- Install persistent backdoors
- Mine cryptocurrency

**guardinstall sandboxes each install script** using Linux kernel primitives (seccomp-BPF, Landlock, namespaces) to block dangerous behavior before it happens.

## Installation

```bash
npm install -g @guardinstall/cli
# or
pnpm add -g @guardinstall/cli
```

**Verify installation:**
```bash
guardinstall --version
gi --version
```

## Usage

### Replace npm install

```bash
# Instead of: npm install
guardinstall install

# Instead of: npm add express
guardinstall add express

# Global installs too
guardinstall add -g typescript
```

### Shortcut command

```bash
# 'gi' is a built-in alias for 'guardinstall'
gi install
gi add lodash
gi audit
gi check
```

### Check system compatibility

```bash
guardinstall check
```

Reports kernel version, Landlock ABI, seccomp availability, user namespaces, and whether the sandboxer binary is installed:

```
  ✓  Kernel        6.17.0 (Landlock ready)
  ✓  Landlock      ABI v3 (file read/write + file append)
  ✓  Seccomp       Available (mode 2)
  ✓  User Namespaces  Available (max: 91108)
  ✓  Sandboxer Binary  Found at /usr/lib/node_modules/...
  ✓  npm           v11.9.0
  ✓  pnpm          v10.30.1
  ℹ  bun           Not found
```

### CI Mode (fail-fast)

```bash
guardinstall install --ci
```

In CI environments, guardinstall will fail the build if any critical security issues are detected.

### Audit existing packages

```bash
guardinstall audit
```

Scan existing `node_modules` for packages with suspicious install scripts.

### Allowlisting packages (graceful fail)

Create a `guardinstall.json` in your project root to allow specific packages to bypass the sandbox:

```json
{
  "allowlist": ["@my-org/*", "trusted-tool"],
  "denylist": ["suspicious-pkg"],
  "concurrency": 4,
  "timeout": 30000,
  "ci": {
    "fail_on": "critical"
  }
}
```

Packages on the allowlist skip seccomp (network allowed) and Landlock restrictions. This is useful for packages that legitimately need network access (e.g., fetching pre-built binaries). Wildcards are supported (`@my-org/*`).

## How it works

1. **Install phase**: Runs `npm install --ignore-scripts` to download packages without running any scripts
2. **Detection phase**: Identifies packages with install scripts (postinstall, preinstall, install)
3. **Sandbox phase**: Runs each install script through a sandbox that uses:
   - **seccomp-BPF**: Blocks network syscalls (socket, connect)
   - **Landlock**: Restricts filesystem access to only /tmp and package directory
4. **Report phase**: Shows a security report with findings and prompts for action

## Performance

Sandbox overhead per package: **~125ms** (includes fork + Landlock + seccomp + exec + cleanup).

With default concurrency of 4, a project with 20 install scripts adds ~625ms to install time — well under 10% overhead for most projects (typical `npm install` takes 30-60s).

```
Benchmark (50 runs, release build):
  avg    124.90ms  per sandboxed script
  p50    125ms
  p95    129ms
  BPF filter build:  < 0.02ms
```

## Security events blocked

- Network access (curl, wget, DNS lookups, socket syscalls)
- Reading sensitive files (/etc/passwd, ~/.ssh/, ~/.aws/)
- Writing outside package directory
- Suspicious command execution

## Requirements

- **Linux**: Kernel 5.13+ for Landlock, seccomp-BPF (kernel 3.5+)
- **macOS**: Seatbelt (experimental, untested)
- **Windows**: Job Objects (experimental)
- Node.js 18+
- npm, pnpm, or bun

## License

MIT
