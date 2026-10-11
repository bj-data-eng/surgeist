//! Static text-assistance readbacks; input-method execution/preferences are external.
//! Own capitalization is the IDL getter phase. Used capitalization independently
//! selects type/host evidence; used autocorrection and computed suggestions have
//! their own source algorithms. Queries never clone result vectors. Explicit
//! caller Clone follows the existing fact contract outside query accounting.

use super::*;
use crate::{
    HtmlAttributeRecognition, HtmlAttributeReport, HtmlAutocapitalize, HtmlAutocapitalizeState,
    HtmlAutocorrect, HtmlInputType, HtmlWritingSuggestions, HtmlWritingSuggestionsState,
};

/// Own hint with actual content and consulted form-owner evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlOwnAutocapitalizationHint {
    data: Vec<HtmlOwnAutocapitalizationHintData>,
}
#[derive(Clone, Debug, PartialEq)]
struct HtmlOwnAutocapitalizationHintData {
    own_attribute: Option<Attribute>,
    own_report: Option<HtmlAttributeReport<HtmlAutocapitalizeState>>,
    owner: Option<HtmlFormOwner>,
    form_attribute: Option<Attribute>,
    form_report: Option<HtmlAttributeReport<HtmlAutocapitalizeState>>,
    state: HtmlAutocapitalizeState,
}
impl HtmlOwnAutocapitalizationHint {
    pub fn own_attribute(&self) -> Option<&Attribute> {
        self.data[0].own_attribute.as_ref()
    }
    /// Present only for actually processed nonempty own content.
    pub fn own_report(&self) -> Option<&HtmlAttributeReport<HtmlAutocapitalizeState>> {
        self.data[0].own_report.as_ref()
    }
    pub fn owner(&self) -> Option<&HtmlFormOwner> {
        self.data[0].owner.as_ref()
    }
    pub fn form_attribute(&self) -> Option<&Attribute> {
        self.data[0].form_attribute.as_ref()
    }
    pub fn form_report(&self) -> Option<&HtmlAttributeReport<HtmlAutocapitalizeState>> {
        self.data[0].form_report.as_ref()
    }
    pub fn state(&self) -> HtmlAutocapitalizeState {
        self.data[0].state
    }
    /// Canonical autocapitalize IDL getter value, including Default's empty string.
    pub fn idl_value(&self) -> &'static str {
        match self.state() {
            HtmlAutocapitalizeState::Default => "",
            HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::None) => "none",
            HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences) => "sentences",
            HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Words) => "words",
            HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Characters) => "characters",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlUsedAutocapitalizationBranch {
    SuppressedInput,
    Control,
    EditingHost,
}
/// Used hint, distinct from the own getter and from actual input-method effects.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlUsedAutocapitalizationHint {
    data: Vec<HtmlUsedAutocapitalizationHintData>,
}
#[derive(Clone, Debug, PartialEq)]
struct HtmlUsedAutocapitalizationHintData {
    branch: HtmlUsedAutocapitalizationBranch,
    input_type: Option<HtmlAttributeReport<HtmlInputType>>,
    selected_subject: Option<HtmlCaptureSubjectId>,
    own_hint: Option<HtmlOwnAutocapitalizationHint>,
    state: HtmlAutocapitalizeState,
}
impl HtmlUsedAutocapitalizationHint {
    pub fn branch(&self) -> HtmlUsedAutocapitalizationBranch {
        self.data[0].branch
    }
    pub fn input_type(&self) -> Option<&HtmlAttributeReport<HtmlInputType>> {
        self.data[0].input_type.as_ref()
    }
    pub fn selected_subject(&self) -> Option<&HtmlCaptureSubjectId> {
        self.data[0].selected_subject.as_ref()
    }
    pub fn own_hint(&self) -> Option<&HtmlOwnAutocapitalizationHint> {
        self.data[0].own_hint.as_ref()
    }
    pub fn state(&self) -> HtmlAutocapitalizeState {
        self.data[0].state
    }
}

