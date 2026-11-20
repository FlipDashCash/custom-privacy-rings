//! Backend service regressions. Fake chain fixtures do not qualify real proofs or Localnet.
#[allow(dead_code)]
#[path = "common/relay.rs"]
mod relay_fixture;
#[path = "common/store.rs"]
mod store_fixture;
use async_trait::async_trait;
use relay_fixture::{RecordingChain, RecordingProver};
use sha2::{Digest, Sha256};
use solana_address::Address;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spr_client::{compile_message, ComputeBudgetConfig};
use spr_indexer_api::Base64String;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use zolana_ring_client::rpc::{GetRelayStatusRequest, ServiceTarget, SubmitRelayRequest};
use zolana_ring_rpc::{
    ChainSubmit, CommitOrdinaryProofOnlyRequest, Migrator, OrdinaryProofOnlyContext,
    OrdinaryProofOnlyPolicy, OrdinaryProofOnlyRecovery, OrdinaryProofOnlyReview,
    PrepareOrdinaryProofOnlyRequest, RecoverOrdinaryProofOnlyRequest, Relay, RelayBudget,
    RelayParts, RingRpcError, ServedRings, Store, STORE_CONNECTIONS,
};
const RING: Address = Address::new_from_array([6; 32]);
const SERVICE: ServiceTarget = ServiceTarget {
    cluster: [7; 32],
    service: Address::new_from_array([9; 32]),
};
struct Policy {
    context: OrdinaryProofOnlyContext,
}
#[async_trait]
impl OrdinaryProofOnlyPolicy for Policy {
    fn context(&self) -> &OrdinaryProofOnlyContext {
        &self.context
    }
    async fn review(&self, wire: &Base64String) -> Result<OrdinaryProofOnlyReview, RingRpcError> {
        let tx: VersionedTransaction = wincode::deserialize(&wire.0).unwrap();
        Ok(OrdinaryProofOnlyReview {
            ring: RING.into(),
            policy_sha256: self.context.policy_sha256.clone(),
            message_sha256: hex::encode(Sha256::digest(tx.message.serialize())),
            last_valid_block_height: "500".into(),
            maximum_debit_lamports: "100000".into(),
            nullifiers: vec!["11".repeat(32), "22".repeat(32)],
        })
    }
}
struct Rings;
#[async_trait]
impl ServedRings for Rings {
    async fn check(&self, ring: Address) -> Result<(), RingRpcError> {
        assert_eq!(ring, RING);
        Ok(())
    }
    async fn delegate(&self, _: Address) -> Result<Option<Address>, RingRpcError> {
        Ok(None)
    }
}
fn authorization(
    c: &OrdinaryProofOnlyContext,
    action: &str,
    wire: &Base64String,
    key: &Keypair,
    signed_hash: &str,
) -> String {
    let tx: VersionedTransaction = wincode::deserialize(&wire.0).unwrap();
    let bytes = format!(
        "pyrol/ordinary-proof-only/v1\n{action}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{signed_hash}",
        c.genesis_hash,
        c.ring.0,
        c.tree.0,
        c.payer.0,
        c.policy_sha256,
        hex::encode(Sha256::digest(tx.message.serialize())),
        key.pubkey()
    );
    key.sign_message(bytes.as_bytes()).to_string()
}
async fn fixture(
    budget: u64,
    enabled: bool,
) -> (
    Relay,
    Arc<RecordingChain>,
    PrepareOrdinaryProofOnlyRequest,
    Keypair,
    OrdinaryProofOnlyContext,
) {
    let payer = Keypair::new_from_array([31; 32]);
    let key = Keypair::new_from_array([32; 32]);
    let chain = Arc::new(RecordingChain::default());
    let context = OrdinaryProofOnlyContext {
        genesis_hash: Address::new_from_array(SERVICE.cluster).to_string(),
        ring: RING.into(),
        tree: Address::new_from_array([8; 32]).into(),
        payer: payer.pubkey().into(),
        policy_sha256: "ab".repeat(32),
    };
    let tx = VersionedTransaction {
        message: compile_message(
            &payer.pubkey(),
            &[Instruction {
                program_id: RING,
                accounts: vec![],
                data: vec![55],
            }],
            chain.latest_blockhash().await.unwrap().blockhash,
            ComputeBudgetConfig::new(400000),
        )
        .unwrap(),
        signatures: vec![Signature::default()],
    };
    let wire = Base64String(wincode::serialize(&tx).unwrap());
    let request = PrepareOrdinaryProofOnlyRequest {
        authorization_signature: authorization(&context, "prepare", &wire, &key, ""),
        transaction: wire,
        authorization_public_key: key.pubkey().into(),
        policy_sha256: context.policy_sha256.clone(),
    };
    let relay = Relay::new(
        RelayParts {
            store: store_fixture::test_store().await,
            prover: Arc::new(RecordingProver::default()),
            fee_payer: payer,
            chain: chain.clone(),
        },
        SERVICE,
    )
    .with_budget(RelayBudget {
        total_lamports: budget,
        signer_lamports: budget,
    })
    .with_rings(Arc::new(Rings));
    (
        if enabled {
            relay.with_ordinary_proof_only_policy(Arc::new(Policy {
                context: context.clone(),
            }))
        } else {
            relay
        },
        chain,
        request,
        key,
        context,
    )
}
#[tokio::test]
async fn prepare_recover_tracker_and_legacy_submit_never_broadcast() {
    let (relay, chain, request, key, c) = fixture(1000000, true).await;
    let prepared = relay.prepare_ordinary_proof_only(&request).await.unwrap();
    relay.track().await.unwrap();
    assert!(chain.sent().is_empty());
    let retry = relay.prepare_ordinary_proof_only(&request).await.unwrap();
    assert_eq!(retry.transaction.0, prepared.transaction.0);
    let recovery = relay
        .recover_ordinary_proof_only(&RecoverOrdinaryProofOnlyRequest {
            id: prepared.id.clone(),
            authorization_public_key: key.pubkey().into(),
            authorization_signature: authorization(&c, "recover", &request.transaction, &key, ""),
            policy_sha256: c.policy_sha256.clone(),
        })
        .await
        .unwrap();
    let OrdinaryProofOnlyRecovery::Prepared(recovered) = recovery else {
        panic!("lost durable original")
    };
    assert_eq!(recovered.transaction.0, prepared.transaction.0);
    let signed: VersionedTransaction = wincode::deserialize(&prepared.transaction.0).unwrap();
    assert!(signed.signatures[0].verify(c.payer.0.as_ref(), &signed.message.serialize()));
    assert!(relay
        .submit(&SubmitRelayRequest {
            relay_id: prepared.id.clone(),
            signer: c.payer,
            signature: signed.signatures[0].into()
        })
        .await
        .is_err());
    assert!(chain.sent().is_empty());
}
#[tokio::test]
async fn only_explicit_bound_commit_sends_original_wire() {
    let (relay, chain, request, key, c) = fixture(1000000, true).await;
    let prepared = relay.prepare_ordinary_proof_only(&request).await.unwrap();
    let hash = hex::encode(Sha256::digest(&prepared.transaction.0));
    let commit = CommitOrdinaryProofOnlyRequest {
        id: prepared.id.clone(),
        wire_sha256: hash.clone(),
        authorization_public_key: key.pubkey().into(),
        authorization_signature: authorization(&c, "commit", &request.transaction, &key, &hash),
        policy_sha256: c.policy_sha256.clone(),
    };
    let mut wrong = commit.clone();
    wrong.wire_sha256 = "00".repeat(32);
    assert!(relay.commit_ordinary_proof_only(&wrong).await.is_err());
    wrong = commit.clone();
    wrong.authorization_signature = request.authorization_signature.clone();
    assert!(relay.commit_ordinary_proof_only(&wrong).await.is_err());
    assert!(chain.sent().is_empty());
    relay.commit_ordinary_proof_only(&commit).await.unwrap();
    assert_eq!(
        wincode::serialize(&chain.sent()[0]).unwrap(),
        prepared.transaction.0
    );
    relay.commit_ordinary_proof_only(&commit).await.unwrap();
    assert_eq!(chain.sent().len(), 1);
}
#[tokio::test]
async fn expiry_and_unknown_do_not_authorize_new_original() {
    let (relay, chain, request, key, c) = fixture(1000000, true).await;
    let prepared = relay.prepare_ordinary_proof_only(&request).await.unwrap();
    chain.height.store(501, Ordering::Relaxed);
    relay.track().await.unwrap();
    assert!(chain.sent().is_empty());
    assert!(relay
        .status(&GetRelayStatusRequest {
            relay_id: prepared.id.clone()
        })
        .await
        .is_err());
    let hash = hex::encode(Sha256::digest(&prepared.transaction.0));
    relay
        .commit_ordinary_proof_only(&CommitOrdinaryProofOnlyRequest {
            id: prepared.id.clone(),
            wire_sha256: hash.clone(),
            authorization_public_key: key.pubkey().into(),
            authorization_signature: authorization(&c, "commit", &request.transaction, &key, &hash),
            policy_sha256: c.policy_sha256,
        })
        .await
        .unwrap();
    assert!(chain.sent().is_empty());
}
#[tokio::test]
async fn unauthorized_recovery_or_second_key_cannot_read_or_reserve_same_nullifiers() {
    let (relay, chain, request, _, c) = fixture(1000000, true).await;
    let prepared = relay.prepare_ordinary_proof_only(&request).await.unwrap();
    let stranger = Keypair::new_from_array([33; 32]);
    assert!(relay
        .recover_ordinary_proof_only(&RecoverOrdinaryProofOnlyRequest {
            id: prepared.id.clone(),
            authorization_public_key: stranger.pubkey().into(),
            authorization_signature: authorization(
                &c,
                "recover",
                &request.transaction,
                &stranger,
                ""
            ),
            policy_sha256: c.policy_sha256.clone()
        })
        .await
        .is_err());
    let mut other = request.clone();
    other.authorization_public_key = stranger.pubkey().into();
    other.authorization_signature =
        authorization(&c, "prepare", &request.transaction, &stranger, "");
    assert!(relay.prepare_ordinary_proof_only(&other).await.is_err());
    assert!(chain.sent().is_empty());
}
#[tokio::test]
async fn disabled_budget_auth_and_failed_simulation_refuse() {
    for (budget, enable) in [(1000000, false), (0, true)] {
        let (relay, chain, request, _, _) = fixture(budget, enable).await;
        assert!(relay.prepare_ordinary_proof_only(&request).await.is_err());
        assert!(chain.sent().is_empty());
    }
    let (relay, chain, mut request, _, _) = fixture(1000000, true).await;
    let original = request.authorization_signature.clone();
    request.authorization_signature = Signature::default().to_string();
    assert!(relay.prepare_ordinary_proof_only(&request).await.is_err());
    request.authorization_signature = original;
    *chain.simulation_refusal.lock().unwrap() = Some("invalid proof".into());
    assert!(relay.prepare_ordinary_proof_only(&request).await.is_err());
    assert!(chain.sent().is_empty());
}

