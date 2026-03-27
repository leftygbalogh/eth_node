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
   - [call](#call--read-data-from-a-smart-contract)
   - [watch](#watch--stream-live-contract-events)
8. [Recording a session](#8-recording-a-session)
9. [Environment variables](#9-environment-variables)
10. [Troubleshooting](#10-troubleshooting)

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
contract as they happen, in real time. Press `Ctrl+C` to stop.

> Events are how contracts signal that something happened — a transfer, an
> approval, a vote. `watch` lets you observe these live.

**Syntax**:
```bash
eth watch <CONTRACT_ADDRESS> [EVENT_SIGNATURE]
```

**Example** — watch all events from the StubToken contract (no filter):

```bash
eth watch 0x5FbDB2315678afecb367f032d93F642f64180aa3
```

**Example** — filter to only "Transfer" events:

```bash
eth watch \
  0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  "Transfer(address,address,uint256)"
```

You can also pass a pre-computed topic hash:

```bash
eth watch \
  0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
```

**Expected output when running**:
```
2026-...: INFO eth_node_cli: watching contract contract=0x5FbD... topic0=None
Watching 0x5FbDB2315678afecb367f032d93F642f64180aa3 for events (Ctrl+C to stop)
```

Events will appear as they are emitted. On a quiet test network with no
transactions, you'll see no further output until something happens.

Stop with:
```
Ctrl+C
```

---

## 8. Recording a session

Use `scripts/capture-session.sh` to run any command and automatically save:
- `screen.log` — full terminal output
- `state.json` — the result as machine-readable JSON

### Setup (one time)

```bash
chmod +x scripts/capture-session.sh
```

### Usage

```bash
./scripts/capture-session.sh <subcommand> [args...]
```

All the same arguments as `eth`, just prefixed with the script path.

**Examples**:

```bash
# Record a balance check
./scripts/capture-session.sh balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

# Record a send
./scripts/capture-session.sh send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000000000000000

# Record a contract call
./scripts/capture-session.sh call \
  --abi-file /tmp/stubtoken.abi.json \
  0x5FbDB2315678afecb367f032d93F642f64180aa3 \
  balanceOf \
  0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

Artifacts are written to `output/sessions/<timestamp>/`:

```
output/sessions/
  2026-03-27_14-05-30/
    screen.log    ← everything printed to the terminal
    state.json    ← structured JSON result
```

### Manual recording with `tee`

If you prefer not to use the script, pipe output through `tee`:

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
