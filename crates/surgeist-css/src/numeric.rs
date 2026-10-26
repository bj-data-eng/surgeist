//! Exact, context-checked authored numeric expressions shared by CSS consumers.
use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssLengthUnit,
    CssNumericTokenKind, CssNumericTokenRef, CssRelativeColorChannel, CssSerializedValue,
    CssValueOrigin, CssValueTokenRef,
};
use std::{borrow::Cow, fmt};

mod color_expression;
pub(crate) mod color_operands;
pub use color_expression::{
    CssProfileColorCalculation, CssProfileColorExpression, CssProfileColorExpressionRef,
};

#[cfg(test)]
mod borrowed_admission_tests;
pub(crate) mod bounded_decimal;
pub(crate) mod bounded_expression;
mod controlled_check;
mod ordinary_expression;
pub(crate) mod projection;
pub(crate) mod units;
mod value;
pub(crate) use projection::{
    DeferredNumericProjection, NumericComparisonCapture, NumericProjectionOutcome,
    NumericProjectionScale, capture_specified, capture_specified_for_comparison,
    capture_specified_scaled, prepare_keyframe_matching, prepare_specified,
    project_calc_size_sum_into, project_specified_into,
};
pub use value::{
    CssAngle, CssAnglePercentage, CssAnyRange, CssCalculation, CssDimensionRoot, CssFlex,
    CssFrequency, CssFrequencyPercentage, CssHintedNumber, CssInteger, CssLength,
    CssLengthPercentage, CssNonNegative, CssNumber, CssNumeric, CssNumericLiteral, CssNumericRoot,
    CssPercentage, CssPositive, CssRange, CssResolution, CssTime,
};
pub(crate) use value::{RootKind, root_kind};

/// Component storage is acquired only for a completed calculation result.
#[derive(Debug)]
pub(crate) enum CalculationComponents<'a> {
    Owned(CssComponentValues),
    Borrowed(&'a [CssComponentValue]),
}
/// A failed attempt returns the original storage for a grammar-owned fallback.
#[derive(Debug)]
pub(crate) struct CalculationAdmissionFailure<'a> {
    pub(crate) error: CssNumericConstructionError,
    pub(crate) values: CalculationComponents<'a>,
}
impl<'a> CalculationComponents<'a> {
    pub(crate) fn single(
        value: Cow<'a, CssComponentValue>,
    ) -> std::result::Result<Self, CssComponentValueError> {
        match value {
            Cow::Owned(value) => {
                CssComponentValues::try_new(vec![value], Default::default()).map(Self::Owned)
            }
            Cow::Borrowed(value) => Ok(Self::Borrowed(std::slice::from_ref(value))),
        }
    }
    fn items(&self) -> &[CssComponentValue] {
        match self {
            Self::Owned(values) => values.items(),
            Self::Borrowed(items) => items,
        }
    }
    fn authored(&self) -> Result<crate::CssAuthoredDeclarationValue> {
        let mut builder = crate::component_values::CssCanonicalBuilder::new(
            CssComponentValueLimits::default().max_css_bytes(),
        );
        builder
            .push_components(self.items())
            .map_err(CssNumericConstructionError::component)?;
        let serialized = builder
            .finish()
            .map_err(CssNumericConstructionError::component)?;
        Ok(crate::CssAuthoredDeclarationValue::new(
            serialized.into_css(),
        ))
    }
    fn retain(self, limits: CssComponentValueLimits) -> Result<CssComponentValues> {
        match self {
            Self::Owned(values) => Ok(values),
            Self::Borrowed(items) => CssComponentValues::retain_admitted(items, limits)
                .map_err(CssNumericConstructionError::component),
        }
    }
}
impl From<CssComponentValues> for CalculationComponents<'_> {
    fn from(values: CssComponentValues) -> Self {
        Self::Owned(values)
    }
}
impl<'a> From<&'a CssComponentValue> for CalculationComponents<'a> {
    fn from(value: &'a CssComponentValue) -> Self {
        Self::Borrowed(std::slice::from_ref(value))
    }
}

/// Explicit provenance for an existing parser cursor; never ambient parser state.
pub(crate) enum NumericInputContext<'a> {
    Parsed(&'a crate::CssSourceSnapshot),
    /// Strict checked admission whose cursor owns actual component attribution.
    Checked,
    Components(&'a CssComponentValues, &'a CssSerializedValue),
    QuirkyLengths(&'a NumericInputContext<'a>),
}
impl<'a> NumericInputContext<'a> {
    pub(crate) fn ordinary(&self) -> &Self {
        match self {
            Self::QuirkyLengths(source) => source.ordinary(),
            _ => self,
        }
    }
    pub(crate) fn allows_quirky_lengths(&self) -> bool {
        matches!(self, Self::QuirkyLengths(_))
    }

    pub(crate) fn is_source_input(&self) -> bool {
        matches!(self.ordinary(), Self::Parsed(_))
    }

    /// Numerical grammar paths are relative to the actual component cursor,
    /// independently of whether those components share an authored snapshot.
    pub(crate) fn property_error_location<'i>(
        &self,
        error: &CssNumericConstructionError,
        fallback: crate::parser::property_input::Site<'i>,
        root: crate::parser::property_input::Site<'i>,
    ) -> crate::parser::property_input::Site<'i> {
        let truncated = matches!(
            error.kind(),
            CssNumericConstructionErrorKind::MalformedExpression
                | CssNumericConstructionErrorKind::Arity
        );
        let Some(path) = error.path() else {
            // A function cut short of its grammar ends after its contents;
            // the diagnostic rule then names the last authored token there.
            if truncated && let Some(end) = root.descendant_children_end(&[0]) {
                return end;
            }
            return fallback;
        };
        // Only a root function cut short is truncated; a path into it names
        // the component where the expression first stops being valid.
        if truncated
            && path.len() == 1
            && let Some(end) = root.descendant_children_end(path)
        {
            return end;
        }
        // An operator that ends its function lacks the operand the closing
        // token cannot supply.
        if truncated
            && let Some(end) = root
                .descendant(path, false)
                .and_then(|site| site.trailing_operator_end())
        {
            return end;
        }
        // Values 4 §10.9: a grammatical multi-term calculation whose type
        // does not check is responsible as a whole, through its enclosing
        // function; a single operand of the wrong type is itself responsible.
        let path = match error.kind() {
            CssNumericConstructionErrorKind::RootDomainMismatch
                if path.len() > 1
                    && root
                        .descendant(path, false)
                        .is_some_and(|site| site.begins_expression()) =>
            {
                &path[..path.len() - 1]
            }
            _ => path,
        };
        let closing = matches!(error.origin(), Some(CssValueOrigin::ImplicitClosure { .. }));
        root.descendant(path, closing).unwrap_or(fallback)
    }

    pub(crate) fn error_location(
        &self,
        error: &CssNumericConstructionError,
        fallback: cssparser::SourceLocation,
        root_offset: usize,
    ) -> cssparser::SourceLocation {
        match self.ordinary() {
            Self::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
            Self::Parsed(source) => {
                let Some(CssValueOrigin::Parsed(origin)) = error.origin() else {
                    return fallback;
                };
                if !source.same_snapshot(origin.source()) {
                    return fallback;
                }
                let position = origin.span().start();
                cssparser::SourceLocation {
                    line: position.line().value(),
                    column: position.column().value() + 1,
                }
            }
            Self::Checked => {
                unreachable!("checked property errors use their actual component cursor")
            }
            Self::Components(_, serialized) => {
                let Some(relative) = error.path.as_deref() else {
                    return fallback;
                };
                let Some(base) = serialized.component_path_at(root_offset) else {
                    return fallback;
                };
                let mut path = base.to_vec();
                path.extend_from_slice(&relative[1..]);
                let Some(offset) = serialized.component_offset_for_path(&path) else {
                    return fallback;
                };
                let mut input = cssparser::ParserInput::new(&serialized.as_css()[..offset]);
                let mut parser = cssparser::Parser::new(&mut input);
                while parser.next_including_whitespace_and_comments().is_ok() {}
                parser.current_source_location()
            }
        }
    }
    pub(crate) fn admit<'input>(
        &self,
        values: impl Into<CalculationComponents<'input>>,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression> {
        self.admit_with_limits(values, root, CssComponentValueLimits::default())
    }
    /// Moves unique input on success and returns it unchanged on failure.
    pub(crate) fn admit_or_return_input<'input>(
        &self,
        values: CalculationComponents<'input>,
        root: CalculationRoot,
    ) -> std::result::Result<CssCalculationExpression, Box<CalculationAdmissionFailure<'input>>>
    {
        let limits = CssComponentValueLimits::default();
        let mut expression = match prepare_calculation(
            values.items(),
            root,
            limits,
            self.admission_policy(),
            NumericAdmissionContext::Pure,
        ) {
            Ok(expression) => expression,
            Err(error) => return Err(Box::new(CalculationAdmissionFailure { error, values })),
        };
        expression.components = Some(match values {
            CalculationComponents::Owned(values) => values,
            CalculationComponents::Borrowed(items) => {
                match CssComponentValues::retain_admitted(items, limits) {
                    Ok(values) => values,
                    Err(error) => {
                        return Err(Box::new(CalculationAdmissionFailure {
                            error: CssNumericConstructionError::component(error),
                            values: CalculationComponents::Borrowed(items),
                        }));
                    }
                }
            }
        });
        Ok(expression)
    }
    fn admission_policy(&self) -> AdmissionPolicy {
        match self.ordinary() {
            Self::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
            Self::Parsed(_) => AdmissionPolicy::RecoveredSyntax,
            Self::Components(..) | Self::Checked => AdmissionPolicy::Strict,
        }
    }
    /// Check an actual read-only consumer without retaining an input graph.
    pub(crate) fn check_component(
        &self,
        component: &CssComponentValue,
        root: CalculationRoot,
    ) -> Result<()> {
        check_component_with_policy(component, root, self.admission_policy())
    }
    /// Read-only env() integer syntax over producer-checked immutable components.
    /// The called owner retains no expression or diagnostic path graph.
    pub(crate) fn check_component_controlled(
        &self,
        component: &CssComponentValue,
        root: CalculationRoot,
        control: &crate::utf16_value::Control<'_>,
    ) -> std::result::Result<Result<()>, ()> {
        assert!(
            root == CalculationRoot::Integer,
            "controlled env index domain"
        );
        controlled_check::check(component, self.admission_policy(), control)
    }
    /// Classifies checked syntax without retaining its component graph.
    pub(crate) fn component_result_type(
        &self,
        component: &CssComponentValue,
        root: CalculationRoot,
        limits: CssComponentValueLimits,
    ) -> Result<CssCalculationType> {
        prepare_calculation(
            std::slice::from_ref(component),
            root,
            limits,
            self.admission_policy(),
            NumericAdmissionContext::Pure,
        )
        .map(|expression| expression.result_type())
    }
    pub(crate) fn admit_with_limits<'input>(
        &self,
        values: impl Into<CalculationComponents<'input>>,
        root: CalculationRoot,
        limits: CssComponentValueLimits,
    ) -> Result<CssCalculationExpression> {
        construct_with_policy(values, root, limits, self.admission_policy())
    }
    pub(crate) fn admit_grid_track_math<'input>(
        &self,
        values: impl Into<CalculationComponents<'input>>,
    ) -> Result<GridTrackMath> {
        construct_grid_track_math(
            values,
            CssComponentValueLimits::default(),
            self.admission_policy(),
        )
    }
    pub(crate) fn origin_at(&self, offset: usize) -> Option<CssValueOrigin> {
        match self.ordinary() {
            Self::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
            Self::Components(_, serialized) => serialized.value_origin_at(offset).cloned(),
            Self::Checked => None,
            Self::Parsed(source) => {
                let (_, end, _) = crate::tokenization::next_source_token(source.as_str(), offset)?;
                crate::CssParsedOrigin::from_range(source, offset..end).map(CssValueOrigin::Parsed)
            }
        }
    }
    pub(crate) fn parsed(source: &'a crate::CssSourceSnapshot) -> Self {
        Self::Parsed(source)
    }
    pub(crate) fn components(
        values: &'a CssComponentValues,
        serialized: &'a CssSerializedValue,
    ) -> Self {
        Self::Components(values, serialized)
    }
    pub(crate) fn collect(
        &self,
        input: &mut cssparser::Parser<'_, '_>,
    ) -> Result<Cow<'a, CssComponentValue>> {
        input.skip_whitespace();
        match self.ordinary() {
            Self::QuirkyLengths(_) => unreachable!("ordinary numeric provenance"),
            Self::Parsed(source) => CssComponentValue::collect_from_parser(input, source)
                .map(Cow::Owned)
                .map_err(CssNumericConstructionError::component),
            Self::Checked => {
                unreachable!("checked property collection uses its actual component cursor")
            }
            Self::Components(values, serialized) => {
                input.skip_whitespace();
                let offset = input.position().byte_index();
                let path = serialized.component_path_at(offset).ok_or_else(|| {
                    CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::MalformedExpression,
                        None,
                    )
                })?;
                let mut items = values.items();
                let mut found = None;
                for index in path {
                    let c = &items[*index];
                    found = Some(c);
                    items = match c.view() {
                        CssComponentValueRef::Function(f) => f.values().items(),
                        CssComponentValueRef::Block(b) => b.values().items(),
                        _ => &[],
                    };
                }
                let component = found.expect("serialized component path is nonempty");
                let nested = matches!(
                    component.view(),
                    CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
                );
                input.next().map_err(|_| {
                    CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::MalformedExpression,
                        Some(component),
                    )
                })?;
                if nested {
                    input
                        .parse_nested_block(|p| {
                            while p.next_including_whitespace_and_comments().is_ok() {}
                            Ok::<_, cssparser::ParseError<'_, ()>>(())
                        })
                        .map_err(|_| {
                            CssNumericConstructionError::at(
                                CssNumericConstructionErrorKind::MalformedExpression,
                                Some(component),
                            )
                        })?;
                }
                Ok(Cow::Borrowed(component))
            }
        }
    }
}

/// A dimensional axis in the CSS numeric type algebra.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssNumericDimension {
    Length,
    Angle,
    Time,
    Frequency,
    Resolution,
    Flex,
    Percentage,
}
const DIMENSIONS: [CssNumericDimension; 7] = [
    CssNumericDimension::Length,
    CssNumericDimension::Angle,
    CssNumericDimension::Time,
    CssNumericDimension::Frequency,
    CssNumericDimension::Resolution,
    CssNumericDimension::Flex,
    CssNumericDimension::Percentage,
];
impl CssNumericDimension {
    const fn index(self) -> usize {
        self as usize
    }
}

/// Immutable dimensional exponents and the context's percentage hint.
///
/// Type metadata can only be obtained from a checked expression.
/// ```compile_fail
/// use surgeist_css::CssNumericType;
/// let forged = CssNumericType { powers: [0; 7], hint: None };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssNumericType {
    powers: [i32; 7],
    hint: Option<CssNumericDimension>,
}
impl CssNumericType {
    pub(crate) const NUMBER: Self = Self {
        powers: [0; 7],
        hint: None,
    };
    fn dimension(d: CssNumericDimension) -> Self {
        let mut t = Self::NUMBER;
        t.powers[d.index()] = 1;
        t
    }
    /// Returns a dimensional exponent, including intermediate negative powers.
    pub const fn exponent(self, d: CssNumericDimension) -> i32 {
        self.powers[d.index()]
    }
    /// Returns the unresolved percentage basis.
    pub const fn percent_hint(self) -> Option<CssNumericDimension> {
        self.hint
    }
    fn hinted(mut self, h: CssNumericDimension) -> Option<Self> {
        if self.hint.is_some_and(|old| old != h) {
            return None;
        }
        self.powers[h.index()] = self.powers[h.index()].checked_add(self.powers[6])?;
        self.powers[6] = 0;
        self.hint = Some(h);
        Some(self)
    }
    fn consistent(self, other: Self) -> Option<(Self, Self)> {
        match (self.hint, other.hint) {
            (Some(a), Some(b)) if a != b => None,
            (Some(h), _) => Some((self, other.hinted(h)?)),
            (_, Some(h)) => Some((self.hinted(h)?, other)),
            _ => Some((self, other)),
        }
    }
    fn add(self, other: Self) -> Option<Self> {
        let (a, b) = self.consistent(other)?;
        if a.powers == b.powers {
            return Some(a);
        }
        if a.hint.is_none() && (a.powers[6] != 0 || b.powers[6] != 0) {
            for d in &DIMENSIONS[..6] {
                let x = a.hinted(*d)?;
                let y = b.hinted(*d)?;
                if x.powers == y.powers {
                    return Some(x);
                }
            }
        }
        None
    }
    fn product(self, other: Self, divide: bool) -> Option<Self> {
        let (mut a, b) = self.consistent(other)?;
        for i in 0..7 {
            a.powers[i] = if divide {
                a.powers[i].checked_sub(b.powers[i])?
            } else {
                a.powers[i].checked_add(b.powers[i])?
            };
        }
        Some(a)
    }
    fn simple(self) -> bool {
        self.powers.iter().all(|p| *p == 0)
            || self.powers.iter().filter(|p| **p == 1).count() == 1
                && self.powers.iter().all(|p| *p == 0 || *p == 1)
    }
    pub(crate) fn is_number(self) -> bool {
        self.powers == [0; 7]
    }
    pub(crate) fn is(self, d: CssNumericDimension) -> bool {
        self.powers == Self::dimension(d).powers
    }
}

