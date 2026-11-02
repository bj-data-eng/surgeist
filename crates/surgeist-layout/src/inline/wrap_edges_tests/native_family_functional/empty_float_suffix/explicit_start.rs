//! An explicit FormatterStart attachment cannot borrow a selected End port.
use super::*;
use crate::{InlineFormatterEdge, InlineGapAnchor, InlineGapGroupInput};

fn explicit_start_remains_cut_local<S: LayoutScalar>() {
    let f = empty_float_fixture::<S>(false);
    let ordinary_flow = f.flow();
    let prior = compute_layout_with_flow(&ordinary_flow, 0, request(n(100.0)))
        .expect("the actual ordinary no-group Start/float/End control must complete first");
    ordinary_empty_float_geometry(&prior, false);
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(prior.native_inline_box_pieces().count(), 0);
    assert!(f.wrapping.gaps.is_empty());

    let mut demanded = f.wrapping.clone();
    demanded
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    demanded.gaps.push(InlineGapGroupInput {
        anchor: InlineGapAnchor::Formatter {
            node: 0,
            edge: InlineFormatterEdge::Start,
        },
        boxes: vec![InlineBoxRef::NativePair { start: 20, end: 21 }],
    });
    // This is the actual formatter's Start in P's one complete body, with no
    // foreign box, missing order or fabricated content. Global admission must
    // succeed; whether the exact source edge is selected belongs to execution.
    let flow = LayoutFlowInput::try_new(&f.tree, 0, vec![], demanded, Vec::new())
        .expect("the explicit FormatterStart attachment is valid for the real complete body");
    let result = compute_layout_with_flow(&flow, 0, request(n(100.0)));
    // The float separates raw runs0..1 and2..3. P.End is actual selected physical
    // authority in the second run, but does not replace the explicitly requested
    // FormatterStart at raw0 with a derived End or an unselected coordinate.
    ordinary_empty_float_geometry(&prior, false);
    let retry = compute_layout_with_flow(&ordinary_flow, 0, request(n(100.0))).unwrap();
    ordinary_empty_float_geometry(&retry, false);
    let Err(error) = result else {
        panic!("an explicit FormatterStart must not borrow a derived suffix on the End-only run");
    };
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::WrappingScope(
                WrappingScopeError::GapOutsideSelection
            ))
        ),
        "unexpected selected-cut refusal: {error:?}"
    );
    ordinary_empty_float_geometry(&prior, false);
    assert_eq!(prior.native_inline_box_families().count(), 0);
    assert_eq!(prior.native_inline_box_pieces().count(), 0);
}

#[test]
fn explicit_formatter_start_cannot_borrow_empty_float_suffix_f32() {
    explicit_start_remains_cut_local::<f32>();
}
#[test]
fn explicit_formatter_start_cannot_borrow_empty_float_suffix_f64() {
    explicit_start_remains_cut_local::<f64>();
}
