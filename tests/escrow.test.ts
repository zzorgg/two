import { before, describe, test, it } from "node:test";
import assert from "node:assert";
import * as programClient from "../dist/js-client";
import { connect, Connection, SOL, TOKEN_EXTENSIONS_PROGRAM } from "solana-kite";

const SYSTEM_PROGRAM = "11111111111111111111111111111111" as Address;

// For debugging. You could delete these, but then someone else will have to recreate them and then they'll be annoyed with you.
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const log = console.log;
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const stringify = (obj: any) => JSON.stringify(obj, null, 2);

import { lamports, type KeyPairSigner, type Address } from "@solana/kit";

const ONE_SOL = lamports(1n * SOL);

const getRandomBigInt = () => {
  return BigInt(Math.floor(Math.random() * 1_000_000_000_000_000_000));
};

// See https://www.quicknode.com/guides/solana-development/tooling/web3-2/program-clients#generate-clients
describe("Escrow", () => {
  let connection: Connection;
  let user: KeyPairSigner;

  // Alice will be the maker (creator) of the offer
  let alice: KeyPairSigner;

  // Bob will be the taker (acceptor) of the offer
  let bob: KeyPairSigner;

  // tokenMintA represents the token Alice is offering
  // tokenMintB represents the token Alice wants in return
  let tokenMintA: Address;
  let tokenMintB: Address;

  let vault: Address;
  let offer: Address;

  let offerId: bigint;

  let aliceTokenAccountA: Address;
  let bobTokenAccountA: Address;
  let aliceTokenAccountB: Address;

  let tokenAOfferedAmount = 1_000_000_000n;
  let tokenBWantedAmount = 1_000_000_000n;

  // Create Alice and Bob accounts, 2 token mints, and associated token accounts for both tokens for both users
  before(async () => {
    connection = await connect();

    // This is the user that will pay for the transactions to create the token mints
    [user, alice, bob] = await connection.createWallets(3, { airdropAmount: ONE_SOL });

    tokenMintA = await connection.createTokenMint({
      mintAuthority: user,
      decimals: 9,
      name: "Token A",
      symbol: "TOKEN_A",
      uri: "https://example.com/token-a",
      additionalMetadata: {
        keyOne: "valueOne",
        keyTwo: "valueTwo",
      },
    });

    tokenMintB = await connection.createTokenMint({
      mintAuthority: user,
      decimals: 9,
      name: "Token B",
      symbol: "TOKEN_B",
      uri: "https://example.com/token-b",
      additionalMetadata: {
        keyOne: "valueOne",
        keyTwo: "valueTwo",
      },
    });

    // Alice will have 2n * tokenAOfferedAmount of token A and 0 of token B
    // 2n * tokenAOfferedAmount because Alice will make two offers, and each offer will have tokenAOfferedAmount of token A
    // the first will be taken, and the second will be refunded.
    await connection.mintTokens(tokenMintA, user, 2n * tokenAOfferedAmount, alice.address);

    // Get Alice's token A account
    aliceTokenAccountA = await connection.getTokenAccountAddress(alice.address, tokenMintA, true);

    // Bob will have 0 of token A and 1_000_000_000 of token B
    // he will use the Token B to take Alice's offer.
    await connection.mintTokens(tokenMintB, user, 1_000_000_000n, bob.address);
  });

  test("Puts the tokens Alice offers into the vault when Alice makes an offer", async () => {
    offerId = getRandomBigInt();

    // Get Bob's token A account (which may not exist yet)
    bobTokenAccountA = await connection.getTokenAccountAddress(bob.address, tokenMintA, true);

    // Get Alice's token B account (which may not exist yet)
    aliceTokenAccountB = await connection.getTokenAccountAddress(alice.address, tokenMintB, true);

    // Derive the offer PDA
    const offerPDAAndBump = await connection.getPDAAndBump(programClient.ESCROW_PROGRAM_ADDRESS, [
      "offer",
      alice.address,
      offerId,
    ]);

    offer = offerPDAAndBump.pda;

    // Derive the vault PDA (which will be an Associated Token Account)
    vault = await connection.getTokenAccountAddress(offer, tokenMintA, true);

    const makeOfferInstruction = await programClient.getMakeOfferInstructionAsync({
      maker: alice,
      tokenMintA,
      tokenMintB,
      makerTokenAccountA: aliceTokenAccountA,
      offer,
      vault,
      id: offerId,
      tokenAOfferedAmount,
      tokenBWantedAmount,
      tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
    });

    const transactionSignature = await connection.sendTransactionFromInstructions({
      feePayer: alice,
      instructions: [makeOfferInstruction],
    });
  });

  test("Puts the tokens from the vault into Bob's account, and gives Alice Bob's tokens, when Bob takes an offer", async () => {
    const takeOfferInstruction = await programClient.getTakeOfferInstructionAsync({
      taker: bob,
      maker: alice.address,
      tokenMintA,
      tokenMintB,
      takerTokenAccountA: bobTokenAccountA,
      makerTokenAccountB: aliceTokenAccountB,
      offer: offer,
      vault: vault,
      tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
    });

    const transactionSignature = await connection.sendTransactionFromInstructions({
      feePayer: alice,
      instructions: [takeOfferInstruction],
    });

    // Check the offered tokens are now in Bob's account
    // (note: there is no before balance as Bob didn't have any offered tokens before the transaction)
    const bobTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance({
      tokenAccount: bobTokenAccountA,
      mint: tokenMintA,
      useTokenExtensions: true,
    });

    const bobTokenAccountBalanceAfter = bobTokenAccountBalanceAfterResponse.amount;
    assert(bobTokenAccountBalanceAfter === tokenAOfferedAmount);

    // Check the wanted tokens are now in Alice's account
    // (note: there is no before balance as Alice didn't have any wanted tokens before the transaction)
    const aliceTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance({
      tokenAccount: aliceTokenAccountB,
      mint: tokenMintB,
      useTokenExtensions: true,
    });

    const aliceTokenAccountBalanceAfter = aliceTokenAccountBalanceAfterResponse.amount;
    assert(aliceTokenAccountBalanceAfter === tokenBWantedAmount);
  });

  test("Returns tokens to Alice when she refunds her offer", async () => {
    // We'll reuse the same token mints, but make a new offer and then refund it
    // Create a new offer
    const newOfferId = getRandomBigInt();
    const newOfferPDAAndBump = await connection.getPDAAndBump(programClient.ESCROW_PROGRAM_ADDRESS, [
      "offer",
      alice.address,
      newOfferId,
    ]);
    const newOffer = newOfferPDAAndBump.pda;
    const newVault = await connection.getTokenAccountAddress(newOffer, tokenMintA, true);

    const aliceSolBalance = await connection.getLamportBalance(alice.address);

    // Make a new offer, using a new offerId and offer account

    const makeOfferInstruction = await programClient.getMakeOfferInstructionAsync({
      maker: alice,
      tokenMintA,
      tokenMintB,
      makerTokenAccountA: aliceTokenAccountA,
      offer: newOffer,
      vault: newVault,
      id: newOfferId,
      tokenAOfferedAmount,
      tokenBWantedAmount,
      tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
    });
    const transactionSignature = await connection.sendTransactionFromInstructions({
      feePayer: alice,
      instructions: [makeOfferInstruction],
    });

    // Get Alice's token balance before refund
    const aliceTokenAccountBalanceBeforeResponse = await connection.getTokenAccountBalance({
      tokenAccount: aliceTokenAccountA,
      mint: tokenMintA,
      useTokenExtensions: true,
    });
    const aliceTokenAccountBalanceBefore = aliceTokenAccountBalanceBeforeResponse.amount;
    // Refund the offer
    const refundOfferInstruction = await programClient.getRefundOfferInstructionAsync({
      maker: alice,
      tokenMintA,
      makerTokenAccountA: aliceTokenAccountA,
      offer: newOffer,
      vault: newVault,
      tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
    });
    const refundTransactionSignature = await connection.sendTransactionFromInstructions({
      feePayer: alice,
      instructions: [refundOfferInstruction],
    });

    // Check tokens were returned to Alice
    const aliceTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance({
      wallet: alice.address,
      mint: tokenMintA,
      useTokenExtensions: true,
    });
    const aliceTokenAccountBalanceAfter = aliceTokenAccountBalanceAfterResponse.amount;
    // Assert the balance is greater than the before balance
    assert(aliceTokenAccountBalanceAfter > aliceTokenAccountBalanceBefore);
    // Verify vault is closed
    const isClosed = await connection.checkTokenAccountIsClosed({
      tokenAccount: newVault,
      useTokenExtensions: true,
    });
    assert(isClosed);
  });
});
