//! Release-mode cost of the CSS fronts, with the volume each front counts.
//!
//! Every case runs `calls` calls per round over 11 rounds, and each round starts
//! one case later than the one before. A calibration pass outside the rounds
//! picks `calls` so that a round of one case takes about 100 ms (at least one
//! call). The table gives the median, minimum and maximum time of one call, and
//! the median time per input byte where the input has a size.
//!
//! The counted volume is read from the public result where one exists (rules,
//! diagnostics, retention steps, normalized items). Where a front counts work
//! against a public limit that its result does not report, the volume is the
//! smallest value of that limit at which the call still succeeds, found once
//! outside the timed region by doubling and bisection, the way the
//! characterization adapter finds charged work. A dimension that no public limit
//! reaches prints `-`. The parse, serialization, normalization and colour cases
//! "with a host" run the accounted front, and the retention case "with a host"
//! runs the walk through `charge_walk`, against a host operation with no caps;
//! their volume is what one call charges to that operation. The cases named
//! "profile ..." run one representative call of each front through a host at
//! each profile mode: off, summary, timed summary, and timed detail keeping its
//! first 4096 events (later events are counted and dropped). Run with
//! `cargo bench --offline --locked -j 1 -p surgeist-css --bench fronts`.

use std::hint::black_box;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use surgeist_css::accounting::{CssFront, CssHostScope, CssLedger, CssLimits, CssWork};
use surgeist_css::color::{
    CssAuthoredColorOrdinaryLimits, CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest,
    CssColorInput, CssColorOrdinaryLimits, CssColorOrdinaryOutcome, CssColorPropertyFacts,
    CssColorSchemeFact, CssDeviceColorFact, parse_used_color,
};
use surgeist_css::component::CssComponentValueLimits;
use surgeist_css::declaration::CssDeclarationList;
use surgeist_css::normalize::CssNormalizationLimits;
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::rule::{CssRule, CssSheet};
use surgeist_css::serialization::{
    CssRuleGraphInput, CssRuleSerializationLimits, CssSerializationLimits,
};
use surgeist_css::source::{CssSourceRetention, CssUtf16InputLimits};
use surgeist_css::values::background::CssBackgroundBox;
use surgeist_css::{CssParseReport, CssParser};
use surgeist_ledger::{EventFilter, Limits, Operation, ProfileMode};

const ROUNDS: usize = 11;
/// The time one case should take in one round, in nanoseconds.
const ROUND_TARGET_NS: f64 = 100e6;
/// Upper bound of every limit search.
const SEARCH_CEILING: usize = 1 << 40;

