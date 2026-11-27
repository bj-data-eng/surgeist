//! Every settle compared with the frozen reference census of the whole state.
//!
//! A settle answers from the figures the state keeps; the reference accounts
//! for every record of the state from nothing. Under test each settle runs
//! both, so every test that edits a store is also a differential case.
//!
//! The two answers are equal except where the settle's fixed order replaces an
//! order the reference takes from the iteration of hash tables, which are the
//! decisions the settle documents:
//!
//! - when several retained source limits are exceeded, the reference reports the
//!   one its walk reaches first, and the settle the first of the traversal depth, the
//!   retained bytes, the traversal steps and the storage;
//! - when the depth and the visits of the linked inputs are both exceeded, the
//!   settle reports the depth;
//! - the traversal depth of a state several linked inputs reach is the greatest
//!   depth of the paths to it, where the reference takes the depth of the path its
//!   walk takes first, so the settle can refuse a state the reference admits.
//!
//! Each difference is checked against the reference itself: [`exceeded`] asks it
//! which limits the state exceeds, with every other limit lifted.
use super::{BYTES_RESOURCE, DEPTH_RESOURCE, STEPS_RESOURCE, STORAGE_RESOURCE, probe, reference};
use crate::{model::State, *};
use std::collections::HashMap;
use std::sync::Arc;
use surgeist_css::query::media::CssMediaQueryList;

/// What a census of a state accounts for: the figures a settle keeps, in the form the reference reports them.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WholeFigures {
    pub(crate) entries: Option<usize>,
    pub(crate) strings: Option<usize>,
    pub(crate) objects: usize,
    pub(crate) retained_bytes: usize,
    pub(crate) steps: usize,
}

impl State {
    /// The figures of the state, with the entries of the `displaced` media lists.
    pub(crate) fn whole_figures(&self, displaced: &[&CssMediaQueryList]) -> WholeFigures {
        let extra = displaced
            .iter()
            .map(|media| u128::try_from(media.queries().len()).unwrap_or(u128::MAX))
            .sum::<u128>();
        let entries = self
            .figures
            .entries()
            .saturating_add(u128::try_from(self.sheet_order.len()).unwrap_or(u128::MAX))
            .saturating_add(extra);
        WholeFigures {
            entries: usize::try_from(entries).ok(),
            strings: usize::try_from(self.figures.strings()).ok(),
            objects: self.objects().unwrap_or(usize::MAX),
            retained_bytes: self.figures.retained_bytes(),
            steps: usize::try_from(self.figures.walk_steps()).unwrap_or(usize::MAX),
        }
    }

    /// The size of the state as the reference census of the whole state counts it.
    pub(crate) fn whole_retained(&self) -> crate::operation_work::Retained {
        let census = reference::of(self, &CssomLimits::default(), std::iter::empty())
            .result
            .expect("the reference admits the state");
        let figures = census.figures();
        crate::operation_work::Retained::new(
            figures.objects,
            figures.entries.unwrap_or(usize::MAX),
            figures.strings.unwrap_or(usize::MAX),
        )
    }

    /// The answer of a settle, once it has been compared with the reference census of
    /// the whole state, or the reference's own answer when a test asks for it.
    pub(super) fn verified(
        &self,
        limits: &CssomLimits,
        displaced: &[&CssMediaQueryList],
        outcome: Result<(), CssomError>,
    ) -> Result<(), CssomError> {
        if probe::reference() {
            return reference::of(self, limits, displaced.iter().copied())
                .result
                .map(drop);
        }
        if !probe::cross_check() {
            return outcome;
        }
        let whole = reference::of(self, limits, displaced.iter().copied()).result;
        match (&outcome, whole) {
            (Ok(()), Ok(census)) => {
                let mut kept = self.whole_figures(displaced);
                let reference = census.figures();
                if !displaced.is_empty() {
                    // The displaced lists leave the ledger and the walk again.
                    kept.retained_bytes = reference.retained_bytes;
                    kept.steps = reference.steps;
                }
                assert_eq!(kept, reference, "the figures of a settled state");
            }
            (Err(kept), Err(reference)) => {
                differences_are_decisions(self, limits, displaced, kept, &reference);
            }
            (Err(kept), Ok(_)) => assert!(
                is_depth(kept) && shares_a_linked_state(self),
                "a settle refused {kept:?} where the reference admits the state"
            ),
            (Ok(()), Err(reference)) => {
                panic!("a settle admitted a state the reference refuses with {reference:?}")
            }
        }
        outcome
    }
}

fn is_depth(error: &CssomError) -> bool {
    matches!(
        error,
        CssomError::Limit {
            resource: DEPTH_RESOURCE,
            ..
        }
    )
}

fn resource(error: &CssomError) -> Option<&'static str> {
    match error {
        CssomError::Limit { resource, .. } => Some(*resource),
        _ => None,
    }
}

const SOURCE: [&str; 4] = [
    DEPTH_RESOURCE,
    BYTES_RESOURCE,
    STEPS_RESOURCE,
    STORAGE_RESOURCE,
];
const LINKED: [&str; 2] = ["linked input depth", "linked input visits"];
/// The conditions of a single record. Which of several records that fail one is
/// refused first is the order of the records touched, where the reference takes the
/// order of its hash tables.
const PER_RECORD: [&str; 5] = [
    "parse input bytes",
    "retained selector input bytes",
    "retained test input bytes",
    "test payload entries",
    "test payload capacity",
];

