# eth_node CLI Reference

> **Who this is for**: You're comfortable with a Linux/Bash terminal but new to Ethereum.
> Every concept is explained before it's used. Every example is copy-paste ready.

---

## Table of Contents

1. [What is this?](#1-what-is-this)
2. [Prerequisites](#2-prerequisites)
3. [Build the binary](#3-build-the-binary)
4. [Key concepts in 60 seconds](#4-key-concepts-in-60-seconds)
5. [Start a local test network (Anvil)](#5-start-a-local-test-network-anvil)
6. [Test accounts](#6-test-accounts)
7. [Commands](#7-commands)
   - [balance](#balance--check-an-account-balance)
   - [send](#send--transfer-eth)
   - [tx-status](#tx-status--check-what-happened-to-a-transaction)
  - [decode-receipt](#decode-receipt--decode-nft-events-from-a-transaction)
   - [call](#call--read-data-from-a-smart-contract)
   - [watch](#watch--stream-live-contract-events)
8. [Recording a session](#8-recording-a-session)
9. [Environment variables](#9-environment-variables)
10. [Troubleshooting](#10-troubleshooting)
11. [Complex Scenarios](#11-complex-scenarios)
   - [Scenario 1: Executor Pipeline](#scenario-1-executor-pipeline--deploy-simulate-compare-verify)
   - [Scenario 2: NFT Lifecycle](#scenario-2-nft-lifecycle--deploy-mint-transfer-approve-decode)
   - [Scenario 3: Multi-Contract](#scenario-3-multi-contract--token-purchase-with-cross-contract-events)

---

## 1. What is this?

`eth_node_cli` is a command-line tool that lets you interact with an Ethereum
network from your terminal. You can check balances, send funds, and talk to
smart contracts — all without a browser wallet or a GUI.

For learning and testing, it connects to **Anvil**, a fake Ethereum network that
runs entirely on your machine. Nothing you do there is real or costs money.

---

## 2. Prerequisites

You need three things installed. Run the check commands to confirm.

### Rust
```bash
rustc --version
# expected: rustc 1.xx.x (...)
```
Install from: https://rustup.rs

### Foundry (provides `anvil` and `cast` and `forge`)
```bash
anvil --version
# expected: anvil 1.x.x-...
```
Install:
```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### Git Bash (on Windows)
You already have this if you're reading this file in Git Bash.

---

## 3. Build the binary

From the repository root:

```bash
cargo build
```

This produces the binary at `target/debug/eth_node_cli`.

To avoid typing the full path, add a shell alias for your session:

```bash
alias eth='./target/debug/eth_node_cli'
```

Verify it works:

```bash
eth --help
```

Expected output (shortened):
```
Ethereum toolkit CLI (Phase 1)

Usage: eth_node_cli [OPTIONS] <COMMAND>

Commands:
  balance    Print the ETH balance of an address
  send       Send ETH to an address
  watch      Watch and print logs emitted by a contract
  call       Call a view function on a deployed contract
  tx-status  Print the receipt for a transaction
  decode-receipt  Decode NFT logs from a transaction receipt
  help       Print this message or the help of the given subcommand(s)
...
```

---

## 4. Key concepts in 60 seconds

| Concept | Plain English |
|---|---|
| **Address** | A unique ID for an account, like a bank account number. Always 42 characters starting with `0x`. |
| **ETH** | The currency of Ethereum. |
| **Wei** | The smallest unit of ETH. `1 ETH = 1,000,000,000,000,000,000 wei` (10^18). The CLI always works in wei. |
| **Transaction** | Any action that changes state: sending ETH, calling a contract function that writes data. |
| **Transaction hash** | A unique ID for a transaction, like a receipt number. 66 characters starting with `0x`. |
| **Block** | A batch of confirmed transactions. Each block has a number. Block 0 is the start. |
| **Smart contract** | A program with its own address, living on the chain. You call its functions to interact with it. |
| **ABI** | The "user manual" for a smart contract — a JSON file that describes what functions it has and what types they take. |
| **Anvil** | A fake Ethereum network for development. Starts instantly, mines transactions immediately, resets when you stop it. |

---

## 5. Start a local test network (Anvil)

Open a **separate terminal** and run:

```bash
anvil
```

Leave it running. You will see output like:

```
                             _   _
                            (_) | |
      __ _   _ __   __   __  _  | |
     / _` | | '_ \  \ \ / / | | | |
    | (_| | | | | |  \ V /  | | | |
     \__,_| |_| |_|   \_/   |_| |_|

    0.1.0 (...)

Available Accounts
==================
(0) 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (10000.000000000000000000 ETH)
(1) 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (10000.000000000000000000 ETH)
...

Listening on 127.0.0.1:8545
```

Anvil is now running on `http://127.0.0.1:8545`. All commands below connect to
this address by default.

> **Tip**: Everything resets when you stop Anvil. That's fine — it's just for
> testing.

---

## 6. Test accounts

Anvil always starts with the same 10 test accounts. These are not secrets —
they exist for development only. **Never use these keys on a real network.**

| # | Address | Private Key |
|---|---|---|
| 0 | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80` |
| 1 | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | `0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d` |

Account 0 is the default "sender" in examples below. It starts with 10,000 ETH.

---

## 7. Commands

All commands share these global flags:

```
--endpoint <URL>     RPC endpoint (default: http://127.0.0.1:8545)
--dump-state <PATH>  Write result as JSON to this file after success
--quiet              Suppress everything except errors
--log-level <LEVEL>  trace | debug | info | warn | error  (default: info)
```

`--dump-state` can be placed anywhere — before or after the subcommand.

---

### `balance` — Check an account balance

**What it does**: Queries the ETH balance of any address, in wei.

**Syntax**:
```bash
eth balance <ADDRESS>
```

**Example**:
```bash
eth balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

**Expected output**:
```
2026-03-27T...: INFO eth_node_cli: balance queried address=0xf39... wei=10000000000000000000000
Balance: 10000000000000000000000 wei
```

> `10000000000000000000000 wei` = `10000 ETH`.
> To convert: divide by `10^18` (move the decimal 18 places left).

**With state dump** (saves result as JSON):
```bash
eth --dump-state /tmp/balance.json balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
cat /tmp/balance.json
```

```json
{
  "address": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
  "balance_wei": "10000000000000000000000"
}
```

**Quiet mode** (just the balance line, no log noise):
```bash
eth --quiet balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

---

### `send` — Transfer ETH

**What it does**: Sends ETH from one account to another. You need the sender's
private key. The transaction is broadcast to the network; Anvil mines it
immediately.

**Syntax**:
```bash
eth send --private-key <KEY> <RECIPIENT_ADDRESS> <AMOUNT_IN_WEI>
```

**Example** — send 1 ETH (= 1000000000000000000 wei) from account 0 to account 1:

```bash
eth send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000000000000000
```

**Expected output**:
```
2026-...: INFO eth_node_cli: transaction sent hash=0x43aa... block=1
Transaction: 0x43aab2ff... in block 1
```

> The hash shown is your transaction ID. Copy it — you can use it with `tx-status`.

**Convert ETH amounts to wei quickly** (in bash):
```bash
# 0.5 ETH in wei
python3 -c "print(int(0.5 * 10**18))"
# 500000000000000000
```

**Using an environment variable for the private key** (recommended — keeps the
key out of your shell history):
```bash
export ETH_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

eth send \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000000000000000
```

---

### `tx-status` — Check what happened to a transaction

**What it does**: Looks up a transaction by its hash and tells you whether it
succeeded, reverted, or is still pending.

**Syntax**:
```bash
eth tx-status <TX_HASH>
```

**Example** — paste the hash from your `send` output:
```bash
eth tx-status 0x43aab2ff75b7c9b2a3a345de2e2de2ae41ea6ea8a62c8aa5bef4c3b7a24c7421
```

**Expected output** (success):
```
Transaction 0x43aa...: success in block 1
```

**Expected output** (pending — transaction hasn't been mined yet):
```
Transaction 0x43aa...: pending (not yet mined)
```

> On Anvil, transactions mine instantly, so you'll almost always see "success".
> On a real network, you might see "pending" for several seconds.

---

### `decode-receipt` — Decode NFT events from a transaction

**What it does**: Fetches a mined transaction receipt by hash, scans every log
in that receipt, and decodes any standard **ERC-721** or **ERC-1155** NFT
events it finds.

This is the easiest way to manually test the Phase 2 NFT decoder from the
terminal. You trigger an event on-chain, copy the transaction hash, then ask
`eth_node_cli` to decode the logs for you.

**Syntax**:
```bash
eth decode-receipt [--approval-for-all-as erc721|erc1155] <TX_HASH>
```

**Important note about `ApprovalForAll`**:

The event signature `ApprovalForAll(address,address,bool)` is shared by both
ERC-721 and ERC-1155. The log alone does **not** contain enough information to
know which standard the contract meant.

So by default:
- `eth decode-receipt` reports that event as **ambiguous**
- you can force an interpretation with `--approval-for-all-as erc721`
- or with `--approval-for-all-as erc1155`

---

#### Quick walkthrough — decode a live ERC-721 `Transfer`

This uses the repo's test emitter contract so you can exercise the real decoder
from a terminal, without running Rust tests.

**Terminal 1** — start Anvil if it is not already running:
```bash
anvil
```

**Terminal 2** — deploy the ERC-721 emitter contract from the repo:
```bash
forge create src/eth_node/tests/contracts/TestERC721.sol:TestERC721 \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

`forge create` prints something like:
```
Deployed to: 0x<ERC721_CONTRACT_ADDRESS>
Transaction Hash: 0x<DEPLOY_TX_HASH>
```

Copy the deployed contract address.

**Terminal 2** — emit a `Transfer` event using `cast send`:
```bash
cast send <ERC721_CONTRACT_ADDRESS> \
  "emitTransfer(address,address,uint256)" \
  0x2121212121212121212121212121212121212121 \
  0x2222222222222222222222222222222222222222 \
  42 \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

`cast send` prints a transaction hash. Copy it.

**Terminal 2** — decode the receipt:
```bash
eth decode-receipt 0x<TRANSFER_TX_HASH>
```

**Expected output**:
```
Transaction 0x<TRANSFER_TX_HASH>: success in block 2 (1) log(s)
[0] 0x<ERC721_CONTRACT_ADDRESS> ERC-721 Transfer from=0x2121... to=0x2222... token_id=42
```

**Machine-readable output**:
```bash
eth --porcelain decode-receipt 0x<TRANSFER_TX_HASH>
```

Example JSON shape:
```json
{
  "hash": "0x...",
  "status": "success",
  "block_number": 2,
  "logs": [
    {
      "index": 0,
      "address": "0x...",
      "topic0": "0xddf252ad...",
      "decode_status": "decoded",
      "standard": "erc721",
      "event_name": "Transfer",
      "fields": {
        "from": "0x2121212121212121212121212121212121212121",
        "to": "0x2222222222222222222222222222222222222222",
        "token_id": "42"
      }
    }
  ]
}
```

---

#### Shared `ApprovalForAll` example

Deploy the ERC-1155 emitter:
```bash
forge create src/eth_node/tests/contracts/TestERC1155.sol:TestERC1155 \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

Emit `ApprovalForAll`:
```bash
cast send <ERC1155_CONTRACT_ADDRESS> \
  "emitApprovalForAll(address,address,bool)" \
  0x9191919191919191919191919191919191919191 \
  0x9292929292929292929292929292929292929292 \
  false \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

Decode it without an override:
```bash
eth decode-receipt 0x<APPROVAL_TX_HASH>
```

Expected shape:
```
Transaction 0x<APPROVAL_TX_HASH>: success in block 3 (1) log(s)
[0] 0x<ERC1155_CONTRACT_ADDRESS> ApprovalForAll ambiguous subject=0x9191... operator=0x9292... approved=false (ERC-721 owner or ERC-1155 account)
```

Force ERC-1155 interpretation:
```bash
eth decode-receipt --approval-for-all-as erc1155 0x<APPROVAL_TX_HASH>
```

Expected shape:
```
Transaction 0x<APPROVAL_TX_HASH>: success in block 3 (1) log(s)
[0] 0x<ERC1155_CONTRACT_ADDRESS> ERC-1155 ApprovalForAll account=0x9191... operator=0x9292... approved=false
```

---

### `call` — Read data from a smart contract

**What it does**: Calls a "view" function on a deployed smart contract and
prints the return value. View functions only read data — they don't change
anything and cost nothing.

> **You need a deployed contract to use this command.** Follow the
> "Quick contract deployment" steps below to get one in about 2 minutes.

---

#### Step 1 — Deploy a test contract

Create a simple Solidity contract file:

```bash
cat > /tmp/StubToken.sol << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract StubToken {
    mapping(address => uint256) private _bal;

    // When deployed, give the deployer 1000 tokens
    constructor() {
        _bal[msg.sender] = 1000;
    }

    // Read someone's balance (this is the view function we will call)
    function balanceOf(address account) external view returns (uint256) {
        return _bal[account];
    }
}
EOF
```

Deploy it to Anvil:

```bash
forge create /tmp/StubToken.sol:StubToken \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

`forge create` will print something like:

```
[⠊] Compiling...
[⠘] Compiling 1 files with Solc ...
Deployer: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Deployed to: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Transaction Hash: 0x...
```

**Copy the `Deployed to` address** — you'll need it in Step 3.

---

#### Step 2 — Create the ABI file

An ABI file tells `eth_node_cli` how to talk to the contract. It describes the
functions and their parameter types. Save this file once:

```bash
cat > /tmp/stubtoken.abi.json << 'EOF'
[
  {
    "name": "balanceOf",
    "type": "function",
    "stateMutability": "view",
    "inputs":  [{ "name": "account", "type": "address" }],
    "outputs": [{ "name": "",        "type": "uint256"  }]
  }
]
EOF
```

---

#### Step 3 — Call the contract

Replace `<CONTRACT_ADDRESS>` with the address printed by `forge create`:

```bash
eth call \
  --abi-file /tmp/stubtoken.abi.json \
  <CONTRACT_ADDRESS> \
  balanceOf \
  0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

Full example with the address from Step 1:

```bash
eth call \
  --abi-file /tmp/stubtoken.abi.json \
  0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  balanceOf \
  0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

**Expected output**:
```
2026-...: INFO eth_node_cli: calling contract contract=0x5FbD... function=balanceOf args_count=1
Return: Uint(1000, 256)
```

> `Uint(1000, 256)` means the function returned the number `1000` as a 256-bit
> unsigned integer — exactly the 1000 tokens assigned at deployment.

**Argument types are detected automatically**:

| You type | `eth_node_cli` interprets it as |
|---|---|
| `0x` + 40 hex chars | Ethereum address |
| `0x` + other hex | Raw bytes |
| `true` or `false` | Boolean |
| A plain number like `42` | uint256 |
| Anything else | String |

---

### `watch` — Stream live contract events

**What it does**: Connects to the chain and prints events (logs) emitted by a
contract as they happen, in real time. Press `Ctrl-C` to stop.

> Events are how contracts signal that something happened — a transfer, an
> approval, a vote. `watch` lets you observe these live.

**Syntax**:
```bash
eth watch <CONTRACT_ADDRESS> [EVENT_SIGNATURE]
```

**Expected output when running**:
```
2026-...: INFO eth_node_cli: watching contract contract=0x5FbD... topic0=None
Watching contract 0x5FbDB2315678afecb367f032d93F642f64180aa3 for events (Ctrl-C to stop)...
```

When an event fires, a line like this appears for each one:
```
Event #1: tx=0x43aa... topics=2
Event #2: tx=0x7bc1... topics=2
```

On a quiet test network with nothing happening, you'll see no further output
until a transaction touches the contract.

Stop with:
```
Ctrl-C
```

---

#### Quick walkthrough — see events live in under 2 minutes

This deploys a minimal `Receiver` contract that fires an event on every ETH
deposit, then triggers it twice so you can see `Event #1` and `Event #2`
appear in the watcher.

**Terminal 1** — open Anvil (skip if already running):
```bash
anvil
```

**Terminal 2** — create and deploy the contract:
```bash
# Write Receiver.sol
cat > /tmp/Receiver.sol << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;
contract Receiver {
    event Received(address from, uint256 amount);
    receive() external payable {
        emit Received(msg.sender, msg.value);
    }
}
EOF

# Deploy it
forge create /tmp/Receiver.sol:Receiver \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

`forge create` prints:
```
Deployed to: 0x<CONTRACT_ADDRESS>
```

Copy that address.

**Terminal 2** — start watching (replace `<CONTRACT_ADDRESS>`):
```bash
eth watch <CONTRACT_ADDRESS>
```

Output:
```
Watching contract <CONTRACT_ADDRESS> for events (Ctrl-C to stop)...
```

**Terminal 3** — send two deposits (use any two distinct amounts):
```bash
# First deposit — 7919 wei (prime)
eth send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  <CONTRACT_ADDRESS> \
  7919

# Second deposit — 7907 wei (a different prime)
eth send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  <CONTRACT_ADDRESS> \
  7907
```

Back in **Terminal 2** you will see:
```
Event #1: tx=0x... topics=2
Event #2: tx=0x... topics=2
```

Press `Ctrl-C` to stop.

---

#### Filter to a specific event signature

```bash
eth watch \
  <CONTRACT_ADDRESS> \
  "Received(address,uint256)"
```

You can also pass a pre-computed topic-0 hash:

```bash
eth watch \
  <CONTRACT_ADDRESS> \
  0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
```

---

## 8. Recording a session

Two scripts wrap `eth_node_cli` and save everything automatically:

| Script | When to use |
|---|---|
| `scripts/capture-session.sh` | One command at a time |
| `scripts/capture-multi.sh` | Chain several commands into one session |

Both scripts:
- Start Anvil automatically if it isn't already running
- Copy `config/*.abi.json` fixtures to `/tmp/` so `--abi-file /tmp/...` examples work immediately
- Save a `screen.log` (full terminal transcript) to `output/sessions/<timestamp>/`
- Save one `state.json` per successful command

Behavior difference:
- `capture-session.sh` keeps Anvil running after the command finishes (so you can chain follow-up commands)
- `capture-session.ps1` now does the same on Windows PowerShell
- `capture-multi.sh` still runs as one bounded session and stops the temporary Anvil it started

---

### Single command — `capture-session.sh`

```bash
chmod +x scripts/capture-session.sh    # one-time setup

./scripts/capture-session.sh <subcommand> [args...]
```

If the script starts Anvil for you, it now leaves Anvil running after the
command exits. That means your next command can reuse the same chain state.

On Windows PowerShell, use the sibling helper with the same behavior:

```powershell
.\scripts\capture-session.ps1 balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

All six commands work:

```bash
# Check a balance
./scripts/capture-session.sh balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

# Send ETH
./scripts/capture-session.sh send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000000000000000

# Check transaction status
# Replace 0x43aa... with the transaction hash printed by the `send` command above.
./scripts/capture-session.sh tx-status 0x43aab2ff75b7c9b2a3a345de2e2de2ae41ea6ea8a62c8aa5bef4c3b7a24c7421

# Decode NFT logs from a transaction receipt
./scripts/capture-session.sh decode-receipt 0x43aab2ff75b7c9b2a3a345de2e2de2ae41ea6ea8a62c8aa5bef4c3b7a24c7421

# Force the shared ApprovalForAll signature to decode as ERC-1155
./scripts/capture-session.sh decode-receipt --approval-for-all-as erc1155 0x43aab2ff75b7c9b2a3a345de2e2de2ae41ea6ea8a62c8aa5bef4c3b7a24c7421

# Call a contract (ABI file auto-provisioned from config/)
#
# PREREQUISITE: The contract must already be deployed at the address below.
# The address 0x5FbDB... is deterministic ONLY when StubToken is the first
# contract deployed by account 0 on a FRESH Anvil session (no prior txs).
# If Anvil has had prior transactions (e.g. the `send` example above ran),
# restart Anvil and first run the `forge create` step from §7 Step 1.
./scripts/capture-session.sh call \
  --abi-file /tmp/stubtoken.abi.json \
  0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  balanceOf \
  0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

# Watch for events (press Ctrl-C to stop; session log captures everything up to that point)
#
# PREREQUISITE: Use a contract that emits events — the Receiver contract from
# §7 "Quick walkthrough" is ideal. StubToken emits Transfer events only when
# token balances change, not on plain ETH sends.
# Replace 0x67d2... with the address forge create printed for YOUR Receiver.
./scripts/capture-session.sh watch 0x67d269191c92Caf3cD7723F116c85e6E9bf55933
```

#### Chained execution example (send -> tx-status -> decode-receipt)

This keeps everything in one shell and reuses the transaction hash from `send`
without manual copy/paste.

```bash
SEND_OUT="$(./scripts/capture-session.sh send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000000000000000)"

TX_HASH="$(printf '%s\n' "$SEND_OUT" | grep -Eo '0x[a-fA-F0-9]{64}' | head -n 1)"

if [[ -z "$TX_HASH" ]]; then
  echo "Could not extract transaction hash from send output."
  exit 1
fi

echo "Using tx hash: $TX_HASH"

./scripts/capture-session.sh tx-status "$TX_HASH"
./scripts/capture-session.sh decode-receipt "$TX_HASH"
```

Expected behavior:
- `tx-status` reports the same transaction as `success` (not pending)
- `decode-receipt` inspects logs from that exact transaction receipt

#### Stop the managed Anvil instance

If `capture-session.sh` or `capture-session.ps1` started Anvil for you, you can
stop that managed instance later with one of these commands:

```bash
./scripts/capture-session.sh --stop-anvil
```

```powershell
.\scripts\capture-session.ps1 --stop-anvil
```

Important scope note:
- these commands stop only the Anvil process that the helper script started and tracked
- if you started `anvil` manually in another terminal, stop it there with `Ctrl-C`

Artifacts are written to `output/sessions/<timestamp>/`:

```
output/sessions/
  2026-03-27_14-05-30/
    screen.log    ← everything printed to the terminal
    state.json    ← structured JSON result (only on exit 0)
```

---

### Chaining commands — `capture-multi.sh`

Run a sequence of commands in one shared session. The script stops after the
first failure and tells you which step failed.

```bash
chmod +x scripts/capture-multi.sh    # one-time setup

./scripts/capture-multi.sh <commands-file>
```

**Commands file format** — one subcommand per non-blank, non-comment line:

```
# comments and blank lines are ignored
balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
balance 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
send --private-key 0xac09...80 0x7099...C8 1000000000000000000
balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
balance 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

**Worked example** — a ready-made scenario is included:

```bash
./scripts/capture-multi.sh scripts/examples/scenario-send-and-check.txt
```

This runs: balance account 0 → balance account 1 → send 1 ETH → balance account 0 → balance account 1.

Sample output:

```
=== Multi-step session ===
Session artifacts: output/sessions/2026-03-27_10-31-06
Binary:            target/release/eth_node_cli

Anvil already running on 127.0.0.1:8545.

--- Running 5 step(s) ---

--- Step 1/5: balance 0xf39Fd6... ---
Balance: 10000000000000000000000 wei
(state saved: state-1.json)

--- Step 2/5: balance 0x70997... ---
Balance: 10000000000000000000000 wei
(state saved: state-2.json)

--- Step 3/5: send ... ---
Transaction success: 0x5bb3... in block 1
(state saved: state-3.json)

--- Step 4/5: balance 0xf39Fd6... ---
Balance: 8999790000000000000000 wei    ← account 0 lost ~1 ETH + gas
(state saved: state-4.json)

--- Step 5/5: balance 0x70997... ---
Balance: 11000000000000000000000 wei   ← account 1 gained 1 ETH
(state saved: state-5.json)

=== All 5 step(s) completed successfully ===
```

Each step gets its own `state-N.json`. If step 3 had failed, steps 4 and 5
would not have run, and the footer would say:

```
=== ABORTED at step 3/5 ===
Command   : send ...
Exit code : 1
```

---

### Suppress log noise — `--quiet` and `--porcelain`

By default, every command prints structured JSON log lines to `stderr`.
Two flags let you control this:

| Flag | Effect |
|---|---|
| `--quiet` | Suppress INFO logs; only errors appear on stderr |
| `--porcelain` | No logs at all; stdout is exactly the JSON result, nothing else |

**`--quiet` — clean terminal output**:

```bash
eth --quiet balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

Output:
```
Balance: 10000000000000000000000 wei
```

**Redirect stderr yourself** — suppress all log lines without changing the binary flag:

```bash
eth balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 2>/dev/null
```

Output:
```
Balance: 10000000000000000000000 wei
```

**`--porcelain` — JSON only, nothing else** (ideal for scripting):

```bash
eth --porcelain balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

Output (stdout only, no labels, no logs):
```json
{
  "address": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
  "balance_wei": "10000000000000000000000"
}
```

Pipe it to `jq` for field extraction (requires `jq` installed):

```bash
eth --porcelain balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 | jq .balance_wei
# "10000000000000000000000"
```

On Windows without `jq`, use PowerShell:

```powershell
& eth --porcelain balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 |
  ConvertFrom-Json | Select-Object -ExpandProperty balance_wei
# 10000000000000000000000
```

`--porcelain` works with all five commands. The JSON shapes are:

| Command | JSON keys |
|---|---|
| `balance` | `address`, `balance_wei` |
| `send` | `transaction_hash`, `block_number`, `status` |
| `tx-status` | `hash`, `status`, `block_number` |
| `decode-receipt` | `hash`, `status`, `block_number`, `logs` |
| `call` | `contract`, `function`, `return_values` |
| `watch` | `contract`, `events_received` |

---

### Manual recording with `tee`

If you prefer not to use the scripts, pipe output through `tee`:

```bash
mkdir -p output/sessions/my-session

eth --dump-state output/sessions/my-session/state.json \
  balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
  2>&1 | tee output/sessions/my-session/screen.log
```


---

## 9. Environment variables

| Variable | What it does | Example |
|---|---|---|
| `ETH_ENDPOINT` | Default RPC endpoint (replaces `--endpoint`) | `export ETH_ENDPOINT=http://127.0.0.1:8545` |
| `ETH_PRIVATE_KEY` | Default private key for `send` (replaces `--private-key`) | `export ETH_PRIVATE_KEY=0xac09...` |

Set them for the session:
```bash
export ETH_ENDPOINT=http://127.0.0.1:8545
export ETH_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

Or put them in a local `.envrc` file (do **not** commit this with real keys):
```bash
cat > .envrc << 'EOF'
export ETH_ENDPOINT=http://127.0.0.1:8545
export ETH_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
EOF
source .envrc
```

---

## 10. Troubleshooting

### `error: Connection refused` or `transport error`
Anvil is not running. Open a new terminal and run `anvil`.

### `exit code: 2` with a usage error
A required argument is missing or a flag name is wrong. Run with `--help`:
```bash
eth <subcommand> --help
# example
eth call --help
```

### `invalid address: ...`
The address is malformed. Make sure it starts with `0x` and is exactly 42 characters long (0x + 40 hex digits). Addresses are case-insensitive.

### `invalid amount: ...`
Amounts must be in wei (whole numbers, no decimals). `1.5` is not valid; use `1500000000000000000` for 1.5 ETH.

### `decode-receipt` says `pending`
The transaction exists as a hash, but the receipt is not available yet. On
Anvil this is uncommon because blocks mine immediately. On other networks,
wait a few seconds and try again.

### `decode-receipt` says `unsupported`
The receipt log was real, but it was not one of the NFT events this decoder
currently understands. Right now the command targets standard ERC-721 and
ERC-1155 events only.

### `decode-receipt` says `ambiguous`
This happens for `ApprovalForAll(address,address,bool)` because ERC-721 and
ERC-1155 use the same event signature and data layout. Re-run with one of:

```bash
eth decode-receipt --approval-for-all-as erc721 0x<TX_HASH>
eth decode-receipt --approval-for-all-as erc1155 0x<TX_HASH>
```

### `one of --abi or --abi-file is required`
The `call` command needs an ABI. Pass either:
- `--abi-file /path/to/abi.json` (recommended)
- `--abi '[{"name":...}]'` (inline JSON — may have quoting issues in some shells)

### `cannot read --abi-file ...`
The path you gave to `--abi-file` does not exist or is not readable. Check:
```bash
ls -la /tmp/stubtoken.abi.json
```

### Balance shows 0 after a send
Check you're querying the right address. Also confirm you sent to account 1's
address, not account 0's.

### `cargo build` fails
Run `rustup update stable` to make sure you're on a recent stable toolchain,
then try again.

---

## 11. Complex Scenarios

These multi-step workflows demonstrate how to combine CLI commands and library APIs for real-world use cases. Each scenario is capped at ≤10 steps to stay focused.

### Scenario 1: Executor Pipeline — Deploy, Simulate, Compare, Verify

**Goal:** Deploy a contract to Anvil, simulate a transaction locally, compare simulation to Anvil execution, and verify consistency.

**Steps:** (7 steps)

1. **Start Anvil** in a separate terminal:
   ```bash
   anvil
   ```

2. **Deploy a simple contract** (e.g., StubToken) to Anvil:
   ```bash
   forge create --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
     config/StubToken.sol:StubToken
   ```
   *Note the deployed contract address (e.g., `0x5FbDB2315678afecb367f032d93F642f64180aa3`).*

3. **Build a transfer transaction** (in Rust code or CLI):
   ```rust
   use eth_node::executor::{simulate_tx, compare_to_anvil, SimulationContext};
   use alloy_rpc_types::TransactionRequest;
   use alloy_primitives::{Address, U256, Bytes};

   let tx = TransactionRequest {
       from: Some(Address::from([0xf3, 0x9f, /* ... account 0 */])),
       to: Some(contract_address.into()),
       data: Some(transfer_calldata), // Encoded transfer(recipient, amount)
       gas: Some(100_000),
       gas_price: Some(U256::from(10)),
       ..Default::default()
   };
   ```

4. **Simulate the transaction locally**:
   ```rust
   let context = SimulationContext {
       block_number: 1,
       timestamp: 1710000000,
       base_fee_per_gas: Some(10),
       gas_limit: 30_000_000,
   };

   let result = simulate_tx(&tx, &context)?;
   println!("Local gas used: {}", result.gas_used);
   println!("Success: {}", result.success);
   ```

5. **Compare local simulation to Anvil execution**:
   ```rust
   let report = compare_to_anvil(&tx, "http://127.0.0.1:8545", &context).await?;

   println!("Gas delta: {}", report.gas_delta);
   println!("Return data match: {}", report.return_data_match);
   ```

6. **Verify gas tolerance** (5% threshold):
   ```rust
   let threshold = (report.gas_used_anvil as f64 * 0.05).ceil() as u64;
   if (report.gas_delta).unsigned_abs() > threshold {
       eprintln!("⚠️ Gas mismatch exceeds 5%");
       for diff in &report.differences {
           eprintln!("  - {}", diff);
       }
   } else {
       println!("✅ Gas within 5% tolerance");
   }
   ```

7. **Assert consistency**:
   ```rust
   assert!(report.return_data_match, "Return data must match");
   assert!(report.logs_match, "Logs must match");
   println!("✅ Simulation verified against Anvil");
   ```

**Validation:** Build as integration test in `tests/executor_pipeline_scenario.rs` to automate validation.

---

### Scenario 2: NFT Lifecycle — Deploy, Mint, Transfer, Approve, Decode

**Goal:** Deploy an ERC-721 contract, mint a token, transfer it, approve an operator, and decode all emitted events.

**Steps:** (8 steps)

1. **Start Anvil**:
   ```bash
   anvil
   ```

2. **Deploy ERC-721 contract** (e.g., using forge or custom Rust deployment):
   ```bash
   forge create --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
     config/SimpleNFT.sol:SimpleNFT
   ```
   *Save the contract address.*

3. **Mint an NFT** (token ID 1):
   ```bash
   cast send <CONTRACT_ADDRESS> \
     "mint(address,uint256)" \
     0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 1 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   ```
   *Save the transaction hash.*

4. **Decode the Transfer event**:
   ```bash
   eth decode-receipt <TX_HASH>
   ```
   Expected output:
   ```
   Event: Transfer
     from: 0x0000000000000000000000000000000000000000
     to:   0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
     tokenId: 1
   ```

5. **Transfer the NFT** to account 1:
   ```bash
   cast send <CONTRACT_ADDRESS> \
     "transferFrom(address,address,uint256)" \
     0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
     0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
     1 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   ```
   *Save the transaction hash.*

6. **Decode the Transfer event**:
   ```bash
   eth decode-receipt <TX_HASH>
   ```
   Expected output:
   ```
   Event: Transfer
     from: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
     to:   0x70997970C51812dc3A010C7d01b50e0d17dc79C8
     tokenId: 1
   ```

7. **Approve an operator** (account 2) to manage account 1's tokens:
   ```bash
   # Need account 1's private key for this
   cast send <CONTRACT_ADDRESS> \
     "approve(address,uint256)" \
     0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC \
     1 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
   ```
   *Save the transaction hash.*

8. **Decode the Approval event**:
   ```bash
   eth decode-receipt <TX_HASH>
   ```
   Expected output:
   ```
   Event: Approval
     owner: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
     approved: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
     tokenId: 1
   ```

**Validation:** All events decode successfully; ownership chain (mint → transfer → approve) is consistent.

---

### Scenario 3: Multi-Contract — Token Purchase with Cross-Contract Events

**Goal:** Deploy an ERC-20 token and an NFT marketplace, approve token spending, purchase an NFT, and decode cross-contract events.

**Steps:** (10 steps)

1. **Start Anvil**:
   ```bash
   anvil
   ```

2. **Deploy ERC-20 token contract**:
   ```bash
   forge create --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
     config/StubToken.sol:StubToken
   ```
   *Save as `TOKEN_ADDRESS`.*

3. **Deploy ERC-721 NFT contract**:
   ```bash
   forge create --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
     config/SimpleNFT.sol:SimpleNFT
   ```
   *Save as `NFT_ADDRESS`.*

4. **Mint test tokens** to account 0 (1 million tokens):
   ```bash
   cast send <TOKEN_ADDRESS> \
     "mint(address,uint256)" \
     0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
     1000000000000000000000000 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   ```

5. **Mint NFT** (token ID 1) to marketplace owner (account 1):
   ```bash
   cast send <NFT_ADDRESS> \
     "mint(address,uint256)" \
     0x70997970C51812dc3A010C7d01b50e0d17dc79C8 1 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   ```

6. **Approve token spending** (buyer approves marketplace to spend 100 tokens):
   ```bash
   cast send <TOKEN_ADDRESS> \
     "approve(address,uint256)" \
     0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
     100000000000000000000 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
   ```
   *Save transaction hash; decode to verify `Approval` event.*

7. **Simulate purchase transaction** (buyer calls `transferFrom` on token, seller transfers NFT):
   For this step, you'd need a marketplace contract with a `purchase` function. Simplified here as two calls:
   
   **Transfer tokens from buyer to seller**:
   ```bash
   cast send <TOKEN_ADDRESS> \
     "transferFrom(address,address,uint256)" \
     0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
     0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
     100000000000000000000 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
   ```
   *Save transaction hash.*

8. **Transfer NFT from seller to buyer**:
   ```bash
   cast send <NFT_ADDRESS> \
     "transferFrom(address,address,uint256)" \
     0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
     0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
     1 \
     --rpc-url http://127.0.0.1:8545 \
     --private-key 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
   ```
   *Save transaction hash.*

9. **Decode token Transfer event**:
   ```bash
   eth decode-receipt <TOKEN_TX_HASH>
   ```
   Expected: `Transfer` event (100 tokens from buyer to seller).

10. **Decode NFT Transfer event**:
    ```bash
    eth decode-receipt <NFT_TX_HASH>
    ```
    Expected: `Transfer` event (token ID 1 from seller to buyer).

**Validation:** Cross-contract event consistency—buyer paid 100 tokens, received NFT; seller received 100 tokens, transferred NFT.

**Note:** For production scenarios, deploy a marketplace contract that atomically handles both transfers in a single transaction (prevents partial execution).

---

### Scenario Validation

All scenarios above can be automated as integration tests:
- **Scenario 1:** `tests/executor_pipeline_scenario.rs`
- **Scenario 2:** `tests/nft_lifecycle_scenario.rs`
- **Scenario 3:** `tests/multi_contract_scenario.rs`

Run all scenario tests:
```bash
cargo test --test '*_scenario' -- --nocapture
```

Each test should:
1. Spin up a temporary Anvil instance
2. Execute all scenario steps programmatically
3. Assert expected outcomes (gas tolerance, event consistency, state changes)
4. Clean up (kill Anvil) on completion

See [`docs/LIBRARY_API_GUIDE.md`](docs/LIBRARY_API_GUIDE.md) for detailed library API usage within these scenarios.
