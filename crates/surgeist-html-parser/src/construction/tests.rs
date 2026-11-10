//! Fixtures shared by the construction tests, and the topic modules that use them.

use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode, HtmlFragment},
    values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
};
mod attribute_validation;
mod body_recovery;
mod child_order;
mod element_guard;
mod formatting;
mod image_form;
mod input;
mod input_types;
mod link_history;
mod listed_form;
mod literals;
mod native_payloads;
pub(crate) mod native_view;
mod output;
mod pi_origin_walk;
mod prologue;
mod radio_groups;
mod record_reserve;
mod root_merge;
mod schema;
mod script;
mod shared_history;
mod table_implied;
mod template_documents;
mod text_merging;
mod textarea;
mod token_admission;
mod topology;

fn output_owner<'a>(
    source: &'a crate::HtmlSource,
    caps: HtmlParseLimits,
) -> (
    ParseControl,
    ParserArenaSession,
    ParserNode,
    crate::parser::tokenizer::tests::LexerFixture<'a>,
) {
    output_owner_cancellable(source, caps, HtmlParseCancellation::new())
}

fn output_owner_cancellable<'a>(
    source: &'a crate::HtmlSource,
    caps: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
) -> (
    ParseControl,
    ParserArenaSession,
    ParserNode,
    crate::parser::tokenizer::tests::LexerFixture<'a>,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let mut c = ParseControl::new(caps, cancellation);
    c.admit_source(source).unwrap();
    let mut tokenizer = LexerFixture::new(source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut tag = Some(next_fixture_tag(
        &mut tokenizer,
        &mut c,
        &[111, 117, 116, 112, 117, 116],
    ));
    let output = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut tag,
            &root,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    assert!(tag.is_none());
    session.insert(&mut c, &root, &output, None, None).unwrap();
    (c, session, output, tokenizer)
}

fn next_text(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) -> Option<Text> {
    let TokenKind::Text(text) = lexical_kind(tokenizer, c) else {
        panic!("actual lexical text")
    };
    Some(text)
}
use crate::HtmlParseResource;
use crate::{HtmlParseCancellation, HtmlParseLimits};

pub(super) fn body_lexical_value(source: &crate::HtmlSource, c: &mut ParseControl) -> Text {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    c.admit_source(source).unwrap();
    let mut tokenizer = LexerFixture::new(source, HtmlTokenizerState::Data, None, c).unwrap();
    let (value, tag, attribute) = lexical_attribute_text(&mut tokenizer, c);
    tag.release(c).unwrap();
    attribute.release(c).unwrap();
    tokenizer.release(c).unwrap();
    value
}
fn radio_next_tag(
    lexer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) -> (Tag, crate::HtmlSourceRange) {
    loop {
        match lexer.next(c, false).unwrap().unwrap() {
            crate::parser::tokenizer::token::Event::Token(event) => {
                let token = event.into_token(c).unwrap();
                if let TokenKind::Tag(tag) = token.kind {
                    return (tag, token.source);
                }
                release_token_kind(&mut Some(token.kind), c).unwrap();
            }
            event => event.release(c).unwrap(),
        }
    }
}
pub(super) fn radio_next_attribute(
    lexer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) -> Option<crate::parser::tokenizer::token::Attribute> {
    let (tag, _) = radio_next_tag(lexer, c);
    let Tag {
        name,
        mut attributes,
        ..
    } = tag;
    name.release(c).unwrap();
    let attr = attributes.try_pop(c).unwrap();
    release_token_attributes(attributes, c).unwrap();
    attr
}
pub(super) fn input_owner<'a>(
    request: &'a crate::HtmlDocumentParseRequest,
) -> (
    ParseControl,
    ParserArenaSession,
    crate::parser::tree_builder::ConstructionCapability<'a>,
    crate::parser::tokenizer::tests::LexerFixture<'a>,
    ParserNode,
    Option<Tag>,
    crate::HtmlSourceRange,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let (mut c, session, capability) = fresh_script_owner(request);
    let mut lexer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (tag, source) = loop {
        match lexer.next(&mut c, false).unwrap().unwrap() {
            crate::parser::tokenizer::token::Event::Token(event) => {
                let token = event.into_token(&mut c).unwrap();
                if matches!(&token.kind, TokenKind::Tag(tag) if tag.name.value.text().code_units()==[105,110,112,117,116])
                {
                    let TokenKind::Tag(tag) = token.kind else {
                        unreachable!()
                    };
                    break (Some(tag), token.source);
                }
                release_token_kind(&mut Some(token.kind), &mut c).unwrap();
            }
            event => event.release(&mut c).unwrap(),
        }
    };
    let parent = session
        .active()
        .unwrap()
        .node(
            &mut c,
            session
                .active()
                .unwrap()
                .construction
                .node_at(0)
                .expect("retained construction position")
                .child_at(0)
                .expect("retained child ordinal")
                .position(),
        )
        .unwrap();
    (c, session, capability, lexer, parent, tag, source)
}

