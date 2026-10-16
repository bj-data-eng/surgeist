use surgeist_html::*;
#[test]
fn document_body_roles_use_exact_html_names_and_admit_each_name_read() {
    for (namespace, local, expected) in [
        (Namespace::html(), "html", HtmlDocumentBodyRole::Html),
        (Namespace::html(), "body", HtmlDocumentBodyRole::Body),
        (Namespace::html(), "frameset", HtmlDocumentBodyRole::Body),
        (Namespace::html(), "BODY", HtmlDocumentBodyRole::Other),
        (Namespace::svg(), "body", HtmlDocumentBodyRole::Other),
        (Namespace::none(), "html", HtmlDocumentBodyRole::Other),
    ] {
        let name = QualifiedName::new(namespace, None, local);
        let mut work = 0;
        assert_eq!(
            document_body_role_with(&name, |_| {
                work += 1;
                Ok::<_, ()>(())
            })
            .unwrap(),
            expected
        );
        for cut in 0..work {
            let mut spent = 0;
            assert_eq!(
                document_body_role_with(&name, |_| {
                    if spent == cut {
                        return Err(cut);
                    }
                    spent += 1;
                    Ok(())
                }),
                Err(cut)
            );
            assert_eq!(spent, cut);
        }
    }
}
#[test]
fn explicit_blocking_processing_uses_ascii_space_tokens_and_stops_before_unadmitted_reads() {
    for (raw, expected) in [
        (None, HtmlBlockingTokens::Empty),
        (Some(""), HtmlBlockingTokens::Empty),
        (Some("other\tReNdEr\nother"), HtmlBlockingTokens::Render),
        (Some("rendering render\u{a0}"), HtmlBlockingTokens::Empty),
        (Some("\u{c}render\r"), HtmlBlockingTokens::Render),
        (Some("rеnder"), HtmlBlockingTokens::Empty),
    ] {
        let raw = raw.map(DocumentString::from);
        let mut work = 0;
        assert_eq!(
            blocking_tokens_with(raw.as_ref(), || {
                work += 1;
                Ok::<_, ()>(())
            })
            .unwrap(),
            expected
        );
        for cut in 0..work {
            let mut reached = 0;
            assert_eq!(
                blocking_tokens_with(raw.as_ref(), || {
                    if reached == cut {
                        return Err(cut);
                    }
                    reached += 1;
                    Ok(())
                }),
                Err(cut)
            );
            assert_eq!(reached, cut);
        }
    }
}
#[test]
fn potential_blocking_requires_actual_parser_document_join_and_document_body_gate() {
    for state in [
        HtmlStyleState::fresh(),
        HtmlStyleState::document_parser_created(),
    ] {
        for current in [false, true] {
            assert!(style_is_potentially_render_blocking(
                HtmlBlockingTokens::Render,
                state,
                current
            ));
            assert_eq!(
                style_is_potentially_render_blocking(HtmlBlockingTokens::Empty, state, current),
                state.created_by_document_parser() && current
            );
        }
    }
    assert!(document_allows_render_blocking(true, false));
    assert!(!document_allows_render_blocking(false, false));
    assert!(!document_allows_render_blocking(true, true));
    assert!(!document_allows_render_blocking(false, true));
}
#[test]
fn removal_precedes_disconnection_type_and_csp_guards() {
    for stop in 0..3 {
        let mut c = HtmlStyleUpdateCursor::new();
        assert_eq!(c.action(), HtmlStyleUpdateAction::RemoveAssociatedSheet);
        c.reply(HtmlStyleUpdateReply::Applied).unwrap();
        assert_eq!(c.action(), HtmlStyleUpdateAction::ReadConnected);
        c.reply(HtmlStyleUpdateReply::Connected(stop != 0)).unwrap();
        if stop > 0 {
            assert_eq!(c.action(), HtmlStyleUpdateAction::ReadType);
            c.reply(HtmlStyleUpdateReply::TypeEligible(stop != 1))
                .unwrap();
        }
        if stop > 1 {
            assert_eq!(c.action(), HtmlStyleUpdateAction::CheckInlineCsp);
            c.reply(HtmlStyleUpdateReply::InlineAllowed(false)).unwrap();
        }
        assert_eq!(c.action(), HtmlStyleUpdateAction::Complete);
    }
}
#[test]
fn creation_script_and_render_blocking_preserve_source_acknowledgement_order() {
    use HtmlStyleUpdateAction as A;
    use HtmlStyleUpdateReply as R;
    let mut c = HtmlStyleUpdateCursor::new();
    for (action, reply) in [
        (A::RemoveAssociatedSheet, R::Applied),
        (A::ReadConnected, R::Connected(true)),
        (A::ReadType, R::TypeEligible(true)),
        (A::CheckInlineCsp, R::InlineAllowed(true)),
        (A::CreateSheet, R::Applied),
        (A::ReadScriptBlocking, R::Blocking(true)),
        (A::RegisterScriptBlocking, R::Applied),
        (A::ReadRenderBlocking, R::Blocking(true)),
        (A::BlockRendering, R::Applied),
    ] {
        assert_eq!(c.action(), action);
        let before = c;
        assert!(c.reply(R::Connected(false)).is_err() || action == A::ReadConnected);
        c = before;
        c.reply(reply).unwrap();
    }
    assert_eq!(c.action(), A::Complete);
}
#[test]
fn type_and_actual_trigger_rules_do_not_parse_parameters_or_use_type_mutation_as_update() {
    for (value, expected) in [
        (None, true),
        (Some(""), true),
        (Some("TeXt/CsS"), true),
        (Some(" text/css"), false),
        (Some("text/css; charset=utf-8"), false),
    ] {
        let value = value.map(DocumentString::from);
        assert_eq!(
            style_type_allows_css_with(value.as_ref(), || Ok::<_, ()>(())).unwrap(),
            expected
        );
    }
    assert_eq!(
        style_attribute_effect(&Namespace::none(), KnownHtmlAttribute::Type),
        HtmlStyleAttributeEffect::None
    );
    assert_eq!(
        style_attribute_effect(&Namespace::none(), KnownHtmlAttribute::Media),
        HtmlStyleAttributeEffect::OwnerAttributes
    );
    assert_eq!(
        style_attribute_effect(&Namespace::from_uri("urn:test"), KnownHtmlAttribute::Title),
        HtmlStyleAttributeEffect::None
    );
    assert!(style_update_is_required(
        HtmlStyleTrigger::ChildrenChanged {
            on_parser_stack: false
        }
    ));
    assert!(!style_update_is_required(HtmlStyleTrigger::Insertion {
        connected: true,
        on_parser_stack: true
    }));
}
#[test]
fn networking_task_dispatch_precedes_script_removal_and_unblock() {
    use HtmlStyleNotificationAction as A;
    use HtmlStyleNotificationReply as R;
    for success in [true, false] {
        for script_blocking in [true, false] {
            let mut c = HtmlStyleNotificationCursor::new(success);
            assert_eq!(
                c.action(),
                if success {
                    A::DispatchLoad
                } else {
                    A::DispatchError
                }
            );
            let before = c;
            assert!(c.reply(R::ScriptBlocking(script_blocking)).is_err());
            assert_eq!(c, before);
            c.reply(R::Applied).unwrap();
            assert_eq!(c.action(), A::ReadScriptBlocking);
            c.reply(R::ScriptBlocking(script_blocking)).unwrap();
            if script_blocking {
                assert_eq!(c.action(), A::RemoveScriptBlocking);
                c.reply(R::Applied).unwrap();
            }
            assert_eq!(c.action(), A::UnblockRendering);
            c.reply(R::Applied).unwrap();
            assert_eq!(c.action(), A::Complete);
        }
    }
}

