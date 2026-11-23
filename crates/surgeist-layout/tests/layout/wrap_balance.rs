//! Current ordinary wrapping over real captured Ahem source and mandatory groups.
use std::{convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Tree<S: l::scalar::LayoutScalar> {
    inputs: Vec<l::node::LayoutInputOf<S>>,
    non_box: l::node::NodeInputOf<S>,
    children: Vec<Vec<u32>>,
}
impl<S: l::scalar::LayoutScalar> l::tree::Traverse for Tree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, n: u32) -> Self::Children<'_> {
        self.children[n as usize].iter().copied()
    }
    fn child_count(&self, n: u32) -> usize {
        self.children[n as usize].len()
    }
    fn child(&self, n: u32, i: usize) -> u32 {
        self.children[n as usize][i]
    }
}
impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for Tree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, n: u32) -> &l::node::NodeInputOf<S> {
        self.inputs[n as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, n: u32) -> l::node::LayoutInputViewOf<'_, S> {
        self.inputs[n as usize].as_view()
    }
}
struct Fixture<S: l::scalar::LayoutScalar> {
    capture: Arc<t::processing::ProcessedSource>,
    prepared: t::prepared::PreparedText,
    binding: l::text_layout::TextParagraphBinding<u32>,
    tree: Tree<S>,
}
impl<S: l::scalar::LayoutScalar> Fixture<S> {
    fn new(direction: l::geometry::Direction, wrap: t::style::Wrap, forced_groups: bool) -> Self {
        use t::capture::SourceProcessingEvent as E;
        let raw = if forced_groups {
            "AAA BB CAAA BB C"
        } else {
            "AAA BB C"
        };
        let base = if direction == l::geometry::Direction::Ltr {
            t::paragraph::ParagraphBaseDirection::LeftToRight
        } else {
            t::paragraph::ParagraphBaseDirection::RightToLeft
        };
        let style = t::style::Style {
            font: t::style::Font::new().family("Ahem"),
            size: 10.0,
            line_height: t::style::LineHeight::Absolute(10.0),
            white_space: t::style::WhiteSpace::Collapse,
            wrap,
            word_break: t::style::WordBreak::Normal,
            overflow_wrap: t::style::OverflowWrap::Normal,
            hyphens: t::hyphenation::Hyphens::None,
            ..t::style::Style::default()
        };
        let mut source = t::source::Source::identified(
            raw,
            t::source::Id::from_u64(if forced_groups { 658401 } else { 658400 }),
            55,
        );
        source.span(t::source::Range::new(0, raw.len()), style.clone());
        let ranges = if forced_groups {
            vec![t::source::Range::new(0, 8), t::source::Range::new(8, 16)]
        } else {
            vec![t::source::Range::new(0, 8)]
        };
        let mut events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: t::layout::Affinity::After,
            },
        ];
        if forced_groups {
            events.extend([
                E::Control {
                    control: 0,
                    affinity: t::layout::Affinity::After,
                },
                E::Connection(0),
                E::Origin {
                    origin: 1,
                    affinity: t::layout::Affinity::After,
                },
            ]);
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let capture = Arc::new(
            t::capture::process_contextual_source(t::capture::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source: source.clone(),
                segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::capture::SourceProcessingEnvelope::Complete,
                origins: ranges
                    .iter()
                    .map(|range| t::paragraph::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: *range,
                        aggregate: *range,
                        kind: t::paragraph::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                controls: if forced_groups {
                    vec![t::paragraph::ParagraphControl {
                        at: 8,
                        kind: t::paragraph::ParagraphControlKind::ForcedBreak,
                    }]
                } else {
                    vec![]
                },
                connections: if forced_groups {
                    vec![t::capture::SourceProcessingConnection {
                        before_origin: 0,
                        after_origin: 1,
                        kind: t::paragraph::ParagraphConnectionKind::Continuous,
                    }]
                } else {
                    vec![]
                },
                boundary_policies: if forced_groups {
                    vec![t::paragraph::ParagraphBoundaryPolicy {
                        definition: t::paragraph::ParagraphBoundaryDefinition::Connection(0),
                        preceding_wrap: wrap,
                        following_wrap: wrap,
                        ancestor_wrap: wrap,
                    }]
                } else {
                    vec![]
                },
                replacements: vec![],
                computed_line_boundaries: vec![],
                scopes: vec![t::capture::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: t::capture::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style,
                    used_direction: base,
                }],
                paragraphs: vec![t::capture::SourceProcessingParagraph {
                    base_direction: base,
                    paragraph_strut: t::paragraph::ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        let analyzed = t::analyzed::AnalyzedSource::from_processed_paragraph(
            capture.paragraphs().next().unwrap(),
            t::analyzed::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let policies = analyzed
            .wrapping_relations()
            .filter(|r| r.ancestor_wrap().is_none())
            .map(|r| t::analyzed::WrappingPolicy {
                relation: r.handle(),
                ancestor_wrap: wrap,
            })
            .collect();
        let mut fonts = t::System::default();
        fonts
            .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
            .unwrap();
        let prepared = fonts
            .prepare(
                &analyzed.with_wrapping_policies(policies).unwrap(),
                t::prepared::PreparationOptions::default(),
            )
            .unwrap();
        assert_eq!(
            prepared
                .captured_owner()
                .unwrap()
                .members()
                .collect::<Vec<_>>(),
            vec![capture.captured_paragraph(0).unwrap()]
        );
        assert_eq!(prepared.origin_views().count(), ranges.len());
        assert_eq!(prepared.control_views().count(), usize::from(forced_groups));
        assert_eq!(prepared.atomic_views().count(), 0);
        assert_eq!(prepared.virtual_separators().count(), 0);
        assert!(capture.captured_computed_line_boundary(0).is_none());
        assert_eq!(
            capture.source().text().code_units(),
            raw.encode_utf16().collect::<Vec<_>>()
        );
        for (view, expected) in prepared.origin_views().zip(&ranges) {
            assert_eq!(view.source(), capture.source());
            assert_eq!(view.local_range(), *expected);
            assert_eq!(view.aggregate_range(), *expected);
        }
        let origin_handles = prepared
            .origin_views()
            .map(|view| view.handle())
            .collect::<Vec<_>>();
        let mut inputs = vec![
            l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
                display: l::box_model::Display::Block,
                direction,
                size: l::geometry::Size::new(
                    l::sizing::PreferredSizeOf::value(
                        l::length::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                    ),
                    l::sizing::PreferredSizeOf::AUTO,
                ),
                ..l::node::NodeInputOf::default()
            }),
            l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                origin_handles[0].clone(),
            )),
        ];
        let mut origins = vec![l::text_layout::TextOriginBinding {
            origin: origin_handles[0].clone(),
            node: 1,
        }];
        let mut controls = vec![];
        if forced_groups {
            let control = prepared.control_views().next().unwrap().handle();
            assert!(matches!(
                control.view().kind(),
                t::paragraph::ParagraphControlKind::ForcedBreak
            ));
            assert_eq!(control.captured_control(), capture.captured_control(0));
            inputs.push(l::node::LayoutInputOf::line_break(
                l::text_flow::LineBreakInputOf::new()
                    .with_direction(direction)
                    .with_metrics(
                        l::inline::InlineMetricsOf::try_new(
                            surgeist_layout::baseline::InlineBaselineContext::Physical,
                            S::from_f64(8.0),
                            S::from_f64(10.0),
                        )
                        .unwrap(),
                    ),
            ));
            inputs.push(l::node::LayoutInputOf::inline_text(
                l::text_layout::TextParticipation::new(origin_handles[1].clone()),
            ));
            origins.push(l::text_layout::TextOriginBinding {
                origin: origin_handles[1].clone(),
                node: 3,
            });
            controls.push(l::text_layout::TextControlBinding {
                occurrence: control,
                node: 2,
            });
        }
        let nodes = if forced_groups {
            vec![1, 2, 3]
        } else {
            vec![1]
        };
        let mut children = vec![vec![]; inputs.len()];
        children[0] = nodes.clone();
        let binding = l::text_layout::TextParagraphBinding {
            inline_box_fragments:
                surgeist_layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: prepared.clone(),
            container: 0,
            nodes,
            origins,
            controls,
            atomics: vec![],
            line_strut: None,
        };
        Self {
            capture,
            prepared,
            binding,
            tree: Tree {
                inputs,
                non_box: l::node::NodeInputOf::non_box(),
                children,
            },
        }
    }
    fn compute(&self) -> l::batch::CompletedLayoutBatchOf<u32, S> {
        let flow = l::text_layout::LayoutFlowInput::try_new(
            &self.tree,
            0,
            vec![self.binding.clone()],
            l::inline_wrapping::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let request = {
            let sizing_request = crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
                surgeist_layout::geometry::Size::new(
                    surgeist_layout::constraint::SizingOperation::Normal,
                    surgeist_layout::constraint::SizingOperation::Normal,
                ),
            );
            l::request::LayoutRootRequestOf::standalone(
                sizing_request,
                surgeist_layout::geometry::FlowAxes::new(
                    surgeist_layout::tree::LayoutTree::node_input(&self.tree, 0).writing_mode,
                    surgeist_layout::tree::LayoutTree::node_input(&self.tree, 0).direction,
                ),
                sizing_request
                    .orthogonal_fallback()
                    .initial_containing_block(),
            )
        };
        l::compute_layout_with_flow(&flow, 0, request).unwrap()
    }
    fn assert_lines(
        &self,
        batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
        expected: &[(t::source::Range, f64, t::selection::LineEnd)],
        direction: l::geometry::Direction,
    ) {
        assert_eq!(batch.placed_text().len(), 1);
        let entry = &batch.placed_text()[0];
        assert_eq!(entry.container(), 0);
        assert_eq!(entry.nodes(), self.binding.nodes.as_slice());
        let layout = entry.layout();
        assert_eq!(
            layout.analyzed_source().identity(),
            self.prepared.analyzed_source().identity()
        );
        assert_eq!(layout.source(), self.capture.source());
        assert_eq!(
            layout.source().id(),
            Some(t::source::Id::from_u64(
                if self.binding.controls.is_empty() {
                    658400
                } else {
                    658401
                }
            ))
        );
        assert_eq!(layout.source().revision(), 55);
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|(range, _, _)| *range)
                .collect::<Vec<_>>()
        );
        assert_eq!(layout.metrics().line_count(), expected.len());
        assert_eq!(layout.nonformatted_lines().count(), 0);
        let mut finalized = vec![None; expected.len()];
        for fragment in batch.final_inline_fragments() {
            let l::text_layout::InlineFragmentOwner::Node(node) = fragment.owner() else {
                continue;
            };
            if !self
                .binding
                .origins
                .iter()
                .any(|origin| origin.node == *node)
            {
                continue;
            }
            let index = fragment.fragment().line_index();
            let line = fragment.fragment().finalized_fragment().finalized_line();
            assert_eq!(line.prepared_text(), self.prepared);
            if let Some(old) = &finalized[index] {
                assert_eq!(old, &line);
            } else {
                finalized[index] = Some(line);
            }
        }
        for (i, (range, alignment, end)) in expected.iter().enumerate() {
            let line = finalized[i]
                .as_ref()
                .expect("each actual body line has a source fragment");
            assert_eq!(line.measurements().alignment().advance(), *alignment);
            assert_eq!(line.measurements().fit().advance(), *alignment);
            assert_eq!(line.feedback().advance, *alignment);
            let selected = line.selected_line();
            let request = selected.request();
            assert_eq!(request.context.end, *end);
            assert_eq!(request.start.view().source_after().get(), range.start);
            assert_eq!(request.end.view().source_after().get(), range.end);
            let t::selection::PreparedTermination::Ordinary(offer) = &request.termination else {
                panic!("current ordinary wrapping uses actual ordinary/mandatory capabilities");
            };
            assert_eq!(
                offer.view().kind(),
                match end {
                    t::selection::LineEnd::Soft => t::prepared::PreparedBreakKind::Allowed,
                    t::selection::LineEnd::Forced => t::prepared::PreparedBreakKind::Forced,
                    t::selection::LineEnd::Paragraph =>
                        t::prepared::PreparedBreakKind::ParagraphEnd,
                }
            );
            assert!(request.alternative.is_none());
            assert!(request.incoming.is_none());
            let published = layout.lines()[i];
            assert_eq!(published.bounds().size, t::geometry::Size::new(60.0, 10.0));
            assert_eq!(published.baseline(), 8.0 + 10.0 * i as f64);
            let mut glyph_ranges = vec![];
            for run in line.run_views() {
                assert_eq!(run.face().bytes(), AHEM);
                assert_eq!(run.font_size(), 10.0);
                for glyph in run.glyphs() {
                    assert!(
                        glyph.id() > 0,
                        "actual fixture coverage, no .notdef surrogate"
                    );
                    assert_eq!(glyph.advance(), 10.0);
                    glyph_ranges.push(glyph.range());
                }
            }
            // Every selected internal SPACE is one Ahem cell. Only the actual
            // collapsing trailing SPACE at a soft edge is removed from painting.
            let painted_end = if *end == t::selection::LineEnd::Soft {
                range.end - 1
            } else {
                range.end
            };
            assert_eq!(
                glyph_ranges,
                (range.start..painted_end)
                    .map(|at| t::source::Range::new(at, at + 1))
                    .collect::<Vec<_>>()
            );
            if *end == t::selection::LineEnd::Forced {
                assert_eq!(request.controls.len(), 1);
                assert_eq!(
                    request.controls[0].occurrence,
                    self.binding.controls[0].occurrence
                );
            }
        }
        let root = batch
            .node_layout(0)
            .unwrap()
            .single_box()
            .expect("fixture node produces one box")
            .final_output();
        assert_eq!(
            root.size,
            l::geometry::Size::new(S::from_f64(60.0), S::from_f64(10.0 * expected.len() as f64))
        );
        if expected.len() == 2 {
            let last = layout.run_views().find(|run| run.line() == 1).unwrap();
            let x = if direction == l::geometry::Direction::Ltr {
                0.0
            } else {
                60.0 - expected[1].1
            };
            let glyph = last.glyphs().next().unwrap();
            assert_eq!(last.origin().x + glyph.pen() + f64::from(glyph.x()), x);
        }
    }
}
fn characterize<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    wrap: t::style::Wrap,
    forced: bool,
    expected: &[(t::source::Range, f64, t::selection::LineEnd)],
) {
    let fixture = Fixture::<S>::new(direction, wrap, forced);
    let before = fixture
        .prepared
        .captured_owner()
        .unwrap()
        .members()
        .collect::<Vec<_>>();
    let batch = fixture.compute();
    fixture.assert_lines(&batch, expected, direction);
    let replay = fixture.compute();
    fixture.assert_lines(&replay, expected, direction);
    assert_eq!(
        batch.placed_text()[0].layout().key(),
        replay.placed_text()[0].layout().key()
    );
    assert_eq!(
        fixture
            .prepared
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        before
    );
    fixture.assert_lines(&batch, expected, direction);
}
#[test]
fn current_ordinary_wrapping_uses_full_fitting_prefix_then_remaining_word() {
    let expected = [
        (
            t::source::Range::new(0, 7),
            60.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(7, 8),
            10.0,
            t::selection::LineEnd::Paragraph,
        ),
    ];
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        characterize::<f32>(direction, t::style::Wrap::Word, false, &expected);
        characterize::<f64>(direction, t::style::Wrap::Word, false, &expected);
    }
}
#[test]
fn actual_forced_control_separates_two_equivalent_ordinary_wrapping_groups() {
    let expected = [
        (
            t::source::Range::new(0, 7),
            60.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(7, 8),
            10.0,
            t::selection::LineEnd::Forced,
        ),
        (
            t::source::Range::new(8, 15),
            60.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(15, 16),
            10.0,
            t::selection::LineEnd::Paragraph,
        ),
    ];
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        characterize::<f32>(direction, t::style::Wrap::Word, true, &expected);
        characterize::<f64>(direction, t::style::Wrap::Word, true, &expected);
    }
}
#[test]
fn nowrap_preserves_the_same_real_source_on_one_overflowing_line() {
    let expected = [(
        t::source::Range::new(0, 8),
        80.0,
        t::selection::LineEnd::Paragraph,
    )];
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        characterize::<f32>(direction, t::style::Wrap::None, false, &expected);
        characterize::<f64>(direction, t::style::Wrap::None, false, &expected);
    }
}

