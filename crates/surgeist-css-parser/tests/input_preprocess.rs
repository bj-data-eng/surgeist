//! Tests for `input::preprocess_utf16`, `input::filter_code_points` and the
//! `SourceMap` (CSS Syntax 3 section 3.3, "Preprocessing the input stream").
//!
//! The expectations come from section 3.3 of the pinned edition: a CR, an FF
//! and a CR LF pair each become one LF, and a NUL and every surrogate code
//! point become U+FFFD (here, an unpaired surrogate unit, the only way UTF-16
//! holds a surrogate code point). The reference reading below is written from
//! those sentences and from UTF-16's definition of a pair, independently of
//! the cursor that the function runs.

mod common;

use std::cell::{Cell, RefCell};
use std::mem::size_of;

use surgeist_css_parser::input::{
    InputCursor, PreprocessError, PreprocessLimits, PreprocessResource, Preprocessed, SourceMap,
    filter_code_points, preprocess_utf16,
};
use surgeist_css_parser::limits::{Failure, Meter, Refused};

// ---- Meters -----------------------------------------------------------------

/// One call a preprocessing made on its meter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Call {
    Step,
    Add(usize),
    Released(usize),
    Note(Failure),
}

use Call::{Add, Note, Released, Step};

/// A meter that records every call and refuses the fallible call (a `step` or
/// an `add`, counted from zero in the order made) at one chosen index.
#[derive(Default)]
struct Recording {
    calls: RefCell<Vec<Call>>,
    fallible: Cell<usize>,
    refuse_at: Option<usize>,
    /// The bytes of a refused `add`, which was never charged.
    refused_bytes: Cell<usize>,
}

impl Recording {
    fn admitting() -> Self {
        Self::default()
    }

    fn refusing_call(index: usize) -> Self {
        Self {
            refuse_at: Some(index),
            ..Self::default()
        }
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.borrow().clone()
    }

    fn steps(&self) -> usize {
        self.calls().iter().filter(|call| **call == Step).count()
    }

    fn fallible_calls(&self) -> usize {
        self.fallible.get()
    }

    /// Bytes added and not yet released.
    fn outstanding(&self) -> usize {
        let mut live = 0_usize;
        let refused = self.refused_bytes.get();
        for call in self.calls() {
            match call {
                Add(bytes) => live += bytes,
                Released(bytes) => {
                    live = live
                        .checked_sub(bytes)
                        .expect("only charged bytes are released");
                }
                Step | Note(_) => {}
            }
        }
        live.checked_sub(refused)
            .expect("a refused add is one of the adds")
    }

    fn admit(&self, call: Call) -> Result<(), Refused> {
        self.calls.borrow_mut().push(call);
        let index = self.fallible.get();
        self.fallible.set(index + 1);
        if self.refuse_at == Some(index) {
            Err(Refused)
        } else {
            Ok(())
        }
    }
}

impl Meter for Recording {
    fn step(&self) -> Result<(), Refused> {
        self.admit(Step)
    }

    fn add(&self, bytes: usize) -> Result<(), Refused> {
        let result = self.admit(Add(bytes));
        if result.is_err() {
            self.refused_bytes.set(bytes);
        }
        result
    }

    fn released(&self, bytes: usize) {
        self.calls.borrow_mut().push(Released(bytes));
    }

    fn note(&self, failure: Failure) -> Refused {
        self.calls.borrow_mut().push(Note(failure));
        Refused
    }
}

