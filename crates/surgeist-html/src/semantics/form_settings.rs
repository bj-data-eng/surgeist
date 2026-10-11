//! Source method/enctype selection and no-validate state for potential HTML
//! input/button submitters.
//! Intrinsic content reports and actual submit-button classification remain
//! separate from IDL reflection, successful submission and execution effects.

use super::*;
use crate::{
    HtmlAttributeReport, HtmlButtonSubmitterContext, HtmlButtonType, HtmlButtonTypeState,
    HtmlFormEnctype, HtmlFormMethod, HtmlInputType,
};

/// The actual type interpretation consulted only for a present override.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSubmitterTypeObservation {
    Input(HtmlAttributeReport<HtmlInputType>),
    Button(HtmlAttributeReport<HtmlButtonTypeState>),
}

pub(super) struct SubmitterRole {
    pub(super) observation: HtmlSubmitterTypeObservation,
    pub(super) submit_button: Result<bool, HtmlSemanticMissing>,
}

macro_rules! form_setting {
    ($result:ident, $selection:ident, $data:ident, $value:ty,
     $query:ident, $override_name:literal, $ordinary:ident, $input:ident, $button:ident,
     $form_name:literal, $form:ident) => {
        /// Source selection distinct from intrinsic content and IDL reflection.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum $selection {
            /// Present override on an actual submit button, including invalid default.
            Override($value),
            Owner {
                value: $value,
                owner: HtmlFormOwner,
            },
            /// Established null owner: the selected clause defines no fallback state.
            NoFormOwner(HtmlFormOwner),
            /// Required classification or association evidence was not established.
            Missing(HtmlSemanticMissing),
        }

        /// Capture-qualified selection for a potential HTML input/button submitter.
        /// Non-submit controls follow the source owner branch, retaining override
        /// author violations. A missing selection never claims an effective state.
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $result {
            // Exactly one fallibly admitted record keeps the public fact compact.
            data: Vec<$data>,
        }
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct $data {
            intrinsic: HtmlAttributeReport<$value>,
            submitter_type: Option<HtmlSubmitterTypeObservation>,
            selection: $selection,
            form_attribute: Option<HtmlAttributeReport<$value>>,
        }
        impl $result {
            /// Actual override content, preserving absence, empty and exact UTF-16.
            pub fn intrinsic(&self) -> &HtmlAttributeReport<$value> {
                &self.data[0].intrinsic
            }
            /// Absent when no type classification was required by this selection.
            pub fn submitter_type(&self) -> Option<&HtmlSubmitterTypeObservation> {
                self.data[0].submitter_type.as_ref()
            }
            pub fn selection(&self) -> &$selection {
                &self.data[0].selection
            }
            /// Present only when a non-null owner's actual form attribute was read.
            pub fn form_attribute(&self) -> Option<&HtmlAttributeReport<$value>> {
                self.data[0].form_attribute.as_ref()
            }
        }

        impl HtmlSemanticSession<'_> {
            pub(super) fn $query(
                &mut self,
                subject: &HtmlCaptureSubjectId,
            ) -> Result<HtmlSemanticValue<$result>, HtmlSemanticFailure> {
                let node = self.node(subject)?;
                let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                    return Ok(HtmlSemanticValue::Inapplicable);
                };
                if !self.html_namespace(element)? {
                    return Ok(HtmlSemanticValue::Unsupported(
                        HtmlSemanticUnsupported::ForeignNamespace,
                    ));
                }
                let kind = self.known(element)?;
                if !matches!(
                    kind,
                    Some(KnownHtmlElement::Input | KnownHtmlElement::Button)
                ) {
                    return Ok(HtmlSemanticValue::Inapplicable);
                }
                let raw = self.attribute(element, $override_name)?;
                self.interpreter
                    .admit_attribute_raw(raw.map(Attribute::value))
                    .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
                // Absent overrides choose the owner branch without type, command,
                // or parent reads. Presence, including empty, needs classification.
                let role = if raw.is_some() {
                    Some(self.form_submitter_role(subject, element, kind)?)
                } else {
                    None
                };
                self.admit_intrinsic(element, raw, 3)?;
                let intrinsic = match &role {
                    Some(SubmitterRole {
                        observation: HtmlSubmitterTypeObservation::Input(report),
                        ..
                    }) => {
                        let state = report.intrinsic().copied().ok_or_else(|| {
                            self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        })?;
                        self.interpreter.$input(element.name(), raw, state)
                    }
                    Some(SubmitterRole {
                        observation: HtmlSubmitterTypeObservation::Button(_),
                        submit_button: Ok(submit_button),
                    }) => self.interpreter.$button(
                        element.name(),
                        raw,
                        if *submit_button {
                            HtmlButtonSubmitterContext::SubmitButton
                        } else {
                            HtmlButtonSubmitterContext::NotSubmitButton
                        },
                    ),
                    _ => self.interpreter.$ordinary(element.name(), raw),
                }
                .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
                self.poll()?;
                let (submitter_type, predicate) = match role {
                    Some(role) => (Some(role.observation), role.submit_button),
                    None => (None, Ok(false)),
                };
                let mut form_attribute = None;
                let selection = match predicate {
                    Err(missing) => $selection::Missing(missing),
                    Ok(true) => {
                        $selection::Override(intrinsic.intrinsic().copied().ok_or_else(|| {
                            self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        })?)
                    }
                    Ok(false) => match self.form_owner(subject)? {
                        HtmlSemanticValue::Known(owner) => {
                            if let Some(form) = owner.owner() {
                                let node = self.node(form)?;
                                let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                                    return Err(
                                        self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                                    );
                                };
                                let raw = self.attribute(element, $form_name)?;
                                self.admit_intrinsic(element, raw, 3)?;
                                let attribute = self
                                    .interpreter
                                    .$form(element.name(), raw)
                                    .map_err(|error| {
                                        self.stop(HtmlSemanticFailureKind::Attribute(error))
                                    })?;
                                self.poll()?;
                                let value = attribute.intrinsic().copied().ok_or_else(|| {
                                    self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                                })?;
                                form_attribute = Some(attribute);
                                $selection::Owner { value, owner }
                            } else {
                                $selection::NoFormOwner(owner)
                            }
                        }
                        HtmlSemanticValue::Missing(missing) => $selection::Missing(missing),
                        _ => return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
                    },
                };
                let mut data = Vec::new();
                self.reserve_nested_output(&mut data, 1)?;
                self.charge(1)?;
                data.push($data {
                    intrinsic,
                    submitter_type,
                    selection,
                    form_attribute,
                });
                Ok(HtmlSemanticValue::Known($result { data }))
            }
        }
    };
}

