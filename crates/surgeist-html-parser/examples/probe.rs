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
    let text = if let Some(n) = doc.strip_prefix("nest:") {
        let depth: usize = n.parse().expect("count");
        let mut html = String::from("<!DOCTYPE html><html><body>");
        for run in 0..4 {
            for _ in 0..depth {
                html.push_str("<div>");
            }
            html.push_str(&format!("<p>Leaf of run {run}</p>"));
            for _ in 0..depth {
                html.push_str("</div>");
            }
        }
        html.push_str("</body></html>");
        html
    } else if let Some(n) = doc.strip_prefix("attrs:") {
        let count: usize = n.parse().expect("count");
        let mut html = String::from("<!DOCTYPE html><html><body>");
        for element in 0..count {
            html.push_str(&format!("<div id=\"e{element}\""));
            for attribute in 0..24_usize {
                html.push_str(&format!(" data-a{attribute}=\"value {element} {attribute}\""));
            }
            html.push_str(">content</div>");
        }
        html.push_str("</body></html>");
        html
    } else if let Some(n) = doc.strip_prefix("diag:") {
        let count: usize = n.parse().expect("count");
        let mut html = String::from("<!DOCTYPE html><html><body>");
        for index in 0..count {
            html.push_str(&format!("</span><p class=\"a\" class=\"b\">text &bogus; &#x110000; {index}</i><table>stray {index}<tr><td>cell</table></b>"));
        }
        html.push_str("</body></html>");
        html
    } else if let Some(n) = doc.strip_prefix("sections:") {
        let count: usize = n.parse().expect("count");
        let mut html = String::from("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Ordinary</title><style>body{margin:0}</style></head><body>");
        for section in 0..count {
            html.push_str(&format!("<section id=\"s{section}\" class=\"part\"><h2>Section {section}</h2><p>Paragraph with <b>bold</b>, <i>italic</i> and <a href=\"/p/{section}\">a link</a>. Entities &amp; &lt;markup&gt; &copy; stay text.</p><ul>"));
            for item in 0..6_usize {
                html.push_str(&format!("<li class=\"item\"><span>Item {item}</span> of section {section}</li>"));
            }
            html.push_str("</ul><p>Closing paragraph of the section.</p></section>");
        }
        html.push_str("</body></html>");
        html
    } else if let Some(n) = doc.strip_prefix("tables:") {
        let n: usize = n.parse().expect("count");
        let mut html = String::from("<!DOCTYPE html><html><body>");
        for table in 0..n {
            use std::fmt::Write as _;
            let _ = write!(html, "<table id=\"t{table}\"><caption>Table {table}</caption><thead><tr>");
            for column in 0..16_usize {
                let _ = write!(html, "<th scope=\"col\">C{column}</th>");
            }
            html.push_str("</tr></thead><tbody>");
            for row in 0..60_usize {
                html.push_str("<tr>");
                for column in 0..16_usize {
                    if column == 3 && row % 7 == 0 {
                        let _ = write!(html, "<td colspan=\"2\">{row}:{column}</td>");
                    } else {
                        let _ = write!(html, "<td>{row}:{column}</td>");
                    }
                }
                html.push_str("</tr>");
            }
            html.push_str("</tbody></table>");
        }
        html.push_str("</body></html>");
        html
    } else { match doc {
        "ordinary" => documents::ordinary(),
        "deeply-nested" => documents::deeply_nested(),
        "many-attributes" => documents::many_attributes(),
        "table-heavy" => documents::table_heavy(),
        "many-diagnostics" => documents::many_diagnostics(),
        other => panic!("unknown document {other}"),
    } };
    let source = HtmlSource::utf8(text.as_str());
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