/// A front under measurement: its name, the bytes of its input, the volume it
/// counts, and the call.
struct Case {
    name: String,
    bytes: Option<usize>,
    counts: Vec<(&'static str, Option<usize>)>,
    run: Box<dyn FnMut() -> usize>,
}

impl Case {
    fn new(
        name: impl Into<String>,
        bytes: Option<usize>,
        counts: Vec<(&'static str, Option<usize>)>,
        run: impl FnMut() -> usize + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            bytes,
            counts,
            run: Box::new(run),
        }
    }
}

/// The smallest value in `0..=maximum` for which `accepts` holds, assuming that
/// acceptance is monotone; `None` when even `maximum` is refused.
fn smallest(maximum: usize, accepts: impl Fn(usize) -> bool) -> Option<usize> {
    if !accepts(maximum) {
        return None;
    }
    if accepts(0) {
        return Some(0);
    }
    let mut refused = 0;
    let mut probe = 1_usize;
    while probe < maximum && !accepts(probe) {
        refused = probe;
        probe = probe.saturating_mul(2);
    }
    let mut accepted = probe.min(maximum);
    while accepted - refused > 1 {
        let middle = refused + (accepted - refused) / 2;
        if accepts(middle) {
            accepted = middle;
        } else {
            refused = middle;
        }
    }
    Some(accepted)
}

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

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

/// A sheet of at least `minimum` bytes whose style rules hold only
/// declarations that normalization expands: shorthands, longhands and colours.
fn normalizable_sheet(minimum: usize) -> String {
    let mut sheet = String::with_capacity(minimum + 1024);
    let mut index = 0_usize;
    while sheet.len() < minimum {
        let n = 3 + index % 29;
        match index % 4 {
            0 => sheet.push_str(&format!(
                ".card-{index} > a:hover {{ color: #123456; margin: {n}px {n}em 0 auto; padding-block: 1px 2px; }}\n"
            )),
            1 => sheet.push_str(&format!(
                "@media (min-width: {}px) {{ .m{index} {{ width: calc(100% - {n}px); border-radius: {n}px {n}px 0 0; }} }}\n",
                n * 40
            )),
            2 => sheet.push_str(&format!(
                ".t{index} {{ color: red; margin-block: {n}px; margin-inline: 2px {n}px; display: block; }}\n"
            )),
            _ => sheet.push_str(&format!(
                "h{} {{ line-height: 1.{n}; letter-spacing: -0.0{n}em; --c{n}: {n}px; }}\n",
                1 + n % 6
            )),
        }
        index += 1;
    }
    sheet
}

/// A declaration list of `count` declarations of mixed shapes.
fn declaration_block(count: usize) -> String {
    let mut block = String::new();
    for index in 0..count {
        let n = 3 + index % 29;
        match index % 6 {
            0 => block.push_str(&format!("color: #{:06x}; ", 0x0010_0000 + index * 37 % 0x00ef_ffff)),
            1 => block.push_str(&format!("margin: {n}px {n}em 0 auto; ")),
            2 => block.push_str(&format!("background: url(images/bg-{index}.png) no-repeat; ")),
            3 => block.push_str(&format!("width: calc(100% - {n}px); ")),
            4 => block.push_str(&format!(
                "transition: opacity .{n}s ease-in-out, transform {n}ms cubic-bezier(.4, 0, .2, 1); "
            )),
            _ => block.push_str(&format!("line-height: 1.{n}; ")),
        }
    }
    block
}

/// The input of every CSSTree case of the stylesheet context, in file order and,
/// within a file, in case order. The case files under `expectations/stylesheet`
/// list one input per case; the source files nest several per entry.
fn csstree_stylesheet_sources() -> Vec<String> {
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/csstree/expectations/stylesheet");
    let mut files: Vec<_> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect();
    files.sort();
    let mut sources = Vec::new();
    for file in files {
        let bytes =
            std::fs::read(&file).unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        let document: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        let cases = document
            .get("cases")
            .and_then(serde_json::Value::as_array)
            .expect("a file of cases");
        sources.extend(
            cases
                .iter()
                .filter_map(|case| case.get("input").and_then(serde_json::Value::as_str))
                .map(str::to_owned),
        );
    }
    sources
}

fn parse_sheet(text: &str) -> CssParseReport<CssSheet> {
    CssParser::default().parse_sheet(text)
}

fn declarations(text: &str) -> CssParseReport<CssDeclarationList> {
    CssParser::default().parse_block_contents(text)
}

fn utf16(text: &str) -> Arc<Vec<u16>> {
    Arc::new(text.encode_utf16().collect())
}

fn units_parse(
    units: &Arc<Vec<u16>>,
    limits: CssUtf16InputLimits,
) -> Option<CssParseReport<CssSheet>> {
    CssParser::default()
        .parse_sheet_utf16(Arc::clone(units), limits, || Ok::<_, ()>(()))
        .ok()
}

// ---------------------------------------------------------------------------
// Cases
// ---------------------------------------------------------------------------

fn parse_cases(cases: &mut Vec<Case>) {
    let synthetic = synthetic_sheet(1 << 20);
    let bytes = synthetic.len();
    let report = parse_sheet(&synthetic);
    let rules = report.syntax().rules().len();
    let diagnostics = report.diagnostics().len();
    assert!(rules > 0);
    let text = synthetic.clone();
    cases.push(Case::new(
        "parse_sheet, 1 MiB synthetic sheet",
        Some(bytes),
        vec![
            ("rules", Some(rules)),
            ("diagnostics", Some(diagnostics)),
            ("component limit", None),
        ],
        move || {
            let report = parse_sheet(black_box(&text));
            let count = report.syntax().rules().len() + report.diagnostics().len();
            black_box(&report);
            count
        },
    ));

    let sources = csstree_stylesheet_sources();
    assert_eq!(
        sources.len(),
        76,
        "the CSSTree stylesheet context holds 76 cases"
    );
    let joined = sources.join("\n");
    let bytes = joined.len();
    let report = parse_sheet(&joined);
    let (rules, diagnostics) = (report.syntax().rules().len(), report.diagnostics().len());
    assert!(rules > 0);
    cases.push(Case::new(
        "parse_sheet, CSSTree stylesheet cases joined",
        Some(bytes),
        vec![
            ("rules", Some(rules)),
            ("diagnostics", Some(diagnostics)),
            ("component limit", None),
        ],
        move || {
            let report = parse_sheet(black_box(&joined));
            let count = report.syntax().rules().len() + report.diagnostics().len();
            black_box(&report);
            count
        },
    ));

    let bytes = sources.iter().map(String::len).sum::<usize>();
    let alone: Vec<(usize, usize)> = sources
        .iter()
        .map(|source| {
            let report = parse_sheet(source);
            (report.syntax().rules().len(), report.diagnostics().len())
        })
        .collect();
    let rules = alone.iter().map(|pair| pair.0).sum::<usize>();
    let diagnostics = alone.iter().map(|pair| pair.1).sum::<usize>();
    let calls = sources.len();
    cases.push(Case::new(
        format!("parse_sheet, {calls} CSSTree stylesheet cases, one call each"),
        Some(bytes),
        vec![
            ("rules", Some(rules)),
            ("diagnostics", Some(diagnostics)),
            ("component limit", None),
        ],
        move || {
            let mut count = 0;
            for source in &sources {
                let report = parse_sheet(black_box(source));
                count += report.syntax().rules().len() + report.diagnostics().len();
                black_box(&report);
            }
            count
        },
    ));

    // UTF-16 preparation counts against seven public limits.
    let units = utf16(&synthetic);
    let defaults = CssUtf16InputLimits::default();
    let baseline = {
        let report = units_parse(&units, defaults).expect("the default limits admit the input");
        (report.syntax().rules().len(), report.diagnostics().len())
    };
    type Field = fn(&mut CssUtf16InputLimits) -> &mut usize;
    let fields: [(&'static str, Field); 7] = [
        ("units", |limits| &mut limits.max_input_units),
        ("output bytes", |limits| &mut limits.max_output_bytes),
        ("map entries", |limits| &mut limits.max_map_entries),
        ("checkpoints", |limits| &mut limits.max_checkpoints),
        ("scratch bytes", |limits| &mut limits.max_scratch_bytes),
        ("retained bytes", |limits| &mut limits.max_retained_bytes),
        ("work", |limits| &mut limits.max_work),
    ];
    let mut counts: Vec<(&'static str, Option<usize>)> = vec![
        ("rules", Some(baseline.0)),
        ("diagnostics", Some(baseline.1)),
    ];
    for (name, field) in fields {
        let mut maximum = defaults;
        let ceiling = *field(&mut maximum);
        let value = smallest(ceiling, |value| {
            let mut limits = defaults;
            *field(&mut limits) = value;
            units_parse(&units, limits).is_some_and(|report| {
                (report.syntax().rules().len(), report.diagnostics().len()) == baseline
            })
        });
        counts.push((name, value));
    }
    cases.push(Case::new(
        "parse_sheet_utf16, 1 MiB synthetic sheet",
        Some(synthetic.len()),
        counts,
        move || {
            let report = units_parse(black_box(&units), defaults).expect("admitted");
            let count = report.syntax().rules().len() + report.diagnostics().len();
            black_box(&report);
            count
        },
    ));
}

/// The smallest component limits at which `text` still parses to `baseline`.
fn block_counts(text: &str, baseline: (usize, usize)) -> Vec<(&'static str, Option<usize>)> {
    let observe = |limits: Option<CssComponentValueLimits>| {
        limits.is_some_and(|limits| {
            let report = CssParser::new(Default::default(), limits).parse_block_contents(text);
            (report.syntax().len(), report.diagnostics().len()) == baseline
        })
    };
    let defaults = CssComponentValueLimits::default();
    let depth = smallest(defaults.max_nesting_depth() as usize, |depth| {
        observe(CssComponentValueLimits::try_new(
            u32::try_from(depth).unwrap(),
            defaults.max_components(),
            defaults.max_css_bytes(),
        ))
    });
    let components = smallest(SEARCH_CEILING, |components| {
        observe(CssComponentValueLimits::try_new(
            defaults.max_nesting_depth(),
            components,
            defaults.max_css_bytes(),
        ))
    });
    let bytes = smallest(SEARCH_CEILING, |bytes| {
        observe(CssComponentValueLimits::try_new(
            defaults.max_nesting_depth(),
            defaults.max_components(),
            bytes,
        ))
    });
    vec![
        ("declarations", Some(baseline.0)),
        ("diagnostics", Some(baseline.1)),
        ("depth", depth),
        ("components", components),
        ("css bytes", bytes),
    ]
}

fn block_cases(cases: &mut Vec<Case>) {
    for (name, text) in [
        (
            "parse_block_contents, 3000 declarations",
            declaration_block(3000),
        ),
        (
            "parse_block_contents, one declaration (color: red)",
            "color: red".to_owned(),
        ),
    ] {
        let report = declarations(&text);
        let baseline = (report.syntax().len(), report.diagnostics().len());
        assert!(baseline.0 > 0 && report.is_clean(), "{name}");
        let counts = block_counts(&text, baseline);
        cases.push(Case::new(name, Some(text.len()), counts, move || {
            let report = declarations(black_box(&text));
            let count = report.syntax().len() + report.diagnostics().len();
            black_box(&report);
            count
        }));
    }
}

/// A host operation without caps, and the CSS scopes below its root.
fn bench_host(mode: ProfileMode) -> (CssLedger, CssHostScope) {
    let limits: CssLimits = Limits::unlimited();
    let operation = Operation::with_profile("bench", limits, 1 + CssFront::ALL.len(), mode)
        .expect("the bench operation is created")
        .share();
    let host =
        CssHostScope::open(operation.clone(), operation.root()).expect("the CSS scopes are opened");
    (operation, host)
}

/// What one serialization call charges to a fresh host operation.
fn charged_serialization(
    serialize: impl FnOnce(&CssHostScope),
) -> Vec<(&'static str, Option<usize>)> {
    let (ledger, host) = bench_host(ProfileMode::Off);
    serialize(&host);
    let usage = ledger
        .usage(ledger.root())
        .expect("the root of the bench operation");
    let read = |dimension| usize::try_from(usage.work(dimension)).ok();
    vec![
        ("input nodes", read(CssWork::SerializedInputNode)),
        ("projection nodes", read(CssWork::SerializedProjectionNode)),
        ("output bytes", read(CssWork::SerializedByte)),
    ]
}

/// What one colour call charges to a fresh host operation.
fn charged_colour(resolve: impl FnOnce(&CssHostScope)) -> Vec<(&'static str, Option<usize>)> {
    let (ledger, host) = bench_host(ProfileMode::Off);
    resolve(&host);
    let usage = ledger
        .usage(ledger.root())
        .expect("the root of the bench operation");
    let read = |dimension| usize::try_from(usage.work(dimension)).ok();
    vec![
        ("input units", read(CssWork::ColorInputUnit)),
        ("actions", read(CssWork::ColorAction)),
    ]
}

/// What one call of `parse` charges to a fresh host operation.
fn charged_volume(parse: impl FnOnce(&CssHostScope)) -> Vec<(&'static str, Option<usize>)> {
    let (ledger, host) = bench_host(ProfileMode::Off);
    parse(&host);
    let usage = ledger
        .usage(ledger.root())
        .expect("the root of the bench operation");
    let read = |dimension| usize::try_from(usage.work(dimension)).ok();
    vec![
        ("input units", read(CssWork::InputUnit)),
        ("preparation steps", read(CssWork::PreparationStep)),
        ("documents", read(CssWork::Document)),
        ("components", read(CssWork::Component)),
        ("spelling bytes", read(CssWork::SpellingByte)),
    ]
}

/// The parse fronts called through a host handle. Each case has the input of
/// the plain case of the same name, so the two rows differ by the accounting.
fn accounted_cases(cases: &mut Vec<Case>) {
    let synthetic = synthetic_sheet(1 << 20);
    let counts = charged_volume(|host| {
        let _ = CssParser::default().accounted(host).parse_sheet(&synthetic);
    });
    let (_, host) = bench_host(ProfileMode::Off);
    cases.push(Case::new(
        "parse_sheet with a host, 1 MiB synthetic sheet",
        Some(synthetic.len()),
        counts,
        move || {
            let report = CssParser::default()
                .accounted(&host)
                .parse_sheet(black_box(&synthetic))
                .expect("a host without caps admits the charge");
            let count = report.syntax().rules().len() + report.diagnostics().len();
            black_box(&report);
            count
        },
    ));

    let units = utf16(&synthetic_sheet(1 << 20));
    let defaults = CssUtf16InputLimits::default();
    let accounted_units = move |host: &CssHostScope, units: &Arc<Vec<u16>>| {
        CssParser::default()
            .accounted(host)
            .parse_sheet_utf16(Arc::clone(units), defaults, || Ok::<_, ()>(()))
            .expect("a host without caps admits the charge")
            .expect("the default limits admit the sheet")
    };
    let counts = charged_volume(|host| {
        let _ = accounted_units(host, &units);
    });
    let (_, host) = bench_host(ProfileMode::Off);
    cases.push(Case::new(
        "parse_sheet_utf16 with a host, 1 MiB synthetic sheet",
        Some(units.len()),
        counts,
        move || {
            let report = accounted_units(&host, black_box(&units));
            let count = report.syntax().rules().len() + report.diagnostics().len();
            black_box(&report);
            count
        },
    ));

    {
        let text = "color: red";
        let short = utf16(text);
        let plain = short.clone();
        cases.push(Case::new(
            format!("parse_block_contents_utf16, one declaration ({text})"),
            Some(text.len()),
            vec![("declarations", Some(1))],
            move || {
                let report = CssParser::default()
                    .parse_block_contents_utf16(Arc::clone(black_box(&plain)), defaults, || {
                        Ok::<_, ()>(())
                    })
                    .expect("the default limits admit the declaration");
                let count = report.report().syntax().len() + report.report().diagnostics().len();
                black_box(&report);
                count
            },
        ));
        let accounted_block = move |host: &CssHostScope, units: &Arc<Vec<u16>>| {
            CssParser::default()
                .accounted(host)
                .parse_block_contents_utf16(Arc::clone(units), defaults, || Ok::<_, ()>(()))
                .expect("a host without caps admits the charge")
                .expect("the default limits admit the declaration")
        };
        let counts = charged_volume(|host| {
            let _ = accounted_block(host, &short);
        });
        let (_, host) = bench_host(ProfileMode::Off);
        cases.push(Case::new(
            format!("parse_block_contents_utf16 with a host, one declaration ({text})"),
            Some(text.len()),
            counts,
            move || {
                let report = accounted_block(&host, black_box(&short));
                let count = report.report().syntax().len() + report.report().diagnostics().len();
                black_box(&report);
                count
            },
        ));
    }

    let off = ProfileMode::Off;
    let timed = ProfileMode::Summary { timing: true };
    for (name, text, mode) in [
        (
            "parse_block_contents with a host, 3000 declarations",
            declaration_block(3000),
            off,
        ),
        (
            "parse_block_contents with a host, one declaration (color: red)",
            "color: red".to_owned(),
            off,
        ),
        (
            "parse_block_contents with a host and a timed summary, one declaration (color: red)",
            "color: red".to_owned(),
            timed,
        ),
    ] {
        let counts = charged_volume(|host| {
            let _ = CssParser::default()
                .accounted(host)
                .parse_block_contents(&text);
        });
        let (_, host) = bench_host(mode);
        cases.push(Case::new(name, Some(text.len()), counts, move || {
            let report = CssParser::default()
                .accounted(&host)
                .parse_block_contents(black_box(&text))
                .expect("a host without caps admits the charge");
            let count = report.syntax().len() + report.diagnostics().len();
            black_box(&report);
            count
        }));
    }
}

/// The smallest of each of the three serialization limits at which `serialize`
/// still succeeds.
fn serialization_counts<E>(
    serialize: impl Fn(CssSerializationLimits) -> Result<String, E>,
) -> Vec<(&'static str, Option<usize>)> {
    let wide = CssSerializationLimits::new(usize::MAX, usize::MAX, usize::MAX);
    let text = serialize(wide).unwrap_or_else(|_| panic!("the wide limits admit the output"));
    let input = smallest(SEARCH_CEILING, |value| {
        serialize(CssSerializationLimits::new(value, usize::MAX, usize::MAX)).is_ok()
    });
    let projection = smallest(SEARCH_CEILING, |value| {
        serialize(CssSerializationLimits::new(usize::MAX, value, usize::MAX)).is_ok()
    });
    let bytes = smallest(SEARCH_CEILING, |value| {
        serialize(CssSerializationLimits::new(usize::MAX, usize::MAX, value)).is_ok()
    });
    assert_eq!(
        bytes,
        Some(text.len()),
        "the byte limit is the output length"
    );
    vec![
        ("input nodes", input),
        ("projection nodes", projection),
        ("output bytes", bytes),
    ]
}

fn serialization_cases(cases: &mut Vec<Case>) {
    let wide = CssSerializationLimits::new(usize::MAX, usize::MAX, usize::MAX);

    // A keyword.
    let counts =
        serialization_counts(|limits| CssBackgroundBox::PaddingBox.serialize_specified(limits));
    cases.push(Case::new(
        "serialize_specified, keyword (CssBackgroundBox::PaddingBox)",
        None,
        counts,
        move || {
            let text = black_box(CssBackgroundBox::PaddingBox)
                .serialize_specified(wide)
                .expect("serialized");
            black_box(text).len()
        },
    ));

    // Declarations.
    for (name, source) in [
        (
            "serialize_specified, declaration (margin-block: 1px 2px)",
            "margin-block: 1px 2px",
        ),
        (
            "serialize_specified, declaration (transition list)",
            "transition: opacity .3s ease-in-out, transform 5ms cubic-bezier(.4, 0, .2, 1)",
        ),
    ] {
        let report = declarations(source);
        assert!(report.is_clean(), "{source}");
        let counts = serialization_counts(|limits| report.syntax()[0].serialize_specified(limits));
        cases.push(Case::new(name, Some(source.len()), counts, move || {
            let text = black_box(&report.syntax()[0])
                .serialize_specified(wide)
                .expect("serialized");
            black_box(text).len()
        }));
    }

    // A sheet, as specified text and as CSSOM text.
    let source = synthetic_sheet(256 * 1024);
    let report = parse_sheet(&source);
    let sheet: &'static CssParseReport<CssSheet> = Box::leak(Box::new(report));
    let counts = serialization_counts(|limits| sheet.syntax().serialize_specified(limits));
    cases.push(Case::new(
        "CssSheet::serialize_specified, 256 KiB synthetic sheet",
        Some(source.len()),
        counts,
        move || {
            let text = black_box(sheet.syntax())
                .serialize_specified(wide)
                .expect("serialized");
            black_box(text).len()
        },
    ));
    let counts = serialization_counts(|limits| sheet.syntax().serialize_cssom(limits));
    cases.push(Case::new(
        "CssSheet::serialize_cssom, 256 KiB synthetic sheet",
        Some(source.len()),
        counts,
        move || {
            let text = black_box(sheet.syntax())
                .serialize_cssom(wide)
                .expect("serialized");
            black_box(text).len()
        },
    ));

    // A rule graph.
    let rule_source = "@media (min-width: 40px) { .a { color: red; margin: 1px 2px 3px 4px } .b { width: calc(100% - 3px); transition: opacity .3s ease-in-out } }";
    let report = parse_sheet(rule_source);
    assert!(report.is_clean());
    let rule: &'static CssRule = Box::leak(Box::new(report.syntax().rules()[0].clone()));
    let graph_limits = |values| CssRuleSerializationLimits {
        values,
        ..Default::default()
    };
    let counts = serialization_counts(|values| {
        CssRuleGraphInput::from_rule(rule).serialize_specified(graph_limits(values))
    });
    cases.push(Case::new(
        "CssRuleGraphInput::from_rule(..).serialize_specified, @media rule",
        Some(rule_source.len()),
        counts,
        move || {
            let text = CssRuleGraphInput::from_rule(black_box(rule))
                .serialize_specified(graph_limits(wide))
                .expect("serialized");
            black_box(text).len()
        },
    ));

    // The same sheet and rule graph through a host handle.
    let counts = charged_serialization(|host| {
        let _ = sheet.syntax().accounted(host).serialize_cssom(wide);
    });
    let (_, host) = bench_host(ProfileMode::Off);
    cases.push(Case::new(
        "CssSheet::serialize_cssom with a host, 256 KiB synthetic sheet",
        Some(source.len()),
        counts,
        move || {
            let text = black_box(sheet.syntax())
                .accounted(&host)
                .serialize_cssom(wide)
                .expect("a host without caps admits the charge")
                .expect("serialized");
            black_box(text).len()
        },
    ));
    for (name, mode) in [
        (
            "CssRuleGraphInput::from_rule(..).serialize_specified with a host, @media rule",
            ProfileMode::Off,
        ),
        (
            "CssRuleGraphInput::from_rule(..).serialize_specified with a host and a timed summary, @media rule",
            ProfileMode::Summary { timing: true },
        ),
    ] {
        let counts = charged_serialization(|host| {
            let _ = CssRuleGraphInput::from_rule(rule)
                .accounted(host)
                .serialize_specified(graph_limits(wide));
        });
        let (_, host) = bench_host(mode);
        cases.push(Case::new(
            name,
            Some(rule_source.len()),
            counts,
            move || {
                let text = CssRuleGraphInput::from_rule(black_box(rule))
                    .accounted(&host)
                    .serialize_specified(graph_limits(wide))
                    .expect("a host without caps admits the charge")
                    .expect("serialized");
                black_box(text).len()
            },
        ));
    }
}

fn normalize_cases(cases: &mut Vec<Case>) {
    let source = normalizable_sheet(256 * 1024);
    let report = parse_sheet(&source);
    assert!(report.is_clean());
    let sheet: &'static CssSheet = Box::leak(Box::new(report.syntax().clone()));
    let normalized = sheet.normalize(Default::default()).expect("normalized");
    let items = normalized.items().len();
    let defaults = CssNormalizationLimits::default();
    let accepts = |limits: Option<CssNormalizationLimits>| {
        limits.is_some_and(|limits| sheet.normalize(limits).is_ok())
    };
    let depth = smallest(defaults.max_rule_depth() as usize, |depth| {
        accepts(CssNormalizationLimits::try_new(
            u32::try_from(depth).unwrap(),
            usize::MAX,
            usize::MAX,
            usize::MAX,
        ))
    });
    let rules = smallest(SEARCH_CEILING, |value| {
        accepts(CssNormalizationLimits::try_new(
            defaults.max_rule_depth(),
            value,
            usize::MAX,
            usize::MAX,
        ))
    });
    let declarations = smallest(SEARCH_CEILING, |value| {
        accepts(CssNormalizationLimits::try_new(
            defaults.max_rule_depth(),
            usize::MAX,
            value,
            usize::MAX,
        ))
    });
    let contributions = smallest(SEARCH_CEILING, |value| {
        accepts(CssNormalizationLimits::try_new(
            defaults.max_rule_depth(),
            usize::MAX,
            usize::MAX,
            value,
        ))
    });
    cases.push(Case::new(
        "CssSheet::normalize, 256 KiB normalizable sheet",
        Some(source.len()),
        vec![
            ("normalized items", Some(items)),
            ("rule depth", depth),
            ("rules", rules),
            ("declarations", declarations),
            ("contributions", contributions),
        ],
        move || {
            let normalized = black_box(sheet)
                .normalize(Default::default())
                .expect("normalized");
            let count = normalized.items().len();
            black_box(&normalized);
            count
        },
    ));

    // The same sheet through a host handle.
    let read_normalized = move |host: &CssHostScope| {
        black_box(sheet)
            .accounted(host)
            .normalize(Default::default())
            .expect("a host without caps admits the charge")
            .expect("normalized")
    };
    let (ledger, host) = bench_host(ProfileMode::Off);
    let _ = read_normalized(&host);
    let usage = ledger
        .usage(ledger.root())
        .expect("the root of the bench operation");
    let read = |dimension| usize::try_from(usage.work(dimension)).ok();
    let counts = vec![
        ("rules", read(CssWork::NormalizedRule)),
        ("declarations", read(CssWork::NormalizedDeclaration)),
        ("contributions", read(CssWork::NormalizedContribution)),
    ];
    let (_, host) = bench_host(ProfileMode::Off);
    cases.push(Case::new(
        "CssSheet::normalize with a host, 256 KiB normalizable sheet",
        Some(source.len()),
        counts,
        move || {
            let normalized = read_normalized(&host);
            let count = normalized.items().len();
            black_box(&normalized);
            count
        },
    ));
}

/// The inputs of the colour cases: the ordinary colours, numerals at the edges
/// of the exact window, and numerals outside it.
const COLOURS: [(&str, &str); 14] = [
    ("ordinary", "red"),
    ("ordinary", "rgb(10 20 30)"),
    ("ordinary", "color(srgb 1 0.5 0)"),
    ("edge of the exact window", "lab(0 3.4028234663852886e38 0)"),
    ("edge of the exact window", "lab(0 9.999999999999999e40 0)"),
    ("edge of the exact window", "lab(0 1e-46 0)"),
    ("edge of the exact window", "lab(0 1.401298464324817e-45 0)"),
    ("outside the exact window", "lab(0 1e41 0)"),
    ("outside the exact window", "lab(0 1e308 0)"),
    ("outside the exact window", "lab(0 1e999 0)"),
    ("outside the exact window", "lab(0 -1e999 0)"),
    ("outside the exact window", "lab(0 9.99e-47 0)"),
    ("outside the exact window", "lab(0 5e-324 0)"),
    ("outside the exact window", "lab(0 1e2000000 0)"),
];

fn colour_cases(cases: &mut Vec<Case>) {
    let facts: &'static CssColorFacts<'static> = Box::leak(Box::new(CssColorFacts::new(
        CssColorFactIdentity::new(1, 1),
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Light,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )));
    for (kind, text) in COLOURS {
        let bytes: &'static [u8] = text.as_bytes();
        let used = |limits: CssColorOrdinaryLimits| {
            parse_used_color(
                CssColorInput::Utf8(bytes),
                CssColorGrammarRequest::WholeColor,
                facts,
                facts.identity(),
                limits,
            )
        };
        let defaults = CssColorOrdinaryLimits::default();
        let result = used(defaults).expect("within limits");
        assert!(
            matches!(result.outcome(), CssColorOrdinaryOutcome::Used(_)),
            "{text}"
        );
        let wide = CssColorOrdinaryLimits {
            input_units: usize::MAX,
            algorithm_records: usize::MAX,
            arithmetic_records: usize::MAX,
            evaluation_records: usize::MAX,
            actions: usize::MAX,
        };
        let field = |set: fn(&mut CssColorOrdinaryLimits, usize)| {
            smallest(SEARCH_CEILING, |value| {
                let mut limits = wide;
                set(&mut limits, value);
                used(limits).is_ok()
            })
        };
        let counts = vec![
            ("input units", Some(text.len())),
            ("actions", field(|limits, value| limits.actions = value)),
            (
                "algorithm records",
                field(|limits, value| limits.algorithm_records = value),
            ),
            (
                "arithmetic records",
                field(|limits, value| limits.arithmetic_records = value),
            ),
            (
                "evaluation records",
                field(|limits, value| limits.evaluation_records = value),
            ),
        ];
        cases.push(Case::new(
            format!("parse_used_color, {kind}: {text}"),
            Some(text.len()),
            counts,
            move || {
                let result = used(black_box(wide)).expect("within limits");
                usize::from(matches!(result.outcome(), CssColorOrdinaryOutcome::Used(_)))
            },
        ));
        if kind == "ordinary" {
            let accounted = move |host: &CssHostScope, limits| {
                CssColorInput::Utf8(bytes)
                    .accounted(host)
                    .parse_used_color(
                        CssColorGrammarRequest::WholeColor,
                        facts,
                        facts.identity(),
                        limits,
                    )
                    .expect("a host without caps admits the charge")
                    .expect("within limits")
            };
            let counts = charged_colour(|host| {
                accounted(host, wide);
            });
            let (_, host) = bench_host(ProfileMode::Off);
            cases.push(Case::new(
                format!("parse_used_color with a host, {kind}: {text}"),
                Some(text.len()),
                counts,
                move || {
                    let result = accounted(&host, black_box(wide));
                    usize::from(matches!(result.outcome(), CssColorOrdinaryOutcome::Used(_)))
                },
            ));
        }

        // The authored colour of `color: <text>`.
        let report: &'static CssParseReport<CssDeclarationList> =
            Box::leak(Box::new(declarations(&format!("color: {text}"))));
        assert!(report.is_clean(), "{text}");
        let Some(CssKnownPropertyValueRef::Color(color)) = report.syntax()[0]
            .known()
            .and_then(|known| known.property_value())
        else {
            panic!("{text}: a colour property value");
        };
        let authored = |limits: CssAuthoredColorOrdinaryLimits| {
            color.value().resolve(facts, facts.identity(), limits)
        };
        let wide = CssAuthoredColorOrdinaryLimits {
            algorithm_records: usize::MAX,
            arithmetic_records: usize::MAX,
            evaluation_records: usize::MAX,
            path_steps: usize::MAX,
            actions: usize::MAX,
        };
        let result = authored(wide).expect("within limits");
        assert!(
            matches!(
                result.outcome(),
                surgeist_css::color::CssAuthoredColorOutcome::Used(_)
            ),
            "{text}"
        );
        let field = |set: fn(&mut CssAuthoredColorOrdinaryLimits, usize)| {
            smallest(SEARCH_CEILING, |value| {
                let mut limits = wide;
                set(&mut limits, value);
                authored(limits).is_ok()
            })
        };
        let counts = vec![
            ("actions", field(|limits, value| limits.actions = value)),
            (
                "algorithm records",
                field(|limits, value| limits.algorithm_records = value),
            ),
            (
                "arithmetic records",
                field(|limits, value| limits.arithmetic_records = value),
            ),
            (
                "evaluation records",
                field(|limits, value| limits.evaluation_records = value),
            ),
            (
                "path steps",
                field(|limits, value| limits.path_steps = value),
            ),
        ];
        cases.push(Case::new(
            format!("CssColor::resolve, {kind}: {text}"),
            Some(text.len()),
            counts,
            move || {
                let result = authored(black_box(wide)).expect("within limits");
                usize::from(matches!(
                    result.outcome(),
                    surgeist_css::color::CssAuthoredColorOutcome::Used(_)
                ))
            },
        ));
        if kind == "ordinary" {
            let value = color.value();
            let accounted = move |host: &CssHostScope, limits| {
                value
                    .accounted(host)
                    .resolve(facts, facts.identity(), limits)
                    .expect("a host without caps admits the charge")
                    .expect("within limits")
            };
            let counts = charged_colour(|host| {
                accounted(host, wide);
            });
            let (_, host) = bench_host(ProfileMode::Off);
            cases.push(Case::new(
                format!("CssColor::resolve with a host, {kind}: {text}"),
                Some(text.len()),
                counts,
                move || {
                    let result = accounted(&host, black_box(wide));
                    usize::from(matches!(
                        result.outcome(),
                        surgeist_css::color::CssAuthoredColorOutcome::Used(_)
                    ))
                },
            ));
        }
    }
}

fn retention_cases(cases: &mut Vec<Case>) {
    let source = synthetic_sheet(256 * 1024);
    let report: &'static CssParseReport<CssSheet> = Box::leak(Box::new(parse_sheet(&source)));
    let snapshot = report
        .syntax()
        .parse_source()
        .expect("a parsed sheet")
        .source();
    let bytes = snapshot.storage_usage().retained_bytes();
    let mut measured = CssSourceRetention::new(bytes, usize::MAX, 256);
    measured.visit(report.syntax()).expect("within limits");
    assert_eq!(measured.usage(), snapshot.storage_usage());
    let steps = measured.steps();
    assert!(steps > 0);
    // The walk at the smallest step limit still succeeds, and one below refuses.
    let exact = smallest(SEARCH_CEILING, |limit| {
        CssSourceRetention::new(bytes, limit, 256)
            .visit(report.syntax())
            .is_ok()
    });
    assert_eq!(exact, Some(steps));
    cases.push(Case::new(
        "CssSourceRetention::visit, parsed 256 KiB synthetic sheet",
        Some(source.len()),
        vec![("steps", Some(steps)), ("retained bytes", Some(bytes))],
        move || {
            let mut retention = CssSourceRetention::new(bytes, usize::MAX, 256);
            retention
                .visit(black_box(report.syntax()))
                .expect("within limits");
            retention.steps()
        },
    ));

    // The same walk through a host handle.
    let (_, host) = bench_host(ProfileMode::Off);
    cases.push(Case::new(
        "CssSourceRetention::visit with a host, parsed 256 KiB synthetic sheet",
        Some(source.len()),
        vec![("steps", Some(steps)), ("retained bytes", Some(bytes))],
        move || {
            let mut retention = CssSourceRetention::new(bytes, usize::MAX, 256);
            retention
                .charge_walk(&host, |retention| {
                    retention.visit(black_box(report.syntax()))
                })
                .expect("a host without caps admits the charge")
                .expect("within limits");
            retention.steps()
        },
    ));
}

/// One representative call of each front through a host at each profile mode.
fn profile_cases(cases: &mut Vec<Case>) {
    let modes = [
        ("off", ProfileMode::Off),
        ("summary", ProfileMode::Summary { timing: false }),
        ("timed summary", ProfileMode::Summary { timing: true }),
        (
            "timed detail",
            ProfileMode::Detailed {
                timing: true,
                event_limit: 4096,
                filter: EventFilter::All,
            },
        ),
    ];
    let large: &'static str = Box::leak(synthetic_sheet(1 << 20).into_boxed_str());
    let medium: &'static str = Box::leak(synthetic_sheet(256 * 1024).into_boxed_str());
    let sheet: &'static CssSheet = Box::leak(Box::new(parse_sheet(medium).into_parts().0));
    let normalizable: &'static CssSheet = Box::leak(Box::new(
        parse_sheet(&normalizable_sheet(256 * 1024)).into_parts().0,
    ));
    let facts: &'static CssColorFacts<'static> = Box::leak(Box::new(CssColorFacts::new(
        CssColorFactIdentity::new(1, 1),
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Light,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )));
    let wide = CssSerializationLimits::new(usize::MAX, usize::MAX, usize::MAX);
    for (label, mode) in modes {
        let (_, host) = bench_host(mode);
        let host: &'static CssHostScope = Box::leak(Box::new(host));
        cases.push(Case::new(
            format!("profile {label}: parse_sheet with a host, 1 MiB synthetic sheet"),
            Some(large.len()),
            Vec::new(),
            move || {
                let report = CssParser::default()
                    .accounted(host)
                    .parse_sheet(black_box(large))
                    .expect("a host without caps admits the charge");
                report.syntax().rules().len()
            },
        ));
        cases.push(Case::new(
            format!(
                "profile {label}: parse_block_contents with a host, one declaration (color: red)"
            ),
            Some("color: red".len()),
            Vec::new(),
            move || {
                let report = CssParser::default()
                    .accounted(host)
                    .parse_block_contents(black_box("color: red"))
                    .expect("a host without caps admits the charge");
                report.syntax().len()
            },
        ));
        cases.push(Case::new(
            format!(
                "profile {label}: CssSheet::serialize_cssom with a host, 256 KiB synthetic sheet"
            ),
            Some(medium.len()),
            Vec::new(),
            move || {
                sheet
                    .accounted(host)
                    .serialize_cssom(black_box(wide))
                    .expect("a host without caps admits the charge")
                    .expect("the wide limits admit the sheet")
                    .len()
            },
        ));
        cases.push(Case::new(
            format!("profile {label}: CssSheet::normalize with a host, 256 KiB normalizable sheet"),
            None,
            Vec::new(),
            move || {
                let normalized = normalizable
                    .accounted(host)
                    .normalize(black_box(CssNormalizationLimits::default()))
                    .expect("a host without caps admits the charge")
                    .expect("the default limits admit the sheet");
                black_box(&normalized);
                1
            },
        ));
        cases.push(Case::new(
            format!("profile {label}: parse_used_color with a host, ordinary: red"),
            Some(3),
            Vec::new(),
            move || {
                let result = CssColorInput::Utf8(b"red")
                    .accounted(host)
                    .parse_used_color(
                        CssColorGrammarRequest::WholeColor,
                        facts,
                        facts.identity(),
                        black_box(CssColorOrdinaryLimits::default()),
                    )
                    .expect("a host without caps admits the charge")
                    .expect("within limits");
                usize::from(matches!(result.outcome(), CssColorOrdinaryOutcome::Used(_)))
            },
        ));
        cases.push(Case::new(
            format!("profile {label}: CssSourceRetention::visit with a host, parsed 256 KiB synthetic sheet"),
            Some(medium.len()),
            Vec::new(),
            move || {
                let mut retention = CssSourceRetention::new(usize::MAX, usize::MAX, 256);
                retention
                    .charge_walk(host, |retention| retention.visit(black_box(sheet)))
                    .expect("a host without caps admits the charge")
                    .expect("within limits");
                retention.steps()
            },
        ));
    }
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Whether a counted dimension is a volume that work is charged per unit of,
/// as opposed to a nesting depth or a fixed register that a call does not
/// repeat.
fn is_volume(name: &str) -> bool {
    !matches!(
        name,
        "depth" | "rule depth" | "path steps" | "scratch bytes"
    )
}

fn main() {
    let mut cases = Vec::new();
    parse_cases(&mut cases);
    block_cases(&mut cases);
    accounted_cases(&mut cases);
    serialization_cases(&mut cases);
    normalize_cases(&mut cases);
    colour_cases(&mut cases);
    retention_cases(&mut cases);
    profile_cases(&mut cases);

    // Calibrate outside the rounds: the first call warms, the next three set the
    // number of calls per round, and the result of every call is checked
    // against the first.
    let mut expected = Vec::new();
    let mut calls = Vec::new();
    for case in &mut cases {
        let first = (case.run)();
        let start = Instant::now();
        for _ in 0..3 {
            assert_eq!(
                (case.run)(),
                first,
                "{}: a call is deterministic",
                case.name
            );
        }
        let one = start.elapsed().as_secs_f64() * 1e9 / 3.0;
        expected.push(first);
        calls.push(((ROUND_TARGET_NS / one.max(1.0)) as usize).clamp(1, 1_000_000));
    }

    let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(ROUNDS); cases.len()];
    for round in 0..ROUNDS {
        for step in 0..cases.len() {
            let index = (round + step) % cases.len();
            let case = &mut cases[index];
            let start = Instant::now();
            let mut sum = 0_usize;
            for _ in 0..calls[index] {
                sum = sum.wrapping_add(black_box((case.run)()));
            }
            let elapsed = start.elapsed().as_secs_f64() * 1e9;
            assert_eq!(
                sum,
                expected[index].wrapping_mul(calls[index]),
                "{}: every call gave the same result",
                case.name
            );
            times[index].push(elapsed / calls[index] as f64);
        }
    }

