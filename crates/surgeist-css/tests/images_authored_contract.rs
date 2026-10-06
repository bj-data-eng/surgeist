#![forbid(unsafe_code)]
//! Existing-callable authored Images 3 contracts for object-position,
//! image-orientation, image-rendering and object-fit. Authority: selected Images 3
//! CRD 2023-12-18 §§4.5–5.2 (including deprecated UA-accepted rendering syntax),
//! imported Values 3 §7.3 physical position grammar, and Surgeist's public atomic
//! cumulative specified-output / intrinsic expansion contracts. No downstream
//! fit, orientation rounding, image loading or painting is performed here.
//! New terminal borrowed variants and new rendering variants require separate
//! functional tests alongside implementation; this draft names none of them.

#[path = "common/authored_property.rs"]
mod authored_property;
#[path = "common/property_expectations.rs"]
mod property_expectations;

use authored_property::{ParserFront, assert_source, checked, invalid};
use surgeist_css::{
    CssKnownProperty as P, CssSpecifiedValueSerializationErrorKind as K,
    CssSpecifiedValueSerializationLimits as L, *,
};

const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("inherit", CssGlobalKeyword::Inherit),
    ("initial", CssGlobalKeyword::Initial),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

// Specialized grammar/output stimuli, not another property inventory.
const POSITIONS: &[(&str, &str)] = &[
    ("left", "left center"),
    ("right", "right center"),
    ("top", "center top"),
    ("bottom", "center bottom"),
    ("center", "center center"),
    ("25%", "25% center"),
    ("0", "0 center"),
    ("-1px", "-1px center"),
    ("bottom right", "right bottom"),
    ("center left", "left center"),
    ("left top", "left top"),
    ("10% 20px", "10% 20px"),
    ("left 10px", "left 10px"),
    ("10px bottom", "10px bottom"),
    ("bottom -2% right -1px", "right -1px bottom -2%"),
    ("left 0px top 0%", "left 0px top 0%"),
    ("calc(1px + 5%) center", "calc(5% + 1px) center"),
    (r"r\69 ght b\6f ttom", "right bottom"),
];
const ORIENTATIONS: &[(&str, &str)] = &[
    ("from-image", "from-image"),
    ("NONE", "none"),
    ("flip", "flip"),
    ("0deg", "0deg"),
    ("0deg flip", "0deg flip"),
    ("flip -0.25turn", "-0.25turn flip"),
    ("3.000e1deg", "30deg"),
    ("3.000e1deg flip", "30deg flip"),
    ("flip 3.000e1deg", "30deg flip"),
    ("1rad", "1rad"),
    ("flip 1e2grad", "100grad flip"),
    ("flip calc(15deg + 15deg)", "calc(30deg) flip"),
    (r"f\6c ip 30deg", "30deg flip"),
];
const RENDERING: &[(&str, &str)] = &[
    ("AUTO", "auto"),
    ("smooth", "smooth"),
    ("HIGH-QUALITY", "high-quality"),
    ("pixelated", "pixelated"),
    ("CRISP-EDGES", "crisp-edges"),
    ("optimizeSpeed", "optimizespeed"),
    ("OPTIMIZEQUALITY", "optimizequality"),
    (r"s\6d ooth", "smooth"),
    (r"h\69 gh-quality", "high-quality"),
    (r"optimize\53 peed", "optimizespeed"),
    (r"optimize\51 uality", "optimizequality"),
];
const FITS: &[(&str, &str)] = &[
    ("FILL", "fill"),
    ("contain", "contain"),
    ("cover", "cover"),
    ("none", "none"),
    ("scale-down", "scale-down"),
    (r"c\6f ver", "cover"),
    (r"scale-\64 own", "scale-down"),
];

fn owned(property: P) -> bool {
    matches!(
        property,
        P::ObjectPosition | P::ImageOrientation | P::ImageRendering | P::ObjectFit
    )
}
fn cases() -> impl Iterator<Item = &'static property_expectations::PropertyExpectation> {
    property_expectations::CASES
        .iter()
        .filter(|row| owned(row.property))
}
fn parsed(property: P, value: &str) -> CssDeclaration {
    ParserFront::StyleAttribute.parse(property, value)
}
fn assert_wrapper(source: &CssDeclaration, input: &str) {
    let value = source.known().unwrap().property_value().unwrap();
    match (source.known().unwrap().property(), value) {
        (P::ObjectPosition, CssKnownPropertyValueRef::ObjectPosition(v)) => {
            assert_eq!(v.as_css(), input)
        }
        (P::ImageOrientation, CssKnownPropertyValueRef::ImageOrientation(v)) => {
            assert_eq!(v.as_css(), input)
        }
        (P::ImageRendering, CssKnownPropertyValueRef::ImageRendering(v)) => {
            assert_eq!(v.as_css(), input)
        }
        (P::ObjectFit, CssKnownPropertyValueRef::ObjectFit(v)) => assert_eq!(v.as_css(), input),
        _ => panic!("property-specific ordinary wrapper: {value:?}"),
    }
}
fn accept(property: P, values: &[(&str, &str)]) {
    for &(input, expected) in values {
        for front in FRONTS {
            let source = front.valid(property, input, expected);
            assert_wrapper(&source, input);
            let output = source.to_specified_css().unwrap();
            let report = parse_style_attribute(&output);
            assert!(report.is_clean(), "{output}: {:?}", report.diagnostics());
            let [reparsed] = report.syntax().as_slice() else {
                panic!("one output occurrence")
            };
            assert_eq!(reparsed.known().unwrap().property(), property);
            assert_eq!(reparsed.importance(), CssImportance::Important);
            assert_eq!(reparsed.to_specified_css().unwrap(), output);
        }
    }
}
#[test]
fn physical_position_grammar_emits_explicit_horizontal_then_vertical_axes() {
    accept(P::ObjectPosition, POSITIONS);
}
#[test]
fn authored_orientation_keeps_omitted_and_explicit_angle_before_flip() {
    accept(P::ImageOrientation, ORIENTATIONS);
}
#[test]
fn every_standard_and_deprecated_rendering_keyword_keeps_its_specified_spelling() {
    accept(P::ImageRendering, RENDERING);
}
#[test]
fn every_fit_keyword_has_its_own_ordinary_wrapper_and_canonical_text() {
    accept(P::ObjectFit, FITS);
}
#[test]
fn smooth_and_high_quality_are_admitted_by_every_existing_front() {
    accept(
        P::ImageRendering,
        &[("smooth", "smooth"), ("high-quality", "high-quality")],
    );
}
#[test]
fn deprecated_ua_rendering_keywords_are_admitted_case_insensitively_without_alias_collapse() {
    accept(
        P::ImageRendering,
        &[
            ("OptimizeSpeed", "optimizespeed"),
            ("OptimizeQuality", "optimizequality"),
        ],
    );
}

