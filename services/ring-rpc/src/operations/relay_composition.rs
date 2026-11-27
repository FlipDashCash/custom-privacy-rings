use std::collections::HashSet;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use solana_address::Address;
use solana_message::{v1, VersionedMessage};
use solana_signature::Signature;
use solana_transaction::versioned::VersionedTransaction;
use zolana_ring_client::rpc::{RelayId, RelayStep, ServiceTarget};

use super::{message_fee, step, AttemptWrite, Relay, RelayError, StagedAttempt};
use crate::{
    error::RingRpcError,
    landing::Simulation,
    relay_reservations::{Admit, Fund},
    relay_submit::CollectedSignature,
    relays::{Awaiting, RelayRecord, RelayState},
};

/// A decision about the exact signed message at the operator's trusted bank.
pub struct CompositionReview {
    pub ring: Address,
    pub message_hash: [u8; 32],
    /// Canonical spend identity, stable across rebuilds and independent of the blockhash.
    pub attempt_key: [u8; 32],
    /// A verified non-payer signer charged against the per-signer relay budget.
    pub budget_signer: Address,
    /// The original blockhash's expiration height or a conservative upper bound.
    pub last_valid_block_height: u64,
    pub network_fee_lamports: u64,
    /// Gross payer debit on success, including fees and rent, before any reimbursement.
    pub maximum_debit_lamports: u64,
}

/// Trusted implementation supplied by the relay operator. Never use a client verdict.
///
/// Review the entire message and canonical state for this service and cluster. Admit
/// only the supported proven Ring calls, authorized users, venues, accounts and
/// bounded payer privileges. Require atomic protected settlement, independently
/// enforced authority over transient assets and any required Ring policy approvals.
/// Authenticated reimbursement, if required, must be an exact proof-bound SOL
/// withdrawal to the payer. Verify the original recent blockhash at the bank
/// and obtain its fee and expiration bound. Refuse unsupported co-signer profiles.
/// An attempt key serializes one spend identity. If a profile can compose partially
/// overlapping nullifier sets, the operator must serialize that overlap separately.
#[async_trait]
pub trait CompositionPolicy: Send + Sync {
    async fn review(
        &self,
        service: ServiceTarget,
        transaction: &VersionedTransaction,
    ) -> Result<CompositionReview, RingRpcError>;
}

