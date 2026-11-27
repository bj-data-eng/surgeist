//! The figures of a retained state, kept with the state.
//!
//! A state accounts for what it holds: its objects, entries, host string
//! bytes, the source snapshots its payloads retain, the steps of the walk that
//! reaches them and the linked inputs of its contexts. The figures are the sum of
//! what each record contributes, so a record that changes replaces its
//! contribution: [`Figures::apply`] takes a contribution from the sums, or adds
//! it, and a settle ([`crate::state_limits`]) replaces the contribution of every
//! record whose table journaled it, before testing the thresholds.
//!
//! The figures are exact for a state whose table journals are empty: they equal
//! what accounting for every record of the state, from nothing, gives. Nothing
//! in them depends on the limits, which is why a published state, whose settle
//! the limits of its store admitted, can serve as a linked state of any other
//! store.
//!
//! Linked states are immutable, so a state's figures stand in for them. A
//! state keeps a reference count of the linked states its contexts reach, and
//! the ledger of retained snapshots takes the ledger of each state a context
//! links; both change only when the set of linked inputs does, at a cost in
//! proportion to the closure of the state that comes or goes.
use crate::model::State;
use crate::source_retention::{Sign, Walked};
use crate::state_limits::measure::{self, Base, Counts};
use crate::*;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::sync::Arc;
use surgeist_css::source::CssSourceLedger;

type Table<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;

/// A base URL counted once, however many sheets share it.
#[derive(Clone, Debug)]
struct Shared {
    references: usize,
    bytes: u128,
    /// Keeps the address that identifies the base URL its own while it is counted.
    _units: Arc<Vec<u16>>,
}

/// A state a context of this state reaches, directly or through the states it links.
#[derive(Clone)]
struct Reached {
    state: Arc<State>,
    /// The linked inputs of this state whose closure holds it.
    references: usize,
}

/// What the linked inputs of a context add up to.
#[derive(Clone, Debug, Default)]
pub(crate) struct Closure {
    /// The linked states, with the steps of the walk that reaches them.
    pub(crate) roots: Vec<Arc<State>>,
    /// The linked inputs: one step of the walk's queue each.
    pub(crate) edges: usize,
    /// The linked states a depth-first walk of the context visits, a state once for each path to it.
    pub(crate) visits: u128,
    /// The deepest chain of linked states the context starts.
    pub(crate) depth: usize,
    /// The deepest source nesting a linked state's walk reaches below the context.
    pub(crate) reach: usize,
}

impl Closure {
    pub(crate) fn of(context: &CssomContext) -> Self {
        let mut closure = Self::default();
        for linked in &context.linked {
            let state = &linked.snapshot.state;
            closure.edges = closure.edges.saturating_add(1);
            closure.visits = closure
                .visits
                .saturating_add(1)
                .saturating_add(state.figures.link_visits);
            closure.depth = closure
                .depth
                .max(state.figures.link_depth().saturating_add(1));
            closure.reach = closure
                .reach
                .max(state.figures.source_depth().saturating_add(1));
            closure.roots.push(state.clone());
        }
        closure
    }
}

/// What one record, or one context, contributes to the figures of its state.
#[derive(Clone, Debug, Default)]
pub(crate) struct Contribution {
    pub(crate) counts: Counts,
    /// The steps and nesting of the record's own walk.
    pub(crate) walked: Walked,
    /// The context the walk of the record enqueues, with the states it links.
    pub(crate) links: Closure,
    /// Whether that context is one whose linked inputs the state validates: the
    /// context of the state and of its inline declaration blocks.
    pub(crate) validated: bool,
}

impl Contribution {
    /// The source nesting below the record, linked states included.
    fn reach(&self) -> usize {
        self.walked.reach.max(self.links.reach)
    }
}

/// Why a contribution could not be applied.
#[derive(Debug)]
pub(crate) enum Refused {
    /// The figures cannot count one more base URL.
    Base(CssomError),
    /// The ledger of retained snapshots cannot hold what a linked state retains.
    Source(surgeist_css::source::CssSourceRetentionError),
}

/// The figures of one state; see the [module](self).
#[derive(Clone)]
pub(crate) struct Figures {
    /// Set when a settle refused: the figures are then unusable.
    poisoned: bool,
    entries: u128,
    strings: u128,
    bases: Arc<Table<usize, Shared>>,
    base_strings: u128,
    /// The steps of the walk charged by the records and the context of the state.
    steps: u128,
    /// The linked inputs of the contexts the walk enqueues.
    edges: u128,
    /// The count of records, by the source nesting below them.
    reaches: BTreeMap<usize, usize>,
    /// The distinct snapshots the state and the states it links retain.
    ledger: Arc<CssSourceLedger>,
    /// The linked states reached, by address.
    reached: Arc<Table<usize, Reached>>,
    /// The steps of the walks of the reached states.
    reached_steps: u128,
    /// The visits of a depth-first walk of the linked inputs of the state's context
    /// and of its inline declaration contexts.
    pub(crate) link_visits: u128,
    /// The count of those contexts, by the depth of the chain of linked states they start.
    link_depths: BTreeMap<usize, usize>,
    /// The context of the state that the figures account for.
    context: Option<Arc<CssomContext>>,
}