/// Classified root types, with complete algebra available for intermediate nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCalculationType {
    Number,
    Percentage,
    Length,
    LengthPercentage,
    Angle,
    AnglePercentage,
    Time,
    TimePercentage,
    Frequency,
    FrequencyPercentage,
    Resolution,
    Flex,
    Algebra(CssNumericType),
}
impl CssCalculationType {
    pub(crate) fn from_numeric(t: CssNumericType) -> Self {
        // A percentage hint does not introduce a dimensional power. Keep the
        // Number category; numeric_type() retains its complete contextual type.
        if t.is_number() {
            return Self::Number;
        }
        if t.hint.is_none() {
            for (d, v) in [
                (CssNumericDimension::Length, Self::Length),
                (CssNumericDimension::Angle, Self::Angle),
                (CssNumericDimension::Time, Self::Time),
                (CssNumericDimension::Frequency, Self::Frequency),
                (CssNumericDimension::Resolution, Self::Resolution),
                (CssNumericDimension::Flex, Self::Flex),
                (CssNumericDimension::Percentage, Self::Percentage),
            ] {
                if t.is(d) {
                    return v;
                }
            }
        }
        if t.hint == Some(CssNumericDimension::Length) && t.is(CssNumericDimension::Length) {
            return Self::LengthPercentage;
        }
        if t.hint == Some(CssNumericDimension::Time) && t.is(CssNumericDimension::Time) {
            return Self::TimePercentage;
        }
        if t.hint == Some(CssNumericDimension::Frequency) && t.is(CssNumericDimension::Frequency) {
            return Self::FrequencyPercentage;
        }
        if t.hint == Some(CssNumericDimension::Angle) && t.is(CssNumericDimension::Angle) {
            return Self::AnglePercentage;
        }
        Self::Algebra(t)
    }
    pub(crate) fn numeric(self) -> CssNumericType {
        match self {
            Self::Number => CssNumericType::NUMBER,
            Self::Percentage => CssNumericType::dimension(CssNumericDimension::Percentage),
            Self::Length => CssNumericType::dimension(CssNumericDimension::Length),
            Self::Angle => CssNumericType::dimension(CssNumericDimension::Angle),
            Self::Time => CssNumericType::dimension(CssNumericDimension::Time),
            Self::Frequency => CssNumericType::dimension(CssNumericDimension::Frequency),
            Self::Resolution => CssNumericType::dimension(CssNumericDimension::Resolution),
            Self::Flex => CssNumericType::dimension(CssNumericDimension::Flex),
            Self::LengthPercentage => CssNumericType::dimension(CssNumericDimension::Length)
                .hinted(CssNumericDimension::Length)
                .unwrap(),
            Self::AnglePercentage => CssNumericType::dimension(CssNumericDimension::Angle)
                .hinted(CssNumericDimension::Angle)
                .unwrap(),
            Self::TimePercentage => CssNumericType::dimension(CssNumericDimension::Time)
                .hinted(CssNumericDimension::Time)
                .unwrap(),
            Self::FrequencyPercentage => CssNumericType::dimension(CssNumericDimension::Frequency)
                .hinted(CssNumericDimension::Frequency)
                .unwrap(),
            Self::Algebra(t) => t,
        }
    }
    /// Returns the exponent of one dimension.
    pub fn exponent(self, d: CssNumericDimension) -> i32 {
        self.numeric().exponent(d)
    }
    /// Returns the percentage hint encoded by this classification.
    ///
    /// The Number category reports `None` even when its originating calculation
    /// retains a hint. Use the expression or checked root's `numeric_type()` for
    /// the complete contextual type. A hint names a dimension without resolving
    /// its percentage basis.
    pub fn percent_hint(self) -> Option<CssNumericDimension> {
        self.numeric().hint
    }
}

#[cfg(test)]
mod calculation_type_classification_tests {
    use super::CssCalculationType;

    /// Values 4 §5.6 names one mixed-percentage type per dimension that admits
    /// percentages; every named classification must survive its own numeric
    /// type, so a missing branch cannot fall through to `Algebra` silently.
    #[test]
    fn every_named_calculation_type_classifies_its_own_numeric_type() {
        use CssCalculationType as T;
        let named = [
            T::Number,
            T::Percentage,
            T::Length,
            T::LengthPercentage,
            T::Angle,
            T::AnglePercentage,
            T::Time,
            T::TimePercentage,
            T::Frequency,
            T::FrequencyPercentage,
            T::Resolution,
            T::Flex,
        ];
        for classification in named {
            // Exhaustive without a wildcard: a new named variant must join `named`.
            match classification {
                T::Number
                | T::Percentage
                | T::Length
                | T::LengthPercentage
                | T::Angle
                | T::AnglePercentage
                | T::Time
                | T::TimePercentage
                | T::Frequency
                | T::FrequencyPercentage
                | T::Resolution
                | T::Flex => {}
                T::Algebra(_) => unreachable!("only named classifications are listed"),
            }
            assert_eq!(
                T::from_numeric(classification.numeric()),
                classification,
                "{classification:?}"
            );
        }
    }
}

/// Intrinsic rejection of checked numeric construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CssNumericConstructionErrorKind {
    EmptyValue,
    MultipleValues,
    RecoveredComponent,
    SubstitutionRequired,
    MalformedExpression,
    UnknownFunction,
    Arity,
    InvalidArgumentType,
    IncompatibleTypes,
    RootDomainMismatch,
    OutOfRange,
    ResourceLimit,
    Component(CssComponentValueErrorKind),
}
/// A rejected checked numeric construction, located at its responsible component.
///
/// The violated invariant is read through the `is_*` predicates. A rejected component is
/// exposed by [`Self::component_error`] and is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNumericConstructionError {
    kind: CssNumericConstructionErrorKind,
    origin: Option<CssValueOrigin>,
    source: Option<Box<CssComponentValueError>>,
    path: Option<Box<[usize]>>,
}
impl CssNumericConstructionError {
    /// The private classification behind the predicates.
    pub(crate) fn kind(&self) -> &CssNumericConstructionErrorKind {
        &self.kind
    }
    crate::error::kind_predicates! { kind;
        /// Returns whether no numeric value was supplied.
        is_empty_value => CssNumericConstructionErrorKind::EmptyValue,
        /// Returns whether more than one numeric value was supplied.
        is_multiple_values => CssNumericConstructionErrorKind::MultipleValues,
        /// Returns whether an implicitly closed component was supplied.
        is_recovered_component => CssNumericConstructionErrorKind::RecoveredComponent,
        /// Returns whether the value needs substitution before checked construction.
        is_substitution_required => CssNumericConstructionErrorKind::SubstitutionRequired,
        /// Returns whether a math expression is malformed.
        is_malformed_expression => CssNumericConstructionErrorKind::MalformedExpression,
        /// Returns whether a math function is unknown.
        is_unknown_function => CssNumericConstructionErrorKind::UnknownFunction,
        /// Returns whether a math function has the wrong number of arguments.
        is_arity => CssNumericConstructionErrorKind::Arity,
        /// Returns whether a math function argument has an invalid type.
        is_invalid_argument_type => CssNumericConstructionErrorKind::InvalidArgumentType,
        /// Returns whether math operands have incompatible types.
        is_incompatible_types => CssNumericConstructionErrorKind::IncompatibleTypes,
        /// Returns whether the value's type does not match the requested numeric domain.
        is_root_domain_mismatch => CssNumericConstructionErrorKind::RootDomainMismatch,
        /// Returns whether a literal lies outside the requested range.
        is_out_of_range => CssNumericConstructionErrorKind::OutOfRange,
        /// Returns whether a nesting, component-count or byte limit, or a refused deep parse,
        /// rejected the components.
        is_resource_limit => CssNumericConstructionErrorKind::ResourceLimit,
    }
    /// Returns the responsible component's origin, when known.
    #[must_use]
    pub fn origin(&self) -> Option<&CssValueOrigin> {
        self.origin.as_ref()
    }
    /// Returns the child-index path to the responsible component inside the value.
    #[must_use]
    pub fn path(&self) -> Option<&[usize]> {
        self.path.as_deref()
    }
    /// Returns the component failure that rejected the value.
    #[must_use]
    pub fn component_error(&self) -> Option<&CssComponentValueError> {
        self.source.as_deref()
    }
    pub(crate) fn at(kind: CssNumericConstructionErrorKind, c: Option<&CssComponentValue>) -> Self {
        Self {
            kind,
            origin: c.map(|c| c.origin().clone()),
            source: None,
            path: None,
        }
    }
    pub(crate) fn at_origin(kind: CssNumericConstructionErrorKind, origin: CssValueOrigin) -> Self {
        Self {
            kind,
            origin: Some(origin),
            source: None,
            path: None,
        }
    }
    pub(crate) fn component(e: CssComponentValueError) -> Self {
        Self {
            kind: if e.is_resource_limit() {
                CssNumericConstructionErrorKind::ResourceLimit
            } else {
                CssNumericConstructionErrorKind::Component(e.kind)
            },
            origin: Some(e.origin().clone()),
            source: Some(Box::new(e)),
            path: None,
        }
    }
}
impl CssNumericConstructionError {
    pub(crate) fn with_path(mut self, path: Option<Box<[usize]>>) -> Self {
        if self.path.is_none() {
            self.path = path;
        }
        self
    }
    pub(crate) fn prefix_path(mut self, prefix: &[usize]) -> Self {
        let mut path = prefix.to_vec();
        if let Some(relative) = self.path.take() {
            path.extend(relative);
        }
        self.path = Some(path.into_boxed_slice());
        self
    }
    pub(crate) fn offset_path(mut self, prefix: &[usize], offset: usize) -> Self {
        let mut path = prefix.to_vec();
        if let Some(relative) = self.path.take() {
            let mut relative = relative.into_vec();
            if let Some(first) = relative.first_mut() {
                *first = first.saturating_add(offset);
            }
            path.extend(relative);
        } else {
            path.push(offset);
        }
        self.path = Some(path.into_boxed_slice());
        self
    }
}
impl fmt::Display for CssNumericConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CssNumericConstructionErrorKind as Kind;
        f.write_str("invalid CSS numeric value: ")?;
        f.write_str(match &self.kind {
            Kind::EmptyValue => "no value was supplied",
            Kind::MultipleValues => "more than one value was supplied",
            Kind::RecoveredComponent => "an implicitly closed component was supplied",
            Kind::SubstitutionRequired => "the value needs substitution",
            Kind::MalformedExpression => "the math expression is malformed",
            Kind::UnknownFunction => "the math function is unknown",
            Kind::Arity => "the math function has the wrong number of arguments",
            Kind::InvalidArgumentType => "a math function argument has an invalid type",
            Kind::IncompatibleTypes => "the math operands have incompatible types",
            Kind::RootDomainMismatch => "the value's type does not match its numeric domain",
            Kind::OutOfRange => "the value is out of range",
            Kind::ResourceLimit => "a resource limit refused the components",
            Kind::Component(_) => "a component is invalid",
        })
    }
}
impl std::error::Error for CssNumericConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|e| e as _)
    }
}
type Result<T> = std::result::Result<T, CssNumericConstructionError>;

/// A checked canonical unit identity, independent of authored unit spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssNumericUnit {
    Length(CssLengthUnit),
    Angle(crate::CssAngleUnit),
    Time(crate::CssTimeUnit),
    Frequency(crate::CssFrequencyUnit),
    Resolution(crate::CssResolutionUnit),
    Flex,
}

impl CssNumericUnit {
    /// Decodes through the one unit registry. The bounded caller supplies its
    /// fixed ASCII name cache; ordinary authored callers lend their existing name.
    pub(crate) fn parse(unit: &str) -> Option<Self> {
        units::parse(unit)
    }
    /// Intrinsic canonical coefficient conversion, derived from the registry's
    /// exact ratio. Contextual lengths and flex values have no invented basis.
    pub(crate) fn intrinsic_projection(self) -> Option<units::Binary64Projection> {
        units::binary64_projection(self)
    }
    pub(crate) const fn dimension(self) -> CssNumericDimension {
        match self {
            Self::Length(_) => CssNumericDimension::Length,
            Self::Angle(_) => CssNumericDimension::Angle,
            Self::Time(_) => CssNumericDimension::Time,
            Self::Frequency(_) => CssNumericDimension::Frequency,
            Self::Resolution(_) => CssNumericDimension::Resolution,
            Self::Flex => CssNumericDimension::Flex,
        }
    }
}

/// A checked exact lexical numeric leaf.
#[derive(Clone, Copy, Debug)]
pub struct CssNumericLiteralRef<'a> {
    component: &'a CssComponentValue,
    ty: CssNumericType,
}
impl<'a> CssNumericLiteralRef<'a> {
    pub fn canonical_unit(self) -> Option<CssNumericUnit> {
        let unit = self.unit()?;
        Some(CssNumericUnit::parse(unit).expect("checked numeric unit"))
    }

