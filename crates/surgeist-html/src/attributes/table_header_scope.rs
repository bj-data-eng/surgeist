//! Local th scope keywords and their source-defined content defaults.
//! Table slot anchoring and actual header associations remain table semantics.

use super::*;

/// Conforming authored th scope keywords.
///
/// Group keywords prove local spelling only. Their separate author requirement
/// needs actual row/column group anchoring in a qualified table model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableHeaderScope {
    Row,
    Column,
    RowGroup,
    ColumnGroup,
}

impl HtmlTableHeaderScope {
    /// Returns the source's conforming canonical author keyword.
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "col",
            Self::RowGroup => "rowgroup",
            Self::ColumnGroup => "colgroup",
        }
    }

    /// Emits a canonical no-namespace scope attribute with ordinary allocation.
    /// This does not prove a th role or the group keywords' anchoring requirement.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Scope.qualified_name(),
            self.canonical_keyword(),
        )
    }
}

/// Intrinsic th scope state, independent of actual header association.
/// Auto has no conforming authored keyword or native attribute emitter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableHeaderScopeState {
    Keyword(HtmlTableHeaderScope),
    /// Missing or invalid content selects context-dependent Auto processing.
    Auto,
}

enum_rule!(TH_SCOPE, HtmlTableHeaderScopeState, Scope,
    Context::Elements(&[KnownHtmlElement::Th]), [134], "attr-th-scope",
    Some(HtmlTableHeaderScopeState::Auto), Some(HtmlTableHeaderScopeState::Auto), None,
    ["row" => ("row", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Row)),
     "col" => ("col", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Column)),
     "rowgroup" => ("rowgroup", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::RowGroup)),
     "colgroup" => ("colgroup", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::ColumnGroup))]);

impl HtmlValueInterpreter {
    /// Interprets th scope content through the shared enumeration owner.
    ///
    /// Missing and invalid content select Auto without changing recognition or
    /// author spelling. Empty and literal `auto` are invalid authored keywords.
    /// Selected RowGroup/ColumnGroup retain their state and leave actual table
    /// anchoring Unchecked; other states select no additional context check.
    /// No parent-name inference, table reconstruction, header association or
    /// complete th/table conformance is performed.
    ///
    /// Entire original raw UTF-16 admission precedes expanded-name applicability.
    /// Reports use ordinary shared allocation and the existing per-value cap,
    /// without physical-memory, work or cancellation accounting.
    pub fn th_scope(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlTableHeaderScopeState>, HtmlValueAdmissionError> {
        let mut report = self.enumerated(&TH_SCOPE, element, raw)?;
        if matches!(
            report.intrinsic,
            Some(HtmlTableHeaderScopeState::Keyword(
                HtmlTableHeaderScope::RowGroup | HtmlTableHeaderScope::ColumnGroup
            ))
        ) {
            report.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        Ok(report)
    }
}
