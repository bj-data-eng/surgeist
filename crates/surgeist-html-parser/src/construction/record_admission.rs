//! The session's side of the record operations of HTML's construction storage: what the
//! controller admits and records for each operation, and the calls that issue them.
use super::template::template_slot_of;
use super::*;
use surgeist_html::construction::HtmlConstructionCause;
#[cfg(test)]
use surgeist_html::values::QualifiedName;
use surgeist_html::{
    accounting::{HtmlChange, HtmlChargedError, HtmlResource, HtmlWork},
    bounded::construction::{
        HtmlConstructionBank, HtmlConstructionBankGrowth, HtmlConstructionChange,
        HtmlConstructionGrowthRecord, HtmlConstructionRecordError, HtmlConstructionStorage,
        HtmlRecordAdmission, NameId,
    },
    construction::{HtmlConstructionResult, HtmlDocumentMode, HtmlShadowRootProperties},
    values::{Namespace, OrderedAttributes},
};
use surgeist_ledger::Charge;

/// What one parser allocation creates. A leaf carries the units it copies, an element its name
/// and its validated attributes, and a root only its kind.
pub(super) enum NewRecord<'a> {
    Document(HtmlDocumentMode),
    Fragment,
    Text(&'a [u16]),
    Comment(&'a [u16]),
    ProcessingInstruction(&'a [u16], &'a [u16]),
    Doctype(&'a [u16], &'a [u16], &'a [u16]),
    /// An element of a name that its caller holds, which the storage interns by content. Only an
    /// element that exists before the parse has no string rows to share its name.
    #[cfg(test)]
    Element(
        QualifiedName,
        OrderedAttributes,
        Option<surgeist_html::elements::link::HtmlLinkFlags>,
    ),
    /// An element of a name the storage has interned.
    NamedElement(
        NameId,
        OrderedAttributes,
        Option<surgeist_html::elements::link::HtmlLinkFlags>,
    ),
    ShadowContents(HtmlShadowRootProperties),
}

impl NewRecord<'_> {
    /// Whether the record is a leaf, whose strings the parse holds.
    fn is_leaf(&self) -> bool {
        matches!(
            self,
            Self::Text(_) | Self::Comment(_) | Self::ProcessingInstruction(..) | Self::Doctype(..)
        )
    }
}

/// The view of a session that an insertion consults while it is admitted: the registry rows and
/// context that decide whether the insertion leaves the closed ordinary producer inventory, and
/// the effect payloads it will write. The storage itself is lent by the insertion.
pub(super) struct Insertion<'a> {
    pub(super) parent: usize,
    pub(super) payloads: &'a mut PayloadStorage,
    pub(super) fragment: &'a Option<FragmentContextStorage>,
    pub(super) ineligibility: &'a Option<HtmlParseUnsupportedState>,
    /// Whether the parse writes its form effects with the insertion.
    pub(super) form_effects: bool,
    /// Whether the insertion revokes the eligibility of the standalone document.
    pub(super) revoke: bool,
}

/// What the controller admits and records for one record operation.
pub(super) struct RecordAdmission<'a> {
    control: &'a mut ParseControl,
    identity: &'a ControlIdentity,
    holds: &'a mut StorageHolds,
    /// The logical resources the operation acquires, which the controller checks and acquires.
    requests: &'a [(HtmlParseResource, u64)],
    /// The bytes of the layout the growth under way was admitted to.
    layout: u64,
    /// Whether the units the operation adds are the strings of a leaf, which the parse holds.
    leaf: bool,
    insertion: Option<Insertion<'a>>,
    /// The bytes the parse holds for the contents vector of a publication, which that
    /// publication frees when its records move.
    contents: u64,
}

impl Charge<HtmlWork, HtmlResource> for RecordAdmission<'_> {
    type Stop = SessionError;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), SessionError> {
        Charge::charge(&mut *self.control, batch).map_err(SessionError::Stop)
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), SessionError> {
        Charge::record_largest(&mut *self.control, resource, value).map_err(SessionError::Stop)
    }
}

