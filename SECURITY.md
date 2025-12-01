## Security Policy

This document describes the security features and configuration options for the Thalora web browser.

## Security Model

Thalora is designed for AI model integration and automation. It implements defense-in-depth security with the following layers:

### 1. **Critical**: Encrypted Credential Storage (Priority 1 - FIXED)

#### Issue (Before Fix)
- Used XOR cipher with hardcoded key
- All stored passwords could be decrypted in <1ms
- No authentication or tampering detection

#### Fix (Current)
- **AES-256-GCM** authenticated encryption
- **Argon2id** key derivation (resistant to GPU attacks)
- Random salt per password (prevents rainbow tables)
- Random nonce per encryption (prevents pattern analysis)
- 128-bit authentication tag (detects tampering)

#### Configuration

**Required**: Set a strong master password (minimum 32 characters):

```bash
export THALORA_MASTER_PASSWORD="your-strong-password-minimum-32-characters-long!"
```

**Security Requirements**:
- Master password MUST be at least 32 characters
- Password is required for all credential encryption/decryption
- Credentials encrypted with old XOR format will NOT decrypt (must be re-entered)

#### Migration from Legacy Format

If you have credentials encrypted with the old XOR format:

1. Delete old credential cache:
   ```bash
   rm -rf ~/.cache/thalora/ai_memory/
   ```

2. Set new master password:
   ```bash
   export THALORA_MASTER_PASSWORD="your-new-strong-password-32chars!"
   ```

3. Re-enter all credentials

**Data Location**: `~/.cache/thalora/ai_memory/`

---

### 2. **High**: JavaScript Execution Security (Priority 2 - FIXED)

#### Issue (Before Fix)
- Weak pattern matching (easily bypassed)
- eval(), Function() constructor not blocked
- No AST-based validation
- 10MB code size limit (too large)

#### Fix (Current)
- **AST-based JavaScript validation** using SWC parser
- **Hard blocks** (no feature flags, no overrides):
  - `eval()` calls - BLOCKED
  - `Function()` constructor - BLOCKED
  - `setTimeout`/`setInterval` with strings - BLOCKED
  - `__proto__` access - BLOCKED (prototype pollution)
  - `constructor.constructor` - BLOCKED
  - `with` statements - BLOCKED
  - `import` statements - BLOCKED
  - `document.write` - BLOCKED
  - WebAssembly instantiation - BLOCKED

#### Validation Examples

**Blocked Code**:
```javascript
eval('alert(1)')                    // ❌ BLOCKED
Function('return 1')()              // ❌ BLOCKED
setTimeout('alert(1)', 1000)        // ❌ BLOCKED
obj.__proto__ = {}                  // ❌ BLOCKED
obj.constructor.constructor()       // ❌ BLOCKED
with (obj) { x = 1; }              // ❌ BLOCKED
import { foo } from 'bar'           // ❌ BLOCKED
```

**Allowed Code**:
```javascript
const x = 1 + 2;                    // ✅ ALLOWED
function add(a, b) { return a + b; } // ✅ ALLOWED
setTimeout(() => console.log('ok'), 1000) // ✅ ALLOWED
const arr = [1, 2, 3].map(x => x * 2); // ✅ ALLOWED
```

**No Configuration Options**: JavaScript security is enforced by default with no way to disable it. This is intentional to prevent security bypass.

---

### 3. **High**: Origin-Based Isolation (Priority 3 - FIXED)

#### Issue (Before Fix)
- No per-origin storage isolation
- Site A could access Site B's localStorage
- Credentials accessible cross-origin

#### Fix (Current)
- **Same-Origin Policy** enforced
- localStorage/sessionStorage isolated per origin
- Credentials stored per-origin
- Origin validation for all storage access

#### Origin Format

```
scheme://host:port
```

Examples:
- `https://example.com` (port 443 implied)
- `https://example.com:8080` (explicit port)
- `http://localhost:3000`

**Cross-Origin Access**: BLOCKED

