//! The voucher of one transfer, mirror `prover/circuits/policy/voucher.go`.

use custom_ring_interface::domain::VouchMessage;
use spr_hasher::primitives::solana_owner_identity;
use spr_keypair::{PublicKey, SigningKey};

use crate::{request::VoucherParams, RingError};

/// Signs one transfer's vouch digest after the transfer is fixed, the key stays with its wallet.
pub trait VouchSigner: Sync {
    fn public_key(&self) -> PublicKey;

    /// Ed25519 over the 32 digest bytes with no header.
    fn sign_vouch(&self, digest: &[u8; 32]) -> Result<[u8; 64], RingError>;
}

impl VouchSigner for SigningKey {
    fn public_key(&self) -> PublicKey {
        self.pubkey()
    }

    fn sign_vouch(&self, digest: &[u8; 32]) -> Result<[u8; 64], RingError> {
        Ok(self.sign_message(digest)?)
    }
}

/// The Ed25519 key the auditor stream seals.
#[derive(Clone, Copy)]
pub(super) struct VoucherKey(pub [u8; 32]);

impl VoucherKey {
    pub fn of(voucher: &dyn VouchSigner) -> Result<Self, RingError> {
        Ok(Self(voucher.public_key().as_ed25519()?))
    }

    /// The member the voucher's records sit under.
    pub fn identity(self) -> Result<[u8; 32], RingError> {
        Ok(solana_owner_identity(&self.0)?)
    }
}

pub(super) struct Vouch<'a> {
    pub voucher: Option<&'a dyn VouchSigner>,
    pub message: VouchMessage<'a>,
}

impl Vouch<'_> {
    /// The circuit verifies a signature on every proof, a fresh key signs where nobody vouches.
    pub fn sign(self) -> Result<VoucherParams, RingError> {
        let digest = self.message.digest()?;
        let throwaway = SigningKey::new_ed25519();
        let voucher = self.voucher.unwrap_or(&throwaway);
        let signature = voucher.sign_vouch(&digest)?;
        if !voucher.public_key().verify_message(&digest, &signature) {
            return Err(RingError::VoucherSignatureMismatch);
        }
        Ok(VoucherParams {
            public_key: VoucherKey::of(voucher)?.0,
            signature,
        })
    }
}
