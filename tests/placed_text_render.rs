#![cfg(feature = "render-text")]
#![forbid(unsafe_code)]

//! Real immutable normalized capture, Layout selection, and Render consumption.
//! This uses explicit native inputs; it does not authenticate a live Style tree.

use std::convert::Infallible;

use surgeist::{layout, render, text};

#[path = "placed_text_render/inter_word.rs"]
mod inter_word;

#[path = "placed_text_render/used_alphabetic.rs"]
mod used_alphabetic;

#[path = "placed_text_render/casing.rs"]
mod casing_transform;

#[path = "placed_text_render/width.rs"]
mod width_transform;

#[path = "placed_text_render/math.rs"]
mod math_transform;

#[path = "placed_text_render/separators.rs"]
mod separators_transform;

#[path = "placed_text_render/wbr.rs"]
mod wbr_transform;

#[path = "placed_text_render/phrase.rs"]
mod phrase_transform;

#[path = "placed_text_render/phrase_continuation.rs"]
mod phrase_continuation;

#[path = "placed_text_render/hanging.rs"]
mod hanging_punctuation;

#[path = "placed_text_render/computed_boundaries.rs"]
mod computed_boundaries;

#[path = "placed_text_render/sideways_rl.rs"]
mod sideways_rl;

#[path = "placed_text_render/sideways_atomics.rs"]
mod sideways_atomics;

struct Tree {
    inputs: [layout::LayoutInput; 2],
    non_box: layout::NodeInput,
}

impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        usize::from(node == 0)
    }

    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl layout::LayoutTree for Tree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }

    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

#[test]
fn actual_layout_lines_supply_paint_and_interaction_without_rewrapping() {
    assert_retained_layout_composition(false, 0.0, false);
}

#[cfg(feature = "text-accessibility")]
#[test]
fn expanded_spaces_supply_same_paint_cursor_and_geometric_accessibility() {
    assert_retained_layout_composition(true, 0.0, false);
}

#[test]
fn indented_lines_supply_retained_paint_and_interaction() {
    assert_retained_layout_composition(false, 5.0, false);
}

#[test]
fn contextual_trim_keeps_original_offsets_through_paint_and_interaction() {
    assert_retained_layout_composition(false, 0.0, true);
}

#[test]
fn css_tab_geometry_supplies_the_same_source_carets_and_render_admission() {
    assert_css_tab_composition(text::TabSize::length(80.0).unwrap(), 0.0, 80.0);
}

#[test]
fn negative_css_tab_geometry_supplies_the_same_source_carets_and_render_admission() {
    // Ahem SPACE20 plus block word-spacing -21 derives T=-4; ZERO20 gives
    // half-ch10. From pen20 the next stop24 receives one bump to28.
    assert_css_tab_composition(text::TabSize::spaces(4.0).unwrap(), -21.0, 28.0);
}

fn assert_css_tab_composition(tab_size: text::TabSize, block_spacing: f64, b_x: f64) {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!("../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        )
        .unwrap();
    let source = text::Source::new("A\tB");
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 20.0,
        line_height: text::LineHeight::Absolute(20.0),
        white_space: text::WhiteSpace::Preserve,
        tab_size,
        ..text::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: Some(
                text::ParagraphTabReference::new(
                    text::Style {
                        word_spacing: block_spacing,
                        ..style.clone()
                    },
                    text::ParagraphBaseDirection::LeftToRight,
                )
                .unwrap(),
            ),
            source: source.clone(),
            base_style: style,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(0, 3),
                aggregate: text::Range::new(0, 3),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::try_new(1.0, text::TabPreparation::CssStops).unwrap(),
        )
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                size: layout::Size::new(
                    layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(100.0).unwrap()),
                    layout::PreferredSizeOf::AUTO,
                ),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &capture,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(100.0),
            layout::Available::MaxContent,
        ))
        .unwrap(),
    )
    .unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.lines().len(), 1);
    assert_eq!(placed.source(), &source);
    let glyphs = placed
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.id(), run.origin().x + glyph.pen(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(35, 0.0, 20.0), (36, b_x, 20.0)]);
    let tab = placed
        .source_range_fragments(&origin, text::Range::new(1, 2))
        .unwrap();
    assert_eq!(
        tab.units()
            .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [text::Rect::new(20.0, 0.0, b_x - 20.0, 20.0)]
    );
    for (index, x) in [(1, 20.0), (2, b_x)] {
        assert_eq!(
            placed
                .cursor(text::Cursor::new(index, text::Affinity::After))
                .unwrap()
                .rect()
                .origin,
            text::Point::new(x, 0.0)
        );
    }
    drop(system);
    let paint = render::prepare_text_layout(
        placed,
        render::Transform::identity(),
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            8,
            32,
            8,
        ),
    )
    .unwrap();
    assert!(std::ptr::eq(paint.layout(), placed));
    assert!(!paint.scene().is_empty());
    assert_eq!(paint.layout().lines(), placed.lines());
}

