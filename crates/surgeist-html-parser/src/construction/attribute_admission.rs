//! The session's side of edits to the attributes of HTML's storage: the admission and
//! accounting of the banks an edit grows and of the logical slots and units it holds.
use super::record_admission::RecordAdmission;
use super::storage_attributes::KeptCandidate;
use super::*;
use surgeist_html::{
    accounting::{HtmlChange, HtmlChargedError, HtmlResource, HtmlWork},
    bounded::construction::{HtmlAttributeMerge, NameId},
    construction::{ConstructionNodeKind, HtmlBuilderNode},
    values::DocumentString,
};
use surgeist_ledger::Charge;

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

/// The UTF-16 units a name counts: its local name, prefix and namespace URI.
fn name_units_of(
    control: &mut ParseControl,
    name: &surgeist_html::values::QualifiedName,
) -> Result<usize, SessionError> {
    control.charge(HtmlParseWorkCategory::RecordVisit, 4)?;
    name.local_name()
        .len()
        .checked_add(name.namespace().uri().map_or(0, DocumentString::len))
        .and_then(|sum| sum.checked_add(name.prefix().map_or(0, DocumentString::len)))
        .ok_or_else(|| {
            SessionError::Stop(control.stop_representation(HtmlParseResource::ManagedUtf16Units))
        })
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
            ConstructionNodeKind::Element(element) => element.attributes().count(control)?,
            _ => return Err(SessionError::WrongToken),
        };
        Ok((record.handle(), len))
    }

    /// Ensures the attributes of the element at `node` have room for `additional` more, growing
    /// the banks once when they do not.
    pub(super) fn reserve_attributes(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        additional: usize,
    ) -> Result<(), SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        let mut admission = RecordAdmission::new(control, identity, holds, &[]);
        match construction.reserve_attributes(&handle, additional, &mut admission) {
            Ok(_) => Ok(()),
            Err(error) => Err(admission.error(error)),
        }
    }

    /// Appends the attribute of the interned name `name` and the value `value` to the element at
    /// `node`. `planned` names the candidate of a merge plan that the attribute is. The logical
    /// slot and units of the attribute are acquired with the append, but for `counted` units of
    /// its name that an earlier registration of them counted. Returns the position the attribute
    /// took.
    pub(super) fn append_named_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        name: &NameId,
        value: &[u16],
        planned: Option<&DocumentString>,
        counted: usize,
    ) -> Result<usize, SessionError> {
        let (handle, len) = self.attribute_target(control, node)?;
        let units = self
            .attribute_units_of(control, name, value.len())?
            .checked_sub(counted as u64)
            .ok_or_else(|| latch_registry_row(control))?;
        let requests = [
            (HtmlParseResource::ManagedAttributes, 1),
            (HtmlParseResource::ManagedUtf16Units, units),
        ];
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        // The table of names serves an edit of a long list. The edit borrows it from the holds,
        // which the admission also borrows, so it is taken out for the edit and put back.
        let mut names = if len.saturating_add(1) < NAME_INDEX_MIN_ENTRIES {
            None
        } else {
            holds.name_index.take()
        };
        let mut admission = RecordAdmission::new(control, identity, holds, &requests);
        admission.leaf = true;
        admission.attributes = true;
        admission.counted = counted as u64;
        let result = construction.append_named_attribute(
            &handle,
            name,
            value,
            planned,
            names.as_mut(),
            &mut admission,
        );
        let outcome = result.map_err(|error| admission.error(error));
        if names.is_some() {
            self.holds.name_index = names;
        }
        outcome
    }

    /// The units the attribute of the interned name `name` and a value of `value` units counts.
    fn attribute_units_of(
        &self,
        control: &mut ParseControl,
        name: &NameId,
        value: usize,
    ) -> Result<u64, SessionError> {
        let name = self
            .construction
            .name(name)
            .map_err(|error| construction_error(control, error))?;
        let units = name_units_of(control, name)?
            .checked_add(value)
            .ok_or_else(|| {
                SessionError::Stop(
                    control.stop_representation(HtmlParseResource::ManagedUtf16Units),
                )
            })?;
        Ok(units as u64)
    }

    /// Replaces the attribute at `index` of the element at `node` with one of the interned name
    /// `name` and the value `value`. The units the old attribute held are credited, and the new
    /// ones acquired.
    ///
    /// The old attribute must have no namespace and no prefix: the units of a literal namespace
    /// or prefix are counted by the registry row of the literal when the attribute is its first
    /// use, and by the attribute otherwise, so the credit of the attribute cannot tell which
    /// units it holds. An attribute that has one is refused with the registry-row defect before
    /// anything is credited or changed.
    pub(super) fn replace_named_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
        name: &NameId,
        value: &[u16],
    ) -> Result<(), SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        self.require_unqualified_attribute(control, node, index)?;
        let old = self.attribute_units_at(control, node, index)?;
        let new = self.attribute_units_of(control, name, value.len())?;
        let added = new.saturating_sub(old);
        let requests = [(HtmlParseResource::ManagedUtf16Units, added)];
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        let mut admission = RecordAdmission::new(
            control,
            identity,
            holds,
            if added == 0 { &[] } else { &requests },
        );
        admission.leaf = true;
        let replaced =
            construction.replace_named_attribute(&handle, index, name, value, &mut admission);
        replaced.map_err(|error| admission.error(error))?;
        // A shorter attribute gives the units it no longer holds back.
        self.holds
            .release_string_units(control, old.saturating_sub(new))
    }

    /// Refuses an attribute that has a namespace or a prefix with the registry-row defect.
    fn require_unqualified_attribute(
        &self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
    ) -> Result<(), SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(ConstructionNodeKind::Element(element)) =
            self.construction.node_at(node).map(|record| record.kind())
        else {
            return Err(SessionError::WrongToken);
        };
        let attribute = element
            .attributes()
            .at(control, index)?
            .ok_or_else(|| latch_registry_row(control))?;
        let name = attribute.name();
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if name.namespace().uri().is_some() || name.prefix().is_some() {
            return Err(latch_registry_row(control));
        }
        Ok(())
    }

    /// The units the attribute at `index` of the element at `node` counts.
    fn attribute_units_at(
        &self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
    ) -> Result<u64, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(ConstructionNodeKind::Element(element)) =
            self.construction.node_at(node).map(|record| record.kind())
        else {
            return Err(SessionError::WrongToken);
        };
        let attribute = element
            .attributes()
            .at(control, index)?
            .ok_or_else(|| latch_registry_row(control))?;
        let value = attribute.value().units(control)?.len();
        Ok(name_units_of(control, attribute.name())?
            .checked_add(value)
            .ok_or_else(|| {
                SessionError::Stop(
                    control.stop_representation(HtmlParseResource::ManagedUtf16Units),
                )
            })? as u64)
    }

    /// Removes the attribute at `index` of the element at `node`, crediting its logical slot and
    /// units.
    ///
    /// The attribute must have no namespace and no prefix, as for
    /// [`Self::replace_named_attribute`]: one that has either is refused with the registry-row
    /// defect before anything is credited or changed.
    pub(super) fn remove_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
    ) -> Result<(), SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        self.require_unqualified_attribute(control, node, index)?;
        let units = self.attribute_units_at(control, node, index)?;
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        let mut admission = RecordAdmission::new(control, identity, holds, &[]);
        construction
            .remove_attribute(&handle, index, &mut admission)
            .map_err(|error| admission.error(error))?;
        self.holds.release_attribute(control)?;
        self.holds.release_string_units(control, units)
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
            construction,
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
        match construction.plan_attribute_merge(&handle, index, &mut merge) {
            Ok(count) => Ok(count),
            Err(HtmlChargedError::Stopped(error)) => Err(error),
            Err(HtmlChargedError::Faulted(error)) => Err(construction_error(merge.control, error)),
        }
    }
}
