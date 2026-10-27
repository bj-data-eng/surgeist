//! Actual source/cut witnesses for threshold-free paragraph-tail repair.
use super::*;

pub(super) fn rows<S: l::LayoutScalar>(
    fixture: &Advanced<S>,
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    expected: &[(usize, f64)],
) {
    assert_eq!(fixture.cuts_and_natural(batch), expected);
    let receipts = fixture.receipts(batch);
    let layout = batch.placed_text()[0].layout();
    let mut start = 0;
    for (i, ((line, receipt), &(end, _))) in layout
        .lines()
        .iter()
        .zip(receipts)
        .zip(expected)
        .enumerate()
    {
        assert_eq!(line.range(), t::Range::new(start, end));
        assert_eq!(
            line.bounds(),
            t::Rect::new(0.0, i as f64 * 10.0, fixture.width, 10.0)
        );
        assert_eq!(line.baseline(), i as f64 * 10.0 + 8.0);
        let selected = receipt.selected_line();
        let request = selected.request();
        assert_eq!(request.context.available_inline, Some(fixture.width));
        assert_eq!(request.context.indent, 0.0);
        let ending = if i + 1 == expected.len() {
            t::LineEnd::Paragraph
        } else {
            t::LineEnd::Soft
        };
        assert_eq!(request.context.end, ending);
        assert_eq!(request.start.view().source_after().get(), start);
        assert_eq!(request.end.view().source_after().get(), end);
        let t::PreparedTermination::Ordinary(offer) = &request.termination else {
            panic!("genuine ordinary source offer required")
        };
        assert_eq!(
            offer.view().kind(),
            if ending == t::LineEnd::Paragraph {
                t::PreparedBreakKind::ParagraphEnd
            } else {
                t::PreparedBreakKind::Allowed
            }
        );
        assert!(request.alternative.is_none());
        assert!(
            receipt
                .run_views()
                .all(|run| run.font_size() == 10.0 && run.face().bytes() == AHEM)
        );
        start = end;
    }
    let root = batch
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(
            S::from_f64(fixture.width),
            S::from_f64(10.0 * expected.len() as f64)
        )
    );
}
struct TextProfile<'a> {
    source: &'a str,
    width: f64,
    ordinary: &'a [(usize, f64)],
    repaired: &'a [(usize, f64)],
}
fn literal_profiles<S: l::LayoutScalar>() {
    let profiles = [
        TextProfile {
            source: "AAA BB C",
            width: 60.0,
            ordinary: &[(7, 60.0), (8, 10.0)],
            repaired: &[(4, 30.0), (8, 40.0)],
        },
        // Longer tail alone is insufficient: moving BBBB gives10/60.
        TextProfile {
            source: "A BBBB C",
            width: 60.0,
            ordinary: &[(7, 60.0), (8, 10.0)],
            repaired: &[(7, 60.0), (8, 10.0)],
        },
        // Whole Balance may improve30/40/50, but this mode freezes cut7.
        TextProfile {
            source: "AAA BB C CCCCC",
            width: 60.0,
            ordinary: &[(7, 60.0), (9, 10.0), (14, 50.0)],
            repaired: &[(7, 60.0), (9, 10.0), (14, 50.0)],
        },
        // Pinned Mississippi:160/150/50 must not become160/30/170.
        TextProfile {
            source: "Circumnavigating the Mississippi river",
            width: 170.0,
            ordinary: &[(17, 160.0), (33, 150.0), (38, 50.0)],
            repaired: &[(17, 160.0), (33, 150.0), (38, 50.0)],
        },
    ];
    for TextProfile {
        source,
        width,
        ordinary,
        repaired,
    } in profiles
    {
        let mut f = Advanced::<S>::new(&[source], width, None, false);
        let auto = f.compute(&[]);
        rows(&f, &auto, ordinary);
        f.root(|r| r.text_wrap_style = l::TextWrapStyle::AvoidShortLastLine);
        let actual = f.compute(&[0]);
        rows(&f, &actual, repaired);
        let replay = f.compute(&[]);
        rows(&f, &replay, repaired);
        rows(&f, &auto, ordinary);
        let mut fresh = Advanced::<S>::new(&[source], width, None, false);
        fresh.root(|r| r.text_wrap_style = l::TextWrapStyle::AvoidShortLastLine);
        rows(&fresh, &fresh.compute(&[]), repaired);
    }
}
#[test]
fn avoid_short_last_line_text_repairs_without_transferring_or_rebalancing_prefix_f32() {
    literal_profiles::<f32>();
}
#[test]
fn avoid_short_last_line_text_repairs_without_transferring_or_rebalancing_prefix_f64() {
    literal_profiles::<f64>();
}

