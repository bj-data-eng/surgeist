//! The performance gate of the free-running `Lexer` (user decision: 1.0x).
//!
//! In a release build, unmetered, the lexer must match or beat cssparser's
//! tokenizer on two inputs, measured on the same machine in the same process:
//!
//! 1. the CSSTree corpus's sources, the 935 case texts of
//!    `crates/surgeist-css/tests/corpus/csstree/source` (the repository's one
//!    corpus of real-shaped CSS fragments); and
//! 2. a 1 MiB synthetic style sheet built from rule shapes that real sheets
//!    use: selectors, declarations with colours, lengths, calc, urls and
//!    strings, at-rules, comments and the occasional escape.
//!
//! cssparser is driven through its public `Parser`, entering every nested
//! block so that every token is produced, and the lexer through
//! `next_lexeme`. Each side reads the token and its end position, so neither is
//! optimized away.
//!
//! This is a measurement, not a regression test: it is ignored, and it runs only
//! when `SURGEIST_CSS_PARSER_PERF=1` is set, in release mode:
//!
//! ```sh
//! SURGEIST_CSS_PARSER_PERF=1 cargo test --release --offline --locked -j 1 \
//!     -p surgeist-css-parser --test lexer_performance -- --ignored --nocapture
//! ```
//!
//! It prints, per input, the bytes, the token counts of both sides, the
//! throughput of both in MB/s (best round and median round) and their ratio,
//! and fails when the ratio of the median rounds is below 1.0 on either input.
//! The CSSTree sources are read from the sibling `surgeist-css` crate in the
//! workspace; the test does nothing outside a workspace checkout.

use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use cssparser::{Parser, ParserInput, Token as Css};
use surgeist_css_parser::lexer::{Lexer, Options};

const ROUNDS: usize = 15;

/// Reads the string value of every `"source"` key in the corpus's JSON files.
fn csstree_sources(root: &Path) -> Vec<String> {
    fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                files(&path, out);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "json")
                && !path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with('.'))
            {
                out.push(path);
            }
        }
    }
    let mut paths = Vec::new();
    files(root, &mut paths);
    let mut sources = Vec::new();
    for path in paths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut rest = text.as_str();
        while let Some(at) = rest.find("\"source\":") {
            rest = &rest[at + "\"source\":".len()..];
            let Some(open) = rest.find('"') else { break };
            let mut value = String::new();
            let mut chars = rest[open + 1..].chars();
            let mut consumed = open + 1;
            while let Some(point) = chars.next() {
                consumed += point.len_utf8();
                match point {
                    '"' => break,
                    '\\' => {
                        let Some(escape) = chars.next() else { break };
                        consumed += escape.len_utf8();
                        match escape {
                            'n' => value.push('\n'),
                            'r' => value.push('\r'),
                            't' => value.push('\t'),
                            'f' => value.push('\u{c}'),
                            'b' => value.push('\u{8}'),
                            'u' => {
                                let mut code = 0_u32;
                                for _ in 0..4 {
                                    if let Some(digit) = chars.next() {
                                        consumed += 1;
                                        code = code * 16 + digit.to_digit(16).unwrap_or(0);
                                    }
                                }
                                // A high surrogate pairs with the escape after it.
                                if (0xd800..0xdc00).contains(&code) {
                                    let mut low = 0_u32;
                                    for index in 0..6 {
                                        if let Some(digit) = chars.next() {
                                            consumed += 1;
                                            if index >= 2 {
                                                low = low * 16 + digit.to_digit(16).unwrap_or(0);
                                            }
                                        }
                                    }
                                    code = 0x10000 + ((code - 0xd800) << 10) + (low - 0xdc00);
                                }
                                value.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                            }
                            other => value.push(other),
                        }
                    }
                    other => value.push(other),
                }
            }
            sources.push(value);
            rest = &rest[consumed.min(rest.len())..];
        }
    }
    sources
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

fn walk(parser: &mut Parser<'_, '_>, count: &mut usize, sum: &mut usize) {
    loop {
        let token = match parser.next_including_whitespace_and_comments() {
            Ok(token) => black_box(token.clone()),
            Err(_) => return,
        };
        *count += 1;
        *sum = sum.wrapping_add(parser.position().byte_index());
        if matches!(
            token,
            Css::Function(_)
                | Css::ParenthesisBlock
                | Css::SquareBracketBlock
                | Css::CurlyBracketBlock
        ) {
            let _: Result<(), cssparser::ParseError<'_, ()>> = parser.parse_nested_block(|inner| {
                walk(inner, count, sum);
                Ok(())
            });
        }
    }
}