impl HtmlRecordAdmission for RecordAdmission<'_> {
    fn admit_change(
        &mut self,
        storage: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), SessionError> {
        let _ = change;
        let Some(insertion) = self.insertion.as_mut() else {
            return Ok(());
        };
        // Source semantics stay between the depth and the edge checks, before any scratch bank.
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        insertion.revoke = insertion.ineligibility.is_none()
            && associated_document_parent_of(
                insertion.payloads,
                insertion.fragment,
                self.control,
                storage,
                insertion.parent,
            )?;
        self.control.check_acquire_many(self.requests)?;
        Ok(())
    }

    fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, SessionError> {
        let target = self.control.bank_growth_target(growth)?;
        self.control.prepare_bank_growth(growth, target)?;
        Ok(target)
    }

    fn record_growth(&mut self, record: HtmlConstructionGrowthRecord) -> Result<(), SessionError> {
        let growth = record.growth();
        let first = record.recorded() == growth.before();
        let old = ParseControl::bank_record_bytes(growth, record.recorded().capacity(), first)?;
        // A bank that came back to the dimensions it had before the growth holds no allocation of
        // it, and so no wrapper either. A first record that chooses none is not a return: an
        // empty string owns its vector.
        let returned = !first && record.reached() == growth.before();
        let actual =
            ParseControl::bank_record_bytes(growth, record.reached().capacity(), returned)?;
        let expected = ParseControl::bank_record_bytes(growth, record.chosen(), false)?;
        // A record that takes a bank back admits no layout: the layout of the last admitted growth
        // is the one an allocation refusal reports.
        let takes_back = returned && record.chosen() == growth.before().capacity();
        if !takes_back {
            self.layout = expected;
        }
        let held = match growth.bank() {
            HtmlConstructionBank::Text
            | HtmlConstructionBank::TextArena
            | HtmlConstructionBank::TextBlock => &mut self.holds.string_bytes,
            _ => &mut self.holds.bank_bytes,
        };
        self.control
            .record_bank_growth(old, actual, expected, held)?;
        if growth.bank() == HtmlConstructionBank::PublishedContents {
            self.contents = actual;
        }
        Ok(())
    }

    fn record_change(
        &mut self,
        _: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), SessionError> {
        if let Some(insertion) = &self.insertion {
            // These fixed source payloads precede the atomic commit, after the child-order
            // preparation.
            if insertion.revoke {
                payload_write::<Option<HtmlParseUnsupportedState>>(self.control, 1)?;
            }
            if insertion.form_effects {
                payload_write::<Option<FormEffects>>(self.control, 1)?;
            }
        }
        let released = change
            .before()
            .edges()
            .saturating_sub(change.after().edges()) as u64;
        // The sum of the units the parse holds is checked before the controller acquires them.
        let held = if self.leaf {
            let added = change
                .after()
                .utf16_units()
                .saturating_sub(change.before().utf16_units()) as u64;
            Some(self.holds.string_units.checked_add(added).ok_or_else(|| {
                self.control
                    .stop_representation(HtmlParseResource::ManagedUtf16Units)
            })?)
        } else {
            None
        };
        self.control
            .commit_construction_logical(self.identity, self.requests, released)?;
        if let Some(held) = held {
            self.holds.string_units = held;
        }
        Ok(())
    }
}

impl<'a> RecordAdmission<'a> {
    fn new(
        control: &'a mut ParseControl,
        identity: &'a ControlIdentity,
        holds: &'a mut StorageHolds,
        requests: &'a [(HtmlParseResource, u64)],
    ) -> Self {
        Self {
            control,
            identity,
            holds,
            requests,
            layout: 0,
            leaf: false,
            insertion: None,
            contents: 0,
        }
    }

