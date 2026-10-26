//! Current ordinary wrapping over real captured Ahem source and mandatory groups.
use std::{convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Tree<S: l::LayoutScalar> {
    inputs: Vec<l::LayoutInputOf<S>>,
    non_box: l::NodeInputOf<S>,
    children: Vec<Vec<u32>>,
}
impl<S: l::LayoutScalar> l::Traverse for Tree<S> {
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
impl<S: l::LayoutScalar> l::LayoutTree for Tree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, n: u32) -> &l::NodeInputOf<S> {
        self.inputs[n as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, n: u32) -> l::LayoutInputViewOf<'_, S> {
        self.inputs[n as usize].as_view()
    }
}
struct Fixture<S: l::LayoutScalar> {
    capture: Arc<t::ProcessedSource>,
    prepared: t::PreparedText,
    binding: l::TextParagraphBinding<u32>,
    tree: Tree<S>,
}
impl<S: l::LayoutScalar> Fixture<S> {
    fn new(direction: l::Direction, wrap: t::Wrap, forced_groups: bool) -> Self {
        use t::SourceProcessingEvent as E;
        let raw = if forced_groups {
            "AAA BB CAAA BB C"
        } else {
            "AAA BB C"
        };
        let base = if direction == l::Direction::Ltr {
            t::ParagraphBaseDirection::LeftToRight
        } else {
            t::ParagraphBaseDirection::RightToLeft
        };
        let style = t::Style {
            font: t::Font::new().family("Ahem"),
            size: 10.0,
            line_height: t::LineHeight::Absolute(10.0),
            white_space: t::WhiteSpace::Collapse,
            wrap,
            word_break: t::WordBreak::Normal,
            overflow_wrap: t::OverflowWrap::Normal,
            hyphens: t::Hyphens::None,
            ..t::Style::default()
        };
        let mut source = t::Source::identified(
            raw,
            t::Id::from_u64(if forced_groups { 658401 } else { 658400 }),
            55,
        );
        source.span(t::Range::new(0, raw.len()), style.clone());
        let ranges = if forced_groups {
            vec![t::Range::new(0, 8), t::Range::new(8, 16)]
        } else {
            vec![t::Range::new(0, 8)]
        };
        let mut events = vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::Origin {
                origin: 0,
                affinity: t::Affinity::After,
            },
        ];
        if forced_groups {
            events.extend([
                E::Control {
                    control: 0,
                    affinity: t::Affinity::After,
                },
                E::Connection(0),
                E::Origin {
                    origin: 1,
                    affinity: t::Affinity::After,
                },
            ]);
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                source: source.clone(),
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: ranges
                    .iter()
                    .map(|range| t::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: *range,
                        aggregate: *range,
                        kind: t::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                controls: if forced_groups {
                    vec![t::ParagraphControl {
                        at: 8,
                        kind: t::ParagraphControlKind::ForcedBreak,
                    }]
                } else {
                    vec![]
                },
                connections: if forced_groups {
                    vec![t::SourceProcessingConnection {
                        before_origin: 0,
                        after_origin: 1,
                        kind: t::ParagraphConnectionKind::Continuous,
                    }]
                } else {
                    vec![]
                },
                boundary_policies: if forced_groups {
                    vec![t::ParagraphBoundaryPolicy {
                        definition: t::ParagraphBoundaryDefinition::Connection(0),
                        preceding_wrap: wrap,
                        following_wrap: wrap,
                        ancestor_wrap: wrap,
                    }]
                } else {
                    vec![]
                },
                replacements: vec![],
                computed_line_boundaries: vec![],
                scopes: vec![t::SourceProcessingScope::Block {
                    topology: t::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style,
                    used_direction: base,
                }],
                paragraphs: vec![t::SourceProcessingParagraph {
                    base_direction: base,
                    paragraph_strut: t::ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        let analyzed =
            t::AnalyzedSource::from_processed_paragraph(capture.paragraphs().next().unwrap())
                .unwrap();
        let policies = analyzed
            .wrapping_relations()
            .filter(|r| r.ancestor_wrap().is_none())
            .map(|r| t::WrappingPolicy {
                relation: r.handle(),
                ancestor_wrap: wrap,
            })
            .collect();
        let mut fonts = t::System::new(t::SystemOptions {
            font_catalogue: t::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fonts.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = fonts
            .prepare(
                &analyzed.with_wrapping_policies(policies).unwrap(),
                t::PreparationOptions::default(),
            )
            .unwrap();
        assert_eq!(prepared.captured_paragraph(), capture.captured_paragraph(0));
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
            l::LayoutInputOf::box_input(l::NodeInputOf {
                display: l::Display::Block,
                direction,
                size: l::Size::new(
                    l::PreferredSizeOf::value(
                        l::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                    ),
                    l::PreferredSizeOf::AUTO,
                ),
                ..l::NodeInputOf::default()
            }),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(origin_handles[0].clone())),
        ];
        let mut origins = vec![l::TextOriginBinding {
            origin: origin_handles[0].clone(),
            node: 1,
        }];
        let mut controls = vec![];
        if forced_groups {
            let control = prepared.control_views().next().unwrap().handle();
            assert!(matches!(
                control.view().kind(),
                t::ParagraphControlKind::ForcedBreak
            ));
            assert_eq!(control.captured_control(), capture.captured_control(0));
            inputs.push(l::LayoutInputOf::line_break(
                l::LineBreakInputOf::new()
                    .with_direction(direction)
                    .with_metrics(
                        l::InlineMetricsOf::try_new(S::from_f64(8.0), S::from_f64(10.0)).unwrap(),
                    ),
            ));
            inputs.push(l::LayoutInputOf::inline_text(l::TextParticipation::new(
                origin_handles[1].clone(),
            )));
            origins.push(l::TextOriginBinding {
                origin: origin_handles[1].clone(),
                node: 3,
            });
            controls.push(l::TextControlBinding {
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
        let binding = l::TextParagraphBinding {
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
                non_box: l::NodeInputOf::non_box(),
                children,
            },
        }
    }
    fn compute(&self) -> l::CompletedLayoutBatchOf<u32, S> {
        let flow = l::LayoutFlowInput::try_new(
            &self.tree,
            0,
            vec![self.binding.clone()],
            l::WrappingInput::default(),
        )
        .unwrap();
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(60.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout_with_flow(&flow, 0, request).unwrap()
    }
    fn assert_lines(
        &self,
        batch: &l::CompletedLayoutBatchOf<u32, S>,
        expected: &[(t::Range, f64, t::LineEnd)],
        direction: l::Direction,
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
            Some(t::Id::from_u64(if self.binding.controls.is_empty() {
                658400
            } else {
                658401
            }))
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
            let l::InlineFragmentOwner::Node(node) = fragment.owner() else {
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
                .expect("each actual body line has a genuine source fragment");
            assert_eq!(line.measurements().alignment().advance(), *alignment);
            assert_eq!(line.measurements().fit().advance(), *alignment);
            assert_eq!(line.feedback().advance, *alignment);
            let selected = line.selected_line();
            let request = selected.request();
            assert_eq!(request.context.end, *end);
            assert_eq!(request.start.view().source_after().get(), range.start);
            assert_eq!(request.end.view().source_after().get(), range.end);
            let t::PreparedTermination::Ordinary(offer) = &request.termination else {
                panic!("current ordinary wrapping uses actual ordinary/mandatory capabilities");
            };
            assert_eq!(
                offer.view().kind(),
                match end {
                    t::LineEnd::Soft => t::PreparedBreakKind::Allowed,
                    t::LineEnd::Forced => t::PreparedBreakKind::Forced,
                    t::LineEnd::Paragraph => t::PreparedBreakKind::ParagraphEnd,
                }
            );
            assert!(request.alternative.is_none());
            assert!(request.incoming.is_none());
            let published = layout.lines()[i];
            assert_eq!(published.bounds().size, t::Size::new(60.0, 10.0));
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
            let painted_end = if *end == t::LineEnd::Soft {
                range.end - 1
            } else {
                range.end
            };
            assert_eq!(
                glyph_ranges,
                (range.start..painted_end)
                    .map(|at| t::Range::new(at, at + 1))
                    .collect::<Vec<_>>()
            );
            if *end == t::LineEnd::Forced {
                assert_eq!(request.controls.len(), 1);
                assert_eq!(
                    request.controls[0].occurrence,
                    self.binding.controls[0].occurrence
                );
            }
        }
        let root = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        assert_eq!(
            root.size,
            l::Size::new(S::from_f64(60.0), S::from_f64(10.0 * expected.len() as f64))
        );
        if expected.len() == 2 {
            let last = layout.run_views().find(|run| run.line() == 1).unwrap();
            let x = if direction == l::Direction::Ltr {
                0.0
            } else {
                60.0 - expected[1].1
            };
            let glyph = last.glyphs().next().unwrap();
            assert_eq!(last.origin().x + glyph.pen() + f64::from(glyph.x()), x);
        }
    }
}
fn characterize<S: l::LayoutScalar>(
    direction: l::Direction,
    wrap: t::Wrap,
    forced: bool,
    expected: &[(t::Range, f64, t::LineEnd)],
) {
    let fixture = Fixture::<S>::new(direction, wrap, forced);
    let before = fixture.prepared.captured_paragraph();
    let batch = fixture.compute();
    fixture.assert_lines(&batch, expected, direction);
    let replay = fixture.compute();
    fixture.assert_lines(&replay, expected, direction);
    assert_eq!(
        batch.placed_text()[0].layout().key(),
        replay.placed_text()[0].layout().key()
    );
    assert_eq!(fixture.prepared.captured_paragraph(), before);
    fixture.assert_lines(&batch, expected, direction);
}
#[test]
fn current_ordinary_wrapping_uses_full_fitting_prefix_then_remaining_word() {
    let expected = [
        (t::Range::new(0, 7), 60.0, t::LineEnd::Soft),
        (t::Range::new(7, 8), 10.0, t::LineEnd::Paragraph),
    ];
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        characterize::<f32>(direction, t::Wrap::Word, false, &expected);
        characterize::<f64>(direction, t::Wrap::Word, false, &expected);
    }
}
#[test]
fn actual_forced_control_separates_two_equivalent_ordinary_wrapping_groups() {
    let expected = [
        (t::Range::new(0, 7), 60.0, t::LineEnd::Soft),
        (t::Range::new(7, 8), 10.0, t::LineEnd::Forced),
        (t::Range::new(8, 15), 60.0, t::LineEnd::Soft),
        (t::Range::new(15, 16), 10.0, t::LineEnd::Paragraph),
    ];
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        characterize::<f32>(direction, t::Wrap::Word, true, &expected);
        characterize::<f64>(direction, t::Wrap::Word, true, &expected);
    }
}
#[test]
fn nowrap_preserves_the_same_real_source_on_one_overflowing_line() {
    let expected = [(t::Range::new(0, 8), 80.0, t::LineEnd::Paragraph)];
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        characterize::<f32>(direction, t::Wrap::None, false, &expected);
        characterize::<f64>(direction, t::Wrap::None, false, &expected);
    }
}
