#![forbid(unsafe_code)]
//! Functional typed API evidence for the selected Text4 authored contracts.
//! The existing test-only suites own preimplementation RED. These new tests
//! exercise checked constructors, preserved origins, omission views, exact typed
//! terminals and providers that did not exist at that checkpoint.

use CssSpecifiedValueSerializationErrorKind as Kind;
use CssSpecifiedValueSerializationLimits as Limits;
use surgeist_css::*;

#[path = "common/authored_property.rs"]
mod authored_property;
use authored_property::ParserFront;

const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];

// The grammar admits permutations of one contiguous unordered group. The caller
// supplies each group and its canonical spelling independently of the parser.
fn permutations(words: &[&str]) -> Vec<String> {
    if words.is_empty() {
        return vec![String::new()];
    }
    let mut result = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let rest: Vec<_> = words
            .iter()
            .enumerate()
            .filter_map(|(i, word)| (i != index).then_some(*word))
            .collect();
        for suffix in permutations(&rest) {
            result.push(if suffix.is_empty() {
                (*word).to_owned()
            } else {
                format!("{word} {suffix}")
            });
        }
    }
    result
}

fn declaration(name: &str, input: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("/*😀*/{name}:{input}!important"));
    assert!(
        report.is_clean(),
        "{name}:{input}: {:?}",
        report.diagnostics()
    );
    report.syntax()[0].clone()
}
fn components(input: &str) -> CssComponentValues {
    parse_component_values(input).unwrap()
}
fn count(input: &str) -> CssHyphenateLimitCharsComponent {
    let value =
        CssIntegerLiteral::try_from_component(CssComponentValue::try_number(input).unwrap())
            .unwrap();
    CssHyphenateLimitCharsComponent::Integer(
        CssHyphenateLimitInteger::try_new(CssIntegerValue::Literal(value)).unwrap(),
    )
}
macro_rules! limits {
    ($value:expr, $expected:expr, $nodes:expr) => {{
        limits!($value, $expected, $nodes, $nodes);
    }};
    ($value:expr, $expected:expr, $input_nodes:expr, $projection_nodes:expr) => {{
        let value = $value;
        let before = value.clone();
        let expected: &str = $expected;
        let input_nodes = $input_nodes;
        let projection_nodes = $projection_nodes;
        let exact = Limits::new(input_nodes, projection_nodes, expected.len());
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(input_nodes - 1, projection_nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(input_nodes, projection_nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(input_nodes, projection_nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    value
                        .serialize_specified_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(value, before);
            }
        }
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
    }};
}
#[test]
fn checked_hyphenation_strings_keep_decoded_content_component_and_original_origin() {
    for (input, decoded, expected) in [
        (r#""""#, "", r#""""#),
        (r#""😀᐀""#, "😀᐀", r#""😀᐀""#),
        (r#""\2010 ""#, "‐", r#""‐""#),
        (r#"'a"b'"#, "a\"b", r#""a\"b""#),
        (r#""a\\b""#, "a\\b", r#""a\\b""#),
    ] {
        let source = declaration("hyphenate-character", input);
        let CssKnownPropertyValueRef::HyphenateCharacter(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed character")
        };
        let CssHyphenateCharacter::String(value) = wrapper.value() else {
            panic!("authored string")
        };
        assert_eq!(value.as_str(), decoded);
        let token = source
            .value_components()
            .items()
            .iter()
            .find(|v| {
                matches!(
                    v.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::String(_))
                )
            })
            .unwrap();
        assert_eq!(value.component(), token);
        assert_eq!(value.origin(), token.origin());
        let CssValueOrigin::Parsed(origin) = value.origin() else {
            panic!("original parsed string")
        };
        assert_eq!(
            origin.source().as_str(),
            source.parsed_value().unwrap().source().as_str()
        );
        limits!(value.clone(), expected, 1);
        let programmatic = CssHyphenateString::try_new(decoded).unwrap();
        assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(programmatic.serialize_specified().unwrap(), expected);
    }
    assert!(CssHyphenateString::try_new("a\0b").is_err());
    assert!(
        CssHyphenateString::try_from_component(CssComponentValue::try_ident("auto").unwrap())
            .is_none()
    );
    limits!(CssHyphenateCharacter::Auto, "auto", 1);
}
#[test]
fn nonnegative_integer_constructor_rechecks_bare_roots_without_clamping_function_math() {
    for (input, expected) in [
        ("0", "0"),
        ("-0", "0"),
        ("+0000", "0"),
        ("+0002", "2"),
        (
            "9999999999999999999999999999999999999999",
            "9999999999999999999999999999999999999999",
        ),
    ] {
        let literal =
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(input).unwrap())
                .unwrap();
        let origin = literal.origin().clone();
        let integer = CssHyphenateLimitInteger::try_new(CssIntegerValue::Literal(literal)).unwrap();
        assert_eq!(integer.origin(), &origin);
        assert!(matches!(integer.value(), CssIntegerValue::Literal(_)));
        limits!(integer, expected, 1);
        let bare = CssIntegerCalculation::try_from_components(components(input)).unwrap();
        let bare_origin = bare.origin().clone();
        let checked =
            CssHyphenateLimitInteger::try_new(CssIntegerValue::Calculation(bare)).unwrap();
        assert!(matches!(checked.value(), CssIntegerValue::Literal(_)));
        assert_eq!(checked.origin(), &bare_origin);
        let CssIntegerValue::Literal(literal) = checked.value() else {
            panic!("bare root crosses literal admission")
        };
        assert_eq!(literal.numeric().representation(), input);
        assert_eq!(checked.serialize_specified().unwrap(), expected);
    }
    for input in ["-1", "-9999999999999999999999999999999999999999"] {
        let literal =
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(input).unwrap())
                .unwrap();
        assert!(CssHyphenateLimitInteger::try_new(CssIntegerValue::Literal(literal)).is_none());
        // The shared Integer-root owner is signed. Hyphenation must re-admit
        // this exact ordinary token through its own nonnegative range boundary.
        let raw = components(input);
        let component = &raw.items()[0];
        let bare = CssIntegerCalculation::try_from_components(raw.clone()).unwrap();
        assert_eq!(bare.components(), &raw);
        assert_eq!(bare.origin(), component.origin());
        assert!(
            matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation()==input)
        );
        assert!(CssHyphenateLimitInteger::try_new(CssIntegerValue::Calculation(bare)).is_none());
    }
    for input in ["1.5", "1e0"] {
        assert!(
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(input).unwrap())
                .is_err()
        );
        // Noninteger bare tokens are rejected before a checked Integer root
        // exists; deferred fractional function math remains a separate branch.
        let error = CssIntegerCalculation::try_from_components(components(input)).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
    }
    for (input, expected) in [
        ("calc(-1)", "calc(-1)"),
        ("calc(1.5)", "calc(1.5)"),
        ("calc(infinity)", "calc(infinity)"),
        ("calc(NaN)", "calc(NaN)"),
    ] {
        let calculation = CssIntegerCalculation::try_from_components(components(input)).unwrap();
        let origin = calculation.origin().clone();
        let checked =
            CssHyphenateLimitInteger::try_new(CssIntegerValue::Calculation(calculation)).unwrap();
        assert!(matches!(checked.value(), CssIntegerValue::Calculation(_)));
        assert_eq!(checked.origin(), &origin);
        assert_eq!(checked.serialize_specified().unwrap(), expected);
    }
}
#[test]
fn chars_constructor_preserves_authored_arity_and_effective_defaults_without_new_origins() {
    assert!(CssHyphenateLimitChars::try_new(count("8"), None, Some(count("3"))).is_none());
    for (before, after, expected, arity, effective) in [
        (None, None, "8", 1, ["8", "auto", "auto"]),
        (Some(count("2")), None, "8 2", 2, ["8", "2", "2"]),
        (
            Some(CssHyphenateLimitCharsComponent::Auto),
            Some(count("3")),
            "8 auto 3",
            3,
            ["8", "auto", "3"],
        ),
    ] {
        let value = CssHyphenateLimitChars::try_new(count("8"), before, after).unwrap();
        assert_eq!(value.authored_len(), arity);
        assert_eq!(value.before().is_some(), arity > 1);
        assert_eq!(value.after().is_some(), arity > 2);
        for (slot, expected) in value.effective_components().iter().zip(effective) {
            assert_eq!(slot.serialize_specified().unwrap(), expected);
        }
        limits!(value, expected, arity);
    }
    let source = declaration("hyphenate-limit-chars", "8 2");
    let CssKnownPropertyValueRef::HyphenateLimitChars(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed slots")
    };
    let value = wrapper.value();
    let triple = value.effective_components();
    assert_eq!(&triple[0], value.total());
    assert_eq!(&triple[1], value.before().unwrap());
    assert_eq!(triple[1], triple[2]);
    let CssHyphenateLimitCharsComponent::Integer(before) = &triple[1] else {
        panic!("explicit integer")
    };
    let CssHyphenateLimitCharsComponent::Integer(after) = &triple[2] else {
        panic!("copied integer")
    };
    assert_eq!(before.origin(), after.origin());
    assert!(value.after().is_none());
    assert_eq!(
        CssHyphenateLimitChars::auto().effective_components(),
        [
            CssHyphenateLimitCharsComponent::Auto,
            CssHyphenateLimitCharsComponent::Auto,
            CssHyphenateLimitCharsComponent::Auto
        ]
    );
}
#[test]
fn finite_justification_roles_preserve_modifier_only_and_legacy_authored_base() {
    assert!(CssTextJustify::try_new(None, false).is_none());
    let modifier = CssTextJustify::try_new(None, true).unwrap();
    assert_eq!(modifier.base(), None);
    assert!(modifier.no_compress());
    limits!(modifier, "no-compress", 1);
    for (base, text) in [
        (CssTextJustifyBase::Auto, "auto"),
        (CssTextJustifyBase::None, "none"),
        (CssTextJustifyBase::InterWord, "inter-word"),
        (CssTextJustifyBase::InterCharacter, "inter-character"),
        (CssTextJustifyBase::Ruby, "ruby"),
        (CssTextJustifyBase::Distribute, "distribute"),
    ] {
        for modifier in [false, true] {
            let value = CssTextJustify::try_new(Some(base), modifier).unwrap();
            let expected = if modifier {
                format!("{text} no-compress")
            } else {
                text.to_owned()
            };
            assert_eq!(value.base(), Some(base));
            assert_eq!(value.no_compress(), modifier);
            limits!(value, expected.as_str(), if modifier { 2 } else { 1 });
            for input in [
                expected.clone(),
                if modifier {
                    format!("no-compress {text}")
                } else {
                    text.to_owned()
                },
            ] {
                let source = declaration("text-justify", &input);
                let CssKnownPropertyValueRef::TextJustify(wrapper) =
                    source.known().unwrap().property_value().unwrap()
                else {
                    panic!("typed justification")
                };
                assert_eq!(wrapper.value(), &value);
            }
        }
    }
    assert_ne!(
        CssTextJustifyBase::Distribute,
        CssTextJustifyBase::InterCharacter
    );
}
const FLAG_STATES: [(bool, bool, bool, &str); 7] = [
    (true, false, false, "ideograph-alpha"),
    (false, true, false, "ideograph-numeric"),
    (false, false, true, "punctuation"),
    (true, true, false, "ideograph-alpha ideograph-numeric"),
    (true, false, true, "ideograph-alpha punctuation"),
    (false, true, true, "ideograph-numeric punctuation"),
    (
        true,
        true,
        true,
        "ideograph-alpha ideograph-numeric punctuation",
    ),
];
#[test]
fn autospace_checked_states_preserve_authored_mode_omission_and_symbolic_insert_default() {
    assert!(CssAutospaceValues::try_new(false, false, false, None).is_none());
    for (alpha, numeric, punctuation, text) in FLAG_STATES {
        for (mode, suffix, effective) in [
            (None, "", CssAutospaceMode::Insert),
            (
                Some(CssAutospaceMode::Insert),
                " insert",
                CssAutospaceMode::Insert,
            ),
            (
                Some(CssAutospaceMode::Replace),
                " replace",
                CssAutospaceMode::Replace,
            ),
        ] {
            let value = CssAutospaceValues::try_new(alpha, numeric, punctuation, mode).unwrap();
            assert_eq!(
                (
                    value.ideograph_alpha(),
                    value.ideograph_numeric(),
                    value.punctuation()
                ),
                (alpha, numeric, punctuation)
            );
            assert_eq!(value.mode(), mode);
            assert_eq!(value.effective_mode(), effective);
            let expected = format!("{text}{suffix}");
            limits!(
                value,
                expected.as_str(),
                expected.split_whitespace().count()
            );
            let source = declaration("text-autospace", &expected);
            let CssKnownPropertyValueRef::TextAutospace(wrapper) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("typed autospace")
            };
            assert_eq!(
                wrapper.value(),
                &CssTextAutospace::Autospace(CssAutospace::Spacing(value))
            );
        }
    }
    for (mode, text) in [
        (CssAutospaceMode::Insert, "insert"),
        (CssAutospaceMode::Replace, "replace"),
    ] {
        let value = CssAutospaceValues::try_new(false, false, false, Some(mode)).unwrap();
        limits!(value, text, 1);
    }
    limits!(CssTextAutospace::Normal, "normal", 1);
    limits!(CssTextAutospace::Auto, "auto", 1);
    limits!(CssAutospace::NoAutospace, "no-autospace", 1);
}
#[test]
fn shorthand_constituents_exclude_longhand_only_choices_and_keep_explicit_omission_views() {
    assert!(CssTextSpacingValues::try_new(None, None).is_none());
    for (trim, text) in [
        (CssSpacingTrim::SpaceAll, "space-all"),
        (CssSpacingTrim::Normal, "normal"),
        (CssSpacingTrim::SpaceFirst, "space-first"),
        (CssSpacingTrim::TrimStart, "trim-start"),
        (CssSpacingTrim::TrimBoth, "trim-both"),
        (CssSpacingTrim::TrimAll, "trim-all"),
    ] {
        let value = CssTextSpacingValues::try_new(Some(trim), None).unwrap();
        assert_eq!(value.trim(), Some(trim));
        assert_eq!(value.autospace(), None);
        limits!(value, text, 1);
        let source = declaration("text-spacing", text);
        let CssKnownPropertyValueRef::TextSpacing(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed shorthand")
        };
        assert_eq!(wrapper.value(), &CssTextSpacing::Components(value));
        limits!(CssTextSpacingTrim::Trim(trim), text, 1);
    }
    let auto = CssAutospace::Spacing(
        CssAutospaceValues::try_new(false, false, false, Some(CssAutospaceMode::Replace)).unwrap(),
    );
    let value = CssTextSpacingValues::try_new(None, Some(auto)).unwrap();
    assert_eq!(value.trim(), None);
    assert_eq!(value.autospace(), Some(auto));
    limits!(CssTextSpacing::Components(value), "replace", 1);
    limits!(CssTextSpacing::None, "none", 1);
    limits!(CssTextSpacing::Auto, "auto", 1);
    limits!(CssTextSpacingTrim::Auto, "auto", 1);
}
#[test]
fn hanging_constructor_excludes_empty_roles_and_serializes_each_finite_state() {
    assert!(CssHangingPunctuationValues::try_new(false, None, false).is_none());
    for (first, end, last, text) in [
        (true, None, false, "first"),
        (false, None, true, "last"),
        (true, None, true, "first last"),
        (
            false,
            Some(CssHangingPunctuationEnd::ForceEnd),
            false,
            "force-end",
        ),
        (
            true,
            Some(CssHangingPunctuationEnd::ForceEnd),
            false,
            "first force-end",
        ),
        (
            false,
            Some(CssHangingPunctuationEnd::ForceEnd),
            true,
            "force-end last",
        ),
        (
            true,
            Some(CssHangingPunctuationEnd::ForceEnd),
            true,
            "first force-end last",
        ),
        (
            false,
            Some(CssHangingPunctuationEnd::AllowEnd),
            false,
            "allow-end",
        ),
        (
            true,
            Some(CssHangingPunctuationEnd::AllowEnd),
            false,
            "first allow-end",
        ),
        (
            false,
            Some(CssHangingPunctuationEnd::AllowEnd),
            true,
            "allow-end last",
        ),
        (
            true,
            Some(CssHangingPunctuationEnd::AllowEnd),
            true,
            "first allow-end last",
        ),
    ] {
        let value = CssHangingPunctuationValues::try_new(first, end, last).unwrap();
        assert_eq!(
            (value.first(), value.end(), value.last()),
            (first, end, last)
        );
        limits!(
            CssHangingPunctuation::Hang(value),
            text,
            text.split_whitespace().count()
        );
        let source = declaration("hanging-punctuation", text);
        let CssKnownPropertyValueRef::HangingPunctuation(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed hanging")
        };
        assert_eq!(wrapper.value(), &CssHangingPunctuation::Hang(value));
    }
    limits!(CssHangingPunctuation::None, "none", 1);
}
#[test]
fn generated_longhand_borrowed_variants_expose_all_twelve_typed_initials() {
    macro_rules! initial {
        ($property:ident,$variant:ident,$pattern:pat $(if $guard:expr)?) => {{
            let metadata=CssKnownProperty::$property.metadata().unwrap();
            let CssPropertyKindRef::Longhand(meta)=metadata.kind() else {panic!("longhand")};
            let initial=meta.initial_value();
            let CssInitialValueRef::Value(value)=initial.view() else {panic!("ordinary initial")};
            assert!(matches!(value.view(),CssLonghandValueRef::$variant($pattern) $(if $guard)?));
        }};
    }
    initial!(Hyphens, Hyphens, CssHyphens::Manual);
    initial!(
        HyphenateCharacter,
        HyphenateCharacter,
        CssHyphenateCharacter::Auto
    );
    initial!(HyphenateLimitZone,HyphenateLimitZone,value if value.literal_component().is_some_and(|component| component.origin()==&CssValueOrigin::Programmatic && matches!(component.view(),CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation()=="0")));
    initial!(HyphenateLimitChars,HyphenateLimitChars,value if value.authored_len()==1 && matches!(value.total(),CssHyphenateLimitCharsComponent::Auto));
    initial!(
        HyphenateLimitLines,
        HyphenateLimitLines,
        CssHyphenateLimitLines::NoLimit
    );
    initial!(
        HyphenateLimitLast,
        HyphenateLimitLast,
        CssHyphenateLimitLast::None
    );
    initial!(TextJustify,TextJustify,value if value.base()==Some(CssTextJustifyBase::Auto) && !value.no_compress());
    initial!(TextGroupAlign, TextGroupAlign, CssTextGroupAlign::None);
    initial!(LinePadding,LinePadding,value if value.literal_component().is_some_and(|component| component.origin()==&CssValueOrigin::Programmatic && matches!(component.view(),CssComponentValueRef::Token(CssValueTokenRef::Number(number)) if number.representation()=="0")));
    initial!(TextAutospace, TextAutospace, CssTextAutospace::Normal);
    initial!(
        TextSpacingTrim,
        TextSpacingTrim,
        CssTextSpacingTrim::Trim(CssSpacingTrim::Normal)
    );
    initial!(
        HangingPunctuation,
        HangingPunctuation,
        CssHangingPunctuation::None
    );
}

