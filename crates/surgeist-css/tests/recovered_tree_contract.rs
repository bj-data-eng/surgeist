#![forbid(unsafe_code)]
//! Public recovery and round-trip contracts from CSS Syntax 3 §§3,9–10.
use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssCustomPropertyName,
    CssDeclarationList, CssErrorCode, CssFontFaceDescriptorKind, CssImportance, CssKnownProperty,
    CssMediaQuery, CssMediaType, CssParseReport, CssPropertyNameRef, CssRecoveryAction, CssRule,
    CssSelector, CssSheet, CssSourceSpan, CssStyleRule, CssValueTokenRef, ErrorKind,
    parse_declaration, parse_property_value, parse_sheet, parse_style_attribute, validate_sheet,
};

fn span(span: CssSourceSpan, expected: std::ops::Range<usize>) {
    assert_eq!(span.start().byte_offset().value(), expected.start);
    assert_eq!(span.end().byte_offset().value(), expected.end);
}
fn style<'a>(rule: &'a CssRule, class: &str) -> &'a CssStyleRule {
    let CssRule::Style(rule) = rule else {
        panic!("style rule for {class}")
    };
    let [selector] = rule.selectors().selectors() else {
        panic!("one complete class selector")
    };
    assert_eq!(selector.selector(), &CssSelector::Class(class.into()));
    rule
}
fn properties(declarations: &CssDeclarationList) -> Vec<&str> {
    declarations
        .iter()
        .map(|d| match d.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            _ => panic!("selected property name"),
        })
        .collect()
}
fn strict_failure(source: &str, report: &CssParseReport<CssSheet>) {
    assert!(!report.is_clean());
    assert!(!report.diagnostics().is_empty());
    let failure = validate_sheet(source).unwrap_err();
    assert_eq!(failure.diagnostics(), report.diagnostics());
    assert_eq!(failure.first(), &report.diagnostics()[0]);
    assert_eq!(failure.into_diagnostics(), report.diagnostics());
    let failure = report.clone().into_validation_result().unwrap_err();
    assert_eq!(failure.diagnostics(), report.diagnostics());
}

#[test]
fn required_media_block_at_eof_reports_actual_missing_input_without_an_authored_semicolon() {
    // End of input has no authored token to expose as an encountered delimiter.
    for (source, unit_start, eof, line, column, siblings) in [
        ("@media screen", 0, 13, 0, 13, 0),
        ("@media screen\r\n", 0, 15, 1, 0, 0),
        (".before{}@media screen", 9, 22, 0, 22, 1),
        ("/*😀*/\r\n.before{}@media screen", 19, 32, 1, 22, 1),
    ] {
        let report = parse_sheet(source);
        assert_eq!(report.syntax().rules().len(), siblings);
        if siblings == 1 {
            assert!(
                style(&report.syntax().rules()[0], "before")
                    .declarations()
                    .is_empty()
            );
        }
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected required-block unit")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert!(
            matches!(diagnostic.error().kind(), ErrorKind::UnexpectedEnd(_)),
            "EOF has no authored encountered token: {diagnostic:?}"
        );
        let at = diagnostic.error().position();
        assert_eq!(at.byte_offset().value(), eof);
        assert_eq!((at.line().value(), at.column().value()), (line, column));
        span(diagnostic.span(), unit_start..eof);
        strict_failure(source, &report);
    }
}

#[test]
fn authored_semicolon_ends_media_statement_and_reports_missing_body_at_the_bounded_end() {
    let source = "@media screen;.after{}";
    let report = parse_sheet(source);
    let [after] = report.syntax().rules() else {
        panic!("later valid sibling survives missing-body rejection")
    };
    assert!(style(after, "after").declarations().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected required-block statement")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
    let ErrorKind::InvalidAtRuleBody(detail) = diagnostic.error().kind() else {
        panic!("defined media body is missing after the statement terminator")
    };
    assert_eq!(detail.name().as_str(), "media");
    assert_eq!(detail.production().as_str(), "baseline.rule.media");
    assert!(detail.encountered().is_none());
    let at = diagnostic.error().position();
    assert_eq!(at.byte_offset().value(), 14);
    assert_eq!(at.line().value(), 0);
    assert_eq!(at.column().value(), 14);
    span(diagnostic.span(), 0..14);
    strict_failure(source, &report);
}

#[test]
fn defined_layer_statement_and_complete_media_block_are_clean_distinct_productions() {
    let source = "@layer theme;@media screen{}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::LayerStatement(layer), CssRule::Media(media)] = report.syntax().rules() else {
        panic!("statement and block have their defined result types")
    };
    assert_eq!(layer.names().names()[0].components(), &["theme".to_owned()]);
    assert!(media.rules().is_empty());
    let [CssMediaQuery::Typed(query)] = media.query().queries() else {
        panic!("typed screen grammar")
    };
    assert_eq!(query.media_type(), CssMediaType::Screen);
    assert_eq!(validate_sheet(source), Ok(report.syntax().clone()));
    assert_eq!(
        report.clone().into_validation_result(),
        Ok(report.syntax().clone())
    );
}

