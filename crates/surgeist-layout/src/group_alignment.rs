//! Accepted natural line groups and their immutable alignment-frame replay.
use crate::{AvailableOf, ComputeInputOf, LayoutScalar, TextGroupAlign};
use surgeist_text::{LineAlignmentInsets, LineBoxDirection};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericError {
    bits: u64,
    reason: crate::LayoutTextNumericError,
}
impl NumericError {
    pub(crate) fn new(value: f64, reason: crate::LayoutTextNumericError) -> Self {
        Self {
            bits: value.to_bits(),
            reason,
        }
    }
    pub(crate) fn value(self) -> f64 {
        f64::from_bits(self.bits)
    }
    pub(crate) fn reason(self) -> crate::LayoutTextNumericError {
        self.reason
    }
    pub(crate) fn layout<Node, S: LayoutScalar, M>(
        self,
        node: Node,
    ) -> crate::LayoutErrorOf<Node, S, M> {
        crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::TextPlacement,
            crate::LayoutErrorKindOf::TextNumeric {
                value: self.value(),
                reason: self.reason,
            },
        )
    }
}
fn finite(value: f64) -> Result<f64, NumericError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(NumericError::new(
            value,
            crate::LayoutTextNumericError::NonFinite,
        ))
    }
}
pub(crate) fn scalar<S: LayoutScalar>(value: f64) -> Result<S, NumericError> {
    finite(value)?;
    let result = S::from_f64(value);
    if !result.is_finite() {
        Err(NumericError::new(
            value,
            crate::LayoutTextNumericError::OutOfRange,
        ))
    } else if value != 0.0 && result == S::ZERO {
        Err(NumericError::new(
            value,
            crate::LayoutTextNumericError::NonZeroLost,
        ))
    } else {
        Ok(result)
    }
}
fn checked<S: LayoutScalar>(value: f64) -> Result<f64, NumericError> {
    scalar::<S>(value)?;
    Ok(value)
}
/// Add one actually consumed contribution. Cancellation is valid; absorption
/// of a requested nonzero delta is not. Convert at the canonical scalar boundary.
pub(crate) fn add<S: LayoutScalar>(base: f64, delta: f64) -> Result<f64, NumericError> {
    let result = crate::scalar::checked_geometry_add(base, delta)
        .map_err(|error| NumericError::new(error.value, error.reason))?;
    checked::<S>(result)
}
/// The consumed lower Text arithmetic order, including signed indent/advance.
pub(crate) fn canonical_capacity<S: LayoutScalar>(
    width: f64,
    insets: LineAlignmentInsets,
    indent: f64,
) -> Result<f64, NumericError> {
    let left = add::<S>(width, -insets.line_left())?;
    let right = add::<S>(left, -insets.line_right())?;
    add::<S>(right, -indent)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Snapshot {
    owners: Vec<(TextGroupAlign, f64)>,
}
impl Snapshot {
    pub(crate) fn is_active(&self) -> bool {
        !self.owners.is_empty()
    }
    pub(crate) fn insets<S: LayoutScalar>(
        &self,
        direction: LineBoxDirection,
    ) -> Result<LineAlignmentInsets, NumericError> {
        let mut left = 0.0;
        let mut right = 0.0;
        for &(align, padding) in &self.owners {
            scalar::<S>(padding)?;
            let (l, r) = match align {
                TextGroupAlign::None => (0.0, 0.0),
                TextGroupAlign::Left => (0.0, padding),
                TextGroupAlign::Right => (padding, 0.0),
                TextGroupAlign::Start if direction == LineBoxDirection::LeftToRight => {
                    (0.0, padding)
                }
                TextGroupAlign::Start => (padding, 0.0),
                TextGroupAlign::End if direction == LineBoxDirection::LeftToRight => (padding, 0.0),
                TextGroupAlign::End => (0.0, padding),
                TextGroupAlign::Center => {
                    let half = padding / 2.0;
                    if padding != 0.0 && half == 0.0 {
                        return Err(NumericError::new(
                            padding,
                            crate::LayoutTextNumericError::NonZeroLost,
                        ));
                    }
                    scalar::<S>(half)?;
                    (half, half)
                }
            };
            left = add::<S>(left, l)?;
            right = add::<S>(right, r)?;
            scalar::<S>(left)?;
            scalar::<S>(right)?;
        }
        // Each consumed side is already finite and nonnegative.
        LineAlignmentInsets::try_new(left, right)
            .map_err(|_| NumericError::new(left + right, crate::LayoutTextNumericError::NonFinite))
    }
}

use crate::block::float_context::shares_parent_context;

/// Host caches own sizing, not accepted group-member receipts. Only actual
/// same-BFC line-owning blocks need suppression; atomic/independent interiors
/// retain their ordinary sizing cache. Their own declarations are separate seeds.
pub(crate) fn dependencies<Tree: crate::LayoutTree>(
    tree: &Tree,
    graph: &crate::engine::validation::ValidationGraph<Tree::Node>,
) -> Option<Vec<Tree::Node>> {
    let mut pending = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, &node)| {
            let style = tree.node_input(node);
            (style.text_group_align != TextGroupAlign::None
                && style.display.inner_display() == crate::Display::Block)
                .then_some(index)
        })
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return None;
    }
    let mut included = vec![false; graph.nodes.len()];
    while let Some(index) = pending.pop() {
        let node = graph.nodes[index];
        if included[index] || tree.has_leaf_measurement(node) {
            continue;
        }
        included[index] = true;
        let style = tree.node_input(node);
        let axes = crate::FlowAxes::new(style.writing_mode, style.direction);
        pending.extend(graph.children[index].iter().copied().filter(|&child| {
            matches!(tree.layout_input(graph.nodes[child]),crate::LayoutInputViewOf::Box(style)
                if shares_parent_context(style,axes,crate::ParentFormattingContext::BlockFlow))
        }));
    }
    Some(
        graph
            .nodes
            .iter()
            .copied()
            .zip(included)
            .filter_map(|(node, included)| included.then_some(node))
            .collect(),
    )
}

