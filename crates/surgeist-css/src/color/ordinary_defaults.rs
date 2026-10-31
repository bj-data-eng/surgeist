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
//! a case still runs to its outcome. Each reference is at least the requirement per input
//! unit of every colour case of the public-report characterization corpus
//! (`tests/css/characterization`), so the ceilings keep large headroom over that corpus;
//! the corpus tests check that every case fits within a quarter of each ceiling, pro rata to
//! its length. The counters grow linearly with input length for the padded and nested
//! constructions that were measured; the actions of a numeric literal that needs the exact
//! search are bounded by its midpoint of at most 1,100 digits, whatever its exponent.
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

/// 1277 algorithm records in 19 units: 67.2 per unit.
const ALGORITHM_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 1277,
    units: 19,
};

/// 28 arithmetic records in 33 units: 0.85 per unit.
const ARITHMETIC_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 28,
    units: 33,
};

/// 28 evaluation records in 33 units: the evaluation bank grows with the arithmetic bank.
const EVALUATION_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 28,
    units: 33,
};

/// 19,390,200 actions in 39 units: 497,185 per unit, far above any corpus colour, including
/// one with an extreme-magnitude literal such as `1e308`.
const ACTIONS_WORST: CorpusWorst = CorpusWorst {
    required: 19_390_200,
    units: 39,
};

/// The default `algorithm_records` ceiling: 275,295 records, [`ALGORITHM_RECORDS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_ALGORITHM_RECORDS: usize = ceiling(ALGORITHM_RECORDS_WORST);

/// The default `arithmetic_records` ceiling: 3,476 records, [`ARITHMETIC_RECORDS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_ARITHMETIC_RECORDS: usize = ceiling(ARITHMETIC_RECORDS_WORST);

/// The default `evaluation_records` ceiling: 3,476 records, [`EVALUATION_RECORDS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_EVALUATION_RECORDS: usize = ceiling(EVALUATION_RECORDS_WORST);

/// The default `actions` ceiling: 2,036,468,185 actions, [`ACTIONS_WORST`] at
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs pass explicit limits.
pub(crate) const DEFAULT_ACTIONS: usize = ceiling(ACTIONS_WORST);
