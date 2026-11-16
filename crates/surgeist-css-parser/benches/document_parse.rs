//! Release-mode cost of `Document::parse`.
//!
//! The cases parse a 1 MiB synthetic style sheet and the `input` texts of the
//! crate's WPT vectors, unmetered and with the default limits. Each case runs
//! over 11 rounds, and each round starts one case later than the one before.
//! The table gives the median, minimum and maximum time of one pass over the
//! input, the input's bytes, the time per byte, and the document's metrics
//! (components, maximum depth, spelling bytes) summed over the texts of the
//! case, with the components per byte. Run with
//! `cargo bench --offline --locked -j 1 -p surgeist-css-parser --bench document_parse`.

use std::{hint::black_box, time::Instant};

use surgeist_css_parser::{
    Document,
    lexer::Options,
    limits::{DepthExcess, Limits},
};

const ROUNDS: usize = 11;

/// The vectors whose cases carry an `input` text.
const WPT_VECTORS: [&str; 6] = [
    include_str!("../tests/corpus/wpt/tokens.json"),
    include_str!("../tests/corpus/wpt/not-whitespace.json"),
    include_str!("../tests/corpus/wpt/units.json"),
    include_str!("../tests/corpus/wpt/consecutive-tokens.json"),
    include_str!("../tests/corpus/wpt/escaped-identifier-round-trips.json"),
    include_str!("../tests/corpus/wpt/unclosed-constructs.json"),
];

/// The `input` text of every case of the WPT vectors.
fn wpt_inputs() -> Vec<String> {
    let mut inputs = Vec::new();
    for vector in WPT_VECTORS {
        let document: serde_json::Value =
            serde_json::from_str(vector).unwrap_or_else(|error| panic!("a WPT vector: {error}"));
        let cases = document
            .get("cases")
            .and_then(serde_json::Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        inputs.extend(
            cases
                .iter()
                .filter_map(|case| case.get("input").and_then(serde_json::Value::as_str))
                .map(str::to_owned),
        );
    }
    inputs
}

/// A style sheet of at least `minimum` bytes, built from rule shapes that real
/// sheets use: selectors, declarations with colours, lengths, calc, urls and
/// strings, at-rules, comments and the occasional escape. Deterministic.
fn synthetic_sheet(minimum: usize) -> String {
    let mut sheet = String::with_capacity(minimum + 1024);
    let mut index = 0_usize;
    while sheet.len() < minimum {
        let n = 3 + index % 29;
        let hex = 0x0010_0000 + (index * 2_654_435_761) % 0x00ef_ffff;
        match index % 8 {
            0 => sheet.push_str(&format!(
                ".card-{index} > a:hover, #item{index}[data-kind=\"k{n}\"]::before {{\n  color: #{hex:06x};\n  margin: {n}px {n}em 0 auto;\n  background: url(images/bg-{index}.png) no-repeat;\n  font: 12px/1.5 \"Helvetica Neue\", sans-serif;\n}}\n"
            )),
            1 => sheet.push_str(&format!("/* section {index}: layout and spacing */\n")),
            2 => sheet.push_str(&format!(
                "@media (min-width: {}px) and (max-width: {}px) {{\n  .m{index} {{ padding: 0 {n}px; width: calc(100% - {}px); content: \"item {n}\"; }}\n}}\n",
                n * 40,
                n * 80,
                n * 3
            )),
            3 => sheet.push_str(&format!(
                ".t{index} {{ transition: opacity .{n}s ease-in-out, transform {n}ms cubic-bezier(.4, 0, .2, 1); transform: rotate(-{n}deg) translate(-.5em, {n}%); }}\n"
            )),
            4 => sheet.push_str(&format!(
                "a.link-{index}:not(.active):nth-child(2n+{n}), input[type=text]:focus {{ outline: 1px solid rgba(0, 0, 0, .{n}); border-radius: {n}px {n}px 0 0; }}\n"
            )),
            5 => sheet.push_str(&format!(
                ".\\31 0{index} {{ margin: -.5e2px; --custom-{n}: {{ a: b }}; grid-template-areas: \"head head\" \"nav main\"; }}\n"
            )),
            6 => sheet.push_str(&format!(
                "@font-face {{ font-family: \"Face {index}\"; src: url(\"fonts/f{index}.woff2\") format(\"woff2\"); unicode-range: U+0025-00FF; }}\n"
            )),
            _ => sheet.push_str(&format!(
                "h{} , p.note-{index}::first-line {{ line-height: 1.{n}; letter-spacing: -0.0{n}em; color: var(--c{n}, #abc) !important; }}\n",
                1 + n % 6
            )),
        }
        index += 1;
    }
    sheet
}

/// What one pass over a case's texts produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Pass {
    components: usize,
    nodes: usize,
    maximum_depth: u32,
    spelling_bytes: usize,
}

