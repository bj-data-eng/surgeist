#![forbid(unsafe_code)]

//! Fonts 4 (2026-09-07) §§4.9.1, 6.9.1 and 6.9.2, with the selected exact
//! integer/cardinality policy. Literal payload/origin assertions are primary.

use surgeist_css::{
    CssBlockFragment, CssErrorCode, CssEscapeError, CssFontDisplay, CssFontFeatureValueBlock,
    CssFontFeatureValueDefinition, CssFontFeatureValueKind as Kind,
    CssFontFeatureValuesItem as Item, CssParseReport, CssParsedOrigin, CssRecoveryAction, CssRule,
    CssSourcePosition, ErrorKind, parse_font_feature_value_block, parse_font_feature_values_block,
    parse_sheet,
};

type OuterReport = CssParseReport<Option<CssBlockFragment<Vec<Item>>>>;
type SubsidiaryReport = CssParseReport<Option<CssBlockFragment<CssFontFeatureValueBlock>>>;

const KINDS: [Kind; 7] = [
    Kind::Stylistic,
    Kind::HistoricalForms,
    Kind::Styleset,
    Kind::CharacterVariant,
    Kind::Swash,
    Kind::Ornaments,
    Kind::Annotation,
];

fn outer(source: &str) -> OuterReport {
    parse_font_feature_values_block(source)
}
fn subsidiary(source: &str, kind: Kind) -> SubsidiaryReport {
    parse_font_feature_value_block(source, kind)
}

fn block(item: &Item) -> &CssFontFeatureValueBlock {
    let Item::Block(block) = item else {
        panic!("actual subsidiary block")
    };
    block
}

fn display(item: &Item, expected: CssFontDisplay) {
    let Item::FontDisplay(display) = item else {
        panic!("actual outer descriptor")
    };
    assert_eq!(display.value(), expected);
}

fn definition(actual: &CssFontFeatureValueDefinition, name: &str, digits: &[&str]) {
    assert_eq!(actual.name().as_str(), name);
    assert_eq!(
        actual
            .indexes()
            .iter()
            .map(|n| n.as_decimal_str())
            .collect::<Vec<_>>(),
        digits
    );
}

fn position(actual: CssSourcePosition, byte: usize, line: u32, column: u32) {
    assert_eq!(actual.byte_offset().value(), byte);
    assert_eq!(actual.line().value(), line);
    assert_eq!(actual.column().value(), column);
}

fn origin(
    actual: &CssParsedOrigin,
    source: &str,
    start: (usize, u32, u32),
    end: (usize, u32, u32),
) {
    assert_eq!(actual.source().as_str(), source);
    position(actual.span().start(), start.0, start.1, start.2);
    position(actual.span().end(), end.0, end.1, end.2);
}

fn validation<T: Clone + PartialEq + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    clean: bool,
) {
    assert_eq!(report.is_clean(), clean, "{report:?}");
    assert_eq!(report.diagnostics().is_empty(), clean);
    match report.clone().into_validation_result() {
        Ok(syntax) => {
            assert!(clean);
            assert_eq!(&syntax, report.syntax());
        }
        Err(failure) => {
            assert!(!clean);
            assert_eq!(failure.diagnostics(), report.diagnostics());
        }
    }
}

fn rejected<T: Clone + PartialEq + std::fmt::Debug>(
    report: CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
) {
    assert!(report.syntax().is_none(), "{source:?}: {report:?}");
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    validation(&report, false);
}

fn real_index_sources(block: &CssFontFeatureValueBlock, carrier: &CssParsedOrigin) {
    for definition in block.definitions() {
        assert!(definition.position().is_some());
        for index in definition.indexes() {
            let actual = index.origin().expect("actual Number token origin");
            assert!(carrier.source().same_snapshot(actual.source()));
            assert_eq!(actual.source().as_str(), carrier.source().as_str());
        }
    }
}

