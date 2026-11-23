import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { initializePoseidon } from "@heliuslabs/spr";
import { bytesToHex, hexToBytes } from "@noble/hashes/utils.js";
import type { Bytes32 } from "@heliuslabs/spr/interface";

import rulesVector from "../../../../../test-vectors/policy/ring-policy-rules.json" with { type: "json" };
import { EMPTY_MINT_POLICY } from "../../../src/protocol/membership/mint-policy.js";
import {
  admitTransfer,
  decodeRule,
  decodeRuleTable,
  evaluateInputAssets,
  evaluatePayments,
  evaluateSenders,
  ListId,
  recordBit,
  vouchedBy,
  evaluateVoucher,
  policyTableFields,
  policyTableHash,
  type RecordLists,
} from "../../../src/protocol/membership/policy.js";

await initializePoseidon();

const field = (text: string): Bytes32 => hexToBytes(text) as Bytes32;

interface TableCase {
  readonly name: string;
  readonly sources: readonly Readonly<{ list_id: number; registry: string }>[];
  readonly rules: readonly string[];
  readonly fields: readonly string[];
  readonly policy_hash: string;
}

/** A u64 threshold passes a double, the rows carry it as hex and `rule_parts` is unread. */
const tableVector = JSON.parse(
  readFileSync(
    new URL("../../../../../test-vectors/policy/ring-policy-table.json", import.meta.url),
    "utf8",
  ),
) as Readonly<{ cases: readonly TableCase[] }>;

describe("ring-policy-table.json", () => {
  it.each(tableVector.cases.map((vector) => [vector.name, vector] as const))("%s", (_, vector) => {
    const table = { rules: vector.rules.map((row) => decodeRule(field(row))) };
    const sources = vector.sources.map((slot) => ({
      listId: slot.list_id,
      registry: field(slot.registry),
    }));
    expect(policyTableFields(table, sources).map((row) => bytesToHex(row))).toEqual(vector.fields);
    expect(bytesToHex(policyTableHash(table, sources))).toBe(vector.policy_hash);
  });
});

describe("ring-policy-rules.json", () => {
  type Case = (typeof rulesVector.cases)[number];
  const expected = (outcome: Case["outcome"]) =>
    outcome.kind === "pass"
      ? undefined
      : {
          kind: "lists",
          rule: outcome.rule,
          subject:
            outcome.subject === "sender"
              ? { kind: "sender" }
              : { kind: outcome.subject, output: outcome.output },
        };

  it.each(rulesVector.cases.map((vector) => [vector.name, vector] as const))("%s", (_, vector) => {
    const lookup = (list: readonly { member: string; lists: number }[], member: string): bigint =>
      BigInt(list.find((record) => record.member === member)?.lists ?? 0);
    const lists = (member: string): RecordLists => ({
      own: lookup(vector.own_records, member),
      curator: lookup(vector.curator_records, member),
    });
    const refusal = admitTransfer(
      decodeRuleTable({ rules: vector.rules.map(field) }),
      vector.sources.map((source) => ({
        listId: source.list_id,
        registry: field(source.registry),
      })),
      {
        sender: lists(vector.sender),
        ringInputs: (
          vector.ring_inputs as readonly { owner: string; asset: string; amount: number }[]
        ).map((input) => ({
          owner: field(input.owner),
          asset: field(input.asset),
          amount: BigInt(input.amount),
        })),
        outputs: vector.outputs.map((output) => ({
          ring: output.ring,
          owner: field(output.owner),
          asset: field(output.asset),
          amount: BigInt(output.amount),
          ownerLists: lists(output.owner),
          assetLists: lists(output.asset),
          mintPolicy: {
            ...EMPTY_MINT_POLICY,
            threshold: BigInt(
              vector.inline_limits[
                (vector.inline_assets as readonly string[]).indexOf(output.asset)
              ] ?? 0,
            ),
          },
        })),
      },
    );
    expect(refusal).toEqual(expected(vector.outcome));
  });
});

describe("sender, payment and input asset evaluation", () => {
  const member = (byte: number): Bytes32 => new Uint8Array(32).fill(byte) as Bytes32;
  const sources = [] as const;
  const onBlock = 1n << BigInt(ListId.block - 1);
  const off: RecordLists = { own: 0n, curator: 0n };
  const on: RecordLists = { own: onBlock, curator: 0n };
  const table = (subject: "sender" | "outputOwner" | "asset") =>
    Object.freeze({
      rules: [
        {
          subject,
          source: { kind: "lists" as const, present: [], absent: [ListId.block] },
          guard: { kind: "always" as const },
        },
      ],
    });
  const payment = (ownerLists: RecordLists, assetLists: RecordLists) => ({
    ring: true,
    owner: member(1),
    asset: member(2),
    amount: 5n,
    ownerLists,
    assetLists,
    mintPolicy: EMPTY_MINT_POLICY,
  });
  const noSenders = (subject: "outputOwner" | "asset", lists: RecordLists) =>
    evaluatePayments(table(subject), sources, {
      ringInputs: [],
      outputs: [payment(lists, lists)],
    });

  it("refuses a blocked recipient and asset with no sender row", () => {
    expect(noSenders("outputOwner", on)?.subject).toEqual({ kind: "owner", output: 0 });
    expect(noSenders("asset", on)?.subject).toEqual({ kind: "asset", output: 0 });
    expect(noSenders("outputOwner", off)).toBeUndefined();
    expect(noSenders("asset", off)).toBeUndefined();
  });

  it("holds sender rules to the sender alone", () => {
    expect(evaluateSenders(table("sender"), sources, [off, on])?.subject).toEqual({
      kind: "sender",
    });
    expect(evaluateSenders(table("sender"), sources, [off])).toBeUndefined();
    expect(evaluateSenders(table("sender"), sources, [])).toBeUndefined();
    expect(
      evaluatePayments(table("sender"), sources, { ringInputs: [], outputs: [payment(on, on)] }),
    ).toBeUndefined();
  });

  it("refuses a delisted input asset under an unconditional asset rule alone", () => {
    const inputs = [{ input: 1, lists: on }];
    expect(evaluateInputAssets(table("asset"), sources, inputs)?.subject).toEqual({
      kind: "inputAsset",
      input: 1,
    });
    expect(evaluateInputAssets(table("outputOwner"), sources, inputs)).toBeUndefined();
    const guarded = {
      rules: [{ ...table("asset").rules[0]!, guard: { kind: "aboveAmount" as const, amount: 9n } }],
    };
    expect(evaluateInputAssets(guarded, sources, inputs)).toBeUndefined();
  });
});

