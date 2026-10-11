//! HTML's local navigable target names and four reserved target keywords.
//! Naming or choosing a navigable and selecting an effective target are separate.

use super::*;

/// A local navigable target-name author violation, measured in UTF-16 units.
/// A leading underscore is reported before the combined-character restriction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNavigableTargetError {
    Empty,
    /// Names beginning with underscore are reserved for special keywords.
    ReservedLeadingUnderscore,
    /// The first TAB/LF/CR and first less-than unit are both present.
    TabOrNewlineWithLessThan {
        tab_or_newline_code_unit_offset: usize,
        less_than_code_unit_offset: usize,
    },
}

/// Exact checked local authored name, without a navigable/document identity.
///
/// Only nonempty names not starting with underscore and not containing both
/// TAB/LF/CR and less-than are accepted. Spaces, form feed, NULL and isolated
/// surrogates otherwise remain exact. This is not serialized-character or full
/// element/document conformance. Construction uses ordinary string allocation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlNavigableTargetName {
    value: DocumentString,
}

impl HtmlNavigableTargetName {
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlNavigableTargetError> {
        let value = value.into();
        validate_name(&value)?;
        Ok(Self { value })
    }

    /// Returns the exact accepted code units without normalization.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}

/// The four authored keywords admitted by the name-or-keyword grammar.
/// These do not choose a target or establish navigation permissions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlNavigableTargetKeyword {
    Blank,
    SelfTarget,
    Parent,
    Top,
}

impl HtmlNavigableTargetKeyword {
    fn spelling(self) -> &'static str {
        match self {
            Self::Blank => "_blank",
            Self::SelfTarget => "_self",
            Self::Parent => "_parent",
            Self::Top => "_top",
        }
    }

    /// Emits the canonical lowercase keyword with ordinary string allocation.
    pub fn as_document_string(self) -> DocumentString {
        self.spelling().into()
    }
}

/// A valid local authored name or a reserved keyword, not an effective target.
/// Iframe/object name authoring must use the name-only type instead.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HtmlNavigableTarget {
    Name(HtmlNavigableTargetName),
    Keyword(HtmlNavigableTargetKeyword),
}

impl HtmlNavigableTarget {
    /// Retains exact name units or emits a canonical lowercase keyword.
    pub fn as_document_string(&self) -> DocumentString {
        match self {
            Self::Name(name) => name.value().clone(),
            Self::Keyword(keyword) => keyword.as_document_string(),
        }
    }
}

/// The exact defining role of a local target/name content attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlNavigableTargetAttribute {
    /// a/area target; href presence is a separate author requirement.
    HyperlinkTarget,
    /// base target; actual placement before hyperlinks remains unchecked.
    BaseTarget,
    FormTarget,
    /// button/input formtarget; actual submitter/input-state eligibility is unchecked.
    SubmitterFormtarget,
    /// iframe name admits names only, excluding all four underscore keywords.
    IframeName,
    /// object name admits names only, excluding all four underscore keywords.
    ObjectName,
}

impl HtmlNavigableTargetAttribute {
    fn metadata(self) -> RuleMetadata {
        let (attribute, rows, source, elements): (_, &'static [u16], _, &'static [_]) = match self {
            Self::HyperlinkTarget => (
                KnownHtmlAttribute::Target,
                &[157],
                source!("attr-hyperlink-target"),
                &[KnownHtmlElement::A, KnownHtmlElement::Area],
            ),
            Self::BaseTarget => (
                KnownHtmlAttribute::Target,
                &[158],
                source!("attr-base-target"),
                &[KnownHtmlElement::Base],
            ),
            Self::FormTarget => (
                KnownHtmlAttribute::Target,
                &[159],
                source!("attr-fs-target"),
                &[KnownHtmlElement::Form],
            ),
            Self::SubmitterFormtarget => (
                KnownHtmlAttribute::Formtarget,
                &[60],
                source!("attr-fs-formtarget"),
                &[KnownHtmlElement::Button, KnownHtmlElement::Input],
            ),
            Self::IframeName => (
                KnownHtmlAttribute::Name,
                &[105],
                source!("attr-iframe-name"),
                &[KnownHtmlElement::Iframe],
            ),
            Self::ObjectName => (
                KnownHtmlAttribute::Name,
                &[105],
                source!("attr-object-name"),
                &[KnownHtmlElement::Object],
            ),
        };
        RuleMetadata {
            attribute,
            rows,
            source,
            context: Context::Elements(elements),
            vocabulary_source: None,
        }
    }

