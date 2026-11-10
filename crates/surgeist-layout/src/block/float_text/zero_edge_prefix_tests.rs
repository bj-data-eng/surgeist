//! Fitting a real captured float after an allocated, nominal-only prefix.
use crate::test_support::inline_endpoint::Fixture;
use crate::test_support::readback::{Phase, single_box_output};
use crate::*;

fn request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(30.0)).unwrap();
    crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
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
    ))
}

fn fitting_float_after_nominal_prefix<S: LayoutScalar>() {
    let px = S::from_f64;
    // The float-first control exercises the same actual captured replay owner.
    // Only F's source position relative to the opening control changes.
    for after_opening in [false, true] {
        let fixture = Fixture::new(
            "A B C D",
            (0, 7),
            surgeist_text::WhiteSpace::Collapse,
            false,
            false,
        );
        let mut binding = fixture.binding.clone();
        binding.nodes.insert(usize::from(after_opening), 2);
        let opening = binding
            .controls
            .iter()
            .find(|row| row.node == 20)
            .unwrap()
            .occurrence
            .clone();
        let ordinary = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
            min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
            ..Default::default()
        };
        let mut children = vec![52];
        children.extend(binding.nodes.iter().copied());
        let tree = fixture
            .tree::<S>(
                InlineBoundaryEdgeOf::ZERO,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(30.0),
                true,
            )
            .children(0, children)
            .style(0, ordinary.clone())
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
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request()).unwrap();
        let [placed] = batch.placed_text() else {
            panic!("one authentic processed paragraph");
        };
        let layout = placed.layout();
        assert_eq!(
            layout.source().text(),
            &surgeist_text::Text::from("A B C D")
        );
        let receipt = batch
            .final_inline_fragments()
            .iter()
            .map(|entry| entry.fragment().finalized_fragment().finalized_line())
            .find(|line| {
                line.selected_line()
                    .request()
                    .controls
                    .iter()
                    .any(|row| row.occurrence == opening)
            })
            .expect("the actual opening allocation retains its line receipt");
        let prefix = receipt
            .prefix_at(&surgeist_text::TextSeam::AfterParticipant(
                surgeist_text::PreparedParticipant::Control(opening),
            ))
            .unwrap();
        assert_eq!(prefix.advance(), 0.0);
        assert!(prefix.has_retained_flow_participation());
        assert!(!prefix.has_in_flow_content());
        let before = single_box_output(&batch, 52, Phase::Final);
        assert_eq!((before.location.y, before.size.height), (S::ZERO, px(10.0)));
        let float = single_box_output(&batch, 2, Phase::Final);
        assert_eq!(float.size, Size::new(px(25.0), px(20.0)));
        // A's bottom10 + adjoining margin12 gives source22. The nominal
        // prefix has zero advance and zero edges, so F25 fits width30 at22.
        // Its remaining5px cannot hold an Ahem10 word: actual lines are42/52.
        assert_eq!(
            (
                float.location.y,
                layout
                    .lines()
                    .iter()
                    .map(|line| line.bounds().origin.y)
                    .collect::<Vec<_>>(),
                single_box_output(&batch, 0, Phase::Final).size.height,
            ),
            (px(22.0), vec![42.0, 52.0], px(62.0)),
            "float after opening control: {after_opening}",
        );
    }
}

#[test]
fn fitting_float_after_nominal_zero_edge_prefix_f32() {
    fitting_float_after_nominal_prefix::<f32>();
}

#[test]
fn fitting_float_after_nominal_zero_edge_prefix_f64() {
    fitting_float_after_nominal_prefix::<f64>();
}
