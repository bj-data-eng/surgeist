#![forbid(unsafe_code)]

//! Public consumer for the first selector/query fragment contract.
//!
//! Admission follows Selectors 4 (2026-01-22), sections 3.9 and 18; media
//! recovery follows Media Queries 4 (2026-02-19), section 3.2. Actual EOF
//! closes retained components under CSS Syntax 3 (2021-12-24), “consume a
//! simple block” and “consume a function”, preserving source and parse error.
//! Sources: https://www.w3.org/TR/2026/WD-selectors-4-20260122/
//! https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/
//!
//! Literal coordinates and semantic expectations below are independent of
//! parser output. The current public responsible-position/report contract
//! supplies diagnostic ordering, complete recovery units, and the 256-level
//! limit. No wrapper is an input or an oracle for these fragment assertions.

use std::fmt::Debug;
use surgeist_css::{
    CssCompoundSelector, CssDefinedFalseMediaReason, CssErrorCode, CssLengthUnit,
    CssMediaConditionKind, CssMediaFeatureQuery, CssMediaQuery, CssMediaQueryList,
    CssMediaQueryModifier, CssMediaType, CssNamespaceConstraint, CssNamespaceContext,
    CssNamespaceName, CssNamespacePrefix, CssParseReport, CssPseudoClass, CssQueryLength,
    CssRecoveryAction, CssRecoveryDiagnostic, CssSelector, CssSelectorCombinator,
    CssSourcePosition, CssStyleSelector, CssStyleSelectorList, CssTokenKind, ErrorKind,
    parse_media_query, parse_media_query_list, parse_selector, parse_selector_list, parse_sheet,
};

fn assert_position(actual: CssSourcePosition, expected: (usize, u32, u32)) {
    assert_eq!(
        actual.byte_offset().value(),
        expected.0,
        "UTF-8 byte offset"
    );
    assert_eq!(actual.line().value(), expected.1, "zero-based line");
    assert_eq!(
        actual.column().value(),
        expected.2,
        "zero-based UTF-16 column"
    );
}

fn assert_span(diagnostic: &CssRecoveryDiagnostic, start: usize, end: usize) {
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
}

fn validate_report<T>(report: CssParseReport<T>) -> Result<T, surgeist_css::CssValidationFailure> {
    report.into_validation_result()
}

fn assert_validation_parity<T: Clone + Debug + PartialEq>(report: &CssParseReport<T>) {
    assert_eq!(report.is_clean(), report.diagnostics().is_empty());
    let (syntax, diagnostics) = report.clone().into_parts();
    assert_eq!(&syntax, report.syntax());
    assert_eq!(diagnostics, report.diagnostics());
    let validated = validate_report(report.clone());
    if diagnostics.is_empty() {
        assert_eq!(validated.unwrap(), syntax);
    } else {
        let failure = validated.unwrap_err();
        assert_eq!(failure.first(), &diagnostics[0]);
        assert_eq!(failure.diagnostics(), diagnostics);
        assert_eq!(failure.into_diagnostics(), diagnostics);
    }
}

fn clean_selector(source: &str, context: &CssNamespaceContext) -> CssSelector {
    let report = parse_selector(source, context);
    assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
    assert_validation_parity(&report);
    report
        .into_validation_result()
        .unwrap()
        .expect("one selector")
}

fn compound(selector: &CssSelector) -> &CssCompoundSelector {
    let CssSelector::Compound(compound) = selector else {
        panic!("expected a compound selector: {selector:?}");
    };
    compound
}

fn prefix(name: &str) -> CssNamespacePrefix {
    CssNamespacePrefix::try_new(name).expect("valid decoded namespace prefix")
}

fn assert_type_namespace(selector: &CssSelector, namespace: CssNamespaceConstraint) {
    let name = compound(selector)
        .type_selector()
        .expect("authored type name");
    assert_eq!(name.local_name(), Some("leaf"));
    assert_eq!(name.namespace(), &namespace);
}

