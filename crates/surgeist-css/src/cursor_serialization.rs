//! Canonical specified UI 4 cursor text with one cumulative writer.
//!
//! Keywords cost one input and projection node. Image lists, entries, URL sets
//! and set options each cost one aggregate node. Source/descriptor enum and
//! hotspot-pair carriers are transparent. URL, Number and Resolution children
//! delegate their complete costs; a string reference costs one node and type()
//! costs its function plus string node. Punctuation costs only output bytes.

use crate::cursor_values::*;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssCursorKeyword, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssCursor {
    /// Emits canonical authored cursor syntax without loading resources or clamping hotspots.
    /// Candidate order and omission survive; set descriptors emit resolution before type.
    /// The standard prefixed image-set alias emits `image-set()`.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Shares cumulative child visits, numeric projection work and final UTF-8 bytes.
    /// Failure returns no partial text and leaves authored values and origins unchanged.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Keyword(value) => keyword(*value, writer),
            Self::Images(value) => {
                charge(writer)?;
                for image in value.images() {
                    charge(writer)?;
                    source(image.source(), writer)?;
                    if let Some([x, y]) = image.hotspot() {
                        writer.append(" ")?;
                        x.append_to_rule_writer(writer)?;
                        writer.append(" ")?;
                        y.append_to_rule_writer(writer)?;
                    }
                    writer.append(", ")?;
                }
                keyword(value.fallback(), writer)
            }
        }
    }
}

