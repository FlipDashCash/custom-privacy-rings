//! Inert Custom Jupiter measured-output descriptor context V1.
//!
//! Exercised only by host tests and typechecked in the no-entrypoint target
//! review. There is no authenticated caller yet. The
//! deployment pins, live roots, stock withdrawal reference, role addresses
//! and guard views must come from independent checks in the future processor,
//! never from instruction assertions or a quote. The result is not a proof,
//! execution approval or observation of an output amount.

#![forbid(unsafe_code)]

#[cfg(target_os = "solana")]
use pinocchio::Address;
#[cfg(not(target_os = "solana"))]
use solana_address::Address;
use spr_hasher::{
    primitives::{hash_bytes, is_canonical_bn254_scalar_be},
    Hasher, Sha256,
};

use super::wire::{Access, DraftFrame, Role, DRAFT_TAG, DRAFT_VERSION, ROLES};
// The host includes this source below a test crate; the target imports the
// same public types through its existing normal dependencies.
#[cfg(not(target_os = "solana"))]
use crate::{storage::Revision, TransferRoots};
#[cfg(target_os = "solana")]
use custom_ring_interface::{storage::Revision, TransferRoots};

const DOMAIN: &[u8] = b"pyrol:custom:jupiter:measured-descriptor-context:v1\0";
pub(super) const GUARD_FRAME_BYTES: usize = 126;
const GUARD_FIXED_ROLES: usize = 6;
// Existing PYJDYN01 guard message roster bounds, not a new route/venue policy.
const GUARD_MAX_ROLES: usize = 63;
const GUARD_MAGIC: &[u8; 8] = b"PYJDYN01";
const ROLE_BYTES: usize = 33;
/// The existing descriptor relation needs a maximum. SPL admits a u64; this is
/// the representational ceiling, with no additional product amount cap.
pub(super) const SPL_OUTPUT_MAX: u64 = u64::MAX;

/// Chosen by the independently reviewed release profile, not specified in
/// tag 68 wire. A nonzero hash here does not authenticate an artifact.
pub(super) struct ProfilePins {
    pub(super) review_profile_sha256: [u8; 32],
    pub(super) genesis: [u8; 32],
    pub(super) guard_elf_sha256: [u8; 32],
    pub(super) jupiter_elf_sha256: [u8; 32],
    pub(super) descriptor_r1cs_sha256: [u8; 32],
    pub(super) descriptor_vk_sha256: [u8; 32],
}

/// Same-operation stock withdrawal fields. A future internal caller must
/// obtain both digests from the genuine, already consumed withdrawal.
pub(super) struct OriginalWithdrawal {
    pub(super) first_nullifier: [u8; 32],
    pub(super) private_tx_hash: [u8; 32],
    pub(super) gross_input: u64,
    pub(super) commercial_fee: u64,
}

pub(super) struct GuardAccount {
    pub(super) address: Address,
    pub(super) is_writable: bool,
    pub(super) is_signer: bool,
}

/// Actual compiled guard bytes and common ordered meta roster, after the
/// existing prefix checks have matched both guards and the Jupiter route.
pub(super) struct MatchedGuards<'a> {
    pub(super) pre: &'a [u8; GUARD_FRAME_BYTES],
    pub(super) post: &'a [u8; GUARD_FRAME_BYTES],
    pub(super) accounts: &'a [GuardAccount],
}

pub(super) struct ContextInput<'a> {
    pub(super) profile: &'a ProfilePins,
    pub(super) ring_program: &'a Address,
    /// The V1 outer role order, including staging, mint and SPP vault.
    pub(super) roles: &'a [Address; ROLES.len()],
    pub(super) guards: MatchedGuards<'a>,
    pub(super) original: &'a OriginalWithdrawal,
    pub(super) tree_id: &'a [u8; 32],
    pub(super) ring_generation: u32,
}

