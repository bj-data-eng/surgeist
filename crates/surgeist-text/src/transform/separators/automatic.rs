//! Native Japanese phrase proposals and their authentic scoped insertion seams.
use super::super::context::{SourceContext, StyleRow};
use super::super::relation::{ScopeData, TransformMap, VirtualSeparatorData};
use crate::engine::analysis::wrapping::{Event, LogicalEvent};
use crate::phrase::qualification::{Punctuation, punctuation, separator};
use crate::phrase::{PhraseError, PhraseModel};
use crate::processing::contextual::ParagraphTopology;
use crate::projection::SourceProjection;
use crate::protocol::paragraph::SemanticAnchorItem;
use crate::{
    Error, ErrorCode, ErrorDetail, InlineBoxKind, Result, SourceProcessingEvent, TextTransformError,
};
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup, LineBreak},
};

pub(crate) struct Proposal {
    pub(in crate::transform) occurrence: usize,
    pub(in crate::transform) code_point: u32,
    pub(in crate::transform) style: usize,
}

#[derive(Clone, Copy)]
enum LanguageRead {
    Unread,
    Resolved(Option<fontique::Language>),
}

#[derive(Clone, Copy)]
enum RunRead {
    Unread,
    Unsupported,
    Qualified(usize),
}

/// Genuine adjacent original participants and their actual mixed occurrences.
#[derive(Clone, Copy)]
struct IncidentPair {
    left: usize,
    right: usize,
    left_event: usize,
    right_event: usize,
}

#[derive(Clone, Copy)]
struct Insertion {
    scope: usize,
    source: usize,
    capture_cut: usize,
}

#[derive(Clone, Copy)]
enum Site {
    Neutral,
    Unqualified(IncidentPair),
    Captured {
        pair: IncidentPair,
        insertion: Insertion,
    },
}

