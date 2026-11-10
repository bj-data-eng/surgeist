//! Exact public microdata ownership, admission and publication contracts.
#![forbid(unsafe_code)]

use surgeist_html::{
    attributes::microdata::{HtmlMicrodataTokenKind, HtmlMicrodataTokenReport},
    bounded::microdata::{
        HtmlMicrodataTokenActionError, HtmlMicrodataTokenActionKind,
        HtmlMicrodataTokenInterpretation, HtmlMicrodataTokenInterpretationError,
        HtmlMicrodataTokenMeter, HtmlMicrodataTokenReservation,
    },
    values::DocumentString,
};

type Owner<'a> = HtmlMicrodataTokenInterpretation<'a>;
/// Finishes the owner in `slot`. A refused finish hands the owner back, so it returns to the slot
/// for the test to keep driving, and success leaves the slot empty.
fn try_finish<'a>(
    slot: &mut Option<Owner<'a>>,
) -> Result<HtmlMicrodataTokenReport, HtmlMicrodataTokenInterpretationError> {
    let owner = slot.take().expect("an owner to finish");
    owner.finish().map_err(|failure| {
        let (owner, cause) = failure.into_parts();
        *slot = Some(owner);
        cause
    })
}
#[derive(Default)]
struct Meter {
    work: Vec<usize>,
    reservations: Vec<(usize, usize, usize)>,
    fail_after: Option<usize>,
    skip_reserve: bool,
    fail_after_reserve: bool,
}
impl HtmlMicrodataTokenMeter for Meter {
    type Error = &'static str;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        if self.fail_after == Some(self.work.len()) {
            return Err("caller stopped");
        }
        self.work.push(n);
        Ok(())
    }
    fn reserve(
        &mut self,
        guard: &mut HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error> {
        if self.skip_reserve {
            return Ok(());
        }
        let len = guard.len();
        guard.try_reserve().unwrap();
        self.reservations
            .push((len, guard.capacity(), guard.record_size()));
        assert!(matches!(guard.try_reserve(), Err(ref e) if e.is_reservation_already_attempted()));
        if self.fail_after_reserve {
            return Err("post reserve cancelled");
        }
        Ok(())
    }
}
fn drive(owner: &mut Owner<'_>, meter: &mut Meter) {
    for _ in 0..10_000 {
        let Some(action) = owner.next_action().unwrap() else {
            return;
        };
        action.advance(meter).unwrap();
    }
    panic!("fixture action bound exhausted");
}
#[test]
fn original_utf16_ranges_first_duplicates_and_itemprop_suppression_are_retained() {
    // Infra ASCII whitespace only; non-ASCII space and isolated surrogates stay exact.
    let raw = DocumentString::from_utf16([32, 97, 9, 65, 10, 97, 12, 120, 46, 58, 0xd800, 0xa0]);
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
    assert!(try_finish(&mut owner).unwrap_err().is_not_ready());
    let mut meter = Meter::default();
    drive(owner.as_mut().unwrap(), &mut meter);
    let report = try_finish(&mut owner).unwrap();
    let actual: Vec<_> = report
        .tokens()
        .iter()
        .map(|t| {
            (
                t.start(),
                t.end(),
                t.duplicate_of(),
                t.contains_dot_or_colon(),
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            (1, 2, None, false),
            (3, 4, None, false),
            (5, 6, Some(0), false),
            (7, 12, None, true)
        ]
    );
    assert_eq!(
        report.processing_tokens().collect::<Vec<_>>(),
        vec![&[97][..], &[65][..], &[120, 46, 58, 0xd800, 0xa0][..]]
    );
    assert!(report.raw().unwrap().shares_storage_with(&raw));
    assert!(!report.local_author_conforming());
    assert!(owner.is_none());
}
#[test]
fn single_token_includes_separate_loop_header_eof_done_and_report_admissions() {
    let raw = DocumentString::from("p");
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
    let mut meter = Meter::default();
    drive(owner.as_mut().unwrap(), &mut meter);
    assert_eq!(meter.work, [1, 1, 4, 1, 2, 1, 1, 1, 2, 1]);
    assert_eq!(meter.reservations.len(), 1);
    assert_eq!(meter.reservations[0].0, 0);
    assert!(meter.reservations[0].1 >= 1);
    assert!(meter.reservations[0].2 > 0);
    assert_eq!(
        try_finish(&mut owner).unwrap().token_units(0),
        Some(&[112][..])
    );
}
#[test]
fn absent_and_empty_are_distinct_and_assembly_is_the_last_required_action() {
    let empty = DocumentString::from("");
    for raw in [None, Some(&empty)] {
        let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemType, raw));
        let mut meter = Meter::default();
        drive(owner.as_mut().unwrap(), &mut meter);
        assert_eq!(
            meter.work,
            if raw.is_some() {
                vec![1, 1, 2, 1]
            } else {
                vec![1]
            }
        );
        assert!(meter.reservations.is_empty());
        let report = try_finish(&mut owner).unwrap();
        assert_eq!(report.raw().is_some(), raw.is_some());
        assert_eq!(report.local_author_conforming(), raw.is_none());
    }
}
#[test]
fn callback_error_at_every_work_checkpoint_stops_without_report_or_repeated_work() {
    let raw = DocumentString::from("x.y x.y x:z");
    let mut complete = Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw));
    let mut count = Meter::default();
    drive(&mut complete, &mut count);
    for stop in 0..count.work.len() {
        let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
        let mut meter = Meter {
            fail_after: Some(stop),
            ..Meter::default()
        };
        loop {
            let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
            if let Err(error) = action.advance(&mut meter) {
                assert_eq!(
                    error,
                    HtmlMicrodataTokenActionError::Admission("caller stopped")
                );
                break;
            }
        }
        assert!(try_finish(&mut owner).unwrap_err().is_stopped());
        assert!(matches!(
            owner.as_mut().unwrap().next_action(),
            Err(ref error) if error.is_stopped()
        ));
        assert_eq!(meter.work.len(), stop);
        assert!(std::ptr::eq(owner.take().unwrap().abort().1.unwrap(), &raw));
    }
}
#[test]
fn unexecuted_reservation_refuses_and_can_be_retried_but_post_reserve_error_stops() {
    let raw = DocumentString::from("p");
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
    let mut meter = Meter {
        skip_reserve: true,
        ..Meter::default()
    };
    loop {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        if action.kind() == HtmlMicrodataTokenActionKind::Reserve {
            assert!(
                matches!(action.advance(&mut meter), Err(HtmlMicrodataTokenActionError::Protocol(ref e)) if e.is_reservation_not_performed())
            );
            break;
        }
        action.advance(&mut meter).unwrap();
    }
    assert!(try_finish(&mut owner).unwrap_err().is_not_ready());
    meter.skip_reserve = false;
    meter.fail_after_reserve = true;
    assert_eq!(
        owner
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .advance(&mut meter),
        Err(HtmlMicrodataTokenActionError::Admission(
            "post reserve cancelled"
        ))
    );
    assert!(meter.reservations[0].1 >= 1);
    assert!(try_finish(&mut owner).unwrap_err().is_stopped());
    assert_eq!(raw.block().holders(), 1);
    assert!(std::ptr::eq(owner.take().unwrap().abort().1.unwrap(), &raw));
}
#[test]
fn explicit_between_action_stop_prevents_publication_even_after_assembly() {
    let raw = DocumentString::from("p");
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemRef, Some(&raw)));
    drive(owner.as_mut().unwrap(), &mut Meter::default());
    owner.as_mut().unwrap().stop();
    assert!(try_finish(&mut owner).unwrap_err().is_stopped());
    assert!(std::ptr::eq(owner.take().unwrap().abort().1.unwrap(), &raw));
}

