use solana_message::VersionedMessage;
use zolana_ring_client::rpc::{CoSignTransactionRequest, CoSignTransactionResponse};

use super::scope::{ApprovedCall, SignerScope};
use crate::{
    cosign::{CoSignDecision, CoSignError, CoSignPolicy},
    error::RingRpcError,
    landing::{ChainSubmit, InnerCall, Simulation},
    relay::RelayError,
};

#[must_use]
pub(crate) struct DirectCoSign<'a> {
    pub chain: &'a dyn ChainSubmit,
    pub policy: &'a dyn CoSignPolicy,
}

impl DirectCoSign<'_> {
    pub async fn sign(
        self,
        request: &CoSignTransactionRequest,
    ) -> Result<CoSignTransactionResponse, RingRpcError> {
        let signer = self.policy.signer().ok_or(CoSignError::Disabled)?;
        if request.message.0.len() > solana_message::v1::MAX_TRANSACTION_SIZE {
            return Err(CoSignError::MessageScope.into());
        }
        let message: VersionedMessage =
            wincode::deserialize(&request.message.0).map_err(|_| CoSignError::MessageScope)?;
        let scoped = SignerScope {
            ring: request.ring.0,
            signer,
            approved_transfer: request.approved_transfer.clone(),
        }
        .admit(message)?;
        // No rule verifies the ring proof, an unproven frame would hold a window reservation.
        let calls = match self.chain.simulate(scoped.message()).await? {
            Simulation::Passed { calls } => calls,
            Simulation::Refused { reason } => return Err(RelayError::SendRefused { reason }.into()),
        };
        if let Some(approved) = scoped.approved_transfer() {
            approved.ran_once_in(&calls)?;
        }
        let request = scoped.into_request();
        match self.policy.decide(&request).await? {
            CoSignDecision::Signed(signature) => Ok(CoSignTransactionResponse {
                signer: signer.into(),
                signature: signature.into(),
            }),
            CoSignDecision::Refused(refusal) => Err(CoSignError::from(refusal).into()),
        }
    }
}

impl ApprovedCall {
    /// A program can pass simulation and never run the transfer its approval names.
    fn ran_once_in(&self, calls: &[InnerCall]) -> Result<(), CoSignError> {
        let position = self.approval().position;
        let made = calls
            .iter()
            .filter(|call| {
                call.position == position
                    && call.program == self.ring()
                    && call.data == self.data()
                    && call.accounts == self.accounts()
            })
            .count();
        match made {
            1 => Ok(()),
            _ => Err(CoSignError::ApprovedCallNotOnce),
        }
    }
}

#[cfg(test)]
mod tests {
    use custom_ring_interface::{tag, TransferApproval};
    use solana_address::Address;
    use zolana_ring_client::rpc::ApprovedTransfer;

    use super::*;

    const RING: Address = Address::new_from_array([0x11; 32]);
    const ACCOUNTS: [Address; 2] = [
        Address::new_from_array([0x22; 32]),
        Address::new_from_array([0x33; 32]),
    ];
    const DATA: [u8; 2] = [tag::TRANSFER, 8];

    fn call() -> InnerCall {
        InnerCall {
            position: 1,
            program: RING,
            accounts: ACCOUNTS.to_vec(),
            data: DATA.to_vec(),
        }
    }

    #[test]
    fn the_approved_call_runs_exactly_once_under_its_position() {
        let approval = TransferApproval {
            position: 1,
            digest: TransferApproval::digest(&RING, &DATA, &ACCOUNTS).expect("digest"),
        };
        let transfer = ApprovedTransfer {
            data: DATA.to_vec().into(),
            accounts: ACCOUNTS.map(Into::into).to_vec(),
        };
        let approved = ApprovedCall::committed(RING, approval, transfer).expect("committed");
        let unrelated = InnerCall {
            program: ACCOUNTS[0],
            ..call()
        };
        assert_eq!(approved.ran_once_in(&[unrelated.clone(), call()]), Ok(()));

        let reordered = ACCOUNTS.iter().rev().copied().collect();
        for calls in [
            Vec::new(),
            vec![unrelated],
            vec![call(), call()],
            vec![InnerCall {
                position: 0,
                ..call()
            }],
            vec![InnerCall {
                data: vec![tag::TRANSFER],
                ..call()
            }],
            vec![InnerCall {
                accounts: reordered,
                ..call()
            }],
            vec![InnerCall {
                accounts: ACCOUNTS[..1].to_vec(),
                ..call()
            }],
        ] {
            assert_eq!(
                approved.ran_once_in(&calls),
                Err(CoSignError::ApprovedCallNotOnce)
            );
        }
    }
}
