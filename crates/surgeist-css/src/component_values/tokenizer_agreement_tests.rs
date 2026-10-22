//! CSS Syntax 3 defines one tokenization (§4) of one preprocessed input stream
//! (§3.3). Component values reach consumers through three construction paths:
//! direct collection from the cssparser provider, promotion from the private
//! syntax-consumption arena, and the crate's bounded tokenizer over
//! preprocessed scalars. Each must produce the same values for equivalent UTF-8
//! and UTF-16 input.
//!
//! Equivalence follows §3.3. A UTF-16 input and its UTF-8 transcoding are the
//! same code points. The UTF-16 and decoded-UTF-8 fronts retain the
//! preprocessed stream as their snapshot text, so their values, spellings and
//! spans equal direct collection over the preprocessed text. Direct collection
//! over raw text retains authored spellings and offsets, so it agrees on decoded
//! token identity. A lone surrogate has no UTF-8 form; §3.3 replaces it with
//! U+FFFD, so its equivalent UTF-8 input is that preprocessed text.

use std::sync::Arc;

use super::*;
use crate::utf16_value::Control;
use crate::{CssUtf16InputLimits, CssUtf16PropertyValueLimits};

type Outcome = Result<CssComponentValues, CssComponentValueError>;

/// §3.3 written independently of the crate's input cursor: surrogate code
/// points and NULL become U+FFFD; CRLF, CR and FF become LF.
fn preprocess(units: &[u16]) -> String {
    let decoded: String = char::decode_utf16(units.iter().copied())
        .map(|point| point.unwrap_or('\u{fffd}'))
        .collect();
    let mut output = String::with_capacity(decoded.len());
    let mut points = decoded.chars().peekable();
    while let Some(point) = points.next() {
        match point {
            '\r' => {
                if points.peek() == Some(&'\n') {
                    points.next();
                }
                output.push('\n');
            }
            '\u{c}' => output.push('\n'),
            '\0' => output.push('\u{fffd}'),
            point => output.push(point),
        }
    }
    output
}

fn preprocess_text(text: &str) -> String {
    preprocess(&text.encode_utf16().collect::<Vec<_>>())
}

fn limits() -> CssUtf16PropertyValueLimits {
    CssUtf16PropertyValueLimits {
        input: CssUtf16InputLimits::default(),
        syntax: CssComponentValueLimits::default(),
        max_work: usize::MAX,
        max_live_bytes: usize::MAX,
        max_retained_bytes: usize::MAX,
    }
}

