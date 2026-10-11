//! Capture-qualified autocomplete observations and the source owner default.
//! The intrinsic owner supplies syntax and literal token processing. This child
//! resolves only expectation defaults; it performs no autofill or credential effect.

use super::*;
use crate::attributes::{
    AutocompleteMaterializationError, AutocompleteMaterializationLimits,
    AutocompleteMaterializationUsage, autocomplete_materialization_work_bound,
};
use crate::{
    HtmlAttributeReport, HtmlAutocompleteCredentialOutcome, HtmlAutocompleteDefaultField,
    HtmlAutocompleteProcessingState, HtmlAutocompleteReport, HtmlFormAutocomplete, HtmlInputType,
};

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
            // The selected input-type rule has 22 authored keyword candidates.
            self.admit_intrinsic(element, raw_type, 22)?;
            Some(
                self.interpreter
                    .input_type(element.name(), raw_type)
                    .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?,
            )
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
        let baseline = self.usage;
        let materialized = self.interpreter.control_autocomplete_controlled(
            element.name(),
            raw,
            state,
            AutocompleteMaterializationLimits {
                allocated_bytes: self.remaining_allocated_bytes(),
                nested_output_records: self
                    .limits
                    .nested_output_records
                    .saturating_sub(baseline.nested_output_records),
                scratch_records: self
                    .limits
                    .scratch_records
                    .saturating_sub(baseline.scratch_records),
            },
        );
        let observed = match &materialized {
            Ok(value) => value.usage,
            Err(failure) => failure.usage,
        };
        self.record_autocomplete_usage(observed)?;
        let intrinsic = materialized
            .map_err(|failure| {
                let kind = match failure.kind {
                    AutocompleteMaterializationError::RawAdmission(error) => {
                        HtmlSemanticFailureKind::Attribute(error)
                    }
                    AutocompleteMaterializationError::BytesLimit { required, .. } => self
                        .autocomplete_limit(
                            HtmlSemanticResource::AllocatedBytes,
                            baseline.allocated_bytes,
                            required,
                        ),
                    AutocompleteMaterializationError::OutputLimit { required, .. } => self
                        .autocomplete_limit(
                            HtmlSemanticResource::NestedOutputRecords,
                            baseline.nested_output_records,
                            required,
                        ),
                    AutocompleteMaterializationError::ScratchLimit { required, .. } => self
                        .autocomplete_limit(
                            HtmlSemanticResource::ScratchRecords,
                            baseline.scratch_records,
                            required,
                        ),
                    AutocompleteMaterializationError::Allocation { .. } => {
                        HtmlSemanticFailureKind::AllocationFailed
                    }
                    AutocompleteMaterializationError::Representation => {
                        HtmlSemanticFailureKind::RepresentationExhausted
                    }
                };
                self.stop(kind)
            })?
            .report;
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

    fn autocomplete_limit(
        &self,
        resource: HtmlSemanticResource,
        baseline: usize,
        required: usize,
    ) -> HtmlSemanticFailureKind {
        let Some(required) = baseline.checked_add(required) else {
            return HtmlSemanticFailureKind::RepresentationExhausted;
        };
        let limit = match resource {
            HtmlSemanticResource::AllocatedBytes => self.limits.allocated_bytes,
            HtmlSemanticResource::NestedOutputRecords => self.limits.nested_output_records,
            HtmlSemanticResource::ScratchRecords => self.limits.scratch_records,
            _ => return HtmlSemanticFailureKind::RepresentationExhausted,
        };
        HtmlSemanticFailureKind::LimitExceeded {
            resource,
            limit,
            required,
        }
    }

    fn record_autocomplete_usage(
        &mut self,
        observed: AutocompleteMaterializationUsage,
    ) -> Result<(), HtmlSemanticFailure> {
        let bytes = self
            .usage
            .allocated_bytes
            .checked_add(observed.allocated_bytes);
        let output = self
            .usage
            .nested_output_records
            .checked_add(observed.nested_output_records);
        let scratch = self
            .usage
            .scratch_records
            .checked_add(observed.peak_scratch_records);
        let (Some(bytes), Some(output), Some(scratch)) = (bytes, output, scratch) else {
            return Err(self.stop(HtmlSemanticFailureKind::RepresentationExhausted));
        };
        self.usage.allocated_bytes = bytes;
        self.usage.nested_output_records = output;
        self.usage.peak_scratch_records = self.usage.peak_scratch_records.max(scratch);
        // Temporary materialization vectors have been dropped before the owner
        // returns; current scratch remains unchanged on success and failure.
        Ok(())
    }
}
