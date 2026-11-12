//! HTML's local script type meaning and type/language classification.
//! External MIME author grammar, script contents, preparation metadata and execution
//! remain separate. No string here is parsed as a MIME record or a JS/JSON program.

use super::*;
use crate::{
    internal::elements::script_type::HtmlScriptKind,
    internal::elements::script_type::HtmlScriptTypeCursor,
};

/// The three HTML-defined authored type keywords. Classic is expressed by
/// omission; data blocks retain an authored MIME string rather than a keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptTypeKeyword {
    /// The `module` keyword: the script is a module script (HTML877d #the-script-element).
    Module,
    /// The `importmap` keyword: the script body is an import map.
    ImportMap,
    /// The `speculationrules` keyword: the script body is a speculation rules document.
    SpeculationRules,
}
impl HtmlScriptTypeKeyword {
    /// The keyword's exact lowercase spelling.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::ImportMap => "importmap",
            Self::SpeculationRules => "speculationrules",
        }
    }
    /// Canonical no-namespace type emission, with ordinary native allocation.
    /// This proves the type keyword, not source/content or placement conformance.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Type.qualified_name(), self.keyword())
    }
}

/// Meaning of the type content attribute alone, before legacy language fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptTypeMeaning {
    /// The `type` value selects a script kind through the standard's keywords or a JavaScript MIME
    /// type essence match. An omitted or empty `type` selects `Classic`.
    Script(HtmlScriptKind),
    /// The `type` value is any other nonempty string, so the element is a data block that the user
    /// agent does not process (HTML877d #the-script-element).
    DataBlock,
}
/// Local author checking, independent of content meaning and processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptTypeAuthorState {
    /// The rule does not apply because the element is not an HTML `script` or the supplied
    /// attribute is not `type`.
    Unapplied,
    /// The HTML-defined omitted/empty/essence/keyword branch is established.
    /// Independent script contents/context are not certified; see author advice.
    HtmlDefined,
    /// Data-block authoring requires a valid external MIME string not matching a
    /// JavaScript essence. This operation has not checked that MIME grammar.
    ExternalMimeUnchecked,
}
/// Source author recommendation, separate from a hard local syntax requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptTypeAuthorAdvice {
    /// No recommendation applies.
    None,
    /// A present `type` that only restates classic script is redundant, and the standard advises
    /// omitting it.
    OmitRedundantType,
}

/// Exact standalone content observation. Missing denotes nominal classic;
/// actual preparation classification may additionally depend on language.
/// Every applicable present value leaves wider author context Unchecked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlScriptTypeReport {
    element: QualifiedName,
    raw: Option<Attribute>,
    applicability: HtmlAttributeApplicability,
    observation: HtmlAttributeObservation,
    meaning: Option<HtmlScriptTypeMeaning>,
    author: HtmlScriptTypeAuthorState,
    advice: HtmlScriptTypeAuthorAdvice,
}
impl HtmlScriptTypeReport {
    /// The exact element name that was supplied, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        &self.element
    }
    /// The exact supplied `type` attribute, or `None` when it was absent.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.raw.as_ref()
    }
    /// The standard attribute this report is about, `type`.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        KnownHtmlAttribute::Type
    }
    /// The row indices of the pinned specification's attribute index that this rule reproduces.
    pub fn inventory_row_indices(&self) -> &'static [usize] {
        &[171]
    }
    /// The URL of the `type` attribute's definition in the pinned HTML Standard snapshot.
    pub fn source_clause(&self) -> &'static str {
        source!("attr-script-type")
    }
    /// Whether the `type` rule applies to the supplied element.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.applicability
    }
    /// Whether `type` was absent, present and empty, or present and nonempty.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.observation
    }
    /// The meaning of the `type` attribute alone, or `None` when the rule does not apply. An absent
    /// attribute means nominal classic script.
    pub fn meaning(&self) -> Option<HtmlScriptTypeMeaning> {
        self.meaning
    }
    /// The local author verdict for the value.
    pub fn author(&self) -> HtmlScriptTypeAuthorState {
        self.author
    }
    /// The author recommendation for the value, which is advice and not a conformance requirement.
    pub fn author_advice(&self) -> HtmlScriptTypeAuthorAdvice {
        self.advice
    }
    /// The wider author-context verdict: `Unapplied` for the wrong element or attribute,
    /// `Unchecked` for any present value, and `NotRequired` for an absent attribute.
    pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
        if self.applicability != HtmlAttributeApplicability::Applicable {
            HtmlAttributeContextAuthorState::Unapplied
        } else if self.raw.is_some() {
            HtmlAttributeContextAuthorState::Unchecked
        } else {
            HtmlAttributeContextAuthorState::NotRequired
        }
    }
}

