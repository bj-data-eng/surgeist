use std::ops::Range as IndexRange;

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup, GraphemeClusterBreak, Script},
    script::ScriptWithExtensions,
};
use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};

use super::casing::{self, Locale, Mapper, MappingError, Operation};
use crate::{
    Error, ErrorCode, ErrorDetail, Result, TextTransform, TextTransformError,
    projection::SourceProjection,
};

pub(super) fn failed(reason: MappingError) -> Error {
    if let MappingError::Context(edge) = reason {
        return edge.unavailable(crate::SourceContinuityOperation::Transformation);
    }
    let (code, reason) = match reason {
        MappingError::DataUnavailable => (
            ErrorCode::UnsupportedFeature,
            TextTransformError::CasingDataUnavailable,
        ),
        MappingError::InvalidException | MappingError::InvalidScalar => (
            ErrorCode::InvalidSourceProcessing,
            TextTransformError::InvalidCasingData,
        ),
        MappingError::InvalidLocale => (
            ErrorCode::InvalidStyle,
            TextTransformError::InvalidContentLocale,
        ),
        MappingError::Context(_) => unreachable!("context refusal returned above"),
        MappingError::Source(error) => return error,
        MappingError::Storage => (
            ErrorCode::SourceProcessingCapacity,
            TextTransformError::Capacity,
        ),
    };
    Error::new(
        code,
        "text transformation could not retain its actual mapping",
    )
    .with_detail(ErrorDetail::TextTransform { reason })
}

#[derive(Clone, Copy, Default)]
struct Initial {
    title: bool,
    context: bool,
}

/// Starts are facts about existing input positions, never an assertion that a
/// temporary window end is the end of a source word.
#[derive(Clone, Copy)]
pub(super) struct WordStarts {
    pub(super) ordinary: usize,
    pub(super) hyphen: usize,
}

pub(super) struct WordFacts {
    pub(super) initial: Vec<Option<Operation>>,
    pub(super) starts: Vec<WordStarts>,
}

struct WordEffects {
    initial: Vec<Initial>,
    starts: Vec<WordStarts>,
}

fn base(input: &SourceProjection, unit: IndexRange<usize>) -> Option<usize> {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    if input.atoms[unit.clone()].iter().any(|atom| {
        categories.get32(atom.code_point) == GeneralCategory::EnclosingMark
            && scripts.has_script32(atom.code_point, Script::Common)
    }) {
        return None;
    }
    let gcb = CodePointMapData::<GraphemeClusterBreak>::new();
    let base = unit
        .clone()
        .find(|&row| gcb.get32(input.atoms[row].code_point) != GraphemeClusterBreak::Prepend)?;
    let category = categories.get32(input.atoms[base].code_point);
    if category == GeneralCategory::SpaceSeparator && unit.len() > 1 {
        return None;
    }
    (GeneralCategoryGroup::Letter.contains(category)
        || GeneralCategoryGroup::Number.contains(category))
    .then_some(base)
}

fn classify(
    input: &SourceProjection,
    values: &[u32],
    domain: IndexRange<usize>,
    mapper: &Mapper,
    effects: &mut WordEffects,
) -> Result<()> {
    let point = |atom: usize| {
        input
            .atoms
            .get(atom)
            .map_or(input.presentation_end(), |atom| atom.presentation.start)
    };
    let byte = input.segmentation_byte(point(domain.start));
    let end = input.segmentation_byte(point(domain.end));
    let atom_at = |byte| -> Result<usize> {
        let point = input.segmentation_unit(byte)?;
        Ok(input
            .atoms
            .partition_point(|atom| atom.presentation.start < point))
    };
    let mut previous = domain.start;
    for (offset, kind) in
        WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default())
            .segment_str(&input.segmentation[byte..end])
            .iter_with_word_type()
    {
        let next = atom_at(byte + offset)?;
        if previous < next && kind.is_word_like() {
            let word = previous..next;
            for row in word.clone() {
                effects.starts[row] = WordStarts {
                    ordinary: previous,
                    hyphen: previous,
                };
            }
            if let Some(following) = effects.starts.get_mut(next) {
                following.hyphen = previous;
            }
            // Seek the first EGC whose end is after this word start. Keep
            // a unit beginning before the word: its base/context still owns
            // the initial-unit decision when the ranges genuinely overlap.
            let word_start = point(word.start);
            let word_end = point(word.end);
            let grapheme = input
                .graphemes
                .partition_point(|&edge| edge <= word_start)
                .saturating_sub(1);
            for pair in input.graphemes[grapheme..]
                .windows(2)
                .take_while(|pair| pair[0] < word_end)
            {
                let first = input
                    .atoms
                    .partition_point(|atom| atom.presentation.start < pair[0]);
                let last = input
                    .atoms
                    .partition_point(|atom| atom.presentation.start < pair[1]);
                if last <= word.start || first >= word.end {
                    continue;
                }
                let Some(base) = base(input, first..last) else {
                    continue;
                };
                // The first Letter/Number unit occupies this slot, even when
                // it is a number, uppercase or caseless letter.
                if mapper.case_type(values[base])
                    == Some(icu_casemap::provider::data::CaseType::Lower)
                {
                    effects.initial[base].title = true;
                    for (row, unit) in effects
                        .initial
                        .iter_mut()
                        .enumerate()
                        .take(last)
                        .skip(first)
                    {
                        if row != base {
                            unit.context = true;
                        }
                    }
                    for row in base + 1..word.end {
                        if matches!(values[row], 0x6a | 0x4a)
                            && mapper
                                .dutch_j(&values[word.start..row], &values[row + 1..word.end])
                                .map_err(failed)?
                        {
                            effects.initial[row].context = true;
                        }
                    }
                }
                break;
            }
        }
        previous = next;
    }
    Ok(())
}

