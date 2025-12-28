-- Insert audit remediation items into database
-- This script creates 18 items across 4 priority waves

-- Wave 1: Immediate
INSERT INTO items (iteration_id, title, priority, notes) VALUES
  (3, 'Fix OPENROUTER-001: Mutually exclusive TLS features prevent default compilation', 'critical', 'Default feature enables both rustls and native-tls causing compile error. Need to fix Cargo.toml feature configuration.');

-- Wave 2: Short-term
INSERT INTO items (iteration_id, title, priority, notes) VALUES
  (3, 'Fix OPENROUTER-002: Potential panic on schema object unwrap in structured.rs', 'high', 'Replace .unwrap() with proper error handling after type check'),
  (3, 'Fix OPENROUTER-003: Ignored Result from response bytes consumption in retry.rs', 'high', 'Add error handling for consumed response bytes or log errors');

-- Wave 3: Medium-term (Clippy warnings)
INSERT INTO items (iteration_id, title, priority, notes) VALUES
  (3, 'Fix OPENROUTER-007: Clippy warnings for bool assert comparisons', 'medium', '7 instances in src/types/status/mod.rs and tests/message_consolidation.rs'),
  (3, 'Fix OPENROUTER-008: Clippy warnings for unused must-use values', 'medium', '4 instances in tests/id_newtypes.rs - format!() results ignored'),
  (3, 'Fix OPENROUTER-009: Unused import in tests/message_consolidation.rs', 'low', 'Remove unused import of openrouter_api::types::common');

-- Wave 4: Backlog items
INSERT INTO items (iteration_id, title, priority, notes) VALUES
  (3, 'Monitor OPENROUTER-004: Unmaintained dependency instant', 'low', 'Transitive dependency via wiremock. Monitor for updates.'),
  (3, 'Monitor OPENROUTER-005: Unmaintained dependency rustls-pemfile', 'low', 'Transitive dependency via reqwest. Monitor for updates.'),
  (3, 'Evaluate OPENROUTER-006: Excessive header cloning in hot paths', 'low', 'Performance concern - headers cloned on every request. Evaluate impact.'),
  (3, 'Refactor OPENROUTER-010: Large functions in API modules', 'low', 'Extract URL building and query parameter construction into helpers'),
  (3, 'Refactor OPENROUTER-011: Manual loops to iterators', 'low', 'Opportunistic refactor for idiomatic Rust'),
  (3, 'Review OPENROUTER-012: Redundant string clones in to_api_config', 'low', 'Evaluate if clones are necessary or can be documented'),
  (3, 'Remove OPENROUTER-013: Unused import in test file', 'low', 'Same as OPENROUTER-009 - consolidate'),
  (3, 'Review OPENROUTER-014: Cache get method mutates data', 'low', 'Consider if mutation during get is acceptable or needs redesign'),
  (3, 'Fix OPENROUTER-015: Test format calls not using results', 'low', '4 instances in tests/id_newtypes.rs'),
  (3, 'Split OPENROUTER-016: Large source files', 'low', 'analytics.rs (440 lines) and providers.rs (495 lines) - consider splitting'),
  (3, 'Review OPENROUTER-017: Cache size limits', 'low', 'Consider adding max size and eviction strategy'),
  (3, 'Evaluate OPENROUTER-018: JSON-RPC ID validation', 'low', 'MCP client should validate response ID matches request ID');
