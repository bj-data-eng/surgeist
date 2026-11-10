//! Where the parser created scripts, styles and event handler content
//! attributes, whatever detail the parse selected.
//!
//! Each expectation is derived by hand from the source text: a start tag ends
//! after its `>`, a line ends at LF, CRLF or a lone CR, and a column counts the
//! UTF-16 code units of the decoded source from the start of the line.

use crate::parser::support::limits;
use surgeist_html::{
    construction::{HtmlNodeId, HtmlNodeKind},
    values::DocumentString,
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseReport, HtmlSource, HtmlSourcePosition, HtmlSourceRangeError,
};

fn parse(source: HtmlSource) -> HtmlParseReport {
    HtmlDocumentParseRequest::offline(source, limits())
        .parse()
        .expect("the document parses")
}

/// The elements of a report's document, in tree order, by local name.
fn elements(report: &HtmlParseReport) -> Vec<(String, HtmlNodeId)> {
    let document = report.document().expect("a document report");
    let mut found = Vec::new();
    let mut pending = vec![document.root().id()];
    while let Some(id) = pending.pop() {
        let node = document.node(&id).expect("a node of the document");
        if let HtmlNodeKind::Element(element) = node.kind() {
            let name = element.name().local_name();
            found.push((String::from_utf16(name.code_units()).unwrap(), id.clone()));
        }
        let mut children: Vec<_> = node.children().collect();
        children.reverse();
        pending.extend(children);
    }
    found
}

fn first(report: &HtmlParseReport, local: &str) -> HtmlNodeId {
    elements(report)
        .into_iter()
        .find(|(name, _)| name == local)
        .unwrap_or_else(|| panic!("no {local} element"))
        .1
}

fn position(line: usize, column: usize) -> Option<(usize, usize)> {
    Some((line, column))
}

fn pair(value: Option<HtmlSourcePosition>) -> Option<(usize, usize)> {
    value.map(|p| (p.line(), p.column()))
}

#[test]
fn a_script_starts_where_its_start_tag_ends() {
    // `<script>` is eight units, so the tag ends at offset 8: column 9.
    let report = parse(HtmlSource::utf8("<script>x</script>"));
    let id = first(&report, "script");
    assert_eq!(pair(report.script_position(&id).unwrap()), position(1, 9));
    assert_eq!(report.style_position(&id).unwrap(), None);
}

#[test]
fn a_style_starts_where_its_start_tag_ends_on_its_own_line() {
    // "a" and LF take two units; `<style>` is seven more: column 8 of line 2.
    let report = parse(HtmlSource::utf8("a\n<style>b</style>"));
    let id = first(&report, "style");
    assert_eq!(pair(report.style_position(&id).unwrap()), position(2, 8));
}

#[test]
fn crlf_and_lone_cr_end_one_line_each() {
    for (text, line) in [
        ("a\r\n<script>", 2),
        ("a\r<script>", 2),
        ("a\n<script>", 2),
        ("a\r\n\r\n<script>", 3),
        ("a\r\r\n<script>", 3),
        ("\n\n\n<script>", 4),
    ] {
        for source in [
            HtmlSource::utf8(text),
            HtmlSource::utf16(DocumentString::from(text)),
        ] {
            let report = parse(source);
            let id = first(&report, "script");
            assert_eq!(
                pair(report.script_position(&id).unwrap()),
                position(line, 9),
                "{text:?}"
            );
        }
    }
}

#[test]
fn a_column_counts_utf16_units_after_text_outside_the_bmp() {
    // U+1F600 is four UTF-8 bytes and two UTF-16 units, so the column is
    // 2 + 8 + 1 = 11 in both encodings, while the UTF-8 offset is 12.
    let text = "\u{1F600}<script>";
    let utf8 = parse(HtmlSource::utf8(text));
    let utf16 = parse(HtmlSource::utf16(DocumentString::from(text)));
    for report in [&utf8, &utf16] {
        let id = first(report, "script");
        assert_eq!(pair(report.script_position(&id).unwrap()), position(1, 11));
    }
    assert_eq!(
        pair(Some(utf8.source().position(12).unwrap())),
        position(1, 11)
    );
    assert_eq!(
        pair(Some(utf16.source().position(10).unwrap())),
        position(1, 11)
    );
}

#[test]
fn only_a_parser_created_script_or_style_has_a_position() {
    let report = parse(HtmlSource::utf8("<p>x</p><div></div>"));
    for local in ["p", "div"] {
        let id = first(&report, local);
        assert_eq!(report.script_position(&id).unwrap(), None);
        assert_eq!(report.style_position(&id).unwrap(), None);
        assert_eq!(report.event_handler_positions(&id).count(), 0);
    }
}

fn handlers(report: &HtmlParseReport, local: &str) -> Vec<(usize, (usize, usize))> {
    let id = first(report, local);
    report
        .event_handler_positions(&id)
        .map(|entry| {
            let (attribute, position) = entry.unwrap();
            (attribute, (position.line(), position.column()))
        })
        .collect()
}

#[test]
fn event_handler_attributes_are_the_names_of_the_specification_list() {
    // The tag `<p onclick=a title=b onmouseover=c>` is 35 units: both
    // handlers share the position where it ended, and `title` is not one.
    let report = parse(HtmlSource::utf8("<p onclick=a title=b onmouseover=c>"));
    assert_eq!(handlers(&report, "p"), vec![(0, (1, 36)), (2, (1, 36))]);
    // A name that only starts with `on`, and an upper-case spelling the
    // tokenizer lowers to a listed name.
    let report = parse(HtmlSource::utf8("\n<p onfoo=1 ONCLICK=2 on=3>"));
    assert_eq!(handlers(&report, "p"), vec![(1, (2, 27))]);
}

