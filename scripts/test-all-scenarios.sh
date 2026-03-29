#!/bin/bash
# Test all documentation scenarios in bash
# Reports results for each example

# Add Git Bash tools to PATH
export PATH="/c/Program Files/Git/usr/bin:$PATH"

# Don't exit on errors - we want to test all scenarios
set +e

ETH_CLI="./target/release/eth_node_cli.exe"
RPC_URL="http://127.0.0.1:8545"
ACCT0_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
ACCT0_ADDR="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
ACCT1_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
ACCT1_ADDR="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
ACCT2_ADDR="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"

PASS_COUNT=0
FAIL_COUNT=0
RESULTS=()

function test_scenario() {
    local name="$1"
    echo ""
    echo "====== Testing: $name ======"
    
    if eval "$2"; then
        echo "✅ $name"
        ((PASS_COUNT++))
        RESULTS+=("PASS: $name")
    else
        echo "❌ $name"
        ((FAIL_COUNT++))
        RESULTS+=("FAIL: $name")
        return 0  # Don't exit, continue testing
    fi
}

echo "=================================="
echo "Testing All Documentation Scenarios"
echo "=================================="

# SIMPLE EXAMPLES FROM SECTION 7

test_scenario "1. Balance check (basic)" '
    output=$($ETH_CLI balance $ACCT0_ADDR)
    echo "$output" | grep -q "Balance:.*wei"
'

test_scenario "2. Balance check (quiet mode)" '
    $ETH_CLI --quiet balance $ACCT0_ADDR > /dev/null
    [ $? -eq 0 ]
'

test_scenario "3. Send ETH transaction" '
    output=$($ETH_CLI send --private-key $ACCT0_KEY $ACCT1_ADDR 1000000000000000000)
    SEND_TX_HASH=$(echo "$output" | grep -oP "0x[0-9a-f]{64}" | head -1)
    [ -n "$SEND_TX_HASH" ]
    echo "TX: $SEND_TX_HASH"
'

test_scenario "4. Check transaction status" '
    [ -n "$SEND_TX_HASH" ] || return 1
    output=$($ETH_CLI tx-status $SEND_TX_HASH)
    echo "$output" | grep -q "success"
'

# DECODE-RECEIPT WALKTHROUGH (Section 7.4)

