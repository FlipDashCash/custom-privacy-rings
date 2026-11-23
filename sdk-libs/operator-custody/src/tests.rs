use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use async_trait::async_trait;
use aws_sdk_kms::{
    operation::{
        decrypt::DecryptOutput,
        generate_data_key_without_plaintext::GenerateDataKeyWithoutPlaintextOutput,
        get_public_key::{builders::GetPublicKeyOutputBuilder, GetPublicKeyOutput},
        sign::SignOutput,
    },
    primitives::Blob,
    types::{KeySpec, KeyUsageType, MessageType, SigningAlgorithmSpec},
    Client,
};
use aws_smithy_mocks::{mock, mock_client, Rule, RuleMode};
use ed25519_dalek::{pkcs8::EncodePublicKey, Signer, SigningKey, VerifyingKey};
use solana_address::Address;
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_message::{Message, VersionedMessage};
use solana_signature::Signature;
use spr_client::{rpc::compile_message, ClientError, ComputeBudgetConfig};

use super::{
    presigned_transaction, signed_transaction, CollectedSignature, CustodyError, KeyArn, Kms,
    KmsEndpoint, OperatorSigner, SecretContext, SecretPurpose, SignerRef, WrappedSecret,
};

const ARN: &str = "arn:aws:kms:us-east-1:111122223333:key/abcfef01-2345-6789-abcd-ef0123456789";
const OTHER_ARN: &str = "arn:aws:kms:us-east-1:111122223333:key/other";
const MAINNET: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d";

fn arn() -> KeyArn {
    ARN.parse().unwrap()
}

fn kms(rules: &[&Rule]) -> Kms {
    Kms(mock_client!(aws_sdk_kms, RuleMode::MatchAny, rules))
}

const LOCALNET: Hash = Hash::new_from_array([19; 32]);

fn context() -> SecretContext<'static> {
    SecretContext {
        deployment: "operator-a",
        genesis: &LOCALNET,
        purpose: SecretPurpose::GrantRows,
    }
}

fn public_key_reply(public: VerifyingKey) -> GetPublicKeyOutputBuilder {
    GetPublicKeyOutput::builder()
        .key_id(ARN)
        .key_spec(KeySpec::EccNistEdwards25519)
        .key_usage(KeyUsageType::SignVerify)
        .signing_algorithms(SigningAlgorithmSpec::Ed25519Sha512)
        .public_key(Blob::new(public.to_public_key_der().unwrap().as_bytes()))
}

#[test]
fn key_arn_accepts_a_key_and_refuses_the_rest() {
    assert_eq!(arn().as_str(), ARN);
    for refused in [
        "",
        "arn:aws:kms:us-east-1:111122223333:alias/ring",
        "arn:aws:kms:us-east-1:11112222333:key/abc",
        "arn:aws:kms::111122223333:key/abc",
        "arn:aws:kms:us-east-1:111122223333:key/",
        "arn:aws:kms:us-east-1:111122223333:key/abc:def",
        "arn:aws:s3:us-east-1:111122223333:key/abc",
        "arn:other:kms:us-east-1:111122223333:key/abc",
    ] {
        assert!(
            matches!(refused.parse::<KeyArn>(), Err(CustodyError::KeyArn)),
            "{refused}"
        );
    }
    assert!(serde_json::from_str::<SignerRef>(&format!(
        r#"{{"key_arn":"alias/ring","address":"{}"}}"#,
        Address::default()
    ))
    .is_err());
}

#[tokio::test]
async fn endpoints_refuse_a_remote_local_kms_a_public_cluster_and_a_bad_region() {
    assert!(matches!(
        KmsEndpoint::Localnet("10.0.0.1:8787".parse().unwrap())
            .connect(&LOCALNET)
            .await,
        Err(CustodyError::Endpoint)
    ));
    let loopback = KmsEndpoint::Localnet("127.0.0.1:8787".parse().unwrap());
    assert!(matches!(
        loopback.connect(&MAINNET.parse().unwrap()).await,
        Err(CustodyError::PublicCluster)
    ));
    assert!(loopback.connect(&LOCALNET).await.is_ok());
    assert!(matches!(
        KmsEndpoint::Aws {
            region: "us-east-1.attacker.example/".into()
        }
        .connect(&LOCALNET)
        .await,
        Err(CustodyError::Region)
    ));
}

