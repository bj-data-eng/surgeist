//! The session's side of edits to the attribute lists of HTML's storage: the admission and
//! accounting of the growth of a list.
use super::storage_attributes::KeptCandidate;
use super::*;
use crate::parser::control::HtmlParseInvariant;
use surgeist_html::construction::HtmlConstructionCause;
use surgeist_html::{
    accounting::{HtmlChange, HtmlChargedError, HtmlResource, HtmlWork},
    bounded::construction::{
        HtmlAttributeAdmission, HtmlAttributeMerge, HtmlConstructionAttributeCapacity,
        HtmlConstructionAttributeError, HtmlConstructionAttributeGrowth,
    },
    construction::{ConstructionNodeKind, HtmlBuilderNode},
    values::{Attribute, DocumentString},
};
use surgeist_ledger::Charge;

/// What the controller admits and records for one edit of an attribute list.
struct Admission<'a> {
    control: &'a mut ParseControl,
    /// The bytes the parse holds for all attribute lists.
    bytes: &'a mut u64,
    /// The growth under way: the list before it, and the bytes of the layout it was admitted to.
    growing: Option<(HtmlConstructionAttributeCapacity, u64)>,
}

impl Charge<HtmlWork, HtmlResource> for Admission<'_> {
    type Stop = HtmlParseStop;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), HtmlParseStop> {
        Charge::charge(&mut *self.control, batch)
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), HtmlParseStop> {
        Charge::record_largest(&mut *self.control, resource, value)
    }
}

impl HtmlAttributeAdmission for Admission<'_> {
    fn admit_growth(
        &mut self,
        growth: HtmlConstructionAttributeGrowth,
    ) -> Result<usize, HtmlParseStop> {
        let before = growth.before();
        let target = self
            .control
            .attribute_growth_target(before, growth.minimum_capacity());
        let expected = self.control.prepare_attribute_growth(before, target)?;
        self.growing = Some((before, expected));
        Ok(target)
    }

    fn record_growth(
        &mut self,
        reached: HtmlConstructionAttributeCapacity,
    ) -> Result<(), HtmlParseStop> {
        let Some((before, expected)) = self.growing.take() else {
            return Err(self
                .control
                .fail_invariant(HtmlParseInvariant::CapacityMismatch));
        };
        let recorded = self
            .control
            .record_attribute_growth(before, reached, expected, self.bytes);
        // The capacity just recorded is the one the next record replaces, and an allocator that
        // refused the reservation reports the layout it was asked for.
        self.growing = Some((reached, expected));
        recorded
    }
}

impl<'a> Admission<'a> {
    fn new(control: &'a mut ParseControl, bytes: &'a mut u64) -> Self {
        Self {
            control,
            bytes,
            growing: None,
        }
    }

    /// The session error for an edit's refusal. An allocation the allocator refused reports the
    /// layout the growth was admitted to.
    fn error(self, error: HtmlConstructionAttributeError<HtmlParseStop>) -> SessionError {
        match error {
            HtmlChargedError::Stopped(stop) => SessionError::Stop(stop),
            HtmlChargedError::Faulted(error) => {
                if error.cause() == HtmlConstructionCause::AllocationFailed {
                    let layout = self
                        .growing
                        .map_or(0, |(_, bytes)| usize::try_from(bytes).unwrap_or(usize::MAX));
                    return SessionError::Stop(self.control.construction_allocation_failed(layout));
                }
                construction_error(self.control, error)
            }
        }
    }
}

/// Where the candidates of a merge come from and where its decisions go.
pub(in crate::construction) struct MergeCandidates<'a> {
    control: &'a mut ParseControl,
    names: &'a [crate::parser::tokenizer::token::Attribute],
    kept: &'a mut OwnedVec<KeptCandidate>,
}

impl<'a> MergeCandidates<'a> {
    pub(super) fn new(
        control: &'a mut ParseControl,
        names: &'a [crate::parser::tokenizer::token::Attribute],
        kept: &'a mut OwnedVec<KeptCandidate>,
    ) -> Self {
        Self {
            control,
            names,
            kept,
        }
    }
}

impl Charge<HtmlWork, HtmlResource> for MergeCandidates<'_> {
    type Stop = SessionError;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), SessionError> {
        Charge::charge(&mut *self.control, batch).map_err(SessionError::Stop)
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), SessionError> {
        Charge::record_largest(&mut *self.control, resource, value).map_err(SessionError::Stop)
    }
}

