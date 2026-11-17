//! The limits under which adoption admits a sheet, an inserted rule group and a
//! replacement, and which refusal arrives first when two limits are exceeded
//! at different rules.
//!
//! Every figure is the smallest value of one `CssomLimits` field, with the others
//! at their defaults, at which the operation succeeds; one below it the operation
//! is refused with the recorded error. They were found once by bisection. The
//! identities issued before a refusal are observed through `max_identity`: the
//! smallest value that still lets the refusal itself (not identity exhaustion)
//! arrive is the last identity issued.
use std::sync::atomic::AtomicBool;
use surgeist_cssom::*;

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

/// One operation whose admission is characterized.
#[derive(Clone)]
enum Scenario {
    /// Parse `text` into a new sheet and commit it.
    Create { text: String },
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
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}

/// The error without its payload text: a limit with its resource and maximum,
/// any other refusal by its variant name.
fn describe(error: &CssomError) -> String {
    match error {
        CssomError::Limit { resource, maximum } => format!("{resource} {maximum}"),
        other => format!("{other:?}")
            .chars()
            .take_while(|c| c.is_alphanumeric())
            .collect(),
    }
}

impl Scenario {
    fn run(&self, limits: CssomLimits) -> Result<(), CssomError> {
        let mut store = CssomStore::new(limits, CssomContext::default())?;
        match self {
            Scenario::Create { text } => {
                let mut edit = batch(&store);
                edit.create_parsed_sheet(text, CssomSheetInputs::external(version("a"), true))?;
                store.commit(edit)?;
            }
            Scenario::Insert { base, text } => {
                let mut edit = batch(&store);
                let ticket =
                    edit.create_parsed_sheet(base, CssomSheetInputs::external(version("a"), true))?;
                let sheet = store.commit(edit)?.sheet(&ticket).unwrap().clone();
                let mut edit = batch(&store);
                edit.insert_sheet_rule(&sheet, text.as_str(), 0, &AtomicBool::new(false))?;
                store.commit(edit)?;
            }
            Scenario::Replace { base, text } => {
                let inputs =
                    CssomSheetInputs::constructed(version("a"), "document".to_owned(), None);
                let mut edit = batch(&store);
                let ticket = edit.create_parsed_sheet(base, inputs)?;
                let sheet = store.commit(edit)?.sheet(&ticket).unwrap().clone();
                let mut edit = batch(&store);
                edit.replace_sheet_text(&sheet, text)?;
                store.commit(edit)?;
            }
        }
        Ok(())
    }
    fn outcome(&self, limits: CssomLimits) -> Result<(), String> {
        self.run(limits).map_err(|error| describe(&error))
    }
    fn with(&self, field: Field, value: u64) -> Result<(), String> {
        let mut limits = CssomLimits::default();
        field.set(&mut limits, value);
        self.outcome(limits)
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

const SHAPES: [(&str, &str); 9] = [
    ("flat", FLAT),
    ("nested", NESTED),
    ("groups", GROUPS),
    ("mixed", MIXED),
    ("named_supports", NAMED_SUPPORTS),
    ("custom_media", CUSTOM_MEDIA),
    ("layers", LAYERS),
    ("imports", IMPORTS),
    ("recovery", RECOVERY),
];

/// The smallest value of `field` at which the scenario succeeds, by bisection.
fn smallest(scenario: &Scenario, field: Field) -> u64 {
    let (mut low, mut high) = (0u64, field.default_value());
    assert!(scenario.with(field, high).is_ok());
    while low < high {
        let middle = low + (high - low) / 2;
        if scenario.with(field, middle).is_ok() {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// The outcome under `limits` and the last identity issued before it: the
/// smallest `max_identity` at which the outcome is not identity exhaustion.
fn issued(scenario: &Scenario, limits: &CssomLimits) -> (Result<(), String>, u64) {
    let outcome_at = |value: u64| {
        let mut limits = limits.clone();
        limits.max_identity = value;
        scenario.outcome(limits)
    };
    let exhausted =
        |outcome: &Result<(), String>| outcome.as_ref().is_err_and(|e| e == "IdentityExhausted");
    let (mut low, mut high) = (0u64, u64::MAX);
    assert!(!exhausted(&outcome_at(high)));
    while low < high {
        let middle = low + (high - low) / 2;
        if exhausted(&outcome_at(middle)) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    (outcome_at(low), low)
}

const INSERT_BASE: &str = ".a { color: red }
.b { color: blue }
";
const INSERTED: &str = "@media screen { .x { color: red } @supports (display: grid) { .y { margin: 1px 0 } } .z { color: blue } }";
const REPLACEMENT: &str = "@media screen { .x { color: red } .y { color: blue } }
.z { margin: 1px 0 }
@keyframes k { from { color: red } to { color: blue } }
";

/// Every characterized operation by name.
fn scenarios() -> Vec<(&'static str, Scenario)> {
    let mut all = SHAPES
        .iter()
        .map(|(name, text)| {
            (
                *name,
                Scenario::Create {
                    text: (*text).to_owned(),
                },
            )
        })
        .collect::<Vec<_>>();
    all.push((
        "insert",
        Scenario::Insert {
            base: INSERT_BASE.to_owned(),
            text: INSERTED.to_owned(),
        },
    ));
    all.push((
        "replace",
        Scenario::Replace {
            base: INSERT_BASE.to_owned(),
            text: REPLACEMENT.to_owned(),
        },
    ));
    all
}

#[derive(Clone, Copy)]
enum Kind {
    Create,
    Insert,
    Replace,
}
#[derive(Clone, Copy)]
enum Body {
    Flat,
    Named,
}
fn body(shape: Body, rules: usize) -> String {
    (0..rules)
        .map(|k| match shape {
            Body::Flat => format!(".r{k} {{ color: red }}\n"),
            Body::Named => {
                format!("@counter-style c{k:03} {{ system: cyclic; symbols: \"*\" }}\n")
            }
        })
        .collect()
}
fn scenario(kind: Kind, shape: Body, rules: usize) -> Scenario {
    let text = body(shape, rules);
    match kind {
        Kind::Create => Scenario::Create { text },
        Kind::Insert => Scenario::Insert {
            base: INSERT_BASE.to_owned(),
            text: format!("@media screen {{ {text} }}"),
        },
        Kind::Replace => Scenario::Replace {
            base: INSERT_BASE.to_owned(),
            text,
        },
    }
}

/// A sheet of `RULES` rules in which `late` is exceeded at a later rule than
/// `early`, although `late` comes first in the order the state census checks.
struct Pair {
    name: &'static str,
    kind: Kind,
    shape: Body,
    late: (Field, usize),
    early: (Field, usize),
}
const RULES: usize = 24;
const PAIRS: [Pair; 8] = [
    Pair {
        name: "objects after entries",
        kind: Kind::Create,
        shape: Body::Flat,
        late: (Field::Objects, 18),
        early: (Field::Entries, 6),
    },
    Pair {
        name: "objects after steps",
        kind: Kind::Create,
        shape: Body::Flat,
        late: (Field::Objects, 18),
        early: (Field::SourceSteps, 6),
    },
    Pair {
        name: "entries after steps",
        kind: Kind::Create,
        shape: Body::Flat,
        late: (Field::Entries, 18),
        early: (Field::SourceSteps, 6),
    },
    Pair {
        name: "objects after strings",
        kind: Kind::Create,
        shape: Body::Named,
        late: (Field::Objects, 18),
        early: (Field::StringBytes, 6),
    },
    Pair {
        name: "entries after strings",
        kind: Kind::Create,
        shape: Body::Named,
        late: (Field::Entries, 18),
        early: (Field::StringBytes, 6),
    },
    Pair {
        name: "steps after strings",
        kind: Kind::Create,
        shape: Body::Named,
        late: (Field::SourceSteps, 18),
        early: (Field::StringBytes, 6),
    },
    Pair {
        name: "insert objects after steps",
        kind: Kind::Insert,
        shape: Body::Flat,
        late: (Field::Objects, 18),
        early: (Field::SourceSteps, 6),
    },
    Pair {
        name: "replace objects after entries",
        kind: Kind::Replace,
        shape: Body::Flat,
        late: (Field::Objects, 18),
        early: (Field::Entries, 6),
    },
];

/// The scenario, the field, the smallest value of the field at which the
/// scenario succeeds, and the refusal one below it.
const SMALLEST: [(&str, Field, u64, &str); 99] = [
    ("flat", Field::Objects, 22, "objects 21"),
    ("flat", Field::Entries, 33, "entries 32"),
    (
        "flat",
        Field::StringBytes,
        1,
        "host and live string bytes 0",
    ),
    ("flat", Field::InputBytes, 128, "parse input bytes 127"),
    (
        "flat",
        Field::RetainedSourceBytes,
        192,
        "retained parse source bytes 191",
    ),
    (
        "flat",
        Field::SourceSteps,
        1442,
        "retained source traversal steps 1441",
    ),
    (
        "flat",
        Field::SourceDepth,
        12,
        "retained source traversal depth 11",
    ),
    ("flat", Field::RuleDepth, 1, "rule depth 0"),
    ("flat", Field::Identity, 22, "IdentityExhausted"),
    ("nested", Field::Objects, 22, "objects 21"),
    ("nested", Field::Entries, 22, "entries 21"),
    (
        "nested",
        Field::StringBytes,
        1,
        "host and live string bytes 0",
    ),
    ("nested", Field::InputBytes, 113, "parse input bytes 112"),
    (
        "nested",
        Field::RetainedSourceBytes,
        177,
        "retained parse source bytes 176",
    ),
    (
        "nested",
        Field::SourceSteps,
        1251,
        "retained source traversal steps 1250",
    ),
    (
        "nested",
        Field::SourceDepth,
        17,
        "retained source traversal depth 16",
    ),
    ("nested", Field::RuleDepth, 3, "rule depth 2"),
    ("nested", Field::Identity, 22, "IdentityExhausted"),
    ("groups", Field::Objects, 35, "objects 34"),
    ("groups", Field::Entries, 34, "entries 33"),
    (
        "groups",
        Field::StringBytes,
        1,
        "host and live string bytes 0",
    ),
    ("groups", Field::InputBytes, 260, "parse input bytes 259"),
    (
        "groups",
        Field::RetainedSourceBytes,
        388,
        "retained parse source bytes 387",
    ),
    (
        "groups",
        Field::SourceSteps,
        2100,
        "retained source traversal steps 2099",
    ),
    (
        "groups",
        Field::SourceDepth,
        17,
        "retained source traversal depth 16",
    ),
    ("groups", Field::RuleDepth, 3, "rule depth 2"),
    ("groups", Field::Identity, 35, "IdentityExhausted"),
    ("mixed", Field::Objects, 32, "objects 31"),
    ("mixed", Field::Entries, 53, "entries 52"),
    (
        "mixed",
        Field::StringBytes,
        26,
        "host and live string bytes 25",
    ),
    ("mixed", Field::InputBytes, 404, "parse input bytes 403"),
    (
        "mixed",
        Field::RetainedSourceBytes,
        532,
        "retained parse source bytes 531",
    ),
    (
        "mixed",
        Field::SourceSteps,
        2268,
        "retained source traversal steps 2267",
    ),
    (
        "mixed",
        Field::SourceDepth,
        15,
        "retained source traversal depth 14",
    ),
    ("mixed", Field::RuleDepth, 2, "rule depth 1"),
    ("mixed", Field::Identity, 32, "IdentityExhausted"),
    ("named_supports", Field::Objects, 15, "objects 14"),
    ("named_supports", Field::Entries, 70, "entries 69"),
    (
        "named_supports",
        Field::StringBytes,
        1631,
        "host and live string bytes 1630",
    ),
    (
        "named_supports",
        Field::InputBytes,
        116,
        "parse input bytes 115",
    ),
    (
        "named_supports",
        Field::RetainedSourceBytes,
        180,
        "retained parse source bytes 179",
    ),
    (
        "named_supports",
        Field::SourceSteps,
        858,
        "retained source traversal steps 857",
    ),
    (
        "named_supports",
        Field::SourceDepth,
        12,
        "retained source traversal depth 11",
    ),
    ("named_supports", Field::RuleDepth, 2, "Format"),
    ("named_supports", Field::Identity, 15, "IdentityExhausted"),
    ("custom_media", Field::Objects, 14, "objects 13"),
    ("custom_media", Field::Entries, 14, "entries 13"),
    (
        "custom_media",
        Field::StringBytes,
        24,
        "host and live string bytes 23",
    ),
    (
        "custom_media",
        Field::InputBytes,
        140,
        "parse input bytes 139",
    ),
    (
        "custom_media",
        Field::RetainedSourceBytes,
        204,
        "retained parse source bytes 203",
    ),
    (
        "custom_media",
        Field::SourceSteps,
        604,
        "retained source traversal steps 603",
    ),
    (
        "custom_media",
        Field::SourceDepth,
        14,
        "retained source traversal depth 13",
    ),
    ("custom_media", Field::RuleDepth, 2, "rule depth 1"),
    ("custom_media", Field::Identity, 14, "IdentityExhausted"),
    ("layers", Field::Objects, 15, "objects 14"),
    ("layers", Field::Entries, 13, "entries 12"),
    (
        "layers",
        Field::StringBytes,
        1,
        "host and live string bytes 0",
    ),
    ("layers", Field::InputBytes, 90, "parse input bytes 89"),
    (
        "layers",
        Field::RetainedSourceBytes,
        154,
        "retained parse source bytes 153",
    ),
    (
        "layers",
        Field::SourceSteps,
        424,
        "retained source traversal steps 423",
    ),
    (
        "layers",
        Field::SourceDepth,
        14,
        "retained source traversal depth 13",
    ),
    ("layers", Field::RuleDepth, 2, "rule depth 1"),
    ("layers", Field::Identity, 15, "IdentityExhausted"),
    ("imports", Field::Objects, 11, "objects 10"),
    ("imports", Field::Entries, 10, "entries 9"),
    (
        "imports",
        Field::StringBytes,
        1,
        "host and live string bytes 0",
    ),
    ("imports", Field::InputBytes, 97, "parse input bytes 96"),
    (
        "imports",
        Field::RetainedSourceBytes,
        161,
        "retained parse source bytes 160",
    ),
    (
        "imports",
        Field::SourceSteps,
        514,
        "retained source traversal steps 513",
    ),
    (
        "imports",
        Field::SourceDepth,
        12,
        "retained source traversal depth 11",
    ),
    ("imports", Field::RuleDepth, 1, "rule depth 0"),
    ("imports", Field::Identity, 11, "IdentityExhausted"),
    ("recovery", Field::Objects, 10, "objects 9"),
    ("recovery", Field::Entries, 10, "entries 9"),
    (
        "recovery",
        Field::StringBytes,
        1,
        "host and live string bytes 0",
    ),
    ("recovery", Field::InputBytes, 84, "parse input bytes 83"),
    (
        "recovery",
        Field::RetainedSourceBytes,
        148,
        "retained parse source bytes 147",
    ),
    (
        "recovery",
        Field::SourceSteps,
        331,
        "retained source traversal steps 330",
    ),
    (
        "recovery",
        Field::SourceDepth,
        12,
        "retained source traversal depth 11",
    ),
    ("recovery", Field::RuleDepth, 1, "rule depth 0"),
    ("recovery", Field::Identity, 10, "IdentityExhausted"),
    ("insert", Field::Objects, 24, "objects 23"),
    ("insert", Field::Entries, 52, "ParseResource"),
    (
        "insert",
        Field::StringBytes,
        1,
        "host and live string bytes 0",
    ),
    ("insert", Field::InputBytes, 105, "ParseResource"),
    (
        "insert",
        Field::RetainedSourceBytes,
        270,
        "retained parse source bytes 269",
    ),
    (
        "insert",
        Field::SourceSteps,
        1466,
        "retained source traversal steps 1465",
    ),
    (
        "insert",
        Field::SourceDepth,
        16,
        "retained source traversal depth 15",
    ),
    ("insert", Field::RuleDepth, 3, "ParseResource"),
    ("insert", Field::Identity, 24, "IdentityExhausted"),
    ("replace", Field::Objects, 28, "objects 27"),
    ("replace", Field::Entries, 29, "entries 28"),
    (
        "replace",
        Field::StringBytes,
        10,
        "host and live string bytes 9",
    ),
    ("replace", Field::InputBytes, 132, "parse input bytes 131"),
    (
        "replace",
        Field::RetainedSourceBytes,
        297,
        "retained parse source bytes 296",
    ),
    (
        "replace",
        Field::SourceSteps,
        1550,
        "retained source traversal steps 1549",
    ),
    (
        "replace",
        Field::SourceDepth,
        15,
        "retained source traversal depth 14",
    ),
    ("replace", Field::RuleDepth, 2, "rule depth 1"),
    ("replace", Field::Identity, 28, "IdentityExhausted"),
];

/// The pair, the value of the later-rule limit, the value of the earlier-rule
/// limit, the refusal and the last identity issued before it.
const PINNED: [(&str, u64, u64, &str, u64); 8] = [
    ("objects after entries", 58, 25, "entries 25", 31),
    (
        "objects after steps",
        58,
        951,
        "retained source traversal steps 951",
        28,
    ),
    (
        "entries after steps",
        73,
        951,
        "retained source traversal steps 951",
        28,
    ),
    (
        "objects after strings",
        40,
        25,
        "host and live string bytes 25",
        18,
    ),
    (
        "entries after strings",
        73,
        25,
        "host and live string bytes 25",
        18,
    ),
    (
        "steps after strings",
        4893,
        25,
        "host and live string bytes 25",
        18,
    ),
    (
        "insert objects after steps",
        67,
        1456,
        "retained source traversal steps 1456",
        37,
    ),
    ("replace objects after entries", 64, 29, "entries 29", 43),
];

#[test]
fn every_limit_admits_each_operation_at_its_smallest_value_and_refuses_one_below() {
    let scenarios = scenarios();
    for (name, field, value, refusal) in SMALLEST {
        let scenario = &scenarios
            .iter()
            .find(|(known, _)| *known == name)
            .unwrap()
            .1;
        assert_eq!(
            scenario.with(field, value),
            Ok(()),
            "{name} {field:?} at {value}"
        );
        assert_eq!(
            scenario.with(field, value - 1),
            Err(refusal.to_owned()),
            "{name} {field:?} at {}",
            value - 1
        );
    }
}

#[test]
fn the_limit_exceeded_at_the_earlier_rule_is_refused_whatever_the_census_order() {
    for (pair, (name, late, early, refusal, last)) in PAIRS.iter().zip(PINNED) {
        assert_eq!(pair.name, name);
        let full = scenario(pair.kind, pair.shape, RULES);
        // Each limit is the smallest that admits a prefix of the sheet.
        let prefix = |(field, rules): (Field, usize)| {
            smallest(&scenario(pair.kind, pair.shape, rules), field)
        };
        assert_eq!(prefix(pair.late), late, "{name}");
        assert_eq!(prefix(pair.early), early, "{name}");
        let limits_of = |fields: &[(Field, u64)]| {
            let mut limits = CssomLimits::default();
            for (field, value) in fields {
                field.set(&mut limits, *value);
            }
            limits
        };
        let (late_outcome, late_last) = issued(&full, &limits_of(&[(pair.late.0, late)]));
        let (early_outcome, early_last) = issued(&full, &limits_of(&[(pair.early.0, early)]));
        // Alone, each limit refuses the sheet, the later-rule one after more identities.
        assert!(late_outcome.is_err() && early_outcome.is_err(), "{name}");
        assert!(early_last < late_last, "{name}");
        // Together, the refusal is the one that arrives first, at the same identity.
        let both = issued(
            &full,
            &limits_of(&[(pair.late.0, late), (pair.early.0, early)]),
        );
        assert_eq!(both, (early_outcome, early_last), "{name}");
        assert_eq!(both, (Err(refusal.to_owned()), last), "{name}");
    }
}
