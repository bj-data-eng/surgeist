//! Checked symbolic `calc-size()` syntax from CSS Values 5 §10.
//!
//! Interpolation canonicalization and used layout values belong to downstream
//! consumers; this model retains the authored basis and symbolic `size` leaf.

use crate::{
    CssCalculationExpressionRef, CssComponentValue, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssNumericConstructionError, CssNumericConstructionErrorKind,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef,
    numeric::{
        CssCalculationExpression, parse_calc_size_sum, project_calc_size_sum_into,
        validate_calc_size_graph,
    },
    specified_serialization::SpecifiedSerializationContext,
};

type Result<T> = std::result::Result<T, CssNumericConstructionError>;
type SerializationResult<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

/// A keyword permitted as a calc-size basis. `none` is never a basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssIntrinsicSizeKeyword {
    Auto,
    Content,
    Stretch,
    Contain,
    MinContent,
    MaxContent,
    FitContent,
}
impl CssIntrinsicSizeKeyword {
    fn parse(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "auto" => Self::Auto,
            "content" => Self::Content,
            "stretch" => Self::Stretch,
            "contain" => Self::Contain,
            "min-content" => Self::MinContent,
            "max-content" => Self::MaxContent,
            "fit-content" => Self::FitContent,
            _ => return None,
        })
    }
    fn as_css(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Content => "content",
            Self::Stretch => "stretch",
            Self::Contain => "contain",
            Self::MinContent => "min-content",
            Self::MaxContent => "max-content",
            Self::FitContent => "fit-content",
        }
    }
}

#[derive(Clone, Debug)]
enum Basis {
    Keyword(CssIntrinsicSizeKeyword),
    Any,
    Nested(CssCalcSize),
    Sum(Box<CssCalculationExpression>),
}

/// Borrowed, checked calc-size basis.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum CssCalcSizeBasisRef<'a> {
    Keyword(CssIntrinsicSizeKeyword),
    Any,
    Nested(&'a CssCalcSize),
    Sum(CssCalculationExpressionRef<'a>),
}

/// One checked, symbolic calc-size function and its authored provenance.
#[derive(Clone, Debug)]
pub struct CssCalcSize {
    data: Box<CssCalcSizeData>,
}

#[derive(Clone, Debug)]
struct CssCalcSizeData {
    basis: Basis,
    basis_origin: CssValueOrigin,
    basis_path: Box<[usize]>,
    calculation: Box<CssCalculationExpression>,
    origin: CssValueOrigin,
}

impl PartialEq for CssCalcSize {
    fn eq(&self, other: &Self) -> bool {
        self.structural_eq(other)
    }
}
impl Eq for CssCalcSize {}

fn significant(items: &[CssComponentValue]) -> impl Iterator<Item = &CssComponentValue> {
    items.iter().filter(|component| {
        !matches!(
            component.view(),
            CssComponentValueRef::Comment(_)
                | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
        )
    })
}

fn parts(
    component: &CssComponentValue,
) -> Result<(&[CssComponentValue], &[CssComponentValue], usize)> {
    let CssComponentValueRef::Function(function) = component.view() else {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::RootDomainMismatch,
            Some(component),
        ));
    };
    if !function.name().eq_ignore_ascii_case("calc-size") {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::UnknownFunction,
            Some(component),
        ));
    }
    let items = function.values().items();
    let mut commas = items.iter().enumerate().filter(|(_, c)| {
        matches!(
            c.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Comma)
        )
    });
    let Some((comma, _)) = commas.next() else {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::Arity,
            Some(component),
        ));
    };
    if commas.next().is_some()
        || significant(&items[..comma]).next().is_none()
        || significant(&items[comma + 1..]).next().is_none()
    {
        return Err(CssNumericConstructionError::at(
            CssNumericConstructionErrorKind::Arity,
            Some(component),
        ));
    }
    Ok((&items[..comma], &items[comma + 1..], comma))
}

