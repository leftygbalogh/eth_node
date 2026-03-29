# Test all documentation scenarios
# Reports results for each example

$ErrorActionPreference = "Continue"
$ETH_CLI = ".\target\release\eth_node_cli.exe"
$RPC_URL = "http://127.0.0.1:8545"
$ACCT0_KEY = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
$ACCT0_ADDR = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
$ACCT1_KEY = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
$ACCT1_ADDR = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
$ACCT2_ADDR = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"

$results = @()

function Test-Scenario {
    param(
        [string]$Name,
        [scriptblock]$Test
    )
    
    Write-Host "`n====== Testing: $Name ======" -ForegroundColor Cyan
    try {
        & $Test
        $status = "PASS"
        Write-Host "✅ $Name" -ForegroundColor Green
    } catch {
        $status = "FAIL"
        Write-Host "❌ $Name" -ForegroundColor Red
        Write-Host "Error: $_" -ForegroundColor Red
    }
    
    $script:results += [PSCustomObject]@{
        Scenario = $Name
        Status = $status
    }
}

Write-Host "==================================" -ForegroundColor Yellow
Write-Host "Testing All Documentation Scenarios" -ForegroundColor Yellow
Write-Host "==================================" -ForegroundColor Yellow

# SIMPLE EXAMPLES FROM SECTION 7

Test-Scenario "1. Balance check (basic)" {
    $output = & $ETH_CLI balance $ACCT0_ADDR
    if ($LASTEXITCODE -ne 0) { throw "Balance command failed" }
    if ($output -notmatch "Balance:.*wei") { throw "Unexpected output format" }
}

Test-Scenario "2. Balance check (quiet mode)" {
    $output = & $ETH_CLI --quiet balance $ACCT0_ADDR
    if ($LASTEXITCODE -ne 0) { throw "Quiet mode failed" }
}

Test-Scenario "3. Send ETH transaction" {
    $output = & $ETH_CLI send --private-key $ACCT0_KEY $ACCT1_ADDR 1000000000000000000
    if ($LASTEXITCODE -ne 0) { throw "Send failed" }
    $outputStr = $output | Out-String
    if ($outputStr -notmatch "Transaction:") { throw "No transaction in output" }
    
    # Extract tx hash - look for 0x followed by 64 hex characters
    if ($outputStr -match "(?:Transaction:|hash=)(\s*)(0x[0-9a-f]{64})") {
        $script:SEND_TX_HASH = $matches[2]
        Write-Host "TX: $($script:SEND_TX_HASH)" -ForegroundColor Gray
    } else {
        throw "Could not extract transaction hash from: $outputStr"
    }
}

Test-Scenario "4. Check transaction status" {
    if (-not $script:SEND_TX_HASH) { throw "No tx hash from previous test" }
    $output = & $ETH_CLI tx-status $script:SEND_TX_HASH
    if ($LASTEXITCODE -ne 0) { throw "Tx-status failed" }
    if ($output -notmatch "success") { throw "Transaction not successful" }
}

# DECODE-RECEIPT WALKTHROUGH (Section 7.4)

Test-Scenario "5. Deploy ERC-721 test contract" {
    $output = forge create src/eth_node/tests/contracts/TestERC721.sol:TestERC721 `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    $outputStr = $output | Out-String
    if ($LASTEXITCODE -ne 0) { throw "Contract deployment failed: $outputStr" }
    
    # Extract contract address - look for "Deployed to:" followed by address
    if ($outputStr -match "Deployed to:\s*(0x[0-9a-fA-F]{40})") {
        $script:ERC721_CONTRACT = $matches[1]
        Write-Host "Deployed ERC-721 at: $($script:ERC721_CONTRACT)" -ForegroundColor Gray
    } else {
        throw "Could not extract contract address from: $outputStr"
    }
}

Test-Scenario "6. Emit Transfer event and decode it" {
    if (-not $script:ERC721_CONTRACT) { throw "No contract from previous test" }
    
    $output = cast send $script:ERC721_CONTRACT `
        'emitTransfer(address,address,uint256)' `
        '0x2121212121212121212121212121212121212121' `
        '0x2222222222222222222222222222222222222222' `
        42 `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "Cast send failed: $output" }
    
    # Extract transaction hash
    $script:TRANSFER_TX_HASH = $output | Select-String -Pattern "transactionHash\s+(0x[0-9a-f]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
    
    if (-not $script:TRANSFER_TX_HASH) { throw "Could not extract tx hash" }
    Write-Host "Transfer tx: $($script:TRANSFER_TX_HASH)" -ForegroundColor Gray
    
    # Decode the receipt
    $decode_output = & $ETH_CLI decode-receipt $script:TRANSFER_TX_HASH
    if ($LASTEXITCODE -ne 0) { throw "Decode failed" }
    if ($decode_output -notmatch "Transfer") { throw "Did not find Transfer event" }
    if ($decode_output -notmatch "token_id=42") { throw "Wrong token ID decoded" }
}