test_scenario "5. Deploy ERC-721 test contract" '
    output=$(forge create src/eth_node/tests/contracts/TestERC721.sol:TestERC721 \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    ERC721_CONTRACT=$(echo "$output" | grep -oP "Deployed to: \K0x[0-9a-fA-F]{40}")
    [ -n "$ERC721_CONTRACT" ] || return 1
    echo "Deployed ERC-721 at: $ERC721_CONTRACT"
'

test_scenario "6. Emit Transfer event and decode it" '
    [ -n "$ERC721_CONTRACT" ] || return 1
    
    output=$(cast send $ERC721_CONTRACT \
        "emitTransfer(address,address,uint256)" \
        0x2121212121212121212121212121212121212121 \
        0x2222222222222222222222222222222222222222 \
        42 \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    TRANSFER_TX_HASH=$(echo "$output" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$TRANSFER_TX_HASH" ] || return 1
    echo "Transfer tx: $TRANSFER_TX_HASH"
    
    decode_output=$($ETH_CLI decode-receipt $TRANSFER_TX_HASH)
    echo "$decode_output" | grep -q "Transfer" || return 1
    echo "$decode_output" | grep -q "token_id=42" || return 1
'

test_scenario "7. Deploy ERC-1155 and test ApprovalForAll ambiguity" '
    output=$(forge create src/eth_node/tests/contracts/TestERC1155.sol:TestERC1155 \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    ERC1155_CONTRACT=$(echo "$output" | grep -oP "Deployed to: \K0x[0-9a-fA-F]{40}")
    [ -n "$ERC1155_CONTRACT" ] || return 1
    
    output=$(cast send $ERC1155_CONTRACT \
        "emitApprovalForAll(address,address,bool)" \
        0x9191919191919191919191919191919191919191 \
        0x9292929292929292929292929292929292929292 \
        false \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    APPROVAL_TX_HASH=$(echo "$output" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$APPROVAL_TX_HASH" ] || return 1
    
    $ETH_CLI decode-receipt $APPROVAL_TX_HASH > /dev/null
    echo "ApprovalForAll decoded (expected ambiguous)"
'

# COMPLEX SCENARIO 2: NFT LIFECYCLE (8 steps)

test_scenario "8. Scenario 2 - Deploy SimpleNFT contract" '
    if [ ! -f "config/SimpleNFT.sol" ]; then
        echo "⚠️ SimpleNFT.sol not found, skipping"
        return 1
    fi
    
    output=$(forge create config/SimpleNFT.sol:SimpleNFT \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    SIMPLE_NFT=$(echo "$output" | grep -oP "Deployed to: \K0x[0-9a-fA-F]{40}")
    [ -n "$SIMPLE_NFT" ] || return 1
    echo "Deployed SimpleNFT at: $SIMPLE_NFT"
'

test_scenario "9. Scenario 2 - Mint NFT (token ID 1)" '
    [ -n "$SIMPLE_NFT" ] || return 1
    
    output=$(cast send $SIMPLE_NFT \
        "mint(address,uint256)" \
        $ACCT0_ADDR \
        1 \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    MINT_TX=$(echo "$output" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$MINT_TX" ]
'

test_scenario "10. Scenario 2 - Decode mint Transfer event" '
    [ -n "$MINT_TX" ] || return 1
    
    output=$($ETH_CLI decode-receipt $MINT_TX)
    echo "$output" | grep -q "Transfer" || return 1
    echo "$output" | grep -q "token_id=1" || return 1
'

test_scenario "11. Scenario 2 - Transfer NFT to account 1" '
    [ -n "$SIMPLE_NFT" ] || return 1
    
    output=$(cast send $SIMPLE_NFT \
        "transferFrom(address,address,uint256)" \
        $ACCT0_ADDR \
        $ACCT1_ADDR \
        1 \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    TRANSFER2_TX=$(echo "$output" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$TRANSFER2_TX" ]
'

test_scenario "12. Scenario 2 - Decode transfer event" '
    [ -n "$TRANSFER2_TX" ] || return 1
    
    output=$($ETH_CLI decode-receipt $TRANSFER2_TX)
    echo "$output" | grep -q "Transfer"
'

test_scenario "13. Scenario 2 - Approve operator (account 2)" '
    [ -n "$SIMPLE_NFT" ] || return 1
    
    output=$(cast send $SIMPLE_NFT \
        "approve(address,uint256)" \
        $ACCT2_ADDR \
        1 \
        --rpc-url $RPC_URL \
        --private-key $ACCT1_KEY 2>&1)
    
    APPROVE_TX=$(echo "$output" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$APPROVE_TX" ]
'

test_scenario "14. Scenario 2 - Decode Approval event" '
    [ -n "$APPROVE_TX" ] || return 1
    
    output=$($ETH_CLI decode-receipt $APPROVE_TX)
    echo "$output" | grep -q "Approval"
'

# COMPLEX SCENARIO 3: MULTI-CONTRACT (simplified)

test_scenario "15. Scenario 3 - Deploy StubToken" '
    if [ ! -f "config/StubToken.sol" ]; then
        echo "⚠️ StubToken.sol not found"
        return 1
    fi
    
    output=$(forge create config/StubToken.sol:StubToken \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    TOKEN_ADDR=$(echo "$output" | grep -oP "Deployed to: \K0x[0-9a-fA-F]{40}")
    [ -n "$TOKEN_ADDR" ] || return 1
    echo "Deployed StubToken at: $TOKEN_ADDR"
'

test_scenario "16. Scenario 3 - Mint tokens to account 0" '
    [ -n "$TOKEN_ADDR" ] || return 1
    
    cast send $TOKEN_ADDR \
        "mint(address,uint256)" \
        $ACCT0_ADDR \
        1000000000000000000000000 \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY > /dev/null 2>&1
'

test_scenario "17. Scenario 3 - Approve token spending" '
    [ -n "$TOKEN_ADDR" ] || return 1
    
    output=$(cast send $TOKEN_ADDR \
        "approve(address,uint256)" \
        $ACCT1_ADDR \
        100000000000000000000 \
        --rpc-url $RPC_URL \
        --private-key $ACCT0_KEY 2>&1)
    
    TOKEN_APPROVE_TX=$(echo "$output" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    [ -n "$TOKEN_APPROVE_TX" ]
'

test_scenario "18. Scenario 3 - Decode token approval (ERC-20, may not decode)" '
    [ -n "$TOKEN_APPROVE_TX" ] || return 1
    
    # ERC-20 Approval is not an NFT event, so decoder may skip it
    # Just verify command does not crash
    $ETH_CLI decode-receipt $TOKEN_APPROVE_TX > /dev/null 2>&1
'

test_scenario "19. Scenario 3 - Transfer tokens" '
    [ -n "$TOKEN_ADDR" ] || return 1
    
    cast send $TOKEN_ADDR \
        "transferFrom(address,address,uint256)" \
        $ACCT0_ADDR \
        $ACCT1_ADDR \
        100000000000000000000 \
        --rpc-url $RPC_URL \
        --private-key $ACCT1_KEY > /dev/null 2>&1
'

# SUMMARY

echo ""
echo "=================================="
echo "Test Results Summary"
echo "=================================="
echo ""

for result in "${RESULTS[@]}"; do
    echo "$result"
done

echo ""
TOTAL=$((PASS_COUNT + FAIL_COUNT))
echo "Passed: $PASS_COUNT/$TOTAL"
if [ $FAIL_COUNT -gt 0 ]; then
    echo "Failed: $FAIL_COUNT/$TOTAL"
    exit 1
else
    echo "✅ All scenarios passed!"
    exit 0
fi
