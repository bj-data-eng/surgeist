#![forbid(unsafe_code)]
//! Compositing 1 CRD 2024-03-21 §§3.1, 3.4.2, 3.4.3 defines the selected
//! sixteen blend modes, noninherited normal/auto/normal initials and Background's
//! reset of background-blend-mode. Cascade 5 CR 2022-01-13 §3.1 propagates
//! CSS-wide values to reset-only members. Provenance, typed borrowed payloads
//! and cumulative atomic limits are Surgeist authored-layer contracts.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const FAMILY: [(P, &str); 3] = [
    (P::BackgroundBlendMode, "hue, normal, hue"),
    (P::Isolation, "isolate"),
    (P::MixBlendMode, "multiply"),
];
const MODES: [(&str, CssBlendMode); 16] = [
    ("normal", CssBlendMode::Normal),
    ("darken", CssBlendMode::Darken),
    ("multiply", CssBlendMode::Multiply),
    ("color-burn", CssBlendMode::ColorBurn),
    ("lighten", CssBlendMode::Lighten),
    ("screen", CssBlendMode::Screen),
    ("color-dodge", CssBlendMode::ColorDodge),
    ("overlay", CssBlendMode::Overlay),
    ("soft-light", CssBlendMode::SoftLight),
    ("hard-light", CssBlendMode::HardLight),
    ("difference", CssBlendMode::Difference),
    ("exclusion", CssBlendMode::Exclusion),
    ("hue", CssBlendMode::Hue),
    ("saturation", CssBlendMode::Saturation),
    ("color", CssBlendMode::Color),
    ("luminosity", CssBlendMode::Luminosity),
];
const BACKGROUND: [P; 9] = [
    P::BackgroundImage,
    P::BackgroundPosition,
    P::BackgroundSize,
    P::BackgroundRepeat,
    P::BackgroundAttachment,
    P::BackgroundOrigin,
    P::BackgroundClip,
    P::BackgroundColor,
    P::BackgroundBlendMode,
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn parsed(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}
fn checked(property: P, css: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        property.grammar(),
        parse_component_values(css).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}
fn context(
    item: &CssLonghandContribution,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(item.replacement_components(), replacement);
    for (a, b) in item
        .source()
        .value_components()
        .items()
        .iter()
        .zip(source.value_components().items())
    {
        assert_eq!(a, b);
        if let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (a.origin(), b.origin()) {
            assert!(a.source().same_snapshot(b.source()));
            assert_eq!(a.span(), b.span());
        }
    }
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed longhands")
    };
    for item in values.items() {
        context(item, source, None);
    }
    values
}
fn one(values: &CssLonghandContributions, property: P) -> CssLonghandValueRef<'_> {
    let [item] = values.items() else {
        panic!("one family longhand")
    };
    assert_eq!(item.property(), property);
    assert_eq!(
        item.ordinary_value().unwrap().property().known_property(),
        property
    );
    item.ordinary_value().unwrap().view()
}
fn output(
    value: CssLonghandValueRef<'_>,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match value {
        CssLonghandValueRef::BackgroundBlendMode(v) => v.serialize_specified_with_limits(limits),
        CssLonghandValueRef::Isolation(v) => v.serialize_specified_with_limits(limits),
        CssLonghandValueRef::MixBlendMode(v) => v.serialize_specified_with_limits(limits),
        other => panic!("compositing payload: {other:?}"),
    }
}
fn implicit_origin(components: &CssComponentValues, original: &str) -> CssValueOrigin {
    // Actual emitted mappings retain original EOF; repaired comment text does
    // not establish an origin by its length or require a closing suffix.
    let serialized = components.serialize().unwrap();
    let origin = (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original implicit closure");
    let CssValueOrigin::ImplicitClosure { at, .. } = &origin else {
        unreachable!()
    };
    assert_eq!(at.source().as_str(), original);
    assert_eq!(at.span().start().byte_offset().value(), original.len());
    assert_eq!(at.span().start(), at.span().end());
    origin
}
fn reset(values: &CssLonghandContributions) -> &CssBlendModeList {
    assert_eq!(
        values
            .items()
            .iter()
            .map(CssLonghandContribution::property)
            .collect::<Vec<_>>(),
        BACKGROUND
    );
    let CssLonghandValueRef::BackgroundBlendMode(value) =
        values.items()[8].ordinary_value().unwrap().view()
    else {
        panic!("typed reset-only blend list")
    };
    assert_eq!(value.modes(), &[CssBlendMode::Normal]);
    value
}

#[test]
fn three_borrowed_initial_views_have_noninherited_exact_scalar_contents() {
    for (property, _) in FAMILY {
        let metadata = property.metadata().unwrap();
        assert_eq!(metadata.grammar(), property.grammar());
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("longhand")
        };
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary initial")
        };
        match initial.view() {
            CssLonghandValueRef::BackgroundBlendMode(v) => {
                assert_eq!(v.modes(), &[CssBlendMode::Normal])
            }
            CssLonghandValueRef::Isolation(v) => assert_eq!(*v, CssIsolation::Auto),
            CssLonghandValueRef::MixBlendMode(v) => assert_eq!(*v, CssBlendMode::Normal),
            other => panic!("compositing initial: {other:?}"),
        }
        let css = if property == P::Isolation {
            "auto"
        } else {
            "normal"
        };
        assert_eq!(
            output(
                initial.view(),
                CssSpecifiedValueSerializationLimits::default()
            )
            .unwrap(),
            css
        );
        let explicit = completed(&checked(property, css));
        assert_eq!(initial.view(), one(&explicit, property));
    }
}

