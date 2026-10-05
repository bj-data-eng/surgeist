#![forbid(unsafe_code)]
//! Existing public admission boundaries for UI 4 cursor images.
//! Independent authority: selected UI 4 WD 2026-01-20 §5.1.1,
//! references/css-ui-4--WD-css-ui-4-20260120--42d9c51404fb.md:
//! `<cursor-image> = [ <url> | <url-set> ] <number>{2}?`.
//! A url-set restricts the image arm to URLs; the final keyword is required.
//! Images 4 WD 2025-09-30 §2.4 separately admits strings representing URLs,
//! optional resolution/type descriptors in either order, and authored options
//! whose MIME type or duplicate resolution affects downstream selection.
//! https://www.w3.org/TR/2025/WD-css-images-4-20250930/#image-set-notation
//! Its Appendix A makes -webkit-image-set() a standard parse-time alias.
//! Pinned Values 4 WD 2024-03-12 §§7.4, 10.12 excludes negative ordinary
//! resolutions, admits zero, and defers mathematical range checking.
//! Hotspot clamping and resource selection are downstream operations. These tests
//! assert admission and retained authored components, without assuming a cursor
//! carrier layout or a future model accessor or serialization API.

use surgeist_css::*;

const HOTSPOTS: &[(&str, &[&str])] = &[
    (
        "url(cursor.cur) 3.000e1 -0.25, pointer",
        &["3.000e1", "-0.25"],
    ),
    ("url(cursor.cur) -1e999 1e-999, auto", &["-1e999", "1e-999"]),
    (
        "url(first.cur) 0 0, url(second.png) -3.5 4.25, default",
        &["0", "0", "-3.5", "4.25"],
    ),
    (
        "url(\"cursor.svg\") /* hotspot */ -3.500 +4.250, crosshair",
        &["-3.500", "+4.250"],
    ),
];

const MATH_HOTSPOTS: &[(&str, &[&str])] = &[
    (
        "url(cursor.cur) calc(1 + 2) calc(-0.25), pointer",
        &["1", "2", "-0.25"],
    ),
    (
        "url(cursor.cur) min(3.000e1, 40) max(-2.5, -0.25), move",
        &["3.000e1", "40", "-2.5", "-0.25"],
    ),
    ("url(cursor.cur) calc(pi) calc(e), auto", &[]),
];

const URL_SETS: &[(&str, &[&str])] = &[
    (
        "image-set(url(cursor.png) 1x, url(cursor-2.png) 2x), pointer",
        &[],
    ),
    (
        "image-set(url(\"cursor.png\") 1dppx) -3.500 4.250, crosshair",
        &["-3.500", "4.250"],
    ),
    (
        "image-set(url(cursor.png) 1x) 1 2, url(other.cur) -3 4, auto",
        &["1", "2", "-3", "4"],
    ),
];

