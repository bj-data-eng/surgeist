use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    values::DocumentString,
};

#[test]
fn template_real_document_pair_node_cap_rejects_before_lexical_take_and_disposes_all_owners() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // The ordinary root already exists. Creation needs one actual inert
    // Document, the template Element and its hosted Fragment: three more.
    for cap in [3, 4] {
        let source = crate::HtmlSource::utf8("<template>");
        let mut c = ParseControl::new(
            limits().into_builder().managed_nodes(cap).finish().unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
            panic!("actual template")
        };
        let mut input = Some(tag);
        let result = session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &root,
            None,
            CreationContext::Ordinary,
        );
        if cap == 4 {
            let template = result.unwrap();
            assert!(input.is_none());
            let state = session.template_state(&mut c, &template).unwrap();
            let inert = session
                .node_document(&mut c, &state.contents)
                .unwrap()
                .node(&mut c, &session)
                .unwrap();
            assert_ne!(inert.index, root.index);
            assert_eq!(
                session.active().unwrap().construction.test_storage().len(),
                4
            );
            assert!(state.target.is_none() && state._start.is_none() && state.end.is_none());
        } else {
            let Err(SessionError::Stop(first)) = result else {
                panic!("real node cap")
            };
            assert!(matches!(
                first,
                HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedNodes,
                    limit: 3,
                    requested_delta: 3,
                    ..
                }
            ));
            assert!(input.is_some());
            assert_eq!(
                session.active().unwrap().construction.test_storage().len(),
                1
            );
            assert!(
                !session
                    .active()
                    .unwrap()
                    .payloads
                    .values
                    .iter()
                    .any(|entry| matches!(entry, RetainedPayload::Template { .. }))
            );
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
        }
        if let Some(tag) = input {
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn template_document_and_facts_slots_have_independent_pretransfer_and_installed_cap_boundaries() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Root record/self-document/cache facts =3 plus its append index =4.
    // Fresh template adds three native records, five document/template facts,
    // name/namespace owners, two nonce slots, three append indices
    // and one creation row: completed20. Document/native preflight needs11
    // above the root's4 before lexical take. At19 the tag is transferred but
    // admission of the remaining records fails after the inert Document.
    for cap in [14, 19, 20] {
        let source = crate::HtmlSource::utf8("<template>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .auxiliary_entries(cap)
                .active_formatting_entries(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
            panic!("actual template")
        };
        let mut input = Some(tag);
        let result = session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &root,
            None,
            CreationContext::Ordinary,
        );
        if cap == 20 {
            let node = result.unwrap();
            assert!(input.is_none());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                20
            );
            assert_eq!(
                session.active().unwrap().construction.test_storage().len(),
                4
            );
            assert!(
                session
                    .template_state(&mut c, &node)
                    .unwrap()
                    .target
                    .is_none()
            );
        } else {
            let Err(SessionError::Stop(first)) = result else {
                panic!("actual initialized slot cap")
            };
            assert!(
                matches!(first, HtmlParseStop::LimitExceeded{resource: HtmlParseResource::AuxiliaryEntries,limit,..} if limit==cap)
            );
            assert_eq!(input.is_some(), cap == 14);
            assert_eq!(
                session.active().unwrap().construction.test_storage().len(),
                if cap == 14 { 1 } else { 2 }
            );
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            assert_eq!(c.stop(), Some(first));
        }
        if let Some(tag) = input {
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn template_pi_map_coexists_with_original_fields_at_exact_managed_units_and_reclaims_stopped_pair()
{
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Original marker target6 and data name='A'8 remain native-owned;
    // the independently decoded name4/value1 add5 managed units.
    for cap in [18, 19] {
        let source = crate::HtmlSource::utf8("<?marker name='A'?>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .managed_utf16_units(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c)
        else {
            panic!("real PI")
        };
        let mut input = Some((target, data));
        let result = session.allocate_processing_instruction(&mut c, &mut input, None);
        assert!(input.is_none());
        assert_eq!(
            session.active().unwrap().construction.test_storage().len(),
            2
        );
        if cap == 19 {
            let node = result.unwrap();
            assert_eq!(
                session
                    .active()
                    .unwrap()
                    .pi_attribute(&mut c, node.index, "name")
                    .unwrap()
                    .unwrap()
                    .code_units(),
                [65]
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                19
            );
        } else {
            let Err(SessionError::Stop(first)) = result else {
                panic!("decoded value coexistence")
            };
            assert!(matches!(
                first,
                HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedUtf16Units,
                    current: 18,
                    requested_delta: 1,
                    ..
                }
            ));
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            let ConstructionNodeKind::ProcessingInstruction(pi) = &session
                .active()
                .unwrap()
                .construction
                .node_at(1)
                .expect("retained construction position")
                .kind()
            else {
                panic!("original PI survives")
            };
            assert_eq!(pi.target().code_units(), [109, 97, 114, 107, 101, 114]);
            assert_eq!(pi.data().code_units(), [110, 97, 109, 101, 61, 39, 65, 39]);
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn template_cross_document_cancel_retains_completed_removal_before_any_adoption_binding() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head><script src=x><template><div>"),
        limits(),
    );
    let (mut c, mut session, _proof, script) = fresh_script(&request, false, false);
    let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
    let html = session.active().unwrap().node(&mut c, 1).unwrap();
    session.insert(&mut c, &div, &script, None, None).unwrap();
    session.insert(&mut c, &html, &div, None, None).unwrap();
    let old = session
        .node_document(&mut c, &div)
        .unwrap()
        .node(&mut c, &session)
        .unwrap();
    let new = session
        .node_document(&mut c, &contents)
        .unwrap()
        .node(&mut c, &session)
        .unwrap();
    assert_ne!(old.index, new.index);
    let metadata = session.script_metadata(&mut c, &script).unwrap();
    session.active_mut().unwrap().cancel_after_removal = Some(request.cancellation().clone());
    assert_eq!(
        session.insert(&mut c, &contents, &div, None, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let storage = session.active().unwrap();
    assert_eq!(
        storage
            .construction
            .node_at(div.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(contents.index)
    );
    assert!(
        !storage
            .construction
            .node_at(html.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>()
            .contains(&div.index)
    );
    assert_eq!(
        storage
            .construction
            .node_at(script.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(div.index)
    );
    assert!(matches!(
        storage.effects.form,
        Some(FormEffects {
            stage: FormEffectStage::Adoption,
            ..
        })
    ));
    for node in [div.index, script.index] {
        assert!(storage.payloads.values.iter().any(|p|matches!(p,RetainedPayload::NodeDocument{node:Some(bound),document} if *bound==node && *document==DocumentRef::Owned(old.index))));
    }
    assert!(storage.payloads.values.iter().any(|p|matches!(p,RetainedPayload::Script{node:Some(node),metadata:retained} if *node==script.index && *retained==metadata)));
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
}
#[test]
fn template_pi_every_bounded_allocation_and_decode_prefix_keeps_original_and_map_tickets_owned() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Sweep each credit in this finite operation, rather than deriving a
    // success total from it. Independent witnesses require an installed
    // original PI, a partially decoded map, and the complete literal map.
    let mut installed = false;
    let mut partial = false;
    let mut completed = false;
    // The sole evidence identity, origin/group/cause vectors and decoded
    // field groups extend the original prefix interval. Keep every old
    // credit and sweep the extended finite interval without skipping cuts.
    for remaining in 0..=131_072 {
        let source = crate::HtmlSource::utf8("<?marker name='A' x='B'?>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c)
        else {
            panic!("real lexical PI")
        };
        let mut input = Some((target, data));
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - used - remaining,
        )
        .unwrap();
        let result = session.allocate_processing_instruction(&mut c, &mut input, None);
        let storage = session.active().unwrap();
        if storage.construction.test_storage().len() == 2 {
            installed = true;
            let ConstructionNodeKind::ProcessingInstruction(pi) = &storage
                .construction
                .node_at(1)
                .expect("retained construction position")
                .kind()
            else {
                panic!("installed PI")
            };
            assert_eq!(
                pi.target().code_units(),
                DocumentString::from("marker").code_units()
            );
            assert_eq!(
                pi.data().code_units(),
                DocumentString::from("name='A' x='B'").code_units()
            );
            partial |= storage.payloads.values.iter().any(|p| matches!(p, RetainedPayload::PiAttributes { values, ready: false, .. } if values.len() != 0));
        }
        match result {
            Ok(node) => {
                completed = true;
                let values = storage
                    .payloads
                    .values
                    .iter()
                    .find_map(|p| match p {
                        RetainedPayload::PiAttributes {
                            node: Some(bound),
                            values,
                            ready: true,
                        } if *bound == node.index => Some(values.as_slice()),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(values.len(), 2);
                assert_eq!(values[0].name.code_units(), [110, 97, 109, 101]);
                assert_eq!(values[0].value.code_units(), [65]);
                assert_eq!(values[1].name.code_units(), [120]);
                assert_eq!(values[1].value.code_units(), [66]);
            }
            Err(SessionError::Stop(first)) => {
                assert!(matches!(
                    first,
                    HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                assert_eq!(
                    session.freeze(&mut c).err(),
                    Some(SessionError::Stop(first))
                );
                assert_eq!(c.stop(), Some(first));
            }
            other => panic!("unexpected prefix result {other:?}"),
        }
        let first = c.stop();
        if let Some((target, data)) = input {
            target.release(&mut c).unwrap();
            data.release(&mut c).unwrap();
        }
        lexer.release(&mut c).unwrap();
        match session.release(&mut c) {
            Ok(()) => {}
            Err(SessionError::Stop(stop)) => {
                assert_eq!(c.stop(), Some(stop));
                session.release(&mut c).unwrap();
            }
            other => panic!("cleanup failed: {other:?}"),
        }
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            0,
            "remaining={remaining}"
        );
        storage_zero(&c);
        if let Some(first) = first {
            assert_eq!(c.stop(), Some(first));
        }
    }
    assert!(
        installed && partial && completed,
        "installed={installed}, partial={partial}, completed={completed}"
    );
}
#[test]
fn shared_pi_map_literal_xml_closure_keeps_native_fields_and_reclaims_real_owners() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let cases: &[(&str, &[(&str, &str)])] = &[
        ("name='X'", &[("name", "X")]),
        ("name = \"&#88;\" x='ok' ", &[("name", "X"), ("x", "ok")]),
        ("name='&amp;&lt;&gt;&quot;&apos;'", &[("name", "&<>\"'")]),
        ("name='&#x1D11E;'", &[("name", "𝄞")]),
        (
            "name='&#0000000000000000000000000000000000000088;'",
            &[("name", "X")],
        ),
        ("𐀀='v' name='X'", &[("𐀀", "v"), ("name", "X")]),
        ("a='A' A='B'", &[("a", "A"), ("A", "B")]),
        ("a='A' a='B'", &[]),
        ("name='X' x='1' x='2'", &[]),
        ("name='X'junk", &[]),
        ("name='X'x='Y'", &[]),
        ("name='X'\u{000c}x='Y'", &[]),
        ("name='&unknown;'", &[]),
        ("name='&#0;'", &[]),
        ("name='&#xD800;'", &[]),
        ("name='&#x110000;'", &[]),
        ("name='&#999999999999999999999999999999;'", &[]),
        ("name='&#xFFFFFFFFFFFFFFFFFFFFFFFFFFFF;'", &[]),
        ("name='<'", &[]),
        ("name='unterminated", &[]),
        ("0bad='X'", &[]),
    ];
    for &(data, expected) in cases {
        let source = crate::HtmlSource::utf8(format!("<?marker {data}?>"));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::ProcessingInstruction { target, data: raw } =
            lexical_kind(&mut lexer, &mut c)
        else {
            panic!("real PI")
        };
        let node = session
            .allocate_processing_instruction(&mut c, &mut Some((target, raw)), None)
            .unwrap();
        let storage = session.active().unwrap();
        let ConstructionNodeKind::ProcessingInstruction(pi) = &storage
            .construction
            .node_at(node.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("original PI")
        };
        assert_eq!(pi.target().code_units(), [109, 97, 114, 107, 101, 114]);
        assert_eq!(
            pi.data().code_units(),
            DocumentString::from(data).code_units()
        );
        let fields = storage
            .payloads
            .values
            .iter()
            .find_map(|p| match p {
                RetainedPayload::PiAttributes {
                    node: Some(bound),
                    values,
                    ready: true,
                } if *bound == node.index => Some(values.as_slice()),
                _ => None,
            })
            .unwrap();
        assert_eq!(fields.len(), expected.len(), "{data}");
        for (field, (name, value)) in fields.iter().zip(expected) {
            assert_eq!(
                field.name.code_units(),
                DocumentString::from(*name).code_units(),
                "{data}"
            );
            assert_eq!(
                field.value.code_units(),
                DocumentString::from(*value).code_units(),
                "{data}"
            );
        }
        if data == "name='&#x1D11E;'" {
            let group = storage
                .payloads
                .values
                .iter()
                .find_map(|p| match p {
                    RetainedPayload::String {
                        value,
                        origins: Some(group),
                        ..
                    } if value.code_units() == [0xd834, 0xdd1e] => Some(group),
                    _ => None,
                })
                .unwrap();
            let rows = storage
                .evidence
                .as_ref()
                .unwrap()
                .observed_origins(group, &mut c)
                .unwrap();
            assert_eq!(
                rows.iter()
                    .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                    .collect::<Vec<_>>(),
                [(0, 2, 15, 24)]
            );
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
