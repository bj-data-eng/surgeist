//! Static Checkbox/Radio missingness, HTML 877d6146487f §4.10.5.1.15–16.
//! Current checkedness and form pointers come from the existing qualified facts.
//! This predicate neither normalizes radio state nor executes validation.
//! Source conditions: [Checkbox](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#checkbox-state-(type=checkbox)) and
//! [Radio](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#radio-button-state-(type=radio)).

use super::*;
use crate::HtmlInputType;

enum RadioChecked {
    Known(bool),
    Missing(HtmlSemanticMissing),
}

enum RadioMembership {
    Confirmed,
    Possible(HtmlSemanticMissing),
}

// Bit i represents (any_required = i & 2 != 0, any_checked = i & 1 != 0).
// Possible inclusion preserves correlation within each actual candidate.
struct RadioMissingSummary {
    possibilities: u8,
    required_missing: Option<HtmlSemanticMissing>,
    checked_missing: Option<HtmlSemanticMissing>,
}
impl RadioMissingSummary {
    fn new() -> Self {
        Self {
            possibilities: 1,
            required_missing: None,
            checked_missing: None,
        }
    }
}

impl<'a> HtmlSemanticSession<'a> {
    pub(super) fn checkable_input_value_missing(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Input) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let input_type = self
            .control_input_type_report(element)?
            .intrinsic()
            .copied();
        if !matches!(
            input_type,
            Some(HtmlInputType::Checkbox | HtmlInputType::Radio)
        ) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let required = self.attribute(element, "required")?.is_some();
        if input_type == Some(HtmlInputType::Checkbox) {
            return if required {
                Ok(self
                    .control_flag(subject, HtmlControlFlag::Checkedness)?
                    .map(|v| !v))
            } else {
                Ok(HtmlSemanticValue::Known(false))
            };
        }
        let checked = self.radio_checked(subject)?;
        // Self belongs even without a name, owner or complete ancestry.
        if matches!(checked, RadioChecked::Known(true)) {
            return Ok(HtmlSemanticValue::Known(false));
        }
        self.charge(1)?;
        let mut summary = RadioMissingSummary::new();
        self.radio_summary_add(&mut summary, required, checked, RadioMembership::Confirmed)?;
        let name = self.attribute(element, "name")?;
        let Some(name) = name else {
            return self.radio_summary_result(summary);
        };
        self.radio_admit_name(name)?;
        self.charge(1)?;
        if name.value().is_empty() {
            return self.radio_summary_result(summary);
        }
        let root = match self.ordinary_tree_root(subject)? {
            Ok(root) => root,
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let mut frames = Vec::new();
        let result = self.radio_group_missing(subject, name, root, &mut summary, &mut frames);
        self.release_scratch(frames.capacity());
        result
    }

