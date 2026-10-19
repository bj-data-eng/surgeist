//! Local dirname and placeholder text checks over exact raw attributes.
//! Submission entry construction and actual hint presentation are separate.

use super::*;

/// An exact nonempty authored direction submission name.
///
/// Spaces, casing, NULL and isolated surrogate code units remain unchanged.
/// This proves the local dirname value constraint, not an input type state,
/// directionality, successful control or submitted form entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDirectionSubmissionName {
    value: DocumentString,
}

impl HtmlDirectionSubmissionName {
    /// Rejects only the empty string, using ordinary DocumentString allocation.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlAttributeConformanceError> {
        let value = value.into();
        if value.is_empty() {
            return Err(HtmlAttributeConformanceError::DirnameMustNotBeEmpty);
        }
        Ok(Self { value })
    }

    /// Returns the exact accepted UTF-16 text.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }

    /// Moves the accepted text into the canonical no-namespace dirname attribute.
    /// Element/state author requirements remain independently unproved.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Dirname.qualified_name(), self.value)
    }
}

/// An exact authored input placeholder containing no CR or LF code units.
///
/// Empty, other whitespace, NULL and isolated surrogates are allowed by this
/// local text constraint. Input-state eligibility and actual UI presentation
/// are separate. Textarea placeholders have no corresponding newline prohibition
/// and can be authored directly with Attribute/DocumentString.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInputPlaceholder {
    value: DocumentString,
}

impl HtmlInputPlaceholder {
    /// Checks the source's CR/LF prohibition without trimming or normalization.
    /// Allocation follows the ordinary DocumentString contract.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlAttributeConformanceError> {
        let value = value.into();
        if let Some(error) = input_placeholder_error(&value) {
            return Err(error);
        }
        Ok(Self { value })
    }

    /// Returns the exact accepted UTF-16 text.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }

    /// Moves the accepted text into the canonical no-namespace placeholder attribute.
    /// This does not prove an input type state or that a hint is displayed.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(KnownHtmlAttribute::Placeholder.qualified_name(), self.value)
    }
}

fn newline(unit: u16) -> bool {
    matches!(unit, 0x0A | 0x0D)
}

fn input_placeholder_error(value: &DocumentString) -> Option<HtmlAttributeConformanceError> {
    value
        .code_units()
        .iter()
        .position(|&unit| newline(unit))
        .map(
            |code_unit_offset| HtmlAttributeConformanceError::InputPlaceholderContainsNewline {
                code_unit_offset,
            },
        )
}

#[derive(Clone, Copy)]
enum ControlTextRole {
    Dirname,
    Placeholder,
}

impl ControlTextRole {
    fn metadata(self, element: &(impl crate::HtmlNameInput + ?Sized)) -> RuleMetadata {
        let (attribute, rows, source): (_, &'static [u16], _) = match self {
            Self::Dirname => (
                KnownHtmlAttribute::Dirname,
                &[43],
                source!("attr-fe-dirname"),
            ),
            Self::Placeholder => (
                KnownHtmlAttribute::Placeholder,
                &[117],
                if KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Textarea) {
                    source!("attr-textarea-placeholder")
                } else {
                    source!("attr-input-placeholder")
                },
            ),
        };
        RuleMetadata {
            attribute,
            rows,
            source,
            context: Context::Elements(&[KnownHtmlElement::Input, KnownHtmlElement::Textarea]),
            vocabulary_source: None,
        }
    }

    fn input_allows(self, state: HtmlInputType) -> bool {
        use HtmlInputType::{
            Email, Hidden, Number, Password, Search, Submit, Telephone, Text, Url,
        };
        // The normative bookkeeping in HTML §4.10.5.1, not its summary table:
        // Hidden 35201–35204; Text/Search 35318–35329; Telephone 35420–35431;
        // URL 35527–35538; Email 35778–35790; Password 35875–35885;
        // Number 36816–36854; Submit 37823–37847. Hidden/Submit permit only
        // dirname, Number only placeholder; all remaining states forbid both.
        match self {
            Self::Dirname => matches!(
                state,
                Hidden | Text | Search | Telephone | Url | Email | Password | Submit
            ),
            Self::Placeholder => matches!(
                state,
                Text | Search | Telephone | Url | Email | Password | Number
            ),
        }
    }
}