/// Tokenizes `text` with cssparser through its `Parser`, entering every block.
fn run_cssparser(text: &str) -> (usize, usize) {
    let mut input = ParserInput::new(text);
    let mut parser = Parser::new(&mut input);
    let (mut count, mut sum) = (0, 0_usize);
    walk(&mut parser, &mut count, &mut sum);
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

/// Measures one input and returns the median-round ratio of the lexer's
/// throughput to cssparser's.
fn gate(name: &str, texts: &[String], repeats: usize) -> f64 {
    let bytes: usize = texts.iter().map(String::len).sum();
    let ours: (usize, usize) = texts
        .iter()
        .map(|text| run_lexer(text))
        .fold((0, 0), |a, b| (a.0 + b.0, a.1.wrapping_add(b.1)));
    let theirs: (usize, usize) = texts
        .iter()
        .map(|text| run_cssparser(text))
        .fold((0, 0), |a, b| (a.0 + b.0, a.1.wrapping_add(b.1)));
    // Interleave the sides so slow drift of the machine hits both alike: warm
    // each once, then measure the lexer, cssparser, the lexer again and
    // cssparser again, and keep the better round of each.
    let _ = measure(texts, 1, run_lexer);
    let _ = measure(texts, 1, run_cssparser);
    let a = measure(texts, repeats, run_lexer);
    let b = measure(texts, repeats, run_cssparser);
    let c = measure(texts, repeats, run_lexer);
    let d = measure(texts, repeats, run_cssparser);
    let (lexer_best, lexer_median) = (a.best.min(c.best), a.median.min(c.median));
    let (css_best, css_median) = (b.best.min(d.best), b.median.min(d.median));
    let rate = |time| megabytes_per_second(bytes, repeats, time);
    let ratio = rate(lexer_median) / rate(css_median);
    println!(
        "{name}: {bytes} bytes in {} texts; tokens: lexer {} (closers included), cssparser {} (closers inside blocks not returned)",
        texts.len(),
        ours.0,
        theirs.0
    );
    println!(
        "  lexer     best {:8.1} MB/s  median {:8.1} MB/s",
        rate(lexer_best),
        rate(lexer_median)
    );
    println!(
        "  cssparser best {:8.1} MB/s  median {:8.1} MB/s",
        rate(css_best),
        rate(css_median)
    );
    println!(
        "  ratio (lexer / cssparser): best {:.3}, median {:.3}",
        rate(lexer_best) / rate(css_best),
        ratio
    );
    ratio
}

fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

#[test]
#[ignore = "a measurement: run in release mode with SURGEIST_CSS_PARSER_PERF=1"]
fn the_lexer_matches_or_beats_cssparser_on_the_csstree_sources_and_a_synthetic_sheet() {
    if std::env::var_os("SURGEIST_CSS_PARSER_PERF").is_none() {
        eprintln!("SURGEIST_CSS_PARSER_PERF is not set: nothing measured");
        return;
    }
    assert!(!is_debug_build(), "the gate is measured in a release build");
    let corpus =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../surgeist-css/tests/corpus/csstree/source");
    let sources = csstree_sources(&corpus);
    assert_eq!(
        sources.len(),
        935,
        "the CSSTree corpus holds 935 case sources"
    );
    let sheet = synthetic_sheet(1 << 20);
    assert!(sheet.len() >= 1 << 20);

    let csstree = gate("CSSTree sources", &sources, 300);
    let synthetic = gate("1 MiB synthetic sheet", &[sheet], 6);
    assert!(
        csstree >= 1.0 && synthetic >= 1.0,
        "the lexer must match or beat cssparser: CSSTree {csstree:.3}, synthetic {synthetic:.3}"
    );
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
    let which = std::env::var("SURGEIST_CSS_PARSER_PROFILE_SIDE").unwrap_or_default();
    let end = Instant::now() + Duration::from_secs(seconds);
    let mut total = 0;
    while Instant::now() < end {
        total += if which == "cssparser" {
            run_cssparser(black_box(&sheet)).1
        } else {
            run_lexer(black_box(&sheet)).1
        };
    }
    black_box(total);
}