#[test]
fn an_implicitly_closed_valid_style_tree_is_retained_while_clean_validation_reports_the_eof_fault()
{
    let source = ".x{color:red";
    let report = parse_sheet(source);
    let [rule] = report.syntax().rules() else {
        panic!("retained implicitly closed style")
    };
    let rule = style(rule, "x");
    assert_eq!(properties(rule.declarations()), ["color"]);
    assert_eq!(
        rule.declarations()[0]
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        "red"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one retained style block EOF")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 12);
    span(diagnostic.span(), 12..12);
    strict_failure(source, &report);
    let complete = parse_sheet(".x{color:red}");
    assert!(complete.is_clean());
    assert_eq!(
        complete.syntax().to_specified_css().unwrap(),
        ".x { color: red; }"
    );
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".x { color: red; }"
    );
}

#[test]
fn invalid_selector_list_discards_the_entire_style_and_unknown_at_rule_without_losing_valid_siblings()
 {
    let source = ".before{width:1px}.bad,>{color:red}@unknown x;.after{height:2px}";
    let report = parse_sheet(source);
    let [before, after] = report.syntax().rules() else {
        panic!("only before and after styles survive")
    };
    assert_eq!(
        properties(style(before, "before").declarations()),
        ["width"]
    );
    assert_eq!(properties(style(after, "after").declarations()), ["height"]);
    let [selector, unknown] = report.diagnostics() else {
        panic!("both invalid rule units reported")
    };
    assert_eq!(selector.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(selector.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(unknown.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(unknown.error().code(), CssErrorCode::UnknownAtRule);
    let ErrorKind::UnknownAtRule(detail) = unknown.error().kind() else {
        panic!("unknown at-rule identity")
    };
    assert_eq!(detail.name().as_str(), "unknown");
    for (diagnostic, unit) in [(selector, ".bad,>{color:red}"), (unknown, "@unknown x;")] {
        let start = source.find(unit).unwrap();
        span(diagnostic.span(), start..start + unit.len());
    }
    assert!(selector.error().position() < unknown.error().position());
    strict_failure(source, &report);
}

#[test]
fn style_property_names_are_ascii_insensitive_custom_names_keep_case_and_invalid_declarations_are_local()
 {
    let source = ".x{CoLoR:red;--Tone:A;--tone:B;mystery:1;width:nope;HEIGHT:2px}";
    let report = parse_sheet(source);
    let [rule] = report.syntax().rules() else {
        panic!("parent remains valid")
    };
    let rule = style(rule, "x");
    assert!(rule.rules().is_empty());
    assert_eq!(
        properties(rule.declarations()),
        ["color", "--Tone", "--tone", "height"]
    );
    assert_eq!(
        rule.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        rule.declarations()[3].known().unwrap().property(),
        CssKnownProperty::Height
    );
    for (declaration, value) in rule.declarations().as_slice()[1..3].iter().zip(["A", "B"]) {
        let [component] = declaration.value_components().items() else {
            panic!("one retained custom ident")
        };
        assert!(
            matches!(component.view(),CssComponentValueRef::Token(CssValueTokenRef::Ident(actual)) if actual==value)
        );
    }
    let [unknown, invalid] = report.diagnostics() else {
        panic!("only malformed declarations are dropped")
    };
    assert_eq!(unknown.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(invalid.error().code(), CssErrorCode::InvalidPropertyValue);
    assert_eq!(unknown.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(invalid.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        unknown.error().position().byte_offset().value(),
        source.find("mystery").unwrap()
    );
    assert_eq!(
        invalid.error().position().byte_offset().value(),
        source.find("nope").unwrap()
    );
    strict_failure(source, &report);
}

#[test]
fn identical_declaration_token_shapes_use_property_or_descriptor_grammar_from_the_owning_rule() {
    let source = ".p{font-display:swap;font-weight:400}@font-face{font-display:swap;font-weight:400;color:red}";
    let report = parse_sheet(source);
    let [property, CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("style and font descriptor owner survive")
    };
    assert_eq!(
        properties(style(property, "p").declarations()),
        ["font-weight"]
    );
    assert_eq!(
        face.descriptors()
            .occurrences()
            .map(|d| d.value().kind())
            .collect::<Vec<_>>(),
        [
            CssFontFaceDescriptorKind::FontDisplay,
            CssFontFaceDescriptorKind::FontWeight
        ]
    );
    let [property_error, descriptor_error] = report.diagnostics() else {
        panic!("wrong-context names rejected independently")
    };
    assert_eq!(property_error.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(property_error.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        descriptor_error.error().code(),
        CssErrorCode::UnknownDescriptor
    );
    assert_eq!(descriptor_error.action(), CssRecoveryAction::DropDescriptor);
    let ErrorKind::UnknownDescriptor(detail) = descriptor_error.error().kind() else {
        panic!("descriptor context carried explicitly")
    };
    assert_eq!(detail.at_rule().as_str(), "font-face");
    assert_eq!(detail.descriptor().as_str(), "color");
    strict_failure(source, &report);
}

#[test]
fn retained_comments_are_grammar_trivia_but_do_not_merge_two_identifier_tokens() {
    for source in [
        ".x{color:red;height:2px}",
        ".x/**/{/**/color/**/:/**/red/**/;/**/height/**/:/**/2px/**/}",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{report:?}");
        let [rule] = report.syntax().rules() else {
            panic!("one style")
        };
        let declarations = style(rule, "x").declarations();
        assert_eq!(properties(declarations), ["color", "height"]);
        assert_eq!(
            declarations
                .iter()
                .map(|d| d.to_specified_css().unwrap())
                .collect::<Vec<_>>(),
            ["color: red;", "height: 2px;"]
        );
    }
    let source = ".x{co/**/lor:red;height:2px}";
    let report = parse_sheet(source);
    let [rule] = report.syntax().rules() else {
        panic!("invalid declaration cannot drop parent")
    };
    assert_eq!(properties(style(rule, "x").declarations()), ["height"]);
    let [diagnostic] = report.diagnostics() else {
        panic!("comment does not join separate co and lor idents")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    strict_failure(source, &report);
}

#[test]
fn specified_sheet_serialization_reparses_to_the_same_selector_and_authored_declaration_structure()
{
    let source = ".x{--Case:A/**/B;--case:f([x,y]{z;!});height:2px}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let expected = ".x { --Case: A/**/B; --case: f([x,y]{z;!}); height: 2px; }";
    let emitted = report.syntax().to_specified_css().unwrap();
    assert_eq!(emitted, expected);
    assert_eq!(
        report, before,
        "serialization does not mutate retained source or diagnostics"
    );
    let reparsed = parse_sheet(&emitted);
    assert!(reparsed.is_clean(), "{reparsed:?}");
    let [original] = report.syntax().rules() else {
        panic!("original style")
    };
    let [reparsed_rule] = reparsed.syntax().rules() else {
        panic!("same reparsed style")
    };
    let original = style(original, "x");
    let reparsed_rule = style(reparsed_rule, "x");
    assert_eq!(
        properties(original.declarations()),
        ["--Case", "--case", "height"]
    );
    assert_eq!(
        properties(reparsed_rule.declarations()),
        ["--Case", "--case", "height"]
    );
    assert!(original.rules().is_empty());
    assert!(reparsed_rule.rules().is_empty());
    for (before, after) in original
        .declarations()
        .iter()
        .zip(reparsed_rule.declarations().iter())
    {
        assert_eq!(
            before.body(),
            after.body(),
            "compare syntax meaning; provenance names different input occurrences"
        );
        assert_eq!(before.importance(), after.importance());
    }
    assert_eq!(reparsed.syntax().to_specified_css().unwrap(), expected);
}

#[test]
fn constructed_adjacent_identifiers_reparse_as_two_idents_without_requiring_the_informative_separator_algorithm()
 {
    let name = CssCustomPropertyName::try_new("--Tokens").unwrap();
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("A").unwrap(),
        CssComponentValue::try_ident("B").unwrap(),
    ])
    .unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Custom(&name),
        components,
        CssImportance::Normal,
    )
    .unwrap();
    let original = declaration.clone();
    let emitted = declaration.to_specified_css().unwrap();
    assert_eq!(declaration, original);
    // The singular front rejects the serializer's root list separator.
    let singular = parse_declaration(&emitted);
    assert!(singular.syntax().is_none());
    let [diagnostic] = singular.diagnostics() else {
        panic!("the terminated declaration is not a singular source fragment")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);

    // Consume the unchanged serialized declaration with its documented list front.
    let reparsed = parse_style_attribute(&emitted);
    assert!(reparsed.is_clean(), "{reparsed:?}");
    let [reparsed] = reparsed.syntax().as_slice() else {
        panic!("exactly one retained serialized declaration")
    };
    assert_eq!(reparsed.property_name(), CssPropertyNameRef::Custom(&name));
    assert_eq!(reparsed.importance(), CssImportance::Normal);
    // Retained source components include the canonical colon-edge whitespace.
    // Exclude only root edge whitespace; any whitespace between A and B is invalid.
    let components = reparsed.value_components().items();
    let start = components
        .iter()
        .position(|component| {
            !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .unwrap_or(components.len());
    let end = components
        .iter()
        .rposition(|component| {
            !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .map_or(start, |index| index + 1);
    let mut values = Vec::new();
    for component in &components[start..end] {
        match component.view() {
            CssComponentValueRef::Comment(_) => {}
            CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) => values.push(value),
            other => panic!(
                "no merged token, introduced whitespace or changed component kind: {other:?}; emitted {emitted:?}"
            ),
        }
    }
    assert_eq!(values, ["A", "B"]);
    assert_eq!(reparsed.to_specified_css().unwrap(), emitted);
    assert_eq!(declaration, original);
}
