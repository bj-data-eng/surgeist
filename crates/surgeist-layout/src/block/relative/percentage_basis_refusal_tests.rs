//! Accepted percentage placement requires actual measured-basis authority.
use super::{InsetReference, resolve};
use crate::test_support::layout_tree::OracleTreeOf;
use crate::{
    Direction, Display, Edges, FlowAxes, LayoutErrorKindOf, LayoutErrorSiteOf,
    LayoutMissingContext, LayoutOperation, LayoutScalar, LengthAutoOf, LengthPercentageOf,
    NodeInputOf, Point, Position, PreferredSizeOf, Size, WritingMode,
};

fn accepted_missing_basis_refuses_percent_but_allows_length<S: LayoutScalar>() {
    let mut child = NodeInputOf {
        display: Display::Block,
        position: Position::Relative,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(10.0)),
            PreferredSizeOf::px(S::from_f64(10.0)),
        ),
        inset: Edges {
            top: LengthAutoOf::value(
                LengthPercentageOf::from_percent_fraction(S::from_f64(0.04)).unwrap(),
            ),
            ..Edges::all(LengthAutoOf::AUTO)
        },
        ..NodeInputOf::default()
    };
    // A declared50x20 parent does not authorize reconstructing an absent
    // accepted basis. The real OracleCompute adapter owns these ordinary boxes.
    let mut tree = OracleTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(50.0)),
                    PreferredSizeOf::px(S::from_f64(20.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(1, child.clone());
    let axes = FlowAxes::new(
        tree.node_input(0).writing_mode,
        tree.node_input(0).direction,
    );
    let inset = tree.node_input(1).inset;
    let outcome = resolve(
        &tree.borrow_compute(),
        1,
        inset,
        InsetReference::Accepted(None),
        axes,
    );
    let Err(error) = outcome else {
        panic!("a demanded percentage cannot use an absent accepted basis");
    };
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
    ));

    child.inset = Edges {
        top: LengthAutoOf::px(S::from_f64(2.0)),
        left: LengthAutoOf::px(S::from_f64(3.0)),
        ..Edges::all(LengthAutoOf::AUTO)
    };
    tree.insert_style(1, child);
    let inset = tree.node_input(1).inset;
    let offset = resolve(
        &tree.borrow_compute(),
        1,
        inset,
        InsetReference::Accepted(None),
        axes,
    )
    .expect("length-only placement has no percentage-basis demand");
    assert_eq!(offset, Point::new(S::from_f64(3.0), S::from_f64(2.0)));
}

#[test]
fn accepted_missing_percentage_basis_is_typed_while_lengths_still_resolve_f32() {
    accepted_missing_basis_refuses_percent_but_allows_length::<f32>();
}
#[test]
fn accepted_missing_percentage_basis_is_typed_while_lengths_still_resolve_f64() {
    accepted_missing_basis_refuses_percent_but_allows_length::<f64>();
}
