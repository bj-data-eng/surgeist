//! Synchronous admission accounting for the retained CSSOM state.
//!
//! The state keeps its figures ([`crate::figures`]): the sums of what its
//! records contribute. A check of the state, a *settle*, replaces the
//! contribution of every record its tables journaled since the last one by the
//! contribution of the record the key holds now, and then tests the thresholds in
//! a fixed order: the contexts, the object count, the conditions of each record
//! touched, the entries, the retained sources and the string bytes. What a settle
//! measures is what changed, so its work does not depend on how many records the
//! state holds.
//!
//! The retained sources are refused in a fixed order when several limits are
//! exceeded at once: the traversal depth, the retained bytes, the traversal
//! steps, the traversal storage. The step limit bounds the size of the state
//! as the figure of the steps of the walk that reaches its sources, and the
//! measurement of one payload part. The traversal depth of a state that several
//! linked inputs reach is the greatest depth of the paths that reach it.
use crate::contribute::{self, Before, Measured};
use crate::figures::{Contribution, Figures, Refused, contribution_of_context, widen};
use crate::model::State;
use crate::shared_table::SharedTable;
use crate::source_retention::{
    BYTES_RESOURCE, DEPTH_RESOURCE, Measurer, STEPS_RESOURCE, STORAGE_RESOURCE, Sign,
};
use crate::*;
use measure::Counts;
use std::sync::Arc;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::source::{CssSourceRetentionError, CssSourceRetentionResource};

pub(crate) mod measure;

/// The limits of the retained sources a measurement was refused by.
#[derive(Clone, Copy, Debug, Default)]
struct Refusals {
    depth: bool,
    bytes: bool,
    steps: bool,
    storage: bool,
}

impl Refusals {
    /// Notes the refusal of a measurement or of the ledger.
    fn note(&mut self, error: CssSourceRetentionError) {
        match error.exhausted_limit() {
            Some(CssSourceRetentionResource::RetainedBytes) => self.bytes = true,
            Some(CssSourceRetentionResource::Depth) => self.depth = true,
            Some(CssSourceRetentionResource::Storage) | None => self.storage = true,
            Some(_) => self.steps = true,
        }
    }
}

/// The media lists a batch has displaced: their entries, the steps of their
/// walk and the nesting below them, which count while the batch checks itself.
#[derive(Clone, Copy, Debug, Default)]
struct Extra {
    entries: u128,
    steps: u128,
    reach: usize,
}

/// The work of one settle.
#[derive(Clone, Copy, Debug, Default)]
struct Work {
    /// Records whose contribution was measured.
    examined: usize,
    /// Entry lists their measurement examined.
    entries: usize,
}

/// What a settle collected about the records it measured, to test them after
/// every contribution has been replaced.
#[derive(Default)]
struct Pending<'a> {
    /// The contexts to validate: the state's, then each inline declaration context.
    validate: Vec<&'a CssomContext>,
    /// The refusal of a context that links a sheet its snapshot lacks: the
    /// state's, a rule's, a block's.
    contexts: [Option<CssomError>; 3],
    /// Each sheet's counts, or the refusal to count its base URL.
    sheets: Vec<Result<Counts, CssomError>>,
    rules: Vec<Counts>,
}

/// The measurement of the records a settle touches.
struct Settle<'a> {
    figures: &'a mut Figures,
    measurer: Measurer,
    refusals: Refusals,
    work: Work,
}

const STATE_CONTEXT: usize = 0;
const RULE_CONTEXT: usize = 1;
const BLOCK_CONTEXT: usize = 2;

