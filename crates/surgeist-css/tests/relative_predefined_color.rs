#![forbid(unsafe_code)]
//! Relative predefined color() syntax, CSS Color 5 WD 2026-09-08 §§5–5.1, 11.3.
//! https://www.w3.org/TR/2026/WD-css-color-5-20260908/#relative-color-function
//! Source SHA-256: f749cb75ec1c0faca7d2443546c88be4e22750526dd45f9fefcfcdd079ed3c09
//! The independent table lists seven RGB spaces and three XYZ spellings (xyz
//! aliases D65). References are numbers; direct 100% equals 1. Extended channel
//! values remain authored, and no origin conversion or calculation evaluation
//! is expected at this boundary.

use surgeist_css::*;

#[derive(Clone, Copy)]
struct Space {
    spelling: &'static str,
    value: CssPredefinedColorSpace,
    canonical: &'static str,
    xyz: bool,
}

const SPACES: [Space; 10] = [
    Space {
        spelling: "srgb",
        value: CssPredefinedColorSpace::Srgb,
        canonical: "srgb",
        xyz: false,
    },
    Space {
        spelling: "srgb-linear",
        value: CssPredefinedColorSpace::SrgbLinear,
        canonical: "srgb-linear",
        xyz: false,
    },
    Space {
        spelling: "display-p3",
        value: CssPredefinedColorSpace::DisplayP3,
        canonical: "display-p3",
        xyz: false,
    },
    Space {
        spelling: "display-p3-linear",
        value: CssPredefinedColorSpace::DisplayP3Linear,
        canonical: "display-p3-linear",
        xyz: false,
    },
    Space {
        spelling: "a98-rgb",
        value: CssPredefinedColorSpace::A98Rgb,
        canonical: "a98-rgb",
        xyz: false,
    },
    Space {
        spelling: "prophoto-rgb",
        value: CssPredefinedColorSpace::ProphotoRgb,
        canonical: "prophoto-rgb",
        xyz: false,
    },
    Space {
        spelling: "rec2020",
        value: CssPredefinedColorSpace::Rec2020,
        canonical: "rec2020",
        xyz: false,
    },
    Space {
        spelling: "xyz",
        value: CssPredefinedColorSpace::XyzD65,
        canonical: "xyz-d65",
        xyz: true,
    },
    Space {
        spelling: "xyz-d50",
        value: CssPredefinedColorSpace::XyzD50,
        canonical: "xyz-d50",
        xyz: true,
    },
    Space {
        spelling: "xyz-d65",
        value: CssPredefinedColorSpace::XyzD65,
        canonical: "xyz-d65",
        xyz: true,
    },
];

