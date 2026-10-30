//! Genuine native controls, nonformatted receipts and physical float constraints.
use super::*;
use std::cell::RefCell;

fn central() -> l::InlineBaselineContext {
    l::InlineBaselineContext::Named(l::BaselineName::Central)
}

struct Native {
    prepared: t::PreparedText,
    source: t::Source,
}

fn native(raw: &str, height: f64, forced: bool, whitespace: t::WhiteSpace) -> Native {
    let bytes = chi_font();
    qualify_font(&bytes);
    let source = t::Source::new(raw);
    let len = raw.encode_utf16().count();
    let style = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 4.0,
        line_height: t::LineHeight::Absolute(height),
        white_space: whitespace,
        wrap: t::Wrap::Word,
        ..t::Style::default()
    };
    use t::SourceProcessingEvent as E;
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: t::Affinity::After,
        },
    ];
    let controls = if forced {
        events.push(E::Control {
            control: 0,
            affinity: t::Affinity::After,
        });
        vec![t::ParagraphControl {
            at: len,
            kind: t::ParagraphControlKind::ForcedBreak,
        }]
    } else {
        vec![]
    };
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let capture = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: vec![t::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: t::Range::new(0, len),
                aggregate: t::Range::new(0, len),
                kind: t::ParagraphOriginKind::Authored,
            }],
            controls,
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![t::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: t::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: t::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![t::SourceProcessingParagraph {
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: t::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let typography = t::AnalysisTypography::Upright {
        computed_direction: t::ParagraphBaseDirection::RightToLeft,
    };
    let analyzed = t::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        typography,
    )
    .unwrap();
    assert_eq!(analyzed.typography(), typography);
    assert_eq!(analyzed.direction(), t::Direction::LeftToRight);
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|r| r.ancestor_wrap().is_none())
        .map(|r| t::WrappingPolicy {
            relation: r.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(wrapping).unwrap();
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            t::PreparationOptions::default().with_orientation(t::LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.captured_paragraph(), capture.captured_paragraph(0));
    assert_eq!(prepared.analyzed_source().source(), &source);
    assert_eq!(prepared.control_views().count(), usize::from(forced));
    assert_eq!(prepared.atomic_views().count(), 0);
    Native { prepared, source }
}

fn control_allocation(control: t::PreparedControl) -> t::ControlAllocation {
    t::ControlAllocation {
        occurrence: control,
        boundary: None,
        strut: Some(t::ControlStrut {
            baseline: 8.0,
            line_extent: 10.0,
            alignment: t::LineBoxAlignment::Baseline,
        }),
    }
}

fn qualify(native: &Native, expected: (f64, f64, f64)) {
    let prepared = &native.prepared;
    let forced = prepared
        .opportunity_views()
        .find(|o| o.kind() == t::PreparedBreakKind::Forced)
        .map(|o| o.handle());
    let end = forced.clone().unwrap_or_else(|| {
        prepared
            .opportunity_views()
            .find(|o| o.kind() == t::PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle()
    });
    let boundary = end.view().boundary();
    let to = boundary.view().participant_offset();
    let controls = prepared
        .control_views()
        .map(|c| control_allocation(c.handle()))
        .collect();
    let line = prepared
        .select(t::LineSelectionRequest {
            start: prepared.boundary_views().next().unwrap().handle(),
            end: boundary,
            participants: prepared.participants().take(to).collect(),
            termination: t::PreparedTermination::Ordinary(end),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls,
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: t::LineContext {
                alignment_insets: t::LineAlignmentInsets::ZERO,
                formatted_line: t::FormattedLineContext::default(),
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::LineBoxDirection::LeftToRight,
                orientation: t::LineOrientation::Vertical,
                start: t::LineStart::Paragraph,
                end: if forced.is_some() {
                    t::LineEnd::Forced
                } else {
                    t::LineEnd::Paragraph
                },
                available_inline: None,
                indent: 0.0,
                alignment: t::Alignment::Start,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::LineAllocations {
            spacing: t::SpacingAllocations::None,
            scopes: surgeist_text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (
            line.feedback().baseline,
            line.feedback().line_height,
            line.feedback().advance
        ),
        expected
    );
    assert_eq!(line.prepared_text(), *prepared);
    assert_eq!(line.selected_line().request().line_strut, None);
    if expected.2 == 0.0 {
        assert!(!line.has_in_flow_content());
        assert!(line.nonformatted().is_ok());
    }
}

fn host<S: l::LayoutScalar>(mode: l::WritingMode, inline: f64) -> Host<S> {
    let mut input = root(mode, l::Direction::Ltr, central());
    input.size = l::Size::new(px(40.0), px(inline));
    Host::new(input)
}

fn attach<S: l::LayoutScalar>(
    host: &mut Host<S>,
    native: &Native,
    mode: l::WritingMode,
) -> l::TextParagraphBinding<u32> {
    let origin = native.prepared.origin_views().next().unwrap().handle();
    let text = host.push(
        Some(0),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone())),
    );
    let mut nodes = vec![text];
    let mut controls = Vec::new();
    for control in native.prepared.control_views() {
        let node = host.push(
            Some(0),
            l::LayoutInputOf::line_break(
                l::LineBreakInputOf::new()
                    .with_writing_mode(mode)
                    .with_direction(l::Direction::Ltr)
                    .with_metrics(l::InlineMetricsOf::try_new(central(), n(8.0), n(10.0)).unwrap()),
            ),
        );
        nodes.push(node);
        controls.push(l::TextControlBinding {
            occurrence: control.handle(),
            node,
        });
    }
    l::TextParagraphBinding {
        prepared: native.prepared.clone(),
        container: 0,
        nodes,
        origins: vec![l::TextOriginBinding { origin, node: text }],
        controls,
        atomics: vec![],
        line_strut: None,
    }
}

fn request<S: l::LayoutScalar>(inline: f64) -> l::LayoutRootRequestOf<S> {
    l::LayoutRootRequestOf::viewport(l::Size::new(
        l::AvailableOf::definite(n(40.0)),
        l::AvailableOf::definite(n(inline)),
    ))
    .unwrap()
}

fn compute<S: l::LayoutScalar>(
    host: &Host<S>,
    binding: &l::TextParagraphBinding<u32>,
    inline: f64,
) -> l::CompletedLayoutBatchOf<u32, S> {
    let flow = l::LayoutFlowInput::try_new(
        host,
        0,
        vec![binding.clone()],
        l::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    l::compute_layout_with_flow(&flow, 0, request(inline)).unwrap()
}

fn placed<S: l::LayoutScalar>(batch: &l::CompletedLayoutBatchOf<u32, S>) -> &t::Layout {
    assert_eq!(batch.placed_text().len(), 1);
    batch.placed_text()[0].layout()
}

fn central_pair<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
) -> l::BaselinePairOf<S> {
    node(batch.final_entries(), 0)
        .named_baselines
        .get(l::BaselineName::Central)
}

fn controls_and_completion<S: l::LayoutScalar>() {
    let native = native("\u{4e00}", 4.0, true, t::WhiteSpace::Preserve);
    qualify(&native, (8.0, 10.0, 4.0));
    for (mode, center, completion_x) in [
        (l::WritingMode::VerticalRl, 32.0, 30.0),
        (l::WritingMode::VerticalLr, 2.0, 10.0),
    ] {
        let mut host = host::<S>(mode, 15.0);
        let binding = attach(&mut host, &native, mode);
        let batch = compute(&host, &binding, 15.0);
        let text = placed(&batch);
        assert_eq!(text.source(), &native.source);
        assert_eq!(text.source().text().code_units(), [0x4e00]);
        assert_eq!(text.lines().len(), 1);
        assert_eq!(text.lines()[0].baseline(), center);
        let pair = central_pair(&batch);
        let fact = l::BaselineFactOf::Available(
            l::BaselineCoordinateOf::try_new(l::PhysicalAxis::Horizontal, n(center)).unwrap(),
        );
        assert_eq!((pair.first(), pair.last()), (fact, fact));
        let completions = text.nonformatted_lines().collect::<Vec<_>>();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].origin, t::Point::new(completion_x, 0.0));
        let receipt = completions[0].line.finalized_line();
        assert!(!receipt.has_in_flow_content());
        assert_eq!(receipt.prepared_text(), native.prepared);
        assert_eq!(
            receipt.selected_line().request().context.end,
            t::LineEnd::Paragraph
        );
        assert!(receipt.selected_line().request().participants.is_empty());
        let control = &binding.controls[0];
        let output = node(batch.final_entries(), control.node);
        assert_eq!(output.location, l::Point::new(n(center), n(4.0)));
        assert_eq!(output.size, l::Size::ZERO);
        let forced = batch.unrounded_inline_fragments().iter()
            .find(|f| matches!(f.owner(), l::InlineFragmentOwner::Node(node) if *node == binding.origins[0].node))
            .unwrap().fragment().finalized_fragment().finalized_line();
        assert_eq!(
            forced.selected_line().request().controls[0].occurrence,
            control.occurrence
        );
        assert_eq!(
            forced.selected_line().request().controls[0]
                .strut
                .as_ref()
                .unwrap()
                .baseline,
            8.0
        );
        assert_eq!(
            (forced.feedback().baseline, forced.feedback().line_height),
            (8.0, 10.0)
        );
    }
}

fn nonformatted_has_no_authority<S: l::LayoutScalar>() {
    let native = native(" ", 4.0, false, t::WhiteSpace::Collapse);
    qualify(&native, (2.0, 4.0, 0.0));
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let mut host = host::<S>(mode, 15.0);
        let binding = attach(&mut host, &native, mode);
        let batch = compute(&host, &binding, 15.0);
        let text = placed(&batch);
        assert_eq!(text.source(), &native.source);
        assert!(text.lines().is_empty());
        assert_eq!(text.nonformatted_lines().count(), 1);
        assert!(batch.unrounded_inline_fragments().is_empty());
        let pair = central_pair(&batch);
        assert_eq!(
            (pair.first(), pair.last()),
            (l::BaselineFactOf::Absent, l::BaselineFactOf::Absent)
        );
    }
}

fn incoming<S: l::LayoutScalar>(
    host: &mut Host<S>,
    mode: l::WritingMode,
    cross: f64,
    inline: f64,
    shape: bool,
) -> u32 {
    host.push(
        Some(0),
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            writing_mode: mode,
            float: l::Float::Left,
            float_exclusion: if shape {
                l::FloatExclusion::Shape
            } else {
                l::FloatExclusion::MarginBox
            },
            size: l::Size::new(px(cross), px(inline)),
            ..l::NodeInputOf::default()
        }),
    )
}