fn controlled<T>(operation: impl FnOnce(&Control<'_>) -> T) -> T {
    let admit = |_| Ok(());
    let release = |_| {};
    let control = Control::new(limits(), &admit, &release);
    operation(&control)
}

fn direct(text: &str) -> Outcome {
    parse_component_values(text)
}

fn arena(text: &str) -> Outcome {
    let document = crate::syntax_consumption::source_document_with_limits(
        text,
        CssComponentValueLimits::default(),
    )?;
    promote_nodes(
        &document,
        &document.lists[document.root],
        CssComponentValueLimits::default(),
    )
}

fn utf16_snapshot(units: &[u16]) -> CssSourceSnapshot {
    crate::utf16_input::prepare_utf16(
        Arc::new(units.to_vec()),
        CssUtf16InputLimits::default(),
        || Ok::<_, ()>(()),
        |_, _| Ok(()),
        |_, _| {},
    )
    .expect("UTF-16 preparation within default limits")
}

fn arena_utf16(units: &[u16]) -> Outcome {
    let snapshot = utf16_snapshot(units);
    let document = crate::syntax_consumption::source_document_snapshot_with_limits(
        &snapshot,
        CssComponentValueLimits::default(),
    )?;
    promote_nodes(
        &document,
        &document.lists[document.root],
        CssComponentValueLimits::default(),
    )
}

fn bounded_utf16(units: &[u16]) -> Outcome {
    let snapshot = utf16_snapshot(units);
    controlled(|control| {
        parse_controlled_snapshot(&snapshot, CssComponentValueLimits::default(), control)
    })
}

fn bounded_decoded_utf8(text: &str) -> Outcome {
    controlled(|control| {
        let snapshot = CssSourceSnapshot::try_from_decoded_utf8(text, control)
            .expect("decoded UTF-8 preparation within unbounded limits");
        parse_controlled_snapshot(&snapshot, CssComponentValueLimits::default(), control)
    })
}

/// Decoded token identity and structure, without authored spellings or spans.
/// Comment contents are authored text, so they compare after §3.3.
fn assert_same_tokenization(raw: &Outcome, preprocessed: &Outcome, case: &str) {
    match (raw, preprocessed) {
        (Ok(raw), Ok(preprocessed)) => {
            let mut pending = vec![(raw.items(), preprocessed.items())];
            while let Some((left, right)) = pending.pop() {
                assert_eq!(left.len(), right.len(), "component count: {case:?}");
                for (left, right) in left.iter().zip(right) {
                    match (&left.data, &right.data) {
                        (ComponentData::Token(a), ComponentData::Token(b)) => {
                            assert_eq!(a.data, b.data, "token: {case:?}");
                            assert_eq!(
                                a.implicit_end.is_some(),
                                b.implicit_end.is_some(),
                                "implicit token closure: {case:?}"
                            );
                        }
                        (ComponentData::Function(a), ComponentData::Function(b)) => {
                            assert_eq!(a.name, b.name, "function name: {case:?}");
                            assert_eq!(
                                matches!(a.closing.origin, CssValueOrigin::ImplicitClosure { .. }),
                                matches!(b.closing.origin, CssValueOrigin::ImplicitClosure { .. }),
                                "implicit function closure: {case:?}"
                            );
                            pending.push((a.values.items(), b.values.items()));
                        }
                        (ComponentData::Block(a), ComponentData::Block(b)) => {
                            assert_eq!(a.kind, b.kind, "block kind: {case:?}");
                            assert_eq!(
                                matches!(a.closing.origin, CssValueOrigin::ImplicitClosure { .. }),
                                matches!(b.closing.origin, CssValueOrigin::ImplicitClosure { .. }),
                                "implicit block closure: {case:?}"
                            );
                            pending.push((a.values.items(), b.values.items()));
                        }
                        (
                            ComponentData::Comment {
                                content: a,
                                implicit_end: a_end,
                                ..
                            },
                            ComponentData::Comment {
                                content: b,
                                implicit_end: b_end,
                                ..
                            },
                        ) => {
                            assert_eq!(preprocess_text(a), **b, "comment: {case:?}");
                            assert_eq!(a_end.is_some(), b_end.is_some(), "comment end: {case:?}");
                        }
                        (left, right) => {
                            panic!("component kind differs for {case:?}: {left:?} / {right:?}")
                        }
                    }
                }
            }
        }
        (Err(raw), Err(preprocessed)) => {
            assert_eq!(raw.kind(), preprocessed.kind(), "failure: {case:?}");
        }
        (raw, preprocessed) => {
            panic!("outcomes differ for {case:?}: {raw:?} / {preprocessed:?}")
        }
    }
}

/// Every path over one input. `utf8` is the input's UTF-8 form, when it has one.
fn assert_paths_agree(utf8: Option<&str>, utf16: &[u16]) {
    let preprocessed = preprocess(utf16);
    let case = utf8.unwrap_or(&preprocessed);
    let expected = direct(&preprocessed);
    assert_eq!(
        arena(&preprocessed),
        expected,
        "arena over preprocessed text: {case:?}"
    );
    assert_eq!(bounded_utf16(utf16), expected, "bounded UTF-16: {case:?}");
    assert_eq!(arena_utf16(utf16), expected, "arena UTF-16: {case:?}");
    assert_eq!(
        bounded_decoded_utf8(&preprocessed),
        expected,
        "bounded decoded UTF-8 over preprocessed text: {case:?}"
    );
    if let Some(text) = utf8 {
        let raw = direct(text);
        assert_eq!(arena(text), raw, "arena over raw text: {case:?}");
        assert_eq!(
            bounded_decoded_utf8(text),
            expected,
            "bounded decoded UTF-8 over raw text: {case:?}"
        );
        assert_same_tokenization(&raw, &expected, case);
    }
}

fn assert_text_paths_agree(text: &str) {
    assert_paths_agree(Some(text), &text.encode_utf16().collect::<Vec<_>>());
}

fn corpus_inputs() -> Vec<String> {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/csstree/expectations");
    let mut directories = vec![root];
    let mut inputs = Vec::new();
    while let Some(directory) = directories.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(&directory)
            .expect("corpus expectation directory")
            .map(|entry| entry.expect("corpus entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                if path.file_name() != Some("generation-reports".as_ref()) {
                    directories.push(path);
                }
            } else if path.extension() == Some("json".as_ref()) {
                let document: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&path).expect("corpus expectation"))
                        .expect("corpus expectation JSON");
                for case in document["cases"].as_array().expect("corpus cases") {
                    inputs.push(case["input"].as_str().expect("corpus input").to_owned());
                }
            }
        }
    }
    inputs
}

