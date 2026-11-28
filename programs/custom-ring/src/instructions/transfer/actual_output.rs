//! Sweep a signing execution account into a canonical SPP ring deposit. Stock Ring cryptography
//! certifies ownership and auditor access at a fixed synthetic amount. SPP settles the measured
//! amount. The execution authority signs the complete transaction. The Ring signs for the
//! native reservation and for its SPP deposit authority.
use custom_ring_interface::{
    actual_output::ActualOutputIxData, storage::StorageRegistry, CustomRingError, PublicLeg,
    PublicSlots, RingMode, RootRead, SignerSet, ANONYMOUS_PARTIES, MEMBER_REGISTRY_EMPTY_ROOT,
    VIEWING_PUBKEY_TAG,
};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};
use spl_token_2022_interface::{extension::PodStateWithExtensions, pod::PodAccount};
use spr_hasher::{
    primitives::{hash_bytes, right_align},
    Hasher, Poseidon,
};
use spr_interface::{
    instruction::{
        instruction_data::transact::{CircuitId, InputUtxo, TreeContext, NO_UTXO_ROOT},
        tag::RING_DEPOSIT,
        DepositAssetKind, EncryptedRingDepositData, InterfaceTransfer, RingDepositEntry,
        RingDepositIxData,
    },
    pda,
    state::discriminator::TREE_ACCOUNT_DISCRIMINATOR,
    N_PUBLIC_SLOTS, SOL_ASSET_FIELD, UTXO_DOMAIN,
};
use spr_program::{
    cpi::SppTransactAccounts,
    derive_private_tx_blinding,
    program::{AddressSeed, NewAddress, PdaOwner, ACCOUNT_BLINDING_SEED},
    PrivateTxHash, TransactExternalData, TransactInputs, ADDRESS_TREE_ID,
};
use spr_tree::TreeAccount;
use zolana_ring_policy::ring_id_field;

use super::{accounts::TransactAccounts, verify::RingStatements};
use crate::instructions::{
    instruction_runtime::{check_system_program, Pda, RingAuth},
    loader::{load_policy, validate_spp_program, RegistrySlots},
};

const DESCRIPTOR_AMOUNT: u64 = 1;
const RESERVATION_OWNER_SEED: &[u8] = b"actual_output_reservation";

