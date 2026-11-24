//! Unregistered runtime account reader for the Custom Jupiter V1 output roles.
//!
//! Only the host integration test and no-entrypoint target review import
//! this file. It can read and compare canonical output account balances; it
//! cannot establish an authorized original or guard, attribute venue output,
//! approve a deposit, or verify a proof.

#![forbid(unsafe_code)]

use pinocchio::{error::ProgramError, AccountView, Address};
use spr_interface::{
    ASSOCIATED_TOKEN_PROGRAM_ID, SHIELDED_POOL_CPI_AUTHORITY, SHIELDED_POOL_PROGRAM_ID,
    SPL_INTERFACE_PDA_SEED, SPL_TOKEN_ACCOUNT_AMOUNT_END, SPL_TOKEN_ACCOUNT_AMOUNT_OFFSET,
    SPL_TOKEN_ACCOUNT_INITIALIZED, SPL_TOKEN_ACCOUNT_LEN, SPL_TOKEN_ACCOUNT_STATE_OFFSET,
    SPL_TOKEN_MINT_ACCOUNT_LEN, SPL_TOKEN_PROGRAM_ID,
};

const LEGACY_TOKEN: Address = Address::new_from_array(SPL_TOKEN_PROGRAM_ID);
const ATA: Address = Address::new_from_array(ASSOCIATED_TOKEN_PROGRAM_ID);
const SPP: Address = Address::new_from_array(SHIELDED_POOL_PROGRAM_ID);
const VAULT_AUTHORITY: Address = Address::new_from_array(SHIELDED_POOL_CPI_AUTHORITY);
const MINT_INITIALIZED_OFFSET: usize = 45;
// The pinned Jupiter legacy-SPL prefix explicitly excludes native wrapping.
const WRAPPED_SOL: Address = Address::from_str_const("So11111111111111111111111111111111111111112");

/// The stage and vault are retained together so both reads can use the same
/// validated account handles. This is not a proof, approved role set or receipt.
pub(super) struct LegacySplOutputViews<'a> {
    staging: &'a AccountView,
    vault: &'a AccountView,
    mint: &'a Address,
    depositor: &'a Address,
    vault_bump: u8,
}

/// Observed absolute token balances, not the settled or measured credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OutputBalances {
    pub(super) staging_atomic: u64,
    pub(super) vault_atomic: u64,
}

