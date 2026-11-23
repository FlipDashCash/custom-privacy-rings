//! Unlinked single-graph source handoff for Custom measured Jupiter V1.
//!
//! Host integration test only. Projection checks do not establish runtime
//! order, successful proofs, authorization, or trusted executable images.
//! This module never constructs a transaction, invokes a program or exposes
//! the stock binding. Provenance of the exact lifted helpers is documented.
#![forbid(unsafe_code)]

use custom_ring_interface::{
    cosign::CoSignOperation, CoSignScope, InstructionTag, RingMode, RingPolicy, RingRail,
    RingTransactData, MAX_SPP_CPI_ACCOUNTS, RING_TRANSACT_ACCOUNT_PREFIX_LEN,
    RING_TRANSACT_RING_CONFIG_INDEX,
};
use pinocchio::Address;
use spr_hasher::{Hasher, Sha256};
use spr_interface::{
    instruction::{instruction_data::settlement_accounts, InterfaceTransfer},
    RING_AUTH_PDA_SEED, SHIELDED_POOL_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID,
};

use crate::context::{
    ContextFields, ContextInput, GuardAccount, MatchedGuards, OriginalWithdrawal, ProfilePins,
};
use crate::wire::{DraftFrame, Role, ROLES};
use crate::TransferRoots;

const RING_PREFIX: usize = 2; // policy and pinned tracking, for the checked profile.
const MAX_DATA: usize = 4096; // Bounded for pinned v1 transactions; transport is qualified separately.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BindingError {
    WrongProgram,
    UnsupportedProfile,
    UnsupportedTransfer,
    InvalidAccounts,
    InvalidFrame,
    ZeroNullifier,
    WithdrawalMismatch,
}

/// Exact public settlement expected by an authenticated operation plan.
/// This Rust-only type deliberately has no serialization or shared wire ABI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExpectedWithdrawal {
    /// `None` means native SOL; SPL names both mint and exact token program.
    asset: Option<(Address, Address)>,
    destination: Address,
    amount: u64,
}

/// Private output of the pinned projection, not runtime or proof authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WithdrawalBinding {
    first_nullifier: [u8; 32],
    private_tx_hash: [u8; 32],
    predecessor_index: u16,
    ring_config: Address,
}

impl WithdrawalBinding {
    const fn first_nullifier(&self) -> &[u8; 32] {
        &self.first_nullifier
    }

    const fn private_tx_hash(&self) -> &[u8; 32] {
        &self.private_tx_hash
    }

    const fn predecessor_index(&self) -> u16 {
        self.predecessor_index
    }

