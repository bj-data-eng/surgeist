use super::*;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};

fn percentage_shape_tree<S: LayoutScalar>(
    shape: bool,
    top: LengthAutoOf<S>,
) -> Fri06C02StatefulTextTree<S> {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::at_boundary(
        vec![ParagraphPart::text(
            2,
            surgeist_text::Source::new("X X"),
            style.clone(),
        )],
        style,
        surgeist_text::ParagraphBaseDirection::LeftToRight,
        surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
    );
    let mut tree = if shape {
        Fri06C02StatefulTextTree::new_shape_provider(Fri06C05ShapeProvider::Interval {
            minimum: S::ZERO,
            maximum: S::from_f64(8.0),
        })
    } else {
        Fri06C02StatefulTextTree::new_float_mixed()
    };
    tree.inputs.insert(2, paragraph.text_input(2));
    tree.text = paragraph.binding(0);
    tree.children.insert(0, vec![1, 2, 3]);
    tree.children.insert(3, Vec::new());
    tree.inputs.insert(
        3,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            position: Position::Relative,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(40.0)),
                PreferredSizeOf::px(S::from_f64(10.0)),
            ),
            inset: Edges {
                top,
                ..Edges::all(LengthAutoOf::AUTO)
            },
            ..NodeInputOf::default()
        }),
    );
    assert_eq!(tree.text.nodes, [2]);
    assert_eq!(
        tree.node_input(1).float_exclusion,
        if shape {
            FloatExclusion::Shape
        } else {
            FloatExclusion::MarginBox
        }
    );
    tree
}

fn percentage_shape_facts<S: LayoutScalar>(
    tree: &Fri06C02StatefulTextTree<S>,
    batch: &CompletedLayoutBatchOf<u32, S>,
    shape: bool,
) {
    // The real float is15.25 wide in a40px container. Its margin-box band
    // leaves24.75, so Ahem "X X" (30px) needs two10px lines. The actual shape
    // interval0..8 leaves32 and fits one line. The following normal Block10
    // therefore makes final contentH30 versus20; relative placement is visual.
    let height = if shape { 20.0 } else { 30.0 };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries.iter().find(|row| row.node() == 0).unwrap().output();
        assert_eq!(root.size, Size::new(S::from_f64(40.0), S::from_f64(height)));
    }
    let placed = batch.placed_text();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].container(), 0);
    let text = placed[0].layout();
    assert_eq!(text.source().text().code_units(), &[88, 32, 88]);
    assert_eq!(text.lines().len(), if shape { 1 } else { 2 });
    for run in text.run_views() {
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.face().index(), 0);
        assert_eq!(
            run.face().bytes(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            .as_slice()
        );
    }
    if shape {
        assert!(tree.shape_queries.get() > 0);
        assert_eq!(
            tree.shape_queries.get(),
            tree.shape_query_inputs.borrow().len()
        );
    }
}

fn final_percentage_uses_actual_shape_height<S: LayoutScalar>() {
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(
            Some(
                crate::NonNegativeFiniteOf::new(S::from_f64(40.0))
                    .expect("declared fixture extent is finite and nonnegative"),
            ),
            None,
        ),
        crate::OrthogonalFallbackOf::new(
            crate::Size::NONE,
            crate::NearestScrollportOf::Unknown,
            crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
        ),
    )
    .request(Size::new(
        crate::AxisSizingRequestOf::new(
            crate::AvailableSpaceOf::Definite(
                crate::NonNegativeFiniteOf::new(S::from_f64(40.0))
                    .expect("fixture physical extent is finite and nonnegative"),
            ),
            crate::SizingOperation::Normal,
        ),
        crate::AxisSizingRequestOf::new(
            crate::AvailableSpaceOf::Indefinite,
            crate::SizingOperation::Normal,
        ),
    ));
    for shape in [true, false] {
        let control_tree = percentage_shape_tree::<S>(shape, LengthAutoOf::AUTO);
        let control = control_tree.compute(request).unwrap();
        percentage_shape_facts(&control_tree, &control, shape);
        let before = if shape { 10.0 } else { 20.0 };
        let unshifted = control
            .unrounded_entries()
            .iter()
            .find(|row| row.node() == 3)
            .unwrap()
            .output();
        assert_eq!(unshifted.location, Point::new(S::ZERO, S::from_f64(before)));

        let percentage = LengthAutoOf::value(
            LengthPercentageOf::from_percent_fraction(S::from_f64(0.1)).unwrap(),
        );
        let tree = percentage_shape_tree::<S>(shape, percentage);
        let actual = tree.compute(request).unwrap();
        percentage_shape_facts(&tree, &actual, shape);
        let expected = before + if shape { 2.0 } else { 3.0 };
        for entries in [actual.unrounded_entries(), actual.final_entries()] {
            let relative = entries.iter().find(|row| row.node() == 3).unwrap().output();
            assert_eq!(
                relative.location,
                Point::new(S::ZERO, S::from_f64(expected))
            );
            assert_eq!(
                relative.size,
                Size::new(S::from_f64(40.0), S::from_f64(10.0))
            );
        }
        // The previously completed result is immutable, with its old source
        // publication and unshifted normal placement still available.
        assert_eq!(
            control
                .unrounded_entries()
                .iter()
                .find(|row| row.node() == 3)
                .unwrap()
                .output(),
            unshifted
        );
    }
}

#[test]
fn relative_percentage_uses_actual_shape_height_f32() {
    final_percentage_uses_actual_shape_height::<f32>();
}

#[test]
fn relative_percentage_uses_actual_shape_height_f64() {
    final_percentage_uses_actual_shape_height::<f64>();
}
