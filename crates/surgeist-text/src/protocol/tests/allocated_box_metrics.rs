//! Actual non-control allocations without a default-font contribution.

use super::*;
use crate::{LineBoxAlignment, LineBoxStrut, LineHeight, LineStrut, Range};

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

#[test]
fn allocated_empty_paragraph_uses_only_real_box_metric_groups() {
    // The genuine empty origin is a baseline anchor, not a font strut. The
    // paragraph explicitly requests allocated contributions rather than font
    // metrics, so these literal used boxes are the only metric suppliers.
    let source = Source::new("");
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                line_height: LineHeight::Absolute(100.0),
                ..style()
            },
            paragraph_strut: ParagraphStrut::AllocatedContributions,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![crate::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, 0),
                aggregate: Range::new(0, 0),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            anchor_order: vec![crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::After,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let p = system()
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    let no_allocations = p
        .select(request(&p))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap_err();
    assert!(matches!(
        no_allocations.detail(),
        Some(crate::ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::MissingLineMetrics
        })
    ));

    // Top12 and Bottom20 alone produce a20px line with baseline8. Adding an
    // actual baseline-aligned6/3 box establishes baseline14 in that same20px
    // span. The paragraph's unused100px font line-height contributes nothing.
    for (rows, expected) in [
        (
            vec![
                row(4.0, 12.0, LineBoxAlignment::Top),
                row(8.0, 20.0, LineBoxAlignment::Bottom),
            ],
            (8.0, 20.0),
        ),
        (
            vec![
                row(6.0, 9.0, LineBoxAlignment::Baseline),
                row(4.0, 12.0, LineBoxAlignment::Top),
                row(8.0, 20.0, LineBoxAlignment::Bottom),
            ],
            (14.0, 20.0),
        ),
    ] {
        let mut input = request(&p);
        input.line_box_struts = rows.clone();
        let line = p
            .select(input)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            expected
        );
        assert_eq!(line.feedback().advance, 0.0);
        assert_eq!(
            line.run_views().map(|r| r.glyphs().count()).sum::<usize>(),
            0
        );
        assert_eq!(line.selected_line().request().line_box_struts, rows);
        assert!(line.selected_line().request().controls.is_empty());
        assert!(line.selected_line().request().line_strut.is_none());
    }
}

#[test]
fn measurement_observation_retains_and_validates_real_aligned_box_allocations() {
    let p = prepared(&mut system(), "AB", WhiteSpace::Preserve);
    let portion = p.word_views().next().unwrap().portions().unwrap().remove(0);
    let source_segment = p.source_segments().next().unwrap();
    let flow = p
        .measurement_flow(
            &source_segment,
            &request(&p).start,
            None,
            &PreparedTermination::Measurement(portion),
            None,
        )
        .unwrap();
    let segment = flow
        .segments()
        .find(|s| s.role() == FlowSegmentRole::Target)
        .unwrap();
    let mut input = FlowObservationRequest {
        context: LineContext {
            end: LineEnd::Soft,
            ..request(&p).context
        },
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![row(3.0, 12.0, LineBoxAlignment::Top)],
    };
    let top = p.observe_flow(&segment, &input).unwrap();
    assert_eq!(top.request().line_box_struts, input.line_box_struts);
    assert_eq!(top, p.observe_flow(&segment, &input).unwrap());
    input.line_box_struts[0].alignment = LineBoxAlignment::Bottom;
    let bottom = p.observe_flow(&segment, &input).unwrap();
    assert_ne!(top, bottom);
    assert_eq!(
        top.has_retained_flow_participation(),
        bottom.has_retained_flow_participation()
    );
    assert_eq!(
        top.request().line_box_struts,
        [row(3.0, 12.0, LineBoxAlignment::Top)]
    );
    input.line_box_struts[0].metrics.baseline = -1.0;
    let error = p.observe_flow(&segment, &input).unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::InvalidLineStrut
        })
    ));
}
