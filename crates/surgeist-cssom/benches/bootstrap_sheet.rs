//! Release-mode cost of parsing one real stylesheet, adopting it into a CSSOM
//! store and editing the store that holds it.
//!
//! The stylesheet is read at run time from the path in
//! `SURGEIST_BENCH_STYLESHEET`; without it the bench prints how to run it and
//! exits. It was written for Bootstrap's `dist/css/bootstrap.css`, but any
//! stylesheet with at least one style rule inside a grouping rule works.
//!
//! The report has four parts.
//!
//! - Acceptance: the diagnostics of the parse, each with its byte span and the
//!   first bytes of the text it covers; the rules the committed snapshot holds,
//!   counted by kind through every rule list; and whether a store with the
//!   default `CssomLimits` accepts the sheet.
//! - Creation: the parse alone (`CssParser::parse_sheet`), the adoption of its
//!   report (`create_parsed_sheet_report` in a fresh batch) and the commit, each
//!   timed on its own, with the bytes that remain allocated once the store holds
//!   the committed sheet and the time to drop that store.
//! - Edits, on one store that holds the sheet: one declaration set through
//!   `CssStyleProperties::set_property`, one rule inserted into and deleted from
//!   the sheet's own list and the largest grouping rule's list, one selector set
//!   through `CssStyleRule::set_selector_text`, and the whole text replaced through
//!   `CssStyleSheet::replace_sync` with one rule changed. Each edit alternates
//!   between two states, so the sheet returns to the rules it started with. A
//!   one-rule control sheet is replaced alone in its own store, and beside the
//!   sheet before and after the whole-text replacements. A replaced sheet's old
//!   records stay in the store, detached, so the bytes the replacements add are
//!   reported too.
//! - Whole-text replacements on a store with the default `CssomLimits`, until a
//!   limit refuses one.
//!
//! Every timed call publishes one revision. The first call of every case is
//! reported apart from the rest, whose median and 95th percentile the table
//! gives. A global counting allocator supplies the memory figures; it adds a few
//! atomic additions to every allocation. The whole-text replacements of
//! Bootstrap hold about 1.8 GB by the end of the edits.
//!
//! Run with
//! `SURGEIST_BENCH_STYLESHEET=/path/to/bootstrap.css cargo bench --offline --locked -j 1 -p surgeist-cssom --bench bootstrap_sheet`.
//! `SURGEIST_BENCH_LOOP_CREATE=<seconds>` instead repeats creation for that long
//! and prints nothing else, for a sampling profiler to attach to.
//! `SURGEIST_BENCH_LOOP_CONTROL=<seconds>` likewise repeats the control
//! replacement beside the sheet after ten whole-text replacements.

use std::alloc::System;
use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::{CssParseReport, CssParser, rule::CssSheet};
use surgeist_cssom::{
    CssomAdmission, CssomAdmissionRequest, CssomAuthoredRule, CssomContext,
    CssomDeclarationRequestLimits, CssomDeclarationUsability, CssomEditGuard, CssomGroupPrelude,
    CssomInputRole, CssomInputVersion, CssomLimits, CssomRuleData, CssomRuleId, CssomRuleListId,
    CssomSheetId, CssomSheetInputs, CssomSnapshot, CssomStore,
};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// Timed calls per case after the first.
const CALLS: usize = 61;
/// Timed whole-text replacements after the first; each retains its
/// predecessor's records, about 55 MiB for Bootstrap.
const WHOLE_CALLS: usize = 31;
/// Timed creations after the first.
const CREATIONS: usize = 31;

/// Limits that admit any stylesheet this bench is given.
fn wide_limits() -> CssomLimits {
    CssomLimits {
        max_objects: 1 << 30,
        max_entries: 1 << 30,
        max_string_bytes: 1 << 34,
        max_input_bytes: 1 << 34,
        max_retained_source_bytes: 1 << 34,
        max_source_traversal_steps: 1 << 40,
        css: CssSerializationLimits::new(usize::MAX, usize::MAX, usize::MAX),
        ..CssomLimits::default()
    }
}

fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "document".to_owned(),
            revision: 1,
        },
        "document".into(),
        None,
    )
}

/// The parser of the stores here, whose default context selects the default
/// parser context; adoption refuses a report parsed in any other.
fn parser(_store: &CssomStore) -> CssParser {
    CssParser::default()
}

/// The bytes allocated and not freed over a region.
fn held(change: &stats_alloc::Stats) -> i64 {
    change.bytes_allocated as i64 + change.bytes_reallocated as i64
        - change.bytes_deallocated as i64
}

