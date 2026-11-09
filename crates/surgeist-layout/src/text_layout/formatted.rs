//! Invocation-local formatted-line authority. This records emitted flow, not
//! text content, geometry guesses or a second line formatter.

use std::sync::Arc;

use super::scope_alignment::{EntryBacking, EntryFunding, EntryFundingRef, ScopeLease};
use surgeist_text::{PreparedAlternative, PreparedBoundary, PreparedSourceSegment};

#[cfg(test)]
#[path = "formatted_tests.rs"]
mod tests;

/// An authentic mixed endpoint. Starts and full candidate requests deliberately
/// do not enter this key: a different start can absorb an old terminal line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FormattedTailEndpoint {
    pub(crate) boundary: PreparedBoundary,
    pub(crate) alternative: Option<PreparedAlternative>,
}

impl FormattedTailEndpoint {
    pub(crate) fn new(
        boundary: PreparedBoundary,
        alternative: Option<PreparedAlternative>,
    ) -> Self {
        Self {
            boundary,
            alternative,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TailPredicate {
    AcceptedEndpoint,
    ExternalAfterRun,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FormattedTailAssignment {
    pub(crate) endpoint: FormattedTailEndpoint,
    pub(crate) predicate: TailPredicate,
    pub(crate) last: bool,
}

impl FormattedTailAssignment {
    pub(crate) fn last(&self) -> bool {
        self.last
    }
}

/// Node-free immutable loan, bound to owner/run/compute purpose by Layout's
/// session. A known scope may provisionally answer false; unavailable scope
/// never manufactures that answer. Resolution occurs only after Text demands it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FormattedTailSnapshot {
    segment: Option<PreparedSourceSegment>,
    assignments: Option<SnapshotAssignments>,
    observes: bool,
}

impl FormattedTailSnapshot {
    pub(crate) fn known(
        segment: PreparedSourceSegment,
        assignments: Option<Arc<[FormattedTailAssignment]>>,
    ) -> Self {
        Self {
            segment: Some(segment),
            assignments: assignments.map(SnapshotAssignments::Known),
            observes: false,
        }
    }

    pub(crate) fn observes(&self) -> bool {
        self.observes
    }

    pub(crate) fn resolve(
        &self,
        segment: &PreparedSourceSegment,
        boundary: &PreparedBoundary,
        alternative: Option<&PreparedAlternative>,
    ) -> Option<FormattedTailAssignment> {
        self.resolve_predicate(
            segment,
            boundary,
            alternative,
            TailPredicate::AcceptedEndpoint,
        )
    }

    pub(crate) fn resolve_external(
        &self,
        segment: &PreparedSourceSegment,
        boundary: &PreparedBoundary,
        alternative: Option<&PreparedAlternative>,
    ) -> Option<FormattedTailAssignment> {
        self.resolve_predicate(
            segment,
            boundary,
            alternative,
            TailPredicate::ExternalAfterRun,
        )
    }

    fn resolve_predicate(
        &self,
        requested_segment: &PreparedSourceSegment,
        boundary: &PreparedBoundary,
        alternative: Option<&PreparedAlternative>,
        predicate: TailPredicate,
    ) -> Option<FormattedTailAssignment> {
        let segment = self.segment.as_ref()?;
        let prepared = segment.start().prepared_text();
        if segment != requested_segment
            || boundary.prepared_text() != prepared
            || alternative.is_some_and(|value| value.prepared_text() != prepared)
        {
            return None;
        }
        let endpoint = FormattedTailEndpoint::new(boundary.clone(), alternative.cloned());
        let last = self
            .assignments
            .as_deref()
            .and_then(|values| {
                values
                    .iter()
                    .find(|value| value.endpoint == endpoint && value.predicate == predicate)
            })
            .is_some_and(FormattedTailAssignment::last);
        Some(FormattedTailAssignment {
            endpoint,
            predicate,
            last,
        })
    }
}

#[derive(Clone, Debug)]
enum SnapshotAssignments {
    Known(Arc<[FormattedTailAssignment]>),
    Observed(SharedPayload<Vec<FormattedTailAssignment>>),
}
impl PartialEq for SnapshotAssignments {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl Deref for SnapshotAssignments {
    type Target = [FormattedTailAssignment];
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Known(rows) => rows,
            Self::Observed(rows) => rows,
        }
    }
}

/// Returned semantic observations include role-dependent decisions even when
/// their candidate was not ultimately emitted. Prospective role uses retain
/// their own local flow proof and external dependency in the adapter's RoleUse.
#[derive(Default)]
pub(crate) struct FormattedTailUses {
    pub(crate) demands: Vec<FormattedTailAssignment>,
    pub(crate) roles: Vec<super::fit::RoleUse>,
    pub(crate) accepted: Vec<FormattedTailEndpoint>,
    storage: Option<FormattedUsesStorage>,
}
#[derive(Default)]
struct FormattedUsesStorage {
    demands: EntryBacking,
    roles: EntryBacking,
    accepted: EntryBacking,
}
impl FormattedTailUses {
    pub(crate) fn push_demand(
        &mut self,
        value: FormattedTailAssignment,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<()> {
        if funding.is_some() {
            let storage = self
                .storage
                .get_or_insert_with(FormattedUsesStorage::default);
            storage
                .demands
                .reserve_push_ref(&mut self.demands, funding)?;
        }
        self.demands.push(value);
        Ok(())
    }
    pub(crate) fn push_role(
        &mut self,
        value: super::fit::RoleUse,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<()> {
        if funding.is_some() {
            let storage = self
                .storage
                .get_or_insert_with(FormattedUsesStorage::default);
            storage.roles.reserve_push_ref(&mut self.roles, funding)?;
        }
        self.roles.push(value);
        Ok(())
    }
    pub(crate) fn push_accepted(
        &mut self,
        value: FormattedTailEndpoint,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<()> {
        if funding.is_some() {
            let storage = self
                .storage
                .get_or_insert_with(FormattedUsesStorage::default);
            storage
                .accepted
                .reserve_push_ref(&mut self.accepted, funding)?;
        }
        self.accepted.push(value);
        Ok(())
    }
}

impl std::fmt::Debug for FormattedTailUses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FormattedTailUses")
            .field("demands", &self.demands)
            .field("roles", &self.roles)
            .field("accepted", &self.accepted)
            .finish()
    }
}
impl PartialEq for FormattedTailUses {
    fn eq(&self, other: &Self) -> bool {
        self.demands == other.demands
            && self.roles == other.roles
            && self.accepted == other.accepted
    }
}

/// Numeric inline selection has no Text endpoint capability. These are actual
/// accepted source/descriptor occurrences, including zero-width/Forced lines.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NumericFormattedLine {
    pub(crate) source_end: usize,
    pub(crate) descriptor_end: usize,
    pub(crate) ending: surgeist_text::LineEnd,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ComputeScope<Node, S: crate::LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) anonymous: bool,
    pub(crate) frame: Option<FlowFrame<S>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FlowFrame<S: crate::LayoutScalar> {
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) available: crate::AxisSizingRequestOf<S>,
    pub(crate) publication: bool,
}

/// Immutable retained payloads share their actual entry backing. Ordinary
/// invocations keep their existing value storage and ordinary cloning behavior.
pub(crate) struct SharedPayload<T>(Payload<T>);
enum Payload<T> {
    Value(T),
    Entry(Arc<EntryValue<T>>),
}
struct EntryValue<T> {
    value: T,
    _copies: EntryCopies,
    _header: ScopeLease,
}
impl<T: Clone> Clone for SharedPayload<T> {
    fn clone(&self) -> Self {
        match &self.0 {
            Payload::Value(value) => Self(Payload::Value(value.clone())),
            Payload::Entry(value) => Self(Payload::Entry(value.clone())),
        }
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for SharedPayload<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<T: PartialEq> PartialEq for SharedPayload<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl<T> std::ops::Deref for SharedPayload<T> {
    type Target = T;
    fn deref(&self) -> &T {
        match &self.0 {
            Payload::Value(value) => value,
            Payload::Entry(value) => &value.value,
        }
    }
}
impl<T> SharedPayload<T> {
    fn is_shared(&self) -> bool {
        matches!(&self.0, Payload::Entry(_))
    }
    pub(crate) fn new(
        value: T,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<Self> {
        Self::with_copies_ref(value, EntryCopies::default(), funding)
    }
    fn with_copies<S: crate::LayoutScalar>(
        value: T,
        copies: EntryCopies,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Self> {
        Self::with_copies_ref(value, copies, funding.map(EntryFunding::as_ref))
    }
    pub(crate) fn with_copies_ref(
        value: T,
        copies: EntryCopies,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<Self> {
        let Some(funding) = funding else {
            return Ok(Self(Payload::Value(value)));
        };
        let header = match funding.header::<(usize, usize, EntryValue<T>)>() {
            Ok(header) => header,
            Err(error) => {
                drop(value);
                drop(copies);
                return Err(error);
            }
        };
        Ok(Self(Payload::Entry(Arc::new(EntryValue {
            value,
            _copies: copies,
            _header: header,
        }))))
    }
}
/// Consuming ordinary rows preserves moves; funded rows keep the shared payload
/// alive while yielding its immutable handles.
pub(crate) struct SharedRows<T>(RowsSource<T>);
enum RowsSource<T> {
    Value(std::vec::IntoIter<T>),
    Entry {
        payload: Arc<EntryValue<Vec<T>>>,
        index: usize,
    },
}
impl<T: Clone> Iterator for SharedRows<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match &mut self.0 {
            RowsSource::Value(rows) => rows.next(),
            RowsSource::Entry { payload, index } => {
                let row = payload.value.get(*index)?.clone();
                *index += 1;
                Some(row)
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = match &self.0 {
            RowsSource::Value(rows) => rows.len(),
            RowsSource::Entry { payload, index } => payload.value.len() - *index,
        };
        (count, Some(count))
    }
}
impl<T> SharedPayload<Vec<T>> {
    /// Move ordinary backing without detaching any funded payload custody.
    pub(crate) fn into_native(self) -> Result<Vec<T>, Self> {
        match self.0 {
            Payload::Value(rows) => Ok(rows),
            Payload::Entry(payload) => Err(Self(Payload::Entry(payload))),
        }
    }
}
impl<T: Clone> SharedPayload<Vec<T>> {
    pub(crate) fn into_rows(self) -> SharedRows<T> {
        SharedRows(match self.0 {
            Payload::Value(rows) => RowsSource::Value(rows.into_iter()),
            Payload::Entry(payload) => RowsSource::Entry { payload, index: 0 },
        })
    }
}

use std::ops::Deref;

#[derive(Default)]
pub(crate) struct EntryCopies {
    leases: Vec<ScopeLease>,
    lease_backing: EntryBacking,
    row_backing: EntryBacking,
    uses_backing: EntryBacking,
    selected_backing: EntryBacking,
    original_uses: Option<FormattedUsesStorage>,
}
impl EntryCopies {
    fn rows<T, S: crate::LayoutScalar>(
        &mut self,
        funding: Option<&EntryFunding<S>>,
        count: usize,
    ) -> surgeist_text::Result<Vec<T>> {
        self.rows_ref(funding.map(EntryFunding::as_ref), count)
    }
    pub(crate) fn rows_ref<T>(
        &mut self,
        funding: Option<EntryFundingRef<'_>>,
        count: usize,
    ) -> surgeist_text::Result<Vec<T>> {
        let Some(funding) = funding else {
            return Ok(Vec::with_capacity(count));
        };
        if count == 0 {
            return Ok(Vec::new());
        }
        self.lease_backing
            .reserve_push_ref(&mut self.leases, Some(funding))?;
        let (rows, lease) = funding.buffer(count)?;
        self.leases.push(lease);
        Ok(rows)
    }
    pub(crate) fn header_ref<T>(
        &mut self,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<()> {
        if let Some(funding) = funding {
            self.lease_backing
                .reserve_push_ref(&mut self.leases, Some(funding))?;
            let lease = funding.header::<T>()?;
            self.leases.push(lease);
        }
        Ok(())
    }
}
fn visit<S: crate::LayoutScalar>(
    funding: Option<&EntryFunding<S>>,
    count: usize,
) -> surgeist_text::Result<()> {
    if let Some(funding) = funding {
        funding.visit(count)?;
    }
    Ok(())
}
fn rows_equal<T: PartialEq>(
    left: &[T],
    right: &[T],
    funding: Option<EntryFundingRef<'_>>,
) -> surgeist_text::Result<bool> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}
fn copied_rows<T: Clone, S: crate::LayoutScalar>(
    source: &[T],
    funding: Option<&EntryFunding<S>>,
    copies: &mut EntryCopies,
) -> surgeist_text::Result<Vec<T>> {
    let mut rows = copies.rows(funding, source.len())?;
    for row in source {
        visit(funding, 1)?;
        rows.push(row.clone());
    }
    Ok(rows)
}
fn state_rows<T: Clone, S: crate::LayoutScalar>(
    source: &[T],
    funding: Option<&EntryFunding<S>>,
) -> surgeist_text::Result<(Vec<T>, EntryBacking)> {
    let Some(funding) = funding else {
        return Ok((source.to_vec(), EntryBacking::default()));
    };
    if source.is_empty() {
        return Ok((Vec::new(), EntryBacking::default()));
    }
    let (lease, mut rows) = funding
        .buffer(source.len())
        .map(|(rows, lease)| (lease, rows))?;
    for row in source {
        funding.visit(1)?;
        rows.push(row.clone());
    }
    EntryBacking::from_buffer(rows, source.len(), lease, funding)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RunBinding<Node, S: crate::LayoutScalar>(SharedPayload<RunIdentity<Node, S>>);
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RunIdentity<Node, S: crate::LayoutScalar> {
    pub(crate) container: Node,
    pub(crate) nodes: Vec<Node>,
    pub(crate) segment: PreparedSourceSegment,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) mode: crate::AxisSizingRequestOf<f64>,
    pub(crate) lineage: Vec<ComputeScope<Node, S>>,
    pub(crate) owner: Node,
}
impl<Node, S: crate::LayoutScalar> Deref for RunBinding<Node, S> {
    type Target = RunIdentity<Node, S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Node: Copy, S: crate::LayoutScalar> RunBinding<Node, S> {
    fn same(
        &self,
        other: &Self,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<bool>
    where
        Node: Eq,
    {
        if self.container != other.container
            || self.segment != other.segment
            || self.input != other.input
            || self.mode != other.mode
            || self.owner != other.owner
        {
            return Ok(false);
        }
        Ok(rows_equal(&self.nodes, &other.nodes, funding)?
            && rows_equal(&self.lineage, &other.lineage, funding)?)
    }
    fn retain(&self, funding: Option<&EntryFunding<S>>) -> surgeist_text::Result<Self> {
        if funding.is_none() || self.0.is_shared() {
            return Ok(self.clone());
        }
        let mut copies = EntryCopies::default();
        let nodes = copied_rows(&self.nodes, funding, &mut copies)?;
        let lineage = copied_rows(&self.lineage, funding, &mut copies)?;
        let identity = RunIdentity {
            container: self.container,
            nodes,
            segment: self.segment.clone(),
            input: self.input,
            mode: self.mode,
            lineage,
            owner: self.owner,
        };
        Ok(Self(SharedPayload::with_copies(identity, copies, funding)?))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FormattedFlowItem<Node, S: crate::LayoutScalar> {
    Text(RunBinding<Node, S>, Vec<FormattedTailEndpoint>),
    Numeric(Vec<Node>, Vec<NumericFormattedLine>),
    Child(Vec<ComputeScope<Node, S>>),
    Retained(RetainedItem<Node, S>),
}
#[derive(Clone)]
pub(crate) struct RetainedItem<Node, S: crate::LayoutScalar>(
    Arc<EntryValue<FormattedFlowItem<Node, S>>>,
);
impl<Node: std::fmt::Debug, S: crate::LayoutScalar> std::fmt::Debug for RetainedItem<Node, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.fmt(f)
    }
}
impl<Node: PartialEq, S: crate::LayoutScalar> PartialEq for RetainedItem<Node, S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}
impl<Node, S: crate::LayoutScalar> FormattedFlowItem<Node, S> {
    fn view(&self) -> &Self {
        match self {
            Self::Retained(value) => &value.0.value,
            value => value,
        }
    }
}
impl<Node: PartialEq, S: crate::LayoutScalar> PartialEq for FormattedFlowItem<Node, S> {
    fn eq(&self, other: &Self) -> bool {
        match (self.view(), other.view()) {
            (Self::Text(a, b), Self::Text(c, d)) => a == c && b == d,
            (Self::Numeric(a, b), Self::Numeric(c, d)) => a == c && b == d,
            (Self::Child(a), Self::Child(b)) => a == b,
            _ => false,
        }
    }
}
impl<Node: Copy, S: crate::LayoutScalar> FormattedFlowItem<Node, S> {
    fn retained(
        self,
        copies: EntryCopies,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Self> {
        let Some(funding) = funding else {
            return Ok(self);
        };
        let header = match funding.header::<(usize, usize, EntryValue<Self>)>() {
            Ok(lease) => lease,
            Err(error) => {
                drop(self);
                drop(copies);
                return Err(error);
            }
        };
        Ok(Self::Retained(RetainedItem(Arc::new(EntryValue {
            value: self,
            _copies: copies,
            _header: header,
        }))))
    }
    pub(crate) fn numeric(
        nodes: &[Node],
        lines: &[NumericFormattedLine],
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Self> {
        let mut copies = EntryCopies::default();
        let nodes = copied_rows(nodes, funding, &mut copies)?;
        let lines = copied_rows(lines, funding, &mut copies)?;
        Self::Numeric(nodes, lines).retained(copies, funding)
    }
}

/// Produced by one actual block traversal. Entry backing follows mutable rows.
pub(crate) struct FormattedFlow<Node, S: crate::LayoutScalar> {
    pub(crate) frame: Option<FlowFrame<S>>,
    pub(crate) items: Vec<FormattedFlowItem<Node, S>>,
    backing: EntryBacking,
    funding: Option<EntryFunding<S>>,
}
impl<Node: Copy, S: crate::LayoutScalar> FormattedFlow<Node, S> {
    pub(crate) fn new(frame: Option<FlowFrame<S>>, funding: Option<&EntryFunding<S>>) -> Self {
        Self {
            frame,
            items: Vec::new(),
            backing: EntryBacking::default(),
            funding: funding.cloned(),
        }
    }
    pub(crate) fn push(&mut self, item: FormattedFlowItem<Node, S>) -> surgeist_text::Result<()> {
        self.backing
            .reserve_push(&mut self.items, self.funding.as_ref())?;
        self.items.push(item);
        Ok(())
    }
    pub(crate) fn try_clone(
        &self,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Self> {
        let mut flow = Self::new(self.frame, funding);
        (flow.items, flow.backing) = state_rows(&self.items, funding)?;
        Ok(flow)
    }
    fn retained(
        self,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<SharedPayload<FlowContents<Node, S>>> {
        // Retain the backing before exposing its moved rows to a shared record.
        let copies = EntryCopies {
            row_backing: self.backing,
            ..EntryCopies::default()
        };
        SharedPayload::with_copies(
            FlowContents {
                frame: self.frame,
                items: self.items,
            },
            copies,
            funding,
        )
    }
}
#[derive(Clone, Debug, PartialEq)]
struct FlowContents<Node, S: crate::LayoutScalar> {
    frame: Option<FlowFrame<S>>,
    items: Vec<FormattedFlowItem<Node, S>>,
}
#[derive(Clone, Debug, PartialEq)]
struct UsesContents {
    demands: Vec<FormattedTailAssignment>,
    roles: Vec<super::fit::RoleUse>,
    accepted: Vec<FormattedTailEndpoint>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoundUses<Node, S: crate::LayoutScalar> {
    pub(crate) binding: RunBinding<Node, S>,
    uses: SharedPayload<UsesContents>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScopedFlow<Node, S: crate::LayoutScalar>(SharedPayload<ScopedContents<Node, S>>);
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScopedContents<Node, S: crate::LayoutScalar> {
    scope: Vec<ComputeScope<Node, S>>,
    flow: SharedPayload<FlowContents<Node, S>>,
}
impl<Node, S: crate::LayoutScalar> Deref for ScopedFlow<Node, S> {
    type Target = ScopedContents<Node, S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SelectedFlow<Node, S: crate::LayoutScalar>(SharedPayload<Selection<Node, S>>);
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Selection<Node, S: crate::LayoutScalar> {
    call: Vec<ComputeScope<Node, S>>,
    actual: Vec<ComputeScope<Node, S>>,
}
impl<Node, S: crate::LayoutScalar> Deref for SelectedFlow<Node, S> {
    type Target = Selection<Node, S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SummaryReceipt<Node, S: crate::LayoutScalar> {
    contents: SharedPayload<SummaryContents<Node, S>>,
    output: Option<(crate::CacheKeyContext, crate::ComputeOutputOf<S>)>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SummaryContents<Node, S: crate::LayoutScalar> {
    wrapping: Option<Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    frames: Arc<[super::state::CanonicalFrame<Node, S>]>,
    node: Node,
    input: crate::ComputeInputOf<S>,
    parent: Vec<ComputeScope<Node, S>>,
    flows: Vec<ScopedFlow<Node, S>>,
    uses: Vec<BoundUses<Node, S>>,
    selected: Vec<SelectedFlow<Node, S>>,
}
impl<Node, S: crate::LayoutScalar> Deref for SummaryReceipt<Node, S> {
    type Target = SummaryContents<Node, S>;
    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoundAssignment<Node, S: crate::LayoutScalar> {
    pub(crate) binding: RunBinding<Node, S>,
    pub(crate) assignment: FormattedTailAssignment,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormattedPassBasis<Node, S: crate::LayoutScalar>(
    SharedPayload<PassContents<Node, S>>,
);
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PassContents<Node, S: crate::LayoutScalar> {
    wrapping: Option<Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    frame_basis: Arc<[super::state::CanonicalFrame<Node, S>]>,
    supplied: Vec<BoundAssignment<Node, S>>,
    uses: Vec<BoundUses<Node, S>>,
    flows: Vec<ScopedFlow<Node, S>>,
    observed: Vec<BoundAssignment<Node, S>>,
    selected: Vec<SelectedFlow<Node, S>>,
}
impl<Node, S: crate::LayoutScalar> Deref for FormattedPassBasis<Node, S> {
    type Target = PassContents<Node, S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct FormattedState<Node, S: crate::LayoutScalar> {
    pub(crate) wrapping: Option<Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    pub(crate) frames: Arc<[super::state::CanonicalFrame<Node, S>]>,
    pub(crate) stack: Vec<ComputeScope<Node, S>>,
    pub(crate) assignments: Vec<BoundAssignment<Node, S>>,
    pub(crate) uses: Vec<BoundUses<Node, S>>,
    pub(crate) flows: Vec<ScopedFlow<Node, S>>,
    pub(crate) selected: Vec<SelectedFlow<Node, S>>,
    pub(crate) receipts: Vec<SummaryReceipt<Node, S>>,
    pub(crate) demanded: Vec<Node>,
    stack_backing: EntryBacking,
    assignment_backing: EntryBacking,
    uses_backing: EntryBacking,
    flow_backing: EntryBacking,
    selected_backing: EntryBacking,
    receipt_backing: EntryBacking,
    demanded_backing: EntryBacking,
    funding: Option<EntryFunding<S>>,
}
pub(crate) struct EntryFormattedBasis<Node, S: crate::LayoutScalar> {
    pub(crate) assignments: Vec<BoundAssignment<Node, S>>,
    pub(crate) parent: Option<Vec<ComputeScope<Node, S>>>,
    _copies: EntryCopies,
}

impl<Node: Copy + Eq, S: crate::LayoutScalar> EntryFormattedBasis<Node, S> {
    pub(crate) fn matches(
        &self,
        assignments: &[BoundAssignment<Node, S>],
        parent: Option<&[ComputeScope<Node, S>]>,
        funding: Option<EntryFundingRef<'_>>,
    ) -> surgeist_text::Result<bool> {
        if self.assignments.len() != assignments.len() {
            return Ok(false);
        }
        for (expected, actual) in self.assignments.iter().zip(assignments) {
            if let Some(funding) = funding {
                funding.visit(1)?;
            }
            if expected.assignment != actual.assignment
                || !expected.binding.same(&actual.binding, funding)?
            {
                return Ok(false);
            }
        }
        match (&self.parent, parent) {
            (Some(expected), Some(actual)) => rows_equal(expected, actual, funding),
            (None, _) => Ok(true),
            _ => Ok(false),
        }
    }
}

impl<Node: Copy + Eq, S: crate::LayoutScalar> FormattedState<Node, S> {
    pub(crate) fn new_entry(
        funding: Option<&EntryFunding<S>>,
        frames: Arc<[super::state::CanonicalFrame<Node, S>]>,
        wrapping: Option<Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    ) -> surgeist_text::Result<Self> {
        visit(funding, 1)?;
        Ok(Self {
            wrapping,
            frames,
            stack: Vec::new(),
            assignments: Vec::new(),
            uses: Vec::new(),
            flows: Vec::new(),
            selected: Vec::new(),
            receipts: Vec::new(),
            demanded: Vec::new(),
            stack_backing: EntryBacking::default(),
            assignment_backing: EntryBacking::default(),
            uses_backing: EntryBacking::default(),
            flow_backing: EntryBacking::default(),
            selected_backing: EntryBacking::default(),
            receipt_backing: EntryBacking::default(),
            demanded_backing: EntryBacking::default(),
            funding: funding.cloned(),
        })
    }
    fn visit(&self, count: usize) -> surgeist_text::Result<()> {
        visit(self.funding.as_ref(), count)
    }
    fn replacing(&self) -> surgeist_text::Result<()> {
        if let Some(funding) = &self.funding {
            funding.materialize(1)?;
        }
        Ok(())
    }
    pub(crate) fn push_scope(&mut self, scope: ComputeScope<Node, S>) -> surgeist_text::Result<()> {
        self.stack_backing
            .reserve_push(&mut self.stack, self.funding.as_ref())?;
        self.stack.push(scope);
        Ok(())
    }
    pub(crate) fn begin_flow(&mut self, frame: FlowFrame<S>) -> surgeist_text::Result<()> {
        if !self.stack.is_empty() {
            self.visit(1)?;
            self.replacing()?;
            self.stack.last_mut().unwrap().frame = Some(frame);
        }
        Ok(())
    }
    pub(crate) fn binding(
        &self,
        container: Node,
        nodes: &[Node],
        segment: &PreparedSourceSegment,
        input: crate::ComputeInputOf<S>,
        mode: crate::AxisSizingRequestOf<f64>,
    ) -> surgeist_text::Result<RunBinding<Node, S>> {
        let mut owner = container;
        for scope in self.stack.iter().rev() {
            self.visit(1)?;
            if !scope.anonymous {
                owner = scope.node;
                break;
            }
        }
        let mut copies = EntryCopies::default();
        let nodes = copied_rows(nodes, self.funding.as_ref(), &mut copies)?;
        let lineage = copied_rows(&self.stack, self.funding.as_ref(), &mut copies)?;
        Ok(RunBinding(SharedPayload::with_copies(
            RunIdentity {
                container,
                nodes,
                segment: segment.clone(),
                input,
                mode,
                lineage,
                owner,
            },
            copies,
            self.funding.as_ref(),
        )?))
    }
    pub(crate) fn snapshot_run(
        &self,
        container: Node,
        nodes: &[Node],
        segment: &PreparedSourceSegment,
        input: crate::ComputeInputOf<S>,
        mode: crate::AxisSizingRequestOf<f64>,
    ) -> surgeist_text::Result<FormattedTailSnapshot> {
        let binding = self.binding(container, nodes, segment, input, mode)?;
        self.snapshot(&binding)
    }
    pub(crate) fn snapshot(
        &self,
        binding: &RunBinding<Node, S>,
    ) -> surgeist_text::Result<FormattedTailSnapshot> {
        let mut copies = EntryCopies::default();
        let mut count = 0;
        for value in &self.assignments {
            self.visit(1)?;
            if value
                .binding
                .same(binding, self.funding.as_ref().map(EntryFunding::as_ref))?
            {
                count += 1;
            }
        }
        let mut assignments = copies.rows(self.funding.as_ref(), count)?;
        for value in &self.assignments {
            self.visit(1)?;
            if value
                .binding
                .same(binding, self.funding.as_ref().map(EntryFunding::as_ref))?
            {
                assignments.push(value.assignment.clone());
            }
        }
        let assignments = if assignments.is_empty() {
            None
        } else if self.funding.is_none() {
            Some(SnapshotAssignments::Known(assignments.into()))
        } else {
            Some(SnapshotAssignments::Observed(SharedPayload::with_copies(
                assignments,
                copies,
                self.funding.as_ref(),
            )?))
        };
        Ok(FormattedTailSnapshot {
            segment: Some(binding.segment.clone()),
            assignments,
            observes: !self.demanded.is_empty(),
        })
    }
    pub(crate) fn merge_run(
        &mut self,
        container: Node,
        nodes: &[Node],
        segment: &PreparedSourceSegment,
        input: crate::ComputeInputOf<S>,
        mode: crate::AxisSizingRequestOf<f64>,
        uses: FormattedTailUses,
    ) -> surgeist_text::Result<FormattedFlowItem<Node, S>> {
        let binding = self.binding(container, nodes, segment, input, mode)?;
        let mut copies = EntryCopies::default();
        let endpoints = copied_rows(&uses.accepted, self.funding.as_ref(), &mut copies)?;
        let item = FormattedFlowItem::Text(binding.clone(), endpoints)
            .retained(copies, self.funding.as_ref())?;
        self.merge(binding, uses)?;
        Ok(item)
    }
    pub(crate) fn merge(
        &mut self,
        binding: RunBinding<Node, S>,
        uses: FormattedTailUses,
    ) -> surgeist_text::Result<()> {
        let mut copies = EntryCopies::default();
        let FormattedTailUses {
            demands,
            roles,
            accepted,
            storage,
        } = uses;
        copies.original_uses = storage;
        let uses = UsesContents {
            demands,
            roles,
            accepted,
        };
        let bound = BoundUses {
            binding,
            uses: SharedPayload::with_copies(uses, copies, self.funding.as_ref())?,
        };
        self.merge_bound(bound)
    }
    fn merge_bound(&mut self, bound: BoundUses<Node, S>) -> surgeist_text::Result<()> {
        if !bound.uses.demands.is_empty() {
            let mut found = false;
            for owner in &self.demanded {
                self.visit(1)?;
                if *owner == bound.binding.owner {
                    found = true;
                    break;
                }
            }
            if !found {
                self.demanded_backing
                    .reserve_push(&mut self.demanded, self.funding.as_ref())?;
                self.demanded.push(bound.binding.owner);
            }
        }
        for index in 0..self.uses.len() {
            self.visit(1)?;
            if self.uses[index].binding.same(
                &bound.binding,
                self.funding.as_ref().map(EntryFunding::as_ref),
            )? {
                self.replacing()?;
                self.uses[index] = bound;
                return Ok(());
            }
        }
        self.uses_backing
            .reserve_push(&mut self.uses, self.funding.as_ref())?;
        self.uses.push(bound);
        Ok(())
    }
    pub(crate) fn child(
        &self,
        scope: ComputeScope<Node, S>,
    ) -> surgeist_text::Result<FormattedFlowItem<Node, S>> {
        let mut copies = EntryCopies::default();
        let mut lineage = copies.rows(self.funding.as_ref(), self.stack.len() + 1)?;
        for parent in &self.stack {
            self.visit(1)?;
            lineage.push(parent.clone());
        }
        self.visit(1)?;
        lineage.push(scope);
        FormattedFlowItem::Child(lineage).retained(copies, self.funding.as_ref())
    }
    pub(crate) fn record(&mut self, flow: FormattedFlow<Node, S>) -> surgeist_text::Result<()> {
        if self.stack.is_empty() {
            return Ok(());
        }
        let mut copies = EntryCopies::default();
        let mut scope = copied_rows(&self.stack, self.funding.as_ref(), &mut copies)?;
        scope.last_mut().unwrap().frame = flow.frame;
        let flow = flow.retained(self.funding.as_ref())?;
        let record = ScopedFlow(SharedPayload::with_copies(
            ScopedContents { scope, flow },
            copies,
            self.funding.as_ref(),
        )?);
        self.store_flow(record)
    }
    fn store_flow(&mut self, record: ScopedFlow<Node, S>) -> surgeist_text::Result<()> {
        for index in 0..self.flows.len() {
            self.visit(1)?;
            if rows_equal(
                &self.flows[index].scope,
                &record.scope,
                self.funding.as_ref().map(EntryFunding::as_ref),
            )? {
                self.replacing()?;
                self.flows[index] = record;
                return Ok(());
            }
        }
        self.flow_backing
            .reserve_push(&mut self.flows, self.funding.as_ref())?;
        self.flows.push(record);
        Ok(())
    }
    fn same_call(
        &self,
        left: &[ComputeScope<Node, S>],
        right: &[ComputeScope<Node, S>],
    ) -> surgeist_text::Result<bool> {
        if left.len() != right.len() {
            return Ok(false);
        }
        let length = left.len();
        for (index, (left, right)) in left.iter().zip(right).enumerate() {
            self.visit(1)?;
            let mut a = left.clone();
            let mut b = right.clone();
            if index + 1 == length {
                a.frame = None;
                b.frame = None;
            }
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn within_call(
        &self,
        scope: &[ComputeScope<Node, S>],
        call: &[ComputeScope<Node, S>],
    ) -> surgeist_text::Result<bool> {
        Ok(scope.len() >= call.len()
            && !call.is_empty()
            && self.same_call(&scope[..call.len()], call)?)
    }
    pub(crate) fn select(&mut self, flow: FormattedFlow<Node, S>) -> surgeist_text::Result<()> {
        if self.stack.is_empty() {
            return Ok(());
        }
        let mut copies = EntryCopies::default();
        let mut actual = copied_rows(&self.stack, self.funding.as_ref(), &mut copies)?;
        actual.last_mut().unwrap().frame = flow.frame;
        let mut call = copied_rows(&self.stack, self.funding.as_ref(), &mut copies)?;
        call.last_mut().unwrap().frame = None;
        let selected = SelectedFlow(SharedPayload::with_copies(
            Selection { call, actual },
            copies,
            self.funding.as_ref(),
        )?);
        self.record(flow)?;
        self.store_selected(selected)
    }
    fn store_selected(&mut self, selected: SelectedFlow<Node, S>) -> surgeist_text::Result<()> {
        for index in 0..self.selected.len() {
            self.visit(1)?;
            if rows_equal(
                &self.selected[index].call,
                &selected.call,
                self.funding.as_ref().map(EntryFunding::as_ref),
            )? {
                self.replacing()?;
                self.selected[index] = selected;
                return Ok(());
            }
        }
        self.selected_backing
            .reserve_push(&mut self.selected, self.funding.as_ref())?;
        self.selected.push(selected);
        Ok(())
    }
    fn flow(
        &self,
        scope: &[ComputeScope<Node, S>],
    ) -> surgeist_text::Result<Option<&ScopedFlow<Node, S>>> {
        let mut actual = scope;
        for selected in &self.selected {
            self.visit(1)?;
            if rows_equal(
                &selected.call,
                scope,
                self.funding.as_ref().map(EntryFunding::as_ref),
            )? {
                actual = &selected.actual;
                break;
            }
        }
        for record in &self.flows {
            self.visit(1)?;
            if rows_equal(
                &record.scope,
                actual,
                self.funding.as_ref().map(EntryFunding::as_ref),
            )? {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }
    fn supplied(
        &self,
        binding: &RunBinding<Node, S>,
        assignment: &FormattedTailAssignment,
    ) -> surgeist_text::Result<bool> {
        for value in &self.assignments {
            self.visit(1)?;
            if value
                .binding
                .same(binding, self.funding.as_ref().map(EntryFunding::as_ref))?
                && value.assignment.endpoint == assignment.endpoint
                && value.assignment.predicate == assignment.predicate
            {
                return Ok(value.assignment.last);
            }
        }
        Ok(false)
    }
    fn owner_scope<'a>(
        &self,
        binding: &'a RunBinding<Node, S>,
    ) -> surgeist_text::Result<Option<&'a [ComputeScope<Node, S>]>> {
        for (index, scope) in binding.lineage.iter().enumerate().rev() {
            self.visit(1)?;
            if scope.node == binding.owner {
                return Ok(Some(&binding.lineage[..=index]));
            }
        }
        Ok(None)
    }
    fn events<'a>(
        &'a self,
        scope: &[ComputeScope<Node, S>],
    ) -> surgeist_text::Result<Option<Events<'a, Node, S>>> {
        let Some(record) = self.flow(scope)? else {
            return Ok(None);
        };
        let mut result = Events {
            rows: Vec::new(),
            backing: EntryBacking::default(),
        };
        let mut pending_backing = EntryBacking::default();
        let mut pending = Vec::new();
        for item in record.flow.items.iter().rev() {
            self.visit(1)?;
            pending_backing.reserve_push(&mut pending, self.funding.as_ref())?;
            pending.push(item);
        }
        while let Some(item) = pending.pop() {
            self.visit(1)?;
            match item.view() {
                FormattedFlowItem::Text(binding, lines) => {
                    for endpoint in lines {
                        self.visit(1)?;
                        result
                            .backing
                            .reserve_push(&mut result.rows, self.funding.as_ref())?;
                        result.rows.push(FlowEvent::Text(binding, endpoint));
                    }
                    result
                        .backing
                        .reserve_push(&mut result.rows, self.funding.as_ref())?;
                    result.rows.push(FlowEvent::RunEnd(binding));
                }
                FormattedFlowItem::Numeric(_, lines) => {
                    for _ in lines {
                        self.visit(1)?;
                        result
                            .backing
                            .reserve_push(&mut result.rows, self.funding.as_ref())?;
                        result.rows.push(FlowEvent::Numeric);
                    }
                }
                FormattedFlowItem::Child(child) => {
                    let Some(record) = self.flow(child)? else {
                        return Ok(None);
                    };
                    for item in record.flow.items.iter().rev() {
                        self.visit(1)?;
                        pending_backing.reserve_push(&mut pending, self.funding.as_ref())?;
                        pending.push(item);
                    }
                }
                FormattedFlowItem::Retained(_) => {
                    unreachable!("retained item view resolves its one owning wrapper")
                }
            }
        }
        drop(pending);
        drop(pending_backing);
        Ok(Some(result))
    }
    fn observe(
        &self,
        binding: &RunBinding<Node, S>,
        assignment: &FormattedTailAssignment,
    ) -> surgeist_text::Result<Option<bool>> {
        let Some(scope) = self.owner_scope(binding)? else {
            return Ok(None);
        };
        let Some(events) = self.events(scope)? else {
            return Ok(None);
        };
        let mut completion = None;
        for (index, event) in events.rows.iter().enumerate() {
            self.visit(1)?;
            if let FlowEvent::RunEnd(run) = event
                && run.same(binding, self.funding.as_ref().map(EntryFunding::as_ref))?
            {
                completion = Some(index);
                break;
            }
        }
        let Some(completion) = completion else {
            return Ok(None);
        };
        match assignment.predicate {
            TailPredicate::ExternalAfterRun => {
                for event in &events.rows[completion + 1..] {
                    self.visit(1)?;
                    if event.is_line() {
                        return Ok(Some(false));
                    }
                }
                Ok(Some(true))
            }
            TailPredicate::AcceptedEndpoint => {
                for event in events.rows.iter().rev() {
                    self.visit(1)?;
                    if event.is_line() {
                        let matches = match event {
                            FlowEvent::Text(run, end) => {
                                run.same(binding, self.funding.as_ref().map(EntryFunding::as_ref))?
                                    && **end == assignment.endpoint
                            }
                            _ => false,
                        };
                        return Ok(Some(matches));
                    }
                }
                Ok(Some(false))
            }
        }
    }
    pub(crate) fn valid(&self) -> surgeist_text::Result<bool> {
        for bound in &self.uses {
            self.visit(1)?;
            for role in &bound.uses.roles {
                self.visit(1)?;
                if let Some(assignment) = role.last_assignment()
                    && self.observe(&bound.binding, assignment)? != Some(assignment.last)
                {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    pub(crate) fn basis(&self) -> surgeist_text::Result<FormattedPassBasis<Node, S>> {
        let mut copies = EntryCopies::default();
        let mut observed_backing = EntryBacking::default();
        let mut observed = Vec::new();
        for bound in &self.uses {
            self.visit(1)?;
            for assignment in &bound.uses.demands {
                self.visit(1)?;
                if let Some(last) = self.observe(&bound.binding, assignment)? {
                    observed_backing.reserve_push(&mut observed, self.funding.as_ref())?;
                    observed.push(BoundAssignment {
                        binding: bound.binding.clone(),
                        assignment: FormattedTailAssignment {
                            last,
                            ..assignment.clone()
                        },
                    });
                }
            }
        }
        copies.row_backing = observed_backing;
        let supplied = copied_rows(&self.assignments, self.funding.as_ref(), &mut copies)?;
        let uses = copied_rows(&self.uses, self.funding.as_ref(), &mut copies)?;
        let flows = copied_rows(&self.flows, self.funding.as_ref(), &mut copies)?;
        let selected = copied_rows(&self.selected, self.funding.as_ref(), &mut copies)?;
        Ok(FormattedPassBasis(SharedPayload::with_copies(
            PassContents {
                wrapping: self.wrapping.clone(),
                frame_basis: self.frames.clone(),
                supplied,
                uses,
                flows,
                observed,
                selected,
            },
            copies,
            self.funding.as_ref(),
        )?))
    }
    fn store_assignment(&mut self, next: BoundAssignment<Node, S>) -> surgeist_text::Result<()> {
        for index in 0..self.assignments.len() {
            self.visit(1)?;
            let old = &self.assignments[index];
            if old.binding.same(
                &next.binding,
                self.funding.as_ref().map(EntryFunding::as_ref),
            )? && old.assignment.endpoint == next.assignment.endpoint
                && old.assignment.predicate == next.assignment.predicate
            {
                self.replacing()?;
                self.assignments[index] = next;
                return Ok(());
            }
        }
        self.assignment_backing
            .reserve_push(&mut self.assignments, self.funding.as_ref())?;
        self.assignments.push(next);
        Ok(())
    }
    pub(crate) fn next_pass(&mut self) -> surgeist_text::Result<()> {
        let basis = self.basis()?;
        for next in &basis.observed {
            self.visit(1)?;
            self.store_assignment(next.clone())?;
        }
        self.stack.clear();
        self.uses.clear();
        self.flows.clear();
        self.selected.clear();
        Ok(())
    }
    pub(crate) fn finish_block(
        &mut self,
        scope: ComputeScope<Node, S>,
        success: bool,
    ) -> surgeist_text::Result<()> {
        let Some(last) = self.stack.last() else {
            return Ok(());
        };
        let mut last = last.clone();
        last.frame = None;
        self.visit(1)?;
        if last != scope {
            return Ok(());
        }
        if !success {
            self.stack.pop();
            return Ok(());
        }
        let mut call_copies = EntryCopies::default();
        let mut call = copied_rows(&self.stack, self.funding.as_ref(), &mut call_copies)?;
        call.last_mut().unwrap().frame = None;
        if success && self.flow(&call)?.is_some() {
            let mut copies = EntryCopies::default();
            let parent = copied_rows(
                &self.stack[..self.stack.len() - 1],
                self.funding.as_ref(),
                &mut copies,
            )?;
            let mut flow_backing = EntryBacking::default();
            let mut flows = Vec::new();
            for flow in &self.flows {
                self.visit(1)?;
                if self.within_call(&flow.scope, &self.stack)? {
                    flow_backing.reserve_push(&mut flows, self.funding.as_ref())?;
                    flows.push(flow.clone());
                }
            }
            copies.row_backing = flow_backing;
            let mut use_backing = EntryBacking::default();
            let mut uses = Vec::new();
            for bound in &self.uses {
                self.visit(1)?;
                if self.within_call(&bound.binding.lineage, &self.stack)? {
                    use_backing.reserve_push(&mut uses, self.funding.as_ref())?;
                    uses.push(bound.clone());
                }
            }
            copies.uses_backing = use_backing;
            let mut selected_backing = EntryBacking::default();
            let mut selected = Vec::new();
            for value in &self.selected {
                self.visit(1)?;
                if self.within_call(&value.call, &self.stack)? {
                    selected_backing.reserve_push(&mut selected, self.funding.as_ref())?;
                    selected.push(value.clone());
                }
            }
            copies.selected_backing = selected_backing;
            let receipt = SummaryReceipt {
                output: None,
                contents: SharedPayload::with_copies(
                    SummaryContents {
                        wrapping: self.wrapping.clone(),
                        frames: self.frames.clone(),
                        node: scope.node,
                        input: scope.input,
                        parent,
                        flows,
                        uses,
                        selected,
                    },
                    copies,
                    self.funding.as_ref(),
                )?,
            };
            self.store_receipt(receipt)?;
        }
        drop(call);
        drop(call_copies);
        self.stack.pop();
        Ok(())
    }
    fn store_receipt(&mut self, receipt: SummaryReceipt<Node, S>) -> surgeist_text::Result<()> {
        for index in 0..self.receipts.len() {
            self.visit(1)?;
            let old = &self.receipts[index];
            if old.node == receipt.node
                && old.input == receipt.input
                && rows_equal(
                    &old.parent,
                    &receipt.parent,
                    self.funding.as_ref().map(EntryFunding::as_ref),
                )?
            {
                self.replacing()?;
                self.receipts[index] = receipt;
                return Ok(());
            }
        }
        self.receipt_backing
            .reserve_push(&mut self.receipts, self.funding.as_ref())?;
        self.receipts.push(receipt);
        Ok(())
    }
    pub(crate) fn receipt(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> surgeist_text::Result<Option<&SummaryReceipt<Node, S>>> {
        for receipt in &self.receipts {
            self.visit(1)?;
            if receipt.wrapping != self.wrapping
                || receipt.frames != self.frames
                || receipt.node != node
                || receipt.input != *input
                || !rows_equal(
                    &receipt.parent,
                    &self.stack,
                    self.funding.as_ref().map(EntryFunding::as_ref),
                )?
            {
                continue;
            }
            let mut valid = true;
            for bound in &receipt.uses {
                self.visit(1)?;
                for role in &bound.uses.roles {
                    self.visit(1)?;
                    if let Some(assignment) = role.last_assignment()
                        && self.supplied(&bound.binding, assignment)? != assignment.last
                    {
                        valid = false;
                        break;
                    }
                }
                if !valid {
                    break;
                }
            }
            if valid {
                return Ok(Some(receipt));
            }
        }
        Ok(None)
    }
    pub(crate) fn restore(
        &mut self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> surgeist_text::Result<bool> {
        let Some(receipt) = self.receipt(node, input)?.cloned() else {
            return Ok(false);
        };
        for flow in &receipt.flows {
            self.visit(1)?;
            self.store_flow(flow.clone())?;
        }
        for bound in &receipt.uses {
            self.visit(1)?;
            self.merge_bound(bound.clone())?;
        }
        for selected in &receipt.selected {
            self.visit(1)?;
            self.store_selected(selected.clone())?;
        }
        Ok(true)
    }
    pub(crate) fn bind_output(
        &mut self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
        context: crate::CacheKeyContext,
        output: crate::ComputeOutputOf<S>,
    ) -> surgeist_text::Result<()> {
        for index in 0..self.receipts.len() {
            self.visit(1)?;
            let receipt = &self.receipts[index];
            if receipt.node == node
                && receipt.input == *input
                && rows_equal(
                    &receipt.parent,
                    &self.stack,
                    self.funding.as_ref().map(EntryFunding::as_ref),
                )?
            {
                self.replacing()?;
                self.receipts[index].output = Some((context, output));
                break;
            }
        }
        Ok(())
    }
    pub(crate) fn cache_available(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> surgeist_text::Result<bool> {
        Ok(self
            .receipt(node, input)?
            .is_some_and(|receipt| receipt.output.is_some()))
    }
    pub(crate) fn cache_matches(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
        context: crate::CacheKeyContext,
        output: crate::ComputeOutputOf<S>,
    ) -> surgeist_text::Result<bool> {
        Ok(self
            .receipt(node, input)?
            .is_some_and(|receipt| receipt.output == Some((context, output))))
    }
    pub(crate) fn fork(&self) -> Self {
        Self {
            wrapping: self.wrapping.clone(),
            frames: self.frames.clone(),
            stack: Vec::new(),
            assignments: self.assignments.clone(),
            demanded: self.demanded.clone(),
            uses: Vec::new(),
            flows: Vec::new(),
            selected: Vec::new(),
            receipts: Vec::new(),
            stack_backing: EntryBacking::default(),
            assignment_backing: EntryBacking::default(),
            uses_backing: EntryBacking::default(),
            flow_backing: EntryBacking::default(),
            selected_backing: EntryBacking::default(),
            receipt_backing: EntryBacking::default(),
            demanded_backing: EntryBacking::default(),
            funding: None,
        }
    }
    pub(crate) fn fork_in_flow(&self) -> Self {
        Self {
            stack: self.stack.clone(),
            ..self.fork()
        }
    }
    pub(crate) fn fork_entry(
        &self,
        funding: Option<&EntryFunding<S>>,
        in_flow: bool,
    ) -> surgeist_text::Result<Self> {
        let Some(funding) = funding else {
            return Ok(if in_flow {
                self.fork_in_flow()
            } else {
                self.fork()
            });
        };
        let mut state = Self::new_entry(Some(funding), self.frames.clone(), self.wrapping.clone())?;
        let (lease, mut assignments) = funding
            .buffer(self.assignments.len())
            .map(|(rows, lease)| (lease, rows))?;
        for value in &self.assignments {
            funding.visit(1)?;
            assignments.push(BoundAssignment {
                binding: value.binding.retain(Some(funding))?,
                assignment: value.assignment.clone(),
            });
        }
        (state.assignments, state.assignment_backing) =
            EntryBacking::from_buffer(assignments, self.assignments.len(), lease, funding)?;
        (state.demanded, state.demanded_backing) = state_rows(&self.demanded, Some(funding))?;
        if in_flow {
            (state.stack, state.stack_backing) = state_rows(&self.stack, Some(funding))?;
        }
        Ok(state)
    }
    pub(crate) fn entry_basis(
        &self,
        funding: Option<&EntryFunding<S>>,
        in_flow: bool,
    ) -> surgeist_text::Result<EntryFormattedBasis<Node, S>> {
        let mut copies = EntryCopies::default();
        let mut assignments = copies.rows(funding, self.assignments.len())?;
        for value in &self.assignments {
            visit(funding, 1)?;
            assignments.push(BoundAssignment {
                binding: value.binding.retain(funding)?,
                assignment: value.assignment.clone(),
            });
        }
        let parent = if in_flow {
            Some(copied_rows(&self.stack, funding, &mut copies)?)
        } else {
            None
        };
        Ok(EntryFormattedBasis {
            assignments,
            parent,
            _copies: copies,
        })
    }
    pub(crate) fn accept(&mut self, other: &Self) -> surgeist_text::Result<()> {
        for &owner in &other.demanded {
            self.visit(1)?;
            let mut found = false;
            for known in &self.demanded {
                self.visit(1)?;
                if *known == owner {
                    found = true;
                    break;
                }
            }
            if !found {
                self.demanded_backing
                    .reserve_push(&mut self.demanded, self.funding.as_ref())?;
                self.demanded.push(owner);
            }
        }
        for bound in &other.uses {
            self.visit(1)?;
            self.merge_bound(bound.clone())?;
        }
        for flow in &other.flows {
            self.visit(1)?;
            self.store_flow(flow.clone())?;
        }
        for selected in &other.selected {
            self.visit(1)?;
            self.store_selected(selected.clone())?;
        }
        for receipt in &other.receipts {
            self.visit(1)?;
            self.store_receipt(receipt.clone())?;
        }
        Ok(())
    }
}
struct Events<'a, Node, S: crate::LayoutScalar> {
    rows: Vec<FlowEvent<'a, Node, S>>,
    backing: EntryBacking,
}
enum FlowEvent<'a, Node, S: crate::LayoutScalar> {
    Text(&'a RunBinding<Node, S>, &'a FormattedTailEndpoint),
    Numeric,
    RunEnd(&'a RunBinding<Node, S>),
}
impl<Node, S: crate::LayoutScalar> FlowEvent<'_, Node, S> {
    fn is_line(&self) -> bool {
        !matches!(self, Self::RunEnd(_))
    }
}
