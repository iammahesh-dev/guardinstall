# AGENTS.md - Context for Future Sessions (Updated 2026-05-07)

> **Project:** guardinstall - Kernel-level behavioral sandbox for npm/pnpm/bun install scripts  
> **Repo:** `git@github.com:iammahesh-dev/guardinstall.git`  
> **Current Branch:** `dev` (all work happens here)  
> **Main Branch:** `main` (clean, only updated when explicitly requested)  
> **Last Updated:** 2026-05-07

---

## Project Overview

guardinstall catches supply chain attacks at install time by sandboxing npm package install scripts using kernel-level security primitives:
- **Linux:** seccomp-BPF + namespaces (+ Landlock stubbed)
- **macOS:** Seatbelt (sandbox-exec)
- **Windows:** Job Objects + restricted tokens

---

## Current Status (as of 2026-05-07)

### Git State
- **Branch:** `dev` (ALL development happens here)
- **Main:** `main` (clean at commit `dc39488`, NOT updated unless explicitly requested)
- **Remote:** `origin` → `git@github.com:iammahesh-dev/guardinstall.git`
- **Dev commits ahead of main:** All main commits rebased into dev ✅

### Buildspec Completion (ALL GAPS FIXED - but seccomp BPF needs work)
- Phase 1: CLI Foundation — DONE
- Phase 2: Linux Sandbox Core — **IN PROGRESS** (seccomp BPF debugging)
- Phase 3: Policy Engine + UX — DONE
- Phase 4: Cross-Platform Support — DONE
- Phase 5: Community & Ecosystem — DONE

### Seccomp BPF Issue (IN PROGRESS)
**Problem:** BPF filters cause SIGSEGV/EINVAL in Rust but work in C.

**Working:**
- ✅ `sandboxer_working_c` (Rust replica of C) - **WORKS perfectly**
  - Uses `syscall(SYS_execve, ...)` instead of `Command::new().status()`
  - Blocks execve with EPERM as expected
  - Commit: `6f55bad` on `dev`

**Not Working:**
- ❌ Main `sandboxer.rs` with BPF filter causes EINVAL
- ❌ `sandboxer_seccomp`, `sandboxer_block`, etc. - all fail with EINVAL

**Key Finding:**
- C minimal filter (allow-all) → Works ✅
- C filter blocking execve → Works ✅  
- C filter blocking socket(AF_INET) → Fails with EINVAL (BPF complexity issue)
- Rust `sandboxer_working_c` (exact C replica) → Works ✅
- Rust `sandboxer.rs` (same BPF) → Fails ❌

**Hypothesis:** Issue is in how Rust passes `sock_fprog` struct to `prctl`.

### Test Binaries Created (on `dev` branch)
- `sandboxer_simple` - Basic fork+exec ✅
- `sandboxer_allow_all` - BPF allow-all ✅  
- `sandboxer_exec` - Using `execl()` directly ✅
- `sandboxer_netns` - Network namespace only ✅ (requires root)
- `sandboxer_working_c` - **WORKING Rust replica of C** ✅
- `sandboxer_seccomp` / `sandboxer_block` / `sandboxer_working` - BPF attempts ❌

---

## TODO / Next Steps

### HIGH PRIORITY
1. **Fix seccomp BPF in main `sandboxer.rs`**
   - `sandboxer_working_c` works → use same approach in `sandboxer.rs`
   - Key: Use `syscall(SYS_execve, ...)` instead of `Command::new().status()`
   - Simplify BPF filter: start with allow-all, then add one rule at a time
   - **Goal:** Get BPF filter actually blocking malicious behavior

2. **Test malicious script blocking**
   - Create test: `malicious.sh` tries `curl|sh`, reading `/etc/passwd`
   - Verify sandbox blocks network access / sensitive file reads
   - Need to fix `sandboxer.rs` first

3. **Network namespace isolation**
   - Currently requires root (`sudo` or `CAP_SYS_ADMIN`)
   - Consider using `unshare(CLONE_NEWNET)` with proper capabilities
   - Alternative: Use Landlock for filesystem + seccomp for network

### MEDIUM PRIORITY
4. **Landlock filesystem restriction**
   - Currently stubbed (API complexity - `RestrictionStatus` struct not enum)
   - Fix API usage: `RestrictionStatus { ruleset: RulesetStatus::FullyEnforced, no_new_privs: true }`
   - Block access to `/etc/passwd`, `~/.ssh/`, other sensitive paths

5. **Commit and push working seccomp to `dev`**
   - Once `sandboxer.rs` works, commit to `dev`
   - Do NOT merge to `main` unless explicitly requested

### LOW PRIORITY
6. **ARM64 seccomp support**
   - Add `cfg!(target_arch = "aarch64")` support
   - Need to test on ARM64 machine or emulator

7. **macOS Seatbelt integration**
   - Dispatch in `sandboxer.rs` via `#[cfg(target_os = "macos")]`
   - Test on macOS machine

---

## Key Commands