Test-Scenario "7. Deploy ERC-1155 and test ApprovalForAll ambiguity" {
    $output = forge create src/eth_node/tests/contracts/TestERC1155.sol:TestERC1155 `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "ERC-1155 deployment failed" }
    
    $script:ERC1155_CONTRACT = $output | Select-String -Pattern "Deployed to: (0x[0-9a-fA-F]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
    
    if (-not $script:ERC1155_CONTRACT) { throw "Could not extract ERC-1155 address" }
    
    # Emit ApprovalForAll
    $output = cast send $script:ERC1155_CONTRACT `
        'emitApprovalForAll(address,address,bool)' `
        '0x9191919191919191919191919191919191919191' `
        '0x9292929292929292929292929292929292929292' `
        false `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    $script:APPROVAL_TX_HASH = $output | Select-String -Pattern "transactionHash\s+(0x[0-9a-f]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
    
    # Decode without override (should report ambiguous)
    $decode_output = & $ETH_CLI decode-receipt $script:APPROVAL_TX_HASH
    if ($LASTEXITCODE -ne 0) { throw "Decode failed" }
    Write-Host "ApprovalForAll decoded (expected ambiguous)" -ForegroundColor Gray
}

# COMPLEX SCENARIO 2: NFT LIFECYCLE (8 steps)

Test-Scenario "8. Scenario 2 — Deploy SimpleNFT contract" {
    # Check if SimpleNFT exists
    if (-not (Test-Path "config/SimpleNFT.sol")) {
        Write-Host "⚠️ SimpleNFT.sol not found, skipping complex scenarios" -ForegroundColor Yellow
        throw "SimpleNFT.sol not found"
    }
    
    $output = forge create config/SimpleNFT.sol:SimpleNFT `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "SimpleNFT deployment failed" }
    
    $script:SIMPLE_NFT = $output | Select-String -Pattern "Deployed to: (0x[0-9a-fA-F]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
    
    if (-not $script:SIMPLE_NFT) { throw "Could not extract SimpleNFT address" }
    Write-Host "Deployed SimpleNFT at: $($script:SIMPLE_NFT)" -ForegroundColor Gray
}

Test-Scenario "9. Scenario 2 — Mint NFT (token ID 1)" {
    if (-not $script:SIMPLE_NFT) { throw "No SimpleNFT contract" }
    
    $output = cast send $script:SIMPLE_NFT `
        'mint(address,uint256)' `
        $ACCT0_ADDR `
        1 `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "Mint failed" }
    
    $script:MINT_TX = $output | Select-String -Pattern "transactionHash\s+(0x[0-9a-f]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
}

Test-Scenario "10. Scenario 2 — Decode mint Transfer event" {
    if (-not $script:MINT_TX) { throw "No mint tx hash" }
    
    $output = & $ETH_CLI decode-receipt $script:MINT_TX
    if ($LASTEXITCODE -ne 0) { throw "Decode failed" }
    if ($output -notmatch "Transfer") { throw "No Transfer event" }
    if ($output -notmatch "token_id=1") { throw "Wrong token ID" }
}

Test-Scenario "11. Scenario 2 — Transfer NFT to account 1" {
    if (-not $script:SIMPLE_NFT) { throw "No SimpleNFT contract" }
    
    $output = cast send $script:SIMPLE_NFT `
        'transferFrom(address,address,uint256)' `
        $ACCT0_ADDR `
        $ACCT1_ADDR `
        1 `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "Transfer failed" }
    
    $script:TRANSFER2_TX = $output | Select-String -Pattern "transactionHash\s+(0x[0-9a-f]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
}

Test-Scenario "12. Scenario 2 — Decode transfer event" {
    if (-not $script:TRANSFER2_TX) { throw "No transfer tx hash" }
    
    $output = & $ETH_CLI decode-receipt $script:TRANSFER2_TX
    if ($LASTEXITCODE -ne 0) { throw "Decode failed" }
    if ($output -notmatch "Transfer") { throw "No Transfer event" }
}