#[test]
fn every_csstree_corpus_input_has_one_tokenization_across_construction_paths() {
    let inputs = corpus_inputs();
    // The README states the pinned corpus size.
    assert_eq!(inputs.len(), 935);
    for input in &inputs {
        assert_text_paths_agree(input);
    }
}

#[test]
fn escapes_have_one_tokenization_across_construction_paths() {
    for text in [
        "\\72 gb(1 2 3)",
        "\\72gb(1)",
        "a\\",
        "\\",
        "@a\\",
        "#a\\",
        "#\\",
        "1\\",
        "1e\\",
        "a\\\\",
        "\"a\\",
        "'a\\",
        "\"a\\\\",
        "url(a\\",
        "url(\\",
        "url(a\\\\",
        "\\0 x",
        "\\d800 x",
        "\\dfff",
        "\\110000",
        "\\10ffff",
        "\\000061b",
        "\\0000061",
        "\\31 23",
        "\\31\t23",
        "#\\31 23",
        "#-\\31",
        "#\\",
        "@\\6d edia",
        "@\\",
        "a\\\nb",
        "\\\n",
        "\"a\\\nb\"",
        "'a\\'b'",
        "u\\72l(x)",
        "\\75rl( a )",
        "URL(\\29)",
        "url(\\\\)",
        "-\\31",
        "--\\31",
        "1\\70x",
        "1e\\31",
        "\\-",
        "\\\\",
        "f\\(x)",
    ] {
        assert_text_paths_agree(text);
    }
}

#[test]
fn unicode_has_one_tokenization_across_construction_paths() {
    for text in [
        "é",
        "😀",
        "--😀",
        "#😀",
        "@😀",
        "1😀",
        "a\u{a0}b",
        "\u{2603}",
        "\u{fffe}",
        "\u{10ffff}",
        "\u{fffd}",
        "a\u{feff}b",
        "\"😀\"",
        "url(😀)",
        "f😀(é)",
        "\\1f600 x",
    ] {
        assert_text_paths_agree(text);
    }
}

#[test]
fn utf16_surrogates_have_one_tokenization_across_construction_paths() {
    const HIGH: u16 = 0xd83d;
    const LOW: u16 = 0xde00;
    let quote = u16::from(b'"');
    let a = u16::from(b'a');
    let b = u16::from(b'b');
    let url: Vec<u16> = "url(".encode_utf16().collect();
    for units in [
        vec![HIGH, LOW],
        vec![HIGH],
        vec![LOW],
        vec![LOW, HIGH],
        vec![HIGH, HIGH, LOW],
        vec![a, HIGH, b],
        vec![a, LOW, b],
        vec![quote, HIGH, quote],
        vec![quote, LOW],
        [url.as_slice(), &[HIGH, u16::from(b')')]].concat(),
        [url.as_slice(), &[a, LOW]].concat(),
        vec![u16::from(b'#'), HIGH],
        vec![u16::from(b'@'), LOW, a],
        vec![u16::from(b'1'), HIGH],
        vec![u16::from(b'\\'), HIGH],
        vec![u16::from(b'/'), u16::from(b'*'), LOW],
    ] {
        assert_paths_agree(None, &units);
    }
}

