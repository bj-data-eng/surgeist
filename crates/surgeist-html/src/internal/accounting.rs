//! The HTML accounting schema: the closed work and resource dimension sets that
//! the HTML crates charge through `surgeist-ledger`, with each dimension's unit,
//! custody policy, conversion rate and cap scope.
//!
//! Three crates charge these dimensions. The parser charges work categories and
//! parse resources, `surgeist-html` charges the work of its bounded operations,
//! and semantics charges query, fact and capture dimensions. One pair
//! of dimension sets, [`HtmlWork`] and [`HtmlResource`], covers all of them, so
//! one ledger operation can hold every HTML charge and a host can read them in
//! its own scope tree.
//!
//! # Dimensions are quantities, scopes are owners
//!
//! A dimension names one quantity with one meaning, such as the parser's managed
//! UTF-16 payload units. Two owners that count the same quantity independently
//! use the same dimension in different scopes, each scope with its own cap. The
//! fact families of the semantics crate share [`HtmlResource::FactRecords`] this
//! way. Quantities with different meanings are different dimensions even when
//! their unit is the same: the parser keeps four UTF-16 dimensions because it
//! caps four different stores.
//!
//! # Conversion rates
//!
//! A tariff maps a dimension of a scope to a dimension of its parent with an
//! integral scale. The schema defines two tariff tables:
//!
//! - [`charging_tariffs`] connects a scope that charges to the scope that carries
//!   its caps. Each of the eight parser work categories converts to
//!   [`HtmlWork::Credits`] at rate one, which defines the parser's
//!   `WorkCredits` total and the single pool-work count of the DOM adapter once.
//!   Every other dimension keeps its identity.
//! - [`aggregate_tariffs`] connects a cap scope to a host or aggregate scope.
//!   Every dimension keeps its identity except the dimensions with
//!   [`HtmlCustody::Gauge`] custody, which convert at scale zero. A gauge records
//!   the largest value one owner reached; the sum of two owners' maxima is not a
//!   quantity anyone reached, so a gauge stays in the scope that observed it.
//!
//! Semantic work ([`HtmlWork::SemanticStep`]) has no conversion into credits.
//! Credits count the parser's physical operations, no consumer adds semantic
//! steps to them, and a host that wants one combined count defines that rate in
//! its own tariffs.
//!
//! # Scope tree
//!
//! ```text
//! host scope                      (host dimensions, or an HTML operation root)
//! └── cap scope                   Limits carry the caps; aggregate_tariffs to the host
//!     └── charging scope          Limits::unlimited(); charging_tariffs to the cap scope
//!         └── attribution scopes  Limits::unlimited(); Tariffs::identity()
//! ```
//!
//! The charging scope records per-category parser work and every gauge. Gauges
//! are recorded only there: attribution scopes use identity tariffs, so a gauge
//! recorded in two sibling attribution scopes would sum in the charging scope.
//! Attribution scopes record work and balance resources only. The cap scope
//! records credits and the capped resources. [`HtmlCapScope`] lists every cap scope and
//! the dimensions whose caps it carries. A scope that carries no cap of its own
//! is created with `Limits::unlimited()`; the ledger still checks its
//! arithmetic.
//!
//! # Hosts
//!
//! A ledger operation has one pair of dimension sets, so a host that attributes
//! HTML work to its own operation runs an operation of the pair
//! `Operation<HtmlWork, HtmlResource>` and creates the cap scope below its own
//! scope with [`aggregate_tariffs`]. The HTML crates take the scope the host
//! supplies. A host that needs its own dimensions as well keeps a separate
//! operation for them; the schema defines no conversion between dimension sets.
//!
//! # Caps that stay local to their scope
//!
//! - [`HtmlResource::ActiveFormattingEntries`] has its own cap beside
//!   [`HtmlResource::AuxiliaryEntries`] and no tariff joins them. The list of
//!   active formatting elements must not exhaust the entries that the tokenizer
//!   buffer, pending whitespace, template modes and the stack index lists share.
//! - [`HtmlResource::OpenElements`], [`HtmlResource::ContextEffects`] and
//!   [`HtmlResource::TokenAttributes`] likewise keep separate caps: each bounds a
//!   different structure, and a combined cap would let one structure starve
//!   another.
//! - Gauge dimensions keep their caps in the scope that observes them; see
//!   [`HtmlCustody::Gauge`].
//! - The semantic integer digit cap bounds the digits of one integer. It is an
//!   argument of the numeric owner, a property of a single value rather than a
//!   cumulative or live quantity, and has no dimension.
//! - The cap of each semantic fact family, and of the construction, autocomplete,
//!   capture import, registry formation and option value scopes, applies to that
//!   scope alone. Autocomplete is a remaining allowance of the query that starts
//!   it, so its scope is a child of the query scope with identity tariffs: its
//!   charges also count in the query.
//! - [`HtmlResource::RetainedBytes`] and [`HtmlResource::RetainedUtf16Units`] are
//!   live custody of a controlled option value capture, capped by its envelope. The
//!   retained payload of a capture or a base facts graph is a measurement of actual
//!   capacities, and its consumer charges that value to its own accounting.
//!
//! # Resources that are not plain acquire and release
//!
//! [`HtmlResource::custody`] states how each resource maps onto ledger changes:
//!
//! - [`HtmlCustody::Balance`]: acquired and released as storage is taken and
//!   returned.
//! - [`HtmlCustody::Permanent`]: acquired and never released, such as
//!   [`HtmlResource::ManagedNodes`], which is never refunded, and
//!   [`HtmlResource::SourceUnits`], which is admitted once.
//! - [`HtmlCustody::Gauge`]: a value recorded as the largest reached, such as
//!   [`HtmlResource::TreeDepth`] and [`HtmlResource::AtomicLayoutBytes`]. Use
//!   [`record_largest`]: one batch releases the previous value and acquires the
//!   new one, so the cap is checked against the new value and the peak keeps the
//!   largest. A refusal names item 1, the acquire, and reports the usage after
//!   the release as its current value. The parser's stop reports the previous
//!   value as current, and as requested change the value itself for
//!   [`HtmlResource::AtomicLayoutBytes`] and the value minus the previous one
//!   for [`HtmlResource::TreeDepth`]; a consumer builds those from the usage it
//!   read before the batch.
//! - [`HtmlCustody::Retention`]: [`HtmlResource::Diagnostics`] bounds what is
//!   retained and never ends a parse. The consumer acquires one unit per
//!   retained diagnostic; a refusal of that single change means the cap is
//!   reached, and the consumer counts the diagnostic as dropped instead of
//!   stopping.
//!
//! # Simultaneous-failure precedence
//!
//! The ledger checks the changes of a batch in input order. The declaration order
//! of [`HtmlResource`] is the deterministic precedence among parse limits, so a
//! consumer orders a coalesced batch by [`surgeist_ledger::Dimension::ALL`].
//! [`retained_provider_storage`] and the conversions built on it are in that
//! order.
//!
//! The parser checks the credit cap once for a whole charge: the cap is compared
//! with the pre-batch credits plus the sum of every item, and a refusal reports
//! the pre-batch credits as current and the sum as requested change. A ledger
//! refusal names one item with the batch prefix as current, so a consumer
//! recomputes those two figures from the usage it read before the batch and the
//! batch's amounts.
//!
//! # Charging
//!
//! A bounded operation of this crate charges its own work through a
//! [`surgeist_ledger::Charge`] that its caller supplies, before the work runs. A
//! cursor charges in its advancing call, and a description, pool action or layout
//! has a charging method. Each charges the dimensions of this schema in the
//! batches and the order the consumers charge them. The parser, the DOM and
//! semantics implement the trait over their own accounting. An operation that can
//! find a fault of its own reports [`HtmlChargedError`], which keeps the caller's
//! stop and the operation's fault apart.
//!
//! The native builders and the controlled autocomplete materialization enforce
//! their typed limits with a private ledger scope built from the caps below, and
//! read their typed usage back from it.
//!
//! `HtmlRangeNumericLayout` and the request byte accessors of the range numeric
//! machine describe register and buffer sizes whose charges the consumers compose
//! from their own record sizes, so they carry no amount rule of their own.

