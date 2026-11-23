//! Private no-entrypoint review input for the unregistered measured Jupiter V1.
//!
//! Only the on-target reader can observe execution depth and Clock. The
//! separate review match checks the actual prefix against retained comparands.
//! Neither step authenticates a deployment profile or roots, verifies a
//! descriptor, observes output balances, or invokes another program.
#![allow(dead_code)] // No processor calls this review input.

#[path = "../../../../../program-libs/interface/src/instructions/jupiter_measured_deposit_draft.rs"]
mod wire;
#[cfg(target_os = "solana")]
#[path = "../settlement/jupiter_measured_v1_graph_draft.rs"]
mod graph;

use pinocchio::{
    account::Ref,
    error::ProgramError,
    sysvars::instructions::{Instructions, IntrospectedInstruction, INSTRUCTIONS_ID},
    AccountView, Address,
};
#[cfg(target_os = "solana")]
use pinocchio::sysvars::{clock::Clock, Sysvar};
#[cfg(target_os = "solana")]
use super::{instruction_runtime::Pda, loader::load_policy};
#[cfg(target_os = "solana")]
use custom_ring_interface::{RingPolicy, RING_POLICY_PDA_SEED};
use wire::{Access, Role, DRAFT_TAG, DRAFT_VERSION, FRAME_LEN, ROLES};

const ROLE_COUNT: usize = ROLES.len();
const PRIOR: u16 = 5; // stock withdrawal, fee, pre-guard, Jupiter, post-guard
const TRANSACTION_LEVEL_STACK_HEIGHT: u64 = 1;
// Sysvar1111111111111111111111111111111111111, the SVM-created account owner.
const SYSVAR_OWNER: Address = Address::new_from_array([
    6, 167, 213, 23, 24, 117, 247, 41, 199, 61, 147, 64, 143, 33, 97, 32, 6, 126, 216, 140,
    118, 224, 140, 40, 127, 193, 148, 96, 0, 0, 0, 0,
]);

/// An immutable read in one top-level Custom invocation, not a settlement.
/// There is deliberately no host constructor or injected stack/clock fallback.
#[cfg(target_os = "solana")]
pub(super) struct ObservedV1<'a> {
    window: BoundWindow<'a>,
    slot: u64,
}

#[cfg(target_os = "solana")]
impl<'a> ObservedV1<'a> {
    /// Pass the unchanged entrypoint program ID, account slice and full data.
    pub(super) fn load(
        program: &Address,
        accounts: &'a [AccountView],
        data: &'a [u8],
    ) -> Result<Self, ProgramError> {
        // SAFETY: The pinned Pinocchio re-export calls the SVM's no-argument
        // sol_get_stack_height syscall; it does not read or write any pointer.
        let stack_height = unsafe { pinocchio::syscalls::sol_get_stack_height() };
        if stack_height != TRANSACTION_LEVEL_STACK_HEIGHT {
            return Err(ProgramError::InvalidInstructionData);
        }
        let window = BoundWindow::load(program, accounts, data)?;
        let slot = Clock::get()?.slot;
        Ok(Self { window, slot })
    }

    pub(super) fn slot(&self) -> u64 {
        self.slot
    }

    pub(super) fn envelope(&self) -> &[u8] {
        self.window.envelope
    }

    pub(super) fn roles(&self) -> &[AccountView; ROLE_COUNT] {
        self.window.accounts
    }