    /// The session error for an operation's refusal. An allocation the allocator refused reports
    /// the layout the growth was admitted to.
    fn error(self, error: HtmlConstructionRecordError<SessionError>) -> SessionError {
        match error {
            HtmlChargedError::Stopped(error) => error,
            HtmlChargedError::Faulted(error) => {
                if error.cause() == HtmlConstructionCause::AllocationFailed {
                    let layout = usize::try_from(self.layout).unwrap_or(usize::MAX);
                    return SessionError::Stop(self.control.construction_allocation_failed(layout));
                }
                construction_error(self.control, error)
            }
        }
    }
}

/// The session error of a refusal of an operation that grows no bank.
pub(super) fn record_error(
    control: &mut ParseControl,
    error: HtmlConstructionRecordError<HtmlParseStop>,
) -> SessionError {
    match error {
        HtmlChargedError::Stopped(stop) => SessionError::Stop(stop),
        HtmlChargedError::Faulted(error) => construction_error(control, error),
    }
}
/// As [`record_error`], for a charge that stopped the session rather than the controller.
fn charged_error(
    control: &mut ParseControl,
    error: HtmlConstructionRecordError<SessionError>,
) -> SessionError {
    match error {
        HtmlChargedError::Stopped(error) => error,
        HtmlChargedError::Faulted(error) => construction_error(control, error),
    }
}

impl SessionStorage {
    /// Reserves room for `additional` new records with geometric spare capacity, so that a
    /// document of N records costs O(N) bytes of reservation work instead of O(N^2).
    pub(super) fn reserve_records(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        let mut admission = RecordAdmission::new(control, identity, holds, &[]);
        construction
            .reserve_records(additional, &mut admission)
            .map_err(|error| admission.error(error))
    }

    /// Creates the record `kind` names, acquiring `requests` as the logical resources it adds.
    pub(super) fn create_record(
        &mut self,
        control: &mut ParseControl,
        kind: NewRecord<'_>,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlBuilderNode, SessionError> {
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        let leaf = kind.is_leaf();
        let mut admission = RecordAdmission::new(control, identity, holds, requests);
        admission.leaf = leaf;
        let created = match kind {
            NewRecord::Document(mode) => construction.create_document(mode, &mut admission),
            NewRecord::Fragment => construction.create_fragment(&mut admission),
            NewRecord::Text(units) => construction.create_text(units, &mut admission),
            NewRecord::Comment(units) => construction.create_comment(units, &mut admission),
            NewRecord::ProcessingInstruction(target, data) => {
                construction.create_processing_instruction(target, data, &mut admission)
            }
            NewRecord::Doctype(name, public, system) => {
                construction.create_doctype(name, Some(public), Some(system), &mut admission)
            }
            #[cfg(test)]
            NewRecord::Element(name, attributes, flags) => {
                construction.create_element(&name, attributes, flags, &mut admission)
            }
            NewRecord::NamedElement(name, attributes, flags) => {
                construction.create_element_named(&name, attributes, flags, &mut admission)
            }
            NewRecord::ShadowContents(properties) => {
                construction.create_shadow_contents(properties, &mut admission)
            }
        };
        created.map_err(|error| admission.error(error))
    }

