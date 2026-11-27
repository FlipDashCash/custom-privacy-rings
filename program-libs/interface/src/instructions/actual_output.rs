//! Versioned actual-output settlement. The certificate is an unchanged stock Ring proof, never a
//! spend authorization. Its one synthetic amount-one output certifies an encrypted ownership
//! descriptor; the SPP event supplies the only runtime amount used by recovery.
use solana_address::Address;
use spr_hasher::{Hasher, Poseidon};
use spr_interface::instruction::DepositAssetKind;
use wincode::{containers, len::FixIntLen, SchemaRead, SchemaWrite};

use crate::{storage::StorageRegistry, CustomRingError, MEMBER_REGISTRY_EMPTY_ROOT};

pub const ACTUAL_OUTPUT_VERSION: u8 = 1;
pub const ACTUAL_OUTPUT_MAGIC: &[u8; 8] = b"CRADv001";
pub const DESCRIPTOR_AMOUNT: u64 = 1;
pub const DESCRIPTOR_SLOT: u8 = 1;

/// A live, never-written registry can hold no limits. An older empty root is
/// insufficient, even when normally usable for in-flight proofs.
pub fn pristine_records(
    registry: &StorageRegistry,
    ring_id: &[u8; 32],
    tracking: &Address,
) -> bool {
    let newest = registry.newest();
    registry.ring_id == *ring_id
        && registry.tracking_registry == *tracking
        && registry.tracking_floor.value() == 0
        && registry.next_index() == 1
        && newest.revision.value() == 0
        && newest.expires_at() == 0
        && newest.hash == MEMBER_REGISTRY_EMPTY_ROOT
}

#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub struct ActualOutputIxData {
    pub version: u8,
    pub descriptor_slot: u8,
    pub minimum_output: u64,
    /// Explicit reserve retained by the signing SOL authority. Must be zero for SPL.
    pub keep_lamports: u64,
    pub owner_utxo_hash: [u8; 32],
    pub ring_data_hash: [u8; 32],
    /// Safe only for this no-input certificate, never publish a spend's private blinding.
    pub private_tx_blinding: [u8; 32],
    /// Owner commitment of the stock frame's zero-SOL change. Never deposited by settlement.
    pub zero_change_owner_utxo_hash: [u8; 32],
    pub asset: DepositAssetKind,
    #[wincode(with = "containers::Vec<u8, FixIntLen<u16>>")]
    pub certificate: Vec<u8>,
}

impl ActualOutputIxData {
    pub fn parse(data: &[u8]) -> Result<Self, CustomRingError> {
        let request: Self = wincode::deserialize_exact(data)
            .map_err(|_| CustomRingError::InvalidInstructionData)?;
        if request.version != ACTUAL_OUTPUT_VERSION
            || request.descriptor_slot != DESCRIPTOR_SLOT
            || request.minimum_output == 0
        {
            return Err(CustomRingError::InvalidInstructionData);
        }
        Ok(request)
    }

    /// Canonical SPP encrypted-record body. The stock frame contains only public proof/ciphertext
    /// fields; no wallet key, owner preimage or private blinding is added to this event.
    pub fn recovery_body(&self) -> Vec<u8> {
        let mut body = Vec::with_capacity(9 + self.certificate.len());
        body.extend_from_slice(ACTUAL_OUTPUT_MAGIC);
        body.push(self.descriptor_slot);
        body.extend_from_slice(&self.certificate);
        body
    }
}

/// The actual amount is never taken from a quote, certificate or external-program return data.
pub fn sweep_amount(balance: u64, reserve: u64, minimum: u64) -> Result<u64, CustomRingError> {
    balance
        .checked_sub(reserve)
        .filter(|amount| *amount > 0 && *amount >= minimum)
        .ok_or(CustomRingError::InvalidSettlementAccounts)
}

