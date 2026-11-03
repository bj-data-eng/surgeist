//! HTML 877d #pragma-directives: source-recognized states are not all conforming.
use super::*;

/// Conforming author keywords. Historical processing states are excluded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaPragma {
    /// The `content-type` keyword: the Encoding declaration state, an
    /// alternative form of the `charset` attribute (HTML877d #pragma-directives).
    ContentType,
    /// The `default-style` keyword: sets the name of the default CSS style sheet
    /// set (HTML877d #pragma-directives).
    DefaultStyle,
    /// The `refresh` keyword: a timed redirect or reload (HTML877d #pragma-directives).
    Refresh,
    /// The `x-ua-compatible` keyword, whose `content` must be `IE=edge`
    /// compared ASCII case-insensitively (HTML877d #pragma-directives).
    XUaCompatible,
    /// The `content-security-policy` keyword, whose `content` must be a valid
    /// Content Security Policy (HTML877d #pragma-directives).
    ContentSecurityPolicy,
}
impl HtmlMetaPragma {
    /// The exact lowercase keyword that authors write for this pragma.
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::ContentType => "content-type",
            Self::DefaultStyle => "default-style",
            Self::Refresh => "refresh",
            Self::XUaCompatible => "x-ua-compatible",
            Self::ContentSecurityPolicy => "content-security-policy",
        }
    }
    /// Builds the no-namespace `http-equiv` attribute with the canonical
    /// keyword. The pragma's `content` requirements are not part of it.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::HttpEquiv.qualified_name(),
            self.canonical_keyword(),
        )
    }
}

/// Intrinsic states, including source-required nonconforming legacy processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaPragmaState {
    /// The `content-language` keyword. Nonconforming; it sets the pragma-set
    /// default language (HTML877d #pragma-directives).
    ContentLanguage,
    /// The `content-type` keyword, the Encoding declaration state, which is a
    /// character encoding declaration (HTML877d #pragma-directives).
    EncodingDeclaration,
    /// The `default-style` keyword.
    DefaultStyle,
    /// The `refresh` keyword.
    Refresh,
    /// The `set-cookie` keyword. Nonconforming (HTML877d #pragma-directives).
    SetCookie,
    /// The `x-ua-compatible` keyword.
    XUaCompatible,
    /// The `content-security-policy` keyword.
    ContentSecurityPolicy,
}
const HTTP_EQUIV: EnumRule<HtmlMetaPragmaState> = EnumRule {
    metadata: RuleMetadata {
        attribute: KnownHtmlAttribute::HttpEquiv,
        rows: &[71],
        source: source!("attr-meta-http-equiv"),
        context: Context::Elements(&[KnownHtmlElement::Meta]),
        vocabulary_source: None,
    },
    keywords: &[
        Keyword {
            spelling: "content-language",
            canonical: "content-language",
            state: HtmlMetaPragmaState::ContentLanguage,
            conforming: false,
        },
        Keyword {
            spelling: "content-type",
            canonical: "content-type",
            state: HtmlMetaPragmaState::EncodingDeclaration,
            conforming: true,
        },
        Keyword {
            spelling: "default-style",
            canonical: "default-style",
            state: HtmlMetaPragmaState::DefaultStyle,
            conforming: true,
        },
        Keyword {
            spelling: "refresh",
            canonical: "refresh",
            state: HtmlMetaPragmaState::Refresh,
            conforming: true,
        },
        Keyword {
            spelling: "set-cookie",
            canonical: "set-cookie",
            state: HtmlMetaPragmaState::SetCookie,
            conforming: false,
        },
        Keyword {
            spelling: "x-ua-compatible",
            canonical: "x-ua-compatible",
            state: HtmlMetaPragmaState::XUaCompatible,
            conforming: true,
        },
        Keyword {
            spelling: "content-security-policy",
            canonical: "content-security-policy",
            state: HtmlMetaPragmaState::ContentSecurityPolicy,
            conforming: true,
        },
    ],
    missing: None,
    invalid: None,
    empty: None,
    exact_keywords: false,
};
impl HtmlValueInterpreter {
    /// Intrinsic classification only: no HTTP-header equivalence, content parsing
    /// or pragma effects. Present applicable context remains independently unchecked.
    pub fn meta_http_equiv(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<HtmlMetaPragmaState>, HtmlValueAdmissionError> {
        let mut result = self.enumerated(&HTTP_EQUIV, element, raw)?;
        if result.applicability == HtmlAttributeApplicability::Applicable && raw.is_some() {
            result.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        Ok(result)
    }
}

/// Intrinsic recognized pragma state without an invented Element context.
/// Operands are already retained; the caller admits each keyword read.
pub fn meta_pragma_state_with<E>(
    raw: &DocumentString,
    mut admit: impl FnMut() -> Result<(), E>,
) -> Result<Option<HtmlMetaPragmaState>, E> {
    Ok(enum_value_selection_with(&HTTP_EQUIV, Some(raw), |_| admit())?.intrinsic)
}
