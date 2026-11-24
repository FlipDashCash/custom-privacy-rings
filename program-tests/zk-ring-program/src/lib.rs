//! A counter on a custom ring, one data note of the program's PDA updated under the program's own proof.

#[cfg(feature = "circuits")]
pub mod circuits;
#[cfg(all(
    feature = "circuits",
    not(feature = "build-circuits"),
    not(xark),
    not(xark_client)
))]
pub mod client;
#[cfg(all(not(feature = "build-circuits"), not(xark), not(xark_client)))]
pub mod instruction;
#[cfg(all(not(feature = "build-circuits"), not(xark), not(xark_client)))]
pub mod verifier;

pub const OWNER_SEED: &[u8] = b"owner";

pub mod tag {
    pub const CREATE: u8 = 0;
    pub const UPDATE: u8 = 1;
}

pinocchio::address::declare_id!("SkXfzjxYXbPiupcFv8SLyLJZMQv1qjazP9eRBcmH2QZ");

#[cfg(all(feature = "bpf-entrypoint", not(feature = "build-circuits")))]
mod entrypoint {
    pinocchio::entrypoint!(crate::instruction::process_instruction);
}
