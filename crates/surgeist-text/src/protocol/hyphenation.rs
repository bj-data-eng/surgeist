//! Preparation-owned lexical words, resource observations and genuine cuts.
use std::{ops::Range as IndexRange, sync::Arc};

use super::{
    PreparedBoundary, PreparedHyphenationCut, PreparedOpportunity, PreparedOrigin, PreparedText,
    PreparedWord, paragraph::OriginIntersection,
};
use crate::engine::analysis::hyphenation::{CutCounts, en_us};
use crate::{
    Range,
    engine::{AnalyzedData, PreparedData},
};

/// The actual resource qualification of a complete processed lexical word.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HyphenationResource {
    /// Exact en-US, compared ASCII-insensitively, over every contribution.
    EnUs2005,
    /// At least one genuine text contribution has another or no locale.
    UnavailableLanguage,
    /// An isolated UTF-16 unit has no scalar spelling for this UTF-8 resource.
    NonScalarSource,
}

/// Provenance of an observed resource or genuinely authored cut.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HyphenationProvenance {
    Source,
    Control,
    Pattern,
    SourceException { entry: &'static str },
    TextCorrection { entry: &'static str },
}
impl From<en_us::Provenance> for HyphenationProvenance {
    fn from(value: en_us::Provenance) -> Self {
        match value {
            en_us::Provenance::Pattern => Self::Pattern,
            en_us::Provenance::SourceException { entry } => Self::SourceException { entry },
            en_us::Provenance::TextCorrection { entry } => Self::TextCorrection { entry },
        }
    }
}

/// CSS counting-unit contributions at one authenticated complete-word cut.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HyphenationCounts {
    pub total: usize,
    pub before: usize,
    pub after: usize,
}
impl From<CutCounts> for HyphenationCounts {
    fn from(value: CutCounts) -> Self {
        Self {
            total: value.total,
            before: value.before,
            after: value.after,
        }
    }
}

#[derive(Debug)]
pub(crate) struct WordData {
    pub(crate) lexical: usize,
    pub(crate) resource: HyphenationResource,
    pub(crate) query: String,
    /// Exact query byte endpoints, including both sides of omitted SHY atoms.
    pub(crate) endpoints: Vec<(usize, usize)>,
    pub(crate) proposals: Vec<(usize, HyphenationProvenance)>,
    pub(crate) provenance: Option<HyphenationProvenance>,
}
#[derive(Debug)]
pub(crate) struct CutData {
    pub(crate) word: Option<usize>,
    pub(crate) boundary: usize,
    pub(crate) provenance: HyphenationProvenance,
    pub(crate) style: usize,
    pub(crate) preceding: Option<IndexRange<usize>>,
    pub(crate) counts: Option<CutCounts>,
    pub(crate) opportunity: Option<usize>,
}

pub(crate) fn words(analyzed: &AnalyzedData) -> crate::Result<Vec<WordData>> {
    analyzed
        .words
        .rows()
        .iter()
        .enumerate()
        .map(|(lexical, word)| {
            let query_length = word
                .atoms
                .clone()
                .try_fold(0usize, |length, atom| {
                    let cp = analyzed.projection.atoms[atom].code_point;
                    length.checked_add(if cp == 0xad {
                        0
                    } else {
                        char::from_u32(cp).map_or(0, char::len_utf8)
                    })
                })
                .ok_or_else(|| {
                    crate::hyphenation::error(crate::HyphenationError::ResourceLengthOverflow)
                })?;
            let mut query = String::new();
            query
                .try_reserve_exact(query_length)
                .map_err(|_| crate::hyphenation::error(crate::HyphenationError::ResourceStorage))?;
            let mut endpoints = vec![(0, word.atoms.start)];
            let mut resource = HyphenationResource::EnUs2005;
            for atom in word.atoms.clone() {
                let value = &analyzed.projection.atoms[atom];
                let style = analyzed.analysis.infos[atom].1;
                if !analyzed.styles[style]
                    .style
                    .authored()
                    .locale
                    .as_deref()
                    .is_some_and(|locale| locale.eq_ignore_ascii_case("en-US"))
                    && resource != HyphenationResource::NonScalarSource
                {
                    resource = HyphenationResource::UnavailableLanguage;
                }
                if value.code_point != 0xad {
                    if let Some(ch) = char::from_u32(value.code_point) {
                        query.push(ch);
                    } else {
                        resource = HyphenationResource::NonScalarSource;
                    }
                }
                endpoints.push((query.len(), atom + 1));
            }
            let mut provenance = None;
            let proposals = if resource == HyphenationResource::EnUs2005 {
                {
                    let observed = en_us::observe(&query).map_err(|error| {
                        crate::hyphenation::error(match error {
                            en_us::ObservationError::LengthOverflow => {
                                crate::HyphenationError::ResourceLengthOverflow
                            }
                            en_us::ObservationError::Storage(_) => {
                                crate::HyphenationError::ResourceStorage
                            }
                            en_us::ObservationError::InvalidProviderSlice { .. }
                            | en_us::ObservationError::IncompleteProvider { .. }
                            | en_us::ObservationError::InvalidCut { .. } => {
                                crate::HyphenationError::InvalidResourceMapping
                            }
                        })
                    })?;
                    if observed.word() != query
                        || observed.cuts().iter().any(|cut| {
                            cut.before().len() != cut.byte()
                                || query.get(cut.byte()..) != Some(cut.after())
                        })
                    {
                        return Err(crate::hyphenation::error(
                            crate::HyphenationError::InvalidResourceMapping,
                        ));
                    }
                    provenance = Some(observed.provenance().into());
                    observed
                        .cuts()
                        .iter()
                        .map(|cut| (cut.byte(), observed.provenance().into()))
                        .collect()
                }
            } else {
                Vec::new()
            };
            Ok(WordData {
                lexical,
                resource,
                query,
                endpoints,
                proposals,
                provenance,
            })
        })
        .collect()
}