    fn radio_checked(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<RadioChecked, HtmlSemanticFailure> {
        match self.control_flag(subject, HtmlControlFlag::Checkedness)? {
            HtmlSemanticValue::Known(value) => Ok(RadioChecked::Known(value)),
            HtmlSemanticValue::Missing(missing) => Ok(RadioChecked::Missing(missing)),
            _ => Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
        }
    }

    fn radio_admit_name(&mut self, name: &Attribute) -> Result<(), HtmlSemanticFailure> {
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(Some(name.value()))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))
    }

    fn radio_group_missing(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        name: &'a Attribute,
        root: HtmlCaptureSubjectId,
        summary: &mut RadioMissingSummary,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let mut current = Some(root);
        // Resolve the query's pointer only when a nonneutral same-name candidate
        // actually makes membership relevant. None here means not consulted.
        let mut owner = None;
        while let Some(id) = current {
            let node = self.node(&id)?;
            self.charge(1)?;
            if &id != subject
                && let HtmlCapturedNodeKind::Element(element) = node.kind()
                && self.known(element)? == Some(KnownHtmlElement::Input)
                && self.control_input_type_report(element)?.intrinsic()
                    == Some(&HtmlInputType::Radio)
                && let Some(candidate_name) = self.attribute(element, "name")?
            {
                self.radio_admit_name(candidate_name)?;
                if self.control_equal_units(
                    name.value().code_units(),
                    candidate_name.value().code_units(),
                )? {
                    let required = self.attribute(element, "required")?.is_some();
                    let checked = self.radio_checked(&id)?;
                    // A nonrequired, definitely unchecked candidate contributes
                    // nothing whether included or excluded: no pointer is needed.
                    if required || !matches!(checked, RadioChecked::Known(false)) {
                        if owner.is_none() {
                            owner = Some(self.form_owner(subject)?);
                        }
                        let candidate_owner = self.form_owner(&id)?;
                        let membership = match (owner.as_ref(), candidate_owner) {
                            (
                                Some(HtmlSemanticValue::Known(left)),
                                HtmlSemanticValue::Known(right),
                            ) => {
                                self.charge(1)?;
                                if left.owner() == right.owner() {
                                    Some(RadioMembership::Confirmed)
                                } else {
                                    None
                                }
                            }
                            (_, HtmlSemanticValue::Missing(missing)) => {
                                self.charge(1)?;
                                Some(RadioMembership::Possible(missing))
                            }
                            (Some(HtmlSemanticValue::Missing(missing)), _) => {
                                self.charge(1)?;
                                Some(RadioMembership::Possible(missing.clone()))
                            }
                            _ => {
                                return Err(
                                    self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                                );
                            }
                        };
                        if let Some(membership) = membership {
                            self.radio_summary_add(summary, required, checked, membership)?;
                            self.charge(1)?;
                            if summary.possibilities & 0b0101 == 0 {
                                return Ok(HtmlSemanticValue::Known(false));
                            }
                        }
                    }
                }
            }
            current = match self.ordinary_tree_next(node, frames)? {
                Ok(next) => next,
                Err(missing) => {
                    self.radio_summary_gap(summary, missing)?;
                    self.ordinary_tree_resume(frames)?
                }
            };
        }
        self.charge(1)?;
        self.radio_summary_result(RadioMissingSummary {
            possibilities: summary.possibilities,
            required_missing: summary.required_missing.take(),
            checked_missing: summary.checked_missing.take(),
        })
    }

    fn radio_summary_add(
        &mut self,
        summary: &mut RadioMissingSummary,
        required: bool,
        checked: RadioChecked,
        membership: RadioMembership,
    ) -> Result<(), HtmlSemanticFailure> {
        self.charge(1)?;
        let base = if required { 2 } else { 0 };
        let may_be_checked = !matches!(checked, RadioChecked::Known(false));
        let mut contributions = match checked {
            RadioChecked::Known(value) => 1 << (base | u8::from(value)),
            RadioChecked::Missing(missing) => {
                self.charge(1)?;
                summary.checked_missing.get_or_insert(missing);
                (1 << base) | (1 << (base | 1))
            }
        };
        if let RadioMembership::Possible(missing) = membership {
            self.charge(1)?;
            contributions |= 1;
            if required {
                self.charge(1)?;
                summary.required_missing.get_or_insert(missing.clone());
            }
            if may_be_checked {
                self.charge(1)?;
                summary.checked_missing.get_or_insert(missing);
            }
        }
        self.radio_summary_merge(summary, contributions)
    }

    fn radio_summary_gap(
        &mut self,
        summary: &mut RadioMissingSummary,
        missing: HtmlSemanticMissing,
    ) -> Result<(), HtmlSemanticFailure> {
        self.charge(1)?;
        summary.required_missing.get_or_insert(missing.clone());
        self.charge(1)?;
        summary.checked_missing.get_or_insert(missing);
        self.radio_summary_merge(summary, 0b1111)
    }

    fn radio_summary_merge(
        &mut self,
        summary: &mut RadioMissingSummary,
        contributions: u8,
    ) -> Result<(), HtmlSemanticFailure> {
        let mut possibilities = 0;
        for before in 0..4 {
            self.charge(1)?;
            if summary.possibilities & (1 << before) != 0 {
                for contribution in 0..4 {
                    self.charge(1)?;
                    if contributions & (1 << contribution) != 0 {
                        self.charge(1)?;
                        possibilities |= 1 << (before | contribution);
                    }
                }
            }
        }
        self.charge(1)?;
        summary.possibilities = possibilities;
        Ok(())
    }

    fn radio_summary_result(
        &mut self,
        summary: RadioMissingSummary,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        self.charge(1)?;
        // Only state (required, unchecked) suffers from being missing.
        if summary.possibilities == 0b0100 {
            return Ok(HtmlSemanticValue::Known(true));
        }
        if summary.possibilities & 0b0100 == 0 {
            return Ok(HtmlSemanticValue::Known(false));
        }
        let missing = if summary.possibilities & 0b0011 == 0 {
            summary.checked_missing.or(summary.required_missing)
        } else {
            summary.required_missing.or(summary.checked_missing)
        };
        match missing {
            Some(missing) => Ok(HtmlSemanticValue::Missing(missing)),
            None => Err(self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture() -> HtmlCapture {
        crate::HtmlCaptureBuilder::new(
            crate::HtmlCaptureLimits::new(0, 0, 0, 0, 0, 0, 0),
            HtmlHostFactsRevision::fresh(),
        )
        .finish()
        .unwrap()
    }

    #[test]
    fn full_four_state_merge_admits_thirty_seven_actual_visits_and_writes() {
        let c = capture();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            0,
        )
        .unwrap();
        let mut ss = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(0),
            HtmlSemanticLimits::new(37, 0, 0, 0, 0, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let mut summary = RadioMissingSummary {
            possibilities: 0b1111,
            required_missing: None,
            checked_missing: None,
        };
        ss.radio_summary_merge(&mut summary, 0b1111).unwrap();
        assert_eq!(summary.possibilities, 0b1111);
        assert_eq!(ss.usage().work(), 37);
        let mut ss = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(0),
            HtmlSemanticLimits::new(36, 0, 0, 0, 0, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let f = ss.radio_summary_merge(&mut summary, 0b1111).unwrap_err();
        assert_eq!(
            f.kind(),
            &HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: 36,
                required: 37
            }
        );
        assert_eq!(ss.query(&[]).unwrap_err(), f);
    }

    #[test]
    fn inclusion_correlates_required_and_checked_without_a_spurious_missing_state() {
        let c = capture();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            0,
        )
        .unwrap();
        let mut ss = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(0),
            HtmlSemanticLimits::new(100, 0, 0, 0, 0, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let mut summary = RadioMissingSummary::new();
        // One candidate contributes either nothing or (required AND checked).
        ss.radio_summary_merge(&mut summary, 0b1001).unwrap();
        assert_eq!(summary.possibilities, 0b1001);
        assert_eq!(
            ss.radio_summary_result(summary).unwrap(),
            HtmlSemanticValue::Known(false)
        );
    }

    #[test]
    fn exact_name_owner_charges_length_then_only_inspected_utf16_pairs() {
        let c = capture();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]),
            0,
        )
        .unwrap();
        for (left, right, expected, cost) in [
            (&[][..], &[][..], true, 1),
            (&[97, 98][..], &[97][..], false, 1),
            (&[97, 98][..], &[97, 98][..], true, 5),
            (&[97, 98][..], &[97, 120][..], false, 5),
            (&[97, 98][..], &[120, 98][..], false, 3),
            (&[0xd800, 0][..], &[0xd800, 0][..], true, 5),
        ] {
            let mut ss = HtmlSemanticSession::new(
                &c,
                &cx,
                HtmlValueInterpreter::new(0),
                HtmlSemanticLimits::new(cost, 0, 0, 0, 0, 0),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
            assert_eq!(ss.control_equal_units(left, right).unwrap(), expected);
            assert_eq!(ss.usage().work(), cost);
        }
    }
}