#[test]
fn mixed_outer_body_retains_display_and_all_seven_subsidiary_payloads_in_order() {
    let source = "{font-display:swap;@stylistic{first:1}@historical-forms{old:1 2 3}@styleset{set:0 100 101}@character-variant{cv:0 101}@swash{sw:2}@ornaments{orn:3}@annotation{ann:4}font-display:optional}";
    let report = outer(source);
    validation(&report, true);
    let actual = report.syntax().as_ref().unwrap();
    assert_eq!(actual.body().len(), 9);
    display(&actual.body()[0], CssFontDisplay::Swap);
    display(&actual.body()[8], CssFontDisplay::Optional);
    let expected: &[(Kind, &str, &[&str])] = &[
        (Kind::Stylistic, "first", &["1"]),
        (Kind::HistoricalForms, "old", &["1", "2", "3"]),
        (Kind::Styleset, "set", &["0", "100", "101"]),
        (Kind::CharacterVariant, "cv", &["0", "101"]),
        (Kind::Swash, "sw", &["2"]),
        (Kind::Ornaments, "orn", &["3"]),
        (Kind::Annotation, "ann", &["4"]),
    ];
    for (item, (kind, name, digits)) in actual.body()[1..8].iter().zip(expected) {
        let child = block(item);
        assert_eq!(child.kind(), *kind);
        let [first] = child.definitions() else {
            panic!("one real definition")
        };
        definition(first, name, digits);
        assert!(
            child.position().is_some(),
            "this nested child has a real at-keyword"
        );
        real_index_sources(child, actual.origin());
    }
    assert_eq!(actual.origin().source().as_str(), source);
    assert_eq!(actual.origin().span().start().byte_offset().value(), 0);
    assert_eq!(
        actual.origin().span().end().byte_offset().value(),
        source.len()
    );
}

#[test]
fn duplicate_displays_blocks_and_friendly_names_remain_authored_sequence() {
    let report = outer(
        "{font-display:block;@swash{First:1;First:2}font-display:swap;@swash{Last:3}font-display:block}",
    );
    validation(&report, true);
    let items = report.syntax().as_ref().unwrap().body();
    assert_eq!(items.len(), 5);
    display(&items[0], CssFontDisplay::Block);
    display(&items[2], CssFontDisplay::Swap);
    display(&items[4], CssFontDisplay::Block);
    let first = block(&items[1]);
    assert_eq!(first.kind(), Kind::Swash);
    assert_eq!(first.definitions().len(), 2);
    definition(&first.definitions()[0], "First", &["1"]);
    definition(&first.definitions()[1], "First", &["2"]);
    let last = block(&items[3]);
    assert_eq!(last.kind(), Kind::Swash);
    definition(&last.definitions()[0], "Last", &["3"]);
}

#[test]
fn every_display_keyword_remains_an_outer_descriptor_without_a_family_prelude() {
    for (source, expected) in [
        ("{font-display:auto}", CssFontDisplay::Auto),
        ("{font-display:block}", CssFontDisplay::Block),
        ("{font-display:sWaP}", CssFontDisplay::Swap),
        ("{font-display:fallback}", CssFontDisplay::Fallback),
        ("{font-display:optional}", CssFontDisplay::Optional),
        (r"{font-display:s\77 ap}", CssFontDisplay::Swap),
    ] {
        let report = outer(source);
        validation(&report, true);
        let [item] = report.syntax().as_ref().unwrap().body().as_slice() else {
            panic!("one display")
        };
        display(item, expected);
        let Item::FontDisplay(display) = item else {
            unreachable!()
        };
        position(display.position().unwrap(), 1, 0, 1);
    }
}

#[test]
fn every_supplied_kind_accepts_empty_and_duplicate_named_definitions_without_an_at_keyword() {
    for kind in KINDS {
        let report = subsidiary("{}", kind);
        validation(&report, true);
        let actual = report.syntax().as_ref().unwrap();
        assert_eq!(actual.body().kind(), kind);
        assert!(actual.body().definitions().is_empty());
        assert!(actual.body().position().is_none());
        origin(actual.origin(), "{}", (0, 0, 0), (2, 0, 2));

        let source = "{first:1;first:2}";
        let report = subsidiary(source, kind);
        validation(&report, true);
        let actual = report.syntax().as_ref().unwrap();
        assert_eq!(actual.body().kind(), kind);
        assert!(actual.body().position().is_none());
        let [one, two] = actual.body().definitions() else {
            panic!("two duplicate definitions")
        };
        definition(one, "first", &["1"]);
        definition(two, "first", &["2"]);
        position(one.position().unwrap(), 1, 0, 1);
        position(two.position().unwrap(), 9, 0, 9);
        real_index_sources(actual.body(), actual.origin());
    }
}