fn preprocess(units: &[u16]) -> Preprocessed {
    preprocess_utf16(units, &PreprocessLimits::default(), &Recording::admitting())
        .expect("preprocessing within the default limits succeeds")
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

// ---- The reference reading of section 3.3 -----------------------------------

/// One scalar of the reference reading: its value and the original units it
/// came from.
struct Expected {
    value: char,
    units: std::ops::Range<usize>,
}

/// Reads `units` as section 3.3 says, one decision per sentence of the
/// specification: pair the surrogates UTF-16 pairs, turn the others into
/// U+FFFD, then replace CR LF, CR, FF and NUL.
fn reference(units: &[u16]) -> Vec<Expected> {
    let mut scalars = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        let next = units.get(index + 1).copied();
        let (value, length) = match unit {
            0xd800..=0xdbff if matches!(next, Some(0xdc00..=0xdfff)) => {
                let high = u32::from(unit) - 0xd800;
                let low = u32::from(next.unwrap()) - 0xdc00;
                (char::from_u32(0x10000 + (high << 10) + low).unwrap(), 2)
            }
            // A surrogate code point that is not part of a pair.
            0xd800..=0xdfff => ('\u{fffd}', 1),
            0x0d if next == Some(0x0a) => ('\n', 2),
            0x0d | 0x0c => ('\n', 1),
            0x00 => ('\u{fffd}', 1),
            unit => (char::from_u32(u32::from(unit)).unwrap(), 1),
        };
        scalars.push(Expected {
            value,
            units: index..index + length,
        });
        index += length;
    }
    scalars
}

/// Asserts that `result` is the reference reading of `units`: the text, and a
/// map that sends every run of scalars to the units it replaced.
fn assert_reads_as_section_3_3(units: &[u16], result: &Preprocessed) {
    let expected = reference(units);
    let text: String = expected.iter().map(|scalar| scalar.value).collect();
    assert_eq!(result.text(), text, "text of {units:x?}");
    assert_eq!(result.map().scalar_count(), expected.len(), "{units:x?}");
    assert_eq!(result.map().original_units(), units.len(), "{units:x?}");
    // The offsets of the scalar boundaries in the normalized text.
    let mut boundaries = vec![0];
    for scalar in &expected {
        boundaries.push(boundaries.last().unwrap() + scalar.value.len_utf8());
    }
    assert_eq!(*boundaries.last().unwrap(), result.text().len());
    for start in 0..expected.len() {
        for end in start..=expected.len() {
            let original_start = expected.get(start).map_or(units.len(), |s| s.units.start);
            let original_end = expected.get(end).map_or(units.len(), |s| s.units.start);
            assert_eq!(
                result
                    .map()
                    .original_utf16(boundaries[start]..boundaries[end]),
                Some(original_start..original_end),
                "scalars {start}..{end} of {units:x?}"
            );
        }
    }
    // An offset inside a scalar is not a boundary.
    for (index, scalar) in expected.iter().enumerate() {
        for inside in 1..scalar.value.len_utf8() {
            let offset = boundaries[index] + inside;
            assert_eq!(result.map().original_utf16(offset..offset), None);
            assert_eq!(result.map().original_utf16(0..offset), None);
        }
    }
    // Outside the text, or backwards.
    let length = result.text().len();
    assert_eq!(result.map().original_utf16(0..length + 1), None);
    assert_eq!(result.map().original_utf16(length + 1..length + 1), None);
    if length > 0 {
        assert_eq!(result.map().original_utf16(length..length - 1), None);
    }
}

// ---- Section 3.3 over UTF-16 units ------------------------------------------

#[test]
fn empty_input_is_empty_text_with_an_empty_map() {
    let result = preprocess(&[]);
    assert_eq!(result.text(), "");
    assert_eq!(result.map().scalar_count(), 0);
    assert_eq!(result.map().original_utf16(0..0), Some(0..0));
    assert_eq!(result.map().capacity(), 0);
    assert_eq!(result.text_capacity(), 0);
}

#[test]
fn ordinary_units_are_one_scalar_each_with_their_own_interval() {
    let result = preprocess(&utf16("a\u{e9}\u{20ac}"));
    assert_eq!(result.text(), "a\u{e9}\u{20ac}");
    assert_eq!(result.map().original_utf16(0..1), Some(0..1));
    assert_eq!(result.map().original_utf16(1..3), Some(1..2));
    assert_eq!(result.map().original_utf16(3..6), Some(2..3));
    assert_eq!(result.map().original_utf16(0..6), Some(0..3));
}