describe("a vouched screen", () => {
  const sources = [] as const;
  const bits = (...lists: readonly ListId[]): bigint =>
    lists.reduce((set, list) => set | recordBit(list), 0n);
  const own = (...lists: readonly ListId[]): RecordLists => ({ own: bits(...lists), curator: 0n });
  const off = own();
  const allowed = own(ListId.allow);
  const rule = (subject: "sender" | "outputOwner", present: ListId[], absent: ListId[]) => ({
    subject,
    source: { kind: "lists" as const, present, absent },
    guard: { kind: "always" as const },
  });
  const senders = {
    rules: [rule("sender", [ListId.allow], []), rule("sender", [], [ListId.block])],
  };
  const refusedRule = (rows: readonly RecordLists[], voucher: RecordLists | undefined) => {
    const refusal =
      voucher === undefined
        ? evaluateSenders(senders, sources, rows)
        : (evaluateVoucher(senders, sources, voucher) ??
          evaluateSenders(
            senders,
            sources,
            rows.map((row) => vouchedBy(row, voucher)),
          ));
    return refusal?.kind === "lists" ? refusal.rule : refusal?.kind;
  };

  it("refuses an unlisted sender nobody vouches for", () => {
    expect(refusedRule([off], undefined)).toBe(0);
  });

  it("admits an unlisted sender beside a listed voucher", () => {
    expect(refusedRule([off], allowed)).toBeUndefined();
    expect(refusedRule([off, allowed, off], allowed)).toBeUndefined();
  });

  it("keeps a covered sender's block entry its own", () => {
    expect(refusedRule([own(ListId.block)], allowed)).toBe(1);
  });

  it("refuses a voucher off the allow list though every sender is listed", () => {
    expect(refusedRule([allowed], off)).toBe("voucherUnlisted");
    const blocklist = { rules: [rule("sender", [], [ListId.block])] };
    expect(evaluateVoucher(blocklist, sources, off)).toEqual({ kind: "voucherUnlisted" });
    expect(evaluateVoucher(blocklist, sources, allowed)).toBeUndefined();
  });

  it("refuses a blocked voucher and copies none of its deny entries", () => {
    const blocked = own(ListId.allow, ListId.block, ListId.frozen);
    expect(refusedRule([off], blocked)).toBe(1);
    expect(vouchedBy(off, blocked)).toEqual({ own: bits(ListId.allow), curator: 0n, salted: 0n });
  });

  it("covers the allow and the private allow list alone, in each registry", () => {
    const approval = { rules: [rule("sender", [ListId.approval], [])] };
    const approved = own(ListId.approval);
    expect(evaluateSenders(approval, sources, [vouchedBy(off, approved)])?.rule).toBe(0);
    const every = bits(...Object.values(ListId));
    const covered = bits(ListId.allow, ListId.privateAllow);
    expect(vouchedBy(off, { own: every, curator: every, salted: every })).toEqual({
      own: covered,
      curator: covered,
      salted: covered,
    });
    const held = { own: bits(ListId.frozen), curator: bits(ListId.block), salted: 0n };
    expect(vouchedBy(held, { own: 0n, curator: 0n, salted: bits(ListId.privateAllow) })).toEqual({
      ...held,
      salted: bits(ListId.privateAllow),
    });
  });

  it("admits an unlisted payment owner beside a listed voucher", () => {
    const owners = { rules: [rule("outputOwner", [ListId.allow], [])] };
    const paid = (ownerLists: RecordLists) =>
      evaluatePayments(owners, sources, {
        ringInputs: [],
        outputs: [
          {
            ring: true,
            owner: new Uint8Array(32).fill(1) as Bytes32,
            asset: new Uint8Array(32).fill(2) as Bytes32,
            amount: 5n,
            ownerLists,
            assetLists: off,
            mintPolicy: EMPTY_MINT_POLICY,
          },
        ],
      });
    expect(paid(off)?.subject).toEqual({ kind: "owner", output: 0 });
    expect(paid(vouchedBy(off, allowed))).toBeUndefined();
    expect(paid(vouchedBy(off, off))?.subject).toEqual({ kind: "owner", output: 0 });
  });
});
