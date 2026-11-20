//! Journal/security regression tests using fake chain/proof infrastructure, not E2E evidence.
#[allow(dead_code)]
#[path = "common/relay.rs"]
mod relay_fixture;
#[path = "common/store.rs"]
mod store_fixture;
use async_trait::async_trait;
use relay_fixture::{RecordingChain, RecordingProver};
use sha2::{Digest, Sha256};
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use spr_client::{compile_message, ComputeBudgetConfig};
use spr_indexer_api::Base64String;
use std::sync::{atomic::Ordering, Arc};
use zolana_ring_client::rpc::{
    GetRelayStatusRequest, RelayStatus, ServiceTarget, SubmitRelayRequest,
};
use zolana_ring_rpc::{
    ChainOutcome, ChainSubmit, CompositionPolicy, CompositionReview, Migrator,
    PrepareCompositionRequest, Relay, RelayBudget, RelayParts, RingRpcError, ServedRings, Store,
    STORE_CONNECTIONS,
};
const RING: Address = Address::new_from_array([6; 32]);
const SERVICE: ServiceTarget = ServiceTarget {
    cluster: [7; 32],
    service: Address::new_from_array([9; 32]),
};
struct Policy;
#[async_trait]
impl CompositionPolicy for Policy {
    async fn review(
        &self,
        r: &PrepareCompositionRequest,
    ) -> Result<CompositionReview, RingRpcError> {
        let tx: VersionedTransaction = wincode::deserialize(&r.transaction.0).unwrap();
        Ok(CompositionReview {
            ring: RING.into(),
            message_sha256: hex::encode(Sha256::digest(tx.message.serialize())),
            attempt_key: "11".repeat(32),
            last_valid_block_height: "500".into(),
            maximum_debit_lamports: "100000".into(),
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
async fn fixture(
    budget: u64,
    enable: bool,
) -> (
    Relay,
    Arc<RecordingChain>,
    PrepareCompositionRequest,
    Keypair,
) {
    let payer = Keypair::new_from_array([31; 32]);
    let base = Keypair::new_from_array([32; 32]);
    let chain = Arc::new(RecordingChain::default());
    let message = compile_message(
        &payer.pubkey(),
        &[Instruction {
            program_id: RING,
            accounts: vec![AccountMeta::new_readonly(base.pubkey(), true)],
            data: vec![1],
        }],
        chain.latest_blockhash().await.unwrap().blockhash,
        ComputeBudgetConfig::new(400000),
    )
    .unwrap();
    let tx = VersionedTransaction {
        signatures: vec![
            Signature::default(),
            base.sign_message(&message.serialize()),
        ],
        message,
    };
    let request = PrepareCompositionRequest {
        transaction: Base64String(wincode::serialize(&tx).unwrap()),
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
        if enable {
            relay.with_composition_policy(Arc::new(Policy))
        } else {
            relay
        },
        chain,
        request,
        base,
    )
}
#[tokio::test]
async fn composition_preparation_is_durable_idempotent_and_never_broadcasts() {
    let (relay, chain, request, base) = fixture(1000000, true).await;
    let prepared = relay.prepare_composition(&request).await.unwrap();
    let signed: VersionedTransaction = wincode::deserialize(&prepared.transaction.0).unwrap();
    assert!(signed.signatures[0].verify(
        signed.message.static_account_keys()[0].as_ref(),
        &signed.message.serialize()
    ));
    relay.track().await.unwrap();
    assert!(chain.sent().is_empty());
    let retry = relay.prepare_composition(&request).await.unwrap();
    assert_eq!(retry.transaction.0, prepared.transaction.0);
    assert_eq!(retry.relay_id, prepared.relay_id);
    let status = relay
        .submit(&SubmitRelayRequest {
            relay_id: prepared.relay_id,
            signer: base.pubkey().into(),
            signature: signed.signatures[1].into(),
        })
        .await
        .unwrap();
    assert!(matches!(status, RelayStatus::Confirmed { .. }));
    assert_eq!(chain.sent().len(), 1);
}
#[tokio::test]
async fn composition_external_broadcast_is_reconciled_before_expiration() {
    let (relay, chain, request, _) = fixture(1000000, true).await;
    let prepared = relay.prepare_composition(&request).await.unwrap();
    let signed: VersionedTransaction = wincode::deserialize(&prepared.transaction.0).unwrap();
    chain.send(&signed).await.unwrap();
    chain.height.store(501, Ordering::Relaxed);
    let status = relay
        .status(&GetRelayStatusRequest {
            relay_id: prepared.relay_id,
        })
        .await
        .unwrap();
    assert!(matches!(status, RelayStatus::Confirmed { .. }));
}
#[tokio::test]
async fn composition_unsigned_expiry_never_sends() {
    let (relay, chain, request, _) = fixture(1000000, true).await;
    let prepared = relay.prepare_composition(&request).await.unwrap();
    chain.height.store(501, Ordering::Relaxed);
    assert!(matches!(
        relay
            .status(&GetRelayStatusRequest {
                relay_id: prepared.relay_id
            })
            .await
            .unwrap(),
        RelayStatus::Failed { .. }
    ));
    assert!(chain.sent().is_empty());
}
#[tokio::test]
async fn composition_rejects_missing_policy_budget_signature_and_invalid_proof() {
    for (budget, enable) in [(1000000, false), (0, true)] {
        let (relay, chain, request, _) = fixture(budget, enable).await;
        assert!(relay.prepare_composition(&request).await.is_err());
        assert!(chain.sent().is_empty());
    }
    let (relay, chain, mut request, _) = fixture(1000000, true).await;
    let mut tx: VersionedTransaction = wincode::deserialize(&request.transaction.0).unwrap();
    tx.signatures[1] = Signature::default();
    request.transaction.0 = wincode::serialize(&tx).unwrap();
    assert!(relay.prepare_composition(&request).await.is_err());
    assert!(chain.sent().is_empty());
    let (relay, chain, request, _) = fixture(1000000, true).await;
    *chain.simulation_refusal.lock().unwrap() = Some("invalid proof".into());
    assert!(relay.prepare_composition(&request).await.is_err());
    assert!(chain.sent().is_empty());
}
#[tokio::test]
async fn composition_external_failure_is_terminal() {
    let (relay, chain, request, _) = fixture(1000000, true).await;
    let prepared = relay.prepare_composition(&request).await.unwrap();
    *chain.outcome.lock().unwrap() = Some(Some(ChainOutcome::Failed {
        reason: "rollback".into(),
    }));
    assert!(matches!(
        relay
            .status(&GetRelayStatusRequest {
                relay_id: prepared.relay_id
            })
            .await
            .unwrap(),
        RelayStatus::Failed { .. }
    ));
    assert!(chain.sent().is_empty());
}

#[tokio::test]
async fn composition_json_rpc_exposes_two_phase_contract() {
    let (relay, chain, request, _) = fixture(1000000, true).await;
    let module = zolana_ring_rpc::relay_module(Arc::new(relay)).unwrap();
    let serde_json::Value::Object(params) = serde_json::to_value(&request).unwrap() else {
        panic!("object");
    };
    let prepared: zolana_ring_rpc::PreparedComposition =
        module.call("prepareComposition", params).await.unwrap();
    assert!(!prepared.transaction.0.is_empty());
    assert!(serde_json::to_value(&prepared.relay_id)
        .unwrap()
        .is_string());
    assert!(chain.sent().is_empty());
}