/// Rebuild either of the stock descriptor's two canonical SPP commitments, with no application data.
pub fn descriptor_hash(
    tree_id: &[u8; 32],
    asset: &[u8; 32],
    ring_hash: &[u8; 32],
    owner_utxo_hash: &[u8; 32],
    amount: u64,
) -> Result<[u8; 32], CustomRingError> {
    let mut domain = [0; 32];
    domain[30..].copy_from_slice(&spr_interface::UTXO_DOMAIN.to_be_bytes());
    let mut amount_field = [0; 32];
    amount_field[24..].copy_from_slice(&amount.to_be_bytes());
    Ok(Poseidon::hashv(&[
        &domain,
        tree_id,
        asset,
        &amount_field,
        &[0; 32],
        ring_hash,
        owner_utxo_hash,
    ])?)
}

/// With a zero change, descriptor and no inputs/addresses, stock PrivateTxHash authenticates both.
pub fn descriptor_transaction_hash(
    zero_change: &[u8; 32],
    output: &[u8; 32],
    blinding: &[u8; 32],
) -> Result<[u8; 32], CustomRingError> {
    let outputs = Poseidon::hashv(&[zero_change, output])?;
    Ok(Poseidon::hashv(&[&[0; 32], &outputs, &[0; 32], blinding])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::RegistryId, storage_instruction::InitializeIxData};

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
    fn entire_positive_balance_with_explicit_reserve_and_minimum() {
        assert_eq!(sweep_amount(80, 10, 60), Ok(70));
        for (balance, reserve, min) in [(0, 0, 0), (10, 11, 1), (10, 10, 1), (10, 0, 11)] {
            assert!(sweep_amount(balance, reserve, min).is_err());
        }
    }

    #[test]
    fn settlement_wire_pins_token_kind_and_rejects_unversioned_data() {
        let request = ActualOutputIxData {
            version: ACTUAL_OUTPUT_VERSION,
            descriptor_slot: DESCRIPTOR_SLOT,
            minimum_output: (1 << 53) + 3,
            keep_lamports: 0,
            owner_utxo_hash: [7; 32],
            ring_data_hash: [0; 32],
            private_tx_blinding: [9; 32],
            zero_change_owner_utxo_hash: [8; 32],
            asset: DepositAssetKind::Spl {
                spl_interface_bump: 253,
            },
            certificate: vec![4, 5, 6],
        };
        let encoded = wincode::serialize(&request).unwrap();
        assert_eq!(&encoded[..2], &[1, 1]);
        assert_eq!(&encoded[2..10], &request.minimum_output.to_le_bytes());
        assert_eq!(&encoded[114..146], &[8; 32]);
        assert_eq!(&encoded[146..], &[1, 253, 3, 0, 4, 5, 6]);
        assert_eq!(ActualOutputIxData::parse(&encoded), Ok(request));
        for invalid in [
            [&[2u8], &encoded[1..]].concat(),
            [&encoded[..1], &[0], &encoded[2..]].concat(),
            [&encoded[..2], &[0; 8], &encoded[10..]].concat(),
            [&encoded[..], &[0]].concat(),
        ] {
            assert!(ActualOutputIxData::parse(&invalid).is_err());
        }
    }

    #[test]
    fn transaction_binding_changes_with_descriptor_tree_asset_or_owner() {
        let [tree, mint, ring, owner, blinding] = [1, 2, 3, 4, 5].map(|value| {
            let mut field = [0; 32];
            field[31] = value;
            field
        });
        let binding = |tree, asset, owner, change_owner| {
            let change = descriptor_hash(
                tree,
                &spr_interface::SOL_ASSET_FIELD,
                &ring,
                change_owner,
                0,
            )
            .unwrap();
            let output = descriptor_hash(tree, asset, &ring, owner, DESCRIPTOR_AMOUNT).unwrap();
            descriptor_transaction_hash(&change, &output, &blinding).unwrap()
        };
        let original = binding(&tree, &mint, &owner, &tree);
        assert_ne!(original, binding(&mint, &mint, &owner, &tree));
        assert_ne!(original, binding(&tree, &tree, &owner, &tree));
        assert_ne!(original, binding(&tree, &mint, &tree, &tree));
        assert_ne!(original, binding(&tree, &mint, &owner, &mint));
    }
}