/// Parses every text and sums the documents' metrics.
fn parse_all(texts: &[String]) -> Pass {
    let options = Options::default();
    let mut pass = Pass {
        components: 0,
        nodes: 0,
        maximum_depth: 0,
        spelling_bytes: 0,
    };
    for text in texts {
        let document: Document<'_> = Document::parse(
            black_box(text),
            &options,
            Limits::default(),
            0,
            DepthExcess::Fail,
        )
        .expect("the text parses within the default limits");
        let metrics = document.metrics();
        pass.components += metrics.components();
        pass.nodes += document.node_count();
        pass.maximum_depth = pass.maximum_depth.max(metrics.maximum_depth());
        pass.spelling_bytes += metrics.spelling_bytes();
        black_box(&document);
    }
    pass
}

struct Case {
    name: &'static str,
    texts: Vec<String>,
    passes: u32,
}

fn main() {
    let cases = [
        Case {
            name: "Document::parse, 1 MiB synthetic sheet",
            texts: vec![synthetic_sheet(1 << 20)],
            passes: 30,
        },
        Case {
            name: "Document::parse, WPT vector inputs",
            texts: wpt_inputs(),
            passes: 2000,
        },
    ];
    let volumes: Vec<(usize, Pass)> = cases
        .iter()
        .map(|case| {
            let bytes = case.texts.iter().map(String::len).sum::<usize>();
            let pass = parse_all(&case.texts);
            assert!(bytes > 0 && pass.components > 0 && pass.nodes > 0);
            assert_eq!(pass, parse_all(&case.texts), "a pass is deterministic");
            (bytes, pass)
        })
        .collect();

    let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(ROUNDS); cases.len()];
    for round in 0..ROUNDS {
        for step in 0..cases.len() {
            let index = (round + step) % cases.len();
            let case = &cases[index];
            let start = Instant::now();
            for _ in 0..case.passes {
                let pass = parse_all(&case.texts);
                assert_eq!(pass, volumes[index].1);
            }
            times[index].push(start.elapsed().as_secs_f64() * 1e9 / f64::from(case.passes));
        }
    }

    println!("{ROUNDS} rounds, each starting one case later");
    println!(
        "case\tmedian ns/pass\tmin ns/pass\tmax ns/pass\tbytes\tmedian ns/byte\ttexts\tcomponents\tnodes\tmaximum depth\tspelling bytes\tcomponents/byte"
    );
    for (index, case) in cases.iter().enumerate() {
        let mut sorted = times[index].clone();
        sorted.sort_by(f64::total_cmp);
        let (bytes, pass) = volumes[index];
        println!(
            "{}\t{:.0}\t{:.0}\t{:.0}\t{}\t{:.3}\t{}\t{}\t{}\t{}\t{}\t{:.4}",
            case.name,
            sorted[ROUNDS / 2],
            sorted[0],
            sorted[ROUNDS - 1],
            bytes,
            sorted[ROUNDS / 2] / bytes as f64,
            case.texts.len(),
            pass.components,
            pass.nodes,
            pass.maximum_depth,
            pass.spelling_bytes,
            pass.components as f64 / bytes as f64,
        );
    }
}
