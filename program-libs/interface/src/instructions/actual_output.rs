//! Compact certificate and native SPP reservation proof for an actual-output settlement.

use spr_interface::instruction::{instruction_data::transact::TransactProof, DepositAssetKind};
use wincode::{containers, len::FixIntLen, SchemaRead, SchemaWrite};

use crate::{CustomRingError, InstructionTag, ParsedTransact, RingTransactData};

pub const ACTUAL_OUTPUT_FORMAT: u8 = 4;
const RECOVERY_MAGIC: &[u8; 8] = b"CRADv001";
const DESCRIPTOR_SLOT: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq, SchemaRead, SchemaWrite)]
pub struct ActualOutputIxData {
    pub format: u8,
    pub minimum_output: u64,
    /// Explicit reserve retained by the signing SOL authority. Must be zero for SPL.
    pub keep_lamports: u64,
    pub owner_utxo_hash: [u8; 32],
    /// Safe only for this no-input certificate, never publish a spend's private blinding.
    pub private_tx_blinding: [u8; 32],
    /// Owner commitment of the stock frame's zero-SOL change. Never deposited by settlement.
    pub zero_change_owner_utxo_hash: [u8; 32],
    pub asset: DepositAssetKind,
    /// Tagged stock Ring transfer frame with the unused SPP proof omitted.
    #[wincode(with = "containers::Vec<u8, FixIntLen<u16>>")]
    pub certificate: Vec<u8>,
    pub nullifier_root_index: u16,
    pub reservation_proof: TransactProof,
}

impl ActualOutputIxData {
    pub fn parse(data: &[u8]) -> Result<Self, CustomRingError> {
        let request: Self = wincode::deserialize_exact(data)
            .map_err(|_| CustomRingError::InvalidInstructionData)?;
        if request.format != ACTUAL_OUTPUT_FORMAT
            || request.minimum_output == 0
            || (request.asset != DepositAssetKind::Sol && request.keep_lamports != 0)
        {
            return Err(CustomRingError::InvalidInstructionData);
        }
        Ok(request)
    }

    pub fn parse_certificate(&self) -> Result<ActualOutputCertificate<'_>, CustomRingError> {
        let (tag, frame) = self
            .certificate
            .split_first()
            .ok_or(CustomRingError::InvalidInstructionData)?;
        if *tag != InstructionTag::Transfer as u8 {
            return Err(CustomRingError::InvalidSettlementAccounts);
        }
        let (transact, offset) = RingTransactData::parse_certificate(frame)?;
        Ok(ActualOutputCertificate {
            transact,
            wire: &self.certificate,
            omitted_spp_proof_at: 1 + offset,
        })
    }
}

pub struct ActualOutputCertificate<'a> {
    transact: ParsedTransact<'a>,
    wire: &'a [u8],
    omitted_spp_proof_at: usize,
}

impl<'a> ActualOutputCertificate<'a> {
    pub fn transact(&self) -> &ParsedTransact<'a> {
        &self.transact
    }

    /// Embed the stock frame so recipients and the auditor can open the descriptor from SPP history.
    pub fn recovery_body(&self) -> Vec<u8> {
        let proof = bytemuck::bytes_of(self.transact.data().spp_proof);
        let (prefix, suffix) = self.wire.split_at(self.omitted_spp_proof_at);
        let mut body = Vec::with_capacity(RECOVERY_MAGIC.len() + 1 + self.wire.len() + proof.len());
        body.extend_from_slice(RECOVERY_MAGIC);
        body.push(DESCRIPTOR_SLOT);
        body.extend_from_slice(prefix);
        body.extend_from_slice(proof);
        body.extend_from_slice(suffix);
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_wire_pins_token_kind_and_refuses_other_layouts() {
        let request = ActualOutputIxData {
            format: ACTUAL_OUTPUT_FORMAT,
            minimum_output: (1 << 53) + 3,
            keep_lamports: 0,
            owner_utxo_hash: [7; 32],
            private_tx_blinding: [9; 32],
            zero_change_owner_utxo_hash: [8; 32],
            asset: DepositAssetKind::Spl {
                spl_interface_bump: 253,
            },
            certificate: vec![4, 5, 6],
            nullifier_root_index: 0x1234,
            reservation_proof: TransactProof {
                a: [8; 32],
                b: [9; 128],
                c: [10; 32],
            },
        };
        let encoded = wincode::serialize(&request).unwrap();
        assert_eq!(encoded[0], 4);
        assert_eq!(&encoded[1..9], &request.minimum_output.to_le_bytes());
        assert_eq!(&encoded[9..17], &[0; 8]);
        assert_eq!(&encoded[17..49], &[7; 32]);
        assert_eq!(&encoded[49..81], &[9; 32]);
        assert_eq!(&encoded[81..113], &[8; 32]);
        assert_eq!(&encoded[113..120], &[1, 253, 3, 0, 4, 5, 6]);
        assert_eq!(&encoded[120..122], &[0x34, 0x12]);
        assert_eq!(&encoded[122..154], &[8; 32]);
        assert_eq!(&encoded[154..282], &[9; 128]);
        assert_eq!(&encoded[282..], &[10; 32]);
        assert_eq!(ActualOutputIxData::parse(&encoded), Ok(request));
        for invalid in [
            [&[u8::MAX], &encoded[1..]].concat(),
            encoded[..encoded.len() - 1].to_vec(),
            [&encoded[..1], &[0; 8], &encoded[9..]].concat(),
            [&encoded[..9], &1u64.to_le_bytes(), &encoded[17..]].concat(),
            [&encoded[..], &[0]].concat(),
        ] {
            assert!(ActualOutputIxData::parse(&invalid).is_err());
        }
    }
}
