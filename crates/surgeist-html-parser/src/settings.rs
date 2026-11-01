//! Checked finite parser configuration, independent of parse execution.
//!
//! Caps describe admitted logical storage, known allocation layouts and physical
//! work. They do not promise wall-clock, RSS, allocator-header or provider bounds.

use crate::keyed::{Table, slots};
use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(crate) const RESOURCE_COUNT: usize = 19;
/// One value per [`HtmlParseResource`]; a lookup cannot be out of range.
pub(crate) type ResourceTable<T> = Table<HtmlParseResource, T, RESOURCE_COUNT>;

/// Parser limit domains, in deterministic simultaneous-failure precedence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum HtmlParseResource {
    /// Supplied UTF-8 bytes or UTF-16 units, admitted once.
    SourceUnits,
    /// Live buffered UTF-16 units.
    BufferedUtf16Units,
    /// Live token attribute records.
    TokenAttributes,
    /// Live remaining transient slots.
    AuxiliaryEntries,
    /// Permanent allocated construction records; never refunded.
    ManagedNodes,
    /// Live ordinary and association edges.
    ManagedEdges,
    /// Live construction attribute slots.
    ManagedAttributes,
    /// Live construction UTF-16 payload units.
    ManagedUtf16Units,
    /// Live source association records.
    SourceMapEntries,
    /// Live source-map UTF-16 payload units.
    SourceMapPayloadUnits,
    /// Retained required diagnostics; exhaustion is terminal.
    Diagnostics,
    /// Live diagnostic UTF-16 payload units.
    DiagnosticPayloadUnits,
    /// Live contextual effect and overlay records.
    ContextEffects,
    /// Live open-element stack slots.
    OpenElements,
    /// Maximum association-aware construction ancestry.
    TreeDepth,
    /// Live known owned capacity bytes, excluding external source/capture.
    ManagedCapacityBytes,
    /// Conservative overlapping old and replacement layout bytes.
    PeakTransientBytes,
    /// Largest admitted indivisible allocation or owned-record layout.
    AtomicLayoutBytes,
    /// Cumulative deterministic physical-operation credits; never refunded.
    WorkCredits,
}

pub(crate) const RESOURCES: [HtmlParseResource; RESOURCE_COUNT] = [
    HtmlParseResource::SourceUnits,
    HtmlParseResource::BufferedUtf16Units,
    HtmlParseResource::TokenAttributes,
    HtmlParseResource::AuxiliaryEntries,
    HtmlParseResource::ManagedNodes,
    HtmlParseResource::ManagedEdges,
    HtmlParseResource::ManagedAttributes,
    HtmlParseResource::ManagedUtf16Units,
    HtmlParseResource::SourceMapEntries,
    HtmlParseResource::SourceMapPayloadUnits,
    HtmlParseResource::Diagnostics,
    HtmlParseResource::DiagnosticPayloadUnits,
    HtmlParseResource::ContextEffects,
    HtmlParseResource::OpenElements,
    HtmlParseResource::TreeDepth,
    HtmlParseResource::ManagedCapacityBytes,
    HtmlParseResource::PeakTransientBytes,
    HtmlParseResource::AtomicLayoutBytes,
    HtmlParseResource::WorkCredits,
];

slots!(HtmlParseResource, RESOURCE_COUNT, {
    SourceUnits => 0,
    BufferedUtf16Units => 1,
    TokenAttributes => 2,
    AuxiliaryEntries => 3,
    ManagedNodes => 4,
    ManagedEdges => 5,
    ManagedAttributes => 6,
    ManagedUtf16Units => 7,
    SourceMapEntries => 8,
    SourceMapPayloadUnits => 9,
    Diagnostics => 10,
    DiagnosticPayloadUnits => 11,
    ContextEffects => 12,
    OpenElements => 13,
    TreeDepth => 14,
    ManagedCapacityBytes => 15,
    PeakTransientBytes => 16,
    AtomicLayoutBytes => 17,
    WorkCredits => 18,
});
// `RESOURCES` lists every resource once, in discriminant order.
const _: () = {
    let mut i = 0;
    while i < RESOURCE_COUNT {
        assert!(RESOURCES[i] as usize == i);
        i += 1;
    }
};

impl HtmlParseResource {
    pub(crate) fn storage(self) -> bool {
        (self as usize) > 0 && (self as usize) < 14
    }
}

