//! Invocation-local formatted-line authority. This records emitted flow, not
//! text content, geometry guesses or a second line formatter.

use std::sync::Arc;

use surgeist_text::{PreparedAlternative, PreparedBoundary, PreparedText};

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
    prepared: Option<PreparedText>,
    assignments: Option<Arc<[FormattedTailAssignment]>>,
    observes: bool,
}

impl FormattedTailSnapshot {
    pub(crate) fn known(
        prepared: PreparedText,
        assignments: Option<Arc<[FormattedTailAssignment]>>,
    ) -> Self {
        Self {
            prepared: Some(prepared),
            assignments,
            observes: false,
        }
    }

    pub(crate) fn observes(&self) -> bool {
        self.observes
    }

    fn observing(mut self, observes: bool) -> Self {
        self.observes = observes;
        self
    }

    pub(crate) fn resolve(
        &self,
        boundary: &PreparedBoundary,
        alternative: Option<&PreparedAlternative>,
    ) -> Option<FormattedTailAssignment> {
        self.resolve_predicate(boundary, alternative, TailPredicate::AcceptedEndpoint)
    }

    pub(crate) fn resolve_external(
        &self,
        boundary: &PreparedBoundary,
        alternative: Option<&PreparedAlternative>,
    ) -> Option<FormattedTailAssignment> {
        self.resolve_predicate(boundary, alternative, TailPredicate::ExternalAfterRun)
    }