/// Charges every unit of work and answers every reservation, keeping only totals so that large
/// lists stay cheap to drive.
#[derive(Default)]
struct Totals {
    work: usize,
}
impl HtmlMicrodataTokenMeter for Totals {
    type Error = std::convert::Infallible;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        self.work += n;
        Ok(())
    }
    fn reserve(
        &mut self,
        guard: &mut HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error> {
        guard.try_reserve().unwrap();
        Ok(())
    }
}
fn interpret(kind: HtmlMicrodataTokenKind, text: &DocumentString) -> (usize, Vec<Option<usize>>) {
    let mut owner = Some(Owner::new(kind, Some(text)));
    let mut meter = Totals::default();
    while let Some(action) = owner.as_mut().unwrap().next_action().unwrap() {
        action.advance(&mut meter).unwrap();
    }
    let report = try_finish(&mut owner).unwrap();
    (
        meter.work,
        report.tokens().iter().map(|t| t.duplicate_of()).collect(),
    )
}

/// The charged work is the observable cost of the duplicate check. A check that compares each new
/// token with every earlier one charges one visit per earlier token, so doubling the number of
/// equal-length distinct tokens roughly quadruples the total. A check whose cost follows the
/// token's own length charges the same per token however many precede it: doubling the tokens
/// roughly doubles the total, plus the few extra levels of an index that is `log n` deep. The
/// bound of two and a half sits between the two shapes.
#[test]
fn charged_work_for_distinct_equal_length_tokens_grows_linearly_in_their_count() {
    let list = |count: usize| {
        let words: Vec<String> = (0..count).map(|i| format!("t{i:06}")).collect();
        DocumentString::from(words.join(" ").as_str())
    };
    for kind in [
        HtmlMicrodataTokenKind::ItemType,
        HtmlMicrodataTokenKind::ItemProp,
        HtmlMicrodataTokenKind::ItemRef,
    ] {
        let (small, small_duplicates) = interpret(kind, &list(1_000));
        let (large, large_duplicates) = interpret(kind, &list(2_000));
        assert!(small_duplicates.iter().all(Option::is_none));
        assert!(large_duplicates.iter().all(Option::is_none));
        assert!(
            large * 2 <= small * 5,
            "{kind:?}: doubling 1000 distinct tokens charged {small} then {large} units of work, \
             more than 2.5 times"
        );
    }
}