impl Settle<'_> {
    /// Adds `contribution` to the figures or takes it out.
    fn apply(&mut self, contribution: &Contribution, sign: Sign) -> Option<CssomError> {
        match self
            .figures
            .apply(contribution, sign, self.measurer.delta())
        {
            Ok(()) => None,
            Err(Refused::Base(error)) => Some(error),
            Err(Refused::Source(error)) => {
                self.refusals.note(error);
                None
            }
        }
    }

    /// Takes out what a record contributed when it was last settled.
    fn take_out(&mut self, before: &Before) {
        if let Some(error) = before.walk {
            // What the record retained could not be counted, so the figures are short of it.
            self.refusals.note(error);
        }
        self.apply(&before.contribution, Sign::Remove);
        if let Some(removal) = &before.removal
            && let Err(error) = self.measurer.delta().merge(removal)
        {
            self.refusals.note(error);
        }
    }

    /// Replaces what a record contributed, `before`, by what it contributes now,
    /// `measured`, and gives back what the new contribution is tested by.
    ///
    /// The states a context links join the figures when the context joins them, and leave
    /// when it does: a record that holds the same context before and after leaves them be.
    fn replace(
        &mut self,
        before: Option<Before>,
        after: Option<(Measured, Option<&Arc<CssomContext>>)>,
    ) -> Option<(Contribution, Option<CssomError>)> {
        let mut before = before;
        let mut after = after;
        if let (Some(before), Some((measured, context))) = (before.as_mut(), after.as_mut())
            && let (Some(held), Some(now)) = (&before.context, context)
            && Arc::ptr_eq(held, now)
        {
            before.contribution.links.roots.clear();
            measured.contribution.links.roots.clear();
        }
        if let Some(before) = &before {
            self.take_out(before);
        }
        let (measured, _) = after?;
        if let Some(error) = measured.walk {
            self.refusals.note(error);
        }
        self.work.examined = self.work.examined.saturating_add(1);
        self.work.entries = self
            .work
            .entries
            .saturating_add(measured.contribution.counts.work);
        #[cfg(test)]
        probe::examined(1);
        Some((measured.contribution, measured.context))
    }

    /// Replaces what the sheet records the table journaled contributed.
    fn sheets(
        &mut self,
        table: &SharedTable<CssomSheetId, CssomSheet>,
        journal: Vec<(CssomSheetId, Option<Before>)>,
        pending: &mut Pending<'_>,
    ) {
        for (key, before) in journal {
            let after = table.get(&key).map(|sheet| {
                (
                    contribute::sheet(&mut self.measurer, sheet, Sign::Add),
                    None,
                )
            });
            if let Some((contribution, _)) = self.replace(before, after) {
                pending.sheets.push(Ok(contribution.counts.clone()));
                if let Some(error) = self.apply(&contribution, Sign::Add) {
                    pending.sheets.push(Err(error));
                }
            }
        }
    }

    fn lists(
        &mut self,
        table: &SharedTable<CssomRuleListId, Vec<CssomRuleId>>,
        journal: Vec<(CssomRuleListId, Option<Before>)>,
    ) {
        for (key, before) in journal {
            let after = table.get(&key).map(|list| (contribute::list(list), None));
            if let Some((contribution, _)) = self.replace(before, after) {
                self.apply(&contribution, Sign::Add);
            }
        }
    }

    fn media(
        &mut self,
        table: &SharedTable<CssomMediaListId, CssMediaQueryList>,
        journal: Vec<(CssomMediaListId, Option<Before>)>,
    ) {
        for (key, before) in journal {
            let after = table.get(&key).map(|media| {
                (
                    contribute::media(&mut self.measurer, media, Sign::Add),
                    None,
                )
            });
            if let Some((contribution, _)) = self.replace(before, after) {
                self.apply(&contribution, Sign::Add);
            }
        }
    }

    fn maps(
        &mut self,
        table: &SharedTable<CssomFeatureMapId, CssomFeatureMap>,
        journal: Vec<(CssomFeatureMapId, Option<Before>)>,
    ) {
        for (key, before) in journal {
            let after = table
                .get(&key)
                .map(|map| (contribute::map(&mut self.measurer, map, Sign::Add), None));
            if let Some((contribution, _)) = self.replace(before, after) {
                self.apply(&contribution, Sign::Add);
            }
        }
    }

    fn blocks<'t>(
        &mut self,
        table: &'t SharedTable<CssomBlockId, CssomBlock>,
        journal: Vec<(CssomBlockId, Option<Before>)>,
        pending: &mut Pending<'t>,
    ) {
        for (key, before) in journal {
            let block = table.get(&key);
            let after = block.map(|block| {
                (
                    contribute::block(&mut self.measurer, block, Sign::Add),
                    contribute::block_context(block),
                )
            });
            if let Some((contribution, refusal)) = self.replace(before, after) {
                if let Some(context) = block.and_then(CssomBlock::inline_context) {
                    pending.validate.push(context.context());
                }
                if pending.contexts[BLOCK_CONTEXT].is_none() {
                    pending.contexts[BLOCK_CONTEXT] = refusal;
                }
                self.apply(&contribution, Sign::Add);
            }
        }
    }

    fn rules(
        &mut self,
        table: &SharedTable<CssomRuleId, CssomRule>,
        journal: Vec<(CssomRuleId, Option<Before>)>,
        pending: &mut Pending<'_>,
    ) {
        for (key, before) in journal {
            let after = table.get(&key).map(|rule| {
                (
                    contribute::rule(&mut self.measurer, rule, Sign::Add),
                    rule.selector_inputs.as_ref(),
                )
            });
            if let Some((contribution, refusal)) = self.replace(before, after) {
                pending.rules.push(contribution.counts.clone());
                if pending.contexts[RULE_CONTEXT].is_none() {
                    pending.contexts[RULE_CONTEXT] = refusal;
                }
                self.apply(&contribution, Sign::Add);
            }
        }
    }
}

