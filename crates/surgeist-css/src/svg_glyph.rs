//! Independent authored SVG glyph orientation, before computed rotation and rounding.
use crate::*;

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Auto,
    Angle(CssAngleValue),
    Unitless(Box<CssComponentValue>),
}

/// An authored SVG `auto`, angle, or Number literal with implied degrees.
/// Equality includes the retained providers' provenance, not computed orientation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSvgGlyphOrientationVerticalValue {
    value: Value,
}

// Frozen WebKit consumes raw numeric tokens as binary64. This conversion only
// decides literal eligibility; no floating value is stored or used for output.
fn raw_number(
    number: CssNumericTokenRef<'_>,
    origin: &CssValueOrigin,
) -> Result<f64, CssNumericConstructionError> {
    number
        .representation()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::OutOfRange,
                origin.clone(),
            )
        })
}

impl CssSvgGlyphOrientationVerticalValue {
    /// Constructs the intrinsic Auto initial value.
    #[must_use]
    pub const fn auto() -> Self {
        Self { value: Value::Auto }
    }
    /// Retains an Angle-root provider, rejecting recovered closure and raw overflow.
    /// Nonfinite constants inside angle math remain authored and unevaluated.
    pub fn try_from_angle(angle: CssAngleValue) -> Result<Self, CssNumericConstructionError> {
        if let Some(calculation) = angle.calculation()
            && let Some(origin) = calculation.components().first_implicit_origin()
        {
            return Err(CssNumericConstructionError::at_origin(
                CssNumericConstructionErrorKind::RecoveredComponent,
                origin.clone(),
            ));
        }
        Self::from_parser_angle(angle)
    }
    pub(crate) fn from_parser_angle(
        angle: CssAngleValue,
    ) -> Result<Self, CssNumericConstructionError> {
        if let Some(literal) = angle.literal() {
            raw_number(literal.numeric(), literal.origin())?;
        }
        Ok(Self {
            value: Value::Angle(angle),
        })
    }
    /// Requires one finite raw Number token, preserving its flag, spelling and origin.
    /// This intrinsic constructor does not select CSS versus attribute admission.
    pub fn try_from_unitless(
        component: CssComponentValue,
    ) -> Result<Self, CssNumericConstructionError> {
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            return Err(CssNumericConstructionError::at(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                Some(&component),
            ));
        };
        raw_number(number, component.origin())?;
        Ok(Self {
            value: Value::Unitless(Box::new(component)),
        })
    }
    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self.value, Value::Auto)
    }
    #[must_use]
    pub const fn angle(&self) -> Option<&CssAngleValue> {
        match &self.value {
            Value::Angle(value) => Some(value),
            _ => None,
        }
    }
    #[must_use]
    pub fn unitless_component(&self) -> Option<&CssComponentValue> {
        match &self.value {
            Value::Unitless(value) => Some(value),
            _ => None,
        }
    }
    pub(crate) fn admitted(&self, admission: SvgGlyphAdmission) -> bool {
        let Some(component) = self.unitless_component() else {
            return true;
        };
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
            unreachable!("checked Number")
        };
        admission.is_attribute()
            || admission.mode() == CssParserMode::Quirks
            || raw_number(number, component.origin()).is_ok_and(|value| value == 0.0)
    }
    /// Emits specified syntax with the shared six-fraction-place CSSOM policy.
    /// Unitless Numbers become explicit degrees; no computed quadrant is selected.
    pub fn serialize_specified(&self) -> Result<String, CssSpecifiedValueSerializationError> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Emits atomically using shared cumulative resource limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError> {
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }
    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), CssSpecifiedValueSerializationError> {
        match &self.value {
            Value::Auto => writer.keyword("auto"),
            Value::Angle(angle) => angle.append_specified(&mut writer.context, &mut writer.css),
            Value::Unitless(component) => {
                let text =
                    crate::specified_numeric::capture_literal(component, &mut writer.context)?;
                writer.append(&text)?;
                writer.append("deg")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Css,
    PresentationAttribute,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SvgGlyphAdmission {
    mode: CssParserMode,
    role: Role,
}
impl SvgGlyphAdmission {
    pub(crate) const fn css(context: CssParserContext) -> Self {
        Self {
            mode: context.mode(),
            role: Role::Css,
        }
    }
    pub(crate) const fn attribute(context: CssParserContext) -> Self {
        Self {
            mode: context.mode(),
            role: Role::PresentationAttribute,
        }
    }
    pub(crate) const fn mode(self) -> CssParserMode {
        self.mode
    }
    pub(crate) const fn is_attribute(self) -> bool {
        matches!(self.role, Role::PresentationAttribute)
    }
}

/// One checked independent SVG authored body, retaining actual admission for reentry.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSvgGlyphOrientationVerticalDeclaration {
    declared: crate::syntax::CssDeclaredValue<CssSvgGlyphOrientationVerticalValue>,
    admission: SvgGlyphAdmission,
}
impl CssSvgGlyphOrientationVerticalDeclaration {
    pub(crate) const fn new(
        declared: crate::syntax::CssDeclaredValue<CssSvgGlyphOrientationVerticalValue>,
        admission: SvgGlyphAdmission,
    ) -> Self {
        Self {
            declared,
            admission,
        }
    }
    pub(crate) const fn admission(&self) -> SvgGlyphAdmission {
        self.admission
    }
    #[must_use]
    pub const fn property(&self) -> CssPropertyNameRef<'static> {
        CssPropertyNameRef::SvgGlyphOrientationVertical
    }
    #[must_use]
    pub const fn value(&self) -> Option<&CssSvgGlyphOrientationVerticalValue> {
        match &self.declared {
            crate::syntax::CssDeclaredValue::Value(value) => Some(value),
            _ => None,
        }
    }
    #[must_use]
    pub const fn global(&self) -> Option<CssGlobalKeyword> {
        match self.declared {
            crate::syntax::CssDeclaredValue::Global(value) => Some(value),
            _ => None,
        }
    }
    #[must_use]
    pub const fn substitution_dependent(&self) -> Option<&CssSubstitutionDependentValue> {
        match &self.declared {
            crate::syntax::CssDeclaredValue::SubstitutionDependent(value) => Some(value),
            _ => None,
        }
    }
    #[must_use]
    pub const fn parser_mode(&self) -> CssParserMode {
        self.admission.mode()
    }
    #[must_use]
    pub const fn is_presentation_attribute(&self) -> bool {
        self.admission.is_attribute()
    }
    #[must_use]
    pub const fn metadata() -> &'static CssSvgGlyphOrientationVerticalMetadata {
        &METADATA
    }
}

