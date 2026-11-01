//! Actual itemless paragraph coverage for an original inline continuation.
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
    children: [Vec<u32>; 4],
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

fn fixture_empty<S: l::LayoutScalar>()
-> (Tree<S>, Vec<l::TextParagraphBinding<u32>>, t::CapturedScope) {
    use t::SourceProcessingEvent as E;
    let base = t::Style {
        font: t::Font::new().family("Ahem"),
        size: 10.0,
        line_height: t::LineHeight::Absolute(10.0),
        wrap: t::Wrap::None,
        hyphens: t::Hyphens::None,
        ..t::Style::default()
    };
    let mut source = t::Source::identified("AX", t::Id::from_u64(658200), 41);
    source.span(t::Range::new(0, 2), base.clone());
    let block = || t::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: t::SourceProcessingTopology::Complete,
        trim: Default::default(),
        base_style: base.clone(),
        used_direction: t::ParagraphBaseDirection::LeftToRight,
    };
    let capture = Arc::new(
        t::process_contextual_source(t::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: t::DocumentSegmentBreaks::LineFeedDefault,
            envelope: t::SourceProcessingEnvelope::Complete,
            origins: (0..2)
                .map(|i| t::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: t::Range::new(i, i + 1),
                    aggregate: t::Range::new(i, i + 1),
                    kind: t::ParagraphOriginKind::Authored,
                })
                .collect(),
            controls: vec![],
            connections: vec![],
            boundary_policies: vec![],
            computed_line_boundaries: vec![],
            replacements: vec![],
            scopes: vec![
                block(),
                t::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: None,
                },
                block(),
            ],
            paragraphs: (0..3)
                .map(|_| t::SourceProcessingParagraph {
                    base_direction: t::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: t::ParagraphStrut::DefaultFont,
                })
                .collect(),
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                E::Origin {
                    origin: 0,
                    affinity: t::Affinity::After,
                },
                E::ExitParagraph(0),
                E::EnterScope(2),
                E::EnterParagraph(1),
                E::Origin {
                    origin: 1,
                    affinity: t::Affinity::After,
                },
                E::ExitParagraph(1),
                E::ExitScope(2),
                E::EnterParagraph(2),
                E::ExitScope(1),
                E::ExitParagraph(2),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(capture.source().text().code_units(), [0x41, 0x58]);
    assert!(capture.captured_control(0).is_none());
    assert!(capture.captured_origin(2).is_none());
    let scope = capture.captured_scope(1).unwrap();
    let p0 = capture.captured_paragraph(0).unwrap();
    let empty = capture.captured_paragraph(2).unwrap();
    assert_eq!(scope.parent(), capture.captured_scope(0));
    assert!(scope.entry_control().is_none());
    assert!(scope.exit_control().is_none());
    assert!(p0.enter_position() < scope.enter_position());
    assert!(empty.enter_position() < scope.exit_position());
    assert!(scope.exit_position() < empty.exit_position());
    assert_eq!(empty.domain(), p0.domain());
    let mut fonts = t::System::new(t::SystemOptions {
        font_catalogue: t::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts.register_fonts(AHEM.to_vec()).unwrap();
    // Root0 -> A1, block2; block2 -> X3. There is no host node for p2.
    let mut inputs = vec![
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            size: l::Size::new(
                l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(100.0)).unwrap()),
                l::PreferredSizeOf::AUTO,
            ),
            ..l::NodeInputOf::default()
        }),
        l::LayoutInputOf::box_input(l::NodeInputOf::default()),
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            ..l::NodeInputOf::default()
        }),
        l::LayoutInputOf::box_input(l::NodeInputOf::default()),
    ];
    let bindings = capture
        .paragraphs()
        .enumerate()
        .map(|(i, p)| {
            assert!(Arc::ptr_eq(p.processed_source(), &capture));
            assert_eq!(
                p.source_range(),
                if i < 2 {
                    t::Range::new(i, i + 1)
                } else {
                    t::Range::new(2, 2)
                }
            );
            let a = t::AnalyzedSource::from_processed_paragraph(
                p,
                t::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let missing = a
                .wrapping_relations()
                .filter(|r| r.ancestor_wrap().is_none())
                .map(|r| t::WrappingPolicy {
                    relation: r.handle(),
                    ancestor_wrap: t::Wrap::None,
                })
                .collect();
            let a = a.with_wrapping_policies(missing).unwrap();
            let prepared = fonts.prepare(&a, t::PreparationOptions::default()).unwrap();
            assert_eq!(
                prepared
                    .captured_owner()
                    .unwrap()
                    .members()
                    .collect::<Vec<_>>(),
                vec![capture.captured_paragraph(i).unwrap()]
            );
            let (container, nodes, origins) = if i < 2 {
                let node = if i == 0 { 1 } else { 3 };
                let origin = prepared.origin_views().next().unwrap().handle();
                assert_eq!(origin.view().aggregate_range(), t::Range::new(i, i + 1));
                inputs[node as usize] =
                    l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone()));
                (
                    if i == 0 { 0 } else { 2 },
                    vec![node],
                    vec![l::TextOriginBinding { origin, node }],
                )
            } else {
                // These observations establish the actual AssociationOnly profile;
                // no EmptyOrigin/control/zero-font stand-in supplies the closure.
                assert_eq!(prepared.origin_views().count(), 0);
                assert_eq!(prepared.control_views().count(), 0);
                assert_eq!(prepared.atomic_views().count(), 0);
                assert_eq!(prepared.participants().count(), 0);
                assert_eq!(prepared.scopes().count(), 0);
                assert_eq!(prepared.virtual_separators().count(), 0);
                (0, vec![], vec![])
            };
            l::TextParagraphBinding {
                inline_box_fragments: surgeist_layout::InlineBoxFragmentationDemand::NotRequested,
                prepared,
                container,
                nodes,
                origins,
                controls: vec![],
                atomics: vec![],
                line_strut: None,
            }
        })
        .collect();
    let children = std::array::from_fn(|i| match i {
        0 => vec![1, 2],
        2 => vec![3],
        _ => vec![],
    });
    (
        Tree {
            inputs,
            non_box: l::NodeInputOf::non_box(),
            children,
        },
        bindings,
        scope,
    )
}

