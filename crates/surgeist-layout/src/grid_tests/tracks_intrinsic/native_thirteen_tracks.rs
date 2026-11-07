//! Independent complete native thirteen-track intrinsic/normal geometry.
use super::*;
use crate::test_support::readback::{self, Phase};
use crate::{LayoutTree, Traverse};
use std::cell::RefCell;

struct ThirteenContent<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    calls: RefCell<Vec<LeafMeasureInputOf<S>>>,
}
impl<S: LayoutScalar> Traverse for ThirteenContent<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}
impl<S: LayoutScalar> LayoutTree for ThirteenContent<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        node == 1
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != 1 {
            return None;
        }
        self.calls.borrow_mut().push(input);
        let n = S::from_f64;
        let known = input.known_content_size();
        // Two native unbreakable160-wide pieces. This is a leaf size provider,
        // never a supplied Grid result, track vector, source or baseline.
        let width = known.width.unwrap_or_else(|| {
            let axis = input.content_sizing_request().axes().width;
            match axis.operation() {
                SizingOperation::MinContent => n(160.0),
                SizingOperation::MaxContent => n(320.0),
                SizingOperation::Normal => axis
                    .space()
                    .definite_size()
                    .map_or(n(320.0), |band| band.get().max(n(160.0)).min(n(320.0))),
            }
        });
        Some(Ok(Size::new(width, known.height.unwrap_or(n(40.0)))))
    }
}

fn columns<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    let n = S::from_f64;
    let percentage =
        SizingCalculationOf::value(LengthPercentageOf::from_coefficients(S::ZERO, n(0.2)).unwrap());
    let fixed_min = |maximum| {
        TrackComponentOf::Track(TrackSizingOf::minmax(MinTrackSizingOf::px(n(2.0)), maximum))
    };
    vec![
        TrackComponentOf::Track(TrackSizingOf::MIN_CONTENT),
        TrackComponentOf::Track(TrackSizingOf::MAX_CONTENT),
        TrackComponentOf::Track(TrackSizingOf::fit_content(SizingCalculationOf::value(
            LengthPercentageOf::px(n(20.0)).unwrap(),
        ))),
        TrackComponentOf::AUTO,
        TrackComponentOf::px(n(10.0)),
        TrackComponentOf::Track(TrackSizingOf::minmax(
            MinTrackSizingOf::Calculation(percentage.clone()),
            MaxTrackSizingOf::Calculation(percentage),
        )),
        fixed_min(MaxTrackSizingOf::AUTO),
        fixed_min(MaxTrackSizingOf::px(n(4.0))),
        fixed_min(MaxTrackSizingOf::MIN_CONTENT),
        fixed_min(MaxTrackSizingOf::MAX_CONTENT),
        TrackComponentOf::Track(TrackSizingOf::minmax(
            MinTrackSizingOf::MIN_CONTENT,
            MaxTrackSizingOf::MAX_CONTENT,
        )),
        TrackComponentOf::Track(TrackSizingOf::minmax(
            MinTrackSizingOf::MIN_CONTENT,
            MaxTrackSizingOf::AUTO,
        )),
        TrackComponentOf::Track(TrackSizingOf::minmax(
            MinTrackSizingOf::MAX_CONTENT,
            MaxTrackSizingOf::AUTO,
        )),
    ]
}

fn source<S: LayoutScalar>(overflow: Overflow) -> ThirteenContent<S> {
    let n = S::from_f64;
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, 1..=14)
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                box_sizing: BoxSizing::BorderBox,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                grid_template_columns: columns(),
                grid_template_rows: vec![TrackComponentOf::px(n(40.0)); 2],
                gap: Size::splat(LengthOf::px(S::ZERO)),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        )
        .children(1, [])
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                overflow: computed_overflow(overflow, overflow),
                grid_column: GridPlacement::try_lines(1, 14).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                ..NodeInputOf::default()
            },
        );
    // Empty Flex boxes have zero intrinsic demand. Stretch later makes
    // their actual border boxes observable markers of all13 track allocations.
    for index in 0..13 {
        tree = tree.children(index + 2, []).style(
            index + 2,
            NodeInputOf {
                display: Display::Flex,
                grid_column: GridPlacement::try_lines(index as isize + 1, index as isize + 2)
                    .unwrap(),
                grid_row: GridPlacement::try_lines(2, 3).unwrap(),
                ..NodeInputOf::default()
            },
        );
    }
    ThirteenContent {
        tree,
        calls: RefCell::new(Vec::new()),
    }
}

fn request<S: LayoutScalar>(operation: SizingOperation) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, operation),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    )
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    let mut rows = entries.iter().filter(|row| row.node() == node);
    let result = rows
        .next()
        .expect("actual native node has geometry")
        .output();
    assert!(rows.next().is_none(), "actual node is published once");
    result
}

