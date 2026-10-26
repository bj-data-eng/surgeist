//! Final semantic phrase inference and Appendix-H-qualified spans.

use std::ops::Range;

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup, LineBreak},
};

use super::{anchor_atom, capacity, is_hard_separator, style_at};
use crate::engine::resolve::AnalyzedStyleRun;
use crate::phrase::{
    PhraseError, PhraseModel,
    qualification::{Punctuation, punctuation, separator},
};
use crate::projection::SourceProjection;
use crate::protocol::paragraph::{ParagraphControlKind, ParagraphData, SemanticAnchorItem};
use crate::{InlineBox, InlineBoxKind, Result};

#[derive(Clone, Debug)]
pub(crate) struct PhraseComponent {
    pub(crate) model: PhraseModel,
    pub(crate) atoms: Range<usize>,
    /// Raw inference and qualified cuts are distinct absolute atom endpoints.
    pub(crate) raw_cuts: Vec<usize>,
    pub(crate) qualified_cuts: Vec<usize>,
}

#[derive(Clone, Debug)]
pub(crate) struct PhraseData {
    pub(crate) component: usize,
    pub(crate) atoms: Range<usize>,
    pub(crate) presentation: crate::Range,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PhraseCatalog {
    pub(crate) components: Vec<PhraseComponent>,
    pub(crate) phrases: Vec<PhraseData>,
}

impl PhraseCatalog {
    pub(crate) fn phrase_containing_cut(&self, atom: usize) -> Option<usize> {
        let index = self
            .phrases
            .partition_point(|phrase| phrase.atoms.end <= atom);
        self.phrases
            .get(index)
            .filter(|phrase| phrase.atoms.start < atom && atom < phrase.atoms.end)
            .map(|_| index)
    }
}

fn invalid_model() -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::InvalidPhraseAnalysis,
        "invalid final phrase model result",
    )
}

