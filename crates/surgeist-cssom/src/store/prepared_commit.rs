//! Exclusive preparation and a publication tail containing only reserved moves.
use super::*;
use crate::sheet_replacement::PendingReplacement;
use std::{
    collections::hash_map::{Entry, OccupiedEntry},
    fmt,
};

/// Private completed producer work under one exclusive Store loan. No effect,
/// creation ticket resolution or tentative snapshot is available before publish.
/// Drop abandons staging and releases each private effect's own quota reservation.
///
/// The exclusive borrow prevents observation/mutation through the Store:
/// ```compile_fail,E0502
/// use surgeist_cssom::{CssomStore, CssomEditGuard};
/// let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
/// let batch = store.batch(CssomEditGuard::from_snapshot(&store.snapshot())).unwrap();
/// let prepared = store.prepare_commit(batch).unwrap();
/// let snapshot = store.snapshot(); // the prepared writer still owns its loan
/// let _ = prepared.publish();
/// ```
#[must_use = "publish or drop this private prepared commit"]
pub struct CssomPreparedCommit<'store> {
    state: &'store mut Arc<State>,
    history: &'store mut VecDeque<CssomChangeSummary>,
    next_effect: &'store mut u64,
    // Only changed preparations have successor/terminal authority. History is
    // present exactly when changed and history retention is enabled.
    successor: Option<Arc<State>>,
    history_summary: Option<CssomChangeSummary>,
    history_limit: usize,
    terminal: Option<OccupiedEntry<'store, CssomSheetId, PendingReplacement>>,
    output: CssomCommit,
    effect_successor: u64,
}
impl fmt::Debug for CssomPreparedCommit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CssomPreparedCommit")
            .finish_non_exhaustive()
    }
}

/// Dead backing removed by publication. It has no execution, acknowledgement,
/// cancellation or Store authority. Drop it after the caller's composed fence.
/// Current logical quotas exclude caller-held retirement and old snapshots;
/// this capsule is not a total physical-memory or bounded destructor guarantee.
pub struct CssomCommitRetirement {
    _state: Option<Arc<State>>,
    _evicted_summary: Option<CssomChangeSummary>,
    _pending_replacement: Option<(CssomSheetId, PendingReplacement)>,
}
impl fmt::Debug for CssomCommitRetirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CssomCommitRetirement")
            .finish_non_exhaustive()
    }
}

impl CssomPreparedCommit<'_> {
    /// Publish by prechecked moves, prelocated removal and reserved deque work.
    /// No allocation, serialization, payload clone, callback, fallible lookup or
    /// potentially large last-owner destruction occurs in this tail.
    pub fn publish(self) -> (CssomCommit, CssomCommitRetirement) {
        let Self {
            state,
            history,
            next_effect,
            successor,
            history_summary,
            history_limit,
            terminal,
            output,
            effect_successor,
        } = self;
        let mut retirement = CssomCommitRetirement {
            _state: None,
            _evicted_summary: None,
            _pending_replacement: None,
        };
        if let Some(successor) = successor {
            if let Some(summary) = history_summary {
                history.push_back(summary);
                if history.len() > history_limit {
                    retirement._evicted_summary = history.pop_front();
                }
            }
            retirement._state = Some(std::mem::replace(state, successor));
            if let Some(terminal) = terminal {
                retirement._pending_replacement = Some(terminal.remove_entry());
            }
        }
        *next_effect = effect_successor;
        (output, retirement)
    }
}

struct Ready {
    successor: Option<Arc<State>>,
    history_summary: Option<CssomChangeSummary>,
    terminal_sheet: Option<CssomSheetId>,
    output: CssomCommit,
    effect_successor: u64,
}