pub(super) fn word_facts(input: &SourceProjection, mapper: &Mapper) -> Result<WordFacts> {
    let values = super::context::collect_rows(input.atoms.iter().map(|atom| atom.code_point))?;
    let mut effects = WordEffects {
        initial: super::context::filled(values.len(), Initial::default())?,
        starts: super::context::collect_rows((0..values.len()).map(|row| WordStarts {
            ordinary: row,
            hyphen: row,
        }))?,
    };
    classify(input, &values, 0..values.len(), mapper, &mut effects)?;
    let initial = super::context::collect_rows(effects.initial.into_iter().map(|role| {
        if role.title {
            Some(Operation::Title)
        } else if role.context {
            Some(Operation::TitleContext)
        } else {
            None
        }
    }))?;
    Ok(WordFacts {
        initial,
        starts: effects.starts,
    })
}

#[derive(Clone, Copy, Default)]
pub(super) struct PrefixEffect {
    pub(super) preserve: bool,
    pub(super) remove: bool,
    pub(super) insert: bool,
}

pub(super) struct MappingPlan {
    pub(super) operation: Option<Operation>,
    pub(super) locale: Locale,
    pub(super) prefix: PrefixEffect,
}

pub(super) struct ComponentInput<'a> {
    pub(super) code_point: u32,
    pub(super) style: super::context::StyleRef<'a>,
    pub(super) math_eligible: bool,
    pub(super) separator_space: bool,
    pub(super) preserves_space: bool,
}

pub(super) struct Mapped {
    pub(super) output: Vec<u32>,
    pub(super) identities: Vec<u32>,
    pub(super) operation: Option<Operation>,
}

pub(super) fn component(
    input: &casing::Input<'_>,
    at: usize,
    component: ComponentInput<'_>,
    plan: MappingPlan,
    mapper: Option<&Mapper>,
) -> Result<Mapped> {
    let mut output = Vec::new();
    output
        .try_reserve(2)
        .map_err(|_| failed(MappingError::Storage))?;
    if !plan.prefix.remove {
        if let Some(operation) = plan.operation
            && !plan.prefix.preserve
        {
            if plan.prefix.insert {
                output.push(0x2d);
            }
            mapper
                .expect("actual demanded casing mapper")
                .map_in_context(input, at, operation, plan.locale, &mut output)
                .map_err(failed)?;
        } else {
            output.push(component.code_point);
        }
    }
    let transform = component.style.text_transform();
    if transform == TextTransform::MathAuto
        && component.math_eligible
        && let Some(target) = super::math::map(component.code_point)
    {
        output[0] = target;
    }
    let identities = super::context::collect_rows(output.iter().copied().map(|cp| {
        if let TextTransform::Set(set) = transform
            && set.full_size_kana()
        {
            super::full_size_kana(cp).map_or(cp, u32::from)
        } else {
            cp
        }
    }))?;
    if let TextTransform::Set(set) = transform {
        if set.full_width() {
            for cp in &mut output {
                if (*cp != 0x20 || component.separator_space || component.preserves_space)
                    && let Some(target) = super::width::map(*cp)
                {
                    *cp = target;
                }
            }
        }
        if set.full_size_kana() {
            for cp in &mut output {
                if let Some(target) = super::full_size_kana(*cp) {
                    *cp = u32::from(target);
                }
            }
        }
    }
    Ok(Mapped {
        output,
        identities,
        operation: plan.operation,
    })
}

pub(super) struct Mappings {
    pub(super) output: Vec<u32>,
    pub(super) identities: Vec<u32>,
    pub(super) spans: Vec<IndexRange<usize>>,
    pub(super) operations: Vec<Option<Operation>>,
}
