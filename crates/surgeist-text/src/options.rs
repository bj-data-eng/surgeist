use super::{
    Error, ErrorCode, ErrorDetail, NumericRequirement, Result, TextStyleFeature,
    UnsupportedTextStyleReason,
};

/// Paragraph-level layout options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    pub width: Option<f64>,
    pub scale: f32,
    pub alignment: Alignment,
    pub text_overflow: TextOverflow,
    pub text_align_last: TextAlignLast,
    pub indent: Indent,
    pub quantize: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: None,
            scale: 1.0,
            alignment: Alignment::Start,
            text_overflow: TextOverflow::Clip,
            text_align_last: TextAlignLast::Auto,
            indent: Indent::default(),
            quantize: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Alignment {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextOverflow {
    Clip,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextAlignLast {
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Indent {
    pub amount: f64,
    pub first_line: bool,
    pub each_line: bool,
    pub hanging: bool,
}

impl Default for Indent {
    fn default() -> Self {
        Self {
            amount: 0.0,
            first_line: true,
            each_line: false,
            hanging: false,
        }
    }
}

/// Intrinsically valid standalone layout options.
#[derive(Clone, Copy, Debug)]
pub struct ValidatedOptions {
    authored: Options,
}

impl ValidatedOptions {
    fn new(authored: Options) -> Self {
        Self { authored }
    }

    #[must_use]
    pub const fn authored(self) -> Options {
        self.authored
    }

    #[must_use]
    pub const fn indent(self) -> Indent {
        self.authored.indent
    }

    #[must_use]
    pub const fn text_overflow(self) -> TextOverflow {
        self.authored.text_overflow
    }

    #[must_use]
    pub const fn text_align_last(self) -> TextAlignLast {
        self.authored.text_align_last
    }
}

impl TryFrom<Options> for ValidatedOptions {
    type Error = Error;

    fn try_from(options: Options) -> Result<Self> {
        validate_options(&options)?;
        validate_indent(options.indent)?;
        Ok(Self::new(options))
    }
}

fn validate_options(options: &Options) -> Result<()> {
    validate_positive_f32(options.scale, "text scale")?;
    if let Some(width) = options.width {
        validate_non_negative_f64(width, "layout width")?;
    }
    validate_finite_f64(options.indent.amount, "text indent")?;
    Ok(())
}

fn validate_positive_f32(value: f32, name: &str) -> Result<()> {
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            format!("{name} must be finite and greater than 0"),
        )
        .with_detail(ErrorDetail::InvalidNumericField {
            field: numeric_field(name),
            value: f64::from(value),
            requirement: NumericRequirement::FiniteGreaterThanZero,
        }));
    }
    Ok(())
}

fn validate_non_negative_f64(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() || value < 0.0 {
        return Err(Error::new(
            ErrorCode::InvalidStyle,
            format!("{name} must be finite and non-negative"),
        )
        .with_detail(ErrorDetail::InvalidNumericField {
            field: numeric_field(name),
            value,
            requirement: NumericRequirement::FiniteNonNegative,
        }));
    }
    Ok(())
}

fn validate_finite_f64(value: f64, name: &str) -> Result<()> {
    if !value.is_finite() {
        return Err(
            Error::new(ErrorCode::InvalidStyle, format!("{name} must be finite")).with_detail(
                ErrorDetail::InvalidNumericField {
                    field: numeric_field(name),
                    value,
                    requirement: NumericRequirement::Finite,
                },
            ),
        );
    }
    Ok(())
}

fn numeric_field(name: &str) -> &'static str {
    match name {
        "text scale" => "text scale",
        "layout width" => "layout width",
        "text indent" => "text indent",
        _ => "numeric options field",
    }
}

fn validate_indent(indent: Indent) -> Result<()> {
    if indent.amount != 0.0 && !indent.first_line && indent.each_line && !indent.hanging {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "each-line indent without first-line indent requires a selected standalone policy",
        )
        .with_detail(ErrorDetail::UnsupportedTextStyle {
            feature: TextStyleFeature::TextIndent,
            reason: UnsupportedTextStyleReason::RequiresStandaloneIndentPolicy,
        }));
    }
    Ok(())
}
