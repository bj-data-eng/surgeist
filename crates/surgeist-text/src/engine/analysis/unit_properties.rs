//! Shared raw-Unicode primitives for operation-specific typographic units.

use icu_properties::{
    CodePointMapData,
    props::{EastAsianWidth, GeneralCategory, GeneralCategoryGroup, GraphemeClusterBreak, Script},
    script::ScriptWithExtensions,
};

use crate::{Result, engine::OriginalAtom};

use super::capacity;

/// Appendix E's exception uses raw ScriptExtensions, not a shaping-run script.
pub(crate) fn has_common_enclosing_mark(atoms: &[OriginalAtom]) -> bool {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    atoms.iter().any(|atom| {
        categories.get32(atom.code_point) == GeneralCategory::EnclosingMark
            && scripts.has_script32(atom.code_point, Script::Common)
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

pub(super) fn base(atoms: &[OriginalAtom]) -> Option<&OriginalAtom> {
    let category = CodePointMapData::<GeneralCategory>::new();
    let grapheme = CodePointMapData::<GraphemeClusterBreak>::new();
    atoms
        .iter()
        .find(|atom| {
            category.get32(atom.code_point) != GeneralCategory::Format
                && grapheme.get32(atom.code_point) != GraphemeClusterBreak::Prepend
        })
        .or_else(|| first_non_format(atoms))
        .or_else(|| atoms.first())
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
pub(super) struct Properties {
    pub(super) category: GeneralCategory,
    pub(super) width: EastAsianWidth,
    pub(super) scripts: Vec<Script>,
}

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
        scripts,
    })
}

pub(super) fn properties(atoms: &[OriginalAtom]) -> Result<Properties> {
    let base = base(atoms).expect("a nonempty EGC has an actual scalar or opaque source unit");
    let base_index = atoms
        .iter()
        .position(|atom| std::ptr::eq(atom, base))
        .expect("actual EGC base");
    let base = scalar_properties(base.code_point)?;
    let common_enclosing = has_common_enclosing_mark(atoms);
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