/// Borrowed exact source custody; it is not an enclosing word range.
#[derive(Clone)]
pub struct HyphenationSourcePieceView<'a> {
    prepared: &'a Arc<PreparedData>,
    row: OriginIntersection,
}
impl HyphenationSourcePieceView<'_> {
    #[must_use]
    pub fn origin(&self) -> PreparedOrigin {
        PreparedOrigin {
            prepared: Arc::clone(self.prepared),
            index: self.row.origin,
        }
    }
    #[must_use]
    pub fn source(&self) -> Range {
        self.row.aggregate
    }
    #[must_use]
    pub fn local(&self) -> Range {
        self.row.local
    }
    #[must_use]
    pub fn processed_range(&self) -> crate::ProcessedRange {
        self.prepared
            .analyzed
            .processed
            .range(self.row.processed.start, self.row.processed.end)
            .expect("issued processed correspondence")
    }
}
fn pieces(
    prepared: &Arc<PreparedData>,
    atoms: IndexRange<usize>,
) -> impl Iterator<Item = HyphenationSourcePieceView<'_>> {
    pieces_for_rows(
        prepared,
        atoms
            .filter_map(|atom| prepared.analyzed.transform.original_row_for_atom(atom))
            .collect(),
    )
}

fn pieces_for_rows(
    prepared: &Arc<PreparedData>,
    mut rows: Vec<usize>,
) -> impl Iterator<Item = HyphenationSourcePieceView<'_>> {
    // Source observation follows actual input occurrence order. Several emitted
    // components can share one row, including components in different EGCs.
    rows.sort_unstable();
    rows.dedup();
    rows.into_iter().flat_map(move |row| {
        prepared
            .analyzed
            .paragraph
            .origin_intersections(
                &prepared.analyzed.processed,
                prepared.analyzed.transform.rows[row].source,
            )
            .filter(|row| !row.aggregate.is_empty())
            .map(|row| HyphenationSourcePieceView { prepared, row })
    })
}

