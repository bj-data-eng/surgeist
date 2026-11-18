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
use crate::component::{CssComponentValue, CssComponentValueLimits, CssComponentValues};
use crate::declaration::CssImportance;
use crate::parser::bounded_execution::work;
use crate::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTextTarget};
use crate::source::{CssDecodedStylesheetInput, CssUtf16InputLimits, CssUtf16Resource};
use crate::{CssParser, CssParserContext};
use std::sync::Arc;

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
fn a_host_cap_refuses_block_contents_with_its_limit_and_total() {
    // `color:red` builds 3 documents; one earlier call has charged 1.
    let caps = cap(CssWorkFields {
        Document: 3,
        ..CssWorkFields::splat(u64::MAX)
    });
    let (ledger, host) = host(caps, ProfileMode::Off);
    let parser = CssParser::default();
    parser.accounted(&host).parse_sheet("").unwrap();
    let stop = parser
        .accounted(&host)
        .parse_block_contents("color:red")
        .unwrap_err();
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(refusal.change(), CssChange::Work(CssWork::Document, 3));
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (3, 1, 3)
    );
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

fn utf16(text: &str) -> Arc<Vec<u16>> {
    Arc::new(text.encode_utf16().collect())
}

fn never() -> Result<(), &'static str> {
    Ok(())
}

fn work_limit(max_work: usize) -> CssUtf16InputLimits {
    CssUtf16InputLimits {
        max_work,
        ..CssUtf16InputLimits::default()
    }
}

/// The preparation steps of a UTF-16 sheet: the smallest work limit under
/// which the plain front admits it, since the limit counts every step.
fn preparation_steps(text: &str) -> u64 {
    let admitted = |max_work| {
        CssParser::default()
            .parse_sheet_utf16(utf16(text), work_limit(max_work), never)
            .is_ok()
    };
    let smallest = (0..100_000).find(|&max_work| admitted(max_work)).unwrap();
    u64::try_from(smallest).unwrap()
}

fn steps(ledger: &CssLedger) -> u64 {
    ledger
        .usage(ledger.root())
        .unwrap()
        .work(CssWork::PreparationStep)
}

#[test]
fn a_utf16_sheet_charges_its_units_its_preparation_steps_and_its_documents() {
    let text = "a{color:red}";
    let (ledger, host) = open_host();
    let report = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16(text), CssUtf16InputLimits::default(), never)
        .unwrap()
        .unwrap();
    let plain = CssParser::default()
        .parse_sheet_utf16(utf16(text), CssUtf16InputLimits::default(), never)
        .unwrap();
    assert_eq!(report, plain);
    // The documents of the same text as a UTF-8 sheet.
    assert_eq!(charged(&ledger), [12, 2, 6, 15]);
    assert_eq!(steps(&ledger), preparation_steps(text));
    assert!(steps(&ledger) > 12, "more than a step per unit");
}

#[test]
fn utf16_block_contents_charge_their_units_and_the_documents_of_block_contents() {
    let text = "color:red";
    let (ledger, host) = open_host();
    let report = CssParser::default()
        .accounted(&host)
        .parse_block_contents_utf16(utf16(text), CssUtf16InputLimits::default(), never)
        .unwrap()
        .unwrap();
    let plain = CssParser::default()
        .parse_block_contents_utf16(utf16(text), CssUtf16InputLimits::default(), never)
        .unwrap();
    assert_eq!(report.report(), plain.report());
    assert_eq!(
        report.report(),
        &CssParser::default().parse_block_contents(text)
    );
    assert_eq!(charged(&ledger), [9, 3, 7, 21]);
    assert!(steps(&ledger) > 9);
}