form_setting!(
    HtmlSubmitterFormMethod,
    HtmlFormMethodSelection,
    FormMethodData,
    HtmlFormMethod,
    submitter_form_method,
    "formmethod",
    formmethod,
    formmethod_for_input,
    formmethod_for_button,
    "method",
    form_method
);
form_setting!(
    HtmlSubmitterFormEnctype,
    HtmlFormEnctypeSelection,
    FormEnctypeData,
    HtmlFormEnctype,
    submitter_form_enctype,
    "formenctype",
    formenctype,
    formenctype_for_input,
    formenctype_for_button,
    "enctype",
    form_enctype
);

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn form_submitter_role(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        kind: Option<KnownHtmlElement>,
    ) -> Result<SubmitterRole, HtmlSemanticFailure> {
        let raw = self.attribute(element, "type")?;
        if kind == Some(KnownHtmlElement::Input) {
            self.admit_intrinsic(element, raw, 22)?;
            let report = self
                .interpreter
                .input_type(element.name(), raw)
                .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
            self.poll()?;
            let state = report
                .intrinsic()
                .copied()
                .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            return Ok(SubmitterRole {
                observation: HtmlSubmitterTypeObservation::Input(report),
                submit_button: Ok(matches!(
                    state,
                    HtmlInputType::Submit | HtmlInputType::Image
                )),
            });
        }
        self.admit_intrinsic(element, raw, 3)?;
        let report = self
            .interpreter
            .button_type(element.name(), raw)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        self.poll()?;
        let state = report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        let submit_button = match state {
            HtmlButtonTypeState::Keyword(HtmlButtonType::Submit) => Ok(true),
            HtmlButtonTypeState::Keyword(HtmlButtonType::Button | HtmlButtonType::Reset) => {
                Ok(false)
            }
            HtmlButtonTypeState::Auto => {
                if self.attribute(element, "command")?.is_some()
                    || self.attribute(element, "commandfor")?.is_some()
                {
                    Ok(false)
                } else {
                    let node = self.node(subject)?;
                    match self.parent(node)? {
                        Err(missing) => Err(missing),
                        Ok(None) => Ok(true),
                        Ok(Some(parent)) => {
                            let node = self.node(&parent)?;
                            let is_select = match node.kind() {
                                HtmlCapturedNodeKind::Element(parent) => {
                                    self.known(parent)? == Some(KnownHtmlElement::Select)
                                }
                                _ => false,
                            };
                            Ok(!is_select)
                        }
                    }
                }
            }
        };
        Ok(SubmitterRole {
            observation: HtmlSubmitterTypeObservation::Button(report),
            submit_button,
        })
    }
}

/// The sufficient source evidence for no-validate, distinct from Boolean authorship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlNoValidateResolution {
    /// An actual submit button has its own formnovalidate attribute present.
    OwnSubmitter,
    FormOwner {
        value: bool,
        owner: HtmlFormOwner,
    },
    /// The own branch is false and the actual owner is null: false otherwise.
    NoFormOwner(HtmlFormOwner),
    Missing {
        missing: HtmlSemanticMissing,
        /// A consulted known pointer/null remains evidence despite missing eligibility.
        owner: Option<HtmlFormOwner>,
    },
}

