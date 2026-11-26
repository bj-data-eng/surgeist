//! Independent group alignment through actual capture, preparation and placement.
use std::{cell::Cell, convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/ahem/Ahem.ttf"
));

#[derive(Clone)]
struct Published<S: l::scalar::LayoutScalar> {
    caches: Vec<l::cache::CacheOf<S>>,
    unrounded: Vec<Option<l::batch::LayoutOutputEntryOf<u32, S>>>,
    batch: Option<l::batch::CompletedLayoutBatchOf<u32, S>>,
}
struct Host<S: l::scalar::LayoutScalar> {
    inputs: Vec<l::node::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::node::NodeInputOf<S>,
    published: Published<S>,
    publication_targets: l::batch::LayoutPublicationTargets<u32>,
    hits: Vec<Cell<usize>>,
}
impl<S: l::scalar::LayoutScalar> l::tree::Traverse for Host<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        self.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.children[node as usize][index]
    }
}
impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for Host<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::node::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::cache::ComputeInputOf<S>,
        context: l::cache::CacheKeyContext,
    ) -> Option<l::cache::ComputeOutputOf<S>> {
        let answer = self.published.caches[node as usize].get_with_context(input, context);
        if answer.is_some() {
            self.hits[node as usize].set(self.hits[node as usize].get() + 1);
        }
        answer
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::batch::LayoutOutputEntryOf<u32, S>> {
        self.published.unrounded[node as usize]
    }
}
impl<S: l::scalar::LayoutScalar> l::tree::LayoutBatchSink<u32, S> for Host<S> {
    type Error = Infallible;
    type Prepared = Published<S>;
    fn publication_targets(&self) -> &l::batch::LayoutPublicationTargets<u32> {
        &self.publication_targets
    }
    fn publication_targets_mut(&mut self) -> &mut l::batch::LayoutPublicationTargets<u32> {
        &mut self.publication_targets
    }
    fn prepare_layout_batch<'call>(
        &self,
        preparation: l::batch::LayoutBatchPreparation<'call, u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let batch = preparation.batch();
        let mut next = self.published.clone();
        for &node in preparation
            .previous_members()
            .iter()
            .filter(|&&node| batch.node_layout(node).is_none())
        {
            next.caches[node as usize].clear_performed();
            next.unrounded[node as usize] = None;
        }
        for &node in batch.invalidated_nodes() {
            next.caches[node as usize].clear();
            next.unrounded[node as usize] = None;
        }
        for row in batch.cache_clear_entries() {
            next.caches[row.node() as usize].clear();
        }
        for (node, view) in batch.node_layouts() {
            let entry = match view {
                l::fragmentation::NodeLayoutRefOf::SingleBox(view) => view.unrounded_entry(),
                l::fragmentation::NodeLayoutRefOf::NonBox(view) => view.unrounded_entry(),
                l::fragmentation::NodeLayoutRefOf::Fragments(_) => {
                    panic!("continuous fixture unexpectedly fragmented")
                }
            };
            next.unrounded[node as usize] = Some(entry);
        }
        for row in batch.cache_store_entries() {
            next.caches[row.node() as usize].store_with_context(
                row.input(),
                row.context(),
                row.output(),
            );
        }
        next.batch = Some(batch.clone());
        Ok(next)
    }
    fn commit_layout_batch<'call>(
        &mut self,
        commit: l::batch::LayoutBatchCommit<'call, u32, S, Self::Prepared>,
    ) {
        let prepared = commit.into_prepared();
        self.published = prepared;
    }
}
impl<S: l::scalar::LayoutScalar> Host<S> {
    fn new(direction: l::geometry::Direction, group: l::text_flow::TextGroupAlign) -> Self {
        let mut host = Self {
            inputs: vec![],
            children: vec![],
            non_box: l::node::NodeInputOf::non_box(),
            publication_targets: l::batch::LayoutPublicationTargets::new(),
            published: Published {
                caches: vec![],
                unrounded: vec![],
                batch: None,
            },
            hits: vec![],
        };
        assert_eq!(
            host.push(
                None,
                l::node::LayoutInputOf::box_input(block(direction, group))
            ),
            0
        );
        host
    }
    fn push(&mut self, parent: Option<u32>, input: l::node::LayoutInputOf<S>) -> u32 {
        let node = u32::try_from(self.inputs.len()).unwrap();
        self.inputs.push(input);
        self.children.push(vec![]);
        self.published.caches.push(l::cache::CacheOf::new());
        self.published.unrounded.push(None);
        self.hits.push(Cell::new(0));
        if let Some(parent) = parent {
            self.children[parent as usize].push(node);
        }
        node
    }
    fn box_change(&mut self, node: u32, change: impl FnOnce(&mut l::node::NodeInputOf<S>)) {
        let mut input = self.inputs[node as usize].as_box().unwrap().clone();
        change(&mut input);
        self.inputs[node as usize] = l::node::LayoutInputOf::box_input(input);
    }
    fn child_block(
        &mut self,
        parent: u32,
        direction: l::geometry::Direction,
        group: l::text_flow::TextGroupAlign,
    ) -> u32 {
        self.push(
            Some(parent),
            l::node::LayoutInputOf::box_input(block(direction, group)),
        )
    }
}
fn block<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    group: l::text_flow::TextGroupAlign,
) -> l::node::NodeInputOf<S> {
    l::node::NodeInputOf {
        display: l::box_model::Display::Block,
        direction,
        text_group_align: group,
        size: l::geometry::Size::new(
            l::sizing::PreferredSizeOf::value(
                l::length::LengthPercentageOf::px(S::from_f64(100.0)).unwrap(),
            ),
            l::sizing::PreferredSizeOf::AUTO,
        ),
        ..l::node::NodeInputOf::default()
    }
}
fn base(direction: l::geometry::Direction) -> t::paragraph::ParagraphBaseDirection {
    if direction == l::geometry::Direction::Ltr {
        t::paragraph::ParagraphBaseDirection::LeftToRight
    } else {
        t::paragraph::ParagraphBaseDirection::RightToLeft
    }
}
fn style() -> t::style::Style {
    t::style::Style {
        font: t::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::style::LineHeight::Absolute(10.0),
        white_space: t::style::WhiteSpace::Collapse,
        wrap: t::style::Wrap::Word,
        word_break: t::style::WordBreak::Normal,
        hyphens: t::hyphenation::Hyphens::None,
        ..t::style::Style::default()
    }
}
#[derive(Clone, Copy)]
enum Part<'a> {
    Text(&'a str),
    Forced,
    Atomic(f64),
    Start(f64),
    End(f64),
}

