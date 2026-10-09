#![forbid(unsafe_code)]
//! Motion WD 2024-11-05 authored syntax and lifecycle contract.
//! Quoted offset-path imports its complete property grammar; Box3 coord-box
//! has six names. Direct Motion positions import the full Values5 position grammar.
//! Bearing, paths, anchors, tangent rotation and resource loading stay symbolic.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};
const LONGHANDS: [&str; 5] = [
    "offset-position",
    "offset-path",
    "offset-distance",
    "offset-rotate",
    "offset-anchor",
];
const NAMES: [&str; 6] = [
    "offset-position",
    "offset-path",
    "offset-distance",
    "offset-rotate",
    "offset-anchor",
    "offset",
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn property(name: &str) -> CssKnownProperty {
    let property = CssKnownProperty::from_name(name).unwrap_or_else(|| {
        panic!("selected authored property {name} must accept public discovery")
    });
    assert_eq!(property.canonical_name(), name);
    assert_eq!(
        CssKnownProperty::from_name(&name.to_ascii_uppercase()),
        Some(property)
    );
    assert_eq!(
        CssPropertyGrammar::from_name(name).unwrap(),
        property.grammar()
    );
    property
}

fn declaration(name: &str, input: &str, front: usize) -> CssDeclaration {
    let p = property(name);
    let source = match front {
        0 => {
            let css = format!("/*😀*/{name}:{input}!important");
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
            let [source] = report.syntax().as_slice() else {
                panic!("one authored occurrence")
            };
            assert!(source.position().is_some());
            assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
            source.clone()
        }
        1 | 2 => {
            let components = parse_component_values(input).unwrap();
            let before = components.clone();
            let source = checked(p, components.clone(), front == 2).unwrap();
            assert_eq!(components, before);
            assert_eq!(source.value_components(), &components);
            assert!(source.position().is_none());
            assert!(source.parsed_value().is_none());
            source
        }
        3 | 4 => {
            let report = if front == 3 {
                parse_property_value_text(
                    input,
                    CssPropertyNameRef::Known(p),
                    CssImportance::Important,
                )
            } else {
                parse_property_value_text_for_grammar(input, p.grammar(), CssImportance::Important)
            };
            assert!(
                report.is_clean(),
                "{name}:{input}: {:?}",
                report.diagnostics()
            );
            let source = report
                .syntax()
                .as_ref()
                .expect("retained direct value")
                .clone();
            assert!(source.position().is_none());
            assert_eq!(source.parsed_value().unwrap().source().as_str(), input);
            source
        }
        _ => unreachable!(),
    };
    assert_eq!(source.known().unwrap().property(), p);
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}

fn checked(
    property: CssKnownProperty,
    components: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    }
}

fn accept(name: &str, cases: &[(&str, &str)]) {
    for &(input, expected) in cases {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let before = source.clone();
            let output = format!("{name}: {expected} !important;");
            assert_eq!(
                source.to_specified_css().unwrap(),
                output,
                "{input} front {front}"
            );
            assert_eq!(source, before);
            let reparse = parse_style_attribute(&output);
            assert!(reparse.is_clean(), "{output}: {:?}", reparse.diagnostics());
            let [reparsed] = reparse.syntax().as_slice() else {
                panic!("one reentered occurrence")
            };
            assert_eq!(reparsed.known().unwrap().property(), property(name));
            assert_eq!(reparsed.importance(), CssImportance::Important);
            assert_eq!(reparsed.to_specified_css().unwrap(), output);
        }
    }
}

fn invalid(name: &str, values: &[&str]) {
    let p = property(name);
    for value in values {
        let middle = format!("{name}:{value};");
        let css = format!("/*😀*/color:red!important;{middle}color:blue");
        let report = parse_style_attribute(&css);
        let [before, after] = report.syntax().as_slice() else {
            panic!("only adjacent colors survive {css}")
        };
        assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(after.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(before.importance(), CssImportance::Important);
        assert_eq!(after.importance(), CssImportance::Normal);
        let [diagnostic] = report.diagnostics() else {
            panic!("one atomic diagnostic {css}")
        };
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("actual property grammar diagnostic")
        };
        assert_eq!(detail.property(), p);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let start = css.find(&middle).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + middle.len()
        );
        assert_eq!(
            diagnostic.span().start().column().value() as usize,
            css[..start].encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let components = parse_component_values(value).unwrap();
        let original = components.clone();
        for grammar in [false, true] {
            assert!(
                checked(p, components.clone(), grammar).is_err(),
                "{name}:{value}"
            );
        }
        assert_eq!(components, original);
    }
}