fn close<S: LayoutScalar>(actual: S, expected: f64, tolerance: f64) {
    assert!(
        (actual.to_f64() - expected).abs() <= tolerance,
        "actual {actual:?}, independently derived {expected}"
    );
}

fn qualify<S: LayoutScalar>(tree: &ThirteenContent<S>) {
    let n = S::from_f64;
    assert_eq!(
        Traverse::children(tree, 0).collect::<Vec<_>>(),
        (1..=14).collect::<Vec<_>>()
    );
    assert_eq!(tree.node_input(0).grid_template_columns, columns());
    assert_eq!(
        tree.node_input(0).grid_template_rows,
        vec![TrackComponentOf::px(n(40.0)); 2]
    );
    assert_eq!(
        tree.node_input(0).size,
        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
    );
    for node in 0..=14 {
        let style = tree.node_input(node);
        assert_eq!(style.margin, Edges::all(LengthAutoOf::px(S::ZERO)));
        assert_eq!(style.border, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(style.padding, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(tree.has_leaf_measurement(node), node == 1);
        if node != 0 {
            assert_eq!(tree.child_count(node), 0);
        }
    }
    for (operation, width) in [
        (SizingOperation::MinContent, 160.0),
        (SizingOperation::MaxContent, 320.0),
    ] {
        let query = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::Grid,
            ),
            request::<S>(operation).sizing_request(),
        );
        let measured = crate::measurement::compute_tree_leaf(tree, 1, query, tree.node_input(1))
            .expect("actual native leaf query succeeds");
        assert_eq!(measured.size, Size::new(n(width), n(40.0)));
        assert_eq!(measured.content_size, measured.size);
        assert_eq!(measured.baselines(), BaselinesOf::NONE);
        assert_eq!(measured.named_baselines, NamedBaselinesOf::unknown());
        assert!(
            tree.calls
                .borrow()
                .iter()
                .any(|call| call.known_content_size().width.is_none()
                    && call.content_sizing_request().axes().width.operation() == operation)
        );
    }
    for node in 2..=14 {
        for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
            let marker = compute_layout(tree, node, request(operation))
                .expect("real empty marker completes");
            assert_eq!(
                readback::single_box_output(&marker, node, Phase::Unrounded).size,
                Size::ZERO
            );
        }
    }
    tree.calls.borrow_mut().clear();
}

fn edges(operation: SizingOperation, overflow: Overflow) -> ([f64; 14], [f64; 14]) {
    // Independent Grid1 §11.5.1/§11.6 arithmetic. Initial intrinsic fixed
    // occupancy18; nonaffected2..4 absorbs2, leaving140/8=17.5. In formal
    // MaxContent, fit20 takes2.5 then remaining157.5/4=39.375.
    if operation == SizingOperation::MaxContent {
        return (
            [
                0., 17.5, 74.375, 94.375, 151.25, 161.25, 218.125, 220.125, 224.125, 226.125,
                228.125, 245.625, 263.125, 320.,
            ],
            [
                0., 18., 74., 94., 151., 161., 218., 220., 224., 226., 228., 246., 263., 320.,
            ],
        );
    }
    // Accepted Normal content320 resolves20% to64. Fixed bases82 become84
    // when nonaffected2..4 absorbs2. Hidden0 minimum leaves the content-min
    // phase to distribute76 over5 tracks. Clip's content minimum160 instead
    // distributes76 over7 intrinsic-min tracks. Both then add160/2=80 to
    // the two MaxContent-min tracks. Neither reuses the intrinsic vector.
    if overflow == Overflow::Hidden {
        (
            [
                0., 15.2, 110.4, 110.4, 110.4, 120.4, 184.4, 186.4, 190.4, 192.4, 194.4, 209.6,
                224.8, 320.,
            ],
            [
                0., 15., 110., 110., 110., 120., 184., 186., 190., 192., 194., 210., 225., 320.,
            ],
        )
    } else {
        (
            [
                0.,
                76. / 7.,
                712. / 7.,
                788. / 7.,
                864. / 7.,
                934. / 7.,
                1382. / 7.,
                1396. / 7.,
                1424. / 7.,
                1438. / 7.,
                1452. / 7.,
                1528. / 7.,
                1604. / 7.,
                320.,
            ],
            [
                0., 11., 102., 113., 123., 133., 197., 199., 203., 205., 207., 218., 229., 320.,
            ],
        )
    }
}