impl HtmlAttributeMerge for MergeCandidates<'_> {
    fn candidates(&self) -> usize {
        self.names.len()
    }

    fn name(&self, index: usize) -> Option<DocumentString> {
        self.names
            .get(index)
            .map(|candidate| candidate.name.value.text().clone())
    }

    fn keep(&mut self, index: usize) -> Result<(), SessionError> {
        let Some(name) = self.name(index) else {
            return Err(latch_registry_row(self.control));
        };
        self.kept
            .try_push_geometric_toward(
                self.control,
                KeptCandidate {
                    index,
                    name: Some(name),
                },
                self.names.len(),
            )
            .map_err(SessionError::Stop)
    }

    fn kept(&self, rank: usize) -> Option<usize> {
        self.kept.as_slice().get(rank).map(|kept| kept.index)
    }
}

impl SessionStorage {
    /// The handle of the element at `node` and the length of its attribute list, after the
    /// visit and provider call that qualify an edit.
    fn attribute_target(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<(HtmlBuilderNode, usize), SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let record = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        let len = match record.kind() {
            ConstructionNodeKind::Element(element) => element.attributes().len(),
            _ => return Err(SessionError::WrongToken),
        };
        Ok((record.handle(), len))
    }

    /// Ensures the attribute list of the element at `node` holds `additional` more attributes,
    /// growing it once when it does not.
    pub(super) fn reserve_attributes(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        additional: usize,
    ) -> Result<(), SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        let Self {
            construction: storage,
            holds,
            ..
        } = self;
        let mut admission = Admission::new(control, &mut holds.attribute_bytes);
        match storage.reserve_attributes(&handle, additional, &mut admission) {
            Ok(_) => Ok(()),
            Err(error) => Err(admission.error(error)),
        }
    }

    /// Appends the attribute in `incoming` to the element at `node`. `planned` names the
    /// candidate of a merge plan that the attribute is.
    pub(super) fn append_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        incoming: &mut Option<Attribute>,
        planned: Option<&DocumentString>,
    ) -> Result<usize, SessionError> {
        let (handle, len) = self.attribute_target(control, node)?;
        let Self {
            construction: storage,
            holds,
            ..
        } = self;
        let name_index = &mut holds.name_index;
        let names = if len.saturating_add(1) < NAME_INDEX_MIN_ENTRIES {
            None
        } else {
            name_index.as_mut()
        };
        let mut admission = Admission::new(control, &mut holds.attribute_bytes);
        let result = match planned {
            Some(candidate) => storage.append_planned_attribute(
                &handle,
                incoming,
                candidate,
                names,
                &mut admission,
            ),
            None => storage.append_attribute(&handle, incoming, names, &mut admission),
        };
        result.map_err(|error| admission.error(error))
    }

    /// Replaces the attribute at `index` of the element at `node` with the one in `incoming`
    /// and returns the one it replaced.
    pub(super) fn replace_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
        incoming: &mut Option<Attribute>,
    ) -> Result<Attribute, SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        let Self {
            construction: storage,
            holds,
            ..
        } = self;
        let mut admission = Admission::new(control, &mut holds.attribute_bytes);
        storage
            .replace_attribute(&handle, index, incoming, &mut admission)
            .map_err(|error| admission.error(error))
    }

    /// Removes the attribute at `index` of the element at `node` and returns it.
    pub(super) fn remove_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
    ) -> Result<Attribute, SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        let Self {
            construction: storage,
            holds,
            ..
        } = self;
        let mut admission = Admission::new(control, &mut holds.attribute_bytes);
        storage
            .remove_attribute(&handle, index, &mut admission)
            .map_err(|error| admission.error(error))
    }

    /// Decides which of the candidates of `names` the element at `node` keeps, pushing the
    /// index of each kept candidate to `kept`. Returns how many are kept.
    pub(super) fn plan_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        names: &[crate::parser::tokenizer::token::Attribute],
        kept: &mut OwnedVec<KeptCandidate>,
    ) -> Result<usize, SessionError> {
        let (handle, len) = self.attribute_target(control, node)?;
        let Self {
            construction: storage,
            holds,
            ..
        } = self;
        let name_index = &mut holds.name_index;
        let entries = len.saturating_add(names.len());
        let index = if entries < NAME_INDEX_MIN_ENTRIES {
            None
        } else {
            name_index.as_mut()
        };
        let mut merge = MergeCandidates::new(control, names, kept);
        match storage.plan_attribute_merge(&handle, index, &mut merge) {
            Ok(count) => Ok(count),
            Err(HtmlChargedError::Stopped(error)) => Err(error),
            Err(HtmlChargedError::Faulted(error)) => Err(construction_error(merge.control, error)),
        }
    }
}
