//! Closing demand retains continuing owners and canonical conditional source.
use crate::test_support::inline_endpoint::Fixture;
use crate::test_support::readback::{Phase, single_box_output};
use crate::*;
use surgeist_text as text;

fn execute<S: LayoutScalar>(
    fixture: &Fixture,
    end_padding: f64,
    float_width: f64,
    before_source: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut binding = fixture.binding.clone();
    assert_eq!(binding.nodes, [20, 10, 21]);
    let start = binding
        .controls
        .iter()
        .find(|row| row.node == 20)
        .unwrap()
        .occurrence
        .captured_control()
        .unwrap();
    let end = binding
        .controls
        .iter()
        .find(|row| row.node == 21)
        .unwrap()
        .occurrence
        .captured_control()
        .unwrap();
    let scope = start.scope();
    assert_eq!(scope.entry_control(), Some(start));
    assert_eq!(scope.exit_control(), Some(end));
    binding.nodes.insert(if before_source { 1 } else { 2 }, 2);
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let mut children = vec![52];
    children.extend(binding.nodes.iter().copied());
    let mut tree = fixture
        .tree::<S>(
            InlineBoundaryEdgeOf::ZERO,
            InlineBoundaryEdgeOf::ZERO,
            false,
            Some(30.0),
            false,
        )
        .children(0, children)
        .style(0, ordinary.clone())
        .children(2, [])
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(px(float_width)),
                    PreferredSizeOf::px(px(20.0)),
                ),
                ..Default::default()
            },
        )
        .children(52, [])
        .style(
            52,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::px(px(10.0))),
                margin: Edges {
                    bottom: LengthAutoOf::px(px(12.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..ordinary
            },
        );
    for (node, kind, padding) in [
        (20, InlineBoundaryKind::Start, 0.0),
        (21, InlineBoundaryKind::End, end_padding),
    ] {
        tree = tree.input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
                kind,
                InlineMetricsOf::try_new(InlineBaselineContext::Physical, px(8.0), px(10.0))
                    .unwrap(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, S::ZERO, px(padding)).unwrap(),
                axes,
            )),
        );
    }
    let inline = GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope.clone()));
    let child = |box_, participation| GeneratedChildBoxInput {
        box_,
        participation,
    };
    let flow = |box_| child(box_, GeneratedBoxParticipation::InFlow);
    let float = child(
        GeneratedBoxRef::Node(2),
        GeneratedBoxParticipation::OutOfFlow,
    );
    let text = flow(GeneratedBoxRef::Node(10));
    let order = |parent, children| InlineChildOrderInput {
        parent,
        children,
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    };
    let wrapping = WrappingInput {
        scopes: vec![WrappingScopeInput::Captured {
            scope,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            // These are the actual endpoint's8/10 metrics. They remain a
            // truthful continuation witness when Start is not on a later line.
            metric_contribution: Some(InlineBoxMetricContribution::Strut(text::LineBoxStrut {
                scope: None,
                metrics: text::LineStrut {
                    baseline: 8.0,
                    line_extent: 10.0,
                },
                alignment: text::LineBoxAlignment::Baseline,
            })),
        }],
        child_order: vec![
            order(
                GeneratedBoxRef::Node(0),
                vec![flow(GeneratedBoxRef::Node(52)), flow(inline.clone())],
            ),
            order(
                inline,
                if before_source {
                    vec![float, text]
                } else {
                    vec![text, float]
                },
            ),
        ],
        ..Default::default()
    };
    let capture = LayoutFlowInput::try_new(&tree, 0, vec![binding], wrapping, Vec::new()).unwrap();
    let width = NonNegativeFiniteOf::new(px(30.0)).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    compute_layout_with_flow(&capture, 0, request).unwrap()
}

fn geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    width: f64,
    top: f64,
    root: f64,
) {
    let px = S::from_f64;
    let prior = single_box_output(batch, 52, Phase::Final);
    assert_eq!((prior.location.y, prior.size.height), (S::ZERO, px(10.0)));
    let float = single_box_output(batch, 2, Phase::Final);
    assert_eq!(float.size, Size::new(px(width), px(20.0)));
    assert_eq!(float.location.y, px(top));
    assert_eq!(
        single_box_output(batch, 0, Phase::Final).size.height,
        px(root)
    );
}

