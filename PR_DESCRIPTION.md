# PR Title (copy this exactly):

test(provider): add ChainIdFiller test coverage (0% → 99%)

---

# PR Description (copy everything below this line):

## Summary

Adds comprehensive test coverage for `ChainIdFiller` (93 lines, previously untested). This PR implements 17 tests covering all acceptance criteria including caching behavior, error handling, concurrency safety, clone semantics, and edge cases.

## Coverage

- **Lines:** 99.24% (263/265)
- **Regions:** 99.10% (556/561)
- **Functions:** 100% (37/37)

## Test Categories

**AC-1: Immutability & Caching** (2 tests)
- Cache persistence across multiple calls
- Pre-configured values never fetch from provider

**AC-2: Error Handling** (3 tests)
- First fetch error propagates correctly
- Recovery after initial failure
- Cache persistence across errors

**AC-3: Concurrency Safety** (2 tests)
- Single initialization with 10 concurrent tasks
- 100 concurrent readers accessing populated cache

**AC-4: Clone Behavior** (2 tests)
- Cloned fillers share Arc<OnceLock> cache
- Pre-configured fillers maintain shared state

**AC-5: Pre-set Detection** (3 tests)
- `status()` returns Finished/Ready correctly
- `fill_sync()` respects pre-set chain IDs

**AC-6: Type Bounds** (2 tests)
- Handles u64::MAX without overflow
- Accepts zero value (documents current behavior)

**AC-7: Edge Cases** (3 tests)
- `default()` behaves like `new(None)`
- PartialEq semantics (same/different/empty cases)
- Debug format doesn't panic

## Pattern Compliance

Tests follow established alloy conventions:
- Anvil integration for smoke tests (matches `nonce.rs:203-323`)
- MockTransport (Asserter pattern) for error simulation
- GIVEN/WHEN/THEN inline comments for test clarity
- Real `tokio::spawn` for concurrency tests

## Motivation

The `ChainIdFiller` module (93 lines) had zero test coverage. This PR addresses that gap by implementing comprehensive tests covering all public API surface, error paths, concurrency safety, and edge cases.

## Solution

Added 17 tests within the `chain_id.rs` module following alloy's established testing patterns (Anvil integration + MockTransport). Tests verify immutability guarantees, error handling, concurrent access safety, and edge case behavior.

## Testing

```bash
cargo test --lib --package alloy-provider chain_id
```

**Result:** 18 tests passed (17 chain_id + 1 existing provider test)

## PR Checklist

- [x] Added Tests
- [ ] Added Documentation (inline comments sufficient)
- [ ] Breaking changes (none - tests only)
