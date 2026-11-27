use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use solana_instruction::{AccountMeta, Instruction};
use spr_client::{compile_message, ComputeBudgetConfig};
use zolana_ring_rpc::{ChainSubmit, CompositionPolicy, CompositionReview};

use super::{
    signers_verify, Fixture, GetRelayStatusRequest, Keypair, RelayError, RelayStatus, RelayStep,
    RingRpcError, ServiceTarget, Signature, Signer, SubmitRelayRequest, VersionedTransaction,
    GET_RELAY_STATUS, LAST_VALID_BLOCK_HEIGHT, RING, SERVICE,
};

const NETWORK_FEE: u64 = 15_000;
const GROSS_DEBIT: u64 = 30_000;

#[derive(Default)]
struct Review(AtomicUsize);

#[async_trait]
impl CompositionPolicy for Review {
    async fn review(
        &self,
        service: ServiceTarget,
        transaction: &VersionedTransaction,
    ) -> Result<CompositionReview, RingRpcError> {
        assert_eq!(service, SERVICE);
        self.0.fetch_add(1, Ordering::Relaxed);
        let message = &transaction.message;
        Ok(CompositionReview {
            ring: RING,
            message_hash: Sha256::digest(message.serialize()).into(),
            attempt_key: [82; 32],
            budget_signer: message.static_account_keys()[2],
            last_valid_block_height: LAST_VALID_BLOCK_HEIGHT,
            network_fee_lamports: NETWORK_FEE,
            maximum_debit_lamports: GROSS_DEBIT,
        })
    }
}

async fn transaction(fixture: &Fixture) -> VersionedTransaction {
    let operation = Keypair::new_from_array([33; 32]);
    let holder = Keypair::new_from_array([35; 32]);
    let instructions = [Instruction {
        program_id: RING,
        accounts: vec![
            AccountMeta::new(operation.pubkey(), true),
            AccountMeta::new_readonly(holder.pubkey(), true),
        ],
        data: vec![1],
    }];
    let message = compile_message(
        &fixture.fee_payer,
        &instructions,
        fixture.chain.latest_blockhash().await.unwrap().blockhash,
        ComputeBudgetConfig::new(100_000),
    )
    .unwrap();
    let bytes = message.serialize();
    let signatures = vec![
        Signature::default(),
        operation.sign_message(&bytes),
        holder.sign_message(&bytes),
    ];
    VersionedTransaction {
        message,
        signatures,
    }
}

#[tokio::test]
async fn composition_preserves_the_original_across_lost_responses() {
    let fixture = Fixture::with_budget(GROSS_DEBIT * 2).await;
    let policy = Review::default();
    let transaction = transaction(&fixture).await;
    let wire = wincode::serialize(&transaction).unwrap();
    let prepared = fixture
        .relay
        .prepare_composition(&wire, &policy)
        .await
        .unwrap();
    let RelayStep::SignMessage { relay_id, message } = &prepared else {
        panic!("preparing must leave the original awaiting submission");
    };
    assert_eq!(message.0, transaction.message.serialize());
    assert_eq!(fixture.outstanding().await, GROSS_DEBIT as i64);
    assert!(fixture.sent().is_empty());
    assert_eq!(
        fixture
            .relay
            .prepare_composition(&wire, &policy)
            .await
            .unwrap(),
        prepared
    );
    assert_eq!(policy.0.load(Ordering::Relaxed), 1);

    fixture.chain.reply_lost.store(true, Ordering::Relaxed);
    let submit = SubmitRelayRequest {
        relay_id: relay_id.clone(),
        signer: transaction.message.static_account_keys()[1].into(),
        signature: transaction.signatures[1].into(),
    };
    assert!(fixture.relay.submit(&submit).await.is_err());
    let [sent] = fixture.sent().try_into().expect("the original sent once");
    assert_eq!(sent.message, transaction.message);
    assert_eq!(sent.signatures[1..], transaction.signatures[1..]);
    assert!(signers_verify(&sent));
    let status: RelayStatus = fixture
        .call(
            GET_RELAY_STATUS,
            GetRelayStatusRequest {
                relay_id: relay_id.clone(),
            },
        )
        .await;
    assert!(matches!(
        status,
        RelayStatus::Confirmed { signature, .. } if signature.0 == sent.signatures[0]
    ));
    assert_eq!(fixture.relay.submit(&submit).await.unwrap(), status);
    assert_eq!(fixture.sent().len(), 1);
}

#[tokio::test]
async fn composition_requires_all_approvals_and_successful_simulation() {
    let fixture = Fixture::with_budget(GROSS_DEBIT).await;
    let policy = Review::default();
    let transaction = transaction(&fixture).await;
    let mut incomplete = transaction.clone();
    incomplete.signatures[2] = Signature::default();
    assert!(matches!(
        fixture
            .relay
            .prepare_composition(&wincode::serialize(&incomplete).unwrap(), &policy)
            .await,
        Err(RingRpcError::Relay(RelayError::SignerSignature))
    ));
    assert_eq!(policy.0.load(Ordering::Relaxed), 0);

    *fixture.chain.simulation_refusal.lock().unwrap() = Some("simulation refused".to_owned());
    assert!(fixture
        .relay
        .prepare_composition(&wincode::serialize(&transaction).unwrap(), &policy)
        .await
        .is_err());
    assert_eq!(policy.0.load(Ordering::Relaxed), 1);
    assert_eq!(fixture.row_count().await, 0);
    assert!(fixture.sent().is_empty());
}
