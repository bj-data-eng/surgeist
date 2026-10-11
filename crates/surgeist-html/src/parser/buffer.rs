//! Exact mutable UTF-16 storage for the private parser. Growth and transfer use
//! the execution's accounting owner, never a second text or allocation model.

use super::{
    control::{
        DocumentStringAllocation, OwnedVec, ParseControl, ParseStop, Resource, WorkCategory,
    },
    input::CodePoint,
};
use crate::DocumentString;
use std::mem::size_of;

pub(crate) struct Utf16Buffer {
    units: OwnedVec<u16>,
}

impl Utf16Buffer {
    pub(crate) fn new(control: &mut ParseControl) -> Result<Self, ParseStop> {
        control.charge(WorkCategory::RecordWrite, 1)?;
        let mut units = OwnedVec::new(Resource::BufferedUtf16Units)?;
        units.require_owner(control)?;
        Ok(Self { units })
    }

    pub(crate) fn units(&self) -> &[u16] {
        self.units.as_slice()
    }

    pub(crate) fn len(&self) -> usize {
        self.units.len()
    }

    pub(crate) fn push_unit(
        &mut self,
        unit: u16,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        self.units.try_push(control, unit)
    }

    pub(crate) fn push_code_point(
        &mut self,
        code_point: CodePoint,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        self.units.require_owner(control)?;
        control.charge(WorkCategory::InspectByte, size_of::<CodePoint>() as u64)?;
        let value = code_point.value();
        if value <= 0xFFFF {
            return self.push_unit(value as u16, control);
        }
        let value = value - 0x10000;
        let pair = [
            0xD800 | ((value >> 10) as u16),
            0xDC00 | ((value & 0x3FF) as u16),
        ];
        self.extend_units(&pair, control)
    }

    pub(crate) fn extend_units(
        &mut self,
        units: &[u16],
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        self.units.try_extend_from_slice(control, units)
    }

    pub(crate) fn clear(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        self.units.require_owner(control)?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        self.units.clear(control)
    }

    pub(crate) fn release(self, control: &mut ParseControl) -> Result<(), ParseStop> {
        self.units.release(control)
    }

    /// Transfers the admitted vector without copying its variable payload.
    /// The controller accounts the known fixed Vec payload submitted to Arc;
    /// Arc's opaque header/allocator metadata and infallible setup are explicit
    /// standard-library limitations, not recoverable allocation-failure claims.
    pub(crate) fn freeze(mut self, control: &mut ParseControl) -> Result<FrozenUtf16, ParseStop> {
        if let Err(stop) = self.units.require_owner(control) {
            self.units.release(control)?;
            return Err(stop);
        }
        if let Err(stop) = control.charge(WorkCategory::CopyByte, size_of::<Vec<u16>>() as u64) {
            self.units.release(control)?;
            return Err(stop);
        }
        let (units, allocation) = control.admit_document_string_wrapper(self.units)?;
        let text = DocumentString::from_utf16(units);
        if let Err(stop) = control.checkpoint() {
            drop(text);
            control.release_document_string(allocation)?;
            return Err(stop);
        }
        Ok(FrozenUtf16 { text, allocation })
    }
}

/// The text is the shared construction value. The non-cloneable ticket retains
/// its unique allocation/accounting owner, including after token buffering.
pub(crate) struct FrozenUtf16 {
    text: DocumentString,
    allocation: DocumentStringAllocation,
}

impl FrozenUtf16 {
    pub(crate) fn text(&self) -> &DocumentString {
        &self.text
    }

    /// The consumer owns both text and its accounting. Later construction may
    /// transfer logical domains; dropping this ticket alone is not a refund.
    pub(crate) fn into_parts(self) -> (DocumentString, DocumentStringAllocation) {
        (self.text, self.allocation)
    }

    pub(crate) fn release(self, control: &mut ParseControl) -> Result<(), ParseStop> {
        let (text, allocation) = self.into_parts();
        drop(text);
        control.release_document_string(allocation)
    }
}

#[cfg(test)]
mod tests {
    use super::super::control::{Cancellation, ParseLimits};
    use super::*;

    fn limits() -> ParseLimits {
        ParseLimits {
            source_units: 1000,
            buffered_utf16_units: 1000,
            token_attributes: 1000,
            auxiliary_entries: 1000,
            managed_nodes: 1000,
            managed_edges: 1000,
            managed_attributes: 1000,
            managed_utf16_units: 1000,
            source_map_entries: 1000,
            source_map_payload_units: 1000,
            diagnostics: 1000,
            diagnostic_payload_units: 1000,
            context_effects: 1000,
            open_elements: 1000,
            tree_depth: 1000,
            managed_capacity_bytes: 100_000,
            peak_transient_bytes: 100_000,
            atomic_layout_bytes: 100_000,
            work_credits: 100_000,
        }
    }

    fn control() -> ParseControl {
        ParseControl::new(limits(), Cancellation::new()).unwrap()
    }