/// Every cap is explicitly supplied and representable. Zero is a valid cap.
/// This configuration proves neither source admission nor parse completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseLimits {
    values: ResourceTable<u64>,
}
impl HtmlParseLimits {
    /// Starts construction with all nineteen required caps absent.
    pub fn builder() -> HtmlParseLimitsBuilder {
        HtmlParseLimitsBuilder {
            values: ResourceTable::filled(None),
        }
    }
    /// Reconfigures explicitly supplied caps through the same checked builder.
    pub fn into_builder(self) -> HtmlParseLimitsBuilder {
        HtmlParseLimitsBuilder {
            values: self.values().map(Some),
        }
    }
    pub(crate) fn values(self) -> ResourceTable<u64> {
        self.values
    }
    /// Returns the source units cap.
    pub fn source_units(&self) -> u64 {
        *self.values.get(HtmlParseResource::SourceUnits)
    }
    /// Returns the buffered utf16 units cap.
    pub fn buffered_utf16_units(&self) -> u64 {
        *self.values.get(HtmlParseResource::BufferedUtf16Units)
    }
    /// Returns the token attributes cap.
    pub fn token_attributes(&self) -> u64 {
        *self.values.get(HtmlParseResource::TokenAttributes)
    }
    /// Returns the auxiliary entries cap.
    pub fn auxiliary_entries(&self) -> u64 {
        *self.values.get(HtmlParseResource::AuxiliaryEntries)
    }
    /// Returns the managed nodes cap.
    pub fn managed_nodes(&self) -> u64 {
        *self.values.get(HtmlParseResource::ManagedNodes)
    }
    /// Returns the managed edges cap.
    pub fn managed_edges(&self) -> u64 {
        *self.values.get(HtmlParseResource::ManagedEdges)
    }
    /// Returns the managed attributes cap.
    pub fn managed_attributes(&self) -> u64 {
        *self.values.get(HtmlParseResource::ManagedAttributes)
    }
    /// Returns the managed utf16 units cap.
    pub fn managed_utf16_units(&self) -> u64 {
        *self.values.get(HtmlParseResource::ManagedUtf16Units)
    }
    /// Returns the source map entries cap.
    pub fn source_map_entries(&self) -> u64 {
        *self.values.get(HtmlParseResource::SourceMapEntries)
    }
    /// Returns the source map payload units cap.
    pub fn source_map_payload_units(&self) -> u64 {
        *self.values.get(HtmlParseResource::SourceMapPayloadUnits)
    }
    /// Returns the diagnostics cap.
    pub fn diagnostics(&self) -> u64 {
        *self.values.get(HtmlParseResource::Diagnostics)
    }
    /// Returns the diagnostic payload units cap.
    pub fn diagnostic_payload_units(&self) -> u64 {
        *self.values.get(HtmlParseResource::DiagnosticPayloadUnits)
    }
    /// Returns the context effects cap.
    pub fn context_effects(&self) -> u64 {
        *self.values.get(HtmlParseResource::ContextEffects)
    }
    /// Returns the open elements cap.
    pub fn open_elements(&self) -> u64 {
        *self.values.get(HtmlParseResource::OpenElements)
    }
    /// Returns the tree depth cap.
    pub fn tree_depth(&self) -> u64 {
        *self.values.get(HtmlParseResource::TreeDepth)
    }
    /// Returns the managed capacity bytes cap.
    pub fn managed_capacity_bytes(&self) -> u64 {
        *self.values.get(HtmlParseResource::ManagedCapacityBytes)
    }
    /// Returns the peak transient bytes cap.
    pub fn peak_transient_bytes(&self) -> u64 {
        *self.values.get(HtmlParseResource::PeakTransientBytes)
    }
    /// Returns the atomic layout bytes cap.
    pub fn atomic_layout_bytes(&self) -> u64 {
        *self.values.get(HtmlParseResource::AtomicLayoutBytes)
    }
    /// Returns the work credits cap.
    pub fn work_credits(&self) -> u64 {
        *self.values.get(HtmlParseResource::WorkCredits)
    }
}