    pub fn numeric_type(self) -> CssNumericType {
        self.ty
    }
    pub fn numeric(self) -> CssNumericTokenRef<'a> {
        match self.component.view() {
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(n)
                | CssValueTokenRef::Percentage(n)
                | CssValueTokenRef::Dimension { number: n, .. },
            ) => n,
            _ => unreachable!("checked numeric leaf"),
        }
    }
    pub fn representation(self) -> &'a str {
        self.numeric().representation()
    }
    pub fn unit(self) -> Option<&'a str> {
        match self.component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) => Some(unit),
            _ => None,
        }
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        self.component.origin()
    }
}
impl PartialEq for CssNumericLiteralRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component && self.ty == other.ty
    }
}
impl Eq for CssNumericLiteralRef<'_> {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssCalculationValueRef<'a> {
    Integer(CssNumericLiteralRef<'a>),
    Number(CssNumericLiteralRef<'a>),
    Percentage(CssNumericLiteralRef<'a>),
    Length(CssNumericLiteralRef<'a>),
    Angle(CssNumericLiteralRef<'a>),
    Time(CssNumericLiteralRef<'a>),
    Frequency(CssNumericLiteralRef<'a>),
    Resolution(CssNumericLiteralRef<'a>),
    Flex(CssNumericLiteralRef<'a>),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssNumericConstant {
    E,
    Pi,
    Infinity,
    NegativeInfinity,
    NaN,
}
impl CssNumericConstant {
    pub(crate) fn parse(name: &str) -> Option<Self> {
        cssparser::match_ignore_ascii_case! { name,
            "e" => Some(Self::E), "pi" => Some(Self::Pi),
            "infinity" => Some(Self::Infinity), "-infinity" => Some(Self::NegativeInfinity),
            "nan" => Some(Self::NaN), _ => None,
        }
    }
    pub(crate) fn magnitude(self) -> f64 {
        match self {
            Self::E => std::f64::consts::E,
            Self::Pi => std::f64::consts::PI,
            Self::Infinity => f64::INFINITY,
            Self::NegativeInfinity => f64::NEG_INFINITY,
            Self::NaN => f64::NAN,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::E => "e",
            Self::Pi => "pi",
            Self::Infinity => "infinity",
            Self::NegativeInfinity => "-infinity",
            Self::NaN => "NaN",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssMathFunction {
    Calc,
    Min,
    Max,
    Clamp,
    Round,
    Mod,
    Rem,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Pow,
    Sqrt,
    Hypot,
    Log,
    Exp,
    Abs,
    Sign,
}
impl CssMathFunction {
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Some(cssparser::match_ignore_ascii_case! { name,
            "calc" => Self::Calc,
            "min" => Self::Min,
            "max" => Self::Max,
            "clamp" => Self::Clamp,
            "round" => Self::Round,
            "mod" => Self::Mod,
            "rem" => Self::Rem,
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "tan" => Self::Tan,
            "asin" => Self::Asin,
            "acos" => Self::Acos,
            "atan" => Self::Atan,
            "atan2" => Self::Atan2,
            "pow" => Self::Pow,
            "sqrt" => Self::Sqrt,
            "hypot" => Self::Hypot,
            "log" => Self::Log,
            "exp" => Self::Exp,
            "abs" => Self::Abs,
            "sign" => Self::Sign,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Calc => "calc",
            Self::Min => "min",
            Self::Max => "max",
            Self::Clamp => "clamp",
            Self::Round => "round",
            Self::Mod => "mod",
            Self::Rem => "rem",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Atan2 => "atan2",
            Self::Pow => "pow",
            Self::Sqrt => "sqrt",
            Self::Hypot => "hypot",
            Self::Log => "log",
            Self::Exp => "exp",
            Self::Abs => "abs",
            Self::Sign => "sign",
        }
    }
}
pub(crate) fn is_math_function(name: &str) -> bool {
    CssMathFunction::parse(name).is_some()
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssRoundingStrategy {
    Nearest,
    Up,
    Down,
    ToZero,
}
impl CssRoundingStrategy {
    fn name(self) -> &'static str {
        match self {
            Self::Nearest => "nearest",
            Self::Up => "up",
            Self::Down => "down",
            Self::ToZero => "to-zero",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssCalculationSumOperator {
    Add,
    Subtract,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssCalculationProductOperator {
    Multiply,
    Divide,
}

/// Contextual integer-valued functions evaluated against a container element.
/// These remain symbolic and are admitted only by container size construction,
/// not by the context-free numeric constructors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssTreeCountingFunction {
    SiblingCount,
    SiblingIndex,
}
impl CssTreeCountingFunction {
    fn parse(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("sibling-count") {
            Some(Self::SiblingCount)
        } else if name.eq_ignore_ascii_case("sibling-index") {
            Some(Self::SiblingIndex)
        } else {
            None
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::SiblingCount => "sibling-count",
            Self::SiblingIndex => "sibling-index",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum NodeKind {
    Value(Box<CssComponentValue>),
    Size,
    Constant(CssNumericConstant),
    Variable(CssRelativeColorChannel),
    ProfileChannel(crate::CssColorProfileComponentName),
    TreeCounting(CssTreeCountingFunction),
    Sum(Vec<(Option<CssCalculationSumOperator>, CssCalculationExpression)>),
    Product(
        Vec<(
            Option<CssCalculationProductOperator>,
            CssCalculationExpression,
        )>,
    ),
    Group(Box<CssCalculationExpression>),
    Function {
        function: CssMathFunction,
        args: Vec<Option<CssCalculationExpression>>,
        strategy: Option<CssRoundingStrategy>,
    },
}
impl NodeKind {
    // This is syntactic admission, not numerical resolvability: ordinary
    // dimensions may still need external unit facts. Only these symbolic leaves
    // require a container, calc-size or relative-color expression environment.
    fn admits_pure_context(&self) -> bool {
        match self {
            Self::Value(_) | Self::Constant(_) => true,
            Self::Size | Self::Variable(_) | Self::ProfileChannel(_) | Self::TreeCounting(_) => {
                false
            }
            Self::Sum(values) => values.iter().all(|(_, value)| value.admits_pure_context),
            Self::Product(values) => values.iter().all(|(_, value)| value.admits_pure_context),
            Self::Group(value) => value.admits_pure_context,
            Self::Function { args, .. } => {
                args.iter().flatten().all(|value| value.admits_pure_context)
            }
        }
    }
}
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CssCalculationExpression {
    kind: NodeKind,
    admits_pure_context: bool,
    ty: CssNumericType,
    origin: CssValueOrigin,
    components: Option<CssComponentValues>,
    syntax: Vec<CssComponentValue>,
    closing: Option<CssValueOrigin>,
}
impl Clone for CssCalculationExpression {
    fn clone(&self) -> Self {
        // Operator nodes add numerical depth between admitted nested functions.
        // Rebuild bottom-up so that cloning never recurses through that tree.
        let mut pending = vec![(self, None)];
        let mut completed = Vec::<Self>::new();
        while let Some((node, start)) = pending.pop() {
            let Some(start) = start else {
                pending.push((node, Some(completed.len())));
                match &node.kind {
                    NodeKind::Sum(values) => {
                        pending.extend(values.iter().rev().map(|(_, child)| (child, None)));
                    }
                    NodeKind::Product(values) => {
                        pending.extend(values.iter().rev().map(|(_, child)| (child, None)));
                    }
                    NodeKind::Group(child) => pending.push((child, None)),
                    NodeKind::Function { args, .. } => {
                        pending.extend(args.iter().rev().flatten().map(|child| (child, None)));
                    }
                    NodeKind::Value(_)
                    | NodeKind::Size
                    | NodeKind::Constant(_)
                    | NodeKind::Variable(_)
                    | NodeKind::ProfileChannel(_)
                    | NodeKind::TreeCounting(_) => {}
                }
                continue;
            };
            // Each finish event owns exactly the completed suffix for its
            // children, in their original order. Function rebuilding also
            // retains the positions of absent arguments.
            let mut children = completed.drain(start..);
            let mut next = || children.next().expect("completed numerical child");
            let kind = match &node.kind {
                NodeKind::Value(value) => NodeKind::Value(value.clone()),
                NodeKind::Size => NodeKind::Size,
                NodeKind::Constant(value) => NodeKind::Constant(*value),
                NodeKind::Variable(channel) => NodeKind::Variable(*channel),
                NodeKind::ProfileChannel(name) => NodeKind::ProfileChannel(name.clone()),
                NodeKind::TreeCounting(function) => NodeKind::TreeCounting(*function),
                NodeKind::Sum(values) => NodeKind::Sum(
                    values
                        .iter()
                        .map(|(operator, _)| (*operator, next()))
                        .collect(),
                ),
                NodeKind::Product(values) => NodeKind::Product(
                    values
                        .iter()
                        .map(|(operator, _)| (*operator, next()))
                        .collect(),
                ),
                NodeKind::Group(_) => NodeKind::Group(Box::new(next())),
                NodeKind::Function {
                    function,
                    args,
                    strategy,
                } => NodeKind::Function {
                    function: *function,
                    args: args
                        .iter()
                        .map(|arg| arg.as_ref().map(|_| next()))
                        .collect(),
                    strategy: *strategy,
                },
            };
            drop(children);
            completed.push(Self {
                kind,
                admits_pure_context: node.admits_pure_context,
                ty: node.ty,
                origin: node.origin.clone(),
                components: node.components.clone(),
                syntax: node.syntax.clone(),
                closing: node.closing.clone(),
            });
        }
        completed.pop().expect("completed numerical root")
    }
}
impl CssCalculationExpression {
    /// Cached bottom-up admission of the complete typed tree. Checking reuse at
    /// another owning boundary never scans syntax or reparses retained text.
    pub(crate) const fn admits_pure_context(&self) -> bool {
        self.admits_pure_context
    }

    /// CSS Typed OM 1 "equal numeric values" over the checked tree: the same
    /// operations and functions with pairwise equal operands in order. Leaves
    /// compare by unit and exact value; source origins, spelling and the raw
    /// component graph are ignored. Parenthesised blocks and nested `calc()`
    /// functions leave no node (Values 4 "parse a calculation"), so they are
    /// seen through; every other math function is compared as a node.
    pub(crate) fn value_eq(&self, other: &Self) -> bool {
        Self::structural_eq_with(
            false,
            true,
            vec![(self, other)],
            |_, _| true,
            |a, b| {
                Ok::<_, std::convert::Infallible>(crate::specified_numeric::ordinary_literal_equal(
                    a, b,
                ))
            },
            |pending, value| {
                pending.push(value);
                Ok(())
            },
        )
        .unwrap_or_else(|never| match never {})
    }

    /// Compares checked numeric structure and authored spelling while
    /// excluding diagnostic source coordinates and the redundant raw root
    /// graph: the specified serializer's shortening identity.
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        Self::structural_eq_with(
            true,
            false,
            vec![(self, other)],
            |a, b| a.structural_eq_ignoring_origin(b),
            |a, b| Ok::<_, std::convert::Infallible>(a.structural_eq_ignoring_origin(b)),
            |pending, value| {
                pending.push(value);
                Ok(())
            },
        )
        .unwrap_or_else(|never| match never {})
    }

    /// Exact retained leaf identity within the same checked expression structure.
    /// Does not solve differing expression trees or use projected magnitudes.
    pub(crate) fn specified_identity_eq(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> std::result::Result<bool, crate::CssSerializationError> {
        Self::structural_eq_with(
            false,
            false,
            vec![(self, other)],
            |a, b| a.structural_eq_ignoring_origin(b),
            |a, b| exact_numeric_leaf_identity(a, b, context),
            |pending, value| {
                pending.push(value);
                Ok(())
            },
        )
    }

    /// The new inverse consumer retains the exact structural relation, with
    /// every real comparison work slot admitted before fallible allocation.
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> std::result::Result<bool, crate::CssSerializationError> {
        use crate::{
            CssSerializationError as Error, CssSpecifiedValueSerializationErrorKind as Kind,
        };
        context.charge_generated_projection(1)?;
        let mut pending = Vec::new();
        pending
            .try_reserve(1)
            .map_err(|_| Error::new(Kind::CapacityOverflow))?;
        pending.push((self, other));
        Self::structural_eq_with(
            true,
            false,
            pending,
            |a, b| {
                a.token_structural_eq_ignoring_origin(b)
                    .expect("checked numeric syntax token")
            },
            |a, b| {
                Ok(a.token_structural_eq_ignoring_origin(b)
                    .expect("checked numeric literal token"))
            },
            |pending, value| {
                context.charge_generated_projection(1)?;
                pending
                    .try_reserve(1)
                    .map_err(|_| Error::new(Kind::CapacityOverflow))?;
                pending.push(value);
                Ok(())
            },
        )
    }

    /// The node a value comparison continues with: Values 4 "parse a
    /// calculation" leaves no node for a parenthesised block or a nested
    /// `calc()`, so both are seen through. Every other math function is kept.
    fn without_grouping(&self) -> &Self {
        let mut node = self;
        loop {
            node = match &node.kind {
                NodeKind::Group(inner) => inner,
                NodeKind::Function {
                    function: CssMathFunction::Calc,
                    args,
                    ..
                } => match args.as_slice() {
                    [Some(inner)] => inner,
                    _ => return node,
                },
                _ => return node,
            };
        }
    }

    fn structural_eq_with<'a, E>(
        compare_raw_syntax: bool,
        see_through_grouping: bool,
        mut pending: Vec<(&'a Self, &'a Self)>,
        mut syntax_eq: impl FnMut(&CssComponentValue, &CssComponentValue) -> bool,
        mut value_eq: impl FnMut(&CssComponentValue, &CssComponentValue) -> std::result::Result<bool, E>,
        mut push: impl FnMut(
            &mut Vec<(&'a Self, &'a Self)>,
            (&'a Self, &'a Self),
        ) -> std::result::Result<(), E>,
    ) -> std::result::Result<bool, E> {
        let child = |node: &'a Self| {
            if see_through_grouping {
                node.without_grouping()
            } else {
                node
            }
        };
        while let Some((left, right)) = pending.pop() {
            if left.ty != right.ty
                || (compare_raw_syntax
                    && (left.syntax.len() != right.syntax.len()
                        || !left
                            .syntax
                            .iter()
                            .zip(&right.syntax)
                            .all(|(a, b)| syntax_eq(a, b))))
                || left.closing.is_some() != right.closing.is_some()
            {
                return Ok(false);
            }
            match (&left.kind, &right.kind) {
                (NodeKind::Value(a), NodeKind::Value(b)) => {
                    if !value_eq(a, b)? {
                        return Ok(false);
                    }
                }
                (NodeKind::Size, NodeKind::Size) => {}
                (NodeKind::Constant(a), NodeKind::Constant(b)) if a == b => {}
                (NodeKind::Variable(a), NodeKind::Variable(b)) if a == b => {}
                (NodeKind::ProfileChannel(a), NodeKind::ProfileChannel(b)) if a == b => {}
                (NodeKind::TreeCounting(a), NodeKind::TreeCounting(b)) if a == b => {}
                (NodeKind::Sum(a), NodeKind::Sum(b)) if a.len() == b.len() => {
                    for ((a_operator, a_child), (b_operator, b_child)) in a.iter().zip(b).rev() {
                        if a_operator != b_operator {
                            return Ok(false);
                        }
                        push(&mut pending, (child(a_child), child(b_child)))?;
                    }
                }
                (NodeKind::Product(a), NodeKind::Product(b)) if a.len() == b.len() => {
                    for ((a_operator, a_child), (b_operator, b_child)) in a.iter().zip(b).rev() {
                        if a_operator != b_operator {
                            return Ok(false);
                        }
                        push(&mut pending, (child(a_child), child(b_child)))?;
                    }
                }
                (NodeKind::Group(a), NodeKind::Group(b)) => {
                    push(&mut pending, (child(a), child(b)))?
                }
                (
                    NodeKind::Function {
                        function: a_function,
                        args: a_args,
                        strategy: a_strategy,
                    },
                    NodeKind::Function {
                        function: b_function,
                        args: b_args,
                        strategy: b_strategy,
                    },
                ) if a_function == b_function
                    && a_strategy == b_strategy
                    && a_args.len() == b_args.len() =>
                {
                    for (a, b) in a_args.iter().zip(b_args).rev() {
                        match (a, b) {
                            (Some(a), Some(b)) => push(&mut pending, (child(a), child(b)))?,
                            (None, None) => {}
                            _ => return Ok(false),
                        }
                    }
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.components
            .as_ref()
            .is_some_and(|v| v.first_implicit_origin().is_none())
    }

    pub(crate) fn component_nesting_depth(&self) -> u32 {
        self.components
            .as_ref()
            .expect("checked root component graph")
            .nesting_depth()
    }

    fn canonical_len(&self, limit: usize) -> Option<usize> {
        let mut nodes = vec![self];
        let mut length = 0usize;
        while let Some(node) = nodes.pop() {
            let added = match &node.kind {
                NodeKind::Value(c) => {
                    let literal = CssNumericLiteralRef {
                        component: c,
                        ty: node.ty,
                    };
                    literal.representation().len().checked_add(match c.view() {
                        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => 1,
                        CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                            unit, ..
                        }) => unit.len(),
                        _ => 0,
                    })?
                }
                NodeKind::ProfileChannel(c) => escaped_profile_name(c).len(),
                NodeKind::Size => 4,
                NodeKind::Constant(c) => c.name().len(),
                NodeKind::TreeCounting(function) => function.name().len().checked_add(2)?,
                NodeKind::Variable(c) => {
                    if *c == CssRelativeColorChannel::Alpha {
                        5
                    } else {
                        1
                    }
                }
                NodeKind::Group(value) => {
                    nodes.push(value);
                    2
                }
                NodeKind::Sum(values) => {
                    nodes.extend(values.iter().map(|(_, v)| v));
                    values.len().checked_sub(1)?.checked_mul(3)?
                }
                NodeKind::Product(values) => {
                    nodes.extend(values.iter().map(|(_, v)| v));
                    values.len().checked_sub(1)?.checked_mul(3)?
                }
                NodeKind::Function {
                    function,
                    args,
                    strategy,
                } => {
                    nodes.extend(args.iter().flatten());
                    function
                        .as_str()
                        .len()
                        .checked_add(2)?
                        .checked_add(args.len().checked_sub(1)?.checked_mul(2)?)?
                        .checked_add(args.iter().filter(|a| a.is_none()).count().checked_mul(4)?)?
                        .checked_add(strategy.map_or(0, |s| s.name().len() + 2))?
                }
            };
            length = length.checked_add(added)?;
            if length > limit {
                return None;
            }
        }
        Some(length)
    }
    fn syntax_items(&self) -> &[CssComponentValue] {
        &self.syntax
    }
    fn closing_origin(&self) -> &CssValueOrigin {
        self.closing.as_ref().unwrap_or(&self.origin)
    }
    /// Emits canonical tokens without consuming call stack per expression node.
    /// Both string and provenance serialization use this one traversal.
    fn emit_canonical(&self, mut emit: impl FnMut(String, &CssValueOrigin)) {
        enum Work<'a> {
            Visit(&'a CssCalculationExpression),
            Token(String, &'a CssValueOrigin),
        }
        let mut pending = vec![Work::Visit(self)];
        while let Some(work) = pending.pop() {
            let node = match work {
                Work::Token(text, origin) => {
                    emit(text, origin);
                    continue;
                }
                Work::Visit(node) => node,
            };
            let mut next = Vec::new();
            match &node.kind {
                NodeKind::Value(component) => {
                    let literal = CssNumericLiteralRef {
                        component,
                        ty: node.ty,
                    };
                    let text = match component.view() {
                        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                            format!("{}%", literal.representation())
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                            unit, ..
                        }) => format!("{}{}", literal.representation(), unit.to_ascii_lowercase()),
                        _ => literal.representation().to_owned(),
                    };
                    emit(text, &node.origin);
                }
                NodeKind::ProfileChannel(name) => emit(escaped_profile_name(name), &node.origin),
                NodeKind::Size => emit("size".into(), &node.origin),
                NodeKind::Constant(constant) => emit(constant.name().to_owned(), &node.origin),
                NodeKind::TreeCounting(function) => {
                    emit(format!("{}(", function.name()), &node.origin);
                    emit(")".into(), node.closing_origin());
                }
                NodeKind::Variable(channel) => {
                    emit(format!("{channel:?}").to_ascii_lowercase(), &node.origin)
                }
                NodeKind::Group(value) => {
                    emit("(".into(), &node.origin);
                    next.push(Work::Visit(value));
                    next.push(Work::Token(")".into(), node.closing_origin()));
                }
                NodeKind::Sum(values) => {
                    let mut operators = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Delim('+' | '-'))
                        )
                    });
                    for (operator, value) in values {
                        if let Some(operator) = operator {
                            let text = match operator {
                                CssCalculationSumOperator::Add => " + ",
                                CssCalculationSumOperator::Subtract => " - ",
                            };
                            next.push(Work::Token(
                                text.into(),
                                operators.next().expect("checked sum operator").origin(),
                            ));
                        }
                        next.push(Work::Visit(value));
                    }
                }
                NodeKind::Product(values) => {
                    let mut operators = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Delim('*' | '/'))
                        )
                    });
                    for (operator, value) in values {
                        if let Some(operator) = operator {
                            let text = match operator {
                                CssCalculationProductOperator::Multiply => " * ",
                                CssCalculationProductOperator::Divide => " / ",
                            };
                            next.push(Work::Token(
                                text.into(),
                                operators.next().expect("checked product operator").origin(),
                            ));
                        }
                        next.push(Work::Visit(value));
                    }
                }
                NodeKind::Function {
                    function,
                    args,
                    strategy,
                } => {
                    emit(format!("{}(", function.as_str()), &node.origin);
                    let mut commas = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Comma)
                        )
                    });
                    let mut names = node.syntax_items().iter().filter(|component| {
                        matches!(
                            component.view(),
                            CssComponentValueRef::Token(CssValueTokenRef::Ident(_))
                        )
                    });
                    if let Some(strategy) = strategy {
                        next.push(Work::Token(
                            strategy.name().into(),
                            names.next().expect("checked strategy").origin(),
                        ));
                        next.push(Work::Token(
                            ", ".into(),
                            commas.next().expect("checked strategy comma").origin(),
                        ));
                    }
                    for (index, argument) in args.iter().enumerate() {
                        if index > 0 {
                            next.push(Work::Token(
                                ", ".into(),
                                commas.next().expect("checked argument comma").origin(),
                            ));
                        }
                        if let Some(argument) = argument {
                            next.push(Work::Visit(argument));
                        } else {
                            next.push(Work::Token(
                                "none".into(),
                                names
                                    .find(|component| ident(component, "none"))
                                    .expect("checked absent bound")
                                    .origin(),
                            ));
                        }
                    }
                    next.push(Work::Token(")".into(), node.closing_origin()));
                }
            }
            pending.extend(next.into_iter().rev());
        }
    }
    fn canonical_tokens(&self, out: &mut Vec<(String, CssValueOrigin)>) {
        self.emit_canonical(|text, origin| out.push((text, origin.clone())));
    }
    pub(crate) fn references(&self) -> Vec<CssRelativeColorChannel> {
        let mut found = Vec::new();
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            match &node.kind {
                NodeKind::Variable(c) => found.push(*c),
                NodeKind::Sum(v) => stack.extend(v.iter().rev().map(|(_, e)| e)),
                NodeKind::Product(v) => stack.extend(v.iter().rev().map(|(_, e)| e)),
                NodeKind::Group(e) => stack.push(e),
                NodeKind::Function { args, .. } => stack.extend(args.iter().rev().flatten()),
                _ => {}
            }
        }
        found
    }
    pub(crate) fn result_type(&self) -> CssCalculationType {
        CssCalculationType::from_numeric(self.ty)
    }
    pub(crate) fn as_ref(&self) -> CssCalculationExpressionRef<'_> {
        match &self.kind {
            NodeKind::Value(c) => {
                let v = CssNumericLiteralRef {
                    component: c,
                    ty: self.ty,
                };
                let value = match c.view() {
                    CssComponentValueRef::Token(CssValueTokenRef::Number(n)) => {
                        if n.kind() == CssNumericTokenKind::Integer {
                            CssCalculationValueRef::Integer(v)
                        } else {
                            CssCalculationValueRef::Number(v)
                        }
                    }
                    CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                        CssCalculationValueRef::Percentage(v)
                    }
                    _ => match unit_dimension(v.unit().unwrap()) {
                        Some(CssNumericDimension::Length) => CssCalculationValueRef::Length(v),
                        Some(CssNumericDimension::Angle) => CssCalculationValueRef::Angle(v),
                        Some(CssNumericDimension::Time) => CssCalculationValueRef::Time(v),
                        Some(CssNumericDimension::Frequency) => {
                            CssCalculationValueRef::Frequency(v)
                        }
                        Some(CssNumericDimension::Resolution) => {
                            CssCalculationValueRef::Resolution(v)
                        }
                        Some(CssNumericDimension::Flex) => CssCalculationValueRef::Flex(v),
                        _ => unreachable!("checked dimension"),
                    },
                };
                CssCalculationExpressionRef::Value(value)
            }
            NodeKind::Constant(_) => {
                CssCalculationExpressionRef::Constant(CssCalculationConstantRef { node: self })
            }
            NodeKind::Size => {
                CssCalculationExpressionRef::Size(CssCalculationSizeRef { node: self })
            }
            NodeKind::TreeCounting(_) => {
                CssCalculationExpressionRef::TreeCounting(CssCalculationTreeCountingRef {
                    node: self,
                })
            }
            NodeKind::ProfileChannel(_) => {
                CssCalculationExpressionRef::ProfileChannel(CssCalculationProfileChannelRef {
                    node: self,
                })
            }
            NodeKind::Variable(_) => {
                CssCalculationExpressionRef::Variable(CssCalculationVariableRef { node: self })
            }
            NodeKind::Sum(_) => {
                CssCalculationExpressionRef::Sum(CssCalculationSumRef { node: self })
            }
            NodeKind::Product(_) => {
                CssCalculationExpressionRef::Product(CssCalculationProductRef { node: self })
            }
            NodeKind::Group(_) => {
                CssCalculationExpressionRef::Group(CssCalculationUnaryRef { node: self })
            }
            NodeKind::Function {
                function: CssMathFunction::Calc,
                ..
            } => CssCalculationExpressionRef::NestedCalc(CssCalculationUnaryRef { node: self }),
            NodeKind::Function { .. } => {
                CssCalculationExpressionRef::Function(CssCalculationFunctionRef { node: self })
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssCalculationExpressionRef<'a> {
    Value(CssCalculationValueRef<'a>),
    Size(CssCalculationSizeRef<'a>),
    Constant(CssCalculationConstantRef<'a>),
    Variable(CssCalculationVariableRef<'a>),
    ProfileChannel(CssCalculationProfileChannelRef<'a>),
    TreeCounting(CssCalculationTreeCountingRef<'a>),
    Sum(CssCalculationSumRef<'a>),
    Product(CssCalculationProductRef<'a>),
    Group(CssCalculationUnaryRef<'a>),
    NestedCalc(CssCalculationUnaryRef<'a>),
    Function(CssCalculationFunctionRef<'a>),
}
impl<'a> CssCalculationExpressionRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        match self {
            Self::Value(v) => v.literal().origin(),
            Self::Size(v) => v.origin(),
            Self::Constant(v) => v.origin(),
            Self::Variable(v) => v.origin(),
            Self::ProfileChannel(v) => v.origin(),
            Self::TreeCounting(v) => v.origin(),
            Self::Sum(v) => v.origin(),
            Self::Product(v) => v.origin(),
            Self::Group(v) | Self::NestedCalc(v) => v.origin(),
            Self::Function(v) => v.origin(),
        }
    }
    pub fn numeric_type(self) -> CssNumericType {
        match self {
            Self::Value(v) => v.literal().numeric_type(),
            Self::Size(v) => v.numeric_type(),
            Self::Constant(v) => v.node.ty,
            Self::Variable(v) => v.node.ty,
            Self::ProfileChannel(v) => v.node.ty,
            Self::TreeCounting(v) => v.node.ty,
            Self::Sum(v) => v.node.ty,
            Self::Product(v) => v.node.ty,
            Self::Group(v) | Self::NestedCalc(v) => v.node.ty,
            Self::Function(v) => v.node.ty,
        }
    }
}
/// The symbolic `size` value within a checked calc-size calculation.
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationSizeRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationSizeRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.node.ty
    }
}
impl<'a> CssCalculationValueRef<'a> {
    pub fn literal(self) -> CssNumericLiteralRef<'a> {
        match self {
            Self::Integer(v)
            | Self::Number(v)
            | Self::Percentage(v)
            | Self::Length(v)
            | Self::Angle(v)
            | Self::Time(v)
            | Self::Frequency(v)
            | Self::Resolution(v)
            | Self::Flex(v) => v,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationConstantRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationConstantRef<'a> {
    pub fn value(self) -> CssNumericConstant {
        let NodeKind::Constant(v) = self.node.kind else {
            unreachable!()
        };
        v
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.node.ty
    }
}
/// Borrowed symbolic tree-counting node; its integer result has number type.
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationTreeCountingRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationTreeCountingRef<'a> {
    pub fn function(self) -> CssTreeCountingFunction {
        let NodeKind::TreeCounting(function) = self.node.kind else {
            unreachable!()
        };
        function
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.node.ty
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationVariableRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationVariableRef<'a> {
    pub fn channel(self) -> CssRelativeColorChannel {
        let NodeKind::Variable(v) = self.node.kind else {
            unreachable!()
        };
        v
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.node.ty
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationSumRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationSumRef<'a> {
    pub fn len(self) -> usize {
        match &self.node.kind {
            NodeKind::Sum(v) => v.len(),
            _ => unreachable!(),
        }
    }
    pub fn is_empty(self) -> bool {
        false
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn term(self, i: usize) -> Option<CssCalculationSumTermRef<'a>> {
        let NodeKind::Sum(v) = &self.node.kind else {
            unreachable!()
        };
        v.get(i)
            .map(|(operator, expression)| CssCalculationSumTermRef {
                operator: *operator,
                expression,
                operator_origin: i
                    .checked_sub(1)
                    .and_then(|n| {
                        self.node
                            .syntax
                            .iter()
                            .filter(|c| {
                                matches!(
                                    c.view(),
                                    CssComponentValueRef::Token(CssValueTokenRef::Delim('+' | '-'))
                                )
                            })
                            .nth(n)
                    })
                    .map(CssComponentValue::origin),
            })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationSumTermRef<'a> {
    operator: Option<CssCalculationSumOperator>,
    expression: &'a CssCalculationExpression,
    operator_origin: Option<&'a CssValueOrigin>,
}
impl<'a> CssCalculationSumTermRef<'a> {
    pub fn operator_origin(self) -> Option<&'a CssValueOrigin> {
        self.operator_origin
    }
    pub fn operator(self) -> Option<CssCalculationSumOperator> {
        self.operator
    }
    pub fn expression(self) -> CssCalculationExpressionRef<'a> {
        self.expression.as_ref()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationProductRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationProductRef<'a> {
    pub fn len(self) -> usize {
        match &self.node.kind {
            NodeKind::Product(v) => v.len(),
            _ => unreachable!(),
        }
    }
    pub fn is_empty(self) -> bool {
        false
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn factor(self, i: usize) -> Option<CssCalculationProductFactorRef<'a>> {
        let NodeKind::Product(v) = &self.node.kind else {
            unreachable!()
        };
        v.get(i)
            .map(|(operator, expression)| CssCalculationProductFactorRef {
                operator: *operator,
                expression,
                operator_origin: i
                    .checked_sub(1)
                    .and_then(|n| {
                        self.node
                            .syntax
                            .iter()
                            .filter(|c| {
                                matches!(
                                    c.view(),
                                    CssComponentValueRef::Token(CssValueTokenRef::Delim('*' | '/'))
                                )
                            })
                            .nth(n)
                    })
                    .map(CssComponentValue::origin),
            })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationProductFactorRef<'a> {
    operator: Option<CssCalculationProductOperator>,
    expression: &'a CssCalculationExpression,
    operator_origin: Option<&'a CssValueOrigin>,
}
impl<'a> CssCalculationProductFactorRef<'a> {
    pub fn operator_origin(self) -> Option<&'a CssValueOrigin> {
        self.operator_origin
    }
    pub fn operator(self) -> Option<CssCalculationProductOperator> {
        self.operator
    }
    pub fn expression(self) -> CssCalculationExpressionRef<'a> {
        self.expression.as_ref()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationUnaryRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationUnaryRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn operand(self) -> CssCalculationExpressionRef<'a> {
        match &self.node.kind {
            NodeKind::Group(v) => v.as_ref().as_ref(),
            NodeKind::Function { args, .. } => args[0].as_ref().unwrap().as_ref(),
            _ => unreachable!(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationFunctionRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationFunctionRef<'a> {
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn function(self) -> CssMathFunction {
        let NodeKind::Function { function, .. } = &self.node.kind else {
            unreachable!()
        };
        *function
    }
    pub fn len(self) -> usize {
        let NodeKind::Function { args, .. } = &self.node.kind else {
            unreachable!()
        };
        args.len()
    }
    pub fn is_empty(self) -> bool {
        false
    }
    pub fn argument(self, i: usize) -> Option<Option<CssCalculationExpressionRef<'a>>> {
        let NodeKind::Function { args, .. } = &self.node.kind else {
            unreachable!()
        };
        args.get(i)
            .map(|a| a.as_ref().map(CssCalculationExpression::as_ref))
    }
    pub fn rounding_strategy(self) -> Option<CssRoundingStrategy> {
        let NodeKind::Function { strategy, .. } = &self.node.kind else {
            unreachable!()
        };
        *strategy
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CalculationRoot {
    ProfileRelative,
    NumberPercentage,
    NamedDimensionOrNumber,
    Number,
    HintedNumber,
    Integer,
    Percentage,
    Length,
    LengthPercentage,
    Flex,
    Angle,
    AnglePercentage,
    Time,
    Frequency,
    FrequencyPercentage,
    Resolution,
    Relative(
        crate::CssRelativeColorEnvironment,
        crate::CssRelativeColorResultDomain,
    ),
}
impl CalculationRoot {
    fn hint(self) -> Option<CssNumericDimension> {
        match self {
            Self::LengthPercentage => Some(CssNumericDimension::Length),
            Self::AnglePercentage => Some(CssNumericDimension::Angle),
            Self::FrequencyPercentage => Some(CssNumericDimension::Frequency),
            _ => None,
        }
    }
    fn accepts(self, t: CssNumericType) -> bool {
        match self {
            Self::NamedDimensionOrNumber => {
                t.hint.is_none()
                    && (t.is_number() || DIMENSIONS[..6].iter().any(|dimension| t.is(*dimension)))
            }
            Self::Number | Self::Integer => t.is_number() && t.hint.is_none(),
            Self::HintedNumber => t.is_number() && t.hint.is_some(),
            Self::Percentage => t.is(CssNumericDimension::Percentage) && t.hint.is_none(),
            Self::Length => t.is(CssNumericDimension::Length) && t.hint.is_none(),
            Self::LengthPercentage => t.is(CssNumericDimension::Length),
            Self::Flex => t.is(CssNumericDimension::Flex) && t.hint.is_none(),
            Self::Angle => t.is(CssNumericDimension::Angle) && t.hint.is_none(),
            Self::AnglePercentage => {
                t.is(CssNumericDimension::Angle)
                    && (t.hint.is_none() || t.hint == Some(CssNumericDimension::Angle))
            }
            Self::Time => t.is(CssNumericDimension::Time) && t.hint.is_none(),
            Self::Frequency => t.is(CssNumericDimension::Frequency) && t.hint.is_none(),
            Self::FrequencyPercentage => {
                t.is(CssNumericDimension::Frequency)
                    && (t.hint.is_none() || t.hint == Some(CssNumericDimension::Frequency))
            }
            Self::Resolution => t.is(CssNumericDimension::Resolution) && t.hint.is_none(),
            Self::Relative(_, crate::CssRelativeColorResultDomain::Hue) => {
                t.hint.is_none() && (t.is_number() || t.is(CssNumericDimension::Angle))
            }
            Self::NumberPercentage | Self::ProfileRelative | Self::Relative(_, _) => {
                t.is_number() || t.is(CssNumericDimension::Percentage)
            }
        }
    }
}

pub(crate) fn literal_type(unit: Option<CssNumericUnit>, percentage: bool) -> CssNumericType {
    if percentage {
        CssNumericType::dimension(CssNumericDimension::Percentage)
    } else {
        unit.map_or(CssNumericType::NUMBER, |unit| {
            CssNumericType::dimension(unit.dimension())
        })
    }
}

fn unit_dimension(unit: &str) -> Option<CssNumericDimension> {
    CssNumericUnit::parse(unit).map(CssNumericUnit::dimension)
}

fn trivia(c: &CssComponentValue) -> bool {
    matches!(
        c.view(),
        CssComponentValueRef::Comment(_)
            | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
    )
}
fn direct_syntax(items: &[CssComponentValue]) -> Vec<CssComponentValue> {
    items
        .iter()
        .filter(|c| {
            matches!(
                c.view(),
                CssComponentValueRef::Token(
                    CssValueTokenRef::Ident(_)
                        | CssValueTokenRef::Comma
                        | CssValueTokenRef::Delim(_)
                )
            )
        })
        .cloned()
        .collect()
}
fn ident(c: &CssComponentValue, name: &str) -> bool {
    matches!(c.view(),CssComponentValueRef::Token(CssValueTokenRef::Ident(s))if s.eq_ignore_ascii_case(name))
}
fn exact_zero(n: CssNumericTokenRef<'_>) -> bool {
    n.representation()
        .split(['e', 'E'])
        .next()
        .unwrap()
        .chars()
        .all(|c| matches!(c, '+' | '-' | '.' | '0'))
}

// These keys identify occurrences in the borrowed component graph only. They
// never escape admission, and neither token equality nor origin equality is
// used to combine independently positioned components.
#[derive(Clone, Copy)]
struct ComponentKey<'a>(&'a CssComponentValue);
impl PartialEq for ComponentKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for ComponentKey<'_> {}
impl std::hash::Hash for ComponentKey<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&std::ptr::from_ref(self.0), state);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum NumericAdmissionContext {
    Pure,
    ContainerSize,
    CalcSizeCalculation,
}

struct NumericParser<'a> {
    root: CalculationRoot,
    context: NumericAdmissionContext,
    ready: std::collections::HashMap<ComponentKey<'a>, CssCalculationExpression>,
    paths: std::collections::HashMap<ComponentKey<'a>, Box<[usize]>>,
}
impl<'a> NumericParser<'a> {
    fn error(
        &self,
        kind: CssNumericConstructionErrorKind,
        component: Option<&'a CssComponentValue>,
    ) -> CssNumericConstructionError {
        CssNumericConstructionError::at(kind, component)
            .with_path(component.and_then(|c| self.paths.get(&ComponentKey(c)).cloned()))
    }
    fn take(&mut self, component: &'a CssComponentValue) -> Result<CssCalculationExpression> {
        if let Some(expression) = self.ready.remove(&ComponentKey(component)) {
            return Ok(expression);
        }
        if matches!(
            component.view(),
            CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
        ) {
            return Err(self.error(
                CssNumericConstructionErrorKind::MalformedExpression,
                Some(component),
            ));
        }
        parse_node(component, self)
            .map_err(|e| e.with_path(self.paths.get(&ComponentKey(component)).cloned()))
    }
}

fn parse_tree(
    component: &CssComponentValue,
    root: CalculationRoot,
    context: NumericAdmissionContext,
    root_index: usize,
) -> Result<CssCalculationExpression> {
    let mut parser = NumericParser {
        root,
        context,
        ready: std::collections::HashMap::new(),
        paths: std::collections::HashMap::new(),
    };
    let mut pending = vec![(component, vec![root_index], false)];
    while let Some((current, path, visited)) = pending.pop() {
        if visited {
            let expression = parse_node(current, &mut parser)
                .map_err(|e| e.with_path(Some(path.into_boxed_slice())))?;
            parser.ready.insert(ComponentKey(current), expression);
            continue;
        }
        parser
            .paths
            .insert(ComponentKey(current), path.clone().into_boxed_slice());
        pending.push((current, path.clone(), true));
        let children = match current.view() {
            CssComponentValueRef::Function(f) => f.values().items(),
            CssComponentValueRef::Block(b) => b.values().items(),
            _ => &[],
        };
        for (index, child) in children.iter().enumerate().rev() {
            let mut child_path = path.clone();
            child_path.push(index);
            parser
                .paths
                .insert(ComponentKey(child), child_path.clone().into_boxed_slice());
            if matches!(
                child.view(),
                CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
            ) {
                pending.push((child, child_path, false));
            }
        }
    }
    Ok(parser
        .ready
        .remove(&ComponentKey(component))
        .expect("admitted root node"))
}

/// Parses one strict calc-sum from an already validated parent graph. `size`
/// is enabled only for the non-`any` calculation branch of calc-size().
pub(crate) fn parse_calc_size_sum(
    items: &[CssComponentValue],
    allow_size: bool,
) -> Result<CssCalculationExpression> {
    let context = if allow_size {
        NumericAdmissionContext::CalcSizeCalculation
    } else {
        NumericAdmissionContext::Pure
    };
    let mut parser = NumericParser {
        root: CalculationRoot::LengthPercentage,
        context,
        ready: std::collections::HashMap::new(),
        paths: std::collections::HashMap::new(),
    };
    for (index, component) in items.iter().enumerate() {
        parser
            .paths
            .insert(ComponentKey(component), vec![index].into_boxed_slice());
        if matches!(
            component.view(),
            CssComponentValueRef::Function(_) | CssComponentValueRef::Block(_)
        ) {
            let expression = parse_tree(component, parser.root, parser.context, index)?;
            parser.ready.insert(ComponentKey(component), expression);
        }
    }
    let expression = sequence(items, &mut parser)?;
    if !CalculationRoot::LengthPercentage.accepts(expression.ty) {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            items.iter().find(|value| !trivia(value)),
        )
        .with_path(
            items
                .iter()
                .position(|value| !trivia(value))
                .map(|index| vec![index].into_boxed_slice()),
        ));
    }
    Ok(expression)
}

pub(crate) fn validate_calc_size_graph(
    values: &CssComponentValues,
    limits: CssComponentValueLimits,
    recovered: bool,
) -> Result<()> {
    if !recovered && let Some(origin) = values.first_implicit_origin() {
        return Err(CssNumericConstructionError::at_origin(
            CssNumericConstructionErrorKind::RecoveredComponent,
            origin.clone(),
        ));
    }
    validate_components(
        values.items(),
        limits,
        if recovered {
            AdmissionPolicy::RecoveredSyntax
        } else {
            AdmissionPolicy::Strict
        },
    )
}

fn sequence<'a>(
    items: &'a [CssComponentValue],
    parser: &mut NumericParser<'a>,
) -> Result<CssCalculationExpression> {
    ordinary_expression::parse(items, parser, None)
}
fn parse_node<'a>(
    c: &'a CssComponentValue,
    parser: &mut NumericParser<'a>,
) -> Result<CssCalculationExpression> {
    let root = parser.root;
    let error = |kind| CssNumericConstructionError::at(kind, Some(c));
    let intrinsic = intrinsic_token_type(c).transpose()?;
    let (kind, mut ty) = match c.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => (
            NodeKind::Value(Box::new(c.clone())),
            intrinsic.expect("number type"),
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => (
            NodeKind::Value(Box::new(c.clone())),
            intrinsic.expect("percentage type"),
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. }) => (
            NodeKind::Value(Box::new(c.clone())),
            intrinsic.expect("dimension type"),
        ),
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => {
            let constant = CssNumericConstant::parse(name);
            if let Some(v) = constant {
                (NodeKind::Constant(v), intrinsic.expect("constant type"))
            } else if root == CalculationRoot::ProfileRelative {
                let name = crate::CssColorProfileComponentName::try_new(name)
                    .ok_or_else(|| error(CssNumericConstructionErrorKind::InvalidArgumentType))?;
                (NodeKind::ProfileChannel(name), CssNumericType::NUMBER)
            } else if let CalculationRoot::Relative(environment, _) = root {
                let (channel, ty) =
                    crate::syntax::color::numeric_relative_channel(environment, name).ok_or_else(
                        || error(CssNumericConstructionErrorKind::InvalidArgumentType),
                    )?;
                (NodeKind::Variable(channel), ty.numeric())
            } else if parser.context == NumericAdmissionContext::CalcSizeCalculation
                && name.eq_ignore_ascii_case("size")
            {
                (
                    NodeKind::Size,
                    CssNumericType::dimension(CssNumericDimension::Length),
                )
            } else {
                return Err(error(CssNumericConstructionErrorKind::InvalidArgumentType));
            }
        }
        CssComponentValueRef::Block(b) if b.kind() == CssBlockKind::Parenthesis => {
            let child = sequence(b.values().items(), parser)?;
            let ty = child.ty;
            (NodeKind::Group(Box::new(child)), ty)
        }
        CssComponentValueRef::Function(f) => {
            if f.name().eq_ignore_ascii_case("var") {
                return Err(error(CssNumericConstructionErrorKind::SubstitutionRequired));
            }
            if parser.context == NumericAdmissionContext::ContainerSize
                && let Some(function) = CssTreeCountingFunction::parse(f.name())
            {
                if !f.values().items().iter().all(trivia) {
                    return Err(error(CssNumericConstructionErrorKind::Arity));
                }
                (NodeKind::TreeCounting(function), CssNumericType::NUMBER)
            } else {
                let function = CssMathFunction::parse(f.name())
                    .ok_or_else(|| error(CssNumericConstructionErrorKind::UnknownFunction))?;
                parse_function(function, f.values(), parser, c)?
            }
        }
        _ => return Err(error(CssNumericConstructionErrorKind::MalformedExpression)),
    };
    if let Some(h) = root.hint()
        && ty.is(CssNumericDimension::Percentage)
    {
        ty = ty.hinted(h).unwrap()
    }
    let (syntax, closing) = match c.view() {
        CssComponentValueRef::Function(f) => (
            direct_syntax(f.values().items()),
            Some(f.closing_origin().clone()),
        ),
        CssComponentValueRef::Block(b) => (Vec::new(), Some(b.closing_origin().clone())),
        _ => (Vec::new(), None),
    };
    Ok(CssCalculationExpression {
        admits_pure_context: kind.admits_pure_context(),
        kind,
        ty,
        origin: c.origin().clone(),
        components: None,
        syntax,
        closing,
    })
}
/// Intrinsic numeric leaves shared by owned projection and borrowed checks.
/// Relative/profile channels are qualified by their existing contextual owner.
fn intrinsic_token_type(component: &CssComponentValue) -> Option<Result<CssNumericType>> {
    Some(match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(_)) => Ok(CssNumericType::NUMBER),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
            Ok(CssNumericType::dimension(CssNumericDimension::Percentage))
        }
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) => {
            unit_dimension(unit)
                .map(CssNumericType::dimension)
                .ok_or_else(|| {
                    CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::InvalidArgumentType,
                        Some(component),
                    )
                })
        }
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name))
            if CssNumericConstant::parse(name).is_some() =>
        {
            Ok(CssNumericType::NUMBER)
        }
        _ => return None,
    })
}
fn parse_function<'a>(
    function: CssMathFunction,
    values: &'a CssComponentValues,
    parser: &mut NumericParser<'a>,
    component: &'a CssComponentValue,
) -> Result<(NodeKind, CssNumericType)> {
    let value = ordinary_expression::parse(values.items(), parser, Some((function, component)))?;
    Ok((value.kind, value.ty))
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AdmissionPolicy {
    Strict,
    RecoveredSyntax,
}

fn validate_components(
    values: &[CssComponentValue],
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
) -> Result<()> {
    CssComponentValues::validate_items_with_limits(values, limits)
        .map_err(CssNumericConstructionError::component)?;
    if policy == AdmissionPolicy::Strict {
        let mut pending: Vec<_> = values.iter().collect();
        while let Some(component) = pending.pop() {
            let nested = match component.view() {
                CssComponentValueRef::Function(function) => {
                    Some((function.values(), function.closing_origin()))
                }
                CssComponentValueRef::Block(block) => {
                    Some((block.values(), block.closing_origin()))
                }
                _ => None,
            };
            if let Some((children, closing)) = nested {
                if matches!(closing, CssValueOrigin::ImplicitClosure { .. }) {
                    return Err(CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::RecoveredComponent,
                        Some(component),
                    ));
                }
                pending.extend(children.items());
            }
        }
    }
    Ok(())
}
pub(crate) fn construct(
    values: CssComponentValues,
    root: CalculationRoot,
    limits: CssComponentValueLimits,
) -> Result<CssCalculationExpression> {
    construct_with_policy(values, root, limits, AdmissionPolicy::Strict)
}
/// Pure checked admission for a consumer that only needs validity.
pub(crate) fn check_component(component: &CssComponentValue, root: CalculationRoot) -> Result<()> {
    check_component_with_policy(component, root, AdmissionPolicy::Strict)
}
fn check_component_with_policy(
    component: &CssComponentValue,
    root: CalculationRoot,
    policy: AdmissionPolicy,
) -> Result<()> {
    let values = std::slice::from_ref(component);
    let limits = CssComponentValueLimits::default();
    prepare_calculation(values, root, limits, policy, NumericAdmissionContext::Pure).map(drop)
}
fn construct_with_policy<'input>(
    values: impl Into<CalculationComponents<'input>>,
    root: CalculationRoot,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
) -> Result<CssCalculationExpression> {
    construct_with_context(values, root, limits, policy, NumericAdmissionContext::Pure)
}

