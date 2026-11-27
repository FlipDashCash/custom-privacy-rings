//! Sweep a signing execution account into a canonical SPP ring deposit. Stock Ring cryptography
//! certifies ownership and auditor access at a fixed synthetic amount; SPP settles the actual amount.
//! This instruction authorizes no withdrawal and never signs for the execution authority. That
//! authority must sign the complete reviewed transaction containing its predecessor withdrawal.
use custom_ring_interface::{
    actual_output::{
        descriptor_hash, descriptor_transaction_hash, pristine_records, sweep_amount,
        ActualOutputIxData, DESCRIPTOR_AMOUNT,
    },
    CustomRingError, InstructionTag, PublicLeg, PublicSlots, RingRail, RingTransactData, RootRead,
    SignerSet,
};
use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, Address, ProgramResult,
};
use spl_token_2022_interface::{
    extension::{BaseStateWithExtensions, ExtensionType, PodStateWithExtensions},
    pod::{PodAccount, PodMint},
};
use spr_hasher::{primitives::hash_bytes, Hasher, Poseidon};
use spr_interface::{
    instruction::{
        tag::RING_DEPOSIT, DepositAssetKind, EncryptedRingDepositData, InterfaceTransfer,
        RingDepositEntry, RingDepositIxData,
    },
    SHIELDED_POOL_PROGRAM_ID, SOL_ASSET_FIELD,
};
use spr_tree::TreeAccount;

use crate::instructions::{
    instruction_runtime::{check_system_program, Pda, PdaAllocation, RingAuth},
    loader::{load_policy, validate_spp_program, RegistrySlots},
    transfer::{accounts::TransactAccounts, verify::RingStatements},
};

