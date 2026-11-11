//! The active session storage: one arena with its payload registry, effects and evidence.

use super::*;
pub(super) struct SessionStorage {
    pub(super) csp_slot: Option<usize>,
    #[cfg(test)]
    pub(super) cancel_after_radio_peer: Option<crate::HtmlParseCancellation>,
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
        Option<surgeist_html::bounded::numeric::HtmlRangeScratchRole>,
        crate::HtmlParseCancellation,
    )>,
    #[cfg(test)]
    pub(super) cancel_after_range_ingress: Option<range::RangeIngressCancellation>,
    pub(super) standalone_ineligibility: Option<HtmlParseUnsupportedState>,
    pub(super) construction: ConstructionOwner,
    pub(super) payloads: PayloadStorage,
    /// The units the text append being applied added to the end of the last
    /// child, from the append's commit until the children-changed hook takes
    /// them. Set exactly when the hook's stage is `AppendChildren`.
    pub(super) appended_units: Option<NonZeroUsize>,
    /// A `Join::StyleOrder` row at or after this registry slot is the current
    /// list of its tree's style elements: every change to that list since the
    /// row was built has been applied to it. A change that cannot be applied
    /// raises the floor to the end of the registry, which makes every earlier
    /// row stale at once.
    pub(super) style_order_floor: usize,
    /// The offset at which the start tag being processed ended: where the
    /// parser creates the script, style and event handler content attributes of
    /// that tag.
    pub(super) token_end: Option<usize>,
    /// The row listing the start positions of event handler content attributes,
    /// once the parser has created one.
    pub(super) handler_starts: Option<usize>,
    /// The main tree, the tree rooted at record 0, has held no style element
    /// since the session began and nothing has dropped that knowledge: its
    /// order is empty without a row, and the first style inserted into it
    /// starts the row without a walk.
    pub(super) main_tree_styleless: bool,
    /// The subtree whose insertion last found its tree's style order unable to
    /// take one of its styles; see [`pending_association::StyleSubtree`].
    pub(super) style_subtree: Option<pending_association::StyleSubtree>,
    /// The order of the subtree whose removal is being applied, while the
    /// removal hooks of its styles run; see [`pending_association::DetachedOrder`].
    pub(super) detached_order: Option<pending_association::DetachedOrder>,
    /// The subtree whose insertion or removal last walked each color selection;
    /// see [`meta::MetaBatch`].
    pub(super) meta_batches: meta::MetaDomainTable<Option<meta::MetaBatch>>,
}

impl SessionStorage {
    pub(super) fn new(control: &mut ParseControl, root: RootKind) -> Result<Self, SessionError> {
        let detail_origins = DetailOrigins::new();
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
            cancel_after_radio_peer: None,
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
            appended_units: None,
            style_order_floor: 0,
            token_end: None,
            handler_starts: None,
            main_tree_styleless: true,
            style_subtree: None,
            detached_order: None,
            meta_batches: meta::MetaDomainTable::filled(None),
            payloads: PayloadStorage {
                values,
                allocation,
                node_joins,
                node_joins_allocation,
                kind_slots,
                explicit_forms: None,
                prepared_entries: 0,
                literals: [None; LITERALS],
                preexisting_units: 0,
                preexisting_attributes: 0,
                selectedcontent_rows: 0,
                repeated_unique_rows: 0,
                tree_stamp: NonZeroU32::new(1),
                topology_changes: Some(0),
                detail_origins,
            },
        };
        let kind = match root {
            RootKind::Document(mode) => NewRecord::Document(mode),
            #[cfg(test)]
            RootKind::Fragment => NewRecord::Fragment,
        };
        let root_is_document = matches!(kind, NewRecord::Document(_));
        let allocated = payload_write::<Option<FormEffects>>(control, 1)
            .and_then(|()| session.allocate_native(control, kind, false, 0, 0));
        if let Err(error) = allocated {
            // The allocation failure is the error reported; a refused release has
            // latched its own stop, which is not reported over the first.
            let _ = session.release(control);
            return Err(error);
        }
        // The root just allocated has the kind `root` names; nothing is looked up
        // that could fail while the session is live and unreleased.
        if root_is_document {
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.effects.form.is_some() {
            return Err(control
                .fail_invariant(crate::parser::control::HtmlParseInvariant::PendingFormEffects)
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
    fn node_owned(
        &self,
        control: &mut ParseControl,
        index: usize,
    ) -> Result<ParserNode, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<ParserNode>() as u64,
        )?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        self.construction
            .storage(control)?
            .position(&node.handle)
            .map_err(|_| SessionError::ForeignNode)
    }
}

#[cfg(test)]
impl SessionStorage {
    /// The readers of leaf strings over the registry and the records at rest.
    pub(super) fn leaf_strings(&self) -> provenance::LeafStrings<'_> {
        provenance::LeafStrings::with_records(
            &self.payloads,
            ConstructionView::Active(self.construction.test_storage()),
        )
    }
}