#[test]
fn friendly_identifier_case_keywords_and_escaped_punctuation_are_retained() {
    let source = r"{Fancy:1;fancy:2;Fancy:3;inherit:4;a\+b:5}";
    let report = subsidiary(source, Kind::Swash);
    validation(&report, true);
    let actual = report.syntax().as_ref().unwrap();
    assert!(actual.body().position().is_none());
    assert_eq!(actual.body().definitions().len(), 5);
    for (actual, (name, digits)) in actual.body().definitions().iter().zip([
        ("Fancy", "1"),
        ("fancy", "2"),
        ("Fancy", "3"),
        ("inherit", "4"),
        ("a+b", "5"),
    ]) {
        definition(actual, name, &[digits]);
    }
    real_index_sources(actual.body(), actual.origin());
}

#[test]
fn selected_cardinalities_recover_only_the_invalid_named_definition() {
    for kind in [Kind::HistoricalForms, Kind::Styleset] {
        let report = subsidiary("{first:0 100 101;last:2}", kind);
        validation(&report, true);
        let definitions = report.syntax().as_ref().unwrap().body().definitions();
        assert_eq!(definitions.len(), 2);
        definition(&definitions[0], "first", &["0", "100", "101"]);
        definition(&definitions[1], "last", &["2"]);
    }
    let report = subsidiary("{first:0 101;last:2}", Kind::CharacterVariant);
    validation(&report, true);
    let definitions = report.syntax().as_ref().unwrap().body().definitions();
    definition(&definitions[0], "first", &["0", "101"]);
    definition(&definitions[1], "last", &["2"]);
    for (kind, source) in [
        (Kind::Stylistic, "{first:1 2;last:2}"),
        (Kind::Swash, "{first:1 2;last:2}"),
        (Kind::Ornaments, "{first:1 2;last:2}"),
        (Kind::Annotation, "{first:1 2;last:2}"),
        (Kind::CharacterVariant, "{first:1 2 3;last:2}"),
    ] {
        let report = subsidiary(source, kind);
        validation(&report, false);
        let actual = report.syntax().as_ref().unwrap();
        assert_eq!(actual.body().kind(), kind);
        assert!(actual.body().position().is_none());
        let [last] = actual.body().definitions() else {
            panic!("only valid later definition")
        };
        definition(last, "last", &["2"]);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
    }
}

#[test]
fn exact_indexes_preserve_number_origins_and_normalize_above_u64_for_every_kind() {
    let source = " /*é*/\r\n{💡:+0002;Other:-0000;Huge:18446744073709551616} /*💡*/ ";
    for kind in KINDS {
        let report = subsidiary(source, kind);
        validation(&report, true);
        let actual = report.syntax().as_ref().unwrap();
        assert!(actual.body().position().is_none());
        origin(actual.origin(), source, (9, 1, 0), (59, 1, 48));
        let [light, other, huge] = actual.body().definitions() else {
            panic!("three exact definitions")
        };
        definition(light, "💡", &["2"]);
        definition(other, "Other", &["0"]);
        definition(huge, "Huge", &["18446744073709551616"]);
        position(light.position().unwrap(), 10, 1, 1);
        position(other.position().unwrap(), 21, 1, 10);
        position(huge.position().unwrap(), 33, 1, 22);
        origin(
            light.indexes()[0].origin().unwrap(),
            source,
            (15, 1, 4),
            (20, 1, 9),
        );
        origin(
            other.indexes()[0].origin().unwrap(),
            source,
            (27, 1, 16),
            (32, 1, 21),
        );
        origin(
            huge.indexes()[0].origin().unwrap(),
            source,
            (38, 1, 27),
            (58, 1, 47),
        );
        assert_eq!(&source[15..20], "+0002");
        assert_eq!(&source[27..32], "-0000");
        assert_eq!(&source[38..58], "18446744073709551616");
        assert_eq!(light.indexes()[0].to_u32(), Some(2));
        assert_eq!(other.indexes()[0].to_u32(), Some(0));
        assert_eq!(huge.indexes()[0].to_u32(), None);
        real_index_sources(actual.body(), actual.origin());
    }
}