    const fn ring_config(&self) -> &Address {
        &self.ring_config
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Meta {
    pub(super) key: Address,
    pub(super) writable: bool,
    pub(super) signer: bool,
}

pub(super) struct PreviousInstruction<'a> {
    pub(super) program_id: Address,
    pub(super) accounts: &'a [Meta],
    pub(super) data: &'a [u8],
}

const GUARD_SIZE: usize = 126;
const MAX_GUARD_METAS: usize = 63;
const MAX_ROUTER_METAS: usize = 96;
const MAX_ROUTER_DATA: usize = 2048;
const FEE_SIZE: usize = 10; // Legacy SPL TransferChecked: tag, LE64 amount, decimals.
const FIRST_GUARD_PHASE: usize = 8;
const ROUTER_DOMAIN: &[u8] = b"pyrol:dynamic-jupiter:router:v1\0";
const ROUTE_TAGS: [[u8; 8]; 4] = [
    [229, 23, 203, 151, 122, 227, 173, 42],
    [187, 100, 250, 204, 49, 196, 175, 20],
    [193, 32, 155, 51, 65, 214, 156, 129],
    [209, 152, 83, 147, 124, 254, 216, 233],
];
// Same program IDs as the stock value guard, not deployment or ELF pins.
// JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4
const ROUTER: Address = Address::new_from_array([
    4, 121, 213, 91, 242, 49, 192, 110, 238, 116, 197, 110, 206, 104, 21, 7,
    253, 177, 178, 222, 163, 244, 142, 81, 2, 177, 205, 162, 86, 188, 19, 143,
]);
// ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL
const ATA: Address = Address::new_from_array([
    140, 151, 37, 143, 78, 36, 137, 241, 187, 61, 16, 41, 20, 142, 13, 131,
    11, 90, 19, 153, 218, 255, 16, 132, 4, 142, 123, 216, 219, 233, 248, 89,
]);
// Sysvar1nstructions1111111111111111111111111
const SYSVAR: Address = Address::new_from_array([
    6, 167, 213, 23, 24, 123, 209, 102, 53, 218, 212, 4, 85, 253, 194, 192,
    193, 36, 198, 143, 33, 86, 117, 165, 219, 186, 203, 95, 8, 0, 0, 0,
]);
// So11111111111111111111111111111111111111112
const WSOL: Address = Address::new_from_array([
    6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53,
    218, 196, 57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1,
]);
const TOKEN: Address = Address::new_from_array(SPL_TOKEN_PROGRAM_ID);

/// Comparands from an independent, retained review of the original. This is
/// ordinary data, not authority. In particular the guard address does not pin
/// its executable bytes. Do not construct it from an API quote or the sysvar
/// being checked. An authorized caller and immutable images still need review.
pub(super) struct ExpectedLegacySplPrefix<'a> {
    pub(super) guard_program: Address,
    pub(super) payer: Address,
    pub(super) authority: Address,
    pub(super) source: Address,
    pub(super) staging: Address,
    pub(super) input_mint: Address,
    pub(super) output_mint: Address,
    pub(super) treasury_token_account: Address,
    pub(super) gross_input: u64,
    pub(super) commercial_fee: u64,
    pub(super) mint_decimals: u8,
    /// Exact stock pre-guard payload including floors, rents, and both hashes.
    /// Its router hash uses compiled, not unsigned, privileges.
    pub(super) pre_guard_payload: [u8; GUARD_SIZE],
    /// The exact ordered guard accounts from the retained compiled original.
    pub(super) guard_accounts: &'a [ExpectedGuardAccount],
}

