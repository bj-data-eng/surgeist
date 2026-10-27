//! Checked authored style queries. Computing values and query truth is external.
use crate::component::values::serialize::CssCanonicalBuilder;
use crate::component::{
    CssComponentValue, CssComponentValueError, CssSerializedValue, CssValueOrigin,
};
use crate::declaration::CssCustomPropertyName;
use crate::property::CssPropertyGrammar;
use crate::query::CssQueryComparison;
use crate::query::container::CssContainerGeneralEnclosed;
use crate::query::supports::construction::SupportsLexical;
use crate::source::CssSourcePosition;

/// A leaf of a style query: a style feature, or any other enclosure.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssStyleQueryLeaf {
    /// A style feature test, such as `--theme: dark` or `color`.
    Feature(CssContainerStyleFeature),
    /// A `<general-enclosed>` production that is not a recognized style feature.
    GeneralEnclosed(CssContainerGeneralEnclosed),
}

/// A property grammar identity or a case-sensitive custom-property identity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainerStyleFeatureName {
    /// A recognized CSS property, identified by its grammar entry.
    Property(CssPropertyGrammar),
    /// A custom property name, which is case-sensitive.
    Custom(CssCustomPropertyName),
}

/// Authored features, without property-value computation or range evaluation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerStyleFeature {
    /// A feature name alone, such as `style(--flag)`, with no value or comparison.
    Boolean(CssContainerStyleFeatureName),
    /// A feature name followed by `:` and a value, such as `--theme: dark`.
    Plain {
        /// The property or custom property being tested.
        name: CssContainerStyleFeatureName,
        /// The authored value after the colon.
        value: CssContainerStyleValue,
    },
    /// A range-syntax comparison, such as `--a < --b` or `1 <= --n < 3`.
    Range(CssContainerStyleRange),
}

/// An admitted style feature value. A bare `--name` here remains literal syntax.
/// Whitespace tokens count as values; comments alone do not.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerStyleValue {
    lexical: SupportsLexical,
}

/// An admitted range operand, retaining its lexical value and contextual identity.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerStyleRangeOperand {
    value: CssContainerStyleValue,
    reference: Option<CssCustomPropertyName>,
}
/// Only a complete bare custom-property name is an implicit range reference.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssContainerStyleRangeOperandRef<'a> {
    /// A complete bare custom-property name used as an implicit reference to that property's value.
    CustomProperty(&'a CssCustomPropertyName),
    /// Any other operand, kept as authored lexical syntax.
    Value(&'a CssContainerStyleValue),
}
impl CssContainerStyleRangeOperand {
    pub(crate) fn new(
        value: CssContainerStyleValue,
        reference: Option<CssCustomPropertyName>,
    ) -> Self {
        Self { value, reference }
    }
    /// Returns the referenced custom property when the operand is a complete bare custom-property name, and otherwise the authored value.
    pub fn view(&self) -> CssContainerStyleRangeOperandRef<'_> {
        match &self.reference {
            Some(name) => CssContainerStyleRangeOperandRef::CustomProperty(name),
            None => CssContainerStyleRangeOperandRef::Value(&self.value),
        }
    }
    /// Returns authored components for either contextual operand kind.
    pub fn value(&self) -> &CssContainerStyleValue {
        &self.value
    }
}