#[test]
fn offset_path_admits_exact_coord_boxes_alone_or_with_any_path_branch_in_either_order() {
    for name in [
        "content-box",
        "padding-box",
        "border-box",
        "fill-box",
        "stroke-box",
        "view-box",
    ] {
        accept("offset-path", &[(name, name)]);
        for (path, canonical) in [
            ("circle()", "circle()"),
            ("ray(90deg)", "ray(90deg)"),
            ("url('#p')", "url(\"#p\")"),
            ("src('#p')", "src(\"#p\")"),
        ] {
            let expected = format!("{canonical} {name}");
            for input in [format!("{path} {name}"), format!("{name} {path}")] {
                accept("offset-path", &[(input.as_str(), expected.as_str())]);
            }
        }
    }
    accept(
        "offset-path",
        &[("NONE", "none"), (r"v\69 ew-box", "view-box")],
    );
}

#[test]
fn omitted_path_box_remains_distinct_from_authored_border_box_without_inserting_geometry() {
    accept(
        "offset-path",
        &[
            ("circle()", "circle()"),
            ("circle() border-box", "circle() border-box"),
            ("border-box", "border-box"),
        ],
    );
    let omitted = declaration("offset-path", "circle()", 0);
    let explicit = declaration("offset-path", "circle() border-box", 0);
    assert_ne!(
        omitted.known().unwrap().property_value(),
        explicit.known().unwrap().property_value()
    );
}

#[test]
fn path_rejects_margin_examples_foreign_boxes_duplicates_and_exclusive_none() {
    invalid(
        "offset-path",
        &[
            "margin-box",
            "circle() margin-box",
            "ray(90deg) margin-box",
            "text",
            "no-clip",
            "none border-box",
            "none ray(90deg)",
            "circle() ellipse()",
            "ray(0deg) circle()",
            "fill-box stroke-box",
            "ray(90deg) view-box border-box",
            "url('#p') src('#q')",
            "circle(),circle()",
            "linear-gradient(red,blue)",
            "inherit circle()",
            "1px",
        ],
    );
}

#[test]
fn imported_all_eight_shapes_keep_adopted_radius_position_path_and_command_syntax() {
    accept(
        "offset-path",
        &[
            ("inset(1px)", "inset(1px)"),
            ("circle(calc(1in + 1in))", "circle(calc(192px))"),
            (
                "ellipse(abs(-2em) hypot(3em,4em))",
                "ellipse(calc(2em) calc(5em))",
            ),
            (
                "polygon(nonzero,0 0,100% 0)",
                "polygon(nonzero, 0 0, 100% 0)",
            ),
            ("rect(1px auto 2px 0)", "rect(1px auto 2px 0)"),
            ("xywh(0 0 calc(1px - 2px) 3px)", "xywh(0 0 calc(-1px) 3px)"),
            (
                "path(evenodd,'M01.00,0 L1e2 -0 Z')",
                "path(evenodd, \"M01.00,0 L1e2 -0 Z\")",
            ),
            (
                "shape(from 0px 0px, arc by 1px 2% of 3px)",
                "shape(from 0px 0px, arc by 1px 2% of 3px)",
            ),
            (
                "circle(at inline-end block-start)",
                "circle(at block-start inline-end)",
            ),
        ],
    );
    invalid(
        "offset-path",
        &[
            "circle(1px 2px)",
            "ellipse(1px)",
            "shape(from 0 0 line to 1px 2px)",
            "shape(from 0 0, arc to 1px 2px)",
        ],
    );
}