#[tokio::test]
async fn signing_binds_the_pinned_address_and_raw_message() {
    let key = SigningKey::from_bytes(&[71; 32]);
    let public_key = key.verifying_key();
    let lookup = mock!(Client::get_public_key)
        .match_requests(|request| request.key_id() == Some(ARN))
        .then_output(move || public_key_reply(public_key).build());
    let message = b"message".as_slice();
    let other_message = b"another message".as_slice();
    let signature = key.sign(message).to_bytes();
    let sign = mock!(Client::sign)
        .match_requests(move |request| {
            request.key_id() == Some(ARN)
                && request
                    .message()
                    .is_some_and(|blob| [message, other_message].contains(&blob.as_ref()))
                && request.message_type() == Some(&MessageType::Raw)
                && request.signing_algorithm() == Some(&SigningAlgorithmSpec::Ed25519Sha512)
        })
        .then_output(move || {
            SignOutput::builder()
                .key_id(ARN)
                .signing_algorithm(SigningAlgorithmSpec::Ed25519Sha512)
                .signature(Blob::new(signature))
                .build()
        });
    let kms = kms(&[&lookup, &sign]);
    let mut pin = SignerRef {
        key_arn: arn(),
        address: Address::default(),
    };
    assert!(matches!(
        kms.signer(&pin).await,
        Err(CustodyError::AddressMismatch)
    ));
    pin.address = Address::new_from_array(public_key.to_bytes());
    let signer = kms.signer(&pin).await.unwrap();
    assert_eq!(signer.sign(message).await.unwrap().as_ref(), signature);
    assert!(matches!(
        signer.sign(other_message).await,
        Err(CustodyError::Signature)
    ));
    assert!(matches!(
        signer.sign(&[0; 4097]).await,
        Err(CustodyError::MessageTooLong)
    ));
}

#[tokio::test]
async fn signing_refuses_a_reply_for_another_key() {
    let key = SigningKey::from_bytes(&[71; 32]);
    let public_key = key.verifying_key();
    let lookup =
        mock!(Client::get_public_key).then_output(move || public_key_reply(public_key).build());
    let signature = key.sign(b"message").to_bytes();
    let sign = mock!(Client::sign).then_output(move || {
        SignOutput::builder()
            .key_id(OTHER_ARN)
            .signing_algorithm(SigningAlgorithmSpec::Ed25519Sha512)
            .signature(Blob::new(signature))
            .build()
    });
    let signer = kms(&[&lookup, &sign])
        .signer(&SignerRef {
            key_arn: arn(),
            address: Address::new_from_array(public_key.to_bytes()),
        })
        .await
        .unwrap();
    assert!(matches!(
        signer.sign(b"message").await,
        Err(CustodyError::ReplyKey)
    ));
}

