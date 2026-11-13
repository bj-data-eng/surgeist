//! A log of the rows one family keeps for the parse, addressed by the position each was pushed at.
//!
//! A row that no node owns, such as a policy or a missing dependency, has no key to search for:
//! the position it was pushed at names it for as long as the parse retains it, and a row is never
//! removed or moved. Admission and the write are separate steps, as for every other bank of the
//! parse: [`admit`] reserves the vector and acquires the entries of the rows to come, and
//! [`push`] only writes into what was admitted.
//!
//! [`admit`]: RowLog::admit
//! [`push`]: RowLog::push

use super::row_bank::RowBank;
use super::session_error::latch_registry_row;
use super::*;

pub(super) struct RowLog<T> {
    bank: RowBank<T>,
}

impl<T> RowLog<T> {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        Ok(Self {
            bank: RowBank::new(control)?,
        })
    }
    /// Admits `additional` rows: the vector has room for them and for every row admitted and not
    /// yet written, and their auxiliary entries are acquired. A refusal leaves nothing admitted
    /// that this call added.
    #[cfg_attr(test, track_caller)]
    pub(super) fn admit(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        self.bank.admit(control, additional)
    }
    /// Writes the next row into admitted room and returns its position. A row that was not
    /// admitted is the registry-row defect; the log is unchanged.
    pub(super) fn push(&mut self, row: T) -> Result<usize, SessionError> {
        self.bank.push(row)
    }
    /// The row at `position`, a position [`Self::push`] returned.
    pub(super) fn row(&self, position: usize) -> Option<&T> {
        self.bank.row(position)
    }
    pub(super) fn row_mut(&mut self, position: usize) -> Option<&mut T> {
        self.bank.row_mut(position)
    }
    /// The rows with their positions, in the order they were pushed.
    pub(super) fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.bank.rows().iter().enumerate()
    }
    /// The rows from `position` on: none when `position` is the length of the log. A position
    /// past the length names rows the log never held, which is the registry-row defect.
    pub(super) fn rows_from(
        &self,
        control: &mut ParseControl,
        position: usize,
    ) -> Result<&[T], SessionError> {
        self.bank
            .rows()
            .get(position..)
            .ok_or_else(|| latch_registry_row(control))
    }
    pub(super) fn len(&self) -> usize {
        self.bank.len()
    }
    /// Drops every row and keeps its entry admitted, as a log whose rows were never written.
    /// The number of rows dropped.
    #[cfg(test)]
    pub(super) fn drop_rows_for_test(&mut self) -> usize {
        self.bank.drop_rows_from_for_test(0)
    }
    /// Releases the bank and every entry of the log, rows admitted and not written included.
    /// `dispose` is called with each row before it is dropped, for a row that owns a ticket.
    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
        dispose: impl FnMut(&mut ParseControl, &mut T) -> Result<(), SessionError>,
    ) -> Result<(), SessionError> {
        self.bank.release(control, owner, dispose, |_| Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control() -> ParseControl {
        ParseControl::new(
            super::super::tests::limits(),
            crate::HtmlParseCancellation::new(),
        )
    }

    fn entries(c: &ParseControl) -> u64 {
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current
    }

    #[test]
    fn rows_are_named_by_the_position_they_were_pushed_at() {
        let mut c = control();
        let owner = c.identity();
        let mut log: RowLog<u64> = RowLog::new(&mut c).unwrap();
        log.admit(&mut c, 5).unwrap();
        for value in 0..5_u64 {
            assert_eq!(log.push(value * 10), Ok(value as usize));
        }
        assert_eq!(log.len(), 5);
        for position in 0..5 {
            assert_eq!(log.row(position), Some(&(position as u64 * 10)));
        }
        assert_eq!(log.row(5), None);
        assert_eq!(log.rows_from(&mut c, 3), Ok(&[30, 40][..]));
        assert_eq!(log.rows_from(&mut c, 0).map(<[u64]>::len), Ok(5));
        // The length of the log is the position after its last row: no rows, no defect.
        assert_eq!(log.rows_from(&mut c, 5), Ok(&[][..]));
        assert_eq!(c.stop(), None);
        let walked: Vec<(usize, u64)> = log.iter().map(|(at, row)| (at, *row)).collect();
        assert_eq!(walked, [(0, 0), (1, 10), (2, 20), (3, 30), (4, 40)]);
        *log.row_mut(2).unwrap() = 7;
        assert_eq!(log.row(2), Some(&7));
        assert_eq!(log.row_mut(5), None);
        log.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_position_past_the_log_is_the_registry_row_defect_and_the_length_is_an_empty_read() {
        let mut c = control();
        let owner = c.identity();
        let mut log: RowLog<u8> = RowLog::new(&mut c).unwrap();
        log.admit(&mut c, 2).unwrap();
        log.push(1).unwrap();
        log.push(2).unwrap();
        assert_eq!(log.rows_from(&mut c, 2), Ok(&[][..]));
        assert_eq!(c.stop(), None);
        let invariant = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        assert_eq!(log.rows_from(&mut c, 3), Err(SessionError::Stop(invariant)));
        assert_eq!(c.stop(), Some(invariant));
        assert_eq!(
            log.rows_from(&mut c, usize::MAX),
            Err(SessionError::Stop(invariant))
        );
        log.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_row_that_was_not_admitted_is_refused_and_the_log_is_unchanged() {
        let mut c = control();
        let owner = c.identity();
        let mut log: RowLog<u8> = RowLog::new(&mut c).unwrap();
        assert_eq!(log.push(1), Err(SessionError::registry_row()));
        log.admit(&mut c, 1).unwrap();
        assert_eq!(log.push(1), Ok(0));
        assert_eq!(log.push(2), Err(SessionError::registry_row()));
        assert_eq!(log.len(), 1);
        log.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn two_admissions_before_their_writes_leave_room_for_both_rows() {
        let mut c = control();
        let owner = c.identity();
        let base = entries(&c);
        let mut log: RowLog<u32> = RowLog::new(&mut c).unwrap();
        log.admit(&mut c, 1).unwrap();
        log.admit(&mut c, 1).unwrap();
        assert_eq!(entries(&c), base + 2);
        assert_eq!(log.push(1), Ok(0));
        assert_eq!(log.push(2), Ok(1));
        assert_eq!(log.len(), 2);
        log.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        assert_eq!(entries(&c), base);
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn release_returns_every_entry_including_the_rows_admitted_and_not_written() {
        let mut c = control();
        let owner = c.identity();
        let base = entries(&c);
        let mut log: RowLog<u32> = RowLog::new(&mut c).unwrap();
        log.admit(&mut c, 3).unwrap();
        assert_eq!(entries(&c), base + 3);
        log.push(1).unwrap();
        let mut disposed = 0;
        log.release(&mut c, &owner, |_, _| {
            disposed += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(disposed, 1);
        assert_eq!(entries(&c), base);
        super::super::tests::storage_zero(&c);
    }

    /// A parse that stops between an admission and its write must leave nothing retained, so a
    /// test build counts each admission as a prepared slot, at the caller that asked for it, and
    /// arms the stop at the first checkpoint after it.
    #[test]
    fn an_admission_is_a_prepared_slot_at_its_caller() {
        use crate::parser::control::{
            PreparedSlotStopScope, prepared_slot_sites, prepared_slots, stop_after_prepared_slot,
        };
        let _scope = PreparedSlotStopScope::begin();
        let mut c = control();
        let owner = c.identity();
        let mut log: RowLog<u8> = RowLog::new(&mut c).unwrap();
        stop_after_prepared_slot(Some(1));
        log.admit(&mut c, 1).unwrap();
        let asked_at = line!() + 1;
        log.admit(&mut c, 2).unwrap();
        assert_eq!(prepared_slots(), 2);
        let sites = prepared_slot_sites();
        assert!(sites.iter().all(|site| site.file().ends_with("row_log.rs")));
        assert_eq!(sites.last().map(|site| site.line()), Some(asked_at));
        // The stop armed for the second admission fires at the next checkpoint, with both rows
        // admitted and none written.
        assert!(matches!(
            c.charge(HtmlParseWorkCategory::RecordVisit, 1),
            Err(HtmlParseStop::Cancelled)
        ));
        log.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_refused_admission_leaves_the_log_as_it_was() {
        let base = entries(&control());
        let limits = super::super::tests::limits()
            .into_builder()
            .auxiliary_entries(base + 1)
            .active_formatting_entries(base + 1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let owner = c.identity();
        let mut log: RowLog<u32> = RowLog::new(&mut c).unwrap();
        log.admit(&mut c, 1).unwrap();
        assert_eq!(log.push(1), Ok(0));
        assert!(matches!(
            log.admit(&mut c, 1),
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                ..
            }))
        ));
        assert_eq!(log.len(), 1);
        assert_eq!(log.push(2), Err(SessionError::registry_row()));
        let stop = c.stop();
        log.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
}
