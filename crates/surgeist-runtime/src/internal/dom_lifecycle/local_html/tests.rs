use super::super::constructor_settings_tests::{limits, session};
use super::*;
use surgeist_dom::hosted::{DomHostedError, DomHostedLimits};

#[test]
fn local_html_top_level_returns_the_active_empty_document() {
    use surgeist_dom::document::DomDocumentCreationCause;
    use surgeist_html::construction::HtmlDocumentMode;

    let cancel = AtomicBool::new(false);
    let mut session = session(limits());
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    let top = hosted
        .create_local_html_top_level(&mut dom, host, &cancel)
        .unwrap();

    let snapshot = dom.snapshot();
    let document = snapshot.document(top.document()).unwrap();
    assert!(document.is_fully_active().unwrap());
    assert!(matches!(
        document.metadata().creation_cause(),
        DomDocumentCreationCause::HostedLocalHtml { source, .. }
            if *source == top.host().source()
    ));
    assert_eq!(document.metadata().mode(), HtmlDocumentMode::NoQuirks);
    assert!(
        snapshot
            .node(top.document().node())
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(
        snapshot.global_document(top.global()).unwrap(),
        top.document()
    );
    hosted
        .with_current_top_level(&mut dom, top, &cancel, |source, dom| {
            source
                .qualify_hosted_local_html_parser(dom.raw(), top.document(), top.global(), &cancel)
                .map_err(DomHostedRuntimeError::Dom)
        })
        .unwrap();
}

fn local_html_limits() -> surgeist_html_parser::HtmlParseLimits {
    surgeist_html_parser::HtmlParseLimits::builder()
        .source_units(128)
        .buffered_utf16_units(1024)
        .token_attributes(8)
        .auxiliary_entries(256)
        .managed_nodes(64)
        .managed_edges(64)
        .managed_attributes(16)
        .managed_utf16_units(1024)
        .source_map_entries(0)
        .source_map_payload_units(0)
        .diagnostics(0)
        .diagnostic_payload_units(0)
        .context_effects(0)
        .open_elements(16)
        .active_formatting_entries(0)
        .tree_depth(16)
        .managed_capacity_bytes(1_048_576)
        .peak_transient_bytes(1_048_576)
        .atomic_layout_bytes(65_536)
        .work_credits(1_048_576)
        .finish()
        .unwrap()
}

#[test]
fn classic_script_pause_accepts_inert_guest_edits_and_refuses_dynamic_script() {
    use surgeist_dom::{
        DomBatch,
        script::{DomDocumentScriptRegistration, DomDocumentScriptingFacts},
    };
    use surgeist_html::{
        scripting::{HtmlScriptingEngine, HtmlScriptingPolicy},
        values::DocumentString,
    };

    let html = HtmlSource::utf8(
        "<!doctype html><meta charset=utf-8><div id=before></div><script>x()</script><div id=after></div>",
    );
    let cancel = AtomicBool::new(false);
    for dynamic_script in [false, true] {
        let mut session = session(limits());
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let limits = dom.raw().limits().with_meta_processing_work(1_024);
        dom.raw_mut().set_limits(limits).unwrap();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        let top = hosted
            .create_local_html_top_level(&mut dom, host, &cancel)
            .unwrap();
        let mut parser = hosted
            .begin_local_classic_html_parser(&mut dom, top, &html, local_html_limits(), &cancel)
            .unwrap();
        parser.accept_eof();
        let limit = NonZeroUsize::new(128).unwrap();

        assert!(matches!(
            parser.advance(&hosted, &mut dom, limit, &cancel),
            Err(DomHostedHtmlError::Parse(HtmlLiveParseError::Host(
                DomError::ScriptPreparationRequired
            )))
        ));
        let (_, _registration) = hosted
            .with_current_top_level(&mut dom, top, &cancel, |source, dom| {
                let backing = source
                    .retain_runtime_payload(
                        dom.raw_mut(),
                        DomDocumentScriptRegistration::storage_bytes(),
                        &cancel,
                    )
                    .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
                source
                    .attach_document_script_host(
                        dom.raw_mut(),
                        top.global(),
                        top.document(),
                        DomDocumentScriptingFacts {
                            engine: HtmlScriptingEngine::Supported,
                            policy: HtmlScriptingPolicy {
                                user_allows: true,
                                automation_allows: true,
                            },
                        },
                        backing,
                        &cancel,
                    )
                    .map_err(DomHostedRuntimeError::Dom)
            })
            .unwrap();

        assert_eq!(
            parser.advance(&hosted, &mut dom, limit, &cancel).unwrap(),
            HtmlLiveParseProgress::Script
        );
        let script = parser.pending_script().unwrap();
        let paused = dom.snapshot();
        let script_node = paused.node(script).unwrap();
        let body = script_node.parent().unwrap();
        let canonical = script_node.as_html_script().unwrap();
        assert_eq!(canonical.parser_document(), Some(top.document()));
        assert_eq!(canonical.preparation_document(), Some(top.document()));
        assert!(canonical.flags().already_started());
        assert_eq!(script_node.children().len(), 1);
        let preceding = paused.node(body).unwrap().children()[0];
        assert_eq!(
            paused
                .node(preceding)
                .unwrap()
                .attribute(&DocumentString::from("id")),
            Some(&DocumentString::from("before"))
        );
        assert_eq!(paused.node(body).unwrap().children(), [preceding, script]);
        assert!(
            parser
                .take_stop_parsing(&hosted, &mut dom, &cancel)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            parser.complete_script(&hosted, &mut dom, preceding, &cancel),
            Err(DomHostedHtmlError::UnexpectedScript)
        ));

        if dynamic_script {
            let mut edit = DomBatch::new();
            let ticket = edit.create_element(top.document(), "script");
            edit.set_attribute(&ticket, "src", "/unplanned.js");
            let node = dom
                .apply(edit, &cancel)
                .unwrap()
                .created(&ticket)
                .unwrap()
                .unwrap();
            let mut edit = DomBatch::new();
            edit.destroy(node);
            dom.apply(edit, &cancel).unwrap();
            assert!(dom.snapshot().node(node).is_err());
        } else {
            let mut edit = DomBatch::new();
            let ticket = edit.create_element(top.document(), "p");
            let paragraph = dom
                .apply(edit, &cancel)
                .unwrap()
                .created(&ticket)
                .unwrap()
                .unwrap();
            let mut edit = DomBatch::new();
            edit.set_attribute(paragraph, "id", "named");
            edit.insert_before(paragraph, body, None);
            dom.apply(edit, &cancel).unwrap();
            let mut edit = DomBatch::new();
            edit.detach(paragraph);
            dom.apply(edit, &cancel).unwrap();
        }

        assert_eq!(
            parser.advance(&hosted, &mut dom, limit, &cancel).unwrap(),
            HtmlLiveParseProgress::Script
        );
        if dynamic_script {
            assert!(matches!(
                parser.complete_script(&hosted, &mut dom, script, &cancel),
                Err(DomHostedHtmlError::LoadProfileChanged)
            ));
            assert!(
                parser
                    .take_stop_parsing(&hosted, &mut dom, &cancel)
                    .unwrap()
                    .is_none()
            );
            parser.abort().unwrap();
            continue;
        }
        parser
            .complete_script(&hosted, &mut dom, script, &cancel)
            .unwrap();
        assert_eq!(
            parser.advance(&hosted, &mut dom, limit, &cancel).unwrap(),
            HtmlLiveParseProgress::StopParsing
        );
        let finished = dom.snapshot();
        let children = finished.node(body).unwrap().children();
        assert_eq!(&children[..2], [preceding, script]);
        assert_eq!(
            finished
                .node(children[2])
                .unwrap()
                .attribute(&DocumentString::from("id")),
            Some(&DocumentString::from("after"))
        );
        let stop = parser
            .take_stop_parsing(&hosted, &mut dom, &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(stop.parser_continuation().eof().source(), html.identity());
        assert!(stop.parser_continuation().parser_scripts_are_absent());
        assert_eq!(stop.load_checkpoint(), finished.stamp());
        assert!(
            parser
                .take_stop_parsing(&hosted, &mut dom, &cancel)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn local_html_prefix_and_later_text_use_the_current_canonical_node() {
    use std::num::NonZeroUsize;
    use surgeist_dom::{DomBatch, tree::DomNodeKind};
    use surgeist_html_parser::HtmlSource;

    let source = HtmlSource::utf16(
        "<!doctype html><html><head></head><body><div>AB</div></body></html>".into(),
    );
    let parser_limits = local_html_limits();
    let cancel = AtomicBool::new(false);
    let mut session = session(limits());
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    let top = hosted
        .create_local_html_top_level(&mut dom, host, &cancel)
        .unwrap();
    let mut parser = hosted
        .begin_local_html_parser(&mut dom, top, &source, parser_limits, &cancel)
        .unwrap();
    assert!(matches!(
        hosted.begin_local_html_parser(&mut dom, top, &source, parser_limits, &cancel),
        Err(DomHostedHtmlError::ParserAlreadyClaimed)
    ));

    let mut prefix = None;
    for _ in 0..32 {
        parser
            .advance(&hosted, &mut dom, NonZeroUsize::MIN, &cancel)
            .unwrap();
        let snapshot = dom.snapshot();
        prefix = snapshot.nodes().find(|node| {
            matches!(
                snapshot.node(*node).unwrap().kind(),
                DomNodeKind::Text(data) if data.code_units() == [u16::from(b'A')]
            )
        });
        if prefix.is_some() {
            break;
        }
    }
    let text = prefix.expect("the first source character is visible in the active Document");
    let mut edit = DomBatch::new();
    edit.replace_data(text, 0, 1, "X");
    dom.apply(edit, &cancel).unwrap();
    parser
        .advance(&hosted, &mut dom, NonZeroUsize::MIN, &cancel)
        .unwrap();

    let snapshot = dom.snapshot();
    let node = snapshot.node(text).unwrap();
    assert_eq!(node.owner_document(), Some(top.document()));
    let DomNodeKind::Text(data) = node.kind() else {
        panic!("the original node remains Text");
    };
    assert_eq!(data.code_units(), [u16::from(b'X'), u16::from(b'B')]);
    assert_eq!(
        snapshot.global_document(top.global()).unwrap(),
        top.document()
    );
    parser.accept_eof();
    assert_eq!(
        parser
            .advance(&hosted, &mut dom, NonZeroUsize::new(32).unwrap(), &cancel)
            .unwrap(),
        surgeist_html_parser::live::HtmlLiveParseProgress::StopParsing
    );
    assert!(matches!(
        parser.take_stop_parsing(&hosted, &mut dom, &cancel),
        Err(DomHostedHtmlError::LoadProfileChanged)
    ));
    assert!(matches!(
        parser.first_refusal(),
        Some(DomHostedHtmlError::LoadProfileChanged)
    ));
}

#[test]
fn local_html_handoff_refuses_foreign_aborted_and_retiring_progress() {
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use surgeist_dom::{
        document::DomNativeInputPlatform,
        installation::{DomParsedDocumentEnvironment, DomParsedHtmlInstallLimits},
    };
    use surgeist_html_parser::{
        HtmlDocumentParseRequest, HtmlSource,
        live::{HtmlLiveParseError, HtmlLiveParseProgress},
    };
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme,
    };

    let source = HtmlSource::utf8("<!doctype html><html><head></head><body></body></html>");
    let cancel = AtomicBool::new(false);
    let foreign_session = session(limits());
    let foreign_hosted = foreign_session.hosted_execution();
    let mut session = session(limits());
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    let blank = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
    assert!(matches!(
        hosted.begin_local_html_parser(&mut dom, blank, &source, local_html_limits(), &cancel),
        Err(DomHostedHtmlError::Hosted(DomHostedRuntimeError::Dom(
            DomHostedError::UnsupportedProfile
        )))
    ));
    let basis = dom.snapshot();
    let origin = basis
        .document(blank.document())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
        "https://example.test/offline".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        origin,
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    let offline_limits = local_html_limits()
        .into_builder()
        .atomic_layout_bytes(131_072)
        .finish()
        .unwrap();
    let report = HtmlDocumentParseRequest::offline(source.clone(), offline_limits)
        .parse()
        .unwrap();
    let offline = dom
        .raw_mut()
        .install_parsed_html_document(
            basis.stamp(),
            Arc::new(report),
            environment,
            DomParsedHtmlInstallLimits::new(64, 1_048_576, 1_048_576, 1_048_576),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        hosted.with_current_top_level(&mut dom, blank, &cancel, |source, dom| {
            source
                .qualify_hosted_local_html_parser(
                    dom.raw(),
                    offline.document(),
                    blank.global(),
                    &cancel,
                )
                .map_err(DomHostedRuntimeError::Dom)
        }),
        Err(DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity))
    ));

    for disposition in ["retry", "abort", "retire"] {
        let top = hosted
            .create_local_html_top_level(&mut dom, host, &cancel)
            .unwrap();
        let mut parser = hosted
            .begin_local_html_parser(&mut dom, top, &source, local_html_limits(), &cancel)
            .unwrap();
        parser.accept_eof();
        assert_eq!(
            parser
                .advance(&hosted, &mut dom, NonZeroUsize::new(32).unwrap(), &cancel)
                .unwrap(),
            HtmlLiveParseProgress::StopParsing
        );
        match disposition {
            "retry" => {
                assert!(matches!(
                    parser.take_stop_parsing(&foreign_hosted, &mut dom, &cancel),
                    Err(DomHostedHtmlError::Hosted(DomHostedRuntimeError::Binding))
                ));
                let receipt = parser
                    .take_stop_parsing(&hosted, &mut dom, &cancel)
                    .unwrap()
                    .expect("retry claims the original continuation");
                assert_eq!(receipt.top_level(), top);
                assert_eq!(
                    receipt.parser_continuation().eof().source(),
                    source.identity()
                );
                assert!(matches!(
                    parser.first_refusal(),
                    Some(DomHostedHtmlError::Hosted(DomHostedRuntimeError::Binding))
                ));
            }
            "abort" => {
                parser.abort().unwrap();
                assert!(matches!(
                    parser.take_stop_parsing(&hosted, &mut dom, &cancel),
                    Err(DomHostedHtmlError::Start(HtmlLiveParseError::Aborted))
                ));
            }
            "retire" => {
                hosted
                    .destroy_top_level(&mut dom, top.navigable(), &cancel)
                    .unwrap();
                assert!(matches!(
                    parser.take_stop_parsing(&hosted, &mut dom, &cancel),
                    Err(DomHostedHtmlError::Hosted(_))
                ));
                parser.abort().unwrap();
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn local_html_reports_reached_unsupported_work_after_a_foreign_refusal() {
    use std::num::NonZeroUsize;
    use surgeist_html_parser::{
        HtmlSource,
        live::{HtmlLiveParseError, HtmlLiveUnsupported},
    };

    let prefix = "<!doctype html><html><head></head><body>";
    let source = HtmlSource::utf8(format!("{prefix}<script></script>"));
    let cancel = AtomicBool::new(false);
    let foreign = session(limits());
    let mut session = session(limits());
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    let top = hosted
        .create_local_html_top_level(&mut dom, host, &cancel)
        .unwrap();
    let mut parser = hosted
        .begin_local_html_parser(&mut dom, top, &source, local_html_limits(), &cancel)
        .unwrap();
    let tokens = NonZeroUsize::new(32).unwrap();

    assert!(matches!(
        parser.advance(&foreign.hosted_execution(), &mut dom, tokens, &cancel),
        Err(DomHostedHtmlError::Hosted(DomHostedRuntimeError::Binding))
    ));
    for _ in 0..2 {
        assert!(matches!(
            parser.advance(&hosted, &mut dom, tokens, &cancel),
            Err(DomHostedHtmlError::Parse(HtmlLiveParseError::Unsupported {
                reason: HtmlLiveUnsupported::Element,
                source: range,
            })) if range.source() == source.identity() && range.start() == prefix.len()
        ));
    }
    assert!(matches!(
        parser.first_refusal(),
        Some(DomHostedHtmlError::Hosted(DomHostedRuntimeError::Binding))
    ));
}

#[test]
fn local_html_admits_the_canonical_text_copy_before_publication() {
    use std::num::NonZeroUsize;
    use surgeist_dom::{DomBatch, DomError, tree::DomNodeKind};
    use surgeist_html_parser::{HtmlSource, live::HtmlLiveParseError};

    const WORK: u64 = 25_000;
    let source = HtmlSource::utf8("<!doctype html><html><head></head><body>AB</body></html>");
    let cancel = AtomicBool::new(false);
    let mut policy = limits();
    policy.canonical = DomHostedLimits::new(
        policy.canonical.records(),
        policy.canonical.retained_bytes(),
        policy.canonical.scratch_bytes(),
        WORK,
    );
    let mut session = session(policy);
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    let top = hosted
        .create_local_html_top_level(&mut dom, host, &cancel)
        .unwrap();
    let mut parser = hosted
        .begin_local_html_parser(&mut dom, top, &source, local_html_limits(), &cancel)
        .unwrap();
    parser
        .advance(&hosted, &mut dom, NonZeroUsize::new(6).unwrap(), &cancel)
        .unwrap();
    let snapshot = dom.snapshot();
    let text = snapshot.nodes().find(|&node| {
        matches!(snapshot.node(node).unwrap().kind(), DomNodeKind::Text(data) if data.code_units() == [u16::from(b'A')])
    }).expect("the first Text token was published");
    let mut edit = DomBatch::new();
    edit.replace_data(text, 0, 1, "X".repeat(50_000));
    dom.apply(edit, &cancel).unwrap();
    let before = dom.snapshot();

    for _ in 0..2 {
        assert!(matches!(
            parser.advance(&hosted, &mut dom, NonZeroUsize::MIN, &cancel),
            Err(DomHostedHtmlError::Parse(HtmlLiveParseError::Host(
                DomError::SourceWorkLimit { limit: WORK }
            )))
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    assert!(
        matches!(before.node(text).unwrap().kind(), DomNodeKind::Text(data) if data.len() == 50_000)
    );
    parser.abort().unwrap();
}
