//! The native mirror of `subjectView.cover` and `assertSenderRules` in `prover/circuits/policy`.

use zolana_ring_policy::{
    ListId, ListSet, MintPolicy, OutputSubjects, PaymentSubjects, RecordLists, Rule, RuleRefusal,
    RuleTable, SourceMap, Subject, TransferSubject,
};

const ALLOW: u64 = ListId::Allow.record_bit();
const BLOCK: u64 = ListId::Block.record_bit();
const APPROVAL: u64 = ListId::Approval.record_bit();

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
        let voucher = self.voucher.map(own);
        let covered = |lists: u64| match &voucher {
            Some(voucher) => own(lists).vouched_by(voucher),
            None => own(lists),
        };
        if let Some(voucher) = &voucher {
            table.evaluate_voucher(&sources, voucher)?;
        }
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
    let every_list = RecordLists {
        own: u64::MAX,
        curator: u64::MAX,
        salted: u64::MAX,
    };
    let cover = ListSet::VOUCHED.record_bits();
    assert_eq!(
        own(BLOCK).vouched_by(&every_list),
        RecordLists {
            own: BLOCK | cover,
            curator: cover,
            salted: cover,
        }
    );
    assert_eq!(
        ListSet::VOUCHED,
        ListSet::of(&[ListId::Allow, ListId::PrivateAllow])
    );
}
