//! Caller grammar boundaries through existing public parsing interfaces.
//! Independent authority: Syntax 3 CRD20261001 §§5.4.1–2, 5.4.10, 5.5.7–10, 7, 7.2;
//! selected MQ5 WD20260219 §3, Variables CR20220616 §2,
//! Sizing3 WD20260904 §3.1.1 and Transforms1 CR20190214 §9.1.
//! This is not a public raw-error stream or generic grammar DSL test.
use crate::css::support::authored_syntax::{
    assert_checked_invalid_property_value, assert_diagnostics, assert_invalid_property_value,
    assert_strict_rejection, parsed_origin as parsed,
};
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssBlockKind, CssComponentValueRef, CssComponentValues, CssSerializedOrigin, CssValueOrigin,
    CssValueTokenRef,
};
use surgeist_css::declaration::{CssCustomPropertyName, CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{
    CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef, CssPropertyTarget,
    CssPropertyTextTarget,
};
use surgeist_css::query::media::{CssMediaLeaf, CssMediaQuery, CssMediaQueryList, CssMediaType};
use surgeist_css::query::supports::CssSupportsDeclaration;
use surgeist_css::query::{CssConditionKind, CssGeneralEnclosed};
use surgeist_css::rule::CssRule;
use surgeist_css::selector::CssNamespaceContext;

fn width(
    source: &str,
) -> surgeist_css::CssParseReport<Option<surgeist_css::declaration::CssDeclaration>> {
    CssParser::default().parse_property_value(
        source,
        CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
        CssImportance::Normal,
    )
}

fn opaque(query: &CssMediaQuery) -> &CssGeneralEnclosed {
    let CssMediaQuery::Condition(condition) = query else {
        panic!("media condition")
    };
    let CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(value)) = condition.kind() else {
        panic!("opaque any-value enclosure")
    };
    value
}

fn root_punctuation(component: &surgeist_css::component::CssComponentValue) {
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
        CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Grammar(property.grammar()),
            CssImportance::Normal,
        ),
    ] {
        assert!(report.is_clean(), "{report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        let CssKnownPropertyValueRef::Width(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("defined typed width match")
        };
        assert_eq!(
            value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            "12px"
        );
        let origin = declaration.parsed_value().unwrap();
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), source.len());
        assert!(declaration.parsed_name().is_none());
    }
    // `<width>` takes one value, so the first token after it is responsible.
    for (source, responsible, token) in [
        ("12px 13px", 5, "13px"),
        ("12px,13px", 4, ","),
        ("auto extra", 5, "extra"),
        ("calc(1px + 2px) extra", 16, "extra"),
    ] {
        for report in [
            width(source),
            CssParser::default().parse_property_value(
                source,
                CssPropertyTextTarget::Grammar(property.grammar()),
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
        // A supplied checked value still needs the whole width grammar.
        let error = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert_checked_invalid_property_value(&error, property, Some(token));
        let CssSerializedOrigin::Token(origin) = error.origin() else {
            panic!("{source}: {error:?}")
        };
        let origin = parsed(&origin);
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), responsible);
    }
}

#[test]
fn media_whitespace_only_has_no_results_but_empty_comma_segments_have_individual_failures() {
    // Syntax 3 (2026-10-01) §5.4.10 appends a member only while input remains,
    // so a final comma followed by no token adds no member.
    for source in ["", " \t\r\n", " /**/ "] {
        let report = CssMediaQueryList::parse(&CssParser::default(), source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert!(report.syntax().queries().is_empty());
    }
    let source = ",screen,,print,";
    let report = CssMediaQueryList::parse(&CssParser::default(), source);
    let [
        CssMediaQuery::Never(first),
        CssMediaQuery::Typed(screen),
        CssMediaQuery::Never(middle),
        CssMediaQuery::Typed(print),
    ] = report.syntax().queries()
    else {
        panic!("leading and consecutive empty members retain failures in place")
    };
    assert_eq!(screen.media_type(), CssMediaType::Screen);
    assert_eq!(print.media_type(), CssMediaType::Print);
    for (never, at) in [(first, 0), (middle, 8)] {
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
        [CssRecoveryAction::ReplaceMediaQueryWithNever; 2]
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.error().code() == CssErrorCode::InvalidMediaQuery)
    );
    assert!(report.clone().into_validation_result().is_err());
    let comma = CssMediaQueryList::parse(&CssParser::default(), ",");
    assert!(matches!(
        comma.syntax().queries(),
        [CssMediaQuery::Never(_)]
    ));
    assert_eq!(comma.diagnostics().len(), 1);
}

