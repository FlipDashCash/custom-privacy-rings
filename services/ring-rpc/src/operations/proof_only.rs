//! Ordinary single-payer public-wire sponsorship. Never receives proof witnesses.
//! Preparing reserves and signs, but only a separately authenticated commit can send.
use super::*;
use serde::{Deserialize, Serialize};
use solana_transaction::versioned::VersionedTransaction;
use spr_indexer_api::{Base64String, SerializablePubkey};
use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PrepareOrdinaryProofOnlyRequest {
    pub transaction: Base64String,
    pub authorization_public_key: SerializablePubkey,
    pub authorization_signature: String,
    pub policy_sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RecoverOrdinaryProofOnlyRequest {
    pub id: RelayId,
    pub authorization_public_key: SerializablePubkey,
    pub authorization_signature: String,
    pub policy_sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CommitOrdinaryProofOnlyRequest {
    pub id: RelayId,
    pub wire_sha256: String,
    pub authorization_public_key: SerializablePubkey,
    pub authorization_signature: String,
    pub policy_sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PreparedOrdinaryProofOnly {
    pub id: RelayId,
    pub transaction: Base64String,
    pub policy_sha256: String,
}
#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum OrdinaryProofOnlyRecovery {
    Prepared(PreparedOrdinaryProofOnly),
    Unknown { id: RelayId, state: &'static str },
}
#[derive(Clone, Serialize)]
pub struct OrdinaryProofOnlyCommitted {
    pub id: RelayId,
}

/// Fixed public context; the request cannot choose the service's policy or chain.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdinaryProofOnlyContext {
    pub genesis_hash: String,
    pub ring: SerializablePubkey,
    pub tree: SerializablePubkey,
    pub payer: SerializablePubkey,
    #[serde(skip)]
    pub policy_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OrdinaryProofOnlyReview {
    pub ring: SerializablePubkey,
    pub message_sha256: String,
    pub policy_sha256: String,
    pub last_valid_block_height: String,
    pub maximum_debit_lamports: String,
    pub nullifiers: Vec<String>,
}
#[async_trait]
pub trait OrdinaryProofOnlyPolicy: Send + Sync {
    fn context(&self) -> &OrdinaryProofOnlyContext;
    /// Reconstruct all instructions/accounts/config, verify live pins and public
    /// settlement, bound gross costs, and genuinely simulate the whole message.
    async fn review(
        &self,
        transaction: &Base64String,
    ) -> Result<OrdinaryProofOnlyReview, RingRpcError>;
}
pub struct LocalOrdinaryProofOnlyPolicy {
    executable: std::path::PathBuf,
    policy_file: std::path::PathBuf,
    rpc_origin: String,
    context: OrdinaryProofOnlyContext,
}
fn refused() -> RingRpcError {
    RelayError::Proof("ordinary proof-only admission refused".into()).into()
}
fn hex32(value: &str) -> Result<[u8; 32], RingRpcError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(refused());
    }
    hex::decode(value)
        .map_err(|_| refused())?
        .try_into()
        .map_err(|_| refused())
}
fn message_hash(message: &VersionedMessage) -> String {
    hex::encode(Sha256::digest(message.serialize()))
}
fn identifier(policy: &str, message: &str, key: &Address) -> RelayId {
    let bytes = format!("pyrol/ordinary-proof-only/id/v1\n{policy}\n{message}\n{key}");
    let digest: [u8; 32] = Sha256::digest(bytes.as_bytes()).into();
    RelayId(digest.into())
}
fn authorize(
    context: &OrdinaryProofOnlyContext,
    action: &str,
    message: &str,
    key: &Address,
    signature: &str,
    wire: &str,
) -> Result<(), RingRpcError> {
    let bytes = format!(
        "pyrol/ordinary-proof-only/v1\n{action}\n{}\n{}\n{}\n{}\n{}\n{message}\n{key}\n{wire}",
        context.genesis_hash,
        context.ring.0,
        context.tree.0,
        context.payer.0,
        context.policy_sha256
    );
    let parsed = Signature::from_str(signature).map_err(|_| refused())?;
    if parsed.to_string() != signature || !parsed.verify(key.as_ref(), bytes.as_bytes()) {
        return Err(refused());
    }
    Ok(())
}
impl LocalOrdinaryProofOnlyPolicy {
    pub fn new(
        executable: std::path::PathBuf,
        policy_file: std::path::PathBuf,
    ) -> Result<Self, RingRpcError> {
        if !executable.is_absolute() || !executable.is_file() || !policy_file.is_absolute() {
            return Err(refused());
        }
        // A bounded read of an operator-selected public file, never a request path.
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(&policy_file)
            .map_err(|_| refused())?
            .take(65537)
            .read_to_end(&mut bytes)
            .map_err(|_| refused())?;
        if bytes.is_empty() || bytes.len() > 65536 {
            return Err(refused());
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Config {
            schema: String,
            development_only: bool,
            rpc_url: String,
            policy: OrdinaryProofOnlyContext,
        }
        let config: Config = serde_json::from_slice(&bytes).map_err(|_| refused())?;
        if config.schema != "pyrol-ordinary-proof-only-policy-v1"
            || !config.development_only
            || config.policy.genesis_hash.contains('\n')
        {
            return Err(refused());
        }
        let rpc_origin = crate::local_relay_host::rpc_origin(&config.rpc_url)
            .ok_or_else(refused)?
            .to_owned();
        let mut context = config.policy;
        // Parsing as a pubkey validates the genesis hash's canonical base58 length.
        let genesis = Address::from_str(&context.genesis_hash).map_err(|_| refused())?;
        if genesis.to_string() != context.genesis_hash {
            return Err(refused());
        }
        context.policy_sha256 = hex::encode(Sha256::digest(&bytes));
        Ok(Self {
            executable,
            policy_file,
            rpc_origin,
            context,
        })
    }
}
#[async_trait]
impl OrdinaryProofOnlyPolicy for LocalOrdinaryProofOnlyPolicy {
    fn context(&self) -> &OrdinaryProofOnlyContext {
        &self.context
    }
    async fn review(
        &self,
        transaction: &Base64String,
    ) -> Result<OrdinaryProofOnlyReview, RingRpcError> {
        let mut child = tokio::process::Command::new(&self.executable)
            .arg(&self.policy_file)
            .env_clear()
            .env("PATH", "/usr/local/bin:/usr/bin:/bin")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| refused())?;
        let input = serde_json::to_vec(&serde_json::json!({ "transaction": transaction, "policySha256": self.context.policy_sha256 })).map_err(|_| refused())?;
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
            if output.len() > 8192 || !child.wait().await.map_err(|_| refused())?.success() {
                return Err(refused());
            }
            let review: OrdinaryProofOnlyReview =
                serde_json::from_slice(&output).map_err(|_| refused())?;
            if review.policy_sha256 != self.context.policy_sha256 {
                return Err(refused());
            }
            Ok(review)
        };
        tokio::time::timeout(Duration::from_secs(45), work)
            .await
            .map_err(|_| refused())?
    }
}
impl Relay {
    #[must_use]
    pub fn with_ordinary_proof_only_policy(
        mut self,
        policy: Arc<dyn OrdinaryProofOnlyPolicy>,
    ) -> Self {
        self.proof_only = Some(policy);
        self
    }
    #[must_use]
    pub(crate) fn with_local_ordinary_proof_only_policy(mut self, rpc: &str) -> Self {
        if std::env::var("ZOLANA_RING_LOCAL_ORDINARY_PROOF_ONLY").as_deref() == Ok("1") {
            self.proof_only = std::env::var_os("ZOLANA_RING_ORDINARY_PROOF_ONLY_REVIEWER")
                .zip(std::env::var_os("ZOLANA_RING_ORDINARY_PROOF_ONLY_POLICY"))
                .and_then(|(exe, config)| {
                    LocalOrdinaryProofOnlyPolicy::new(exe.into(), config.into()).ok()
                })
                .filter(|policy| policy.rpc_origin == rpc)
                .map(|policy| Arc::new(policy) as Arc<dyn OrdinaryProofOnlyPolicy>);
        }
        self
    }
    fn ordinary_context(&self, policy: &str) -> Result<&OrdinaryProofOnlyContext, RingRpcError> {
        hex32(policy)?;
        let context = self.proof_only.as_ref().ok_or_else(refused)?.context();
        if context.policy_sha256 != policy
            || context.payer.0 != self.fee_payer()
            || context.genesis_hash != Address::new_from_array(self.service.cluster).to_string()
        {
            return Err(refused());
        }
        Ok(context)
    }
    fn ordinary_row(
        &self,
        id: &RelayId,
        row: &RelayRow,
        key: &Address,
        policy: &str,
    ) -> Result<(), RingRpcError> {
        let context = self.ordinary_context(policy)?;
        let auth = row.record.proof_only.as_ref().ok_or_else(refused)?;
        if auth.authorization_public_key.0 != *key
            || auth.policy_sha256 != policy
            || auth.ring.0 != context.ring.0
            || identifier(policy, &message_hash(&row.record.message), key) != *id
        {
            return Err(refused());
        }
        Ok(())
    }
    /// No payer key use until budgets, per-nullifier holds and exact message are durable.
    pub async fn prepare_ordinary_proof_only(
        &self,
        request: &PrepareOrdinaryProofOnlyRequest,
    ) -> Result<PreparedOrdinaryProofOnly, RingRpcError> {
        let context = self.ordinary_context(&request.policy_sha256)?;
        if !(100..=4096).contains(&request.transaction.0.len()) {
            return Err(refused());
        }
        let transaction: VersionedTransaction =
            wincode::deserialize(&request.transaction.0).map_err(|_| refused())?;
        let message = transaction.message.clone();
        let keys = message.static_account_keys();
        if wincode::serialize(&transaction).map_err(|_| refused())? != request.transaction.0
            || message.header().num_required_signatures != 1
            || message.header().num_readonly_signed_accounts != 0
            || keys.first() != Some(&self.fee_payer())
            || transaction.signatures != [Signature::default()]
            || keys.contains(&request.authorization_public_key.0)
        {
            return Err(refused());
        }
        let hash = message_hash(&message);
        authorize(
            context,
            "prepare",
            &hash,
            &request.authorization_public_key.0,
            &request.authorization_signature,
            "",
        )?;
        let id = identifier(
            &request.policy_sha256,
            &hash,
            &request.authorization_public_key.0,
        );
        if let Some(row) = self.relays().get(&id).await? {
            self.ordinary_row(
                &id,
                &row,
                &request.authorization_public_key.0,
                &request.policy_sha256,
            )?;
            if row.record.message.serialize() != message.serialize() {
                return Err(refused());
            }
            return self.finish_ordinary_prepare(id, row).await;
        }
        let _permit = self
            .proofs
            .try_acquire()
            .map_err(|_| RingRpcError::RelayProverBusy)?;
        let review = self
            .proof_only
            .as_ref()
            .ok_or_else(refused)?
            .review(&request.transaction)
            .await?;
        if review.message_sha256 != hash
            || review.policy_sha256 != request.policy_sha256
            || review.ring.0 != context.ring.0
        {
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
        let mut nullifiers = review
            .nullifiers
            .iter()
            .map(|n| hex32(n))
            .collect::<Result<Vec<_>, _>>()?;
        if nullifiers.is_empty() || nullifiers.len() > 16 || nullifiers.contains(&[0; 32]) {
            return Err(refused());
        }
        nullifiers.sort_unstable();
        if nullifiers.windows(2).any(|w| w[0] == w[1]) {
            return Err(refused());
        }
        let mut attempt = Sha256::new();
        attempt.update(b"zolana/ring-rpc-composition-nullifiers/v1");
        attempt.update(review.ring.0.to_string().as_bytes());
        for n in &nullifiers {
            attempt.update(n);
        }
        let key: [u8; 32] = attempt.finalize().into();
        let fee = message_fee(&message);
        if debit < fee
            || self.chain.block_height().await? > expiry
            || expiry > self.chain.latest_blockhash().await?.last_valid_block_height
        {
            return Err(refused());
        }
        if !matches!(
            self.chain.simulate(&message).await?,
            crate::landing::Simulation::Passed
        ) {
            return Err(refused());
        }
        let claim = self
            .admit(Admit {
                key,
                relay_id: &id,
                signer: request.authorization_public_key.0,
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
                message,
                last_valid_block_height: expiry,
                state: RelayState::PreparedProofOnly,
                proof_only: Some(ProofOnlyAuthorization {
                    authorization_public_key: request.authorization_public_key.clone(),
                    policy_sha256: request.policy_sha256.clone(),
                    ring: review.ring,
                    nullifiers,
                    transaction: None,
                }),
            };
            match self
                .store_attempt(StagedAttempt {
                    lease: Some(lease),
                    relay_id: &id,
                    record: &record,
                    slot: self.chain.slot().await?,
                })
                .await
            {
                Ok(AttemptWrite::Stored | AttemptWrite::Existing) => (),
                Ok(AttemptWrite::Refused(error)) => return Err(error),
                Err(_) => return Err(RingRpcError::StateUnavailable),
            }
            let row = self
                .relays()
                .get(&id)
                .await?
                .ok_or(RingRpcError::StateUnavailable)?;
            self.ordinary_row(
                &id,
                &row,
                &request.authorization_public_key.0,
                &request.policy_sha256,
            )?;
            self.finish_ordinary_prepare(id, row).await
        };
        self.under_lease(Some(lease), staged).await
    }
    /// Retry can only finish the original durable authorization; never rebuild it.
    async fn finish_ordinary_prepare(
        &self,
        id: RelayId,
        mut row: RelayRow,
    ) -> Result<PreparedOrdinaryProofOnly, RingRpcError> {
        loop {
            let auth = row.record.proof_only.as_ref().ok_or_else(refused)?;
            if let Some(wire) = &auth.transaction {
                stored_signature(&row.record)?;
                return Ok(PreparedOrdinaryProofOnly {
                    id,
                    transaction: wire.clone(),
                    policy_sha256: auth.policy_sha256.clone(),
                });
            }
            if !matches!(row.record.state, RelayState::PreparedProofOnly)
                || self.chain.block_height().await? > row.record.last_valid_block_height
            {
                return Err(refused());
            }
            let transaction = VersionedTransaction {
                message: row.record.message.clone(),
                signatures: vec![self.fee_payer_signs(&row.record.message).signature],
            };
            let mut record = row.record.clone();
            record.proof_only.as_mut().ok_or_else(refused)?.transaction = Some(Base64String(
                wincode::serialize(&transaction).map_err(|_| refused())?,
            ));
            match self.relays().commit(&id, row.revision, &record).await? {
                Committed::Written(revision) => row = RelayRow { revision, record },
                Committed::Stale => {
                    row = self
                        .relays()
                        .get(&id)
                        .await?
                        .ok_or(RingRpcError::StateUnavailable)?
                }
            }
        }
    }
    pub async fn recover_ordinary_proof_only(
        &self,
        request: &RecoverOrdinaryProofOnlyRequest,
    ) -> Result<OrdinaryProofOnlyRecovery, RingRpcError> {
        let context = self.ordinary_context(&request.policy_sha256)?;
        let Some(row) = self.relays().get(&request.id).await? else {
            // Not financial evidence; an unknown id cannot authorize another original.
            return Ok(OrdinaryProofOnlyRecovery::Unknown {
                id: request.id.clone(),
                state: "unknown",
            });
        };
        self.ordinary_row(
            &request.id,
            &row,
            &request.authorization_public_key.0,
            &request.policy_sha256,
        )?;
        authorize(
            context,
            "recover",
            &message_hash(&row.record.message),
            &request.authorization_public_key.0,
            &request.authorization_signature,
            "",
        )?;
        // Recovery is read-only with respect to the payer key. A crash between
        // admission and signed-wire persistence needs explicit prepare-original.
        if row
            .record
            .proof_only
            .as_ref()
            .and_then(|a| a.transaction.as_ref())
            .is_none()
        {
            return Err(refused());
        }
        Ok(OrdinaryProofOnlyRecovery::Prepared(
            self.finish_ordinary_prepare(request.id.clone(), row)
                .await?,
        ))
    }
    pub async fn commit_ordinary_proof_only(
        &self,
        request: &CommitOrdinaryProofOnlyRequest,
    ) -> Result<OrdinaryProofOnlyCommitted, RingRpcError> {
        let context = self.ordinary_context(&request.policy_sha256)?;
        hex32(&request.wire_sha256)?;
        let row = self.relays().get(&request.id).await?.ok_or_else(refused)?;
        self.ordinary_row(
            &request.id,
            &row,
            &request.authorization_public_key.0,
            &request.policy_sha256,
        )?;
        let wire = row
            .record
            .proof_only
            .as_ref()
            .and_then(|a| a.transaction.as_ref())
            .ok_or_else(refused)?;
        if hex::encode(Sha256::digest(&wire.0)) != request.wire_sha256 {
            return Err(refused());
        }
        let fee_payer = stored_signature(&row.record)?.ok_or_else(refused)?;
        authorize(
            context,
            "commit",
            &message_hash(&row.record.message),
            &request.authorization_public_key.0,
            &request.authorization_signature,
            &request.wire_sha256,
        )?;
        let row = self.advance(&request.id, row).await?;
        // A CAS winner persists Sent before the network, all retries use this wire.
        if matches!(row.record.state, RelayState::PreparedProofOnly) {
            // Preparation may precede a service-side Ring/auditor policy change.
            // A new commitment must still be admitted by the current service.
            // Recovery stays read-only and an already committed original keeps
            // its existing reconciliation path; this cannot revoke its wire.
            self.rings.check(context.ring.0).await?;
            // A delayed explicit commit must still satisfy current public costs,
            // pins, roots and lifetime. This reviews the SAME message, not a
            // refreshed proof/blockhash; the persisted signed wire stays intact.
            let _permit = self
                .proofs
                .try_acquire()
                .map_err(|_| RingRpcError::RelayProverBusy)?;
            let unsigned = VersionedTransaction {
                message: row.record.message.clone(),
                signatures: vec![Signature::default()],
            };
            let review = self
                .proof_only
                .as_ref()
                .ok_or_else(refused)?
                .review(&Base64String(
                    wincode::serialize(&unsigned).map_err(|_| refused())?,
                ))
                .await?;
            let auth = row.record.proof_only.as_ref().ok_or_else(refused)?;
            let mut nullifiers = review
                .nullifiers
                .iter()
                .map(|n| hex32(n))
                .collect::<Result<Vec<_>, _>>()?;
            nullifiers.sort_unstable();
            let debit: u64 = review
                .maximum_debit_lamports
                .parse()
                .map_err(|_| refused())?;
            let held = self
                .reservations()
                .by_relay(&request.id)
                .await?
                .ok_or_else(refused)?;
            if review.message_sha256 != message_hash(&row.record.message)
                || review.policy_sha256 != auth.policy_sha256
                || review.ring.0 != auth.ring.0
                || nullifiers != auth.nullifiers
                || debit < message_fee(&row.record.message)
                || debit > held.lamports
                || self.chain.block_height().await? > row.record.last_valid_block_height
            {
                return Err(refused());
            }
            if !matches!(
                self.chain.simulate(&row.record.message).await?,
                crate::landing::Simulation::Passed
            ) {
                return Err(refused());
            }
            if let Applied::Written(row) = self
                .transition(&request.id, row, RelayEvent::CommitProofOnly { fee_payer })
                .await?
            {
                self.first_send(&request.id, row).await?;
            }
        }
        Ok(OrdinaryProofOnlyCommitted {
            id: request.id.clone(),
        })
    }
}
/// Tracker observes an already persisted signature; it never creates one.
pub(super) fn stored_signature(
    record: &RelayRecord,
) -> Result<Option<CollectedSignature>, RingRpcError> {
    let auth = record.proof_only.as_ref().ok_or_else(refused)?;
    let Some(wire) = &auth.transaction else {
        return Ok(None);
    };
    let tx: VersionedTransaction = wincode::deserialize(&wire.0).map_err(|_| refused())?;
    let payer = record
        .message
        .static_account_keys()
        .first()
        .copied()
        .ok_or_else(refused)?;
    if tx.message.serialize() != record.message.serialize()
        || tx.signatures.len() != 1
        || record.message.header().num_required_signatures != 1
        || wincode::serialize(&tx).map_err(|_| refused())? != wire.0
        || !tx.signatures[0].verify(payer.as_ref(), &record.message.serialize())
    {
        return Err(refused());
    }
    Ok(Some(CollectedSignature {
        signer: payer,
        signature: tx.signatures[0],
    }))
}