impl State {
    /// The objects of the state: the state itself and every sheet, rule, block, list,
    /// media list and feature map it holds, detached ones included.
    fn objects(&self) -> Option<usize> {
        1usize
            .checked_add(self.sheets.len())
            .and_then(|v| v.checked_add(self.rules.len()))
            .and_then(|v| v.checked_add(self.blocks.len()))
            .and_then(|v| v.checked_add(self.lists.len()))
            .and_then(|v| v.checked_add(self.media.len()))
            .and_then(|v| v.checked_add(self.maps.len()))
    }

    /// Whether the figures account for every record the state holds: no table has
    /// journaled a change since the last settle, and the last settle was not refused.
    pub(crate) fn is_settled(&self) -> bool {
        !self.figures.is_poisoned()
            && self.sheets.is_settled()
            && self.rules.is_settled()
            && self.blocks.is_settled()
            && self.lists.is_settled()
            && self.media.is_settled()
            && self.maps.is_settled()
            && self
                .figures
                .context()
                .is_some_and(|settled| Arc::ptr_eq(settled, &self.context))
    }

    /// The size of the state, as its figures account for it.
    pub(crate) fn retained(&self) -> crate::operation_work::Retained {
        self.figures
            .retained(self.objects().unwrap_or(usize::MAX), self.sheet_order.len())
    }

    /// Accounts for what the tables of the state have journaled since the last
    /// settle, and refuses the state when it exceeds `limits`.
    ///
    /// `displaced` are the media lists the batch holding this state has displaced,
    /// which count for this check alone.
    ///
    /// A refused settle leaves the figures unusable: every later settle of this
    /// state answers [`CssomError::BatchAborted`].
    ///
    /// # Errors
    ///
    /// The refusal of the state: the first threshold it exceeds, in the order of the
    /// module documentation.
    pub(crate) fn settle(
        &mut self,
        limits: &CssomLimits,
        displaced: &[&CssMediaQueryList],
    ) -> Result<(), CssomError> {
        if self.figures.is_poisoned() {
            return Err(CssomError::BatchAborted);
        }
        self.figures.poison();
        let mut work = Work::default();
        let outcome = self.account(limits, displaced, &mut work);
        crate::operation_work::census(work.examined, work.entries);
        #[cfg(test)]
        let outcome = self.verified(limits, displaced, outcome);
        if outcome.is_ok() {
            self.figures.heal();
        }
        outcome
    }