#[test]
fn imported_urls_preserve_function_modifiers_target_and_empty_resource_without_loading() {
    accept(
        "offset-path",
        &[
            ("url()", "url(\"\")"),
            ("src('')", "src(\"\")"),
            (
                "SRC(\"é.svg\" CORS integrity(\"sha256\"))",
                "src(\"é.svg\" CORS integrity(\"sha256\"))",
            ),
            ("url('#p') fill-box", "url(\"#p\") fill-box"),
        ],
    );
}

#[test]
fn ray_accepts_all_five_sizes_and_unordered_whole_arguments_in_grammar_order() {
    for size in [
        "closest-side",
        "closest-corner",
        "farthest-side",
        "farthest-corner",
        "sides",
    ] {
        let expected = if size == "closest-side" {
            "ray(90deg contain at left top)".to_owned()
        } else {
            format!("ray(90deg {size} contain at left top)")
        };
        for input in [
            format!("ray(90deg {size} contain at top left)"),
            format!("ray(contain {size} at top left 90deg)"),
            format!("ray(at top left 90deg contain {size})"),
        ] {
            accept("offset-path", &[(input.as_str(), expected.as_str())]);
        }
    }
}

#[test]
fn ray_keeps_bearing_magnitude_and_math_without_direction_or_geometry_resolution() {
    accept(
        "offset-path",
        &[
            ("ray(0deg)", "ray(0deg)"),
            ("ray(-90deg)", "ray(-90deg)"),
            ("ray(450deg)", "ray(450deg)"),
            ("ray(.25turn)", "ray(0.25turn)"),
            ("ray(100grad)", "ray(100grad)"),
            ("ray(calc(1turn - 90deg))", "ray(calc(270deg))"),
            ("ray(90deg at 25%)", "ray(90deg at 25% center)"),
            (
                "ray(90deg at bottom 2% right 1px)",
                "ray(90deg at right 1px bottom 2%)",
            ),
        ],
    );
}

#[test]
fn ray_omits_defined_default_size_but_retains_explicit_authored_presence() {
    accept(
        "offset-path",
        &[
            ("ray(90deg closest-side)", "ray(90deg)"),
            ("ray(90deg at center)", "ray(90deg at center center)"),
        ],
    );
    let omitted = declaration("offset-path", "ray(90deg)", 0);
    let size = declaration("offset-path", "ray(90deg closest-side)", 0);
    let position = declaration("offset-path", "ray(90deg at center)", 0);
    assert_ne!(
        omitted.known().unwrap().property_value(),
        size.known().unwrap().property_value()
    );
    assert_ne!(
        omitted.known().unwrap().property_value(),
        position.known().unwrap().property_value()
    );
}

#[test]
fn ray_rejects_missing_angle_duplicate_fields_unitless_angle_and_foreign_direct_positions() {
    invalid(
        "offset-path",
        &[
            "ray()",
            "ray(closest-side)",
            "ray(contain)",
            "ray(at center)",
            "ray(0)",
            "ray(1px)",
            "ray(25%)",
            "ray(0deg 90deg)",
            "ray(90deg contain contain)",
            "ray(90deg sides closest-side)",
            "ray(90deg at left at right)",
            "ray(90deg at)",
            "ray(90deg at left 1px top)",
            "ray(90deg at inline-end y-start)",
            "ray(90deg at end inline-start)",
            "ray(90deg,contain)",
            "ray(calc(1px))",
        ],
    );
}

#[test]
fn distance_keeps_negative_and_mixed_length_percentage_authored_values_without_path_context() {
    accept(
        "offset-distance",
        &[
            ("0", "0"),
            ("-25%", "-25%"),
            ("-2em", "-2em"),
            ("1e-999px", "0px"),
            ("calc(1px + 2%)", "calc(2% + 1px)"),
            ("calc(1px - 2px)", "calc(-1px)"),
        ],
    );
    invalid(
        "offset-distance",
        &[
            "none",
            "auto",
            "normal",
            "1",
            "1deg",
            "1px 2px",
            "calc(1deg)",
        ],
    );
}

