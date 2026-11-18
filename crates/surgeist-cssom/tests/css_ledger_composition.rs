//! Every CSS front charged through one host handle.
//!
//! The independent figure for each front is the same call made alone on a
//! fresh handle: composed, each scope holds the sum of what its fronts charge
//! alone, and no front charges a scope that is not its own. The ledger calls
//! follow the accounting contract: one batch per charged scope and one phase
//! per outermost call, and for a CSSOM store one of each per operation.

use std::convert::Infallible;
use std::sync::Arc;

use surgeist_css::CssParser;
use surgeist_css::accounting::{
    CssFront, CssHostScope, CssLedger, CssLimits, CssResource, CssResourceTable, CssWork,
    CssWorkFields, CssWorkTable,
};
use surgeist_css::color::{
    CssAuthoredColorOrdinaryLimits, CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest,
    CssColorInput, CssColorOrdinaryLimits, CssColorPropertyFacts, CssColorSchemeFact,
    CssDeviceColorFact,
};
use surgeist_css::normalize::CssNormalizationLimits;
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::rule::CssSheet;
use surgeist_css::serialization::{
    CssRuleGraphInput, CssRuleSerializationLimits, CssSerializationLimits,
};
use surgeist_css::source::{CssSourceRetention, CssUtf16InputLimits};
use surgeist_cssom::{
    CssomContext, CssomEditGuard, CssomError, CssomInputRole, CssomInputVersion, CssomLimits,
    CssomRuleFormatLimits, CssomSheetInputs, CssomStore,
};
use surgeist_ledger::{Dimension, Limits, Operation, ProfileMode, ScopeReport};

const SHEET: &str = "@media screen { .a { color: red } .b { margin: 1px 2px } } .c { color: rgb(10 20 30) } .d { width: 1px }";

fn host(caps: CssLimits) -> (CssLedger, CssHostScope) {
    let mode = ProfileMode::Summary { timing: false };
    let operation = Operation::with_profile("host", caps, 1 + CssFront::ALL.len(), mode)
        .unwrap()
        .share();
    let host = CssHostScope::open(operation.clone(), operation.root()).unwrap();
    (operation, host)
}

fn open_host() -> (CssLedger, CssHostScope) {
    host(Limits::unlimited())
}

fn facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(1, 1),
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Light,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )
}

fn sheet() -> CssSheet {
    CssParser::default().parse_sheet(SHEET).into_parts().0
}

fn utf16(text: &str) -> Arc<Vec<u16>> {
    Arc::new(text.encode_utf16().collect())
}

fn never() -> Result<(), Infallible> {
    Ok(())
}

/// One front: what it does through a handle, and the scopes it charges.
struct Front {
    name: &'static str,
    run: fn(&CssHostScope),
    /// The outermost calls `run` makes.
    calls: u64,
    /// The scopes `run` charges, and the batches each takes per call.
    charges: &'static [(CssFront, u64)],
    /// The batches beyond those, such as the gauge releases of a store.
    extra_batches: u64,
}

fn parse(host: &CssHostScope) {
    let report = CssParser::default()
        .accounted(host)
        .parse_sheet(SHEET)
        .unwrap();
    assert!(report.diagnostics().is_empty());
}

fn parse_utf16(host: &CssHostScope) {
    CssParser::default()
        .accounted(host)
        .parse_sheet_utf16(utf16(SHEET), CssUtf16InputLimits::default(), never)
        .unwrap()
        .unwrap();
}

fn serialize(host: &CssHostScope) {
    let sheet = sheet();
    let limits = CssSerializationLimits::new(usize::MAX, usize::MAX, usize::MAX);
    sheet
        .accounted(host)
        .serialize_cssom(limits)
        .unwrap()
        .unwrap();
}

fn serialize_graph(host: &CssHostScope) {
    let sheet = sheet();
    let limits = CssRuleSerializationLimits {
        values: CssSerializationLimits::new(usize::MAX, usize::MAX, usize::MAX),
        max_depth: 64,
    };
    CssRuleGraphInput::from_rule(&sheet.rules()[0])
        .accounted(host)
        .serialize_specified(limits)
        .unwrap()
        .unwrap();
}

fn normalize(host: &CssHostScope) {
    let sheet = sheet();
    sheet
        .accounted(host)
        .normalize(CssNormalizationLimits::default())
        .unwrap()
        .unwrap();
}

fn used_colour(host: &CssHostScope) {
    let facts = facts();
    CssColorInput::Utf8(b"rgb(10 20 30)")
        .accounted(host)
        .parse_used_color(
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
            CssColorOrdinaryLimits::default(),
        )
        .unwrap()
        .unwrap();
}

