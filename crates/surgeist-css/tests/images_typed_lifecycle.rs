#![forbid(unsafe_code)]
//! Functional tests for the Images 3 authored rendering alternatives and borrowed
//! longhand terminals introduced with their implementation. Independent authority:
//! selected Images 3 CRD 2023-12-18 §§4.5–5.2, including §5.2 standard legacy
//! acceptance, and imported physical Values 3 positions. No missing-symbol RED.

#[path = "common/property_expectations.rs"]
mod property_expectations;

use surgeist_css::{
    CssKnownProperty as P, CssSpecifiedValueSerializationErrorKind as K,
    CssSpecifiedValueSerializationLimits as L, *,
};

// Specialized rendering identity/output cases; generic property/initial inventory
// remains in the shared independent common records.
const RENDERING: [(CssImageRendering, &str); 7] = [
    (CssImageRendering::Auto, "auto"),
    (CssImageRendering::Smooth, "smooth"),
    (CssImageRendering::HighQuality, "high-quality"),
    (CssImageRendering::Pixelated, "pixelated"),
    (CssImageRendering::CrispEdges, "crisp-edges"),
    (CssImageRendering::OptimizeSpeed, "optimizespeed"),
    (CssImageRendering::OptimizeQuality, "optimizequality"),
];
fn parsed(property: P, value: &str) -> CssDeclaration {
    let text = format!("/*😀*/{}:{value}!important", property.canonical_name());
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one typed Images occurrence")
    };
    source.clone()
}
fn checked(property: P, components: CssComponentValues, grammar: bool) -> CssDeclaration {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    }
    .unwrap()
}
fn ordinary(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one completed ordinary longhand")
    };
    let [item] = items.items() else {
        panic!("one Images terminal")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert!(item.replacement_components().is_none());
    assert!(item.ordinary_value().is_some());
    items
}
fn exact_origin(value: &CssValueOrigin, text: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = value else {
        panic!("parsed scalar child")
    };
    assert_eq!(origin.source().as_str(), text);
    let start = text.find(token).unwrap();
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + token.len()
    );
    assert_eq!(
        origin.span().start().column().value() as usize,
        text[..start].encode_utf16().count()
    );
    assert_eq!(
        origin.span().end().column().value() as usize,
        text[..start + token.len()].encode_utf16().count()
    );
}
fn terminal_matches_authored(item: &CssLonghandContribution, source: &CssDeclaration) {
    let typed = item.ordinary_value().unwrap().view();
    let value = source.known().unwrap().property_value().unwrap();
    match (typed, value) {
        (
            CssLonghandValueRef::ObjectPosition(terminal),
            CssKnownPropertyValueRef::ObjectPosition(authored),
        ) => assert_eq!(terminal, authored.position()),
        (
            CssLonghandValueRef::ImageOrientation(terminal),
            CssKnownPropertyValueRef::ImageOrientation(authored),
        ) => assert_eq!(terminal, authored.orientation()),
        (
            CssLonghandValueRef::ImageRendering(terminal),
            CssKnownPropertyValueRef::ImageRendering(authored),
        ) => assert_eq!(terminal, authored.rendering()),
        (
            CssLonghandValueRef::ObjectFit(terminal),
            CssKnownPropertyValueRef::ObjectFit(authored),
        ) => assert_eq!(terminal, authored.fit()),
        _ => panic!("ordinary terminal preserves its property's borrowed type"),
    }
}
#[test]
fn all_seven_rendering_variants_construct_serialize_parse_and_expand_without_alias_collapse() {
    for (index, &(expected, css)) in RENDERING.iter().enumerate() {
        assert!(
            RENDERING
                .iter()
                .skip(index + 1)
                .all(|(other, _)| *other != expected)
        );
        assert_eq!(expected.serialize_specified().unwrap(), css);
        for source in [
            parsed(P::ImageRendering, &css.to_ascii_uppercase()),
            checked(
                P::ImageRendering,
                parse_component_values(css).unwrap(),
                false,
            ),
            checked(
                P::ImageRendering,
                parse_component_values(css).unwrap(),
                true,
            ),
        ] {
            let CssKnownPropertyValueRef::ImageRendering(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("rendering typed wrapper")
            };
            assert_eq!(value.rendering(), &expected);
            let values = ordinary(&source);
            terminal_matches_authored(&values.items()[0], &source);
            let CssLonghandValueRef::ImageRendering(terminal) =
                values.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("rendering typed terminal")
            };
            assert_eq!(*terminal, expected);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("image-rendering: {css} !important;")
            );
            assert_eq!(
                parsed(P::ImageRendering, &terminal.serialize_specified().unwrap())
                    .to_specified_css()
                    .unwrap(),
                format!("image-rendering: {css} !important;")
            );
        }
    }
    assert_ne!(
        CssImageRendering::OptimizeSpeed,
        CssImageRendering::CrispEdges
    );
    assert_ne!(
        CssImageRendering::OptimizeQuality,
        CssImageRendering::Smooth
    );
}
#[test]
fn new_rendering_keywords_keep_exact_primitive_and_declaration_work_budgets() {
    for (value, expected) in RENDERING {
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(1, 1, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(0, 1, expected.len()), K::InputNodeLimit),
            (L::new(1, 0, expected.len()), K::ProjectionNodeLimit),
            (L::new(1, 1, expected.len() - 1), K::ByteLimit),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    value
                        .serialize_specified_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
            }
        }
        let source = parsed(P::ImageRendering, expected);
        let before = source.clone();
        let output = format!("image-rendering: {expected} !important;");
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(3, 3, output.len()))
                .unwrap(),
            output
        );
        for (limits, kind) in [
            (L::new(2, 3, output.len()), K::InputNodeLimit),
            (L::new(3, 2, output.len()), K::ProjectionNodeLimit),
            (L::new(3, 3, output.len() - 1), K::ByteLimit),
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
        assert_eq!(source.to_specified_css().unwrap(), output);
    }
}
#[test]
fn selected_fixed_initials_are_exact_typed_values_with_programmatic_percentage_origins() {
    for row in property_expectations::CASES.iter().filter(|row| {
        matches!(
            row.property,
            P::ObjectPosition | P::ImageOrientation | P::ImageRendering | P::ObjectFit
        )
    }) {
        let metadata = row.property.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("independent Images longhand")
        };
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("fixed intrinsic initial")
        };
        assert_eq!(value.property().known_property(), row.property);
        match (row.property, value.view()) {
            (P::ObjectPosition, CssLonghandValueRef::ObjectPosition(position)) => {
                let (CssHorizontalPosition::Offset(x), CssVerticalPosition::Offset(y)) =
                    (position.horizontal(), position.vertical())
                else {
                    panic!("two free percentage initial axes")
                };
                for offset in [x, y] {
                    assert_eq!(offset.origin(), &CssValueOrigin::Programmatic);
                    assert!(
                        matches!(offset.literal_component().unwrap().view(), CssComponentValueRef::Token(
                        CssValueTokenRef::Percentage(number)) if number.representation() == "50")
                    );
                }
                assert_eq!(position.serialize_specified().unwrap(), "50% 50%");
                assert!(!longhand.inherited_by_default());
            }
            (P::ImageOrientation, CssLonghandValueRef::ImageOrientation(value)) => {
                assert_eq!(*value, CssImageOrientation::FromImage);
                assert!(longhand.inherited_by_default());
            }
            (P::ImageRendering, CssLonghandValueRef::ImageRendering(value)) => {
                assert_eq!(*value, CssImageRendering::Auto);
                assert!(longhand.inherited_by_default());
            }
            (P::ObjectFit, CssLonghandValueRef::ObjectFit(value)) => {
                assert_eq!(*value, CssObjectFit::Fill);
                assert!(!longhand.inherited_by_default());
            }
            _ => panic!("initial has exact property-specific borrowed payload"),
        }
        assert_eq!(metadata.grammar(), row.property.grammar());
    }
}
#[test]
fn parsed_and_checked_terminal_payloads_preserve_raw_scalars_origins_and_property_types() {
    for (property, input) in [
        (P::ObjectPosition, "bottom -2.00% right -1e-2px"),
        (P::ImageOrientation, "flip -3.000e1grad"),
        (P::ImageRendering, "OptimizeQuality"),
        (P::ObjectFit, "COVER"),
    ] {
        for source in [
            parsed(property, input),
            checked(property, parse_component_values(input).unwrap(), false),
            checked(property, parse_component_values(input).unwrap(), true),
        ] {
            let before = source.clone();
            let values = ordinary(&source);
            let item = &values.items()[0];
            terminal_matches_authored(item, &source);
            let text = source
                .parsed_name()
                .map_or(input, |name| name.source().as_str());
            match item.ordinary_value().unwrap().view() {
                CssLonghandValueRef::ObjectPosition(position) => {
                    let (
                        CssHorizontalPosition::RightOffset(x),
                        CssVerticalPosition::BottomOffset(y),
                    ) = (position.horizontal(), position.vertical())
                    else {
                        panic!("preserved edge directions")
                    };
                    exact_origin(x.origin(), text, "-1e-2px");
                    exact_origin(y.origin(), text, "-2.00%");
                    assert!(
                        matches!(x.literal_component().unwrap().view(), CssComponentValueRef::Token(
                        CssValueTokenRef::Dimension { number, unit }) if number.representation() == "-1e-2" && unit == "px")
                    );
                    assert!(
                        matches!(y.literal_component().unwrap().view(), CssComponentValueRef::Token(
                        CssValueTokenRef::Percentage(number)) if number.representation() == "-2.00")
                    );
                }
                CssLonghandValueRef::ImageOrientation(CssImageOrientation::Flip(Some(angle))) => {
                    exact_origin(angle.origin(), text, "-3.000e1grad");
                    assert_eq!(
                        angle.literal().unwrap().numeric().representation(),
                        "-3.000e1"
                    );
                    assert_eq!(angle.literal().unwrap().unit(), CssAngleUnit::Gradians);
                }
                CssLonghandValueRef::ImageRendering(value) => {
                    assert_eq!(*value, CssImageRendering::OptimizeQuality)
                }
                CssLonghandValueRef::ObjectFit(value) => assert_eq!(*value, CssObjectFit::Cover),
                _ => panic!("selected authored payload"),
            }
            assert_eq!(source, before);
        }
    }
}
fn pending(property: P) -> (CssDeclaration, CssPendingSubstitution) {
    let source = parsed(property, "var(--image)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending typed Images role")
    };
    (source, handle)
}
fn reentered(
    handle: &CssPendingSubstitution,
    source: &CssDeclaration,
    replacement: &CssComponentValues,
) -> CssLonghandContributions {
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("typed completed reentry")
    };
    let [item] = values.items() else {
        panic!("one intrinsic Images terminal")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(replacement));
    values
}
#[test]
fn physical_position_reentry_preserves_mixed_scalar_origins_and_original_pending_occurrence() {
    let (source, handle) = pending(P::ObjectPosition);
    let before = source.clone();
    let text = "/*😀*/right -1e-2px bottom -2.00%";
    let components = parse_component_values(text).unwrap();
    let mut items = components.items().to_vec();
    let x_index = items
        .iter()
        .position(|item| {
            matches!(item.view(), CssComponentValueRef::Token(
        CssValueTokenRef::Dimension { number, .. }) if number.representation() == "-1e-2")
        })
        .unwrap();
    items[x_index] = CssComponentValue::try_token("-1e-2px").unwrap();
    let replacement = CssComponentValues::try_new(items).unwrap();
    let old = replacement.clone();
    for _ in 0..2 {
        let values = reentered(&handle, &source, &replacement);
        let CssLonghandValueRef::ObjectPosition(position) =
            values.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("physical payload")
        };
        let (CssHorizontalPosition::RightOffset(x), CssVerticalPosition::BottomOffset(y)) =
            (position.horizontal(), position.vertical())
        else {
            panic!("paired original edges")
        };
        assert_eq!(x.origin(), &CssValueOrigin::Programmatic);
        exact_origin(y.origin(), text, "-2.00%");
        assert_eq!(
            position.serialize_specified().unwrap(),
            "right -0.01px bottom -2%"
        );
        assert_eq!(replacement, old);
        assert_eq!(source, before);
        assert!(handle.source().same_occurrence(&source));
    }
}
#[test]
fn orientation_reentry_preserves_authored_angle_units_math_and_programmatic_origin() {
    let (source, handle) = pending(P::ImageOrientation);
    let before = source.clone();
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_token("flip").unwrap(),
        CssComponentValue::try_token("-0.25turn").unwrap(),
    ])
    .unwrap();
    let old = components.clone();
    for _ in 0..2 {
        let values = reentered(&handle, &source, &components);
        let CssLonghandValueRef::ImageOrientation(CssImageOrientation::Flip(Some(angle))) =
            values.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("angle plus flip terminal")
        };
        assert_eq!(angle.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(angle.literal().unwrap().numeric().representation(), "-0.25");
        assert_eq!(angle.literal().unwrap().unit(), CssAngleUnit::Turns);
        assert_eq!(components, old);
        assert_eq!(source, before);
    }
    let text = "/*😀*/flip calc(15deg + 15deg)";
    let replacement = parse_component_values(text).unwrap();
    let values = reentered(&handle, &source, &replacement);
    let CssLonghandValueRef::ImageOrientation(orientation @ CssImageOrientation::Flip(Some(angle))) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("symbolic angle calculation")
    };
    assert!(angle.literal().is_none());
    assert!(angle.calculation().is_some());
    exact_origin(angle.origin(), text, "calc(");
    let calculation = angle.calculation().unwrap();
    let CssComponentValueRef::Function(function) = calculation.components().items()[0].view()
    else {
        panic!("retained angle calculation function")
    };
    exact_origin(function.closing_origin(), text, ")");
    assert_eq!(
        calculation.components().serialize().unwrap().as_css(),
        "calc(15deg + 15deg)"
    );
    assert_eq!(
        orientation.serialize_specified().unwrap(),
        "calc(30deg) flip"
    );
    assert_eq!(source, before);
}
#[test]
fn all_rendering_and_fit_replacements_preserve_exact_typed_alternatives() {
    let (render_source, render_handle) = pending(P::ImageRendering);
    let before = render_source.clone();
    for (expected, css) in RENDERING {
        let replacement = parse_component_values(&css.to_ascii_uppercase()).unwrap();
        for _ in 0..2 {
            let values = reentered(&render_handle, &render_source, &replacement);
            let CssLonghandValueRef::ImageRendering(actual) =
                values.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("rendering replacement typed variant")
            };
            assert_eq!(*actual, expected);
            assert_eq!(render_source, before);
        }
    }
    let (source, handle) = pending(P::ObjectFit);
    let before = source.clone();
    for (expected, css) in [
        (CssObjectFit::Fill, "fill"),
        (CssObjectFit::Contain, "contain"),
        (CssObjectFit::Cover, "cover"),
        (CssObjectFit::None, "none"),
        (CssObjectFit::ScaleDown, "scale-down"),
    ] {
        let replacement = parse_component_values(css).unwrap();
        let values = reentered(&handle, &source, &replacement);
        let CssLonghandValueRef::ObjectFit(actual) =
            values.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("fit replacement typed variant")
        };
        assert_eq!(*actual, expected);
        assert_eq!(source, before);
    }
}
#[test]
fn rendering_normalization_retains_seven_typed_occurrences_with_authored_order_and_importance() {
    let report = parse_sheet(concat!(
        ".image{image-rendering:auto;image-rendering:smooth!important;image-rendering:high-quality;",
        "image-rendering:pixelated;image-rendering:crisp-edges;image-rendering:optimizeSpeed;image-rendering:optimizeQuality}"
    ));
    assert!(report.is_clean());
    let before = report.clone();
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("image rule")
    };
    let normalized = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 7, 7).unwrap(),
    )
    .unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 7);
    for (order, ((normalized, authored), (expected, css))) in declarations
        .iter()
        .zip(style.declarations().iter())
        .zip(RENDERING)
        .enumerate()
    {
        assert_eq!(normalized.order(), order);
        assert!(normalized.source().same_occurrence(authored));
        assert_eq!(
            normalized.source().importance(),
            if order == 1 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            normalized.expansion()
        else {
            panic!("one rendering terminal")
        };
        let [item] = values.items() else {
            panic!("one rendering contribution")
        };
        let CssLonghandValueRef::ImageRendering(actual) = item.ordinary_value().unwrap().view()
        else {
            panic!("rendering terminal")
        };
        assert_eq!(*actual, expected);
        assert_eq!(actual.serialize_specified().unwrap(), css);
    }
    assert_eq!(report, before);
}
#[test]
fn selected_images_source_stays_dated_with_exact_public_property_productions() {
    let source = specification_source("O-IMAGES3").unwrap();
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2023/CRD-css-images-3-20231218/")
    );
    for row in property_expectations::CASES.iter().filter(|row| {
        matches!(
            row.property,
            P::ObjectPosition | P::ImageOrientation | P::ImageRendering | P::ObjectFit
        )
    }) {
        let feature = property_support_metadata(row.name).unwrap().feature();
        assert_eq!(feature.source(), *source);
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.production(), format!("#propdef-{}", row.name));
        assert!(feature.supported_subset().is_none());
        assert!(feature.unsupported_remainder().is_none());
    }
}