struct Paragraph {
    capture: Arc<t::processing::ProcessedSource>,
    prepared: t::prepared::PreparedText,
    binding: l::text_layout::TextParagraphBinding<u32>,
}
impl Paragraph {
    fn pair<S: l::scalar::LayoutScalar>(host: &mut Host<S>, container: u32) -> Self {
        let direction = host.inputs[container as usize].as_box().unwrap().direction;
        Self::new(
            host,
            container,
            &[Part::Text("AA A A"), Part::Forced, Part::Text("B BB")],
            style(),
            base(direction),
            false,
        )
    }
    fn new<S: l::scalar::LayoutScalar>(
        host: &mut Host<S>,
        container: u32,
        parts: &[Part<'_>],
        style: t::style::Style,
        body: t::paragraph::ParagraphBaseDirection,
        css_tabs: bool,
    ) -> Self {
        let options = if css_tabs {
            t::prepared::PreparationOptions::try_new(1.0, t::tabs::TabPreparation::CssStops)
                .unwrap()
        } else {
            t::prepared::PreparationOptions::default()
        };
        Self::with_preparation(host, container, parts, style, body, options)
    }
    fn with_preparation<S: l::scalar::LayoutScalar>(
        host: &mut Host<S>,
        container: u32,
        parts: &[Part<'_>],
        style: t::style::Style,
        body: t::paragraph::ParagraphBaseDirection,
        options: t::prepared::PreparationOptions,
    ) -> Self {
        use t::capture::SourceProcessingEvent as E;
        let direction = host.inputs[container as usize].as_box().unwrap().direction;
        let writing_mode = host.inputs[container as usize]
            .as_box()
            .unwrap()
            .writing_mode;
        let raw = parts
            .iter()
            .filter_map(|part| {
                if let Part::Text(value) = part {
                    Some(*value)
                } else {
                    None
                }
            })
            .collect::<String>();
        let mut source = t::source::Source::new(raw.clone());
        source.span(
            t::source::Range::new(0, raw.encode_utf16().count()),
            style.clone(),
        );
        let mut origins = Vec::new();
        let mut controls = Vec::new();
        let mut connections = Vec::new();
        let mut policies = Vec::new();
        let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
        let mut nodes = Vec::new();
        let mut origin_nodes = Vec::new();
        let mut control_nodes = Vec::new();
        let mut atomic_nodes = Vec::new();
        let mut at = 0;
        let mut previous_origin = None;
        let mut boundary_start = None;
        let mut boundary_pair = None;
        let mut inputs = Vec::new();
        for part in parts {
            let node = u32::try_from(host.inputs.len() + nodes.len()).unwrap();
            let input = match *part {
                Part::Text(value) => {
                    let range = t::source::Range::new(at, at + value.encode_utf16().count());
                    let origin = origins.len();
                    if let Some(previous) = previous_origin {
                        let connection = connections.len();
                        connections.push(t::capture::SourceProcessingConnection {
                            before_origin: previous,
                            after_origin: origin,
                            kind: t::paragraph::ParagraphConnectionKind::Continuous,
                        });
                        policies.push(t::paragraph::ParagraphBoundaryPolicy {
                            definition: t::paragraph::ParagraphBoundaryDefinition::Connection(
                                connection,
                            ),
                            preceding_wrap: style.wrap,
                            following_wrap: style.wrap,
                            ancestor_wrap: style.wrap,
                        });
                        events.push(E::Connection(connection));
                    }
                    origins.push(t::paragraph::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: range,
                        aggregate: range,
                        kind: t::paragraph::ParagraphOriginKind::Authored,
                    });
                    events.push(E::Origin {
                        origin,
                        affinity: t::layout::Affinity::After,
                    });
                    at = range.end;
                    previous_origin = Some(origin);
                    origin_nodes.push(node);
                    None
                }
                Part::Forced | Part::Start(_) | Part::End(_) => {
                    let kind = match part {
                        Part::Forced => t::paragraph::ParagraphControlKind::ForcedBreak,
                        Part::Start(_) => t::paragraph::ParagraphControlKind::InlineBoundaryStart,
                        Part::End(_) => t::paragraph::ParagraphControlKind::InlineBoundaryEnd,
                        _ => unreachable!(),
                    };
                    let control = controls.len();
                    controls.push(t::paragraph::ParagraphControl { at, kind });
                    if matches!(part, Part::Start(_)) {
                        events.push(E::EnterScope(1));
                        boundary_start = Some(control);
                    }
                    events.push(E::Control {
                        control,
                        affinity: if matches!(part, Part::End(_)) {
                            t::layout::Affinity::Before
                        } else {
                            t::layout::Affinity::After
                        },
                    });
                    let metrics = l::inline::InlineMetricsOf::try_new(
                        surgeist_layout::baseline::InlineBaselineContext::Physical,
                        S::from_f64(8.0),
                        S::from_f64(10.0),
                    )
                    .unwrap();
                    let input = match *part {
                        Part::Forced => l::node::LayoutInputOf::line_break(
                            l::text_flow::LineBreakInputOf::new()
                                .with_writing_mode(writing_mode)
                                .with_direction(direction)
                                .with_metrics(metrics),
                        ),
                        Part::Start(edge) | Part::End(edge) => {
                            let kind = if matches!(part, Part::Start(_)) {
                                l::inline::InlineBoundaryKind::Start
                            } else {
                                l::inline::InlineBoundaryKind::End
                            };
                            l::node::LayoutInputOf::inline_boundary(
                                l::inline::InlineBoundaryInputOf::contribution(
                                    kind,
                                    metrics,
                                    l::inline::InlineBoundaryEdgeOf::try_new(
                                        S::ZERO,
                                        S::from_f64(edge),
                                        S::ZERO,
                                    )
                                    .unwrap(),
                                )
                                .unwrap()
                                .with_writing_mode(writing_mode)
                                .with_direction(direction),
                            )
                        }
                        _ => unreachable!(),
                    };
                    control_nodes.push(node);
                    if matches!(part, Part::End(_)) {
                        boundary_pair = Some(t::capture::SourceProcessingScopeBoundaries {
                            start: boundary_start.take().unwrap(),
                            end: control,
                        });
                        events.push(E::ExitScope(1));
                    }
                    Some(input)
                }
                Part::Atomic(width) => {
                    let input = l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
                        display: l::box_model::Display::InlineBlock,
                        size: l::geometry::Size::new(
                            l::sizing::PreferredSizeOf::value(
                                l::length::LengthPercentageOf::px(S::from_f64(width)).unwrap(),
                            ),
                            l::sizing::PreferredSizeOf::value(
                                l::length::LengthPercentageOf::px(S::from_f64(10.0)).unwrap(),
                            ),
                        ),
                        ..l::node::NodeInputOf::default()
                    });
                    let atomic = atomic_nodes.len();
                    // This ID is the real authored fixture box's host identity;
                    // every prepared/captured capability is issued by Text below.
                    source.inline_box(t::source::InlineBox::new(
                        t::source::Id::from_u64(u64::from(node)),
                        t::source::InlineBoxKind::InFlow,
                        at,
                        t::geometry::Size::new(width, 10.0),
                    ));
                    events.push(E::Atomic {
                        atomic,
                        affinity: t::layout::Affinity::After,
                    });
                    atomic_nodes.push(node);
                    for definition in [
                        t::paragraph::ParagraphBoundaryDefinition::AtomicBefore(atomic),
                        t::paragraph::ParagraphBoundaryDefinition::AtomicAfter(atomic),
                    ] {
                        policies.push(t::paragraph::ParagraphBoundaryPolicy {
                            definition,
                            preceding_wrap: style.wrap,
                            following_wrap: style.wrap,
                            ancestor_wrap: style.wrap,
                        });
                    }
                    Some(input)
                }
            };
            nodes.push(node);
            inputs.push(input);
        }
        events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
        let mut scopes = vec![t::capture::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: t::capture::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style.clone(),
            used_direction: base(direction),
        }];
        if let Some(boundaries) = boundary_pair {
            scopes.push(t::capture::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: Default::default(),
                style: None,
                boundaries: Some(boundaries),
            });
        }
        let capture = Arc::new(
            t::capture::process_contextual_source(t::capture::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source,
                segment_breaks: t::processing::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::capture::SourceProcessingEnvelope::Complete,
                origins,
                controls,
                connections,
                boundary_policies: policies,
                replacements: vec![],
                computed_line_boundaries: vec![],
                scopes,
                paragraphs: vec![t::capture::SourceProcessingParagraph {
                    base_direction: body,
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
        let wrapping = analyzed
            .wrapping_relations()
            .filter(|row| row.ancestor_wrap().is_none())
            .map(|row| t::analyzed::WrappingPolicy {
                relation: row.handle(),
                ancestor_wrap: style.wrap,
            })
            .collect();
        let mut system = t::System::default();
        system
            .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
            .unwrap();
        let prepared = system
            .prepare(&analyzed.with_wrapping_policies(wrapping).unwrap(), options)
            .unwrap();
        assert_eq!(
            prepared
                .captured_owner()
                .unwrap()
                .members()
                .collect::<Vec<_>>(),
            vec![capture.captured_paragraph(0).unwrap()]
        );
        assert_eq!(prepared.origin_views().count(), origin_nodes.len());
        assert_eq!(prepared.control_views().count(), control_nodes.len());
        assert_eq!(prepared.atomic_views().count(), atomic_nodes.len());
        // Insert only final roles. Text nodes receive their issued
        // participation after preparation; no dummy box input is ever queried.
        {
            let mut issued_origins = prepared.origin_views();
            for (&node, input) in nodes.iter().zip(inputs) {
                let input = input.unwrap_or_else(|| {
                    l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                        issued_origins.next().unwrap().handle(),
                    ))
                });
                assert_eq!(host.push(Some(container), input), node);
            }
            assert!(issued_origins.next().is_none());
        }
        let binding = l::text_layout::TextParagraphBinding {
            inline_box_fragments:
                surgeist_layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: prepared.clone(),
            container,
            nodes,
            origins: prepared
                .origin_views()
                .zip(origin_nodes)
                .map(|(origin, node)| l::text_layout::TextOriginBinding {
                    origin: origin.handle(),
                    node,
                })
                .collect(),
            controls: prepared
                .control_views()
                .zip(control_nodes)
                .map(|(control, node)| l::text_layout::TextControlBinding {
                    occurrence: control.handle(),
                    node,
                })
                .collect(),
            atomics: prepared
                .atomic_views()
                .zip(atomic_nodes)
                .map(|(atomic, node)| l::text_layout::TextAtomicBinding {
                    occurrence: atomic.handle(),
                    node,
                })
                .collect(),
            line_strut: None,
        };
        Self {
            capture,
            prepared,
            binding,
        }
    }
}
fn compute<S: l::scalar::LayoutScalar>(
    host: &Host<S>,
    paragraphs: &[&Paragraph],
    changed: &[u32],
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    let flow = l::text_layout::LayoutFlowInput::try_new(
        host,
        0,
        paragraphs
            .iter()
            .map(|paragraph| paragraph.binding.clone())
            .collect(),
        l::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let request = {
        let sizing_request = crate::layout::native_request::sizing(
            surgeist_layout::geometry::Size::new(Some(S::from_f64(100.0)), None),
            surgeist_layout::geometry::Size::new(
                surgeist_layout::constraint::SizingOperation::Normal,
                surgeist_layout::constraint::SizingOperation::Normal,
            ),
        );
        l::request::LayoutRootRequestOf::standalone(
            sizing_request,
            surgeist_layout::geometry::FlowAxes::new(
                surgeist_layout::tree::LayoutTree::node_input(host, 0).writing_mode,
                surgeist_layout::tree::LayoutTree::node_input(host, 0).direction,
            ),
            sizing_request
                .orthogonal_fallback()
                .initial_containing_block(),
        )
    };
    l::compute_layout_with_flow_invalidated(&flow, 0, request, changed).unwrap()
}
fn placed<'a, S: l::scalar::LayoutScalar>(
    batch: &'a l::batch::CompletedLayoutBatchOf<u32, S>,
    paragraph: &Paragraph,
) -> &'a t::layout::Layout {
    let entry = batch
        .placed_text()
        .iter()
        .find(|entry| {
            entry.container() == paragraph.binding.container
                && entry.layout().analyzed_source().identity()
                    == paragraph.prepared.analyzed_source().identity()
        })
        .unwrap();
    assert_eq!(entry.nodes(), paragraph.binding.nodes.as_slice());
    assert_eq!(entry.layout().source(), paragraph.capture.source());
    entry.layout()
}
fn receipts<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    paragraph: &Paragraph,
) -> Vec<t::finalized::FinalizedLine> {
    let layout = placed(batch, paragraph);
    let mut lines = vec![None; layout.lines().len()];
    for row in batch.final_inline_fragments() {
        if !matches!(row.owner(), l::text_layout::InlineFragmentOwner::Node(node) if paragraph.binding.origins.iter().any(|origin| origin.node == *node))
        {
            continue;
        }
        let line = row.fragment().finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), paragraph.prepared);
        let index = row.fragment().line_index();
        if let Some(previous) = &lines[index] {
            assert_eq!(previous, &line);
        }
        lines[index] = Some(line);
    }
    lines
        .into_iter()
        .map(|line| line.expect("every tested formatted line has an origin, including EmptyOrigin"))
        .collect()
}
fn glyph_extents(layout: &t::layout::Layout) -> Vec<(f64, f64)> {
    let mut extents = vec![(f64::INFINITY, f64::NEG_INFINITY); layout.lines().len()];
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs().filter(|glyph| glyph.advance() > 0.0) {
            let x = run.origin().x + glyph.pen() + f64::from(glyph.x());
            extents[run.line()].0 = extents[run.line()].0.min(x);
            extents[run.line()].1 = extents[run.line()].1.max(x + f64::from(glyph.advance()));
        }
    }
    extents
}
fn node_output<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe((batch, true), node, crate::layout::observation::Kind::Box)
}
fn node_output_of_kind<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    node: u32,
    expected_kind: crate::layout::observation::Kind,
) -> crate::layout::observation::Observation<S> {
    crate::layout::observation::observe((batch, true), node, expected_kind)
}

mod geometry;
mod hierarchy;
mod shared_floats;
mod sideways;
