use std::collections::HashMap;

use aws_sdk_kms::{primitives::Blob, types::DataKeySpec};
use serde::{Deserialize, Serialize};
use solana_address::Address;
use solana_hash::Hash;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

use crate::{CustodyError, KeyArn, Kms};

#[serde_with::serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WrappedSecret {
    pub key_arn: KeyArn,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub ciphertext: Vec<u8>,
}

pub enum SecretPurpose {
    Auditor(Address),
    AuditorRoot,
    GrantRows,
}

/// The encryption context, a secret opens only under the fields it was wrapped with.
pub struct SecretContext<'a> {
    pub deployment: &'a str,
    pub genesis: &'a Hash,
    pub purpose: SecretPurpose,
}

impl SecretContext<'_> {
    pub(crate) fn fields(self) -> Result<HashMap<String, String>, CustodyError> {
        if self.deployment.is_empty() || self.deployment != self.deployment.trim() {
            return Err(CustodyError::Deployment);
        }
        let mut fields = HashMap::from([
            ("application".into(), "zolana-custom-rings-v1".into()),
            ("deployment".into(), self.deployment.into()),
            ("genesis".into(), self.genesis.to_string()),
        ]);
        let purpose = match self.purpose {
            SecretPurpose::Auditor(ring) => {
                fields.insert("ring".into(), ring.to_string());
                "auditor"
            }
            SecretPurpose::AuditorRoot => "auditor-root",
            SecretPurpose::GrantRows => "grant-rows",
        };
        fields.insert("purpose".into(), purpose.into());
        Ok(fields)
    }
}

impl Kms {
    pub async fn provision(
        &self,
        key_arn: KeyArn,
        context: SecretContext<'_>,
    ) -> Result<WrappedSecret, CustodyError> {
        let reply = self
            .0
            .generate_data_key_without_plaintext()
            .key_id(key_arn.as_str())
            .key_spec(DataKeySpec::Aes256)
            .set_encryption_context(Some(context.fields()?))
            .send()
            .await
            .map_err(CustodyError::service)?;
        if reply.key_id() != Some(key_arn.as_str()) {
            return Err(CustodyError::ReplyKey);
        }
        let ciphertext = reply
            .ciphertext_blob
            .map(Blob::into_inner)
            .filter(|ciphertext| !ciphertext.is_empty())
            .ok_or(CustodyError::Ciphertext)?;
        Ok(WrappedSecret {
            key_arn,
            ciphertext,
        })
    }

    pub async fn open(
        &self,
        secret: &WrappedSecret,
        context: SecretContext<'_>,
    ) -> Result<Zeroizing<[u8; 32]>, CustodyError> {
        let reply = self
            .0
            .decrypt()
            .key_id(secret.key_arn.as_str())
            .ciphertext_blob(Blob::new(secret.ciphertext.as_slice()))
            .set_encryption_context(Some(context.fields()?))
            .send()
            .await
            .map_err(CustodyError::service)?;
        let reply_key = reply.key_id() == Some(secret.key_arn.as_str());
        let plaintext = Zeroizing::new(reply.plaintext.ok_or(CustodyError::Secret)?.into_inner());
        if !reply_key {
            return Err(CustodyError::ReplyKey);
        }
        let bytes = <[u8; 32]>::try_from(plaintext.as_slice())
            .map(Zeroizing::new)
            .map_err(|_| CustodyError::Secret)?;
        if bool::from(bytes.ct_eq(&[0; 32])) {
            return Err(CustodyError::Secret);
        }
        Ok(bytes)
    }
}
