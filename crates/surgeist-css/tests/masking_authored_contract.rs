#![forbid(unsafe_code)]
//! Masking 1 CRD20210805 authored grammar and composed lifecycle contracts.
//! Property-specific productions govern geometry, trailing fill and optional width.
//! Grammar/canonical/budget oracles are independently authored. Generic property
//! records belong in tests/common/property_expectations/records.rs later.
//! This file names only existing public symbols. New typed model/variant/provider
//! tests accompany functional implementation; absent symbols/stubs are not RED.
use surgeist_css::*;
type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

// These are specialized ordered shorthand expansion behavior oracles, not a
// second identity/metadata/dispatch inventory. The pin's grammar defines order.
const MASK_MEMBERS: &[&str] = &[
    "mask-image",
    "mask-position",
    "mask-size",
    "mask-repeat",
    "mask-origin",
    "mask-clip",
    "mask-composite",
    "mask-mode",
    "mask-border-source",
    "mask-border-slice",
    "mask-border-width",
    "mask-border-outset",
    "mask-border-repeat",
    "mask-border-mode",
];
const BORDER_MEMBERS: &[&str] = &[
    "mask-border-source",
    "mask-border-slice",
    "mask-border-width",
    "mask-border-outset",
    "mask-border-repeat",
    "mask-border-mode",
];

fn admitted(name: &str, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", name.to_ascii_uppercase());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(&validate_style_attribute(&css).unwrap(), report.syntax());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    // Assert the observable typed admission, after the real parser stimulus.
    // New absent names are not looked up to manufacture registration-only RED.
    assert_eq!(source.known().unwrap().property().canonical_name(), name);
    assert_eq!(source.importance(), CssImportance::Important);
    let position = source.position().unwrap();
    assert_eq!(position.byte_offset().value(), "/*😀*/".len());
    assert_eq!(
        position.column().value() as usize,
        "/*😀*/".encode_utf16().count()
    );
    assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
    source.clone()
}

