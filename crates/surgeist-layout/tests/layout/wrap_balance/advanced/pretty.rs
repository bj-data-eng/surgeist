//! Staged real Text quality and the actual cached native float contexts.
use super::super::numeric::{numeric_atomic, numeric_forced, numeric_node, numeric_tree};
use super::avoid_short_last_line::rows;
use super::*;

struct Profile<'a> {
    source: &'a str,
    width: f64,
    auto: &'a [(usize, f64)],
    pretty: &'a [(usize, f64)],
}
fn text_profiles<S: l::LayoutScalar>() {
    for Profile {
        source,
        width,
        auto,
        pretty,
    } in [
        Profile {
            source: "AAA BB C",
            width: 60.0,
            auto: &[(7, 60.0), (8, 10.0)],
            pretty: &[(4, 30.0), (8, 40.0)],
        },
        Profile {
            source: "AAA BB C CCCCC",
            width: 60.0,
            auto: &[(7, 60.0), (9, 10.0), (14, 50.0)],
            pretty: &[(4, 30.0), (9, 40.0), (14, 50.0)],
        },
        Profile {
            source: "A BBBB C",
            width: 60.0,
            auto: &[(7, 60.0), (8, 10.0)],
            pretty: &[(7, 60.0), (8, 10.0)],
        },
        Profile {
            source: "Circumnavigating the Mississippi river",
            width: 170.0,
            auto: &[(17, 160.0), (33, 150.0), (38, 50.0)],
            pretty: &[(17, 160.0), (33, 150.0), (38, 50.0)],
        },
    ] {
        let mut fixture = Advanced::<S>::new(&[source], width, None, false);
        let ordinary = fixture.compute(&[]);
        rows(&fixture, &ordinary, auto);
        fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::Pretty);
        let selected = fixture.compute(&[0]);
        rows(&fixture, &selected, pretty);
        let repeated = fixture.compute(&[]);
        rows(&fixture, &repeated, pretty);
        rows(&fixture, &ordinary, auto);
        // AAA BB C deliberately retains Tail's30/40 although Body Q rises
        // from pre-Tail Auto0 to1/4. The three-line Body case has equal terminal
        // natural50 and Q MEAN25/72 ->13/72, not the corresponding sums.
    }
}
#[test]
fn pretty_text_stages_tail_then_body_without_transferring_shortness_f32() {
    text_profiles::<f32>();
}
#[test]
fn pretty_text_stages_tail_then_body_without_transferring_shortness_f64() {
    text_profiles::<f64>();
}