#[test]
fn all_selected_blend_and_isolation_keywords_reach_typed_views_through_three_fronts() {
    for (property, cases) in [
        (
            P::MixBlendMode,
            MODES.iter().map(|(css, _)| *css).collect::<Vec<_>>(),
        ),
        (
            P::BackgroundBlendMode,
            MODES.iter().map(|(css, _)| *css).collect::<Vec<_>>(),
        ),
        (P::Isolation, vec!["auto", "isolate"]),
    ] {
        for css in cases {
            let original = parsed(&format!(
                "/*😀*/{}:{}!important",
                property.canonical_name(),
                css.to_ascii_uppercase()
            ));
            let grammar = checked(property, css);
            let named = parse_property_value(
                CssPropertyNameRef::Known(property),
                parse_component_values(css).unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            assert!(original.position().is_some());
            for source in [&grammar, &named] {
                assert!(source.position().is_none());
            }
            for source in [&original, &grammar, &named] {
                let values = completed(source);
                let view = one(&values, property);
                match view {
                    CssLonghandValueRef::MixBlendMode(v) => {
                        assert_eq!(*v, MODES.iter().find(|(text, _)| *text == css).unwrap().1)
                    }
                    CssLonghandValueRef::BackgroundBlendMode(v) => assert_eq!(
                        v.modes(),
                        &[MODES.iter().find(|(text, _)| *text == css).unwrap().1]
                    ),
                    CssLonghandValueRef::Isolation(v) => assert_eq!(
                        *v,
                        if css == "auto" {
                            CssIsolation::Auto
                        } else {
                            CssIsolation::Isolate
                        }
                    ),
                    other => panic!("family payload: {other:?}"),
                }
                assert_eq!(
                    output(view, CssSpecifiedValueSerializationLimits::default()).unwrap(),
                    css
                );
            }
        }
    }
}

#[test]
fn sixty_four_authored_blend_entries_keep_every_index_duplicate_and_decoded_order() {
    let css = (0usize..64)
        .map(|index| {
            if index.is_multiple_of(16) {
                r"N\6f rmal".to_owned()
            } else {
                MODES[index % 16].0.to_ascii_uppercase()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let expected_modes = (0..64).map(|index| MODES[index % 16].1).collect::<Vec<_>>();
    let expected_css = (0..64)
        .map(|index| MODES[index % 16].0)
        .collect::<Vec<_>>()
        .join(", ");
    for source in [
        parsed(&format!("background-blend-mode:{css}!important")),
        checked(P::BackgroundBlendMode, &css),
        parse_property_value(
            CssPropertyNameRef::Known(P::BackgroundBlendMode),
            parse_component_values(&css).unwrap(),
            CssImportance::Important,
        )
        .unwrap(),
    ] {
        let values = completed(&source);
        let CssLonghandValueRef::BackgroundBlendMode(value) = one(&values, P::BackgroundBlendMode)
        else {
            panic!("blend list")
        };
        assert_eq!(value.modes().len(), 64);
        for (index, mode) in value.modes().iter().enumerate() {
            assert_eq!(*mode, expected_modes[index]);
        }
        assert_eq!(value.modes(), expected_modes);
        assert_eq!(value.serialize_specified().unwrap(), expected_css);
        let CssKnownPropertyValueRef::BackgroundBlendMode(original) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("original list")
        };
        assert_eq!(value, original.modes());
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    65,
                    65,
                    expected_css.len()
                ))
                .unwrap(),
            expected_css
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    64,
                    65,
                    expected_css.len()
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
        assert_eq!(value.modes(), expected_modes);
    }
}

#[test]
fn checked_programmatic_keywords_preserve_component_origins_and_original_grammar() {
    for (property, css) in FAMILY {
        let parsed_components = parse_component_values(css).unwrap();
        let mut items = parsed_components.items().to_vec();
        let last = items
            .iter()
            .rposition(|v| {
                matches!(
                    v.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Ident(_))
                )
            })
            .unwrap();
        let keyword = if property == P::BackgroundBlendMode {
            "HUE"
        } else if property == P::Isolation {
            "ISOLATE"
        } else {
            "MULTIPLY"
        };
        let programmatic = CssComponentValue::try_ident(keyword).unwrap();
        items[last] = programmatic.clone();
        let components = CssComponentValues::try_new(items).unwrap();
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
            assert_eq!(source.value_components(), &components);
            assert_eq!(source.value_components().items()[last], programmatic);
            assert_eq!(
                source.value_components().items()[last].origin(),
                &CssValueOrigin::Programmatic
            );
            let values = completed(&source);
            assert_eq!(
                output(
                    one(&values, property),
                    CssSpecifiedValueSerializationLimits::default()
                )
                .unwrap(),
                css
            );
            if property == P::BackgroundBlendMode {
                assert!(matches!(
                    source.value_components().items()[0].origin(),
                    CssValueOrigin::Parsed(_)
                ));
            }
        }
    }
}

