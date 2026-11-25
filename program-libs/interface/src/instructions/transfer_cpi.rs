//! A program's call into the ring's `Transfer` under its own PDA.
//!
//! The ring takes an owner by its signature alone, a PDA signs only through its program's call.
//! `RingTransferCpi` raises that one PDA to a signer and forwards no tag but `TRANSFER`.
//!
//! Read after `program-libs/interface/src/instructions/transfer_approval.rs`. Read before
//! `examples/enrollment-vault/src/lib.rs`.

use spr_interface::instruction::InterfaceTransfer;

use crate::{tag, CustomRingError, ParsedTransact, RingMode, RingRail, RingTransactData};

/// What a calling program binds its own proof to ahead of the call.
pub struct TransferFrame<'a>(ParsedTransact<'a>);

impl<'a> TransferFrame<'a> {
    /// `data` from the `TRANSFER` tag.
    pub fn parse(data: &'a [u8], mode: RingMode) -> Result<Self, CustomRingError> {
        let frame = data
            .strip_prefix(&[tag::TRANSFER])
            .ok_or(CustomRingError::InvalidInstructionData)?;
        RingTransactData::parse(frame, RingRail::transfer(mode)).map(Self)
    }

    pub fn private_tx_hash(&self) -> &'a [u8; 32] {
        self.0.data().private_tx_hash
    }

    pub fn interface_transfers(&self) -> &[InterfaceTransfer] {
        &self.0.data().interface_transfers
    }
}

#[cfg(feature = "cpi")]
pub use cpi::RingTransferCpi;

#[cfg(feature = "cpi")]
mod cpi {
    use arrayvec::ArrayVec;
    use pinocchio::{
        address::address_eq,
        cpi::{invoke_signed_with_bounds, Signer, MAX_STATIC_CPI_ACCOUNTS},
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    };

    use crate::{tag, CustomRingError};

    #[must_use]
    pub struct RingTransferCpi<'a> {
        /// Fixed by the caller's signer seeds or its own state, never taken from the caller's accounts alone.
        pub ring_program: &'a Address,
        /// The PDA the signer seeds derive.
        pub owner: &'a Address,
        /// The transfer's list in call order.
        pub accounts: &'a [AccountView],
        /// From the `TRANSFER` tag.
        pub data: &'a [u8],
    }

    impl RingTransferCpi<'_> {
        /// The owner PDA signs beside every signer of the outer call.
        #[inline(never)]
        pub fn invoke_signed(self, signer: Signer<'_, '_>) -> ProgramResult {
            if self.data.first() != Some(&tag::TRANSFER) {
                return Err(CustomRingError::InvalidInstructionData.into());
            }
            let mut metas = ArrayVec::<InstructionAccount, MAX_STATIC_CPI_ACCOUNTS>::new();
            for account in self.accounts {
                let signs = account.is_signer() || address_eq(account.address(), self.owner);
                metas
                    .try_push(InstructionAccount::new(
                        account.address(),
                        account.is_writable(),
                        signs,
                    ))
                    .map_err(|_| CustomRingError::TooManyAccounts)?;
            }
            let instruction = InstructionView {
                program_id: self.ring_program,
                accounts: &metas,
                data: self.data,
            };
            invoke(&instruction, self.accounts, signer)
        }
    }

    /// Its own frame, the metas and pinocchio's account array together pass one SBF frame.
    #[inline(never)]
    fn invoke(
        instruction: &InstructionView,
        accounts: &[AccountView],
        signer: Signer<'_, '_>,
    ) -> ProgramResult {
        invoke_signed_with_bounds::<MAX_STATIC_CPI_ACCOUNTS, _>(instruction, accounts, &[signer])
    }
}
