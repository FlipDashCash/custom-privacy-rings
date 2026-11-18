use custom_ring_interface::tag;
use custom_ring_sdk::CustomRing;
use solana_address::Address;
use solana_message::{compiled_instruction::CompiledInstruction, VersionedMessage};

use crate::cosign::CoSignError;

#[must_use]
pub(crate) struct SignerScope {
    pub ring: Address,
    pub signer: Address,
}

/// Only `SignerScope::admit` builds one.
#[must_use]
pub(crate) struct ScopedMessage(VersionedMessage);

impl SignerScope {
    /// The signature covers every instruction, the key signs read-only in one co-signer slot alone.
    pub fn admit(self, message: VersionedMessage) -> Result<ScopedMessage, CoSignError> {
        let keys = message.static_account_keys();
        let header = message.header();
        let signed = usize::from(header.num_required_signatures);
        let read_only = signed.saturating_sub(usize::from(header.num_readonly_signed_accounts));
        let signer = keys
            .iter()
            .position(|key| *key == self.signer)
            .filter(|index| (read_only..signed).contains(index))
            .ok_or(CoSignError::MessageScope)?;
        let record = keys
            .iter()
            .position(|key| *key == CustomRing::new(self.ring).co_signer_pda(&self.signer));
        let names = |instruction: &&CompiledInstruction| {
            usize::from(instruction.program_id_index) == signer
                || instruction
                    .accounts
                    .iter()
                    .any(|account| usize::from(*account) == signer)
        };
        let mut naming = message.instructions().iter().filter(names);
        let co_signed = match (naming.next(), naming.next()) {
            (Some(instruction), None) => {
                keys.get(usize::from(instruction.program_id_index)) == Some(&self.ring)
                    && CoSignerSlot { record, signer }.holds_alone(instruction)
            }
            _ => false,
        };
        match co_signed {
            true => Ok(ScopedMessage(message)),
            false => Err(CoSignError::MessageScope),
        }
    }
}

/// The key positions of one `[record, signer]` pair.
struct CoSignerSlot {
    record: Option<usize>,
    signer: usize,
}

impl CoSignerSlot {
    /// The delegate rail demands no co-signer, a signature there would be the delegate's.
    fn holds_alone(&self, instruction: &CompiledInstruction) -> bool {
        let position = |key: usize| {
            let mut places = instruction
                .accounts
                .iter()
                .enumerate()
                .filter(move |(_, account)| usize::from(**account) == key);
            match (places.next(), places.next()) {
                (Some((place, _)), None) => Some(place),
                _ => None,
            }
        };
        let after_record = self
            .record
            .and_then(position)
            .zip(position(self.signer))
            .is_some_and(|(record, signer)| record + 1 == signer);
        after_record && instruction.data.first() != Some(&tag::AUTHORITY_TRANSFER)
    }
}

impl ScopedMessage {
    pub fn into_message(self) -> VersionedMessage {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use solana_instruction::{AccountMeta, Instruction};
    use spr_client::{compile_message, ComputeBudgetConfig};

    use super::*;

    const RING: Address = Address::new_from_array([0x11; 32]);
    const PAYER: Address = Address::new_from_array([0x22; 32]);
    const SIGNER: Address = Address::new_from_array([0x33; 32]);
    const OTHER_PROGRAM: Address = Address::new_from_array([0x44; 32]);

    fn record() -> AccountMeta {
        AccountMeta::new_readonly(CustomRing::new(RING).co_signer_pda(&SIGNER), false)
    }

    fn instruction(program_id: Address, accounts: Vec<AccountMeta>) -> Instruction {
        Instruction {
            program_id,
            accounts,
            data: vec![tag::TRANSFER],
        }
    }

    fn admitted(fee_payer: Address, instructions: &[Instruction]) -> bool {
        let message = compile_message(
            &fee_payer,
            instructions,
            solana_hash::Hash::default(),
            ComputeBudgetConfig::new(1),
        )
        .expect("message");
        SignerScope {
            ring: RING,
            signer: SIGNER,
        }
        .admit(message)
        .is_ok()
    }

    #[test]
    fn the_signer_signs_read_only_in_one_co_signer_slot() {
        let signer = || AccountMeta::new_readonly(SIGNER, true);
        let payer = || AccountMeta::new(PAYER, true);
        let co_signed = instruction(RING, vec![payer(), record(), signer()]);
        let unrelated = instruction(OTHER_PROGRAM, vec![payer()]);
        assert!(admitted(PAYER, &[unrelated, co_signed.clone()]));

        let delegate_frame = Instruction {
            data: vec![tag::AUTHORITY_TRANSFER],
            ..co_signed.clone()
        };
        for refused in [
            vec![instruction(RING, vec![payer()])],
            vec![instruction(
                RING,
                vec![payer(), record(), AccountMeta::new(SIGNER, true)],
            )],
            vec![instruction(RING, vec![payer(), signer()])],
            vec![instruction(RING, vec![record(), payer(), signer()])],
            vec![instruction(RING, vec![record(), signer(), signer()])],
            vec![instruction(OTHER_PROGRAM, vec![record(), signer()])],
            vec![
                co_signed.clone(),
                instruction(OTHER_PROGRAM, vec![signer()]),
            ],
            vec![
                co_signed.clone(),
                instruction(
                    OTHER_PROGRAM,
                    vec![AccountMeta::new_readonly(SIGNER, false)],
                ),
            ],
            vec![co_signed.clone(), co_signed.clone()],
            vec![delegate_frame],
        ] {
            assert!(!admitted(PAYER, &refused));
        }
        assert!(!admitted(SIGNER, &[co_signed]));
    }
}
