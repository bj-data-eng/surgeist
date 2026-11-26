//! The census after every class of edit, and after seeded sequences of edits,
//! against the frozen reference census.
//!
//! Each scenario builds a store that retains several source snapshots (two
//! sheets parsed from different texts and a rule inserted into one of them
//! from a third), runs its edits one batch at a time and compares
//!
//! - after every operation, the staged state the operation left, and after
//!   every commit the published state: the census the crate runs and the
//!   reference answer alike (the outcome, the whole error value, the figures
//!   and the work charged), under the default limits and under limits around
//!   the smallest value of each limit that admits the state, alone and in
//!   pairs;
//! - the whole run (every operation's outcome, the identities issued, the
//!   staged state and the published state) when the crate's census is replaced
//!   by the reference, under the default limits and, for each operation, under
//!   limits around the smallest value that admits what the operation leaves.
use super::{Field, fingerprint, version};
use crate::state_limits::{Census, probe, reference};
use crate::{model::State, *};
use std::cell::Cell;
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_css::CssParserContext;
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::rule::counter_style::CssCounterStyleDescriptorKind;
use surgeist_css::rule::font_feature_values::CssFontFeatureValueKind;

/// The comparisons a test made, reported so the size of the harness is known.
#[derive(Clone, Copy, Debug, Default)]
struct Tally {
    scenarios: usize,
    operations: usize,
    states: usize,
    comparisons: usize,
    refusals: usize,
    displaced: usize,
}

thread_local! {
    static TALLY: Cell<Tally> = const { Cell::new(Tally {
        scenarios: 0, operations: 0, states: 0, comparisons: 0, refusals: 0, displaced: 0,
    }) };
}
fn tally(update: impl FnOnce(&mut Tally)) {
    TALLY.with(|cell| {
        let mut value = cell.get();
        update(&mut value);
        cell.set(value);
    });
}

// ---------------------------------------------------------------------------
// One census compared with the reference.

#[derive(Debug, Eq, PartialEq)]
enum Answer {
    Admitted(crate::state_limits::Figures),
    Refused(String),
}

/// An answer and the work the census charged to get it.
#[derive(Debug, Eq, PartialEq)]
struct Measured {
    answer: Answer,
    work: probe::Run,
}

fn measured(state: &State, limits: &CssomLimits, displaced: &[&CssMediaQueryList]) -> Measured {
    probe::set_reference(false);
    let census = Census::of(state, limits, displaced.iter().copied());
    let work = probe::last_run().expect("a census run is recorded");
    let answer = match census {
        Ok(census) => Answer::Admitted(census.figures()),
        Err(error) => Answer::Refused(format!("{error:?}")),
    };
    Measured { answer, work }
}

fn frozen(state: &State, limits: &CssomLimits, displaced: &[&CssMediaQueryList]) -> Measured {
    let run = reference::of(state, limits, displaced.iter().copied());
    let answer = match run.result {
        Ok(census) => Answer::Admitted(census.figures()),
        Err(error) => Answer::Refused(format!("{error:?}")),
    };
    Measured {
        answer,
        work: probe::Run {
            examined: run.examined,
            counted: run.counted,
            steps: run.steps,
        },
    }
}

fn limits_with(fields: &[(Field, u64)]) -> CssomLimits {
    let mut limits = CssomLimits::default();
    for (field, value) in fields {
        field.set(&mut limits, *value);
    }
    limits
}

/// Compares the census with the reference under `limits`.
fn agree(state: &State, displaced: &[&CssMediaQueryList], fields: &[(Field, u64)], at: &str) {
    let limits = limits_with(fields);
    let reference = frozen(state, &limits, displaced);
    let census = measured(state, &limits, displaced);
    assert_eq!(census, reference, "{at}: limits {fields:?}");
    tally(|tally| {
        tally.comparisons += 1;
        tally.refusals += usize::from(matches!(reference.answer, Answer::Refused(_)));
    });
}

