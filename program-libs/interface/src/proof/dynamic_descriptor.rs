//! The public statement of the dynamic deposit descriptor v1 (`CRDD`).
//!
//! Mirrors `prover/circuits/dynamicdescriptor/descriptor.go`. It builds a
//! statement only. The caller must authenticate its account-derived values,
//! live tracking root and operation context. This module neither authorizes a
//! deposit or venue call nor selects a key or verifies a proof.

#![forbid(unsafe_code)]

use spr_hasher::{
    hash_chain::create_hash_chain_4_from_slice,
    primitives::{hash_bytes, is_canonical_bn254_scalar_be, pack_be, right_align},
};

use crate::{TAGGED_X25519_KEY_LEN, VIEWING_PUBKEY_TAG};

pub const DOMAIN: u64 = 0x4352_4444; // CRDD, distinct from a ring transfer (CRTX).
pub const VERSION: u64 = 1;
pub const ELEMENT_COUNT: usize = 25;
pub const OUTPUT_LIMIT: u8 = 5;
pub const RECIPIENT_CIPHERTEXT_LEN: usize = 105;
pub const AUDITOR_CIPHERTEXT_LEN: usize = 64;

// Descriptor v1 fixes the tagged-key scheme and packing widths.
const _: () = assert!(TAGGED_X25519_KEY_LEN == 33 && VIEWING_PUBKEY_TAG == 4);

#[derive(Clone, Debug)]
pub struct DescriptorStatement {
    pub ring_id: [u8; 32],
    pub auditor_pubkey: [u8; TAGGED_X25519_KEY_LEN],
    pub tracking_root: [u8; 32],
    pub ring_generation: u32,
    pub operation_context: [u8; 32],
    pub output_index: u8,
    pub source_authority_identity: [u8; 32],
    pub payer_identity: [u8; 32],
    pub tree_id: [u8; 32],
    pub asset: [u8; 32],
    pub owner_utxo_hash: [u8; 32],
    pub min_amount: u64,
    pub max_amount: u64,
    pub expiry_slot: u64,
    pub tx_viewing_pubkey: [u8; TAGGED_X25519_KEY_LEN],
    pub salt: [u8; 16],
    /// The low eight bytes of the anonymous recipient tag, zero-extended.
    pub recipient_tag: [u8; 32],
    pub recipient_zero_ciphertext: [u8; RECIPIENT_CIPHERTEXT_LEN],
    pub auditor_ciphertext: [u8; AUDITOR_CIPHERTEXT_LEN],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatementError {
    Encoding,
    Hashing,
}

impl DescriptorStatement {
    /// The 25 field elements in circuit order, including zero ring data and
    /// the final element count. Encoding checks do not establish a valid
    /// curve key, an enrollment, or ownership of any note.
    pub fn elements(&self) -> Result<[[u8; 32]; ELEMENT_COUNT], StatementError> {
        for value in [
            &self.ring_id,
            &self.tracking_root,
            &self.operation_context,
            &self.source_authority_identity,
            &self.payer_identity,
            &self.tree_id,
            &self.asset,
            &self.owner_utxo_hash,
        ] {
            if !is_canonical_bn254_scalar_be(value) {
                return Err(StatementError::Encoding);
            }
        }
        if self.operation_context == [0; 32]
            || self.output_index >= OUTPUT_LIMIT
            || self.min_amount > self.max_amount
            || self.auditor_pubkey[0] != VIEWING_PUBKEY_TAG
            || self.tx_viewing_pubkey[0] != VIEWING_PUBKEY_TAG
            || self.recipient_tag[..24] != [0; 24]
        {
            return Err(StatementError::Encoding);
        }
        let [aud0, aud1] = pack_be(&self.auditor_pubkey);
        let [tx0, tx1] = pack_be(&self.tx_viewing_pubkey);
        Ok([
            field(DOMAIN),
            field(VERSION),
            self.ring_id,
            aud0,
            aud1,
            self.tracking_root,
            field(u64::from(self.ring_generation)),
            self.operation_context,
            field(u64::from(self.output_index)),
            self.source_authority_identity,
            self.payer_identity,
            self.tree_id,
            self.asset,
            self.owner_utxo_hash,
            [0; 32], // v1 supports no ring data.
            field(self.min_amount),
            field(self.max_amount),
            field(self.expiry_slot),
            tx0,
            tx1,
            right_align(&self.salt),
            self.recipient_tag,
            hash_bytes(&self.recipient_zero_ciphertext).map_err(|_| StatementError::Hashing)?,
            hash_bytes(&self.auditor_ciphertext).map_err(|_| StatementError::Hashing)?,
            field(ELEMENT_COUNT as u64),
        ])
    }

    /// The circuit's one public input, HashChain4 over the complete statement.
    pub fn hash(&self) -> Result<[u8; 32], StatementError> {
        create_hash_chain_4_from_slice(&self.elements()?).map_err(|_| StatementError::Hashing)
    }
}

fn field(value: u64) -> [u8; 32] {
    right_align(&value.to_be_bytes())
}