fn semantic_prefix(before: &t::FinalizedLine, after: &t::FinalizedLine) {
    assert_eq!(before.prepared_text(), after.prepared_text());
    assert_eq!(
        before.selected_line().request(),
        after.selected_line().request()
    );
    assert_eq!(before.feedback(), after.feedback());
    assert_eq!(before.metric_bounds(), after.metric_bounds());
    assert_eq!(
        before.hyphenation_continuation(),
        after.hyphenation_continuation()
    );
    let a = before.measurements();
    let b = after.measurements();
    assert_eq!(a.formatted_line_usage(), b.formatted_line_usage());
    assert_eq!(a.fit(), b.fit());
    assert_eq!(a.alignment(), b.alignment());
    assert_eq!(a.min_content_contribution(), b.min_content_contribution());
    assert_eq!(a.max_content_contribution(), b.max_content_contribution());
    let slots = |line: &t::FinalizedLine| {
        line.spacing_slots()
            .map(|feedback| {
                (
                    feedback.slot.pieces().collect::<Vec<_>>(),
                    feedback.slot.participants().cloned().collect::<Vec<_>>(),
                    feedback.slot.presentation_range(),
                    feedback.disposition,
                    feedback.advance,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(slots(before), slots(after));
}
fn long_and_justified<S: l::LayoutScalar>() {
    let source = "AAAAAA ".repeat(9) + "AAA BB C CCCCC";
    let mut fixture = Advanced::<S>::new(&[source.as_str()], 60.0, None, false);
    let mut ordinary = (1..=9).map(|i| (i * 7, 60.0)).collect::<Vec<_>>();
    ordinary.extend([(70, 60.0), (72, 10.0), (77, 50.0)]);
    let mut expected = ordinary[..9].to_vec();
    expected.extend([(67, 30.0), (72, 40.0), (77, 50.0)]);
    let auto = fixture.compute(&[]);
    rows(&fixture, &auto, &ordinary);
    let original = fixture.receipts(&auto);
    fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::Pretty);
    let pretty = fixture.compute(&[0]);
    rows(&fixture, &pretty, &expected);
    let actual = fixture.receipts(&pretty);
    for (before, after) in original[..9].iter().zip(&actual[..9]) {
        semantic_prefix(before, after);
    }
    // Nine full prefix rows contribute0 to the11 nonterminal terms:
    // Q25/396 ->13/396. No Forced marker or ten-line exemption is present.
    rows(&fixture, &auto, &ordinary);
    assert_eq!(fixture.receipts(&auto), original); // Original-packet identity.

    let mut justified = Advanced::<S>::new(&["AA B C D E FFFFF"], 60.0, None, false);
    justified.root(|root| {
        root.text_align = l::TextAlign::Justify;
        root.text_align_last = l::TextAlignLast::Start;
        root.text_justification = l::TextJustification::Spaces;
    });
    let auto = justified.compute(&[]);
    rows(&justified, &auto, &[(7, 60.0), (11, 30.0), (16, 50.0)]);
    justified.root(|root| root.text_wrap_style = l::TextWrapStyle::Pretty);
    let pretty = justified.compute(&[0]);
    rows(&justified, &pretty, &[(5, 40.0), (11, 50.0), (16, 50.0)]);
    // Both actual programs allocate60/60 on nonterminal rows. Natural Body
    // Q1/8 ->5/72 remains distinct; score allocated widths and it disappears.
    for batch in [&auto, &pretty] {
        let receipts = justified.receipts(batch);
        for line in &receipts[..2] {
            assert_eq!(line.measurements().alignment().advance(), 60.0);
            assert!(line.spacing_slots().any(|slot| matches!(
                slot.disposition,
                t::SpacingSlotDisposition::Retained { .. }
            )));
            assert_eq!(
                line.selected_line().request().context.available_inline,
                Some(60.0)
            );
        }
        assert_eq!(receipts[2].measurements().alignment().advance(), 50.0);
    }
}
#[test]
fn pretty_text_body_exceeds_ten_lines_and_scores_natural_not_justified_rows_f32() {
    long_and_justified::<f32>();
}
#[test]
fn pretty_text_body_exceeds_ten_lines_and_scores_natural_not_justified_rows_f64() {
    long_and_justified::<f64>();
}

fn source_and_cache<S: l::LayoutScalar>() {
    let mut old = Advanced::<S>::with_revision(&["AAA BB C"], 60.0, None, false, 3);
    old.root(|r| r.text_wrap_style = l::TextWrapStyle::Pretty);
    let packet = old.compute(&[]);
    rows(&old, &packet, &[(4, 30.0), (8, 40.0)]);
    packet.apply_to(&mut old.host).unwrap();
    let old_receipts = old.receipts(&packet);
    let old_source = packet.placed_text()[0].layout().source().clone();
    let mut current = Advanced::<S>::with_revision(&["AAA BB CCCC"], 60.0, None, false, 4);
    current.root(|r| r.text_wrap_style = l::TextWrapStyle::Pretty);
    let cold = current.compute(&[]);
    rows(&current, &cold, &[(7, 60.0), (11, 40.0)]);
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
    invalidated.apply_to(&mut current.host).unwrap();
    rows(&current, &current.compute(&[]), &[(7, 60.0), (11, 40.0)]);
    assert_eq!(old.receipts(&packet), old_receipts);
    rows(&old, &packet, &[(4, 30.0), (8, 40.0)]);

    let mut fixture = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
    let auto = fixture.compute(&[]);
    assert_float(&fixture, &auto, false);
    auto.apply_to(&mut fixture.host).unwrap();
    let mut retained = vec![(auto, false)];
    for style in [
        l::TextWrapStyle::Pretty,
        l::TextWrapStyle::Balance,
        l::TextWrapStyle::Stable,
        l::TextWrapStyle::Pretty,
    ] {
        let repaired = style != l::TextWrapStyle::Stable;
        fixture.root(|r| r.text_wrap_style = style);
        let selected = fixture.compute(&[0]);
        assert_float(&fixture, &selected, repaired);
        selected.apply_to(&mut fixture.host).unwrap();
        let leaf = fixture.float.unwrap() as usize;
        let before = fixture.host.hits[leaf].get();
        let warm = fixture.compute(&[]);
        assert_float(&fixture, &warm, repaired);
        assert!(
            fixture.host.hits[leaf].get() > before,
            "positive warm proof is the genuine eligible float leaf"
        );
        warm.apply_to(&mut fixture.host).unwrap();
        let mut fresh = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
        fresh.root(|r| r.text_wrap_style = style);
        assert_float(&fresh, &fresh.compute(&[]), repaired);
        retained.push((selected, repaired));
        for (prior, was_repaired) in &retained {
            assert_float(&fixture, prior, *was_repaired);
        }
    }
    // Text finalization is authentic reexecution; it is not a Text-root hit.
}
#[test]
fn pretty_text_source_revision_and_real_warm_float_leaf_keep_publication_coherent_f32() {
    source_and_cache::<f32>();
}
#[test]
fn pretty_text_source_revision_and_real_warm_float_leaf_keep_publication_coherent_f64() {
    source_and_cache::<f64>();
}

fn forced_body<S: l::LayoutScalar>() {
    let mut fixture = Advanced::<S>::new(&["AAA BB C CCCCC", "AAA BB C"], 60.0, None, false);
    fixture.root(|r| r.text_wrap_style = l::TextWrapStyle::Pretty);
    let packet = fixture.compute(&[]);
    assert_eq!(
        fixture.cuts_and_natural(&packet),
        [(4, 30.0), (9, 40.0), (14, 50.0), (18, 30.0), (22, 40.0)]
    );
    let receipts = fixture.receipts(&packet);
    for (i, line) in receipts.iter().enumerate() {
        let selected = line.selected_line();
        let request = selected.request();
        assert_eq!(request.context.available_inline, Some(60.0));
        assert_eq!(
            request.context.end,
            if i == 2 {
                t::LineEnd::Forced
            } else if i == 4 {
                t::LineEnd::Paragraph
            } else {
                t::LineEnd::Soft
            }
        );
        let published = packet.placed_text()[0].layout().lines()[i];
        assert_eq!(
            published.bounds(),
            t::Rect::new(0.0, 10.0 * i as f64, 60.0, 10.0)
        );
        assert_eq!(published.baseline(), 8.0 + 10.0 * i as f64);
    }
    assert_eq!(
        receipts[2].selected_line().request().controls[0].occurrence,
        fixture.binding.controls[0].occurrence
    );
    assert_eq!(
        packet
            .final_entries()
            .iter()
            .find(|e| e.node() == 0)
            .unwrap()
            .output()
            .size,
        l::Size::new(S::from_f64(60.0), S::from_f64(50.0))
    );
    assert_eq!(
        fixture.cuts_and_natural(&fixture.compute(&[])),
        fixture.cuts_and_natural(&packet)
    );
}
#[test]
fn pretty_text_body_improves_forced_group_before_paragraph_tail_f32() {
    forced_body::<f32>();
}
#[test]
fn pretty_text_body_improves_forced_group_before_paragraph_tail_f64() {
    forced_body::<f64>();
}

fn native_float_host<S: l::LayoutScalar>(entry: bool) -> Host<S> {
    let n = S::from_f64;
    let float = l::LayoutInputOf::box_input(l::NodeInputOf {
        display: l::Display::Block,
        float: l::Float::Left,
        float_exclusion: if entry {
            l::FloatExclusion::MarginBox
        } else {
            l::FloatExclusion::Shape
        },
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(n(20.0)).unwrap()),
            l::PreferredSizeOf::value(
                l::LengthPercentageOf::px(n(if entry { 60.0 } else { 20.0 })).unwrap(),
            ),
        ),
        ..l::NodeInputOf::default()
    });
    let atom = |width, height| {
        numeric_atomic(
            l::Direction::Ltr,
            width,
            height,
            l::InlineBreakOpportunity::Allowed,
        )
    };
    let participants = if entry {
        vec![
            float,
            atom(30.0, 10.0),
            atom(30.0, 30.0),
            atom(20.0, 30.0),
            atom(50.0, 10.0),
            numeric_forced(l::Direction::Ltr, 10.0, 10.0),
            atom(30.0, 10.0),
            atom(30.0, 10.0),
            atom(10.0, 10.0),
        ]
    } else {
        vec![float, atom(60.0, 10.0), atom(0.0, 30.0), atom(10.0, 10.0)]
    };
    let mut tree = numeric_tree(l::Direction::Ltr, participants);
    let root = tree.inputs[0].as_box().cloned().unwrap();
    tree.inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(n(80.0)).unwrap()),
            l::PreferredSizeOf::AUTO,
        ),
        ..root
    });
    let mut host = Host::new(tree);
    host.triangle = (!entry).then_some(1);
    host
}
fn native_compute<S: l::LayoutScalar>(
    host: &Host<S>,
    changed: &[u32],
) -> l::CompletedLayoutBatchOf<u32, S> {
    l::compute_layout_invalidated(
        host,
        0,
        l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(80.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
        changed,
    )
    .unwrap()
}
fn native_style<S: l::LayoutScalar>(host: &mut Host<S>, style: l::TextWrapStyle) {
    let root = host.tree.inputs[0].as_box().cloned().unwrap();
    host.tree.inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
        text_wrap_style: style,
        ..root
    });
}
fn native_packet<S: l::LayoutScalar>(
    packet: &l::CompletedLayoutBatchOf<u32, S>,
    expected: &[(u32, f64, f64, f64, f64)],
    height: f64,
) {
    assert!(packet.placed_text().is_empty());
    assert!(packet.final_inline_fragments().is_empty());
    for entries in [packet.unrounded_entries(), packet.final_entries()] {
        assert_eq!(entries.len(), expected.len() + 1);
        assert_eq!(
            numeric_node(entries, 0).size,
            l::Size::new(S::from_f64(80.0), S::from_f64(height))
        );
        for &(id, x, y, width, block) in expected {
            assert_eq!(entries.iter().filter(|e| e.node() == id).count(), 1);
            let output = numeric_node(entries, id);
            assert_eq!(output.source_index, l::SourceIndex::new(id as usize - 1));
            assert_eq!(
                output.location,
                l::Point::new(S::from_f64(x), S::from_f64(y))
            );
            assert_eq!(
                output.size,
                l::Size::new(S::from_f64(width), S::from_f64(block))
            );
        }
    }
}
fn streaming_entry<S: l::LayoutScalar>() {
    let auto_expected = [
        (1, 0.0, 0.0, 20.0, 60.0),
        (2, 20.0, 20.0, 30.0, 10.0),
        (3, 50.0, 0.0, 30.0, 30.0),
        (4, 20.0, 30.0, 20.0, 30.0),
        (5, 0.0, 60.0, 50.0, 10.0),
        (6, 50.0, 70.0, 0.0, 0.0),
        (7, 0.0, 70.0, 30.0, 10.0),
        (8, 30.0, 70.0, 30.0, 10.0),
        (9, 60.0, 70.0, 10.0, 10.0),
    ];
    let pretty_expected = [
        (1, 0.0, 0.0, 20.0, 60.0),
        (2, 20.0, 0.0, 30.0, 10.0),
        (3, 20.0, 10.0, 30.0, 30.0),
        (4, 50.0, 10.0, 20.0, 30.0),
        (5, 20.0, 40.0, 50.0, 10.0),
        (6, 70.0, 50.0, 0.0, 0.0),
        (7, 20.0, 50.0, 30.0, 10.0),
        (8, 0.0, 60.0, 30.0, 10.0),
        (9, 30.0, 60.0, 10.0, 10.0),
    ];
    let mut host = native_float_host::<S>(true);
    let original = native_compute(&host, &[]);
    native_packet(&original, &auto_expected, 80.0);
    original.apply_to(&mut host).unwrap();
    native_style(&mut host, l::TextWrapStyle::Pretty);
    let selected = native_compute(&host, &[0]);
    native_packet(&selected, &pretty_expected, 70.0);
    assert!(selected.invalidated_nodes().contains(&0));
    selected.apply_to(&mut host).unwrap();
    // Invalidate only the root so an eligible whole-native-root cache hit
    // cannot bypass the particular float leaf reuse being observed.
    let before = host.hits[1].get();
    let warm = native_compute(&host, &[0]);
    native_packet(&warm, &pretty_expected, 70.0);
    assert!(
        host.hits[1].get() > before,
        "real native float leaf cache answer required"
    );
    warm.apply_to(&mut host).unwrap();
    native_packet(&native_compute(&host, &[]), &pretty_expected, 70.0);
    let mut fresh = native_float_host::<S>(true);
    native_style(&mut fresh, l::TextWrapStyle::Pretty);
    native_packet(&native_compute(&fresh, &[]), &pretty_expected, 70.0);
    // First Body changes outgoing entry70->50. There local Auto has TWO
    // EF/G rows (not stale EFG/count1); Tail then chooses E/FG. Thus global
    // count4->5 while root80->70. Private owner tests prove phase invocation.
    native_packet(&original, &auto_expected, 80.0);
    native_packet(&selected, &pretty_expected, 70.0);
    native_style(&mut host, l::TextWrapStyle::Auto);
    native_packet(&native_compute(&host, &[0]), &auto_expected, 80.0);
}
#[test]
fn pretty_native_body_changes_actual_later_entry_and_local_count_with_real_cache_f32() {
    streaming_entry::<f32>();
}
#[test]
fn pretty_native_body_changes_actual_later_entry_and_local_count_with_real_cache_f64() {
    streaming_entry::<f64>();
}

