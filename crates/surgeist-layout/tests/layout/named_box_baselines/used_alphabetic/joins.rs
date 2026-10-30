//! Used-Alphabetic knowledge, physical box ownership, and transactional joins.
use super::*;
use std::cell::RefCell;

fn atomic_capture(
    system: &mut t::System,
    orientation: t::LineOrientation,
    kind: t::InlineBoxKind,
    id: u64,
) -> Captured {
    let raw = "AB";
    let style = style(10.0, 10.0);
    let mut source = t::Source::new(raw);
    source.inline_box(t::InlineBox::new(
        t::Id::from_u64(id),
        kind,
        1,
        t::Size::new(6.0, 14.0),
    ));
    let ranges = [t::Range::new(0, 1), t::Range::new(1, 2)];
    use t::SourceProcessingEvent as E;
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: t::Affinity::After,
        },
    ];
    events.extend([
        E::Atomic {
            atomic: 0,
            affinity: t::Affinity::After,
        },
        E::Connection(0),
        E::Origin {
            origin: 1,
            affinity: t::Affinity::After,
        },
    ]);
    let boundaries = [
        t::ParagraphBoundaryDefinition::AtomicBefore(0),
        t::ParagraphBoundaryDefinition::AtomicAfter(0),
        t::ParagraphBoundaryDefinition::Connection(0),
    ];
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let processed = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: ranges
                .into_iter()
                .map(|range| t::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: range,
                    aggregate: range,
                    kind: t::ParagraphOriginKind::Authored,
                })
                .collect(),
            controls: vec![],
            connections: vec![t::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: t::ParagraphConnectionKind::Continuous,
            }],
            boundary_policies: boundaries
                .into_iter()
                .map(|definition| t::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: t::Wrap::Word,
                    following_wrap: t::Wrap::Word,
                    ancestor_wrap: t::Wrap::Word,
                })
                .collect(),
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
    let analysis = t::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        t::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analysis
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| t::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: t::Wrap::Word,
        })
        .collect();
    let analysis = analysis.with_wrapping_policies(policies).unwrap();
    let options = t::PreparationOptions::default()
        .with_orientation(orientation)
        .with_baseline_reference(ALPHA);
    let prepared = system
        .prepare(&analysis, options)
        .expect("the complete genuine used-Alphabetic producer must succeed before Layout");
    assert_eq!(prepared.baseline_reference(), Some(ALPHA));
    assert_eq!(
        prepared
            .preparation_options()
            .requested_baseline_reference(),
        Some(ALPHA)
    );
    assert_eq!(prepared.preparation_options(), options);
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(
        prepared
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        vec![processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(prepared.orientation(), orientation);
    assert_eq!(prepared.atomic_views().count(), 1);
    assert_eq!(
        prepared
            .atomic_views()
            .next()
            .unwrap()
            .original_box()
            .kind(),
        kind
    );
    Captured { prepared, source }
}

const MODES: [l::WritingMode; 4] = [
    l::WritingMode::HorizontalTb,
    l::WritingMode::SidewaysRl,
    l::WritingMode::VerticalRl,
    l::WritingMode::VerticalLr,
];
fn named_context() -> l::InlineBaselineContext {
    l::InlineBaselineContext::Named(l::BaselineName::Alphabetic)
}
fn clockwise(mode: l::WritingMode) -> bool {
    mode != l::WritingMode::HorizontalTb
}
fn orientation(mode: l::WritingMode) -> t::LineOrientation {
    if clockwise(mode) {
        t::LineOrientation::SidewaysClockwise
    } else {
        t::LineOrientation::Horizontal
    }
}
fn axis(mode: l::WritingMode) -> l::PhysicalAxis {
    if clockwise(mode) {
        l::PhysicalAxis::Horizontal
    } else {
        l::PhysicalAxis::Vertical
    }
}
fn common(mode: l::WritingMode) -> f64 {
    match mode {
        l::WritingMode::HorizontalTb => 7.0,
        l::WritingMode::VerticalLr => 3.0,
        _ => 33.0,
    }
}
fn system() -> (t::System, Vec<u8>) {
    let bytes = font();
    qualify_font(&bytes);
    let mut system = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    (system, bytes)
}
fn paragraph<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parent: &Captured,
) -> l::TextParagraphBinding<u32> {
    let origin = parent.prepared.origin_views().next().unwrap().handle();
    let text = host.push(
        Some(container),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone())),
    );
    child_binding(parent, container, text)
}
fn attach<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    parent: &Captured,
    input: l::NodeInputOf<S>,
    child: Option<&Captured>,
) -> Vec<l::TextParagraphBinding<u32>> {
    let origins = parent
        .prepared
        .origin_views()
        .map(|row| row.handle())
        .collect::<Vec<_>>();
    assert_eq!(origins.len(), 2);
    let first = host.push(
        Some(container),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[0].clone())),
    );
    let atomic = host.push(Some(container), l::LayoutInputOf::box_input(input));
    assert_eq!(
        parent
            .prepared
            .atomic_views()
            .next()
            .unwrap()
            .original_box()
            .id(),
        t::Id::from_u64(u64::from(atomic))
    );
    let last = host.push(
        Some(container),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origins[1].clone())),
    );
    let mut bindings = vec![l::TextParagraphBinding {
        prepared: parent.prepared.clone(),
        container,
        nodes: vec![first, atomic, last],
        origins: origins
            .into_iter()
            .zip([first, last])
            .map(|(origin, node)| l::TextOriginBinding { origin, node })
            .collect(),
        atomics: vec![l::TextAtomicBinding {
            occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
            node: atomic,
        }],
        controls: vec![],
        line_strut: None,
    }];
    if let Some(child) = child {
        let origin = child.prepared.origin_views().next().unwrap().handle();
        let text = host.push(
            Some(atomic),
            l::LayoutInputOf::inline_text(l::TextParticipation::new(origin)),
        );
        bindings.push(child_binding(child, atomic, text));
    }
    bindings
}
fn scene<S: l::LayoutScalar>(
    mode: l::WritingMode,
    parent: &Captured,
    input: l::NodeInputOf<S>,
    child: Option<&Captured>,
) -> (Host<S>, Vec<l::TextParagraphBinding<u32>>) {
    let mut input_root = root(mode, l::Direction::Ltr, named_context());
    input_root.size = l::Size::new(px(40.0), px(40.0));
    let mut host = Host::new(input_root);
    let bindings = attach(&mut host, 0, parent, input, child);
    (host, bindings)
}
fn compute<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: &[l::TextParagraphBinding<u32>],
    changed: &[u32],
    available: l::Size<l::AvailableOf<S>>,
) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
    let flow = l::LayoutFlowInput::try_new(
        host,
        0,
        bindings.to_vec(),
        l::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(available).unwrap(),
        changed,
    )
}
fn receipt<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    source: u32,
) -> t::FinalizedLine {
    batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| matches!(row.owner(),l::InlineFragmentOwner::Node(node) if *node==source))
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line()
        .clone()
}
fn custody<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    container: u32,
    parent: &Captured,
) {
    let text = placed(batch, container);
    assert_eq!(text.source(), &parent.source);
    assert_eq!(
        text.analyzed_source().identity(),
        parent.prepared.analyzed_source().identity()
    );
}
fn knowledge<S: l::LayoutScalar>() {
    let (mut system, bytes) = system();
    let child_h = capture(
        &mut system,
        "A\nB",
        style(5.0, 6.0),
        t::LineOrientation::Horizontal,
        false,
    );
    let child_cw = capture(
        &mut system,
        "A\nB",
        style(5.0, 6.0),
        t::LineOrientation::SidewaysClockwise,
        false,
    );
    for (child, cw) in [(&child_h, false), (&child_cw, true)] {
        qualify_child_text(child, &bytes, cw);
        qualify_physical_child::<S>(child, cw);
    }
    for mode in MODES {
        let cw = clockwise(mode);
        let child = if cw { &child_cw } else { &child_h };
        let opposite = if cw { &child_h } else { &child_cw };
        let parent = capture(
            &mut system,
            "AB",
            style(10.0, 10.0),
            orientation(mode),
            true,
        );
        // A real empty child establishes Absent. It is not an Unknown Text box.
        let empty = child_input::<S>(cw, named_context());
        let mut alone = Host::new(empty.clone());
        alone.change(0, |input| input.display = l::Display::Block);
        let empty_batch = compute(&alone, &[], &[], space(40.0, 40.0)).unwrap();
        absent(node(empty_batch.unrounded_entries(), 0).named_baselines);
        let (host, bindings) = scene::<S>(mode, &parent, empty, None);
        let batch = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap();
        let (baseline, height, x, y) = if cw {
            (common(mode), 10.0, common(mode), 10.0)
        } else {
            (14.0, 17.0, 10.0, 0.0)
        };
        alpha(
            node(batch.unrounded_entries(), 0).named_baselines,
            axis(mode),
            baseline,
            baseline,
        );
        absent(node(batch.unrounded_entries(), 2).named_baselines);
        assert_eq!(
            node(batch.unrounded_entries(), 2).location,
            l::Point::new(n(x), n(y))
        );
        assert_eq!(receipt(&batch, 1).feedback().line_height, height);
        custody(&batch, 0, &parent);
        // A genuine Physical child remains Unknown despite its normalized Text.
        let (host, bindings) = scene::<S>(
            mode,
            &parent,
            child_input::<S>(cw, l::InlineBaselineContext::Physical),
            Some(child),
        );
        let error = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap_err();
        assert_eq!(error.site(), l::LayoutErrorSiteOf::Node(2));
        assert_eq!(error.operation(), l::LayoutOperation::ChildLayout);
        assert!(
            matches!(
                error.kind(),
                l::LayoutErrorKindOf::MissingContext(l::LayoutMissingContext::NamedInlineBaseline)
            ),
            "{mode:?}: {error:?}"
        );
        let (host, bindings) = scene::<S>(
            mode,
            &parent,
            child_input::<S>(!cw, named_context()),
            Some(opposite),
        );
        let error = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap_err();
        assert_eq!(error.site(), l::LayoutErrorSiteOf::Node(2));
        assert!(
            matches!(error.kind(),l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::InlineBaseline(l::InlineBaselineError::AxisMismatch{expected,actual})) if *expected==axis(mode) && *actual==axis(if cw {l::WritingMode::HorizontalTb} else {l::WritingMode::SidewaysRl})),
            "{mode:?}: {error:?}"
        );
        // Edge alignment never consumes the otherwise unavailable child name.
        for align in [l::VerticalAlignOf::Top, l::VerticalAlignOf::Bottom] {
            let mut input = child_input::<S>(cw, l::InlineBaselineContext::Physical);
            input.vertical_align = align;
            let (host, bindings) = scene::<S>(mode, &parent, input, Some(child));
            let batch = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap();
            let top = align == l::VerticalAlignOf::Top;
            let b = if cw {
                common(mode)
            } else if top {
                7.0
            } else {
                11.0
            };
            let x = if cw {
                b + if top { 1.0 } else { -3.0 }
            } else {
                10.0
            };
            let y = if cw { 10.0 } else { 0.0 };
            alpha(
                node(batch.unrounded_entries(), 0).named_baselines,
                axis(mode),
                b,
                b,
            );
            unknown(node(batch.unrounded_entries(), 2).named_baselines);
            assert_eq!(
                node(batch.unrounded_entries(), 2).location,
                l::Point::new(n(x), n(y))
            );
            assert_eq!(
                receipt(&batch, 1).feedback().line_height,
                if cw { 10.0 } else { 14.0 }
            );
            assert_eq!(
                placed(&batch, 0).inline_boxes()[0].rect(),
                t::Rect::new(x, y, 6.0, 14.0)
            );
            custody(&batch, 2, child);
        }
    }
}
#[test]
fn used_alphabetic_distinguishes_absent_unknown_axes_and_unused_edge_alignment() {
    knowledge::<f32>();
    knowledge::<f64>();
}

