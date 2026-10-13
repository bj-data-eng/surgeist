//! Finite fixture recipes. Domain payloads, validity and concrete replay codecs
//! remain consumer-owned. These helpers never retry or repair rejected requests.
#![forbid(unsafe_code)]

use std::mem::size_of;

use crate::{FuzzError, FuzzErrorKind, InputLimits, InputUsage, Result};

fn resource(detail: &'static str) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Resource, "generate bounded fixture", detail)
}

fn invalid(detail: &'static str) -> FuzzError {
    FuzzError::new(
        FuzzErrorKind::InvalidCase,
        "generate bounded fixture",
        detail,
    )
}

/// Explicit resource credit, including consumer-owned payload costs. Depth is
/// a maximum; the other dimensions accumulate. Bytes include container slots.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BudgetCharge {
    pub bytes: u64,
    pub nodes: u64,
    pub edges: u64,
    pub depth: u64,
    pub text_units: u64,
    pub operations: u64,
}

impl BudgetCharge {
    fn combine(self, other: Self) -> Result<Self> {
        let add = |a: u64, b: u64| {
            a.checked_add(b)
                .ok_or_else(|| resource("resource credit overflow"))
        };
        Ok(Self {
            bytes: add(self.bytes, other.bytes)?,
            nodes: add(self.nodes, other.nodes)?,
            edges: add(self.edges, other.edges)?,
            depth: self.depth.max(other.depth),
            text_units: add(self.text_units, other.text_units)?,
            operations: add(self.operations, other.operations)?,
        })
    }

    fn repeated(self, count: u64) -> Result<Self> {
        let mul = |value: u64| {
            value
                .checked_mul(count)
                .ok_or_else(|| resource("resource credit overflow"))
        };
        Ok(Self {
            bytes: mul(self.bytes)?,
            nodes: mul(self.nodes)?,
            edges: mul(self.edges)?,
            depth: if count == 0 { 0 } else { self.depth },
            text_units: mul(self.text_units)?,
            operations: mul(self.operations)?,
        })
    }
}

/// Shared hard ceilings for a complete generated case. Check credit before
/// constructing a payload; an unknown generic payload must declare its costs.
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    limits: InputLimits,
    used: BudgetCharge,
}

impl Budget {
    #[must_use]
    pub const fn new(limits: InputLimits) -> Self {
        Self {
            limits,
            used: BudgetCharge {
                bytes: 0,
                nodes: 0,
                edges: 0,
                depth: 0,
                text_units: 0,
                operations: 0,
            },
        }
    }
    #[must_use]
    pub const fn used(&self) -> BudgetCharge {
        self.used
    }
    #[must_use]
    pub const fn limits(&self) -> InputLimits {
        self.limits
    }

    fn credited(&self, charge: BudgetCharge) -> Result<Self> {
        let used = self.used.combine(charge)?;
        let usage = InputUsage::new(
            used.bytes,
            used.nodes,
            used.depth,
            used.text_units,
            used.operations,
        )?;
        self.limits.check(usage)?;
        // A rooted forest has at most nodes - 1 edges. InputLimits' node ceiling
        // therefore supplies its hard edge ceiling without a second policy.
        if used.edges > self.limits.nodes().saturating_sub(1) {
            return Err(resource("edge budget exhausted"));
        }
        Ok(Self {
            limits: self.limits,
            used,
        })
    }

    /// Reserves credit before consumer-owned allocation. Credit is conservative:
    /// failed consumer construction may retain it rather than silently retry.
    pub fn credit(&mut self, charge: BudgetCharge) -> Result<()> {
        *self = self.credited(charge)?;
        Ok(())
    }
}

