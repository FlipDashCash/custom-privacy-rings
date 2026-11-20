//! Keyless Rust parity with independently generated native Go descriptor statements.

use custom_ring_interface::dynamic_descriptor::{
    DescriptorStatement, StatementError, ELEMENT_COUNT,
};
use serde::Deserialize;
use spr_hasher::primitives::BN254_SCALAR_MODULUS_BE;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Fixture>,
}

#[derive(Deserialize)]
struct Fixture {
    name: String,
    input: Input,
    elements: Vec<String>,
    public_input_hash: String,
}

#[derive(Deserialize)]
struct Input {
    ring_id: String,
    auditor_pubkey: String,
    tracking_root: String,
    ring_generation: u32,
    operation_context: String,
    output_index: u8,
    source_authority_identity: String,
    payer_identity: String,
    tree_id: String,
    asset: String,
    owner_utxo_hash: String,
    min_amount: u64,
    max_amount: u64,
    expiry_slot: u64,
    tx_viewing_pubkey: String,
    salt: String,
    recipient_tag: String,
    recipient_zero_ciphertext: String,
    auditor_ciphertext: String,
}

fn array<const N: usize>(value: &str) -> [u8; N] {
    hex::decode(value).unwrap().try_into().unwrap()
}

impl Input {
    fn statement(&self) -> DescriptorStatement {
        DescriptorStatement {
            ring_id: array(&self.ring_id),
            auditor_pubkey: array(&self.auditor_pubkey),
            tracking_root: array(&self.tracking_root),
            ring_generation: self.ring_generation,
            operation_context: array(&self.operation_context),
            output_index: self.output_index,
            source_authority_identity: array(&self.source_authority_identity),
            payer_identity: array(&self.payer_identity),
            tree_id: array(&self.tree_id),
            asset: array(&self.asset),
            owner_utxo_hash: array(&self.owner_utxo_hash),
            min_amount: self.min_amount,
            max_amount: self.max_amount,
            expiry_slot: self.expiry_slot,
            tx_viewing_pubkey: array(&self.tx_viewing_pubkey),
            salt: array(&self.salt),
            recipient_tag: array(&self.recipient_tag),
            recipient_zero_ciphertext: array(&self.recipient_zero_ciphertext),
            auditor_ciphertext: array(&self.auditor_ciphertext),
        }
    }
}

fn corpus() -> Corpus {
    serde_json::from_str(include_str!(
        "../../../test-vectors/settlement/dynamic_descriptor_v1.json"
    ))
    .unwrap()
}

fn historical() -> DescriptorStatement {
    corpus().cases[0].input.statement()
}

fn rejects(statement: &DescriptorStatement) {
    assert_eq!(statement.elements(), Err(StatementError::Encoding));
    assert_eq!(statement.hash(), Err(StatementError::Encoding));
}

#[test]
fn native_rust_matches_three_go_cases_element_for_element() {
    let corpus = corpus();
    assert_eq!(corpus.cases.len(), 3);
    for case in corpus.cases {
        let statement = case.input.statement();
        let elements = statement.elements().unwrap();
        assert_eq!(case.elements.len(), ELEMENT_COUNT, "{}", case.name);
        for (index, wanted) in case.elements.iter().enumerate() {
            assert_eq!(elements[index], array(wanted), "{} cell {index}", case.name);
        }
        assert_eq!(
            statement.hash().unwrap(),
            array(&case.public_input_hash),
            "{}",
            case.name
        );
    }
}

#[test]
fn all_eight_scalar_cells_use_the_exact_bn254_boundary() {
    let p = BN254_SCALAR_MODULUS_BE;
    let mut last = p;
    last[31] -= 1;
    let mut above = p;
    above[31] += 1;
    for (selector, index) in [2, 5, 7, 9, 10, 11, 12, 13].into_iter().enumerate() {
        let mut statement = historical();
        for scalar in [last, p, above, [0xff; 32]] {
            *match selector {
                0 => &mut statement.ring_id,
                1 => &mut statement.tracking_root,
                2 => &mut statement.operation_context,
                3 => &mut statement.source_authority_identity,
                4 => &mut statement.payer_identity,
                5 => &mut statement.tree_id,
                6 => &mut statement.asset,
                7 => &mut statement.owner_utxo_hash,
                _ => unreachable!(),
            } = scalar;
            if scalar == last {
                assert_eq!(statement.elements().unwrap()[index], last);
                assert!(statement.hash().is_ok());
            } else {
                rejects(&statement);
            }
        }
    }
}