/// A comparison row, not an execution privilege or checked account handle.
pub(super) struct ExpectedGuardAccount {
    pub(super) address: Address,
    pub(super) is_writable: bool,
    pub(super) is_signer: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum LegacySplPrefixError {
    Runtime(BindingError),
    Review,
    FeeLayout,
    GuardLayout,
    RouterLayout,
}

fn check_predecessor(
    program_id: &Address,
    policy_address: &Address,
    policy: &RingPolicy,
    previous: PreviousInstruction<'_>,
    predecessor_index: u16,
    expected_payer: &Address,
    expected: &[ExpectedWithdrawal],
) -> Result<WithdrawalBinding, BindingError> {
    if previous.program_id != *program_id {
        return Err(BindingError::WrongProgram);
    }
    if previous.data.len() > MAX_DATA
        || previous.data.first() != Some(&(InstructionTag::Transfer as u8))
    {
        return Err(BindingError::UnsupportedTransfer);
    }
    if policy.mode() != Ok(RingMode::Anonymous) || policy.has_policy() || policy.thresholds_active()
    {
        return Err(BindingError::UnsupportedProfile);
    }
    let parsed = RingTransactData::parse(&previous.data[1..], RingRail::Anonymous)
        .map_err(|_| BindingError::InvalidFrame)?;
    let data = parsed.data();
    if data.no_spend
        || data.public_owners
        || data.own_revision.is_some()
        || data.policy.is_some()
        || data.extensions.cache.is_some()
        || data.interface_transfers.is_empty()
        || data.interface_transfers.iter().any(|leg| leg.is_deposit())
        || (CoSignOperation {
            spends_note: true,
            public_legs: CoSignScope::WITHDRAWALS,
            approval: data.approval,
        })
        .demand()
        .requirement(policy)
        .is_some()
    {
        return Err(BindingError::UnsupportedProfile);
    }
    let first_nullifier = *data
        .first_nullifier()
        .map_err(|_| BindingError::InvalidFrame)?;
    if first_nullifier == [0; 32] {
        return Err(BindingError::ZeroNullifier);
    }
    let accounts = previous.accounts;
    if accounts.len() > RING_PREFIX + MAX_SPP_CPI_ACCOUNTS
        || accounts.first().map(|meta| meta.key) != Some(*policy_address)
        || accounts.get(1).map(|meta| meta.key) != Some(policy.tracking)
    {
        return Err(BindingError::InvalidAccounts);
    }
    let spp = accounts
        .get(RING_PREFIX..)
        .ok_or(BindingError::InvalidAccounts)?;
    let (ring_auth, _) = Address::derive_program_address(&[RING_AUTH_PDA_SEED], program_id)
        .ok_or(BindingError::InvalidAccounts)?;
    if spp.len() < RING_TRANSACT_ACCOUNT_PREFIX_LEN
        || spp[0].key != *expected_payer
        || !spp[0].signer
        || !spp[0].writable
        || spp[2].key != Address::new_from_array(SHIELDED_POOL_PROGRAM_ID)
        || spp[3].key != Address::default()
        || spp[RING_TRANSACT_RING_CONFIG_INDEX].key != ring_auth
        || accounts.iter().filter(|meta| meta.key == ring_auth).count() != 1
    {
        return Err(BindingError::InvalidAccounts);
    }
    let groups = settlement_accounts(&data.interface_transfers, data.circuit(), spp)
        .ok_or(BindingError::InvalidAccounts)?;
    if !data
        .owner_signers(spp, groups.len())
        .map_err(|_| BindingError::InvalidAccounts)?
        .is_empty()
    {
        return Err(BindingError::UnsupportedProfile);
    }
    if data.interface_transfers.len() != expected.len() {
        return Err(BindingError::WithdrawalMismatch);
    }
    let mut remaining = groups;
    for (leg, expected) in data.interface_transfers.iter().zip(expected) {
        let (group, rest) = remaining
            .split_at_checked(leg.settlement_account_count())
            .ok_or(BindingError::InvalidAccounts)?;
        remaining = rest;
        let matches = match (*leg, expected.asset) {
            (InterfaceTransfer::SolWithdrawal { amount }, None) => {
                amount == expected.amount
                    && group[1].key == expected.destination
                    && group[1].writable
            }
            (InterfaceTransfer::SplWithdrawal { amount, .. }, Some((mint, token_program))) => {
                amount == expected.amount
                    && group[1].key == mint
                    && group[3].key == expected.destination
                    && group[3].writable
                    && group[4].key == token_program
            }
            _ => false,
        };
        if !matches || expected.amount == 0 {
            return Err(BindingError::WithdrawalMismatch);
        }
    }
    Ok(WithdrawalBinding {
        first_nullifier,
        private_tx_hash: *data.private_tx_hash,
        predecessor_index,
        ring_config: ring_auth,
    })
}

fn check_projection(
    program: &Address,
    policy_address: &Address,
    policy: &RingPolicy,
    first: u16,
    review: &ExpectedLegacySplPrefix<'_>,
    [withdrawal, fee, pre, router, post]: [PreviousInstruction<'_>; 5],
) -> Result<WithdrawalBinding, LegacySplPrefixError> {
    use LegacySplPrefixError::{GuardLayout, Review, RouterLayout, Runtime};

    let net = review
        .gross_input
        .checked_sub(review.commercial_fee)
        .filter(|n| *n != 0)
        .ok_or(Review)?;
    let roles = [
        *program,
        *policy_address,
        SYSVAR,
        ROUTER,
        TOKEN,
        Address::new_from_array(SHIELDED_POOL_PROGRAM_ID),
        review.guard_program,
        review.payer,
        review.authority,
        review.source,
        review.staging,
        review.input_mint,
        review.output_mint,
        review.treasury_token_account,
    ];
    if review.commercial_fee == 0
        || [review.input_mint, review.output_mint].contains(&WSOL)
        || roles
            .iter()
            .enumerate()
            .any(|(i, role)| *role == Address::default() || roles[..i].contains(role))
    {
        return Err(Review);
    }
    for (mint, account) in [
        (review.input_mint, review.source),
        (review.output_mint, review.staging),
    ] {
        let address = Address::derive_program_address(
            &[review.authority.as_ref(), TOKEN.as_ref(), mint.as_ref()],
            &ATA,
        )
        .map(|(address, _)| address);
        if address != Some(account)
        {
            return Err(Review);
        }
    }
    check_fee(&fee, review)?;
    if pre.program_id != review.guard_program
        || post.program_id != review.guard_program
        || pre.data != review.pre_guard_payload.as_slice()
        || post.data.len() != GUARD_SIZE
        || &pre.data[..8] != b"PYJDYN01"
        || pre.data[FIRST_GUARD_PHASE] != 0
        || post.data[FIRST_GUARD_PHASE] != 1
        || pre.data[..FIRST_GUARD_PHASE] != post.data[..FIRST_GUARD_PHASE]
        || pre.data[9..] != post.data[9..]
        || pre.data[9] > 1
        || pre.data[11] != 0
        || pre.data[12..20] != net.to_le_bytes()
        || !(7..=MAX_GUARD_METAS).contains(&pre.accounts.len())
        || !same_guard_accounts(pre.accounts, review.guard_accounts)
        || !same_guard_accounts(post.accounts, review.guard_accounts)
    {
        return Err(GuardLayout);
    }
    // These are only the stock frame's independently checkable invariants.
    // Token owner/rent/state and any shared reserves belong to the actual guard.
    let read_u64 = |offset: usize| {
        u64::from_le_bytes(core::array::from_fn(|n| review.pre_guard_payload[offset + n]))
    };
    let (maximum_input, minimum_output, exact_credit) = (read_u64(20), read_u64(28), read_u64(36));
    if maximum_input < net
        || minimum_output == 0
        || read_u64(44) == 0
        || read_u64(52) == 0
        || (pre.data[9] == 0 && exact_credit != 0)
        || (pre.data[9] == 1 && exact_credit < minimum_output)
        || u16::from_le_bytes([pre.data[124], pre.data[125]]) >= 10_000
    {
        return Err(GuardLayout);
    }
    let fixed = [
        (SYSVAR, false, false),
        (review.authority, false, true),
        (review.source, true, false),
        (review.staging, true, false),
        (review.input_mint, false, false),
        (review.output_mint, false, false),
    ];
    if !same_roles(pre.accounts.get(..fixed.len()).ok_or(GuardLayout)?, &fixed)
        || pre.accounts.iter().enumerate().any(|(i, m)| {
            pre.accounts[..i].iter().any(|n| n.key == m.key)
        })
        || !pre
            .accounts
            .iter()
            .any(|m| m.key == ROUTER && !m.signer && !m.writable)
    {
        return Err(GuardLayout);
    }
    let route_tag = ROUTE_TAGS.get(usize::from(pre.data[10])).ok_or(RouterLayout)?;
    if router.data.get(..8) != Some(route_tag.as_slice())
        || ![review.authority, review.source, review.staging]
            .iter()
            .all(|key| router.accounts.iter().any(|m| m.key == *key))
        || router.accounts.iter().any(|m| {
            (m.signer && m.key != review.authority)
                || !pre
                    .accounts
                    .iter()
                    .any(|a| (a.key, a.writable, a.signer) == (m.key, m.writable, m.signer))
        })
        || pre.data[60..92] != router_hash(&router)?
    {
        return Err(RouterLayout);
    }
    // Keep the original stock value; the structural public entry drops it.
    check_predecessor(
        program,
        policy_address,
        policy,
        withdrawal,
        first,
        &review.payer,
        &[ExpectedWithdrawal {
            asset: Some((review.input_mint, TOKEN)),
            destination: review.source,
            amount: review.gross_input,
        }],
    )
    .map_err(Runtime)
}

fn same_guard_accounts(actual: &[Meta], expected: &[ExpectedGuardAccount]) -> bool {
    actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(seen, kept)| {
            (seen.key, seen.writable, seen.signer) == (kept.address, kept.is_writable, kept.is_signer)
        })
}

fn same_roles(actual: &[Meta], expected: &[(Address, bool, bool)]) -> bool {
    actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(seen, kept)| {
            (seen.key, seen.writable, seen.signer) == *kept
        })
}

