//! Complete selected typographic units and their effective punctuation properties.

use std::ops::Range;

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory as Category, GeneralCategoryGroup},
};

use crate::engine::{
    OriginalAtom,
    analysis::unit_properties::{first_non_format, has_common_enclosing_mark},
};
use crate::protocol::{PresentationRange, selection::SelectedData};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Reclassification {
    CommonEnclosingMark,
    SpaceAndMark,
}

#[derive(Clone, Copy, Debug)]
struct Properties {
    category: Category,
    code_point: u32,
    base: usize,
    reclassification: Option<Reclassification>,
}

impl Properties {
    fn literal(self) -> Option<u32> {
        self.reclassification.is_none().then_some(self.code_point)
    }
}

fn properties(atoms: &[OriginalAtom]) -> Option<Properties> {
    let base = first_non_format(atoms)?;
    let index = atoms.iter().position(|atom| std::ptr::eq(atom, base))?;
    let categories = CodePointMapData::<Category>::new();
    let category = categories.get32(base.code_point);
    let reclassification = if has_common_enclosing_mark(atoms) {
        Some(Reclassification::CommonEnclosingMark)
    } else if category == Category::SpaceSeparator
        && atoms[index + 1..]
            .iter()
            .any(|atom| GeneralCategoryGroup::Mark.contains(categories.get32(atom.code_point)))
    {
        Some(Reclassification::SpaceAndMark)
    } else {
        None
    };
    Some(Properties {
        category: match reclassification {
            Some(Reclassification::CommonEnclosingMark) => Category::OtherSymbol,
            Some(Reclassification::SpaceAndMark) => Category::ModifierSymbol,
            None => category,
        },
        code_point: base.code_point,
        base: index,
        reclassification,
    })
}

#[derive(Clone, Debug)]
pub(super) struct Unit {
    pub(super) atoms: Range<usize>,
    pub(super) presentation: PresentationRange,
    pub(super) style: usize,
    pub(super) first: bool,
    pub(super) last: bool,
    pub(super) stop: bool,
}

pub(super) fn units(selected: &SelectedData) -> Vec<Unit> {
    selected
        .presentation
        .analysis
        .graphemes
        .iter()
        .filter_map(|range| {
            let atoms = &selected.presentation.projection.atoms[range.clone()];
            let props = properties(atoms)?;
            let literal_quote = matches!(props.literal(), Some(0x22 | 0x27));
            Some(Unit {
                atoms: range.clone(),
                presentation: PresentationRange {
                    start: atoms.first()?.presentation.start,
                    end: atoms.last()?.presentation.end,
                },
                style: selected.presentation.analysis.infos[range.start + props.base].1,
                first: matches!(
                    props.category,
                    Category::OpenPunctuation
                        | Category::FinalPunctuation
                        | Category::InitialPunctuation
                ) || literal_quote
                    || props.literal() == Some(0x3000),
                last: matches!(
                    props.category,
                    Category::ClosePunctuation
                        | Category::FinalPunctuation
                        | Category::InitialPunctuation
                ) || literal_quote,
                stop: props.literal().is_some_and(stop),
            })
        })
        .collect()
}

fn stop(cp: u32) -> bool {
    matches!(
        cp,
        0x2c | 0x2e
            | 0x60c
            | 0x6d4
            | 0x3001
            | 0x3002
            | 0xff0c
            | 0xff0e
            | 0xfe50
            | 0xfe51
            | 0xfe52
            | 0xff61
            | 0xff64
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Source, projection::SourceProjection};

    #[test]
    fn effective_units_keep_marked_literals_but_reclassify_a_real_space_mark_sequence() {
        for (text, category, literal, reclassified) in [
            (".\u{20dd}", Category::OtherPunctuation, Some(0x2e), false),
            ("'\u{20dd}", Category::OtherPunctuation, Some(0x27), false),
            ("(\u{301}", Category::OpenPunctuation, Some(0x28), false),
            ("\u{3000}", Category::SpaceSeparator, Some(0x3000), false),
            ("\u{3000}\u{301}", Category::ModifierSymbol, None, true),
        ] {
            let projection = SourceProjection::new(&Source::new(text)).unwrap();
            assert_eq!(projection.graphemes.len(), 2, "{text:?}");
            let props = properties(&projection.atoms).unwrap();
            assert_eq!(props.category, category, "{text:?}");
            assert_eq!(props.literal(), literal, "{text:?}");
            assert_eq!(props.reclassification.is_some(), reclassified, "{text:?}");
        }
        let projection = SourceProjection::new(&Source::new("\u{200e}")).unwrap();
        assert!(properties(&projection.atoms).is_none());
    }

    #[test]
    fn exactly_the_pinned_thirteen_stops_and_commas_are_selected() {
        let expected = [
            0x2c, 0x2e, 0x60c, 0x6d4, 0x3001, 0x3002, 0xff0c, 0xff0e, 0xfe50, 0xfe51, 0xfe52,
            0xff61, 0xff64,
        ];
        assert_eq!((0..=0x10ffff).filter(|&cp| stop(cp)).count(), 13);
        assert!(expected.into_iter().all(stop));
        for cp in [0x3b, 0x21, 0x3f, 0x2026, 0x2014] {
            assert!(!stop(cp));
        }
    }
}
