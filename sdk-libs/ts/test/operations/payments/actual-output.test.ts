import { AccountRole, getProgramDerivedAddress } from "@solana/kit";
import { describe, expect, it } from "vitest";

import { SHIELDED_POOL_PROGRAM_ID, SPL_TOKEN_2022_PROGRAM_ID } from "@heliuslabs/spr/interface";
import { SYSTEM_PROGRAM, splInterfaceWithBump } from "@heliuslabs/spr/internal";
import { SOL_MINT } from "@heliuslabs/spr/transaction";

import {
  buildSettleActualOutputInstruction,
  type ActualOutputDescriptor,
} from "../../../src/operations/payments/actual-output.js";
import { filled, randomAddress, TREE } from "../../helpers/ring-pipeline.js";

const descriptor: ActualOutputDescriptor = {
  ringProgramId: randomAddress(),
  asset: randomAddress(),
  outputTree: TREE,
  registries: [randomAddress()],
  slot: 1,
  ownerUtxoHash: filled(7),
  privateTxBlinding: filled(9),
  zeroChangeOwnerUtxoHash: filled(8),
  certificate: Uint8Array.of(4, 5, 6),
};
const request = {
  descriptor,
  authority: randomAddress(),
  receiptPayer: randomAddress(),
  minimumOutput: (1n << 53n) + 3n,
  sourceTokenAccount: randomAddress(),
  tokenProgram: SPL_TOKEN_2022_PROGRAM_ID,
};

describe("runtime amount deposits", () => {
  it("matches the canonical SPL wire and separates the source authority from the receipt payer", async () => {
    const [vault, bump] = await splInterfaceWithBump(descriptor.asset);
    const [receipt] = await getProgramDerivedAddress({
      programAddress: descriptor.ringProgramId,
      seeds: ["actual_output", descriptor.ownerUtxoHash],
    });
    const { data, accounts } = await buildSettleActualOutputInstruction(request);
    expect(data?.subarray(0, 3)).toEqual(Uint8Array.of(67, 1, 1));
    expect(data && new DataView(Uint8Array.from(data).buffer).getBigUint64(3, true)).toBe(
      request.minimumOutput,
    );
    expect(data?.slice(115, 147)).toEqual(descriptor.zeroChangeOwnerUtxoHash);
    expect(data?.subarray(147)).toEqual(Uint8Array.of(1, bump, 3, 0, 4, 5, 6));
    expect(accounts?.[2]?.address).toBe(receipt);
    expect(accounts?.[3]).toEqual({
      address: request.receiptPayer,
      role: AccountRole.WRITABLE_SIGNER,
    });
    expect(accounts?.[6]).toEqual({
      address: request.authority,
      role: AccountRole.WRITABLE_SIGNER,
    });
    expect(accounts?.slice(8).map((account) => account.address)).toEqual([
      SHIELDED_POOL_PROGRAM_ID,
      request.tokenProgram,
      descriptor.asset,
      request.sourceTokenAccount,
      vault,
    ]);
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
