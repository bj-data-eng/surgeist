//! Capture-qualified autocomplete observations and the source owner default.
//! The intrinsic owner supplies syntax and literal token processing. This child
//! resolves only expectation defaults; it performs no autofill or credential effect.

use super::*;
use surgeist_html::attributes::{
    HtmlAttributeReport, HtmlFormAutocomplete, HtmlInputType,
    autocomplete::{
        HtmlAutocompleteCredentialOutcome, HtmlAutocompleteDefaultField,
        HtmlAutocompleteProcessingState, HtmlAutocompleteReport,
    },
};
use surgeist_html::bounded::attributes::{
    HtmlAutocompleteMaterializationError, autocomplete_materialization_work_bound,
};
use surgeist_html::{values::Attribute, vocabulary::KnownHtmlElement};

/// A default field observation, independent of the empty default IDL value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlAutocompleteDefaultResolution {
    /// Processing or an explicit input-state exclusion needs no owner default.
    NotRequired,
    /// The anchor default sets the field to empty before consulting association.
    AnchorEmpty,
    Resolved {
        setting: HtmlFormAutocomplete,
        owner: HtmlFormOwner,
    },
    /// No effective default field is established from unavailable owner evidence.
    Missing(HtmlSemanticMissing),
}

/// Admitted intrinsic observations plus the actually consulted default evidence.
/// A `Missing` resolution retains raw/syntax without claiming an effective field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlControlAutocomplete {
    // Exactly one admitted record. Vec supplies stable fallible reservation;
    // no infallible Box allocation or duplicate projection is needed.
    data: Vec<HtmlControlAutocompleteData>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct HtmlControlAutocompleteData {
    intrinsic: HtmlAutocompleteReport,
    input_type: Option<HtmlAttributeReport<HtmlInputType>>,
    resolution: HtmlAutocompleteDefaultResolution,
    form_autocomplete: Option<HtmlAttributeReport<HtmlFormAutocomplete>>,
}
impl HtmlControlAutocomplete {
    pub fn intrinsic(&self) -> &HtmlAutocompleteReport {
        &self.data[0].intrinsic
    }
    pub fn input_type(&self) -> Option<&HtmlAttributeReport<HtmlInputType>> {
        self.data[0].input_type.as_ref()
    }
    pub fn default_resolution(&self) -> &HtmlAutocompleteDefaultResolution {
        &self.data[0].resolution
    }
    /// Present only when a non-null form owner was consulted by Default.
    pub fn form_autocomplete(&self) -> Option<&HtmlAttributeReport<HtmlFormAutocomplete>> {
        self.data[0].form_autocomplete.as_ref()
    }
    /// Default and early setting branches do not assign this property. `None`
    /// means processing is outside the supplied control's source domain.
    pub fn credential_type(&self) -> Option<HtmlAutocompleteCredentialOutcome> {
        match self.data[0].intrinsic.processing() {
            HtmlAutocompleteProcessingState::Processed(value) => Some(value.credential_type()),
            HtmlAutocompleteProcessingState::Defaulted(_) => {
                Some(HtmlAutocompleteCredentialOutcome::NotAssigned)
            }
            _ => None,
        }
    }
}

impl HtmlSemanticSession<'_> {
    pub(super) fn control_autocomplete(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlControlAutocomplete>, HtmlSemanticFailure> {
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
            Some(KnownHtmlElement::Input | KnownHtmlElement::Select | KnownHtmlElement::Textarea)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let raw = self.attribute(element, "autocomplete")?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        let input_type = if kind == Some(KnownHtmlElement::Input) {
            let raw_type = self.attribute(element, "type")?;
            Some(self.admitted_input_type_report(element, raw_type)?)
        } else {
            None
        };
        let state = input_type
            .as_ref()
            .and_then(|report| report.intrinsic())
            .copied();
        if input_type.is_some() && state.is_none() {
            return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable));
        }
        self.admit_intrinsic(element, raw, 0)?;
        let work = autocomplete_materialization_work_bound(raw.map_or(0, |raw| raw.value().len()))
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(work)?;
        // The materialization charges this session's scopes: its growth is observed before the
        // session's caps admit or refuse it, and its scratch is returned by what it holds.
        let (ledger, caps, observed) = self.account.scopes();
        let intrinsic = self
            .interpreter
            .control_autocomplete_in_scopes(element.name(), raw, state, ledger, caps, observed)
            .map_err(|failure| {
                let kind = match failure.kind() {
                    HtmlAutocompleteMaterializationError::RawAdmission(error) => {
                        HtmlSemanticFailureKind::Attribute(error)
                    }
                    HtmlAutocompleteMaterializationError::BytesLimit { limit, required } => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::AllocatedBytes,
                            limit,
                            required,
                        }
                    }
                    HtmlAutocompleteMaterializationError::OutputLimit { limit, required } => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::NestedOutputRecords,
                            limit,
                            required,
                        }
                    }
                    HtmlAutocompleteMaterializationError::ScratchLimit { limit, required } => {
                        HtmlSemanticFailureKind::LimitExceeded {
                            resource: HtmlSemanticResource::ScratchRecords,
                            limit,
                            required,
                        }
                    }
                    HtmlAutocompleteMaterializationError::Allocation { .. } => {
                        HtmlSemanticFailureKind::AllocationFailed
                    }
                    HtmlAutocompleteMaterializationError::Representation => {
                        HtmlSemanticFailureKind::RepresentationExhausted
                    }
                    HtmlAutocompleteMaterializationError::LedgerState => {
                        HtmlSemanticFailureKind::ProtocolViolated
                    }
                };
                self.stop(kind)
            })?
            .into_parts()
            .0;
        // The controlled owner checked every variable reservation. Observe its
        // capacities before polling, including discarded intermediates on error.
        self.poll()?;
        let field = match intrinsic.processing() {
            HtmlAutocompleteProcessingState::Defaulted(value) => Some(value.field()),
            _ => None,
        };
        let mut report = HtmlControlAutocompleteData {
            intrinsic,
            input_type,
            resolution: HtmlAutocompleteDefaultResolution::NotRequired,
            form_autocomplete: None,
        };
        match field {
            Some(HtmlAutocompleteDefaultField::Empty) => {
                report.resolution = HtmlAutocompleteDefaultResolution::AnchorEmpty;
            }
            Some(HtmlAutocompleteDefaultField::FormOwnerRequired) => {
                match self.form_owner(subject)? {
                    HtmlSemanticValue::Known(owner) => {
                        let setting = if let Some(form) = owner.owner() {
                            let node = self.node(form)?;
                            let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                                return Err(
                                    self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                                );
                            };
                            let raw = self.attribute(element, "autocomplete")?;
                            self.admit_intrinsic(element, raw, 2)?;
                            let value = self
                                .interpreter
                                .form_autocomplete(element.name(), raw)
                                .map_err(|error| {
                                    self.stop(HtmlSemanticFailureKind::Attribute(error))
                                })?;
                            let setting = value.intrinsic().copied().ok_or_else(|| {
                                self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                            })?;
                            report.form_autocomplete = Some(value);
                            setting
                        } else {
                            HtmlFormAutocomplete::On
                        };
                        report.resolution =
                            HtmlAutocompleteDefaultResolution::Resolved { setting, owner };
                    }
                    HtmlSemanticValue::Missing(missing) => {
                        report.resolution = HtmlAutocompleteDefaultResolution::Missing(missing);
                    }
                    _ => return Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
                }
            }
            None => {}
        }
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(report);
        Ok(HtmlSemanticValue::Known(HtmlControlAutocomplete { data }))
    }
}
