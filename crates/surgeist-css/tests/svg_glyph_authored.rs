#![forbid(unsafe_code)]
//! Functional new-API evidence from frozen WebKit73aa6c89 and the adopted
//! independent SVG contract. No executable preimplementation RED is claimed.
use surgeist_css::*;

const SVG: CssParserContext =
    CssParserContext::new(CssParserMode::Standards).with_svg_glyph_orientation_vertical();
const QUIRKS: CssParserContext =
    CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
const NAME: CssPropertyNameRef<'static> = CssPropertyNameRef::SvgGlyphOrientationVertical;
fn declaration(context: CssParserContext, value: &str) -> CssDeclaration {
    let report = context.parse_declaration(&format!("glyph-orientation-vertical:{value}"));
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    report.syntax().as_ref().unwrap().clone()
}
fn svg(source: &CssDeclaration) -> &CssSvgGlyphOrientationVerticalDeclaration {
    assert_eq!(source.property_name(), NAME);
    assert!(source.known().is_none() && source.custom().is_none());
    source.svg_glyph_orientation_vertical().unwrap()
}
fn query(condition: &CssSupportsCondition) -> &CssSupportsDeclaration {
    let CssSupportsConditionKind::Declaration(value) = condition.kind() else {
        panic!("declaration test")
    };
    value
}
fn original(origin: &CssValueOrigin, source: &str) -> CssSourceSpan {
    let CssValueOrigin::Parsed(value) = origin else {
        panic!("parsed origin: {origin:?}")
    };
    assert_eq!(value.source().as_str(), source);
    value.span()
}
fn pending(source: &CssDeclaration) -> CssPendingSubstitution {
    let CssExpansion::Pending(value) = expand_declaration(source).unwrap() else {
        panic!("pending")
    };
    value
}
fn completed(source: &CssDeclaration) -> CssSvgGlyphOrientationVerticalContribution {
    let CssExpansion::Contributions(CssContributions::SvgGlyphOrientationVertical(value)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one independent SVG contribution")
    };
    value
}
fn rejected_literal(
    error: &CssPropertyValueParseError,
    text: &str,
    kind: CssTokenKind,
    component: &CssComponentValue,
) {
    let CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedToken(detail)) = error.kind()
    else {
        panic!("{text}: exact present-token rejection: {error:?}")
    };
    assert_eq!(detail.encountered().kind(), kind, "{text}");
    assert_eq!(detail.encountered().authored(), text, "{text}");
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(component.origin().clone()),
        "{text}"
    );
}
fn rejected_source_token(error: &Error, spelling: &str, kind: CssTokenKind, offset: usize) {
    let ErrorKind::UnexpectedToken(detail) = error.kind() else {
        panic!("{spelling}: exact source-token rejection: {error:?}")
    };
    assert_eq!(detail.encountered().kind(), kind);
    assert_eq!(detail.encountered().authored(), spelling);
    assert_eq!(error.position().byte_offset().value(), offset);
}

