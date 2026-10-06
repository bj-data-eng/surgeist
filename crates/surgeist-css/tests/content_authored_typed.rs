#![forbid(unsafe_code)]
//! Functional new API expectations from Content 3 WD 2025-12-04's selected
//! property tables and the bounded named-string disposition recorded in #680.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn name(value: &str) -> CssContentName {
    CssContentName::try_new(CssIdent::try_new(value).unwrap()).unwrap()
}
fn string(value: &str) -> CssContentString {
    CssContentString::try_new(value).unwrap()
}
fn declaration(property: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{property}:{value}");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}
fn calculation(value: &str) -> CssPositiveIntegerValue {
    CssPositiveIntegerValue::Calculation(
        CssIntegerCalculation::try_from_components(parse_component_values(value).unwrap()).unwrap(),
    )
}

#[test]
fn string_set_checks_both_nonempty_boundaries_and_preserves_duplicate_names() {
    assert!(CssStringSet::try_entries(vec![]).is_none());
    assert!(CssStringSetEntry::try_new(name("Chapter"), vec![]).is_none());
    assert!(CssContentString::try_new("a\0b").is_none());
    let first = CssStringSetEntry::try_new(name("none"), vec![string(""), string("A")]).unwrap();
    let second = CssStringSetEntry::try_new(name("none"), vec![string("B")]).unwrap();
    let value = CssStringSet::try_entries(vec![first, second]).unwrap();
    let entries = value.entries().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name().as_str(), "none");
    assert_eq!(entries[0].strings()[0].as_str(), "");
    assert_eq!(entries[0].strings()[1].as_str(), "A");
    assert_eq!(
        value.serialize_specified().unwrap(),
        "none \"\" \"A\", none \"B\""
    );
    assert!(CssStringSet::none().entries().is_none());
    assert_eq!(CssStringSet::none().serialize_specified().unwrap(), "none");
}

