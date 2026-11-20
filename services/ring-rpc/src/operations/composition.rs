//! Local-only composition admission; the official journal owns signing and landing.
use super::*;
use serde::{Deserialize, Serialize};
use solana_transaction::versioned::VersionedTransaction;
use spr_indexer_api::{Base64String, SerializablePubkey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PrepareCompositionRequest {
    pub transaction: Base64String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PreparedComposition {
    pub relay_id: RelayId,
    pub transaction: Base64String,
}

/// Public review output from an operator-trusted, fail-closed reviewer.
/// Debit is reserved gross, without netting speculative reimbursement or rent refunds.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CompositionReview {
    pub ring: SerializablePubkey,
    pub message_sha256: String,
    pub attempt_key: String,
    pub last_valid_block_height: String,
    pub maximum_debit_lamports: String,
}

#[async_trait]
pub trait CompositionPolicy: Send + Sync {
    /// Must reconstruct the entire message from current canonical accounts, enforce
    /// programs/ABIs, proof-bound recipients/amounts, owner exclusion, budgets,
    /// non-extractable transient accounts, chain identity and blockhash validity.
    /// Required Ring co-signers must already have signed; unsupported policies refuse.
    async fn review(
        &self,
        request: &PrepareCompositionRequest,
    ) -> Result<CompositionReview, RingRpcError>;
}

pub struct LocalCompositionPolicy {
    executable: std::path::PathBuf,
}

fn refused() -> RingRpcError {
    RelayError::Proof("composition admission refused".into()).into()
}

impl LocalCompositionPolicy {
    pub fn new(executable: std::path::PathBuf) -> Result<Self, RingRpcError> {
        if !executable.is_absolute() || !executable.is_file() {
            return Err(refused());
        }
        Ok(Self { executable })
    }
}

#[async_trait]
impl CompositionPolicy for LocalCompositionPolicy {
    async fn review(
        &self,
        request: &PrepareCompositionRequest,
    ) -> Result<CompositionReview, RingRpcError> {
        // No shell, request-selected program, fee-payer secret, inherited stdin or stderr.
        let mut child = tokio::process::Command::new(&self.executable)
            .env_clear()
            .env("PATH", "/usr/local/bin:/usr/bin:/bin")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| refused())?;
        let input = serde_json::to_vec(request).map_err(|_| refused())?;
        let work = async {
            let mut stdin = child.stdin.take().ok_or_else(refused)?;
            stdin.write_all(&input).await.map_err(|_| refused())?;
            drop(stdin);
            let mut output = Vec::new();
            child
                .stdout
                .take()
                .ok_or_else(refused)?
                .take(8193)
                .read_to_end(&mut output)
                .await
                .map_err(|_| refused())?;
            if output.len() > 8192 {
                return Err(refused());
            }
            if !child.wait().await.map_err(|_| refused())?.success() {
                return Err(refused());
            }
            serde_json::from_slice(&output).map_err(|_| refused())
        };
        tokio::time::timeout(Duration::from_secs(45), work)
            .await
            .map_err(|_| refused())?
    }
}

impl Relay {
    #[must_use]
    pub fn with_composition_policy(mut self, policy: Arc<dyn CompositionPolicy>) -> Self {
        self.composition = Some(policy);
        self
    }

    /// Explicitly local development only. Hosted services must deploy their own review policy.
    #[must_use]
    pub(crate) fn with_local_composition_policy(mut self) -> Self {
        if std::env::var("ZOLANA_RING_LOCAL_COMPOSITION").as_deref() == Ok("1") {
            self.composition = std::env::var_os("ZOLANA_RING_COMPOSITION_REVIEWER")
                .and_then(|path| LocalCompositionPolicy::new(path.into()).ok())
                .map(|policy| Arc::new(policy) as Arc<dyn CompositionPolicy>);
        }
        self
    }