#[test]
fn finite_hyphenation_and_group_alignment_domains_keep_typed_choices_and_leaf_costs() {
    macro_rules! domain {
        ($property:ident, $variant:ident, [$(($value:expr, $text:literal)),+ $(,)?]) => {
            $(
                limits!($value, $text, 1);
                for front in FRONTS {
                    let source = front.valid(CssKnownProperty::$property, $text, $text);
                    let CssKnownPropertyValueRef::$variant(wrapper) = source.known().unwrap().property_value().unwrap() else { panic!("typed finite value") };
                    assert_eq!(wrapper.value(), &$value);
                }
            )+
        };
    }
    domain!(
        Hyphens,
        Hyphens,
        [
            (CssHyphens::None, "none"),
            (CssHyphens::Manual, "manual"),
            (CssHyphens::Auto, "auto")
        ]
    );
    domain!(
        HyphenateLimitLast,
        HyphenateLimitLast,
        [
            (CssHyphenateLimitLast::None, "none"),
            (CssHyphenateLimitLast::Always, "always"),
            (CssHyphenateLimitLast::Column, "column"),
            (CssHyphenateLimitLast::Page, "page"),
            (CssHyphenateLimitLast::Spread, "spread"),
        ]
    );
    domain!(
        TextGroupAlign,
        TextGroupAlign,
        [
            (CssTextGroupAlign::None, "none"),
            (CssTextGroupAlign::Start, "start"),
            (CssTextGroupAlign::End, "end"),
            (CssTextGroupAlign::Left, "left"),
            (CssTextGroupAlign::Right, "right"),
            (CssTextGroupAlign::Center, "center"),
        ]
    );
    limits!(CssHyphenateLimitLines::NoLimit, "no-limit", 1);
    limits!(
        CssHyphenateLimitLines::Integer(
            CssHyphenateLimitInteger::try_new(CssIntegerValue::Literal(
                CssIntegerLiteral::from_i32(2)
            ))
            .unwrap()
        ),
        "2",
        1
    );
}

