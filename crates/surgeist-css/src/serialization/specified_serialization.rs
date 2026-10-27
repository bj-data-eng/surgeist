//! Bounded, context-independent specified-value text.

use crate::serialization::numeric_formatting::format_css_number;
use crate::values::box_model::CssBoxSizing;
use crate::values::effects::{CssOpacityScalarKind, CssOpacityValue};
use crate::values::table::{CssBorderCollapse, CssCaptionSide, CssEmptyCells, CssTableLayout};
use std::fmt;

/// Resource policy for specified-value projection and serialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssSerializationLimits {
    max_input_nodes: usize,
    max_projection_nodes: usize,
    max_css_bytes: usize,
}

impl CssSerializationLimits {
    /// Sets independent limits. Zero is a valid limit.
    #[must_use]
    pub const fn new(
        max_input_nodes: usize,
        max_projection_nodes: usize,
        max_css_bytes: usize,
    ) -> Self {
        Self {
            max_input_nodes,
            max_projection_nodes,
            max_css_bytes,
        }
    }
    /// Returns the allowance of input nodes: structural nodes of the value being serialized.
    #[must_use]
    pub const fn max_input_nodes(self) -> usize {
        self.max_input_nodes
    }
    /// Returns the allowance of projection nodes: structural nodes of the serialized output.
    #[must_use]
    pub const fn max_projection_nodes(self) -> usize {
        self.max_projection_nodes
    }
    /// Returns the allowance of UTF-8 bytes of CSS text the operation may emit.
    #[must_use]
    pub const fn max_css_bytes(self) -> usize {
        self.max_css_bytes
    }
}

impl Default for CssSerializationLimits {
    fn default() -> Self {
        Self::new(65_536, 262_144, 1_048_576)
    }
}

/// An exhausted resource or a value that cannot be faithfully represented, read through
/// the predicates of [`CssSerializationError`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssSpecifiedValueSerializationErrorKind {
    InputNodeLimit,
    ProjectionNodeLimit,
    ByteLimit,
    /// The explicitly depth-bounded graph operation exceeded structural depth.
    DepthLimit,
    CapacityOverflow,
    /// Retained component boundaries cannot be emitted without changing tokens.
    UnserializableBoundary,
    /// The owning serializer cannot represent the value within its selected
    /// capability or precision policy. This includes invalid retained selector
    /// identity and numeric rounding that loses a required slot distinction.
    UnrepresentableValue,
    /// An explicitly supplied selector context lacks a required namespace binding.
    NamespaceBindingUnavailable,
}

/// Why a specified, CSSOM or lexical serialization returned no text.
///
/// Failure is atomic: no partial CSS is returned and the serialized value is
/// unchanged. The `is_*` predicates classify the failure and
/// [`Self::is_resource_limit`] groups the resource refusals; a lexical component
/// failure is also available as [`Self::component_error`], which is the
/// [`source`](std::error::Error::source).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSerializationError {
    pub(crate) kind: CssSpecifiedValueSerializationErrorKind,
    detail: Option<Box<SerializationDetail>>,
}

/// Facts beyond the kind, boxed so the error stays two words wide.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SerializationDetail {
    cause: Option<SerializationCause>,
    origin: Option<crate::component::CssValueOrigin>,
    rule_index: Option<usize>,
}

/// The specific failure behind a kind, when the kind alone does not name it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum SerializationCause {
    /// Retained component syntax or its byte budget refused lexical output.
    Component(crate::component::CssComponentValueError),
    /// A recovered media query has no authored serialization.
    RecoveredMedia,
    /// Reparsing the serialized import would change its interpretation.
    ImportInterpretation,
}

