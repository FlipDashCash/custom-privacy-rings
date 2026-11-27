use custom_ring_interface::{
    pda_enrollment::{PdaEnrollment, VaultEnrollmentPolicy},
    tag,
    transfer_cpi::RingTransferCpi,
};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};

pinocchio::entrypoint!(process_instruction);

const OWNER_SEED: &[u8] = b"owner";

struct VaultState {
    manager: Address,
    ring: Address,
    tracking: Address,
}

impl VaultState {
    fn load(program_id: &Address, vault: &AccountView) -> Result<Self, ProgramError> {
        if !vault.owned_by(program_id) {
            return Err(ProgramError::InvalidAccountData);
        }
        let state = vault.try_borrow()?;
        let ([manager, ring, tracking], []) = state.as_chunks::<32>() else {
            return Err(ProgramError::InvalidAccountData);
        };
        let [manager, ring, tracking] = [*manager, *ring, *tracking].map(Address::new_from_array);
        Ok(Self {
            manager,
            ring,
            tracking,
        })
    }
}

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let (owner, bump) = Address::derive_program_address(&[OWNER_SEED], program_id)
        .ok_or(ProgramError::InvalidSeeds)?;
    let bump = [bump];
    let seeds = [Seed::from(OWNER_SEED), Seed::from(&bump)];
    let signer = Signer::from(seeds.as_slice());
    // A ring transfer leads with its tag, an enrollment with `ENROLL_PDA`.
    if data.first() == Some(&tag::TRANSFER) {
        let [manager, vault, ring, transfer @ ..] = &*accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        let state = VaultState::load(program_id, vault)?;
        if !manager.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        if manager.address() != &state.manager
            || vault.address() != &owner
            || ring.address() != &state.ring
        {
            return Err(ProgramError::InvalidArgument);
        }
        return RingTransferCpi {
            ring_program: &state.ring,
            owner: &owner,
            accounts: transfer,
            data,
        }
        .invoke_signed(signer);
    }
    let enrollment = PdaEnrollment::parse(accounts, data)?;
    let state = VaultState::load(program_id, enrollment.vault())?;
    enrollment.invoke_signed(
        VaultEnrollmentPolicy {
            manager: &state.manager,
            vault: &owner,
            ring: &state.ring,
            tracking_registry: &state.tracking,
        },
        signer,
    )
}
