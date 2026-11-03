//! Checked source ownership for explicitly requested physical inline edges.
mod components;
pub(crate) mod fragments;
pub(crate) mod incidence;
pub use fragments::{
    FinalizedInlineBoxPiece, InlineBoxFragment, InlineBoxFragmentationError, InlineBoxPieceMember,
    PlacedInlineBoxPiece, PreparedInlineBoxFamily, SelectedInlineBoxPiece,
};
pub(crate) mod execution;
use super::{LineEdgeSide, PreparedText, ProtocolError, protocol_error};
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::{
    CapturedControl, CapturedEventPosition, CapturedScope, ParagraphControlKind, Result, ScopeRole,
};
pub(crate) use components::EdgeExecution;
pub use components::InlineBoxEdgeComponents;

/// An actual captured box or an explicitly declared native endpoint pair.
/// This owns source references, never a Layout pointer or another source tree.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InlineBoxSource {
    pub(crate) kind: InlineBoxSourceKind,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum InlineBoxSourceKind {
    Captured(CapturedScope),
    NativePair {
        start: CapturedControl,
        end: CapturedControl,
    },
}
impl InlineBoxSource {
    #[must_use]
    pub fn captured(scope: CapturedScope) -> Self {
        Self {
            kind: InlineBoxSourceKind::Captured(scope),
        }
    }
    #[must_use]
    pub fn captured_scope(&self) -> Option<&CapturedScope> {
        match &self.kind {
            InlineBoxSourceKind::Captured(scope) => Some(scope),
            InlineBoxSourceKind::NativePair { .. } => None,
        }
    }
    pub(crate) fn domain(&self) -> CapturedScope {
        match &self.kind {
            InlineBoxSourceKind::Captured(scope) => scope.domain(),
            InlineBoxSourceKind::NativePair { start, .. } => start.paragraph().domain(),
        }
    }
    pub(crate) fn interval(&self) -> (CapturedEventPosition, CapturedEventPosition) {
        match &self.kind {
            InlineBoxSourceKind::Captured(scope) => (scope.enter_position(), scope.exit_position()),
            InlineBoxSourceKind::NativePair { start, end } => {
                (start.event_position(), end.event_position())
            }
        }
    }
    pub(crate) fn endpoints(&self) -> Option<(CapturedControl, CapturedControl)> {
        match &self.kind {
            InlineBoxSourceKind::Captured(scope) => {
                Some((scope.entry_control()?, scope.exit_control()?))
            }
            InlineBoxSourceKind::NativePair { start, end } => Some((start.clone(), end.clone())),
        }
    }
    pub(crate) fn is_inline(&self) -> bool {
        !matches!(&self.kind, InlineBoxSourceKind::Captured(scope) if scope.role() == ScopeRole::Domain)
    }
}

/// Source owner, immediate parent, and physical side of its SOURCE Start edge.
/// End uses the opposite side. Construction belongs to the actual preparation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InlineBoxEdgeContext {
    pub(crate) owner: InlineBoxSource,
    pub(crate) parent: InlineBoxSource,
    pub(crate) start_side: LineEdgeSide,
}
impl InlineBoxEdgeContext {
    #[must_use]
    pub fn owner(&self) -> &InlineBoxSource {
        &self.owner
    }
    #[must_use]
    pub fn parent(&self) -> &InlineBoxSource {
        &self.parent
    }
    #[must_use]
    pub const fn start_side(&self) -> LineEdgeSide {
        self.start_side
    }
}

/// A source/parent declaration or selected incidence that cannot be represented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InlineBoxEdgeError {
    MissingCapture,
    ForeignCapture,
    WrongDomain,
    InvalidOwner,
    InvalidPair,
    InvalidParent,
    InconsistentContext,
    UnrepresentedIncidence,
}
pub(crate) fn invalid(reason: InlineBoxEdgeError) -> crate::Error {
    protocol_error(ProtocolError::InlineBoxEdge(reason))
}