fn set_normalized_wrap_style<S: l::scalar::LayoutScalar>(
    fixture: &mut Fixture<S>,
    style: l::text_flow::TextWrapStyle,
) {
    let original = fixture.tree.inputs[0].as_box().cloned().unwrap();
    fixture.tree.inputs[0] = l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
        text_wrap_style: style,
        ..original
    });
    assert_eq!(
        fixture.tree.inputs[0].as_box().unwrap().text_wrap_style,
        style
    );
}

fn assert_real_capacity_and_tab_basis<S: l::scalar::LayoutScalar>(
    fixture: &Fixture<S>,
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    count: usize,
) -> Vec<t::tabs::TabContext> {
    let mut tabs = vec![None; count];
    for fragment in batch.final_inline_fragments() {
        let l::text_layout::InlineFragmentOwner::Node(node) = fragment.owner() else {
            continue;
        };
        if !fixture
            .binding
            .origins
            .iter()
            .any(|origin| origin.node == *node)
        {
            continue;
        }
        let line = fragment.fragment().finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), fixture.prepared);
        let selected = line.selected_line();
        let context = &selected.request().context;
        // A selection goal cannot replace the actual 60px fit/tab/percent frame.
        assert_eq!(context.available_inline, Some(60.0));
        assert_eq!(context.indent, 0.0);
        assert_eq!(
            context.orientation,
            t::selection::LineOrientation::Horizontal
        );
        let slot = &mut tabs[fragment.fragment().line_index()];
        if let Some(previous) = slot {
            assert_eq!(*previous, context.tabs);
        } else {
            *slot = Some(context.tabs.clone());
        }
    }
    tabs.into_iter()
        .map(|basis| basis.expect("actual body receipt supplies each line's tab basis"))
        .collect()
}

