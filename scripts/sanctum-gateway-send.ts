/*
  Demo: Build and send a simple SOL transfer via Sanctum Gateway (Sanctum Sender)

  Requirements:
  - export GATEWAY_API_KEY=... (from Sanctum dashboard)
  - export GATEWAY_CLUSTER=devnet|mainnet (default: devnet)
  - export SENDER_SECRET_KEY_JSON='[<64 numbers>]' (secret key array from `solana-keygen new -o keypair.json`)
  - optional export RECIPIENT_ADDRESS=<base58>

  Run:
    npm run sanctum:send
*/
import 'dotenv/config';
import {
  address,
  appendTransactionMessageInstructions,
  blockhash,
  compileTransaction,
  createTransactionMessage,
  getBase64EncodedWireTransaction,
  getBase64Encoder,
  getTransactionDecoder,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransaction,
} from "@solana/kit";
import { getTransferSolInstruction } from "@solana-program/system";
import { createGatewayClient, type Cluster } from "./gatewayClient.js";

function env(name: string, fallback?: string): string {
  const v = process.env[name] ?? fallback;
  if (v === undefined) throw new Error(`Missing required env: ${name}`);
  return v;
}

async function main() {
  const apiKey = env("GATEWAY_API_KEY");
  const cluster = (process.env.GATEWAY_CLUSTER as Cluster) || "devnet";
  const deliveryMethod = (process.env.DELIVERY_METHOD || "rpc") as
    | "rpc"
    | "jito"
    | "sanctum-sender"
    | "helius-sender";
  const kpJson = env("SENDER_SECRET_KEY_JSON");
  const secret: number[] = JSON.parse(kpJson);
  const secretBytes = new Uint8Array(secret);
  // lazy import to avoid top-level type coupling
  const { createKeyPairSignerFromBytes } = await import("@solana/kit");
  const feePayer = await createKeyPairSignerFromBytes(secretBytes);

  console.log("Fee payer address:", feePayer.address);
  console.log("Cluster:", cluster, "Delivery method:", deliveryMethod);

  const recipient = process.env.RECIPIENT_ADDRESS
    ? address(process.env.RECIPIENT_ADDRESS)
    : feePayer.address; // self-transfer if not provided

  const tipLamports = 100_000n; // 0.0001 SOL minimal tip per docs
  const transferIx = getTransferSolInstruction({
    source: feePayer,
    destination: recipient,
    amount: tipLamports, // small transfer
  });

  const unsignedTx = createUnsignedTx([transferIx], feePayer);

  const client = createGatewayClient({ apiKey, cluster });
  console.log("Gateway endpoint:", client.endpoint);

  const built = await client.buildGatewayTransaction(
    getBase64EncodedWireTransaction(unsignedTx as any),
    {
      encoding: "base64",
      deliveryMethodType: deliveryMethod,
      // You can also override cuPriceRange/jitoTipRange/etc here
    }
  );

  const decodedTx = getTransactionDecoder().decode(
    getBase64Encoder().encode(built.transaction)
  ) as any;

  const signedTx = await signTransaction([feePayer.keyPair] as any, decodedTx as any);

  try {
    const sig = await client.sendTransaction(
      getBase64EncodedWireTransaction(signedTx as any),
      { encoding: "base64" }
    );
    console.log("Sent via Sanctum Gateway (Sanctum Sender)", { signature: sig, cluster });
    console.log(`View on Solana Explorer: https://explorer.solana.com/tx/${sig}?cluster=${cluster}`);
  } catch (e: any) {
    const msg = String(e?.message ?? e);
    if (msg.includes("No delivery methods found")) {
      console.error("No delivery methods are linked to your project for this cluster.");
      console.error("Next steps: \n- Open Dashboard > Delivery Methods\n- Create or select Sanctum Sender (cluster must match:", cluster, ")\n- Click 'Add to Project' to link it\n- Re-run: npm run sanctum:send");
      console.error("Quick link: https://gateway.sanctum.so/dashboard/delivery-methods");
    }
    throw e;
  }
}

function createUnsignedTx(ixs: any[], feePayer: any) {
  const m0 = createTransactionMessage({ version: 0 });
  const m1 = appendTransactionMessageInstructions(ixs as any, m0);
  const m2 = setTransactionMessageFeePayerSigner(feePayer, m1);
  const m3 = setTransactionMessageLifetimeUsingBlockhash(
    { blockhash: blockhash("11111111111111111111111111111111"), lastValidBlockHeight: 1n },
    m2
  );
  return compileTransaction(m3);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
