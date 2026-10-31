//! Clockwise glyphs compose with actual unrotated physical atomic rectangles.
//! Local inline/line-under box allocations are distinct from physical boxes.
use super::*;
mod supplemental;
use crate::{
    InlineBox, InlineBoxKind, Layout, ParagraphAnchor, ParagraphAnchorItem,
    ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const INFLOW: Id = Id::from_u64(1_556_001);
const POSITIONED: Id = Id::from_u64(1_556_002);

fn mixed_input() -> ParagraphInput {
    let mut src = Source::identified("A", Id::from_u64(1_556_000), 1);
    src.inline_box(InlineBox::new(
        INFLOW,
        InlineBoxKind::InFlow,
        1,
        Size::new(6.0, 14.0),
    ));
    src.inline_box(InlineBox::new(
        POSITIONED,
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(60.0, 140.0),
    ));
    let mut value = input(src, own_style(), false);
    // These are actual same-seam source occurrences. Their explicit semantic
    // order is positioned1 then in-flow0, rather than source-box storage order.
    value.anchor_order = [1, 0]
        .into_iter()
        .map(|index| ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(index),
            affinity: Affinity::Before,
        })
        .collect();
    value.boundary_policies = (0..2)
        .flat_map(|index| {
            [
                ParagraphBoundaryDefinition::AtomicBefore(index),
                ParagraphBoundaryDefinition::AtomicAfter(index),
            ]
        })
        .map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    value
}

fn allocation() -> AtomicLineAllocation {
    AtomicLineAllocation::InFlow {
        flow: AtomicFlowAllocation {
            advance: 14.0,
            before_baseline: 2.0,
            after_baseline: 4.0,
            occupied_start: 0.0,
            occupied_end: 14.0,
        },
        box_geometry: AtomicBoxAllocation {
            border: Rect::new(0.0, -2.0, 14.0, 6.0),
            content: Rect::new(2.0, -1.0, 10.0, 4.0),
            baseline: 0.0,
        },
        alignment: LineBoxAlignment::Baseline,
    }
}