impl PreparedText {
    /// Declare a real native box from captured Start/End controls. Exact captured
    /// nominations in their common source scope canonicalize to that scope;
    /// partial nominations or different source scopes refuse.
    pub fn native_inline_box(
        &self,
        start: CapturedControl,
        end: CapturedControl,
    ) -> Result<InlineBoxSource> {
        let topology = self
            .inner
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
        if !std::sync::Arc::ptr_eq(&topology.capture, &start.capture)
            || !std::sync::Arc::ptr_eq(&topology.capture, &end.capture)
        {
            return Err(invalid(InlineBoxEdgeError::ForeignCapture));
        }
        if start.paragraph().domain() != end.paragraph().domain()
            || start.paragraph().domain() != topology.owner.domain()
        {
            return Err(invalid(InlineBoxEdgeError::WrongDomain));
        }
        if *start.kind() != ParagraphControlKind::InlineBoundaryStart
            || *end.kind() != ParagraphControlKind::InlineBoundaryEnd
            || start.event_position() >= end.event_position()
        {
            return Err(invalid(InlineBoxEdgeError::InvalidPair));
        }
        let scope = start.scope();
        if scope != end.scope() {
            return Err(invalid(InlineBoxEdgeError::InvalidPair));
        }
        let first = scope.entry_control().as_ref() == Some(&start);
        let last = scope.exit_control().as_ref() == Some(&end);
        if first || last {
            if first && last {
                return Ok(InlineBoxSource::captured(scope));
            }
            return Err(invalid(InlineBoxEdgeError::InvalidPair));
        }
        Ok(InlineBoxSource {
            kind: InlineBoxSourceKind::NativePair { start, end },
        })
    }

    /// Admit the local capture and immediate-container proof without allocating.
    /// Selection separately validates the complete supplied native family,
    /// omitted captured intermediates and the real selected control roles.
    pub fn inline_box_edge_context(
        &self,
        owner: InlineBoxSource,
        parent: InlineBoxSource,
        start_side: LineEdgeSide,
    ) -> Result<InlineBoxEdgeContext> {
        validate_context(&self.inner, &owner, &parent)?;
        Ok(InlineBoxEdgeContext {
            owner,
            parent,
            start_side,
        })
    }
}

pub(crate) fn validate_context(
    prepared: &crate::engine::PreparedData,
    owner: &InlineBoxSource,
    parent: &InlineBoxSource,
) -> Result<()> {
    let topology = prepared
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
    let domain = owner.domain();
    if !std::sync::Arc::ptr_eq(&topology.capture, &domain.capture)
        || !std::sync::Arc::ptr_eq(&topology.capture, &parent.domain().capture)
    {
        return Err(invalid(InlineBoxEdgeError::ForeignCapture));
    }
    if domain.index != topology.capture.domain_scopes[topology.domain] || parent.domain() != domain
    {
        return Err(invalid(InlineBoxEdgeError::WrongDomain));
    }
    if !owner.is_inline() || owner.endpoints().is_none() {
        return Err(invalid(InlineBoxEdgeError::InvalidOwner));
    }
    let (start, end) = owner.interval();
    let (before, after) = parent.interval();
    if owner == parent || !(before < start && end < after) {
        return Err(invalid(InlineBoxEdgeError::InvalidParent));
    }
    let container = match &owner.kind {
        InlineBoxSourceKind::Captured(scope) => scope
            .parent()
            .ok_or_else(|| invalid(InlineBoxEdgeError::InvalidOwner))?,
        InlineBoxSourceKind::NativePair { start, end } => {
            let scope = start.scope();
            if scope != end.scope() {
                return Err(invalid(InlineBoxEdgeError::InvalidPair));
            }
            scope
        }
    };
    let immediate = match &parent.kind {
        InlineBoxSourceKind::Captured(scope) => *scope == container,
        InlineBoxSourceKind::NativePair { start, end } => {
            let scope = start.scope();
            if scope != end.scope() {
                return Err(invalid(InlineBoxEdgeError::InvalidPair));
            }
            scope == container
        }
    };
    if !immediate {
        return Err(invalid(InlineBoxEdgeError::InvalidParent));
    }
    Ok(())
}

pub(crate) fn contexts<'a>(
    controls: &'a [super::ControlAllocation],
    context: &'a [super::InlineBoundaryContextAllocation],
) -> impl Iterator<Item = (&'a super::PreparedControl, &'a InlineBoxEdgeContext)> {
    controls
        .iter()
        .filter_map(|row| {
            row.boundary
                .as_ref()?
                .box_context()
                .map(|ctx| (&row.occurrence, ctx))
        })
        .chain(
            context
                .iter()
                .filter_map(|row| row.boundary.box_context().map(|ctx| (&row.occurrence, ctx))),
        )
}

