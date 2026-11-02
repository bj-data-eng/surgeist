//! Source: [Image](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#image-button-state-(type=image)),
//! [File](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#file-upload-state-(type=file))
//! and [value](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-input-value). Ordinary immutable
//! construction establishes empty File resources and initial Image coordinates;
//! images-disabled is explicit operation evidence, independent of scripting.
use super::*;

// A parser-phase specialization: source values are actual managed raw slots.
// HTML owns the observation/history rules; this owner still qualifies, admits,
// compares, copies and retires their backing strings.
pub(super) type ImageState = surgeist_html::HtmlInputImageHistory<usize>;
impl SessionStorage {
    fn image_source(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        for (position, attribute) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            if attribute.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attribute.name().local_name()), "src")?
            {
                return self.attribute_value_slot(c, node, position).map(Some);
            }
        }
        Ok(None)
    }
    fn image_source_value(
        &self,
        c: &mut ParseControl,
        source: usize,
    ) -> Result<&DocumentString, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::String {
            value, allocation, ..
        }) = self.payloads.values.get(source)
        else {
            return Err(SessionError::InputMetadataUnavailable);
        };
        c.require_document_string(allocation, value)?;
        Ok(value)
    }
    fn input_images_supported(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<crate::HtmlImageSupport, SessionError> {
        let document = self.node_document_ref(c, node)?;
        self.validate_document_ref(c, &document)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(fragment) = &self.fragment {
            let id = match &document {
                DocumentRef::Captured(id) => id,
                // Temporary/inert owned documents use only the explicitly
                // attested policy, retaining their distinct document identity.
                DocumentRef::Owned(_) => fragment.context.target_document(),
            };
            c.charge(WorkCategory::RecordVisit, 2)?;
            let captured = fragment
                .context
                .capture()
                .node(id.subject())
                .map_err(|_| SessionError::ContextUnavailable)?;
            let crate::HtmlCapturedNodeKind::Document(facts) = captured.kind() else {
                return Err(SessionError::NotDocument);
            };
            Ok(facts.parser_creation_image_support())
        } else {
            let slot = self.completion_slot(c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let Some(RetainedPayload::StandaloneDocumentCompletion { state, .. }) =
                self.payloads.values.get(slot)
            else {
                return Err(latch_registry_row(c));
            };
            let _ = state;
            // Only the admitted fresh offline prologue issues this fact; the
            // sealed ordinary capability explicitly carries Disabled too.
            Ok(crate::HtmlImageSupport::Disabled)
        }
    }
    fn image_update(
        &self,
        c: &mut ParseControl,
        node: usize,
        source: Option<usize>,
    ) -> Result<(), SessionError> {
        use surgeist_html::{HtmlInputImageUpdateGuard as Guard, HtmlInputImagesDisabled};
        // Preserve the source prefix: no source/empty source returns before
        // the actual creation-qualified environment lookup and URL/resource work.
        let guard = match source {
            None => Guard::for_source(None),
            Some(source) => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                Guard::for_source(Some(self.image_source_value(c, source)?))
            }
        };
        match guard {
            Guard::NoSource | Guard::EmptySource => return Ok(()),
            Guard::NeedsImageEnvironment => {}
            // `for_source` never answers the environment-discharged state.
            Guard::ImagesDisabled => return Err(latch_registry_row(c)),
        }
        match self.input_images_supported(c, node)? {
            crate::HtmlImageSupport::Disabled => {
                if guard.with_disabled_images(HtmlInputImagesDisabled::ExplicitlyDisabled)
                    != Guard::ImagesDisabled
                {
                    return Err(latch_registry_row(c));
                }
                Ok(())
            }
            crate::HtmlImageSupport::Unavailable => Err(SessionError::ContextFactUnavailable(
                crate::HtmlCaptureFact::ImageSupport,
            )),
            crate::HtmlImageSupport::Enabled => Err(SessionError::Stop(
                c.stop_unsupported_standalone(StandaloneUnsupported::ImageResource),
            )),
        }
    }
    fn store_image_source(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        source: Option<usize>,
    ) -> Result<(), SessionError> {
        let mut state = self.read_input_state(c, slot)?;
        let retired = state.image.observed_source().flatten().copied();
        let retained = source
            .map(|source| self.copy_raw_field(c, source))
            .transpose()?;
        state.image = state.image.with_observed_source(retained);
        self.write_input_state(c, slot, state)?;
        if let Some(retired) = retired {
            self.retire_raw_field(c, retired)?;
        }
        Ok(())
    }
    pub(super) fn image_type_entered(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        slot: usize,
    ) -> Result<(), SessionError> {
        let state = self.read_input_state(c, slot)?;
        let source = self.image_source(c, node)?;
        let changed =
            state
                .image
                .entering_requires_observation(source.as_ref(), |old, current| {
                    let old = textarea_raw(&self.payloads, c, *old)?;
                    let current = self.image_source_value(c, *current)?;
                    exact_string_equal(c, old, current)
                })?;
        if changed {
            self.image_update(c, node, source)?;
            self.store_image_source(c, slot, source)?;
        }
        Ok(())
    }
    pub(super) fn image_source_changed(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        slot: usize,
    ) -> Result<(), SessionError> {
        // Direct applicable src updates run even when exact text is unchanged.
        // Do not use the type-reentry comparison shortcut here.
        let source = self.image_source(c, node)?;
        self.image_update(c, node, source)?;
        self.store_image_source(c, slot, source)
    }
}