impl CssSerializationError {
    pub(crate) const fn new(kind: CssSpecifiedValueSerializationErrorKind) -> Self {
        Self { kind, detail: None }
    }
    fn with_cause(
        kind: CssSpecifiedValueSerializationErrorKind,
        cause: SerializationCause,
        origin: &crate::component::CssValueOrigin,
    ) -> Self {
        Self {
            kind,
            detail: Some(Box::new(SerializationDetail {
                cause: Some(cause),
                origin: Some(origin.clone()),
                rule_index: None,
            })),
        }
    }
    fn detail_mut(&mut self) -> &mut SerializationDetail {
        self.detail.get_or_insert_with(Box::default)
    }
    fn cause(&self) -> Option<&SerializationCause> {
        self.detail.as_ref()?.cause.as_ref()
    }
    /// A lexical component failure classified by its serializer, located at the
    /// component's origin.
    pub(crate) fn component(
        kind: CssSpecifiedValueSerializationErrorKind,
        error: crate::component::CssComponentValueError,
    ) -> Self {
        let origin = error.origin().clone();
        Self::with_cause(kind, SerializationCause::Component(error), &origin)
    }
    /// A recovered media query member that authored serialization cannot emit.
    pub(crate) fn recovered_media(origin: &crate::component::CssValueOrigin) -> Self {
        Self::with_cause(
            CssSpecifiedValueSerializationErrorKind::UnserializableBoundary,
            SerializationCause::RecoveredMedia,
            origin,
        )
    }
    /// An import whose serialized form would reparse with different clauses.
    pub(crate) fn import_interpretation(origin: &crate::component::CssValueOrigin) -> Self {
        Self::with_cause(
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
            SerializationCause::ImportInterpretation,
            origin,
        )
    }
    /// Genuine owning/component provenance when the selected operation has an
    /// attributable witness. A programmatic graph wrapper can have no origin.
    pub fn origin(&self) -> Option<&crate::component::CssValueOrigin> {
        self.detail.as_ref()?.origin.as_ref()
    }
    pub(crate) fn at_origin(mut self, origin: &crate::component::CssValueOrigin) -> Self {
        if self.origin().is_none() {
            self.detail_mut().origin = Some(origin.clone());
        }
        self
    }
    /// The private classification, for in-crate tests.
    #[cfg(test)]
    pub(crate) const fn kind(&self) -> CssSpecifiedValueSerializationErrorKind {
        self.kind
    }
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether the input node limit was exceeded.
        is_input_node_limit => CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        /// Returns whether the projection node limit was exceeded.
        is_projection_node_limit => CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        /// Returns whether the CSS byte limit was exceeded.
        is_byte_limit => CssSpecifiedValueSerializationErrorKind::ByteLimit,
        /// Returns whether an explicitly depth-bounded graph operation exceeded its
        /// structural depth.
        is_depth_limit => CssSpecifiedValueSerializationErrorKind::DepthLimit,
        /// Returns whether a capacity calculation overflowed.
        is_capacity_overflow => CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        /// Returns whether retained component boundaries cannot be emitted without changing
        /// tokens.
        is_unserializable_boundary => CssSpecifiedValueSerializationErrorKind::UnserializableBoundary,
        /// Returns whether the owning serializer cannot represent the value within its
        /// selected capability or precision policy.
        is_unrepresentable_value => CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
        /// Returns whether an explicitly supplied selector context lacks a required namespace
        /// binding.
        is_namespace_binding_unavailable =>
            CssSpecifiedValueSerializationErrorKind::NamespaceBindingUnavailable,
        /// Returns whether a resource limit refused the serialization: the input node,
        /// projection node or byte limit, or a capacity overflow.
        is_resource_limit => CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            | CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            | CssSpecifiedValueSerializationErrorKind::ByteLimit
            | CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
    }
    /// Zero-based stylesheet rule index when a sheet's rule caused the failure.
    #[must_use]
    pub fn rule_index(&self) -> Option<usize> {
        self.detail.as_ref()?.rule_index
    }
    pub(crate) fn at_rule_index(mut self, index: usize) -> Self {
        if self.rule_index().is_none() {
            self.detail_mut().rule_index = Some(index);
        }
        self
    }
    /// Returns the lexical component failure behind the kind.
    #[must_use]
    pub fn component_error(&self) -> Option<&crate::component::CssComponentValueError> {
        match self.cause() {
            Some(SerializationCause::Component(error)) => Some(error),
            _ => None,
        }
    }
}
impl fmt::Display for CssSerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cause() {
            Some(SerializationCause::RecoveredMedia) => {
                return formatter
                    .write_str("a recovered media query cannot be serialized as authored");
            }
            Some(SerializationCause::ImportInterpretation) => {
                return formatter
                    .write_str("the serialized import would change its interpretation");
            }
            Some(SerializationCause::Component(_)) | None => {}
        }
        formatter.write_str(match self.kind {
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit => {
                "specified-value input node limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit => {
                "specified-value projection node limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::ByteLimit => {
                "specified-value CSS byte limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::DepthLimit => {
                "CSS graph structural depth limit exceeded"
            }
            CssSpecifiedValueSerializationErrorKind::CapacityOverflow => {
                "specified-value capacity calculation overflowed"
            }
            CssSpecifiedValueSerializationErrorKind::UnserializableBoundary => {
                "specified-value component boundary cannot be serialized"
            }
            CssSpecifiedValueSerializationErrorKind::NamespaceBindingUnavailable => {
                "selector namespace binding unavailable"
            }
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue => {
                "specified value cannot be represented by the selected serializer"
            }
        })
    }
}
impl std::error::Error for CssSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