use crate::bounded::attributes::HtmlAutocompleteMaterializationLimits;
use crate::construction::HtmlConstructionLimits;
use std::{error::Error, fmt};
use surgeist_ledger::{Change, Limits, Tariff, Tariffs, dimensions};

/// What one unit of a dimension counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlUnit {
    /// Bytes of storage, layout, copied or inspected data.
    Bytes,
    /// UTF-16 code units.
    Utf16Units,
    /// Units of supplied source text, UTF-8 bytes or UTF-16 code units by the
    /// source's coordinates.
    SourceUnits,
    /// Retained records, nodes, edges, attributes or entries.
    Records,
    /// Record visits, reads of retained records.
    Visits,
    /// Record writes, writes of record fields.
    Writes,
    /// Requests to a provider.
    Calls,
    /// Individual steps of a state machine or validation.
    Steps,
    /// The parser's total of weighted physical operations.
    Credits,
    /// Levels of nesting.
    Levels,
}

dimensions! {
    /// Work the HTML crates charge. Work is cumulative and never refunded.
    ///
    /// The eight parser categories each weigh one credit. Their units are
    /// physical bytes or individual events, not instructions or elapsed time.
    pub enum HtmlWork => HtmlWorkTable {
        /// One supplied source unit read.
        InputUnitRead,
        /// One byte inspected without copying.
        InspectByte,
        /// One byte copied, including bookkeeping values that move.
        CopyByte,
        /// One state machine step.
        StateStep,
        /// One retained record read.
        RecordVisit,
        /// One record field written.
        RecordWrite,
        /// One byte of a storage reservation, including old and replacement
        /// layout bytes that overlap.
        ReserveByte,
        /// One request to a provider.
        ProviderCall,
        /// The weighted total of the eight parser categories and the single
        /// count of value-pool work. It is charged only through
        /// [`charging_tariffs`].
        Credits,
        /// One semantic step: a visit of a query, a validation step of a fact
        /// family, or a step of a capture import.
        SemanticStep,
    }
}

