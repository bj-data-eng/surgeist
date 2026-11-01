//! Prepared family authority and retained native bidi-fragment identity.
mod admission;
mod bodies;
mod build;
mod overlay;
mod project;
mod views;
pub(crate) use project::{SelectedBoxPieces, project};
pub use views::{
    FinalizedInlineBoxPiece, InlineBoxFragment, InlineBoxPieceMember, PlacedInlineBoxPiece,
    SelectedInlineBoxPiece,
};

use super::{InlineBoxSource, InlineBoxSourceKind};
use crate::engine::{
    PreparedData,
    analysis::BidiOwner,
    scope_alignment::{
        accounting::{self as account, Ledger},
        census::{AllocationVisitor, AllocationWitness},
    },
};
use crate::{
    PreparedText, Result, ScopeAlignmentAllowance, ScopeAlignmentRetention, ScopeAlignmentUsage,
};
use std::{fmt, ops::Range, sync::Arc};

/// Refusal of a global fragment capability, separate from local edge admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoxFragmentationError {
    IncompleteTopology { domain: usize },
    IncompleteBodyCoverage { section: usize, paragraph: usize },
    ForeignFamily,
    FamilyContextMismatch { control: usize },
    DifferentBody,
    InvalidIncidence,
}
pub(crate) fn invalid(reason: InlineBoxFragmentationError) -> crate::Error {
    crate::protocol::protocol_error(crate::ProtocolError::InlineBoxFragmentation(reason))
}

