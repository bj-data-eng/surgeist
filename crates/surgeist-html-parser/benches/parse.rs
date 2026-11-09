//! Release-mode parse benchmark over five deterministic generated documents.
//!
//! Each round parses one document once with generous limits and no profiling.
//! The report lists each document's size and its `HtmlParseUsage` work and
//! resource totals beside the median, minimum and maximum parse time over the
//! rounds. Totals depend only on the document and the limits, so they identify
//! the accounted work a timing change is measured against. A second table
//! repeats the rounds with the parse's own ledger operation profiled in summary
//! mode without timing.
//!
//! Run with `cargo bench --offline --locked -j 1 -p surgeist-html-parser --bench parse`.

use std::{fmt::Write as _, hint::black_box, time::Instant};

#[path = "support/documents.rs"]
mod documents;

use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseAccounting, HtmlParseLimits, HtmlParseResource,
    HtmlParseUsage, HtmlParseWorkCategory, HtmlSource,
};
use surgeist_ledger::ProfileMode;

/// Measured rounds per document, after one unmeasured warm-up parse.
const ROUNDS: usize = 11;

const WORK: [(HtmlParseWorkCategory, &str); 8] = [
    (HtmlParseWorkCategory::InputUnitRead, "InputUnitRead"),
    (HtmlParseWorkCategory::InspectByte, "InspectByte"),
    (HtmlParseWorkCategory::CopyByte, "CopyByte"),
    (HtmlParseWorkCategory::StateStep, "StateStep"),
    (HtmlParseWorkCategory::RecordVisit, "RecordVisit"),
    (HtmlParseWorkCategory::RecordWrite, "RecordWrite"),
    (HtmlParseWorkCategory::ReserveByte, "ReserveByte"),
    (HtmlParseWorkCategory::ProviderCall, "ProviderCall"),
];

const RESOURCES: [(HtmlParseResource, &str); 20] = [
    (HtmlParseResource::SourceUnits, "SourceUnits"),
    (HtmlParseResource::BufferedUtf16Units, "BufferedUtf16Units"),
    (HtmlParseResource::TokenAttributes, "TokenAttributes"),
    (HtmlParseResource::AuxiliaryEntries, "AuxiliaryEntries"),
    (HtmlParseResource::ManagedNodes, "ManagedNodes"),
    (HtmlParseResource::ManagedEdges, "ManagedEdges"),
    (HtmlParseResource::ManagedAttributes, "ManagedAttributes"),
    (HtmlParseResource::ManagedUtf16Units, "ManagedUtf16Units"),
    (HtmlParseResource::SourceMapEntries, "SourceMapEntries"),
    (
        HtmlParseResource::SourceMapPayloadUnits,
        "SourceMapPayloadUnits",
    ),
    (HtmlParseResource::Diagnostics, "Diagnostics"),
    (
        HtmlParseResource::DiagnosticPayloadUnits,
        "DiagnosticPayloadUnits",
    ),
    (HtmlParseResource::ContextEffects, "ContextEffects"),
    (HtmlParseResource::OpenElements, "OpenElements"),
    (
        HtmlParseResource::ActiveFormattingEntries,
        "ActiveFormattingEntries",
    ),
    (HtmlParseResource::TreeDepth, "TreeDepth"),
    (
        HtmlParseResource::ManagedCapacityBytes,
        "ManagedCapacityBytes",
    ),
    (HtmlParseResource::PeakTransientBytes, "PeakTransientBytes"),
    (HtmlParseResource::AtomicLayoutBytes, "AtomicLayoutBytes"),
    (HtmlParseResource::WorkCredits, "WorkCredits"),
];

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(1 << 30)
        .buffered_utf16_units(1 << 30)
        .token_attributes(1 << 30)
        .auxiliary_entries(1 << 30)
        .active_formatting_entries(1 << 30)
        .managed_nodes(1 << 30)
        .managed_edges(1 << 30)
        .managed_attributes(1 << 30)
        .managed_utf16_units(1 << 30)
        .source_map_entries(1 << 30)
        .source_map_payload_units(1 << 30)
        .diagnostics(1 << 30)
        .diagnostic_payload_units(1 << 30)
        .context_effects(1 << 30)
        .open_elements(1 << 30)
        .tree_depth(1 << 30)
        .managed_capacity_bytes(1 << 40)
        .peak_transient_bytes(1 << 40)
        .atomic_layout_bytes(1 << 40)
        .work_credits(u64::MAX)
        .finish()
        .expect("benchmark limits are representable")
}