dimensions! {
    /// Resources the HTML crates hold. Declaration order is the deterministic
    /// precedence among simultaneous parse limit failures.
    pub enum HtmlResource => HtmlResourceTable {
        /// Supplied source units, admitted once.
        SourceUnits,
        /// Live buffered UTF-16 units.
        BufferedUtf16Units,
        /// Live token attribute records.
        TokenAttributes,
        /// Live remaining transient slots. Tokenizer event buffer, pending
        /// whitespace and table text, template modes and stack index lists
        /// share them; active formatting entries do not.
        AuxiliaryEntries,
        /// Permanently allocated construction records, never refunded.
        ManagedNodes,
        /// Live ordinary and association edges.
        ManagedEdges,
        /// Live construction attribute slots.
        ManagedAttributes,
        /// Live construction UTF-16 payload units.
        ManagedUtf16Units,
        /// Live source association records.
        SourceMapEntries,
        /// Live source map UTF-16 payload units.
        SourceMapPayloadUnits,
        /// Retained diagnostics. The cap bounds retention and never refuses the
        /// parse: a diagnostic past the cap is counted as dropped.
        Diagnostics,
        /// Live diagnostic UTF-16 payload units.
        DiagnosticPayloadUnits,
        /// Live contextual effect and overlay records.
        ContextEffects,
        /// Live open element stack slots.
        OpenElements,
        /// Live entries of the list of active formatting elements, markers
        /// included.
        ActiveFormattingEntries,
        /// Association-aware construction ancestry, recorded as the largest
        /// depth reached.
        TreeDepth,
        /// Live known owned capacity bytes, excluding external source and
        /// capture.
        ManagedCapacityBytes,
        /// Live bytes with the overlap of an old and a replacement layout as
        /// its peak.
        PeakTransientBytes,
        /// The largest admitted indivisible allocation or owned record layout.
        AtomicLayoutBytes,
        /// Records retained in a semantic output.
        OutputRecords,
        /// Live records of a transient working vector. The peak is the
        /// reported peak of scratch records.
        ScratchRecords,
        /// Records retained in nested semantic output.
        NestedOutputRecords,
        /// Bytes allocated by a semantic operation, including allocator
        /// overshoot, never refunded.
        AllocatedBytes,
        /// Records admitted into one fact bank.
        FactRecords,
        /// UTF-16 code units admitted into one fact bank.
        FactCodeUnits,
        /// Assigned nodes admitted into language and direction facts.
        AssignedNodes,
        /// Retained entries of a construction capture map.
        MappingEntries,
        /// Subjects admitted into a capture.
        CapturedSubjects,
        /// Relationships admitted into a capture.
        CapturedRelationships,
        /// Attributes admitted into a capture.
        CapturedAttributes,
        /// UTF-16 units admitted into a capture.
        CapturedUtf16Units,
        /// The deepest subject admitted into a capture, recorded as the
        /// largest depth reached.
        CapturedDepth,
        /// Registries admitted into a capture.
        CapturedRegistries,
        /// Custom element definitions admitted into a capture.
        CapturedDefinitions,
        /// Cumulative element name, attribute and saved-is storage of a
        /// capture, including pending formation.
        ElementValueBytes,
        /// Live retained payload bytes of a controlled option value capture,
        /// capped by its envelope. The measured payload of a capture or a base
        /// facts graph is a value its consumer charges to its own accounting.
        RetainedBytes,
        /// Live retained UTF-16 capacity of a controlled option value capture,
        /// capped by its envelope.
        RetainedUtf16Units,
        /// Bytes of incoming fact vectors admitted into one fact bank.
        FactBytes,
        /// The checked total a registry formation needs at one point: its fixed
        /// workspace, the live backing and the next request, recorded as the
        /// largest value reached.
        FormationBytes,
        /// Live backing bytes of a registry formation, requested or actual.
        FormationBacking,
    }
}

