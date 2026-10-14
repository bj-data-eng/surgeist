//! Defining test-only artifact adapter for actual ordinary progressive input.
//! No benchmark crate, ignored Rust source, other parser or live sink is used.
use super::{Kind, Name, Node, Projection, project, units};
use crate::*;
use std::{io::Write, time::Instant};

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
fn number(out: &mut Vec<u8>, n: usize) {
    out.extend_from_slice(&u64::try_from(n).unwrap().to_le_bytes());
}
fn optional(out: &mut Vec<u8>, n: Option<usize>) {
    number(out, usize::from(n.is_some()));
    if let Some(n) = n {
        number(out, n);
    }
}
fn text(out: &mut Vec<u8>, v: &[u16]) {
    number(out, v.len());
    for n in v {
        out.extend_from_slice(&n.to_le_bytes());
    }
}
fn optional_text(out: &mut Vec<u8>, v: Option<&[u16]>) {
    number(out, usize::from(v.is_some()));
    if let Some(v) = v {
        text(out, v);
    }
}
fn name(out: &mut Vec<u8>, n: &Name) {
    optional_text(out, n.namespace.as_deref());
    optional_text(out, n.prefix.as_deref());
    text(out, &n.local);
}
fn encode(out: &mut Vec<u8>, p: &Projection) {
    number(out, 0); // Independently checked NoQuirks document mode.
    number(out, p.nodes.len());
    for node in &p.nodes {
        optional(out, node.parent);
        number(out, node.children.len());
        for child in &node.children {
            number(out, *child);
        }
        optional(out, node.template);
        assert!(node.owner.is_none() && node.shadow.is_none());
        optional(out, None);
        match &node.kind {
            Kind::Document(HtmlDocumentMode::NoQuirks) => number(out, 0),
            Kind::Element(n, attrs) => {
                number(out, 2);
                name(out, n);
                number(out, attrs.len());
                for a in attrs {
                    name(out, &a.name);
                    text(out, &a.value);
                }
            }
            Kind::Text(t) => {
                number(out, 3);
                text(out, t);
            }
            Kind::Pi { target, data } => {
                number(out, 6);
                text(out, target);
                text(out, data);
            }
            Kind::Doctype {
                name,
                public,
                system,
            } => {
                number(out, 5);
                text(out, name);
                optional_text(out, public.as_deref());
                optional_text(out, system.as_deref());
            }
            _ => panic!("outside the selected ordinary literal protocol"),
        }
    }
}
fn report(l: &Literal, cuts: &[usize], utf16: bool) -> HtmlParseReport {
    use super::super::{report::ImmutableParseReport, seal::SealedOrdinaryConstruction};
    let source = if utf16 {
        HtmlSource::utf16(DocumentString::from(l.input.as_str()))
    } else {
        HtmlSource::utf8(l.input.clone())
    };
    let identity = source.identity().clone();
    let request = HtmlDocumentParseRequest::offline(source, super::super::tests::limits(10_000));
    let cancellation = request.cancellation().clone();
    let clock = Instant::now();
    let parts = {
        let mut completed = super::comment_diagnostic_corpus::completed_at_cuts(&request, cuts);
        eprintln!(
            "FEED_PHASE family={} utf16={utf16} phase=initialization_and_driver ns={} usage={:?}",
            l.name,
            clock.elapsed().as_nanos(),
            completed.usage().unwrap()
        );
        let clock = Instant::now();
        completed.prepare_native_associations().unwrap();
        eprintln!(
            "FEED_PHASE family={} utf16={utf16} phase=associations ns={} usage={:?}",
            l.name,
            clock.elapsed().as_nanos(),
            completed.usage().unwrap()
        );
        let clock = Instant::now();
        let parts = completed.freeze_parts().unwrap();
        eprintln!(
            "FEED_PHASE family={} utf16={utf16} phase=freeze ns={} usage={:?}",
            l.name,
            clock.elapsed().as_nanos(),
            parts.final_usage()
        );
        parts
    };
    let clock = Instant::now();
    let mut owner =
        Some(SealedOrdinaryConstruction::take(&mut Some(request), &mut Some(parts)).unwrap());
    let report = ImmutableParseReport::take_document(&mut owner).unwrap();
    eprintln!(
        "FEED_PHASE family={} utf16={utf16} phase=seal_and_report ns={} usage={:?}",
        l.name,
        clock.elapsed().as_nanos(),
        report.usage()
    );
    assert_eq!(report.source().identity(), &identity);
    cancellation.cancel();
    report
}
#[test]
fn ordinary_scalar_feeds_keep_independent_native_literals_and_complete_report_artifact() {
    let literals = literals();
    let mut encoded = Vec::from(&b"SGHTML-COMPARE\0\x01"[..]);
    number(&mut encoded, literals.len() * 2);
    for l in &literals {
        for width in [1, 7] {
            let cuts = scalar_cuts(&l.input, width, false);
            let report = report(l, &cuts, false);
            let usage = report.usage().clone();
            let document = report.document().unwrap();
            assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
            let actual = project(document).unwrap();
            assert_eq!(actual, l.expected, "{} scalar{width}", l.name);
            encode(&mut encoded, &actual);
            let rows = report.associations();
            for i in 0..rows.len() {
                if let Some(s) = rows.row(i).unwrap().source() {
                    assert_eq!(s.source(), report.source().identity());
                    assert!(s.end() <= report.source().len());
                }
            }
            assert_eq!(report.usage(), &usage);
            eprintln!(
                "FEED_CASE family={} width={width} source={} cuts={cuts:?} native={} d1={} groups={} diagnostics={} usage={usage:?}",
                l.name,
                report.source().len(),
                actual.nodes.len(),
                rows.len(),
                report.history_view().len(),
                report.diagnostics().len()
            );
        }
    }
    // Optional standard-library artifact output; no path or Rust source from
    // ignored working material is named or read by this defining test.
    if let Some(path) = std::env::var_os("SURGEIST_COMPARISON_ARTIFACT") {
        std::fs::File::create(path)
            .unwrap()
            .write_all(&encoded)
            .unwrap();
    }
}
#[test]
fn utf16_scalar_and_pair_unit_feeds_keep_the_same_literal_outputs_and_source() {
    for l in literals() {
        for width in [1, 7] {
            let report = report(&l, &scalar_cuts(&l.input, width, true), true);
            assert_eq!(project(report.document().unwrap()).unwrap(), l.expected);
            assert_eq!(
                report.source().as_utf16().unwrap().code_units(),
                units(&l.input)
            );
        }
        if l.name == "Unicode" {
            let raw = units(&l.input);
            let report = report(&l, &(0..=raw.len()).collect::<Vec<_>>(), true);
            assert_eq!(project(report.document().unwrap()).unwrap(), l.expected);
        }
    }
}
