//! Retained HTML877d local rel interpretation and checked refinement.
#![forbid(unsafe_code)]
use surgeist_html::{
    attributes::HtmlLinkDestination,
    bounded::rel::{
        HtmlRelActionError, HtmlRelInputs, HtmlRelInterpretation, HtmlRelInterpretationError,
        HtmlRelMeter, HtmlRelRefinement, HtmlRelReservation,
    },
    elements::link::{
        HtmlRelAlternate, HtmlRelEffect, HtmlRelKeyword, HtmlRelLinks, HtmlRelRole,
        HtmlRelTitleAuthor,
    },
    values::DocumentString,
};
#[derive(Default)]
struct Meter {
    work: Vec<usize>,
    fail_at: Option<usize>,
    reservations: Vec<(usize, usize, usize)>,
    fail_reserve: bool,
}
impl HtmlRelMeter for Meter {
    type Error = &'static str;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        if self.fail_at == Some(self.work.len()) {
            return Err("original stop");
        }
        self.work.push(n);
        Ok(())
    }
    fn reserve(&mut self, guard: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error> {
        let len = guard.len();
        guard.try_reserve().unwrap();
        self.reservations
            .push((len, guard.capacity(), guard.record_size()));
        if self.fail_reserve {
            return Err("original reserve stop");
        }
        Ok(())
    }
}
fn interpreted<'a>(inputs: HtmlRelInputs<'a>) -> HtmlRelInterpretation<'a> {
    let mut owner = HtmlRelInterpretation::new(inputs);
    owner.interpret(&mut Meter::default()).unwrap();
    owner
}
/// A refinement and the predicate that recognizes its expected refusal.
type RefusalCase = (
    usize,
    HtmlRelRefinement,
    fn(&HtmlRelInterpretationError) -> bool,
);

