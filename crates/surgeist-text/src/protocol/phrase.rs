//! Authentic final-analysis phrases and non-placeable remaining-piece trials.
use super::{
    HyphenationContinuation, LineSelectionRequest, PreparedBoundary, PreparedControl,
    PreparedOrigin, PreparedText, PreparedVirtualSeparator, PresentationRange, ProtocolError,
    protocol_error,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Qualified retained phrase model identity.
pub enum PreparedPhraseProvenance {
    BudouXJapanese093,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// One final-analysis phrase with original, control and virtual custody.
pub struct PreparedPhrase {
    pub(crate) prepared: PreparedText,
    pub(crate) index: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// Actual processed presentation contribution owner.
pub enum PreparedPhrasePieceOwner {
    Original {
        origin: PreparedOrigin,
        aggregate: crate::Range,
        local: crate::Range,
    },
    Control(PreparedControl),
    Virtual(PreparedVirtualSeparator),
}
#[derive(Clone, Debug, Eq, PartialEq)]
/// A final presentation span and its captured source contribution.
pub struct PreparedPhrasePiece {
    pub presentation: PresentationRange,
    pub owner: PreparedPhrasePieceOwner,
}

impl PreparedPhrase {
    pub fn prepared_text(&self) -> PreparedText {
        self.prepared.clone()
    }
    fn component(&self) -> &crate::engine::analysis::phrase::PhraseComponent {
        let catalog = &self.prepared.inner.analyzed.analysis.phrases;
        &catalog.components[catalog.phrases[self.index].component]
    }
    pub fn provenance(&self) -> PreparedPhraseProvenance {
        match self.component().model {
            crate::phrase::PhraseModel::BudouXJapanese093 => {
                PreparedPhraseProvenance::BudouXJapanese093
            }
        }
    }
    pub fn component_presentation_range(&self) -> PresentationRange {
        let component = self.component();
        let atoms = &self.prepared.inner.analyzed.projection.atoms;
        PresentationRange {
            start: atoms[component.atoms.start].presentation.start,
            end: atoms[component.atoms.end - 1].presentation.end,
        }
    }
    /// Final UTF-16 presentation cut positions, before Appendix-H qualification.
    pub fn raw_presentation_cuts(&self) -> impl Iterator<Item = usize> + '_ {
        self.component().raw_cuts.iter().map(|&atom| {
            self.prepared.inner.analyzed.projection.atoms[atom]
                .presentation
                .start
        })
    }
    /// Final UTF-16 presentation cut positions admitted by Appendix H.
    pub fn qualified_presentation_cuts(&self) -> impl Iterator<Item = usize> + '_ {
        self.component().qualified_cuts.iter().map(|&atom| {
            self.prepared.inner.analyzed.projection.atoms[atom]
                .presentation
                .start
        })
    }
    pub fn presentation_range(&self) -> PresentationRange {
        let range = self.prepared.inner.analyzed.analysis.phrases.phrases[self.index].presentation;
        PresentationRange {
            start: range.start,
            end: range.end,
        }
    }
    pub fn pieces(&self) -> Vec<PreparedPhrasePiece> {
        use crate::transform::relation::ComponentOwner;
        let analyzed = &self.prepared.inner.analyzed;
        let phrase = &analyzed.analysis.phrases.phrases[self.index];
        let mut pieces = Vec::new();
        for atom in phrase.atoms.clone() {
            let range = analyzed.projection.atoms[atom].presentation;
            let presentation = PresentationRange {
                start: range.start,
                end: range.end,
            };
            match analyzed.transform.component_for_atom(atom).owner {
                ComponentOwner::Original(row) => {
                    for piece in analyzed.transform.original_pieces(row) {
                        pieces.push(PreparedPhrasePiece {
                            presentation,
                            owner: PreparedPhrasePieceOwner::Original {
                                origin: PreparedOrigin {
                                    prepared: Arc::clone(&self.prepared.inner),
                                    index: piece.origin,
                                },
                                aggregate: piece.aggregate,
                                local: piece.local,
                            },
                        });
                    }
                }
                ComponentOwner::Wbr(occurrence) => pieces.push(PreparedPhrasePiece {
                    presentation,
                    owner: PreparedPhrasePieceOwner::Control(PreparedControl {
                        prepared: Arc::clone(&self.prepared.inner),
                        index: self.prepared.inner.tables.control_rows[occurrence],
                    }),
                }),
                ComponentOwner::Virtual(index) => pieces.push(PreparedPhrasePiece {
                    presentation,
                    owner: PreparedPhrasePieceOwner::Virtual(PreparedVirtualSeparator {
                        prepared: Arc::clone(&self.prepared.inner),
                        index,
                    }),
                }),
            }
        }
        pieces
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PreparedPhrasePortion {
    pub(crate) phrase: PreparedPhrase,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) incoming: Option<HyphenationContinuation>,
    pub(crate) query: super::PreparedConditionalSearch,
}
impl PreparedPhrasePortion {
    pub fn phrase(&self) -> PreparedPhrase {
        self.phrase.clone()
    }
    pub fn start(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.phrase.prepared.inner),
            index: self.start,
        }
    }
    pub fn end(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.phrase.prepared.inner),
            index: self.end,
        }
    }
    pub fn incoming(&self) -> Option<HyphenationContinuation> {
        self.incoming.clone()
    }
    pub(super) fn validate(
        &self,
        prepared: &PreparedText,
        request: &LineSelectionRequest,
    ) -> crate::Result<()> {
        if self.phrase.prepared != *prepared
            || request.start != self.start()
            || request.end != self.end()
            || request.incoming != self.incoming
            || request.alternative.is_some()
            || request.context.available_inline.is_none()
        {
            return Err(protocol_error(ProtocolError::InvalidPhrasePortion));
        }
        Ok(())
    }
}
impl PreparedText {
    pub fn phrases(&self) -> impl ExactSizeIterator<Item = PreparedPhrase> + '_ {
        (0..self.inner.analyzed.analysis.phrases.phrases.len()).map(|index| PreparedPhrase {
            prepared: self.clone(),
            index,
        })
    }
}
