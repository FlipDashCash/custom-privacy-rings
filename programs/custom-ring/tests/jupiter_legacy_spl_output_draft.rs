#[path = "../src/instructions/settlement/jupiter_legacy_spl_output_draft.rs"]
mod draft;

use draft::{LegacySplOutputViews, OutputBalances};
use pinocchio::{
    account::{RuntimeAccount, NOT_BORROWED},
    AccountView, Address,
};
use spr_interface::{
    pda, BPF_LOADER_UPGRADEABLE_PUBKEY, SHIELDED_POOL_CPI_AUTHORITY, SPL_TOKEN_ACCOUNT_AMOUNT_END,
    SPL_TOKEN_ACCOUNT_AMOUNT_OFFSET, SPL_TOKEN_ACCOUNT_INITIALIZED, SPL_TOKEN_ACCOUNT_LEN,
    SPL_TOKEN_ACCOUNT_STATE_OFFSET, SPL_TOKEN_MINT_ACCOUNT_LEN, SPL_TOKEN_PROGRAM_ID,
};

// Test-only in-memory AccountView backing, not a real program account, message,
// signature, guard observation, proof, or qualifier.
#[repr(C)]
struct Fixture<const N: usize> {
    header: RuntimeAccount,
    data: [u8; N],
}

impl<const N: usize> Fixture<N> {
    fn new(
        address: Address,
        owner: Address,
        signer: bool,
        writable: bool,
        executable: bool,
        data: [u8; N],
    ) -> Self {
        Self {
            header: RuntimeAccount {
                borrow_state: NOT_BORROWED,
                is_signer: u8::from(signer),
                is_writable: u8::from(writable),
                executable: u8::from(executable),
                padding: [0; 4],
                address,
                owner,
                lamports: 0,
                data_len: N as u64,
            },
            data,
        }
    }

    fn view(&mut self) -> AccountView {
        assert_eq!(
            core::mem::offset_of!(Self, data),
            core::mem::size_of::<RuntimeAccount>()
        );
        // SAFETY: repr(C) and the asserted offset place exactly N initialized
        // bytes after the header. Tests keep this fixture alive and unmoved
        // until its view and all data borrows are finished.
        unsafe { AccountView::new_unchecked(core::ptr::from_mut(self).cast()) }
    }
}

fn classic_token(mint: &Address, owner: &Address, amount: u64) -> [u8; SPL_TOKEN_ACCOUNT_LEN] {
    let mut data = [0; SPL_TOKEN_ACCOUNT_LEN];
    data[..32].copy_from_slice(mint.as_ref());
    data[32..64].copy_from_slice(owner.as_ref());
    data[SPL_TOKEN_ACCOUNT_AMOUNT_OFFSET..SPL_TOKEN_ACCOUNT_AMOUNT_END]
        .copy_from_slice(&amount.to_le_bytes());
    data[SPL_TOKEN_ACCOUNT_STATE_OFFSET] = SPL_TOKEN_ACCOUNT_INITIALIZED;
    data
}

#[test]
fn reads_canonical_staging_and_backing_without_a_balance_argument() {
    let depositor = Address::new_from_array([17; 32]);
    let mint = Address::new_from_array([29; 32]);
    let token = Address::new_from_array(SPL_TOKEN_PROGRAM_ID);
    let vault_authority = Address::new_from_array(SHIELDED_POOL_CPI_AUTHORITY);
    let staging_key =
        pda::associated_token_address(&depositor.to_bytes().into(), &mint.to_bytes().into());
    let (vault_key, bump) = pda::spl_interface_with_bump(&mint.to_bytes().into());

    let mut depositor_fixture = Fixture::new(depositor, Address::default(), true, false, false, []);
    let mut mint_data = [0; SPL_TOKEN_MINT_ACCOUNT_LEN];
    mint_data[44] = 6;
    mint_data[45] = 1;
    let mut mint_fixture = Fixture::new(mint, token, false, false, false, mint_data);
    let mut token_fixture = Fixture::new(
        token,
        Address::new_from_array(BPF_LOADER_UPGRADEABLE_PUBKEY.to_bytes()),
        false,
        false,
        true,
        [],
    );
    let expected = OutputBalances {
        staging_atomic: (1_u64 << 53) + 1,
        vault_atomic: (1_u64 << 62) + 7,
    };
    let mut staging_fixture = Fixture::new(
        Address::new_from_array(staging_key.to_bytes()),
        token,
        false,
        true,
        false,
        classic_token(&mint, &depositor, expected.staging_atomic),
    );
    let mut vault_fixture = Fixture::new(
        Address::new_from_array(vault_key.to_bytes()),
        token,
        false,
        true,
        false,
        classic_token(&mint, &vault_authority, expected.vault_atomic),
    );

    let depositor = depositor_fixture.view();
    let mint = mint_fixture.view();
    let staging = staging_fixture.view();
    let vault = vault_fixture.view();
    let token_program = token_fixture.view();
    let (output, observed) =
        LegacySplOutputViews::load(&depositor, &mint, &staging, &vault, &token_program)
            .expect("canonical legacy SPL output roles");
    assert_eq!(output.vault_bump(), bump);
    assert_eq!(observed, expected);
    assert_eq!(output.balances().expect("read retained views"), expected);
}