```javascript
// Page: https://example.com
localStorage.setItem('token', 'secret'); // ✅ Stored for example.com

// Page: https://attacker.com
localStorage.getItem('token'); // ❌ BLOCKED - Cannot access example.com's storage
```

---

### 4. **Medium**: SSRF Prevention (Priority 4 - FIXED)

#### Issue (Before Fix)
- Could access internal networks (localhost, 192.168.x.x, etc.)
- No IP filtering
- DNS rebinding possible

#### Fix (Current)
- **IP Range Blocking** (hard-coded, cannot be overridden)
- **DNS Resolution** before request
- **Scheme Whitelist** (http, https only)

#### Blocked IP Ranges

**IPv4**:
- `127.0.0.0/8` - Localhost
- `10.0.0.0/8` - Private
- `172.16.0.0/12` - Private
- `192.168.0.0/16` - Private
- `169.254.0.0/16` - Link-local (AWS metadata)
- `224.0.0.0/4` - Multicast

**IPv6**:
- `::1/128` - Localhost
- `fc00::/7` - Unique local addresses
- `fe80::/10` - Link-local

#### Examples

**Blocked Requests**:
```bash
http://localhost              # ❌ BLOCKED
http://127.0.0.1             # ❌ BLOCKED
http://192.168.1.1           # ❌ BLOCKED
http://10.0.0.1              # ❌ BLOCKED
http://169.254.169.254       # ❌ BLOCKED (AWS metadata)
file:///etc/passwd           # ❌ BLOCKED (scheme not allowed)
ftp://example.com            # ❌ BLOCKED (scheme not allowed)
```

**Allowed Requests**:
```bash
https://example.com          # ✅ ALLOWED (public IP)
http://api.github.com        # ✅ ALLOWED (public IP)
```

**No Configuration Options**: SSRF protection is always enabled and cannot be disabled.

---

### 5. **Low**: Additional Hardening (Priority 5 - IMPLEMENTED)

#### WebRTC Security
- **Default**: Disabled
- **Reason**: Prevents IP address leaks
- **Configuration**: None (hard-disabled)

#### Service Worker Security
- Scope restrictions enforced
- Cannot intercept requests outside registered scope
- Scripts validated before registration

#### Path Security
- Socket paths include random UUIDs (not predictable)
- Proper file permissions (0600)
- Temp directories created securely

#### Security Headers
The following headers are set automatically:
- `Strict-Transport-Security: max-age=31536000`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: SAMEORIGIN`
- `Referrer-Policy: strict-origin-when-cross-origin`

#### Mixed Content
- HTTPS pages cannot load HTTP resources (hard block)

---

## Security Configuration Summary

### Required Environment Variables

```bash
# REQUIRED: Master password for credential encryption
export THALORA_MASTER_PASSWORD="your-strong-password-32-chars-minimum!"
```

### Optional Environment Variables

**None** - All security features are hard-coded and cannot be disabled.

### Logging

Security events are logged to stderr:

```bash
# Enable security debug logging
export RUST_LOG=thalora=debug
```

Example logs:
```
🔒 SECURITY: JavaScript validation failed: eval() is not allowed
🔒 SECURITY: Access to 127.0.0.1 is blocked (private IP range)
```

---

## Threat Model

### Protected Against

✅ **Local Attackers**
- Cannot decrypt stored credentials (AES-256-GCM)
- Cannot execute arbitrary code (JavaScript validation)

✅ **Network Attackers**
- Cannot intercept HTTPS traffic (TLS enforced)
- Cannot perform SSRF attacks (IP filtering)
- Cannot bypass CORS (enforced)

✅ **Malicious Websites**
- Cannot steal cross-origin data (Same-Origin Policy)
- Cannot execute dangerous JavaScript (AST validation)
- Cannot access internal networks (SSRF prevention)

✅ **Prototype Pollution**
- `__proto__` access blocked
- `constructor.constructor` blocked

✅ **XSS Attacks**
- `eval()` blocked
- `document.write` blocked
- CSP enforced

### NOT Protected Against

❌ **Compromised Master Password**
- If attacker obtains `THALORA_MASTER_PASSWORD`, credentials can be decrypted
- **Mitigation**: Use strong password (32+ chars), store securely

❌ **Physical Access**
- If attacker has root access to machine, they can extract master password from environment
- **Mitigation**: Use OS-level security, encrypt home directory

❌ **Malicious AI Models**
- If AI model itself is compromised, it can abuse browser capabilities
- **Mitigation**: Only use trusted AI models

---

## Reporting Security Vulnerabilities

If you discover a security vulnerability in Thalora, please report it to:

**Email**: [security@brainwires.com](mailto:security@brainwires.com)

**What to include**:
1. Description of the vulnerability
2. Steps to reproduce
3. Potential impact
4. Suggested fix (if any)

**Response Time**: We aim to respond within 48 hours.

**Disclosure Policy**: We follow responsible disclosure. Please allow us 90 days to fix the issue before public disclosure.

---

## Security Audit History

| Date       | Auditor           | Scope                | Findings               | Status   |
|------------|-------------------|----------------------|------------------------|----------|
| 2025-11-07 | Internal Security | Full codebase        | 10 Critical/High issues| **FIXED**|

---

## Security Best Practices

### For Users

1. **Set a strong master password** (32+ characters, random)
   ```bash
   # Good
   export THALORA_MASTER_PASSWORD="$(openssl rand -base64 32)"

   # Bad
   export THALORA_MASTER_PASSWORD="password123"  # Too weak!
   ```

2. **Protect your environment variables**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export THALORA_MASTER_PASSWORD="your-strong-password"

   # Set proper permissions
   chmod 600 ~/.bashrc
   ```

