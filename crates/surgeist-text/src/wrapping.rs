//! Analysis-bound native ancestor wrapping facts over actual processed neighbors.

use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{AnalyzedSource, Range, Wrap, engine::AnalyzedData};

/// Readback provenance for an actual influencing occurrence. Numeric observations
/// cannot construct a relation capability or assert native ancestor permission.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WrappingNeighbor {
    Text { origin: usize, local: Range },
    Atomic { occurrence: usize },
}

/// An exact pair issued by one immutable analysis context.
#[derive(Clone)]
pub struct AnalyzedWrappingRelation {
    analyzed: Arc<AnalyzedData>,
    index: usize,
}
impl PartialEq for AnalyzedWrappingRelation {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.analyzed, &other.analyzed)
    }
}
impl Eq for AnalyzedWrappingRelation {}
impl Hash for AnalyzedWrappingRelation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.analyzed).hash(state);
        self.index.hash(state);
    }
}
impl std::fmt::Debug for AnalyzedWrappingRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyzedWrappingRelation")
            .field("relation", &self.index)
            .finish_non_exhaustive()
    }
}

/// Actual producer-supplied NCA permission for one issued pair.
/// Text validates the pair's custody; the producer owns actual ancestry facts.
#[derive(Clone, Debug)]
pub struct WrappingPolicy {
    pub relation: AnalyzedWrappingRelation,
    pub ancestor_wrap: Wrap,
}

pub struct AnalyzedWrappingRelationView<'a> {
    analyzed: &'a Arc<AnalyzedData>,
    index: usize,
}
impl AnalyzedWrappingRelationView<'_> {
    #[must_use]
    pub fn handle(&self) -> AnalyzedWrappingRelation {
        AnalyzedWrappingRelation {
            analyzed: Arc::clone(self.analyzed),
            index: self.index,
        }
    }
    #[must_use]
    pub fn before(&self) -> WrappingNeighbor {
        self.analyzed.logical.relations[self.index]
            .neighbors
            .expect("issued actual neighbor pair")
            .0
    }
    #[must_use]
    pub fn after(&self) -> WrappingNeighbor {
        self.analyzed.logical.relations[self.index]
            .neighbors
            .expect("issued actual neighbor pair")
            .1
    }
    #[must_use]
    pub fn ancestor_wrap(&self) -> Option<Wrap> {
        self.analyzed.wrapping[self.index]
    }
}

impl AnalyzedSource {
    /// Borrows actual processed influencing pairs. No font, DOM traversal or
    /// repeated Unicode/whitespace processing is needed by the capture producer.
    pub fn wrapping_relations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnalyzedWrappingRelationView<'_>> {
        self.inner
            .logical
            .relation_indices
            .iter()
            .map(|&index| AnalyzedWrappingRelationView {
                analyzed: &self.inner,
                index,
            })
    }

    /// Creates a new immutable context, sharing the already executed source and
    /// Unicode core. Supplied policies replace those pairs in the prior context;
    /// unbound pairs remain explicit and fail only when an operation needs them.
    /// A handle issued by another context, even one sharing this core, is foreign.
    pub fn with_wrapping_policies(&self, policies: Vec<WrappingPolicy>) -> crate::Result<Self> {
        use crate::{ProtocolError, protocol::protocol_error};
        if self.inner.paragraph.base_direction.is_none() {
            return Err(protocol_error(ProtocolError::UnsupportedWrappingAnalysis));
        }
        let mut wrapping = self.inner.wrapping.clone();
        let mut supplied = std::collections::BTreeMap::new();
        for policy in policies {
            let index = policy.relation.index;
            if !Arc::ptr_eq(&self.inner, &policy.relation.analyzed) {
                return Err(protocol_error(ProtocolError::ForeignWrappingRelation {
                    relation: index,
                }));
            }
            if let Some(previous) = supplied.insert(index, policy.ancestor_wrap) {
                return Err(protocol_error(if previous == policy.ancestor_wrap {
                    ProtocolError::DuplicateWrappingPolicy { relation: index }
                } else {
                    ProtocolError::ConflictingWrappingPolicy { relation: index }
                }));
            }
            wrapping[index] = Some(policy.ancestor_wrap);
        }
        Ok(Self {
            inner: Arc::new(AnalyzedData {
                core: Arc::clone(&self.inner.core),
                wrapping,
            }),
        })
    }
}
