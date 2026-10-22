//! Public source-only Preserve behavior over exact authored UTF-16 units.
//! CSS Text4 §4 defines document-language breaks and CR-as-space; §4.3.1
//! preserves tabs/spaces and represents a preserved segment break as one LF.
use surgeist_text::{
    DocumentSegmentBreaks, Id, Range, Source, Style, Text, WhiteSpace, process_source_whitespace,
};

fn assert_original(source: &Source, id: Id, revision: u64, units: &[u16]) {
    assert_eq!(source.id(), Some(id));
    assert_eq!(source.revision(), revision);
    assert_eq!(source.text().code_units(), units);
}

#[test]
fn preserve_normalizes_only_defined_breaks_and_keeps_cr_tab_and_delimiter_distinctions() {
    const ORIGINAL: &[u16] = &[0x0041, 0x000d, 0x000a, 0x0042, 0x000d, 0x0009, 0x0043];
    let id = Id::from_u64(721);
    let original = Source::identified(Text::from_code_units(ORIGINAL.to_vec()), id, 37);
    let cases: [(&str, DocumentSegmentBreaks, &[u16]); 4] = [
        (
            "document CRLF is one preserved LF",
            DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
            &[0x0041, 0x000a, 0x0042, 0x0020, 0x0009, 0x0043],
        ),
        (
            "document single CR is LF; unclassified CR is space",
            DocumentSegmentBreaks::Defined(vec![Range::new(4, 5)]),
            &[0x0041, 0x0020, 0x000a, 0x0042, 0x000a, 0x0009, 0x0043],
        ),
        (
            "LF fallback does not silently absorb a preceding CR",
            DocumentSegmentBreaks::LineFeedDefault,
            &[0x0041, 0x0020, 0x000a, 0x0042, 0x0020, 0x0009, 0x0043],
        ),
        (
            "explicitly no delimiters remains distinct from LF fallback",
            DocumentSegmentBreaks::Defined(vec![]),
            &[0x0041, 0x0020, 0x000a, 0x0042, 0x0020, 0x0009, 0x0043],
        ),
    ];
    let mut refusals = Vec::new();
    for (label, breaks, expected) in cases {
        let outcome = process_source_whitespace(
            original.clone(),
            Style {
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
            breaks.clone(),
        );
        // A rejected operation also cannot rewrite the caller's retained source.
        assert_original(&original, id, 37, ORIGINAL);
        match outcome {
            Ok(processed) => {
                assert_eq!(processed.processed_text().code_units(), expected, "{label}");
                assert_original(processed.source(), id, 37, ORIGINAL);
                // Preserve emits the same LF code unit for the final two rows,
                // but the actual captured document classification is different.
                assert_eq!(processed.segment_breaks(), &breaks, "{label}");
            }
            Err(error) => refusals.push(format!("{label}: {error:?}")),
        }
    }
    assert!(
        refusals.is_empty(),
        "Preserve must execute: {}",
        refusals.join("\n")
    );
}

#[test]
fn preserve_keeps_isolated_and_paired_units_nbsp_and_other_space_separators_exact() {
    const ORIGINAL: &[u16] = &[
        0xd800, 0x000d, 0x000a, 0xd83d, 0xde00, 0x00a0, 0x2002, 0x2003, 0x202f, 0x3000, 0xdc00,
        0x0009, 0x0020,
    ];
    const EXPECTED: &[u16] = &[
        0xd800, 0x000a, 0xd83d, 0xde00, 0x00a0, 0x2002, 0x2003, 0x202f, 0x3000, 0xdc00, 0x0009,
        0x0020,
    ];
    let id = Id::from_u64(722);
    let original = Source::identified(Text::from_code_units(ORIGINAL.to_vec()), id, 91);
    let breaks = DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]);
    let outcome = process_source_whitespace(
        original.clone(),
        Style {
            white_space: WhiteSpace::Preserve,
            ..Style::default()
        },
        breaks.clone(),
    );
    assert_original(&original, id, 91, ORIGINAL);
    let processed =
        outcome.expect("source-only Preserve admits isolated units without shaping or repair");
    assert_eq!(processed.processed_text().code_units(), EXPECTED);
    assert_original(processed.source(), id, 91, ORIGINAL);
    assert_eq!(processed.segment_breaks(), &breaks);
}