#[test]
fn a_utf16_call_runs_in_an_input_phase_that_contains_its_parse_phase() {
    for rules in [1, 100] {
        let source = sheet(rules);
        let mode = ProfileMode::Summary { timing: false };
        let (ledger, host) = host(Limits::unlimited(), mode);
        CssParser::default()
            .accounted(&host)
            .parse_sheet_utf16(utf16(&source), CssUtf16InputLimits::default(), never)
            .unwrap()
            .unwrap();
        for front in CssFront::ALL {
            let scope = report(&ledger, host.scope(front));
            let expected = match front {
                CssFront::Parse => (1, 3, 1),
                CssFront::Input => (1, 2, 1),
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
fn a_refused_preparation_is_returned_inside_ok_and_charged_the_steps_it_took() {
    let text = "a{color:red}";
    let needed = usize::try_from(preparation_steps(text)).unwrap();
    for max_work in [0, 3, needed - 1] {
        let (ledger, host) = open_host();
        let refused = CssParser::default()
            .accounted(&host)
            .parse_sheet_utf16(utf16(text), work_limit(max_work), never)
            .unwrap()
            .unwrap_err();
        assert_eq!(refused.exhausted_limit(), Some(CssUtf16Resource::Work));
        let plain = CssParser::default()
            .parse_sheet_utf16(utf16(text), work_limit(max_work), never)
            .unwrap_err();
        assert_eq!(refused, plain);
        // The step that crossed the limit is counted.
        assert_eq!(steps(&ledger), u64::try_from(max_work + 1).unwrap());
        assert_eq!(charged(&ledger)[0], 12);
    }
}

#[test]
fn a_preparation_refused_before_the_grammar_builds_no_document() {
    let (ledger, host) = open_host();
    let refused = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16("a{color:red}"), work_limit(3), never)
        .unwrap()
        .unwrap_err();
    assert_eq!(refused.exhausted_limit(), Some(CssUtf16Resource::Work));
    assert_eq!(charged(&ledger), [12, 0, 0, 0]);
}

/// The last step follows the grammar, so a limit one short of the steps a
/// call needs refuses it after its documents are built.
#[test]
fn a_preparation_refused_at_its_last_step_is_charged_the_documents_it_built() {
    let text = "a{color:red}";
    let needed = preparation_steps(text);
    let (ledger, host) = open_host();
    let refused = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(
            utf16(text),
            work_limit(usize::try_from(needed).unwrap() - 1),
            never,
        )
        .unwrap()
        .unwrap_err();
    assert_eq!(refused.exhausted_limit(), Some(CssUtf16Resource::Work));
    assert_eq!(charged(&ledger), [12, 2, 6, 15]);
    assert_eq!(steps(&ledger), needed);
}

/// The first poll precedes every step and each step polls once, so the poll
/// of the last step is one more than the steps a call needs.
#[test]
fn a_call_interrupted_at_its_last_poll_is_charged_the_documents_it_built() {
    let text = "a{color:red}";
    let needed = preparation_steps(text);
    let last_poll = usize::try_from(needed).unwrap() + 1;
    let (ledger, host) = open_host();
    let mut polls = 0_usize;
    let interrupted = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16(text), CssUtf16InputLimits::default(), || {
            polls += 1;
            if polls == last_poll {
                Err("stop")
            } else {
                Ok(())
            }
        })
        .unwrap()
        .unwrap_err();
    assert_eq!(interrupted.into_interruption(), Ok("stop"));
    assert_eq!(charged(&ledger), [12, 2, 6, 15]);
    assert_eq!(steps(&ledger), needed - 1);
}

#[test]
fn a_host_cap_on_parse_work_replaces_the_result_of_a_utf16_call() {
    // `a{color:red}` builds 6 components.
    let caps = cap(CssWorkFields {
        Component: 5,
        ..CssWorkFields::splat(u64::MAX)
    });
    let (ledger, host) = host(caps, ProfileMode::Off);
    let stop = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16("a{color:red}"), CssUtf16InputLimits::default(), never)
        .unwrap_err();
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(refusal.change(), CssChange::Work(CssWork::Component, 6));
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (5, 0, 6)
    );
    // The parse charge is refused first, so the input batch is not made.
    assert_eq!(charged(&ledger), [0, 0, 0, 0]);
    assert_eq!(steps(&ledger), 0);
}

