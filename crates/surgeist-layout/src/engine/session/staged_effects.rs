//! The staged effects of a computation, recorded so that a reused computation
//! can repeat them where a recomputation would have.
//!
//! A cache store, a cache clear and an unrounded geometry write each replace an
//! earlier write to the same slot and move it last; a formatter family's native
//! runs and completion are removed and appended again each time the formatter
//! computes. A repeat is therefore observable in the order the batch publishes.
//! Reusing a computation replays the writes the computation staged, in order,
//! through the same session methods. A write that cannot be recorded, or a
//! computation that changed state this log does not cover, leaves the capture
//! incomplete and the computation is repeated instead of reused.

use std::sync::Arc;

use crate::inline_wrapping::family::{NativeFormatterCompletion, NativePublication};
use crate::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, InlineFragmentOutputOf, LayoutOutputEntryOf,
    LayoutScalar,
};

/// One write a computation staged in its session.
pub(crate) enum StagedEffect<Node, S: LayoutScalar> {
    Store {
        node: Node,
        input: ComputeInputOf<S>,
        context: CacheKeyContext,
        output: ComputeOutputOf<S>,
    },
    Clear(Node),
    Geometry(LayoutOutputEntryOf<Node, S>),
    InlineFragments {
        node: Node,
        fragments: Option<Vec<InlineFragmentOutputOf<S>>>,
    },
    /// A formatter family's native runs and completion were removed to be
    /// computed again.
    NativeReset {
        family: usize,
    },
    /// A formatter family's native runs, but not its completion, were removed
    /// to be accepted again from a float probe.
    NativeRunsDrop {
        family: usize,
    },
    NativeRun {
        node: Node,
        run: NativePublication<S>,
    },
    NativeCompletion {
        node: Node,
        completion: NativeFormatterCompletion<S>,
    },
}

impl<Node: Copy, S: LayoutScalar> StagedEffect<Node, S> {
    /// A copy, or `None` when it could not be allocated.
    pub(super) fn try_clone(&self) -> Option<Self> {
        Some(match self {
            Self::Store {
                node,
                input,
                context,
                output,
            } => Self::Store {
                node: *node,
                input: *input,
                context: *context,
                output: *output,
            },
            Self::Clear(node) => Self::Clear(*node),
            Self::Geometry(entry) => Self::Geometry(*entry),
            Self::InlineFragments { node, fragments } => Self::InlineFragments {
                node: *node,
                fragments: match fragments {
                    Some(fragments) => {
                        let mut copy = Vec::new();
                        copy.try_reserve_exact(fragments.len()).ok()?;
                        copy.extend_from_slice(fragments);
                        Some(copy)
                    }
                    None => None,
                },
            },
            Self::NativeReset { family } => Self::NativeReset { family: *family },
            Self::NativeRunsDrop { family } => Self::NativeRunsDrop { family: *family },
            Self::NativeRun { node, run } => Self::NativeRun {
                node: *node,
                run: run.try_clone().ok()?,
            },
            Self::NativeCompletion { node, completion } => Self::NativeCompletion {
                node: *node,
                completion: *completion,
            },
        })
    }
}

/// The writes of one computation, in the order it staged them.
pub(crate) type StagedEffects<Node, S> = Arc<[StagedEffect<Node, S>]>;

/// What a capture stands in for. A reused preparation of a subgrid wrapper may
/// have been staged amid other computations of the same nodes, so a text
/// placement inside it, written last by whichever computation of its node ran
/// last, cannot be reused. A reused request is replayed where its own
/// recomputation would have run, after the first pass and before any other
/// computation of its subtree. Text placement is keyed by the container it
/// places and replaced in place, so the request's recomputation writes the
/// same keys with the same values, and its text effects need no poison.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureKind {
    Preparation,
    Request,
}

struct Capture {
    kind: CaptureKind,
    incomplete: bool,
}