/// Used autocorrection state and its Boolean IDL getter, without input effects.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlUsedAutocorrectionState {
    data: Vec<HtmlUsedAutocorrectionStateData>,
}
#[derive(Clone, Debug, PartialEq)]
struct HtmlUsedAutocorrectionStateData {
    input_type: Option<HtmlAttributeReport<HtmlInputType>>,
    own_report: Option<HtmlAttributeReport<HtmlAutocorrect>>,
    owner: Option<HtmlFormOwner>,
    form_report: Option<HtmlAttributeReport<HtmlAutocorrect>>,
    state: HtmlAutocorrect,
}
impl HtmlUsedAutocorrectionState {
    pub fn input_type(&self) -> Option<&HtmlAttributeReport<HtmlInputType>> {
        self.data[0].input_type.as_ref()
    }
    pub fn own_report(&self) -> Option<&HtmlAttributeReport<HtmlAutocorrect>> {
        self.data[0].own_report.as_ref()
    }
    pub fn owner(&self) -> Option<&HtmlFormOwner> {
        self.data[0].owner.as_ref()
    }
    pub fn form_report(&self) -> Option<&HtmlAttributeReport<HtmlAutocorrect>> {
        self.data[0].form_report.as_ref()
    }
    pub fn state(&self) -> HtmlAutocorrect {
        self.data[0].state
    }
    pub fn idl_value(&self) -> bool {
        self.state() == HtmlAutocorrect::On
    }
}

/// Computed getter state with the deciding element's exact lexical observation.
/// A foreign selecting ancestor is not certified as HTML-author-conforming.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlComputedWritingSuggestions {
    data: Vec<HtmlComputedWritingSuggestionsData>,
}
#[derive(Clone, Debug, PartialEq)]
struct HtmlComputedWritingSuggestionsData {
    selecting_subject: HtmlCaptureSubjectId,
    selecting_attribute: Option<Attribute>,
    recognition: HtmlAttributeRecognition,
    local_state: HtmlWritingSuggestionsState,
    state: HtmlWritingSuggestions,
}
impl HtmlComputedWritingSuggestions {
    pub fn selecting_subject(&self) -> &HtmlCaptureSubjectId {
        &self.data[0].selecting_subject
    }
    pub fn selecting_attribute(&self) -> Option<&Attribute> {
        self.data[0].selecting_attribute.as_ref()
    }
    pub fn recognition(&self) -> HtmlAttributeRecognition {
        self.data[0].recognition
    }
    pub fn local_state(&self) -> HtmlWritingSuggestionsState {
        self.data[0].local_state
    }
    pub fn state(&self) -> HtmlWritingSuggestions {
        self.data[0].state
    }
    pub fn idl_value(&self) -> &'static str {
        match self.state() {
            HtmlWritingSuggestions::True => "true",
            HtmlWritingSuggestions::False => "false",
        }
    }
}

fn assistance_inherits(kind: Option<KnownHtmlElement>) -> bool {
    matches!(
        kind,
        Some(
            KnownHtmlElement::Button
                | KnownHtmlElement::Fieldset
                | KnownHtmlElement::Input
                | KnownHtmlElement::Output
                | KnownHtmlElement::Select
                | KnownHtmlElement::Textarea
        )
    )
}
fn suppressed_type(state: HtmlInputType) -> bool {
    matches!(
        state,
        HtmlInputType::Url | HtmlInputType::Email | HtmlInputType::Password
    )
}