fn continuing_owner<S: LayoutScalar>(end_padding: f64, expected_top: f64, expected_root: f64) {
    let fixture = Fixture::new("X X", (0, 3), text::WhiteSpace::Collapse, false, false);
    let start = fixture
        .binding
        .controls
        .iter()
        .find(|row| row.node == 20)
        .unwrap()
        .occurrence
        .clone();
    let end = fixture
        .binding
        .controls
        .iter()
        .find(|row| row.node == 21)
        .unwrap()
        .occurrence
        .clone();
    let scope = start.captured_control().unwrap().scope();
    let batch = execute::<S>(&fixture, end_padding, 17.0, false);
    // End3/4 makes full X SPACE X width33/34, hence two source lines. F is
    // after the whole original and is reached on line2, whose prefix is X10.
    // End3:10+17+3=30 fits. End4:31>30 defers F past that10px line.
    geometry(&batch, 17.0, expected_top, expected_root);
    let [placed] = batch.placed_text() else {
        panic!("one continuing paragraph")
    };
    assert_eq!(placed.layout().source().text(), &text::Text::from("X X"));
    assert_eq!(
        placed
            .layout()
            .lines()
            .iter()
            .map(|line| line.bounds().origin.y)
            .collect::<Vec<_>>(),
        [22.0, 32.0]
    );
    let line = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.selected_line()
                .request()
                .controls
                .iter()
                .any(|row| row.occurrence == end)
        })
        .expect("the actual closing line owns End21");
    assert!(
        !line
            .selected_line()
            .request()
            .controls
            .iter()
            .any(|row| row.occurrence == start)
    );
    let selected = line.selected_line();
    let closing = selected
        .request()
        .controls
        .iter()
        .find(|row| row.occurrence == end)
        .expect("the actual closing line allocates End21");
    assert_eq!(
        closing
            .boundary
            .as_ref()
            .unwrap()
            .box_context()
            .unwrap()
            .owner(),
        &text::InlineBoxSource::captured(scope),
    );
    let prefix = line
        .prefix_at(&text::TextSeam::BeforeParticipant(
            text::PreparedParticipant::Control(end),
        ))
        .unwrap();
    assert_eq!(prefix.advance(), 10.0);
    assert!(prefix.has_in_flow_content());
    assert_eq!(line.feedback().advance, 10.0 + end_padding);
}

fn canonical_terminal_shy_stops_without_alternative<S: LayoutScalar>() {
    let fixture = Fixture::new("\u{ad}", (0, 1), text::WhiteSpace::Collapse, false, false);
    let origin = fixture.binding.origins[0].origin.clone();
    assert_eq!(origin.view().source().text(), &text::Text::from("\u{ad}"));
    let end = fixture
        .binding
        .controls
        .iter()
        .find(|row| row.node == 21)
        .unwrap()
        .occurrence
        .clone();
    let scope = end.captured_control().unwrap().scope();
    assert_eq!(scope.style().unwrap().hyphens, text::Hyphens::Manual);
    assert!(fixture.binding.prepared.unit_views().any(|unit| {
        unit.empty_origin().is_none()
            && !unit.processed_range().is_empty()
            && unit.origins().any(|row| row.origin() == origin)
    }));
    let batch = execute::<S>(&fixture, 6.0, 25.0, true);
    // Canonical terminal SHY is a nonempty operation with no defining
    // alternative. Its unchanged source role still stops static applicability.
    // F25 fits at22; End6 cannot fit its remaining5px and formats at42.
    geometry(&batch, 25.0, 22.0, 52.0);
    let [placed] = batch.placed_text() else {
        panic!("one conditional-source paragraph")
    };
    assert_eq!(placed.layout().source().text(), &text::Text::from("\u{ad}"));
    assert!(
        placed
            .layout()
            .lines()
            .iter()
            .any(|line| line.bounds().origin.y == 42.0 && line.bounds().size.height == 10.0)
    );
    let line = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.selected_line()
                .request()
                .controls
                .iter()
                .any(|row| row.occurrence == end)
        })
        .expect("the actual selected line retains End6");
    assert!(line.selected_line().request().alternative.is_none());
    assert!(line.selected_line().request().incoming.is_none());
    let source_line = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .chain(
            placed
                .layout()
                .nonformatted_lines()
                .map(|row| row.line.finalized_line().clone()),
        )
        .find(|line| {
            line.mapping_views().any(|mapping| {
                mapping.kind() == text::PresentationMappingKind::Source
                    && mapping.source_range() == text::Range::new(0, 1)
            })
        })
        .expect("the accepted source-owning receipt retains terminal SHY as Source");
    assert_eq!(source_line.prepared_text(), fixture.binding.prepared);
    assert!(source_line.selected_line().request().alternative.is_none());
    assert!(source_line.selected_line().request().incoming.is_none());
    assert_eq!(line.feedback().advance, 6.0);
}

#[test]
fn continuing_owner_exact_closing_demand_keeps_float_on_line_f32() {
    continuing_owner::<f32>(3.0, 32.0, 52.0);
}
#[test]
fn continuing_owner_exact_closing_demand_keeps_float_on_line_f64() {
    continuing_owner::<f64>(3.0, 32.0, 52.0);
}
#[test]
fn continuing_owner_excess_closing_demand_defers_float_f32() {
    continuing_owner::<f32>(4.0, 42.0, 62.0);
}
#[test]
fn continuing_owner_excess_closing_demand_defers_float_f64() {
    continuing_owner::<f64>(4.0, 42.0, 62.0);
}
#[test]
fn canonical_terminal_shy_stops_reservation_without_alternative_f32() {
    canonical_terminal_shy_stops_without_alternative::<f32>();
}
#[test]
fn canonical_terminal_shy_stops_reservation_without_alternative_f64() {
    canonical_terminal_shy_stops_without_alternative::<f64>();
}
