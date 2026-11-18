#![forbid(unsafe_code)]

//! Messages of the font, geometry, bidi and hyphenation error enums.
//!
//! Every variant is public, so each table constructs all of them. A row states
//! the documented meaning of one variant; the `index` functions are exhaustive
//! matches without a fallback arm, so a new variant does not compile until it
//! has a row. Only `FontMetricTableError::BaselineOutline` wraps another error.

use surgeist_text::{
    BidiLevelError, FontMetricNumericError, FontMetricTable, FontMetricTableError,
    FontOutlineCapability, FontOutlineError, HyphenationError, SuppliedLineError, TextNumericError,
};

mod error_support;
use error_support::{check_table, check_wrapper};

fn index_table(error: &FontMetricTableError) -> usize {
    use FontMetricTableError as E;
    match error {
        E::Missing => 0,
        E::MissingCompanion { .. } => 1,
        E::LongMetricCount { .. } => 2,
        E::Duplicate => 3,
        E::Inaccessible => 4,
        E::Malformed => 5,
        E::UnsupportedVersion => 6,
        E::VariationWithoutAxes => 7,
        E::UnsupportedRecordSize { .. } => 8,
        E::VariationRecordOrder { .. } => 9,
        E::VariationOffset { .. } => 10,
        E::VariationStoreFormat { .. } => 11,
        E::VariationAxisCount { .. } => 12,
        E::VariationRegionCount { .. } => 13,
        E::VariationRegionCoordinate { .. } => 14,
        E::VariationDeltaIndex { .. } => 15,
        E::VariationRegionIndex { .. } => 16,
        E::VariationWordCount { .. } => 17,
        E::VariationRow { .. } => 18,
        E::VariationScalar { .. } => 19,
        E::BaselineTagOrder => 20,
        E::BaselineScriptOrder => 21,
        E::BaselineValueCount { .. } => 22,
        E::BaselineDefaultIndex { .. } => 23,
        E::BaselineCoordinateFormat { .. } => 24,
        E::BaselineDeviceFormat { .. } => 25,
        E::BaselineVariableDevice => 26,
        E::BaselineVariationStoreMissing => 27,
        E::BaselineOutline { .. } => 28,
        E::BaselinePointRange { .. } => 29,
        E::BaselineScriptUnavailable => 30,
        E::InvalidUnitsPerEm { .. } => 31,
    }
}