#[inline(never)]
pub fn process_actual_output_ix(
    program_id: &Address,
    accounts: &mut [AccountView],
    body: &[u8],
) -> ProgramResult {
    let request = ActualOutputIxData::parse(body)?;
    let invalid = CustomRingError::InvalidSettlementAccounts;
    let (policy_account, rest) = accounts
        .split_first_mut()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let ring = load_policy(program_id, policy_account)?;
    // The proof certifies amount one. Admit audit-only rings and require the authoritative,
    // never-written sentinel for any own records registry.
    if ring.mode()? != RingMode::Anonymous || ring.co_signers.covered().is_some() {
        return Err(invalid.into());
    }
    let clock = Clock::get()?;
    let read = RootRead::transfer(&ring, false);
    let registry_count = match read {
        RootRead::None => 1,
        RootRead::Own => 2,
        RootRead::Policy => return Err(invalid.into()),
    };
    let (registry_accounts, rest) = rest
        .split_at_mut_checked(registry_count)
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let (reservation, spp) = rest
        .split_first_chunk_mut::<5>()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let mut after_ring = registry_accounts.iter();
    let registries = RegistrySlots { ring: &ring, read }
        .load(|| after_ring.next().ok_or(ProgramError::NotEnoughAccountKeys))?;
    let roots = registries.roots(clock.slot);
    let ring_id = ring_id_field(program_id.as_array()).map_err(CustomRingError::from)?;
    if roots
        .own
        .is_some_and(|registry| !pristine_records(registry, &ring_id, &ring.tracking))
    {
        return Err(invalid.into());
    }
    // SPP's ring_deposit prefix is followed by exactly one settlement group.
    let (mint, asset_field, leg) = match (request.asset, spp.get(4..)) {
        (DepositAssetKind::Sol, Some([_, _])) => (
            None,
            SOL_ASSET_FIELD,
            InterfaceTransfer::SolDeposit {
                amount: DESCRIPTOR_AMOUNT,
            },
        ),
        (DepositAssetKind::Spl { spl_interface_bump }, Some([_, mint, _, _])) => {
            let mint = mint.address().to_bytes();
            (
                Some(mint),
                hash_bytes(&mint).map_err(CustomRingError::from)?,
                InterfaceTransfer::SplDeposit {
                    amount: DESCRIPTOR_AMOUNT,
                    spl_interface_bump,
                },
            )
        }
        _ => return Err(invalid.into()),
    };
    let ring_auth = RingAuth::derive(program_id)?.check_slot(spp, 2)?;
    let [tree, authority, _, spp_program, asset_accounts @ ..] = spp else {
        return Err(invalid.into());
    };
    validate_spp_program(core::slice::from_ref(spp_program))?;
    let tree_id =
        TreeAccount::from_account_view_mut(tree, spp_program.address(), TREE_ACCOUNT_DISCRIMINATOR)
            .map_err(|_| invalid)?
            .tree_id_array();
    let certificate = request.parse_certificate()?;
    let transfer = certificate.transact();
    let data = transfer.data();
    if !data.no_spend
        || !data.public_owners
        || data.has_data
        || data.ring_data
        || data.entry
        || data.vouched
        || !data.approval.is_empty()
        || data.own_revision.map(|revision| revision.value()) != roots.own.map(|_| 0)
        || data.interface_transfers.as_slice() != [leg]
        || i64::from(data.expiry_unix_ts) < clock.unix_timestamp
    {
        return Err(invalid.into());
    }
    let [change, output] = data.outputs.as_slice() else {
        return Err(invalid.into());
    };
    if change.is_exit() || output.is_exit() {
        return Err(invalid.into());
    }
    let ring_hash = Poseidon::hashv(&[&[0; 32], &ring_id]).map_err(CustomRingError::from)?;
    let expected_change = descriptor_hash(
        &tree_id,
        &SOL_ASSET_FIELD,
        &ring_hash,
        &request.zero_change_owner_utxo_hash,
        0,
    )?;
    let expected = descriptor_hash(
        &tree_id,
        &asset_field,
        &ring_hash,
        &request.owner_utxo_hash,
        DESCRIPTOR_AMOUNT,
    )?;
    if change.hash != &expected_change
        || output.hash != &expected
        || data.private_tx_hash
            != &PrivateTxHash::new(
                &[],
                &[expected_change, expected],
                &request.private_tx_blinding,
            )
            .hash()
            .map_err(CustomRingError::from)?
    {
        return Err(invalid.into());
    }
    let public_slots = PublicSlots::fold::<CustomRingError>([Ok(PublicLeg {
        transfer: leg,
        mint,
    })])?;
    let validated = TransactAccounts {
        roots: data.roots(&ring, &roots)?,
        sender: [0; 32],
        delegate_asset: [0; 32],
        signers: SignerSet::default(),
        party_identities: [[0; 32]; ANONYMOUS_PARTIES],
        public_slots,
        withdrawals: Vec::new(),
        spp: 0,
        ring_auth,
    };
    RingStatements {
        program_id,
        ring: &ring,
        transfer,
        accounts: &validated,
    }
    .verify()?;
    reserve_actual_output(program_id, reservation, spp_program, &request)?;
    let balance = match asset_accounts {
        [_, _, source, _] => {
            // SPP checks the source and requires the vault to receive this exact base amount.
            let bytes = source.try_borrow()?;
            let token =
                PodStateWithExtensions::<PodAccount>::unpack(&bytes).map_err(|_| invalid)?;
            token.base.amount.into()
        }
        // The SOL source can also be the address tree fee payer.
        _ => authority.lamports(),
    };
    let amount = balance
        .checked_sub(request.keep_lamports)
        .filter(|amount| *amount >= request.minimum_output)
        .ok_or(invalid)?;
    let mut tx_viewing_pk = [0; 33];
    tx_viewing_pk[0] = VIEWING_PUBKEY_TAG;
    tx_viewing_pk[1..].copy_from_slice(data.tx_viewing_u);
    let deposit = RingDepositIxData {
        assets: vec![request.asset],
        deposits: vec![RingDepositEntry {
            asset_index: 0,
            view_tag: output.owner_tag,
            owner_utxo_hash: request.owner_utxo_hash,
            amount,
            ring_data_hash: [0; 32],
            encrypted: EncryptedRingDepositData {
                tx_viewing_pk,
                salt: *data.salt,
                ciphertext: certificate.recovery_body(),
            },
        }],
    };
    let mut forwarded = vec![RING_DEPOSIT];
    forwarded.extend(
        deposit
            .serialize()
            .map_err(|_| CustomRingError::InvalidInstructionData)?,
    );
    // SPP checks its active ring config, the canonical interface and exact SPL vault credit.
    // A failed CPI rolls back settlement and the reservation with the rest of the transaction.
    validated.ring_auth.invoke(spp, &forwarded)
}

