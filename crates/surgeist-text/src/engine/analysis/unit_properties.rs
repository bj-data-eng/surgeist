//! Shared raw-Unicode primitives for operation-specific typographic units.

use icu_properties::{
    CodePointMapData, CodePointSetData,
    props::{
        DefaultIgnorableCodePoint, EastAsianWidth, ExtendedPictographic, GeneralCategory,
        GeneralCategoryGroup, GraphemeClusterBreak, IndicConjunctBreak, Script,
        VerticalOrientation,
    },
    script::ScriptWithExtensions,
};

use crate::{Result, engine::OriginalAtom};

use super::capacity;

/// Appendix E's exception uses raw ScriptExtensions, not a shaping-run script.
pub(crate) fn has_common_enclosing_mark(atoms: &[OriginalAtom]) -> bool {
    has_common_enclosing_mark_in(atoms.iter().map(|atom| atom.code_point))
}

/// The same raw Unicode predicate for source-only transformed scalar streams.
pub(crate) fn has_common_enclosing_mark_in(values: impl IntoIterator<Item = u32>) -> bool {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    values.into_iter().any(|code_point| {
        categories.get32(code_point) == GeneralCategory::EnclosingMark
            && scripts.has_script32(code_point, Script::Common)
    })
}

/// Native unit operations ignore raw Cf when locating their base. Other marks,
/// native visible Cc, and opaque original UTF-16 units keep their own identity.
pub(crate) fn first_non_format(atoms: &[OriginalAtom]) -> Option<&OriginalAtom> {
    let categories = CodePointMapData::<GeneralCategory>::new();
    atoms
        .iter()
        .find(|atom| categories.get32(atom.code_point) != GeneralCategory::Format)
}

fn base_tier(atom: &OriginalAtom) -> u8 {
    let category = CodePointMapData::<GeneralCategory>::new().get32(atom.code_point);
    if category == GeneralCategory::Format {
        2
    } else if CodePointMapData::<GraphemeClusterBreak>::new().get32(atom.code_point)
        == GraphemeClusterBreak::Prepend
    {
        1
    } else {
        0
    }
}

pub(in crate::engine) fn base(atoms: &[OriginalAtom]) -> Option<&OriginalAtom> {
    atoms
        .iter()
        .enumerate()
        .min_by_key(|(index, atom)| (base_tier(atom), *index))
        .map(|(_, atom)| atom)
}

/// Possible governors under arbitrary final DI exclusions, within this original
/// EGC. A mandatory candidate stops its tier and makes lower tiers unreachable.
/// This predicts operands only; it neither excludes input nor segments text.
pub(in crate::engine) enum BaseVisit {
    Inspect,
    Candidate(usize),
}