/// Admits container-size numeric syntax after the owning query boundary has
/// checked the component graph, including trusted parsed EOF closures.
pub(crate) fn construct_container(
    values: CssComponentValues,
    root: CalculationRoot,
    limits: CssComponentValueLimits,
) -> Result<CssCalculationExpression> {
    construct_with_context(
        values,
        root,
        limits,
        AdmissionPolicy::RecoveredSyntax,
        NumericAdmissionContext::ContainerSize,
    )
}

fn construct_with_context<'input>(
    values: impl Into<CalculationComponents<'input>>,
    root: CalculationRoot,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
    context: NumericAdmissionContext,
) -> Result<CssCalculationExpression> {
    let values = values.into();
    let mut expression = prepare_calculation(values.items(), root, limits, policy, context)?;
    expression.components = Some(values.retain(limits)?);
    Ok(expression)
}

fn prepare_calculation(
    values: &[CssComponentValue],
    root: CalculationRoot,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
    context: NumericAdmissionContext,
) -> Result<CssCalculationExpression> {
    validate_components(values, limits, policy)?;
    construct_validated(values, root, limits, context)
}

fn construct_validated(
    values: &[CssComponentValue],
    root: CalculationRoot,
    limits: CssComponentValueLimits,
    context: NumericAdmissionContext,
) -> Result<CssCalculationExpression> {
    let mut significant = values.iter().filter(|c| !trivia(c));
    let c = significant.next().ok_or_else(|| {
        CssNumericConstructionError::at(CssNumericConstructionErrorKind::EmptyValue, None)
    })?;
    if let Some(extra) = significant.next() {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::MultipleValues,
            Some(extra),
        ));
    }
    let literal = matches!(
        c.view(),
        CssComponentValueRef::Token(
            CssValueTokenRef::Number(_)
                | CssValueTokenRef::Dimension { .. }
                | CssValueTokenRef::Percentage(_)
        )
    );
    if !literal && !matches!(c.view(), CssComponentValueRef::Function(_)) {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(c),
        )
        .with_path(
            values
                .iter()
                .position(|item| std::ptr::eq(item, c))
                .map(|index| vec![index].into_boxed_slice()),
        ));
    }
    let root_index = values
        .iter()
        .position(|value| std::ptr::eq(value, c))
        .expect("root component belongs to input");
    let mut result = parse_tree(c, root, context, root_index)?;
    if root == CalculationRoot::Integer
        && literal
        && !matches!(c.view(),CssComponentValueRef::Token(CssValueTokenRef::Number(n))if n.kind()==CssNumericTokenKind::Integer)
    {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(c),
        ));
    }
    if matches!(
        root,
        CalculationRoot::Length | CalculationRoot::LengthPercentage
    ) && matches!(c.view(),CssComponentValueRef::Token(CssValueTokenRef::Number(n))if exact_zero(n))
    {
        result.ty = CssNumericType::dimension(CssNumericDimension::Length)
    }
    if !root.accepts(result.ty) {
        // Values 4 §10.9: where percentages resolve against no other type,
        // a sum that needed a percent hint fails "add two types" itself.
        let kind = if root == CalculationRoot::Percentage && result.ty.percent_hint().is_some() {
            CssNumericConstructionErrorKind::IncompatibleTypes
        } else {
            CssNumericConstructionErrorKind::RootDomainMismatch
        };
        return Err(CssNumericConstructionError::at(kind, Some(c))
            .with_path(Some(vec![root_index].into_boxed_slice())));
    }
    let canonical_bytes = result.canonical_len(usize::MAX).ok_or_else(|| {
        CssNumericConstructionError::component(CssComponentValueError::new(
            CssComponentValueErrorKind::CapacityOverflow,
            c.origin().clone(),
        ))
    })?;
    if canonical_bytes > limits.max_css_bytes() {
        return Err(CssNumericConstructionError::component(
            CssComponentValueError::new(CssComponentValueErrorKind::ByteLimit, c.origin().clone()),
        ));
    }
    Ok(result)
}

