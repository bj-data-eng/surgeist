//! HTML 877d #pragma-directives: source-recognized states are not all conforming.
use super::*;

/// Conforming author keywords. Historical processing states are excluded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaPragma {
    ContentType,
    DefaultStyle,
    Refresh,
    XUaCompatible,
    ContentSecurityPolicy,
}
impl HtmlMetaPragma {
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::ContentType => "content-type",
            Self::DefaultStyle => "default-style",
            Self::Refresh => "refresh",
            Self::XUaCompatible => "x-ua-compatible",
            Self::ContentSecurityPolicy => "content-security-policy",
        }
    }
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
    ContentLanguage,
    EncodingDeclaration,
    DefaultStyle,
    Refresh,
    SetCookie,
    XUaCompatible,
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlMetaPragmaState>, HtmlValueAdmissionError> {
        let mut result = self.enumerated(&HTTP_EQUIV, element, raw)?;
        if result.applicability == HtmlAttributeApplicability::Applicable && raw.is_some() {
            result.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        Ok(result)
    }
}