type Result<T> = std::result::Result<T, CssSerializationError>;
use CssSpecifiedValueSerializationErrorKind as Kind;

/// Monotonic resources shared by every projection participating in one public
/// specified-value serialization. Dropping a child arena never refunds work.
pub(crate) struct SpecifiedSerializationContext {
    limits: CssSerializationLimits,
    input_nodes: usize,
    projection_nodes: usize,
    css_bytes: usize,
    output_suppressed: bool,
    temporary_css_bytes: Option<usize>,
    semantic_traversal: bool,
}

impl SpecifiedSerializationContext {
    pub(crate) const fn new(limits: CssSerializationLimits) -> Self {
        Self {
            limits,
            input_nodes: 0,
            projection_nodes: 0,
            css_bytes: 0,
            output_suppressed: false,
            temporary_css_bytes: None,
            semantic_traversal: false,
        }
    }

    fn charge(current: &mut usize, amount: usize, limit: usize, kind: Kind) -> Result<()> {
        let next = current
            .checked_add(amount)
            .ok_or_else(|| CssSerializationError::new(Kind::CapacityOverflow))?;
        if next > limit {
            return Err(CssSerializationError::new(kind));
        }
        *current = next;
        Ok(())
    }

    pub(crate) fn charge_input(&mut self, amount: usize) -> Result<()> {
        if self.semantic_traversal {
            return Self::charge(
                &mut self.projection_nodes,
                amount,
                self.limits.max_projection_nodes(),
                Kind::ProjectionNodeLimit,
            );
        }
        Self::charge(
            &mut self.input_nodes,
            amount,
            self.limits.max_input_nodes(),
            Kind::InputNodeLimit,
        )
    }

    /// Charges `count` visited nodes to both the input and the projection
    /// budget; the single owner of the node charge.
    pub(crate) fn charge_nodes(&mut self, count: usize) -> Result<()> {
        self.charge_input(count)?;
        self.charge_projection(count)
    }

    /// Emits one keyword as one charged node followed by its bytes.
    pub(crate) fn keyword(&mut self, output: &mut String, text: &str) -> Result<()> {
        self.charge_nodes(1)?;
        self.append(output, text)
    }

    pub(crate) fn charge_projection(&mut self, amount: usize) -> Result<()> {
        if self.semantic_traversal {
            return Ok(());
        }
        self.charge_generated_projection(amount)
    }

    /// Generated arena nodes are real work even during a semantic probe.
    /// Ordinary provider projection visits are already represented by the
    /// redirected input visits; allocations retain their independent tariff.
    pub(crate) fn charge_generated_projection(&mut self, amount: usize) -> Result<()> {
        Self::charge(
            &mut self.projection_nodes,
            amount,
            self.limits.max_projection_nodes(),
            Kind::ProjectionNodeLimit,
        )
    }

    pub(crate) fn remaining_bytes(&self) -> usize {
        self.limits.max_css_bytes() - self.css_bytes - self.temporary_css_bytes.unwrap_or(0)
    }

    /// Charges a distinct owned capture without concatenating its text.
    pub(crate) fn charge_captured_bytes(&mut self, bytes: usize) -> Result<()> {
        Self::charge(
            &mut self.css_bytes,
            bytes,
            self.limits.max_css_bytes(),
            Kind::ByteLimit,
        )
    }