#[test]
fn present_svg_tokens_and_outer_number_roots_keep_exact_rejection_origins() {
    fn responsible_component<'a>(
        values: &'a CssComponentValues,
        path: &[usize],
    ) -> &'a CssComponentValue {
        let mut component = &values.items()[path[0]];
        for &index in &path[1..] {
            let children = match component.view() {
                CssComponentValueRef::Function(function) => function.values(),
                CssComponentValueRef::Block(block) => block.values(),
                _ => panic!("independently expected nested component path"),
            };
            component = &children.items()[index];
        }
        component
    }
    // Literal token spellings, offsets and component indices are independent
    // grammar/origin oracles, including unchanged next-token exhaustion controls.
    for (text, spelling, kind, index, offset) in [
        ("sideways", "sideways", CssTokenKind::Ident, 0, 0),
        (r"\73 ideways", r"\73 ideways", CssTokenKind::Ident, 0, 0),
        ("/*😀*/ sideways", "sideways", CssTokenKind::Ident, 2, 9),
        (" /*x*/ sideways", "sideways", CssTokenKind::Ident, 3, 7),
        ("10%", "10%", CssTokenKind::Percentage, 0, 0),
        ("\"auto\"", "\"auto\"", CssTokenKind::String, 0, 0),
        ("#abc", "#abc", CssTokenKind::IdHash, 0, 0),
        ("#123", "#123", CssTokenKind::Hash, 0, 0),
        ("url(a)", "url(a)", CssTokenKind::Url, 0, 0),
        ("+", "+", CssTokenKind::Delim, 0, 0),
        ("@foo", "@foo", CssTokenKind::AtKeyword, 0, 0),
        ("(90deg)", "(", CssTokenKind::ParenthesisBlock, 0, 0),
        ("[90deg]", "[", CssTokenKind::SquareBracketBlock, 0, 0),
        ("{90deg}", "{", CssTokenKind::CurlyBracketBlock, 0, 0),
        ("calc(0)", "calc(", CssTokenKind::Function, 0, 0),
        ("calc(90)", "calc(", CssTokenKind::Function, 0, 0),
        ("auto 90deg", "90deg", CssTokenKind::Dimension, 2, 5),
        ("90deg 0", "0", CssTokenKind::Number, 2, 6),
        ("inherit auto", "auto", CssTokenKind::Ident, 2, 8),
    ] {
        let values = parse_component_values(text).unwrap();
        let before = values.clone();
        for context in [SVG, QUIRKS] {
            let checked = context
                .parse_property_value(NAME, values.clone(), CssImportance::Normal)
                .unwrap_err();
            rejected_literal(&checked, spelling, kind, &values.items()[index]);
            let attribute = context
                .parse_svg_glyph_orientation_vertical_attribute_components(values.clone())
                .unwrap_err();
            rejected_literal(&attribute, spelling, kind, &values.items()[index]);
            for report in [
                context.parse_property_value_text(text, NAME, CssImportance::Normal),
                context.parse_svg_glyph_orientation_vertical_attribute_value(text),
            ] {
                assert!(report.syntax().is_none() && !report.is_clean(), "{text}");
                rejected_source_token(report.diagnostics()[0].error(), spelling, kind, offset);
            }
            let source = format!("glyph-orientation-vertical:{text}");
            let report = context.parse_declaration(&source);
            assert!(report.syntax().is_none() && !report.is_clean(), "{text}");
            rejected_source_token(
                report.diagnostics()[0].error(),
                spelling,
                kind,
                "glyph-orientation-vertical:".len() + offset,
            );
        }
        assert_eq!(values, before);
        assert_eq!(values.serialize().unwrap().as_css(), text);
    }
    for argument in ["0", "90"] {
        let component =
            CssComponentValue::try_function("calc", parse_component_values(argument).unwrap())
                .unwrap();
        assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
        let values = CssComponentValues::try_new(vec![component.clone()]).unwrap();
        let native = CssAngleCalculation::try_from_components(values.clone()).unwrap_err();
        assert_eq!(
            native.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert_eq!(native.origin(), Some(component.origin()));
        assert_eq!(native.path(), Some(&[0][..]));
        for context in [SVG, QUIRKS] {
            let error = context
                .parse_property_value(NAME, values.clone(), CssImportance::Normal)
                .unwrap_err();
            rejected_literal(&error, "calc(", CssTokenKind::Function, &component);
        }
        let CssComponentValueRef::Function(function) = component.view() else {
            unreachable!()
        };
        original(function.values().items()[0].origin(), argument);
    }
    for (block_kind, spelling, kind) in [
        (
            CssBlockKind::Parenthesis,
            "(",
            CssTokenKind::ParenthesisBlock,
        ),
        (
            CssBlockKind::SquareBracket,
            "[",
            CssTokenKind::SquareBracketBlock,
        ),
        (
            CssBlockKind::CurlyBracket,
            "{",
            CssTokenKind::CurlyBracketBlock,
        ),
    ] {
        let child_values = parse_component_values("90deg").unwrap();
        let block = CssComponentValue::try_block(block_kind, child_values.clone()).unwrap();
        let block_values = CssComponentValues::try_new(vec![block.clone()]).unwrap();
        let native = CssAngleCalculation::try_from_components(block_values.clone()).unwrap_err();
        assert_eq!(
            native.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert_eq!(native.path(), Some(&[0][..]));
        assert_eq!(native.origin(), Some(&CssValueOrigin::Programmatic));
        for context in [SVG, QUIRKS] {
            for error in [
                context
                    .parse_property_value(NAME, block_values.clone(), CssImportance::Normal)
                    .unwrap_err(),
                context
                    .parse_svg_glyph_orientation_vertical_attribute_components(block_values.clone())
                    .unwrap_err(),
            ] {
                rejected_literal(&error, spelling, kind, &block);
            }
        }
        let CssComponentValueRef::Block(block_view) = block.view() else {
            unreachable!()
        };
        assert_eq!(block_view.values(), &child_values);
        original(block_view.values().items()[0].origin(), "90deg");
    }
    let string = CssComponentValue::try_token("\"auto\"").unwrap();
    let native = CssAngleLiteral::try_from_component(string.clone()).unwrap_err();
    assert_eq!(native.kind(), CssComponentValueErrorKind::InvalidToken);
    assert_eq!(native.origin(), string.origin());
    let string_values = CssComponentValues::try_new(vec![string.clone()]).unwrap();
    for context in [SVG, QUIRKS] {
        let error = context
            .parse_property_value(NAME, string_values.clone(), CssImportance::Normal)
            .unwrap_err();
        rejected_literal(&error, "\"auto\"", CssTokenKind::String, &string);
    }
    for (text, spelling, kind, offset, path, native_kind) in [
        (
            "foo(90deg)",
            "foo(",
            CssTokenKind::Function,
            0,
            &[0][..],
            CssNumericConstructionErrorKind::UnknownFunction,
        ),
        (
            "calc(90deg + 1px)",
            "1px",
            CssTokenKind::Dimension,
            13,
            &[0, 4][..],
            CssNumericConstructionErrorKind::IncompatibleTypes,
        ),
        (
            "calc(90deg + foo(1deg))",
            "foo(",
            CssTokenKind::Function,
            13,
            &[0, 4][..],
            CssNumericConstructionErrorKind::UnknownFunction,
        ),
        (
            "calc(90deg + 1foo)",
            "1foo",
            CssTokenKind::Dimension,
            13,
            &[0, 4][..],
            CssNumericConstructionErrorKind::InvalidArgumentType,
        ),
        (
            "calc(90deg 1deg)",
            "1deg",
            CssTokenKind::Dimension,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg 0)",
            "0",
            CssTokenKind::Number,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg 10%)",
            "10%",
            CssTokenKind::Percentage,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg abs(1deg))",
            "abs(",
            CssTokenKind::Function,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg (1deg))",
            "(",
            CssTokenKind::ParenthesisBlock,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg pi)",
            "pi",
            CssTokenKind::Ident,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(abs(90deg 1deg))",
            "1deg",
            CssTokenKind::Dimension,
            15,
            &[0, 0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(\"auto\")",
            "\"auto\"",
            CssTokenKind::String,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(url(a))",
            "url(a)",
            CssTokenKind::Url,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(#abc)",
            "#abc",
            CssTokenKind::IdHash,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(@foo)",
            "@foo",
            CssTokenKind::AtKeyword,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc([90deg])",
            "[",
            CssTokenKind::SquareBracketBlock,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc({90deg})",
            "{",
            CssTokenKind::CurlyBracketBlock,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(%)",
            "%",
            CssTokenKind::Delim,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(+)",
            "+",
            CssTokenKind::Delim,
            5,
            &[0, 0][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "min(90deg,+)",
            "+",
            CssTokenKind::Delim,
            10,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg \"auto\")",
            "\"auto\"",
            CssTokenKind::String,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg foo)",
            "foo",
            CssTokenKind::Ident,
            11,
            &[0, 2][..],
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
    ] {
        let values = parse_component_values(text).unwrap();
        let CssComponentValueRef::Function(function) = values.items()[0].view() else {
            unreachable!()
        };
        let responsible = responsible_component(&values, path);
        let native = CssAngleCalculation::try_from_components(values.clone()).unwrap_err();
        assert_eq!(native.kind(), &native_kind);
        assert_eq!(native.path(), Some(path));
        assert_eq!(native.origin(), Some(responsible.origin()));
        let outer =
            CssComponentValue::try_function(function.name(), function.values().clone()).unwrap();
        let mixed = CssComponentValues::try_new(vec![outer.clone()]).unwrap();
        let mixed_responsible = responsible_component(&mixed, path);
        assert_eq!(outer.origin(), &CssValueOrigin::Programmatic);
        if path.len() > 1 {
            original(mixed_responsible.origin(), text);
        }
        let mixed_native = CssAngleCalculation::try_from_components(mixed.clone()).unwrap_err();
        assert_eq!(mixed_native.kind(), &native_kind);
        assert_eq!(mixed_native.path(), Some(path));
        assert_eq!(mixed_native.origin(), Some(mixed_responsible.origin()));
        for context in [SVG, QUIRKS] {
            for (components, responsible) in [(&values, responsible), (&mixed, mixed_responsible)] {
                let error = context
                    .parse_property_value(NAME, components.clone(), CssImportance::Normal)
                    .unwrap_err();
                rejected_literal(&error, spelling, kind, responsible);
                let attribute = context
                    .parse_svg_glyph_orientation_vertical_attribute_components(components.clone())
                    .unwrap_err();
                rejected_literal(&attribute, spelling, kind, responsible);
            }
            for report in [
                context.parse_property_value_text(text, NAME, CssImportance::Normal),
                context.parse_svg_glyph_orientation_vertical_attribute_value(text),
            ] {
                assert!(report.syntax().is_none() && !report.is_clean(), "{text}");
                rejected_source_token(report.diagnostics()[0].error(), spelling, kind, offset);
            }
            let source = format!("glyph-orientation-vertical:{text}");
            let report = context.parse_declaration(&source);
            assert!(report.syntax().is_none() && !report.is_clean(), "{text}");
            rejected_source_token(
                report.diagnostics()[0].error(),
                spelling,
                kind,
                "glyph-orientation-vertical:".len() + offset,
            );
        }
        assert_eq!(values.serialize().unwrap().as_css(), text);
        assert_eq!(mixed.serialize().unwrap().as_css(), text);
    }
    // Missing bounded arguments retain provider diagnostics. In particular,
    // the numeric cursor falls back to '+' when there is no right operand;
    // that origin is not evidence that a rejected operand was present.
    for (text, path, offset, native_kind) in [
        (
            "calc(90deg +)",
            &[0, 2][..],
            11,
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc()",
            &[0][..],
            0,
            CssNumericConstructionErrorKind::Arity,
        ),
        (
            "calc(90deg +abs(1deg))",
            &[0, 3][..],
            12,
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(90deg+ abs(1deg))",
            &[0, 1][..],
            10,
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
        (
            "calc(abs(90deg +abs(1deg)))",
            &[0, 0, 3][..],
            16,
            CssNumericConstructionErrorKind::MalformedExpression,
        ),
    ] {
        let values = parse_component_values(text).unwrap();
        let responsible = responsible_component(&values, path);
        let native = CssAngleCalculation::try_from_components(values.clone()).unwrap_err();
        assert_eq!(native.kind(), &native_kind);
        assert_eq!(native.path(), Some(path));
        assert_eq!(native.origin(), Some(responsible.origin()));
        for context in [SVG, QUIRKS] {
            let error = context
                .parse_property_value(NAME, values.clone(), CssImportance::Normal)
                .unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ),
                "{text}: {error:?}"
            );
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::Token(responsible.origin().clone())
            );
            for report in [
                context.parse_property_value_text(text, NAME, CssImportance::Normal),
                context.parse_svg_glyph_orientation_vertical_attribute_value(text),
            ] {
                assert!(report.syntax().is_none() && !report.is_clean());
                assert!(matches!(
                    report.diagnostics()[0].error().kind(),
                    ErrorKind::UnexpectedEnd(_)
                ));
                assert_eq!(
                    report.diagnostics()[0]
                        .error()
                        .position()
                        .byte_offset()
                        .value(),
                    offset
                );
            }
        }
    }
    for text in ["", " "] {
        let values = parse_component_values(text).unwrap();
        let expected =
            CssSerializedOrigin::End(values.items().last().map(|value| value.origin().clone()));
        for context in [SVG, QUIRKS] {
            let error = context
                .parse_property_value(NAME, values.clone(), CssImportance::Normal)
                .unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ),
                "{text:?}: {error:?}"
            );
            assert_eq!(error.origin(), &expected);
            let report = context.parse_property_value_text(text, NAME, CssImportance::Normal);
            assert!(report.syntax().is_none() && !report.is_clean());
            assert!(matches!(
                report.diagnostics()[0].error().kind(),
                ErrorKind::UnexpectedEnd(_)
            ));
            assert_eq!(
                report.diagnostics()[0]
                    .error()
                    .position()
                    .byte_offset()
                    .value(),
                text.len()
            );
        }
    }
}

