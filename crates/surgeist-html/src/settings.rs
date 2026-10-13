//! Checked finite parser configuration, independent of parse execution.
//!
//! Caps describe admitted logical storage, known allocation layouts and physical
//! work. They do not promise wall-clock, RSS, allocator-header or provider bounds.

use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(crate) const RESOURCE_COUNT: usize = 19;

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

impl HtmlParseResource {
    pub(crate) fn storage(self) -> bool {
        (self as usize) > 0 && (self as usize) < 14
    }
}

/// Every cap is explicitly supplied and representable. Zero is a valid cap.
/// This configuration proves neither source admission nor parse completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseLimits {
    values: [u64; RESOURCE_COUNT],
}
impl HtmlParseLimits {
    /// Starts construction with all nineteen required caps absent.
    pub fn builder() -> HtmlParseLimitsBuilder {
        HtmlParseLimitsBuilder {
            values: [None; RESOURCE_COUNT],
        }
    }
    /// Reconfigures explicitly supplied caps through the same checked builder.
    pub fn into_builder(self) -> HtmlParseLimitsBuilder {
        HtmlParseLimitsBuilder {
            values: self.values().map(Some),
        }
    }
    pub(crate) fn values(self) -> [u64; RESOURCE_COUNT] {
        self.values
    }
    /// Returns the source units cap.
    pub fn source_units(&self) -> u64 {
        self.values[HtmlParseResource::SourceUnits as usize]
    }
    /// Returns the buffered utf16 units cap.
    pub fn buffered_utf16_units(&self) -> u64 {
        self.values[HtmlParseResource::BufferedUtf16Units as usize]
    }
    /// Returns the token attributes cap.
    pub fn token_attributes(&self) -> u64 {
        self.values[HtmlParseResource::TokenAttributes as usize]
    }
    /// Returns the auxiliary entries cap.
    pub fn auxiliary_entries(&self) -> u64 {
        self.values[HtmlParseResource::AuxiliaryEntries as usize]
    }
    /// Returns the managed nodes cap.
    pub fn managed_nodes(&self) -> u64 {
        self.values[HtmlParseResource::ManagedNodes as usize]
    }
    /// Returns the managed edges cap.
    pub fn managed_edges(&self) -> u64 {
        self.values[HtmlParseResource::ManagedEdges as usize]
    }
    /// Returns the managed attributes cap.
    pub fn managed_attributes(&self) -> u64 {
        self.values[HtmlParseResource::ManagedAttributes as usize]
    }
    /// Returns the managed utf16 units cap.
    pub fn managed_utf16_units(&self) -> u64 {
        self.values[HtmlParseResource::ManagedUtf16Units as usize]
    }
    /// Returns the source map entries cap.
    pub fn source_map_entries(&self) -> u64 {
        self.values[HtmlParseResource::SourceMapEntries as usize]
    }
    /// Returns the source map payload units cap.
    pub fn source_map_payload_units(&self) -> u64 {
        self.values[HtmlParseResource::SourceMapPayloadUnits as usize]
    }
    /// Returns the diagnostics cap.
    pub fn diagnostics(&self) -> u64 {
        self.values[HtmlParseResource::Diagnostics as usize]
    }
    /// Returns the diagnostic payload units cap.
    pub fn diagnostic_payload_units(&self) -> u64 {
        self.values[HtmlParseResource::DiagnosticPayloadUnits as usize]
    }
    /// Returns the context effects cap.
    pub fn context_effects(&self) -> u64 {
        self.values[HtmlParseResource::ContextEffects as usize]
    }
    /// Returns the open elements cap.
    pub fn open_elements(&self) -> u64 {
        self.values[HtmlParseResource::OpenElements as usize]
    }
    /// Returns the tree depth cap.
    pub fn tree_depth(&self) -> u64 {
        self.values[HtmlParseResource::TreeDepth as usize]
    }
    /// Returns the managed capacity bytes cap.
    pub fn managed_capacity_bytes(&self) -> u64 {
        self.values[HtmlParseResource::ManagedCapacityBytes as usize]
    }
    /// Returns the peak transient bytes cap.
    pub fn peak_transient_bytes(&self) -> u64 {
        self.values[HtmlParseResource::PeakTransientBytes as usize]
    }
    /// Returns the atomic layout bytes cap.
    pub fn atomic_layout_bytes(&self) -> u64 {
        self.values[HtmlParseResource::AtomicLayoutBytes as usize]
    }
    /// Returns the work credits cap.
    pub fn work_credits(&self) -> u64 {
        self.values[HtmlParseResource::WorkCredits as usize]
    }
}

