use crate::component::CssComponentValues;
use crate::numeric::{CssFiniteNumber, CssLengthUnit};
use crate::source::CssSourcePosition;

/// One checked function or parenthesis component, including EOF-implied closure.
///
/// This lexical wrapper does not select a supports or media grammar branch.
/// Equality compares exact token spelling, children and source-text/span origins;
/// it is not semantic equivalence or source snapshot identity.
///
/// ```compile_fail
/// use surgeist_css::component::CssComponentValue;
/// use surgeist_css::query::CssGeneralEnclosed;
/// let component = CssComponentValue::try_token("x").unwrap();
/// let forged = CssGeneralEnclosed { component: Box::new(component) };
/// ```
///
/// ```compile_fail
/// use surgeist_css::component::CssComponentValue;
/// use surgeist_css::query::CssGeneralEnclosed;
/// use surgeist_css::source::CssParsedOrigin;
/// let parsed = CssParsedOrigin { source: todo!(), span: todo!() };
/// let component = CssComponentValue { data: todo!(), parsed: Some(parsed) };
/// let forged = CssGeneralEnclosed::try_from_component(component);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssGeneralEnclosed {
    component: Box<crate::component::CssComponentValue>,
}

impl CssGeneralEnclosed {
    /// Checks the outer lexical enclosure without changing any supplied origin.
    pub fn try_from_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, crate::query::CssConditionConstructionError> {
        let valid = match component.view() {
            crate::component::CssComponentValueRef::Function(_) => true,
            crate::component::CssComponentValueRef::Block(block) => {
                block.kind() == crate::component::CssBlockKind::Parenthesis
            }
            _ => false,
        };
        if !valid {
            return Err(
                crate::query::when::ConditionConstructionKind::WrongOuterComponent {
                    origin: component.origin().clone(),
                }
                .into(),
            );
        }
        Ok(Self {
            component: Box::new(component),
        })
    }

    /// Constructs programmatic function delimiters while preserving child origins.
    pub fn try_function(
        name: impl Into<String>,
        values: CssComponentValues,
    ) -> Result<Self, crate::query::CssConditionConstructionError> {
        Self::try_from_component(
            crate::component::CssComponentValue::try_function(name, values)
                .map_err(crate::query::CssConditionConstructionError::from)?,
        )
    }

    /// Constructs programmatic parentheses while preserving child origins.
    pub fn try_parenthesized(
        values: CssComponentValues,
    ) -> Result<Self, crate::query::CssConditionConstructionError> {
        Self::try_from_component(
            crate::component::CssComponentValue::try_block(
                crate::component::CssBlockKind::Parenthesis,
                values,
            )
            .map_err(crate::query::CssConditionConstructionError::from)?,
        )
    }

    /// Returns the immutable underlying component.
    #[must_use]
    pub const fn component(&self) -> &crate::component::CssComponentValue {
        &self.component
    }

    /// Returns the opener's real origin.
    #[must_use]
    pub const fn origin(&self) -> &crate::component::CssValueOrigin {
        self.component.origin()
    }

    /// Returns coordinates only when the opener was parsed from source.
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        match self.origin() {
            crate::component::CssValueOrigin::Parsed(origin) => Some(origin.span().start()),
            _ => None,
        }
    }

    /// Returns the complete original component slice, including unclosed EOF input.
    /// Programmatic enclosures have no single authored slice, even with parsed children.
    #[must_use]
    pub fn authored(&self) -> Option<&str> {
        let origin = self.component.parsed_origin()?;
        Some(
            &origin.source().as_str()[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
        )
    }

    /// Serializes tokens and explicit or EOF-implied delimiters with their origins.
    ///
    /// `limits` bound the single-component list and its output, including
    /// inserted syntax.
    ///
    /// # Errors
    ///
    /// Returns [`crate::component::CssComponentValueError`] when a limit is exhausted.
    pub fn serialize_tokens(
        &self,
        limits: crate::component::CssComponentValueLimits,
    ) -> Result<crate::component::CssSerializedValue, crate::component::CssComponentValueError>
    {
        CssComponentValues::try_new(vec![self.component().clone()], limits)?
            .serialize_tokens(limits)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssQueryComparison {
    LessThan,
    LessThanOrEqual,
    Equal,
    GreaterThanOrEqual,
    GreaterThan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRangeFeature<T> {
    comparison: Option<CssQueryComparison>,
    value: T,
}

impl<T> CssRangeFeature<T> {
    #[must_use]
    pub const fn comparison(&self) -> Option<CssQueryComparison> {
        self.comparison
    }

    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssQueryLength {
    value: CssFiniteNumber,
    unit: Option<CssLengthUnit>,
}

impl CssQueryLength {
    #[must_use]
    pub fn try_new(value: f32, unit: CssLengthUnit) -> Option<Self> {
        if value >= 0.0 {
            CssFiniteNumber::try_new(value).map(|value| Self {
                value,
                unit: Some(unit),
            })
        } else {
            None
        }
    }

    /// Constructs the exact unitless-zero spelling accepted by the CSS length grammar.
    #[must_use]
    pub fn unitless_zero() -> Self {
        Self {
            value: CssFiniteNumber::try_new(0.0).expect("zero is finite"),
            unit: None,
        }
    }

    #[must_use]
    pub const fn value(self) -> CssFiniteNumber {
        self.value
    }

    /// Returns the exact authored unit, or `None` for a valid unitless zero.
    #[must_use]
    pub const fn unit(self) -> Option<CssLengthUnit> {
        self.unit
    }
}
