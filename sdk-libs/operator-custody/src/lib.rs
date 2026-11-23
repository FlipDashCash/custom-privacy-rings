mod key_arn;
mod kms;
mod manifest;
mod secret;
mod transaction;

use std::sync::Arc;

use async_trait::async_trait;
use aws_sdk_kms::error::DisplayErrorContext;
use serde::{Deserialize, Serialize};
use solana_address::Address;
use solana_signature::Signature;
use thiserror::Error;

pub use key_arn::KeyArn;
pub use kms::{Kms, KmsEndpoint};
pub use manifest::RpcCustody;
pub use secret::{SecretContext, SecretPurpose, WrappedSecret};
pub use transaction::{presigned_transaction, signed_transaction, CollectedSignature};

#[derive(Debug, Error)]
pub enum CustodyError {
    #[error(transparent)]
    Transaction(#[from] spr_client::ClientError),
    #[error("invalid KMS key ARN")]
    KeyArn,
    #[error("invalid AWS region")]
    Region,
    #[error("local KMS requires a loopback address")]
    Endpoint,
    #[error("local KMS refuses a public cluster")]
    PublicCluster,
    #[error("KMS answered for another key or algorithm")]
    ReplyKey,
    #[error("KMS key is not an Ed25519 signing key")]
    KeySpec,
    #[error("invalid KMS public key")]
    PublicKey,
    #[error("KMS key differs from the pinned address")]
    AddressMismatch,
    #[error("message exceeds the KMS raw signing limit")]
    MessageTooLong,
    #[error("invalid KMS signature")]
    Signature,
    #[error("the message does not require signer {0}")]
    UnrequiredSigner(Address),
    #[error("deployment name is empty or padded")]
    Deployment,
    #[error("KMS returned no wrapped secret")]
    Ciphertext,
    #[error("wrapped secret does not open to a nonzero 32-byte key")]
    Secret,
    #[error("KMS operation failed, {}", DisplayErrorContext(.0.as_ref()))]
    Service(Box<dyn std::error::Error + Send + Sync>),
}

impl CustodyError {
    pub(crate) fn service(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Service(Box::new(error))
    }
}

#[async_trait]
pub trait OperatorSigner: Send + Sync {
    fn address(&self) -> Address;
    async fn sign(&self, message: &[u8]) -> Result<Signature, CustodyError>;
}

pub type SigningIdentity = Arc<dyn OperatorSigner>;

#[serde_with::serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignerRef {
    pub key_arn: KeyArn,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub address: Address,
}

#[cfg(feature = "test-fixtures")]
#[async_trait]
impl OperatorSigner for solana_keypair::Keypair {
    fn address(&self) -> Address {
        solana_signer::Signer::pubkey(self)
    }

    async fn sign(&self, message: &[u8]) -> Result<Signature, CustodyError> {
        Ok(solana_signer::Signer::sign_message(self, message))
    }
}

#[cfg(test)]
mod tests;
