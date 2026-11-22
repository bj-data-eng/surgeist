//! Symbolic font-palette sampling over caller-supplied computed operands.
//!
//! Follows [Fonts 4 §9.1](https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-palette-prop).
//!
//! Style retains the meaning of each [`PaletteSource`]: a computed keyword,
//! identifier or existing palette mix. Reuse its identity for known-equivalent
//! operands. Animation returns Oklab mix operations without interpreting font
//! metadata, rules, current color or individual palette entries. Style resolves
//! those entries and any context-dependent equivalence before consumer lowering.
//!
//! Plans contain only flat postorder nodes. Children precede their parent and
//! the final node is the result. Consumers can process them in a single forward
//! pass, preserving each nested mix's color semantics. No recursive traversal,
//! recursive ownership or fixed nesting limit is involved.
//!
//! ```
//! use surgeist_animation::interpolation::InterpolationProgress;
//! use surgeist_animation::palette::{PaletteNode, PaletteSource, PaletteValue};
//! let light = PaletteSource::new(); // The caller retains the computed value.
//! let dark = PaletteSource::new();
//! let first = PaletteValue::source(light.clone());
//! let second = PaletteValue::source(dark);
//! let middle = first.interpolate(&second, InterpolationProgress::new(0.5)?)?;
//! assert!(matches!(middle.nodes().last(), Some(PaletteNode::MixOklab { .. })));
//! assert_eq!(first.as_source(), Some(&light));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use core::fmt;
use core::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::sync::Arc;

use crate::interpolation::InterpolationProgress;

/// An immutable identity for one caller-owned computed palette operand.
///
/// Clones retain identity; construction creates a distinct source. The caller
/// owns the source-to-value mapping, supplies the same source for equivalent
/// computed operands and retains that mapping while sampled plans are needed.
#[derive(Debug, Clone)]
pub struct PaletteSource {
    identity: Arc<()>,
}

impl PaletteSource {
    pub fn new() -> Self {
        Self {
            identity: Arc::new(()),
        }
    }
}
impl Default for PaletteSource {
    fn default() -> Self {
        Self::new()
    }
}
impl PartialEq for PaletteSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
}
impl Eq for PaletteSource {}
impl Hash for PaletteSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::ptr::hash(Arc::as_ptr(&self.identity), state);
    }
}

/// One immutable operation in a validated postorder palette plan.
///
/// `MixOklab` interpolates the two palette operands per entry in Oklab, with
/// `progress` the second operand's strictly interior fraction. Style performs
/// the CSS Color operations; Animation does not flatten nested color mixes.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteNode {
    Source(PaletteSource),
    MixOklab {
        first: usize,
        second: usize,
        progress: InterpolationProgress,
    },
}

/// A palette operation's representability failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteErrorKind {
    Capacity,
}

/// A palette plan's storage or index space could not be represented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteError {
    kind: PaletteErrorKind,
}
impl PaletteError {
    fn capacity() -> Self {
        Self {
            kind: PaletteErrorKind::Capacity,
        }
    }
    pub const fn kind(&self) -> PaletteErrorKind {
        self.kind
    }
}
impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("palette plan storage or index space could not be represented")
    }
}
impl std::error::Error for PaletteError {}