#[test]
fn malformed_positions_drop_only_their_declaration_and_have_typed_checked_failures() {
    for value in [
        "left right",
        "top bottom",
        "left 1px top",
        "left top 1px",
        "top 1px",
        "left,top",
        "center 1px top 2px",
        "left 1px top 2px right",
        "x-start y-end",
        "block-start inline-end",
        "start end",
        "10deg",
        "calc(1deg)",
        "inherit left",
    ] {
        invalid(P::ObjectPosition, value);
    }
}
#[test]
fn duplicate_and_exclusive_orientation_components_remain_strict() {
    for value in [
        "none flip",
        "from-image flip",
        "from-image none",
        "none 30deg",
        "flip flip",
        "30deg 60deg",
        "flip 30deg flip",
        "30px",
        "30%",
        "1",
        "inherit flip",
    ] {
        invalid(P::ImageOrientation, value);
    }
}
#[test]
fn rendering_rejects_unknown_or_multiple_keywords_and_nonkeyword_values() {
    for value in [
        "nearest-neighbor",
        "smooth high-quality",
        "auto pixelated",
        "optimizeSpeed smooth",
        "auto, smooth",
        "1",
        "calc(1)",
        "inherit auto",
    ] {
        invalid(P::ImageRendering, value);
    }
}
#[test]
fn fitting_rejects_other_image_keywords_and_multiple_alternatives() {
    for value in [
        "auto",
        "smooth",
        "contain cover",
        "fill none",
        "none,cover",
        "1px",
        "calc(1)",
        "inherit fill",
    ] {
        invalid(P::ObjectFit, value);
    }
}

