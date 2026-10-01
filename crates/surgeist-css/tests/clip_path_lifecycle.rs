#![forbid(unsafe_code)]
//! Authored grammar and intrinsic lifecycle, independently specified by:
//! Masking 1 CRD 2021-08-05 §5.1 and Shapes 1 CRD 2025-06-12 §3.1;
//! polygon's unrestricted authored length range uses Values 4 WD 2024-03-12 §5.1.
//! Ellipse's imported radial-size discrepancy uses the adopted WebKit
//! 73aa6c89e2cb77c46184a81aec944e4ab99d114d operational contract:
//! CSSPropertyParserConsumer+Shapes.cpp:243–318 and CSSEllipseFunction.h:35–42.
//! Positions here cover the physical subset, not unfinished logical positions.
//! Reference-box resolution, used geometry and clipping are downstream concerns.

use surgeist_css::*;

const BOXES: [&str; 7] = [
    "content-box",
    "padding-box",
    "border-box",
    "margin-box",
    "fill-box",
    "stroke-box",
    "view-box",
];

fn declaration(value: &str, importance: CssImportance) -> CssDeclaration {
    let suffix = if importance == CssImportance::Important {
        "!important"
    } else {
        ""
    };
    let source = format!("clip-path:{value}{suffix}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        validate_style_attribute(&source),
        Ok(report.syntax().clone())
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn clip_value(source: &CssDeclaration) -> &CssClipPath {
    let CssKnownPropertyValueRef::ClipPath(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip-path property")
    };
    value.value()
}

fn terminals(source: &CssDeclaration) -> CssLonghandContributions {
    match expand_declaration(source).expect("clip-path intrinsic expansion") {
        CssExpansion::Contributions(CssContributions::Longhands(values)) => values,
        other => panic!("completed terminal, got {other:?}"),
    }
}

fn assert_rejected(value: &str) {
    let prefix = "/* 🦀 */ ";
    let unit = format!("clip-path:{value};");
    let source = format!("{prefix}{unit} color:blue");
    let report = parse_style_attribute(&source);
    let [sibling] = report.syntax().as_slice() else {
        panic!("only color survives {value}: {:?}", report.syntax())
    };
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    let [diagnostic] = report.diagnostics() else {
        panic!("one diagnostic: {value}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    let end = prefix.len() + unit.len();
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    assert_eq!(
        diagnostic.span().start().column().value() as usize,
        prefix.encode_utf16().count()
    );
    assert_eq!(
        diagnostic.span().end().column().value() as usize,
        source[..end].encode_utf16().count()
    );
    let position = diagnostic.error().position();
    let byte = position.byte_offset().value();
    assert!(
        byte >= prefix.len() + "clip-path:".len() && byte < end,
        "{value}: {position:?}"
    );
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value() as usize,
        source[..byte].encode_utf16().count()
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn every_geometry_box_is_a_clean_standalone_clip_path() {
    for keyword in BOXES {
        let source = declaration(keyword, CssImportance::Normal);
        let CssKnownPropertyValueRef::ClipPath(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("clip-path value")
        };
        assert_eq!(value.as_css(), keyword);
    }
}

#[test]
fn one_geometry_box_combines_with_each_shape_in_either_authored_order() {
    for keyword in BOXES {
        for shape in ["inset(1px)", "circle()", "ellipse()", "polygon(0 0)"] {
            for text in [format!("{shape} {keyword}"), format!("{keyword} {shape}")] {
                let source = declaration(&text, CssImportance::Important);
                let CssKnownPropertyValueRef::ClipPath(value) =
                    source.known().unwrap().property_value().unwrap()
                else {
                    panic!("clip-path value")
                };
                assert_eq!(value.as_css(), text, "retain authored combination order");
            }
        }
    }
}

#[test]
fn duplicate_boxes_shapes_and_none_or_url_combinations_recover_once() {
    for value in [
        "border-box border-box",
        "content-box padding-box",
        "circle() border-box padding-box",
        "border-box circle() border-box",
        "circle() ellipse()",
        "circle() circle()",
        "none border-box",
        "border-box none",
        "none circle()",
        "circle() none",
        "url(\"#clip\") border-box",
        "border-box url(\"#clip\")",
        "url(\"#clip\") circle()",
        "circle() url(\"#clip\")",
        "none url(\"#clip\")",
        "text",
        "no-clip",
        "circle() trailing",
    ] {
        assert_rejected(value);
    }
}

#[test]
fn ellipse_accepts_two_independent_numeric_or_extent_components() {
    for value in [
        "ellipse(10px 25%)",
        "ellipse(0 0)",
        "ellipse(closest-side farthest-corner)",
        "ellipse(farthest-side closest-corner)",
        "ellipse(10% closest-side)",
        "ellipse(farthest-corner 2px)",
        "ellipse(calc(10% + 2px) farthest-side at left top)",
        "ellipse(closest-corner calc(20px + 1%) at right 5% bottom 2px)",
    ] {
        declaration(value, CssImportance::Normal);
    }
}

#[test]
fn ellipse_rejects_every_lone_explicit_radius_component() {
    for value in [
        "ellipse(closest-side)",
        "ellipse(farthest-side)",
        "ellipse(closest-corner)",
        "ellipse(farthest-corner)",
        "ellipse(1px)",
        "ellipse(10%)",
        "ellipse(0)",
        "ellipse(calc(1px + 2%))",
        "ellipse(closest-side at center)",
        "ellipse(2% at left top)",
        "ellipse(-1px 2px)",
        "ellipse(1px -2%)",
        "ellipse(1px 2px 3px)",
    ] {
        assert_rejected(value);
    }
}

#[test]
fn omitted_radii_and_physical_positions_remain_valid() {
    for value in [
        "circle()",
        "circle(at left)",
        "circle(25% at center)",
        "circle(closest-side at right 5% bottom 2px)",
        "ellipse()",
        "ellipse(at center)",
        "ellipse(at left top)",
        "ellipse(at right 5% bottom 2px)",
    ] {
        declaration(value, CssImportance::Normal);
    }
}

#[test]
fn polygon_rounding_accepts_signed_pure_lengths_after_optional_fill() {
    for value in [
        "polygon(round -1px, 0 0)",
        "polygon(nonzero round -1px, 0 0)",
        "polygon(evenodd round 0, 0 0, 100% 0)",
        "polygon(round +2em, -1px -2%)",
        "polygon(round calc(1px - 2px), 0 0)",
        "polygon(nonzero, 0 0)",
    ] {
        declaration(value, CssImportance::Normal);
    }
}

#[test]
fn polygon_reversed_duplicate_modifiers_and_percentage_rounding_recover_once() {
    for value in [
        "polygon(round 1px nonzero, 0 0)",
        "polygon(round 1px evenodd, 0 0)",
        "polygon(nonzero evenodd, 0 0)",
        "polygon(nonzero nonzero, 0 0)",
        "polygon(round 1px round 2px, 0 0)",
        "polygon(round 10%, 0 0)",
        "polygon(round calc(1px + 2%), 0 0)",
        "polygon(evenodd 0 0)",
    ] {
        assert_rejected(value);
    }
}

#[test]
fn clip_path_is_noninherited_with_an_intrinsic_none_initial() {
    let metadata = CssKnownProperty::ClipPath
        .metadata()
        .expect("clip-path intrinsic metadata");
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("clip-path terminal")
    };
    assert_eq!(
        longhand.property().known_property(),
        CssKnownProperty::ClipPath
    );
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::ClipPath
    );
    let CssInitialValueRef::Value(initial_value) = initial.view() else {
        panic!("intrinsic ordinary initial")
    };
    let none = declaration("none", CssImportance::Normal);
    assert_eq!(clip_value(&none), &CssClipPath::None);
    let values = terminals(&none);
    let [value] = values.items() else {
        panic!("one none terminal")
    };
    // No not-yet-existing borrowed variant is needed to compare the proven None.
    assert_eq!(initial_value, value.ordinary_value().unwrap());
}

#[test]
fn parsed_and_checked_ordinary_values_expand_with_their_occurrence_and_importance() {
    for importance in [CssImportance::Normal, CssImportance::Important] {
        for text in [
            "none",
            "url(\"#clip\")",
            "circle(25% at left top)",
            "border-box",
            "ellipse() fill-box",
        ] {
            let parsed = declaration(text, importance);
            let components = parse_component_values(text).unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
                components.clone(),
                importance,
            )
            .unwrap();
            assert!(checked.parsed_value().is_none());
            assert_eq!(checked.value_components(), &components);
            let parsed_values = terminals(&parsed);
            let checked_values = terminals(&checked);
            for (source, values) in [(&parsed, &parsed_values), (&checked, &checked_values)] {
                let [value] = values.items() else {
                    panic!("one ordinary terminal")
                };
                assert_eq!(value.property(), CssKnownProperty::ClipPath);
                assert!(matches!(
                    value.value(),
                    CssContributionValueRef::Ordinary(_)
                ));
                assert_eq!(
                    value.ordinary_value().unwrap().property().known_property(),
                    CssKnownProperty::ClipPath
                );
                assert!(value.source().same_occurrence(source));
                assert_eq!(value.source().importance(), importance);
                assert_eq!(value.source().value_components(), source.value_components());
                assert!(value.replacement_components().is_none());
            }
            assert_eq!(
                parsed_values.items()[0].ordinary_value(),
                checked_values.items()[0].ordinary_value()
            );
        }
    }
}

#[test]
fn css_wide_keywords_remain_symbolic_single_terminal_contributions() {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(text, CssImportance::Important);
        let values = terminals(&source);
        let [value] = values.items() else {
            panic!("one symbolic terminal")
        };
        assert_eq!(value.property(), CssKnownProperty::ClipPath);
        assert_eq!(value.value(), CssContributionValueRef::Global(keyword));
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert!(value.replacement_components().is_none());
    }
}

#[test]
fn substitution_reentry_rejects_invalid_or_residual_values_and_remains_reusable() {
    for text in ["var(--clip)", "env(clip)"] {
        let source = declaration(text, CssImportance::Important);
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending substitution")
        };
        assert!(pending.source().same_occurrence(&source));
        let before = source.clone();
        for invalid in [
            "",
            "none border-box",
            "ellipse(closest-side)",
            "polygon(round 1px nonzero, 0 0)",
            "polygon(round 2%, 0 0)",
            "circle();color:red",
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
        for residual in ["var(--again)", "circle(env(radius))", "attr(clip)"] {
            assert_eq!(
                pending
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        for valid in [
            "none",
            "border-box circle()",
            "ellipse(10% closest-side)",
            "polygon(round -1px, 0 0)",
            "inherit",
        ] {
            let replacement = parse_component_values(valid).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) = pending
                    .reenter(replacement.clone())
                    .expect("valid reentry after failed attempts")
                else {
                    panic!("completed replacement")
                };
                let [value] = values.items() else {
                    panic!("one replacement terminal")
                };
                assert_eq!(value.property(), CssKnownProperty::ClipPath);
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
        }
        assert_eq!(source, before);
    }
}

#[test]
fn normalization_preserves_child_order_pending_context_and_resource_errors() {
    let css = ".outer{clip-path:circle(25%)!important;color:red;.child{clip-path:inherit}clip-path:var(--clip)}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 3, 4, 4).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact)
        .expect("intrinsic clip-path normalization");
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (order, (value, property)) in values
        .iter()
        .zip([
            CssKnownProperty::ClipPath,
            CssKnownProperty::Color,
            CssKnownProperty::ClipPath,
            CssKnownProperty::ClipPath,
        ])
        .enumerate()
    {
        assert_eq!(value.order(), order);
        assert_eq!(value.source().known().unwrap().property(), property);
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
    for index in [0, 2] {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            values[index].expansion()
        else {
            panic!("ordinary/global terminal")
        };
        let [value] = items.items() else {
            panic!("one clip terminal")
        };
        assert_eq!(value.property(), CssKnownProperty::ClipPath);
        assert!(value.source().same_occurrence(values[index].source()));
        if index == 2 {
            assert_eq!(
                value.value(),
                CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
            );
        } else {
            assert!(value.ordinary_value().is_some());
        }
    }
    let CssExpansion::Pending(pending) = values[3].expansion() else {
        panic!("trailing pending")
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
fn recovery_normalization_keeps_valid_clip_occurrences_in_source_order() {
    let report = parse_sheet(
        ".a{clip-path:none;clip-path:none border-box;clip-path:circle()!important;clip-path:inherit}",
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    let normalized =
        normalize_sheet(report.syntax()).expect("recovered valid declarations normalize");
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 3);
    for (order, (value, importance)) in values
        .iter()
        .zip([
            CssImportance::Normal,
            CssImportance::Important,
            CssImportance::Normal,
        ])
        .enumerate()
    {
        assert_eq!(value.order(), order);
        assert_eq!(value.source().importance(), importance);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) = value.expansion()
        else {
            panic!("terminal")
        };
        let [terminal] = items.items() else {
            panic!("single contribution")
        };
        assert_eq!(terminal.property(), CssKnownProperty::ClipPath);
        assert!(terminal.source().same_occurrence(value.source()));
    }
    assert_eq!(validate_sheet(".a{clip-path:none;clip-path:none border-box;clip-path:circle()!important;clip-path:inherit}").unwrap_err().diagnostics(), report.diagnostics());
}
