#!/bin/bash
# Complete documentation validation - all scenarios including complex multi-step workflows
# Tests every executable example from CLI_REFERENCE.md

export PATH="/c/Program Files/Git/usr/bin:$PATH"
set +e

ETH_CLI="./target/release/eth_node_cli.exe"
RPC="http://127.0.0.1:8545"
PK0="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
PK1="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
ADDR0="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
ADDR1="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
ADDR2="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"

PASS=0
FAIL=0

test_it() {
    echo ""; echo "====== $1 ======"
    if eval "$2" 2>&1; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
    fi
}

echo "=========================================="
echo "Complete Documentation Validation"
echo "=========================================="
echo ""

# ========================================
# SECTION 7: Basic CLI Commands
# ========================================

echo "## Section 7.1-7.3: Basic Commands"

test_it "balance (basic)" '$ETH_CLI balance $ADDR0 | grep -q "Balance:"'
test_it "balance (quiet)" '$ETH_CLI --quiet balance $ADDR0 > /dev/null'
test_it "send ETH" '
    OUT=$($ETH_CLI send --private-key $PK0 $ADDR1 1000000000000000000)
    TX=$(echo "$OUT" | grep -oP "0x[0-9a-f]{64}" | head -1)
    [ -n "$TX" ]
'
test_it "tx-status" '
    [ -n "$TX" ] || return 1
    $ETH_CLI tx-status $TX | grep -q "success"
'

# ========================================
# SECTION 7.4: decode-receipt Walkthrough
# ========================================

echo ""; echo "## Section 7.4: decode-receipt Walkthrough"