3. **Use HTTPS** whenever possible

4. **Regularly rotate master password** (every 90 days recommended)

5. **Enable debug logging** to monitor security events
   ```bash
   export RUST_LOG=thalora=debug
   ```

### For Developers

1. **Never disable security features** in production
2. **Always validate user input**
3. **Use secure defaults** (all security features enabled)
4. **Log security events** for audit trail
5. **Keep dependencies updated**
6. **Run security tests** before each release:
   ```bash
   cargo test --test security_tests
   ```

---

## Compliance

Thalora implements security controls aligned with:

- **OWASP Top 10** (2021)
  - A01: Broken Access Control ✅ FIXED
  - A02: Cryptographic Failures ✅ FIXED
  - A03: Injection ✅ FIXED (JavaScript validation)
  - A07: Identification and Authentication Failures ✅ FIXED
  - A10: Server-Side Request Forgery ✅ FIXED

- **CWE Top 25** Most Dangerous Software Weaknesses
  - CWE-79: Cross-site Scripting ✅ PROTECTED
  - CWE-89: SQL Injection ✅ N/A (no SQL)
  - CWE-798: Use of Hard-coded Credentials ✅ FIXED
  - CWE-918: Server-Side Request Forgery ✅ FIXED

---

## Version History

### v0.2.0 (2025-11-07) - Security Hardening Release

**Breaking Changes**:
- Master password (32+ chars) now required
- Legacy XOR-encrypted credentials will not decrypt
- Dangerous JavaScript operations now blocked (no bypass)

**Security Fixes**:
- [CRITICAL] Replaced XOR with AES-256-GCM encryption
- [HIGH] Implemented AST-based JavaScript validation
- [HIGH] Added origin-based storage isolation
- [MEDIUM] Implemented SSRF prevention
- [LOW] Disabled WebRTC by default
- [LOW] Randomized socket paths

**New Security Features**:
- Argon2id key derivation
- Authenticated encryption with AES-256-GCM
- Same-Origin Policy enforcement
- IP range filtering
- CSP enforcement
- CORS validation
- Security headers

**Migration Guide**: See "Migration from Legacy Format" above.

---

## Additional Resources

- [OWASP Cheat Sheets](https://cheatsheetseries.owasp.org/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Web Security Best Practices](https://developer.mozilla.org/en-US/docs/Web/Security)

---

## License

This security policy is part of the Thalora project and is licensed under the MIT License.
