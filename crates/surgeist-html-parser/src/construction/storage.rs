//! The active session storage: one arena with its payload registry, effects and evidence.

use super::*;

pub(super) struct SessionStorage {
    pub(super) csp_slot: Option<usize>,
    #[cfg(test)]
    pub(super) cancel_after_details_shift: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    pub(super) cancel_after_radio_peer: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    pub(super) cancel_after_text_append_chunk: Option<crate::HtmlParseCancellation>,
    pub(super) control: ControlIdentity,
    pub(super) effects: ConstructionEffects,
    pub(super) evidence: Option<ParseEvidence>,
    pub(super) fragment: Option<FragmentContextStorage>,
    #[cfg(test)]
    pub(super) cancel_after_removal: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    pub(super) cancel_after_textarea_reset: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    pub(super) cancel_after_output_reset: Option<crate::HtmlParseCancellation>,
    #[cfg(test)]
    pub(super) cancel_after_input_reset: Option<crate::HtmlParseCancellation>,
    /// Takes the form-association row the next creation has registered and
    /// not yet bound out of the registry, as a registry that lost the row
    /// would, and keeps it with its slot for the test to put back.
    #[cfg(test)]
    pub(super) lose_association_row: Option<Option<(usize, RetainedPayload)>>,
    #[cfg(test)]
    pub(super) cancel_after_range_append: Option<(
        Option<surgeist_html::bounded::RangeScratchRole>,
        crate::HtmlParseCancellation,
    )>,
    #[cfg(test)]
    pub(super) cancel_after_range_ingress: Option<range::RangeIngressCancellation>,
    pub(super) standalone_ineligibility: Option<StandaloneUnsupported>,
    pub(super) construction: ConstructionOwner,
    pub(super) payloads: PayloadStorage,
    pub(super) failed_text_append: Option<text_append::FailedTextAppend>,
    /// The registry row listing the document tree's style elements while every
    /// change to that list since it was built has been applied to it.
    pub(super) style_order: Option<pending_association::CurrentOrder>,
    /// The subtree whose insertion or removal last walked each color selection;
    /// see [`meta::MetaBatch`].
    pub(super) meta_batches: meta::MetaDomainTable<Option<meta::MetaBatch>>,
}

impl SessionStorage {
    pub(super) fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        let effects = ConstructionEffects::new(control)?;
        let construction = ConstructionOwner::new(control)?;
        let (values, allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (node_joins, node_joins_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (forms, forms_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, node_joins, node_joins_allocation)?;
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (bases, bases_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, forms, forms_allocation)?;
                release_vector(control, node_joins, node_joins_allocation)?;
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        let (documents, documents_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, bases, bases_allocation)?;
                release_vector(control, forms, forms_allocation)?;
                release_vector(control, node_joins, node_joins_allocation)?;
                release_vector(control, values, allocation)?;
                construction.retire(control)?;
                return Err(stop.into());
            }
        };
        // Indexed by `KindList`.
        let kind_slots = [
            KindSlots {
                slots: forms,
                allocation: forms_allocation,
            },
            KindSlots {
                slots: bases,
                allocation: bases_allocation,
            },
            KindSlots {
                slots: documents,
                allocation: documents_allocation,
            },
        ];
        let mut session = Self {
            csp_slot: None,
            #[cfg(test)]
            cancel_after_details_shift: None,
            #[cfg(test)]
            cancel_after_radio_peer: None,
            #[cfg(test)]
            cancel_after_text_append_chunk: None,
            control: control.identity(),
            effects,
            evidence: None,
            fragment: None,
            #[cfg(test)]
            cancel_after_removal: None,
            #[cfg(test)]
            cancel_after_textarea_reset: None,
            #[cfg(test)]
            cancel_after_output_reset: None,
            #[cfg(test)]
            cancel_after_input_reset: None,
            #[cfg(test)]
            lose_association_row: None,
            #[cfg(test)]
            cancel_after_range_append: None,
            #[cfg(test)]
            cancel_after_range_ingress: None,
            standalone_ineligibility: None,
            construction,
            failed_text_append: None,
            style_order: Some(pending_association::CurrentOrder::empty_main_tree()),
            meta_batches: meta::MetaDomainTable::filled(None),
            payloads: PayloadStorage {
                values,
                allocation,
                node_joins,
                node_joins_allocation,
                kind_slots,
                explicit_forms: None,
                prepared_entries: 0,
                preexisting_units: 0,
                preexisting_attributes: 0,
                audited_rows: 0,
                selectedcontent_rows: 0,
                repeated_unique_rows: 0,
                tree_stamp: NonZeroU32::new(1),
                topology_changes: Some(0),
            },
        };
        let kind = match root {
            RootKind::Document(mode) => HtmlNodeKind::Document(mode),
            #[cfg(test)]
            RootKind::Fragment => HtmlNodeKind::Fragment,
        };
        let allocated = payload_write::<Option<FormEffects>>(control, 1)
            .and_then(|()| session.allocate_native(control, kind, false, 0, 0));
        if let Err(error) = allocated {
            // The allocation failure is the error reported; a refused release has
            // latched its own stop, which is not reported over the first.
            let _ = session.release(control);
            return Err(error);
        }
        if matches!(
            session
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            let facts = (|| {
                let prepared = session.prepare_payload_slot_owned(control)?;
                session
                    .payloads
                    .push_row(
                        prepared,
                        RetainedPayload::Document {
                            node: Some(0),
                            state: template::DocumentState::Ordinary { inert: None },
                            class: crate::HtmlDocumentClass::Html,
                            meta: meta::MetaHistory::offline(),
                            interaction: dialog::InteractionHistory::new(),
                        },
                    )
                    .map_err(|_| latch_registry_row(control))?;
                let prepared = session.prepare_payload_slot_owned(control)?;
                session
                    .payloads
                    .push_row(
                        prepared,
                        RetainedPayload::NodeDocument {
                            node: Some(0),
                            document: DocumentRef::Owned(0),
                        },
                    )
                    .map_err(|_| latch_registry_row(control))?;
                Ok::<_, SessionError>(())
            })();
            if let Err(error) = facts {
                let _ = session.release(control);
                return Err(error);
            }
        }
        Ok(session)
    }

    pub(super) fn require(&self, control: &mut ParseControl) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.effects.form.is_some() {
            return Err(control
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into());
        }
        self.effects.require_idle(control)?;
        self.require_context_ready(control)?;
        Ok(())
    }
    pub(super) fn node(
        &self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        self.node_owned(control, index)
    }
    pub(super) fn node_owned(
        &self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<ParserNode, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        control.charge(WorkCategory::CopyByte, mem::size_of::<ParserNode>() as u64)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let view = self
            .construction
            .node_at(index)
            .ok_or(SessionError::ForeignNode)?;
        Ok(ParserNode {
            handle: view.handle(),
            index: view.position(),
        })
    }
    pub(super) fn root(&self, control: &mut ParseControl) -> Result<ParserNode, SessionError> {
        self.node(control, 0)
    }
    pub(super) fn index(
        &self,
        control: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<usize, SessionError> {
        self.require(control)?;
        control.charge(WorkCategory::RecordVisit, 2)?;
        self.construction
            .storage(control)?
            .position(&node.handle)
            .map_err(|_| SessionError::ForeignNode)
    }
}