/// Intrinsic metadata for the independent SVG terminal, outside the canonical schema.
#[derive(Debug)]
pub struct CssSvgGlyphOrientationVerticalMetadata {
    initial: CssSvgGlyphOrientationVerticalValue,
}
static METADATA: CssSvgGlyphOrientationVerticalMetadata = CssSvgGlyphOrientationVerticalMetadata {
    initial: CssSvgGlyphOrientationVerticalValue::auto(),
};
impl CssSvgGlyphOrientationVerticalMetadata {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        "glyph-orientation-vertical"
    }
    #[must_use]
    pub const fn property(&self) -> CssPropertyNameRef<'static> {
        CssPropertyNameRef::SvgGlyphOrientationVertical
    }
    #[must_use]
    pub const fn feature_id(&self) -> CssFeatureId {
        CssFeatureId::new("interop.property.svg-glyph-orientation-vertical")
    }
    #[must_use]
    pub const fn initial_value(&self) -> &CssSvgGlyphOrientationVerticalValue {
        &self.initial
    }
    #[must_use]
    pub const fn inherited_by_default(&self) -> bool {
        true
    }
    #[must_use]
    pub const fn is_animatable(&self) -> bool {
        false
    }
    #[must_use]
    pub const fn settable_members(&self) -> &'static [CssPropertyNameRef<'static>] {
        &[CssPropertyNameRef::SvgGlyphOrientationVertical]
    }
    #[must_use]
    pub const fn reset_only_members(&self) -> &'static [CssPropertyNameRef<'static>] {
        &[]
    }
}

#[cfg(test)]
mod writer_tests {
    use super::*;
    use crate::specified_rule_serialization::SpecifiedRuleWriter;
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    fn number() -> CssSvgGlyphOrientationVerticalValue {
        CssSvgGlyphOrientationVerticalValue::try_from_unitless(
            CssComponentValue::try_number("-135").unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn transparent_numbers_share_prefix_sibling_nodes_and_bytes() {
        let value = number();
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, 2, 21));
        writer.append("prefix ").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, "prefix -135deg-135deg");
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("xx").unwrap_err().kind(), Kind::ByteLimit);
        assert_eq!(value, before);
        for (limits, kind) in [
            (Limits::new(1, 2, 21), Kind::InputNodeLimit),
            (Limits::new(2, 1, 21), Kind::ProjectionNodeLimit),
            (Limits::new(2, 2, 19), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("prefix ").unwrap();
            value.append_to_rule_writer(&mut writer).unwrap();
            assert_eq!(
                value.append_to_rule_writer(&mut writer).unwrap_err().kind(),
                kind
            );
            assert!(writer.css.starts_with("prefix -135deg"));
            assert_eq!(value, before);
        }
    }
    #[test]
    fn nested_suppression_charges_number_leaf_and_restores_after_failure() {
        let value = number();
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(1, 1, 1));
        writer
            .without_output(|writer| {
                writer.without_output(|writer| value.append_to_rule_writer(writer))?;
                assert!(writer.context.output_suppressed());
                assert_eq!(
                    writer
                        .without_output(|writer| value.append_to_rule_writer(writer))
                        .unwrap_err()
                        .kind(),
                    Kind::InputNodeLimit
                );
                assert!(writer.context.output_suppressed());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
        assert_eq!(value, before);
    }
}
