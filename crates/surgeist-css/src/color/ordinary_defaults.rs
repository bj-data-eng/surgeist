//! Default ceilings of the ordinary colour resolution limits.
//!
//! [`CssColorOrdinaryLimits`](crate::color::CssColorOrdinaryLimits) and
//! [`CssAuthoredColorOrdinaryLimits`](crate::color::CssAuthoredColorOrdinaryLimits) share
//! these constants. Every ceiling is a fixed reference requirement, scaled to
//! [`DEFAULT_INPUT_UNITS`] and multiplied by [`HEADROOM`]:
//!
//! ```text
//! ceiling = ceil(HEADROOM * required * DEFAULT_INPUT_UNITS / corpus_units)
//! ```
//!
//! `required` and `corpus_units` are the reference requirement of one counter and the
//! input length it is stated for. A counter's requirement is the smallest ceiling at which
//! a case still runs to its outcome. Each reference is the densest legitimate input found
//! for its counter: at least the requirement per input unit of every colour case of the
//! public-report characterization corpus (`tests/css/characterization`), which includes the
//! dense inputs named at each reference below, so the ceilings keep a fourfold margin over
//! them; the corpus tests check that every case fits within a quarter of each ceiling, pro
//! rata to its length. The counters grow linearly with input length for the padded, nested
//! and repeated constructions that were measured (at 144, 512 and 1,024 units). A numeral
//! outside `[1e-46, 1e41)` is not searched; inside it the exact search costs the most per
//! input unit at the lower edge (`1e-45` needs about 10,800 actions as one numeral), and a
//! short numeral repeated as the operands of a `min()` costs more per unit than one numeral
//! in a long colour.
//!
//! A caller whose inputs are longer than [`DEFAULT_INPUT_UNITS`] passes explicit limits.

/// The fourfold margin kept over every reference requirement.
pub(crate) const HEADROOM: u64 = 4;

/// The input length, in original units, at which the defaults are derived and which the
/// default `input_units` admits; it is also the default `path_steps`, because a component
/// path of a parsed color has at most one step per source unit.
///
/// The component-value default (`CssComponentValueLimits::default()`) imposes no length
/// policy, so it supplies no length at which a per-length requirement exists; 1024 is the
/// length the in-repository colour callers and tests already admit.
pub(crate) const DEFAULT_INPUT_UNITS: usize = 1024;

/// One reference requirement: `required` of one counter for an input of `units` input
/// units.
#[derive(Clone, Copy)]
struct CorpusWorst {
    required: u64,
    units: u64,
}

/// The ceiling for a reference: its requirement per unit, at [`DEFAULT_INPUT_UNITS`], with
/// [`HEADROOM`], rounded up.
const fn ceiling(worst: CorpusWorst) -> usize {
    let wide = (HEADROOM * worst.required * DEFAULT_INPUT_UNITS as u64).div_ceil(worst.units);
    assert!(
        wide <= usize::MAX as u64,
        "a default ceiling must fit the platform's usize"
    );
    wide as usize
}

/// 29,096 algorithm records in 1,021 units: 28.5 per unit. The input is
/// `color.dense-numerals-1e-46` of the characterization corpus, a `min()` list of `1e-46`
/// (the numeral inside the exact window that needs the most records per unit); the authored
/// path needs 28,237.
const ALGORITHM_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 29_096,
    units: 1021,
};

/// 3,030 arithmetic records in 1,023 units: 2.96 per unit. The input is
/// `color.dense-arithmetic`, a `min()` of one operand and `1*1` products, on the authored
/// path (the used path needs 1,769).
const ARITHMETIC_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 3030,
    units: 1023,
};

/// 3,030 evaluation records in 1,023 units: the evaluation bank grows with the arithmetic
/// bank, and `color.dense-arithmetic` needs as many of each.
const EVALUATION_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 3030,
    units: 1023,
};

/// 1,913,935 actions in 1,021 units: 1,874.6 per unit. The input is
/// `color.dense-numerals-5e-45`, a `min()` list of `5e-45`, on the authored path (the numeral
/// inside the exact window that needs the most actions per unit; the used path needs
/// 1,810,895). A numeral outside the window is not searched and needs far fewer actions.
const ACTIONS_WORST: CorpusWorst = CorpusWorst {
    required: 1_913_935,
    units: 1021,
};

/// The default `algorithm_records` ceiling: 116,726 records, [`ALGORITHM_RECORDS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_ALGORITHM_RECORDS: usize = ceiling(ALGORITHM_RECORDS_WORST);

/// The default `arithmetic_records` ceiling: 12,132 records, [`ARITHMETIC_RECORDS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_ARITHMETIC_RECORDS: usize = ceiling(ARITHMETIC_RECORDS_WORST);

/// The default `evaluation_records` ceiling: 12,132 records, [`EVALUATION_RECORDS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_EVALUATION_RECORDS: usize = ceiling(EVALUATION_RECORDS_WORST);

/// The default `actions` ceiling: 7,678,235 actions, [`ACTIONS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_ACTIONS: usize = ceiling(ACTIONS_WORST);