    pub(crate) const fn output_suppressed(&self) -> bool {
        self.output_suppressed
    }

    pub(crate) fn replace_output_suppression(&mut self, suppressed: bool) -> bool {
        std::mem::replace(&mut self.output_suppressed, suppressed)
    }

    pub(crate) fn replace_semantic_traversal(&mut self, semantic: bool) -> bool {
        std::mem::replace(&mut self.semantic_traversal, semantic)
    }

    pub(crate) fn replace_temporary_output(&mut self, bytes: Option<usize>) -> Option<usize> {
        std::mem::replace(&mut self.temporary_css_bytes, bytes)
    }

    pub(crate) fn append(&mut self, output: &mut String, text: &str) -> Result<()> {
        if self.output_suppressed {
            return Ok(());
        }
        if let Some(bytes) = self.temporary_css_bytes {
            debug_assert_eq!(output.len(), bytes);
            let next = bytes
                .checked_add(text.len())
                .ok_or_else(|| CssSerializationError::new(Kind::CapacityOverflow))?;
            if next > self.limits.max_css_bytes() - self.css_bytes {
                return Err(CssSerializationError::new(Kind::ByteLimit));
            }
            output
                .try_reserve(text.len())
                .map_err(|_| CssSerializationError::new(Kind::CapacityOverflow))?;
            output.push_str(text);
            self.temporary_css_bytes = Some(next);
            return Ok(());
        }
        debug_assert_eq!(output.len(), self.css_bytes);
        let next = self
            .css_bytes
            .checked_add(text.len())
            .ok_or_else(|| CssSerializationError::new(Kind::CapacityOverflow))?;
        if next > self.limits.max_css_bytes() {
            return Err(CssSerializationError::new(Kind::ByteLimit));
        }
        output
            .try_reserve(text.len())
            .map_err(|_| CssSerializationError::new(Kind::CapacityOverflow))?;
        output.push_str(text);
        self.css_bytes = next;
        Ok(())
    }

    /// Appends to a caller-owned scratch value without charging final-output
    /// bytes. The scratch value is still bounded by the final output space that
    /// remains, so branch selection cannot allocate a per-channel value larger
    /// than the public operation could return.
    pub(crate) fn append_temporary(&self, output: &mut String, text: &str) -> Result<()> {
        let next = output
            .len()
            .checked_add(text.len())
            .ok_or_else(|| CssSerializationError::new(Kind::CapacityOverflow))?;
        if next > self.remaining_bytes() {
            return Err(CssSerializationError::new(Kind::ByteLimit));
        }
        output
            .try_reserve(text.len())
            .map_err(|_| CssSerializationError::new(Kind::CapacityOverflow))?;
        output.push_str(text);
        Ok(())
    }
}

impl crate::numeric::CssZeroLiteral {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(output, "0")
    }
}
impl crate::numeric::CssAngleOrZero {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Angle(value) => value.append_specified(context, output),
            Self::Zero(value) => value.append_specified(context, output),
        }
    }
}

/// Formats a checked ordinary coefficient with six fractional places at most.
/// Suffix bytes belong to this operation's budget, not a later unchecked append.
pub(crate) fn format_coefficient(
    text: &str,
    shift: i128,
    suffix: &str,
    limit: usize,
) -> Result<String> {
    let remaining = limit.checked_sub(suffix.len()).ok_or_else(|| {
        crate::serialization::CssSerializationError::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::ByteLimit)
    })?;
    let mut result = format_css_number(text, shift, remaining)?;
    result.try_reserve(suffix.len()).map_err(|_| {
        crate::serialization::CssSerializationError::new(
            crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow,
        )
    })?;
    result.push_str(suffix);
    Ok(result)
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for crate::values::text::CssOverflowWrap
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Normal => "normal",
            Self::BreakWord => "break-word",
            Self::Anywhere => "anywhere",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    crate::values::text::CssOverflowWrap
);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssBoxSizing {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::ContentBox => "content-box",
            Self::BorderBox => "border-box",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssBoxSizing);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssBorderCollapse {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Collapse => "collapse",
            Self::Separate => "separate",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssBorderCollapse);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssCaptionSide {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssCaptionSide);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssEmptyCells {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Show => "show",
            Self::Hide => "hide",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssEmptyCells);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssTableLayout {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::Fixed => "fixed",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssTableLayout);