#[test]
fn font_metric_table_errors_state_their_cause() {
    use FontMetricTableError as E;
    check_table(
        32,
        vec![
            (E::Missing, "a required table is absent"),
            (
                E::MissingCompanion {
                    required: FontMetricTable::Vmtx,
                },
                "a present table lacks its required companion table",
            ),
            (
                E::LongMetricCount {
                    count: 0,
                    glyph_count: 10,
                },
                "long metric count 0 must be at least 1 and at most the glyph count 10",
            ),
            (
                E::Duplicate,
                "more than one directory entry claims the table",
            ),
            (E::Inaccessible, "the table's recorded range cannot be read"),
            (E::Malformed, "the table is truncated or malformed"),
            (E::UnsupportedVersion, "the table version is not supported"),
            (
                E::VariationWithoutAxes,
                "a metric variation source is present but the face has no variation axes",
            ),
            (
                E::UnsupportedRecordSize { size: 12 },
                "the value-record size 12 is larger than the supported eight bytes",
            ),
            (
                E::VariationRecordOrder { record: 3 },
                "value record 3 has a tag that is not greater than the previous record's tag",
            ),
            (
                E::VariationOffset { offset: 70_000 },
                "variation offset 70000 cannot be resolved",
            ),
            (
                E::VariationStoreFormat { format: 2 },
                "variation store format 2 is not the supported format 1",
            ),
            (
                E::VariationAxisCount {
                    expected: 2,
                    actual: 3,
                },
                "the variation axis count is 3 where 2 was expected",
            ),
            (
                E::VariationRegionCount { count: 40_000 },
                "variation region count 40000 is not below 32768",
            ),
            (
                E::VariationRegionCoordinate {
                    region: 1,
                    axis: 2,
                    value: 20_000,
                },
                "variation region 1 axis 2 has a coordinate outside the range minus one to one",
            ),
            (
                E::VariationDeltaIndex {
                    record: 4,
                    outer: 5,
                    inner: 6,
                },
                "record 4 refers to delta set 5:6, which its variation store does not contain",
            ),
            (
                E::VariationRegionIndex {
                    offset: 8,
                    index: 9,
                },
                "the data subtable at offset 8 refers to region 9, outside the region list",
            ),
            (
                E::VariationWordCount {
                    offset: 8,
                    words: 5,
                    regions: 3,
                },
                "the data subtable at offset 8 declares a wide-delta count of 5 but a region count of only 3",
            ),
            (
                E::VariationRow {
                    offset: 8,
                    inner: 2,
                    expected: 4,
                    actual: 3,
                },
                "row 2 of the data subtable at offset 8 yields a delta count of 3 where 4 is declared",
            ),
            (
                E::VariationScalar {
                    region: 7,
                    value: 70_000,
                },
                "the scalar of variation region 7 is outside zero through one",
            ),
            (
                E::BaselineTagOrder,
                "baseline tags are duplicated or not strictly sorted",
            ),
            (
                E::BaselineScriptOrder,
                "baseline script tags are duplicated or not strictly sorted",
            ),
            (
                E::BaselineValueCount {
                    expected: 3,
                    actual: 2,
                },
                "the selected script's coordinate count 2 differs from its baseline tag count 3",
            ),
            (
                E::BaselineDefaultIndex { index: 4, count: 3 },
                "default baseline index 4 is outside the coordinate count 3",
            ),
            (
                E::BaselineCoordinateFormat { format: 4 },
                "baseline coordinate format 4 is not 1, 2 or 3",
            ),
            (
                E::BaselineDeviceFormat { format: 5 },
                "baseline device format 5 is invalid",
            ),
            (
                E::BaselineVariableDevice,
                "a device table is not allowed in the baseline table of a variable face",
            ),
            (
                E::BaselineVariationStoreMissing,
                "a variation index is used but the baseline table has no variation store",
            ),
            (
                E::BaselineOutline {
                    glyph: 6,
                    point: 7,
                    reason: FontOutlineError::InvalidFont,
                },
                "the stored outline of glyph 6 cannot validate point 7",
            ),
            (
                E::BaselinePointRange {
                    glyph: 6,
                    point: 9,
                    count: 4,
                },
                "point 9 of glyph 6 is outside its stored point count 4",
            ),
            (
                E::BaselineScriptUnavailable,
                "the run's script has no canonical four-letter Unicode short name",
            ),
            (
                E::InvalidUnitsPerEm { value: 8 },
                "units-per-em 8 is outside 16 to 16384",
            ),
        ],
        index_table,
        |error| matches!(error, E::BaselineOutline { .. }),
    );
}

#[test]
fn a_baseline_outline_failure_reports_the_outline_refusal_from_its_source() {
    let reason = FontOutlineError::GlyphRange { count: 3 };
    let error = FontMetricTableError::BaselineOutline {
        glyph: 9,
        point: 1,
        reason,
    };
    check_wrapper(&error, &reason);
    assert_eq!(
        error.to_string(),
        "the stored outline of glyph 9 cannot validate point 1"
    );
    assert_eq!(
        std::error::Error::source(&error).unwrap().to_string(),
        "the requested glyph is outside the face's glyph count of 3"
    );
}

fn index_numeric(error: &FontMetricNumericError) -> usize {
    use FontMetricNumericError as E;
    match error {
        E::NonFinite => 0,
        E::Negative => 1,
        E::Positive => 2,
        E::NonPositive => 3,
        E::NonZero => 4,
    }
}

#[test]
fn font_metric_numeric_errors_state_the_violated_requirement() {
    use FontMetricNumericError as E;
    check_table(
        5,
        vec![
            (E::NonFinite, "a required finite metric value is not finite"),
            (
                E::Negative,
                "a metric value that must not be negative is negative",
            ),
            (
                E::Positive,
                "a metric value that must not be positive is positive",
            ),
            (
                E::NonPositive,
                "a metric value that must be positive is zero or negative",
            ),
            (E::NonZero, "a metric value that must be zero is not zero"),
        ],
        index_numeric,
        |_| false,
    );
}

fn index_outline(error: &FontOutlineError) -> usize {
    use FontOutlineError as E;
    match error {
        E::InvalidFont => 0,
        E::GlyphRange { .. } => 1,
        E::UnavailableOutline => 2,
        E::Unsupported { .. } => 3,
        E::ProviderRefused { .. } => 4,
        E::CapacityOverflow => 5,
        E::AllocationRefused => 6,
    }
}

