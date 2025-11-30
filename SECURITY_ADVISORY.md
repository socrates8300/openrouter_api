# Security Advisory: v0.4.2 - Critical Security Fixes

**Advisory ID**: OPENROUTOR-API-2025-003
**Release Date**: November 30, 2025
**Version**: 0.4.2
**Severity**: High
**Status**: Resolved

## Summary

This security advisory addresses a high-severity vulnerability identified in the OpenRouter API Rust client library:

- **SECURITY-02**: Unbounded Response Body Read in Main Client (HIGH)

## Details

### [SECURITY-02] Unbounded Response Body Read (Client)

The main API client (`OpenRouterClient`) previously relied on `response.text().await` which reads the entire body into memory without a size limit.

A malicious server could exploit this by sending a massive body, causing the client to allocate excessive memory and potentially crash (OOM).

**Resolution**: The client now implements a `max_response_bytes` configuration (default 10MB). It checks the `Content-Length` header and enforces the limit on the actual bytes read using a bounded reader.

## Affected Versions

- **Vulnerable**: v0.4.1 and earlier
- **Patched**: v0.4.2+
- **Safe**: v0.4.2 (this release and later)

## Recommendations

### Immediate Actions
1. **Update Dependencies**: Upgrade to v0.4.2 or later immediately.
2. **Review Configuration**: Ensure `max_response_bytes` in `ClientConfig` is set to an appropriate limit for your environment (default is 10MB).

---

# Security Advisory: v0.4.1 - Critical Security Fixes

**Advisory ID**: OPENROUTOR-API-2025-002
**Release Date**: November 30, 2025
**Version**: 0.4.1
**Severity**: High
**Status**: Resolved

## Summary

This security advisory addresses a high-severity vulnerability identified in the OpenRouter API Rust client library:

- **SECURITY-01**: Unbounded Response Body Read in MCP Client (HIGH)

## Details

### [SECURITY-01] Unbounded Response Body Read

The MCP client previously relied on the `Content-Length` header to enforce the `max_response_size` configuration. However, it subsequently called `response.text().await`, which reads the entire body into memory regardless of the configuration if the `Content-Length` header is missing or misleading (e.g., smaller than the actual body).

A malicious server could exploit this by sending a massive body with a small or missing `Content-Length` header, causing the client to allocate excessive memory and potentially crash (OOM).

**Resolution**: The client now uses a bounded stream reader (`response.bytes_stream()`) to read the response body. It enforces the `max_response_size` limit on the actual bytes read, ensuring that the client never allocates more memory than configured, regardless of the headers provided by the server.

## Affected Versions

- **Vulnerable**: v0.4.0 and earlier
- **Patched**: v0.4.1+
- **Safe**: v0.4.1 (this release and later)

## Recommendations

### Immediate Actions
1. **Update Dependencies**: Upgrade to v0.4.1 or later immediately.
2. **Review Configuration**: Ensure `max_response_size` in `McpConfig` is set to an appropriate limit for your environment (default is 10MB).

---

# Security Advisory: v0.3.2 - Critical Security Fixes

**Advisory ID**: OPENROUTOR-API-2025-001  
**Release Date**: October 18, 2025  
**Version**: 0.3.2  
**Severity**: Critical  
**Status**: Resolved  

## Summary

This security advisory addresses three critical vulnerabilities identified in the OpenRouter API Rust client library:

- **SEC-001**: SecureApiKey Clone Implementation (CRITICAL)
- **SEC-002**: MCP Client Missing Timeouts and Size Limits (HIGH)  
- **SEC-003**: Retry Logic Missing Total Time Caps (HIGH)

All issues have been completely resolved with comprehensive security improvements and thorough testing.

## Affected Versions

- **Vulnerable**: v0.3.0 and earlier
- **Patched**: v0.3.2+
- **Safe**: v0.3.2 (this release and later)

## Breaking Changes

### Required Actions

**For SecureApiKey Users:**
```rust
// Before (compiles):
let key1 = SecureApiKey::new("sk-secret")?;
let key2 = key1.clone(); // ← Vulnerable

// After (compile error):
let key1 = SecureApiKey::new("sk-secret")?;
let key2 = SecureApiKey::new("sk-secret")?; // ← Secure alternative
// Or pass references:
some_function(&key1);
```

**For MCP Client Users:**
```rust
// Before:
let client = MCPClient::new("https://mcp-server.com")?;

// After (still works with defaults):
let client = MCPClient::new("https://mcp-server.com")?;

// Or with custom config:
let config = McpConfig {
    request_timeout: Duration::from_secs(30),
    max_response_size: 5 * 1024 * 1024, // 5MB
    max_request_size: 1024 * 1024,        // 1MB
    max_concurrent_requests: 5,
};
let client = MCPClient::new_with_config("https://mcp-server.com", config)?;
```

**For RetryConfig Users:**
```rust
// Before:
let config = RetryConfig::default();

// After (still works with enhanced defaults):
let config = RetryConfig::default();

// Or with custom time caps:
let config = RetryConfig::default()
    .with_total_timeout(Duration::from_secs(60))
    .with_max_retry_interval(Duration::from_secs(10));
```

## Testing

### Security Test Results
- ✅ **262 tests passing** (including 15 new security tests)
- ✅ **0 failures** across all security scenarios
- ✅ **Memory safety** verified through automated testing
- ✅ **Timeout enforcement** validated in edge cases
- ✅ **Size limits** tested with mock malicious servers

## Recommendations

### Immediate Actions
1. **Update Dependencies**: Upgrade to v0.3.2 or later immediately
2. **Review Code**: Update any code that clones SecureApiKey
3. **Update CI/CD**: Ensure all pipelines use the patched version
4. **Monitor**: Set up monitoring for timeout and size limit violations

### Best Practices
1. **Never Clone API Keys**: Use references and secure storage
2. **Configure Timeouts**: Set appropriate timeouts for your environment
3. **Monitor Usage**: Track retry rates and timeout patterns
4. **Regular Updates**: Keep dependencies current with security patches

## Contact

For questions or concerns regarding this security advisory:
- **Security Team**: security@openrouter.ai
- **Project Maintainer**: James Ray <openrouter.aea1p@passmail.net>
- **GitHub Issues**: [Create an issue](https://github.com/socrates8300/openrouter_api/issues)

---

*This security advisory is provided for informational purposes. All users should upgrade to the latest version to ensure protection against the identified vulnerabilities.*