    /// Replaces the contribution of every record touched, and tests the thresholds.
    fn account(
        &mut self,
        limits: &CssomLimits,
        displaced: &[&CssMediaQueryList],
        work: &mut Work,
    ) -> Result<(), CssomError> {
        let Self {
            context,
            sheet_order,
            sheets,
            rules,
            blocks,
            lists,
            media,
            maps,
            figures,
            ..
        } = self;
        let journals = (
            sheets.take_journal(),
            lists.take_journal(),
            media.take_journal(),
            blocks.take_journal(),
            maps.take_journal(),
            rules.take_journal(),
        );
        let mut settle = Settle {
            figures: &mut *figures,
            measurer: Measurer::new(limits),
            refusals: Refusals::default(),
            work: Work::default(),
        };
        let mut pending = Pending::default();
        if settle
            .figures
            .context()
            .is_none_or(|settled| !Arc::ptr_eq(settled, context))
        {
            if let Some(old) = settle.figures.take_context() {
                let (contribution, _) = Self::state_context(&old);
                settle.apply(&contribution, Sign::Remove);
            }
            let (contribution, refusal) = Self::state_context(context);
            settle.work.entries = settle.work.entries.saturating_add(contribution.counts.work);
            settle.apply(&contribution, Sign::Add);
            pending.contexts[STATE_CONTEXT] = refusal;
            pending.validate.push(context.as_ref());
            settle.figures.set_context(context.clone());
        }
        settle.sheets(sheets, journals.0, &mut pending);
        settle.lists(lists, journals.1);
        settle.media(media, journals.2);
        settle.blocks(blocks, journals.3, &mut pending);
        settle.maps(maps, journals.4);
        settle.rules(rules, journals.5, &mut pending);
        // What the records retain joins the ledger of the state once.
        let delta = std::mem::take(settle.measurer.delta());
        if !delta.is_empty()
            && let Err(error) = settle.figures.ledger_mut().apply(&delta)
        {
            settle.refusals.note(error);
        }
        // The media lists the batch has displaced join it for this check alone.
        let mut extra = Extra::default();
        for media in displaced {
            extra.entries = extra.entries.saturating_add(widen(media.queries().len()));
            match settle
                .measurer
                .walk(Sign::Add, &crate::source_retention::Displaced(media))
            {
                Ok(walked) => {
                    extra.steps = extra.steps.saturating_add(widen(walked.steps));
                    extra.reach = extra.reach.max(walked.reach);
                }
                Err(error) => settle.refusals.note(error),
            }
        }
        let mut undo = std::mem::take(settle.measurer.delta());
        if !undo.is_empty() {
            if let Err(error) = settle.figures.ledger_mut().apply(&undo) {
                settle.refusals.note(error);
            }
            undo.invert();
        }
        let (refusals, steps) = (settle.refusals, settle.measurer.work());
        *work = settle.work;
        crate::operation_work::walked(steps);
        let outcome = (|| {
            for context in &pending.validate {
                context.validate()?;
            }
            check_links(figures, limits)?;
            let objects = 1usize
                .checked_add(sheets.len())
                .and_then(|v| v.checked_add(rules.len()))
                .and_then(|v| v.checked_add(blocks.len()))
                .and_then(|v| v.checked_add(lists.len()))
                .and_then(|v| v.checked_add(media.len()))
                .and_then(|v| v.checked_add(maps.len()));
            limit(objects, limits.max_objects, "objects")?;
            if let Some(error) = pending.contexts.iter().flatten().next() {
                return Err(error.clone());
            }
            for sheet in &pending.sheets {
                match sheet {
                    Ok(counts) => counts.check(limits)?,
                    Err(error) => return Err(error.clone()),
                }
            }
            for counts in &pending.rules {
                counts.check(limits)?;
            }
            check_totals(figures, limits, sheet_order.len(), extra, refusals)
        })();
        // The displaced media lists leave the ledger again.
        if !undo.is_empty() {
            let _ = figures.ledger_mut().apply(&undo);
        }
        outcome
    }

    /// The contribution of the context of the state: the walk charges it and each input it links.
    fn state_context(context: &CssomContext) -> (Contribution, Option<CssomError>) {
        contribution_of_context(context, true)
    }
}

fn limit(actual: Option<usize>, maximum: usize, resource: &'static str) -> Result<(), CssomError> {
    if actual.is_none_or(|v| v > maximum) {
        Err(CssomError::Limit { resource, maximum })
    } else {
        Ok(())
    }
}

/// Refuses the state when the snapshots its contexts link are nested deeper than
/// `max_depth`, and then when its depth-first walk of them visits more than `max_entries`.
fn check_links(figures: &Figures, limits: &CssomLimits) -> Result<(), CssomError> {
    if figures.link_depth() > limits.max_depth {
        return Err(CssomError::Limit {
            resource: "linked input depth",
            maximum: limits.max_depth,
        });
    }
    if figures.link_visits > widen(limits.max_entries) {
        return Err(CssomError::Limit {
            resource: "linked input visits",
            maximum: limits.max_entries,
        });
    }
    Ok(())
}

