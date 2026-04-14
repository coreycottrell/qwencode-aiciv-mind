---
name: solana-token-operations
version: 1.0.0
author: skills-master
created: 2025-12-27
last_updated: 2025-12-27
line_count: 498
compliance_status: compliant

description: |
  Complete reference for Solana SPL token operations including balances, transfers,
  wallet management, and transaction ledger tracking. Use when checking token/SOL
  balances, transferring tokens, managing wallet registries, or recording transactions.
  Production-ready patterns verified on mainnet with ACGEE token.

applicable_agents:
  - sol-dev
  - primary
  - arcx-coder
  - coder

activation_trigger: |
  Load this skill when:
  - Checking SOL or SPL token balances
  - Transferring tokens between wallets
  - Creating or managing token accounts
  - Recording transactions to ledger
  - Managing wallet registry
  - Setting up Solana CLI environment
  - Converting keypair formats (JSON to Base58)

required_tools:
  - Read
  - Write
  - Edit
  - Bash

category: main

depends_on:
  - memory-first-protocol

related_skills:
  - security-analysis
  - verification-before-completion
---

# Solana Token Operations Skill

**Complete reference for SPL token operations: balances, transfers, wallet management, and ledger tracking.**

## Purpose

This skill enables agents to perform Solana blockchain operations safely and correctly. It documents production-verified patterns from A-C-Gee's ACGEE token operations including treasury management, inter-civilization transfers, and immutable transaction recording.

---

## When to Use

**Trigger scenarios:**
- Checking token or SOL balances
- Transferring tokens between wallets
- Creating token accounts for recipients
- Verifying transactions on-chain
- Managing wallet registries and transaction ledgers
- Setting up new tokens or mints

---

## Prerequisites

### CLI Tools Required

```bash
# Verify installations
solana --version   # solana-cli 2.x+
spl-token --version  # spl-token-cli 5.x+
```

### Environment Setup

```bash
# Check current config
solana config get

# Set network (mainnet-beta, devnet, testnet)
solana config set --url mainnet-beta
solana config set --url devnet  # For testing

# Set default keypair
solana config set --keypair /path/to/keypair.json
```

---

## Core Operations

### 1. Check Balances

```bash
# Check SOL balance (any wallet)
solana balance <WALLET_ADDRESS>

# Check SOL balance (default keypair)
solana balance

# Check SPL token balance (specific owner + mint)
spl-token balance <TOKEN_MINT> --owner <WALLET_ADDRESS>

# Check all token balances for a wallet
spl-token accounts --owner <WALLET_ADDRESS>
```

**Example (ACGEE):**
```bash
# Treasury SOL
solana balance 1v2P7wTw48YS81bD9rzymmpmuDAMbQAUFEb8L15eNFp

# Treasury ACGEE tokens
spl-token balance M8FVGq7rdEPRXa9vDcH2diknyMqkKT4ZpCWEi9h6F7k --owner 1v2P7wTw48YS81bD9rzymmpmuDAMbQAUFEb8L15eNFp
```

### 2. Transfer SOL

```bash
# Basic transfer (requires default keypair set)
solana transfer <RECIPIENT> <AMOUNT_SOL>

# Transfer with explicit keypair
solana transfer <RECIPIENT> <AMOUNT_SOL> --keypair /path/to/keypair.json

# Allow unfunded recipient (new wallet)
solana transfer <RECIPIENT> <AMOUNT_SOL> --allow-unfunded-recipient
```

**Example:**
```bash
solana transfer 8DHPzLjhm9k9549Qxy9MwpZSPA4E6jtXtsEYatHQX1Ei 0.1
```

### 3. Transfer SPL Tokens

```bash
# Basic token transfer
spl-token transfer <TOKEN_MINT> <AMOUNT> <RECIPIENT_WALLET>

# Fund recipient (creates token account if needed) - RECOMMENDED
spl-token transfer <TOKEN_MINT> <AMOUNT> <RECIPIENT_WALLET> --fund-recipient

# With explicit keypair
spl-token transfer <TOKEN_MINT> <AMOUNT> <RECIPIENT_WALLET> --fund-recipient --owner /path/to/keypair.json
```

**Example (ACGEE):**
```bash
spl-token transfer M8FVGq7rdEPRXa9vDcH2diknyMqkKT4ZpCWEi9h6F7k 10000000 8DHPzLjhm9k9549Qxy9MwpZSPA4E6jtXtsEYatHQX1Ei --fund-recipient
```

**Output includes:**
- Transaction signature
- Sender token account
- Recipient token account (created if --fund-recipient used)

### 4. Verify Keypair

```bash
# Verify keypair produces expected public key
solana-keygen verify <EXPECTED_PUBKEY> /path/to/keypair.json

# Display keypair public key
solana-keygen pubkey /path/to/keypair.json
```

### 5. View Transaction

