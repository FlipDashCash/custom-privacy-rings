//! One owned-graph host regression; no transaction, signer, validator, or proof.
#![forbid(unsafe_code)]

#[path = "../src/instructions/settlement/jupiter_measured_v1_graph_draft.rs"]
mod draft;
#[path = "../../../program-libs/interface/src/proof/jupiter_measured_descriptor_context_draft.rs"]
mod context;
#[path = "../../../program-libs/interface/src/proof/jupiter_measured_descriptor_statement_draft.rs"]
mod descriptor_statement;
#[path = "../../../program-libs/interface/src/instructions/jupiter_measured_deposit_draft.rs"]
mod wire;
#[allow(dead_code, unused_imports)]
#[path = "../../../program-libs/interface/tests/frames/mod.rs"]
mod stock_frames;

use bytemuck::Zeroable;
use context::{ProfilePins, SPL_OUTPUT_MAX};
use custom_ring_interface::{
    storage, tracking::TrackingRegistry, InstructionTag, RingMode, RingPolicy, TransferRoots,
    RING_POLICY, RING_POLICY_PDA_SEED,
};
use draft::{
    compare_measured_prefix, prepare_context, prepare_descriptor, ComparedRoles,
    DescriptorScope, ExpectedGuardAccount, ExpectedLegacySplPrefix, Meta, PrefixProjection,
    PrepareError, PreviousInstruction,
};
use serde_json::Value;
use solana_address::Address;
use spr_hasher::{
    hash_chain::create_hash_chain_4_from_slice,
    primitives::{hash_bytes, pack_be, right_align, solana_owner_identity},
};
use spr_interface::{
    instruction::InterfaceTransfer, pda, SHIELDED_POOL_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID,
};
use wire::{DraftFrame, Role, ROLES, SPP_SPL_DEPOSIT_ROLES};

fn address(value: &Value) -> Address {
    Address::from_str_const(value.as_str().expect("fixture address"))
}

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .expect("fixture byte array")
        .iter()
        .map(|byte| u8::try_from(byte.as_u64().expect("fixture byte")).expect("byte range"))
        .collect()
}

fn writable(key: Address) -> Meta {
    Meta {
        key,
        writable: true,
        signer: false,
    }
}

fn stock_gross_spl_withdrawal() -> Vec<u8> {
    // Exact compact, unsigned codec shape used by the earlier positive fixture.
    // The stock fixture's artificial proof data is never verified or executed.
    let fixture = stock_frames::Fixture::new();
    let mut data = fixture.anonymous_compact();
    data.n_in = 2;
    data.n_out = 2;
    data.nullifiers = &fixture.fields[1..3];
    data.outputs.truncate(1);
    let mut data = fixture.with_audit_stream(data);
    data.interface_transfers = vec![InterfaceTransfer::SplWithdrawal {
        amount: 101,
        spl_interface_bump: 255,
    }];
    let mut encoded = vec![InstructionTag::Transfer as u8];
    encoded.extend_from_slice(&data.encode().expect("owned stock anonymous encoder"));
    encoded
}

