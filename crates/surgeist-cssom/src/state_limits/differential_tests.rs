//! The incremental census of adoption against the census of the whole state
//! after every adopted rule, which is the reference.
//!
//! Every run builds the same store twice, once with each check, and compares
//! the outcome, the whole error value, the identities issued and the staged
//! state left behind. The limits come from the smallest value of each limit
//! that admits the operation (found with the reference) and values around it,
//! alone and in pairs. While the incremental check runs, every adopted rule is
//! also cross-checked against a census of the whole state.
use super::{Census, Extension, Inserted, probe};
use crate::{model::State, *};
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Field {
    Objects,
    Entries,
    StringBytes,
    InputBytes,
    RetainedSourceBytes,
    SourceSteps,
    SourceDepth,
    RuleDepth,
    Identity,
}
impl Field {
    /// The limits the census checks, and the rule depth adoption checks.
    const CHECKED: [Field; 8] = [
        Field::Objects,
        Field::Entries,
        Field::StringBytes,
        Field::InputBytes,
        Field::RetainedSourceBytes,
        Field::SourceSteps,
        Field::SourceDepth,
        Field::RuleDepth,
    ];
    fn set(self, limits: &mut CssomLimits, value: u64) {
        let size = usize::try_from(value).unwrap_or(usize::MAX);
        match self {
            Field::Objects => limits.max_objects = size,
            Field::Entries => limits.max_entries = size,
            Field::StringBytes => limits.max_string_bytes = size,
            Field::InputBytes => limits.max_input_bytes = size,
            Field::RetainedSourceBytes => limits.max_retained_source_bytes = size,
            Field::SourceSteps => limits.max_source_traversal_steps = size,
            Field::SourceDepth => limits.max_source_traversal_depth = size,
            Field::RuleDepth => limits.max_depth = size,
            Field::Identity => limits.max_identity = value,
        }
    }
    fn default_value(self) -> u64 {
        let defaults = CssomLimits::default();
        match self {
            Field::Objects => defaults.max_objects as u64,
            Field::Entries => defaults.max_entries as u64,
            Field::StringBytes => defaults.max_string_bytes as u64,
            Field::InputBytes => defaults.max_input_bytes as u64,
            Field::RetainedSourceBytes => defaults.max_retained_source_bytes as u64,
            Field::SourceSteps => defaults.max_source_traversal_steps as u64,
            Field::SourceDepth => defaults.max_source_traversal_depth as u64,
            Field::RuleDepth => defaults.max_depth as u64,
            Field::Identity => defaults.max_identity,
        }
    }
}

#[derive(Clone)]
enum Scenario {
    /// Parse `text` into a new sheet of a store whose context may link a snapshot.
    Create { text: String, linked: bool },
    /// Commit a sheet of `base`, then insert `text` at its start and commit.
    Insert { base: String, text: String },
    /// Commit a constructed sheet of `base`, then replace its text and commit.
    Replace { base: String, text: String },
}

fn version(name: &str) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: name.into(),
        revision: 1,
    }
}

/// A context linking the snapshot of a store that holds one sheet.
fn linked_context() -> CssomContext {
    let mut donor = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = donor
        .batch(CssomEditGuard::from_snapshot(&donor.snapshot()))
        .unwrap();
    let ticket = edit
        .create_parsed_sheet(
            "donor { color: red }",
            CssomSheetInputs::external(version("donor"), true),
        )
        .unwrap();
    let sheet = donor.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    CssomContext {
        inputs: Vec::new(),
        linked: vec![CssomLinkedInput {
            version: version("link"),
            snapshot: donor.snapshot(),
            sheet,
        }],
    }
}

/// What a run leaves behind: its outcome, the identities issued and the
/// staged state, rendered so that two stores compare.
#[derive(Debug, Eq, PartialEq)]
struct Run {
    result: Result<(), String>,
    issued: u64,
    state: Vec<String>,
}

fn fingerprint(state: &State) -> Vec<String> {
    let mut lines = vec![
        format!("allocation {}", state.last_allocation),
        format!("order {:?}", state.sheet_order),
    ];
    let mut records = Vec::new();
    records.extend(
        state
            .sheets
            .iter()
            .map(|(id, record)| (id.serial, format!("sheet {id:?} {record:?}"))),
    );
    records.extend(
        state
            .rules
            .iter()
            .map(|(id, record)| (id.serial, format!("rule {id:?} {record:?}"))),
    );
    records.extend(
        state
            .blocks
            .iter()
            .map(|(id, record)| (id.serial, format!("block {id:?} {record:?}"))),
    );
    records.extend(
        state
            .lists
            .iter()
            .map(|(id, record)| (id.serial, format!("list {id:?} {record:?}"))),
    );
    records.extend(
        state
            .media
            .iter()
            .map(|(id, record)| (id.serial, format!("media {id:?} {record:?}"))),
    );
    records.extend(
        state
            .maps
            .iter()
            .map(|(id, record)| (id.serial, format!("map {id:?} {record:?}"))),
    );
    records.sort();
    lines.extend(records.into_iter().map(|(_, line)| line));
    lines
}