#[derive(Debug)]
pub(crate) enum GridTrackMath {
    Flex(CssCalculationExpression),
    LengthPercentage(CssCalculationExpression),
}

#[cfg(test)]
mod grid_track_math_tests {
    use super::*;

    fn selected(source: &str) -> Result<GridTrackMath> {
        construct_grid_track_math(
            crate::parse_component_values(source).unwrap(),
            CssComponentValueLimits::default(),
            AdmissionPolicy::Strict,
        )
    }

    #[test]
    fn flex_and_percentage_hinted_length_are_distinct_contextual_results() {
        for source in ["calc(2 * 1fr)", "calc(1fr * (1% / 1%))"] {
            let GridTrackMath::Flex(value) = selected(source).unwrap() else {
                panic!("flex result: {source}");
            };
            assert_eq!(value.result_type(), CssCalculationType::Flex);
            assert_eq!(value.ty.percent_hint(), None);
        }
        let GridTrackMath::LengthPercentage(value) = selected("calc(10px + 5%)").unwrap() else {
            panic!("length-percentage result");
        };
        assert_eq!(value.ty.percent_hint(), Some(CssNumericDimension::Length));
    }

    #[test]
    fn failed_alternatives_report_typed_error_with_origin_and_path() {
        for (source, kind, path, offset) in [
            (
                "calc(1fr + 10px)",
                CssNumericConstructionErrorKind::IncompatibleTypes,
                &[0, 4][..],
                11,
            ),
            (
                "calc(1fr + 1%)",
                CssNumericConstructionErrorKind::IncompatibleTypes,
                &[0, 4][..],
                11,
            ),
            (
                "round(1fr)",
                CssNumericConstructionErrorKind::InvalidArgumentType,
                &[0][..],
                0,
            ),
            (
                "calc((1% + 1px) / 1px)",
                CssNumericConstructionErrorKind::RootDomainMismatch,
                &[0][..],
                0,
            ),
            (
                "calc(1fr * sin(1% / 1px))",
                CssNumericConstructionErrorKind::InvalidArgumentType,
                &[0, 4][..],
                11,
            ),
        ] {
            let error = selected(source).unwrap_err();
            assert_eq!(error.kind(), &kind, "{source}");
            let Some(CssValueOrigin::Parsed(origin)) = error.origin() else {
                panic!("parsed diagnostic origin: {source}: {error:?}");
            };
            assert_eq!(
                origin.span().start().byte_offset().value(),
                offset,
                "{source}"
            );
            assert_eq!(error.path.as_deref(), Some(path), "{source}: {error:?}");
        }
    }