/// Validate the actual related family once under the operation's real ledger.
/// The scalar operation neither allocates this family nor claims its authority.
pub(crate) fn validate_request(
    prepared: &crate::engine::PreparedData,
    controls: &[super::ControlAllocation],
    context: &[super::InlineBoundaryContextAllocation],
    ledger: &mut Ledger,
) -> Result<()> {
    if contexts(controls, context).next().is_none() {
        return Ok(());
    }
    let topology = prepared
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
    account::work(ledger, controls.len())?;
    account::work(ledger, context.len())?;
    let mut family: Vec<&InlineBoxEdgeContext> = Vec::new();
    for (control, declaration) in contexts(controls, context) {
        account::work(ledger, 1)?;
        if !std::sync::Arc::ptr_eq(&topology.capture, &declaration.owner.domain().capture)
            || !std::sync::Arc::ptr_eq(&topology.capture, &declaration.parent.domain().capture)
        {
            return Err(invalid(InlineBoxEdgeError::ForeignCapture));
        }
        if declaration.owner.domain().index != topology.capture.domain_scopes[topology.domain]
            || declaration.parent.domain() != declaration.owner.domain()
        {
            return Err(invalid(InlineBoxEdgeError::WrongDomain));
        }
        let captured = control
            .captured_control()
            .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
        let (start, end) = declaration
            .owner
            .endpoints()
            .ok_or_else(|| invalid(InlineBoxEdgeError::InvalidOwner))?;
        if captured != start && captured != end {
            return Err(invalid(InlineBoxEdgeError::InvalidOwner));
        }
        account::work(ledger, family.len())?;
        if let Some(prior) = family.iter().find(|prior| prior.owner == declaration.owner) {
            if **prior != *declaration {
                return Err(invalid(InlineBoxEdgeError::InconsistentContext));
            }
        } else {
            account::push_fallible(ledger, &mut family, declaration)?;
        }
    }
    for row in &family {
        let pair = row
            .owner
            .endpoints()
            .ok_or_else(|| invalid(InlineBoxEdgeError::InvalidOwner))?;
        for (occurrence, boundary) in controls
            .iter()
            .filter_map(|allocation| {
                allocation
                    .boundary
                    .as_ref()
                    .map(|boundary| (&allocation.occurrence, boundary))
            })
            .chain(
                context
                    .iter()
                    .map(|allocation| (&allocation.occurrence, &allocation.boundary)),
            )
        {
            account::work(ledger, 1)?;
            let captured = occurrence
                .captured_control()
                .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
            if (captured == pair.0 || captured == pair.1) && boundary.box_context() != Some(*row) {
                return Err(invalid(InlineBoxEdgeError::InconsistentContext));
            }
        }
        account::work(ledger, 1)?;
        let (a, b) = row.owner.interval();
        let (p, q) = row.parent.interval();
        if !row.owner.is_inline() || row.owner == row.parent || !(p < a && b < q) {
            return Err(invalid(InlineBoxEdgeError::InvalidParent));
        }
        for source in family
            .iter()
            .flat_map(|context| [&context.owner, &context.parent])
        {
            account::work(ledger, 1)?;
            if source == &row.owner || source == &row.parent {
                continue;
            }
            let (c, d) = source.interval();
            if p < c && d < q && c < a && b < d {
                return Err(invalid(InlineBoxEdgeError::InvalidParent));
            }
        }
    }
    // A parent-only declaration still names a real source box. Compare all
    // explicit references without allocating another inventory or assigning an
    // edge operation/side to those parent-only boxes.
    for source in family
        .iter()
        .flat_map(|context| [&context.owner, &context.parent])
    {
        account::work(ledger, 1)?;
        let (a, b) = source.interval();
        for other in family
            .iter()
            .flat_map(|context| [&context.owner, &context.parent])
        {
            account::work(ledger, 1)?;
            if source == other {
                continue;
            }
            let (c, d) = other.interval();
            if (a < c && c < b && b < d) || (c < a && a < d && d < b) {
                return Err(invalid(InlineBoxEdgeError::InvalidPair));
            }
        }
    }
    // Every captured scope is inspected once. A declaration cannot skip a real
    // intermediate Inline merely because that scope has no selected edge.
    for (index, scope) in topology.capture.scopes.iter().enumerate() {
        account::work(ledger, 1)?;
        if scope.domain != topology.domain || scope.definition.is_some() {
            continue;
        }
        for row in &family {
            account::work(ledger, 1)?;
            let source = |value: &InlineBoxSource| matches!(&value.kind, InlineBoxSourceKind::Captured(s) if s.index == index);
            if source(&row.owner) || source(&row.parent) {
                continue;
            }
            let (a, b) = row.owner.interval();
            let (p, q) = row.parent.interval();
            if p.index < scope.enter
                && scope.exit < q.index
                && scope.enter < a.index
                && b.index < scope.exit
            {
                return Err(invalid(InlineBoxEdgeError::InvalidParent));
            }
            if (a.index < scope.enter && scope.enter < b.index && b.index < scope.exit)
                || (scope.enter < a.index && a.index < scope.exit && scope.exit < b.index)
            {
                return Err(invalid(InlineBoxEdgeError::InvalidPair));
            }
        }
    }
    account::drop_vec(ledger, family)?;
    Ok(())
}