/// One complete, source-authenticated family for the exact immutable preparation.
/// Its key is semantic; allocation identity is used separately by resource census.
#[derive(Clone)]
pub struct PreparedInlineBoxFamily {
    pub(crate) inner: Arc<FamilyData>,
}
impl fmt::Debug for PreparedInlineBoxFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedInlineBoxFamily")
            .field("prepared", &self.inner.prepared)
            .field("native_boxes", &self.inner.native_boxes)
            .finish_non_exhaustive()
    }
}
impl PartialEq for PreparedInlineBoxFamily {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
            || (Arc::ptr_eq(&self.inner.prepared.inner, &other.inner.prepared.inner)
                && self.inner.native_boxes == other.inner.native_boxes)
    }
}
impl Eq for PreparedInlineBoxFamily {}
impl std::hash::Hash for PreparedInlineBoxFamily {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner.prepared.inner).hash(state);
        self.inner.native_boxes.hash(state);
    }
}
impl PreparedInlineBoxFamily {
    #[must_use]
    pub fn prepared_text(&self) -> PreparedText {
        self.inner.prepared.clone()
    }
    #[must_use]
    pub fn scope_alignment_retention(&self) -> Option<ScopeAlignmentRetention> {
        self.inner.retention
    }
    #[must_use]
    pub fn scope_alignment_usage(&self) -> Option<ScopeAlignmentUsage> {
        self.inner.usage
    }
    pub(crate) fn require_prepared(&self, prepared: &PreparedData) -> Result<()> {
        if !std::ptr::eq(self.inner.prepared.inner.as_ref(), prepared) {
            return Err(invalid(InlineBoxFragmentationError::ForeignFamily));
        }
        Ok(())
    }
    pub(crate) fn visit_allocations(&self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        self.inner.visit_allocations(visit)
    }
}
impl PreparedText {
    /// Admit complete topology/body authority and derive one width-independent
    /// family. Extra native boxes must be the producer's complete declaration.
    pub fn inline_box_family(
        &self,
        native_boxes: Vec<InlineBoxSource>,
        allowance: Option<ScopeAlignmentAllowance>,
    ) -> Result<PreparedInlineBoxFamily> {
        admission::prepare(self, native_boxes, allowance)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Owner {
    Scope(usize),
    Native(usize),
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum RowSource {
    Bidi(BidiOwner),
    Empty(Owner),
}
pub(crate) struct Row {
    pub(crate) source: RowSource,
    pub(crate) body: usize,
    pub(crate) level: u8,
    pub(crate) content: bool,
    pub(crate) section: usize,
    pub(crate) reset: crate::engine::bidi::LineReset,
    pub(crate) ends_body: bool,
    pub(crate) owners: Range<usize>,
}
pub(crate) struct Body {
    pub(crate) visual: Range<usize>,
}
pub(crate) type Fragment = crate::inline_box_components::Component<Owner>;
pub(crate) struct NativeAncestor {
    enclosing: Option<usize>,
    parent: Owner,
}
pub(crate) struct FamilyData {
    pub(crate) prepared: PreparedText,
    pub(crate) native_boxes: Vec<InlineBoxSource>,
    native_ancestors: Vec<NativeAncestor>,
    captured_native_parents: Vec<(usize, usize)>,
    pub(crate) rows: Vec<Row>,
    pub(crate) bodies: Vec<Body>,
    pub(crate) order: Vec<usize>,
    pub(crate) fragments: Vec<Fragment>,
    pub(crate) row_owners: Vec<Owner>,
    pub(crate) row_lookup: Vec<(BidiOwner, usize)>,
    pub(crate) empty_lookup: Vec<((usize, Owner), usize)>,
    /// Actual closed endpoint occurrence to its existing Source-frontier fragment.
    pub(crate) endpoint_fragments: Vec<(usize, usize)>,
    pub(crate) row_fragments: Vec<(usize, usize)>,
    pub(crate) retention: Option<ScopeAlignmentRetention>,
    pub(crate) usage: Option<ScopeAlignmentUsage>,
}
impl FamilyData {
    pub(crate) fn source(&self, owner: Owner) -> InlineBoxSource {
        match owner {
            Owner::Scope(index) => InlineBoxSource::captured(crate::CapturedScope {
                capture: Arc::clone(
                    &self
                        .prepared
                        .inner
                        .analyzed
                        .paragraph
                        .scope_topology
                        .as_ref()
                        .expect("admitted family capture")
                        .capture,
                ),
                index,
            }),
            Owner::Native(index) => self.native_boxes[index].clone(),
        }
    }
    pub(crate) fn interval(&self, owner: Owner) -> (usize, usize) {
        match owner {
            Owner::Scope(index) => {
                let row = &self
                    .prepared
                    .inner
                    .analyzed
                    .paragraph
                    .scope_topology
                    .as_ref()
                    .expect("admitted family capture")
                    .capture
                    .scopes[index];
                (row.enter, row.exit)
            }
            Owner::Native(index) => {
                let (a, b) = self.native_boxes[index].interval();
                (a.index, b.index)
            }
        }
    }
    pub(crate) fn visit_allocations(&self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        visit(AllocationWitness::object(self, 0))?;
        visit(AllocationWitness::buffer(
            &self.native_boxes,
            self.native_boxes.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.native_ancestors,
            self.native_ancestors.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.captured_native_parents,
            self.captured_native_parents.len(),
        )?)?;
        visit(AllocationWitness::buffer(&self.rows, self.rows.len())?)?;
        visit(AllocationWitness::buffer(&self.bodies, self.bodies.len())?)?;
        visit(AllocationWitness::buffer(&self.order, self.order.len())?)?;
        visit(AllocationWitness::buffer(
            &self.fragments,
            self.fragments.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.row_owners,
            self.row_owners.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.row_lookup,
            self.row_lookup.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.empty_lookup,
            self.empty_lookup.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.endpoint_fragments,
            self.endpoint_fragments.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.row_fragments,
            self.row_fragments.len(),
        )?)
    }
    /// Reuse admitted topology; inspect only this operation's real contexts and
    /// endpoint allocations. Source/preparation/role checks remain at admission.
    pub(crate) fn require_contexts(
        &self,
        controls: &[crate::ControlAllocation],
        boundary_context: &[crate::InlineBoundaryContextAllocation],
        ledger: &mut Ledger,
    ) -> Result<()> {
        let mut contexts = Vec::new();
        for (control, context) in super::contexts(controls, boundary_context) {
            account::work(ledger, 1)?;
            let owner = self.owner(&context.owner, ledger)?;
            let parent = self.parent(owner, ledger)?;
            if parent.is_none_or(|parent| self.source(parent) != context.parent) {
                return Err(invalid(
                    InlineBoxFragmentationError::FamilyContextMismatch {
                        control: control.index,
                    },
                ));
            }
            account::push_fallible(ledger, &mut contexts, (owner, context))?;
        }
        if contexts.is_empty() {
            return Ok(());
        }
        crate::processing::contextual::alignment::operations::order_by(
            &mut contexts,
            ledger.as_mut(),
            |a, b| a.0.cmp(&b.0),
        )?;
        let mut endpoints = Vec::new();
        let mut previous: Option<(Owner, &super::InlineBoxEdgeContext)> = None;
        for &(owner, context) in &contexts {
            account::work(ledger, 1)?;
            if let Some((prior, receipt)) = previous
                && prior == owner
            {
                if receipt != context {
                    return Err(super::invalid(
                        super::InlineBoxEdgeError::InconsistentContext,
                    ));
                }
                continue;
            }
            previous = Some((owner, context));
            let (start, end) = context
                .owner
                .endpoints()
                .ok_or_else(|| super::invalid(super::InlineBoxEdgeError::InvalidOwner))?;
            account::push_fallible(ledger, &mut endpoints, (start.index, context))?;
            account::push_fallible(ledger, &mut endpoints, (end.index, context))?;
        }
        crate::processing::contextual::alignment::operations::order_by(
            &mut endpoints,
            ledger.as_mut(),
            |a, b| a.0.cmp(&b.0),
        )?;
        for pair in endpoints.windows(2) {
            account::work(ledger, 1)?;
            if pair[0].0 == pair[1].0 && pair[0].1 != pair[1].1 {
                return Err(super::invalid(
                    super::InlineBoxEdgeError::InconsistentContext,
                ));
            }
        }
        for (control, boundary) in controls
            .iter()
            .filter_map(|row| row.boundary.as_ref().map(|b| (&row.occurrence, b)))
            .chain(
                boundary_context
                    .iter()
                    .map(|row| (&row.occurrence, &row.boundary)),
            )
        {
            account::work(ledger, 1)?;
            let Some(actual) = control.captured_control() else {
                continue;
            };
            let (mut a, mut b) = (0, endpoints.len());
            while a < b {
                account::work(ledger, 1)?;
                let m = a + (b - a) / 2;
                if endpoints[m].0 < actual.index {
                    a = m + 1
                } else {
                    b = m
                }
            }
            if let Some(&(index, context)) = endpoints.get(a)
                && index == actual.index
                && boundary.box_context() != Some(context)
            {
                return Err(super::invalid(
                    super::InlineBoxEdgeError::InconsistentContext,
                ));
            }
        }
        account::drop_vec(ledger, endpoints)?;
        account::drop_vec(ledger, contexts)
    }
}
