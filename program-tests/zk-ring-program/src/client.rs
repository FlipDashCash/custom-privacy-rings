use pinocchio::Address;
use solana_instruction::{AccountMeta, Instruction};
use spr_program::{CompressedProof, ProgramOwner};

use crate::{owner_seeds, tag, ID};

pub fn owner(ring_program: &Address) -> ProgramOwner {
    ProgramOwner::find(&owner_seeds(ring_program), &ID)
}

/// A ring transfer forwarded under the proof of `tag`.
pub struct ProvenInstruction {
    pub tag: u8,
    pub proof: CompressedProof,
    /// The ring transfer staged under the commitment of `proof`.
    pub transfer: Instruction,
}

impl ProvenInstruction {
    pub fn create(proof: CompressedProof, transfer: Instruction) -> Self {
        Self {
            tag: tag::CREATE,
            proof,
            transfer,
        }
    }

    pub fn update(proof: CompressedProof, transfer: Instruction) -> Self {
        Self {
            tag: tag::UPDATE,
            proof,
            transfer,
        }
    }

    pub fn instruction(self) -> Instruction {
        let Self {
            tag,
            proof,
            transfer,
        } = self;
        let mut accounts = vec![AccountMeta::new_readonly(transfer.program_id, false)];
        accounts.extend(transfer.accounts);
        Instruction {
            program_id: ID,
            accounts,
            data: [
                [tag].as_slice(),
                proof.a.as_slice(),
                proof.b.as_slice(),
                proof.c.as_slice(),
                transfer.data.as_slice(),
            ]
            .concat(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_ring_has_its_own_owner() {
        let [ring, other] = [42, 43].map(|byte| Address::new_from_array([byte; 32]));
        assert_ne!(owner(&ring).pda(), owner(&other).pda());
    }
}