    pub(super) fn prior_five(
        &self,
    ) -> Result<(u16, [IntrospectedInstruction<'_>; PRIOR as usize]), ProgramError> {
        self.window.prior_five()
    }

    /// Compare against a separately retained operation. Comparands are not
    /// approval, executable pins, or a substitute for authenticating the plan.
    fn compare_retained_prefix<'read>(
        &'read self,
        retained: &RetainedOperation<'_>,
    ) -> Result<RuntimePrefixMatch<'read, 'a>, ProgramError> {
        use graph::{ComparedRoles, PrefixProjection, PreviousInstruction};

        let views = self.roles();
        let actual_roles = views.each_ref().map(|view| *view.address());
        let compared = ComparedRoles::new(&actual_roles, retained.ordered_roles)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        let policy_view = &views[Role::RingPolicy as usize];
        let program = &self.window.program;
        Pda {
            program_id: program,
            seeds: &[RING_POLICY_PDA_SEED],
        }
        .bump::<RingPolicy>(policy_view.address())?;
        let policy = load_policy(program, policy_view)?;

        let (first, rows) = self.prior_five()?;
        for (row, (max_metas, max_data)) in rows.iter().zip(graph::WINDOW_BOUNDS) {
            if row.num_account_metas() > max_metas || row.get_instruction_data().len() > max_data {
                return Err(ProgramError::InvalidInstructionData);
            }
        }
        let metas = [
            project_metas(&rows[0])?,
            project_metas(&rows[1])?,
            project_metas(&rows[2])?,
            project_metas(&rows[3])?,
            project_metas(&rows[4])?,
        ];
        let instructions = core::array::from_fn(|i| PreviousInstruction {
            program_id: *rows[i].get_program_id(),
            accounts: &metas[i],
            data: rows[i].get_instruction_data(),
        });
        let withdrawal = graph::compare_measured_prefix(
            PrefixProjection {
                program,
                policy_address: policy_view.address(),
                policy: &policy,
                first,
                review: retained.original,
                instructions,
            },
            compared,
        )
        .map_err(|_| ProgramError::InvalidInstructionData)?;
        Ok(RuntimePrefixMatch {
            _observed: self,
            _withdrawal: withdrawal,
        })
    }
}

/// Plain per-operation comparison data, deliberately distinct from immutable
/// deployment/profile pins. Nothing here is deserialized from tag 68.
#[cfg(target_os = "solana")]
struct RetainedOperation<'a> {
    ordered_roles: &'a [Address; ROLE_COUNT],
    original: &'a graph::ExpectedLegacySplPrefix<'a>,
}

/// Opaque review-stage value tied to the same real observation. There is no
/// promotion to a descriptor, receipt, signer, or settlement API.
#[cfg(target_os = "solana")]
struct RuntimePrefixMatch<'read, 'invocation> {
    _observed: &'read ObservedV1<'invocation>,
    _withdrawal: graph::WithdrawalBinding,
}

#[cfg(target_os = "solana")]
fn project_metas(row: &IntrospectedInstruction<'_>) -> Result<Vec<graph::Meta>, ProgramError> {
    (0..row.num_account_metas())
        .map(|index| {
            let meta = row.get_instruction_account_at(index)?;
            Ok(graph::Meta {
                key: meta.key,
                writable: meta.is_writable(),
                signer: meta.is_signer(),
            })
        })
        .collect()
}

struct BoundWindow<'a> {
    instructions: Instructions<Ref<'a, [u8]>>,
    program: Address,
    accounts: &'a [AccountView; ROLE_COUNT],
    envelope: &'a [u8],
    first: u16,
}

impl<'a> BoundWindow<'a> {
    fn load(
        program: &Address,
        accounts: &'a [AccountView],
        data: &'a [u8],
    ) -> Result<Self, ProgramError> {
        use ProgramError::{IncorrectProgramId, InvalidInstructionData, UnsupportedSysvar};

        let accounts: &[_; ROLE_COUNT] = accounts.try_into().map_err(|_| InvalidInstructionData)?;
        if data.len() != FRAME_LEN || !data.starts_with(&[DRAFT_TAG, DRAFT_VERSION]) {
            return Err(InvalidInstructionData);
        }
        let sysvar = &accounts[Role::Instructions as usize];
        if sysvar.address() != &INSTRUCTIONS_ID
            || !sysvar.owned_by(&SYSVAR_OWNER)
            || sysvar.executable()
            || sysvar.is_writable()
            || sysvar.is_signer()
        {
            return Err(UnsupportedSysvar);
        }

        // Pinocchio bounds indices, but trusts the SVM's internal offsets and
        // row lengths. Check the extent of the six rows before its typed reads.
        let index = {
            let raw = sysvar.try_borrow()?;
            checked_current_index(&raw).ok_or(InvalidInstructionData)?
        };
        let instructions = Instructions::try_from(sysvar)?;
        if index != instructions.load_current_index() {
            return Err(InvalidInstructionData);
        }
        let current = instructions.load_instruction_at(usize::from(index))?;
        if current.get_program_id() != program {
            return Err(IncorrectProgramId);
        }
        if current.get_instruction_data() != data || current.num_account_metas() != ROLE_COUNT {
            return Err(InvalidInstructionData);
        }
        for (i, ((_, access), view)) in ROLES.iter().zip(accounts).enumerate() {
            let privileges = match access {
                Access::ReadOnly => (false, false),
                Access::Writable => (true, false),
                Access::ReadOnlySigner => (false, true),
                Access::WritableSigner => (true, true),
            };
            let meta = current.get_instruction_account_at(i)?;
            if view.address() != &meta.key
                || (view.is_writable(), view.is_signer()) != privileges
                || (meta.is_writable(), meta.is_signer()) != privileges
            {
                return Err(InvalidInstructionData);
            }
        }
        let first = index.checked_sub(PRIOR).ok_or(InvalidInstructionData)?;
        Ok(Self {
            instructions,
            program: *program,
            accounts,
            envelope: data,
            first,
        })
    }