#[test]
fn a_carriage_return_a_form_feed_and_a_pair_each_become_one_line_feed() {
    // CR; FF; CR LF are three line feeds; a lone LF stays one.
    for (units, text, ranges) in [
        (vec![0x0d], "\n", vec![(0, 1)]),
        (vec![0x0c], "\n", vec![(0, 1)]),
        (vec![0x0d, 0x0a], "\n", vec![(0, 2)]),
        (vec![0x0a], "\n", vec![(0, 1)]),
        // CR CR LF is a CR and then a pair; LF CR is two scalars.
        (vec![0x0d, 0x0d, 0x0a], "\n\n", vec![(0, 1), (1, 3)]),
        (vec![0x0a, 0x0d], "\n\n", vec![(0, 1), (1, 2)]),
        // FF LF is not a pair: the form feed and the line feed are one each.
        (vec![0x0c, 0x0a], "\n\n", vec![(0, 1), (1, 2)]),
        // LF CR LF: the CR pairs with the second LF.
        (vec![0x0a, 0x0d, 0x0a], "\n\n", vec![(0, 1), (1, 3)]),
        // A CR at the very end is a line feed, not lost.
        (vec![0x61, 0x0d], "a\n", vec![(0, 1), (1, 2)]),
    ] {
        let result = preprocess(&units);
        assert_eq!(result.text(), text, "{units:x?}");
        let mut start = 0;
        for (from, to) in ranges {
            // Each scalar here is the one byte of a line feed.
            let end = start + 1;
            assert_eq!(
                result.map().original_utf16(start..end),
                Some(from..to),
                "{units:x?}"
            );
            start = end;
        }
        assert_reads_as_section_3_3(&units, &result);
    }
}

#[test]
fn a_null_becomes_a_replacement_character_that_covers_its_one_unit() {
    let result = preprocess(&[0x61, 0x00, 0x62]);
    assert_eq!(result.text(), "a\u{fffd}b");
    assert_eq!(result.map().original_utf16(1..4), Some(1..2));
    assert_eq!(preprocess(&[0, 0, 0]).text(), "\u{fffd}\u{fffd}\u{fffd}");
}

#[test]
fn a_surrogate_pair_is_one_scalar_covering_both_units() {
    // U+1F600 is D83D DE00; U+10000 is D800 DC00; U+10FFFF is DBFF DFFF.
    for (units, scalar) in [
        (vec![0xd83d, 0xde00], '\u{1f600}'),
        (vec![0xd800, 0xdc00], '\u{10000}'),
        (vec![0xdbff, 0xdfff], '\u{10ffff}'),
    ] {
        let result = preprocess(&units);
        assert_eq!(result.text(), scalar.to_string());
        assert_eq!(result.map().original_utf16(0..4), Some(0..2));
        assert_eq!(result.map().scalar_count(), 1);
    }
}

#[test]
fn an_unpaired_surrogate_unit_becomes_a_replacement_character() {
    // A low surrogate alone; a high surrogate alone, at the end and before an
    // ordinary unit; two highs and then a low; a low and then a high.
    for (units, text) in [
        (vec![0xdc00], "\u{fffd}"),
        (vec![0xd800], "\u{fffd}"),
        (vec![0x61, 0xd800], "a\u{fffd}"),
        (vec![0xd800, 0x61], "\u{fffd}a"),
        (vec![0xd800, 0xd800, 0xdc00], "\u{fffd}\u{10000}"),
        (vec![0xdc00, 0xd800], "\u{fffd}\u{fffd}"),
        (vec![0xdc00, 0xdc00], "\u{fffd}\u{fffd}"),
        (vec![0xdfff, 0xd800, 0xdc00], "\u{fffd}\u{10000}"),
    ] {
        let result = preprocess(&units);
        assert_eq!(result.text(), text, "{units:x?}");
        assert_reads_as_section_3_3(&units, &result);
    }
}

#[test]
fn a_high_surrogate_before_a_carriage_return_does_not_swallow_it() {
    // The unit after an unpaired high surrogate is processed as itself.
    let result = preprocess(&[0xd800, 0x0d, 0x0a, 0x61]);
    assert_eq!(result.text(), "\u{fffd}\na");
    assert_eq!(result.map().original_utf16(0..3), Some(0..1));
    assert_eq!(result.map().original_utf16(3..4), Some(1..3));
    let result = preprocess(&[0xd800, 0x00]);
    assert_eq!(result.text(), "\u{fffd}\u{fffd}");
    assert_eq!(result.map().original_utf16(3..6), Some(1..2));
}

