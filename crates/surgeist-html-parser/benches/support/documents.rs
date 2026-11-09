//! The deterministic generated documents the HTML benchmarks parse.
//!
//! Shared by the `parse` and `semantics` benchmarks so that both measure the
//! same five documents.

use std::fmt::Write as _;

/// A large ordinary document: headings, paragraphs with inline formatting,
/// links, lists and a few attributes per element.
pub fn ordinary() -> String {
    let mut html = String::from(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <title>Ordinary</title><style>body{margin:0}</style></head><body>",
    );
    for section in 0..240_usize {
        let _ = write!(
            html,
            "<section id=\"s{section}\" class=\"part\"><h2>Section {section}</h2>\
             <p>Paragraph with <b>bold</b>, <i>italic</i> and <a href=\"/p/{section}\">a link</a>. \
             Entities &amp; &lt;markup&gt; &copy; stay text.</p>\
             <ul>"
        );
        for item in 0..6_usize {
            let _ = write!(
                html,
                "<li class=\"item\"><span>Item {item}</span> of section {section}</li>"
            );
        }
        html.push_str("</ul><p>Closing paragraph of the section.</p></section>");
    }
    html.push_str("</body></html>");
    html
}

/// Deeply nested elements: runs of nested `div` elements closed in order.
pub fn deeply_nested() -> String {
    const DEPTH: usize = 1_000;
    const RUNS: usize = 10;
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for run in 0..RUNS {
        for _ in 0..DEPTH {
            html.push_str("<div>");
        }
        let _ = write!(html, "<p>Leaf of run {run}</p>");
        for _ in 0..DEPTH {
            html.push_str("</div>");
        }
    }
    html.push_str("</body></html>");
    html
}

/// Elements that each carry many attributes.
pub fn many_attributes() -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for element in 0..1_000_usize {
        let _ = write!(html, "<div id=\"e{element}\"");
        for attribute in 0..24_usize {
            let _ = write!(html, " data-a{attribute}=\"value {element} {attribute}\"");
        }
        html.push_str(">content</div>");
    }
    html.push_str("</body></html>");
    html
}

/// Tables with many rows and cells, with explicit sections and spans.
pub fn table_heavy() -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for table in 0..16_usize {
        let _ = write!(
            html,
            "<table id=\"t{table}\"><caption>Table {table}</caption><thead><tr>"
        );
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
}

/// Markup that produces parse-error diagnostics: stray end tags, unknown
/// character references, duplicate attributes and misplaced table content.
pub fn many_diagnostics() -> String {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    for index in 0..2_400_usize {
        let _ = write!(
            html,
            "</span><p class=\"a\" class=\"b\">text &bogus; &#x110000; {index}</i>\
             <table>stray {index}<tr><td>cell</table></b>"
        );
    }
    html.push_str("</body></html>");
    html
}
