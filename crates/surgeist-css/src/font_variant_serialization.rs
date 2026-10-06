//! Bounded canonical specified serialization of the authored variant family.

use crate::font_variant::*;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssSpecifiedValueSerializationError as Error, CssSpecifiedValueSerializationLimits as Limits,
};

type Result<T> = std::result::Result<T, Error>;

fn word(writer: &mut SpecifiedRuleWriter, start: usize, value: &str) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    if writer.css.len() != start && !writer.css.ends_with('(') {
        writer.append(" ")?;
    }
    writer.append(value)
}

fn name(writer: &mut SpecifiedRuleWriter, value: &crate::CssFontFeatureValueName) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    writer.append_identifier(value.as_str())
}

fn named_function(
    writer: &mut SpecifiedRuleWriter,
    start: usize,
    function: &str,
    names: &[crate::CssFontFeatureValueName],
) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    if writer.css.len() != start {
        writer.append(" ")?;
    }
    writer.append(function)?;
    writer.append("(")?;
    for (index, item) in names.iter().enumerate() {
        if index != 0 {
            writer.append(", ")?;
        }
        name(writer, item)?;
    }
    writer.append(")")
}

impl CssFontVariantLigatures {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        match self {
            Self::Normal => word(writer, start, "normal"),
            Self::None => word(writer, start, "none"),
            Self::Values(values) => {
                for (choice, positive, negative) in [
                    (values.common(), "common-ligatures", "no-common-ligatures"),
                    (
                        values.discretionary(),
                        "discretionary-ligatures",
                        "no-discretionary-ligatures",
                    ),
                    (
                        values.historical(),
                        "historical-ligatures",
                        "no-historical-ligatures",
                    ),
                    (values.contextual(), "contextual", "no-contextual"),
                ] {
                    if let Some(choice) = choice {
                        word(
                            writer,
                            start,
                            if choice == CssFontVariantLigatureState::Enabled {
                                positive
                            } else {
                                negative
                            },
                        )?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl CssFontVariantCaps {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        word(
            writer,
            start,
            match self {
                Self::Normal => "normal",
                Self::SmallCaps => "small-caps",
                Self::AllSmallCaps => "all-small-caps",
                Self::PetiteCaps => "petite-caps",
                Self::AllPetiteCaps => "all-petite-caps",
                Self::Unicase => "unicase",
                Self::TitlingCaps => "titling-caps",
            },
        )
    }
}

impl CssFontVariantAlternates {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        match self {
            Self::Normal => word(writer, start, "normal"),
            Self::Values(values) => values.write(writer, start),
        }
    }
}

impl CssFontVariantAlternateValues {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        let values = self;
        if let Some(value) = values.stylistic() {
            named_function(writer, start, "stylistic", std::slice::from_ref(value))?;
        }
        if values.historical_forms() {
            word(writer, start, "historical-forms")?;
        }
        if let Some(values) = values.styleset() {
            named_function(writer, start, "styleset", values.names())?;
        }
        if let Some(values) = values.character_variant() {
            named_function(writer, start, "character-variant", values.names())?;
        }
        if let Some(value) = values.swash() {
            named_function(writer, start, "swash", std::slice::from_ref(value))?;
        }
        if let Some(value) = values.ornaments() {
            named_function(writer, start, "ornaments", std::slice::from_ref(value))?;
        }
        if let Some(value) = values.annotation() {
            named_function(writer, start, "annotation", std::slice::from_ref(value))?;
        }
        Ok(())
    }
}

impl CssFontVariantNumeric {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        match self {
            Self::Normal => word(writer, start, "normal"),
            Self::Values(values) => {
                if let Some(value) = values.figure() {
                    word(
                        writer,
                        start,
                        match value {
                            CssFontVariantNumericFigure::LiningNums => "lining-nums",
                            CssFontVariantNumericFigure::OldstyleNums => "oldstyle-nums",
                        },
                    )?;
                }
                if let Some(value) = values.spacing() {
                    word(
                        writer,
                        start,
                        match value {
                            CssFontVariantNumericSpacing::ProportionalNums => "proportional-nums",
                            CssFontVariantNumericSpacing::TabularNums => "tabular-nums",
                        },
                    )?;
                }
                if let Some(value) = values.fraction() {
                    word(
                        writer,
                        start,
                        match value {
                            CssFontVariantNumericFraction::DiagonalFractions => {
                                "diagonal-fractions"
                            }
                            CssFontVariantNumericFraction::StackedFractions => "stacked-fractions",
                        },
                    )?;
                }
                if values.ordinal() {
                    word(writer, start, "ordinal")?;
                }
                if values.slashed_zero() {
                    word(writer, start, "slashed-zero")?;
                }
                Ok(())
            }
        }
    }
}

impl CssFontVariantEastAsian {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        match self {
            Self::Normal => word(writer, start, "normal"),
            Self::Values(values) => {
                if let Some(value) = values.variant() {
                    word(
                        writer,
                        start,
                        match value {
                            CssFontVariantEastAsianVariant::Jis78 => "jis78",
                            CssFontVariantEastAsianVariant::Jis83 => "jis83",
                            CssFontVariantEastAsianVariant::Jis90 => "jis90",
                            CssFontVariantEastAsianVariant::Jis04 => "jis04",
                            CssFontVariantEastAsianVariant::Simplified => "simplified",
                            CssFontVariantEastAsianVariant::Traditional => "traditional",
                        },
                    )?;
                }
                if let Some(value) = values.width() {
                    word(
                        writer,
                        start,
                        match value {
                            CssFontVariantEastAsianWidth::FullWidth => "full-width",
                            CssFontVariantEastAsianWidth::ProportionalWidth => "proportional-width",
                        },
                    )?;
                }
                if values.ruby() {
                    word(writer, start, "ruby")?;
                }
                Ok(())
            }
        }
    }
}