#[test]
fn a_mixed_input_maps_each_scalar_to_the_original_units_it_replaced() {
    // CR LF, FF, NUL, a surrogate pair, a high surrogate, "a", a low surrogate:
    // seven scalars written by hand from section 3.3 as (bytes of the
    // normalized text, original units).
    let units = [13, 10, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00];
    let result = preprocess(&units);
    assert_eq!(result.text(), "\n\n\u{fffd}\u{1f600}\u{fffd}a\u{fffd}");
    assert_eq!(result.map().scalar_count(), 7);
    for (bytes, original) in [
        (0..1, 0..2),
        (1..2, 2..3),
        (2..5, 3..4),
        (5..9, 4..6),
        (9..12, 6..7),
        (12..13, 7..8),
        (13..16, 8..9),
        (16..16, 9..9),
    ] {
        assert_eq!(
            result.map().original_utf16(bytes.clone()),
            Some(original),
            "{bytes:?}"
        );
    }
    assert_eq!(result.map().original_utf16(0..16), Some(0..9));
}

#[test]
fn every_short_sequence_of_interesting_units_reads_as_section_3_3() {
    let alphabet: [u16; 9] = [0x61, 0x0d, 0x0a, 0x0c, 0x00, 0xd83d, 0xde00, 0xd800, 0xe9];
    let mut units = Vec::new();
    fn visit(alphabet: &[u16], units: &mut Vec<u16>, depth: usize) {
        let result = preprocess(units);
        assert_reads_as_section_3_3(units, &result);
        if depth == 0 {
            return;
        }
        for unit in alphabet {
            units.push(*unit);
            visit(alphabet, units, depth - 1);
            units.pop();
        }
    }
    visit(&alphabet, &mut units, 4);
}

// ---- filter_code_points ------------------------------------------------------

#[test]
fn filtering_replaces_carriage_returns_form_feeds_pairs_and_nulls() {
    for (text, filtered) in [
        ("", ""),
        ("a", "a"),
        ("\r", "\n"),
        ("\x0c", "\n"),
        ("\r\n", "\n"),
        ("\r\r\n", "\n\n"),
        ("\n\r", "\n\n"),
        ("\x0c\n", "\n\n"),
        ("a\r\nb\rc\x0cd\0e", "a\nb\nc\nd\u{fffd}e"),
        ("\0\0", "\u{fffd}\u{fffd}"),
        ("\u{e9}\r\u{1f600}\0", "\u{e9}\n\u{1f600}\u{fffd}"),
    ] {
        assert_eq!(filter_code_points(text), filtered, "{text:?}");
    }
}

#[test]
fn text_with_nothing_to_filter_is_returned_borrowed() {
    for text in [
        "",
        "a b\nc",
        "\u{e9}\u{1f600}",
        "a\u{fffd}b",
        "\u{85}\u{2028}",
    ] {
        assert!(
            matches!(filter_code_points(text), std::borrow::Cow::Borrowed(_)),
            "{text:?}"
        );
    }
    for text in ["\r", "\x0c", "\0"] {
        assert!(
            matches!(filter_code_points(text), std::borrow::Cow::Owned(_)),
            "{text:?}"
        );
    }
}

#[test]
fn filtering_a_string_gives_the_text_that_preprocessing_its_units_gives() {
    // Over every text of up to five characters of an alphabet with every case,
    // the two implementations of section 3.3 agree.
    let alphabet = ['a', '\r', '\n', '\x0c', '\0', '\u{e9}', '\u{1f600}'];
    let mut text = String::new();
    fn visit(alphabet: &[char], text: &mut String, depth: usize) {
        let result = preprocess(&text.encode_utf16().collect::<Vec<_>>());
        assert_eq!(filter_code_points(text), result.text(), "{text:?}");
        if depth == 0 {
            return;
        }
        for point in alphabet {
            text.push(*point);
            visit(alphabet, text, depth - 1);
            text.pop();
        }
    }
    visit(&alphabet, &mut text, 5);
}

