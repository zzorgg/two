import { before, describe, test, it } from "node:test";
import { assert } from "node:console";
import * as programClient from "../dist/js-client";
import { connect, Connection, SOL, TOKEN_EXTENSIONS_PROGRAM, TOKEN_PROGRAM } from "@helius-dev/kite";

const SYSTEM_PROGRAM = "11111111111111111111111111111111" as Address;

// For debugging. You could delete this, but then someone else will have to recreate it and then they'll be annoyed with you.
const log = console.log;
const stringify = (obj: any) => JSON.stringify(obj, null, 2);

import {
  lamports,
  getProgramDerivedAddress,
  getAddressEncoder,
  address,
  type KeyPairSigner,
  type Address,
} from "@solana/web3.js";

const ONE_SOL = lamports(1n * SOL);

const getRandomBigInt = () => {
  return BigInt(Math.floor(Math.random() * 1000000000000000000));
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
    console.log(programClient);

    // This is the user that will pay for the transactions to create the token mints
    log("Creating user, alice, and bob...");
    [user, alice, bob] = await connection.createWallets(3, { airdropAmount: ONE_SOL });
    log("  ✅ User, alice, and bob created");

    log("Creating token mints...");
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

    log("  ✅ Token mints created");

    // Alice will have 1_000_000_000 of token A and 0 of token B
    log("Minting tokens to Alice's account...");
    await connection.mintTokens(tokenMintA, user, 1_000_000_000n, alice.address);
    log("  ✅ Tokens minted to Alice's account... ");

    // Get Alice's token A account
    aliceTokenAccountA = await connection.getTokenAccountAddress(alice.address, tokenMintA, true);

    // Bob will have 0 of token A and 1_000_000_000 of token B
    log("Minting tokens to bob...");
    await connection.mintTokens(tokenMintB, user, 1_000_000_000n, bob.address);
    log("  ✅ Tokens minted to bob");
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

    log("Making offer instruction...");

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

    console.log("  ✅ Transaction signature:", transactionSignature);
  });

  it("Puts the tokens from the vault into Bob's account, and gives Alice Bob's tokens, when Bob takes an offer", async () => {
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
    const bobTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance(bob.address, tokenMintA, true);

    // TODO: why is this any? Types aren't importing maybe?
    assert(bobTokenAccountBalanceAfterResponse.amount === tokenAOfferedAmount);

    // Check the wanted tokens are now in Alice's account
    // (note: there is no before balance as Alice didn't have any wanted tokens before the transaction)
    const aliceTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance(
      alice.address,
      tokenMintB,
      true,
    );

    assert(aliceTokenAccountBalanceAfterResponse.amount === tokenBWantedAmount);
  });

  // it("Returns tokens to Alice when she refunds her offer", async () => {
  //   // Create a new offer first
  //   const newOfferId = getRandomBigInt();

  //   const newOffer = await connection.getPDAAndBump(programClient.ESCROW_PROGRAM_ADDRESS, [
  //     "offer",
  //     alice.address,
  //     newOfferId,
  //   ]);

  //   const newVault = await connection.getTokenAccountAddress(newOffer, tokenMintA, true);

  //   // aliceTokenAccountA

  //   // Make a new offer, using a new offerId and offer account
  //   const makeOfferInstruction = await programClient.getMakeOfferInstructionAsync({
  //     maker: alice,
  //     tokenMintA,
  //     tokenMintB,
  //     makerTokenAccountA: aliceTokenAccountA,
  //     offer: newOffer,
  //     vault: newVault,
  //     id: newOfferId,
  //     tokenAOfferedAmount,
  //     tokenBWantedAmount,
  //     tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
  //   });

  //   const transactionSignature = await connection.sendTransactionFromInstructions({
  //     feePayer: alice,
  //     instructions: [makeOfferInstruction],
  //   });

  //   // Get Alice's token balance before refund
  //   const aliceTokenAccountBalanceBeforeResponse = await connection.getTokenAccountBalance(aliceTokenAccountA);
  //   const aliceTokenAccountBalanceBefore = aliceTokenAccountBalanceBeforeResponse.amount;

  //   // Refund the offer
  //   const refundOfferInstruction = await programClient.getRefundOfferInstructionAsync({
  //     maker: alice,
  //     tokenMintA,
  //     makerTokenAccountA: aliceTokenAccountA,
  //     offer: newOffer,
  //     vault: newVault,
  //     tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
  //   });

  //   const refundTransactionSignature = await connection.sendTransactionFromInstructions({
  //     feePayer: alice,
  //     instructions: [refundOfferInstruction],
  //   });

  //   // Check tokens were returned to Alice
  //   const aliceTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance(aliceTokenAccountA);
  //   const aliceTokenAccountBalanceAfter: BigInt = aliceTokenAccountBalanceAfterResponse.amount;

  //   // Assert the balance is greater than the before balance
  //   assert(aliceTokenAccountBalanceAfter > aliceTokenAccountBalanceBefore);

  //   // Verify vault is closed
  //   try {
  //     await connection.getTokenAccountBalance(newVault);
  //     assert(false, "Vault should be closed");
  //   } catch (thrownObject) {
  //     const error = thrownObject as Error;
  //     assert(error.name === "TokenAccountNotFoundError");
  //   }
  // });
});