const STRING_URL_SETS: &[(&str, &[&str])] = &[
    ("image-set(\"cursor.png\" 1x), auto", &["cursor.png"]),
    ("image-set(\"cursor.png\"), pointer", &["cursor.png"]),
    (
        "image-set(\"first.png\" 1x, url(second.png) 2x), move",
        &["first.png"],
    ),
    (r#"image-set("cursor\2e png" 1x), auto"#, &["cursor.png"]),
    ("image-set(\"\" 1x), none", &[""]),
];

const DESCRIPTOR_SETS: &[(&str, &[&str])] = &[
    ("image-set(url(cursor.png)), auto", &[]),
    (
        "image-set(url(cursor.png) type(\"image/png\")), auto",
        &["image/png"],
    ),
    (
        "image-set(url(cursor.png) type(\"image/png\") 96dpi), auto",
        &["image/png"],
    ),
    (
        "image-set(url(cursor.png) 96dpi type(\"image/png\")), auto",
        &["image/png"],
    ),
    ("image-set(url(cursor.png) type(\"\")), auto", &[""]),
    (
        "image-set(url(cursor.png) type(\"not a MIME type\")), auto",
        &["not a MIME type"],
    ),
    (
        "image-set(url(cursor.png) type(\"image/unregistered\")), auto",
        &["image/unregistered"],
    ),
    (
        "image-set(url(first.png) 1x, url(second.png) 96dpi), auto",
        &[],
    ),
    ("image-set(url(first.png), url(second.png)), auto", &[]),
];

const RESOLUTION_SETS: &[(&str, &[(&str, &str)])] = &[
    ("image-set(url(cursor.png) 0x), auto", &[("0", "x")]),
    ("image-set(url(cursor.png) -0dppx), auto", &[("-0", "dppx")]),
    ("image-set(url(cursor.png) calc(-1x)), auto", &[("-1", "x")]),
    (
        "image-set(url(cursor.png) min(-1dppx, 2dppx)), auto",
        &[("-1", "dppx"), ("2", "dppx")],
    ),
    (
        "image-set(url(cursor.png) calc(-1e999x)), auto",
        &[("-1e999", "x")],
    ),
];

const ALIAS_SETS: &[(&str, &str)] = &[
    (
        "-webkit-image-set(url(cursor.png) 1x), auto",
        "-webkit-image-set",
    ),
    (
        "-WEBKIT-IMAGE-SET(\"cursor.png\" type(\"image/png\") 2x), pointer",
        "-WEBKIT-IMAGE-SET",
    ),
    (
        r"-webkit-image-s\65 t(url(cursor.png)), auto",
        "-webkit-image-set",
    ),
];

const INVALID: &[&str] = &[
    "url(cursor.cur) 1, auto",
    "url(cursor.cur) 1 2 3, auto",
    "url(cursor.cur) 1px 2, auto",
    "url(cursor.cur) 1 2px, auto",
    "url(cursor.cur) 1% 2, auto",
    "url(cursor.cur) 1 2%, auto",
    "url(cursor.cur) calc(1px) 2, auto",
    "url(cursor.cur) 1 calc(2%), auto",
    "url(cursor.cur) 1 2",
    "url(cursor.cur)",
    "url(cursor.cur),",
    "url(cursor.cur) 1 2, url(other.cur) 3, auto",
    "image-set(url(cursor.png) 1x) 1, auto",
    "image-set(url(cursor.png) 1x) 1 2 3, auto",
    "image-set(url(cursor.png) 1x)",
    "image-set(linear-gradient(red, blue) 1x), auto",
    "image-set(image-set(url(cursor.png) 1x) 2x), auto",
    "image-set(url(cursor.png) -1x), auto",
    "image-set(\"cursor.png\" -1e999dpi), auto",
    "image-set(url(cursor.png) 1x 2x), auto",
    "image-set(url(cursor.png) type(\"image/png\") type(\"image/jpeg\")), auto",
    "image-set(url(cursor.png) type()), auto",
    "image-set(url(cursor.png) type(image/png)), auto",
    "image-set(url(cursor.png) type(\"image/png\" \"image/jpeg\")), auto",
    "image-set(), auto",
    "image-set(url(cursor.png) 1x,), auto",
    "-webkit-image-set(linear-gradient(red, blue) 1x), auto",
    "-webkit-image-set(image-set(url(cursor.png)) 1x), auto",
];

fn position(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value() as usize,
        source[..offset].encode_utf16().count()
    );
}

fn cursor(declaration: &CssDeclaration) {
    let known = declaration.known().expect("known cursor occurrence");
    assert_eq!(known.property(), CssKnownProperty::Cursor);
    assert_eq!(known.grammar(), CssKnownProperty::Cursor.grammar());
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    assert!(matches!(
        known.property_value(),
        Some(CssKnownPropertyValueRef::Cursor(_))
    ));
}