// ---- Tokenizing the preprocessed text ----------------------------------------

#[test]
fn lexing_the_preprocessed_text_gives_the_tokens_of_the_text_and_maps_back_exactly() {
    // The lexer applies section 3.3 itself over UTF-8, so the two readings of
    // the same text must agree: the tokens equal, and each token's range in
    // the normalized text maps to original units whose filtered text is the
    // token's own text.
    let texts = [
        "a { color: red }",
        "a\r\n{\x0ccolor:\r\nred\0}",
        "\"a\r\nb\" 'c\x0cd'",
        "url(  a\0b  ) \u{e9}\u{1f600}x /* \r\n */ \\\r\nz",
        "@media\rscreen{\u{fffd}}",
        "#\0a .\u{1f600} 1e3 1\r\n2",
    ];
    for text in texts {
        let units = utf16(text);
        let result = preprocess(&units);
        let from_units = common::lex_all(result.text()).0;
        let from_text = common::lex_all(text).0;
        let rendered = |items: &[common::Item]| -> Vec<String> {
            items.iter().map(|item| item.token.clone()).collect()
        };
        assert_eq!(rendered(&from_units), rendered(&from_text), "{text:?}");
        for item in &from_units {
            let original = result
                .map()
                .original_utf16(item.range.clone())
                .expect("a lexeme ends on scalar boundaries");
            let original_text = String::from_utf16(&units[original]).expect("whole scalars");
            assert_eq!(
                filter_code_points(&original_text),
                result.text()[item.range.clone()],
                "{text:?} token {item:?}"
            );
        }
    }
}

// ---- Charging ----------------------------------------------------------------

#[test]
fn an_empty_preprocessing_charges_the_registers_the_end_and_their_release() {
    // Hand-derived. The registers are the cursor, a String and a Vec of 16-byte
    // entries. There is one cursor request (the end), so one step. The cursor
    // is released at the end; the String and Vec headers stay with the result.
    let meter = Recording::admitting();
    let result = preprocess_utf16(&[], &PreprocessLimits::default(), &meter).unwrap();
    let registers =
        size_of::<InputCursor>() + size_of::<String>() + size_of::<Vec<(usize, usize)>>();
    assert_eq!(
        meter.calls(),
        [Add(registers), Step, Released(size_of::<InputCursor>())]
    );
    assert_eq!(
        result.charged_bytes(),
        size_of::<String>() + size_of::<Vec<(usize, usize)>>()
    );
    assert_eq!(meter.outstanding(), result.charged_bytes());
}

#[test]
fn a_success_leaves_exactly_the_charged_bytes_of_the_result_with_the_meter() {
    for text in ["a", "color: red", "a\r\nb\0\u{1f600}", &"x".repeat(300)] {
        let meter = Recording::admitting();
        let result = preprocess_utf16(&utf16(text), &PreprocessLimits::default(), &meter).unwrap();
        // The text and the map each hold their capacity and one header.
        let expected = result.text_capacity()
            + size_of::<String>()
            + result.map().capacity() * 16
            + size_of::<Vec<(usize, usize)>>();
        assert_eq!(result.charged_bytes(), expected, "{text:?}");
        assert_eq!(meter.outstanding(), expected, "{text:?}");
        assert!(meter.calls().iter().all(|call| !matches!(call, Note(_))));
    }
}

#[test]
fn the_text_and_the_map_grow_by_doubling_from_one() {
    // Hand-derived for 5 ASCII units: scalars arrive one at a time, so the text
    // grows 1, 2, 4, 8 and the map grows 1, 2, 4, 8 (when length equals
    // capacity, twice the capacity or one).
    let meter = Recording::admitting();
    let result = preprocess_utf16(&utf16("abcde"), &PreprocessLimits::default(), &meter).unwrap();
    assert_eq!(result.text_capacity(), 8);
    assert_eq!(result.map().capacity(), 8);
}

