//! Dormant V1 descriptor statement and one public signal. No key or verifier.
//!
//! Exact public statement/projection arithmetic lifted from owned closed Custom
//! #5 f841ff6b9fb454c1228f7d5591735a9536d055e0, blob
//! 68ddff0e1d1debaa0c2a7fbcf043dd9bedc03832. The matching relation in that
//! source is a reference, not a release profile. Host tests exercise this
//! file; the no-entrypoint target review only typechecks it. A future caller
//! must authenticate the policy, tracking revision/root and runtime original.

#![forbid(unsafe_code)]

#[cfg(target_os = "solana")]
use pinocchio::Address;
#[cfg(not(target_os = "solana"))]
use solana_address::Address;
use spr_hasher::{
    hash_chain::create_hash_chain_4_from_slice,
    primitives::{
        hash_bytes, is_canonical_bn254_scalar_be, pack_be, right_align, solana_owner_identity,
    },
};

use custom_ring_interface::{
    storage::Revision, tracking::TrackingRegistry, RingPolicy, TAGGED_X25519_KEY_LEN,
    VIEWING_PUBKEY_TAG,
};

pub(super) const DOMAIN: u64 = 0x4352_4444; // CRDD, distinct from a ring transfer (CRTX).
pub(super) const VERSION: u64 = 1;
pub(super) const ELEMENT_COUNT: usize = 25;
pub(super) const OUTPUT_LIMIT: u8 = 5;
pub(super) const RECIPIENT_CIPHERTEXT_LEN: usize = 105;
pub(super) const AUDITOR_CIPHERTEXT_LEN: usize = 64;

// Descriptor v1 fixes the tagged-key scheme and packing widths.
const _: () = assert!(TAGGED_X25519_KEY_LEN == 33 && VIEWING_PUBKEY_TAG == 4);

#[derive(Clone, Debug)]
pub(super) struct DescriptorStatement {
    pub(super) ring_id: [u8; 32],
    pub(super) auditor_pubkey: [u8; TAGGED_X25519_KEY_LEN],
    pub(super) tracking_root: [u8; 32],
    pub(super) ring_generation: u32,
    pub(super) operation_context: [u8; 32],
    pub(super) output_index: u8,
    pub(super) source_authority_identity: [u8; 32],
    pub(super) payer_identity: [u8; 32],
    pub(super) tree_id: [u8; 32],
    pub(super) asset: [u8; 32],
    pub(super) owner_utxo_hash: [u8; 32],
    pub(super) min_amount: u64,
    pub(super) max_amount: u64,
    pub(super) expiry_slot: u64,
    pub(super) tx_viewing_pubkey: [u8; TAGGED_X25519_KEY_LEN],
    pub(super) salt: [u8; 16],
    /// The low eight bytes of the anonymous recipient tag, zero-extended.
    pub(super) recipient_tag: [u8; 32],
    pub(super) recipient_zero_ciphertext: [u8; RECIPIENT_CIPHERTEXT_LEN],
    pub(super) auditor_ciphertext: [u8; AUDITOR_CIPHERTEXT_LEN],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StatementError {
    Encoding,
    Hashing,
}

impl DescriptorStatement {
    /// The 25 field elements in circuit order, including zero ring data and
    /// the final element count. Encoding checks do not establish a valid
    /// curve key, an enrollment, or ownership of any note.
    pub(super) fn elements(&self) -> Result<[[u8; 32]; ELEMENT_COUNT], StatementError> {
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
    pub(super) fn hash(&self) -> Result<[u8; 32], StatementError> {
        create_hash_chain_4_from_slice(&self.elements()?).map_err(|_| StatementError::Hashing)
    }
}

fn field(value: u64) -> [u8; 32] {
    right_align(&value.to_be_bytes())
}

/// Shared fields of an already validated single-output operation (ordinal 0).
/// The caller supplies actual account identities, the predecessor's first
/// nullifier, and the already resolved exact tracking revision and root. This
/// value does not read accounts, authenticate a route, or resolve a live root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DescriptorProjection {
    pub(super) ring_program: Address,
    pub(super) tracking_account: Address,
    pub(super) tracking_revision: Revision,
    pub(super) tracking_root: [u8; 32],
    pub(super) first_nullifier: [u8; 32],
    pub(super) source_authority: Address,
    pub(super) payer: Address,
    pub(super) tree_id: [u8; 32],
    pub(super) output_mint: Address,
    pub(super) min_amount: u64,
    pub(super) max_amount: u64,
    pub(super) expiry_slot: u64,
}

/// Descriptor-specific commitment fields. No proof or witness is accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DescriptorRecord {
    pub(super) owner_utxo_hash: [u8; 32],
    pub(super) tx_viewing_pubkey: [u8; TAGGED_X25519_KEY_LEN],
    pub(super) salt: [u8; 16],
    pub(super) recipient_tag: [u8; 32],
    pub(super) recipient_zero_ciphertext: [u8; RECIPIENT_CIPHERTEXT_LEN],
    pub(super) auditor_ciphertext: [u8; AUDITOR_CIPHERTEXT_LEN],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProjectionError<E> {
    RingBinding,
    Context(E),
    Statement(StatementError),
}

/// A statement and its public input, frozen together. This is not an approval
/// or proof of the route. No mutable access to either value is provided.
///
#[derive(Clone, Debug)]
pub(super) struct PreparedDescriptor {
    statement: DescriptorStatement,
    public_input_hash: [u8; 32],
}

impl PreparedDescriptor {
    pub(super) fn statement(&self) -> &DescriptorStatement {
        &self.statement
    }