fn assert_retained_layout_composition(expanded: bool, indent: f32, contextual: bool) {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!("../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        )
        .unwrap();
    let source = text::Source::new(if contextual {
        "  A X  "
    } else if expanded {
        "A X X"
    } else {
        "AX AX"
    });
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 20.0,
        line_height: text::LineHeight::Absolute(20.0),
        white_space: text::WhiteSpace::Collapse,
        ..text::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let origins = vec![text::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::Range::new(0, source.text().len()),
        aggregate: text::Range::new(0, source.text().len()),
        kind: text::ParagraphOriginKind::Authored,
    }];
    let analyzed = if contextual {
        let processed = std::sync::Arc::new(
            text::process_contextual_source(text::ContextualSourceInput {
                computed_line_boundaries: vec![],
                source: source.clone(),
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                envelope: text::SourceProcessingEnvelope::Complete,
                origins,
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                scopes: vec![
                    text::SourceProcessingScope::Block {
                        topology: text::SourceProcessingTopology::Unknown,
                        used_direction: text::ParagraphBaseDirection::LeftToRight,
                        trim: text::WhiteSpaceTrim::default(),
                        base_style: style,
                    },
                    text::SourceProcessingScope::Inline {
                        style: None,
                        boundaries: None,
                        trim: text::WhiteSpaceTrim {
                            discard_inner: true,
                            ..text::WhiteSpaceTrim::default()
                        },
                    },
                ],
                paragraphs: vec![text::SourceProcessingParagraph {
                    base_direction: text::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                }],
                events: vec![
                    text::SourceProcessingEvent::EnterScope(0),
                    text::SourceProcessingEvent::EnterParagraph(0),
                    text::SourceProcessingEvent::EnterScope(1),
                    text::SourceProcessingEvent::Origin {
                        origin: 0,
                        affinity: text::Affinity::After,
                    },
                    text::SourceProcessingEvent::ExitScope(1),
                    text::SourceProcessingEvent::ExitParagraph(0),
                    text::SourceProcessingEvent::ExitScope(0),
                ],
            })
            .unwrap(),
        );
        let paragraph = processed.paragraphs().next().unwrap();
        assert_eq!(paragraph.source_range(), text::Range::new(0, 7));
        let plaintext: Vec<u16> = paragraph
            .plaintext_items(text::Affinity::Before)
            .flat_map(|item| match item {
                text::PlaintextItem::Text(text) => text.code_units().to_vec(),
                text::PlaintextItem::InlineBox(_) => panic!("no authored atomic"),
            })
            .collect();
        assert_eq!(plaintext, "A X".encode_utf16().collect::<Vec<_>>());
        text::AnalyzedSource::from_processed_paragraph(
            paragraph,
            text::AnalysisTypography::HorizontalComposition,
        )
    } else {
        text::AnalyzedSource::from_paragraph(
            text::ParagraphInput {
                terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: style,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
                origins,
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            text::AnalysisTypography::HorizontalComposition,
        )
    }
    .unwrap();
    // This fixture's text belongs to the single inline box in Tree. Its
    // containing wrapping context is explicit even after contextual trimming.
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                text_align: if expanded {
                    layout::TextAlign::Justify
                } else {
                    layout::TextAlign::Start
                },
                text_justification: if expanded {
                    layout::TextJustification::Spaces
                } else {
                    layout::TextJustification::None
                },
                text_indent: layout::TextIndent {
                    amount: layout::LengthPercentageOf::px(indent).unwrap(),
                    ..layout::TextIndent::ZERO
                },
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::TextOriginBinding { origin, node: 1 }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request = layout::LayoutRootRequest::viewport(layout::Size::new(
        layout::Available::Definite(if expanded { 65.0 } else { 45.0 }),
        layout::Available::MaxContent,
    ))
    .unwrap();
    let batch = layout::compute_layout_with_flow(&capture, 0, request).unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    let committed = &batch.placed_text()[0];
    assert_eq!(committed.container(), 0);
    assert_eq!(committed.nodes(), &[1]);
    let placed = committed.layout();
    let lines = placed.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[0].range(),
        text::Range::new(0, if expanded || contextual { 4 } else { 3 })
    );
    assert_eq!(
        lines[1].range(),
        text::Range::new(
            if expanded || contextual { 4 } else { 3 },
            if contextual { 7 } else { 5 }
        )
    );
    // Ahem's one-em cells, 0.8em ascent, and explicit 20px line height
    // independently determine the glyph advances and baselines. Each line
    // retains the separate container allocation supplied by Layout. The expanded
    // first line has two 20px letters, one 20px internal space plus 5px extra,
    // and a discarded trailing space. The final line is a single 20px X.
    assert_eq!(lines[0].baseline(), 16.0);
    assert_eq!(lines[1].baseline(), 36.0);
    assert_eq!(
        lines[0].bounds().size.width,
        if expanded { 65.0 } else { 45.0 }
    );
    assert_eq!(
        lines[1].bounds().size.width,
        if expanded { 65.0 } else { 45.0 }
    );
    assert_eq!(
        placed
            .glyph_runs()
            .iter()
            .map(|run| run.advance())
            .collect::<Vec<_>>(),
        if expanded {
            vec![65.0, 20.0]
        } else if contextual {
            vec![20.0, 20.0]
        } else {
            vec![40.0, 40.0]
        }
    );
    assert_eq!(
        placed
            .cursor(text::Cursor::new(
                if expanded { 2 } else { 4 },
                text::Affinity::After
            ))
            .unwrap()
            .rect()
            .origin,
        if expanded {
            text::Point::new(45.0, 0.0)
        } else if contextual {
            text::Point::new(0.0, 20.0)
        } else {
            text::Point::new(20.0, 20.0)
        }
    );
    let stats = system.stats();
    // The indented AX AX case fits its first 40px word beside 5px in 45px.
    // Its first caret moves by that indent; the second line stays unindented.
    assert_eq!(
        placed
            .cursor(text::Cursor::new(
                if contextual { 2 } else { 0 },
                text::Affinity::After
            ))
            .unwrap()
            .rect()
            .origin,
        text::Point::new(f64::from(indent), 0.0)
    );
    drop(system);
    let paint = render::prepare_text_layout(
        placed,
        render::Transform::identity(),
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            8,
            32,
            8,
        ),
    )
    .unwrap();
    assert!(std::ptr::eq(paint.layout(), placed));
    assert_eq!(paint.scene().len(), 2);
    // Admission consumes the exact retained result; it cannot change its lines,
    // source associations, physical origins, or already finalized glyph rows.
    assert_eq!(paint.layout().lines(), lines);
    assert_eq!(stats.layout_misses(), 0);

    #[cfg(feature = "text-accessibility")]
    {
        let mut accessibility = text::Accessibility::new(10);
        let parent = 1000_u64.into();
        let update = paint
            .layout()
            .accessibility_update(
                &mut accessibility,
                parent,
                parent,
                text::Point::new(100.0, 200.0),
            )
            .unwrap();
        let parent_node = &update.nodes.iter().find(|(id, _)| *id == parent).unwrap().1;
        let parent_bounds = parent_node.bounds().unwrap();
        assert_eq!(
            (
                parent_bounds.x0,
                parent_bounds.y0,
                parent_bounds.x1,
                parent_bounds.y1
            ),
            (100.0, 200.0, if expanded { 165.0 } else { 145.0 }, 240.0)
        );
        let cursor = text::Cursor::new(if expanded { 2 } else { 4 }, text::Affinity::After);
        let position = paint
            .layout()
            .access_position(&accessibility, cursor)
            .unwrap()
            .unwrap();
        let node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == position.node)
            .unwrap()
            .1;
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            if expanded {
                (145.0, 200.0, 165.0, 220.0)
            } else if contextual {
                (100.0, 220.0, 120.0, 240.0)
            } else {
                (120.0, 220.0, 140.0, 240.0)
            }
        );
        assert_eq!(position.character_index, 0);
        assert_eq!(node.value(), Some("X"));
        assert_eq!(node.font_size(), Some(20.0));
        let first = paint
            .layout()
            .access_position(
                &accessibility,
                text::Cursor::new(if contextual { 2 } else { 0 }, text::Affinity::After),
            )
            .unwrap()
            .unwrap();
        let first_node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == first.node)
            .unwrap()
            .1;
        let first_bounds = first_node.bounds().unwrap();
        assert_eq!(
            (
                first_bounds.x0,
                first_bounds.y0,
                first_bounds.x1,
                first_bounds.y1
            ),
            (
                100.0 + f64::from(indent),
                200.0,
                120.0 + f64::from(indent),
                220.0
            )
        );
        assert_eq!(first_node.value(), Some("A"));
        let other = layout::compute_layout_with_flow(&capture, 0, request).unwrap();
        assert_eq!(
            other.placed_text()[0]
                .layout()
                .access_position(&accessibility, cursor)
                .unwrap_err()
                .detail(),
            Some(&text::ErrorDetail::AccessibilityBindingMismatch)
        );
    }
}

