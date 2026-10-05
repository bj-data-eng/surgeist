#![forbid(unsafe_code)]

//! Functional checked-value and canonical-writer contracts from Will Change 1 §2,
//! Scroll Anchoring 1 §3, and Fragmentation 3 §5.4, using the pinned editions.

use surgeist_css::*;

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{value}"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn will_change(source: &CssDeclaration) -> &CssWillChange {
    let CssKnownPropertyValueRef::WillChange(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed will-change payload")
    };
    value.value()
}

#[test]
fn checked_names_enforce_only_the_will_change_exclusions() {
    for excluded in [
        "inherit",
        "INITIAL",
        "unset",
        "ReVeRt",
        "revert-layer",
        "default",
        "will-change",
        "none",
        "all",
        "auto",
        "scroll-position",
        "contents",
        "",
        "bad\0name",
    ] {
        assert!(
            CssWillChangePropertyName::try_new(excluded).is_none(),
            "{excluded:?}"
        );
    }
    for allowed in [
        "span",
        "SPAN",
        "FutureProperty",
        "--custom",
        "opacity",
        "x y",
        "1name",
    ] {
        let name = CssWillChangePropertyName::try_new(allowed).unwrap();
        assert_eq!(name.as_str(), allowed);
        let value = CssWillChange::Features(
            CssWillChangeFeatures::try_new(vec![CssWillChangeFeature::Property(name)]).unwrap(),
        );
        let css = value.serialize_specified().unwrap();
        assert_eq!(will_change(&declaration("will-change", &css)), &value);
    }
    assert!(CssWillChangeFeatures::try_new(vec![]).is_none());
}

#[test]
fn keyword_alternatives_and_repeated_decoded_property_names_keep_order_and_case() {
    let source = declaration(
        "will-change",
        r"ScRoLl-PoSiTiOn, CONTENTS, f\75 ture, FutureProperty, --custom, span, future",
    );
    let expected = CssWillChange::Features(
        CssWillChangeFeatures::try_new(vec![
            CssWillChangeFeature::ScrollPosition,
            CssWillChangeFeature::Contents,
            CssWillChangeFeature::Property(CssWillChangePropertyName::try_new("future").unwrap()),
            CssWillChangeFeature::Property(
                CssWillChangePropertyName::try_new("FutureProperty").unwrap(),
            ),
            CssWillChangeFeature::Property(CssWillChangePropertyName::try_new("--custom").unwrap()),
            CssWillChangeFeature::Property(CssWillChangePropertyName::try_new("span").unwrap()),
            CssWillChangeFeature::Property(CssWillChangePropertyName::try_new("future").unwrap()),
        ])
        .unwrap(),
    );
    assert_eq!(will_change(&source), &expected);
    let canonical = "scroll-position, contents, future, FutureProperty, --custom, span, future";
    assert_eq!(expected.serialize_specified().unwrap(), canonical);
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::WillChange),
        source.value_components().clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(will_change(&checked), &expected);
    assert_eq!(
        will_change(&declaration("will-change", canonical)),
        &expected
    );
    assert_eq!(
        will_change(&declaration("will-change", "AUTO")),
        &CssWillChange::Auto
    );
}

#[test]
fn mixed_keyword_exclusions_and_escaped_exclusions_reject_with_component_origins() {
    for text in [
        "opacity, DEFAULT",
        r"opacity, n\6f ne",
        "span, inherit",
        "ALL",
        "will-change",
        "contents, auto",
    ] {
        let components = parse_component_values(text).unwrap();
        let original = components.clone();
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::WillChange),
            components,
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(
            matches!(error.kind(), CssPropertyValueErrorKind::Grammar(_)),
            "{text}"
        );
        let origin = match error.origin() {
            CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
            other => panic!("{other:?}"),
        };
        assert!(
            original.items().iter().any(|item| item.origin() == origin),
            "{text}"
        );
    }
}