#[test]
fn pending_family_handles_retry_strict_original_eof_grammar_residual_and_provider_failures() {
    for (property, good) in FAMILY {
        let programmatic = CssComponentValues::try_new(vec![
            CssComponentValue::try_function(
                "var",
                CssComponentValues::try_new(vec![CssComponentValue::try_ident("--mode").unwrap()])
                    .unwrap(),
            )
            .unwrap(),
        ])
        .unwrap();
        for source in [
            parsed(&format!(
                "/*😀*/{}:var(--mode)!important",
                property.canonical_name()
            )),
            parse_property_value_for_grammar(
                property.grammar(),
                programmatic.clone(),
                CssImportance::Important,
            )
            .unwrap(),
        ] {
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            let recovered_text = format!("{good}/*");
            let recovered = parse_component_values(&recovered_text).unwrap();
            let origin = implicit_origin(&recovered, &recovered_text);
            let before = recovered.clone();
            let error = handle.reenter(recovered.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("original grammar failure")
            };
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(error.origin(), &CssSerializedOrigin::End(Some(origin)));
            assert_eq!(recovered, before);
            let invalid = parse_component_values("plus-lighter").unwrap();
            let expected = parse_property_value_for_grammar(
                property.grammar(),
                invalid.clone(),
                CssImportance::Normal,
            )
            .unwrap_err();
            let error = handle.reenter(invalid).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("same invalid grammar")
            };
            assert_eq!(error, &expected);
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement = parse_component_values(good).unwrap();
            let CssContributions::Longhands(first) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed replacement")
            };
            assert_eq!(
                output(
                    one(&first, property),
                    CssSpecifiedValueSerializationLimits::new(100, 100, 0)
                )
                .unwrap_err()
                .kind(),
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            );
            let CssContributions::Longhands(retry) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("reusable handle")
            };
            assert_eq!(one(&first, property), one(&retry, property));
            assert_eq!(
                output(
                    one(&retry, property),
                    CssSpecifiedValueSerializationLimits::default()
                )
                .unwrap(),
                good
            );
            context(&retry.items()[0], &source, Some(&replacement));
            for (actual, expected) in retry.items()[0]
                .replacement_components()
                .unwrap()
                .items()
                .iter()
                .zip(replacement.items())
            {
                assert_eq!(actual, expected);
                if let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) =
                    (actual.origin(), expected.origin())
                {
                    assert!(a.source().same_snapshot(b.source()));
                }
            }
        }
    }
}

