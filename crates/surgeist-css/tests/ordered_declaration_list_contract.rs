#![forbid(unsafe_code)]

//! Functional tests for ordered authored-list operations.
//! Style Attributes REC 2013-11-07 §3 supplies unbraced declaration syntax;
//! the adopted list contract and owning specified-value contracts supply exact
//! punctuation, work tariffs, provider failures and occurrence preservation.
//! These new operations have no claimed executable preimplementation RED.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn parsed(source: &str) -> CssDeclaration {
    let report = parse_declaration(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().as_ref().unwrap().clone()
}

fn checked(name: &str, value: &str, importance: CssImportance) -> CssDeclaration {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name(name).unwrap(),
        parse_component_values(value).unwrap(),
        importance,
    )
    .unwrap()
}

fn custom(name: &str, values: CssComponentValues, importance: CssImportance) -> CssDeclaration {
    let name = CssCustomPropertyName::try_new(name).unwrap();
    parse_property_value(CssPropertyNameRef::Custom(&name), values, importance).unwrap()
}

fn same_parsed_origin(actual: &CssParsedOrigin, original: &CssParsedOrigin) {
    assert_eq!(actual, original);
    assert!(actual.source().same_snapshot(original.source()));
}

fn same_origin(actual: &CssValueOrigin, original: &CssValueOrigin) {
    assert_eq!(actual, original);
    match (actual, original) {
        (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) => same_parsed_origin(a, b),
        (
            CssValueOrigin::ImplicitClosure {
                opening: a,
                at: a_eof,
            },
            CssValueOrigin::ImplicitClosure {
                opening: b,
                at: b_eof,
            },
        ) => {
            same_parsed_origin(a, b);
            same_parsed_origin(a_eof, b_eof);
        }
        _ => {}
    }
}

fn same_components(actual: &CssComponentValues, original: &CssComponentValues) {
    assert_eq!(actual, original);
    assert_eq!(actual.items().len(), original.items().len());
    for (a, b) in actual.items().iter().zip(original.items()) {
        same_origin(a.origin(), b.origin());
        match (a.view(), b.view()) {
            (CssComponentValueRef::Function(a), CssComponentValueRef::Function(b)) => {
                same_origin(a.closing_origin(), b.closing_origin());
                same_components(a.values(), b.values());
            }
            (CssComponentValueRef::Block(a), CssComponentValueRef::Block(b)) => {
                same_origin(a.closing_origin(), b.closing_origin());
                same_components(a.values(), b.values());
            }
            _ => {}
        }
    }
}

fn unchanged(list: &CssDeclarationList, originals: &[CssDeclaration]) {
    assert_eq!(list.as_slice(), originals);
    for (actual, original) in list.iter().zip(originals) {
        assert!(actual.same_occurrence(original));
        assert_eq!(actual.parser_context(), original.parser_context());
        assert_eq!(actual.importance(), original.importance());
        assert_eq!(actual.parsed_name(), original.parsed_name());
        assert_eq!(actual.parsed_value(), original.parsed_value());
        if let (Some(a), Some(b)) = (actual.parsed_name(), original.parsed_name()) {
            same_parsed_origin(a, b);
        }
        if let (Some(a), Some(b)) = (actual.parsed_value(), original.parsed_value()) {
            same_parsed_origin(a, b);
        }
        same_components(actual.value_components(), original.value_components());
    }
}

fn clean_round_trip(css: &str, count: usize) {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), count);
    assert_eq!(report.syntax().to_specified_css().unwrap(), css);
    let validated = validate_style_attribute(css).unwrap();
    assert_eq!(validated.len(), count);
    assert_eq!(validated.to_specified_css().unwrap(), css);
}

