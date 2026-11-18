import { ed25519 } from "@noble/curves/ed25519.js";
import { getBase58Decoder, getBase64Encoder, type TransactionPartialSigner } from "@solana/kit";
import { describe, expect, it } from "vitest";

import type { Address, Signature } from "@heliuslabs/spr/interface";

import type {
  CoSignTransactionRequest,
  CoSignTransactionResponse,
} from "../../src/protocol/ring-rpc/ring-rpc-types.js";
import { ringRpcCoSigner } from "../../src/rpc/ring-rpc-co-signer.js";

const base58 = getBase58Decoder();

const secretKey = new Uint8Array(32).fill(8);
const signer = base58.decode(ed25519.getPublicKey(secretKey)) as Address;
const ring = base58.decode(new Uint8Array(32).fill(7)) as Address;
const messageBytes = Uint8Array.of(0x81, 1, 2, 3, 4);

type Transaction = Parameters<TransactionPartialSigner["signTransactions"]>[0][number];
const transaction = { messageBytes, signatures: { [signer]: null } } as unknown as Transaction;

function answering(answer: (message: Uint8Array) => CoSignTransactionResponse) {
  const requests: CoSignTransactionRequest[] = [];
  const rpc = {
    coSignTransaction: async (request: CoSignTransactionRequest) => {
      requests.push(request);
      return answer(new Uint8Array(getBase64Encoder().encode(request.message)));
    },
  };
  return { requests, coSigner: ringRpcCoSigner({ rpc, ring, signer }) };
}

function signedBy(key: Uint8Array, message: Uint8Array): Signature {
  return base58.decode(ed25519.sign(message, key)) as Signature;
}

describe("ringRpcCoSigner", () => {
  it("returns the service's signature over the message bytes", async () => {
    const { requests, coSigner } = answering((message) => ({
      signer,
      signature: signedBy(secretKey, message),
    }));
    expect(coSigner.address).toBe(signer);
    const [signatures] = await coSigner.signTransactions([transaction]);
    expect(signatures?.[signer]).toEqual(ed25519.sign(messageBytes, secretKey));
    expect(requests).toEqual([{ ring, message: "gQECAwQ=" }]);
  });

  it("refuses a signature from another signer", async () => {
    const otherKey = new Uint8Array(32).fill(9);
    const { coSigner } = answering((message) => ({
      signer: base58.decode(ed25519.getPublicKey(otherKey)) as Address,
      signature: signedBy(otherKey, message),
    }));
    await expect(coSigner.signTransactions([transaction])).rejects.toMatchObject({
      code: "RING_RPC",
      details: { method: "coSignTransaction", reason: "signer is not the expected co-signer" },
    });
  });

  it("refuses a signature over other bytes", async () => {
    const { coSigner } = answering(() => ({
      signer,
      signature: signedBy(secretKey, Uint8Array.of(9, 9, 9)),
    }));
    await expect(coSigner.signTransactions([transaction])).rejects.toMatchObject({
      code: "RING_RPC",
      details: { method: "coSignTransaction", reason: "co-signature is invalid" },
    });
  });
});