```bash
# View transaction details
solana confirm <SIGNATURE> -v

# Or use Solscan (browser)
# https://solscan.io/tx/<SIGNATURE>
```

---

## Wallet & Ledger Management

### Wallet Registry Pattern

Store known wallets in structured JSON:

```json
{
  "schema_version": "1.0",
  "description": "Wallet registry - maps addresses to entities",
  "wallets": {
    "<WALLET_ADDRESS>": {
      "entity_name": "Entity Name",
      "entity_type": "treasury | civilization | individual | reserve",
      "registered_date": "2025-12-27T00:00:00Z",
      "notes": "Description of this wallet",
      "verified": true
    }
  }
}
```

**Location**: `memories/token/wallets.json`

### Transaction Ledger Pattern

Record all transactions immutably:

```json
{
  "schema_version": "1.0",
  "transactions": [
    {
      "tx_id": "TX-YYYYMMDD-NNNN",
      "timestamp": "2025-12-27T12:00:00Z",
      "type": "sol_transfer | token_transfer",
      "from_wallet": "<ADDRESS>",
      "from_entity": "Entity Name",
      "to_wallet": "<ADDRESS>",
      "to_entity": "Recipient Name",
      "amount": "10000000",
      "currency": "SOL | ACGEE",
      "token_mint": "<MINT_ADDRESS>",
      "status": "completed | pending | failed",
      "description": "Purpose of transfer",
      "significance": "historic | routine",
      "on_chain_signature": "<TX_SIGNATURE>",
      "recorded_by": "agent-name"
    }
  ],
  "summary": {
    "total_sol_received": "0.11",
    "total_sol_sent": "0",
    "total_tokens_distributed": "10000000",
    "transaction_count": 3
  }
}
```

**Location**: `memories/token/ledger.json`

### Allocation Tracking Pattern

Track token distribution commitments:

```json
{
  "allocations": {
    "ALLOC-001": {
      "recipient": "Entity Name",
      "recipient_wallet": "<ADDRESS>",
      "amount": "10000000",
      "percentage": "1.0",
      "status": "reserved | pending | distributed",
      "purpose": "Allocation purpose",
      "created_date": "2025-12-27T00:00:00Z",
      "distribution_date": "2025-12-27T12:00:00Z",
      "notes": "Additional context"
    }
  },
  "summary": {
    "distributed": "10000000",
    "pending": "0",
    "reserved": "90000000",
    "unallocated": "900000000"
  }
}
```

**Location**: `memories/token/allocations.json`

---

## Advanced Operations

### Keypair Format Conversion (JSON to Base58)

Many SDKs expect Base58 format. Convert using Python:

```python
from solders.keypair import Keypair
import json
import base58

# Read JSON keypair
with open('/path/to/keypair.json', 'r') as f:
    secret_key_bytes = json.load(f)

# Convert to Base58
keypair = Keypair.from_bytes(bytes(secret_key_bytes))
base58_key = base58.b58encode(bytes(keypair)).decode()

print(f"Base58: {base58_key}")
print(f"Pubkey: {keypair.pubkey()}")
```

### Token Account Management

```bash
# Get associated token account for a wallet
spl-token accounts --owner <WALLET_ADDRESS>

# Create token account (if needed)
spl-token create-account <TOKEN_MINT>
```

### Token Supply & Mint Info

```bash
# View token supply and mint details
spl-token supply <TOKEN_MINT>
spl-token display <TOKEN_MINT>
```

---

## Security Checklist

### DO
- Store keypairs in `/memories/secure/` (gitignored)
- Set file permissions: `chmod 600 keypair.json`
- Verify keypair before operations: `solana-keygen verify`
- Check balance before transfers
- Record all transactions in ledger
- Use `--fund-recipient` for new recipients

### DO NOT
- Commit keypair files to git
- Log private keys anywhere
- Store keys in plaintext in code
- Use mainnet keypair for testing
- Skip balance verification

---

## Transaction Workflow

### Standard Transfer Flow

1. **Verify sender balance**
   ```bash
   spl-token balance <MINT> --owner <SENDER>
   ```

2. **Execute transfer**
   ```bash
   spl-token transfer <MINT> <AMOUNT> <RECIPIENT> --fund-recipient
   ```

3. **Capture signature** from output

4. **Verify on-chain**
   ```bash
   solana confirm <SIGNATURE> -v
   ```

5. **Update ledger** (memories/token/ledger.json)

6. **Update allocations** if applicable

### Example Complete Transaction Record