fn first_slice_signatures_and_namespace_context() {
    let _: fn(&str, &CssNamespaceContext) -> CssParseReport<Option<CssSelector>> = parse_selector;
    let _: fn(&str, &CssNamespaceContext) -> CssParseReport<Option<CssStyleSelectorList>> =
        parse_selector_list;
    let _: fn(&str) -> CssParseReport<CssMediaQuery> = parse_media_query;
    let _: fn(&str) -> CssParseReport<CssMediaQueryList> = parse_media_query_list;

    let absent = CssNamespaceContext::default();
    assert!(absent.default_namespace().is_none());
    assert!(absent.named_namespace(&prefix("svg")).is_none());
    assert_type_namespace(
        &clean_selector("leaf.mark", &absent),
        CssNamespaceConstraint::Any,
    );

    let empty_default = CssNamespaceContext::from_bindings([(None, CssNamespaceName::new(""))]);
    assert_eq!(empty_default.default_namespace().unwrap().as_str(), "");
    assert_type_namespace(
        &clean_selector("leaf.mark", &empty_default),
        CssNamespaceConstraint::Default,
    );
    let attribute = clean_selector("leaf[data-x]", &empty_default);
    assert_eq!(
        compound(&attribute).attributes()[0].namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );

    // IntoIterator, authored order, exact prefix case, and empty named bindings.
    let bindings = [
        (None, CssNamespaceName::new("urn:old")),
        (Some(prefix("svg")), CssNamespaceName::new("urn:old-svg")),
        (Some(prefix("SVG")), CssNamespaceName::new("urn:upper")),
        (None, CssNamespaceName::new("urn:default")),
        (Some(prefix("svg")), CssNamespaceName::new("")),
    ];
    let context = CssNamespaceContext::from_bindings(bindings);
    assert_eq!(context.default_namespace().unwrap().as_str(), "urn:default");
    assert_eq!(
        context.named_namespace(&prefix("svg")).unwrap().as_str(),
        ""
    );
    assert_eq!(
        context.named_namespace(&prefix("SVG")).unwrap().as_str(),
        "urn:upper"
    );
    assert!(context.named_namespace(&prefix("Svg")).is_none());
    assert_type_namespace(
        &clean_selector("svg|leaf.mark", &context),
        CssNamespaceConstraint::Named(prefix("svg")),
    );
    assert_type_namespace(
        &clean_selector("SVG|leaf.mark", &context),
        CssNamespaceConstraint::Named(prefix("SVG")),
    );
    assert_type_namespace(
        &clean_selector("|leaf.mark", &context),
        CssNamespaceConstraint::ExplicitNone,
    );
    assert_type_namespace(
        &clean_selector("*|leaf.mark", &context),
        CssNamespaceConstraint::Any,
    );
    let missing = parse_selector("Svg|leaf", &context);
    assert!(missing.syntax().is_none());
    assert_eq!(missing.diagnostics().len(), 1);
    assert_eq!(
        missing.diagnostics()[0].action(),
        CssRecoveryAction::RejectInput
    );
    assert_validation_parity(&missing);
    // Parsing is a shared borrow and does not register an undeclared prefix.
    assert!(context.named_namespace(&prefix("Svg")).is_none());

    let sheet = parse_sheet(concat!(
        "@namespace \"urn:old\";@namespace svg \"urn:old-svg\";",
        "@namespace SVG \"urn:upper\";@namespace \"\";@namespace svg \"\";",
        ".kept{color:red}",
    ));
    assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
    let from_sheet = CssNamespaceContext::from_sheet(sheet.syntax());
    drop(sheet);
    assert_eq!(from_sheet.default_namespace().unwrap().as_str(), "");
    assert_eq!(
        from_sheet.named_namespace(&prefix("svg")).unwrap().as_str(),
        ""
    );
    assert_eq!(
        from_sheet.named_namespace(&prefix("SVG")).unwrap().as_str(),
        "urn:upper"
    );
    assert!(from_sheet.named_namespace(&prefix("Svg")).is_none());
    let symbolic = clean_selector("svg|leaf.mark", &from_sheet);
    assert_type_namespace(&symbolic, CssNamespaceConstraint::Named(prefix("svg")));
    assert_eq!(
        from_sheet.named_namespace(&prefix("svg")).unwrap().as_str(),
        ""
    );
    let blank_sheet = parse_sheet("");
    let blank_context = CssNamespaceContext::from_sheet(blank_sheet.syntax());
    assert!(blank_context.default_namespace().is_none());
    assert!(blank_context.named_namespace(&prefix("svg")).is_none());
    println!("fragment signatures and immutable namespace context: ok");
}