#[test]
fn mixed_unicode_body_retains_real_descriptor_at_keyword_name_and_number_positions() {
    let source = " /*é*/\r\n{font-display:sWaP;@swash{💡:+0002}} /*💡*/ ";
    let report = outer(source);
    validation(&report, true);
    let actual = report.syntax().as_ref().unwrap();
    origin(actual.origin(), source, (9, 1, 0), (47, 1, 36));
    let [Item::FontDisplay(display), Item::Block(child)] = actual.body().as_slice() else {
        panic!("mixed original body")
    };
    assert_eq!(display.value(), CssFontDisplay::Swap);
    position(display.position().unwrap(), 10, 1, 1);
    assert_eq!(child.kind(), Kind::Swash);
    position(child.position().unwrap(), 28, 1, 19);
    let [named] = child.definitions() else {
        panic!("one definition")
    };
    definition(named, "💡", &["2"]);
    position(named.position().unwrap(), 35, 1, 26);
    origin(
        named.indexes()[0].origin().unwrap(),
        source,
        (40, 1, 29),
        (45, 1, 34),
    );
    assert_eq!(&source[40..45], "+0002");
    real_index_sources(child, actual.origin());
    // Equal source text is value equality, while separate parses own separate snapshots.
    let second = outer(source);
    validation(&second, true);
    assert_eq!(report.syntax(), second.syntax());
    assert!(
        !actual
            .origin()
            .source()
            .same_snapshot(second.syntax().as_ref().unwrap().origin().source())
    );
}

#[test]
fn clean_empty_recovered_empty_and_outer_invalid_reports_are_distinct() {
    for source in ["{}", "{;}", "{/*comment*/ ; }"] {
        let report = outer(source);
        validation(&report, true);
        assert!(report.syntax().as_ref().unwrap().body().is_empty());
        for kind in KINDS {
            let report = subsidiary(source, kind);
            validation(&report, true);
            assert!(
                report
                    .syntax()
                    .as_ref()
                    .unwrap()
                    .body()
                    .definitions()
                    .is_empty()
            );
            assert!(
                report
                    .syntax()
                    .as_ref()
                    .unwrap()
                    .body()
                    .position()
                    .is_none()
            );
        }
    }
    let report = outer("{font-display:invalid}");
    validation(&report, false);
    assert!(report.syntax().as_ref().unwrap().body().is_empty());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
    let report = subsidiary("{first:1.0}", Kind::Swash);
    validation(&report, false);
    assert!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .body()
            .definitions()
            .is_empty()
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
    for source in [
        "",
        " \t",
        "/**/",
        "[]",
        "()",
        "font-display:swap",
        "first:1",
        "@swash{}",
        "{} {}",
        "{}x",
        "{};",
        "{})",
        "{}]",
        "{}}",
    ] {
        rejected(outer(source), source);
        for kind in KINDS {
            rejected(subsidiary(source, kind), source);
        }
    }
}

#[test]
fn trailing_nontrivia_rejects_the_whole_body_at_original_tail_token() {
    let source = "{font-display:swap} extra";
    let report = outer(source);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    position(diagnostic.error().position(), 20, 0, 20);
    rejected(report, source);
    let source = "{Good:1} extra";
    let report = subsidiary(source, Kind::Swash);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap();
    position(diagnostic.error().position(), 9, 0, 9);
    rejected(report, source);
}

#[test]
fn invalid_outer_members_preserve_later_displays_and_subsidiary_blocks() {
    let source = "{font-display:swap;@unknown{lost:2}@swash extra{lost:3}font-display:inherit;bogus:1;@swash{last:3}font-display:optional}";
    let report = outer(source);
    validation(&report, false);
    let actual = report.syntax().as_ref().unwrap();
    let [first, child, last] = actual.body().as_slice() else {
        panic!("admitted later siblings")
    };
    display(first, CssFontDisplay::Swap);
    display(last, CssFontDisplay::Optional);
    assert_eq!(block(child).kind(), Kind::Swash);
    definition(&block(child).definitions()[0], "last", &["3"]);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropAtRule)
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RejectInput)
    );
}