fn nanos(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1e9
}

fn milliseconds(nanos: f64) -> String {
    format!("{:.3}", nanos / 1e6)
}

/// The first call, the median and the 95th percentile of the rest, in ns.
struct Summary {
    first: f64,
    median: f64,
    p95: f64,
    calls: usize,
}

fn summary(first: f64, mut rest: Vec<f64>) -> Summary {
    rest.sort_by(f64::total_cmp);
    let p95 = (rest.len() * 95).div_ceil(100).max(1) - 1;
    Summary {
        first,
        median: rest[rest.len() / 2],
        p95: rest[p95],
        calls: rest.len(),
    }
}

/// Parses `text`, adopts it into a fresh batch of `store` and commits it,
/// returning the sheet and the three times in ns.
fn create_split(store: &mut CssomStore, text: &str) -> (CssomSheetId, [f64; 3]) {
    let parser = parser(store);
    let start = Instant::now();
    let report = parser.parse_sheet(black_box(text));
    let parsed = start.elapsed();
    let mut edit = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .expect("a batch");
    let start = Instant::now();
    let ticket = edit
        .create_parsed_sheet_report(report, inputs())
        .expect("the store adopts the sheet");
    let adopted = start.elapsed();
    let start = Instant::now();
    let commit = store.commit(edit).expect("a commit");
    let committed = start.elapsed();
    let sheet = commit.sheet(&ticket).expect("the sheet").clone();
    (sheet, [nanos(parsed), nanos(adopted), nanos(committed)])
}

fn create(store: &mut CssomStore, text: &str) -> CssomSheetId {
    create_split(store, text).0
}

/// The rules reachable from `list`, in document order, with their depth.
fn walk(
    snapshot: &CssomSnapshot,
    list: &CssomRuleListId,
    depth: usize,
    out: &mut Vec<(CssomRuleId, usize)>,
) {
    for id in snapshot.rules(list).expect("a rule list") {
        out.push((id.clone(), depth));
        if let Some(children) = snapshot.rule(id).expect("a rule").data().children() {
            walk(snapshot, children, depth + 1, out);
        }
    }
}

fn all_rules(snapshot: &CssomSnapshot, sheet: &CssomSheetId) -> Vec<(CssomRuleId, usize)> {
    let mut out = Vec::new();
    walk(
        snapshot,
        snapshot.sheet(sheet).expect("the sheet").rules(),
        0,
        &mut out,
    );
    out
}

/// The interface name of one retained rule.
fn kind(snapshot: &CssomSnapshot, id: &CssomRuleId) -> String {
    let rule = snapshot.rule(id).expect("a rule");
    match rule.data() {
        CssomRuleData::Style { .. } => "CSSStyleRule".into(),
        CssomRuleData::Group { prelude, .. } => match prelude {
            CssomGroupPrelude::Media(_) => "CSSMediaRule".into(),
            CssomGroupPrelude::Supports(_) => "CSSSupportsRule".into(),
            CssomGroupPrelude::Container(_) => "CSSContainerRule".into(),
            CssomGroupPrelude::Layer(_) => "CSSLayerBlockRule".into(),
            CssomGroupPrelude::Scope { .. } => "CSSScopeRule".into(),
            CssomGroupPrelude::When(_) => "CSSWhenRule".into(),
            CssomGroupPrelude::Else(_) => "CSSElseRule".into(),
        },
        CssomRuleData::NestedDeclarations { .. } => "CSSNestedDeclarations".into(),
        CssomRuleData::Page { .. } => "CSSPageRule".into(),
        CssomRuleData::Margin { .. } => "CSSMarginRule".into(),
        CssomRuleData::Keyframes { .. } => "CSSKeyframesRule".into(),
        CssomRuleData::Keyframe { .. } => "CSSKeyframeRule".into(),
        CssomRuleData::Import { .. } => "CSSImportRule".into(),
        CssomRuleData::CounterStyle { .. } => "CSSCounterStyleRule".into(),
        CssomRuleData::FontFace { .. } => "CSSFontFaceRule".into(),
        CssomRuleData::FontFeatureValues { .. } => "CSSFontFeatureValuesRule".into(),
        CssomRuleData::CustomMedia { .. } => "CSSCustomMediaRule".into(),
        CssomRuleData::NamedSupports { .. } => "CSSSupportsConditionRule".into(),
        CssomRuleData::SupportsTestChild(_) => "(supports test child)".into(),
        CssomRuleData::Leaf { current } => match current {
            CssomAuthoredRule::Rule(rule) => {
                let debug = format!("{rule:?}");
                let variant = debug.split('(').next().unwrap_or("?").to_owned();
                format!("leaf {variant}")
            }
            other => format!("leaf {:?}", std::mem::discriminant(other)),
        },
    }
}

