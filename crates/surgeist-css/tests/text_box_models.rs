#![forbid(unsafe_code)]
//! Functional new-model evidence alongside implementation, with no prior
//! compile-failure RED claim. Inline3 WD20241218 §§5.2/6.1–6.3 independently
//! define finite roles, authored arity, metadata and expansion; corrected
//! omitted trim follows CSSWG editorial commit 97441f74 / PR12765.

use surgeist_css::*;

fn checked(property: CssKnownProperty, text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap()
}

fn typed_edge(declaration: &CssDeclaration) -> CssTextBoxEdge {
    let CssKnownPropertyValueRef::TextBoxEdge(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed edge wrapper")
    };
    *value.value()
}

fn block(css: &str) -> CssSpecifiedDeclarationBlock {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap()
}

#[test]
fn all_single_and_ordered_pair_constructors_round_trip_typed_roles_without_arity_loss() {
    for (metric, over, under, text) in [
        (
            CssTextEdgeMetric::Text,
            CssTextOverEdge::Text,
            CssTextUnderEdge::Text,
            "text",
        ),
        (
            CssTextEdgeMetric::Ideographic,
            CssTextOverEdge::Ideographic,
            CssTextUnderEdge::Ideographic,
            "ideographic",
        ),
        (
            CssTextEdgeMetric::IdeographicInk,
            CssTextOverEdge::IdeographicInk,
            CssTextUnderEdge::IdeographicInk,
            "ideographic-ink",
        ),
    ] {
        let single = CssTextEdge::Single(metric);
        assert_eq!(single.over(), over);
        assert_eq!(single.under(), under);
        assert_eq!(metric.serialize_specified().unwrap(), text);
        assert_eq!(single.serialize_specified().unwrap(), text);
        let pair = CssTextEdge::Pair { over, under };
        assert_ne!(single, pair);
        assert_eq!(
            pair.serialize_specified().unwrap(),
            format!("{text} {text}")
        );
        assert_eq!(
            typed_edge(&checked(CssKnownProperty::TextBoxEdge, text)),
            CssTextBoxEdge::Edge(single)
        );
        assert_eq!(
            typed_edge(&checked(
                CssKnownProperty::TextBoxEdge,
                &format!("{text} {text}")
            )),
            CssTextBoxEdge::Edge(pair)
        );
    }
    for (over, over_text) in [
        (CssTextOverEdge::Text, "text"),
        (CssTextOverEdge::Ideographic, "ideographic"),
        (CssTextOverEdge::IdeographicInk, "ideographic-ink"),
        (CssTextOverEdge::Cap, "cap"),
        (CssTextOverEdge::Ex, "ex"),
    ] {
        for (under, under_text) in [
            (CssTextUnderEdge::Text, "text"),
            (CssTextUnderEdge::Ideographic, "ideographic"),
            (CssTextUnderEdge::IdeographicInk, "ideographic-ink"),
            (CssTextUnderEdge::Alphabetic, "alphabetic"),
        ] {
            let edge = CssTextEdge::Pair { over, under };
            let expected = format!("{over_text} {under_text}");
            assert_eq!(edge.over(), over);
            assert_eq!(edge.under(), under);
            assert_eq!(over.serialize_specified().unwrap(), over_text);
            assert_eq!(under.serialize_specified().unwrap(), under_text);
            assert_eq!(edge.serialize_specified().unwrap(), expected);
            assert_eq!(
                typed_edge(&checked(CssKnownProperty::TextBoxEdge, &expected)),
                CssTextBoxEdge::Edge(edge)
            );
        }
    }
    assert_eq!(
        typed_edge(&checked(CssKnownProperty::TextBoxEdge, "auto")),
        CssTextBoxEdge::Auto
    );
    assert_eq!(CssTextBoxEdge::Auto.serialize_specified().unwrap(), "auto");
}

#[test]
fn empty_shorthand_construction_rejects_and_each_authored_slot_remains_observable() {
    assert_eq!(CssTextBoxValues::try_new(None, None), None);
    let edge = CssTextBoxEdge::Edge(CssTextEdge::Pair {
        over: CssTextOverEdge::Cap,
        under: CssTextUnderEdge::Alphabetic,
    });
    for (trim, edge, expected) in [
        (Some(CssTextBoxTrim::TrimEnd), None, "trim-end"),
        (None, Some(edge), "cap alphabetic"),
        (
            Some(CssTextBoxTrim::None),
            Some(CssTextBoxEdge::Auto),
            "none auto",
        ),
    ] {
        let values = CssTextBoxValues::try_new(trim, edge).unwrap();
        assert_eq!(values.trim(), trim);
        assert_eq!(values.edge(), edge);
        assert_eq!(values.serialize_specified().unwrap(), expected);
        let value = CssTextBox::Components(values);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let declaration = checked(CssKnownProperty::TextBox, expected);
        let CssKnownPropertyValueRef::TextBox(actual) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("typed shorthand")
        };
        assert_eq!(actual.value(), &value);
    }
    assert_eq!(CssTextBox::Normal.serialize_specified().unwrap(), "normal");
    assert_ne!(
        CssTextBox::Normal,
        CssTextBox::Components(
            CssTextBoxValues::try_new(Some(CssTextBoxTrim::None), Some(CssTextBoxEdge::Auto))
                .unwrap()
        )
    );
}