pub(super) fn fresh_script_owner<'a>(
    request: &'a crate::HtmlDocumentParseRequest,
) -> (
    ParseControl,
    ParserArenaSession,
    crate::parser::tree_builder::ConstructionCapability<'a>,
) {
    use crate::parser::tree_builder::StandaloneDocumentDriver;
    let mut driver = Some(crate::parser::tree_builder::entry::initialized_driver(
        request,
    ));
    for _ in 0..128 {
        let owner = driver.as_mut().unwrap();
        if owner.standalone_parts().is_some() {
            break;
        }
        owner.step().unwrap();
    }
    assert!(driver.as_mut().unwrap().standalone_parts().is_some());
    let parts = StandaloneDocumentDriver::take_standalone_fixture(&mut driver).unwrap();
    assert!(driver.is_none());
    parts
}
fn fresh_script<'a>(
    request: &'a crate::HtmlDocumentParseRequest,
    connected: bool,
    text: bool,
) -> (
    ParseControl,
    ParserArenaSession,
    crate::parser::tree_builder::ConstructionCapability<'a>,
    ParserNode,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let (mut c, mut session, capability) = fresh_script_owner(request);
    let mut tokenizer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut incoming = loop {
        let kind = lexical_kind(&mut tokenizer, &mut c);
        if matches!(&kind, TokenKind::Tag(tag) if tag.name.value.text().code_units() == [0x73,0x63,0x72,0x69,0x70,0x74])
        {
            let TokenKind::Tag(tag) = kind else {
                unreachable!()
            };
            break Some(tag);
        }
        assert!(!matches!(kind, TokenKind::Eof));
        release_token_kind(&mut Some(kind), &mut c).unwrap();
    };
    let node = session
        .allocate_disabled_parser_script(
            &mut c,
            &mut incoming,
            None,
            crate::construction::CreationContext::Ordinary,
            &capability,
        )
        .unwrap();
    assert!(incoming.is_none());
    if connected {
        let html_index = session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .expect("retained construction position")
            .child_at(0)
            .expect("retained child ordinal")
            .position();
        let html = session.active().unwrap().node(&mut c, html_index).unwrap();
        session.insert(&mut c, &html, &node, None, None).unwrap();
    }
    if text {
        let TokenKind::Text(value) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("literal text")
        };
        session
            .insert_text(&mut c, &node, None, &mut Some(value), None)
            .unwrap();
    }
    tokenizer.release(&mut c).unwrap();
    (c, session, capability, node)
}

fn next_fixture_tag(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
    expected: &[u16],
) -> Tag {
    loop {
        let kind = lexical_kind(tokenizer, c);
        if matches!(&kind, TokenKind::Tag(tag) if tag.kind == TagKind::Start && tag.name.value.text().code_units() == expected)
        {
            let TokenKind::Tag(tag) = kind else {
                unreachable!()
            };
            return tag;
        }
        assert!(!matches!(kind, TokenKind::Eof));
        release_token_kind(&mut Some(kind), c).unwrap();
    }
}

