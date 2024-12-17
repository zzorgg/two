# Anchor Escrow

An Anchor Escrow Project: ![CI Badge](https://github.com/your-username/your-repo/actions/workflows/your-workflow.yml/badge.svg)

## Introduction

This Solana program is called an **_escrow_** - it allows a user to swap a specific amount of one token for a desired amount of another token.

For example, Alice is offering 10 USDC, and wants 100 WIF in return.

Without our program, users would have to engage in manual token swapping. Imagine the potential problems if Bob promised to send Alice 100 WIF, but instead took the 10 USDC and ran? Or what if Alice was dishonest, received the 10 USDC from Bob, and decided not to send the 100 WIF? Our Escrow program handles these complexities by acting a trusted entity that will only release tokens to both parties at the right time.

Our Escrow program is designed to provide a secure environment for users to swap a specific amount of one token with a specific amount of another token without having to trust each other.

Better yet, since our program allows Alice and Bob to transact directly with each other, they both get a hundred percent of the token they desire!

## Versions

You can check the versions on your own machine with:

```bash
echo "Solana CLI: $(solana -V)\nAnchor: $(anchor --version)\nNode: $(node --version)\nRust: $(rustc -V)"
```

This repo was tested with:

```
Solana CLI: solana-cli 2.0.17 (src:7104d713; feat:607245837, client:Agave)
Anchor: anchor-cli 0.30.1
Node: v22.11.0
Rust: rustc 1.84.0-nightly (03ee48451 2024-11-18)
```

## Usage

[Current releases of Rust mean you may wish to set the following environment variable](https://solana.stackexchange.com/questions/17777/unexpected-cfg-condition-value-solana):

```bash
export RUSTUP_TOOLCHAIN='nightly-2024-11-19'
```

Then `anchor test`, `anchor deploy` etc.

## Changelog

This project has a [CHANGELOG](CHANGELOG.md). Go read it.
