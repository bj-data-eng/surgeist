//! Actual registered-font GPOS across original/computed ownership.
use super::*;
use crate::Range;
use paragraph::{
    ParagraphAnchor, ParagraphAnchorItem, ParagraphControl, ParagraphControlKind, ParagraphOrigin,
    ParagraphOriginKind, ParagraphReplacement, ParagraphReplacementDefinition,
};

fn input(source: Source, direction: ParagraphBaseDirection) -> ParagraphInput {
    let length = source.text().len();
    ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: style(),
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: direction,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![ParagraphOrigin {
            source,
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn prepare(input: ParagraphInput, fixture: &[u8]) -> PreparedText {
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input).unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(fixture.to_vec()).unwrap();
    // Returning only the immutable preparation also tests retained font custody.
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}
fn line(
    prepared: &PreparedText,
    start: PreparedBoundary,
    kind: PreparedBreakKind,
    direction: ParagraphBaseDirection,
) -> FinalizedLine {
    let offer = prepared
        .opportunity_views()
        .find(|view| view.kind() == kind)
        .unwrap();
    let mut value = request(prepared);
    value.start = start;
    value.end = offer.boundary();
    let from = value.start.view().participant_offset();
    let to = value.end.view().participant_offset();
    value.participants = prepared.participants().skip(from).take(to - from).collect();
    value.controls = value
        .participants
        .iter()
        .filter_map(|part| match part {
            PreparedParticipant::Control(control) => Some(ControlAllocation {
                occurrence: control.clone(),
                strut: None,
            }),
            _ => None,
        })
        .collect();
    value.termination = PreparedTermination::Ordinary(offer.handle());
    value.alternative = if kind == PreparedBreakKind::Discretionary {
        Some(offer.alternatives().next().unwrap().handle())
    } else {
        None
    };
    value.context.base_direction = direction;
    value.context.spacing = SpacingPolicy::None;
    value.context.start = if from == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = if kind == PreparedBreakKind::ParagraphEnd {
        LineEnd::Paragraph
    } else {
        LineEnd::Soft
    };
    prepared
        .select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn whole(prepared: &PreparedText, direction: ParagraphBaseDirection) -> FinalizedLine {
    line(
        prepared,
        prepared.boundary_views().next().unwrap().handle(),
        PreparedBreakKind::ParagraphEnd,
        direction,
    )
}
fn glyphs(line: &FinalizedLine, fixture: &[u8]) -> Vec<(Range, u32, f32)> {
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), fixture);
        assert_eq!(run.font_size(), 20.0);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 20.0);
        glyphs.extend(
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    assert!(!glyphs.is_empty());
    glyphs.sort_by_key(|(range, _, _)| (range.start, range.end));
    glyphs
}
fn placed(prepared: &PreparedText, first: FinalizedLine, second: FinalizedLine) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        lines: [first, second]
            .into_iter()
            .enumerate()
            .map(|(index, line)| LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0 + 20.0 * index as f64,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })
            .collect(),
    })
    .unwrap()
}
fn replaced(
    mut value: ParagraphInput,
    source_range: Option<Range>,
    anchor: usize,
    origin: Option<usize>,
    replacement: &str,
) -> ParagraphInput {
    let definition = if let Some(source) = source_range {
        ParagraphReplacementDefinition::Source { origin: 0, source }
    } else {
        value.controls.push(ParagraphControl {
            at: anchor,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(value.base_style.clone()),
            },
        });
        value.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: crate::Affinity::After,
        });
        ParagraphReplacementDefinition::Control { control: 0, origin }
    };
    value.replacements.push(
        ParagraphReplacement::resolved_native_logical_before(
            definition,
            anchor,
            vec![crate::Text::from(replacement)],
        )
        .unwrap(),
    );
    value
}

#[test]
fn actual_pair_positioning_distinguishes_joint_ab_from_two_independent_runs() {
    let fixture = crate::shaping_test_fonts::pair_adjusted_ahem();
    let direction = ParagraphBaseDirection::LeftToRight;
    let ab = whole(
        &prepare(input(Source::new("AB"), direction), &fixture),
        direction,
    );
    let a = whole(
        &prepare(input(Source::new("A"), direction), &fixture),
        direction,
    );
    let b = whole(
        &prepare(input(Source::new("B"), direction), &fixture),
        direction,
    );
    assert_eq!(
        glyphs(&ab, &fixture),
        [(Range::new(0, 1), 35, 10.0), (Range::new(1, 2), 36, 20.0)]
    );
    assert_eq!(glyphs(&a, &fixture), [(Range::new(0, 1), 35, 20.0)]);
    assert_eq!(glyphs(&b, &fixture), [(Range::new(0, 1), 36, 20.0)]);
    assert_eq!(
        (
            ab.feedback().advance,
            a.feedback().advance + b.feedback().advance
        ),
        (30.0, 40.0)
    );
}
fn latin_replacement(source_shy: bool, origin: Option<usize>) {
    let fixture = crate::shaping_test_fonts::pair_adjusted_ahem();
    let direction = ParagraphBaseDirection::LeftToRight;
    let (source, replaced_range, anchor, following) = if source_shy {
        (
            Source::new("A\u{ad}X"),
            Some(Range::new(1, 2)),
            2,
            Range::new(2, 3),
        )
    } else {
        (Source::new("AX"), None, 1, Range::new(1, 2))
    };
    let value = replaced(
        input(source.clone(), direction),
        replaced_range,
        anchor,
        origin,
        "B",
    );
    let prepared = prepare(value, &fixture);
    let unchanged = whole(&prepared, direction);
    let mut expected = vec![(Range::new(0, 1), 35, 20.0)];
    if source_shy {
        // Preserved SHY is default-ignorable: provider hide_default_ignorables
        // retains its source row with this fixture's SPACE3 and zero advance.
        expected.push((Range::new(1, 2), 3, 0.0));
    }
    expected.push((following, 58, 20.0));
    assert_eq!(glyphs(&unchanged, &fixture), expected);
    let first = line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        PreparedBreakKind::Discretionary,
        direction,
    );
    let second = line(
        &prepared,
        first.selected_line().request().end.clone(),
        PreparedBreakKind::ParagraphEnd,
        direction,
    );
    assert_eq!(glyphs(&second, &fixture), [(following, 58, 20.0)]);
    assert_eq!(
        glyphs(&first, &fixture),
        [
            (Range::new(0, 1), 35, 10.0),
            (Range::new(anchor, anchor), 36, 20.0)
        ]
    );
    assert_eq!(first.feedback().advance, 30.0);
    assert_eq!(second.feedback().advance, 20.0);
    let alternative = first.selected_line().request().alternative.clone().unwrap();
    assert_eq!(alternative.prepared_text(), prepared);
    let layout = placed(&prepared, first, second);
    assert_eq!(layout.source(), &source);
    assert_eq!(layout.lines().len(), 2);
}
#[test]
fn selected_source_shy_keeps_pair_positioning_across_original_and_replacement() {
    latin_replacement(true, Some(0));
}
#[test]
fn selected_control_keeps_pair_positioning_with_both_authentic_origin_associations() {
    latin_replacement(false, None);
    latin_replacement(false, Some(0));
}