fn coherent_revision<S: l::LayoutScalar>() {
    let mut old = Advanced::<S>::with_revision(&["AAA BB C"], 60.0, None, false, 3);
    old.root(|r| r.text_wrap_style = l::TextWrapStyle::AvoidShortLastLine);
    let published = old.compute(&[]);
    rows(&old, &published, &[(4, 30.0), (8, 40.0)]);
    published.apply_to(&mut old.host).unwrap();
    let old_source = published.placed_text()[0].layout().source().clone();
    let old_receipts = old.receipts(&published);
    let mut current = Advanced::<S>::with_revision(&["AAA BB CCCC"], 60.0, None, false, 4);
    current.root(|r| r.text_wrap_style = l::TextWrapStyle::AvoidShortLastLine);
    let fresh = current.compute(&[]);
    rows(&current, &fresh, &[(7, 60.0), (11, 40.0)]);
    assert_eq!(old_source.id(), current.capture.source().id());
    assert_eq!(old_source.revision(), 3);
    assert_eq!(current.capture.source().revision(), 4);
    assert_eq!(
        old_source.text().code_units(),
        &current.capture.source().text().code_units()[..8]
    );
    assert_ne!(old.prepared, current.prepared);
    current.host.published = old.host.published.clone();
    let invalidated = current.compute(&[0]);
    assert!(invalidated.invalidated_nodes().contains(&0));
    rows(&current, &invalidated, &[(7, 60.0), (11, 40.0)]);
    assert_eq!(
        current.cuts_and_natural(&fresh),
        current.cuts_and_natural(&invalidated)
    );
    invalidated.apply_to(&mut current.host).unwrap();
    let repeated = current.compute(&[]);
    rows(&current, &repeated, &[(7, 60.0), (11, 40.0)]);
    assert_eq!(
        current.cuts_and_natural(&repeated),
        current.cuts_and_natural(&fresh)
    );
    rows(&old, &published, &[(4, 30.0), (8, 40.0)]);
    assert_eq!(old.receipts(&published), old_receipts);
    assert_eq!(published.placed_text()[0].layout().source(), &old_source);
}
#[test]
fn avoid_short_last_line_text_revision_invalidates_current_source_without_mutating_old_packet_f32()
{
    coherent_revision::<f32>();
}
#[test]
fn avoid_short_last_line_text_revision_invalidates_current_source_without_mutating_old_packet_f64()
{
    coherent_revision::<f64>();
}

fn real_float_cache<S: l::LayoutScalar>() {
    let mut f = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
    let auto = f.compute(&[]);
    assert_float(&f, &auto, false);
    auto.apply_to(&mut f.host).unwrap();
    let mut retained = vec![(auto, false)];
    for style in [
        l::TextWrapStyle::AvoidShortLastLine,
        l::TextWrapStyle::Balance,
        l::TextWrapStyle::Stable,
        l::TextWrapStyle::AvoidShortLastLine,
    ] {
        let repaired = style != l::TextWrapStyle::Stable;
        f.root(|r| r.text_wrap_style = style);
        let switched = f.compute(&[0]);
        assert!(switched.invalidated_nodes().contains(&0));
        // Ordinary60/10 uses spans60/80; repair30/40 uses spans60/60.
        // Natural tail10->40 increases, unlike the private narrow-band guard.
        assert_float(&f, &switched, repaired);
        switched.apply_to(&mut f.host).unwrap();
        let leaf = f.float.unwrap() as usize;
        let before = f.host.hits[leaf].get();
        let warm = f.compute(&[]);
        assert_float(&f, &warm, repaired);
        assert!(
            f.host.hits[leaf].get() > before,
            "actual eligible float leaf cache hit required"
        );
        warm.apply_to(&mut f.host).unwrap();
        let mut fresh = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
        fresh.root(|r| r.text_wrap_style = style);
        assert_float(&fresh, &fresh.compute(&[]), repaired);
        retained.push((switched, repaired));
        for (packet, was_repaired) in &retained {
            assert_float(&f, packet, *was_repaired);
        }
    }
    // Positive caching is the real float leaf; Text finalization reexecutes.
}
#[test]
fn avoid_short_last_line_real_float_bands_and_warm_leaf_survive_mode_replay_f32() {
    real_float_cache::<f32>();
}
#[test]
fn avoid_short_last_line_real_float_bands_and_warm_leaf_survive_mode_replay_f64() {
    real_float_cache::<f64>();
}

