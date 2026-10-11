//! Default-button selection and no-button implicit-submission conditions.
//! HTML 877d6146487f §4.10.22.2. These observations do not dispatch input,
//! activate buttons, validate controls or submit forms.

use super::*;
use crate::HtmlInputType;

/// First owned submit button in ordinary tree order, even when disabled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDefaultButton {
    subject: HtmlCaptureSubjectId,
    form_owner: HtmlFormOwner,
}
impl HtmlDefaultButton {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn form_owner(&self) -> &HtmlFormOwner {
        &self.form_owner
    }
}

/// Exact threshold used by the source's no-submit-button branch.
/// Multiple proves at least two fields; it does not claim an exact count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImplicitSubmissionBlockingFields {
    Zero,
    One,
    Multiple,
}
/// Static branch conditions, separate from UA support and execution eligibility.
/// A DefaultButton does not claim enabledness or successful activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlImplicitSubmissionCondition {
    DefaultButton(HtmlDefaultButton),
    NoButton {
        blocking_fields: HtmlImplicitSubmissionBlockingFields,
    },
}

enum DefaultStop {
    Missing(HtmlSemanticMissing),
    Failure(HtmlSemanticFailure),
}
impl From<HtmlSemanticFailure> for DefaultStop {
    fn from(value: HtmlSemanticFailure) -> Self {
        Self::Failure(value)
    }
}
type DefaultResult<T> = Result<T, DefaultStop>;

impl<'a> HtmlSemanticSession<'a> {
    fn default_form_role(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<()>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        Ok(if self.known(element)? == Some(KnownHtmlElement::Form) {
            HtmlSemanticValue::Known(())
        } else {
            HtmlSemanticValue::Inapplicable
        })
    }

    fn default_tree_root(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> DefaultResult<HtmlCaptureSubjectId> {
        let mut current = subject.clone();
        loop {
            let node = self.node(&current)?;
            match self.parent(node)?.map_err(DefaultStop::Missing)? {
                Some(parent) => current = parent,
                None => return Ok(current),
            }
        }
    }

    fn default_owner(&mut self, subject: &HtmlCaptureSubjectId) -> DefaultResult<HtmlFormOwner> {
        match self.form_owner(subject)? {
            HtmlSemanticValue::Known(owner) => Ok(owner),
            HtmlSemanticValue::Missing(missing) => Err(DefaultStop::Missing(missing)),
            _ => Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into()),
        }
    }

    pub(super) fn form_default_button(
        &mut self,
        form: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlDefaultButton>>, HtmlSemanticFailure> {
        match self.default_form_role(form)? {
            HtmlSemanticValue::Known(()) => {}
            value => return Ok(value.map(|()| None)),
        }
        let mut frames = Vec::new();
        let result = self.default_button_inner(form, &mut frames);
        self.release_scratch(frames.capacity());
        match result {
            Ok(value) => Ok(HtmlSemanticValue::Known(value)),
            Err(DefaultStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(DefaultStop::Failure(failure)) => Err(failure),
        }
    }

    fn default_button_inner(
        &mut self,
        form: &HtmlCaptureSubjectId,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> DefaultResult<Option<HtmlDefaultButton>> {
        let mut current = Some(self.default_tree_root(form)?);
        while let Some(subject) = current {
            let node = self.node(&subject)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                let kind = self.known(element)?;
                if matches!(
                    kind,
                    Some(KnownHtmlElement::Input | KnownHtmlElement::Button)
                ) {
                    let role = self.form_submitter_role(&subject, element, kind)?;
                    if role.submit_button != Ok(false) {
                        let owner = self.default_owner(&subject)?;
                        self.charge(1)?;
                        if owner.owner() == Some(form) {
                            role.submit_button.map_err(DefaultStop::Missing)?;
                            self.charge(1)?;
                            return Ok(Some(HtmlDefaultButton {
                                subject,
                                form_owner: owner,
                            }));
                        }
                    }
                }
            }
            current = self
                .ordinary_tree_next(node, frames)?
                .map_err(DefaultStop::Missing)?;
        }
        Ok(None)
    }

    pub(super) fn form_implicit_submission_condition(
        &mut self,
        form: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlImplicitSubmissionCondition>, HtmlSemanticFailure> {
        match self.form_default_button(form)? {
            HtmlSemanticValue::Known(Some(button)) => {
                self.charge(1)?;
                return Ok(HtmlSemanticValue::Known(
                    HtmlImplicitSubmissionCondition::DefaultButton(button),
                ));
            }
            HtmlSemanticValue::Known(None) => {}
            HtmlSemanticValue::Missing(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
            HtmlSemanticValue::Inapplicable => return Ok(HtmlSemanticValue::Inapplicable),
            HtmlSemanticValue::Unsupported(value) => {
                return Ok(HtmlSemanticValue::Unsupported(value));
            }
        }
        let mut frames = Vec::new();
        let result = self.default_blockers_inner(form, &mut frames);
        self.release_scratch(frames.capacity());
        match result {
            Ok(blocking_fields) => {
                self.charge(1)?;
                Ok(HtmlSemanticValue::Known(
                    HtmlImplicitSubmissionCondition::NoButton { blocking_fields },
                ))
            }
            Err(DefaultStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(DefaultStop::Failure(failure)) => Err(failure),
        }
    }

    fn default_blockers_inner(
        &mut self,
        form: &HtmlCaptureSubjectId,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> DefaultResult<HtmlImplicitSubmissionBlockingFields> {
        let mut current = Some(self.default_tree_root(form)?);
        let mut count = HtmlImplicitSubmissionBlockingFields::Zero;
        let mut missing = None;
        while let Some(subject) = current {
            let node = self.node(&subject)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Input)
            {
                let role =
                    self.form_submitter_role(&subject, element, Some(KnownHtmlElement::Input))?;
                let HtmlSubmitterTypeObservation::Input(report) = role.observation else {
                    return Err(self
                        .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        .into());
                };
                let state = report
                    .intrinsic()
                    .copied()
                    .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
                self.charge(1)?;
                if matches!(
                    state,
                    HtmlInputType::Text
                        | HtmlInputType::Search
                        | HtmlInputType::Telephone
                        | HtmlInputType::Url
                        | HtmlInputType::Email
                        | HtmlInputType::Password
                        | HtmlInputType::Date
                        | HtmlInputType::Month
                        | HtmlInputType::Week
                        | HtmlInputType::Time
                        | HtmlInputType::LocalDateTime
                        | HtmlInputType::Number
                ) {
                    match self.default_owner(&subject) {
                        Ok(owner) => {
                            self.charge(1)?;
                            if owner.owner() == Some(form) {
                                self.charge(1)?;
                                count = match count {
                                    HtmlImplicitSubmissionBlockingFields::Zero => {
                                        HtmlImplicitSubmissionBlockingFields::One
                                    }
                                    _ => return Ok(HtmlImplicitSubmissionBlockingFields::Multiple),
                                };
                            }
                        }
                        Err(DefaultStop::Missing(value)) => {
                            self.charge(1)?;
                            if missing.is_none() {
                                missing = Some(value);
                            }
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
            current = self
                .ordinary_tree_next(node, frames)?
                .map_err(DefaultStop::Missing)?;
        }
        match missing {
            Some(value) => Err(DefaultStop::Missing(value)),
            None => Ok(count),
        }
    }
}
