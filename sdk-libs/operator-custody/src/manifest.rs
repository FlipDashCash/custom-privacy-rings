use serde::Deserialize;

use crate::{SignerRef, WrappedSecret};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcCustody {
    pub auditor: WrappedSecret,
    pub service: SignerRef,
    pub storage_authority: Option<SignerRef>,
    pub co_signer: Option<SignerRef>,
    pub grant_rows: Option<WrappedSecret>,
    pub fee_payer: Option<SignerRef>,
    pub enrollment_applier: Option<SignerRef>,
}

impl RpcCustody {
    pub fn signers(&self) -> impl Iterator<Item = &SignerRef> {
        [
            Some(&self.service),
            self.storage_authority.as_ref(),
            self.co_signer.as_ref(),
            self.fee_payer.as_ref(),
            self.enrollment_applier.as_ref(),
        ]
        .into_iter()
        .flatten()
    }
}