impl crate::values::box_model::CssFlowTolerance {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        let keyword = match self.as_ref() {
            crate::values::box_model::CssFlowToleranceRef::Normal => "normal",
            crate::values::box_model::CssFlowToleranceRef::Infinite => "infinite",
            crate::values::box_model::CssFlowToleranceRef::LengthPercentage(value) => {
                return value.append_specified(context, output);
            }
        };
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(output, keyword)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for crate::values::box_model::CssFlowTolerance
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.append_specified(context, output)?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    crate::values::box_model::CssFlowTolerance
);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssOpacityValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<()> {
        match self {
            Self::NumberCalculation(value) => crate::numeric::projection::project_specified_into(
                &value.expression,
                &mut writer.context,
                &mut writer.css,
            )
            .map(|_| ()),
            Self::HintedNumberCalculation(value) => {
                crate::numeric::projection::project_specified_into(
                    &value.expression,
                    &mut writer.context,
                    &mut writer.css,
                )
                .map(|_| ())
            }
            Self::PercentageCalculation(value) => {
                crate::numeric::projection::project_specified_into(
                    &value.expression,
                    &mut writer.context,
                    &mut writer.css,
                )
                .map(|_| ())
            }
            Self::Scalar(value) => {
                writer.node()?;
                if writer.context.output_suppressed() {
                    return Ok(());
                }
                let text = format_lexical(
                    value.numeric().representation(),
                    value.kind() == CssOpacityScalarKind::Percentage,
                    writer.context.remaining_bytes(),
                )?;
                writer.append(&text)
            }
        }
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssOpacityValue);

fn format_lexical(text: &str, percentage: bool, limit: usize) -> Result<String> {
    format_css_number(text, if percentage { -2 } else { 0 }, limit)
}

#[cfg(test)]
mod composed_value_tests {
    use super::*;
    use crate::color::CssColor;
    use crate::numeric::{CssCalculation, CssInteger, CssNumeric};

    #[test]
    fn ordinary_siblings_use_rounded_bytes_in_one_context_without_partial_child_text() {
        let first = crate::numeric::CssNumeric::<
            crate::numeric::CssNumber,
            crate::numeric::CssNonNegative,
        >::try_from_component(
            crate::component::CssComponentValue::try_number(".12345649").unwrap(),
        )
        .unwrap();
        let second = crate::numeric::CssNumeric::<
            crate::numeric::CssNumber,
            crate::numeric::CssNonNegative,
        >::try_from_component(
            crate::component::CssComponentValue::try_number(".9999996").unwrap()
        )
        .unwrap();
        let expected = "0.123456 1";
        let before = (first.clone(), second.clone());
        for (limits, failure) in [
            (CssSerializationLimits::new(2, 2, 10), None),
            (
                CssSerializationLimits::new(1, 2, 10),
                Some(Kind::InputNodeLimit),
            ),
            (
                CssSerializationLimits::new(2, 1, 10),
                Some(Kind::ProjectionNodeLimit),
            ),
            (CssSerializationLimits::new(2, 2, 9), Some(Kind::ByteLimit)),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            first.append_specified(&mut context, &mut output).unwrap();
            context.append(&mut output, " ").unwrap();
            let result = second.append_specified(&mut context, &mut output);
            if let Some(kind) = failure {
                assert_eq!(result.unwrap_err().kind(), kind);
                assert_eq!(output, "0.123456 ");
            } else {
                result.unwrap();
                assert_eq!(output, expected);
            }
            assert_eq!((&first, &second), (&before.0, &before.1));
        }
    }