impl CssCalcSize {
    /// Compares checked basis and calculation structure without diagnostic
    /// origins or root-relative error paths.
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        let mut pending = vec![(self, other)];
        while let Some((left, right)) = pending.pop() {
            if !left.data.calculation.structural_eq(&right.data.calculation) {
                return false;
            }
            match (&left.data.basis, &right.data.basis) {
                (Basis::Keyword(a), Basis::Keyword(b)) if a == b => {}
                (Basis::Any, Basis::Any) => {}
                (Basis::Nested(a), Basis::Nested(b)) => pending.push((a, b)),
                (Basis::Sum(a), Basis::Sum(b)) if a.structural_eq(b) => {}
                _ => return false,
            }
        }
        true
    }

    /// The CSSOM inverse uses this owner's same calculation-before-basis order,
    /// admitting every actual calc-size pending slot before fallible reserve.
    pub(crate) fn specified_inverse_eq(
        &self,
        other: &Self,
        context: &mut SpecifiedSerializationContext,
    ) -> SerializationResult<bool> {
        context.charge_generated_projection(1)?;
        let mut pending = Vec::new();
        pending.try_reserve(1).map_err(|_| {
            CssSpecifiedValueSerializationError::new(
                CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
            )
        })?;
        pending.push((self, other));
        while let Some((left, right)) = pending.pop() {
            if !left
                .data
                .calculation
                .specified_inverse_eq(&right.data.calculation, context)?
            {
                return Ok(false);
            }
            match (&left.data.basis, &right.data.basis) {
                (Basis::Keyword(a), Basis::Keyword(b)) if a == b => {}
                (Basis::Any, Basis::Any) => {}
                (Basis::Nested(a), Basis::Nested(b)) => {
                    context.charge_generated_projection(1)?;
                    pending.try_reserve(1).map_err(|_| {
                        CssSpecifiedValueSerializationError::new(
                            CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
                        )
                    })?;
                    pending.push((a, b));
                }
                (Basis::Sum(a), Basis::Sum(b)) => {
                    if !a.specified_inverse_eq(b, context)? {
                        return Ok(false);
                    }
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    pub fn try_from_component(component: CssComponentValue) -> Result<Self> {
        Self::try_from_component_with_limits(component, CssComponentValueLimits::default())
    }

    pub fn try_from_component_with_limits(
        component: CssComponentValue,
        limits: CssComponentValueLimits,
    ) -> Result<Self> {
        Self::from_component_with_policy(component, limits, false)
    }

    pub(crate) fn from_component_with_policy(
        component: CssComponentValue,
        limits: CssComponentValueLimits,
        recovered: bool,
    ) -> Result<Self> {
        let values = CssComponentValues::try_new_with_limits(vec![component], limits)
            .map_err(CssNumericConstructionError::component)?;
        validate_calc_size_graph(&values, limits, recovered)?;
        let root = &values.items()[0];
        let mut chain = Vec::new();
        let mut current = root;
        let mut current_path = vec![0];
        loop {
            let (basis, calculation, comma) = parts(current)
                .map_err(|error| error.with_path(Some(current_path.clone().into_boxed_slice())))?;
            let first_basis = significant(basis).next().expect("checked nonempty basis");
            chain.push((current, basis, calculation, comma, current_path.clone()));
            let mut sig = significant(basis);
            let one = sig.next();
            if sig.next().is_none()
                && let Some(CssComponentValueRef::Function(function)) =
                    one.map(CssComponentValue::view)
                && function.name().eq_ignore_ascii_case("calc-size")
            {
                let index = basis
                    .iter()
                    .position(|value| std::ptr::eq(value, first_basis))
                    .expect("basis component belongs to function");
                current_path.push(index);
                current = first_basis;
            } else {
                break;
            }
        }
        let mut built: Option<CssCalcSize> = None;
        while let Some((component, basis_items, calculation_items, comma, path)) = chain.pop() {
            let first = significant(basis_items)
                .next()
                .expect("checked nonempty basis");
            let basis_index = basis_items
                .iter()
                .position(|value| std::ptr::eq(value, first))
                .expect("basis component belongs to function");
            let mut basis_path = path.clone();
            basis_path.push(basis_index);
            let mut sig = significant(basis_items);
            let sole = sig.next().filter(|_| sig.next().is_none());
            let basis = if let Some(child) = built.take() {
                Basis::Nested(child)
            } else if let Some(CssComponentValueRef::Token(CssValueTokenRef::Ident(name))) =
                sole.map(CssComponentValue::view)
            {
                if name.eq_ignore_ascii_case("any") {
                    Basis::Any
                } else if let Some(keyword) = CssIntrinsicSizeKeyword::parse(name) {
                    Basis::Keyword(keyword)
                } else {
                    return Err(CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::InvalidArgumentType,
                        Some(first),
                    )
                    .with_path(Some(basis_path.clone().into_boxed_slice())));
                }
            } else {
                if matches!(sole.map(CssComponentValue::view), Some(CssComponentValueRef::Function(f)) if f.name().eq_ignore_ascii_case("fit-content"))
                {
                    return Err(CssNumericConstructionError::at(
                        CssNumericConstructionErrorKind::InvalidArgumentType,
                        Some(first),
                    )
                    .with_path(Some(basis_path.clone().into_boxed_slice())));
                }
                Basis::Sum(Box::new(
                    parse_calc_size_sum(basis_items, false)
                        .map_err(|error| error.prefix_path(&path))?,
                ))
            };
            let allow_size = !matches!(basis, Basis::Any);
            let calculation = parse_calc_size_sum(calculation_items, allow_size)
                .map_err(|error| error.offset_path(&path, comma + 1))?;
            built = Some(Self {
                data: Box::new(CssCalcSizeData {
                    basis,
                    basis_origin: first.origin().clone(),
                    basis_path: basis_path.into_boxed_slice(),
                    calculation: Box::new(calculation),
                    origin: component.origin().clone(),
                }),
            });
        }
        Ok(built.expect("nonempty calc-size chain"))
    }

    pub fn basis(&self) -> CssCalcSizeBasisRef<'_> {
        match &self.data.basis {
            Basis::Keyword(keyword) => CssCalcSizeBasisRef::Keyword(*keyword),
            Basis::Any => CssCalcSizeBasisRef::Any,
            Basis::Nested(value) => CssCalcSizeBasisRef::Nested(value),
            Basis::Sum(value) => CssCalcSizeBasisRef::Sum(value.as_ref().as_ref()),
        }
    }
    pub fn basis_origin(&self) -> &CssValueOrigin {
        &self.data.basis_origin
    }
    pub fn calculation(&self) -> CssCalculationExpressionRef<'_> {
        self.data.calculation.as_ref().as_ref()
    }
    pub fn origin(&self) -> &CssValueOrigin {
        &self.data.origin
    }
    /// Rejects the first `auto` basis at a maximum-size owner boundary.
    pub(crate) fn validate_maximum_context(&self) -> Result<()> {
        let mut current = self;
        loop {
            match &current.data.basis {
                Basis::Keyword(CssIntrinsicSizeKeyword::Auto) => {
                    return Err(CssNumericConstructionError::at_origin(
                        CssNumericConstructionErrorKind::InvalidArgumentType,
                        current.data.basis_origin.clone(),
                    )
                    .with_path(Some(current.data.basis_path.clone())));
                }
                Basis::Nested(child) => current = child,
                _ => return Ok(()),
            }
        }
    }

    /// Checks that no nested basis is the flex-only `content` keyword.
    pub(crate) fn validate_box_context(&self) -> Result<()> {
        let mut current = self;
        loop {
            match &current.data.basis {
                Basis::Keyword(CssIntrinsicSizeKeyword::Content) => {
                    return Err(CssNumericConstructionError::at_origin(
                        CssNumericConstructionErrorKind::InvalidArgumentType,
                        current.data.basis_origin.clone(),
                    )
                    .with_path(Some(current.data.basis_path.clone())));
                }
                Basis::Nested(child) => current = child,
                _ => return Ok(()),
            }
        }
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        let mut nested = Vec::new();
        let mut current = self;
        loop {
            context.charge_input(1)?;
            context.append(output, "calc-size(")?;
            match &current.data.basis {
                Basis::Keyword(keyword) => {
                    context.charge_input(1)?;
                    context.charge_projection(1)?;
                    context.append(output, keyword.as_css())?;
                }
                Basis::Any => {
                    context.charge_input(1)?;
                    context.charge_projection(1)?;
                    context.append(output, "any")?;
                }
                Basis::Sum(expression) => {
                    project_calc_size_sum_into(expression, context, output)?;
                }
                Basis::Nested(child) => {
                    nested.push(current);
                    current = child;
                    continue;
                }
            }
            context.append(output, ", ")?;
            project_calc_size_sum_into(&current.data.calculation, context, output)?;
            context.append(output, ")")?;
            while let Some(parent) = nested.pop() {
                context.append(output, ", ")?;
                project_calc_size_sum_into(&parent.data.calculation, context, output)?;
                context.append(output, ")")?;
            }
            return Ok(());
        }
    }
}