impl fmt::Debug for Figures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Figures")
            .field("poisoned", &self.poisoned)
            .field("entries", &self.entries)
            .field("strings", &(self.strings + self.base_strings))
            .finish_non_exhaustive()
    }
}

/// The charges of the walk of a state that do not belong to a record: the initial
/// step, and the pop and the identity check of the state itself.
const WALK_FIXED_STEPS: u128 = 3;

impl Figures {
    /// The figures of a state without records, which a first settle accounts for.
    pub(crate) fn new() -> Self {
        Self {
            poisoned: false,
            entries: 0,
            strings: 0,
            bases: Arc::new(Table::default()),
            base_strings: 0,
            steps: 0,
            edges: 0,
            reaches: BTreeMap::new(),
            ledger: Arc::new(CssSourceLedger::new()),
            reached: Arc::new(Table::default()),
            reached_steps: 0,
            link_visits: 0,
            link_depths: BTreeMap::new(),
            context: None,
        }
    }

    pub(crate) const fn is_poisoned(&self) -> bool {
        self.poisoned
    }
    pub(crate) fn poison(&mut self) {
        self.poisoned = true;
    }
    pub(crate) fn heal(&mut self) {
        self.poisoned = false;
    }

    /// The context the figures account for.
    pub(crate) fn context(&self) -> Option<&Arc<CssomContext>> {
        self.context.as_ref()
    }
    pub(crate) fn set_context(&mut self, context: Arc<CssomContext>) {
        self.context = Some(context);
    }
    pub(crate) fn take_context(&mut self) -> Option<Arc<CssomContext>> {
        self.context.take()
    }

    /// The deepest source nesting any record of the state reaches, linked states included.
    pub(crate) fn source_depth(&self) -> usize {
        self.reaches.last_key_value().map_or(0, |(depth, _)| *depth)
    }
    /// The depth of the deepest chain of linked states the contexts of the state start.
    pub(crate) fn link_depth(&self) -> usize {
        self.link_depths
            .last_key_value()
            .map_or(0, |(depth, _)| *depth)
    }

    /// The steps of the walk of this state alone: its records, its context and
    /// the queue entry of each linked input.
    fn own_weight(&self) -> u128 {
        self.steps.saturating_add(self.edges.saturating_mul(2))
    }
    /// The steps of the walk of the state and every state it reaches.
    pub(crate) fn walk_steps(&self) -> u128 {
        WALK_FIXED_STEPS
            .saturating_add(self.own_weight())
            .saturating_add(self.reached_steps)
    }
    pub(crate) fn entries(&self) -> u128 {
        self.entries
    }
    pub(crate) fn strings(&self) -> u128 {
        self.strings.saturating_add(self.base_strings)
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.ledger.bytes()
    }
    pub(crate) fn ledger_mut(&mut self) -> &mut CssSourceLedger {
        Arc::make_mut(&mut self.ledger)
    }
    pub(crate) fn ledger(&self) -> &CssSourceLedger {
        &self.ledger
    }

