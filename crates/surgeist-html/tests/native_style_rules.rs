use surgeist_html::*;
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
