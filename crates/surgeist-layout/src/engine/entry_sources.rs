//! Native producer observations shared by one entry operation and its trials.
use crate::text_layout::scope_alignment::{EntryFunding, ScopeLease};
use crate::{
    CacheKeyContext, ComputeInputOf, ComputeOutputOf, LayoutScalar, LayoutTree, LeafMeasureInputOf,
    Size,
};
use std::collections::TryReserveError;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
struct Sample<Node, S: LayoutScalar> {
    node: Node,
    request: LeafMeasureInputOf<S>,
    result: Size<S>,
}

pub(super) struct SourceMemo<Node, S: LayoutScalar> {
    samples: Vec<Sample<Node, S>>,
    admitted: usize,
    backing: Option<ScopeLease>,
    _owner: Option<ScopeLease>,
    float_eligible: Option<Vec<bool>>,
    floats: Vec<FloatSlot<Node, S>>,
}
impl<Node: Copy + Eq, S: LayoutScalar> SourceMemo<Node, S> {
    pub(super) fn new(owner: Option<ScopeLease>) -> Self {
        Self {
            samples: Vec::new(),
            admitted: 0,
            backing: None,
            _owner: owner,
            float_eligible: None,
            floats: Vec::new(),
        }
    }

    pub(super) fn contains(
        &self,
        node: Node,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<bool> {
        for sample in &self.samples {
            if let Some(funding) = funding {
                funding.visit(1)?;
            }
            if sample.node == node {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn find(
        &self,
        node: Node,
        request: LeafMeasureInputOf<S>,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<Option<Size<S>>> {
        for sample in &self.samples {
            if let Some(funding) = funding {
                funding.visit(1)?;
            }
            if sample.node == node && sample.request == request {
                return Ok(Some(sample.result));
            }
        }
        Ok(None)
    }

    /// Fund the actual new backing before invoking the provider. Discarded
    /// copies release their live lease, never materialization/work history.
    pub(super) fn reserve(
        &mut self,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<()> {
        if self.samples.len() < self.admitted {
            return Ok(());
        }
        let count = self.samples.len().checked_add(1).ok_or_else(|| {
            crate::text_layout::scope_alignment::refusal(
                surgeist_text::ScopeAlignmentError::Overflow {
                    kind: surgeist_text::ScopeAlignmentLimitKind::Records,
                },
            )
        })?;
        let backing;
        let mut values = match funding {
            Some(funding) => {
                let (values, lease) = funding.buffer(count)?;
                backing = Some(lease);
                values
            }
            None => {
                let mut values = Vec::new();
                values.try_reserve_exact(count).map_err(|_| {
                    crate::text_layout::scope_alignment::refusal(
                        surgeist_text::ScopeAlignmentError::Allocation {
                            kind: surgeist_text::ScopeAlignmentLimitKind::PayloadBytes,
                        },
                    )
                })?;
                backing = None;
                values
            }
        };
        for value in &self.samples {
            if let Some(funding) = funding {
                funding.visit(1)?;
            }
            values.push(*value);
        }
        self.samples = values;
        self.admitted = count;
        self.backing = backing;
        Ok(())
    }

    pub(super) fn record(
        &mut self,
        node: Node,
        request: LeafMeasureInputOf<S>,
        result: Size<S>,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<()> {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        assert!(
            self.samples.len() < self.admitted,
            "provider storage was funded before execution"
        );
        self.samples.push(Sample {
            node,
            request,
            result,
        });
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct FloatKey<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    context: CacheKeyContext,
}

enum FloatSlot<Node, S: LayoutScalar> {
    Vacant,
    Pending(FloatKey<Node, S>),
    Complete(FloatKey<Node, S>, ComputeOutputOf<S>),
}

pub(super) enum FloatObservation<Node, S: LayoutScalar> {
    Hit(ComputeOutputOf<S>),
    Miss(FloatReservation<Node, S>),
}

/// A distinct slot is reserved before its producer can issue nested misses.
/// An abandoned producer removes only its own pending slot; it never publishes
/// an output, and a later retry cannot mistake it for a completion.
pub(super) struct FloatReservation<Node, S: LayoutScalar> {
    sources: Arc<Mutex<SourceMemo<Node, S>>>,
    slot: Option<usize>,
}

impl<Node, S: LayoutScalar> FloatReservation<Node, S> {
    pub(super) fn complete(mut self, output: ComputeOutputOf<S>) {
        let index = self.slot.take().expect("unconsumed source reservation");
        let mut sources = self
            .sources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slot = &mut sources.floats[index];
        let FloatSlot::Pending(key) = std::mem::replace(slot, FloatSlot::Vacant) else {
            unreachable!("only its pending producer owns a source slot");
        };
        *slot = FloatSlot::Complete(key, output);
    }
}

impl<Node, S: LayoutScalar> Drop for FloatReservation<Node, S> {
    fn drop(&mut self) {
        if let Some(index) = self.slot.take() {
            let mut sources = self
                .sources
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            sources.floats[index] = FloatSlot::Vacant;
        }
    }
}

impl<Node: Copy + Eq, S: LayoutScalar> SourceMemo<Node, S> {
    pub(super) fn float_observation<Tree: LayoutTree<Node = Node, Scalar = S>>(
        sources: &Arc<Mutex<Self>>,
        tree: &Tree,
        graph: &super::validation::ValidationGraph<Node>,
        node: Node,
        input: ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Result<Option<FloatObservation<Node, S>>, TryReserveError> {
        let mut memo = sources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if memo._owner.is_some() {
            return Ok(None);
        }
        if memo.float_eligible.is_none() {
            let mut eligible = Vec::new();
            eligible.try_reserve_exact(graph.nodes.len())?;
            eligible.resize(graph.nodes.len(), false);
            for &index in &graph.finish {
                if let crate::LayoutInputViewOf::Box(style) = tree.layout_input(graph.nodes[index])
                {
                    eligible[index] = style.float_exclusion != crate::FloatExclusion::Shape
                        && graph.children[index].iter().all(|&child| eligible[child]);
                }
            }
            memo.float_eligible = Some(eligible);
        }
        let Some(index) = graph.index(node) else {
            return Ok(None);
        };
        if !memo
            .float_eligible
            .as_ref()
            .expect("initialized eligibility")[index]
        {
            return Ok(None);
        }
        let key = FloatKey {
            node,
            input,
            context,
        };
        if let Some(output) = memo.floats.iter().find_map(|slot| match slot {
            FloatSlot::Complete(stored, output)
                if stored.node == node
                    && stored.context == context
                    && stored.input == input
                    && exact_size(stored.input.known(), input.known())
                    && exact_size(stored.input.parent(), input.parent()) =>
            {
                Some(*output)
            }
            _ => None,
        }) {
            return Ok(Some(FloatObservation::Hit(output)));
        }
        let slot = if let Some(index) = memo
            .floats
            .iter()
            .position(|slot| matches!(slot, FloatSlot::Vacant))
        {
            index
        } else {
            memo.floats.try_reserve(1)?;
            let index = memo.floats.len();
            memo.floats.push(FloatSlot::Vacant);
            index
        };
        memo.floats[slot] = FloatSlot::Pending(key);
        Ok(Some(FloatObservation::Miss(FloatReservation {
            sources: sources.clone(),
            slot: Some(slot),
        })))
    }
}

fn exact_size<S: LayoutScalar>(left: Size<Option<S>>, right: Size<Option<S>>) -> bool {
    left.zip_map(right, |left, right| {
        left.map(|value| value.to_f64().to_bits()) == right.map(|value| value.to_f64().to_bits())
    }) == Size::splat(true)
}

#[cfg(test)]
#[path = "entry_sources_tests.rs"]
mod tests;
