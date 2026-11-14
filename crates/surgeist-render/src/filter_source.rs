//! Shared immutable native filter source declarations; the public GaussianSource
//! remains the actual iterative Scene owner used by restrictive native receivers.
use crate::{Color, FilterDomains, FilterEdgeMode, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum FilterSourceDescriptor {
    Finite {
        domains: FilterDomains,
        edge: FilterEdgeMode,
    },
    Constant {
        color: Color,
        filter: Rect,
        result: Rect,
    },
    Periodic {
        period: Rect,
        filter: Rect,
        result: Rect,
    },
}
impl FilterSourceDescriptor {
    pub(crate) const fn filter(self) -> Rect {
        match self {
            Self::Finite { domains, .. } => domains.filter(),
            Self::Constant { filter, .. } | Self::Periodic { filter, .. } => filter,
        }
    }
    pub(crate) const fn result(self) -> Rect {
        match self {
            Self::Finite { domains, .. } => domains.result(),
            Self::Constant { result, .. } | Self::Periodic { result, .. } => result,
        }
    }
    /// Preserves the admitted source and resolves the internal result into F.
    pub(crate) const fn for_filter_output(self) -> Self {
        match self {
            Self::Finite { domains, edge } => Self::Finite {
                domains: domains.for_filter_output(),
                edge,
            },
            Self::Constant { color, filter, .. } => Self::Constant {
                color,
                filter,
                result: filter,
            },
            Self::Periodic { period, filter, .. } => Self::Periodic {
                period,
                filter,
                result: filter,
            },
        }
    }
    pub(crate) const fn is_infinite(self) -> bool {
        !matches!(self, Self::Finite { .. })
    }
}
