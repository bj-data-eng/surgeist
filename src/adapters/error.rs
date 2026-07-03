use std::{error, fmt};

pub type AdapterResult<T> = Result<T, AdapterError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterError {
    kind: AdapterErrorKind,
}

impl AdapterError {
    #[must_use]
    pub const fn new(kind: AdapterErrorKind) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(&self) -> &AdapterErrorKind {
        &self.kind
    }

    #[must_use]
    pub const fn boundary(&self) -> AdapterBoundary {
        self.kind.boundary()
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.boundary(), self.kind)
    }
}

impl error::Error for AdapterError {}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterBoundary {
    CssToStyle,
    RetainedToStyle,
    StyleToText,
    StyleToLayout,
    StrictTreeToLayout,
    UnsupportedAdapterInput,
}

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterErrorKind {
    CssValueUnsupported {
        property: String,
        reason: String,
    },
    CssStyleValidation {
        property: String,
        reason: String,
    },
    RetainedTraversal {
        node: String,
        reason: String,
    },
    RetainedChangeMapping {
        reason: String,
    },
    StyleTextValue {
        property: String,
        reason: String,
    },
    StyleLayoutValue {
        property: String,
        reason: String,
    },
    StrictTreeInput {
        node: String,
        reason: String,
    },
    UnsupportedAdapterInput {
        boundary: AdapterBoundary,
        reason: String,
    },
}

impl AdapterErrorKind {
    #[must_use]
    pub const fn boundary(&self) -> AdapterBoundary {
        match self {
            Self::CssValueUnsupported { .. } | Self::CssStyleValidation { .. } => {
                AdapterBoundary::CssToStyle
            }
            Self::RetainedTraversal { .. } | Self::RetainedChangeMapping { .. } => {
                AdapterBoundary::RetainedToStyle
            }
            Self::StyleTextValue { .. } => AdapterBoundary::StyleToText,
            Self::StyleLayoutValue { .. } => AdapterBoundary::StyleToLayout,
            Self::StrictTreeInput { .. } => AdapterBoundary::StrictTreeToLayout,
            Self::UnsupportedAdapterInput { boundary, .. } => *boundary,
        }
    }
}