/// A finite deterministic entropy cursor. Exhaustion is an explicit error;
/// choice modulo a range is intentionally not a uniform-distribution promise.
pub struct Entropy<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Entropy<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    pub fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| resource("entropy length overflow"))?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| resource("insufficient entropy"))?;
        self.position = end;
        Ok(bytes)
    }

    pub fn range(&mut self, minimum: u64, maximum: u64) -> Result<u64> {
        if minimum > maximum {
            return Err(invalid("reversed numeric range"));
        }
        if minimum == maximum {
            return Ok(minimum);
        }
        let value = u64::from_le_bytes(self.take(8)?.try_into().expect("eight bytes"));
        Ok(
            match maximum
                .checked_sub(minimum)
                .and_then(|span| span.checked_add(1))
            {
                Some(width) => minimum + value % width,
                None => value,
            },
        )
    }

    pub fn choice<'b, T>(&mut self, choices: &'b [T]) -> Result<&'b T> {
        if choices.is_empty() {
            return Err(invalid("empty choice"));
        }
        let index = self.range(0, (choices.len() - 1) as u64)? as usize;
        Ok(&choices[index])
    }

    /// Zero weights are disabled choices; all-zero and overflow totals reject.
    pub fn weighted_choice<'b, T>(&mut self, choices: &'b [(u64, T)]) -> Result<&'b T> {
        let total = choices.iter().try_fold(0_u64, |sum, (weight, _)| {
            sum.checked_add(*weight)
                .ok_or_else(|| invalid("weight overflow"))
        })?;
        if total == 0 {
            return Err(invalid("no enabled weighted choice"));
        }
        let mut selected = self.range(0, total - 1)?;
        for (weight, value) in choices {
            if selected < *weight {
                return Ok(value);
            }
            selected -= weight;
        }
        unreachable!("validated nonzero total covers selection")
    }

    pub fn bytes(&mut self, count: usize, budget: &mut Budget) -> Result<Vec<u8>> {
        let next = budget.credited(BudgetCharge {
            bytes: count as u64,
            ..BudgetCharge::default()
        })?;
        let source = self.take(count)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| resource("byte allocation failed"))?;
        result.extend_from_slice(source);
        *budget = next;
        Ok(result)
    }

    /// Strict UTF-8: malformed bytes reject instead of lossy replacement.
    pub fn utf8(&mut self, bytes: usize, budget: &mut Budget) -> Result<String> {
        let source = self.take(bytes)?;
        let value = std::str::from_utf8(source).map_err(|_| invalid("invalid UTF-8 recipe"))?;
        let next = budget.credited(BudgetCharge {
            bytes: bytes as u64,
            text_units: value.chars().count() as u64,
            ..BudgetCharge::default()
        })?;
        let mut result = String::new();
        result
            .try_reserve_exact(bytes)
            .map_err(|_| resource("string allocation failed"))?;
        result.push_str(value);
        *budget = next;
        Ok(result)
    }

    /// Lossless UTF-16 code units, including unpaired surrogates. No String
    /// conversion or scalar-value validity is imposed by this fixture primitive.
    pub fn utf16(&mut self, units: usize, budget: &mut Budget) -> Result<Vec<u16>> {
        let count = units
            .checked_mul(2)
            .ok_or_else(|| resource("UTF-16 length overflow"))?;
        let next = budget.credited(BudgetCharge {
            bytes: count as u64,
            text_units: units as u64,
            ..BudgetCharge::default()
        })?;
        let source = self.take(count)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(units)
            .map_err(|_| resource("UTF-16 allocation failed"))?;
        result.extend(
            source
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
        );
        *budget = next;
        Ok(result)
    }
}

/// Supplementary deterministic recipe basis, not a persisted concrete case.
/// Materialize and encode through the owner's CaseTarget before evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeterministicConfig {
    pub seed: u64,
    pub entropy_bytes: usize,
}

impl DeterministicConfig {
    pub fn entropy(self, limits: InputLimits) -> Result<Vec<u8>> {
        let mut budget = Budget::new(limits);
        budget.credit(BudgetCharge {
            bytes: self.entropy_bytes as u64,
            ..BudgetCharge::default()
        })?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(self.entropy_bytes)
            .map_err(|_| resource("entropy allocation failed"))?;
        let mut state = self.seed;
        for _ in 0..self.entropy_bytes {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = state;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            output.push((value ^ (value >> 31)) as u8);
        }
        Ok(output)
    }
}

