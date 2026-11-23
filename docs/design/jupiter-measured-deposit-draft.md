# Custom measured Jupiter deposit V1 wire draft

This is source for a **disabled and unregistered** Custom instruction: one
legacy SPL ExactIn Jupiter route, one legacy SPL staging account, one protected
output. It defines bytes and account positions, not a working processor. Only
`program-libs/interface/tests/jupiter_measured_deposit_draft.rs` imports the codec.
There is no program or SDK import, verifier key, guard address, trusted loader,
release flag, instruction builder, or runtime call. Nothing is deployed.

## Local wire

The local draft discriminator is byte `68` and version byte `1`. Neither
[Custom #2's registry](https://github.com/FlipDashCash/custom-privacy-rings/blob/4f1f485b756c37ffa4a3f55ba3368991bb4b5686/program-libs/interface/src/instructions/instruction.rs#L58-L73)
nor the
[pinned SPP registry](https://github.com/FlipDashCash/privacy-rings/blob/0616075910dca92a33394ef149611cf9d9e963d2/program-libs/event/src/tag.rs#L1-L45)
assigns `68`. The historical CPMM extension and Custom delivery draft both use
`67`; this draft does not change either. No registry entry is added for `68`.

| Offset (end exclusive) | Field |
| --- | --- |
| 0..2 | Local discriminator and wire version |
| 2..10 | Descriptor expiry slot, little-endian u64 |
| 10..18 | Selected tracking revision, little-endian u64 |
| 18..50 | Selected tracking root |
| 50..82 | Owner UTXO commitment |
| 82..115 | Tagged transaction viewing public key |
| 115..131 | Salt |
| 131..163 | Recipient view tag |
| 163..268 | Encrypted recipient zero-amount record |
| 268..332 | Auditor ciphertext |
| 332..524 | Opaque descriptor proof bytes |

The recipient fields mirror the
[historical descriptor statement V1](https://github.com/FlipDashCash/custom-privacy-rings/blob/f841ff6b9fb454c1228f7d5591735a9536d055e0/program-libs/interface/src/proof/dynamic_descriptor.rs#L33-L55).
Equal proof width does not make the descriptor an ordinary verified transfer.
The 524 bytes contain **no amount, deposit bump, mint override, operation-context
override, guard profile, or verifier key**. This length describes the draft
envelope, not a budget for full Solana message size or compute. A different
layout or descriptor relation must get a new explicit wire version; nothing
automatically negotiates a format or substitutes a verifier.

## Fixed roles

The first six roles reproduce the compiled
[guard endpoint order](https://github.com/FlipDashCash/yardex-markets/blob/8c158858ce45b28becfcdf9d7c8e9318d0516a62/integrations/dynamic-ring-settlement/withdrawal-binding/src/jupiter_prefix_review.rs#L284-L300).
The remaining roles are the Ring policy and tracking inputs, the same original's
payer and fee receiver, the SPP deposit-specific roles and two readonly executable
roles. In order:

```text
0 instructions sysvar  RO        9 fee receiver        RW
1 depositor/stage auth  RO signer 10 SPP tree           RW
2 input source         RW       11 RingConfig           RO
3 output staging       RW       12 SPP program          RO
4 input mint           RO       13 legacy token program RO
5 output mint          RO       14 SPP output vault     RW
6 Ring policy          RO       15 value guard program  RO
7 tracking registry    RO       16 Jupiter program      RO
8 original fee payer   RW signer
```

The stock SPP one-SPL-asset deposit order projects these slots as
`[10, 1, 11, 12, 13, 5, 3, 14]` (tree, depositor, RingConfig, SPP,
token program, mint, staging, vault). The existing
[SPP SPL-deposit validator](https://github.com/FlipDashCash/privacy-rings/blob/0616075910dca92a33394ef149611cf9d9e963d2/programs/shielded-pool/src/instructions/settlement/validate.rs#L133-L151)
requires the depositor to sign but not to be writable. The depositor therefore
remains a **readonly signer**, exactly as the guard expects. The existing
generic mixed-SOL/SPL SDK builder can mark the depositor writable and is
unchanged; it is not the outer wire model here. RingConfig is not a signer in
the outer message. Only the existing authenticated Ring PDA may sign in the
single validated SPP RingConfig slot. These are account positions, not proof
that the account keys, state or executables are valid.

## Runtime integration, not implemented

The program-local `jupiter_legacy_spl_output_draft.rs` now binds the actual
Pinocchio views for the readonly depositor, legacy mint, staging ATA, SPP vault,
and SPL Token program (V1 outer slots 1, 5, 3, 14, and 13). It derives the
canonical addresses and SPP bump from the bound mint and depositor, enforces
initialized non-native classic SPL account layout and owners, and reads the
two exact balances from account data with no retained data borrow. The
[guard's original staging rules](https://github.com/FlipDashCash/yardex-markets/blob/8c158858ce45b28becfcdf9d7c8e9318d0516a62/integrations/jupiter-dynamic-value-guard/src/lib.rs#L69-L97)
and the
[stock SPP SPL settlement checks](https://github.com/FlipDashCash/privacy-rings/blob/0616075910dca92a33394ef149611cf9d9e963d2/programs/shielded-pool/src/instructions/settlement/validate.rs#L93-L152)
remain in force. It is not linked into the Custom
program and has no guard, proof, or custody authority. A checked account
view is not an authenticated instruction or an amount attributable to Jupiter.

### Descriptor context V1 source

`jupiter_measured_descriptor_context_draft.rs` implements this separate
deterministic input to a future descriptor relation. Only its host test imports
it. It reuses the tag-68 role order, descriptor selection, stock `Revision`
and `TransferRoots`, and the pinned SPR hashing primitives. A future selected
caller must obtain the original's first nullifier **and** private transaction
hash from the genuine stock withdrawal binder after the checked withdrawal,
not from a signed message containing this descriptor, instruction assertions,
wallet history or a quote. The current Jupiter prefix check still returns
only unit; obtaining that stock result in the new private caller remains
unimplemented.

Byte order is concatenation with no implicit padding. Integers below are
unsigned little-endian, and every listed word/key/hash is 32 raw bytes:

1. ASCII `pyrol:custom:jupiter:measured-descriptor-context:v1`, one NUL byte,
   then tag byte 68 and version byte 1.
2. Independently selected profile words in order: reviewed profile manifest
   SHA-256, genesis, guard ELF SHA-256, Jupiter ELF SHA-256, descriptor R1CS
   SHA-256, and raw descriptor VK SHA-256. None comes from the instruction.
   Select the canonical manifest before this build; do not hash a manifest
   that includes a digest of its own resulting Custom ELF or this descriptor.
   The exact deployed Custom executable/bank is qualified separately.
   A nonzero word does not verify any of these artifacts.
3. Current Custom program address; stock first nullifier and private
   transaction hash; gross input and approved fee (two u64); canonical tree
   ID and live ring generation (u32).
4. Selected tracking revision (u64), then exact accepted stock roots in order
   policy hash, own root, curator root, and tracking root. The first three are
   zero in the restricted anonymous profile. Use the pinned tracking
   registry's exact served revision at the verified Clock slot; match the
   frame's selected hash, not a replacement current root. Append the frame
   expiry slot (u64) and SPL's representational maximum u64.
5. All 17 V1 outer rows in the fixed order above. Each row is the raw address
   and a byte with writable bit 0 and signer bit 1. These include the exact
   input/output mints, output stage, SPP vault, tree account, payer, depositor,
   fee receiver, and guard/Jupiter program addresses.
6. Exactly 126 bytes each for the authenticated pre- and post-guard, in that
   order, followed by their common compiled roster count (u8) and the ordered
   raw address/privilege rows in the same 33-byte format. The two guard frames
   must already match the exact route and be minimum-output mode. Their first
   six rows match the V1 endpoints, and their checked router digest commits
   the actual compiled Jupiter instruction. The preserved floor is read from
   pre-guard bytes 28..36, with no independently supplied amount or ceiling.

Take standard SHA-256 of these bytes, then the existing SPR `hash_bytes`
at **fixed length 32** of that digest for the canonical `operation_context`
field. The frame's encrypted fields and proof are deliberately excluded to
avoid circular binding. A future reviewed port of
[the descriptor projection](https://github.com/FlipDashCash/custom-privacy-rings/blob/f841ff6b9fb454c1228f7d5591735a9536d055e0/program-libs/interface/src/proof/dynamic_descriptor.rs#L195-L246)
would feed this field to its existing context closure; its other statement
inputs use the same authenticated ring/registry/roles, selected revision, tree,
floor, expiry, and V1 recipient fields. It must verify under the separately
selected **matching** circuit and VK for this Jupiter profile. No part of
historical CPMM verification or proof readiness transfers automatically.
The actual credit remains a runtime measurement, never a transcript input.

The host regression reuses ordinary public V44 guard case 0 and the existing
shared inert tag-68 vector; unprovisioned profile and original fields use
test-only placeholders. The independent SHA-256 digest is
`bd97fc8fcfdc8517bffd217dfef505818ca4b006897ce552efb6f0a9186afdfe`.
No immutable release pins, prover, or verifier are supplied.

```sh
cargo test --offline --locked -p custom-ring-interface --test jupiter_measured_descriptor_context_draft
```

Before wiring an owned Custom handler, independently select and pin the exact
guard and executable profile, Jupiter-specific descriptor transcript and
matching verifier identity. They are **unprovisioned**. Reuse the checked
same-operation prefix and live Ring policy, accepted exact tracking root, and
original payer/depositor authorization. Bind all fixed roles and the actual
current instruction. A successful exact pre-guard establishes an empty stage;
the subsequent dedicated processor must take one observation from these exact
views immediately after the matching post-guard, not accept a balance or
quote in this envelope. The still-unimplemented processor will derive the
amount from that observation and the authenticated zero and floor, verify the
descriptor using the chosen transcript, and use the derived SPP bump in
the one-SPL deposit. The ephemeral depositor stays the same readonly signer;
only the checked stock RingAuth signs at SPP RingConfig slot 2. The processor
must reread the same staging/vault views and require restored staging and
exact custody increase before success. Actual credit accounting is not added
here without the authenticated instruction. Qualify the complete message,
runtime, and independently recoverable output separately before any release.

The account reader is only compiled by a host integration test. It uses an
in-memory `AccountView` with the pinned layout, not a crafted transaction or
real state. It checks ordinary owned balances and stock PDA derivations.

```sh
cargo test --offline --locked -p custom-ring-program --test jupiter_legacy_spl_output_draft
```

For the host codec checks on the complete pinned source (Rust 1.98.1 through
the nested Privacy toolchain):

```sh
cargo test --offline --locked -p custom-ring-interface --test jupiter_measured_deposit_draft
```
