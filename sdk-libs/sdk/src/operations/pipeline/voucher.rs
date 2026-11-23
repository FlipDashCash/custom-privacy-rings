//! The voucher of one transfer, mirror `prover/circuits/policy/voucher.go`.

use custom_ring_interface::domain::VouchMessage;
use spr_client::OwnerAuthorization;
use spr_hasher::primitives::solana_owner_identity;
use spr_keypair::SigningKey;

use crate::{request::VoucherParams, RingError};

/// The Ed25519 key the auditor stream seals.
#[derive(Clone, Copy)]
pub(super) struct VoucherKey(pub [u8; 32]);

impl VoucherKey {
    pub fn of(voucher: &OwnerAuthorization<'_>) -> Result<Self, RingError> {
        Ok(Self(voucher.public_key().as_ed25519()?))
    }

    /// The member the voucher's records sit under.
    pub fn identity(self) -> Result<[u8; 32], RingError> {
        Ok(solana_owner_identity(&self.0)?)
    }
}

pub(super) struct Vouch<'a> {
    pub voucher: Option<OwnerAuthorization<'a>>,
    pub message: VouchMessage<'a>,
}

impl Vouch<'_> {
    /// The circuit verifies a signature on every proof, a fresh key signs where nobody vouches.
    pub fn sign(self) -> Result<VoucherParams, RingError> {
        let digest = self.message.digest()?;
        let throwaway;
        let voucher = match self.voucher {
            Some(voucher) => voucher,
            None => {
                throwaway = SigningKey::new_ed25519();
                OwnerAuthorization::SigningKey(&throwaway)
            }
        };
        let signature = match voucher {
            OwnerAuthorization::SigningKey(key) => key.sign_message(&digest)?,
            OwnerAuthorization::Prepared { owner, signature } => {
                match owner.verify_message(&digest, &signature) {
                    true => signature,
                    false => return Err(RingError::VoucherSignatureMismatch),
                }
            }
        };
        Ok(VoucherParams {
            public_key: VoucherKey::of(&voucher)?.0,
            signature,
        })
    }
}
