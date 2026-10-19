#![forbid(unsafe_code)]
//! Authored requirements from Filter Effects 1 WD 2018-12-18 §§5, 6.1, 6.3:
//! https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/
//! The named backdrop property imports only §2 of the immutable exploring ED:
//! https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/Overview.bs
//! This checks authored grammar and intrinsic lifecycle, not filter execution.

use surgeist_css::*;

const BACKDROP_SOURCE: &str = "https://raw.githubusercontent.com/w3c/csswg-drafts/034f50a78495b619478342d71117cd7ca7e76de7/filter-effects-2/Overview.bs";

fn declaration(
    property: CssKnownProperty,
    value: &str,
    importance: CssImportance,
) -> CssDeclaration {
    let suffix = if importance == CssImportance::Important {
        "!important"
    } else {
        ""
    };
    let source = format!("{}:{value}{suffix}", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        validate_style_attribute(&source),
        Ok(report.syntax().clone())
    );
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration: {source}");
    };
    value.clone()
}

fn filter_value(declaration: &CssDeclaration) -> &CssFilter {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Filter(value) => value.value(),
        CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
        other => panic!("filter property, got {other:?}"),
    }
}

fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    match expand_declaration(source).expect("filter intrinsic expansion") {
        CssExpansion::Contributions(CssContributions::Longhands(values)) => values,
        other => panic!("completed longhand, got {other:?}"),
    }
}

fn assert_omitted_hue_order(property: CssKnownProperty) {
    let source = declaration(
        property,
        concat!(
            "blur() hue-rotate() brightness() contrast(25%) ",
            "drop-shadow(red -1px 2px) grayscale() invert() opacity() ",
            "saturate() sepia() url(\"filters.svg#rough\")"
        ),
        CssImportance::Normal,
    );
    let CssFilter::Functions(list) = filter_value(&source) else {
        panic!("ordered function list");
    };
    let [
        CssFilterFunction::Blur(blur),
        CssFilterFunction::HueRotate(_),
        CssFilterFunction::Brightness(CssFilterAmount::Default),
        CssFilterFunction::Contrast(CssFilterAmount::Percentage(contrast)),
        CssFilterFunction::DropShadow(shadow),
        CssFilterFunction::Grayscale(CssFilterAmount::Default),
        CssFilterFunction::Invert(CssFilterAmount::Default),
        CssFilterFunction::Opacity(CssFilterAmount::Default),
        CssFilterFunction::Saturate(CssFilterAmount::Default),
        CssFilterFunction::Sepia(CssFilterAmount::Default),
        CssFilterFunction::Url(url),
    ] = list.functions()
    else {
        panic!("retain authored function order, defaults and URL: {list:?}");
    };
    assert!(blur.authored_length().is_none());
    assert_eq!(blur.length().serialize_specified().unwrap(), "0px");
    assert_eq!(contrast.serialize_specified().unwrap(), "25%");
    assert_eq!(shadow.offset_x().serialize_specified().unwrap(), "-1px");
    assert_eq!(shadow.offset_y().serialize_specified().unwrap(), "2px");
    assert!(shadow.standard_deviation().is_none());
    assert!(shadow.color().is_some());
    assert_eq!(url.as_str(), "filters.svg#rough");
}

#[test]
fn filter_accepts_omitted_hue_in_an_ordered_typed_list() {
    assert_omitted_hue_order(CssKnownProperty::Filter);
}

#[test]
fn backdrop_accepts_omitted_hue_in_an_ordered_typed_list() {
    assert_omitted_hue_order(CssKnownProperty::BackdropFilter);
}

#[test]
fn explicit_hue_zero_units_and_symbolic_angles_remain_valid() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        for value in [
            "hue-rotate(0)",
            "hue-rotate(0deg)",
            "hue-rotate(-.25turn)",
            "hue-rotate(calc(1turn - 90deg))",
        ] {
            let source = declaration(property, value, CssImportance::Normal);
            let CssFilter::Functions(list) = filter_value(&source) else {
                panic!("one hue function");
            };
            assert!(matches!(
                list.functions(),
                [CssFilterFunction::HueRotate(_)]
            ));
            let authored = match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::Filter(filter) => filter.as_css(),
                CssKnownPropertyValueRef::BackdropFilter(filter) => filter.as_css(),
                other => panic!("filter property: {other:?}"),
            };
            assert_eq!(authored, value);
        }
    }
}