#[test]
fn subsidiary_value_failures_and_nested_rules_recover_at_named_units() {
    for kind in KINDS {
        for value in [
            "", "-1", "1.0", "1e0", "1px", "1%", "\"1\"", "1,2", "var(--x)", "env(x)", "calc(1)",
        ] {
            let source = format!("{{bad:{value};last:2}}");
            let report = subsidiary(&source, kind);
            validation(&report, false);
            let actual = report.syntax().as_ref().unwrap();
            let [last] = actual.body().definitions() else {
                panic!("valid last definition")
            };
            definition(last, "last", &["2"]);
            let diagnostic = report
                .diagnostics()
                .iter()
                .find(|d| d.action() == CssRecoveryAction::DropDescriptor)
                .unwrap();
            let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
                panic!("real named descriptor grammar")
            };
            assert_eq!(detail.descriptor().as_str(), "bad");
        }
    }
    let source = "{@swash{lost:1}last:2}";
    let report = subsidiary(source, Kind::Annotation);
    validation(&report, false);
    let [last] = report.syntax().as_ref().unwrap().body().definitions() else {
        panic!("at-rule does not consume later definition")
    };
    definition(last, "last", &["2"]);
    position(last.position().unwrap(), 15, 0, 15);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::DropAtRule)
        .unwrap();
    position(diagnostic.span().start(), 1, 0, 1);
    position(diagnostic.span().end(), 15, 0, 15);
}

#[test]
fn importance_is_invalid_in_real_named_body_declarations_and_keeps_later_members() {
    let report = outer("{font-display:swap!important;font-display:block}");
    validation(&report, false);
    let [last] = report.syntax().as_ref().unwrap().body().as_slice() else {
        panic!("later display retained")
    };
    display(last, CssFontDisplay::Block);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::DropDescriptor)
        .unwrap();
    position(diagnostic.error().position(), 18, 0, 18);
    let ErrorKind::InvalidDeclarationAnnotation(detail) = diagnostic.error().kind() else {
        panic!("actual named outer annotation")
    };
    assert_eq!(detail.encountered().authored(), "!");

    let report = subsidiary("{Good:1!important;last:2}", Kind::Swash);
    validation(&report, false);
    let [last] = report.syntax().as_ref().unwrap().body().definitions() else {
        panic!("later named definition retained")
    };
    definition(last, "last", &["2"]);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::DropDescriptor)
        .unwrap();
    position(diagnostic.error().position(), 7, 0, 7);
    let ErrorKind::InvalidDeclarationAnnotation(detail) = diagnostic.error().kind() else {
        panic!("actual friendly-name annotation")
    };
    assert_eq!(detail.encountered().authored(), "!");
}

fn closure<T: Clone + PartialEq + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    eof: usize,
) {
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .unwrap();
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    position(diagnostic.error().position(), eof, 0, eof as u32);
    position(diagnostic.span().start(), eof, 0, eof as u32);
    position(diagnostic.span().end(), eof, 0, eof as u32);
    validation(report, false);
}

