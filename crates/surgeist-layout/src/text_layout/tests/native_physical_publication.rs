//! Existing Baseline consumers must publish the actual native physical point.
//! These cases use no resolved-offset API and can run on the pre-offset source.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const WIDTH: f64 = 100_000_000.0;
const SPACER: f64 = WIDTH - 16.0;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn px<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::px(n(value))
}
fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn table(tag: [u8; 4]) -> Option<&'static [u8]> {
    (0..usize::from(u16_at(AHEM, 4))).find_map(|index| {
        let at = 12 + index * 16;
        if AHEM[at..at + 4] != tag {
            return None;
        }
        let offset = usize::try_from(u32::from_be_bytes(
            AHEM[at + 8..at + 12].try_into().unwrap(),
        ))
        .unwrap();
        let length = usize::try_from(u32::from_be_bytes(
            AHEM[at + 12..at + 16].try_into().unwrap(),
        ))
        .unwrap();
        Some(&AHEM[offset..offset + length])
    })
}
fn premises() {
    assert_eq!(u16_at(table(*b"head").unwrap(), 18), 1000);
    assert_eq!(u16_at(table(*b"hmtx").unwrap(), 58 * 4), 1000);
    for tag in [*b"BASE", *b"vhea", *b"vmtx"] {
        assert!(
            table(tag).is_none(),
            "the retained Ahem fixture has no {tag:?}"
        );
    }
    // The native absent-vertical-metrics policy uses a centered one-em
    // interval. Size10/LH10 therefore supplies5/5, independently of hhea A/D.
    assert_eq!(WIDTH as f32 as f64, WIDTH);
    assert_eq!(SPACER as f32 as f64, SPACER);
}

struct Paragraph {
    prepared: text::PreparedText,
    original: Source,
    binding: TextParagraphBinding<u32>,
}
fn paragraph(control: bool) -> Paragraph {
    let original = Source::new("X");
    let mut source = original.clone();
    if !control {
        source.inline_box(text::InlineBox::new(
            text::Id::from_u64(2),
            text::InlineBoxKind::InFlow,
            1,
            text::Size::new(4.0, 20.0),
        ));
    }
    let style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: text::Wrap::None,
        ..ahem_style(10.0, 10.0)
    };
    let range = text::Range::new(0, 1);
    let input = text::ParagraphInput {
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
            local: range,
            aggregate: range,
            kind: text::ParagraphOriginKind::Authored,
        }],
        controls: if control {
            vec![text::ParagraphControl {
                at: 1,
                kind: text::ParagraphControlKind::ForcedBreak,
            }]
        } else {
            vec![]
        },
        connections: vec![],
        replacements: vec![],
        boundary_policies: if control {
            vec![]
        } else {
            [
                text::ParagraphBoundaryDefinition::AtomicBefore(0),
                text::ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| text::ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: text::Wrap::None,
                following_wrap: text::Wrap::None,
                ancestor_wrap: text::Wrap::None,
            })
            .collect()
        },
        anchor_order: vec![text::ParagraphAnchor {
            item: if control {
                text::ParagraphAnchorItem::Control(0)
            } else {
                text::ParagraphAnchorItem::Atomic(0)
            },
            affinity: text::Affinity::Before,
        }],
    };
    let analyzed = text::AnalyzedSource::from_paragraph(
        input,
        text::AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::LeftToRight,
        },
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::None);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(text::LineOrientation::Vertical),
        )
        .unwrap();
    let binding = TextParagraphBinding {
        prepared: prepared.clone(),
        container: 0,
        nodes: vec![1, 2],
        origins: vec![TextOriginBinding {
            node: 1,
            origin: prepared.origin_views().next().unwrap().handle(),
        }],
        atomics: prepared
            .atomic_views()
            .map(|view| TextAtomicBinding {
                node: 2,
                occurrence: view.handle(),
            })
            .collect(),
        controls: prepared
            .control_views()
            .map(|view| TextControlBinding {
                node: 2,
                occurrence: view.handle(),
            })
            .collect(),
        line_strut: None,
    };
    Paragraph {
        prepared,
        original,
        binding,
    }
}

fn tree<S: LayoutScalar>(paragraph: &Paragraph, control: bool) -> PublicLayoutTreeOf<S> {
    let named = InlineBaselineContext::Named(BaselineName::Central);
    let tree = PublicLayoutTreeOf::new()
        .children(0, [9, 1, 2])
        .children(9, [])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                inline_baseline_context: named,
                size: Size::new(px(WIDTH), px(100.0)),
                ..NodeInputOf::default()
            },
        )
        .style(
            9,
            NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                inline_baseline_context: named,
                size: Size::new(px(SPACER), px(100.0)),
                ..NodeInputOf::default()
            },
        )
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(
                paragraph.binding.origins[0].origin.clone(),
            )),
        );
    if control {
        let metrics = InlineMetricsOf::try_new(named, n(2.0), n(4.0)).unwrap();
        tree.input(
            2,
            LayoutInputOf::line_break(
                LineBreakInputOf::new()
                    .with_writing_mode(WritingMode::VerticalRl)
                    .with_metrics(metrics),
            ),
        )
    } else {
        // Genuine empty layout establishes absence; no measured output or
        // baseline facts are injected. Default alignment is existing Baseline.
        tree.style(
            2,
            NodeInputOf::<S> {
                display: Display::InlineBlock,
                writing_mode: WritingMode::VerticalRl,
                inline_baseline_context: named,
                size: Size::new(px(4.0), px(20.0)),
                ..NodeInputOf::default()
            },
        )
    }
}

