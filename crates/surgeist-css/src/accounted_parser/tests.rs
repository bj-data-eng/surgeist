//! What the accounted parse fronts charge.
//!
//! The expected documents, components and spelling bytes of the small inputs
//! are derived by hand from what each front builds: a sheet builds one
//! document of its whole text for its structure and one document for the
//! value of each declaration it reads; block contents build one document of
//! the whole text under the component limits, one for the structure and one
//! per declaration value. A component is a token other than a matched closing
//! token; the spelling bytes of a document are the bytes of its tokens,
//! closing tokens included. For larger inputs the component sum is compared
//! with the crate's own count of the nodes its source documents hold.

use surgeist_css_parser::input::{DecodeHints, DecodeLimits};
use surgeist_ledger::{Limits, Operation, ProfileMode, Scope, ScopeReport};

use crate::accounting::{
    CssAccountingError, CssChange, CssFront, CssHostScope, CssLedger, CssLimits, CssResource,
    CssResourceTable, CssWork, CssWorkFields, CssWorkTable,
};
use crate::component::CssComponentValueLimits;
use crate::parser::bounded_execution::work;
use crate::source::CssDecodedStylesheetInput;
use crate::{CssParser, CssParserContext};

fn host(root: CssLimits, mode: ProfileMode) -> (CssLedger, CssHostScope) {
    let operation = Operation::with_profile("host", root, 1 + CssFront::ALL.len(), mode)
        .unwrap()
        .share();
    let host = CssHostScope::open(operation.clone(), operation.root()).unwrap();
    (operation, host)
}

fn open_host() -> (CssLedger, CssHostScope) {
    host(Limits::unlimited(), ProfileMode::Off)
}

/// The input units, documents, components and spelling bytes the root holds.
fn charged(ledger: &CssLedger) -> [u64; 4] {
    let usage = ledger.usage(ledger.root()).unwrap();
    [
        usage.work(CssWork::InputUnit),
        usage.work(CssWork::Document),
        usage.work(CssWork::Component),
        usage.work(CssWork::SpellingByte),
    ]
}

fn report(ledger: &CssLedger, scope: Scope) -> ScopeReport<CssWork, CssResource> {
    let snapshot = ledger.snapshot().unwrap().unwrap();
    snapshot
        .scopes
        .into_iter()
        .find(|entry| entry.scope == scope)
        .unwrap()
}

fn cap(work: CssWorkFields<u64>) -> CssLimits {
    CssLimits::new(CssWorkTable::new(work), CssResourceTable::splat(u64::MAX))
}

/// `rules` style rules of two declarations each.
fn sheet(rules: usize) -> String {
    (0..rules)
        .map(|rule| format!(".c{rule} {{ color: red; margin: {rule}px }}\n"))
        .collect()
}

#[test]
fn a_sheet_charges_its_bytes_and_the_documents_of_its_structure_and_values() {
    let cases: [(&str, [u64; 4]); 4] = [
        // No text: the structure document of nothing.
        ("", [0, 1, 0, 0]),
        // Structure: `a`, `{`, `color`, `:`, `red` are 5 components and with
        // `}` 12 bytes. Value `red`: 1 component, 3 bytes.
        ("a{color:red}", [12, 2, 6, 15]),
        // Structure: 9 components (the two whitespace runs inside the block,
        // one before it), 16 bytes. Value ` red `: 3 components, 5 bytes.
        ("a { color: red }", [16, 2, 12, 21]),
        // Structure: `a`, `{`, `color`, `:`, `red`, `;`, `width`, `:`, `1px`
        // are 9 components, 22 bytes. Values `red` and `1px`: 1 component and
        // 3 bytes each.
        ("a{color:red;width:1px}", [22, 3, 11, 28]),
    ];
    for (source, expected) in cases {
        let (ledger, host) = open_host();
        let report = CssParser::default()
            .accounted(&host)
            .parse_sheet(source)
            .unwrap();
        assert_eq!(
            report,
            CssParser::default().parse_sheet(source),
            "{source:?}"
        );
        assert_eq!(charged(&ledger), expected, "{source:?}");
    }
}

