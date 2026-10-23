use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::*;

#[test]
fn scalar_style_keywords_share_exact_bounded_serialization() {
    for (style, expected) in [
        (CssBorderStyle::None, "none"),
        (CssBorderStyle::Hidden, "hidden"),
        (CssBorderStyle::Dotted, "dotted"),
        (CssBorderStyle::Dashed, "dashed"),
        (CssBorderStyle::Solid, "solid"),
        (CssBorderStyle::Double, "double"),
        (CssBorderStyle::Groove, "groove"),
        (CssBorderStyle::Ridge, "ridge"),
        (CssBorderStyle::Inset, "inset"),
        (CssBorderStyle::Outset, "outset"),
    ] {
        assert_eq!(
            style.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
    let style = CssBorderStyle::None;
    let exact = CssSerializationLimits::new(1, 1, 4);
    assert_eq!(style.serialize_specified(exact).unwrap(), "none");
    for (limits, kind) in [
        (
            CssSerializationLimits::new(0, 1, 4),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 0, 4),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 1, 3),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&style.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}

#[test]
fn logical_pairs_keep_authored_omission_and_cumulative_budget() {
    let one = CssBorderStylePair::new(CssBorderStyle::Dotted, None);
    assert_eq!(one.start(), &CssBorderStyle::Dotted);
    assert_eq!(one.authored_end(), None);
    assert_eq!(one.end(), one.start());
    assert_eq!(
        one.serialize_specified(Default::default()).unwrap(),
        "dotted"
    );
    assert_ne!(
        one,
        CssBorderStylePair::new(CssBorderStyle::Dotted, Some(CssBorderStyle::Dotted))
    );

    let pair = CssBorderStylePair::new(CssBorderStyle::Dotted, Some(CssBorderStyle::Dashed));
    assert_eq!(pair.authored_end(), Some(&CssBorderStyle::Dashed));
    assert_eq!(pair.end(), &CssBorderStyle::Dashed);
    let exact = CssSerializationLimits::new(3, 3, 13);
    assert_eq!(pair.serialize_specified(exact).unwrap(), "dotted dashed");
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, 13),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 13),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 12),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&pair.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}

#[test]
fn four_side_styles_preserve_authored_arity_and_physical_or_logical_roles() {
    use CssBorderStyle::{Dashed, Dotted, None, Solid};
    assert!(CssBorderStyleShorthand::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssBorderStyleShorthand::try_new(CssBoxSideKind::Logical, vec![None; 5]).is_none());
    for (values, assigned, serialized) in [
        (vec![None], ["none", "none", "none", "none"], "none"),
        (
            vec![None, Solid],
            ["none", "solid", "none", "solid"],
            "none solid",
        ),
        (
            vec![None, Solid, Dashed],
            ["none", "solid", "dashed", "solid"],
            "none solid dashed",
        ),
        (
            vec![None, Solid, Dashed, Dotted],
            ["none", "solid", "dashed", "dotted"],
            "none solid dashed dotted",
        ),
    ] {
        for kind in [CssBoxSideKind::Physical, CssBoxSideKind::Logical] {
            let shorthand = CssBorderStyleShorthand::try_new(kind, values.clone()).unwrap();
            assert_eq!(shorthand.kind(), kind);
            assert_eq!(shorthand.authored_values(), values);
            let expected = match kind {
                CssBoxSideKind::Physical => serialized.to_string(),
                CssBoxSideKind::Logical => format!("logical {serialized}"),
            };
            assert_eq!(
                shorthand.serialize_specified(Default::default()).unwrap(),
                expected
            );
            assert_eq!(
                shorthand
                    .assigned_values()
                    .map(|value| value.serialize_specified(Default::default()).unwrap()),
                assigned
            );
        }
    }
    let one = CssBorderStyleShorthand::try_new(CssBoxSideKind::Physical, vec![None]).unwrap();
    let repeated =
        CssBorderStyleShorthand::try_new(CssBoxSideKind::Physical, vec![None; 4]).unwrap();
    let logical = CssBorderStyleShorthand::try_new(CssBoxSideKind::Logical, vec![None]).unwrap();
    assert_ne!(one, repeated);
    assert_ne!(one, logical);

    let logical = CssBorderStyleShorthand::try_new(
        CssBoxSideKind::Logical,
        vec![None, Solid, Dashed, CssBorderStyle::Double],
    )
    .unwrap();
    let exact = CssSerializationLimits::new(5, 5, 32);
    assert_eq!(
        logical.serialize_specified(exact).unwrap(),
        "logical none solid dashed double"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(4, 5, 32),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 4, 32),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 5, 31),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&logical.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}

#[test]
fn parsed_styles_retain_authored_roles_and_arity() {
    for (css, kind, authored) in [
        (
            "border-style:solid dashed",
            CssBoxSideKind::Physical,
            vec![CssBorderStyle::Solid, CssBorderStyle::Dashed],
        ),
        (
            "border-style:logical solid dashed",
            CssBoxSideKind::Logical,
            vec![CssBorderStyle::Solid, CssBorderStyle::Dashed],
        ),
    ] {
        let report = parse_style_attribute(css);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let CssKnownPropertyValueRef::BorderStyle(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("typed border-style")
        };
        assert_eq!(value.value().kind(), kind);
        assert_eq!(value.value().authored_values(), authored);
        assert_eq!(
            value.value().assigned_values(),
            [
                &CssBorderStyle::Solid,
                &CssBorderStyle::Dashed,
                &CssBorderStyle::Solid,
                &CssBorderStyle::Dashed
            ]
        );
    }

    let report = parse_style_attribute("border-block-style:dotted dashed");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderBlockStyle(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed border-block-style")
    };
    assert_eq!(
        value.value(),
        &CssBorderStylePair::new(CssBorderStyle::Dotted, Some(CssBorderStyle::Dashed))
    );
}