#[test]
fn programmatic_components_keep_programmatic_scalar_origins_in_all_four_terminals() {
    for (property, tokens) in [
        (P::ObjectPosition, vec!["right", "-1px", "bottom", "-2%"]),
        (P::ImageOrientation, vec!["flip", "-0.25turn"]),
        (P::ImageRendering, vec!["optimizespeed"]),
        (P::ObjectFit, vec!["scale-down"]),
    ] {
        let components = CssComponentValues::try_new(
            tokens
                .into_iter()
                .map(|token| CssComponentValue::try_token(token).unwrap())
                .collect(),
        )
        .unwrap();
        let before = components.clone();
        for grammar in [false, true] {
            let source = checked(property, components.clone(), grammar);
            let values = ordinary(&source);
            let item = &values.items()[0];
            terminal_matches_authored(item, &source);
            assert!(source.position().is_none());
            assert!(source.parsed_name().is_none());
            assert!(source.parsed_value().is_none());
            assert!(
                source
                    .value_components()
                    .items()
                    .iter()
                    .all(|component| component.origin() == &CssValueOrigin::Programmatic)
            );
            match item.ordinary_value().unwrap().view() {
                CssLonghandValueRef::ObjectPosition(position) => {
                    let (
                        CssHorizontalPosition::RightOffset(x),
                        CssVerticalPosition::BottomOffset(y),
                    ) = (position.horizontal(), position.vertical())
                    else {
                        panic!("programmatic paired edges")
                    };
                    assert_eq!(x.origin(), &CssValueOrigin::Programmatic);
                    assert_eq!(y.origin(), &CssValueOrigin::Programmatic);
                }
                CssLonghandValueRef::ImageOrientation(CssImageOrientation::Flip(Some(angle))) => {
                    assert_eq!(angle.origin(), &CssValueOrigin::Programmatic)
                }
                CssLonghandValueRef::ImageRendering(value) => {
                    assert_eq!(*value, CssImageRendering::OptimizeSpeed)
                }
                CssLonghandValueRef::ObjectFit(value) => {
                    assert_eq!(*value, CssObjectFit::ScaleDown)
                }
                _ => panic!("exact programmatic Images terminal"),
            }
            assert_eq!(components, before);
        }
    }
}
