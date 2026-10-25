//! Ordinary line assumptions for qualified Thai/Lao TCUs without lexical models.
//! These proposals never enter detected word ranges or cluster word boundaries.

use std::ops::Range;

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup, LineBreak, Script},
    script::ScriptWithExtensions,
};

use super::{
    line_break::MissingLexicalModels,
    style_at,
    unit_properties::{first_non_format, has_common_enclosing_mark},
};
use crate::{engine::resolve::AnalyzedStyleRun, projection::SourceProjection};

pub(super) fn propose(
    _missing_models: MissingLexicalModels,
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    units: &[Range<usize>],
    proposals: &mut [bool],
) {
    // The caller partitions already-built TCUs once across genuine components.
    // Sliding pairs retain one qualified unit, without rescanning earlier TCUs.
    let mut previous = None;
    for unit in units {
        let script = qualified_unit(projection, unit.clone());
        if let Some((end, Some(before_script), false)) = previous {
            let at = unit.start;
            if end == at
                && script == Some(before_script)
                && styles[style_at(projection, styles, at)]
                    .style
                    .authored()
                    .effective_breaking()
                    .lexical_line_breaking
                && !protected_edge(projection.atoms[at].code_point)
            {
                proposals[at] = true;
            }
        }
        previous = Some((unit.end, script, protected_ending(projection, unit.clone())));
    }
}

/// Appendix-E properties of a real Thai/Lao SA letter unit, not a block guess.
fn qualified_unit(projection: &SourceProjection, unit: Range<usize>) -> Option<Script> {
    let atoms = &projection.atoms[unit];
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    // A Common enclosing mark makes the complete TCU Other Symbol/Common.
    if has_common_enclosing_mark(atoms) {
        return None;
    }
    let base = first_non_format(atoms)?;
    let category = categories.get32(base.code_point);
    // This also excludes Zs-based sequences (Appendix E's Modifier Symbol),
    // pure marks, punctuation, symbols and formatting-only units.
    if !(GeneralCategoryGroup::Letter.contains(category)
        || GeneralCategoryGroup::Number.contains(category))
        || CodePointMapData::<LineBreak>::new().get32(base.code_point) != LineBreak::ComplexContext
    {
        return None;
    }
    [Script::Thai, Script::Lao]
        .into_iter()
        .find(|script| scripts.has_script32(base.code_point, *script))
}

fn protected_edge(code_point: u32) -> bool {
    matches!(
        CodePointMapData::<LineBreak>::new().get32(code_point),
        LineBreak::Glue | LineBreak::WordJoiner | LineBreak::ZWJ
    )
}

fn protected_ending(projection: &SourceProjection, unit: Range<usize>) -> bool {
    let classes = CodePointMapData::<LineBreak>::new();
    let categories = CodePointMapData::<GeneralCategory>::new();
    let mut effective = LineBreak::Alphabetic;
    let atoms = &projection.atoms[unit];
    for atom in atoms {
        let class = classes.get32(atom.code_point);
        // LB1 resolves SA marks to CM; LB9 then retains the preceding base.
        // In particular a GL mark followed by a Thai tone mark still carries GL.
        let combining = matches!(class, LineBreak::CombiningMark | LineBreak::ZWJ)
            || class == LineBreak::ComplexContext
                && matches!(
                    categories.get32(atom.code_point),
                    GeneralCategory::NonspacingMark | GeneralCategory::SpacingMark
                );
        if combining {
            if matches!(
                effective,
                LineBreak::MandatoryBreak
                    | LineBreak::CarriageReturn
                    | LineBreak::LineFeed
                    | LineBreak::NextLine
                    | LineBreak::Space
                    | LineBreak::ZWSpace
            ) {
                effective = LineBreak::Alphabetic;
            }
        } else {
            effective = class;
        }
    }
    matches!(effective, LineBreak::Glue | LineBreak::WordJoiner)
        || atoms
            .last()
            .is_some_and(|atom| classes.get32(atom.code_point) == LineBreak::ZWJ)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_glue_at_the_tcu_end_prevents_assumed_line_permission() {
        // Qualify an actual GL nonspacing mark from the pinned data. CGJ is CM
        // in this edition and must not be mislabeled as the required fixture.
        let classes = CodePointMapData::<LineBreak>::new();
        let categories = CodePointMapData::<GeneralCategory>::new();
        let graphemes = CodePointMapData::<icu_properties::props::GraphemeClusterBreak>::new();
        let glue_mark = classes
            .iter_ranges_for_value(LineBreak::Glue)
            .flatten()
            .find(|cp| {
                categories.get32(*cp) == GeneralCategory::NonspacingMark
                    && graphemes.get32(*cp) == icu_properties::props::GraphemeClusterBreak::Extend
            })
            .and_then(char::from_u32)
            .expect("pinned data has an actual combining GL fixture");
        // The following SA tone mark is CM under LB1 and retains GL under LB9,
        // so checking only the final mark's raw SA class would be wrong.
        for text in [
            format!("ก{glue_mark}ก"),
            format!("ก{glue_mark}\u{0e48}ก"),
            "ก\u{200d}ก".to_owned(),
        ] {
            let end = text.encode_utf16().count();
            let analyzed = crate::AnalyzedSource::new(
                crate::Source::new(text.clone()),
                crate::Style::default(),
            )
            .unwrap();
            assert_eq!(
                analyzed.graphemes().collect::<Vec<_>>(),
                [
                    crate::Range::new(0, end - 1),
                    crate::Range::new(end - 1, end)
                ]
            );
            assert!(
                analyzed
                    .inner
                    .analysis
                    .infos
                    .iter()
                    .all(|(info, _)| !info.lexical_fallback_before()),
                "{text:?}"
            );
        }
    }

    #[test]
    fn appendix_e_uses_the_letter_base_but_does_not_promote_space_or_pure_marks() {
        let scripts = ScriptWithExtensions::new();
        // This inherited Me is not the Common-script exception. Do not apply a
        // blanket enclosing-mark veto to a genuine Thai base's TCU.
        assert_eq!(
            CodePointMapData::<GeneralCategory>::new().get32(0x20dd),
            GeneralCategory::EnclosingMark
        );
        assert!(!scripts.has_script32(0x20dd, Script::Common));
        for (text, expected) in [
            ("ก\u{20dd}", Some(Script::Thai)),
            (" \u{0e48}", None),
            ("\u{0e48}", None),
            ("\u{200e}", None),
            ("\u{600}ก", Some(Script::Thai)),
            ("\u{1}", None),
        ] {
            let projection = SourceProjection::new(&crate::Source::new(text)).unwrap();
            assert_eq!(
                qualified_unit(&projection, 0..projection.atoms.len()),
                expected,
                "{text:?}"
            );
        }
        let projection =
            SourceProjection::new(&crate::Source::new(crate::Text::from_code_units(vec![
                0xd800, 0x301,
            ])))
            .unwrap();
        assert_eq!(projection.graphemes, [0, 2]);
        assert_eq!(qualified_unit(&projection, 0..2), None);
    }
}
