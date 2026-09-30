#![forbid(unsafe_code)]
//! Backgrounds 3 (2024-03-11), §2.2: background-color is a noninherited
//! <color> longhand with transparent initial. This suite exercises existing
//! lifecycle boundaries for the currently supported checked color grammar.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#background-color
//! Typed initial and new borrowed contribution variants are supplementary
//! functional tests after implementation, not compilation-failure RED.

use surgeist_css::*;

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}

fn color(source: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::BackgroundColor(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("checked background color")
    };
    value.value()
}

fn completed(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).expect("background-color must expand")
    else {
        panic!("completed terminal")
    };
    let [item] = items.items() else {
        panic!("one background-color contribution")
    };
    item.clone()
}

fn assert_ordinary(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert_eq!(item.property(), CssKnownProperty::BackgroundColor);
    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
    assert_eq!(
        item.ordinary_value().unwrap().property().known_property(),
        CssKnownProperty::BackgroundColor
    );
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
}

#[test]
fn metadata_identifies_a_noninherited_terminal() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::BackgroundColor.metadata().unwrap().kind()
    else {
        panic!("background-color is a longhand")
    };
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::BackgroundColor
    );
    assert!(!metadata.inherited_by_default());
}

#[test]
fn ordinary_colors_expand_once_retaining_checked_source_and_importance() {
    for (css, expected) in [
        (
            "background-color:red!important",
            CssColor::from_named(CssNamedColor::try_new("red").unwrap()),
        ),
        ("background-color:currentcolor", CssColor::current_color()),
        (
            "background-color:transparent!important",
            CssColor::transparent(),
        ),
    ] {
        let source = declaration(css);
        assert_eq!(color(&source), &expected);
        let item = completed(&source);
        assert_ordinary(&item, &source);
        assert_eq!(color(item.source()), &expected);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn all_five_css_wide_keywords_remain_symbolic() {
    for (text, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("background-color:{text}!important"));
        let item = completed(&source);
        assert_eq!(item.property(), CssKnownProperty::BackgroundColor);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.ordinary_value().is_none());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn pending_color_reentry_is_strict_reusable_and_retains_replacement_origins() {
    let source = declaration("background-color:var(--paint)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("substitution-dependent color is pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["", "red blue", "10px", "rgb(1 2)"] {
        let error = pending
            .reenter(parse_component_values(invalid).unwrap())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    for residual in ["var(--again)", "env(paint)", "attr(paint)"] {
        assert_eq!(
            pending
                .reenter(parse_component_values(residual).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
    }
    let parsed = parse_component_values("red").unwrap();
    let programmatic =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("currentcolor").unwrap()])
            .unwrap();
    assert!(matches!(
        parsed.items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        programmatic.items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    for replacement in [parsed, programmatic] {
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed color replacement")
            };
            let [item] = items.items() else {
                panic!("one replacement terminal")
            };
            assert_ordinary(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert_eq!(
                item.replacement_components().unwrap().items()[0].origin(),
                replacement.items()[0].origin()
            );
        }
    }
    let global = parse_component_values("unset").unwrap();
    let CssContributions::Longhands(items) = pending.reenter(global.clone()).unwrap() else {
        panic!("symbolic global replacement")
    };
    let [item] = items.items() else {
        panic!("one global terminal")
    };
    assert_eq!(item.property(), CssKnownProperty::BackgroundColor);
    assert_eq!(
        item.value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
    );
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&global));
    assert!(pending.source().same_occurrence(&source));
}

#[test]
fn normalization_keeps_ordered_occurrences_and_counts_one_unit_each() {
    let css = ".a{background-color:red!important; background-color:currentcolor}.b{background-color:var(--paint)}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    let [red, current, pending] = declarations.as_slice() else {
        panic!("three ordered occurrences")
    };
    for (index, value) in [red, current, pending].into_iter().enumerate() {
        assert_eq!(value.order(), index);
        assert_eq!(
            value.source().known().unwrap().property(),
            CssKnownProperty::BackgroundColor
        );
    }
    for value in [red, current] {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) = value.expansion()
        else {
            panic!("ordinary normalized color")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_ordinary(item, value.source());
    }
    assert_eq!(color(red.source()).to_specified_css().unwrap(), "red");
    assert!(color(current.source()).is_current_color());
    assert_eq!(red.source().importance(), CssImportance::Important);
    assert_eq!(
        red.source().position().unwrap().byte_offset().value(),
        css.find("background-color").unwrap()
    );
    let CssExpansion::Pending(handle) = pending.expansion() else {
        panic!("pending final occurrence")
    };
    assert!(handle.source().same_occurrence(pending.source()));
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 2).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2,
        }
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::BackgroundColor
    );
}

#[test]
fn parsed_literal_and_symbolic_colors_use_existing_checked_color_provider() {
    let literal = declaration("background-color:red");
    assert_eq!(
        color(&literal),
        &CssColor::from_named(CssNamedColor::try_new("red").unwrap())
    );
    assert_eq!(color(&literal).to_specified_css().unwrap(), "red");
    let symbolic = declaration("background-color:currentcolor");
    assert_eq!(color(&symbolic), &CssColor::current_color());
    assert_eq!(color(&symbolic).to_specified_css().unwrap(), "currentcolor");
    assert!(matches!(
        literal.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
}

#[test]
fn invalid_colors_drop_only_the_declaration_with_structured_diagnostics() {
    for (invalid, expected_code) in [
        ("red blue", CssErrorCode::InvalidPropertyValue),
        ("10px", CssErrorCode::InvalidColorSyntax),
        ("rgb(1 2)", CssErrorCode::InvalidColorSyntax),
    ] {
        let css = format!("background-color:{invalid};background-image:none");
        let report = parse_style_attribute(&css);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid color diagnostic")
        };
        assert_eq!(diagnostic.error().code(), expected_code);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        match diagnostic.error().kind() {
            ErrorKind::InvalidPropertyValue(detail) => {
                assert_eq!(detail.property(), CssKnownProperty::BackgroundColor);
            }
            ErrorKind::InvalidColorSyntax(_) => {}
            _ => panic!("structured property or color diagnostic"),
        }
        let [sibling] = report.syntax().as_slice() else {
            panic!("only valid sibling retained")
        };
        assert_eq!(
            sibling.known().unwrap().property(),
            CssKnownProperty::BackgroundImage
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}