#[test]
fn selection_preserves_modes_finite_alias_and_explicit_definition_precedence() {
    assert_eq!(SVG.with_svg_glyph_orientation_vertical(), SVG);
    assert_ne!(SVG, CssParserContext::default());
    assert_eq!(QUIRKS.mode(), CssParserMode::Quirks);
    for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        for (text, expected) in [
            ("auto", CssTextOrientation::Mixed),
            ("0", CssTextOrientation::Upright),
            ("90", CssTextOrientation::Sideways),
            ("9e1deg", CssTextOrientation::Sideways),
        ] {
            let source = declaration(CssParserContext::new(mode), text);
            let CssKnownPropertyValueRef::TextOrientation(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("finite target")
            };
            assert_eq!(value.orientation(), &expected);
            assert!(source.svg_glyph_orientation_vertical().is_none());
        }
    }
    for text in [
        "180deg",
        "1e-100deg",
        "90.000001deg",
        "0rad",
        "0.0",
        "calc(90deg)",
    ] {
        let report = CssParserContext::default()
            .parse_declaration(&format!("glyph-orientation-vertical:{text}"));
        assert!(report.syntax().is_none() && !report.is_clean(), "{text}");
    }
    assert!(svg(&declaration(SVG, "auto")).value().unwrap().is_auto());
    for text in [r"\61 uto", "AuTo"] {
        let source = declaration(SVG, text);
        assert!(svg(&source).value().unwrap().is_auto());
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            text
        );
    }
    let escaped = SVG.parse_declaration(r"\67 lyph-orientation-vertical:180deg");
    assert!(escaped.is_clean());
    assert_eq!(escaped.syntax().as_ref().unwrap().property_name(), NAME);
    let source = declaration(SVG, "180deg");
    assert_eq!(completed(&source).property(), NAME);
    assert!(completed(&source).source().same_occurrence(&source));
    assert_eq!(
        completed(&source)
            .ordinary_value()
            .unwrap()
            .angle()
            .unwrap()
            .literal()
            .unwrap()
            .numeric()
            .representation(),
        "180"
    );
    let finite = CssPropertyGrammar::from_name("glyph-orientation-vertical").unwrap();
    let explicit = QUIRKS
        .parse_property_value_for_grammar(
            finite,
            parse_component_values("90").unwrap(),
            CssImportance::Important,
        )
        .unwrap();
    assert_eq!(explicit.known().unwrap().grammar(), finite);
    assert!(explicit.svg_glyph_orientation_vertical().is_none());
    let explicit_text =
        SVG.parse_property_value_text_for_grammar("90deg", finite, CssImportance::Normal);
    assert_eq!(
        explicit_text
            .syntax()
            .as_ref()
            .unwrap()
            .known()
            .unwrap()
            .grammar(),
        finite
    );
    let named = CssParserContext::default()
        .parse_property_value(
            NAME,
            parse_component_values("180deg").unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
    assert_eq!(svg(&named).parser_mode(), CssParserMode::Standards);
    assert_eq!(
        svg(&named)
            .value()
            .unwrap()
            .angle()
            .unwrap()
            .literal()
            .unwrap()
            .unit(),
        CssAngleUnit::Degrees
    );
}

#[test]
fn all_authored_fronts_preserve_angle_units_coefficients_origins_and_numeric_flags() {
    for (text, coefficient, unit) in [
        ("45deg", "45", CssAngleUnit::Degrees),
        ("180deg", "180", CssAngleUnit::Degrees),
        ("270deg", "270", CssAngleUnit::Degrees),
        ("0rad", "0", CssAngleUnit::Radians),
        ("100grad", "100", CssAngleUnit::Gradians),
        (".25turn", ".25", CssAngleUnit::Turns),
        ("1e-400deg", "1e-400", CssAngleUnit::Degrees),
        (r"90d\65g", "90", CssAngleUnit::Degrees),
    ] {
        for context in [SVG, QUIRKS] {
            let parsed = declaration(context, text);
            let raw = context.parse_property_value_text(text, NAME, CssImportance::Important);
            assert!(raw.is_clean());
            let values = parse_component_values(text).unwrap();
            let rust =
                CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()])
                    .unwrap();
            for source in [
                parsed,
                raw.syntax().as_ref().unwrap().clone(),
                context
                    .parse_property_value(NAME, values.clone(), CssImportance::Important)
                    .unwrap(),
                context
                    .parse_property_value(NAME, rust.clone(), CssImportance::Important)
                    .unwrap(),
            ] {
                let body = svg(&source);
                assert_eq!(body.parser_mode(), context.mode());
                assert!(!body.is_presentation_attribute());
                let angle = body.value().unwrap().angle().unwrap().literal().unwrap();
                assert_eq!(angle.numeric().representation(), coefficient);
                assert_eq!(angle.unit(), unit);
                assert_eq!(
                    source.value_components().serialize().unwrap().as_css(),
                    text
                );
            }
            let attribute = context.parse_svg_glyph_orientation_vertical_attribute_value(text);
            assert!(attribute.is_clean());
            for source in [
                attribute.syntax().as_ref().unwrap().clone(),
                context
                    .parse_svg_glyph_orientation_vertical_attribute_components(rust.clone())
                    .unwrap(),
            ] {
                let body = svg(&source);
                assert!(body.is_presentation_attribute());
                assert_eq!(body.parser_mode(), context.mode());
                assert_eq!(source.importance(), CssImportance::Normal);
                let literal = body.value().unwrap().angle().unwrap().literal().unwrap();
                assert_eq!(literal.numeric().representation(), coefficient);
                assert_eq!(literal.unit(), unit);
            }
            original(values.items()[0].origin(), text);
            assert_eq!(rust.items()[0].origin(), &CssValueOrigin::Programmatic);
        }
    }
    for (text, kind) in [
        ("0", CssNumericTokenKind::Integer),
        ("-0", CssNumericTokenKind::Integer),
        ("0.0", CssNumericTokenKind::Number),
        (".0", CssNumericTokenKind::Number),
        ("0e0", CssNumericTokenKind::Number),
        ("1e-400", CssNumericTokenKind::Number),
    ] {
        for context in [SVG, QUIRKS] {
            let source = declaration(context, text);
            let component = svg(&source).value().unwrap().unitless_component().unwrap();
            let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view()
            else {
                panic!("Number leaf")
            };
            assert_eq!(number.kind(), kind);
            assert_eq!(number.representation(), text);
            original(
                component.origin(),
                &format!("glyph-orientation-vertical:{text}"),
            );
        }
    }
    for text in ["90", "90.0", "9e1", "-135", "45.0001", "1e308"] {
        for context in [SVG, QUIRKS] {
            let values = parse_component_values(text).unwrap();
            let result = context.parse_property_value(NAME, values.clone(), CssImportance::Normal);
            if context == SVG {
                let error = result.unwrap_err();
                rejected_literal(&error, text, CssTokenKind::Number, &values.items()[0]);
                let raw = context.parse_property_value_text(text, NAME, CssImportance::Normal);
                assert!(raw.syntax().is_none() && !raw.is_clean());
                let parsed =
                    context.parse_declaration(&format!("glyph-orientation-vertical:{text}"));
                assert!(parsed.syntax().is_none() && !parsed.is_clean());
            } else {
                assert_eq!(
                    svg(&result.unwrap()).value().unwrap().unitless_component(),
                    Some(&values.items()[0])
                );
                let parsed = declaration(context, text);
                assert_eq!(
                    parsed.value_components().serialize().unwrap().as_css(),
                    text
                );
                let raw = context.parse_property_value_text(text, NAME, CssImportance::Important);
                assert!(raw.is_clean());
                assert!(
                    svg(raw.syntax().as_ref().unwrap())
                        .value()
                        .unwrap()
                        .unitless_component()
                        .is_some()
                );
            }
            let attr = context
                .parse_svg_glyph_orientation_vertical_attribute_components(values.clone())
                .unwrap();
            assert!(svg(&attr).is_presentation_attribute());
            assert_eq!(svg(&attr).parser_mode(), context.mode());
            assert_eq!(
                svg(&attr).value().unwrap().unitless_component(),
                Some(&values.items()[0])
            );
            assert_eq!(attr.importance(), CssImportance::Normal);
            let raw = CssParserContext::new(context.mode())
                .parse_svg_glyph_orientation_vertical_attribute_value(text);
            assert!(raw.is_clean());
            assert!(svg(raw.syntax().as_ref().unwrap()).is_presentation_attribute());
        }
    }
}

