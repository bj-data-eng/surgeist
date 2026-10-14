//! Exact mutable UTF-16 storage for the private parser. Growth and transfer use
//! the execution's accounting owner, never a second text or allocation model.

use super::{
    control::{DocumentStringAllocation, OwnedVec, ParseControl, ParseStop, WorkCategory},
    input::CodePoint,
};
use crate::DocumentString;
use crate::HtmlParseResource;
#[cfg(test)]
use crate::{HtmlParseCancellation, HtmlParseLimits};
use std::mem::size_of;

pub(crate) struct Utf16Buffer {
    units: OwnedVec<u16>,
}

impl Utf16Buffer {
    pub(crate) fn new(control: &mut ParseControl) -> Result<Self, ParseStop> {
        control.charge(WorkCategory::RecordWrite, 1)?;
        let mut units = OwnedVec::new(HtmlParseResource::BufferedUtf16Units)?;
        units.require_owner(control)?;
        Ok(Self { units })
    }

    /// Preflight actual destination ownership without copying or mutating units.
    #[cfg(test)]
    pub(crate) fn require_owner(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        self.units.require_owner(control)
    }

    pub(crate) fn units(&self) -> &[u16] {
        self.units.as_slice()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.units.capacity()
    }

    /// Physical reservation only; each initialized unit remains a push action.
    pub(crate) fn reserve_units(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), ParseStop> {
        self.units.reserve_geometric(control, additional)
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
        let (text, allocation) = control.freeze_document_string(self.units)?;
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
    pub(crate) fn allocation(&self) -> &DocumentStringAllocation {
        &self.allocation
    }
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
    use super::*;

    fn limits() -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(1000)
            .buffered_utf16_units(1000)
            .token_attributes(1000)
            .auxiliary_entries(1000)
            .managed_nodes(1000)
            .managed_edges(1000)
            .managed_attributes(1000)
            .managed_utf16_units(1000)
            .source_map_entries(1000)
            .source_map_payload_units(1000)
            .diagnostics(1000)
            .diagnostic_payload_units(1000)
            .context_effects(1000)
            .open_elements(1000)
            .tree_depth(1000)
            .managed_capacity_bytes(100_000)
            .peak_transient_bytes(100_000)
            .atomic_layout_bytes(100_000)
            .work_credits(100_000)
            .finish()
            .unwrap()
    }

    fn control() -> ParseControl {
        ParseControl::new(limits(), HtmlParseCancellation::new())
    }

    #[test]
    fn physical_reservation_lends_actual_capacity_without_initializing_units() {
        let mut c = control();
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        buffer.reserve_units(&mut c, 4).unwrap();
        let actual = buffer.capacity();
        assert!(actual >= 4);
        assert_eq!(buffer.len(), 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            actual as u64 * 2
        );
        buffer.push_unit(0xD800, &mut c).unwrap();
        assert_eq!(buffer.capacity(), actual);
        assert_eq!(buffer.units(), [0xD800]);
        buffer.release(&mut c).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
    }

    #[test]
    fn physical_reservation_one_short_and_cancellation_preserve_empty_owner_and_first_stop() {
        // Four required u16 cells need eight physical bytes before any unit is
        // initialized. Seven is independently one short of that required layout.
        let caps = limits()
            .into_builder()
            .managed_capacity_bytes(7)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        let stop = buffer.reserve_units(&mut c, 4).unwrap_err();
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            }
        ));
        assert_eq!(buffer.capacity(), 0);
        assert_eq!(buffer.len(), 0);
        buffer.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(stop));
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );

        let token = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), token.clone());
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        token.cancel();
        assert_eq!(buffer.reserve_units(&mut c, 4), Err(ParseStop::Cancelled));
        assert_eq!(buffer.capacity(), 0);
        buffer.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn destination_owner_preflight_preserves_units_and_rejects_foreign_control() {
        let mut first = control();
        let mut destination = Utf16Buffer::new(&mut first).unwrap();
        destination.push_unit(0xD800, &mut first).unwrap();
        destination.require_owner(&mut first).unwrap();
        assert_eq!(destination.units(), [0xD800]);
        let mut other = control();
        assert!(destination.require_owner(&mut other).is_err());
        assert_eq!(destination.units(), [0xD800]);
        assert_eq!(
            first
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        assert_eq!(
            other
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        destination.release(&mut first).unwrap();
        assert_eq!(
            first
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
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
                .resource(HtmlParseResource::BufferedUtf16Units)
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
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn unit_cap_rejects_supplementary_pair_atomically() {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .buffered_utf16_units(2)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer.push_unit(0x61, &mut control).unwrap();
        let stop = buffer
            .push_code_point(CodePoint::new(0x1F642).unwrap(), &mut control)
            .unwrap_err();
        assert_eq!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::BufferedUtf16Units,
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
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
    }

    #[test]
    fn exact_cap_clear_and_reuse_preserve_capacity_and_monotone_work() {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .buffered_utf16_units(3)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer
            .extend_units(&[0xD800, 0, 0xDC00], &mut control)
            .unwrap();
        let capacity = control
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current;
        let before = control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        buffer.clear(&mut control).unwrap();
        assert_eq!(buffer.len(), 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .high_water,
            3
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            capacity
        );
        buffer
            .extend_units(&[0xFEFF, 0x20, 0], &mut control)
            .unwrap();
        assert!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current
                > before
        );
        buffer
            .freeze(&mut control)
            .unwrap()
            .release(&mut control)
            .unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn cancellation_prevents_append_and_failure_cleanup_retains_first_stop() {
        let cancellation = HtmlParseCancellation::new();
        let mut control = ParseControl::new(limits(), cancellation.clone());
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
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
    }

    #[test]
    fn fixed_wrapper_cap_failure_publishes_no_frozen_text() {
        let mut caps = limits();
        caps = caps
            .into_builder()
            .atomic_layout_bytes((size_of::<Vec<u16>>() - 1) as u64)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
        let mut buffer = Utf16Buffer::new(&mut control).unwrap();
        buffer.push_unit(0xD800, &mut control).unwrap();
        assert!(matches!(
            buffer.freeze(&mut control),
            Err(ParseStop::LimitExceeded {
                resource: HtmlParseResource::AtomicLayoutBytes,
                ..
            })
        ));
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
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
            other
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0
        );
        assert_eq!(
            other
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current,
            0
        );
        assert_eq!(
            first
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
    }
}