/// An observation of an executed lexical word and resource query, not an input.
pub struct PreparedWordView<'a> {
    prepared: &'a Arc<PreparedData>,
    index: usize,
}
impl PreparedWordView<'_> {
    #[must_use]
    pub fn handle(&self) -> PreparedWord {
        PreparedWord {
            prepared: Arc::clone(self.prepared),
            index: self.index,
        }
    }
    #[must_use]
    pub fn resource(&self) -> HyphenationResource {
        self.prepared.tables.words[self.index].resource
    }
    /// Exact temporary scalar spelling: authentic SHY alone is omitted.
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        let row = &self.prepared.tables.words[self.index];
        (row.resource != HyphenationResource::NonScalarSource).then_some(row.query.as_str())
    }
    #[must_use]
    pub fn provenance(&self) -> Option<HyphenationProvenance> {
        self.prepared.tables.words[self.index].provenance
    }
    pub fn source_pieces(&self) -> impl Iterator<Item = HyphenationSourcePieceView<'_>> {
        let row = &self.prepared.tables.words[self.index];
        let atoms = &self.prepared.analyzed.words.rows()[row.lexical].atoms;
        let map = &self.prepared.analyzed.transform;
        let start = map.emission_for_atom(atoms.start).event;
        let mut end = map.emission_for_atom(atoms.end - 1).event + 1;
        // Only consecutive transform removals in this genuine terminal seam
        // belong here; an independent anchor/participant stops the extent.
        while matches!(
            map.events().get(end),
            Some(crate::transform::relation::RelationEvent::Removal { .. })
        ) {
            end += 1;
        }
        let rows = map.events()[start..end]
            .iter()
            .filter_map(|event| match *event {
                crate::transform::relation::RelationEvent::Text { atom } => {
                    map.original_row_for_atom(atom)
                }
                crate::transform::relation::RelationEvent::Removal { row } => Some(row),
                _ => None,
            })
            .collect();
        pieces_for_rows(self.prepared, rows)
    }
    pub fn cuts(&self) -> impl Iterator<Item = PreparedHyphenationCutView<'_>> {
        self.prepared
            .tables
            .hyphenation_cuts
            .iter()
            .enumerate()
            .filter(move |(_, row)| row.word == Some(self.index))
            .map(|(index, _)| PreparedHyphenationCutView {
                prepared: self.prepared,
                index,
            })
    }
}
impl PreparedWord {
    #[must_use]
    pub fn view(&self) -> PreparedWordView<'_> {
        PreparedWordView {
            prepared: &self.prepared,
            index: self.index,
        }
    }
}

/// One genuine mixed cut; word/count absence remains observable for raw controls.
pub struct PreparedHyphenationCutView<'a> {
    prepared: &'a Arc<PreparedData>,
    index: usize,
}
impl PreparedHyphenationCutView<'_> {
    fn data(&self) -> &CutData {
        &self.prepared.tables.hyphenation_cuts[self.index]
    }
    #[must_use]
    pub fn handle(&self) -> PreparedHyphenationCut {
        PreparedHyphenationCut {
            prepared: Arc::clone(self.prepared),
            index: self.index,
        }
    }
    #[must_use]
    pub fn word(&self) -> Option<PreparedWord> {
        self.data().word.map(|index| PreparedWord {
            prepared: Arc::clone(self.prepared),
            index,
        })
    }
    #[must_use]
    pub fn boundary(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(self.prepared),
            index: self.data().boundary,
        }
    }
    #[must_use]
    pub fn provenance(&self) -> HyphenationProvenance {
        self.data().provenance
    }
    #[must_use]
    pub fn counts(&self) -> Option<HyphenationCounts> {
        self.data().counts.map(Into::into)
    }
    #[must_use]
    pub fn style(&self) -> &crate::Style {
        self.prepared.analyzed.styles[self.data().style]
            .style
            .authored()
    }
    #[must_use]
    pub fn opportunity(&self) -> Option<PreparedOpportunity> {
        self.data().opportunity.map(|index| PreparedOpportunity {
            prepared: Arc::clone(self.prepared),
            index,
        })
    }
    /// The genuine preceding EGC, distinct from the insertion's empty source span.
    pub fn preceding_source_pieces(&self) -> impl Iterator<Item = HyphenationSourcePieceView<'_>> {
        self.data()
            .preceding
            .clone()
            .into_iter()
            .flat_map(|atoms| pieces(self.prepared, atoms))
    }
}
impl PreparedHyphenationCut {
    #[must_use]
    pub fn view(&self) -> PreparedHyphenationCutView<'_> {
        PreparedHyphenationCutView {
            prepared: &self.prepared,
            index: self.index,
        }
    }
}
impl PreparedText {
    pub fn word_views(&self) -> impl ExactSizeIterator<Item = PreparedWordView<'_>> {
        (0..self.inner.tables.words.len()).map(|index| PreparedWordView {
            prepared: &self.inner,
            index,
        })
    }
    pub fn hyphenation_cut_views(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedHyphenationCutView<'_>> {
        (0..self.inner.tables.hyphenation_cuts.len()).map(|index| PreparedHyphenationCutView {
            prepared: &self.inner,
            index,
        })
    }
}

impl std::fmt::Debug for HyphenationSourcePieceView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyphenationSourcePieceView")
            .field("source", &self.row.aggregate)
            .field("origin", &self.row.origin)
            .finish()
    }
}
impl std::fmt::Debug for PreparedWordView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedWordView")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for PreparedHyphenationCutView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedHyphenationCutView")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}