#[test]
fn position_and_anchor_share_the_imported_generic_position_domain() {
    for name in ["offset-position", "offset-anchor"] {
        accept(
            name,
            &[
                ("AUTO", "auto"),
                ("top", "center top"),
                ("left", "left center"),
                ("center", "center center"),
                ("bottom right", "right bottom"),
                ("25%", "25% center"),
                ("25% 75%", "25% 75%"),
                ("inline-end block-start", "block-start inline-end"),
                ("end start", "end start"),
                ("x-start y-end", "x-start y-end"),
                ("bottom 2% right 1px", "right 1px bottom 2%"),
                ("calc(1px + 2%) -3px", "calc(2% + 1px) -3px"),
            ],
        );
        invalid(
            name,
            &[
                "none",
                "left 1px top",
                "center 1px top",
                "inline-end y-start",
                "end inline-start",
                "x-start block-end",
                "1deg",
                "left right",
                "top bottom",
                "auto center",
                "left,top",
            ],
        );
    }
    accept("offset-position", &[("NORMAL", "normal")]);
    invalid("offset-anchor", &["normal"]);
}

#[test]
fn rotate_keeps_absent_modifier_distinct_from_auto_and_reverse_with_optional_angle() {
    accept(
        "offset-rotate",
        &[
            ("AUTO", "auto"),
            ("REVERSE", "reverse"),
            ("90deg", "90deg"),
            ("auto .25turn", "auto 0.25turn"),
            ("100grad reverse", "reverse 100grad"),
            ("auto 0deg", "auto 0deg"),
            ("reverse 0deg", "reverse 0deg"),
            ("reverse calc(1turn - 90deg)", "reverse calc(270deg)"),
        ],
    );
    let alone = declaration("offset-rotate", "90deg", 0);
    let automatic = declaration("offset-rotate", "auto 90deg", 0);
    assert_ne!(
        alone.known().unwrap().property_value(),
        automatic.known().unwrap().property_value()
    );
}

#[test]
fn rotate_rejects_duplicate_modifier_angle_and_unitless_zero_without_transform_exception() {
    invalid(
        "offset-rotate",
        &[
            "auto reverse",
            "auto auto",
            "reverse reverse",
            "90deg 180deg",
            "0",
            "1px",
            "25%",
            "none",
            "normal",
            "auto,90deg",
            "inherit 90deg",
        ],
    );
}

#[test]
fn shorthand_imports_full_quoted_path_and_all_five_quoted_properties() {
    accept(
        "offset",
        &[
            ("none", "none"),
            ("fill-box", "fill-box"),
            ("normal", "normal"),
            ("auto", "auto"),
            ("25%", "25% center"),
            (
                "none -25% reverse 90deg / bottom right",
                "none -25% reverse 90deg / right bottom",
            ),
            ("view-box 25%", "view-box 25%"),
        ],
    );
    accept(
        "offset",
        &[
            (
                "url('#p') view-box 25% 90deg",
                "url(\"#p\") view-box 25% 90deg",
            ),
            (
                "normal stroke-box url('#p') reverse 90deg 25% / auto",
                "normal url(\"#p\") stroke-box 25% reverse 90deg / auto",
            ),
            (
                "top left ray(.25turn closest-side) reverse 90deg -25% / center",
                "left top ray(0.25turn) -25% reverse 90deg / center center",
            ),
            ("none 0", "none 0"),
            ("none 0deg", "none 0deg"),
            ("none auto", "none auto"),
        ],
    );
}

#[test]
fn shorthand_rejects_empty_required_group_and_distance_rotation_without_a_path() {
    invalid(
        "offset",
        &[
            "/ center",
            "90deg",
            "normal 90deg",
            "auto reverse",
            "normal 1px 2px",
            "none /",
            "none / center / left",
            "none 1px 2px",
            "ray(90deg) auto reverse",
            "circle() none",
            "none circle()",
            "margin-box",
            "inherit none",
            "none,none",
            "border-box none",
        ],
    );
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary longhand contribution")
    };
    items
}

fn terminals(name: &str) -> Vec<&str> {
    if name == "offset" {
        LONGHANDS.to_vec()
    } else {
        vec![name]
    }
}

fn member(
    items: &CssLonghandContributions,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    let expected = terminals(source.known().unwrap().property().canonical_name());
    assert_eq!(items.items().len(), expected.len());
    for (item, name) in items.items().iter().zip(expected) {
        assert_eq!(item.property(), property(name));
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(
            item.source().known().unwrap().grammar(),
            source.known().unwrap().grammar()
        );
        assert_eq!(item.replacement_components(), replacement);
    }
}

