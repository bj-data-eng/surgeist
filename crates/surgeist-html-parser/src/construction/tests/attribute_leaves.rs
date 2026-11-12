//! The leaves of the origin groups of attribute strings: a live string is read from the element's
//! attributes, in a build and once published, and the history of a string an edit changed or
//! removed is read from the units that were copied before the edit.

use super::*;
use crate::construction::history::{FrozenHistory, HtmlParseHistoryFrame, HtmlParseHistoryStep};
use crate::construction::provenance::{GroupBody, LeafValue};
use crate::construction::string_storage::AttributeString;
use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
use surgeist_html::construction::HtmlNodeKind;

/// A session over a fragment root whose first start tag, `source`'s, is an element in it. The
/// tokens are retained with their lexical evidence, so the session can be published once the
/// source is at its end.
fn element<'a>(
    source: &'a crate::HtmlSource,
) -> (
    ParseControl,
    ParserArenaSession,
    ParserNode,
    LexerFixture<'a>,
) {
    use crate::parser::tokenizer::token::Event;
    let mut c = control();
    c.admit_source(source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, source).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut lexer = LexerFixture::new(source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut node = None;
    loop {
        let event = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap();
        let Event::Token(event) = event else {
            event.release(&mut c).unwrap();
            continue;
        };
        let token = event.into_token(&mut c).unwrap();
        session
            .active_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .lexical_disposition(&mut c, token.lexical.as_ref(), false)
            .unwrap();
        match token.kind {
            TokenKind::Tag(tag) => {
                let created = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
                session.insert(&mut c, &root, &created, None, None).unwrap();
                node = Some(created);
            }
            TokenKind::Eof => break,
            _ => panic!("the source is one start tag"),
        }
    }
    (c, session, node.expect("a start tag"), lexer)
}

/// The index of the group the table of origins holds for the string of `part` of the attribute at
/// `position` of `node`.
fn group_of(
    session: &ParserArenaSession,
    node: &ParserNode,
    position: usize,
    part: AttributeString,
) -> usize {
    session
        .active()
        .unwrap()
        .payloads
        .detail_origins
        .peek(FieldRef::of_attribute(node.index, position, part))
        .expect("the attribute string has a group")
        .index
}

/// What a leaf of a live attribute string names.
fn attribute_leaf(leaf: LeafValue) -> (u32, u32, AttributeString, usize) {
    match leaf {
        LeafValue::Attribute {
            node,
            position,
            part,
            offset,
        } => (node, position, part, offset),
        other => panic!(
            "not the leaf of a live attribute string: {}",
            describe(other)
        ),
    }
}

fn describe(leaf: LeafValue) -> &'static str {
    match leaf {
        LeafValue::Pending => "pending",
        LeafValue::Payload(_) => "a registry row's string",
        LeafValue::Field { .. } => "a string of a non-element record",
        LeafValue::Attribute { .. } => "a live attribute string",
        LeafValue::Empty => "empty",
    }
}

/// The leaf the group at `index` has, the units it covers, and whether it is retired.
fn leaf_of(session: &ParserArenaSession, index: usize) -> (LeafValue, usize, bool) {
    let evidence = session.active().unwrap().evidence.as_ref().unwrap();
    let group = evidence.group_at(index).expect("a group of the evidence");
    let GroupBody::Leaf { value, .. } = group.body else {
        panic!("the group of a token string is a leaf")
    };
    (value, group.units, group.retirement.is_some())
}

/// The units the leaf of the group at `index` names in the build.
fn built_units(session: &ParserArenaSession, c: &mut ParseControl, index: usize) -> Vec<u16> {
    let (value, units, _) = leaf_of(session, index);
    session
        .active()
        .unwrap()
        .leaf_strings()
        .units(c, value, units)
        .unwrap()
        .to_vec()
}

/// Publishes the parse of `source`: its associations are projected, its tokenizer released and
/// its history frozen.
fn publish(
    session: &mut ParserArenaSession,
    c: &mut ParseControl,
    lexer: LexerFixture<'_>,
    source: &crate::HtmlSource,
) -> FrozenHistory {
    session.prepare_native_associations(c, source).unwrap();
    lexer.release(c).unwrap();
    session.freeze_history(c).unwrap()
}

