//! Source-less wbr occurrences through native Layout and retained paint.

use super::*;

struct WbrTree {
    inputs: Vec<layout::node::LayoutInput>,
    non_box: layout::node::NodeInput,
}
impl layout::tree::Traverse for WbrTree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::ops::Range<usize>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 {
            1..self.inputs.len()
        } else {
            0..0
        }
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { self.inputs.len() - 1 } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}
impl layout::tree::LayoutTree for WbrTree {
    type MeasureError = Infallible;
    fn node_input(&self, node: usize) -> &layout::node::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::node::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn capture(
    system: &mut text::System,
    shared_mark: bool,
    replacements: &[text::style::WordSpaceTransform],
    authored_width: Option<f32>,
    request: layout::request::LayoutRootRequest,
) -> (
    layout::batch::CompletedLayoutBatch<usize>,
    Vec<text::prepared::PreparedOrigin>,
    Vec<text::prepared::PreparedControl>,
    text::source::Source,
) {
    let source = text::source::Source::identified(
        if shared_mark { "A\u{3099}" } else { "" },
        text::source::Id::from_u64(6308),
        31,
    );
    let style = text::style::Style {
        font: text::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::style::LineHeight::Absolute(10.0),
        white_space: text::style::WhiteSpace::Discard,
        ..text::style::Style::default()
    };
    let at = usize::from(shared_mark);
    let origins = if shared_mark {
        (0..2)
            .map(|index| text::paragraph::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::source::Range::new(index, index + 1),
                aggregate: text::source::Range::new(index, index + 1),
                kind: text::paragraph::ParagraphOriginKind::Authored,
            })
            .collect()
    } else {
        vec![]
    };
    let analyzed = text::analyzed::AnalyzedSource::from_paragraph(
        text::paragraph::ParagraphInput {
            source: source.clone(),
            base_style: style.clone(),
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
            terminal_edge: text::paragraph::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins,
            connections: if shared_mark {
                vec![text::paragraph::ParagraphConnection {
                    at: 1,
                    kind: text::paragraph::ParagraphConnectionKind::Continuous,
                }]
            } else {
                vec![]
            },
            boundary_policies: if shared_mark {
                vec![text::paragraph::ParagraphBoundaryPolicy {
                    definition: text::paragraph::ParagraphBoundaryDefinition::Connection(0),
                    preceding_wrap: style.wrap,
                    following_wrap: style.wrap,
                    ancestor_wrap: style.wrap,
                }]
            } else {
                vec![]
            },
            controls: replacements
                .iter()
                .map(|&replacement| text::paragraph::ParagraphControl {
                    at,
                    kind: text::paragraph::ParagraphControlKind::Wbr {
                        style: Box::new(text::style::Style {
                            word_space_transform: replacement,
                            ..style.clone()
                        }),
                    },
                })
                .collect(),
            anchor_order: (0..replacements.len())
                .map(|index| text::paragraph::ParagraphAnchor {
                    item: text::paragraph::ParagraphAnchorItem::Control(index),
                    affinity: text::layout::Affinity::After,
                })
                .collect(),
            replacements: vec![],
        },
        text::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.processed_source().processed_text(), source.text());
    let policies = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| text::analyzed::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: style.wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::prepared::PreparationOptions::default())
        .unwrap();
    let origins = prepared
        .origin_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    let controls = prepared
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), replacements.len());
    let mut inputs = vec![layout::node::LayoutInput::box_input(
        layout::node::NodeInput {
            display: layout::box_model::Display::Block,
            size: layout::geometry::Size::new(
                match authored_width {
                    Some(width) => layout::sizing::PreferredSizeOf::value(
                        layout::length::LengthPercentageOf::px(width).unwrap(),
                    ),
                    None => layout::sizing::PreferredSizeOf::AUTO,
                },
                layout::sizing::PreferredSizeOf::AUTO,
            ),
            ..layout::node::NodeInput::DEFAULT
        },
    )];
    let mut origin_bindings = vec![];
    let mut control_bindings = vec![];
    if shared_mark {
        origin_bindings.push(layout::text_layout::TextOriginBinding {
            origin: origins[0].clone(),
            node: inputs.len(),
        });
        inputs.push(layout::node::LayoutInput::InlineText(
            layout::text_layout::TextParticipation::new(origins[0].clone()),
        ));
    }
    for control in &controls {
        control_bindings.push(layout::text_layout::TextControlBinding {
            occurrence: control.clone(),
            node: inputs.len(),
        });
        inputs.push(layout::node::LayoutInput::TextControl(
            layout::text_layout::TextControlParticipation::new(control.clone()),
        ));
    }
    if shared_mark {
        origin_bindings.push(layout::text_layout::TextOriginBinding {
            origin: origins[1].clone(),
            node: inputs.len(),
        });
        inputs.push(layout::node::LayoutInput::InlineText(
            layout::text_layout::TextParticipation::new(origins[1].clone()),
        ));
    }
    let nodes = (1..inputs.len()).collect();
    let tree = WbrTree {
        inputs,
        non_box: layout::node::NodeInput::non_box(),
    };
    let captured = layout::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::text_layout::TextParagraphBinding {
            inline_box_fragments: layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: 0,
            nodes,
            origins: origin_bindings,
            controls: control_bindings,
            atomics: vec![],
            line_strut: None,
        }],
        surgeist_layout::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(&captured, 0, request).unwrap();
    (batch, origins, controls, source)
}
fn system(bytes: &[u8]) -> text::System {
    let mut system = text::System::default();
    system
        .register_fonts(bytes.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    system
}
fn glyphs(placed: &text::layout::Layout, bytes: &[u8]) -> Vec<(u32, f32)> {
    placed
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}
fn line(batch: &layout::batch::CompletedLayoutBatch<usize>) -> text::finalized::FinalizedLine {
    batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line()
}
fn at(placed: &text::layout::Layout, x: f64) -> text::layout::PlacedPosition {
    placed
        .positions()
        .find(|position| {
            placed.position_geometry(position).unwrap().rect().origin
                == text::geometry::Point::new(x, 0.0)
        })
        .unwrap()
}
fn paint(placed: &text::layout::Layout) -> render::text_layout::PreparedTextScene<'_> {
    render::text_layout::prepare_text_layout(
        placed,
        render::geometry::Transform::identity(),
        render::text_layout::TextLayoutLimits::new(
            render::text::GlyphRunLimits::new(64, 4096, 8 << 20, 8 << 20),
            16,
            64,
            8,
        ),
        &mut surgeist_render::accounting::RenderAccountingContext::default(),
    )
    .unwrap()
}
fn unit_rect(unit: &text::range_fragments::SourceRangeUnit<'_>) -> text::geometry::Rect {
    let pieces = unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>();
    assert!(!pieces.is_empty());
    let left = pieces
        .iter()
        .map(|rect| rect.origin.x)
        .fold(f64::INFINITY, f64::min);
    let right = pieces
        .iter()
        .map(|rect| rect.origin.x + rect.size.width)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        pieces
            .iter()
            .all(|rect| rect.origin.y == 0.0 && rect.size.height == 10.0)
    );
    text::geometry::Rect::new(left, 0.0, right - left, 10.0)
}

