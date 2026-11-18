//! Native hyphenation policy and indication inputs with bounded en-US resources.

pub use crate::protocol::{
    HyphenationContinuation, HyphenationCounts, HyphenationProvenance, HyphenationResource,
    HyphenationSourcePieceView, PreparedHyphenationCut, PreparedHyphenationCutView, PreparedWord,
    PreparedWordPortion, PreparedWordView,
};
use crate::{Error, ErrorCode, ErrorDetail, Result, text::Text};

/// Whether authored and resource-qualified automatic hyphenation points may be used.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Hyphens {
    /// Suppress hyphenation points, retaining independent breaks.
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
/// Raw standalone layout retains its unconstrained count operation.
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
    /// Emit the chosen text at the logical end of the terminating line.
    LogicalBefore,
    /// Emit the chosen text at the logical start of its receiving line.
    LogicalAfter,
    /// Emit the same chosen text once on each side of the actual cut.
    BothSides,
}
impl HyphenationPlacement {
    pub(crate) const fn includes_before(self) -> bool {
        matches!(self, Self::LogicalBefore | Self::BothSides)
    }
    pub(crate) const fn includes_after(self) -> bool {
        matches!(self, Self::LogicalAfter | Self::BothSides)
    }
}

/// A native producer's explicit convention claim. This is not a language table.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NativeHyphenationConvention {
    locale: Option<String>,
    script: String,
    automatic_text: Text,
    placement: HyphenationPlacement,
}

/// Invalid input, missing resource, or unsupported indication composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HyphenationError {
    /// The resource, or a cut it reports, cannot be mapped onto the retained text:
    /// the query is inconsistent with the cut, or a cut's atoms have no origin
    /// intersection or preceding atoms.
    InvalidResourceMapping,
    /// A resource query length or offset is not representable.
    ResourceLengthOverflow,
    /// Storage for a resource query or its observation could not be reserved.
    ResourceStorage,
    /// The native convention names an invalid locale or script, or a script
    /// that disagrees with its locale.
    InvalidConvention,
    /// The native convention does not bind to the content locale: the content
    /// has no locale, an invalid one, or one the convention does not match.
    InvalidConventionBinding,
    /// A native logical-before replacement lists no alternatives.
    EmptyNativeAlternatives,
    /// The style selects no font for the automatic hyphen.
    MissingPrimaryFont,
    /// The continuation is inconsistent with the cut or placement it accompanies.
    InvalidContinuation,
    /// An incoming indication has no receiving bidi authority.
    MissingReceivingBidiAuthority,
    /// The indication cannot attach to a separator.
    UnsupportedIndicationSeparator,
    /// The indication cannot be composed with the surrounding text.
    UnsupportedIndicationComposition,
    /// No captured font query covers the indication.
    UncapturedIndicationFontQuery,
}

impl std::fmt::Display for HyphenationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidResourceMapping => {
                "the hyphenation resource or a cut it reports cannot be mapped onto the retained text"
            }
            Self::ResourceLengthOverflow => {
                "a hyphenation resource length or offset is not representable"
            }
            Self::ResourceStorage => {
                "storage for a hyphenation resource query or its observation could not be reserved"
            }
            Self::InvalidConvention => {
                "the native hyphenation convention has an invalid or inconsistent locale or script"
            }
            Self::InvalidConventionBinding => {
                "the native hyphenation convention does not bind to the content locale"
            }
            Self::EmptyNativeAlternatives => {
                "the native logical-before replacement lists no alternatives"
            }
            Self::MissingPrimaryFont => "the style selects no font for the automatic hyphen",
            Self::InvalidContinuation => {
                "the hyphenation continuation is inconsistent with the cut or placement it accompanies"
            }
            Self::MissingReceivingBidiAuthority => {
                "an incoming hyphenation indication has no receiving bidi authority"
            }
            Self::UnsupportedIndicationSeparator => {
                "the hyphenation indication cannot attach to a separator"
            }
            Self::UnsupportedIndicationComposition => {
                "the hyphenation indication cannot be composed with the surrounding text"
            }
            Self::UncapturedIndicationFontQuery => {
                "no captured font query covers the hyphenation indication"
            }
        })
    }
}

impl std::error::Error for HyphenationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidResourceMapping
            | Self::ResourceLengthOverflow
            | Self::ResourceStorage
            | Self::InvalidConvention
            | Self::InvalidConventionBinding
            | Self::EmptyNativeAlternatives
            | Self::MissingPrimaryFont
            | Self::InvalidContinuation
            | Self::MissingReceivingBidiAuthority
            | Self::UnsupportedIndicationSeparator
            | Self::UnsupportedIndicationComposition
            | Self::UncapturedIndicationFontQuery => None,
        }
    }
}

pub(crate) fn error(reason: HyphenationError) -> Error {
    Error::new(
        match reason {
            HyphenationError::InvalidResourceMapping
            | HyphenationError::ResourceStorage
            | HyphenationError::ResourceLengthOverflow => ErrorCode::LayoutFailed,
            HyphenationError::MissingPrimaryFont => ErrorCode::NoUsableFontFace,
            HyphenationError::MissingReceivingBidiAuthority
            | HyphenationError::UnsupportedIndicationSeparator
            | HyphenationError::UnsupportedIndicationComposition
            | HyphenationError::UncapturedIndicationFontQuery => ErrorCode::UnsupportedFeature,
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
