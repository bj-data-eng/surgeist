//! Shared Appendix-H character classification; each consumer owns its phase.

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup},
};

/// Native recognized repertoire, not an exhaustive CSS word-separator property.
pub(crate) fn separator(cp: u32) -> bool {
    cp == 0x200b
        || CodePointMapData::<GeneralCategory>::new().get32(cp) == GeneralCategory::SpaceSeparator
        || matches!(
            cp,
            0x1361 | 0x10100 | 0x10101 | 0x1039f | 0x1091f | 0x103d0 | 0x12470
        )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Punctuation {
    Other,
    Opening,
    Closing,
    Unfamiliar,
}

fn ordinary_suffix(cp: u32) -> bool {
    matches!(
        cp,
        0x2c | 0x2e
            | 0x3b
            | 0x3a
            | 0x21
            | 0x3f
            | 0x2025
            | 0x2026
            | 0x3001
            | 0x3002
            | 0xff01
            | 0xff0c
            | 0xff0e
            | 0xff1a
            | 0xff1b
            | 0xff1f
    )
}

fn closing_quote_context(cp: u32) -> bool {
    let category = CodePointMapData::<GeneralCategory>::new().get32(cp);
    GeneralCategoryGroup::Letter.contains(category)
        || GeneralCategoryGroup::Number.contains(category)
        || matches!(
            category,
            GeneralCategory::ClosePunctuation | GeneralCategory::FinalPunctuation
        )
        || ordinary_suffix(cp)
}

pub(crate) fn punctuation(values: &[u32], index: usize) -> Punctuation {
    let cp = values[index];
    let categories = CodePointMapData::<GeneralCategory>::new();
    let category = categories.get32(cp);
    if matches!(cp, 0x22 | 0x27) {
        return if index
            .checked_sub(1)
            .is_some_and(|previous| closing_quote_context(values[previous]))
        {
            Punctuation::Closing
        } else {
            Punctuation::Opening
        };
    }
    if matches!(
        category,
        GeneralCategory::OpenPunctuation | GeneralCategory::InitialPunctuation
    ) || matches!(cp, 0xa1 | 0xbf)
    {
        Punctuation::Opening
    } else if matches!(
        category,
        GeneralCategory::ClosePunctuation | GeneralCategory::FinalPunctuation
    ) || ordinary_suffix(cp)
    {
        Punctuation::Closing
    } else if GeneralCategoryGroup::Punctuation.contains(category) {
        Punctuation::Unfamiliar
    } else {
        Punctuation::Other
    }
}