fn selector_admission_and_semantics() {
    let context = CssNamespaceContext::default();
    assert_eq!(
        clean_selector(" /*comment*/ .card \r\n", &context),
        CssSelector::Class("card".into())
    );
    let complex = clean_selector("article#first#second.card[data-x] > a:hover", &context);
    let CssSelector::Complex(complex) = complex else {
        panic!("one complex selector")
    };
    assert_eq!(complex.first().ids(), ["first", "second"]);
    assert_eq!(complex.first().classes(), ["card"]);
    assert_eq!(complex.first().attributes()[0].name().as_str(), "data-x");
    let [part] = complex.rest() else {
        panic!("one child combinator")
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(part.selector().pseudo_classes(), [CssPseudoClass::Hover]);

    let report = parse_selector_list(".a,:is(.b,.c),[data-x=\"a,b\"]", &context);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let list = report.syntax().as_ref().expect("three ordinary selectors");
    assert_eq!(
        list.selectors().len(),
        3,
        "nested commas are not root commas"
    );
    assert!(
        list.selectors()
            .iter()
            .all(|selector| matches!(selector, CssStyleSelector::Selector(_)))
    );
    assert_eq!(
        list.selectors()[0].selector(),
        &CssSelector::Class("a".into())
    );
    assert_validation_parity(&report);

    let anchor = clean_selector("&.a", &context);
    assert_eq!(compound(&anchor).nesting_selectors(), 1);
    assert!(!compound(&anchor).has_scope_anchor());

    // Each root must consume exactly its requested grammar, including EOF.
    for source in [
        "",
        " \r\n/**/",
        ">.a",
        ".a{}",
        ".a;",
        ".a,???,.b",
        ":not(.a,???)",
        ":has(.a,???)",
    ] {
        let report = parse_selector_list(source, &context);
        assert!(report.syntax().is_none(), "{source:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one rejected root for {source:?}: {:?}",
                report.diagnostics()
            )
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_span(diagnostic, 0, source.len());
        assert_validation_parity(&report);
    }
    for (source, offset, kind, spelling) in [
        (".a,.b", 2, CssTokenKind::Comma, ","),
        (".a,:is(.ok", 2, CssTokenKind::Comma, ","),
        ("[a|] {}", 2, CssTokenKind::Delim, "|"),
        (":has(.a{)", 7, CssTokenKind::CurlyBracketBlock, "{"),
        (":test", 1, CssTokenKind::Ident, "test"),
    ] {
        let report = parse_selector(source, &context);
        assert!(report.syntax().is_none(), "{source:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one rejected selector for {source:?}: {:?}",
                report.diagnostics()
            )
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_span(diagnostic, 0, source.len());
        assert_position(diagnostic.error().position(), (offset, 0, offset as u32));
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("typed selector error")
        };
        let encountered = detail.encountered().expect("responsible authored token");
        assert_eq!(encountered.kind(), kind);
        assert_eq!(encountered.authored(), spelling);
        assert_validation_parity(&report);
    }
    let trailing = parse_selector_list(".a,", &context);
    assert!(trailing.syntax().is_none());
    let [diagnostic] = trailing.diagnostics() else {
        panic!("one missing-selector error")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_position(diagnostic.error().position(), (3, 0, 3));
    assert_span(diagnostic, 0, 3);
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("typed missing selector")
    };
    assert!(
        detail.encountered().is_none(),
        "actual EOF has no wrapper token"
    );
    assert_validation_parity(&trailing);
    println!("selector exact admission and authored semantics: ok");
}

fn forgiving_members(selector: &CssSelector) -> &[CssSelector] {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::Is(list) | CssPseudoClass::Where(list)) => {
            list.selectors()
        }
        _ => panic!("expected an :is() or :where() selector"),
    }
}

fn assert_eof_closures<T>(report: &CssParseReport<T>, count: usize, eof: (usize, u32, u32)) {
    let closures = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .collect::<Vec<_>>();
    assert_eq!(closures.len(), count);
    for diagnostic in closures {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_position(diagnostic.error().position(), eof);
        assert_position(diagnostic.span().start(), eof);
        assert_position(diagnostic.span().end(), eof);
        assert!(matches!(
            diagnostic.error().kind(),
            ErrorKind::UnexpectedEnd(_)
        ));
    }
}