#[test]
fn media_nested_commas_stay_in_one_member_and_a_failed_middle_member_preserves_the_last() {
    let enclosure = "future([a,b] {c,d} nested(e,f))";
    let source = format!("{enclosure},screen and,print");
    let report = CssMediaQueryList::parse(&CssParser::default(), &source);
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
    assert_eq!(
        value.serialize_tokens(Default::default()).unwrap().as_css(),
        enclosure
    );
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
        let values = CssComponentValues::parse(&CssParser::default(), source).unwrap();
        let [component] = values.items() else {
            panic!("one escaped identifier token")
        };
        assert!(
            matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) if value == ident)
        );
        let error = CssGeneralEnclosed::try_from_component(component.clone()).unwrap_err();
        assert!(error.is_wrong_outer_component());
        assert_eq!(error.origin(), component.origin());
    }
    let at = CssComponentValues::parse(&CssParser::default(), "@future").unwrap();
    assert!(matches!(
        at.items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::AtKeyword("future"))
    ));
    // `<general-enclosed>` is a function or a parenthesized block, never an
    // at-keyword token.
    let error = CssGeneralEnclosed::try_from_component(at.items()[0].clone()).unwrap_err();
    assert!(error.is_wrong_outer_component());
    assert_eq!(error.origin(), at.items()[0].origin());
    let function = CssComponentValues::parse(&CssParser::default(), "future()").unwrap();
    assert!(
        matches!(function.items()[0].view(), CssComponentValueRef::Function(value) if value.name() == "future")
    );
    assert_eq!(
        CssGeneralEnclosed::try_from_component(function.items()[0].clone())
            .unwrap()
            .authored(),
        Some("future()")
    );

    let escaped = CssRule::parse(
        &CssParser::default(),
        r"\@media all {}",
        &CssNamespaceContext::default(),
    );
    assert!(escaped.is_clean(), "{escaped:?}");
    assert!(matches!(escaped.syntax(), Some(CssRule::Style(_))));
    let actual = CssRule::parse(
        &CssParser::default(),
        "@media all {}",
        &CssNamespaceContext::default(),
    );
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
        assert_eq!(
            value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            "auto"
        );
    }
    // A function or at-keyword is not the `auto` identifier.
    for source in ["auto()", "@auto"] {
        let report = width(source);
        assert!(report.syntax().is_none(), "{source}");
        assert_diagnostics(
            report.diagnostics(),
            &[(
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::RejectInput,
                0,
            )],
        );
    }
    for source in [
        "translateX(12px)",
        "TrAnSlAtEx(12px)",
        r"\74 ranslateX(12px)",
    ] {
        let report = CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Transform)),
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
        CssComponentValues::parse(&CssParser::default(), ident)
            .unwrap()
            .items()[0]
            .view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("translateX("))
    ));
    assert!(
        CssParser::default()
            .parse_property_value(
                ident,
                CssPropertyTextTarget::Property(CssPropertyNameRef::Known(
                    CssKnownProperty::Transform
                )),
                CssImportance::Normal
            )
            .syntax()
            .is_none()
    );
    let media = CssMediaQuery::parse(&CssParser::default(), "ScReEn");
    assert!(media.is_clean());
    assert!(
        matches!(media.syntax(), CssMediaQuery::Typed(query) if query.media_type() == CssMediaType::Screen)
    );
}

