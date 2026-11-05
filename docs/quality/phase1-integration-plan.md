# Phase 1 Integration Plan: Wiring Existing Implementations

## Overview

This plan addresses the gap between "code exists" and "code is functional" identified in the
quality assessment. All 10 Phase A tasks have implementations, but many are not integrated
into the running system. This plan wires them up to raise the quality score from 6.4 to 8.0+.

**Target**: Close integration gaps, not add new features.

## Current Quality Score: 6.4/10

| Dimension | Current | Target | Gap |
|-----------|---------|--------|-----|
| Security | 6/10 | 8/10 | CSRF/rate-limit defined but not wired |
| Performance | 5/10 | 8/10 | Inverted index/cache defined but not used |
| Data Integrity | 7/10 | 8/10 | Atomic writes work but inefficient |
| Error Handling | 6/10 | 8/10 | Basic enhanced, specialized missing |
| Test Coverage | 7/10 | 8/10 | E2E/property basic |
| Code Quality | 6/10 | 8/10 | Dead code, unused functions |
| Documentation | 8/10 | 8/10 | Already good |

## Priority 1: CSRF Protection (Critical - 2-3 days)

### Gap
- `validate_csrf_token()` middleware exists but is commented out
- CSRF token generated during login but not exposed to clients
- Forms don't include CSRF hidden field
- Protection is completely non-functional

### Steps
1. **Expose CSRF token to client** (0.5 day)
   - Add `csrf_token` to login response as HTTP-only cookie or response header
   - Store token in session cookie alongside session token
   - File: `platform/src/governance_web.rs` login_post()

2. **Add CSRF hidden field to all forms** (1 day)
   - Add `<input type="hidden" name="csrf_token" value="{{csrf_token}}">` to all POST forms
   - Inject CSRF token into page context for template rendering
   - Files: `platform/src/governance_web.rs` (all form handlers)

3. **Wire up CSRF middleware** (0.5 day)
   - Uncomment middleware in router
   - Update `validate_csrf_token()` to check form body for CSRF token
   - File: `platform/src/governance_web.rs` router()

4. **Update tests** (0.5 day)
   - Add CSRF token to test requests
   - Add test for CSRF rejection
   - File: `platform/tests/governance_web.rs`

### Acceptance Criteria
- [ ] Login response includes CSRF token (cookie or header)
- [ ] All POST forms include CSRF hidden field
- [ ] CSRF middleware rejects requests without valid token
- [ ] Existing tests pass with CSRF token included
- [ ] New test verifies CSRF rejection returns 403

### Effort: 2-3 days

---

## Priority 2: Rate Limiting (Critical - 1-2 days)

### Gap
- `RateLimiter` struct and `check_rate_limit()` method exist
- `rate_limit_guard()` middleware exists
- Middleware is commented out in router
- Global static state causes test failures

### Steps
1. **Make rate limiter test-aware** (0.5 day)
   - Add `EKPI_TEST_MODE` environment variable check
   - Skip rate limiting when in test mode
   - File: `platform/src/governance_web.rs` rate_limit_guard()

2. **Wire up rate limiting middleware** (0.5 day)
   - Uncomment middleware in router
   - Wire up IP extraction properly
   - File: `platform/src/governance_web.rs` router()