#[test]
fn after_indication_keeps_authored_carets_and_accessibility_through_paint_admission() {
    indication_keeps_authored_carets_and_accessibility_through_paint_admission(
        text::HyphenationPlacement::LogicalAfter,
    );
}

#[test]
fn both_indications_keep_authored_carets_and_accessibility_through_paint_admission() {
    indication_keeps_authored_carets_and_accessibility_through_paint_admission(
        text::HyphenationPlacement::BothSides,
    );
}

fn indication_keeps_authored_carets_and_accessibility_through_paint_admission(
    placement: text::HyphenationPlacement,
) {
    const AHEM: &[u8] =
        include_bytes!("../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let source = text::Source::identified("AB\u{ad}CDE", text::Id::from_u64(66191), 4);
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        hyphens: text::Hyphens::Manual,
        locale: Some("und-Latn".into()),
        hyphenate_character: text::HyphenateCharacter::Auto,
        native_hyphenation_convention: Some(
            text::NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                "A".into(),
                placement,
            )
            .unwrap(),
        ),
        ..text::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(0, 6),
                aggregate: text::Range::new(0, 6),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let cut = prepared.hyphenation_cut_views().next().unwrap();
    assert_eq!(
        cut.counts(),
        Some(text::HyphenationCounts {
            total: 5,
            before: 2,
            after: 3,
        })
    );
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                size: layout::Size::new(
                    layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(40.0).unwrap()),
                    layout::PreferredSizeOf::AUTO,
                ),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &capture,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(40.0),
            layout::Available::MaxContent,
        ))
        .unwrap(),
    )
    .unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source(), &source);
    assert_eq!(source.text().code_units(), [65, 66, 173, 67, 68, 69]);
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|line| (line.range(), line.baseline()))
            .collect::<Vec<_>>(),
        [
            (text::Range::new(0, 3), 8.0),
            (text::Range::new(3, 6), 18.0)
        ]
    );
    // Ahem's one-em cells give AB20 or ABA30, then incoming-A+CDE40.
    let mut glyphs = Vec::new();
    for run in placed.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
        for glyph in run.glyphs() {
            if glyph.advance() == 0.0 {
                // The actual original SHY may retain a formatting row, never
                // a second visible indication or arbitrary zero-width glyph.
                assert_eq!(run.line(), 0);
                assert!(run.range().start <= 2 && run.range().end >= 3);
                continue;
            }
            glyphs.push((
                run.line(),
                glyph.id(),
                glyph.advance(),
                run.origin().x + glyph.pen() + f64::from(glyph.x()),
                run.origin().y + f64::from(glyph.y()),
            ));
        }
    }
    let expected = match placement {
        text::HyphenationPlacement::LogicalAfter => vec![
            (0, 35, 10.0, 0.0, 8.0),
            (0, 36, 10.0, 10.0, 8.0),
            (1, 35, 10.0, 0.0, 18.0),
            (1, 37, 10.0, 10.0, 18.0),
            (1, 38, 10.0, 20.0, 18.0),
            (1, 39, 10.0, 30.0, 18.0),
        ],
        text::HyphenationPlacement::BothSides => vec![
            (0, 35, 10.0, 0.0, 8.0),
            (0, 36, 10.0, 10.0, 8.0),
            (0, 35, 10.0, 20.0, 8.0),
            (1, 35, 10.0, 0.0, 18.0),
            (1, 37, 10.0, 10.0, 18.0),
            (1, 38, 10.0, 20.0, 18.0),
            (1, 39, 10.0, 30.0, 18.0),
        ],
        text::HyphenationPlacement::LogicalBefore => unreachable!(),
    };
    assert_eq!(glyphs, expected);
    let shy = placed
        .source_range_fragments(&origin, text::Range::new(2, 3))
        .unwrap();
    let units = shy.units().collect::<Vec<_>>();
    let expected_units = match placement {
        text::HyphenationPlacement::LogicalAfter => {
            vec![(1, 1, vec![text::Rect::new(0.0, 10.0, 10.0, 10.0)])]
        }
        text::HyphenationPlacement::BothSides => vec![
            (0, 1, vec![text::Rect::new(20.0, 0.0, 10.0, 10.0)]),
            (1, 1, vec![text::Rect::new(0.0, 10.0, 10.0, 10.0)]),
        ],
        text::HyphenationPlacement::LogicalBefore => unreachable!(),
    };
    assert_eq!(
        units
            .iter()
            .map(|unit| (
                unit.line_index(),
                unit.alternatives().count(),
                unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
        expected_units
    );
    if placement == text::HyphenationPlacement::BothSides {
        assert!(!units[0].same_unit(&units[1]));
    }
    let seam = text::Cursor::new(3, text::Affinity::After);
    let assert_ambiguous = |error: text::Error| {
        assert_eq!(
            error.detail(),
            Some(&text::ErrorDetail::InvalidTextProtocol {
                reason: text::ProtocolError::AmbiguousSourcePosition {
                    index: 3,
                    affinity: text::Affinity::After,
                },
            })
        );
    };
    // Both emissions have genuine source-empty occurrences at the same source
    // endpoint on distinct lines. Retain issued positions for those exact edges.
    let seam_positions = if placement == text::HyphenationPlacement::BothSides {
        assert_ambiguous(placed.cursor(seam).unwrap_err());
        let positions = placed
            .positions()
            .filter(|position| position.source_cursor().index() == 3)
            .map(|position| {
                let rect = placed.position_geometry(&position).unwrap().rect();
                assert_eq!(rect.size, text::Size::new(1.0, 10.0));
                (position, rect)
            })
            .collect::<Vec<_>>();
        let mut edges = Vec::new();
        for (_, rect) in &positions {
            if !edges.contains(&rect.origin) {
                edges.push(rect.origin);
            }
        }
        edges.sort_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)));
        assert_eq!(
            edges,
            [
                text::Point::new(20.0, 0.0),
                text::Point::new(30.0, 0.0),
                text::Point::new(0.0, 10.0),
                text::Point::new(10.0, 10.0),
            ]
        );
        positions
    } else {
        Vec::new()
    };
    // D is an unambiguous authored caret for Both; retain the existing C/After
    // control where only the receiving manifestation exists.
    let (cursor, caret_x) = if placement == text::HyphenationPlacement::BothSides {
        (text::Cursor::new(4, text::Affinity::After), 20.0)
    } else {
        (seam, 10.0)
    };
    let caret = placed.cursor(cursor).unwrap().rect();
    assert_eq!(caret.origin, text::Point::new(caret_x, 10.0));
    assert_eq!(caret.size.height, 10.0);
    drop(system);
    let paint = render::prepare_text_layout(
        placed,
        render::Transform::identity(),
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            8,
            32,
            8,
        ),
    )
    .unwrap();
    assert!(!paint.scene().is_empty());
    assert!(std::ptr::eq(paint.layout(), placed));
    assert_eq!(paint.layout().cursor(cursor).unwrap().rect(), caret);
    for (position, rect) in &seam_positions {
        assert_eq!(
            paint.layout().position_geometry(position).unwrap().rect(),
            *rect
        );
    }
    if placement == text::HyphenationPlacement::BothSides {
        assert_ambiguous(paint.layout().cursor(seam).unwrap_err());
    }
    assert_eq!(paint.layout().source(), &source);
    #[cfg(feature = "text-accessibility")]
    {
        let mut accessibility = text::Accessibility::new(10);
        let parent = 1000_u64.into();
        let update = paint
            .layout()
            .accessibility_update(
                &mut accessibility,
                parent,
                parent,
                text::Point::new(100.0, 200.0),
            )
            .unwrap();
        let position = paint
            .layout()
            .access_position(&accessibility, cursor)
            .unwrap()
            .unwrap();
        let node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == position.node)
            .unwrap()
            .1;
        let character = if placement == text::HyphenationPlacement::BothSides {
            "D"
        } else {
            "C"
        };
        assert_eq!(node.value(), Some(character));
        assert_eq!(position.character_index, 0);
        if placement == text::HyphenationPlacement::BothSides {
            assert_ambiguous(
                paint
                    .layout()
                    .access_position(&accessibility, seam)
                    .unwrap_err(),
            );
        }
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (100.0 + caret_x, 210.0, 110.0 + caret_x, 220.0)
        );
    }
}

