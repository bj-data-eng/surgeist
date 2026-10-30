//! Actual used numerical box contributions join the existing native metric groups.
use super::*;
use crate::{
    LineBoxAlignment, LineBoxStrut, LineHeight, LineStrut, OverflowWrap, Range, TextNumericError,
    TextNumericPhase, Wrap,
};
use std::hash::{Hash, Hasher};

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn prepare(text: &str, overflow: OverflowWrap) -> PreparedText {
    let source = Source::identified(text, crate::Id::from_u64(75083), 7);
    let end = source.text().len();
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                line_height: LineHeight::Absolute(10.0),
                white_space: WhiteSpace::Preserve,
                word_break: crate::WordBreak::Normal,
                wrap: Wrap::Word,
                overflow_wrap: overflow,
                ..Style::default()
            },
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![crate::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, end),
                aggregate: Range::new(0, end),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: vec![],
            anchor_order: vec![],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = system();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn row(baseline: f64, line_extent: f64, alignment: LineBoxAlignment) -> LineBoxStrut {
    LineBoxStrut {
        scope: None,
        metrics: LineStrut {
            baseline,
            line_extent,
        },
        alignment,
    }
}
fn whole(
    p: &PreparedText,
    rows: Vec<LineBoxStrut>,
    line_strut: Option<LineStrut>,
    available: f64,
) -> FinalizedLine {
    let mut r = request(p);
    r.line_box_struts = rows;
    r.line_strut = line_strut;
    r.context.available_inline = Some(available);
    p.select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}
fn fingerprint(value: &impl Hash) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut h);
    h.finish()
}

#[test]
fn used_baseline_top_and_bottom_rows_are_combined_once_with_real_font_and_line_strut() {
    let p = prepare("A", OverflowWrap::Normal);
    let source = p.analyzed_source().source().clone();
    let neutral = whole(&p, vec![], None, 100.0);
    assert_eq!(
        (neutral.feedback().baseline, neutral.feedback().line_height),
        (8.0, 10.0)
    );
    let baseline = || row(12.0, 20.0, LineBoxAlignment::Baseline);
    let top = || row(3.0, 30.0, LineBoxAlignment::Top);
    let bottom = || row(7.0, 40.0, LineBoxAlignment::Bottom);
    // Ahem supplies a real 8/2 baseline group at size 10. Baseline allocation
    // gives 12/8, Top requires 30, and Bottom 40 grows the over side by 10.
    for (rows, line_strut, expected) in [
        (vec![], None, (8.0, 10.0)),
        (vec![baseline()], None, (12.0, 20.0)),
        (vec![top()], None, (8.0, 30.0)),
        (vec![bottom()], None, (38.0, 40.0)),
        (vec![baseline(), top(), bottom()], None, (22.0, 40.0)),
        (vec![bottom(), top(), baseline()], None, (22.0, 40.0)),
        (
            vec![baseline(), top(), bottom()],
            Some(LineStrut {
                baseline: 15.0,
                line_extent: 18.0,
            }),
            (25.0, 40.0),
        ),
    ] {
        let line = whole(&p, rows.clone(), line_strut, 100.0);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            expected
        );
        assert_eq!(line.feedback().advance, 10.0);
        assert!(line.run_views().all(|run| run.face().bytes() == AHEM));
        assert_eq!(line.selected_line().request().line_box_struts, rows);
        assert_eq!(line.selected_line().request().line_strut, line_strut);
        assert_eq!(
            line.run_views().map(|run| run.origin()).collect::<Vec<_>>(),
            [crate::Point::new(0.0, expected.0)]
        );
        assert_eq!(
            line.run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|g| (g.id(), g.advance(), g.range()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [(35, 10.0, Range::new(0, 1))]
        );
        assert_eq!(line.prepared_text().analyzed_source().source(), &source);
    }
    assert_eq!(
        (neutral.feedback().baseline, neutral.feedback().line_height),
        (8.0, 10.0)
    );
    assert!(neutral.selected_line().request().line_box_struts.is_empty());
}

