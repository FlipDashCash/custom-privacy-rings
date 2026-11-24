//! Private no-entrypoint review input for the unregistered measured Jupiter V1.
//!
//! Only the on-target reader can observe execution depth and Clock. The
//! separate review match checks the actual prefix against retained comparands.
//! The exact descriptor tracking revision must also be served by the policy's
//! pinned registry at that same slot and match the frame. No deployment
//! profile is selected or proof verified here. Read-only output checks bind
//! canonical SPL views to this same observation; they do not attribute credit
//! to Jupiter or perform a deposit. The existing Solana no-entrypoint
//! review profile also prepares the public input and SPP body from a single
//! bound invocation. This path has no verifier, signer, or dispatch entry.
#![allow(dead_code)] // No processor calls this review input.

#[path = "../../../../../program-libs/interface/src/instructions/jupiter_measured_deposit_draft.rs"]
mod wire;
#[cfg(target_os = "solana")]
#[path = "../settlement/jupiter_measured_v1_graph_draft.rs"]
mod graph;
#[cfg(target_os = "solana")]
#[path = "../settlement/jupiter_legacy_spl_output_draft.rs"]
mod output;
#[cfg(any(
    all(test, not(target_os = "solana")),
    all(target_os = "solana", feature = "test-sbf", feature = "no-entrypoint")
))]
#[path = "jupiter_measured_v1_spp_tree_review.rs"]
mod spp_tree;