/// A computed palette operand or immutable symbolic Oklab mix plan.
///
/// Construction and sampling preserve a nonempty, acyclic postorder plan.
/// Plan nodes are observable but cannot be used to forge a palette value.
///
/// ```compile_fail
/// use std::sync::Arc;
/// use surgeist_animation::palette::PaletteValue;
/// let value = PaletteValue { nodes: Arc::from([]) };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteValue {
    nodes: Arc<[PaletteNode]>,
}
impl PaletteValue {
    pub fn source(source: PaletteSource) -> Self {
        Self {
            nodes: Arc::from([PaletteNode::Source(source)]),
        }
    }
    /// Returns the entire plan in child-before-parent order, with its result last.
    pub fn nodes(&self) -> &[PaletteNode] {
        &self.nodes
    }
    pub fn as_source(&self) -> Option<&PaletteSource> {
        match self.nodes.as_ref() {
            [PaletteNode::Source(source)] => Some(source),
            _ => None,
        }
    }
    /// Samples palettes with exact endpoints and an Oklab mix strictly between them.
    ///
    /// Progress outside the unit interval selects its nearest endpoint. Equal
    /// supplied operands retain their value at every progress. Errors return no
    /// partial result and leave both operands unchanged.
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, PaletteError> {
        if progress.value() <= 0.0 || self == other {
            return Ok(self.clone());
        }
        if progress.value() >= 1.0 {
            return Ok(other.clone());
        }
        let capacity = plan_capacity(self.nodes.len(), other.nodes.len())?;
        let mut plan = PlanBuilder::new(capacity)?;
        let first = plan.import(self)?;
        let second = plan.import(other)?;
        if first == second {
            return Ok(self.clone());
        }
        plan.intern(PaletteNode::MixOklab {
            first,
            second,
            progress,
        });
        Ok(Self {
            nodes: plan.nodes.into(),
        })
    }
    /// Selects the effect palette for nonadditive palette composition.
    pub fn add(&self, effect: &Self) -> Self {
        effect.clone()
    }
}

// A compact key interns operations after their operand indices are remapped.
// Interior fractions are finite and positive, so their bits give exact equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NodeKey {
    Source(PaletteSource),
    MixOklab {
        first: usize,
        second: usize,
        progress_bits: u64,
    },
}
impl From<&PaletteNode> for NodeKey {
    fn from(node: &PaletteNode) -> Self {
        match node {
            PaletteNode::Source(source) => Self::Source(source.clone()),
            PaletteNode::MixOklab {
                first,
                second,
                progress,
            } => Self::MixOklab {
                first: *first,
                second: *second,
                progress_bits: progress.value().to_bits(),
            },
        }
    }
}
struct PlanBuilder {
    nodes: Vec<PaletteNode>,
    indices: HashMap<NodeKey, usize>,
}
impl PlanBuilder {
    fn new(capacity: usize) -> Result<Self, PaletteError> {
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(capacity)
            .map_err(|_| PaletteError::capacity())?;
        let mut indices = HashMap::new();
        indices
            .try_reserve(capacity)
            .map_err(|_| PaletteError::capacity())?;
        Ok(Self { nodes, indices })
    }
    fn intern(&mut self, node: PaletteNode) -> usize {
        let key = NodeKey::from(&node);
        if let Some(index) = self.indices.get(&key) {
            return *index;
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        self.indices.insert(key, index);
        index
    }
    fn import(&mut self, value: &PaletteValue) -> Result<usize, PaletteError> {
        let mut mapping = Vec::new();
        mapping
            .try_reserve_exact(value.nodes.len())
            .map_err(|_| PaletteError::capacity())?;
        for node in value.nodes.iter() {
            let mapped = match node {
                PaletteNode::Source(_) => node.clone(),
                PaletteNode::MixOklab {
                    first,
                    second,
                    progress,
                } => PaletteNode::MixOklab {
                    first: *mapping.get(*first).ok_or_else(PaletteError::capacity)?,
                    second: *mapping.get(*second).ok_or_else(PaletteError::capacity)?,
                    progress: *progress,
                },
            };
            mapping.push(self.intern(mapped));
        }
        mapping.last().copied().ok_or_else(PaletteError::capacity)
    }
}
fn plan_capacity(first: usize, second: usize) -> Result<usize, PaletteError> {
    first
        .checked_add(second)
        .and_then(|length| length.checked_add(1))
        .ok_or_else(PaletteError::capacity)
}

#[cfg(test)]
mod tests {
    use super::{PaletteErrorKind, PlanBuilder, plan_capacity};

    #[test]
    fn unrepresentable_plan_storage_returns_a_typed_capacity_error() {
        assert_eq!(
            plan_capacity(usize::MAX, 1).unwrap_err().kind(),
            PaletteErrorKind::Capacity
        );
        assert_eq!(
            plan_capacity(usize::MAX - 1, 1).unwrap_err().kind(),
            PaletteErrorKind::Capacity
        );
        assert_eq!(
            PlanBuilder::new(usize::MAX).err().unwrap().kind(),
            PaletteErrorKind::Capacity
        );
    }
}