#[test]
fn the_work_charged_is_the_requests_plus_the_elements_moved() {
    // Hand-derived for 64 ASCII units. Each unit costs two requests (read, then
    // emit) and the input ends with one: 129. The text grows at the 1st, 2nd,
    // 3rd, 5th, 9th, 17th and 33rd scalar, moving 0+1+2+4+8+16+32 = 63
    // scalars; the map moves the same 63 entries.
    let meter = Recording::admitting();
    preprocess_utf16(
        &utf16(&"a".repeat(64)),
        &PreprocessLimits::default(),
        &meter,
    )
    .unwrap();
    assert_eq!(meter.steps(), 129 + 63 + 63);
}

// ---- Limits ------------------------------------------------------------------

fn limited(
    units: &[u16],
    limits: PreprocessLimits,
) -> (Result<Preprocessed, PreprocessError>, Recording) {
    let meter = Recording::admitting();
    let result = preprocess_utf16(units, &limits, &meter);
    (result, meter)
}

fn exhausted(result: &Result<Preprocessed, PreprocessError>) -> Option<PreprocessResource> {
    result
        .as_ref()
        .err()
        .and_then(PreprocessError::exhausted_limit)
}

#[test]
fn each_limit_accepts_its_exact_size_and_refuses_one_less() {
    // 64 ASCII units: 64 output bytes, 64 map entries, text and map capacity 64
    // each (the doubling reaches 64 exactly), 64 + 64 * 16 = 1088 retained
    // bytes, and 255 steps of work (129 requests and 126 moves).
    let units = utf16(&"a".repeat(64));
    let exact = PreprocessLimits::default()
        .with_max_input_units(64)
        .with_max_output_bytes(64)
        .with_max_map_entries(64)
        .with_max_scratch_bytes(PreprocessLimits::SCRATCH_BYTES)
        .with_max_retained_bytes(1088)
        .with_max_work(255);
    let (result, meter) = limited(&units, exact);
    let result = result.expect("the exact limits accept the input");
    assert_eq!(result.text_capacity(), 64);
    assert_eq!(result.map().capacity(), 64);
    assert_eq!(meter.steps(), 255);
    for (limits, resource) in [
        (
            exact.with_max_input_units(63),
            PreprocessResource::InputUnits,
        ),
        (
            exact.with_max_output_bytes(63),
            PreprocessResource::OutputBytes,
        ),
        (
            exact.with_max_map_entries(63),
            PreprocessResource::MapEntries,
        ),
        (
            exact.with_max_scratch_bytes(PreprocessLimits::SCRATCH_BYTES - 1),
            PreprocessResource::ScratchBytes,
        ),
        (
            exact.with_max_retained_bytes(1087),
            PreprocessResource::RetainedBytes,
        ),
        (exact.with_max_work(254), PreprocessResource::Work),
    ] {
        let (result, meter) = limited(&units, limits);
        assert_eq!(exhausted(&result), Some(resource), "{resource:?}");
        assert!(result.as_ref().unwrap_err().is_limit());
        assert_eq!(
            meter.outstanding(),
            0,
            "{resource:?} leaves nothing charged"
        );
    }
}

#[test]
fn the_scratch_limit_is_the_size_of_the_input_cursor() {
    assert_eq!(PreprocessLimits::SCRATCH_BYTES, size_of::<InputCursor>());
    let limits = PreprocessLimits::default();
    assert!(limits.max_scratch_bytes() >= PreprocessLimits::SCRATCH_BYTES);
}

#[test]
fn capacity_growth_is_clamped_to_the_output_and_map_limits() {
    // 40 units with a limit of 40 would double 32 to 64; the limit clamps it to
    // exactly 40, so the input fits.
    let units = utf16(&"a".repeat(40));
    let limits = PreprocessLimits::default()
        .with_max_output_bytes(40)
        .with_max_map_entries(40);
    let (result, _) = limited(&units, limits);
    let result = result.expect("a clamped growth accepts the input");
    assert_eq!(result.text_capacity(), 40);
    assert_eq!(result.map().capacity(), 40);
    // One more unit than the limit is refused where it would be stored.
    let units = utf16(&"a".repeat(41));
    let (result, meter) = limited(&units, limits);
    assert_eq!(exhausted(&result), Some(PreprocessResource::OutputBytes));
    assert_eq!(meter.outstanding(), 0);
    let (result, meter) = limited(&units, limits.with_max_output_bytes(100));
    assert_eq!(exhausted(&result), Some(PreprocessResource::MapEntries));
    assert_eq!(meter.outstanding(), 0);
}