    fn sum(total: &mut u128, sign: Sign, value: u128) {
        *total = match sign {
            Sign::Add => total.saturating_add(value),
            Sign::Remove => total.saturating_sub(value),
        };
    }
    fn count<K: Ord + Copy>(counts: &mut BTreeMap<K, usize>, sign: Sign, key: K) {
        match sign {
            Sign::Add => {
                let count = counts.entry(key).or_insert(0);
                *count = count.saturating_add(1);
            }
            Sign::Remove => {
                if let Some(count) = counts.get_mut(&key) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        counts.remove(&key);
                    }
                }
            }
        }
    }

    /// Adds `contribution` to the figures, or takes it out.
    ///
    /// The change to the ledger of retained snapshots the contribution's walk made is
    /// not applied here: the linked states the contribution reaches add their own
    /// snapshots to `delta`.
    ///
    /// # Errors
    ///
    /// The refusal to count one more base URL, which the storage of the figures
    /// cannot hold.
    pub(crate) fn apply(
        &mut self,
        contribution: &Contribution,
        sign: Sign,
        delta: &mut surgeist_css::source::CssSourceDelta,
    ) -> Result<(), Refused> {
        Self::sum(&mut self.entries, sign, widen(contribution.counts.entries));
        Self::sum(&mut self.strings, sign, widen(contribution.counts.strings));
        Self::sum(&mut self.steps, sign, widen(contribution.walked.steps));
        Self::sum(&mut self.edges, sign, widen(contribution.links.edges));
        let reach = contribution.reach();
        if reach > 0 {
            Self::count(&mut self.reaches, sign, reach);
        }
        for root in &contribution.links.roots {
            self.reach_root(root, sign, delta)?;
        }
        if contribution.validated {
            Self::sum(&mut self.link_visits, sign, contribution.links.visits);
            if contribution.links.depth > 0 {
                Self::count(&mut self.link_depths, sign, contribution.links.depth);
            }
        }
        if let Some(base) = &contribution.counts.base {
            self.share_base(base, sign).map_err(Refused::Base)?;
        }
        Ok(())
    }

    /// Counts the base URL of a sheet once, or stops counting it with the last sheet.
    fn share_base(&mut self, base: &Base, sign: Sign) -> Result<(), CssomError> {
        match sign {
            Sign::Add => {
                if let Some(shared) = Arc::make_mut(&mut self.bases).get_mut(&base.identity) {
                    shared.references = shared.references.saturating_add(1);
                    return Ok(());
                }
                let bases = Arc::make_mut(&mut self.bases);
                bases.try_reserve(1).map_err(|_| CssomError::Limit {
                    resource: "base URL source identities",
                    maximum: usize::MAX,
                })?;
                bases.insert(
                    base.identity,
                    Shared {
                        references: 1,
                        bytes: base.bytes,
                        _units: base.units.clone(),
                    },
                );
                Self::sum(&mut self.base_strings, Sign::Add, base.bytes);
            }
            Sign::Remove => {
                let bases = Arc::make_mut(&mut self.bases);
                if let Some(shared) = bases.get_mut(&base.identity) {
                    shared.references = shared.references.saturating_sub(1);
                    if shared.references == 0 {
                        let bytes = shared.bytes;
                        bases.remove(&base.identity);
                        Self::sum(&mut self.base_strings, Sign::Remove, bytes);
                    }
                }
            }
        }
        Ok(())
    }

    /// Counts a linked state as reached by one more context, or one fewer.
    ///
    /// The state and every state it reaches join the states the walk of this
    /// state reaches; a state joins, with the steps of its own walk, when no other
    /// context reaches it, and leaves when the last context that did is gone. The
    /// snapshots the linked state retains, those of its closure included, go to
    /// or come from `delta`.
    fn reach_root(
        &mut self,
        root: &Arc<State>,
        sign: Sign,
        delta: &mut surgeist_css::source::CssSourceDelta,
    ) -> Result<(), Refused> {
        #[cfg(test)]
        crate::state_limits::probe::root_reached();
        let reached = Arc::make_mut(&mut self.reached);
        let closure = std::iter::once(root).chain(root.figures.reached.values().map(|r| &r.state));
        for state in closure {
            let key = Arc::as_ptr(state).addr();
            match sign {
                Sign::Add => {
                    let entry = reached.entry(key).or_insert_with(|| Reached {
                        state: state.clone(),
                        references: 0,
                    });
                    entry.references = entry.references.saturating_add(1);
                    if entry.references == 1 {
                        Self::sum(
                            &mut self.reached_steps,
                            Sign::Add,
                            state.figures.own_weight(),
                        );
                    }
                }
                Sign::Remove => {
                    if let Some(entry) = reached.get_mut(&key) {
                        entry.references = entry.references.saturating_sub(1);
                        if entry.references == 0 {
                            reached.remove(&key);
                            Self::sum(
                                &mut self.reached_steps,
                                Sign::Remove,
                                state.figures.own_weight(),
                            );
                        }
                    }
                }
            }
        }
        match sign {
            Sign::Add => delta.add_ledger(root.figures.ledger()),
            Sign::Remove => delta.remove_ledger(root.figures.ledger()),
        }
        .map_err(Refused::Source)
    }

    /// The size of the state in the units of the host's gauges.
    pub(crate) fn retained(
        &self,
        objects: usize,
        sheet_order: usize,
    ) -> crate::operation_work::Retained {
        let entries = self.entries.saturating_add(widen(sheet_order));
        crate::operation_work::Retained::new(
            objects,
            usize::try_from(entries).unwrap_or(usize::MAX),
            usize::try_from(self.strings()).unwrap_or(usize::MAX),
        )
    }
}

/// A count as the wide integers the figures sum in.
pub(crate) fn widen(count: usize) -> u128 {
    u128::try_from(count).unwrap_or(u128::MAX)
}

/// The contribution of one context: its inputs and linked inputs, and the
/// refusal when a linked sheet is missing from the snapshot it names.
///
/// The walk charges the context itself and each input it links.
pub(crate) fn contribution_of_context(
    context: &CssomContext,
    validated: bool,
) -> (Contribution, Option<CssomError>) {
    let (counts, refusal) = match measure::context(context) {
        Ok(counts) => (counts, None),
        Err(error) => (Counts::default(), Some(error)),
    };
    (
        Contribution {
            counts,
            walked: Walked {
                steps: context.linked.len().saturating_add(1),
                reach: 0,
            },
            links: Closure::of(context),
            validated,
        },
        refusal,
    )
}