#[cfg(test)]
thread_local! {
    static DEEPEST_CAPTURE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The most preparation captures that were open at once on this thread since the
/// last call, for tests of how deeply preparations nest.
#[cfg(test)]
pub(crate) fn take_deepest_capture() -> usize {
    DEEPEST_CAPTURE.with(std::cell::Cell::take)
}

/// A session that is dropped with a capture open, other than by a panic, has
/// left it unclosed.
#[cfg(test)]
impl<Node, S: LayoutScalar> Drop for EffectLog<Node, S> {
    fn drop(&mut self) {
        assert!(
            self.open == 0 || std::thread::panicking(),
            "{} captures were left open",
            self.open
        );
    }
}

/// Records the writes staged while at least one capture is open. Captures nest.
pub(super) struct EffectLog<Node, S: LayoutScalar> {
    effects: Vec<StagedEffect<Node, S>>,
    captures: Vec<Capture>,
    /// How many captures are open, including those a failed reservation kept
    /// out of `captures`.
    open: usize,
    /// A capture could not be reserved: every capture closes incomplete until
    /// all of them have closed.
    broken: bool,
}

impl<Node: Copy, S: LayoutScalar> EffectLog<Node, S> {
    pub(super) const fn new() -> Self {
        Self {
            effects: Vec::new(),
            captures: Vec::new(),
            open: 0,
            broken: false,
        }
    }

    /// Whether a write staged now would be recorded.
    pub(super) fn recording(&self) -> bool {
        !self.broken && self.captures.iter().any(|capture| !capture.incomplete)
    }

    /// Records a write.
    pub(super) fn record(&mut self, effect: StagedEffect<Node, S>) {
        if self.effects.try_reserve(1).is_err() {
            self.poison_all();
            return;
        }
        self.effects.push(effect);
    }

    /// Leaves every open capture incomplete: a write could not be recorded.
    pub(super) fn poison_all(&mut self) {
        self.poison(None);
    }

    /// Whether no capture is open.
    pub(super) fn is_idle(&self) -> bool {
        self.open == 0
    }

    /// Leaves the open captures of `kind`, or of every kind, incomplete: the
    /// computation changed state the log does not cover.
    pub(super) fn poison(&mut self, kind: Option<CaptureKind>) {
        for capture in &mut self.captures {
            if kind.is_none_or(|kind| capture.kind == kind) {
                capture.incomplete = true;
            }
        }
    }

    /// Opens a capture and returns where its writes begin.
    pub(super) fn begin(&mut self, kind: CaptureKind) -> usize {
        self.open += 1;

        if !self.broken {
            if self.captures.try_reserve(1).is_ok() {
                self.captures.push(Capture {
                    kind,
                    incomplete: false,
                });
            } else {
                self.broken = true;
            }
        }
        #[cfg(test)]
        {
            let preparing = self
                .captures
                .iter()
                .filter(|capture| capture.kind == CaptureKind::Preparation)
                .count();
            DEEPEST_CAPTURE.with(|deepest| deepest.set(deepest.get().max(preparing)));
        }
        self.effects.len()
    }

    /// Closes the innermost capture and returns its writes, which are no
    /// longer in the log, and whether the capture is complete. The caller
    /// returns the writes the enclosing captures need with `restore`.
    pub(super) fn finish(&mut self, start: usize) -> (Vec<StagedEffect<Node, S>>, bool) {
        self.open -= 1;
        let complete = !self.broken
            && self
                .captures
                .pop()
                .is_some_and(|capture| !capture.incomplete);
        let writes = self.effects.split_off(start);
        if self.open == 0 {
            self.broken = false;
            self.captures.clear();
            self.effects.clear();
        }
        (writes, complete)
    }

    /// Returns the writes of a closed capture to the log, for the captures
    /// that enclose it.
    pub(super) fn restore(&mut self, writes: &[StagedEffect<Node, S>]) {
        if self.open == 0 || !self.recording() {
            return;
        }
        for write in writes {
            match write.try_clone() {
                Some(copy) => self.record(copy),
                None => {
                    self.poison_all();
                    return;
                }
            }
        }
    }
}