    /// Interns the name of `namespace`, no prefix and the local name `units` in the storage.
    pub(super) fn intern_name(
        &mut self,
        control: &mut ParseControl,
        namespace: &Namespace,
        units: &[u16],
    ) -> Result<NameId, SessionError> {
        control.require_identity(&self.control)?;
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        let mut admission = RecordAdmission::new(control, identity, holds, &[]);
        construction
            .intern_name(namespace, None, units, &mut admission)
            .map_err(|error| admission.error(error))
    }
    /// Inserts `node` below `parent`, before `before` or last. `form_effects` says that the
    /// parse writes its form effects with the insertion. Returns whether the insertion revokes
    /// the eligibility of the standalone document, which the caller applies once the insertion
    /// has committed.
    pub(super) fn insert_record(
        &mut self,
        control: &mut ParseControl,
        node: &HtmlBuilderNode,
        parent: (usize, &HtmlBuilderNode),
        before: Option<&HtmlBuilderNode>,
        form_effects: bool,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<bool, SessionError> {
        let Self {
            construction,
            holds,
            payloads,
            fragment,
            standalone_ineligibility,
            control: identity,
            ..
        } = self;
        let mut admission = RecordAdmission::new(control, identity, holds, requests);
        admission.insertion = Some(Insertion {
            parent: parent.0,
            payloads,
            fragment,
            ineligibility: standalone_ineligibility,
            form_effects,
            revoke: false,
        });
        let inserted = construction.insert(node, parent.1, before, &mut admission);
        let revoke = admission
            .insertion
            .as_ref()
            .is_some_and(|insertion| insertion.revoke);
        match inserted {
            Ok(()) => Ok(revoke),
            Err(error) => Err(admission.error(error)),
        }
    }

    /// Removes `node` from its parent.
    pub(super) fn detach_record(
        &mut self,
        control: &mut ParseControl,
        node: &HtmlBuilderNode,
    ) -> Result<(), SessionError> {
        let Self {
            construction,
            holds,
            control: identity,
            ..
        } = self;
        let mut admission = RecordAdmission::new(control, identity, holds, &[]);
        construction
            .detach(node, &mut admission)
            .map_err(|error| admission.error(error))
    }

    /// Checks that the children of `from` can move into `to`.
    pub(super) fn check_child_transfer_record(
        &self,
        control: &mut ParseControl,
        from: &HtmlBuilderNode,
        to: &HtmlBuilderNode,
    ) -> Result<(), SessionError> {
        let mut charge = SessionCharge(control);
        let checked = self
            .construction
            .check_child_transfer(from, to, &mut charge);
        checked.map_err(|error| charged_error(charge.0, error))
    }

    /// Sets the compatibility mode of the Document `document`.
    pub(super) fn set_document_mode_record(
        &mut self,
        control: &mut ParseControl,
        document: &HtmlBuilderNode,
        mode: HtmlDocumentMode,
    ) -> Result<(), SessionError> {
        let mut charge = SessionCharge(control);
        let set = self
            .construction
            .set_document_mode(document, mode, &mut charge);
        set.map_err(|error| charged_error(charge.0, error))
    }

    /// Records link construction flags on the `link` element `element`.
    #[cfg(test)]
    pub(super) fn set_link_flags_record(
        &mut self,
        control: &mut ParseControl,
        element: &HtmlBuilderNode,
        flags: surgeist_html::elements::link::HtmlLinkFlags,
    ) -> Result<(), SessionError> {
        let mut charge = SessionCharge(control);
        let set = self
            .construction
            .set_link_flags(element, flags, &mut charge);
        set.map_err(|error| charged_error(charge.0, error))
    }
}

/// Makes `contents` the declarative shadow root of `host`, or, when `template` is given, the
/// shadow contents the template refers to in place of the contents it owned. The fields of the
/// session storage are lent one by one, so that a caller that holds a borrow of another field can
/// issue the operation.
pub(super) fn attach_shadow_of(
    construction: &mut HtmlConstructionStorage,
    holds: &mut StorageHolds,
    identity: &ControlIdentity,
    control: &mut ParseControl,
    attachment: ShadowAttachment<'_>,
    requests: &[(HtmlParseResource, u64)],
) -> Result<(), SessionError> {
    let ShadowAttachment {
        template,
        host,
        contents,
    } = attachment;
    let mut admission = RecordAdmission::new(control, identity, holds, requests);
    let attached = match (template, host) {
        (Some(template), Some(host)) => {
            construction.attach_template_shadow(template, host, contents, &mut admission)
        }
        (Some(template), None) => {
            construction.set_template_shadow_contents(template, contents, &mut admission)
        }
        (None, Some(host)) => construction.attach_shadow(host, contents, &mut admission),
        // A shadow attachment names a host or a template.
        (None, None) => return Err(latch_owner_refusal(admission.control)),
    };
    attached.map_err(|error| admission.error(error))
}

/// The nodes of a shadow attachment: the contents, and the host or the template they attach to.
pub(super) struct ShadowAttachment<'a> {
    pub(super) template: Option<&'a HtmlBuilderNode>,
    pub(super) host: Option<&'a HtmlBuilderNode>,
    pub(super) contents: &'a HtmlBuilderNode,
}

