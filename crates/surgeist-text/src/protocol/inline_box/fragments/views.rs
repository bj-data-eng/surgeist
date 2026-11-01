//! Views of actual selected/finalized pieces; no free-standing public forest.
use super::*;
use crate::protocol::readback::FinalizedControlView;
use crate::protocol::selection::SelectedData;
use crate::{FinalizedLine, PreparedControl, PreparedParticipant, PresentationRange, SelectedLine};

/// Stable identity retains its actual prepared family, not a line/glyph ordinal.
#[derive(Clone)]
pub struct InlineBoxFragment {
    family: PreparedInlineBoxFamily,
    index: usize,
}
impl fmt::Debug for InlineBoxFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InlineBoxFragment")
            .field("owner", &self.owner())
            .finish_non_exhaustive()
    }
}
impl PartialEq for InlineBoxFragment {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.family == other.family
    }
}
impl Eq for InlineBoxFragment {}
impl InlineBoxFragment {
    pub fn owner(&self) -> InlineBoxSource {
        self.family
            .inner
            .source(self.family.inner.fragments[self.index].owner)
    }
    pub fn parent(&self) -> Option<Self> {
        self.family.inner.fragments[self.index]
            .parent
            .map(|index| Self {
                family: self.family.clone(),
                index,
            })
    }
    pub fn nearest_common_ancestor(&self, other: &Self) -> Result<Self> {
        if self.family != other.family {
            return Err(invalid(InlineBoxFragmentationError::ForeignFamily));
        }
        let data = &self.family.inner;
        if data.fragments[self.index].body != data.fragments[other.index].body {
            return Err(invalid(InlineBoxFragmentationError::DifferentBody));
        }
        let depth = |mut at: usize| {
            let mut depth = 0;
            while let Some(p) = data.fragments[at].parent {
                depth += 1;
                at = p;
            }
            depth
        };
        let (mut a, mut b) = (self.index, other.index);
        let (mut da, mut db) = (depth(a), depth(b));
        while da > db {
            a = data.fragments[a].parent.expect("known depth");
            da -= 1;
        }
        while db > da {
            b = data.fragments[b].parent.expect("known depth");
            db -= 1;
        }
        while a != b {
            a = data.fragments[a].parent.expect("one admitted body root");
            b = data.fragments[b].parent.expect("one admitted body root");
        }
        Ok(Self {
            family: self.family.clone(),
            index: a,
        })
    }
}

/// The owned unit carrying an actual piece incidence. Its packet can also carry
/// other original pieces; the unit's source range is not exclusive box custody.
/// Ancillary removed/empty/control aliases remain independently retained in
/// canonical Source/unit readback. This view is not an exhaustive or exclusive
/// assignment of every source alias to a global bidi fragment.
pub struct InlineBoxPieceMember<'a> {
    selected: &'a SelectedData,
    unit: usize,
}
impl InlineBoxPieceMember<'_> {
    pub fn participant(&self) -> Option<&PreparedParticipant> {
        self.selected.presentation.units[self.unit]
            .participant
            .map(|p| &self.selected.request.participants[p])
    }
    pub fn unit_source_range(&self) -> crate::Range {
        self.selected.presentation.units[self.unit].source
    }
    pub fn presentation_range(&self) -> PresentationRange {
        let r = self.selected.presentation.units[self.unit].presentation;
        PresentationRange {
            start: r.start,
            end: r.end,
        }
    }
}