fn finish(prepared: &PreparedText, atomics: Vec<AtomicAllocation>) -> FinalizedLine {
    let mut value = request(prepared);
    value.context.orientation = prepared.orientation();
    value.context.available_inline = None;
    value.context.alignment = crate::Alignment::Start;
    value.context.spacing = SpacingPolicy::None;
    value.atomics = atomics;
    prepared
        .select(value)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

#[test]
fn clockwise_mixed_atomics_keep_unrotated_physical_boxes_and_horizontal_font_metrics() {
    let font = skrifa::FontRef::new(AHEM).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    for tag in [b"BASE", b"vhea", b"vmtx", b"fvar"] {
        assert!(font.table_data(skrifa::Tag::new(tag)).is_none());
    }
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(35)), Some(1000.0));
    let bounds = metrics.bounds(GlyphId::new(35)).unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -200.0, 1000.0, 800.0]
    );
    let mut system = system_with(AHEM.to_vec());
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), AHEM);
    assert_eq!(face.index(), 0);
    assert_eq!(
        face.unsynthesized_instance(10.0, &[]).unwrap().axis_count(),
        0
    );
    // Horizontal Ahem A contributes actual8/2 at size10. No-atomic controls
    // qualify both real native profiles before the expected atomic admission.
    for orientation in [H, CW] {
        let plain = capture(&mut system, "A", own_style(), false, orientation);
        let line = finish(&plain, vec![]);
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (10.0, 8.0, 10.0)
        );
        assert_eq!(
            line.run_views()
                .flat_map(|run| run
                    .glyphs()
                    .map(|g| (g.id(), g.x(), g.y(), g.advance()))
                    .collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [(35, 0.0, 0.0, 10.0)]
        );
    }
    let analysis = analyze(mixed_input());
    assert_eq!(
        analysis.typography(),
        crate::AnalysisTypography::HorizontalComposition
    );
    assert_eq!(analysis.direction(), crate::Direction::LeftToRight);
    let horizontal = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    let order = |prepared: &PreparedText| {
        prepared
            .participants()
            .filter_map(|participant| match participant {
                PreparedParticipant::Atomic(atomic) => Some((
                    atomic.view().original_box().id(),
                    atomic.view().original_box().kind(),
                    atomic.view().source_position().get(),
                )),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        order(&horizontal),
        [
            (POSITIONED, InlineBoxKind::OutOfFlow, 1),
            (INFLOW, InlineBoxKind::InFlow, 1)
        ]
    );
    let prepared = system.prepare(&analysis, PreparationOptions::default().with_orientation(CW))
        .expect("qualified HorizontalComposition must admit native clockwise in-flow and positioned atomics");
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(order(&prepared), order(&horizontal));
    let atoms = prepared
        .atomic_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let inflow = atoms
        .iter()
        .find(|atomic| atomic.view().original_box().kind() == InlineBoxKind::InFlow)
        .unwrap()
        .clone();
    let positioned = atoms
        .iter()
        .find(|atomic| atomic.view().original_box().kind() == InlineBoxKind::OutOfFlow)
        .unwrap()
        .clone();
    let line = finish(
        &prepared,
        vec![
            AtomicAllocation {
                occurrence: inflow.clone(),
                allocation: allocation(),
            },
            AtomicAllocation {
                occurrence: positioned.clone(),
                allocation: AtomicLineAllocation::OutOfFlow,
            },
        ],
    );
    // Baseline-group maxima: before=max(font8,child2)=8,
    // after=max(font2,child4)=4. Thus H12, while advance is10+14+0=24.
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (24.0, 8.0, 12.0)
    );
    let slots = line.atomic_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    for (atomic, expected) in [
        (&inflow, allocation()),
        (&positioned, AtomicLineAllocation::OutOfFlow),
    ] {
        let slot = slots
            .iter()
            .find(|slot| &slot.occurrence == atomic)
            .unwrap();
        assert_eq!(slot.origin, Point::new(10.0, 8.0));
        assert_eq!(slot.allocation, expected);
    }
    let origin = prepared.origin_views().next().unwrap().handle();
    let fragments = line.fragment_views().collect::<Vec<_>>();
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0].origin(), Some(origin));
    assert_eq!(fragments[0].aggregate_range(), Some(Range::new(0, 1)));
    assert_eq!(fragments[0].bounds(), Some(Rect::new(0.0, 0.0, 10.0, 10.0)));
    let physical = [
        AtomicPlacement {
            occurrence: inflow.clone(),
            border: Rect::new(96.0, 30.0, 6.0, 14.0),
            content: Rect::new(97.0, 32.0, 4.0, 10.0),
            baseline: 100.0,
        },
        AtomicPlacement {
            occurrence: positioned.clone(),
            border: Rect::new(300.0, 500.0, 60.0, 140.0),
            content: Rect::new(305.0, 507.0, 48.0, 126.0),
            baseline: 321.0,
        },
    ];
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: CW,
            atomics: physical.to_vec(),
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), analysis.source());
    assert_eq!(layout.orientation(), CW);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(96.0, 20.0, 12.0, 24.0)
    );
    let boxes = layout.inline_boxes();
    for (id, rect) in [
        (INFLOW, physical[0].border),
        (POSITIONED, physical[1].border),
    ] {
        let actual = boxes.iter().find(|item| item.id() == id).unwrap();
        assert_eq!(actual.rect(), rect);
    }
    assert_eq!(layout.hit(Point::new(98.0, 35.0)), Hit::InlineBox(INFLOW));
    assert_eq!(
        layout.hit(Point::new(310.0, 510.0)),
        Hit::InlineBox(POSITIONED)
    );
    for run in layout.run_views() {
        assert_eq!(run.glyph_basis(), crate::GlyphBasis::Clockwise);
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.origin(), Point::new(100.0, 20.0));
        assert_eq!(run.range(), Range::new(0, 1));
        assert_eq!(
            run.glyphs()
                .map(|g| (g.id(), g.x(), g.y(), g.advance(), g.pen()))
                .collect::<Vec<_>>(),
            [(35, 0.0, 0.0, 10.0, 0.0)]
        );
    }
    drop(system);
    assert_eq!(line.prepared_text(), prepared);
}
