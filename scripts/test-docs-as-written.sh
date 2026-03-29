#!/bin/bash
# Test documentation examples EXACTLY as they appear in CLI_REFERENCE.md
# Automates only the manual copy-paste steps (saving addresses/hashes)

export PATH="/c/Program Files/Git/usr/bin:$PATH"
set +e

# Use 'eth' function for CLI to match documentation (aliases don't work in scripts)
ETH_CLI="./target/release/eth_node_cli.exe"
eth() {
    "$ETH_CLI" "$@"
}

PASS=0
FAIL=0
ISSUE_LOG="documentation_issues.txt"
> "$ISSUE_LOG"  # Clear previous issues

log_issue() {
    echo "❌ ISSUE: $1" | tee -a "$ISSUE_LOG"
}

test_scenario() {
    local name="$1"
    local cmd="$2"
    echo ""; echo "====== $name ======"
    if eval "$cmd" 2>&1; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "$name failed"
    fi
}

echo "=========================================="
echo "Documentation Examples - As Written Test"
echo "=========================================="
echo ""
echo "Testing if documentation examples work exactly as shown..."
echo ""

# ========================================
# BASIC CLI EXAMPLES (Section 7.1-7.3)
# ========================================

echo "## Basic CLI Examples"

# Example: eth balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
test_scenario "balance query" \
    'eth balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 | grep -q "Balance:"'

# Example: eth --quiet balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
test_scenario "balance (quiet mode)" \
    'eth --quiet balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 > /dev/null'

# Example: eth send --private-key ... 0x70997... 1000000000000000000
echo ""; echo "====== send ETH ======"
SEND_OUT=$(eth send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000000000000000 2>&1)

if echo "$SEND_OUT" | grep -q "Transaction success:"; then
    echo "✅ PASS"; ((PASS++))
    # User would copy this hash manually from output
    TX_HASH=$(echo "$SEND_OUT" | grep -oP "0x[0-9a-f]{64}" | head -1)
    echo "Captured TX_HASH (user would copy): $TX_HASH"
else
    echo "❌ FAIL"; ((FAIL++))
    log_issue "send ETH - expected 'Transaction success:' in output"
fi

# Example: eth tx-status <TX_HASH>
if [ -n "$TX_HASH" ]; then
    test_scenario "tx-status" \
        'eth tx-status '"$TX_HASH"' | grep -q "success"'
else
    echo "⚠️  SKIP tx-status (no TX_HASH)"
fi

# ========================================
# DECODE-RECEIPT WALKTHROUGH (Section 7.4)
# ========================================

echo ""; echo "## decode-receipt Walkthrough"

# Example: forge create src/eth_node/tests/contracts/TestERC721.sol:TestERC721 \
#   --rpc-url http://127.0.0.1:8545 \
#   --private-key 0xac09... \
#   --broadcast

echo ""; echo "====== Deploy TestERC721 ======"
FORGE_OUT=$(forge create src/eth_node/tests/contracts/TestERC721.sol:TestERC721 \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast 2>&1)

