//! One accepted OutOfFlow occurrence retains true normal/visual static ports.
//! Source setup follows the maintained Native constructor; only the actual
//! authored box role differs. No post-capture mutation or fabricated handle.
use super::*;

fn outflow_native(
    raw: &str,
    font_size: f32,
    line_height: f64,
    mode: WritingMode,
    boxes: &[(u32, f64, f64)],
) -> Native {
    let original = Source::new(raw);
    let mut source = original.clone();
    for &(node, inline, cross) in boxes {
        let physical = size::<f64>(mode, inline, cross);
        source.inline_box(text::InlineBox::new(
            text::Id::from_u64(u64::from(node)),
            text::InlineBoxKind::OutOfFlow,
            original.text().len(),
            text::Size::new(physical.width, physical.height),
        ));
    }
    let style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::None,
        ..ahem_style(font_size, line_height)
    };
    let extent = text::Range::new(0, original.text().len());
    let typography = if name(mode) == BaselineName::Central {
        text::AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::LeftToRight,
        }
    } else {
        text::AnalysisTypography::HorizontalComposition
    };
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source,
            base_style: style,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: original.clone(),
                local: extent,
                aggregate: extent,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: (0..boxes.len())
                .flat_map(|index| {
                    [
                        text::ParagraphBoundaryDefinition::AtomicBefore(index),
                        text::ParagraphBoundaryDefinition::AtomicAfter(index),
                    ]
                })
                .map(|definition| text::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: text::Wrap::None,
                    following_wrap: text::Wrap::None,
                    ancestor_wrap: text::Wrap::None,
                })
                .collect(),
            anchor_order: (0..boxes.len())
                .map(|index| text::ParagraphAnchor {
                    item: text::ParagraphAnchorItem::Atomic(index),
                    affinity: text::Affinity::Before,
                })
                .collect(),
        },
        typography,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::None);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let orientation = match mode {
        WritingMode::HorizontalTb => text::LineOrientation::Horizontal,
        WritingMode::SidewaysRl => text::LineOrientation::SidewaysClockwise,
        WritingMode::VerticalRl | WritingMode::VerticalLr => text::LineOrientation::Vertical,
        WritingMode::SidewaysLr => text::LineOrientation::SidewaysCounterclockwise,
    };
    let mut options = text::PreparationOptions::default().with_orientation(orientation);
    if name(mode) == BaselineName::Alphabetic {
        options = options.with_baseline_reference(text::FontBaseline::Alphabetic);
    }
    Native {
        prepared: system.prepare(&analyzed, options).unwrap(),
        original,
    }
}

fn normal<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> crate::output::NormalGeometryOf<S> {
    let entry = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap();
    match entry.geometry_state() {
        crate::output::GeometryStateOf::OrdinaryCanonical { normal }
        | crate::output::GeometryStateOf::CanonicalAtomic {
            normal,
            readback: crate::output::AtomicReadbackPhase::Finalized,
            ..
        } => normal,
        phase => panic!("final output retained a noncanonical phase: {phase:?}"),
    }
}