/// A change in the HTML schema.
pub type HtmlChange = Change<HtmlWork, HtmlResource>;
/// Caps in the HTML schema. One value configures every cap of a scope.
pub type HtmlLimits = Limits<HtmlWork, HtmlResource>;
/// Conversion rates between HTML scopes.
pub type HtmlTariffs = Tariffs<HtmlWork, HtmlResource>;

/// How a resource maps onto ledger changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustody {
    /// Acquired as storage is taken and released as it is returned.
    Balance,
    /// Acquired and never released.
    Permanent,
    /// A value recorded as the largest reached, through [`record_largest`]. It
    /// is not additive between owners, so [`aggregate_tariffs`] converts it at
    /// scale zero. A refusal reports the usage after the release as its current
    /// value; see [`record_largest`].
    Gauge,
    /// Bounds retention without refusing the operation. A refused acquisition
    /// means the cap is reached and the item is counted as dropped.
    Retention,
}

impl HtmlWork {
    /// The unit this dimension counts.
    pub const fn unit(self) -> HtmlUnit {
        match self {
            Self::InputUnitRead => HtmlUnit::SourceUnits,
            Self::InspectByte | Self::CopyByte | Self::ReserveByte => HtmlUnit::Bytes,
            Self::StateStep | Self::SemanticStep => HtmlUnit::Steps,
            Self::RecordVisit => HtmlUnit::Visits,
            Self::RecordWrite => HtmlUnit::Writes,
            Self::ProviderCall => HtmlUnit::Calls,
            Self::Credits => HtmlUnit::Credits,
        }
    }

    /// How many credits one unit of this dimension counts in a cap scope, or
    /// `None` for a dimension that has no conversion into credits. Each of the
    /// eight parser categories and the credits themselves weigh one.
    pub const fn credits_per_unit(self) -> Option<u64> {
        match self {
            Self::InputUnitRead
            | Self::InspectByte
            | Self::CopyByte
            | Self::StateStep
            | Self::RecordVisit
            | Self::RecordWrite
            | Self::ReserveByte
            | Self::ProviderCall
            | Self::Credits => Some(1),
            Self::SemanticStep => None,
        }
    }

    /// The tariff of this dimension from a charging scope to its cap scope.
    pub const fn charging_tariff(self) -> Tariff<HtmlWork> {
        match self.credits_per_unit() {
            Some(rate) => Tariff::new(Self::Credits, rate),
            None => Tariff::new(Self::SemanticStep, 1),
        }
    }
}

