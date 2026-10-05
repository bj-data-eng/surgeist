//! Bounded specified keyword rendering for fragmentation controls.

use crate::specified_serialization::serialize_keyword_sequence;
use crate::{
    CssBreakBetween, CssBreakInside, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

impl CssBreakBetween {
    /// Serializes one canonical specified break keyword.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one input and projection node plus the canonical CSS bytes.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        serialize_keyword_sequence(
            match self {
                Self::Auto => "auto",
                Self::Avoid => "avoid",
                Self::AvoidPage => "avoid-page",
                Self::Page => "page",
                Self::Left => "left",
                Self::Right => "right",
                Self::Recto => "recto",
                Self::Verso => "verso",
                Self::AvoidColumn => "avoid-column",
                Self::Column => "column",
                Self::AvoidRegion => "avoid-region",
                Self::Region => "region",
            },
            limits,
        )
    }
}

impl CssBreakInside {
    /// Serializes one canonical specified break-inside keyword.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one input and projection node plus the canonical CSS bytes.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        serialize_keyword_sequence(
            match self {
                Self::Auto => "auto",
                Self::Avoid => "avoid",
                Self::AvoidPage => "avoid-page",
                Self::AvoidColumn => "avoid-column",
                Self::AvoidRegion => "avoid-region",
            },
            limits,
        )
    }
}

impl crate::CssBoxDecorationBreak {
    /// Emits the canonical authored fragmentation-decoration keyword.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges one input node, one projection node, and the emitted bytes.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        serialize_keyword_sequence(
            match self {
                Self::Slice => "slice",
                Self::Clone => "clone",
            },
            limits,
        )
    }
}