/// The first `count` characters of `text` from byte `start`, on one line.
fn excerpt(text: &str, start: usize, count: usize) -> String {
    let start = (0..=start.min(text.len()))
        .rev()
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(0);
    text[start..]
        .chars()
        .take(count)
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect()
}

fn report_acceptance(text: &str, report: &CssParseReport<CssSheet>) {
    println!("## Acceptance");
    println!();
    println!(
        "input bytes: {}; top-level rules in the parse: {}; diagnostics: {}",
        text.len(),
        report.syntax().rules().len(),
        report.diagnostics().len()
    );
    for diagnostic in report.diagnostics() {
        let span = diagnostic.span();
        let start = span.start().byte_offset().value();
        println!(
            "- {:?} at bytes {start}..{} (line {}): {} | {}",
            diagnostic.action(),
            span.end().byte_offset().value(),
            span.start().line().value() + 1,
            diagnostic.error(),
            excerpt(text, start, 100),
        );
    }

    let mut store = CssomStore::new(wide_limits(), CssomContext::default()).expect("a store");
    let sheet = create(&mut store, text);
    let snapshot = store.snapshot();
    let rules = all_rules(&snapshot, &sheet);
    let mut kinds = BTreeMap::<String, usize>::new();
    for (id, _) in &rules {
        *kinds.entry(kind(&snapshot, id)).or_default() += 1;
    }
    let keyframes = kinds.get("CSSKeyframeRule").copied().unwrap_or(0);
    println!();
    println!(
        "retained rules through every list: {} ({} without CSSKeyframeRule); by kind:",
        rules.len(),
        rules.len() - keyframes
    );
    for (name, count) in &kinds {
        println!("- {name}: {count}");
    }
    println!(
        "sheet diagnostics retained: {}",
        snapshot
            .sheet(&sheet)
            .expect("the sheet")
            .diagnostics()
            .len()
    );

    let mut defaults =
        CssomStore::new(CssomLimits::default(), CssomContext::default()).expect("a store");
    let mut edit = defaults
        .batch(CssomEditGuard::from_snapshot(&defaults.snapshot()))
        .expect("a batch");
    let accepted = edit
        .create_parsed_sheet(text, inputs())
        .map_err(|error| format!("{error:?}"))
        .and_then(|_| {
            defaults
                .commit(edit)
                .map(drop)
                .map_err(|e| format!("{e:?}"))
        });
    println!(
        "default CssomLimits: {}",
        match accepted {
            Ok(()) => "accepted".to_owned(),
            Err(error) => format!("refused: {error}"),
        }
    );
    println!();
}

fn report_creation(text: &str) {
    let mut parse = Vec::new();
    let mut adopt = Vec::new();
    let mut commit = Vec::new();
    let mut total = Vec::new();
    let mut drops = Vec::new();
    let mut store_bytes = 0;
    let mut report_bytes = 0;
    for round in 0..=CREATIONS {
        let region = Region::new(GLOBAL);
        let mut store = CssomStore::new(wide_limits(), CssomContext::default()).expect("a store");
        let (sheet, [p, a, c]) = create_split(&mut store, text);
        black_box(&sheet);
        let change = region.change();
        if round == 0 {
            store_bytes = held(&change);
            let region = Region::new(GLOBAL);
            let report = parser(&store).parse_sheet(text);
            let change = region.change();
            report_bytes = held(&change);
            drop(report);
        }
        let start = Instant::now();
        drop(store);
        let dropped = nanos(start.elapsed());
        parse.push(p);
        adopt.push(a);
        commit.push(c);
        total.push(p + a + c);
        drops.push(dropped);
    }
    println!("## Creation ({} timed rounds after the first)", CREATIONS);
    println!();
    println!("| phase | first ms | median ms | p95 ms |");
    println!("| --- | ---: | ---: | ---: |");
    for (name, values) in [
        ("parse (`CssParser::parse_sheet`)", parse),
        ("adopt (`create_parsed_sheet_report`)", adopt),
        ("commit", commit),
        ("parse + adopt + commit", total),
        ("drop the store", drops),
    ] {
        let first = values[0];
        let s = summary(first, values[1..].to_vec());
        println!(
            "| {name} | {} | {} | {} |",
            milliseconds(s.first),
            milliseconds(s.median),
            milliseconds(s.p95)
        );
    }
    println!();
    println!(
        "bytes allocated and not freed while the store holds the committed sheet: {store_bytes} ({:.1} MiB, {:.1}x the input)",
        store_bytes as f64 / (1024.0 * 1024.0),
        store_bytes as f64 / text.len() as f64
    );
    println!(
        "bytes the parse report alone holds: {report_bytes} ({:.1} MiB)",
        report_bytes as f64 / (1024.0 * 1024.0)
    );
    println!();
}