    fn prior_five(
        &self,
    ) -> Result<(u16, [IntrospectedInstruction<'_>; PRIOR as usize]), ProgramError> {
        let first = usize::from(self.first);
        let at = |n| self.instructions.load_instruction_at(first + n);
        Ok((self.first, [at(0)?, at(1)?, at(2)?, at(3)?, at(4)?]))
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?))
}

/// SVM instruction sysvar format: count, offsets, records, current index. A
/// record holds a u16 meta count, 33 bytes per meta, one 32-byte program ID,
/// a u16 data length, and the data. Inspect only the required adjacent window.
fn checked_current_index(bytes: &[u8]) -> Option<u16> {
    const META_LEN: usize = 1 + pinocchio::address::ADDRESS_BYTES;
    let rows = bytes.get(..bytes.len().checked_sub(2)?)?;
    let count = read_u16(rows, 0)?;
    let current = read_u16(bytes, rows.len())?;
    let first = current.checked_sub(PRIOR)?;
    let header_end = 2 + 2 * usize::from(count);
    if current >= count || header_end > rows.len() {
        return None;
    }
    let offset = |index| read_u16(rows, 2 + 2 * usize::from(index)).map(usize::from);
    for index in first..=current {
        let start = offset(index)?;
        let end = if index + 1 < count {
            offset(index + 1)?
        } else {
            rows.len()
        };
        if start < header_end {
            return None;
        }
        let row = rows.get(start..end)?;
        let accounts = usize::from(read_u16(row, 0)?);
        let length_at = 2 + META_LEN * accounts + pinocchio::address::ADDRESS_BYTES;
        if length_at + 2 + usize::from(read_u16(row, length_at)?) != row.len() {
            return None;
        }
    }
    Some(current)
}

#[cfg(all(test, not(target_os = "solana")))]
mod tests {
    use super::*;
    use pinocchio::account::{RuntimeAccount, NOT_BORROWED};

    fn inert_frame() -> Vec<u8> {
        hex::decode(
            include_str!("../../../../../program-libs/interface/tests/fixtures/jupiter_measured_deposit_draft_v1.hex")
                .trim(),
        )
        .expect("shared inert V1 envelope")
    }

    #[repr(C)]
    struct Fixture {
        header: RuntimeAccount,
        data: [u8; 2048],
    }

    impl Fixture {
        fn view(&mut self) -> AccountView {
            assert_eq!(
                core::mem::offset_of!(Self, data),
                core::mem::size_of::<RuntimeAccount>()
            );
            // SAFETY: Fixtures are stable, initialized and unmoved while views
            // are live; the initialized array spans the entire declared length.
            unsafe { AccountView::new_unchecked(core::ptr::from_mut(self).cast()) }
        }
    }

