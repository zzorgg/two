import { before, describe, test, it } from "node:test";
import assert from "node:assert";
import * as programClient from "../dist/js-client";
import { connect, Connection, SOL, TOKEN_EXTENSIONS_PROGRAM } from "solana-kite";

// For debugging. You could delete these, but then someone else will have to recreate them and then they'll be annoyed with you.
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const log = console.log;
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const stringify = (object: any) => {
  const bigIntReplacer = (key: string, value: any) => (typeof value === "bigint" ? value.toString() : value);
  return JSON.stringify(object, bigIntReplacer, 2);
};

import { lamports, type KeyPairSigner, type Address } from "@solana/kit";

const ONE_SOL = lamports(1n * SOL);

// System program errors
// Reference: https://github.com/solana-labs/solana/blob/master/sdk/program/src/system_instruction.rs#L59
enum SystemError {
  // Account already in use
  AlreadyInUse = 0,
}

// SPL Token program errors
// Reference: https://github.com/solana-program/token-2022/blob/main/program/src/error.rs#L11-L13
enum SplTokenError {
  InsufficientFunds = 1,
}

// Anchor framework errors (2000-2999 range)
// Reference: https://github.com/coral-xyz/anchor/blob/master/lang/src/error.rs#L72-L74
enum AnchorError {
  ConstraintHasOne = 2001,
  ConstraintSeeds = 2006,
  AccountAlreadyInitialized = 2001,
  AccountNotInitialized = 3012,
}

const getRandomBigInt = () => {
  return BigInt(Math.floor(Math.random() * 1_000_000_000_000_000_000));
};

// Helper function to check for specific program errors
// Note: Solana errors do not include the program ID in the error object, so we can only check the error code in the message.
function assertProgramError(error: Error, expectedCode: SystemError | SplTokenError | AnchorError) {
  // Only check the error code in the message
  assert(
    error.message.includes(`custom program error: #${expectedCode}`),
    `Expected error code ${expectedCode} but got: ${error.message}`,
  );
}

// Helper function to create a test offer
async function createTestOffer(params: {
  connection: Connection;
  maker: KeyPairSigner;
  tokenMintA: Address;
  tokenMintB: Address;
  makerTokenAccountA: Address;
  tokenAOfferedAmount: bigint;
  tokenBWantedAmount: bigint;
  offerId?: bigint;
}) {
  const {
    connection,
    maker,
    tokenMintA,
    tokenMintB,
    makerTokenAccountA,
    tokenAOfferedAmount,
    tokenBWantedAmount,
    offerId = getRandomBigInt(),
  } = params;

  const offerPDAAndBump = await connection.getPDAAndBump(programClient.ESCROW_PROGRAM_ADDRESS, ["offer", offerId]);
  const offer = offerPDAAndBump.pda;
  const vault = await connection.getTokenAccountAddress(offer, tokenMintA, true);

  const makeOfferInstruction = await programClient.getMakeOfferInstructionAsync({
    maker,
    tokenMintA,
    tokenMintB,
    makerTokenAccountA,
    offer,
    vault,
    id: offerId,
    tokenAOfferedAmount,
    tokenBWantedAmount,
    tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
  });

  const signature = await connection.sendTransactionFromInstructions({
    feePayer: maker,
    instructions: [makeOfferInstruction],
  });

  return { offer, vault, offerId, signature };
}