// This parent module is itself excluded by bpf-entrypoint. Require
// no-entrypoint as well for the additional producer dependency check.
#[cfg(all(target_os = "solana", feature = "test-sbf", feature = "no-entrypoint"))]
#[path = "../../../../../program-libs/interface/src/proof/jupiter_measured_descriptor_context_draft.rs"]
mod context;
#[cfg(all(target_os = "solana", feature = "test-sbf", feature = "no-entrypoint"))]
#[path = "../../../../../program-libs/interface/src/proof/jupiter_measured_descriptor_statement_draft.rs"]
mod descriptor_statement;
#[cfg(all(target_os = "solana", feature = "test-sbf", feature = "no-entrypoint"))]
#[path = "../settlement/jupiter_measured_v1_spp_payload_draft.rs"]
mod spp_payload;
#[cfg(all(target_os = "solana", feature = "test-sbf", feature = "no-entrypoint"))]
use self::graph as draft;

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
use super::loader::{RegistrySlots, RingRegistries};
use custom_ring_interface::{
    storage::Revision, LiveRoots, RingMode, RingPolicy, RootRead, TransferRoots,
};
#[cfg(target_os = "solana")]
use custom_ring_interface::RING_POLICY_PDA_SEED;
use spr_hasher::primitives::is_canonical_bn254_scalar_be;
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

    fn with_original_projection<R>(
        &self,
        retained: &RetainedOperation<'_>,
        prepare: impl FnOnce(graph::PrefixProjection<'_>) -> Result<R, ProgramError>,
    ) -> Result<R, ProgramError> {
        use graph::{PrefixProjection, PreviousInstruction};

        let views = self.roles();
        let policy_view = &views[Role::RingPolicy as usize];
        if policy_view.executable() {
            return Err(ProgramError::InvalidAccountData);
        }
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
        prepare(PrefixProjection {
            program,
            policy_address: policy_view.address(),
            policy: &policy,
            first,
            review: retained.original,
            instructions,
        })
    }

    /// Compare against a separately retained operation. Comparands are not
    /// approval, executable pins, or a substitute for authenticating the plan.
    fn compare_retained_prefix<'read>(
        &'read self,
        retained: &RetainedOperation<'_>,
    ) -> Result<RuntimePrefixMatch<'read, 'a>, ProgramError> {
        let roles = self.roles().each_ref().map(|view| *view.address());
        let compared = graph::ComparedRoles::new(&roles, retained.ordered_roles)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        self.with_original_projection(retained, |projection| {
            let policy = projection.policy;
            let matched = graph::compare_measured_prefix(projection, compared)
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            let tracking = self.read_selected_tracking(policy)?;
            let views = self.roles();
            #[cfg(feature = "no-entrypoint")]
            let output_tree_field = spp_tree::read_from_roles(views)?;
            let output = output::LegacySplOutputReview::observe(
                &views[Role::Depositor as usize],
                &views[Role::OutputMint as usize],
                &views[Role::OutputStaging as usize],
                &views[Role::OutputVault as usize],
                &views[Role::LegacyTokenProgram as usize],
                matched.minimum_output,
            )?;
            Ok(RuntimePrefixMatch {
                _observed: self,
                _withdrawal: matched.withdrawal,
                _tracking: tracking,
                _ring_generation: policy.generation(),
                #[cfg(feature = "no-entrypoint")]
                _output_tree_field: output_tree_field,
                output,
            })
        })
    }

    /// Prepare one unverified deposit from the bound invocation. Roles and
    /// profile still require independent authentication by a future caller.
    #[cfg(feature = "no-entrypoint")]
    fn prepare_retained_deposit<'read>(
        &'read self,
        retained: &RetainedOperation<'_>,
        profile: &context::ProfilePins,
    ) -> Result<UnverifiedMeasuredDeposit<'read>, ProgramError> {
        self.with_original_projection(retained, |projection| {
            let frame = wire::DraftFrame::decode(self.envelope())
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            let views = self.roles();
            let registry = load_pinned_tracking(
                projection.policy,
                &views[Role::TrackingRegistry as usize],
            )?;
            let live = registry.roots(self.slot());
            let selected = self.match_tracking(projection.policy, &live, &frame)?;
            let tree_id = spp_tree::read_from_roles(views)?;
            let scope = graph::DescriptorScope {
                profile,
                roles: retained.ordered_roles,
                tree_id: &tree_id,
                accepted_roots: &selected.roots,
            };
            let body = spp_payload::prepare_original(projection, &scope, &frame, live.tracking, views)
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            Ok(UnverifiedMeasuredDeposit {
                body,
                descriptor_proof_bytes: frame.descriptor_proof_bytes,
            })
        })
    }

    /// The observed frame names a revision; it cannot supply the accepted
    /// root. The sole account given to the canonical loader is this very
    /// invocation's tracking role, not a retained operation comparand.
    #[inline(never)]
    fn read_selected_tracking(&self, policy: &RingPolicy) -> Result<ServedTracking, ProgramError> {
        let view = &self.roles()[Role::TrackingRegistry as usize];
        let registries = load_pinned_tracking(policy, view)?;
        let live = registries.roots(self.slot());
        let frame = wire::DraftFrame::decode(self.envelope())
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        self.match_tracking(policy, &live, &frame)
    }

    fn match_tracking(
        &self,
        policy: &RingPolicy,
        live: &LiveRoots<'_>,
        frame: &wire::DraftFrame<'_>,
    ) -> Result<ServedTracking, ProgramError> {
        let ring_id = spr_hasher::primitives::hash_bytes(self.window.program.as_array())
            .map_err(|_| ProgramError::InvalidAccountData)?;
        if live.tracking.ring_id != ring_id || live.tracking.auditor_pubkey != policy.auditor_pubkey {
            return Err(ProgramError::InvalidAccountData);
        }
        FrameTracking::from_frame(frame).compare(live)
    }
}

/// Plain per-operation comparison data, deliberately distinct from immutable
/// deployment/profile pins. Nothing here is deserialized from tag 68.
#[cfg(target_os = "solana")]
struct RetainedOperation<'a> {
    ordered_roles: &'a [Address; ROLE_COUNT],
    original: &'a graph::ExpectedLegacySplPrefix<'a>,
}

/// One public input, the exact opaque frame proof, and a body retaining the
/// same staged amount and account views. This type grants no settlement authority.
#[cfg(all(target_os = "solana", feature = "no-entrypoint"))]
struct UnverifiedMeasuredDeposit<'a> {
    body: spp_payload::PreparedSppBody<'a>,
    descriptor_proof_bytes: &'a [u8; wire::PROOF_LEN],
}

/// Load the one registry the matched no-policy V1 rail reads. There is no
/// alternate registry list and no fallback to a key or root from the frame.
fn load_pinned_tracking<'a>(
    policy: &RingPolicy,
    view: &'a AccountView,
) -> Result<RingRegistries<'a>, ProgramError> {
    let read = RootRead::transfer(policy, false);
    if view.executable()
        || view.is_signer()
        || view.is_writable()
        || policy.mode()? != RingMode::Anonymous
        || read != RootRead::None
    {
        return Err(ProgramError::InvalidAccountData);
    }
    let mut only = Some(view);
    let registries = RegistrySlots { ring: policy, read }
        .load(|| only.take().ok_or(ProgramError::NotEnoughAccountKeys))?;
    if registries.len != 1 {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(registries)
}