#[test]
fn recovered_hyphenation_string_constructor_keeps_the_original_component_and_snapshot() {
    let css = ".a{hyphenate-character:\"😀";
    let report = parse_sheet(css);
    assert!(!report.is_clean());
    let before = report.clone();
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("style rule")
    };
    let source = &rule.declarations()[0];
    let component = source
        .value_components()
        .items()
        .iter()
        .find(|value| {
            matches!(
                value.view(),
                CssComponentValueRef::Token(CssValueTokenRef::String(_))
            )
        })
        .unwrap();
    let value = CssHyphenateString::try_from_component(component.clone()).unwrap();
    assert_eq!(value.component(), component);
    assert_eq!(value.as_str(), "😀");
    assert_eq!(value.origin(), component.origin());
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed opening and content")
    };
    assert!(
        origin
            .source()
            .same_snapshot(source.parsed_value().unwrap().source())
    );
    assert_eq!(origin.source().as_str(), css);
    let CssKnownPropertyValueRef::HyphenateCharacter(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("recovered typed string")
    };
    assert_eq!(
        wrapper.value(),
        &CssHyphenateCharacter::String(value.clone())
    );
    limits!(value, "\"😀\"", 1);
    assert_eq!(report, before);
}

#[test]
fn all_authored_auto_integer_slot_patterns_keep_their_arity_and_effective_symbolic_counts() {
    for arity in 1..=3 {
        for flags in 0..(1 << arity) {
            let slots: Vec<_> = (0..arity)
                .map(|index| {
                    if flags & (1 << index) == 0 {
                        CssHyphenateLimitCharsComponent::Auto
                    } else {
                        count("2")
                    }
                })
                .collect();
            let expected = slots
                .iter()
                .map(|slot| match slot {
                    CssHyphenateLimitCharsComponent::Auto => "auto",
                    CssHyphenateLimitCharsComponent::Integer(_) => "2",
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
                .join(" ");
            let value = CssHyphenateLimitChars::try_new(
                slots[0].clone(),
                slots.get(1).cloned(),
                slots.get(2).cloned(),
            )
            .unwrap();
            assert_eq!(value.authored_len(), arity);
            let effective = value.effective_components();
            assert_eq!(effective[0], slots[0]);
            assert_eq!(
                effective[1],
                slots
                    .get(1)
                    .cloned()
                    .unwrap_or(CssHyphenateLimitCharsComponent::Auto)
            );
            assert_eq!(effective[2], slots.get(2).unwrap_or(&effective[1]).clone());
            limits!(value, expected.as_str(), arity);
            for front in FRONTS {
                let source =
                    front.valid(CssKnownProperty::HyphenateLimitChars, &expected, &expected);
                let CssKnownPropertyValueRef::HyphenateLimitChars(wrapper) =
                    source.known().unwrap().property_value().unwrap()
                else {
                    panic!("typed authored slots")
                };
                assert_eq!(wrapper.value().authored_len(), arity);
            }
        }
    }
}

#[test]
fn every_autospace_group_order_and_shorthand_member_order_keeps_the_same_typed_roles() {
    let trims = [
        (CssSpacingTrim::SpaceAll, "space-all"),
        (CssSpacingTrim::Normal, "normal"),
        (CssSpacingTrim::SpaceFirst, "space-first"),
        (CssSpacingTrim::TrimStart, "trim-start"),
        (CssSpacingTrim::TrimBoth, "trim-both"),
        (CssSpacingTrim::TrimAll, "trim-all"),
    ];
    for (alpha, numeric, punctuation, flags) in
        FLAG_STATES.into_iter().chain([(false, false, false, "")])
    {
        for (mode, mode_text) in [
            (None, ""),
            (Some(CssAutospaceMode::Insert), "insert"),
            (Some(CssAutospaceMode::Replace), "replace"),
        ] {
            if flags.is_empty() && mode.is_none() {
                continue;
            }
            let roles = CssAutospaceValues::try_new(alpha, numeric, punctuation, mode).unwrap();
            let canonical = match (flags.is_empty(), mode.is_some()) {
                (true, _) => mode_text.to_owned(),
                (false, true) => format!("{flags} {mode_text}"),
                (false, false) => flags.to_owned(),
            };
            let words: Vec<_> = flags.split_whitespace().collect();
            for flag_order in permutations(&words) {
                let group_orders = if mode.is_none() {
                    vec![flag_order]
                } else if flag_order.is_empty() {
                    vec![mode_text.to_owned()]
                } else {
                    vec![
                        format!("{flag_order} {mode_text}"),
                        format!("{mode_text} {flag_order}"),
                    ]
                };
                for input in group_orders {
                    for front in FRONTS {
                        let source =
                            front.valid(CssKnownProperty::TextAutospace, &input, &canonical);
                        let CssKnownPropertyValueRef::TextAutospace(wrapper) =
                            source.known().unwrap().property_value().unwrap()
                        else {
                            panic!("typed autospace group")
                        };
                        assert_eq!(
                            wrapper.value(),
                            &CssTextAutospace::Autospace(CssAutospace::Spacing(roles))
                        );
                    }
                    for (trim, trim_text) in trims {
                        let value = CssTextSpacing::Components(
                            CssTextSpacingValues::try_new(
                                Some(trim),
                                Some(CssAutospace::Spacing(roles)),
                            )
                            .unwrap(),
                        );
                        let expected = format!("{trim_text} {canonical}");
                        limits!(
                            value,
                            expected.as_str(),
                            expected.split_whitespace().count()
                        );
                        for shorthand_input in [
                            format!("{trim_text} {input}"),
                            format!("{input} {trim_text}"),
                        ] {
                            for front in FRONTS {
                                let source = front.valid(
                                    CssKnownProperty::TextSpacing,
                                    &shorthand_input,
                                    &expected,
                                );
                                let CssKnownPropertyValueRef::TextSpacing(wrapper) =
                                    source.known().unwrap().property_value().unwrap()
                                else {
                                    panic!("typed shorthand groups")
                                };
                                assert_eq!(wrapper.value(), &value);
                            }
                        }
                    }
                }
            }
        }
    }
    for (trim, trim_text) in trims {
        let expected = format!("{trim_text} no-autospace");
        let value = CssTextSpacing::Components(
            CssTextSpacingValues::try_new(Some(trim), Some(CssAutospace::NoAutospace)).unwrap(),
        );
        for input in [expected.clone(), format!("no-autospace {trim_text}")] {
            for front in FRONTS {
                let source = front.valid(CssKnownProperty::TextSpacing, &input, &expected);
                let CssKnownPropertyValueRef::TextSpacing(wrapper) =
                    source.known().unwrap().property_value().unwrap()
                else {
                    panic!("typed exclusive autospace group")
                };
                assert_eq!(wrapper.value(), &value);
            }
        }
    }
}

#[test]
fn every_hanging_role_permutation_preserves_the_exclusive_end_choice() {
    for first in [false, true] {
        for end in [
            None,
            Some(CssHangingPunctuationEnd::ForceEnd),
            Some(CssHangingPunctuationEnd::AllowEnd),
        ] {
            for last in [false, true] {
                if !first && end.is_none() && !last {
                    continue;
                }
                let value = CssHangingPunctuation::Hang(
                    CssHangingPunctuationValues::try_new(first, end, last).unwrap(),
                );
                let mut words = Vec::new();
                if first {
                    words.push("first");
                }
                match end {
                    Some(CssHangingPunctuationEnd::ForceEnd) => words.push("force-end"),
                    Some(CssHangingPunctuationEnd::AllowEnd) => words.push("allow-end"),
                    None => {}
                    _ => unreachable!(),
                }
                if last {
                    words.push("last");
                }
                let expected = words.join(" ");
                for input in permutations(&words) {
                    for front in FRONTS {
                        let source =
                            front.valid(CssKnownProperty::HangingPunctuation, &input, &expected);
                        let CssKnownPropertyValueRef::HangingPunctuation(wrapper) =
                            source.known().unwrap().property_value().unwrap()
                        else {
                            panic!("typed hanging roles")
                        };
                        assert_eq!(wrapper.value(), &value);
                    }
                }
            }
        }
    }
}

#[test]
fn spacing_constituent_boundaries_reject_interleaving_and_project_only_the_two_typed_members() {
    for input in [
        "ideograph-alpha insert punctuation",
        "punctuation replace ideograph-numeric",
    ] {
        authored_property::invalid(CssKnownProperty::TextAutospace, input);
    }
    for input in [
        "ideograph-alpha trim-start punctuation",
        "insert trim-start ideograph-alpha",
        "normal normal",
        "auto insert",
        "trim-start auto",
    ] {
        authored_property::invalid(CssKnownProperty::TextSpacing, input);
    }
    for (input, trim, autospace) in [
        (
            "none",
            CssTextSpacingTrim::Trim(CssSpacingTrim::SpaceAll),
            CssTextAutospace::Autospace(CssAutospace::NoAutospace),
        ),
        ("auto", CssTextSpacingTrim::Auto, CssTextAutospace::Auto),
        (
            "normal",
            CssTextSpacingTrim::Trim(CssSpacingTrim::Normal),
            CssTextAutospace::Normal,
        ),
        (
            "trim-both",
            CssTextSpacingTrim::Trim(CssSpacingTrim::TrimBoth),
            CssTextAutospace::Normal,
        ),
        (
            "replace",
            CssTextSpacingTrim::Trim(CssSpacingTrim::Normal),
            CssTextAutospace::Autospace(CssAutospace::Spacing(
                CssAutospaceValues::try_new(false, false, false, Some(CssAutospaceMode::Replace))
                    .unwrap(),
            )),
        ),
    ] {
        let source = declaration("text-spacing", input);
        let before = source.clone();
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("ordinary projection")
        };
        let [trim_item, autospace_item] = values.items() else {
            panic!("exactly two settable members")
        };
        assert_eq!(trim_item.property(), CssKnownProperty::TextSpacingTrim);
        assert_eq!(autospace_item.property(), CssKnownProperty::TextAutospace);
        assert!(
            matches!(trim_item.ordinary_value().unwrap().view(),CssLonghandValueRef::TextSpacingTrim(value) if value==&trim)
        );
        assert!(
            matches!(autospace_item.ordinary_value().unwrap().view(),CssLonghandValueRef::TextAutospace(value) if value==&autospace)
        );
        for item in values.items() {
            authored_property::assert_source(item, &source);
        }
        assert_eq!(source, before);
    }
}

#[test]
fn numeric_children_keep_signed_domains_exact_origins_and_existing_math_work_prices() {
    for (input, expected) in [("-0", "0"), ("-2px", "-2px"), ("-25%", "-25%")] {
        let source = declaration("hyphenate-limit-zone", input);
        let CssKnownPropertyValueRef::HyphenateLimitZone(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed signed zone")
        };
        let value = wrapper.value();
        let component = source
            .value_components()
            .items()
            .iter()
            .find(|value| {
                !matches!(
                    value.view(),
                    CssComponentValueRef::Comment(_)
                        | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                )
            })
            .unwrap();
        assert_eq!(value.literal_component(), Some(component));
        limits!(value.clone(), expected, 1);
    }
    let source = declaration("line-padding", "-2px");
    let CssKnownPropertyValueRef::LinePadding(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed signed padding")
    };
    limits!(wrapper.value().clone(), "-2px", 1);
    // Numeric's existing contract charges four input and three projection
    // nodes for a calc group containing a sum of two ordinary scalar leaves.
    let zone = CssSpecifiedLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(components("calc(1px + 2px)")).unwrap(),
    )
    .unwrap();
    let padding = CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(components("calc(-2px + -3px)")).unwrap(),
    )
    .unwrap();
    limits!(zone, "calc(3px)", 4, 3);
    limits!(padding, "calc(-5px)", 4, 3);
    let integer = CssHyphenateLimitInteger::try_new(CssIntegerValue::Calculation(
        CssIntegerCalculation::try_from_components(components("calc(-3 / 2)")).unwrap(),
    ))
    .unwrap();
    // Division also creates an inverse projection node under that contract.
    limits!(integer.clone(), "calc(-1.5)", 4, 4);
    limits!(
        CssHyphenateLimitLines::Integer(integer.clone()),
        "calc(-1.5)",
        4,
        4
    );
    let chars = CssHyphenateLimitChars::try_new(
        CssHyphenateLimitCharsComponent::Integer(integer),
        Some(CssHyphenateLimitCharsComponent::Auto),
        None,
    )
    .unwrap();
    limits!(chars, "calc(-1.5) auto", 5, 5);
}
