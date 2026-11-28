import { AccountRole, getProgramDerivedAddress } from "@solana/kit";
import { describe, expect, it } from "vitest";

import {
  NO_UTXO_ROOT,
  SHIELDED_POOL_PROGRAM_ID,
  SPL_TOKEN_2022_PROGRAM_ID,
  type InterfaceTransfer,
  type TransactProof,
} from "@heliuslabs/spr/interface";
import {
  concatBytes,
  Reader,
  SYSTEM_PROGRAM,
  nullifierPdaAddress,
  ringAuthAddress,
  splInterfaceWithBump,
  treeAddress,
} from "@heliuslabs/spr/internal";
import { ShieldedPublicKey } from "@heliuslabs/spr/keypair";
import { ProofInputUtxo, SOL_MINT, type SppProofInputs } from "@heliuslabs/spr/transaction";

import {
  buildSettleActualOutputInstruction,
  type ActualOutputDescriptor,
} from "../../../src/operations/payments/actual-output.js";
import { proveActualOutputReservation } from "../../../src/operations/payments/actual-output-reservation.js";
import { ringPolicyAddress } from "../../../src/protocol/pda/index.js";
import { CUSTOM_RING_PROOF_LENGTH } from "../../../src/protocol/proof/custom-ring-proof.js";
import {
  compactMeasuredCertificate,
  PLAIN_GROTH16_PROOF_LENGTH,
  type RingTransactData,
} from "../../../src/protocol/transaction/transact.js";
import { ringFrame, ringInstructionData } from "../../helpers/ring-frame.js";
import { filled, randomAddress, TREE } from "../../helpers/ring-pipeline.js";

function measuredFrame(leg: InterfaceTransfer, withRecords: boolean): RingTransactData {
  const { ownRevision: _ownRevision, ...frame } = ringFrame({
    rail: "anonymous",
    inputs: withRecords ? 2 : 1,
    outputs: 2,
    publicOwners: true,
  });
  return {
    ...frame,
    ...(withRecords ? { ownRevision: 0n } : {}),
    noSpend: true,
    interfaceTransfers: [leg],
    ...(withRecords
      ? {
          inputTrees: {
            kind: "two",
            first:
              frame.inputTrees.kind === "one" ? frame.inputTrees.context : frame.inputTrees.first,
            second: { utxoTreeRootIndex: 0xffff, nullifierTreeRootIndex: 9 },
            secondInputs: 2n,
          },
        }
      : {}),
  };
}

const proof: TransactProof = {
  a: filled(10),
  b: new Uint8Array(128).fill(11) as TransactProof["b"],
  c: filled(12),
};
const descriptor: ActualOutputDescriptor = {
  ringProgramId: randomAddress(),
  asset: randomAddress(),
  outputTree: TREE,
  registries: [randomAddress()],
  ownerUtxoHash: filled(7),
  privateTxBlinding: filled(9),
  zeroChangeOwnerUtxoHash: filled(8),
  certificate: ringInstructionData(
    measuredFrame({ kind: "splDeposit", amount: 1n, splInterfaceBump: 39 }, false),
  ),
  nativeReservation: { nullifierRootIndex: 0x1234, proof },
};
const request = {
  descriptor,
  authority: randomAddress(),
  reservationPayer: randomAddress(),
  minimumOutput: (1n << 53n) + 3n,
  sourceTokenAccount: randomAddress(),
  tokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
};