/// Deterministic statement input only. It cannot authenticate the caller or
/// be used as a receipt, grant, observed balance or selected verifier.
pub(super) struct ContextFields {
    pub(super) operation_context: [u8; 32],
    pub(super) tracking_revision: Revision,
    pub(super) minimum_output: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ContextError {
    Profile,
    Root,
    Original,
    Guard,
    Hash,
}

fn role_flags(access: Access) -> u8 {
    match access {
        Access::ReadOnly => 0,
        Access::Writable => 1,
        Access::ReadOnlySigner => 2,
        Access::WritableSigner => 3,
    }
}

impl GuardAccount {
    fn flags(&self) -> u8 {
        u8::from(self.is_writable) | (u8::from(self.is_signer) << 1)
    }
}

impl ContextInput<'_> {
    pub(super) fn prepare(
        &self,
        frame: &DraftFrame<'_>,
        accepted_roots: &TransferRoots,
    ) -> Result<ContextFields, ContextError> {
        let pins = self.profile;
        if pins.review_profile_sha256 == [0; 32]
            || pins.genesis == [0; 32]
            || pins.guard_elf_sha256 == [0; 32]
            || pins.jupiter_elf_sha256 == [0; 32]
            || pins.descriptor_r1cs_sha256 == [0; 32]
            || pins.descriptor_vk_sha256 == [0; 32]
            || frame.expiry_slot == 0
        {
            return Err(ContextError::Profile);
        }
        // This is the original restricted anonymous, no-policy first profile.
        // Stock LiveRoots must have already served the exact selected revision.
        if accepted_roots.policy_hash != [0; 32]
            || accepted_roots.own_root != [0; 32]
            || accepted_roots.curator_root != [0; 32]
            || &accepted_roots.tracking_root != frame.tracking_root
            || !is_canonical_bn254_scalar_be(&accepted_roots.tracking_root)
        {
            return Err(ContextError::Root);
        }
        let tracking_revision = Revision(frame.tracking_revision.to_le_bytes());
        let original = self.original;
        if original.first_nullifier == [0; 32] || original.commercial_fee == 0 {
            return Err(ContextError::Original);
        }
        let market_input = original
            .gross_input
            .checked_sub(original.commercial_fee)
            .filter(|amount| *amount != 0)
            .ok_or(ContextError::Original)?;
        let pre = self.guards.pre;
        let post = self.guards.post;
        let accounts = self.guards.accounts;
        if &pre[..8] != GUARD_MAGIC
            || pre[8] != 0
            || post[8] != 1
            || post[..8] != pre[..8]
            || post[9..] != pre[9..]
            || pre[9] != 0 // Only the minimum-output guard path.
            || pre[36..44] != [0; 8]
            || pre[12..20] != market_input.to_le_bytes()
            || !(GUARD_FIXED_ROLES + 1..=GUARD_MAX_ROLES).contains(&accounts.len())
        {
            return Err(ContextError::Guard);
        }
        let minimum_output =
            u64::from_le_bytes(pre[28..36].try_into().map_err(|_| ContextError::Guard)?);
        if minimum_output == 0 {
            return Err(ContextError::Guard);
        }

        let mut fixed = [0; ROLES.len() * ROLE_BYTES];
        for (i, (role, access)) in ROLES.iter().copied().enumerate() {
            let address = &self.roles[role as usize];
            let flags = role_flags(access);
            if i < GUARD_FIXED_ROLES
                && (accounts[i].address != *address || accounts[i].flags() != flags)
            {
                return Err(ContextError::Guard);
            }
            let offset = i * ROLE_BYTES;
            fixed[offset..offset + 32].copy_from_slice(address.as_ref());
            fixed[offset + 32] = flags;
        }
        let jupiter = &self.roles[Role::JupiterProgram as usize];
        if !accounts.iter().any(|m| &m.address == jupiter && m.flags() == 0) {
            return Err(ContextError::Guard);
        }
        let mut ordered = Vec::with_capacity(accounts.len() * ROLE_BYTES);
        for account in accounts {
            ordered.extend_from_slice(account.address.as_ref());
            ordered.push(account.flags());
        }

        // Fixed lengths except for the guard roster, which is count-prefixed
        // and whose rows are exactly 33 bytes. No ciphertext or proof cycles.
        let digest = Sha256::hashv(&[
            DOMAIN,
            &[DRAFT_TAG, DRAFT_VERSION],
            &pins.review_profile_sha256,
            &pins.genesis,
            &pins.guard_elf_sha256,
            &pins.jupiter_elf_sha256,
            &pins.descriptor_r1cs_sha256,
            &pins.descriptor_vk_sha256,
            self.ring_program.as_ref(),
            &original.first_nullifier,
            &original.private_tx_hash,
            &original.gross_input.to_le_bytes(),
            &original.commercial_fee.to_le_bytes(),
            self.tree_id,
            &self.ring_generation.to_le_bytes(),
            &tracking_revision.0,
            &accepted_roots.policy_hash,
            &accepted_roots.own_root,
            &accepted_roots.curator_root,
            &accepted_roots.tracking_root,
            &frame.expiry_slot.to_le_bytes(),
            &SPL_OUTPUT_MAX.to_le_bytes(),
            &fixed,
            pre,
            post,
            &[accounts.len() as u8],
            &ordered,
        ])
        .map_err(|_| ContextError::Hash)?;
        let operation_context = hash_bytes(&digest).map_err(|_| ContextError::Hash)?;
        if operation_context == [0; 32] {
            return Err(ContextError::Hash);
        }
        Ok(ContextFields {
            operation_context,
            tracking_revision,
            minimum_output,
        })
    }
}