fn metadata(name: &str, inherited: bool) {
    let p = property(name);
    let meta = p.metadata().expect("Motion intrinsic metadata");
    assert_eq!(meta.grammar(), p.grammar());
    let CssPropertyKindRef::Longhand(longhand) = meta.kind() else {
        panic!("one intrinsic longhand")
    };
    assert_eq!(longhand.property().known_property(), p);
    assert_eq!(longhand.inherited_by_default(), inherited);
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("fixed symbolic initial, no user-agent requirement")
    };
    assert_eq!(initial.property().known_property(), p);
    // Independent none/zero/normal/auto payload assertions are functional
    // new-variant tests with implementation, not inferred by comparing production.
}

fn ordinary(name: &str, value: &str) {
    for front in 0..5 {
        let source = declaration(name, value, front);
        let before = source.clone();
        let items = completed(&source);
        member(&items, &source, None);
        for (item, terminal) in items.items().iter().zip(terminals(name)) {
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                property(terminal)
            );
        }
        assert_eq!(source, before);
    }
}

fn globals(name: &str) {
    for (input, keyword) in GLOBALS {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let items = completed(&source);
            member(&items, &source, None);
            for item in items.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.ordinary_value().is_none());
            }
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {input} !important;")
            );
        }
    }
}

fn pending(name: &str, valid: &str, invalid: &str) {
    let p = property(name);
    for authored in [
        "bogus var(--motion)",
        "bogus env(motion)",
        "bogus attr(data-motion)",
    ] {
        for front in 0..5 {
            let source = declaration(name, authored, front);
            let original = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole pending source")
            };
            assert!(handle.source().same_occurrence(&source));
            for _ in 0..2 {
                for bad in [invalid, "inherit bogus", "bogus"] {
                    let components = parse_component_values(bad).unwrap();
                    let direct = checked(p, components.clone(), false).unwrap_err();
                    let error = handle.reenter(components).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                        panic!("typed strict replacement error")
                    };
                    assert_eq!(actual.kind(), direct.kind());
                    assert_eq!(actual.origin(), direct.origin());
                }
                for residual in [
                    "bogus var(--again)",
                    "[f(env(motion))]",
                    "f(attr(data-motion))",
                    r"f(v\61 r(--again))",
                ] {
                    assert_eq!(
                        handle
                            .reenter(parse_component_values(residual).unwrap())
                            .unwrap_err()
                            .kind(),
                        &CssExpansionErrorKind::ResidualSubstitution
                    );
                }
                let replacement = parse_component_values(&format!("/*😀*/{valid}")).unwrap();
                let before = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one resolved terminal")
                };
                member(&values, &source, Some(&replacement));
                assert!(
                    values
                        .items()
                        .iter()
                        .all(|item| item.ordinary_value().is_some())
                );
                assert_eq!(replacement, before);
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("one global terminal")
                    };
                    member(&values, &source, Some(&replacement));
                    for item in values.items() {
                        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    }
                }
            }
            assert_eq!(source, original);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {authored} !important;")
            );
        }
    }
}

macro_rules! longhand_lifecycle {
    ($module:ident, $name:literal, $good:literal, $bad:literal) => {
        mod $module {
            use super::*;
            #[test]
            fn metadata_is_noninherited_fixed_and_terminal() {
                metadata($name, false);
            }
            #[test]
            fn ordinary_expansion_preserves_occurrence() {
                ordinary($name, $good);
            }
            #[test]
            fn all_css_wide_keywords_stay_symbolic() {
                globals($name);
            }
            #[test]
            fn failed_residual_and_successful_reentry_remains_atomic_and_reusable() {
                pending($name, $good, $bad);
            }
        }
    };
}
longhand_lifecycle!(position, "offset-position", "top left", "none");
longhand_lifecycle!(
    path,
    "offset-path",
    "url('#p') view-box",
    "circle() margin-box"
);
longhand_lifecycle!(distance, "offset-distance", "calc(1px + 2%)", "1deg");
longhand_lifecycle!(rotate, "offset-rotate", "reverse .25turn", "auto reverse");
longhand_lifecycle!(anchor, "offset-anchor", "right 1px bottom 2%", "normal");

