# Anchor Escrow 2025

## 🆕 Updated for Anchor 0.32.1, Solana Kit, Kite, and Codama

## 🆕 Now includes Rust/LiteSVM tests - check out `programs/escrow/src/tests.rs`

[![CI Badge](https://github.com/mikemaccana/anchor-escrow-2025/actions/workflows/tests.yaml/badge.svg)](https://github.com/mikemaccana/anchor-escrow-2025/actions)

**Start here for your first real Solana program / smart contract** (Solana generally uses the word 'program', older blockchains use 'smart contract'). As the saying goes, "All Solana programs are variations of an escrow."

This makes **Anchor Escrow 2025** the perfect starting point for anyone diving into Solana program development with a practical, real-world application.

**Anchor Escrow 2025** provides:

- Full compatibility with the latest Rust, Agave CLI, Node.js, Anchor, and Solana Kit.
- Clean builds with zero warnings or errors.
- Testing via npm and Node.js, avoiding third-party package managers or test runners.

## Animated walk through

Check out the **full animated explanation** of the Escrow program (also called Swap) from this video from QuickNode:

[![Full animated explanation of the Escrow program](https://img.youtube.com/vi/B5eBWWQfQuM/maxresdefault.jpg)](https://www.youtube.com/watch?v=B5eBWWQfQuM)

This repository is [designed for teaching and learning](CHANGELOG.md).

## Introduction

This Solana program implements an **escrow**, enabling secure token swaps between users. For example, Alice can offer 10 USDC in exchange for 100 WIF.

Without an escrow, users face significant risks:

- **Traditional finance** charges 1-6% in fees, eating into your funds.
- **Manual swaps** are prone to fraud. If Bob takes Alice's 10 USDC but doesn't send the 100 WIF, or if Alice fails to deliver after receiving Bob's tokens, someone gets burned.

The **Anchor Escrow 2025** program acts as a trusted intermediary, releasing tokens only when both parties meet the agreed terms. This ensures Alice and Bob each receive 100% of their desired tokens, securely and without middleman fees.

### New: Native SOL Duel Escrow (Head-to-Head Games)

In addition to SPL token swaps, this repo now includes a native SOL (lamports) duel escrow designed for real-time head-to-head games (e.g., math duels):

- Both players deposit the same SOL stake into a PDA-owned game account.
- A lightweight Game state PDA tracks players, stake, deposits, status, and expiry.
- Once your game backend/referee determines the winner, it finalizes on-chain and the winner receives the full pot (both stakes).
- On timeout, deposits are refundable back to players.

Program entrypoints (see `programs/escrow/src/lib.rs`):

- `create_game(id, player_a, player_b, stake_lamports, expiry_ts)`
- `deposit(amount)` — called by either player to deposit exactly `stake_lamports`.
- `finalize_game(winner)` — authority assigns winner (1 for A, 2 for B) and pays out.
- `cancel_game()` — after `expiry_ts`, refunds any deposited stakes back to players.

This flow uses native SOL, not wrapped SOL or SPL tokens.

## Versions

Verify your local environment with:

```bash
bash show-versions.sh
```

This repository was tested with:

```text
OS:
  MacOS 15.4.1
Solana CLI:
  solana-cli 2.1.21 (src:8a085eeb; feat:1416569292, client:Agave)
Anchor:
  anchor-cli 0.32.1
Node:
  v22.14.0
Rust:
  rustc 1.86.0 (05f9846f8 2025-03-31)
build-sbf version:
  solana-cargo-build-sbf 2.1.21
```

Using different versions may cause compatibility issues.

## Usage

1. Clone the repository:

   ```bash
   git clone https://github.com/mikemaccana/anchor-escrow-2025.git
   cd anchor-escrow-2025
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

3. Run TypeScript tests:

   ```bash
   anchor test
   ```

4. Run LiteSVM tests:

   ```bash
   cd programs/escrow
   cargo test
   ```

5. Deploy the program:

  ```bash
   anchor deploy
   ```

## Sanctum Gateway integration (Sanctum Sender)

This repo now includes a minimal integration with Sanctum Gateway so you can build and deliver transactions through Sanctum Sender.

What’s included:

- Reusable client at `scripts/gatewayClient.ts`
- A demo sender script at `scripts/sanctum-gateway-send.ts`

Run the demo on devnet:

1. Prepare environment variables (copy `.env.example` and fill in values or export them in your shell):

- `GATEWAY_API_KEY` — your key from <https://gateway.sanctum.so/>
- `GATEWAY_CLUSTER` — `devnet` or `mainnet` (default: `devnet`)
- `SENDER_SECRET_KEY_JSON` — JSON array from a `solana-keygen` keypair file (64-byte secret)
- optional `RECIPIENT_ADDRESS` — base58 address; defaults to self-transfer

1. Execute:

```bash
npm run sanctum:send
```

The script will:

- Create a simple SOL transfer instruction
- Call `buildGatewayTransaction` with `deliveryMethodType: "sanctum-sender"`
- Sign the optimized transaction locally
- Deliver it via Gateway’s `sendTransaction`

Note: The Anchor tests still run against a local validator and are unchanged. The Gateway integration is provided as an opt-in, standalone flow for devnet/mainnet usage.

## Changelog and Credits

See the [CHANGELOG](CHANGELOG.md) for updates and contributor credits.