/// Borrow actual request facts for identity; never copy a source transcript into
/// the Native numeric proof vector. Each row's source/preparation remains owned.
pub(crate) fn context_identity<'a>(
    controls: &'a [super::ControlAllocation],
    context: &'a [super::InlineBoundaryContextAllocation],
) -> impl Iterator<
    Item = (
        bool,
        &'a super::PreparedControl,
        &'a InlineBoxEdgeContext,
        &'a InlineBoxEdgeComponents,
    ),
> {
    controls
        .iter()
        .filter_map(|row| {
            row.boundary.as_ref()?.box_context().map(|ctx| {
                (
                    false,
                    &row.occurrence,
                    ctx,
                    row.boundary
                        .as_ref()
                        .expect("boxed control retains its allocation")
                        .box_components()
                        .expect("boxed control retains components"),
                )
            })
        })
        .chain(context.iter().filter_map(|row| {
            row.boundary.box_context().map(|ctx| {
                (
                    true,
                    &row.occurrence,
                    ctx,
                    row.boundary
                        .box_components()
                        .expect("boxed context retains components"),
                )
            })
        }))
}

/// Constant-record source checks precede operation funding and provider work.
/// The complete related-family traversal is separate and metered once.
pub(crate) fn validate_sources(
    prepared: &crate::engine::PreparedData,
    controls: &[super::ControlAllocation],
    context: &[super::InlineBoundaryContextAllocation],
) -> Result<()> {
    for (control, declaration) in contexts(controls, context) {
        let topology = prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
        if !std::sync::Arc::ptr_eq(&topology.capture, &declaration.owner.domain().capture)
            || !std::sync::Arc::ptr_eq(&topology.capture, &declaration.parent.domain().capture)
        {
            return Err(invalid(InlineBoxEdgeError::ForeignCapture));
        }
        if declaration.owner.domain().index != topology.capture.domain_scopes[topology.domain]
            || declaration.parent.domain() != declaration.owner.domain()
        {
            return Err(invalid(InlineBoxEdgeError::WrongDomain));
        }
        let actual = control
            .captured_control()
            .ok_or_else(|| invalid(InlineBoxEdgeError::MissingCapture))?;
        let (start, end) = declaration
            .owner
            .endpoints()
            .ok_or_else(|| invalid(InlineBoxEdgeError::InvalidOwner))?;
        if actual != start && actual != end {
            return Err(invalid(InlineBoxEdgeError::InvalidOwner));
        }
    }
    Ok(())
}

/// A box-bearing request owns the WHOLE actual mixed buffers, not only its
/// contextual entries. Vec headers remain in their real stack/selected owner;
/// existing strong capture allocations are already owned by this preparation.
pub(crate) fn visit_request_allocations(
    request: &super::LineSelectionRequest,
    visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
) -> Result<()> {
    if contexts(&request.controls, &request.boundary_context)
        .next()
        .is_none()
    {
        return Ok(());
    }
    use crate::engine::scope_alignment::census::AllocationWitness as W;
    visit(W::buffer(&request.controls, request.controls.len())?)?;
    visit(W::buffer(
        &request.boundary_context,
        request.boundary_context.len(),
    )?)?;
    Ok(())
}

pub(crate) fn comparison_work(ledger: &mut Ledger, rows: usize) -> Result<()> {
    let count = rows.checked_mul(rows).ok_or_else(|| {
        if ledger.is_some() {
            crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
                kind: crate::ScopeAlignmentLimitKind::StructuralWork,
            })
        } else {
            crate::Error::new(
                crate::ErrorCode::LayoutFailed,
                "inline edge comparison size overflow",
            )
        }
    })?;
    account::work(ledger, count)
}