#[test]
fn string_set_uses_generic_names_and_escaped_separate_string_leaves() {
    for forbidden in ["default", "INHERIT", "revert-layer"] {
        assert!(CssContentName::try_new(CssIdent::try_new(forbidden).unwrap()).is_none());
    }
    let source = declaration("string-set", r#"Chapter\ name "a\22 b" """#);
    let CssKnownPropertyValueRef::StringSet(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("string-set")
    };
    let entries = wrapper.value().entries().unwrap();
    assert_eq!(entries[0].name().as_str(), "Chapter name");
    assert_eq!(
        entries[0]
            .strings()
            .iter()
            .map(CssContentString::as_str)
            .collect::<Vec<_>>(),
        ["a\"b", ""]
    );
    assert_eq!(
        wrapper.value().serialize_specified().unwrap(),
        r#"Chapter\ name "a\"b" """#
    );
    assert!(source.position().is_some());
    assert!(source.parsed_value().is_some());
}

#[test]
fn string_set_direct_writer_charges_all_assignments_and_recovers_after_failure() {
    let value = CssStringSet::try_entries(vec![
        CssStringSetEntry::try_new(name("Chapter"), vec![string("A"), string("B")]).unwrap(),
        CssStringSetEntry::try_new(name("Author"), vec![string("C")]).unwrap(),
    ])
    .unwrap();
    let expected = "Chapter \"A\" \"B\", Author \"C\"";
    let before = value.clone();
    // One outer list + two entries + two names + three strings = eight nodes.
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(8, 8, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(7, 8, expected.len()), K::InputNodeLimit),
        (L::new(8, 7, expected.len()), K::ProjectionNodeLimit),
        (L::new(8, 8, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn bookmark_level_constructor_rejects_nonpositive_bare_calculation_tokens() {
    for bare in ["0", "+0", "-0", "-1"] {
        assert!(
            CssBookmarkLevel::try_new(calculation(bare)).is_none(),
            "{bare}"
        );
    }
    let positive = CssBookmarkLevel::try_new(calculation("+001")).unwrap();
    let Some(CssPositiveIntegerValue::Literal(literal)) = positive.level() else {
        panic!("bare positive calculation normalizes to literal")
    };
    assert_eq!(literal.integer().numeric().representation(), "+001");
    assert_eq!(positive.serialize_specified().unwrap(), "1");
    for deferred in ["calc(-1)", "calc(0)", "min(-1, 0)"] {
        let level = CssBookmarkLevel::try_new(calculation(deferred)).unwrap();
        assert!(matches!(
            level.level(),
            Some(CssPositiveIntegerValue::Calculation(_))
        ));
    }
    assert!(CssBookmarkLevel::none().level().is_none());
    assert_eq!(
        CssBookmarkLevel::none().serialize_specified().unwrap(),
        "none"
    );
}

#[test]
fn bookmark_level_retains_exact_large_integer_and_parsed_numeric_origin() {
    let digits = "123456789012345678901234567890123456789012345678901234567890123456789";
    let source = declaration("bookmark-level", digits);
    let CssKnownPropertyValueRef::BookmarkLevel(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("bookmark-level")
    };
    let Some(CssPositiveIntegerValue::Literal(literal)) = wrapper.value().level() else {
        panic!("exact positive literal")
    };
    assert_eq!(literal.integer().numeric().representation(), digits);
    assert!(matches!(
        literal.integer().component().origin(),
        CssValueOrigin::Parsed(_)
    ));
    let copied =
        CssBookmarkLevel::try_new(CssPositiveIntegerValue::Literal(literal.clone())).unwrap();
    let Some(CssPositiveIntegerValue::Literal(retained)) = copied.level() else {
        panic!("literal")
    };
    assert_eq!(
        retained.integer().component().origin(),
        literal.integer().component().origin()
    );
    assert_eq!(
        copied
            .serialize_specified_with_limits(L::new(1, 1, digits.len()))
            .unwrap(),
        digits
    );
    assert_eq!(
        copied
            .serialize_specified_with_limits(L::new(0, 1, digits.len()))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
}

#[test]
fn full_content_list_checks_empty_and_keeps_sole_label_image_as_list() {
    assert!(CssContentList::try_new(vec![]).is_none());
    let source = declaration("bookmark-label", "url(\"a.png\")");
    let CssKnownPropertyValueRef::BookmarkLabel(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("bookmark-label")
    };
    let [CssContentValueItem::Image(image)] = wrapper.value().items() else {
        panic!("sole list image")
    };
    let list = CssContentList::try_new(vec![CssContentValueItem::Image(image.clone())]).unwrap();
    assert!(matches!(list.items(), [CssContentValueItem::Image(_)]));
    assert_eq!(list.serialize_specified().unwrap(), "url(\"a.png\")");
    let generated = CssGeneratedContent::try_new(list.items().to_vec(), None).unwrap();
    assert!(matches!(
        generated.body(),
        CssGeneratedContentBodyRef::Replacement(_)
    ));
}

#[test]
fn mixed_parsed_image_and_programmatic_content_children_share_list_and_writer() {
    let parsed = declaration("bookmark-label", "url(\"a.png\") counter(Chapter, decimal)");
    let CssKnownPropertyValueRef::BookmarkLabel(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("bookmark-label")
    };
    let mut items = wrapper.value().items().to_vec();
    let retained = items.clone();
    items.insert(0, CssContentValueItem::String(string("programmatic")));
    items.push(CssContentValueItem::Content(Some(
        CssContentReferenceMode::Before,
    )));
    let list = CssContentList::try_new(items).unwrap();
    assert_eq!(&list.items()[1..3], retained.as_slice());
    let expected = "\"programmatic\" url(\"a.png\") counter(Chapter) content(before)";
    assert_eq!(list.serialize_specified().unwrap(), expected);
    let reentered = declaration("bookmark-label", expected);
    assert_eq!(
        reentered.to_specified_css().unwrap(),
        format!("bookmark-label: {expected};")
    );
}

#[test]
fn explicit_content_text_omission_keeps_work_tariff_and_argument_view() {
    let explicit = CssContentList::try_new(vec![CssContentValueItem::Content(Some(
        CssContentReferenceMode::Text,
    ))])
    .unwrap();
    let implicit = CssContentList::try_new(vec![CssContentValueItem::Content(None)]).unwrap();
    assert_eq!(explicit.serialize_specified().unwrap(), "content()");
    assert_eq!(implicit.serialize_specified().unwrap(), "content()");
    assert_eq!(
        explicit
            .serialize_specified_with_limits(L::new(3, 3, 9))
            .unwrap(),
        "content()"
    );
    assert_eq!(
        explicit
            .serialize_specified_with_limits(L::new(2, 3, 9))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
    assert_eq!(
        implicit
            .serialize_specified_with_limits(L::new(2, 2, 9))
            .unwrap(),
        "content()"
    );
    assert!(matches!(
        explicit.items(),
        [CssContentValueItem::Content(Some(
            CssContentReferenceMode::Text
        ))]
    ));
}

#[test]
fn explicit_counter_and_named_string_defaults_cost_nodes_even_when_suppressed() {
    let counter = CssContentCounter::new(
        CssContentCounterName::try_new(CssIdent::try_new("Chapter").unwrap()).unwrap(),
        Some(CssCounterStyleValue::try_named(CssIdent::try_new("decimal").unwrap()).unwrap()),
    );
    let named = CssNamedString::new(name("Title"), Some(CssNamedStringMode::First));
    let list = CssContentList::try_new(vec![
        CssContentValueItem::Counter(counter),
        CssContentValueItem::NamedString(named),
    ])
    .unwrap();
    let expected = "counter(Chapter) string(Title)";
    // List one; each function, name and explicit default one: total seven.
    assert_eq!(
        list.serialize_specified_with_limits(L::new(7, 7, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        list.serialize_specified_with_limits(L::new(7, 6, expected.len()))
            .unwrap_err()
            .kind(),
        K::ProjectionNodeLimit
    );
}

#[test]
fn four_new_intrinsic_initial_views_retain_exact_payloads() {
    for property in [
        CssKnownProperty::StringSet,
        CssKnownProperty::BookmarkLevel,
        CssKnownProperty::BookmarkLabel,
        CssKnownProperty::BookmarkState,
    ] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("longhand")
        };
        assert!(!metadata.inherited_by_default());
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("fixed initial")
        };
        match value.view() {
            CssLonghandValueRef::StringSet(value) => assert!(value.entries().is_none()),
            CssLonghandValueRef::BookmarkLevel(value) => assert!(value.level().is_none()),
            CssLonghandValueRef::BookmarkLabel(value) => {
                assert!(matches!(
                    value.items(),
                    [CssContentValueItem::Content(Some(
                        CssContentReferenceMode::Text
                    ))]
                ));
                assert_eq!(value.serialize_specified().unwrap(), "content()");
            }
            CssLonghandValueRef::BookmarkState(value) => assert_eq!(*value, CssBookmarkState::Open),
            _ => panic!("new initial payload"),
        }
        assert_eq!(value.property().known_property(), property);
    }
}

#[test]
fn bookmark_state_direct_writer_has_only_keyword_tariff() {
    for (state, expected) in [
        (CssBookmarkState::Open, "open"),
        (CssBookmarkState::Closed, "closed"),
    ] {
        assert_eq!(
            state
                .serialize_specified_with_limits(L::new(1, 1, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            state
                .serialize_specified_with_limits(L::new(1, 1, expected.len() - 1))
                .unwrap_err()
                .kind(),
            K::ByteLimit
        );
        assert_eq!(
            state
                .serialize_specified_with_limits(L::new(0, 1, expected.len()))
                .unwrap_err()
                .kind(),
            K::InputNodeLimit
        );
        assert_eq!(state.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn checked_reconstruction_keeps_original_components_and_strict_closure() {
    for (property, input) in [
        (CssKnownProperty::StringSet, "Chapter \"A\""),
        (CssKnownProperty::BookmarkLevel, "calc(-1)"),
        (CssKnownProperty::BookmarkLabel, "counter(Chapter, decimal)"),
        (CssKnownProperty::BookmarkState, "closed"),
    ] {
        let components = parse_component_values(input).unwrap();
        let before = components.clone();
        let value = parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(value.value_components(), &before);
        assert!(value.parsed_value().is_none());
        assert!(value.position().is_none());
        assert_eq!(components, before);
        let incomplete = parse_component_values(&format!("{input}/*unfinished")).unwrap();
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property),
                incomplete,
                CssImportance::Normal
            )
            .is_err()
        );
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property),
                before,
                CssImportance::Normal
            )
            .is_ok()
        );
    }
}

#[test]
fn string_set_checked_front_preserves_mixed_component_origins_without_inventing_leaf_spans() {
    let parsed = parse_component_values("Chapter ").unwrap();
    let mut components = parsed.items().to_vec();
    components.push(CssComponentValue::try_string("programmatic").unwrap());
    let mixed = CssComponentValues::try_new(components).unwrap();
    assert!(matches!(
        mixed.items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        mixed.items().last().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::StringSet),
        mixed.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &mixed);
    assert!(checked.position().is_none());
    assert!(checked.parsed_value().is_none());
    let CssKnownPropertyValueRef::StringSet(wrapper) =
        checked.known().unwrap().property_value().unwrap()
    else {
        panic!("checked string-set")
    };
    let [entry] = wrapper.value().entries().unwrap() else {
        panic!("one entry")
    };
    assert_eq!(entry.name().as_str(), "Chapter");
    assert_eq!(entry.strings()[0].as_str(), "programmatic");
    assert_eq!(
        checked.to_specified_css().unwrap(),
        "string-set: Chapter \"programmatic\";"
    );
}

// The Image owner qualifies its existing consumers separately. This new Content
// consumer must preserve the same complete graph and original resource origin.
fn content_image_source(light_dark_functions: usize) -> String {
    // An unquoted URL is one raw token but one retained image function.
    let mut value = "url(image.svg)".to_owned();
    for _ in 0..light_dark_functions {
        value = format!("light-dark({value}, none)");
    }
    format!("/*😀*/ {value}")
}

fn checked_content_image(
    property: CssKnownProperty,
    values: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), values, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            values,
            CssImportance::Important,
        )
    }
}

#[test]
fn bookmark_label_inherits_checked_image_resource_origin_and_pending_retry() {
    // 255 LightDark functions plus a URL fit exactly; 256 plus URL exceed 256.
    let valid = parse_component_values(&content_image_source(255)).unwrap();
    let invalid = parse_component_values(&content_image_source(256)).unwrap();
    assert_eq!(valid.nesting_depth(), 255);
    assert_eq!(invalid.nesting_depth(), 256);
    let valid_before = valid.clone();
    let invalid_before = invalid.clone();
    let opener = invalid
        .items()
        .iter()
        .find(|item| matches!(item.view(), CssComponentValueRef::Function(_)))
        .unwrap()
        .origin()
        .clone();
    let CssValueOrigin::Parsed(parsed) = &opener else {
        panic!("authored image opener")
    };
    assert_eq!(parsed.span().start().byte_offset().value(), "/*😀*/ ".len());

    for name in ["bookmark-label"] {
        let property = CssKnownProperty::from_name(name).unwrap();
        let pending = checked_content_image(
            property,
            parse_component_values("var(--image)").unwrap(),
            false,
        )
        .unwrap();
        let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
            panic!("pending image consumer")
        };
        for _ in 0..2 {
            for grammar in [false, true] {
                assert!(checked_content_image(property, valid.clone(), grammar).is_ok());
                let error = checked_content_image(property, invalid.clone(), grammar).unwrap_err();
                assert_eq!(
                    error.kind(),
                    &CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit),
                    "{name} image resource failure survives grammar alternatives"
                );
                assert_eq!(error.origin(), &CssSerializedOrigin::Token(opener.clone()));
            }
            let error = handle.reenter(invalid.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("typed image replacement failure")
            };
            assert_eq!(
                error.kind(),
                &CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit)
            );
            assert_eq!(error.origin(), &CssSerializedOrigin::Token(opener.clone()));
            assert!(handle.reenter(valid.clone()).is_ok());
            assert!(handle.source().same_occurrence(&pending));
            assert_eq!(handle.source().importance(), CssImportance::Important);
            assert_eq!(valid, valid_before);
            assert_eq!(invalid, invalid_before);
        }
    }
}