impl HtmlResource {
    /// The unit this dimension counts.
    pub const fn unit(self) -> HtmlUnit {
        match self {
            Self::SourceUnits => HtmlUnit::SourceUnits,
            Self::BufferedUtf16Units
            | Self::ManagedUtf16Units
            | Self::SourceMapPayloadUnits
            | Self::DiagnosticPayloadUnits
            | Self::FactCodeUnits
            | Self::CapturedUtf16Units
            | Self::RetainedUtf16Units => HtmlUnit::Utf16Units,
            Self::TokenAttributes
            | Self::AuxiliaryEntries
            | Self::ManagedNodes
            | Self::ManagedEdges
            | Self::ManagedAttributes
            | Self::SourceMapEntries
            | Self::Diagnostics
            | Self::ContextEffects
            | Self::OpenElements
            | Self::ActiveFormattingEntries
            | Self::OutputRecords
            | Self::ScratchRecords
            | Self::NestedOutputRecords
            | Self::FactRecords
            | Self::AssignedNodes
            | Self::MappingEntries
            | Self::CapturedSubjects
            | Self::CapturedRelationships
            | Self::CapturedAttributes
            | Self::CapturedRegistries
            | Self::CapturedDefinitions => HtmlUnit::Records,
            Self::TreeDepth | Self::CapturedDepth => HtmlUnit::Levels,
            Self::ManagedCapacityBytes
            | Self::PeakTransientBytes
            | Self::AtomicLayoutBytes
            | Self::AllocatedBytes
            | Self::ElementValueBytes
            | Self::RetainedBytes
            | Self::FactBytes
            | Self::FormationBytes
            | Self::FormationBacking => HtmlUnit::Bytes,
        }
    }

    /// How this resource maps onto ledger changes.
    pub const fn custody(self) -> HtmlCustody {
        match self {
            Self::SourceUnits
            | Self::ManagedNodes
            | Self::OutputRecords
            | Self::NestedOutputRecords
            | Self::AllocatedBytes
            | Self::FactRecords
            | Self::FactCodeUnits
            | Self::AssignedNodes
            | Self::MappingEntries
            | Self::CapturedSubjects
            | Self::CapturedRelationships
            | Self::CapturedAttributes
            | Self::CapturedUtf16Units
            | Self::CapturedRegistries
            | Self::CapturedDefinitions
            | Self::ElementValueBytes
            | Self::FactBytes => HtmlCustody::Permanent,
            Self::TreeDepth
            | Self::AtomicLayoutBytes
            | Self::CapturedDepth
            | Self::FormationBytes => HtmlCustody::Gauge,
            Self::Diagnostics => HtmlCustody::Retention,
            Self::BufferedUtf16Units
            | Self::TokenAttributes
            | Self::AuxiliaryEntries
            | Self::ManagedEdges
            | Self::ManagedAttributes
            | Self::ManagedUtf16Units
            | Self::SourceMapEntries
            | Self::SourceMapPayloadUnits
            | Self::DiagnosticPayloadUnits
            | Self::ContextEffects
            | Self::OpenElements
            | Self::ActiveFormattingEntries
            | Self::ManagedCapacityBytes
            | Self::PeakTransientBytes
            | Self::ScratchRecords
            | Self::RetainedBytes
            | Self::RetainedUtf16Units
            | Self::FormationBacking => HtmlCustody::Balance,
        }
    }

    /// The tariff of this dimension from a cap scope to a host or aggregate
    /// scope: identity, or scale zero for a gauge.
    pub const fn aggregate_tariff(self) -> Tariff<HtmlResource> {
        match self.custody() {
            HtmlCustody::Gauge => Tariff::new(self, 0),
            HtmlCustody::Balance | HtmlCustody::Permanent | HtmlCustody::Retention => {
                Tariff::new(self, 1)
            }
        }
    }
}

/// Conversion rates from a scope that charges to the scope that carries its
/// caps. Each parser work category converts to [`HtmlWork::Credits`] at rate one.
pub fn charging_tariffs() -> HtmlTariffs {
    Tariffs::from_fn(HtmlWork::charging_tariff, |resource| {
        Tariff::new(resource, 1)
    })
}

