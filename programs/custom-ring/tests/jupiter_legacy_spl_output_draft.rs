#[path = "../src/instructions/settlement/jupiter_legacy_spl_output_draft.rs"]
mod draft;

// Root imports used by the original checked descriptor-context source.
use custom_ring_interface::{storage, TransferRoots};
#[path = "fixtures/jupiter_measured_v1_graph_case.rs"]
mod original_case;

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
    // The producer takes the original observation, not the now-empty stage.
    assert_eq!(review.observed_spl_entry(), (expected.staging_atomic, bump));
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

#[test]
fn materializes_the_originals_full_observed_units_into_one_stock_spl_body() {
    use spr_interface::{
        event::{encode_encrypted_ring_deposit_output_ref, EncryptedRingDepositOutputRef},
        instruction::{DepositAssetKind, RingDepositIxData, RingDepositIxDataRef},
    };

    original_case::with_archived_graph(|roles, raw, context, program, make| {
        // The roles come from the same archived, unsigned graph as the prepared
        // descriptor. These are ordinary synthetic account views, not a bank.
        let depositor = roles[1];
        let mint = roles[5];
        let token = Address::new_from_array(SPL_TOKEN_PROGRAM_ID);
        let vault_auth = Address::new_from_array(SHIELDED_POOL_CPI_AUTHORITY);
        let (expected_vault, bump) = pda::spl_interface_with_bump(&mint.to_bytes().into());
        assert_eq!(expected_vault.to_bytes(), roles[14].to_bytes());
        let amount = (1_u64 << 53) + 0x123;
        let vault_start = 71;
        let mut others: [Fixture<0>; 17] = core::array::from_fn(|i| {
            Fixture::new(roles[i], Address::default(), matches!(i, 1 | 8),
                matches!(i, 2 | 3 | 8 | 9 | 10 | 14), false, [])
        });
        let mut mint_bytes = [0; SPL_TOKEN_MINT_ACCOUNT_LEN];
        mint_bytes[44] = 6;
        mint_bytes[45] = 1;
        let mut mint_state = Fixture::new(mint, token, false, false, false, mint_bytes);
        let mut stage_state = Fixture::new(roles[3], token, false, true, false,
            classic_token(&mint, &depositor, amount));
        let mut vault_state = Fixture::new(roles[14], token, false, true, false,
            classic_token(&mint, &vault_auth, vault_start));
        let mut token_state = Fixture::new(token,
            Address::new_from_array(BPF_LOADER_UPGRADEABLE_PUBKEY.to_bytes()),
            false, false, true, []);
        let mut views = others.each_mut().map(Fixture::view);
        views[3] = stage_state.view();
        views[5] = mint_state.view();
        views[13] = token_state.view();
        views[14] = vault_state.view();
        let body = make(&views).expect("matched unsigned graph + canonical SPL views");
        let encoded = body.serialize().expect("owned stock wincode serializer");
        assert_eq!(RingDepositIxData::deserialize(&encoded).unwrap(), body);
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(RingDepositIxDataRef::from_bytes(&trailing).is_err());
        let borrowed = RingDepositIxDataRef::from_bytes(&encoded).unwrap();
        assert_eq!(borrowed.assets.as_slice(), &[DepositAssetKind::Spl { spl_interface_bump: bump }]);
        assert_eq!(borrowed.deposits.len(), 1);
        let entry = &borrowed.deposits[0];
        assert_eq!(entry.asset_index, 0);
        assert_eq!(entry.amount, amount); // neither the 990 floor nor a rounded JS number
        assert_eq!(entry.view_tag.as_slice(), &raw[131..163]);
        assert_eq!(entry.owner_utxo_hash.as_slice(), &raw[50..82]);
        assert_eq!(entry.ring_data_hash, &[0; 32]);
        assert_eq!(entry.encrypted.tx_viewing_pk.as_slice(), &raw[82..115]);
        assert_eq!(entry.encrypted.salt.as_slice(), &raw[115..131]);

        let capsule = entry.encrypted.ciphertext;
        assert_eq!(capsule.len(), 210);
        assert_eq!(&capsule[..8], b"CRDYN001");
        assert_eq!(&capsule[8..40], &context);
        assert_eq!(capsule[40], 0);
        assert_eq!(&capsule[146..], &raw[268..332]);
        // Reconstruct the original template from the independent pinned wire
        // layout, not by calling the producer's own transformation a second time.
        let record = &capsule[41..146];
        let delta = u64::from_be_bytes(core::array::from_fn(|i| record[1 + i] ^ raw[164 + i]));
        assert_eq!(delta, amount);
        assert_eq!(record[0], raw[163]);
        assert_eq!(&record[9..], &raw[172..268]);

        let event_body = encode_encrypted_ring_deposit_output_ref(EncryptedRingDepositOutputRef {
            owner_utxo_hash: entry.owner_utxo_hash,
            asset: mint.as_array(),
            amount: entry.amount,
            data_hash: None,
            ring_program_id: program.as_array(),
            ring_data_hash: entry.ring_data_hash,
            encrypted: entry.encrypted,
        });
        // Pinned SPP's actual scheme-8 serializer, without emitting an event.
        // wincode uses a u16 for the record; the event Borsh body uses a u32.
        assert_eq!(encoded.len(), 370);
        assert_eq!(&encoded[158..160], &210u16.to_le_bytes());
        assert_eq!(&encoded[160..], capsule);
        assert_eq!(event_body.len(), 406);
        assert_eq!(event_body[0], 1);
        assert_eq!(&event_body[1..5], &401u32.to_le_bytes());
        assert_eq!(event_body[5], 8);
        assert_eq!(&event_body[6..38], entry.owner_utxo_hash);
        assert_eq!(&event_body[38..70], mint.as_ref());
        assert_eq!(&event_body[70..78], &amount.to_le_bytes());
        assert_eq!(event_body[78], 0); // no app data
        assert_eq!(&event_body[79..111], program.as_ref());
        assert_eq!(&event_body[111..143], &[0; 32]);
        assert_eq!(&event_body[143..176], entry.encrypted.tx_viewing_pk);
        assert_eq!(&event_body[176..192], entry.encrypted.salt);
        assert_eq!(&event_body[192..196], &210u32.to_le_bytes());
        assert_eq!(&event_body[196..], capsule);

        // Swapping two non-output roles must not be accepted just because the
        // same mint, stage and balance still match. Nor can the same key with
        // different access, a sub-floor stage, or an overflowing vault pass.
        views.swap(6, 7);
        assert!(make(&views).is_none());
        views.swap(6, 7);
        let mut wrong_mode = Fixture::new(roles[6], Address::default(), false, true, false, []);
        let original = core::mem::replace(&mut views[6], wrong_mode.view());
        assert!(make(&views).is_none());
        views[6] = original;
        set_local_amount(&mut views[3], 989);
        assert!(make(&views).is_none());
        set_local_amount(&mut views[3], 990);
        assert_eq!(make(&views).expect("inclusive minimum").deposits[0].amount, 990);
        set_local_amount(&mut views[3], amount);
        set_local_amount(&mut views[14], u64::MAX);
        assert!(make(&views).is_none());
    });
}