/// The smallest value of `field`, in `0..=upper`, at which the reference
/// admits `state`, or `None` when it admits none.
fn smallest(
    state: &State,
    displaced: &[&CssMediaQueryList],
    field: Field,
    upper: u64,
) -> Option<u64> {
    let admits = |value: u64| {
        matches!(
            frozen(state, &limits_with(&[(field, value)]), displaced).answer,
            Answer::Admitted(_)
        )
    };
    if !admits(upper) {
        return None;
    }
    let (mut low, mut high) = (0u64, upper);
    while low < high {
        let middle = low + (high - low) / 2;
        if admits(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    Some(low)
}

/// The smallest value of each limit the census checks at which the reference
/// admits `state`, or an empty list when the default limits refuse it.
fn thresholds(state: &State, displaced: &[&CssMediaQueryList]) -> Vec<(Field, u64)> {
    let default = frozen(state, &CssomLimits::default(), displaced);
    let Answer::Admitted(figures) = default.answer else {
        return Vec::new();
    };
    let to_u64 = |value: usize| u64::try_from(value).unwrap_or(u64::MAX);
    [
        (Field::Objects, to_u64(figures.objects)),
        (Field::Entries, figures.entries.map_or(0, to_u64)),
        (Field::StringBytes, figures.strings.map_or(0, to_u64)),
        (Field::RetainedSourceBytes, to_u64(figures.retained_bytes)),
        (Field::SourceSteps, to_u64(figures.steps)),
        (Field::InputBytes, 1 << 14),
        (Field::SourceDepth, 1 << 6),
        (Field::RuleDepth, 1 << 4),
    ]
    .into_iter()
    .filter_map(|(field, upper)| smallest(state, displaced, field, upper).map(|v| (field, v)))
    .collect()
}

/// Compares the census with the reference under the default limits, under
/// limits around each threshold and under pairs of them.
fn compare_state(state: &State, displaced: &[&CssMediaQueryList], at: &str) {
    agree(state, displaced, &[], at);
    let thresholds = thresholds(state, displaced);
    let below = |(field, value): &(Field, u64)| (*field, value.saturating_sub(1));
    for threshold in &thresholds {
        agree(state, displaced, &[below(threshold)], at);
        agree(state, displaced, &[*threshold], at);
    }
    for (index, first) in thresholds.iter().enumerate() {
        for second in &thresholds[index + 1..] {
            agree(state, displaced, &[below(first), below(second)], at);
            agree(state, displaced, &[below(first), *second], at);
            agree(state, displaced, &[*first, below(second)], at);
        }
    }
    tally(|tally| {
        tally.states += 1;
        tally.displaced += usize::from(!displaced.is_empty());
    });
}

fn displaced_of(batch: &CssomBatch) -> Vec<&CssMediaQueryList> {
    batch
        .media_restorations
        .values()
        .filter_map(crate::media_operations::MediaRestoration::displaced)
        .collect()
}

// ---------------------------------------------------------------------------
// The store every scenario starts from.

const SHEET_A: &str = "@import url(\"b.css\") screen;
@media screen { .m { color: red } @supports (display: grid) { .n { margin: 1px 0 } } }
.a { color: red; margin: 1px 0 }
.b { color: blue; .c { color: green } }
@keyframes pulse { from { color: red } 50% { color: blue } to { color: green } }
@page :first { margin: 1cm; @top-left { content: \"a\" } }
@counter-style marks { system: cyclic; symbols: \"*\" \"+\" }
@font-feature-values Sample { @styleset { nice: 1 2 } @swash { fancy: 3 } }
@font-face { font-family: \"Sample\"; src: url(sample.woff2) format(\"woff2\"); font-weight: 400 }
@supports-condition --x { first: a; @future { } last: z; }
@custom-media --narrow (max-width: 30em);
@layer base { .l { color: red } }
";
const SHEET_B: &str = ".b1 { color: red }
.b2 { margin: 2px; color: blue }
@media print { .b3 { color: green } }
";

fn owner_attributes<'a>(
    revision: u64,
    title: Option<&'a [u16]>,
    media: Option<&'a str>,
) -> CssomSheetOwnerAttributes<'a> {
    CssomSheetOwnerAttributes {
        owner: "style",
        version: CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "style".into(),
            revision,
        },
        title,
        media: CssomOwnerMediaInput::Utf8(media),
    }
}

fn linked_donor_context() -> CssomContext {
    super::linked_context()
}

/// A store holding two sheets from different source texts and a third source
/// snapshot, a rule inserted into the second.
fn fixture() -> CssomStore {
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let mut inputs = CssomSheetInputs::external(version("a"), true);
    inputs.owner = Some("style".into());
    inputs.location = Some("https://example.test/a.css".into());
    inputs.title = Arc::new("Title".encode_utf16().collect());
    inputs.base_url = Some(Arc::new("https://example.test/".encode_utf16().collect()));
    batch.create_parsed_sheet(SHEET_A, inputs).unwrap();
    let constructed = CssomSheetInputs::constructed(version("b"), "document".to_owned(), None);
    batch.create_parsed_sheet(SHEET_B, constructed).unwrap();
    store.commit(batch).unwrap();
    let b = store.snapshot().sheets()[1].clone();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let exact = Arc::new(
        ".inserted { color: green }"
            .encode_utf16()
            .collect::<Vec<_>>(),
    );
    batch
        .insert_sheet_rule(&b, CssomSheetText::Utf16(exact), 0, &AtomicBool::new(false))
        .unwrap();
    store.commit(batch).unwrap();
    store
}

// ---------------------------------------------------------------------------
// The edits.

/// What an edit needs to find in the staged state.
struct Handles {
    sheet_a: CssomSheetId,
    sheet_b: CssomSheetId,
}

type Operation = fn(&mut CssomBatch, &Handles) -> Result<(), CssomError>;

fn serial_sorted<K: Clone, V>(
    entries: impl Iterator<Item = (K, V)>,
    serial: impl Fn(&K) -> u64,
) -> Vec<(K, V)> {
    let mut entries = entries.collect::<Vec<_>>();
    entries.sort_by_key(|(key, _)| serial(key));
    entries
}

/// The rule of lowest identity whose data `pick` accepts.
fn rule_where(state: &State, pick: impl Fn(&CssomRule) -> bool) -> Result<CssomRuleId, CssomError> {
    serial_sorted(state.rules.iter(), |id| id.serial)
        .into_iter()
        .find(|(_, rule)| pick(rule))
        .map(|(id, _)| id.clone())
        .ok_or(CssomError::MissingObject)
}
fn block_where(
    state: &State,
    pick: impl Fn(&CssomBlock) -> bool,
) -> Result<CssomBlockId, CssomError> {
    serial_sorted(state.blocks.iter(), |id| id.serial)
        .into_iter()
        .find(|(_, block)| pick(block))
        .map(|(id, _)| id.clone())
        .ok_or(CssomError::MissingObject)
}
fn map_where(
    state: &State,
    kind: CssFontFeatureValueKind,
) -> Result<CssomFeatureMapId, CssomError> {
    serial_sorted(state.maps.iter(), |id| id.serial)
        .into_iter()
        .find(|(_, map)| map.kind() == kind)
        .map(|(id, _)| id.clone())
        .ok_or(CssomError::MissingObject)
}
fn media_group(state: &State) -> Result<CssomMediaListId, CssomError> {
    serial_sorted(state.rules.iter(), |id| id.serial)
        .into_iter()
        .find_map(|(_, rule)| match rule.data() {
            CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(id),
                ..
            } => Some(id.clone()),
            _ => None,
        })
        .ok_or(CssomError::MissingObject)
}
fn style_rule(state: &State) -> Result<CssomRuleId, CssomError> {
    rule_where(state, |rule| {
        matches!(rule.data(), CssomRuleData::Style { .. })
            && matches!(rule.parent(), CssomParent::Sheet(_))
    })
}
fn style_block(state: &State) -> Result<CssomBlockId, CssomError> {
    match state.rule(&style_rule(state)?)?.data() {
        CssomRuleData::Style { block, .. } => Ok(block.clone()),
        _ => Err(CssomError::WrongKind),
    }
}
fn counter_rule(state: &State) -> Result<CssomRuleId, CssomError> {
    rule_where(state, |rule| {
        matches!(rule.data(), CssomRuleData::CounterStyle { .. })
    })
}
fn named_supports(state: &State) -> Result<CssomRuleId, CssomError> {
    rule_where(state, |rule| {
        matches!(rule.data(), CssomRuleData::NamedSupports { .. })
    })
}

fn set_property(
    batch: &mut CssomBatch,
    block: &CssomBlockId,
    name: &str,
    value: &str,
) -> Result<(), CssomError> {
    let prepared = batch.prepare_set_property(
        block,
        name,
        value,
        "",
        CssomDeclarationRequestLimits::default(),
    )?;
    batch.apply_declaration(prepared, &[]).map(drop)
}

fn inline_context(owner: &str, revision: u64) -> CssomInlineDeclarationContext {
    let support = CssomDeclarationSupport::try_new(
        ["width", "height"]
            .map(|name| {
                (
                    CssPropertyGrammar::from_name(name).unwrap(),
                    CssomUsabilityRequirement::AllCheckedValues,
                )
            })
            .to_vec(),
    )
    .unwrap();
    let versioned = |role, name: &str| CssomInputVersion {
        role,
        identity: format!("{owner}:{name}"),
        revision,
    };
    CssomInlineDeclarationContext::try_new(CssomContext {
        inputs: vec![
            CssomInput {
                version: CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: owner.into(),
                    revision,
                },
                data: CssomInputData::Owner {
                    identity: owner.into(),
                    attribute: Some("style".into()),
                },
            },
            CssomInput {
                version: versioned(CssomInputRole::ParserMode, "parser"),
                data: CssomInputData::ParserMode(CssParserContext::default()),
            },
            CssomInput {
                version: versioned(CssomInputRole::Support, "support"),
                data: CssomInputData::Support(support),
            },
            CssomInput {
                version: versioned(CssomInputRole::Document, "document"),
                data: CssomInputData::Document {
                    identity: format!("document-{owner}"),
                },
            },
            CssomInput {
                version: versioned(CssomInputRole::Base, "base"),
                data: CssomInputData::Base {
                    url: Some(format!("https://example.test/{owner}/{revision}/")),
                    location: None,
                },
            },
        ],
        linked: Vec::new(),
    })
    .unwrap()
}

fn inline_block(state: &State) -> Result<CssomBlockId, CssomError> {
    block_where(state, |block| block.inline_context().is_some())
}

fn proceed() -> AtomicBool {
    AtomicBool::new(false)
}

/// The edits, by class. Each fails with a missing object when the staged state
/// does not hold what it edits, so a sequence of them stays well defined.
fn operations() -> Vec<(&'static str, Operation)> {
    vec![
        ("disable a sheet", |b, h| {
            b.set_disabled(&h.sheet_a, true).map(drop)
        }),
        ("enable a sheet", |b, h| {
            b.set_disabled(&h.sheet_a, false).map(drop)
        }),
        ("owner attributes with a title and media", |b, h| {
            let title = "New title".encode_utf16().collect::<Vec<_>>();
            b.update_sheet_owner_attributes(
                &h.sheet_a,
                owner_attributes(2, Some(&title), Some("screen and (min-width: 10px)")),
            )
            .map(drop)
        }),
        ("owner attributes clearing the title", |b, h| {
            b.update_sheet_owner_attributes(&h.sheet_a, owner_attributes(3, None, None))
                .map(drop)
        }),
        ("set the media of a sheet", |b, h| {
            let media = b.staged.sheet(&h.sheet_a)?.media.clone();
            b.set_media_text(&media, "screen, print and (color)", &proceed())
                .map(drop)
        }),
        ("set the media of a sheet from exact UTF-16", |b, h| {
            let media = b.staged.sheet(&h.sheet_a)?.media.clone();
            let text = Arc::new("tv, (min-width: 3px)".encode_utf16().collect::<Vec<_>>());
            b.set_media_text(&media, CssomSheetText::Utf16(text), &proceed())
                .map(drop)
        }),
        ("displace exact media", |b, h| {
            let media = b.staged.sheet(&h.sheet_a)?.media.clone();
            let text = Arc::new("tv, (min-width: 3px)".encode_utf16().collect::<Vec<_>>());
            b.set_media_text(&media, CssomSheetText::Utf16(text), &proceed())?;
            b.append_medium(&media, "print", &proceed()).map(drop)
        }),
        ("append and delete a medium", |b, h| {
            let media = b.staged.sheet(&h.sheet_a)?.media.clone();
            b.set_media_text(&media, "screen", &proceed())?;
            b.append_medium(&media, "print", &proceed())?;
            b.delete_medium(&media, "screen", &proceed()).map(drop)
        }),
        ("set the media of a group rule", |b, _| {
            let media = media_group(&b.staged)?;
            b.set_media_text(&media, "print, (min-width: 20px)", &proceed())
                .map(drop)
        }),
        ("replace the media list of a group rule", |b, _| {
            let media = media_group(&b.staged)?;
            b.set_media(&media, CssMediaQueryList::new(Vec::new()))
                .map(drop)
        }),
        ("set a style selector", |b, _| {
            let rule = style_rule(&b.staged)?;
            b.set_style_selector_text(&rule, ".changed > .selector, #id")
                .map(drop)
        }),
        ("set a page selector", |b, _| {
            let rule = rule_where(&b.staged, |rule| {
                matches!(rule.data(), CssomRuleData::Page { .. })
            })?;
            b.set_page_selector_text(&rule, ":left").map(drop)
        }),
        ("set a keyframe key", |b, _| {
            let rule = rule_where(&b.staged, |rule| {
                matches!(rule.data(), CssomRuleData::Keyframe { .. })
            })?;
            b.set_keyframe_key_text(&rule, "25%")
        }),
        ("set a keyframes name", |b, _| {
            let rule = rule_where(&b.staged, |rule| {
                matches!(rule.data(), CssomRuleData::Keyframes { .. })
            })?;
            b.set_keyframes_name(&rule, "renamed").map(drop)
        }),
        ("set a counter style name", |b, _| {
            let rule = counter_rule(&b.staged)?;
            b.set_counter_name(&rule, "renamed-marks").map(drop)
        }),
        ("set a counter style descriptor", |b, _| {
            let rule = counter_rule(&b.staged)?;
            b.set_counter_descriptor(
                &rule,
                CssCounterStyleDescriptorKind::Prefix,
                "'P'",
                CssomDeclarationRequestLimits::default(),
            )
            .map(drop)
        }),
        ("set the families of a font feature values rule", |b, _| {
            let rule = rule_where(&b.staged, |rule| {
                matches!(rule.data(), CssomRuleData::FontFeatureValues { .. })
            })?;
            b.set_font_feature_family_text(&rule, "Other, \"Third Family\"")
                .map(drop)
        }),
        ("set and delete a font feature", |b, _| {
            let map = map_where(&b.staged, CssFontFeatureValueKind::Styleset)?;
            b.set_feature(&map, "added", &[5, 6])?;
            b.delete_feature(&map, "nice").map(drop)
        }),
        ("clear a feature map", |b, _| {
            let map = map_where(&b.staged, CssFontFeatureValueKind::Swash)?;
            b.clear_feature_map(&map).map(drop)
        }),
        ("insert into a named supports rule", |b, _| {
            let rule = named_supports(&b.staged)?;
            b.insert_group_rule(&rule, "added: value", 0).map(drop)
        }),
        ("delete from a named supports rule", |b, _| {
            let rule = named_supports(&b.staged)?;
            b.delete_group_rule(&rule, 0).map(drop)
        }),
        ("associate an import", |b, h| {
            let rule = rule_where(&b.staged, |rule| {
                matches!(rule.data(), CssomRuleData::Import { .. })
            })?;
            b.associate_import(
                &rule,
                &h.sheet_b,
                version("import"),
                Some("https://example.test/b.css".into()),
            )
            .map(drop)
        }),
        ("insert a rule into a sheet", |b, h| {
            b.insert_sheet_rule(
                &h.sheet_a,
                "@media (min-width: 5px) { .x { color: red } }",
                1,
                &proceed(),
            )
            .map(drop)
        }),
        ("insert a rule into a sheet from exact UTF-16", |b, h| {
            let text = Arc::new(
                ".exact { color: blue; margin: 0 }"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            );
            b.insert_sheet_rule(&h.sheet_a, CssomSheetText::Utf16(text), 3, &proceed())
                .map(drop)
        }),
        ("insert one exact rule text into two sheets", |b, h| {
            // Two distinct snapshots that share one original UTF-16 backing,
            // which the retained bytes count once.
            let text = Arc::new(
                ".twin { color: teal; margin: 3px }"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            );
            b.insert_sheet_rule(
                &h.sheet_a,
                CssomSheetText::Utf16(text.clone()),
                3,
                &proceed(),
            )?;
            b.insert_sheet_rule(&h.sheet_b, CssomSheetText::Utf16(text), 0, &proceed())
                .map(drop)
        }),
        ("insert a rule into a group", |b, _| {
            let rule = rule_where(&b.staged, |rule| {
                matches!(
                    rule.data(),
                    CssomRuleData::Group {
                        prelude: CssomGroupPrelude::Media(_),
                        ..
                    }
                )
            })?;
            b.insert_group_rule(&rule, ".g { color: red }", 0).map(drop)
        }),
        ("delete a rule of a sheet", |b, h| {
            b.delete_sheet_rule(&h.sheet_a, 2).map(drop)
        }),
        ("set a declaration", |b, _| {
            let block = style_block(&b.staged)?;
            set_property(b, &block, "color", "orange")?;
            set_property(b, &block, "padding", "2px 3px")
        }),
        ("remove a declaration", |b, _| {
            let block = style_block(&b.staged)?;
            b.remove_property(&block, "margin", CssomDeclarationRequestLimits::default())
                .map(drop)
        }),
        ("create inline declarations", |b, _| {
            b.create_inline_declarations(inline_context("element", 1))
                .map(drop)
        }),
        ("create and edit inline declarations", |b, _| {
            let ticket = b.create_inline_declarations(inline_context("edited", 1))?;
            let block = b.created_block_id(&ticket)?.clone();
            set_property(b, &block, "width", "4px")
        }),
        ("update an inline declaration context", |b, _| {
            let block = inline_block(&b.staged)?;
            let owner = b
                .staged
                .block(&block)?
                .inline_context()
                .map(|context| context.owner().identity.clone())
                .ok_or(CssomError::WrongKind)?;
            b.update_inline_declaration_context(&block, inline_context(&owner, 2))
                .map(drop)
        }),
        ("retire inline declarations", |b, _| {
            let block = inline_block(&b.staged)?;
            let owner = b
                .staged
                .block(&block)?
                .owner()
                .cloned()
                .ok_or(CssomError::WrongKind)?;
            b.retire_inline_declarations(&block, &owner)
        }),
        ("create and retire inline declarations", |b, _| {
            let ticket = b.create_inline_declarations(inline_context("brief", 1))?;
            let block = b.created_block_id(&ticket)?.clone();
            let owner = b
                .staged
                .block(&block)?
                .owner()
                .cloned()
                .ok_or(CssomError::WrongKind)?;
            b.retire_inline_declarations(&block, &owner)
        }),
        ("update the context without a linked snapshot", |b, _| {
            let mut context = (*b.staged.context).clone();
            context.inputs.push(CssomInput {
                version: version("window"),
                data: CssomInputData::Document {
                    identity: "document".into(),
                },
            });
            context.inputs[0].version.role = CssomInputRole::Document;
            b.update_context(context).map(drop)
        }),
        ("update the context linking a snapshot", |b, _| {
            b.update_context(linked_donor_context()).map(drop)
        }),
        ("remove a sheet", |b, h| {
            b.remove_sheet(&h.sheet_a).map(drop)
        }),
        ("replace the text of a sheet", |b, h| {
            b.replace_sheet_text(
                &h.sheet_b,
                "@media screen { .r { color: red } } .s { margin: 1px 0 }
@keyframes k { from { color: red } to { color: blue } }",
            )
        }),
    ]
}

// ---------------------------------------------------------------------------
// Running a scenario.

/// What one run of a scenario left, rendered so that runs compare.
#[derive(Debug, Eq, PartialEq)]
struct Transcript {
    lines: Vec<String>,
}

/// Runs `chosen` operations in order, each in a batch of its own that commits
/// when the operation succeeds. `tighten` gives the limits operation `index`
/// runs under, and `compare` says whether the states are compared with the
/// reference as they arise.
fn run_scenario(
    names: &[usize],
    reference_census: bool,
    tighten: Option<(usize, &[(Field, u64)])>,
    compare: bool,
) -> (Transcript, Vec<Vec<(Field, u64)>>) {
    let table = operations();
    probe::set_whole_state(reference_census);
    probe::set_reference(reference_census);
    let mut store = fixture();
    let mut lines = Vec::new();
    let mut thresholds_after = Vec::new();
    for (position, index) in names.iter().enumerate() {
        let (name, operation) = table[*index];
        let handles = {
            let order = &store.state.sheet_order;
            match (order.first(), order.get(1)) {
                (Some(a), Some(b)) => Handles {
                    sheet_a: a.clone(),
                    sheet_b: b.clone(),
                },
                (Some(a), None) => Handles {
                    sheet_a: a.clone(),
                    sheet_b: a.clone(),
                },
                _ => {
                    // A removed sheet leaves nothing for the edits to address.
                    let sheet = serial_sorted(store.state.sheets.iter(), |id| id.serial)
                        .first()
                        .map(|(id, _)| (*id).clone());
                    match sheet {
                        Some(sheet) => Handles {
                            sheet_a: sheet.clone(),
                            sheet_b: sheet,
                        },
                        None => {
                            lines.push(format!("{name}: no sheet"));
                            thresholds_after.push(Vec::new());
                            continue;
                        }
                    }
                }
            }
        };
        let original = store.limits.clone();
        if let Some((target, fields)) = tighten
            && target == position
        {
            let mut tight = original.clone();
            for (field, value) in fields {
                field.set(&mut tight, *value);
            }
            store.limits = tight;
        }
        let mut batch = store
            .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
            .unwrap();
        let result = operation(&mut batch, &handles);
        let issued = batch
            .staged
            .identity_issuer
            .load(std::sync::atomic::Ordering::Relaxed);
        lines.push(format!("{position} {name}: {result:?} issued {issued}"));
        lines.extend(fingerprint(&batch.staged));
        if compare && !reference_census {
            let displaced = displaced_of(&batch);
            compare_state(&batch.staged, &displaced, &format!("{name}, staged"));
            thresholds_after.push(thresholds(&batch.staged, &displaced));
        }
        if result.is_ok() {
            let committed = store.commit(batch);
            lines.push(format!("{position} commit: {:?}", committed.as_ref().err()));
            drop(committed);
            lines.extend(fingerprint(&store.state));
            if compare && !reference_census {
                compare_state(&store.state, &[], &format!("{name}, published"));
            }
        }
        store.limits = original;
        tally(|tally| tally.operations += 1);
    }
    probe::set_whole_state(false);
    probe::set_reference(false);
    (Transcript { lines }, thresholds_after)
}

/// Runs the scenario with the census the crate runs and with the reference,
/// and with the limits around each operation's thresholds, and compares them.
fn check_scenario(label: &str, chosen: &[usize]) {
    tally(|tally| tally.scenarios += 1);
    let (crate_run, thresholds) = run_scenario(chosen, false, None, true);
    let (reference_run, _) = run_scenario(chosen, true, None, false);
    assert_eq!(crate_run, reference_run, "{label}: default limits");
    let table = operations();
    for (position, index) in chosen.iter().enumerate() {
        let name = table[*index].0;
        let line = crate_run
            .lines
            .iter()
            .find(|line| line.starts_with(&format!("{position} {name}: ")))
            .expect("an operation is recorded");
        assert!(
            line.contains(": Ok("),
            "{label}: {name} does not succeed: {line}"
        );
    }
    for (position, fields) in thresholds.iter().enumerate() {
        for (index, (field, value)) in fields.iter().enumerate() {
            for value in [value.saturating_sub(1), *value] {
                let tight = [(*field, value)];
                let ours = run_scenario(chosen, false, Some((position, &tight)), false).0;
                let theirs = run_scenario(chosen, true, Some((position, &tight)), false).0;
                assert_eq!(
                    ours, theirs,
                    "{label}: operation {position}, limit {field:?} = {value} ({index})"
                );
                tally(|tally| tally.comparisons += 1);
            }
        }
    }
}

fn index_of(name: &str) -> usize {
    operations()
        .iter()
        .position(|(candidate, _)| *candidate == name)
        .unwrap_or_else(|| panic!("no edit named {name}"))
}

/// Edits that leave the sheets of a fresh fixture in a state the next ones address.
fn prelude(names: &[&str]) -> Vec<usize> {
    names.iter().map(|name| index_of(name)).collect()
}

#[test]
fn every_class_of_edit_leaves_a_state_the_reference_census_agrees_with() {
    let classes: &[&[&str]] = &[
        &["disable a sheet", "enable a sheet"],
        &[
            "owner attributes with a title and media",
            "owner attributes clearing the title",
        ],
        &[
            "set the media of a sheet",
            "set the media of a sheet from exact UTF-16",
        ],
        &["displace exact media", "set the media of a sheet"],
        &["append and delete a medium"],
        &[
            "set the media of a group rule",
            "replace the media list of a group rule",
        ],
        &["set a style selector", "set a page selector"],
        &["set a keyframe key", "set a keyframes name"],
        &["set a counter style name", "set a counter style descriptor"],
        &["set the families of a font feature values rule"],
        &["set and delete a font feature", "clear a feature map"],
        &[
            "insert into a named supports rule",
            "delete from a named supports rule",
        ],
        &["associate an import"],
        &["insert one exact rule text into two sheets"],
        &[
            "insert a rule into a sheet",
            "insert a rule into a sheet from exact UTF-16",
            "insert a rule into a group",
            "delete a rule of a sheet",
        ],
        &["set a declaration", "remove a declaration"],
        &[
            "create inline declarations",
            "update an inline declaration context",
            "retire inline declarations",
        ],
        &[
            "create and edit inline declarations",
            "create and retire inline declarations",
        ],
        &[
            "update the context without a linked snapshot",
            "update the context linking a snapshot",
        ],
        &["replace the text of a sheet", "remove a sheet"],
    ];
    for names in classes {
        check_scenario(&names.join(", "), &prelude(names));
    }
    let tally = TALLY.with(Cell::get);
    eprintln!("edit classes: {tally:?}");
    assert!(
        tally.displaced > 0,
        "a state with displaced media: {tally:?}"
    );
    assert!(tally.refusals > 0, "a refusal: {tally:?}");
}

/// A deterministic generator: a 64-bit xorshift.
struct Seed(u64);
impl Seed {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(bound).unwrap_or(1)).unwrap_or(0)
    }
}