/// Conversion rates from a cap scope to a host or aggregate scope. Gauges stay
/// in the scope that observed them.
pub fn aggregate_tariffs() -> HtmlTariffs {
    Tariffs::from_fn(|work| Tariff::new(work, 1), HtmlResource::aggregate_tariff)
}

/// A scope that carries caps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCapScope {
    /// The parse operation: every parser limit.
    Parse,
    /// A construction arena used on its own: `HtmlConstructionLimits`. Under a
    /// parse the scope is created unlimited and the parse scope carries the caps.
    Construction,
    /// A controlled autocomplete materialization:
    /// `HtmlAutocompleteMaterializationLimits`.
    Autocomplete,
    /// A semantic query session: `HtmlSemanticLimits` without the integer digit
    /// cap.
    Query,
    /// A capture under construction: `HtmlCaptureLimits`.
    Capture,
    /// A construction to capture import: `HtmlConstructionCaptureLimits`.
    CaptureImport,
    /// Base facts: `HtmlBaseFactsLimits`.
    BaseFacts,
    /// Element reference facts: `HtmlElementReferenceFactsLimits`.
    ElementReferenceFacts,
    /// Control facts: `HtmlControlFactsLimits`.
    ControlFacts,
    /// Global inheritance facts: `HtmlGlobalInheritanceLimits`.
    GlobalInheritance,
    /// Output default facts: `HtmlOutputDefaultLimits`.
    OutputDefault,
    /// Inert facts: `HtmlInertFactLimits`.
    InertFacts,
    /// Language and direction facts: `HtmlLanguageDirectionLimits`.
    LanguageDirection,
    /// Media facts: `HtmlMediaFactsLimits`.
    MediaFacts,
    /// Link relationship facts: `HtmlRelFactsLimits`.
    LinkRelationshipFacts,
    /// Image dimension facts: `HtmlImageDimensionFactsLimits`.
    ImageDimensionFacts,
    /// Natural dimension facts: `HtmlNaturalDimensionFactsLimits`.
    NaturalDimensionFacts,
    /// A capacity cap on supplied context records: the list participation
    /// supplement's fact cap or the context's classification cap.
    ContextFacts,
    /// A complete-empty Global registry formation: its formation byte
    /// allowance.
    RegistryFormation,
    /// A controlled option value capture: `HtmlOptionValueEnvelope`.
    OptionValue,
}

impl HtmlCapScope {
    /// Every cap scope.
    pub const ALL: [Self; 20] = [
        Self::Parse,
        Self::Construction,
        Self::Autocomplete,
        Self::Query,
        Self::Capture,
        Self::CaptureImport,
        Self::BaseFacts,
        Self::ElementReferenceFacts,
        Self::ControlFacts,
        Self::GlobalInheritance,
        Self::OutputDefault,
        Self::InertFacts,
        Self::LanguageDirection,
        Self::MediaFacts,
        Self::LinkRelationshipFacts,
        Self::ImageDimensionFacts,
        Self::NaturalDimensionFacts,
        Self::ContextFacts,
        Self::RegistryFormation,
        Self::OptionValue,
    ];

