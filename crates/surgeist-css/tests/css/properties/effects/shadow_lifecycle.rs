#![forbid(unsafe_code)]
//! Independent authored requirements, not computed shadow or paint assertions.
//! Backgrounds 3 CRD 2024-03-11 §6.1: <color>? && [<length>{2}
//! <length [0,∞]>? <length>?] && inset?; none initial, not inherited.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#box-shadow
//! Filter Effects 1 WD 2018-12-18 §6.1: <color>? && <length>{2,3}.
//! https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#funcdef-filter-drop-shadow
//! && reorders complete groups, not individual tokens inside a group.

use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    value.clone()
}

fn reject_interrupted(property: &str, value: &str) {
    let prefix = "/* 🦀 */ ";
    let unit = format!("{property}: {value};");
    let source = format!("{prefix}{unit} color: blue");
    let report = parse_style_attribute(&source);
    assert!(!report.is_clean(), "{source}: interrupted group accepted");
    assert_eq!(
        report.syntax().len(),
        1,
        "only following color survives: {source}"
    );
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        prefix.len() + unit.len()
    );
    assert_eq!(
        diagnostic.span().start().column().value() as usize,
        prefix.encode_utf16().count()
    );
    let error_position = diagnostic.error().position();
    let byte = error_position.byte_offset().value();
    assert!(byte >= prefix.len() && byte < prefix.len() + unit.len());
    assert_eq!(error_position.line().value(), 0);
    assert_eq!(
        error_position.column().value() as usize,
        source[..byte].encode_utf16().count()
    );
    assert!(validate_style_attribute(&source).is_err());
}

#[test]
fn box_color_cannot_interrupt_offsets_or_optional_lengths() {
    for value in ["1px red 2px", "1px 2px red 3px", "1px 2px 3px red 4px"] {
        reject_interrupted("box-shadow", value);
    }
}

#[test]
fn box_inset_cannot_interrupt_offsets_or_optional_lengths() {
    for value in [
        "1px inset 2px",
        "1px 2px inset 3px",
        "1px 2px 3px inset 4px",
    ] {
        reject_interrupted("box-shadow", value);
    }
}

#[test]
fn filter_color_cannot_interrupt_drop_shadow_length_group() {
    for value in ["drop-shadow(1px red 2px)", "drop-shadow(1px 2px red 3px)"] {
        reject_interrupted("filter", value);
    }
}

#[test]
fn backdrop_filter_color_cannot_interrupt_drop_shadow_length_group() {
    for value in ["drop-shadow(1px red 2px)", "drop-shadow(1px 2px red 3px)"] {
        reject_interrupted("backdrop-filter", value);
    }
}

#[test]
fn complete_box_length_groups_allow_all_color_inset_group_orders() {
    for value in [
        "red inset -1px 2px 3px -4px",
        "inset red -1px 2px 3px -4px",
        "red -1px 2px 3px -4px inset",
        "inset -1px 2px 3px -4px red",
        "-1px 2px 3px -4px red inset",
        "-1px 2px 3px -4px inset red",
        "red 1px 2px",
        "1px 2px red",
        "inset 1px 2px",
        "1px 2px inset",
        "calc(1px + 2em) calc(-2px) red, inset 0 0 0 -1px",
    ] {
        let source = format!("box-shadow:{value}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            validate_style_attribute(&source),
            Ok(report.syntax().clone())
        );
    }
}

#[test]
fn complete_drop_shadow_length_groups_allow_color_before_or_after() {
    for property in ["filter", "backdrop-filter"] {
        for value in [
            "red 1px 2px",
            "1px 2px red",
            "red -1px 2px 3px",
            "-1px 2px 3px red",
            "1px 2px",
            "calc(1px + 2em) -2px 0 red",
        ] {
            let source = format!("{property}:drop-shadow({value})");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(
                validate_style_attribute(&source),
                Ok(report.syntax().clone())
            );
        }
    }
}

#[test]
fn box_shadow_metadata_is_a_noninherited_terminal_with_an_intrinsic_initial() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::BoxShadow.metadata().unwrap().kind()
    else {
        panic!("box-shadow terminal")
    };
    assert!(!metadata.inherited_by_default());
    assert_eq!(
        metadata.initial_value().property().known_property(),
        CssKnownProperty::BoxShadow
    );
    assert!(matches!(
        metadata.initial_value().view(),
        CssInitialValueRef::Value(_)
    ));
}

#[test]
fn ordinary_box_shadow_expands_once_without_losing_source_or_importance() {
    for css in [
        "box-shadow:none",
        "box-shadow:inset 1px 2px red!important",
        "box-shadow:1px 2px,red 3px 4px",
    ] {
        let source = declaration(css);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one completed box-shadow")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        assert_eq!(value.property(), CssKnownProperty::BoxShadow);
        assert!(value.ordinary_value().is_some());
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), source.importance());
    }
}

#[test]
fn css_wide_box_shadow_expansion_stays_symbolic() {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("box-shadow:{text}!important"));
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("completed global")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        assert_eq!(value.property(), CssKnownProperty::BoxShadow);
        assert_eq!(value.value(), CssContributionValueRef::Global(keyword));
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
    }
}

#[test]
fn pending_box_shadow_reentry_is_strict_and_reusable() {
    let source = declaration("box-shadow:var(--shadow)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending box-shadow")
    };
    for invalid in ["", "1px red 2px", "1px 2px -3px", "none,1px 2px"] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    for residual in ["var(--again)", "env(shadow)", "attr(shadow)"] {
        assert_eq!(
            pending
                .reenter(parse_component_values(residual).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
    }
    for text in ["none", "red 1px 2px", "initial"] {
        let replacement = parse_component_values(text).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed replacement")
        };
        let [value] = values.items() else {
            panic!("one replacement contribution")
        };
        assert_eq!(value.property(), CssKnownProperty::BoxShadow);
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.replacement_components(), Some(&replacement));
    }
}
