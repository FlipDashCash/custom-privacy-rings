//! Offline parity of one frozen, inert public descriptor statement.
#![forbid(unsafe_code)]

#[path = "../../../program-libs/interface/src/proof/jupiter_measured_descriptor_statement_draft.rs"]
mod statement;

use serde_json::Value;
use spr_hasher::primitives::{hash_bytes, solana_owner_identity};
use statement::DescriptorStatement;

const INPUT: &str =
    include_str!("../../../test-vectors/settlement/jupiter_measured_descriptor_v1_public_input.json");

fn bytes<const N: usize>(value: &Value, field: &str) -> [u8; N] {
    hex::decode(value[field].as_str().expect("synthetic public hex"))
        .expect("valid hex")
        .try_into()
        .expect("exact public width")
}

fn decimal(value: &Value, field: &str) -> u64 {
    value[field]
        .as_str()
        .expect("public decimal word")
        .parse()
        .expect("u64 public word")
}

#[test]
fn prints_one_frozen_public_descriptor_statement() {
    let row: Value = serde_json::from_str(INPUT).expect("shared public parity fixture");
    assert_eq!(
        row["schema"].as_str(),
        Some("pyrol-custom-jupiter-v1-public-statement-parity")
    );
    // This seed is the separately checked V44 host transcript's SHA-256.
    // It is not an authorization to construct a receipt, nor a runtime input.
    let context_digest = bytes::<32>(&row, "checkedContextPreimageSha256");
    let descriptor = DescriptorStatement {
        ring_id: hash_bytes(&bytes::<32>(&row, "ringProgram")).unwrap(),
        auditor_pubkey: bytes(&row, "auditorPublicWord"),
        tracking_root: bytes(&row, "trackingRoot"),
        ring_generation: u32::try_from(decimal(&row, "ringGeneration")).unwrap(),
        operation_context: hash_bytes(&context_digest).unwrap(),
        output_index: 0,
        source_authority_identity: solana_owner_identity(&bytes::<32>(&row, "sourceAuthority")).unwrap(),
        payer_identity: solana_owner_identity(&bytes::<32>(&row, "feePayer")).unwrap(),
        tree_id: bytes(&row, "treeId"),
        asset: hash_bytes(&bytes::<32>(&row, "outputMint")).unwrap(),
        owner_utxo_hash: bytes(&row, "ownerCommitment"),
        min_amount: decimal(&row, "minimumOutputAtoms"),
        max_amount: u64::MAX,
        expiry_slot: decimal(&row, "expirySlot"),
        tx_viewing_pubkey: bytes(&row, "txViewingPublicWord"),
        salt: bytes(&row, "salt"),
        recipient_tag: bytes(&row, "recipientTag"),
        recipient_zero_ciphertext: bytes(&row, "recipientCiphertext"),
        auditor_ciphertext: bytes(&row, "auditorCiphertext"),
    };
    let cells: Vec<u8> = descriptor.elements().unwrap().into_iter().flatten().collect();
    let public_input = descriptor.hash().unwrap();
    println!(
        "PYROL_JUPITER_V1_PUBLIC_PARITY {}.{}",
        hex::encode(cells),
        hex::encode(public_input),
    );
}