#[test]
fn implicit_outer_and_child_braces_close_at_actual_source_eof() {
    let report = outer("{");
    assert!(report.syntax().as_ref().unwrap().body().is_empty());
    origin(
        report.syntax().as_ref().unwrap().origin(),
        "{",
        (0, 0, 0),
        (1, 0, 1),
    );
    closure(&report, 1);
    for kind in KINDS {
        let report = subsidiary("{", kind);
        assert!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .body()
                .definitions()
                .is_empty()
        );
        assert!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .body()
                .position()
                .is_none()
        );
        closure(&report, 1);
    }
    let source = "{font-display:swap";
    let report = outer(source);
    let actual = report.syntax().as_ref().unwrap();
    display(&actual.body()[0], CssFontDisplay::Swap);
    origin(actual.origin(), source, (0, 0, 0), (18, 0, 18));
    closure(&report, 18);
    let source = "{@swash{Good:1";
    let report = outer(source);
    let actual = report.syntax().as_ref().unwrap();
    let child = block(&actual.body()[0]);
    position(child.position().unwrap(), 1, 0, 1);
    definition(&child.definitions()[0], "Good", &["1"]);
    position(child.definitions()[0].position().unwrap(), 8, 0, 8);
    origin(
        child.definitions()[0].indexes()[0].origin().unwrap(),
        source,
        (13, 0, 13),
        (14, 0, 14),
    );
    origin(actual.origin(), source, (0, 0, 0), (14, 0, 14));
    closure(&report, 14);
    let source = "{Good:1";
    let report = subsidiary(source, Kind::Swash);
    let actual = report.syntax().as_ref().unwrap();
    definition(&actual.body().definitions()[0], "Good", &["1"]);
    assert!(actual.body().position().is_none());
    origin(actual.origin(), source, (0, 0, 0), (7, 0, 7));
    origin(
        actual.body().definitions()[0].indexes()[0]
            .origin()
            .unwrap(),
        source,
        (6, 0, 6),
        (7, 0, 7),
    );
    closure(&report, 7);
}

fn comment<T: Clone + PartialEq + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    start: usize,
    eof: usize,
) {
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::IgnoreUnterminatedComment)
        .unwrap();
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
    position(diagnostic.error().position(), eof, 0, eof as u32);
    position(diagnostic.span().start(), start, 0, start as u32);
    position(diagnostic.span().end(), eof, 0, eof as u32);
    validation(report, false);
}

#[test]
fn lexical_comment_eof_preserves_explicit_or_implicit_brace_provenance() {
    let source = "{}/*x";
    let report = outer(source);
    origin(
        report.syntax().as_ref().unwrap().origin(),
        source,
        (0, 0, 0),
        (2, 0, 2),
    );
    assert!(report.syntax().as_ref().unwrap().body().is_empty());
    comment(&report, 2, 5);
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    let report = subsidiary(source, Kind::Swash);
    origin(
        report.syntax().as_ref().unwrap().origin(),
        source,
        (0, 0, 0),
        (2, 0, 2),
    );
    comment(&report, 2, 5);
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );

    let source = "{Good:1/*x";
    let report = subsidiary(source, Kind::Swash);
    let actual = report.syntax().as_ref().unwrap();
    definition(&actual.body().definitions()[0], "Good", &["1"]);
    origin(actual.origin(), source, (0, 0, 0), (10, 0, 10));
    origin(
        actual.body().definitions()[0].indexes()[0]
            .origin()
            .unwrap(),
        source,
        (6, 0, 6),
        (7, 0, 7),
    );
    comment(&report, 7, 10);
    closure(&report, 10);
    let source = "{font-display:swap/*x";
    let report = outer(source);
    display(
        &report.syntax().as_ref().unwrap().body()[0],
        CssFontDisplay::Swap,
    );
    origin(
        report.syntax().as_ref().unwrap().origin(),
        source,
        (0, 0, 0),
        (21, 0, 21),
    );
    comment(&report, 18, 21);
    closure(&report, 21);
}

#[test]
fn invalid_string_or_function_eof_keeps_only_genuine_admitted_body_units() {
    for source in ["{bad:\"1", "{bad:calc(1"] {
        let report = subsidiary(source, Kind::Swash);
        assert!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .body()
                .definitions()
                .is_empty()
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
        );
        origin(
            report.syntax().as_ref().unwrap().origin(),
            source,
            (0, 0, 0),
            (source.len(), 0, source.len() as u32),
        );
        closure(&report, source.len());
    }
    let report = subsidiary("{bad:\"1\n;last:2}", Kind::Swash);
    validation(&report, false);
    let [last] = report.syntax().as_ref().unwrap().body().definitions() else {
        panic!("later definition after bad String")
    };
    definition(last, "last", &["2"]);
    let report = outer("{font-display:\"swap\n;font-display:block}");
    validation(&report, false);
    let [last] = report.syntax().as_ref().unwrap().body().as_slice() else {
        panic!("later display after bad String")
    };
    display(last, CssFontDisplay::Block);
}

