use solana_address::Address;
use solana_message::VersionedMessage;
use solana_signature::Signature;
use solana_signer::{PresignerError, Signer, SignerError};
use solana_transaction::versioned::VersionedTransaction;
use spr_client::{sign_transaction, ClientError};

use crate::{CustodyError, OperatorSigner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectedSignature {
    pub signer: Address,
    pub signature: Signature,
}

impl Signer for CollectedSignature {
    fn try_pubkey(&self) -> Result<Address, SignerError> {
        Ok(self.signer)
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        if self.signature.verify(self.signer.as_ref(), message) {
            Ok(self.signature)
        } else {
            Err(PresignerError::VerificationFailure.into())
        }
    }

    fn is_interactive(&self) -> bool {
        false
    }
}

/// Each required signer must authenticate the same serialized message.
pub fn presigned_transaction(
    message: &VersionedMessage,
    signatures: &[CollectedSignature],
) -> Result<VersionedTransaction, CustodyError> {
    let signers: Vec<&dyn Signer> = signatures
        .iter()
        .map(|signer| signer as &dyn Signer)
        .collect();
    Ok(sign_transaction(message.clone(), &signers)?)
}

/// Refuses a legacy message or an unrequired signer ahead of the first KMS call.
pub async fn signed_transaction(
    message: &VersionedMessage,
    signers: &[&dyn OperatorSigner],
) -> Result<VersionedTransaction, CustodyError> {
    let VersionedMessage::V1(_) = message else {
        return Err(ClientError::UnsupportedTransactionVersion.into());
    };
    let required = usize::from(message.header().num_required_signatures);
    let mut unique = Vec::<(Address, &dyn OperatorSigner)>::with_capacity(signers.len());
    for signer in signers {
        let address = signer.address();
        if unique.iter().any(|(kept, _)| *kept == address) {
            continue;
        }
        if !message
            .static_account_keys()
            .iter()
            .take(required)
            .any(|key| *key == address)
        {
            return Err(CustodyError::UnrequiredSigner(address));
        }
        unique.push((address, *signer));
    }
    let bytes = message.serialize();
    let mut signatures = Vec::with_capacity(unique.len());
    for (signer, operator) in unique {
        signatures.push(CollectedSignature {
            signer,
            signature: operator.sign(&bytes).await?,
        });
    }
    presigned_transaction(message, &signatures)
}
