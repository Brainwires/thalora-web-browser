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

### `THALORA_ENABLE_AI_MEMORY`

**Required for**: Persistent AI memory storage
**Type**: Boolean (any value = enabled)
**Default**: Disabled (not set)

Controls whether the AI Memory Heap persists data to disk. When disabled, Thalora uses in-memory storage only.

**Example**:
```bash
# Enable AI memory persistence
export THALORA_ENABLE_AI_MEMORY=1

# Or any non-empty value
export THALORA_ENABLE_AI_MEMORY=true
export THALORA_ENABLE_AI_MEMORY=yes
```

**Behavior**:
- **Disabled** (default): Creates temporary in-memory storage, no disk writes
- **Enabled**: Persists research, credentials, bookmarks, and sessions to `~/.cache/thalora/ai_memory/`

**Use Cases**:
- **Production/CI**: Keep disabled to avoid persistent state
- **Development**: Enable for testing memory features
- **AI Agents**: Enable to maintain context across sessions

---

## Engine Configuration

### `THALORA_ENGINE`

**Required for**: JavaScript engine selection
**Type**: String (`boa` or `v8`)
**Default**: `boa`

Selects which JavaScript engine to use for code execution.

**Example**:
```bash
# Use Boa engine (default)
export THALORA_ENGINE=boa

# Use V8 engine (if available)
export THALORA_ENGINE=v8
```

**Notes**:
- Boa is the default and recommended engine
- V8 integration is experimental and may not be available in all builds

---

## Testing

### Test-specific variables used during test runs:

**Cargo Test Environment**:
```bash
# Required for running security tests
export THALORA_MASTER_PASSWORD="test_master_password_min_32chars_secure"

# Run tests with AI memory enabled
export THALORA_ENABLE_AI_MEMORY=1
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
export THALORA_ENABLE_AI_MEMORY=1

# Engine selection (OPTIONAL - defaults to boa)
export THALORA_ENGINE=boa

# Run Thalora
cargo run --release
```

---

## Persistence Locations

When features are enabled, Thalora uses these directories:

### Linux/macOS:
- AI Memory: `~/.cache/thalora/ai_memory/`
- Credentials: `~/.cache/thalora/ai_memory/heap.json` (encrypted)

### Windows:
- AI Memory: `%LOCALAPPDATA%\thalora\ai_memory\`
- Credentials: `%LOCALAPPDATA%\thalora\ai_memory\heap.json` (encrypted)

---

## Security Best Practices

1. **Master Password**:
   - Generate strong random passwords: `openssl rand -base64 48`
   - Store in environment variables, not in code
   - Use different passwords for dev/staging/production

2. **AI Memory**:
   - Only enable in trusted environments
   - Encrypted credentials require master password
   - Clear cache when changing environments: `rm -rf ~/.cache/thalora/`

3. **CI/CD**:
   - Use secrets management for `THALORA_MASTER_PASSWORD`
   - Keep `THALORA_ENABLE_AI_MEMORY` disabled by default
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

### AI Memory not persisting

**Check**: Is `THALORA_ENABLE_AI_MEMORY` set?
```bash
echo $THALORA_ENABLE_AI_MEMORY
# Should output: 1 (or any non-empty value)
```

**Solution**: Enable it:
```bash
export THALORA_ENABLE_AI_MEMORY=1
```

### "Failed to load AI memory heap"

This is a warning, not an error. Thalora will create a new memory heap automatically.

To clear corrupted memory:
```bash
rm -rf ~/.cache/thalora/ai_memory/
```

---

## See Also

- [SECURITY.md](SECURITY.md) - Security architecture and best practices
- [SECURITY_REMEDIATION_SUMMARY.md](SECURITY_REMEDIATION_SUMMARY.md) - Security fixes and verification
- [FEATURES.md](FEATURES.md) - Complete feature list including AI Memory