/// Refuses the state when its entries, retained sources or string bytes exceed `limits`.
fn check_totals(
    figures: &Figures,
    limits: &CssomLimits,
    sheet_order: usize,
    extra: Extra,
    refusals: Refusals,
) -> Result<(), CssomError> {
    let entries = figures
        .entries()
        .saturating_add(widen(sheet_order))
        .saturating_add(extra.entries);
    if entries > widen(limits.max_entries) {
        return Err(CssomError::Limit {
            resource: "entries",
            maximum: limits.max_entries,
        });
    }
    // Several sources limits are exceeded in one order: the depth, the bytes, the steps, the storage.
    if refusals.depth || figures.source_depth().max(extra.reach) > limits.max_source_traversal_depth
    {
        return Err(CssomError::Limit {
            resource: DEPTH_RESOURCE,
            maximum: limits.max_source_traversal_depth,
        });
    }
    if refusals.bytes || figures.retained_bytes() > limits.max_retained_source_bytes {
        return Err(CssomError::Limit {
            resource: BYTES_RESOURCE,
            maximum: limits.max_retained_source_bytes,
        });
    }
    if refusals.steps
        || figures.walk_steps().saturating_add(extra.steps)
            > widen(limits.max_source_traversal_steps)
    {
        return Err(CssomError::Limit {
            resource: STEPS_RESOURCE,
            maximum: limits.max_source_traversal_steps,
        });
    }
    if refusals.storage {
        return Err(CssomError::Limit {
            resource: STORAGE_RESOURCE,
            maximum: limits.max_source_traversal_steps,
        });
    }
    if figures.strings() > widen(limits.max_string_bytes) {
        return Err(CssomError::Limit {
            resource: "host and live string bytes",
            maximum: limits.max_string_bytes,
        });
    }
    Ok(())
}

impl State {
    /// Refuses the settled state when its totals exceed `limits`, which a store may
    /// tighten for the objects and string bytes that pending replacements reserve.
    ///
    /// The conditions of single records were tested when each was admitted, under
    /// limits a store never changes.
    ///
    /// # Errors
    ///
    /// The first threshold the figures exceed.
    pub(crate) fn check_limits(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        crate::operation_work::census(0, 0);
        check_links(&self.figures, limits)?;
        limit(self.objects(), limits.max_objects, "objects")?;
        check_totals(
            &self.figures,
            limits,
            self.sheet_order.len(),
            Extra::default(),
            Refusals::default(),
        )
    }
}

/// Counts of the work settles do on the calling thread.
#[cfg(test)]
pub(crate) mod probe {
    use std::cell::Cell;

    thread_local! {
        static EXAMINED: Cell<usize> = const { Cell::new(0) };
        static STEPS: Cell<usize> = const { Cell::new(0) };
        static ENTRIES: Cell<usize> = const { Cell::new(0) };
        static ROOTS: Cell<usize> = const { Cell::new(0) };
        static MANIFESTS: Cell<usize> = const { Cell::new(0) };
        static RUNS: Cell<usize> = const { Cell::new(0) };
        static CROSS_CHECK: Cell<bool> = const { Cell::new(true) };
        static REFERENCE: Cell<bool> = const { Cell::new(false) };
        static UNSETTLED_PUBLICATION: Cell<bool> = const { Cell::new(false) };
        static UNMEASURABLE_BEFORE: Cell<bool> = const { Cell::new(false) };
        static PARTS: Cell<usize> = const { Cell::new(0) };
    }

    /// One payload part has been measured.
    pub(crate) fn part_measured() {
        PARTS.with(|total| total.set(total.get().saturating_add(1)));
    }
    /// The payload parts measured on this thread since the last reset.
    pub(crate) fn parts() -> usize {
        PARTS.with(Cell::get)
    }

    /// Whether a commit publishes the state it staged without settling it first,
    /// which the publication then refuses to do.
    pub(crate) fn unsettled_publication() -> bool {
        UNSETTLED_PUBLICATION.with(Cell::get)
    }
    pub(crate) fn set_unsettled_publication(value: bool) {
        UNSETTLED_PUBLICATION.with(|flag| flag.set(value));
    }

    /// Whether measuring what a record contributed, before it changes, fails as an
    /// allocation failure does.
    pub(crate) fn unmeasurable_before() -> bool {
        UNMEASURABLE_BEFORE.with(Cell::get)
    }
    pub(crate) fn set_unmeasurable_before(value: bool) {
        UNMEASURABLE_BEFORE.with(|flag| flag.set(value));
    }

