# AutoPeer API - Testing TODO

## Test Coverage Status: 62.65%

This document tracks missing test coverage based on `cargo llvm-cov` analysis.

---

## 🔴 CRITICAL - No Coverage (0%)

### 1. HTTP Server & Routing (`main.rs` - 0% coverage)

**Missing:**
- [ ] Server startup and initialization
- [ ] Route registration verification
- [ ] Middleware integration (CookieManagerLayer)
- [ ] Error handling during startup
- [ ] Graceful shutdown
- [ ] Integration test: full HTTP request/response cycle

**Priority:** HIGH - This is the entry point of the application

---

### 2. JWT Middleware (`middleware/auth.rs` - 0% coverage)

**Missing:**
- [ ] `FromRequestParts` implementation for `JwtAuth`
- [ ] Cookie extraction from HTTP requests
- [ ] JWT token decoding from cookies
- [ ] Error case: missing `autopeer_token` cookie
- [ ] Error case: invalid JWT token in cookie
- [ ] Error case: malformed cookie
- [ ] Error case: expired JWT token
- [ ] Integration with Axum extractor system

**Priority:** HIGH - Core authentication mechanism

---

### 3. API Endpoint Handlers (`api/peering.rs` - 4.78% coverage)

Currently only testing request/response struct serialization. Missing full handler logic:

#### `init_peering()` - 0% handler coverage
- [ ] Valid ASN validation
- [ ] Challenge generation
- [ ] Pending config creation
- [ ] File I/O error handling
- [ ] Response with challenge

#### `verify_peering()` - 0% handler coverage
- [ ] GPG signature verification flow
- [ ] Valid signature → JWT generation
- [ ] Cookie setting for multiple domains
- [ ] Cookie security attributes (HttpOnly, Secure, SameSite)
- [ ] WireGuard config completion
- [ ] Pending → verified config transition
- [ ] Error case: invalid signature
- [ ] Error case: wrong ASN
- [ ] Error case: missing pending config
- [ ] Error case: key fingerprint mismatch

#### `deploy_peering()` - 0% handler coverage
- [ ] JWT authentication via middleware
- [ ] ASN mismatch validation (JWT vs request body)
- [ ] Verified config loading
- [ ] WireGuard interface deployment
- [ ] BIRD config deployment
- [ ] Successful deployment response
- [ ] Error case: config not found
- [ ] Error case: deployment failure
- [ ] Error case: BIRD reload failure

#### `get_config()` - 0% handler coverage
- [ ] JWT authentication via middleware
- [ ] ASN extraction from JWT
- [ ] Verified config retrieval
- [ ] Response with config
- [ ] Error case: config not found
- [ ] Error case: invalid JWT

#### `update_peering()` - 0% handler coverage
- [ ] JWT authentication via middleware
- [ ] Optional endpoint validation
- [ ] Config update logic
- [ ] WireGuard interface update (remove old, deploy new)
- [ ] BIRD config update
- [ ] Error case: invalid endpoint format
- [ ] Error case: config not found
- [ ] Error case: re-deployment failure

#### `delete_peering()` - 0% handler coverage
- [ ] JWT authentication via middleware
- [ ] WireGuard interface removal
- [ ] BIRD config removal
- [ ] Verified config file deletion
- [ ] Success response
- [ ] Error case: config not found
- [ ] Error case: interface removal failure
- [ ] Error case: BIRD removal failure

**Priority:** HIGH - Core API functionality

---

## 🟡 HIGH - Deployment Logic

### 4. WireGuard Deployment (`wireguard/deploy.rs` - 47% coverage)

**Covered:** Keypair generation (47%)

**Missing:**
- [ ] `deploy_interface()` - actual WireGuard interface creation
  - [ ] Successful interface deployment
  - [ ] Error handling for `wg-quick up` failures
  - [ ] Permissions issues
  - [ ] Interface already exists
- [ ] `remove_interface()` - interface cleanup
  - [ ] Successful interface removal
  - [ ] Error handling for `wg-quick down` failures
  - [ ] Interface doesn't exist
- [ ] Integration with config generation

**Priority:** HIGH - Deployment is core functionality

---

### 5. BIRD Deployment (`bird/deploy.rs` - 12% coverage)

**Covered:** Path formatting (12%)

**Missing:**
- [ ] `deploy_config()` - config file writing and daemon reload
  - [ ] Successful config deployment
  - [ ] File I/O error handling
  - [ ] BIRD reload via `birdc configure`
  - [ ] Error handling for reload failures
  - [ ] Invalid config detection
- [ ] `remove_config()` - config cleanup
  - [ ] Successful config removal
  - [ ] Error handling for file deletion
  - [ ] BIRD reload after removal
