//! Release-mode cost of adopting a parsed sheet into a CSSOM store and of
//! committing one small edit to a store that holds it.
//!
//! For each size N (style rules in one sheet) two cases run: creating a store
//! with `create_parsed_sheet` of N style rules followed by `commit`, and one
//! `set_disabled` edit with its `commit` on a store that already holds the sheet
//! (the edit alternates between disabled and enabled). Timing covers the batch,
//! the operation and the commit; building the store, parsing the sample text and
//! dropping the store are outside it. Every size runs its own number of rounds,
//! each round starting one case later than the one before. The table gives the
//! median, minimum and maximum time of one call, the time per rule for creation,
//! the input bytes and the rules read back from the committed snapshot.
//!
//! Adoption checks the limits after each adopted rule. The first check of an
//! adoption is the census of the whole state; each later check extends that
//! census with the objects the rule inserted, so creation costs time in
//! proportion to N. The census of the whole state also runs once at the end of
//! each operation and once at commit, so an edit costs time in proportion to the
//! rules the store retains. The census column gives the smallest value of three
//! `CssomLimits` fields at which creation still succeeds, found by doubling and
//! bisection where a probe is cheap enough, and `-` elsewhere.
//!
//! The 100 000-rule creation holds about 1.7 GB while it runs, and the edit
//! case keeps its own store of that size alive for the whole run.
//! Run with
//! `cargo bench --offline --locked -j 1 -p surgeist-cssom --bench store_edits`.

use std::hint::black_box;
use std::time::{Duration, Instant};

use surgeist_css::serialization::CssSerializationLimits;
use surgeist_cssom::{
    CssomContext, CssomEditGuard, CssomInputRole, CssomInputVersion, CssomLimits, CssomSheetId,
    CssomSheetInputs, CssomStore,
};

/// Upper bound of every limit search.
const SEARCH_CEILING: usize = 1 << 40;

/// Sheet sizes in style rules, each with the rounds its cases run, the commits
/// one round of the edit case makes, and whether to derive the census limits.
const SIZES: [Size; 4] = [
    Size {
        rules: 100,
        rounds: 11,
        edits: 200,
        probe: true,
    },
    Size {
        rules: 1_000,
        rounds: 5,
        edits: 20,
        probe: true,
    },
    Size {
        rules: 10_000,
        rounds: 3,
        edits: 5,
        probe: false,
    },
    Size {
        rules: 100_000,
        rounds: 3,
        edits: 3,
        probe: false,
    },
];

#[derive(Clone, Copy)]
struct Size {
    rules: usize,
    rounds: usize,
    edits: usize,
    probe: bool,
}

/// Limits that admit every size; the defaults allow 100 000 objects.
fn limits() -> CssomLimits {
    CssomLimits {
        max_objects: 1 << 30,
        max_entries: 1 << 30,
        max_string_bytes: 1 << 34,
        max_input_bytes: 1 << 34,
        max_retained_source_bytes: 1 << 34,
        max_source_traversal_steps: 1 << 40,
        css: CssSerializationLimits::new(usize::MAX, usize::MAX, usize::MAX),
        ..CssomLimits::default()
    }
}

/// A sheet of `rules` style rules, one per line.
fn sheet_text(rules: usize) -> String {
    let mut text = String::with_capacity(rules * 48);
    for index in 0..rules {
        text.push_str(&format!(
            ".rule-{index} > a:hover {{ color: red; margin: {}px 0 }}\n",
            1 + index % 29
        ));
    }
    text
}

fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "document".to_owned(),
            revision: 1,
        },
        "document".into(),
        None,
    )
}

fn new_store() -> CssomStore {
    CssomStore::new(limits(), CssomContext::default()).expect("a store")
}

/// Adopts `text` into `store` and returns the sheet, or `None` when a limit
/// refuses.
fn try_create(store: &mut CssomStore, text: &str) -> Option<CssomSheetId> {
    let mut edit = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .ok()?;
    let ticket = edit.create_parsed_sheet(text, inputs()).ok()?;
    Some(store.commit(edit).ok()?.sheet(&ticket).ok()?.clone())
}

fn create(store: &mut CssomStore, text: &str) -> CssomSheetId {
    try_create(store, text).expect("the limits admit the sheet")
}

/// Toggles the disabled flag of `sheet` in one batch and commits it.
fn toggle(store: &mut CssomStore, sheet: &CssomSheetId, disabled: bool) {
    let mut edit = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .expect("a batch");
    edit.set_disabled(sheet, disabled).expect("an edit");
    black_box(store.commit(edit).expect("a commit"));
}