#[test]
fn wbr_inserts_before_a_real_mark_and_retains_both_original_and_control_custody() {
    let bytes = super::width_transform::width_ahem();
    for (replacement, expected, total) in [
        (
            text::style::WordSpaceTransform::None,
            vec![(35, 10.0), (100, 0.0)],
            10.0,
        ),
        (
            text::style::WordSpaceTransform::Space,
            vec![(35, 10.0), (3, 5.0), (100, 0.0)],
            15.0,
        ),
        (
            text::style::WordSpaceTransform::IdeographicSpace,
            vec![(35, 10.0), (1, 20.0), (100, 0.0)],
            30.0,
        ),
    ] {
        let mut system = system(&bytes);
        let (batch, origins, controls, source) = capture(
            &mut system,
            true,
            &[replacement],
            Some(100.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(100.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        assert_eq!(source.text().code_units(), [0x41, 0x3099]);
        assert_eq!(placed.source(), &source);
        // None independently qualifies the added3099→100 zero-advance fixture
        // through the real registered provider, rather than trusting helper code.
        assert_eq!(glyphs(placed, &bytes), expected);
        assert_eq!(line(&batch).feedback().advance, total);
        if replacement != text::style::WordSpaceTransform::None {
            let mark = placed
                .source_range_fragments(&origins[1], text::source::Range::new(1, 2))
                .unwrap();
            assert_eq!(mark.units().count(), 1);
            let unit = mark.units().next().unwrap();
            assert_eq!(unit.controls().collect::<Vec<_>>(), controls);
            assert_eq!(
                unit_rect(&unit),
                text::geometry::Rect::new(10.0, 0.0, total - 10.0, 10.0)
            );
            assert!(unit.fragments().any(|fragment|matches!(fragment.view().owner(),text::finalized::FinalizedFragmentOwner::Control(control) if control==controls[0])));
            let a = placed
                .source_range_fragments(&origins[0], text::source::Range::new(0, 1))
                .unwrap();
            assert_eq!(a.units().count(), 1);
            assert_eq!(a.units().next().unwrap().controls().count(), 0);
            assert_eq!(
                unit_rect(&a.units().next().unwrap()),
                text::geometry::Rect::new(0.0, 0.0, 10.0, 10.0)
            );
            let finalized = line(&batch);
            let generated = finalized
                .mapping_views()
                .filter(|mapping| {
                    mapping.kind() == text::finalized::PresentationMappingKind::GeneratedSeparator
                })
                .collect::<Vec<_>>();
            assert_eq!(generated.len(), 1);
            assert_eq!(generated[0].source_range(), text::source::Range::new(1, 1));
            assert_eq!(generated[0].processed_range().start().get(), 1);
            assert_eq!(generated[0].processed_range().end().get(), 1);
            assert_eq!(generated[0].separator_control(), Some(controls[0].clone()));
        }
        drop(system);
        let paint = paint(placed);
        assert!(std::ptr::eq(paint.layout(), placed));
        assert!(!paint.scene().is_empty());
        assert_eq!(paint.layout().source(), &source);
    }
}

#[test]
fn equal_anchor_wbr_keep_distinct_positions_and_only_the_second_owns_the_shared_mark_unit() {
    let bytes = super::width_transform::width_ahem();
    let mut system = system(&bytes);
    let (batch, origins, controls, source) = capture(
        &mut system,
        true,
        &[
            text::style::WordSpaceTransform::Space,
            text::style::WordSpaceTransform::IdeographicSpace,
        ],
        Some(100.0),
        NativeViewport::new(
            layout::geometry::Size::new(Some(100.0), None),
            layout::geometry::FlowAxes::new(
                layout::geometry::WritingMode::HorizontalTb,
                layout::geometry::Direction::Ltr,
            ),
        )
        .request(layout::geometry::Size::new(
            layout::constraint::SizingOperation::Normal,
            layout::constraint::SizingOperation::Normal,
        )),
    );
    let placed = batch.placed_text()[0].layout();
    assert_eq!(
        glyphs(placed, &bytes),
        [(35, 10.0), (3, 5.0), (1, 20.0), (100, 0.0)]
    );
    assert_eq!(line(&batch).feedback().advance, 35.0);
    assert_ne!(controls[0], controls[1]);
    let mark = placed
        .source_range_fragments(&origins[1], text::source::Range::new(1, 2))
        .unwrap();
    let unit = mark.units().next().unwrap();
    assert_eq!(mark.units().count(), 1);
    assert_eq!(
        unit.controls().collect::<Vec<_>>().as_slice(),
        std::slice::from_ref(&controls[1])
    );
    assert_eq!(
        unit_rect(&unit),
        text::geometry::Rect::new(15.0, 0.0, 20.0, 10.0)
    );
    let first = placed
        .position_selection(&at(placed, 10.0), &at(placed, 15.0))
        .unwrap();
    assert!(!first.is_collapsed());
    assert!(first.source_text().is_empty());
    assert!(first.plaintext().is_empty());
    assert!(first.source_pieces().is_empty());
    assert_eq!(
        first.geometry().rects()[0].rect(),
        text::geometry::Rect::new(10.0, 0.0, 5.0, 10.0)
    );
    for (a, b) in [
        (&at(placed, 15.0), &at(placed, 35.0)),
        (&at(placed, 35.0), &at(placed, 15.0)),
    ] {
        let selected = placed.position_selection(a, b).unwrap();
        assert_eq!(selected.source_text().code_units(), [0x3099]);
        assert_eq!(selected.plaintext().code_units(), [0x3099]);
        assert_eq!(selected.source_pieces().len(), 1);
        assert_eq!(selected.source_pieces()[0].origin(), &origins[1]);
        assert_eq!(
            selected.source_pieces()[0].local_range(),
            text::source::Range::new(1, 2)
        );
        assert_eq!(
            selected.geometry().rects()[0].rect(),
            text::geometry::Rect::new(15.0, 0.0, 20.0, 10.0)
        );
    }
    let whole = placed
        .position_selection(&at(placed, 0.0), &at(placed, 35.0))
        .unwrap();
    assert_eq!(whole.source_text().code_units(), [0x41, 0x3099]);
    assert_eq!(whole.plaintext().code_units(), [0x41, 0x3099]);
    let generated = line(&batch)
        .mapping_views()
        .filter_map(|mapping| mapping.separator_control())
        .collect::<Vec<_>>();
    assert_eq!(generated, controls);
    drop(system);
    let retained = paint(placed);
    assert_eq!(retained.layout().source(), &source);
    assert!(!retained.scene().is_empty());
}

#[test]
fn source_less_only_wbr_has_real_fit_and_intrinsics_without_authored_text() {
    let bytes = super::width_transform::width_ahem();
    for (replacement, glyph, advance) in [
        (text::style::WordSpaceTransform::Space, 3, 5.0),
        (text::style::WordSpaceTransform::IdeographicSpace, 1, 20.0),
    ] {
        let mut system = system(&bytes);
        let (batch, origins, controls, source) = capture(
            &mut system,
            false,
            &[replacement],
            Some(4.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(4.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        assert!(origins.is_empty());
        assert!(source.text().is_empty());
        assert_eq!(glyphs(placed, &bytes), [(glyph, advance as f32)]);
        assert_eq!(line(&batch).measurements().fit().advance(), advance);
        let fragments = line(&batch).fragments().collect::<Vec<_>>();
        assert!(fragments.iter().any(|fragment|matches!(fragment.view().owner(),text::finalized::FinalizedFragmentOwner::Control(control) if control==controls[0])&&fragment.view().origin().is_none()&&fragment.view().bounds()==Some(text::geometry::Rect::new(0.0,0.0,advance,10.0))));
        let selected = placed
            .position_selection(&at(placed, 0.0), &at(placed, advance))
            .unwrap();
        assert!(!selected.is_collapsed());
        assert!(selected.source_pieces().is_empty());
        assert!(selected.source_text().is_empty());
        assert!(selected.plaintext().is_empty());
        assert_eq!(
            selected.geometry().rects()[0].rect(),
            text::geometry::Rect::new(0.0, 0.0, advance, 10.0)
        );
        for purpose in [
            layout::constraint::SizingOperation::MinContent,
            layout::constraint::SizingOperation::MaxContent,
        ] {
            let (intrinsic, _, _, _) = capture(
                &mut system,
                false,
                &[replacement],
                None,
                NativeViewport::new(
                    layout::geometry::Size::new(None, None),
                    layout::geometry::FlowAxes::new(
                        layout::geometry::WritingMode::HorizontalTb,
                        layout::geometry::Direction::Ltr,
                    ),
                )
                .request(layout::geometry::Size::new(
                    purpose,
                    layout::constraint::SizingOperation::Normal,
                )),
            );
            assert_eq!(
                intrinsic
                    .node_layout(0)
                    .unwrap()
                    .single_box()
                    .expect("fixture node produces one box")
                    .final_output()
                    .size
                    .width,
                advance as f32
            );
        }
        let actual = placed
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| {
                        render::text::TextGlyph::try_new(
                            glyph.id(),
                            glyph.pen() + f64::from(glyph.x()),
                            f64::from(glyph.y()),
                            glyph.advance(),
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let font = render::text::FontData::try_from_bytes(
            bytes.clone(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let instance =
            render::text::TextInstance::try_new(10.0, vec![], render::text::GlyphSynthesis::none())
                .unwrap();
        let blank = font
            .prepare_glyph_run(
                &instance,
                &actual,
                render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                &mut surgeist_render::accounting::RenderAccountingContext::default(),
            )
            .unwrap();
        assert_eq!(
            blank.bounds().kind(),
            render::text::TextRunBoundsKind::Empty
        );
        drop(system);
        let retained = paint(placed);
        assert!(std::ptr::eq(retained.layout(), placed));
        // A blank run still retains a real prepared command/resource. Scene
        // command emptiness is not a measurement of painted ink.
        assert!(!retained.scene().is_empty());
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn wbr_accessibility_keeps_only_actual_authored_a_and_mark_with_their_real_bounds() {
    let bytes = super::width_transform::width_ahem();
    for (replacements, width) in [
        (vec![text::style::WordSpaceTransform::Space], 15.0),
        (
            vec![
                text::style::WordSpaceTransform::Space,
                text::style::WordSpaceTransform::IdeographicSpace,
            ],
            35.0,
        ),
    ] {
        let mut system = system(&bytes);
        let (batch, _, _, source) = capture(
            &mut system,
            true,
            &replacements,
            Some(100.0),
            NativeViewport::new(
                layout::geometry::Size::new(Some(100.0), None),
                layout::geometry::FlowAxes::new(
                    layout::geometry::WritingMode::HorizontalTb,
                    layout::geometry::Direction::Ltr,
                ),
            )
            .request(layout::geometry::Size::new(
                layout::constraint::SizingOperation::Normal,
                layout::constraint::SizingOperation::Normal,
            )),
        );
        let placed = batch.placed_text()[0].layout();
        drop(system);
        let mut state = text::accessibility::Accessibility::new(20);
        let parent = 1000_u64.into();
        let source_update = placed
            .analyzed_source()
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        let values = source_update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<Vec<_>>();
        assert_eq!(values, ["A\u{3099}"]);
        assert!(
            source_update
                .nodes
                .iter()
                .filter(|(_, node)| node.value().is_some())
                .all(|(_, node)| node.bounds().is_none())
        );
        let update = placed
            .accessibility_update(
                &mut state,
                parent,
                parent,
                text::geometry::Point::new(100.0, 200.0),
            )
            .unwrap();
        let values = update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<Vec<_>>();
        assert_eq!(values, ["A\u{3099}"]);
        let authored = &update
            .nodes
            .iter()
            .find(|(_, node)| node.value() == Some("A\u{3099}"))
            .unwrap()
            .1;
        assert_eq!(authored.character_lengths(), [4]);
        let bounds = authored.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (100.0, 200.0, 100.0 + width, 210.0)
        );
        assert_eq!(placed.source(), &source);
    }
}