impl CssFontVariantPosition {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        word(
            writer,
            start,
            match self {
                Self::Normal => "normal",
                Self::Sub => "sub",
                Self::Super => "super",
            },
        )
    }
}

impl CssFontVariantEmoji {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        word(
            writer,
            start,
            match self {
                Self::Normal => "normal",
                Self::Text => "text",
                Self::Emoji => "emoji",
                Self::Unicode => "unicode",
            },
        )
    }
}

impl CssFontVariantValue {
    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        match self {
            Self::Normal => word(writer, start, "normal"),
            Self::None => word(writer, start, "none"),
            Self::Values(values) => {
                if let Some(value) = values.ligatures() {
                    writer.source_property(
                        crate::CssKnownProperty::FontVariantLigatures,
                        |writer| CssFontVariantLigatures::Values(*value).write(writer, start),
                    )?;
                }
                if let Some(value) = values.caps() {
                    writer.source_property(crate::CssKnownProperty::FontVariantCaps, |writer| {
                        value.write(writer, start)
                    })?;
                }
                if let Some(value) = values.alternates() {
                    writer.source_property(
                        crate::CssKnownProperty::FontVariantAlternates,
                        |writer| value.write(writer, start),
                    )?;
                }
                if let Some(value) = values.numeric() {
                    writer
                        .source_property(crate::CssKnownProperty::FontVariantNumeric, |writer| {
                            CssFontVariantNumeric::Values(*value).write(writer, start)
                        })?;
                }
                if let Some(value) = values.east_asian() {
                    writer.source_property(
                        crate::CssKnownProperty::FontVariantEastAsian,
                        |writer| CssFontVariantEastAsian::Values(*value).write(writer, start),
                    )?;
                }
                if let Some(value) = values.position() {
                    writer.source_property(
                        crate::CssKnownProperty::FontVariantPosition,
                        |writer| value.write(writer, start),
                    )?;
                }
                if let Some(value) = values.emoji() {
                    writer
                        .source_property(crate::CssKnownProperty::FontVariantEmoji, |writer| {
                            value.write(writer, start)
                        })?;
                }
                Ok(())
            }
        }
    }
}

macro_rules! public_serializer {
    ($value:ty) => {
        impl $value {
            /// Canonical specified CSS for this checked authored value.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(Limits::default())
            }
            /// Serializes under a shared input, projection, and output budget.
            pub fn serialize_specified_with_limits(&self, limits: Limits) -> Result<String> {
                let mut writer =
                    crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }

            pub(crate) fn append_to_rule_writer(
                &self,
                writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
                let start = writer.css.len();
                self.write(writer, start)?;
                Ok(())
            }
        }
    };
}

public_serializer!(CssFontVariantLigatures);
public_serializer!(CssFontVariantCaps);
public_serializer!(CssFontVariantAlternates);
public_serializer!(CssFontVariantNumeric);
public_serializer!(CssFontVariantEastAsian);
public_serializer!(CssFontVariantPosition);
public_serializer!(CssFontVariantEmoji);
public_serializer!(CssFontVariantValue);
