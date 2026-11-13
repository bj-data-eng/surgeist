//! The bank of rows that a row log and a family table keep their rows in, and the admission and
//! release of those rows.
//!
//! A row is admitted before it is written: the bank has room for it and for every row admitted
//! and not yet written, and its auxiliary entry is acquired. Writing a row takes room that was
//! admitted. The release returns the bank and one entry for every row written and every row
//! admitted and not written. A type that keeps something beside the rows, such as the keys of a
//! family table, reserves and releases it through the closures its admission and release take, at
//! the point of the sequence where it belongs.

use super::session_error::latch_registry_row;
use super::*;
use crate::parser::control::VectorCapacity;

pub(super) struct RowBank<T> {
    rows: Vec<T>,
    allocation: VectorCapacity<T>,
    /// The rows [`Self::admit`] has acquired entries for and [`Self::push`] has not written.
    admitted: usize,
}

impl<T> RowBank<T> {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        let (rows, allocation) = control.new_vector()?;
        Ok(Self {
            rows,
            allocation,
            admitted: 0,
        })
    }
    /// Admits `additional` rows: the bank has room for them and for every row admitted and not
    /// yet written, and their auxiliary entries are acquired. A refusal leaves nothing admitted
    /// that this call added. In a test build the call is counted as a prepared slot, at its
    /// caller, so that a stop between the admission and the write is exercised.
    #[cfg_attr(test, track_caller)]
    pub(super) fn admit(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        self.admit_beside(control, additional, |_, _| Ok(()))
    }
    /// [`Self::admit`] for a type that keeps a bank beside the rows: `beside` is called with the
    /// number of rows to make room for, before the rows' own reservation.
    #[cfg_attr(test, track_caller)]
    pub(super) fn admit_beside(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
        beside: impl FnOnce(&mut ParseControl, usize) -> Result<(), SessionError>,
    ) -> Result<(), SessionError> {
        let admitted = self
            .admitted
            .checked_add(additional)
            .ok_or_else(|| latch_registry_row(control))?;
        // Rows admitted and not yet written take room beyond the written length too.
        beside(control, admitted)?;
        control.reserve_vector_geometric(&mut self.rows, &mut self.allocation, admitted)?;
        control.admit_atomic_layout(mem::size_of::<T>())?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            (mem::size_of::<T>() as u64).saturating_mul(additional as u64),
        )?;
        control.charge(HtmlParseWorkCategory::RecordWrite, additional as u64)?;
        control.acquire(HtmlParseResource::AuxiliaryEntries, additional as u64)?;
        self.admitted = admitted;
        #[cfg(test)]
        control.note_prepared_slot(std::panic::Location::caller());
        Ok(())
    }
    /// Writes the next row into admitted room and returns its position. A row that was not
    /// admitted is the registry-row defect; the bank is unchanged.
    pub(super) fn push(&mut self, row: T) -> Result<usize, SessionError> {
        if self.admitted == 0 || self.rows.len() >= self.rows.capacity() {
            return Err(SessionError::registry_row());
        }
        self.admitted = self
            .admitted
            .checked_sub(1)
            .ok_or_else(SessionError::registry_row)?;
        let position = self.rows.len();
        self.rows.push(row);
        Ok(position)
    }
    pub(super) fn rows(&self) -> &[T] {
        &self.rows
    }
    pub(super) fn row(&self, position: usize) -> Option<&T> {
        self.rows.get(position)
    }
    pub(super) fn row_mut(&mut self, position: usize) -> Option<&mut T> {
        self.rows.get_mut(position)
    }
    pub(super) fn len(&self) -> usize {
        self.rows.len()
    }
    /// Releases the bank and every entry, rows admitted and not written included. `dispose` is
    /// called with each row before it is dropped, for a row that owns a ticket. `beside` releases
    /// what the type keeps beside the rows, after the rows are disposed and before their bank.
    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
        mut dispose: impl FnMut(&mut ParseControl, &mut T) -> Result<(), SessionError>,
        beside: impl FnOnce(&mut ParseControl) -> Result<(), SessionError>,
    ) -> Result<(), SessionError> {
        let Self {
            mut rows,
            allocation,
            admitted,
        } = self;
        let entries = (rows.len() as u64).saturating_add(admitted as u64);
        let mut refused = None;
        for row in &mut rows {
            keep_first(&mut refused, dispose(control, row));
        }
        keep_first(&mut refused, beside(control));
        keep_first(&mut refused, release_vector(control, rows, allocation));
        keep_first(
            &mut refused,
            release_logical(control, owner, HtmlParseResource::AuxiliaryEntries, entries),
        );
        refused.map_or(Ok(()), Err)
    }
    /// The rows admitted and not written.
    #[cfg(test)]
    pub(super) fn admitted(&self) -> usize {
        self.admitted
    }
    /// Drops the rows from `len` on and keeps their entries admitted, as a bank whose rows were
    /// never written. The number of rows dropped.
    #[cfg(test)]
    pub(super) fn drop_rows_from_for_test(&mut self, len: usize) -> usize {
        let dropped = self.rows.len().saturating_sub(len);
        self.rows.truncate(len);
        self.admitted = self
            .admitted
            .checked_add(dropped)
            .expect("admitted rows fit usize");
        dropped
    }
}