#[test]
fn intrinsic_constructors_reject_wrong_roots_raw_overflow_and_recovered_math() {
    let auto = CssSvgGlyphOrientationVerticalValue::auto();
    assert!(auto.is_auto());
    assert!(auto.angle().is_none() && auto.unitless_component().is_none());
    for text in ["90", "90.0", "9e1", "1e-400"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        assert_eq!(
            CssSvgGlyphOrientationVerticalValue::try_from_unitless(component.clone())
                .unwrap()
                .unitless_component(),
            Some(&component)
        );
    }
    for text in ["auto", "90deg", "90%", "calc(90)"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        let error =
            CssSvgGlyphOrientationVerticalValue::try_from_unitless(component.clone()).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert_eq!(error.origin(), Some(component.origin()));
    }
    for text in ["1e309", "-1e1000"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        let error =
            CssSvgGlyphOrientationVerticalValue::try_from_unitless(component.clone()).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert_eq!(error.origin(), Some(component.origin()));
    }
    let angle = CssAngleValue::from_literal(
        CssAngleLiteral::try_new("1e1000", CssAngleUnit::Degrees).unwrap(),
    );
    let error = CssSvgGlyphOrientationVerticalValue::try_from_angle(angle.clone()).unwrap_err();
    assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
    assert_eq!(error.origin(), Some(angle.origin()));
    for text in [
        "1e1000deg",
        "1e1000",
        "0foo",
        "10%",
        "auto 90deg",
        "90deg 0",
        "inherit auto",
        "calc(0)",
        "calc(90)",
    ] {
        for context in [SVG, QUIRKS] {
            let values = parse_component_values(text).unwrap();
            let error = context
                .parse_property_value(NAME, values.clone(), CssImportance::Normal)
                .unwrap_err();
            assert!(
                matches!(error.kind(), CssPropertyValueErrorKind::Grammar(_)),
                "{text}: {error:?}"
            );
            assert_eq!(values.serialize().unwrap().as_css(), text);
            let report = context.parse_svg_glyph_orientation_vertical_attribute_value(text);
            assert!(report.syntax().is_none() && !report.is_clean());
        }
    }
    for text in [
        "calc(90deg)",
        "calc(15deg + 15deg)",
        "min(10deg, 20deg)",
        "max(10deg, 20deg)",
        "clamp(0deg, 10deg, 20deg)",
        "calc(infinity * 1deg)",
        "calc(NaN * 1deg)",
    ] {
        let source = declaration(SVG, text);
        let value = svg(&source).value().unwrap();
        assert!(value.angle().unwrap().calculation().is_some());
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            text
        );
        let native =
            CssSvgGlyphOrientationVerticalValue::try_from_angle(value.angle().unwrap().clone())
                .unwrap();
        assert_eq!(native, *value);
        for context in [SVG, QUIRKS] {
            let attribute = context.parse_svg_glyph_orientation_vertical_attribute_value(text);
            assert!(attribute.is_clean());
            assert!(
                svg(attribute.syntax().as_ref().unwrap())
                    .value()
                    .unwrap()
                    .angle()
                    .unwrap()
                    .calculation()
                    .is_some()
            );
            let checked = context
                .parse_property_value(
                    NAME,
                    parse_component_values(text).unwrap(),
                    CssImportance::Normal,
                )
                .unwrap();
            assert!(
                svg(&checked)
                    .value()
                    .unwrap()
                    .angle()
                    .unwrap()
                    .calculation()
                    .is_some()
            );
        }
    }
    let recovered = declaration_recovered("calc(90deg");
    let angle = svg(&recovered).value().unwrap().angle().unwrap().clone();
    let error = CssSvgGlyphOrientationVerticalValue::try_from_angle(angle).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
    assert!(matches!(
        error.origin(),
        Some(CssValueOrigin::ImplicitClosure { .. })
    ));
}

#[test]
fn binary64_raw_boundaries_never_replace_exact_original_coefficients() {
    for text in [
        "1.7976931348623157e308deg",
        "5e-324deg",
        "1e-324deg",
        "-1e-400deg",
    ] {
        let source = declaration(SVG, text);
        let literal = svg(&source)
            .value()
            .unwrap()
            .angle()
            .unwrap()
            .literal()
            .unwrap();
        assert_eq!(
            literal.numeric().representation(),
            text.strip_suffix("deg").unwrap()
        );
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            text
        );
    }
    let tiny = declaration(SVG, "1e-324");
    assert_eq!(
        tiny.value_components().serialize().unwrap().as_css(),
        "1e-324"
    );
    assert!(svg(&tiny).value().unwrap().unitless_component().is_some());
    for text in ["5e-324", "1.7976931348623157e308"] {
        let values = parse_component_values(text).unwrap();
        let error = SVG
            .parse_property_value(NAME, values.clone(), CssImportance::Normal)
            .unwrap_err();
        rejected_literal(&error, text, CssTokenKind::Number, &values.items()[0]);
        let source = QUIRKS
            .parse_property_value(NAME, values.clone(), CssImportance::Normal)
            .unwrap();
        assert_eq!(
            svg(&source).value().unwrap().unitless_component(),
            Some(&values.items()[0])
        );
    }
    for text in [
        "1.7976931348623159e308deg",
        "-1.7976931348623159e308deg",
        "1.7976931348623159e308",
    ] {
        let values = parse_component_values(text).unwrap();
        let before = values.clone();
        let error = QUIRKS
            .parse_property_value(NAME, values.clone(), CssImportance::Normal)
            .unwrap_err();
        let kind = if text.ends_with("deg") {
            CssTokenKind::Dimension
        } else {
            CssTokenKind::Number
        };
        rejected_literal(&error, text, kind, &values.items()[0]);
        assert_eq!(values, before);
    }
}
fn declaration_recovered(text: &str) -> CssDeclaration {
    let report = SVG.parse_declaration(&format!("glyph-orientation-vertical:{text}"));
    assert!(!report.is_clean());
    report.syntax().as_ref().unwrap().clone()
}