if echo "$FORGE_OUT" | grep -q "Deployed to:"; then
    echo "✅ PASS"; ((PASS++))
    ERC721_CONTRACT_ADDRESS=$(echo "$FORGE_OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    echo "Deployed (user would copy): $ERC721_CONTRACT_ADDRESS"
else
    echo "❌ FAIL"; ((FAIL++))
    log_issue "Deploy TestERC721 - forge create might be missing --broadcast flag in docs"
fi

# Example: cast send <ERC721_CONTRACT_ADDRESS> "emitTransfer..." ...
if [ -n "$ERC721_CONTRACT_ADDRESS" ]; then
    echo ""; echo "====== Emit Transfer ======"
    CAST_OUT=$(cast send "$ERC721_CONTRACT_ADDRESS" \
      "emitTransfer(address,address,uint256)" \
      0x2121212121212121212121212121212121212121 \
      0x2222222222222222222222222222222222222222 \
      42 \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 2>&1)
    
    if echo "$CAST_OUT" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        TRANSFER_TX_HASH=$(echo "$CAST_OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
        echo "Transfer TX (user would copy): $TRANSFER_TX_HASH"
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Emit Transfer failed"
    fi
fi

# Example: eth decode-receipt <TRANSFER_TX_HASH>
if [ -n "$TRANSFER_TX_HASH" ]; then
    echo ""; echo "====== Decode Transfer ======"
    DECODE_OUT=$(eth decode-receipt "$TRANSFER_TX_HASH" 2>&1)
    
    if echo "$DECODE_OUT" | grep -q "Transfer" && echo "$DECODE_OUT" | grep -q "token_id=42"; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "decode-receipt for Transfer - expected token_id=42"
    fi
fi

# Deploy ERC1155
echo ""; echo "====== Deploy TestERC1155 ======"
FORGE1155_OUT=$(forge create src/eth_node/tests/contracts/TestERC1155.sol:TestERC1155 \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast 2>&1)

if echo "$FORGE1155_OUT" | grep -q "Deployed to:"; then
    echo "✅ PASS"; ((PASS++))
    ERC1155_CONTRACT_ADDRESS=$(echo "$FORGE1155_OUT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    echo "Deployed (user would copy): $ERC1155_CONTRACT_ADDRESS"
else
    echo "❌ FAIL"; ((FAIL++))
    log_issue "Deploy TestERC1155 failed"
fi

# Emit ApprovalForAll
if [ -n "$ERC1155_CONTRACT_ADDRESS" ]; then
    echo ""; echo "====== Emit ApprovalForAll ======"
    CAST1155_OUT=$(cast send "$ERC1155_CONTRACT_ADDRESS" \
      "emitApprovalForAll(address,address,bool)" \
      0x9191919191919191919191919191919191919191 \
      0x9292929292929292929292929292929292929292 \
      false \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 2>&1)
    
    if echo "$CAST1155_OUT" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        APPROVAL_TX_HASH=$(echo "$CAST1155_OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
        echo "Approval TX (user would copy): $APPROVAL_TX_HASH"
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Emit ApprovalForAll failed"
    fi
fi

# Decode ambiguous ApprovalForAll
if [ -n "$APPROVAL_TX_HASH" ]; then
    echo ""; echo "====== Decode ApprovalForAll (ambiguous) ======"
    DECODE_APPROVAL=$(eth decode-receipt "$APPROVAL_TX_HASH" 2>&1)
    
    if echo "$DECODE_APPROVAL" | grep -q "ambiguous"; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "decode-receipt should show ApprovalForAll as ambiguous"
    fi
fi

# ========================================
# SCENARIO 2: NFT Lifecycle (Section 11)
# ========================================

echo ""; echo "## Scenario 2: NFT Lifecycle (8 steps)"

# Step 2: Deploy ERC-721 contract
echo ""; echo "====== S2.2: Deploy SimpleNFT ======"
FORGE_NFT=$(forge create --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast \
  config/SimpleNFT.sol:SimpleNFT 2>&1)

if echo "$FORGE_NFT" | grep -q "Deployed to:"; then
    echo "✅ PASS"; ((PASS++))
    CONTRACT_ADDRESS=$(echo "$FORGE_NFT" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    echo "Contract address (user would save): $CONTRACT_ADDRESS"
else
    echo "❌ FAIL"; ((FAIL++))
    log_issue "Scenario 2 Step 2: Deploy SimpleNFT failed"
fi

# Step 3: Mint an NFT
if [ -n "$CONTRACT_ADDRESS" ]; then
    echo ""; echo "====== S2.3: Mint NFT token ID 1 ======"
    MINT_OUT=$(cast send "$CONTRACT_ADDRESS" \
      "mint(address,uint256)" \
      0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 1 \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 2>&1)
    
    if echo "$MINT_OUT" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        TX_HASH=$(echo "$MINT_OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
        echo "TX hash (user would save): $TX_HASH"
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 2 Step 3: Mint failed"
    fi
fi

# Step 4: Decode the Transfer event
if [ -n "$TX_HASH" ]; then
    echo ""; echo "====== S2.4: Decode Transfer event ======"
    DECODE_OUT=$(eth decode-receipt "$TX_HASH" 2>&1)
    
    if echo "$DECODE_OUT" | grep -q "Transfer" && \
       echo "$DECODE_OUT" | grep -q "token_id=1"; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 2 Step 4: Expected Transfer event with token_id=1"
    fi
fi

# Step 5: Transfer the NFT to account 1
if [ -n "$CONTRACT_ADDRESS" ]; then
    echo ""; echo "====== S2.5: Transfer NFT to account 1 ======"
    TRANSFER_OUT=$(cast send "$CONTRACT_ADDRESS" \
      "transferFrom(address,address,uint256)" \
      0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
      0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
      1 \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 2>&1)
    
    if echo "$TRANSFER_OUT" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        TX_HASH=$(echo "$TRANSFER_OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
        echo "TX hash (user would save): $TX_HASH"
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 2 Step 5: Transfer failed"
    fi
fi

# Step 6: Decode the Transfer event
if [ -n "$TX_HASH" ]; then
    echo ""; echo "====== S2.6: Decode Transfer event ======"
    DECODE_OUT=$(eth decode-receipt "$TX_HASH" 2>&1)
    
    if echo "$DECODE_OUT" | grep -qi "Transfer" && \
       echo "$DECODE_OUT" | grep -qi "70997970C51812dc3A010C7d01b50e0d17dc79C8"; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 2 Step 6: Expected Transfer to account 1"
    fi
fi

# Step 7: Approve an operator (account 2)
if [ -n "$CONTRACT_ADDRESS" ]; then
    echo ""; echo "====== S2.7: Approve operator (account 2) ======"
    APPROVE_OUT=$(cast send "$CONTRACT_ADDRESS" \
      "approve(address,uint256)" \
      0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC \
      1 \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d 2>&1)
    
    if echo "$APPROVE_OUT" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        TX_HASH=$(echo "$APPROVE_OUT" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
        echo "TX hash (user would save): $TX_HASH"
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 2 Step 7: Approve failed"
    fi
fi

# Step 8: Decode the Approval event
if [ -n "$TX_HASH" ]; then
    echo ""; echo "====== S2.8: Decode Approval event ======"
    DECODE_OUT=$(eth decode-receipt "$TX_HASH" 2>&1)
    
    if echo "$DECODE_OUT" | grep -qi "Approval" && \
       echo "$DECODE_OUT" | grep -qi "3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 2 Step 8: Expected Approval for account 2"
    fi
fi

# ========================================
# SCENARIO 3: Multi-Contract (Section 11)
# ========================================

echo ""; echo "## Scenario 3: Multi-Contract (10 steps)"

# Step 2: Deploy ERC-20 token contract
echo ""; echo "====== S3.2: Deploy StubToken ======"
TOKEN_DEPLOY=$(forge create --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast \
  config/StubToken.sol:StubToken 2>&1)

if echo "$TOKEN_DEPLOY" | grep -q "Deployed to:"; then
    echo "✅ PASS"; ((PASS++))
    TOKEN_ADDRESS=$(echo "$TOKEN_DEPLOY" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    echo "Token address (user would save): $TOKEN_ADDRESS"
else
    echo "❌ FAIL"; ((FAIL++))
    log_issue "Scenario 3 Step 2: Deploy StubToken failed"
fi

# Step 3: Deploy ERC-721 NFT contract
echo ""; echo "====== S3.3: Deploy SimpleNFT ======"
NFT_DEPLOY=$(forge create --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast \
  config/SimpleNFT.sol:SimpleNFT 2>&1)

if echo "$NFT_DEPLOY" | grep -q "Deployed to:"; then
    echo "✅ PASS"; ((PASS++))
    NFT_ADDRESS=$(echo "$NFT_DEPLOY" | grep -oP "Deployed to:\s*\K0x[0-9a-fA-F]{40}")
    echo "NFT address (user would save): $NFT_ADDRESS"
else
    echo "❌ FAIL"; ((FAIL++))
    log_issue "Scenario 3 Step 3: Deploy SimpleNFT failed"
fi

# Step 4: Mint test tokens (1 million)
if [ -n "$TOKEN_ADDRESS" ]; then
    test_scenario "S3.4: Mint 1M tokens" \
        'cast send '"$TOKEN_ADDRESS"' \
          "mint(address,uint256)" \
          0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
          1000000000000000000000000 \
          --rpc-url http://127.0.0.1:8545 \
          --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null 2>&1'
fi

# Step 5: Mint NFT (token ID 1) to marketplace owner (account 1)
if [ -n "$NFT_ADDRESS" ]; then
    test_scenario "S3.5: Mint NFT to seller" \
        'cast send '"$NFT_ADDRESS"' \
          "mint(address,uint256)" \
          0x70997970C51812dc3A010C7d01b50e0d17dc79C8 1 \
          --rpc-url http://127.0.0.1:8545 \
          --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null 2>&1'
fi

# Step 6: Approve token spending
if [ -n "$TOKEN_ADDRESS" ]; then
    echo ""; echo "====== S3.6: Approve token spending ======"
    APPROVE_TOKEN=$(cast send "$TOKEN_ADDRESS" \
      "approve(address,uint256)" \
      0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
      100000000000000000000 \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 2>&1)
    
    if echo "$APPROVE_TOKEN" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        TOKEN_APPROVAL_TX=$(echo "$APPROVE_TOKEN" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 3 Step 6: Token approval failed"
    fi
fi

# Step 7: Transfer tokens from buyer to seller
if [ -n "$TOKEN_ADDRESS" ]; then
    echo ""; echo "====== S3.7: Transfer tokens ======"
    TOKEN_TRANSFER=$(cast send "$TOKEN_ADDRESS" \
      "transferFrom(address,address,uint256)" \
      0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
      0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
      100000000000000000000 \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d 2>&1)
    
    if echo "$TOKEN_TRANSFER" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        TOKEN_TX_HASH=$(echo "$TOKEN_TRANSFER" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 3 Step 7: Token transfer failed"
    fi
fi

# Step 8: Transfer NFT from seller to buyer
if [ -n "$NFT_ADDRESS" ]; then
    echo ""; echo "====== S3.8: Transfer NFT ======"
    NFT_TRANSFER=$(cast send "$NFT_ADDRESS" \
      "transferFrom(address,address,uint256)" \
      0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
      0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
      1 \
      --rpc-url http://127.0.0.1:8545 \
      --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d 2>&1)
    
    if echo "$NFT_TRANSFER" | grep -q "transactionHash"; then
        echo "✅ PASS"; ((PASS++))
        NFT_TX_HASH=$(echo "$NFT_TRANSFER" | grep -oP "transactionHash\s+\K0x[0-9a-f]{64}")
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 3 Step 8: NFT transfer failed"
    fi
fi

# Step 9: Decode token Transfer event
if [ -n "$TOKEN_TX_HASH" ]; then
    test_scenario "S3.9: Decode token transfer" \
        'eth decode-receipt '"$TOKEN_TX_HASH"' > /dev/null 2>&1'
fi

# Step 10: Decode NFT Transfer event
if [ -n "$NFT_TX_HASH" ]; then
    echo ""; echo "====== S3.10: Decode NFT transfer ======"
    DECODE_NFT=$(eth decode-receipt "$NFT_TX_HASH" 2>&1)
    
    if echo "$DECODE_NFT" | grep -qi "Transfer"; then
        echo "✅ PASS"; ((PASS++))
    else
        echo "❌ FAIL"; ((FAIL++))
        log_issue "Scenario 3 Step 10: Expected NFT Transfer event"
    fi
fi

# ========================================
# FINAL REPORT
# ========================================

echo ""
echo "=========================================="
echo "RESULTS"
echo "=========================================="
echo "PASSED: $PASS"
echo "FAILED: $FAIL"
echo "TOTAL:  $((PASS + FAIL))"
echo ""

if [ -s "$ISSUE_LOG" ]; then
    echo "⚠️  DOCUMENTATION ISSUES FOUND:"
    echo "=========================================="
    cat "$ISSUE_LOG"
    echo ""
    echo "Common issue: forge create commands in docs may be missing --broadcast flag"
    echo "Without --broadcast, deployments are dry-run only."
else
    echo "✅ NO ISSUES - Documentation examples work as written!"
fi

exit 0