#[test]
fn initial_only_finish_preserves_original_ranges_aliases_and_alternate_stylesheet_rules() {
    let rel = DocumentString::from("ALTERNATE stylesheet alternate");
    let href = DocumentString::from("sheet.css");
    let title = DocumentString::from("Night");
    let owner = interpreted(
        HtmlRelInputs::new(Some(HtmlRelRole::Link))
            .with_rel(Some(&rel))
            .with_href(Some(&href))
            .with_title(Some(&title)),
    );
    assert_eq!(owner.occurrences().unwrap()[2].duplicate_of(), Some(0));
    let report = owner.finish().unwrap();
    assert_eq!(report.alternate(), HtmlRelAlternate::StylesheetModifier);
    assert_eq!(
        report.alternate_stylesheet_title_author(),
        HtmlRelTitleAuthor::Satisfied
    );
    assert_eq!(report.links(), HtmlRelLinks::Declared);
    assert_eq!(
        report.occurrence_units(0),
        Some(&[65, 76, 84, 69, 82, 78, 65, 84, 69][..])
    );
    assert!(report.rel().unwrap().shares_storage_with(&rel));
}
#[test]
fn checked_refinement_rejects_wrong_source_spelling_order_and_extension_effect_without_admission() {
    let raw = DocumentString::from("vendor vendor2 alternate");
    let rev = DocumentString::from("made");
    let mut owner = interpreted(
        HtmlRelInputs::new(Some(HtmlRelRole::Link))
            .with_rel(Some(&raw))
            .with_rev(Some(&rev)),
    );
    owner.begin_refinement().unwrap();
    let mut meter = Meter::default();
    let cases: [RefusalCase; 6] = [
        (
            9,
            HtmlRelRefinement::NoEffect,
            HtmlRelInterpretationError::is_unknown_occurrence,
        ),
        (
            3,
            HtmlRelRefinement::NoEffect,
            HtmlRelInterpretationError::is_wrong_source,
        ),
        (
            2,
            HtmlRelRefinement::NoEffect,
            HtmlRelInterpretationError::is_wrong_spelling,
        ),
        (
            0,
            HtmlRelRefinement::Extension(HtmlRelEffect::InternalResource),
            HtmlRelInterpretationError::is_invalid_extension_effect,
        ),
        (
            0,
            HtmlRelRefinement::Extension(HtmlRelEffect::StylesheetModifier),
            HtmlRelInterpretationError::is_invalid_extension_effect,
        ),
        (
            0,
            HtmlRelRefinement::Extension(HtmlRelEffect::Annotation),
            HtmlRelInterpretationError::is_invalid_extension_effect,
        ),
    ];
    for (index, value, expected) in cases {
        assert!(matches!(
            owner.refine(index, value, &mut meter),
            Err(HtmlRelActionError::Protocol(ref error)) if expected(error)
        ));
    }
    assert!(meter.work.is_empty());
    owner
        .refine(
            1,
            HtmlRelRefinement::Standard(HtmlRelKeyword::Stylesheet),
            &mut meter,
        )
        .unwrap();
    assert_eq!(meter.work, [4]);
    assert!(
        matches!(owner.refine(1, HtmlRelRefinement::NoEffect, &mut meter), Err(HtmlRelActionError::Protocol(ref e)) if e.is_repeated_refinement())
    );
    assert!(
        matches!(owner.refine(0, HtmlRelRefinement::NoEffect, &mut meter), Err(HtmlRelActionError::Protocol(ref e)) if e.is_out_of_order_refinement())
    );
    // A refinement that was not refreshed is refused as not ready and handed back.
    let refusal = owner.finish().unwrap_err();
    assert!(refusal.cause().is_not_ready());
    let (mut owner, _) = refusal.into_parts();
    owner.refresh(&mut meter).unwrap();
    let report = owner.finish().unwrap();
    assert_eq!(report.alternate(), HtmlRelAlternate::StylesheetModifier);
    assert!(report.occurrences()[1].registry_resolved());
    assert_eq!(
        report.occurrences()[1].effect(),
        Some(HtmlRelEffect::ExternalResource)
    );
}
#[test]
fn refresh_uses_original_type_title_and_image_preload_operands() {
    let rel = DocumentString::from("vendor alternate");
    let href = DocumentString::from("u");
    let hint = DocumentString::from("application/rss+xml");
    let images = DocumentString::from("x 1x");
    let mut owner = interpreted(
        HtmlRelInputs::new(Some(HtmlRelRole::Link))
            .with_rel(Some(&rel))
            .with_href(Some(&href))
            .with_type_hint(Some(&hint))
            .with_imagesrcset(Some(&images))
            .with_as_destination(Some(HtmlLinkDestination::Image)),
    );
    assert!(std::ptr::eq(
        owner.inputs().unwrap().type_hint().unwrap(),
        &hint
    ));
    owner.begin_refinement().unwrap();
    owner
        .refine(
            0,
            HtmlRelRefinement::Standard(HtmlRelKeyword::Preload),
            &mut Meter::default(),
        )
        .unwrap();
    owner.refresh(&mut Meter::default()).unwrap();
    let report = owner.finish().unwrap();
    assert_eq!(report.alternate(), HtmlRelAlternate::Feed);
    assert!(report.image_source_preload_context());
}
#[test]
fn every_refresh_admission_failure_hides_report_and_occurrences_including_after_aggregate_writes() {
    let raw = DocumentString::from("vendor alternate stylesheet");
    let href = DocumentString::from("x");
    let title = DocumentString::from("t");
    let make = || {
        let mut o = interpreted(
            HtmlRelInputs::new(Some(HtmlRelRole::Link))
                .with_rel(Some(&raw))
                .with_href(Some(&href))
                .with_title(Some(&title)),
        );
        o.begin_refinement().unwrap();
        o
    };
    let mut complete = make();
    let mut count = Meter::default();
    complete.refresh(&mut count).unwrap();
    // Three occurrence visits, fixed feed comparison, three effect updates,
    // aggregate write admission, then the image-preload context admission.
    assert_eq!(count.work, [12, 12, 12, 44, 5, 5, 5, 5, 3]);
    for i in 0..count.work.len() {
        let mut owner = make();
        let mut meter = Meter {
            fail_at: Some(i),
            ..Meter::default()
        };
        assert_eq!(
            owner.refresh(&mut meter),
            Err(HtmlRelActionError::Admission("original stop"))
        );
        let refusal = owner.finish().unwrap_err();
        assert!(refusal.cause().is_stopped());
        let owner = refusal.into_parts().0;
        assert!(matches!(
            owner.occurrences(),
            Err(ref error) if error.is_stopped()
        ));
        let inputs = owner.abort();
        assert!(std::ptr::eq(inputs.rel().unwrap(), &raw));
    }
}
#[test]
fn initial_error_retains_raw_and_actual_reservation_and_between_action_stop_is_terminal() {
    let raw = DocumentString::from_utf16([120, 0xd800]);
    let mut owner = HtmlRelInterpretation::new(
        HtmlRelInputs::new(Some(HtmlRelRole::Anchor)).with_rel(Some(&raw)),
    );
    let mut meter = Meter {
        fail_reserve: true,
        ..Meter::default()
    };
    assert_eq!(
        owner.interpret(&mut meter),
        Err(HtmlRelActionError::Admission("original reserve stop"))
    );
    assert!(meter.reservations[0].1 >= 1);
    let refusal = owner.finish().unwrap_err();
    assert!(refusal.cause().is_stopped());
    assert!(std::ptr::eq(
        refusal.into_parts().0.abort().rel().unwrap(),
        &raw
    ));
    let mut owner = interpreted(HtmlRelInputs::new(Some(HtmlRelRole::Anchor)).with_rel(Some(&raw)));
    owner.stop();
    assert!(owner.finish().unwrap_err().cause().is_stopped());
}