    println!(
        "{ROUNDS} rounds, each starting one case later; calls per round are calibrated to about 100 ms"
    );
    println!(
        "case\tmedian ns/call\tmin ns/call\tmax ns/call\tcalls/round\tinput bytes\tmedian ns/byte\tcounted volume\tmedian ns per counted unit"
    );
    for (index, case) in cases.iter().enumerate() {
        let mut sorted = times[index].clone();
        sorted.sort_by(f64::total_cmp);
        let median = sorted[ROUNDS / 2];
        let (bytes, per_byte) = case.bytes.map_or_else(
            || ("-".to_owned(), "-".to_owned()),
            |bytes| (bytes.to_string(), format!("{:.3}", median / bytes as f64)),
        );
        let volume = case
            .counts
            .iter()
            .map(|(name, value)| {
                format!(
                    "{name}={}",
                    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let per_unit = case
            .counts
            .iter()
            .map(|(name, value)| match value {
                Some(value) if *value > 0 && is_volume(name) => {
                    format!("{name}={:.3}", median / *value as f64)
                }
                _ => format!("{name}=-"),
            })
            .collect::<Vec<_>>()
            .join("; ");
        println!(
            "{}\t{median:.0}\t{:.0}\t{:.0}\t{}\t{bytes}\t{per_byte}\t{volume}\t{per_unit}",
            case.name,
            sorted[0],
            sorted[ROUNDS - 1],
            calls[index],
        );
    }
}
