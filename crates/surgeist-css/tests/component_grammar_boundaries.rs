#![forbid(unsafe_code)]
//! Caller grammar boundaries through existing public parsing interfaces.
//! Independent authority: Syntax 3 CRD20211224 §§5.3.1–2, 5.4.7–9, 8, 8.2;
//! selected MQ5 WD20260219 §3, Variables CR20220616 §2,
//! Sizing3 WD20260904 §3.1.1 and Transforms1 CR20190214 §9.1.
//! This is not a public raw-error stream or generic grammar DSL test.
use surgeist_css::{
    CssBlockKind, CssComponentValueErrorKind, CssComponentValueRef, CssCustomPropertyName,
    CssErrorCode, CssGeneralEnclosed, CssGeneralEnclosedError, CssImportance, CssKnownProperty,
    CssKnownPropertyValueRef, CssMediaConditionKind, CssMediaQuery, CssMediaType,
    CssNamespaceContext, CssParsedOrigin, CssPropertyNameRef, CssRecoveryAction, CssRule,
    CssSupportsConstructionError, CssValueOrigin, CssValueTokenRef, ErrorKind,
    parse_component_values, parse_css_supports_declaration, parse_media_query,
    parse_media_query_list, parse_property_value, parse_property_value_text,
    parse_property_value_text_for_grammar, parse_rule,
};

fn width(source: &str) -> surgeist_css::CssParseReport<Option<surgeist_css::CssDeclaration>> {
    parse_property_value_text(
        source,
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        CssImportance::Normal,
    )
}

fn opaque(query: &CssMediaQuery) -> &CssGeneralEnclosed {
    let CssMediaQuery::Condition(condition) = query else {
        panic!("media condition")
    };
    let CssMediaConditionKind::GeneralEnclosed(value) = condition.kind() else {
        panic!("opaque any-value enclosure")
    };
    value
}

fn parsed(origin: &CssValueOrigin) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("actual parsed source origin")
    };
    origin
}

fn root_punctuation(component: &surgeist_css::CssComponentValue) {
    let contents = match component.view() {
        CssComponentValueRef::Function(function) => function.values(),
        CssComponentValueRef::Block(block) => block.values(),
        _ => panic!("one complete enclosure"),
    };
    let [name, semicolon, bang] = contents.items() else {
        panic!("literal a, semicolon, bang at this contents-list root")
    };
    assert!(matches!(
        name.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("a"))
    ));
    assert!(matches!(
        semicolon.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Semicolon)
    ));
    assert!(matches!(
        bang.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Delim('!'))
    ));
}

#[test]
fn whole_width_production_matches_complete_input_and_rejects_a_valid_prefix_with_residual_components()
 {
    let property = CssKnownProperty::Width;
    let source = " \t12px /**/ ";
    for report in [
        width(source),
        parse_property_value_text_for_grammar(source, property.grammar(), CssImportance::Normal),
    ] {
        assert!(report.is_clean(), "{report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        let CssKnownPropertyValueRef::Width(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("defined typed width match")
        };
        assert_eq!(value.value().serialize_specified().unwrap(), "12px");
        let origin = declaration.parsed_value().unwrap();
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), source.len());
        assert!(declaration.parsed_name().is_none());
    }
    for source in [
        "12px 13px",
        "12px,13px",
        "auto extra",
        "calc(1px + 2px) extra",
    ] {
        for report in [
            width(source),
            parse_property_value_text_for_grammar(
                source,
                property.grammar(),
                CssImportance::Normal,
            ),
        ] {
            assert!(report.syntax().is_none(), "{source}: {report:?}");
            let [diagnostic] = report.diagnostics() else {
                panic!("one complete-input failure")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
            assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
            assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        }
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property),
                parse_component_values(source).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "supplied checked value still needs whole width grammar: {source}"
        );
    }
}

