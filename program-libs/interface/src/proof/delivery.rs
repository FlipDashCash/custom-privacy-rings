//! Explicit delivery-v1 profile, separate from every classic transfer key and frame.
use crate::{key_catalog::RingCircuit, CustomRingError};
use spr_hasher::{
    hash_chain::create_hash_chain_4_from_slice,
    primitives::{hash_bytes, right_align},
    Hasher, HasherError, Poseidon,
};

pub const RECIPIENTS_DOMAIN: u32 = 0x43524b31; // CRK1
pub const STATEMENT_DOMAIN: u32 = 0x43524431; // CRD1
pub const PREFIX_LEN: usize = 32;

/// Exact ordered published X25519 u-coordinates, zero suffix through the key capacity.
/// Owner identities alone do not establish delivery; the new circuit binds this value
/// to the recipient wires already used by its original ciphertext relation.
pub fn recipient_commitment(keys: &[[u8; 32]], capacity: usize) -> Result<[u8; 32], HasherError> {
    if !matches!(capacity, 4 | 8) || keys.is_empty() || keys.len() > capacity {
        return Err(HasherError::InvalidInputLength(capacity, keys.len()));
    }
    let mut fields = vec![
        right_align(&RECIPIENTS_DOMAIN.to_be_bytes()),
        right_align(&(keys.len() as u32).to_be_bytes()),
        right_align(&(capacity as u32).to_be_bytes()),
    ];
    for key in keys {
        fields.push(hash_bytes(key)?);
    }
    fields.resize(3 + capacity, [0; 32]);
    fields.push(right_align(&((fields.len() + 1) as u32).to_be_bytes()));
    create_hash_chain_4_from_slice(&fields)
}

pub fn statement(original: &[u8; 32], recipients: &[u8; 32]) -> Result<[u8; 32], HasherError> {
    Poseidon::hashv(&[
        &right_align(&STATEMENT_DOMAIN.to_be_bytes()),
        original,
        recipients,
    ])
}

pub fn split_frame(data: &[u8]) -> Result<(&[u8; 32], &[u8]), CustomRingError> {
    data.split_first_chunk::<PREFIX_LEN>()
        .filter(|(_, frame)| !frame.is_empty())
        .ok_or(CustomRingError::InvalidInstructionData)
}

// No published or historical verification key is changed. Localnet setup is explicit,
// independently pinned, and unavailable in the default profile.
#[cfg(feature = "experimental-delivery-localnet")]
mod local {
    include!(env!("PYROL_RING_DELIVERY_V1_KEYS_RS"));
}

pub fn verifying_key(
    circuit: RingCircuit,
) -> Option<&'static groth16_solana::groth16::Groth16Verifyingkey<'static>> {
    #[cfg(feature = "experimental-delivery-localnet")]
    {
        local::verifying_key(circuit)
    }
    #[cfg(not(feature = "experimental-delivery-localnet"))]
    {
        let _ = circuit;
        None
    }
}

pub fn proving_key_sha256(circuit: RingCircuit) -> Option<[u8; 32]> {
    #[cfg(feature = "experimental-delivery-localnet")]
    {
        local::proving_key_sha256(circuit)
    }
    #[cfg(not(feature = "experimental-delivery-localnet"))]
    {
        let _ = circuit;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_order_count_capacity_and_profile_are_bound() {
        let keys = [[1; 32], [2; 32]];
        let expected = recipient_commitment(&keys, 4).unwrap();
        assert_eq!(
            hex::encode(expected),
            "0831b17480fce60b2323f9af44d420fd3b08f389160163ff3136e710f297a6c2"
        );
        assert_ne!(
            expected,
            recipient_commitment(&[keys[1], keys[0]], 4).unwrap()
        );
        assert_ne!(expected, recipient_commitment(&keys[..1], 4).unwrap());
        assert_ne!(expected, recipient_commitment(&keys, 8).unwrap());
        assert_ne!(
            expected,
            recipient_commitment(&[keys[0], [3; 32]], 4).unwrap()
        );
        assert_ne!([7; 32], statement(&[7; 32], &expected).unwrap());
        assert!(recipient_commitment(&[], 4).is_err());
        assert!(recipient_commitment(&keys, 2).is_err());
        assert!(split_frame(&[0; 32]).is_err());
        assert!(split_frame(&[0; 31]).is_err());
        let body = [0; 33];
        assert_eq!(split_frame(&body).unwrap().1, &[0]);
    }
}

/// No speculative release key is appended to the classic catalog.
/// The separate development profile exposes only the two implemented local shapes.
pub fn ring_key(circuit: RingCircuit) -> Option<crate::key_catalog::RingKey> {
    #[cfg(feature = "experimental-delivery-localnet")]
    {
        let name = match circuit {
            RingCircuit::DeliveryTransfer {
                outputs: 4,
                inputs: 24,
                withdrawals: 4,
                owners: 4,
            } => "custom-ring-delivery-v1-transfer-4-in24-w4-s4",
            RingCircuit::DeliveryTransfer {
                outputs: 8,
                inputs: 16,
                withdrawals: 4,
                owners: 4,
            } => "custom-ring-delivery-v1-transfer-8-in16-w4-s4",
            _ => return None,
        };
        Some(crate::key_catalog::RingKey { circuit, name })
    }
    #[cfg(not(feature = "experimental-delivery-localnet"))]
    {
        let _ = circuit;
        None
    }
}