    fn resolve_predicate(
        &self,
        boundary: &PreparedBoundary,
        alternative: Option<&PreparedAlternative>,
        predicate: TailPredicate,
    ) -> Option<FormattedTailAssignment> {
        let prepared = self.prepared.as_ref()?;
        if boundary.prepared_text() != *prepared
            || alternative.is_some_and(|value| value.prepared_text() != *prepared)
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

/// Returned semantic observations include role-dependent decisions even when
/// their candidate was not ultimately emitted. Prospective role uses retain
/// their own local flow proof and external dependency in the adapter's RoleUse.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FormattedTailUses {
    pub(crate) demands: Vec<FormattedTailAssignment>,
    pub(crate) roles: Vec<super::fit::RoleUse>,
    pub(crate) accepted: Vec<FormattedTailEndpoint>,
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
    pub(crate) available: crate::AvailableOf<S>,
    pub(crate) publication: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SelectedFlow<Node, S: crate::LayoutScalar> {
    pub(crate) call: Vec<ComputeScope<Node, S>>,
    pub(crate) actual: Vec<ComputeScope<Node, S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RunBinding<Node, S: crate::LayoutScalar> {
    pub(crate) container: Node,
    pub(crate) nodes: Vec<Node>,
    pub(crate) prepared: PreparedText,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) mode: super::fit::Available,
    pub(crate) lineage: Vec<ComputeScope<Node, S>>,
    pub(crate) owner: Node,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FormattedFlowItem<Node, S: crate::LayoutScalar> {
    Text(RunBinding<Node, S>, Vec<FormattedTailEndpoint>),
    Numeric(Vec<Node>, Vec<NumericFormattedLine>),
    Child(Vec<ComputeScope<Node, S>>),
}

/// Produced by one actual block in-flow traversal. A child reference is added
/// only by the accepting in-flow block owner, never by atomic/float probes.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FormattedFlow<Node, S: crate::LayoutScalar> {
    pub(crate) frame: Option<FlowFrame<S>>,
    pub(crate) items: Vec<FormattedFlowItem<Node, S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoundUses<Node, S: crate::LayoutScalar> {
    pub(crate) binding: RunBinding<Node, S>,
    pub(crate) uses: FormattedTailUses,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScopedFlow<Node, S: crate::LayoutScalar> {
    pub(crate) scope: Vec<ComputeScope<Node, S>>,
    pub(crate) flow: FormattedFlow<Node, S>,
}

/// This receipt exists only in one ComputeSession and its unpublished replays.
/// The host geometry cache has no storage for it and cannot manufacture one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SummaryReceipt<Node, S: crate::LayoutScalar> {
    pub(crate) wrapping: Option<Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    pub(crate) output: Option<(crate::CacheKeyContext, crate::ComputeOutputOf<S>)>,
    pub(crate) frames: Arc<[super::state::CanonicalFrame<Node, S>]>,
    pub(crate) node: Node,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) parent: Vec<ComputeScope<Node, S>>,
    pub(crate) flows: Vec<ScopedFlow<Node, S>>,
    pub(crate) uses: Vec<BoundUses<Node, S>>,
    pub(crate) selected: Vec<SelectedFlow<Node, S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoundAssignment<Node, S: crate::LayoutScalar> {
    pub(crate) binding: RunBinding<Node, S>,
    pub(crate) assignment: FormattedTailAssignment,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct FormattedPassBasis<Node, S: crate::LayoutScalar> {
    pub(crate) wrapping: Option<Arc<crate::inline_wrapping::WrappingContext<Node>>>,
    pub(crate) frame_basis: Arc<[super::state::CanonicalFrame<Node, S>]>,
    pub(crate) supplied: Vec<BoundAssignment<Node, S>>,
    pub(crate) uses: Vec<BoundUses<Node, S>>,
    pub(crate) flows: Vec<ScopedFlow<Node, S>>,
    pub(crate) observed: Vec<BoundAssignment<Node, S>>,
    pub(crate) selected: Vec<SelectedFlow<Node, S>>,
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
}

impl<Node: Copy + Eq, S: crate::LayoutScalar> FormattedState<Node, S> {
    pub(crate) fn new() -> Self {
        Self {
            wrapping: None,
            frames: Arc::from([]),
            stack: Vec::new(),
            assignments: Vec::new(),
            uses: Vec::new(),
            flows: Vec::new(),
            selected: Vec::new(),
            receipts: Vec::new(),
            demanded: Vec::new(),
        }
    }

    pub(crate) fn binding(
        &self,
        container: Node,
        nodes: &[Node],
        prepared: &PreparedText,
        input: crate::ComputeInputOf<S>,
        mode: super::fit::Available,
    ) -> RunBinding<Node, S> {
        let owner = self
            .stack
            .iter()
            .rev()
            .find(|scope| !scope.anonymous)
            .map_or(container, |scope| scope.node);
        RunBinding {
            container,
            nodes: nodes.to_vec(),
            prepared: prepared.clone(),
            input,
            mode,
            lineage: self.stack.clone(),
            owner,
        }
    }

    pub(crate) fn snapshot(&self, binding: &RunBinding<Node, S>) -> FormattedTailSnapshot {
        let assignments = self
            .assignments
            .iter()
            .filter(|value| value.binding == *binding)
            .map(|value| value.assignment.clone())
            .collect::<Vec<_>>();
        FormattedTailSnapshot::known(
            binding.prepared.clone(),
            (!assignments.is_empty()).then(|| assignments.into()),
        )
        .observing(!self.demanded.is_empty())
    }

    pub(crate) fn merge(&mut self, binding: RunBinding<Node, S>, uses: FormattedTailUses) {
        if !uses.demands.is_empty() && !self.demanded.contains(&binding.owner) {
            self.demanded.push(binding.owner);
        }
        let bound = BoundUses { binding, uses };
        // One semantic run invocation replaces its previous speculative pass;
        // different actual compute purposes and owner lineages remain separate.
        if let Some(old) = self
            .uses
            .iter_mut()
            .find(|old| old.binding == bound.binding)
        {
            *old = bound;
        } else {
            self.uses.push(bound);
        }
    }

    pub(crate) fn record(&mut self, flow: FormattedFlow<Node, S>) {
        if self.stack.is_empty() {
            return;
        }
        let mut scope = self.stack.clone();
        scope.last_mut().unwrap().frame = flow.frame;
        let record = ScopedFlow { scope, flow };
        if let Some(old) = self.flows.iter_mut().find(|old| old.scope == record.scope) {
            *old = record;
        } else {
            self.flows.push(record);
        }
    }

    fn call_scope(scope: &[ComputeScope<Node, S>]) -> Vec<ComputeScope<Node, S>> {
        let mut call = scope.to_vec();
        if let Some(last) = call.last_mut() {
            last.frame = None;
        }
        call
    }

    fn within_call(scope: &[ComputeScope<Node, S>], call: &[ComputeScope<Node, S>]) -> bool {
        scope.len() >= call.len()
            && !call.is_empty()
            && Self::call_scope(&scope[..call.len()]) == Self::call_scope(call)
    }

    pub(crate) fn select(&mut self, flow: FormattedFlow<Node, S>) {
        if self.stack.is_empty() {
            return;
        }
        let mut actual = self.stack.clone();
        actual.last_mut().unwrap().frame = flow.frame;
        let selected = SelectedFlow {
            call: Self::call_scope(&self.stack),
            actual,
        };
        self.record(flow);
        if let Some(old) = self
            .selected
            .iter_mut()
            .find(|old| old.call == selected.call)
        {
            *old = selected;
        } else {
            self.selected.push(selected);
        }
    }

    fn flow(&self, scope: &[ComputeScope<Node, S>]) -> Option<&ScopedFlow<Node, S>> {
        let actual = self
            .selected
            .iter()
            .find(|selected| selected.call == scope)
            .map_or(scope, |selected| selected.actual.as_slice());
        self.flows.iter().find(|record| record.scope == actual)
    }

    fn supplied(
        &self,
        binding: &RunBinding<Node, S>,
        assignment: &FormattedTailAssignment,
    ) -> bool {
        self.assignments
            .iter()
            .find(|value| {
                value.binding == *binding
                    && value.assignment.endpoint == assignment.endpoint
                    && value.assignment.predicate == assignment.predicate
            })
            .is_some_and(|value| value.assignment.last)
    }

    fn owner_scope(binding: &RunBinding<Node, S>) -> Option<&[ComputeScope<Node, S>]> {
        let owner = binding
            .lineage
            .iter()
            .rposition(|scope| scope.node == binding.owner)?;
        Some(&binding.lineage[..=owner])
    }

    fn events(
        &self,
        scope: &[ComputeScope<Node, S>],
        result: &mut Vec<FlowEvent<Node, S>>,
    ) -> Option<()> {
        let record = self.flow(scope)?;
        let mut pending = record.flow.items.iter().rev().collect::<Vec<_>>();
        while let Some(item) = pending.pop() {
            match item {
                FormattedFlowItem::Text(binding, lines) => {
                    result.extend(
                        lines
                            .iter()
                            .cloned()
                            .map(|endpoint| FlowEvent::Text(binding.clone(), endpoint)),
                    );
                    // A genuine run completion occurrence is not a formatted
                    // line. It authenticates external-after even for neutral runs.
                    result.push(FlowEvent::RunEnd(binding.clone()));
                }
                FormattedFlowItem::Numeric(_, lines) => {
                    result.extend(lines.iter().map(|_| FlowEvent::Numeric));
                }
                FormattedFlowItem::Child(child) => {
                    let record = self.flow(child)?;
                    pending.extend(record.flow.items.iter().rev());
                }
            }
        }
        Some(())
    }

    fn observe(
        &self,
        binding: &RunBinding<Node, S>,
        assignment: &FormattedTailAssignment,
    ) -> Option<bool> {
        let scope = Self::owner_scope(binding)?;
        let mut events = Vec::new();
        self.events(scope, &mut events)?;
        // Reaching this marker proves the exact actual run occurs in the
        // owner's accepted normal-flow traversal, independent of target membership.
        let completion = events.iter().position(|event| {
            matches!(event,
            FlowEvent::RunEnd(run) if run == binding)
        })?;
        Some(match assignment.predicate {
            TailPredicate::ExternalAfterRun => {
                !events[completion + 1..].iter().any(FlowEvent::is_line)
            }
            TailPredicate::AcceptedEndpoint => {
                matches!(events.iter().rev().find(|event| event.is_line()),
                Some(FlowEvent::Text(run, end)) if run == binding && *end == assignment.endpoint)
            }
        })
    }

    pub(crate) fn valid(&self) -> bool {
        self.uses.iter().all(|bound| {
            bound.uses.roles.iter().all(|role| {
                role.last_assignment().is_none_or(|assignment| {
                    self.observe(&bound.binding, assignment) == Some(assignment.last)
                })
            })
        })
    }

    pub(crate) fn basis(&self) -> FormattedPassBasis<Node, S> {
        let observed = self
            .uses
            .iter()
            .flat_map(|bound| {
                bound.uses.demands.iter().filter_map(|assignment| {
                    self.observe(&bound.binding, assignment)
                        .map(|last| BoundAssignment {
                            binding: bound.binding.clone(),
                            assignment: FormattedTailAssignment {
                                last,
                                ..assignment.clone()
                            },
                        })
                })
            })
            .collect();
        FormattedPassBasis {
            wrapping: self.wrapping.clone(),
            frame_basis: self.frames.clone(),
            supplied: self.assignments.clone(),
            uses: self.uses.clone(),
            flows: self.flows.clone(),
            observed,
            selected: self.selected.clone(),
        }
    }

    pub(crate) fn next_pass(&mut self) {
        let basis = self.basis();
        for next in basis.observed {
            if let Some(old) = self.assignments.iter_mut().find(|old| {
                old.binding == next.binding
                    && old.assignment.endpoint == next.assignment.endpoint
                    && old.assignment.predicate == next.assignment.predicate
            }) {
                *old = next;
            } else {
                self.assignments.push(next);
            }
        }
        self.stack.clear();
        self.uses.clear();
        self.flows.clear();
        self.selected.clear();
    }

    pub(crate) fn finish_block(&mut self, scope: ComputeScope<Node, S>, success: bool) {
        if Self::call_scope(&self.stack).last() != Some(&scope) {
            return;
        }
        if success && self.flow(&Self::call_scope(&self.stack)).is_some() {
            let receipt = SummaryReceipt {
                wrapping: self.wrapping.clone(),
                output: None,
                frames: self.frames.clone(),
                node: scope.node,
                input: scope.input,
                parent: self.stack[..self.stack.len() - 1].to_vec(),
                flows: self
                    .flows
                    .iter()
                    .filter(|flow| Self::within_call(&flow.scope, &self.stack))
                    .cloned()
                    .collect(),
                uses: self
                    .uses
                    .iter()
                    .filter(|bound| Self::within_call(&bound.binding.lineage, &self.stack))
                    .cloned()
                    .collect(),
                selected: self
                    .selected
                    .iter()
                    .filter(|selected| Self::within_call(&selected.call, &self.stack))
                    .cloned()
                    .collect(),
            };
            if let Some(old) = self.receipts.iter_mut().find(|old| {
                old.node == receipt.node
                    && old.input == receipt.input
                    && old.parent == receipt.parent
            }) {
                *old = receipt;
            } else {
                self.receipts.push(receipt);
            }
        }
        self.stack.pop();
    }

    pub(crate) fn receipt(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
    ) -> Option<&SummaryReceipt<Node, S>> {
        self.receipts.iter().find(|receipt| {
            receipt.wrapping == self.wrapping
                && receipt.frames == self.frames
                && receipt.node == node
                && receipt.input == *input
                && receipt.parent == self.stack
                && receipt.uses.iter().all(|bound| {
                    bound.uses.roles.iter().all(|role| {
                        role.last_assignment().is_none_or(|assignment| {
                            self.supplied(&bound.binding, assignment) == assignment.last
                        })
                    })
                })
        })
    }

    pub(crate) fn restore(&mut self, node: Node, input: &crate::ComputeInputOf<S>) -> bool {
        let Some(receipt) = self.receipt(node, input).cloned() else {
            return false;
        };
        for flow in receipt.flows {
            if let Some(old) = self.flows.iter_mut().find(|old| old.scope == flow.scope) {
                *old = flow;
            } else {
                self.flows.push(flow);
            }
        }
        for bound in receipt.uses {
            self.merge(bound.binding, bound.uses);
        }
        for selected in receipt.selected {
            if let Some(old) = self
                .selected
                .iter_mut()
                .find(|old| old.call == selected.call)
            {
                *old = selected;
            } else {
                self.selected.push(selected);
            }
        }
        true
    }

    pub(crate) fn bind_output(
        &mut self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
        context: crate::CacheKeyContext,
        output: crate::ComputeOutputOf<S>,
    ) {
        if let Some(receipt) = self.receipts.iter_mut().find(|receipt| {
            receipt.node == node && receipt.input == *input && receipt.parent == self.stack
        }) {
            receipt.output = Some((context, output));
        }
    }

    pub(crate) fn cache_matches(
        &self,
        node: Node,
        input: &crate::ComputeInputOf<S>,
        context: crate::CacheKeyContext,
        output: crate::ComputeOutputOf<S>,
    ) -> bool {
        self.receipt(node, input)
            .is_some_and(|receipt| receipt.output == Some((context, output)))
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            wrapping: self.wrapping.clone(),
            frames: self.frames.clone(),
            assignments: self.assignments.clone(),
            demanded: self.demanded.clone(),
            ..Self::new()
        }
    }

    pub(crate) fn accept(&mut self, other: &Self) {
        for &owner in &other.demanded {
            if !self.demanded.contains(&owner) {
                self.demanded.push(owner);
            }
        }
        for bound in &other.uses {
            self.merge(bound.binding.clone(), bound.uses.clone());
        }
        for flow in &other.flows {
            if let Some(old) = self.flows.iter_mut().find(|old| old.scope == flow.scope) {
                *old = flow.clone();
            } else {
                self.flows.push(flow.clone());
            }
        }
        for selected in &other.selected {
            if let Some(old) = self
                .selected
                .iter_mut()
                .find(|old| old.call == selected.call)
            {
                *old = selected.clone();
            } else {
                self.selected.push(selected.clone());
            }
        }
        for receipt in &other.receipts {
            if let Some(old) = self.receipts.iter_mut().find(|old| {
                old.node == receipt.node
                    && old.input == receipt.input
                    && old.parent == receipt.parent
            }) {
                *old = receipt.clone();
            } else {
                self.receipts.push(receipt.clone());
            }
        }
    }
}

enum FlowEvent<Node, S: crate::LayoutScalar> {
    Text(RunBinding<Node, S>, FormattedTailEndpoint),
    Numeric,
    RunEnd(RunBinding<Node, S>),
}
impl<Node, S: crate::LayoutScalar> FlowEvent<Node, S> {
    fn is_line(&self) -> bool {
        match self {
            Self::Text(_, _) => true,
            Self::Numeric => true,
            Self::RunEnd(_) => false,
        }
    }
}
