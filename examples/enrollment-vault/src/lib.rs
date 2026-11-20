use custom_ring_interface::pda_enrollment::{PdaEnrollment, VaultEnrollmentPolicy};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};

pinocchio::entrypoint!(process_instruction);

const OWNER_SEED: &[u8] = b"owner";

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let (owner, bump) = Address::derive_program_address(&[OWNER_SEED], program_id)
        .ok_or(ProgramError::InvalidSeeds)?;
    let enrollment = PdaEnrollment::parse(accounts, data)?;
    let vault = enrollment.vault();
    if !vault.owned_by(program_id) {
        return Err(ProgramError::InvalidAccountData);
    }
    let [authority, ring, tracking]: [Address; 3] = {
        let state = vault.try_borrow()?;
        let ([authority, ring, tracking], []) = state.as_chunks::<32>() else {
            return Err(ProgramError::InvalidAccountData);
        };
        [*authority, *ring, *tracking].map(Address::new_from_array)
    };
    let bump = [bump];
    let seeds = [Seed::from(OWNER_SEED), Seed::from(&bump)];
    enrollment.invoke_signed(
        VaultEnrollmentPolicy {
            manager: &authority,
            vault: &owner,
            ring: &ring,
            tracking_registry: &tracking,
        },
        Signer::from(seeds.as_slice()),
    )
}