describe("runtime amount deposits", () => {
  it("encodes the canonical SPL wire and account roster", async () => {
    const [vault, bump] = await splInterfaceWithBump(descriptor.asset);
    const [owner] = await getProgramDerivedAddress({
      programAddress: descriptor.ringProgramId,
      seeds: ["actual_output_reservation"],
    });
    const { data, accounts } = await buildSettleActualOutputInstruction(request);
    if (data === undefined) throw new Error("missing settlement data");
    const tree = treeAddress(0);
    const slot = ProofInputUtxo.address(ShieldedPublicKey.fromPda(owner), descriptor.ownerUtxoHash);
    expect(data.subarray(0, 2)).toEqual(Uint8Array.of(67, 4));
    expect(new DataView(Uint8Array.from(data).buffer).getBigUint64(2, true)).toBe(
      request.minimumOutput,
    );
    expect(data.subarray(18, 50)).toEqual(descriptor.ownerUtxoHash);
    expect(data.subarray(50, 82)).toEqual(descriptor.privateTxBlinding);
    expect(data.subarray(82, 114)).toEqual(descriptor.zeroChangeOwnerUtxoHash);
    expect(data.subarray(114, 116)).toEqual(Uint8Array.of(1, bump));
    const tail = new Reader(Uint8Array.from(data.subarray(116)));
    const compact = tail.bytes(tail.u16("certificate.length"), "certificate");
    expect(compact).toEqual(compactMeasuredCertificate(descriptor.certificate));
    expect(compact.length).toBe(descriptor.certificate.length - 128);
    expect(tail.u16("nullifier root index")).toBe(0x1234);
    expect(tail.bytes(192, "reservation proof")).toEqual(
      Uint8Array.of(...proof.a, ...proof.b, ...proof.c),
    );
    tail.done();
    expect(accounts).toEqual([
      { address: await ringPolicyAddress(descriptor.ringProgramId), role: AccountRole.READONLY },
      { address: descriptor.registries[0], role: AccountRole.READONLY },
      { address: request.reservationPayer, role: AccountRole.WRITABLE_SIGNER },
      { address: SYSTEM_PROGRAM, role: AccountRole.READONLY },
      { address: tree, role: AccountRole.WRITABLE },
      { address: await nullifierPdaAddress(tree, slot.nullifier()), role: AccountRole.WRITABLE },
      { address: owner, role: AccountRole.READONLY },
      { address: descriptor.outputTree, role: AccountRole.WRITABLE },
      { address: request.authority, role: AccountRole.WRITABLE_SIGNER },
      { address: await ringAuthAddress(descriptor.ringProgramId), role: AccountRole.READONLY },
      { address: SHIELDED_POOL_PROGRAM_ID, role: AccountRole.READONLY },
      { address: request.tokenProgram, role: AccountRole.READONLY },
      { address: descriptor.asset, role: AccountRole.READONLY },
      { address: request.sourceTokenAccount, role: AccountRole.WRITABLE },
      { address: vault, role: AccountRole.WRITABLE },
    ]);
  });

  it.each([
    { kind: "solDeposit", amount: 1n },
    { kind: "splDeposit", amount: 1n, splInterfaceBump: 220 },
  ] as const)("omits only the stock proof bytes for $kind with and without an own root", (leg) => {
    for (const withRecords of [false, true]) {
      const frame = measuredFrame(leg, withRecords);
      const full = ringInstructionData(frame);
      const compact = compactMeasuredCertificate(full);
      const afterProof = frame.nullifiers.length * 32 + (withRecords ? 9 : 4);
      const offset = full.length - afterProof - PLAIN_GROTH16_PROOF_LENGTH;
      expect(compact).toEqual(
        concatBytes(full.subarray(0, offset), full.subarray(offset + PLAIN_GROTH16_PROOF_LENGTH)),
      );
      expect(compact.length).toBe(full.length - 128);
    }
  });

  it("rejects certificates with a nonzero proof, omitted bytes in the wrong section, or noncanonical framing", () => {
    const frame = measuredFrame({ kind: "solDeposit", amount: 1n }, false);
    const full = ringInstructionData(frame);
    const nonzero = frame.sppProof.slice();
    nonzero[0] = 1;
    expect(() =>
      compactMeasuredCertificate(ringInstructionData({ ...frame, sppProof: nonzero })),
    ).toThrow();
    expect(() =>
      compactMeasuredCertificate(ringInstructionData({ ...frame, noSpend: false })),
    ).toThrow();
    const misplaced = concatBytes(full.subarray(0, 1), full.subarray(129));
    const extensionsAt = 1 + 5 + CUSTOM_RING_PROOF_LENGTH + 8 + 4 + 32 + 16;
    const redundantExtension = concatBytes(
      full.subarray(0, extensionsAt),
      Uint8Array.of(0),
      full.subarray(extensionsAt),
    );
    redundantExtension[1] = (redundantExtension[1] ?? 0) | 0x80;
    for (const invalid of [full.subarray(0, -1), misplaced, redundantExtension]) {
      expect(() => compactMeasuredCertificate(invalid)).toThrow();
    }
  });

  it("requests a stock 1×2 proof without a funded input, owner key, or published output", async () => {
    const result = await proveActualOutputReservation(
      {
        proveTransact(inputs: SppProofInputs, keys) {
          expect(keys).toEqual([]);
          expect(inputs.payer).toBe(request.reservationPayer);
          expect(inputs.outputTreeId).toBe(0);
          const [address] = inputs.inputUtxos;
          if (address === undefined) throw new Error("missing reservation");
          expect(inputs.inputUtxos).toHaveLength(1);
          expect(address.isAddress()).toBe(true);
          expect(address.utxo.owner.signatureType()).toBe("pda");
          expect(inputs.outputs.map((slot) => slot.isCompact())).toEqual([true, true]);
          expect(inputs.externalData.outputs).toEqual([]);
          expect(inputs.externalData.txViewingPublicKey.toBytes()).toEqual(
            Uint8Array.of(4, ...new Uint8Array(32)),
          );
          expect(inputs.externalData.expiryUnixTs).toBe(0xffff_ffff_ffff_ffffn);
          return Promise.resolve({
            circuit: { kind: "confidentialSigner", inputs: 1, outputs: 2, publicAssetSlots: 3 },
            inputs: [{ nullifierHash: address.nullifier(), treeIndex: 0 }],
            treeContexts: [{ utxoTreeRootIndex: NO_UTXO_ROOT, nullifierTreeRootIndex: 0x1234 }],
            privateTxHash: inputs.privateTxHash(),
            expiryUnixTs: inputs.externalData.expiryUnixTs,
            txViewingPk: inputs.externalData.txViewingPublicKey.toBytes(),
            salt: inputs.externalData.salt,
            interfaceTransfers: [],
            outputs: [],
            messages: [],
            proof,
          });
        },
      },
      descriptor.ringProgramId,
      descriptor.ownerUtxoHash,
      request.reservationPayer,
    );
    expect(result).toEqual(descriptor.nativeReservation);
  });

  it("rejects a zero minimum or inconsistent asset and reserve", async () => {
    for (const invalid of [
      { ...request, minimumOutput: 0n },
      { ...request, keepLamports: 1n },
      { ...request, tokenProgram: SYSTEM_PROGRAM },
      { ...request, descriptor: { ...descriptor, asset: SOL_MINT } },
    ]) {
      await expect(buildSettleActualOutputInstruction(invalid)).rejects.toMatchObject({
        code: "RING_BUILD_DEPOSIT",
      });
    }
  });
});