/// Generic vector recipe: the slots are charged automatically; item_charge
/// must additionally cover every allocation/resource inside each generic T.
/// Consumers also own nonrecursive traversal/destruction of their payloads.
pub fn bounded_vector<T>(
    count: usize,
    item_charge: BudgetCharge,
    budget: &mut Budget,
    mut construct: impl FnMut() -> Result<T>,
) -> Result<Vec<T>> {
    let slot = BudgetCharge {
        bytes: size_of::<T>() as u64,
        ..BudgetCharge::default()
    };
    let next = budget.credited(item_charge.combine(slot)?.repeated(count as u64)?)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| resource("vector allocation failed"))?;
    for _ in 0..count {
        result.push(construct()?);
    }
    *budget = next;
    Ok(result)
}

/// Fixture-local arena index, unrelated to production/live identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FixtureId(usize);

impl FixtureId {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    parent: Option<FixtureId>,
    first_child: Option<FixtureId>,
    last_child: Option<FixtureId>,
    next_sibling: Option<FixtureId>,
    depth: u64,
}

/// Checked rooted ordered forest, with flat storage and iterative drop/traversal.
/// A parent must already exist; order is append order. Payloads and additional
/// relations are owner-defined and must be separately admitted. Bytes account
/// for materialized node slots, not allocator metadata or strategy scratch space.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrderedForest {
    nodes: Vec<Node>,
    first_root: Option<FixtureId>,
    last_root: Option<FixtureId>,
}

impl OrderedForest {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            first_root: None,
            last_root: None,
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    #[must_use]
    pub const fn node_bytes() -> u64 {
        size_of::<Node>() as u64
    }

    pub fn push(&mut self, parent: Option<FixtureId>, budget: &mut Budget) -> Result<FixtureId> {
        let depth = match parent {
            Some(id) => self
                .nodes
                .get(id.0)
                .ok_or_else(|| invalid("parent does not exist"))?
                .depth
                .checked_add(1)
                .ok_or_else(|| resource("depth overflow"))?,
            None => 1,
        };
        let next = budget.credited(BudgetCharge {
            bytes: Self::node_bytes(),
            nodes: 1,
            edges: u64::from(parent.is_some()),
            depth,
            ..BudgetCharge::default()
        })?;
        self.nodes
            .try_reserve_exact(1)
            .map_err(|_| resource("forest allocation failed"))?;
        let id = FixtureId(self.nodes.len());
        let previous = parent.map_or(self.last_root, |parent| self.nodes[parent.0].last_child);
        self.nodes.push(Node {
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
            depth,
        });
        if let Some(previous) = previous {
            self.nodes[previous.0].next_sibling = Some(id);
        }
        if let Some(parent) = parent {
            let node = &mut self.nodes[parent.0];
            node.first_child.get_or_insert(id);
            node.last_child = Some(id);
        } else {
            self.first_root.get_or_insert(id);
            self.last_root = Some(id);
        }
        *budget = next;
        Ok(id)
    }

    /// Checks every reference without rebasing IDs or repairing forward/self
    /// references. A None parent explicitly declares a disconnected root.
    pub fn from_parents(parents: &[Option<FixtureId>], budget: &mut Budget) -> Result<Self> {
        let mut next = *budget;
        let mut forest = Self::new();
        for parent in parents {
            forest.push(*parent, &mut next)?;
        }
        *budget = next;
        Ok(forest)
    }

    pub fn parent(&self, id: FixtureId) -> Result<Option<FixtureId>> {
        Ok(self.node(id)?.parent)
    }
    pub fn depth(&self, id: FixtureId) -> Result<u64> {
        Ok(self.node(id)?.depth)
    }
    fn node(&self, id: FixtureId) -> Result<&Node> {
        self.nodes
            .get(id.0)
            .ok_or_else(|| invalid("fixture ID does not exist"))
    }

    pub fn children(&self, id: FixtureId) -> Result<impl Iterator<Item = FixtureId> + '_> {
        let first = self.node(id)?.first_child;
        Ok(std::iter::successors(first, |id| {
            self.nodes[id.0].next_sibling
        }))
    }

    pub fn roots(&self) -> impl Iterator<Item = FixtureId> + '_ {
        std::iter::successors(self.first_root, |id| self.nodes[id.0].next_sibling)
    }

    pub fn preorder(&self) -> ForestWalk<'_> {
        ForestWalk {
            forest: self,
            next: self.first_root,
            boundary: None,
        }
    }

    /// Reduction traversal hook. Owners decide whether removing/simplifying
    /// this subtree preserves payloads, cross-references and semantic validity.
    pub fn subtree(&self, id: FixtureId) -> Result<ForestWalk<'_>> {
        self.node(id)?;
        Ok(ForestWalk {
            forest: self,
            next: Some(id),
            boundary: Some(id),
        })
    }
}