    pub(super) fn public_input_hash(&self) -> &[u8; 32] {
        &self.public_input_hash
    }
}

impl DescriptorProjection {
    /// Rebuild the descriptor with native identity and mint hashes. The caller
    /// owns the versioned route transcript; compose it from the immutable
    /// projection passed to the closure, not a second copy of shared fields.
    /// The closure is trusted code, not a verifier of a caller-supplied hash.
    /// The caller must still authenticate accounts, profile, exact root, and
    /// predecessor before preparing a statement. This method performs only the
    /// ring/registry cross-check and deterministic statement reconstruction.
    pub(super) fn prepare<E>(
        self,
        policy: &RingPolicy,
        tracking: &TrackingRegistry,
        record: &DescriptorRecord,
        compose_context: impl FnOnce(&Self) -> Result<[u8; 32], E>,
    ) -> Result<PreparedDescriptor, ProjectionError<E>> {
        let hashing = |_| ProjectionError::Statement(StatementError::Hashing);
        let ring_id = hash_bytes(self.ring_program.as_array()).map_err(hashing)?;
        if policy.tracking != self.tracking_account
            || tracking.ring_id != ring_id
            || tracking.auditor_pubkey != policy.auditor_pubkey
        {
            return Err(ProjectionError::RingBinding);
        }
        let context = compose_context(&self).map_err(ProjectionError::Context)?;
        let statement = DescriptorStatement {
            ring_id,
            auditor_pubkey: policy.auditor_pubkey,
            tracking_root: self.tracking_root,
            ring_generation: policy.generation(),
            operation_context: context,
            output_index: 0,
            source_authority_identity: solana_owner_identity(self.source_authority.as_array())
                .map_err(hashing)?,
            payer_identity: solana_owner_identity(self.payer.as_array()).map_err(hashing)?,
            tree_id: self.tree_id,
            asset: hash_bytes(self.output_mint.as_array()).map_err(hashing)?,
            owner_utxo_hash: record.owner_utxo_hash,
            min_amount: self.min_amount,
            max_amount: self.max_amount,
            expiry_slot: self.expiry_slot,
            tx_viewing_pubkey: record.tx_viewing_pubkey,
            salt: record.salt,
            recipient_tag: record.recipient_tag,
            recipient_zero_ciphertext: record.recipient_zero_ciphertext,
            auditor_ciphertext: record.auditor_ciphertext,
        };
        let public_input_hash = statement.hash().map_err(ProjectionError::Statement)?;
        Ok(PreparedDescriptor {
            statement,
            public_input_hash,
        })
    }
}
