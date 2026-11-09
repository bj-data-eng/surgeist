//! Native producer observations shared by one entry operation and its trials.
use crate::text_layout::scope_alignment::{
    ScopeAlignmentFacts, ScopeAlignmentResources, ScopeLease, allocation_buffer, allocation_header,
};
use crate::{LayoutScalar, LeafMeasureInputOf, Size};
use surgeist_text::{PreparedText, ScopeAlignmentCensusInput, ScopeAlignmentProduct};

pub(super) struct Funding<S: LayoutScalar> {
    pub(super) resources: ScopeAlignmentResources<S>,
    pub(super) prepared: PreparedText,
}
impl<S: LayoutScalar> Funding<S> {
    fn visit(&self) -> surgeist_text::Result<()> {
        if let Some(limits) = self
            .prepared
            .captured_owner()
            .and_then(|owner| owner.domain().scope_alignment_limits())
        {
            self.resources.visit(limits, 1)?;
        }
        Ok(())
    }
    pub(super) fn header<T>(&self) -> surgeist_text::Result<ScopeLease> {
        let operation = self
            .resources
            .begin(
                ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(&self.prepared)),
                0,
            )?
            .ok_or_else(|| {
                crate::text_layout::scope_alignment::refusal(
                    surgeist_text::ScopeAlignmentError::MissingCapture,
                )
            })?;
        allocation_header::<T>(&self.resources, &operation)
    }
    fn buffer<T>(&self, count: usize) -> surgeist_text::Result<(Vec<T>, ScopeLease)> {
        let operation = self
            .resources
            .begin(
                ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(&self.prepared)),
                0,
            )?
            .ok_or_else(|| {
                crate::text_layout::scope_alignment::refusal(
                    surgeist_text::ScopeAlignmentError::MissingCapture,
                )
            })?;
        allocation_buffer(&self.resources, &operation, count)
    }
}

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
}
impl<Node: Copy + Eq, S: LayoutScalar> SourceMemo<Node, S> {
    pub(super) fn new(owner: Option<ScopeLease>) -> Self {
        Self {
            samples: Vec::new(),
            admitted: 0,
            backing: None,
            _owner: owner,
        }
    }

    pub(super) fn contains(
        &self,
        node: Node,
        funding: Option<&Funding<S>>,
    ) -> surgeist_text::Result<bool> {
        for sample in &self.samples {
            if let Some(funding) = funding {
                funding.visit()?;
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
        funding: Option<&Funding<S>>,
    ) -> surgeist_text::Result<Option<Size<S>>> {
        for sample in &self.samples {
            if let Some(funding) = funding {
                funding.visit()?;
            }
            if sample.node == node && sample.request == request {
                return Ok(Some(sample.result));
            }
        }
        Ok(None)
    }

    /// Fund the actual new backing before invoking the provider. Discarded
    /// copies release their live lease, never materialization/work history.
    pub(super) fn reserve(&mut self, funding: Option<&Funding<S>>) -> surgeist_text::Result<()> {
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
        let (mut values, backing) = match funding {
            Some(funding) => {
                let (values, lease) = funding.buffer(count)?;
                (values, Some(lease))
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
                (values, None)
            }
        };
        for value in &self.samples {
            if let Some(funding) = funding {
                funding.visit()?;
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
        funding: Option<&Funding<S>>,
    ) -> surgeist_text::Result<()> {
        if let Some(funding) = funding {
            funding.visit()?;
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