#[tokio::test]
async fn public_keys_refuse_a_foreign_reply_a_wrong_spec_and_a_bad_encoding() {
    let public_key = SigningKey::from_bytes(&[71; 32]).verifying_key();
    let identity = VerifyingKey::from_bytes(&{
        let mut bytes = [0; 32];
        bytes[0] = 1;
        bytes
    })
    .unwrap();
    type Reply = Box<dyn Fn() -> GetPublicKeyOutput + Send + Sync>;
    type Refused = fn(&CustodyError) -> bool;
    let cases: [(Reply, Refused); 5] = [
        (
            Box::new(move || public_key_reply(public_key).key_id(OTHER_ARN).build()),
            |error| matches!(error, CustodyError::ReplyKey),
        ),
        (
            Box::new(move || {
                public_key_reply(public_key)
                    .key_spec(KeySpec::EccNistP256)
                    .build()
            }),
            |error| matches!(error, CustodyError::KeySpec),
        ),
        (
            Box::new(move || {
                public_key_reply(public_key)
                    .key_usage(KeyUsageType::EncryptDecrypt)
                    .build()
            }),
            |error| matches!(error, CustodyError::KeySpec),
        ),
        (
            Box::new(move || {
                public_key_reply(public_key)
                    .public_key(Blob::new([7; 44]))
                    .build()
            }),
            |error| matches!(error, CustodyError::PublicKey),
        ),
        (
            Box::new(move || public_key_reply(identity).build()),
            |error| matches!(error, CustodyError::PublicKey),
        ),
    ];
    for (index, (reply, refused)) in cases.into_iter().enumerate() {
        let lookup = mock!(Client::get_public_key).then_output(reply);
        let error = kms(&[&lookup])
            .signer(&SignerRef {
                key_arn: arn(),
                address: Address::new_from_array(public_key.to_bytes()),
            })
            .await
            .err()
            .unwrap();
        assert!(refused(&error), "case {index} returned {error:?}");
    }
}

#[tokio::test]
async fn provisioning_returns_the_blob_of_the_requested_key() {
    let reply = |key: &'static str, blob: &'static [u8]| {
        mock!(Client::generate_data_key_without_plaintext).then_output(move || {
            GenerateDataKeyWithoutPlaintextOutput::builder()
                .key_id(key)
                .ciphertext_blob(Blob::new(blob))
                .build()
        })
    };
    let wrapped = kms(&[&reply(ARN, b"wrapped")])
        .provision(arn(), context())
        .await
        .unwrap();
    assert_eq!(wrapped.ciphertext, b"wrapped");
    assert_eq!(
        serde_json::to_value(&wrapped).unwrap()["ciphertext"],
        "d3JhcHBlZA=="
    );
    assert!(matches!(
        kms(&[&reply(OTHER_ARN, b"wrapped")])
            .provision(arn(), context())
            .await,
        Err(CustodyError::ReplyKey)
    ));
    assert!(matches!(
        kms(&[&reply(ARN, b"")]).provision(arn(), context()).await,
        Err(CustodyError::Ciphertext)
    ));
}

#[tokio::test]
async fn opening_requires_the_wrapping_key_and_a_nonzero_32_byte_secret() {
    let wrapped = WrappedSecret {
        key_arn: arn(),
        ciphertext: b"wrapped".to_vec(),
    };
    let reply = |key: &'static str, plaintext: &'static [u8]| {
        mock!(Client::decrypt)
            .match_requests(|request| {
                request.key_id() == Some(ARN)
                    && request
                        .ciphertext_blob()
                        .is_some_and(|blob| blob.as_ref() == b"wrapped")
                    && request
                        .encryption_context()
                        .is_some_and(|fields| fields["purpose"] == "grant-rows")
            })
            .then_output(move || {
                DecryptOutput::builder()
                    .key_id(key)
                    .plaintext(Blob::new(plaintext))
                    .build()
            })
    };
    assert_eq!(
        *kms(&[&reply(ARN, &[5; 32])])
            .open(&wrapped, context())
            .await
            .unwrap(),
        [5; 32]
    );
    assert!(matches!(
        kms(&[&reply(OTHER_ARN, &[5; 32])])
            .open(&wrapped, context())
            .await,
        Err(CustodyError::ReplyKey)
    ));
    for plaintext in [[5; 31].as_slice(), &[5; 33], &[0; 32]] {
        assert!(matches!(
            kms(&[&reply(ARN, plaintext)])
                .open(&wrapped, context())
                .await,
            Err(CustodyError::Secret)
        ));
    }
}

