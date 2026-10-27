//! Canonical text for authored declarations and ordered occurrence lists.
//!
//! This owner preserves the declaration's grammar identity and visits semantic
//! value providers. Occurrence lists retain every duplicate and importance flag;
//! they do not perform CSSOM declaration-block selection or shorthand coalescing.

use crate::component::values::serialize::CssCanonicalBuilder;
use crate::component::{CssComponentValueRef, CssComponentValues, CssValueTokenRef};
use crate::declaration::{
    CssDeclaration, CssDeclarationBody, CssDeclarationList, CssGlobalKeyword, CssImportance,
};
use crate::property::properties::{
    CssLegacyPropertyAlias, CssResolvedPropertyName, property_schema,
};
use crate::property::{CssKnownDeclaredValueRef, CssKnownPropertyValueRef, CssPropertyGrammar};
use crate::serialization::CssSerializationError;
use crate::serialization::pending_serialization::append_pending_specified;
use crate::serialization::specified_rule_serialization::SpecifiedProvider as _;
use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
use crate::values::box_model::CssBreakBetween;
use crate::values::text::CssTextOrientation;

type Result<T> = std::result::Result<T, CssSerializationError>;

// Only the owners requiring declaration context differ from the common bridge.
// The schema supplies every ordinary arm, type and accessor exactly once.
macro_rules! append_ordinary {
    (CounterReset, $ty:ident, $value:expr, $grammar:ident, $writer:ident) => {
        $value.append_to_rule_writer(crate::values::content::CssCounterProperty::Reset, $writer)
    };
    (CounterIncrement, $ty:ident, $value:expr, $grammar:ident, $writer:ident) => {
        $value.append_to_rule_writer(crate::values::content::CssCounterProperty::Increment, $writer)
    };
    (CounterSet, $ty:ident, $value:expr, $grammar:ident, $writer:ident) => {
        $value.append_to_rule_writer(crate::values::content::CssCounterProperty::Set, $writer)
    };
    ($variant:ident, CssBreakBetween, $value:expr, $grammar:ident, $writer:ident) => {
        append_break_between($value, $grammar, $writer)
    };
    ($variant:ident, CssTextOrientation, $value:expr, $grammar:ident, $writer:ident) => {
        append_text_orientation($value, $grammar, $writer)
    };
    ($variant:ident, $ty:ident, $value:expr, $grammar:ident, $writer:ident) => {
        crate::serialization::specified_rule_serialization::SpecifiedProvider::append_to_rule_writer(
            $value, $writer,
        )
    };
}

pub(crate) use append_ordinary;

macro_rules! define_ordinary_dispatch {
    ($input:ident, $numeric:ident;
        All, $all_name:literal, [$($all_alias:literal),*], $all_id:literal,
        $all_value:ty, $all_parser:ident, $all_dispatch:block
        $(, expansion = $all_expansion:ident { $($all_metadata:tt)* })?;
        $(
            $variant:ident, $name:literal, [$($alias:literal),*], $id:literal,
            $value:ident $(<$($argument:ident),+>)?, $parser:ident, $dispatch:block
            $(, expansion = $expansion:ident { $($metadata:tt)* })?;
        )*
    ) => {
        fn append_property_value(
            value: CssKnownPropertyValueRef<'_>,
            grammar: CssPropertyGrammar,
            writer: &mut SpecifiedRuleWriter,
        ) -> Result<()> {
            match value {
                $(CssKnownPropertyValueRef::$variant(value) =>
                    append_ordinary!($variant, $value, value.value(), grammar, writer),)*
            }
        }
    };
}

property_schema!(
    define_ordinary_dispatch,
    declaration_input,
    declaration_numeric
);

pub(crate) fn append_break_between(
    value: &CssBreakBetween,
    grammar: CssPropertyGrammar,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    if matches!(
        grammar.resolved(),
        CssResolvedPropertyName::LegacyShorthand(
            CssLegacyPropertyAlias::PageBreakBefore | CssLegacyPropertyAlias::PageBreakAfter
        )
    ) && *value == CssBreakBetween::Page
    {
        // Break 3 §3.4 maps the legacy `always` spelling to modern `page`.
        writer.without_output(|writer| value.append_to_rule_writer(writer))?;
        writer.append("always")
    } else {
        value.append_to_rule_writer(writer)
    }
}