#[test]
fn checks_one_stock_operation_then_consumes_its_own_binding_in_the_v1_context() {
    let case: Value =
        serde_json::from_str(include_str!("fixtures/jupiter_measured_v1_guard_case0.json"))
            .expect("archived V44 case 0 projection");
    let router = &case["router"];
    let router_program = address(&router["program"]);
    let guard_program = address(&case["guard"]);
    let keys: Vec<_> = case["keys"].as_array().unwrap().iter().map(address).collect();
    let router_accounts: Vec<_> = router["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let flags = row[1].as_u64().unwrap();
            Meta {
                key: address(&row[0]),
                writable: flags & 1 != 0,
                signer: flags & 2 != 0,
            }
        })
        .collect();
    let metas: Vec<_> = keys
        .iter()
        .map(|key| Meta {
            key: *key,
            writable: router_accounts.iter().any(|m| m.key == *key && m.writable),
            signer: router_accounts.iter().any(|m| m.key == *key && m.signer),
        })
        .collect();
    let expected_guard_accounts: Vec<_> = metas
        .iter()
        .map(|m| ExpectedGuardAccount {
            address: m.key,
            is_writable: m.writable,
            is_signer: m.signer,
        })
        .collect();
    let pre = bytes(&case["pre"]);
    let post = bytes(&case["post"]);
    let route = bytes(&router["data"]);

    let ring = Address::new_from_array([0x21; 32]);
    let mut roles: [Address; ROLES.len()] =
        core::array::from_fn(|i| Address::new_from_array([0xa0 + i as u8; 32]));
    roles[..6].copy_from_slice(&keys[..6]);
    let policy_address = Address::derive_program_address(&[RING_POLICY_PDA_SEED], &ring)
        .expect("stock policy PDA")
        .0;
    let ring_config = Address::new_from_array(pda::ring_auth(&ring.to_bytes().into()).0.to_bytes());
    let spp = Address::new_from_array(SHIELDED_POOL_PROGRAM_ID);
    let token = Address::new_from_array(SPL_TOKEN_PROGRAM_ID);
    roles[Role::RingPolicy as usize] = policy_address;
    roles[Role::RingConfig as usize] = ring_config;
    roles[Role::ShieldedPoolProgram as usize] = spp;
    roles[Role::LegacyTokenProgram as usize] = token;
    roles[Role::ValueGuardProgram as usize] = guard_program;
    roles[Role::JupiterProgram as usize] = router_program;
    roles[SPP_SPL_DEPOSIT_ROLES[7] as usize] =
        Address::new_from_array(pda::spl_interface(&keys[5].to_bytes().into()).to_bytes());
    let mut policy = RingPolicy::zeroed();
    policy.discriminator = RING_POLICY;
    policy.mode = RingMode::Anonymous as u8;
    policy.tracking = roles[Role::TrackingRegistry as usize];
    policy.generation = 7u32.to_le_bytes();
    // The already-public inert auditor word from historical #5's typed test.
    // This is only a tagged fixture, not an installed key or authority.
    policy.auditor_pubkey = [4; 33];

    let review = ExpectedLegacySplPrefix {
        guard_program,
        payer: roles[Role::FeePayer as usize],
        authority: keys[1],
        source: keys[2],
        staging: keys[3],
        input_mint: keys[4],
        output_mint: keys[5],
        treasury_token_account: roles[Role::FeeReceiver as usize],
        gross_input: 101,
        commercial_fee: 1,
        mint_decimals: 6,
        pre_guard_payload: pre.as_slice().try_into().unwrap(),
        guard_accounts: &expected_guard_accounts,
    };
    let scratch = |b| Address::new_from_array([b; 32]);
    let withdrawal_accounts = [
        writable(policy_address),
        writable(policy.tracking),
        Meta {
            key: review.payer,
            writable: true,
            signer: true,
        },
        writable(roles[Role::Tree as usize]),
        writable(spp),
        writable(Address::default()),
        writable(ring_config),
        writable(scratch(6)),
        writable(scratch(7)),
        writable(scratch(8)),
        writable(scratch(11)),
        writable(review.input_mint),
        writable(scratch(13)),
        writable(review.source),
        writable(token),
    ];
    let fee_accounts = [
        writable(review.source),
        Meta {
            key: review.input_mint,
            writable: false,
            signer: false,
        },
        writable(review.treasury_token_account),
        Meta {
            key: review.authority,
            writable: false,
            signer: true,
        },
    ];
    let mut fee_bytes = [0; 10];
    fee_bytes[0] = 12;
    fee_bytes[1..9].copy_from_slice(&review.commercial_fee.to_le_bytes());
    fee_bytes[9] = review.mint_decimals;
    let encoded_withdrawal = stock_gross_spl_withdrawal();
    let projection = || PrefixProjection {
        program: &ring,
        policy_address: &policy_address,
        policy: &policy,
        first: 2,
        review: &review,
        instructions: [
            PreviousInstruction {
                program_id: ring,
                accounts: &withdrawal_accounts,
                data: &encoded_withdrawal,
            },
            PreviousInstruction {
                program_id: token,
                accounts: &fee_accounts,
                data: &fee_bytes,
            },
            PreviousInstruction {
                program_id: guard_program,
                accounts: &metas,
                data: &pre,
            },
            PreviousInstruction {
                program_id: router_program,
                accounts: &router_accounts,
                data: &route,
            },
            PreviousInstruction {
                program_id: guard_program,
                accounts: &metas,
                data: &post,
            },
        ],
    };

    // Same production-source matcher used by the target-only review bridge.
    // These archived structures are not a runtime observation or an approved plan.
    let compared = ComparedRoles::new(&roles, &roles).expect("exact ordered role comparison");
    let matched = compare_measured_prefix(projection(), compared).expect("same archived prefix");
    assert_eq!(matched.minimum_output, 990);
    let mut different = roles;
    different.swap(Role::Tree as usize, Role::OutputVault as usize);
    assert!(matches!(
        ComparedRoles::new(&roles, &different),
        Err(PrepareError::RoleScope)
    ));
    different = roles;
    different[Role::InputMint as usize] = keys[5];
    let compared = ComparedRoles::new(&different, &different).expect("equal but unrelated roles");
    assert!(matches!(
        compare_measured_prefix(projection(), compared),
        Err(PrepareError::RoleScope)
    ));

    let raw_frame = hex::decode(
        include_str!("../../../program-libs/interface/tests/fixtures/jupiter_measured_deposit_draft_v1.hex")
            .trim(),
    )
    .unwrap();
    let frame = DraftFrame::decode(&raw_frame).expect("inert descriptor envelope");
    assert_eq!(frame.encode().as_slice(), raw_frame.as_slice());
    let roots = TransferRoots {
        tracking_root: *frame.tracking_root,
        ..TransferRoots::default()
    };
    // Test placeholders only, never release pins, a verified manifest, or a selected key.
    let profile = ProfilePins {
        review_profile_sha256: [0x2e; 32],
        genesis: [0x29; 32],
        guard_elf_sha256: [0x2a; 32],
        jupiter_elf_sha256: [0x2b; 32],
        descriptor_r1cs_sha256: [0x2c; 32],
        descriptor_vk_sha256: [0x2d; 32],
    };
    let scope = DescriptorScope {
        profile: &profile,
        roles: &roles,
        tree_id: &[0x26; 32],
        accepted_roots: &roots,
    };
    let context = prepare_context(projection(), &scope, &frame).expect("owned graph typed handoff");

    // Independent preimage encoding using stock fixture fields 1 and 0,
    // different from the detached transcript test's placeholder digest words.
    let expected_digest = hex::decode("752bba61a4d42985b5d768c62ca362e15fb926a72960af8b65e5f7c048ec7a3d")
        .unwrap();
    let expected_digest: [u8; 32] = expected_digest.try_into().unwrap();
    assert_eq!(context.operation_context, hash_bytes(&expected_digest).unwrap());
    assert_eq!(context.tracking_revision.0, frame.tracking_revision.to_le_bytes());
    assert_eq!(context.minimum_output, 990);
    assert_eq!(SPL_OUTPUT_MAX, u64::MAX);

    // Pure statement handoff. No live root lookup, bank, proof or verifier.
    let ring_id = hash_bytes(ring.as_array()).unwrap();
    let mut tracking = TrackingRegistry::zeroed();
    tracking.ring_id = ring_id;
    tracking.auditor_pubkey = [4; 33];
    let prepared = prepare_descriptor(projection(), &scope, &frame, &tracking)
        .expect("same stock projection produces the descriptor public signal");

    let scalar = |n: u64| right_align(&n.to_be_bytes());
    let [aud0, aud1] = pack_be(&[4; 33]);
    let [tx0, tx1] = pack_be(frame.tx_viewing_pubkey);
    // Independently lay out the 25 cells in the pinned Go relation's order.
    // Primitive operations use exactly the owned graph's stock SPR types.
    let expected = [
        scalar(0x4352_4444),
        scalar(1),
        ring_id,
        aud0,
        aud1,
        [0x11; 32],
        scalar(7),
        hash_bytes(&expected_digest).unwrap(),
        [0; 32],
        solana_owner_identity(review.authority.as_array()).unwrap(),
        solana_owner_identity(review.payer.as_array()).unwrap(),
        [0x26; 32],
        hash_bytes(keys[5].as_array()).unwrap(),
        [0x22; 32],
        [0; 32],
        scalar(990),
        scalar(u64::MAX),
        scalar(frame.expiry_slot),
        tx0,
        tx1,
        right_align(frame.salt),
        *frame.recipient_tag,
        hash_bytes(frame.recipient_zero_ciphertext).unwrap(),
        hash_bytes(frame.auditor_ciphertext).unwrap(),
        scalar(25),
    ];
    assert_eq!(prepared.statement().elements().unwrap(), expected);
    assert_eq!(
        *prepared.public_input_hash(),
        create_hash_chain_4_from_slice(&expected).unwrap(),
    );
}
