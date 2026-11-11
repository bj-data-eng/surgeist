use super::*;

#[test]
fn wrong_case_and_duplicate_token_attributes_do_not_bypass_expanded_name_validation() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let mut tag = Some(owned_tag(&mut c, "DIV", &[]));
    assert!(matches!(
        session.allocate_html_tag(&mut c, &mut tag),
        Err(SessionError::WrongToken)
    ));
    assert!(tag.is_some());
    let Tag {
        name, attributes, ..
    } = tag.take().unwrap();
    name.release(&mut c).unwrap();
    release_token_attributes(attributes, &mut c).unwrap();
    let mut tag = Some(owned_tag(&mut c, "div", &[("x", &[0x61]), ("x", &[0x62])]));
    assert!(!tag.as_ref().unwrap().had_duplicate_attributes);
    assert!(matches!(
        session.allocate_html_tag(&mut c, &mut tag),
        Err(SessionError::DuplicateAttribute)
    ));
    assert!(tag.is_none());
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        1
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn external_text_alias_rejects_without_consuming_owners() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut text = Some(owned_text(&mut c, &[0x61]));
    let alias = text.as_ref().unwrap().value.text().clone();
    assert!(matches!(
        session.insert_text(&mut c, &root, None, &mut text, None),
        Err(SessionError::AliasedPayload)
    ));
    assert!(text.is_some());
    drop(alias);
    session
        .insert_text(&mut c, &root, None, &mut text, None)
        .unwrap()
        .unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn foreign_token_owner_preserves_input_and_original_ledger_for_cleanup() {
    let mut c = control();
    let mut foreign = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let mut tag = Some(owned_tag(&mut foreign, "div", &[]));
    assert!(matches!(
        session.allocate_html_tag(&mut c, &mut tag),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert!(tag.is_some());
    let Tag {
        name, attributes, ..
    } = tag.take().unwrap();
    name.release(&mut foreign).unwrap();
    release_token_attributes(attributes, &mut foreign).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    storage_zero(&foreign);
}
#[test]
fn managed_string_cap_stops_the_append_and_releases_the_token_string() {
    let caps = limits()
        .into_builder()
        .managed_utf16_units(1)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut first = Some(owned_text(&mut c, &[0x61]));
    let first = session
        .insert_text(&mut c, &root, None, &mut first, None)
        .unwrap()
        .unwrap();
    let mut second = Some(owned_text(&mut c, &[0x62]));
    let error = session.insert_text(&mut c, &root, None, &mut second, None);
    assert!(matches!(
        error,
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedUtf16Units,
            limit: 1,
            current: 1,
            requested_delta: 1
        }))
    ));
    assert_eq!(native_text(&session, &first._node), [0x61]);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        1
    );
    // The token's string is released with the refusal.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    let stop = c.stop();
    session.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    storage_zero(&c);
}