impl CssomStore {
    /// Consumes the attempted batch and prepares all producer work privately.
    /// An error or later Drop does not recover/replay the input. Operational
    /// history capacity and per-effect reservations may be acquired here, but
    /// canonical state/history, identities and replacement membership stay put.
    pub fn prepare_commit(
        &mut self,
        batch: CssomBatch,
    ) -> Result<CssomPreparedCommit<'_>, CssomError> {
        // This function boundary disposes unused batch/scratch before lending
        // disjoint publication fields. Only fully ready output crosses it.
        let ready = prepare_data(self, batch)?;
        let Self {
            state,
            history,
            pending_replacements,
            next_effect,
            limits,
            ..
        } = self;
        let terminal = ready
            .terminal_sheet
            .map(|sheet| match pending_replacements.entry(sheet) {
                Entry::Occupied(entry) => entry,
                Entry::Vacant(_) => {
                    unreachable!("authenticated terminal under exclusive Store loan")
                }
            });
        Ok(CssomPreparedCommit {
            state,
            history,
            next_effect,
            successor: ready.successor,
            history_summary: ready.history_summary,
            history_limit: limits.summary_history,
            terminal,
            output: ready.output,
            effect_successor: ready.effect_successor,
        })
    }
}

fn prepare_data(store: &mut CssomStore, batch: CssomBatch) -> Result<Ready, CssomError> {
    // Keep direct commit's qualification/error precedence intact.
    store.check_guard(&CssomEditGuard {
        revision: batch.base.revision.clone(),
        context: batch.base.context.clone(),
    })?;
    if batch.pending_preparations.load(Ordering::Relaxed) != 0 {
        return Err(CssomError::UnresolvedDeclarationPreparation);
    }
    if batch.poisoned {
        return Err(CssomError::BatchAborted);
    }
    let reserved = store.replacement_reservations(batch.terminal_replacement.as_ref())?;
    batch
        .staged
        .check_limits(&store.replacement_limits(reserved)?)?;
    if !batch.changed() {
        let (effects, effect_successor) =
            store.reserve_owner_effects(&batch, &store.state.revision)?;
        return Ok(Ready {
            successor: None,
            history_summary: None,
            terminal_sheet: None,
            effect_successor,
            output: CssomCommit {
                effects,
                token: batch.token,
                created: batch.created,
                publication: CssomPublication::Unchanged {
                    revision: store.state.revision.clone(),
                },
            },
        });
    }
    let revision = store
        .state
        .revision
        .value
        .checked_add(1)
        .filter(|v| {
            u64::try_from(reserved.0)
                .ok()
                .and_then(|count| v.checked_add(count))
                .is_some_and(|end| end <= store.limits.max_revision)
        })
        .ok_or(CssomError::RevisionExhausted)?;
    let effect_revision = CssomRevision {
        owner: store.state.owner.clone(),
        value: revision,
    };
    let (effects, effect_successor) = store.reserve_owner_effects(&batch, &effect_revision)?;
    let before = store.snapshot();
    let mut next = batch.staged;
    next.revision = CssomRevision {
        owner: next.owner.clone(),
        value: revision,
    };
    let next = Arc::new(next);
    let after = CssomSnapshot {
        state: next.clone(),
    };
    let summary = CssomChangeSummary {
        owner: store.state.owner.clone(),
        from: store.state.revision.clone(),
        to: next.revision.clone(),
        categories: batch.categories,
        affected: batch.affected,
        before_inputs: before.inputs(),
        after_inputs: after.inputs(),
        full_recompute: batch.full_recompute,
    };
    let history_summary = if store.limits.summary_history > 0 {
        let slots = reserved.0.checked_add(1).ok_or(CssomError::Limit {
            resource: "summary capacity",
            maximum: store.limits.summary_history,
        })?;
        store
            .history
            .try_reserve(slots)
            .map_err(|_| CssomError::Limit {
                resource: "summary capacity",
                maximum: store.limits.summary_history,
            })?;
        Some(summary.clone())
    } else {
        None
    };
    let terminal_sheet = batch
        .terminal_replacement
        .map(|token| token.sheet().clone());
    Ok(Ready {
        successor: Some(next),
        history_summary,
        terminal_sheet,
        effect_successor,
        output: CssomCommit {
            effects,
            token: batch.token,
            created: batch.created,
            publication: CssomPublication::Changed(Box::new(summary)),
        },
    })
}