#[derive(Clone, Debug, PartialEq)]
struct Invocation<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    frame: Option<(AvailableOf<S>, bool)>,
}
#[derive(Clone, Debug, PartialEq)]
struct Owner<Node, S: LayoutScalar> {
    path: Vec<Invocation<Node, S>>,
    align: TextGroupAlign,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Witness<S: LayoutScalar> {
    Text {
        request: Box<surgeist_text::LineSelectionRequest>,
        amounts: Vec<f64>,
    },
    Numeric {
        participants: Vec<crate::inline::MixedInlineParticipantOf<S>>,
        discarded: Vec<bool>,
        descriptor_start: usize,
        next: (usize, usize, bool),
        end: surgeist_text::LineEnd,
        band: crate::inline::LogicalLineBandOf<S>,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NaturalLine<S: LayoutScalar> {
    pub(crate) width: f64,
    pub(crate) indent: f64,
    pub(crate) advance: f64,
    pub(crate) direction: LineBoxDirection,
    pub(crate) used: LineAlignmentInsets,
    pub(crate) witness: Witness<S>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Member<Node, S: LayoutScalar> {
    issued: usize,
    placement: Vec<Invocation<Node, S>>,
    owners: Vec<OwnerSlot>,
    line: NaturalLine<S>,
}
#[derive(Clone, Debug, PartialEq)]
struct Assignment<Node, S: LayoutScalar> {
    owner: Owner<Node, S>,
    padding: f64,
    limiter: Option<usize>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Basis<Node, S: LayoutScalar> {
    members: Vec<Member<Node, S>>,
    plan: Plan<Node, S>,
    resolved_insets: Vec<LineAlignmentInsets>,
}
impl<Node, S: LayoutScalar> Basis<Node, S> {
    pub(crate) fn is_reproduced(&self) -> bool {
        self.members
            .iter()
            .zip(&self.resolved_insets)
            .all(|(member, insets)| member.line.used == *insets)
    }
}
/// Private slots index this invocation's authentic observations. The complete
/// semantic owner key is retained and revalidated before a prior value is used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OwnerSlot {
    namespace: usize,
    local: usize,
}
type Plan<Node, S> = Vec<Vec<Option<Assignment<Node, S>>>>;
#[derive(Clone)]
struct Active<Node, S: LayoutScalar> {
    invocation: Invocation<Node, S>,
    axes: crate::FlowAxes,
    align: TextGroupAlign,
    independent: bool,
    owner: Option<(OwnerSlot, f64)>,
    collecting: bool,
    members: Vec<Member<Node, S>>,
    selected: Vec<Member<Node, S>>,
}
#[derive(Clone)]
pub(crate) struct State<Node, S: LayoutScalar> {
    active: Vec<Active<Node, S>>,
    completed: Option<(Node, Vec<Member<Node, S>>)>,
    roots: Vec<Vec<Member<Node, S>>>,
    plan: std::sync::Arc<Plan<Node, S>>,
    next_issued: usize,
    namespace: usize,
    next_namespace: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    owners: Vec<Owner<Node, S>>,
    accepted_owners: Vec<(usize, Vec<Owner<Node, S>>)>,
}
impl<Node: Copy + Eq, S: LayoutScalar> State<Node, S> {
    pub(crate) fn new() -> Self {
        Self {
            active: Vec::new(),
            completed: None,
            roots: Vec::new(),
            plan: std::sync::Arc::new(Vec::new()),
            next_issued: 0,
            namespace: 0,
            next_namespace: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(1)),
            owners: Vec::new(),
            accepted_owners: Vec::new(),
        }
    }
    pub(crate) fn fork(&self) -> Self {
        // Reserve scratch namespace authority for this actual probe. A rejected
        // packet contributes no owner/member rows to the accepting computation.
        let namespace = self
            .next_namespace
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            plan: self.plan.clone(),
            namespace,
            next_namespace: self.next_namespace.clone(),
            ..Self::new()
        }
    }
    pub(crate) fn same_plan(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.plan, &other.plan)
            && std::sync::Arc::ptr_eq(&self.next_namespace, &other.next_namespace)
    }
    pub(crate) fn accept(&mut self, other: &Self) {
        let offset = self.next_issued;
        self.next_issued += other.next_issued;
        let remap = |members: &[Member<Node, S>]| {
            members
                .iter()
                .cloned()
                .map(|mut member| {
                    member.issued += offset;
                    member
                })
                .collect::<Vec<_>>()
        };
        self.roots
            .extend(other.roots.iter().map(|members| remap(members)));
        self.completed = other
            .completed
            .as_ref()
            .map(|(node, members)| (*node, remap(members)));
        self.accepted_owners
            .push((other.namespace, other.owners.clone()));
        self.accepted_owners.extend(other.accepted_owners.clone());
    }
    pub(crate) fn begin(
        &mut self,
        node: Node,
        input: ComputeInputOf<S>,
        style: &crate::NodeInputOf<S>,
    ) {
        let independent = self.active.is_empty()
            || !shares_parent_context(
                style,
                input.containing_layout_context().flow_axes(),
                input.parent_formatting_context(),
            );
        self.active.push(Active {
            invocation: Invocation {
                node,
                input,
                frame: None,
            },
            axes: crate::FlowAxes::new(style.writing_mode, style.direction),
            align: style.text_group_align,
            independent,
            owner: None,
            collecting: false,
            members: Vec::new(),
            selected: Vec::new(),
        });
    }
    pub(crate) fn begin_flow(
        &mut self,
        input: ComputeInputOf<S>,
        available: AvailableOf<S>,
        publication: bool,
    ) {
        let Some(active) = self.active.last_mut() else {
            return;
        };
        let actual_available = active.axes.logical_size(input.available()).inline;
        let available = if matches!(actual_available, AvailableOf::Definite(_)) || publication {
            available
        } else {
            actual_available
        };
        active.invocation.input = input;
        active.invocation.frame = Some((available, publication));
        active.collecting = true;
        active.members.clear();
        active.owner = None;
        if active.align == TextGroupAlign::None || !matches!(available, AvailableOf::Definite(_)) {
            return;
        }
        let owner = Owner {
            path: self
                .active
                .iter()
                .map(|active| active.invocation.clone())
                .collect(),
            align: self.active.last().expect("active flow").align,
        };
        let slot = OwnerSlot {
            namespace: self.namespace,
            local: self.owners.len(),
        };
        let padding = self
            .plan
            .get(slot.namespace)
            .and_then(|rows| rows.get(slot.local))
            .and_then(Option::as_ref)
            .filter(|assignment| assignment.owner == owner)
            .map_or(0.0, |assignment| assignment.padding);
        self.owners.push(owner);
        self.active.last_mut().expect("active flow").owner = Some((slot, padding));
    }
    pub(crate) fn end_flow(&mut self) -> Vec<Member<Node, S>> {
        self.active.last_mut().map_or_else(Vec::new, |active| {
            active.collecting = false;
            core::mem::take(&mut active.members)
        })
    }
    pub(crate) fn select(&mut self, members: Vec<Member<Node, S>>) {
        if let Some(active) = self.active.last_mut() {
            active.selected = members;
        }
    }
    pub(crate) fn end(&mut self, success: bool) {
        if let Some(active) = self.active.pop()
            && success
        {
            self.completed = Some((active.invocation.node, active.selected.clone()));
            if self.active.is_empty() || active.independent {
                self.roots.push(active.selected);
            }
        }
    }
    pub(crate) fn child(&mut self, node: Node) {
        // The accepting owner calls this immediately after its actual child
        // request. A measured/cache-only child issues no new line summary;
        // never search an older invocation with the same native node.
        let Some((actual, members)) = &self.completed else {
            return;
        };
        if *actual != node {
            return;
        }
        let members = members.clone();
        if let Some(active) = self.active.last_mut() {
            if active.collecting {
                active.members.extend(members);
            } else {
                active.selected.extend(members);
            }
        }
    }
    fn active_owners(&self) -> &[Active<Node, S>] {
        if !self.active.last().is_some_and(|active| {
            active
                .invocation
                .frame
                .is_some_and(|(width, _)| matches!(width, AvailableOf::Definite(_)))
        }) {
            return &[];
        }
        let first = self
            .active
            .iter()
            .rposition(|active| active.independent)
            .unwrap_or(0);
        &self.active[first..]
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            owners: self
                .active_owners()
                .iter()
                .filter_map(|active| active.owner.map(|(_, padding)| (active.align, padding)))
                .collect(),
        }
    }
    pub(crate) fn observe(&mut self, line: NaturalLine<S>) {
        let owners = self
            .active_owners()
            .iter()
            .filter_map(|active| active.owner.map(|(slot, _)| slot))
            .collect::<Vec<_>>();
        if owners.is_empty() {
            return;
        }
        let placement = self
            .active
            .iter()
            .map(|active| active.invocation.clone())
            .collect();
        let issued = self.next_issued;
        self.next_issued += 1;
        self.active
            .last_mut()
            .expect("active member owner")
            .members
            .push(Member {
                issued,
                placement,
                owners,
                line,
            });
    }
    pub(crate) fn basis(&self) -> Result<Basis<Node, S>, NumericError> {
        let mut copied = vec![false; self.next_issued];
        let members = self
            .roots
            .iter()
            .flatten()
            .filter(|member| {
                let duplicate = copied[member.issued];
                copied[member.issued] = true;
                !duplicate
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut inventory = vec![
            Vec::new();
            self.next_namespace
                .load(std::sync::atomic::Ordering::Relaxed)
        ];
        inventory[self.namespace] = self.owners.clone();
        for (namespace, owners) in &self.accepted_owners {
            inventory[*namespace] = owners.clone();
        }
        // Build exact owner/member incidence lists once. Resolution visits only
        // the actual member list of each owner, never unrelated global cohorts.
        let mut member_lists = inventory
            .iter()
            .map(|owners| vec![Vec::new(); owners.len()])
            .collect::<Vec<_>>();
        for (index, member) in members.iter().enumerate() {
            for owner in &member.owners {
                member_lists[owner.namespace][owner.local].push(index);
            }
        }
        let order = inventory
            .iter()
            .enumerate()
            .flat_map(|(namespace, owners)| {
                (0..owners.len()).map(move |local| OwnerSlot { namespace, local })
            })
            .collect::<Vec<_>>();
        // Slots are issued when an actual owner enters its flow, before any
        // member or descendant owner. Probes start an independent namespace;
        // no member mixes namespaces. Issuance order is therefore topological
        // without comparing or sorting unrelated owner keys.
        let mut insets = vec![LineAlignmentInsets::ZERO; members.len()];
        let mut plan = inventory
            .iter()
            .map(|owners| vec![None; owners.len()])
            .collect::<Plan<Node, S>>();
        for slot in order {
            let owner = &inventory[slot.namespace][slot.local];
            let indices = &member_lists[slot.namespace][slot.local];
            let mut minimum = None;
            for &index in indices {
                let member = &members[index];
                let remaining = add::<S>(
                    canonical_capacity::<S>(member.line.width, insets[index], member.line.indent)?,
                    -member.line.advance,
                )?;
                if minimum.is_none_or(|(_, value)| remaining < value) {
                    minimum = Some((index, remaining));
                }
            }
            let padding = minimum.map_or(0.0, |(_, value)| value.max(0.0));
            scalar::<S>(padding)?;
            for &index in indices {
                let addition = Snapshot {
                    owners: vec![(owner.align, padding)],
                }
                .insets::<S>(members[index].line.direction)?;
                let left = add::<S>(insets[index].line_left(), addition.line_left())?;
                let right = add::<S>(insets[index].line_right(), addition.line_right())?;
                insets[index] = LineAlignmentInsets::try_new(left, right).map_err(|_| {
                    NumericError::new(left + right, crate::LayoutTextNumericError::NonFinite)
                })?;
            }
            plan[slot.namespace][slot.local] = Some(Assignment {
                owner: owner.clone(),
                padding,
                limiter: minimum.map(|(index, _)| index),
            });
        }
        Ok(Basis {
            members,
            plan,
            resolved_insets: insets,
        })
    }
    pub(crate) fn next_pass(&mut self, basis: &Basis<Node, S>) {
        let plan = std::sync::Arc::new(basis.plan.clone());
        *self = Self {
            plan,
            ..Self::new()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_directional_padding_uses_the_actual_line_sides() {
        let snapshot = Snapshot {
            owners: vec![
                (TextGroupAlign::Start, 40.0),
                (TextGroupAlign::End, 20.0),
                (TextGroupAlign::Center, 10.0),
            ],
        };
        let ltr = snapshot
            .insets::<f64>(LineBoxDirection::LeftToRight)
            .unwrap();
        let rtl = snapshot
            .insets::<f64>(LineBoxDirection::RightToLeft)
            .unwrap();
        assert_eq!((ltr.line_left(), ltr.line_right()), (25.0, 45.0));
        assert_eq!((rtl.line_left(), rtl.line_right()), (45.0, 25.0));
        assert_eq!(canonical_capacity::<f64>(100.0, ltr, -10.0).unwrap(), 40.0);
    }
    #[test]
    fn center_refuses_losing_a_positive_half_in_either_scalar_lane() {
        for padding in [f64::from_bits(1), f32::from_bits(1) as f64] {
            let snapshot = Snapshot {
                owners: vec![(TextGroupAlign::Center, padding)],
            };
            let error = if padding == f64::from_bits(1) {
                snapshot
                    .insets::<f64>(LineBoxDirection::LeftToRight)
                    .unwrap_err()
            } else {
                snapshot
                    .insets::<f32>(LineBoxDirection::LeftToRight)
                    .unwrap_err()
            };
            assert_eq!(error.reason(), crate::LayoutTextNumericError::NonZeroLost);
        }
    }
    #[test]
    fn consumed_canonical_capacity_refuses_overflow_before_later_cancellation() {
        let maximum = f32::MAX as f64;
        let error =
            canonical_capacity::<f32>(maximum, LineAlignmentInsets::ZERO, -maximum).unwrap_err();
        assert_eq!(error.reason(), crate::LayoutTextNumericError::OutOfRange);
        let insets = LineAlignmentInsets::try_new(40.0, 10.0).unwrap();
        assert_eq!(
            canonical_capacity::<f64>(20.0, insets, -50.0).unwrap(),
            20.0
        );
    }
    #[test]
    fn active_frames_refuse_an_absorbed_side_or_nested_contribution() {
        let insets = LineAlignmentInsets::try_new(1.0, 0.0).unwrap();
        let error = canonical_capacity::<f64>(1e20, insets, 0.0).unwrap_err();
        assert_eq!(error.reason(), crate::LayoutTextNumericError::NonZeroLost);
        let snapshot = Snapshot {
            owners: vec![(TextGroupAlign::Right, 1e20), (TextGroupAlign::Right, 1.0)],
        };
        assert_eq!(
            snapshot
                .insets::<f64>(LineBoxDirection::LeftToRight)
                .unwrap_err()
                .reason(),
            crate::LayoutTextNumericError::NonZeroLost
        );
        assert_eq!(add::<f64>(16.0, -16.0).unwrap(), 0.0);
        assert_eq!(
            scalar::<f32>(f64::MAX).unwrap_err().reason(),
            crate::LayoutTextNumericError::OutOfRange
        );
    }
}