    #[test]
    fn terminal_grid_errors_do_not_fall_through_to_another_numeric_root() {
        for (source, kind) in [
            (
                "unknown(1fr)",
                CssNumericConstructionErrorKind::UnknownFunction,
            ),
            (
                "calc(1fr + )",
                CssNumericConstructionErrorKind::MalformedExpression,
            ),
            (
                "calc(1fr",
                CssNumericConstructionErrorKind::RecoveredComponent,
            ),
        ] {
            let error = selected(source).unwrap_err();
            assert_eq!(error.kind(), &kind, "{source}: {error:?}");
            assert!(
                matches!(
                    error.origin(),
                    Some(CssValueOrigin::Parsed(_) | CssValueOrigin::ImplicitClosure { .. })
                ),
                "{source}: {error:?}"
            );
        }

        let values = crate::parse_component_values("calc(1fr)").unwrap();
        let error = construct_grid_track_math(
            values,
            CssComponentValueLimits::try_new(256, 1, 100).unwrap(),
            AdmissionPolicy::Strict,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::ResourceLimit
        );

        let programmatic = CssComponentValues::try_new(
            vec![CssComponentValue::try_dimension("1", "s").unwrap()],
            Default::default(),
        )
        .unwrap();
        let error = construct_grid_track_math(
            programmatic,
            CssComponentValueLimits::default(),
            AdmissionPolicy::Strict,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert_eq!(error.path.as_deref(), Some(&[0][..]));
        assert!(matches!(error.origin(), Some(CssValueOrigin::Programmatic)));
    }
}

fn construct_grid_track_math<'input>(
    values: impl Into<CalculationComponents<'input>>,
    limits: CssComponentValueLimits,
    policy: AdmissionPolicy,
) -> Result<GridTrackMath> {
    let values = values.into();
    validate_components(values.items(), limits, policy)?;
    let flex = construct_validated(
        values.items(),
        CalculationRoot::Flex,
        limits,
        NumericAdmissionContext::Pure,
    );
    let mut selected = match flex {
        Ok(expression) => Ok(GridTrackMath::Flex(expression)),
        Err(flex_error) => {
            if !matches!(
                flex_error.kind(),
                CssNumericConstructionErrorKind::RootDomainMismatch
                    | CssNumericConstructionErrorKind::IncompatibleTypes
                    | CssNumericConstructionErrorKind::InvalidArgumentType
            ) {
                return Err(flex_error);
            }
            match construct_validated(
                values.items(),
                CalculationRoot::LengthPercentage,
                limits,
                NumericAdmissionContext::Pure,
            ) {
                Ok(expression) => Ok(GridTrackMath::LengthPercentage(expression)),
                Err(length_error) => {
                    if matches!(
                        length_error.kind(),
                        CssNumericConstructionErrorKind::RootDomainMismatch
                    ) && !matches!(
                        flex_error.kind(),
                        CssNumericConstructionErrorKind::RootDomainMismatch
                    ) {
                        Err(flex_error)
                    } else {
                        Err(length_error)
                    }
                }
            }
        }
    }?;
    match &mut selected {
        GridTrackMath::Flex(expression) | GridTrackMath::LengthPercentage(expression) => {
            expression.components = Some(values.retain(limits)?);
        }
    }
    Ok(selected)
}

/// Resource accounting for assembling already admitted roots. No child graph is
/// cloned until the complete iterator has passed these aggregate checks.
struct SumAssemblyBudget {
    limits: CssComponentValueLimits,
    components: usize,
    lexical: crate::component_values::CssCanonicalBuilder,
    canonical_bytes: usize,
}
impl SumAssemblyBudget {
    fn error(
        kind: CssComponentValueErrorKind,
        origin: &CssValueOrigin,
    ) -> CssNumericConstructionError {
        CssNumericConstructionError::component(CssComponentValueError::new(kind, origin.clone()))
    }
    fn add(
        total: &mut usize,
        amount: usize,
        limit: usize,
        kind: CssComponentValueErrorKind,
        origin: &CssValueOrigin,
    ) -> Result<()> {
        *total = total
            .checked_add(amount)
            .ok_or_else(|| Self::error(CssComponentValueErrorKind::CapacityOverflow, origin))?;
        if *total > limit {
            return Err(Self::error(kind, origin));
        }
        Ok(())
    }
    fn new(limits: CssComponentValueLimits) -> Result<Self> {
        let mut budget = Self {
            limits,
            components: 0,
            lexical: crate::component_values::CssCanonicalBuilder::counting(limits.max_css_bytes()),
            canonical_bytes: 0,
        };
        let origin = CssValueOrigin::Programmatic;
        Self::add(
            &mut budget.components,
            1,
            limits.max_components(),
            CssComponentValueErrorKind::ComponentLimit,
            &origin,
        )?;
        if limits.max_nesting_depth() == 0 {
            return Err(Self::error(
                CssComponentValueErrorKind::NestingLimit,
                &origin,
            ));
        }
        // Canonical numeric output includes both wrapper tokens. Lexical output
        // carries shared boundary state, including any inserted separators.
        budget.add_canonical_bytes(6, &origin)?;
        budget
            .lexical
            .push_grammar(
                crate::component_values::CssCanonicalToken::Function("calc"),
                &origin,
            )
            .map_err(CssNumericConstructionError::component)?;
        budget.reserve_closing()?;
        Ok(budget)
    }
    fn add_canonical_bytes(&mut self, canonical: usize, origin: &CssValueOrigin) -> Result<()> {
        Self::add(
            &mut self.canonical_bytes,
            canonical,
            self.limits.max_css_bytes(),
            CssComponentValueErrorKind::ByteLimit,
            origin,
        )
    }
    fn reserve_closing(&self) -> Result<()> {
        let mut bytes = self.lexical.byte_len();
        Self::add(
            &mut bytes,
            1,
            self.limits.max_css_bytes(),
            CssComponentValueErrorKind::ByteLimit,
            &CssValueOrigin::Programmatic,
        )
    }
    fn separator(&mut self, operator: CssCalculationSumOperator) -> Result<()> {
        use crate::component_values::CssCanonicalToken;
        let origin = CssValueOrigin::Programmatic;
        Self::add(
            &mut self.components,
            3,
            self.limits.max_components(),
            CssComponentValueErrorKind::ComponentLimit,
            &origin,
        )?;
        for token in [
            CssCanonicalToken::Whitespace,
            CssCanonicalToken::Delim(match operator {
                CssCalculationSumOperator::Add => '+',
                CssCalculationSumOperator::Subtract => '-',
            }),
            CssCanonicalToken::Whitespace,
        ] {
            self.lexical
                .push_grammar(token, &origin)
                .map_err(CssNumericConstructionError::component)?;
        }
        self.reserve_closing()?;
        self.add_canonical_bytes(3, &origin)
    }
    fn finish(mut self) -> Result<()> {
        self.lexical
            .push_grammar(
                crate::component_values::CssCanonicalToken::CloseParen,
                &CssValueOrigin::Programmatic,
            )
            .map_err(CssNumericConstructionError::component)
    }
    fn operand(&mut self, value: &CssCalculation<CssLengthPercentage>) -> Result<()> {
        // Iterator frames keep traversal storage proportional to nesting depth.
        let mut pending = vec![(value.components().items().iter(), 1u32)];
        while let Some((items, depth)) = pending.last_mut() {
            let Some(component) = items.next() else {
                pending.pop();
                continue;
            };
            Self::add(
                &mut self.components,
                1,
                self.limits.max_components(),
                CssComponentValueErrorKind::ComponentLimit,
                component.origin(),
            )?;
            let children = match component.view() {
                CssComponentValueRef::Function(function) => Some(function.values()),
                CssComponentValueRef::Block(block) => Some(block.values()),
                _ => None,
            };
            if let Some(children) = children {
                if *depth >= self.limits.max_nesting_depth() {
                    return Err(Self::error(
                        CssComponentValueErrorKind::NestingLimit,
                        component.origin(),
                    ));
                }
                let child_depth = *depth + 1;
                pending.push((children.items().iter(), child_depth));
            }
        }
        self.lexical
            .push_components(value.components().items())
            .map_err(CssNumericConstructionError::component)?;
        self.reserve_closing()?;
        let canonical = value.expression.canonical_len(usize::MAX).ok_or_else(|| {
            Self::error(CssComponentValueErrorKind::CapacityOverflow, value.origin())
        })?;
        self.add_canonical_bytes(canonical, value.origin())
    }
}

impl CssCalculation<CssLengthPercentage> {
    /// Assembles a symbolic sum while preserving every operand's original components.
    ///
    /// The first operand has no binary operator. Even a single operand receives
    /// a programmatic `calc()` wrapper. Root-only unitless zero is rechecked as
    /// an arithmetic operand and can fail ordinary numeric type admission.
    ///
    /// Assembles a sum under aggregate component, nesting and output-byte limits.
    ///
    /// Limits include inserted punctuation and whitespace. Admission stops
    /// consuming operands when a limit is exceeded, before cloning child graphs.
    /// Unlimited count and byte limits do not bound an arbitrary input iterator.
    /// Already admitted recovery origins remain intact; the public component
    /// constructor continues to reject recovered input.
    pub fn try_sum(
        first: Self,
        rest: impl IntoIterator<Item = (CssCalculationSumOperator, Self)>,
        limits: CssComponentValueLimits,
    ) -> Result<Self> {
        let mut budget = SumAssemblyBudget::new(limits)?;
        budget.operand(&first)?;
        let mut operands = vec![(None, first)];
        for (operator, value) in rest {
            budget.separator(operator)?;
            budget.operand(&value)?;
            operands.push((Some(operator), value));
        }
        budget.finish()?;
        let mut components = Vec::new();
        for (operator, value) in &operands {
            if let Some(operator) = operator {
                for token in [
                    " ",
                    match operator {
                        CssCalculationSumOperator::Add => "+",
                        CssCalculationSumOperator::Subtract => "-",
                    },
                    " ",
                ] {
                    components.push(
                        CssComponentValue::try_token(token)
                            .map_err(CssNumericConstructionError::component)?,
                    );
                }
            }
            components.extend(value.components().items().iter().cloned());
        }
        let children = CssComponentValues::try_new(components, Default::default())
            .map_err(CssNumericConstructionError::component)?;
        let function = CssComponentValue::try_function("calc", children)
            .map_err(CssNumericConstructionError::component)?;
        let values = CssComponentValues::try_new(vec![function], Default::default())
            .map_err(CssNumericConstructionError::component)?;
        construct_with_policy(
            values,
            CalculationRoot::LengthPercentage,
            limits,
            AdmissionPolicy::RecoveredSyntax,
        )
        .map(Self::from_expression)
    }
}

fn escaped_profile_name(name: &crate::CssColorProfileComponentName) -> String {
    let mut text = String::new();
    cssparser::serialize_identifier(name.as_str(), &mut text)
        .expect("String writing is infallible");
    text
}

/// A symbolic profile descriptor component in a checked numeric expression.
#[derive(Clone, Copy, Debug)]
pub struct CssCalculationProfileChannelRef<'a> {
    node: &'a CssCalculationExpression,
}
impl<'a> CssCalculationProfileChannelRef<'a> {
    pub fn name(self) -> &'a crate::CssColorProfileComponentName {
        let NodeKind::ProfileChannel(name) = &self.node.kind else {
            unreachable!("checked profile reference")
        };
        name
    }
    pub fn origin(self) -> &'a CssValueOrigin {
        &self.node.origin
    }
    pub fn numeric_type(self) -> CssNumericType {
        self.node.ty
    }
}