#[test]
fn bad_urls_have_one_tokenization_across_construction_paths() {
    for text in [
        "url(a b)",
        "url(a\"b)",
        "url(a'b)",
        "url(a(b)",
        "url(a\u{7f}b)",
        "url(a\u{1}b)",
        "url(a\u{b}b)",
        "url(a\\\nb)",
        "url(a b",
        "url(a\"\\)",
        "url(a\"\\29 ",
        "url(a\"x\\)y)tail",
        "url(a\"x\\29 y)tail",
        "url( a \\)",
        "url( a \\\\)",
        "url(a \\\\\\) b)",
        "url(a b\\) c) d",
        "x url(a b) y",
        "f(url(a b))",
        "url(a\nb)",
    ] {
        assert_text_paths_agree(text);
    }
}

#[test]
fn newlines_have_one_tokenization_across_construction_paths() {
    for text in [
        "a\r\nb",
        "a\rb",
        "a\u{c}b",
        "a\r\r\nb",
        "a\n\rb",
        "\r",
        "\r\n",
        "\u{c}",
        "a\\\r\nb",
        "a\\\rb",
        "a\\\u{c}b",
        "\"a\\\r\nb\"",
        "\"a\\\rb\"",
        "\"a\\\u{c}b\"",
        "\"a\r\nb\"",
        "'a\rb'",
        "\"a\u{c}b\"",
        "/*\r\n*/",
        "/*\r*/x",
        "/*\u{c}",
        "url(\r\na\r\n)",
        "url(\ra\u{c})",
        "url(a\r\nb)",
        "url(a \\\r\n)",
        "\\31\r\nx",
        "\\31\rx",
        "\\31\u{c}x",
        "url(\\31\r\nx)",
        "f(\r\n)",
        "url(\r\n\"v\")",
    ] {
        assert_text_paths_agree(text);
    }
}

#[test]
fn null_has_one_tokenization_across_construction_paths() {
    for text in [
        "\0",
        "a\0b",
        "\"\0\"",
        "'a\0",
        "url(\0)",
        "url(a\0b)",
        "url(\0",
        "#\0",
        "@\0",
        "-\0",
        "1\0",
        "/*\0*/",
        "\\\0",
        "f\0(x)",
    ] {
        assert_text_paths_agree(text);
    }
}

#[test]
fn numbers_delimiters_and_cdo_cdc_have_one_tokenization_across_construction_paths() {
    for text in [
        "<!-- -->",
        "<!--",
        "-->",
        "--a",
        "-",
        "@-",
        "1-",
        "#-",
        "#",
        "+.5e-3px",
        "1e+x",
        "1e",
        "-0",
        ".5%",
        "1.",
        "a|=b ~= c ^= d $= e *= f",
        "u+0-7f",
        "(a [b {c",
        "a:b;c,d",
        "f(g(h(",
    ] {
        assert_text_paths_agree(text);
    }
}

/// Spec-derived anchors, so the paths cannot agree on a wrong tokenization.
#[test]
fn preprocessed_inputs_decode_to_the_syntax_3_tokens() {
    let ident = |values: &CssComponentValues, index: usize| match values.items()[index].view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => name.to_owned(),
        other => panic!("expected an identifier: {other:?}"),
    };
    // §3.3: a lone surrogate and NULL become U+FFFD, an ident code point (§4.2).
    let lone = bounded_utf16(&[u16::from(b'a'), 0xd800, u16::from(b'b')]).unwrap();
    assert_eq!(lone.items().len(), 1);
    assert_eq!(ident(&lone, 0), "a\u{fffd}b");
    let null = direct("a\0b").unwrap();
    assert_eq!(null.items().len(), 1);
    assert_eq!(ident(&null, 0), "a\u{fffd}b");
    // §3.3 then §4.3.7: CRLF is one newline, so one whitespace terminates the
    // hexadecimal escape and `x` continues the identifier.
    for values in [
        direct("\\31\r\nx").unwrap(),
        bounded_utf16(&"\\31\r\nx".encode_utf16().collect::<Vec<_>>()).unwrap(),
    ] {
        assert_eq!(values.items().len(), 1);
        assert_eq!(ident(&values, 0), "1x");
    }
    // §3.3: CR, FF and CRLF are each one newline; one whitespace token results.
    for text in ["a\r\nb", "a\rb", "a\u{c}b"] {
        let values = bounded_utf16(&text.encode_utf16().collect::<Vec<_>>()).unwrap();
        assert_eq!(values.items().len(), 3, "{text:?}");
        assert!(matches!(
            values.items()[1].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace("\n"))
        ));
    }
}