#[tokio::test]
async fn delayed_commit_rechecks_real_chain_simulation_before_sent_marker() {
    let (relay, chain, request, key, c) = fixture(1000000, true).await;
    let prepared = relay.prepare_ordinary_proof_only(&request).await.unwrap();
    let hash = hex::encode(Sha256::digest(&prepared.transaction.0));
    let commit = CommitOrdinaryProofOnlyRequest {
        id: prepared.id.clone(),
        wire_sha256: hash.clone(),
        authorization_public_key: key.pubkey().into(),
        authorization_signature: authorization(&c, "commit", &request.transaction, &key, &hash),
        policy_sha256: c.policy_sha256,
    };
    *chain.simulation_refusal.lock().unwrap() = Some("changed public state".into());
    assert!(relay.commit_ordinary_proof_only(&commit).await.is_err());
    relay.track().await.unwrap();
    assert!(chain.sent().is_empty());
}

struct SwitchableRings(Arc<AtomicBool>);
#[async_trait]
impl ServedRings for SwitchableRings {
    async fn check(&self, ring: Address) -> Result<(), RingRpcError> {
        assert_eq!(ring, RING);
        if self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(RingRpcError::StateUnavailable)
        }
    }
    async fn delegate(&self, _: Address) -> Result<Option<Address>, RingRpcError> {
        Ok(None)
    }
}

