use icu_normalizer::properties::{CanonicalDecomposition, Decomposed};
use icu_properties::props::{GeneralCategory, GeneralCategoryGroup};

use super::unit_properties;
use crate::{
    Error, ErrorCode, Range, Result, Source, Text, engine::OriginalAtom,
    projection::SourceProjection,
};

/// The adjacent UTF-16 ranges selected from an eligible line head.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirstLetterSelection {
    prefix: Option<Range>,
    letter: Range,
    suffix: Option<Range>,
}

impl FirstLetterSelection {
    /// Punctuation and intervening spaces preceding the first letter.
    #[must_use]
    pub const fn prefix(self) -> Option<Range> {
        self.prefix
    }

    /// The initial letter, number, or symbol unit.
    #[must_use]
    pub const fn letter(self) -> Range {
        self.letter
    }

    /// Punctuation and intervening spaces following the first letter.
    #[must_use]
    pub const fn suffix(self) -> Option<Range> {
        self.suffix
    }
}

impl Text {
    /// Selects first-letter text from the supplied eligible line head.
    ///
    /// Supply the containing block's content language as a Unicode locale
    /// identifier. An absent language uses the default grapheme units. The
    /// ranges are UTF-16 offsets into this text. An empty or nonmatching head
    /// returns `None`. Invalid locale syntax returns `ErrorCode::InvalidStyle`.
    pub fn first_letter(&self, language: Option<&str>) -> Result<Option<FirstLetterSelection>> {
        let dutch = uses_dutch_ij(language)?;
        let projection = SourceProjection::new(&Source::new(self.clone()))?;
        let mut remaining_atoms = projection.atoms.as_slice();
        let mut units = projection
            .graphemes
            .windows(2)
            .map(|edges| -> Result<Unit<'_>> {
                let range = Range::new(edges[0], edges[1]);
                let count = remaining_atoms.partition_point(|atom| atom.source.end <= range.end);
                let (atoms, rest) = remaining_atoms.split_at(count);
                remaining_atoms = rest;
                let base =
                    unit_properties::base(atoms).expect("a projected grapheme has a base atom");
                let category = unit_properties::visit_properties(atoms, |_| Ok(()))?.category;
                Ok(Unit {
                    range,
                    category,
                    base: base.code_point,
                    atoms,
                })
            });

        let Some(mut first) = units.next().transpose()? else {
            return Ok(None);
        };
        if first.punctuation() {
            loop {
                let Some(unit) = units.next().transpose()? else {
                    return Ok(None);
                };
                first = unit;
                if !first.punctuation() && !first.space() {
                    break;
                }
            }
        }
        if !first.initial() {
            return Ok(None);
        }

        let mut letter = first.range;
        let mut next = units.next().transpose()?;
        if dutch && let Some(j) = next.filter(|j| first.forms_dutch_ij(*j)) {
            letter.end = j.range.end;
            next = None;
        }

        let mut suffix_end = letter.end;
        for unit in next.into_iter().map(Ok).chain(units) {
            let unit = unit?;
            if unit.punctuation()
                && !matches!(
                    unit.category,
                    GeneralCategory::OpenPunctuation | GeneralCategory::DashPunctuation
                )
            {
                suffix_end = unit.range.end;
            } else if !unit.space() || unit_properties::is_declared_word_separator(unit.base) {
                break;
            }
        }

        Ok(Some(FirstLetterSelection {
            prefix: (letter.start != 0).then_some(Range::new(0, letter.start)),
            letter,
            suffix: (letter.end != suffix_end).then_some(Range::new(letter.end, suffix_end)),
        }))
    }
}

#[derive(Clone, Copy)]
struct Unit<'a> {
    range: Range,
    category: GeneralCategory,
    base: u32,
    atoms: &'a [OriginalAtom],
}

impl Unit<'_> {
    fn forms_dutch_ij(self, next: Self) -> bool {
        match (self.dutch_component(), next.dutch_component()) {
            (Some((i, i_stressed)), Some((j, j_stressed))) => {
                // A stress mark on i permits omission on j.
                matches!((i, j), ('I', 'J') | ('i', 'j')) && (i_stressed || !j_stressed)
            }
            _ => false,
        }
    }

    fn dutch_component(self) -> Option<(char, bool)> {
        match self.atoms {
            [base] => match base.code_point {
                0x00cd => Some(('I', true)),
                0x00ed => Some(('i', true)),
                code_point => Some((char::from_u32(code_point)?, false)),
            },
            [base, mark] => {
                let base = char::from_u32(base.code_point)?;
                let mark = char::from_u32(mark.code_point)?;
                let acute = mark == '\u{0301}'
                    || matches!(
                        CanonicalDecomposition::new().decompose(mark),
                        Decomposed::Singleton('\u{0301}')
                    );
                acute.then_some((base, true))
            }
            _ => None,
        }
    }

    fn initial(self) -> bool {
        GeneralCategoryGroup::Letter.contains(self.category)
            || GeneralCategoryGroup::Number.contains(self.category)
            || GeneralCategoryGroup::Symbol.contains(self.category)
    }

    fn punctuation(self) -> bool {
        GeneralCategoryGroup::Punctuation.contains(self.category)
    }

    fn space(self) -> bool {
        self.category == GeneralCategory::SpaceSeparator && self.base != 0x3000
    }
}

fn uses_dutch_ij(language: Option<&str>) -> Result<bool> {
    let Some(language) = language else {
        return Ok(false);
    };
    let locale: icu_locale::Locale = language.parse().map_err(|_| {
        Error::new(
            ErrorCode::InvalidStyle,
            "invalid first-letter content locale",
        )
    })?;
    Ok(locale.id.language.as_str() == "nl"
        && locale
            .id
            .script
            .is_none_or(|script| script.as_str() == "Latn"))
}