fn long_final_group<S: l::LayoutScalar>() {
    // Ten50-unit words cannot share a60-unit row with the next word. Their
    // real trailing SPACE is consumed at each soft cut, six source units each.
    // The final AAA BB C is the ordinary60/10 versus repaired30/40 witness.
    let source = "AAAAA ".repeat(10) + "AAA BB C";
    let mut fixture = Advanced::<S>::new(&[source.as_str()], 60.0, None, false);
    let mut ordinary = (1..=10).map(|i| (i * 6, 50.0)).collect::<Vec<_>>();
    ordinary.extend([(67, 60.0), (68, 10.0)]);
    let mut repaired = ordinary[..10].to_vec();
    repaired.extend([(64, 30.0), (68, 40.0)]);
    let auto = fixture.compute(&[]);
    rows(&fixture, &auto, &ordinary);
    let prefix = fixture.receipts(&auto);
    fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::AvoidShortLastLine);
    let actual = fixture.compute(&[0]);
    rows(&fixture, &actual, &repaired);
    let actual_receipts = fixture.receipts(&actual);
    // FinalizedLine equality is allocation identity (not semantic equality)
    // across computations. Compare the actual public prefix state instead.
    for (before, after) in prefix[..10].iter().zip(&actual_receipts[..10]) {
        assert_eq!(before.prepared_text(), after.prepared_text());
        assert_eq!(before.prepared_text(), fixture.prepared);
        assert_eq!(
            before.selected_line().request(),
            after.selected_line().request()
        );
        assert_eq!(before.has_in_flow_content(), after.has_in_flow_content());
        assert_eq!(before.feedback(), after.feedback());
        assert_eq!(before.metric_bounds(), after.metric_bounds());
        assert_eq!(
            before.hyphenation_continuation(),
            after.hyphenation_continuation()
        );
        let left = before.measurements();
        let right = after.measurements();
        assert_eq!(left.formatted_line_usage(), right.formatted_line_usage());
        assert_eq!(left.fit(), right.fit());
        assert_eq!(left.alignment(), right.alignment());
        assert_eq!(
            left.min_content_contribution(),
            right.min_content_contribution()
        );
        assert_eq!(
            left.max_content_contribution(),
            right.max_content_contribution()
        );
        let spacing = |line: &t::FinalizedLine| {
            line.spacing_slots()
                .map(|feedback| {
                    (
                        feedback.slot.owner(),
                        feedback.slot.source_range(),
                        feedback.slot.presentation_range(),
                        feedback.disposition,
                        feedback.advance,
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(spacing(before), spacing(after));
    }
    for (before, after) in auto.placed_text()[0].layout().lines()[..10]
        .iter()
        .zip(&actual.placed_text()[0].layout().lines()[..10])
    {
        assert_eq!(before.range(), after.range());
        assert_eq!(before.bounds(), after.bounds());
        assert_eq!(before.baseline(), after.baseline());
    }
    let repeated = fixture.compute(&[]);
    rows(&fixture, &repeated, &repaired);
    rows(&fixture, &auto, &ordinary);
    // Re-reading the ORIGINAL packet retains its exact minted line handles.
    assert_eq!(fixture.receipts(&auto), prefix);
}
#[test]
fn avoid_short_last_line_twelve_line_text_group_repairs_only_its_real_tail_f32() {
    long_final_group::<f32>();
}
#[test]
fn avoid_short_last_line_twelve_line_text_group_repairs_only_its_real_tail_f64() {
    long_final_group::<f64>();
}