fn compute_changed_wrap_style<S: l::scalar::LayoutScalar>(
    fixture: &Fixture<S>,
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    let flow = l::text_layout::LayoutFlowInput::try_new(
        &fixture.tree,
        0,
        vec![fixture.binding.clone()],
        l::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let request = {
        let sizing_request = crate::layout::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::tree::LayoutTree::node_input(&fixture.tree, 0).writing_mode,
                surgeist_layout::tree::LayoutTree::node_input(&fixture.tree, 0).direction,
            ),
            sizing_request
                .orthogonal_fallback()
                .initial_containing_block(),
        )
    };
    let batch = l::compute_layout_with_flow_invalidated(&flow, 0, request, &[0]).unwrap();
    assert!(batch.invalidated_nodes().contains(&0));
    batch
}

fn balance_transition<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    wrap: t::style::Wrap,
    forced_groups: bool,
    auto_expected: &[(t::source::Range, f64, t::selection::LineEnd)],
    balance_expected: &[(t::source::Range, f64, t::selection::LineEnd)],
) {
    let mut fixture = Fixture::<S>::new(direction, wrap, forced_groups);
    assert_eq!(
        fixture.tree.inputs[0].as_box().unwrap().text_wrap_style,
        l::text_flow::TextWrapStyle::Auto
    );
    let preparation = fixture.prepared.clone();
    let capture = Arc::clone(&fixture.capture);
    let binding = fixture.binding.clone();
    let auto = fixture.compute();
    fixture.assert_lines(&auto, auto_expected, direction);
    let actual_tabs = assert_real_capacity_and_tab_basis(&fixture, &auto, auto_expected.len());

    set_normalized_wrap_style(&mut fixture, l::text_flow::TextWrapStyle::Balance);
    assert_eq!(fixture.prepared, preparation);
    assert!(Arc::ptr_eq(&fixture.capture, &capture));
    assert_eq!(fixture.binding, binding);
    let balanced = compute_changed_wrap_style(&fixture);
    fixture.assert_lines(&balanced, balance_expected, direction);
    assert_eq!(
        balanced.placed_text()[0].layout().metrics().line_count(),
        auto.placed_text()[0].layout().metrics().line_count()
    );
    assert_eq!(
        assert_real_capacity_and_tab_basis(&fixture, &balanced, balance_expected.len()),
        actual_tabs
    );
    let balanced_replay = fixture.compute();
    fixture.assert_lines(&balanced_replay, balance_expected, direction);
    assert_eq!(
        balanced.placed_text()[0].layout().key(),
        balanced_replay.placed_text()[0].layout().key()
    );
    if auto_expected != balance_expected {
        assert_ne!(
            auto.placed_text()[0].layout().key(),
            balanced.placed_text()[0].layout().key()
        );
    }
    // Existing completed Auto output is immutable after a different real choice.
    fixture.assert_lines(&auto, auto_expected, direction);

    set_normalized_wrap_style(&mut fixture, l::text_flow::TextWrapStyle::Auto);
    let auto_again = compute_changed_wrap_style(&fixture);
    fixture.assert_lines(&auto_again, auto_expected, direction);
    assert_eq!(
        assert_real_capacity_and_tab_basis(&fixture, &auto_again, auto_expected.len()),
        actual_tabs
    );
    assert_eq!(
        auto.placed_text()[0].layout().key(),
        auto_again.placed_text()[0].layout().key()
    );
    assert_eq!(fixture.prepared, preparation);
    assert!(Arc::ptr_eq(&fixture.capture, &capture));
    assert_eq!(fixture.binding, binding);
    fixture.assert_lines(&balanced, balance_expected, direction);
    fixture.assert_lines(&auto, auto_expected, direction);
}