fn fronts(name: &str, value: &str) -> Vec<CssDeclaration> {
    let parsed = admitted(name, value);
    // Derive the dispatch target only from a successfully admitted declaration;
    // assert specified outcomes through every existing public grammar boundary.
    let property = parsed.known().unwrap().property();
    let components = parse_component_values(value).unwrap();
    let snapshot = components.clone();
    let mut sources = vec![parsed];
    for source in [
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap(),
        parse_property_value_for_grammar(
            property.grammar(),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap(),
    ] {
        assert!(source.position().is_none());
        assert!(source.parsed_name().is_none());
        assert!(source.parsed_value().is_none());
        assert_eq!(source.value_components(), &components);
        sources.push(source);
    }
    for report in [
        parse_property_value_text(
            value,
            CssPropertyNameRef::Known(property),
            CssImportance::Important,
        ),
        parse_property_value_text_for_grammar(value, property.grammar(), CssImportance::Important),
    ] {
        assert!(
            report.is_clean(),
            "{name}:{value}: {:?}",
            report.diagnostics()
        );
        let source = report.syntax().as_ref().unwrap();
        assert_eq!(source.parsed_value().unwrap().source().as_str(), value);
        sources.push(source.clone());
    }
    assert_eq!(components, snapshot);
    sources
}

fn rejected(name: &str, value: &str) {
    let css = format!("/*😀*/color:red;{name}:{value}!important;width:2px");
    let report = parse_style_attribute(&css);
    let [before, after] = report.syntax().as_slice() else {
        panic!("only siblings: {css}")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    let [diagnostic] = report.diagnostics() else {
        panic!("one diagnostic: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("typed grammar error")
    };
    assert_eq!(detail.property().canonical_name(), name);
    let start = css.find(&format!("{name}:")).unwrap();
    let end = start + css[start..].find(';').unwrap() + 1;
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    assert_eq!(
        diagnostic.span().start().column().value() as usize,
        css[..start].encode_utf16().count()
    );
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    // The real error's named property selects checked boundaries without absent enums.
    let property = detail.property();
    let components = parse_component_values(value).unwrap();
    let snapshot = components.clone();
    for error in [
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap_err(),
        parse_property_value_for_grammar(
            property.grammar(),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap_err(),
    ] {
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    assert_eq!(components, snapshot);
}

fn canonical(name: &str, authored: &str, expected_value: &str) {
    let expected = format!("{name}: {expected_value} !important;");
    for source in fronts(name, authored) {
        let snapshot = source.clone();
        let components = source.value_components().clone();
        assert_eq!(source.to_specified_css().unwrap(), expected);
        let exact = Limits::new(65_536, 262_144, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for _ in 0..2 {
            assert_eq!(
                source
                    .to_specified_css_with_limits(Limits::new(65_536, 262_144, expected.len() - 1))
                    .unwrap_err()
                    .kind(),
                Kind::ByteLimit
            );
            assert_eq!(source, snapshot);
            assert_eq!(source.value_components(), &components);
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
    assert_eq!(
        admitted(name, expected_value).to_specified_css().unwrap(),
        expected
    );
}

fn expanded(source: &CssDeclaration, members: &[&str]) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete intrinsic expansion")
    };
    assert_members(&items, source, members, None);
    items
}

fn assert_members(
    items: &CssLonghandContributions,
    source: &CssDeclaration,
    members: &[&str],
    replacement: Option<&CssComponentValues>,
) {
    assert_eq!(
        items
            .items()
            .iter()
            .map(|v| v.property().canonical_name())
            .collect::<Vec<_>>(),
        members
    );
    for item in items.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        assert_eq!(item.source().position(), source.position());
        assert_eq!(item.source().value_components(), source.value_components());
        assert_eq!(item.replacement_components(), replacement);
        if let Some(values) = replacement {
            for (actual, original) in item
                .replacement_components()
                .unwrap()
                .items()
                .iter()
                .zip(values.items())
            {
                assert_eq!(actual.origin(), original.origin());
            }
        }
    }
}

fn payload(items: &CssLonghandContributions, index: usize, name: &str, expected: &str) {
    // Independently authored longhand stimulus expresses the semantic payload;
    // do not ask the shorthand serializer to generate expected longhand text.
    let reference = expanded(&admitted(name, expected), &[name]);
    assert_eq!(
        items.items()[index].ordinary_value().unwrap(),
        reference.items()[0].ordinary_value().unwrap()
    );
}

#[test]
fn complete_mask_groups_bind_boxes_and_order_image_position_repeat_composite_mode() {
    for input in [
        "url(mask.svg) left top / contain no-repeat padding-box no-clip exclude alpha",
        "alpha no-clip url(mask.svg) exclude padding-box no-repeat left top / contain",
        "exclude padding-box alpha left top / contain no-clip url(mask.svg) no-repeat",
    ] {
        canonical(
            "mask",
            input,
            "url(\"mask.svg\") left top / contain no-repeat padding-box no-clip exclude alpha",
        );
    }
    for input in ["padding-box no-clip", "no-clip padding-box"] {
        let source = admitted("mask", input);
        let values = expanded(&source, MASK_MEMBERS);
        payload(&values, 4, "mask-origin", "padding-box");
        payload(&values, 5, "mask-clip", "no-clip");
    }
    for (input, origin, clip) in [
        ("content-box", "content-box", "content-box"),
        ("content-box stroke-box", "content-box", "stroke-box"),
        ("stroke-box content-box", "stroke-box", "content-box"),
        ("no-clip", "border-box", "no-clip"),
    ] {
        let source = admitted("mask", input);
        let values = expanded(&source, MASK_MEMBERS);
        payload(&values, 4, "mask-origin", origin);
        payload(&values, 5, "mask-clip", clip);
    }
}

#[test]
fn layer_grammar_rejects_duplicate_groups_and_intrusions_between_position_and_size() {
    for input in [
        "alpha luminance",
        "add exclude",
        "no-clip no-clip",
        "border-box padding-box content-box",
        "border-box padding-box no-clip",
        "left alpha / contain",
        "left url(a) / contain",
        "/ contain",
        "left / contain / cover",
        "none,",
        "none,, alpha",
        "margin-box",
    ] {
        rejected("mask", input);
    }
    for input in [
        "left / contain alpha",
        "alpha left / contain",
        "no-clip",
        "fill-box",
        "view-box",
    ] {
        admitted("mask", input);
    }
}

#[test]
fn mask_layer_lists_keep_authored_arity_and_all_six_border_resets() {
    let source = admitted("mask", "none, url(a.svg) alpha padding-box");
    let before = source.clone();
    let values = expanded(&source, MASK_MEMBERS);
    for (index, name, expected) in [
        (0, "mask-image", "none, url(a.svg)"),
        (1, "mask-position", "0% 0%, 0% 0%"),
        (2, "mask-size", "auto, auto"),
        (3, "mask-repeat", "repeat, repeat"),
        (4, "mask-origin", "border-box, padding-box"),
        (5, "mask-clip", "border-box, padding-box"),
        (6, "mask-composite", "add, add"),
        (7, "mask-mode", "match-source, alpha"),
        (8, "mask-border-source", "none"),
        (9, "mask-border-slice", "0"),
        (10, "mask-border-width", "auto"),
        (11, "mask-border-outset", "0"),
        (12, "mask-border-repeat", "stretch"),
        (13, "mask-border-mode", "alpha"),
    ] {
        payload(&values, index, name, expected);
    }
    assert_eq!(source, before);
    canonical(
        "mask-mode",
        "ALPHA, luminance, match-source",
        "alpha, luminance, match-source",
    );
    canonical(
        "mask-clip",
        "content-box, no-clip, stroke-box, view-box",
        "content-box, no-clip, stroke-box, view-box",
    );
    canonical(
        "mask-origin",
        "padding-box, fill-box",
        "padding-box, fill-box",
    );
    canonical(
        "mask-composite",
        "EXCLUDE, subtract, intersect, add",
        "exclude, subtract, intersect, add",
    );
    canonical(
        "mask-size",
        "contain, 1px, auto auto",
        "contain, 1px auto, auto",
    );
    canonical(
        "mask-position",
        "center, right 1em bottom 2px",
        "center center, right 1em bottom 2px",
    );
    canonical(
        "mask-repeat",
        "repeat-x, round space, no-repeat",
        "repeat-x, round space, no-repeat",
    );
}

#[test]
fn new_layer_longhands_reject_wrong_domains_and_keep_clip_path_margin_independent() {
    for (name, value) in [
        ("mask-mode", "auto"),
        ("mask-mode", "alpha luminance"),
        ("mask-composite", "xor"),
        ("mask-composite", "add,"),
        ("mask-origin", "no-clip"),
        ("mask-origin", "margin-box"),
        ("mask-clip", "margin-box"),
        ("mask-clip", "no-clip border-box"),
        ("mask-size", "-0.000000000001px"),
        ("mask-size", "cover auto"),
        ("mask-position", "left 10px top"),
        ("mask-position", "inline-start"),
        ("mask-repeat", "repeat-x round"),
    ] {
        rejected(name, value);
    }
    canonical("clip-path", "margin-box", "margin-box");
    for name in ["mask-origin", "mask-clip"] {
        for value in [
            "content-box",
            "padding-box",
            "border-box",
            "fill-box",
            "stroke-box",
            "view-box",
        ] {
            admitted(name, value);
        }
    }
}

#[test]
fn mask_border_orders_six_domains_and_preserves_its_own_default_values() {
    canonical(
        "mask-border",
        "luminance round space url(frame.png) 1 2 3 4 fill / 2 3 4 5 / 1 2 3 4",
        "url(\"frame.png\") 1 2 3 4 fill / 2 3 4 5 / 1 2 3 4 round space luminance",
    );
    for input in [
        "none",
        "luminance",
        "round space",
        "10",
        "10 /",
        "10 / / 2",
        "10 / auto / 2",
    ] {
        expanded(&admitted("mask-border", input), BORDER_MEMBERS);
    }
    let source = admitted("mask-border", "none");
    let values = expanded(&source, BORDER_MEMBERS);
    for (index, name, expected) in [
        (0, "mask-border-source", "none"),
        (1, "mask-border-slice", "0"),
        (2, "mask-border-width", "auto"),
        (3, "mask-border-outset", "0"),
        (4, "mask-border-repeat", "stretch"),
        (5, "mask-border-mode", "alpha"),
    ] {
        payload(&values, index, name, expected);
    }
    for (name, input, expected) in [
        ("mask-border-source", "url(\"#🦀\")", "url(\"#🦀\")"),
        ("mask-border-slice", "1 2 3 4 fill", "1 2 3 4 fill"),
        ("mask-border-width", "auto 2 30% 4px", "auto 2 30% 4px"),
        ("mask-border-outset", "1 2px 3 4em", "1 2px 3 4em"),
        ("mask-border-repeat", "round space", "round space"),
        ("mask-border-mode", "LUMINANCE", "luminance"),
    ] {
        canonical(name, input, expected);
    }
}

#[test]
fn mask_border_imports_nonnegative_domains_without_widening_fill_or_slash_grammar() {
    for (name, input) in [
        ("mask-border-slice", "-0.000000000001"),
        ("mask-border-slice", "-1%"),
        ("mask-border-slice", "fill 1"),
        ("mask-border-slice", "1 fill 2"),
        ("mask-border-slice", "1 2 3 4 5"),
        ("mask-border-width", "-1px"),
        ("mask-border-width", "-1"),
        ("mask-border-width", "-1%"),
        ("mask-border-outset", "1%"),
        ("mask-border-outset", "-1em"),
        ("mask-border-outset", "-0.000000000001"),
        ("mask-border-repeat", "no-repeat"),
        ("mask-border-repeat", "round space repeat"),
        ("mask-border-mode", "match-source"),
        ("mask-border-source", "none, url(a)"),
        ("mask-border", "/ 1"),
        ("mask-border", "10 / /"),
        ("mask-border", "10 / 1 / 2 / 3"),
        ("mask-border", "alpha luminance"),
        ("mask-border", "none url(a)"),
    ] {
        rejected(name, input);
    }
    // Independent prior grammar controls: imported value semantics do not replace
    // source-specific fill grammar, defaults or shorthand slash productions.
    canonical("border-image-slice", "fill 1 2", "1 2 fill");
    admitted("border-image-width", "1");
    admitted("mask-border-slice", "-0");
    admitted("mask-border-slice", "-0%");
    admitted("mask-border-width", "calc(1px - 2px)");
}

#[test]
fn clip_rule_mask_type_and_layer_border_modes_keep_distinct_domains() {
    canonical("clip-rule", "EVENODD", "evenodd");
    canonical("mask-type", "ALPHA", "alpha");
    for (name, value) in [
        ("clip-rule", "alpha"),
        ("clip-rule", "nonzero evenodd"),
        ("mask-type", "match-source"),
        ("mask-type", "alpha,luminance"),
        ("mask-border-mode", "match-source"),
    ] {
        rejected(name, value);
    }
    // CSS basic shapes keep their own fill rule; clip-rule is a separate SVG
    // property, with no mutation of the neighboring clip-path authored model.
    let report = parse_style_attribute(
        "clip-rule:evenodd;clip-path:polygon(nonzero,0 0);mask-type:alpha;mask-mode:match-source;mask-border-mode:luminance",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("clip-path")
    };
    assert_eq!(
        value.value().serialize_specified().unwrap(),
        "polygon(nonzero, 0 0)"
    );
}

fn lifecycle(name: &str, ordinary: &str, invalid: &str, members: &[&str]) {
    for (text, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for source in fronts(name, text) {
            let values = expanded(&source, members);
            assert!(
                values
                    .items()
                    .iter()
                    .all(|item| item.value() == CssContributionValueRef::Global(keyword))
            );
        }
    }
    for source in fronts(name, "var(--mask)") {
        let snapshot = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("whole declaration pending")
        };
        assert!(handle.source().same_occurrence(&source));
        for residual in [
            "var(--again)",
            "env(mask)",
            "attr(data-mask)",
            "calc(var(--n) * 1px)",
        ] {
            assert_eq!(
                handle
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        assert!(matches!(
            handle
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        let replacement = parse_component_values(ordinary).unwrap();
        let before = replacement.clone();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("strict completed reentry")
            };
            assert_members(&values, &source, members, Some(&replacement));
        }
        let global = parse_component_values("revert-layer").unwrap();
        let CssContributions::Longhands(values) = handle.reenter(global.clone()).unwrap() else {
            panic!("global retry")
        };
        assert_members(&values, &source, members, Some(&global));
        assert!(
            values.items().iter().all(|item| item.value()
                == CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer))
        );
        assert_eq!(source, snapshot);
        assert_eq!(replacement, before);
        assert!(handle.source().same_occurrence(&source));
    }
}

#[test]
fn mask_and_border_shorthands_reenter_once_with_original_and_replacement_provenance() {
    lifecycle(
        "mask",
        "none, url(a) alpha no-clip",
        "alpha luminance",
        MASK_MEMBERS,
    );
    lifecycle(
        "mask-border",
        "url(frame) 1 2 fill / auto / 2 round luminance",
        "1 / /",
        BORDER_MEMBERS,
    );
}

#[test]
fn mask_longhand_lists_and_svg_keywords_share_global_pending_retry_contracts() {
    for (name, ordinary, invalid) in [
        ("mask-size", "contain, 1px", "-1px"),
        (
            "mask-position",
            "center, right 1em bottom 2px",
            "left right",
        ),
        ("mask-repeat", "round space, repeat-x", "repeat-x round"),
        ("mask-mode", "alpha, match-source", "auto"),
        ("mask-origin", "fill-box, view-box", "no-clip"),
        ("mask-clip", "no-clip, padding-box", "margin-box"),
        ("mask-composite", "exclude, subtract", "xor"),
        ("mask-border-source", "url(#mask)", "none,url(a)"),
        ("mask-border-slice", "1 2% fill", "fill 1"),
        ("mask-border-width", "auto 2px", "-1%"),
        ("mask-border-outset", "1 2em", "1%"),
        ("mask-border-repeat", "round space", "no-repeat"),
        ("mask-border-mode", "luminance", "match-source"),
        ("clip-rule", "evenodd", "alpha"),
        ("mask-type", "alpha", "match-source"),
        ("clip-path", "circle(1px)", "none border-box"),
    ] {
        lifecycle(name, ordinary, invalid, &[name]);
    }
}

#[test]
fn programmatic_keyword_replacements_keep_their_origin_and_original_occurrence() {
    for (name, keyword) in [
        ("mask", "none"),
        ("mask-border", "none"),
        ("mask-size", "contain"),
        ("mask-position", "center"),
        ("mask-repeat", "no-repeat"),
        ("mask-mode", "alpha"),
        ("mask-origin", "fill-box"),
        ("mask-clip", "no-clip"),
        ("mask-composite", "exclude"),
        ("mask-border-source", "none"),
        ("mask-border-width", "auto"),
        ("mask-border-repeat", "round"),
        ("mask-border-mode", "luminance"),
        ("clip-rule", "evenodd"),
        ("mask-type", "alpha"),
        ("clip-path", "none"),
    ] {
        let source = admitted(name, "var(--mask)");
        let snapshot = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(keyword).unwrap()])
                .unwrap();
        assert_eq!(
            replacement.items()[0].origin(),
            &CssValueOrigin::Programmatic
        );
        let members: &[&str] = if name == "mask" {
            MASK_MEMBERS
        } else if name == "mask-border" {
            BORDER_MEMBERS
        } else {
            &[name]
        };
        for _ in 0..2 {
            let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed")
            };
            assert_members(&items, &source, members, Some(&replacement));
            assert_eq!(
                items.items()[0].replacement_components().unwrap().items()[0].origin(),
                &CssValueOrigin::Programmatic
            );
        }
        assert_eq!(source, snapshot);
        assert!(handle.source().same_occurrence(&source));
    }
}

#[test]
fn original_repaired_closures_fail_checked_and_reentry_but_closed_retry_succeeds() {
    for (name, ordinary) in [
        ("mask", "none"),
        ("mask-border", "none"),
        ("mask-mode", "alpha"),
        ("clip-rule", "evenodd"),
        ("mask-type", "alpha"),
        ("clip-path", "circle()"),
    ] {
        let source = admitted(name, "var(--mask)");
        let property = source.known().unwrap().property();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for input in [format!("{ordinary}/*"), "var(--mask".to_owned()] {
            let components = parse_component_values(&input).unwrap();
            let snapshot = components.clone();
            let serialized = components.serialize().unwrap();
            let origin = (0..=serialized.as_css().len())
                .find_map(|offset| match serialized.origin_at(offset) {
                    Some(CssSerializedOrigin::Token(
                        v @ CssValueOrigin::ImplicitClosure { .. },
                    )) => Some(v.clone()),
                    Some(CssSerializedOrigin::End(Some(
                        v @ CssValueOrigin::ImplicitClosure { .. },
                    ))) => Some(v.clone()),
                    _ => None,
                })
                .or_else(|| {
                    components.items().iter().find_map(|v| match v.view() {
                        CssComponentValueRef::Function(v) => match v.closing_origin() {
                            origin @ CssValueOrigin::ImplicitClosure { .. } => Some(origin.clone()),
                            _ => None,
                        },
                        _ => None,
                    })
                })
                .expect("original closure origin");
            for error in [
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap_err(),
                parse_property_value_for_grammar(
                    property.grammar(),
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap_err(),
            ] {
                assert!(matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ));
                assert_eq!(
                    error.origin(),
                    &CssSerializedOrigin::End(Some(origin.clone()))
                );
            }
            // Residual substitution has documented precedence before grammar.
            if input.starts_with("var(") {
                assert_eq!(
                    handle.reenter(components.clone()).unwrap_err().kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
            } else {
                let error = handle.reenter(components.clone()).unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                    panic!("strict original closure")
                };
                assert_eq!(error.origin(), &CssSerializedOrigin::End(Some(origin)));
            }
            assert_eq!(components, snapshot);
        }
        assert!(
            handle
                .reenter(parse_component_values(&format!("{ordinary}/**/")).unwrap())
                .is_ok()
        );
        assert!(handle.source().same_occurrence(&source));
    }
}

fn exact_budget(source: &CssDeclaration, expected: &str, input: usize, projection: usize) {
    let snapshot = source.clone();
    let exact = Limits::new(input, projection, expected.len());
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(input - 1, projection, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(input, projection - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(input, projection, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(
            source
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(*source, snapshot);
    }
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}

#[test]
fn existing_source_graph_tariffs_are_cumulative_exact_and_atomic_at_declaration_boundary() {
    for (name, authored, expected, input, projection) in [
        ("mask", "none", "mask: none !important;", 5, 5),
        ("mask", "none, none", "mask: none, none !important;", 7, 7),
        (
            "mask-size",
            "contain",
            "mask-size: contain !important;",
            4,
            4,
        ),
        ("mask-size", "1px", "mask-size: 1px auto !important;", 6, 7),
        (
            "mask-position",
            "center",
            "mask-position: center center !important;",
            6,
            6,
        ),
        (
            "mask-repeat",
            "no-repeat",
            "mask-repeat: no-repeat !important;",
            6,
            6,
        ),
        (
            "clip-path",
            "circle()",
            "clip-path: circle() !important;",
            4,
            4,
        ),
    ] {
        for source in fronts(name, authored) {
            exact_budget(&source, expected, input, projection);
        }
    }
    let source = admitted("mask", "none, none");
    let snapshot = source.clone();
    for bytes in [
        "mask: none".len(),
        "mask: none, ".len(),
        "mask: none, none".len(),
    ] {
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(7, 7, bytes))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(source, snapshot);
    }
    exact_budget(&source, "mask: none, none !important;", 7, 7);
}

#[test]
fn checked_component_source_budget_counts_comma_and_both_layers_cumulatively() {
    // Two four-byte identifiers plus one comma; all are depth-zero leaf tokens.
    let components = parse_component_values("none,none").unwrap();
    let snapshot = components.clone();
    let exact = CssComponentValueLimits::try_new(0, 3, 9).unwrap();
    let admitted_components =
        CssComponentValues::try_new_with_limits(components.items().to_vec(), exact).unwrap();
    assert_eq!(
        admitted_components.serialize().unwrap().as_css(),
        "none,none"
    );
    assert_eq!(admitted_components, components);
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(0, 2, 9).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(0, 3, 8).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            CssComponentValues::try_new_with_limits(components.items().to_vec(), limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(components, snapshot);
    }
    let source = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Mask),
        admitted_components,
        CssImportance::Important,
    )
    .unwrap();
    exact_budget(&source, "mask: none, none !important;", 7, 7);
}

#[test]
fn all_masking_globals_spend_one_keyword_without_spending_shorthand_member_output() {
    for name in [
        "mask",
        "mask-border",
        "mask-mode",
        "mask-clip",
        "mask-origin",
        "mask-composite",
        "clip-rule",
        "mask-type",
    ] {
        for keyword in ["inherit", "initial", "unset", "revert", "revert-layer"] {
            let source = admitted(name, keyword);
            exact_budget(&source, &format!("{name}: {keyword} !important;"), 3, 3);
        }
    }
}

#[test]
fn composed_normalization_retains_order_importance_and_exact_twenty_seven_contributions() {
    let css = "/*😀*/.a{mask:none!important;mask-border:none;mask-mode:alpha,luminance;clip-rule:evenodd;mask-type:alpha;clip-path:circle();clip:auto;mask-image:none;mask:var(--m)}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let snapshot = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 9, 27).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 9);
    for (order, (value, name)) in values
        .iter()
        .zip([
            "mask",
            "mask-border",
            "mask-mode",
            "clip-rule",
            "mask-type",
            "clip-path",
            "clip",
            "mask-image",
            "mask",
        ])
        .enumerate()
    {
        assert_eq!(value.order(), order);
        assert_eq!(
            value.source().known().unwrap().property().canonical_name(),
            name
        );
        assert_eq!(
            value.source().importance(),
            if order == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        match value.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(items)) => {
                let names: &[&str] = if order == 0 {
                    MASK_MEMBERS
                } else if order == 1 {
                    BORDER_MEMBERS
                } else {
                    &[name]
                };
                assert_members(items, value.source(), names, None);
            }
            CssExpansion::Pending(handle) if order == 8 => {
                assert!(handle.source().same_occurrence(value.source()))
            }
            _ => panic!("composed expansion contract"),
        }
    }
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 9, 26).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 26
        }
    );
    assert_eq!(error.declaration_order(), Some(8));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(values[8].source())
    );
    assert_eq!(report, snapshot);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}

// Functional implementation companions, not absent-symbol RED:
// - checked narrowed six-value mask box; no margin/no-clip in origin;
// - all new keyword enum/list constructors, empty rejection and getters;
// - complete Mask layer constructor and authored geometry one/pair/no-clip views;
// - MaskBorder six-field aggregate, programmatic Number/Percentage/hinted math /
//   Length/LengthPercentage/Auto branches, all four side repetition rules;
// - new serializers with documented aggregate/leaf tariffs, suppressed child
//   traversal, prefix/sibling budgets, failure after separator and valid retry;
// - constructed size-only Mask keeps projection-only default 0% 0%, non-BMP
//   URL bytes, original numeric graph/origin and exact omission distinctions;
// - programmatic reentry components retain Programmatic origin alongside parsed
//   replacement spans, equal rounded unequal edge components do not compress;
// - common records independently assert all exact initials/inheritance/aliases/
//   source IDs/ordered settable and reset-only members and their generic consumers.
