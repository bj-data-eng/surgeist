#![forbid(unsafe_code)]

//! Existing-public-API composition witnesses, independently authored.
//! Values4 WD20240312 §§2.2–2.6, 4.2, 5–6, 8.3, 10.11; Syntax3
//! CRD20211224 §§3.3, 4.3.5; selected Fonts4 WD20260907 §§2.1.1, 2.7;
//! Motion1 WD20241105 §2.1.1; CSSOM WD20210826 §2.1 and the public
//! immutable provenance/resource/whole-production contracts supply the oracles.

use surgeist_css::*;

const TWENTY: &str =
    "F01,F02,F03,F04,F05,F06,F07,F08,F09,F10,F11,F12,F13,F14,F15,F16,F17,F18,F19,F20";
const TWENTY_CSS: &str = "F01, F02, F03, F04, F05, F06, F07, F08, F09, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20";
const NAMES: [&str; 20] = [
    "F01", "F02", "F03", "F04", "F05", "F06", "F07", "F08", "F09", "F10", "F11", "F12", "F13",
    "F14", "F15", "F16", "F17", "F18", "F19", "F20",
];

fn families(declaration: &CssDeclaration) -> &CssFontFamilyList {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::FontFamily(value) => value.families(),
        CssKnownPropertyValueRef::Font(value) => {
            let CssFontValue::Explicit(value) = value.font() else {
                panic!("explicit font tail");
            };
            value.families()
        }
        other => panic!("a family-importing property, got {other:?}"),
    }
}

fn original(origin: &CssValueOrigin, source: &str, start: usize, end: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("an original parsed token origin");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

fn parity(source: &str) -> CssParseReport<CssDeclarationList> {
    let report = parse_style_attribute(source);
    if report.is_clean() {
        assert_eq!(
            validate_style_attribute(source),
            Ok(report.syntax().clone())
        );
    } else {
        assert_eq!(
            validate_style_attribute(source).unwrap_err().diagnostics(),
            report.diagnostics(),
        );
    }
    report
}

fn atomic_bytes(declaration: &CssDeclaration, expected: &str) {
    let before = declaration.clone();
    let components = declaration.value_components().clone();
    let exact = CssSpecifiedValueSerializationLimits::new(10_000, 10_000, expected.len());
    assert_eq!(
        declaration.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    assert_eq!(
        declaration
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                10_000,
                10_000,
                expected.len() - 1,
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit,
    );
    assert_eq!(declaration, &before);
    assert!(declaration.same_occurrence(&before));
    assert_eq!(declaration.value_components(), &components);
    assert_eq!(declaration.to_specified_css().unwrap(), expected);
    let reparsed = parity(expected);
    assert!(reparsed.is_clean(), "{:?}", reparsed.diagnostics());
    let [reparsed] = reparsed.syntax().as_slice() else {
        panic!("one complete emitted declaration");
    };
    assert_eq!(reparsed.importance(), declaration.importance());
    assert_eq!(reparsed.to_specified_css().unwrap(), expected);
}

#[test]
fn twenty_family_repetitions_keep_order_and_use_one_atomic_output_budget() {
    // Twenty three-byte identifiers, nineteen comma tokens: 79 authored bytes.
    // Specified comma-space output is 60 + 19*2 = 98 bytes. A name plus its
    // retained identifier costs two nodes by the public family writer contract.
    assert_eq!(TWENTY.len(), 79);
    assert_eq!(TWENTY_CSS.len(), 98);
    let source = format!("font-family:{TWENTY}!important;width:2px");
    let report = parity(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [parsed, width] = report.syntax().as_slice() else {
        panic!("the twenty-member declaration and later width");
    };
    assert_eq!(width.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(width.position().unwrap().byte_offset().value(), 102);
    original(
        parsed.value_components().items()[0].origin(),
        &source,
        12,
        15,
    );
    original(
        parsed.value_components().items()[38].origin(),
        &source,
        88,
        91,
    );

    let components = parse_component_values(TWENTY).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::FontFamily),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &components);
    assert!(checked.parsed_value().is_none());
    for declaration in [parsed, &checked] {
        assert_eq!(declaration.importance(), CssImportance::Important);
        let values = families(declaration);
        assert_eq!(
            values
                .families()
                .iter()
                .map(CssFontFamilyName::as_str)
                .collect::<Vec<_>>(),
            NAMES,
        );
        assert!(
            values
                .families()
                .iter()
                .all(|value| value.kind() == CssFontFamilyNameKind::IdentSequence)
        );
        assert_eq!(
            values
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    40, 40, 98
                ))
                .unwrap(),
            TWENTY_CSS,
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(39, 40, 98),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(40, 39, 98),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(40, 40, 97),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                values
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(values.serialize_specified().unwrap(), TWENTY_CSS);
        }
        atomic_bytes(
            declaration,
            &format!("font-family: {TWENTY_CSS} !important;"),
        );
    }
    assert_eq!(
        report.syntax()[1].to_specified_css().unwrap(),
        "width: 2px;"
    );
}