/// Stackless preorder walk over a checked flat forest.
pub struct ForestWalk<'a> {
    forest: &'a OrderedForest,
    next: Option<FixtureId>,
    boundary: Option<FixtureId>,
}

impl Iterator for ForestWalk<'_> {
    type Item = FixtureId;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.forest.nodes[current.0].first_child;
        if self.next.is_none() {
            let mut ancestor = current;
            loop {
                if Some(ancestor) == self.boundary {
                    break;
                }
                let node = &self.forest.nodes[ancestor.0];
                if let Some(sibling) = node.next_sibling {
                    self.next = Some(sibling);
                    break;
                }
                match node.parent {
                    Some(parent) => ancestor = parent,
                    None => break,
                }
            }
        }
        Some(current)
    }
}

/// Constructs topology from choices, not product requests. Each choice indexes
/// the root option followed by already-existing parents with remaining depth.
/// Invalid product requests belong in SequenceLane::ExpectedRejection instead.
pub fn forest_from_choices(choices: &[u64], budget: &mut Budget) -> Result<OrderedForest> {
    let mut next = *budget;
    let mut forest = OrderedForest::new();
    for choice in choices {
        let available = forest
            .nodes
            .iter()
            .filter(|node| node.depth < next.limits.depth())
            .count();
        let selected = choice % (available as u64 + 1);
        let parent = if selected == 0 {
            None
        } else {
            forest
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.depth < next.limits.depth())
                .nth(selected as usize - 1)
                .map(|(index, _)| FixtureId(index))
        };
        forest.push(parent, &mut next)?;
    }
    *budget = next;
    Ok(forest)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceLane {
    Valid,
    ExpectedRejection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceStep<O> {
    operation: O,
    lane: SequenceLane,
}

impl<O> SequenceStep<O> {
    #[must_use]
    pub const fn operation(&self) -> &O {
        &self.operation
    }
    #[must_use]
    pub const fn lane(&self) -> SequenceLane {
        self.lane
    }
}

/// Finite operation recipe. The consumer owns state/preconditions and all
/// dependent references. Invalid requests are retained exactly, never repaired.
#[derive(Debug)]
pub struct SequenceBuilder<O> {
    steps: Vec<SequenceStep<O>>,
    rejected_preconditions: u64,
}

impl<O> Default for SequenceBuilder<O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O> SequenceBuilder<O> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            steps: Vec::new(),
            rejected_preconditions: 0,
        }
    }
    #[must_use]
    pub fn steps(&self) -> &[SequenceStep<O>] {
        &self.steps
    }
    #[must_use]
    pub const fn rejected_preconditions(&self) -> u64 {
        self.rejected_preconditions
    }
    #[must_use]
    pub fn into_steps(self) -> Vec<SequenceStep<O>> {
        self.steps
    }

    /// Returns false for one explicit precondition discard, with no retry.
    /// Credit covers slots automatically and unknown payloads via payload_charge.
    /// Construction happens only after the hard credit check. Evaluate a valid
    /// operation against consumer state only after this method returns true.
    pub fn append_valid(
        &mut self,
        budget: &mut Budget,
        payload_charge: BudgetCharge,
        construct: impl FnOnce() -> Result<O>,
        precondition: impl FnOnce(&[SequenceStep<O>], &O) -> bool,
    ) -> Result<bool> {
        self.append(
            SequenceLane::Valid,
            budget,
            payload_charge,
            construct,
            precondition,
        )
    }

    /// Consumers evaluate rejection and atomicity against their own state. The
    /// exact operation is retained even when its references intentionally dangle.
    pub fn append_invalid(
        &mut self,
        budget: &mut Budget,
        payload_charge: BudgetCharge,
        construct: impl FnOnce() -> Result<O>,
    ) -> Result<()> {
        self.append(
            SequenceLane::ExpectedRejection,
            budget,
            payload_charge,
            construct,
            |_, _| true,
        )?;
        Ok(())
    }

    fn append(
        &mut self,
        lane: SequenceLane,
        budget: &mut Budget,
        payload_charge: BudgetCharge,
        construct: impl FnOnce() -> Result<O>,
        precondition: impl FnOnce(&[SequenceStep<O>], &O) -> bool,
    ) -> Result<bool> {
        let charge = payload_charge.combine(BudgetCharge {
            bytes: size_of::<SequenceStep<O>>() as u64,
            operations: 1,
            ..BudgetCharge::default()
        })?;
        let next = budget.credited(charge)?;
        let operation = construct()?;
        if !precondition(&self.steps, &operation) {
            self.rejected_preconditions = self
                .rejected_preconditions
                .checked_add(1)
                .ok_or_else(|| resource("precondition counter overflow"))?;
            return Ok(false);
        }
        self.steps
            .try_reserve_exact(1)
            .map_err(|_| resource("sequence allocation failed"))?;
        self.steps.push(SequenceStep { operation, lane });
        *budget = next;
        Ok(true)
    }
}

