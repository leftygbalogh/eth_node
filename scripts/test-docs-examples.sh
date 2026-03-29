#!/bin/bash
# Comprehensive documentation scenario test - bash version
# Tests all examples that can actually run with existing contracts

export PATH="/c/Program Files/Git/usr/bin:$PATH"
set +e

ETH_CLI="./target/release/eth_node_cli.exe"
RPC="http://127.0.0.1:8545"
PK0="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
ADDR0="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
ADDR1="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

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

echo "==== Documentation Examples Test Report ===="
echo ""

# Section 7.1: balance command
test_it "balance (basic)" '$ETH_CLI balance $ADDR0 | grep -q "Balance:"'
test_it "balance (quiet)" '$ETH_CLI --quiet balance $ADDR0 > /dev/null'

# Section 7.2: send command  
test_it "send ETH"  '
    OUT=$($ETH_CLI send --private-key $PK0 $ADDR1 1000000000000000000)
    TX=$(echo "$OUT" | grep -oP "0x[0-9a-f]{64}" | head -1)
    [ -n "$TX" ]
'

# Section 7.3: tx-status command
test_it "tx-status" '
    [ -n "$TX" ] || return 1
    $ETH_CLI tx-status $TX | grep -q "success"
'

# Section 7.4: decode-receipt walkthrough - Deploy TestERC721
test_it "deploy TestERC721" '
    OUT=$(forge create src/eth_node/tests/contracts/TestERC721.sol:TestERC721 \
        --rpc-url $RPC --private-key $PK0 --broadcast 2>&1)
    ERC721=$(echo "$OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    [ -n "$ERC721" ]
    echo "Contract: $ERC721"
'

# Section 7.4: emit and decode Transfer event
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

# Section 7.4: Deploy TestERC1155  
test_it "deploy TestERC1155" '
    OUT=$(forge create src/eth_node/tests/contracts/TestERC1155.sol:TestERC1155 \
        --rpc-url $RPC --private-key $PK0 --broadcast 2>&1)
    ERC1155=$(echo "$OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    [ -n "$ERC1155" ]
    echo "Contract: $ERC1155"
'

# Section 7.4: emit and decode ApprovalForAll (ambiguous)
test_it "emit+decode ApprovalForAll" '
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

# Test chained execution (bash style from docs)
test_it "chained execution (send→status→decode)" '
    TX_OUT=$($ETH_CLI send --private-key $PK0 $ADDR1 500000000000000000)
    HASH=$(echo "$TX_OUT" | grep -oP "0x[0-9a-f]{64}" | head -1)
    [ -n "$HASH" ] || return 1
    STATUS=$($ETH_CLI tx-status $HASH)
    echo "$STATUS" | grep -q "success"
'

echo ""
echo "===================================="
echo "SUMMARY"
echo "===================================="
echo "PASSED: $PASS"
echo "FAILED: $FAIL"
echo "TOTAL:  $((PASS + FAIL))"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo "✅ All tested scenarios work correctly"
    exit 0
else
    echo "⚠️  Some scenarios failed"
    exit 1
fi
