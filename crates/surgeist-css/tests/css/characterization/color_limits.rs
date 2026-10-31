//! The default ordinary colour limits against the colour cases of the table.
//!
//! The defaults in `src/color/ordinary_defaults.rs` are fixed ceilings: a
//! reference requirement per input unit, scaled to the default input length,
//! times four. The golden file records every corpus outcome under the explicit
//! limits of the adapter, so it is the oracle that the defaults change no
//! outcome. A requirement is the smallest ceiling at which a case still reaches
//! its outcome; a case needs at most a quarter of its default ceiling, pro rata
//! to its own length, when it still reaches that outcome under exactly that
//! allowance.

use surgeist_css::color::{
    CssAuthoredColorOrdinaryLimits, CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest,
    CssColorInput, CssColorOrdinaryLimits, CssColorOrdinaryResource, CssColorPropertyFacts,
    CssColorSchemeFact, CssDeviceColorFact, parse_used_color,
};

use super::adapter::{Observation, observe_color_under};
use super::cases::{self, Case};
use super::{GOLDEN, escape};

/// The headroom the defaults keep over a corpus requirement.
const HEADROOM: usize = 4;

/// Column positions of the golden header.
const COLUMN_ENTRY: usize = 1;
const COLUMN_INPUT: usize = 3;
const COLUMN_ARGUMENT: usize = 8;
const COLUMN_SHAPE: usize = 9;
const COLUMN_FACTS: usize = 13;

/// The fronts that resolve a colour under ordinary limits.
const RESOLUTION_FRONTS: [&str; 5] = [
    "color.used.utf8.whole",
    "color.used.utf16.whole",
    "color.used.utf8.prefix",
    "color.used.utf16.prefix",
    "color.authored",
];

/// The refusal the explicit action ceiling of the golden gives the overflow case.
const EXPLICIT_ACTION_REFUSAL: &str = "limit:actions:required=";

fn resolution_cases() -> Vec<Case> {
    cases::all()
        .into_iter()
        .filter(|case| RESOLUTION_FRONTS.contains(&case.front.as_str()))
        .collect()
}

fn golden_row(id: &str) -> Vec<&'static str> {
    GOLDEN
        .lines()
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .find(|columns| columns[0] == id)
        .unwrap_or_else(|| panic!("golden row {id}"))
}

/// The recorded shape and facts of a case. The golden refuses the overflow case
/// at its explicit action ceiling; its outcome is the recorded one of the same
/// input under the large action allowance row.
fn expected(case: &Case) -> (String, String) {
    let row = golden_row(&case.id);
    if !row[COLUMN_FACTS].contains(EXPLICIT_ACTION_REFUSAL) {
        return (row[COLUMN_SHAPE].to_owned(), row[COLUMN_FACTS].to_owned());
    }
    let allowance_front = if case.front == "color.authored" {
        "color.authored.large-actions"
    } else {
        "color.used.large-actions"
    };
    let relaxed = GOLDEN
        .lines()
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .find(|columns| {
            columns[COLUMN_ENTRY] == allowance_front
                && columns[COLUMN_INPUT] == escape(&case.input)
                && columns[COLUMN_ARGUMENT] == case.argument
        })
        .unwrap_or_else(|| panic!("{}: no relaxed-allowance row", case.id));
    (
        relaxed[COLUMN_SHAPE].to_owned(),
        relaxed[COLUMN_FACTS].to_owned(),
    )
}

fn assert_outcome(
    case: &Case,
    observed: &Observation,
    expectation: &(String, String),
    under: &str,
) {
    assert_eq!(
        (escape(&observed.shape), escape(&observed.facts)),
        *expectation,
        "{} under {under}",
        case.id
    );
}

/// Input units of a case in its front's own unit.
fn units(case: &Case) -> usize {
    if case.front.contains("utf16") {
        case.input.encode_utf16().count()
    } else {
        case.input.len()
    }
}

