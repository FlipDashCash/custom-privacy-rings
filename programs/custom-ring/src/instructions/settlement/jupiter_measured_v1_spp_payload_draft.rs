//! Inert producer-side materialization of one stock SPP SPL deposit body.
//!
//! Host source composition only: it accepts no caller-supplied amount, bump,
//! second recipient record or operation-context override. Preparing this data
//! does not verify the descriptor, establish an executed original, or make the
//! observed staging balance attributable to Jupiter. It creates no account
//! metas, signed instruction, event, or credit.
#![forbid(unsafe_code)]

use pinocchio::AccountView;
use spr_hasher::primitives::is_canonical_bn254_scalar_be;
use spr_interface::instruction::{
    DepositAssetKind, EncryptedRingDepositData, RingDepositEntry, RingDepositIxData,
};

use super::{
    descriptor_statement::{DescriptorStatement, AUDITOR_CIPHERTEXT_LEN, RECIPIENT_CIPHERTEXT_LEN},
    draft::{prepare_descriptor, ComparedRoles, DescriptorScope, PrefixProjection},
    output::LegacySplOutputReview,
    wire::{Access, DraftFrame, Role, ROLES},
};
use custom_ring_interface::tracking::TrackingRegistry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PayloadError {
    Roles,
    Projection,
    Observation,
    Capsule,
}

/// Retain both the serialized body's source and its original account handles.
/// Neither a body nor matching post-deposit balances make this an authorization.
pub(super) struct PreparedSppBody<'a> {
    body: RingDepositIxData,
    observation: LegacySplOutputReview<'a>,
}

impl PreparedSppBody<'_> {
    pub(super) fn body(&self) -> &RingDepositIxData {
        &self.body
    }

    pub(super) fn compare_same_output_views(&self) -> Result<(), pinocchio::error::ProgramError> {
        self.observation.compare_reread()
    }
}

/// Compare the complete role roster for this source projection, derive the
/// descriptor ONCE from this original, then sample those very output views
/// ONCE. This is not the authenticated runtime/window or a verifier caller.
pub(super) fn prepare_original<'a>(
    original: PrefixProjection<'_>,
    scope: &DescriptorScope<'_>,
    frame: &DraftFrame<'_>,
    tracking: &TrackingRegistry,
    views: &'a [AccountView; ROLES.len()],
) -> Result<PreparedSppBody<'a>, PayloadError> {
    let addresses = views.each_ref().map(|view| *view.address());
    ComparedRoles::new(&addresses, scope.roles).map_err(|_| PayloadError::Roles)?;
    for ((_, access), view) in ROLES.iter().zip(views) {
        let privileges = match access {
            Access::ReadOnly => (false, false),
            Access::Writable => (true, false),
            Access::ReadOnlySigner => (false, true),
            Access::WritableSigner => (true, true),
        };
        if (view.is_writable(), view.is_signer()) != privileges {
            return Err(PayloadError::Roles);
        }
    }
    let prepared = prepare_descriptor(original, scope, frame, tracking)
        .map_err(|_| PayloadError::Projection)?;
    let statement = prepared.statement();
    let observation = LegacySplOutputReview::observe(
        &views[Role::Depositor as usize],
        &views[Role::OutputMint as usize],
        &views[Role::OutputStaging as usize],
        &views[Role::OutputVault as usize],
        &views[Role::LegacyTokenProgram as usize],
        statement.min_amount,
    )
    .map_err(|_| PayloadError::Observation)?;

    // The only materializer call is private to this matched composition. The
    // lower account reader retains the first read for both encoding and its
    // same-handle postcondition. No reread selects an alternative amount.
    let body = materialize(statement, &observation)?;
    Ok(PreparedSppBody { body, observation })
}

fn materialize(
    statement: &DescriptorStatement,
    observed: &LegacySplOutputReview<'_>,
) -> Result<RingDepositIxData, PayloadError> {
    let (amount, spl_interface_bump) = observed.observed_spl_entry();
    if statement.output_index != 0
        || statement.min_amount == 0
        || amount < statement.min_amount
        || amount > statement.max_amount
        || statement.operation_context == [0; 32]
        || !is_canonical_bn254_scalar_be(&statement.operation_context)
    {
        return Err(PayloadError::Capsule);
    }

    // Pure CRDYN001 encode/materialize semantics, owned app 9d62f55,
    // integrations/dynamic-ring-settlement/interface/src/lib.rs (blob
    // f48f089486b75db3aaae83d729084f5ff24b14be). This transformation
    // cannot prove its template opens to amount zero. That is a requirement
    // on the eventual genuine verifier, NOT an assertion about this input.
    let mut record = statement.recipient_zero_ciphertext;
    for (position, byte) in amount.to_be_bytes().iter().enumerate() {
        record[1 + position] ^= byte;
    }
    let mut capsule = [0u8; 8 + 32 + 1 + RECIPIENT_CIPHERTEXT_LEN + AUDITOR_CIPHERTEXT_LEN];
    capsule[..8].copy_from_slice(b"CRDYN001");
    capsule[8..40].copy_from_slice(&statement.operation_context);
    capsule[40] = statement.output_index;
    capsule[41..146].copy_from_slice(&record);
    capsule[146..].copy_from_slice(&statement.auditor_ciphertext);

    // Only the stock types of the owned pinned Privacy submodule. No outer
    // wrapper wire, venue, signer, key selection or invocation is ported.
    Ok(RingDepositIxData {
        assets: vec![DepositAssetKind::Spl { spl_interface_bump }],
        deposits: vec![RingDepositEntry {
            asset_index: 0,
            view_tag: statement.recipient_tag,
            owner_utxo_hash: statement.owner_utxo_hash,
            amount,
            ring_data_hash: [0; 32],
            encrypted: EncryptedRingDepositData {
                tx_viewing_pk: statement.tx_viewing_pubkey,
                salt: statement.salt,
                ciphertext: capsule.to_vec(),
            },
        }],
    })
}