/// Runs one edit in a batch of `store`, commits it when it succeeds and
/// returns its outcome with what the batch left staged.
fn edit<T>(
    store: &mut CssomStore,
    operation: impl FnOnce(&mut CssomBatch) -> Result<T, CssomError>,
) -> (Result<T, CssomError>, u64, Vec<String>) {
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let result = operation(&mut batch);
    let issued = batch.staged.identity_issuer.load(Ordering::Relaxed);
    let state = fingerprint(&batch.staged);
    if result.is_ok() {
        store.commit(batch).unwrap();
    }
    (result, issued, state)
}

impl Scenario {
    /// Whether every retained payload shares one source snapshot, so that the
    /// walk steps do not depend on the order objects are walked in.
    fn one_snapshot(&self) -> bool {
        matches!(self, Scenario::Create { .. })
    }
    fn run(&self, limits: &CssomLimits, whole_state: bool) -> Run {
        probe::set_whole_state(whole_state);
        probe::set_exact_steps(self.one_snapshot());
        let context = match self {
            Scenario::Create { linked: true, .. } => linked_context(),
            _ => CssomContext::default(),
        };
        let mut store = match CssomStore::new(limits.clone(), context) {
            Ok(store) => store,
            Err(error) => {
                return Run {
                    result: Err(format!("{error:?}")),
                    issued: 0,
                    state: Vec::new(),
                };
            }
        };
        let (result, issued, state) = match self {
            Scenario::Create { text, .. } => edit(&mut store, |batch| {
                batch
                    .create_parsed_sheet(text, CssomSheetInputs::external(version("a"), true))
                    .map(drop)
            }),
            Scenario::Insert { base, text } => {
                let created = edit(&mut store, |batch| {
                    let inputs = CssomSheetInputs::external(version("a"), true);
                    batch.create_parsed_sheet(base, inputs)
                });
                match created {
                    (Ok(_), ..) => {
                        let sheet = store.snapshot().sheets()[0].clone();
                        edit(&mut store, |batch| {
                            batch
                                .insert_sheet_rule(
                                    &sheet,
                                    text.as_str(),
                                    0,
                                    &AtomicBool::new(false),
                                )
                                .map(drop)
                        })
                    }
                    (Err(error), issued, state) => (Err(error), issued, state),
                }
            }
            Scenario::Replace { base, text } => {
                let created = edit(&mut store, |batch| {
                    let inputs =
                        CssomSheetInputs::constructed(version("a"), "document".to_owned(), None);
                    batch.create_parsed_sheet(base, inputs)
                });
                match created {
                    (Ok(_), ..) => {
                        let sheet = store.snapshot().sheets()[0].clone();
                        edit(&mut store, |batch| batch.replace_sheet_text(&sheet, text))
                    }
                    (Err(error), issued, state) => (Err(error), issued, state),
                }
            }
        };
        Run {
            result: result.map_err(|error| format!("{error:?}")),
            issued,
            state,
        }
    }
    /// The smallest value of `field` at which the reference admits the operation.
    fn smallest(&self, field: Field) -> u64 {
        let admits = |value: u64| {
            let mut limits = CssomLimits::default();
            field.set(&mut limits, value);
            self.run(&limits, true).result.is_ok()
        };
        let (mut low, mut high) = (0u64, field.default_value());
        assert!(admits(high));
        while low < high {
            let middle = low + (high - low) / 2;
            if admits(middle) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        low
    }
    fn assert_agrees(&self, fields: &[(Field, u64)]) {
        let mut limits = CssomLimits::default();
        for (field, value) in fields {
            field.set(&mut limits, *value);
        }
        let reference = self.run(&limits, true);
        let incremental = self.run(&limits, false);
        assert_eq!(incremental, reference, "limits {fields:?}");
    }
}

const FLAT: &str = ".a { color: red }
.b { margin: 1px 0; color: blue }
.c { color: green }
.d { margin: 2px }
.e { color: red }
.f { color: blue }
";
const NESTED: &str = ".a { color: red; .b { color: blue; .c { margin: 1px 0 } } }
.d { & .e { color: green; &:hover { color: red } } }
";
const GROUPS: &str =
    "@media screen { .a { color: red } @supports (display: grid) { .b { margin: 1px 0 } } }
@supports (color: red) { .c { color: blue } }
@container (min-width: 10px) { .d { color: red } }
@layer base { .e { color: red } }
@scope (.f) to (.g) { .h { color: red } }
";
const MIXED: &str = "@font-face { font-family: \"Sample\"; src: url(sample.woff2) format(\"woff2\"); font-weight: 400 }
@keyframes pulse { from { color: red } 50% { color: blue; margin: 1px 0 } to { color: green } }
@page :first { margin: 1cm; @top-left { content: \"a\" } }
@counter-style marks { system: cyclic; symbols: \"*\" \"+\" }
@font-feature-values Sample { @styleset { nice: 1 2 } @swash { fancy: 3 } }
.after { color: red }
";
const NAMED_SUPPORTS: &str = "@supports-condition --x { first: a; @future { } last: z; }
@supports-condition --y { color: red }
.a { color: red }
";
const CUSTOM_MEDIA: &str = "@custom-media --narrow (max-width: 30em);
@custom-media --always true;
@custom-media --never false;
@media (--narrow) { .a { color: red } }
";
const LAYERS: &str = "@layer base, theme;
@layer base { .a { color: red } }
@layer theme { .b { color: blue } }
";
const IMPORTS: &str = "@import url(\"a.css\") screen;
@import \"b.css\" layer(x) supports(display: grid);
.a { color: red }
";
const RECOVERY: &str = ".a { color: red } } @bogus x;
.b { color: : blue; margin: 1px }
.c { color: green }
";
const BASE: &str = ".a { color: red }
.b { color: blue }
";

fn scenarios() -> Vec<(&'static str, Scenario)> {
    let create = |text: &str, linked| Scenario::Create {
        text: text.to_owned(),
        linked,
    };
    let many = (0..30)
        .map(|index| format!(".r{index} {{ color: red; .n {{ margin: {index}px 0 }} }}\n"))
        .collect::<String>();
    vec![
        ("flat", create(FLAT, false)),
        ("nested", create(NESTED, false)),
        ("groups", create(GROUPS, false)),
        ("mixed", create(MIXED, false)),
        ("named supports", create(NAMED_SUPPORTS, false)),
        ("custom media", create(CUSTOM_MEDIA, false)),
        ("layers", create(LAYERS, false)),
        ("imports", create(IMPORTS, false)),
        ("recovery", create(RECOVERY, false)),
        ("many nested", create(&many, false)),
        ("linked context", create(FLAT, true)),
        (
            "insert",
            Scenario::Insert {
                base: BASE.to_owned(),
                text: "@media screen { .x { color: red } @supports (display: grid) { .y { margin: 1px 0 } } .z { color: blue } }".to_owned(),
            },
        ),
        (
            "replace",
            Scenario::Replace {
                base: BASE.to_owned(),
                text: "@media screen { .x { color: red } .y { color: blue } }
.z { margin: 1px 0 }
@keyframes k { from { color: red } to { color: blue } }
"
                .to_owned(),
            },
        ),
    ]
}

fn singles(smallest: u64) -> Vec<u64> {
    let mut values = vec![0, smallest.saturating_sub(1), smallest, smallest + 1];
    values.dedup();
    values
}
fn halves(smallest: u64) -> Vec<u64> {
    let mut values = vec![smallest / 2, smallest.saturating_sub(1), smallest];
    values.dedup();
    values
}

#[test]
fn every_single_limit_around_its_smallest_value_gives_the_reference_outcome() {
    let mut runs = 0usize;
    for (name, scenario) in scenarios() {
        for field in Field::CHECKED.into_iter().chain([Field::Identity]) {
            if field == Field::SourceSteps && !scenario.one_snapshot() {
                continue;
            }
            for value in singles(scenario.smallest(field)) {
                scenario.assert_agrees(&[(field, value)]);
                runs += 1;
            }
        }
        eprintln!("{name}: single limits agree");
    }
    assert!(runs > 300, "{runs} single-limit comparisons");
}

#[test]
fn every_pair_of_limits_around_their_smallest_values_gives_the_reference_outcome() {
    let mut runs = 0usize;
    for (name, scenario) in scenarios() {
        let fields = Field::CHECKED
            .into_iter()
            .filter(|field| *field != Field::SourceSteps || scenario.one_snapshot())
            .map(|field| (field, halves(scenario.smallest(field))))
            .collect::<Vec<_>>();
        for (index, (first, first_values)) in fields.iter().enumerate() {
            for (second, second_values) in &fields[index + 1..] {
                for a in first_values {
                    for b in second_values {
                        scenario.assert_agrees(&[(*first, *a), (*second, *b)]);
                        runs += 1;
                    }
                }
            }
        }
        eprintln!("{name}: limit pairs agree");
    }
    assert!(runs > 2000, "{runs} pair comparisons");
}

/// The rules each adoption of a scenario adopts, counted from its text: every
/// rule record created, nested rules, margin rules, keyframe blocks and test
/// children included. Creating a sheet is one adoption; an insert and a
/// replacement are a second one after the base sheet.
fn adopted_rules(name: &str) -> &'static [usize] {
    match name {
        // Six rules.
        "flat" => &[6],
        // The donor store whose snapshot the context links adopts one rule; then six.
        "linked context" => &[1, 6],
        // `.a .b .c` and `.d .e &:hover`.
        "nested" => &[6],
        // `@media` with `.a`, `@supports` and `.b`; `@supports` with `.c`; `@container`
        // with `.d`; `@layer` with `.e`; `@scope` with `.h`.
        "groups" => &[12],
        // `@font-face`; `@keyframes` and three blocks; `@page` and one margin rule;
        // `@counter-style`; `@font-feature-values`; `.after`.
        "mixed" => &[10],
        // `--x` and three test children; `--y` and one; `.a`.
        "named supports" => &[7],
        // Three `@custom-media`; `@media` with `.a`.
        "custom media" => &[5],
        // `@layer base, theme;`; two layer blocks with one rule each.
        "layers" => &[5],
        // Two `@import`; `.a`.
        "imports" => &[3],
        // Recovery keeps two of the three rules.
        "recovery" => &[2],
        // Thirty rules, each with one nested rule.
        "many nested" => &[60],
        // The base sheet of two rules, then `@media` with `.x`, `@supports`, `.y` and `.z`.
        "insert" => &[2, 5],
        // The base sheet of two rules, then `@media` with `.x` and `.y`, `.z`, and
        // `@keyframes` with two blocks.
        "replace" => &[2, 7],
        other => panic!("no adopted rule count for {other}"),
    }
}

#[test]
fn every_scenario_walks_the_whole_state_once_per_adoption_and_extends_for_every_other_rule() {
    for (name, scenario) in scenarios() {
        let adoptions = adopted_rules(name);
        probe::reset();
        let run = scenario.run(&CssomLimits::default(), false);
        assert_eq!(run.result, Ok(()), "{name}");
        // The first check of an adoption is the census of the whole state; each
        // later check of it extends that census.
        assert_eq!(
            probe::whole_runs(),
            adoptions.len(),
            "{name}: whole-state runs"
        );
        assert_eq!(
            probe::extensions(),
            adoptions.iter().map(|rules| rules - 1).sum::<usize>(),
            "{name}: extensions"
        );
    }
}

fn staged_state() -> (State, CssomLimits) {
    let store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    ((*store.state).clone(), CssomLimits::default())
}

fn rule(source: &str) -> CssomAuthoredRule {
    let (sheet, _) = surgeist_css::CssParser::default()
        .parse_sheet(source)
        .into_parts();
    CssomAuthoredRule::Rule(sheet.rules()[0].clone())
}

#[test]
fn inserted_objects_that_link_states_or_are_missing_leave_the_census_to_the_whole_state() {
    let (mut state, limits) = staged_state();
    let id = state
        .adopt(rule("a { color: red }"), CssomParent::Detached, 1, &limits)
        .unwrap();
    let mut census = Census::of(&state, &limits, std::iter::empty()).unwrap();
    let before = census.totals.clone();
    let steps = census.sources.steps();

    // A rule with selector inputs reaches the states its inputs link.
    state.rules.get_mut(&id).unwrap().selector_inputs = Some(Arc::new(CssomContext::default()));
    let mut inserted = Inserted::default();
    inserted.rule(id.clone());
    assert_eq!(
        census.extend(&state, &inserted, &limits).unwrap(),
        Extension::WholeStateNeeded
    );

    // An identity the state does not hold.
    let mut inserted = Inserted::default();
    inserted.block(CssomBlockId {
        owner: state.owner.clone(),
        serial: u64::MAX,
    });
    assert_eq!(
        census.extend(&state, &inserted, &limits).unwrap(),
        Extension::WholeStateNeeded
    );
    assert_eq!(census.totals.sums(), before.sums());
    assert_eq!(census.sources.steps(), steps);
}
