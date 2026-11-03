//! Explicit opt-in tool for the comparison artifact and phase timings.
//!
//! The artifact adapter needs the crate-private progressive driver and the
//! sealed report constructors, which only the test harness can reach, so the
//! tool is an ignored harness entry rather than a binary or example (both would
//! be limited to the public API, which has no progressive feed). The ordinary
//! test run never executes it; the assertion test in the parent module owns
//! the correctness contract.
//!
//! Run from the repository root:
//!
//! ```sh
//! SURGEIST_COMPARISON_ARTIFACT=/absolute/path/artifact.bin \
//!   cargo test --offline --locked -j 1 -p surgeist-html-parser --lib \
//!   comparison_artifact_tool -- --ignored --nocapture
//! ```
//!
//! Phase timings and per-case rows go to stderr (`FEED_PHASE`, `FEED_CASE`).
//! Without the environment variable no artifact is written.
use super::super::{Kind, Name, Projection};
use super::{Phases, literals, scalar_feed};

use std::{io::Write, time::Instant};
use surgeist_html::construction::HtmlDocumentMode;

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

/// Times each phase since the previous one and prints it with its usage.
struct Timing {
    family: &'static str,
    clock: Instant,
}
impl Phases for Timing {
    fn phase(&mut self, name: &'static str, usage: &dyn std::fmt::Debug) {
        eprintln!(
            "FEED_PHASE family={} utf16=false phase={name} ns={} usage={usage:?}",
            self.family,
            self.clock.elapsed().as_nanos(),
        );
        self.clock = Instant::now();
    }
}

#[test]
#[ignore = "explicit tool: prints phase timings and optionally writes the comparison artifact"]
fn comparison_artifact_tool() {
    let literals = literals();
    let mut encoded = Vec::from(&b"SGHTML-COMPARE\0\x01"[..]);
    number(&mut encoded, literals.len() * 2);
    for l in &literals {
        for width in [1, 7] {
            let mut timing = Timing {
                family: l.name,
                clock: Instant::now(),
            };
            let (cuts, report, actual) = scalar_feed(l, width, &mut timing);
            encode(&mut encoded, &actual);
            let rows = report.associations();
            eprintln!(
                "FEED_CASE family={} width={width} source={} cuts={cuts:?} native={} d1={} groups={} diagnostics={} usage={:?}",
                l.name,
                report.source().len(),
                actual.nodes.len(),
                rows.len(),
                report.history_view().len(),
                report.diagnostics().len(),
                report.usage()
            );
        }
    }
    // Optional standard-library artifact output; no path or Rust source from
    // ignored working material is named or read by this tool.
    if let Some(path) = std::env::var_os("SURGEIST_COMPARISON_ARTIFACT") {
        std::fs::File::create(path)
            .unwrap()
            .write_all(&encoded)
            .unwrap();
    }
}