fn explicit(
    parent: &Captured,
    line: t::FinalizedLine,
    anchor: t::LineAnchor,
    atomic: t::AtomicPlacement,
) -> t::Layout {
    t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: parent.prepared.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor,
            orientation: parent.prepared.orientation(),
            atomics: vec![atomic],
        })],
    })
    .unwrap()
}
fn length<S: l::LayoutScalar>(value: f64) -> l::LengthOf<S> {
    l::LengthOf::value(l::LengthPercentageOf::px(n(value)).unwrap())
}
fn positioned<S: l::LayoutScalar>() {
    let (mut system, bytes) = system();
    let children = [
        capture(
            &mut system,
            "A\nB",
            style(5.0, 6.0),
            t::LineOrientation::Horizontal,
            false,
        ),
        capture(
            &mut system,
            "A\nB",
            style(5.0, 6.0),
            t::LineOrientation::SidewaysClockwise,
            false,
        ),
    ];
    for (i, child) in children.iter().enumerate() {
        qualify_child_text(child, &bytes, i == 1);
        qualify_physical_child::<S>(child, i == 1);
    }
    for mode in MODES {
        let cw = clockwise(mode);
        let parent = atomic_capture(
            &mut system,
            orientation(mode),
            t::InlineBoxKind::OutOfFlow,
            2,
        );
        let qualified = select(
            &parent,
            parent.prepared.boundary_views().next().unwrap().handle(),
            terminal(&parent.prepared),
            vec![t::AtomicAllocation {
                occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                allocation: t::AtomicLineAllocation::OutOfFlow,
            }],
            40.0,
        );
        assert_eq!(
            (
                qualified.feedback().advance,
                qualified.feedback().baseline,
                qualified.feedback().line_height
            ),
            (20.0, 7.0, 10.0)
        );
        for child_cw in [false, true] {
            let child = &children[usize::from(child_cw)];
            for alignment in [
                l::VerticalAlignOf::Baseline,
                l::VerticalAlignOf::Top,
                l::VerticalAlignOf::Bottom,
            ] {
                for inset in [false, true] {
                    let child_named = alignment == l::VerticalAlignOf::Baseline && !inset;
                    let mut input = child_input::<S>(
                        child_cw,
                        if child_named {
                            named_context()
                        } else {
                            l::InlineBaselineContext::Physical
                        },
                    );
                    input.position = l::Position::Absolute;
                    input.baseline_source = l::BaselineSource::Last;
                    input.vertical_align = alignment;
                    if inset {
                        input.inset.left =
                            l::LengthAutoOf::value(l::LengthPercentageOf::px(n(3.0)).unwrap());
                        input.inset.right =
                            l::LengthAutoOf::value(l::LengthPercentageOf::px(n(9.0)).unwrap());
                        input.inset.top =
                            l::LengthAutoOf::value(l::LengthPercentageOf::px(n(2.0)).unwrap());
                        input.inset.bottom =
                            l::LengthAutoOf::value(l::LengthPercentageOf::px(n(8.0)).unwrap());
                    }
                    let (host, bindings) = scene::<S>(mode, &parent, input, Some(child));
                    let batch = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap();
                    let (x, y) = if inset {
                        (3.0, 2.0)
                    } else if !cw {
                        (10.0, 0.0)
                    } else if mode == l::WritingMode::VerticalLr {
                        (0.0, 10.0)
                    } else {
                        (34.0, 10.0)
                    };
                    let border = t::Rect::new(x, y, 6.0, 14.0);
                    let numeric = if !cw {
                        y + if child_cw { 14.0 } else { 4.0 }
                    } else {
                        x + if child_cw { 2.0 } else { 0.0 }
                    };
                    let line = receipt(&batch, 1);
                    assert_eq!(line.prepared_text(), parent.prepared);
                    assert_eq!(
                        (
                            line.feedback().advance,
                            line.feedback().baseline,
                            line.feedback().line_height
                        ),
                        (20.0, 7.0, 10.0)
                    );
                    let expected = explicit(
                        &parent,
                        line,
                        t::LineAnchor {
                            inline_origin: 0.0,
                            baseline: common(mode),
                        },
                        t::AtomicPlacement {
                            occurrence: parent.prepared.atomic_views().next().unwrap().handle(),
                            border,
                            content: border,
                            baseline: numeric,
                        },
                    );
                    assert_eq!(
                        placed(&batch, 0).key(),
                        expected.key(),
                        "{mode:?} childCW={child_cw} {alignment:?} inset={inset}"
                    );
                    for entries in [batch.unrounded_entries(), batch.final_entries()] {
                        assert_eq!(node(entries, 2).location, l::Point::new(n(x), n(y)));
                        if child_named {
                            alpha(
                                node(entries, 2).named_baselines,
                                if child_cw {
                                    l::PhysicalAxis::Horizontal
                                } else {
                                    l::PhysicalAxis::Vertical
                                },
                                if child_cw { 2.0 } else { 4.0 },
                                if child_cw { -4.0 } else { 10.0 },
                            );
                        } else {
                            unknown(node(entries, 2).named_baselines);
                        }
                        alpha(
                            node(entries, 0).named_baselines,
                            axis(mode),
                            common(mode),
                            common(mode),
                        );
                    }
                    custody(&batch, 0, &parent);
                    custody(&batch, 2, child);
                }
            }
        }
    }
}
#[test]
fn used_alphabetic_positioned_boxes_keep_physical_first_reference_and_inset_precedence() {
    positioned::<f32>();
    positioned::<f64>();
}

