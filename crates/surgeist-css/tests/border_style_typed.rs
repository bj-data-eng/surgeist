#![forbid(unsafe_code)]

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
        assert_eq!(style.serialize_specified().unwrap(), expected);
    }
    let style = CssBorderStyle::None;
    let exact = CssSpecifiedValueSerializationLimits::new(1, 1, 4);
    assert_eq!(
        style.serialize_specified_with_limits(exact).unwrap(),
        "none"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 4),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 4),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 3),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            style
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
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
    assert_eq!(one.serialize_specified().unwrap(), "dotted");
    assert_ne!(
        one,
        CssBorderStylePair::new(CssBorderStyle::Dotted, Some(CssBorderStyle::Dotted))
    );

    let pair = CssBorderStylePair::new(CssBorderStyle::Dotted, Some(CssBorderStyle::Dashed));
    assert_eq!(pair.authored_end(), Some(&CssBorderStyle::Dashed));
    assert_eq!(pair.end(), &CssBorderStyle::Dashed);
    let exact = CssSpecifiedValueSerializationLimits::new(3, 3, 13);
    assert_eq!(
        pair.serialize_specified_with_limits(exact).unwrap(),
        "dotted dashed"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 13),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 13),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 12),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            pair.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
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
            assert_eq!(shorthand.serialize_specified().unwrap(), expected);
            assert_eq!(
                shorthand
                    .assigned_values()
                    .map(|value| value.serialize_specified().unwrap()),
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
    let exact = CssSpecifiedValueSerializationLimits::new(5, 5, 32);
    assert_eq!(
        logical.serialize_specified_with_limits(exact).unwrap(),
        "logical none solid dashed double"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, 32),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, 32),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, 31),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            logical
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}

#[test]
fn parsed_style_current_retains_roles_and_physical_legacy_view() {
    for (css, kind, authored, legacy) in [
        (
            "border-style:solid dashed",
            CssBoxSideKind::Physical,
            vec![CssBorderStyle::Solid, CssBorderStyle::Dashed],
            true,
        ),
        (
            "border-style:logical solid dashed",
            CssBoxSideKind::Logical,
            vec![CssBorderStyle::Solid, CssBorderStyle::Dashed],
            false,
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
        assert_eq!(value.current().kind(), kind);
        assert_eq!(value.current().authored_values(), authored);
        assert_eq!(value.i01_subset().is_some(), legacy);
        if legacy {
            assert_eq!(
                value.i01_subset(),
                Some(&CssBorderStyles::new(
                    CssBorderStyle::Solid,
                    CssBorderStyle::Dashed,
                    CssBorderStyle::Solid,
                    CssBorderStyle::Dashed,
                ))
            );
        }
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
        value.current(),
        &CssBorderStylePair::new(CssBorderStyle::Dotted, Some(CssBorderStyle::Dashed))
    );
}