#[test]
fn actual_parser_creation_and_stack_transitions_are_distinct_from_fresh_clone_state() {
    use surgeist_html::{HtmlStyleParserStateError, HtmlStyleState};
    let fresh = HtmlStyleState::fresh();
    assert!(!fresh.created_by_document_parser());
    assert!(!fresh.on_parser_stack());
    let mut ordinary = fresh;
    assert_eq!(
        ordinary.parser_stack_added(),
        Err(HtmlStyleParserStateError)
    );
    assert_eq!(ordinary, fresh);
    let mut parsed = HtmlStyleState::document_parser_created();
    assert!(parsed.created_by_document_parser());
    assert!(
        !parsed.on_parser_stack(),
        "allocation is not an implicit stack push"
    );
    parsed.parser_stack_added().unwrap();
    assert!(parsed.on_parser_stack());
    assert!(!surgeist_html::style_update_is_required(
        surgeist_html::HtmlStyleTrigger::ChildrenChanged {
            on_parser_stack: parsed.on_parser_stack()
        }
    ));
    parsed.parser_stack_removed().unwrap();
    assert!(!parsed.on_parser_stack());
    assert!(
        parsed.created_by_document_parser(),
        "a pop preserves actual creation history"
    );
    assert!(
        !HtmlStyleState::fresh().created_by_document_parser(),
        "general cloning initializes a fresh Style rather than copying parser flags"
    );
}