#[test]
fn attribute_root_boundaries_precede_globals_and_pending_and_keep_fixed_importance() {
    for text in [
        "auto!important",
        "initial!important",
        "var(--g)!important",
        "90;",
        "90}",
    ] {
        let report = SVG.parse_svg_glyph_orientation_vertical_attribute_value(text);
        assert!(report.syntax().is_none() && !report.is_clean(), "{text}");
        if let Ok(values) = parse_component_values(text) {
            let error = SVG
                .parse_svg_glyph_orientation_vertical_attribute_components(values)
                .unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedToken(_))
                ),
                "{text}: {error:?}"
            );
        }
    }
    let report =
        SVG.parse_svg_glyph_orientation_vertical_attribute_value("var(--g, \"!important\")");
    let source = report.syntax().as_ref().unwrap();
    assert!(report.is_clean());
    assert!(svg(source).substitution_dependent().is_some());
    assert_eq!(source.importance(), CssImportance::Normal);
    assert!(source.parsed_name().is_none());
    assert_eq!(
        source.parsed_value().unwrap().source().as_str(),
        "var(--g, \"!important\")"
    );
}

#[test]
fn checked_original_closure_utf16_mixed_sources_and_retry_keep_real_origins() {
    for text in [
        "auto/*",
        "initial/*",
        "var(--g",
        "calc(90deg",
        "var(--g, calc(1deg)",
    ] {
        let values = parse_component_values(text).unwrap();
        let before = values.clone();
        for attribute in [false, true] {
            let error = if attribute {
                SVG.parse_svg_glyph_orientation_vertical_attribute_components(values.clone())
            } else {
                SVG.parse_property_value(NAME, values.clone(), CssImportance::Important)
            }
            .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { opening, at })) =
                error.origin()
            else {
                panic!("original EOF")
            };
            assert_eq!(opening.source().as_str(), text);
            assert!(opening.source().same_snapshot(at.source()));
            assert_eq!(at.span().start().byte_offset().value(), text.len());
            assert_eq!(at.span().start(), at.span().end());
        }
        assert_eq!(values, before);
    }
    let comment = parse_component_values("/*").unwrap();
    let mixed = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("auto").unwrap(),
        comment.items()[0].clone(),
    ])
    .unwrap();
    let error = SVG
        .parse_property_value(NAME, mixed, CssImportance::Normal)
        .unwrap_err();
    let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { opening, at })) =
        error.origin()
    else {
        panic!("mixed EOF")
    };
    assert_eq!(opening.source().as_str(), "/*");
    assert_eq!(at.span().start().byte_offset().value(), 2);
    let css = "/*😀*/ GLYPH-ORIENTATION-VERTICAL: +090.0deg !important";
    let source = SVG
        .parse_declaration(css)
        .syntax()
        .as_ref()
        .unwrap()
        .clone();
    let name = source.parsed_name().unwrap();
    assert_eq!(name.span().start().byte_offset().value(), 9);
    assert_eq!(name.span().start().column().value(), 7);
    assert_eq!(source.importance(), CssImportance::Important);
    let angle = svg(&source)
        .value()
        .unwrap()
        .angle()
        .unwrap()
        .literal()
        .unwrap();
    let span = original(angle.origin(), css);
    assert_eq!(
        span.start().byte_offset().value(),
        css.find("+090.0deg").unwrap()
    );
    assert_eq!(
        span.end().byte_offset().value(),
        css.find("+090.0deg").unwrap() + 9
    );
    assert_eq!(angle.numeric().representation(), "+090.0");
    let parsed = parse_component_values("/*😀*/180deg").unwrap();
    let mixed = CssComponentValues::try_new(vec![
        CssComponentValue::try_token(" ").unwrap(),
        parsed.items().last().unwrap().clone(),
    ])
    .unwrap();
    let checked = SVG
        .parse_property_value(NAME, mixed.clone(), CssImportance::Normal)
        .unwrap();
    assert!(
        checked.position().is_none()
            && checked.parsed_name().is_none()
            && checked.parsed_value().is_none()
    );
    assert_eq!(checked.value_components(), &mixed);
    assert_eq!(
        svg(&checked).value().unwrap().angle().unwrap().origin(),
        parsed.items().last().unwrap().origin()
    );
}