fn selector_recovery_eof_and_coordinates() {
    let context = CssNamespaceContext::default();
    let source = ":is(???(a,b),.ok)";
    let report = parse_selector(source, &context);
    assert_eq!(
        forgiving_members(report.syntax().as_ref().unwrap()),
        [CssSelector::Class("ok".into())]
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one balanced malformed member")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropSelectorListItem);
    assert_span(diagnostic, 4, 12);
    assert_position(diagnostic.error().position(), (4, 0, 4));
    assert_validation_parity(&report);
    let empty = parse_selector(":where(???)", &context);
    assert!(forgiving_members(empty.syntax().as_ref().unwrap()).is_empty());
    assert_eq!(empty.diagnostics().len(), 1);
    assert_eq!(
        empty.diagnostics()[0].action(),
        CssRecoveryAction::DropSelectorListItem
    );
    assert_validation_parity(&empty);

    let recovered = parse_selector(":is(.ok,???", &context);
    assert_eq!(
        forgiving_members(recovered.syntax().as_ref().unwrap()),
        [CssSelector::Class("ok".into())]
    );
    assert_eq!(recovered.diagnostics().len(), 2);
    assert_eq!(
        recovered.diagnostics()[0].action(),
        CssRecoveryAction::DropSelectorListItem
    );
    assert_span(&recovered.diagnostics()[0], 8, 11);
    assert_eof_closures(&recovered, 1, (11, 0, 11));
    assert_validation_parity(&recovered);

    // The inner forgiving error survives, but the rejected unclosed :not()
    // does not publish a closure claiming that its syntax was retained.
    let source = ":not(:is(.ok,???),???";
    let rejected = parse_selector(source, &context);
    assert!(rejected.syntax().is_none());
    let [inner, outer] = rejected.diagnostics() else {
        panic!(
            "inner recovery plus outer rejection: {:?}",
            rejected.diagnostics()
        )
    };
    assert_eq!(inner.action(), CssRecoveryAction::DropSelectorListItem);
    assert_span(inner, 13, 16);
    assert_position(inner.error().position(), (13, 0, 13));
    assert_eq!(outer.action(), CssRecoveryAction::RejectInput);
    assert_span(outer, 0, 21);
    assert_position(outer.error().position(), (18, 0, 18));
    assert_eof_closures(&rejected, 0, (21, 0, 21));
    assert_validation_parity(&rejected);

    // Whole-list parsing followed by a length check would parse the second
    // :is() and could leak its implicit closure into this rejected single root.
    let rejected = parse_selector(":is(???,.ok),:is(.after", &context);
    assert!(rejected.syntax().is_none());
    let [inner, outer] = rejected.diagnostics() else {
        panic!("inner recovery plus one exact-admission failure")
    };
    assert_eq!(inner.action(), CssRecoveryAction::DropSelectorListItem);
    assert_span(inner, 4, 7);
    assert_position(inner.error().position(), (4, 0, 4));
    assert_eq!(outer.action(), CssRecoveryAction::RejectInput);
    assert_span(outer, 0, 23);
    assert_position(outer.error().position(), (12, 0, 12));
    assert_eof_closures(&rejected, 0, (23, 0, 23));
    assert_validation_parity(&rejected);

    // Two-byte é, four-byte 😀, and CRLF keep byte/UTF-16 coordinates distinct.
    let source = "/*😀*/\r\n/*é*/:test";
    let rejected = parse_selector(source, &context);
    assert!(rejected.syntax().is_none());
    let [diagnostic] = rejected.diagnostics() else {
        panic!("one unknown named pseudo")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_span(diagnostic, 0, 21);
    assert_position(diagnostic.error().position(), (17, 1, 6));
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("typed selector error")
    };
    let token = detail.encountered().unwrap();
    assert_eq!(token.kind(), CssTokenKind::Ident);
    assert_eq!(token.authored(), "test");
    assert_validation_parity(&rejected);

    let source = "/*😀*/\r\n/*é*/[data-x";
    let open = parse_selector(source, &context);
    assert_eq!(
        open.syntax().as_ref().unwrap(),
        &clean_selector("[data-x]", &context)
    );
    assert_eq!(open.diagnostics().len(), 1);
    assert_eof_closures(&open, 1, (23, 1, 12));
    assert_validation_parity(&open);
    let has = clean_selector(":has(>.a,.b)", &context);
    let CssSelector::PseudoClass(CssPseudoClass::Has(list)) = has else {
        panic!("typed relative selectors inside :has()")
    };
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        list.selectors()[1].combinator(),
        CssSelectorCombinator::Descendant
    );
    println!("selector forgiving recovery and actual EOF provenance: ok");
}

