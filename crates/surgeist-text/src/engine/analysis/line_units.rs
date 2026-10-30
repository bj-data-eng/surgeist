//! Operation-specific line indivisibility over unchanged Unicode17 EGCs.

use std::ops::Range;

use icu_properties::{
    CodePointMapData,
    props::{IndicConjunctBreak, Script},
    script::ScriptWithExtensions,
};

use crate::projection::SourceProjection;

/// Restrictions only: an unsuppressed edge still needs genuine line permission.
#[derive(Clone)]
pub(crate) struct LineBreakUnits {
    suppressed: Vec<usize>,
    cells: Vec<Range<usize>>,
}

impl LineBreakUnits {
    pub(crate) fn new(
        projection: &SourceProjection,
        combination: Option<&crate::transform::combined::CombinedOutput>,
    ) -> crate::Result<Self> {
        let properties = CodePointMapData::<IndicConjunctBreak>::new();
        let scripts = ScriptWithExtensions::new();
        let mut barriers = projection.barrier_atoms.clone();
        barriers.sort_unstable();
        barriers.dedup();
        let mut barriers = barriers.into_iter().peekable();
        let mut egcs = projection.graphemes.iter().copied().peekable();
        let mut follows_coeng = false;
        let mut suppressed = Vec::new();
        for (index, atom) in projection.atoms.iter().enumerate() {
            while barriers.peek().is_some_and(|barrier| *barrier <= index) {
                follows_coeng = false;
                barriers.next();
            }
            while egcs
                .peek()
                .is_some_and(|edge| *edge < atom.presentation.start)
            {
                egcs.next();
            }
            let property = properties.get32(atom.code_point);
            if follows_coeng
                && property == IndicConjunctBreak::Consonant
                && scripts.has_script32(atom.code_point, Script::Khmer)
                && egcs.peek() == Some(&atom.presentation.start)
            {
                suppressed.try_reserve(1).map_err(|_| super::capacity())?;
                suppressed.push(index);
            }
            // The adopted restriction is COENG Extend* before a Khmer
            // consonant, not the global newer-Unicode grapheme rule.
            follows_coeng = atom.code_point == 0x17d2
                || follows_coeng && property == IndicConjunctBreak::Extend;
        }
        // Retain cell indivisibility separately: before/after CSS class
        // conversion must still consume the actual content's own TCUs.
        let cells = super::storage::collect(
            combination
                .into_iter()
                .flat_map(|output| output.bindings.iter().map(|binding| binding.atoms.clone())),
        )?;
        Ok(Self { suppressed, cells })
    }

    pub(crate) fn permits_edge(&self, atom: usize) -> bool {
        let next = self.cells.partition_point(|cell| cell.start < atom);
        self.suppressed.binary_search(&atom).is_err()
            && !next
                .checked_sub(1)
                .is_some_and(|index| atom < self.cells[index].end)
    }

    /// Whole outer line units, including actual completed cell restrictions.
    pub(crate) fn ranges<'a>(
        &'a self,
        graphemes: &'a [Range<usize>],
    ) -> impl Iterator<Item = Range<usize>> + 'a {
        let mut units = self.content_ranges(graphemes).peekable();
        std::iter::from_fn(move || {
            let mut unit = units.next()?;
            while !self.permits_edge(unit.end)
                && units.peek().is_some_and(|next| next.start == unit.end)
            {
                unit.end = units.next().expect("actual adjacent content unit").end;
            }
            Some(unit)
        })
    }

    /// Merge whole adjacent EGCs; do not resegment or drop any of their marks.
    pub(crate) fn content_ranges<'a>(
        &'a self,
        graphemes: &'a [Range<usize>],
    ) -> impl Iterator<Item = Range<usize>> + 'a {
        let first = graphemes.first().map_or(0, |unit| unit.start);
        let start = self.suppressed.partition_point(|edge| *edge <= first);
        let mut suppressed = self.suppressed[start..].iter().copied().peekable();
        let mut units = graphemes.iter().peekable();
        std::iter::from_fn(move || {
            let mut unit = units.next()?.clone();
            loop {
                while suppressed.peek().is_some_and(|edge| *edge < unit.end) {
                    suppressed.next();
                }
                if suppressed.peek() != Some(&unit.end)
                    || !units.peek().is_some_and(|next| next.start == unit.end)
                {
                    break;
                }
                unit.end = units.next().expect("actual adjacent EGC").end;
                suppressed.next();
            }
            Some(unit)
        })
    }
}
