//! Actual-field selection and deferred local media report production.
use super::*;
use crate::bounded::ThemeColorNameCursor;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaField {
    Media,
    Name,
}
impl HtmlMediaField {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Media => b"media",
            Self::Name => b"name",
        }
    }
}
/// A successful match is bound to the actual immutable candidate and field.
pub struct HtmlMediaFieldMatch<'raw, A: HtmlAttributeInput> {
    attribute: &'raw A,
    field: HtmlMediaField,
}
impl<'raw, A: HtmlAttributeInput> HtmlMediaFieldMatch<'raw, A> {
    pub fn attribute(&self) -> &'raw A {
        self.attribute
    }
    pub fn field(&self) -> HtmlMediaField {
        self.field
    }
}
/// Replaces the comparison in the caller's original first-match traversal.
/// The candidate visit and each reached local-name unit are admitted separately.
pub fn match_media_field<'raw, E, A: HtmlAttributeInput>(
    attribute: &'raw A,
    field: HtmlMediaField,
    mut admit: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<HtmlMediaFieldMatch<'raw, A>>, E> {
    admit(1)?;
    if attribute.name().namespace_uri().is_some() {
        return Ok(None);
    }
    let units = attribute.name().local_name().code_units();
    if units.len() != field.bytes().len() {
        return Ok(None);
    }
    for (unit, byte) in units.iter().zip(field.bytes()) {
        admit(1)?;
        if *unit != u16::from(*byte) {
            return Ok(None);
        }
    }
    Ok(Some(HtmlMediaFieldMatch { attribute, field }))
}
/// Ordinary matching shares the same rule and performs no controlled admission.
pub fn media_field_matches(
    attribute: &impl crate::HtmlAttributeInput,
    field: HtmlMediaField,
) -> bool {
    match match_media_field(attribute, field, |_| Ok::<_, std::convert::Infallible>(())) {
        Ok(value) => value.is_some(),
        Err(never) => match never {},
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlMediaInterpretationError {
    Consumed,
    NotReady,
    Stopped,
    WrongPhase,
    NameAlreadySupplied,
    NameNotSupplied,
    WrongField,
    RepresentationExhausted,
}
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlMediaActionError<E> {
    Admission(E),
    Protocol(HtmlMediaInterpretationError),
}
#[derive(Clone, Copy)]
enum Name<'raw, A: HtmlAttributeInput> {
    NotSupplied,
    SuppliedAbsent,
    Present(&'raw A),
}
/// Retains actual operands and local decisions; semantic processing remains external.
pub struct HtmlMediaInterpretation<'raw, N: HtmlNameInput + ?Sized, A: HtmlAttributeInput> {
    element: &'raw N,
    raw: Option<&'raw A>,
    raw_field: Option<bool>,
    name: Name<'raw, A>,
    name_field: Option<bool>,
    media_admitted: bool,
    name_admitted: bool,
    html_namespace: Option<bool>,
    namespace_code_units: usize,
    role: Option<Option<HtmlMediaElementRole>>,
    value: Option<HtmlMediaValueCursor<'raw>>,
    theme: Option<ThemeColorNameCursor<'raw>>,
    report: Option<HtmlMediaAttributeReport>,
    stopped: bool,
    applicability: Option<HtmlAttributeApplicability>,
}
impl<'raw, N: HtmlNameInput + ?Sized, A: HtmlAttributeInput> HtmlMediaInterpretation<'raw, N, A> {
    pub(super) fn ordinary(element: &'raw N, raw: Option<&'raw A>) -> Self {
        Self {
            element,
            raw,
            raw_field: None,
            name: Name::NotSupplied,
            name_field: None,
            media_admitted: false,
            name_admitted: false,
            html_namespace: None,
            namespace_code_units: 0,
            role: None,
            value: None,
            theme: None,
            report: None,
            stopped: false,
            applicability: None,
        }
    }
    /// Uses the candidate already matched during the caller's original traversal.
    pub fn from_matched(element: &'raw N, raw: Option<HtmlMediaFieldMatch<'raw, A>>) -> Self {
        let mut owner = Self::ordinary(element, raw.as_ref().map(|m| m.attribute));
        owner.raw_field = Some(raw.is_none_or(|m| m.field == HtmlMediaField::Media));
        owner
    }
    fn require(&self, valid: bool) -> Result<(), HtmlMediaInterpretationError> {
        if self.stopped {
            Err(HtmlMediaInterpretationError::Stopped)
        } else if valid && self.report.is_none() {
            Ok(())
        } else {
            Err(HtmlMediaInterpretationError::WrongPhase)
        }
    }
    fn execute<E>(
        &mut self,
        admit: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        if let Err(error) = admit() {
            self.stopped = true;
            Err(HtmlMediaActionError::Admission(error))
        } else {
            Ok(())
        }
    }
    pub fn admit_media<E>(
        &mut self,
        admit: impl FnOnce(Option<&'raw A>) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(!self.media_admitted)
            .map_err(HtmlMediaActionError::Protocol)?;
        let raw = self.raw;
        self.execute(|| admit(raw))?;
        self.media_admitted = true;
        Ok(())
    }
    fn supply(
        &mut self,
        name: Option<&'raw A>,
        matched: Option<bool>,
    ) -> Result<(), HtmlMediaInterpretationError> {
        self.require(self.value.is_none())?;
        if !matches!(self.name, Name::NotSupplied) {
            return Err(HtmlMediaInterpretationError::NameAlreadySupplied);
        }
        self.name = match name {
            Some(a) => Name::Present(a),
            None => Name::SuppliedAbsent,
        };
        self.name_field = matched;
        self.applicability = None;
        Ok(())
    }
    pub(super) fn supply_ordinary_name(
        &mut self,
        name: Option<&'raw A>,
    ) -> Result<(), HtmlMediaInterpretationError> {
        self.supply(name, None)
    }
    pub fn supply_matched_name(
        &mut self,
        name: Option<HtmlMediaFieldMatch<'raw, A>>,
    ) -> Result<(), HtmlMediaInterpretationError> {
        self.require(self.value.is_none())?;
        if name
            .as_ref()
            .is_some_and(|m| m.field != HtmlMediaField::Name)
        {
            return Err(HtmlMediaInterpretationError::WrongField);
        }
        self.supply(name.map(|m| m.attribute), Some(true))
    }
    pub fn admit_name<E>(
        &mut self,
        admit: impl FnOnce(Option<&'raw A>) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(
            self.media_admitted && !self.name_admitted && !matches!(self.name, Name::NotSupplied),
        )
        .map_err(HtmlMediaActionError::Protocol)?;
        let name = self.name_attribute();
        self.execute(|| admit(name))?;
        self.name_admitted = true;
        Ok(())
    }
    pub fn inspect_namespace<E>(
        &mut self,
        mut admit: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(self.media_admitted && self.html_namespace.is_none())
            .map_err(HtmlMediaActionError::Protocol)?;
        self.execute(|| admit(1))?;
        let expected = b"http://www.w3.org/1999/xhtml";
        let result = if let Some(uri) = self.element.namespace_uri() {
            self.namespace_code_units = uri.len();
            if uri.len() != expected.len() {
                false
            } else {
                let mut equal = true;
                for (unit, byte) in uri.code_units().iter().zip(expected) {
                    self.execute(|| admit(1))?;
                    if *unit != u16::from(*byte) {
                        equal = false;
                        break;
                    }
                }
                equal
            }
        } else {
            false
        };
        self.html_namespace = Some(result);
        Ok(())
    }
    /// Admit the local-name header before requesting it. Namespace length was
    /// already observed under `inspect_namespace`'s header admission. The second
    /// admission covers the remaining role envelope; together the calls retain
    /// its original namespace-length + (local-length + 1) * vocabulary-size bound.
    pub fn derive_role<E>(
        &mut self,
        mut admit: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(self.html_namespace.is_some() && self.role.is_none())
            .map_err(HtmlMediaActionError::Protocol)?;
        self.execute(|| admit(1))?;
        let units = self.element.local_name().len();
        let work = units
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_add(self.namespace_code_units));
        let Some(work) = work else {
            self.stop();
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationError::RepresentationExhausted,
            ));
        };
        let remaining = work
            .checked_sub(1)
            .expect("nonempty HTML vocabulary includes the prepaid local header");
        self.execute(|| admit(remaining))?;
        self.role = Some(HtmlMediaElementRole::from_known(
            KnownHtmlElement::from_name(self.element),
        ));
        Ok(())
    }
    pub fn is_html_namespace(&self) -> Result<bool, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationError::Stopped);
        }
        self.html_namespace
            .ok_or(HtmlMediaInterpretationError::NotReady)
    }
    pub fn role(&self) -> Result<Option<HtmlMediaElementRole>, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationError::Stopped);
        }
        self.role.ok_or(HtmlMediaInterpretationError::NotReady)
    }
    pub fn begin_value(&mut self) -> Result<(), HtmlMediaInterpretationError> {
        self.require(
            self.role.is_some()
                && self.value.is_none()
                && (matches!(self.name, Name::NotSupplied) || self.name_admitted),
        )?;
        self.value = Some(HtmlMediaValueCursor::new(
            self.raw.map(|a| a.value().code_units()),
        ));
        Ok(())
    }
    pub fn value_action(&self) -> Result<HtmlMediaCursorAction, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationError::Stopped);
        }
        self.value
            .as_ref()
            .map(HtmlMediaValueCursor::next_action)
            .ok_or(HtmlMediaInterpretationError::NotReady)
    }
    pub fn advance_value<E>(
        &mut self,
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        let action = self
            .value_action()
            .map_err(HtmlMediaActionError::Protocol)?;
        self.require(action != HtmlMediaCursorAction::Complete)
            .map_err(HtmlMediaActionError::Protocol)?;
        self.execute(|| {
            admit(if action == HtmlMediaCursorAction::Finish {
                4
            } else {
                1
            })
        })?;
        self.value.as_mut().expect("begun value").advance();
        Ok(())
    }
    pub fn value(&self) -> Result<HtmlMediaValue, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationError::Stopped);
        }
        self.value
            .as_ref()
            .and_then(HtmlMediaValueCursor::result)
            .ok_or(HtmlMediaInterpretationError::NotReady)
    }
    pub fn begin_theme_color<E>(
        &mut self,
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(
            self.value().is_ok()
                && self.theme.is_none()
                && self.role == Some(Some(HtmlMediaElementRole::Meta)),
        )
        .map_err(HtmlMediaActionError::Protocol)?;
        if matches!(self.name, Name::NotSupplied) {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationError::NameNotSupplied,
            ));
        }
        self.execute(|| admit(1))?;
        self.theme = Some(ThemeColorNameCursor::new(
            self.name_attribute().map(|a| a.value().code_units()),
        ));
        Ok(())
    }
    pub fn theme_color_complete(&self) -> Result<bool, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationError::Stopped);
        }
        self.theme
            .as_ref()
            .map(ThemeColorNameCursor::is_complete)
            .ok_or(HtmlMediaInterpretationError::NotReady)
    }
    pub fn advance_theme_color<E>(
        &mut self,
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(self.theme_color_complete() == Ok(false))
            .map_err(HtmlMediaActionError::Protocol)?;
        self.execute(|| admit(1))?;
        self.theme.as_mut().expect("begun name").advance();
        Ok(())
    }
    pub fn theme_color(&self) -> Result<bool, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationError::Stopped);
        }
        let cursor = self
            .theme
            .as_ref()
            .ok_or(HtmlMediaInterpretationError::NotReady)?;
        if !cursor.is_complete() {
            return Err(HtmlMediaInterpretationError::NotReady);
        }
        Ok(cursor.result())
    }
    fn name_attribute(&self) -> Option<&'raw A> {
        match self.name {
            Name::Present(a) => Some(a),
            _ => None,
        }
    }
    pub(super) fn applicability(&mut self) -> HtmlAttributeApplicability {
        if let Some(applicability) = self.applicability {
            return applicability;
        }
        let result = self.compute_applicability();
        self.applicability = Some(result);
        result
    }
    fn compute_applicability(&self) -> HtmlAttributeApplicability {
        if self.html_namespace == Some(false) {
            HtmlAttributeApplicability::ForeignElementNamespace
        } else if !self.raw_field.unwrap_or_else(|| {
            self.raw
                .is_none_or(|a| media_field_matches(a, HtmlMediaField::Media))
        }) {
            HtmlAttributeApplicability::DifferentAttribute
        } else if self.role == Some(None) {
            HtmlAttributeApplicability::DifferentElement
        } else if !matches!(self.name, Name::NotSupplied)
            && (self.role != Some(Some(HtmlMediaElementRole::Meta))
                || !self.name_field.unwrap_or_else(|| {
                    self.name_attribute()
                        .is_none_or(|a| media_field_matches(a, HtmlMediaField::Name))
                }))
        {
            HtmlAttributeApplicability::DifferentContext
        } else {
            HtmlAttributeApplicability::Applicable
        }
    }
    pub fn assemble<E>(
        &mut self,
        admit: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(self.value().is_ok())
            .map_err(HtmlMediaActionError::Protocol)?;
        let applicability = self
            .applicability
            .unwrap_or_else(|| self.compute_applicability());
        let placement = if applicability != HtmlAttributeApplicability::Applicable
            || self.role != Some(Some(HtmlMediaElementRole::Meta))
        {
            HtmlMediaPlacementObservation::Unapplied
        } else if self.raw.is_none() {
            HtmlMediaPlacementObservation::Known(HtmlMetaMediaPlacement::NotSpecified)
        } else if matches!(self.name, Name::NotSupplied) {
            HtmlMediaPlacementObservation::Unchecked
        } else {
            HtmlMediaPlacementObservation::Known(
                if self.theme_color().map_err(HtmlMediaActionError::Protocol)? {
                    HtmlMetaMediaPlacement::AllowedForThemeColor
                } else {
                    HtmlMetaMediaPlacement::Forbidden
                },
            )
        };
        self.execute(admit)?;
        self.applicability = Some(applicability);
        self.report = Some(HtmlMediaAttributeReport {
            raw: self.raw.map(capture_attribute),
            name: self.name_attribute().map(capture_attribute),
            applicability,
            role: self.role.expect("classified"),
            value: self.value().expect("complete value"),
            placement,
        });
        Ok(())
    }
    pub fn stop(&mut self) {
        self.stopped = true;
    }
    pub fn raw(&self) -> Option<&'raw A> {
        self.raw
    }
    pub fn abort(self) -> HtmlMediaInputs<'raw, N, A> {
        HtmlMediaInputs {
            element: self.element,
            raw: self.raw,
            name: match self.name {
                Name::NotSupplied => HtmlMediaNameSupply::NotSupplied,
                Name::SuppliedAbsent => HtmlMediaNameSupply::SuppliedAbsent,
                Name::Present(a) => HtmlMediaNameSupply::Present(a),
            },
        }
    }
    pub fn try_finish(
        slot: &mut Option<Self>,
    ) -> Result<HtmlMediaAttributeReport, HtmlMediaInterpretationError> {
        let owner = slot
            .as_ref()
            .ok_or(HtmlMediaInterpretationError::Consumed)?;
        if owner.stopped {
            return Err(HtmlMediaInterpretationError::Stopped);
        }
        if owner.report.is_none() {
            return Err(HtmlMediaInterpretationError::NotReady);
        }
        Ok(slot
            .take()
            .expect("checked owner")
            .report
            .expect("assembled report"))
    }
}

/// Original name operand supply, with absence distinct from non-selection.
pub enum HtmlMediaNameSupply<'raw, A: HtmlAttributeInput> {
    NotSupplied,
    SuppliedAbsent,
    Present(&'raw A),
}
/// Operands recovered without scanning, sharing or allocation.
pub struct HtmlMediaInputs<'raw, N: HtmlNameInput + ?Sized, A: HtmlAttributeInput> {
    element: &'raw N,
    raw: Option<&'raw A>,
    name: HtmlMediaNameSupply<'raw, A>,
}
impl<'raw, N: HtmlNameInput + ?Sized, A: HtmlAttributeInput> HtmlMediaInputs<'raw, N, A> {
    pub fn element(&self) -> &'raw N {
        self.element
    }
    pub fn raw(&self) -> Option<&'raw A> {
        self.raw
    }
    pub fn name(&self) -> &HtmlMediaNameSupply<'raw, A> {
        &self.name
    }
}