    fn admits_keywords(self) -> bool {
        !matches!(self, Self::IframeName | Self::ObjectName)
    }

    fn needs_author_context(self) -> bool {
        matches!(
            self,
            Self::HyperlinkTarget | Self::BaseTarget | Self::SubmitterFormtarget
        )
    }
}

fn validate_name(value: &DocumentString) -> Result<(), HtmlNavigableTargetError> {
    let units = value.code_units();
    let Some(&first) = units.first() else {
        return Err(HtmlNavigableTargetError::Empty);
    };
    if first == 0x5F {
        return Err(HtmlNavigableTargetError::ReservedLeadingUnderscore);
    }
    let mut tab_or_newline = None;
    let mut less_than = None;
    for (offset, &unit) in units.iter().enumerate() {
        if matches!(unit, 0x09 | 0x0A | 0x0D) && tab_or_newline.is_none() {
            tab_or_newline = Some(offset);
        }
        if unit == 0x3C && less_than.is_none() {
            less_than = Some(offset);
        }
        if let (Some(tab_or_newline_code_unit_offset), Some(less_than_code_unit_offset)) =
            (tab_or_newline, less_than)
        {
            return Err(HtmlNavigableTargetError::TabOrNewlineWithLessThan {
                tab_or_newline_code_unit_offset,
                less_than_code_unit_offset,
            });
        }
    }
    Ok(())
}

impl HtmlValueInterpreter {
    /// Checks the selected role's local navigable-name or name-or-keyword rule.
    ///
    /// The complete original raw UTF-16 length is admitted before expanded-name
    /// applicability. Missing remains Missing with no invented default. Raw
    /// records preserve exact units/casing; the payload proves only a local
    /// authored name/keyword. Invalid content having no payload does not imply
    /// that navigation has no target. Iframe/object roles reject keywords.
    ///
    /// Present hyperlink, base and submitter roles leave href, tree-order and
    /// submitter/input-state author requirements Unchecked, even for invalid
    /// local text. This operation does not select an element's effective target,
    /// consult first base, apply malformed-target recovery or name/choose a
    /// navigable. Native values also do not prove full element conformance.
    /// Allocation uses ordinary shared strings/reports plus the per-original raw
    /// cap, without physical-memory, session-work or cancellation guarantees.
    pub fn navigable_target_attribute(
        &self,
        role: HtmlNavigableTargetAttribute,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlNavigableTarget>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut result = report(role.metadata(), element, raw);
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let Some(raw) = raw else {
            result.recognition = HtmlAttributeRecognition::Missing;
            result.author = HtmlAttributeAuthorState::Missing;
            return Ok(result);
        };
        if role.needs_author_context() {
            result.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        if role.admits_keywords() {
            for keyword in [
                HtmlNavigableTargetKeyword::Blank,
                HtmlNavigableTargetKeyword::SelfTarget,
                HtmlNavigableTargetKeyword::Parent,
                HtmlNavigableTargetKeyword::Top,
            ] {
                let spelling = keyword.spelling();
                if ascii_keyword(raw.value(), spelling) {
                    result.recognition = HtmlAttributeRecognition::Keyword {
                        matched: spelling,
                        canonical: spelling,
                    };
                    result.author = HtmlAttributeAuthorState::Conforming;
                    result.intrinsic = Some(HtmlNavigableTarget::Keyword(keyword));
                    return Ok(result);
                }
            }
        }
        match HtmlNavigableTargetName::new(raw.value().clone()) {
            Ok(name) => {
                result.recognition = HtmlAttributeRecognition::NonemptyText;
                result.author = HtmlAttributeAuthorState::Conforming;
                result.intrinsic = Some(HtmlNavigableTarget::Name(name));
            }
            Err(error) => {
                result.recognition = if raw.value().is_empty() {
                    HtmlAttributeRecognition::Empty
                } else {
                    HtmlAttributeRecognition::Unrecognized
                };
                result.author = HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::InvalidNavigableTarget(error),
                );
            }
        }
        Ok(result)
    }
}
