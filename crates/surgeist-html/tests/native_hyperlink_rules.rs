use surgeist_html::*;

#[test]
fn set_url_clears_before_reading_and_only_stores_success() {
    use HtmlHyperlinkSetUrlAction as A;
    use HtmlHyperlinkSetUrlReply as R;
    for (present, parsed, expected) in [
        (false, false, vec![A::ClearUrl, A::ReadHref, A::Complete]),
        (
            true,
            false,
            vec![A::ClearUrl, A::ReadHref, A::ParseHref, A::Complete],
        ),
        (
            true,
            true,
            vec![
                A::ClearUrl,
                A::ReadHref,
                A::ParseHref,
                A::StoreParsedUrl,
                A::Complete,
            ],
        ),
    ] {
        let mut cursor = HtmlHyperlinkSetUrlCursor::new();
        let mut actual = vec![cursor.action()];
        cursor.reply(R::Applied).unwrap();
        actual.push(cursor.action());
        cursor.reply(R::HrefPresent(present)).unwrap();
        actual.push(cursor.action());
        if present {
            cursor.reply(R::Parsed(parsed)).unwrap();
            actual.push(cursor.action());
            if parsed {
                cursor.reply(R::Applied).unwrap();
                actual.push(cursor.action());
            }
        }
        assert_eq!(actual, expected);
        assert_eq!(cursor.reply(R::Applied), Err(HtmlHyperlinkSetUrlError));
        assert_eq!(cursor.action(), A::Complete);
    }
}
#[test]
fn protocol_refusal_does_not_skip_the_clear_or_parse() {
    use HtmlHyperlinkSetUrlAction as A;
    use HtmlHyperlinkSetUrlReply as R;
    let mut cursor = HtmlHyperlinkSetUrlCursor::default();
    assert!(cursor.reply(R::Parsed(true)).is_err());
    assert_eq!(cursor.action(), A::ClearUrl);
    cursor.reply(R::Applied).unwrap();
    assert!(cursor.reply(R::Applied).is_err());
    assert_eq!(cursor.action(), A::ReadHref);
    // Present empty href follows this same present branch; no value parsing here.
    cursor.reply(R::HrefPresent(true)).unwrap();
    assert!(cursor.reply(R::Applied).is_err());
    assert_eq!(cursor.action(), A::ParseHref);
}
#[test]
fn changed_href_precedes_speculation_but_namespace_and_exact_equality_return() {
    use HtmlHyperlinkAttributeEffect as E;
    for attribute in [
        KnownHtmlAttribute::Href,
        KnownHtmlAttribute::Rel,
        KnownHtmlAttribute::Referrerpolicy,
    ] {
        assert_eq!(
            hyperlink_attribute_effect(&Namespace::none(), attribute, true),
            E::None
        );
        assert_eq!(
            hyperlink_attribute_effect(&Namespace::html(), attribute, false),
            E::None
        );
    }
    assert_eq!(
        hyperlink_attribute_effect(&Namespace::none(), KnownHtmlAttribute::Href, false),
        E::SetUrlThenConsiderSpeculativeLoads
    );
    for attribute in [KnownHtmlAttribute::Rel, KnownHtmlAttribute::Referrerpolicy] {
        assert_eq!(
            hyperlink_attribute_effect(&Namespace::none(), attribute, false),
            E::ConsiderSpeculativeLoads
        );
    }
    assert_eq!(
        hyperlink_attribute_effect(&Namespace::none(), KnownHtmlAttribute::Name, false),
        E::None
    );
}
#[test]
fn tree_guards_use_connection_of_the_source_required_subject() {
    use HtmlHyperlinkTreeMutation as M;
    for connected in [false, true] {
        assert_eq!(
            hyperlink_tree_considers_speculative_loads(M::Insertion { connected }),
            connected
        );
        assert_eq!(
            hyperlink_tree_considers_speculative_loads(M::Removal {
                old_ancestor_connected: connected
            }),
            connected
        );
    }
    assert!(hyperlink_tree_considers_speculative_loads(M::Moving));
}