#[test]
fn primitive_providers_charge_exact_nodes_and_bytes_and_retry_without_mutation() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for (value, expected, nodes) in [
        (CssTextBox::Normal, "normal", 1),
        (
            CssTextBox::Components(
                CssTextBoxValues::try_new(Some(CssTextBoxTrim::TrimBoth), None).unwrap(),
            ),
            "trim-both",
            1,
        ),
        (
            CssTextBox::Components(
                CssTextBoxValues::try_new(
                    None,
                    Some(CssTextBoxEdge::Edge(CssTextEdge::Pair {
                        over: CssTextOverEdge::Ex,
                        under: CssTextUnderEdge::Text,
                    })),
                )
                .unwrap(),
            ),
            "ex text",
            2,
        ),
        (
            CssTextBox::Components(
                CssTextBoxValues::try_new(
                    Some(CssTextBoxTrim::TrimStart),
                    Some(CssTextBoxEdge::Edge(CssTextEdge::Pair {
                        over: CssTextOverEdge::Cap,
                        under: CssTextUnderEdge::Alphabetic,
                    })),
                )
                .unwrap(),
            ),
            "trim-start cap alphabetic",
            3,
        ),
    ] {
        let before = value;
        let exact = Limits::new(nodes, nodes, expected.len());
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    value
                        .serialize_specified_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(value, before);
            }
        }
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
    }
    for value in [
        CssTextBoxTrim::None,
        CssTextBoxTrim::TrimStart,
        CssTextBoxTrim::TrimEnd,
        CssTextBoxTrim::TrimBoth,
    ] {
        let text = value.serialize_specified().unwrap();
        let declaration = checked(CssKnownProperty::TextBoxTrim, &text);
        let CssKnownPropertyValueRef::TextBoxTrim(actual) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("typed trim")
        };
        assert_eq!(actual.value(), &value);
    }
}

#[test]
fn cssom_inverse_reconstructs_exact_longhand_roles_and_normal_and_preserves_pair_arity() {
    for (css, expected) in [
        ("text-box-trim:none;text-box-edge:auto", "normal"),
        (
            "text-box-trim:trim-both;text-box-edge:auto",
            "trim-both auto",
        ),
        (
            "text-box-trim:trim-start;text-box-edge:cap alphabetic",
            "trim-start cap alphabetic",
        ),
        (
            "text-box-trim:trim-end;text-box-edge:ideographic",
            "trim-end ideographic",
        ),
        (
            "text-box-trim:trim-end;text-box-edge:ideographic ideographic",
            "trim-end ideographic ideographic",
        ),
        ("text-box:ex text", "trim-both ex text"),
        ("text-box:trim-start", "trim-start auto"),
    ] {
        let actual = block(css);
        assert_eq!(
            actual
                .property_value(CssPropertyNameRef::Known(CssKnownProperty::TextBox))
                .unwrap()
                .as_deref(),
            Some(expected)
        );
        let reparsed = checked(CssKnownProperty::TextBox, expected);
        let CssExpansion::Contributions(CssContributions::Longhands(reexpanded)) =
            expand_declaration(&reparsed).unwrap()
        else {
            panic!("ordinary inverse projection")
        };
        assert_eq!(reexpanded.items().len(), 2);
        for name in [CssKnownProperty::TextBoxTrim, CssKnownProperty::TextBoxEdge] {
            let expected_longhand = actual
                .property_value(CssPropertyNameRef::Known(name))
                .unwrap()
                .unwrap();
            let expected_declaration = checked(name, &expected_longhand);
            let CssExpansion::Contributions(CssContributions::Longhands(expected_values)) =
                expand_declaration(&expected_declaration).unwrap()
            else {
                panic!("one terminal")
            };
            assert_eq!(
                reexpanded
                    .items()
                    .iter()
                    .find(|item| item.property() == name)
                    .unwrap()
                    .ordinary_value(),
                expected_values.items()[0].ordinary_value()
            );
        }
    }
    for css in [
        "text-box-trim:trim-start",
        "text-box-trim:trim-start!important;text-box-edge:auto",
        "text-box-trim:initial;text-box-edge:auto",
    ] {
        assert!(
            block(css)
                .property_value(CssPropertyNameRef::Known(CssKnownProperty::TextBox))
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn source_catalog_reports_the_exact_selected_pre_cr_feature_edition() {
    for property in [
        CssKnownProperty::TextBox,
        CssKnownProperty::TextBoxEdge,
        CssKnownProperty::TextBoxTrim,
    ] {
        let support = property_support_metadata(property.canonical_name()).unwrap();
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        let source = support.feature().source();
        assert_eq!(source.id().as_str(), "F-INLINE3-20241218");
        assert_eq!(
            source.tier(),
            CssSpecificationTier::Snapshot2026PreCrException
        );
        assert_eq!(
            source.url(),
            Some("https://www.w3.org/TR/2024/WD-css-inline-3-20241218/")
        );
        assert_eq!(specification_source("F-INLINE3-20241218"), Some(&source));
    }
}
