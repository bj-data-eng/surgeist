//! Executed lexical occurrences, before correspondence coalescence.

use std::sync::Arc;

use super::{
    DeletionCause, MappingPiece, PlaintextToken, Range, WhiteSpace, contextual::ScopeCapture,
};

#[cfg(test)]
mod tests;

/// Identity construction did not execute Phase I. An executed empty inventory
/// is a different fact from that untransformed input.
#[derive(Debug)]
pub(crate) enum LexicalState {
    Untransformed,
    Executed {
        context: LexicalContext,
        records: Vec<LexicalRecord>,
    },
}

#[derive(Debug)]
pub(crate) enum LexicalContext {
    SourceLocal,
    Captured(Arc<ScopeCapture>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LexicalRecord {
    pub(crate) domain: usize,
    pub(crate) kind: LexicalRecordKind,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum LexicalRecordKind {
    Character {
        source: Range,
        mode: WhiteSpace,
        role: LexicalCharacterRole,
        disposition: LexicalDisposition,
    },
    ScopeMarker {
        event: usize,
    },
    Barrier {
        origin: LexicalBarrierOrigin,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LexicalCharacterRole {
    Text,
    BidiControl,
    Space,
    SegmentBreak,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum LexicalDisposition {
    Emitted {
        processed: Range,
    },
    /// CollapsedSpace alone retains semantic U+0020. Other causes are genuine
    /// elimination; neither an empty output nor an original width decides this.
    Deleted {
        processed: usize,
        cause: DeletionCause,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum LexicalBarrierOrigin {
    CapturedEvent(usize),
    /// An executed source-local stop, without captured box/scope authority.
    SourceLocalCut(usize),
}

pub(super) struct LexicalOutput {
    pub(super) units: Vec<u16>,
    pub(super) map: Vec<MappingPiece>,
    pub(super) plaintext_tokens: Vec<PlaintextToken>,
    pub(super) lexical: LexicalState,
}

/// A fragment's exact incidence in the sole lexical inventory. The bounded
/// span may contain transparent markers; its length is not a SPACE count.
/// It is interpreted only with the processed owner retained by the same analysis.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LexicalDeletion {
    source: Range,
    cause: DeletionCause,
    first: usize,
    end: usize,
}

impl LexicalRecord {
    fn source_range(&self, context: &LexicalContext) -> Range {
        let event = match self.kind {
            LexicalRecordKind::Character { source, .. } => return source,
            LexicalRecordKind::ScopeMarker { event }
            | LexicalRecordKind::Barrier {
                origin: LexicalBarrierOrigin::CapturedEvent(event),
            } => event,
            LexicalRecordKind::Barrier {
                origin: LexicalBarrierOrigin::SourceLocalCut(at),
            } => return Range::new(at, at),
        };
        let LexicalContext::Captured(capture) = context else {
            unreachable!("source-local execution never issues captured lexical events")
        };
        let at = capture.events[event].at;
        Range::new(at, at)
    }

    /// Existing plaintext presentation consumes these executed facts too. The
    /// transient barrier flags are the lexer owner's actual admitted effects.
    pub(super) fn plaintext(
        &self,
        at: usize,
        reset: bool,
        atomic: Option<usize>,
    ) -> PlaintextToken {
        let (processed, collapsible, break_reset, transparent) = match self.kind {
            LexicalRecordKind::Character {
                mode,
                role,
                disposition,
                ..
            } => {
                let (processed, deleted) = match disposition {
                    LexicalDisposition::Emitted { processed } => (processed, false),
                    LexicalDisposition::Deleted { processed, .. } => {
                        (Range::new(processed, processed), true)
                    }
                };
                (
                    processed,
                    role == LexicalCharacterRole::Space && mode.collapses_spaces()
                        || role == LexicalCharacterRole::SegmentBreak
                            && mode == WhiteSpace::Collapse,
                    !deleted
                        && role == LexicalCharacterRole::SegmentBreak
                        && mode.preserves_breaks(),
                    deleted || role == LexicalCharacterRole::BidiControl,
                )
            }
            LexicalRecordKind::ScopeMarker { .. } => (Range::new(at, at), false, false, true),
            LexicalRecordKind::Barrier { .. } => {
                (Range::new(at, at), false, false, atomic.is_none())
            }
        };
        PlaintextToken {
            processed,
            domain: self.domain,
            collapsible,
            reset: reset || break_reset,
            transparent,
        }
    }
}

impl LexicalState {
    pub(crate) fn deletion(&self, source: Range, cause: DeletionCause) -> LexicalDeletion {
        let Self::Executed { context, records } = self else {
            unreachable!("identity processing has only Copy correspondence")
        };
        LexicalDeletion {
            source,
            cause,
            first: records
                .partition_point(|record| record.source_range(context).end <= source.start),
            end: records.partition_point(|record| record.source_range(context).start < source.end),
        }
    }
}

impl LexicalDeletion {
    /// True deleted character occurrences only, preserving multiplicity even
    /// when correspondence or provenance event partitions have different sizes.
    pub(crate) fn occurrences<'a>(
        &self,
        state: &'a LexicalState,
    ) -> impl Iterator<Item = &'a LexicalRecord> {
        self.indexed_occurrences(state).map(|(_, record)| record)
    }

    /// Allocation-local ledger indices preserve occurrence multiplicity across
    /// several relation fragments referring to the same executed character.
    pub(crate) fn indexed_occurrences<'a>(
        &self,
        state: &'a LexicalState,
    ) -> impl Iterator<Item = (usize, &'a LexicalRecord)> {
        let LexicalState::Executed { records, .. } = state else {
            unreachable!("a lexical deletion is issued only by executed processing")
        };
        let first = self.first;
        let deletion = *self;
        records[self.first..self.end]
            .iter()
            .enumerate()
            .filter_map(move |(offset, record)| {
                deletion.matches(record).then_some((first + offset, record))
            })
    }

    fn matches(&self, record: &LexicalRecord) -> bool {
        matches!(record.kind,
            LexicalRecordKind::Character {
                source: original,
                disposition: LexicalDisposition::Deleted { cause: actual, .. },
                ..
            } if actual == self.cause && original.start < self.source.end
                && self.source.start < original.end)
    }

    pub(crate) fn occurrence_cursor<'a>(
        &self,
        state: &'a LexicalState,
    ) -> LexicalOccurrenceCursor<'a> {
        let LexicalState::Executed { records, .. } = state else {
            unreachable!("a lexical deletion is issued only by executed processing")
        };
        LexicalOccurrenceCursor {
            deletion: *self,
            records: &records[self.first..self.end],
            next: 0,
        }
    }

    /// Reconstitutes the existing event fragment from its actual occurrence
    /// incidence; this does not create an emitted scalar or change its cuts.
    pub(crate) fn original_range(&self, state: &LexicalState) -> Range {
        let mut occurrences = self.occurrences(state).map(|record| {
            let LexicalRecordKind::Character { source, .. } = record.kind else {
                unreachable!("occurrences returns deleted characters")
            };
            Range::new(
                source.start.max(self.source.start),
                source.end.min(self.source.end),
            )
        });
        let first = occurrences
            .next()
            .expect("an executed Delete fragment has an occurrence");
        Range::new(
            first.start,
            occurrences.last().map_or(first.end, |last| last.end),
        )
    }
}

/// Borrowed filtered traversal whose caller admits every underlying inspection.
pub(crate) struct LexicalOccurrenceCursor<'a> {
    deletion: LexicalDeletion,
    records: &'a [LexicalRecord],
    next: usize,
}
impl<'a> LexicalOccurrenceCursor<'a> {
    pub(crate) fn next(
        &mut self,
        mut inspect: impl FnMut() -> crate::Result<()>,
    ) -> crate::Result<Option<(usize, &'a LexicalRecord)>> {
        while self.next < self.records.len() {
            inspect()?;
            let index = self.next;
            self.next += 1;
            let record = &self.records[index];
            if self.deletion.matches(record) {
                return Ok(Some((self.deletion.first + index, record)));
            }
        }
        Ok(None)
    }
}
