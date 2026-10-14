//! HTML 877d #pragma-directives: non-executing static content observations.
//! No insertion history, encoding, URL resolution, stylesheet mutation, CSP or navigation.
use super::*;
use crate::bounded::is_ascii_whitespace as ascii_whitespace;
use crate::{HtmlAttributeReport, HtmlMetaPragmaState, HtmlNonNegativeInteger};

/// An exact range in retained content, produced at checked source cursor boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMetaContentSlice {
    raw: DocumentString,
    start: usize,
    end: usize,
}
impl HtmlMetaContentSlice {
    pub fn raw_content(&self) -> &DocumentString {
        &self.raw
    }
    pub fn code_unit_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
    pub fn code_units(&self) -> &[u16] {
        &self.raw.code_units()[self.start..self.end]
    }
}

/// HTML framing alone is not validation of delegated URL or CSP grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaPragmaContentAuthor {
    Satisfied,
    MissingContent,
    InvalidHtmlSyntax,
    ExternalGrammarUnchecked,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaRefreshTarget {
    /// Source selected the existing Document URL; its actual value is not supplied here.
    DocumentUrl,
    /// Exact pre-URL-parser substring; no resolution, scheme or validity claim.
    UrlString(HtmlMetaContentSlice),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMetaRefreshValue {
    delay: HtmlInteger,
    target: HtmlMetaRefreshTarget,
}
impl HtmlMetaRefreshValue {
    pub fn delay_seconds(&self) -> &HtmlNonNegativeInteger {
        self.delay.magnitude()
    }
    pub fn target(&self) -> &HtmlMetaRefreshTarget {
        &self.target
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaRefreshLexical {
    Rejected,
    Parsed(HtmlMetaRefreshValue),
}
/// Static content only. A candidate never establishes successful pragma processing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaPragmaContent {
    NoState,
    ContentLanguage {
        candidate: Option<HtmlMetaContentSlice>,
    },
    EncodingDeclaration,
    DefaultStyle {
        candidate: Option<HtmlMetaContentSlice>,
    },
    Refresh(HtmlMetaRefreshLexical),
    IgnoredSetCookie,
    IgnoredXUaCompatible,
    /// Direct head relation is independent of the delegated CSP grammar.
    ContentSecurityPolicy {
        head_parent: HtmlSemanticValue<bool>,
        nonempty_content: bool,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaPragmaUniqueness {
    NotRequired,
    NoDocumentMetadataScope,
    Unique,
    Duplicate(HtmlCaptureSubjectId),
    Missing(HtmlSemanticMissing),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaEncodingCharsetConflict {
    NotRequired,
    NoDocumentMetadataScope,
    Absent,
    Present(HtmlCaptureSubjectId),
    Missing(HtmlSemanticMissing),
}
/// Independent exact local classification, content and ordinary-Document observations.
/// This is not full meta, document or pragma execution conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMetaPragmaContentReport {
    data: Vec<MetaPragmaData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct MetaPragmaData {
    http_equiv: HtmlAttributeReport<HtmlMetaPragmaState>,
    content: Option<Attribute>,
    author: Option<HtmlMetaPragmaContentAuthor>,
    interpretation: HtmlMetaPragmaContent,
    uniqueness: HtmlMetaPragmaUniqueness,
    charset_conflict: HtmlMetaEncodingCharsetConflict,
}
impl HtmlMetaPragmaContentReport {
    pub fn http_equiv(&self) -> &HtmlAttributeReport<HtmlMetaPragmaState> {
        &self.data[0].http_equiv
    }
    pub fn content_attribute(&self) -> Option<&Attribute> {
        self.data[0].content.as_ref()
    }
    /// None means no selected state, not a conforming content default.
    pub fn content_author(&self) -> Option<HtmlMetaPragmaContentAuthor> {
        self.data[0].author
    }
    pub fn interpretation(&self) -> &HtmlMetaPragmaContent {
        &self.data[0].interpretation
    }
    pub fn uniqueness(&self) -> &HtmlMetaPragmaUniqueness {
        &self.data[0].uniqueness
    }
    pub fn charset_conflict(&self) -> &HtmlMetaEncodingCharsetConflict {
        &self.data[0].charset_conflict
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn meta_pragma_content(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlMetaPragmaContentReport>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Meta) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let http_equiv = self.pragma_classification(element)?;
        let content = self.attribute(element, "content")?;
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(content.map(Attribute::value))
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        let state = http_equiv.intrinsic().copied();
        let raw = content.map(Attribute::value);
        let mut author = state.map(|_| {
            if content.is_some() {
                HtmlMetaPragmaContentAuthor::Satisfied
            } else {
                HtmlMetaPragmaContentAuthor::MissingContent
            }
        });
        self.charge(1)?;
        let interpretation = match state {
            None => HtmlMetaPragmaContent::NoState,
            Some(HtmlMetaPragmaState::ContentLanguage) => HtmlMetaPragmaContent::ContentLanguage {
                candidate: match raw {
                    Some(raw) => self.pragma_language(raw)?,
                    None => None,
                },
            },
            Some(HtmlMetaPragmaState::EncodingDeclaration) => {
                if let Some(raw) = raw {
                    author = Some(if self.pragma_encoding_author(raw)? {
                        HtmlMetaPragmaContentAuthor::Satisfied
                    } else {
                        HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax
                    })
                }
                HtmlMetaPragmaContent::EncodingDeclaration
            }
            Some(HtmlMetaPragmaState::DefaultStyle) => HtmlMetaPragmaContent::DefaultStyle {
                candidate: match raw {
                    Some(raw) if !raw.is_empty() => Some(self.pragma_slice(raw, 0, raw.len())?),
                    _ => None,
                },
            },
            Some(HtmlMetaPragmaState::Refresh) => {
                let lexical = match raw {
                    Some(raw) => {
                        author = Some(self.pragma_refresh_author(raw)?);
                        self.pragma_refresh(raw)?
                    }
                    None => HtmlMetaRefreshLexical::Rejected,
                };
                HtmlMetaPragmaContent::Refresh(lexical)
            }
            Some(HtmlMetaPragmaState::SetCookie) => HtmlMetaPragmaContent::IgnoredSetCookie,
            Some(HtmlMetaPragmaState::XUaCompatible) => {
                if let Some(raw) = raw {
                    author = Some(
                        if self.pragma_keyword_at(raw.code_units(), 0, b"IE=edge")?
                            && raw.len() == 7
                        {
                            HtmlMetaPragmaContentAuthor::Satisfied
                        } else {
                            HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax
                        },
                    )
                }
                HtmlMetaPragmaContent::IgnoredXUaCompatible
            }
            Some(HtmlMetaPragmaState::ContentSecurityPolicy) => {
                if content.is_some() {
                    author = Some(HtmlMetaPragmaContentAuthor::ExternalGrammarUnchecked)
                }
                let head_parent = match self.parent(node)? {
                    Err(missing) => HtmlSemanticValue::Missing(missing),
                    Ok(None) => HtmlSemanticValue::Known(false),
                    Ok(Some(id)) => {
                        let parent = self.node(&id)?;
                        self.charge(1)?;
                        HtmlSemanticValue::Known(match parent.kind() {
                            HtmlCapturedNodeKind::Element(e) => {
                                self.known(e)? == Some(KnownHtmlElement::Head)
                            }
                            _ => false,
                        })
                    }
                };
                HtmlMetaPragmaContent::ContentSecurityPolicy {
                    head_parent,
                    nonempty_content: raw.is_some_and(|v| !v.is_empty()),
                }
            }
        };
        let mut frames = Vec::new();
        let result = match state {
            Some(state) => self.pragma_population(subject, state, &mut frames),
            None => {
                self.charge(1)?;
                Ok((
                    HtmlMetaPragmaUniqueness::NotRequired,
                    HtmlMetaEncodingCharsetConflict::NotRequired,
                ))
            }
        };
        self.release_scratch(frames.capacity());
        let (uniqueness, charset_conflict) = result?;
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(MetaPragmaData {
            http_equiv,
            content: content.cloned(),
            author,
            interpretation,
            uniqueness,
            charset_conflict,
        });
        Ok(HtmlSemanticValue::Known(HtmlMetaPragmaContentReport {
            data,
        }))
    }
    fn pragma_classification(
        &mut self,
        element: &'a HtmlCapturedElement,
    ) -> Result<HtmlAttributeReport<HtmlMetaPragmaState>, HtmlSemanticFailure> {
        let raw = self.attribute(element, "http-equiv")?;
        self.admit_intrinsic(element, raw, 7)?;
        self.interpreter
            .meta_http_equiv(element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))
    }
    fn pragma_unit(
        &mut self,
        units: &[u16],
        pos: usize,
    ) -> Result<Option<u16>, HtmlSemanticFailure> {
        self.charge(1)?;
        Ok(units.get(pos).copied())
    }
    fn pragma_space(&mut self, units: &[u16], pos: &mut usize) -> Result<(), HtmlSemanticFailure> {
        while self.pragma_unit(units, *pos)?.is_some_and(ascii_whitespace) {
            *pos += 1;
        }
        Ok(())
    }
    fn pragma_keyword_at(
        &mut self,
        units: &[u16],
        pos: usize,
        keyword: &[u8],
    ) -> Result<bool, HtmlSemanticFailure> {
        for (offset, byte) in keyword.iter().enumerate() {
            let Some(unit) = self.pragma_unit(units, pos + offset)? else {
                return Ok(false);
            };
            if !u8::try_from(unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(byte)) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn pragma_slice(
        &mut self,
        raw: &DocumentString,
        start: usize,
        end: usize,
    ) -> Result<HtmlMetaContentSlice, HtmlSemanticFailure> {
        self.charge(1)?;
        // Cursor positions are checked source boundaries in this same raw value.
        Ok(HtmlMetaContentSlice {
            raw: raw.clone(),
            start,
            end,
        })
    }
    fn pragma_language(
        &mut self,
        raw: &DocumentString,
    ) -> Result<Option<HtmlMetaContentSlice>, HtmlSemanticFailure> {
        let units = raw.code_units();
        for pos in 0..units.len() {
            if self.pragma_unit(units, pos)? == Some(44) {
                return Ok(None);
            }
        }
        let mut pos = 0;
        self.pragma_space(units, &mut pos)?;
        let start = pos;
        while self
            .pragma_unit(units, pos)?
            .is_some_and(|u| !ascii_whitespace(u))
        {
            pos += 1;
        }
        if start == pos {
            Ok(None)
        } else {
            self.pragma_slice(raw, start, pos).map(Some)
        }
    }
    fn pragma_encoding_author(
        &mut self,
        raw: &DocumentString,
    ) -> Result<bool, HtmlSemanticFailure> {
        let units = raw.code_units();
        if !self.pragma_keyword_at(units, 0, b"text/html;")? {
            return Ok(false);
        }
        let mut pos = 10;
        self.pragma_space(units, &mut pos)?;
        if !self.pragma_keyword_at(units, pos, b"charset=utf-8")? {
            return Ok(false);
        }
        self.charge(1)?;
        Ok(pos + 13 == units.len())
    }
    fn pragma_refresh_author(
        &mut self,
        raw: &DocumentString,
    ) -> Result<HtmlMetaPragmaContentAuthor, HtmlSemanticFailure> {
        let units = raw.code_units();
        let mut pos = 0;
        while self
            .pragma_unit(units, pos)?
            .is_some_and(|u| (48..=57).contains(&u))
        {
            pos += 1;
        }
        if pos == 0 {
            return Ok(HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax);
        }
        self.charge(1)?;
        if pos == units.len() {
            return Ok(HtmlMetaPragmaContentAuthor::Satisfied);
        }
        if self.pragma_unit(units, pos)? != Some(59) {
            return Ok(HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax);
        }
        pos += 1;
        if !self.pragma_unit(units, pos)?.is_some_and(ascii_whitespace) {
            return Ok(HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax);
        }
        self.pragma_space(units, &mut pos)?;
        if !self.pragma_keyword_at(units, pos, b"URL=")? {
            return Ok(HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax);
        }
        pos += 4;
        if self
            .pragma_unit(units, pos)?
            .is_some_and(|u| u == 39 || u == 34)
        {
            return Ok(HtmlMetaPragmaContentAuthor::InvalidHtmlSyntax);
        }
        // An empty or any other suffix needs the actual delegated URL grammar.
        Ok(HtmlMetaPragmaContentAuthor::ExternalGrammarUnchecked)
    }
    fn pragma_refresh(
        &mut self,
        raw: &DocumentString,
    ) -> Result<HtmlMetaRefreshLexical, HtmlSemanticFailure> {
        let units = raw.code_units();
        let mut pos = 0;
        self.pragma_space(units, &mut pos)?;
        let start = pos;
        while self
            .pragma_unit(units, pos)?
            .is_some_and(|u| (48..=57).contains(&u))
        {
            pos += 1;
        }
        self.charge(1)?;
        let delay = if start == pos {
            if self.pragma_unit(units, pos)? != Some(46) {
                return Ok(HtmlMetaRefreshLexical::Rejected);
            }
            self.integer_count(0)?
        } else {
            // Source has proved a digit-only initial quantity, with no sign.
            // Existing recovery over the original admitted input yields that exact
            // nonnegative prefix; no substring allocation or second integer grammar.
            let remaining = self.numeric_budget(raw.len())?;
            match try_parse_integer(raw, self.limits.integer_digits, remaining) {
                Ok(result) => self.numeric_result(Ok(result))?,
                Err(IntegerRecoveryError::Allocation(error)) => {
                    return self
                        .numeric_result(Err(error))
                        .map(|_| HtmlMetaRefreshLexical::Rejected);
                }
                Err(IntegerRecoveryError::Syntax(_)) => {
                    return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
                }
            }
        };
        while self
            .pragma_unit(units, pos)?
            .is_some_and(|u| (48..=57).contains(&u) || u == 46)
        {
            pos += 1;
        }
        self.charge(1)?;
        let target = if pos == units.len() {
            HtmlMetaRefreshTarget::DocumentUrl
        } else {
            if !self
                .pragma_unit(units, pos)?
                .is_some_and(|u| u == 59 || u == 44 || ascii_whitespace(u))
            {
                return Ok(HtmlMetaRefreshLexical::Rejected);
            }
            self.pragma_space(units, &mut pos)?;
            if self
                .pragma_unit(units, pos)?
                .is_some_and(|u| u == 59 || u == 44)
            {
                pos += 1;
            }
            self.pragma_space(units, &mut pos)?;
            self.charge(1)?;
            if pos == units.len() {
                HtmlMetaRefreshTarget::DocumentUrl
            } else {
                let original = pos;
                let mut skip_quotes = true;
                if self
                    .pragma_unit(units, pos)?
                    .is_some_and(|u| u == 85 || u == 117)
                {
                    pos += 1;
                    if self
                        .pragma_unit(units, pos)?
                        .is_some_and(|u| u == 82 || u == 114)
                    {
                        pos += 1;
                        if self
                            .pragma_unit(units, pos)?
                            .is_some_and(|u| u == 76 || u == 108)
                        {
                            pos += 1;
                            self.pragma_space(units, &mut pos)?;
                            if self.pragma_unit(units, pos)? == Some(61) {
                                pos += 1;
                                self.pragma_space(units, &mut pos)?;
                            } else {
                                skip_quotes = false;
                            }
                        } else {
                            skip_quotes = false;
                        }
                    } else {
                        skip_quotes = false;
                    }
                }
                if !skip_quotes {
                    HtmlMetaRefreshTarget::UrlString(self.pragma_slice(
                        raw,
                        original,
                        units.len(),
                    )?)
                } else {
                    let quote = self
                        .pragma_unit(units, pos)?
                        .filter(|u| *u == 39 || *u == 34);
                    if quote.is_some() {
                        pos += 1;
                    }
                    let start = pos;
                    let mut end = units.len();
                    if let Some(quote) = quote {
                        while let Some(unit) = self.pragma_unit(units, pos)? {
                            if unit == quote {
                                end = pos;
                                break;
                            }
                            pos += 1;
                        }
                    }
                    HtmlMetaRefreshTarget::UrlString(self.pragma_slice(raw, start, end)?)
                }
            }
        };
        self.charge(1)?;
        Ok(HtmlMetaRefreshLexical::Parsed(HtmlMetaRefreshValue {
            delay,
            target,
        }))
    }
    fn pragma_population(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        state: HtmlMetaPragmaState,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<(HtmlMetaPragmaUniqueness, HtmlMetaEncodingCharsetConflict), HtmlSemanticFailure>
    {
        let encoding = state == HtmlMetaPragmaState::EncodingDeclaration;
        let root = match self.ordinary_tree_root(subject)? {
            Ok(root) => root,
            Err(gap) => {
                return Ok((
                    HtmlMetaPragmaUniqueness::Missing(gap.clone()),
                    if encoding {
                        HtmlMetaEncodingCharsetConflict::Missing(gap)
                    } else {
                        HtmlMetaEncodingCharsetConflict::NotRequired
                    },
                ));
            }
        };
        let root_node = self.node(&root)?;
        self.charge(1)?;
        if !matches!(root_node.kind(), HtmlCapturedNodeKind::Document(_)) {
            return Ok((
                HtmlMetaPragmaUniqueness::NoDocumentMetadataScope,
                if encoding {
                    HtmlMetaEncodingCharsetConflict::NoDocumentMetadataScope
                } else {
                    HtmlMetaEncodingCharsetConflict::NotRequired
                },
            ));
        }
        let mut duplicate = None;
        let mut conflict = None;
        let mut missing = None;
        let mut current = Some(root);
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Meta)
            {
                if encoding && conflict.is_none() && self.attribute(element, "charset")?.is_some() {
                    self.charge(1)?;
                    conflict = Some(id.clone());
                }
                if &id != subject
                    && duplicate.is_none()
                    && self.pragma_classification(element)?.intrinsic() == Some(&state)
                {
                    self.charge(1)?;
                    duplicate = Some(id.clone());
                }
                if duplicate.is_some() && (!encoding || conflict.is_some()) {
                    break;
                }
            }
            current = match self.ordinary_tree_next(node, frames)? {
                Ok(next) => next,
                Err(gap) => {
                    if missing.is_none() {
                        self.charge(1)?;
                        missing = Some(gap);
                    }
                    self.ordinary_tree_resume(frames)?
                }
            };
        }
        self.charge(1)?;
        let uniqueness = match duplicate {
            Some(id) => HtmlMetaPragmaUniqueness::Duplicate(id),
            None => missing.clone().map_or(
                HtmlMetaPragmaUniqueness::Unique,
                HtmlMetaPragmaUniqueness::Missing,
            ),
        };
        let charset_conflict = if !encoding {
            HtmlMetaEncodingCharsetConflict::NotRequired
        } else {
            match conflict {
                Some(id) => HtmlMetaEncodingCharsetConflict::Present(id),
                None => missing.map_or(
                    HtmlMetaEncodingCharsetConflict::Absent,
                    HtmlMetaEncodingCharsetConflict::Missing,
                ),
            }
        };
        Ok((uniqueness, charset_conflict))
    }
}