    /// The static scope name for ledger reports.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Parse => "html-parse",
            Self::Construction => "html-construction",
            Self::Autocomplete => "html-autocomplete",
            Self::Query => "html-query",
            Self::Capture => "html-capture",
            Self::CaptureImport => "html-capture-import",
            Self::BaseFacts => "html-base-facts",
            Self::ElementReferenceFacts => "html-element-reference-facts",
            Self::ControlFacts => "html-control-facts",
            Self::GlobalInheritance => "html-global-inheritance",
            Self::OutputDefault => "html-output-default",
            Self::InertFacts => "html-inert-facts",
            Self::LanguageDirection => "html-language-direction",
            Self::MediaFacts => "html-media-facts",
            Self::LinkRelationshipFacts => "html-link-relationship-facts",
            Self::ImageDimensionFacts => "html-image-dimension-facts",
            Self::NaturalDimensionFacts => "html-natural-dimension-facts",
            Self::ContextFacts => "html-context-facts",
            Self::RegistryFormation => "html-registry-formation",
            Self::OptionValue => "html-option-value",
        }
    }

    /// The work dimensions whose caps this scope carries.
    pub const fn capped_work(self) -> &'static [HtmlWork] {
        match self {
            Self::Parse => &[HtmlWork::Credits],
            Self::Construction
            | Self::Autocomplete
            | Self::Capture
            | Self::ContextFacts
            | Self::RegistryFormation => &[],
            Self::Query
            | Self::OptionValue
            | Self::MediaFacts
            | Self::LinkRelationshipFacts
            | Self::ImageDimensionFacts
            | Self::NaturalDimensionFacts
            | Self::CaptureImport
            | Self::BaseFacts
            | Self::ElementReferenceFacts
            | Self::ControlFacts
            | Self::GlobalInheritance
            | Self::OutputDefault
            | Self::InertFacts
            | Self::LanguageDirection => &[HtmlWork::SemanticStep],
        }
    }

    /// The resource dimensions whose caps this scope carries.
    pub const fn capped_resources(self) -> &'static [HtmlResource] {
        match self {
            Self::Parse => &[
                HtmlResource::SourceUnits,
                HtmlResource::BufferedUtf16Units,
                HtmlResource::TokenAttributes,
                HtmlResource::AuxiliaryEntries,
                HtmlResource::ManagedNodes,
                HtmlResource::ManagedEdges,
                HtmlResource::ManagedAttributes,
                HtmlResource::ManagedUtf16Units,
                HtmlResource::SourceMapEntries,
                HtmlResource::SourceMapPayloadUnits,
                HtmlResource::Diagnostics,
                HtmlResource::DiagnosticPayloadUnits,
                HtmlResource::ContextEffects,
                HtmlResource::OpenElements,
                HtmlResource::ActiveFormattingEntries,
                HtmlResource::TreeDepth,
                HtmlResource::ManagedCapacityBytes,
                HtmlResource::PeakTransientBytes,
                HtmlResource::AtomicLayoutBytes,
            ],
            Self::Construction => &[
                HtmlResource::ManagedNodes,
                HtmlResource::ManagedEdges,
                HtmlResource::ManagedAttributes,
                HtmlResource::ManagedUtf16Units,
                HtmlResource::TreeDepth,
            ],
            Self::Autocomplete => &[
                HtmlResource::AllocatedBytes,
                HtmlResource::NestedOutputRecords,
                HtmlResource::ScratchRecords,
            ],
            Self::Query => &[
                HtmlResource::OutputRecords,
                HtmlResource::ScratchRecords,
                HtmlResource::AllocatedBytes,
                HtmlResource::NestedOutputRecords,
            ],
            Self::Capture => &[
                HtmlResource::CapturedSubjects,
                HtmlResource::CapturedRelationships,
                HtmlResource::CapturedAttributes,
                HtmlResource::CapturedUtf16Units,
                HtmlResource::CapturedDepth,
                HtmlResource::CapturedRegistries,
                HtmlResource::CapturedDefinitions,
                HtmlResource::ElementValueBytes,
            ],
            Self::CaptureImport => &[HtmlResource::MappingEntries, HtmlResource::ScratchRecords],
            Self::BaseFacts | Self::OutputDefault => {
                &[HtmlResource::FactRecords, HtmlResource::FactCodeUnits]
            }
            Self::ElementReferenceFacts
            | Self::ControlFacts
            | Self::GlobalInheritance
            | Self::InertFacts
            | Self::ContextFacts => &[HtmlResource::FactRecords],
            Self::MediaFacts
            | Self::LinkRelationshipFacts
            | Self::ImageDimensionFacts
            | Self::NaturalDimensionFacts => &[HtmlResource::FactRecords, HtmlResource::FactBytes],
            Self::RegistryFormation => &[HtmlResource::FormationBytes],
            Self::OptionValue => &[
                HtmlResource::RetainedBytes,
                HtmlResource::RetainedUtf16Units,
            ],
            Self::LanguageDirection => &[
                HtmlResource::FactRecords,
                HtmlResource::AssignedNodes,
                HtmlResource::FactCodeUnits,
            ],
        }
    }
}