#[tokio::test]
async fn delayed_commit_rechecks_served_ring_without_losing_the_original() {
    let (relay, chain, request, key, c) = fixture(1000000, true).await;
    let enabled = Arc::new(AtomicBool::new(true));
    let relay = relay.with_rings(Arc::new(SwitchableRings(enabled.clone())));
    let prepared = relay.prepare_ordinary_proof_only(&request).await.unwrap();
    let hash = hex::encode(Sha256::digest(&prepared.transaction.0));
    let commit = CommitOrdinaryProofOnlyRequest {
        id: prepared.id.clone(),
        wire_sha256: hash.clone(),
        authorization_public_key: key.pubkey().into(),
        authorization_signature: authorization(&c, "commit", &request.transaction, &key, &hash),
        policy_sha256: c.policy_sha256.clone(),
    };
    enabled.store(false, Ordering::SeqCst);
    assert!(relay.commit_ordinary_proof_only(&commit).await.is_err());
    relay.track().await.unwrap();
    assert!(chain.sent().is_empty());
    // The policy change neither destroys nor replaces the already signed wire.
    let recovery = relay
        .recover_ordinary_proof_only(&RecoverOrdinaryProofOnlyRequest {
            id: prepared.id.clone(),
            authorization_public_key: key.pubkey().into(),
            authorization_signature: authorization(&c, "recover", &request.transaction, &key, ""),
            policy_sha256: c.policy_sha256.clone(),
        })
        .await
        .unwrap();
    let OrdinaryProofOnlyRecovery::Prepared(recovered) = recovery else {
        panic!("lost durable original")
    };
    assert_eq!(recovered.transaction.0, prepared.transaction.0);
    enabled.store(true, Ordering::SeqCst);
    relay.commit_ordinary_proof_only(&commit).await.unwrap();
    assert_eq!(chain.sent().len(), 1);
    assert_eq!(
        wincode::serialize(&chain.sent()[0]).unwrap(),
        prepared.transaction.0
    );
}