impl<'a> HtmlSemanticSession<'a> {
    fn assistance_unavailable<T, U>(
        &mut self,
        value: HtmlSemanticValue<T>,
    ) -> Result<HtmlSemanticValue<U>, HtmlSemanticFailure> {
        Ok(match value {
            HtmlSemanticValue::Missing(missing) => HtmlSemanticValue::Missing(missing),
            HtmlSemanticValue::Inapplicable => HtmlSemanticValue::Inapplicable,
            HtmlSemanticValue::Unsupported(unsupported) => {
                HtmlSemanticValue::Unsupported(unsupported)
            }
            HtmlSemanticValue::Known(_) => {
                return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
            }
        })
    }
    // One private record, admitted before writing; no successful empty carrier.
    fn assistance_record<T>(&mut self, record: T) -> Result<Vec<T>, HtmlSemanticFailure> {
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(record);
        Ok(data)
    }
    fn assistance_capitalize_report(
        &mut self,
        element: &'a HtmlCapturedElement,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlAutocapitalizeState>, HtmlSemanticFailure> {
        self.admit_intrinsic(element, raw, 6)?;
        let report = self
            .interpreter
            .autocapitalize(element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        self.poll()?;
        Ok(report)
    }
    fn assistance_autocorrect_report(
        &mut self,
        element: &'a HtmlCapturedElement,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlAutocorrect>, HtmlSemanticFailure> {
        self.admit_intrinsic(element, raw, 2)?;
        let report = self
            .interpreter
            .autocorrect(element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        self.poll()?;
        Ok(report)
    }
    pub(super) fn own_autocapitalization_hint(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlOwnAutocapitalizationHint>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "autocapitalize")?;
        self.charge(1)?;
        let own_attribute = raw.cloned();
        let mut own_report = None;
        let mut owner = None;
        let mut form_attribute = None;
        let mut form_report = None;
        let state = if let Some(raw) = raw.filter(|a| !a.value().is_empty()) {
            let report = self.assistance_capitalize_report(element, Some(raw))?;
            let state = report
                .intrinsic()
                .copied()
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            own_report = Some(report);
            state
        } else if assistance_inherits(self.known(element)?) {
            let actual = match self.form_owner(subject)? {
                HtmlSemanticValue::Known(owner) => owner,
                value => return self.assistance_unavailable(value),
            };
            let state = if let Some(form) = actual.owner() {
                let form_node = self.node(form)?;
                let HtmlCapturedNodeKind::Element(form_element) = form_node.kind() else {
                    return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
                };
                let raw = self.attribute(form_element, "autocapitalize")?;
                self.charge(1)?;
                form_attribute = raw.cloned();
                if let Some(raw) = raw.filter(|a| !a.value().is_empty()) {
                    let report = self.assistance_capitalize_report(form_element, Some(raw))?;
                    let state = report
                        .intrinsic()
                        .copied()
                        .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
                    form_report = Some(report);
                    state
                } else {
                    HtmlAutocapitalizeState::Default
                }
            } else {
                HtmlAutocapitalizeState::Default
            };
            owner = Some(actual);
            state
        } else {
            HtmlAutocapitalizeState::Default
        };
        Ok(HtmlSemanticValue::Known(HtmlOwnAutocapitalizationHint {
            data: self.assistance_record(HtmlOwnAutocapitalizationHintData {
                own_attribute,
                own_report,
                owner,
                form_attribute,
                form_report,
                state,
            })?,
        }))
    }
    pub(super) fn used_autocapitalization_hint(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlUsedAutocapitalizationHint>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        let html = self.html_namespace(element)?;
        let kind = if html { self.known(element)? } else { None };
        let input_type = if kind == Some(KnownHtmlElement::Input) {
            Some(self.control_input_type_report(element)?)
        } else {
            None
        };
        self.poll()?;
        let suppressed = if let Some(report) = &input_type {
            suppressed_type(
                report
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?,
            )
        } else {
            false
        };
        let (branch, selected_subject, own_hint, state) = if suppressed {
            (
                HtmlUsedAutocapitalizationBranch::SuppressedInput,
                None,
                None,
                HtmlAutocapitalizeState::Default,
            )
        } else {
            let (branch, selected) = if matches!(
                kind,
                Some(KnownHtmlElement::Input | KnownHtmlElement::Textarea)
            ) {
                self.charge(1)?;
                (HtmlUsedAutocapitalizationBranch::Control, subject.clone())
            } else {
                match self.editing_host_of(subject)? {
                    HtmlSemanticValue::Known(Some(host)) => {
                        (HtmlUsedAutocapitalizationBranch::EditingHost, host)
                    }
                    HtmlSemanticValue::Known(None) => return Ok(HtmlSemanticValue::Inapplicable),
                    value => return self.assistance_unavailable(value),
                }
            };
            let own = match self.own_autocapitalization_hint(&selected)? {
                HtmlSemanticValue::Known(own) => own,
                value => return self.assistance_unavailable(value),
            };
            let state = own.state();
            (branch, Some(selected), Some(own), state)
        };
        Ok(HtmlSemanticValue::Known(HtmlUsedAutocapitalizationHint {
            data: self.assistance_record(HtmlUsedAutocapitalizationHintData {
                branch,
                input_type,
                selected_subject,
                own_hint,
                state,
            })?,
        }))
    }
    pub(super) fn used_autocorrection_state(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlUsedAutocorrectionState>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let kind = self.known(element)?;
        let input_type = if kind == Some(KnownHtmlElement::Input) {
            Some(self.control_input_type_report(element)?)
        } else {
            None
        };
        self.poll()?;
        let suppressed = if let Some(report) = &input_type {
            suppressed_type(
                report
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?,
            )
        } else {
            false
        };
        let mut own_report = None;
        let mut owner = None;
        let mut form_report = None;
        let state = if suppressed {
            HtmlAutocorrect::Off
        } else {
            let raw = self.attribute(element, "autocorrect")?;
            if raw.is_some() {
                let report = self.assistance_autocorrect_report(element, raw)?;
                let state = report
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
                own_report = Some(report);
                state
            } else if assistance_inherits(kind) {
                let actual = match self.form_owner(subject)? {
                    HtmlSemanticValue::Known(owner) => owner,
                    value => return self.assistance_unavailable(value),
                };
                let state = if let Some(form) = actual.owner() {
                    let form_node = self.node(form)?;
                    let HtmlCapturedNodeKind::Element(form_element) = form_node.kind() else {
                        return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
                    };
                    let raw = self.attribute(form_element, "autocorrect")?;
                    let report = self.assistance_autocorrect_report(form_element, raw)?;
                    let state = report
                        .intrinsic()
                        .copied()
                        .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
                    form_report = Some(report);
                    state
                } else {
                    HtmlAutocorrect::On
                };
                owner = Some(actual);
                state
            } else {
                HtmlAutocorrect::On
            }
        };
        Ok(HtmlSemanticValue::Known(HtmlUsedAutocorrectionState {
            data: self.assistance_record(HtmlUsedAutocorrectionStateData {
                input_type,
                own_report,
                owner,
                form_report,
                state,
            })?,
        }))
    }
    pub(super) fn computed_writing_suggestions(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlComputedWritingSuggestions>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
            };
            let raw = self.attribute(element, "writingsuggestions")?;
            self.admit_intrinsic(element, raw, 2)?;
            let processing = self
                .interpreter
                .writingsuggestions_processing(raw.map(Attribute::value))
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
            self.poll()?;
            let local_state = processing
                .intrinsic
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            let state = match local_state {
                HtmlWritingSuggestionsState::Keyword(state) => state,
                HtmlWritingSuggestionsState::Default => match self.parent(node)? {
                    Ok(Some(parent))
                        if matches!(
                            self.node(&parent)?.kind(),
                            HtmlCapturedNodeKind::Element(_)
                        ) =>
                    {
                        current = parent;
                        continue;
                    }
                    Ok(_) => HtmlWritingSuggestions::True,
                    Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
                },
            };
            self.charge(1)?;
            return Ok(HtmlSemanticValue::Known(HtmlComputedWritingSuggestions {
                data: self.assistance_record(HtmlComputedWritingSuggestionsData {
                    selecting_subject: current,
                    selecting_attribute: raw.cloned(),
                    recognition: processing.recognition,
                    local_state,
                    state,
                })?,
            }));
        }
    }
}