/// A checked `calc-size()` whose complete basis chain is valid for box sizing.
///
/// The flex-only `content` basis cannot enter width, height, or their min/max
/// counterparts through nested typed construction.
///
/// ```compile_fail
/// use surgeist_css::{CssBoxSize, CssCalcSize, parse_component_values};
/// let values = parse_component_values("calc-size(content, size)").unwrap();
/// let generic = CssCalcSize::try_from_component(values.items()[0].clone()).unwrap();
/// let _box_size = CssBoxSize::CalcSize(generic);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBoxCalcSize(CssCalcSize);

impl TryFrom<CssCalcSize> for CssBoxCalcSize {
    type Error = CssNumericConstructionError;

    fn try_from(value: CssCalcSize) -> Result<Self> {
        value.validate_box_context()?;
        Ok(Self(value))
    }
}

impl CssBoxCalcSize {
    pub fn as_calc_size(&self) -> &CssCalcSize {
        &self.0
    }

    pub fn into_calc_size(self) -> CssCalcSize {
        self.0
    }

    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.0.serialize_specified()
    }

    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.0.append_to_rule_writer(writer)
    }

    pub(crate) fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        self.0.serialize_specified_into(context, output)
    }

    pub(crate) fn validate_maximum_context(&self) -> Result<()> {
        self.0.validate_maximum_context()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maximum_context_reports_first_nested_auto_origin() {
        let values =
            crate::parse_component_values("calc-size(calc-size(auto, size), 1px)").unwrap();
        let serialized = values.serialize().unwrap();
        let value = CssCalcSize::try_from_component(values.items()[0].clone()).unwrap();
        let CssCalcSizeBasisRef::Nested(child) = value.basis() else {
            panic!("nested")
        };
        let error = value.validate_maximum_context().unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::InvalidArgumentType
        );
        assert_eq!(error.origin(), Some(child.basis_origin()));
        let fallback = cssparser::SourceLocation { line: 0, column: 1 };
        let location = crate::numeric::NumericInputContext::components(&values, &serialized)
            .error_location(&error, fallback, 0);
        assert_eq!(
            location.column as usize,
            serialized.as_css().find("auto").unwrap() + 1
        );
    }

    #[test]
    fn box_context_reports_nested_content_with_duplicate_programmatic_origins() {
        use crate::numeric::NumericInputContext;

        let content = CssComponentValue::try_ident("content").unwrap();
        let size = CssComponentValue::try_ident("size").unwrap();
        let comma = CssComponentValue::try_token(",").unwrap();
        let nested = CssComponentValue::try_function(
            "calc-size",
            CssComponentValues::try_new(vec![content.clone(), comma.clone(), size.clone()])
                .unwrap(),
        )
        .unwrap();
        let outer = CssComponentValue::try_function(
            "calc-size",
            CssComponentValues::try_new(vec![nested, comma, size]).unwrap(),
        )
        .unwrap();
        let graph = CssComponentValues::try_new(vec![outer.clone()]).unwrap();
        let serialized = graph.serialize().unwrap();
        let value = CssCalcSize::try_from_component(outer).unwrap();
        let CssCalcSizeBasisRef::Nested(child) = value.basis() else {
            panic!("nested basis")
        };
        assert_eq!(child.basis_origin(), content.origin());
        let error = CssBoxCalcSize::try_from(value).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::InvalidArgumentType
        );
        assert_eq!(error.origin(), Some(content.origin()));
        let location = NumericInputContext::components(&graph, &serialized).error_location(
            &error,
            cssparser::SourceLocation { line: 0, column: 1 },
            0,
        );
        assert_eq!(location.line, 0);
        assert_eq!(
            location.column as usize,
            serialized.as_css().find("content").unwrap() + 1
        );
    }

    #[test]
    fn recovered_eof_is_only_admitted_through_parser_policy() {
        let values = crate::parse_component_values("calc-size(min-content, size").unwrap();
        let component = values.items()[0].clone();
        assert_eq!(
            CssCalcSize::try_from_component(component.clone())
                .unwrap_err()
                .kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
        assert!(
            CssCalcSize::from_component_with_policy(
                component,
                CssComponentValueLimits::default(),
                true
            )
            .is_ok()
        );
    }

    #[test]
    fn programmatic_duplicate_origins_use_structural_error_path() {
        use crate::numeric::NumericInputContext;
        let comma = CssComponentValue::try_token(",").unwrap();
        let space = CssComponentValue::try_token(" ").unwrap();
        let plus = CssComponentValue::try_token("+").unwrap();
        let function = CssComponentValue::try_function(
            "calc-size",
            CssComponentValues::try_new(vec![
                CssComponentValue::try_ident("min-content").unwrap(),
                comma,
                space.clone(),
                CssComponentValue::try_ident("size").unwrap(),
                space.clone(),
                plus,
                space,
                CssComponentValue::try_dimension("1", "fr").unwrap(),
            ])
            .unwrap(),
        )
        .unwrap();
        let graph = CssComponentValues::try_new(vec![function.clone()]).unwrap();
        let serialized = graph.serialize().unwrap();
        let error = CssCalcSize::try_from_component(function).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::IncompatibleTypes
        );
        let fallback = cssparser::SourceLocation { line: 0, column: 1 };
        let location = NumericInputContext::components(&graph, &serialized)
            .error_location(&error, fallback, 0);
        assert_eq!(location.line, 0);
        assert_eq!(
            location.column as usize,
            serialized.as_css().find("1fr").unwrap() + 1
        );
    }

    #[test]
    fn container_numeric_context_still_rejects_size() {
        let values = crate::parse_component_values("calc(size + 1px)").unwrap();
        assert!(
            crate::numeric::construct_container(
                values,
                crate::numeric::CalculationRoot::LengthPercentage,
                CssComponentValueLimits::default(),
            )
            .is_err()
        );
    }
}
