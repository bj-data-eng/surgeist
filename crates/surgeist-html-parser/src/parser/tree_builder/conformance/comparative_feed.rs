//! Defining test of actual ordinary progressive input against independent
//! native literals. The tests here assert only; they print no timing and write
//! no artifact. The comparison-artifact and phase-timing output lives in
//! [`artifact_tool`], an explicit opt-in tool.
//! No benchmark crate, ignored Rust source, other parser or live sink is used.
use super::{Kind, Name, Node, Projection, project, units};
use crate::*;

mod artifact_tool;

fn append(p: &mut Projection, parent: usize, kind: Kind) -> usize {
    let index = p.nodes.len();
    p.nodes.push(Node {
        kind,
        parent: Some(parent),
        children: Vec::new(),
        template: None,
        shadow: None,
        owner: None,
    });
    p.nodes[parent].children.push(index);
    index
}
fn element(local: &str) -> Kind {
    Kind::Element(
        Name {
            namespace: Some(units("http://www.w3.org/1999/xhtml")),
            prefix: None,
            local: units(local),
        },
        Vec::new(),
    )
}
fn document() -> Projection {
    let mut p = Projection {
        nodes: vec![Node {
            kind: Kind::Document(HtmlDocumentMode::NoQuirks),
            parent: None,
            children: Vec::new(),
            template: None,
            shadow: None,
            owner: None,
        }],
    };
    append(
        &mut p,
        0,
        Kind::Doctype {
            name: units("html"),
            public: Some(Vec::new()),
            system: Some(Vec::new()),
        },
    );
    append(&mut p, 0, element("html"));
    append(&mut p, 2, element("head"));
    append(&mut p, 2, element("body"));
    p
}
struct Literal {
    name: &'static str,
    input: String,
    expected: Projection,
    /// Lower bound on association rows that carry a source range. Every node
    /// the input's tokens create from explicit source text must be associated
    /// with the range of the token that created it: the doctype, each explicit
    /// start tag, each text token and the processing instruction. Elements the
    /// HTML tree builder implies (no token) and reconstruction copies add
    /// nothing to this count.
    min_sourced_rows: usize,
    /// Source tokens that occur exactly once in `input` and must each be the
    /// exact source range of some association row. The range is the token's
    /// byte span in the (UTF-8) input, found by searching the input text, not
    /// read from any parse output.
    anchors: &'static [&'static str],
}
fn literals() -> Vec<Literal> {
    let mut result = Vec::new();
    for (name, spelling, value) in [
        ("Unicode", "Aé😀", vec![0x41, 0xe9, 0xd83d, 0xde00]),
        (
            "CRLF-references",
            "A\r\n&amp;&#128512;&NotEqualTilde;",
            vec![0x41, 10, 38, 0xd83d, 0xde00, 0x2242, 0x338],
        ),
    ] {
        let mut p = document();
        let parent = append(&mut p, 4, element("p"));
        append(&mut p, parent, Kind::Text(value.repeat(64)));
        result.push(Literal {
            name,
            input: format!(
                "<!doctype html><html><head></head><body><p>{}</p></body></html>",
                spelling.repeat(64)
            ),
            expected: p,
            // doctype, html, head, body, p and the one text token.
            min_sourced_rows: 6,
            anchors: &["<!doctype html>", "<html>", "<head>", "<body>", "<p>"],
        });
    }
    // HTML877d adoption/reconstruction: two distinct i records, not one moved
    // identity or text-only output. This tree is independent of a complete run.
    let mut p = document();
    let parent = append(&mut p, 4, element("p"));
    let b = append(&mut p, parent, element("b"));
    append(&mut p, b, Kind::Text(vec![65]));
    let i = append(&mut p, b, element("i"));
    append(&mut p, i, Kind::Text(vec![66]));
    let copied = append(&mut p, parent, element("i"));
    append(&mut p, copied, Kind::Text(vec![67]));
    append(&mut p, parent, Kind::Text(vec![68]));
    result.push(Literal {
        name: "reconstruction",
        input: "<!doctype html><html><head></head><body><p><b>A<i>B</b>C</i>D</p></body></html>"
            .into(),
        expected: p,
        // doctype, html, head, body, p, b, i and the text tokens A, B, C, D.
        min_sourced_rows: 11,
        anchors: &["<!doctype html>", "<html>", "<p>", "<b>", "<i>"],
    });
    let mut p = document();
    // The independent literal places PI before html. Shift the scaffold's
    // preorder IDs rather than obtaining an expectation from another parse.
    for node in &mut p.nodes {
        node.parent = node.parent.map(|n| if n >= 2 { n + 1 } else { n });
        for child in &mut node.children {
            if *child >= 2 {
                *child += 1;
            }
        }
    }
    p.nodes.insert(
        2,
        Node {
            kind: Kind::Pi {
                target: units("pi"),
                data: units("x"),
            },
            parent: Some(0),
            children: Vec::new(),
            template: None,
            shadow: None,
            owner: None,
        },
    );
    p.nodes[0].children.insert(1, 2);
    let parent = append(&mut p, 5, element("p"));
    append(&mut p, parent, Kind::Text(vec![65]));
    result.push(Literal {
        name: "pinned-PI",
        input: "<!doctype html><?pi x?><p>A</p>".into(),
        expected: p,
        // doctype, processing instruction, p and its text; html, head and body
        // are implied.
        min_sourced_rows: 4,
        anchors: &["<!doctype html>", "<?pi x?>", "<p>"],
    });
    result
}
fn scalar_cuts(raw: &str, width: usize, utf16: bool) -> Vec<usize> {
    let mut offset = 0;
    let ends: Vec<_> = raw
        .chars()
        .map(|c| {
            offset += if utf16 { c.len_utf16() } else { c.len_utf8() };
            offset
        })
        .collect();
    ends.chunks(width)
        .map(|part| *part.last().unwrap())
        .collect()
}
/// Receives the usage reached at the end of each construction phase. The tool
/// times phases through it; the tests pass [`Silent`], which measures nothing.
trait Phases {
    fn phase(&mut self, name: &'static str, usage: &dyn std::fmt::Debug);
}
struct Silent;
impl Phases for Silent {
    fn phase(&mut self, _: &'static str, _: &dyn std::fmt::Debug) {}
}
fn report(l: &Literal, cuts: &[usize], utf16: bool, phases: &mut dyn Phases) -> HtmlParseReport {
    use super::super::{report::ImmutableParseReport, seal::SealedOrdinaryConstruction};
    let source = if utf16 {
        HtmlSource::utf16(DocumentString::from(l.input.as_str()))
    } else {
        HtmlSource::utf8(l.input.clone())
    };
    let identity = source.identity().clone();
    let request = HtmlDocumentParseRequest::offline(source, super::super::tests::limits(10_000));
    let cancellation = request.cancellation().clone();
    let parts = {
        let mut completed = super::comment_diagnostic_corpus::completed_at_cuts(&request, cuts);
        phases.phase("initialization_and_driver", &completed.usage().unwrap());
        completed.prepare_native_associations().unwrap();
        phases.phase("associations", &completed.usage().unwrap());
        let parts = completed.freeze_parts().unwrap();
        phases.phase("freeze", &parts.final_usage());
        parts
    };
    let mut owner =
        Some(SealedOrdinaryConstruction::take(&mut Some(request), &mut Some(parts)).unwrap());
    let report = ImmutableParseReport::take_document(&mut owner).unwrap();
    phases.phase("seal_and_report", report.usage());
    assert_eq!(report.source().identity(), &identity);
    cancellation.cancel();
    report
}
/// One scalar-feed case: the literal parsed with `width`-scalar chunks, its
/// report, and the report's native projection, already compared with the
/// literal's independent expectation.
fn scalar_feed(
    l: &Literal,
    width: usize,
    phases: &mut dyn Phases,
) -> (Vec<usize>, HtmlParseReport, Projection) {
    let cuts = scalar_cuts(&l.input, width, false);
    let report = report(l, &cuts, false, phases);
    let document = report.document().unwrap();
    assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
    let actual = project(document).unwrap();
    assert_eq!(actual, l.expected, "{} scalar{width}", l.name);
    (cuts, report, actual)
}
#[test]
fn ordinary_scalar_feeds_keep_independent_native_literals_and_sourced_associations() {
    for l in literals() {
        for width in [1, 7] {
            let (_, report, _) = scalar_feed(&l, width, &mut Silent);
            let rows = report.associations();
            let mut sourced = 0;
            let mut ranges = Vec::new();
            for i in 0..rows.len() {
                if let Some(s) = rows.row(i).unwrap().unwrap().source() {
                    assert_eq!(s.source(), report.source().identity());
                    assert!(s.end() <= report.source().len());
                    ranges.push((s.start(), s.end()));
                    sourced += 1;
                }
            }
            for token in l.anchors {
                assert_eq!(
                    l.input.matches(token).count(),
                    1,
                    "{} scalar{width}: {token:?} is unique in the input",
                    l.name
                );
                let start = l.input.find(token).unwrap();
                assert!(
                    ranges.contains(&(start, start + token.len())),
                    "{} scalar{width}: no association row has exactly the source range {start}..{} of {token:?}",
                    l.name,
                    start + token.len()
                );
            }
            assert!(
                sourced >= l.min_sourced_rows,
                "{} scalar{width}: {sourced} sourced rows, at least {} expected",
                l.name,
                l.min_sourced_rows
            );
        }
    }
}
#[test]
fn utf16_scalar_and_pair_unit_feeds_keep_the_same_literal_outputs_and_source() {
    for l in literals() {
        for width in [1, 7] {
            let report = report(&l, &scalar_cuts(&l.input, width, true), true, &mut Silent);
            assert_eq!(project(report.document().unwrap()).unwrap(), l.expected);
            assert_eq!(
                report.source().as_utf16().unwrap().code_units(),
                units(&l.input)
            );
        }
        if l.name == "Unicode" {
            let raw = units(&l.input);
            let report = report(&l, &(0..=raw.len()).collect::<Vec<_>>(), true, &mut Silent);
            assert_eq!(project(report.document().unwrap()).unwrap(), l.expected);
        }
    }
}
