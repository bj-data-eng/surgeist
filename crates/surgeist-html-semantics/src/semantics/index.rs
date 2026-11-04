//! Where each subject's fact sits among a producer's facts.
//!
//! Finding the fact of a subject by scanning the facts takes work linear in
//! them, so a producer or a query that looks up facts for many subjects takes
//! quadratic work. The index is sized once by the number of facts, which the
//! producer's capacity check has already bounded, and answers each lookup in
//! one probe.
//!
//! Keys are the dense positions of subjects within their capture
//! ([`HtmlCaptureSubjectId::ordinal`]) with whatever else the fact is bound to
//! (a source, a token position), never anything that depends on an address or a
//! process-global counter, so lookups charge the same work on every run. Only
//! subjects of the indexed capture are entered, because positions of different
//! captures collide; a subject of another capture is never found.

use crate::{HtmlCaptureId, HtmlCaptureSubjectId};
use std::{collections::HashMap, hash::Hash};

/// First positions of facts by subject and by the binding `K` within the subject.
#[derive(Debug)]
pub(crate) struct FactIndex<K = ()> {
    capture: HtmlCaptureId,
    positions: HashMap<(usize, K), usize>,
}

impl<K: Hash + Eq> FactIndex<K> {
    /// Charges the fill of `facts` entries through `charge`, then allocates room
    /// for them; `exhausted` is the producer's refusal when allocation fails.
    pub(crate) fn new<E>(
        capture: &HtmlCaptureId,
        facts: usize,
        charge: impl FnOnce(usize) -> Result<(), E>,
        exhausted: E,
    ) -> Result<Self, E> {
        charge(facts)?;
        let mut positions = HashMap::new();
        positions.try_reserve(facts).map_err(|_| exhausted)?;
        Ok(Self {
            capture: capture.clone(),
            positions,
        })
    }

    /// Enters the fact at `position` for `subject` and `key` unless an earlier
    /// fact is already entered for them, so the first of equal bindings stays
    /// the answer. A subject of another capture is not entered.
    pub(crate) fn enter_bound(&mut self, subject: &HtmlCaptureSubjectId, key: K, position: usize) {
        if subject.capture() == &self.capture {
            self.positions
                .entry((subject.ordinal(), key))
                .or_insert(position);
        }
    }

    /// Enters the fact at `position` as the latest for `subject` and `key`,
    /// returning the position it replaces. A subject of another capture is not
    /// entered.
    pub(crate) fn enter_latest(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        key: K,
        position: usize,
    ) -> Option<usize> {
        if subject.capture() == &self.capture {
            self.positions.insert((subject.ordinal(), key), position)
        } else {
            None
        }
    }

    /// Removes the entry of `subject` and `key`, when it has one.
    pub(crate) fn forget_bound(&mut self, subject: &HtmlCaptureSubjectId, key: K) {
        if subject.capture() == &self.capture {
            self.positions.remove(&(subject.ordinal(), key));
        }
    }

    /// Moves the entry of `subject` and `key` to `position`, when it has one.
    pub(crate) fn move_bound(&mut self, subject: &HtmlCaptureSubjectId, key: K, position: usize) {
        if subject.capture() == &self.capture
            && let Some(at) = self.positions.get_mut(&(subject.ordinal(), key))
        {
            *at = position;
        }
    }

    /// Whether an earlier fact than the one at `position` is entered for `subject`
    /// and `key`, which is what makes that fact a duplicate of it. The fact at
    /// `position` must itself have been entered.
    pub(crate) fn is_repeat(
        &self,
        subject: &HtmlCaptureSubjectId,
        key: K,
        position: usize,
    ) -> bool {
        self.bound_position(subject, key)
            .is_some_and(|first| first != position)
    }

    /// The position of the first fact entered for `subject` and `key`.
    pub(crate) fn bound_position(&self, subject: &HtmlCaptureSubjectId, key: K) -> Option<usize> {
        if subject.capture() == &self.capture {
            self.positions.get(&(subject.ordinal(), key)).copied()
        } else {
            None
        }
    }
}

impl FactIndex {
    /// As [`enter_bound`](Self::enter_bound) for facts bound to a subject alone.
    pub(crate) fn enter(&mut self, subject: &HtmlCaptureSubjectId, position: usize) {
        self.enter_bound(subject, (), position);
    }

    /// As [`bound_position`](Self::bound_position) for facts bound to a subject alone.
    pub(crate) fn position(&self, subject: &HtmlCaptureSubjectId) -> Option<usize> {
        self.bound_position(subject, ())
    }
}

/// Facts grouped by subject, each fact linked to the previous fact of its
/// subject, for a check that compares only facts of one subject.
///
/// A check that compared each fact with every earlier one takes work quadratic
/// in the facts, but the facts it can match all share a subject, and the
/// grouping reaches those alone. A keyed index would need a deterministic key
/// for what the check compares besides the subject, and that may be an opaque
/// identity. The chains are sized once by the number of facts.
#[derive(Debug)]
pub(crate) struct SubjectChains {
    latest: FactIndex,
    before: Vec<Option<usize>>,
}

impl SubjectChains {
    /// Charges the fill of `facts` entries twice through `charge` (the latest
    /// fact of each subject and the link of each fact), then allocates room for
    /// them; `exhausted` is the producer's refusal when allocation fails.
    pub(crate) fn new<E: Clone>(
        capture: &HtmlCaptureId,
        facts: usize,
        mut charge: impl FnMut(usize) -> Result<(), E>,
        exhausted: E,
    ) -> Result<Self, E> {
        let latest = FactIndex::new(capture, facts, &mut charge, exhausted.clone())?;
        charge(facts)?;
        let mut before = Vec::new();
        before.try_reserve(facts).map_err(|_| exhausted)?;
        Ok(Self { latest, before })
    }

    /// Links the next fact, which must be the one at position `self.len()`.
    pub(crate) fn link(&mut self, subject: &HtmlCaptureSubjectId) {
        let position = self.before.len();
        let previous = self.latest.enter_latest(subject, (), position);
        self.before.push(previous);
    }

    /// The latest fact of `subject`.
    pub(crate) fn latest(&self, subject: &HtmlCaptureSubjectId) -> Option<usize> {
        self.latest.position(subject)
    }

    /// The fact of the same subject before the one at `position`.
    pub(crate) fn before(&self, position: usize) -> Option<usize> {
        self.before.get(position).copied().flatten()
    }
}