#[test]
fn a_resource_denied_tail_discards_the_whole_twenty_member_declaration_locally() {
    // Resource admission precedes semantic family admission. The final f() is
    // not a family; its depth257 is a native structural denial, not a synthetic
    // twenty-one-item grammar cap. There are no unmatched/implicit delimiters.
    let source = format!(
        "color:red;font-family:{TWENTY},{}x{};width:2px;",
        "f(".repeat(257),
        ")".repeat(257),
    );
    // color:red;10 + font-family:12 + members79 + comma1 = first f at102.
    // The denied opener is102 + 256*2 =614. The unit ends after 514+1+257
    // nested bytes and its semicolon:102+772+1 =875. The later width is10 bytes.
    assert_eq!(source.len(), 885);
    let report = parity(&source);
    let [color, width] = report.syntax().as_slice() else {
        panic!("no partial family list survives; both source-valid siblings do");
    };
    assert_eq!(color.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(width.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(width.position().unwrap().byte_offset().value(), 875);
    assert_eq!(width.to_specified_css().unwrap(), "width: 2px;");
    let [diagnostic] = report.diagnostics() else {
        panic!("one closed, locally resource-denied declaration");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 614);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 10);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 875);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("the structural owner reports its resource fault");
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(detail.enclosing_production().as_str(), "css.declaration");
}

#[test]
fn string_escapes_and_continuations_keep_decoded_family_identity_in_imported_font_tails() {
    // These are authored string tokens, not decoded constructor arguments.
    for (authored, decoded, canonical) in [
        (r#""A\"B""#, "A\"B", r#""A\"B""#),
        (r"'A\\B'", "A\\B", r#""A\\B""#),
        ("\"Line\\\nBreak\"", "LineBreak", "\"LineBreak\""),
        ("'Line\\\r\nBreak'", "LineBreak", "\"LineBreak\""),
        (r#""A\A B""#, "A\nB", r#""A\a B""#),
    ] {
        let source =
            format!("font-family:{authored},serif!important;font:16px {authored},serif;width:2px");
        let report = parity(&source);
        assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
        let [family, font, width] = report.syntax().as_slice() else {
            panic!("both importing grammars and the later declaration");
        };
        assert_eq!(width.known().unwrap().property(), CssKnownProperty::Width);
        assert_eq!(family.importance(), CssImportance::Important);
        assert_eq!(font.importance(), CssImportance::Normal);
        let component = &family.value_components().items()[0];
        assert!(
            matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::String(value)) if value == decoded)
        );
        original(component.origin(), &source, 12, 12 + authored.len());
        for declaration in [family, font] {
            let [literal, generic] = families(declaration).families() else {
                panic!("one quoted literal and one separate generic");
            };
            assert_eq!(literal.kind(), CssFontFamilyNameKind::Quoted);
            assert_eq!(literal.as_str(), decoded);
            assert_eq!(literal.generic_family(), None);
            assert_eq!(generic.generic_family(), Some(CssGenericFontFamily::Serif));
            assert_eq!(
                families(declaration).serialize_specified().unwrap(),
                format!("{canonical}, serif")
            );
        }
        atomic_bytes(
            family,
            &format!("font-family: {canonical}, serif !important;"),
        );
        let supplied = parse_component_values(&format!("{authored},serif")).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::FontFamily),
            supplied.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &supplied);
        assert_eq!(families(&checked).families()[0].as_str(), decoded);
        assert_eq!(
            checked.to_specified_css().unwrap(),
            format!("font-family: {canonical}, serif !important;")
        );
    }
}