#[test]
fn media_whitespace_only_has_no_results_but_empty_comma_segments_have_individual_failures() {
    for source in ["", " \t\r\n", " /**/ "] {
        let report = parse_media_query_list(source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert!(report.syntax().queries().is_empty());
    }
    let source = ",screen,,print,";
    let report = parse_media_query_list(source);
    let [
        CssMediaQuery::Never(first),
        CssMediaQuery::Typed(screen),
        CssMediaQuery::Never(middle),
        CssMediaQuery::Typed(print),
        CssMediaQuery::Never(last),
    ] = report.syntax().queries()
    else {
        panic!("leading, consecutive and trailing empty members retain failures in place")
    };
    assert_eq!(screen.media_type(), CssMediaType::Screen);
    assert_eq!(print.media_type(), CssMediaType::Print);
    for (never, at) in [(first, 0), (middle, 8), (last, 15)] {
        let origin = parsed(never.origin());
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), at);
        assert_eq!(origin.span().end().byte_offset().value(), at);
    }
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [CssRecoveryAction::ReplaceMediaQueryWithNever; 3]
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.error().code() == CssErrorCode::InvalidMediaQuery)
    );
    assert!(report.clone().into_validation_result().is_err());
    let comma = parse_media_query_list(",");
    assert!(matches!(
        comma.syntax().queries(),
        [CssMediaQuery::Never(_), CssMediaQuery::Never(_)]
    ));
    assert_eq!(comma.diagnostics().len(), 2);
}

#[test]
fn media_nested_commas_stay_in_one_member_and_a_failed_middle_member_preserves_the_last() {
    let enclosure = "future([a,b] {c,d} nested(e,f))";
    let source = format!("{enclosure},screen and,print");
    let report = parse_media_query_list(&source);
    let [
        first,
        CssMediaQuery::Never(failed),
        CssMediaQuery::Typed(print),
    ] = report.syntax().queries()
    else {
        panic!("valid enclosure, individual failure, valid print")
    };
    let value = opaque(first);
    assert_eq!(value.authored(), Some(enclosure));
    assert_eq!(value.serialize().unwrap().as_css(), enclosure);
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("original enclosure occurrence")
    };
    assert_eq!(origin.source().as_str(), source);
    let failed_origin = parsed(failed.origin());
    assert!(origin.source().same_snapshot(failed_origin.source()));
    assert_eq!(
        failed_origin.span().start().byte_offset().value(),
        enclosure.len() + 1
    );
    assert_eq!(
        failed_origin.span().end().byte_offset().value(),
        enclosure.len() + 11
    );
    assert_eq!(print.media_type(), CssMediaType::Print);
    let [diagnostic] = report.diagnostics() else {
        panic!("only failed middle member diagnosed")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
}

#[test]
fn escaped_ident_values_do_not_acquire_function_or_at_keyword_kind() {
    for (source, ident) in [(r"future\(", "future("), (r"\@future", "@future")] {
        let values = parse_component_values(source).unwrap();
        let [component] = values.items() else {
            panic!("one escaped identifier token")
        };
        assert!(
            matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) if value == ident)
        );
        let error = CssGeneralEnclosed::try_from_component(component.clone()).unwrap_err();
        assert!(matches!(
            error,
            CssGeneralEnclosedError::WrongOuterComponent { .. }
        ));
        assert_eq!(error.origin(), component.origin());
    }
    let at = parse_component_values("@future").unwrap();
    assert!(matches!(
        at.items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::AtKeyword("future"))
    ));
    assert!(CssGeneralEnclosed::try_from_component(at.items()[0].clone()).is_err());
    let function = parse_component_values("future()").unwrap();
    assert!(
        matches!(function.items()[0].view(), CssComponentValueRef::Function(value) if value.name() == "future")
    );
    assert!(CssGeneralEnclosed::try_from_component(function.items()[0].clone()).is_ok());

    let escaped = parse_rule(r"\@media all {}", &CssNamespaceContext::default());
    assert!(escaped.is_clean(), "{escaped:?}");
    assert!(matches!(escaped.syntax(), Some(CssRule::Style(_))));
    let actual = parse_rule("@media all {}", &CssNamespaceContext::default());
    assert!(actual.is_clean(), "{actual:?}");
    assert!(matches!(actual.syntax(), Some(CssRule::Media(_))));
}