#[test]
fn seeded_sequences_of_edits_leave_states_the_reference_census_agrees_with() {
    let count = operations().len();
    for seed in [
        0x9e37_79b9_7f4a_7c15u64,
        0xd1b5_4a32_d192_ed03,
        0x2545_f491_4f6c_dd1d,
    ] {
        let mut generator = Seed(seed);
        let chosen = (0..8).map(|_| generator.below(count)).collect::<Vec<_>>();
        let (crate_run, _) = run_scenario(&chosen, false, None, true);
        let (reference_run, _) = run_scenario(&chosen, true, None, false);
        assert_eq!(crate_run, reference_run, "seed {seed:#x}: {chosen:?}");
        tally(|tally| tally.scenarios += 1);
    }
    let tally = TALLY.with(Cell::get);
    eprintln!("seeded sequences: {tally:?}");
    assert!(tally.states > 20, "{tally:?}");
}

#[test]
fn the_fixture_retains_three_distinct_source_snapshots_and_names_each_edit_once() {
    let store = fixture();
    let (a, b) = (&store.state.sheet_order[0], &store.state.sheet_order[1]);
    let text_of = |id: &CssomSheetId| {
        store
            .state
            .sheet(id)
            .unwrap()
            .authored
            .parse_source()
            .map(|source| source.source().clone())
            .unwrap()
    };
    let inserted = store
        .state
        .rules
        .values()
        .find_map(|rule| {
            rule.insertion_origin()
                .map(|origin| origin.source().clone())
        })
        .expect("the inserted rule keeps the snapshot it was parsed from");
    let snapshots = [text_of(a), text_of(b), inserted];
    for (index, first) in snapshots.iter().enumerate() {
        for second in &snapshots[index + 1..] {
            assert!(!first.same_snapshot(second));
        }
    }
    let mut names = std::collections::HashSet::new();
    assert!(operations().iter().all(|(name, _)| names.insert(*name)));
}
