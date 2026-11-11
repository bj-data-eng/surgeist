use std::fmt;

use surgeist_layout::{ItemOrder, SourceIndex};

/// A slot in the immutable box-fact slice passed to one planning operation.
/// This is not a DOM identity. Equal external identities can generate distinct boxes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoxId(usize);
impl BoxId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Ordering policy for a box's immediate children.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildOrder {
    Source,
    Flex,
    Grid,
}

/// A box's resolved participation in its parent. Items must be in flow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Participation {
    Ordinary,
    FlexItem(ItemOrder),
    GridItem(ItemOrder),
}

/// CSS positioning, supplied explicitly rather than inferred from Layout geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}
impl Position {
    pub(crate) const fn out_of_flow(self) -> bool {
        matches!(self, Self::Absolute | Self::Fixed)
    }
}

/// Resolved stack level. Authored calculations and values outside i64 require
/// upstream resolution or a capability diagnostic; no clamping is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZIndex {
    Auto,
    Integer(i64),
}

/// An already-applicable effect's context-creation fact, not its execution.
/// Top-layer and three-dimensional ordering are outside this algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextEffect {
    None,
    EstablishesContext,
}

/// Immutable, caller-resolved rendering-box facts. Cross-box relations are
/// validated by [`crate::plan_stacking`]. Anonymous/generated boxes have slots too.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoxFacts {
    pub(crate) parent: Option<BoxId>,
    pub(crate) source_index: SourceIndex,
    pub(crate) child_order: ChildOrder,
    pub(crate) participation: Participation,
    pub(crate) position: Position,
    pub(crate) z_index: ZIndex,
    pub(crate) effects: ContextEffect,
}
impl BoxFacts {
    pub const fn new(
        parent: Option<BoxId>,
        source_index: SourceIndex,
        child_order: ChildOrder,
        participation: Participation,
        position: Position,
        z_index: ZIndex,
        effects: ContextEffect,
    ) -> Self {
        Self {
            parent,
            source_index,
            child_order,
            participation,
            position,
            z_index,
            effects,
        }
    }
    pub const fn parent(&self) -> Option<BoxId> {
        self.parent
    }
    pub const fn source_index(&self) -> SourceIndex {
        self.source_index
    }
    pub const fn child_order(&self) -> ChildOrder {
        self.child_order
    }
    pub const fn participation(&self) -> Participation {
        self.participation
    }
    pub const fn position(&self) -> Position {
        self.position
    }
    pub const fn z_index(&self) -> ZIndex {
        self.z_index
    }
    pub const fn effects(&self) -> ContextEffect {
        self.effects
    }

    pub(crate) fn classification(self) -> (bool, i64) {
        let applicable =
            self.position != Position::Static || self.participation != Participation::Ordinary;
        let level = if applicable {
            match self.z_index {
                ZIndex::Auto => 0,
                ZIndex::Integer(n) => n,
            }
        } else {
            0
        };
        let creates = (applicable && matches!(self.z_index, ZIndex::Integer(_)))
            || matches!(self.position, Position::Fixed | Position::Sticky)
            || self.effects == ContextEffect::EstablishesContext;
        (creates, level)
    }
    pub(crate) fn effective_order(self) -> i32 {
        match self.participation {
            Participation::FlexItem(order) | Participation::GridItem(order) => order.get(),
            Participation::Ordinary => 0,
        }
    }
}

/// A complete typed refusal. A failed operation publishes no partial plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PaintError {
    Input(crate::PaintInputError),
    Gradient(crate::GradientGeometryError),
    GradientStops(crate::GradientStopsError),
    RadialGradient(crate::RadialGradientError),
    ZeroHeightGradient(crate::ZeroHeightGradientError),
    BorderImage(crate::BorderImageGeometryError),
    MissingRoot,
    MultipleRoots {
        first: BoxId,
        other: BoxId,
    },
    MissingParent {
        child: BoxId,
        parent: BoxId,
    },
    SelfParent {
        box_id: BoxId,
    },
    Cycle {
        box_id: BoxId,
    },
    DuplicateSourceIndex {
        parent: BoxId,
        source_index: SourceIndex,
    },
    IncompatibleParticipation {
        box_id: BoxId,
    },
    CapacityOverflow,
    Allocation {
        resource: crate::ResourceKind,
    },
    Budget(crate::BudgetRefusal),
    Accounting(crate::AccountingFailure),
}
impl fmt::Display for PaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "paint planning failed: {self:?}")
    }
}
impl std::error::Error for PaintError {}

pub use crate::algorithm::{ContextView, StackingPlan};