#[test]
fn block_contents_charge_the_limited_document_the_structure_and_each_value() {
    let cases: [(&str, [u64; 4]); 3] = [
        // The limited document and the structure document: `color`, `:`,
        // `red` are 3 components and 9 bytes each. Value `red`: 1 and 3.
        ("color:red", [9, 3, 7, 21]),
        // Two documents of 7 components and 19 bytes, and the values `red`
        // and `1px`.
        ("color:red;width:1px", [19, 4, 16, 44]),
        // A nested rule is no declaration, so no value is read: two documents
        // of 5 components and 12 bytes.
        ("a{color:red}", [12, 2, 10, 24]),
    ];
    for (source, expected) in cases {
        let (ledger, host) = open_host();
        let report = CssParser::default()
            .accounted(&host)
            .parse_block_contents(source)
            .unwrap();
        assert_eq!(
            report,
            CssParser::default().parse_block_contents(source),
            "{source:?}"
        );
        assert_eq!(charged(&ledger), expected, "{source:?}");
    }
}

#[test]
fn the_component_sum_is_the_node_count_of_the_documents_built() {
    for rules in [1, 7, 200] {
        let source = sheet(rules);
        let (_, built, _) = work::measure(|| {
            let _ = CssParser::default().parse_sheet(&source);
        });
        let (ledger, host) = open_host();
        CssParser::default()
            .accounted(&host)
            .parse_sheet(&source)
            .unwrap();
        let [input, documents, components, _] = charged(&ledger);
        assert_eq!(input, u64::try_from(source.len()).unwrap());
        // The structure, and the values of two declarations per rule.
        assert_eq!(documents, u64::try_from(1 + 2 * rules).unwrap());
        assert_eq!(components, u64::try_from(built).unwrap());
    }
}

#[test]
fn a_call_makes_the_same_ledger_calls_whatever_the_size_of_its_input() {
    for rules in [1, 100] {
        let source = sheet(rules);
        let mode = ProfileMode::Summary { timing: false };
        let (ledger, host) = host(Limits::unlimited(), mode);
        CssParser::default()
            .accounted(&host)
            .parse_sheet(&source)
            .unwrap();
        for front in CssFront::ALL {
            let scope = report(&ledger, host.scope(front));
            let expected = match front {
                CssFront::Parse => (1, 3, 1),
                CssFront::Input => (1, 1, 0),
                _ => (0, 0, 0),
            };
            assert_eq!(
                (scope.accepted_batches, scope.accepted_items, scope.phases),
                expected,
                "{front:?} with {rules} rules"
            );
        }
    }
}

#[test]
fn a_sheet_too_deep_for_the_callers_stack_charges_what_the_parser_thread_built() {
    // Forty nested blocks are past the 32 levels parsed on the caller's stack.
    let source = format!("{}a{{color:red}}{}", "@media x{".repeat(40), "}".repeat(40));
    let (_, built, _) = work::measure(|| {
        let _ = CssParser::default().parse_sheet(&source);
    });
    let (ledger, host) = open_host();
    let report = CssParser::default()
        .accounted(&host)
        .parse_sheet(&source)
        .unwrap();
    assert_eq!(report, CssParser::default().parse_sheet(&source));
    let [input, documents, components, _] = charged(&ledger);
    assert_eq!(input, u64::try_from(source.len()).unwrap());
    assert!(documents >= 2, "the structure and the value: {documents}");
    assert!(built > 0);
    assert_eq!(components, u64::try_from(built).unwrap());
    assert!(!crate::parse_work::is_collecting());
}

