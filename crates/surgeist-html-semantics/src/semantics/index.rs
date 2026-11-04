//! Where each subject's fact sits among a producer's facts.
//!
//! Finding the fact of a subject by scanning the facts takes work linear in
//! them, so a producer that looks up facts for many subjects takes quadratic
//! work. The index is sized once by the number of facts, which the producer's
//! capacity check has already bounded, and answers each lookup in one probe.
//!
//! Keys are the dense positions of subjects within their capture
//! ([`HtmlCaptureSubjectId::ordinal`]), never anything that depends on an
//! address or a process-global counter, so lookups charge the same work on every
//! run. Only subjects of the indexed capture are entered, because positions of
//! different captures collide; a subject of another capture is never found.

use crate::{HtmlCaptureId, HtmlCaptureSubjectId};
use std::collections::HashMap;

pub(crate) struct FactIndex {
    capture: HtmlCaptureId,
    positions: HashMap<usize, usize>,
}

impl FactIndex {
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

    /// Enters the fact at `position` for `subject` unless an earlier fact is
    /// already entered for it, so the first of equal subjects stays the answer.
    /// A subject of another capture is not entered.
    pub(crate) fn enter(&mut self, subject: &HtmlCaptureSubjectId, position: usize) {
        if subject.capture() == &self.capture {
            self.positions.entry(subject.ordinal()).or_insert(position);
        }
    }

    /// The position of the first fact entered for `subject`.
    pub(crate) fn position(&self, subject: &HtmlCaptureSubjectId) -> Option<usize> {
        if subject.capture() == &self.capture {
            self.positions.get(&subject.ordinal()).copied()
        } else {
            None
        }
    }
}