#[test]
fn balance_uses_two_real_word_lines_with_natural_measures_thirty_and_forty() {
    let auto = [
        (
            t::source::Range::new(0, 7),
            60.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(7, 8),
            10.0,
            t::selection::LineEnd::Paragraph,
        ),
    ];
    let balanced = [
        (
            t::source::Range::new(0, 4),
            30.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(4, 8),
            40.0,
            t::selection::LineEnd::Paragraph,
        ),
    ];
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        balance_transition::<f32>(direction, t::style::Wrap::Word, false, &auto, &balanced);
        balance_transition::<f64>(direction, t::style::Wrap::Word, false, &auto, &balanced);
    }
}

#[test]
fn balance_preserves_each_actual_forced_group_and_its_two_line_count() {
    let auto = [
        (
            t::source::Range::new(0, 7),
            60.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(7, 8),
            10.0,
            t::selection::LineEnd::Forced,
        ),
        (
            t::source::Range::new(8, 15),
            60.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(15, 16),
            10.0,
            t::selection::LineEnd::Paragraph,
        ),
    ];
    let balanced = [
        (
            t::source::Range::new(0, 4),
            30.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(4, 8),
            40.0,
            t::selection::LineEnd::Forced,
        ),
        (
            t::source::Range::new(8, 12),
            30.0,
            t::selection::LineEnd::Soft,
        ),
        (
            t::source::Range::new(12, 16),
            40.0,
            t::selection::LineEnd::Paragraph,
        ),
    ];
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        balance_transition::<f32>(direction, t::style::Wrap::Word, true, &auto, &balanced);
        balance_transition::<f64>(direction, t::style::Wrap::Word, true, &auto, &balanced);
    }
}

#[test]
fn balance_keeps_actual_nowrap_source_on_its_original_overflowing_line() {
    let unchanged = [(
        t::source::Range::new(0, 8),
        80.0,
        t::selection::LineEnd::Paragraph,
    )];
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        balance_transition::<f32>(
            direction,
            t::style::Wrap::None,
            false,
            &unchanged,
            &unchanged,
        );
        balance_transition::<f64>(
            direction,
            t::style::Wrap::None,
            false,
            &unchanged,
            &unchanged,
        );
    }
}

mod advanced;

mod numeric;

mod scope_priority;

mod avoid_short_last_line;

mod pretty;