fn check_fee(
    fee: &PreviousInstruction<'_>,
    r: &ExpectedLegacySplPrefix<'_>,
) -> Result<(), LegacySplPrefixError> {
    let expected = [
        (r.source, true, false),
        (r.input_mint, false, false),
        (r.treasury_token_account, true, false),
        (r.authority, false, true),
    ];
    if fee.program_id != TOKEN
        || fee.data.len() != FEE_SIZE
        || fee.data[0] != 12
        || fee.data[1..9] != r.commercial_fee.to_le_bytes()
        || fee.data[9] != r.mint_decimals
        || !same_roles(fee.accounts, &expected)
    {
        return Err(LegacySplPrefixError::FeeLayout);
    }
    Ok(())
}

fn router_hash(ix: &PreviousInstruction<'_>) -> Result<[u8; 32], LegacySplPrefixError> {
    use LegacySplPrefixError::RouterLayout;
    if ix.program_id != ROUTER
        || ix.accounts.len() > MAX_ROUTER_METAS
        || ix.data.len() > MAX_ROUTER_DATA
    {
        return Err(RouterLayout);
    }
    let count = u16::try_from(ix.accounts.len()).map_err(|_| RouterLayout)?.to_le_bytes();
    let len = u32::try_from(ix.data.len()).map_err(|_| RouterLayout)?.to_le_bytes();
    let mut metas = Vec::with_capacity(ix.accounts.len() * 33);
    for m in ix.accounts {
        metas.extend_from_slice(m.key.as_ref());
        metas.push(u8::from(m.writable) | (u8::from(m.signer) << 1));
    }
    // Exact stock guard hash domain and order, never a public quote digest.
    Sha256::hashv(&[ROUTER_DOMAIN, ROUTER.as_ref(), &count, &metas, &len, ix.data])
        .map_err(|_| RouterLayout)
}

