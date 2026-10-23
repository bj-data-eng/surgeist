//! Bounded authored font sources, without resource resolution or support selection.

use crate::{
    CssFontFaceSource, CssFontFaceSourceList, CssFontFaceUrlSource, CssFontFormat,
    CssFontFormatHint, CssFontLocalName, CssFontTechHint, CssSerializationError,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssFontLocalName {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.nodes(2)?;
        writer.append("local(")?;
        writer.append_string(self.as_str())?;
        writer.append(")")
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontLocalName {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontLocalName);

impl CssFontFaceUrlSource {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        self.url().append_specified(writer)?;
        if let Some(format) = self.format() {
            writer.node()?;
            writer.append(" format(")?;
            match format {
                CssFontFormat::Keyword(value) => writer.append(format_keyword(*value))?,
                CssFontFormat::String(value) => writer.append_string(value.as_str())?,
            }
            writer.append(")")?;
        }
        if !self.tech().is_empty() {
            writer.node()?;
            writer.append(" tech(")?;
            for (index, tech) in self.tech().iter().enumerate() {
                writer.node()?;
                if index != 0 {
                    writer.append(", ")?;
                }
                writer.append(tech_keyword(*tech))?;
            }
            writer.append(")")?;
        }
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFaceUrlSource {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFaceUrlSource);

impl CssFontFaceSource {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::Local(value) => value.append_specified(writer),
            Self::Url(value) => value.append_specified(writer),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFaceSource {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFaceSource);

impl CssFontFaceSourceList {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.comma_separated(self.sources(), |writer, source| {
            source.append_specified(writer)
        })?;
        Ok(())
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssFontFaceSourceList {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(writer)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssFontFaceSourceList);

fn format_keyword(value: CssFontFormatHint) -> &'static str {
    match value {
        CssFontFormatHint::Woff => "woff",
        CssFontFormatHint::Woff2 => "woff2",
        CssFontFormatHint::TrueType => "truetype",
        CssFontFormatHint::OpenType => "opentype",
        CssFontFormatHint::Collection => "collection",
        CssFontFormatHint::EmbeddedOpenType => "embedded-opentype",
        CssFontFormatHint::Svg => "svg",
    }
}

fn tech_keyword(value: CssFontTechHint) -> &'static str {
    match value {
        CssFontTechHint::Variations => "variations",
        CssFontTechHint::Palettes => "palettes",
        CssFontTechHint::ColorCOLRv0 => "color-colrv0",
        CssFontTechHint::ColorCOLRv1 => "color-colrv1",
        CssFontTechHint::ColorSVG => "color-svg",
        CssFontTechHint::ColorSbix => "color-sbix",
        CssFontTechHint::ColorCBDT => "color-cbdt",
        CssFontTechHint::FeaturesOpenType => "features-opentype",
        CssFontTechHint::FeaturesAAT => "features-aat",
        CssFontTechHint::FeaturesGraphite => "features-graphite",
        CssFontTechHint::Incremental => "incremental",
    }
}