impl Space {
    fn environment(self) -> CssRelativeColorEnvironment {
        if self.xyz {
            CssRelativeColorEnvironment::Xyz(self.value)
        } else {
            CssRelativeColorEnvironment::PredefinedRgb(self.value)
        }
    }
    fn keywords(self) -> [(&'static str, CssRelativeColorChannel); 3] {
        use CssRelativeColorChannel::*;
        if self.xyz {
            [("x", X), ("y", Y), ("z", Z)]
        } else {
            [("r", R), ("g", G), ("b", B)]
        }
    }
}

fn parsed(text: &str) -> CssColor {
    let source = format!("color:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    assert!(validate_style_attribute(&source).is_ok());
    let CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("color declaration")
    };
    value.value().clone()
}

fn expression(value: CssComponentValue, space: Space, alpha: bool) -> CssRelativeColorExpression {
    CssRelativeColorExpression::try_from_components(
        CssComponentValues::try_new(vec![value]).unwrap(),
        space.environment(),
        if alpha {
            CssRelativeColorResultDomain::Alpha
        } else {
            CssRelativeColorResultDomain::NumberPercentage
        },
    )
    .unwrap()
}
fn direct(text: &str, space: Space, alpha: bool) -> CssRelativeColorExpression {
    expression(CssComponentValue::try_token(text).unwrap(), space, alpha)
}
fn math(
    channel: &str,
    operator: &str,
    operand: &str,
    space: Space,
    alpha: bool,
) -> CssRelativeColorExpression {
    let values = CssComponentValues::try_new(
        [channel, " ", operator, " ", operand]
            .into_iter()
            .map(|token| CssComponentValue::try_token(token).unwrap())
            .collect(),
    )
    .unwrap();
    expression(
        CssComponentValue::try_function("calc", values).unwrap(),
        space,
        alpha,
    )
}
fn constructed(
    space: Space,
    channels: [CssRelativeColorExpression; 3],
    alpha: Option<CssRelativeColorExpression>,
) -> CssColor {
    CssColor::from_relative(
        CssRelativeColor::try_new(
            CssRelativeColorFunction::Color(space.value),
            CssColor::from_named(CssNamedColor::try_new("red").unwrap()),
            channels,
            alpha,
        )
        .unwrap(),
    )
}
fn signature(color: &CssColor, space: Space) -> &CssRelativeColor {
    let value = color.relative_value().unwrap();
    assert_eq!(
        value.function(),
        &CssRelativeColorFunction::Color(space.value)
    );
    assert_eq!(value.environment(), space.environment());
    assert_eq!(value.source().named().unwrap().name(), "red");
    for channel in value.channels() {
        assert_eq!(channel.environment(), space.environment());
        assert_eq!(
            channel.result_domain(),
            CssRelativeColorResultDomain::NumberPercentage
        );
    }
    if let Some(alpha) = value.alpha() {
        assert_eq!(alpha.environment(), space.environment());
        assert_eq!(alpha.result_domain(), CssRelativeColorResultDomain::Alpha);
    }
    value
}
fn projection_unchanged(color: &CssColor, expected: &str) {
    let before = color.clone();
    assert_eq!(color.to_specified_css().unwrap(), expected);
    assert_eq!(color, &before);
}
fn channel(value: &CssRelativeColorExpression, expected: CssRelativeColorChannel) {
    assert!(
        matches!(value.value(), CssRelativeColorExpressionValue::Channel(actual) if *actual == expected)
    );
}

#[test]
fn every_space_preserves_reference_permutations_and_programmatic_identity() {
    for space in SPACES {
        let [(first, first_id), (second, second_id), (third, third_id)] = space.keywords();
        let text = format!(
            "color(from red {} alpha {third} {first} / {second})",
            space.spelling
        );
        let checked = constructed(
            space,
            [
                direct("alpha", space, false),
                direct(third, space, false),
                direct(first, space, false),
            ],
            Some(direct(second, space, true)),
        );
        for (color, programmatic) in [(parsed(&text), false), (checked, true)] {
            let relative = signature(&color, space);
            channel(&relative.channels()[0], CssRelativeColorChannel::Alpha);
            channel(&relative.channels()[1], third_id);
            channel(&relative.channels()[2], first_id);
            channel(relative.alpha().unwrap(), second_id);
            for value in relative.channels().iter().chain(relative.alpha()) {
                assert_eq!(
                    matches!(value.origin(), CssValueOrigin::Programmatic),
                    programmatic
                );
            }
            projection_unchanged(
                &color,
                &format!(
                    "color(from red {} alpha {third} {first} / {second})",
                    space.canonical
                ),
            );
        }
    }
}

#[test]
fn direct_extended_values_missing_and_alpha_remain_typed_before_projection() {
    for space in SPACES {
        for (alpha, suffix) in [
            (None, ""),
            (Some("none"), " / none"),
            (Some("0.25"), " / 0.25"),
            (Some("25%"), " / 0.25"),
            (Some("200%"), " / 1"),
            (Some("-1"), " / 0"),
        ] {
            let authored_alpha = alpha.map_or(String::new(), |value| format!(" / {value}"));
            let text = format!(
                "color(from red {} -20% 2 none{authored_alpha})",
                space.spelling
            );
            let checked = constructed(
                space,
                [
                    direct("-20%", space, false),
                    direct("2", space, false),
                    direct("none", space, false),
                ],
                alpha.map(|value| direct(value, space, true)),
            );
            for color in [parsed(&text), checked] {
                let relative = signature(&color, space);
                assert!(
                    matches!(relative.channels()[0].value(), CssRelativeColorExpressionValue::Percentage(value) if value.numeric().representation() == "-20")
                );
                assert!(
                    matches!(relative.channels()[1].value(), CssRelativeColorExpressionValue::Number(value) if value.numeric().representation() == "2")
                );
                assert!(matches!(
                    relative.channels()[2].value(),
                    CssRelativeColorExpressionValue::None
                ));
                match (
                    alpha,
                    relative.alpha().map(CssRelativeColorExpression::value),
                ) {
                    (None, None) | (Some("none"), Some(CssRelativeColorExpressionValue::None)) => {}
                    (Some("25%"), Some(CssRelativeColorExpressionValue::Percentage(value))) => {
                        assert_eq!(value.numeric().representation(), "25")
                    }
                    (Some("200%"), Some(CssRelativeColorExpressionValue::Percentage(value))) => {
                        assert_eq!(value.numeric().representation(), "200")
                    }
                    (Some(expected), Some(CssRelativeColorExpressionValue::Number(value))) => {
                        assert_eq!(value.numeric().representation(), expected)
                    }
                    _ => panic!("alpha identity"),
                }
                projection_unchanged(
                    &color,
                    &format!("color(from red {} -0.2 2 none{suffix})", space.canonical),
                );
            }
        }
    }
}

#[test]
fn reference_math_retains_number_and_percentage_result_types_in_every_space() {
    for space in SPACES {
        let [(first, first_id), (second, second_id), (third, _)] = space.keywords();
        let text = format!(
            "color(from red {} calc({first} + 1) calc({second} * 1%) {third} / calc(alpha * 0.5))",
            space.spelling
        );
        let checked = constructed(
            space,
            [
                math(first, "+", "1", space, false),
                math(second, "*", "1%", space, false),
                direct(third, space, false),
            ],
            Some(math("alpha", "*", "0.5", space, true)),
        );
        for (color, programmatic) in [(parsed(&text), false), (checked, true)] {
            let relative = signature(&color, space);
            for (expression, result_type, reference, authored) in [
                (
                    &relative.channels()[0],
                    CssCalculationType::Number,
                    first_id,
                    format!("calc({first} + 1)"),
                ),
                (
                    &relative.channels()[1],
                    CssCalculationType::Percentage,
                    second_id,
                    format!("calc({second} * 1%)"),
                ),
                (
                    relative.alpha().unwrap(),
                    CssCalculationType::Number,
                    CssRelativeColorChannel::Alpha,
                    "calc(alpha * 0.5)".to_owned(),
                ),
            ] {
                let CssRelativeColorExpressionValue::Calculation(value) = expression.value() else {
                    panic!("calculation")
                };
                assert_eq!(value.result_type(), result_type);
                assert_eq!(value.references(), &[reference]);
                assert_eq!(
                    matches!(value.expression().origin(), CssValueOrigin::Programmatic),
                    programmatic
                );
                assert_eq!(value.authored().as_css(), authored);
            }
            let before = color.clone();
            // The selected specified phase retains unresolved reference math.
            assert!(color.to_specified_css().unwrap().contains("calc("));
            assert_eq!(color, before);
        }
    }
}

#[test]
fn case_and_escaped_identifiers_keep_original_spans_and_canonical_keywords() {
    for space in SPACES {
        let [(first, first_id), (second, second_id), (third, third_id)] = space.keywords();
        let escaped_space = format!(
            "\\{:x} {}",
            space.spelling.as_bytes()[0],
            &space.spelling[1..]
        );
        let escaped_first = format!("\\{:x}", first.as_bytes()[0]);
        // A hexadecimal escape consumes its optional whitespace terminator.
        let escaped_first_token = format!("{escaped_first} ");
        let text = format!(
            "c\\6f lor(FROM RED {escaped_space} {escaped_first}  {} {} / ALPHA)",
            second.to_ascii_uppercase(),
            third.to_ascii_uppercase()
        );
        let color = parsed(&text);
        let relative = signature(&color, space);
        for (value, expected, authored) in [
            (
                &relative.channels()[0],
                first_id,
                escaped_first_token.as_str(),
            ),
            (
                &relative.channels()[1],
                second_id,
                second.to_ascii_uppercase().as_str(),
            ),
            (
                &relative.channels()[2],
                third_id,
                third.to_ascii_uppercase().as_str(),
            ),
            (
                relative.alpha().unwrap(),
                CssRelativeColorChannel::Alpha,
                "ALPHA",
            ),
        ] {
            channel(value, expected);
            let CssValueOrigin::Parsed(origin) = value.origin() else {
                panic!("parsed provenance")
            };
            assert_eq!(origin.source().as_str(), format!("color:{text}"));
            assert_eq!(
                &origin.source().as_str()[origin.span().start().byte_offset().value()
                    ..origin.span().end().byte_offset().value()],
                authored
            );
        }
        projection_unchanged(
            &color,
            &format!(
                "color(from red {} {first} {second} {third} / alpha)",
                space.canonical
            ),
        );
    }
}

#[test]
fn nested_and_contextual_origins_are_retained_without_conversion() {
    for space in SPACES {
        let text = format!(
            "color(from color(display-p3 -0.2 1.2 0.3 / 0.5) {} none 0 1)",
            space.spelling
        );
        let color = parsed(&text);
        let relative = color.relative_value().unwrap();
        let origin = relative.source().predefined_value().unwrap();
        assert_eq!(origin.color_space(), CssPredefinedColorSpace::DisplayP3);
        assert!(
            matches!(&origin.channels()[0], CssColorComponent::Number(value) if value.numeric().representation() == "-0.2")
        );
        assert!(
            matches!(&origin.channels()[1], CssColorComponent::Number(value) if value.numeric().representation() == "1.2")
        );
        projection_unchanged(
            &color,
            &format!(
                "color(from color(display-p3 -0.2 1.2 0.3 / 0.5) {} none 0 1)",
                space.canonical
            ),
        );
        let checked = CssColor::from_relative(
            CssRelativeColor::try_new(
                CssRelativeColorFunction::Color(space.value),
                CssColor::current_color(),
                [
                    direct("none", space, false),
                    direct("0", space, false),
                    direct("1", space, false),
                ],
                None,
            )
            .unwrap(),
        );
        assert_eq!(
            checked.relative_value().unwrap().source(),
            &CssColor::current_color()
        );
        projection_unchanged(
            &checked,
            &format!("color(from currentcolor {} none 0 1)", space.canonical),
        );
    }
}

fn rejected(text: &str) {
    let source = format!("color:{text};opacity:0.5");
    let report = parse_style_attribute(&source);
    assert_eq!(report.diagnostics().len(), 1, "{source}");
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    // The dropped declaration includes its terminating semicolon.
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        "color:".len() + text.len() + 1
    );
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    let failure = validate_style_attribute(&source).unwrap_err();
    assert_eq!(failure.diagnostics(), report.diagnostics());
}