#[test]
fn font_outline_errors_state_their_cause() {
    use FontOutlineError as E;
    check_table(
        7,
        vec![
            (
                E::InvalidFont,
                "the font has a malformed directory, header, outline or variation source",
            ),
            (
                E::GlyphRange { count: 3 },
                "the requested glyph is outside the face's glyph count of 3",
            ),
            (
                E::UnavailableOutline,
                "the face has no available outline representation",
            ),
            (
                E::Unsupported {
                    capability: FontOutlineCapability::PostScript,
                },
                "the face uses an outline representation or feature that is not supported",
            ),
            (
                E::ProviderRefused { glyph: 12 },
                "the font provider cannot decode or draw glyph 12",
            ),
            (
                E::CapacityOverflow,
                "an outline scratch size or count exceeds the representable range",
            ),
            (
                E::AllocationRefused,
                "reserving temporary outline storage failed",
            ),
        ],
        index_outline,
        |_| false,
    );
}

fn index_text_numeric(error: &TextNumericError) -> usize {
    use TextNumericError as E;
    match error {
        E::NonFinite => 0,
        E::PositiveLost => 1,
        E::NonZeroLost => 2,
        E::OutOfRange => 3,
    }
}

#[test]
fn text_numeric_errors_state_the_category_that_was_not_preserved() {
    use TextNumericError as E;
    check_table(
        4,
        vec![
            (
                E::NonFinite,
                "a geometry value, or the result of converting or combining it, is not finite",
            ),
            (
                E::PositiveLost,
                "a positive geometry value did not stay positive",
            ),
            (
                E::NonZeroLost,
                "a nonzero geometry value was lost: it became zero or no longer changed its result",
            ),
            (
                E::OutOfRange,
                "a geometry value is outside the domain the operation admits",
            ),
        ],
        index_text_numeric,
        |_| false,
    );
}

fn index_bidi(error: &BidiLevelError) -> usize {
    match error {
        BidiLevelError::OutOfRange { .. } => 0,
    }
}

#[test]
fn a_bidi_level_error_states_the_highest_resolved_level() {
    check_table(
        1,
        vec![(
            BidiLevelError::OutOfRange { level: 200 },
            "bidi level 200 is above the highest resolved level 126",
        )],
        index_bidi,
        |_| false,
    );
}

fn index_supplied(error: &SuppliedLineError) -> usize {
    match error {
        SuppliedLineError::OrderBufferLength { .. } => 0,
    }
}

#[test]
fn a_supplied_line_error_compares_slots_with_participants() {
    check_table(
        1,
        vec![(
            SuppliedLineError::OrderBufferLength {
                expected: 3,
                actual: 2,
            },
            "the visual-order buffer length is 2 but the line's participant count is 3",
        )],
        index_supplied,
        |_| false,
    );
}

fn index_hyphenation(error: &HyphenationError) -> usize {
    use HyphenationError as E;
    match error {
        E::InvalidResourceMapping => 0,
        E::ResourceLengthOverflow => 1,
        E::ResourceStorage => 2,
        E::InvalidConvention => 3,
        E::InvalidConventionBinding => 4,
        E::EmptyNativeAlternatives => 5,
        E::MissingPrimaryFont => 6,
        E::InvalidContinuation => 7,
        E::MissingReceivingBidiAuthority => 8,
        E::UnsupportedIndicationSeparator => 9,
        E::UnsupportedIndicationComposition => 10,
        E::UncapturedIndicationFontQuery => 11,
    }
}

#[test]
fn hyphenation_errors_state_their_cause() {
    use HyphenationError as E;
    check_table(
        12,
        vec![
            (
                E::InvalidResourceMapping,
                "the hyphenation resource or a cut it reports cannot be mapped onto the retained text",
            ),
            (
                E::ResourceLengthOverflow,
                "a hyphenation resource length or offset is not representable",
            ),
            (
                E::ResourceStorage,
                "storage for a hyphenation resource query or its observation could not be reserved",
            ),
            (
                E::InvalidConvention,
                "the native hyphenation convention has an invalid or inconsistent locale or script",
            ),
            (
                E::InvalidConventionBinding,
                "the native hyphenation convention does not bind to the content locale",
            ),
            (
                E::EmptyNativeAlternatives,
                "the native logical-before replacement lists no alternatives",
            ),
            (
                E::MissingPrimaryFont,
                "the style selects no font for the automatic hyphen",
            ),
            (
                E::InvalidContinuation,
                "the hyphenation continuation is inconsistent with the cut or placement it accompanies",
            ),
            (
                E::MissingReceivingBidiAuthority,
                "an incoming hyphenation indication has no receiving bidi authority",
            ),
            (
                E::UnsupportedIndicationSeparator,
                "the hyphenation indication cannot attach to a separator",
            ),
            (
                E::UnsupportedIndicationComposition,
                "the hyphenation indication cannot be composed with the surrounding text",
            ),
            (
                E::UncapturedIndicationFontQuery,
                "no captured font query covers the hyphenation indication",
            ),
        ],
        index_hyphenation,
        |_| false,
    );
}
