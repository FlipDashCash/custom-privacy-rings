/** Opt-in absent seeded SPL destination for an exact client-reviewed composition.
 * This object authorizes no spend and never fabricates an RPC account. */
import {
  AccountRole,
  createAddressWithSeed,
  isOffCurveAddress,
  type Instruction,
} from "@solana/kit";
import { getCreateAccountWithSeedInstructionDataEncoder } from "@solana-program/system";
import {
  AccountState,
  getTokenDecoder,
  getTokenSize,
  getInitializeAccount3Instruction,
  getMintDecoder,
  getMintSize,
} from "@solana-program/token";
import {
  associatedTokenAddress,
  checkedAddress,
  encodeBase58,
  SYSTEM_PROGRAM,
  treeAddress,
} from "@heliuslabs/spr/internal";
import {
  SPL_TOKEN_PROGRAM_ID,
  type Address,
  type Bytes32,
  type RequestContext,
} from "@heliuslabs/spr/interface";
import type { ChainReader } from "@heliuslabs/spr/client";
import { ringAuthAddress, ringPolicyAddress } from "../../protocol/pda/index.js";
import type { PreparedTransfer, SettlementTransfer } from "@heliuslabs/spr/transaction";

const u64 = (n: bigint) => typeof n === "bigint" && n > 0n && n <= 0xffff_ffff_ffff_ffffn;
const refused = () => new Error("PLANNED_SPL_EXIT_PROFILE_REFUSED");
export interface PlannedSplCompanionWithdrawal {
  readonly recipientOwner: Address;
  readonly recipientTokenAccount: Address;
  readonly amount: bigint;
}
export class PlannedSplExit {
  readonly #companion: PlannedSplCompanionWithdrawal | undefined;
  readonly #payer: Address;
  readonly #base: Address;
  readonly #seed: string;
  readonly #mint: Address;
  readonly #address: Address;
  readonly #rent: bigint;
  private constructor(
    payer: Address,
    base: Address,
    seed: string,
    mint: Address,
    address: Address,
    rent: bigint,
    companion?: PlannedSplCompanionWithdrawal,
  ) {
    this.#companion = companion;
    this.#payer = payer;
    this.#base = base;
    this.#seed = seed;
    this.#mint = mint;
    this.#address = address;
    this.#rent = rent;
    Object.freeze(this);
  }
  static async create(
    input: Readonly<{
      payer: Address;
      base: Address;
      seed: string;
      mint: Address;
      rent: bigint;
      companionWithdrawal?: PlannedSplCompanionWithdrawal;
    }>,
  ): Promise<PlannedSplExit> {
    const payer = checkedAddress(input.payer, "payer"),
      base = checkedAddress(input.base, "base"),
      mint = checkedAddress(input.mint, "mint");
    const length = new TextEncoder().encode(input.seed).length;
    if (
      length === 0 ||
      length > 32 ||
      !u64(input.rent) ||
      payer === base ||
      isOffCurveAddress(payer) ||
      isOffCurveAddress(base) ||
      mint === "So11111111111111111111111111111111111111112"
    )
      throw refused();
    const address = await createAddressWithSeed({
      baseAddress: base,
      programAddress: SPL_TOKEN_PROGRAM_ID,
      seed: input.seed,
    });
    if (!isOffCurveAddress(address)) throw refused();
    const supplied = input.companionWithdrawal;
    const companion =
      supplied === undefined
        ? undefined
        : Object.freeze({
            recipientOwner: checkedAddress(supplied.recipientOwner, "companion owner"),
            recipientTokenAccount: checkedAddress(
              supplied.recipientTokenAccount,
              "companion account",
            ),
            amount: supplied.amount,
          });
    if (
      companion &&
      (!u64(companion.amount) ||
        [payer, base, mint, address].includes(companion.recipientOwner) ||
        [payer, base, mint, address].includes(companion.recipientTokenAccount) ||
        (await associatedTokenAddress(companion.recipientOwner, mint, SPL_TOKEN_PROGRAM_ID)) !==
          companion.recipientTokenAccount)
    )
      throw refused();
    return new PlannedSplExit(payer, base, input.seed, mint, address, input.rent, companion);
  }
  /** Exact opt-in companion owner, never a generic override for other exits. */
  companionOwner(transfer: Extract<SettlementTransfer, { kind: "spl" }>): Address | undefined {
    const leg = this.#companion;
    return leg &&
      !transfer.isDeposit &&
      transfer.mint === this.#mint &&
      transfer.tokenAccount === leg.recipientTokenAccount &&
      transfer.amount === leg.amount
      ? leg.recipientOwner
      : undefined;
  }
  get payer(): Address {
    return this.#payer;
  }
  get base(): Address {
    return this.#base;
  }
  get address(): Address {
    return this.#address;
  }
  get mint(): Address {
    return this.#mint;
  }
  get seed(): string {
    return this.#seed;
  }
  get rent(): bigint {
    return this.#rent;
  }
  instructions(): readonly Instruction[] {
    return Object.freeze([
      Object.freeze({
        programAddress: SYSTEM_PROGRAM,
        accounts: Object.freeze([
          { address: this.#payer, role: AccountRole.WRITABLE_SIGNER },
          { address: this.#address, role: AccountRole.WRITABLE },
          { address: this.#base, role: AccountRole.READONLY_SIGNER },
        ]),
        data: getCreateAccountWithSeedInstructionDataEncoder().encode({
          base: this.#base,
          seed: this.#seed,
          amount: this.#rent,
          space: 165n,
          programAddress: SPL_TOKEN_PROGRAM_ID,
        }),
      }),
      getInitializeAccount3Instruction(
        { account: this.#address, mint: this.#mint, owner: this.#base },
        { programAddress: SPL_TOKEN_PROGRAM_ID },
      ),
    ]);
  }
  async resolveOwner(
    client: Pick<ChainReader, "getAccount">,
    transfer: Extract<SettlementTransfer, { kind: "spl" }>,
    context?: RequestContext,
  ): Promise<Address | undefined> {
    if (
      transfer.isDeposit ||
      transfer.mint !== this.#mint ||
      transfer.tokenAccount !== this.#address
    )
      return undefined;
    if ((await client.getAccount(this.#address, context)) !== undefined) throw refused();
    const mint = await client.getAccount(this.#mint, context);
    if (
      mint === undefined ||
      mint.owner !== SPL_TOKEN_PROGRAM_ID ||
      mint.data.length !== getMintSize() ||
      !getMintDecoder().decode(mint.data).isInitialized
    )
      throw refused();
    return this.#base;
  }
  async validate(
    client: Pick<ChainReader, "getAccount">,
    prepared: PreparedTransfer,
    ring: Address,
    hidden: Bytes32,
    context?: RequestContext,
  ): Promise<void> {
    const { payer, interfaceTransfers: transfers } = prepared;
    const nonzero = (bytes: Bytes32 | undefined) => bytes?.some((byte) => byte !== 0) ?? false;
    const protectedKeys = [
      ring,
      await ringPolicyAddress(ring),
      await ringAuthAddress(ring),
      treeAddress(prepared.outputTreeId),
      ...prepared.inputTreeIds.map(treeAddress),
    ];
    if (
      [this.#payer, this.#base, this.#mint, this.#address].some((key) =>
        protectedKeys.includes(key),
      )
    )
      throw refused();
    if (
      prepared.inputs.some(
        (input) =>
          input.isAddress() ||
          input.cacheSlot !== undefined ||
          (!input.isDummy() && input.utxo.ringProgramId !== ring) ||
          nonzero(input.dataHash) ||
          nonzero(input.ringDataHash) ||
          !input.utxo.data.isEmpty(),
      ) ||
      prepared.outputs.some(
        (output) =>
          (!output.isDummy() && output.ringProgramId !== ring) ||
          output.cacheSlot !== undefined ||
          nonzero(output.dataHash) ||
          nonzero(output.ringDataHash) ||
          !output.data.isEmpty(),
      )
    )
      throw refused();
    const owner = encodeBase58(hidden);
    if (
      payer !== this.#payer ||
      [this.#payer, this.#base, this.#mint, this.#address].includes(owner) ||
      (await associatedTokenAddress(owner, this.#mint, SPL_TOKEN_PROGRAM_ID)) === this.#address
    )
      throw refused();
    // Ordinary callers retain the original one-SPL profile. The explicit
    // companion profile adds exactly one pre-reviewed same-mint canonical ATA
    // withdrawal; it never changes the absent signer-gated destination rules.
    const companion = this.#companion;
    if (transfers.length < (companion ? 2 : 1) || transfers.length > (companion ? 3 : 2))
      throw refused();
    const spl = transfers.filter((leg) => leg.kind === "spl");
    const sol = transfers.filter((leg) => leg.kind === "sol");
    if (
      spl.length !== (companion ? 2 : 1) ||
      sol.length > 1 ||
      sol.some((leg) => leg.isDeposit || !u64(leg.amount) || leg.userSolAccount !== payer)
    )
      throw refused();
    if (companion) {
      const leg = spl[1];
      if (
        transfers[0] !== spl[0] ||
        transfers[1] !== leg ||
        !leg ||
        leg.kind !== "spl" ||
        leg.isDeposit ||
        leg.mint !== this.#mint ||
        leg.tokenAccount !== companion.recipientTokenAccount ||
        leg.amount !== companion.amount ||
        !u64(leg.amount) ||
        protectedKeys.includes(companion.recipientOwner) ||
        protectedKeys.includes(companion.recipientTokenAccount) ||
        companion.recipientOwner === owner ||
        companion.recipientTokenAccount === owner ||
        !u64((spl[0]?.amount ?? 0n) + leg.amount)
      )
        throw refused();
      const account = await client.getAccount(companion.recipientTokenAccount, context);
      if (
        !account ||
        account.owner !== SPL_TOKEN_PROGRAM_ID ||
        account.data.length !== getTokenSize()
      )
        throw refused();
      const token = getTokenDecoder().decode(account.data);
      if (
        token.mint !== this.#mint ||
        token.owner !== companion.recipientOwner ||
        token.state !== AccountState.Initialized ||
        token.delegate.__option !== "None" ||
        token.closeAuthority.__option !== "None" ||
        token.isNative.__option !== "None"
      )
        throw refused();
    }
    const transfer = spl[0];
    if (
      transfer?.kind !== "spl" ||
      !u64(transfer.amount) ||
      (await this.resolveOwner(client, transfer, context)) !== this.#base
    )
      throw refused();
  }
}
