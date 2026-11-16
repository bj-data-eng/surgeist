//! The staged effects of a computation, recorded so that a reused computation
//! can repeat them where a recomputation would have.
//!
//! A cache store, a cache clear and an unrounded geometry write each replace an
//! earlier write to the same slot and move it last, so a repeat is observable
//! in the order the batch publishes. Reusing a computation therefore replays
//! the writes the computation staged, in order, through the same session
//! methods.

use std::sync::Arc;

use crate::{CacheKeyContext, ComputeInputOf, ComputeOutputOf, LayoutOutputEntryOf, LayoutScalar};

/// One write a computation staged in its session.
#[derive(Clone, Copy)]
pub(crate) enum StagedEffect<Node, S: LayoutScalar> {
    Store {
        node: Node,
        input: ComputeInputOf<S>,
        context: CacheKeyContext,
        output: ComputeOutputOf<S>,
    },
    Clear(Node),
    Geometry(LayoutOutputEntryOf<Node, S>),
}

/// The writes of one computation, in the order it staged them.
pub(crate) type StagedEffects<Node, S> = Arc<[StagedEffect<Node, S>]>;

/// Records the writes staged while at least one capture is open. Captures nest.
pub(super) struct EffectLog<Node, S: LayoutScalar> {
    effects: Vec<StagedEffect<Node, S>>,
    open: usize,
    incomplete: bool,
}

impl<Node: Copy, S: LayoutScalar> EffectLog<Node, S> {
    pub(super) const fn new() -> Self {
        Self {
            effects: Vec::new(),
            open: 0,
            incomplete: false,
        }
    }

    pub(super) fn record(&mut self, effect: StagedEffect<Node, S>) {
        if self.open == 0 || self.incomplete {
            return;
        }
        if self.effects.try_reserve(1).is_err() {
            self.incomplete = true;
            return;
        }
        self.effects.push(effect);
    }

    /// Opens a capture and returns where its writes begin.
    pub(super) fn begin(&mut self) -> usize {
        self.open += 1;
        self.effects.len()
    }

    /// Closes the innermost capture, returning its writes, or `None` when a
    /// write could not be recorded.
    pub(super) fn end(&mut self, start: usize) -> Option<StagedEffects<Node, S>> {
        let captured = if self.incomplete {
            None
        } else {
            let mut writes = Vec::new();
            writes
                .try_reserve_exact(self.effects.len() - start)
                .ok()
                .map(|()| {
                    writes.extend_from_slice(&self.effects[start..]);
                    Arc::from(writes)
                })
        };
        self.open -= 1;
        if self.open == 0 {
            self.effects.clear();
            self.incomplete = false;
        }
        captured
    }
}
