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
    // The ceilings are fixed values, ceil(4 * reference * 1024 / units), from the
    // densest legitimate input found for each counter, at the default input length:
    // - algorithm records: 29,096 in 1,021 units, `color.dense-numerals-1e-46`, a
    //   `min()` list of `1e-46` (used path; the authored path needs 28,237);
    // - actions: 1,913,935 in 1,021 units, `color.dense-numerals-5e-45`, a `min()`
    //   list of `5e-45` (authored path; the used path needs 1,810,895);
    // - arithmetic and evaluation records: 3,030 each in 1,023 units,
    //   `color.dense-arithmetic`, a `min()` of one operand and `1*1` products
    //   (authored path; the used path needs 1,769).
    // Each reference is at least the requirement per unit of every corpus colour, so
    // the ceilings keep large headroom over the corpus (the tests below check a
    // quarter of them). The path allows one step per source unit.
    let ordinary = CssColorOrdinaryLimits::default();
    assert_eq!(
        ordinary,
        CssColorOrdinaryLimits {
            input_units: 1024,
            algorithm_records: 116_726,
            arithmetic_records: 12_132,
            evaluation_records: 12_132,
            actions: 7_678_235,
        }
    );
    assert_eq!(
        CssAuthoredColorOrdinaryLimits::default(),
        CssAuthoredColorOrdinaryLimits {
            algorithm_records: 116_726,
            arithmetic_records: 12_132,
            evaluation_records: 12_132,
            path_steps: 1024,
            actions: 7_678_235,
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

/// The fronts that resolve an input under limits: the whole-colour used path and
/// the authored path.
const LIMIT_FRONTS: [&str; 2] = ["color.used.utf8.whole", "color.authored"];

/// `lab(0 min(N,N,...) 0)` with `leaves` operands: 13 + 6 * `leaves` units for a 5-unit
/// `N`.
fn numeral_list(numeral: &str, leaves: usize) -> String {
    format!(
        "lab(0 min({numeral}{}) 0)",
        format!(",{numeral}").repeat(leaves - 1)
    )
}

/// `lab(0 min(1,1*1,1*1,...) 0)` with `groups` products: 15 + 4 * `groups` units.
fn product_list(groups: usize) -> String {
    format!("lab(0 min(1{}) 0)", ",1*1".repeat(groups))
}

/// Limits under which only one counter can refuse: the input length and every counter
/// are lifted, then the counter `set_ordinary` and `set_authored` name is fixed.
fn lifted_except(
    set_ordinary: fn(&mut CssColorOrdinaryLimits, usize),
    set_authored: fn(&mut CssAuthoredColorOrdinaryLimits, usize),
    ceiling: usize,
) -> (CssColorOrdinaryLimits, CssAuthoredColorOrdinaryLimits) {
    let mut ordinary = CssColorOrdinaryLimits {
        input_units: usize::MAX,
        algorithm_records: usize::MAX,
        arithmetic_records: usize::MAX,
        evaluation_records: usize::MAX,
        actions: usize::MAX,
    };
    let mut authored = CssAuthoredColorOrdinaryLimits {
        algorithm_records: usize::MAX,
        arithmetic_records: usize::MAX,
        evaluation_records: usize::MAX,
        path_steps: usize::MAX,
        actions: usize::MAX,
    };
    set_ordinary(&mut ordinary, ceiling);
    set_authored(&mut authored, ceiling);
    (ordinary, authored)
}

/// The observation of `input` on `front` with one ceiling fixed, every other
/// counter lifted.
fn observe_with_ceiling(
    front: &str,
    input: &str,
    set_ordinary: fn(&mut CssColorOrdinaryLimits, usize),
    set_authored: fn(&mut CssAuthoredColorOrdinaryLimits, usize),
    ceiling: usize,
) -> Observation {
    let (ordinary, authored) = lifted_except(set_ordinary, set_authored, ceiling);
    observe_color_under(front, "absent", input, ordinary, authored)
}

/// On each front of `inputs` (`(front, above, below)`), `above` needs more of the counter
/// than `ceiling` and is refused with `limit:{resource}`, and `below` needs less and
/// resolves.
fn assert_refused_above_resolves_below(
    inputs: &[(&str, &str, &str)],
    resource: &str,
    set_ordinary: fn(&mut CssColorOrdinaryLimits, usize),
    set_authored: fn(&mut CssAuthoredColorOrdinaryLimits, usize),
    ceiling: usize,
) {
    for (front, above, below) in inputs {
        let refused = observe_with_ceiling(front, above, set_ordinary, set_authored, ceiling);
        assert_eq!(refused.shape, "err", "{front}: the longer input");
        assert!(
            refused.facts.contains(&format!("limit:{resource}")),
            "{front}: {}",
            refused.facts
        );
        let resolved = observe_with_ceiling(front, below, set_ordinary, set_authored, ceiling);
        assert_eq!(resolved.shape, "used", "{front}: the shorter input");
    }
}

#[test]
fn a_numeral_list_needing_more_actions_than_the_default_is_refused_and_a_shorter_one_resolves() {
    // A `min()` of `5e-45` needs about 1,790 actions per input unit on the used path and
    // 1,900 on the authored path. 740 operands (4,453 units) need about 7.98M used and
    // 8.45M authored, more than the 7,678,235 of the default; 650 operands (3,913 units)
    // need about 7.02M used and 7.41M authored, less.
    let (above, below) = (numeral_list("5e-45", 740), numeral_list("5e-45", 650));
    assert_eq!((above.len(), below.len()), (4453, 3913));
    let inputs = LIMIT_FRONTS.map(|front| (front, above.as_str(), below.as_str()));
    assert_refused_above_resolves_below(
        &inputs,
        "actions",
        |limits, value| limits.actions = value,
        |limits, value| limits.actions = value,
        CssColorOrdinaryLimits::default().actions,
    );
}

#[test]
fn a_numeral_list_needing_more_algorithm_records_than_the_default_is_refused_and_a_shorter_one_resolves()
 {
    // A `min()` of `1e-46` needs about 28.8 algorithm records per input unit on the used
    // path and 28 on the authored path. 740 operands (4,453 units) need about 128.1k used
    // and 124.7k authored, more than the 116,726 of the default; 650 operands (3,913 units)
    // need about 112.8k used and 109.3k authored, less.
    let (above, below) = (numeral_list("1e-46", 740), numeral_list("1e-46", 650));
    assert_eq!((above.len(), below.len()), (4453, 3913));
    let inputs = LIMIT_FRONTS.map(|front| (front, above.as_str(), below.as_str()));
    assert_refused_above_resolves_below(
        &inputs,
        "algorithm-records",
        |limits, value| limits.algorithm_records = value,
        |limits, value| limits.algorithm_records = value,
        CssColorOrdinaryLimits::default().algorithm_records,
    );
}

#[test]
fn a_product_list_needing_more_arithmetic_or_evaluation_records_than_the_default_is_refused_and_a_shorter_one_resolves()
 {
    // `min(1,1*1,...)` needs the same number of arithmetic and evaluation records: about
    // 3 per input unit on the authored path and 1.75 on the used path. On the authored
    // front 1,125 products (4,515 units) need 13,506 of each and 925 products (3,715
    // units) need 11,106. On the used front 1,900 products (7,615 units) need 13,305 and
    // 1,550 products (6,215 units) need 10,855. The default is 12,132 of each.
    let (authored_above, authored_below) = (product_list(1125), product_list(925));
    let (used_above, used_below) = (product_list(1900), product_list(1550));
    assert_eq!(
        (
            authored_above.len(),
            authored_below.len(),
            used_above.len(),
            used_below.len()
        ),
        (4515, 3715, 7615, 6215)
    );
    let inputs = [
        (
            "color.authored",
            authored_above.as_str(),
            authored_below.as_str(),
        ),
        (
            "color.used.utf8.whole",
            used_above.as_str(),
            used_below.as_str(),
        ),
    ];
    assert_refused_above_resolves_below(
        &inputs,
        "arithmetic-records",
        |limits, value| limits.arithmetic_records = value,
        |limits, value| limits.arithmetic_records = value,
        CssColorOrdinaryLimits::default().arithmetic_records,
    );
    assert_refused_above_resolves_below(
        &inputs,
        "evaluation-records",
        |limits, value| limits.evaluation_records = value,
        |limits, value| limits.evaluation_records = value,
        CssColorOrdinaryLimits::default().evaluation_records,
    );
}
