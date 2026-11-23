#[path = "../src/instructions/jupiter_measured_deposit_draft.rs"]
mod wire;

use custom_ring_interface::{CustomRingError, CustomRingProof, InstructionTag};
use wire::{
    Access, DraftFrame, FrameError, Role, DRAFT_TAG, DRAFT_VERSION, FRAME_LEN, PROOF_LEN, ROLES,
    SPP_SPL_DEPOSIT_ROLES,
};

#[test]
fn preview_tag_is_not_registered_by_either_program() {
    assert_eq!(DRAFT_TAG, 68);
    assert_eq!(DRAFT_VERSION, 1);
    assert_eq!(
        InstructionTag::try_from(DRAFT_TAG),
        Err(CustomRingError::InvalidInstructionData)
    );
    assert!(spr_interface::instruction::InstructionTag::try_from(DRAFT_TAG).is_err());
    assert_eq!(PROOF_LEN, CustomRingProof::SIZE);
}

#[test]
fn fixed_wire_matches_the_inert_v1_vector() {
    // All fields are inert; the all-zero proof here is not a valid proof.
    let mut viewing_key = [0; 33];
    viewing_key[..2].copy_from_slice(&[4, 9]);
    let mut view_tag = [0; 32];
    view_tag[24..].fill(0x55);
    let frame = DraftFrame {
        expiry_slot: 0x0123_4567_89ab_cdef,
        tracking_revision: 0x1020_3040_5060_7080,
        tracking_root: &[0x11; 32],
        owner_utxo_hash: &[0x22; 32],
        tx_viewing_pubkey: &viewing_key,
        salt: &[0x44; 16],
        recipient_tag: &view_tag,
        recipient_zero_ciphertext: &[0x66; 105],
        auditor_ciphertext: &[0x77; 64],
        descriptor_proof_bytes: &[0; 192],
    };
    let fixture = include_str!("fixtures/jupiter_measured_deposit_draft_v1.hex").trim();
    assert_eq!(fixture.len(), FRAME_LEN * 2);
    let mut expected = [0; FRAME_LEN];
    for (index, byte) in expected.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&fixture[index * 2..index * 2 + 2], 16)
            .expect("inert v1 draft fixture hex");
    }

    assert_eq!(frame.encode(), expected);
    let parsed = DraftFrame::decode(&expected).expect("inert v1 draft bytes");
    assert!(parsed == frame);
    assert_eq!(parsed.encode(), expected);
}

#[test]
fn version_and_wire_length_are_not_inferred() {
    let mut frame = [0; FRAME_LEN + 1];
    frame[0] = DRAFT_TAG;
    frame[1] = DRAFT_VERSION;
    assert!(matches!(DraftFrame::decode(&frame), Err(FrameError::Length)));
    frame[1] = DRAFT_VERSION + 1;
    assert!(matches!(
        DraftFrame::decode(&frame[..FRAME_LEN]),
        Err(FrameError::Header)
    ));
}

#[test]
fn fixed_account_roles_project_guard_endpoints_and_stock_spp_slots() {
    assert!(ROLES
        .iter()
        .enumerate()
        .all(|(index, (role, _))| *role as usize == index));
    assert_eq!(
        &ROLES[..6],
        &[
            (Role::Instructions, Access::ReadOnly),
            (Role::Depositor, Access::ReadOnlySigner),
            (Role::InputSource, Access::Writable),
            (Role::OutputStaging, Access::Writable),
            (Role::InputMint, Access::ReadOnly),
            (Role::OutputMint, Access::ReadOnly),
        ]
    );
    assert_eq!(
        SPP_SPL_DEPOSIT_ROLES.map(|role| role as usize),
        [10, 1, 11, 12, 13, 5, 3, 14]
    );
    assert_eq!(
        SPP_SPL_DEPOSIT_ROLES.map(|role| ROLES[role as usize].1),
        [
            Access::Writable,
            Access::ReadOnlySigner,
            Access::ReadOnly,
            Access::ReadOnly,
            Access::ReadOnly,
            Access::ReadOnly,
            Access::Writable,
            Access::Writable,
        ]
    );
    assert_eq!(ROLES[8], (Role::FeePayer, Access::WritableSigner));
}