#[test]
fn a_utf16_sheet_too_deep_for_the_callers_stack_charges_what_the_parser_thread_built() {
    let source = format!("{}a{{color:red}}{}", "@media x{".repeat(40), "}".repeat(40));
    let (_, built, _) = work::measure(|| {
        let _ = CssParser::default().parse_sheet(&source);
    });
    let (ledger, host) = open_host();
    let report = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16(&source), CssUtf16InputLimits::default(), never)
        .unwrap()
        .unwrap();
    assert_eq!(report, CssParser::default().parse_sheet(&source));
    let [input, _, components, _] = charged(&ledger);
    assert_eq!(input, u64::try_from(source.len()).unwrap());
    assert!(built > 0);
    assert_eq!(components, u64::try_from(built).unwrap());
    assert!(!crate::parse_work::is_collecting());
}

/// The document of a `unicode-range` value is built outside the sites the
/// crate's node counter sees, so it shows as the difference between the two:
/// `U+26` is one token.
#[test]
fn a_unicode_range_descriptor_charges_its_own_document() {
    let parser = CssParser::default();
    let source = "font-family:x;unicode-range:U+26";
    let (_, built, _) = work::measure(|| {
        let _ = parser.parse_font_face_block_contents(source);
    });
    let (ledger, host) = open_host();
    let report = parser
        .accounted(&host)
        .parse_font_face_block_contents(source)
        .unwrap();
    assert_eq!(report, parser.parse_font_face_block_contents(source));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [_, _, components, _] = charged(&ledger);
    assert_eq!(components, u64::try_from(built).unwrap() + 1);
}

#[test]
fn an_input_longer_than_its_unit_limit_is_refused_before_any_step() {
    let limits = CssUtf16InputLimits {
        max_input_units: 11,
        ..CssUtf16InputLimits::default()
    };
    let (ledger, host) = open_host();
    let refused = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16("a{color:red}"), limits, never)
        .unwrap()
        .unwrap_err();
    assert_eq!(
        refused.exhausted_limit(),
        Some(CssUtf16Resource::InputUnits)
    );
    assert_eq!(charged(&ledger), [12, 0, 0, 0]);
    assert_eq!(steps(&ledger), 0);
}

#[test]
fn an_interrupted_call_returns_the_interruption_and_charges_the_steps_before_it() {
    // The first poll precedes every step; the fourth interrupts the third step.
    for (failing, expected) in [(1_usize, 0_u64), (4, 2)] {
        let (ledger, host) = open_host();
        let mut polls = 0_usize;
        let interrupted = CssParser::default()
            .accounted(&host)
            .parse_sheet_utf16(
                utf16("a{color:red}"),
                CssUtf16InputLimits::default(),
                || {
                    polls += 1;
                    if polls == failing {
                        Err("stop")
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap()
            .unwrap_err();
        assert_eq!(interrupted.into_interruption(), Ok("stop"));
        assert_eq!(steps(&ledger), expected, "poll {failing}");
        assert_eq!(charged(&ledger), [12, 0, 0, 0]);
    }
}

#[test]
fn a_host_cap_on_preparation_steps_replaces_the_result_of_a_utf16_call() {
    let text = "a{color:red}";
    let needed = preparation_steps(text);
    let caps = cap(CssWorkFields {
        PreparationStep: needed - 1,
        ..CssWorkFields::splat(u64::MAX)
    });
    let (ledger, host) = host(caps, ProfileMode::Off);
    let stop = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16(text), CssUtf16InputLimits::default(), never)
        .unwrap_err();
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(
        refusal.change(),
        CssChange::Work(CssWork::PreparationStep, needed)
    );
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (needed - 1, 0, needed)
    );
    // The input batch is refused whole; the parse work was charged before it.
    assert_eq!(charged(&ledger), [0, 2, 6, 15]);
    assert_eq!(steps(&ledger), 0);
}

#[test]
fn a_property_value_charges_its_structure_and_its_value_document() {
    // `calc(` opens one component holding `100%`, ` `, `-`, ` `, `12px`: six
    // components and 17 bytes, in the structure document and in the value
    // document.
    let source = "calc(100% - 12px)";
    let target =
        || CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width));
    let (ledger, host) = open_host();
    let report = CssParser::default()
        .accounted(&host)
        .parse_property_value(source, target(), CssImportance::Normal)
        .unwrap();
    assert_eq!(
        report,
        CssParser::default().parse_property_value(source, target(), CssImportance::Normal)
    );
    assert!(report.is_clean());
    assert_eq!(charged(&ledger), [17, 2, 12, 34]);
}

