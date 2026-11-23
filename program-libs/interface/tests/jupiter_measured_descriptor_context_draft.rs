#[path = "../src/proof/jupiter_measured_descriptor_context_draft.rs"]
mod context;
#[path = "../src/instructions/jupiter_measured_deposit_draft.rs"]
mod wire;

use context::{
    ContextInput, GuardAccount, MatchedGuards, OriginalWithdrawal, ProfilePins, GUARD_FRAME_BYTES,
    SPL_OUTPUT_MAX,
};
use custom_ring_interface::{storage, TransferRoots, RING_POLICY_PDA_SEED};
use solana_address::Address;
use spr_hasher::primitives::hash_bytes;
use spr_interface::{pda, SHIELDED_POOL_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID};
use wire::{DraftFrame, Role, ROLES, SPP_SPL_DEPOSIT_ROLES};

// Exact V44 public guard shape, ordinary route case 0. These are only
// unsigned guard bytes and packed role rows from that archived fixture. No
// live deployment, stock withdrawal bytes, proof, or wallet data is loaded.
const PRE: &str = concat!(
    "50594a44594e30310000000064000000000000006900000000000000de030000000000000000000000",
    "000000f01d1f0000000000f01d1f0000000000d982139c9c5f7e33ccf35d859ef97a7f4cb07ed7ae14",
    "e8cc324de9d18da1155b19e9963314ee972f395a05d2fee1442b91074bba7e41a2e83ee6d8359d046a",
    "f73200",
);
const POST: &str = concat!(
    "50594a44594e30310100000064000000000000006900000000000000de030000000000000000000000",
    "000000f01d1f0000000000f01d1f0000000000d982139c9c5f7e33ccf35d859ef97a7f4cb07ed7ae14",
    "e8cc324de9d18da1155b19e9963314ee972f395a05d2fee1442b91074bba7e41a2e83ee6d8359d046a",
    "f73200",
);
const GUARD_ROWS: [&str; 11] = [
    "06a7d517187bd16635dad40455fdc2c0c124c68f215675a5dbbacb5f0800000000",
    "030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dc02",
    "10d79126a6a7d8b9a36a55f1e06a25afebd185fc9141b2e9eab140ecca14388101",
    "624222b521d6e169727b2655b9578b5bd58cf9d0702fc1db5a981c2a326c7cdb01",
    "0a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dce300",
    "0b121920272e353c434a51585f666d747b828990979ea5acb3bac1c8cfd6dde400",
    "06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a900",
    "0479d55bf231c06eee74c56ece681507fdb1b2dea3f48e5102b1cda256bc138f00",
    "b43ffa27f5d7f64a74c09b1f295879de4b09ab36dfc9dd514b321aa7b38ce5e800",
    "272e353c434a51585f666d747b828990979ea5acb3bac1c8cfd6dde4ebf2f90500",
    "282f363d444b525960676e757c838a91989fa6adb4bbc2c9d0d7dee5ecf3fa0601",
];

fn bytes<const N: usize>(s: &str) -> [u8; N] {
    hex::decode(s)
        .expect("published fixture hex")
        .try_into()
        .expect("fixed length")
}

#[test]
fn published_guard_role_shape_and_wire_root_feed_the_fixed_descriptor_context() {
    let compact_wire =
        hex::decode(include_str!("fixtures/jupiter_measured_deposit_draft_v1.hex").trim())
            .expect("shared inert wire fixture");
    let frame = DraftFrame::decode(&compact_wire).expect("tag-68/V1 envelope");
    assert_eq!(frame.encode().as_slice(), compact_wire.as_slice());
    let pre = bytes::<GUARD_FRAME_BYTES>(PRE);
    let post = bytes::<GUARD_FRAME_BYTES>(POST);
    let guards = GUARD_ROWS.map(|row| {
        let row = bytes::<33>(row);
        GuardAccount {
            address: Address::new_from_array(row[..32].try_into().expect("role address")),
            is_writable: row[32] & 1 != 0,
            is_signer: row[32] & 2 != 0,
        }
    });

    let ring = Address::new_from_array([0x21; 32]);
    let mut roles: [Address; ROLES.len()] =
        core::array::from_fn(|i| Address::new_from_array([0xa0 + i as u8; 32]));
    for (role, guard) in roles[..6].iter_mut().zip(&guards) {
        *role = guard.address;
    }
    roles[Role::RingPolicy as usize] =
        Address::derive_program_address(&[RING_POLICY_PDA_SEED], &ring)
            .expect("stock ring policy PDA")
            .0;
    roles[Role::RingConfig as usize] =
        Address::new_from_array(pda::ring_auth(&ring.to_bytes().into()).0.to_bytes());
    roles[Role::ShieldedPoolProgram as usize] = Address::new_from_array(SHIELDED_POOL_PROGRAM_ID);
    roles[Role::LegacyTokenProgram as usize] = Address::new_from_array(SPL_TOKEN_PROGRAM_ID);
    let mint = &roles[Role::OutputMint as usize].to_bytes().into();
    let vault = Address::new_from_array(pda::spl_interface(mint).to_bytes());
    roles[SPP_SPL_DEPOSIT_ROLES[7] as usize] = vault;
    roles[Role::ValueGuardProgram as usize] =
        Address::new_from_array(bytes("0d141b222930373e454c535a61686f767d848b9299a0a7aeb5bcc3cad1d8dfe6"));
    roles[Role::JupiterProgram as usize] = guards[7].address;

    // Distinct fixture placeholders for unprovisioned release pins and
    // original proof-bound fields, NOT selected artifact identities.
    let profile = ProfilePins {
        review_profile_sha256: [0x2e; 32],
        genesis: [0x29; 32],
        guard_elf_sha256: [0x2a; 32],
        jupiter_elf_sha256: [0x2b; 32],
        descriptor_r1cs_sha256: [0x2c; 32],
        descriptor_vk_sha256: [0x2d; 32],
    };
    let original = OriginalWithdrawal {
        first_nullifier: [0x27; 32],
        private_tx_hash: [0x28; 32],
        gross_input: 101,
        commercial_fee: 1,
    };
    let accepted_roots = TransferRoots {
        tracking_root: *frame.tracking_root,
        ..TransferRoots::default()
    };
    let input = ContextInput {
        profile: &profile,
        ring_program: &ring,
        roles: &roles,
        guards: MatchedGuards {
            pre: &pre,
            post: &post,
            accounts: &guards,
        },
        original: &original,
        tree_id: &[0x26; 32],
        ring_generation: 7,
    };
    let result = input
        .prepare(&frame, &accepted_roots)
        .expect("inert context projection");
    // Independent SHA256 over the versioned transcript and stock role/PDA
    // data, then the existing SPR fixed-length field hash, not a proof.
    let digest: [u8; 32] = bytes("bd97fc8fcfdc8517bffd217dfef505818ca4b006897ce552efb6f0a9186afdfe");
    assert_eq!(
        result.operation_context,
        hash_bytes(&digest).expect("stock field hash")
    );
    assert_eq!(
        result.tracking_revision.0,
        frame.tracking_revision.to_le_bytes()
    );
    assert_eq!(result.minimum_output, 990);
    assert_eq!(SPL_OUTPUT_MAX, u64::MAX);
}