#[test]
fn invalid_filter_values_recover_to_the_sibling_with_utf16_provenance() {
    for property in ["filter", "backdrop-filter"] {
        for value in [
            "none blur(1px)",
            "blur(-1px)",
            "blur(2%)",
            "hue-rotate(1)",
            "hue-rotate(1deg, 2deg)",
            "brightness(-1)",
            "contrast(-1%)",
            "opacity(1 2)",
            "drop-shadow(1px 2px -3px)",
            "blur(1px),opacity(1)",
            "blur(1px) trailing",
            "unknown(1)",
        ] {
            let prefix = "/* 🦀 */ ";
            let unit = format!("{property}:{value};");
            let source = format!("{prefix}{unit} color:blue");
            let report = parse_style_attribute(&source);
            assert!(!report.is_clean(), "{source}");
            let [sibling] = report.syntax().as_slice() else {
                panic!("only valid color sibling survives: {source}");
            };
            assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
            let [diagnostic] = report.diagnostics() else {
                panic!("one responsible declaration diagnostic: {source}");
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            let span = diagnostic.span();
            assert_eq!(span.start().byte_offset().value(), prefix.len());
            assert_eq!(span.end().byte_offset().value(), prefix.len() + unit.len());
            assert_eq!(
                span.start().column().value() as usize,
                prefix.encode_utf16().count()
            );
            assert_eq!(
                span.end().column().value() as usize,
                source[..prefix.len() + unit.len()].encode_utf16().count()
            );
            let position = diagnostic.error().position();
            let byte = position.byte_offset().value();
            assert!(
                byte > prefix.len() + property.len() && byte < prefix.len() + unit.len(),
                "responsible value position: {source}: {position:?}"
            );
            assert_eq!(position.line().value(), 0);
            assert_eq!(
                position.column().value() as usize,
                source[..byte].encode_utf16().count()
            );
            let error = validate_style_attribute(&source).unwrap_err();
            assert_eq!(error.diagnostics(), report.diagnostics());
        }
    }
}

fn assert_metadata(property: CssKnownProperty) {
    let metadata = property.metadata().expect("filter intrinsic metadata");
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("filter is a terminal");
    };
    assert_eq!(longhand.property().known_property(), property);
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(initial.property().known_property(), property);
    let CssInitialValueRef::Value(initial_value) = initial.view() else {
        panic!("context-independent ordinary initial");
    };
    assert_eq!(initial_value.property().known_property(), property);
    // Existing public metadata cannot match a not-yet-added borrowed filter
    // variant. Compare with a terminal whose authored payload is proven None.
    let none = declaration(property, "none", CssImportance::Normal);
    assert!(matches!(filter_value(&none), CssFilter::None));
    let values = contributions(&none);
    let [value] = values.items() else {
        panic!("one none terminal");
    };
    assert_eq!(initial_value, value.ordinary_value().unwrap());
}

#[test]
fn filter_is_noninherited_with_an_intrinsic_none_initial() {
    assert_metadata(CssKnownProperty::Filter);
}

#[test]
fn backdrop_is_noninherited_with_an_intrinsic_none_initial() {
    assert_metadata(CssKnownProperty::BackdropFilter);
}

fn assert_ordinary(property: CssKnownProperty) {
    for importance in [CssImportance::Normal, CssImportance::Important] {
        for text in ["none", "blur(1px) opacity(.5)"] {
            let parsed = declaration(property, text, importance);
            let components = parse_component_values(text).unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                importance,
            )
            .unwrap();
            for source in [parsed, checked] {
                let values = contributions(&source);
                let [value] = values.items() else {
                    panic!("one terminal contribution");
                };
                assert_eq!(value.property(), property);
                assert!(matches!(
                    value.value(),
                    CssContributionValueRef::Ordinary(_)
                ));
                assert_eq!(
                    value.ordinary_value().unwrap().property().known_property(),
                    property
                );
                assert!(value.source().same_occurrence(&source));
                assert_eq!(value.source().importance(), importance);
                assert_eq!(value.source().value_components(), source.value_components());
                assert!(value.replacement_components().is_none());
            }
        }
    }
}

#[test]
fn filter_ordinary_expansion_retains_one_occurrence_and_importance() {
    assert_ordinary(CssKnownProperty::Filter);
}

#[test]
fn backdrop_ordinary_expansion_retains_one_occurrence_and_importance() {
    assert_ordinary(CssKnownProperty::BackdropFilter);
}

fn assert_globals(property: CssKnownProperty) {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(property, text, CssImportance::Important);
        let values = contributions(&source);
        let [value] = values.items() else {
            panic!("one symbolic terminal");
        };
        assert_eq!(value.property(), property);
        assert_eq!(value.value(), CssContributionValueRef::Global(keyword));
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert!(value.replacement_components().is_none());
    }
}

#[test]
fn filter_css_wide_contributions_remain_symbolic() {
    assert_globals(CssKnownProperty::Filter);
}

#[test]
fn backdrop_css_wide_contributions_remain_symbolic() {
    assert_globals(CssKnownProperty::BackdropFilter);
}

