//! A float remains live across authentic parent/child source interruptions.
use super::*;

const FLOAT: u32 = 90;
const WRAPPER: u32 = 100;

fn assert_interrupted_scene<S: LayoutScalar>(
    fixture: &Fixture,
    batch: &CompletedLayoutBatchOf<u32, S>,
    joined: bool,
    shared: bool,
) {
    assert_eq!(batch.placed_text().len(), if joined { 2 } else { 3 });
    let mut actual = batch
        .placed_text()
        .iter()
        .flat_map(|output| {
            let layout = output.layout();
            assert_eq!(layout.source(), &fixture.source);
            let expected = if output.container() == BLOCK {
                &fixture.prepared[1]
            } else {
                assert_eq!(output.container(), ROOT);
                if joined || output.nodes().contains(&A) {
                    &fixture.prepared[0]
                } else {
                    assert_eq!(output.nodes(), [C]);
                    &fixture.prepared[2]
                }
            };
            provenance(layout, expected);
            packets(layout)
        })
        .collect::<Vec<_>>();
    actual.sort_unstable_by_key(|value| value.0.start);
    // The live5px float leaves25px. A/C use10px glyphs and10px lines;
    // B uses a20px glyph/line. Each fits beside the float until its42px end.
    assert_eq!(
        actual,
        [
            (text::Range::new(0, 1), 35, 5.0, 8.0, 10.0),
            (text::Range::new(1, 2), 36, 5.0, 26.0, 20.0),
            (text::Range::new(2, 3), 37, 5.0, 38.0, 10.0),
        ]
    );
    for phase in [Phase::Unrounded, Phase::Final] {
        let floating = readback::single_box_output(batch, FLOAT, phase);
        assert_eq!(floating.location, Point::ZERO);
        assert_eq!(
            floating.size,
            Size::new(S::from_f64(5.0), S::from_f64(42.0))
        );
        let e = readback::single_box_output(batch, ROOT, phase);
        assert_eq!(e.location, Point::ZERO);
        assert_eq!(
            e.size,
            Size::new(
                S::from_f64(30.0),
                S::from_f64(if shared { 40.0 } else { 42.0 })
            )
        );
        let child = readback::single_box_output(batch, BLOCK, phase);
        assert_eq!(child.location, Point::new(S::ZERO, S::from_f64(10.0)));
        assert_eq!(child.size, Size::new(S::from_f64(30.0), S::from_f64(20.0)));
        if shared {
            let outer = readback::single_box_output(batch, WRAPPER, phase);
            assert_eq!(outer.location, Point::ZERO);
            assert_eq!(outer.size, Size::new(S::from_f64(100.0), S::from_f64(42.0)));
        }
    }
    if joined {
        let parent: Vec<_> = batch
            .placed_text()
            .iter()
            .filter(|output| output.container() == ROOT)
            .collect();
        assert_eq!(parent.len(), 1);
        assert_eq!(parent[0].layout().lines().len(), 2);
        assert_eq!(
            packets(parent[0].layout())
                .into_iter()
                .map(|row| row.0)
                .collect::<Vec<_>>(),
            [text::Range::new(0, 1), text::Range::new(2, 3)]
        );
    }
}

fn interrupted_float<S: LayoutScalar>(joined: bool) {
    let fixture = fixture(Shape::Child { joined });
    assert_eq!(fixture.raws.len(), 3);
    if joined {
        let owner = fixture.prepared[0].captured_owner().unwrap();
        assert_eq!(
            owner.members().collect::<Vec<_>>(),
            [fixture.raws[0].clone(), fixture.raws[2].clone()]
        );
        assert_eq!(fixture.prepared[0].source_segments().len(), 2);
    }
    let (tree, mut bindings) = bind::<S>(&fixture);
    assert_eq!(bindings[0].nodes, if joined { vec![A, C] } else { vec![A] });
    bindings[0].nodes.insert(0, FLOAT);
    if !joined {
        assert_eq!(bindings[2].nodes, [C]);
    }
    let tree = tree
        .children(ROOT, [FLOAT, A, BLOCK, C])
        .children(FLOAT, [])
        .style(
            ROOT,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(30.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..Default::default()
            },
        )
        .style(
            FLOAT,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(5.0)),
                    PreferredSizeOf::px(S::from_f64(42.0)),
                ),
                ..Default::default()
            },
        );
    let owning_capture = capture(&tree, bindings.clone());
    let owning = compute_layout_with_flow(
        &owning_capture,
        ROOT,
        request(
            100.0,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .unwrap();
    assert_interrupted_scene(&fixture, &owning, joined, false);

    let shared_tree = tree.children(WRAPPER, [ROOT]).style(
        WRAPPER,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(100.0)),
                PreferredSizeOf::AUTO,
            ),
            ..Default::default()
        },
    );
    let shared_capture = LayoutFlowInput::try_new(
        &shared_tree,
        WRAPPER,
        bindings,
        crate::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let shared = compute_layout_with_flow(
        &shared_capture,
        WRAPPER,
        request(
            100.0,
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        ),
    )
    .unwrap();
    assert_interrupted_scene(&fixture, &shared, joined, true);
}

#[test]
fn interrupted_joint_float_preserves_shared_and_owning_extents_f32() {
    interrupted_float::<f32>(true);
}

#[test]
fn interrupted_joint_float_preserves_shared_and_owning_extents_f64() {
    interrupted_float::<f64>(true);
}

#[test]
fn interrupted_independent_float_preserves_shared_and_owning_extents_f32() {
    interrupted_float::<f32>(false);
}

#[test]
fn interrupted_independent_float_preserves_shared_and_owning_extents_f64() {
    interrupted_float::<f64>(false);
}
