//! Default ceilings of the ordinary colour resolution limits.
//!
//! [`CssColorOrdinaryLimits`](crate::color::CssColorOrdinaryLimits) and
//! [`CssAuthoredColorOrdinaryLimits`](crate::color::CssAuthoredColorOrdinaryLimits) share
//! these constants. Every ceiling is the worst requirement measured over the colour cases of
//! the public-report characterization corpus (`tests/css/characterization`), scaled to
//! [`DEFAULT_INPUT_UNITS`] and multiplied by [`HEADROOM`]:
//!
//! ```text
//! ceiling = ceil(HEADROOM * required * DEFAULT_INPUT_UNITS / corpus_units)
//! ```
//!
//! `required` and `corpus_units` belong to the one corpus input with the highest requirement
//! per input unit of that counter. A counter's requirement is the smallest ceiling at which
//! the case still runs to its outcome. The counters grow linearly with input length for the
//! padded and nested constructions that were measured; the actions of a numeric literal
//! depend on its magnitude, not its length, and are bounded by the range of `f64`.
//!
//! A caller whose inputs are longer than [`DEFAULT_INPUT_UNITS`], or that carry more
//! extreme numeric literals than the corpus, passes explicit limits.

/// The fourfold margin kept over every measured corpus requirement.
pub(crate) const HEADROOM: u64 = 4;

/// The input length, in original units, at which the defaults are derived and which the
/// default `input_units` admits; it is also the default `path_steps`, because a component
/// path of a parsed color has at most one step per source unit.
///
/// The component-value default (`CssComponentValueLimits::default()`) imposes no length
/// policy, so it supplies no length at which a per-length requirement exists; 1024 is the
/// length the in-repository colour callers and tests already admit.
pub(crate) const DEFAULT_INPUT_UNITS: usize = 1024;

/// One measured worst corpus requirement: `required` of one counter for a corpus input of
/// `units` input units.
#[derive(Clone, Copy)]
struct CorpusWorst {
    required: u64,
    units: u64,
}

/// The ceiling for `worst`: its requirement per unit, at [`DEFAULT_INPUT_UNITS`], with
/// [`HEADROOM`], rounded up.
const fn ceiling(worst: CorpusWorst) -> usize {
    let wide = (HEADROOM * worst.required * DEFAULT_INPUT_UNITS as u64).div_ceil(worst.units);
    assert!(
        wide <= usize::MAX as u64,
        "a default ceiling must fit the platform's usize"
    );
    wide as usize
}

/// 1277 algorithm records for `oklab(0.5 0.1 -0.1)` (19 units): the highest algorithm
/// records per unit of the corpus, 67.2 (the authored path needs 1253).
const ALGORITHM_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 1277,
    units: 19,
};

/// 28 arithmetic records, authored, for `rgb(calc(255 / 2) calc(10 * 3) 0)` (33 units): the
/// highest arithmetic records per unit of the corpus, 0.85 (the used path needs 16).
const ARITHMETIC_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 28,
    units: 33,
};

/// 28 evaluation records, authored, for the same input: the evaluation bank grows with the
/// arithmetic bank, 0.85 per unit.
const EVALUATION_RECORDS_WORST: CorpusWorst = CorpusWorst {
    required: 28,
    units: 33,
};

/// 19,390,200 actions, authored, for `color-mix(in srgb, lab(0 1e308 0), red)` (39 units): the
/// highest actions per unit of the corpus, 497,185 (the used path needs 19,150,362). Its
/// `1e308` literal drives the count through the digit arithmetic of an extreme magnitude.
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
/// [`DEFAULT_INPUT_UNITS`] with [`HEADROOM`]. Callers with larger inputs, or with more
/// extreme numeric literals, pass explicit limits.
pub(crate) const DEFAULT_ACTIONS: usize = ceiling(ACTIONS_WORST);
