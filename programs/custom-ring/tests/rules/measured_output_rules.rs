//! Measured deposits use an amount-one certificate, so amount-dependent Ring rules must refuse it.

use custom_ring_interface::{
    actual_output::{ActualOutputIxData, ACTUAL_OUTPUT_FORMAT},
    tag, CustomRingError, RingPolicy, RootRead,
};
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Signer;
use spr_interface::{
    instruction::{instruction_data::transact::TransactProof, DepositAssetKind},
    SOL_INTERFACE,
};

use crate::common::{policy_pda, refused, ring_auth, Ring, Setup, PROGRAM, SYSTEM};

/// The policy and SPP-program checks run before loading the tree or reading the certificate.
fn measured_sol(ring: &Ring) -> Instruction {
    let request = ActualOutputIxData {
        format: ACTUAL_OUTPUT_FORMAT,
        minimum_output: 1,
        keep_lamports: 0,
        owner_utxo_hash: [7; 32],
        private_tx_blinding: [0; 32],
        zero_change_owner_utxo_hash: [0; 32],
        asset: DepositAssetKind::Sol,
        certificate: Vec::new(),
        nullifier_root_index: 0,
        reservation_proof: TransactProof::zeroed(),
    };
    let (reservation_owner, _) =
        Address::find_program_address(&[b"actual_output_reservation"], &PROGRAM);
    let payer = ring.payer.pubkey();
    let policy = ring.read::<RingPolicy>(&policy_pda());
    let mut accounts = vec![AccountMeta::new_readonly(policy_pda(), false)];
    accounts.extend(ring.registries(RootRead::transfer(&policy, false)));
    accounts.extend([
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(SYSTEM, false),
        AccountMeta::new(payer, false), // Address tree placeholder, never read.
        AccountMeta::new(payer, false), // Nullifier PDA placeholder, never read.
        AccountMeta::new_readonly(reservation_owner, false),
        AccountMeta::new(payer, false), // Tree placeholder, never read.
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(ring_auth(), false),
        AccountMeta::new_readonly(SYSTEM, false), // Deliberately invalid SPP program.
        AccountMeta::new_readonly(SYSTEM, false),
        AccountMeta::new(Address::new_from_array(SOL_INTERFACE), false),
    ]);
    let mut data = vec![tag::SETTLE_ACTUAL_OUTPUT];
    data.extend(wincode::serialize(&request).unwrap());
    Instruction::new_with_bytes(PROGRAM, &data, accounts)
}

#[test]
fn measured_deposit_refuses_an_unknown_format() {
    let mut ring = Ring::new(Setup::BareAnonymous);
    let mut instruction = measured_sol(&ring);
    instruction.data[1] = u8::MAX;
    refused(
        ring.send(instruction, &[]),
        CustomRingError::InvalidInstructionData,
    );
}

#[test]
fn measured_deposit_refuses_confidential_policy_and_non_pristine_records() {
    for setup in [Setup::Bare, Setup::AnonymousPolicy, Setup::Anonymous] {
        let mut ring = Ring::new(setup);
        refused(
            ring.send(measured_sol(&ring), &[]),
            CustomRingError::InvalidSettlementAccounts,
        );
    }
}

#[test]
fn compatible_ring_checks_the_canonical_spp_program_before_reading_source_accounts() {
    let mut ring = Ring::new(Setup::BareAnonymous);
    refused(
        ring.send(measured_sol(&ring), &[]),
        CustomRingError::InvalidShieldedPoolProgram,
    );
}

#[test]
fn measured_deposit_refuses_to_forward_ring_authority_as_depositor() {
    let mut ring = Ring::new(Setup::BareAnonymous);
    let mut instruction = measured_sol(&ring);
    let depositor = instruction.accounts.len() - 5;
    instruction.accounts[depositor] = AccountMeta::new_readonly(ring_auth(), false);
    refused(
        ring.send(instruction, &[]),
        CustomRingError::RingAuthOutOfSlot,
    );
}