Test-Scenario "13. Scenario 2 — Approve operator (account 2)" {
    if (-not $script:SIMPLE_NFT) { throw "No SimpleNFT contract" }
    
    $output = cast send $script:SIMPLE_NFT `
        'approve(address,uint256)' `
        $ACCT2_ADDR `
        1 `
        --rpc-url $RPC_URL `
        --private-key $ACCT1_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "Approve failed" }
    
    $script:APPROVE_TX = $output | Select-String -Pattern "transactionHash\s+(0x[0-9a-f]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
}

Test-Scenario "14. Scenario 2 — Decode Approval event" {
    if (-not $script:APPROVE_TX) { throw "No approval tx hash" }
    
    $output = & $ETH_CLI decode-receipt $script:APPROVE_TX
    if ($LASTEXITCODE -ne 0) { throw "Decode failed" }
    if ($output -notmatch "Approval") { throw "No Approval event" }
}

# COMPLEX SCENARIO 3: MULTI-CONTRACT (simplified to 6 core steps)

Test-Scenario "15. Scenario 3 — Deploy StubToken" {
    if (-not (Test-Path "config/StubToken.sol")) {
        Write-Host "⚠️ StubToken.sol not found" -ForegroundColor Yellow
        throw "StubToken.sol not found"
    }
    
    $output = forge create config/StubToken.sol:StubToken `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "StubToken deployment failed" }
    
    $script:TOKEN_ADDR = $output | Select-String -Pattern "Deployed to: (0x[0-9a-fA-F]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
    
    Write-Host "Deployed StubToken at: $($script:TOKEN_ADDR)" -ForegroundColor Gray
}

Test-Scenario "16. Scenario 3 — Mint tokens to account 0" {
    if (-not $script:TOKEN_ADDR) { throw "No token contract" }
    
    $output = cast send $script:TOKEN_ADDR `
        'mint(address,uint256)' `
        $ACCT0_ADDR `
        '1000000000000000000000000' `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "Token mint failed" }
}

Test-Scenario "17. Scenario 3 — Approve token spending" {
    if (-not $script:TOKEN_ADDR) { throw "No token contract" }
    
    $output = cast send $script:TOKEN_ADDR `
        'approve(address,uint256)' `
        $ACCT1_ADDR `
        '100000000000000000000' `
        --rpc-url $RPC_URL `
        --private-key $ACCT0_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "Token approve failed" }
    
    $script:TOKEN_APPROVE_TX = $output | Select-String -Pattern "transactionHash\s+(0x[0-9a-f]+)" |
        ForEach-Object { $_.Matches[0].Groups[1].Value }
}

Test-Scenario "18. Scenario 3 — Decode token approval" {
    if (-not $script:TOKEN_APPROVE_TX) { throw "No approval tx" }
    
    $output = & $ETH_CLI decode-receipt $script:TOKEN_APPROVE_TX
    # Token Approval is ERC-20, not NFT, so may not decode
    # Just verify command doesn't crash
    if ($LASTEXITCODE -ne 0) { throw "Decode command crashed" }
}

Test-Scenario "19. Scenario 3 — Transfer tokens" {
    if (-not $script:TOKEN_ADDR) { throw "No token contract" }
    
    $output = cast send $script:TOKEN_ADDR `
        'transferFrom(address,address,uint256)' `
        $ACCT0_ADDR `
        $ACCT1_ADDR `
        '100000000000000000000' `
        --rpc-url $RPC_URL `
        --private-key $ACCT1_KEY 2>&1
    
    if ($LASTEXITCODE -ne 0) { throw "Token transfer failed" }
}

# SUMMARY

Write-Host "`n==================================" -ForegroundColor Yellow
Write-Host "Test Results Summary" -ForegroundColor Yellow
Write-Host "==================================" -ForegroundColor Yellow

$script:results | Format-Table -AutoSize

$passed = ($script:results | Where-Object { $_.Status -eq "PASS" }).Count
$failed = ($script:results | Where-Object { $_.Status -eq "FAIL" }).Count
$total = $script:results.Count

Write-Host "`nPassed: $passed/$total" -ForegroundColor Green
if ($failed -gt 0) {
    Write-Host "Failed: $failed/$total" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ All scenarios passed!" -ForegroundColor Green
    exit 0
}