    /// Whether every settle answers as the reference census of the whole state does.
    pub(crate) fn reference() -> bool {
        REFERENCE.with(Cell::get)
    }
    pub(crate) fn set_reference(value: bool) {
        REFERENCE.with(|flag| flag.set(value));
    }
    /// Whether each settle is compared with the reference census of the whole
    /// state (on by default).
    pub(crate) fn cross_check() -> bool {
        CROSS_CHECK.with(Cell::get)
    }
    pub(crate) fn set_cross_check(value: bool) {
        CROSS_CHECK.with(|flag| flag.set(value));
    }

    /// The checks made, records measured, entry lists examined and source walk
    /// steps measured.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct Work {
        pub(crate) runs: usize,
        pub(crate) examined: usize,
        pub(crate) entries: usize,
        pub(crate) steps: usize,
    }

    /// A linked state has joined the states a state reaches, or left them.
    pub(crate) fn root_reached() {
        ROOTS.with(|total| total.set(total.get().saturating_add(1)));
    }
    /// The linked states that joined or left since the last reset.
    pub(crate) fn roots() -> usize {
        ROOTS.with(Cell::get)
    }
    /// The input manifest of a whole state has been built from nothing.
    pub(crate) fn manifest_built() {
        MANIFESTS.with(|total| total.set(total.get().saturating_add(1)));
    }
    /// The input manifests built from nothing on this thread since the last reset.
    pub(crate) fn manifests() -> usize {
        MANIFESTS.with(Cell::get)
    }

    /// One check that examined `entries` entry lists.
    pub(crate) fn ran(entries: usize) {
        RUNS.with(|total| total.set(total.get().saturating_add(1)));
        ENTRIES.with(|total| total.set(total.get().saturating_add(entries)));
    }

    pub(crate) fn examined(objects: usize) {
        EXAMINED.with(|total| total.set(total.get().saturating_add(objects)));
    }
    pub(crate) fn steps(steps: usize) {
        STEPS.with(|total| total.set(total.get().saturating_add(steps)));
    }
    pub(crate) fn reset() {
        EXAMINED.with(|total| total.set(0));
        STEPS.with(|total| total.set(0));
        ENTRIES.with(|total| total.set(0));
        ROOTS.with(|total| total.set(0));
        RUNS.with(|total| total.set(0));
        MANIFESTS.with(|total| total.set(0));
        PARTS.with(|total| total.set(0));
    }
    pub(crate) fn read() -> Work {
        Work {
            runs: RUNS.with(Cell::get),
            examined: EXAMINED.with(Cell::get),
            entries: ENTRIES.with(Cell::get),
            steps: STEPS.with(Cell::get),
        }
    }
}

#[cfg(test)]
mod differential_tests;
#[cfg(test)]
mod edit_work_tests;
#[cfg(test)]
mod memo_tests;
#[cfg(test)]
mod reference;
#[cfg(test)]
mod settle_tests;
#[cfg(test)]
mod verified;

#[cfg(test)]
mod bounded_work_tests {
    use super::probe;
    use crate::*;

    /// The work census runs do while one batch creates a sheet of `rules` style rules.
    fn creation_work(rules: usize) -> probe::Work {
        let text = (0..rules)
            .map(|index| format!(".r{index} {{ color: red; margin: 1px 0 }}\n"))
            .collect::<String>();
        let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
        let mut edit = store
            .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
            .unwrap();
        let inputs = CssomSheetInputs::external(
            CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: "census-work".into(),
                revision: 1,
            },
            true,
        );
        probe::reset();
        edit.create_parsed_sheet(&text, inputs).unwrap();
        let work = probe::read();
        store.commit(edit).unwrap();
        work
    }

    #[test]
    fn creating_a_sheet_does_census_work_in_proportion_to_its_rules() {
        // The cross-check runs the census of the whole state it is compared with.
        probe::set_cross_check(false);
        let sizes = [100, 200, 400].map(creation_work);
        for pair in sizes.windows(2) {
            let (small, large) = (pair[0], pair[1]);
            // Doubling the rules at most a little more than doubles the work.
            assert!(
                large.examined * 4 <= small.examined * 9,
                "objects examined grew from {} to {} (100, 200, 400 rules: {sizes:?})",
                small.examined,
                large.examined
            );
            assert!(
                large.steps * 4 <= small.steps * 9,
                "walk steps grew from {} to {} (100, 200, 400 rules: {sizes:?})",
                small.steps,
                large.steps
            );
        }
    }
}