fn origin(origin: &CssValueOrigin, source: &str) {
    let CssValueOrigin::Parsed(parsed) = origin else {
        panic!("original parsed component origin")
    };
    assert_eq!(parsed.source().as_str(), source);
    position(
        parsed.span().start(),
        source,
        parsed.span().start().byte_offset().value(),
    );
    position(
        parsed.span().end(),
        source,
        parsed.span().end().byte_offset().value(),
    );
}

fn components<'a>(values: &'a CssComponentValues, source: &str, numbers: &mut Vec<&'a str>) {
    for component in values.items() {
        origin(component.origin(), source);
        match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
                numbers.push(number.representation());
                let CssValueOrigin::Parsed(parsed) = component.origin() else {
                    unreachable!("origin checked above")
                };
                assert_eq!(
                    &source[parsed.span().start().byte_offset().value()
                        ..parsed.span().end().byte_offset().value()],
                    number.representation()
                );
            }
            CssComponentValueRef::Function(function) => {
                origin(function.closing_origin(), source);
                components(function.values(), source, numbers);
            }
            CssComponentValueRef::Block(block) => {
                origin(block.closing_origin(), source);
                components(block.values(), source, numbers);
            }
            _ => {}
        }
    }
}

fn retained_components(declaration: &CssDeclaration, source: &str, expected_numbers: &[&str]) {
    let mut numbers = Vec::new();
    components(declaration.value_components(), source, &mut numbers);
    assert_eq!(numbers, expected_numbers);
}

fn parsed_accepts(value: &str, numbers: &[&str]) -> CssDeclaration {
    let source = format!("/* 🦀 */ color:red !important; cursor:{value} !important; width:1px");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [before, declaration, after] = report.syntax().as_slice() else {
        panic!("all three declarations must survive in source order: {source}")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(before.importance(), CssImportance::Important);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(after.importance(), CssImportance::Normal);
    cursor(declaration);
    assert_eq!(declaration.importance(), CssImportance::Important);
    position(
        declaration.position().unwrap(),
        &source,
        source.find("cursor:").unwrap(),
    );
    let parsed = declaration.parsed_value().unwrap();
    assert_eq!(parsed.source().as_str(), source);
    assert_eq!(
        source[parsed.span().start().byte_offset().value()
            ..parsed.span().end().byte_offset().value()]
            .trim(),
        value
    );
    let retained = declaration.clone();
    drop(report);
    retained_components(&retained, &source, numbers);
    retained
}

fn checked_accepts(value: &str, numbers: &[&str], grammar: bool) -> CssDeclaration {
    let supplied = parse_component_values(value).expect("well-formed supplied components");
    let declaration = if grammar {
        parse_property_value_for_grammar(
            CssKnownProperty::Cursor.grammar(),
            supplied.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Cursor),
            supplied.clone(),
            CssImportance::Important,
        )
    }
    .unwrap_or_else(|error| panic!("{value}: {error:?}"));
    cursor(&declaration);
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert_eq!(declaration.value_components(), &supplied);
    retained_components(&declaration, value, numbers);
    declaration
}

fn retained_tokens<'a>(values: &'a CssComponentValues, tokens: &mut Vec<CssValueTokenRef<'a>>) {
    for component in values.items() {
        match component.view() {
            CssComponentValueRef::Token(token) => tokens.push(token),
            CssComponentValueRef::Function(function) => retained_tokens(function.values(), tokens),
            CssComponentValueRef::Block(block) => retained_tokens(block.values(), tokens),
            _ => {}
        }
    }
}

