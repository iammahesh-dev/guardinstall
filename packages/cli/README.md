# @guardinstall/cli

A kernel-level behavioral sandbox for npm/pnpm/bun install scripts. Catches supply chain attacks at install time — before they execute.

## Why?

Every time you run `npm install`, **postinstall scripts execute with your full user permissions**. Malicious packages can:
- Steal your SSH keys, AWS credentials, and .env files
- Exfiltrate data to remote servers
- Install persistent backdoors
- Mine cryptocurrency

**guardinstall sandboxes each install script** using Linux kernel primitives (seccomp-BPF, namespaces, Landlock) to block dangerous behavior before it happens.

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
```

### Shortcut command

```bash
# 'gi' is a built-in alias for 'guardinstall'
gi install
gi add lodash
gi audit
```

### CI Mode (fail-fast)

```bash
guardinstall --ci install
```

In CI environments, guardinstall will fail the build if any malicious behavior is detected.

### Audit existing packages

```bash
guardinstall audit
```

Scan existing `node_modules` for packages with suspicious install scripts.

## How it works

1. **Install phase**: Runs `npm install --ignore-scripts` to download packages without running any scripts
2. **Detection phase**: Identifies packages with install scripts (postinstall, preinstall, install)
3. **Sandbox phase**: Runs each install script through a sandbox that uses:
   - **seccomp-BPF**: Blocks network syscalls (socket, connect)
   - **Landlock**: Restricts filesystem access to only /tmp and package directory
4. **Report phase**: Shows a security report with findings and prompts for action

## Security events blocked

- Network access (curl, wget, DNS lookups)
- Reading sensitive files (/etc/passwd, ~/.ssh/, ~/.aws/)
- Writing outside package directory
- Suspicious command execution

## Requirements

- Linux kernel 5.13+ for Landlock support
- Node.js 18+
- npm, pnpm, or bun

## License

MIT