/// How a bounded operation that charges before it runs can fail: the charge
/// stopped, or the operation found a fault of its own.
///
/// `Stopped` carries the caller's stop from the charge, so the caller keeps its own
/// typed stop and its first-failure rule. `Faulted` carries the operation's typed
/// fault, which the charge did not cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlChargedError<Stop, Fault> {
    /// The charge before the operation was refused or failed.
    Stopped(Stop),
    /// The operation reported its own fault.
    Faulted(Fault),
}

impl<Stop: fmt::Display, Fault: fmt::Display> fmt::Display for HtmlChargedError<Stop, Fault> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped(stop) => write!(formatter, "the charge stopped: {stop}"),
            Self::Faulted(fault) => fault.fmt(formatter),
        }
    }
}

impl<Stop: Error + 'static, Fault: Error + 'static> Error for HtmlChargedError<Stop, Fault> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Stopped(stop) => Some(stop),
            Self::Faulted(fault) => Some(fault),
        }
    }
}

/// A conversion amount that does not fit the ledger's `u64`.
///
/// Callers stop with their own representation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlChargeOverflow;

impl fmt::Display for HtmlChargeOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("accounting amount is not representable")
    }
}

impl Error for HtmlChargeOverflow {}

/// Widens a platform amount. An amount beyond `u64` saturates, and every finite
/// cap refuses `u64::MAX`.
pub(crate) fn wide(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Records `value` as the largest reached by a gauge resource whose previous
/// value is `held`. Apply both changes in one batch: the release returns the
/// previous value, the acquire checks the cap against the new value and raises
/// the peak.
///
/// A refusal names item 1, the acquire, with the usage after the release as its
/// current value. The parser's stop reports the previous value as current and,
/// as requested change, the value itself for
/// [`HtmlResource::AtomicLayoutBytes`] and the value minus the previous one for
/// [`HtmlResource::TreeDepth`]; a consumer derives those from the usage it read
/// before the batch. Record gauges in the charging scope only.
pub fn record_largest(resource: HtmlResource, held: u64, value: u64) -> [HtmlChange; 2] {
    [
        Change::Release(resource, held),
        Change::Acquire(resource, value),
    ]
}

/// The retained storage of a provider graph: bytes count in both capacity
/// resources, UTF-16 units and entries in their own.
///
/// The order is the resource precedence the parser refuses in: entries, UTF-16
/// units, capacity bytes, peak bytes. It is the order of
/// [`surgeist_ledger::Dimension::ALL`], so a batch in this order reaches the same
/// first refusal as the parser when several caps fail at once.
pub fn retained_provider_storage(bytes: u64, units: u64, entries: u64) -> [(HtmlResource, u64); 4] {
    [
        (HtmlResource::AuxiliaryEntries, entries),
        (HtmlResource::ManagedUtf16Units, units),
        (HtmlResource::ManagedCapacityBytes, bytes),
        (HtmlResource::PeakTransientBytes, bytes),
    ]
}

/// The caps of a construction arena.
pub(crate) fn construction_limits(limits: HtmlConstructionLimits) -> HtmlLimits {
    Limits::from_fn(
        |_| u64::MAX,
        |resource| match resource {
            HtmlResource::ManagedNodes => wide(limits.nodes()),
            HtmlResource::ManagedEdges => wide(limits.edges()),
            HtmlResource::ManagedAttributes => wide(limits.attributes()),
            HtmlResource::ManagedUtf16Units => wide(limits.utf16_units()),
            HtmlResource::TreeDepth => wide(limits.depth()),
            _ => u64::MAX,
        },
    )
}

/// The caps of a controlled autocomplete materialization.
pub(crate) fn autocomplete_limits(limits: HtmlAutocompleteMaterializationLimits) -> HtmlLimits {
    Limits::from_fn(
        |_| u64::MAX,
        |resource| match resource {
            HtmlResource::AllocatedBytes => wide(limits.allocated_bytes),
            HtmlResource::NestedOutputRecords => wide(limits.nested_output_records),
            HtmlResource::ScratchRecords => wide(limits.scratch_records),
            _ => u64::MAX,
        },
    )
}

#[cfg(test)]
mod tests;