#[test]
fn foreign_channels_dimensions_and_malformed_signatures_drop_only_color() {
    for space in SPACES {
        let [(first, _), (second, _), (third, _)] = space.keywords();
        let foreign = if space.xyz { "r" } else { "x" };
        for arguments in [
            format!("{foreign} {second} {third}"),
            format!("calc({foreign} + 1) {second} {third}"),
            format!("calc({first} + 1%) {second} {third}"),
            format!("calc({first} * 1deg) {second} {third}"),
            format!("1px {second} {third}"),
            format!("{first} {second}"),
            format!("{first} {second} {third} 0"),
            format!("{first}, {second}, {third}"),
            format!("{first} {second} {third} / 1 0"),
            format!("{first} {second} {third} / {foreign}"),
        ] {
            rejected(&format!("color(from red {} {arguments})", space.spelling));
        }
    }
    for name in ["srgb2", "xyz-d60", "unknown"] {
        rejected(&format!("color(from red {name} 0 0 0)"));
    }
    // Dashed profiles are a separate valid authored branch, not unknown spaces.
    let custom = parsed("color(from red --profile 0 0 0)");
    assert!(custom.relative_value().is_none());
}

#[test]
fn checked_payloads_reject_environment_and_result_slot_mismatches() {
    for space in SPACES {
        let other = if space.xyz { SPACES[0] } else { SPACES[8] };
        let channels = [
            direct("0", space, false),
            direct("0", space, false),
            direct("0", space, false),
        ];
        for (channels, alpha) in [
            (
                [
                    direct("0", other, false),
                    channels[1].clone(),
                    channels[2].clone(),
                ],
                None,
            ),
            (
                [
                    direct("0", space, true),
                    channels[1].clone(),
                    channels[2].clone(),
                ],
                None,
            ),
            (channels.clone(), Some(direct("0", space, false))),
            (channels, Some(direct("0", other, true))),
        ] {
            assert_eq!(
                CssRelativeColor::try_new(
                    CssRelativeColorFunction::Color(space.value),
                    CssColor::current_color(),
                    channels,
                    alpha
                )
                .unwrap_err(),
                CssColorConstructionError::InvalidExpressionEnvironment
            );
        }
        for token in ["1deg", "1px", "1s"] {
            assert!(
                CssRelativeColorExpression::try_from_components(
                    CssComponentValues::try_new(vec![CssComponentValue::try_token(token).unwrap()])
                        .unwrap(),
                    space.environment(),
                    CssRelativeColorResultDomain::NumberPercentage
                )
                .is_err()
            );
        }
    }
}