fn fractional_minima<S: l::LayoutScalar>() {
    let native = native("\u{4e00}", 10.5, false, t::WhiteSpace::Preserve);
    qualify(&native, (5.25, 10.5, 4.0));
    for (mode, nearest, constrained) in [
        (l::WritingMode::VerticalRl, 35.0, 24.0),
        (l::WritingMode::VerticalLr, 5.0, 16.0),
    ] {
        for has_float in [false, true] {
            let mut host = host::<S>(mode, 15.0);
            let float = has_float.then(|| incoming(&mut host, mode, 10.0, 15.0, false));
            let binding = attach(&mut host, &native, mode);
            let batch = compute(&host, &binding, 15.0);
            let text = placed(&batch);
            let center = if has_float { constrained } else { nearest };
            assert_eq!(text.lines().len(), 1);
            assert_eq!(text.lines()[0].baseline(), center);
            assert_eq!(
                text.lines()[0].bounds(),
                t::Rect::new(center - 5.25, 0.0, 10.5, 15.0)
            );
            assert_eq!(text.source(), &native.source);
            if let Some(float) = float {
                let x = if mode == l::WritingMode::VerticalRl {
                    30.0
                } else {
                    0.0
                };
                assert_eq!(
                    node(batch.final_entries(), float).location,
                    l::Point::new(n(x), n(0.0))
                );
                assert_eq!(
                    node(batch.final_entries(), float).size,
                    l::Size::new(n(10.0), n(15.0))
                );
                let rect = text.lines()[0].bounds();
                if mode == l::WritingMode::VerticalRl {
                    assert!(rect.origin.x + rect.size.width <= 30.0);
                } else {
                    assert!(rect.origin.x >= 10.0);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    LaterBand,
}

struct ShapeHost<S: l::LayoutScalar> {
    inner: Host<S>,
    float: u32,
    mode: l::WritingMode,
    refuse: Cell<bool>,
    queries: RefCell<Vec<(l::FloatExclusionQueryOf<S>, bool)>>,
}
impl<S: l::LayoutScalar> l::Traverse for ShapeHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(&self.inner, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(&self.inner, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(&self.inner, node, index)
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for ShapeHost<S> {
    type MeasureError = Refusal;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.inner, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.inner, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        l::LayoutTree::cache_get(&self.inner, node, input, context)
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        l::LayoutTree::unrounded_layout(&self.inner, node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: l::FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<l::FloatExclusionIntervalOf<S>>, Refusal>> {
        if node != self.float {
            return None;
        }
        assert_eq!(
            query.flow_axes(),
            l::FlowAxes::new(self.mode, l::Direction::Ltr)
        );
        assert_eq!(query.margin_box().origin(), l::Point::ZERO);
        assert_eq!(query.margin_box().size(), l::Size::new(n(40.0), n(4.0)));
        let later = query.band_minimum() < query.band_maximum()
            && match self.mode {
                l::WritingMode::VerticalRl => query.band_maximum() <= n(30.0),
                l::WritingMode::VerticalLr => query.band_minimum() >= n(10.0),
                _ => panic!("this provider owns native vertical fixtures"),
            };
        let refused = self.refuse.get() && later;
        self.queries.borrow_mut().push((query, refused));
        // A genuine Shape provider may return no excluded interval. The only
        // failure stimulus is the actual physical later band, never a call count.
        Some(if refused {
            Err(Refusal::LaterBand)
        } else {
            Ok(None)
        })
    }
}
impl<S: l::LayoutScalar> ShapeHost<S> {
    fn compute(
        &self,
        binding: &l::TextParagraphBinding<u32>,
        changed: &[u32],
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S, Refusal>> {
        let flow = l::LayoutFlowInput::try_new(
            self,
            0,
            vec![binding.clone()],
            l::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        l::compute_layout_with_flow_invalidated(&flow, 0, request(4.0), changed)
    }
}

fn late_refusal<S: l::LayoutScalar>() {
    let native = native("\u{4e00}\u{4e00}", 10.5, false, t::WhiteSpace::Preserve);
    qualify(&native, (5.25, 10.5, 8.0));
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let mut inner = host::<S>(mode, 4.0);
        let float = incoming(&mut inner, mode, 40.0, 4.0, true);
        let binding = attach(&mut inner, &native, mode);
        let mut host = ShapeHost {
            inner,
            float,
            mode,
            refuse: Cell::new(false),
            queries: RefCell::new(Vec::new()),
        };
        let old = host.compute(&binding, &[]).unwrap();
        let text = placed(&old);
        assert_eq!(
            text.lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [t::Range::new(0, 1), t::Range::new(1, 2)]
        );
        assert_eq!(text.source(), &native.source);
        old.apply_to(&mut host.inner).unwrap();
        let caches = host.inner.published.caches.clone();
        let nodes = host.inner.published.unrounded.clone();
        let published = host.inner.published.batch.as_ref().unwrap().clone();
        host.queries.borrow_mut().clear();
        host.refuse.set(true);
        let error = host.compute(&binding, &[0, float]).unwrap_err();
        assert_eq!(error.operation(), l::LayoutOperation::FloatExclusionQuery);
        assert_eq!(
            error.site(),
            l::LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: float
            }
        );
        assert!(matches!(
            error.kind(),
            l::LayoutErrorKindOf::Measurement(Refusal::LaterBand)
        ));
        let queries = host.queries.borrow();
        let failed = queries.iter().position(|(_, refused)| *refused).unwrap();
        assert!(
            queries[..failed].iter().any(|(q, refused)| !*refused
                && q.band_minimum() < q.band_maximum()
                && if mode == l::WritingMode::VerticalRl {
                    q.band_maximum() > n(30.0)
                } else {
                    q.band_minimum() < n(10.0)
                }),
            "the actual provider succeeded for an earlier nonempty physical band before refusing the later band"
        );
        drop(queries);
        assert_eq!(host.inner.published.caches, caches);
        assert_eq!(host.inner.published.unrounded, nodes);
        let retained = host.inner.published.batch.as_ref().unwrap();
        assert_eq!(retained.unrounded_entries(), published.unrounded_entries());
        assert_eq!(retained.final_entries(), published.final_entries());
        assert_eq!(
            retained.unrounded_inline_fragments(),
            published.unrounded_inline_fragments()
        );
        assert_eq!(
            retained.final_inline_fragments(),
            published.final_inline_fragments()
        );
        assert_eq!(
            placed(retained).positions().collect::<Vec<_>>(),
            placed(&published).positions().collect::<Vec<_>>()
        );
        assert_eq!(placed(retained).source(), &native.source);
        host.refuse.set(false);
        let restored = host.compute(&binding, &[0, float]).unwrap();
        assert_eq!(restored.final_entries(), old.final_entries());
        // Separately placed results issue distinct position identities. Compare
        // their source observations and geometry, while the retained snapshot
        // above must preserve its exact original handles.
        let positions = |text: &t::Layout| {
            text.positions()
                .map(|position| {
                    (
                        position.source_cursor(),
                        text.position_geometry(&position).unwrap().rect(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(positions(placed(&restored)), positions(placed(&old)));
    }
}

#[test]
fn native_forced_control_and_terminal_completion_keep_named_authority_f32() {
    controls_and_completion::<f32>();
}
#[test]
fn native_forced_control_and_terminal_completion_keep_named_authority_f64() {
    controls_and_completion::<f64>();
}
#[test]
fn native_nonformatted_only_receipt_has_no_named_authority_f32() {
    nonformatted_has_no_authority::<f32>();
}
#[test]
fn native_nonformatted_only_receipt_has_no_named_authority_f64() {
    nonformatted_has_no_authority::<f64>();
}
#[test]
fn native_fractional_float_minimum_uses_actual_block_progression_f32() {
    fractional_minima::<f32>();
}
#[test]
fn native_fractional_float_minimum_uses_actual_block_progression_f64() {
    fractional_minima::<f64>();
}
#[test]
fn native_later_provider_refusal_preserves_every_published_channel_f32() {
    late_refusal::<f32>();
}
#[test]
fn native_later_provider_refusal_preserves_every_published_channel_f64() {
    late_refusal::<f64>();
}