fn triangle_contrast<S: l::LayoutScalar>() {
    let ordinary = [
        (1, 0.0, 0.0, 20.0, 20.0),
        (2, 20.0, 20.0, 60.0, 10.0),
        (3, 80.0, 0.0, 0.0, 30.0),
        (4, 0.0, 30.0, 10.0, 10.0),
    ];
    let balanced = [
        (1, 0.0, 0.0, 20.0, 20.0),
        (2, 10.0, 0.0, 60.0, 10.0),
        (3, 20.0, 10.0, 0.0, 30.0),
        (4, 20.0, 30.0, 10.0, 10.0),
    ];
    let mut host = native_float_host::<S>(false);
    let auto = native_compute(&host, &[]);
    native_packet(&auto, &ordinary, 40.0);
    native_style(&mut host, l::TextWrapStyle::Balance);
    let balance = native_compute(&host, &[0]);
    native_packet(&balance, &balanced, 40.0);
    native_style(&mut host, l::TextWrapStyle::Pretty);
    let pretty = native_compute(&host, &[0]);
    native_packet(&pretty, &ordinary, 40.0);
    // Genuine contour gives ordinary capacities60/80, alternate70/60.
    // Both natural tails10: Tail rejects; Body Q0 beats1/49, so Pretty
    // differs from this actually executed within-cap Balance alternative.
    let repeated = native_compute(&host, &[]);
    native_packet(&repeated, &ordinary, 40.0);
    native_packet(&auto, &ordinary, 40.0);
    native_packet(&balance, &balanced, 40.0);
}
#[test]
fn pretty_real_triangle_keeps_zero_atomic_custody_and_differs_from_balance_f32() {
    triangle_contrast::<f32>();
}
#[test]
fn pretty_real_triangle_keeps_zero_atomic_custody_and_differs_from_balance_f64() {
    triangle_contrast::<f64>();
}
