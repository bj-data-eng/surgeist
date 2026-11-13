//! The log of the associations a parse records, and the inputs that its facts were parsed with.
//!
//! A Document capture and an occurrence are rows of one log, named by the position each was
//! pushed at, and a position is the identity the report lends for it. A row is never removed or
//! moved, so a position names the same row for as long as the parse retains it, and one row names
//! another only by a position below its own. The captures have a list of their positions of their
//! own, so that finding the capture of a Document reads the Documents of the parse and not its
//! occurrences. Admission and the write are separate steps, as for every other bank of the parse:
//! an admission reserves the banks and acquires the entries of the rows to come, and a push only
//! writes into what was admitted.
//!
//! The inputs installed from the request are at most one value of the parse, held beside the
//! rows.

use super::associated::{AssociatedStorageUsage, Inputs};
use super::pending_association::{DocumentCapture, Occurrence};
use super::session_error::latch_registry_row;
use super::*;

/// A position in the association log.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AssociationPosition(usize);

impl AssociationPosition {
    pub(super) fn get(self) -> usize {
        self.0
    }
    /// A position that names no row of a log that is shorter, for a test that corrupts a row.
    #[cfg(test)]
    pub(super) fn at_for_test(position: usize) -> Self {
        Self(position)
    }
}

/// A row of the association log.
pub(super) enum AssociationRow {
    Document(DocumentCapture),
    Occurrence(Occurrence),
}

