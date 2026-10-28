//! Native whole-upright arrangement and source-script capability admission.
//!
//! These units never replace editing EGCs or offered line boundaries. The same
//! operation consumes original and authentic selected-fragment presentations.

use std::{collections::BTreeSet, ops::Range as IndexRange};

use icu_properties::{
    CodePointMapData, PropertyNamesShort, PropertyParser,
    props::{EastAsianWidth, GeneralCategory, GeneralCategoryGroup, GraphemeClusterBreak, Script},
    script::ScriptWithExtensions,
};

use super::{Analysis, capacity, is_hard_separator, unit_properties};
use crate::{
    Error, ErrorCode, ErrorDetail, Result, TextScript, TextStyleOwner, UprightContent,
    UprightScriptContext, UprightScriptContextFailure, UprightScriptFailure,
    engine::{
        OriginalAtom,
        resolve::{AnalyzedStyleOwner, AnalyzedStyleRun},
    },
    projection::SourceProjection,
};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UprightUnitRole {
    IsolatedUnit,
    NormalVerticalContext,
}

/// The reason selects only context boundaries, never a provider script.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UprightUnitContext {
    WidthOverride(UprightUnitRole),
    ExplicitScript(UprightUnitRole),
    UniformScripts(UprightUnitRole),
    /// The admitted candidate roles disagree and no source context resolves them.
    MixedRoleFallback,
}