    #[test]
    fn flow_tolerance_keywords_and_math_share_monotonic_resources() {
        let keyword = crate::values::box_model::CssFlowTolerance::normal();
        let math = crate::values::box_model::CssFlowTolerance::length_percentage(
            crate::numeric::CssNumeric::<crate::numeric::CssLengthPercentage>::try_from_calculation(
                crate::numeric::CssCalculation::<crate::numeric::CssLengthPercentage>::try_from_components(
                    crate::component::values::parse_component_values("calc(1px + 2em)").unwrap(),
                    Default::default(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let expected = "normal calc(2em + 1px)";
        let mut context =
            SpecifiedSerializationContext::new(CssSerializationLimits::new(5, 6, expected.len()));
        let mut output = String::new();
        keyword.append_specified(&mut context, &mut output).unwrap();
        context.append(&mut output, " ").unwrap();
        math.append_specified(&mut context, &mut output).unwrap();
        assert_eq!(output, expected);

        for (limits, kind) in [
            (
                CssSerializationLimits::new(4, 6, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(5, 5, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                CssSerializationLimits::new(5, 6, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut output = String::new();
            keyword.append_specified(&mut context, &mut output).unwrap();
            context.append(&mut output, " ").unwrap();
            assert_eq!(
                math.append_specified(&mut context, &mut output)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(output, "normal ");
        }
    }

    #[test]
    fn color_and_integer_share_input_projection_and_output_limits() {
        let color = CssColor::transparent();
        let integer =
            CssNumeric::<CssInteger>::from_literal(crate::numeric::CssNumericLiteral::from_i32(7));

        let mut context = SpecifiedSerializationContext::new(CssSerializationLimits::new(2, 2, 13));
        let mut css = String::new();
        color.append_specified(&mut context, &mut css).unwrap();
        context.append(&mut css, " ").unwrap();
        integer.append_specified(&mut context, &mut css).unwrap();
        assert_eq!(css, "transparent 7");

        for (limits, expected) in [
            (CssSerializationLimits::new(1, 2, 13), Kind::InputNodeLimit),
            (
                CssSerializationLimits::new(2, 1, 13),
                Kind::ProjectionNodeLimit,
            ),
            (CssSerializationLimits::new(2, 2, 12), Kind::ByteLimit),
        ] {
            let mut context = SpecifiedSerializationContext::new(limits);
            let mut css = String::new();
            color.append_specified(&mut context, &mut css).unwrap();
            context.append(&mut css, " ").unwrap();
            assert_eq!(
                integer
                    .append_specified(&mut context, &mut css)
                    .unwrap_err()
                    .kind(),
                expected
            );
            assert_eq!(css, "transparent ");
        }
    }

    #[test]
    fn sequential_colors_and_integer_calculation_cannot_reset_byte_limit() {
        let color = CssColor::transparent();
        // A math root; a bare root would re-enter literal admission.
        let calculation = CssNumeric::<CssInteger>::try_from_calculation(
            CssCalculation::try_from_components(
                crate::component::values::parse_component_values("calc(2)").unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap();

        let mut context =
            SpecifiedSerializationContext::new(CssSerializationLimits::new(100, 100, 21));
        let mut css = String::new();
        color.append_specified(&mut context, &mut css).unwrap();
        assert_eq!(
            color
                .append_specified(&mut context, &mut css)
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );

        let mut context =
            SpecifiedSerializationContext::new(CssSerializationLimits::new(100, 100, 11));
        let mut css = String::new();
        color.append_specified(&mut context, &mut css).unwrap();
        assert_eq!(
            calculation
                .append_specified(&mut context, &mut css)
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(css, "transparent");
    }
}

// Rust's shortest round-trip spelling supplies the nearest decimal candidate.
// At an exact decimal midpoint choose the numerically greater candidate, even
// when the host formatter's last-digit tie rule chose the other neighbor.
pub(crate) fn format_binary64(value: f64) -> String {
    debug_assert!(value.is_finite());
    if value == 0.0 {
        return "0".into();
    }
    let negative = value.is_sign_negative();
    let shortest = value.abs().to_string();
    let (mantissa, exponent) = shortest.split_once(['e', 'E']).unwrap_or((&shortest, "0"));
    let fractional = mantissa
        .find('.')
        .map_or(0, |point| mantissa.len() - point - 1);
    let mut exponent =
        exponent.parse::<i32>().expect("binary64 decimal exponent") - fractional as i32;
    let significant = mantissa.trim_end_matches('0');
    let trailing = mantissa.len() - significant.len();
    // Trimming a whole-number suffix is balanced by an exponent shift; a
    // decimal point itself is skipped, never counted as a coefficient digit.
    exponent += trailing as i32;
    let mut coefficient = significant
        .bytes()
        .filter(|c| *c != b'.')
        .fold(0u64, |n, c| n * 10 + u64::from(c - b'0'));
    let next = if negative {
        coefficient - 1
    } else {
        coefficient + 1
    };
    let midpoint = (coefficient + next) * 5;
    if binary64_equals_decimal(value.abs(), midpoint, exponent - 1) {
        coefficient = next;
    }
    while coefficient.is_multiple_of(10) {
        coefficient /= 10;
        exponent += 1;
    }
    let coefficient = coefficient.to_string();
    crate::serialization::numeric_formatting::emit(
        coefficient.bytes().map(|c| c - b'0'),
        coefficient.len(),
        i128::from(exponent),
        negative,
        usize::MAX,
    )
    .expect("finite binary64 text has bounded length")
}

fn binary64_equals_decimal(value: f64, mut coefficient: u64, mut exponent: i32) -> bool {
    if coefficient == 0 {
        return false;
    }
    while coefficient.is_multiple_of(10) {
        coefficient /= 10;
        exponent += 1;
    }
    let bits = value.to_bits();
    let encoded = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mut binary_coefficient, binary_exponent) = if encoded == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1u64 << 52), encoded - 1075)
    };
    let mut digits = [0u8; 1100];
    let mut len = 0;
    while binary_coefficient != 0 {
        digits[len] = (binary_coefficient % 10) as u8;
        binary_coefficient /= 10;
        len += 1;
    }
    let multiplier = if binary_exponent < 0 { 5 } else { 2 };
    for _ in 0..binary_exponent.unsigned_abs() {
        let mut carry = 0;
        for digit in &mut digits[..len] {
            let product = *digit * multiplier + carry;
            *digit = product % 10;
            carry = product / 10;
        }
        if carry != 0 {
            digits[len] = carry;
            len += 1;
        }
    }
    let trailing = digits[..len].iter().take_while(|&&d| d == 0).count();
    if binary_exponent.min(0) + trailing as i32 != exponent {
        return false;
    }
    let decimal = coefficient.to_string();
    decimal.len() == len - trailing
        && decimal
            .bytes()
            .map(|c| c - b'0')
            .eq(digits[trailing..len].iter().rev().copied())
}

#[cfg(test)]
mod tests {
    use super::{binary64_equals_decimal, format_binary64};

    #[test]
    fn binary64_text_uses_fixed_notation_and_preserves_retained_value() {
        for (value, expected) in [
            (0.0, "0"),
            (-0.0, "0"),
            (0.1, "0.1"),
            (1.0 / 3.0, "0.3333333333333333"),
            (-1.0 / 3.0, "-0.3333333333333333"),
            (1e20, "100000000000000000000"),
            (1e-7, "0.0000001"),
        ] {
            assert_eq!(format_binary64(value), expected);
        }
        for bits in [
            1,
            2,
            3,
            0x0010_0000_0000_0000,
            0x3fef_ffff_ffff_ffff,
            0x3ff0_0000_0000_0001,
            0x7fef_ffff_ffff_ffff,
        ] {
            for value in [f64::from_bits(bits), -f64::from_bits(bits)] {
                let text = format_binary64(value);
                assert!(!text.contains(['e', 'E']));
                assert_eq!(text.parse::<f64>().unwrap().to_bits(), value.to_bits());
            }
        }
    }

    #[test]
    fn decimal_midpoint_comparison_uses_exact_binary_value() {
        assert!(binary64_equals_decimal(0.125, 125, -3));
        assert!(binary64_equals_decimal(1.5, 150, -2));
        assert!(binary64_equals_decimal(
            9007199254740992.0,
            9007199254740992,
            0
        ));
        assert!(!binary64_equals_decimal(0.1, 1, -1));
        assert!(!binary64_equals_decimal(f64::from_bits(1), 5, -324));
    }
}