pub(in crate::transform) struct PhraseReads {
    languages: Vec<LanguageRead>,
    sites: Vec<Site>,
    stops: Vec<usize>,
    controls_before: Vec<usize>,
    runs: Vec<RunRead>,
}
impl PhraseReads {
    /// A source-selection run may demand this stage only when its authentic
    /// potential site is not proved to belong to another effecting scope.
    /// Neutral sites retain ready absence; unqualified sites keep the normal
    /// material topology refusal instead of being cached absent.
    pub(in crate::transform) fn may_stage_scope(&self, cut: usize, scope: usize) -> bool {
        match self.sites[cut] {
            Site::Captured { insertion, .. } => insertion.scope == scope,
            Site::Neutral | Site::Unqualified(_) => true,
        }
    }
    pub(in crate::transform) fn new(
        styles: &[StyleRow<'_>],
        input: &SourceProjection,
        paragraph: &SourceContext<'_>,
        map: &TransformMap,
        events: &[LogicalEvent],
        seams: &[super::ConnectionSeam],
    ) -> Result<Self> {
        let length = events.len().checked_add(1).ok_or_else(super::capacity)?;
        let mut sites = super::super::context::filled(length, Site::Neutral)?;
        let mut text_events = super::super::context::filled(input.atoms.len(), 0)?;
        let mut controls_before = Vec::new();
        let mut controls = 0;
        for (index, event) in events.iter().enumerate() {
            super::super::context::push(&mut controls_before, controls)?;
            match event.kind {
                Event::Text(row) => text_events[row] = index,
                Event::Anchor(anchor) if matches!(anchor.item, SemanticAnchorItem::Control(_)) => {
                    controls += 1
                }
                _ => (),
            }
        }
        super::super::context::push(&mut controls_before, controls)?;
        let mut stops = super::super::context::collect_rows(input.barrier_atoms.iter().copied())?;
        super::super::context::push(&mut stops, 0)?;
        super::super::context::push(&mut stops, input.atoms.len())?;
        for (row, atom) in input.atoms.iter().enumerate() {
            if crate::engine::analysis::is_hard_separator(atom.code_point) {
                super::super::context::push(&mut stops, row)?;
                super::super::context::push(&mut stops, row + 1)?;
            }
        }
        for seam in seams {
            super::super::context::push(&mut stops, seam.atom)?;
        }
        stops.sort_unstable();
        stops.dedup();
        for right in 1..input.atoms.len() {
            let left = right - 1;
            if ![left, right].iter().any(|&row| {
                styles[map.rows[row].style]
                    .style
                    .authored()
                    .word_space_transform
                    .detects_phrases()
            }) || stops.binary_search(&right).is_ok()
            {
                continue;
            }
            let pair = IncidentPair {
                left,
                right,
                left_event: text_events[left],
                right_event: text_events[right],
            };
            if events[pair.left_event + 1..pair.right_event]
                .iter()
                .any(|event| {
                    super::forced(event, input, paragraph.processed.source(), paragraph)
                        == Some(true)
                        || matches!(event.kind, Event::Anchor(anchor) if matches!(anchor.item,
                    SemanticAnchorItem::Atomic(index)
                    if paragraph.processed.source().boxes()[index].kind() == InlineBoxKind::InFlow))
                })
            {
                continue;
            }
            if let Some(topology) = paragraph
                .scope_topology
                .filter(|topology| topology.capture.complete[topology.domain])
            {
                let (scope, cut, source, capture_cut) =
                    custody(topology, paragraph, input, map, events, pair);
                sites[cut] = Site::Captured {
                    pair,
                    insertion: Insertion {
                        scope,
                        source,
                        capture_cut,
                    },
                };
            } else {
                // No side/owner is invented. The existing material proposal
                // path still qualifies topology after actual detector demand.
                for site in &mut sites[pair.left_event + 1..=pair.right_event] {
                    *site = Site::Unqualified(pair);
                }
            }
        }
        Ok(Self {
            languages: super::super::context::filled(styles.len(), LanguageRead::Unread)?,
            sites,
            stops,
            controls_before,
            runs: super::super::context::filled(input.atoms.len(), RunRead::Unread)?,
        })
    }

    /// This qualifies root output demand only. A foreign site remains readable
    /// when a genuine casing/unit predicate later needs its context.
    pub(in crate::transform) fn root_gap(&self, paragraph: &SourceContext<'_>, cut: usize) -> bool {
        match self.sites[cut] {
            Site::Captured { insertion, .. } => paragraph.owns_capture_cut(insertion.capture_cut),
            Site::Neutral | Site::Unqualified(_) => true,
        }
    }
    fn language(
        &mut self,
        styles: &[StyleRow<'_>],
        index: usize,
    ) -> Result<Option<fontique::Language>> {
        if let LanguageRead::Resolved(language) = self.languages[index] {
            return Ok(language);
        }
        let language = styles[index].style.phrase_language()?;
        self.languages[index] = LanguageRead::Resolved(language);
        Ok(language)
    }

    fn supported(
        &mut self,
        styles: &[StyleRow<'_>],
        map: &TransformMap,
        row: usize,
    ) -> Result<bool> {
        match self.runs[row] {
            RunRead::Unsupported => Ok(false),
            RunRead::Qualified(_) => Ok(true),
            RunRead::Unread => {
                let supported =
                    PhraseModel::for_language(self.language(styles, map.rows[row].style)?)
                        .is_some();
                if !supported {
                    self.runs[row] = RunRead::Unsupported;
                }
                Ok(supported)
            }
        }
    }

    /// Discovery and detector execution belong to the first genuine demand.
    /// Only a fully qualified actual run publishes indexed membership.
    fn run(
        &mut self,
        input: &SourceProjection,
        styles: &[StyleRow<'_>],
        paragraph: &SourceContext<'_>,
        map: &mut TransformMap,
        pair: IncidentPair,
    ) -> Result<Option<usize>> {
        for row in [pair.left, pair.right] {
            if !self.supported(styles, map, row)? {
                return Ok(None);
            }
        }
        if let RunRead::Qualified(receipt) = self.runs[pair.left] {
            return Ok(Some(receipt));
        }
        if let RunRead::Qualified(receipt) = self.runs[pair.right] {
            return Ok(Some(receipt));
        }
        let lower = self.stops.partition_point(|&at| at <= pair.left);
        let hard_start = self.stops[lower - 1];
        let upper = self.stops.partition_point(|&at| at <= pair.right);
        let hard_end = self.stops.get(upper).copied().unwrap_or(input.atoms.len());
        let mut start = pair.left;
        while start > hard_start && self.supported(styles, map, start - 1)? {
            start -= 1;
        }
        let mut end = pair.right + 1;
        while end < hard_end && self.supported(styles, map, end)? {
            end += 1;
        }
        if let Some(edge) = paragraph.missing.iter().find(|edge| {
            input.atoms[start].source.start < edge.at && edge.at < input.atoms[end - 1].source.end
        }) {
            return Err(edge.unavailable(crate::SourceContinuityOperation::Transformation));
        }
        let language = self
            .language(styles, map.rows[start].style)?
            .expect("supported source language");
        let model = PhraseModel::for_language(Some(language)).expect("supported actual model");
        let mut analysis = PhraseAnalysis {
            rows: start..end,
            model,
            language,
            input: super::super::context::collect_rows(
                input.atoms[start..end].iter().map(|atom| atom.code_point),
            )?,
            coverage: super::super::context::collect_rows(
                input.atoms[start..end].iter().map(|atom| atom.source),
            )?,
            cuts: Vec::new(),
            qualified: Vec::new(),
        };
        analysis.complete(input)?;
        let receipt = map.phrases.len();
        map.phrases.try_reserve(1).map_err(|_| super::capacity())?;
        map.phrases.push(std::sync::Arc::new(analysis));
        for row in &mut self.runs[start..end] {
            *row = RunRead::Qualified(receipt);
        }
        Ok(Some(receipt))
    }
}

/// The retained pre-replacement semantic analysis belongs to this exact map,
/// whose original rows and owning AnalyzedSource retain its input identity.
#[derive(Clone, Debug)]
pub(crate) struct PhraseAnalysis {
    /// Canonical pre-replacement run coordinates, never output-local row IDs.
    rows: std::ops::Range<usize>,
    model: PhraseModel,
    language: fontique::Language,
    input: Vec<u32>,
    coverage: Vec<crate::Range>,
    cuts: Vec<usize>,
    qualified: Vec<bool>,
}

impl PhraseAnalysis {
    /// Actual canonical original-row interval of this completed native run.
    /// Local maps retain the receipt without relabeling these coordinates.
    pub(in crate::transform) fn canonical_rows(&self) -> std::ops::Range<usize> {
        self.rows.clone()
    }
    fn complete(&mut self, projection: &SourceProjection) -> Result<()> {
        // Execute the retained actual provider/language pair, then qualify its
        // native cuts. No scope style or topology for other sites is executed.
        self.cuts = match (self.model, self.language.language()) {
            (PhraseModel::BudouXJapanese093, "ja") => self.model.boundaries(&self.input),
            _ => Err(PhraseError::InvalidData),
        }
        .map_err(|error| match error {
            PhraseError::Capacity => super::capacity(),
            PhraseError::InvalidData => {
                Error::new(ErrorCode::InvalidSourceProcessing, "invalid phrase model").with_detail(
                    ErrorDetail::TextTransform {
                        reason: TextTransformError::InvalidPhraseData,
                    },
                )
            }
        })?;
        let length = self
            .input
            .len()
            .checked_add(1)
            .ok_or_else(super::capacity)?;
        let mut qualified = super::super::context::filled(length, false)?;
        for &cut in &self.cuts {
            qualified[cut] = eligible(projection, &self.input, self.rows.start, cut);
        }
        let categories = CodePointMapData::<GeneralCategory>::new();
        let mut previous = 0;
        for end in self
            .cuts
            .iter()
            .copied()
            .chain(std::iter::once(self.input.len()))
        {
            if self.input[previous..end]
                .iter()
                .all(|&cp| GeneralCategoryGroup::Punctuation.contains(categories.get32(cp)))
            {
                qualified[previous] = false;
                qualified[end] = false;
            }
            previous = end;
        }
        self.qualified = qualified;
        Ok(())
    }
}

fn refusal(reason: TextTransformError) -> Error {
    Error::new(
        ErrorCode::UnsupportedFeature,
        "automatic phrase separator lacks captured context",
    )
    .with_detail(ErrorDetail::TextTransform { reason })
}

fn eligible(input: &SourceProjection, values: &[u32], start: usize, cut: usize) -> bool {
    let left = values[cut - 1];
    let right = values[cut];
    if separator(left) || separator(right) {
        return false;
    }
    let classes = CodePointMapData::<LineBreak>::new();
    if [left, right].iter().any(|&cp| {
        matches!(
            classes.get32(cp),
            LineBreak::Glue | LineBreak::WordJoiner | LineBreak::ZWJ
        )
    }) {
        return false;
    }
    let position = input.atoms[start + cut].presentation.start;
    if input.graphemes.binary_search(&position).is_err() {
        return false;
    }
    !matches!(
        punctuation(values, cut - 1),
        Punctuation::Opening | Punctuation::Unfamiliar
    ) && !matches!(
        punctuation(values, cut),
        Punctuation::Closing | Punctuation::Unfamiliar
    )
}

fn depth(topology: &ParagraphTopology, mut scope: usize) -> usize {
    let mut depth = 0;
    while let Some(parent) = topology.capture.scopes[scope].parent {
        if topology.capture.scopes[parent].domain != topology.domain {
            break;
        }
        scope = parent;
        depth += 1;
    }
    depth
}

/// Returns the outermost owner and the earliest actual parent-domain seam.
fn custody(
    topology: &ParagraphTopology,
    paragraph: &SourceContext,
    input: &SourceProjection,
    map: &TransformMap,
    events: &[LogicalEvent],
    pair: IncidentPair,
) -> (usize, usize, usize, usize) {
    let left = map
        .original_pieces(pair.left)
        .last()
        .and_then(|piece| piece.origin.captured_event())
        .expect("actual captured left contribution");
    let right = map
        .original_pieces(pair.right)
        .first()
        .and_then(|piece| piece.origin.captured_event())
        .expect("actual captured right contribution");
    let mut owner = topology.capture.events[left]
        .before
        .expect("origin has actual scope");
    let mut seam = left;
    let mut at = input.atoms[pair.right].source.start;
    if left != right {
        for event in &topology.capture.events[left + 1..right] {
            let scope = match event.event {
                SourceProcessingEvent::EnterScope(scope)
                | SourceProcessingEvent::ExitScope(scope) => scope,
                _ => continue,
            };
            let fact = &topology.capture.scopes[scope];
            if fact.domain == topology.domain && fact.definition.is_none() {
                let parent = fact.parent.expect("inline scope has parent");
                if depth(topology, parent) < depth(topology, owner) {
                    owner = parent;
                }
            }
        }
        for index in left + 1..=right {
            let event = &topology.capture.events[index];
            if event.before == Some(owner) {
                seam = index;
                at = event.at;
                break;
            }
        }
    }
    let mut cut = pair.left_event + 1;
    while cut < pair.right_event {
        let before = match events[cut].kind {
            Event::Anchor(anchor) => paragraph.capture_order(anchor.item) < seam,
            Event::Deleted(range, _, _) => range.end <= at,
            Event::Text(_) => false,
        };
        if !before {
            break;
        }
        cut += 1;
    }
    (owner, cut, at, seam)
}

/// A demanded authentic mixed seam observes its canonical candidate run once.
/// Other cuts retain unread stage state, rather than becoming absent by default.
pub(in crate::transform) fn proposal_at<'a>(
    input: &SourceProjection,
    styles: &mut Vec<StyleRow<'a>>,
    paragraph: &SourceContext<'a>,
    map: &mut TransformMap,
    event_cut: usize,
    reads: &mut PhraseReads,
) -> Result<Option<Proposal>> {
    let (pair, insertion) = match reads.sites[event_cut] {
        Site::Neutral => return Ok(None),
        Site::Unqualified(pair) => (pair, None),
        Site::Captured { pair, insertion } => (pair, Some(insertion)),
    };
    let Some(receipt) = reads.run(input, styles, paragraph, map, pair)? else {
        return Ok(None);
    };
    let analysis = &map.phrases[receipt];
    let cut = pair.right - analysis.rows.start;
    if !analysis.qualified[cut] {
        return Ok(None);
    }
    let at = analysis.coverage[cut].start;
    let Insertion {
        scope,
        source: at,
        capture_cut,
    } = insertion.ok_or_else(|| refusal(TextTransformError::MissingPhraseTopology { at }))?;
    let topology = paragraph
        .scope_topology
        .expect("qualified insertion capture");
    let own_style = topology.capture.scopes[scope]
        .style
        .as_ref()
        .ok_or_else(|| refusal(TextTransformError::MissingPhraseScopeStyle { scope, at }))?;
    let Some(code_point) = own_style.word_space_transform.replacement() else {
        return Ok(None);
    };
    let style = paragraph.append_scope_style(styles, scope)?;
    let issued_scope = if let Some(index) = map.scopes.iter().position(|row| row.ordinal == scope) {
        index
    } else {
        let index = map.scopes.len();
        map.scopes.try_reserve(1).map_err(|_| super::capacity())?;
        map.scopes.push(ScopeData { ordinal: scope });
        index
    };
    let occurrence = map.virtuals.len();
    map.virtuals.try_reserve(1).map_err(|_| super::capacity())?;
    let control_cut = reads.controls_before[event_cut];
    let context_origins = [
        map.original_pieces(pair.left)
            .first()
            .expect("actual left source incidence")
            .origin,
        map.original_pieces(pair.right)
            .last()
            .expect("actual right source incidence")
            .origin,
    ];
    map.virtuals.push(VirtualSeparatorData {
        phrase: receipt,
        context_origins,
        capture_cut,
        scope: issued_scope,
        source: at,
        processed: paragraph.processed.project(at, crate::Affinity::After),
        control_cut,
    });
    Ok(Some(Proposal {
        occurrence,
        code_point,
        style,
    }))
}

#[cfg(test)]
mod tests {
    use super::{Punctuation, punctuation};

    #[test]
    fn symmetric_quotes_attach_after_native_closing_context_without_recursion() {
        for quote in [0x22, 0x27] {
            for previous in ['猫', '3', ')', '’', '.', '。', '！'] {
                assert_eq!(
                    punctuation(&[u32::from(previous), quote], 1),
                    Punctuation::Closing,
                    "{previous:?}"
                );
            }
            for previous in ['(', ' ', '"', '\''] {
                assert_eq!(
                    punctuation(&[u32::from(previous), quote], 1),
                    Punctuation::Opening,
                    "{previous:?}"
                );
            }
            assert_eq!(punctuation(&[quote], 0), Punctuation::Opening);
        }
    }
}