fn assert_typed_query(query: &CssMediaQuery, expected: CssMediaType, offset: usize) {
    let CssMediaQuery::Typed(query) = query else {
        panic!("typed media query")
    };
    assert_eq!(query.media_type(), expected);
    assert_eq!(query.position().byte_offset().value(), offset);
}

fn assert_defined_false(query: &CssMediaQuery, source: &str, reason: CssDefinedFalseMediaReason) {
    assert!(
        !query.is_guaranteed_false(),
        "defined-false syntax is not a malformed-member sentinel"
    );
    let CssMediaQuery::Condition(condition) = query else {
        panic!("authored media condition")
    };
    let CssMediaConditionKind::DefinedFalse(value) = condition.kind() else {
        panic!("defined-false media expression")
    };
    assert_eq!(value.as_css(), source);
    assert_eq!(value.reason(), reason);
}

fn media_admission_and_recovery() {
    for source in ["", " \r\n/**/"] {
        let list = parse_media_query_list(source);
        assert!(list.is_clean(), "an empty list is valid authored syntax");
        assert!(list.syntax().queries().is_empty());
        assert_validation_parity(&list);
        let single = parse_media_query(source);
        assert!(
            single.syntax().is_guaranteed_false(),
            "one requested query cannot be empty"
        );
        let [diagnostic] = single.diagnostics() else {
            panic!("one missing query")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
        assert_span(diagnostic, 0, source.len());
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(
            single.syntax().position().byte_offset().value(),
            source.len()
        );
        assert_validation_parity(&single);
    }
    let typed = parse_media_query("only screen and (width:1px)");
    assert!(typed.is_clean(), "{:?}", typed.diagnostics());
    let CssMediaQuery::Typed(value) = typed.syntax() else {
        panic!("one typed query")
    };
    assert_eq!(value.modifier(), Some(CssMediaQueryModifier::Only));
    assert_eq!(value.media_type(), CssMediaType::Screen);
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(width)) =
        value.condition().unwrap().kind()
    else {
        panic!("authored width condition");
    };
    assert_eq!(
        width.value(),
        &CssQueryLength::try_new(1.0, CssLengthUnit::Px).unwrap()
    );
    assert_validation_parity(&typed);

    for (source, reason) in [
        ("(fo)", CssDefinedFalseMediaReason::UnknownFeature),
        ("(width:2qu)", CssDefinedFalseMediaReason::UnknownValue),
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_defined_false(report.syntax(), source, reason);
        assert_position(report.syntax().position(), (0, 0, 0));
        assert_validation_parity(&report);
    }
    let unknown = parse_media_query(r"only F\75ture-Screen");
    assert!(unknown.is_clean(), "{:?}", unknown.diagnostics());
    let CssMediaQuery::Typed(value) = unknown.syntax() else {
        panic!("unknown authored media type")
    };
    assert_eq!(value.media_type(), CssMediaType::Unknown);
    assert_eq!(value.modifier(), Some(CssMediaQueryModifier::Only));
    let name = value.unknown_media_type().unwrap();
    assert_eq!(name.as_css(), r"F\75ture-Screen");
    assert_eq!(name.reason(), CssDefinedFalseMediaReason::UnknownType);
    assert_position(name.position(), (5, 0, 5));
    assert!(!unknown.syntax().is_guaranteed_false());
    assert_validation_parity(&unknown);

    let list = parse_media_query_list("screen,print");
    assert!(list.is_clean());
    assert_eq!(list.syntax().queries().len(), 2);
    assert_typed_query(&list.syntax().queries()[0], CssMediaType::Screen, 0);
    assert_typed_query(&list.syntax().queries()[1], CssMediaType::Print, 7);
    assert_validation_parity(&list);
    let single = parse_media_query("screen,print");
    assert!(single.syntax().is_guaranteed_false());
    let [diagnostic] = single.diagnostics() else {
        panic!("single-query trailing comma rejects the complete input")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_span(diagnostic, 0, 12);
    assert_position(diagnostic.error().position(), (6, 0, 6));
    let ErrorKind::InvalidMediaQuery(detail) = diagnostic.error().kind() else {
        panic!("typed query error")
    };
    let token = detail.encountered().unwrap();
    assert_eq!(token.kind(), CssTokenKind::Comma);
    assert_eq!(token.authored(), ",");
    assert_validation_parity(&single);

    let source = "???,screen,,print,???";
    let list = parse_media_query_list(source);
    let queries = list.syntax().queries();
    assert_eq!(queries.len(), 5);
    assert_typed_query(&queries[1], CssMediaType::Screen, 4);
    assert_typed_query(&queries[3], CssMediaType::Print, 12);
    for (index, offset) in [(0, 0), (2, 11), (4, 18)] {
        assert!(queries[index].is_guaranteed_false());
        assert_eq!(queries[index].position().byte_offset().value(), offset);
    }
    assert_eq!(list.diagnostics().len(), 3);
    for (diagnostic, (start, end, offset)) in
        list.diagnostics()
            .iter()
            .zip([(0, 3, 0), (11, 12, 11), (18, 21, 18)])
    {
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        assert_span(diagnostic, start, end);
        assert_position(diagnostic.error().position(), (offset, 0, offset as u32));
    }
    assert_validation_parity(&list);
    for (source, span_start, span_end, responsible, never_index) in
        [(",screen", 0, 1, 0, 0), ("screen,", 6, 7, 7, 1)]
    {
        let report = parse_media_query_list(source);
        assert_eq!(report.syntax().queries().len(), 2);
        assert!(report.syntax().queries()[never_index].is_guaranteed_false());
        let [diagnostic] = report.diagnostics() else {
            panic!("one empty comma-delimited member")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        assert_span(diagnostic, span_start, span_end);
        assert_position(
            diagnostic.error().position(),
            (responsible, 0, responsible as u32),
        );
        assert_validation_parity(&report);
    }

    // A comma inside an invalid balanced function belongs to one malformed
    // query, while the root comma still admits the following valid query.
    let report = parse_media_query_list("???(a,b),print");
    assert_eq!(report.syntax().queries().len(), 2);
    assert!(report.syntax().queries()[0].is_guaranteed_false());
    assert_typed_query(&report.syntax().queries()[1], CssMediaType::Print, 9);
    let [diagnostic] = report.diagnostics() else {
        panic!("one complete malformed member")
    };
    assert_span(diagnostic, 0, 8);
    assert_position(diagnostic.error().position(), (0, 0, 0));
    assert_validation_parity(&report);
    let balanced = parse_media_query_list("(unknown:f(a,b)),print");
    assert!(balanced.is_clean(), "{:?}", balanced.diagnostics());
    assert_eq!(balanced.syntax().queries().len(), 2);
    assert_defined_false(
        &balanced.syntax().queries()[0],
        "(unknown:f(a,b))",
        CssDefinedFalseMediaReason::UnknownFeature,
    );
    assert_typed_query(&balanced.syntax().queries()[1], CssMediaType::Print, 17);
    assert_validation_parity(&balanced);
    println!("media exact admission and member recovery: ok");
}

fn media_actual_eof_and_coordinates() {
    let eof = parse_media_query("(fo");
    assert_defined_false(
        eof.syntax(),
        "(fo",
        CssDefinedFalseMediaReason::UnknownFeature,
    );
    assert_eq!(eof.diagnostics().len(), 1);
    assert_eof_closures(&eof, 1, (3, 0, 3));
    assert_validation_parity(&eof);
    let list_eof = parse_media_query_list("print,(fo");
    assert_eq!(list_eof.syntax().queries().len(), 2);
    assert_typed_query(&list_eof.syntax().queries()[0], CssMediaType::Print, 0);
    assert_defined_false(
        &list_eof.syntax().queries()[1],
        "(fo",
        CssDefinedFalseMediaReason::UnknownFeature,
    );
    assert_position(list_eof.syntax().queries()[1].position(), (6, 0, 6));
    assert_eq!(list_eof.diagnostics().len(), 1);
    assert_eof_closures(&list_eof, 1, (9, 0, 9));
    assert_validation_parity(&list_eof);

    let source = "/*😀*/\r\n/*é*/???,print";
    let report = parse_media_query_list(source);
    assert_eq!(report.syntax().queries().len(), 2);
    assert!(report.syntax().queries()[0].is_guaranteed_false());
    assert_position(report.syntax().queries()[0].position(), (16, 1, 5));
    assert_position(report.syntax().queries()[1].position(), (20, 1, 9));
    let [diagnostic] = report.diagnostics() else {
        panic!("one malformed first member")
    };
    assert_span(diagnostic, 0, 19);
    assert_position(diagnostic.error().position(), (16, 1, 5));
    let ErrorKind::InvalidMediaQuery(detail) = diagnostic.error().kind() else {
        panic!("typed media error")
    };
    assert_eq!(detail.encountered().unwrap().kind(), CssTokenKind::Delim);
    assert_eq!(detail.encountered().unwrap().authored(), "?");
    assert_validation_parity(&report);

    let open = parse_media_query("/*😀*/\r\n/*é*/(fo");
    assert_defined_false(
        open.syntax(),
        "(fo",
        CssDefinedFalseMediaReason::UnknownFeature,
    );
    assert_position(open.syntax().position(), (16, 1, 5));
    assert_eq!(open.diagnostics().len(), 1);
    assert_eof_closures(&open, 1, (19, 1, 8));
    assert_validation_parity(&open);
    let malformed = parse_media_query("(width:");
    assert!(malformed.syntax().is_guaranteed_false());
    assert_eq!(malformed.diagnostics().len(), 1);
    assert_eq!(
        malformed.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eof_closures(&malformed, 0, (7, 0, 7));
    assert_validation_parity(&malformed);
    println!("media actual EOF and source coordinates: ok");
}

fn nested_selector(depth: usize, closed: bool) -> String {
    let mut source = ":is(".repeat(depth);
    source.push_str("svg|leaf");
    if closed {
        source.push_str(&")".repeat(depth));
    }
    source
}

fn assert_nested_selector(mut selector: &CssSelector, depth: usize) {
    for _ in 0..depth {
        let [inner] = forgiving_members(selector) else {
            panic!("one retained member at each depth")
        };
        selector = inner;
    }
    assert_type_namespace(selector, CssNamespaceConstraint::Named(prefix("svg")));
}

fn nested_media(depth: usize, closed: bool) -> String {
    let mut source = String::from("(unknown: ");
    source.push_str(&"f(".repeat(depth - 1));
    source.push('x');
    if closed {
        source.push_str(&")".repeat(depth));
    }
    source
}

fn assert_limit(
    diagnostic: &CssRecoveryDiagnostic,
    offset: usize,
    start: usize,
    end: usize,
    production: &str,
) {
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_position(diagnostic.error().position(), (offset, 0, offset as u32));
    assert_span(diagnostic, start, end);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("typed limit details")
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(detail.enclosing_production().as_str(), production);
}

fn exact_depth_and_eof_on_an_ordinary_thread() {
    // This matches an ordinary Rust test-thread stack. The consumer must not
    // supply the parser's larger stack and accidentally mask a missing >128 path.
    std::thread::Builder::new()
        .name("fragment-contract-small-stack".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let context = CssNamespaceContext::from_bindings([(
                Some(prefix("svg")),
                CssNamespaceName::new("urn:svg"),
            )]);
            for depth in [128_usize, 129, 255, 256] {
                for closed in [true, false] {
                    let source = nested_selector(depth, closed);
                    let single = parse_selector(&source, &context);
                    assert_nested_selector(
                        single.syntax().as_ref().expect("within selector limit"),
                        depth,
                    );
                    assert_eq!(single.diagnostics().len(), if closed { 0 } else { depth });
                    assert_eof_closures(
                        &single,
                        if closed { 0 } else { depth },
                        (source.len(), 0, source.len() as u32),
                    );
                    assert_validation_parity(&single);
                    let list = parse_selector_list(&source, &context);
                    let [selector] = list.syntax().as_ref().unwrap().selectors() else {
                        panic!("one deep list member")
                    };
                    assert_nested_selector(selector.selector(), depth);
                    assert_eq!(list.diagnostics(), single.diagnostics());
                    assert_validation_parity(&list);

                    let source = nested_media(depth, closed);
                    let single = parse_media_query(&source);
                    assert_defined_false(
                        single.syntax(),
                        &source,
                        CssDefinedFalseMediaReason::UnknownFeature,
                    );
                    assert_eq!(single.diagnostics().len(), if closed { 0 } else { depth });
                    assert_eof_closures(
                        &single,
                        if closed { 0 } else { depth },
                        (source.len(), 0, source.len() as u32),
                    );
                    assert_validation_parity(&single);
                    let list = parse_media_query_list(&source);
                    assert_eq!(list.syntax().queries(), [single.syntax().clone()]);
                    assert_eq!(list.diagnostics(), single.diagnostics());
                    assert_validation_parity(&list);
                }
            }
            for closed in [true, false] {
                let source = nested_selector(257, closed);
                let single = parse_selector(&source, &context);
                assert!(single.syntax().is_none());
                let [diagnostic] = single.diagnostics() else {
                    panic!("one first-over-limit selector error")
                };
                assert_limit(
                    diagnostic,
                    1025,
                    0,
                    source.len(),
                    "baseline.selector.complex",
                );
                assert_validation_parity(&single);
                let list = parse_selector_list(&source, &context);
                assert!(list.syntax().is_none());
                assert_eq!(list.diagnostics(), single.diagnostics());
                assert_validation_parity(&list);

                let source = nested_media(257, closed);
                let single = parse_media_query(&source);
                assert!(single.syntax().is_guaranteed_false());
                let [diagnostic] = single.diagnostics() else {
                    panic!("one first-over-limit media error")
                };
                assert_limit(
                    diagnostic,
                    520,
                    0,
                    source.len(),
                    "baseline.media.query-list",
                );
                assert_validation_parity(&single);
                let list = parse_media_query_list(&source);
                assert_eq!(list.syntax().queries(), [single.syntax().clone()]);
                assert_eq!(list.diagnostics(), single.diagnostics());
                assert_validation_parity(&list);
            }
            let selector_source = format!(".first,{},.after", nested_selector(257, true));
            let selectors = parse_selector_list(&selector_source, &context);
            assert!(
                selectors.syntax().is_none(),
                "unforgiving root list is one recovery unit"
            );
            let [diagnostic] = selectors.diagnostics() else {
                panic!("one over-limit list member")
            };
            assert_limit(
                diagnostic,
                1032,
                0,
                selector_source.len(),
                "baseline.selector.complex",
            );
            assert_validation_parity(&selectors);
            let nested = nested_media(257, true);
            let source = format!("print,{nested},screen");
            let report = parse_media_query_list(&source);
            let queries = report.syntax().queries();
            assert_eq!(queries.len(), 3);
            assert_typed_query(&queries[0], CssMediaType::Print, 0);
            assert!(queries[1].is_guaranteed_false());
            assert_position(queries[1].position(), (6, 0, 6));
            assert_typed_query(&queries[2], CssMediaType::Screen, 7 + nested.len());
            let [diagnostic] = report.diagnostics() else {
                panic!("one over-limit media member")
            };
            assert_limit(
                diagnostic,
                526,
                6,
                6 + nested.len(),
                "baseline.media.query-list",
            );
            assert_validation_parity(&report);
        })
        .expect("create ordinary consumer thread")
        .join()
        .expect("fragment parsing must not unwind");
    println!("fragment 128/129 and 255/256/257 depth plus EOF: ok");
}

fn main() {
    first_slice_signatures_and_namespace_context();
    selector_admission_and_semantics();
    selector_recovery_eof_and_coordinates();
    media_admission_and_recovery();
    media_actual_eof_and_coordinates();
    exact_depth_and_eof_on_an_ordinary_thread();
}
