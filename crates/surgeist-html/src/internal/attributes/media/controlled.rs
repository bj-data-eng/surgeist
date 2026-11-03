//! Actual-field selection and deferred local media report production.
use super::*;
use crate::bounded::ThemeColorNameCursor;
/// The two no-namespace attributes that media interpretation matches by local
/// name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMediaField {
    /// The `media` attribute.
    Media,
    /// The `name` attribute, which decides whether a `meta` element may carry
    /// `media`.
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
#[derive(Debug)]
pub struct HtmlMediaFieldMatch<'raw, A: HtmlAttributeInput> {
    attribute: &'raw A,
    field: HtmlMediaField,
}
impl<'raw, A: HtmlAttributeInput> HtmlMediaFieldMatch<'raw, A> {
    /// The matched candidate, borrowed from the caller's original attribute
    /// collection.
    pub fn attribute(&self) -> &'raw A {
        self.attribute
    }
    /// The field that the candidate's local name matched.
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
/// A misuse of the media interpretation protocol, such as a step requested out
/// of order, on a stopped or consumed owner, or before its input is ready.
///
/// The cause is private; ask about it with the `is_*` predicates. A refusal by
/// the caller's own admission is not this error: it is
/// `HtmlMediaActionError::Admission`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlMediaInterpretationError {
    kind: HtmlMediaInterpretationErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlMediaInterpretationErrorKind {
    Consumed,
    NotReady,
    Stopped,
    WrongPhase,
    NameAlreadySupplied,
    NameNotSupplied,
    WrongField,
    RepresentationExhausted,
}

impl HtmlMediaInterpretationErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlMediaInterpretationError {
        HtmlMediaInterpretationError { kind: self }
    }
}

impl From<HtmlMediaInterpretationErrorKind> for HtmlMediaInterpretationError {
    fn from(kind: HtmlMediaInterpretationErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlMediaInterpretationError {
    /// Whether the cause is: media interpretation was already consumed.
    pub fn is_consumed(&self) -> bool {
        matches!(self.kind, HtmlMediaInterpretationErrorKind::Consumed)
    }
    /// Whether the cause is: media interpretation has an unfinished step.
    pub fn is_not_ready(&self) -> bool {
        matches!(self.kind, HtmlMediaInterpretationErrorKind::NotReady)
    }
    /// Whether the cause is: media interpretation was stopped.
    pub fn is_stopped(&self) -> bool {
        matches!(self.kind, HtmlMediaInterpretationErrorKind::Stopped)
    }
    /// Whether the cause is: media interpretation step was requested out of order.
    pub fn is_wrong_phase(&self) -> bool {
        matches!(self.kind, HtmlMediaInterpretationErrorKind::WrongPhase)
    }
    /// Whether the cause is: media interpretation already has its name attribute.
    pub fn is_name_already_supplied(&self) -> bool {
        matches!(
            self.kind,
            HtmlMediaInterpretationErrorKind::NameAlreadySupplied
        )
    }
    /// Whether the cause is: media interpretation has no name attribute yet.
    pub fn is_name_not_supplied(&self) -> bool {
        matches!(self.kind, HtmlMediaInterpretationErrorKind::NameNotSupplied)
    }
    /// Whether the cause is: attribute is not the field this interpretation expects.
    pub fn is_wrong_field(&self) -> bool {
        matches!(self.kind, HtmlMediaInterpretationErrorKind::WrongField)
    }
    /// Whether the cause is: media interpretation size cannot be represented on this platform.
    pub fn is_representation_exhausted(&self) -> bool {
        matches!(
            self.kind,
            HtmlMediaInterpretationErrorKind::RepresentationExhausted
        )
    }
}

impl std::fmt::Display for HtmlMediaInterpretationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlMediaInterpretationErrorKind::Consumed => {
                f.write_str("media interpretation was already consumed")
            }
            HtmlMediaInterpretationErrorKind::NotReady => {
                f.write_str("media interpretation has an unfinished step")
            }
            HtmlMediaInterpretationErrorKind::Stopped => {
                f.write_str("media interpretation was stopped")
            }
            HtmlMediaInterpretationErrorKind::WrongPhase => {
                f.write_str("media interpretation step was requested out of order")
            }
            HtmlMediaInterpretationErrorKind::NameAlreadySupplied => {
                f.write_str("media interpretation already has its name attribute")
            }
            HtmlMediaInterpretationErrorKind::NameNotSupplied => {
                f.write_str("media interpretation has no name attribute yet")
            }
            HtmlMediaInterpretationErrorKind::WrongField => {
                f.write_str("attribute is not the field this interpretation expects")
            }
            HtmlMediaInterpretationErrorKind::RepresentationExhausted => {
                f.write_str("media interpretation size cannot be represented on this platform")
            }
        }
    }
}