/// The controller as a consumer of operations that grow nothing, whose refusals are session
/// errors.
struct SessionCharge<'a>(&'a mut ParseControl);
impl Charge<HtmlWork, HtmlResource> for SessionCharge<'_> {
    type Stop = SessionError;
    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), SessionError> {
        Charge::charge(&mut *self.0, batch).map_err(SessionError::Stop)
    }
    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), SessionError> {
        Charge::record_largest(&mut *self.0, resource, value).map_err(SessionError::Stop)
    }
}

/// Publishes the records and returns the result with the bytes the parse held for the contents
/// vector of the publication, which the publication has freed. A refusal hands the storage back
/// with the session error, so the session keeps its records and releases them as it would after
/// any refused step.
#[expect(clippy::result_large_err)] // Refusal hands the complete storage back without allocating after failure.
pub(super) fn publish_records(
    control: &mut ParseControl,
    identity: &ControlIdentity,
    holds: &mut StorageHolds,
    storage: HtmlConstructionStorage,
    root: &HtmlBuilderNode,
) -> Result<(HtmlConstructionResult, u64), (HtmlConstructionStorage, SessionError)> {
    let mut admission = RecordAdmission::new(control, identity, holds, &[]);
    match storage.publish(root, &mut admission) {
        Ok(published) => Ok((published, admission.contents)),
        Err(refusal) => {
            let (storage, error) = refusal.into_parts();
            Err((storage, admission.error(error)))
        }
    }
}

/// An insertion's depth, edge and standalone checks run against the lent storage; this is the
/// check of whether the tree rooted at `parent` is a fragment that an associated document owns.
pub(super) fn associated_document_parent_of(
    payloads: &mut PayloadStorage,
    fragment: &Option<FragmentContextStorage>,
    control: &mut ParseControl,
    storage: &HtmlConstructionStorage,
    parent: usize,
) -> Result<bool, SessionError> {
    let root = payloads.tree_root(control, storage, parent)?;
    associated_root_of(payloads, fragment, control, storage, root)
}

/// Whether the tree rooted at `root`, found along parent edges, is a fragment that an associated
/// document owns. Only a fragment has an owner, and an owned fragment has no parent, so the root
/// decides.
pub(super) fn associated_root_of(
    payloads: &PayloadStorage,
    fragment: &Option<FragmentContextStorage>,
    control: &mut ParseControl,
    storage: &HtmlConstructionStorage,
    root: usize,
) -> Result<bool, SessionError> {
    use surgeist_html::construction::HtmlFragmentOwner;
    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    let record = storage.node_at(root).ok_or(SessionError::ForeignNode)?;
    match record.fragment_owner() {
        None => Ok(false),
        Some(HtmlFragmentOwner::ShadowHost(_)) => Ok(fragment.is_none()),
        Some(HtmlFragmentOwner::Template(template)) => {
            control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            let template_position = storage
                .node_by_result_id(template)
                .map_err(|_| SessionError::ForeignNode)?
                .position();
            let (_, state) =
                template_slot_of(payloads, fragment, control, storage, template_position)?;
            if state.contents != root {
                return Err(SessionError::TemplateMetadataUnavailable);
            }
            Ok(false)
        }
    }
}