pub(super) fn analyze(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    boxes: &[InlineBox],
    paragraph: &ParagraphData,
    units: &[Range<usize>],
) -> Result<PhraseCatalog> {
    let mut catalog = PhraseCatalog::default();
    if !styles
        .iter()
        .any(|row| row.style.authored().effective_breaking().auto_phrase)
    {
        return Ok(catalog);
    }
    let mut stops = Vec::new();
    stops
        .try_reserve(
            projection
                .barrier_atoms
                .len()
                .checked_add(units.len().checked_mul(2).ok_or_else(capacity)?)
                .and_then(|n| n.checked_add(paragraph.anchor_order.len()))
                .and_then(|n| n.checked_add(2))
                .ok_or_else(capacity)?,
        )
        .map_err(|_| capacity())?;
    stops.extend_from_slice(&projection.barrier_atoms);
    stops.extend([0, projection.atoms.len()]);
    let mut previous_model = None;
    for unit in units {
        // A language/style seam never creates a fractional TCU identity.
        let mut model = None;
        for atom in unit.clone() {
            let next = PhraseModel::for_language(
                styles[style_at(projection, styles, atom)]
                    .style
                    .font_locale(),
            );
            if next.is_none()
                || model.is_some_and(|model| Some(model) != next)
                || is_hard_separator(projection.atoms[atom].code_point)
            {
                model = None;
                break;
            }
            model = next;
        }
        if model.is_none() {
            stops.extend([unit.start, unit.end]);
        } else if previous_model.is_some() && previous_model != model {
            stops.push(unit.start);
        }
        previous_model = model;
    }
    for (occurrence, anchor) in paragraph.anchor_order.iter().enumerate() {
        if matches!(anchor.item, SemanticAnchorItem::Atomic(i)
            if boxes[i].kind() == InlineBoxKind::InFlow)
            || matches!(anchor.item, SemanticAnchorItem::Control(i)
                if paragraph.controls[i].kind == ParagraphControlKind::ForcedBreak)
        {
            stops.push(anchor_atom(occurrence, paragraph, boxes, projection));
        }
    }
    stops.sort_unstable();
    stops.dedup();
    for pair in stops.windows(2) {
        let atoms = pair[0]..pair[1];
        if atoms.is_empty()
            || is_hard_separator(projection.atoms[atoms.start].code_point)
            || !(atoms.clone()).any(|atom| {
                styles[style_at(projection, styles, atom)]
                    .style
                    .authored()
                    .effective_breaking()
                    .auto_phrase
            })
        {
            continue;
        }
        let model = PhraseModel::for_language(
            styles[style_at(projection, styles, atoms.start)]
                .style
                .font_locale(),
        );
        let Some(model) = model else { continue };
        // Unsupported TCUs form their own stop pair; don't infer from its first
        // supported contribution when another contribution lacks that language.
        if atoms.clone().any(|atom| {
            PhraseModel::for_language(
                styles[style_at(projection, styles, atom)]
                    .style
                    .font_locale(),
            ) != Some(model)
        }) {
            continue;
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(atoms.len())
            .map_err(|_| capacity())?;
        values.extend(
            projection.atoms[atoms.clone()]
                .iter()
                .map(|atom| atom.code_point),
        );
        let raw = model.boundaries(&values).map_err(|error| match error {
            PhraseError::Capacity => capacity(),
            PhraseError::InvalidData => invalid_model(),
        })?;
        let qualified_cuts = qualify(&values, &raw, atoms.start, units)?;
        let mut raw_cuts = raw;
        raw_cuts.iter_mut().for_each(|cut| *cut += atoms.start);
        let component = catalog.components.len();
        catalog.components.try_reserve(1).map_err(|_| capacity())?;
        catalog
            .phrases
            .try_reserve(qualified_cuts.len().checked_add(1).ok_or_else(capacity)?)
            .map_err(|_| capacity())?;
        let mut start = atoms.start;
        for end in qualified_cuts
            .iter()
            .copied()
            .chain(std::iter::once(atoms.end))
        {
            catalog.phrases.push(PhraseData {
                component,
                atoms: start..end,
                presentation: crate::Range::new(
                    projection.atoms[start].presentation.start,
                    projection.atoms[end - 1].presentation.end,
                ),
            });
            start = end;
        }
        catalog.components.push(PhraseComponent {
            model,
            atoms,
            raw_cuts,
            qualified_cuts,
        });
    }
    Ok(catalog)
}

/// Final-analysis qualification can attach suffixes; separator insertion keeps
/// its published filtering-only execution and never calls this relocation.
fn qualify(
    values: &[u32],
    raw: &[usize],
    start: usize,
    units: &[Range<usize>],
) -> Result<Vec<usize>> {
    let n = values.len();
    if raw.windows(2).any(|pair| pair[0] >= pair[1]) || raw.iter().any(|cut| *cut == 0 || *cut >= n)
    {
        return Err(invalid_model());
    }
    let mut kinds = Vec::new();
    kinds.try_reserve_exact(n).map_err(|_| capacity())?;
    kinds.extend((0..n).map(|i| punctuation(values, i)));
    let mut opening = Vec::new();
    opening
        .try_reserve_exact(n.checked_add(1).ok_or_else(capacity)?)
        .map_err(|_| capacity())?;
    opening.push(0);
    for i in 0..n {
        opening.push(if kinds[i] == Punctuation::Opening {
            opening[i]
        } else {
            i + 1
        });
    }
    let mut suffix = Vec::new();
    suffix.try_reserve_exact(n + 1).map_err(|_| capacity())?;
    suffix.resize(n + 1, n);
    for i in (0..n).rev() {
        suffix[i] = if separator(values[i]) || values[i] == 9 || kinds[i] == Punctuation::Closing {
            suffix[i + 1]
        } else {
            i
        };
    }
    let mut punctuation_only = Vec::new();
    punctuation_only
        .try_reserve_exact(n + 1)
        .map_err(|_| capacity())?;
    punctuation_only.resize(n + 1, false);
    let categories = CodePointMapData::<GeneralCategory>::new();
    let mut previous = 0;
    for end in raw.iter().copied().chain(std::iter::once(n)) {
        if values[previous..end]
            .iter()
            .all(|cp| GeneralCategoryGroup::Punctuation.contains(categories.get32(*cp)))
        {
            punctuation_only[previous] = true;
            punctuation_only[end] = true;
        }
        previous = end;
    }
    let classes = CodePointMapData::<LineBreak>::new();
    let mut effective = Vec::new();
    effective.try_reserve_exact(n).map_err(|_| capacity())?;
    for cp in values {
        let class = super::line_break::resolved_class(*cp, classes.get32(*cp));
        let inherited = matches!(class, LineBreak::CombiningMark | LineBreak::ZWJ)
            .then(|| {
                effective.last().copied().filter(|prior| {
                    !matches!(
                        *prior,
                        LineBreak::MandatoryBreak
                            | LineBreak::CarriageReturn
                            | LineBreak::LineFeed
                            | LineBreak::NextLine
                            | LineBreak::Space
                            | LineBreak::ZWSpace
                    )
                })
            })
            .flatten();
        effective.push(inherited.unwrap_or(
            if matches!(class, LineBreak::CombiningMark | LineBreak::ZWJ) {
                LineBreak::Alphabetic
            } else {
                class
            },
        ));
    }
    let protected = |class| {
        matches!(
            class,
            LineBreak::Glue | LineBreak::WordJoiner | LineBreak::ZWJ
        )
    };
    let mut cuts = Vec::new();
    cuts.try_reserve_exact(raw.len()).map_err(|_| capacity())?;
    let mut unit = units.partition_point(|unit| unit.start < start);
    for raw in raw.iter().copied() {
        if punctuation_only[raw] {
            continue;
        }
        let cut = suffix[opening[raw]];
        if cut == 0
            || cut >= n
            || cuts.last().is_some_and(|previous| *previous >= start + cut)
            || protected(effective[cut - 1])
            || protected(classes.get32(values[cut]))
            || values[cut - 1] == 0x200d
            || matches!(
                kinds[cut - 1],
                Punctuation::Opening | Punctuation::Unfamiliar
            )
            || matches!(kinds[cut], Punctuation::Closing | Punctuation::Unfamiliar)
        {
            continue;
        }
        while units.get(unit).is_some_and(|unit| unit.start < start + cut) {
            unit += 1;
        }
        if units
            .get(unit)
            .is_some_and(|unit| unit.start == start + cut)
        {
            cuts.push(start + cut);
        }
    }
    Ok(cuts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::analysis::LineCause;

    fn style(locale: &str) -> crate::Style {
        crate::Style {
            word_break: crate::WordBreak::AutoPhrase,
            locale: Some(locale.to_owned()),
            ..crate::Style::default()
        }
    }

    #[test]
    fn published_phrase_context_includes_normal_runs_and_keeps_final_coordinates() {
        let mut source = crate::Source::new("今日は天気です。");
        source.span(crate::Range::new(1, 3), style("ja-JP"));
        let analyzed = crate::AnalyzedSource::new(
            source,
            crate::Style {
                word_break: crate::WordBreak::Normal,
                locale: Some("ja".to_owned()),
                ..crate::Style::default()
            },
        )
        .unwrap();
        let catalog = &analyzed.inner.analysis.phrases;
        assert_eq!(catalog.components.len(), 1);
        assert_eq!(catalog.components[0].atoms, 0..8);
        assert_eq!(catalog.components[0].model, PhraseModel::BudouXJapanese093);
        // Retained upstream published literal: 今日は▁天気です。
        assert_eq!(catalog.components[0].raw_cuts, [3]);
        assert_eq!(catalog.components[0].qualified_cuts, [3]);
        assert_eq!(
            catalog
                .phrases
                .iter()
                .map(|row| row.atoms.clone())
                .collect::<Vec<_>>(),
            [0..3, 3..8]
        );
        assert_eq!(
            catalog
                .phrases
                .iter()
                .map(|row| row.presentation)
                .collect::<Vec<_>>(),
            [crate::Range::new(0, 3), crate::Range::new(3, 8)]
        );
        assert_eq!(catalog.phrase_containing_cut(2), Some(0));
        assert_eq!(catalog.phrase_containing_cut(3), None);
        assert_eq!(catalog.phrase_containing_cut(8), None);
    }

    #[test]
    fn supported_empty_inference_and_unsupported_language_are_distinct() {
        for locale in ["ja", "ja-JP", "ja-Jpan", "ja-u-ca-japanese"] {
            let analyzed =
                crate::AnalyzedSource::new(crate::Source::new("日"), style(locale)).unwrap();
            let catalog = &analyzed.inner.analysis.phrases;
            assert_eq!(catalog.components.len(), 1, "{locale}");
            assert!(catalog.components[0].raw_cuts.is_empty());
            assert_eq!(catalog.phrases[0].atoms, 0..1);
        }
        for locale in ["en-JP", "und-Jpan", "ja-Latn", "zh", "ko"] {
            let authored = style(locale);
            assert!(authored.effective_breaking().auto_phrase);
            let analyzed =
                crate::AnalyzedSource::new(crate::Source::new("日本へ"), authored).unwrap();
            assert!(
                analyzed.inner.analysis.phrases.components.is_empty(),
                "{locale}"
            );
            assert!(
                analyzed.inner.analysis.phrases.phrases.is_empty(),
                "{locale}"
            );
        }
    }

    #[test]
    fn hard_and_unsupported_tcus_bound_final_components() {
        let mut source = crate::Source::new("日は\n天気");
        source.span(crate::Range::new(4, 5), style("en"));
        let analyzed = crate::AnalyzedSource::new(source, style("ja")).unwrap();
        assert_eq!(
            analyzed
                .inner
                .analysis
                .phrases
                .components
                .iter()
                .map(|row| row.atoms.clone())
                .collect::<Vec<_>>(),
            [0..2, 3..4]
        );
        let mut source = crate::Source::new("か\u{3099}日");
        source.span(crate::Range::new(1, 2), style("en"));
        let analyzed = crate::AnalyzedSource::new(source, style("ja")).unwrap();
        assert_eq!(analyzed.inner.analysis.graphemes, [0..2, 2..3]);
        assert_eq!(analyzed.inner.analysis.phrases.components[0].atoms, 2..3);
    }

    #[test]
    fn final_qualifier_attaches_suffixes_but_keeps_whole_tcus_and_glue() {
        for (text, raw, expected) in [
            ("a b", vec![1], vec![2]),
            ("ab, c", vec![2], vec![4]),
            ("a(b", vec![2], vec![1]),
            ("a\u{a0}b", vec![1], vec![]),
            ("a\u{2060}b", vec![1, 2], vec![]),
            ("a\u{200d}b", vec![1, 2], vec![]),
            ("a\u{301}b", vec![1], vec![]),
        ] {
            let projection = SourceProjection::new(&crate::Source::new(text)).unwrap();
            let units: Vec<_> = projection
                .graphemes
                .windows(2)
                .map(|pair| {
                    projection
                        .atoms
                        .partition_point(|atom| atom.presentation.start < pair[0])
                        ..projection
                            .atoms
                            .partition_point(|atom| atom.presentation.start < pair[1])
                })
                .collect();
            let values: Vec<_> = projection
                .atoms
                .iter()
                .map(|atom| atom.code_point)
                .collect();
            assert_eq!(
                qualify(&values, &raw, 0, &units).unwrap(),
                expected,
                "{text:?}"
            );
        }
    }

    #[test]
    fn causes_keep_independent_owners_without_rewriting_word_metadata() {
        let mut authored = style("ja");
        authored.line_break = crate::LineBreak::Loose;
        let analyzed = crate::AnalyzedSource::new(crate::Source::new("漢ぁ"), authored).unwrap();
        assert!(
            analyzed
                .inner
                .analysis
                .line_candidates
                .iter()
                .any(|row| row.atom == 1 && row.cause == LineCause::RequiredLineBreak)
        );
        let analyzed =
            crate::AnalyzedSource::new(crate::Source::new("a\u{200b} b"), style("ja")).unwrap();
        assert!(
            analyzed
                .inner
                .analysis
                .line_candidates
                .iter()
                .any(|row| row.atom == 3 && row.cause == LineCause::SourceZwsp { atom: 1 })
        );
        assert!(
            analyzed
                .inner
                .analysis
                .line_candidates
                .iter()
                .any(|row| row.atom == 3 && row.cause == LineCause::Whitespace { atom: 2 })
        );
        let mut source = crate::Source::new("漢 漢");
        source.span(crate::Range::new(1, 2), style("ja"));
        let analyzed = crate::AnalyzedSource::new(
            source,
            crate::Style {
                locale: Some("ja".to_owned()),
                ..crate::Style::default()
            },
        )
        .unwrap();
        let whitespace = analyzed
            .inner
            .analysis
            .line_candidates
            .iter()
            .find(|row| row.atom == 2 && row.cause == LineCause::Whitespace { atom: 1 })
            .unwrap();
        assert_eq!(whitespace.style, analyzed.inner.analysis.infos[2].1);
        assert_ne!(whitespace.style, analyzed.inner.analysis.infos[1].1);
        let source = crate::Source::new("กก");
        let normal = crate::AnalyzedSource::new(
            source.clone(),
            crate::Style {
                word_break: crate::WordBreak::Normal,
                locale: Some("th".to_owned()),
                ..crate::Style::default()
            },
        )
        .unwrap();
        let analyzed = crate::AnalyzedSource::new(source, style("th")).unwrap();
        assert!(analyzed.inner.analysis.phrases.phrases.is_empty());
        assert!(
            analyzed
                .inner
                .analysis
                .line_candidates
                .iter()
                .any(|row| row.atom == 1 && row.cause == LineCause::LexicalFallback)
        );
        assert_eq!(
            analyzed
                .inner
                .words
                .rows()
                .iter()
                .map(|word| word.atoms.clone())
                .collect::<Vec<_>>(),
            normal
                .inner
                .words
                .rows()
                .iter()
                .map(|word| word.atoms.clone())
                .collect::<Vec<_>>()
        );
    }
}
