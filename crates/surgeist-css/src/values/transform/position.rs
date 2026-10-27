//! Checked authored coordinate families and consumer-specific position restrictions.
use crate::numeric::{CssLength, CssNumeric};
use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
use crate::values::position::{CssHorizontalPosition, CssPhysicalPosition, CssVerticalPosition};

/// A physical planar transform origin plus an optional authored pure-length Z.
///
/// # Specified serialization
///
/// Emits canonical specified syntax from retained typed operands. Percentages and calculations
/// retain their owning specified policy without resolving reference boxes or matrices. Numeric
/// output retains six-place formatting. General rotate3d axes share the Rotate owner's
/// approximation and representability policy.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransformOrigin {
    position: CssPhysicalPosition,
    z: Option<CssNumeric<CssLength>>,
}
impl CssTransformOrigin {
    /// Borrows the checked planar owner without exposing another public getter.
    pub(crate) fn append_planar_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), crate::serialization::CssSerializationError> {
        self.position.append_to_rule_writer(writer)
    }

    /// Rejects planar edge offsets while retaining the optional checked Z length.
    #[must_use]
    pub fn try_new(
        position: CssPhysicalPosition,
        z: Option<CssNumeric<CssLength>>,
    ) -> Option<Self> {
        if position.horizontal().is_edge_offset() || position.vertical().is_edge_offset() {
            return None;
        }
        Some(Self { position, z })
    }
    /// Returns the authored horizontal planar axis.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        self.position.horizontal()
    }
    /// Returns the authored vertical planar axis.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        self.position.vertical()
    }
    /// Returns the optional checked authored Z length.
    #[must_use]
    pub const fn z(&self) -> Option<&CssNumeric<CssLength>> {
        self.z.as_ref()
    }
}