/// Static no-validate readback for potential HTML input/button submitters.
/// This does not perform validation, select a successful submitter or reflect IDL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSubmitterNoValidate {
    // Exactly one record, reserved through the same finite semantic ledger.
    data: Vec<NoValidateData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct NoValidateData {
    intrinsic: HtmlAttributeReport<bool>,
    submitter_type: Option<HtmlSubmitterTypeObservation>,
    resolution: HtmlNoValidateResolution,
    form_attribute: Option<HtmlAttributeReport<bool>>,
}
impl HtmlSubmitterNoValidate {
    /// Exact formnovalidate content presence, independent of its contextual effect.
    pub fn intrinsic(&self) -> &HtmlAttributeReport<bool> {
        &self.data[0].intrinsic
    }
    pub fn submitter_type(&self) -> Option<&HtmlSubmitterTypeObservation> {
        self.data[0].submitter_type.as_ref()
    }
    pub fn resolution(&self) -> &HtmlNoValidateResolution {
        &self.data[0].resolution
    }
    /// Actual form novalidate report, only when a non-null owner was consulted.
    pub fn form_attribute(&self) -> Option<&HtmlAttributeReport<bool>> {
        self.data[0].form_attribute.as_ref()
    }
    /// None is unavailable necessary evidence, never established false.
    pub fn value(&self) -> Option<bool> {
        match self.resolution() {
            HtmlNoValidateResolution::OwnSubmitter => Some(true),
            HtmlNoValidateResolution::FormOwner { value, .. } => Some(*value),
            HtmlNoValidateResolution::NoFormOwner(_) => Some(false),
            HtmlNoValidateResolution::Missing { .. } => None,
        }
    }
}

impl HtmlSemanticSession<'_> {
    pub(super) fn submitter_no_validate(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlSubmitterNoValidate>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let kind = self.known(element)?;
        if !matches!(
            kind,
            Some(KnownHtmlElement::Input | KnownHtmlElement::Button)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "formnovalidate")?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        // Without the attribute, the own conjunction is false independently
        // of type and ordinary parent. Reuse the accepted classifier otherwise.
        let role = if raw.is_some() {
            Some(self.form_submitter_role(subject, element, kind)?)
        } else {
            None
        };
        self.admit_intrinsic(element, raw, 1)?;
        let intrinsic = match &role {
            Some(SubmitterRole {
                observation: HtmlSubmitterTypeObservation::Input(report),
                ..
            }) => {
                let state = report
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
                self.interpreter.boolean_attribute_for_input(
                    HtmlBooleanAttribute::Formnovalidate,
                    element.name(),
                    raw,
                    state,
                )
            }
            Some(SubmitterRole {
                observation: HtmlSubmitterTypeObservation::Button(_),
                submit_button: Ok(is_submitter),
            }) => self.interpreter.formnovalidate_for_button(
                element.name(),
                raw,
                if *is_submitter {
                    HtmlButtonSubmitterContext::SubmitButton
                } else {
                    HtmlButtonSubmitterContext::NotSubmitButton
                },
            ),
            _ => self.interpreter.boolean_attribute(
                HtmlBooleanAttribute::Formnovalidate,
                element.name(),
                raw,
            ),
        }
        .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        self.poll()?;
        // Intrinsic presence must remain available even when author context is
        // prohibited or unknown; it is not itself the no-validate result.
        let present = intrinsic
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
        let (submitter_type, own) = match role {
            Some(role) => (
                Some(role.observation),
                role.submit_button.map(|submit| submit && present),
            ),
            None => (None, Ok(false)),
        };
        let mut form_attribute = None;
        let resolution = if own == Ok(true) {
            HtmlNoValidateResolution::OwnSubmitter
        } else {
            match self.form_owner(subject)? {
                HtmlSemanticValue::Known(owner) => {
                    if let Some(form) = owner.owner() {
                        let node = self.node(form)?;
                        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                            return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
                        };
                        let raw = self.attribute(element, "novalidate")?;
                        self.admit_intrinsic(element, raw, 1)?;
                        let report = self
                            .interpreter
                            .boolean_attribute(
                                HtmlBooleanAttribute::Novalidate,
                                element.name(),
                                raw,
                            )
                            .map_err(|error| {
                                self.stop(HtmlSemanticFailureKind::Attribute(error))
                            })?;
                        self.poll()?;
                        let value = report.intrinsic().copied().ok_or_else(|| {
                            self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        })?;
                        form_attribute = Some(report);
                        // A true owner disjunct proves true despite unavailable
                        // own classification. False still needs both branches.
                        match (value, own) {
                            (false, Err(missing)) => HtmlNoValidateResolution::Missing {
                                missing,
                                owner: Some(owner),
                            },
                            _ => HtmlNoValidateResolution::FormOwner { value, owner },
                        }
                    } else if let Err(missing) = own {
                        HtmlNoValidateResolution::Missing {
                            missing,
                            owner: Some(owner),
                        }
                    } else {
                        HtmlNoValidateResolution::NoFormOwner(owner)
                    }
                }
                HtmlSemanticValue::Missing(missing) => HtmlNoValidateResolution::Missing {
                    missing,
                    owner: None,
                },
                _ => return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
            }
        };
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(NoValidateData {
            intrinsic,
            submitter_type,
            resolution,
            form_attribute,
        });
        Ok(HtmlSemanticValue::Known(HtmlSubmitterNoValidate { data }))
    }
}