/// The quarter of a default ceiling that a case of `units` units may use:
/// `ceiling * units / (HEADROOM * default input units)`, rounded down.
fn pro_rata_quarter(ceiling: usize, units: usize, default_input_units: usize) -> usize {
    let wide =
        ceiling as u128 * units.max(1) as u128 / (HEADROOM as u128 * default_input_units as u128);
    usize::try_from(wide).expect("a quarter of a usize ceiling fits usize")
}

fn quarter_limits(case: &Case) -> (CssColorOrdinaryLimits, CssAuthoredColorOrdinaryLimits) {
    let ordinary = CssColorOrdinaryLimits::default();
    let authored = CssAuthoredColorOrdinaryLimits::default();
    let units = units(case);
    let quarter = |ceiling| pro_rata_quarter(ceiling, units, ordinary.input_units);
    (
        CssColorOrdinaryLimits {
            input_units: ordinary.input_units,
            algorithm_records: quarter(ordinary.algorithm_records),
            arithmetic_records: quarter(ordinary.arithmetic_records),
            evaluation_records: quarter(ordinary.evaluation_records),
            actions: quarter(ordinary.actions),
        },
        CssAuthoredColorOrdinaryLimits {
            algorithm_records: quarter(authored.algorithm_records),
            arithmetic_records: quarter(authored.arithmetic_records),
            evaluation_records: quarter(authored.evaluation_records),
            path_steps: quarter(authored.path_steps),
            actions: quarter(authored.actions),
        },
    )
}

#[test]
fn default_ceilings_are_the_documented_derivation() {
    // The ceilings are fixed values, ceil(4 * reference * 1024 / units), from a
    // reference requirement per counter: algorithm 1277 records in 19 units,
    // arithmetic and evaluation 28 records in 33 units, actions 19,390,200 in 39
    // units. Each reference is at least the requirement per unit of every corpus
    // colour, so the ceilings keep large headroom over the corpus (the tests
    // below check a quarter of them). The path allows one step per source unit.
    let ordinary = CssColorOrdinaryLimits::default();
    assert_eq!(
        ordinary,
        CssColorOrdinaryLimits {
            input_units: 1024,
            algorithm_records: 275_295,
            arithmetic_records: 3476,
            evaluation_records: 3476,
            actions: 2_036_468_185,
        }
    );
    assert_eq!(
        CssAuthoredColorOrdinaryLimits::default(),
        CssAuthoredColorOrdinaryLimits {
            algorithm_records: 275_295,
            arithmetic_records: 3476,
            evaluation_records: 3476,
            path_steps: 1024,
            actions: 2_036_468_185,
        }
    );
}

#[test]
fn every_corpus_color_case_resolves_under_default_limits_as_the_golden_records() {
    let (ordinary, authored) = (
        CssColorOrdinaryLimits::default(),
        CssAuthoredColorOrdinaryLimits::default(),
    );
    let mut checked = 0;
    for case in resolution_cases() {
        let observed =
            observe_color_under(&case.front, &case.argument, &case.input, ordinary, authored);
        assert_outcome(&case, &observed, &expected(&case), "the default limits");
        checked += 1;
    }
    // Every colour input of the table reaches each of the five fronts.
    assert!(checked >= 5 * 50, "only {checked} corpus resolutions");
}

#[test]
fn corpus_requirements_scaled_to_the_default_length_stay_within_a_quarter_of_every_ceiling() {
    for case in resolution_cases() {
        let (ordinary, authored) = quarter_limits(&case);
        let observed =
            observe_color_under(&case.front, &case.argument, &case.input, ordinary, authored);
        assert_outcome(
            &case,
            &observed,
            &expected(&case),
            "a quarter of the default ceilings, pro rata to its length",
        );
    }
}

/// `parse_used_color` over the UTF-8 `input` with absent facts.
fn used_error_resource(
    input: &[u8],
    limits: CssColorOrdinaryLimits,
) -> (Option<(CssColorOrdinaryResource, usize, usize)>, bool) {
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(7, 3),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let error = parse_used_color(
        CssColorInput::Utf8(input),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        limits,
    )
    .expect_err("the input needs more than a ceiling");
    (error.exhausted_limit(), error.color_error().is_some())
}