#[test]
fn root_semicolon_and_bang_are_allowed_by_any_value_but_excluded_from_declaration_value_contexts() {
    let custom = CssCustomPropertyName::try_new("--tokens").unwrap();
    for source in ["a;", "a!", "a;!"] {
        let report = CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Custom(&custom)),
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
            CssSupportsDeclaration::parse(&CssParser::default(), "--tokens", source)
                .as_ref()
                .is_err_and(|error| error.is_invalid_declaration_grammar()),
            "two-argument method grammar: {source}"
        );
        let query = CssMediaQuery::parse(&CssParser::default(), format!("future({source})"));
        assert!(
            query.is_clean(),
            "any-value admits contents-root punctuation: {source}: {query:?}"
        );
        assert_eq!(
            opaque(query.syntax()).authored(),
            Some(format!("future({source})").as_str())
        );
    }
    let any = CssMediaQuery::parse(&CssParser::default(), "future(a;!)");
    root_punctuation(opaque(any.syntax()).component());
    for source in ["f(a;!)", "[a;!]", "{a;!}", "(a;!)"] {
        let report = CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Custom(&custom)),
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
        let supports =
            CssSupportsDeclaration::parse(&CssParser::default(), "--tokens", source).unwrap();
        assert_eq!(supports.importance(), CssImportance::Normal);
        assert!(supports.known().is_none());
    }
}

#[test]
fn nullable_defining_grammars_do_not_make_nonnullable_known_property_values_match() {
    let custom = CssCustomPropertyName::try_new("--empty").unwrap();
    for source in ["", " \t", "/**/"] {
        let report = width(source);
        assert!(
            report.syntax().is_none(),
            "width needs its own value: {source:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one missing-value rejection: {source:?}: {report:?}")
        };
        assert_invalid_property_value(diagnostic.error(), CssKnownProperty::Width, None);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len(),
            "{source:?}"
        );
        let report = CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Custom(&custom)),
            CssImportance::Normal,
        );
        assert!(
            report.is_clean(),
            "Variables selects declaration-value?: {source:?}: {report:?}"
        );
        assert!(report.syntax().as_ref().unwrap().custom().is_some());
    }
    for source in ["future()", "()", "future( \t)"] {
        let report = CssMediaQuery::parse(&CssParser::default(), source);
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
        ("f(url(a b))", "url(", ComponentFailure::BadUrl),
        ("f(\"bad\n)", "\"", ComponentFailure::BadString),
        ("f(])", "]", ComponentFailure::UnmatchedClosingDelimiter),
        ("f([)])", ")", ComponentFailure::UnmatchedClosingDelimiter),
    ] {
        let error = CssComponentValues::parse(&CssParser::default(), payload).unwrap_err();
        assert_eq!(component_failure(&error), kind, "{payload}");
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("original checked lexical failure")
        };
        assert_eq!(origin.source().as_str(), payload);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            payload.find(responsible).unwrap()
        );
        let custom = CssCustomPropertyName::try_new("--bad").unwrap();
        let property = CssParser::default().parse_property_value(
            payload,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Custom(&custom)),
            CssImportance::Normal,
        );
        assert!(
            property.syntax().is_none(),
            "declaration-value cannot retain a bad nested token: {payload}"
        );
        let [diagnostic] = property.diagnostics() else {
            panic!("one lexical rejection: {payload}: {property:?}")
        };
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            payload.find(responsible).unwrap(),
            "{payload}"
        );
        let source = format!("future({payload}),print");
        let report = CssMediaQueryList::parse(&CssParser::default(), &source);
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
        let Some(error) = diagnostic.error().component_error() else {
            panic!("original typed lexical error")
        };
        assert_eq!(component_failure(error), kind);
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
        let error = CssComponentValues::parse(&CssParser::default(), source).unwrap_err();
        assert!(error.is_unmatched_closing_delimiter(), "{error:?}");
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
        let report = CssParser::default().parse_property_value(
            source,
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Transform)),
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
                assert_strict_rejection(
                    report.clone().into_validation_result(),
                    report.diagnostics(),
                );
            }
            (false, CssValueOrigin::Parsed(close)) => {
                assert!(close.source().same_snapshot(original));
                assert_eq!(close.span().start().byte_offset().value(), source.len() - 1);
                assert_eq!(close.span().end().byte_offset().value(), source.len());
                assert!(report.is_clean());
                assert_eq!(
                    report.clone().into_validation_result().unwrap(),
                    *report.syntax()
                );
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
        let values = CssComponentValues::parse(&CssParser::default(), source).unwrap();
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
        assert_eq!(
            values
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            source
        );
    }
}
