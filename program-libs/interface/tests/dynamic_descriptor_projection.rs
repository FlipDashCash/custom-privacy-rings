use bytemuck::Zeroable;
use custom_ring_interface::{
    dynamic_descriptor::{
        DescriptorProjection, DescriptorRecord, PreparedDescriptor, ProjectionError, StatementError,
    },
    tracking::TrackingRegistry,
    RingPolicy,
};
use solana_address::Address;
use spr_hasher::primitives::{hash_bytes, solana_owner_identity};

fn key(byte: u8) -> Address {
    Address::new_from_array([byte; 32])
}

fn fixture() -> (
    DescriptorProjection,
    RingPolicy,
    TrackingRegistry,
    DescriptorRecord,
) {
    let route = DescriptorProjection {
        ring_program: key(1),
        tracking_account: key(2),
        tracking_revision: 9.into(),
        tracking_root: [3; 32],
        first_nullifier: [4; 32],
        source_authority: key(5),
        payer: key(6),
        tree_id: [7; 32],
        output_mint: key(8),
        min_amount: (1_u64 << 53) + 5,
        max_amount: u64::MAX - 1,
        expiry_slot: u64::MAX - 2,
    };
    let mut ring = RingPolicy::zeroed();
    ring.tracking = route.tracking_account;
    ring.generation = u32::MAX.to_le_bytes();
    ring.auditor_pubkey = [4; 33];
    let mut registry = TrackingRegistry::zeroed();
    registry.ring_id = hash_bytes(route.ring_program.as_array()).unwrap();
    registry.auditor_pubkey = ring.auditor_pubkey;
    let record = DescriptorRecord {
        owner_utxo_hash: [9; 32],
        tx_viewing_pubkey: [4; 33],
        salt: [10; 16],
        recipient_tag: [0; 32],
        recipient_zero_ciphertext: [11; 105],
        auditor_ciphertext: [12; 64],
    };
    (route, ring, registry, record)
}

// Synthetic application callback for this test, not a venue transcript or ABI.
fn context(input: &DescriptorProjection, application_route: u8) -> Result<[u8; 32], ()> {
    let mut bytes = Vec::new();
    for field in [
        input.ring_program.as_array(),
        input.tracking_account.as_array(),
        &input.tracking_root,
        &input.first_nullifier,
        input.source_authority.as_array(),
        input.payer.as_array(),
        &input.tree_id,
        input.output_mint.as_array(),
    ] {
        bytes.extend_from_slice(field);
    }
    bytes.extend_from_slice(&input.tracking_revision.value().to_le_bytes());
    bytes.extend_from_slice(&input.min_amount.to_le_bytes());
    bytes.extend_from_slice(&input.max_amount.to_le_bytes());
    bytes.extend_from_slice(&input.expiry_slot.to_le_bytes());
    bytes.extend([0, application_route]);
    hash_bytes::<290>(&bytes.try_into().unwrap()).map_err(|_| ())
}

#[test]
fn derives_ring_and_owner_identities_and_the_exact_mint_from_typed_inputs() {
    let (input, ring, tracking, record) = fixture();
    let expected_context = context(&input, 20).unwrap();
    let value = input
        .clone()
        .prepare(&ring, &tracking, &record, |projection| {
            assert_eq!(projection, &input);
            context(projection, 20)
        })
        .unwrap();
    let statement = value.statement();
    assert_eq!(statement.ring_id, tracking.ring_id);
    assert_eq!(statement.auditor_pubkey, ring.auditor_pubkey);
    assert_eq!(statement.ring_generation, u32::MAX);
    assert_eq!(statement.operation_context, expected_context);
    assert_eq!(statement.output_index, 0);
    assert_eq!(
        statement.source_authority_identity,
        solana_owner_identity(input.source_authority.as_array()).unwrap()
    );
    assert_eq!(
        statement.payer_identity,
        solana_owner_identity(input.payer.as_array()).unwrap()
    );
    assert_eq!(
        statement.asset,
        hash_bytes(input.output_mint.as_array()).unwrap()
    );
    assert_eq!(statement.min_amount, input.min_amount);
    assert_eq!(statement.max_amount, input.max_amount);
    assert_eq!(statement.expiry_slot, input.expiry_slot);
    assert_eq!(statement.hash().unwrap(), *value.public_input_hash());
}