fn point(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value() as usize,
        source[..offset].encode_utf16().count()
    );
}
fn origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(value) = origin else {
        panic!("parsed authored child")
    };
    assert_eq!(value.source().as_str(), source);
    let start = source.find(token).unwrap();
    point(value.span().start(), source, start);
    point(value.span().end(), source, start + token.len());
}
#[test]
fn physical_position_and_orientation_keep_exact_scalar_bytes_utf16_and_semantic_units() {
    for property in [P::ObjectPosition, P::ImageOrientation] {
        let input = if property == P::ObjectPosition {
            "bottom -2.00% right -1e-2px"
        } else {
            "flip -3.000e1grad"
        };
        for source in [
            parsed(property, input),
            checked(property, input, false),
            checked(property, input, true),
        ] {
            let before = source.clone();
            let raw = if source.parsed_name().is_some() {
                source.parsed_name().unwrap().source().as_str()
            } else {
                input
            };
            match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::ObjectPosition(value) => {
                    let (
                        CssHorizontalPosition::RightOffset(x),
                        CssVerticalPosition::BottomOffset(y),
                    ) = (value.position().horizontal(), value.position().vertical())
                    else {
                        panic!("edge axes")
                    };
                    origin(x.origin(), raw, "-1e-2px");
                    origin(y.origin(), raw, "-2.00%");
                    assert!(
                        matches!(x.literal_component().unwrap().view(), CssComponentValueRef::Token(
                        CssValueTokenRef::Dimension { number, unit }) if number.representation() == "-1e-2" && unit == "px")
                    );
                    assert!(
                        matches!(y.literal_component().unwrap().view(), CssComponentValueRef::Token(
                        CssValueTokenRef::Percentage(number)) if number.representation() == "-2.00")
                    );
                    assert_eq!(
                        value.position().serialize_specified().unwrap(),
                        "right -0.01px bottom -2%"
                    );
                }
                CssKnownPropertyValueRef::ImageOrientation(value) => {
                    let CssImageOrientation::Flip(Some(angle)) = value.orientation() else {
                        panic!("angle + flip")
                    };
                    origin(angle.origin(), raw, "-3.000e1grad");
                    let literal = angle.literal().unwrap();
                    assert_eq!(literal.numeric().representation(), "-3.000e1");
                    assert_eq!(literal.unit(), CssAngleUnit::Gradians);
                    assert_eq!(
                        value.orientation().serialize_specified().unwrap(),
                        "-30grad flip"
                    );
                }
                _ => panic!("selected numeric image role"),
            }
            assert_eq!(source, before);
        }
    }
}
#[test]
fn keyword_component_origins_preserve_decoded_escape_identity_and_source_spans() {
    for (property, input, token) in [
        (P::ImageRendering, r"p\69 xelated", r"p\69 xelated"),
        (P::ObjectFit, r"c\6f ver", r"c\6f ver"),
        (P::ImageOrientation, r"n\6f ne", r"n\6f ne"),
        (P::ObjectPosition, r"r\69 ght", r"r\69 ght"),
    ] {
        let source = parsed(property, input);
        let raw = source.parsed_name().unwrap().source().as_str();
        let component = source
            .value_components()
            .items()
            .iter()
            .find(|c| {
                matches!(
                    c.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Ident(_))
                )
            })
            .unwrap();
        origin(component.origin(), raw, token);
        let start = raw.find(':').unwrap() - property.canonical_name().len();
        point(source.position().unwrap(), raw, start);
        let before = source.clone();
        source.to_specified_css().unwrap();
        assert_eq!(source, before);
    }
}
#[test]
fn invalid_keyword_occurrences_keep_exact_recovery_span_and_adjacent_importance() {
    for row in cases() {
        let declaration = format!("{}: bogus !important;", row.name);
        let css = format!("/*😀*/color:red!important;{declaration}color:blue");
        let report = parse_style_attribute(&css);
        let [before, after] = report.syntax().as_slice() else {
            panic!("two neighbors")
        };
        assert_eq!(before.importance(), CssImportance::Important);
        assert_eq!(after.importance(), CssImportance::Normal);
        let [diagnostic] = report.diagnostics() else {
            panic!("one grammar diagnostic")
        };
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("typed grammar failure")
        };
        assert_eq!(detail.property(), row.property);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let start = css.find(&declaration).unwrap();
        point(diagnostic.span().start(), &css, start);
        point(diagnostic.span().end(), &css, start + declaration.len());
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

fn metadata(property: P) {
    for row in cases().filter(|row| row.property == property) {
        let metadata = row
            .property
            .metadata()
            .expect("Images role has intrinsic metadata");
        assert_eq!(metadata.grammar(), row.property.grammar());
        let CssPropertyKindRef::Longhand(value) = metadata.kind() else {
            panic!("Images role is a longhand")
        };
        assert_eq!(value.property().known_property(), row.property);
        assert_eq!(
            value.inherited_by_default(),
            matches!(row.property, P::ImageOrientation | P::ImageRendering)
        );
        let initial = value.initial_value();
        assert_eq!(initial.property().known_property(), row.property);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("intrinsic fixed initial, not UA context")
        };
        assert_eq!(initial.property().known_property(), row.property);
        // Exact initial payloads are added to the authoritative common records
        // when their currently absent borrowed terminal variants exist.
    }
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).expect("intrinsic Images expansion")
    else {
        panic!("completed terminal contribution")
    };
    items
}
fn member(
    items: &CssLonghandContributions,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    let [item] = items.items() else {
        panic!("one contribution, never shorthand projection")
    };
    let property = source.known().unwrap().property();
    assert_eq!(item.property(), property);
    assert_source(item, source);
    assert_eq!(item.replacement_components(), replacement);
}
fn ordinary(property: P, input: &str) {
    for front in FRONTS {
        let source = front.parse(property, input);
        let before = source.clone();
        let values = completed(&source);
        member(&values, &source, None);
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            property
        );
        assert_eq!(source, before);
    }
}
fn globals(property: P) {
    for (input, keyword) in GLOBALS {
        for front in FRONTS {
            let source = front.valid(property, input, input);
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            let values = completed(&source);
            member(&values, &source, None);
            assert_eq!(
                values.items()[0].value(),
                CssContributionValueRef::Global(keyword)
            );
            assert!(values.items()[0].ordinary_value().is_none());
        }
    }
}
fn pending(property: P, valid: &str, expected: &str, rejected: &str) {
    for authored in [
        "bogus var(--image)",
        "bogus env(image, x)",
        "bogus attr(data-image)",
    ] {
        for front in FRONTS {
            let source = front.parse(property, authored);
            let before = source.clone();
            let expected_pending = format!("{}: {authored} !important;", property.canonical_name());
            assert_eq!(source.to_specified_css().unwrap(), expected_pending);
            assert_eq!(
                source
                    .to_specified_css_with_limits(L::new(
                        usize::MAX,
                        usize::MAX,
                        expected_pending.len()
                    ))
                    .unwrap(),
                expected_pending
            );
            assert_eq!(
                source
                    .to_specified_css_with_limits(L::new(
                        usize::MAX,
                        usize::MAX,
                        expected_pending.len() - 1
                    ))
                    .unwrap_err()
                    .kind(),
                K::ByteLimit
            );
            assert_eq!(source, before);
            assert!(source.known().unwrap().substitution_dependent().is_some());
            assert!(source.known().unwrap().property_value().is_none());
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole-value pending")
            };
            assert!(handle.source().same_occurrence(&source));
            for _ in 0..2 {
                for invalid_value in [rejected, "inherit bogus", "bogus"] {
                    let components = parse_component_values(invalid_value).unwrap();
                    let direct = parse_property_value(
                        CssPropertyNameRef::Known(property),
                        components.clone(),
                        CssImportance::Normal,
                    )
                    .unwrap_err();
                    let error = handle.reenter(components).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                        panic!("typed strict grammar reentry")
                    };
                    assert_eq!(actual.kind(), direct.kind());
                    assert_eq!(actual.origin(), direct.origin());
                }
                for residual in [
                    "bogus var(--again)",
                    "[f(env(image))]",
                    "f(attr(data-image))",
                    r"f(v\61 r(--again))",
                ] {
                    assert_eq!(
                        handle
                            .reenter(parse_component_values(residual).unwrap())
                            .unwrap_err()
                            .kind(),
                        &CssExpansionErrorKind::ResidualSubstitution,
                        "residual must win over invalid grammar"
                    );
                }
                let replacement = parse_component_values(&format!("/*😀*/{valid}")).unwrap();
                let old_replacement = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("completed reentry")
                };
                member(&values, &source, Some(&replacement));
                assert!(values.items()[0].ordinary_value().is_some());
                assert_eq!(replacement, old_replacement);
                let direct = checked(property, valid, false);
                assert_eq!(
                    direct.to_specified_css().unwrap(),
                    format!("{}: {expected} !important;", property.canonical_name())
                );
                for (global, keyword) in GLOBALS {
                    let replacement = parse_component_values(global).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("global reentry")
                    };
                    member(&values, &source, Some(&replacement));
                    assert_eq!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(keyword)
                    );
                }
                assert_eq!(source, before);
                assert!(handle.source().same_occurrence(&source));
            }
        }
    }
}
fn normalization(property: P, ordinary: &str) {
    let css = format!(
        ".image{{{}:{ordinary}!important;{}:var(--v);{}:unset}}",
        property.canonical_name(),
        property.canonical_name(),
        property.canonical_name()
    );
    let report = parse_sheet(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let before = report.clone();
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style owner")
    };
    let exact = CssNormalizationLimits::try_new(0, 1, 3, 3).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let [
        CssNormalizedItem::Rule(rule),
        CssNormalizedItem::Declaration(first),
        CssNormalizedItem::Declaration(second),
        CssNormalizedItem::Declaration(third),
    ] = normalized.items()
    else {
        panic!("one rule and three ordered occurrences")
    };
    assert_eq!([first.order(), second.order(), third.order()], [0, 1, 2]);
    for (item, source) in [first, second, third]
        .into_iter()
        .zip(style.declarations().iter())
    {
        assert!(item.source().same_occurrence(source));
        assert!(item.rule_context().same_context(rule));
    }
    assert_eq!(first.source().importance(), CssImportance::Important);
    assert_eq!(second.source().importance(), CssImportance::Normal);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = first.expansion() else {
        panic!("ordinary")
    };
    member(values, first.source(), None);
    assert!(values.items()[0].ordinary_value().is_some());
    let CssExpansion::Pending(handle) = second.expansion() else {
        panic!("middle stays pending")
    };
    assert!(handle.source().same_occurrence(second.source()));
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = third.expansion() else {
        panic!("symbolic global")
    };
    member(values, third.source(), None);
    assert_eq!(
        values.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
    );
    for (limit, resource) in [
        (
            CssNormalizationLimits::try_new(0, 1, 3, 2).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 2, 3).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limit).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 2 }
            );
            assert_eq!(error.declaration_order(), Some(2));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(&style.declarations()[2])
            );
            assert!(error.rule_context().is_some());
            assert_eq!(report, before);
        }
    }
    assert!(normalize_report_with_limits(&report, exact).is_ok());
    assert_eq!(report, before);
}
macro_rules! lifecycle {
    ($module:ident, $property:ident, $input:literal, $expected:literal, $invalid:literal) => {
        mod $module { use super::*;
            #[test] fn metadata_is_fixed_terminal_and_has_selected_inheritance() { metadata(P::$property); }
            #[test] fn original_ordinary_and_global_unclosed_comments_are_rejected_at_original_eof() { original_comment_closures(P::$property, $input, false); }
            #[test] fn original_pending_unclosed_comments_are_rejected_before_pending_admission() { original_comment_closures(P::$property, $input, true); }
            #[test] fn original_pending_functions_cannot_be_repaired_before_checked_grammar() { original_pending_function_closures(P::$property); }
            #[test] fn closed_comments_globals_and_pending_functions_remain_checked_controls_without_metadata() { closed_component_controls(P::$property, $input); }
            #[test] fn browser_eof_recovery_keeps_occurrences_origins_and_diagnostics_through_normalization() { recovered_images_report(P::$property, $input); }
            #[test] fn strict_replacement_rejects_original_closure_and_residuals_win_before_atomic_retry() { closed_reentry(P::$property, $input); }
            #[test] fn ordinary_is_one_terminal_with_original_occurrence_and_importance() { ordinary(P::$property, $input); }
            #[test] fn global_keywords_remain_symbolic_and_preserve_occurrence() { globals(P::$property); }
            #[test] fn pending_whole_value_reentry_is_residual_first_atomic_and_repeatable() { pending(P::$property, $input, $expected, $invalid); }
            #[test] fn normalization_retains_order_importance_and_pending_with_exact_budgets() { normalization(P::$property, $input); }
        }
    };
}
lifecycle!(
    position,
    ObjectPosition,
    "bottom -2% right -1px",
    "right -1px bottom -2%",
    "left 1px top"
);
lifecycle!(
    orientation,
    ImageOrientation,
    "flip 30deg",
    "30deg flip",
    "none flip"
);
lifecycle!(
    rendering,
    ImageRendering,
    "pixelated",
    "pixelated",
    "auto smooth"
);
lifecycle!(fit, ObjectFit, "scale-down", "scale-down", "fill cover");
#[test]
fn newly_admitted_rendering_keywords_also_cross_ordinary_and_pending_lifecycle() {
    for (input, expected) in [
        ("SMOOTH", "smooth"),
        ("HIGH-QUALITY", "high-quality"),
        ("OptimizeSpeed", "optimizespeed"),
        ("OptimizeQuality", "optimizequality"),
    ] {
        ordinary(P::ImageRendering, input);
        pending(P::ImageRendering, input, expected, "smooth high-quality");
    }
}
#[test]
fn universal_reset_retains_all_images_roles_as_symbolic_targets() {
    for (input, keyword) in GLOBALS {
        let source = parsed(P::All, input);
        let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("all symbolic reset")
        };
        assert_eq!(reset.keyword(), keyword);
        assert!(reset.source().same_occurrence(&source));
        for row in cases() {
            assert!(!reset.excludes(CssPropertyNameRef::Known(row.property)));
        }
    }
}
fn declaration_budget(property: P, input: &str, output: &str, inputs: usize, projections: usize) {
    for source in [parsed(property, input), checked(property, input, true)] {
        let expected = format!("{}: {output} !important;", property.canonical_name());
        let exact = L::new(inputs + 2, projections + 2, expected.len());
        let before = source.clone();
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                L::new(inputs + 1, projections + 2, expected.len()),
                K::InputNodeLimit,
            ),
            (
                L::new(inputs + 2, projections + 1, expected.len()),
                K::ProjectionNodeLimit,
            ),
            (
                L::new(inputs + 2, projections + 2, expected.len() - 1),
                K::ByteLimit,
            ),
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
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
}
#[test]
fn every_rendering_and_fit_keyword_charges_one_semantic_node_plus_declaration_and_name() {
    for (input, expected) in RENDERING {
        declaration_budget(P::ImageRendering, input, expected, 1, 1);
    }
    for (input, expected) in FITS {
        declaration_budget(P::ObjectFit, input, expected, 1, 1);
    }
}
#[test]
fn orientation_children_share_declaration_work_and_preserve_explicit_zero_and_angle_units() {
    for (input, output, inputs, projections) in [
        ("none", "none", 1, 1),
        ("from-image", "from-image", 1, 1),
        ("flip", "flip", 1, 1),
        ("0deg flip", "0deg flip", 2, 2),
        ("flip -0.25turn", "-0.25turn flip", 2, 2),
    ] {
        declaration_budget(P::ImageOrientation, input, output, inputs, projections);
    }
}
#[test]
fn physical_position_children_share_one_declaration_budget_across_both_math_arenas() {
    for (input, output, inputs, projections) in [
        ("top", "center top", 3, 3),
        ("25%", "25% center", 4, 4),
        ("bottom 2% right 1px", "right 1px bottom 2%", 5, 5),
        (
            "calc(1px + 2em) calc(1px + 2em)",
            "calc(2em + 1px) calc(2em + 1px)",
            11,
            13,
        ),
    ] {
        declaration_budget(P::ObjectPosition, input, output, inputs, projections);
    }
}
#[test]
fn global_declarations_charge_one_global_node_without_materializing_initials() {
    for row in cases() {
        for (input, _) in GLOBALS {
            declaration_budget(row.property, input, input, 1, 1);
        }
    }
}
#[test]
fn sibling_image_roles_share_sheet_work_and_final_output_bytes_atomically() {
    let report = parse_sheet(concat!(
        ".a{object-position:bottom 2% right 1px!important;image-orientation:flip 30deg}",
        ".b{image-rendering:pixelated;object-fit:scale-down}"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { object-position: right 1px bottom 2% !important; image-orientation: 30deg flip; }\n.b { image-rendering: pixelated; object-fit: scale-down; }";
    // Sheet 1. Each simple class style rule: rule 1, selector list + class 2, declaration list 1.
    // Four declarations/names 8; physical position 5; angle + flip 2; two keywords 2.
    // Total 1 + 2*(1+2+1) + 8 + 5 + 2 + 2 = 26 semantic nodes in both work budgets.
    let exact = L::new(26, 26, expected.len());
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    for (limits, kind) in [
        (L::new(25, 26, expected.len()), K::InputNodeLimit),
        (L::new(26, 25, expected.len()), K::ProjectionNodeLimit),
        (L::new(26, 26, expected.len() - 1), K::ByteLimit),
    ] {
        assert!(
            sheet
                .rules()
                .iter()
                .all(|rule| rule.to_specified_css_with_limits(limits).is_ok())
        );
        for _ in 0..2 {
            let error = sheet.to_specified_css_with_limits(limits).unwrap_err();
            assert_eq!(
                error.kind(),
                CssSpecifiedRuleSerializationErrorKind::Resource(kind)
            );
            assert_eq!(error.rule_index(), Some(1));
            assert_eq!(sheet, &before);
        }
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}

fn lp(text: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
        .unwrap()
}
fn primitive_budget(
    expected: &str,
    inputs: usize,
    projections: usize,
    emit: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    let exact = L::new(inputs, projections, expected.len());
    assert_eq!(emit(exact).unwrap(), expected);
    for (limits, kind) in [
        (
            L::new(inputs - 1, projections, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(inputs, projections - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (
            L::new(inputs, projections, expected.len() - 1),
            K::ByteLimit,
        ),
    ] {
        for _ in 0..2 {
            assert_eq!(emit(limits).unwrap_err().kind(), kind);
        }
    }
    assert_eq!(emit(exact).unwrap(), expected);
}
#[test]
fn existing_typed_keyword_and_orientation_constructors_emit_independent_canonical_values() {
    for (value, expected) in [
        (CssImageRendering::Auto, "auto"),
        (CssImageRendering::Pixelated, "pixelated"),
        (CssImageRendering::CrispEdges, "crisp-edges"),
    ] {
        primitive_budget(expected, 1, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
    for (value, expected) in [
        (CssObjectFit::Fill, "fill"),
        (CssObjectFit::Contain, "contain"),
        (CssObjectFit::Cover, "cover"),
        (CssObjectFit::None, "none"),
        (CssObjectFit::ScaleDown, "scale-down"),
    ] {
        primitive_budget(expected, 1, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
    for (value, expected, nodes) in [
        (CssImageOrientation::FromImage, "from-image", 1),
        (CssImageOrientation::None, "none", 1),
        (CssImageOrientation::Flip(None), "flip", 1),
        (
            CssImageOrientation::Flip(Some(CssAngleValue::from_literal(
                CssAngleLiteral::try_new("-0.25", CssAngleUnit::Turns).unwrap(),
            ))),
            "-0.25turn flip",
            2,
        ),
    ] {
        let before = value.clone();
        primitive_budget(expected, nodes, nodes, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        assert_eq!(value, before);
    }
}
#[test]
fn physical_position_checked_construction_rejects_unpaired_and_logical_edges_and_keeps_programmatic_origins()
 {
    assert_eq!(
        CssPhysicalPosition::try_new(
            CssHorizontalPosition::RightOffset(lp("1px")),
            CssVerticalPosition::Top
        )
        .unwrap_err(),
        CssPositionConstructionError::UnpairedEdgeOffsets
    );
    assert_eq!(
        CssPhysicalPosition::try_new(CssHorizontalPosition::XStart, CssVerticalPosition::Top)
            .unwrap_err(),
        CssPositionConstructionError::NonPhysicalKeyword
    );
    assert_eq!(
        CssPhysicalPosition::try_new(CssHorizontalPosition::Left, CssVerticalPosition::YEnd)
            .unwrap_err(),
        CssPositionConstructionError::NonPhysicalKeyword
    );
    let position = CssPhysicalPosition::try_new(
        CssHorizontalPosition::RightOffset(lp("-1px")),
        CssVerticalPosition::BottomOffset(lp("-2%")),
    )
    .unwrap();
    let before = position.clone();
    primitive_budget("right -1px bottom -2%", 5, 5, |limits| {
        position.serialize_specified_with_limits(limits)
    });
    let (CssHorizontalPosition::RightOffset(x), CssVerticalPosition::BottomOffset(y)) =
        (position.horizontal(), position.vertical())
    else {
        panic!("paired offsets")
    };
    assert_eq!(x.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(y.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(position, before);
}
#[test]
fn programmatic_checked_property_components_keep_origins_without_declaration_coordinates() {
    for (property, tokens, expected) in [
        (
            P::ObjectPosition,
            vec!["right", "-1px", "bottom", "-2%"],
            "right -1px bottom -2%",
        ),
        (
            P::ImageOrientation,
            vec!["flip", "-0.25turn"],
            "-0.25turn flip",
        ),
        (P::ImageRendering, vec!["pixelated"], "pixelated"),
        (P::ObjectFit, vec!["scale-down"], "scale-down"),
    ] {
        let components = CssComponentValues::try_new(
            tokens
                .into_iter()
                .map(|t| CssComponentValue::try_token(t).unwrap())
                .collect(),
        )
        .unwrap();
        let before = components.clone();
        for grammar in [false, true] {
            let source = authored_property::checked_components(
                property,
                components.clone(),
                grammar,
                CssImportance::Important,
            )
            .unwrap();
            assert_eq!(source.value_components(), &components);
            assert!(source.position().is_none());
            assert!(source.parsed_name().is_none());
            assert!(source.parsed_value().is_none());
            assert!(
                source
                    .value_components()
                    .items()
                    .iter()
                    .all(|item| item.origin() == &CssValueOrigin::Programmatic)
            );
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{}: {expected} !important;", property.canonical_name())
            );
        }
        assert_eq!(components, before);
    }
}

#[test]
fn normalization_composes_all_four_image_roles_without_reordering_or_resolving_occurrences() {
    let report = parse_sheet(concat!(
        ".image{object-position:right bottom!important;image-orientation:flip 30deg;",
        "image-rendering:pixelated!important;object-fit:scale-down;object-position:var(--p);",
        "image-orientation:env(orientation);image-rendering:attr(data-rendering);object-fit:unset}"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("image style")
    };
    let exact = CssNormalizationLimits::try_new(0, 1, 8, 8).unwrap();
    let result = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = result
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 8);
    for (order, (normalized, source)) in declarations
        .iter()
        .zip(style.declarations().iter())
        .enumerate()
    {
        assert_eq!(normalized.order(), order);
        assert!(normalized.source().same_occurrence(source));
        assert_eq!(normalized.source().importance(), source.importance());
        match (order, normalized.expansion()) {
            (0..=3 | 7, CssExpansion::Contributions(CssContributions::Longhands(values))) => {
                member(values, source, None)
            }
            (4..=6, CssExpansion::Pending(handle)) => {
                assert!(handle.source().same_occurrence(source))
            }
            _ => panic!("ordinary, pending and global occurrences in exact order"),
        }
    }
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 8, 7).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 7
        }
    );
    assert_eq!(error.declaration_order(), Some(7));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(&style.declarations()[7])
    );
    assert_eq!(report, before);
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

// Checked components must report their original EOF, even when the browser-style
// component serializer could emit the missing delimiter. These helpers execute
// without metadata so absent expansion support cannot conceal closure RED.
const PENDING_CLOSED: [&str; 3] = ["var(--image)", "env(image)", "attr(data-image)"];
const PENDING_OPEN: [&str; 3] = ["var(--image", "env(image", "attr(data-image"];
fn images_implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for item in components.items() {
        let closing = match item.view() {
            CssComponentValueRef::Function(value) => Some(value.closing_origin()),
            CssComponentValueRef::Block(value) => Some(value.closing_origin()),
            _ => None,
        };
        if let Some(value @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return value.clone();
        }
    }
    let output = components.serialize().unwrap();
    (0..output.as_css().len())
        .find_map(|offset| match output.origin_at(offset) {
            Some(CssSerializedOrigin::Token(value @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(value.clone())
            }
            _ => None,
        })
        .expect("fixture contains an original implicit closure")
}
fn original_closure_coordinates(value: &CssValueOrigin, source: &str) {
    let CssValueOrigin::ImplicitClosure { opening, at } = value else {
        panic!("original implicit closure")
    };
    assert_eq!(opening.source().as_str(), source);
    assert_eq!(at.source().as_str(), source);
    assert!(opening.source().same_snapshot(at.source()));
    let comment = source.rfind("/*unfinished");
    let (start, end) = if let Some(start) = comment {
        (start, source.len())
    } else {
        let start = source
            .rfind("var(")
            .or_else(|| source.rfind("env("))
            .or_else(|| source.rfind("attr("))
            .or_else(|| source.rfind("calc("))
            .or_else(|| source.rfind("future("))
            .expect("independent unclosed function stimulus");
        (start, start + source[start..].find('(').unwrap() + 1)
    };
    point(opening.span().start(), source, start);
    point(opening.span().end(), source, end);
    point(at.span().start(), source, source.len());
    point(at.span().end(), source, source.len());
}
fn is_original_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin) -> bool {
    matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ) && error.origin() == &CssSerializedOrigin::End(Some(origin.clone()))
}
fn strict_original_components(property: P, inputs: &[String]) {
    let mut failures = Vec::new();
    for text in inputs {
        let components = parse_component_values(text).unwrap();
        let before = components.clone();
        let origin = images_implicit_origin(&components);
        original_closure_coordinates(&origin, text);
        for (front, result) in [
            (
                "property",
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
            (
                "grammar",
                parse_property_value_for_grammar(
                    property.grammar(),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
        ] {
            match result {
                Err(error) if is_original_closure(&error, &origin) => {},
                Err(error) => failures.push(format!("{front}/{text}: expected typed original UnexpectedEnd + exact ImplicitClosure EOF origin, got {error:?}")),
                Ok(_) => failures.push(format!("{front}/{text}: original recovered components were admitted after repair")),
            }
            assert_eq!(components, before);
        }
    }
    assert!(
        failures.is_empty(),
        "{property:?}:\n{}",
        failures.join("\n")
    );
}
fn original_comment_closures(property: P, ordinary: &str, pending_only: bool) {
    let values: Vec<_> = if pending_only {
        PENDING_CLOSED.to_vec()
    } else {
        std::iter::once(ordinary)
            .chain(GLOBALS.iter().map(|(value, _)| *value))
            .collect()
    };
    let inputs: Vec<_> = values
        .into_iter()
        .map(|value| format!("/*😀*/{value}/*unfinished"))
        .collect();
    strict_original_components(property, &inputs);
}
fn original_pending_function_closures(property: P) {
    strict_original_components(
        property,
        &PENDING_OPEN
            .iter()
            .map(|value| format!("/*😀*/{value}"))
            .collect::<Vec<_>>(),
    );
}
fn numeric_closure(property: P) -> Option<(&'static str, &'static str)> {
    match property {
        P::ObjectPosition => Some(("calc(1px + 5%", "calc(1px + 5%)")),
        P::ImageOrientation => Some(("calc(15deg + 15deg", "calc(15deg + 15deg)")),
        _ => None,
    }
}
#[test]
fn original_imported_length_percentage_calculation_cannot_be_repaired_into_object_position() {
    let (open, _) = numeric_closure(P::ObjectPosition).unwrap();
    strict_original_components(P::ObjectPosition, &[format!("/*😀*/{open}")]);
}
#[test]
fn original_imported_angle_calculation_cannot_be_repaired_into_image_orientation() {
    let (open, _) = numeric_closure(P::ImageOrientation).unwrap();
    strict_original_components(P::ImageOrientation, &[format!("/*😀*/{open}")]);
}
fn closed_component_controls(property: P, ordinary: &str) {
    let mut values: Vec<_> = std::iter::once(ordinary)
        .chain(GLOBALS.iter().map(|(text, _)| *text))
        .chain(PENDING_CLOSED)
        .map(|text| format!("/*😀*/{text}/**/"))
        .collect();
    if let Some((_, closed)) = numeric_closure(property) {
        values.push(format!("/*😀*/{closed}/**/"));
    }
    for text in values {
        for grammar in [false, true] {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            let source = authored_property::checked_components(
                property,
                components.clone(),
                grammar,
                CssImportance::Important,
            )
            .unwrap();
            assert_eq!(source.value_components(), &components);
            assert_eq!(source.importance(), CssImportance::Important);
            assert_eq!(source.known().unwrap().grammar(), property.grammar());
            assert!(source.position().is_none());
            assert!(source.parsed_name().is_none());
            assert!(source.parsed_value().is_none());
            assert_eq!(components, before);
        }
    }
}
fn recovered_images_report(property: P, ordinary: &str) {
    let mut values: Vec<_> = std::iter::once(ordinary)
        .chain(GLOBALS.iter().map(|(text, _)| *text))
        .chain(PENDING_CLOSED)
        .map(|text| format!("{text}/*unfinished"))
        .collect();
    values.extend(PENDING_OPEN.iter().map(|text| (*text).to_owned()));
    if let Some((open, _)) = numeric_closure(property) {
        values.push(open.to_owned());
    }
    for value in values {
        let css = format!(
            "/*😀*/.image{{color:red;{}:{value}",
            property.canonical_name()
        );
        let report = parse_sheet(&css);
        assert!(!report.is_clean());
        let before = report.clone();
        assert!(report.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic.error().kind(),
            ErrorKind::UnexpectedEnd(_)
        ) && diagnostic.action()
            == CssRecoveryAction::RetainWithImplicitClosure));
        assert_eq!(
            validate_sheet(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let CssRule::Style(style) = &report.syntax().rules()[0] else {
            panic!("retained recovered style")
        };
        let [color, source] = style.declarations().as_slice() else {
            panic!("retained valid sibling and recovered Images occurrence")
        };
        assert_eq!(color.known().unwrap().property(), P::Color);
        assert_eq!(source.known().unwrap().property(), property);
        let implicit = images_implicit_origin(source.value_components());
        original_closure_coordinates(&implicit, &css);
        assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
        let exact = CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert!(!normalized.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let [
            CssNormalizedItem::Rule(_),
            CssNormalizedItem::Declaration(first),
            CssNormalizedItem::Declaration(second),
        ] = normalized.syntax().items()
        else {
            panic!("retained two original declaration occurrences")
        };
        assert_eq!([first.order(), second.order()], [0, 1]);
        assert!(first.source().same_occurrence(color));
        assert!(second.source().same_occurrence(source));
        assert_eq!(
            images_implicit_origin(second.source().value_components()),
            implicit
        );
        match second.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(items)) => {
                member(items, source, None)
            }
            CssExpansion::Pending(handle) => assert!(handle.source().same_occurrence(source)),
            _ => panic!("ordinary/global/pending recovered longhand"),
        }
        assert!(
            normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 2, 1).unwrap()
            )
            .is_err()
        );
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}
fn closed_reentry(property: P, ordinary: &str) {
    for pending in PENDING_CLOSED {
        let source = checked(property, pending, true);
        let before = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending closure reentry")
        };
        let mut replacements: Vec<_> = std::iter::once(ordinary)
            .chain(GLOBALS.iter().map(|(text, _)| *text))
            .map(|text| format!("/*😀*/{text}/*unfinished"))
            .collect();
        replacements.push("/*😀*/future(value".to_owned());
        if let Some((open, _)) = numeric_closure(property) {
            replacements.push(format!("/*😀*/{open}"));
        }
        for text in replacements {
            let components = parse_component_values(&text).unwrap();
            let components_before = components.clone();
            let origin = images_implicit_origin(&components);
            original_closure_coordinates(&origin, &text);
            for _ in 0..2 {
                let error = handle.reenter(components.clone()).unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                    panic!("strict closure invalid replacement")
                };
                assert!(
                    is_original_closure(error, &origin),
                    "{property:?}/{text}: {error:?}"
                );
                assert_eq!(components, components_before);
                assert!(handle.source().same_occurrence(&source));
                assert_eq!(source, before);
            }
        }
        // Presence of a residual substitution wins even when the same original
        // replacement also has an implicit closure and cannot enter the grammar.
        for text in [
            "var(--again",
            "env(again",
            "attr(data-again",
            "bogus var(--again)/*unfinished",
            "[future(env(again",
            r"f(v\61 r(--again",
        ] {
            let replacement = parse_component_values(text).unwrap();
            let old = replacement.clone();
            for _ in 0..2 {
                assert_eq!(
                    handle.reenter(replacement.clone()).unwrap_err().kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
                assert_eq!(replacement, old);
                assert!(handle.source().same_occurrence(&source));
                assert_eq!(source, before);
            }
        }
        let mut retry: Vec<_> = std::iter::once(ordinary)
            .chain(GLOBALS.iter().map(|(text, _)| *text))
            .map(|text| format!("/*😀*/{text}/**/"))
            .collect();
        if let Some((_, closed)) = numeric_closure(property) {
            retry.push(closed.to_owned());
        }
        for text in retry {
            let replacement = parse_component_values(&text).unwrap();
            let old = replacement.clone();
            for _ in 0..2 {
                let CssContributions::Longhands(items) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("closed immutable retry")
                };
                member(&items, &source, Some(&replacement));
                assert_eq!(replacement, old);
                assert_eq!(source, before);
            }
        }
    }
}