#[test]
fn checked_exact_numeric_origins_survive_whole_declaration_output_failure_and_retry() {
    const AUTHORED: &str = r"+2\50 X";
    const EXPECTED: &str = "width: 2px !important;";
    let parsed = parse_component_values(AUTHORED).unwrap().items()[0].clone();
    original(parsed.origin(), AUTHORED, 0, 7);
    for component in [
        parsed,
        CssComponentValue::try_dimension("+2", "PX").unwrap(),
    ] {
        let exact =
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component.clone()).unwrap();
        assert_eq!(exact.literal_component(), Some(&component));
        assert_eq!(exact.origin(), component.origin());
        let components = CssComponentValues::try_new(vec![component.clone()]).unwrap();
        let declaration = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(declaration.value_components(), &components);
        assert!(declaration.position().is_none());
        assert!(declaration.parsed_name().is_none());
        assert!(declaration.parsed_value().is_none());
        let CssKnownPropertyValueRef::Width(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("the nonnegative width owner");
        };
        let CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(value)) = value.value() else {
            panic!("an exact literal length");
        };
        assert_eq!(value.literal_component(), Some(&component));
        assert_eq!(value.origin(), component.origin());
        atomic_bytes(&declaration, EXPECTED);

        // A valid prefix cannot make a residual value complete. Supplied
        // components remain reusable; the successful declaration is immutable.
        let residual = CssComponentValues::try_new(vec![
            component.clone(),
            CssComponentValue::try_ident("auto").unwrap(),
        ])
        .unwrap();
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Width),
                residual.clone(),
                CssImportance::Normal
            )
            .is_err()
        );
        assert_eq!(declaration.value_components(), &components);
        assert_eq!(declaration.to_specified_css().unwrap(), EXPECTED);
    }
    let negative = CssComponentValue::try_dimension("-1e-999", "px").unwrap();
    assert_eq!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(negative.clone())
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::OutOfRange,
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            CssComponentValues::try_new(vec![negative]).unwrap(),
            CssImportance::Normal,
        )
        .is_err()
    );
    let report = parity(r"width:+2\50 X!important;width:-1e-999px;height:3px");
    let [width, height] = report.syntax().as_slice() else {
        panic!("the exact negative is rejected rather than underflowed to zero");
    };
    assert_eq!(width.to_specified_css().unwrap(), EXPECTED);
    assert_eq!(height.known().unwrap().property(), CssKnownProperty::Height);
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected range atom");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
}

#[test]
fn a_greedy_imported_position_stops_before_the_whole_angle_function_and_rejects_residuals() {
    const EXPECTED: &str =
        "offset-path: ray(calc(270deg) contain at right 1px bottom 2%) !important;";
    for authored in [
        "ray(at bottom 2% right 1px calc(1turn - 90deg) contain)",
        "ray(calc(1turn - 90deg) contain at bottom 2% right 1px)",
    ] {
        let source = format!("offset-path:{authored}!important;width:2px");
        let report = parity(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [native, width] = report.syntax().as_slice() else {
            panic!("ray and later sibling");
        };
        assert_eq!(width.known().unwrap().property(), CssKnownProperty::Width);
        let supplied = parse_component_values(authored).unwrap();
        let checked = parse_property_value_for_grammar(
            CssKnownProperty::OffsetPath.grammar(),
            supplied.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &supplied);
        for declaration in [native, &checked] {
            let CssKnownPropertyValueRef::OffsetPath(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("offset-path");
            };
            let CssOffsetPathRef::Path(path) = value.value().view() else {
                panic!("one actual path");
            };
            let CssOffsetPathKind::Ray(ray) = path.path() else {
                panic!("ray");
            };
            assert!(path.coord_box().is_none());
            assert!(ray.size().is_none());
            assert!(ray.contain());
            let position = ray.position().unwrap();
            let CssPositionRef::Cartesian(position) = position.view() else {
                panic!("Cartesian motion position")
            };
            let CssHorizontalPosition::RightOffset(right) = position.horizontal() else {
                panic!("right edge offset");
            };
            let CssVerticalPosition::BottomOffset(bottom) = position.vertical() else {
                panic!("bottom edge offset");
            };
            assert_eq!(right.serialize_specified().unwrap(), "1px");
            assert_eq!(bottom.serialize_specified().unwrap(), "2%");
            assert!(ray.angle().calculation().is_some());
            assert_eq!(
                ray.serialize_specified().unwrap(),
                "ray(calc(270deg) contain at right 1px bottom 2%)"
            );
            let original_source = if declaration.position().is_some() {
                source.as_str()
            } else {
                authored
            };
            let opener = original_source.find("calc(").unwrap();
            original(ray.angle().origin(), original_source, opener, opener + 5);
            let right_start = original_source.find("1px").unwrap();
            original(
                right.origin(),
                original_source,
                right_start,
                right_start + 3,
            );
            let bottom_start = original_source.find("2%").unwrap();
            original(
                bottom.origin(),
                original_source,
                bottom_start,
                bottom_start + 2,
            );
            atomic_bytes(declaration, EXPECTED);
        }
    }
    for invalid in [
        "ray(at bottom 2% right 1px calc(1turn - 90deg) contain, sides)",
        "ray(at bottom 2% right 1px calc(1turn - 90deg) contain contain)",
        "ray(at bottom 2% right 1px calc(1turn - 90deg) contain) garbage",
    ] {
        let source = format!("color:red;offset-path:{invalid};width:2px");
        let report = parity(&source);
        let [color, width] = report.syntax().as_slice() else {
            panic!("only the malformed whole value is rejected");
        };
        assert_eq!(color.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(width.known().unwrap().property(), CssKnownProperty::Width);
        let [diagnostic] = report.diagnostics() else {
            panic!("one whole-production failure");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 10);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            23 + invalid.len()
        );
        let supplied = parse_component_values(invalid).unwrap();
        assert!(
            parse_property_value_for_grammar(
                CssKnownProperty::OffsetPath.grammar(),
                supplied,
                CssImportance::Important
            )
            .is_err()
        );
    }
}
