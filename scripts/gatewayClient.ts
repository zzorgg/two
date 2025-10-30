// Lightweight Sanctum Gateway client for build + send
// Avoid importing kit types here to keep this client decoupled

export type Cluster = "mainnet" | "devnet";

export type BuildGatewayTransactionOptions = {
  encoding?: "base64" | "base58";
  skipSimulation?: boolean;
  skipPriorityFee?: boolean;
  cuPriceRange?: "low" | "medium" | "high";
  jitoTipRange?: "low" | "medium" | "high" | "max";
  expireInSlots?: number;
  deliveryMethodType?: "rpc" | "jito" | "sanctum-sender" | "helius-sender";
};

export function createGatewayClient(params: { apiKey: string; cluster: Cluster }) {
  const { apiKey, cluster } = params;
  const endpoint = `https://tpg.sanctum.so/v1/${cluster}?apiKey=${apiKey}`;

  async function rpc<T = any>(body: any): Promise<T> {
    const res = await fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: "sanctum-integration", jsonrpc: "2.0", ...body }),
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`Gateway HTTP error ${res.status}: ${text}`);
    }
    const json = await res.json();
    if (json.error) {
      throw new Error(`Gateway RPC error ${json.error.code}: ${json.error.message}`);
    }
    return json as T;
  }

  return {
    endpoint,
    async buildGatewayTransaction(
      encodedUnsignedTx: string,
      options: BuildGatewayTransactionOptions = {}
    ): Promise<{
      transaction: string;
      latestBlockhash: { blockhash: string; lastValidBlockHeight: string };
    }> {
      const body = {
        method: "buildGatewayTransaction",
        params: [encodedUnsignedTx, options],
      };
      const json = await rpc<{ result: { transaction: string; latestBlockhash: { blockhash: string; lastValidBlockHeight: string } } }>(body);
      return json.result;
    },

    async getTipInstructions(params: {
      feePayer: string;
      jitoTipRange?: "low" | "medium" | "high" | "max";
      deliveryMethodType?: "rpc" | "jito" | "sanctum-sender" | "helius-sender";
    }): Promise<any[]> {
      const json = await rpc<{ result: any[] }>({ method: "getTipInstructions", params: [params] });
      // Coerce data fields back to Uint8Array if needed by caller
      return json.result.map((ix: any) => ({ ...ix, data: new Uint8Array(Object.values(ix.data ?? {})) }));
    },

    async sendTransaction(encodedSignedTx: string, options?: { encoding?: "base64" | "base58"; startSlot?: number }) {
      const body = { method: "sendTransaction", params: [encodedSignedTx, options ?? { encoding: "base64" }] };
      const json = await rpc<{ result: string }>(body);
      return json.result; // signature
    },
  };
}
