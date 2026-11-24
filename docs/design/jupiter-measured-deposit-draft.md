# Custom measured Jupiter deposit V1 wire draft

This is source for a **disabled and unregistered** Custom instruction: one
legacy SPL ExactIn Jupiter route, one legacy SPL staging account, one protected
output. It defines bytes and account positions, not a working processor. Test
and private review modules import the codec. There is no SDK builder, release
profile, installed verifier or runtime caller. Nothing is deployed.

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
deterministic input to a future descriptor relation. Host tests exercise it;
the no-entrypoint target review typechecks it. It reuses the tag-68 role order,
descriptor selection, stock `Revision`
and `TransferRoots`, and the pinned SPR hashing primitives. A future selected
caller must obtain the original's first nullifier **and** private transaction
hash from the genuine stock withdrawal binder after the checked withdrawal,
not from a signed message containing this descriptor, instruction assertions,
wallet history or a quote. Pyrol's public Jupiter prefix check still returns
only unit. The host-only co-location below retains that result privately;
the authenticated runtime caller remains unimplemented.

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

### Single owned-graph source handoff

The separate `custom-ring-program` host target co-locates the pinned stock
anonymous-withdrawal and Jupiter-prefix check with the actual descriptor
context source. It runs the existing check over the five projected
instructions once, then passes the stock binding's first nullifier and
private transaction hash, with the already-matched gross and fee, into
`ContextInput::original`. The exact matched pre/post guard bytes and metas
are borrowed from that same projection. The private handoff also matches
the reviewed policy/registry, stock RingConfig, original payer/fee receiver
and program roles already named by those checks. Neither the stock binding
nor a new program API is exported. The output is only deterministic
context fields, not a proof, receipt, or amount.

The source graph is owned Custom
`108d011606def666d1552c4a2255994d03db8ae2` / owned Privacy
`0616075910dca92a33394ef149611cf9d9e963d2`, with the unchanged Cargo.lock
blob `4f9bc471770f30ab0f7e759fe73c71badf8b1a53` and program manifest
blob `8c1307ae9c72837465e1c5dea0a31e383fb4d89b`. The stock helper body is
from Pyrol #10 `d8adfe0d55c64ecc9009ba1ee10f2e384e9cbe12`,
`integrations/dynamic-ring-settlement/withdrawal-binding/src/lib.rs`
blob `c87a45de801846b40be9d83e7b4700315d7980ac`; the retained-result
prefix closure is from the exact private-retention candidate
blob `857a87d2b245a71f30f60acc4eaf0a870c4d2117` (patch SHA-256
`48e2606fd818b96c86e21f5ea206509692f95e3e780cf303904a472ce8843572`).
Only the projection closure is copied, not the historical Cargo graph,
runtime sysvar reader, public unit API, or an older verifier. The pinned
withdrawal binder's no-policy/ring/cosign checks are retained verbatim.
The same stock anonymous parser/first-nullifier and account-locator path is
used. The old/owned Custom `transact.rs` blobs
`2442ad4bad6786c93e4fbd1398f54e3ba656207e` and
`ef1a08f1821f3a51d871663d444e4039a1c83f56` differ in delegate handling
and statement fields; the examined `parse`, `owner_signers`, and
`first_nullifier` methods are byte-identical. The owned SPP locator and
SHA-256 / HB32 sources used are blobs
`eb5115b5f9de6076ab5305289acff1147ea71c9d`,
`ac1ce721836e66f6c90b1029e419a67997fcff45`, and
`9cfdf5bc0e9b7ff348cf763ae093f64918e5f889`. This is not a cross-version
proof claim.

The host handoff originally used owned wire/context source blobs
`cc3a30023cb00400aa3a20e970942633711731a8` and
`47bc6ca921bf37c4fb2012b59d529462cbc87afd`. The target review uses the
same context arithmetic with imports from existing target dependencies.
The existing shared wire, stock encoder and its common fixture blobs are
`bb1c97670288318649921682cb2aa3a298ebd02b`,
`625153a2ee76bd38cd007be608b07d62f5029599` and
`043aebfb68a3c63001265cebc1677b02057a5bde`, respectively. The checked-in
unsigned case-0 projection retains only guard/keys/pre/post/router from
public V44 blob `28d1df72dca962e292a2b00502cfe6010663e9db`. It omits
the bank and does not select any live identity. Independent context
preimage SHA-256: `752bba61a4d42985b5d768c62ca362e15fb926a72960af8b65e5f7c048ec7a3d`.

