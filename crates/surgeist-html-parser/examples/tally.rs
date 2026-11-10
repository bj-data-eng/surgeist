//! Single-document parse timing probe: `probe DOC ROUNDS [usage]`.

use std::{hint::black_box, time::Instant};

#[path = "../benches/support/documents.rs"]
mod documents;

use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseResource, HtmlParseWorkCategory,
    HtmlSource,
};

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
        .expect("limits")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let doc = args.get(1).map_or("table-heavy", String::as_str);
    let rounds: usize = args.get(2).and_then(|r| r.parse().ok()).unwrap_or(5);
    let show_usage = args.get(3).is_some_and(|a| a == "usage");
    let text = match doc {
        "ordinary" => documents::ordinary(),
        "deeply-nested" => documents::deeply_nested(),
        "many-attributes" => documents::many_attributes(),
        "table-heavy" => documents::table_heavy(),
        "many-diagnostics" => documents::many_diagnostics(),
        other => panic!("unknown document {other}"),
    };
    let source = HtmlSource::utf8(text.as_str());
    surgeist_html_parser::tally_reset();
    let mut times = Vec::new();
    let mut last = None;
    for round in 0..rounds {
        let start = Instant::now();
        let report = HtmlDocumentParseRequest::offline(black_box(&source).clone(), limits())
            .parse()
            .unwrap_or_else(|f| panic!("parse failed: {f:?}"));
        let parsed = start.elapsed().as_secs_f64() * 1000.0;
        let usage = report.usage().clone();
        let d = Instant::now();
        drop(black_box(report));
        let dropped = d.elapsed().as_secs_f64() * 1000.0;
        eprintln!("round {round}: parse {parsed:.2} ms, drop {dropped:.2} ms");
        times.push(parsed);
        last = Some(usage);
    }
    println!("{}", surgeist_html_parser::tally_report());
    times.sort_by(f64::total_cmp);
    println!(
        "{doc} bytes={} median={:.2} min={:.2} max={:.2} ms",
        text.len(),
        times[times.len() / 2],
        times[0],
        times[times.len() - 1]
    );
    if show_usage {
        let usage = last.expect("usage");
        for c in [
            HtmlParseWorkCategory::InputUnitRead,
            HtmlParseWorkCategory::InspectByte,
            HtmlParseWorkCategory::CopyByte,
            HtmlParseWorkCategory::StateStep,
            HtmlParseWorkCategory::RecordVisit,
            HtmlParseWorkCategory::RecordWrite,
            HtmlParseWorkCategory::ReserveByte,
            HtmlParseWorkCategory::ProviderCall,
        ] {
            println!("work {c:?} = {}", usage.work(c));
        }
        for r in [
            HtmlParseResource::SourceMapEntries,
            HtmlParseResource::Diagnostics,
            HtmlParseResource::ManagedNodes,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::PeakTransientBytes,
            HtmlParseResource::AtomicLayoutBytes,
            HtmlParseResource::WorkCredits,
        ] {
            let u = usage.resource(r);
            println!("res {r:?} current={} high={}", u.current(), u.high_water());
        }
    }
}