impl AssociationRow {
    /// Releases the storage the row holds that its release returns to the parse: the text of an
    /// occurrence's style inputs and the usage of a capture's captured context.
    pub(super) fn dispose(
        &mut self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> Result<(), SessionError> {
        match self {
            Self::Occurrence(occurrence) => occurrence.release_inputs(control),
            Self::Document(capture) => capture.release_captured(control, owner),
        }
    }
}

/// The inputs a parse was given for its Documents, with the storage the parse holds for them.
pub(super) struct InstalledInputs {
    pub(super) inputs: Arc<Inputs>,
    pub(super) usage: AssociatedStorageUsage,
}

pub(super) struct AssociationLog {
    rows: RowLog<AssociationRow>,
    /// The positions of the Document rows, in the order they were captured.
    documents: RowLog<AssociationPosition>,
    installed: Option<InstalledInputs>,
}

impl AssociationLog {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        let rows = RowLog::new(control)?;
        let documents = match RowLog::new(control) {
            Ok(documents) => documents,
            Err(error) => {
                rows.release(control, &control.identity(), |_, _| Ok(()))?;
                return Err(error);
            }
        };
        Ok(Self {
            rows,
            documents,
            installed: None,
        })
    }
    /// Admits the capture of one Document: its row and its place in the list of captures.
    #[cfg_attr(test, track_caller)]
    pub(super) fn admit_document(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.rows.admit(control, 1)?;
        self.documents.admit(control, 1)
    }
    /// Writes the capture admitted by [`Self::admit_document`] and returns its position. A
    /// capture that was not admitted is the registry-row defect.
    pub(super) fn push_document(
        &mut self,
        capture: DocumentCapture,
    ) -> Result<AssociationPosition, SessionError> {
        let position = AssociationPosition(
            self.rows
                .push(AssociationRow::Document(capture))
                .map_err(|_| SessionError::registry_row())?,
        );
        self.documents.push(position)?;
        Ok(position)
    }
    /// Admits one occurrence.
    #[cfg_attr(test, track_caller)]
    pub(super) fn admit_occurrence(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.rows.admit(control, 1)
    }
    /// Writes the occurrence admitted by [`Self::admit_occurrence`] and returns its position. An
    /// occurrence that was not admitted is the registry-row defect, and is returned with it for
    /// the caller to release: the text of its inputs is a ticket the refusal must not drop.
    #[expect(
        clippy::result_large_err,
        reason = "The refusal hands back the occurrence whose ticketed text its owner must release; the refusal is a defect that ends the parse, so the size of the value is not a cost of the ordinary path"
    )]
    pub(super) fn push_occurrence(
        &mut self,
        control: &mut ParseControl,
        occurrence: Occurrence,
    ) -> Result<AssociationPosition, (SessionError, Occurrence)> {
        match self.rows.room() {
            Some(room) => Ok(AssociationPosition(
                room.write(AssociationRow::Occurrence(occurrence)),
            )),
            None => Err((latch_registry_row(control), occurrence)),
        }
    }
    /// The Document capture at `position`, which is not an occurrence.
    pub(super) fn document(&self, position: AssociationPosition) -> Option<&DocumentCapture> {
        match self.rows.row(position.0) {
            Some(AssociationRow::Document(capture)) => Some(capture),
            _ => None,
        }
    }
    pub(super) fn document_mut(
        &mut self,
        position: AssociationPosition,
    ) -> Option<&mut DocumentCapture> {
        match self.rows.row_mut(position.0) {
            Some(AssociationRow::Document(capture)) => Some(capture),
            _ => None,
        }
    }
    /// The occurrence at `position`, which is not a Document capture.
    pub(super) fn occurrence(&self, position: AssociationPosition) -> Option<&Occurrence> {
        match self.rows.row(position.0) {
            Some(AssociationRow::Occurrence(occurrence)) => Some(occurrence),
            _ => None,
        }
    }
    /// The positions of the Document captures, in the order they were captured.
    pub(super) fn documents(&self) -> impl Iterator<Item = AssociationPosition> {
        self.documents.iter().map(|(_, position)| *position)
    }
    /// The rows with their positions, in the order they were pushed.
    pub(super) fn iter(&self) -> impl Iterator<Item = (AssociationPosition, &AssociationRow)> {
        self.rows
            .iter()
            .map(|(position, row)| (AssociationPosition(position), row))
    }
    /// The row at `index` and its position, for a walk that reads the log one row at a time.
    pub(super) fn at(&self, index: usize) -> Option<(AssociationPosition, &AssociationRow)> {
        self.rows
            .row(index)
            .map(|row| (AssociationPosition(index), row))
    }
    /// The occurrence at `position` for editing, for a test that corrupts a report.
    #[cfg(test)]
    pub(super) fn occurrence_mut_for_test(
        &mut self,
        position: AssociationPosition,
    ) -> Option<&mut Occurrence> {
        match self.rows.row_mut(position.0) {
            Some(AssociationRow::Occurrence(occurrence)) => Some(occurrence),
            _ => None,
        }
    }
    /// How many occurrences the log holds.
    #[cfg(test)]
    pub(super) fn occurrence_count(&self) -> usize {
        self.iter()
            .filter(|(_, row)| matches!(row, AssociationRow::Occurrence(_)))
            .count()
    }
    pub(super) fn len(&self) -> usize {
        self.rows.len()
    }
    pub(super) fn installed(&self) -> Option<&InstalledInputs> {
        self.installed.as_ref()
    }
    /// Holds the inputs of the parse. A parse holds one value, so a second is handed back.
    pub(super) fn install(&mut self, inputs: InstalledInputs) -> Result<(), InstalledInputs> {
        match self.installed {
            Some(_) => Err(inputs),
            None => {
                self.installed = Some(inputs);
                Ok(())
            }
        }
    }
    /// Hands the inputs of the parse to the caller that releases them.
    pub(super) fn take_installed(&mut self) -> Option<InstalledInputs> {
        self.installed.take()
    }
    /// Releases the banks, every entry, and the storage the rows and the inputs hold. A refusal
    /// is remembered and the release goes on, so everything else is still credited; the first
    /// refusal is returned.
    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> Result<(), SessionError> {
        let Self {
            rows,
            documents,
            installed,
        } = self;
        let mut refused = None;
        keep_first(
            &mut refused,
            rows.release(control, owner, |control, row| row.dispose(control, owner)),
        );
        keep_first(
            &mut refused,
            documents.release(control, owner, |_, _| Ok(())),
        );
        if let Some(InstalledInputs { inputs, usage }) = installed {
            drop(inputs);
            keep_first(&mut refused, usage.release(control, owner));
        }
        refused.map_or(Ok(()), Err)
    }
}