    // In-memory comparison rows only, never a message, transaction, verifier
    // input, chain snapshot or proof of the Solana-only reader.
    fn fixtures(
        data: &[u8],
        ring: &Address,
        current: u16,
        count: u16,
    ) -> [Fixture; ROLE_COUNT] {
        let mut fixtures: [Fixture; ROLE_COUNT] = core::array::from_fn(|i| {
            // Independent expected privilege sets in frozen outer order.
            let signer = u8::from([1, 8].contains(&i));
            let writable = u8::from([2, 3, 8, 9, 10, 14].contains(&i));
            Fixture {
                header: RuntimeAccount {
                    borrow_state: NOT_BORROWED,
                    is_signer: signer,
                    is_writable: writable,
                    executable: 0,
                    padding: [0; 4],
                    address: if i == 0 {
                        INSTRUCTIONS_ID
                    } else {
                        Address::new_from_array([0x40 + i as u8; 32])
                    },
                    owner: if i == 0 { SYSVAR_OWNER } else { Address::default() },
                    lamports: 0,
                    data_len: 0,
                },
                data: [0; 2048],
            }
        });
        // Ordinary serialized Instructions sysvar layout. Indices before and
        // after the window have distinguishable inert programs and data.
        let mut serialized = vec![0; 2 + usize::from(count) * 2];
        serialized[..2].copy_from_slice(&count.to_le_bytes());
        for i in 0..count {
            let start = u16::try_from(serialized.len()).unwrap();
            let offset = 2 + usize::from(i) * 2;
            serialized[offset..offset + 2].copy_from_slice(&start.to_le_bytes());
            if i == current {
                serialized.extend_from_slice(&(ROLE_COUNT as u16).to_le_bytes());
                for row in &fixtures {
                    serialized.push(row.header.is_signer | (row.header.is_writable << 1));
                    serialized.extend_from_slice(row.header.address.as_ref());
                }
                serialized.extend_from_slice(ring.as_ref());
                serialized.extend_from_slice(&(data.len() as u16).to_le_bytes());
                serialized.extend_from_slice(data);
            } else {
                serialized.extend_from_slice(&0u16.to_le_bytes());
                serialized.extend_from_slice(&[i as u8 + 0x20; 32]);
                serialized.extend_from_slice(&1u16.to_le_bytes());
                serialized.push(i as u8);
            }
        }
        serialized.extend_from_slice(&current.to_le_bytes());
        fixtures[0].header.data_len = serialized.len() as u64;
        fixtures[0].data[..serialized.len()].copy_from_slice(&serialized);
        fixtures
    }

    fn check_window(current: u16, count: u16) {
        let bytes = inert_frame();
        let ring = Address::new_from_array([0x11; 32]);
        let mut fixtures = fixtures(&bytes, &ring, current, count);
        let views = fixtures.each_mut().map(Fixture::view);
        let read = BoundWindow::load(&ring, &views, &bytes).expect("same V1 invocation view");
        assert_eq!(read.program, ring);
        assert!(core::ptr::eq(read.accounts, &views));
        assert_eq!(read.envelope, bytes.as_slice());
        let (first, prior) = read.prior_five().unwrap();
        assert_eq!(first, current - 5);
        for (i, row) in prior.iter().enumerate() {
            let absolute = first as u8 + i as u8;
            assert_eq!(row.get_program_id().as_array(), &[absolute + 0x20; 32]);
            assert_eq!(row.get_instruction_data(), &[absolute]);
        }
    }

    #[test]
    fn jupiter_v1_runtime_review_preserves_exact_prefix_with_surrounding_rows() {
        check_window(6, 8);
        assert_eq!(
            TRANSACTION_LEVEL_STACK_HEIGHT,
            solana_instruction::TRANSACTION_LEVEL_STACK_HEIGHT as u64
        );
    }

    #[test]
    fn jupiter_v1_runtime_review_accepts_minimal_window_at_end() {
        check_window(5, 6);
    }

    #[test]
    fn jupiter_v1_runtime_review_requires_same_data_and_ordered_account_views() {
        let bytes = inert_frame();
        let ring = Address::new_from_array([0x11; 32]);
        let mut fixtures = fixtures(&bytes, &ring, 5, 6);
        let mut views = fixtures.each_mut().map(Fixture::view);
        let mut changed = bytes.clone();
        changed[2] ^= 1; // another expiry in the inert envelope
        assert!(matches!(
            BoundWindow::load(&ring, &views, &changed),
            Err(ProgramError::InvalidInstructionData)
        ));
        views.swap(Role::InputMint as usize, Role::OutputMint as usize);
        assert!(matches!(
            BoundWindow::load(&ring, &views, &bytes),
            Err(ProgramError::InvalidInstructionData)
        ));
    }

    #[test]
    fn jupiter_v1_runtime_review_does_not_register_stock_dispatch() {
        let ring = Address::new_from_array([0x11; 32]);
        assert_eq!(
            crate::process_instruction(&ring, &mut [], &inert_frame()),
            Err(custom_ring_interface::CustomRingError::InvalidInstructionData.into())
        );
    }
}
