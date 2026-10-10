//! Authored URL models and checked request-modifier admission.

use super::{CssAuthoredFunctionArguments, CssIdent};
use crate::CssValueOrigin;

/// The two authored function identities in the selected Values 4 `<url>` grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUrlFunction {
    Url,
    Src,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUrl {
    function: CssUrlFunction,
    value: String,
    modifiers: Vec<CssUrlModifier>,
}

impl CssUrl {
    /// Preserves a decoded authored URL, including empty or whitespace values.
    /// Resource resolution and usability belong to downstream consumers.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self::from_parts(CssUrlFunction::Url, value, Vec::new())
    }

    /// Constructs a decoded authored URL with its function identity and ordered modifiers.
    /// Resource resolution and modifier interpretation belong downstream.
    #[must_use]
    pub fn from_parts(
        function: CssUrlFunction,
        value: impl Into<String>,
        modifiers: Vec<CssUrlModifier>,
    ) -> Self {
        Self {
            function,
            value: value.into(),
            modifiers,
        }
    }

    #[must_use]
    pub const fn function(&self) -> CssUrlFunction {
        self.function
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Whether this authored URL has the Values 4 local URL flag.
    /// A fragment-only target remains symbolic; tree-scoped resolution is downstream.
    #[must_use]
    pub fn is_local_url(&self) -> bool {
        self.value.starts_with('#')
    }

    /// Returns the authored modifiers of a quoted `url()` or `src()` value.
    ///
    /// The Values 4 identifier/function carrier remains open to extensions.
    /// Values 5 request functions additionally expose their checked argument
    /// semantics through [`CssUrlModifierFunction::request_modifier`]. Neither
    /// inspection nor construction applies request steps or loads resources.
    #[must_use]
    pub fn modifiers(&self) -> &[CssUrlModifier] {
        &self.modifiers
    }
}

/// One authored URL modifier in the open Values 4 identifier/function carrier.
///
/// Known Values 5 request functions have grammar-checked arguments; bare
/// identifiers and unknown functions remain generic extension syntax. Modifiers
/// stay ordered and symbolic without applying request steps or loading resources.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssUrlModifier {
    Ident(CssIdent),
    Function(CssUrlModifierFunction),
}

/// One checked functional URL modifier with retained authored argument tokens.
/// Equality remains based on its decoded name and authored argument text,
/// independent of the retained component origins.
#[derive(Clone)]
pub struct CssUrlModifierFunction {
    name: CssIdent,
    arguments: CssAuthoredFunctionArguments,
    argument_components: crate::CssComponentValues,
    request_modifier: Option<RequestUrlModifier>,
}

impl CssUrlModifierFunction {
    /// Constructs a structurally valid modifier, checking recognized request arguments.
    ///
    /// `crossorigin()` requires one of its two keywords, `integrity()` one string,
    /// and `referrerpolicy()` one of its eight keywords. Invalid recognized
    /// arguments return [`crate::CssComponentValueErrorKind::InvalidFunction`]
    /// with their original token origin. Unknown functions retain generic syntax.
    pub fn try_new(
        name: CssIdent,
        arguments: crate::CssComponentValues,
    ) -> Result<Self, crate::CssComponentValueError> {
        crate::CssComponentValue::try_function(name.as_str(), arguments.clone())?;
        let request_modifier = validate_request_url_modifier(name.as_str(), &arguments)?;
        let css = arguments
            .serialize_with_limit(crate::CssComponentValueLimits::default().max_css_bytes())?
            .as_css()
            .to_owned();
        Ok(Self {
            name,
            arguments: CssAuthoredFunctionArguments::new(css),
            argument_components: arguments,
            request_modifier,
        })
    }

    pub(crate) fn from_parsed(
        name: CssIdent,
        arguments: CssAuthoredFunctionArguments,
        argument_components: crate::CssComponentValues,
    ) -> Result<Self, crate::CssComponentValueError> {
        let request_modifier = validate_request_url_modifier(name.as_str(), &argument_components)?;
        Ok(Self {
            name,
            arguments,
            argument_components,
            request_modifier,
        })
    }

    /// Returns checked Values 5 request data, or `None` for an unknown function.
    ///
    /// The view borrows the decoded integrity string and normalizes recognized
    /// keyword identity without changing the retained authored tokens or name.
    /// Request execution and interpretation of unknown extensions stay downstream.
    #[must_use]
    pub fn request_modifier(&self) -> Option<CssRequestUrlModifierRef<'_>> {
        self.request_modifier
            .as_ref()
            .map(|modifier| match modifier {
                RequestUrlModifier::CrossOrigin(value) => {
                    CssRequestUrlModifierRef::CrossOrigin(*value)
                }
                RequestUrlModifier::Integrity(value) => CssRequestUrlModifierRef::Integrity(value),
                RequestUrlModifier::ReferrerPolicy(value) => {
                    CssRequestUrlModifierRef::ReferrerPolicy(*value)
                }
            })
    }

    /// Returns the decoded, case-preserving function name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the preserved authored function arguments.
    #[must_use]
    pub const fn arguments(&self) -> &CssAuthoredFunctionArguments {
        &self.arguments
    }

    /// Returns the original immutable argument tokens and their source origins.
    #[must_use]
    pub const fn argument_components(&self) -> &crate::CssComponentValues {
        &self.argument_components
    }
}