pub(crate) fn append_text_orientation(
    value: &CssTextOrientation,
    grammar: CssPropertyGrammar,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    if !matches!(
        grammar.resolved(),
        CssResolvedPropertyName::LegacyShorthand(CssLegacyPropertyAlias::GlyphOrientationVertical)
    ) {
        return value.append_to_rule_writer(writer);
    }
    // Writing Modes 3 §5.1.3 supplies the inverse semantic representation.
    // Visit the modern owner even when its spelling cannot enter this grammar.
    writer.without_output(|writer| value.append_to_rule_writer(writer))?;
    let text = match value {
        CssTextOrientation::Mixed => "auto",
        CssTextOrientation::Upright => {
            writer.context.charge_projection(1)?;
            "0deg"
        }
        CssTextOrientation::Sideways => {
            writer.context.charge_projection(1)?;
            "90deg"
        }
    };
    writer.append(text)
}

pub(crate) fn append_global(
    keyword: CssGlobalKeyword,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.keyword(match keyword {
        CssGlobalKeyword::Inherit => "inherit",
        CssGlobalKeyword::Initial => "initial",
        CssGlobalKeyword::Unset => "unset",
        CssGlobalKeyword::Revert => "revert",
        CssGlobalKeyword::RevertLayer => "revert-layer",
    })
}

