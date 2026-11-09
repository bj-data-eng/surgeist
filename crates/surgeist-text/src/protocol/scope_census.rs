//! Bounded census of actual simultaneously retained scope-alignment products.

use std::sync::{Arc, Weak};

use super::scope_alignment::{ScopeAlignmentAllowance, ScopeAlignmentRetention};
use super::{
    FinalizedLine, FlowObservationRequest, LineSelectionRequest, PreparedFlowSegment, PreparedText,
    SelectedLine,
};
use crate::ScopeAlignmentUsage;

/// Executed census costs after traversal scratch has been discarded.
/// Borrowed retained rows remain in usage; only newly materialized census rows
/// are added to a consumer's cumulative materialization history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopeAlignmentCensusAccounting {
    usage: ScopeAlignmentUsage,
    newly_materialized_records: usize,
}
impl ScopeAlignmentCensusAccounting {
    pub(crate) const fn new(usage: ScopeAlignmentUsage, newly_materialized_records: usize) -> Self {
        Self {
            usage,
            newly_materialized_records,
        }
    }
    pub const fn usage(self) -> ScopeAlignmentUsage {
        self.usage
    }
    pub const fn newly_materialized_records(self) -> usize {
        self.newly_materialized_records
    }
}

/// A refused standalone census, preserving its original cause and completed
/// charges. None means no budget was funded. A funded failure never issues a
/// complete retained-union receipt or authorizes a prospective operation.
#[derive(Debug)]
pub struct ScopeAlignmentCensusFailure {
    error: crate::Error,
    accounting: Option<ScopeAlignmentCensusAccounting>,
}
impl ScopeAlignmentCensusFailure {
    pub(crate) fn new(
        error: crate::Error,
        accounting: Option<ScopeAlignmentCensusAccounting>,
    ) -> Self {
        Self { error, accounting }
    }
    pub fn error(&self) -> &crate::Error {
        &self.error
    }
    pub const fn accounting(&self) -> Option<ScopeAlignmentCensusAccounting> {
        self.accounting
    }
    pub fn into_parts(self) -> (crate::Error, Option<ScopeAlignmentCensusAccounting>) {
        (self.error, self.accounting)
    }
}
impl std::fmt::Display for ScopeAlignmentCensusFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}
impl std::error::Error for ScopeAlignmentCensusFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// A borrowed actual Text product, including the preparation and selection it
/// retains. Clones and shared ancestors contribute their live allocations once.
#[derive(Clone, Copy, Debug)]
pub enum ScopeAlignmentProduct<'a> {
    Prepared(&'a PreparedText),
    InlineBoxFamily(&'a super::PreparedInlineBoxFamily),
    Selected(&'a SelectedLine),
    Finalized(&'a FinalizedLine),
    SemanticWindow(&'a super::SemanticWindowEvidence),
}

/// Actual resource roots for a snapshot or a valid prospective native operation.
/// Selection and flow inputs include their typed request-proof frames, so
/// those already retained owners are not misclassified as external products.
#[derive(Clone, Copy, Debug)]
pub enum ScopeAlignmentCensusInput<'a> {
    Product(ScopeAlignmentProduct<'a>),
    InlineBoxFamily {
        prepared: &'a PreparedText,
        native_boxes: &'a Vec<super::InlineBoxSource>,
    },
    Selection {
        prepared: &'a PreparedText,
        request: &'a LineSelectionRequest,
    },
    SemanticWindow {
        prepared: &'a PreparedText,
        request: &'a super::SemanticWindowRequest,
    },
    Flow {
        prepared: &'a PreparedText,
        segment: &'a PreparedFlowSegment,
        request: &'a FlowObservationRequest,
    },
}

/// A repeatable borrowed collection of actual strong products. The borrow
/// keeps storage and membership stable throughout the census. Callers retain
/// those products through any operation admitted against its live custody;
/// Text performs no weak upgrades.
#[derive(Clone, Copy, Debug)]
pub enum ScopeAlignmentProducts<'a> {
    /// Heterogeneous products, including shared preparations and selections.
    Mixed(&'a [ScopeAlignmentProduct<'a>]),
    /// Caller-initialized strong guard slots. Empty slots contribute no Text
    /// product but their inspections are still charged as structural work.
    FinalizedSlots(&'a [Option<FinalizedLine>]),
    /// Actual binding-indexed family custody. None slots add no Text owner;
    /// every slot inspection still consumes structural work.
    InlineBoxFamilySlots(&'a [Option<super::PreparedInlineBoxFamily>]),
    /// Actual computation-owned finalized guards and binding-owned families.
    FinalizedAndInlineBoxFamilies {
        finalized: &'a [Option<FinalizedLine>],
        families: &'a [Option<super::PreparedInlineBoxFamily>],
    },
}

/// Scalar facts from one completed, bounded census. Allocation identities and
/// traversal storage remain private and are dropped before this result escapes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopeAlignmentCensus {
    input_retention: ScopeAlignmentRetention,
    external_retention: ScopeAlignmentRetention,
    accounting: ScopeAlignmentCensusAccounting,
}

impl ScopeAlignmentCensus {
    pub(crate) const fn new(
        input_retention: ScopeAlignmentRetention,
        external_retention: ScopeAlignmentRetention,
        accounting: ScopeAlignmentCensusAccounting,
    ) -> Self {
        Self {
            input_retention,
            external_retention,
            accounting,
        }
    }
    /// Unique retained owners reached from the actual input first.
    pub const fn input_retention(&self) -> ScopeAlignmentRetention {
        self.input_retention
    }

    /// Actual additional owners reached only from the related live products.
    pub const fn external_retention(&self) -> ScopeAlignmentRetention {
        self.external_retention
    }

    /// Actual traversal, unique retained custody and temporary seen-set costs.
    /// The final live-byte count excludes the dropped traversal storage; peak
    /// includes its real allocation and reallocation coexistence.
    pub const fn usage(&self) -> ScopeAlignmentUsage {
        self.accounting.usage()
    }

    pub const fn accounting(&self) -> ScopeAlignmentCensusAccounting {
        self.accounting
    }
}

/// Measures the input and related products as one live allocation union under
/// the input capture's immutable limits. The allowance can reduce only records
/// and payload bytes; it cannot increase any captured ceiling.
///
/// The input must have a contextual capture with explicit alignment limits.
/// Related products without feature retention contribute no feature storage.
/// This operation neither resolves fonts nor changes a product or cache.
/// Selection/Flow describe the explicit preparation and their typed proof
/// frames; this resource receipt is not request validation or authorization.
/// The native operation retains its association guards and semantic admission.
pub fn scope_alignment_census(
    input: ScopeAlignmentCensusInput<'_>,
    related: ScopeAlignmentProducts<'_>,
    allowance: ScopeAlignmentAllowance,
) -> std::result::Result<ScopeAlignmentCensus, ScopeAlignmentCensusFailure> {
    crate::engine::scope_alignment::census::live_products(input, related, allowance)
}

/// A non-owning lifetime observation of one actual canonical finalized product.
/// It cannot authorize a scope association or keep a product alive by itself.
/// Candidate registries must discard expired observations before their census:
/// a dead weak observation can still keep the backing Arc allocation reserved.
#[derive(Clone, Debug)]
pub struct ScopeAlignmentWeakProduct {
    inner: Weak<crate::engine::FinalizedLine>,
}

impl ScopeAlignmentWeakProduct {
    /// Retains the actual canonical product only if a strong owner still lives.
    pub fn upgrade(&self) -> Option<FinalizedLine> {
        self.inner.upgrade().map(|inner| FinalizedLine { inner })
    }

    /// Whether two weak observations refer to the same canonical Arc object.
    pub fn same_product(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.inner, &other.inner)
    }
}

impl FinalizedLine {
    /// Observes the canonical Arc lifetime only for actual feature retention.
    /// Ordinary no-feature output creates no scope-alignment tracking demand.
    pub fn scope_alignment_weak_product(&self) -> Option<ScopeAlignmentWeakProduct> {
        self.inner
            .data
            .scope_retention
            .map(|_| ScopeAlignmentWeakProduct {
                inner: Arc::downgrade(&self.inner),
            })
    }
}