#[test]
fn shorthand_ordinary_expansion_keeps_five_occurrences() {
    ordinary("offset", "normal ray(90deg) 25% reverse / auto");
}
#[test]
fn shorthand_css_wide_keywords_contribute_all_five_symbolic_terminals() {
    globals("offset");
}
#[test]
fn shorthand_pending_reentry_is_whole_strict_reusable_and_preserves_all_sources() {
    pending(
        "offset",
        "normal url('#p') view-box 25% reverse 90deg / right top",
        "/ auto",
    );
}

#[test]
fn shorthand_metadata_has_five_settable_members_in_source_order_and_no_reset_only_members() {
    let meta = property("offset")
        .metadata()
        .expect("Motion shorthand metadata");
    let CssPropertyKindRef::Shorthand(short) = meta.kind() else {
        panic!("offset is shorthand")
    };
    assert_eq!(
        short
            .members()
            .iter()
            .map(|p| p.known_property())
            .collect::<Vec<_>>(),
        LONGHANDS.into_iter().map(property).collect::<Vec<_>>()
    );
    assert!(short.reset_only_members().is_empty());
}

#[test]
fn omitted_shorthand_members_still_contribute_all_five_ordinary_terminals_in_source_order() {
    // New Motion payload variants have no preimplementation callable borrowed
    // boundary. Independent normal/none/zero/auto/auto payload and provenance
    // assertions belong to functional new-variant tests with implementation.
    // Published CssLonghandValue exposes property/view but no generic serializer.
    for input in [
        "none",
        "25%",
        "ray(90deg) / bottom right",
        "auto url('#p') fill-box -25% reverse 90deg / left top",
    ] {
        for front in 0..5 {
            let source = declaration("offset", input, front);
            let values = completed(&source);
            member(&values, &source, None);
            for (value, name) in values.items().iter().zip(LONGHANDS) {
                assert_eq!(value.property(), property(name));
                assert_eq!(
                    value.ordinary_value().unwrap().property().known_property(),
                    property(name)
                );
            }
        }
    }
}

fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    let output = components.serialize().unwrap();
    (0..output.as_css().len())
        .find_map(|offset| match output.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original fixture has implicit closure")
}

fn strict_original(name: &str, ordinary: &str, unclosed: &str) {
    let p = property(name);
    let inputs = [
        format!("/*😀*/{unclosed}"),
        format!("{ordinary}/*unfinished"),
        "inherit/*unfinished".into(),
        "var(--motion".into(),
        "env(shape".into(),
        "attr(data-motion".into(),
        "var(--motion)/*unfinished".into(),
    ];
    for input in inputs {
        let components = parse_component_values(&input).unwrap();
        let original = components.clone();
        let origin = implicit_origin(&components);
        for grammar in [false, true] {
            let error = checked(p, components.clone(), grammar).unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ),
                "{input}: {error:?}"
            );
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(components, original);
    }
}

#[test]
fn every_motion_property_checks_original_function_comment_and_pending_closure() {
    for (name, ordinary, open) in [
        ("offset-position", "normal", "var(--position"),
        ("offset-path", "ray(90deg)", "ray(90deg"),
        ("offset-distance", "1px", "calc(1px + 2%"),
        ("offset-rotate", "reverse", "calc(90deg"),
        ("offset-anchor", "auto", "var(--anchor"),
        ("offset", "none", "ray(90deg"),
    ] {
        strict_original(name, ordinary, open);
    }
}

