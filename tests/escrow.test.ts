import { before, describe, test, it } from "node:test";
import { assert } from "node:console";
import * as programClient from "../dist/js-client";
import { connect } from "@helius-dev/kite";
import { lamports, address, KeyPairSigner, Address } from "@solana/web3.js";

const SOL = 1_000_000_000n;
const ONE_SOL = lamports(1n * SOL);

// See https://www.quicknode.com/guides/solana-development/tooling/web3-2/program-clients#generate-clients
describe("Escrow", () => {
  let user: KeyPairSigner;

  // Alice will be the maker (creator) of the offer

  let alice: KeyPairSigner;

  // Bob will be the taker (acceptor) of the offer

  let bob: KeyPairSigner;

  // tokenMintA represents the token Alice is offering
  // tokenMintB represents the token Alice wants in return

  let tokenMintA: Address;
  let tokenMintB: Address;

  // Create Alice and Bob accounts, 2 token mints, and associated token accounts for both tokens for both users
  before(async () => {
    const connection = await connect();
    console.log(programClient);

    // This is the user that will pay for the transactions to create the token mints
    user = await connection.createWallet({ airdropAmount: ONE_SOL });

    [alice, bob] = await connection.createWallets(2, { airdropAmount: ONE_SOL });

    tokenMintA = await connection.createTokenMint(user, 9, "Token A", "TOKEN_A", "https://example.com/token-a", {
      website: "https://example.com",
      twitter: "https://twitter.com/example",
    });

    tokenMintB = await connection.createTokenMint(user, 9, "Token B", "TOKEN_B", "https://example.com/token-b", {
      website: "https://example.com",
      twitter: "https://twitter.com/example",
    });

    // Alice will have 1_000_000_000 of token A and 0 of token B
    // Mint 1_000_000_000 of token A to alice
    await connection.mintTokens(tokenMintA, user, 1_000_000_000n, alice.address);

    // Bob will have 0 of token A and 1_000_000_000 of token B
    await connection.mintTokens(tokenMintB, user, 1_000_000_000n, bob.address);
  });

  test("passed", () => {
    assert(true);
  });

  // it("Puts the tokens Alice offers into the vault when Alice makes an offer", async () => {
  //   // Pick a random ID for the offer we'll make
  //   const offerId = getRandomBigNumber();

  //   // Then determine the account addresses we'll use for the offer and the vault
  //   const offer = PublicKey.findProgramAddressSync(
  //     [Buffer.from("offer"), accounts.maker.toBuffer(), offerId.toArrayLike(Buffer, "le", 8)],
  //     program.programId,
  //   )[0];

  //   const vault = getAssociatedTokenAddressSync(accounts.tokenMintA, offer, true, TOKEN_PROGRAM);

  //   accounts.offer = offer;
  //   accounts.vault = vault;

  //   const transactionSignature = await program.methods
  //     .makeOffer(offerId, tokenAOfferedAmount, tokenBWantedAmount)
  //     // @ts-expect-error the error says tokenMintA, tokenMintB, tokenProgram are missing, however they are created in the before hook
  //     .accounts({ ...accounts })
  //     .signers([alice])
  //     .rpc();

  //   await confirmTransaction(connection, transactionSignature);

  //   // Check our vault contains the tokens offered
  //   const vaultBalanceResponse = await connection.getTokenAccountBalance(vault);
  //   const vaultBalance = new BN(vaultBalanceResponse.value.amount);
  //   assert(vaultBalance.eq(tokenAOfferedAmount));

  //   // Check our Offer account contains the correct data
  //   const offerAccount = await program.account.offer.fetch(offer);

  //   assert(offerAccount.maker.equals(alice.publicKey));
  //   assert(offerAccount.tokenMintA.equals(accounts.tokenMintA));
  //   assert(offerAccount.tokenMintB.equals(accounts.tokenMintB));
  //   assert(offerAccount.tokenBWantedAmount.eq(tokenBWantedAmount));
  // });

  // it("Puts the tokens from the vault into Bob's account, and gives Alice Bob's tokens, when Bob takes an offer", async () => {
  //   const transactionSignature = await program.methods
  //     .takeOffer()
  //     // @ts-expect-error the error says tokenMintA, tokenProgram are missing, however they are created in the before hook
  //     .accounts({ ...accounts })
  //     .signers([bob])
  //     .rpc();

  //   await confirmTransaction(connection, transactionSignature);

  //   // Check the offered tokens are now in Bob's account
  //   // (note: there is no before balance as Bob didn't have any offered tokens before the transaction)
  //   const bobTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance(accounts.takerTokenAccountA);
  //   const bobTokenAccountBalanceAfter = new BN(bobTokenAccountBalanceAfterResponse.value.amount);
  //   assert(bobTokenAccountBalanceAfter.eq(tokenAOfferedAmount));

  //   // Check the wanted tokens are now in Alice's account
  //   // (note: there is no before balance as Alice didn't have any wanted tokens before the transaction)
  //   const aliceTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance(accounts.makerTokenAccountB);
  //   const aliceTokenAccountBalanceAfter = new BN(aliceTokenAccountBalanceAfterResponse.value.amount);
  //   assert(aliceTokenAccountBalanceAfter.eq(tokenBWantedAmount));
  // });

  // it("Returns tokens to Alice when she refunds her offer", async () => {
  //   // Create a new offer first
  //   const offerId = getRandomBigNumber();

  //   const offer = PublicKey.findProgramAddressSync(
  //     [Buffer.from("offer"), accounts.maker.toBuffer(), offerId.toArrayLike(Buffer, "le", 8)],
  //     program.programId,
  //   )[0];

  //   const vault = getAssociatedTokenAddressSync(accounts.tokenMintA, offer, true, TOKEN_PROGRAM);

  //   accounts.offer = offer;
  //   accounts.vault = vault;

  //   // Make the offer
  //   let transactionSignature = await program.methods
  //     .makeOffer(offerId, tokenAOfferedAmount, tokenBWantedAmount)
  //     // @ts-expect-error the error says tokenMintA, tokenMintB, tokenProgram are missing, however they are created in the before hook
  //     .accounts({ ...accounts })
  //     .signers([alice])
  //     .rpc();

  //   await confirmTransaction(connection, transactionSignature);

  //   // Get Alice's token balance before refund
  //   const aliceTokenAccountBalanceBeforeResponse = await connection.getTokenAccountBalance(accounts.makerTokenAccountA);
  //   const aliceTokenAccountBalanceBefore = new BN(aliceTokenAccountBalanceBeforeResponse.value.amount);

  //   // Refund the offer
  //   transactionSignature = await program.methods
  //     .refundOffer()
  //     .accounts({
  //       // @ts-expect-error maker exists, the tests pass with it
  //       maker: accounts.maker,
  //       tokenMintA: accounts.tokenMintA,
  //       makerTokenAccountA: accounts.makerTokenAccountA,
  //       offer: accounts.offer,
  //       vault: accounts.vault,
  //       tokenProgram: accounts.tokenProgram,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .signers([alice])
  //     .rpc();

  //   await confirmTransaction(connection, transactionSignature);

  //   // Check tokens were returned to Alice
  //   const aliceTokenAccountBalanceAfterResponse = await connection.getTokenAccountBalance(accounts.makerTokenAccountA);
  //   const aliceTokenAccountBalanceAfter = new BN(aliceTokenAccountBalanceAfterResponse.value.amount);
  //   assert(aliceTokenAccountBalanceAfter.gt(aliceTokenAccountBalanceBefore));

  //   // Verify vault is closed
  //   try {
  //     await connection.getTokenAccountBalance(accounts.vault);
  //     assert(false, "Vault should be closed");
  //   } catch (thrownObject) {
  //     const error = thrownObject as Error;
  //     assert(thrownObject.name === "TokenAccountNotFoundError");
  //   }
});