fn actual_template_and_div(
    source: &crate::HtmlSource,
    c: &mut ParseControl,
    session: &mut ParserArenaSession,
) -> (ParserNode, ParserNode) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let mut tokenizer = LexerFixture::new(source, HtmlTokenizerState::Data, None, c).unwrap();
    let template = next_fixture_tag(
        &mut tokenizer,
        c,
        &[0x74, 0x65, 0x6d, 0x70, 0x6c, 0x61, 0x74, 0x65],
    );
    let template = session.allocate_html_tag(c, &mut Some(template)).unwrap();
    let contents = session.template_contents(c, &template).unwrap().unwrap();
    let div = next_fixture_tag(&mut tokenizer, c, &[0x64, 0x69, 0x76]);
    let div = session.allocate_html_tag(c, &mut Some(div)).unwrap();
    tokenizer.release(c).unwrap();
    (contents, div)
}

fn form_session(c: &mut ParseControl) -> (ParserArenaSession, ParserNode) {
    let mut session =
        ParserArenaSession::new(c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(c).unwrap();
    let html = session.allocate_implicit_html(c).unwrap();
    session.insert(c, &root, &html, None, None).unwrap();
    let body = session.allocate_implicit_body(c, None).unwrap();
    session.insert(c, &html, &body, None, None).unwrap();
    (session, body)
}
fn form_creation(
    session: &mut ParserArenaSession,
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
    parent: &ParserNode,
    form: Option<&ParserNode>,
) -> ParserNode {
    let TokenKind::Tag(tag) = lexical_kind(tokenizer, c) else {
        panic!("actual tag")
    };
    let mut input = Some(tag);
    let node = session
        .allocate_html_tag_for_creation(
            c,
            &mut input,
            parent,
            form,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    assert!(input.is_none());
    node
}
fn assert_form_owner(
    session: &ParserArenaSession,
    c: &mut ParseControl,
    image: &ParserNode,
    owner: Option<&ParserNode>,
    inserted: bool,
) {
    let actual = session.image_form_owner(c, image).unwrap();
    match (actual, owner) {
        (Some(actual), Some(expected)) => {
            assert!(session.same_node(c, &actual, expected).unwrap())
        }
        (None, None) => {}
        _ => panic!("wrong actual form owner"),
    }
    assert_eq!(
        session.image_form_state(c, image).unwrap().parser_inserted,
        inserted
    );
}

pub(super) fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(1_000)
        .buffered_utf16_units(1_000)
        .token_attributes(1_000)
        .auxiliary_entries(1_000)
        .active_formatting_entries(1_000)
        .managed_nodes(1_000)
        .managed_edges(1_000)
        .managed_attributes(1_000)
        .managed_utf16_units(100_000)
        .source_map_entries(1_000)
        .source_map_payload_units(1_000)
        .diagnostics(1_000)
        .diagnostic_payload_units(1_000)
        .context_effects(1_000)
        .open_elements(1_000)
        .tree_depth(1_000)
        .managed_capacity_bytes(1_000_000)
        .peak_transient_bytes(2_000_000)
        .atomic_layout_bytes(1_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap()
}
pub(super) fn control() -> ParseControl {
    ParseControl::new(limits(), HtmlParseCancellation::new())
}
/// How many holders share an element's attribute backing, not counting the clone the view makes.
fn attribute_holders(attributes: surgeist_html::construction::HtmlAttributes<'_>) -> usize {
    let owned = attributes.to_ordered();
    Arc::strong_count(owned.shared_attributes()) - 1
}
fn element(namespace: Namespace, local: &str) -> PreexistingElement {
    PreexistingElement::new(
        QualifiedName::new(namespace, None, local),
        OrderedAttributes::new(Vec::new()).unwrap(),
    )
}
fn html(local: &str) -> PreexistingElement {
    element(Namespace::html(), local)
}
fn frozen(mut session: ParserArenaSession, control: &mut ParseControl) -> FrozenArena {
    match session.freeze(control) {
        Ok(value) => value,
        Err(failure) => panic!("freeze failed: {:?}", failure),
    }
}
fn locals(fragment: &HtmlFragment) -> Vec<String> {
    fragment
        .root()
        .children()
        .map(|id| {
            let surgeist_html::construction::HtmlNodeKind::Element(element) =
                fragment.node(&id).unwrap().kind()
            else {
                panic!("expected element");
            };
            element.name().local_name().to_utf8().unwrap()
        })
        .collect()
}

pub(super) fn owned_text(c: &mut ParseControl, units: &[u16]) -> Text {
    let source = crate::HtmlSource::utf16(DocumentString::from_utf16(units.to_vec()));
    let mut value = Utf16Buffer::new(c).unwrap();
    value.extend_units(units, c).unwrap();
    let mut origins = OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap();
    origins
        .try_push(
            c,
            HtmlParseTextOrigin {
                start: 0,
                end: units.len(),
                source: source.range(0, units.len()).unwrap(),
                kind: crate::parser::tokenizer::token::HtmlParseOriginKind::Literal,
            },
        )
        .unwrap();
    Text {
        value: value.freeze(c).unwrap(),
        origins,
    }
}
fn owned_tag(c: &mut ParseControl, name: &str, attrs: &[(&str, &[u16])]) -> Tag {
    let source = crate::HtmlSource::utf8("fixture");
    let mut attributes = OwnedVec::new(HtmlParseResource::TokenAttributes).unwrap();
    for (name, value) in attrs {
        let name = owned_text(c, &name.encode_utf16().collect::<Vec<_>>());
        let value = owned_text(c, value);
        attributes
            .try_push(
                c,
                crate::parser::tokenizer::token::Attribute {
                    lexical: None,
                    name,
                    value,
                    source: source.range(0, 7).unwrap(),
                },
            )
            .unwrap();
    }
    Tag {
        kind: TagKind::Start,
        name: owned_text(c, &name.encode_utf16().collect::<Vec<_>>()),
        self_closing: false,
        attributes,
        had_duplicate_attributes: false,
    }
}
pub(super) fn native_text(session: &ParserArenaSession, node: &ParserNode) -> Vec<u16> {
    let ConstructionNodeKind::Text(value) = &session
        .active()
        .unwrap()
        .construction
        .node_at(node.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("text node")
    };
    value.code_units().to_vec()
}
pub(super) fn storage_zero(c: &ParseControl) {
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::SourceMapEntries,
    ] {
        assert_eq!(c.usage().resource(resource).current, 0, "{resource:?}");
    }
}
/// The stop a result reports when it ends in `LimitExceeded` for the
/// scenario's own resource and limit. The caller derives both from the
/// configured limits, not from the observed stop; the arithmetic check is
/// that the refused charge really crosses that limit.
pub(super) fn limit_stop_of<T>(
    result: &Result<T, SessionError>,
    resource: HtmlParseResource,
    limit: u64,
) -> HtmlParseStop {
    let Err(SessionError::Stop(stop)) = result else {
        panic!(
            "expected a {resource:?} limit stop, got {:?}",
            result.as_ref().err()
        );
    };
    let HtmlParseStop::LimitExceeded {
        resource: stopped,
        limit: stopped_limit,
        current,
        requested_delta,
    } = *stop
    else {
        panic!("expected a {resource:?} limit stop, got {stop:?}");
    };
    assert_eq!((stopped, stopped_limit), (resource, limit));
    assert!(
        current <= limit && current + requested_delta > limit,
        "{stop:?} must refuse a charge that crosses {limit}"
    );
    *stop
}
/// Freezing a session whose control has stopped reports exactly the sticky
/// first stop and leaves it unchanged.
pub(super) fn assert_freeze_reports_first_stop(
    session: &mut ParserArenaSession,
    c: &mut ParseControl,
    first: HtmlParseStop,
) {
    assert_eq!(c.stop(), Some(first), "the control latched the first stop");
    assert_eq!(
        session.freeze(c).err(),
        Some(SessionError::Stop(first)),
        "freeze reports the sticky first stop"
    );
    assert_eq!(c.stop(), Some(first), "freeze keeps the first stop");
}
/// Runs a copy of a stopped range-ingress cursor to completion over the
/// given source units and retained coefficient cells, servicing each
/// request as the cursor's public protocol defines it. Returns the final
/// result and the number of replies the cursor needed.
pub(super) fn resume_range_ingress(
    mut cursor: surgeist_html::bounded::numeric::HtmlFiniteDecimalIngressCursor,
    source: &[u16],
    coefficient: &[u8],
) -> (
    Option<Result<u64, surgeist_html::bounded::numeric::HtmlRangeNumericError>>,
    usize,
) {
    use surgeist_html::bounded::numeric::{
        HtmlRangeNumericReply, HtmlRangeNumericRequest, HtmlRangeScratchRole,
    };
    let headers = cursor.headers();
    let mut buffers: [Vec<u8>; 8] = std::array::from_fn(|_| Vec::new());
    let coefficient_buffer = &mut buffers[HtmlRangeScratchRole::Coefficient.index()];
    coefficient_buffer.reserve_exact(headers.buffer(HtmlRangeScratchRole::Coefficient).capacity());
    coefficient_buffer.extend_from_slice(coefficient);
    for step in 0..1_000_000 {
        let Some(request) = cursor.request() else {
            return (cursor.result(), step);
        };
        let reply = match request {
            HtmlRangeNumericRequest::SourceUnit { index } => {
                HtmlRangeNumericReply::Unit(source[index])
            }
            HtmlRangeNumericRequest::ReadCells { cells } => HtmlRangeNumericReply::Cells(
                cells.map(|cell| cell.map(|c| buffers[c.role().index()][c.index()])),
            ),
            HtmlRangeNumericRequest::Reserve { role, required_len } => {
                let buffer = &mut buffers[role.index()];
                buffer.reserve_exact(required_len - buffer.len());
                HtmlRangeNumericReply::Reserved {
                    actual_capacity: buffer.capacity(),
                    actual_len: buffer.len(),
                }
            }
            HtmlRangeNumericRequest::AppendCell { role, value } => {
                buffers[role.index()].push(value);
                HtmlRangeNumericReply::Ack
            }
            HtmlRangeNumericRequest::WriteCell { cell, value } => {
                buffers[cell.role().index()][cell.index()] = value;
                HtmlRangeNumericReply::Ack
            }
            HtmlRangeNumericRequest::PopCell { role } => {
                buffers[role.index()].pop().expect("a cell to pop");
                HtmlRangeNumericReply::Ack
            }
            HtmlRangeNumericRequest::ScalarStep => HtmlRangeNumericReply::Ack,
            HtmlRangeNumericRequest::ReserveOutput { .. }
            | HtmlRangeNumericRequest::EmitUnit { .. } => {
                panic!("ingress never writes output: {request:?} at step {step}")
            }
        };
        cursor.reply(reply).expect("a faithful reply is accepted");
    }
    panic!("range ingress did not finish");
}
pub(super) fn lexical_kind(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) -> TokenKind {
    loop {
        match tokenizer.next(c, false).unwrap().expect("fixture token") {
            crate::parser::tokenizer::token::Event::Token(event) => {
                return event.into_token(c).unwrap().kind;
            }
            event => event.release(c).unwrap(),
        }
    }
}
/// Actual tokenizer Text with multiple points and per-point origins. Data
/// currently emits one point per token; an attribute uses the same concrete
/// Text producer and gives the splitter a real mixed record, not a fake map.
fn lexical_attribute_text(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) -> (Text, Text, Text) {
    let TokenKind::Tag(Tag {
        name,
        mut attributes,
        ..
    }) = lexical_kind(tokenizer, c)
    else {
        panic!("tag");
    };
    let attribute = attributes.try_pop(c).unwrap().unwrap();
    attributes.release(c).unwrap();
    (attribute.value, name, attribute.name)
}
