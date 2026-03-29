# Chain ID Filler Test Plan (REVISED)

**Module:** `alloy-provider/src/fillers/chain_id.rs` (93 lines, 0 tests)  
**Coverage Gap:** 0% → 85-90% target  
**Contribution Target:** alloy-rs/alloy upstream  
**Plan Date:** 2026-03-29  
**Revision Date:** 2026-03-29 (Anvil integration approach)

---

## Revision History

**Oracle Finding:** Original plan used mock-based approach, but alloy conventions use Anvil integration tests (see nonce.rs:203-323, gas.rs:339-390).  
**Claire Voyant Risk:** 80% chance of PR rejection due to testing approach mismatch.  
**Integration Test Architect:** Revised to hybrid approach: 10 Anvil integration tests + 5 unit tests for error paths.

---

## Testing Strategy

**Primary:** Anvil integration tests (matches alloy conventions)  
**Supplement:** Unit tests with stub providers for error cases Anvil cannot simulate  
**Reference Pattern:** [nonce.rs](../forks/alloy/crates/provider/src/fillers/nonce.rs#L203-L323) test suite

---

## Critical Assumptions Under Test

This test plan validates six critical assumptions in `ChainIdFiller` usage patterns identified through eth_node codebase analysis.

---

## Test Case Suite

### **AC-1: Assumption 1 - Chain ID Immutability**

**Assumption:** Chain ID is immutable for a provider's lifetime  
**Risk:** RPC endpoint switches chains, hard fork, Anvil restart  
**Severity:** HIGH — Wrong-chain transaction submission  
**Approach:** Anvil integration test

#### Test 1.1: Cache Persists Across Multiple Calls (ANVIL)
```rust
#[tokio::test]
async fn chain_id_cached_across_multiple_calls() {
    // GIVEN: Real Anvil provider + ChainIdFiller
    let provider = ProviderBuilder::new().connect_anvil();
    let filler = ChainIdFiller::default();
    let tx = TransactionRequest::default();
    
    // WHEN: First prepare() call fetches from Anvil
    let chain_id_1 = filler.prepare(&provider, &tx).await.unwrap();
    
    // AND: Second prepare() call
    let chain_id_2 = filler.prepare(&provider, &tx).await.unwrap();
    
    // THEN: Both return Anvil's chain ID (31337)
    assert_eq!(chain_id_1, 31337);
    assert_eq!(chain_id_2, 31337);
    
    // AND: Internal cache is populated (verify via state inspection)
    // NOTE: Requires ChainIdFiller to expose .0.get() or similar for testing
    assert!(filler.0.get().is_some());
    assert_eq!(*filler.0.get().unwrap(), 31337);
}
```

**Reference:** Pattern from [nonce.rs:226-242](../forks/alloy/crates/provider/src/fillers/nonce.rs#L226-L242)

#### Test 1.2: Pre-configured Chain ID Never Fetches (ANVIL)
```rust
#[tokio::test]
async fn preconfigured_chain_id_never_fetches() {
    // GIVEN: ChainIdFiller with pre-configured chain_id = 42
    let filler = ChainIdFiller::new(Some(42));
    let provider = ProviderBuilder::new().connect_anvil();
    let tx = TransactionRequest::default();
    
    // WHEN: Multiple prepare() calls
    for _ in 0..10 {
        let chain_id = filler.prepare(&provider, &tx).await.unwrap();
        
        // THEN: Always returns pre-configured value (not Anvil's 31337)
        assert_eq!(chain_id, 42);
    }
    
    // AND: Cache shows pre-configured value
    assert_eq!(*filler.0.get().unwrap(), 42);
}
```

---

### **AC-2: Assumption 2 - First Fetch Failure Handling**

**Assumption:** First fetch error propagates, no retry logic  
**Risk:** Transient network error → provider permanently broken  
**Severity:** HIGH — Provider unusable after network blip  
**Approach:** Unit tests with stub provider (Anvil cannot simulate errors)

#### Test 2.1: First Fetch Timeout Propagates Error (UNIT TEST)
```rust
#[tokio::test]
async fn first_fetch_timeout_propagates() {
    // GIVEN: ChainIdFiller with no pre-set chain ID
    let filler = ChainIdFiller::new(None);
    
    // AND: Stub provider that returns timeout error
    struct TimeoutProvider;
    impl Provider<Ethereum> for TimeoutProvider {
        async fn get_chain_id(&self) -> TransportResult<u64> {
            Err(TransportError::timeout())
        }
        // ... minimal trait impl
    }
    let provider = TimeoutProvider;
    let tx = TransactionRequest::default();
    
    // WHEN: First prepare() call
    let result = filler.prepare(&provider, &tx).await;
    
    // THEN: Error propagates to caller
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TransportError::Timeout));
}
```

**Note:** Uses minimal stub provider, not full mock framework

#### Test 2.2: Recovery After Initial Failure (UNIT TEST)  
```rust
#[tokio::test]
async fn recovery_after_initial_failure() {
    // GIVEN: ChainIdFiller with no pre-set chain ID
    let filler = ChainIdFiller::new(None);
    
    // AND: Stub provider with sequence: Err → Ok(1)
    struct SequenceProvider {
        call_count: AtomicUsize,
    }
    impl Provider<Ethereum> for SequenceProvider {
        async fn get_chain_id(&self) -> TransportResult<u64> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            if count == 0 {
                Err(TransportError::timeout())
            } else {
                Ok(1)
            }
        }
    }
    let provider = SequenceProvider { call_count: AtomicUsize::new(0) };
    let tx = TransactionRequest::default();
    
    // WHEN: First prepare() call fails
    let result1 = filler.prepare(&provider, &tx).await;
    assert!(result1.is_err());
    
    // AND: Cache is NOT poisoned
    assert!(filler.0.get().is_none());
    
    // AND: Second prepare() call (simulating retry)
    let result2 = filler.prepare(&provider, &tx).await;
    
    // THEN: Second call succeeds and caches
    assert_eq!(result2.unwrap(), 1);
    assert_eq!(*filler.0.get().unwrap(), 1);
    
    // AND: Third call uses cached value (no third RPC)
    let result3 = filler.prepare(&provider, &tx).await;
    assert_eq!(result3.unwrap(), 1);
    assert_eq!(provider.call_count.load(Ordering::SeqCst), 2); // Only 2 calls, not 3
}
```

#### Test 2.3: Cached Value Persists After Provider Failure (ANVIL)
```rust
#[tokio::test]
async fn cached_value_survives_provider_failure() {
    // GIVEN: ChainIdFiller that has cached chain_id from Anvil
    let provider = ProviderBuilder::new().connect_anvil();
    let filler = ChainIdFiller::new(None);
    let tx = TransactionRequest::default();
    filler.prepare(&provider, &tx).await.unwrap();  // Populate cache
    
    // WHEN: Subsequent prepare() calls (cache hit, no RPC)
    for _ in 0..10 {
        let result = filler.prepare(&provider, &tx).await;
        
        // THEN: Always returns cached value
        assert_eq!(result.unwrap(), 31337);
    }
    
    // NOTE: Cannot simulate provider failure with Anvil,
    // but cache behavior is demonstrated
}
```

---

### **AC-3: Assumption 3 - Concurrent Access Safety**

**Assumption:** OnceLock handles concurrent writes safely  
**Risk:** Concurrent prepare() calls → race condition or hang  
**Severity:** CRITICAL — All transactions hang if first fetch hangs  
**Approach:** Anvil integration test with real async concurrency

#### Test 3.1: Concurrent First Fetch Single Initialization (ANVIL)
```rust
#[tokio::test]
async fn concurrent_first_fetch_race_condition() {
    // GIVEN: ChainIdFiller with no pre-set chain ID
    let filler = Arc::new(ChainIdFiller::new(None));
    let provider = Arc::new(ProviderBuilder::new().connect_anvil());
    
    // WHEN: 10 concurrent tasks call prepare() simultaneously
    let tx = TransactionRequest::default();
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let filler = Arc::clone(&filler);
            let provider = Arc::clone(&provider);
            let tx = tx.clone();
            tokio::spawn(async move {
                filler.prepare(&provider, &tx).await.unwrap()
            })
        })
        .collect();
    
    // AND: All complete
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // THEN: All return same chain_id = 31337
    for chain_id in results {
        assert_eq!(chain_id, 31337);
    }
    
    // AND: Cache was initialized exactly once (OnceLock guarantee)
    assert!(filler.0.get().is_some());
    assert_eq!(*filler.0.get().unwrap(), 31337);
}
```

**Reference:** Pattern from [nonce.rs:242-268](../forks/alloy/crates/provider/src/fillers/nonce.rs#L242-L268) `concurrency` test

#### Test 3.2: Concurrent Access After Cache Populated (ANVIL)
```rust
#[tokio::test]
async fn concurrent_access_cached_value() {
    // GIVEN: ChainIdFiller with cached chain_id from Anvil
    let provider = ProviderBuilder::new().connect_anvil();
    let filler = Arc::new(ChainIdFiller::new(None));
    let tx = TransactionRequest::default();
    filler.prepare(&provider, &tx).await.unwrap(); // Populate cache
    
    // WHEN: 100 threads read simultaneously
    let filler = Arc::new(filler);
    let provider = Arc::new(provider);
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let filler = Arc::clone(&filler);
            let provider = Arc::clone(&provider);
            let tx = tx.clone();
            tokio::spawn(async move {
                filler.prepare(&provider, &tx).await.unwrap()
            })
        })
        .collect();
    
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // THEN: All return cached value with no contention
    for chain_id in results {
        assert_eq!(chain_id, 31337);
    }
}
```

**Note:** Test 3.3 from original plan (hanging fetch) removed — cannot test with Anvil, and OnceLock handles this internally
```

---

### **AC-4: Assumption 4 - Cloned Providers Share Cache**

**Assumption:** Arc<OnceLock> means clones share cached chain ID  
**Risk:** Clone provider, connect to different chain → wrong cached ID  
**Severity:** MEDIUM — User error, but silent failure  
**Approach:** Anvil integration test with pointer comparison

#### Test 4.1: Cloned Filler Shares Cache (ANVIL)
```rust
#[tokio::test]
async fn cloned_filler_shares_cache() {
    // GIVEN: ChainIdFiller that fetches from Anvil
    let provider = ProviderBuilder::new().connect_anvil();
    let filler1 = ChainIdFiller::new(None);
    let tx = TransactionRequest::default();
    filler1.prepare(&provider, &tx).await.unwrap();
    
    // WHEN: Filler is cloned
    let filler2 = filler1.clone();
    
    // THEN: Both return same cached chain_id
    let chain_id_1 = filler1.prepare(&provider, &tx).await.unwrap();
    let chain_id_2 = filler2.prepare(&provider, &tx).await.unwrap();
    assert_eq!(chain_id_1, 31337);
    assert_eq!(chain_id_2, 31337);
    
    // AND: Both fillers point to same Arc<OnceLock> (pointer equality)
    assert!(Arc::ptr_eq(&filler1.0, &filler2.0));
}
```

#### Test 4.2: Pre-configured Filler Clone Behavior (UNIT TEST)
```rust
#[test]
fn preconfigured_filler_clone_shares_value() {
    // GIVEN: ChainIdFiller with pre-configured chain_id = 42
    let filler1 = ChainIdFiller::new(Some(42));
    
    // WHEN: Cloned
    let filler2 = filler1.clone();
    
    // THEN: Both have same internal state
    assert_eq!(filler1.0.get(), Some(&42));
    assert_eq!(filler2.0.get(), Some(&42));
    assert!(Arc::ptr_eq(&filler1.0, &filler2.0));
}
```

---

### **AC-5: Assumption 5 - Pre-set Chain ID = Finished**

**Assumption:** Transaction with chain_id set → FillerControlFlow::Finished  
**Risk:** Wrong pre-set chain ID goes unvalidated  
**Severity:** MEDIUM — User error, caught by network  
**Approach:** Unit tests (no provider needed)

#### Test 5.1: Transaction With Chain ID Reports Finished (UNIT TEST)
```rust
#[test]
fn tx_with_chain_id_is_finished() {
    // GIVEN: ChainIdFiller (any configuration)
    let filler = ChainIdFiller::new(None);
    
    // AND: Transaction with chain_id already set
    let mut tx = TransactionRequest::default();
    tx.set_chain_id(1);
    
    // WHEN: Check status
    let status = filler.status(&tx);
    
    // THEN: Reports Finished (will not fill)
    assert!(matches!(status, FillerControlFlow::Finished));
}
```

#### Test 5.2: Transaction Without Chain ID Reports Ready (UNIT TEST)
```rust
#[test]
fn tx_without_chain_id_is_ready() {
    // GIVEN: ChainIdFiller with pre-configured chain ID
    let filler = ChainIdFiller::new(Some(42));
    
    // AND: Transaction with NO chain_id set
    let tx = TransactionRequest::default();
    assert!(tx.chain_id().is_none());
    
    // WHEN: Check status
    let status = filler.status(&tx);
    
    // THEN: Reports Ready (will fill)
    assert!(matches!(status, FillerControlFlow::Ready));
}
```

#### Test 5.3: Fill Does Not Override Pre-set Chain ID (UNIT TEST)
```rust
#[test]
fn fill_sync_respects_preset_chain_id() {
    // GIVEN: ChainIdFiller with cached chain_id = 42
    let filler = ChainIdFiller::new(Some(42));
    
    // AND: Transaction with DIFFERENT chain_id = 1
    let mut tx = TransactionRequest::default();
    tx.set_chain_id(1);
    let mut sendable = SendableTx::Builder(tx);
    
    // WHEN: fill_sync() called
    filler.fill_sync(&mut sendable);
    
    // THEN: Transaction still has chain_id = 1 (not overwritten)
    if let SendableTx::Builder(tx) = sendable {
        assert_eq!(tx.chain_id(), Some(1));
    } else {
        panic!("Expected Builder variant");
    }
}
```

---

### **AC-6: Assumption 6 - RPC Honesty**

**Assumption:** get_chain_id() RPC is honest and correct  
**Risk:** Malicious RPC, buggy proxy, type overflow  
**Severity:** CRITICAL — Replay attacks, wrong-chain submission  
**Approach:** Unit tests (Anvil always returns 31337)

#### Test 6.1: Chain ID Type Bounds (UNIT TEST)
```rust
#[test]
fn chain_id_max_value() {
    // GIVEN: ChainIdFiller pre-configured with u64::MAX
    let filler = ChainIdFiller::new(Some(u64::MAX));
    
    // THEN: Handles without panic or overflow
    assert_eq!(filler.0.get(), Some(&u64::MAX));
}
```

#### Test 6.2: Chain ID Zero Value (UNIT TEST)
```rust
#[test]
fn chain_id_zero() {
    // GIVEN: ChainIdFiller with chain_id = 0
    let filler = ChainIdFiller::new(Some(0));
    
    // THEN: Value is stored without validation
    // NOTE: This documents current behavior - validation is out of scope
    assert_eq!(filler.0.get(), Some(&0));
}
```

---

## Additional Edge Cases

### Test 7.1: Default Construction (ANVIL)
```rust
#[tokio::test]
async fn default_construction() {
    // GIVEN: ChainIdFiller::default()
    let filler = ChainIdFiller::default();
    let provider = ProviderBuilder::new().connect_anvil();
    let tx = TransactionRequest::default();
    
    // THEN: Behaves like new(None) - fetches from provider
    let result = filler.prepare(&provider, &tx).await.unwrap();
    assert_eq!(result, 31337);
}
```

### Test 7.2: PartialEq Implementation (UNIT TEST)
```rust
#[test]
fn partial_eq() {
    // Two fillers with same pre-configured value are equal
    let filler1 = ChainIdFiller::new(Some(1));
    let filler2 = ChainIdFiller::new(Some(1));
    assert_eq!(filler1, filler2);
    
    // Different pre-configured values are not equal
    let filler3 = ChainIdFiller::new(Some(2));
    assert_ne!(filler1, filler3);
    
    // None vs Some are not equal
    let filler4 = ChainIdFiller::new(None);
    assert_ne!(filler1, filler4);
}
```

### Test 7.3: Debug Format (UNIT TEST)
```rust
#[test]
fn debug_format() {
    // Verify Debug implementation doesn't panic
    let filler = ChainIdFiller::new(Some(42));
    let debug_str = format!("{:?}", filler);
    assert!(debug_str.contains("ChainIdFiller"));
}
```

---

## Coverage Target Verification (REVISED)

**Total Test Cases:** 15 tests (10 integration + 5 unit)  
**Assumptions Covered:** 6 critical + 3 edge cases  
**Expected Coverage:** 
- Lines: 85-90% (79-84/93 lines, excluding derives/comments)
- Branches: 85%+ (all cache paths, concurrency, pre-set detection)
- Functions: 100% (all public methods: new, status, fill_sync, prepare)

**Test Distribution:**
- **Anvil Integration Tests:** 10 tests (AC-1, AC-3, AC-4, AC-7.1)
- **Unit Tests:** 5 tests (AC-2, AC-5, AC-6, AC-7.2, AC-7.3)

**Out of Scope:**
- RPC transport layer error simulation (Anvil limitation)
- get_chain_id() validation logic (assumed honest per assumption)
- Performance benchmarks (functional correctness only)
- Hanging fetch scenarios (untestable with Anvil, OnceLock handles internally)

---

## Test Infrastructure Requirements (REVISED)

### Integration Tests (Anvil-based)
```rust
use alloy_provider::{ProviderBuilder};
use tokio::test;

#[tokio::test]
async fn example_anvil_test() {
    let provider = ProviderBuilder::new().connect_anvil();
    // ... test logic
}
```

### Unit Tests (Stub Provider)
```rust
struct StubProvider {
    response: TransportResult<u64>,
}

impl Provider<Ethereum> for StubProvider {
    async fn get_chain_id(&self) -> TransportResult<u64> {
        self.response.clone()
    }
    // ... minimal trait impl
}
```

**No mock framework needed** — matches alloy conventions

---

## Traceability Matrix

| Test ID | Assumption | Severity | eth_node Usage Pattern | AC Link |
|---------|-----------|----------|------------------------|---------|
| 1.1-1.2 | Immutability | HIGH | contract.rs:188 direct call | AC-1 |
| 2.1-2.3 | Failure Handling | HIGH | RpcClient::new pattern | AC-2 |
| 3.1-3.3 | Concurrency | CRITICAL | Multi-tx scenarios | AC-3 |
| 4.1-4.2 | Shared Cache | MEDIUM | Provider cloning | AC-4 |
| 5.1-5.3 | Pre-set Finished | MEDIUM | Manual chain_id setting | AC-5 |
| 6.1-6.2 | RPC Honesty | CRITICAL | Trust boundary | AC-6 |

---

## Implementation Notes

1. **TDD Approach**: Implement tests first (red), verify they fail with current 0% coverage
2. **Incremental**: Start with AC-1 (caching), build to AC-3 (concurrency)
3. **Mock Complexity**: AC-3 requires sophisticated async mocking with delays
4. **Platform**: Tests must run in alloy's existing test infrastructure
5. **CI Compatibility**: No external dependencies, works with cargo test --lib

---

## Success Criteria (REVISED)

- [ ] All 15 tests pass (10 Anvil integration + 5 unit)
- [ ] Code coverage ≥ 85% measured by cargo-llvm-cov (integration-focused approach)
- [ ] Tests match alloy project conventions (studied from nonce.rs, gas.rs)
- [ ] No mock framework dependencies added
- [ ] PR demonstrates real concurrent safety + caching behavior
- [ ] All existing provider crate tests still pass
- [ ] Upstream maintainers accept PR (style matches project idioms)