#[inline(never)]
pub fn process_actual_output_ix(
    program_id: &Address,
    accounts: &mut [AccountView],
    body: &[u8],
) -> ProgramResult {
    let request = ActualOutputIxData::parse(body)?;
    let invalid = CustomRingError::InvalidSettlementAccounts;
    if accounts.len() < 11 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (policy_account, rest) = accounts.split_at_mut(1);
    let ring = load_policy(program_id, &policy_account[0])?;
    // Amount-one screening cannot certify unknown final-amount limits. A registry is
    // accepted only at its authoritative, never-written sentinel root.
    if ring.mode()? != custom_ring_interface::RingMode::Anonymous
        || ring.policy_hash != [0; 32]
        || ring.co_signers.covered().is_some()
        || request.ring_data_hash != [0; 32]
    {
        return Err(invalid.into());
    }
    let clock = Clock::get()?;
    let read = RootRead::transfer(&ring, false);
    let registry_count = match read {
        RootRead::None => 1,
        RootRead::Own => 2,
        RootRead::Policy => return Err(invalid.into()),
    };
    let (registry_accounts, rest) = rest.split_at_mut(registry_count);
    let (receipt_accounts, spp) = rest.split_at_mut(3);
    let mut after_ring = registry_accounts.iter();
    let registries = RegistrySlots { ring: &ring, read }
        .load(|| after_ring.next().ok_or(ProgramError::NotEnoughAccountKeys))?;
    let roots = registries.roots(clock.slot);
    let ring_id = hash_bytes(program_id.as_array()).map_err(CustomRingError::from)?;
    if roots
        .own
        .is_some_and(|registry| !pristine_records(registry, &ring_id, &ring.tracking))
    {
        return Err(invalid.into());
    }
    let expected_len = match request.asset {
        DepositAssetKind::Sol => 6,
        DepositAssetKind::Spl { .. } => 8,
    };
    if spp.len() != expected_len || !spp[1].is_signer() || !spp[1].is_writable() {
        return Err(invalid.into());
    }
    validate_spp_program(spp)?;
    let ring_auth = RingAuth::derive(program_id)?.check_slot(spp, 2)?;
    let spp_id = Address::new_from_array(SHIELDED_POOL_PROGRAM_ID);
    if spp[3].address() != &spp_id {
        return Err(invalid.into());
    }
    let tree_id = TreeAccount::from_account_view_mut(
        &mut spp[0],
        &spp_id,
        spr_interface::state::discriminator::TREE_ACCOUNT_DISCRIMINATOR,
    )
    .map_err(|_| invalid)?
    .tree_id_array();
    // An unchanged Ring certificate does not bind the source payer (SPP normally supplies that
    // binding). It is public after submission. A permanent one-use receipt prevents a copied
    // certificate from recreating the same owner/blinding commitment at the same final amount.
    check_system_program(&receipt_accounts[2])?;
    let (receipt, receipt_tail) = receipt_accounts.split_at_mut(1);
    let receipt = &mut receipt[0];
    if receipt.data_len() != 0 || !receipt_tail[0].is_signer() {
        return Err(invalid.into());
    }
    let seeds = [
        b"actual_output".as_slice(),
        request.owner_utxo_hash.as_slice(),
    ];
    let pda = Pda {
        program_id,
        seeds: &seeds,
    };
    let bump = pda.canonical(receipt.address()).ok_or(invalid)?;
    pda.allocate(PdaAllocation {
        payer: &receipt_tail[0],
        account: receipt,
        space: 1,
        bump,
    })?;
    receipt.try_borrow_mut()?[0] = 1;
    let (asset, asset_field, amount, leg) = match request.asset {
        DepositAssetKind::Sol => (
            [0; 32],
            SOL_ASSET_FIELD,
            sweep_amount(
                spp[1].lamports(),
                request.keep_lamports,
                request.minimum_output,
            )?,
            InterfaceTransfer::SolDeposit {
                amount: DESCRIPTOR_AMOUNT,
            },
        ),
        DepositAssetKind::Spl { spl_interface_bump } => {
            if request.keep_lamports != 0
                || !spp[5].owned_by(spp[4].address())
                || !spp[6].owned_by(spp[4].address())
            {
                return Err(invalid.into());
            }
            if spp[4].address().as_array() != &spr_interface::SPL_TOKEN_PROGRAM_ID
                && spp[4].address().as_array() != &spr_interface::SPL_TOKEN_2022_PROGRAM_ID
            {
                return Err(invalid.into());
            }
            let mint_bytes = spp[5].try_borrow()?;
            let mint_state =
                PodStateWithExtensions::<PodMint>::unpack(&mint_bytes).map_err(|_| invalid)?;
            if !bool::from(mint_state.base.is_initialized)
                || !mint_state
                    .get_extension_types()
                    .map_err(|_| invalid)?
                    .iter()
                    .all(|extension| {
                        matches!(
                            extension,
                            ExtensionType::MetadataPointer
                                | ExtensionType::TokenMetadata
                                | ExtensionType::GroupPointer
                                | ExtensionType::TokenGroup
                                | ExtensionType::GroupMemberPointer
                                | ExtensionType::TokenGroupMember
                                | ExtensionType::InterestBearingConfig
                                | ExtensionType::ScaledUiAmount
                        )
                    })
            {
                return Err(invalid.into());
            }
            let bytes = spp[6].try_borrow()?;
            let token =
                PodStateWithExtensions::<PodAccount>::unpack(&bytes).map_err(|_| invalid)?;
            if token.base.mint.to_bytes() != spp[5].address().to_bytes()
                || token.base.owner.to_bytes() != spp[1].address().to_bytes()
                || token.base.delegate.is_some()
                || token.base.close_authority.is_some()
                || !token
                    .get_extension_types()
                    .map_err(|_| invalid)?
                    .iter()
                    .all(|extension| matches!(extension, ExtensionType::ImmutableOwner))
            {
                return Err(invalid.into());
            }
            let mint = spp[5].address().to_bytes();
            (
                mint,
                hash_bytes(&mint).map_err(CustomRingError::from)?,
                sweep_amount(u64::from(token.base.amount), 0, request.minimum_output)?,
                InterfaceTransfer::SplDeposit {
                    amount: DESCRIPTOR_AMOUNT,
                    spl_interface_bump,
                },
            )
        }
    };
    let (tag, frame) = request
        .certificate
        .split_first()
        .ok_or(CustomRingError::InvalidInstructionData)?;
    if *tag != InstructionTag::Transfer as u8 {
        return Err(invalid.into());
    }
    let certificate = RingTransactData::parse(frame, RingRail::Anonymous)?;
    let data = certificate.data();
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
    let ring_hash =
        Poseidon::hashv(&[&request.ring_data_hash, &ring_id]).map_err(CustomRingError::from)?;
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
            != &descriptor_transaction_hash(
                &expected_change,
                &expected,
                &request.private_tx_blinding,
            )?
    {
        return Err(invalid.into());
    }
    let public_slots = PublicSlots::fold::<CustomRingError>([Ok(PublicLeg {
        transfer: leg,
        mint: (asset != [0; 32]).then_some(asset),
    })])?;
    let validated = TransactAccounts {
        roots: data.roots(&ring, &roots)?,
        sender: [0; 32],
        delegate_asset: [0; 32],
        signers: SignerSet::default(),
        party_identities: [[0; 32]; custom_ring_interface::ANONYMOUS_PARTIES],
        public_slots,
        withdrawals: Vec::new(),
        spp: 0,
        ring_auth,
    };
    RingStatements {
        program_id,
        ring: &ring,
        transfer: &certificate,
        accounts: &validated,
    }
    .verify()?;
    let mut tx_viewing_pk = [0; 33];
    tx_viewing_pk[0] = custom_ring_interface::VIEWING_PUBKEY_TAG;
    tx_viewing_pk[1..].copy_from_slice(data.tx_viewing_u);
    let deposit = RingDepositIxData {
        assets: vec![request.asset],
        deposits: vec![RingDepositEntry {
            asset_index: 0,
            view_tag: output.owner_tag,
            owner_utxo_hash: request.owner_utxo_hash,
            amount,
            ring_data_hash: request.ring_data_hash,
            encrypted: EncryptedRingDepositData {
                tx_viewing_pk,
                salt: *data.salt,
                ciphertext: request.recovery_body(),
            },
        }],
    };
    let mut forwarded = vec![RING_DEPOSIT];
    forwarded.extend(
        deposit
            .serialize()
            .map_err(|_| CustomRingError::InvalidInstructionData)?,
    );
    // SPP validates real mint/program ownership, canonical interfaces, token extension policy and
    // exact vault credit. A refused/fee-bearing transfer rolls back the whole composition.
    validated.ring_auth.invoke(spp, &forwarded)
}