/// Answers every reservation and counts the reservations that changed the occurrence vector's
/// capacity, found by its record size. A capacity change is exactly one reallocation request,
/// because a vector that already has room is not asked to grow.
#[derive(Default)]
struct CapacityChanges {
    capacity: std::collections::BTreeMap<usize, usize>,
    changes: std::collections::BTreeMap<usize, usize>,
}
impl HtmlRelMeter for CapacityChanges {
    type Error = std::convert::Infallible;
    fn work(&mut self, _: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn reserve(&mut self, guard: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error> {
        guard.try_reserve().unwrap();
        let known = self.capacity.entry(guard.record_size()).or_insert(0);
        if *known != guard.capacity() {
            *known = guard.capacity();
            *self.changes.entry(guard.record_size()).or_insert(0) += 1;
        }
        Ok(())
    }
}

/// Appending n occurrences one at a time requests a logarithmic number of reallocations in n. The
/// oracle is the doubling bound: from any positive capacity, `ceil(log2 n)` doublings reach `n`,
/// so `floor(log2 n) + 2` changes allow the first allocation and one more for rounding. The
/// record-sized report data takes exactly one record, so its single reservation is one change.
#[test]
fn reallocation_requests_for_n_occurrences_grow_at_most_logarithmically() {
    for count in [64_usize, 1_024] {
        let words: Vec<String> = (0..count).map(|i| format!("x{i:06}")).collect();
        let raw = DocumentString::from(words.join(" ").as_str());
        let mut owner = HtmlRelInterpretation::new(
            HtmlRelInputs::new(Some(HtmlRelRole::Anchor)).with_rel(Some(&raw)),
        );
        let mut meter = CapacityChanges::default();
        owner.interpret(&mut meter).unwrap();
        assert_eq!(owner.occurrences().unwrap().len(), count);
        let allowed = count.ilog2() as usize + 2;
        let mut by_size: Vec<_> = meter.changes.into_iter().collect();
        by_size.sort();
        assert_eq!(by_size.len(), 2, "occurrence and data vectors: {by_size:?}");
        for (size, changes) in by_size {
            assert!(
                changes <= allowed,
                "{count} occurrences changed the {size}-byte vector's capacity {changes} times, \
                 more than {allowed}"
            );
        }
    }
}

/// Sums the charges, and reserves like an allocation-admitting consumer.
#[derive(Default)]
struct SumMeter {
    work: usize,
}
impl HtmlRelMeter for SumMeter {
    type Error = std::convert::Infallible;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        self.work += n;
        Ok(())
    }
    fn reserve(&mut self, guard: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error> {
        guard.try_reserve().unwrap();
        Ok(())
    }
}

fn duplicates_and_work(text: &str) -> (Vec<Option<usize>>, usize) {
    let raw = DocumentString::from(text);
    let mut owner = HtmlRelInterpretation::new(
        HtmlRelInputs::new(Some(HtmlRelRole::Link)).with_rel(Some(&raw)),
    );
    let mut meter = SumMeter::default();
    owner.interpret(&mut meter).unwrap();
    let duplicates = owner
        .occurrences()
        .unwrap()
        .iter()
        .map(|occurrence| occurrence.duplicate_of())
        .collect();
    (duplicates, meter.work)
}

/// The `rel` value must be an unordered set of unique space-separated tokens
/// (references/html-language-elements-text-links--877d6146487f--8833a85fd242.md:8471), an
/// author conformance finding, so finding the earlier twin of each token must not compare
/// every pair of tokens. Ordering the tokens costs n log n comparisons, so each doubling may
/// multiply the charge by 2 * (1 + 1 / log2 n), about 2.2, plus jitter from the heap shape; the
/// pairwise comparison multiplies it by four.
#[test]
fn rel_duplicate_detection_charge_grows_near_linearly_with_distinct_tokens() {
    let charges: Vec<usize> = [512, 1024, 2048, 4096]
        .into_iter()
        .map(|tokens| {
            let text: Vec<String> = (0..tokens).map(|n| format!("t{n}")).collect();
            let (duplicates, work) = duplicates_and_work(&text.join(" "));
            assert_eq!(duplicates.len(), tokens);
            assert!(duplicates.iter().all(Option::is_none));
            work
        })
        .collect();
    for pair in charges.windows(2) {
        assert!(
            pair[1] * 10 <= pair[0] * 26,
            "doubling the tokens grew the charge from {} to {}: {charges:?}",
            pair[0],
            pair[1]
        );
    }
}

/// One token repeated is the other extreme: every comparison runs to the end of the token.
#[test]
fn rel_duplicate_detection_charge_grows_near_linearly_with_repeated_tokens() {
    let charges: Vec<usize> = [512, 1024, 2048, 4096]
        .into_iter()
        .map(|tokens| {
            let (duplicates, work) = duplicates_and_work(&"Stylesheet ".repeat(tokens));
            assert_eq!(duplicates.len(), tokens);
            assert_eq!(duplicates[0], None);
            assert!(duplicates[1..].iter().all(|&twin| twin == Some(0)));
            work
        })
        .collect();
    for pair in charges.windows(2) {
        assert!(
            pair[1] * 10 <= pair[0] * 26,
            "doubling the tokens grew the charge from {} to {}: {charges:?}",
            pair[0],
            pair[1]
        );
    }
}

/// The first earlier token equal under ASCII case folding, by a quadratic oracle that shares
/// nothing with the implementation. Non-ASCII letters keep their case: U+00C9 and U+00E9 differ.
#[test]
fn rel_duplicate_of_names_the_first_ascii_case_insensitive_twin() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = move |bound: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % bound as u64) as usize
    };
    let letters = ['a', 'A', 'b', 'B', '\u{c9}', '\u{e9}', 'z', '0'];
    let separators = [" ", "  ", "\t", "\n", "\u{c}", "\r\n", " \t "];
    for _ in 0..400 {
        let tokens: Vec<String> = (0..next(30))
            .map(|_| {
                (0..1 + next(3))
                    .map(|_| letters[next(letters.len())])
                    .collect()
            })
            .collect();
        let mut text = separators[next(separators.len())].repeat(next(2));
        for token in &tokens {
            text.push_str(token);
            text.push_str(separators[next(separators.len())]);
        }
        let (duplicates, _) = duplicates_and_work(&text);
        let folded: Vec<String> = tokens.iter().map(|t| t.to_ascii_lowercase()).collect();
        let expected: Vec<Option<usize>> = (0..folded.len())
            .map(|i| (0..i).find(|&j| folded[j] == folded[i]))
            .collect();
        assert_eq!(duplicates, expected, "{text:?}");
    }
}