#[test]
fn scalar_context_ordinal_tags_and_padding_preserve_legacy_refusals() {
    let base = historical();
    let mut statement = base.clone();
    statement.operation_context = [0; 32];
    rejects(&statement);

    for b in 0..=u8::MAX {
        let mut statement = base.clone();
        statement.output_index = b;
        if b <= 4 {
            let cell = statement.elements().unwrap()[8];
            assert_eq!(cell[..31], [0; 31]);
            assert_eq!(cell[31], b);
        } else {
            rejects(&statement);
        }
        for tx_key in [false, true] {
            let mut statement = base.clone();
            if tx_key {
                statement.tx_viewing_pubkey[0] = b;
            } else {
                statement.auditor_pubkey[0] = b;
            }
            if b == 4 {
                assert!(statement.elements().is_ok());
            } else {
                rejects(&statement);
            }
        }
    }
    for byte in 0..24 {
        let mut statement = base.clone();
        statement.recipient_tag[byte] = 1;
        rejects(&statement);
    }
}

#[test]
fn the_full_unsigned_amount_range_keeps_equal_and_zero_bounds() {
    let base = historical();
    let ranges = [0, 1, u64::from(u32::MAX), (1_u64 << 53) + 1, u64::MAX];
    for min in ranges {
        for max in ranges {
            let mut statement = base.clone();
            statement.min_amount = min;
            statement.max_amount = max;
            if min <= max {
                let cells = statement.elements().unwrap();
                assert_eq!(cells[15][..24], [0; 24]);
                assert_eq!(cells[16][..24], [0; 24]);
                assert_eq!(cells[15][24..], min.to_be_bytes());
                assert_eq!(cells[16][24..], max.to_be_bytes());
            } else {
                rejects(&statement);
            }
        }
    }
}

#[test]
fn each_preimage_field_has_the_exact_effect_on_the_transcript() {
    type Edit = (fn(&mut DescriptorStatement), &'static [usize]);
    let tests: &[Edit] = &[
        (|s| s.ring_id[31] ^= 1, &[2]),
        (
            |s| {
                s.auditor_pubkey[1] ^= 1;
                s.auditor_pubkey[32] ^= 1;
            },
            &[3, 4],
        ),
        (|s| s.tracking_root[31] ^= 1, &[5]),
        (|s| s.ring_generation ^= 1, &[6]),
        (|s| s.operation_context[31] ^= 1, &[7]),
        (|s| s.output_index ^= 1, &[8]),
        (|s| s.source_authority_identity[31] ^= 1, &[9]),
        (|s| s.payer_identity[31] ^= 1, &[10]),
        (|s| s.tree_id[31] ^= 1, &[11]),
        (|s| s.asset[31] ^= 1, &[12]),
        (|s| s.owner_utxo_hash[31] ^= 1, &[13]),
        (|s| s.min_amount ^= 1, &[15]),
        (|s| s.max_amount ^= 1, &[16]),
        (|s| s.expiry_slot ^= 1, &[17]),
        (
            |s| {
                s.tx_viewing_pubkey[1] ^= 1;
                s.tx_viewing_pubkey[32] ^= 1;
            },
            &[18, 19],
        ),
        (|s| s.salt[0] ^= 1, &[20]),
        (|s| s.recipient_tag[31] ^= 1, &[21]),
        (|s| s.recipient_zero_ciphertext[104] ^= 1, &[22]),
        (|s| s.auditor_ciphertext[63] ^= 1, &[23]),
    ];
    let base = historical();
    let before = base.elements().unwrap();
    let expected_hash = base.hash().unwrap();
    for (edit, expected) in tests {
        let mut changed = base.clone();
        edit(&mut changed);
        let after = changed.elements().unwrap();
        let positions: Vec<_> = (0..ELEMENT_COUNT)
            .filter(|i| before[*i] != after[*i])
            .collect();
        assert_eq!(&positions[..], *expected);
        assert_ne!(changed.hash().unwrap(), expected_hash, "{expected:?}");
    }
}
