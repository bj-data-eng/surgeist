//! Old allocated API characterization: absence is not a zero parent strut.
use super::*;
use crate::{
    Affinity, AtomicBoxAllocation, AtomicFlowAllocation, AtomicLineAllocation, Id, InlineBox,
    InlineBoxKind, LineBoxAlignment, Point, Range, Rect, Size,
};

#[test]
fn actual_fontless_signed_atomic_keeps_ten_unit_occupied_extent_without_a_parent_strut() {
    let mut source = Source::new("");
    source.inline_box(InlineBox::new(
        Id::from_u64(1574001),
        InlineBoxKind::InFlow,
        0,
        Size::new(6.0, 10.0),
    ));
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("No registered face"),
                ..Style::default()
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
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            anchor_order: vec![
                crate::ParagraphAnchor {
                    item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                    affinity: Affinity::Before,
                },
                crate::ParagraphAnchor {
                    item: crate::ParagraphAnchorItem::Atomic(0),
                    affinity: Affinity::After,
                },
            ],
            boundary_policies: [
                crate::ParagraphBoundaryDefinition::AtomicBefore(0),
                crate::ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| crate::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    assert_eq!(system.registered_font_faces().count(), 0);
    let p = system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap();
    let atomic = p.atomic_views().next().unwrap().handle();
    assert_eq!(atomic.view().original_box().size(), Size::new(6.0, 10.0));
    let mut r = request(&p);
    assert!(r.line_strut.is_none() && r.line_box_struts.is_empty() && r.controls.is_empty());
    r.atomics = vec![AtomicAllocation {
        occurrence: atomic.clone(),
        allocation: AtomicLineAllocation::InFlow {
            flow: AtomicFlowAllocation {
                advance: 6.0,
                before_baseline: 20.0,
                after_baseline: -10.0,
                occupied_start: 0.0,
                occupied_end: 6.0,
            },
            box_geometry: AtomicBoxAllocation {
                border: Rect::new(0.0, -20.0, 6.0, 10.0),
                content: Rect::new(0.0, -20.0, 6.0, 10.0),
                baseline: 0.0,
            },
            alignment: LineBoxAlignment::Baseline,
        },
    }];
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    // The actual box occupies [-20,-10] relative to its baseline: extent10,
    // wholly on one side. No glyph, font strut, root strut or zero is donated.
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (20.0, 10.0)
    );
    assert_eq!(line.feedback().advance, 6.0);
    assert_eq!(line.run_views().count(), 0);
    let slot = line.atomic_slots().next().unwrap();
    assert_eq!(slot.occurrence, atomic);
    assert_eq!(slot.origin, Point::new(0.0, 20.0));
}
