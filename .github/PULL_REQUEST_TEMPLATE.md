# Pull Request

## Description
<!-- Provide a clear and concise description of what this PR does -->

## Type of Change
<!-- Check all that apply -->
- [ ] 🐛 Bug fix (non-breaking change that fixes an issue)
- [ ] ✨ New feature (non-breaking change that adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📚 Documentation update
- [ ] 🧹 Code cleanup/refactoring
- [ ] ⚡ Performance improvement
- [ ] 🔒 Security fix
- [ ] 🧪 Test improvement

## Related Issues
<!-- Link to any related issues -->
Fixes #(issue number)
Relates to #(issue number)

## Changes Made
<!-- List the specific changes made in this PR -->
- 
- 
- 

## Quality Checklist
<!-- Ensure all items are checked before requesting review -->

### Code Quality
- [ ] Code follows the project's style guidelines
- [ ] Self-review of code has been performed
- [ ] Code is self-documenting with clear variable/function names
- [ ] Complex logic includes explanatory comments

### Testing
- [ ] All quality gates pass (`./scripts/pre_quality.sh`)
- [ ] New tests added for new functionality
- [ ] All existing tests pass
- [ ] Test coverage is maintained or improved
- [ ] Edge cases and error conditions are tested

### Documentation
- [ ] Documentation updated for public API changes
- [ ] Rust doc comments added/updated for new public functions
- [ ] README updated if needed
- [ ] Examples updated if applicable

### Security
- [ ] No sensitive data exposed in logs or error messages
- [ ] Input validation added where appropriate
- [ ] Error messages use `create_safe_error_message` utility
- [ ] No hardcoded secrets or API keys

## API Impact
<!-- For changes affecting the public API -->

### Breaking Changes
<!-- If this introduces breaking changes, explain what breaks and how to migrate -->
- [ ] No breaking changes
- [ ] Breaking changes documented with migration guide

### New Public APIs
<!-- List any new public functions, types, or modules -->
```rust
// Example:
pub fn new_function() -> Result<()> {
    // ...
}
```

## Testing Strategy
<!-- Describe how you tested your changes -->

### Manual Testing
<!-- Steps you took to manually test the changes -->
1. 
2. 
3. 

### Automated Testing
<!-- New tests added or existing tests that validate the changes -->
- [ ] Unit tests: `cargo test test_name`
- [ ] Integration tests: `cargo test --test integration_tests`
- [ ] Documentation tests: `cargo test --doc`

## Performance Impact
<!-- If applicable, describe performance implications -->
- [ ] No performance impact
- [ ] Performance improved
- [ ] Performance impact acceptable for the feature
- [ ] Performance benchmarks included

## Security Considerations
<!-- Describe any security implications of the changes -->
- [ ] No security implications
- [ ] Security review completed
- [ ] Follows security best practices
- [ ] Potential security concerns documented

## Dependencies
<!-- List any new dependencies or dependency changes -->
- [ ] No new dependencies
- [ ] New dependencies justified and minimal
- [ ] Dependencies updated with security patches

## Deployment Notes
<!-- Any special considerations for deployment -->
- [ ] No special deployment considerations
- [ ] Migration steps documented
- [ ] Environment variables or configuration changes needed

## Additional Context
<!-- Add any other context, screenshots, or information about the PR -->

## Reviewer Notes
<!-- Anything specific you want reviewers to focus on -->
- Please pay special attention to:
- Areas where feedback is particularly welcome:
- Known limitations or trade-offs:

---

<!-- 
## For Maintainers

### Review Checklist
- [ ] Code quality meets project standards
- [ ] All quality gates pass
- [ ] Security implications reviewed
- [ ] Documentation is complete and accurate
- [ ] Tests provide adequate coverage
- [ ] Breaking changes are justified and documented
- [ ] Performance impact is acceptable
-->