impl std::error::Error for HtmlMediaInterpretationError {}

/// A failed controlled media action: the caller's refusal, or a protocol misuse.
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlMediaActionError<E> {
    /// The caller's admission refused the action. The owner is stopped and
    /// accepts no further step except `abort`.
    Admission(E),
    /// The step was requested in the wrong state, or the work to charge cannot
    /// be represented. See `HtmlMediaInterpretationError`.
    Protocol(HtmlMediaInterpretationError),
}
impl<E> From<HtmlMediaInterpretationError> for HtmlMediaActionError<E> {
    fn from(error: HtmlMediaInterpretationError) -> Self {
        Self::Protocol(error)
    }
}
impl<E> std::fmt::Display for HtmlMediaActionError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Admission(_) => "media interpretation was refused by the caller's admission",
            Self::Protocol(_) => "media interpretation protocol was violated",
        })
    }
}
impl<E: std::fmt::Debug + std::error::Error + 'static> std::error::Error
    for HtmlMediaActionError<E>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::Protocol(error) => Some(error),
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Name<'raw, A: HtmlAttributeInput> {
    NotSupplied,
    SuppliedAbsent,
    Present(&'raw A),
}
/// Retains actual operands and local decisions; semantic processing remains external.
///
/// The owner advances through admission, namespace, role, value and (for a
/// `meta` element with a name) theme-color steps, each charged to the caller
/// before it runs. A refused admission stops the owner. `assemble` produces the
/// report and `try_finish` hands it out once. It is not `Clone`, so one owner
/// cannot be advanced twice.
#[derive(Debug)]
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
            Err(HtmlMediaInterpretationErrorKind::Stopped.error())
        } else if valid && self.report.is_none() {
            Ok(())
        } else {
            Err(HtmlMediaInterpretationErrorKind::WrongPhase.error())
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
    /// Admits the `media` operand once, before any other step. `admit` receives
    /// the raw attribute, or `None` when it is absent. A refusal stops the owner.
    pub fn admit_media<E>(
        &mut self,
        admit: impl FnOnce(Option<&'raw A>) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(!self.media_admitted)?;
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
            return Err(HtmlMediaInterpretationErrorKind::NameAlreadySupplied.error());
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
    /// Supplies the `name` operand from the caller's original traversal. `None`
    /// is an established absence. Fails with `WrongField` if the match is for
    /// another field and with `NameAlreadySupplied` on a second supply. It must
    /// precede `begin_value` and needs no earlier `admit_media`; only `admit_name` waits for it.
    pub fn supply_matched_name(
        &mut self,
        name: Option<HtmlMediaFieldMatch<'raw, A>>,
    ) -> Result<(), HtmlMediaInterpretationError> {
        self.require(self.value.is_none())?;
        if name
            .as_ref()
            .is_some_and(|m| m.field != HtmlMediaField::Name)
        {
            return Err(HtmlMediaInterpretationErrorKind::WrongField.error());
        }
        self.supply(name.map(|m| m.attribute), Some(true))
    }
    /// Admits the supplied `name` operand once, after `admit_media`. `admit`
    /// receives the name attribute, or `None` when it is absent. Fails with
    /// `WrongPhase` unless a name was supplied. A refusal stops the owner.
    pub fn admit_name<E>(
        &mut self,
        admit: impl FnOnce(Option<&'raw A>) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(
            self.media_admitted && !self.name_admitted && !matches!(self.name, Name::NotSupplied),
        )?;
        let name = self.name_attribute();
        self.execute(|| admit(name))?;
        self.name_admitted = true;
        Ok(())
    }
    /// Compares the element's namespace URI with the HTML namespace after
    /// `admit_media`. `admit` is charged one unit for the header and, when the
    /// URI is as long as the HTML namespace, one more for each code unit
    /// compared; comparison ends at the first difference. A refusal stops the
    /// owner. Read the result with `is_html_namespace`.
    pub fn inspect_namespace<E>(
        &mut self,
        mut admit: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(self.media_admitted && self.html_namespace.is_none())?;
        self.execute(|| admit(1))?;
        let expected = crate::internal::values::HTML_NAMESPACE_URI.as_bytes();
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
        self.require(self.html_namespace.is_some() && self.role.is_none())?;
        self.execute(|| admit(1))?;
        let units = self.element.local_name().len();
        let work = units
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| n.checked_add(self.namespace_code_units));
        let Some(work) = work else {
            self.stop();
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::RepresentationExhausted.error(),
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
    /// Whether the element is in the HTML namespace. `NotReady` before
    /// `inspect_namespace`; `Stopped` after a stop.
    pub fn is_html_namespace(&self) -> Result<bool, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationErrorKind::Stopped.error());
        }
        self.html_namespace
            .ok_or(HtmlMediaInterpretationErrorKind::NotReady.error())
    }
    /// The element role: `Some` for `link`, `style`, `source` or `meta`, `None`
    /// for any other element. `NotReady` before `derive_role`.
    pub fn role(&self) -> Result<Option<HtmlMediaElementRole>, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationErrorKind::Stopped.error());
        }
        self.role
            .ok_or(HtmlMediaInterpretationErrorKind::NotReady.error())
    }
    /// Starts the value scan once the role is derived, and the name is admitted
    /// if one was supplied. The scan borrows the raw value and allocates
    /// nothing.
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
    /// The next announced action of the value scan. `NotReady` before
    /// `begin_value`.
    pub fn value_action(&self) -> Result<HtmlMediaCursorAction, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationErrorKind::Stopped.error());
        }
        self.value
            .as_ref()
            .map(HtmlMediaValueCursor::next_action)
            .ok_or(HtmlMediaInterpretationErrorKind::NotReady.error())
    }
    /// Admits and performs the announced value action: one unit for inspecting a
    /// code unit, four for the final classification. `WrongPhase` once the scan
    /// is complete or the report is assembled. A refusal stops the owner.
    pub fn advance_value<E>(
        &mut self,
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        if self.stopped {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::Stopped.error(),
            ));
        }
        let Some(cursor) = self.value.as_mut() else {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::NotReady.error(),
            ));
        };
        let action = cursor.next_action();
        if action == HtmlMediaCursorAction::Complete || self.report.is_some() {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::WrongPhase.error(),
            ));
        }
        if let Err(error) = admit(if action == HtmlMediaCursorAction::Finish {
            4
        } else {
            1
        }) {
            self.stopped = true;
            return Err(HtmlMediaActionError::Admission(error));
        }
        cursor.advance();
        Ok(())
    }
    /// The finished value classification. `NotReady` until `value_action`
    /// reports `Complete`.
    pub fn value(&self) -> Result<HtmlMediaValue, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationErrorKind::Stopped.error());
        }
        self.value
            .as_ref()
            .and_then(HtmlMediaValueCursor::result)
            .ok_or(HtmlMediaInterpretationErrorKind::NotReady.error())
    }
    /// Starts the comparison of the `name` with `theme-color`, charging one
    /// unit. Allowed only for a `meta` role after the value is complete. Fails
    /// with `NameNotSupplied` if no name was supplied (an absent name is fine
    /// and never equals `theme-color`).
    pub fn begin_theme_color<E>(
        &mut self,
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(
            self.value().is_ok()
                && self.theme.is_none()
                && self.role == Some(Some(HtmlMediaElementRole::Meta)),
        )?;
        if matches!(self.name, Name::NotSupplied) {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::NameNotSupplied.error(),
            ));
        }
        self.execute(|| admit(1))?;
        self.theme = Some(ThemeColorNameCursor::new(
            self.name_attribute().map(|a| a.value().code_units()),
        ));
        Ok(())
    }
    /// Whether the `theme-color` comparison has finished. `NotReady` before
    /// `begin_theme_color`.
    pub fn theme_color_complete(&self) -> Result<bool, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationErrorKind::Stopped.error());
        }
        self.theme
            .as_ref()
            .map(ThemeColorNameCursor::is_complete)
            .ok_or(HtmlMediaInterpretationErrorKind::NotReady.error())
    }
    /// Admits one unit and compares the next code unit of the name. `WrongPhase`
    /// when not started, already complete, or the report is assembled.
    pub fn advance_theme_color<E>(
        &mut self,
        admit: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        if self.stopped {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::Stopped.error(),
            ));
        }
        let Some(cursor) = self.theme.as_mut() else {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::WrongPhase.error(),
            ));
        };
        if cursor.is_complete() || self.report.is_some() {
            return Err(HtmlMediaActionError::Protocol(
                HtmlMediaInterpretationErrorKind::WrongPhase.error(),
            ));
        }
        if let Err(error) = admit(1) {
            self.stopped = true;
            return Err(HtmlMediaActionError::Admission(error));
        }
        cursor.advance();
        Ok(())
    }
    /// Whether the name is an ASCII case-insensitive match for `theme-color`.
    /// `NotReady` until the comparison is complete.
    pub fn theme_color(&self) -> Result<bool, HtmlMediaInterpretationError> {
        if self.stopped {
            return Err(HtmlMediaInterpretationErrorKind::Stopped.error());
        }
        let cursor = self
            .theme
            .as_ref()
            .ok_or(HtmlMediaInterpretationErrorKind::NotReady)?;
        if !cursor.is_complete() {
            return Err(HtmlMediaInterpretationErrorKind::NotReady.error());
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
    /// Admits the final report assembly and builds the report once the value is
    /// complete. For an applicable `meta` with both `media` and a supplied name,
    /// the theme-color comparison must be complete too. `try_finish` hands the
    /// report out.
    pub fn assemble<E>(
        &mut self,
        admit: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), HtmlMediaActionError<E>> {
        self.require(self.value().is_ok())?;
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
            HtmlMediaPlacementObservation::Known(if self.theme_color()? {
                HtmlMetaMediaPlacement::AllowedForThemeColor
            } else {
                HtmlMetaMediaPlacement::Forbidden
            })
        };
        let role = self.role.ok_or(HtmlMediaActionError::Protocol(
            HtmlMediaInterpretationErrorKind::NotReady.error(),
        ))?;
        let value = self.value()?;
        self.execute(admit)?;
        self.applicability = Some(applicability);
        self.report = Some(HtmlMediaAttributeReport {
            raw: self.raw.map(capture_attribute),
            name: self.name_attribute().map(capture_attribute),
            applicability,
            role,
            value,
            placement,
        });
        Ok(())
    }
    /// Stops the owner. Every later step fails with `Stopped`; `abort` still
    /// returns the operands.
    pub fn stop(&mut self) {
        self.stopped = true;
    }
    /// The `media` operand this owner was created with.
    pub fn raw(&self) -> Option<&'raw A> {
        self.raw
    }
    /// Gives up the interpretation and returns its original operands, without
    /// scanning, sharing or allocating.
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
    /// Takes the assembled report out of the slot, leaving it empty. Fails with
    /// `Consumed` for an empty slot, `Stopped` for a stopped owner and
    /// `NotReady` before `assemble`. A failure leaves the owner in the slot.
    pub fn try_finish(
        slot: &mut Option<Self>,
    ) -> Result<HtmlMediaAttributeReport, HtmlMediaInterpretationError> {
        let owner = slot
            .as_ref()
            .ok_or(HtmlMediaInterpretationErrorKind::Consumed)?;
        if owner.stopped {
            return Err(HtmlMediaInterpretationErrorKind::Stopped.error());
        }
        if owner.report.is_none() {
            return Err(HtmlMediaInterpretationErrorKind::NotReady.error());
        }
        Ok(slot
            .take()
            .expect("checked owner")
            .report
            .expect("assembled report"))
    }
}