/// Required-field construction; there is no default or unlimited sentinel.
#[derive(Clone, Debug)]
pub struct HtmlParseLimitsBuilder {
    values: [Option<u64>; RESOURCE_COUNT],
}
impl HtmlParseLimitsBuilder {
    /// Supplies the source units cap.
    pub fn source_units(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::SourceUnits as usize] = Some(cap);
        self
    }
    /// Supplies the buffered utf16 units cap.
    pub fn buffered_utf16_units(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::BufferedUtf16Units as usize] = Some(cap);
        self
    }
    /// Supplies the token attributes cap.
    pub fn token_attributes(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::TokenAttributes as usize] = Some(cap);
        self
    }
    /// Supplies the auxiliary entries cap.
    pub fn auxiliary_entries(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::AuxiliaryEntries as usize] = Some(cap);
        self
    }
    /// Supplies the managed nodes cap.
    pub fn managed_nodes(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::ManagedNodes as usize] = Some(cap);
        self
    }
    /// Supplies the managed edges cap.
    pub fn managed_edges(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::ManagedEdges as usize] = Some(cap);
        self
    }
    /// Supplies the managed attributes cap.
    pub fn managed_attributes(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::ManagedAttributes as usize] = Some(cap);
        self
    }
    /// Supplies the managed utf16 units cap.
    pub fn managed_utf16_units(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::ManagedUtf16Units as usize] = Some(cap);
        self
    }
    /// Supplies the source map entries cap.
    pub fn source_map_entries(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::SourceMapEntries as usize] = Some(cap);
        self
    }
    /// Supplies the source map payload units cap.
    pub fn source_map_payload_units(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::SourceMapPayloadUnits as usize] = Some(cap);
        self
    }
    /// Supplies the diagnostics cap.
    pub fn diagnostics(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::Diagnostics as usize] = Some(cap);
        self
    }
    /// Supplies the diagnostic payload units cap.
    pub fn diagnostic_payload_units(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::DiagnosticPayloadUnits as usize] = Some(cap);
        self
    }
    /// Supplies the context effects cap.
    pub fn context_effects(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::ContextEffects as usize] = Some(cap);
        self
    }
    /// Supplies the open elements cap.
    pub fn open_elements(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::OpenElements as usize] = Some(cap);
        self
    }
    /// Supplies the tree depth cap.
    pub fn tree_depth(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::TreeDepth as usize] = Some(cap);
        self
    }
    /// Supplies the managed capacity bytes cap.
    pub fn managed_capacity_bytes(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::ManagedCapacityBytes as usize] = Some(cap);
        self
    }
    /// Supplies the peak transient bytes cap.
    pub fn peak_transient_bytes(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::PeakTransientBytes as usize] = Some(cap);
        self
    }
    /// Supplies the atomic layout bytes cap.
    pub fn atomic_layout_bytes(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::AtomicLayoutBytes as usize] = Some(cap);
        self
    }
    /// Supplies the work credits cap.
    pub fn work_credits(mut self, cap: u64) -> Self {
        self.values[HtmlParseResource::WorkCredits as usize] = Some(cap);
        self
    }
    /// Requires every cap, then validates their representation in resource order.
    pub fn finish(self) -> Result<HtmlParseLimits, HtmlParseLimitsError> {
        let mut values = [0; RESOURCE_COUNT];
        for resource in RESOURCES {
            values[resource as usize] = self.values[resource as usize]
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
            let provided = values[resource as usize];
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
