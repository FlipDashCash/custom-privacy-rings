import { getProgramDerivedAddress } from "@solana/kit";
import { initializePoseidon } from "@heliuslabs/spr";
import type { SprClient } from "@heliuslabs/spr/client";
import {
  NO_UTXO_ROOT,
  type Address,
  type Bytes32,
  type RequestContext,
} from "@heliuslabs/spr/interface";
import {
  equalBytes,
  nullifierPdaAddress,
  rightAlign,
  treeAddress,
  ZERO_32,
} from "@heliuslabs/spr/internal";
import { bytes16, ShieldedPublicKey, X25519PublicKey } from "@heliuslabs/spr/keypair";
import {
  createExternalData,
  createProofOutput,
  outputBlindingSeed,
  ProofInputUtxo,
  SOL_MINT,
  SppProofInputs,
  transactOutputBlinding,
} from "@heliuslabs/spr/transaction";

import { RingError } from "../../protocol/error.js";
import { taggedX25519 } from "../../protocol/transaction/transact.js";
import { N_PUBLIC_SLOTS } from "../../protocol/transaction/spp.js";

const ADDRESS_TREE_ID = 0;
const ADDRESS_TREE = treeAddress(ADDRESS_TREE_ID);
// Stock spr_program ACCOUNT_BLINDING_SEED, a big-endian field element.
const BLINDING_SEED = rightAlign(Uint8Array.of(1));
// The no-output transaction has no encrypted recipient payload.
const EMPTY_VIEWING_KEY = X25519PublicKey.fromBytes(taggedX25519(ZERO_32));

async function reservationInput(ringProgramId: Address, ownerUtxoHash: Bytes32) {
  const [[owner]] = await Promise.all([
    getProgramDerivedAddress({
      programAddress: ringProgramId,
      seeds: ["actual_output_reservation"],
    }),
    initializePoseidon(),
  ]);
  const input = ProofInputUtxo.address(
    ShieldedPublicKey.fromPda(owner),
    ownerUtxoHash,
    ADDRESS_TREE_ID,
  );
  return { owner, input };
}

/** @internal The addresses used by the measured settlement's SPP CPI. */
export async function actualOutputReservationAddresses(
  ringProgramId: Address,
  ownerUtxoHash: Bytes32,
) {
  const { owner, input } = await reservationInput(ringProgramId, ownerUtxoHash);
  return {
    addressTree: ADDRESS_TREE,
    owner,
    nullifierPda: await nullifierPdaAddress(ADDRESS_TREE, input.nullifier()),
  };
}

/** @internal Proves only the measured output's address reservation, with no spend or published outputs. */
export async function proveActualOutputReservation(
  client: Pick<SprClient, "proveTransact">,
  ringProgramId: Address,
  ownerUtxoHash: Bytes32,
  payer: Address,
  context?: RequestContext,
) {
  const { input } = await reservationInput(ringProgramId, ownerUtxoHash);
  const nullifier = input.nullifier();
  const outputSeed = outputBlindingSeed(nullifier, BLINDING_SEED);
  const proven = await client.proveTransact(
    new SppProofInputs({
      payer,
      inputUtxos: [input],
      outputs: [0, 1].map((index) =>
        createProofOutput({
          asset: SOL_MINT,
          amount: 0n,
          blinding: transactOutputBlinding(nullifier, outputSeed, index),
          compact: true,
        }),
      ),
      externalData: createExternalData({
        txViewingPublicKey: EMPTY_VIEWING_KEY,
        salt: bytes16(new Uint8Array(16)),
        outputs: [],
        resolvedOwnerTags: [],
        messages: [],
      }),
      blindingSeed: BLINDING_SEED,
      outputTreeId: ADDRESS_TREE_ID,
    }),
    [],
    undefined,
    context,
  );
  const [root] = proven.treeContexts;
  if (
    proven.circuit.kind !== "confidentialSigner" ||
    proven.circuit.inputs !== 1 ||
    proven.circuit.outputs !== 2 ||
    proven.circuit.publicAssetSlots !== N_PUBLIC_SLOTS ||
    proven.inputs.length !== 1 ||
    proven.inputs[0]?.treeIndex !== 0 ||
    !equalBytes(proven.inputs[0].nullifierHash, nullifier) ||
    proven.outputs.length !== 0 ||
    proven.treeContexts.length !== 1 ||
    root?.utxoTreeRootIndex !== NO_UTXO_ROOT
  )
    throw new RingError("RING_BUILD_DEPOSIT", {
      details: { reason: "invalid actual-output address reservation proof" },
    });
  return Object.freeze({ nullifierRootIndex: root.nullifierTreeRootIndex, proof: proven.proof });
}
