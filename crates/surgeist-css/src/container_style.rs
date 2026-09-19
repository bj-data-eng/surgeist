//! Checked authored style queries. Computing values and query truth is external.
use crate::component_values::CssCanonicalBuilder;
use crate::supports::SupportsLexical;
use crate::*;

/// One admitted style-query node, retaining its original lexical region.
/// Construct through [`CssContainerCondition::try_from_components`] or stylesheet parsing.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerStyleQuery {
    kind: Box<CssContainerStyleQueryKind>,
    lexical: SupportsLexical,
}

/// Inspectable authored logic. A kind alone cannot construct a checked query.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerStyleQueryKind {
    Feature(CssContainerStyleFeature),
    Parenthesized(Box<CssContainerStyleQuery>),
    Not(Box<CssContainerStyleQuery>),
    And(CssContainerStyleQueryList),
    Or(CssContainerStyleQueryList),
    GeneralEnclosed(CssContainerGeneralEnclosed),
}

/// At least two operands admitted by one homogeneous logical production.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerStyleQueryList {
    queries: Vec<CssContainerStyleQuery>,
}
impl CssContainerStyleQueryList {
    pub(crate) fn new(queries: Vec<CssContainerStyleQuery>) -> Self {
        debug_assert!(queries.len() >= 2);
        Self { queries }
    }
    pub fn queries(&self) -> &[CssContainerStyleQuery] {
        &self.queries
    }
}

/// A property grammar identity or a case-sensitive custom-property identity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContainerStyleFeatureName {
    Property(CssPropertyGrammar),
    Custom(CssCustomPropertyName),
}

/// Authored features, without property-value computation or range evaluation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerStyleFeature {
    Boolean(CssContainerStyleFeatureName),
    Plain {
        name: CssContainerStyleFeatureName,
        value: CssContainerStyleValue,
    },
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
    CustomProperty(&'a CssCustomPropertyName),
    Value(&'a CssContainerStyleValue),
}
impl CssContainerStyleRangeOperand {
    pub(crate) fn new(
        value: CssContainerStyleValue,
        reference: Option<CssCustomPropertyName>,
    ) -> Self {
        Self { value, reference }
    }
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
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssContainerStyleRangeRef<'a> {
    Binary {
        left: &'a CssContainerStyleRangeOperand,
        comparison: CssQueryComparison,
        right: &'a CssContainerStyleRangeOperand,
    },
    Ascending {
        left: &'a CssContainerStyleRangeOperand,
        left_inclusive: bool,
        middle: &'a CssContainerStyleRangeOperand,
        right_inclusive: bool,
        right: &'a CssContainerStyleRangeOperand,
    },
    Descending {
        left: &'a CssContainerStyleRangeOperand,
        left_inclusive: bool,
        middle: &'a CssContainerStyleRangeOperand,
        right_inclusive: bool,
        right: &'a CssContainerStyleRangeOperand,
    },
}
impl CssContainerStyleRange {
    pub(crate) fn new(kind: StyleRangeKind) -> Self {
        Self { kind }
    }
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

impl CssContainerStyleQuery {
    pub(crate) fn new(kind: CssContainerStyleQueryKind, lexical: SupportsLexical) -> Self {
        Self {
            kind: Box::new(kind),
            lexical,
        }
    }
    pub fn kind(&self) -> &CssContainerStyleQueryKind {
        &self.kind
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
            pub fn components(&self) -> &[CssComponentValue] {
                self.lexical.items()
            }
            pub fn origin(&self) -> &CssValueOrigin {
                self.components()
                    .iter()
                    .find(|value| !crate::supports::trivia(value))
                    .or_else(|| self.components().first())
                    .expect("admitted nonempty lexical region")
                    .origin()
            }
            pub fn position(&self) -> Option<CssSourcePosition> {
                crate::media::parsed_position(self.origin())
            }
            pub fn serialize(&self) -> Result<CssSerializedValue, CssComponentValueError> {
                self.serialize_with_limit(usize::MAX)
            }
            pub fn serialize_with_limit(
                &self,
                max_css_bytes: usize,
            ) -> Result<CssSerializedValue, CssComponentValueError> {
                let mut out = CssCanonicalBuilder::new(max_css_bytes);
                out.push_components(self.components())?;
                out.finish()
            }
        }
    };
}
lexical_access!(CssContainerStyleQuery);
lexical_access!(CssContainerStyleValue);
