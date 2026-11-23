//! The native mirror of `subjectView.cover` and `assertSenderRules` in `prover/circuits/policy`.

use zolana_ring_policy::{
    ListId, ListSet, MintPolicy, OutputSubjects, PaymentSubjects, RecordLists, Rule, RuleRefusal,
    RuleTable, SourceMap, Subject, TransferSubject,
};

const ALLOW: u64 = ListId::Allow.record_bit();
const BLOCK: u64 = ListId::Block.record_bit();
const APPROVAL: u64 = ListId::Approval.record_bit();
const FROZEN: u64 = ListId::Frozen.record_bit();
const PRIVATE_ALLOW: u64 = ListId::PrivateAllow.record_bit();

const SENDER_ALLOW: usize = 0;
const OWNER_BLOCK: usize = 2;
const OWNER_APPROVAL: usize = 3;

fn own(lists: u64) -> RecordLists {
    RecordLists {
        own: lists,
        ..RecordLists::default()
    }
}

struct Transfer {
    sender: u64,
    owner: u64,
    voucher: Option<u64>,
    approval: bool,
}

impl Transfer {
    fn screen(self) -> Result<(), RuleRefusal> {
        let sources = SourceMap::new(
            &[ListId::Allow, ListId::Block, ListId::Approval].map(|list_id| (list_id, [0; 32])),
        )
        .unwrap();
        let mut table = RuleTable::builder()
            .rule(Rule::require(Subject::Sender, ListId::Allow))
            .rule(Rule::require(Subject::OutputOwner, ListId::Allow))
            .rule(Rule::forbid(Subject::OutputOwner, ListId::Block));
        if self.approval {
            table = table.rule(Rule::require(Subject::OutputOwner, ListId::Approval));
        }
        let table = table.build();
        let voucher = self
            .voucher
            .map(|voucher| table.evaluate_voucher(&sources, own(voucher)))
            .transpose()?;
        let covered = |lists: u64| {
            voucher.map_or_else(|| own(lists), |voucher| own(lists).vouched_by(voucher))
        };
        table.evaluate_senders(&sources, &[covered(self.sender)])?;
        table.evaluate_payments(
            &sources,
            PaymentSubjects {
                ring_inputs: &[],
                outputs: &[OutputSubjects {
                    ring: true,
                    owner: [1; 32],
                    asset: [2; 32],
                    amount: 1,
                    owner_lists: covered(self.owner),
                    asset_lists: RecordLists::default(),
                    mint_policy: MintPolicy::EMPTY,
                }],
            },
        )
    }
}

fn refused(rule: usize, subject: TransferSubject) -> Result<(), RuleRefusal> {
    Err(RuleRefusal::Lists { rule, subject })
}

#[test]
fn an_unlisted_sender_and_owner_pass_only_under_a_listed_voucher() {
    let unlisted = |voucher| Transfer {
        sender: 0,
        owner: 0,
        voucher,
        approval: false,
    };
    assert_eq!(
        unlisted(None).screen(),
        refused(SENDER_ALLOW, TransferSubject::Sender)
    );
    assert_eq!(unlisted(Some(ALLOW)).screen(), Ok(()));
}

#[test]
fn a_voucher_off_the_allow_list_is_refused_by_name() {
    let vouched = |voucher| Transfer {
        sender: ALLOW,
        owner: ALLOW,
        voucher: Some(voucher),
        approval: false,
    };
    assert_eq!(vouched(ALLOW).screen(), Ok(()));
    for voucher in [0, BLOCK, APPROVAL] {
        assert_eq!(vouched(voucher).screen(), Err(RuleRefusal::VoucherUnlisted));
    }
}

#[test]
fn a_blocked_owner_stays_refused_under_a_voucher() {
    let blocked = Transfer {
        sender: 0,
        owner: BLOCK,
        voucher: Some(ALLOW),
        approval: false,
    };
    assert_eq!(
        blocked.screen(),
        refused(OWNER_BLOCK, TransferSubject::Owner(0))
    );
}

#[test]
fn a_voucher_covers_no_list_outside_the_vouched_set() {
    let approved = |owner| Transfer {
        sender: 0,
        owner,
        voucher: Some(ALLOW | APPROVAL),
        approval: true,
    };
    assert_eq!(
        approved(0).screen(),
        refused(OWNER_APPROVAL, TransferSubject::Owner(0))
    );
    assert_eq!(approved(APPROVAL).screen(), Ok(()));
}

#[test]
fn cover_adds_the_vouched_bits_of_each_record_and_no_other() {
    let sources = SourceMap::new(&[(ListId::Allow, [0; 32])]).unwrap();
    let every_list = RecordLists {
        own: u64::MAX,
        curator: u64::MAX,
        salted: u64::MAX,
    };
    let voucher = RuleTable::builder()
        .build()
        .evaluate_voucher(&sources, every_list)
        .unwrap();
    let cover = ALLOW | PRIVATE_ALLOW;
    assert_eq!(ListSet::VOUCHED.record_bits(), cover);
    assert_eq!(
        own(BLOCK).vouched_by(voucher),
        RecordLists {
            own: BLOCK | cover,
            curator: cover,
            salted: cover,
        }
    );
}

#[test]
fn a_listed_voucher_stays_under_every_sender_rule() {
    let sources =
        SourceMap::new(&[ListId::Allow, ListId::Frozen].map(|list_id| (list_id, [0; 32]))).unwrap();
    let table = RuleTable::builder()
        .rule(Rule::require(Subject::Sender, ListId::Allow))
        .rule(Rule::forbid(Subject::Sender, ListId::Frozen))
        .build();
    assert!(table.evaluate_voucher(&sources, own(ALLOW)).is_ok());
    assert_eq!(
        table.evaluate_voucher(&sources, own(ALLOW | FROZEN)),
        Err(RuleRefusal::Lists {
            rule: 1,
            subject: TransferSubject::Voucher,
        })
    );
}

#[test]
fn the_voucher_reads_each_allow_list_from_the_record_its_source_names() {
    let curated = SourceMap::new(&[(ListId::Allow, [7; 32])]).unwrap();
    let private = SourceMap::new(&[(ListId::PrivateAllow, [0; 32])]).unwrap();
    let table = RuleTable::builder().build();
    let curator = RecordLists {
        curator: ALLOW,
        ..RecordLists::default()
    };
    let salted = RecordLists {
        salted: PRIVATE_ALLOW,
        ..RecordLists::default()
    };
    assert!(table.evaluate_voucher(&curated, curator).is_ok());
    assert!(table.evaluate_voucher(&private, salted).is_ok());
    for (sources, misplaced) in [
        (&curated, own(ALLOW)),
        (&curated, salted),
        (&private, own(PRIVATE_ALLOW)),
        (&private, curator),
    ] {
        assert_eq!(
            table.evaluate_voucher(sources, misplaced),
            Err(RuleRefusal::VoucherUnlisted)
        );
    }
}
