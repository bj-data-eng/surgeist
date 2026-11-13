//! Derived joins from node positions into the payload registry.

use super::*;

/// Derived joins into the actual payload registry, one cell per record
/// position. Dense record positions are append-only; a join never substitutes
/// for the row's location or ticket. `textarea` is rewritten as
/// its row is reset; `rows` names the first registry row of each
/// node-keyed metadata kind bound to the record, so a lookup visits one cell and
/// one row instead of scanning the registry. A join is written only for a row
/// that exists, and registry rows are appended but never removed or reordered
/// while the registry is live; rows of a joined kind are never replaced, so a
/// join always names a row of its kind (the callers still check what they get).
#[derive(Clone, Copy)]
pub(super) struct NodeJoins {
    pub(super) textarea: Option<NonZeroU32>,
    pub(super) rows: JoinTable<Option<NonZeroU32>>,
    /// The record's tree root as last found; see [`tree_root`].
    pub(super) tree_root: tree_root::TreeRootCell,
}

/// Node-keyed registry row kinds with a join in [`NodeJoins`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Join {
    NodeDocument,
    Template,
    FormAssociation,
    Script,
    Image,
    Hyperlink,
    Style,
    Dialog,
    Nonce,
    Input,
    SelectFamily,
    Base,
    BaseDocument,
    LookupRegistry,
    /// The `StyleOrder` row listing the style elements of the tree the record
    /// roots, while it is current; see `SessionStorage::style_order_floor`.
    StyleOrder,
    /// The pseudo-attribute map of a processing instruction.
    ProcessingInstructionAttributes,
    /// The state of an SVG `script` element.
    SvgScript,
    /// The notification domain of a `details` element.
    Details,
}

impl Join {
    /// Report preparation requires exactly one row of these kinds per record.
    pub(super) fn unique_per_record(self) -> bool {
        matches!(
            self,
            Self::Image | Self::Hyperlink | Self::Style | Self::Dialog
        )
    }
}

const JOIN_COUNT: usize = Join::Details as usize + 1;

/// One value per [`Join`]; a lookup cannot be out of range.
type JoinTable<T> = Table<Join, T, JOIN_COUNT>;

slots!(Join, JOIN_COUNT, {
    NodeDocument => 0,
    Template => 1,
    FormAssociation => 2,
    Script => 3,
    Image => 4,
    Hyperlink => 5,
    Style => 6,
    Dialog => 7,
    Nonce => 8,
    Input => 9,
    SelectFamily => 10,
    Base => 11,
    BaseDocument => 12,
    LookupRegistry => 13,
    StyleOrder => 14,
    ProcessingInstructionAttributes => 15,
    SvgScript => 16,
    Details => 17,
});

/// Cells hold a registry slot plus one in 32 bits; a slot beyond that is
/// refused as the registry invariant (see `joined_cell`).
const _: () = assert!(
    mem::size_of::<NodeJoins>() == 4 * (JOIN_COUNT + 1) + mem::size_of::<tree_root::TreeRootCell>()
);

impl Default for NodeJoins {
    fn default() -> Self {
        Self {
            textarea: None,
            rows: JoinTable::filled(None),
            tree_root: tree_root::TreeRootCell::default(),
        }
    }
}

impl NodeJoins {
    pub(super) fn get(&self, join: Join) -> Option<usize> {
        self.rows.copy(join).and_then(cell_slot)
    }
    /// The registry row holding the record's textarea state.
    pub(super) fn textarea(&self) -> Option<usize> {
        self.textarea.and_then(cell_slot)
    }
    /// Names the latest registry row, the one a back-to-front scan finds.
    pub(super) fn set_latest(&mut self, join: Join, slot: usize) -> Result<(), SessionError> {
        *self.rows.get_mut(join) = Some(joined_cell(slot)?);
        Ok(())
    }
    /// Forgets the registry row of `join`.
    pub(super) fn clear(&mut self, join: Join) {
        *self.rows.get_mut(join) = None;
    }
    /// Keeps the earliest registry row, the one a front-to-back scan finds, and
    /// reports whether the cell already named another row.
    pub(super) fn set_first(&mut self, join: Join, slot: usize) -> Result<bool, SessionError> {
        let cell = self.rows.get_mut(join);
        let slot = joined_cell(slot)?;
        let repeated = cell.is_some_and(|current| current != slot);
        if cell.is_none_or(|current| slot < current) {
            *cell = Some(slot);
        }
        Ok(repeated)
    }
}

/// Slot plus one as a nonzero 32-bit cell. A registry of more than `u32::MAX`
/// rows cannot be joined (it would hold over a terabyte of 320-byte rows), so
/// a larger slot is refused as the registry invariant, never truncated or
/// wrapped onto another row.
pub(super) fn joined_cell(slot: usize) -> Result<NonZeroU32, SessionError> {
    u32::try_from(slot)
        .ok()
        .and_then(|slot| slot.checked_add(1))
        .and_then(NonZeroU32::new)
        .ok_or_else(SessionError::registry_row)
}

/// The registry slot a cell names.
pub(super) fn cell_slot(cell: NonZeroU32) -> Option<usize> {
    usize::try_from(cell.get().checked_sub(1)?).ok()
}