/// The rules the committed snapshot holds in `sheet`.
fn retained_rules(store: &CssomStore, sheet: &CssomSheetId) -> usize {
    let snapshot = store.snapshot();
    let list = snapshot.sheet(sheet).expect("the sheet").rules();
    snapshot.rules(list).expect("the rules").len()
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

/// The smallest `max_objects`, `max_entries` and `max_string_bytes` at which
/// creating a sheet of `text` still succeeds.
fn census(text: &str) -> String {
    type Field = fn(&mut CssomLimits, usize);
    let fields: [(&str, Field); 3] = [
        ("objects", |limits, value| limits.max_objects = value),
        ("entries", |limits, value| limits.max_entries = value),
        ("string bytes", |limits, value| {
            limits.max_string_bytes = value;
        }),
    ];
    fields
        .iter()
        .map(|(name, set)| {
            let value = smallest(SEARCH_CEILING, |value| {
                let mut limits = limits();
                set(&mut limits, value);
                CssomStore::new(limits, CssomContext::default())
                    .ok()
                    .and_then(|mut store| try_create(&mut store, text))
                    .is_some()
            });
            format!(
                "{name}={}",
                value.map_or_else(|| "-".to_owned(), |v| v.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

enum Kind {
    Create,
    Edit,
}

struct Case {
    name: String,
    kind: Kind,
    size: Size,
    text: String,
    store: Option<(CssomStore, CssomSheetId)>,
    disabled: bool,
    census: String,
}

impl Case {
    /// One call of the case, returning the time it took.
    fn call(&mut self) -> Duration {
        match self.kind {
            Kind::Create => {
                let mut store = new_store();
                let start = Instant::now();
                let sheet = create(&mut store, black_box(&self.text));
                let elapsed = start.elapsed();
                assert_eq!(retained_rules(&store, &sheet), self.size.rules);
                elapsed
            }
            Kind::Edit => {
                let (store, sheet) = self.store.as_mut().expect("a store with the sheet");
                self.disabled = !self.disabled;
                let start = Instant::now();
                toggle(store, sheet, self.disabled);
                start.elapsed()
            }
        }
    }

    /// The calls of one round.
    fn calls(&self) -> usize {
        match self.kind {
            Kind::Create => 1,
            Kind::Edit => self.size.edits,
        }
    }
}

fn main() {
    let mut cases = Vec::new();
    for size in SIZES {
        let text = sheet_text(size.rules);
        let counted = if size.probe {
            census(&text)
        } else {
            "-".to_owned()
        };
        for kind in [Kind::Create, Kind::Edit] {
            let (label, store) = match kind {
                Kind::Create => ("create_parsed_sheet + commit", None),
                Kind::Edit => {
                    let mut store = new_store();
                    let sheet = create(&mut store, &text);
                    assert_eq!(retained_rules(&store, &sheet), size.rules);
                    ("set_disabled + commit", Some((store, sheet)))
                }
            };
            cases.push(Case {
                name: format!("{label}, {} rules", size.rules),
                size,
                text: if matches!(kind, Kind::Create) {
                    text.clone()
                } else {
                    String::new()
                },
                store,
                disabled: false,
                census: counted.clone(),
                kind,
            });
        }
    }

    let maximum_rounds = SIZES.iter().map(|size| size.rounds).max().unwrap_or(0);
    let mut times: Vec<Vec<f64>> = vec![Vec::new(); cases.len()];
    for round in 0..maximum_rounds {
        for step in 0..cases.len() {
            let index = (round + step) % cases.len();
            let case = &mut cases[index];
            if round >= case.size.rounds {
                continue;
            }
            let calls = case.calls();
            let mut total = Duration::ZERO;
            for _ in 0..calls {
                total += case.call();
            }
            times[index].push(total.as_secs_f64() * 1e9 / calls as f64);
        }
    }

    println!(
        "each round starts one case later; rounds per size and commits per edit round are set per size"
    );
    println!(
        "case\trounds\tcalls/round\tmedian ns/call\tmin ns/call\tmax ns/call\tmedian ns/rule\tinput bytes\tretained rules\tcensus"
    );
    for (index, case) in cases.iter().enumerate() {
        let per_rule = |time: f64| match case.kind {
            Kind::Create => format!("{:.1}", time / case.size.rules as f64),
            Kind::Edit => "-".to_owned(),
        };
        let bytes = match case.kind {
            Kind::Create => sheet_text(case.size.rules).len().to_string(),
            Kind::Edit => "-".to_owned(),
        };
        let mut sorted = times[index].clone();
        sorted.sort_by(f64::total_cmp);
        let Some(&time) = sorted.get(sorted.len() / 2) else {
            continue;
        };
        println!(
            "{}\t{}\t{}\t{time:.0}\t{:.0}\t{:.0}\t{}\t{bytes}\t{}\t{}",
            case.name,
            sorted.len(),
            case.calls(),
            sorted[0],
            sorted[sorted.len() - 1],
            per_rule(time),
            case.size.rules,
            case.census,
        );
    }
}