/// A real retained finalized line and one of its actual selected pieces.
#[derive(Clone)]
pub struct FinalizedInlineBoxPiece {
    line: FinalizedLine,
    index: usize,
}
impl fmt::Debug for FinalizedInlineBoxPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FinalizedInlineBoxPiece")
            .field("fragment", &self.fragment())
            .finish_non_exhaustive()
    }
}
impl FinalizedInlineBoxPiece {
    fn data(&self) -> &super::project::Piece {
        &self
            .line
            .inner
            .selected
            .basis
            .inline_boxes
            .as_ref()
            .expect("issued piece has selected projection")
            .pieces[self.index]
    }
    pub fn finalized_line(&self) -> &FinalizedLine {
        &self.line
    }
    pub fn family(&self) -> &PreparedInlineBoxFamily {
        self.line
            .inline_box_family()
            .expect("issued piece retains family")
    }
    pub fn fragment(&self) -> InlineBoxFragment {
        InlineBoxFragment {
            family: self.family().clone(),
            index: self.data().fragment,
        }
    }
    pub fn owner(&self) -> InlineBoxSource {
        self.fragment().owner()
    }
    pub fn members(&self) -> impl ExactSizeIterator<Item = InlineBoxPieceMember<'_>> {
        let selected = self.line.inner.selected.as_ref();
        let data = selected
            .basis
            .inline_boxes
            .as_ref()
            .expect("issued projection");
        data.members[self.data().members.clone()]
            .iter()
            .map(move |member| InlineBoxPieceMember {
                selected,
                unit: member.unit,
            })
    }
    pub fn start_control(&self) -> Option<PreparedControl> {
        self.control(self.data().start)
    }
    pub fn end_control(&self) -> Option<PreparedControl> {
        self.control(self.data().end)
    }
    fn control(&self, index: Option<usize>) -> Option<PreparedControl> {
        index.map(|index| PreparedControl {
            prepared: Arc::clone(&self.line.inner.selected.prepared),
            index,
        })
    }
    pub fn start_edge(&self) -> Option<FinalizedControlView<'_>> {
        let c = self.start_control()?;
        self.line.control_slots().find(|v| v.control() == c)
    }
    pub fn end_edge(&self) -> Option<FinalizedControlView<'_>> {
        let c = self.end_control()?;
        self.line.control_slots().find(|v| v.control() == c)
    }
    pub fn shares_packet_with(&self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.line.inner, &other.line.inner) {
            return false;
        }
        let (a, b) = (self.data(), other.data());
        a.packets.start < b.packets.end && b.packets.start < a.packets.end
    }
}
/// A retained selected piece before allocation-dependent execution.
#[derive(Clone)]
pub struct SelectedInlineBoxPiece {
    line: SelectedLine,
    index: usize,
}
impl SelectedInlineBoxPiece {
    fn data(&self) -> &super::project::Piece {
        &self
            .line
            .inner
            .basis
            .inline_boxes
            .as_ref()
            .expect("issued selected piece")
            .pieces[self.index]
    }
    pub fn selected_line(&self) -> &SelectedLine {
        &self.line
    }
    pub fn family(&self) -> &PreparedInlineBoxFamily {
        self.line
            .inline_box_family()
            .expect("issued selected family")
    }
    pub fn fragment(&self) -> InlineBoxFragment {
        InlineBoxFragment {
            family: self.family().clone(),
            index: self.data().fragment,
        }
    }
    pub fn owner(&self) -> InlineBoxSource {
        self.fragment().owner()
    }
    pub fn members(&self) -> impl ExactSizeIterator<Item = InlineBoxPieceMember<'_>> {
        let selected = self.line.inner.as_ref();
        let data = selected
            .basis
            .inline_boxes
            .as_ref()
            .expect("issued projection");
        data.members[self.data().members.clone()]
            .iter()
            .map(move |m| InlineBoxPieceMember {
                selected,
                unit: m.unit,
            })
    }
}
impl fmt::Debug for SelectedInlineBoxPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectedInlineBoxPiece")
            .field("fragment", &self.fragment())
            .finish_non_exhaustive()
    }
}
impl SelectedLine {
    pub fn inline_box_pieces(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = SelectedInlineBoxPiece> + '_> {
        let data = self.inner.basis.inline_boxes.as_ref()?;
        Some((0..data.pieces.len()).map(|index| SelectedInlineBoxPiece {
            line: self.clone(),
            index,
        }))
    }
    pub fn inline_box_family(&self) -> Option<&PreparedInlineBoxFamily> {
        self.inner.request.inline_boxes.as_ref()
    }
}
impl FinalizedLine {
    pub fn inline_box_family(&self) -> Option<&PreparedInlineBoxFamily> {
        self.inner.selected.request.inline_boxes.as_ref()
    }
    pub fn inline_box_pieces(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = FinalizedInlineBoxPiece> + '_> {
        let data = self.inner.selected.basis.inline_boxes.as_ref()?;
        Some((0..data.pieces.len()).map(|index| FinalizedInlineBoxPiece {
            line: self.clone(),
            index,
        }))
    }
}

/// Borrowed from the sole real placed-line frame; no rectangle is recomputed.
pub struct PlacedInlineBoxPiece<'a> {
    line: &'a crate::engine::PlacedLine,
    line_index: usize,
    index: usize,
}
impl PlacedInlineBoxPiece<'_> {
    pub fn line_index(&self) -> usize {
        self.line_index
    }
    pub fn family(&self) -> &PreparedInlineBoxFamily {
        self.line
            .finalized
            .selected
            .request
            .inline_boxes
            .as_ref()
            .expect("issued placed piece has family")
    }
    pub fn fragment(&self) -> InlineBoxFragment {
        self.finalized_piece().fragment()
    }
    pub fn finalized_piece(&self) -> FinalizedInlineBoxPiece {
        FinalizedInlineBoxPiece {
            line: FinalizedLine {
                inner: Arc::clone(&self.line.finalized),
            },
            index: self.index,
        }
    }
    pub fn anchor(&self) -> crate::Point {
        self.line.anchor
    }
    pub fn orientation(&self) -> crate::LineOrientation {
        self.line.orientation
    }
}
impl crate::Layout {
    pub fn inline_box_pieces(&self) -> impl Iterator<Item = PlacedInlineBoxPiece<'_>> {
        self.inner
            .lines
            .iter()
            .enumerate()
            .flat_map(|(line_index, line)| {
                let count = line
                    .finalized
                    .selected
                    .basis
                    .inline_boxes
                    .as_ref()
                    .map_or(0, |p| p.pieces.len());
                (0..count).map(move |index| PlacedInlineBoxPiece {
                    line,
                    line_index,
                    index,
                })
            })
    }
}
