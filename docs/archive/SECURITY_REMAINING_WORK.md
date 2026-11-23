# Security Remediation - Remaining Work

**Date**: December 2024
**Status**: 4 of 7 critical/high vulnerabilities fixed

## Summary of Completed Work

| Issue | Severity | Status |
|-------|----------|--------|
| Path Traversal via session_id | CRITICAL | FIXED |
| SSRF Protection Not Enforced | CRITICAL | FIXED |
| Cookie Injection (CRLF/Null) | HIGH | FIXED |
| Missing URL Validation | HIGH | FIXED |

## Remaining Vulnerabilities

---

### 1. JavaScript Execution Timeout Not Enforced
**Severity**: HIGH
**CWE**: CWE-400 (Uncontrolled Resource Consumption)
**Priority**: 1 (Should be fixed next)

**Current State**:
The `evaluate_javascript_with_timeout` function in `src/engine/renderer/execution.rs` accepts a timeout parameter but **ignores it** (note the underscore prefix `_timeout_duration`):

```rust
// src/engine/renderer/execution.rs:118-191
fn evaluate_javascript_with_timeout(&mut self, js_code: &str, _timeout_duration: Duration) -> Result<String> {
    // _timeout_duration is NEVER USED
    // JavaScript runs indefinitely until completion
}
```

**Attack Vector**:
Malicious JavaScript can cause denial of service:
```javascript
while(true) {} // Infinite loop - blocks server forever
```

**Fix Required**:
Option A: Thread-based timeout (recommended for Boa engine)
```rust
use std::thread;
use std::sync::mpsc;

fn evaluate_javascript_with_timeout(&mut self, js_code: &str, timeout: Duration) -> Result<String> {
    let (sender, receiver) = mpsc::channel();
    let code = js_code.to_string();

    // Clone necessary context for the thread
    let context = self.context.clone(); // Need to make context cloneable or use Arc<Mutex<>>

    thread::spawn(move || {
        let result = /* execute JS */;
        let _ = sender.send(result);
    });

    match receiver.recv_timeout(timeout) {
        Ok(result) => result,
        Err(_) => Err(anyhow!("JavaScript execution timeout after {:?}", timeout))
    }
}
```

Option B: Boa engine instruction limit (simpler)
```rust
// In engines/boa/core/engine/src/vm/runtime_limits.rs
impl RuntimeLimits {
    pub fn with_loop_iteration_limit(mut self, limit: u64) -> Self {
        self.loop_iteration = limit;
        self
    }
}

// Usage in renderer
context.set_runtime_limits(RuntimeLimits::default().with_loop_iteration_limit(1_000_000));
```

**Estimated Effort**: Medium - Requires architectural changes to JS execution
**Files to Modify**:
- `src/engine/renderer/execution.rs`
- `src/engine/renderer/core.rs`
- Possibly `engines/boa/core/engine/src/vm/runtime_limits.rs`

---

### 2. Cryptographic Keys Not Zeroed from Memory
**Severity**: MEDIUM
**CWE**: CWE-316 (Cleartext Storage of Sensitive Information in Memory)
**Priority**: 2

**Current State**:
In `src/features/ai_memory/crypto.rs`, encryption keys remain in memory after use:

```rust
// src/features/ai_memory/crypto.rs:75-79
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_SIZE]> {
    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(hash_bytes);
    Ok(key)  // Key stays in memory, could be dumped/swapped to disk
}
```

**Attack Vector**:
- Memory dump attacks can extract encryption keys
- Keys may be swapped to disk in plaintext
- Core dumps may contain sensitive key material

**Fix Required**:
Add the `zeroize` crate and use `Zeroizing` wrapper:

```toml
# Cargo.toml
[dependencies]
zeroize = { version = "1.7", features = ["derive"] }
```

```rust
// src/features/ai_memory/crypto.rs
use zeroize::{Zeroize, Zeroizing};

fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_SIZE]>> {
    let mut key = Zeroizing::new([0u8; KEY_SIZE]);
    // ... derive key ...
    Ok(key)  // Automatically zeroed when dropped
}

// Also update encrypt/decrypt functions to use Zeroizing
fn encrypt_data(key: &Zeroizing<[u8; KEY_SIZE]>, data: &[u8]) -> Result<Vec<u8>> {
    // key is automatically zeroed when this function returns
}
```

**Estimated Effort**: Low - Add dependency and update type signatures
**Files to Modify**:
- `Cargo.toml` (add zeroize dependency)
- `src/features/ai_memory/crypto.rs`

---

### 3. Input Length Limits Not Enforced on MCP Parameters
**Severity**: HIGH
**CWE**: CWE-400 (Uncontrolled Resource Consumption)
**Priority**: 3

**Current State**:
MCP tool parameters accept arbitrarily large strings without validation:

```rust
// Example: src/protocols/mcp_server/tools/mod.rs
let url = arguments["url"].as_str();  // No length limit!
let query = arguments["query"].as_str();  // No length limit!
let code = arguments["code"].as_str();  // No length limit!
```

**Attack Vector**:
- Send 1GB URL string to exhaust memory
- Send massive JavaScript code to overwhelm parser
- Send huge search queries to cause DoS

**Fix Required**:
Use the `limit_input_length` function from the security module:

```rust
use crate::protocols::security::limit_input_length;

// Define constants for limits
const MAX_URL_LENGTH: usize = 8192;  // 8KB
const MAX_QUERY_LENGTH: usize = 1024;  // 1KB
const MAX_JS_CODE_LENGTH: usize = 1024 * 1024;  // 1MB
const MAX_SESSION_ID_LENGTH: usize = 64;
const MAX_SELECTOR_LENGTH: usize = 1024;

// Apply validation
let url = match arguments["url"].as_str() {
    Some(u) => limit_input_length(u, MAX_URL_LENGTH, "URL")?,
    None => return McpResponse::error(-1, "URL is required"),
};
```

**Locations Requiring Updates**:
1. `src/protocols/mcp_server/scraping/core.rs` - URL, selectors
2. `src/protocols/mcp_server/tools/mod.rs` - All tool parameters
3. `src/protocols/browser_tools/handlers/*.rs` - All handler parameters
4. `src/protocols/cdp_tools/*.rs` - CDP command parameters
5. `src/protocols/mcp_server/scraping/search/*.rs` - Search queries

**Estimated Effort**: Medium - Many files to update but straightforward changes
**Files to Modify**: ~15 files across protocols directory

---

### 4. Integer Overflow in Chunk Calculations
**Severity**: HIGH
**CWE**: CWE-190 (Integer Overflow)
**Priority**: 4

**Current State**:
Various locations perform arithmetic on sizes without overflow checking:

```rust
// Hypothetical vulnerable code
let total_size = chunk_count * chunk_size;  // Could overflow!
let buffer_size = header_size + data_size;  // Could overflow!
```

**Fix Required**:
Use checked arithmetic:

```rust
let total_size = chunk_count
    .checked_mul(chunk_size)
    .ok_or_else(|| anyhow!("Integer overflow calculating total size"))?;

let buffer_size = header_size
    .checked_add(data_size)
    .ok_or_else(|| anyhow!("Integer overflow calculating buffer size"))?;
```

**Estimated Effort**: Low-Medium - Search for arithmetic operations on sizes
**Files to Audit**:
- `src/features/ai_memory/*.rs`
- `src/engine/browser/*.rs`
- `vfs/src/*.rs`

---

### 5. No Rate Limiting on Resource-Intensive Operations
**Severity**: MEDIUM
**CWE**: CWE-770 (Allocation of Resources Without Limits)
**Priority**: 5

**Current State**:
MCP tools can be called unlimited times without throttling:
- Unlimited concurrent browser sessions
- Unlimited search queries per second
- Unlimited navigation requests

**Fix Required**:
Add rate limiting middleware:

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct RateLimiter {
    limits: HashMap<String, (usize, Duration)>,  // tool -> (max_requests, window)
    state: HashMap<String, Vec<Instant>>,  // tool -> request timestamps
}

impl RateLimiter {
    pub fn check(&mut self, tool_name: &str) -> Result<()> {
        // Remove old entries outside window
        // Check if under limit
        // Add new timestamp
    }
}
```

**Estimated Effort**: Medium - New infrastructure required
**Files to Modify**:
- New file: `src/protocols/rate_limiter.rs`
- `src/protocols/mcp_server/tools/mod.rs` - Add rate limit checks

---

### 6. Session Data Not Encrypted at Rest
**Severity**: MEDIUM
**CWE**: CWE-312 (Cleartext Storage of Sensitive Information)
**Priority**: 6

**Current State**:
VFS session backing files are stored unencrypted:

```rust
// src/protocols/mcp_server/core.rs
let file = dir.join(format!("vfs-session-{}.bin", session_id));
// File contents are not encrypted
```

**Fix Required**:
Use the existing crypto module to encrypt session data:

```rust
use crate::features::ai_memory::crypto::{encrypt_data, decrypt_data};

// When persisting
let encrypted = encrypt_data(&key, &session_data)?;
fs::write(&file, encrypted)?;

// When loading
let encrypted = fs::read(&file)?;
let decrypted = decrypt_data(&key, &encrypted)?;
```

**Estimated Effort**: Medium - Need to integrate crypto with VFS
**Files to Modify**:
- `src/protocols/mcp_server/core.rs`
- `vfs/src/lib.rs`

---

## Recommended Fix Order

1. **JS Execution Timeout** (HIGH) - Prevents DoS via infinite loops
2. **Key Zeroing** (MEDIUM) - Simple fix, improves crypto security
3. **Input Length Limits** (HIGH) - Prevents memory exhaustion
4. **Integer Overflow** (HIGH) - Prevents undefined behavior
5. **Rate Limiting** (MEDIUM) - Prevents abuse
6. **Session Encryption** (MEDIUM) - Protects data at rest

---

## Testing Checklist

After each fix, verify:

- [ ] `cargo test` passes
- [ ] `cargo test --lib protocols::security` passes
- [ ] Manual testing of attack vectors fails
- [ ] Legitimate use cases still work
- [ ] No performance regression

---

## Security Module Extension

The `src/protocols/security.rs` module can be extended with:

```rust
// Already implemented
pub fn sanitize_session_id(session_id: &str) -> Result<String>
pub fn validate_url_for_navigation(url: &str) -> Result<()>
pub fn validate_cookie(name: &str, value: &str) -> Result<()>
pub fn limit_input_length<'a>(input: &'a str, max: usize, field: &str) -> Result<&'a str>

// To be added
pub fn validate_js_code_safe(code: &str) -> Result<()>  // Basic JS validation
pub fn sanitize_file_path(path: &str) -> Result<PathBuf>  // Path validation
pub fn validate_integer_range<T: TryInto<usize>>(val: T, min: usize, max: usize) -> Result<usize>
```