    #[test]
    fn literal_units_and_codepoints_freeze_without_replacement_or_payload_copy() {
        let mut control = control();
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer
            .extend_units(&[0, 0x20, 0xD800], &mut control)
            .unwrap();
        buffer
            .push_code_point(CodePoint::new(0x1F642).unwrap(), &mut control)
            .unwrap();
        buffer
            .push_code_point(CodePoint::new(0xDC00).unwrap(), &mut control)
            .unwrap();
        buffer.push_unit(0xFEFF, &mut control).unwrap();
        assert_eq!(buffer.len(), 7);
        let pointer = buffer.units().as_ptr();
        let before = control.usage().work(WorkCategory::CopyByte);
        let frozen = buffer.freeze(&mut control).unwrap();
        assert_eq!(
            frozen.text().code_units(),
            [0, 0x20, 0xD800, 0xD83D, 0xDE42, 0xDC00, 0xFEFF]
        );
        assert_eq!(frozen.text().code_units().as_ptr(), pointer);
        assert_eq!(
            control.usage().work(WorkCategory::CopyByte) - before,
            size_of::<Vec<u16>>() as u64
        );
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            7
        );
        let (text, allocation) = frozen.into_parts();
        assert_eq!(allocation.units(), 7);
        assert!(allocation.vector_capacity() >= 7);
        assert_eq!(allocation.wrapper_bytes(), size_of::<Vec<u16>>() as u64);
        drop(text);
        control.release_document_string(allocation).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn unit_cap_rejects_supplementary_pair_atomically() {
        let mut caps = limits();
        caps.buffered_utf16_units = 2;
        let mut control = ParseControl::new(caps, Cancellation::new()).unwrap();
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer.push_unit(0x61, &mut control).unwrap();
        let stop = buffer
            .push_code_point(CodePoint::new(0x1F642).unwrap(), &mut control)
            .unwrap_err();
        assert_eq!(
            stop,
            ParseStop::LimitExceeded {
                resource: Resource::BufferedUtf16Units,
                limit: 2,
                current: 1,
                requested_delta: 2
            }
        );
        assert_eq!(buffer.units(), [0x61]);
        buffer.release(&mut control).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            0
        );
    }

    #[test]
    fn exact_cap_clear_and_reuse_preserve_capacity_and_monotone_work() {
        let mut caps = limits();
        caps.buffered_utf16_units = 3;
        let mut control = ParseControl::new(caps, Cancellation::new()).unwrap();
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer
            .extend_units(&[0xD800, 0, 0xDC00], &mut control)
            .unwrap();
        let capacity = control
            .usage()
            .resource(Resource::ManagedCapacityBytes)
            .current;
        let before = control.usage().resource(Resource::WorkCredits).current;
        buffer.clear(&mut control).unwrap();
        assert_eq!(buffer.len(), 0);
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .high_water,
            3
        );
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .current,
            capacity
        );
        buffer
            .extend_units(&[0xFEFF, 0x20, 0], &mut control)
            .unwrap();
        assert!(control.usage().resource(Resource::WorkCredits).current > before);
        buffer
            .freeze(&mut control)
            .unwrap()
            .release(&mut control)
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn cancellation_prevents_append_and_failure_cleanup_retains_first_stop() {
        let cancellation = Cancellation::new();
        let mut control = ParseControl::new(limits(), cancellation.clone()).unwrap();
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer.push_unit(0xD800, &mut control).unwrap();
        cancellation.cancel();
        assert_eq!(
            buffer.extend_units(&[0xDC00], &mut control),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(buffer.units(), [0xD800]);
        buffer.release(&mut control).unwrap();
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn fixed_wrapper_cap_failure_publishes_no_frozen_text() {
        let mut caps = limits();
        caps.atomic_layout_bytes = (size_of::<Vec<u16>>() - 1) as u64;
        let mut control = ParseControl::new(caps, Cancellation::new()).unwrap();
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer.push_unit(0xD800, &mut control).unwrap();
        assert!(matches!(
            buffer.freeze(&mut control),
            Err(ParseStop::LimitExceeded {
                resource: Resource::AtomicLayoutBytes,
                ..
            })
        ));
        assert_eq!(
            control
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(Resource::BufferedUtf16Units)
                .current,
            0
        );
    }

    #[test]
    fn frozen_release_cannot_modify_another_execution_owner() {
        let mut first = control();
        let mut buffer = Utf16Buffer::new(&mut first).unwrap();
        buffer.push_unit(0x61, &mut first).unwrap();
        let frozen = buffer.freeze(&mut first).unwrap();
        let mut other = control();
        assert!(frozen.release(&mut other).is_err());
        assert_eq!(
            other.usage().resource(Resource::BufferedUtf16Units).current,
            0
        );
        assert_eq!(
            other
                .usage()
                .resource(Resource::ManagedCapacityBytes)
                .current,
            0
        );
        assert_eq!(
            first.usage().resource(Resource::BufferedUtf16Units).current,
            1
        );
    }
}