impl UprightUnitContext {
    const fn role(self) -> UprightUnitRole {
        match self {
            Self::WidthOverride(role) | Self::ExplicitScript(role) | Self::UniformScripts(role) => {
                role
            }
            Self::MixedRoleFallback => UprightUnitRole::IsolatedUnit,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UprightUnit {
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) context: UprightUnitContext,
}

impl UprightUnit {
    pub(crate) const fn isolate(&self) -> bool {
        matches!(self.context.role(), UprightUnitRole::IsolatedUnit)
    }
}

/// Uses actual analysis styles and caller-owned occurrence/provenance facts.
/// `content` is evaluated only for a genuine source-script capability failure.
pub(crate) fn units(
    projection: &SourceProjection,
    analysis: &Analysis,
    styles: &[AnalyzedStyleRun],
    barriers: &BTreeSet<usize>,
    content: impl Fn(IndexRange<usize>) -> UprightContent,
) -> Result<Vec<UprightUnit>> {
    let ranges = arrangement_ranges(projection, &analysis.graphemes, barriers)?;
    let mut result = Vec::new();
    result.try_reserve(ranges.len()).map_err(|_| capacity())?;
    for atoms in ranges {
        let first = &analysis.graphemes[analysis
            .graphemes
            .binary_search_by_key(&atoms.start, |egc| egc.start)
            .expect("arrangement units start at an actual presentation EGC")];
        let properties = properties(&projection.atoms[first.clone()])?;
        let indices = style_indices(analysis, atoms.clone())?;
        let context = match context(&properties, styles, &indices) {
            Ok(context) => context,
            Err(reason) => {
                let candidates = diagnostic_scripts(&properties.scripts)?;
                let contexts = diagnostic_contexts(styles, &indices)?;
                return Err(Error::new(
                    ErrorCode::UnsupportedFeature,
                    "upright source script context is not supported",
                )
                .with_detail(ErrorDetail::UprightScript {
                    content: Box::new(content(atoms)),
                    candidates: candidates.into_boxed_slice(),
                    contexts: contexts.into_boxed_slice(),
                    reason,
                }));
            }
        };
        result.push(UprightUnit { atoms, context });
    }
    Ok(result)
}

fn arrangement_ranges(
    projection: &SourceProjection,
    graphemes: &[IndexRange<usize>],
    barriers: &BTreeSet<usize>,
) -> Result<Vec<IndexRange<usize>>> {
    let mut result: Vec<IndexRange<usize>> = Vec::new();
    result
        .try_reserve(graphemes.len())
        .map_err(|_| capacity())?;
    let mut stops = barriers.clone();
    stops.extend(projection.barrier_atoms.iter().copied());
    let mut prior_hard = false;
    for egc in graphemes {
        let atoms = &projection.atoms[egc.clone()];
        let hard = atoms.iter().any(|atom| is_hard_separator(atom.code_point));
        let blocked = stops.contains(&egc.start);
        let internal_stop = stops
            .range((
                std::ops::Bound::Excluded(egc.start),
                std::ops::Bound::Excluded(egc.end),
            ))
            .next()
            .is_some();
        let append = !hard
            && !prior_hard
            && !blocked
            && !internal_stop
            && base(atoms).is_some_and(|atom| tibetan_follower(atom.code_point))
            && result.last().is_some_and(|prior| prior.end == egc.start);
        if append {
            result
                .last_mut()
                .expect("an actual preceding unit exists")
                .end = egc.end;
        } else {
            result.push(egc.clone());
        }
        prior_hard = hard || internal_stop;
    }
    Ok(result)
}

const fn tibetan_follower(cp: u32) -> bool {
    matches!(cp, 0x0f08 | 0x0f0b | 0x0f0c | 0x0f0d..=0x0f12 | 0x0f14)
}

fn base(atoms: &[OriginalAtom]) -> Option<&OriginalAtom> {
    let category = CodePointMapData::<GeneralCategory>::new();
    let grapheme = CodePointMapData::<GraphemeClusterBreak>::new();
    atoms
        .iter()
        .find(|atom| {
            category.get32(atom.code_point) != GeneralCategory::Format
                && grapheme.get32(atom.code_point) != GraphemeClusterBreak::Prepend
        })
        .or_else(|| unit_properties::first_non_format(atoms))
        .or_else(|| atoms.first())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Properties {
    category: GeneralCategory,
    width: EastAsianWidth,
    scripts: Vec<Script>,
}

fn scalar_properties(cp: u32) -> Result<Properties> {
    let set = ScriptWithExtensions::new().get_script_extensions_val32(cp);
    let mut scripts = Vec::new();
    for script in set.iter() {
        scripts.try_reserve(1).map_err(|_| capacity())?;
        scripts.push(script);
    }
    if scripts.is_empty() {
        scripts.try_reserve(1).map_err(|_| capacity())?;
        scripts.push(CodePointMapData::<Script>::new().get32(cp));
    }
    Ok(Properties {
        category: CodePointMapData::<GeneralCategory>::new().get32(cp),
        width: CodePointMapData::<EastAsianWidth>::new().get32(cp),
        scripts,
    })
}

fn properties(atoms: &[OriginalAtom]) -> Result<Properties> {
    let base = base(atoms).expect("a nonempty EGC has an actual scalar or opaque source unit");
    let base_index = atoms
        .iter()
        .position(|atom| std::ptr::eq(atom, base))
        .expect("actual EGC base");
    let base = scalar_properties(base.code_point)?;
    let common_enclosing = unit_properties::has_common_enclosing_mark(atoms);
    let first_mark = if !common_enclosing && base.category == GeneralCategory::SpaceSeparator {
        let categories = CodePointMapData::<GeneralCategory>::new();
        atoms[base_index + 1..]
            .iter()
            .find(|atom| GeneralCategoryGroup::Mark.contains(categories.get32(atom.code_point)))
            .map(|atom| scalar_properties(atom.code_point))
            .transpose()?
    } else {
        None
    };
    resolve_properties(base, common_enclosing, first_mark)
}

/// Pure Appendix E mapping; fact qualification remains in `properties`.
fn resolve_properties(
    base: Properties,
    common_enclosing: bool,
    first_mark: Option<Properties>,
) -> Result<Properties> {
    if common_enclosing {
        return scalar_properties(0xfffd);
    }
    if base.category == GeneralCategory::SpaceSeparator
        && let Some(mut mark) = first_mark
    {
        mark.category = GeneralCategory::ModifierSymbol;
        mark.width = base.width;
        return Ok(mark);
    }
    Ok(base)
}

fn style_indices(analysis: &Analysis, atoms: IndexRange<usize>) -> Result<Vec<usize>> {
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for (_, index) in &analysis.infos[atoms] {
        if seen.insert(*index) {
            result.try_reserve(1).map_err(|_| capacity())?;
            result.push(*index);
        }
    }
    Ok(result)
}

fn context(
    properties: &Properties,
    styles: &[AnalyzedStyleRun],
    indices: &[usize],
) -> std::result::Result<UprightUnitContext, UprightScriptFailure> {
    use UprightUnitRole::{IsolatedUnit, NormalVerticalContext};
    match properties.width {
        EastAsianWidth::Fullwidth | EastAsianWidth::Wide => {
            return Ok(UprightUnitContext::WidthOverride(NormalVerticalContext));
        }
        EastAsianWidth::Halfwidth => {
            return Ok(UprightUnitContext::WidthOverride(IsolatedUnit));
        }
        _ => {}
    }
    let unsupported = properties
        .scripts
        .iter()
        .filter(|&&script| intrinsic(script))
        .count();
    if unsupported == properties.scripts.len() {
        return Err(UprightScriptFailure::Intrinsic { resolved: None });
    }
    let explicit = explicit_script(&properties.scripts, styles, indices);
    if unsupported != 0 {
        return match explicit {
            Ok(script) if intrinsic(script) => Err(UprightScriptFailure::Intrinsic {
                resolved: Some(text_script(script)),
            }),
            Ok(script) => Ok(UprightUnitContext::ExplicitScript(role(script))),
            Err(reason) => Err(UprightScriptFailure::UnresolvedContext { reason }),
        };
    }
    if let Ok(script) = explicit {
        return Ok(UprightUnitContext::ExplicitScript(role(script)));
    }
    let first = role(properties.scripts[0]);
    if properties
        .scripts
        .iter()
        .all(|&script| role(script) == first)
    {
        Ok(UprightUnitContext::UniformScripts(first))
    } else {
        Ok(UprightUnitContext::MixedRoleFallback)
    }
}

fn explicit_script(
    scripts: &[Script],
    styles: &[AnalyzedStyleRun],
    indices: &[usize],
) -> std::result::Result<Script, UprightScriptContextFailure> {
    use UprightScriptContextFailure::{
        ConflictingScripts, MissingExplicitScript, NonMember, UnqualifiedSingleScript,
    };
    let parser = PropertyParser::<Script>::new();
    let names = PropertyNamesShort::<Script>::new();
    let mut selected = None;
    let mut conflict = false;
    for &index in indices {
        let style = &styles[index].style;
        let locale = style.font_locale().ok_or(MissingExplicitScript)?;
        let tag = locale.script().ok_or(MissingExplicitScript)?;
        let script = parser
            .get_strict_utf8(tag.as_bytes())
            .ok_or(UnqualifiedSingleScript)?;
        if matches!(script, Script::Common | Script::Inherited | Script::Unknown)
            || names.get(script) != Some(tag)
        {
            return Err(UnqualifiedSingleScript);
        }
        if !scripts.contains(&script) {
            return Err(NonMember);
        }
        if let Some(prior) = selected {
            conflict |= prior != script;
        } else {
            selected = Some(script);
        }
    }
    if conflict {
        Err(ConflictingScripts)
    } else {
        selected.ok_or(MissingExplicitScript)
    }
}

const fn intrinsic(script: Script) -> bool {
    matches!(
        script,
        Script::Mongolian | Script::PhagsPa | Script::Ogham | Script::OldTurkic
    )
}

const fn role(script: Script) -> UprightUnitRole {
    if matches!(
        script,
        Script::Bopomofo
            | Script::EgyptianHieroglyphs
            | Script::Hiragana
            | Script::Katakana
            | Script::Han
            | Script::Hangul
            | Script::MeroiticCursive
            | Script::MeroiticHieroglyphs
            | Script::Yi
    ) {
        UprightUnitRole::NormalVerticalContext
    } else {
        UprightUnitRole::IsolatedUnit
    }
}

fn text_script(script: Script) -> TextScript {
    TextScript::from_script(script).expect("actual ICU Script has a canonical ISO15924 short name")
}

fn diagnostic_scripts(scripts: &[Script]) -> Result<Vec<TextScript>> {
    let mut result = Vec::new();
    result.try_reserve(scripts.len()).map_err(|_| capacity())?;
    result.extend(scripts.iter().copied().map(text_script));
    result.sort_by_key(|script| script.iso15924());
    result.dedup();
    Ok(result)
}

fn diagnostic_contexts(
    styles: &[AnalyzedStyleRun],
    indices: &[usize],
) -> Result<Vec<UprightScriptContext>> {
    let mut result = Vec::new();
    result.try_reserve(indices.len()).map_err(|_| capacity())?;
    for &index in indices {
        let row = &styles[index];
        let owner = match row.owner {
            AnalyzedStyleOwner::Source(range) => TextStyleOwner::Source { range },
            AnalyzedStyleOwner::Control(occurrence) => TextStyleOwner::Control { occurrence },
            AnalyzedStyleOwner::Scope(scope) => TextStyleOwner::Scope { scope },
        };
        result.push(UprightScriptContext::new(
            owner,
            row.style.locale_tag().map(str::to_owned),
        ));
    }
    Ok(result)
}