/// An exact word portion for measurement, not an ordinary break or placement.
/// Equality retains actual word, mixed endpoints and selected manual variant.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PreparedWordPortion {
    pub(crate) word: PreparedWord,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) alternative: Option<usize>,
    pub(crate) incoming: Option<usize>,
    pub(crate) restored: bool,
}
impl PreparedWordPortion {
    #[must_use]
    pub fn word(&self) -> PreparedWord {
        self.word.clone()
    }
    #[must_use]
    pub fn start(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.word.prepared),
            index: self.start,
        }
    }
    #[must_use]
    pub fn end(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.word.prepared),
            index: self.end,
        }
    }
    #[must_use]
    pub fn incoming(&self) -> Option<super::HyphenationContinuation> {
        super::HyphenationContinuation::from_alternative(super::PreparedAlternative {
            prepared: Arc::clone(&self.word.prepared),
            index: self.incoming?,
        })
    }
    #[must_use]
    pub fn alternative(&self) -> Option<super::PreparedAlternative> {
        self.alternative.map(|index| super::PreparedAlternative {
            prepared: Arc::clone(&self.word.prepared),
            index,
        })
    }
    /// Whether an admitted genuine manual cut partitions this word. This is
    /// source/selection authority, never a claim that a portion fits a frame.
    #[must_use]
    pub fn has_manual_partition(&self) -> bool {
        self.word
            .prepared
            .tables
            .hyphenation_cuts
            .iter()
            .enumerate()
            .any(|(cut, row)| {
                row.word == Some(self.word.index)
                    && manual_alternatives(&self.word.prepared.tables, cut, self.restored)
                        .next()
                        .is_some()
            })
    }
    pub(super) fn validate(
        &self,
        prepared: &PreparedText,
        request: &super::LineSelectionRequest,
    ) -> crate::Result<()> {
        if !Arc::ptr_eq(&prepared.inner, &self.word.prepared) {
            return Err(super::protocol_error(
                super::ProtocolError::ForeignPreparation {
                    kind: super::ProtocolHandleKind::WordPortion,
                    occurrence: self.word.index,
                },
            ));
        }
        if request.start.index != self.start
            || request.end.index != self.end
            || request.incoming != self.incoming()
            || request.alternative.as_ref().map(|row| row.index) != self.alternative
            || !self
                .word
                .view()
                .portions_with_restoration(self.restored)?
                .contains(self)
        {
            return Err(super::protocol_error(
                super::ProtocolError::InvalidWordPortion,
            ));
        }
        if let Some(alternative) = &request.alternative
            && !Arc::ptr_eq(&prepared.inner, &alternative.prepared)
        {
            return Err(super::protocol_error(
                super::ProtocolError::ForeignPreparation {
                    kind: super::ProtocolHandleKind::Alternative,
                    occurrence: alternative.index,
                },
            ));
        }
        Ok(())
    }
}
impl PreparedWordView<'_> {
    /// Exact consecutive admitted manual portions; the final portion is the
    /// unbroken residual word end. No ordinary end permission is synthesized.
    pub fn portions(&self) -> crate::Result<Vec<PreparedWordPortion>> {
        self.portions_with_restoration(false)
    }
    fn portions_with_restoration(&self, restored: bool) -> crate::Result<Vec<PreparedWordPortion>> {
        let tables = &self.prepared.tables;
        let word = &self.prepared.analyzed.words.rows()[tables.words[self.index].lexical];
        let first = tables.participants.iter().position(|part| matches!(*part, super::prepared::ParticipantData::Text(i) if tables.units[i].atoms.contains(&word.atoms.start)))
            .ok_or_else(|| super::protocol_error(super::ProtocolError::InvalidWordPortion))?;
        let last = tables.participants.iter().rposition(|part| matches!(*part, super::prepared::ParticipantData::Text(i) if tables.units[i].atoms.contains(&(word.atoms.end - 1))))
            .ok_or_else(|| super::protocol_error(super::ProtocolError::InvalidWordPortion))? + 1;
        let mut start = tables
            .boundaries
            .iter()
            .position(|row| row.participant == first)
            .ok_or_else(|| super::protocol_error(super::ProtocolError::InvalidWordPortion))?;
        let end = tables
            .boundaries
            .iter()
            .position(|row| row.participant == last)
            .ok_or_else(|| super::protocol_error(super::ProtocolError::InvalidWordPortion))?;
        let mut result = Vec::new();
        let mut incoming = vec![None];
        for (cut_index, cut) in tables.hyphenation_cuts.iter().enumerate() {
            if cut.word != Some(self.index) {
                continue;
            }
            if cut.boundary <= start || cut.boundary > end {
                continue;
            }
            let alternatives: Vec<_> = manual_alternatives(tables, cut_index, restored).collect();
            if alternatives.is_empty() {
                continue;
            }
            for &alternative in &alternatives {
                for &prior in &incoming {
                    result.push(PreparedWordPortion {
                        word: self.handle(),
                        start,
                        end: cut.boundary,
                        alternative: Some(alternative),
                        incoming: prior,
                        restored,
                    });
                }
            }
            incoming = alternatives
                .iter()
                .map(|&alternative| {
                    let handle = super::PreparedAlternative {
                        prepared: Arc::clone(self.prepared),
                        index: alternative,
                    };
                    super::HyphenationContinuation::from_alternative(handle).map(|_| alternative)
                })
                .collect();
            incoming.dedup();
            start = cut.boundary;
        }
        if start < end {
            for prior in incoming {
                result.push(PreparedWordPortion {
                    word: self.handle(),
                    start,
                    end,
                    alternative: None,
                    incoming: prior,
                    restored,
                });
            }
        }
        Ok(result)
    }

    /// Actual `Allowed`/no-alternative fallbacks immediately before this word,
    /// in mixed custody order. No fit decision or synthetic empty offer is made.
    /// `start` is the unchanged incoming line start in this preparation.
    pub fn ordinary_fallbacks(
        &self,
        start: &PreparedBoundary,
    ) -> crate::Result<Vec<PreparedOpportunity>> {
        if !Arc::ptr_eq(self.prepared, &start.prepared) {
            return Err(super::protocol_error(
                super::ProtocolError::ForeignPreparation {
                    kind: super::ProtocolHandleKind::Boundary,
                    occurrence: start.index,
                },
            ));
        }
        Ok((0..self.prepared.tables.opportunities.len())
            .filter(|&index| ordinary_fallback(self.prepared, self.index, start.index, index))
            .map(|index| PreparedOpportunity {
                prepared: Arc::clone(self.prepared),
                index,
            })
            .collect())
    }
}