#[test]
fn decoded_bytes_charge_their_original_length() {
    // A byte order mark and `a{color:red}`: 15 original bytes, 12 of text.
    let bytes = b"\xEF\xBB\xBFa{color:red}";
    let input =
        CssDecodedStylesheetInput::decode(bytes, DecodeHints::default(), DecodeLimits::default())
            .unwrap();
    assert_eq!(input.source().as_str(), "a{color:red}");
    let (ledger, host) = open_host();
    let report = CssParser::default()
        .accounted(&host)
        .parse_sheet_bytes(&input)
        .unwrap();
    assert_eq!(report, CssParser::default().parse_sheet_bytes(&input));
    assert_eq!(charged(&ledger), [15, 2, 6, 15]);
}

#[test]
fn a_request_limit_refusal_is_the_plain_report_and_the_refused_document_adds_nothing() {
    // Seven components against a limit of six: the limited document is refused.
    let limits = CssComponentValueLimits::try_new(256, 6, 4096).unwrap();
    let parser = CssParser::new(CssParserContext::default(), limits);
    let source = "color:red;width:1px";
    let plain = parser.parse_block_contents(source);
    assert!(plain.syntax().is_empty());
    assert!(!plain.is_clean());
    let (_, built, _) = work::measure(|| {
        let _ = parser.parse_block_contents(source);
    });
    let (ledger, host) = open_host();
    let report = parser
        .accounted(&host)
        .parse_block_contents(source)
        .unwrap();
    assert_eq!(report, plain);
    let [input, _, components, _] = charged(&ledger);
    assert_eq!(input, 19);
    assert_eq!(components, u64::try_from(built).unwrap());
    // Less than the 16 components of the same text within its limits.
    assert!(components < 16, "{components}");
}

#[test]
fn a_host_cap_on_parse_work_replaces_the_report_and_charges_nothing() {
    // `a{color:red}` builds 6 components.
    let caps = cap(CssWorkFields {
        Component: 5,
        ..CssWorkFields::splat(u64::MAX)
    });
    let (ledger, host) = host(caps, ProfileMode::Off);
    let stop = CssParser::default()
        .accounted(&host)
        .parse_sheet("a{color:red}")
        .unwrap_err();
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(refusal.scope(), ledger.root());
    assert_eq!(refusal.change(), CssChange::Work(CssWork::Component, 6));
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (5, 0, 6)
    );
    assert_eq!(charged(&ledger), [0, 0, 0, 0]);

    // The cap admits a call that fits it.
    CssParser::default()
        .accounted(&host)
        .parse_sheet("")
        .unwrap();
    assert_eq!(charged(&ledger), [0, 1, 0, 0]);
}

#[test]
fn a_host_cap_on_input_units_refuses_after_the_parse_work_is_charged() {
    let caps = cap(CssWorkFields {
        InputUnit: 11,
        ..CssWorkFields::splat(u64::MAX)
    });
    let (ledger, host) = host(caps, ProfileMode::Off);
    let stop = CssParser::default()
        .accounted(&host)
        .parse_sheet("a{color:red}")
        .unwrap_err();
    assert_eq!(
        stop,
        CssAccountingError::Host(*stop.host_refusal().unwrap())
    );
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(refusal.change(), CssChange::Work(CssWork::InputUnit, 12));
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (11, 0, 12)
    );
    assert_eq!(charged(&ledger), [0, 2, 6, 15]);
}

#[test]
fn a_plain_front_after_an_accounted_call_charges_nothing() {
    let (ledger, host) = open_host();
    let parser = CssParser::default();
    parser.accounted(&host).parse_sheet("a{color:red}").unwrap();
    let after = charged(&ledger);
    assert!(!crate::parse_work::is_collecting());
    let _ = parser.parse_sheet("b{width:1px;height:2px}");
    let _ = parser.parse_block_contents("width:1px");
    assert_eq!(charged(&ledger), after);
    // Each accounted call charges its own documents.
    parser.accounted(&host).parse_sheet("a{color:red}").unwrap();
    assert_eq!(charged(&ledger), after.map(|amount| 2 * amount));
}
