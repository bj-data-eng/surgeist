//! Independently derived natural-width, mandatory-group and real-float controls.
use super::*;
use std::cell::Cell;

#[derive(Clone)]
struct Published<S: l::LayoutScalar> {
    caches: Vec<l::CacheOf<S>>,
    unrounded: Vec<Option<l::NodeOutputOf<S>>>,
    final_output: Vec<Option<l::NodeOutputOf<S>>>,
    batch: Option<l::CompletedLayoutBatchOf<u32, S>>,
}

// A real host cache: only entries issued by a completed public batch are stored.
struct Host<S: l::LayoutScalar> {
    tree: Tree<S>,
    published: Published<S>,
    hits: Vec<Cell<usize>>,
}
impl<S: l::LayoutScalar> l::Traverse for Host<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(&self.tree, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(&self.tree, node, index)
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for Host<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.tree, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.tree, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        let answer = self.published.caches[node as usize].get_with_context(input, context);
        if answer.is_some() {
            self.hits[node as usize].set(self.hits[node as usize].get() + 1);
        }
        answer
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        self.published.unrounded[node as usize]
    }
}
impl<S: l::LayoutScalar> l::LayoutBatchSink<u32, S> for Host<S> {
    type Error = Infallible;
    type Prepared = Published<S>;
    fn prepare_layout_batch(
        &self,
        batch: &l::CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut next = self.published.clone();
        for &node in batch.invalidated_nodes() {
            next.caches[node as usize].clear();
            next.unrounded[node as usize] = None;
            next.final_output[node as usize] = None;
        }
        for entry in batch.cache_clear_entries() {
            next.caches[entry.node() as usize].clear();
        }
        for entry in batch.unrounded_entries() {
            next.unrounded[entry.node() as usize] = Some(entry.output());
        }
        for entry in batch.final_entries() {
            next.final_output[entry.node() as usize] = Some(entry.output());
        }
        for entry in batch.cache_store_entries() {
            next.caches[entry.node() as usize].store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
        next.batch = Some(batch.clone());
        Ok(next)
    }
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.published = prepared;
    }
}

