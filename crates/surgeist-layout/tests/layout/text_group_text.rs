//! Independent group alignment through actual capture, preparation and placement.
use std::{cell::Cell, convert::Infallible, slice, sync::Arc};
use surgeist_layout as l;
use surgeist_text as t;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
));

#[derive(Clone)]
struct Published<S: l::LayoutScalar> {
    caches: Vec<l::CacheOf<S>>,
    unrounded: Vec<Option<l::NodeOutputOf<S>>>,
    batch: Option<l::CompletedLayoutBatchOf<u32, S>>,
}
struct Host<S: l::LayoutScalar> {
    inputs: Vec<l::LayoutInputOf<S>>,
    children: Vec<Vec<u32>>,
    non_box: l::NodeInputOf<S>,
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
        self.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.children[node as usize][index]
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for Host<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
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
        }
        for row in batch.cache_clear_entries() {
            next.caches[row.node() as usize].clear();
        }
        for row in batch.unrounded_entries() {
            next.unrounded[row.node() as usize] = Some(row.output());
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
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.published = prepared;
    }
}
impl<S: l::LayoutScalar> Host<S> {
    fn new(direction: l::Direction, group: l::TextGroupAlign) -> Self {
        let mut host = Self {
            inputs: vec![],
            children: vec![],
            non_box: l::NodeInputOf::non_box(),
            published: Published {
                caches: vec![],
                unrounded: vec![],
                batch: None,
            },
            hits: vec![],
        };
        assert_eq!(
            host.push(None, l::LayoutInputOf::box_input(block(direction, group))),
            0
        );
        host
    }
    fn push(&mut self, parent: Option<u32>, input: l::LayoutInputOf<S>) -> u32 {
        let node = u32::try_from(self.inputs.len()).unwrap();
        self.inputs.push(input);
        self.children.push(vec![]);
        self.published.caches.push(l::CacheOf::new());
        self.published.unrounded.push(None);
        self.hits.push(Cell::new(0));
        if let Some(parent) = parent {
            self.children[parent as usize].push(node);
        }
        node
    }
    fn box_change(&mut self, node: u32, change: impl FnOnce(&mut l::NodeInputOf<S>)) {
        let mut input = self.inputs[node as usize].as_box().unwrap().clone();
        change(&mut input);
        self.inputs[node as usize] = l::LayoutInputOf::box_input(input);
    }
    fn child_block(
        &mut self,
        parent: u32,
        direction: l::Direction,
        group: l::TextGroupAlign,
    ) -> u32 {
        self.push(
            Some(parent),
            l::LayoutInputOf::box_input(block(direction, group)),
        )
    }
}
fn block<S: l::LayoutScalar>(
    direction: l::Direction,
    group: l::TextGroupAlign,
) -> l::NodeInputOf<S> {
    l::NodeInputOf {
        display: l::Display::Block,
        direction,
        text_group_align: group,
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(100.0)).unwrap()),
            l::PreferredSizeOf::AUTO,
        ),
        ..l::NodeInputOf::default()
    }
}
fn base(direction: l::Direction) -> t::ParagraphBaseDirection {
    if direction == l::Direction::Ltr {
        t::ParagraphBaseDirection::LeftToRight
    } else {
        t::ParagraphBaseDirection::RightToLeft
    }
}
fn style() -> t::Style {
    t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        white_space: t::WhiteSpace::Collapse,
        wrap: t::Wrap::Word,
        word_break: t::WordBreak::Normal,
        hyphens: t::Hyphens::None,
        ..t::Style::default()
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
    capture: Arc<t::ProcessedSource>,
    prepared: t::PreparedText,
    binding: l::TextParagraphBinding<u32>,
}
impl Paragraph {
    fn pair<S: l::LayoutScalar>(host: &mut Host<S>, container: u32) -> Self {
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
    fn new<S: l::LayoutScalar>(
        host: &mut Host<S>,
        container: u32,
        parts: &[Part<'_>],
        style: t::Style,
        body: t::ParagraphBaseDirection,
        css_tabs: bool,
    ) -> Self {
        let options = if css_tabs {
            t::PreparationOptions::try_new(1.0, t::TabPreparation::CssStops).unwrap()
        } else {
            t::PreparationOptions::default()
        };
        Self::with_preparation(host, container, parts, style, body, options)
    }
    fn with_preparation<S: l::LayoutScalar>(
        host: &mut Host<S>,
        container: u32,
        parts: &[Part<'_>],
        style: t::Style,
        body: t::ParagraphBaseDirection,
        options: t::PreparationOptions,
    ) -> Self {
        use t::SourceProcessingEvent as E;
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
        let mut source = t::Source::new(raw.clone());
        source.span(t::Range::new(0, raw.encode_utf16().count()), style.clone());
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
                    let range = t::Range::new(at, at + value.encode_utf16().count());
                    let origin = origins.len();
                    if let Some(previous) = previous_origin {
                        let connection = connections.len();
                        connections.push(t::SourceProcessingConnection {
                            before_origin: previous,
                            after_origin: origin,
                            kind: t::ParagraphConnectionKind::Continuous,
                        });
                        policies.push(t::ParagraphBoundaryPolicy {
                            definition: t::ParagraphBoundaryDefinition::Connection(connection),
                            preceding_wrap: style.wrap,
                            following_wrap: style.wrap,
                            ancestor_wrap: style.wrap,
                        });
                        events.push(E::Connection(connection));
                    }
                    origins.push(t::ParagraphOrigin {
                        text_node: None,
                        source: source.clone(),
                        local: range,
                        aggregate: range,
                        kind: t::ParagraphOriginKind::Authored,
                    });
                    events.push(E::Origin {
                        origin,
                        affinity: t::Affinity::After,
                    });
                    at = range.end;
                    previous_origin = Some(origin);
                    origin_nodes.push(node);
                    None
                }
                Part::Forced | Part::Start(_) | Part::End(_) => {
                    let kind = match part {
                        Part::Forced => t::ParagraphControlKind::ForcedBreak,
                        Part::Start(_) => t::ParagraphControlKind::InlineBoundaryStart,
                        Part::End(_) => t::ParagraphControlKind::InlineBoundaryEnd,
                        _ => unreachable!(),
                    };
                    let control = controls.len();
                    controls.push(t::ParagraphControl { at, kind });
                    if matches!(part, Part::Start(_)) {
                        events.push(E::EnterScope(1));
                        boundary_start = Some(control);
                    }
                    events.push(E::Control {
                        control,
                        affinity: if matches!(part, Part::End(_)) {
                            t::Affinity::Before
                        } else {
                            t::Affinity::After
                        },
                    });
                    let metrics = l::InlineMetricsOf::try_new(
                        surgeist_layout::InlineBaselineContext::Physical,
                        S::from_f64(8.0),
                        S::from_f64(10.0),
                    )
                    .unwrap();
                    let input = match *part {
                        Part::Forced => l::LayoutInputOf::line_break(
                            l::LineBreakInputOf::new()
                                .with_writing_mode(writing_mode)
                                .with_direction(direction)
                                .with_metrics(metrics),
                        ),
                        Part::Start(edge) | Part::End(edge) => {
                            let kind = if matches!(part, Part::Start(_)) {
                                l::InlineBoundaryKind::Start
                            } else {
                                l::InlineBoundaryKind::End
                            };
                            l::LayoutInputOf::inline_boundary(
                                l::InlineBoundaryInputOf::new(
                                    kind,
                                    metrics,
                                    l::InlineBoundaryEdgeOf::try_new(S::from_f64(edge), S::ZERO)
                                        .unwrap(),
                                )
                                .with_writing_mode(writing_mode)
                                .with_direction(direction),
                            )
                        }
                        _ => unreachable!(),
                    };
                    control_nodes.push(node);
                    if matches!(part, Part::End(_)) {
                        boundary_pair = Some(t::SourceProcessingScopeBoundaries {
                            start: boundary_start.take().unwrap(),
                            end: control,
                        });
                        events.push(E::ExitScope(1));
                    }
                    Some(input)
                }
                Part::Atomic(width) => {
                    let input = l::LayoutInputOf::box_input(l::NodeInputOf {
                        display: l::Display::InlineBlock,
                        size: l::Size::new(
                            l::PreferredSizeOf::value(
                                l::LengthPercentageOf::px(S::from_f64(width)).unwrap(),
                            ),
                            l::PreferredSizeOf::value(
                                l::LengthPercentageOf::px(S::from_f64(10.0)).unwrap(),
                            ),
                        ),
                        ..l::NodeInputOf::default()
                    });
                    let atomic = atomic_nodes.len();
                    // This ID is the real authored fixture box's host identity;
                    // every prepared/captured capability is issued by Text below.
                    source.inline_box(t::InlineBox::new(
                        t::Id::from_u64(u64::from(node)),
                        t::InlineBoxKind::InFlow,
                        at,
                        t::Size::new(width, 10.0),
                    ));
                    events.push(E::Atomic {
                        atomic,
                        affinity: t::Affinity::After,
                    });
                    atomic_nodes.push(node);
                    for definition in [
                        t::ParagraphBoundaryDefinition::AtomicBefore(atomic),
                        t::ParagraphBoundaryDefinition::AtomicAfter(atomic),
                    ] {
                        policies.push(t::ParagraphBoundaryPolicy {
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
        let mut scopes = vec![t::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: t::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style.clone(),
            used_direction: base(direction),
        }];
        if let Some(boundaries) = boundary_pair {
            scopes.push(t::SourceProcessingScope::Inline {
                horizontal_inner_direction: None,
                alignment_boundary: None,
                font_reference: None,
                trim: Default::default(),
                style: None,
                boundaries: Some(boundaries),
            });
        }
        let capture = Arc::new(
            t::process_contextual_source(t::ContextualSourceInput {
                joint_flows: Vec::new(),
                scope_alignment_limits: None,
                source,
                segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
                envelope: t::SourceProcessingEnvelope::Complete,
                origins,
                controls,
                connections,
                boundary_policies: policies,
                replacements: vec![],
                computed_line_boundaries: vec![],
                scopes,
                paragraphs: vec![t::SourceProcessingParagraph {
                    base_direction: body,
                    paragraph_strut: t::ParagraphStrut::DefaultFont,
                }],
                events,
            })
            .unwrap(),
        );
        let analyzed = t::AnalyzedSource::from_processed_paragraph(
            capture.paragraphs().next().unwrap(),
            t::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let wrapping = analyzed
            .wrapping_relations()
            .filter(|row| row.ancestor_wrap().is_none())
            .map(|row| t::WrappingPolicy {
                relation: row.handle(),
                ancestor_wrap: style.wrap,
            })
            .collect();
        let mut system = t::System::new(t::SystemOptions {
            font_catalogue: t::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
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
        // Insert only genuine final roles. Text nodes receive their issued
        // participation after preparation; no dummy box input is ever queried.
        {
            let mut issued_origins = prepared.origin_views();
            for (&node, input) in nodes.iter().zip(inputs) {
                let input = input.unwrap_or_else(|| {
                    l::LayoutInputOf::inline_text(l::TextParticipation::new(
                        issued_origins.next().unwrap().handle(),
                    ))
                });
                assert_eq!(host.push(Some(container), input), node);
            }
            assert!(issued_origins.next().is_none());
        }
        let binding = l::TextParagraphBinding {
            prepared: prepared.clone(),
            container,
            nodes,
            origins: prepared
                .origin_views()
                .zip(origin_nodes)
                .map(|(origin, node)| l::TextOriginBinding {
                    origin: origin.handle(),
                    node,
                })
                .collect(),
            controls: prepared
                .control_views()
                .zip(control_nodes)
                .map(|(control, node)| l::TextControlBinding {
                    occurrence: control.handle(),
                    node,
                })
                .collect(),
            atomics: prepared
                .atomic_views()
                .zip(atomic_nodes)
                .map(|(atomic, node)| l::TextAtomicBinding {
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
fn compute<S: l::LayoutScalar>(
    host: &Host<S>,
    paragraphs: &[&Paragraph],
    changed: &[u32],
) -> l::CompletedLayoutBatchOf<u32, S> {
    let flow = l::LayoutFlowInput::try_new(
        host,
        0,
        paragraphs
            .iter()
            .map(|paragraph| paragraph.binding.clone())
            .collect(),
        l::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let request = l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(S::from_f64(100.0)),
        l::AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    l::compute_layout_with_flow_invalidated(&flow, 0, request, changed).unwrap()
}
fn placed<'a, S: l::LayoutScalar>(
    batch: &'a l::CompletedLayoutBatchOf<u32, S>,
    paragraph: &Paragraph,
) -> &'a t::Layout {
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
fn receipts<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    paragraph: &Paragraph,
) -> Vec<t::FinalizedLine> {
    let layout = placed(batch, paragraph);
    let mut lines = vec![None; layout.lines().len()];
    for row in batch.final_inline_fragments() {
        if !matches!(row.owner(), l::InlineFragmentOwner::Node(node) if paragraph.binding.origins.iter().any(|origin| origin.node == *node))
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
        .map(|line| {
            line.expect("every tested formatted line has a genuine origin, including EmptyOrigin")
        })
        .collect()
}
fn glyph_extents(layout: &t::Layout) -> Vec<(f64, f64)> {
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
fn node_output<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> l::NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|row| row.node() == node)
        .unwrap()
        .output()
}

mod geometry;
mod hierarchy;
mod shared_floats;
mod sideways;