/// The units the leaves of the group at `index` hold in a published history, and where the first
/// of them is.
fn published_units(history: &FrozenHistory, index: usize) -> (Vec<u16>, Option<*const u16>) {
    let view = history.view();
    let mut scratch = vec![HtmlParseHistoryFrame::default(); view.len()];
    let mut cursor = view
        .cursor(view.group(index).unwrap(), &mut scratch, None)
        .unwrap();
    let (mut units, mut first) = (Vec::new(), None);
    loop {
        match cursor.advance() {
            HtmlParseHistoryStep::Progress => {}
            HtmlParseHistoryStep::Leaf(leaf) => {
                first.get_or_insert(leaf.units().as_ptr());
                units.extend_from_slice(leaf.units());
            }
            HtmlParseHistoryStep::Complete => break,
            HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
            HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
        }
    }
    (units, first)
}

/// The published attributes of the first element of the history's fragment.
fn published_attributes(
    history: &FrozenHistory,
) -> Vec<(
    surgeist_html::values::DocumentString,
    surgeist_html::values::DocumentString,
)> {
    let fragment = history.fragment().unwrap();
    let id = fragment.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
        panic!("the element")
    };
    element
        .attributes()
        .iter()
        .map(|attribute| {
            (
                attribute.name().local_name().clone(),
                attribute.value().clone(),
            )
        })
        .collect()
}

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn the_leaf_of_a_live_attribute_string_is_read_from_the_element_in_a_build_and_once_published() {
    let source = crate::HtmlSource::utf8("<div id=a class=bb>");
    let (mut c, mut session, node, lexer) = element(&source);
    let name = group_of(&session, &node, 1, AttributeString::Name);
    let value = group_of(&session, &node, 1, AttributeString::Value);
    for (index, part) in [
        (name, AttributeString::Name),
        (value, AttributeString::Value),
    ] {
        // The leaf names the attribute and the string, not a copy of its units.
        let (leaf, covered, retired) = leaf_of(&session, index);
        assert!(!retired);
        assert_eq!(
            attribute_leaf(leaf),
            (u32::try_from(node.index).unwrap(), 1, part, 0)
        );
        assert_eq!(covered, if part == AttributeString::Name { 5 } else { 2 });
    }
    // In a build the units are those the element's attributes hold.
    assert_eq!(built_units(&session, &mut c, name), units("class"));
    assert_eq!(built_units(&session, &mut c, value), units("bb"));
    let history = publish(&mut session, &mut c, lexer, &source);
    // Published, they are those of the published attributes, whose values borrow the one block of
    // the parse.
    let attributes = published_attributes(&history);
    let (published, first) = published_units(&history, name);
    assert_eq!(published, units("class"));
    assert_eq!(first, Some(attributes[1].0.code_units().as_ptr()));
    let (published, first) = published_units(&history, value);
    assert_eq!(published, units("bb"));
    assert_eq!(first, Some(attributes[1].1.code_units().as_ptr()));
    history.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn the_leaves_of_the_attributes_after_a_removed_one_name_the_places_they_moved_to() {
    let source = crate::HtmlSource::utf8("<div a=1 b=2 c=3>");
    let (mut c, mut session, node, lexer) = element(&source);
    let before: Vec<_> = (0..3)
        .map(|position| group_of(&session, &node, position, AttributeString::Value))
        .collect();
    session
        .active_mut()
        .unwrap()
        .remove_managed_field(&mut c, node.index, "a")
        .unwrap();
    // The groups of b and c are the ones of the places they hold now, and their leaves name them.
    for (position, (&index, expected)) in before[1..].iter().zip(["2", "3"]).enumerate() {
        assert_eq!(
            group_of(&session, &node, position, AttributeString::Value),
            index
        );
        let (leaf, _, retired) = leaf_of(&session, index);
        assert!(!retired);
        assert_eq!(
            attribute_leaf(leaf),
            (
                u32::try_from(node.index).unwrap(),
                u32::try_from(position).unwrap(),
                AttributeString::Value,
                0
            )
        );
        assert_eq!(built_units(&session, &mut c, index), units(expected));
    }
    let history = publish(&mut session, &mut c, lexer, &source);
    let attributes = published_attributes(&history);
    assert_eq!(attributes.len(), 2);
    for (position, (&index, expected)) in before[1..].iter().zip(["2", "3"]).enumerate() {
        let (published, first) = published_units(&history, index);
        assert_eq!(published, units(expected));
        assert_eq!(
            first,
            Some(attributes[position].1.code_units().as_ptr()),
            "the leaf of {expected} is read from the attribute at {position}"
        );
    }
    history.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn the_history_of_a_removed_attribute_value_is_read_after_publication() {
    let source = crate::HtmlSource::utf8("<div a=1 b=2>");
    let (mut c, mut session, node, lexer) = element(&source);
    let name = group_of(&session, &node, 0, AttributeString::Name);
    let value = group_of(&session, &node, 0, AttributeString::Value);
    session
        .active_mut()
        .unwrap()
        .remove_managed_field(&mut c, node.index, "a")
        .unwrap();
    // The groups of the removed attribute are retired, and their leaves name copies of their units
    // that the removal did not change: the attribute at the place they named is b now.
    for (index, expected) in [(name, "a"), (value, "1")] {
        let (leaf, _, retired) = leaf_of(&session, index);
        assert!(retired);
        assert!(matches!(leaf, LeafValue::Payload(_)), "{}", describe(leaf));
        assert_eq!(built_units(&session, &mut c, index), units(expected));
    }
    let history = publish(&mut session, &mut c, lexer, &source);
    let attributes = published_attributes(&history);
    assert_eq!(attributes.len(), 1);
    assert_eq!(
        attributes[0].0,
        surgeist_html::values::DocumentString::from("b")
    );
    assert_eq!(published_units(&history, name).0, units("a"));
    let (published, first) = published_units(&history, value);
    assert_eq!(published, units("1"));
    assert_ne!(
        first,
        Some(attributes[0].1.code_units().as_ptr()),
        "the removed value is not read from the attribute that took its place"
    );
    history.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn the_history_of_a_replaced_attribute_value_is_read_after_publication() {
    let source = crate::HtmlSource::utf8("<div a=abc b=u>");
    let (mut c, mut session, node, lexer) = element(&source);
    let old: Vec<_> = (0..2)
        .map(|position| group_of(&session, &node, position, AttributeString::Value))
        .collect();
    // The first value is replaced by a shorter one, which the element overwrites in place, and the
    // second by a longer one, which it appends at the end of its arena.
    for (position, text) in [(0, "x"), (1, "wxyz")] {
        let value = literal_string(&mut c, text.as_bytes()).unwrap();
        let storage = session.active_mut().unwrap();
        let field = storage
            .field_copy_string(
                &mut c,
                FieldCopy {
                    value,
                    origins: OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap(),
                    original: None,
                },
            )
            .unwrap();
        storage
            .replace_registered_attribute_value(&mut c, node.index, position, field)
            .unwrap();
    }
    for (position, (&index, (old_text, new_text))) in
        old.iter().zip([("abc", "x"), ("u", "wxyz")]).enumerate()
    {
        // The retired group reads the units that were copied before the edit; the group that
        // replaced it names the attribute.
        let (leaf, _, retired) = leaf_of(&session, index);
        assert!(retired);
        assert!(matches!(leaf, LeafValue::Payload(_)), "{}", describe(leaf));
        assert_eq!(built_units(&session, &mut c, index), units(old_text));
        let current = group_of(&session, &node, position, AttributeString::Value);
        assert_ne!(current, index);
        assert_eq!(built_units(&session, &mut c, current), units(new_text));
    }
    let history = publish(&mut session, &mut c, lexer, &source);
    let attributes = published_attributes(&history);
    for (position, (&index, (old_text, new_text))) in
        old.iter().zip([("abc", "x"), ("u", "wxyz")]).enumerate()
    {
        assert_eq!(published_units(&history, index).0, units(old_text));
        assert_eq!(attributes[position].1.code_units(), units(new_text));
    }
    history.release(&mut c).unwrap();
    storage_zero(&c);
}
