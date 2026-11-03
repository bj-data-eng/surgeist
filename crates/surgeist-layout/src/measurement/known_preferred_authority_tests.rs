//! Fully allocated body dimensions do not demand the leaf's own preferred size.
use crate::*;
use core::convert::Infallible;

fn leaf_input<S: LayoutScalar>(known: Size<Option<S>>) -> ComputeInputOf<S> {
    let dimension = |value| NonNegativeFiniteOf::new(S::from_f64(value)).unwrap();
    let request = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(dimension(20.0)),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(dimension(10.0)),
                SizingOperation::Normal,
            ),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Unknown,
            Size::splat(OrthogonalFallbackLimitOf::Unknown),
        ),
    );
    ComputeInputOf::leaf_layout(
        known,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        request,
    )
    .unwrap()
}

fn percentage<S: LayoutScalar>() -> LengthPercentageOf<S> {
    LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).unwrap()
}

fn requires_basis<S: LayoutScalar>(known: Size<Option<S>>, style: &NodeInputOf<S>, context: &str) {
    let error = compute_leaf(
        leaf_input(known),
        style,
        |_| -> Result<Size<S>, Infallible> {
            panic!("{context}: required-basis refusal precedes native measurement")
        },
    )
    .expect_err(context);
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::Standalone,
        "{context}: site"
    );
    assert_eq!(
        error.operation(),
        LayoutOperation::ValueResolution,
        "{context}: operation"
    );
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
        ),
        "{context}: precise missing percentage basis, got {error:?}"
    );
}

fn qualify<S: LayoutScalar>() {
    let native = Size::new(S::from_f64(20.0), S::from_f64(10.0));
    let known = native.map(Some);
    let automatic = NodeInputOf::<S>::default();
    let input = leaf_input(known);
    assert_eq!(input.known(), known);
    assert_eq!(input.parent(), Size::NONE);
    let control = compute_leaf(input, &automatic, |_| Ok::<_, Infallible>(native)).unwrap();
    assert_eq!(control.size, native);
    assert_eq!(control.content_size, native);

    let mut bounded = automatic.clone();
    bounded.min_size.width = MinSizeOf::px(S::from_f64(25.0));
    bounded.max_size.height = MaxSizeOf::px(S::from_f64(8.0));
    let bounded = compute_leaf(input, &bounded, |_| Ok::<_, Infallible>(native)).unwrap();
    assert_eq!(
        bounded.size,
        Size::new(S::from_f64(25.0), S::from_f64(8.0)),
        "genuine known body still obeys independently authored min/max constraints"
    );
    for minimum in [true, false] {
        let mut needs_basis = automatic.clone();
        if minimum {
            needs_basis.min_size.height = MinSizeOf::value(percentage());
        } else {
            needs_basis.max_size.height = MaxSizeOf::value(percentage());
        }
        requires_basis(
            known,
            &needs_basis,
            if minimum {
                "known body still needs minimum basis"
            } else {
                "known body still needs maximum basis"
            },
        );
    }

    let mut preferred = automatic;
    preferred.size.height = PreferredSizeOf::value(percentage());
    for partial in [
        Size::NONE,
        Size::new(Some(native.width), None),
        Size::new(None, Some(native.height)),
    ] {
        requires_basis(
            partial,
            &preferred,
            "unknown or partial body retains opaque percentage refusal",
        );
    }

    let mut observed = None;
    let output = compute_leaf(input, &preferred, |packet| {
        observed = Some(packet);
        Ok::<_, Infallible>(native)
    })
    .expect("both genuine known dimensions override the leaf's own preferred percentage");
    let packet = observed.expect("performed layout retains real native content measurement");
    assert_eq!(
        packet.known_content_size(),
        Size::NONE,
        "performed provider reports intrinsic content independently of the imposed body"
    );
    assert_eq!(
        packet.content_sizing_request().axes(),
        input.sizing_request().axes()
    );
    assert_eq!(
        packet.containing_layout_context(),
        input.containing_layout_context()
    );
    assert_eq!(output.size, native);
    assert_eq!(output.content_size, native);
    let scroll = output
        .scroll_geometry
        .expect("performed leaf retains actual scroll geometry");
    for rect in [
        scroll.border_box(),
        scroll.padding_box(),
        scroll.content_box(),
        scroll.scrollport(),
        scroll.scrollable_overflow(),
    ] {
        assert_eq!(rect.origin(), Point::ZERO);
        assert_eq!(rect.size(), native);
    }
    let range = scroll.physical_range();
    assert_eq!(
        (
            range.x().minimum(),
            range.x().maximum(),
            range.y().minimum(),
            range.y().maximum()
        ),
        (S::ZERO, S::ZERO, S::ZERO, S::ZERO)
    );
}

#[test]
fn fully_known_leaf_body_ignores_only_own_preferred_percentage_f32() {
    qualify::<f32>();
}
#[test]
fn fully_known_leaf_body_ignores_only_own_preferred_percentage_f64() {
    qualify::<f64>();
}
