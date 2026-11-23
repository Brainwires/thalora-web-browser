# Thalora Environment Variables

This document lists all environment variables used by Thalora and their purposes.

---

## Security

### `THALORA_MASTER_PASSWORD`

**Required for**: Credential encryption/decryption
**Type**: String (minimum 32 characters)
**Default**: None (must be set)

Master password used for AES-256-GCM encryption of stored credentials. This password is used to derive encryption keys via Argon2id.

**Example**:
```bash
export THALORA_MASTER_PASSWORD="your-secure-password-at-least-32-chars"
```

**Security Notes**:
- Must be at least 32 characters long
- Used with Argon2id key derivation (64MB memory, 3 iterations)
- Each encryption produces different output (random salt/nonce)
- Without this variable, credential operations will fail

---

## Features

### `THALORA_ENABLE_VAULT`

**Required for**: the credential vault tools
**Type**: Boolean (`1`, `true`, `yes`, `on`)
**Default**: Disabled

Lists the vault tools (`vault_store_credential`, `vault_list_credentials`,
`vault_delete_credential`, `browser_fill_credential`) and persists the vault to
disk. When disabled, the vault is in-memory only. The retired
`THALORA_ENABLE_AI_MEMORY` is still accepted as an alias.

No tool returns a stored secret: `browser_fill_credential` types it into a page
of the credential's own origin server-side.

### `THALORA_VAULT_PATH`

**Default**: `~/.thalora/credentials.json`

Where the vault is stored, on the real filesystem with owner-only permissions
(`0600` on Unix). Passwords inside it are encrypted with the master password.

---

## MCP Tool Selection

| Variable | Default | Effect |
|---|---|---|
| `THALORA_MCP_MODE` | `minimal` | `minimal` lists only `snapshot_url` and `web_search`; `full` lists the categories enabled below |
| `THALORA_PRESET` | unset | `brainclaw` enables sessions, CDP and search in full mode and adds agent-friendly alias tools |
| `THALORA_ENABLE_SCRAPING` | `true` | `snapshot_url` |
| `THALORA_ENABLE_SEARCH` | `false` | `web_search`, `image_search` |
| `THALORA_ENABLE_SESSIONS` | `false` | Browser session, navigation and form automation tools |
| `THALORA_ENABLE_CDP` | `false` | `cdp_runtime_evaluate` (also enables sessions). The tool currently returns an error: there is no JavaScript engine until V8 is wired in |
| `THALORA_STEALTH_DELAYS` | `false` | Add 1.5–5 s of random "human-like" delay to every navigation (anti-bot evasion) |
| `THALORA_DISABLE_RATE_LIMIT` | unset | `1` disables MCP tool rate limiting (trusted local agents, tests) |
| `THALORA_ALLOW_LOOPBACK` | unset | Test-only: allow navigation to 127.0.0.1/localhost. Honoured only in debug builds or with the `test-hooks` feature |

## Engine Configuration

There is currently no JavaScript engine to configure: the Boa engine was
removed and V8 (the Brainwires deno fork) is being wired in. `THALORA_ENGINE`
is no longer read. Tools that need JavaScript return
`JavaScript execution is unavailable: the Boa engine was removed; V8 support is in progress`.

---

## Testing

### Test-specific variables used during test runs:

**Cargo Test Environment**:
```bash
# Required for running security tests
export THALORA_MASTER_PASSWORD="test_master_password_min_32chars_secure"

# Run tests with the credential vault enabled
export THALORA_ENABLE_VAULT=1
cargo test
```

---

## Complete Example

Here's a complete example for running Thalora with all recommended settings:

```bash
#!/bin/bash

# Security (REQUIRED for credential operations)
export THALORA_MASTER_PASSWORD="$(openssl rand -base64 48)"

# Features (OPTIONAL - enable as needed)
export THALORA_ENABLE_VAULT=1

# Run Thalora
cargo run --release
```

---

## Persistence Locations

When features are enabled, Thalora uses these directories:

- Credential vault: `~/.thalora/credentials.json` (passwords encrypted), or `THALORA_VAULT_PATH`

---

## Security Best Practices

1. **Master Password**:
   - Generate strong random passwords: `openssl rand -base64 48`
   - Store in environment variables, not in code
   - Use different passwords for dev/staging/production

2. **Credential vault**:
   - Only enable in trusted environments
   - Storing credentials requires the master password

3. **CI/CD**:
   - Use secrets management for `THALORA_MASTER_PASSWORD`
   - Keep `THALORA_ENABLE_VAULT` disabled by default, or point `THALORA_VAULT_PATH` at a temp file
   - Use ephemeral test passwords in CI

---

## Troubleshooting

### "Master password required" error

```
Error: Master password required. Set THALORA_MASTER_PASSWORD environment variable (min 32 chars)
```

**Solution**: Set the environment variable before running:
```bash
export THALORA_MASTER_PASSWORD="your-secure-password-at-least-32-characters-long"
```

### Credentials not persisting

The vault is in-memory unless `THALORA_ENABLE_VAULT=1` is set.

### "Failed to open credential vault"

Thalora falls back to an in-memory vault. Check that the vault file
(`~/.thalora/credentials.json` or `THALORA_VAULT_PATH`) is valid JSON.

---

## See Also

- [SECURITY.md](SECURITY.md) - Security architecture and best practices
- [SECURITY_REMEDIATION_SUMMARY.md](archive/SECURITY_REMEDIATION_SUMMARY.md) - Security fixes and verification