struct Advanced<S: l::LayoutScalar> {
    capture: Arc<t::ProcessedSource>,
    prepared: t::PreparedText,
    binding: l::TextParagraphBinding<u32>,
    host: Host<S>,
    width: f64,
    float: Option<u32>,
}
impl<S: l::LayoutScalar> Advanced<S> {
    fn new(groups: &[&str], width: f64, tall: Option<t::Range>, with_float: bool) -> Self {
        use t::SourceProcessingEvent as E;
        let raw = groups.concat();
        let base = t::Style {
            font: t::Font::new().family("Ahem"),
            size: 10.0,
            line_height: t::LineHeight::Absolute(10.0),
            white_space: t::WhiteSpace::Collapse,
            wrap: t::Wrap::Word,
            word_break: t::WordBreak::Normal,
            overflow_wrap: t::OverflowWrap::Normal,
            hyphens: t::Hyphens::None,
            ..t::Style::default()
        };
        let mut source = t::Source::identified(raw.as_str(), t::Id::from_u64(147104), 3);
        source.span(t::Range::new(0, raw.len()), base.clone());
        if let Some(range) = tall {
            source.span(
                range,
                t::Style {
                    line_height: t::LineHeight::Absolute(30.0),
                    ..base.clone()
                },
            );
        }
        let mut at = 0;
        let ranges = groups
            .iter()
            .map(|group| {
                let range = t::Range::new(at, at + group.len());
                at = range.end;
                range
            })
            .collect::<Vec<_>>();
        let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
        for i in 0..groups.len() {
            if i > 0 {
                events.extend([
                    E::Control {
                        control: i - 1,
                        affinity: t::Affinity::After,
                    },
                    E::Connection(i - 1),
                ]);
            }
            events.push(E::Origin {
                origin: i,
                affinity: t::Affinity::After,
            });
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                source: source.clone(),
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins: ranges
                    .iter()
                    .map(|&range| t::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: range,
                        aggregate: range,
                        kind: t::ParagraphOriginKind::Authored,
                    })
                    .collect(),
                controls: ranges
                    .iter()
                    .take(groups.len() - 1)
                    .map(|range| t::ParagraphControl {
                        at: range.end,
                        kind: t::ParagraphControlKind::ForcedBreak,
                    })
                    .collect(),
                connections: (1..groups.len())
                    .map(|i| t::SourceProcessingConnection {
                        before_origin: i - 1,
                        after_origin: i,
                        kind: t::ParagraphConnectionKind::Continuous,
                    })
                    .collect(),
                boundary_policies: (1..groups.len())
                    .map(|i| t::ParagraphBoundaryPolicy {
                        definition: t::ParagraphBoundaryDefinition::Connection(i - 1),
                        preceding_wrap: t::Wrap::Word,
                        following_wrap: t::Wrap::Word,
                        ancestor_wrap: t::Wrap::Word,
                    })
                    .collect(),
                scopes: vec![t::SourceProcessingScope::Block {
                    topology: t::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: base,
                    used_direction: t::ParagraphBaseDirection::LeftToRight,
                }],
                paragraphs: vec![t::SourceProcessingParagraph {
                    base_direction: t::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: t::ParagraphStrut::DefaultFont,
                }],
                replacements: vec![],
                computed_line_boundaries: vec![],
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
                ancestor_wrap: t::Wrap::Word,
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
        let origin_handles = prepared
            .origin_views()
            .map(|view| view.handle())
            .collect::<Vec<_>>();
        let controls = prepared
            .control_views()
            .map(|view| view.handle())
            .collect::<Vec<_>>();
        let mut inputs = vec![l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            size: l::Size::new(
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(width)).unwrap()),
                l::PreferredSizeOf::AUTO,
            ),
            ..l::NodeInputOf::default()
        })];
        let mut nodes = vec![];
        let mut origins = vec![];
        let mut bindings = vec![];
        for (i, origin) in origin_handles.iter().enumerate() {
            if i > 0 {
                let node = inputs.len() as u32;
                inputs.push(l::LayoutInputOf::line_break(
                    l::LineBreakInputOf::new().with_metrics(
                        l::InlineMetricsOf::try_new(
                            surgeist_layout::InlineBaselineContext::Physical,
                            S::from_f64(8.0),
                            S::from_f64(10.0),
                        )
                        .unwrap(),
                    ),
                ));
                nodes.push(node);
                bindings.push(l::TextControlBinding {
                    occurrence: controls[i - 1].clone(),
                    node,
                });
            }
            let node = inputs.len() as u32;
            inputs.push(l::LayoutInputOf::inline_text(l::TextParticipation::new(
                origin.clone(),
            )));
            nodes.push(node);
            origins.push(l::TextOriginBinding {
                origin: origin.clone(),
                node,
            });
        }
        let mut children = nodes.clone();
        let float = with_float.then(|| {
            let node = inputs.len() as u32;
            inputs.push(l::LayoutInputOf::box_input(l::NodeInputOf {
                display: l::Display::Block,
                float: l::Float::Left,
                size: l::Size::new(
                    l::PreferredSizeOf::value(
                        l::LengthPercentageOf::px(S::from_f64(20.0)).unwrap(),
                    ),
                    l::PreferredSizeOf::value(
                        l::LengthPercentageOf::px(S::from_f64(20.0)).unwrap(),
                    ),
                ),
                ..l::NodeInputOf::default()
            }));
            children.insert(0, node);
            node
        });
        let count = inputs.len();
        let mut child_lists = vec![vec![]; count];
        child_lists[0] = children;
        Self {
            capture,
            prepared: prepared.clone(),
            binding: l::TextParagraphBinding {
                prepared,
                container: 0,
                nodes,
                origins,
                controls: bindings,
                atomics: vec![],
                line_strut: None,
            },
            host: Host {
                tree: Tree {
                    inputs,
                    non_box: l::NodeInputOf::non_box(),
                    children: child_lists,
                },
                published: Published {
                    caches: vec![l::CacheOf::new(); count],
                    unrounded: vec![None; count],
                    final_output: vec![None; count],
                    batch: None,
                },
                hits: (0..count).map(|_| Cell::new(0)).collect(),
            },
            width,
            float,
        }
    }
    fn root(&mut self, change: impl FnOnce(&mut l::NodeInputOf<S>)) {
        let mut root = self.host.tree.inputs[0].as_box().unwrap().clone();
        change(&mut root);
        self.host.tree.inputs[0] = l::LayoutInputOf::box_input(root);
    }
    fn compute(&self, changed: &[u32]) -> l::CompletedLayoutBatchOf<u32, S> {
        let flow = l::LayoutFlowInput::try_new(
            &self.host,
            0,
            vec![self.binding.clone()],
            l::WrappingInput::default(),
        )
        .unwrap();
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(self.width)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout_with_flow_invalidated(&flow, 0, request, changed).unwrap()
    }
    fn receipts(&self, batch: &l::CompletedLayoutBatchOf<u32, S>) -> Vec<t::FinalizedLine> {
        assert_eq!(batch.placed_text().len(), 1);
        let layout = batch.placed_text()[0].layout();
        assert_eq!(layout.source(), self.capture.source());
        assert_eq!(
            layout.analyzed_source().identity(),
            self.prepared.analyzed_source().identity()
        );
        assert_eq!(layout.nonformatted_lines().count(), 0);
        let mut result = vec![None; layout.lines().len()];
        for entry in batch.final_inline_fragments() {
            let l::InlineFragmentOwner::Node(node) = entry.owner() else {
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
            let fragment = entry.fragment();
            let line = fragment.finalized_fragment().finalized_line();
            assert_eq!(line.prepared_text(), self.prepared);
            if let Some(old) = &result[fragment.line_index()] {
                assert_eq!(old, &line);
            }
            result[fragment.line_index()] = Some(line);
        }
        result
            .into_iter()
            .map(|line| line.expect("real body line has a published origin fragment"))
            .collect()
    }
    fn cuts_and_natural(&self, batch: &l::CompletedLayoutBatchOf<u32, S>) -> Vec<(usize, f64)> {
        self.receipts(batch)
            .iter()
            .map(|line| {
                let natural = line
                    .selected_line()
                    .finalize(t::SpacingAllocations::None)
                    .unwrap();
                (
                    line.selected_line()
                        .request()
                        .end
                        .view()
                        .source_after()
                        .get(),
                    natural.measurements().alignment().advance(),
                )
            })
            .collect()
    }
}