#[test]
fn lexical_escape_eof_diagnostics_use_original_offsets_after_valid_body_members() {
    let report = subsidiary("{Good:1;name\\", Kind::Swash);
    let [good] = report.syntax().as_ref().unwrap().body().definitions() else {
        panic!("earlier definition retained")
    };
    definition(good, "Good", &["1"]);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| {
            matches!(
                d.error().kind(),
                ErrorKind::EscapeParseError(CssEscapeError::EndOfInput)
            )
        })
        .unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::RecoverEscape);
    position(diagnostic.error().position(), 13, 0, 13);
    closure(&report, 13);

    let report = outer("{font-display:swap;name\\");
    let [first] = report.syntax().as_ref().unwrap().body().as_slice() else {
        panic!("earlier display retained")
    };
    display(first, CssFontDisplay::Swap);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| {
            matches!(
                d.error().kind(),
                ErrorKind::EscapeParseError(CssEscapeError::EndOfInput)
            )
        })
        .unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::RecoverEscape);
    position(diagnostic.error().position(), 24, 0, 24);
    closure(&report, 24);
}

fn resource<T: Clone + PartialEq + std::fmt::Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
    exceeded: bool,
    byte: usize,
    unit_start: usize,
    unit_end: usize,
) {
    assert!(
        report.syntax().is_some(),
        "native recovering body remains admitted: {report:?}"
    );
    origin(
        report.syntax().as_ref().unwrap().origin(),
        source,
        (0, 0, 0),
        (source.len(), 0, source.len() as u32),
    );
    if exceeded {
        let diagnostic = report
            .diagnostics()
            .iter()
            .find(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
            .unwrap();
        let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
            panic!("typed original-unit resource error")
        };
        assert_eq!(detail.limit(), 256);
        position(diagnostic.error().position(), byte, 0, byte as u32);
        position(diagnostic.span().start(), unit_start, 0, unit_start as u32);
        position(diagnostic.span().end(), unit_end, 0, unit_end as u32);
    } else {
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.error().code() == CssErrorCode::NestingLimit)
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
        );
    }
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RejectInput)
    );
    validation(report, false);
}

#[test]
fn outer_brace_counts_once_and_failed_definitions_preserve_later_siblings_at_depth_ceiling() {
    for (groups, exceeded) in [(255, false), (256, true)] {
        // One actual outer brace plus these functions reaches depth256/257.
        let source = format!(
            "{{bad:{}x{};last:2}}",
            "f(".repeat(groups),
            ")".repeat(groups)
        );
        let report = subsidiary(&source, Kind::Swash);
        let actual = report.syntax().as_ref().unwrap();
        let [last] = actual.body().definitions() else {
            panic!("later original sibling")
        };
        definition(last, "last", &["2"]);
        assert!(actual.body().position().is_none());
        position(
            last.position().unwrap(),
            7 + 3 * groups,
            0,
            (7 + 3 * groups) as u32,
        );
        let number = last.indexes()[0].origin().unwrap();
        assert!(actual.origin().source().same_snapshot(number.source()));
        assert_eq!(number.source().as_str(), source);
        resource(&report, &source, exceeded, 515, 1, 7 + 3 * groups);

        let source = format!(
            "{{font-display:{}x{};font-display:swap}}",
            "f(".repeat(groups),
            ")".repeat(groups)
        );
        let report = outer(&source);
        let [last] = report.syntax().as_ref().unwrap().body().as_slice() else {
            panic!("later original display")
        };
        display(last, CssFontDisplay::Swap);
        let Item::FontDisplay(last) = last else {
            unreachable!()
        };
        position(
            last.position().unwrap(),
            16 + 3 * groups,
            0,
            (16 + 3 * groups) as u32,
        );
        resource(&report, &source, exceeded, 524, 1, 16 + 3 * groups);
    }
}

