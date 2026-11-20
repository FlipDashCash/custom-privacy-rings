use std::collections::HashMap;

use custom_ring_sdk::VaultEnrollmentMessage;
use solana_address::Address;
use solana_hash::Hash;
use solana_signature::Signature;
use solana_signer::Signer;
use spr_indexer_api::SerializableSignature;
use zolana_ring_client::rpc::{
    SignedVaultEnrollment, VaultEnrollmentBlock, VaultEnrollmentRequest,
};

use super::{
    ring_tracking, EnrollmentApplier, EnrollmentRefusal, RingRpcError, RingState, RingTracking,
    MAX_HELD_ENROLLMENTS,
};
use crate::{
    landing::{signature_fees, Simulation},
    relay_submit::{presigned_transaction, CollectedSignature},
};

#[derive(Default)]
pub(super) struct VaultAttempts {
    blocks: HashMap<Hash, u64>,
    submissions: HashMap<Signature, u64>,
}

impl VaultAttempts {
    fn expire(&mut self, height: u64) {
        self.blocks.retain(|_, expiry| *expiry >= height);
        self.submissions.retain(|_, expiry| *expiry >= height);
    }
}

impl EnrollmentApplier {
    #[must_use]
    pub fn with_vault_programs(mut self, programs: impl IntoIterator<Item = Address>) -> Self {
        self.vault_programs.extend(programs);
        self
    }

    async fn vault_tracking(
        &self,
        request: &VaultEnrollmentRequest,
    ) -> Result<RingTracking, RingRpcError> {
        let ring = request.ring.0;
        if !self.vault_programs.contains(&request.program.0)
            || !(self.named.contains(&ring) || self.registered().contains(&ring))
        {
            return Err(EnrollmentRefusal::NotServed.into());
        }
        ring_tracking(self.chain.as_ref(), ring)
            .await?
            .filter(|tracking| tracking.writer == self.payer())
            .ok_or_else(|| EnrollmentRefusal::RingMissing.into())
    }

    pub async fn prepare_vault(
        &self,
        request: VaultEnrollmentRequest,
    ) -> Result<VaultEnrollmentBlock, RingRpcError> {
        let tracking = self.vault_tracking(&request).await?;
        let ring = request.ring.0;
        if !self.budget.holds(ring, signature_fees(2)) {
            return Err(EnrollmentRefusal::Budget.into());
        }
        let latest = self.chain.latest_blockhash().await?;
        let block = VaultEnrollmentBlock {
            blockhash: spr_indexer_api::Hash(latest.blockhash.to_bytes()),
            last_valid_block_height: latest.last_valid_block_height,
        };
        VaultEnrollmentMessage {
            request: &request,
            writer: self.payer(),
            registry: tracking.registry,
        }
        .compile(&block)
        .map_err(|_| EnrollmentRefusal::InvalidWrite)?;
        let height = self.chain.block_height().await?;
        let state = self.ring_state(ring);
        let mut state = state.lock().await;
        state.vault.expire(height);
        state
            .vault
            .blocks
            .insert(latest.blockhash, latest.last_valid_block_height);
        Ok(block)
    }

    pub async fn submit_vault(
        &self,
        signed: SignedVaultEnrollment,
    ) -> Result<SerializableSignature, RingRpcError> {
        let request = &signed.enrollment;
        let tracking = self.vault_tracking(request).await?;
        let ring = request.ring.0;
        let message = VaultEnrollmentMessage {
            request,
            writer: self.payer(),
            registry: tracking.registry,
        }
        .compile(&signed.block)
        .map_err(|_| EnrollmentRefusal::InvalidWrite)?;
        let bytes = message.serialize();
        let signature = signed.signature.0;
        if !signature.verify(request.authority.0.as_ref(), &bytes) {
            return Err(EnrollmentRefusal::InvalidWrite.into());
        }
        let state = self.ring_state(ring);
        let mut state = state.lock().await;
        let RingState { vault, .. } = &mut *state;
        let height = self.chain.block_height().await?;
        vault.expire(height);
        let expiry = signed.block.last_valid_block_height;
        if vault
            .blocks
            .get(&Hash::new_from_array(signed.block.blockhash.0))
            != Some(&expiry)
        {
            return Err(EnrollmentRefusal::InvalidWrite.into());
        }
        let writer_signature = self.payer.sign_message(&bytes);
        let transaction = presigned_transaction(
            &message,
            &[
                CollectedSignature {
                    signer: self.payer(),
                    signature: writer_signature,
                },
                CollectedSignature {
                    signer: request.authority.0,
                    signature,
                },
            ],
        )?;
        if vault.submissions.contains_key(&writer_signature) {
            if self.chain.outcome(writer_signature).await?.is_none() {
                self.chain.send(&transaction).await?;
            }
            return Ok(writer_signature.into());
        }
        if vault.submissions.len() >= MAX_HELD_ENROLLMENTS {
            return Err(EnrollmentRefusal::Full.into());
        }
        if !matches!(self.chain.simulate(&message).await?, Simulation::Passed) {
            return Err(EnrollmentRefusal::InvalidWrite.into());
        }
        if !self.budget.charge(ring, signature_fees(2)) {
            return Err(EnrollmentRefusal::Budget.into());
        }
        vault.submissions.insert(writer_signature, expiry);
        self.chain.send(&transaction).await?;
        Ok(writer_signature.into())
    }
}