impl<'a> LegacySplOutputViews<'a> {
    /// Validate the actual V1 account handles, not balances supplied by a
    /// quote. These checks are not approval of the original or guard.
    pub(super) fn load(
        depositor: &'a AccountView,
        output_mint: &'a AccountView,
        staging: &'a AccountView,
        vault: &'a AccountView,
        token_program: &AccountView,
    ) -> Result<(Self, OutputBalances), ProgramError> {
        let mint = output_mint.address();
        let authority = depositor.address();
        let keys = [
            authority,
            mint,
            staging.address(),
            vault.address(),
            token_program.address(),
        ];
        if keys
            .iter()
            .enumerate()
            .any(|(index, key)| keys[..index].contains(key))
            || !depositor.is_signer()
            || depositor.is_writable()
            || !depositor.owned_by(&Address::default())
            || depositor.executable()
            || depositor.data_len() != 0
            || depositor.lamports() != 0
            || token_program.address() != &LEGACY_TOKEN
            || !token_program.executable()
            || token_program.is_writable()
            || token_program.is_signer()
            || mint == &WRAPPED_SOL
            || !output_mint.owned_by(&LEGACY_TOKEN)
            || output_mint.executable()
            || output_mint.is_writable()
            || output_mint.is_signer()
        {
            return Err(ProgramError::InvalidAccountData);
        }
        {
            let data = output_mint.try_borrow()?;
            // is_initialized in the fixed legacy SPL Mint layout.
            if data.len() != SPL_TOKEN_MINT_ACCOUNT_LEN || data[MINT_INITIALIZED_OFFSET] != 1 {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        let staging_key = Address::derive_program_address(
            &[authority.as_ref(), LEGACY_TOKEN.as_ref(), mint.as_ref()],
            &ATA,
        )
        .ok_or(ProgramError::InvalidSeeds)?
        .0;
        let (vault_key, vault_bump) =
            Address::derive_program_address(&[SPL_INTERFACE_PDA_SEED, mint.as_ref()], &SPP)
                .ok_or(ProgramError::InvalidSeeds)?;
        if staging.address() != &staging_key || vault.address() != &vault_key {
            return Err(ProgramError::InvalidAccountData);
        }

        let output = Self {
            staging,
            vault,
            mint,
            depositor: authority,
            vault_bump,
        };
        // This first read validates SPL ownership and shape. Keep the handles
        // for the second read instead of retaining an account-data borrow.
        let observed = output.balances()?;
        Ok((output, observed))
    }

    /// Derived from the canonical mint rather than read from instruction data.
    pub(super) fn vault_bump(&self) -> u8 {
        self.vault_bump
    }

    pub(super) fn balances(&self) -> Result<OutputBalances, ProgramError> {
        Ok(OutputBalances {
            staging_atomic: read_amount(self.staging, self.mint, self.depositor, true)?,
            vault_atomic: read_amount(self.vault, self.mint, &VAULT_AUTHORITY, false)?,
        })
    }
}

/// Review-only comparison of two reads of the same validated accounts. No
/// account-data borrow survives either read. The expected value is numeric
/// conservation, not an amount attributed to Jupiter or a settlement receipt.
pub(super) struct LegacySplOutputReview<'a> {
    views: LegacySplOutputViews<'a>,
    initial_stage_units: u64,
    expected_vault_after: u64,
}

impl<'a> LegacySplOutputReview<'a> {
    pub(super) fn observe(
        depositor: &'a AccountView,
        output_mint: &'a AccountView,
        staging: &'a AccountView,
        vault: &'a AccountView,
        token_program: &AccountView,
        matched_floor: u64,
    ) -> Result<Self, ProgramError> {
        let (views, observed) =
            LegacySplOutputViews::load(depositor, output_mint, staging, vault, token_program)?;
        if matched_floor == 0 || observed.staging_atomic < matched_floor {
            return Err(ProgramError::InvalidAccountData);
        }
        // Conserve all currently visible staging units, not the lower bound.
        let expected_vault_after = observed
            .vault_atomic
            .checked_add(observed.staging_atomic)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        Ok(Self {
            views,
            initial_stage_units: observed.staging_atomic,
            expected_vault_after,
        })
    }

    /// First-read units and the bump derived from that same validated mint.
    /// Neither is an execution receipt or authority to transfer those units.
    pub(super) fn observed_spl_entry(&self) -> (u64, u8) {
        (self.initial_stage_units, self.views.vault_bump())
    }

    /// A future authorized caller would reread immediately after its stock
    /// deposit, before closing staging. Empty staging plus the exact vault
    /// delta is necessary, not proof of an authenticated empty pre-swap
    /// baseline, execution, receipt or note.
    pub(super) fn compare_reread(&self) -> Result<(), ProgramError> {
        let after = self.views.balances()?;
        if after.staging_atomic != 0 || after.vault_atomic != self.expected_vault_after {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

fn read_amount(
    view: &AccountView,
    mint: &Address,
    authority: &Address,
    staging: bool,
) -> Result<u64, ProgramError> {
    if !view.owned_by(&LEGACY_TOKEN) || view.executable() || !view.is_writable() || view.is_signer() {
        return Err(ProgramError::InvalidAccountData);
    }
    let data = view.try_borrow()?;
    if data.len() != SPL_TOKEN_ACCOUNT_LEN
        || &data[..32] != mint.as_ref()
        || &data[32..64] != authority.as_ref()
        || data[SPL_TOKEN_ACCOUNT_STATE_OFFSET] != SPL_TOKEN_ACCOUNT_INITIALIZED
        || data[109..113] != [0; 4] // Ordinary non-native SPL token account.
        || (staging
            && (data[72..76] != [0; 4] // No delegate.
                || data[121..129] != [0; 8] // No delegated amount.
                || data[129..133] != [0; 4])) // No close authority.
    {
        return Err(ProgramError::InvalidAccountData);
    }
    let amount = data[SPL_TOKEN_ACCOUNT_AMOUNT_OFFSET..SPL_TOKEN_ACCOUNT_AMOUNT_END]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    Ok(u64::from_le_bytes(amount))
}