impl Relay {
    /// Stage a complete proved V1 transaction under an operator-owned policy.
    ///
    /// The payer signature stays empty and every other signature must already cover
    /// the original. Staging keeps the original durable and does not sign or send it.
    /// The caller submits the first non-payer's detached signature via `submit`.
    /// Keep the original wire before staging. After an uncertain response, retry
    /// with the same wire to obtain its relay ID and exact staged message, or use
    /// that ID with `status` and check the returned signature in canonical history.
    pub async fn prepare_composition(
        &self,
        wire: &[u8],
        policy: &dyn CompositionPolicy,
    ) -> Result<RelayStep, RingRpcError> {
        let transaction = decode(wire, self.fee_payer())?;
        let message = &transaction.message;
        let bytes = message.serialize();
        let signers = &message.static_account_keys()[..transaction.signatures.len()];
        if self
            .co_signer
            .signer()
            .is_some_and(|signer| signers.contains(&signer))
        {
            return Err(RelayError::CoSignRefused.into());
        }
        let id: [u8; 32] = Sha256::new()
            .chain_update(b"zolana/ring-rpc-composition")
            .chain_update(self.service.cluster)
            .chain_update(self.service.service.as_ref())
            .chain_update(&bytes)
            .finalize()
            .into();
        let relay_id = RelayId(id.into());
        if let Some(row) = self.relays().get(&relay_id).await? {
            return stored_step(relay_id, &row.record, message);
        }
        self.require_budget()?;
        let _permit = self
            .proofs
            .try_acquire()
            .map_err(|_| RingRpcError::RelayProverBusy)?;
        let review = policy.review(self.service, &transaction).await?;
        let fee = message_fee(message);
        let message_hash: [u8; 32] = Sha256::digest(&bytes).into();
        if review.message_hash != message_hash
            || !signers[1..].contains(&review.budget_signer)
            || review.network_fee_lamports != fee
            || review.maximum_debit_lamports < fee
            || review.last_valid_block_height < self.chain.block_height().await?
            || review.last_valid_block_height
                > self.chain.latest_blockhash().await?.last_valid_block_height
        {
            return Err(refused());
        }
        self.rings.check(review.ring).await?;
        let Simulation::Passed { calls } = self.chain.simulate(message).await? else {
            return Err(refused());
        };
        if !message.instructions().iter().any(|instruction| {
            message
                .static_account_keys()
                .get(usize::from(instruction.program_id_index))
                == Some(&review.ring)
        }) && !calls.iter().any(|call| call.program == review.ring)
        {
            return Err(refused());
        }
        let attempt_key = Sha256::new()
            .chain_update(b"zolana/ring-rpc-composition-spend")
            .chain_update(self.service.cluster)
            .chain_update(review.ring.as_ref())
            .chain_update(review.attempt_key)
            .finalize()
            .into();
        let lease = self
            .admit(Admit {
                key: attempt_key,
                relay_id: &relay_id,
                signer: review.budget_signer,
                slot: self.chain.slot().await?,
            })
            .await?
            .lease;
        let staged = async {
            self.fund(Fund {
                lease,
                lamports: review.maximum_debit_lamports,
                fee_allowance: fee,
                landing_rent: review.maximum_debit_lamports - fee,
                budget: self.budget,
            })
            .await?;
            let record = RelayRecord {
                message: message.clone(),
                last_valid_block_height: review.last_valid_block_height,
                state: RelayState::open(
                    Awaiting {
                        signatures: signers[2..]
                            .iter()
                            .zip(&transaction.signatures[2..])
                            .map(|(&signer, &signature)| CollectedSignature { signer, signature })
                            .collect(),
                        awaiting: vec![signers[1]],
                        co_sign: None,
                    },
                    None,
                )?,
            };
            match self
                .store_attempt(StagedAttempt {
                    lease: Some(lease),
                    relay_id: &relay_id,
                    record: &record,
                    slot: self.chain.slot().await?,
                })
                .await?
            {
                AttemptWrite::Stored => Ok(step(relay_id, &record)),
                AttemptWrite::Existing => {
                    let row = self.row(&relay_id).await?;
                    stored_step(relay_id, &row.record, message)
                }
                AttemptWrite::Refused(error) => Err(error),
            }
        };
        self.under_lease(Some(lease), staged).await
    }
}

fn decode(wire: &[u8], payer: Address) -> Result<VersionedTransaction, RingRpcError> {
    if wire.len() > v1::MAX_TRANSACTION_SIZE {
        return Err(refused());
    }
    let transaction: VersionedTransaction = wincode::deserialize(wire).map_err(|_| refused())?;
    transaction.sanitize().map_err(|_| refused())?;
    let VersionedMessage::V1(message) = &transaction.message else {
        return Err(refused());
    };
    let required = usize::from(message.header.num_required_signatures);
    if required < 2
        || transaction.uses_durable_nonce()
        || message.config.priority_fee.unwrap_or_default() != 0
        || message.account_keys.first() != Some(&payer)
        || transaction.signatures.first() != Some(&Signature::default())
        || message.account_keys.iter().collect::<HashSet<_>>().len() != message.account_keys.len()
        || wincode::serialize(&transaction).map_err(|_| refused())? != wire
    {
        return Err(refused());
    }
    let bytes = transaction.message.serialize();
    for (signer, signature) in message.account_keys[1..required]
        .iter()
        .zip(&transaction.signatures[1..])
    {
        if !signature.verify(signer.as_ref(), &bytes) {
            return Err(RelayError::SignerSignature.into());
        }
    }
    Ok(transaction)
}

fn stored_step(
    relay_id: RelayId,
    record: &RelayRecord,
    message: &VersionedMessage,
) -> Result<RelayStep, RingRpcError> {
    if &record.message != message {
        return Err(RingRpcError::StateUnavailable);
    }
    Ok(step(relay_id, record))
}

fn refused() -> RingRpcError {
    RelayError::Proof("composition admission refused".into()).into()
}