#[test]
fn pending_reentry_rejects_original_closure_after_residual_scan_and_keeps_handle_reusable() {
    for (name, valid, open) in [
        ("offset-position", "normal", "normal/*unfinished"),
        ("offset-path", "ray(90deg)", "ray(90deg"),
        ("offset-distance", "1px", "calc(1px + 2%"),
        ("offset-rotate", "reverse", "calc(90deg"),
        ("offset-anchor", "auto", "auto/*unfinished"),
        ("offset", "none", "url('#p'"),
    ] {
        let source = declaration(name, "var(--motion)", 0);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement = parse_component_values(open).unwrap();
        let origin = implicit_origin(&replacement);
        let error = handle.reenter(replacement).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
            panic!("closure failure")
        };
        assert!(matches!(
            actual.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ));
        assert_eq!(actual.origin(), &CssSerializedOrigin::End(Some(origin)));
        assert_eq!(
            handle
                .reenter(parse_component_values("future(var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let replacement = parse_component_values(valid).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("resolved terminals")
        };
        member(&values, &source, Some(&replacement));
        assert!(handle.source().same_occurrence(&source));
    }
}

#[test]
fn composed_normalization_keeps_six_declarations_and_ten_ordered_terminal_contributions() {
    let input = ".a{offset:normal ray(90deg) 25% reverse / auto!important;offset-position:normal;offset-path:url('#p') view-box;offset-distance:var(--distance);offset-rotate:reverse;offset-anchor:initial!important}";
    let report = parse_sheet(input);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style")
    };
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 6, 10).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let items: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(d) => Some(d),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 6);
    for (order, (item, source)) in items.iter().zip(style.declarations().iter()).enumerate() {
        assert_eq!(item.order(), order);
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                let expected = terminals(source.known().unwrap().property().canonical_name());
                assert_eq!(values.items().len(), expected.len());
                for (value, name) in values.items().iter().zip(expected) {
                    assert_eq!(value.property(), property(name));
                    assert!(value.source().same_occurrence(source));
                }
            }
            CssExpansion::Pending(handle) if order == 3 => {
                assert!(handle.source().same_occurrence(source))
            }
            other => panic!("authored occurrence: {other:?}"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(0, 1, 5, 10).unwrap(),
            CssNormalizationResource::Declarations,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 6, 9).unwrap(),
            CssNormalizationResource::Contributions,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        let limit = if resource == CssNormalizationResource::Declarations {
            5
        } else {
            9
        };
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
        assert_eq!(error.declaration_order(), Some(5));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(&style.declarations()[5])
        );
        assert!(error.rule_context().is_some());
        assert_eq!(report, before);
    }
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

#[test]
fn universal_reset_includes_five_motion_longhands_without_resolving_path_context() {
    let source = declaration("all", "initial", 0);
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("all")
    };
    for name in LONGHANDS {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property(name))));
    }
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Direction)));
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::UnicodeBidi)));
}

fn budget(name: &str, input: &str, value: &str, nodes: usize) {
    let output = format!("{name}: {value} !important;");
    for front in 0..5 {
        let source = declaration(name, input, front);
        let before = source.clone();
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(nodes, nodes, output.len()))
                .unwrap(),
            output
        );
        for (limits, kind) in [
            (L::new(nodes - 1, nodes, output.len()), K::InputNodeLimit),
            (
                L::new(nodes, nodes - 1, output.len()),
                K::ProjectionNodeLimit,
            ),
            (L::new(nodes, nodes, output.len() - 1), K::ByteLimit),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(source.to_specified_css().unwrap(), output);
    }
}

#[test]
fn keyword_distance_and_position_providers_share_exact_declaration_budgets() {
    budget("offset-position", "normal", "normal", 3);
    budget("offset-path", "none", "none", 3);
    budget("offset-distance", "-25%", "-25%", 3);
    budget("offset-anchor", "auto", "auto", 3);
    budget("offset-anchor", "left top", "left top", 5);
}

#[test]
fn ray_path_and_rotate_aggregate_counts_include_explicit_suppressed_size() {
    // Path aggregate1 + ray aggregate1 + angle1 + declaration2 =5.
    budget("offset-path", "ray(90deg)", "ray(90deg)", 5);
    budget("offset-path", "ray(90deg closest-side)", "ray(90deg)", 6);
    // Same plus contain1 and physical position aggregate/axes3 =9.
    budget(
        "offset-path",
        "ray(90deg contain at left top)",
        "ray(90deg contain at left top)",
        9,
    );
    // Rotate aggregate1 + provided modifier/angle leaves + declaration2.
    budget("offset-rotate", "auto", "auto", 4);
    budget("offset-rotate", "90deg", "90deg", 4);
    budget("offset-rotate", "auto 90deg", "auto 90deg", 5);
}