#[test]
fn identifier_escaping_and_repeated_entries_share_exact_resource_budgets() {
    let value = CssWillChange::Features(
        CssWillChangeFeatures::try_new(vec![
            CssWillChangeFeature::Property(CssWillChangePropertyName::try_new("x y").unwrap()),
            CssWillChangeFeature::Contents,
            CssWillChangeFeature::Property(CssWillChangePropertyName::try_new("x y").unwrap()),
        ])
        .unwrap(),
    );
    let expected = r"x\ y, contents, x\ y";
    let snapshot = value.clone();
    let exact = CssSpecifiedValueSerializationLimits::new(4, 4, expected.len());
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 4, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 4, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, snapshot);
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
    }
    assert_eq!(will_change(&declaration("will-change", expected)), &value);
}

#[test]
fn auto_writer_has_one_node_and_four_bytes() {
    let exact = CssSpecifiedValueSerializationLimits::new(1, 1, 4);
    assert_eq!(
        CssWillChange::Auto
            .serialize_specified_with_limits(exact)
            .unwrap(),
        "auto"
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
            CssWillChange::Auto
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}

#[test]
fn overflow_anchor_and_decoration_keywords_expose_typed_payloads_and_canonical_output() {
    for (text, expected) in [
        ("AUTO", CssOverflowAnchor::Auto),
        ("NoNe", CssOverflowAnchor::None),
    ] {
        let source = declaration("overflow-anchor", text);
        let CssKnownPropertyValueRef::OverflowAnchor(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed anchor")
        };
        assert_eq!(*value.value(), expected);
        let canonical = expected.serialize_specified().unwrap();
        assert_eq!(canonical, text.to_ascii_lowercase());
        let reparsed = declaration("overflow-anchor", &canonical);
        let CssKnownPropertyValueRef::OverflowAnchor(value) =
            reparsed.known().unwrap().property_value().unwrap()
        else {
            panic!("typed anchor")
        };
        assert_eq!(*value.value(), expected);
    }
    for (text, expected) in [
        ("SLICE", CssBoxDecorationBreak::Slice),
        ("ClOnE", CssBoxDecorationBreak::Clone),
    ] {
        let source = declaration("box-decoration-break", text);
        let CssKnownPropertyValueRef::BoxDecorationBreak(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed decoration")
        };
        assert_eq!(*value.value(), expected);
        assert_eq!(
            expected.serialize_specified().unwrap(),
            text.to_ascii_lowercase()
        );
    }
}

#[test]
fn keyword_writers_enforce_exact_limits_atomically() {
    for (name, css) in [
        ("overflow-anchor", "none"),
        ("box-decoration-break", "clone"),
    ] {
        let source = declaration(name, css);
        let emit = |limits| match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::OverflowAnchor(value) => {
                value.value().serialize_specified_with_limits(limits)
            }
            CssKnownPropertyValueRef::BoxDecorationBreak(value) => {
                value.value().serialize_specified_with_limits(limits)
            }
            _ => panic!("selected keyword"),
        };
        let exact = CssSpecifiedValueSerializationLimits::new(1, 1, css.len());
        assert_eq!(emit(exact).unwrap(), css);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, css.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, css.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, css.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(emit(limits).unwrap_err().kind(), kind);
            assert_eq!(emit(exact).unwrap(), css);
            assert_eq!(source.value_components().serialize().unwrap().as_css(), css);
        }
    }
}

#[test]
fn authored_support_metadata_uses_the_exact_pinned_specification_sources() {
    for (name, id, url) in [
        (
            "will-change",
            "I-WILLCHANGE1",
            "https://www.w3.org/TR/2022/CRD-css-will-change-1-20220505/",
        ),
        (
            "overflow-anchor",
            "I-SCROLLANCHORING1",
            "https://www.w3.org/TR/2020/WD-css-scroll-anchoring-1-20201111/",
        ),
        (
            "box-decoration-break",
            "S-BREAK3",
            "https://www.w3.org/TR/2018/CR-css-break-3-20181204/",
        ),
    ] {
        let metadata = property_support_metadata(name).unwrap();
        assert_eq!(metadata.feature().status(), CssSupportStatus::Complete);
        assert_eq!(metadata.feature().source().id().as_str(), id);
        assert_eq!(metadata.feature().source().url(), Some(url));
        assert_eq!(specification_source(id).unwrap().url(), Some(url));
    }
}