fn paragraph_scene<S: l::LayoutScalar>(
    parent: &Captured,
    mode: l::WritingMode,
) -> (Host<S>, Vec<l::TextParagraphBinding<u32>>) {
    let mut input = root(mode, l::Direction::Ltr, named_context());
    input.size = l::Size::new(
        px(if clockwise(mode) { 40.0 } else { 10.0 }),
        px(if clockwise(mode) { 10.0 } else { 40.0 }),
    );
    let mut host = Host::new(input);
    let binding = paragraph(&mut host, 0, parent);
    (host, vec![binding])
}
fn fitting<S: l::LayoutScalar>() {
    let (mut system, _) = system();
    for mode in MODES {
        let cw = clockwise(mode);
        let parent = capture(
            &mut system,
            "AB",
            style(10.0, 10.0),
            orientation(mode),
            false,
        );
        // The complete public prepared source supplies both actual glyphs and
        // a permitted inner offer; capacity10 admits exactly one advance10.
        assert_eq!(parent.prepared.unit_views().count(), 2);
        assert!(
            parent
                .prepared
                .opportunity_views()
                .any(|row| row.kind() == t::PreparedBreakKind::Allowed)
        );
        let whole = select(
            &parent,
            parent.prepared.boundary_views().next().unwrap().handle(),
            terminal(&parent.prepared),
            vec![],
            40.0,
        );
        assert_eq!(
            (
                whole.feedback().advance,
                whole.feedback().baseline,
                whole.feedback().line_height
            ),
            (20.0, 7.0, 10.0)
        );
        let (host, bindings) = paragraph_scene::<S>(&parent, mode);
        let batch = compute(
            &host,
            &bindings,
            &[],
            if cw {
                space(40.0, 10.0)
            } else {
                space(10.0, 40.0)
            },
        )
        .unwrap();
        let refs = match mode {
            l::WritingMode::HorizontalTb => [7.0, 17.0],
            l::WritingMode::VerticalLr => [3.0, 13.0],
            _ => [33.0, 23.0],
        };
        assert_eq!(
            placed(&batch, 0)
                .lines()
                .iter()
                .map(|line| line.baseline())
                .collect::<Vec<_>>(),
            refs
        );
        alpha(
            node(batch.unrounded_entries(), 0).named_baselines,
            axis(mode),
            refs[0],
            refs[1],
        );
        let glyphs = placed(&batch, 0)
            .run_views()
            .flat_map(|run| run.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [35, 36]);
        custody(&batch, 0, &parent);
        for minimum in [false, true] {
            let (mut host, bindings) = paragraph_scene::<S>(&parent, mode);
            host.change(0, |input| {
                input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO)
            });
            let inline = if minimum {
                l::AvailableOf::MIN_CONTENT
            } else {
                l::AvailableOf::MAX_CONTENT
            };
            let available = if cw {
                l::Size::new(l::AvailableOf::MAX_CONTENT, inline)
            } else {
                l::Size::new(inline, l::AvailableOf::MAX_CONTENT)
            };
            let batch = compute(&host, &bindings, &[], available).unwrap();
            let (width, height, first, last) =
                match (cw, minimum, mode == l::WritingMode::VerticalLr) {
                    (false, true, _) => (10.0, 20.0, 7.0, 17.0),
                    (false, false, _) => (20.0, 10.0, 7.0, 7.0),
                    (true, true, true) => (20.0, 10.0, 3.0, 13.0),
                    (true, true, false) => (20.0, 10.0, 13.0, 3.0),
                    (true, false, _) => (10.0, 20.0, 3.0, 3.0),
                };
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(width), n(height))
            );
            alpha(
                node(batch.unrounded_entries(), 0).named_baselines,
                axis(mode),
                first,
                last,
            );
            assert_eq!(placed(&batch, 0).lines().len(), if minimum { 2 } else { 1 });
            custody(&batch, 0, &parent);
        }
        let (mut host, bindings) = paragraph_scene::<S>(&parent, mode);
        host.change(0, |input| {
            input.size = l::Size::new(px(40.0), px(40.0));
            input.padding.left = length(2.0);
            input.padding.right = length(3.0);
            input.padding.top = length(4.0);
            input.padding.bottom = length(5.0);
        });
        let batch = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap();
        let b = match mode {
            l::WritingMode::HorizontalTb => 11.0,
            l::WritingMode::VerticalLr => 5.0,
            _ => 30.0,
        };
        alpha(
            node(batch.unrounded_entries(), 0).named_baselines,
            axis(mode),
            b,
            b,
        );
        for run in placed(&batch, 0).run_views() {
            assert_eq!(
                if cw { run.origin().x } else { run.origin().y },
                b + if cw { -1.0 } else { 1.0 }
            );
            assert_eq!(
                if cw { run.origin().y } else { run.origin().x },
                if cw { 4.0 } else { 2.0 }
            );
        }
        // A fresh raw preparation of the same actual analysis lacks the name.
        let raw = system
            .prepare(
                &parent.prepared.analyzed_source(),
                t::PreparationOptions::default().with_orientation(orientation(mode)),
            )
            .unwrap();
        assert_eq!(raw.baseline_reference(), None);
        let raw = Captured {
            prepared: raw,
            source: parent.source.clone(),
        };
        let (host, bindings) = paragraph_scene::<S>(&raw, mode);
        let error = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap_err();
        assert_eq!(error.site(), l::LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), l::LayoutOperation::TextCapture);
        assert!(matches!(
            error.kind(),
            l::LayoutErrorKindOf::MissingContext(l::LayoutMissingContext::NamedInlineBaseline)
        ));
    }
}
#[test]
fn used_alphabetic_finite_intrinsic_and_padded_lines_keep_common_references() {
    fitting::<f32>();
    fitting::<f64>();
}