fn admission(request: &CssomAdmissionRequest<'_>) -> CssomAdmission {
    let version = CssomInputVersion {
        role: CssomInputRole::Support,
        identity: "bench".into(),
        revision: 1,
    };
    let decisions = request
        .candidates()
        .iter()
        .map(|candidate| {
            candidate
                .decide(version.clone(), CssomDeclarationUsability::Usable)
                .expect("a decision")
        })
        .collect();
    let mut current = request
        .context()
        .inputs
        .iter()
        .map(|input| input.version.clone())
        .collect::<Vec<_>>();
    current.push(version);
    CssomAdmission::new(decisions, current)
}

/// Times `CALLS + 1` calls of `call`, which receives the call index.
fn timed(call: impl FnMut(usize)) -> Summary {
    timed_n(CALLS, call)
}

/// Times `calls + 1` calls of `call`, which receives the call index.
fn timed_n(calls: usize, mut call: impl FnMut(usize)) -> Summary {
    let mut times = Vec::with_capacity(calls + 1);
    for index in 0..=calls {
        let start = Instant::now();
        call(index);
        times.push(nanos(start.elapsed()));
    }
    summary(times[0], times[1..].to_vec())
}

/// Times `CALLS + 1` pairs of calls, returning the summaries of each half.
fn timed_pair(mut first: impl FnMut(), mut second: impl FnMut()) -> (Summary, Summary) {
    let mut a = Vec::with_capacity(CALLS + 1);
    let mut b = Vec::with_capacity(CALLS + 1);
    for _ in 0..=CALLS {
        let start = Instant::now();
        first();
        a.push(nanos(start.elapsed()));
        let start = Instant::now();
        second();
        b.push(nanos(start.elapsed()));
    }
    (
        summary(a[0], a[1..].to_vec()),
        summary(b[0], b[1..].to_vec()),
    )
}

/// The text with one declaration added to the style rule whose block holds
/// the first declaration ending after the middle of the text.
fn changed_text(text: &str) -> (String, usize) {
    let middle = text.len() / 2;
    let at = text[middle..]
        .find(";\n")
        .map(|offset| middle + offset + 2)
        .expect("a declaration after the middle");
    let mut changed = String::with_capacity(text.len() + 32);
    changed.push_str(&text[..at]);
    changed.push_str("  outline-offset: 1px;\n");
    changed.push_str(&text[at..]);
    (changed, at)
}

