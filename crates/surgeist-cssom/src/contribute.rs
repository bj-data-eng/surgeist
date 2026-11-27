//! What each kind of record contributes to the figures of its state, measured.
//!
//! A settle measures the record a key holds now. The record it held at the last
//! settle is not kept: the first time a table is asked for mutable access to a key
//! after a settle it takes what that record contributed ([`Before`]) and journals
//! that, so a record that is changed in place is never copied for the journal's sake
//! however large it is, and a part that is changed in place is not copied either.
use crate::figures::{Contribution, contribution_of_context};
use crate::shared_table::Journaled;
use crate::source_retention::{Measurer, Sign, Walkable, Walked};
use crate::state_limits::measure;
use crate::*;
use std::sync::Arc;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::source::{CssSourceDelta, CssSourceRetentionError};

/// One record measured.
pub(crate) struct Measured {
    pub(crate) contribution: Contribution,
    /// The refusal of a context that links a sheet its snapshot lacks.
    pub(crate) context: Option<CssomError>,
    /// The refusal of the walk of the record's retained sources.
    pub(crate) walk: Option<CssSourceRetentionError>,
}

/// What a record contributed when it was last settled, and what the ledger of
/// retained snapshots must give back for it.
#[derive(Clone, Debug)]
pub(crate) struct Before {
    pub(crate) contribution: Contribution,
    /// The snapshots the record retained, as the change that removes them.
    pub(crate) removal: Option<Arc<CssSourceDelta>>,
    /// The context the walk of the record enqueued.
    pub(crate) context: Option<Arc<CssomContext>>,
    /// The refusal of measuring the record, when what it retained could not be counted: the
    /// contribution is then short, and the settle that takes it out refuses.
    pub(crate) walk: Option<CssSourceRetentionError>,
}

impl Before {
    /// What `measured` took from a record, with the snapshots `measurer` collected.
    fn of(measured: Measured, mut measurer: Measurer, context: Option<&Arc<CssomContext>>) -> Self {
        crate::operation_work::walked(measurer.work());
        let removal = std::mem::take(measurer.delta());
        #[cfg(test)]
        let measured = if crate::state_limits::probe::unmeasurable_before() {
            Measured {
                walk: Some(CssSourceRetentionError::allocation()),
                ..measured
            }
        } else {
            measured
        };
        Self {
            contribution: measured.contribution,
            removal: (!removal.is_empty()).then(|| Arc::new(removal)),
            context: context.cloned(),
            walk: measured.walk,
        }
    }
}

fn walked(
    measurer: &mut Measurer,
    sign: Sign,
    record: &impl Walkable,
) -> (Walked, Option<CssSourceRetentionError>) {
    match measurer.walk(sign, record) {
        Ok(walked) => (walked, None),
        Err(error) => (Walked::default(), Some(error)),
    }
}

/// A record whose walk is `walked` and whose counts are `counts`, holding `context`.
fn with_context(
    counts: measure::Counts,
    walked: (Walked, Option<CssSourceRetentionError>),
    context: Option<&CssomContext>,
    validated: bool,
) -> Measured {
    let (walked, walk) = walked;
    let mut contribution = Contribution {
        counts,
        walked,
        ..Contribution::default()
    };
    let mut refusal = None;
    if let Some(context) = context {
        let (of_context, error) = contribution_of_context(context, validated);
        contribution.counts.absorb(&of_context.counts);
        contribution.links = of_context.links;
        contribution.validated = validated;
        refusal = error;
    }
    Measured {
        contribution,
        context: refusal,
        walk,
    }
}

pub(crate) fn sheet(measurer: &mut Measurer, sheet: &CssomSheet, sign: Sign) -> Measured {
    let walked = walked(measurer, sign, sheet);
    with_context(measure::sheet(sheet), walked, None, false)
}

pub(crate) fn list(members: &[CssomRuleId]) -> Measured {
    with_context(
        measure::list(members.len()),
        (Walked::default(), None),
        None,
        false,
    )
}

pub(crate) fn media(measurer: &mut Measurer, media: &CssMediaQueryList, sign: Sign) -> Measured {
    let walked = walked(measurer, sign, media);
    with_context(measure::media(media), walked, None, false)
}

pub(crate) fn map(measurer: &mut Measurer, map: &CssomFeatureMap, sign: Sign) -> Measured {
    let walked = walked(measurer, sign, map);
    with_context(measure::map(map), walked, None, false)
}

/// The context of the inline declarations of a block.
pub(crate) fn block_context(block: &CssomBlock) -> Option<&Arc<CssomContext>> {
    block
        .inline_context()
        .map(CssomInlineDeclarationContext::shared_context)
}

pub(crate) fn block(measurer: &mut Measurer, block: &CssomBlock, sign: Sign) -> Measured {
    let walked = walked(measurer, sign, block);
    with_context(
        measure::block(block),
        walked,
        block_context(block).map(|context| &**context),
        true,
    )
}

pub(crate) fn rule(measurer: &mut Measurer, rule: &CssomRule, sign: Sign) -> Measured {
    let walked = walked(measurer, sign, rule);
    with_context(
        measure::rule(rule),
        walked,
        rule.selector_inputs.as_deref(),
        false,
    )
}

impl Journaled for CssomSheet {
    type Before = Before;
    fn before(&self) -> Before {
        let mut measurer = Measurer::unbounded();
        let measured = sheet(&mut measurer, self, Sign::Remove);
        Before::of(measured, measurer, None)
    }
}

impl Journaled for Vec<CssomRuleId> {
    type Before = Before;
    fn before(&self) -> Before {
        Before::of(list(self), Measurer::unbounded(), None)
    }
}

impl Journaled for CssMediaQueryList {
    type Before = Before;
    fn before(&self) -> Before {
        let mut measurer = Measurer::unbounded();
        let measured = media(&mut measurer, self, Sign::Remove);
        Before::of(measured, measurer, None)
    }
}

impl Journaled for CssomFeatureMap {
    type Before = Before;
    fn before(&self) -> Before {
        let mut measurer = Measurer::unbounded();
        let measured = map(&mut measurer, self, Sign::Remove);
        Before::of(measured, measurer, None)
    }
}

impl Journaled for CssomBlock {
    type Before = Before;
    fn before(&self) -> Before {
        let mut measurer = Measurer::unbounded();
        let measured = block(&mut measurer, self, Sign::Remove);
        Before::of(measured, measurer, block_context(self))
    }
}

impl Journaled for CssomRule {
    type Before = Before;
    fn before(&self) -> Before {
        let mut measurer = Measurer::unbounded();
        let measured = rule(&mut measurer, self, Sign::Remove);
        Before::of(measured, measurer, self.selector_inputs.as_ref())
    }
}