```bash
# Build sandboxer binary (on dev branch)
source ~/.cargo/env
cd /home/mahi/app/guardinstall/packages/sandbox
cargo build --release --bin sandboxer

# Test CLI
cd /home/mahi/app/guardinstall/packages/cli && pnpm test

# Test Rust
source ~/.cargo/env && cd /home/mahi/app/guardinstall/packages/sandbox && cargo test

# Test policy engine
cd /home/mahi/app/guardinstall/packages/policy-engine && pnpm test

# Git workflow (ALL development on dev, main stays clean)
cd /home/mahi/app/guardinstall
git checkout dev    # Always work on dev
git push origin dev    # Push only dev to remote
# NEVER push to main unless explicitly asked
```

---

## GAPS — STATUS

### ✅ FIXED (in previous sessions)
- **GAP 1**: Seccomp applied in sandboxer.rs binary — **IN PROGRESS** (BPF issues)
- **GAP 2**: Network namespace isolation — DONE (requires root)
- **GAP 3**: Landlock stubbed — **IN PROGRESS** (API issues)
- **GAP 4**: `add` command order — DONE
- **GAP 5**: Script path construction — DONE
- **GAP 6**: `isExternalIP()` — DONE
- **GAP 7**: Policy allowlist wired up — DONE
- **GAP 8**: macOS Seatbelt dispatch — DONE (not tested)
- **GAP 9**: Tests assert blocking — DONE (needs working seccomp)
- **GAP 10**: seccomp ARM64 support — PENDING

### 🔄 IN PROGRESS
- **GAP 1**: Seccomp BPF filter — `sandboxer_working_c` works, need to apply to main `sandboxer.rs`

---

## Known Issues

### Seccomp BPF Filter
- **Issue:** BPF filter causes EINVAL/SIGSEGV in Rust but works in C
- **Workaround:** `sandboxer_working_c` uses `syscall(SYS_execve, ...)` 
- **Fix needed:** Apply same approach to main `sandboxer.rs`
- **Tracking:** https://github.com/iammahesh-dev/guardinstall/issues (create issue)

### Landlock API
- **Issue:** `RestrictionStatus` is a struct with named fields, not an enum
- **Fix:** Use `RestrictionStatus { ruleset: RulesetStatus::FullyEnforced, no_new_privs: true }`
- **Status:** Fixed in `landlock.rs` but not integrated into `sandboxer.rs` (binary can't access library)

### Network Namespace
- **Issue:** Requires `CAP_SYS_ADMIN` or root
- **Workaround:** Run with `sudo setcap cap_sys_admin+ep ./target/release/sandboxer`
- **Alternative:** Use Landlock for filesystem restrictions instead

---

## Useful Links

- **Buildspec:** `/home/mahi/app/guardinstall/guardinstall-buildspec.md`
- **GitHub Repo:** `https://github.com/iammahesh-dev/guardinstall`
- **GitHub Actions:** `https://github.com/iammahesh-dev/guardinstall/actions`
- **Issues:** `https://github.com/iammahesh-dev/guardinstall/issues`
- **Knowledge Graph:** `/home/mahi/app/guardinstall/graphify-out/graph.html` (open in browser)

---

## Quick Start for New Session

```bash
cd /home/mahi/app/guardinstall
git checkout dev && git status && git log --oneline -3
source ~/.cargo/env && cd packages/sandbox && cargo test
cd ../cli && pnpm test
```

**Important:** 
- ALL development happens on `dev` branch
- `main` is kept clean (only updated when explicitly requested)
- Seccomp BPF debugging is IN PROGRESS (see TODO above)
- `sandboxer_working_c` is the working reference implementation

---

## Relevant Files

- `/home/mahi/app/guardinstall/packages/sandbox/src/bin/sandboxer.rs` - Main binary (IN PROGRESS)
- `/home/mahi/app/guardinstall/packages/sandbox/src/bin/sandboxer_working_c.rs` - **WORKING** reference ✅
- `/home/mahi/app/guardinstall/packages/cli/src/sandboxer.ts` - Invokes `sandboxer` binary
- `/home/mahi/app/guardinstall/packages/cli/src/orchestrator.ts` - Uses `runSandboxed()`
- `/home/mahi/app/guardinstall/packages/sandbox/src/linux/seccomp.rs` - Library seccomp (unused)
- `/home/mahi/app/guardinstall/packages/sandbox/src/linux/landlock.rs` - Landlock (stubbed)
- `/home/mahi/app/guardinstall/AGENTS.md` - This file

---

## Session Summary (2026-05-07)

### What We Did
1. ✅ Rebased `dev` onto `main` (all main commits now in dev)
2. ✅ Pushed updated `dev` to remote
3. ✅ Created `sandboxer_working_c.rs` (WORKING Rust replica of C)
4. ✅ Verified `sandboxer_working_c` blocks execve correctly
5. ✅ Updated this AGENTS.md with current status

### What's Left
1. 🔄 Apply `sandboxer_working_c` approach to main `sandboxer.rs`
2. 🔄 Get BPF filter actually blocking malicious behavior
3. 🔄 Test malicious script (`malicious.sh`) is blocked
4. 🔄 Commit working seccomp to `dev` (do NOT merge to `main`)

### Key Learning
- C seccomp BPF works perfectly
- Rust `sandboxer_working_c` (exact C replica) also works
- Main `sandboxer.rs` fails → need to use `syscall(SYS_execve, ...)` approach
- Network namespace requires root (use `sudo` or capabilities)