fn report_edits(text: &str) {
    let mut store = CssomStore::new(wide_limits(), CssomContext::default()).expect("a store");
    let sheet_id = create(&mut store, text);
    let control_id = create(&mut store, ".control { color: red }");
    let snapshot = store.snapshot();
    let rules = all_rules(&snapshot, &sheet_id);

    // The style rule in the middle of the style rules, which in Bootstrap is
    // inside a layer.
    let styles = rules
        .iter()
        .filter(|(id, _)| {
            matches!(
                snapshot.rule(id).unwrap().data(),
                CssomRuleData::Style { .. }
            )
        })
        .collect::<Vec<_>>();
    let (target_id, target_depth) = styles[styles.len() / 2].clone();
    // The grouping rule with the longest direct list.
    let (group_id, group_len) = rules
        .iter()
        .filter_map(|(id, _)| {
            let rule = snapshot.rule(id).unwrap();
            match rule.data() {
                CssomRuleData::Group { children, .. } => {
                    Some((id.clone(), snapshot.rules(children).unwrap().len()))
                }
                _ => None,
            }
        })
        .max_by_key(|(_, len)| *len)
        .expect("a grouping rule");
    let top_len = snapshot
        .rules(snapshot.sheet(&sheet_id).unwrap().rules())
        .unwrap()
        .len();
    drop(snapshot);

    let sheet = store.css_style_sheet(&sheet_id).expect("the sheet object");
    let control = store
        .css_style_sheet(&control_id)
        .expect("the control object");
    let target = store
        .css_style_rule(&target_id)
        .expect("the style rule object");
    let group = store
        .css_grouping_rule(&group_id)
        .expect("the grouping rule object");
    let selector = target
        .selector_text(&store, CssSerializationLimits::default())
        .expect("the selector");
    let style = target.style(&store).expect("the style object");
    let limits = CssomDeclarationRequestLimits::default();
    let cancel = AtomicBool::new(false);

    println!(
        "## Edits ({CALLS} timed calls after the first, {WHOLE_CALLS} for the whole-text replacement; each call publishes one revision)"
    );
    println!();
    println!(
        "edited style rule: `{}` at depth {target_depth}; largest grouping rule list: {group_len} rules (`{}`); sheet list: {top_len} rules",
        excerpt(&selector, 0, 80),
        kind(&store.snapshot(), &group_id),
    );
    println!();

    let mut rows: Vec<(String, Summary)> = Vec::new();

    let values = ["rgb(1, 2, 3)", "rgb(4, 5, 6)"];
    rows.push((
        "`style.setProperty('color', …)` + publish".into(),
        timed(|index| {
            let _ = black_box(
                style
                    .set_property(
                        &mut store,
                        "color",
                        values[index % 2],
                        Some(""),
                        limits,
                        |r| Ok(admission(r)),
                    )
                    .expect("set_property"),
            );
        }),
    ));

    let inserted = ".surgeist-bench-inserted { color: red }";
    let (insert, delete) = {
        let store = std::cell::RefCell::new(&mut store);
        timed_pair(
            || {
                let _ = black_box(
                    sheet
                        .insert_rule(&mut store.borrow_mut(), inserted, Some(0), &cancel)
                        .expect("insert_rule"),
                );
            },
            || {
                let _ = black_box(
                    sheet
                        .delete_rule(&mut store.borrow_mut(), 0)
                        .expect("delete_rule"),
                );
            },
        )
    };
    rows.push(("`sheet.insertRule(rule, 0)`".into(), insert));
    rows.push(("`sheet.deleteRule(0)`".into(), delete));

    let middle = group_len / 2;
    let (insert, delete) = {
        let store = std::cell::RefCell::new(&mut store);
        timed_pair(
            || {
                let _ = black_box(
                    group
                        .insert_rule(&mut store.borrow_mut(), inserted, Some(middle))
                        .expect("insert_rule"),
                );
            },
            || {
                let _ = black_box(
                    group
                        .delete_rule(&mut store.borrow_mut(), middle)
                        .expect("delete_rule"),
                );
            },
        )
    };
    rows.push((
        format!("`group.insertRule(rule, {middle})` in the largest list"),
        insert,
    ));
    rows.push((
        format!("`group.deleteRule({middle})` in the largest list"),
        delete,
    ));

    let selectors = [".surgeist-bench-selector", selector.as_str()];
    rows.push((
        "`rule.selectorText = …`".into(),
        timed(|index| {
            let _ = black_box(
                target
                    .set_selector_text(&mut store, selectors[index % 2])
                    .expect("set_selector_text"),
            );
        }),
    ));

    let controls = [".control { color: blue }", ".control { color: red }"];
    let mut alone = CssomStore::new(wide_limits(), CssomContext::default()).expect("a store");
    let alone_id = create(&mut alone, controls[1]);
    let alone_sheet = alone
        .css_style_sheet(&alone_id)
        .expect("the control object");
    rows.push((
        "control `replaceSync`, one-rule sheet alone in its store".into(),
        timed(|index| {
            let _ = black_box(
                alone_sheet
                    .replace_sync(&mut alone, controls[index % 2])
                    .expect("replace_sync"),
            );
        }),
    ));
    let replace_control = |store: &mut CssomStore| {
        timed(|index| {
            let _ = black_box(
                control
                    .replace_sync(store, controls[index % 2])
                    .expect("replace_sync"),
            );
        })
    };
    rows.push((
        "control `replaceSync`, one-rule sheet beside the sheet, before the whole-text replacements".into(),
        replace_control(&mut store),
    ));

    let (changed, at) = changed_text(text);
    let texts = [changed.as_str(), text];
    let region = Region::new(GLOBAL);
    let whole = timed_n(WHOLE_CALLS, |index| {
        let _ = black_box(
            sheet
                .replace_sync(&mut store, texts[index % 2])
                .expect("replace_sync"),
        );
    });
    let grown = held(&region.change());
    let replacements = whole.calls + 1;
    rows.push((
        "`sheet.replaceSync(text)` with one rule changed".into(),
        whole,
    ));
    rows.push((
        "control `replaceSync`, one-rule sheet beside the sheet, after the whole-text replacements"
            .into(),
        replace_control(&mut store),
    ));

    println!("| edit | first ms | median ms | p95 ms | calls |");
    println!("| --- | ---: | ---: | ---: | ---: |");
    for (name, s) in &rows {
        println!(
            "| {name} | {} | {} | {} | {} |",
            milliseconds(s.first),
            milliseconds(s.median),
            milliseconds(s.p95),
            s.calls
        );
    }
    println!();
    println!(
        "replaceSync change: `outline-offset: 1px;` added at byte {at}, after `{}`",
        excerpt(text, at.saturating_sub(60), 60)
    );
    println!(
        "bytes the store holds in addition after {replacements} whole-text replacements: {grown} ({:.1} MiB, {:.1} MiB per replacement)",
        grown as f64 / (1024.0 * 1024.0),
        grown as f64 / (1024.0 * 1024.0) / replacements as f64
    );
    let snapshot = store.snapshot();
    assert_eq!(
        all_rules(&snapshot, &sheet_id).len(),
        rules.len(),
        "the edits return to the sheet they started with"
    );
    println!();
}

