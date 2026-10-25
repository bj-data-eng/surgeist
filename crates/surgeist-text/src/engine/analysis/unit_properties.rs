//! Shared raw-Unicode primitives for operation-specific typographic units.

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, Script},
    script::ScriptWithExtensions,
};

use crate::engine::OriginalAtom;

/// Appendix E's exception uses raw ScriptExtensions, not a shaping-run script.
pub(super) fn has_common_enclosing_mark(atoms: &[OriginalAtom]) -> bool {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    atoms.iter().any(|atom| {
        categories.get32(atom.code_point) == GeneralCategory::EnclosingMark
            && scripts.has_script32(atom.code_point, Script::Common)
    })
}

/// Native unit operations ignore raw Cf when locating their base. Other marks,
/// native visible Cc, and opaque original UTF-16 units keep their own identity.
pub(super) fn first_non_format(atoms: &[OriginalAtom]) -> Option<&OriginalAtom> {
    let categories = CodePointMapData::<GeneralCategory>::new();
    atoms
        .iter()
        .find(|atom| categories.get32(atom.code_point) != GeneralCategory::Format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Source, Text, projection::SourceProjection};

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