/// Whether `error` is the refusal of a single record.
fn per_record(error: &CssomError) -> bool {
    matches!(
        error,
        CssomError::InvalidInput("unsupported named-supports test child")
    ) || resource(error).is_some_and(|name| PER_RECORD.contains(&name))
}

/// Whether `state` links a state through two inputs, which a walk reaches by two paths.
fn shares_a_linked_state(state: &State) -> bool {
    fn reach(state: &State, seen: &mut HashMap<usize, usize>) {
        let contexts = std::iter::once(state.context.as_ref())
            .chain(
                state
                    .rules
                    .values()
                    .filter_map(|rule| rule.selector_inputs.as_deref()),
            )
            .chain(state.inline_contexts());
        for linked in contexts.flat_map(|context| &context.linked) {
            let key = Arc::as_ptr(&linked.snapshot.state).addr();
            let count = seen.entry(key).or_insert(0);
            *count += 1;
            if *count == 1 {
                reach(&linked.snapshot.state, seen);
            }
        }
    }
    let mut seen = HashMap::new();
    reach(state, &mut seen);
    seen.values().any(|count| *count > 1)
}

/// Whether `limits`, with the limits `lift` removes, make the reference refuse `state` with `resource`.
fn exceeded(
    state: &State,
    limits: &CssomLimits,
    displaced: &[&CssMediaQueryList],
    lift: impl Fn(&mut CssomLimits),
    resource: &str,
) -> bool {
    let mut lifted = limits.clone();
    lift(&mut lifted);
    matches!(
        reference::of(state, &lifted, displaced.iter().copied()).result,
        Err(CssomError::Limit { resource: found, .. }) if found == resource
    )
}

/// A change that lifts the limits of every source resource but one.
type Lift = fn(&mut CssomLimits);

/// Asserts that two refusals are equal, or differ only by a decision of the settle.
fn differences_are_decisions(
    state: &State,
    limits: &CssomLimits,
    displaced: &[&CssMediaQueryList],
    kept: &CssomError,
    reference: &CssomError,
) {
    if format!("{kept:?}") == format!("{reference:?}") {
        return;
    }
    if per_record(kept) && per_record(reference) {
        return;
    }
    let (Some(kept_resource), Some(reference_resource)) = (resource(kept), resource(reference))
    else {
        panic!("a settle refused {kept:?} where the reference refuses {reference:?}");
    };
    let same_family =
        |family: &[&str]| family.contains(&kept_resource) && family.contains(&reference_resource);
    if same_family(&LINKED) {
        let depth = exceeded(
            state,
            limits,
            displaced,
            |lifted| lifted.max_entries = usize::MAX,
            LINKED[0],
        );
        let expected = if depth { LINKED[0] } else { LINKED[1] };
        assert_eq!(
            kept_resource, expected,
            "the linked limits the reference refuses with {reference:?}"
        );
    } else if same_family(&SOURCE) {
        let lifts: [(&str, Lift); 3] = [
            (DEPTH_RESOURCE, |limits| {
                limits.max_retained_source_bytes = usize::MAX;
                limits.max_source_traversal_steps = usize::MAX;
            }),
            (BYTES_RESOURCE, |limits| {
                limits.max_source_traversal_depth = usize::MAX;
                limits.max_source_traversal_steps = usize::MAX;
            }),
            (STEPS_RESOURCE, |limits| {
                limits.max_source_traversal_depth = usize::MAX;
                limits.max_retained_source_bytes = usize::MAX;
            }),
        ];
        let expected = lifts
            .into_iter()
            .find(|(name, lift)| exceeded(state, limits, displaced, lift, name))
            .map_or(reference_resource, |(name, _)| name);
        // A state several linked inputs reach can be deeper than the reference's walk found it.
        assert!(
            kept_resource == expected
                || (kept_resource == DEPTH_RESOURCE && shares_a_linked_state(state)),
            "the source limits the reference refuses with {reference:?}: settled {kept:?}, expected {expected}"
        );
    } else {
        panic!("a settle refused {kept:?} where the reference refuses {reference:?}");
    }
}

impl State {
    /// A copy of the state whose figures account for nothing and whose tables journal every
    /// record, in the order of their identities, so that settling it measures the whole state.
    pub(crate) fn unsettled_copy(&self) -> Self {
        let mut fresh = self.clone();
        fresh.figures = crate::figures::Figures::new();
        fresh.sheets.journal_all(|id| id.serial);
        fresh.lists.journal_all(|id| id.serial);
        fresh.media.journal_all(|id| id.serial);
        fresh.blocks.journal_all(|id| id.serial);
        fresh.maps.journal_all(|id| id.serial);
        fresh.rules.journal_all(|id| id.serial);
        fresh
    }
}

/// Whether the rendering of a refusal names a limit whose choice among several that are
/// exceeded is a decision of the settle.
pub(super) fn is_decision(rendering: &str) -> bool {
    SOURCE
        .iter()
        .chain(&LINKED)
        .chain(&PER_RECORD)
        .any(|name| rendering.contains(name))
        || rendering.contains("unsupported named-supports test child")
}