#[test]
fn globals_metadata_and_pending_reentry_use_actual_body_and_admission() {
    let metadata = CssSvgGlyphOrientationVerticalDeclaration::metadata();
    assert_eq!(metadata.name(), "glyph-orientation-vertical");
    assert_eq!(metadata.property(), NAME);
    assert!(metadata.initial_value().is_auto());
    assert!(metadata.inherited_by_default());
    assert!(!metadata.is_animatable());
    assert_eq!(metadata.settable_members(), &[NAME]);
    assert!(metadata.reset_only_members().is_empty());
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(SVG, &format!("{text}!important"));
        assert_eq!(svg(&source).global(), Some(keyword));
        let contribution = completed(&source);
        assert_eq!(contribution.global(), Some(keyword));
        assert!(contribution.ordinary_value().is_none());
        assert!(contribution.source().same_occurrence(&source));
        assert_eq!(contribution.source().importance(), CssImportance::Important);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("glyph-orientation-vertical: {text} !important;")
        );
    }
    let standard = declaration(SVG, "var(--g)!important");
    let handle = pending(&standard);
    let invalid = parse_component_values("90").unwrap();
    let error = handle.reenter(invalid.clone()).unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(mapped) = error.kind() else {
        panic!("replacement grammar error")
    };
    assert_eq!(
        mapped,
        &SVG.parse_property_value(NAME, invalid, CssImportance::Normal)
            .unwrap_err()
    );
    assert!(handle.source().same_occurrence(&standard));
    let replacement = parse_component_values("/*😀*/180deg").unwrap();
    let CssContributions::SvgGlyphOrientationVertical(value) =
        handle.reenter(replacement.clone()).unwrap()
    else {
        panic!("SVG replay")
    };
    assert_eq!(value.replacement_components(), Some(&replacement));
    assert!(value.source().same_occurrence(&standard));
    assert_eq!(value.source().importance(), CssImportance::Important);
    assert_eq!(
        value.ordinary_value().unwrap().angle().unwrap().origin(),
        replacement.items().last().unwrap().origin()
    );
    for residual in [
        "var(--g)",
        "calc(env(g) * 1deg)",
        r"calc(\61ttr(data-g type(<number>)) * 1deg)",
    ] {
        assert_eq!(
            handle
                .reenter(parse_component_values(residual).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
    }
    for source in [
        declaration(QUIRKS, "env(g)"),
        SVG.parse_svg_glyph_orientation_vertical_attribute_value("attr(data-g)")
            .syntax()
            .as_ref()
            .unwrap()
            .clone(),
    ] {
        let replacement = parse_component_values("90").unwrap();
        let CssContributions::SvgGlyphOrientationVertical(value) =
            pending(&source).reenter(replacement.clone()).unwrap()
        else {
            panic!("retained number admission")
        };
        assert_eq!(
            value.ordinary_value().unwrap().unitless_component(),
            Some(&replacement.items()[0])
        );
        assert!(value.source().same_occurrence(&source));
    }
    let finite = CssPropertyGrammar::from_name("glyph-orientation-vertical").unwrap();
    let finite_source = QUIRKS
        .parse_property_value_for_grammar(
            finite,
            parse_component_values("var(--g)").unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
    let finite_pending = pending(&finite_source);
    assert!(matches!(
        finite_pending
            .reenter(parse_component_values("180deg").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    let CssContributions::Longhands(values) = finite_pending
        .reenter(parse_component_values("90").unwrap())
        .unwrap()
    else {
        panic!("finite replay")
    };
    assert_eq!(
        values.items()[0].property(),
        CssKnownProperty::TextOrientation
    );
    let explicit = CssParserContext::default()
        .parse_property_value(
            NAME,
            parse_component_values("var(--g)").unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
    assert!(matches!(
        pending(&explicit)
            .reenter(parse_component_values("180deg").unwrap())
            .unwrap(),
        CssContributions::SvgGlyphOrientationVertical(_)
    ));
}

#[test]
fn supports_overloads_preserve_literal_boundary_importance_standards_and_sources() {
    for property in ["glyph-orientation-vertical", "GLYPH-ORIENTATION-VERTICAL"] {
        let value = "/*😀*/180deg";
        let test = QUIRKS
            .parse_css_supports_declaration(property, value)
            .unwrap();
        assert_eq!(test.importance(), CssImportance::Normal);
        assert_eq!(
            test.svg_glyph_orientation_vertical().unwrap().parser_mode(),
            CssParserMode::Standards
        );
        assert!(test.known().is_none());
        original(test.property_component().origin(), property);
        assert_eq!(test.components()[1].origin(), &CssValueOrigin::Programmatic);
        for component in test.value_components() {
            original(component.origin(), value);
        }
    }
    for property in [
        " glyph-orientation-vertical",
        "glyph-orientation-vertical ",
        "/*x*/glyph-orientation-vertical",
        "glyph-orientation-vertical/**/",
        r"\67 lyph-orientation-vertical",
        "glyph-orientation-\nvertical",
    ] {
        let error = SVG
            .parse_css_supports_declaration(property, "180deg")
            .unwrap_err();
        assert!(matches!(
            error,
            CssSupportsConstructionError::InvalidDeclarationGrammar { .. }
        ));
        original(error.origin(), property);
    }
    for text in [
        "180deg!important",
        "180deg ! IMPORTANT",
        "180deg!/**/important",
        "var(--g)!important",
    ] {
        let error = SVG
            .parse_css_supports_declaration("glyph-orientation-vertical", text)
            .unwrap_err();
        assert!(matches!(
            error,
            CssSupportsConstructionError::InvalidDeclarationGrammar { .. }
        ));
        assert_eq!(
            original(error.origin(), text).start().byte_offset().value(),
            text.find('!').unwrap()
        );
        for source in [
            format!("(glyph-orientation-vertical:{text})"),
            format!("glyph-orientation-vertical:{text}"),
        ] {
            let condition = SVG.parse_css_supports_condition(&source).unwrap();
            assert_eq!(query(&condition).importance(), CssImportance::Important);
            assert!(query(&condition).svg_glyph_orientation_vertical().is_some());
        }
    }
    let custom = SVG
        .parse_css_supports_declaration("--Case", "\"!important\"")
        .unwrap();
    assert!(custom.known().is_none() && custom.svg_glyph_orientation_vertical().is_none());
    for text in [
        "(glyph-orientation-vertical:90)",
        "glyph-orientation-vertical:90",
    ] {
        let condition = QUIRKS.parse_css_supports_condition(text).unwrap();
        assert!(query(&condition).svg_glyph_orientation_vertical().is_none());
    }
    assert!(
        QUIRKS
            .parse_css_supports_declaration("glyph-orientation-vertical", "90")
            .unwrap()
            .svg_glyph_orientation_vertical()
            .is_none()
    );
    let rule = QUIRKS
        .parse_supports_condition(
            "(glyph-orientation-vertical:90)",
            &CssNamespaceContext::default(),
        )
        .unwrap();
    assert_eq!(
        query(&rule)
            .svg_glyph_orientation_vertical()
            .unwrap()
            .parser_mode(),
        CssParserMode::Quirks
    );
    for text in [
        "/*😀*/ GLYPH-ORIENTATION-VERTICAL: 180deg",
        "glyph-orientation-vertical: 10",
    ] {
        let original_values = parse_component_values(text).unwrap();
        let condition = SVG.parse_css_supports_condition(text).unwrap();
        let test = query(&condition);
        assert_eq!(test.components(), original_values.items());
        assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(
            test.svg_glyph_orientation_vertical().is_some(),
            text.contains("180deg")
        );
        assert_eq!(condition.serialize().unwrap().as_css(), format!("({text})"));
        original(test.property_component().origin(), text);
    }
    let text = "(glyph-orientation-vertical:180deg) and (width:1px)";
    let condition = SVG.parse_css_supports_condition(text).unwrap();
    let CssSupportsConditionKind::And(children) = condition.kind() else {
        panic!("complete And first")
    };
    assert!(
        query(&children.conditions()[0])
            .svg_glyph_orientation_vertical()
            .is_some()
    );
    assert_eq!(
        query(&children.conditions()[1]).known().unwrap().property(),
        CssKnownProperty::Width
    );
    assert_eq!(condition.serialize().unwrap().as_css(), text);
    assert!(
        parse_css_supports_declaration("glyph-orientation-vertical", "180deg")
            .unwrap()
            .known()
            .is_none()
    );
    assert_eq!(
        parse_css_supports_declaration("glyph-orientation-vertical", "90")
            .unwrap()
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::TextOrientation
    );
}

#[test]
fn supports_component_recovery_and_resource_failures_are_not_grammar_retried() {
    for (property, value, supplied) in [
        (
            "glyph-orientation-vertical/*",
            "180deg",
            "glyph-orientation-vertical/*",
        ),
        (
            "glyph-orientation-vertical",
            "180deg!important/*",
            "180deg!important/*",
        ),
    ] {
        let error = SVG
            .parse_css_supports_declaration(property, value)
            .unwrap_err();
        let CssSupportsConstructionError::RecoveredInput {
            origin: CssValueOrigin::ImplicitClosure { opening, at },
        } = error
        else {
            panic!("recovery precedes literal/importance grammar")
        };
        assert_eq!(opening.source().as_str(), supplied);
        assert_eq!(at.span().start().byte_offset().value(), supplied.len());
    }
    for text in [
        "(glyph-orientation-vertical:calc(90deg)",
        "glyph-orientation-vertical:auto/*",
    ] {
        let error = SVG.parse_css_supports_condition(text).unwrap_err();
        let CssSupportsConstructionError::RecoveredInput {
            origin: CssValueOrigin::ImplicitClosure { opening, at },
        } = error
        else {
            panic!("original recovered failure")
        };
        assert_eq!(opening.source().as_str(), text);
        assert_eq!(at.span().start().byte_offset().value(), text.len());
    }
    let error = SVG
        .parse_css_supports_condition("glyph-orientation-vertical:)")
        .unwrap_err();
    let CssSupportsConstructionError::Component(component) = error else {
        panic!("component error")
    };
    original(component.origin(), "glyph-orientation-vertical:)");
    let text = format!(
        "glyph-orientation-vertical:{}90deg{}",
        "calc(".repeat(257),
        ")".repeat(257)
    );
    let error = SVG.parse_css_supports_condition(&text).unwrap_err();
    let CssSupportsConstructionError::Component(component) = error else {
        panic!("nesting limit")
    };
    assert_eq!(component.kind(), CssComponentValueErrorKind::NestingLimit);
    original(component.origin(), &text);
    let text = "(glyph-orientation-vertical:180deg)";
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(1, 3, text.len()).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 4, text.len() - 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let error = SVG
            .parse_supports_condition_with_limits(text, &CssNamespaceContext::default(), limits)
            .unwrap_err();
        let CssSupportsConstructionError::Component(component) = error else {
            panic!("resource error")
        };
        assert_eq!(component.kind(), kind);
        if kind == CssComponentValueErrorKind::ByteLimit {
            // Raw byte preflight precedes retention of a source snapshot.
            assert_eq!(
                component.origin(),
                &CssValueOrigin::UnretainedInput {
                    byte_length: text.len()
                }
            );
        } else {
            original(component.origin(), text);
        }
    }
    let deep = format!(
        "{}glyph-orientation-vertical:180deg{}",
        "(".repeat(40),
        ")".repeat(40)
    );
    let condition = QUIRKS.parse_css_supports_condition(&deep).unwrap();
    assert_eq!(
        query(&condition)
            .svg_glyph_orientation_vertical()
            .unwrap()
            .parser_mode(),
        CssParserMode::Standards
    );
    assert_eq!(condition.serialize().unwrap().as_css(), deep);
    let valid = SVG
        .parse_css_supports_declaration("glyph-orientation-vertical", "180deg")
        .unwrap();
    assert!(
        valid
            .svg_glyph_orientation_vertical()
            .unwrap()
            .value()
            .unwrap()
            .angle()
            .is_some()
    );
}

#[test]
fn nested_rule_fragment_import_keyframe_and_recovery_fronts_retain_definition() {
    let namespaces = CssNamespaceContext::default();
    for css in [
        ".a{glyph-orientation-vertical:180deg}",
        "@media all{.a{glyph-orientation-vertical:180deg}}",
        "@supports (glyph-orientation-vertical:180deg){.a{glyph-orientation-vertical:180deg}}",
        ".a{& .b{glyph-orientation-vertical:180deg}}",
    ] {
        let sheet = SVG.parse_sheet(css);
        assert!(sheet.is_clean(), "{css}: {:?}", sheet.diagnostics());
        let normalized = normalize_sheet(sheet.syntax()).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| {
                if let CssNormalizedItem::Declaration(value) = item {
                    Some(value)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(declarations.len(), 1);
        assert_eq!(
            svg(declarations[0].source())
                .value()
                .unwrap()
                .angle()
                .unwrap()
                .literal()
                .unwrap()
                .numeric()
                .representation(),
            "180"
        );
        let fragment = SVG.parse_rule(css, &namespaces);
        assert!(fragment.is_clean(), "{css}: {:?}", fragment.diagnostics());
        assert_eq!(
            fragment
                .syntax()
                .as_ref()
                .unwrap()
                .to_specified_css()
                .unwrap(),
            sheet.syntax().rules()[0].to_specified_css().unwrap()
        );
    }
    let block = SVG.parse_style_block("{glyph-orientation-vertical:180deg}", &namespaces);
    assert!(block.is_clean());
    assert_eq!(
        block.syntax().as_ref().unwrap().declarations()[0]
            .to_specified_css()
            .unwrap(),
        "glyph-orientation-vertical: 180deg;"
    );
    let list = QUIRKS.parse_style_attribute(
        "bad:1;glyph-orientation-vertical:90!important;color:123;glyph-orientation-vertical:180deg",
    );
    assert_eq!(list.diagnostics().len(), 1);
    assert_eq!(
        list.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_eq!(list.syntax().len(), 3);
    assert!(
        svg(&list.syntax()[0])
            .value()
            .unwrap()
            .unitless_component()
            .is_some()
    );
    assert_eq!(list.syntax()[0].importance(), CssImportance::Important);
    assert_eq!(
        list.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        list.syntax()[1].to_specified_css().unwrap(),
        "color: rgb(0, 1, 35);"
    );
    assert!(svg(&list.syntax()[2]).value().unwrap().angle().is_some());
    let import = QUIRKS.parse_rule(
        "@import \"x\" supports(glyph-orientation-vertical:90);",
        &namespaces,
    );
    assert!(import.is_clean(), "{:?}", import.diagnostics());
    let CssRule::Import(import) = import.syntax().as_ref().unwrap() else {
        panic!("import")
    };
    let test = query(import.supports().unwrap().condition());
    assert_eq!(
        test.svg_glyph_orientation_vertical().unwrap().parser_mode(),
        CssParserMode::Quirks
    );
    let report = QUIRKS.parse_sheet("@keyframes g{from{glyph-orientation-vertical:90;glyph-orientation-vertical:180deg!important;opacity:0}}");
    assert_eq!(report.diagnostics().len(), 1);
    assert!(matches!(
        report.diagnostics()[0].error().kind(),
        ErrorKind::InvalidDeclarationAnnotation(_)
    ));
    let ErrorKind::InvalidDeclarationAnnotation(detail) = report.diagnostics()[0].error().kind()
    else {
        unreachable!()
    };
    assert_eq!(
        detail.context(),
        CssDeclarationContextRef::KeyframeSvgGlyphOrientationVertical
    );
    let CssRule::Keyframes(keyframes) = &report.syntax().rules()[0] else {
        panic!("keyframes")
    };
    let values = keyframes.blocks()[0].declarations();
    assert_eq!(values.len(), 2);
    let body = values[0].svg_glyph_orientation_vertical().unwrap();
    assert_eq!(body.parser_mode(), CssParserMode::Quirks);
    assert_eq!(values[0].property_name(), NAME);
    assert!(values[0].known().is_none() && values[0].custom().is_none());
    assert!(body.value().unwrap().unitless_component().is_some());
    assert_eq!(
        values[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    let malformed = SVG.parse_declaration("glyph-orientation-vertical:180deg!important extra");
    assert!(malformed.syntax().is_none());
    let ErrorKind::InvalidDeclarationAnnotation(detail) = malformed.diagnostics()[0].error().kind()
    else {
        panic!("ordinary SVG annotation")
    };
    assert_eq!(
        detail.context(),
        CssDeclarationContextRef::SvgGlyphOrientationVertical
    );
    let descriptor = SVG.parse_rule(
        "@font-face{glyph-orientation-vertical:180deg;font-family:Example}",
        &namespaces,
    );
    assert!(!descriptor.is_clean());
    let CssRule::FontFace(rule) = descriptor.syntax().as_ref().unwrap() else {
        panic!("font-face")
    };
    assert_eq!(rule.descriptors().occurrences().len(), 1);
    for value in ["auto/*", "initial/*", "var(--g", "calc(90deg"] {
        let css = format!("glyph-orientation-vertical:{value}");
        let report = SVG.parse_style_attribute(&css);
        assert!(!report.is_clean());
        let source = &report.syntax()[0];
        assert_eq!(source.parser_context(), SVG);
        assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
        assert_eq!(
            report
                .clone()
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
        assert!(source.svg_glyph_orientation_vertical().is_some());
    }
    let deep = format!(
        "{} .a{{glyph-orientation-vertical:180deg}} {}",
        "@media all{".repeat(70),
        "}".repeat(70)
    );
    let report = SVG.parse_sheet(&deep);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declaration = normalized
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
    assert!(svg(declaration.source()).value().unwrap().angle().is_some());
}

#[test]
fn operational_metadata_is_distinct_from_finite_partial_and_canonical_inventory() {
    let source = declaration(SVG, "180deg");
    assert_eq!(completed(&source).property(), NAME);
    let metadata = CssSvgGlyphOrientationVerticalDeclaration::metadata();
    let feature = feature_metadata(metadata.feature_id().as_str()).unwrap();
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.kind(), CssFeatureKind::Property);
    assert_eq!(feature.spelling(), "glyph-orientation-vertical");
    assert_eq!(feature.source().id().as_str(), "I-WEBKIT-SVG-GLYPH");
    assert_eq!(
        feature.source().tier(),
        CssSpecificationTier::Snapshot2026Interop
    );
    assert_eq!(
        feature.source().level(),
        "WebKit73aa6c89e2cb77c46184a81aec944e4ab99d114d"
    );
    assert_eq!(
        feature.source().url(),
        Some(
            "https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/css/CSSProperties.json#L5350-L5367"
        )
    );
    assert_eq!(feature.source().repository_provenance(), None);
    assert_eq!(feature.supported_subset(), None);
    assert_eq!(feature.unsupported_remainder(), None);
    assert_eq!(
        feature_metadata("official.property-alias.glyph-orientation-vertical")
            .unwrap()
            .status(),
        CssSupportStatus::Partial
    );
    assert_eq!(
        CssPropertyGrammar::from_name("glyph-orientation-vertical")
            .unwrap()
            .target_property(),
        CssKnownProperty::TextOrientation
    );
    assert!(CssKnownProperty::TextOrientation.aliases().is_empty());
    assert!(property_support_metadata("glyph-orientation-vertical").is_none());
}

#[test]
fn normalization_retains_order_independent_targets_symbolic_all_and_late_occurrence_failure() {
    let css = ".a{glyph-orientation-vertical:180deg!important;text-orientation:upright;all:unset;glyph-orientation-vertical:var(--g)}";
    let report = SVG.parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let normalized = normalize_report(&report).unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let declarations: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| {
            if let CssNormalizedItem::Declaration(value) = item {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        declarations
            .iter()
            .map(|value| value.order())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    assert_eq!(
        declarations[0].source().importance(),
        CssImportance::Important
    );
    assert!(matches!(
        declarations[0].expansion(),
        CssExpansion::Contributions(CssContributions::SvgGlyphOrientationVertical(_))
    ));
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        declarations[1].expansion()
    else {
        panic!("modern contribution")
    };
    assert_eq!(
        values.items()[0].property(),
        CssKnownProperty::TextOrientation
    );
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        declarations[2].expansion()
    else {
        panic!("symbolic all")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::Unset);
    assert!(!reset.excludes(NAME));
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::TextOrientation)));
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Direction)));
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::UnicodeBidi)));
    assert!(reset.excludes(CssPropertyNameRef::Custom(
        &CssCustomPropertyName::try_new("--x").unwrap()
    )));
    assert!(matches!(
        declarations[3].expansion(),
        CssExpansion::Pending(_)
    ));
    let exact = CssNormalizationLimits::try_new(1, 1, 4, 4).unwrap();
    assert_eq!(
        normalize_sheet_with_limits(report.syntax(), exact)
            .unwrap()
            .items()
            .len(),
        normalized.syntax().items().len()
    );
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(1, 1, 4, 3).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 3
        }
    );
    assert_eq!(error.declaration_order(), Some(3));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(declarations[3].source())
    );
    assert_eq!(report, before);
    let finite = CssParserContext::default()
        .parse_sheet(".a{glyph-orientation-vertical:90!important;text-orientation:upright}");
    let finite = normalize_sheet(finite.syntax()).unwrap();
    let values: Vec<_> = finite
        .items()
        .iter()
        .filter_map(|item| {
            if let CssNormalizedItem::Declaration(value) = item {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    for value in &values {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) = value.expansion()
        else {
            panic!("finite shared target")
        };
        assert_eq!(items.items().len(), 1);
        assert_eq!(
            items.items()[0].property(),
            CssKnownProperty::TextOrientation
        );
    }
    assert_eq!(values[0].source().importance(), CssImportance::Important);
    assert_eq!(values[1].source().importance(), CssImportance::Normal);
}

#[test]
fn specified_goldens_use_shared_leaf_math_tariffs_and_atomic_three_budget_retry() {
    // Literal text/UTF-8 lengths and graph costs are independent design oracles.
    for (context, text, expected, input, projection, bytes) in [
        (SVG, "AUTO", "glyph-orientation-vertical: auto;", 3, 3, 33),
        (
            SVG,
            "+090.0deg",
            "glyph-orientation-vertical: 90deg;",
            3,
            3,
            34,
        ),
        (
            QUIRKS,
            "90.0",
            "glyph-orientation-vertical: 90deg;",
            3,
            3,
            34,
        ),
        (
            QUIRKS,
            "9e1",
            "glyph-orientation-vertical: 90deg;",
            3,
            3,
            34,
        ),
        (
            QUIRKS,
            "-135",
            "glyph-orientation-vertical: -135deg;",
            3,
            3,
            36,
        ),
        (
            SVG,
            ".25turn",
            "glyph-orientation-vertical: 0.25turn;",
            3,
            3,
            37,
        ),
        (SVG, "1rad", "glyph-orientation-vertical: 1rad;", 3, 3, 33),
        (
            SVG,
            "calc(90deg)",
            "glyph-orientation-vertical: calc(90deg);",
            4,
            3,
            40,
        ),
        (
            SVG,
            "calc(15deg + 15deg)",
            "glyph-orientation-vertical: calc(30deg);",
            6,
            5,
            40,
        ),
        (
            SVG,
            "AUTO!important",
            "glyph-orientation-vertical: auto !important;",
            3,
            3,
            44,
        ),
    ] {
        assert_eq!(expected.len(), bytes);
        let source = declaration(context, text);
        let before = source.clone();
        let exact = CssSpecifiedValueSerializationLimits::new(input, projection, bytes);
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        let standalone = expected
            .strip_prefix("glyph-orientation-vertical: ")
            .unwrap()
            .strip_suffix(';')
            .unwrap();
        let standalone = standalone.strip_suffix(" !important").unwrap_or(standalone);
        assert_eq!(
            svg(&source)
                .value()
                .unwrap()
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input - 2,
                    projection - 2,
                    standalone.len()
                ))
                .unwrap(),
            standalone
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(input - 1, projection, bytes),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(input, projection - 1, bytes),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(input, projection, bytes - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(source, before);
            assert!(source.same_occurrence(&before));
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        // Complete specified declarations include their list semicolon.
        let reparsed = SVG.parse_style_attribute(expected);
        assert!(
            reparsed.is_clean(),
            "{expected}: {:?}",
            reparsed.diagnostics()
        );
        assert_eq!(reparsed.syntax().len(), 1);
        let occurrence = &reparsed.syntax()[0];
        assert_eq!(occurrence.property_name(), NAME);
        assert!(occurrence.svg_glyph_orientation_vertical().is_some());
        assert_eq!(occurrence.importance(), source.importance());
        assert_eq!(occurrence.to_specified_css().unwrap(), expected);
    }
    let unitless = declaration(QUIRKS, "45.0000005");
    assert_eq!(
        unitless.to_specified_css().unwrap(),
        "glyph-orientation-vertical: 45.000001deg;"
    );
    let reparse = declaration(SVG, "45.000001deg");
    assert!(
        svg(&reparse)
            .value()
            .unwrap()
            .unitless_component()
            .is_none()
    );
    assert!(svg(&reparse).value().unwrap().angle().is_some());
    assert_eq!(
        unitless.value_components().serialize().unwrap().as_css(),
        "45.0000005"
    );
    let sheet = SVG
        .parse_sheet(".a{glyph-orientation-vertical:180deg}.b{glyph-orientation-vertical:90deg}");
    let before = sheet.clone();
    let expected =
        ".a { glyph-orientation-vertical: 180deg; }\n.b { glyph-orientation-vertical: 90deg; }";
    // Sheet aggregate 1; each rule 1 + selector list 1 + class leaf 1
    // + declaration-list aggregate 1 + declaration/name/value 3 = 7.
    let exact = CssSpecifiedValueSerializationLimits::new(15, 15, expected.len());
    assert_eq!(
        sheet.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    let error = sheet
        .syntax()
        .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
            usize::MAX,
            usize::MAX,
            expected.len() - 1,
        ))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedRuleSerializationErrorKind::Resource(
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        )
    );
    assert_eq!(error.rule_index(), Some(1));
    assert_eq!(sheet, before);
    assert_eq!(
        sheet.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(14, 15, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(15, 14, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        let error = sheet
            .syntax()
            .to_specified_css_with_limits(limits)
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(kind)
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, before);
    }
    for text in ["auto", "initial", "90"] {
        let finite = declaration(CssParserContext::default(), text);
        let projection = if text == "90" { 4 } else { 3 };
        let expected = if text == "90" {
            "glyph-orientation-vertical: 90deg;"
        } else if text == "auto" {
            "glyph-orientation-vertical: auto;"
        } else {
            "glyph-orientation-vertical: initial;"
        };
        assert_eq!(
            finite
                .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                    3,
                    projection,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
    }
}