fn complete<S: LayoutScalar>(operation: SizingOperation, overflow: Overflow, tolerance: f64) {
    let n = S::from_f64;
    let tree = source::<S>(overflow);
    qualify(&tree);
    let batch = compute_layout(&tree, 0, request(operation)).expect("real13-track Grid completes");
    assert!(
        batch.placed_text().is_empty(),
        "this native host issues no Text/font/source identity"
    );
    let (raw_edges, published_edges) = edges(operation, overflow);
    for (published, entries) in [
        (
            false,
            readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
        ),
        (
            true,
            readback::continuous_custody(&batch, Phase::Final).as_slice(),
        ),
    ] {
        assert_eq!(entries.len(), 15);
        let root = output(entries, 0);
        assert_eq!(root.location, Point::ZERO);
        assert_eq!(root.size, Size::new(n(320.0), n(80.0)));
        assert_eq!(root.source_index, SourceIndex::new(0));
        close(root.content_size.width, 320., tolerance);
        close(root.content_size.height, 80., tolerance);
        let subject = output(entries, 1);
        assert_eq!(subject.source_index, SourceIndex::new(0));
        assert_eq!(subject.location, Point::ZERO);
        close(subject.size.width, 320., tolerance);
        assert_eq!(subject.size.height, n(40.0));
        close(subject.content_size.width, 320., tolerance);
        assert_eq!(subject.content_size.height, n(40.0));
        let expected = if published {
            published_edges
        } else {
            raw_edges
        };
        for index in 0..13 {
            let marker = output(entries, index as u32 + 2);
            assert_eq!(marker.source_index, SourceIndex::new(index + 1));
            close(
                marker.location.x,
                expected[index],
                if published { 0. } else { tolerance },
            );
            assert_eq!(marker.location.y, n(40.0));
            close(
                marker.size.width,
                expected[index + 1] - expected[index],
                if published { 0. } else { tolerance },
            );
            assert_eq!(marker.size.height, n(40.0));
        }
        for node in 0..=14 {
            let item = output(entries, node);
            assert_eq!(item.margin, Edges::ZERO);
            assert_eq!(item.border, Edges::ZERO);
            assert_eq!(item.padding, Edges::ZERO);
            assert_eq!(item.content_box_size(), item.size);
            let scroll = item
                .scroll_geometry
                .expect("accepted native box retains scroll geometry");
            assert_eq!(scroll.border_box().origin(), Point::ZERO);
            assert_eq!(scroll.border_box().size(), item.size);
            for rect in [
                scroll.padding_box(),
                scroll.content_box(),
                scroll.scrollport(),
                scroll.scrollable_overflow(),
            ] {
                assert_eq!(rect.origin(), Point::ZERO);
                close(
                    rect.size().width,
                    item.size.width.to_f64(),
                    if published { 0. } else { tolerance },
                );
                assert_eq!(rect.size().height, item.size.height);
            }
            let own_overflow = if node == 1 {
                overflow
            } else {
                Overflow::Visible
            };
            assert_eq!(scroll.used_overflow_x(), own_overflow);
            assert_eq!(scroll.used_overflow_y(), own_overflow);
            assert_eq!(scroll.scrollbar_size(), Size::ZERO);
            let range = scroll.physical_range();
            close(range.x().minimum(), 0., tolerance);
            close(range.x().maximum(), 0., tolerance);
            close(range.y().minimum(), 0., tolerance);
            close(range.y().maximum(), 0., tolerance);
        }
    }
}

macro_rules! cohort {
    ($name:ident, $scalar:ty, $operation:expr, $overflow:expr, $tolerance:expr) => {
        #[test]
        fn $name() {
            complete::<$scalar>($operation, $overflow, $tolerance);
        }
    };
}
// Raw fifths/sevenths require finite-lane arithmetic error bounds. Publication
// endpoints are exact integers and remain exact assertions in both lanes.
// The raw bounds are below four f32 ULPs / eighteen f64 ULPs at320;
// they cover only private representability, never browser-corpus comparison.
cohort!(
    formal_max_hidden_f32,
    f32,
    SizingOperation::MaxContent,
    Overflow::Hidden,
    0.0001
);
cohort!(
    formal_max_hidden_f64,
    f64,
    SizingOperation::MaxContent,
    Overflow::Hidden,
    0.000000000001
);
cohort!(
    formal_max_content_minimum_f32,
    f32,
    SizingOperation::MaxContent,
    Overflow::Clip,
    0.0001
);
cohort!(
    formal_max_content_minimum_f64,
    f64,
    SizingOperation::MaxContent,
    Overflow::Clip,
    0.000000000001
);
cohort!(
    normal_hidden_f32,
    f32,
    SizingOperation::Normal,
    Overflow::Hidden,
    0.0001
);
cohort!(
    normal_hidden_f64,
    f64,
    SizingOperation::Normal,
    Overflow::Hidden,
    0.000000000001
);
cohort!(
    normal_content_minimum_f32,
    f32,
    SizingOperation::Normal,
    Overflow::Clip,
    0.0001
);
cohort!(
    normal_content_minimum_f64,
    f64,
    SizingOperation::Normal,
    Overflow::Clip,
    0.000000000001
);