#[test]
fn input_longer_than_the_default_is_refused_with_the_input_units_resource() {
    let limits = CssColorOrdinaryLimits::default();
    let long = " ".repeat(limits.input_units) + "red";
    assert_eq!(
        used_error_resource(long.as_bytes(), limits),
        (
            Some((
                CssColorOrdinaryResource::InputUnits,
                limits.input_units,
                limits.input_units + 3
            )),
            false
        )
    );
}

#[test]
fn input_needing_more_algorithm_records_than_the_default_is_refused_with_that_resource() {
    // Each leading space is one algorithm record (measured: 10,000 spaces and
    // `red` need 10,009), so the record that exceeds the ceiling is ceiling + 1.
    // The input length ceiling is lifted so the records are the limit reached.
    let defaults = CssColorOrdinaryLimits::default();
    let limits = CssColorOrdinaryLimits {
        input_units: usize::MAX,
        ..defaults
    };
    let padded = " ".repeat(defaults.algorithm_records) + "red";
    assert_eq!(
        used_error_resource(padded.as_bytes(), limits),
        (
            Some((
                CssColorOrdinaryResource::AlgorithmRecords,
                defaults.algorithm_records,
                defaults.algorithm_records + 1
            )),
            false
        )
    );
}

#[test]
fn a_requirement_above_a_quarter_of_a_ceiling_fails_the_headroom_test() {
    // A counter's requirement is the smallest ceiling at which a case still
    // reaches its outcome. Find it by bisection for two cases with a high
    // number of records per input unit, then show that a ceiling whose pro rata
    // quarter is one record short of it refuses the case, so the headroom test
    // can fail, and that a ceiling whose quarter equals it does not.
    for (front, set, input, field) in [
        (
            "color.authored",
            "absent",
            "rgb(calc(255 / 2) calc(10 * 3) 0)",
            "arithmetic",
        ),
        (
            "color.used.utf8.whole",
            "absent",
            "oklab(0.5 0.1 -0.1)",
            "algorithm",
        ),
    ] {
        let case = Case {
            id: "headroom".to_owned(),
            front: front.to_owned(),
            feature: "-".to_owned(),
            argument: set.to_owned(),
            input: input.to_owned(),
        };
        let (ordinary, authored) = quarter_limits(&case);
        let refused_with = |records: usize| {
            let (mut ordinary, mut authored) = (ordinary, authored);
            match field {
                "arithmetic" => authored.arithmetic_records = records,
                _ => ordinary.algorithm_records = records,
            }
            let observed = observe_color_under(front, set, input, ordinary, authored);
            observed.facts.contains(&format!("limit:{field}-records"))
        };
        let quarter = match field {
            "arithmetic" => authored.arithmetic_records,
            _ => ordinary.algorithm_records,
        };
        assert!(!refused_with(quarter), "{input} fits its quarter");
        assert!(refused_with(0), "{input} needs records");
        let (mut below, mut at_least) = (0, quarter);
        while at_least - below > 1 {
            let middle = below + (at_least - below) / 2;
            if refused_with(middle) {
                below = middle;
            } else {
                at_least = middle;
            }
        }
        let required = at_least;
        let units = units(&case);
        let input_units = CssColorOrdinaryLimits::default().input_units;
        // The smallest ceilings whose pro rata quarters are `required` and
        // `required - 1` records.
        let ceiling_for = |wanted: usize| (wanted * HEADROOM * input_units).div_ceil(units);
        assert!(pro_rata_quarter(ceiling_for(required), units, input_units) >= required);
        let short = pro_rata_quarter(ceiling_for(required - 1), units, input_units);
        assert_eq!(short, required - 1, "{input}");
        assert!(refused_with(short), "{input}: one record short is refused");
        assert!(!refused_with(required), "{input}: the requirement fits");
    }
}