Use the checked-in Rust 1.98.1 selected by the owned Privacy submodule's
`rust-toolchain.toml` (blob `a480cb57e1dcf83e9ed337055d0e3cfb091b7198`).

```sh
CARGO_NET_OFFLINE=true cargo test --offline --locked -p custom-ring-program --test jupiter_measured_v1_source_graph
```

There is no runtime projection in this target. The exact future source
choice, once reviewed, is
[Pinocchio 0.11.2's checked Instructions API](https://github.com/anza-xyz/pinocchio/blob/pinocchio@v0.11.2/sdk/src/sysvars/instructions.rs)
(blob `67c55a68e3321bb4c0648ca177f7b1d87c765397`):
`Instructions::try_from(&AccountView)`, `load_current_index`, and
`load_instruction_at`. For the actual `get_stack_height` check, the owned
lock's [solana-instruction 3.5.1](https://github.com/anza-xyz/solana-sdk/blob/instruction@v3.5.1/instruction/Cargo.toml)
(blob `c22a7b25c2b7bb06e232700e420737e56598af3b`) provides the explicit
`syscalls` feature forwarded to
[v4.0.0's syscall implementation](https://github.com/anza-xyz/solana-sdk/blob/instruction@v4.0.0/instruction/src/syscalls.rs)
(blob `d7a77a3d033fab4329c39b6ddb484b6782885ff7`). The wrapper is
currently a program dev dependency; any runtime declaration or feature
needs separate reviewed wiring.
Do not use a host stub, caller-provided index, fabricated sysvar, or the
historical 3.4.0 graph. The new target does not emulate or prove that runtime
path. Immutable release profiles and matching descriptor artifacts remain
unprovisioned, and no dispatch is installed.

### Dormant descriptor public-input binding

The same host target can now consume the exact checked withdrawal and matched
guard context to reconstruct the **one** descriptor public input. The statement
and projection arithmetic come from the owned
[historical Rust descriptor source](https://github.com/FlipDashCash/custom-privacy-rings/blob/f841ff6b9fb454c1228f7d5591735a9536d055e0/program-libs/interface/src/proof/dynamic_descriptor.rs)
(blob `68ddff0e1d1debaa0c2a7fbcf043dd9bedc03832`). The
[reference Go relation](https://github.com/FlipDashCash/custom-privacy-rings/blob/f841ff6b9fb454c1228f7d5591735a9536d055e0/prover/circuits/dynamicdescriptor/descriptor.go)
(blob `5126dcc69e1e0780e56cb44d127155a0ae7026dc`) binds the canonical
nonzero operation context as a field and uses it in the record construction
and the 25-cell HashChain4. It does not hard-code a venue or that context
digest. This is source reuse, not a claim of artifact or verifier compatibility.
The two checked Custom prover trees differ only by the four historical
descriptor-package files; both pinned Privacy `prover/server` trees are
`2a367cfd058873a5f9f247216e35febae4321c84`. A relation-only build can
therefore start with exact `descriptor.go` in the owned graph; no historical
witness endpoint or key must be carried over to establish its source.

The existing ring-to-tracking and auditor cross-check remains exact. The
host caller derives its projection from the same immutable original, guard
views and V1 roles used for the context, not an independent supplied hash.
Use ordinal zero, the guard's floor and SPL's representational u64 maximum.
Only the existing recipient fields enter the statement; the 192 proof bytes
are not read. Neither the actual measured amount nor new signing rights are
introduced. The positive host case independently assembles all 25 cells
from the existing inert vectors, then compares the single hash using the
pinned SPR primitive.

The frozen wire, policy, program and current proof verifier are unchanged.
A real verifier requires a selected matching Jupiter release relation
compiled against the approved gadget graph, independently reviewed R1CS and
matching verifier fingerprints, and a way to bind that selection to the trusted
profile. The closed historical CPMM artifacts are not that selection.
This path chooses no key, consumes no proof and remains unregistered.


### Offline source provenance and verifier intake

Python 3.12+ tool: `tools/check_jupiter_descriptor_release.py`.

- `reference` emits the exact reviewed source/compiled relation identity as
  canonical JSON. This is a pin, not a local compilation or a verifier.
- `compile` (run from the reviewed checkout) reads the immutable Custom
  `prover` and nested Privacy `prover/server` Git objects into a fresh temporary
  directory. It reruns only `TestJupiterDescriptorRelationConstraints` with
  installed Go 1.27.3 and the offline module cache. Its JSON includes the
  observed byte count, pinned `ConstraintSystem.WriteTo` digest and shape. It
  performs no setup, proving, key loading or network requests.
- `check --manifest FILE --expected-manifest-sha256 SHA --raw-vk FILE` compares
  the exact raw public VK to an **independently approved expected manifest
  digest**. Obtain the digest over separately reviewed release metadata, not
  by hashing the untrusted manifest and then passing that hash back to the
  tool. The tool does not authenticate the issuer or establish the
  relation-to-key linkage by itself.

The manifest has exactly `schema: "jupiter-descriptor-public-vk-release-v1"`,
`relation` (the object emitted by the source tool), `release` (repository
`owner/name` and full Git `commit`), and `public_vk` (`encoding:
"gnark-bn254-groth16.VerifyingKey.WriteRawTo"`, integer `bytes`, and
`sha256`). Use sorted keys, compact JSON and one terminal LF. A full setup
bundle is not raw VK bytes. A successful byte match is not proof execution,
approval of the derived Rust verifier or its runtime profile, or permission
to register the tag.

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

The host account-reader test uses an in-memory `AccountView` with the pinned
layout, not a transaction or real state. It checks ordinary balances, stock
PDA derivations and the typed stock deposit body.

```sh
cargo test --offline --locked -p custom-ring-program --test jupiter_legacy_spl_output_draft
```

### Solana producer dependency check

On `sbpf-solana-solana`, `test-sbf,no-entrypoint` now typechecks the exact
`jupiter_measured_v1_spp_payload_draft.rs` used by the host tests. The private
review module shares the checked graph, descriptor/context sources and
legacy SPL reader. An anonymous compile-time assertion refers only to
function items, including the pinned `RingDepositIxData::serialize`. It has
no call, input instance, export, or connection to a runtime handler. The
existing parent module is excluded with `bpf-entrypoint`, even if both
review features are also set. Without `no-entrypoint`, `test-sbf` still
typechecks the previous reader only.

```sh
CARGO_NET_OFFLINE=true cargo +1.89.0-sbpf-solana-v1.54 check --offline --locked \
  --target sbpf-solana-solana -p custom-ring-program --lib --features test-sbf,no-entrypoint
```

This is a target type/dependency check, not an SBF image build or execution.
No pins are selected, proof consumed, CPI issued, or custody changed.

The same review-only match now retains the output tree's canonical field from
role 10 of its bound account views. It checks the frozen tree and SPP-program
privileges, ownership and exact SPP layout size, then uses pinned
`spr_tree::TreeAccount::from_bytes` for the whole layout rather than trusting
the header alone. The checked in-place parser needs a short exclusive borrow
of the same account; only immutable getters are called, the data is never
written, and the borrow is released before returning. An inactive or
noncanonical PDA cannot replace the bound output tree. No tree ID is exposed
outside this private match. The field is not an append or credit receipt.

The existing `test-sbf` review feature names the already-pinned `spr-tree`
crate directly without enabling its `account-view` feature. The ordinary
dispatch and its dependencies are not extended; `no-entrypoint` gates the new
target reader.

For the host codec checks on the complete pinned source (Rust 1.98.1 through
the nested Privacy toolchain):

```sh
cargo test --offline --locked -p custom-ring-interface --test jupiter_measured_deposit_draft
```
