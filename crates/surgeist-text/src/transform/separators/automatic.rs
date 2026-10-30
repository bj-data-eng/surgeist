//! Native Japanese phrase proposals and their authentic scoped insertion seams.
use super::super::relation::{ScopeData, TransformMap, VirtualSeparatorData};
use crate::engine::analysis::wrapping::{Event, LogicalEvent};
use crate::engine::resolve::{AnalyzedStyleOwner, AnalyzedStyleRun};
use crate::phrase::qualification::{Punctuation, punctuation, separator};
use crate::phrase::{PhraseError, PhraseModel};
use crate::processing::contextual::ParagraphTopology;
use crate::projection::SourceProjection;
use crate::protocol::paragraph::{ParagraphControlKind, ParagraphData, SemanticAnchorItem};
use crate::{
    Error, ErrorCode, ErrorDetail, InlineBoxKind, Result, SourceProcessingEvent, TextTransformError,
};
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup, LineBreak},
};

pub(crate) struct Proposal {
    pub(super) cut: usize,
    pub(super) occurrence: usize,
    pub(super) code_point: u32,
    pub(super) style: usize,
}

/// The retained pre-replacement semantic analysis belongs to this exact map,
/// whose original rows and owning AnalyzedSource retain its input identity.
#[derive(Clone, Debug)]
pub(crate) struct PhraseAnalysis {
    model: PhraseModel,
    language: fontique::Language,
    input: Vec<u32>,
    coverage: Vec<crate::Range>,
    cuts: Vec<usize>,
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

fn incident_origins(
    paragraph: &ParagraphData,
    source: crate::Range,
) -> impl DoubleEndedIterator<Item = usize> + '_ {
    paragraph
        .origins
        .iter()
        .enumerate()
        .filter_map(move |(index, origin)| {
            (origin.aggregate.start < source.end && source.start < origin.aggregate.end)
                .then_some(index)
        })
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

fn anchor_order(
    topology: &ParagraphTopology,
    paragraph: &ParagraphData,
    item: SemanticAnchorItem,
) -> usize {
    match item {
        SemanticAnchorItem::Control(index) => topology.control_orders[index],
        SemanticAnchorItem::ComputedLineBoundary(index) => paragraph.computed_boundaries[index].capture_cut,
        SemanticAnchorItem::EmptyOrigin(index) => paragraph.captured_origin_event(index)
            .expect("captured empty origin retains its event"),
        SemanticAnchorItem::Atomic(index) => topology.capture.events.iter().position(|event| matches!(event.event, SourceProcessingEvent::Atomic { atomic, .. } if atomic == index)).expect("admitted atomic occurrence"),
    }
}

/// Returns the outermost owner and the earliest actual parent-domain seam.
fn custody(
    topology: &ParagraphTopology,
    paragraph: &ParagraphData,
    input: &SourceProjection,
    events: &[LogicalEvent],
    atom: usize,
) -> (usize, usize, usize, usize) {
    let left_origin = incident_origins(paragraph, input.atoms[atom - 1].source)
        .next_back()
        .expect("actual left original contribution");
    let right_origin = incident_origins(paragraph, input.atoms[atom].source)
        .next()
        .expect("actual right original contribution");
    let left = paragraph
        .captured_origin_event(left_origin)
        .expect("captured left origin retains its event");
    let right = paragraph
        .captured_origin_event(right_origin)
        .expect("captured right origin retains its event");
    let mut owner = topology.capture.events[left]
        .before
        .expect("origin has actual scope");
    let mut seam = left;
    let mut at = input.atoms[atom].source.start;
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
    let left_event = events
        .iter()
        .position(|event| matches!(event.kind, Event::Text(index) if index == atom - 1))
        .expect("left scalar event");
    let right_event = events
        .iter()
        .position(|event| matches!(event.kind, Event::Text(index) if index == atom))
        .expect("right scalar event");
    let mut cut = left_event + 1;
    while cut < right_event {
        let before = match events[cut].kind {
            Event::Anchor(anchor) => anchor_order(topology, paragraph, anchor.item) < seam,
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

pub(crate) fn proposals(
    input: &SourceProjection,
    styles: &mut Vec<AnalyzedStyleRun>,
    processed: &crate::ProcessedSource,
    paragraph: &ParagraphData,
    map: &mut TransformMap,
) -> Result<Vec<Proposal>> {
    if !styles.iter().any(|style| {
        PhraseModel::for_language(style.style.font_locale()).is_some()
            && style
                .style
                .authored()
                .word_space_transform
                .detects_phrases()
    }) {
        return Ok(Vec::new());
    }
    let events =
        crate::engine::analysis::wrapping::separator_events(processed, paragraph, input, styles);
    let mut stops = input.barrier_atoms.clone();
    stops.extend([0, input.atoms.len()]);
    for (atom, value) in input.atoms.iter().enumerate() {
        let style = &styles[map.rows[atom].style].style;
        if PhraseModel::for_language(style.font_locale()).is_none()
            || crate::engine::analysis::is_hard_separator(value.code_point)
        {
            stops.extend([atom, atom + 1]);
        }
    }
    for event in &events {
        if let Event::Anchor(anchor) = event.kind {
            let barrier = match anchor.item {
                SemanticAnchorItem::Atomic(index) => {
                    processed.source().boxes()[index].kind() == InlineBoxKind::InFlow
                }
                SemanticAnchorItem::Control(index) => {
                    paragraph.controls[index].kind == ParagraphControlKind::ForcedBreak
                }
                _ => false,
            };
            if barrier {
                stops.push(event.atom);
            }
        }
    }
    let connection_seams = super::connection_seams(input, &events, paragraph)?;
    for seam in &connection_seams {
        stops.push(seam.atom);
    }
    stops.sort_unstable();
    stops.dedup();
    let mut output = Vec::new();
    for pair in stops.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        if end - start < 2
            || PhraseModel::for_language(styles[map.rows[start].style].style.font_locale())
                .is_none()
        {
            continue;
        }
        if !(start..end).any(|atom| {
            styles[map.rows[atom].style]
                .style
                .authored()
                .word_space_transform
                .detects_phrases()
        }) {
            continue;
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(end - start)
            .map_err(|_| super::capacity())?;
        values.extend(input.atoms[start..end].iter().map(|atom| atom.code_point));
        let mut coverage = Vec::new();
        coverage
            .try_reserve_exact(end - start)
            .map_err(|_| super::capacity())?;
        coverage.extend(input.atoms[start..end].iter().map(|atom| atom.source));
        let mut analysis = PhraseAnalysis {
            model: PhraseModel::for_language(styles[map.rows[start].style].style.font_locale())
                .expect("supported actual locale selects its model"),
            language: styles[map.rows[start].style]
                .style
                .font_locale()
                .expect("supported actual locale"),
            input: values,
            coverage,
            cuts: Vec::new(),
        };
        analysis.cuts = match (analysis.model, analysis.language.language()) {
            (PhraseModel::BudouXJapanese093, "ja") => analysis.model.boundaries(&analysis.input),
            _ => unreachable!("supported language selects its declared model"),
        }
        .map_err(|error| match error {
            PhraseError::Capacity => super::capacity(),
            PhraseError::InvalidData => Error::new(
                ErrorCode::InvalidSourceProcessing,
                "invalid automatic phrase model",
            )
            .with_detail(ErrorDetail::TextTransform {
                reason: TextTransformError::InvalidPhraseData,
            }),
        })?;
        let values = &analysis.input;
        let mut punctuation_only = Vec::new();
        punctuation_only
            .try_reserve_exact(
                analysis
                    .cuts
                    .len()
                    .checked_add(1)
                    .and_then(|n| n.checked_mul(2))
                    .ok_or_else(super::capacity)?,
            )
            .map_err(|_| super::capacity())?;
        let mut previous = 0;
        let categories = CodePointMapData::<GeneralCategory>::new();
        for next in analysis
            .cuts
            .iter()
            .copied()
            .chain(std::iter::once(values.len()))
        {
            if values[previous..next]
                .iter()
                .all(|&cp| GeneralCategoryGroup::Punctuation.contains(categories.get32(cp)))
            {
                punctuation_only.extend([previous, next]);
            }
            previous = next;
        }
        for &cut in &analysis.cuts {
            let atom = start + cut;
            if punctuation_only.contains(&cut) || !eligible(input, values, start, cut) {
                continue;
            }
            if ![atom - 1, atom].iter().any(|&index| {
                styles[map.rows[index].style]
                    .style
                    .authored()
                    .word_space_transform
                    .detects_phrases()
            }) {
                continue;
            }
            // Prove forced suppression in the actual connected mixed seam before
            // consuming any topology or owner Style.
            let left = events
                .iter()
                .position(|event| matches!(event.kind, Event::Text(index) if index == atom - 1))
                .expect("actual original scalar");
            let right = events
                .iter()
                .position(|event| matches!(event.kind, Event::Text(index) if index == atom))
                .expect("actual original scalar");
            if events[left + 1..right].iter().any(|event| {
                super::forced(event, input, processed.source(), paragraph) == Some(true)
            }) {
                continue;
            }
            let at = analysis.coverage[cut].start;
            let topology = paragraph
                .scope_topology
                .as_ref()
                .filter(|topology| topology.capture.complete[topology.domain])
                .ok_or_else(|| refusal(TextTransformError::MissingPhraseTopology { at }))?;
            let (scope, event_cut, at, capture_cut) =
                custody(topology, paragraph, input, &events, atom);
            let fact = &topology.capture.scopes[scope];
            let own_style = fact.style.as_ref().ok_or_else(|| {
                refusal(TextTransformError::MissingPhraseScopeStyle { scope, at })
            })?;
            let Some(code_point) = own_style.word_space_transform.replacement() else {
                continue;
            };
            let style = if let Some(index) = styles.iter().position(
                |style| matches!(style.owner, AnalyzedStyleOwner::Scope(index) if index == scope),
            ) {
                index
            } else {
                let index = styles.len();
                styles.try_reserve(1).map_err(|_| super::capacity())?;
                styles.push(AnalyzedStyleRun {
                    owner: AnalyzedStyleOwner::Scope(scope),
                    style: crate::ValidatedStyle::for_processed(own_style.clone())?,
                });
                index
            };
            let issued_scope =
                if let Some(index) = map.scopes.iter().position(|row| row.ordinal == scope) {
                    index
                } else {
                    let index = map.scopes.len();
                    map.scopes.try_reserve(1).map_err(|_| super::capacity())?;
                    map.scopes.push(ScopeData { ordinal: scope });
                    index
                };
            let occurrence = map.virtuals.len();
            map.virtuals.try_reserve(1).map_err(|_| super::capacity())?;
            let control_cut = events[..event_cut].iter().filter(|event| matches!(event.kind, Event::Anchor(anchor) if matches!(anchor.item, SemanticAnchorItem::Control(_)))).count();
            map.virtuals.push(VirtualSeparatorData {
                capture_cut,
                scope: issued_scope,
                source: at,
                processed: processed.project(at, crate::Affinity::After),
                control_cut,
            });
            output.try_reserve(1).map_err(|_| super::capacity())?;
            output.push(Proposal {
                cut: event_cut,
                occurrence,
                code_point,
                style,
            });
        }
        map.phrases.try_reserve(1).map_err(|_| super::capacity())?;
        map.phrases.push(analysis);
    }
    output.sort_by_key(|proposal| proposal.cut);
    Ok(output)
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