fn captured_wrapping(scope: t::CapturedScope) -> l::WrappingInput<u32> {
    l::WrappingInput {
        scopes: vec![l::WrappingScopeInput::Captured {
            scope: scope.clone(),
            wrap_inside: l::WrapInside::Auto,
            edges: l::WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![l::InlineChildOrderInput {
            parent: l::GeneratedBoxRef::Inline(l::InlineBoxRef::Scope(scope)),
            children: [1, 2]
                .into_iter()
                .map(|node| l::GeneratedChildBoxInput {
                    box_: l::GeneratedBoxRef::Node(node),
                    participation: l::GeneratedBoxParticipation::InFlow,
                })
                .collect(),
            prefix: l::ChildOrderCompleteness::Complete,
            suffix: l::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        }],
    }
}

fn full_empty<S: l::LayoutScalar>() {
    let (tree, bindings, scope) = fixture_empty::<S>();
    let flow =
        l::LayoutFlowInput::try_new(&tree, 0, bindings, captured_wrapping(scope), Vec::new())
            .expect("the supplied real itemless ending paragraph authenticates S's closure");
    // This is admission only: absence of a supplied box strut makes no zero
    // geometry promise, and no selected line/fit is requested.
    drop(flow);
}

fn missing_empty<S: l::LayoutScalar>() {
    let (tree, mut bindings, scope) = fixture_empty::<S>();
    let omitted = bindings.pop().unwrap();
    assert!(omitted.nodes.is_empty());
    assert!(omitted.origins.is_empty());
    assert!(omitted.controls.is_empty());
    assert!(omitted.atomics.is_empty());
    // A1/X3 are still covered by the first two bindings in the unchanged tree.
    // Complete S order still claims its real original after edge; without p2
    // that needed captured region is unavailable, rather than implied by glyphs.
    let error =
        l::LayoutFlowInput::try_new(&tree, 0, bindings, captured_wrapping(scope), Vec::new())
            .err()
            .expect("all text covered is insufficient without S's real ending paragraph");
    assert!(
        matches!(
            error.kind(),
            l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::WrappingScope(
                l::WrappingScopeError::UnsupportedIncidence
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn real_itemless_ending_paragraph_completes_captured_order_coverage() {
    full_empty::<f32>();
    full_empty::<f64>();
}

#[test]
fn omitting_only_real_itemless_ending_paragraph_leaves_required_region_unavailable() {
    missing_empty::<f32>();
    missing_empty::<f64>();
}