fn justify<S: l::LayoutScalar>() {
    let mut fixture = Advanced::<S>::new(&["AA A A A A"], 60.0, None, false);
    fixture.root(|root| {
        root.text_align = l::TextAlign::Justify;
        root.text_align_last = l::TextAlignLast::Justify;
        root.text_justification = l::TextJustification::Spaces;
    });
    let auto = fixture.compute(&[]);
    assert_eq!(fixture.cuts_and_natural(&auto), [(7, 60.0), (10, 30.0)]);
    fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::Balance);
    let balanced = fixture.compute(&[0]);
    assert_eq!(fixture.cuts_and_natural(&balanced), [(5, 40.0), (10, 50.0)]);
    for batch in [&auto, &balanced] {
        for line in fixture.receipts(batch) {
            assert_eq!(line.measurements().alignment().advance(), 60.0);
            assert_eq!(
                line.selected_line().request().context.available_inline,
                Some(60.0)
            );
        }
    }
    // Allocated advances tie at 60/60 in both programs. Natural residuals are
    // Auto 0/30 versus Balance 20/10: variance 225 versus 25.
    assert_eq!(fixture.cuts_and_natural(&auto), [(7, 60.0), (10, 30.0)]);
}
#[test]
fn justification_does_not_erase_the_natural_balance_preference() {
    justify::<f32>();
    justify::<f64>();
}

fn count_profile<S: l::LayoutScalar>() {
    for count in [5, 6, 10, 11] {
        let group = format!("{}C", "AAA BB ".repeat(count - 1));
        let mut fixture = Advanced::<S>::new(&[&group], 60.0, None, false);
        let auto = fixture.compute(&[]);
        let mut auto_expected = (1..count).map(|i| (i * 7, 60.0)).collect::<Vec<_>>();
        auto_expected.push((group.len(), 10.0));
        assert_eq!(fixture.cuts_and_natural(&auto), auto_expected);
        fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::Balance);
        let balanced = fixture.compute(&[0]);
        let expected = if count > 10 {
            auto_expected
        } else {
            let mut expected = vec![(4, 30.0)];
            expected.extend((1..count - 1).map(|i| (4 + i * 7, 60.0)));
            expected.push((group.len(), 40.0));
            expected
        };
        assert_eq!(fixture.cuts_and_natural(&balanced), expected);
        assert_eq!(
            balanced.placed_text()[0].layout().metrics().line_count(),
            count
        );
    }
    // Two independently eligible six-line groups remain eligible in a 12-line run.
    let group = format!("{}C", "AAA BB ".repeat(5));
    let mut fixture = Advanced::<S>::new(&[&group, &group], 60.0, None, false);
    fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::Balance);
    let balanced = fixture.compute(&[]);
    assert_eq!(
        fixture.cuts_and_natural(&balanced),
        [
            (4, 30.0),
            (11, 60.0),
            (18, 60.0),
            (25, 60.0),
            (32, 60.0),
            (36, 40.0),
            (40, 30.0),
            (47, 60.0),
            (54, 60.0),
            (61, 60.0),
            (68, 60.0),
            (72, 40.0),
        ]
    );
    let receipts = fixture.receipts(&balanced);
    assert_eq!(
        receipts[5].selected_line().request().context.end,
        t::LineEnd::Forced
    );
    assert_eq!(
        receipts[11].selected_line().request().context.end,
        t::LineEnd::Paragraph
    );
    assert_eq!(
        receipts[5].selected_line().request().controls[0].occurrence,
        fixture.binding.controls[0].occurrence
    );
}
#[test]
fn five_six_and_ten_line_counts_and_group_local_exemption_are_preserved() {
    count_profile::<f32>();
    count_profile::<f64>();
}