test_it "deploy TestERC721" '
    OUT=$(forge create src/eth_node/tests/contracts/TestERC721.sol:TestERC721 \
        --rpc-url $RPC --private-key $PK0 --broadcast 2>&1)
    ERC721=$(echo "$OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    [ -n "$ERC721" ]
    echo "Contract: $ERC721"
'

test_it "emit+decode Transfer" '
    [ -n "$ERC721" ] || return 1
    OUT=$(cast send $ERC721 \
        "emitTransfer(address,address,uint256)" \
        0x2121212121212121212121212121212121212121 \
        0x2222222222222222222222222222222222222222 \
        42 \
        --rpc-url $RPC --private-key $PK0 2>&1)
    TXHASH=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$TXHASH" ] || return 1
    DECODE=$($ETH_CLI decode-receipt $TXHASH)
    echo "$DECODE" | grep -q "Transfer" || return 1
    echo "$DECODE" | grep -q "token_id=42" || return 1
'

test_it "deploy TestERC1155" '
    OUT=$(forge create src/eth_node/tests/contracts/TestERC1155.sol:TestERC1155 \
        --rpc-url $RPC --private-key $PK0 --broadcast 2>&1)
    ERC1155=$(echo "$OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    [ -n "$ERC1155" ]
    echo "Contract: $ERC1155"
'

test_it "emit+decode ApprovalForAll (ambiguous)" '
    [ -n "$ERC1155" ] || return 1
    OUT=$(cast send $ERC1155 \
        "emitApprovalForAll(address,address,bool)" \
        0x9191919191919191919191919191919191919191 \
        0x9292929292929292929292929292929292929292 \
        false \
        --rpc-url $RPC --private-key $PK0 2>&1)
    TXHASH=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$TXHASH" ] || return 1
    $ETH_CLI decode-receipt $TXHASH > /dev/null
'

# ========================================
# SECTION 11: COMPLEX SCENARIO 2
# NFT Lifecycle (8 steps)
# ========================================

echo ""; echo "## Section 11: Complex Scenario 2 - NFT Lifecycle (8 steps)"

test_it "S2.1: Deploy SimpleNFT" '
    OUT=$(forge create config/SimpleNFT.sol:SimpleNFT \
        --rpc-url $RPC --private-key $PK0 --broadcast 2>&1)
    SIMPLE_NFT=$(echo "$OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    [ -n "$SIMPLE_NFT" ] || return 1
    echo "Contract: $SIMPLE_NFT"
'

test_it "S2.2: Mint NFT token ID 1" '
    [ -n "$SIMPLE_NFT" ] || return 1
    OUT=$(cast send $SIMPLE_NFT \
        "mint(address,uint256)" \
        $ADDR0 1 \
        --rpc-url $RPC --private-key $PK0 2>&1)
    MINT_TX=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$MINT_TX" ]
'

test_it "S2.3: Decode mint Transfer event" '
    [ -n "$MINT_TX" ] || return 1
    OUT=$($ETH_CLI decode-receipt $MINT_TX)
    echo "$OUT" | grep -q "Transfer" || return 1
    echo "$OUT" | grep -q "token_id=1" || return 1
    echo "$OUT" | grep -q "from=0x0000000000000000000000000000000000000000" || return 1
'

test_it "S2.4: Transfer NFT to account 1" '
    [ -n "$SIMPLE_NFT" ] || return 1
    OUT=$(cast send $SIMPLE_NFT \
        "transferFrom(address,address,uint256)" \
        $ADDR0 $ADDR1 1 \
        --rpc-url $RPC --private-key $PK0 2>&1)
    TRANSFER_TX=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$TRANSFER_TX" ]
'

test_it "S2.5: Decode transfer event" '
    [ -n "$TRANSFER_TX" ] || return 1
    OUT=$($ETH_CLI decode-receipt $TRANSFER_TX)
    echo "$OUT" | grep -q "Transfer" || return 1
    echo "$OUT" | grep -qi "$(echo $ADDR1 | tr A-F a-f)" || return 1
'

test_it "S2.6: Approve operator (account 2)" '
    [ -n "$SIMPLE_NFT" ] || return 1
    OUT=$(cast send $SIMPLE_NFT \
        "approve(address,uint256)" \
        $ADDR2 1 \
        --rpc-url $RPC --private-key $PK1 2>&1)
    APPROVE_TX=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$APPROVE_TX" ]
'

test_it "S2.7: Decode Approval event" '
    [ -n "$APPROVE_TX" ] || return 1
    OUT=$($ETH_CLI decode-receipt $APPROVE_TX)
    echo "$OUT" | grep -q "Approval" || return 1
    echo "$OUT" | grep -qi "$(echo $ADDR2 | tr A-F a-f)" || return 1
'

# ========================================
# SECTION 11: COMPLEX SCENARIO 3
# Multi-Contract Token Purchase (10 steps)
# ========================================

echo ""; echo "## Section 11: Complex Scenario 3 - Multi-Contract (10 steps)"

test_it "S3.1: Deploy ERC-20 StubToken" '
    OUT=$(forge create config/StubToken.sol:StubToken \
        --rpc-url $RPC --private-key $PK0 --broadcast 2>&1)
    TOKEN=$(echo "$OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    [ -n "$TOKEN" ] || return 1
    echo "Token: $TOKEN"
'

test_it "S3.2: Deploy ERC-721 SimpleNFT (2nd instance)" '
    OUT=$(forge create config/SimpleNFT.sol:SimpleNFT \
        --rpc-url $RPC --private-key $PK0 --broadcast 2>&1)
    NFT=$(echo "$OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    [ -n "$NFT" ] || return 1
    echo "NFT: $NFT"
'

test_it "S3.3: Mint 1M tokens to account 0" '
    [ -n "$TOKEN" ] || return 1
    cast send $TOKEN \
        "mint(address,uint256)" \
        $ADDR0 1000000000000000000000000 \
        --rpc-url $RPC --private-key $PK0 > /dev/null 2>&1
'

test_it "S3.4: Mint NFT token 1 to account 1 (seller)" '
    [ -n "$NFT" ] || return 1
    cast send $NFT \
        "mint(address,uint256)" \
        $ADDR1 1 \
        --rpc-url $RPC --private-key $PK0 > /dev/null 2>&1
'

test_it "S3.5: Buyer approves seller to spend 100 tokens" '
    [ -n "$TOKEN" ] || return 1
    OUT=$(cast send $TOKEN \
        "approve(address,uint256)" \
        $ADDR1 100000000000000000000 \
        --rpc-url $RPC --private-key $PK0 2>&1)
    APPROVE_TX=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$APPROVE_TX" ]
'

test_it "S3.6: Decode token approval (ERC-20, not NFT)" '
    [ -n "$APPROVE_TX" ] || return 1
    # ERC-20 Approval is not an NFT event, decoder will skip it
    # Just verify command does not crash
    $ETH_CLI decode-receipt $APPROVE_TX > /dev/null 2>&1
'

test_it "S3.7: Seller transfers 100 tokens from buyer" '
    [ -n "$TOKEN" ] || return 1
    OUT=$(cast send $TOKEN \
        "transferFrom(address,address,uint256)" \
        $ADDR0 $ADDR1 100000000000000000000 \
        --rpc-url $RPC --private-key $PK1 2>&1)
    TOKEN_TX=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$TOKEN_TX" ]
'

test_it "S3.8: Seller transfers NFT to buyer" '
    [ -n "$NFT" ] || return 1
    OUT=$(cast send $NFT \
        "transferFrom(address,address,uint256)" \
        $ADDR1 $ADDR0 1 \
        --rpc-url $RPC --private-key $PK1 2>&1)
    NFT_TX=$(echo "$OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$NFT_TX" ]
'

test_it "S3.9: Decode token transfer (ERC-20, not NFT)" '
    [ -n "$TOKEN_TX" ] || return 1
    # ERC-20 Transfer is not an NFT event
    $ETH_CLI decode-receipt $TOKEN_TX > /dev/null 2>&1
'

test_it "S3.10: Decode NFT transfer" '
    [ -n "$NFT_TX" ] || return 1
    OUT=$($ETH_CLI decode-receipt $NFT_TX)
    echo "$OUT" | grep -q "Transfer" || return 1
    echo "$OUT" | grep -q "token_id=1" || return 1
'

# ========================================
# ADDITIONAL: Chained Execution
# ========================================

echo ""; echo "## Chained Execution Pattern"

test_it "send → tx-status chain" '
    TX_OUT=$($ETH_CLI send --private-key $PK0 $ADDR1 500000000000000000)
    HASH=$(echo "$TX_OUT" | grep -oP "0x[0-9a-f]{64}" | head -1)
    [ -n "$HASH" ] || return 1
    STATUS=$($ETH_CLI tx-status $HASH)
    echo "$STATUS" | grep -q "success"
'

# ========================================
# SUMMARY
# ========================================

echo ""
echo "=========================================="
echo "FINAL SUMMARY"
echo "=========================================="
echo "PASSED: $PASS"
echo "FAILED: $FAIL"
echo "TOTAL:  $((PASS + FAIL))"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo "✅ ALL SCENARIOS PASSED"
    echo ""
    echo "Documentation validation complete:"
    echo "- Basic CLI commands work"
    echo "- Test contracts (TestERC721, TestERC1155) work"
    echo "- Complex Scenario 2 (NFT Lifecycle, 8 steps) works"
    echo "- Complex Scenario 3 (Multi-Contract, 10 steps) works"
    echo "- Chained execution patterns work"
    exit 0
else
    echo "⚠️  SOME SCENARIOS FAILED"
    echo "See failures above for details"
    exit 1
fi