fn qualify<S: LayoutScalar>(control: bool) {
    premises();
    let paragraph = paragraph(control);
    let tree = tree::<S>(&paragraph, control);
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding.clone()],
        crate::WrappingInput::default(),
    )
    .unwrap();
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(n(WIDTH)),
        AvailableOf::definite(n(100.0)),
    ))
    .unwrap();
    let batch = compute_layout_with_flow(&flow, 0, request).unwrap();
    let layout = batch
        .placed_text()
        .iter()
        .find(|entry| entry.container() == 0)
        .unwrap()
        .layout();
    assert_eq!(layout.source().text(), paragraph.original.text());
    let original = paragraph.prepared.origin_views().next().unwrap();
    assert_eq!(original.source(), &paragraph.original);
    assert_eq!(original.local_range(), text::Range::new(0, 1));
    assert_eq!(original.aggregate_range(), text::Range::new(0, 1));
    assert_eq!(layout.orientation(), text::LineOrientation::Vertical);
    assert_eq!(layout.lines().len(), 1);
    // Physical right edge after the actual block spacer is C-(C-16)=16.
    // The real centered10px strut sets Central16-5=11. Child Central is2,
    // synthesized only after actual absence; its physical left is11-2=9.
    assert_eq!(layout.lines()[0].baseline(), 11.0);
    let source_fragment = batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| row.owner() == &InlineFragmentOwner::Node(1))
        .unwrap();
    let line = source_fragment
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(line.prepared_text(), paragraph.prepared);
    assert_eq!(
        (line.feedback().baseline, line.feedback().line_height),
        (5.0, 10.0)
    );
    let mut glyphs = 0;
    for run in layout.run_views() {
        assert_eq!(
            (run.face().bytes(), run.face().index(), run.font_size()),
            (AHEM, 0, 10.0)
        );
        for glyph in run.glyphs() {
            assert_eq!((glyph.id(), glyph.advance()), (58, 10.0));
            glyphs += 1;
        }
    }
    assert_eq!(glyphs, 1);
    if control {
        let strut = line.selected_line().request().controls[0].strut.unwrap();
        assert_eq!((strut.baseline, strut.line_extent), (2.0, 4.0));
        assert_eq!(line.control_slots().next().unwrap().slot().y, 5.0);
    } else {
        assert_eq!(
            layout.inline_boxes()[0].rect(),
            text::Rect::new(9.0, 10.0, 4.0, 20.0)
        );
        let output = batch
            .unrounded_entries()
            .iter()
            .find(|row| row.node() == 2)
            .unwrap()
            .output();
        let pair = output.named_baselines.get(BaselineName::Central);
        assert_eq!(
            (pair.first(), pair.last()),
            (BaselineFactOf::Absent, BaselineFactOf::Absent)
        );
    }
    // Assert an independent physical result, including the unrounded consumer.
    // Reconstructing it through a large S logical block coordinate loses it.
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = entries
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert_eq!(output.location.x, n(if control { 11.0 } else { 9.0 }));
        assert_eq!(output.location.y, n(10.0));
        // The parent's actual first/last named reference is the same physical
        // line anchor11; publishing it must not reconstruct a large S logical
        // distance and thereby replace that retained reference with8.
        let parent = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        let pair = parent.named_baselines.get(BaselineName::Central);
        let expected = BaselineFactOf::Available(
            BaselineCoordinateOf::try_new(PhysicalAxis::Horizontal, n(11.0)).unwrap(),
        );
        assert_eq!(pair.first(), expected);
        assert_eq!(pair.last(), expected);
    }
}

#[test]
fn f32_native_atomic_keeps_actual_physical_left_after_large_vertical_spacer() {
    qualify::<f32>(false);
}
#[test]
fn f64_native_atomic_keeps_actual_physical_left_after_large_vertical_spacer() {
    qualify::<f64>(false);
}
#[test]
fn f32_native_control_keeps_actual_physical_central_after_large_vertical_spacer() {
    qualify::<f32>(true);
}
#[test]
fn f64_native_control_keeps_actual_physical_central_after_large_vertical_spacer() {
    qualify::<f64>(true);
}