#[test]
fn refuses_a_substituted_tracking_address_ring_id_or_auditor_before_context() {
    let (input, ring, tracking, record) = fixture();
    let mut address = input.clone();
    address.tracking_account = key(90);
    let mut other_ring = tracking;
    other_ring.ring_id[31] ^= 1;
    let mut other_auditor = tracking;
    other_auditor.auditor_pubkey[1] ^= 1;
    for (operation, tracker) in [
        (address, &tracking),
        (input.clone(), &other_ring),
        (input, &other_auditor),
    ] {
        assert_eq!(
            operation
                .prepare(&ring, tracker, &record, |_| -> Result<[u8; 32], ()> {
                    panic!("context composer ran on a mismatched ring")
                })
                .unwrap_err(),
            ProjectionError::RingBinding
        );
    }
}

#[test]
fn descriptor_digest_is_frozen_and_tracks_the_same_route_account_mint_and_values() {
    let (mut input, mut ring, tracking, mut record) = fixture();
    let prepared = input
        .clone()
        .prepare(&ring, &tracking, &record, |scope| context(scope, 20))
        .unwrap();
    let initial = *prepared.public_input_hash();
    let baseline = prepared.statement().elements().unwrap();
    let assert_change = |value: &PreparedDescriptor, exact: &[usize]| {
        let modified = value.statement().elements().unwrap();
        let changed: Vec<_> = baseline
            .iter()
            .zip(modified)
            .enumerate()
            .filter_map(|(i, (before, after))| (*before != after).then_some(i))
            .collect();
        assert_eq!(changed, exact);
        assert_ne!(*value.public_input_hash(), initial);
    };

    type Edit = (fn(&mut DescriptorProjection), &'static [usize]);
    let edits: &[Edit] = &[
        (|s| s.source_authority = key(30), &[7, 9]),
        (|s| s.payer = key(31), &[7, 10]),
        (|s| s.output_mint = key(32), &[7, 12]),
        (|s| s.tree_id[31] ^= 1, &[7, 11]),
        (|s| s.tracking_root[31] ^= 1, &[5, 7]),
        (|s| s.tracking_revision = 10.into(), &[7]),
        (|s| s.first_nullifier[31] ^= 1, &[7]),
        (|s| s.min_amount += 1, &[7, 15]),
        (|s| s.max_amount += 1, &[7, 16]),
        (|s| s.expiry_slot += 1, &[7, 17]),
    ];
    for (edit, cells) in edits {
        let mut altered = input.clone();
        edit(&mut altered);
        assert_change(
            &altered
                .prepare(&ring, &tracking, &record, |scope| context(scope, 20))
                .unwrap(),
            cells,
        );
    }
    assert_change(
        &input
            .clone()
            .prepare(&ring, &tracking, &record, |scope| context(scope, 21))
            .unwrap(),
        &[7],
    );
    input.payer = key(99);
    ring.generation = 7u32.to_le_bytes();
    record.recipient_zero_ciphertext[0] ^= 1;
    let other = input
        .prepare(&ring, &tracking, &record, |scope| context(scope, 20))
        .unwrap();
    assert_change(&other, &[6, 7, 10, 22]);
    assert_eq!(prepared.statement().elements().unwrap(), baseline);
    assert_eq!(prepared.statement().hash().unwrap(), initial);
    assert_eq!(*prepared.public_input_hash(), initial);
}

#[test]
fn route_errors_and_invalid_statement_encoding_are_not_outputs() {
    let (mut input, ring, tracking, record) = fixture();
    assert_eq!(
        input
            .clone()
            .prepare(&ring, &tracking, &record, |_| Err("route rejected"))
            .unwrap_err(),
        ProjectionError::Context("route rejected")
    );
    assert_eq!(
        input
            .clone()
            .prepare(&ring, &tracking, &record, |_| Ok::<_, ()>([0; 32]))
            .unwrap_err(),
        ProjectionError::Statement(StatementError::Encoding)
    );
    input.min_amount = u64::MAX;
    assert_eq!(
        input
            .prepare(&ring, &tracking, &record, |scope| context(scope, 20))
            .unwrap_err(),
        ProjectionError::Statement(StatementError::Encoding)
    );
}