/// Original name operand supply, with absence distinct from non-selection.
#[derive(Debug)]
pub enum HtmlMediaNameSupply<'raw, A: HtmlAttributeInput> {
    /// No name operand was supplied: the caller did not ask for meta placement.
    NotSupplied,
    /// A name operand was supplied and the attribute is established absent.
    SuppliedAbsent,
    /// The supplied name attribute.
    Present(&'raw A),
}
/// Operands recovered without scanning, sharing or allocation.
#[derive(Debug)]
pub struct HtmlMediaInputs<'raw, N: HtmlNameInput + ?Sized, A: HtmlAttributeInput> {
    element: &'raw N,
    raw: Option<&'raw A>,
    name: HtmlMediaNameSupply<'raw, A>,
}
impl<'raw, N: HtmlNameInput + ?Sized, A: HtmlAttributeInput> HtmlMediaInputs<'raw, N, A> {
    /// The element name operand.
    pub fn element(&self) -> &'raw N {
        self.element
    }
    /// The `media` operand, or `None` when absent.
    pub fn raw(&self) -> Option<&'raw A> {
        self.raw
    }
    /// The `name` operand supply.
    pub fn name(&self) -> &HtmlMediaNameSupply<'raw, A> {
        &self.name
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    /// Every cause states its documented meaning, and exactly one of the
    /// cause predicates, its own, recognizes it.
    #[test]
    fn every_cause_has_its_message_and_exactly_one_matching_predicate() {
        type Predicate = fn(&HtmlMediaInterpretationError) -> bool;
        let cases: Vec<(HtmlMediaInterpretationError, &str, Predicate)> = vec![
            (
                HtmlMediaInterpretationErrorKind::Consumed.error(),
                "media interpretation was already consumed",
                HtmlMediaInterpretationError::is_consumed,
            ),
            (
                HtmlMediaInterpretationErrorKind::NotReady.error(),
                "media interpretation has an unfinished step",
                HtmlMediaInterpretationError::is_not_ready,
            ),
            (
                HtmlMediaInterpretationErrorKind::Stopped.error(),
                "media interpretation was stopped",
                HtmlMediaInterpretationError::is_stopped,
            ),
            (
                HtmlMediaInterpretationErrorKind::WrongPhase.error(),
                "media interpretation step was requested out of order",
                HtmlMediaInterpretationError::is_wrong_phase,
            ),
            (
                HtmlMediaInterpretationErrorKind::NameAlreadySupplied.error(),
                "media interpretation already has its name attribute",
                HtmlMediaInterpretationError::is_name_already_supplied,
            ),
            (
                HtmlMediaInterpretationErrorKind::NameNotSupplied.error(),
                "media interpretation has no name attribute yet",
                HtmlMediaInterpretationError::is_name_not_supplied,
            ),
            (
                HtmlMediaInterpretationErrorKind::WrongField.error(),
                "attribute is not the field this interpretation expects",
                HtmlMediaInterpretationError::is_wrong_field,
            ),
            (
                HtmlMediaInterpretationErrorKind::RepresentationExhausted.error(),
                "media interpretation size cannot be represented on this platform",
                HtmlMediaInterpretationError::is_representation_exhausted,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}