    /// Prepare never broadcasts. The client checkpoints the signed wire, then submits
    /// its final non-payer signature through the existing submitRelay method.
    pub async fn prepare_composition(
        &self,
        request: &PrepareCompositionRequest,
    ) -> Result<PreparedComposition, RingRpcError> {
        let policy = self.composition.as_ref().ok_or_else(refused)?;
        if !(100..=4096).contains(&request.transaction.0.len()) {
            return Err(refused());
        }
        let mut transaction: VersionedTransaction =
            wincode::deserialize(&request.transaction.0).map_err(|_| refused())?;
        let message = transaction.message.clone();
        let bytes = message.serialize();
        let keys = message.static_account_keys();
        let count = usize::from(message.header().num_required_signatures);
        if count < 2
            || count > 16
            || count > keys.len()
            || transaction.signatures.len() != count
            || keys[0] != self.fee_payer()
        {
            return Err(refused());
        }
        let signatures: Vec<_> = keys[1..count]
            .iter()
            .zip(&transaction.signatures[1..])
            .map(|(&signer, &signature)| CollectedSignature { signer, signature })
            .collect();
        if signatures
            .iter()
            .any(|s| !s.signature.verify(s.signer.as_ref(), &bytes))
        {
            return Err(RelayError::SignerSignature.into());
        }
        if transaction.signatures[0] != Signature::default()
            && !transaction.signatures[0].verify(self.fee_payer().as_ref(), &bytes)
        {
            return Err(refused());
        }
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        let relay_id = RelayId(digest.into());
        // Retry uses the durable record and does not re-review spent transients.
        if let Some(row) = self.relays().get(&relay_id).await? {
            if row.record.message.serialize() != bytes
                || matches!(row.record.state, RelayState::Failed { .. })
            {
                return Err(refused());
            }
            // This exact durable row already passed admission. No secret-key
            // signing occurs while reviewing a new or refused composition.
            transaction.signatures[0] = self.fee_payer_signs(&message).signature;
            return Ok(PreparedComposition {
                relay_id,
                transaction: Base64String(wincode::serialize(&transaction).map_err(|_| refused())?),
            });
        }
        let _permit = self
            .proofs
            .try_acquire()
            .map_err(|_| RingRpcError::RelayProverBusy)?;
        let review = policy.review(request).await?;
        if review.message_sha256 != hex::encode(digest) {
            return Err(refused());
        }
        self.rings.check(review.ring.0).await?;
        let expiry: u64 = review
            .last_valid_block_height
            .parse()
            .map_err(|_| refused())?;
        let debit: u64 = review
            .maximum_debit_lamports
            .parse()
            .map_err(|_| refused())?;
        let key: [u8; 32] = hex::decode(&review.attempt_key)
            .map_err(|_| refused())?
            .try_into()
            .map_err(|_| refused())?;
        let fee = message_fee(&message);
        if debit < fee
            || self.chain.block_height().await? > expiry
            || expiry > self.chain.latest_blockhash().await?.last_valid_block_height
        {
            return Err(refused());
        }
        // Real on-chain program/proof verification, no signature or cryptography bypass.
        if !matches!(
            self.chain.simulate(&message).await?,
            crate::landing::Simulation::Passed
        ) {
            return Err(refused());
        }
        let signer = signatures.last().ok_or_else(refused)?.signer;
        let claim = self
            .admit(Admit {
                key,
                relay_id: &relay_id,
                signer,
                slot: self.chain.slot().await?,
            })
            .await?;
        let lease = claim.lease;
        let staged = async {
            self.fund(Fund {
                lease,
                lamports: debit,
                fee_allowance: fee,
                landing_rent: debit - fee,
                budget: self.budget,
            })
            .await?;
            let record = RelayRecord {
                proof_only: None,
                message: message.clone(),
                last_valid_block_height: expiry,
                state: RelayState::AwaitingSignature(Awaiting {
                    signatures: signatures[..signatures.len() - 1].to_vec(),
                    awaiting: vec![signer],
                    co_sign: None,
                }),
            };
            match self
                .store_attempt(StagedAttempt {
                    lease: Some(lease),
                    relay_id: &relay_id,
                    record: &record,
                    slot: self.chain.slot().await?,
                })
                .await
            {
                Ok(AttemptWrite::Stored | AttemptWrite::Existing) => (),
                Ok(AttemptWrite::Refused(error)) => return Err(error),
                Err(_) => return Err(RingRpcError::StateUnavailable),
            }
            // Policy, genuine simulation, budgets and durable nullifier
            // reservation must all succeed before using the payer's key.
            transaction.signatures[0] = self.fee_payer_signs(&message).signature;
            Ok(PreparedComposition {
                relay_id,
                transaction: Base64String(wincode::serialize(&transaction).map_err(|_| refused())?),
            })
        };
        self.under_lease(Some(lease), staged).await
    }
}
