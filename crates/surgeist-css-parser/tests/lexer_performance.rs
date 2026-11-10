//! The throughput of the free-running `Lexer`.
//!
//! In a release build, unmetered, this measures how many megabytes per second
//! the lexer tokenizes on two inputs, in one process on one machine:
//!
//! 1. the inputs of the crate's WPT vectors (`tests/corpus/wpt/*.json`: the
//!    `input` text of every case of the token, whitespace, unit, consecutive
//!    token, escaped identifier and unclosed construct vectors), each lexed as
//!    its own text; and
//! 2. a 1 MiB synthetic style sheet built from rule shapes that real sheets
//!    use: selectors, declarations with colours, lengths, calc, urls and
//!    strings, at-rules, comments and the occasional escape.
//!
//! The lexer is driven through `next_lexeme`, and each iteration reads the
//! token's kind and the lexeme's end, so the work is not optimized away.
//!
//! This is a measurement, not a regression test: it asserts nothing about the
//! speed, it is ignored, and it runs only when `SURGEIST_CSS_PARSER_PERF=1` is
//! set, in release mode:
//!
//! ```sh
//! SURGEIST_CSS_PARSER_PERF=1 cargo test --release --offline --locked -j 1 \
//!     -p surgeist-css-parser --test lexer_performance -- --ignored --nocapture
//! ```
//!
//! It prints, per input, the bytes, the token count and the throughput in MB/s
//! (best round and median round). The printed figures are the baseline that a
//! change to the lexer is compared with, on the same machine.

use std::hint::black_box;
use std::time::{Duration, Instant};

use surgeist_css_parser::lexer::{Lexer, Options};

const ROUNDS: usize = 15;

/// The vectors whose cases carry an `input` text.
const WPT_VECTORS: [&str; 6] = [
    include_str!("corpus/wpt/tokens.json"),
    include_str!("corpus/wpt/not-whitespace.json"),
    include_str!("corpus/wpt/units.json"),
    include_str!("corpus/wpt/consecutive-tokens.json"),
    include_str!("corpus/wpt/escaped-identifier-round-trips.json"),
    include_str!("corpus/wpt/unclosed-constructs.json"),
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
/// sheets use. Deterministic.
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

/// Tokenizes `text` with the lexer; returns the token count and a checksum of
/// every token's kind and end.
fn run_lexer(text: &str) -> (usize, usize) {
    let mut lexer = Lexer::new(text, &Options::default());
    let (mut count, mut sum) = (0, 0_usize);
    while let Some(lexeme) = lexer.next_lexeme() {
        let token = lexeme.token();
        sum = sum.wrapping_add(black_box(token.kind() as usize));
        sum = sum.wrapping_add(lexeme.range().end.value());
        count += 1;
    }
    (count, sum)
}

struct Measured {
    best: Duration,
    median: Duration,
}

fn measure(texts: &[String], repeats: usize, run: fn(&str) -> (usize, usize)) -> Measured {
    let mut rounds = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let start = Instant::now();
        let mut total = 0;
        for _ in 0..repeats {
            for text in texts {
                total += black_box(run(black_box(text))).1;
            }
        }
        black_box(total);
        rounds.push(start.elapsed());
    }
    rounds.sort();
    Measured {
        best: rounds[0],
        median: rounds[ROUNDS / 2],
    }
}

fn megabytes_per_second(bytes: usize, repeats: usize, time: Duration) -> f64 {
    (bytes * repeats) as f64 / 1e6 / time.as_secs_f64()
}

/// Measures one input and prints the lexer's throughput on it.
fn report(name: &str, texts: &[String], repeats: usize) {
    let bytes: usize = texts.iter().map(String::len).sum();
    let (tokens, _) = texts
        .iter()
        .map(|text| run_lexer(text))
        .fold((0_usize, 0_usize), |a, b| {
            (a.0 + b.0, a.1.wrapping_add(b.1))
        });
    // Warm once, then measure two series of rounds and keep the better of each
    // statistic, so a slow drift of the machine does not decide the figure.
    let _ = measure(texts, 1, run_lexer);
    let first = measure(texts, repeats, run_lexer);
    let second = measure(texts, repeats, run_lexer);
    let (best, median) = (first.best.min(second.best), first.median.min(second.median));
    let rate = |time| megabytes_per_second(bytes, repeats, time);
    println!(
        "{name}: {bytes} bytes in {} texts, {tokens} tokens (closers included)",
        texts.len()
    );
    println!(
        "  lexer best {:8.1} MB/s  median {:8.1} MB/s",
        rate(best),
        rate(median)
    );
}

fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

#[test]
#[ignore = "a measurement: run in release mode with SURGEIST_CSS_PARSER_PERF=1"]
fn the_lexer_throughput_is_measured_on_the_wpt_vector_inputs_and_a_synthetic_sheet() {
    if std::env::var_os("SURGEIST_CSS_PARSER_PERF").is_none() {
        eprintln!("SURGEIST_CSS_PARSER_PERF is not set: nothing measured");
        return;
    }
    assert!(
        !is_debug_build(),
        "the throughput is measured in a release build"
    );
    let inputs = wpt_inputs();
    assert!(!inputs.is_empty(), "the WPT vectors hold case inputs");
    let sheet = synthetic_sheet(1 << 20);
    assert!(sheet.len() >= 1 << 20);

    report("WPT vector inputs", &inputs, 300);
    report("1 MiB synthetic sheet", &[sheet], 6);
}

/// Spins the lexer on the synthetic sheet for `SURGEIST_CSS_PARSER_PROFILE_SECONDS`
/// seconds, so a sampling profiler can attach to the process.
#[test]
#[ignore = "a profiling aid: run in release mode with SURGEIST_CSS_PARSER_PROFILE_SECONDS=n"]
fn spin_the_lexer_for_a_profiler() {
    let Some(seconds) = std::env::var("SURGEIST_CSS_PARSER_PROFILE_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
    else {
        return;
    };
    let sheet = synthetic_sheet(1 << 20);
    let end = Instant::now() + Duration::from_secs(seconds);
    let mut total = 0;
    while Instant::now() < end {
        total += run_lexer(black_box(&sheet)).1;
    }
    black_box(total);
}
