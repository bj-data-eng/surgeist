//! Complete processed lexical words and operation-specific CSS counting units.
use std::ops::Range;

use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup, Script},
    script::ScriptWithExtensions,
};

use crate::projection::SourceProjection;

/// Atom identities refer to the retained, unbroken processed projection. Neither
/// font/style runs nor a selected replacement supply another lexical spelling.
pub(crate) struct Words {
    rows: Vec<Word>,
    stops: Vec<usize>,
}
struct Word {
    atoms: Range<usize>,
    units: Vec<Range<usize>>,
    prefix: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CutCounts {
    pub(crate) total: usize,
    pub(crate) before: usize,
    pub(crate) after: usize,
}
impl CutCounts {
    pub(crate) fn admits_initial(self) -> bool {
        self.total >= 5 && self.before >= 2 && self.after >= 2
    }
}

impl Words {
    pub(crate) fn new(
        projection: &SourceProjection,
        graphemes: &[Range<usize>],
        words: Vec<Range<usize>>,
    ) -> Self {
        let rows = words
            .into_iter()
            .map(|atoms| {
                let first = graphemes.partition_point(|unit| unit.start < atoms.start);
                let last = graphemes.partition_point(|unit| unit.end <= atoms.end);
                let units = graphemes[first..last.max(first)].to_vec();
                let mut prefix = Vec::with_capacity(units.len() + 1);
                prefix.push(0);
                for unit in &units {
                    // At most one contribution per actual EGC, so the bounded
                    // vector length also bounds every prefix count.
                    prefix.push(
                        prefix.last().copied().unwrap()
                            + usize::from(counts(projection, unit.clone())),
                    );
                }
                Word {
                    atoms,
                    units,
                    prefix,
                }
            })
            .collect();
        let stops = graphemes
            .iter()
            .map(|unit| unit.start)
            .chain(std::iter::once(projection.atoms.len()))
            .collect();
        Self { rows, stops }
    }

    /// A genuine mixed-sequence cut lends its atom endpoint. A count map never
    /// turns an interior-EGC or word-exterior endpoint into an admitted cut.
    pub(crate) fn at(&self, atom: usize) -> Option<CutCounts> {
        self.stops.binary_search(&atom).ok()?;
        let word = self
            .rows
            .get(self.rows.partition_point(|word| word.atoms.end < atom))?;
        if atom < word.atoms.start || atom > word.atoms.end {
            return None;
        }
        let index = word.units.partition_point(|unit| unit.end <= atom);
        if word.units.get(index).is_some_and(|unit| unit.start < atom) {
            return None;
        }
        let total = *word.prefix.last()?;
        let before = word.prefix[index];
        Some(CutCounts {
            total,
            before,
            after: total - before,
        })
    }
}

fn counts(projection: &SourceProjection, unit: Range<usize>) -> bool {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    let atoms = &projection.atoms[unit];
    // Appendix E's Common enclosing-mark exception precedes the base category.
    if atoms.iter().any(|atom| {
        categories.get32(atom.code_point) == GeneralCategory::EnclosingMark
            && scripts.has_script32(atom.code_point, Script::Common)
    }) {
        return true;
    }
    // Native tailoring: formatting-only EGCs have no independent contribution;
    // embedded format controls neither add a unit nor disqualify its base.
    let Some(base) = atoms.iter().find(|atom| {
        atom.code_point != 0xad && categories.get32(atom.code_point) != GeneralCategory::Format
    }) else {
        return false;
    };
    let category = categories.get32(base.code_point);
    let category = if category == GeneralCategory::SpaceSeparator && atoms.len() > 1 {
        GeneralCategory::ModifierSymbol
    } else {
        category
    };
    category != GeneralCategory::NonspacingMark
        && !GeneralCategoryGroup::Punctuation.contains(category)
}

#[cfg(test)]
mod tests;