3. **Add rate limit headers** (0.5 day)
   - Add `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers
   - Use `rate_limit_error_response()` from errors.rs
   - File: `platform/src/governance_web.rs` rate_limit_guard()

4. **Update tests** (0.5 day)
   - Add test for rate limit rejection
   - Verify rate limit headers in responses
   - File: `platform/tests/governance_web.rs`

### Acceptance Criteria
- [ ] Rate limiting middleware is active in production
- [ ] Rate limiting is bypassed in test mode (EKPI_TEST_MODE=1)
- [ ] Rate limit headers present in all responses
- [ ] Rate limit exceeded returns 429 with retry-after
- [ ] Existing tests pass

### Effort: 1-2 days

---

## Priority 3: Inverted Index Usage (High - 1 day)

### Gap
- `InvertedIndex` struct with `build()` and `search()` methods exists
- `build_inverted_index()` and `search_with_index()` on Retriever exist
- `search()` method does NOT use inverted index (falls back to linear)
- `extract_bigrams` has UTF-8 bug (byte indices instead of char indices)

### Steps
1. **Fix extract_bigrams for UTF-8** (0.25 day)
   - Change to use char indices: `text.chars().collect::<Vec<_>>()`
   - Iterate over char windows instead of byte windows
   - File: `platform/src/retrieval.rs` extract_bigrams()

2. **Make search() use inverted index** (0.25 day)
   - Update `search()` to call `search_with_index()` when pool is non-empty
   - Keep linear search as fallback for empty pool
   - File: `platform/src/retrieval.rs` search()

3. **Add performance test** (0.25 day)
   - Benchmark search with large corpus
   - Verify O(1) per term lookup
   - File: `platform/tests/retrieval_perf.rs`

4. **Update existing tests** (0.25 day)
   - Ensure all retrieval tests still pass
   - Add test for UTF-8 bigram extraction
   - File: `platform/tests/retrieval.rs`

### Acceptance Criteria
- [ ] extract_bigrams handles UTF-8 correctly
- [ ] search() uses inverted index by default
- [ ] Search performance is O(1) per unique term
- [ ] All existing retrieval tests pass
- [ ] New UTF-8 test passes

### Effort: 1 day

---

## Priority 4: Specialized Error Responses (High - 1 day)

### Gap
- `error_response()` enhanced with timestamp, request_id, details, documentation_url
- `error_response_with_context()` added
- `validation_error_response()` - NOT IMPLEMENTED
- `auth_error_response()` - NOT IMPLEMENTED
- `rate_limit_error_response()` - NOT IMPLEMENTED

### Steps
1. **Implement validation_error_response()** (0.25 day)
   - Field-specific error details
   - Error count and summary
   - File: `platform/src/errors.rs`

2. **Implement auth_error_response()** (0.25 day)
   - Authentication-specific context (attempt count, lockout time)
   - Account status information
   - File: `platform/src/errors.rs`

3. **Implement rate_limit_error_response()** (0.25 day)
   - Rate limit details (limit, remaining, reset_time)
   - Retry-after calculation
   - File: `platform/src/errors.rs`

4. **Wire up in handlers** (0.25 day)
   - Use validation_error_response in form handlers
   - Use auth_error_response in login handler
   - Use rate_limit_error_response in rate limiter
   - Files: `platform/src/governance_web.rs`

### Acceptance Criteria
- [ ] validation_error_response returns field-specific errors
- [ ] auth_error_response returns authentication context
- [ ] rate_limit_error_response returns rate limit details
- [ ] Handlers use specialized error responses
- [ ] All tests pass

### Effort: 1 day

---

## Priority 5: JSON Parsing Cache Integration (Medium - 0.5 day)

### Gap
- `parse_json_cached()` function exists with OnceLock cache
- Never called anywhere in codebase
- Dead code

### Steps
1. **Integrate cache into DocDatabase** (0.25 day)
   - Use parse_json_cached in DocDatabase::load()
   - Cache parsed documents by path
   - File: `platform/src/docs.rs`

2. **Add cache invalidation** (0.25 day)
   - Clear cache on document reload
   - Add cache size limit
   - File: `platform/src/retrieval.rs`

### Acceptance Criteria
- [ ] parse_json_cached is called in production code
- [ ] Cache improves load performance
- [ ] Cache can be cleared for testing
- [ ] All tests pass

### Effort: 0.5 day

---

## Priority 6: Atomic Write Efficiency (Medium - 1 day)

### Gap
- Atomic write-to-temp + rename + checksum verification works
- O(n) per write (reads entire file)
- No file locking in append()
- Redundant checksum computation

### Steps
1. **Simplify atomic writes** (0.5 day)
   - Replace read-all + rewrite with append + fsync
   - Keep rename for true atomicity on crash
   - Use file locking (LocalFileLock) in append()
   - File: `platform/src/governance.rs` append()

2. **Remove redundant checksum** (0.25 day)
   - Compute checksum once, verify on read
   - Remove double checksum computation
   - File: `platform/src/governance.rs`

3. **Add concurrency test** (0.25 day)
   - Test concurrent writes don't corrupt
   - Verify file locking prevents conflicts
   - File: `platform/tests/governance_concurrent.rs`

### Acceptance Criteria
- [ ] append() uses file locking
- [ ] Write is O(1) (append, not rewrite)
- [ ] fsync ensures durability
- [ ] Concurrent writes are safe
- [ ] All tests pass

### Effort: 1 day

---

## Priority 7: Dead Code Removal (Medium - 0.5 day)

### Gap
- Multiple functions defined but never used
- Code bloat and maintenance burden

### Steps
1. **Audit dead code** (0.25 day)
   - Run cargo clippy with dead_code lint
   - Identify all unused functions
   - Files: all platform/src/*.rs

2. **Remove or wire up dead code** (0.25 day)
   - Remove unused functions or wire them up
   - Update tests for removed functions
   - Files: various

### Dead Code Inventory
| Function | File | Action |
|----------|------|--------|
| `parse_json_cached` | retrieval.rs | Wire up (Priority 5) |
| `clear_json_cache` | retrieval.rs | Keep (testing utility) |
| `load_recent` | governance.rs | Remove or implement properly |
| `build_inverted_index` | retrieval.rs | Wire up (Priority 3) |
| `search_with_index` | retrieval.rs | Wire up (Priority 3) |
| `validate_csrf_token` | governance_web.rs | Wire up (Priority 1) |
| `rate_limit_guard` | governance_web.rs | Wire up (Priority 2) |
| `RateLimiter` | governance_web.rs | Wire up (Priority 2) |

### Acceptance Criteria
- [ ] No dead_code warnings in cargo build
- [ ] All unused functions removed or wired up
- [ ] All tests pass

### Effort: 0.5 day

---

## Priority 8: E2E Test Setup (Medium - 1 day)

### Gap
- Playwright test file exists
- No Playwright configuration
- No package.json
- No CI integration

### Steps
1. **Add Playwright configuration** (0.25 day)
   - Create playwright.config.ts
   - Set base URL, browser, timeout
   - File: `platform/tests/e2e/playwright.config.ts`

2. **Add package.json** (0.25 day)
   - Add Playwright dependency
   - Add test scripts
   - File: `platform/tests/e2e/package.json`

3. **Add CI workflow** (0.25 day)
   - Add E2E test job to .github/workflows/ci.yml
   - Start server, run tests, collect artifacts
   - File: `.github/workflows/ci.yml`

4. **Verify E2E tests run** (0.25 day)
   - Run Playwright tests locally
   - Fix any issues
   - Document setup in README

### Acceptance Criteria
- [ ] Playwright tests run successfully
- [ ] CI workflow includes E2E tests
- [ ] Documentation for E2E setup
- [ ] All E2E tests pass

### Effort: 1 day

---

## Priority 9: Property Test Expansion (Medium - 1 day)

### Gap
- 3 basic property tests exist
- No engine.decide() property tests
- No citation verification tests
- Simple string generators

### Steps
1. **Add engine.decide() property tests** (0.5 day)
   - Test with random rule sets
   - Test with random case inputs
   - Verify output invariants
   - File: `platform/tests/property_tests.rs`

2. **Add citation verification tests** (0.25 day)
   - Test citation round-trip
   - Test verification with corrupted data
   - File: `platform/tests/property_tests.rs`

3. **Improve generators** (0.25 day)
   - Add complex data generators
   - Add edge case generators
   - File: `platform/tests/property_tests.rs`

### Acceptance Criteria
- [ ] 10+ property tests for engine.decide()
- [ ] Citation verification property tests
- [ ] Edge case generators
- [ ] All property tests pass

### Effort: 1 day

---

## Priority 10: Backup Script Testing (Low - 0.5 day)

### Gap
- Backup script exists but untested
- No integrity verification

### Steps
1. **Test backup script** (0.25 day)
   - Execute script in test environment
   - Verify backup file creation
   - Verify backup contents
   - File: `scripts/schedule-backup.ps1`

2. **Add integrity verification** (0.25 day)
   - Add SHA-256 checksum to backup
   - Verify checksum on restore
   - File: `scripts/schedule-backup.ps1`

### Acceptance Criteria
- [ ] Backup script runs successfully
- [ ] Backup file is created
- [ ] Backup contains expected data
- [ ] Integrity verification works

### Effort: 0.5 day

---

## Summary

| Priority | Task | Effort | Impact |
|----------|------|--------|--------|
| 1 | CSRF Protection | 2-3 days | Critical security |
| 2 | Rate Limiting | 1-2 days | Critical DoS protection |
| 3 | Inverted Index Usage | 1 day | High performance |
| 4 | Specialized Error Responses | 1 day | High client compatibility |
| 5 | JSON Parsing Cache | 0.5 day | Medium performance |
| 6 | Atomic Write Efficiency | 1 day | Medium data integrity |
| 7 | Dead Code Removal | 0.5 day | Medium code quality |
| 8 | E2E Test Setup | 1 day | Medium test coverage |
| 9 | Property Test Expansion | 1 day | Medium test coverage |
| 10 | Backup Script Testing | 0.5 day | Low operations |

**Total Effort: 9.5-11.5 days**

## Expected Quality Score Improvement

| Dimension | Before | After | Improvement |
|-----------|--------|-------|-------------|
| Security | 6/10 | 8.5/10 | +2.5 |
| Performance | 5/10 | 8/10 | +3.0 |
| Data Integrity | 7/10 | 8.5/10 | +1.5 |
| Error Handling | 6/10 | 8.5/10 | +2.5 |
| Test Coverage | 7/10 | 8.5/10 | +1.5 |
| Code Quality | 6/10 | 8/10 | +2.0 |
| Documentation | 8/10 | 8/10 | +0.0 |

**Overall: 6.4/10 → 8.3/10** (target: 8.0+)
