//! Native hyphenation policy and indication inputs with bounded en-US resources.

use crate::{Error, ErrorCode, ErrorDetail, Result, Text};

/// Whether authored and resource-qualified automatic hyphenation points may be used.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Hyphens {
    /// Suppress genuine hyphenation points, retaining independent breaks.
    None,
    /// Use actual authored SHY/discretionary controls.
    Manual,
    /// Also permits the pinned en-US resource over a complete qualified word.
    /// Other languages/regions/variants receive no implicit coverage.
    Auto,
}

/// One computed character-count minimum. It is not CSS specified-value syntax.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum HyphenationMinimum {
    /// Native preparation uses the independent fallback for this component.
    #[default]
    Auto,
    /// An exact nonnegative minimum, including zero.
    Count(usize),
}

impl HyphenationMinimum {
    const fn resolve(self, fallback: usize) -> usize {
        match self {
            Self::Auto => fallback,
            Self::Count(value) => value,
        }
    }
}

/// Computed total/before/after character minima at the governing cut's Style.
///
/// The supplying style adapter owns CSS specified-value expansion and checked
/// integer conversion. Every triple is valid independently; Text does not add or
/// clamp these values. Native `Auto` components resolve independently to 5/2/2.
/// Raw standalone layout retains its genuinely unconstrained count operation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct HyphenateLimitChars {
    pub total: HyphenationMinimum,
    pub before: HyphenationMinimum,
    pub after: HyphenationMinimum,
}

impl HyphenateLimitChars {
    pub(crate) const fn native_minima(self) -> [usize; 3] {
        [
            self.total.resolve(5),
            self.before.resolve(2),
            self.after.resolve(2),
        ]
    }
}

/// Exact indication text, or resolution from captured native convention/font.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HyphenateCharacter {
    Auto,
    String(Text),
}

/// Captured native indication placement relative to logical source order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HyphenationPlacement {
    LogicalBefore,
    LogicalAfter,
    BothSides,
}

/// A native producer's explicit convention claim. This is not a language table.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NativeHyphenationConvention {
    locale: Option<String>,
    script: String,
    automatic_text: Text,
    placement: HyphenationPlacement,
}

/// Invalid input, missing resource, or unsupported indication placement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HyphenationError {
    InvalidResourceMapping,
    ResourceLengthOverflow,
    ResourceStorage,
    InvalidConvention,
    InvalidConventionBinding,
    EmptyNativeAlternatives,
    MissingPrimaryFont,
    UnsupportedPlacement(HyphenationPlacement),
}

pub(crate) fn error(reason: HyphenationError) -> Error {
    Error::new(
        match reason {
            HyphenationError::InvalidResourceMapping
            | HyphenationError::ResourceStorage
            | HyphenationError::ResourceLengthOverflow => ErrorCode::LayoutFailed,
            HyphenationError::MissingPrimaryFont => ErrorCode::NoUsableFontFace,
            HyphenationError::UnsupportedPlacement(_) => ErrorCode::UnsupportedFeature,
            _ => ErrorCode::InvalidStyle,
        },
        "hyphenation admission failed",
    )
    .with_detail(ErrorDetail::Hyphenation { reason })
}

impl NativeHyphenationConvention {
    /// Capture a full-locale/script convention without inferring a script table.
    /// The governing style must match the complete locale case-insensitively,
    /// treating admitted underscore/hyphen separators equivalently. An explicit
    /// script in that locale must agree. Variants/extensions remain significant.
    ///
    /// # Errors
    /// Rejects invalid locale/script syntax, unknown/common/inherited script
    /// labels, or a contradictory explicit locale script.
    pub fn language_script(
        locale: &str,
        script: &str,
        automatic_text: Text,
        placement: HyphenationPlacement,
    ) -> Result<Self> {
        let parsed = parlance::Language::parse(locale)
            .map_err(|_| error(HyphenationError::InvalidConvention))?;
        let value = Self::declared_script(script, automatic_text, placement)?;
        if parsed
            .script()
            .is_some_and(|s| !s.eq_ignore_ascii_case(&value.script))
        {
            return Err(error(HyphenationError::InvalidConvention));
        }
        Ok(Self {
            locale: Some(locale.to_owned()),
            ..value
        })
    }

    /// Capture a convention requiring an actual explicit matching locale script.
    /// `und-Latn` establishes a declared Latin context without a known language.
    ///
    /// # Errors
    /// Rejects invalid script syntax and unknown/common/inherited script labels.
    pub fn declared_script(
        script: &str,
        automatic_text: Text,
        placement: HyphenationPlacement,
    ) -> Result<Self> {
        let script = parlance::Script::parse(script)
            .map_err(|_| error(HyphenationError::InvalidConvention))?;
        let script = script.as_str();
        if matches!(script, "Zzzz" | "Zyyy" | "Zinh") {
            return Err(error(HyphenationError::InvalidConvention));
        }
        Ok(Self {
            locale: None,
            script: script.to_owned(),
            automatic_text,
            placement,
        })
    }

    #[must_use]
    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }
    #[must_use]
    pub fn script(&self) -> &str {
        &self.script
    }
    #[must_use]
    pub fn automatic_text(&self) -> &Text {
        &self.automatic_text
    }
    #[must_use]
    pub const fn placement(&self) -> HyphenationPlacement {
        self.placement
    }

    pub(crate) fn bind(&self, locale: Option<&str>) -> Result<()> {
        let Some(locale) = locale else {
            return Err(error(HyphenationError::InvalidConventionBinding));
        };
        let parsed = parlance::Language::parse(locale)
            .map_err(|_| error(HyphenationError::InvalidConventionBinding))?;
        let matches = match &self.locale {
            Some(expected) => {
                expected
                    .bytes()
                    .map(normalize)
                    .eq(locale.bytes().map(normalize))
                    && parsed
                        .script()
                        .is_none_or(|s| s.eq_ignore_ascii_case(&self.script))
            }
            None => parsed
                .script()
                .is_some_and(|s| s.eq_ignore_ascii_case(&self.script)),
        };
        if matches {
            Ok(())
        } else {
            Err(error(HyphenationError::InvalidConventionBinding))
        }
    }
}
fn normalize(c: u8) -> u8 {
    if c == b'_' {
        b'-'
    } else {
        c.to_ascii_lowercase()
    }
}