#[test]
fn empty_membership_has_one_aggregate_charge_and_no_output_bytes() {
    let list = CssDeclarationList::try_new_with_limits(Vec::new(), Limits::new(1, 1, 0)).unwrap();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert_eq!(list.to_specified_css().unwrap(), "");
    assert_eq!(
        list.to_specified_css_with_limits(Limits::new(1, 1, 0))
            .unwrap(),
        ""
    );
    assert!(CssDeclarationList::try_new(Vec::new()).unwrap().is_empty());
    for (limits, kind) in [
        (Limits::new(0, 1, 0), Kind::InputNodeLimit),
        (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
    ] {
        assert_eq!(
            CssDeclarationList::try_new_with_limits(Vec::new(), limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert!(list.is_empty());
        assert_eq!(
            list.to_specified_css_with_limits(Limits::new(1, 1, 0))
                .unwrap(),
            ""
        );
    }
    clean_round_trip("", 0);
}

#[test]
fn duplicate_properties_retain_order_and_priority_under_exact_aggregate_limits() {
    let originals = vec![
        checked("color", "PuRpLe", CssImportance::Important),
        checked("color", "transparent", CssImportance::Normal),
    ];
    const EXPECTED: &str = "color: purple !important; color: transparent;";
    assert_eq!(EXPECTED.len(), 45);
    // One list plus two declaration/name/color triples: I7/P7.
    let exact = Limits::new(7, 7, 45);
    let list = CssDeclarationList::try_new_with_limits(originals.clone(), exact).unwrap();
    assert_eq!(list.to_specified_css_with_limits(exact).unwrap(), EXPECTED);
    assert_eq!(
        CssDeclarationList::try_new(originals.clone())
            .unwrap()
            .to_specified_css()
            .unwrap(),
        EXPECTED
    );
    unchanged(&list, &originals);
    for (limits, kind) in [
        (Limits::new(6, 7, 45), Kind::InputNodeLimit),
        (Limits::new(7, 6, 45), Kind::ProjectionNodeLimit),
        (Limits::new(7, 7, 44), Kind::ByteLimit),
        (Limits::new(7, 7, 0), Kind::ByteLimit),
    ] {
        assert_eq!(
            CssDeclarationList::try_new_with_limits(originals.clone(), limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        unchanged(&list, &originals);
        assert_eq!(list.to_specified_css_with_limits(exact).unwrap(), EXPECTED);
    }
    // Existing CSSOM terminal-winner selection is a different public operation.
    assert_eq!(list.serialize_cssom().unwrap(), "color: purple !important;");
    unchanged(&list, &originals);
    clean_round_trip(EXPECTED, 2);
}

#[test]
fn repeated_handles_are_repeated_work_and_limits_belong_to_each_request() {
    let declaration = checked("color", "purple", CssImportance::Normal);
    let originals = vec![
        declaration.clone(),
        declaration.clone(),
        declaration.clone(),
    ];
    const EXPECTED: &str = "color: purple; color: purple; color: purple;";
    assert_eq!(EXPECTED.len(), 44);
    assert_eq!(
        declaration
            .to_specified_css_with_limits(Limits::new(3, 3, 14))
            .unwrap(),
        "color: purple;"
    );
    // Three individually affordable occurrences still share one aggregate budget.
    assert_eq!(
        CssDeclarationList::try_new_with_limits(originals.clone(), Limits::new(3, 3, 44))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    let list =
        CssDeclarationList::try_new_with_limits(originals.clone(), Limits::new(100, 100, 100))
            .unwrap();
    for (limits, kind) in [
        (Limits::new(9, 10, 44), Kind::InputNodeLimit),
        (Limits::new(10, 9, 44), Kind::ProjectionNodeLimit),
        (Limits::new(10, 10, 43), Kind::ByteLimit),
    ] {
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        unchanged(&list, &originals);
    }
    assert!(list[0].same_occurrence(&list[1]));
    assert!(list[1].same_occurrence(&list[2]));
    assert_eq!(
        list.to_specified_css_with_limits(Limits::new(10, 10, 44))
            .unwrap(),
        EXPECTED
    );
    assert_eq!(list.to_specified_css().unwrap(), EXPECTED);
    unchanged(&list, &originals);
    clean_round_trip(EXPECTED, 3);
}

#[test]
fn assembly_preserves_distinct_snapshots_original_contexts_and_checked_provenance() {
    let first = parsed("CoLoR:PuRpLe!important");
    let second = parsed("CoLoR:PuRpLe!important");
    assert!(
        !first
            .parsed_name()
            .unwrap()
            .source()
            .same_snapshot(second.parsed_name().unwrap().source())
    );
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let report = quirks.parse_declaration("width:7");
    assert!(report.is_clean());
    let width = report.syntax().as_ref().unwrap().clone();
    assert_eq!(width.parser_context(), quirks);
    let programmatic = parse_property_value_for_grammar(
        CssPropertyGrammar::from_name("color").unwrap(),
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("transparent").unwrap()])
            .unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(programmatic.parsed_name().is_none());
    assert!(programmatic.parsed_value().is_none());
    assert_eq!(
        programmatic.value_components().items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    let originals = vec![first, second, width, programmatic];
    let list = CssDeclarationList::try_new(originals.clone()).unwrap();
    const EXPECTED: &str =
        "color: purple !important; color: purple !important; width: 7px; color: transparent;";
    assert_eq!(list.to_specified_css().unwrap(), EXPECTED);
    unchanged(&list, &originals);
    assert_eq!(list[0].position().unwrap().byte_offset().value(), 0);
    assert_eq!(
        list[0]
            .parsed_name()
            .unwrap()
            .span()
            .end()
            .byte_offset()
            .value(),
        5
    );
    assert_eq!(
        list[0]
            .parsed_value()
            .unwrap()
            .span()
            .start()
            .byte_offset()
            .value(),
        6
    );
    assert_eq!(
        list[0]
            .parsed_value()
            .unwrap()
            .span()
            .end()
            .byte_offset()
            .value(),
        12
    );
    assert_eq!(list[2].parsed_value().unwrap().source().as_str(), "width:7");
    clean_round_trip(EXPECTED, 4);
    unchanged(&list, &originals);
}

#[test]
fn custom_case_empty_values_globals_and_pending_values_keep_their_authored_meaning() {
    let originals = vec![
        custom(
            "--Case",
            parse_component_values("INITIAL").unwrap(),
            CssImportance::Normal,
        ),
        custom(
            "--case",
            parse_component_values("MiXeD/**/  Tail").unwrap(),
            CssImportance::Important,
        ),
        custom(
            "--Empty",
            parse_component_values("").unwrap(),
            CssImportance::Normal,
        ),
        checked("width", "INHERIT", CssImportance::Normal),
        checked("width", "VaR(--Case)", CssImportance::Important),
        custom(
            "--Pending",
            parse_component_values("var(--Case, /*in*/  01.00PX )").unwrap(),
            CssImportance::Normal,
        ),
    ];
    assert!(
        originals[4]
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    let list = CssDeclarationList::try_new(originals.clone()).unwrap();
    const EXPECTED: &str = "--Case: INITIAL; --case: MiXeD/**/  Tail !important; --Empty: ; width: inherit; width: VaR(--Case) !important; --Pending: var(--Case, /*in*/  01.00PX );";
    assert_eq!(list.to_specified_css().unwrap(), EXPECTED);
    assert_eq!(list[0].custom().unwrap().name().as_str(), "--Case");
    assert_eq!(list[1].custom().unwrap().name().as_str(), "--case");
    assert!(list[2].value_components().items().is_empty());
    unchanged(&list, &originals);
    clean_round_trip(EXPECTED, 6);
    unchanged(&list, &originals);
}

#[test]
fn complete_admission_charges_utf8_and_inserted_boundaries_after_edge_trimming() {
    let a = parse_component_values("A").unwrap().items()[0].clone();
    let leading = parse_component_values(" \t").unwrap().items()[0].clone();
    let trailing = parse_component_values("\n").unwrap().items()[0].clone();
    let b = CssComponentValue::try_ident("B").unwrap();
    let values = CssComponentValues::try_new(vec![leading, a, b, trailing]).unwrap();
    let declaration = custom("--空", values, CssImportance::Normal);
    let originals = vec![declaration.clone()];
    const EXPECTED: &str = "--空: A/**/B;";
    assert_eq!(EXPECTED.chars().count(), 12);
    assert_eq!(EXPECTED.len(), 14);
    // I/P: list1 + declaration/name2 + retained aggregate1 + four leaves4.
    let exact = Limits::new(8, 8, 14);
    let list = CssDeclarationList::try_new_with_limits(originals.clone(), exact).unwrap();
    assert_eq!(list.to_specified_css_with_limits(exact).unwrap(), EXPECTED);
    assert!(list[0].parsed_name().is_none());
    assert!(list[0].parsed_value().is_none());
    assert!(matches!(
        list[0].value_components().items()[1].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        list[0].value_components().items()[2].origin(),
        &CssValueOrigin::Programmatic
    );
    for (limits, kind) in [
        (Limits::new(7, 8, 14), Kind::InputNodeLimit),
        (Limits::new(8, 7, 14), Kind::ProjectionNodeLimit),
        (Limits::new(8, 8, 13), Kind::ByteLimit),
        (Limits::new(8, 8, 12), Kind::ByteLimit),
    ] {
        assert_eq!(
            CssDeclarationList::try_new_with_limits(originals.clone(), limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        unchanged(&list, &originals);
        assert_eq!(declaration.to_specified_css().unwrap(), EXPECTED);
        assert_eq!(list.to_specified_css_with_limits(exact).unwrap(), EXPECTED);
    }
    let reparsed = validate_style_attribute(EXPECTED).unwrap();
    let mut identifiers = Vec::new();
    for item in reparsed[0].value_components().items() {
        match item.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => identifiers.push(name),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                if identifiers.is_empty() => {}
            CssComponentValueRef::Comment(_) => {}
            other => panic!("changed inner token boundary: {other:?}"),
        }
    }
    assert_eq!(identifiers, ["A", "B"]);
    clean_round_trip(EXPECTED, 1);
    unchanged(&list, &originals);
}

#[test]
fn serializing_a_recovered_member_preserves_its_original_implicit_closure() {
    const SOURCE: &str = "--X:var(--Case";
    let report = parse_style_attribute(SOURCE);
    assert!(!report.is_clean());
    assert_eq!(report.syntax().len(), 1);
    let diagnostics = report.diagnostics().to_vec();
    assert_eq!(
        validate_style_attribute(SOURCE).unwrap_err().diagnostics(),
        diagnostics.as_slice()
    );
    let originals = vec![
        checked("color", "purple", CssImportance::Normal),
        report.syntax()[0].clone(),
    ];
    let list = CssDeclarationList::try_new(originals.clone()).unwrap();
    const EXPECTED: &str = "color: purple; --X: var(--Case);";
    assert_eq!(list.to_specified_css().unwrap(), EXPECTED);
    let CssComponentValueRef::Function(function) = list[1].value_components().items()[0].view()
    else {
        panic!("original var function");
    };
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("retained implicit closing origin");
    };
    assert_eq!(opening.source().as_str(), SOURCE);
    assert_eq!(opening.span().start().byte_offset().value(), 4);
    assert_eq!(opening.span().end().byte_offset().value(), 8);
    assert_eq!(at.span().start().byte_offset().value(), 14);
    assert_eq!(at.span().end().byte_offset().value(), 14);
    assert!(at.source().same_snapshot(opening.source()));
    unchanged(&list, &originals);
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics(), diagnostics.as_slice());
    clean_round_trip(EXPECTED, 2);
    unchanged(&list, &originals);
}

#[test]
fn later_real_provider_failure_returns_neither_an_admitted_list_nor_partial_text() {
    let valid = checked("color", "purple", CssImportance::Normal);
    let unsupported = checked(
        "rotate",
        "calc(.00000001) 1 0 30deg",
        CssImportance::Important,
    );
    let retained = vec![valid.clone(), unsupported.clone()];
    let values = unsupported.value_components().clone();
    // Ownership consumes each submitted Vec; retry uses independently retained handles.
    for _ in 0..2 {
        assert_eq!(
            CssDeclarationList::try_new(retained.clone())
                .unwrap_err()
                .kind(),
            Kind::UnrepresentableValue
        );
        assert_eq!(
            CssDeclarationList::try_new_with_limits(
                retained.clone(),
                Limits::new(1000, 1000, 1000)
            )
            .unwrap_err()
            .kind(),
            Kind::UnrepresentableValue
        );
        assert!(retained[0].same_occurrence(&valid));
        assert!(retained[1].same_occurrence(&unsupported));
        assert_eq!(retained[1].value_components(), &values);
        assert_eq!(valid.to_specified_css().unwrap(), "color: purple;");
        assert_eq!(
            unsupported.to_specified_css().unwrap_err().kind(),
            Kind::UnrepresentableValue
        );
    }
    // Parsing admits authored grammar; it does not promise every provider can emit it.
    let report = parse_style_attribute("color:purple;rotate:calc(.00000001) 1 0 30deg!important");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let parsed_originals = report.syntax().as_slice().to_vec();
    for _ in 0..2 {
        assert_eq!(
            report.syntax().to_specified_css().unwrap_err().kind(),
            Kind::UnrepresentableValue
        );
        assert_eq!(
            report
                .syntax()
                .to_specified_css_with_limits(Limits::new(1000, 1000, 1000))
                .unwrap_err()
                .kind(),
            Kind::UnrepresentableValue
        );
        unchanged(report.syntax(), &parsed_originals);
    }
    let retry = CssDeclarationList::try_new(vec![valid.clone()]).unwrap();
    assert_eq!(retry.to_specified_css().unwrap(), "color: purple;");
    assert!(retry[0].same_occurrence(&valid));
    unchanged(report.syntax(), &parsed_originals);
}