/// Common integer edge choices, including extrema and both sides of zero.
pub const INTEGER_EDGES: &[i64] = &[i64::MIN, -1, 0, 1, i64::MAX];
/// Exact floating bit patterns retain NaNs, signed zero, infinities and subnormals.
pub const FLOAT_EDGES: &[u64] = &[
    0,
    1_u64 << 63,
    1,
    (1_u64 << 63) | 1,
    0x7ff0_0000_0000_0000,
    0xfff0_0000_0000_0000,
    0x7ff8_0000_0000_0001,
    0x7fef_ffff_ffff_ffff,
];

#[cfg(feature = "property")]
pub mod property {
    //! Property strategies have hard size bounds before collection allocation.
    //! No prop_recursive desired-size or rejection filters provide admission.
    use super::*;
    use proptest::{prelude::*, strategy::BoxedStrategy};

    fn capacity(maximum: usize, unit_bytes: u64, unit_text: u64, limits: InputLimits) -> usize {
        let bytes = limits.bytes().checked_div(unit_bytes).unwrap_or(u64::MAX);
        let text = limits
            .text_units()
            .checked_div(unit_text)
            .unwrap_or(u64::MAX);
        maximum.min(usize::try_from(bytes.min(text)).unwrap_or(usize::MAX))
    }

    pub fn bytes(maximum: usize, limits: InputLimits) -> BoxedStrategy<Vec<u8>> {
        proptest::collection::vec(any::<u8>(), 0..=capacity(maximum, 1, 0, limits)).boxed()
    }
    pub fn utf8(maximum_chars: usize, limits: InputLimits) -> BoxedStrategy<String> {
        proptest::collection::vec(any::<char>(), 0..=capacity(maximum_chars, 4, 1, limits))
            .prop_map(|chars| chars.into_iter().collect())
            .boxed()
    }
    pub fn utf16(maximum: usize, limits: InputLimits) -> BoxedStrategy<Vec<u16>> {
        proptest::collection::vec(any::<u16>(), 0..=capacity(maximum, 2, 1, limits)).boxed()
    }
    /// Consumer credit describes resources inside each generated T. Collection
    /// slots are added here; the element strategy must honor that declared cost.
    pub fn vector<T: std::fmt::Debug + 'static>(
        element: impl Strategy<Value = T> + 'static,
        maximum: usize,
        item_charge: BudgetCharge,
        limits: InputLimits,
    ) -> Result<BoxedStrategy<Vec<T>>> {
        let item = item_charge.combine(BudgetCharge {
            bytes: size_of::<T>() as u64,
            ..BudgetCharge::default()
        })?;
        let mut cap = maximum;
        for (cost, ceiling) in [
            (item.bytes, limits.bytes()),
            (item.nodes, limits.nodes()),
            (item.edges, limits.nodes().saturating_sub(1)),
            (item.text_units, limits.text_units()),
            (item.operations, limits.operations()),
        ] {
            cap = cap.min(
                usize::try_from(ceiling.checked_div(cost).unwrap_or(u64::MAX))
                    .unwrap_or(usize::MAX),
            );
        }
        if item.depth > limits.depth() {
            cap = 0;
        }
        if cap != 0 {
            // Reject incoherent structural credit rather than permit a strategy
            // that cannot be admitted through the common boundary.
            Budget::new(limits).credited(item)?;
        }
        Ok(proptest::collection::vec(element, 0..=cap).boxed())
    }
    pub fn forest(maximum: usize, limits: InputLimits) -> BoxedStrategy<OrderedForest> {
        let maximum = capacity(maximum, OrderedForest::node_bytes(), 0, limits)
            .min(usize::try_from(limits.nodes()).unwrap_or(usize::MAX));
        proptest::collection::vec(any::<u64>(), 0..=maximum)
            .prop_map(move |choices| {
                forest_from_choices(&choices, &mut Budget::new(limits))
                    .expect("strategy construction uses admitted node/byte/depth ceilings")
            })
            .boxed()
    }
    pub fn integer_edges() -> BoxedStrategy<i64> {
        prop_oneof![3 => proptest::sample::select(INTEGER_EDGES.to_vec()), 1 => any::<i64>()]
            .boxed()
    }
    pub fn float_bits() -> BoxedStrategy<u64> {
        prop_oneof![3 => proptest::sample::select(FLOAT_EDGES.to_vec()), 1 => any::<u64>()].boxed()
    }
}