/// The two authored `crossorigin()` choices in Values 5 §4.1.1.
/// This value describes request syntax; it does not change Fetch credentials.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUrlCrossOrigin {
    Anonymous,
    UseCredentials,
}

/// The eight authored `referrerpolicy()` choices in Values 5 §4.1.1.
/// This value does not select or execute a network request policy.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUrlReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    SameOrigin,
    Origin,
    StrictOrigin,
    OriginWhenCrossOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

/// Checked, borrowed semantics for a recognized Values 5 request URL modifier.
/// Integrity remains an arbitrary decoded string, including empty text; hash
/// algorithms and resource integrity evaluation are not CSS grammar checks.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssRequestUrlModifierRef<'a> {
    CrossOrigin(CssUrlCrossOrigin),
    Integrity(&'a str),
    ReferrerPolicy(CssUrlReferrerPolicy),
}

#[derive(Clone)]
enum RequestUrlModifier {
    CrossOrigin(CssUrlCrossOrigin),
    Integrity(String),
    ReferrerPolicy(CssUrlReferrerPolicy),
}

// Values 5 WD 2024-11-11 §4.1.1. This is the single grammar boundary for parsed
// functions and checked construction. The generic Values 4 carrier remains open.
fn validate_request_url_modifier(
    name: &str,
    arguments: &crate::CssComponentValues,
) -> Result<Option<RequestUrlModifier>, crate::CssComponentValueError> {
    use crate::{
        CssComponentValueError, CssComponentValueErrorKind, CssComponentValueRef, CssValueTokenRef,
    };

    if !name.eq_ignore_ascii_case("crossorigin")
        && !name.eq_ignore_ascii_case("integrity")
        && !name.eq_ignore_ascii_case("referrerpolicy")
    {
        return Ok(None);
    }
    let invalid =
        |origin| CssComponentValueError::new(CssComponentValueErrorKind::InvalidFunction, origin);
    let mut significant = arguments.items().iter().filter(|component| {
        !matches!(
            component.view(),
            CssComponentValueRef::Comment(_)
                | CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
        )
    });
    let argument = significant.next().ok_or_else(|| {
        invalid(
            arguments
                .items()
                .first()
                .map_or(CssValueOrigin::Programmatic, |component| {
                    component.origin().clone()
                }),
        )
    })?;
    if let Some(extra) = significant.next() {
        return Err(invalid(extra.origin().clone()));
    }
    let request = match argument.view() {
        CssComponentValueRef::Token(CssValueTokenRef::String(value))
            if name.eq_ignore_ascii_case("integrity") =>
        {
            Some(RequestUrlModifier::Integrity(value.to_owned()))
        }
        CssComponentValueRef::Token(CssValueTokenRef::Ident(value))
            if name.eq_ignore_ascii_case("crossorigin") =>
        {
            if value.eq_ignore_ascii_case("anonymous") {
                Some(RequestUrlModifier::CrossOrigin(
                    CssUrlCrossOrigin::Anonymous,
                ))
            } else if value.eq_ignore_ascii_case("use-credentials") {
                Some(RequestUrlModifier::CrossOrigin(
                    CssUrlCrossOrigin::UseCredentials,
                ))
            } else {
                None
            }
        }
        CssComponentValueRef::Token(CssValueTokenRef::Ident(value))
            if name.eq_ignore_ascii_case("referrerpolicy") =>
        {
            let policy = match value.to_ascii_lowercase().as_str() {
                "no-referrer" => Some(CssUrlReferrerPolicy::NoReferrer),
                "no-referrer-when-downgrade" => Some(CssUrlReferrerPolicy::NoReferrerWhenDowngrade),
                "same-origin" => Some(CssUrlReferrerPolicy::SameOrigin),
                "origin" => Some(CssUrlReferrerPolicy::Origin),
                "strict-origin" => Some(CssUrlReferrerPolicy::StrictOrigin),
                "origin-when-cross-origin" => Some(CssUrlReferrerPolicy::OriginWhenCrossOrigin),
                "strict-origin-when-cross-origin" => {
                    Some(CssUrlReferrerPolicy::StrictOriginWhenCrossOrigin)
                }
                "unsafe-url" => Some(CssUrlReferrerPolicy::UnsafeUrl),
                _ => None,
            };
            policy.map(RequestUrlModifier::ReferrerPolicy)
        }
        _ => None,
    };
    request
        .map(Some)
        .ok_or_else(|| invalid(argument.origin().clone()))
}

impl PartialEq for CssUrlModifierFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments
    }
}

impl Eq for CssUrlModifierFunction {}

impl std::fmt::Debug for CssUrlModifierFunction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssUrlModifierFunction")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .finish()
    }
}