/// The first earlier identical token is named, whatever the lengths, prefixes and code units
/// involved. The oracle is the definition: the least earlier index with identical code units.
#[test]
fn duplicate_of_names_the_first_identical_earlier_token_for_prefixes_nuls_and_surrogates() {
    // A tiny alphabet and short lengths force many duplicates and prefix relations; U+0000 and an
    // isolated surrogate are ordinary token units.
    let alphabet: [u16; 5] = [0x0000, 0x0061, 0x0062, 0xd800, 0xffff];
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for round in 0..200 {
        let mut units: Vec<u16> = Vec::new();
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for _ in 0..(next() % 40 + 1) {
            if !units.is_empty() {
                units.push(if next() % 2 == 0 { 0x20 } else { 0x0a });
            }
            let start = units.len();
            for _ in 0..(next() % 4 + 1) {
                units.push(alphabet[(next() % alphabet.len() as u64) as usize]);
            }
            spans.push((start, units.len()));
        }
        let text = DocumentString::from_utf16(units.clone());
        let (_, actual) = interpret(HtmlMicrodataTokenKind::ItemProp, &text);
        let expected: Vec<Option<usize>> = (0..spans.len())
            .map(|i| {
                (0..i).find(|&j| units[spans[j].0..spans[j].1] == units[spans[i].0..spans[i].1])
            })
            .collect();
        assert_eq!(actual, expected, "round {round}: {units:?}");
    }
}

/// The cost of one token follows that token's own length, whatever the earlier tokens look like.
/// A prefix-sharing chain `b`, `ab`, `aab`, ... makes the deepest possible index for its size, and
/// short probes that follow the chain are the worst case for a search that skips shared bits
/// without checking them. Per token the search compares at most `L + 1` code units and descends
/// at most 17 levels per unit, three work units each; scanning charges at most 5 per unit and
/// 24 per token for the loop, punctuation, reservation and commit steps. Every token therefore
/// costs at most `3 * 18 * (L + 1) + 5 * L + 24 <= 80 * (L + 1)`.
#[test]
fn charged_work_stays_within_a_constant_of_total_length_for_deep_prefix_chains() {
    let mut words: Vec<String> = (0..400).map(|i| format!("{}b", "a".repeat(i))).collect();
    // Probes of length 2 that agree with the chain's long `a` runs where they can.
    words.extend((0..2_000).map(|i| format!("a{}", char::from_u32(0x100 + i).unwrap())));
    let text = DocumentString::from(words.join(" ").as_str());
    let weight: usize = words.iter().map(|w| w.encode_utf16().count() + 1).sum();
    let (work, duplicates) = interpret(HtmlMicrodataTokenKind::ItemRef, &text);
    assert!(duplicates.iter().all(Option::is_none));
    assert!(
        work <= 80 * weight,
        "charged {work} units for {weight} units of length and tokens"
    );
}

/// Answers every reservation and counts the reservations that changed the token vector's
/// capacity. A capacity change is exactly one reallocation request, because a vector that already
/// has room is not asked to grow.
#[derive(Default)]
struct CapacityChanges {
    capacity: usize,
    changes: usize,
}
impl HtmlMicrodataTokenMeter for CapacityChanges {
    type Error = std::convert::Infallible;
    fn work(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn reserve(
        &mut self,
        guard: &mut HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error> {
        guard.try_reserve().unwrap();
        if guard.capacity() != self.capacity {
            self.capacity = guard.capacity();
            self.changes += 1;
        }
        Ok(())
    }
}

/// Appending n tokens one at a time requests a logarithmic number of reallocations in n. The
/// oracle is the doubling bound: from any positive capacity, `ceil(log2 n)` doublings reach `n`,
/// so `floor(log2 n) + 2` changes allow the first allocation and one more for rounding.
#[test]
fn reallocation_requests_for_n_tokens_grow_at_most_logarithmically() {
    for count in [64_usize, 1_024, 8_192] {
        let words: Vec<String> = (0..count).map(|i| format!("t{i:06}")).collect();
        let text = DocumentString::from(words.join(" ").as_str());
        let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemRef, Some(&text)));
        let mut meter = CapacityChanges::default();
        while let Some(action) = owner.as_mut().unwrap().next_action().unwrap() {
            action.advance(&mut meter).unwrap();
        }
        assert_eq!(try_finish(&mut owner).unwrap().tokens().len(), count);
        let allowed = count.ilog2() as usize + 2;
        assert!(
            meter.changes <= allowed,
            "{count} tokens changed the capacity {} times, more than {allowed}",
            meter.changes
        );
    }
}