pub(in crate::engine) fn visit_possible_bases(
    atoms: &[OriginalAtom],
    mut visit: impl FnMut(BaseVisit) -> Result<()>,
) -> Result<()> {
    let di = CodePointSetData::new::<DefaultIgnorableCodePoint>();
    for tier in 0..3 {
        for (index, atom) in atoms.iter().enumerate() {
            visit(BaseVisit::Inspect)?;
            if base_tier(atom) != tier {
                continue;
            }
            visit(BaseVisit::Candidate(index))?;
            if !di.contains32(atom.code_point) {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Proof fence for source-conditioned planning, using the canonical projection
/// representative (visible Cc and opaque UTF-16 have explicit representatives).
/// This does not introduce a grapheme boundary or classify a final unit.
pub(in crate::engine) fn mandatory_ordinary_anchor(
    atom: &OriginalAtom,
    representative: char,
) -> bool {
    !CodePointSetData::new::<DefaultIgnorableCodePoint>().contains32(atom.code_point)
        && base_tier(atom) == 0
        && CodePointMapData::<GraphemeClusterBreak>::new().get(representative)
            == GraphemeClusterBreak::Other
        && !CodePointSetData::new::<ExtendedPictographic>().contains(representative)
        && CodePointMapData::<IndicConjunctBreak>::new().get(representative)
            != IndicConjunctBreak::Consonant
}

/// A fixed governor still needs stable Appendix-E exceptions before its current
/// script/category can prove an invariant spacing rule for every DI subset.
pub(in crate::engine) fn di_properties_invariant(atoms: &[OriginalAtom]) -> bool {
    let Some(base) = base(atoms) else {
        return false;
    };
    let categories = CodePointMapData::<GeneralCategory>::new();
    let di = CodePointSetData::new::<DefaultIgnorableCodePoint>();
    atoms
        .iter()
        .filter(|a| di.contains32(a.code_point))
        .all(|a| {
            let category = categories.get32(a.code_point);
            category != GeneralCategory::EnclosingMark
                && !(categories.get32(base.code_point) == GeneralCategory::SpaceSeparator
                    && GeneralCategoryGroup::Mark.contains(category))
        })
}

/// Allocation-free category query over the same Appendix E property source.
/// Neither exception promotes a nonletter base: Common Me produces So, while
/// Zs-based sequences produce Sk. A Letter/Number base can therefore only be
/// rejected by the Common-Me exception; its ScriptExtensions need no Vec.
pub(super) fn is_typographic_letter(atoms: &[OriginalAtom]) -> bool {
    base(atoms).is_some_and(|base| {
        let category = CodePointMapData::<GeneralCategory>::new().get32(base.code_point);
        (GeneralCategoryGroup::Letter.contains(category)
            || GeneralCategoryGroup::Number.contains(category))
            && !has_common_enclosing_mark(atoms)
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::engine) struct Properties {
    pub(in crate::engine) category: GeneralCategory,
    pub(super) width: EastAsianWidth,
    pub(in crate::engine) vertical_orientation: VerticalOrientation,
    pub(in crate::engine) scripts: Vec<Script>,
}

pub(in crate::engine) fn vertical_orientation(cp: u32) -> VerticalOrientation {
    CodePointMapData::<VerticalOrientation>::new().get32(cp)
}

#[cfg(test)]
pub(super) fn scalar_properties(cp: u32) -> Result<Properties> {
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
        vertical_orientation: vertical_orientation(cp),
        scripts,
    })
}

pub(in crate::engine) fn properties(atoms: &[OriginalAtom]) -> Result<Properties> {
    let mut scripts = Vec::new();
    let facts = visit_properties(atoms, |visit| {
        if let PropertyVisit::Script(script) = visit {
            scripts.try_reserve(1).map_err(|_| capacity())?;
            scripts.push(script);
        }
        Ok(())
    })?;
    Ok(Properties {
        category: facts.category,
        width: facts.width,
        vertical_orientation: facts.vertical_orientation,
        scripts,
    })
}

pub(in crate::engine) enum PropertyVisit {
    Inspect,
    Script(Script),
}

/// Raw facts and script emission share the canonical Appendix-E decision. The
/// observer admits its own storage before receiving a script, without making
/// this Unicode owner depend on an operation's resource policy.
pub(in crate::engine) fn visit_properties(
    atoms: &[OriginalAtom],
    mut visit: impl FnMut(PropertyVisit) -> Result<()>,
) -> Result<PropertyFacts> {
    let mut selected = None;
    for (index, atom) in atoms.iter().enumerate() {
        visit(PropertyVisit::Inspect)?;
        let tier = base_tier(atom);
        if selected.is_none_or(|(_, old)| tier < old) {
            selected = Some((index, tier));
        }
    }
    let index = selected
        .expect("a nonempty EGC has an actual source unit")
        .0;
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    visit(PropertyVisit::Inspect)?;
    let base = atoms[index].code_point;
    let mut target = base;
    let mut category = categories.get32(base);
    let mut common = false;
    for atom in atoms {
        visit(PropertyVisit::Inspect)?;
        if categories.get32(atom.code_point) == GeneralCategory::EnclosingMark
            && scripts.has_script32(atom.code_point, Script::Common)
        {
            common = true;
            break;
        }
    }
    let mut width = CodePointMapData::<EastAsianWidth>::new().get32(base);
    if common {
        target = 0xfffd;
        category = categories.get32(target);
        width = CodePointMapData::<EastAsianWidth>::new().get32(target);
    } else if category == GeneralCategory::SpaceSeparator {
        for atom in &atoms[index + 1..] {
            visit(PropertyVisit::Inspect)?;
            if GeneralCategoryGroup::Mark.contains(categories.get32(atom.code_point)) {
                target = atom.code_point;
                category = GeneralCategory::ModifierSymbol;
                break;
            }
        }
    }
    visit(PropertyVisit::Inspect)?;
    let mut any = false;
    for script in scripts.get_script_extensions_val32(target).iter() {
        visit(PropertyVisit::Script(script))?;
        any = true;
    }
    if !any {
        visit(PropertyVisit::Script(
            CodePointMapData::<Script>::new().get32(target),
        ))?;
    }
    Ok(PropertyFacts {
        category,
        width,
        vertical_orientation: vertical_orientation(target),
    })
}

pub(in crate::engine) struct PropertyFacts {
    pub(in crate::engine) category: GeneralCategory,
    pub(in crate::engine) width: EastAsianWidth,
    pub(in crate::engine) vertical_orientation: VerticalOrientation,
}

/// Pure Appendix E mapping; fact qualification remains in `properties`.
#[cfg(test)]
pub(super) fn resolve_properties(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Source, Text, projection::SourceProjection};

    #[test]
    fn letter_query_matches_complete_appendix_e_category_resolution() {
        for (text, expected) in [
            ("A", true),
            ("א\u{5b0}", true),
            ("\u{1b05}", true),
            ("\u{2160}", true),
            ("\u{bc}", true),
            ("א\u{20dd}", true),
            ("\u{600}א", true),
            ("\u{301}", false),
            ("\u{202f}", false),
            ("\u{202f}\u{363}", false),
            ("\u{3000}\u{363}", false),
            ("(", false),
            ("\u{61c}", false),
        ] {
            let projection = SourceProjection::new(&Source::new(text)).unwrap();
            let category = properties(&projection.atoms).unwrap().category;
            assert_eq!(
                is_typographic_letter(&projection.atoms),
                GeneralCategoryGroup::Letter.contains(category)
                    || GeneralCategoryGroup::Number.contains(category),
                "{text:?}"
            );
            assert_eq!(
                is_typographic_letter(&projection.atoms),
                expected,
                "{text:?}"
            );
        }
    }

    #[test]
    fn unicode_seventeen_enclosing_marks_do_not_supply_a_common_script_witness() {
        let categories = CodePointMapData::<GeneralCategory>::new();
        let scripts = ScriptWithExtensions::new();
        assert_eq!(categories.get32(0x20dd), GeneralCategory::EnclosingMark);
        assert_eq!(
            CodePointMapData::<Script>::new().get32(0x20dd),
            Script::Inherited
        );
        for cp in categories
            .iter_ranges_for_value(GeneralCategory::EnclosingMark)
            .flatten()
        {
            assert!(!scripts.has_script32(cp, Script::Common), "U+{cp:04X}");
        }
        // An inherited enclosing mark must not erase the actual Thai base.
        let projection = SourceProjection::new(&Source::new("ก\u{20dd}")).unwrap();
        assert_eq!(projection.graphemes, [0, 2]);
        assert!(!has_common_enclosing_mark(&projection.atoms));
        assert_eq!(
            first_non_format(&projection.atoms).unwrap().code_point,
            0xe01
        );
    }

    #[test]
    fn raw_format_skipping_preserves_first_marks_controls_and_opaque_units() {
        // SHY's existing explicit exclusion is exactly its raw Cf property.
        assert_eq!(
            CodePointMapData::<GeneralCategory>::new().get32(0xad),
            GeneralCategory::Format
        );
        for (units, expected) in [
            (vec![0x600, 0xe01], Some(0xe01)),
            (vec![0x61c, 0xad, 0x200e], None),
            (vec![0x301], Some(0x301)),
            (vec![0x903], Some(0x903)),
            (vec![0x1], Some(0x1)),
            (vec![0xd800, 0x301], Some(0xd800)),
        ] {
            let projection =
                SourceProjection::new(&Source::new(Text::from_code_units(units))).unwrap();
            assert_eq!(
                first_non_format(&projection.atoms).map(|atom| atom.code_point),
                expected
            );
            assert!(!has_common_enclosing_mark(&projection.atoms));
        }
        assert_eq!(
            CodePointMapData::<GeneralCategory>::new().get32(0x1),
            GeneralCategory::Control
        );
        assert_eq!(
            CodePointMapData::<GeneralCategory>::new().get32(0xd800),
            GeneralCategory::Surrogate
        );
    }
}