#[test]
fn actual_literal_grammars_match_decoded_ascii_case_variants_but_require_their_token_kinds() {
    for source in ["AUTO", " AuTo ", r"\61 UTO"] {
        let report = width(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let CssKnownPropertyValueRef::Width(value) = report
            .syntax()
            .as_ref()
            .unwrap()
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("width auto keyword")
        };
        assert_eq!(value.value().serialize_specified().unwrap(), "auto");
    }
    for source in ["auto()", "@auto"] {
        assert!(width(source).syntax().is_none(), "{source}");
    }
    for source in [
        "translateX(12px)",
        "TrAnSlAtEx(12px)",
        r"\74 ranslateX(12px)",
    ] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Known(CssKnownProperty::Transform),
            CssImportance::Normal,
        );
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(matches!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .known()
                .unwrap()
                .property_value()
                .unwrap(),
            CssKnownPropertyValueRef::Transform(_)
        ));
    }
    let ident = r"translateX\(";
    assert!(matches!(
        parse_component_values(ident).unwrap().items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("translateX("))
    ));
    assert!(
        parse_property_value_text(
            ident,
            CssPropertyNameRef::Known(CssKnownProperty::Transform),
            CssImportance::Normal
        )
        .syntax()
        .is_none()
    );
    let media = parse_media_query("ScReEn");
    assert!(media.is_clean());
    assert!(
        matches!(media.syntax(), CssMediaQuery::Typed(query) if query.media_type() == CssMediaType::Screen)
    );
}

#[test]
fn root_semicolon_and_bang_are_allowed_by_any_value_but_excluded_from_declaration_value_contexts() {
    let custom = CssCustomPropertyName::try_new("--tokens").unwrap();
    for source in ["a;", "a!", "a;!"] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Custom(&custom),
            CssImportance::Normal,
        );
        assert!(
            report.syntax().is_none(),
            "root declaration-value exclusion: {source}"
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::RejectInput
        );
        assert!(
            matches!(
                parse_css_supports_declaration("--tokens", source),
                Err(CssSupportsConstructionError::InvalidDeclarationGrammar { .. })
            ),
            "two-argument method grammar: {source}"
        );
        let query = parse_media_query(&format!("future({source})"));
        assert!(
            query.is_clean(),
            "any-value admits contents-root punctuation: {source}: {query:?}"
        );
        assert_eq!(
            opaque(query.syntax()).authored(),
            Some(format!("future({source})").as_str())
        );
    }
    let any = parse_media_query("future(a;!)");
    root_punctuation(opaque(any.syntax()).component());
    for source in ["f(a;!)", "[a;!]", "{a;!}", "(a;!)"] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Custom(&custom),
            CssImportance::Normal,
        );
        assert!(
            report.is_clean(),
            "nested punctuation is below declaration-value root: {source}: {report:?}"
        );
        let declaration = report.syntax().as_ref().unwrap();
        assert_eq!(declaration.custom().unwrap().name().as_str(), "--tokens");
        let [component] = declaration.value_components().items() else {
            panic!("one complete grouped component")
        };
        root_punctuation(component);
        let supports = parse_css_supports_declaration("--tokens", source).unwrap();
        assert_eq!(supports.importance(), CssImportance::Normal);
        assert!(supports.known().is_none());
    }
}

#[test]
fn nullable_defining_grammars_do_not_make_nonnullable_known_property_values_match() {
    let custom = CssCustomPropertyName::try_new("--empty").unwrap();
    for source in ["", " \t", "/**/"] {
        assert!(
            width(source).syntax().is_none(),
            "width needs its own value: {source:?}"
        );
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Custom(&custom),
            CssImportance::Normal,
        );
        assert!(
            report.is_clean(),
            "Variables selects declaration-value?: {source:?}: {report:?}"
        );
        assert!(report.syntax().as_ref().unwrap().custom().is_some());
    }
    for source in ["future()", "()", "future( \t)"] {
        let report = parse_media_query(source);
        assert!(
            report.is_clean(),
            "MQ selects any-value?: {source:?}: {report:?}"
        );
        assert_eq!(opaque(report.syntax()).authored(), Some(source));
    }
}