fn authored_colour(host: &CssHostScope) {
    let facts = facts();
    let report = CssParser::default().parse_block_contents("color: rgb(10 20 30)");
    let Some(CssKnownPropertyValueRef::Color(color)) = report.syntax()[0]
        .known()
        .and_then(|known| known.property_value())
    else {
        panic!("a colour property value");
    };
    color
        .value()
        .accounted(host)
        .resolve(
            &facts,
            facts.identity(),
            CssAuthoredColorOrdinaryLimits::default(),
        )
        .unwrap()
        .unwrap();
}

fn retention(host: &CssHostScope) {
    let sheet = sheet();
    let mut sources = CssSourceRetention::new(usize::MAX, usize::MAX, 256);
    sources
        .charge_walk(host, |sources| sources.visit(&sheet))
        .unwrap()
        .unwrap();
}

fn version(name: &str) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Document,
        identity: name.into(),
        revision: 1,
    }
}

/// Six store operations: the constructor, a creation and its commit, an edit
/// and its commit, and a rule formatted through its handle; dropping the store
/// releases its gauges in one more batch.
fn cssom(host: &CssHostScope) {
    let mut store = CssomStore::with_host(
        CssomLimits::default(),
        CssomContext::default(),
        host.clone(),
    )
    .unwrap();
    let guard = |store: &CssomStore| CssomEditGuard::from_snapshot(&store.snapshot());
    let mut edit = store.batch(guard(&store)).unwrap();
    let inputs = CssomSheetInputs::constructed(version("document"), "document".into(), None);
    let ticket = edit.create_parsed_sheet(SHEET, inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let mut edit = store.batch(guard(&store)).unwrap();
    edit.set_disabled(&sheet, true).unwrap();
    store.commit(edit).unwrap();
    let rule = store
        .css_style_sheet(&sheet)
        .unwrap()
        .css_rules(&store)
        .unwrap()
        .item(&store, 0)
        .unwrap()
        .unwrap();
    let text = rule
        .css_text(&store, &CssomRuleFormatLimits::default())
        .unwrap();
    assert!(text.starts_with("@media screen"), "{text}");
}

const FRONTS: [Front; 9] = [
    Front {
        name: "parse_sheet",
        run: parse,
        calls: 1,
        charges: &[(CssFront::Parse, 1), (CssFront::Input, 1)],
        extra_batches: 0,
    },
    Front {
        name: "parse_sheet_utf16",
        run: parse_utf16,
        calls: 1,
        charges: &[(CssFront::Parse, 1), (CssFront::Input, 1)],
        extra_batches: 0,
    },
    Front {
        name: "serialize_cssom",
        run: serialize,
        calls: 1,
        charges: &[(CssFront::Serialization, 1)],
        extra_batches: 0,
    },
    Front {
        name: "rule graph serialize_specified",
        run: serialize_graph,
        calls: 1,
        charges: &[(CssFront::Serialization, 1)],
        extra_batches: 0,
    },
    Front {
        name: "normalize",
        run: normalize,
        calls: 1,
        charges: &[(CssFront::Normalization, 1)],
        extra_batches: 0,
    },
    Front {
        name: "parse_used_color",
        run: used_colour,
        calls: 1,
        charges: &[(CssFront::Color, 1)],
        extra_batches: 0,
    },
    Front {
        name: "CssColor::resolve",
        run: authored_colour,
        calls: 1,
        charges: &[(CssFront::Color, 1)],
        extra_batches: 0,
    },
    Front {
        name: "charge_walk",
        run: retention,
        calls: 1,
        charges: &[(CssFront::SourceRetention, 1)],
        extra_batches: 0,
    },
    Front {
        name: "CSSOM store",
        run: cssom,
        calls: 6,
        charges: &[(CssFront::Cssom, 1)],
        extra_batches: 1,
    },
];

fn report(
    ledger: &CssLedger,
    host: &CssHostScope,
    front: CssFront,
) -> ScopeReport<CssWork, CssResource> {
    let scope = host.scope(front);
    ledger
        .snapshot()
        .unwrap()
        .unwrap()
        .scopes
        .into_iter()
        .find(|entry| entry.scope == scope)
        .unwrap()
}

/// Every work dimension and gauge a scope holds, and its accepted batches.
fn usage(ledger: &CssLedger, host: &CssHostScope, front: CssFront) -> (Vec<u64>, Vec<u64>, u64) {
    let scope = report(ledger, host, front);
    (
        CssWork::ALL
            .iter()
            .map(|work| scope.usage.work(*work))
            .collect(),
        CssResource::ALL
            .iter()
            .map(|resource| scope.usage.live(*resource))
            .collect(),
        scope.accepted_batches,
    )
}

#[test]
fn one_handle_holds_each_front_once_in_its_own_scope() {
    // Each front alone on a fresh handle.
    let mut expected: Vec<(Vec<u64>, Vec<u64>, u64)> = CssFront::ALL
        .iter()
        .map(|_| {
            (
                vec![0; CssWork::ALL.len()],
                vec![0; CssResource::ALL.len()],
                0,
            )
        })
        .collect();
    for front in &FRONTS {
        let (ledger, host) = open_host();
        (front.run)(&host);
        let mut batches = 0;
        let mut phases = 0;
        for (index, scope) in CssFront::ALL.iter().enumerate() {
            let (work, gauges, accepted) = usage(&ledger, &host, *scope);
            let charged = front.charges.iter().find(|(charged, _)| charged == scope);
            match charged {
                Some((_, per_call)) => {
                    assert!(
                        work.iter().any(|value| *value > 0),
                        "{}: {scope:?}",
                        front.name
                    );
                    for (sum, value) in expected[index].0.iter_mut().zip(&work) {
                        *sum += value;
                    }
                    for (sum, value) in expected[index].1.iter_mut().zip(&gauges) {
                        *sum += value;
                    }
                    expected[index].2 += accepted;
                    batches += accepted;
                    assert!(
                        accepted >= per_call * front.calls,
                        "{}: {scope:?}",
                        front.name
                    );
                }
                None => {
                    assert!(
                        work.iter().all(|value| *value == 0) && accepted == 0,
                        "{} charged {scope:?}",
                        front.name
                    );
                }
            }
            phases += report(&ledger, &host, *scope).phases;
        }
        let per_call: u64 = front.charges.iter().map(|(_, batches)| batches).sum();
        assert_eq!(
            batches,
            per_call * front.calls + front.extra_batches,
            "{}: ledger batches",
            front.name
        );
        let utf16_input_phase = u64::from(front.name == "parse_sheet_utf16");
        assert_eq!(
            phases,
            front.calls + utf16_input_phase,
            "{}: phases",
            front.name
        );
    }
    // All of them through one handle.
    let (ledger, host) = open_host();
    for front in &FRONTS {
        (front.run)(&host);
    }
    for (index, scope) in CssFront::ALL.iter().enumerate() {
        assert_eq!(usage(&ledger, &host, *scope), expected[index], "{scope:?}");
    }
    // The root and one scope per front.
    let scopes = ledger.snapshot().unwrap().unwrap().scopes.len();
    assert_eq!(scopes, 1 + CssFront::ALL.len());
}

#[test]
fn a_host_cap_refuses_the_call_that_crosses_it_with_the_host_figures() {
    let documents = |run: fn(&CssHostScope)| {
        let (ledger, host) = open_host();
        run(&host);
        ledger.usage(ledger.root()).unwrap().work(CssWork::Document)
    };
    let (first, second) = (documents(parse), documents(parse_utf16));
    assert!(first > 0 && second > 0);
    let caps = CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            Document: first + second - 1,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    );
    let (ledger, host) = host(caps);
    parse(&host);
    let before = [CssFront::Parse, CssFront::Input].map(|front| usage(&ledger, &host, front));
    let stop = CssParser::default()
        .accounted(&host)
        .parse_sheet_utf16(utf16(SHEET), CssUtf16InputLimits::default(), never)
        .unwrap_err();
    let refusal = stop.host_refusal().expect("the root's cap refuses");
    assert_eq!(refusal.scope(), ledger.root());
    assert_eq!(
        (refusal.limit(), refusal.total(), refusal.amount()),
        (first + second - 1, first, second)
    );
    // The refused batch changed nothing.
    assert_eq!(
        [CssFront::Parse, CssFront::Input].map(|front| usage(&ledger, &host, front)),
        before
    );
    // A CSSOM store below the same host is refused the same way.
    let caps = CssLimits::new(
        CssWorkTable::new(CssWorkFields {
            CensusRun: 0,
            ..CssWorkFields::splat(u64::MAX)
        }),
        CssResourceTable::splat(u64::MAX),
    );
    let (_, host) = self::host(caps);
    let refused = CssomStore::with_host(CssomLimits::default(), CssomContext::default(), host);
    let Err(CssomError::Accounting(stop)) = refused else {
        panic!("the host refuses the store's first census");
    };
    assert_eq!(stop.host_refusal().unwrap().limit(), 0);
}
