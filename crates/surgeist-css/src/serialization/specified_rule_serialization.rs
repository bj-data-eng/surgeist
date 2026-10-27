//! Canonical specified rule and stylesheet composition.

use crate::serialization::serialization_escaping::EscapedKind;

use crate::rule::{CssNamespaceRule, CssRule, CssSheet};
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use crate::serialization::{CssSerializationError, CssSerializationLimits};

/// Private carrier for `?` conversions inside rule-graph providers.
///
/// Every failure it carries is already a [`CssSerializationError`]; the carrier
/// only lets component, import and custom-media failures convert with `?`
/// without adding public conversions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpecifiedRuleSerializationSource(pub(crate) CssSerializationError);

impl From<CssSerializationError> for SpecifiedRuleSerializationSource {
    fn from(error: CssSerializationError) -> Self {
        Self(error)
    }
}

impl From<crate::component::CssComponentValueError> for SpecifiedRuleSerializationSource {
    fn from(error: crate::component::CssComponentValueError) -> Self {
        Self(crate::component::values::specified_component_error(error))
    }
}

#[derive(Clone, Copy)]
struct InverseSourceTracking {
    property: crate::property::CssKnownProperty,
    mode: Option<crate::values::common::CssBoxSideKind>,
}

impl InverseSourceTracking {
    fn members(self) -> &'static [crate::property::CssLonghandProperty] {
        match self
            .property
            .metadata()
            .expect("selected inverse schema")
            .kind()
        {
            crate::property::CssPropertyKindRef::Shorthand(value) => value.settable_members(),
            crate::property::CssPropertyKindRef::FourSideShorthand(value) => {
                value.settable_members(self.mode.expect("selected side mode"))
            }
            _ => unreachable!("inverse shorthand source selection"),
        }
    }
}

pub(crate) struct SpecifiedRuleWriter {
    pub(crate) context: SpecifiedSerializationContext,
    pub(crate) css: String,
    pub(crate) max_graph_depth: Option<usize>,
    pub(crate) graph_depth: usize,
    inverse_source_tracking: Option<InverseSourceTracking>,
    inverse_source_member: Option<crate::property::CssKnownProperty>,
}

/// The private provider behind every public `serialize_specified` (#1288).
///
/// A serializable type implements this one method, appending its canonical
/// specified CSS and charging the writer's shared input, projection and byte
/// budgets; node and keyword charges are [`SpecifiedRuleWriter::node`] and
/// [`SpecifiedRuleWriter::keyword`]. [`specified_serialization!`] then supplies
/// the public method and its one documentation.
///
/// Leaf values also keep a value-level `append_specified(context, output)`.
/// It stays distinct because context-only callers (media CSSOM capture and
/// keyframe selector comparison) and scratch captures charge a
/// [`SpecifiedSerializationContext`] into their own buffers without a rule
/// writer's inverse-source tracking, graph depth or namespace state; the
/// providers of those values delegate to it with the writer's context and
/// output.
pub(crate) trait SpecifiedProvider {
    fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError>;
}

/// Runs one provider under `limits`, returning its complete text or no text.
pub(crate) fn serialize_specified(
    value: &(impl SpecifiedProvider + ?Sized),
    limits: CssSerializationLimits,
) -> Result<String, CssSerializationError> {
    let mut writer = SpecifiedRuleWriter::new(limits);
    value.append_to_rule_writer(&mut writer)?;
    Ok(writer.css)
}