#[cfg(test)]
pub(super) fn append_native_input_value(
    records: ConstructionView<'_>,
    payloads: &PayloadStorage,
    c: &mut ParseControl,
    node: usize,
    state: InputState,
    destination: &mut Utf16Buffer,
) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    let (_, raw, _) = find_input_state(payloads, c, node).map(|(raw, state)| (node, raw, state))?;
    let internal = textarea_raw(payloads, c, raw)?.code_units();
    let HtmlNodeKind::Element(element) = records
        .node_at(node)
        .ok_or(SessionError::ForeignNode)?
        .kind()
    else {
        return Err(SessionError::WrongToken);
    };
    let mut content = None;
    if matches!(
        state.flags.input_type().value_mode(),
        crate::HtmlInputValueMode::Default | crate::HtmlInputValueMode::DefaultOn
    ) {
        for attribute in element.attributes().as_slice() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            if attribute.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attribute.name().local_name()), "value")?
            {
                content = Some(attribute.value().code_units());
                break;
            }
        }
    }
    let files = state
        .selected_files_empty
        .then_some(crate::HtmlInputFileSelection::Empty);
    let units = state
        .flags
        .input_type()
        .value_mode()
        .readback(internal, content, files)
        .ok_or(SessionError::InputMetadataUnavailable)?;
    destination.extend_units(units, c)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::{input_owner, limits, radio_next_attribute, storage_zero};
    use super::*;
    fn fixture<'a>(
        request: &'a crate::HtmlDocumentParseRequest,
    ) -> (
        ParseControl,
        ParserArenaSession,
        crate::parser::tree_builder::ConstructionCapability<'a>,
        crate::parser::tokenizer::tests::LexerFixture<'a>,
        ParserNode,
    ) {
        let (mut c, mut session, cap, lexer, parent, mut tag, range) = input_owner(request);
        let node = session
            .allocate_input_for_creation(
                &mut c,
                &mut tag,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &cap,
            )
            .unwrap();
        assert!(tag.is_none());
        (c, session, cap, lexer, node)
    }
    fn source(storage: &SessionStorage, c: &mut ParseControl, node: usize) -> Option<Vec<u16>> {
        let (_, state) = storage.radio_input(c, node).unwrap().unwrap();
        let ImageState::Last { source, .. } = state.image else {
            panic!("last Image");
        };
        source.map(|s| {
            textarea_raw(&storage.payloads, c, s)
                .unwrap()
                .code_units()
                .to_vec()
        })
    }
    #[test]
    fn image_source_history_is_independent_of_retired_attribute_and_nonimage_updates() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input type=image src=A><i src=B><i type=text><i src=C><i type=image>",
            ),
            limits(),
        );
        let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
        let old = session
            .active()
            .unwrap()
            .radio_input(&mut c, node.index)
            .unwrap()
            .unwrap()
            .1
            .image;
        let ImageState::Last {
            source: Some(old),
            coordinate,
        } = old
        else {
            panic!("old");
        };
        assert_eq!((coordinate.x(), coordinate.y()), (0, 0));
        let native = session
            .active()
            .unwrap()
            .image_source(&mut c, node.index)
            .unwrap()
            .unwrap();
        assert!(!Arc::ptr_eq(
            textarea_raw(&session.active().unwrap().payloads, &mut c, old)
                .unwrap()
                .shared_code_units(),
            session
                .active()
                .unwrap()
                .image_source_value(&mut c, native)
                .unwrap()
                .shared_code_units()
        ));
        for expected in [
            [66].as_slice(),
            [66].as_slice(),
            [66].as_slice(),
            [67].as_slice(),
        ] {
            let mut attr = radio_next_attribute(&mut lexer, &mut c);
            session
                .set_input_attribute(&mut c, &node, &mut attr, &cap)
                .unwrap();
            assert!(attr.is_none());
            assert_eq!(
                source(session.active().unwrap(), &mut c, node.index).as_deref(),
                Some(expected)
            );
        }
        assert_eq!(
            session
                .input_state(&mut c, &node)
                .unwrap()
                .flags
                .input_type(),
            crate::HtmlInputType::Image
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn file_to_value_installs_actual_default_and_clears_dirty_before_sanitization() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input type=file value='A&#13;B&#10;C'><i type=text>",
            ),
            limits(),
        );
        let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
        let (slot, mut state) = session
            .active()
            .unwrap()
            .radio_input(&mut c, node.index)
            .unwrap()
            .unwrap();
        state.flags = crate::HtmlInputFlags::from_retained(
            state.flags.input_type(),
            true,
            false,
            false,
            false,
            false,
        );
        session
            .active_mut()
            .unwrap()
            .write_input_state(&mut c, slot, state)
            .unwrap();
        assert!(
            session
                .input_state(&mut c, &node)
                .unwrap()
                .flags
                .dirty_value()
        );
        let mut attr = radio_next_attribute(&mut lexer, &mut c);
        session
            .set_input_attribute(&mut c, &node, &mut attr, &cap)
            .unwrap();
        let state = session.input_state(&mut c, &node).unwrap();
        assert!(!state.flags.dirty_value());
        let mut value = Utf16Buffer::new(&mut c).unwrap();
        session
            .append_input_internal_value(&mut c, &node, &mut value)
            .unwrap();
        assert_eq!(value.units(), [65, 66, 67]);
        value.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn file_image_clone_five_flags_and_reset_keep_empty_files_and_fresh_coordinates() {
        for kind in ["file", "image"] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!(
                    "<html><head></head><input type={kind} value='A&#13;B' src=X checked>"
                )),
                limits(),
            );
            let (mut c, mut session, _cap, lexer, node) = fixture(&request);
            let (slot, mut state) = session
                .active()
                .unwrap()
                .radio_input(&mut c, node.index)
                .unwrap()
                .unwrap();
            state.flags = crate::HtmlInputFlags::from_retained(
                state.flags.input_type(),
                true,
                true,
                false,
                true,
                true,
            );
            session
                .active_mut()
                .unwrap()
                .write_input_state(&mut c, slot, state)
                .unwrap();
            let copy = session
                .active_mut()
                .unwrap()
                .clone_node(&mut c, node.index, DocumentRef::Owned(0))
                .unwrap();
            let (copy_slot, cloned) = session
                .active()
                .unwrap()
                .radio_input(&mut c, copy)
                .unwrap()
                .unwrap();
            assert!(
                cloned.flags.dirty_value()
                    && cloned.flags.dirty_checkedness()
                    && cloned.flags.indeterminate()
            );
            assert!(!cloned.flags.checkedness() && !cloned.flags.user_validity());
            assert!(cloned.selected_files_empty);
            if kind == "image" {
                let ImageState::Last {
                    source: Some(a),
                    coordinate,
                } = state.image
                else {
                    panic!("source")
                };
                let ImageState::Last {
                    source: Some(b),
                    coordinate: cloned_coordinate,
                } = cloned.image
                else {
                    panic!("copy")
                };
                assert_eq!(coordinate, cloned_coordinate);
                assert_eq!((cloned_coordinate.x(), cloned_coordinate.y()), (0, 0));
                assert_ne!(a, b);
                assert!(!Arc::ptr_eq(
                    textarea_raw(&session.active().unwrap().payloads, &mut c, a)
                        .unwrap()
                        .shared_code_units(),
                    textarea_raw(&session.active().unwrap().payloads, &mut c, b)
                        .unwrap()
                        .shared_code_units()
                ));
            }
            session
                .active_mut()
                .unwrap()
                .reset_input(&mut c, copy, copy_slot)
                .unwrap();
            let reset = session
                .active()
                .unwrap()
                .radio_input(&mut c, copy)
                .unwrap()
                .unwrap()
                .1;
            assert!(
                !reset.flags.dirty_value()
                    && !reset.flags.dirty_checkedness()
                    && reset.flags.indeterminate()
            );
            assert!(reset.flags.checkedness() && reset.selected_files_empty);
            assert_eq!(reset.image, cloned.image);
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    #[test]
    fn image_managed_source_alias_refuses_before_take_then_empty_and_removal_are_distinct() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head></head><input type=image src=A><i src=''>"),
            limits(),
        );
        let (mut c, mut session, cap, mut lexer, node) = fixture(&request);
        let mut attr = radio_next_attribute(&mut lexer, &mut c);
        let pointer = attr
            .as_ref()
            .unwrap()
            .value
            .value
            .text()
            .shared_code_units()
            .as_ptr();
        let HtmlNodeKind::Element(element) = &session
            .active()
            .unwrap()
            .construction
            .node_at(node.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("element")
        };
        let alias = element.attributes().clone();
        assert_eq!(
            session.set_input_attribute(&mut c, &node, &mut attr, &cap),
            Err(SessionError::AliasedPayload)
        );
        assert_eq!(
            attr.as_ref()
                .unwrap()
                .value
                .value
                .text()
                .shared_code_units()
                .as_ptr(),
            pointer
        );
        assert_eq!(
            source(session.active().unwrap(), &mut c, node.index),
            Some(vec![65])
        );
        drop(alias);
        session
            .set_input_attribute(&mut c, &node, &mut attr, &cap)
            .unwrap();
        assert!(attr.is_none());
        assert_eq!(
            source(session.active().unwrap(), &mut c, node.index),
            Some(vec![])
        );
        session
            .remove_managed_attribute(&mut c, &node, "src", mutation::ManagedFamily::Input)
            .unwrap();
        assert_eq!(source(session.active().unwrap(), &mut c, node.index), None);
        let (_, state) = session
            .active()
            .unwrap()
            .radio_input(&mut c, node.index)
            .unwrap()
            .unwrap();
        let ImageState::Last {
            source: None,
            coordinate,
        } = state.image
        else {
            panic!("absence retained")
        };
        assert_eq!((coordinate.x(), coordinate.y()), (0, 0));
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    #[test]
    fn actual_captured_clone_uses_new_environment_not_original_disabled_or_target_only_policy() {
        use crate::{
            HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCapturedIsValue,
            HtmlDocumentMode, HtmlFragmentContext, HtmlFragmentParseRequest, HtmlHostFactsRevision,
            HtmlImageSupport, HtmlScriptingStatus, Namespace, OrderedAttributes, QualifiedName,
        };
        for support in [
            HtmlImageSupport::Unavailable,
            HtmlImageSupport::Disabled,
            HtmlImageSupport::Enabled,
        ] {
            let mut builder = HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(100, 100, 100, 1000, 100, 100, 100, 16_777_216),
                HtmlHostFactsRevision::fresh(),
            );
            let entry = builder
                .add_document(
                    HtmlDocumentMode::NoQuirks,
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            builder
                .establish_parser_creation_image_support(&entry, HtmlImageSupport::Disabled)
                .unwrap();
            let target = builder
                .add_document(
                    HtmlDocumentMode::NoQuirks,
                    HtmlScriptingStatus::Disabled,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            // A local Disabled observation alone cannot cover the new clone.
            builder
                .establish_document_image_support(&target, HtmlImageSupport::Disabled)
                .unwrap();
            if support != HtmlImageSupport::Unavailable {
                builder
                    .establish_parser_creation_image_support(&target, support)
                    .unwrap();
            }
            let div = builder
                .add_element(
                    &entry,
                    QualifiedName::new(Namespace::html(), None, "div"),
                    OrderedAttributes::default(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
            builder.establish_root(&div).unwrap();
            builder.establish_no_shadow_root(&div).unwrap();
            let capture = builder.finish().unwrap();
            let target = capture
                .document_id(&capture.subject_id(&target).unwrap())
                .unwrap();
            let context =
                HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
            let request = HtmlFragmentParseRequest::new(
                crate::HtmlSource::utf8("<input type=image src=X>"),
                context,
                limits(),
            )
            .unwrap();
            let mut driver = crate::parser::tree_builder::initialized_fragment_fixture(&request);
            let mut lexer = crate::parser::tokenizer::tests::LexerFixture::new(
                request.source(),
                crate::parser::tokenizer::states::State::Data,
                None,
                driver.input_fixture_parts().unwrap().0,
            )
            .unwrap();
            let (c, session, cap) = driver.input_fixture_parts().unwrap();
            let parent = session.active().unwrap().node(c, 1).unwrap();
            let (mut tag, range) = loop {
                let event = lexer.next(c, false).unwrap().unwrap();
                if let crate::parser::tokenizer::token::Event::Token(event) = event {
                    let token = event.into_token(c).unwrap();
                    if let TokenKind::Tag(t) = token.kind {
                        break (Some(t), token.source);
                    }
                    release_token_kind(&mut Some(token.kind), c).unwrap();
                } else {
                    event.release(c).unwrap();
                }
            };
            let node = session
                .allocate_input_for_creation(
                    c,
                    &mut tag,
                    CreationLocation {
                        intended_parent: &parent,
                        current_form: None,
                        context: CreationContext::Ordinary,
                    },
                    &range,
                    cap,
                )
                .unwrap();
            let original_document = session
                .active()
                .unwrap()
                .node_document_ref(c, node.index)
                .unwrap();
            assert!(matches!(original_document, DocumentRef::Owned(_)));
            assert_eq!(
                source(session.active().unwrap(), c, node.index),
                Some(vec![88])
            );
            let target_id = target.clone();
            let result = session.active_mut().unwrap().clone_node(
                c,
                node.index,
                DocumentRef::Captured(target_id.clone()),
            );
            match support {
                HtmlImageSupport::Disabled => {
                    let cloned = result.unwrap();
                    assert_eq!(
                        session
                            .active()
                            .unwrap()
                            .node_document_ref(c, cloned)
                            .unwrap(),
                        DocumentRef::Captured(target_id)
                    );
                    assert_eq!(source(session.active().unwrap(), c, cloned), Some(vec![88]));
                }
                HtmlImageSupport::Unavailable => assert_eq!(
                    result,
                    Err(SessionError::ContextFactUnavailable(
                        crate::HtmlCaptureFact::ImageSupport
                    ))
                ),
                HtmlImageSupport::Enabled => assert_eq!(
                    result,
                    Err(SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                        StandaloneUnsupported::ImageResource
                    )))
                ),
            }
            lexer.release(c).unwrap();
            driver.release().unwrap();
        }
    }

    #[test]
    fn file_image_cancel_after_actual_reset_preserves_first_stop_and_all_owned_history_cleanup() {
        for kind in ["file", "image"] {
            let request = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(format!(
                    "<html><head></head><input type={kind} src='A&amp;B' value='X&#13;Y'>"
                )),
                limits(),
            );
            let (mut c, mut session, cap, lexer, parent, mut tag, range) = input_owner(&request);
            session.active_mut().unwrap().cancel_after_input_reset =
                Some(request.cancellation().clone());
            let result = session.allocate_input_for_creation(
                &mut c,
                &mut tag,
                CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: CreationContext::Ordinary,
                },
                &range,
                &cap,
            );
            assert_eq!(result.err(), Some(SessionError::Stop(ParseStop::Cancelled)));
            assert!(tag.is_none());
            assert!(
                session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .any(|p| matches!(p, RetainedPayload::Input { complete: true, .. }))
            );
            let first = c.first_stop_usage().unwrap().clone();
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(ParseStop::Cancelled))
            );
            assert_eq!(c.first_stop_usage(), Some(&first));
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(ParseStop::Cancelled));
            assert_eq!(c.first_stop_usage(), Some(&first));
        }
    }
}