/// Builds once in the caller's cumulative context and defers numeric emission.
pub(crate) fn prepare_calculation_specified<'a, T: CssNumericRoot>(
    calculation: &'a CssCalculation<T>,
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> std::result::Result<DeferredNumericProjection<'a>, crate::CssSerializationError> {
    prepare_specified(&calculation.expression, context)
}

#[derive(Clone, Copy)]
pub(crate) enum ColorCalculationRef<'a> {
    Number(&'a CssCalculation<CssNumber>),
    HintedNumber(&'a CssCalculation<CssHintedNumber>),
    Percentage(&'a CssCalculation<CssPercentage>),
    Angle(&'a CssCalculation<CssAngle>),
}
impl ColorCalculationRef<'_> {
    fn expression(&self) -> &CssCalculationExpression {
        match self {
            Self::Number(value) => &value.expression,
            Self::HintedNumber(value) => &value.expression,
            Self::Percentage(value) => &value.expression,
            Self::Angle(value) => &value.expression,
        }
    }
}
/// Captures retained color coefficients using generic specified text after the
/// caller selects the slot scale. Origins explicitly select identity to retain
/// their specified dimensions; calculated alpha and weights remain authored.
pub(crate) fn capture_retained_color_calculation(
    calculation: ColorCalculationRef<'_>,
    scale: NumericProjectionScale,
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> std::result::Result<(String, NumericProjectionOutcome), crate::CssSerializationError> {
    capture_specified_scaled(calculation.expression(), scale, context)
}

/// Captures an ordinary non-origin, non-alpha component with its slot's scale.
/// Contextual coefficients use generic text; numeric color scratch is retained.
pub(crate) fn capture_color_component_calculation(
    calculation: ColorCalculationRef<'_>,
    scale: NumericProjectionScale,
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> std::result::Result<(String, NumericProjectionOutcome), crate::CssSerializationError> {
    projection::capture_color_component_specified_scaled(calculation.expression(), scale, context)
}

/// Private numeric-emission control for the protected non-alpha policy.
#[cfg(test)]
fn capture_numeric_color_calculation(
    calculation: ColorCalculationRef<'_>,
    scale: NumericProjectionScale,
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> std::result::Result<(String, NumericProjectionOutcome), crate::CssSerializationError> {
    projection::capture_color_specified_scaled(calculation.expression(), scale, context)
}

/// Temporary owned text, with pre-rounding components only for selected math
/// captures. This is serialization state, never stored authored equality.
pub(crate) enum CapturedNumericComponent {
    Plain(String),
    Projected(NumericComparisonCapture),
}
impl CapturedNumericComponent {
    pub(crate) fn as_css(&self) -> &str {
        match self {
            Self::Plain(css) => css,
            Self::Projected(capture) => capture.as_css(),
        }
    }
    pub(crate) fn capture_calculation(
        expression: &CssCalculationExpression,
        context: &mut crate::specified_serialization::SpecifiedSerializationContext,
    ) -> std::result::Result<Self, crate::CssSerializationError> {
        capture_specified_for_comparison(expression, context)
            .map(|(capture, _)| Self::Projected(capture))
    }
    fn same_math_or_text(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Projected(left), Self::Projected(right)) => {
                left.same_projected_components(right)
            }
            _ => self.as_css() == other.as_css(),
        }
    }
}
pub(crate) fn numeric_components_equal(
    left: Option<&CssComponentValue>,
    right: Option<&CssComponentValue>,
    left_capture: &CapturedNumericComponent,
    right_capture: &CapturedNumericComponent,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => crate::specified_numeric::ordinary_literal_equal(left, right),
        _ => left_capture.same_math_or_text(right_capture),
    }
}
pub(crate) fn length_components_equal(
    left: Option<&CssComponentValue>,
    right: Option<&CssComponentValue>,
    left_capture: &CapturedNumericComponent,
    right_capture: &CapturedNumericComponent,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            crate::specified_numeric::ordinary_length_literal_equal(left, right)
        }
        _ => left_capture.same_math_or_text(right_capture),
    }
}

#[cfg(test)]
mod color_component_calculation_tests {
    use super::*;
    use crate::{
        CssSerializationLimits as Limits, CssSpecifiedValueSerializationErrorKind as Kind,
        specified_serialization::SpecifiedSerializationContext as Context,
    };

    fn components(source: &str) -> CssComponentValues {
        crate::parse_component_values(source).unwrap()
    }

    fn assert_capture(
        calculation: ColorCalculationRef<'_>,
        scale: NumericProjectionScale,
        expected: &str,
        contextual: bool,
    ) -> NumericProjectionOutcome {
        let mut component_context = Context::new(Limits::default());
        let mut color_context = Context::new(Limits::default());
        let (text, component) =
            capture_color_component_calculation(calculation, scale, &mut component_context)
                .unwrap();
        let (_, color) =
            capture_numeric_color_calculation(calculation, scale, &mut color_context).unwrap();
        assert_eq!(text, expected);
        assert_eq!(component.context_dependent, contextual);
        assert_eq!(component.context_dependent, color.context_dependent);
        assert_eq!(
            component.scalar_value.map(f64::to_bits),
            color.scalar_value.map(f64::to_bits)
        );
        component
    }

    #[test]
    fn component_capture_selects_from_math_outcome_and_preserves_scalar_bits() {
        for (source, text, contextual, scalar) in [
            (
                "calc(1 / 128 + 1em / 1px)",
                "calc(0.007813 + (1em / 1px))",
                true,
                None,
            ),
            (
                "calc(1 / 128 + 1em / 1em)",
                "calc(0.007813 + (1em / 1em))",
                true,
                None,
            ),
            (
                "calc(1 / 128 + 1in / 96px)",
                "calc(1.0078125)",
                false,
                Some(1.0078125_f64.to_bits()),
            ),
            (
                "calc(1 / 128)",
                "calc(0.0078125)",
                false,
                Some((1.0_f64 / 128.0).to_bits()),
            ),
            ("calc(0 * -1)", "calc(0)", false, Some((-0.0_f64).to_bits())),
            (
                "calc(1 / (0 * -1))",
                "calc(-infinity)",
                false,
                Some(f64::NEG_INFINITY.to_bits()),
            ),
            (
                "calc(0 * -1 + 1em / 1px)",
                "calc((0 * -1) + (1em / 1px))",
                true,
                None,
            ),
        ] {
            let value = CssCalculation::<CssNumber>::try_from_components(
                components(source),
                Default::default(),
            )
            .unwrap();
            let before = value.clone();
            let outcome = assert_capture(
                ColorCalculationRef::Number(&value),
                NumericProjectionScale::Identity,
                text,
                contextual,
            );
            assert_eq!(outcome.scalar_value.map(f64::to_bits), scalar);
            assert_eq!(value, before);
        }
        let value = CssCalculation::<CssNumber>::try_from_components(
            components("calc(NaN + 1em / 1px)"),
            Default::default(),
        )
        .unwrap();
        let outcome = assert_capture(
            ColorCalculationRef::Number(&value),
            NumericProjectionScale::Identity,
            "calc(NaN)",
            false,
        );
        assert!(outcome.scalar_value.unwrap().is_nan());
    }

    #[test]
    fn component_capture_retains_scale_dimensions_and_canonical_angles() {
        let number = CssCalculation::<CssNumber>::try_from_components(
            components("calc(.0078125 * 1em / 1px)"),
            Default::default(),
        )
        .unwrap();
        assert_capture(
            ColorCalculationRef::Number(&number),
            NumericProjectionScale::NumberToPercentage {
                numerator: 1,
                denominator: 1,
            },
            "calc(0.007813 * 1% * 1em / 1px)",
            true,
        );
        let percentage = CssCalculation::<CssPercentage>::try_from_components(
            components("calc(.0078125% * 1em / 1px)"),
            Default::default(),
        )
        .unwrap();
        assert_capture(
            ColorCalculationRef::Percentage(&percentage),
            NumericProjectionScale::PercentageToNumber {
                numerator: 1,
                denominator: 100,
            },
            "calc(0.007813% * 1em / 1px / 100%)",
            true,
        );
        let angle = CssCalculation::<CssAngle>::try_from_components(
            components("calc(.0000001turn + 1deg * 1em / 1px)"),
            Default::default(),
        )
        .unwrap();
        assert_capture(
            ColorCalculationRef::Angle(&angle),
            NumericProjectionScale::Identity,
            "calc(0.000036deg + (1deg * 1em / 1px))",
            true,
        );
    }