fn charge(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

fn source(source: &CssCursorImageSource, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    match source {
        CssCursorImageSource::Url(url) => url.append_to_rule_writer(writer),
        CssCursorImageSource::UrlSet(set) => {
            charge(writer)?;
            writer.append("image-set(")?;
            for (index, option) in set.options().iter().enumerate() {
                if index != 0 {
                    writer.append(", ")?;
                }
                charge(writer)?;
                match option.reference() {
                    CssCursorUrlSetReference::Url(url) => url.append_to_rule_writer(writer)?,
                    CssCursorUrlSetReference::String(value) => {
                        charge(writer)?;
                        writer.append_string(value.as_str())?;
                    }
                }
                let mut resolution = None;
                let mut image_type = None;
                for descriptor in option.descriptors() {
                    match descriptor {
                        CssCursorUrlSetDescriptor::Resolution(value) => resolution = Some(value),
                        CssCursorUrlSetDescriptor::Type(value) => image_type = Some(value),
                    }
                }
                if let Some(value) = resolution {
                    writer.append(" ")?;
                    value.append_to_rule_writer(writer)?;
                }
                if let Some(value) = image_type {
                    charge(writer)?;
                    writer.append(" type(")?;
                    charge(writer)?;
                    writer.append_string(value.as_str())?;
                    writer.append(")")?;
                }
            }
            writer.append(")")
        }
    }
}

fn keyword(value: CssCursorKeyword, writer: &mut SpecifiedRuleWriter) -> Result<()> {
    use CssCursorKeyword::*;
    writer.keyword(match value {
        Auto => "auto",
        Default => "default",
        None => "none",
        ContextMenu => "context-menu",
        Help => "help",
        Pointer => "pointer",
        Progress => "progress",
        Wait => "wait",
        Cell => "cell",
        Crosshair => "crosshair",
        Text => "text",
        VerticalText => "vertical-text",
        Alias => "alias",
        Copy => "copy",
        Move => "move",
        NoDrop => "no-drop",
        NotAllowed => "not-allowed",
        Grab => "grab",
        Grabbing => "grabbing",
        AllScroll => "all-scroll",
        ColResize => "col-resize",
        RowResize => "row-resize",
        NResize => "n-resize",
        EResize => "e-resize",
        SResize => "s-resize",
        WResize => "w-resize",
        NeResize => "ne-resize",
        NwResize => "nw-resize",
        SeResize => "se-resize",
        SwResize => "sw-resize",
        EwResize => "ew-resize",
        NsResize => "ns-resize",
        NeswResize => "nesw-resize",
        NwseResize => "nwse-resize",
        ZoomIn => "zoom-in",
        ZoomOut => "zoom-out",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as Kind;
    use crate::CssSpecifiedValueSerializationLimits as Limits;
    use crate::{
        CssContentString, CssNumberCalculation, CssSpecifiedNumber, CssUrl, parse_component_values,
    };

    fn number(value: &str) -> CssSpecifiedNumber {
        CssSpecifiedNumber::try_from_component(crate::CssComponentValue::try_token(value).unwrap())
            .unwrap()
    }

    fn value(x: CssSpecifiedNumber) -> CssCursor {
        CssCursor::try_images(
            vec![CssCursorImage::new(
                CssCursorImageSource::Url(CssUrl::new("cursor.cur")),
                Some([x, number("-0.25")]),
            )],
            CssCursorKeyword::Pointer,
        )
        .unwrap()
    }

    #[test]
    fn siblings_share_prefix_bytes_and_exact_aggregate_and_child_visits() {
        let value = value(number("3.000e1"));
        let expected =
            "prefix url(\"cursor.cur\") 30 -0.25, pointerurl(\"cursor.cur\") 30 -0.25, pointer";
        let mut writer = SpecifiedRuleWriter::new(Limits::new(14, 14, expected.len()));
        writer.append("prefix ").unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        value.append_to_rule_writer(&mut writer).unwrap();
        assert_eq!(writer.css, expected);
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("x").unwrap_err().kind(), Kind::ByteLimit);
    }

    #[test]
    fn prefixed_cursor_fails_at_each_shared_resource_boundary() {
        let value = value(number("30"));
        let expected = "prefix url(\"cursor.cur\") 30 -0.25, pointer";
        for (limits, kind) in [
            (Limits::new(6, 7, expected.len()), Kind::InputNodeLimit),
            (Limits::new(7, 6, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(7, 7, expected.len() - 1), Kind::ByteLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            writer.append("prefix ").unwrap();
            assert_eq!(
                value.append_to_rule_writer(&mut writer).unwrap_err().kind(),
                kind
            );
            assert!(writer.css.starts_with("prefix "));
            assert!(!writer.context.output_suppressed());
        }
    }

    #[test]
    fn suppressed_huge_number_spends_every_visit_without_output_scratch() {
        let value = value(number("1e999999999999999999999999"));
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(7, 7, 1));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(value, before);
    }

    #[test]
    fn suppressed_duplicate_options_and_type_children_spend_exactly_their_documented_cost() {
        let resolution = crate::CssResolutionValue::from_literal(
            crate::CssResolutionLiteral::try_new("1", crate::CssResolutionUnit::Dppx).unwrap(),
        );
        let option = CssCursorUrlSetOption::try_new(
            CssCursorUrlSetReference::String(CssContentString::try_new("cursor.png").unwrap()),
            vec![
                CssCursorUrlSetDescriptor::Type(
                    CssContentString::try_new("not a MIME type").unwrap(),
                ),
                CssCursorUrlSetDescriptor::Resolution(resolution),
            ],
        )
        .unwrap();
        let value = CssCursor::try_images(
            vec![CssCursorImage::new(
                CssCursorImageSource::UrlSet(
                    CssCursorUrlSet::try_new(vec![option.clone(), option]).unwrap(),
                ),
                None,
            )],
            CssCursorKeyword::Auto,
        )
        .unwrap();
        // List + image + set + fallback, plus (option + string + resolution +
        // type-function + type-string) for each of the two retained options.
        let mut writer = SpecifiedRuleWriter::new(Limits::new(14, 14, 1));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            Kind::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            Kind::ProjectionNodeLimit
        );
        assert_eq!(writer.append("y").unwrap_err().kind(), Kind::ByteLimit);
    }

    #[test]
    fn suppressed_set_visits_all_options_types_and_resolution_math_even_when_mime_is_unusable() {
        let resolution = crate::CssResolutionValue::try_from_calculation(
            crate::CssResolutionCalculation::try_from_components(
                parse_component_values("calc(-1x)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let option = CssCursorUrlSetOption::try_new(
            CssCursorUrlSetReference::String(CssContentString::try_new("cursor.png").unwrap()),
            vec![
                CssCursorUrlSetDescriptor::Type(
                    CssContentString::try_new("not a MIME type").unwrap(),
                ),
                CssCursorUrlSetDescriptor::Resolution(resolution),
            ],
        )
        .unwrap();
        let set = CssCursorUrlSet::try_new(vec![option.clone(), option]).unwrap();
        let value = CssCursor::try_images(
            vec![CssCursorImage::new(CssCursorImageSource::UrlSet(set), None)],
            CssCursorKeyword::Auto,
        )
        .unwrap();
        let before = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100, 100, 1));
        writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap();
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
        writer.append("x").unwrap();
        for (limits, kind) in [
            (Limits::new(0, 100, 1), Kind::InputNodeLimit),
            (Limits::new(100, 0, 1), Kind::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            assert_eq!(
                writer
                    .without_output(|writer| value.append_to_rule_writer(writer))
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert!(!writer.context.output_suppressed());
            writer.append("x").unwrap();
        }
        assert_eq!(value, before);
    }

    #[test]
    fn nested_suppression_preserves_enclosing_mode_on_success_and_failure() {
        let value = value(number("30"));
        for (limits, expected) in [
            (Limits::new(7, 7, 1), None),
            (Limits::new(6, 7, 1), Some(Kind::InputNodeLimit)),
            (Limits::new(7, 6, 1), Some(Kind::ProjectionNodeLimit)),
        ] {
            let mut writer = SpecifiedRuleWriter::new(limits);
            let previous = writer.context.replace_output_suppression(true);
            let result = writer.without_output(|writer| value.append_to_rule_writer(writer));
            assert_eq!(result.err().map(|error| error.kind()), expected);
            assert!(writer.context.output_suppressed());
            assert!(writer.css.is_empty());
            writer.context.replace_output_suppression(previous);
            writer.append("x").unwrap();
        }
    }

    #[test]
    fn suppressed_number_math_enforces_the_child_projection_limit_and_preserves_original_origin() {
        let components = parse_component_values("calc(1 + 2)").unwrap();
        let number = CssSpecifiedNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(components).unwrap(),
        )
        .unwrap();
        let value = value(number);
        let before = value.clone();
        // The list, entry and URL consume four projections before Number math.
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100, 4, 1));
        let error = writer
            .without_output(|writer| value.append_to_rule_writer(writer))
            .unwrap_err();
        assert_eq!(error.kind(), Kind::ProjectionNodeLimit);
        assert!(!writer.context.output_suppressed());
        assert!(writer.css.is_empty());
        assert_eq!(value, before);
    }
}