#[test]
fn arabic_pair_fixture_uses_only_the_medial_glyph_not_the_natural_final_glyph() {
    let fixture = crate::shaping_test_fonts::arabic_pair_adjusted_ahem();
    let direction = ParagraphBaseDirection::RightToLeft;
    // Nonjoining HAMZA ends Beh's natural presentation join. The authored GPOS
    // coverage is medial277 only, so natural final82 must not receive its kern.
    let natural = whole(
        &prepare(
            input(Source::new("\u{628}\u{628}\u{621}"), direction),
            &fixture,
        ),
        direction,
    );
    assert_eq!(
        glyphs(&natural, &fixture),
        [
            (Range::new(0, 1), 100, 20.0),
            (Range::new(1, 2), 82, 20.0),
            (Range::new(2, 3), 36, 20.0)
        ]
    );
    assert_eq!(natural.feedback().advance, 60.0);
}
fn arabic_replacement(source_shy: bool) {
    let fixture = crate::shaping_test_fonts::arabic_pair_adjusted_ahem();
    let direction = ParagraphBaseDirection::RightToLeft;
    let (source, range, anchor, last) = if source_shy {
        (
            Source::new("\u{628}\u{628}\u{ad}\u{628}"),
            Some(Range::new(2, 3)),
            3,
            Range::new(3, 4),
        )
    } else {
        (
            Source::new("\u{628}\u{628}\u{628}"),
            None,
            2,
            Range::new(2, 3),
        )
    };
    let value = replaced(
        input(source.clone(), direction),
        range,
        anchor,
        None,
        "\u{621}",
    );
    let prepared = prepare(value, &fixture);
    let unchanged = whole(&prepared, direction);
    let mut expected = vec![(Range::new(0, 1), 100, 20.0), (Range::new(1, 2), 277, 20.0)];
    if source_shy {
        expected.push((Range::new(2, 3), 3, 0.0));
    }
    expected.push((last, 82, 20.0));
    assert_eq!(glyphs(&unchanged, &fixture), expected);
    assert_eq!(unchanged.feedback().advance, 60.0);
    let first = line(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        PreparedBreakKind::Discretionary,
        direction,
    );
    let second = line(
        &prepared,
        first.selected_line().request().end.clone(),
        PreparedBreakKind::ParagraphEnd,
        direction,
    );
    assert_eq!(glyphs(&second, &fixture), [(last, 82, 20.0)]);
    // Original context assigns medial277, not presentation-derived final82.
    // Joint GPOS then subtracts500 units from that actual medial before HAMZA36.
    assert_eq!(
        glyphs(&first, &fixture),
        [
            (Range::new(0, 1), 100, 20.0),
            (Range::new(1, 2), 277, 10.0),
            (Range::new(anchor, anchor), 36, 20.0)
        ]
    );
    assert_eq!(first.feedback().advance, 50.0);
    let layout = placed(&prepared, first, second);
    assert_eq!(layout.source(), &source);
    assert_eq!(layout.lines().len(), 2);
}
#[test]
fn selected_source_shy_preserves_original_medial_before_joint_pair_positioning() {
    arabic_replacement(true);
}
#[test]
fn selected_control_preserves_original_medial_before_joint_pair_positioning() {
    arabic_replacement(false);
}

#[test]
fn arabic_medial_pair_positioning_uses_the_authored_jeem_neighbor() {
    let fixture = crate::shaping_test_fonts::arabic_pair_adjusted_ahem();
    let direction = ParagraphBaseDirection::RightToLeft;
    let prepared = prepare(
        input(Source::new("\u{628}\u{628}\u{62c}"), direction),
        &fixture,
    );
    let actual = whole(&prepared, direction);
    assert_eq!(
        glyphs(&actual, &fixture),
        [
            (Range::new(0, 1), 100, 20.0),
            (Range::new(1, 2), 277, 10.0),
            (Range::new(2, 3), 36, 20.0)
        ]
    );
    assert_eq!(actual.feedback().advance, 50.0);
}
