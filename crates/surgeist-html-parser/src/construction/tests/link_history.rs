//! Actual canonical link change and clone paths, independent literal expectations.
use super::*;
use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};
use surgeist_html::construction::ConstructionNodeKind;

fn next_tag(lexer: &mut LexerFixture<'_>, c: &mut ParseControl) -> Tag {
    loop {
        let event = lexer.next(c, false).unwrap().unwrap();
        match event {
            Event::Token(event) => {
                let token = event.into_token(c).unwrap();
                if let TokenKind::Tag(tag) = token.kind {
                    return tag;
                }
                release_token_kind(&mut Some(token.kind), c).unwrap();
            }
            event => event.release(c).unwrap(),
        }
    }
}
fn flags(
    session: &ParserArenaSession,
    node: &ParserNode,
) -> surgeist_html::elements::link::HtmlLinkFlags {
    let ConstructionNodeKind::Element(e) = &session
        .active()
        .unwrap()
        .construction
        .node_at(node.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("link")
    };
    e.link_flags().unwrap()
}
fn attribute(
    lexer: &mut LexerFixture<'_>,
    c: &mut ParseControl,
) -> Option<crate::parser::tokenizer::token::Attribute> {
    let Tag {
        name,
        mut attributes,
        ..
    } = next_tag(lexer, c);
    name.release(c).unwrap();
    let attr = attributes.try_pop(c).unwrap();
    release_token_attributes(attributes, c).unwrap();
    attr
}

#[test]
fn actual_link_disabled_removal_readdition_rel_change_and_clone_use_canonical_flags() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<link rel=stylesheet disabled><link disabled><link rel=icon>"),
        super::tests::limits(),
    );
    let (mut c, mut session, proof) = super::tests::fresh_script_owner(&request);
    let root = session.root(&mut c).unwrap();
    let mut lexer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut tag = Some(next_tag(&mut lexer, &mut c));
    let node = session
        .allocate_for_creation(
            &mut c,
            &mut tag,
            &root,
            None,
            CreationProfile::Ordinary,
            CreationContext::Ordinary,
        )
        .unwrap();
    assert!(tag.is_none());
    assert!(!flags(&session, &node).explicitly_enabled());
    session
        .remove_managed_attribute(&mut c, &node, "disabled", mutation::ManagedFamily::Link)
        .unwrap();
    assert!(flags(&session, &node).explicitly_enabled());
    session
        .remove_managed_attribute(&mut c, &node, "disabled", mutation::ManagedFamily::Link)
        .unwrap();
    let mut input = attribute(&mut lexer, &mut c);
    session
        .set_managed_attribute(
            &mut c,
            &node,
            &mut input,
            &proof,
            mutation::ManagedFamily::Link,
        )
        .unwrap();
    assert!(input.is_none());
    assert!(flags(&session, &node).explicitly_enabled());
    let mut input = attribute(&mut lexer, &mut c);
    session
        .set_managed_attribute(
            &mut c,
            &node,
            &mut input,
            &proof,
            mutation::ManagedFamily::Link,
        )
        .unwrap();
    assert!(input.is_none());
    assert!(flags(&session, &node).explicitly_enabled());
    let storage = session.active_mut().unwrap();
    let document = storage.node_document_ref(&mut c, node.index).unwrap();
    let copy = storage.clone_node(&mut c, node.index, document).unwrap();
    let copy = storage.node(&mut c, copy).unwrap();
    assert!(!flags(&session, &copy).explicitly_enabled());
    assert!(session.active().unwrap().payloads.values.iter().any(
        |p| matches!(p,RetainedPayload::Link{node:Some(n),parser_document:None}if *n==copy.index)
    ));
    let ConstructionNodeKind::Element(original) = &session
        .active()
        .unwrap()
        .construction
        .node_at(node.index)
        .expect("retained construction position")
        .kind()
    else {
        unreachable!()
    };
    let ConstructionNodeKind::Element(cloned) = &session
        .active()
        .unwrap()
        .construction
        .node_at(copy.index)
        .expect("retained construction position")
        .kind()
    else {
        unreachable!()
    };
    assert_eq!(original.attributes(), cloned.attributes());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    super::tests::storage_zero(&c);
}

#[test]
fn removing_absent_disabled_does_not_manufacture_explicit_enablement() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<link rel=stylesheet>"),
        super::tests::limits(),
    );
    let (mut c, mut session, _) = super::tests::fresh_script_owner(&request);
    let root = session.root(&mut c).unwrap();
    let mut lexer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut tag = Some(next_tag(&mut lexer, &mut c));
    let node = session
        .allocate_for_creation(
            &mut c,
            &mut tag,
            &root,
            None,
            CreationProfile::Ordinary,
            CreationContext::Ordinary,
        )
        .unwrap();
    session
        .remove_managed_attribute(&mut c, &node, "disabled", mutation::ManagedFamily::Link)
        .unwrap();
    assert!(!flags(&session, &node).explicitly_enabled());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    super::tests::storage_zero(&c);
}