#[test]
fn recursive_lexical_faults_reject_checked_values_and_only_the_owning_media_member() {
    for (payload, responsible, kind) in [
        ("f(url(a b))", "url(", CssComponentValueErrorKind::BadUrl),
        ("f(\"bad\n)", "\"", CssComponentValueErrorKind::BadString),
        (
            "f(])",
            "]",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
        ),
        (
            "f([)])",
            ")",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
        ),
    ] {
        let error = parse_component_values(payload).unwrap_err();
        assert_eq!(error.kind(), kind, "{payload}");
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("original checked lexical failure")
        };
        assert_eq!(origin.source().as_str(), payload);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            payload.find(responsible).unwrap()
        );
        let custom = CssCustomPropertyName::try_new("--bad").unwrap();
        let property = parse_property_value_text(
            payload,
            CssPropertyNameRef::Custom(&custom),
            CssImportance::Normal,
        );
        assert!(
            property.syntax().is_none(),
            "declaration-value cannot retain a bad nested token: {payload}"
        );
        let source = format!("future({payload}),print");
        let report = parse_media_query_list(&source);
        let [CssMediaQuery::Never(_), CssMediaQuery::Typed(print)] = report.syntax().queries()
        else {
            panic!("failed enclosure and independent print: {source}: {report:?}")
        };
        assert_eq!(print.media_type(), CssMediaType::Print);
        let [diagnostic] = report.diagnostics() else {
            panic!("one lexical failure per member")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidComponentValue
        );
        let ErrorKind::InvalidComponentValue(error) = diagnostic.error().kind() else {
            panic!("original typed lexical error")
        };
        assert_eq!(error.kind(), kind);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("original media lexical origin")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            7 + payload.find(responsible).unwrap()
        );
    }
    for source in [")", "]", "}"] {
        let error = parse_component_values(source).unwrap_err();
        assert_eq!(
            error.kind(),
            CssComponentValueErrorKind::UnmatchedClosingDelimiter
        );
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("actual root closer")
        };
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), 1);
    }
}

#[test]
fn missing_function_end_can_match_property_grammar_while_retaining_eof_fault_and_closure_origin() {
    for (source, implicit) in [("translateX(12px)", false), ("translateX(12px", true)] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Known(CssKnownProperty::Transform),
            CssImportance::Normal,
        );
        let declaration = report.syntax().as_ref().unwrap();
        assert!(matches!(
            declaration.known().unwrap().property_value().unwrap(),
            CssKnownPropertyValueRef::Transform(_)
        ));
        let [component] = declaration.value_components().items() else {
            panic!("one grammar-matching transform function")
        };
        let CssComponentValueRef::Function(function) = component.view() else {
            panic!("function retained as one component")
        };
        assert_eq!(function.name(), "translateX");
        let original = declaration.parsed_value().unwrap().source();
        match (implicit, function.closing_origin()) {
            (true, CssValueOrigin::ImplicitClosure { opening, at }) => {
                assert!(opening.source().same_snapshot(original));
                assert!(at.source().same_snapshot(original));
                assert_eq!(opening.span().start().byte_offset().value(), 0);
                assert_eq!(at.span().start().byte_offset().value(), source.len());
                assert_eq!(at.span().end().byte_offset().value(), source.len());
                let [diagnostic] = report.diagnostics() else {
                    panic!("one retained syntax EOF fault")
                };
                assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
                assert_eq!(
                    diagnostic.action(),
                    CssRecoveryAction::RetainWithImplicitClosure
                );
                assert_eq!(
                    diagnostic.error().position().byte_offset().value(),
                    source.len()
                );
                assert!(report.clone().into_validation_result().is_err());
            }
            (false, CssValueOrigin::Parsed(close)) => {
                assert!(close.source().same_snapshot(original));
                assert_eq!(close.span().start().byte_offset().value(), source.len() - 1);
                assert_eq!(close.span().end().byte_offset().value(), source.len());
                assert!(report.is_clean());
                assert!(report.clone().into_validation_result().is_ok());
            }
            _ => panic!("authored close and EOF-implied close must remain distinct"),
        }
    }
}

#[test]
fn checked_groups_retain_all_opening_kinds_and_matching_explicit_closers_without_changing_nested_punctuation()
 {
    for (source, expected) in [
        ("[a;!]", CssBlockKind::SquareBracket),
        ("{a;!}", CssBlockKind::CurlyBracket),
        ("(a;!)", CssBlockKind::Parenthesis),
    ] {
        let values = parse_component_values(source).unwrap();
        let [component] = values.items() else {
            panic!("one checked complete block")
        };
        let CssComponentValueRef::Block(block) = component.view() else {
            panic!("simple block")
        };
        assert_eq!(block.kind(), expected);
        root_punctuation(component);
        let (CssValueOrigin::Parsed(open), CssValueOrigin::Parsed(close)) =
            (component.origin(), block.closing_origin())
        else {
            panic!("actual authored delimiters")
        };
        assert!(open.source().same_snapshot(close.source()));
        assert_eq!(open.span().start().byte_offset().value(), 0);
        assert_eq!(close.span().start().byte_offset().value(), 4);
        assert_eq!(close.span().end().byte_offset().value(), 5);
        assert_eq!(values.serialize().unwrap().as_css(), source);
    }
}