fn assert_reentry(property: CssKnownProperty) {
    for text in ["var(--effect)", "env(effect)"] {
        let source = declaration(property, text, CssImportance::Important);
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("substitution remains pending");
        };
        assert!(pending.source().same_occurrence(&source));
        for invalid in [
            "",
            "none blur(1px)",
            "blur(-1px)",
            "opacity(1); color:red",
            "none!important",
        ] {
            let error = pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err();
            assert!(
                matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
                "{invalid}: {error:?}"
            );
        }
        for residual in ["var(--again)", "blur(env(radius))", "attr(effect)"] {
            assert_eq!(
                pending
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        for valid in ["none", "blur() hue-rotate()", "inherit"] {
            let replacement = parse_component_values(valid).unwrap();
            let CssContributions::Longhands(values) = pending
                .reenter(replacement.clone())
                .expect("validated replacement after earlier failures")
            else {
                panic!("completed replacement");
            };
            let [value] = values.items() else {
                panic!("one replacement contribution");
            };
            assert_eq!(value.property(), property);
            assert!(value.source().same_occurrence(&source));
            assert_eq!(value.source().importance(), CssImportance::Important);
            assert_eq!(value.replacement_components(), Some(&replacement));
            if valid == "inherit" {
                assert_eq!(
                    value.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
            } else {
                assert!(matches!(
                    value.value(),
                    CssContributionValueRef::Ordinary(_)
                ));
            }
        }
        assert!(pending.source().same_occurrence(&source));
    }
}

#[test]
fn filter_pending_reentry_is_strict_and_reusable() {
    assert_reentry(CssKnownProperty::Filter);
}

#[test]
fn backdrop_pending_reentry_is_strict_and_reusable() {
    assert_reentry(CssKnownProperty::BackdropFilter);
}

fn assert_normalization(property: CssKnownProperty) {
    let css = format!(
        ".outer{{{}:blur(1px)!important; color:red; .child{{{}:none}} {}:var(--effect)}}",
        property.canonical_name(),
        property.canonical_name(),
        property.canonical_name()
    );
    let report = parse_sheet(&css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    // Outer style, child style, and trailing implicit declaration run.
    let exact = CssNormalizationLimits::try_new(1, 3, 4, 4).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact)
        .expect("filter declarations normalize alongside other declarations and child rules");
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (order, (value, expected)) in values
        .iter()
        .zip([property, CssKnownProperty::Color, property, property])
        .enumerate()
    {
        assert_eq!(value.order(), order);
        assert_eq!(value.source().known().unwrap().property(), expected);
    }
    assert_eq!(values[0].source().importance(), CssImportance::Important);
    assert!(
        values[0]
            .selector_context()
            .same_context(values[1].selector_context())
    );
    assert!(
        values[0]
            .selector_context()
            .same_context(values[3].selector_context())
    );
    assert!(
        !values[0]
            .selector_context()
            .same_context(values[2].selector_context())
    );
    let CssExpansion::Contributions(CssContributions::Longhands(first)) = values[0].expansion()
    else {
        panic!("ordinary filter expansion");
    };
    let [first] = first.items() else {
        panic!("one filter contribution");
    };
    assert_eq!(first.property(), property);
    assert!(first.source().same_occurrence(values[0].source()));
    let CssExpansion::Pending(pending) = values[3].expansion() else {
        panic!("trailing substitution stays pending");
    };
    assert!(pending.source().same_occurrence(values[3].source()));
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 3, 3, 4).unwrap(),
            CssNormalizationResource::Declarations,
        ),
        (
            CssNormalizationLimits::try_new(1, 3, 4, 3).unwrap(),
            CssNormalizationResource::Contributions,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit: 3 }
        );
        assert_eq!(error.declaration_order(), Some(3));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(values[3].source())
        );
        assert_eq!(
            error.position().unwrap(),
            values[3].source().position().unwrap()
        );
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}

#[test]
fn filter_normalization_preserves_child_order_pending_context_and_limits() {
    assert_normalization(CssKnownProperty::Filter);
}

#[test]
fn backdrop_normalization_preserves_child_order_pending_context_and_limits() {
    assert_normalization(CssKnownProperty::BackdropFilter);
}

#[test]
fn backdrop_provenance_cites_only_the_named_immutable_exception() {
    let source =
        specification_source("X-BACKDROP-FILTER").expect("selected backdrop source identity");
    assert_eq!(source.module(), "Filter Effects");
    assert_eq!(source.level(), "2 named property exception");
    assert_eq!(source.tier(), CssSpecificationTier::SurgeistExtension);
    assert_eq!(source.url(), Some(BACKDROP_SOURCE));
    assert_eq!(source.repository_provenance(), None);
    assert!(specification_source("X-FILTER2-BASE").is_none());
    let feature = property_support_metadata("backdrop-filter")
        .unwrap()
        .feature();
    assert_eq!(feature.source(), *source);
    assert_eq!(feature.production(), "#BackdropFilterProperty");
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.supported_subset(), None);
    assert_eq!(feature.unsupported_remainder(), None);
    assert_eq!(
        property_support_metadata("filter")
            .unwrap()
            .feature()
            .source()
            .id()
            .as_str(),
        "I-FILTER1"
    );
}
