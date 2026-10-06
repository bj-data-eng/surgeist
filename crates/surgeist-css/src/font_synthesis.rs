//! Authored font-synthesis values, intrinsic projection, and specified emission.
//! Font matching and the execution of synthesis belong to downstream owners.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssSpecifiedValueSerializationError as Error, CssSpecifiedValueSerializationLimits as Limits,
};

type Result<T> = std::result::Result<T, Error>;

/// A nonempty checked set of `font-synthesis` capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontSynthesisValues {
    weight: bool,
    style: bool,
    small_caps: bool,
    position: bool,
}

impl CssFontSynthesisValues {
    #[must_use]
    /// Checks a nonempty set in weight/style/small-caps/position order.
    pub const fn try_new(
        weight: bool,
        style: bool,
        small_caps: bool,
        position: bool,
    ) -> Option<Self> {
        if weight || style || small_caps || position {
            Some(Self {
                weight,
                style,
                small_caps,
                position,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn weight(self) -> bool {
        self.weight
    }

    #[must_use]
    pub const fn style(self) -> bool {
        self.style
    }

    #[must_use]
    pub const fn small_caps(self) -> bool {
        self.small_caps
    }

    #[must_use]
    pub const fn position(self) -> bool {
        self.position
    }
}

/// The current authored `font-synthesis` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontSynthesis {
    None,
    Values(CssFontSynthesisValues),
}

/// The authored `font-synthesis-weight` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontSynthesisWeight {
    Auto,
    None,
}

/// The authored `font-synthesis-style` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontSynthesisStyle {
    Auto,
    None,
    /// Allows oblique synthesis without using it as an italic fallback.
    ObliqueOnly,
}

/// The authored `font-synthesis-small-caps` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontSynthesisSmallCaps {
    Auto,
    None,
}

/// The authored `font-synthesis-position` choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontSynthesisPosition {
    Auto,
    None,
}

impl CssFontSynthesis {
    pub(crate) const fn expanded_weight(self) -> CssFontSynthesisWeight {
        match self {
            Self::Values(values) if values.weight() => CssFontSynthesisWeight::Auto,
            _ => CssFontSynthesisWeight::None,
        }
    }

    pub(crate) const fn expanded_style(self) -> CssFontSynthesisStyle {
        match self {
            Self::Values(values) if values.style() => CssFontSynthesisStyle::Auto,
            _ => CssFontSynthesisStyle::None,
        }
    }

    pub(crate) const fn expanded_small_caps(self) -> CssFontSynthesisSmallCaps {
        match self {
            Self::Values(values) if values.small_caps() => CssFontSynthesisSmallCaps::Auto,
            _ => CssFontSynthesisSmallCaps::None,
        }
    }

    pub(crate) const fn expanded_position(self) -> CssFontSynthesisPosition {
        match self {
            Self::Values(values) if values.position() => CssFontSynthesisPosition::Auto,
            _ => CssFontSynthesisPosition::None,
        }
    }

    fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
        match self {
            Self::None => word(writer, start, "none"),
            Self::Values(values) => {
                for (index, (selected, keyword)) in [
                    (values.weight(), "weight"),
                    (values.style(), "style"),
                    (values.small_caps(), "small-caps"),
                    (values.position(), "position"),
                ]
                .into_iter()
                .enumerate()
                {
                    if selected {
                        writer.source_member(index, |writer| word(writer, start, keyword))?;
                    }
                }
                Ok(())
            }
        }
    }
}

fn word(writer: &mut SpecifiedRuleWriter, start: usize, value: &str) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    if writer.css.len() != start {
        writer.append(" ")?;
    }
    writer.append(value)
}

macro_rules! keyword_writer {
    ($value:ty, $($variant:ident => $keyword:literal),+ $(,)?) => {
        impl $value {
            fn write(&self, writer: &mut SpecifiedRuleWriter, start: usize) -> Result<()> {
                word(writer, start, match self { $(Self::$variant => $keyword),+ })
            }
        }
    };
}

keyword_writer!(CssFontSynthesisWeight, Auto => "auto", None => "none");
keyword_writer!(CssFontSynthesisStyle, Auto => "auto", None => "none", ObliqueOnly => "oblique-only");
keyword_writer!(CssFontSynthesisSmallCaps, Auto => "auto", None => "none");
keyword_writer!(CssFontSynthesisPosition, Auto => "auto", None => "none");

macro_rules! public_serializer {
    ($value:ty) => {
        impl $value {
            /// Canonical specified CSS for this checked authored value.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(Limits::default())
            }

            /// Serializes under a cumulative input, projection, and output budget.
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

public_serializer!(CssFontSynthesis);
public_serializer!(CssFontSynthesisWeight);
public_serializer!(CssFontSynthesisStyle);
public_serializer!(CssFontSynthesisSmallCaps);
public_serializer!(CssFontSynthesisPosition);