/// The only comparison inputs from the frozen envelope. In particular, the
/// root bytes here are never the root we return.
struct FrameTracking<'a> {
    expiry_slot: u64,
    revision: Revision,
    claimed_root: &'a [u8; 32],
}

struct ServedTracking {
    revision: Revision,
    roots: TransferRoots,
}

impl<'a> FrameTracking<'a> {
    fn from_frame(frame: &wire::DraftFrame<'a>) -> Self {
        Self {
            expiry_slot: frame.expiry_slot,
            revision: frame.tracking_revision.into(),
            claimed_root: frame.tracking_root,
        }
    }

    fn compare(self, live: &LiveRoots<'_>) -> Result<ServedTracking, ProgramError> {
        // Inclusive nonzero descriptor deadline. The stock registry has a
        // separate, exclusive retirement deadline and enforces its own floor.
        if self.expiry_slot == 0 || live.slot > self.expiry_slot {
            return Err(ProgramError::InvalidInstructionData);
        }
        let served = live.tracking(self.revision)?;
        if !is_canonical_bn254_scalar_be(&served) {
            return Err(ProgramError::InvalidAccountData);
        }
        if &served != self.claimed_root {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(ServedTracking {
            revision: self.revision,
            roots: TransferRoots {
                tracking_root: served,
                ..TransferRoots::default()
            },
        })
    }
}

/// Opaque review-stage value tied to the same real observation. Neither a
/// served root nor these structural and account reads authorize settlement.
#[cfg(target_os = "solana")]
struct RuntimePrefixMatch<'read, 'invocation> {
    _observed: &'read ObservedV1<'invocation>,
    _withdrawal: graph::WithdrawalBinding,
    _tracking: ServedTracking,
    _ring_generation: u32,
    // The stock identity from those same bound views, never a host-provided
    // descriptor value. Nothing currently consumes it outside this review.
    #[cfg(feature = "no-entrypoint")]
    _output_tree_field: [u8; 32],
    output: output::LegacySplOutputReview<'read>,
}

#[cfg(target_os = "solana")]
impl RuntimePrefixMatch<'_, '_> {
    /// Accept no replacement account slice or claimed amount. This review
    /// comparison is dormant until an independently authorized deposit exists.
    fn compare_same_output_views(&self) -> Result<(), ProgramError> {
        self.output.compare_reread()
    }
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

#[cfg(all(test, not(target_os = "solana")))]
mod selected_tracking_tests {
    use super::*;
    use custom_ring_interface::{
        storage::Root, tracking::TrackingRegistry, CustomRingError, STORAGE_PROGRAM_ID,
    };
    use pinocchio::account::{RuntimeAccount, NOT_BORROWED};

    // Public root-history arithmetic and ordinary in-memory account layout.
    // No runtime, Clock, profile, proof, witness or transaction is supplied.
    const NOW: [u8; 32] = [1; 32];
    const OLD: [u8; 32] = [2; 32];
    fn history() -> Box<TrackingRegistry> {
        let mut registry: Box<TrackingRegistry> = bytemuck::zeroed_box();
        registry.discriminator = TrackingRegistry::DISCRIMINATOR;
        registry.next_index = 1u64.to_le_bytes();
        registry.roots[0] = Root {
            hash: NOW,
            revision: 8u64.into(),
            expires_at: [0; 8],
        };
        registry.roots[1] = Root {
            hash: OLD,
            revision: 7u64.into(),
            expires_at: 100u64.to_le_bytes(),
        };
        registry
    }

    fn select(
        registry: &TrackingRegistry,
        slot: u64,
        floor: u64,
        expiry_slot: u64,
        revision: u64,
        claimed_root: &[u8; 32],
    ) -> Result<ServedTracking, ProgramError> {
        FrameTracking {
            expiry_slot,
            revision: revision.into(),
            claimed_root,
        }
        .compare(&LiveRoots {
            own: None,
            curator: None,
            tracking: registry,
            tracking_floor: floor.into(),
            slot,
        })
    }

    #[repr(C)]
    struct RegistryBytes {
        header: RuntimeAccount,
        data: TrackingRegistry,
    }