/// Supplies `serialize_specified` for each listed [`SpecifiedProvider`].
macro_rules! specified_serialization {
    ($($ty:ty),+ $(,)?) => {$(
        impl $ty {
            /// Serializes this value as canonical specified CSS.
            ///
            /// The text is context-independent: nothing is computed, resolved
            /// or substituted, and symbolic values stay symbolic. `limits`
            /// bounds input nodes, projection nodes and output UTF-8 bytes
            /// cumulatively across the value and all of its children.
            ///
            /// # Errors
            ///
            /// Returns [`CssSerializationError`](crate::serialization::CssSerializationError)
            /// when a limit is exhausted or the value cannot be represented.
            /// No partial text is returned and the value is unchanged.
            pub fn serialize_specified(
                &self,
                limits: $crate::serialization::CssSerializationLimits,
            ) -> ::std::result::Result<::std::string::String, $crate::serialization::CssSerializationError> {
                $crate::serialization::specified_rule_serialization::serialize_specified(self, limits)
            }
        }
    )+};
}
pub(crate) use specified_serialization;

impl SpecifiedRuleWriter {
    pub(crate) fn new(limits: CssSerializationLimits) -> Self {
        Self {
            context: SpecifiedSerializationContext::new(limits),
            css: String::new(),
            max_graph_depth: None,
            graph_depth: 0,
            inverse_source_tracking: None,
            inverse_source_member: None,
        }
    }