fn retained_strings(declaration: &CssDeclaration, expected: &[&str]) {
    let mut tokens = Vec::new();
    retained_tokens(declaration.value_components(), &mut tokens);
    let strings: Vec<_> = tokens
        .into_iter()
        .filter_map(|token| match token {
            CssValueTokenRef::String(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(strings, expected);
}

fn retained_dimensions(declaration: &CssDeclaration, expected: &[(&str, &str)]) {
    let mut tokens = Vec::new();
    retained_tokens(declaration.value_components(), &mut tokens);
    let dimensions: Vec<_> = tokens
        .into_iter()
        .filter_map(|token| match token {
            CssValueTokenRef::Dimension { number, unit } => Some((number.representation(), unit)),
            _ => None,
        })
        .collect();
    assert_eq!(dimensions, expected);
}

fn retained_alias(declaration: &CssDeclaration, expected: &str) {
    let function = declaration
        .value_components()
        .items()
        .iter()
        .find_map(|component| match component.view() {
            CssComponentValueRef::Function(function) => Some(function),
            _ => None,
        })
        .expect("retained authored alias function");
    assert_eq!(function.name(), expected);
}

fn validator_accepts(value: &str) {
    let source = format!("color:red !important; cursor:{value} !important; width:1px");
    assert!(validate_style_attribute(&source).is_ok(), "{source}");
}

#[test]
fn url_hotspot_pairs_are_admitted_with_exact_numbers_origins_importance_and_neighbors() {
    for &(value, numbers) in HOTSPOTS {
        parsed_accepts(value, numbers);
    }
}

#[test]
fn validator_admits_url_hotspot_pairs() {
    for &(value, _) in HOTSPOTS {
        validator_accepts(value);
    }
}

#[test]
fn checked_property_admits_url_hotspot_pairs_and_retains_supplied_components() {
    for &(value, numbers) in HOTSPOTS {
        checked_accepts(value, numbers, false);
    }
}

#[test]
fn checked_grammar_admits_url_hotspot_pairs_and_retains_supplied_components() {
    for &(value, numbers) in HOTSPOTS {
        checked_accepts(value, numbers, true);
    }
}

#[test]
fn number_math_hotspots_remain_authored_with_original_nested_components() {
    for &(value, numbers) in MATH_HOTSPOTS {
        parsed_accepts(value, numbers);
    }
}

#[test]
fn validator_admits_number_math_hotspot_pairs() {
    for &(value, _) in MATH_HOTSPOTS {
        validator_accepts(value);
    }
}

#[test]
fn checked_property_admits_number_math_hotspots_without_fabricated_declaration_origins() {
    for &(value, numbers) in MATH_HOTSPOTS {
        checked_accepts(value, numbers, false);
    }
}

#[test]
fn checked_grammar_admits_number_math_hotspots_without_fabricated_declaration_origins() {
    for &(value, numbers) in MATH_HOTSPOTS {
        checked_accepts(value, numbers, true);
    }
}

#[test]
fn url_only_image_sets_are_admitted_with_optional_hotspots_and_ordered_neighbors() {
    for &(value, numbers) in URL_SETS {
        parsed_accepts(value, numbers);
    }
}

#[test]
fn validator_admits_url_only_image_sets() {
    for &(value, _) in URL_SETS {
        validator_accepts(value);
    }
}

#[test]
fn checked_property_admits_url_only_image_sets_and_preserves_supplied_components() {
    for &(value, numbers) in URL_SETS {
        checked_accepts(value, numbers, false);
    }
}

#[test]
fn checked_grammar_admits_url_only_image_sets_and_preserves_supplied_components() {
    for &(value, numbers) in URL_SETS {
        checked_accepts(value, numbers, true);
    }
}

#[test]
fn string_url_candidates_are_admitted_with_decoded_targets_original_components_and_neighbors() {
    for &(value, strings) in STRING_URL_SETS {
        retained_strings(&parsed_accepts(value, &[]), strings);
    }
}

#[test]
fn validator_admits_string_url_candidates() {
    for &(value, _) in STRING_URL_SETS {
        validator_accepts(value);
    }
}

#[test]
fn checked_property_admits_string_url_candidates_and_preserves_supplied_origins() {
    for &(value, strings) in STRING_URL_SETS {
        retained_strings(&checked_accepts(value, &[], false), strings);
    }
}

#[test]
fn checked_grammar_admits_string_url_candidates_and_preserves_supplied_origins() {
    for &(value, strings) in STRING_URL_SETS {
        retained_strings(&checked_accepts(value, &[], true), strings);
    }
}

#[test]
fn descriptor_omissions_orders_and_selection_dependent_values_are_admitted() {
    for &(value, strings) in DESCRIPTOR_SETS {
        retained_strings(&parsed_accepts(value, &[]), strings);
    }
}

#[test]
fn validator_admits_omitted_ordered_and_selection_dependent_descriptors() {
    for &(value, _) in DESCRIPTOR_SETS {
        validator_accepts(value);
    }
}

#[test]
fn checked_property_admits_omitted_ordered_and_selection_dependent_descriptors() {
    for &(value, strings) in DESCRIPTOR_SETS {
        retained_strings(&checked_accepts(value, &[], false), strings);
    }
}

#[test]
fn checked_grammar_admits_omitted_ordered_and_selection_dependent_descriptors() {
    for &(value, strings) in DESCRIPTOR_SETS {
        retained_strings(&checked_accepts(value, &[], true), strings);
    }
}

#[test]
fn zero_ordinary_and_negative_math_resolutions_are_admitted_without_authored_clamping() {
    for &(value, dimensions) in RESOLUTION_SETS {
        retained_dimensions(&parsed_accepts(value, &[]), dimensions);
    }
}

#[test]
fn validator_admits_zero_ordinary_and_negative_math_resolutions() {
    for &(value, _) in RESOLUTION_SETS {
        validator_accepts(value);
    }
}

#[test]
fn checked_property_retains_zero_ordinary_and_negative_math_resolution_components() {
    for &(value, dimensions) in RESOLUTION_SETS {
        retained_dimensions(&checked_accepts(value, &[], false), dimensions);
    }
}

#[test]
fn checked_grammar_retains_zero_ordinary_and_negative_math_resolution_components() {
    for &(value, dimensions) in RESOLUTION_SETS {
        retained_dimensions(&checked_accepts(value, &[], true), dimensions);
    }
}

#[test]
fn standard_prefixed_image_set_alias_is_admitted_while_original_function_spelling_survives() {
    for &(value, name) in ALIAS_SETS {
        retained_alias(&parsed_accepts(value, &[]), name);
    }
}

#[test]
fn validator_admits_the_standard_prefixed_image_set_alias() {
    for &(value, _) in ALIAS_SETS {
        validator_accepts(value);
    }
}

#[test]
fn checked_property_retains_the_standard_prefixed_image_set_alias_components() {
    for &(value, name) in ALIAS_SETS {
        retained_alias(&checked_accepts(value, &[], false), name);
    }
}

#[test]
fn checked_grammar_retains_the_standard_prefixed_image_set_alias_components() {
    for &(value, name) in ALIAS_SETS {
        retained_alias(&checked_accepts(value, &[], true), name);
    }
}

#[test]
fn programmatic_url_hotspot_numbers_keep_programmatic_origins() {
    let supplied = CssComponentValues::try_new(
        ["url(cursor.cur)", "-3.500", "+4.250", ",", "pointer"]
            .into_iter()
            .map(|token| CssComponentValue::try_token(token).unwrap())
            .collect(),
    )
    .unwrap();
    for declaration in [
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Cursor),
            supplied.clone(),
            CssImportance::Important,
        )
        .unwrap(),
        parse_property_value_for_grammar(
            CssKnownProperty::Cursor.grammar(),
            supplied.clone(),
            CssImportance::Important,
        )
        .unwrap(),
    ] {
        cursor(&declaration);
        assert_eq!(declaration.importance(), CssImportance::Important);
        assert!(declaration.position().is_none());
        assert!(declaration.parsed_name().is_none());
        assert!(declaration.parsed_value().is_none());
        assert_eq!(declaration.value_components(), &supplied);
        for component in declaration.value_components().items() {
            assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
        }
    }
}

fn invalid_source(value: &str) -> String {
    format!("/* 🦀 */ color:red !important; cursor:{value} !important; width:1px")
}

fn parsed_rejects(value: &str) {
    let source = invalid_source(value);
    let report = parse_style_attribute(&source);
    let [before, after] = report.syntax().as_slice() else {
        panic!("only valid neighboring declarations must survive: {source}")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(before.importance(), CssImportance::Important);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(after.importance(), CssImportance::Normal);
    let [diagnostic] = report.diagnostics() else {
        panic!("one cursor grammar diagnostic: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("typed cursor property grammar diagnostic")
    };
    assert_eq!(detail.property(), CssKnownProperty::Cursor);
    let start = source.find("cursor:").unwrap();
    let end = start + source[start..].find(';').unwrap() + 1;
    position(diagnostic.span().start(), &source, start);
    position(diagnostic.span().end(), &source, end);
    let error_offset = diagnostic.error().position().byte_offset().value();
    assert!((start..=end).contains(&error_offset));
    position(diagnostic.error().position(), &source, error_offset);
}

fn checked_rejects(value: &str, grammar: bool) {
    let supplied = parse_component_values(value).unwrap();
    let result = if grammar {
        parse_property_value_for_grammar(
            CssKnownProperty::Cursor.grammar(),
            supplied,
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Cursor),
            supplied,
            CssImportance::Important,
        )
    };
    let error = result.expect_err(
        "cursor grammar must reject malformed tuples, missing fallbacks or non-URL sets",
    );
    let CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail)) = error.kind()
    else {
        panic!("typed checked cursor property grammar failure: {error:?}")
    };
    assert_eq!(detail.property(), CssKnownProperty::Cursor);
}