#[cfg(feature = "coverage")]
pub mod coverage {
    //! Arbitrary adapters consume a finite Unstructured and allocate only after
    //! hard credit checks. Their distribution differs from property strategies.
    use super::*;
    use arbitrary::{Result as ArbitraryResult, Unstructured};

    pub fn bytes(
        input: &mut Unstructured<'_>,
        maximum: usize,
        limits: InputLimits,
    ) -> ArbitraryResult<Vec<u8>> {
        let cap = maximum.min(usize::try_from(limits.bytes()).unwrap_or(usize::MAX));
        let count = input.int_in_range(0..=cap)?;
        let source = input.bytes(count)?;
        Entropy::new(source)
            .bytes(count, &mut Budget::new(limits))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
    pub fn utf16(
        input: &mut Unstructured<'_>,
        maximum: usize,
        limits: InputLimits,
    ) -> ArbitraryResult<Vec<u16>> {
        let cap = maximum.min(
            usize::try_from((limits.bytes() / 2).min(limits.text_units())).unwrap_or(usize::MAX),
        );
        let count = input.int_in_range(0..=cap)?;
        let bytes = count
            .checked_mul(2)
            .ok_or(arbitrary::Error::IncorrectFormat)?;
        let source = input.bytes(bytes)?;
        Entropy::new(source)
            .utf16(count, &mut Budget::new(limits))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
    /// Strict UTF-8 from bounded bytes. Invalid scalar encoding is an explicit
    /// generation discard, with no retry or replacement character conversion.
    pub fn utf8(
        input: &mut Unstructured<'_>,
        maximum: usize,
        limits: InputLimits,
    ) -> ArbitraryResult<String> {
        let cap = maximum
            .min(usize::try_from(limits.bytes().min(limits.text_units())).unwrap_or(usize::MAX));
        let count = input.int_in_range(0..=cap)?;
        let source = input.bytes(count)?;
        Entropy::new(source)
            .utf8(count, &mut Budget::new(limits))
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
    pub fn forest(
        input: &mut Unstructured<'_>,
        maximum: usize,
        limits: InputLimits,
    ) -> ArbitraryResult<OrderedForest> {
        let cap = maximum.min(
            usize::try_from((limits.bytes() / OrderedForest::node_bytes()).min(limits.nodes()))
                .unwrap_or(usize::MAX),
        );
        let count = input.int_in_range(0..=cap)?;
        let mut forest = OrderedForest::new();
        let mut budget = Budget::new(limits);
        for _ in 0..count {
            let available = forest
                .nodes
                .iter()
                .filter(|node| node.depth < limits.depth())
                .count();
            let selected = input.int_in_range(0..=available)?;
            let parent = if selected == 0 {
                None
            } else {
                forest
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, node)| node.depth < limits.depth())
                    .nth(selected - 1)
                    .map(|(index, _)| FixtureId(index))
            };
            forest
                .push(parent, &mut budget)
                .map_err(|_| arbitrary::Error::IncorrectFormat)?;
        }
        Ok(forest)
    }
}