    /// Final inverse emission alone selects source tracking. Ordinary writers
    /// carry no selection; notifications therefore call their body unchanged.
    pub(crate) fn with_inverse_sources<T>(
        &mut self,
        property: crate::property::CssKnownProperty,
        mode: Option<crate::values::common::CssBoxSideKind>,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> (
        Result<T, CssSerializationError>,
        Option<crate::property::CssKnownProperty>,
    ) {
        let previous_tracking = self
            .inverse_source_tracking
            .replace(InverseSourceTracking { property, mode });
        let previous_member = self.inverse_source_member.take();
        let result = visit(self);
        let responsible = self.inverse_source_member;
        self.inverse_source_tracking = previous_tracking;
        self.inverse_source_member = previous_member;
        (result, responsible)
    }

    /// One terminal visit disables nested shorthand notifications until its
    /// Result returns, but retains the responsible identity for a failure.
    fn source_visit<T>(
        &mut self,
        property: crate::property::CssKnownProperty,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> Result<T, CssSerializationError> {
        let previous = self.inverse_source_tracking.take();
        if previous.is_some() {
            self.inverse_source_member = Some(property);
        }
        let result = visit(self);
        self.inverse_source_tracking = previous;
        result
    }

    pub(crate) fn source_member<T>(
        &mut self,
        index: usize,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> Result<T, CssSerializationError> {
        let Some(selection) = self.inverse_source_tracking else {
            return visit(self);
        };
        self.source_visit(selection.members()[index].known_property(), visit)
    }

    pub(crate) fn source_property<T>(
        &mut self,
        property: crate::property::CssKnownProperty,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> Result<T, CssSerializationError> {
        let Some(selection) = self.inverse_source_tracking else {
            return visit(self);
        };
        debug_assert!(
            selection
                .members()
                .iter()
                .any(|member| member.known_property() == property)
        );
        self.source_visit(property, visit)
    }

    /// Border triads select the first admitted side representative of each
    /// width/style/color group from the existing ordered settable schema.
    pub(crate) fn source_group_member<T>(
        &mut self,
        index: usize,
        groups: usize,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> Result<T, CssSerializationError> {
        let Some(selection) = self.inverse_source_tracking else {
            return visit(self);
        };
        let members = selection.members();
        debug_assert_eq!(members.len() % groups, 0);
        self.source_visit(
            members[index * (members.len() / groups)].known_property(),
            visit,
        )
    }

    pub(crate) fn append(&mut self, text: &str) -> Result<(), CssSerializationError> {
        self.context.append(&mut self.css, text)
    }

    /// Charges one semantic node: one input visit and one projection node.
    /// This is the shared node tariff for specified value providers.
    pub(crate) fn node(&mut self) -> Result<(), CssSerializationError> {
        self.context.charge_nodes(1)
    }

    /// Charges `count` visited nodes to both work budgets.
    pub(crate) fn nodes(&mut self, count: usize) -> Result<(), CssSerializationError> {
        self.context.charge_nodes(count)
    }

    /// Emits one keyword as one charged node followed by its bytes.
    pub(crate) fn keyword(&mut self, text: &str) -> Result<(), CssSerializationError> {
        self.node()?;
        self.append(text)
    }

    /// Emits one decoded identifier as one charged node, escaped as CSSOM
    /// "serialize an identifier" requires.
    pub(crate) fn identifier(&mut self, value: &str) -> Result<(), CssSerializationError> {
        self.node()?;
        self.append_identifier(value)
    }

    /// Emits one decoded string as one charged node, quoted as CSSOM
    /// "serialize a string" requires.
    pub(crate) fn string(&mut self, value: &str) -> Result<(), CssSerializationError> {
        self.node()?;
        self.append_string(value)
    }

    /// CSSOM "serialize a CSS value" step 5: writes the space that separates
    /// an item from an earlier emitted item, then records that one was emitted.
    pub(crate) fn space_before_item(
        &mut self,
        emitted: &mut bool,
    ) -> Result<(), CssSerializationError> {
        if *emitted {
            self.append(" ")?;
        }
        *emitted = true;
        Ok(())
    }

    /// CSSOM "serialize a comma-separated list": items joined by `", "`.
    pub(crate) fn comma_separated<T>(
        &mut self,
        items: &[T],
        mut item: impl FnMut(&mut Self, &T) -> Result<(), CssSerializationError>,
    ) -> Result<(), CssSerializationError> {
        self.separated(items, ", ", &mut item)
    }

    /// CSSOM "serialize a CSS value" step 5: items joined by one space.
    pub(crate) fn space_separated<T>(
        &mut self,
        items: &[T],
        mut item: impl FnMut(&mut Self, &T) -> Result<(), CssSerializationError>,
    ) -> Result<(), CssSerializationError> {
        self.separated(items, " ", &mut item)
    }

    fn separated<T>(
        &mut self,
        items: &[T],
        separator: &str,
        item: &mut impl FnMut(&mut Self, &T) -> Result<(), CssSerializationError>,
    ) -> Result<(), CssSerializationError> {
        for (index, value) in items.iter().enumerate() {
            if index != 0 {
                self.append(separator)?;
            }
            item(self, value)?;
        }
        Ok(())
    }

    /// Uses the owning value visitor to inspect semantic nodes, charging the
    /// CSSOM probe tariff rather than rendering/canonicalizing another value.
    pub(crate) fn visit_semantic<T>(
        &mut self,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> Result<T, CssSerializationError> {
        let previous = self.context.replace_semantic_traversal(true);
        let result = self.without_output(visit);
        self.context.replace_semantic_traversal(previous);
        result
    }

    /// Captures a bounded semantic value under this request's cumulative work.
    /// Scratch bytes consume remaining capacity without charging final bytes twice.
    pub(crate) fn capture_value<T>(
        &mut self,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> Result<(T, String), CssSerializationError> {
        let final_css = std::mem::take(&mut self.css);
        let previous = self.context.replace_temporary_output(Some(0));
        let result = visit(self);
        let scratch = std::mem::replace(&mut self.css, final_css);
        self.context.replace_temporary_output(previous);
        result.map(|value| (value, scratch))
    }

    /// Emits a namespace declaration without resolving or normalizing its literal name.
    pub(crate) fn namespace(
        &mut self,
        rule: &CssNamespaceRule,
    ) -> Result<(), CssSerializationError> {
        self.node()?;
        self.append("@namespace ")?;
        if let Some(prefix) = rule.prefix() {
            self.node()?;
            self.append_identifier(prefix.as_str())?;
            self.append(" ")?;
        }
        self.node()?;
        self.append("url(")?;
        self.append_string(rule.name().as_str())?;
        self.append(");")
    }

    /// Visits a proved simple initial through its provider without producing bytes.
    /// Restores the enclosing emission mode even when a nested visit fails.
    pub(crate) fn without_output<T>(
        &mut self,
        visit: impl FnOnce(&mut Self) -> Result<T, CssSerializationError>,
    ) -> Result<T, CssSerializationError> {
        let previous = self.context.replace_output_suppression(true);
        let result = visit(self);
        self.context.replace_output_suppression(previous);
        result
    }

    pub(crate) fn append_identifier(&mut self, value: &str) -> Result<(), CssSerializationError> {
        self.append_escaped(value, EscapedKind::Identifier)
    }

    pub(crate) fn append_string(&mut self, value: &str) -> Result<(), CssSerializationError> {
        self.append_escaped(value, EscapedKind::String)
    }

    fn append_escaped(
        &mut self,
        value: &str,
        kind: EscapedKind,
    ) -> Result<(), CssSerializationError> {
        if self.context.output_suppressed() {
            return Ok(());
        }
        let scratch = crate::serialization::serialization_escaping::capture_escaped(
            value,
            kind,
            &self.context,
        )?;
        self.append(&scratch)
    }
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssNamespaceRule {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.namespace(self)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssNamespaceRule);

impl SpecifiedProvider for CssRule {
    /// Serializes the authored rule graph, preserving declaration and child order.
    /// Rule, selector, declaration and provider work share one budget.
    fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        writer.append_rule_graph(self)
    }
}
specified_serialization!(CssRule);

impl SpecifiedProvider for CssSheet {
    /// Serializes retained authored rules atomically in source order; discarded
    /// source and its diagnostics remain on the original parse report.
    ///
    /// Charges one sheet aggregate and every logical rule once in both work
    /// budgets, then consumes each provider's shared costs. Enum carriers and
    /// child slices add no node. Top-level rules are separated by one newline,
    /// and a failure caused by a rule carries that rule's index.
    fn append_to_rule_writer(
        &self,
        writer: &mut SpecifiedRuleWriter,
    ) -> Result<(), CssSerializationError> {
        writer.node()?;
        for (index, rule) in self.rules().iter().enumerate() {
            if index != 0 {
                writer
                    .append("\n")
                    .map_err(|error| error.at_rule_index(index))?;
            }
            writer
                .append_rule_graph(rule)
                .map_err(|error| error.at_rule_index(index))?;
        }
        Ok(())
    }
}
specified_serialization!(CssSheet);

#[cfg(test)]
mod suppression_tests {
    use super::*;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind;

    #[test]
    fn nested_suppression_restores_enclosing_mode_on_success_and_failure() {
        let mut writer = SpecifiedRuleWriter::new(CssSerializationLimits::new(1, 1, 1));
        writer
            .without_output(|writer| {
                writer.append("discarded outer text")?;
                writer.without_output(|writer| writer.append("discarded nested text"))?;
                assert!(writer.context.output_suppressed());
                let error = writer
                    .without_output(|writer| writer.context.charge_input(2))
                    .unwrap_err();
                assert_eq!(
                    error.kind(),
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit
                );
                assert!(writer.context.output_suppressed());
                writer.append("still discarded")
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }

    #[test]
    fn failing_suppressed_literal_preserves_nodes_and_resumes_final_emission() {
        let mut writer = SpecifiedRuleWriter::new(CssSerializationLimits::new(1, 0, 1));
        let zero = crate::numeric::CssNumeric::<crate::numeric::CssLengthPercentage>::exact_zero();
        let error = writer
            .without_output(|writer| zero.append_specified(&mut writer.context, &mut writer.css))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }
}

#[cfg(test)]
mod value_error_tests {
    use super::*;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind;
    use std::error::Error;

    #[test]
    fn rule_index_marks_the_first_responsible_rule_and_keeps_the_value_failure() {
        use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as Kind;
        for kind in [
            Kind::InputNodeLimit,
            Kind::ProjectionNodeLimit,
            Kind::ByteLimit,
            Kind::CapacityOverflow,
            Kind::UnserializableBoundary,
            Kind::UnrepresentableValue,
        ] {
            let value = CssSerializationError::new(kind);
            let error = value.clone().at_rule_index(7).at_rule_index(3);
            assert_eq!(error.kind(), kind);
            assert_eq!(error.rule_index(), Some(7));
            assert_eq!(value.rule_index(), None);
            assert_eq!(error.to_string(), value.to_string());
            assert!(error.source().is_none());
        }
    }

    #[test]
    fn unrepresentable_failure_explains_a_value_limit_without_a_resource_claim() {
        let error = CssSerializationError::new(
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue,
        );
        assert_eq!(
            error.to_string(),
            "specified value cannot be represented by the selected serializer"
        );
        assert!(error.source().is_none());
    }

    #[test]
    fn nested_recovered_media_failure_keeps_the_media_origin_and_sheet_rule_index() {
        let report = crate::CssParser::default()
            .parse_sheet("@namespace n 'u';@scope{@media screen,???,print{}}");
        let before = report.clone();
        let [CssRule::Namespace(_), CssRule::Scope(scope)] = report.syntax().rules() else {
            panic!("namespace and scope");
        };
        let [crate::rule::CssRule::Media(media)] = scope.rules().rules() else {
            panic!("scoped media");
        };
        // Sheet 1 + namespace 3 + scope 1 + media 1 + list 1 + screen 2
        // + Never 1 precede its two generated projection nodes.
        let error = report
            .syntax()
            .serialize_specified(CssSerializationLimits::new(usize::MAX, 11, 512))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(error.origin(), Some(media.query().queries()[1].origin()));
        assert_eq!(report, before);
    }
}

#[cfg(test)]
mod inverse_source_scope_contract {
    use super::{SpecifiedProvider as _, SpecifiedRuleWriter};
    use crate::property::CssKnownProperty as Property;
    use crate::property::CssKnownPropertyValueRef as Value;
    use crate::serialization::CssSerializationLimits as Limits;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as Kind;

    #[test]
    fn nested_shape_sources_and_error_suppression_restore_the_enclosing_writer() {
        let report = crate::CssParser::default()
            .parse_block_contents("offset:inset(1px round 2px);margin-block:1px 2px");
        assert!(report.is_clean(), "{report:?}");
        let Value::Offset(offset) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("offset")
        };
        let offset = offset.value();
        let Value::MarginBlock(pair) = report.syntax()[1]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("margin pair")
        };
        let pair = pair.value();
        // Offset + path + inset + offsets-list + 1px + radius-list + 2px:
        // seven real I/P visits. The radius's own source notification must stay
        // inside OffsetPath. The independent literal is twenty UTF-8 bytes.
        let exact = Limits::new(7, 7, 20);
        assert_eq!(
            offset.serialize_specified(exact).unwrap(),
            "inset(1px round 2px)"
        );
        for (limits, kind) in [
            (Limits::new(6, 7, 20), Kind::InputNodeLimit),
            (Limits::new(7, 6, 20), Kind::ProjectionNodeLimit),
            (Limits::new(7, 7, 19), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            let (result, member) = writer.with_inverse_sources(Property::Offset, None, |writer| {
                offset.append_to_rule_writer(writer)
            });
            assert_eq!(result.unwrap_err().kind(), kind);
            assert_eq!(member, Some(Property::OffsetPath));
            assert!(writer.inverse_source_tracking.is_none());
            assert!(writer.inverse_source_member.is_none());
            assert!(!writer.context.output_suppressed());
            let mut retry = SpecifiedRuleWriter::new(exact);
            let (result, member) = retry.with_inverse_sources(Property::Offset, None, |writer| {
                offset.append_to_rule_writer(writer)
            });
            result.unwrap();
            assert_eq!(member, Some(Property::OffsetPath));
            assert_eq!(retry.css, "inset(1px round 2px)");
        }
        // An error inside real suppression restores both modes. An ordinary
        // pair also uses its normal bounded writer without schema notifications.
        // The aggregate takes the first node and the first value the second; the second value fails.
        let mut writer = SpecifiedRuleWriter::new(Limits::new(2, usize::MAX, 0));
        let (result, member) = writer.with_inverse_sources(Property::MarginBlock, None, |writer| {
            writer.without_output(|writer| pair.append_to_rule_writer(writer))
        });
        assert_eq!(result.unwrap_err().kind(), Kind::InputNodeLimit);
        assert_eq!(member, Some(Property::MarginBlockEnd));
        assert!(writer.inverse_source_tracking.is_none());
        assert!(writer.inverse_source_member.is_none());
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        let mut ordinary = SpecifiedRuleWriter::new(Limits::new(3, 3, 7));
        ordinary
            .without_output(|writer| pair.append_to_rule_writer(writer))
            .unwrap();
        assert!(!ordinary.context.output_suppressed());
        assert!(ordinary.inverse_source_tracking.is_none());
        assert!(ordinary.inverse_source_member.is_none());
        assert!(ordinary.css.is_empty());
    }

    #[test]
    fn captured_scroll_values_reselect_the_role_whose_bytes_are_appended() {
        for (source, expected, bytes, final_member) in [
            (
                "scroll-padding-block:1px 2px",
                "1px 2px",
                7,
                Property::ScrollPaddingBlockEnd,
            ),
            (
                "scroll-padding-block:1px 1px",
                "1px",
                3,
                Property::ScrollPaddingBlockStart,
            ),
        ] {
            let report = crate::CssParser::default().parse_block_contents(source);
            assert!(report.is_clean(), "{report:?}");
            let Value::ScrollPaddingBlock(pair) = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("scroll pair")
            };
            let pair = pair.value();
            // The aggregate and both authored literals are charged: I3/P3. An equal end is charged
            // but never appended; the final responsible source is reselected to
            // the start's bytes.
            let exact = Limits::new(3, 3, bytes);
            assert_eq!(pair.serialize_specified(exact).unwrap(), expected);
            let mut writer = SpecifiedRuleWriter::new(exact);
            let (result, member) =
                writer.with_inverse_sources(Property::ScrollPaddingBlock, None, |writer| {
                    pair.append_to_rule_writer(writer)
                });
            result.unwrap();
            assert_eq!(writer.css, expected);
            assert_eq!(member, Some(final_member));
            for (limits, kind) in [
                (Limits::new(2, 3, bytes), Kind::InputNodeLimit),
                (Limits::new(3, 2, bytes), Kind::ProjectionNodeLimit),
            ] {
                let mut writer = SpecifiedRuleWriter::new(limits);
                let (result, member) =
                    writer.with_inverse_sources(Property::ScrollPaddingBlock, None, |writer| {
                        pair.append_to_rule_writer(writer)
                    });
                assert_eq!(result.unwrap_err().kind(), kind);
                assert_eq!(member, Some(Property::ScrollPaddingBlockEnd));
            }
            assert_eq!(pair.serialize_specified(exact).unwrap(), expected);
        }
        let report =
            crate::CssParser::default().parse_block_contents("scroll-padding-block:1px 2px");
        let Value::ScrollPaddingBlock(pair) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("scroll pair")
        };
        let pair = pair.value();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(3, 3, 6));
        let (result, member) =
            writer.with_inverse_sources(Property::ScrollPaddingBlock, None, |writer| {
                pair.append_to_rule_writer(writer)
            });
        assert_eq!(result.unwrap_err().kind(), Kind::ByteLimit);
        assert_eq!(member, Some(Property::ScrollPaddingBlockEnd));
        assert_eq!(writer.css, "1px ");
    }
}