#[test]
fn projected_family_payloads_and_singleton_reset_keep_exact_provider_budgets() {
    let background = completed(&checked(P::Background, "none, none red"));
    assert_eq!(reset(&background).modes().len(), 1);
    let values = FAMILY
        .into_iter()
        .map(|(property, css)| (property, completed(&checked(property, css))))
        .collect::<Vec<_>>();
    for (view, nodes, css) in [
        (one(&values[0].1, values[0].0), 4, "hue, normal, hue"),
        (one(&values[1].1, values[1].0), 1, "isolate"),
        (one(&values[2].1, values[2].0), 1, "multiply"),
        (
            background.items()[8].ordinary_value().unwrap().view(),
            2,
            "normal",
        ),
    ] {
        let limits = CssSpecifiedValueSerializationLimits::new;
        assert_eq!(output(view, limits(nodes, nodes, css.len())).unwrap(), css);
        for (budget, kind) in [
            (
                limits(nodes - 1, nodes, css.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                limits(nodes, nodes - 1, css.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                limits(nodes, nodes, css.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(output(view, budget).unwrap_err().kind(), kind);
            assert_eq!(output(view, limits(nodes, nodes, css.len())).unwrap(), css);
        }
    }
}

#[test]
fn ordinary_background_resets_to_one_normal_independently_of_layer_or_explicit_list_length() {
    let explicit_source = parsed("background-blend-mode:screen, multiply, screen!important");
    let explicit = completed(&explicit_source);
    let CssLonghandValueRef::BackgroundBlendMode(list) = one(&explicit, P::BackgroundBlendMode)
    else {
        panic!("explicit blend list")
    };
    assert_eq!(
        list.modes(),
        &[
            CssBlendMode::Screen,
            CssBlendMode::Multiply,
            CssBlendMode::Screen
        ]
    );
    for count in [1, 2, 20] {
        let css = (0..count)
            .map(|index| if index + 1 == count { "red" } else { "none" })
            .collect::<Vec<_>>()
            .join(", ");
        for source in [
            parsed(&format!("background:{css}!important")),
            checked(P::Background, &css),
        ] {
            let values = completed(&source);
            assert_eq!(reset(&values).modes(), &[CssBlendMode::Normal]);
            let CssLonghandValueRef::BackgroundImage(images) =
                values.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("original background layer projection")
            };
            assert_eq!(images.images().len(), count);
            let CssLonghandValueRef::BackgroundColor(color) =
                values.items()[7].ordinary_value().unwrap().view()
            else {
                panic!("final color stays seventh index")
            };
            assert_eq!(color.to_specified_css().unwrap(), "red");
            let CssKnownPropertyValueRef::Background(original) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("original background")
            };
            assert_eq!(original.background().layers().len(), count);
            assert_eq!(original.background().serialize_specified().unwrap(), css);
        }
    }
    assert_eq!(
        list.modes(),
        &[
            CssBlendMode::Screen,
            CssBlendMode::Multiply,
            CssBlendMode::Screen
        ]
    );
}

#[test]
fn global_and_pending_backgrounds_keep_nine_terminals_and_reset_replacement_context() {
    let source = parsed("/*😀*/background:var(--layers)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending background")
    };
    for (css, keyword) in GLOBALS {
        let direct = checked(P::Background, css);
        let values = completed(&direct);
        assert_eq!(
            values
                .items()
                .iter()
                .map(CssLonghandContribution::property)
                .collect::<Vec<_>>(),
            BACKGROUND
        );
        for item in values.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        }
        let replacement = parse_component_values(css).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("global replacement")
        };
        assert_eq!(
            values
                .items()
                .iter()
                .map(CssLonghandContribution::property)
                .collect::<Vec<_>>(),
            BACKGROUND
        );
        for item in values.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            context(item, &source, Some(&replacement));
        }
    }
    for bad in ["red, none", "none,", "/cover"] {
        assert!(matches!(
            handle
                .reenter(parse_component_values(bad).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    assert_eq!(
        handle
            .reenter(parse_component_values("env(layers)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    let replacement =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("red").unwrap()]).unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("ordinary replacement")
        };
        assert_eq!(reset(&values).serialize_specified().unwrap(), "normal");
        for item in values.items() {
            context(item, &source, Some(&replacement));
        }
        assert_eq!(
            values.items()[8].replacement_components().unwrap().items()[0].origin(),
            &CssValueOrigin::Programmatic
        );
    }
}

#[test]
fn normalized_order_counts_the_singleton_reset_and_recovers_atomically_at_one_short() {
    let report = parse_sheet(
        ".a{background-blend-mode:screen,multiply!important;background:none, red!important}.b{isolation:var(--isolation)}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    // One explicit list, nine Background terminals, one pending occurrence.
    let exact = CssNormalizationLimits::try_new(0, 2, 3, 11).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| {
            if let CssNormalizedItem::Declaration(value) = item {
                Some(value)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (order, declaration) in declarations.iter().enumerate() {
        assert_eq!(declaration.order(), order);
    }
    let CssExpansion::Contributions(CssContributions::Longhands(explicit)) =
        declarations[0].expansion()
    else {
        panic!("explicit blend list")
    };
    let CssLonghandValueRef::BackgroundBlendMode(explicit) = one(explicit, P::BackgroundBlendMode)
    else {
        panic!("typed explicit list")
    };
    assert_eq!(
        explicit.modes(),
        &[CssBlendMode::Screen, CssBlendMode::Multiply]
    );
    let CssExpansion::Contributions(CssContributions::Longhands(background)) =
        declarations[1].expansion()
    else {
        panic!("nine Background terminals")
    };
    assert_eq!(reset(background).modes(), &[CssBlendMode::Normal]);
    for item in background.items() {
        context(item, declarations[1].source(), None);
    }
    assert!(matches!(
        declarations[2].expansion(),
        CssExpansion::Pending(_)
    ));
    for (budget, order) in [(9, 1), (10, 2)] {
        let error = normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(0, 2, 3, budget).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: budget
            }
        );
        assert_eq!(error.declaration_order(), Some(order));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(declarations[order].source())
        );
        assert_eq!(report, before);
        assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
    }
}

#[test]
fn browser_recovered_comments_preserve_typed_payloads_and_unclean_report_normalization() {
    for (property, css) in FAMILY {
        let source = format!(".a{{{}:{css}/*", property.canonical_name());
        let report = parse_sheet(&source);
        assert!(!report.is_clean());
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert!(!normalized.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declaration = normalized
            .syntax()
            .items()
            .iter()
            .find_map(|item| {
                if let CssNormalizedItem::Declaration(value) = item {
                    Some(value)
                } else {
                    None
                }
            })
            .unwrap();
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("recovered ordinary payload")
        };
        assert_eq!(
            output(
                one(values, property),
                CssSpecifiedValueSerializationLimits::default()
            )
            .unwrap(),
            css
        );
        context(&values.items()[0], declaration.source(), None);
        let components = declaration.source().value_components().clone();
        let origin = implicit_origin(&components, &source);
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Important,
            ),
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Important,
            ),
        ] {
            assert_eq!(
                result.unwrap_err().origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        let error = normalize_report_with_limits(
            &report,
            CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 0
            }
        );
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(declaration.source())
        );
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}