fn same_text<S: l::LayoutScalar>(
    a: &l::CompletedLayoutBatchOf<u32, S>,
    b: &l::CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(a.placed_text().len(), b.placed_text().len());
    for row in a.placed_text() {
        let other = placed(b, row.container());
        assert_eq!(row.layout().source(), other.source());
        assert_eq!(row.layout().key(), other.key());
        let positions = |text: &t::Layout| {
            text.positions()
                .map(|p| {
                    (
                        p.source_cursor(),
                        text.position_geometry(&p).unwrap().rect(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(positions(row.layout()), positions(other));
    }
}
fn retained<S: l::LayoutScalar>(
    host: &Host<S>,
    cache: &[l::CacheOf<S>],
    nodes: &[Option<l::NodeOutputOf<S>>],
    old: &l::CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(host.published.caches, cache);
    assert_eq!(host.published.unrounded, nodes);
    let current = host.published.batch.as_ref().unwrap();
    assert_eq!(current.unrounded_entries(), old.unrounded_entries());
    assert_eq!(current.final_entries(), old.final_entries());
    assert_eq!(
        current.unrounded_inline_fragments(),
        old.unrounded_inline_fragments()
    );
    assert_eq!(
        current.final_inline_fragments(),
        old.final_inline_fragments()
    );
    same_text(current, old);
    for row in old.placed_text() {
        assert_eq!(
            placed(current, row.container())
                .positions()
                .collect::<Vec<_>>(),
            row.layout().positions().collect::<Vec<_>>()
        );
    }
}
fn replay<S: l::LayoutScalar>() {
    let (mut system, bytes) = system();
    for mode in MODES {
        let cw = clockwise(mode);
        let child = capture(
            &mut system,
            "A\nB",
            style(5.0, 6.0),
            orientation(mode),
            false,
        );
        qualify_child_text(&child, &bytes, cw);
        for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
            let parent = atomic_capture(&mut system, orientation(mode), kind, 2);
            let make_input = || {
                let mut input = child_input::<S>(
                    cw,
                    if kind == t::InlineBoxKind::InFlow {
                        named_context()
                    } else {
                        l::InlineBaselineContext::Physical
                    },
                );
                input.baseline_source = l::BaselineSource::First;
                if kind == t::InlineBoxKind::OutOfFlow {
                    input.position = l::Position::Absolute;
                }
                input
            };
            let (mut host, bindings) = scene::<S>(mode, &parent, make_input(), Some(&child));
            let old = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap();
            old.apply_to(&mut host).unwrap();
            let cache = host.published.caches.clone();
            let nodes = host.published.unrounded.clone();
            let warm = compute(&host, &bindings, &[], space(40.0, 40.0)).unwrap();
            assert_eq!(semantic(&old), semantic(&warm));
            assert_eq!(old.final_entries(), warm.final_entries());
            same_text(&old, &warm);
            host.change(2, |input| input.size.height = px(18.0));
            let changed = compute(&host, &bindings, &[2], space(40.0, 40.0)).unwrap();
            let (mut fresh, fresh_bindings) = scene::<S>(mode, &parent, make_input(), Some(&child));
            fresh.change(2, |input| input.size.height = px(18.0));
            let fresh = compute(&fresh, &fresh_bindings, &[], space(40.0, 40.0)).unwrap();
            assert_eq!(semantic(&changed), semantic(&fresh));
            assert_eq!(changed.final_entries(), fresh.final_entries());
            same_text(&changed, &fresh);
            let inflow = kind == t::InlineBoxKind::InFlow;
            let b = common(mode);
            let (x, y) = if !cw {
                (10.0, if inflow { 3.0 } else { 0.0 })
            } else if inflow {
                (b - 2.0, 10.0)
            } else if mode == l::WritingMode::VerticalLr {
                (0.0, 10.0)
            } else {
                (34.0, 10.0)
            };
            assert_eq!(
                placed(&changed, 0).inline_boxes()[0].rect(),
                t::Rect::new(x, y, 6.0, 18.0)
            );
            let line = receipt(&changed, 1);
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (
                    if inflow {
                        if cw { 38.0 } else { 26.0 }
                    } else {
                        20.0
                    },
                    7.0,
                    if inflow && !cw { 21.0 } else { 10.0 }
                )
            );
            alpha(
                node(changed.unrounded_entries(), 0).named_baselines,
                axis(mode),
                b,
                b,
            );
            custody(&changed, 0, &parent);
            custody(&changed, 2, &child);
            // Neither successful immutable recomputation changed the applied packet.
            retained(&host, &cache, &nodes, &old);
        }
        // Fractional container placement is governed by the packet's shared
        // global snap, not by treating raw local names as already canonical.
        let parent = capture(
            &mut system,
            "AB",
            style(10.0, 10.0),
            orientation(mode),
            false,
        );
        let mut outer = root(
            l::WritingMode::HorizontalTb,
            l::Direction::Ltr,
            l::InlineBaselineContext::Physical,
        );
        outer.size = l::Size::new(px(100.0), px(100.0));
        outer.padding.left = length(7.25);
        outer.padding.top = length(9.5);
        let mut host = Host::<S>::new(outer);
        let mut inner = root(mode, l::Direction::Ltr, named_context());
        inner.size = l::Size::new(px(40.0), px(40.0));
        assert_eq!(host.push(Some(0), l::LayoutInputOf::box_input(inner)), 1);
        let bindings = vec![paragraph(&mut host, 1, &parent)];
        let batch = compute(&host, &bindings, &[], space(100.0, 100.0)).unwrap();
        let raw = node(batch.unrounded_entries(), 1);
        let final_node = node(batch.final_entries(), 1);
        assert_eq!(raw.location, l::Point::new(n(7.25), n(9.5)));
        let l::BaselineFactOf::Available(raw_name) =
            raw.named_baselines.get(l::BaselineName::Alphabetic).first()
        else {
            panic!("formatted normalized Text must supply its name")
        };
        let l::BaselineFactOf::Available(final_name) = final_node
            .named_baselines
            .get(l::BaselineName::Alphabetic)
            .first()
        else {
            panic!("canonical packet must retain its name")
        };
        let raw_origin = if cw { raw.location.x } else { raw.location.y };
        let final_origin = if cw {
            final_node.location.x
        } else {
            final_node.location.y
        };
        let global = (raw_origin.to_f64() + raw_name.offset().to_f64() + 0.5).floor();
        assert_eq!(final_origin.to_f64() + final_name.offset().to_f64(), global);
        assert_eq!(placed(&batch, 1).lines()[0].baseline(), global);
        for run in placed(&batch, 1).run_views() {
            assert_eq!(
                if cw { run.origin().x } else { run.origin().y },
                global + if cw { -1.0 } else { 1.0 }
            );
        }
        custody(&batch, 1, &parent);
    }
}
#[test]
fn used_alphabetic_changed_atomic_replay_and_fractional_names_keep_one_authority() {
    replay::<f32>();
    replay::<f64>();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    LaterBand,
}
struct ShapeHost<S: l::LayoutScalar> {
    inner: Host<S>,
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
        if node != 1 {
            return None;
        }
        assert_eq!(
            query.flow_axes(),
            l::FlowAxes::new(self.mode, l::Direction::Ltr)
        );
        assert_eq!(query.margin_box().origin(), l::Point::ZERO);
        assert_eq!(
            query.margin_box().size(),
            if clockwise(self.mode) {
                l::Size::new(n(40.0), n(10.0))
            } else {
                l::Size::new(n(10.0), n(40.0))
            }
        );
        let later = query.band_minimum() < query.band_maximum()
            && match self.mode {
                l::WritingMode::SidewaysRl | l::WritingMode::VerticalRl => {
                    query.band_maximum() <= n(30.0)
                }
                _ => query.band_minimum() >= n(10.0),
            };
        let fail = self.refuse.get() && later;
        self.queries.borrow_mut().push((query, fail));
        Some(if fail {
            Err(Refusal::LaterBand)
        } else {
            Ok(None)
        })
    }
}
impl<S: l::LayoutScalar> ShapeHost<S> {
    fn compute(
        &self,
        bindings: &[l::TextParagraphBinding<u32>],
        changed: &[u32],
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S, Refusal>> {
        let flow = l::LayoutFlowInput::try_new(
            self,
            0,
            bindings.to_vec(),
            l::WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        l::compute_layout_with_flow_invalidated(
            &flow,
            0,
            l::LayoutRootRequestOf::viewport(if clockwise(self.mode) {
                space(40.0, 10.0)
            } else {
                space(10.0, 40.0)
            })
            .unwrap(),
            changed,
        )
    }
}
fn shape_scene<S: l::LayoutScalar>(
    mode: l::WritingMode,
    parent: &Captured,
) -> (ShapeHost<S>, Vec<l::TextParagraphBinding<u32>>) {
    let cw = clockwise(mode);
    let mut input = root(mode, l::Direction::Ltr, named_context());
    input.size = l::Size::new(
        px(if cw { 40.0 } else { 10.0 }),
        px(if cw { 10.0 } else { 40.0 }),
    );
    let mut inner = Host::new(input);
    let mut float = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
    float.float = l::Float::Left;
    float.float_exclusion = l::FloatExclusion::Shape;
    float.size = l::Size::new(
        px(if cw { 40.0 } else { 10.0 }),
        px(if cw { 10.0 } else { 40.0 }),
    );
    assert_eq!(inner.push(Some(0), l::LayoutInputOf::box_input(float)), 1);
    let binding = paragraph(&mut inner, 0, parent);
    assert_eq!(binding.nodes, [2]);
    (
        ShapeHost {
            inner,
            mode,
            refuse: Cell::new(false),
            queries: RefCell::new(vec![]),
        },
        vec![binding],
    )
}
fn late_failure<S: l::LayoutScalar>() {
    let (mut system, _) = system();
    for mode in MODES {
        let parent = capture(
            &mut system,
            "AB",
            style(10.0, 10.0),
            orientation(mode),
            false,
        );
        let (mut host, bindings) = shape_scene::<S>(mode, &parent);
        let old = host.compute(&bindings, &[]).unwrap();
        assert_eq!(placed(&old, 0).lines().len(), 2);
        let refs = match mode {
            l::WritingMode::HorizontalTb => [7.0, 17.0],
            l::WritingMode::VerticalLr => [3.0, 13.0],
            _ => [33.0, 23.0],
        };
        alpha(
            node(old.unrounded_entries(), 0).named_baselines,
            axis(mode),
            refs[0],
            refs[1],
        );
        custody(&old, 0, &parent);
        old.apply_to(&mut host.inner).unwrap();
        let cache = host.inner.published.caches.clone();
        let nodes = host.inner.published.unrounded.clone();
        host.queries.borrow_mut().clear();
        host.refuse.set(true);
        let result = host.compute(&bindings, &[0, 1]);
        assert!(
            result.is_err(),
            "no completed batch may escape an actual provider refusal"
        );
        let error = result.unwrap_err();
        assert_eq!(error.operation(), l::LayoutOperation::FloatExclusionQuery);
        assert_eq!(
            error.site(),
            l::LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert!(matches!(
            error.kind(),
            l::LayoutErrorKindOf::Measurement(Refusal::LaterBand)
        ));
        let queries = host.queries.borrow();
        let failed = queries.iter().position(|(_, fail)| *fail).unwrap();
        assert!(queries[failed].0.band_minimum() < queries[failed].0.band_maximum());
        assert!(
            queries[..failed].iter().any(|(q, fail)| !*fail
                && q.band_minimum() < q.band_maximum()
                && match mode {
                    l::WritingMode::SidewaysRl | l::WritingMode::VerticalRl =>
                        q.band_maximum() > n(30.0),
                    _ => q.band_minimum() < n(10.0),
                }),
            "an earlier real nonempty band must have succeeded"
        );
        drop(queries);
        retained(&host.inner, &cache, &nodes, &old);
        host.refuse.set(false);
        let restored = host.compute(&bindings, &[0, 1]).unwrap();
        let (fresh, fresh_bindings) = shape_scene::<S>(mode, &parent);
        let fresh = fresh.compute(&fresh_bindings, &[]).unwrap();
        for other in [&old, &fresh] {
            assert_eq!(semantic(&restored), semantic(other));
            assert_eq!(restored.final_entries(), other.final_entries());
            same_text(&restored, other);
        }
    }
}
#[test]
fn used_alphabetic_later_band_refusal_preserves_applied_named_and_text_channels() {
    late_failure::<f32>();
    late_failure::<f64>();
}