#[test]
fn the_output_limit_counts_normalized_bytes_not_input_units() {
    // CR LF is two units and one byte; a surrogate pair is two units and four.
    let (result, _) = limited(
        &[0x0d, 0x0a, 0x0d, 0x0a],
        PreprocessLimits::default().with_max_output_bytes(2),
    );
    assert_eq!(result.unwrap().text(), "\n\n");
    let (result, _) = limited(
        &[0xd83d, 0xde00],
        PreprocessLimits::default().with_max_output_bytes(3),
    );
    assert_eq!(exhausted(&result), Some(PreprocessResource::OutputBytes));
    let (result, _) = limited(
        &[0xd83d, 0xde00],
        PreprocessLimits::default().with_max_output_bytes(4),
    );
    assert_eq!(result.unwrap().text(), "\u{1f600}");
}

#[test]
fn work_is_charged_for_the_moves_of_a_growth_that_the_limit_interrupts() {
    // For every work limit up to the total, the result is either complete or a
    // work refusal, and a refusal always leaves nothing charged, wherever in
    // the movement of a growth it happens.
    let units = utf16("a\r\nb\u{1f600}\0cde");
    let total = {
        let meter = Recording::admitting();
        preprocess_utf16(&units, &PreprocessLimits::default(), &meter).unwrap();
        meter.steps()
    };
    for work in 0..=total {
        let (result, meter) = limited(&units, PreprocessLimits::default().with_max_work(work));
        if work < total {
            assert_eq!(exhausted(&result), Some(PreprocessResource::Work), "{work}");
            assert_eq!(meter.outstanding(), 0, "work limit {work}");
        } else {
            assert!(result.is_ok());
        }
    }
}

// ---- Refusal by the meter ------------------------------------------------------

#[test]
fn a_refusal_at_any_call_stops_the_preprocessing_and_releases_everything() {
    // Refuse each fallible call (step or add) of a run in turn, for inputs that
    // grow storage several times and for the empty input.
    for text in ["", "ab", "a\r\nb\0\u{1f600}c", &"y".repeat(40)] {
        let units = utf16(text);
        let counting = Recording::admitting();
        preprocess_utf16(&units, &PreprocessLimits::default(), &counting).unwrap();
        let total = counting.fallible_calls();
        for index in 0..total {
            let meter = Recording::refusing_call(index);
            let error = preprocess_utf16(&units, &PreprocessLimits::default(), &meter)
                .expect_err("a refusal fails the whole preprocessing");
            assert!(error.is_refused(), "{text:?} call {index}");
            assert!(!error.is_limit() && !error.is_allocation_failure());
            assert!(!error.is_capacity_overflow());
            assert_eq!(error.exhausted_limit(), None);
            assert_eq!(meter.outstanding(), 0, "{text:?} call {index} leaks");
            assert_eq!(
                meter.fallible_calls(),
                index + 1,
                "no call after the refusal"
            );
        }
    }
}

// ---- Linear work -----------------------------------------------------------------

#[test]
fn preprocessing_work_grows_linearly_with_the_input() {
    // The preparation is a linear walk. Each unit costs two requests (a read
    // and an emit). Storage doubles, so the elements moved over all growths of
    // the text sum to less than its final capacity, which is less than twice
    // the scalar count, and the same holds for the map: at most
    // 2 + 2 + 2 = 6 steps per unit whatever the input's length, where a walk
    // that repeated itself per statement would grow with the length.
    let per_unit = |count: usize| {
        let meter = Recording::admitting();
        preprocess_utf16(&vec![0x61; count], &PreprocessLimits::default(), &meter).unwrap();
        meter.steps() as f64 / count as f64
    };
    for count in [1_000, 10_000, 100_000] {
        let steps = per_unit(count);
        assert!(steps <= 6.0, "{count} units cost {steps:.2} steps each");
        assert!(steps >= 4.0, "{count} units cost {steps:.2} steps each");
    }
}