#[test]
fn unknown_child_resource_failure_uses_its_original_unit_and_keeps_later_outer_display() {
    for (groups, exceeded) in [(254, false), (255, true)] {
        // The actual outer and unknown-child braces both count, even for a dropped child.
        let source = format!(
            "{{@unknown{{bad:{}x{};}}font-display:swap}}",
            "f(".repeat(groups),
            ")".repeat(groups)
        );
        let report = outer(&source);
        let [last] = report.syntax().as_ref().unwrap().body().as_slice() else {
            panic!("only later real display")
        };
        display(last, CssFontDisplay::Swap);
        let Item::FontDisplay(last) = last else {
            unreachable!()
        };
        position(
            last.position().unwrap(),
            17 + 3 * groups,
            0,
            (17 + 3 * groups) as u32,
        );
        resource(&report, &source, exceeded, 522, 1, 17 + 3 * groups);
        if !exceeded {
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::DropAtRule)
            );
        }
    }
}

#[test]
fn defined_child_resource_failure_recovers_the_definition_and_preserves_child_and_outer_siblings() {
    for (groups, exceeded) in [(254, false), (255, true)] {
        let source = format!(
            "{{@swash{{bad:{}x{};last:2}}font-display:optional}}",
            "f(".repeat(groups),
            ")".repeat(groups)
        );
        let report = outer(&source);
        let actual = report.syntax().as_ref().unwrap();
        let [child, last_display] = actual.body().as_slice() else {
            panic!("retained child plus later display")
        };
        display(last_display, CssFontDisplay::Optional);
        let child = block(child);
        position(child.position().unwrap(), 1, 0, 1);
        let [last] = child.definitions() else {
            panic!("later child definition retained")
        };
        definition(last, "last", &["2"]);
        position(
            last.position().unwrap(),
            14 + 3 * groups,
            0,
            (14 + 3 * groups) as u32,
        );
        real_index_sources(child, actual.origin());
        resource(&report, &source, exceeded, 520, 8, 14 + 3 * groups);
    }
}

#[test]
fn genuine_rule_controls_confirm_provider_semantics_without_fabricating_fragment_input() {
    let source = "@font-feature-values Demo{font-display:swap;@styleset{First:0 100 101}font-display:optional;@swash{bad:1.0;Good:+0002}}";
    let real = parse_sheet(source);
    let [CssRule::FontFeatureValues(rule)] = real.syntax().rules() else {
        panic!("genuine existing rule")
    };
    assert_eq!(rule.families()[0].as_str(), "Demo");
    let [first, styleset, last, swash] = rule.items() else {
        panic!("literal mixed rule payload")
    };
    display(first, CssFontDisplay::Swap);
    display(last, CssFontDisplay::Optional);
    definition(
        &block(styleset).definitions()[0],
        "First",
        &["0", "100", "101"],
    );
    let [good] = block(swash).definitions() else {
        panic!("bad definition locally dropped")
    };
    definition(good, "Good", &["2"]);
    assert!(
        real.diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
    assert!(real.clone().into_validation_result().is_err());

    // Each direct input is a literal genuine block. No wrapper construction or offset adjustment.
    let direct = outer(
        "{font-display:swap;@styleset{First:0 100 101}font-display:optional;@swash{bad:1.0;Good:+0002}}",
    );
    validation(&direct, false);
    let [direct_first, direct_styleset, direct_last, direct_swash] =
        direct.syntax().as_ref().unwrap().body().as_slice()
    else {
        panic!("same literal authored sequence")
    };
    display(direct_first, CssFontDisplay::Swap);
    display(direct_last, CssFontDisplay::Optional);
    assert_eq!(
        block(direct_styleset).definitions()[0]
            .indexes()
            .iter()
            .map(|n| n.as_decimal_str())
            .collect::<Vec<_>>(),
        block(styleset).definitions()[0]
            .indexes()
            .iter()
            .map(|n| n.as_decimal_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        block(direct_swash).definitions()[0].name().as_str(),
        good.name().as_str()
    );
    let subsidiary = subsidiary("{Good:+0002}", Kind::Swash);
    validation(&subsidiary, true);
    let actual = subsidiary.syntax().as_ref().unwrap();
    assert!(actual.body().position().is_none());
    definition(&actual.body().definitions()[0], "Good", &["2"]);
    assert_eq!(
        actual.body().definitions()[0].indexes()[0].as_decimal_str(),
        good.indexes()[0].as_decimal_str()
    );
}