pub(super) struct PrefixProjection<'a> {
    pub(super) program: &'a Address,
    pub(super) policy_address: &'a Address,
    pub(super) policy: &'a RingPolicy,
    pub(super) first: u16,
    pub(super) review: &'a ExpectedLegacySplPrefix<'a>,
    pub(super) instructions: [PreviousInstruction<'a>; 5],
}

pub(super) struct DescriptorScope<'a> {
    pub(super) profile: &'a ProfilePins,
    pub(super) roles: &'a [Address; ROLES.len()],
    pub(super) tree_id: &'a [u8; 32],
    pub(super) accepted_roots: &'a TransferRoots,
}

#[derive(Debug)]
pub(super) enum PrepareError {
    Prefix,
    RoleScope,
    Descriptor,
}

/// Use exactly one checked projection, then prepare only deterministic fields.
/// The immutable profile and roots still require a future authenticated caller.
pub(super) fn prepare_context(
    projection: PrefixProjection<'_>,
    scope: &DescriptorScope<'_>,
    frame: &DraftFrame<'_>,
) -> Result<ContextFields, PrepareError> {
    let PrefixProjection { program, policy_address, policy, first, review, instructions } =
        projection;
    // Retain the exact actual bytes/metas from this same matched projection.
    // They are borrowed; no reparse or reconstruction of a guard is needed.
    let (pre, post, metas) =
        (instructions[2].data, instructions[4].data, instructions[2].accounts);
    let binding = check_projection(program, policy_address, policy, first, review, instructions)
        .map_err(|_| PrepareError::Prefix)?;

    for (role, expected) in [
        (Role::RingPolicy, policy_address),
        (Role::TrackingRegistry, &policy.tracking),
        (Role::RingConfig, binding.ring_config()),
        (Role::FeePayer, &review.payer),
        (Role::FeeReceiver, &review.treasury_token_account),
        (Role::ValueGuardProgram, &review.guard_program),
        (Role::JupiterProgram, &ROUTER),
        (Role::LegacyTokenProgram, &TOKEN),
        (Role::ShieldedPoolProgram, &Address::new_from_array(SHIELDED_POOL_PROGRAM_ID)),
    ] {
        if &scope.roles[role as usize] != expected {
            return Err(PrepareError::RoleScope);
        }
    }
    if binding.predecessor_index() != first {
        return Err(PrepareError::RoleScope);
    }

    let original = OriginalWithdrawal {
        first_nullifier: *binding.first_nullifier(),
        private_tx_hash: *binding.private_tx_hash(),
        gross_input: review.gross_input,
        commercial_fee: review.commercial_fee,
    };
    let accounts: Vec<_> = metas
        .iter()
        .map(|meta| GuardAccount {
            address: meta.key,
            is_writable: meta.writable,
            is_signer: meta.signer,
        })
        .collect();
    ContextInput {
        profile: scope.profile,
        ring_program: program,
        roles: scope.roles,
        guards: MatchedGuards {
            pre: pre.try_into().map_err(|_| PrepareError::Prefix)?,
            post: post.try_into().map_err(|_| PrepareError::Prefix)?,
            accounts: &accounts,
        },
        original: &original,
        tree_id: scope.tree_id,
        ring_generation: policy.generation(),
    }
    .prepare(frame, scope.accepted_roots)
    .map_err(|_| PrepareError::Descriptor)
}