#[test]
fn the_component_value_fronts_charge_the_one_document_of_their_text() {
    let parser = CssParser::default();

    // `a`: one component, one byte.
    let (ledger, host) = open_host();
    let value = parser.accounted(&host).parse_component_value("a").unwrap();
    assert_eq!(value, CssComponentValue::parse(&parser, "a"));
    assert_eq!(charged(&ledger), [1, 1, 1, 1]);

    // `a b`: two idents and the whitespace between them.
    let (ledger, host) = open_host();
    let values = parser
        .accounted(&host)
        .parse_component_values("a b")
        .unwrap();
    assert_eq!(values, CssComponentValues::parse(&parser, "a b"));
    assert!(values.is_ok());
    assert_eq!(charged(&ledger), [3, 1, 3, 3]);

    // `a, b`: two idents, the comma and the whitespace.
    let (ledger, host) = open_host();
    let list = parser
        .accounted(&host)
        .parse_comma_separated_component_values("a, b")
        .unwrap();
    assert_eq!(
        list,
        CssComponentValues::parse_comma_separated(&parser, "a, b")
    );
    assert_eq!(charged(&ledger), [4, 1, 4, 4]);
}

#[test]
fn a_refused_component_value_list_is_returned_inside_ok_and_charged_its_bytes() {
    // Three components against a limit of two.
    let limits = CssComponentValueLimits::try_new(256, 2, 4096).unwrap();
    let parser = CssParser::new(CssParserContext::default(), limits);
    let (ledger, host) = open_host();
    let refused = parser
        .accounted(&host)
        .parse_component_values("a b")
        .unwrap();
    assert!(refused.is_err());
    assert_eq!(refused, CssComponentValues::parse(&parser, "a b"));
    assert_eq!(charged(&ledger)[0], 3);
}

#[test]
fn page_and_font_face_contents_charge_the_nodes_of_the_documents_they_build() {
    let parser = CssParser::default();
    let page = "margin:1px;size:a4";
    let font_face = "font-family:x;font-weight:400";

    let (_, built, _) = work::measure(|| {
        let _ = parser.parse_page_block_contents(page);
    });
    let (ledger, host) = open_host();
    let report = parser
        .accounted(&host)
        .parse_page_block_contents(page)
        .unwrap();
    assert_eq!(report, parser.parse_page_block_contents(page));
    assert!(report.syntax().is_some());
    let [input, documents, components, _] = charged(&ledger);
    assert_eq!(input, u64::try_from(page.len()).unwrap());
    assert!(documents >= 2, "{documents}");
    assert_eq!(components, u64::try_from(built).unwrap());

    let (_, built, _) = work::measure(|| {
        let _ = parser.parse_font_face_block_contents(font_face);
    });
    let (ledger, host) = open_host();
    let report = parser
        .accounted(&host)
        .parse_font_face_block_contents(font_face)
        .unwrap();
    assert_eq!(report, parser.parse_font_face_block_contents(font_face));
    assert!(report.syntax().is_some());
    let [input, documents, components, _] = charged(&ledger);
    assert_eq!(input, u64::try_from(font_face.len()).unwrap());
    assert!(documents >= 2, "{documents}");
    assert_eq!(components, u64::try_from(built).unwrap());
}

#[test]
fn deep_fragments_charge_what_the_parser_thread_built() {
    // Seventy nested functions are past the 63 levels a fragment parses on
    // the caller's stack.
    let source = format!("{}1px{}", "calc(".repeat(70), ")".repeat(70));
    let parser = CssParser::default();
    let (_, built, _) = work::measure(|| {
        let _ = CssComponentValue::parse(&parser, &source);
    });
    let (ledger, host) = open_host();
    let value = parser
        .accounted(&host)
        .parse_component_value(&source)
        .unwrap();
    assert_eq!(value, CssComponentValue::parse(&parser, &source));
    let [input, documents, components, _] = charged(&ledger);
    assert_eq!(input, u64::try_from(source.len()).unwrap());
    assert!(documents >= 1);
    assert!(built > 0);
    assert_eq!(components, u64::try_from(built).unwrap());
    assert!(!crate::parse_work::is_collecting());
}