pub(super) fn restored_portions(
    prepared: &PreparedText,
    word: usize,
) -> crate::Result<Vec<PreparedWordPortion>> {
    PreparedWord {
        prepared: Arc::clone(&prepared.inner),
        index: word,
    }
    .view()
    .portions_with_restoration(true)
}
fn manual_alternatives(
    tables: &super::prepared::PreparedTables,
    cut: usize,
    restored: bool,
) -> impl Iterator<Item = usize> + '_ {
    tables
        .alternatives
        .iter()
        .enumerate()
        .filter_map(move |(index, row)| {
            (row.hyphenation_cut == cut
                && (restored || row.opportunity.is_some())
                && !matches!(
                    row.definition,
                    super::prepared::AlternativeDefinition::Automatic { .. }
                ))
            .then_some(index)
        })
}

/// Transparent mixed cuts keep distinct custody; in-flow atomic and control influences are not text aliases.
fn ordinary_fallback(
    prepared: &Arc<PreparedData>,
    word: usize,
    start: usize,
    offer: usize,
) -> bool {
    use super::prepared::ParticipantData;
    use crate::engine::analysis::wrapping::Influence;
    let tables = &prepared.tables;
    let Some(row) = tables.opportunities.get(offer) else {
        return false;
    };
    if row.kind != super::PreparedBreakKind::Allowed {
        return false;
    }
    let boundary = tables.boundaries[row.boundary];
    let from = tables.boundaries[start].participant;
    if boundary.participant <= from {
        return false;
    }
    if tables.opportunities.iter().any(|row| {
        row.kind == super::PreparedBreakKind::Forced
            && tables.boundaries[row.boundary].participant > from
            && tables.boundaries[row.boundary].participant <= boundary.participant
    }) {
        return false;
    }
    let lexical = &prepared.analyzed.words.rows()[tables.words[word].lexical];
    let first=tables.participants.iter().position(|part| matches!(part,ParticipantData::Text(i) if tables.units[*i].atoms.contains(&lexical.atoms.start)));
    let relation =
        &prepared.analyzed.logical.relations[prepared.analyzed.logical.cuts[boundary.participant]];
    matches!(relation.after,Some(Influence::Text { atom, participant }) if atom==lexical.atoms.start && Some(participant)==first)
}