// Native kana contraction through the actual Layout and Render consumer boundary.
// The fixed derivative below reuses licensed Ahem, whose retained provenance is
// crates/surgeist-text/tests/fixtures/fonts/ahem/PROVENANCE.md. Its cmap/hmtx facts
// match the independently qualified Text kana fixture, without private test APIs.
mod kana_transform {
    use super::*;

    fn kana_ahem() -> Vec<u8> {
        fn u16_at(bytes: &[u8], at: usize) -> u16 {
            u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
        }
        fn u32_at(bytes: &[u8], at: usize) -> u32 {
            u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
        }
        fn checksum(bytes: &[u8]) -> u32 {
            bytes.chunks(4).fold(0_u32, |sum, chunk| {
                let mut word = [0; 4];
                word[..chunk.len()].copy_from_slice(chunk);
                sum.wrapping_add(u32::from_be_bytes(word))
            })
        }
        fn record(bytes: &[u8], tag: &[u8; 4]) -> usize {
            (0..usize::from(u16_at(bytes, 4)))
                .map(|index| 12 + index * 16)
                .find(|at| &bytes[*at..*at + 4] == tag)
                .unwrap()
        }
        let mut bytes =
            include_bytes!("../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec();
        // One Windows UCS-4 format12 subtable. Every glyph exists in Ahem.
        let mut cmap = Vec::from(&[0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12][..]);
        cmap.extend(12_u16.to_be_bytes());
        cmap.extend(0_u16.to_be_bytes());
        cmap.extend(64_u32.to_be_bytes()); //16-byte header plus four12-byte groups
        cmap.extend(0_u32.to_be_bytes());
        cmap.extend(4_u32.to_be_bytes());
        for (scalar, glyph) in [(0x41_u32, 35_u32), (0x43, 37), (0x3053, 36), (0x1b132, 82)] {
            cmap.extend(scalar.to_be_bytes());
            cmap.extend(scalar.to_be_bytes());
            cmap.extend(glyph.to_be_bytes());
        }
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let cmap_at = u32::try_from(bytes.len()).unwrap();
        let entry = record(&bytes, b"cmap");
        bytes[entry + 4..entry + 8].copy_from_slice(&checksum(&cmap).to_be_bytes());
        bytes[entry + 8..entry + 12].copy_from_slice(&cmap_at.to_be_bytes());
        bytes[entry + 12..entry + 16]
            .copy_from_slice(&u32::try_from(cmap.len()).unwrap().to_be_bytes());
        bytes.extend(cmap);
        let entry = record(&bytes, b"hmtx");
        let at = usize::try_from(u32_at(&bytes, entry + 8)).unwrap();
        let length = usize::try_from(u32_at(&bytes, entry + 12)).unwrap();
        assert_eq!(u16_at(&bytes, at + 36 * 4), 1000);
        bytes[at + 36 * 4..at + 36 * 4 + 2].copy_from_slice(&500_u16.to_be_bytes());
        let sum = checksum(&bytes[at..at + length]);
        bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
        let entry = record(&bytes, b"head");
        let at = usize::try_from(u32_at(&bytes, entry + 8)).unwrap();
        let length = usize::try_from(u32_at(&bytes, entry + 12)).unwrap();
        assert_eq!(u16_at(&bytes, at + 18), 1000);
        bytes[at + 8..at + 12].fill(0);
        let sum = checksum(&bytes[at..at + length]);
        bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
        let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
        bytes[at + 8..at + 12].copy_from_slice(&adjustment.to_be_bytes());
        assert_eq!(checksum(&bytes), 0xb1b0_afba);
        bytes
    }

    fn capture(
        system: &mut text::System,
        enabled: bool,
        available: layout::Available,
    ) -> (
        layout::CompletedLayoutBatch<usize>,
        text::PreparedOrigin,
        text::Source,
    ) {
        let source = text::Source::identified("A\u{1b132}C", text::Id::from_u64(6301), 11);
        let units = [0x41, 0xd82c, 0xdd32, 0x43];
        let style = text::Style {
            font: text::Font::new().family("Ahem"),
            size: 10.0,
            line_height: text::LineHeight::Absolute(10.0),
            white_space: text::WhiteSpace::Preserve,
            text_transform: text::TextTransform::Set(
                text::TextTransformSet::none().with_full_size_kana(enabled),
            ),
            ..text::Style::default()
        };
        let text::TextTransform::Set(set) = style.text_transform else {
            panic!("expected composable set")
        };
        assert_eq!(set.full_size_kana(), enabled);
        let style = text::Style {
            word_break: text::WordBreak::BreakAll,
            ..style
        };
        let ancestor_wrap = style.wrap;
        let analyzed = text::AnalyzedSource::from_paragraph(
            text::ParagraphInput {
                source: source.clone(),
                base_style: style,
                segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
                terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::Range::new(0, 4),
                    aggregate: text::Range::new(0, 4),
                    kind: text::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            analyzed.processed_source().processed_text().code_units(),
            units
        );
        let plaintext = analyzed
            .processed_source()
            .plaintext_items(text::Affinity::Before)
            .flat_map(|item| match item {
                text::PlaintextItem::Text(value) => value.code_units().to_vec(),
                text::PlaintextItem::InlineBox(_) => panic!("no boxes in the actual source"),
            })
            .collect::<Vec<_>>();
        assert_eq!(plaintext, units);
        let policies = analyzed
            .wrapping_relations()
            .filter(|relation| relation.ancestor_wrap().is_none())
            .map(|relation| text::WrappingPolicy {
                relation: relation.handle(),
                ancestor_wrap,
            })
            .collect();
        let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
        let prepared = system
            .prepare(&analyzed, text::PreparationOptions::default())
            .unwrap();
        let origin = prepared.origin_views().next().unwrap().handle();

        let tree = Tree {
            inputs: [
                layout::LayoutInput::box_input(layout::NodeInput {
                    display: layout::Display::Block,
                    size: layout::Size::new(
                        match available {
                            layout::Available::Definite(width) => layout::PreferredSizeOf::value(
                                layout::LengthPercentageOf::px(width).unwrap(),
                            ),
                            _ => layout::PreferredSizeOf::AUTO,
                        },
                        layout::PreferredSizeOf::AUTO,
                    ),
                    ..layout::NodeInput::DEFAULT
                }),
                layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
            ],
            non_box: layout::NodeInput::non_box(),
        };
        let capture = layout::LayoutFlowInput::try_new(
            &tree,
            0,
            vec![layout::TextParagraphBinding {
                prepared,
                container: 0,
                nodes: vec![1],
                origins: vec![layout::TextOriginBinding {
                    origin: origin.clone(),
                    node: 1,
                }],
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            }],
            surgeist_layout::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let batch = layout::compute_layout_with_flow(
            &capture,
            0,
            layout::LayoutRootRequest::viewport(layout::Size::new(
                available,
                layout::Available::MaxContent,
            ))
            .unwrap(),
        )
        .unwrap();
        (batch, origin, source)
    }

    #[test]
    fn full_size_kana_contraction_keeps_authored_geometry_through_layout_and_paint() {
        let bytes = kana_ahem();
        // None qualifies the original supplementary cmap/metrics separately.
        // Kana fits the25px block with the actual shorter glyph, without any
        // Render-side replacement, reshaping or second line layout.
        for enabled in [false, true] {
            let mut system = text::System::new(text::SystemOptions {
                font_catalogue: text::FontCatalogueMode::RegisteredOnly,
            })
            .unwrap();
            system.register_fonts(bytes.clone()).unwrap();
            let (batch, origin, source) = capture(
                &mut system,
                enabled,
                layout::Available::Definite(if enabled { 25.0 } else { 40.0 }),
            );
            let units = [0x41, 0xd82c, 0xdd32, 0x43];
            let placed = batch.placed_text()[0].layout();
            let advance = if enabled { 25.0 } else { 30.0 };
            let middle = if enabled { 5.0 } else { 10.0 };
            let c_x = 10.0 + middle;
            assert_eq!(placed.source(), &source);
            assert_eq!(placed.source().text().code_units(), units);
            assert_eq!(placed.source().id(), Some(text::Id::from_u64(6301)));
            assert_eq!(placed.source().revision(), 11);
            assert_eq!(placed.lines().len(), 1);
            assert_eq!(placed.lines()[0].range(), text::Range::new(0, 4));
            // Line bounds describe the actual allocated span; glyphs and
            // finalized feedback below own the content-advance observation.
            assert_eq!(
                placed.lines()[0].bounds().size.width,
                if enabled { 25.0 } else { 40.0 }
            );
            let finalized = batch.final_inline_fragments()[0]
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(finalized.feedback().advance, advance);
            assert_eq!(
                finalized
                    .mapping_views()
                    .map(|mapping| mapping.processed_range().end().get())
                    .max(),
                Some(4)
            );
            assert_eq!(
                finalized
                    .mapping_views()
                    .map(|mapping| mapping.presentation_range().end())
                    .max(),
                Some(if enabled { 3 } else { 4 })
            );
            for run in placed.run_views() {
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.font_size(), 10.0);
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            }
            let glyphs = placed
                .glyph_runs()
                .iter()
                .flat_map(|run| {
                    run.glyphs()
                        .iter()
                        .map(|glyph| {
                            (
                                glyph.id(),
                                glyph.advance(),
                                run.origin().x + glyph.pen() + f64::from(glyph.x()),
                                glyph.range(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                glyphs,
                [
                    (35, 10.0, 0.0, text::Range::new(0, 1)),
                    (
                        if enabled { 36 } else { 82 },
                        middle as f32,
                        10.0,
                        text::Range::new(1, 3)
                    ),
                    (37, 10.0, c_x, text::Range::new(3, 4)),
                ]
            );
            let whole = placed
                .source_range_fragments(&origin, text::Range::new(1, 3))
                .unwrap();
            let first = placed
                .source_range_fragments(&origin, text::Range::new(1, 2))
                .unwrap();
            let second = placed
                .source_range_fragments(&origin, text::Range::new(2, 3))
                .unwrap();
            let whole_unit = whole.units().next().unwrap();
            assert_eq!(whole.units().count(), 1);
            assert_eq!(first.local_range(), text::Range::new(1, 2));
            assert_eq!(second.local_range(), text::Range::new(2, 3));
            assert!(whole_unit.same_unit(&first.units().next().unwrap()));
            assert!(whole_unit.same_unit(&second.units().next().unwrap()));
            for fragments in [&whole, &first, &second] {
                assert_eq!(
                    fragments
                        .units()
                        .flat_map(|unit| unit
                            .pieces()
                            .map(|piece| piece.rect())
                            .collect::<Vec<_>>())
                        .collect::<Vec<_>>(),
                    [text::Rect::new(10.0, 0.0, middle, 10.0)]
                );
            }
            for (index, affinity, x) in [
                (1, text::Affinity::After, 10.0),
                (2, text::Affinity::Before, 10.0),
                (2, text::Affinity::After, c_x),
                (3, text::Affinity::Before, c_x),
                (4, text::Affinity::Before, advance),
            ] {
                assert_eq!(
                    placed
                        .cursor(text::Cursor::new(index, affinity))
                        .unwrap()
                        .rect()
                        .origin,
                    text::Point::new(x, 0.0)
                );
            }
            let misses = system.stats().layout_misses();
            drop(system);
            let paint = render::prepare_text_layout(
                placed,
                render::Transform::identity(),
                render::TextLayoutLimits::new(
                    render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
                    8,
                    32,
                    8,
                ),
            )
            .unwrap();
            assert!(std::ptr::eq(paint.layout(), placed));
            assert!(!paint.scene().is_empty());
            assert_eq!(paint.layout().lines(), placed.lines());
            assert_eq!(paint.layout().source(), &source);
            assert_eq!(misses, 0);
            #[cfg(feature = "text-accessibility")]
            {
                let mut accessibility = text::Accessibility::new(10);
                let parent = 1000_u64.into();
                let update = paint
                    .layout()
                    .accessibility_update(
                        &mut accessibility,
                        parent,
                        parent,
                        text::Point::new(100.0, 200.0),
                    )
                    .unwrap();
                let position = paint
                    .layout()
                    .access_position(&accessibility, text::Cursor::new(1, text::Affinity::After))
                    .unwrap()
                    .unwrap();
                let node = &update
                    .nodes
                    .iter()
                    .find(|(id, _)| *id == position.node)
                    .unwrap()
                    .1;
                assert_eq!(node.value(), Some("\u{1b132}"));
                assert_eq!(position.character_index, 0);
                let bounds = node.bounds().unwrap();
                assert_eq!(
                    (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
                    (110.0, 200.0, 110.0 + middle, 210.0)
                );
            }
        }
    }
    #[test]
    fn kana_contraction_changes_actual_finite_fit_and_intrinsic_widths() {
        // Pinned Text4 break-all treats letter units as ID. No hyphenation or
        // glyph fitting substitutes for these actual10/5/10 provider advances.
        let bytes = kana_ahem();
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(bytes).unwrap();
        for enabled in [false, true] {
            let (batch, _, source) =
                capture(&mut system, enabled, layout::Available::Definite(17.0));
            let placed = batch.placed_text()[0].layout();
            let advances = batch
                .final_inline_fragments()
                .iter()
                .map(|entry| {
                    let fragment = entry.fragment();
                    (
                        fragment.line_index(),
                        fragment
                            .finalized_fragment()
                            .finalized_line()
                            .feedback()
                            .advance,
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            assert_eq!(advances.len(), placed.lines().len());
            assert!(
                placed
                    .lines()
                    .iter()
                    .all(|line| line.bounds().size.width == 17.0)
            );
            let actual = placed
                .lines()
                .iter()
                .enumerate()
                .map(|(index, line)| (line.range(), advances[&index]))
                .collect::<Vec<_>>();
            assert_eq!(
                actual,
                if enabled {
                    vec![
                        (text::Range::new(0, 3), 15.0),
                        (text::Range::new(3, 4), 10.0),
                    ]
                } else {
                    vec![
                        (text::Range::new(0, 1), 10.0),
                        (text::Range::new(1, 3), 10.0),
                        (text::Range::new(3, 4), 10.0),
                    ]
                }
            );
            assert_eq!(placed.source(), &source);
            let glyphs = placed
                .run_views()
                .flat_map(|run| {
                    run.glyphs()
                        .map(|glyph| (glyph.id(), glyph.advance()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                glyphs,
                if enabled {
                    vec![(35, 10.0), (36, 5.0), (37, 10.0)]
                } else {
                    vec![(35, 10.0), (82, 10.0), (37, 10.0)]
                }
            );
            for (available, expected) in [
                (layout::Available::MinContent, 10.0),
                (
                    layout::Available::MaxContent,
                    if enabled { 25.0 } else { 30.0 },
                ),
            ] {
                let (intrinsic, _, same_source) = capture(&mut system, enabled, available);
                assert_eq!(same_source.text().code_units(), source.text().code_units());
                let root = intrinsic
                    .final_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output();
                assert_eq!(root.size.width, expected);
            }
        }
    }
}