/// Two or three authored operands. No implicit central property is invented.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerStyleRange {
    kind: StyleRangeKind,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StyleRangeKind {
    Binary {
        left: CssContainerStyleRangeOperand,
        comparison: CssQueryComparison,
        right: CssContainerStyleRangeOperand,
    },
    Ascending {
        left: CssContainerStyleRangeOperand,
        left_inclusive: bool,
        middle: CssContainerStyleRangeOperand,
        right_inclusive: bool,
        right: CssContainerStyleRangeOperand,
    },
    Descending {
        left: CssContainerStyleRangeOperand,
        left_inclusive: bool,
        middle: CssContainerStyleRangeOperand,
        right_inclusive: bool,
        right: CssContainerStyleRangeOperand,
    },
}
/// Borrowed view of a [`CssContainerStyleRange`], retaining the authored operand order and the comparison operators as written.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssContainerStyleRangeRef<'a> {
    /// Two operands joined by one comparison, such as `--a < --b`.
    Binary {
        /// The operand written before the comparison.
        left: &'a CssContainerStyleRangeOperand,
        /// The comparison operator as written between the operands.
        comparison: CssQueryComparison,
        /// The operand written after the comparison.
        right: &'a CssContainerStyleRangeOperand,
    },
    /// Three operands joined by two `<` or `<=` comparisons, such as `1 <= --n < 3`.
    Ascending {
        /// The operand written first.
        left: &'a CssContainerStyleRangeOperand,
        /// Whether the first comparison is `<=` rather than `<`.
        left_inclusive: bool,
        /// The operand written between the two comparisons.
        middle: &'a CssContainerStyleRangeOperand,
        /// Whether the second comparison is `<=` rather than `<`.
        right_inclusive: bool,
        /// The operand written last.
        right: &'a CssContainerStyleRangeOperand,
    },
    /// Three operands joined by two `>` or `>=` comparisons, such as `3 > --n >= 1`.
    Descending {
        /// The operand written first.
        left: &'a CssContainerStyleRangeOperand,
        /// Whether the first comparison is `>=` rather than `>`.
        left_inclusive: bool,
        /// The operand written between the two comparisons.
        middle: &'a CssContainerStyleRangeOperand,
        /// Whether the second comparison is `>=` rather than `>`.
        right_inclusive: bool,
        /// The operand written last.
        right: &'a CssContainerStyleRangeOperand,
    },
}
impl CssContainerStyleRange {
    pub(crate) fn new(kind: StyleRangeKind) -> Self {
        Self { kind }
    }
    /// Returns the range as a borrowed view that preserves operand order and comparison forms.
    pub fn view(&self) -> CssContainerStyleRangeRef<'_> {
        match &self.kind {
            StyleRangeKind::Binary {
                left,
                comparison,
                right,
            } => CssContainerStyleRangeRef::Binary {
                left,
                comparison: *comparison,
                right,
            },
            StyleRangeKind::Ascending {
                left,
                left_inclusive,
                middle,
                right_inclusive,
                right,
            } => CssContainerStyleRangeRef::Ascending {
                left,
                left_inclusive: *left_inclusive,
                middle,
                right_inclusive: *right_inclusive,
                right,
            },
            StyleRangeKind::Descending {
                left,
                left_inclusive,
                middle,
                right_inclusive,
                right,
            } => CssContainerStyleRangeRef::Descending {
                left,
                left_inclusive: *left_inclusive,
                middle,
                right_inclusive: *right_inclusive,
                right,
            },
        }
    }
}

impl CssContainerStyleValue {
    pub(crate) fn new(lexical: SupportsLexical) -> Self {
        Self { lexical }
    }
}

macro_rules! lexical_access {
    ($ty:ty) => {
        impl $ty {
            /// Returns the authored component values, including any trivia.
            pub fn components(&self) -> &[CssComponentValue] {
                self.lexical.items()
            }
            /// Returns the origin of the first non-trivia component, or of the first component when all are trivia.
            pub fn origin(&self) -> &CssValueOrigin {
                self.components()
                    .iter()
                    .find(|value| !crate::query::supports::construction::trivia(value))
                    .or_else(|| self.components().first())
                    .expect("admitted nonempty lexical region")
                    .origin()
            }
            /// Returns the source position of the value, or `None` when its origin was not parsed from source.
            pub fn position(&self) -> Option<CssSourcePosition> {
                crate::query::media::construction::parsed_position(self.origin())
            }
            /// `limits.max_css_bytes()` caps the output, including inserted syntax; the other
            /// component limits do not apply.
            pub fn serialize_tokens(
                &self,
                limits: crate::component::CssComponentValueLimits,
            ) -> Result<CssSerializedValue, CssComponentValueError> {
                let max_css_bytes = limits.max_css_bytes();
                let mut out = CssCanonicalBuilder::new(max_css_bytes);
                out.push_components(self.components())?;
                out.finish()
            }
        }
    };
}
lexical_access!(CssContainerStyleValue);