pub(crate) fn append_retained_value(
    values: &CssComponentValues,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    let whitespace = |value: &crate::component::CssComponentValue| {
        matches!(
            value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
        )
    };
    let items = values.items();
    let start = items
        .iter()
        .position(|value| !whitespace(value))
        .unwrap_or(items.len());
    let end = items
        .iter()
        .rposition(|value| !whitespace(value))
        .map_or(start, |index| index + 1);
    if start == 0 && end == items.len() {
        return append_pending_specified(values, &mut writer.context, &mut writer.css);
    }
    // Syntax 3 declaration consumption trims only top-level edge whitespace.
    // The complete retained region is still visited and charged by its existing
    // owner, including the trivia omitted from output. No visits are refunded.
    writer.without_output(|writer| {
        append_pending_specified(values, &mut writer.context, &mut writer.css)
    })?;
    if writer.context.output_suppressed() {
        return Ok(());
    }
    let mut builder = CssCanonicalBuilder::new(writer.context.remaining_bytes());
    let serialized = builder
        .push_components(&items[start..end])
        .and_then(|()| builder.finish())
        .map_err(crate::component::values::specified_component_error)?;
    writer.append(serialized.as_css())
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssDeclaration {
    fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        append_authored_declaration(
            self.body(),
            self.value_components(),
            self.importance(),
            writer,
        )
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssDeclaration);

/// Shared authored body/value/punctuation owner for ordinary and keyframe
/// occurrences. Keyframe grammar supplies normal importance; its private model
/// cannot represent an important declaration. No occurrence is selected away.
pub(crate) fn append_authored_declaration(
    body: &CssDeclarationBody,
    value_components: &CssComponentValues,
    importance: CssImportance,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.node()?;
    writer.node()?; // One semantic property name.
    match body {
        CssDeclarationBody::SvgGlyphOrientationVertical(value) => {
            writer.append("glyph-orientation-vertical: ")?;
            if let Some(value) = value.value() {
                value.append_to_rule_writer(writer)?;
            } else if let Some(keyword) = value.global() {
                append_global(keyword, writer)?;
            } else {
                append_retained_value(value_components, writer)?;
            }
        }
        CssDeclarationBody::Known(known) => {
            writer.append(known.grammar().name())?;
            writer.append(": ")?;
            match known.declared_value() {
                CssKnownDeclaredValueRef::Property(value) => {
                    append_property_value(value, known.grammar(), writer)?;
                }
                CssKnownDeclaredValueRef::Global(keyword) => append_global(keyword, writer)?,
                CssKnownDeclaredValueRef::SubstitutionDependent(_) => {
                    append_retained_value(value_components, writer)?;
                }
            }
        }
        CssDeclarationBody::Custom(custom) => {
            writer.append_identifier(custom.name().as_str())?;
            writer.append(": ")?;
            // Variables 1 §4.1 requires exact specified token spelling even
            // for the retained CSS-wide branch of a custom declaration.
            append_retained_value(value_components, writer)?;
        }
    }
    if importance == CssImportance::Important {
        writer.append(" !important")?;
    }
    writer.append(";")
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssDeclarationList {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.append_authored_declaration_list(self)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssDeclarationList);

impl SpecifiedRuleWriter {
    /// Composes retained authored occurrences in order, including duplicates.
    /// This private bridge deliberately does not implement CSSOM block selection.
    pub(crate) fn append_authored_declaration_list(
        &mut self,
        declarations: &CssDeclarationList,
    ) -> Result<()> {
        self.node()?;
        for (index, declaration) in declarations.iter().enumerate() {
            if index != 0 {
                self.append(" ")?;
            }
            declaration.append_to_rule_writer(self)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod occurrence_composition_tests {
    use super::*;
    use crate::serialization::CssSerializationLimits as Limits;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as Kind;

    fn declarations() -> CssDeclarationList {
        let report = crate::CssParser::default()
            .parse_block_contents("color:PURPLE!important;color:transparent");
        assert!(report.is_clean());
        report.syntax().clone()
    }

    #[test]
    fn authored_duplicates_and_prefix_share_one_cumulative_budget() {
        let list = declarations();
        let occurrences = list.as_slice().to_vec();
        let expected = "prefix color: purple !important; color: transparent;";
        let compose = |writer: &mut SpecifiedRuleWriter| {
            writer.append("prefix ")?;
            writer.append_authored_declaration_list(&list)
        };
        let mut writer = SpecifiedRuleWriter::new(Limits::new(7, 7, expected.len()));
        compose(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        for (limits, kind) in [
            (Limits::new(6, 7, expected.len()), Kind::InputNodeLimit),
            (Limits::new(7, 6, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(7, 7, expected.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(compose(&mut writer).unwrap_err().kind(), kind);
            for (original, current) in occurrences.iter().zip(list.iter()) {
                assert!(original.same_occurrence(current));
                assert_eq!(original.value_components(), current.value_components());
                assert_eq!(original.parsed_name(), current.parsed_name());
                assert_eq!(original.parsed_value(), current.parsed_value());
            }
        }
    }

    #[test]
    fn suppressed_lists_visit_all_siblings_and_restore_both_enclosing_modes() {
        let list = declarations();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(7, 7, 3));
        writer.append("[").unwrap();
        writer
            .without_output(|writer| {
                writer.without_output(|writer| writer.append_authored_declaration_list(&list))?;
                assert!(writer.context.output_suppressed());
                let error = writer
                    .without_output(|writer| list[0].append_to_rule_writer(writer))
                    .unwrap_err();
                assert_eq!(error.kind(), Kind::InputNodeLimit);
                assert!(writer.context.output_suppressed());
                Ok(())
            })
            .unwrap();
        assert!(!writer.context.output_suppressed());
        writer.append("]").unwrap();
        assert_eq!(writer.css, "[]");
        let error = writer
            .without_output(|writer| list[1].append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), Kind::InputNodeLimit);
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "[]x");
    }

    #[test]
    fn empty_occurrence_list_costs_one_aggregate_and_no_bytes() {
        let list = CssDeclarationList::new(Vec::new());
        let mut writer = SpecifiedRuleWriter::new(Limits::new(1, 1, 0));
        writer.append_authored_declaration_list(&list).unwrap();
        assert!(writer.css.is_empty());
        for (limits, kind) in [
            (Limits::new(0, 1, 0), Kind::InputNodeLimit),
            (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                writer
                    .append_authored_declaration_list(&list)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
    }

    #[test]
    fn suppressed_legacy_inverse_mapping_charges_generated_nodes_and_restores_output() {
        let declaration = crate::declaration::CssDeclaration::parse(
            &crate::CssParser::default(),
            "glyph-orientation-vertical:90",
        )
        .syntax()
        .as_ref()
        .unwrap()
        .clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(3, 4, 1));
        writer
            .without_output(|writer| declaration.append_to_rule_writer(writer))
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
        let mut writer = SpecifiedRuleWriter::new(Limits::new(3, 3, 1));
        assert_eq!(
            writer
                .without_output(|writer| declaration.append_to_rule_writer(writer))
                .unwrap_err()
                .kind(),
            Kind::ProjectionNodeLimit
        );
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }

    #[test]
    fn suppressed_retained_edge_trivia_charges_once_and_restores_after_failure() {
        let declaration = crate::declaration::CssDeclaration::parse(
            &crate::CssParser::default(),
            "--X: \tINITIAL \n",
        )
        .syntax()
        .as_ref()
        .unwrap()
        .clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(6, 6, 1));
        writer
            .without_output(|writer| declaration.append_to_rule_writer(writer))
            .unwrap();
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
        let mut writer = SpecifiedRuleWriter::new(Limits::new(6, 5, 1));
        assert_eq!(
            writer
                .without_output(|writer| declaration.append_to_rule_writer(writer))
                .unwrap_err()
                .kind(),
            Kind::ProjectionNodeLimit
        );
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(writer.css, "x");
    }
}
