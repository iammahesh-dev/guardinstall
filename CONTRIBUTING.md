# Contributing to guardinstall

## Adding Policy Profiles

Policy profiles live in `packages/policies/` as JSON files named `<package-name>.json`.

### Profile Structure

```json
{
  "package": "package-name",
  "versions": ">=1.0.0",
  "maintainers_verified": true,
  "expected_behavior": {
    "network": {
      "allowed_hosts": ["registry.npmjs.org"],
      "reason": "Downloads platform-specific binaries"
    },
    "filesystem": {
      "writes": ["./bin", "./lib"],
      "reason": "Extracts downloaded binaries"
    },
    "exec": false
  }
}
```

### Steps to Add a Profile

1. Create `packages/policies/<package-name>.json`
2. Run `pnpm test` to verify
3. Submit a PR with the new profile

## Testing

```bash
# Run all tests
pnpm test

# Run specific package tests
cd packages/cli && pnpm test
cd packages/policy-engine && pnpm test

# Run Rust tests
cd packages/sandbox && cargo test
```

## Building

```bash
pnpm install
pnpm build
```