describe("Escrow", () => {
  let connection: Connection;
  let user: KeyPairSigner;
  let alice: KeyPairSigner;
  let bob: KeyPairSigner;
  let tokenMintA: Address;
  let tokenMintB: Address;
  let aliceTokenAccountA: Address;
  let bobTokenAccountA: Address;
  let aliceTokenAccountB: Address;

  const tokenDecimals = 9;

  // Both tokens have 9 decimals, so we can use this to convert between major and minor units
  const TOKEN = 10n ** BigInt(tokenDecimals);

  // Alice is going to make a few offers in these tests, so we give her 10 tokens
  const aliceInitialTokenAAmount = 10n * TOKEN;
  // We have a test later where Bob tries to reuse Alice's offer ID, so we give him a tiny amount (1 minor unit) of token A
  const bobInitialTokenAAmount = 1n;
  // Bob has 1 token of token B he will offer in exchange
  const bobInitialTokenBAmount = 1n * TOKEN;

  // Alice will offer 1 token of token A in exchange for 1 token of token B
  const tokenAOfferedAmount = 1n * TOKEN;
  const tokenBWantedAmount = 1n * TOKEN;

  before(async () => {
    connection = await connect();

    // 'user' will be the account we use to create the token mints
    [user, alice, bob] = await connection.createWallets(3, { airdropAmount: ONE_SOL });

    // Create two token mints - the factories that create token A, and token B
    tokenMintA = await connection.createTokenMint({
      mintAuthority: user,
      decimals: tokenDecimals,
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
      decimals: tokenDecimals,
      name: "Token B",
      symbol: "TOKEN_B",
      uri: "https://example.com/token-b",
      additionalMetadata: {
        keyOne: "valueOne",
        keyTwo: "valueTwo",
      },
    });

    // Mint tokens to alice and bob
    await connection.mintTokens(tokenMintA, user, aliceInitialTokenAAmount, alice.address);
    await connection.mintTokens(tokenMintA, user, bobInitialTokenAAmount, bob.address);
    await connection.mintTokens(tokenMintB, user, bobInitialTokenBAmount, bob.address);

    // Get the token accounts for alice and bob
    aliceTokenAccountA = await connection.getTokenAccountAddress(alice.address, tokenMintA, true);
    bobTokenAccountA = await connection.getTokenAccountAddress(bob.address, tokenMintA, true);
    aliceTokenAccountB = await connection.getTokenAccountAddress(alice.address, tokenMintB, true);
  });

  describe("makeOffer", () => {
    test("successfully creates an offer with valid inputs", async () => {
      const { offer, vault } = await createTestOffer({
        connection,
        maker: alice,
        tokenMintA,
        tokenMintB,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        tokenBWantedAmount,
      });

      // Verify the offer was created successfully by checking the vault balance
      const vaultBalanceResponse = await connection.getTokenAccountBalance({
        tokenAccount: vault,
        mint: tokenMintA,
        useTokenExtensions: true,
      });
      assert(vaultBalanceResponse.amount === tokenAOfferedAmount);
    });

    test("fails when trying to reuse an existing offer ID", async () => {
      // First, create an offer with Alice using a specific offer ID
      const offerId = getRandomBigInt();
      await createTestOffer({
        connection,
        maker: alice,
        tokenMintA,
        tokenMintB,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        tokenBWantedAmount,
        offerId,
      });

      // Now try to create another offer with Bob using the same offer ID

      // Create bobTokenAccountA if it doesn't exist
      bobTokenAccountA = await connection.getTokenAccountAddress(bob.address, tokenMintA, true);

      let x: { offer: Address; vault: Address; offerId: bigint; signature: string };
      try {
        x = await createTestOffer({
          connection,
          maker: bob,
          tokenMintA,
          tokenMintB,
          makerTokenAccountA: bobTokenAccountA,
          tokenAOfferedAmount: bobInitialTokenAAmount,
          tokenBWantedAmount,
          offerId, // Reusing the same offer ID
        });
      } catch (thrownObject) {
        const error = thrownObject as Error;
        assertProgramError(error, SystemError.AlreadyInUse);
      }
    });

    test("fails when maker has insufficient token balance", async () => {
      const tooManyTokens = 1_000n * TOKEN;

      try {
        await createTestOffer({
          connection,
          maker: alice,
          tokenMintA,
          tokenMintB,
          makerTokenAccountA: aliceTokenAccountA,
          tokenAOfferedAmount: tooManyTokens,
          tokenBWantedAmount,
        });
        assert.fail("Expected the offer creation to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as Error;
        assertProgramError(error, SplTokenError.InsufficientFunds);
      }
    });
  });

  describe("takeOffer", () => {
    let testOffer: Address;
    let testVault: Address;

    before(async () => {
      const result = await createTestOffer({
        connection,
        maker: alice,
        tokenMintA,
        tokenMintB,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        tokenBWantedAmount,
      });
      testOffer = result.offer;
      testVault = result.vault;
    });

    test("successfully takes an offer", async () => {
      const takeOfferInstruction = await programClient.getTakeOfferInstructionAsync({
        taker: bob,
        maker: alice.address,
        tokenMintA,
        tokenMintB,
        takerTokenAccountA: bobTokenAccountA,
        makerTokenAccountB: aliceTokenAccountB,
        offer: testOffer,
        vault: testVault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      await connection.sendTransactionFromInstructions({
        feePayer: alice,
        instructions: [takeOfferInstruction],
      });

      // Verify token transfers
      const bobTokenABalance = await connection.getTokenAccountBalance({
        tokenAccount: bobTokenAccountA,
        mint: tokenMintA,
        useTokenExtensions: true,
      });
      assert(bobTokenABalance.amount === bobInitialTokenAAmount + tokenAOfferedAmount);

      const aliceTokenBBalance = await connection.getTokenAccountBalance({
        tokenAccount: aliceTokenAccountB,
        mint: tokenMintB,
        useTokenExtensions: true,
      });
      assert(aliceTokenBBalance.amount === tokenBWantedAmount);
    });

    test("fails when taker has insufficient token balance", async () => {
      const { offer, vault } = await createTestOffer({
        connection,
        maker: alice,
        tokenMintA,
        tokenMintB,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        tokenBWantedAmount: 10_000_000_000n,
      });

      const takeOfferInstruction = await programClient.getTakeOfferInstructionAsync({
        taker: bob,
        maker: alice.address,
        tokenMintA,
        tokenMintB,
        takerTokenAccountA: bobTokenAccountA,
        makerTokenAccountB: aliceTokenAccountB,
        offer,
        vault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      try {
        await connection.sendTransactionFromInstructions({
          feePayer: bob,
          instructions: [takeOfferInstruction],
        });
        assert.fail("Expected the take offer to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as Error;
        assertProgramError(error, SplTokenError.InsufficientFunds);
      }
    });
  });

  describe("refundOffer", () => {
    let testOffer: Address;
    let testVault: Address;

    before(async () => {
      const result = await createTestOffer({
        connection,
        maker: alice,
        tokenMintA,
        tokenMintB,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        tokenBWantedAmount,
      });
      testOffer = result.offer;
      testVault = result.vault;
    });

    test("successfully refunds an offer to the maker", async () => {
      const aliceBalanceBefore = await connection.getTokenAccountBalance({
        tokenAccount: aliceTokenAccountA,
        mint: tokenMintA,
        useTokenExtensions: true,
      });

      const refundOfferInstruction = await programClient.getRefundOfferInstructionAsync({
        maker: alice,
        tokenMintA,
        makerTokenAccountA: aliceTokenAccountA,
        offer: testOffer,
        vault: testVault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      await connection.sendTransactionFromInstructions({
        feePayer: alice,
        instructions: [refundOfferInstruction],
      });

      // Verify refund
      const aliceBalanceAfter = await connection.getTokenAccountBalance({
        tokenAccount: aliceTokenAccountA,
        mint: tokenMintA,
        useTokenExtensions: true,
      });
      assert(aliceBalanceAfter.amount > aliceBalanceBefore.amount);

      // Verify vault is closed
      const isClosed = await connection.checkTokenAccountIsClosed({
        tokenAccount: testVault,
        useTokenExtensions: true,
      });
      assert(isClosed);
    });

    test("fails when non-maker tries to refund the offer", async () => {
      const { offer, vault } = await createTestOffer({
        connection,
        maker: alice,
        tokenMintA,
        tokenMintB,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        tokenBWantedAmount,
      });

      const refundOfferInstruction = await programClient.getRefundOfferInstructionAsync({
        maker: bob,
        tokenMintA,
        makerTokenAccountA: bobTokenAccountA,
        offer,
        vault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      try {
        await connection.sendTransactionFromInstructions({
          feePayer: bob,
          instructions: [refundOfferInstruction],
        });
        assert.fail("Expected the refund to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as Error;
        // TODO: double check this is the right error
        assertProgramError(error, AnchorError.ConstraintHasOne);
      }
    });
  });
});