```bash
# 1. Check balance
$ spl-token balance M8FVGq7rdEPRXa9vDcH2diknyMqkKT4ZpCWEi9h6F7k --owner 1v2P7wTw48YS81bD9rzymmpmuDAMbQAUFEb8L15eNFp
935020000

# 2. Transfer
$ spl-token transfer M8FVGq7rdEPRXa9vDcH2diknyMqkKT4ZpCWEi9h6F7k 10000000 8DHPzLjhm9k9549Qxy9MwpZSPA4E6jtXtsEYatHQX1Ei --fund-recipient
Transfer 10000000 tokens
  Sender: 2pVhehUowfM5nd21CQCRhUG4s1oyUK2MxSwNe9uymMDD
  Recipient: 3ayMkxLHkEkVbytQKQabRaANWG45AnL5TqtLsGPezxHg
Signature: 5P3fPPCZVZXqEoiz9WvQ7Ap1KQVgJz7qLhph1LsPujETVALnGVd2hLFNmY2ptUiLNku5Skp9HeHkFK9YVuYVqehc

# 3. Verify
$ solana confirm 5P3fPPCZVZXqEoiz9WvQ7Ap1KQVgJz7qLhph1LsPujETVALnGVd2hLFNmY2ptUiLNku5Skp9HeHkFK9YVuYVqehc -v
Confirmed
```

---

## Quick Reference Card

| Operation | Command |
|-----------|---------|
| SOL balance | `solana balance <ADDR>` |
| Token balance | `spl-token balance <MINT> --owner <ADDR>` |
| All tokens | `spl-token accounts --owner <ADDR>` |
| SOL transfer | `solana transfer <TO> <AMT>` |
| Token transfer | `spl-token transfer <MINT> <AMT> <TO> --fund-recipient` |
| Verify keypair | `solana-keygen verify <PUBKEY> <FILE>` |
| Get pubkey | `solana-keygen pubkey <FILE>` |
| Confirm tx | `solana confirm <SIG> -v` |
| Token supply | `spl-token supply <MINT>` |

---

## Network RPCs

| Network | URL |
|---------|-----|
| Mainnet | `https://api.mainnet-beta.solana.com` |
| Devnet | `https://api.devnet.solana.com` |
| Testnet | `https://api.testnet.solana.com` |

```bash
solana config set --url <URL>
```

---

## Troubleshooting

### "Error: Account not found"
- Recipient may not have a token account
- Use `--fund-recipient` flag

### "Insufficient funds"
- Check SOL balance (need ~0.002 for token account rent)
- Check token balance

### "Keypair not found"
- Verify path: `ls -la /path/to/keypair.json`
- Check permissions: `chmod 600`

### "Transaction failed"
- Check network congestion
- Retry after 30 seconds
- Consider priority fees during high load

---

## Anti-Patterns

### Anti-Pattern 1: Skipping Balance Verification
- **Wrong**: Execute transfer immediately without checking balance
- **Correct**: Always verify balance BEFORE attempting transfer to avoid failed transactions that still cost fees

### Anti-Pattern 2: Forgetting --fund-recipient
- **Wrong**: `spl-token transfer <MINT> <AMT> <RECIPIENT>` (fails if no token account)
- **Correct**: `spl-token transfer <MINT> <AMT> <RECIPIENT> --fund-recipient`

### Anti-Pattern 3: Not Recording Transactions
- **Wrong**: Complete transfer, move on to next task
- **Correct**: Record every transaction in `memories/token/ledger.json` with signature, timestamp, and parties

### Anti-Pattern 4: Using Mainnet Keypair for Testing
- **Wrong**: Test new code with real treasury keypair
- **Correct**: Use devnet with throwaway keypair, switch to mainnet only for production

### Anti-Pattern 5: Logging Private Keys
- **Wrong**: `console.log(privateKey)` or `print(base58_key)` in production
- **Correct**: Never log keys. Use environment variables. Verify via public key only.

---

## Success Indicators

You're using this skill correctly when:
- [ ] Balance checked before every transfer attempt
- [ ] Transaction signature captured and recorded in ledger
- [ ] On-chain confirmation verified (`solana confirm <SIG> -v`)
- [ ] Wallet registry updated for new recipients
- [ ] Allocations JSON updated for distribution tracking
- [ ] No private keys logged or committed to git
- [ ] File permissions set to 600 for keypair files

---

## Files and Paths (A-C-Gee Specific)

```
Treasury Keypair:    ${ACG_ROOT}/memories/secure/wallets/acgee-wallet.json
Treasury Address:    1v2P7wTw48YS81bD9rzymmpmuDAMbQAUFEb8L15eNFp
ACGEE Token Mint:    M8FVGq7rdEPRXa9vDcH2diknyMqkKT4ZpCWEi9h6F7k
Token Decimals:      9

Wallet Registry:     memories/token/wallets.json
Transaction Ledger:  memories/token/ledger.json
Allocations:         memories/token/allocations.json
```

---

## Related Skills

- `security-analysis/SKILL.md` - Static code analysis for Solana/Anchor patterns
- `verification-before-completion.md` - Verify transactions before claiming success
- `memory-first-protocol.md` - Search past token operations before starting new ones

---

*A-C-Gee Financial Infrastructure | Building sovereign AI civilization*