impl HtmlValueInterpreter {
    fn control_text_report<T>(
        &self,
        role: ControlTextRole,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlAttributeReport<T>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut result = report(role.metadata(element), element, raw);
        if let Some(state) = state {
            result.supplied_context = SuppliedContext::Input(state);
            if result.applicability == HtmlAttributeApplicability::Applicable
                && KnownHtmlElement::from_name(element) != Some(KnownHtmlElement::Input)
            {
                result.applicability = HtmlAttributeApplicability::DifferentContext;
                result.context_author = HtmlAttributeContextAuthorState::Unapplied;
            }
        }
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        if raw.is_none() {
            result.recognition = HtmlAttributeRecognition::Missing;
            result.author = HtmlAttributeAuthorState::Missing;
        } else if KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Input) {
            result.context_author = match state {
                None => HtmlAttributeContextAuthorState::Unchecked,
                Some(state) if role.input_allows(state) => {
                    HtmlAttributeContextAuthorState::Conforming
                }
                Some(_) => HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                ),
            };
        }
        Ok(result)
    }

    fn dirname_report(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlAttributeReport<HtmlDirectionSubmissionName>, HtmlValueAdmissionError> {
        let mut result = self.control_text_report(ControlTextRole::Dirname, element, raw, state)?;
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let Some(raw) = raw else { return Ok(result) };
        match HtmlDirectionSubmissionName::new(raw.value().clone()) {
            Ok(name) => {
                result.recognition = HtmlAttributeRecognition::NonemptyText;
                result.author = HtmlAttributeAuthorState::Conforming;
                result.intrinsic = Some(name);
            }
            Err(error) => {
                result.recognition = HtmlAttributeRecognition::Empty;
                result.author = HtmlAttributeAuthorState::Nonconforming(error);
            }
        }
        Ok(result)
    }

    fn placeholder_report(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: Option<HtmlInputType>,
    ) -> Result<HtmlAttributeReport<DocumentString>, HtmlValueAdmissionError> {
        let mut result =
            self.control_text_report(ControlTextRole::Placeholder, element, raw, state)?;
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let Some(raw) = raw else { return Ok(result) };
        if KnownHtmlElement::from_name(element) == Some(KnownHtmlElement::Input)
            && let Some(error) = input_placeholder_error(raw.value())
        {
            result.recognition = HtmlAttributeRecognition::Unrecognized;
            result.author = HtmlAttributeAuthorState::Nonconforming(error);
            result.intrinsic = Some(DocumentString::from_utf16(
                raw.value()
                    .code_units()
                    .iter()
                    .copied()
                    .filter(|&unit| !newline(unit))
                    .collect::<Vec<_>>(),
            ));
        } else {
            result.recognition = if raw.value().is_empty() {
                HtmlAttributeRecognition::Empty
            } else {
                HtmlAttributeRecognition::NonemptyText
            };
            result.author = HtmlAttributeAuthorState::Conforming;
            result.intrinsic = Some(raw.value().clone());
        }
        Ok(result)
    }

    /// Checks dirname's local specified-nonempty author rule on input/textarea.
    /// Input content leaves state eligibility Unchecked; textarea needs no input
    /// state. Applicable absence is Missing without a fabricated default name.
    /// No directionality or submission entry is produced.
    ///
    /// Original raw UTF-16 admission precedes all expanded-name checks. Allocation
    /// is ordinary report/string allocation with the per-value cap, not physical
    /// memory, work or cancellation accounting.
    pub fn dirname(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<HtmlDirectionSubmissionName>, HtmlValueAdmissionError> {
        self.dirname_report(element, raw, None)
    }

    /// Checks dirname with an explicitly supplied actual input type state.
    /// Forbidden authored presence is separate from the local text result.
    /// Absence requires no state-author check. This is an ordinary supplied state,
    /// not a capture/revision certificate; using it for textarea is Unapplied.
    /// It does not execute input ignore rules or submission. Limits match dirname.
    pub fn dirname_for_input(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: HtmlInputType,
    ) -> Result<HtmlAttributeReport<HtmlDirectionSubmissionName>, HtmlValueAdmissionError> {
        self.dirname_report(element, raw, Some(state))
    }

    /// Checks placeholder's local text rule and derives its content projection.
    /// Input rejects authored CR/LF while its projection removes exactly those
    /// units; textarea accepts and retains multiline text, whose line breaks are
    /// handled by rendering. Raw text remains unchanged even when nonconforming.
    /// Empty is conforming; absent is Missing with no fabricated hint.
    ///
    /// Input state conformance is Unchecked for present content. The projection
    /// neither establishes state applicability nor implies actual hint display,
    /// value/focus/autofocus facts or a complete authoring check. Original raw
    /// UTF-16 admission precedes names and normalization. Ordinary allocation and
    /// the per-value cap do not provide physical-memory/work/cancellation limits.
    pub fn placeholder(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<DocumentString>, HtmlValueAdmissionError> {
        self.placeholder_report(element, raw, None)
    }

    /// Adds actual input-state author eligibility to the local placeholder report.
    /// Forbidden presence does not erase local author errors or the content
    /// projection; no hint is applied to an ignored state by this operation.
    /// Missing requires no state-author check. The supplied state is not captured
    /// proof; textarea is Unapplied for this input-only companion. No UI is
    /// executed, and resource limits are the same as placeholder.
    pub fn placeholder_for_input(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: HtmlInputType,
    ) -> Result<HtmlAttributeReport<DocumentString>, HtmlValueAdmissionError> {
        self.placeholder_report(element, raw, Some(state))
    }
}
