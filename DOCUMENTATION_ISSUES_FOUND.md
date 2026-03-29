# Documentation Issues Found - 2026-03-29

## Critical Issues

### 1. forge create Commands Missing --broadcast Flag

**Severity:** CRITICAL  
**Impact:** Users following documentation will perform dry-run deployments only. Contracts won't actually exist on-chain.

**Affected Locations:**
- CLI_REFERENCE.md Section 11 Scenario 2 Step 2 (line ~1147)
- CLI_REFERENCE.md Section 11 Scenario 3 Step 2 (line ~1287)
- CLI_REFERENCE.md Section 11 Scenario 3 Step 3 (line ~1295)

**Current (broken):**
```bash
forge create --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  config/SimpleNFT.sol:SimpleNFT
```

**Should be:**
```bash
forge create --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast \
  config/SimpleNFT.sol:SimpleNFT
```

**Evidence:**
Without --broadcast, forge outputs:
```
Warning: Dry run enabled, not broadcasting transaction
```

### 2. CLI Output Format Mismatch

**Severity:** MEDIUM  
**Impact:** Users won't recognize success output format.

**Affected Location:**
- CLI_REFERENCE.md Section 7.2 send command (line ~244)

**Documentation shows:**
```
Transaction: 0x43aab2ff... in block 1
```

**Actual output:**
```
Transaction success: 0x62f83df... in block 54
```

**Fix:** Update expected output example to match actual CLI behavior.

## Test Results

- Total examples tested: 12
- Passing: 8 (67%)
- Failing: 4 (33%)

## Verification

All issues confirmed by running examples exactly as written in bash:
```bash
./scripts/test-docs-as-written.sh
```

See full test output in documentation_issues.txt.

## Recommendation

**Priority:** Fix --broadcast flag immediately. Without this, Scenarios 2 and 3 are completely non-functional for users following the documentation.
