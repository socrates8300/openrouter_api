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