#[test]
fn wrapped_secrets_bind_deployment_cluster_role_and_ring() {
    let ring = Address::new_from_array([7; 32]);
    for (purpose, name, bound_ring) in [
        (
            SecretPurpose::Auditor(ring),
            "auditor",
            Some(ring.to_string()),
        ),
        (SecretPurpose::AuditorRoot, "auditor-root", None),
        (SecretPurpose::GrantRows, "grant-rows", None),
    ] {
        let context = SecretContext {
            deployment: "operator-a",
            genesis: &LOCALNET,
            purpose,
        }
        .fields()
        .unwrap();
        assert_eq!(context["application"], "zolana-custom-rings-v1");
        assert_eq!(context["deployment"], "operator-a");
        assert_eq!(context["genesis"], LOCALNET.to_string());
        assert_eq!(context["purpose"], name);
        assert_eq!(context.get("ring"), bound_ring.as_ref());
    }
    for deployment in ["", " ", " operator-a", "operator-a\n"] {
        assert!(matches!(
            SecretContext {
                deployment,
                ..context()
            }
            .fields(),
            Err(CustodyError::Deployment)
        ));
    }
}

struct CountingSigner {
    key: SigningKey,
    calls: Arc<AtomicUsize>,
}

impl CountingSigner {
    fn new(seed: u8) -> Self {
        Self {
            key: SigningKey::from_bytes(&[seed; 32]),
            calls: Arc::default(),
        }
    }
}

#[async_trait]
impl OperatorSigner for CountingSigner {
    fn address(&self) -> Address {
        Address::new_from_array(self.key.verifying_key().to_bytes())
    }

    async fn sign(&self, message: &[u8]) -> Result<Signature, CustodyError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(Signature::from(self.key.sign(message).to_bytes()))
    }
}

fn message(payer: &Address, signer: &Address) -> VersionedMessage {
    let instruction = Instruction::new_with_bytes(
        Address::new_from_array([9; 32]),
        &[],
        vec![AccountMeta::new_readonly(*signer, true)],
    );
    compile_message(
        payer,
        &[instruction],
        Hash::default(),
        ComputeBudgetConfig::for_instruction_count(1),
    )
    .unwrap()
}

#[tokio::test]
async fn transactions_sign_each_required_signer_once_and_nothing_else() {
    let payer = CountingSigner::new(1);
    let owner = CountingSigner::new(2);
    let stranger = CountingSigner::new(3);
    let v1 = message(&payer.address(), &owner.address());

    let transaction = signed_transaction(&v1, &[&payer, &owner, &payer])
        .await
        .unwrap();
    let bytes = v1.serialize();
    for (signer, signature) in [&payer, &owner].into_iter().zip(&transaction.signatures) {
        assert!(signature.verify(signer.address().as_ref(), &bytes));
    }
    assert_eq!(transaction.signatures.len(), 2);
    assert_eq!(payer.calls.load(Ordering::Relaxed), 1);
    assert_eq!(owner.calls.load(Ordering::Relaxed), 1);

    assert!(matches!(
        signed_transaction(&v1, &[&payer, &stranger]).await,
        Err(CustodyError::UnrequiredSigner(address)) if address == stranger.address()
    ));
    let legacy = VersionedMessage::Legacy(Message::new(&[], Some(&payer.address())));
    assert!(matches!(
        signed_transaction(&legacy, &[&payer]).await,
        Err(CustodyError::Transaction(
            ClientError::UnsupportedTransactionVersion
        ))
    ));
    assert_eq!(payer.calls.load(Ordering::Relaxed), 1);
    assert_eq!(stranger.calls.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn presigned_transactions_refuse_a_signature_over_another_message() {
    let payer = CountingSigner::new(1);
    let v1 = message(&payer.address(), &payer.address());
    let collected = |bytes: &[u8]| CollectedSignature {
        signer: payer.address(),
        signature: Signature::from(payer.key.sign(bytes).to_bytes()),
    };
    assert!(presigned_transaction(&v1, &[collected(&v1.serialize())]).is_ok());
    assert!(presigned_transaction(&v1, &[collected(b"another message")]).is_err());
}