/// Required-field construction; there is no default or unlimited sentinel.
#[derive(Clone, Debug)]
pub struct HtmlParseLimitsBuilder {
    values: ResourceTable<Option<u64>>,
}
impl HtmlParseLimitsBuilder {
    /// Supplies the source units cap.
    pub fn source_units(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::SourceUnits) = Some(cap);
        self
    }
    /// Supplies the buffered utf16 units cap.
    pub fn buffered_utf16_units(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::BufferedUtf16Units) = Some(cap);
        self
    }
    /// Supplies the token attributes cap.
    pub fn token_attributes(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::TokenAttributes) = Some(cap);
        self
    }
    /// Supplies the auxiliary entries cap.
    pub fn auxiliary_entries(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::AuxiliaryEntries) = Some(cap);
        self
    }
    /// Supplies the managed nodes cap.
    pub fn managed_nodes(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::ManagedNodes) = Some(cap);
        self
    }
    /// Supplies the managed edges cap.
    pub fn managed_edges(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::ManagedEdges) = Some(cap);
        self
    }
    /// Supplies the managed attributes cap.
    pub fn managed_attributes(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::ManagedAttributes) = Some(cap);
        self
    }
    /// Supplies the managed utf16 units cap.
    pub fn managed_utf16_units(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::ManagedUtf16Units) = Some(cap);
        self
    }
    /// Supplies the source map entries cap.
    pub fn source_map_entries(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::SourceMapEntries) = Some(cap);
        self
    }
    /// Supplies the source map payload units cap.
    pub fn source_map_payload_units(mut self, cap: u64) -> Self {
        *self
            .values
            .get_mut(HtmlParseResource::SourceMapPayloadUnits) = Some(cap);
        self
    }
    /// Supplies the diagnostics cap.
    pub fn diagnostics(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::Diagnostics) = Some(cap);
        self
    }
    /// Supplies the diagnostic payload units cap.
    pub fn diagnostic_payload_units(mut self, cap: u64) -> Self {
        *self
            .values
            .get_mut(HtmlParseResource::DiagnosticPayloadUnits) = Some(cap);
        self
    }
    /// Supplies the context effects cap.
    pub fn context_effects(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::ContextEffects) = Some(cap);
        self
    }
    /// Supplies the open elements cap.
    pub fn open_elements(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::OpenElements) = Some(cap);
        self
    }
    /// Supplies the tree depth cap.
    pub fn tree_depth(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::TreeDepth) = Some(cap);
        self
    }
    /// Supplies the managed capacity bytes cap.
    pub fn managed_capacity_bytes(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::ManagedCapacityBytes) = Some(cap);
        self
    }
    /// Supplies the peak transient bytes cap.
    pub fn peak_transient_bytes(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::PeakTransientBytes) = Some(cap);
        self
    }
    /// Supplies the atomic layout bytes cap.
    pub fn atomic_layout_bytes(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::AtomicLayoutBytes) = Some(cap);
        self
    }
    /// Supplies the work credits cap.
    pub fn work_credits(mut self, cap: u64) -> Self {
        *self.values.get_mut(HtmlParseResource::WorkCredits) = Some(cap);
        self
    }
    /// Requires every cap, then validates their representation in resource order.
    pub fn finish(self) -> Result<HtmlParseLimits, HtmlParseLimitsError> {
        let mut values = ResourceTable::filled(0);
        for resource in RESOURCES {
            *values.get_mut(resource) = (*self.values.get(resource))
                .ok_or(HtmlParseLimitsError::MissingResource(resource))?;
        }
        for resource in RESOURCES {
            let maximum = match resource {
                HtmlParseResource::WorkCredits => u64::MAX,
                HtmlParseResource::BufferedUtf16Units
                | HtmlParseResource::ManagedUtf16Units
                | HtmlParseResource::SourceMapPayloadUnits
                | HtmlParseResource::DiagnosticPayloadUnits => (isize::MAX as u64) / 2,
                HtmlParseResource::ManagedCapacityBytes | HtmlParseResource::PeakTransientBytes => {
                    usize::MAX as u64
                }
                _ => isize::MAX as u64,
            };
            let provided = *values.get(resource);
            if provided > maximum {
                return Err(HtmlParseLimitsError::Unrepresentable {
                    resource,
                    provided,
                    maximum,
                });
            }
        }
        Ok(HtmlParseLimits { values })
    }
}

/// Missing configuration or a cap that cannot be represented by its owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseLimitsError {
    MissingResource(HtmlParseResource),
    Unrepresentable {
        resource: HtmlParseResource,
        provided: u64,
        maximum: u64,
    },
}
impl fmt::Display for HtmlParseLimitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML parse limits rejected: {self:?}")
    }
}
impl Error for HtmlParseLimitsError {}

/// Sticky cancellation capability issued fresh by one checked parse request.
/// Clones share cancellation; there is no reset or independent public constructor.
#[derive(Clone, Debug)]
pub struct HtmlParseCancellation(Arc<AtomicBool>);
impl HtmlParseCancellation {
    /// Request constructors issue this identity freshly; clones retain it.
    pub(crate) fn same_handle(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(crate) fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    /// Requests cancellation of this execution and every clone of this handle.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    /// Observes the sticky request flag; this does not produce a parse outcome.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