fn static_ports<S: LayoutScalar>() {
    premises();
    let mode = WritingMode::VerticalRl;
    let outer = native("X", 10.0, 10.0, mode, &[(2, 20.0, 10.4)]);
    let inner = outflow_native("X", 10.0, 10.0, mode, &[(4, 2.0, 2.0)]);
    let descendant = native("X", 1.0, 1.0, mode, &[]);
    let outer_box = outer.prepared.atomic_views().next().unwrap();
    assert_eq!(outer_box.original_box().kind(), text::InlineBoxKind::InFlow);
    assert_eq!(outer_box.original_box().size(), text::Size::new(10.4, 20.0));
    assert_eq!(
        inner.prepared.baseline_reference(),
        Some(text::FontBaseline::Central)
    );
    let outflow = inner.prepared.atomic_views().next().unwrap();
    assert_eq!(
        outflow.original_box().kind(),
        text::InlineBoxKind::OutOfFlow
    );
    assert_eq!(outflow.original_box().size(), text::Size::new(2.0, 2.0));
    assert_eq!(outflow.original_box().id(), text::Id::from_u64(4));
    let mut prior: Option<(crate::output::NormalGeometryOf<S>, Vec<text::Point>)> = None;
    for (left, visual_width, visual_x) in [(0.0, 10.0, 8.0), (0.32, 11.0, 9.0)] {
        let mut parent = child(mode, 10.4, BaselineSource::Auto, VerticalAlignOf::Top);
        parent.position = Position::Relative;
        parent.inset.left = LengthAutoOf::px(n(left));
        let absolute = NodeInputOf {
            display: Display::Block,
            position: Position::Absolute,
            writing_mode: mode,
            inline_baseline_context: InlineBaselineContext::Named(BaselineName::Central),
            size: Size::new(px(2.0), px(2.0)),
            ..NodeInputOf::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [3, 4])
            .children(3, [])
            .children(4, [5])
            .children(5, [])
            .style(0, root(mode))
            .input(1, outer.input())
            .style(2, parent)
            .input(3, inner.input())
            .style(4, absolute)
            .input(5, descendant.input());
        let batch = solve(
            &tree,
            mode,
            vec![
                outer.binding(0, 1),
                inner.binding(2, 3),
                descendant.binding(4, 5),
            ],
        )
        .unwrap();
        let parent_entry = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap();
        let crate::output::GeometryStateOf::CanonicalAtomic {
            measured_normal,
            normal: parent_normal,
            ..
        } = parent_entry.geometry_state()
        else {
            panic!("the real captured in-flow parent must retain canonical atomic custody")
        };
        // Canonical Top flow retains raw normal corners0..10.4. Left+.32
        // gives.32..10.72 ->0..11, without changing normal10 or slot-before10.
        assert_eq!(measured_normal.size.width, n::<S>(10.4));
        assert_eq!(parent_normal.size.width, n::<S>(10.0));
        assert_eq!(parent_entry.output().size.width, n::<S>(visual_width));
        // Actual Vrl flow block-start is the parent's right edge. The 2px
        // absolute border ends there: normal10-2=8; visual10/11-2=8/9.
        assert_eq!(normal(&batch, 4).location, Point::new(n(8.0), n(10.0)));
        let actual = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 4)
            .unwrap()
            .output();
        assert_eq!(actual.location, Point::new(n(visual_x), n(10.0)));
        assert_eq!(actual.size, Size::new(n(2.0), n(2.0)));
        let parent_text = placed(&batch, 2);
        assert_eq!(parent_text.source().text().code_units(), &[0x58]);
        let actual_box = &parent_text.inline_boxes()[0];
        assert_eq!(actual_box.kind(), text::InlineBoxKind::OutOfFlow);
        assert_eq!(actual_box.rect(), text::Rect::new(visual_x, 20.0, 2.0, 2.0));
        let line = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|row| row.owner() == &crate::InlineFragmentOwner::Node(3))
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(
            line.atomic_slots().next().unwrap().occurrence,
            outflow.handle()
        );
        assert_eq!(
            line.atomic_slots().next().unwrap().allocation,
            text::AtomicLineAllocation::OutOfFlow
        );
        // Ahem10 upright metrics use the required Central reference:
        // advance10 and equal before/after5. The positioned occurrence adds
        // neither advance nor line metrics.
        assert_eq!(
            (
                line.feedback().advance,
                line.feedback().baseline,
                line.feedback().line_height
            ),
            (10.0, 5.0, 10.0)
        );
        let child_text = placed(&batch, 4);
        assert_eq!(child_text.source().text().code_units(), &[0x58]);
        let origins = child_text
            .run_views()
            .map(|run| run.origin())
            .collect::<Vec<_>>();
        assert!(!origins.is_empty());
        let normal_child = normal(&batch, 4);
        if let Some((before_normal, before_origins)) = prior.as_ref() {
            assert_eq!(&normal_child, before_normal);
            assert_eq!(origins.len(), before_origins.len());
            for (before, after) in before_origins.iter().zip(&origins) {
                assert_eq!(after.x - before.x, 1.0);
                assert_eq!(after.y, before.y);
            }
        }
        prior = Some((normal_child, origins));
    }
}

#[test]
fn real_outflow_static_ports_keep_normal_end_affinity_f32() {
    static_ports::<f32>();
}
#[test]
fn real_outflow_static_ports_keep_normal_end_affinity_f64() {
    static_ports::<f64>();
}