/// A consumer of a publication that records every growth it is asked to admit and refuses the
/// admission of one bank, for the tests of what a refused publication leaves with the parse.
#[cfg(test)]
pub(super) struct ObservedAdmission<'a> {
    inner: RecordAdmission<'a>,
    refuse: Option<HtmlConstructionBank>,
    /// The bank whose first record this consumer accepts and then refuses.
    refuse_after_record: Option<HtmlConstructionBank>,
    growths: Vec<HtmlConstructionBankGrowth>,
}

#[cfg(test)]
impl Charge<HtmlWork, HtmlResource> for ObservedAdmission<'_> {
    type Stop = SessionError;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), SessionError> {
        self.inner.charge(batch)
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), SessionError> {
        self.inner.record_largest(resource, value)
    }
}

#[cfg(test)]
impl HtmlRecordAdmission for ObservedAdmission<'_> {
    fn admit_change(
        &mut self,
        storage: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), SessionError> {
        self.inner.admit_change(storage, change)
    }

    fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, SessionError> {
        self.growths.push(growth);
        if self.refuse == Some(growth.bank()) {
            return Err(SessionError::Consumed);
        }
        self.inner.admit_growth(growth)
    }

    fn record_growth(&mut self, record: HtmlConstructionGrowthRecord) -> Result<(), SessionError> {
        self.inner.record_growth(record)?;
        if self.refuse_after_record == Some(record.growth().bank()) {
            return Err(SessionError::Consumed);
        }
        Ok(())
    }

    fn record_change(
        &mut self,
        storage: &HtmlConstructionStorage,
        change: HtmlConstructionChange,
    ) -> Result<(), SessionError> {
        self.inner.record_change(storage, change)
    }
}

/// What a publication through an [`ObservedAdmission`] asked and left.
#[cfg(test)]
pub(super) struct ObservedPublication {
    /// The growths the publication asked to admit, in order.
    pub(super) growths: Vec<HtmlConstructionBankGrowth>,
    /// Whether the publication was refused.
    pub(super) refused: bool,
    /// What the parse reports for an allocation refusal after the publication ended, when the
    /// caller asked for it: the report latches a stop on the control.
    pub(super) allocation_failure: Option<SessionError>,
}

#[cfg(test)]
impl SessionStorage {
    /// Publishes the records rooted at `root` through an [`ObservedAdmission`] that refuses the
    /// admission of `refuse`, or accepts the first record of `refuse_after_record` and then
    /// refuses it. A refusal gives the records back to the storage. With `map_allocation_failure`
    /// the publication reports the stop an allocation refusal after it would map to.
    pub(super) fn publish_observed(
        &mut self,
        control: &mut ParseControl,
        root: &HtmlBuilderNode,
        refuse: Option<HtmlConstructionBank>,
        refuse_after_record: Option<HtmlConstructionBank>,
        map_allocation_failure: bool,
    ) -> ObservedPublication {
        let placeholder = HtmlConstructionStorage::new(&mut SessionCharge(control)).unwrap();
        let records = std::mem::replace(&mut self.construction, placeholder);
        let Self {
            holds,
            control: identity,
            ..
        } = self;
        let mut admission = ObservedAdmission {
            inner: RecordAdmission::new(control, identity, holds, &[]),
            refuse,
            refuse_after_record,
            growths: Vec::new(),
        };
        let refused = match records.publish(root, &mut admission) {
            Ok(_) => false,
            Err(refusal) => {
                let (records, _) = refusal.into_parts();
                self.construction = records;
                true
            }
        };
        let ObservedAdmission { inner, growths, .. } = admission;
        let allocation_failure = map_allocation_failure.then(|| {
            inner.error(HtmlChargedError::Faulted(
                HtmlConstructionCause::AllocationFailed.into(),
            ))
        });
        ObservedPublication {
            growths,
            refused,
            allocation_failure,
        }
    }
}