fn parse(source: &HtmlSource) -> HtmlParseUsage {
    let report = HtmlDocumentParseRequest::offline(source.clone(), limits())
        .parse()
        .unwrap_or_else(|failure| panic!("benchmark document failed to parse: {failure:?}"));
    let usage = report.usage().clone();
    drop(black_box(report));
    usage
}

fn timed(source: &HtmlSource, mode: Option<ProfileMode>) -> f64 {
    let start = Instant::now();
    let mut request = HtmlDocumentParseRequest::offline(black_box(source).clone(), limits());
    if let Some(mode) = mode {
        request = request.with_accounting(HtmlParseAccounting::profiled(mode).observing_phases());
    }
    let report = request
        .parse()
        .unwrap_or_else(|failure| panic!("benchmark document failed to parse: {failure:?}"));
    let elapsed = start.elapsed().as_secs_f64() * 1_000.0;
    drop(black_box(report));
    elapsed
}

fn main() {
    let documents: [(&str, String); 5] = [
        ("ordinary", documents::ordinary()),
        ("deeply-nested", documents::deeply_nested()),
        ("many-attributes", documents::many_attributes()),
        ("table-heavy", documents::table_heavy()),
        ("many-diagnostics", documents::many_diagnostics()),
    ];
    println!("rounds per document: {ROUNDS} (after one warm-up parse)");
    println!();
    println!(
        "{:<18} {:>10} {:>10} {:>10} {:>10}",
        "document", "bytes", "median ms", "min ms", "max ms"
    );
    let mut details = String::new();
    let mut summary = String::new();
    for (name, text) in &documents {
        let source = HtmlSource::utf8(text.as_str());
        let usage = parse(&source);
        let mut rounds: Vec<f64> = (0..ROUNDS).map(|_| timed(&source, None)).collect();
        rounds.sort_by(f64::total_cmp);
        let median = rounds[ROUNDS / 2];
        println!(
            "{:<18} {:>10} {:>10.2} {:>10.2} {:>10.2}",
            name,
            text.len(),
            median,
            rounds[0],
            rounds[ROUNDS - 1]
        );
        let mut profiled: Vec<f64> = (0..ROUNDS)
            .map(|_| timed(&source, Some(ProfileMode::Summary { timing: false })))
            .collect();
        profiled.sort_by(f64::total_cmp);
        let _ = writeln!(
            summary,
            "{:<18} {:>10} {:>10.2} {:>10.2} {:>10.2}",
            name,
            text.len(),
            profiled[ROUNDS / 2],
            profiled[0],
            profiled[ROUNDS - 1]
        );
        let _ = writeln!(details, "{name} ({} bytes)", text.len());
        let _ = write!(details, "  work:");
        for (category, label) in WORK {
            let _ = write!(details, " {label}={}", usage.work(category));
        }
        let _ = writeln!(details);
        let _ = write!(details, "  resource high-water:");
        for (resource, label) in RESOURCES {
            let _ = write!(
                details,
                " {label}={}",
                usage.resource(resource).high_water()
            );
        }
        let _ = writeln!(details);
        let _ = write!(details, "  resource current:");
        for (resource, label) in RESOURCES {
            let _ = write!(details, " {label}={}", usage.resource(resource).current());
        }
        let _ = writeln!(details);
    }
    println!();
    println!("profile summary, no timing");
    println!(
        "{:<18} {:>10} {:>10} {:>10} {:>10}",
        "document", "bytes", "median ms", "min ms", "max ms"
    );
    print!("{summary}");
    println!();
    print!("{details}");
}
