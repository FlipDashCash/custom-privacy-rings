use std::{net::SocketAddr, sync::Arc, time::Duration};

use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_kms::{
    config::{timeout::TimeoutConfig, Credentials},
    primitives::Blob,
    types::{KeySpec, KeyUsageType, MessageType, SigningAlgorithmSpec},
    Client,
};
use ed25519_dalek::{pkcs8::DecodePublicKey, VerifyingKey};
use solana_address::Address;
use solana_hash::Hash;
use solana_signature::Signature;

use crate::{key_arn::is_region, CustodyError, KeyArn, OperatorSigner, SignerRef, SigningIdentity};

const OPERATION_TIMEOUT: Duration = Duration::from_secs(10);
/// KMS refuses a longer raw message.
const RAW_MESSAGE_LIMIT: usize = 4096;
const LOCALNET_REGION: &str = "us-east-1";
/// Mainnet, testnet and devnet.
const PUBLIC_CLUSTERS: [&str; 3] = [
    "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d",
    "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY",
    "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KmsEndpoint {
    Aws { region: String },
    Localnet(SocketAddr),
}

#[derive(Clone)]
pub struct Kms(pub(crate) Client);

struct KmsSigner {
    client: Client,
    pin: SignerRef,
}

impl KmsEndpoint {
    /// A local KMS never signs or decrypts for a public cluster.
    pub async fn connect(&self, genesis: &Hash) -> Result<Kms, CustodyError> {
        if matches!(self, Self::Localnet(_))
            && PUBLIC_CLUSTERS.contains(&genesis.to_string().as_str())
        {
            return Err(CustodyError::PublicCluster);
        }
        self.client().await
    }

    /// Reads public key material only, no cluster is involved.
    pub async fn signer_ref(&self, key_arn: KeyArn) -> Result<SignerRef, CustodyError> {
        let address = self.client().await?.public_key(&key_arn).await?;
        Ok(SignerRef { key_arn, address })
    }

    async fn client(&self) -> Result<Kms, CustodyError> {
        let config = match self {
            Self::Aws { region } => {
                if !is_region(region) {
                    return Err(CustodyError::Region);
                }
                let suffix = if region.starts_with("cn-") {
                    "amazonaws.com.cn"
                } else {
                    "amazonaws.com"
                };
                let config = aws_config::defaults(BehaviorVersion::latest())
                    .region(Region::new(region.clone()))
                    .load()
                    .await;
                // An ambient endpoint override would receive each Decrypt plaintext.
                aws_sdk_kms::config::Builder::from(&config)
                    .endpoint_url(format!("https://kms.{region}.{suffix}"))
            }
            Self::Localnet(endpoint) => {
                if !endpoint.ip().is_loopback() {
                    return Err(CustodyError::Endpoint);
                }
                aws_sdk_kms::Config::builder()
                    .behavior_version_latest()
                    .region(Region::new(LOCALNET_REGION))
                    .credentials_provider(Credentials::new(
                        "local", "local", None, None, "localnet",
                    ))
                    .endpoint_url(format!("http://{endpoint}"))
            }
        };
        let timeout = TimeoutConfig::builder()
            .operation_timeout(OPERATION_TIMEOUT)
            .build();
        Ok(Kms(Client::from_conf(
            config.timeout_config(timeout).build(),
        )))
    }
}

impl Kms {
    pub async fn signer(&self, pin: &SignerRef) -> Result<SigningIdentity, CustodyError> {
        if self.public_key(&pin.key_arn).await? != pin.address {
            return Err(CustodyError::AddressMismatch);
        }
        Ok(Arc::new(KmsSigner {
            client: self.0.clone(),
            pin: pin.clone(),
        }))
    }

    async fn public_key(&self, key_arn: &KeyArn) -> Result<Address, CustodyError> {
        let key = self
            .0
            .get_public_key()
            .key_id(key_arn.as_str())
            .send()
            .await
            .map_err(CustodyError::service)?;
        if key.key_id() != Some(key_arn.as_str()) {
            return Err(CustodyError::ReplyKey);
        }
        if key.key_spec() != Some(&KeySpec::EccNistEdwards25519)
            || key.key_usage() != Some(&KeyUsageType::SignVerify)
            || !key
                .signing_algorithms()
                .contains(&SigningAlgorithmSpec::Ed25519Sha512)
        {
            return Err(CustodyError::KeySpec);
        }
        let public = key
            .public_key()
            .and_then(|der| VerifyingKey::from_public_key_der(der.as_ref()).ok())
            .filter(|public| !public.is_weak())
            .ok_or(CustodyError::PublicKey)?;
        Ok(Address::new_from_array(public.to_bytes()))
    }
}

#[async_trait]
impl OperatorSigner for KmsSigner {
    fn address(&self) -> Address {
        self.pin.address
    }

    async fn sign(&self, message: &[u8]) -> Result<Signature, CustodyError> {
        if message.len() > RAW_MESSAGE_LIMIT {
            return Err(CustodyError::MessageTooLong);
        }
        let reply = self
            .client
            .sign()
            .key_id(self.pin.key_arn.as_str())
            .message_type(MessageType::Raw)
            .signing_algorithm(SigningAlgorithmSpec::Ed25519Sha512)
            .message(Blob::new(message))
            .send()
            .await
            .map_err(CustodyError::service)?;
        if reply.key_id() != Some(self.pin.key_arn.as_str())
            || reply.signing_algorithm() != Some(&SigningAlgorithmSpec::Ed25519Sha512)
        {
            return Err(CustodyError::ReplyKey);
        }
        reply
            .signature()
            .and_then(|signature| Signature::try_from(signature.as_ref()).ok())
            .filter(|signature| signature.verify(self.pin.address.as_ref(), message))
            .ok_or(CustodyError::Signature)
    }
}