- [ ] `reload_bird()` - daemon interaction
  - [ ] Successful reload
  - [ ] BIRD not running
  - [ ] Invalid config rejection

**Priority:** HIGH - BGP routing is critical

---

## 🟠 MEDIUM - Edge Cases & Error Paths

### 6. Templates (`templates.rs` - 84.62% coverage)

**Missing:**
- [ ] Template parsing errors (malformed templates)
- [ ] Template rendering errors (missing variables)
- [ ] Variable type mismatches
- [ ] Edge case: empty template
- [ ] Edge case: very large template

**Priority:** MEDIUM

---

### 7. Configuration (`config.rs` - 91.78% coverage)

**Missing:**
- [ ] Missing required environment variables
  - [ ] `JWT_SECRET` not set
  - [ ] `DN42_GIT_USERNAME` not set
  - [ ] `DN42_GIT_TOKEN` not set
- [ ] Invalid configuration values
  - [ ] Invalid bind address format
  - [ ] Invalid ASN format
  - [ ] Empty cookie domains
- [ ] Default value handling

**Priority:** MEDIUM

---

### 8. Registry (`registry/sync.rs` - 67.80%, `registry/parser.rs` - 79.92%)

**Missing:**
- [ ] Git clone failures (network errors)
- [ ] Git pull failures (merge conflicts)
- [ ] Authentication failures (invalid token)
- [ ] Malformed registry data
  - [ ] Invalid AS object format
  - [ ] Missing required fields
  - [ ] Corrupted GPG keys
- [ ] File system errors (permissions, disk full)
- [ ] Concurrent access handling

**Priority:** MEDIUM

---

## 🟢 LOW - Additional Improvements

### 9. Integration Tests

**Missing:**
- [ ] End-to-end API flow tests
  - [ ] Complete peering setup: init → verify → deploy → get_config
  - [ ] Update flow: deploy → update
  - [ ] Delete flow: deploy → delete
- [ ] Multi-user scenarios
  - [ ] Multiple ASNs simultaneously
  - [ ] Concurrent requests
- [ ] State persistence tests
  - [ ] Config survival across restarts
  - [ ] Pending config cleanup

**Priority:** LOW - Nice to have

---

### 10. Security Tests

**Missing:**
- [ ] Cookie security validation
  - [ ] HttpOnly flag set
  - [ ] Secure flag set
  - [ ] SameSite=Strict enforced
  - [ ] Correct domain setting
  - [ ] Max-Age validation
- [ ] JWT security
  - [ ] Token expiration handling
  - [ ] Invalid signature rejection
  - [ ] Token replay attacks
- [ ] ASN authorization
  - [ ] ASN mismatch attacks (JWT ASN ≠ request ASN)
  - [ ] Unauthorized config access
- [ ] GPG signature validation
  - [ ] Tampered signatures
  - [ ] Wrong key signatures
  - [ ] Replay attacks (old challenges)

**Priority:** LOW - Some coverage exists, edge cases missing

---

## 🟢 Performance Tests

**Missing:**
- [ ] Concurrent request handling
- [ ] Rate limiting (if applicable)
- [ ] Large config file handling
- [ ] Memory leak tests
- [ ] Resource cleanup verification

**Priority:** LOW

---

## Test Infrastructure Improvements

**Setup:**
- [x] Test GPG key created (`tests/fixtures/gpg/`)
- [x] Test key private part encrypted with git-crypt
- [x] Test key removed from user keyring

**Needed:**
- [ ] Mock HTTP client for API tests
- [ ] Temporary test directory helpers
- [ ] Test database/config fixtures
- [ ] Mock system command execution (for WireGuard/BIRD without root)
- [ ] CI/CD integration test environment

---

## Summary by Priority

### Must Have (Critical/High):
1. ✅ Main.rs integration tests (server startup, routing)
2. ✅ Middleware tests (JWT auth extractor)
3. ✅ API endpoint handler tests (all 6 endpoints)
4. ✅ WireGuard deployment tests
5. ✅ BIRD deployment tests

### Should Have (Medium):
6. Templates error handling
7. Config validation
8. Registry error cases

### Nice to Have (Low):
9. Full integration tests
10. Security edge cases
11. Performance tests

---

## Next Steps

1. Create test helpers for:
   - Temporary GPG keyring setup
   - Mock HTTP requests to API
   - Temporary file/directory management
   - Mock system commands (wg-quick, birdc)

2. Implement critical endpoint tests first:
   - Start with `init_peering()` - simplest flow
   - Then `verify_peering()` - uses GPG fixtures
   - Then deployment endpoints

3. Add integration tests using `axum::test` helpers

4. Mock system commands to avoid requiring root privileges

---

**Last Updated:** 2025-10-22
**Coverage Tool:** `cargo llvm-cov`
**Current Coverage:** 62.65% lines, 59.71% regions