#[test]
fn path_box_url_and_shorthand_components_share_one_cumulative_budget() {
    // Path aggregate1 + URL provider2 + coord-box1 + declaration2 =6.
    budget(
        "offset-path",
        "src('#p') view-box",
        "src(\"#p\") view-box",
        6,
    );
    // Offset aggregate1 + position1 + path-none1 + distance1 + rotate2 + anchor1 + declaration2 =9.
    budget(
        "offset",
        "normal none 0 auto / auto",
        "normal none 0 auto / auto",
        9,
    );
    budget("offset", "normal", "normal", 4);
}

#[test]
fn nested_shape_numeric_children_retain_shared_work_and_byte_failure_atomicity() {
    let input = "ellipse(abs(-2em) hypot(3em,4em) at 1px 2%) view-box";
    let value = "ellipse(calc(2em) calc(5em) at 1px 2%) view-box";
    // Accepted child ellipse count12 + path aggregate1 + box1 + declaration2 =16.
    budget("offset-path", input, value, 16);
    for name in ["offset-distance", "offset-position"] {
        let authored = if name == "offset-distance" {
            "calc(1px + 2%)"
        } else {
            "calc(1px + 2%) center"
        };
        let expected = if name == "offset-distance" {
            "calc(2% + 1px)"
        } else {
            "calc(2% + 1px) center"
        };
        let source = declaration(name, authored, 0);
        let before = source.clone();
        let output = format!("{name}: {expected} !important;");
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(128, 256, output.len()))
                .unwrap(),
            output
        );
        for (limits, kind) in [
            (L::new(0, 256, output.len()), K::InputNodeLimit),
            (L::new(128, 0, output.len()), K::ProjectionNodeLimit),
            (L::new(128, 256, output.len() - 1), K::ByteLimit),
        ] {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(source, before);
        }
    }
}

#[test]
fn accepted_shape_position_url_and_angle_provider_controls_have_independent_goldens() {
    accept(
        "clip-path",
        &[
            ("circle(calc(1in + 1in))", "circle(calc(192px))"),
            (
                "path(evenodd,'M01.00,0 L1e2 -0 Z')",
                "path(evenodd, \"M01.00,0 L1e2 -0 Z\")",
            ),
        ],
    );
    accept(
        "object-position",
        &[
            ("top", "center top"),
            ("bottom 2% right 1px", "right 1px bottom 2%"),
        ],
    );
    accept(
        "background-image",
        &[("src('#p' CORS)", "src(\"#p\" CORS)")],
    );
    accept(
        "rotate",
        &[
            (".25turn", "0.25turn"),
            ("calc(1turn - 90deg)", "calc(270deg)"),
        ],
    );
}

#[test]
fn valid_attributes_and_structurally_deep_pending_motion_values_keep_whole_value_reentry() {
    let text = format!("{}var(--motion){}", "f(".repeat(255), ")".repeat(255));
    let components = parse_component_values(&text).expect("shared depth256 ceiling");
    assert_eq!(components.nesting_depth(), 256);
    for name in NAMES {
        let p = property(name);
        for grammar in [false, true] {
            let source = checked(p, components.clone(), grammar).unwrap();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("deep whole pending")
            };
            assert_eq!(
                handle.reenter(components.clone()).unwrap_err().kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let valid = match name {
                "offset-path" | "offset" => "ray(90deg)",
                "offset-distance" => "25%",
                "offset-rotate" => "reverse",
                _ => "auto",
            };
            let replacement = parse_component_values(valid).unwrap();
            let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("resolved motion terminals")
            };
            member(&items, &source, Some(&replacement));
        }
        let source = declaration(name, "attr(data-motion)", 0);
        assert!(matches!(
            expand_declaration(&source).unwrap(),
            CssExpansion::Pending(_)
        ));
    }
    let excess = format!("f({text})");
    let error = parse_component_values(&excess).unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("first excess opening retains original source")
    };
    assert_eq!(origin.source().as_str(), excess);
    assert_eq!(origin.span().start().byte_offset().value(), 512);
    assert_eq!(origin.span().end().byte_offset().value(), 516);
}
