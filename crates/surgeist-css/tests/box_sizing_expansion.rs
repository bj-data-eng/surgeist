#![forbid(unsafe_code)]
//! CSS Sizing 3 box-sizing remains a non-inherited terminal longhand.
use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn box_sizing_has_a_non_inherited_content_box_initial() {
    let metadata = CssKnownProperty::BoxSizing.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("box-sizing must be a terminal longhand")
    };
    assert_eq!(
        longhand.property().known_property(),
        CssKnownProperty::BoxSizing
    );
    assert!(!longhand.inherited_by_default());
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("box-sizing has an intrinsic initial")
    };
    let CssLonghandValueRef::BoxSizing(value) = initial.view() else {
        panic!("box-sizing initial value")
    };
    assert_eq!(*value, CssBoxSizing::ContentBox);
}

#[test]
fn box_sizing_keywords_contribute_one_typed_terminal_with_source_identity() {
    for (keyword, expected) in [
        ("content-box", CssBoxSizing::ContentBox),
        ("BORDER-BOX", CssBoxSizing::BorderBox),
    ] {
        let source = declaration(&format!("box-sizing:{keyword}!important"));
        assert!(validate_style_attribute(&format!("box-sizing:{keyword}")).is_ok());
        let CssKnownPropertyValueRef::BoxSizing(authored) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("box-sizing ordinary value")
        };
        assert_eq!(authored.i01_subset(), Some(&expected));
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one longhand contribution")
        };
        let [item] = items.items() else {
            panic!("exactly one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::BoxSizing);
        let CssLonghandValueRef::BoxSizing(value) = item.ordinary_value().unwrap().view() else {
            panic!("typed box-sizing contribution")
        };
        assert_eq!(*value, expected);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn box_sizing_globals_remain_symbolic_single_terminal_values() {
    for (text, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("box-sizing:{text}!important"));
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("global longhand")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::BoxSizing);
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn box_sizing_pending_reentry_checks_the_same_grammar_atomically() {
    let source = declaration("box-sizing:var(--edge)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending substitution")
    };
    for text in [
        "padding-box",
        "border-box extra",
        "border-box!important",
        "border-box; color:red",
    ] {
        let error = handle
            .reenter(parse_component_values(text).unwrap())
            .unwrap_err();
        assert!(
            matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
            "{text}: {error:?}"
        );
    }
    assert_eq!(
        handle
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    let replacement = parse_component_values("BORDER-BOX").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("reentry longhand")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::BoxSizing);
        let CssLonghandValueRef::BoxSizing(value) = item.ordinary_value().unwrap().view() else {
            panic!("typed replacement")
        };
        assert_eq!(*value, CssBoxSizing::BorderBox);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    let CssContributions::Longhands(items) = handle
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("global reentry")
    };
    assert_eq!(items.items().len(), 1);
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn box_sizing_normalization_preserves_valid_occurrences_after_recovery() {
    let source =
        ".a{box-sizing:content-box;box-sizing:padding-box;box-sizing:border-box!important}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    let sheet = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| {
            if let CssNormalizedItem::Declaration(value) = item {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(declarations.len(), 2);
    for (index, expected) in [CssImportance::Normal, CssImportance::Important]
        .into_iter()
        .enumerate()
    {
        let item = declarations[index];
        assert_eq!(item.order(), index);
        assert_eq!(item.source().importance(), expected);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("normalized terminal")
        };
        let [value] = values.items() else {
            panic!("one terminal")
        };
        assert_eq!(value.property(), CssKnownProperty::BoxSizing);
        let CssLonghandValueRef::BoxSizing(actual) = value.ordinary_value().unwrap().view() else {
            panic!("typed normalized value")
        };
        assert_eq!(
            *actual,
            if index == 0 {
                CssBoxSizing::ContentBox
            } else {
                CssBoxSizing::BorderBox
            }
        );
        assert!(value.source().same_occurrence(item.source()));
    }
}

#[test]
fn box_sizing_checked_construction_preserves_programmatic_origins_and_keyword_meaning() {
    for (text, expected) in [
        ("content-box", CssBoxSizing::ContentBox),
        ("BORDER-BOX", CssBoxSizing::BorderBox),
    ] {
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
        let source = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::BoxSizing),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert!(source.parsed_value().is_none());
        assert_eq!(source.value_components(), &components);
        assert!(matches!(
            components.items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        let CssKnownPropertyValueRef::BoxSizing(authored) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("constructed box-sizing")
        };
        assert_eq!(authored.current(), &expected);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("constructed longhand")
        };
        let CssLonghandValueRef::BoxSizing(actual) =
            items.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("constructed typed contribution")
        };
        assert_eq!(*actual, expected);
        assert!(items.items()[0].source().same_occurrence(&source));
    }
}

#[test]
fn box_sizing_specified_serialization_is_canonical_and_resource_bounded() {
    use CssSpecifiedValueSerializationErrorKind as K;
    for (source_text, expected, canonical) in [
        ("content-box", CssBoxSizing::ContentBox, "content-box"),
        ("BORDER-BOX", CssBoxSizing::BorderBox, "border-box"),
        (r"\62 order-box", CssBoxSizing::BorderBox, "border-box"),
    ] {
        let source = declaration(&format!("box-sizing:{source_text}"));
        let CssKnownPropertyValueRef::BoxSizing(authored) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("authored keyword")
        };
        assert_eq!(authored.current(), &expected);
        assert_eq!(authored.as_css(), source_text);
        assert_eq!(authored.current().serialize_specified().unwrap(), canonical);
        assert_eq!(expected.serialize_specified().unwrap(), canonical);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, canonical.len()),
                K::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, canonical.len()),
                K::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, canonical.len() - 1),
                K::ByteLimit,
            ),
        ] {
            assert_eq!(
                expected
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(
            expected
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    canonical.len()
                ))
                .unwrap(),
            canonical
        );
    }
}

#[test]
fn box_sizing_invalid_keyword_and_extra_tokens_are_recovered_and_fail_clean_validation() {
    for text in ["padding-box", "border-box extra", "3", "border-box 3"] {
        let source = format!("box-sizing:{text}");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(report.syntax().is_empty(), "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert!(validate_style_attribute(&source).is_err(), "{source}");
    }
}
