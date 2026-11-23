#[path = "../src/instructions/settlement/jupiter_legacy_spl_output_draft.rs"]
mod draft;

use draft::{LegacySplOutputReview, LegacySplOutputViews, OutputBalances};
use pinocchio::{
    account::{RuntimeAccount, NOT_BORROWED},
    error::ProgramError,
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

fn set_local_amount(view: &mut AccountView, amount: u64) {
    let mut data = view.try_borrow_mut().expect("no outstanding data borrow");
    data[SPL_TOKEN_ACCOUNT_AMOUNT_OFFSET..SPL_TOKEN_ACCOUNT_AMOUNT_END]
        .copy_from_slice(&amount.to_le_bytes());
}

#[test]
fn binds_canonical_output_and_compares_same_handles_without_balance_arguments() {
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

    let observe = |floor| {
        LegacySplOutputReview::observe(&depositor, &mint, &staging, &vault, &token_program, floor)
    };
    for floor in [0, expected.staging_atomic + 1] {
        assert!(matches!(
            observe(floor),
            Err(ProgramError::InvalidAccountData)
        ));
    }
    let review = observe(expected.staging_atomic).expect("inclusive floor");
    let lower_floor = observe(expected.staging_atomic - 1).expect("conserve favorable output");
    let wrong = Err(ProgramError::InvalidAccountData);
    assert_eq!(review.compare_reread(), wrong);

    // Updates only these inert account bytes through the shared borrow flag.
    // Neither a token CPI nor a transfer, proof or runtime is simulated.
    let (mut stage_bytes, mut vault_bytes) = (staging.clone(), vault.clone());
    let full = expected.vault_atomic + expected.staging_atomic;
    set_local_amount(&mut stage_bytes, 0);
    assert_eq!(review.compare_reread(), wrong); // empty staging alone is insufficient
    for balance in [expected.vault_atomic - 1, full - 1, full + 1] {
        set_local_amount(&mut vault_bytes, balance);
        assert_eq!(review.compare_reread(), wrong);
    }
    set_local_amount(&mut vault_bytes, full);
    assert_eq!(review.compare_reread(), Ok(()));
    assert_eq!(lower_floor.compare_reread(), Ok(())); // not capped at the floor
    set_local_amount(&mut stage_bytes, 1);
    assert_eq!(review.compare_reread(), wrong); // exact vault credit leaves residual staging
    set_local_amount(&mut vault_bytes, u64::MAX);
    assert!(matches!(observe(1), Err(ProgramError::ArithmeticOverflow)));

    set_local_amount(&mut stage_bytes, 0);
    set_local_amount(&mut vault_bytes, full);
    stage_bytes.try_borrow_mut().unwrap()[SPL_TOKEN_ACCOUNT_STATE_OFFSET] = 2;
    assert_eq!(review.compare_reread(), wrong); // reread initialized state, too
    stage_bytes.try_borrow_mut().unwrap()[SPL_TOKEN_ACCOUNT_STATE_OFFSET] =
        SPL_TOKEN_ACCOUNT_INITIALIZED;
    assert_eq!(review.compare_reread(), Ok(()));
}