#[test]
fn malformed_hotspots_missing_fallbacks_and_non_url_sets_drop_only_the_cursor_declaration() {
    for value in INVALID {
        parsed_rejects(value);
    }
}

#[test]
fn validator_rejects_malformed_hotspots_missing_fallbacks_and_non_url_sets() {
    for value in INVALID {
        let source = invalid_source(value);
        let report = parse_style_attribute(&source);
        let failure = validate_style_attribute(&source).expect_err("invalid cursor declaration");
        assert_eq!(failure.diagnostics(), report.diagnostics(), "{value}");
    }
}

#[test]
fn checked_property_rejects_malformed_hotspots_missing_fallbacks_and_non_url_sets() {
    for value in INVALID {
        checked_rejects(value, false);
    }
}

#[test]
fn checked_grammar_rejects_malformed_hotspots_missing_fallbacks_and_non_url_sets() {
    for value in INVALID {
        checked_rejects(value, true);
    }
}

#[test]
fn url_lists_without_hotspots_remain_admitted_at_every_boundary() {
    for value in [
        "url(cursor.cur), auto",
        "url(first.cur), url(\"second.png\"), pointer",
        "url(), none",
    ] {
        parsed_accepts(value, &[]);
        validator_accepts(value);
        checked_accepts(value, &[], false);
        checked_accepts(value, &[], true);
    }
}

#[test]
fn predefined_cursor_fallbacks_remain_admitted_at_every_boundary() {
    for value in [
        "auto",
        "default",
        "none",
        "pointer",
        "crosshair",
        "text",
        "vertical-text",
        "move",
        "not-allowed",
        "grab",
        "grabbing",
        "col-resize",
        "row-resize",
        "nwse-resize",
        "zoom-in",
        "zoom-out",
        "POINTER",
        r"p\6f inter",
    ] {
        parsed_accepts(value, &[]);
        validator_accepts(value);
        checked_accepts(value, &[], false);
        checked_accepts(value, &[], true);
    }
}
