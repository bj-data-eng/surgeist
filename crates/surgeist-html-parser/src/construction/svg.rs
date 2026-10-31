//! SVG Tiny processed state under the explicitly selected closed offline operation.
//! HTML's preparation metadata and microtask checkpoint are separate owners.
//! No resource classification, acquisition, execution or notification task is made.

use super::*;

const SVG: &str = "http://www.w3.org/2000/svg";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgScriptOrigin {
    Parser,
    Clone,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SvgScriptState {
    pub(crate) flags: SvgScriptFlags,
    pub(crate) origin: SvgScriptOrigin,
}
impl SvgScriptState {
    pub(super) fn fresh() -> Self {
        Self {
            flags: SvgScriptFlags::fresh(),
            origin: SvgScriptOrigin::Parser,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SvgScriptProcessing {
    AlreadyProcessed,
    Disconnected,
    RestrictedProcessed,
}

fn local(
    c: &mut ParseControl,
    records: ConstructionView<'_>,
    node: usize,
) -> Result<bool, SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    let record = records.node_at(node).ok_or(SessionError::ForeignNode)?;
    match record.kind() {
        HtmlNodeKind::Element(element) => {
            Ok(exact_ascii(c, element.name().namespace().uri(), SVG)?
                && exact_ascii(c, Some(element.name().local_name()), "script")?)
        }
        _ => Ok(false),
    }
}
fn state_at(
    payloads: &PayloadStorage,
    c: &mut ParseControl,
    node: usize,
) -> Result<(usize, SvgScriptState), SessionError> {
    for (slot, payload) in payloads.values.iter().enumerate() {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let RetainedPayload::SvgScript {
            node: Some(bound),
            state,
        } = payload
            && *bound == node
        {
            c.charge(
                WorkCategory::CopyByte,
                mem::size_of::<SvgScriptState>() as u64,
            )?;
            return Ok((slot, *state));
        }
    }
    Err(SessionError::SvgScriptMetadataUnavailable)
}
impl SessionStorage {
    pub(super) fn svg_script_local(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<bool, SessionError> {
        local(
            c,
            ConstructionView::Active(self.construction.storage()),
            node,
        )
    }
    pub(super) fn svg_state_at(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(usize, SvgScriptState), SessionError> {
        if !self.svg_script_local(c, node)? {
            return Err(SessionError::SvgScriptMetadataUnavailable);
        }
        state_at(&self.payloads, c, node)
    }
    pub(super) fn retain_svg_slot(
        &mut self,
        c: &mut ParseControl,
        state: SvgScriptState,
    ) -> Result<usize, SessionError> {
        self.prepare_payload_slot_owned(c)?;
        // Slot preparation already admits the scalar copy/write and entry.
        // Retain it immediately: no fallible gap may strand that acquired entry.
        let slot = self.payloads.values.len();
        self.payloads
            .values
            .push(RetainedPayload::SvgScript { node: None, state });
        Ok(slot)
    }
    /// Caller preadmits this fixed binding before actual native allocation.
    pub(super) fn bind_svg_slot(&mut self, slot: usize, node: usize) {
        let RetainedPayload::SvgScript { node: bound, .. } = &mut self.payloads.values[slot] else {
            unreachable!("owned admitted SVG slot")
        };
        *bound = Some(node);
    }
    /// Effect frames already own readiness; the same sealed domain and source
    /// identity still qualify each invocation before the flag can change.
    pub(super) fn process_svg_owned(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<SvgScriptProcessing, SessionError> {
        c.require_identity(&self.control)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        if let Some(reason) = self.standalone_ineligibility {
            return Err(c.stop_unsupported_standalone(reason).into());
        }
        if !matches!(
            self.construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(c))?
                .kind(),
            HtmlNodeKind::Document(_)
        ) {
            return Err(c
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
                .into());
        }
        let (slot, state) = self.svg_state_at(c, node)?;
        if state.flags.already_processed() {
            return Ok(SvgScriptProcessing::AlreadyProcessed);
        }
        if !self.connected(c, node, None)? {
            return Ok(SvgScriptProcessing::Disconnected);
        }
        // Known internal ordinary/inert documents only. The actual ordinary root
        // determines connectivity; a template's host relation is never followed.
        let document = self.node_document_ref(c, node)?;
        self.validate_document_ref(c, &document)?;
        // Closed fresh construction has no loader, executor, host event target,
        // listener/observer or retained SVG notification producer. No Tiny load
        // event or failed-load claim is fabricated, and no intrinsic FIFO is drained.
        payload_write::<SvgScriptFlags>(c, 1)?;
        let RetainedPayload::SvgScript { state: stored, .. } = &mut self.payloads.values[slot]
        else {
            unreachable!("qualified SVG slot")
        };
        stored.flags.mark_processed();
        Ok(SvgScriptProcessing::RestrictedProcessed)
    }
    pub(super) fn validate_svg_facts(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::SvgScript { node, .. } = payload {
                let node = node.ok_or(SessionError::SvgScriptMetadataUnavailable)?;
                if !self.svg_script_local(c, node)? {
                    return Err(SessionError::SvgScriptMetadataUnavailable);
                }
                self.node_document_ref(c, node)?;
            }
        }
        Ok(())
    }
}
impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn svg_script_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<SvgScriptState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        storage.svg_state_at(c, node).map(|(_, state)| state)
    }
    pub(crate) fn begin_svg_wrapper(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<(), SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active_mut()?;
        let index = storage.index(c, node)?;
        storage.svg_state_at(c, index)?;
        payload_write::<Option<usize>>(c, 1)?;
        storage.effects.svg_wrapper = Some(index);
        Ok(())
    }
    pub(crate) fn process_svg_wrapper(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<SvgScriptProcessing, SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active_mut()?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let position = storage
            .construction
            .storage()
            .position(&node.handle)
            .map_err(|_| SessionError::SvgScriptMetadataUnavailable)?;
        if storage.effects.svg_wrapper != Some(position) {
            return Err(SessionError::SvgScriptMetadataUnavailable);
        }
        storage.process_svg_owned(c, node.index)
    }
    pub(crate) fn finish_svg_wrapper(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        c.require_identity(&storage.control)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let position = storage
            .construction
            .storage()
            .position(&node.handle)
            .map_err(|_| SessionError::SvgScriptMetadataUnavailable)?;
        if storage.effects.svg_wrapper != Some(position) {
            return Err(SessionError::SvgScriptMetadataUnavailable);
        }
        payload_write::<Option<usize>>(c, 1)?;
        storage.effects.svg_wrapper = None;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn process_disabled_svg(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
    ) -> Result<SvgScriptProcessing, SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active_mut()?;
        let node = storage.index(c, node)?;
        storage.process_svg_owned(c, node)
    }
}
impl FrozenArena {
    #[cfg(test)]
    pub(crate) fn svg_script_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<SvgScriptState, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let arena = &storage.view;
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        if !local(c, ConstructionView::Frozen(arena), position)? {
            return Err(SessionError::SvgScriptMetadataUnavailable);
        }
        state_at(&storage.payloads, c, position).map(|(_, state)| state)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{fresh_script_owner, limits, storage_zero};
    use super::*;
    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::Event};
    use crate::parser::tree_builder::ConstructionCapability;

    fn tag(
        lexer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        c: &mut ParseControl,
    ) -> (Tag, crate::HtmlSourceRange) {
        loop {
            let event = lexer.next(c, false).unwrap().expect("real token");
            if let Event::Token(event) = event {
                let token = event.into_token(c).unwrap();
                if let TokenKind::Tag(tag) = token.kind {
                    return (tag, token.source);
                }
                let mut kind = Some(token.kind);
                release_token_kind(&mut kind, c).unwrap();
            } else {
                event.release(c).unwrap();
            }
        }
    }
    fn script(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
        parent: &ParserNode,
        proof: &ConstructionCapability<'_>,
    ) -> ParserNode {
        let (tag, source) = tag(lexer, c);
        session
            .allocate_foreign_tag_for_creation(
                c,
                &mut Some(tag),
                parent,
                ForeignNamespace::Svg,
                &source,
                proof,
            )
            .unwrap()
    }
    fn request(source: &str) -> crate::HtmlDocumentParseRequest {
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits())
    }
    fn frozen_script(frozen: &FrozenArena) -> HtmlNodeId {
        let document = frozen.document().unwrap();
        document
            .root()
            .children()
            .find(|id| {
                matches!(document.node(id).unwrap().kind(),
            HtmlNodeKind::Element(e) if e.name().namespace() == &Namespace::svg()
                && e.name().local_name() == &DocumentString::from("script"))
            })
            .unwrap()
    }
    fn no_executable_or_notification_work(session: &ParserArenaSession, c: &mut ParseControl) {
        session
            .active()
            .unwrap()
            .effects
            .require_freezable(c)
            .unwrap();
    }
    #[test]
    fn parser_connection_defers_processing_for_empty_inline_and_every_reference_class() {
        for attrs in [
            "",
            "xlink:href='offline.js'",
            "xlink:href='data:text/javascript,x'",
            "xlink:href='#local'",
            "xlink:href=''",
        ] {
            let request = request(&format!("<script {attrs}>"));
            let (mut c, mut session, proof) = fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let node = script(&mut session, &mut c, &mut lexer, &root, &proof);
            assert_eq!(
                session.svg_script_state(&mut c, &node).unwrap(),
                SvgScriptState {
                    flags: SvgScriptFlags::from_retained(false),
                    origin: SvgScriptOrigin::Parser
                }
            );
            session
                .insert_parser(&mut c, &root, &node, None, &proof)
                .unwrap();
            assert!(
                !session
                    .svg_script_state(&mut c, &node)
                    .unwrap()
                    .flags
                    .already_processed()
            );
            let attributes = session
                .element_attributes(&mut c, &node)
                .unwrap()
                .unwrap()
                .as_slice()
                .iter()
                .map(|a| (a.name().local_name().clone(), a.value().clone()))
                .collect::<Vec<_>>();
            assert_eq!(
                session.process_disabled_svg(&mut c, &node, &proof).unwrap(),
                SvgScriptProcessing::RestrictedProcessed
            );
            assert_eq!(
                session.process_disabled_svg(&mut c, &node, &proof).unwrap(),
                SvgScriptProcessing::AlreadyProcessed
            );
            assert!(
                session
                    .svg_script_state(&mut c, &node)
                    .unwrap()
                    .flags
                    .already_processed()
            );
            assert_eq!(
                session
                    .element_attributes(&mut c, &node)
                    .unwrap()
                    .unwrap()
                    .as_slice()
                    .iter()
                    .map(|a| (a.name().local_name().clone(), a.value().clone()))
                    .collect::<Vec<_>>(),
                attributes
            );
            no_executable_or_notification_work(&session, &mut c);
            drop(attributes);
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn detached_script_stays_false_until_actual_nonparser_connection_and_never_rearms() {
        let request = request("<script>");
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = script(&mut session, &mut c, &mut lexer, &root, &proof);
        assert_eq!(
            session.process_disabled_svg(&mut c, &node, &proof).unwrap(),
            SvgScriptProcessing::Disconnected
        );
        assert!(
            !session
                .svg_script_state(&mut c, &node)
                .unwrap()
                .flags
                .already_processed()
        );
        session
            .insert_parser(&mut c, &root, &node, None, &proof)
            .unwrap();
        session.remove(&mut c, &node, Some(&proof)).unwrap();
        assert!(
            !session
                .svg_script_state(&mut c, &node)
                .unwrap()
                .flags
                .already_processed()
        );
        session
            .insert(&mut c, &root, &node, None, Some(&proof))
            .unwrap();
        assert!(
            session
                .svg_script_state(&mut c, &node)
                .unwrap()
                .flags
                .already_processed()
        );
        session.remove(&mut c, &node, Some(&proof)).unwrap();
        assert_eq!(
            session.process_disabled_svg(&mut c, &node, &proof).unwrap(),
            SvgScriptProcessing::AlreadyProcessed
        );
        session
            .insert(&mut c, &root, &node, None, Some(&proof))
            .unwrap();
        no_executable_or_notification_work(&session, &mut c);
        lexer.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        assert_eq!(
            frozen
                .svg_script_state(&mut c, &frozen_script(&frozen))
                .unwrap(),
            SvgScriptState {
                flags: SvgScriptFlags::from_retained(true),
                origin: SvgScriptOrigin::Parser
            }
        );
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn clones_copy_false_and_true_before_insertion_and_keep_independent_identity() {
        for processed in [false, true] {
            let request = request("<script xlink:href='#saved'>");
            let (mut c, mut session, proof) = fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let source = script(&mut session, &mut c, &mut lexer, &root, &proof);
            if processed {
                session
                    .insert_parser(&mut c, &root, &source, None, &proof)
                    .unwrap();
                session
                    .process_disabled_svg(&mut c, &source, &proof)
                    .unwrap();
            }
            let index = session
                .active_mut()
                .unwrap()
                .clone_node(&mut c, source.index, DocumentRef::Owned(0))
                .unwrap();
            let copy = session.active().unwrap().node(&mut c, index).unwrap();
            assert_ne!(copy.index, source.index);
            assert_eq!(
                session.svg_script_state(&mut c, &copy).unwrap(),
                SvgScriptState {
                    flags: SvgScriptFlags::from_retained(processed),
                    origin: SvgScriptOrigin::Clone
                }
            );
            assert_eq!(
                session.process_disabled_svg(&mut c, &copy, &proof).unwrap(),
                if processed {
                    SvgScriptProcessing::AlreadyProcessed
                } else {
                    SvgScriptProcessing::Disconnected
                }
            );
            session
                .insert(&mut c, &root, &copy, None, Some(&proof))
                .unwrap();
            assert!(
                session
                    .svg_script_state(&mut c, &copy)
                    .unwrap()
                    .flags
                    .already_processed()
            );
            assert_eq!(
                session
                    .svg_script_state(&mut c, &source)
                    .unwrap()
                    .flags
                    .already_processed(),
                processed
            );
            no_executable_or_notification_work(&session, &mut c);
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn inert_contents_script_is_disconnected_and_real_adoption_then_connection_marks_it() {
        let request = request("<template><script>");
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let (tag, _) = tag(&mut lexer, &mut c);
        let template = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &root,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        let contents = session
            .template_contents(&mut c, &template)
            .unwrap()
            .unwrap();
        let node = script(&mut session, &mut c, &mut lexer, &contents, &proof);
        session
            .insert_parser(&mut c, &contents, &node, None, &proof)
            .unwrap();
        let inert = session
            .node_document(&mut c, &node)
            .unwrap()
            .node(&mut c, &session)
            .unwrap();
        assert_ne!(inert.index, root.index);
        assert_eq!(
            session.process_disabled_svg(&mut c, &node, &proof).unwrap(),
            SvgScriptProcessing::Disconnected
        );
        session
            .insert(&mut c, &root, &node, None, Some(&proof))
            .unwrap();
        assert_eq!(
            session
                .node_document(&mut c, &node)
                .unwrap()
                .node(&mut c, &session)
                .unwrap()
                .index,
            root.index
        );
        assert!(
            session
                .svg_script_state(&mut c, &node)
                .unwrap()
                .flags
                .already_processed()
        );
        assert!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(contents.index)
                .expect("retained construction position")
                .children()
                .map(|node| node.position())
                .collect::<Vec<_>>()
                .is_empty()
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn wrapper_barrier_blocks_freeze_and_useful_reads_before_quiescent_completion() {
        let request = request("<script>");
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = script(&mut session, &mut c, &mut lexer, &root, &proof);
        session
            .insert_parser(&mut c, &root, &node, None, &proof)
            .unwrap();
        lexer.release(&mut c).unwrap();
        session.begin_svg_wrapper(&mut c, &node, &proof).unwrap();
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::PendingIntrinsicWork)
        );
        assert!(session.active().is_ok());
        assert_eq!(
            session.process_svg_wrapper(&mut c, &node, &proof).unwrap(),
            SvgScriptProcessing::RestrictedProcessed
        );
        session.finish_svg_wrapper(&mut c, &node).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        assert!(
            frozen
                .svg_script_state(&mut c, &frozen_script(&frozen))
                .unwrap()
                .flags
                .already_processed()
        );
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn unavailable_state_and_foreign_control_cannot_publish_processing_or_mutation() {
        let request = request("<script>");
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = script(&mut session, &mut c, &mut lexer, &root, &proof);
        let foreign =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8("<script>"), limits());
        let mut other = ParseControl::new(*foreign.limits(), foreign.cancellation().clone());
        other.admit_source(foreign.source()).unwrap();
        assert!(matches!(
            session.process_disabled_svg(&mut other, &node, &proof),
            Err(SessionError::Stop(_))
        ));
        assert!(
            !session
                .svg_script_state(&mut c, &node)
                .unwrap()
                .flags
                .already_processed()
        );
        // Actual missing retained metadata, not an invented imported false bit.
        let (slot, _) = session
            .active()
            .unwrap()
            .svg_state_at(&mut c, node.index)
            .unwrap();
        let RetainedPayload::SvgScript { node: bound, .. } =
            &mut session.active_mut().unwrap().payloads.values[slot]
        else {
            unreachable!()
        };
        *bound = None;
        assert_eq!(
            session.svg_script_state(&mut c, &node).err(),
            Some(SessionError::SvgScriptMetadataUnavailable)
        );
        assert_eq!(
            session.insert(&mut c, &root, &node, None, Some(&proof)),
            Err(SessionError::SvgScriptMetadataUnavailable)
        );
        assert!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(node.index)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position())
                .is_none()
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    fn flag_cost() -> u64 {
        let request = request("<script>");
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let node = script(&mut session, &mut c, &mut lexer, &root, &proof);
        session
            .insert_parser(&mut c, &root, &node, None, &proof)
            .unwrap();
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session.process_disabled_svg(&mut c, &node, &proof).unwrap();
        let cost = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        cost
    }
    #[test]
    fn every_processing_work_prefix_including_one_short_write_retains_false_and_first_stop() {
        let cost = flag_cost();
        for allowance in 0..cost {
            let request = request("<script>");
            let (mut c, mut session, proof) = fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let node = script(&mut session, &mut c, &mut lexer, &root, &proof);
            session
                .insert_parser(&mut c, &root, &node, None, &proof)
                .unwrap();
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - allowance,
            )
            .unwrap();
            let first = session
                .process_disabled_svg(&mut c, &node, &proof)
                .unwrap_err();
            let SessionError::Stop(stop) = first else {
                panic!("actual admission stop")
            };
            assert_eq!(c.stop(), Some(stop));
            assert!(session.active().unwrap().payloads.values.iter().any(|p|matches!(p,
                RetainedPayload::SvgScript { node:Some(bound),state } if *bound==node.index && !state.flags.already_processed())));
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(stop));
        }
    }
    #[test]
    fn svg_metadata_slot_exact_and_one_short_retains_actual_registry_capacity() {
        // Fresh ordinary root owns record/self-document/document facts (3)
        // and its one actual append-field index entry (1).
        // The SVG metadata is a separate retained scalar entry, never a raw bit default.
        for cap in [4, 5] {
            let mut c = ParseControl::new(
                limits()
                    .into_builder()
                    .auxiliary_entries(cap)
                    .finish()
                    .unwrap(),
                crate::HtmlParseCancellation::new(),
            );
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let result = session
                .active_mut()
                .unwrap()
                .retain_svg_slot(&mut c, SvgScriptState::fresh());
            if cap == 5 {
                let slot = result.unwrap();
                assert!(matches!(
                    session.active().unwrap().payloads.values[slot],
                    RetainedPayload::SvgScript {
                        node: None,
                        state: SvgScriptState { flags, .. }
                    } if !flags.already_processed()
                ));
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    5
                );
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::SvgScriptMetadataUnavailable)
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        ..
                    }))
                ));
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    4
                );
            }
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn svg_processing_cancel_admission_keeps_false_and_imported_domain_is_unavailable() {
        {
            let request = request("<script>");
            let (mut c, mut session, proof) = fresh_script_owner(&request);
            let root = session.root(&mut c).unwrap();
            let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
            let node = script(&mut session, &mut c, &mut lexer, &root, &proof);
            session
                .insert_parser(&mut c, &root, &node, None, &proof)
                .unwrap();
            request.cancellation().cancel();
            assert_eq!(
                session.process_disabled_svg(&mut c, &node, &proof),
                Err(SessionError::Stop(ParseStop::Cancelled))
            );
            assert!(
                session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .any(|p| matches!(p,
            RetainedPayload::SvgScript{node:Some(n),state} if *n==node.index && !state.flags.already_processed()))
            );
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
        let request = request("<script>");
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let imported = HtmlElement::new(
            QualifiedName::new(Namespace::svg(), None, DocumentString::from("script")),
            OrderedAttributes::default(),
        );
        let imported = session.allocate_element(&mut c, imported).unwrap();
        assert!(matches!(
            session.process_disabled_svg(&mut c, &imported, &proof),
            Err(SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                StandaloneUnsupported::Provenance
            )))
        ));
        assert!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(imported.index)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position())
                .is_none()
        );
        assert!(
            session
                .active()
                .unwrap()
                .construction
                .node_at(root.index)
                .expect("retained construction position")
                .children()
                .map(|node| node.position())
                .collect::<Vec<_>>()
                .iter()
                .all(|n| *n != imported.index)
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn svg_creation_alias_rejection_preserves_original_before_metadata_slot_and_native_allocation()
    {
        let request = request("<script xlink:href='#saved'>");
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let (tag, source) = tag(&mut lexer, &mut c);
        let alias = tag.name.value.text().clone();
        let mut input = Some(tag);
        let nodes = session.active().unwrap().construction.storage().len();
        let slots = session.active().unwrap().payloads.values.len();
        assert_eq!(
            session
                .allocate_foreign_tag_for_creation(
                    &mut c,
                    &mut input,
                    &root,
                    ForeignNamespace::Svg,
                    &source,
                    &proof
                )
                .err(),
            Some(SessionError::AliasedPayload)
        );
        assert!(input.is_some());
        assert_eq!(
            session.active().unwrap().construction.storage().len(),
            nodes
        );
        assert_eq!(session.active().unwrap().payloads.values.len(), slots);
        drop(alias);
        release_token_kind(&mut input.map(TokenKind::Tag), &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    fn detached_svg_siblings<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
    ) -> (
        ParseControl,
        ParserArenaSession,
        ConstructionCapability<'a>,
        ParserNode,
        ParserNode,
        ParserNode,
        ParserNode,
    ) {
        let (mut c, mut session, proof) = fresh_script_owner(request);
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let (tag, _) = tag(&mut lexer, &mut c);
        let parent = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &root,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        let first = script(&mut session, &mut c, &mut lexer, &parent, &proof);
        session
            .insert_parser(&mut c, &parent, &first, None, &proof)
            .unwrap();
        let second = script(&mut session, &mut c, &mut lexer, &parent, &proof);
        session
            .insert_parser(&mut c, &parent, &second, None, &proof)
            .unwrap();
        assert!(
            !session
                .svg_script_state(&mut c, &first)
                .unwrap()
                .flags
                .already_processed()
        );
        assert!(
            !session
                .svg_script_state(&mut c, &second)
                .unwrap()
                .flags
                .already_processed()
        );
        lexer.release(&mut c).unwrap();
        (c, session, proof, root, parent, first, second)
    }
    #[test]
    fn nonparser_subtree_connection_processes_static_ordinary_list_in_document_order_at_every_cut()
    {
        let request = request("<div><script/><script/>");
        let (mut c, mut session, proof, root, parent, _, _) = detached_svg_siblings(&request);
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session
            .insert(&mut c, &root, &parent, None, Some(&proof))
            .unwrap();
        let cost = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        let mut observed_first_only = false;
        for allowance in 0..cost {
            let (mut c, mut session, proof, root, parent, first, second) =
                detached_svg_siblings(&request);
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - allowance,
            )
            .unwrap();
            assert!(matches!(
                session.insert(&mut c, &root, &parent, None, Some(&proof)),
                Err(SessionError::Stop(_))
            ));
            let stored = |node| {
                session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .find_map(|p| match p {
                        RetainedPayload::SvgScript {
                            node: Some(bound),
                            state,
                        } if *bound == node => Some(state.flags.already_processed()),
                        _ => None,
                    })
                    .unwrap()
            };
            let first = stored(first.index);
            let second = stored(second.index);
            assert!(
                !second || first,
                "second may never process before first, cut {allowance}"
            );
            observed_first_only |= first && !second;
            let stop = c.stop();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
        assert!(
            observed_first_only,
            "a genuine admitted prefix marks just the first document-order script"
        );
    }
    #[test]
    fn every_svg_metadata_registration_work_cut_reclaims_the_actual_scalar_entry() {
        let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        session
            .active_mut()
            .unwrap()
            .retain_svg_slot(&mut c, SvgScriptState::fresh())
            .unwrap();
        let cost = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        for allowance in 0..cost {
            let mut c = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                WorkCategory::StateStep,
                limits().work_credits() - used - allowance,
            )
            .unwrap();
            assert!(matches!(
                session
                    .active_mut()
                    .unwrap()
                    .retain_svg_slot(&mut c, SvgScriptState::fresh()),
                Err(SessionError::Stop(_))
            ));
            let stop = c.stop();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
    }
}
