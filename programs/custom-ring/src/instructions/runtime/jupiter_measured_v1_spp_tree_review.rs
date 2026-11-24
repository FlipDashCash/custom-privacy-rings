//! Private identity read from the same bound Custom V1 role views.
//!
//! The pinned stock tree loader checks the full UTXO/nullifier layout. This
//! read does not authorize an append, select a verifier, or certify the
//! SPP executable's identity. No data, leaf, custody or account is changed.
#![cfg_attr(not(test), forbid(unsafe_code))]

use pinocchio::{error::ProgramError, AccountView, Address};
use spr_interface::{
    pda, state::discriminator::TREE_ACCOUNT_DISCRIMINATOR, SHIELDED_POOL_PROGRAM_ID,
};
use spr_tree::{TreeAccount, INITIALIZED};

use super::wire::{Role, ROLES};

pub(super) fn read_from_roles(roles: &[AccountView; ROLES.len()]) -> Result<[u8; 32], ProgramError> {
    let tree = &roles[Role::Tree as usize];
    let pool = &roles[Role::ShieldedPoolProgram as usize];
    let program = Address::new_from_array(SHIELDED_POOL_PROGRAM_ID);
    if pool.address() != &program
        || !pool.executable()
        || pool.is_writable()
        || pool.is_signer()
        || !tree.owned_by(&program)
        || !tree.is_writable()
        || tree.is_signer()
        || tree.executable()
        || tree.data_len() != TreeAccount::account_size()
    {
        return Err(ProgramError::InvalidAccountData);
    }

    let key = tree.address().to_bytes();
    let (id, field) = {
        // The stock checked in-place parser requires &mut [u8], even for this
        // read. The clone refers to the very same account and shares its
        // checked borrow flag. Call only the constructor and immutable
        // getters; release this borrow without changing any account bytes.
        let mut same_view = tree.clone();
        let mut bytes = same_view.try_borrow_mut()?;
        if bytes.first() != Some(&TREE_ACCOUNT_DISCRIMINATOR) {
            return Err(ProgramError::InvalidAccountData);
        }
        let stock = TreeAccount::from_bytes(&mut bytes, key)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        // Only the active initialized state qualifies for this V1 read.
        // A paused or unknown state is never promoted to a checked identity.
        if stock.state() != INITIALIZED {
            return Err(ProgramError::InvalidAccountData);
        }
        (stock.tree_id(), stock.tree_id_array())
    };
    if key != pda::tree(id).to_bytes() {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(field)
}

#[cfg(all(test, not(target_os = "solana")))]
mod tests {
    use super::*;
    use pinocchio::account::{RuntimeAccount, NOT_BORROWED};
    use spr_interface::state::{default_tree_fees, nullifier_tree_params};
    use spr_tree::{
        nullifier_tree::constants::NULLIFIER_TREE_ZKP_BATCHES, TreeAccountLayout, UTXO_TREE_HEIGHT,
    };

    type StockLayout = TreeAccountLayout<UTXO_TREE_HEIGHT, NULLIFIER_TREE_ZKP_BATCHES>;
    // Same type as the pinned stock account_size(), which is non-const.
    const SIZE: usize = core::mem::size_of::<StockLayout>();

    // Pure host memory only. Both positive and negative tree bytes start from
    // the stock initializer; nothing here constructs an ObservedV1 or a bank.
    #[repr(C)]
    struct AccountBytes<const N: usize> {
        header: RuntimeAccount,
        bytes: [u8; N],
    }

    impl<const N: usize> AccountBytes<N> {
        fn new(address: Address, owner: Address, writable: bool, executable: bool) -> Self {
            Self {
                header: RuntimeAccount {
                    borrow_state: NOT_BORROWED,
                    is_signer: 0,
                    is_writable: u8::from(writable),
                    executable: u8::from(executable),
                    padding: [0; 4],
                    address,
                    owner,
                    lamports: 0,
                    data_len: N as u64,
                },
                bytes: [0; N],
            }
        }

        fn view(&mut self) -> AccountView {
            assert!(self.header.data_len <= N as u64);
            assert_eq!(
                core::mem::offset_of!(Self, bytes),
                core::mem::size_of::<RuntimeAccount>()
            );
            // SAFETY: Adjacent initialized bytes span the declared length.
            // Each fixture remains in place until its views/borrows are gone.
            unsafe { AccountView::new_unchecked(core::ptr::from_mut(self).cast()) }
        }
    }

    fn spp() -> Address {
        Address::new_from_array(SHIELDED_POOL_PROGRAM_ID)
    }

    fn initialize(id: u16) -> Box<AccountBytes<SIZE>> {
        let address = pda::tree(id).to_bytes();
        let mut image = Box::new(AccountBytes::<SIZE>::new(
            Address::new_from_array(address),
            spp(),
            true,
            false,
        ));
        let stock_bytes = TreeAccount::account_size();
        assert_eq!(image.bytes.len(), stock_bytes);
        assert_eq!(
            image.bytes.as_ptr().align_offset(core::mem::align_of::<StockLayout>()),
            0
        );
        TreeAccount::init(
            &mut image.bytes[..stock_bytes],
            TREE_ACCOUNT_DISCRIMINATOR,
            UTXO_TREE_HEIGHT as u8,
            address,
            id,
            nullifier_tree_params(),
            default_tree_fees(),
        )
        .expect("stock tree initialization in host memory");
        image
    }

    fn sample<const N: usize, V>(
        tree: &mut AccountBytes<N>,
        check: impl FnOnce([AccountView; ROLES.len()]) -> V,
    ) -> V {
        let mut other: [AccountBytes<0>; ROLES.len()] = core::array::from_fn(|i| {
            AccountBytes::new(
                Address::new_from_array([i as u8 + 0x40; 32]),
                Address::default(),
                false,
                false,
            )
        });
        let pool = &mut other[Role::ShieldedPoolProgram as usize];
        pool.header.address = spp();
        pool.header.executable = 1;
        let mut views = other.each_mut().map(AccountBytes::view);
        views[Role::Tree as usize] = tree.view();
        check(views)
    }

    fn from_tree<const N: usize>(
        tree: &mut AccountBytes<N>,
    ) -> Result<[u8; 32], ProgramError> {
        sample(tree, |views| read_from_roles(&views))
    }

    fn layout(image: &mut AccountBytes<SIZE>) -> &mut StockLayout {
        wincode::deserialize_mut(&mut image.bytes).expect("pinned stock layout")
    }

    #[test]
    fn jupiter_v1_spp_tree_review_binds_role_bytes_without_writing() {
        for id in [0u16, 0x1234] {
            let mut tree = initialize(id);
            let before = tree.bytes.to_vec();
            let mut wanted = [0; 32];
            wanted[30..].copy_from_slice(&id.to_be_bytes());
            assert_eq!(from_tree(&mut tree).unwrap(), wanted);
            assert_eq!(before.as_slice(), tree.bytes.as_slice());
            assert_eq!(tree.header.borrow_state, NOT_BORROWED);

            sample(&mut tree, |mut views| {
                // No fallback to another role, even if the valid tree remains
                // present elsewhere in this same ordered account array.
                views.swap(Role::Tree as usize, Role::FeeReceiver as usize);
                assert_eq!(read_from_roles(&views), Err(ProgramError::InvalidAccountData));
                views.swap(Role::Tree as usize, Role::FeeReceiver as usize);
                let view = &views[Role::Tree as usize];
                let read_lock = view.try_borrow().unwrap();
                assert_eq!(read_from_roles(&views), Err(ProgramError::AccountBorrowFailed));
                assert!(view.is_borrowed());
                drop(read_lock);
                assert_eq!(read_from_roles(&views), Ok(wanted));
                assert!(!view.is_borrowed());
            });
        }
    }

    #[test]
    fn jupiter_v1_spp_tree_review_rejects_wrong_roles_flags_and_size() {
        let mut tree = initialize(7);
        tree.header.is_writable = 0;
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        tree.header.is_writable = 1;
        tree.header.is_signer = 1;
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        tree.header.is_signer = 0;
        tree.header.executable = 1;
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        tree.header.executable = 0;
        tree.header.owner = Address::default();
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        tree.header.owner = spp();
        tree.header.data_len = SIZE as u64 - 1;
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        tree.header.data_len = SIZE as u64;
        let mut extra = Box::new(AccountBytes::<{ SIZE + 8 }>::new(
            tree.header.address,
            spp(),
            true,
            false,
        ));
        extra.bytes[..SIZE].copy_from_slice(&tree.bytes);
        assert_eq!(from_tree(&mut extra), Err(ProgramError::InvalidAccountData));

        sample(&mut tree, |mut views| {
            views.swap(Role::ShieldedPoolProgram as usize, Role::ValueGuardProgram as usize);
            assert_eq!(read_from_roles(&views), Err(ProgramError::InvalidAccountData));
            let mut wrong_flags = [
                AccountBytes::<0>::new(spp(), Address::default(), false, false),
                AccountBytes::<0>::new(spp(), Address::default(), true, true),
                AccountBytes::<0>::new(spp(), Address::default(), false, true),
            ];
            wrong_flags[2].header.is_signer = 1;
            for bad_view in wrong_flags.each_mut().map(AccountBytes::view) {
                views[Role::ShieldedPoolProgram as usize] = bad_view;
                assert_eq!(read_from_roles(&views), Err(ProgramError::InvalidAccountData));
            }
        });
    }

    #[test]
    fn jupiter_v1_spp_tree_review_preserves_stock_layout_and_canonical_pda() {
        for mutate in [
            (|tree: &mut StockLayout| tree.utxo.root_history_len = 0) as fn(&mut StockLayout),
            |tree| tree.utxo.root_history_cursor = u16::MAX,
            |tree| tree.nullifier.zkp_batch_size = 0,
        ] {
            let mut tree = initialize(9);
            mutate(layout(&mut tree));
            let key = tree.header.address.to_bytes();
            assert!(TreeAccount::from_bytes(&mut tree.bytes, key).is_err());
            let before = tree.bytes.to_vec();
            assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
            assert_eq!(before.as_slice(), tree.bytes.as_slice());
            assert_eq!(tree.header.borrow_state, NOT_BORROWED);
        }

        let mut tree = initialize(0x1234);
        layout(&mut tree).discriminator = 0;
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        layout(&mut tree).discriminator = TREE_ACCOUNT_DISCRIMINATOR;
        let key = tree.header.address.to_bytes();
        TreeAccount::from_bytes(&mut tree.bytes, key).unwrap().set_paused(true);
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        layout(&mut tree).state = u8::MAX;
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidAccountData));
        layout(&mut tree).state = INITIALIZED;

        // A valid stock layout and correct owner cannot stand in for the same
        // raw tree ID at its canonical SPP PDA. No endian conversion alias.
        tree.header.address = Address::new_from_array(pda::tree(0x3412).to_bytes());
        assert_eq!(from_tree(&mut tree), Err(ProgramError::InvalidSeeds));
        assert_eq!(tree.header.borrow_state, NOT_BORROWED);
    }
}
