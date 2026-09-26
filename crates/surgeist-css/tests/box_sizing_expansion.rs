#![forbid(unsafe_code)]
//! CSS Sizing 3 box-sizing remains a non-inherited terminal longhand.
use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn box_sizing_has_a_non_inherited_intrinsic_initial() {
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
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::BoxSizing
    );
}

#[test]
fn box_sizing_keywords_contribute_one_typed_terminal_with_source_identity() {
    for (keyword, expected) in [
        ("content-box", CssBoxSizing::ContentBox),
        ("BORDER-BOX", CssBoxSizing::BorderBox),
    ] {
        let source = declaration(&format!("box-sizing:{keyword}!important"));
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
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            CssKnownProperty::BoxSizing
        );
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
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
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
        assert!(value.source().same_occurrence(item.source()));
    }
}