fn assert_transport(expansion: &CssExpansion, declaration: &CssDeclaration) {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = expansion else {
        panic!("terminal color expansion")
    };
    assert_eq!(values.items().len(), 1);
    assert!(values.items()[0].source().same_occurrence(declaration));
    let CssLonghandValueRef::Color(actual) = values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("typed color contribution")
    };
    let CssKnownPropertyValueRef::Color(expected) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("source color")
    };
    assert_eq!(actual, expected.value());
    let relative = actual.relative_value().unwrap();
    assert!(
        matches!(relative.channels()[0].value(), CssRelativeColorExpressionValue::Percentage(value) if value.numeric().representation() == "-20")
    );
    assert!(matches!(
        relative.channels()[1].value(),
        CssRelativeColorExpressionValue::None
    ));
    let CssRelativeColorExpressionValue::Calculation(alpha) = relative.alpha().unwrap().value()
    else {
        panic!("retained alpha math")
    };
    assert_eq!(alpha.references(), &[CssRelativeColorChannel::Alpha]);
    assert!(matches!(
        alpha.expression().origin(),
        CssValueOrigin::Parsed(_)
    ));
}

#[test]
fn expanded_and_normalized_transport_preserves_relative_graph_and_occurrence() {
    for space in SPACES {
        let text = format!(
            "color(from red {} -20% none 2 / calc(alpha * 0.5))",
            space.spelling
        );
        let report = parse_style_attribute(&format!("color:{text}"));
        assert!(report.is_clean());
        let declaration = &report.syntax()[0];
        assert_transport(&expand_declaration(declaration).unwrap(), declaration);
        let report = parse_sheet(&format!(".a{{color:{text}}}"));
        assert!(report.is_clean());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 1);
        assert_transport(declarations[0].expansion(), declarations[0].source());
    }
}