/// Result of only the type/language step in prepare-the-script-element. NoScript
/// denotes the source's early return with type left null, not missing evidence.
/// It neither installs saved metadata nor claims script preparation/execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptTypeClassification {
    /// The rule does not apply because the element is not an HTML `script` or the supplied
    /// attribute is not `type`.
    Unapplied,
    /// The step selects this script kind.
    Script(HtmlScriptKind),
    /// The step returns early with the script type left null, so no script is prepared.
    NoScript,
}
/// The `type` observation together with the `language` slot and the resulting preparation
/// classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlScriptTypePreparationReport {
    content: HtmlScriptTypeReport,
    language: Option<Attribute>,
    classification: HtmlScriptTypeClassification,
}
impl HtmlScriptTypePreparationReport {
    /// The content-only report for the `type` attribute, computed independently of `language`.
    pub fn content(&self) -> &HtmlScriptTypeReport {
        &self.content
    }
    /// Exact selected obsolete language slot, not proof of language conformance.
    pub fn language_attribute(&self) -> Option<&Attribute> {
        self.language.as_ref()
    }
    /// The classification that the type and language step of preparing a script element produces.
    pub fn classification(&self) -> HtmlScriptTypeClassification {
        self.classification
    }
    /// The URL of the pinned standard's prepare-the-script-element algorithm.
    pub fn source_clause(&self) -> &'static str {
        source!("script-processing-prepare")
    }
}

impl HtmlValueInterpreter {
    /// Observes the local type meaning and HTML-owned author requirements.
    /// Original raw-unit admission precedes exact expanded-name applicability.
    /// No MIME author validator, preparation context or language is inferred.
    /// Reports use ordinary allocation and a per-value UTF-16 cap; this is not a
    /// controlled session/work/cancellation or aggregate memory guarantee.
    pub fn script_type(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
    ) -> Result<HtmlScriptTypeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let applicability = applies(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Type,
                rows: &[171],
                source: source!("attr-script-type"),
                context: Context::Elements(&[KnownHtmlElement::Script]),
                vocabulary_source: None,
            },
            element,
            raw,
        );
        let mut report = HtmlScriptTypeReport {
            element: capture_name(element),
            raw: raw.map(capture_attribute),
            applicability,
            observation: match raw {
                None => HtmlAttributeObservation::Missing,
                Some(raw) if raw.value().is_empty() => HtmlAttributeObservation::PresentEmpty,
                Some(_) => HtmlAttributeObservation::PresentNonempty,
            },
            meaning: None,
            author: HtmlScriptTypeAuthorState::Unapplied,
            advice: HtmlScriptTypeAuthorAdvice::None,
        };
        if applicability != HtmlAttributeApplicability::Applicable {
            return Ok(report);
        }
        let kind = HtmlScriptTypeCursor::content(
            raw.map(|raw| HtmlAttributeInput::value(raw).code_units()),
        )
        .finish_ordinary();
        report.meaning = Some(kind.map_or(
            HtmlScriptTypeMeaning::DataBlock,
            HtmlScriptTypeMeaning::Script,
        ));
        report.author = if kind.is_some() {
            HtmlScriptTypeAuthorState::HtmlDefined
        } else {
            HtmlScriptTypeAuthorState::ExternalMimeUnchecked
        };
        if raw.is_some() && kind == Some(HtmlScriptKind::Classic) {
            report.advice = HtmlScriptTypeAuthorAdvice::OmitRedundantType;
        }
        Ok(report)
    }

    /// Computes the type/language classification from a complete actual ordered
    /// attribute set, independently of the content-only meaning. Both original
    /// selected slots are admitted before applicability or processing, even when
    /// type will override language. Only explicit JavaScript essence matching
    /// uses ASCII-whitespace stripping; keywords and language do not.
    /// Known absence is supplied by this complete set. This does not implement
    /// earlier preparation guards, metadata changes, MIME/JS/JSON validation or
    /// execution; the ordinary per-value allocation contract is unchanged.
    pub fn script_type_with_attributes(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        attributes: &(impl crate::internal::values::semantic_input::HtmlAttributesInput + ?Sized),
    ) -> Result<HtmlScriptTypePreparationReport, HtmlValueAdmissionError> {
        let name = KnownHtmlAttribute::Type.qualified_name();
        let raw = find_attribute(attributes, name.namespace(), name.local_name());
        let language = find_attribute(
            attributes,
            &Namespace::none(),
            &DocumentString::from("language"),
        );
        self.admit_attribute_raw(raw.as_ref().map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(language.as_ref().map(HtmlAttributeInput::value))?;
        let content = self.script_type(element, raw.as_ref())?;
        let classification = if content.applicability != HtmlAttributeApplicability::Applicable {
            HtmlScriptTypeClassification::Unapplied
        } else {
            HtmlScriptTypeCursor::preparation(
                raw.as_ref()
                    .map(|raw| HtmlAttributeInput::value(raw).code_units()),
                language
                    .as_ref()
                    .map(|language| HtmlAttributeInput::value(language).code_units()),
            )
            .finish_ordinary()
            .map_or(
                HtmlScriptTypeClassification::NoScript,
                HtmlScriptTypeClassification::Script,
            )
        };
        Ok(HtmlScriptTypePreparationReport {
            content,
            language: language.as_ref().map(capture_attribute),
            classification,
        })
    }
}
