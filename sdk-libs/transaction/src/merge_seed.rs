use spr_hasher::primitives::right_align;
use spr_keypair::NullifierKey;
use spr_transaction::instructions::merge::{merge_dummy_nullifier, merge_output_blinding};

use crate::RingRailError;

/// The owner's nullifier secret as a merge record carries it.
#[derive(Clone)]
pub struct MergeSeed(NullifierKey);

impl MergeSeed {
    /// A zero seed would open every derivation in public.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, RingRailError> {
        match bytes.split_first() {
            Some((0, secret)) if bytes != [0; 32] => secret
                .try_into()
                .map(|secret| Self(NullifierKey::from_secret(secret)))
                .map_err(|_| RingRailError::MergeSeed),
            _ => Err(RingRailError::MergeSeed),
        }
    }

    pub fn from_nullifier_key(nullifier_key: &NullifierKey) -> Self {
        Self(nullifier_key.clone())
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        right_align(&self.0.secret())
    }

    pub fn output_blinding(&self, first_nullifier: &[u8; 32]) -> Result<[u8; 32], RingRailError> {
        Ok(merge_output_blinding(&self.0, first_nullifier)?)
    }

    pub fn dummy_nullifier(
        &self,
        first_nullifier: &[u8; 32],
        slot_index: u8,
    ) -> Result<[u8; 32], RingRailError> {
        Ok(merge_dummy_nullifier(&self.0, first_nullifier, slot_index)?)
    }
}