// ---- Errors and limit values ---------------------------------------------------

#[test]
fn a_preprocess_error_names_its_cause_and_is_an_error() {
    let (result, _) = limited(
        &utf16("ab"),
        PreprocessLimits::default().with_max_output_bytes(1),
    );
    let error = result.unwrap_err();
    assert_eq!(
        error.to_string(),
        "UTF-16 preprocessing normalized UTF-8 bytes limit exceeded"
    );
    fn is_error<E: std::error::Error + Send + Sync + 'static>(_: &E) {}
    is_error(&error);
    let (result, _) = limited(
        &utf16("ab"),
        PreprocessLimits::default().with_max_input_units(1),
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "UTF-16 preprocessing input UTF-16 units limit exceeded"
    );
    let meter = Recording::refusing_call(0);
    let error = preprocess_utf16(&[], &PreprocessLimits::default(), &meter).unwrap_err();
    assert_eq!(
        error.to_string(),
        "UTF-16 preprocessing was refused by the meter"
    );
}

#[test]
fn the_resources_describe_themselves() {
    for (resource, text) in [
        (PreprocessResource::InputUnits, "input UTF-16 units"),
        (PreprocessResource::OutputBytes, "normalized UTF-8 bytes"),
        (PreprocessResource::MapEntries, "source map entries"),
        (
            PreprocessResource::ScratchBytes,
            "preprocessing scratch bytes",
        ),
        (
            PreprocessResource::RetainedBytes,
            "retained text and map bytes",
        ),
        (PreprocessResource::Work, "preprocessing work"),
    ] {
        assert_eq!(resource.as_str(), text);
    }
}

#[test]
fn limits_report_what_they_were_built_with_and_default_to_finite_values() {
    let limits = PreprocessLimits::default()
        .with_max_input_units(1)
        .with_max_output_bytes(2)
        .with_max_map_entries(3)
        .with_max_scratch_bytes(4)
        .with_max_retained_bytes(5)
        .with_max_work(6);
    assert_eq!(
        (
            limits.max_input_units(),
            limits.max_output_bytes(),
            limits.max_map_entries(),
            limits.max_scratch_bytes(),
            limits.max_retained_bytes(),
            limits.max_work()
        ),
        (1, 2, 3, 4, 5, 6)
    );
    let defaults = PreprocessLimits::default();
    assert_eq!(defaults.max_input_units(), 16 * 1024 * 1024);
    assert_eq!(defaults.max_output_bytes(), 16 * 1024 * 1024);
    assert_eq!(defaults.max_map_entries(), 16 * 1024 * 1024);
    assert_eq!(defaults.max_scratch_bytes(), 1024);
    assert_eq!(defaults.max_retained_bytes(), 512 * 1024 * 1024);
    assert_eq!(defaults.max_work(), 128 * 1024 * 1024);
    assert_eq!(limits, limits);
    assert_ne!(limits, defaults);
}

#[test]
fn the_default_limits_accept_the_default_scratch_registers() {
    assert!(PreprocessLimits::SCRATCH_BYTES <= PreprocessLimits::default().max_scratch_bytes());
}

#[test]
fn a_source_map_is_a_value_that_compares_and_is_taken_apart_with_its_text() {
    let result = preprocess(&utf16("a\r\nb"));
    let copy = result.clone();
    assert_eq!(result, copy);
    let (text, map): (String, SourceMap) = result.into_parts();
    assert_eq!(text, "a\nb");
    assert_eq!(map.original_utf16(0..3), Some(0..4));
}

#[test]
fn the_map_charges_its_capacity_times_sixteen_bytes_beside_the_text_and_two_headers() {
    for text in ["", "a", "a\r\nb", "é😀x", &"abc".repeat(50)] {
        let result = preprocess(&utf16(text));
        assert_eq!(result.map().capacity_bytes(), result.map().capacity() * 16);
        assert_eq!(
            result.charged_bytes(),
            result.text_capacity()
                + std::mem::size_of::<String>()
                + result.map().capacity_bytes()
                + std::mem::size_of::<Vec<(usize, usize)>>(),
            "{text:?}"
        );
    }
}