fn assert_float<S: l::LayoutScalar>(
    fixture: &Advanced<S>,
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    balance: bool,
) {
    assert_eq!(
        fixture.cuts_and_natural(batch),
        if balance {
            vec![(4, 30.0), (8, 40.0)]
        } else {
            vec![(7, 60.0), (8, 10.0)]
        }
    );
    let expected = if balance {
        [(60.0, 10.0, 20.0, 0.0, 8.0), (60.0, 30.0, 20.0, 10.0, 28.0)]
    } else {
        [(60.0, 30.0, 20.0, 0.0, 18.0), (80.0, 10.0, 0.0, 30.0, 38.0)]
    };
    for ((line, receipt), (width, height, x, y, baseline)) in batch.placed_text()[0]
        .layout()
        .lines()
        .iter()
        .zip(fixture.receipts(batch))
        .zip(expected)
    {
        assert_eq!(line.bounds(), t::Rect::new(x, y, width, height));
        assert_eq!(line.baseline(), baseline);
        assert_eq!(
            receipt.selected_line().request().context.available_inline,
            Some(width)
        );
    }
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        l::Size::new(S::from_f64(80.0), S::from_f64(40.0))
    );
    let float = batch
        .final_entries()
        .iter()
        .find(|entry| Some(entry.node()) == fixture.float)
        .unwrap()
        .output();
    assert_eq!(float.location, l::Point::new(S::ZERO, S::ZERO));
    assert_eq!(
        float.size,
        l::Size::new(S::from_f64(20.0), S::from_f64(20.0))
    );
}
fn float_replay<S: l::LayoutScalar>() {
    let mut fixture = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
    let auto = fixture.compute(&[]);
    assert_float(&fixture, &auto, false);
    assert!(
        auto.cache_store_entries()
            .iter()
            .any(|entry| Some(entry.node()) == fixture.float)
    );
    auto.apply_to(&mut fixture.host).unwrap();
    fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::Balance);
    let balanced = fixture.compute(&[0]);
    assert_float(&fixture, &balanced, true);
    assert_float(
        &fixture,
        fixture.host.published.batch.as_ref().unwrap(),
        false,
    );
    assert!(balanced.invalidated_nodes().contains(&0));
    balanced.apply_to(&mut fixture.host).unwrap();
    assert_float(
        &fixture,
        fixture.host.published.batch.as_ref().unwrap(),
        true,
    );
    let float = fixture.float.unwrap() as usize;
    let before = fixture.host.hits[float].get();
    let warm = fixture.compute(&[]);
    assert_float(&fixture, &warm, true);
    assert!(
        fixture.host.hits[float].get() > before,
        "warm proof requires an actual eligible float leaf CacheOf answer"
    );
    warm.apply_to(&mut fixture.host).unwrap();
    let mut fresh = Advanced::<S>::new(&["AAA BB C"], 80.0, Some(t::Range::new(4, 6)), true);
    fresh.root(|root| root.text_wrap_style = l::TextWrapStyle::Balance);
    let fresh_batch = fresh.compute(&[]);
    assert_float(&fresh, &fresh_batch, true);
    fixture.root(|root| root.text_wrap_style = l::TextWrapStyle::Auto);
    let restored = fixture.compute(&[0]);
    assert_float(&fixture, &restored, false);
    restored.apply_to(&mut fixture.host).unwrap();
    assert_float(
        &fixture,
        fixture.host.published.batch.as_ref().unwrap(),
        false,
    );
    assert_float(&fixture, &auto, false);
    assert_float(&fixture, &balanced, true);
}
#[test]
fn changed_font_height_replays_its_real_float_bands_through_publication_and_invalidation() {
    float_replay::<f32>();
    float_replay::<f64>();
}