#[test]
fn every_supplied_box_row_has_real_selection_phase_metric_validation() {
    let p = prepare("A", OverflowWrap::Normal);
    for invalid in [
        row(f64::NAN, 20.0, LineBoxAlignment::Top),
        row(7.0, f64::INFINITY, LineBoxAlignment::Bottom),
    ] {
        let mut r = request(&p);
        r.line_box_struts = vec![invalid];
        let error = p.select(r).unwrap_err();
        assert!(
            matches!(
                error.detail(),
                Some(crate::ErrorDetail::InvalidTextNumeric {
                    phase: TextNumericPhase::Selection,
                    category: TextNumericError::NonFinite,
                    ..
                })
            ),
            "{error:?}"
        );
    }
    for invalid in [
        row(-1.0, 10.0, LineBoxAlignment::Baseline),
        row(12.0, 11.0, LineBoxAlignment::Top),
    ] {
        let mut r = request(&p);
        r.line_box_struts = vec![invalid];
        let error = p.select(r).unwrap_err();
        assert!(
            matches!(reason(&error), ProtocolError::InvalidLineStrut),
            "{error:?}"
        );
    }
    let zero = whole(
        &p,
        vec![row(0.0, 0.0, LineBoxAlignment::Bottom)],
        None,
        100.0,
    );
    assert_eq!(
        (zero.feedback().baseline, zero.feedback().line_height),
        (8.0, 10.0)
    );
    assert_eq!(zero.selected_line().request().line_box_struts.len(), 1);
}

#[test]
fn conditional_identity_retains_box_alignment_without_freezing_candidate_geometry() {
    let p = prepare("AAAA", OverflowWrap::Anywhere);
    assert_eq!(
        p.opportunity_views().map(|o| o.kind()).collect::<Vec<_>>(),
        [PreparedBreakKind::ParagraphEnd]
    );
    let top = whole(&p, vec![row(7.0, 20.0, LineBoxAlignment::Top)], None, 20.0);
    let top_again = whole(&p, vec![row(7.0, 20.0, LineBoxAlignment::Top)], None, 20.0);
    let bottom = whole(
        &p,
        vec![row(7.0, 20.0, LineBoxAlignment::Bottom)],
        None,
        20.0,
    );
    assert_eq!(
        (
            top.feedback().advance,
            top.feedback().baseline,
            top.feedback().line_height
        ),
        (40.0, 8.0, 20.0)
    );
    assert_eq!(
        (
            bottom.feedback().advance,
            bottom.feedback().baseline,
            bottom.feedback().line_height
        ),
        (40.0, 18.0, 20.0)
    );
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = p.opportunity_views().next().unwrap().handle();
    let query = |frame: &FinalizedLine| {
        p.conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(frame),
        )
        .unwrap()
    };
    let a = query(&top);
    let equivalent = query(&top_again);
    let b = query(&bottom);
    assert_eq!(a, equivalent);
    assert_eq!(fingerprint(&a), fingerprint(&equivalent));
    assert_ne!(a, b);
    let cuts = a.begin(&[]).unwrap();
    assert_eq!(cuts.stage(), ConditionalStage::Arbitrary);
    assert_eq!(
        cuts.opportunities()
            .iter()
            .map(|cut| cut.boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    // Enumeration records actual permission only. The full ordinary frame is
    // the genuine 40px-at20px overflow receipt; no prefix-fit proof is invented.
    let positive = whole(&p, vec![row(0.0, 10.0, LineBoxAlignment::Top)], None, 20.0);
    let negative = whole(&p, vec![row(-0.0, 10.0, LineBoxAlignment::Top)], None, 20.0);
    let positive = query(&positive);
    let negative = query(&negative);
    assert_eq!(positive, negative);
    assert_eq!(fingerprint(&positive), fingerprint(&negative));
}