    #[test]
    fn component_capture_uses_rounded_contextual_scratch_and_shared_work() {
        let value = CssCalculation::<CssNumber>::try_from_components(
            components("calc(5e-324 * 1em / 1px)"),
            Default::default(),
        )
        .unwrap();
        let calculation = ColorCalculationRef::Number(&value);
        let scale = NumericProjectionScale::Number {
            numerator: 1,
            denominator: 1,
        };
        // Wrapper, product and three leaves give five inputs. Three leaves,
        // inverse and product give five projections; the single Number is
        // reused. The scale's factor, merged Number and product add three more.
        let mut context = Context::new(Limits::new(5, 8, 19));
        let (text, outcome) =
            capture_color_component_calculation(calculation, scale, &mut context).unwrap();
        assert_eq!(text, "calc(0 * 1em / 1px)");
        assert!(outcome.context_dependent);
        assert_eq!(outcome.scalar_value, None);
        assert_eq!(context.remaining_bytes(), 19);
        assert_eq!(
            context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        for (inputs, projections, bytes, kind) in [
            (4, 8, 19, Kind::InputNodeLimit),
            (5, 7, 19, Kind::ProjectionNodeLimit),
            (5, 8, 18, Kind::ByteLimit),
        ] {
            let mut context = Context::new(Limits::new(inputs, projections, bytes));
            assert_eq!(
                capture_color_component_calculation(calculation, scale, &mut context)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        let mut context = Context::new(Limits::new(5, 8, 19));
        assert_eq!(
            capture_numeric_color_calculation(calculation, scale, &mut context)
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
    }
}

#[cfg(test)]
mod origin_color_calculation_tests {
    use super::*;
    use crate::{
        CssSerializationLimits as Limits, CssSpecifiedValueSerializationErrorKind as Kind,
        specified_serialization::SpecifiedSerializationContext as Context,
    };

    fn components(source: &str) -> CssComponentValues {
        crate::parse_component_values(source).unwrap()
    }

    fn assert_capture(calculation: ColorCalculationRef<'_>, expected: &str) {
        let mut origin_context = Context::new(Limits::default());
        let mut standalone_context = Context::new(Limits::default());
        let (text, origin) = capture_retained_color_calculation(
            calculation,
            NumericProjectionScale::Identity,
            &mut origin_context,
        )
        .unwrap();
        let (_, standalone) = capture_numeric_color_calculation(
            calculation,
            NumericProjectionScale::Identity,
            &mut standalone_context,
        )
        .unwrap();
        assert_eq!(text, expected);
        assert_eq!(origin.context_dependent, standalone.context_dependent);
        assert_eq!(
            origin.scalar_value.map(f64::to_bits),
            standalone.scalar_value.map(f64::to_bits)
        );
    }

    #[test]
    fn origin_capture_keeps_math_outcomes_and_specified_dimensions() {
        for (source, text) in [
            ("calc(1 / 128)", "calc(0.007813)"),
            ("calc(1em / 1px + 1 / 128)", "calc(0.007813 + (1em / 1px))"),
            ("calc(infinity)", "calc(infinity)"),
            ("calc(NaN)", "calc(NaN)"),
            ("calc(0 * -1)", "calc(0)"),
            ("calc(1 / (0 * -1))", "calc(-infinity)"),
            ("calc(0 * -1 + 1em / 1px)", "calc((0 * -1) + (1em / 1px))"),
        ] {
            let value = CssCalculation::<CssNumber>::try_from_components(
                components(source),
                Default::default(),
            )
            .unwrap();
            let before = value.clone();
            assert_capture(ColorCalculationRef::Number(&value), text);
            assert_eq!(value, before);
        }
        let percentage = CssCalculation::<CssPercentage>::try_from_components(
            components("calc(.78125%)"),
            Default::default(),
        )
        .unwrap();
        assert_capture(
            ColorCalculationRef::Percentage(&percentage),
            "calc(0.78125%)",
        );
        let angle = CssCalculation::<CssAngle>::try_from_components(
            components("calc(.0000001turn)"),
            Default::default(),
        )
        .unwrap();
        assert_capture(ColorCalculationRef::Angle(&angle), "calc(0.000036deg)");
    }

    #[test]
    fn origin_capture_uses_rounded_scratch_without_changing_standalone_scratch() {
        let value = CssCalculation::<CssNumber>::try_from_components(
            components("calc(1 / 128)"),
            Default::default(),
        )
        .unwrap();
        let calculation = ColorCalculationRef::Number(&value);
        let mut context = Context::new(Limits::new(usize::MAX, usize::MAX, 14));
        assert_eq!(
            capture_retained_color_calculation(
                calculation,
                NumericProjectionScale::Identity,
                &mut context
            )
            .unwrap()
            .0,
            "calc(0.007813)"
        );
        for (origin, bytes) in [(true, 13), (false, 14)] {
            let mut context = Context::new(Limits::new(usize::MAX, usize::MAX, bytes));
            let error = if origin {
                capture_retained_color_calculation(
                    calculation,
                    NumericProjectionScale::Identity,
                    &mut context,
                )
            } else {
                capture_numeric_color_calculation(
                    calculation,
                    NumericProjectionScale::Identity,
                    &mut context,
                )
            }
            .unwrap_err();
            assert_eq!(error.kind(), Kind::ByteLimit);
        }
        let mut context = Context::new(Limits::new(usize::MAX, usize::MAX, 15));
        assert_eq!(
            capture_numeric_color_calculation(
                calculation,
                NumericProjectionScale::Identity,
                &mut context
            )
            .unwrap()
            .0,
            "calc(0.0078125)"
        );
    }

    #[test]
    fn origin_capture_spends_the_callers_cumulative_work_and_remaining_bytes() {
        let value = CssCalculation::<CssNumber>::try_from_components(
            components("calc(1 / 128)"),
            Default::default(),
        )
        .unwrap();
        let calculation = ColorCalculationRef::Number(&value);
        // Inputs visit the wrapper, product and two leaves. Projection work
        // visits the two leaves, inverse and product: four of each per capture.
        for (inputs, projections, kind) in [
            (7, 8, Kind::InputNodeLimit),
            (8, 7, Kind::ProjectionNodeLimit),
        ] {
            let mut context = Context::new(Limits::new(inputs, projections, 14));
            capture_retained_color_calculation(
                calculation,
                NumericProjectionScale::Identity,
                &mut context,
            )
            .unwrap();
            assert_eq!(
                capture_retained_color_calculation(
                    calculation,
                    NumericProjectionScale::Identity,
                    &mut context
                )
                .unwrap_err()
                .kind(),
                kind
            );
        }
        let mut context = Context::new(Limits::new(8, 8, 15));
        let mut outer = String::new();
        context.append(&mut outer, "x").unwrap();
        assert_eq!(
            capture_retained_color_calculation(
                calculation,
                NumericProjectionScale::Identity,
                &mut context
            )
            .unwrap()
            .0,
            "calc(0.007813)"
        );
        assert_eq!(
            capture_retained_color_calculation(
                calculation,
                NumericProjectionScale::Identity,
                &mut context
            )
            .unwrap()
            .0,
            "calc(0.007813)"
        );
        context.append(&mut outer, "x").unwrap();
        assert_eq!(
            capture_retained_color_calculation(
                calculation,
                NumericProjectionScale::Identity,
                &mut context
            )
            .unwrap_err()
            .kind(),
            Kind::InputNodeLimit
        );
        let mut context = Context::new(Limits::new(usize::MAX, usize::MAX, 14));
        context.append(&mut String::new(), "x").unwrap();
        assert_eq!(
            capture_retained_color_calculation(
                calculation,
                NumericProjectionScale::Identity,
                &mut context
            )
            .unwrap_err()
            .kind(),
            Kind::ByteLimit
        );
    }
}

#[cfg(test)]
mod retained_color_calculation_tests {
    use super::*;
    use crate::{
        CssSerializationLimits as Limits, CssSpecifiedValueSerializationErrorKind as Kind,
        specified_serialization::SpecifiedSerializationContext as Context,
    };

    #[test]
    fn retained_percentage_scale_preserves_outcomes_and_authored_dimensions() {
        for (source, scale, expected, scalar) in [
            (
                "calc(.78125%)",
                NumericProjectionScale::Identity,
                "calc(0.78125%)",
                Some(0.78125_f64.to_bits()),
            ),
            (
                "calc(.78125%)",
                NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                },
                "calc(0.007813)",
                Some((1.0_f64 / 128.0).to_bits()),
            ),
            (
                "calc(.0078125% * 1em / 1px)",
                NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                },
                "calc(0.007813% * 1em / 1px / 100%)",
                None,
            ),
        ] {
            let value = CssCalculation::<CssPercentage>::try_from_components(
                crate::parse_component_values(source).unwrap(),
                Default::default(),
            )
            .unwrap();
            let before = value.clone();
            let calculation = ColorCalculationRef::Percentage(&value);
            let mut retained_context = Context::new(Limits::default());
            let mut numeric_context = Context::new(Limits::default());
            let (text, retained) =
                capture_retained_color_calculation(calculation, scale, &mut retained_context)
                    .unwrap();
            let (_, numeric) =
                capture_numeric_color_calculation(calculation, scale, &mut numeric_context)
                    .unwrap();
            assert_eq!(text, expected);
            assert_eq!(retained.scalar_value.map(f64::to_bits), scalar);
            assert_eq!(
                retained.scalar_value.map(f64::to_bits),
                numeric.scalar_value.map(f64::to_bits)
            );
            assert_eq!(retained.context_dependent, scalar.is_none());
            assert_eq!(retained.context_dependent, numeric.context_dependent);
            assert_eq!(value, before);
        }
    }

    #[test]
    fn retained_scaled_alpha_uses_shared_work_and_remaining_scratch() {
        let value = CssCalculation::<CssPercentage>::try_from_components(
            crate::parse_component_values("calc(.78125%)").unwrap(),
            Default::default(),
        )
        .unwrap();
        let calculation = ColorCalculationRef::Percentage(&value);
        let scale = NumericProjectionScale::PercentageToNumber {
            numerator: 1,
            denominator: 100,
        };
        // Wrapper + leaf: two inputs. Leaf plus scale percentage, inverse and
        // resolved product: four projections. Capture spends work, not final bytes.
        let mut context = Context::new(Limits::new(4, 8, 15));
        context.append(&mut String::new(), "x").unwrap();
        for _ in 0..2 {
            let (text, outcome) =
                capture_retained_color_calculation(calculation, scale, &mut context).unwrap();
            assert_eq!(text, "calc(0.007813)");
            assert_eq!(
                outcome.scalar_value.unwrap().to_bits(),
                (1.0_f64 / 128.0).to_bits()
            );
            assert_eq!(context.remaining_bytes(), 14);
        }
        assert_eq!(
            context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        for (inputs, projections, bytes, kind) in [
            (1, 4, 14, Kind::InputNodeLimit),
            (2, 3, 14, Kind::ProjectionNodeLimit),
            (2, 4, 13, Kind::ByteLimit),
        ] {
            let mut context = Context::new(Limits::new(inputs, projections, bytes));
            assert_eq!(
                capture_retained_color_calculation(calculation, scale, &mut context)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
    }
}

#[cfg(test)]
mod structural_comparison_tests {
    use super::*;

    fn checked(source: &str) -> CssCalculation<CssLengthPercentage> {
        CssCalculation::<CssLengthPercentage>::try_from_components(
            crate::parse_component_values(source).unwrap(),
            Default::default(),
        )
        .unwrap()
    }

    #[test]
    fn numeric_equality_ignores_source_coordinates_and_spelling_but_not_structure() {
        let value = checked("calc(1px + 2%)");
        let shifted = checked("  calc(1px + 2%)");
        assert_eq!(value, shifted, "numeric equality ignores provenance");
        assert_eq!(value, checked("CALC(1.0px + +2%)"));
        assert_eq!(
            value,
            checked("calc((1px + 2%))"),
            "redundant grouping is not a node (Values 4 parse a calculation)"
        );
        assert!(
            !value
                .expression
                .structural_eq(&checked("calc((1px + 2%))").expression),
            "the specified shortening identity keeps the grouping"
        );
        assert!(
            value.expression.structural_eq(&shifted.expression),
            "the specified shortening identity ignores provenance"
        );
        assert!(
            !value
                .expression
                .structural_eq(&checked("calc(1.0px + 2%)").expression),
            "the specified shortening identity keeps authored spelling"
        );

        for different in [
            "calc(1px + 2px)", // numeric type and unit
            "calc(1px - 2%)",  // operator
            "calc(2px + 2%)",  // exact numeric value
            "calc(2% + 1px)",  // operand order
        ] {
            assert_ne!(value, checked(different), "{different}");
        }
    }
}

#[cfg(test)]
mod specified_helpers_tests {
    use crate::{
        CssSerializationLimits as Limits, CssSpecifiedValueSerializationErrorKind as ErrorKind,
        specified_serialization::SpecifiedSerializationContext,
    };

    #[test]
    fn identifier_capture_uses_the_shared_escape_owner_and_remaining_byte_bound() {
        let context = SpecifiedSerializationContext::new(Limits::new(0, 0, 4));
        assert_eq!(
            crate::serialization_escaping::capture_identifier("a b", &context).unwrap(),
            r"a\ b"
        );

        let context = SpecifiedSerializationContext::new(Limits::new(0, 0, 3));
        assert_eq!(
            crate::serialization_escaping::capture_identifier("a b", &context)
                .unwrap_err()
                .kind(),
            ErrorKind::ByteLimit
        );
    }
}

#[cfg(test)]
mod media_helpers_tests {
    use super::*;

    #[test]
    fn generic_media_numbers_admit_named_dimensions_but_not_percentages_or_products() {
        for source in [
            "calc(1)",
            "calc(1px)",
            "calc(1deg)",
            "calc(1s)",
            "calc(1Hz)",
            "calc(1dppx)",
            "calc(1fr)",
        ] {
            let values = crate::parse_component_values(source).unwrap();
            let serialized = values.serialize_tokens(Default::default()).unwrap();
            let context = NumericInputContext::components(&values, &serialized);
            let expression = context
                .admit_with_limits(
                    values.clone(),
                    CalculationRoot::NamedDimensionOrNumber,
                    CssComponentValueLimits::default(),
                )
                .unwrap();
            assert_eq!(expression.components.as_ref(), Some(&values));
        }
        for (source, expected) in [
            (
                "calc(1%)",
                CssNumericConstructionErrorKind::RootDomainMismatch,
            ),
            (
                "calc(1px * 1px)",
                CssNumericConstructionErrorKind::InvalidArgumentType,
            ),
        ] {
            let values = crate::parse_component_values(source).unwrap();
            let error = construct(
                values,
                CalculationRoot::NamedDimensionOrNumber,
                CssComponentValueLimits::default(),
            )
            .unwrap_err();
            assert_eq!(error.kind(), &expected, "{source}");
        }
    }

    #[test]
    fn numeric_resource_errors_retain_the_concrete_component_cause() {
        let values = crate::parse_component_values("calc(1+ 2)").unwrap();
        // Limits precede grammar admission, including malformed mathematics.
        for (limits, kind) in [
            (
                CssComponentValueLimits::try_new(0, 100, 100).unwrap(),
                CssComponentValueErrorKind::NestingLimit,
            ),
            (
                CssComponentValueLimits::try_new(1, 1, 100).unwrap(),
                CssComponentValueErrorKind::ComponentLimit,
            ),
            (
                CssComponentValueLimits::try_new(1, 100, 1).unwrap(),
                CssComponentValueErrorKind::ByteLimit,
            ),
        ] {
            let error = construct(values.clone(), CalculationRoot::Number, limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNumericConstructionErrorKind::ResourceLimit
            );
            assert_eq!(error.component_error().unwrap().kind(), kind);
            assert_eq!(
                error.origin(),
                Some(error.component_error().unwrap().origin())
            );
        }
        let compact = crate::parse_component_values("calc(1*2)").unwrap();
        let error = construct(
            compact,
            CalculationRoot::Number,
            CssComponentValueLimits::try_new(1, 10, 9).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.component_error().unwrap().kind(),
            CssComponentValueErrorKind::ByteLimit
        );
    }
}

#[cfg(test)]
mod container_context_tests {
    use super::*;

    #[test]
    fn container_tree_functions_are_symbolic_numbers_with_distinct_views() {
        for (source, function, canonical) in [
            (
                "sibling-count()",
                CssTreeCountingFunction::SiblingCount,
                "sibling-count()",
            ),
            (
                "SIBLING-INDEX( /**/ )",
                CssTreeCountingFunction::SiblingIndex,
                "sibling-index()",
            ),
            (
                r"sibling-\69 ndex()",
                CssTreeCountingFunction::SiblingIndex,
                "sibling-index()",
            ),
        ] {
            let components = crate::parse_component_values(source).unwrap();
            let expression = construct_container(
                components.clone(),
                CalculationRoot::Number,
                CssComponentValueLimits::default(),
            )
            .unwrap();
            assert_eq!(expression.result_type(), CssCalculationType::Number);
            let CssCalculationExpressionRef::TreeCounting(view) = expression.as_ref() else {
                panic!("distinct contextual tree function view");
            };
            assert_eq!(view.function(), function);
            assert_eq!(view.numeric_type(), CssNumericType::NUMBER);
            assert_eq!(view.origin(), components.items()[0].origin());
            let cloned = expression.clone();
            assert_eq!(expression, cloned);
            let number = CssCalculation::<CssNumber>::from_expression(cloned);
            let serialized = number.serialize_tokens(Default::default()).unwrap();
            assert_eq!(serialized.as_css(), canonical);
            assert_eq!(
                serialized.value_origin_at(0),
                Some(components.items()[0].origin())
            );
            let CssComponentValueRef::Function(original) = components.items()[0].view() else {
                panic!("function component");
            };
            assert_eq!(
                serialized.value_origin_at(canonical.len() - 1),
                Some(original.closing_origin())
            );
            // The owning container context is required again when round-tripping.
            let roundtrip = construct_container(
                crate::parse_component_values(canonical).unwrap(),
                CalculationRoot::Number,
                CssComponentValueLimits::default(),
            )
            .unwrap();
            assert_eq!(
                CssCalculation::<CssNumber>::from_expression(roundtrip)
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                canonical
            );
            assert_eq!(
                CssCalculation::<CssNumber>::try_from_components(components, Default::default())
                    .unwrap_err()
                    .kind(),
                &CssNumericConstructionErrorKind::UnknownFunction
            );
        }
    }

    #[test]
    fn container_tree_functions_obey_length_algebra_and_pure_context_isolation() {
        for source in [
            "calc(sibling-count() * 1px)",
            "max(1px, calc(2em / sibling-index()))",
        ] {
            let components = crate::parse_component_values(source).unwrap();
            let expression = construct_container(
                components.clone(),
                CalculationRoot::Length,
                CssComponentValueLimits::default(),
            )
            .unwrap();
            assert_eq!(expression.result_type(), CssCalculationType::Length);
            assert_eq!(
                CssCalculation::<CssLength>::from_expression(expression)
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            assert_eq!(
                CssCalculation::<CssLength>::try_from_components(components, Default::default())
                    .unwrap_err()
                    .kind(),
                &CssNumericConstructionErrorKind::UnknownFunction
            );
        }
        for (source, expected) in [
            (
                "sibling-index()",
                CssNumericConstructionErrorKind::RootDomainMismatch,
            ),
            (
                "calc(sibling-count() + 1px)",
                CssNumericConstructionErrorKind::IncompatibleTypes,
            ),
            ("sibling-count(1)", CssNumericConstructionErrorKind::Arity),
            ("sibling-index(,)", CssNumericConstructionErrorKind::Arity),
            (
                "future-count()",
                CssNumericConstructionErrorKind::UnknownFunction,
            ),
        ] {
            let result = construct_container(
                crate::parse_component_values(source).unwrap(),
                CalculationRoot::Length,
                CssComponentValueLimits::default(),
            );
            assert_eq!(result.unwrap_err().kind(), &expected, "{source}");
        }
    }

    #[test]
    fn contextual_numeric_construction_preserves_closure_policy_and_limits() {
        let values = crate::parse_component_values("sibling-count(").unwrap();
        assert_eq!(
            construct_with_context(
                values.clone(),
                CalculationRoot::Number,
                CssComponentValueLimits::default(),
                AdmissionPolicy::Strict,
                NumericAdmissionContext::ContainerSize,
            )
            .unwrap_err()
            .kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
        let expression = construct_container(
            values.clone(),
            CalculationRoot::Number,
            CssComponentValueLimits::default(),
        )
        .unwrap();
        let CssComponentValueRef::Function(original) = values.items()[0].view() else {
            panic!("function")
        };
        let serialized = CssCalculation::<CssNumber>::from_expression(expression)
            .serialize_tokens(Default::default())
            .unwrap();
        assert_eq!(
            serialized.value_origin_at(14),
            Some(original.closing_origin())
        );
        for (limits, expected) in [
            (
                CssComponentValueLimits::try_new(0, 100, 100).unwrap(),
                CssComponentValueErrorKind::NestingLimit,
            ),
            (
                CssComponentValueLimits::try_new(10, 0, 100).unwrap(),
                CssComponentValueErrorKind::ComponentLimit,
            ),
            (
                CssComponentValueLimits::try_new(10, 100, 1).unwrap(),
                CssComponentValueErrorKind::ByteLimit,
            ),
        ] {
            let error = construct_container(
                crate::parse_component_values("sibling-count()").unwrap(),
                CalculationRoot::Number,
                limits,
            )
            .unwrap_err();
            assert_eq!(error.component_error().unwrap().kind(), expected);
        }
    }
}

fn exact_numeric_leaf_identity(
    a: &CssComponentValue,
    b: &CssComponentValue,
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> std::result::Result<bool, crate::CssSerializationError> {
    let coefficients = match (a.view(), b.view()) {
        (
            CssComponentValueRef::Token(CssValueTokenRef::Number(a)),
            CssComponentValueRef::Token(CssValueTokenRef::Number(b)),
        )
        | (
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(a)),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(b)),
        ) => Some((a, b)),
        (
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. }),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. }),
        ) => {
            return Ok(exact_dimension_identity(Some(a), Some(b), context)?.unwrap_or(false));
        }
        _ => None,
    };
    Ok(coefficients.is_some_and(|(a, b)| {
        crate::exact_decimal::LexicalDecimal::new(a.representation()).value_eq(
            &crate::exact_decimal::LexicalDecimal::new(b.representation()),
        )
    }))
}

pub(crate) fn exact_dimension_identity(
    a: Option<&CssComponentValue>,
    b: Option<&CssComponentValue>,
    context: &mut crate::specified_serialization::SpecifiedSerializationContext,
) -> std::result::Result<Option<bool>, crate::CssSerializationError> {
    let (Some(a), Some(b)) = (a, b) else {
        return Ok(None);
    };
    let (
        CssComponentValueRef::Token(CssValueTokenRef::Dimension {
            number: a,
            unit: au,
        }),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension {
            number: b,
            unit: bu,
        }),
    ) = (a.view(), b.view())
    else {
        return Ok(Some(false));
    };
    if unit_dimension(au) != unit_dimension(bu) {
        return Ok(Some(false));
    }
    let (a, b) = (a.representation(), b.representation());
    let (al, bl) = (
        crate::exact_decimal::LexicalDecimal::new(a),
        crate::exact_decimal::LexicalDecimal::new(b),
    );
    if au.eq_ignore_ascii_case(bu) {
        return Ok(Some(al.value_eq(&bl)));
    }
    // A finite nonzero decimal radian cannot equal a rational multiple of a
    // degree exactly. Angle units share exact zero; contextual units remain distinct.
    if unit_dimension(au) == Some(CssNumericDimension::Angle)
        && al.value_eq(&crate::exact_decimal::LexicalDecimal::new("0"))
        && bl.value_eq(&crate::exact_decimal::LexicalDecimal::new("0"))
    {
        return Ok(Some(true));
    }
    let (Some(af), Some(bf)) = (
        exact_absolute_unit_factor(au),
        exact_absolute_unit_factor(bu),
    ) else {
        return Ok(Some(false));
    };
    Ok(Some(al.value_eq_scaled(bl, af, bf, context)?))
}

// Values 4 §6.2/§7 fixes these rational ratios in the one unit registry.
// Relative units and nonzero radian conversions have no exact ratio.
fn exact_absolute_unit_factor(unit: &str) -> Option<crate::exact_decimal::ExactFactor> {
    CssNumericUnit::parse(unit).and_then(units::exact_factor)
}