#[test]
fn a_window_reflecting_handler_is_a_handler_only_on_body() {
    let report = parse(HtmlSource::utf8("<div onafterprint=1 onclick=2></div>"));
    assert_eq!(handlers(&report, "div"), vec![(1, (1, 31))]);
    let report = parse(HtmlSource::utf8("<body onafterprint=1 onclick=2>"));
    assert_eq!(handlers(&report, "body"), vec![(0, (1, 32)), (1, (1, 32))]);
}

#[test]
fn a_foreign_element_has_no_event_handler_content_attributes() {
    let report = parse(HtmlSource::utf8(
        "<svg onclick=1></svg><math onclick=1></math>",
    ));
    for local in ["svg", "math"] {
        assert_eq!(handlers(&report, local), vec![], "{local}");
    }
}

#[test]
fn a_handler_merged_into_body_keeps_the_position_of_the_tag_that_added_it() {
    // The second tag, `<body onload=2 title=3>`, is 23 units: column 24 of line 2.
    let report = parse(HtmlSource::utf8(
        "<body onclick=1>\n<body onload=2 title=3>",
    ));
    assert_eq!(handlers(&report, "body"), vec![(0, (1, 17)), (1, (2, 24))]);
}

#[test]
fn a_reconstructed_formatting_element_keeps_the_position_of_its_original_tag() {
    // `<b onclick=1>` follows `<p>` and ends at column 17. The paragraph break closes the b, and
    // the reconstruction creates a second one from the same token.
    let report = parse(HtmlSource::utf8("<p><b onclick=1>x<p>y"));
    let positions: Vec<_> = elements(&report)
        .into_iter()
        .filter(|(name, _)| name == "b")
        .map(|(_, id)| {
            report
                .event_handler_positions(&id)
                .map(|entry| entry.unwrap().1.column())
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(positions, vec![vec![17], vec![17]]);
}

#[test]
fn a_position_past_the_end_or_inside_a_scalar_is_refused() {
    let source = HtmlSource::utf8("a\u{1F600}b");
    assert_eq!(
        source.position(7),
        Err(HtmlSourceRangeError::OutOfBounds { end: 7, length: 6 })
    );
    assert_eq!(
        source.position(2),
        Err(HtmlSourceRangeError::Utf8Boundary { offset: 2 })
    );
    // The end of the source is a position: after `a`, the pair and `b`.
    assert_eq!(pair(Some(source.position(6).unwrap())), position(1, 5));
    // An offset between the CR and the LF of a CRLF is on the first line.
    let crlf = HtmlSource::utf8("a\r\nb");
    assert_eq!(pair(Some(crlf.position(2).unwrap())), position(1, 3));
    assert_eq!(pair(Some(crlf.position(3).unwrap())), position(2, 1));
}

#[test]
fn elements_that_install_their_attributes_one_at_a_time_record_their_handlers() {
    // `<input onclick=1 value=2>` is 25 units, `<select onchange=1>` 19 and
    // `<details onclick=1>` 19: each handler is at index 0.
    for (text, local, column) in [
        ("<input onclick=1 value=2>", "input", 26),
        ("<select onchange=1>", "select", 20),
        ("<details onclick=1>", "details", 20),
        ("<textarea onselect=1>", "textarea", 22),
        ("<img onerror=1>", "img", 16),
        ("<a onclick=1>", "a", 14),
    ] {
        let report = parse(HtmlSource::utf8(text));
        assert_eq!(handlers(&report, local), vec![(0, (1, column))], "{text}");
    }
}

#[test]
fn every_position_of_a_document_of_many_elements_is_read_back() {
    // Each group is three lines: a script (tag end at column 9), a style (column
    // 8) and a div with two handlers (`<div onclick=1 onmouseover=2>` is 29
    // units, so column 30).
    const GROUPS: usize = 100;
    let text =
        "<script></script>\n<style></style>\n<div onclick=1 onmouseover=2></div>\n".repeat(GROUPS);
    let report = parse(HtmlSource::utf8(text));
    let all = elements(&report);
    let named = |local: &str| -> Vec<HtmlNodeId> {
        all.iter()
            .filter(|(name, _)| name == local)
            .map(|(_, id)| id.clone())
            .collect()
    };
    let (scripts, styles, divs) = (named("script"), named("style"), named("div"));
    assert_eq!(
        (scripts.len(), styles.len(), divs.len()),
        (GROUPS, GROUPS, GROUPS)
    );
    for group in 0..GROUPS {
        let line = 3 * group + 1;
        let script = report.script_position(&scripts[group]).unwrap().unwrap();
        assert_eq!(
            (script.line(), script.column()),
            (line, 9),
            "script {group}"
        );
        let style = report.style_position(&styles[group]).unwrap().unwrap();
        assert_eq!(
            (style.line(), style.column()),
            (line + 1, 8),
            "style {group}"
        );
        let handlers: Vec<_> = report
            .event_handler_positions(&divs[group])
            .map(|entry| {
                let (attribute, position) = entry.unwrap();
                (attribute, position.line(), position.column())
            })
            .collect();
        assert_eq!(
            handlers,
            vec![(0, line + 2, 30), (1, line + 2, 30)],
            "div {group}"
        );
    }
}