/// Amount-dependent rules must use the live, never-written empty root.
fn pristine_records(registry: &StorageRegistry, ring_id: &[u8; 32], tracking: &Address) -> bool {
    let newest = registry.newest();
    registry.ring_id == *ring_id
        && registry.tracking_registry == *tracking
        && registry.tracking_floor.value() == 0
        && registry.next_index() == 1
        && newest.revision.value() == 0
        && newest.expires_at() == 0
        && newest.hash == MEMBER_REGISTRY_EMPTY_ROOT
}

/// The stock SPP commitment for a descriptor with no application data.
fn descriptor_hash(
    tree_id: &[u8; 32],
    asset: &[u8; 32],
    ring_hash: &[u8; 32],
    owner_utxo_hash: &[u8; 32],
    amount: u64,
) -> Result<[u8; 32], CustomRingError> {
    Ok(Poseidon::hashv(&[
        &right_align(&UTXO_DOMAIN.to_be_bytes()),
        tree_id,
        asset,
        &right_align(&amount.to_be_bytes()),
        &[0; 32],
        ring_hash,
        owner_utxo_hash,
    ])?)
}

#[inline(never)]
fn reserve_actual_output(
    program_id: &Address,
    reservation: &[AccountView; 5],
    spp_program: &AccountView,
    request: &ActualOutputIxData,
) -> ProgramResult {
    let [payer, system, tree, nullifier_pda, owner] = reservation;
    let invalid = CustomRingError::InvalidSettlementAccounts;
    check_system_program(system)?;
    if !payer.is_signer() || !payer.is_writable() {
        return Err(invalid.into());
    }
    let reservation_owner = Pda {
        program_id,
        seeds: &[RESERVATION_OWNER_SEED],
    };
    let bump = reservation_owner
        .canonical(owner.address())
        .ok_or(invalid)?;
    let owner_pda = PdaOwner::new(owner.address()).map_err(|_| invalid)?;
    // A single namespace covers this commitment regardless of payer, asset, amount or output tree.
    let seed = AddressSeed::new(request.owner_utxo_hash).map_err(|_| invalid)?;
    let address = NewAddress::derive(&owner_pda, seed).map_err(|_| invalid)?;
    let nullifier = *address.address();
    let canonical_tree = pda::tree(ADDRESS_TREE_ID);
    if tree.address().as_array() != canonical_tree.as_array()
        || nullifier_pda.address().as_array()
            != pda::nullifier_pda(&canonical_tree, &nullifier).0.as_array()
    {
        return Err(invalid.into());
    }

    let blinding = derive_private_tx_blinding(&nullifier, &ACCOUNT_BLINDING_SEED)
        .map_err(CustomRingError::from)?;
    let private_tx_hash = PrivateTxHash {
        input_hashes: &[],
        output_hashes: &[],
        address_nullifiers: Some(&[nullifier]),
        blinding: &blinding,
    }
    .hash()
    .map_err(CustomRingError::from)?;
    let mut external = TransactExternalData::from_outputs(Vec::new());
    // The typed SDK encodes a tagged viewing key. This outputless transaction does no encryption.
    external.tx_viewing_pk[0] = VIEWING_PUBKEY_TAG;
    let data = external
        .into_ix_data(
            private_tx_hash,
            CircuitId::ConfidentialSigner(1, 2, N_PUBLIC_SLOTS as u8),
            request.reservation_proof,
            TransactInputs {
                inputs: vec![InputUtxo {
                    nullifier_hash: nullifier,
                    tree_index: 0,
                }],
                tree_contexts: vec![TreeContext {
                    utxo_tree_root_index: NO_UTXO_ROOT,
                    nullifier_tree_root_index: request.nullifier_root_index,
                }],
            },
        )
        .serialize()
        .map_err(|_| CustomRingError::InvalidInstructionData)?;
    let accounts = [
        *payer,
        *tree,
        *spp_program,
        *system,
        *tree,
        *nullifier_pda,
        *owner,
    ];
    let signer_pdas = [owner.address()];
    let cpi = SppTransactAccounts::new(&accounts, &signer_pdas).map_err(|_| invalid)?;
    let bump_seed = [bump];
    let seeds = [Seed::from(RESERVATION_OWNER_SEED), Seed::from(&bump_seed)];
    cpi.invoke::<7>(&data, &[Signer::from(&seeds)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use custom_ring_interface::{storage::RegistryId, storage_instruction::InitializeIxData};

    #[test]
    fn requires_pristine_current_registry_even_when_historical_empty_root_is_seen() {
        let ring_id = [7; 32];
        let tracking = Address::new_from_array([8; 32]);
        let pristine = StorageRegistry::new(
            Address::new_from_array([9; 32]),
            &InitializeIxData {
                registry_id: RegistryId([6; 32]),
                grace_slots: 400u64.to_le_bytes(),
                ring_id,
                tracking_registry: tracking,
            },
        );
        assert!(pristine_records(&pristine, &ring_id, &tracking));
        assert!(!pristine_records(&pristine, &[1; 32], &tracking));
        assert!(!pristine_records(&pristine, &ring_id, &Address::default()));

        let mut revised = pristine;
        revised.roots[0].revision = 1.into();
        revised.roots[1] = pristine.roots[0];
        assert!(!pristine_records(&revised, &ring_id, &tracking));
        let mut nonempty = pristine;
        nonempty.roots[0].hash = [1; 32];
        assert!(!pristine_records(&nonempty, &ring_id, &tracking));
        let mut allocated = pristine;
        allocated.next_index = 2u64.to_le_bytes();
        assert!(!pristine_records(&allocated, &ring_id, &tracking));
    }

    #[test]
    fn descriptor_commitments_match_stock_transaction_hashing() {
        use spr_hasher::primitives::hash_bytes;
        use spr_interface::tree_slot::tree_id_field;
        use spr_keypair::{shielded::ShieldedKeypair, signing_key::SigningKey};
        use spr_transaction::{utxo::owner_utxo_hash, Mint, SppProofOutputUtxo};

        let owner = ShieldedKeypair::from_keypair(SigningKey::from_ed25519_bytes(&[3; 32]))
            .unwrap()
            .shielded_address()
            .unwrap();
        let ring = Address::new_from_array([4; 32]);
        let ring_hash =
            Poseidon::hashv(&[&[0; 32], &hash_bytes(ring.as_array()).unwrap()]).unwrap();
        let mint = Mint::new(Address::new_from_array([5; 32]), 1);
        let tree = 258;
        let build = |asset: Mint, amount, blind: u8| {
            let output = SppProofOutputUtxo {
                asset,
                amount,
                blinding: right_align(&[blind]),
                ring_program_id: Some(ring),
                owner_address: Some(owner),
                ..Default::default()
            };
            let hash = descriptor_hash(
                &tree_id_field(tree),
                &hash_bytes(asset.asset.as_array()).unwrap(),
                &ring_hash,
                &owner_utxo_hash(&owner.owner_hash().unwrap(), &output.blinding).unwrap(),
                amount,
            )
            .unwrap();
            assert_eq!(hash, output.hash(tree).unwrap());
            hash
        };
        let change = build(Mint::SOL, 0, 6);
        let output = build(mint, DESCRIPTOR_AMOUNT, 7);
        let blinding = right_align(&[8]);
        let outputs = Poseidon::hashv(&[&change, &output]).unwrap();
        assert_eq!(
            PrivateTxHash::new(&[], &[change, output], &blinding)
                .hash()
                .unwrap(),
            Poseidon::hashv(&[&[0; 32], &outputs, &[0; 32], &blinding]).unwrap()
        );
    }
}
