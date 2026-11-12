//! A text record grows through the public writer method of the storage. Every expectation is built
//! from the units handed to the method, never read back from the owner.
use super::*;
use surgeist_html::{
    accounting::{HtmlResource, HtmlWork},
    bounded::construction::{
        HtmlConstructionTextCapacity, HtmlConstructionTextGrowth, HtmlTextAdmission,
    },
    construction::{HtmlConstructionCause, HtmlConstructionUsage},
};
use surgeist_ledger::{Change, Charge};

/// A consumer that doubles a full text, records what it is asked and counts the work.
#[derive(Default)]
struct Consumer {
    growths: Vec<HtmlConstructionTextGrowth>,
    recorded: Vec<HtmlConstructionTextCapacity>,
    units: usize,
    copied: u64,
}
impl Charge<HtmlWork, HtmlResource> for Consumer {
    type Stop = ();
    fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), ()> {
        for change in batch {
            if let Change::Work(HtmlWork::CopyByte, amount) = *change {
                self.copied += amount;
            }
        }
        Ok(())
    }
    fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), ()> {
        Ok(())
    }
}
impl HtmlTextAdmission for Consumer {
    fn admit_append(&mut self, growth: HtmlConstructionTextGrowth) -> Result<usize, ()> {
        self.growths.push(growth);
        Ok(growth
            .minimum_capacity()
            .max(growth.before().capacity().saturating_mul(2)))
    }
    fn record_growth(&mut self, reached: HtmlConstructionTextCapacity) -> Result<(), ()> {
        self.recorded.push(reached);
        Ok(())
    }
    fn record_units(&mut self, added: usize) -> Result<(), ()> {
        self.units += added;
        Ok(())
    }
}

fn text_node(units: &[u16]) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(storage(), Make::Text(units.to_vec()))
}
fn units(storage: &HtmlConstructionStorage, node: &HtmlBuilderNode) -> Vec<u16> {
    match storage.node(node).unwrap().kind() {
        ConstructionNodeKind::Text(text) | ConstructionNodeKind::Comment(text) => {
            text.code_units().unwrap().to_vec()
        }
        _ => panic!("a text or comment record"),
    }
}

#[test]
fn appended_units_follow_the_text_in_order_and_count_toward_the_usage() {
    let (mut owner, text) = text_node(&[0x61, 0xd800]);
    let mut consumer = Consumer::default();
    let pieces: [&[u16]; 3] = [&[0xdc00, 0x62], &[], &[0, 0xd83d, 0xde42]];
    let mut expected = vec![0x61u16, 0xd800];
    for piece in pieces {
        let offset = owner.append_text(&text, piece, &mut consumer).unwrap();
        assert_eq!(offset, expected.len());
        expected.extend_from_slice(piece);
        assert_eq!(units(&owner, &text), expected);
        assert_eq!(owner.usage().utf16_units(), expected.len());
    }
    assert_eq!(consumer.units, expected.len() - 2);
    // The empty piece is not an append: it asks nothing of the consumer.
    assert_eq!(consumer.growths.len(), 2);
}

#[test]
fn a_node_that_is_not_text_is_refused_and_the_storage_is_unchanged() {
    let (mut owner, comment) = create(storage(), Make::Comment(vec![0x61, 0x62]));
    let usage: HtmlConstructionUsage = owner.usage();
    let error = owner
        .append_text(&comment, &[0x63], &mut Consumer::default())
        .unwrap_err();
    match error {
        surgeist_html::accounting::HtmlChargedError::Faulted(error) => {
            assert_eq!(error.cause(), HtmlConstructionCause::WrongNodeKind)
        }
        other => panic!("the append's own fault: {other:?}"),
    }
    assert_eq!(owner.usage(), usage);
    assert_eq!(units(&owner, &comment), [0x61, 0x62]);
}

#[test]
fn the_work_of_appending_one_unit_at_a_time_grows_linearly_with_the_units() {
    let copied: Vec<u64> = [1_000, 2_000, 4_000, 8_000]
        .iter()
        .map(|&appends| {
            let (mut owner, text) = text_node(&[]);
            let mut consumer = Consumer::default();
            for _ in 0..appends {
                owner.append_text(&text, &[0x61], &mut consumer).unwrap();
            }
            assert_eq!(units(&owner, &text).len(), appends);
            // The bytes reserved: the old and the new layout of every growth.
            let reserved: usize = consumer
                .growths
                .iter()
                .filter(|growth| growth.grows())
                .map(|growth| {
                    2 * (growth.before().capacity()
                        + growth
                            .minimum_capacity()
                            .max(growth.before().capacity() * 2))
                })
                .sum();
            consumer.copied + reserved as u64
        })
        .collect();
    for pair in copied.windows(2) {
        assert!(pair[1] * 10 <= pair[0] * 22, "{copied:?}");
    }
}