    // Exercise the exact stock address/owner/discriminator loader used on
    // target, not ObservedV1, which has no host constructor or fake syscall.
    fn read_local_registry(
        pin: Address,
        address: Address,
        owner: Address,
        discriminator: u8,
    ) -> Result<[u8; 32], ProgramError> {
        let mut policy: Box<RingPolicy> = bytemuck::zeroed_box();
        policy.mode = RingMode::Anonymous as u8;
        policy.tracking = pin;
        let mut image = Box::new(RegistryBytes {
            header: RuntimeAccount {
                borrow_state: NOT_BORROWED,
                is_signer: 0,
                is_writable: 0,
                executable: 0,
                padding: [0; 4],
                address,
                owner,
                lamports: 0,
                data_len: TrackingRegistry::SIZE as u64,
            },
            data: *history(),
        });
        image.data.discriminator = discriminator;
        assert_eq!(
            core::mem::offset_of!(RegistryBytes, data),
            core::mem::size_of::<RuntimeAccount>()
        );
        // SAFETY: header and the exact-sized initialized POD are adjacent;
        // their box is retained and not mutated or moved throughout this read.
        let ptr = core::ptr::from_mut(image.as_mut()).cast();
        let view = unsafe { AccountView::new_unchecked(ptr) };
        let registries = load_pinned_tracking(&policy, &view)?;
        Ok(registries.roots(99).tracking(7u64.into())?)
    }

    #[test]
    fn jupiter_v1_selected_tracking_uses_stock_pinned_account_loader() {
        // Opaque account labels only, no deployment identity is selected.
        let a = Address::new_from_array([0x21; 32]);
        let b = Address::new_from_array([0x22; 32]);
        let valid = TrackingRegistry::DISCRIMINATOR;
        assert_eq!(read_local_registry(a, a, STORAGE_PROGRAM_ID, valid), Ok(OLD));
        assert_eq!(
            read_local_registry(a, b, STORAGE_PROGRAM_ID, valid),
            Err(CustomRingError::UnboundRegistry.into())
        );
        for (owner, tag) in [(Address::default(), valid), (STORAGE_PROGRAM_ID, 0)] {
            assert_eq!(
                read_local_registry(a, a, owner, tag),
                Err(CustomRingError::InvalidTrackingRegistry.into())
            );
        }
    }

    #[test]
    fn jupiter_v1_selected_tracking_binds_exact_revision_and_hash() {
        let history = history();
        let selected = select(&history, 99, 0, 99, 7, &OLD).expect("still served");
        assert_eq!(selected.revision.value(), 7);
        assert_eq!(selected.roots.tracking_root, OLD);
        assert_eq!(
            (
                selected.roots.policy_hash,
                selected.roots.own_root,
                selected.roots.curator_root,
            ),
            ([0; 32], [0; 32], [0; 32])
        );
        assert_eq!(
            select(&history, 99, 0, 99, 7, &NOW).err(),
            Some(ProgramError::InvalidInstructionData)
        );
        assert_eq!(
            select(&history, 99, 0, 99, 6, &OLD).err(),
            Some(CustomRingError::RootNotServed.into())
        );
    }

    #[test]
    fn jupiter_v1_selected_tracking_keeps_two_distinct_expiries() {
        let history = history();
        assert!(select(&history, 100, 0, 100, 8, &NOW).is_ok());
        assert_eq!(
            select(&history, 100, 0, 100, 7, &OLD).err(),
            Some(CustomRingError::RootNotServed.into())
        );
        assert_eq!(
            select(&history, 101, 0, 100, 8, &NOW).err(),
            Some(ProgramError::InvalidInstructionData)
        );
        assert_eq!(
            select(&history, 0, 0, 0, 8, &NOW).err(),
            Some(ProgramError::InvalidInstructionData)
        );
        assert!(select(&history, u64::MAX, 0, u64::MAX, 8, &NOW).is_ok());
    }

    #[test]
    fn jupiter_v1_selected_tracking_preserves_stock_floor_and_field_encoding() {
        let mut history = history();
        assert_eq!(
            select(&history, 99, 8, 200, 7, &OLD).err(),
            Some(CustomRingError::TrackingRootBelowFloor.into())
        );
        // This is an invalid public field encoding, not a proof or key.
        history.roots[0].hash = [0xff; 32];
        assert_eq!(
            select(&history, 100, 8, 200, 8, &[0xff; 32]).err(),
            Some(ProgramError::InvalidAccountData)
        );
    }
}
