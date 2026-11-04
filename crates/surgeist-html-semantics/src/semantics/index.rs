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