/// Replaces the whole sheet on a store with the default limits until a limit
/// refuses, up to 200 replacements.
fn report_default_replacements(text: &str) {
    let (changed, _) = changed_text(text);
    let texts = [changed.as_str(), text];
    let mut store =
        CssomStore::new(CssomLimits::default(), CssomContext::default()).expect("a store");
    let id = create(&mut store, text);
    let sheet = store.css_style_sheet(&id).expect("the sheet object");
    let mut refusal = None;
    let mut done = 0;
    for index in 0..200 {
        match sheet.replace_sync(&mut store, texts[index % 2]) {
            Ok(_) => done += 1,
            Err(error) => {
                refusal = Some(format!("{error:?}"));
                break;
            }
        }
    }
    println!("## Whole-text replacements under the default CssomLimits");
    println!();
    match refusal {
        Some(error) => println!("{done} replacements succeeded; the next was refused: {error}"),
        None => println!("{done} replacements succeeded"),
    }
    println!();
}

fn main() {
    let Some(path) = std::env::var_os("SURGEIST_BENCH_STYLESHEET") else {
        println!(
            "set SURGEIST_BENCH_STYLESHEET to the path of a stylesheet, for example Bootstrap's dist/css/bootstrap.css"
        );
        return;
    };
    let text = std::fs::read_to_string(&path).expect("a readable UTF-8 stylesheet");

    let seconds = |name: &str| {
        std::env::var_os(name).map(|value| {
            let seconds: f64 = value
                .to_str()
                .and_then(|s| s.parse().ok())
                .expect("a number of seconds");
            Instant::now() + Duration::from_secs_f64(seconds)
        })
    };
    if let Some(end) = seconds("SURGEIST_BENCH_LOOP_CREATE") {
        while Instant::now() < end {
            let mut store =
                CssomStore::new(wide_limits(), CssomContext::default()).expect("a store");
            black_box(create(&mut store, &text));
        }
        return;
    }
    if let Some(end) = seconds("SURGEIST_BENCH_LOOP_CONTROL") {
        let mut store = CssomStore::new(wide_limits(), CssomContext::default()).expect("a store");
        let sheet = create(&mut store, &text);
        let sheet = store.css_style_sheet(&sheet).expect("the sheet object");
        let control = create(&mut store, ".control { color: red }");
        let control = store.css_style_sheet(&control).expect("the control object");
        let (changed, _) = changed_text(&text);
        for index in 0..10 {
            let source = if index % 2 == 0 { &changed } else { &text };
            let _ = sheet
                .replace_sync(&mut store, source)
                .expect("replace_sync");
        }
        let controls = [".control { color: blue }", ".control { color: red }"];
        let mut index = 0;
        while Instant::now() < end {
            let _ = black_box(
                control
                    .replace_sync(&mut store, controls[index % 2])
                    .expect("replace_sync"),
            );
            index += 1;
        }
        return;
    }

    let report = CssParser::default().parse_sheet(&text);
    report_acceptance(&text, &report);
    drop(report);
    report_creation(&text);
    report_edits(&text);
    report_default_replacements(&text);
}
