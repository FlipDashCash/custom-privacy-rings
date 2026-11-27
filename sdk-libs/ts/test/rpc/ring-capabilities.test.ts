import { describe, expect, it } from "vitest";

import { ClientError } from "@heliuslabs/spr";
import type { ProvingKeyReport } from "@heliuslabs/spr/client";
import { PROVING_KEY_SHA256S } from "@heliuslabs/spr/internal";

import { RingError } from "../../src/protocol/error.js";
import { RING_PROVING_KEY_SHA256S } from "../../src/protocol/proof/proving-keys.js";
import {
  featureKeyNames,
  ringCapabilities,
  type RingCapability,
  type RingCapabilityClient,
  type RingFeature,
} from "../../src/rpc/ring-capabilities.js";

const MERGE_KEYS = ["merge_ring_8_1.key", "merge_ring_24_1.key", "merge_ring_52_1.key"];

/** An unpinned key wins over what the prover serves. */
function pinnedOr(feature: RingFeature, expected: RingCapability): RingCapability {
  const unpinned = featureKeyNames(feature).filter(
    (name) =>
      !Object.hasOwn(RING_PROVING_KEY_SHA256S, name) && !Object.hasOwn(PROVING_KEY_SHA256S, name),
  );
  return unpinned.length > 0
    ? { available: false, reason: "unpinned", keyNames: unpinned }
    : expected;
}

function prover(answer: () => ProvingKeyReport): RingCapabilityClient {
  return { checkRingProvingKeys: () => Promise.resolve().then(answer) };
}

function catalog(names: readonly string[]): ProvingKeyReport {
  return { keys: names.map((name) => ({ name, artifacts: [] })) };
}

describe("ringCapabilities", () => {
  it("maps each feature to the pinned keys the prover serves", async () => {
    const pinned = Object.keys(RING_PROVING_KEY_SHA256S);
    expect(featureKeyNames("merge")).toEqual(["custom_ring_merge.key", ...MERGE_KEYS]);
    expect(featureKeyNames("enroll")).toContain("custom_ring_register_key.key");
    expect(featureKeyNames("enroll")).toContain("custom_ring_apply_enrollments_8.key");
    expect(
      featureKeyNames("deposit").every((name) => featureKeyNames("transfer").includes(name)),
    ).toBe(true);
    expect(featureKeyNames("deposit")).not.toContain("custom_ring_transfer_entry_4_in8_w4_s8.key");
    // Every ring pin belongs to a feature.
    expect(
      pinned.filter(
        (name) =>
          !(["transfer", "merge", "enroll", "applyRecords"] as const).some((feature) =>
            featureKeyNames(feature).includes(name),
          ),
      ),
    ).toEqual([]);

    const full = [...pinned, ...MERGE_KEYS];
    const served = await ringCapabilities(prover(() => catalog(full)));
    for (const feature of ["transfer", "deposit", "enroll", "applyRecords"] as const)
      expect(served[feature], feature).toMatchObject({ available: true });
    expect(served.merge).toEqual(
      pinnedOr("merge", { available: true, keyNames: featureKeyNames("merge") }),
    );
    const ringOnly = await ringCapabilities(prover(() => catalog(pinned)));
    expect(ringOnly.merge).toEqual(
      pinnedOr("merge", { available: false, reason: "missing", keyNames: MERGE_KEYS }),
    );

    const short = await ringCapabilities(
      prover(() => {
        throw new RingError("RING_PROVING_KEYS_MISSING", {
          details: { keyNames: "custom_ring_merge.key,custom_ring_apply_records_private_4.key" },
        });
      }),
    );
    expect(short.merge).toEqual(
      pinnedOr("merge", {
        available: false,
        reason: "missing",
        keyNames: ["custom_ring_merge.key"],
      }),
    );
    expect(short.applyRecords).toMatchObject({
      available: false,
      reason: "missing",
      keyNames: ["custom_ring_apply_records_private_4.key"],
    });
    expect(short.transfer).toMatchObject({ available: true });
    expect(short.enroll).toMatchObject({ available: true });

    const mismatched = await ringCapabilities(
      prover(() => {
        throw new ClientError("CLIENT_PROVER_PROVING_KEYS_MISMATCH", {
          details: { keyNames: "custom_ring_merge.key" },
        });
      }),
    );
    expect(mismatched.merge).toEqual(
      pinnedOr("merge", {
        available: false,
        reason: "mismatched",
        keyNames: ["custom_ring_merge.key"],
      }),
    );
    expect(mismatched.transfer).toMatchObject({
      available: false,
      reason: "unverified",
      code: "CLIENT_PROVER_PROVING_KEYS_MISMATCH",
    });
  });

  it("marks every feature unverified when the prover does not answer and rethrows a caller abort", async () => {
    const down = prover(() => {
      throw new ClientError("CLIENT_PROVER_HTTP", {
        details: { method: "provingKeys", status: 503 },
      });
    });
    const capabilities = await ringCapabilities(down);
    for (const feature of ["transfer", "deposit", "merge", "enroll", "applyRecords"] as const)
      expect(capabilities[feature]).toEqual(
        pinnedOr(feature, {
          available: false,
          reason: "unverified",
          keyNames: featureKeyNames(feature),
          code: "CLIENT_PROVER_HTTP",
        }),
      );

    const controller = new AbortController();
    controller.abort();
    await expect(ringCapabilities(down, { signal: controller.signal })).rejects.toMatchObject({
      code: "CLIENT_PROVER_HTTP",
    });
  });
});
