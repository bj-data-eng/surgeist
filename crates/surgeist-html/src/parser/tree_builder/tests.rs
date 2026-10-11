use super::prologue::Payload;
use super::*;
use crate::{
    DocumentString, HtmlConstructionLimits, HtmlDocumentMode, HtmlFragment, HtmlNodeKind,
    HtmlParseCancellation, HtmlParseLimits, HtmlSource, Namespace, OrderedAttributes,
    QualifiedName,
    construction::parser_session::{FrozenArena, RootKind},
    parser::{
        control::Invariant,
        tokenizer::{
            Tokenizer,
            states::State,
            token::{Event, TagKind, TokenKind},
        },
    },
};

pub(super) fn limits(open: u64) -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(open)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}
fn control(open: u64) -> ParseControl {
    ParseControl::new(limits(open), HtmlParseCancellation::new())
}

/// Selects actual tokenizer emissions. There is no fabricated token or tree
/// parser here: each test invokes the named construction operation explicitly.
fn next_kind(tokenizer: &mut Tokenizer<'_>, control: &mut ParseControl) -> TokenKind {
    loop {
        match tokenizer
            .next(control, false)
            .unwrap()
            .expect("test input has a token")
        {
            Event::Token(event) => return event.into_token(control).unwrap().kind,
            event => event.release(control).unwrap(),
        }
    }
}
fn tag(tokenizer: &mut Tokenizer<'_>, control: &mut ParseControl) -> Option<Tag> {
    let TokenKind::Tag(tag) = next_kind(tokenizer, control) else {
        panic!("expected tag")
    };
    Some(tag)
}
fn text(tokenizer: &mut Tokenizer<'_>, control: &mut ParseControl) -> Option<Text> {
    let TokenKind::Text(text) = next_kind(tokenizer, control) else {
        panic!("expected text")
    };
    Some(text)
}
fn begin(
    tokenizer: &mut Tokenizer<'_>,
    kind: RootKind,
    control: &mut ParseControl,
) -> (LocationKernel, ParserNode) {
    let document = matches!(kind, RootKind::Document(_));
    let mut session = ParserArenaSession::new(control, kind).unwrap();
    let mut html = tag(tokenizer, control);
    let html = session.allocate_html_tag(control, &mut html).unwrap();
    if document {
        let root = session.root(control).unwrap();
        session.insert(control, &root, &html, None).unwrap();
    }
    let mut kernel = match LocationKernel::new(&mut Some(session), control) {
        Ok(kernel) => kernel,
        Err(error) => panic!("kernel creation: {error:?}"),
    };
    kernel.push(&html, control).unwrap();
    (kernel, html)
}
fn add(
    kernel: &mut LocationKernel,
    tokenizer: &mut Tokenizer<'_>,
    push: StackPush,
    control: &mut ParseControl,
) -> ParserNode {
    kernel
        .insert_html_tag(&mut tag(tokenizer, control), push, None, control)
        .unwrap()
        .node
}
fn preexisting(
    session: &mut ParserArenaSession,
    namespace: Namespace,
    name: &str,
    control: &mut ParseControl,
) -> ParserNode {
    // Build an actual native artifact outside the parser's physical ledger;
    // this is the documented preexisting-value input, not a foreign-token path.
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(3, 2, 0, 1_000, 2)).unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(namespace, None, DocumentString::from(name)),
            OrderedAttributes::default(),
        )
        .unwrap();
    let fragment = builder.finish();
    let node = fragment
        .node(&fragment.root().children().next().unwrap())
        .unwrap();
    let HtmlNodeKind::Element(element) = node.kind() else {
        panic!("native element")
    };
    session.allocate_element(control, element.clone()).unwrap()
}
fn same(
    kernel: &LocationKernel,
    left: &ParserNode,
    right: &ParserNode,
    control: &mut ParseControl,
) {
    assert!(kernel.session.same_node(control, left, right).unwrap());
}

/// This freezes the native arena for observation only. It does not run or
/// impersonate insertion-mode dispatch, EOF acceptance, or a parse report.
fn freeze(kernel: LocationKernel, control: &mut ParseControl) -> FrozenArena {
    let LocationKernel {
        mut session, open, ..
    } = kernel;
    open.unwrap().release(control).unwrap();
    session.freeze(control).unwrap()
}
pub(super) fn assert_released(control: &ParseControl) {
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::SourceMapPayloadUnits,
        HtmlParseResource::Diagnostics,
        HtmlParseResource::DiagnosticPayloadUnits,
        HtmlParseResource::ContextEffects,
        HtmlParseResource::OpenElements,
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::PeakTransientBytes,
    ] {
        assert_eq!(
            control.usage().resource(resource).current,
            0,
            "{resource:?}"
        );
    }
}

#[test]
fn ordinary_override_and_stack_pop_preserve_native_tree() {
    let source = HtmlSource::utf8("<html><body><p>A&amp;B");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, html) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let p = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let ordinary = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &ordinary.parent, &p, &mut c);
    assert!(ordinary.before.is_none());
    let override_location = kernel.adjusted_location(Some(&html), &mut c).unwrap();
    same(&kernel, &override_location.parent, &html, &mut c);
    assert!(override_location.before.is_none());
    let mut offset = 0;
    loop {
        match next_kind(&mut tokenizer, &mut c) {
            TokenKind::Text(input) => {
                let length = input.value.text().len();
                let mut input = Some(input);
                let result = kernel
                    .insert_text(&mut input, None, &mut c)
                    .unwrap()
                    .unwrap();
                assert_eq!(result.offset, offset);
                assert!(input.is_none());
                offset += length;
            }
            TokenKind::Eof => break,
            _ => panic!("expected remaining text"),
        }
    }
    let popped = kernel.pop(&mut c).unwrap().unwrap();
    same(&kernel, &popped, &p, &mut c);
    assert!(kernel.session.parent(&mut c, &p).unwrap().is_some());
    same(
        &kernel,
        &kernel.adjusted_location(None, &mut c).unwrap().parent,
        &body,
        &mut c,
    );
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    let document = frozen.document().unwrap();
    let html = document
        .node(&document.root().children().next().unwrap())
        .unwrap();
    let body = document.node(&html.children().next().unwrap()).unwrap();
    let p = document.node(&body.children().next().unwrap()).unwrap();
    let value = document.node(&p.children().next().unwrap()).unwrap();
    assert!(matches!(value.kind(), HtmlNodeKind::Text(value) if value.code_units() == [65,38,66]));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn foster_text_uses_actual_table_parent_and_merges_only_preceding_text() {
    let source = HtmlSource::utf8("<html><body>A<table>B");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let mut a = text(&mut tokenizer, &mut c);
    let first = kernel.insert_text(&mut a, None, &mut c).unwrap().unwrap();
    let table = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    kernel.set_foster_parenting(true, &mut c).unwrap();
    let location = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &location.parent, &body, &mut c);
    same(&kernel, location.before.as_ref().unwrap(), &table, &mut c);
    let mut b = text(&mut tokenizer, &mut c);
    let second = kernel.insert_text(&mut b, None, &mut c).unwrap().unwrap();
    same(&kernel, &first.node, &second.node, &mut c);
    assert_eq!(second.offset, 1);
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    let document = frozen.document().unwrap();
    let html = document
        .node(&document.root().children().next().unwrap())
        .unwrap();
    let body = document.node(&html.children().next().unwrap()).unwrap();
    let children: Vec<_> = body.children().collect();
    assert_eq!(children.len(), 2);
    assert!(
        matches!(document.node(&children[0]).unwrap().kind(), HtmlNodeKind::Text(value) if value.code_units() == [65,66])
    );
    assert!(
        matches!(document.node(&children[1]).unwrap().kind(), HtmlNodeKind::Element(value) if value.name().local_name().code_units() == [116,97,98,108,101])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn detached_table_fosters_into_previous_stack_element() {
    let source = HtmlSource::utf8("<html><body><table>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let table = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    kernel.session.remove(&mut c, &table).unwrap();
    kernel.set_foster_parenting(true, &mut c).unwrap();
    let location = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &location.parent, &body, &mut c);
    assert!(location.before.is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        3
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        4
    );
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn foster_parent_uses_moved_table_parent_instead_of_stack_predecessor() {
    let source = HtmlSource::utf8("<html><body><div><table>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, html) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let table = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    kernel.session.insert(&mut c, &html, &table, None).unwrap();
    kernel.set_foster_parenting(true, &mut c).unwrap();
    let location = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &location.parent, &html, &mut c);
    same(&kernel, location.before.as_ref().unwrap(), &table, &mut c);
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn latest_template_or_table_determines_foster_route() {
    // The last matching stack entry wins in both orderings. Template contents
    // are separate fragment associations with no ordinary parent.
    for (input, latest_template) in [
        ("<html><body><table><template><tr>", true),
        ("<html><body><template><table><tr>", false),
    ] {
        let source = HtmlSource::utf8(input);
        let mut c = control(100);
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
        let (mut kernel, _) = begin(
            &mut tokenizer,
            RootKind::Document(HtmlDocumentMode::NoQuirks),
            &mut c,
        );
        add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
        let first = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
        let second = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
        add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
        kernel.set_foster_parenting(true, &mut c).unwrap();
        let location = kernel.adjusted_location(None, &mut c).unwrap();
        let template = if latest_template { &second } else { &first };
        let contents = kernel
            .session
            .template_contents(&mut c, template)
            .unwrap()
            .unwrap();
        same(&kernel, &location.parent, &contents, &mut c);
        if latest_template {
            assert!(location.before.is_none());
        } else {
            same(&kernel, location.before.as_ref().unwrap(), &second, &mut c);
        }
        assert!(kernel.session.parent(&mut c, &contents).unwrap().is_none());
        tokenizer.release(&mut c).unwrap();
        kernel.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn fragment_no_table_foster_case_and_ordinary_first_node_use_actual_fragment_root() {
    let source = HtmlSource::utf8("<html><tr><x>\u{feff}");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, html) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let root = kernel.session.root(&mut c).unwrap();
    let ordinary = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &ordinary.parent, &root, &mut c);
    let tr = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    kernel.set_foster_parenting(true, &mut c).unwrap();
    let foster = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &foster.parent, &root, &mut c);
    assert!(foster.before.is_none());
    let x = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
    let x_parent = kernel.session.parent(&mut c, &x).unwrap().unwrap();
    same(&kernel, &x_parent, &root, &mut c);
    assert!(kernel.session.parent(&mut c, &html).unwrap().is_none());
    let mut input = text(&mut tokenizer, &mut c);
    kernel.insert_text(&mut input, None, &mut c).unwrap();
    assert!(kernel.in_scope(&tr, Scope::Default, &mut c).unwrap());
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    let fragment = frozen.fragment().unwrap();
    let children: Vec<_> = fragment.root().children().collect();
    assert_eq!(children.len(), 3);
    assert!(
        matches!(fragment.node(&children[2]).unwrap().kind(), HtmlNodeKind::Text(value) if value.code_units() == [0xfeff])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn template_contents_routing_precedes_fragment_first_node_substitution() {
    let source = HtmlSource::utf8("<html><template>x");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let template = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let contents = kernel
        .session
        .template_contents(&mut c, &template)
        .unwrap()
        .unwrap();
    let location = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &location.parent, &contents, &mut c);
    kernel
        .insert_text(&mut text(&mut tokenizer, &mut c), None, &mut c)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    let fragment = frozen.fragment().unwrap();
    let template = fragment
        .node(&fragment.root().children().next().unwrap())
        .unwrap();
    assert_eq!(template.children().len(), 0);
    let contents = fragment
        .node(&template.template_contents().unwrap())
        .unwrap();
    assert_eq!(contents.children().len(), 1);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn imported_html_template_has_unavailable_patching_state() {
    let source = HtmlSource::utf8("<html>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let template = preexisting(&mut kernel.session, Namespace::html(), "template", &mut c);
    kernel.push(&template, &mut c).unwrap();
    assert!(matches!(
        kernel.adjusted_location(None, &mut c),
        Err(KernelError::Session(
            SessionError::TemplateInsertionStateUnavailable
        ))
    ));
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn foreign_template_and_table_names_are_ordinary_targets() {
    let source = HtmlSource::utf8("<html>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    kernel.set_foster_parenting(true, &mut c).unwrap();
    for name in ["template", "table", "tr"] {
        let foreign = preexisting(&mut kernel.session, Namespace::svg(), name, &mut c);
        kernel.push(&foreign, &mut c).unwrap();
        let location = kernel.adjusted_location(None, &mut c).unwrap();
        same(&kernel, &location.parent, &foreign, &mut c);
        assert!(location.before.is_none());
        assert!(
            kernel
                .session
                .template_contents(&mut c, &foreign)
                .unwrap()
                .is_none()
        );
        kernel.pop(&mut c).unwrap();
    }
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn foreign_scope_boundaries_include_annotation_xml_and_preserve_case() {
    let mut c = control(100);
    // Independently enumerated normative scope boundaries, not expected values
    // derived from the acquired engine's incomplete integration-point helper.
    for (namespace, names) in [
        (
            Namespace::mathml(),
            &["mi", "mo", "mn", "ms", "mtext", "annotation-xml"][..],
        ),
        (Namespace::svg(), &["foreignObject", "desc", "title"][..]),
    ] {
        for name in names {
            let qualified =
                QualifiedName::new(namespace.clone(), None, DocumentString::from(*name));
            for scope in [Scope::Default, Scope::ListItem, Scope::Button] {
                assert!(
                    tag_sets::scope_boundary(&qualified, scope, &mut c).unwrap(),
                    "{name}"
                );
            }
            assert!(!tag_sets::scope_boundary(&qualified, Scope::Table, &mut c).unwrap());
        }
    }
    for (namespace, units) in [
        (Namespace::none(), "table".encode_utf16().collect()),
        (Namespace::html(), "TABLE".encode_utf16().collect()),
        (Namespace::svg(), "foreignobject".encode_utf16().collect()),
        (Namespace::mathml(), vec![0xd800, 0x69]),
    ] {
        let qualified = QualifiedName::new(namespace, None, DocumentString::from_utf16(units));
        assert!(!tag_sets::scope_boundary(&qualified, Scope::Default, &mut c).unwrap());
    }
}

#[test]
fn target_match_precedes_boundary_and_scopes_use_actual_stack_order() {
    let source = HtmlSource::utf8("<html><body><p><ol><button>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let p = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let ol = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let button = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    assert!(kernel.in_scope(&p, Scope::Default, &mut c).unwrap());
    assert!(!kernel.in_scope(&p, Scope::ListItem, &mut c).unwrap());
    assert!(!kernel.in_scope(&p, Scope::Button, &mut c).unwrap());
    assert!(kernel.in_scope(&p, Scope::Table, &mut c).unwrap());
    assert!(kernel.in_scope(&ol, Scope::ListItem, &mut c).unwrap());
    assert!(kernel.in_scope(&button, Scope::Button, &mut c).unwrap());
    let annotation = preexisting(
        &mut kernel.session,
        Namespace::mathml(),
        "annotation-xml",
        &mut c,
    );
    kernel.push(&annotation, &mut c).unwrap();
    assert!(!kernel.in_scope(&p, Scope::Default, &mut c).unwrap());
    assert!(
        kernel
            .in_scope(&annotation, Scope::Default, &mut c)
            .unwrap()
    );
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn literal_surrogates_and_supplementary_text_survive_real_transfer_and_merge() {
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
        60, 104, 116, 109, 108, 62, 60, 98, 111, 100, 121, 62, 0xd800, 0xdc00, 0xd800, 0xfeff, 38,
        97, 109, 112, 59,
    ]));
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let mut first_node = None;
    loop {
        match next_kind(&mut tokenizer, &mut c) {
            TokenKind::Text(input) => {
                let result = kernel
                    .insert_text(&mut Some(input), None, &mut c)
                    .unwrap()
                    .unwrap();
                if let Some(first) = &first_node {
                    same(&kernel, &result.node, first, &mut c);
                } else {
                    first_node = Some(result.node);
                }
            }
            TokenKind::Eof => break,
            token => {
                let _ = token;
                panic!("unexpected test token");
            }
        }
    }
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    let fragment = frozen.fragment().unwrap();
    let body = fragment
        .node(&fragment.root().children().next().unwrap())
        .unwrap();
    assert_eq!(body.children().len(), 1);
    let text = fragment.node(&body.children().next().unwrap()).unwrap();
    assert!(
        matches!(text.kind(), HtmlNodeKind::Text(value) if value.code_units() == [0xd800,0xdc00,0xd800,0xfeff,38])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn empty_wrong_kind_duplicate_and_foreign_handles_preserve_stack() {
    let source = HtmlSource::utf8("<html><body>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut kernel = match LocationKernel::new(&mut Some(session), &mut c) {
        Ok(k) => k,
        Err(_) => panic!("kernel"),
    };
    assert!(matches!(
        kernel.adjusted_location(None, &mut c),
        Err(KernelError::EmptyStack)
    ));
    assert!(matches!(
        kernel.push(&root, &mut c),
        Err(KernelError::NotElement)
    ));
    let mut html_tag = tag(&mut tokenizer, &mut c);
    let html = kernel
        .session
        .allocate_html_tag(&mut c, &mut html_tag)
        .unwrap();
    let mut body_tag = tag(&mut tokenizer, &mut c);
    let body = kernel
        .session
        .allocate_html_tag(&mut c, &mut body_tag)
        .unwrap();
    assert!(matches!(
        kernel.push(&body, &mut c),
        Err(KernelError::FirstOpenElementNotHtml)
    ));
    kernel.push(&html, &mut c).unwrap();
    assert!(matches!(
        kernel.push(&html, &mut c),
        Err(KernelError::DuplicateOpenElement)
    ));
    let mut second = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let foreign = second.root(&mut c).unwrap();
    assert!(matches!(
        kernel.adjusted_location(Some(&foreign), &mut c),
        Err(KernelError::Session(SessionError::ForeignNode))
    ));
    assert!(matches!(
        kernel.in_scope(&foreign, Scope::Default, &mut c),
        Err(KernelError::Session(SessionError::ForeignNode))
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        1
    );
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    second.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert!(matches!(kernel.pop(&mut c), Err(KernelError::Consumed)));
    assert_released(&c);
}

#[test]
fn exact_stack_cap_rejection_keeps_permanent_nodes_and_cleans_all_live_storage() {
    let source = HtmlSource::utf8("<html><body>");
    let mut c = control(1);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let mut body = tag(&mut tokenizer, &mut c);
    let body = kernel.session.allocate_html_tag(&mut c, &mut body).unwrap();
    assert!(matches!(
        kernel.push(&body, &mut c),
        Err(KernelError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::OpenElements,
            ..
        }))
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        1
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::OpenElements)
            .high_water,
        1
    );
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    assert_eq!(nodes, 3);
    let stop = c.stop();
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    assert_released(&c);
}

#[test]
fn cancellation_and_foreign_control_precede_mutation_and_allow_original_owner_cleanup() {
    let source = HtmlSource::utf8("<html>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(100), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let mut foreign = control(100);
    assert!(matches!(
        kernel.set_foster_parenting(true, &mut foreign),
        Err(KernelError::Stop(ParseStop::InternalInvariant(
            Invariant::ForeignControl
        )))
    ));
    assert!(matches!(
        kernel.release(&mut foreign),
        Err(KernelError::Stop(_))
    ));
    assert!(!kernel.foster_parenting);
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        1
    );
    cancellation.cancel();
    assert!(matches!(
        kernel.set_foster_parenting(true, &mut c),
        Err(KernelError::Stop(ParseStop::Cancelled))
    ));
    assert!(!kernel.foster_parenting);
    let stop = c.stop();
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_released(&c);
}

#[test]
fn foster_flag_has_independent_physical_write_and_copy_work() {
    let mut c = control(100);
    let session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let mut kernel = match LocationKernel::new(&mut Some(session), &mut c) {
        Ok(k) => k,
        Err(_) => panic!("kernel"),
    };
    let before = c.usage().clone();
    kernel.set_foster_parenting(true, &mut c).unwrap();
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::StateStep) - before.work(WorkCategory::StateStep),
        0
    );
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn complete_html_scope_lists_have_exact_namespace_and_prefix_independence() {
    let mut c = control(100);
    for name in [
        "applet", "caption", "html", "table", "td", "th", "marquee", "object", "select", "template",
    ] {
        let qualified = QualifiedName::new(
            Namespace::html(),
            Some(DocumentString::from("prefix")),
            DocumentString::from(name),
        );
        assert!(tag_sets::scope_boundary(&qualified, Scope::Default, &mut c).unwrap());
        assert!(tag_sets::scope_boundary(&qualified, Scope::ListItem, &mut c).unwrap());
        assert!(tag_sets::scope_boundary(&qualified, Scope::Button, &mut c).unwrap());
        assert_eq!(
            tag_sets::scope_boundary(&qualified, Scope::Table, &mut c).unwrap(),
            matches!(name, "html" | "table" | "template")
        );
    }
    for name in ["ol", "ul"] {
        let qualified = QualifiedName::new(Namespace::html(), None, DocumentString::from(name));
        assert!(!tag_sets::scope_boundary(&qualified, Scope::Default, &mut c).unwrap());
        assert!(tag_sets::scope_boundary(&qualified, Scope::ListItem, &mut c).unwrap());
        assert!(!tag_sets::scope_boundary(&qualified, Scope::Button, &mut c).unwrap());
    }
    let button = QualifiedName::new(Namespace::html(), None, DocumentString::from("button"));
    assert!(!tag_sets::scope_boundary(&button, Scope::Default, &mut c).unwrap());
    assert!(!tag_sets::scope_boundary(&button, Scope::ListItem, &mut c).unwrap());
    assert!(tag_sets::scope_boundary(&button, Scope::Button, &mut c).unwrap());
}

#[test]
fn unit_comparison_charges_literal_and_utf16_bytes_before_inspection() {
    let value = DocumentString::from_utf16(vec![0xd800]);
    let mut c = control(100);
    let before = c.usage().clone();
    assert!(!tag_sets::ascii_eq(&value, "x", &mut c).unwrap());
    assert_eq!(
        c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::InspectByte) - before.work(WorkCategory::InspectByte),
        3
    );
    let caps = limits(100).into_builder().work_credits(2).finish().unwrap();
    let mut short = ParseControl::new(caps, HtmlParseCancellation::new());
    assert!(matches!(
        tag_sets::ascii_eq(&value, "x", &mut short),
        Err(ParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    assert_eq!(short.usage().work(WorkCategory::RecordVisit), 1);
    assert_eq!(short.usage().work(WorkCategory::InspectByte), 0);
    assert_eq!(value.code_units(), [0xd800]);
}

#[test]
fn kernel_construction_retains_actual_session_on_foreign_control_failure() {
    let mut owner = control(100);
    let mut input = Some(ParserArenaSession::new(&mut owner, RootKind::Fragment).unwrap());
    let mut foreign = control(100);
    assert!(matches!(
        LocationKernel::new(&mut input, &mut foreign),
        Err(KernelError::Session(SessionError::Stop(
            ParseStop::InternalInvariant(Invariant::ForeignControl)
        )))
    ));
    assert!(input.is_some());
    let mut kernel = LocationKernel::new(&mut input, &mut owner).unwrap();
    assert!(input.is_none());
    kernel.release(&mut owner).unwrap();
    assert_released(&owner);
}

#[test]
fn document_character_target_discards_actual_token_without_allocating_a_text_node() {
    let source = HtmlSource::utf8("<html>A");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let document = kernel.session.root(&mut c).unwrap();
    let before = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    let mut input = text(&mut tokenizer, &mut c);
    assert!(
        kernel
            .insert_text(&mut input, Some(&document), &mut c)
            .unwrap()
            .is_none()
    );
    assert!(input.is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        before
    );
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn existing_document_element_ignores_new_element_insertion_but_retains_created_node() {
    // HTML §13.2.6.1 step 2 returns without inserting when the target is a
    // Document that already has an element child. Creation still precedes it.
    let source = HtmlSource::utf8("<html><div id=x>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let document = kernel.session.root(&mut c).unwrap();
    let mut input = tag(&mut tokenizer, &mut c);
    let created = kernel
        .insert_html_tag(&mut input, StackPush::NoPush, Some(&document), &mut c)
        .unwrap();
    assert!(input.is_none());
    assert!(
        kernel
            .session
            .parent(&mut c, &created.node)
            .unwrap()
            .is_none(),
        "ignored element must remain detached"
    );
    assert_eq!(
        created.insertion,
        ElementInsertionOutcome::Ignored(ElementInsertionIgnored::DocumentAlreadyHasElement)
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        1
    );
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    assert_eq!(frozen.document().unwrap().root().children().len(), 1);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ignored_document_insertion_still_pushes_created_element_for_following_text() {
    let source = HtmlSource::utf8("<html><div>X");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let document = kernel.session.root(&mut c).unwrap();
    let created = kernel
        .insert_html_tag(
            &mut tag(&mut tokenizer, &mut c),
            StackPush::Push,
            Some(&document),
            &mut c,
        )
        .unwrap();
    assert_eq!(
        created.insertion,
        ElementInsertionOutcome::Ignored(ElementInsertionIgnored::DocumentAlreadyHasElement)
    );
    assert!(
        kernel
            .session
            .parent(&mut c, &created.node)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        2
    );
    let inserted = kernel
        .insert_text(&mut text(&mut tokenizer, &mut c), None, &mut c)
        .unwrap()
        .unwrap();
    let parent = kernel
        .session
        .parent(&mut c, &inserted.node)
        .unwrap()
        .unwrap();
    same(&kernel, &parent, &created.node, &mut c);
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    assert_eq!(frozen.document().unwrap().root().children().len(), 1);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn document_without_element_child_accepts_actual_created_element() {
    let source = HtmlSource::utf8("<html><div>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, html) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    kernel.session.remove(&mut c, &html).unwrap();
    let document = kernel.session.root(&mut c).unwrap();
    let created = kernel
        .insert_html_tag(
            &mut tag(&mut tokenizer, &mut c),
            StackPush::NoPush,
            Some(&document),
            &mut c,
        )
        .unwrap();
    assert_eq!(created.insertion, ElementInsertionOutcome::Inserted);
    let parent = kernel
        .session
        .parent(&mut c, &created.node)
        .unwrap()
        .unwrap();
    same(&kernel, &parent, &document, &mut c);
    tokenizer.release(&mut c).unwrap();
    let mut frozen = freeze(kernel, &mut c);
    let document = frozen.document().unwrap();
    let element = document
        .node(&document.root().children().next().unwrap())
        .unwrap();
    assert_eq!(document.root().children().len(), 1);
    assert!(
        matches!(element.kind(), HtmlNodeKind::Element(value) if value.name().local_name().code_units() == [100,105,118])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn already_parented_element_ignores_placement_without_native_move() {
    let source = HtmlSource::utf8("<html><body><p>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, html) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let p = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    assert_eq!(
        kernel.insert_element(&p, Some(&html), &mut c).unwrap(),
        ElementInsertionOutcome::Ignored(ElementInsertionIgnored::ParentAlreadyPresent)
    );
    let parent = kernel.session.parent(&mut c, &p).unwrap().unwrap();
    same(&kernel, &parent, &body, &mut c);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        edges
    );
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn detached_element_is_ignored_as_inclusive_ancestor_of_itself() {
    let source = HtmlSource::utf8("<html><div>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let mut input = tag(&mut tokenizer, &mut c);
    let node = kernel
        .session
        .allocate_html_tag(&mut c, &mut input)
        .unwrap();
    kernel.push(&node, &mut c).unwrap();
    assert_eq!(
        kernel.insert_element(&node, None, &mut c).unwrap(),
        ElementInsertionOutcome::Ignored(ElementInsertionIgnored::HostIncludingInclusiveAncestor)
    );
    assert!(kernel.session.parent(&mut c, &node).unwrap().is_none());
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn detached_template_owns_contents_for_host_including_ancestor_guard() {
    let source = HtmlSource::utf8("<html><template>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let mut input = tag(&mut tokenizer, &mut c);
    let template = kernel
        .session
        .allocate_html_tag(&mut c, &mut input)
        .unwrap();
    let contents = kernel
        .session
        .template_contents(&mut c, &template)
        .unwrap()
        .unwrap();
    assert!(kernel.session.parent(&mut c, &contents).unwrap().is_none());
    kernel.push(&template, &mut c).unwrap();
    let location = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &location.parent, &contents, &mut c);
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    assert_eq!(
        kernel.insert_element(&template, None, &mut c).unwrap(),
        ElementInsertionOutcome::Ignored(ElementInsertionIgnored::HostIncludingInclusiveAncestor)
    );
    assert!(kernel.session.parent(&mut c, &template).unwrap().is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        edges
    );
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ignored_element_push_cap_retains_owned_creation_and_original_terminal_stop() {
    let source = HtmlSource::utf8("<html><div id=x>");
    let mut c = control(1);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let document = kernel.session.root(&mut c).unwrap();
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    let mut input = tag(&mut tokenizer, &mut c);
    assert!(matches!(
        kernel.insert_html_tag(&mut input, StackPush::Push, Some(&document), &mut c),
        Err(KernelError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::OpenElements,
            ..
        }))
    ));
    // Creation and ignored insertion happen before the requested stack push.
    // A later stack rejection retains actual detached payload ownership.
    assert!(input.is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        1
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        edges
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        1
    );
    let stop = c.stop();
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
    assert_released(&c);
}

#[test]
fn cancelled_element_insertion_keeps_actual_token_before_creation() {
    let source = HtmlSource::utf8("<html><div id=x>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(100), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let document = kernel.session.root(&mut c).unwrap();
    let mut input = tag(&mut tokenizer, &mut c);
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    cancellation.cancel();
    assert!(matches!(
        kernel.insert_html_tag(&mut input, StackPush::Push, Some(&document), &mut c),
        Err(KernelError::Stop(ParseStop::Cancelled))
    ));
    assert!(input.is_some());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        edges
    );
    let stop = c.stop();
    // The caller still owns the rejected tag. Dispose its actual payloads
    // through the terminal-only owner seam, without resumed token processing.
    let mut retained = input.take().unwrap();
    retained.name.release(&mut c).unwrap();
    while let Some(attribute) = retained.attributes.cleanup_pop(&mut c).unwrap() {
        attribute.name.release(&mut c).unwrap();
        attribute.value.release(&mut c).unwrap();
    }
    retained.attributes.release(&mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_released(&c);
}

fn next_token(
    tokenizer: &mut Tokenizer<'_>,
    c: &mut ParseControl,
) -> crate::parser::tokenizer::token::Token {
    loop {
        match tokenizer
            .next(c, false)
            .unwrap()
            .expect("tokenized input reaches EOF")
        {
            Event::Token(event) => return event.into_token(c).unwrap(),
            event => event.release(c).unwrap(),
        }
    }
}

fn prologue(source: &HtmlSource, c: &mut ParseControl) -> super::prologue::BeforeHead {
    use super::prologue::{DocumentPrologue, PrologueStep};
    c.admit_source(source).unwrap();
    let mut tokenizer = Tokenizer::new(source, State::Data, None, c).unwrap();
    let mut engine = DocumentPrologue::new(source, c).unwrap();
    loop {
        let mut input = Some(next_token(&mut tokenizer, c));
        let step = engine.process(&mut input, c).unwrap();
        assert!(input.is_none(), "prologue owns the admitted lexical input");
        if step == PrologueStep::BeforeHead {
            break;
        }
    }
    tokenizer.release(c).unwrap();
    engine.take_before_head(c).unwrap()
}

fn observe_prologue(mut output: super::prologue::BeforeHead, c: &mut ParseControl) -> FrozenArena {
    if let Some(input) = output.input.take() {
        let mut kind = Some(input.kind);
        crate::construction::parser_session::release_token_kind(&mut kind, c).unwrap();
        assert!(kind.is_none());
    }
    while let Some(text) = output.whitespace.try_pop(c).unwrap() {
        text.release(c).unwrap();
    }
    output.whitespace.release(c).unwrap();
    output.diagnostics.release(c).unwrap();
    freeze(output.kernel, c)
}

#[test]
fn fresh_document_prologue_preserves_initial_noquirks_and_exact_doctype_fields() {
    use super::prologue::PrologueCondition;
    for (input, id) in [
        ("<!doctype html><html>", None),
        (
            "<!doctype html SYSTEM 'about:legacy-compat'><html>",
            Some("about:legacy-compat"),
        ),
    ] {
        let source = HtmlSource::utf8(input);
        let mut c = control(100);
        let output = prologue(&source, &mut c);
        assert!(output.diagnostics.as_slice().is_empty());
        assert!(output.input.is_none());
        assert_eq!(
            c.usage().resource(HtmlParseResource::OpenElements).current,
            1
        );
        let mut frozen = observe_prologue(output, &mut c);
        let document = frozen.document().unwrap();
        assert!(matches!(
            document.root().kind(),
            HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks)
        ));
        assert_eq!(document.root().children().len(), 2);
        let doctype = document
            .node(&document.root().children().next().unwrap())
            .unwrap();
        let HtmlNodeKind::Doctype(value) = doctype.kind() else {
            panic!("actual doctype");
        };
        assert_eq!(value.name().code_units(), [104, 116, 109, 108]);
        assert_eq!(value.public_id().unwrap().code_units(), []);
        assert_eq!(
            value.system_id().unwrap().code_units(),
            DocumentString::from(id.unwrap_or("")).code_units()
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
    // The pin removed upstream's old-public-ID exemption from parse errors.
    let source = HtmlSource::utf8("<!doctype html PUBLIC '-//W3C//DTD HTML 4.0//EN'><html>");
    let mut c = control(100);
    let output = prologue(&source, &mut c);
    assert_eq!(output.diagnostics.as_slice().len(), 1);
    assert_eq!(
        output.diagnostics.as_slice()[0].condition,
        PrologueCondition::InvalidDoctype
    );
    let mut frozen = observe_prologue(output, &mut c);
    assert!(matches!(
        frozen.document().unwrap().root().kind(),
        HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks)
    ));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn prologue_quirks_reconciles_silmaril_and_html4_empty_or_present_system() {
    use super::prologue::PrologueCondition;
    for (input, expected) in [
        (
            "<!doctype html PUBLIC '+//SiLmArIl//dtd html Pro v0r11 19970101//suffix'><html>",
            HtmlDocumentMode::Quirks,
        ),
        (
            "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN'><html>",
            HtmlDocumentMode::Quirks,
        ),
        (
            "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN' ''><html>",
            HtmlDocumentMode::Quirks,
        ),
        (
            "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN' 'x'><html>",
            HtmlDocumentMode::LimitedQuirks,
        ),
        (
            "<!doctype html PUBLIC '-//W3C//DTD XHTML 1.0 Frameset//EN'><html>",
            HtmlDocumentMode::LimitedQuirks,
        ),
        (
            "<!doctype html SYSTEM 'HTTP://WWW.IBM.COM/DATA/DTD/V11/IBMXHTML1-TRANSITIONAL.DTD'><html>",
            HtmlDocumentMode::Quirks,
        ),
        ("<!doctype other><html>", HtmlDocumentMode::Quirks),
    ] {
        let source = HtmlSource::utf8(input);
        let mut c = control(100);
        let output = prologue(&source, &mut c);
        assert_eq!(output.diagnostics.as_slice().len(), 1);
        assert_eq!(
            output.diagnostics.as_slice()[0].condition,
            PrologueCondition::InvalidDoctype
        );
        let mut frozen = observe_prologue(output, &mut c);
        assert!(
            matches!(frozen.document().unwrap().root().kind(), HtmlNodeKind::Document(actual) if *actual==expected),
            "{input}"
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn prologue_inserts_comment_and_recovered_pi_in_document_order_in_both_modes() {
    use super::prologue::{PrologueCondition, PrologueMode};
    let source = HtmlSource::utf8(" \r\n<!--a--><?x a??><!doctype html><?y b?><html/>");
    let mut c = control(100);
    let output = prologue(&source, &mut c);
    assert_eq!(output.diagnostics.as_slice().len(), 1);
    let diagnostic = &output.diagnostics.as_slice()[0];
    assert_eq!(
        diagnostic.condition,
        PrologueCondition::UnacknowledgedSelfClosing
    );
    assert_eq!(diagnostic.mode, PrologueMode::BeforeHtml);
    assert_eq!(
        source.text(&diagnostic.source).unwrap().code_units(),
        DocumentString::from("<html/>").code_units()
    );
    let whitespace: Vec<u16> = output
        .whitespace
        .as_slice()
        .iter()
        .flat_map(|text| text.value.text().code_units().iter().copied())
        .collect();
    assert_eq!(whitespace, [32, 10]);
    let mut frozen = observe_prologue(output, &mut c);
    let document = frozen.document().unwrap();
    let children: Vec<_> = document.root().children().collect();
    assert_eq!(children.len(), 5);
    assert!(
        matches!(document.node(&children[0]).unwrap().kind(), HtmlNodeKind::Comment(text) if text.code_units()==[97])
    );
    assert!(
        matches!(document.node(&children[1]).unwrap().kind(), HtmlNodeKind::ProcessingInstruction(pi) if pi.target().code_units()==[120] && pi.data().code_units()==[97,63])
    );
    assert!(matches!(
        document.node(&children[2]).unwrap().kind(),
        HtmlNodeKind::Doctype(_)
    ));
    assert!(
        matches!(document.node(&children[3]).unwrap().kind(), HtmlNodeKind::ProcessingInstruction(pi) if pi.target().code_units()==[121] && pi.data().code_units()==[98])
    );
    assert!(
        matches!(document.node(&children[4]).unwrap().kind(), HtmlNodeKind::Element(value) if value.name().local_name().code_units()==[104,116,109,108])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn absent_doctype_creates_actual_implicit_html_and_retains_exact_before_head_token() {
    use super::prologue::PrologueCondition;
    for input in ["<head>", "</head>", "</body>", "</html>", "</br>", ""] {
        let source = HtmlSource::utf8(input);
        let mut c = control(100);
        let output = prologue(&source, &mut c);
        assert_eq!(output.diagnostics.as_slice().len(), 1);
        assert_eq!(
            output.diagnostics.as_slice()[0].condition,
            PrologueCondition::MissingDoctype
        );
        let retained = output
            .input
            .as_ref()
            .expect("same token must be reprocessed by BeforeHead");
        assert_eq!(
            source.text(&retained.source).unwrap().code_units(),
            DocumentString::from(input).code_units()
        );
        assert!(matches!(&retained.kind, TokenKind::Tag(_) | TokenKind::Eof));
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            2
        );
        let mut frozen = observe_prologue(output, &mut c);
        let document = frozen.document().unwrap();
        assert!(matches!(
            document.root().kind(),
            HtmlNodeKind::Document(HtmlDocumentMode::Quirks)
        ));
        let html = document
            .node(&document.root().children().next().unwrap())
            .unwrap();
        assert!(
            matches!(html.kind(), HtmlNodeKind::Element(value) if value.name().local_name().code_units()==[104,116,109,108])
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn before_html_discards_other_end_tags_and_late_doctype_with_qualified_diagnostics() {
    use super::prologue::{PrologueCondition, PrologueMode};
    let source = HtmlSource::utf8("<!doctype html></p><!doctype other><html id=x>");
    let mut c = control(100);
    let output = prologue(&source, &mut c);
    assert_eq!(output.diagnostics.as_slice().len(), 2);
    for (diagnostic, condition, literal) in output
        .diagnostics
        .as_slice()
        .iter()
        .zip([
            PrologueCondition::UnexpectedEndTag,
            PrologueCondition::UnexpectedDoctype,
        ])
        .zip(["</p>", "<!doctype other>"])
        .map(|((a, b), c)| (a, b, c))
    {
        assert_eq!(diagnostic.condition, condition);
        assert_eq!(diagnostic.mode, PrologueMode::BeforeHtml);
        assert_eq!(
            source.text(&diagnostic.source).unwrap().code_units(),
            DocumentString::from(literal).code_units()
        );
        assert_eq!(diagnostic.source.source(), source.identity());
    }
    let mut frozen = observe_prologue(output, &mut c);
    let document = frozen.document().unwrap();
    assert_eq!(document.root().children().len(), 2);
    assert!(matches!(
        document.root().kind(),
        HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks)
    ));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn prologue_whitespace_split_retains_original_source_maps_and_isolated_surrogate_suffix() {
    let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
        32, 13, 10, 9, 0xd800, 0xd83d, 0xde42, 0xfeff,
    ]));
    let mut c = control(100);
    let output = prologue(&source, &mut c);
    let whitespace: Vec<u16> = output
        .whitespace
        .as_slice()
        .iter()
        .flat_map(|text| text.value.text().code_units().iter().copied())
        .collect();
    assert_eq!(whitespace, [32, 10, 9]);
    let mapped: Vec<_> = output
        .whitespace
        .as_slice()
        .iter()
        .flat_map(|text| text.origins.as_slice().iter())
        .map(|origin| (origin.source.start(), origin.source.end()))
        .collect();
    assert_eq!(mapped, [(0, 1), (1, 3), (3, 4)]);
    let retained = output.input.as_ref().unwrap();
    let TokenKind::Text(text) = &retained.kind else {
        panic!("original nonwhitespace text");
    };
    assert_eq!(text.value.text().code_units(), [0xd800]);
    let mapped: Vec<_> = text
        .origins
        .as_slice()
        .iter()
        .map(|origin| {
            (
                origin.start,
                origin.end,
                origin.source.start(),
                origin.source.end(),
            )
        })
        .collect();
    assert_eq!(mapped, [(0, 1, 4, 5)]);
    let mut frozen = observe_prologue(output, &mut c);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn cancelled_prologue_keeps_caller_token_and_allows_original_owner_cleanup() {
    use super::prologue::{DocumentPrologue, PrologueError};
    let source = HtmlSource::utf8("<html id=x>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(100), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let mut engine = DocumentPrologue::new(&source, &mut c).unwrap();
    let mut input = Some(next_token(&mut tokenizer, &mut c));
    cancellation.cancel();
    assert!(matches!(
        engine.process(&mut input, &mut c),
        Err(PrologueError::Kernel(KernelError::Stop(
            ParseStop::Cancelled
        )))
    ));
    assert!(input.is_some());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        1
    );
    let mut kind = Some(input.take().unwrap().kind);
    crate::construction::parser_session::release_token_kind(&mut kind, &mut c).unwrap();
    assert!(kind.is_none());
    tokenizer.release(&mut c).unwrap();
    let stop = c.stop();
    engine.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_released(&c);
}

#[test]
fn prologue_diagnostic_cap_stops_before_implicit_creation_and_releases_retained_tag() {
    use super::prologue::{DocumentPrologue, PrologueError};
    let source = HtmlSource::utf8("<div id=x>");
    let caps = limits(100).into_builder().diagnostics(0).finish().unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let mut engine = DocumentPrologue::new(&source, &mut c).unwrap();
    let mut input = Some(next_token(&mut tokenizer, &mut c));
    assert!(matches!(
        engine.process(&mut input, &mut c),
        Err(PrologueError::Kernel(KernelError::Stop(
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            }
        )))
    ));
    assert!(input.is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        1
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        0
    );
    let stop = c.stop();
    tokenizer.release(&mut c).unwrap();
    engine.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_released(&c);
}

#[test]
fn prologue_stack_limit_keeps_created_element_payload_owned_until_terminal_cleanup() {
    use super::prologue::{DocumentPrologue, PrologueError};
    let source = HtmlSource::utf8("<!doctype html><html id=x>");
    let mut c = control(0);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let mut engine = DocumentPrologue::new(&source, &mut c).unwrap();
    let mut first = Some(next_token(&mut tokenizer, &mut c));
    engine.process(&mut first, &mut c).unwrap();
    let mut input = Some(next_token(&mut tokenizer, &mut c));
    assert!(matches!(
        engine.process(&mut input, &mut c),
        Err(PrologueError::Kernel(KernelError::Stop(
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::OpenElements,
                ..
            }
        )))
    ));
    assert!(input.is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        1
    );
    let stop = c.stop();
    tokenizer.release(&mut c).unwrap();
    engine.release(&mut c).unwrap();
    assert_eq!(c.stop(), stop);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
    assert_released(&c);
}

#[test]
fn fresh_prologue_constructor_requires_actual_admitted_source_before_any_document_allocation() {
    use super::prologue::{DocumentPrologue, PrologueError};
    let source = HtmlSource::utf8("about:srcdoc");
    let mut c = control(100);
    assert!(matches!(
        DocumentPrologue::new(&source, &mut c),
        Err(PrologueError::Kernel(KernelError::Stop(
            ParseStop::InternalInvariant(Invariant::MissingSource)
        )))
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        0
    );
    assert_released(&c);
}

#[test]
fn ordinary_prologue_recognizes_every_normative_quirky_public_prefix_without_name_restriction() {
    // HTML 877d §13.2.6.4.1, independently transcribed public-prefix list.
    // Each actual token contains a suffix: this asserts prefix, not equality.
    for prefix in [
        "+//Silmaril//dtd html Pro v0r11 19970101//",
        "-//AS//DTD HTML 3.0 asWedit + extensions//",
        "-//AdvaSoft Ltd//DTD HTML 3.0 asWedit + extensions//",
        "-//IETF//DTD HTML 2.0 Level 1//",
        "-//IETF//DTD HTML 2.0 Level 2//",
        "-//IETF//DTD HTML 2.0 Strict Level 1//",
        "-//IETF//DTD HTML 2.0 Strict Level 2//",
        "-//IETF//DTD HTML 2.0 Strict//",
        "-//IETF//DTD HTML 2.0//",
        "-//IETF//DTD HTML 2.1E//",
        "-//IETF//DTD HTML 3.0//",
        "-//IETF//DTD HTML 3.2 Final//",
        "-//IETF//DTD HTML 3.2//",
        "-//IETF//DTD HTML 3//",
        "-//IETF//DTD HTML Level 0//",
        "-//IETF//DTD HTML Level 1//",
        "-//IETF//DTD HTML Level 2//",
        "-//IETF//DTD HTML Level 3//",
        "-//IETF//DTD HTML Strict Level 0//",
        "-//IETF//DTD HTML Strict Level 1//",
        "-//IETF//DTD HTML Strict Level 2//",
        "-//IETF//DTD HTML Strict Level 3//",
        "-//IETF//DTD HTML Strict//",
        "-//IETF//DTD HTML//",
        "-//Metrius//DTD Metrius Presentational//",
        "-//Microsoft//DTD Internet Explorer 2.0 HTML Strict//",
        "-//Microsoft//DTD Internet Explorer 2.0 HTML//",
        "-//Microsoft//DTD Internet Explorer 2.0 Tables//",
        "-//Microsoft//DTD Internet Explorer 3.0 HTML Strict//",
        "-//Microsoft//DTD Internet Explorer 3.0 HTML//",
        "-//Microsoft//DTD Internet Explorer 3.0 Tables//",
        "-//Netscape Comm. Corp.//DTD HTML//",
        "-//Netscape Comm. Corp.//DTD Strict HTML//",
        "-//O'Reilly and Associates//DTD HTML 2.0//",
        "-//O'Reilly and Associates//DTD HTML Extended 1.0//",
        "-//O'Reilly and Associates//DTD HTML Extended Relaxed 1.0//",
        "-//SQ//DTD HTML 2.0 HoTMetaL + extensions//",
        "-//SoftQuad Software//DTD HoTMetaL PRO 6.0::19990601::extensions to HTML 4.0//",
        "-//SoftQuad//DTD HoTMetaL PRO 4.0::19971010::extensions to HTML 4.0//",
        "-//Spyglass//DTD HTML 2.0 Extended//",
        "-//Sun Microsystems Corp.//DTD HotJava HTML//",
        "-//Sun Microsystems Corp.//DTD HotJava Strict HTML//",
        "-//W3C//DTD HTML 3 1995-03-24//",
        "-//W3C//DTD HTML 3.2 Draft//",
        "-//W3C//DTD HTML 3.2 Final//",
        "-//W3C//DTD HTML 3.2//",
        "-//W3C//DTD HTML 3.2S Draft//",
        "-//W3C//DTD HTML 4.0 Frameset//",
        "-//W3C//DTD HTML 4.0 Transitional//",
        "-//W3C//DTD HTML Experimental 19960712//",
        "-//W3C//DTD HTML Experimental 970421//",
        "-//W3C//DTD W3 HTML//",
        "-//W3O//DTD W3 HTML 3.0//",
        "-//WebTechs//DTD Mozilla HTML 2.0//",
        "-//WebTechs//DTD Mozilla HTML//",
    ] {
        let source = HtmlSource::utf8(format!("<!doctype html PUBLIC \"{prefix}suffix\"><html>"));
        let mut c = control(100);
        let output = prologue(&source, &mut c);
        assert_eq!(output.diagnostics.as_slice().len(), 1);
        let mut frozen = observe_prologue(output, &mut c);
        assert!(
            matches!(
                frozen.document().unwrap().root().kind(),
                HtmlNodeKind::Document(HtmlDocumentMode::Quirks)
            ),
            "{prefix}"
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn prologue_doctype_matching_preserves_nonascii_units_and_exact_system_error_case() {
    use super::prologue::PrologueCondition;
    for (input, error) in [
        ("<!doctype html SYSTEM 'ABOUT:LEGACY-COMPAT'><html>", true),
        ("<!doctype html SYSTEM 'about:legacy-compat'><html>", false),
        ("<!doctype html PUBLIC 'hTmL'><html>", true),
    ] {
        let source = HtmlSource::utf8(input);
        let mut c = control(100);
        let output = prologue(&source, &mut c);
        assert_eq!(output.diagnostics.as_slice().len(), usize::from(error));
        if error {
            assert_eq!(
                output.diagnostics.as_slice()[0].condition,
                PrologueCondition::InvalidDoctype
            );
        }
        let mut frozen = observe_prologue(output, &mut c);
        let expected = if input.contains("PUBLIC") {
            HtmlDocumentMode::Quirks
        } else {
            HtmlDocumentMode::NoQuirks
        };
        assert!(
            matches!(frozen.document().unwrap().root().kind(),HtmlNodeKind::Document(mode) if *mode==expected)
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
    let mut units: Vec<u16> = "<!doctype html PUBLIC '".encode_utf16().collect();
    units.extend([0xd800, 0xd83d, 0xde42, 0xfeff]);
    units.extend("'><html>".encode_utf16());
    let source = HtmlSource::utf16(DocumentString::from_utf16(units));
    let mut c = control(100);
    let output = prologue(&source, &mut c);
    assert_eq!(output.diagnostics.as_slice().len(), 1);
    let mut frozen = observe_prologue(output, &mut c);
    let document = frozen.document().unwrap();
    assert!(matches!(
        document.root().kind(),
        HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks)
    ));
    let doctype = document
        .node(&document.root().children().next().unwrap())
        .unwrap();
    assert!(
        matches!(doctype.kind(),HtmlNodeKind::Doctype(value) if value.public_id().unwrap().code_units()==[0xd800,0xd83d,0xde42,0xfeff])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ordinary_prologue_rejects_foreign_source_token_before_consuming_its_actual_payload() {
    use super::prologue::{DocumentPrologue, PrologueError};
    let first = HtmlSource::utf8("<!doctype html><html>");
    let second = HtmlSource::utf8("<html id=x>");
    let mut owner = control(100);
    owner.admit_source(&first).unwrap();
    let mut engine = DocumentPrologue::new(&first, &mut owner).unwrap();
    let mut foreign = control(100);
    foreign.admit_source(&second).unwrap();
    let mut tokenizer = Tokenizer::new(&second, State::Data, None, &mut foreign).unwrap();
    let mut input = Some(next_token(&mut tokenizer, &mut foreign));
    assert!(matches!(
        engine.process(&mut input, &mut owner),
        Err(PrologueError::Kernel(KernelError::Stop(
            ParseStop::InternalInvariant(Invariant::ForeignSource)
        )))
    ));
    assert!(input.is_some());
    assert_eq!(
        owner
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        1
    );
    assert_eq!(
        owner
            .usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        0
    );
    let mut kind = Some(input.take().unwrap().kind);
    crate::construction::parser_session::release_token_kind(&mut kind, &mut foreign).unwrap();
    tokenizer.release(&mut foreign).unwrap();
    engine.release(&mut owner).unwrap();
    assert_released(&owner);
    assert_released(&foreign);
}

#[test]
fn prologue_rejects_same_source_foreign_payload_before_ownership_transfer_or_mode_effect() {
    use super::prologue::{DocumentPrologue, PrologueError};
    let source = HtmlSource::utf8("<html id=x>");
    let mut owner = control(100);
    owner.admit_source(&source).unwrap();
    let mut engine = DocumentPrologue::new(&source, &mut owner).unwrap();
    let mut foreign = control(100);
    foreign.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut foreign).unwrap();
    let mut input = Some(next_token(&mut tokenizer, &mut foreign));
    let result = engine.process(&mut input, &mut owner);
    assert!(
        input.is_some(),
        "foreign payload must remain with its actual original owner: {result:?}"
    );
    assert!(matches!(
        result,
        Err(PrologueError::Kernel(
            KernelError::Stop(ParseStop::InternalInvariant(Invariant::ForeignControl))
                | KernelError::Session(SessionError::Stop(ParseStop::InternalInvariant(
                    Invariant::ForeignControl
                )))
        ))
    ));
    assert_eq!(
        owner
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        1
    );
    assert_eq!(
        owner
            .usage()
            .resource(HtmlParseResource::Diagnostics)
            .current,
        0
    );
    let mut kind = Some(input.take().unwrap().kind);
    crate::construction::parser_session::release_token_kind(&mut kind, &mut foreign).unwrap();
    tokenizer.release(&mut foreign).unwrap();
    engine.release(&mut owner).unwrap();
    assert_released(&owner);
    assert_released(&foreign);
}

#[test]
fn prologue_atomic_layout_cap_rejects_fixed_owner_before_native_document_allocation() {
    use super::prologue::{DocumentPrologue, PrologueError};
    let source = HtmlSource::utf8("<!doctype html><html>");
    let cap = (std::mem::size_of::<DocumentPrologue<'_>>() - 1) as u64;
    let caps = limits(100)
        .into_builder()
        .atomic_layout_bytes(cap)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    c.admit_source(&source).unwrap();
    assert!(matches!(
        DocumentPrologue::new(&source, &mut c),
        Err(PrologueError::Kernel(KernelError::Stop(
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::AtomicLayoutBytes,
                ..
            }
        )))
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        0
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        0
    );
    assert_released(&c);
}

#[test]
fn prologue_quirky_public_equality_and_prefix_cases_remain_distinct() {
    for (id, expected) in [
        (
            "-//W3O//DTD W3 HTML Strict 3.0//EN//",
            HtmlDocumentMode::Quirks,
        ),
        (
            "-/W3C/DTD HTML 4.0 Transitional/EN",
            HtmlDocumentMode::Quirks,
        ),
        ("HTML", HtmlDocumentMode::Quirks),
        ("HTMLsuffix", HtmlDocumentMode::NoQuirks),
        (
            "-//W3O//DTD W3 HTML Strict 3.0//EN//suffix",
            HtmlDocumentMode::NoQuirks,
        ),
        (
            "-//W3C//DTD XHTML 1.0 Transitional//EN",
            HtmlDocumentMode::LimitedQuirks,
        ),
        (
            "-//W3C//DTD HTML 4.01 Frameset//EN",
            HtmlDocumentMode::Quirks,
        ),
    ] {
        let source = HtmlSource::utf8(format!("<!doctype html PUBLIC '{id}'><html>"));
        let mut c = control(100);
        let output = prologue(&source, &mut c);
        let mut frozen = observe_prologue(output, &mut c);
        assert!(
            matches!(frozen.document().unwrap().root().kind(),HtmlNodeKind::Document(mode) if *mode==expected),
            "{id}"
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

/// Starts the real fresh prologue and passes its actual paused tokenizer to the
/// owning head stage. This observer releases pre-head lexer evidence explicitly;
/// no final diagnostic/report completeness is inferred from these private tests.
fn paused_before_head<'a>(
    source: &'a HtmlSource,
    c: &mut ParseControl,
) -> (super::prologue::BeforeHead, Tokenizer<'a>) {
    use super::prologue::{DocumentPrologue, PrologueStep};
    c.admit_source(source).unwrap();
    let mut tokenizer = Tokenizer::new(source, State::Data, None, c).unwrap();
    let mut prologue = DocumentPrologue::new(source, c).unwrap();
    loop {
        let mut token = Some(next_token(&mut tokenizer, c));
        if prologue.process(&mut token, c).unwrap() == PrologueStep::BeforeHead {
            break;
        }
    }
    (prologue.take_before_head(c).unwrap(), tokenizer)
}
fn head_stage<'a>(source: &'a HtmlSource, c: &mut ParseControl) -> super::head::HeadStage<'a> {
    let (before, tokenizer) = paused_before_head(source, c);
    super::head::HeadStage::new(source, &mut Some(before), &mut Some(tokenizer), c).unwrap()
}
fn head_boundary<'a>(
    source: &'a HtmlSource,
    c: &mut ParseControl,
) -> super::head::HeadContinuation<'a> {
    let mut driver = head_stage(source, c);
    for _ in 0..1_000 {
        if matches!(driver.step(c).unwrap(), super::head::HeadStep::Boundary(_)) {
            return driver.take_continuation(c).unwrap();
        }
    }
    panic!("finite fixture must reach an owned dispatch boundary")
}
fn observe_head(
    mut output: super::head::HeadContinuation<'_>,
    c: &mut ParseControl,
) -> FrozenArena {
    let mut state = output.driver.state.take().unwrap();
    if let Some(mut pending) = state.pending.take() {
        pending.release(c).unwrap();
    }
    if let Some(tokenizer) = state.tokenizer.take() {
        tokenizer.release(c).unwrap();
    }
    while let Some(event) = state.lexical.try_pop(c).unwrap() {
        event.release(c).unwrap();
    }
    while let Some(text) = state.before.whitespace.try_pop(c).unwrap() {
        text.release(c).unwrap();
    }
    state.before.whitespace.release(c).unwrap();
    state.before.diagnostics.release(c).unwrap();
    state.diagnostics.release(c).unwrap();
    state.lexical.release(c).unwrap();
    freeze(state.before.kernel, c)
}
fn head_names(document: &crate::HtmlDocument) -> Vec<DocumentString> {
    let html = document
        .node(&document.root().children().last().unwrap())
        .unwrap();
    let head = html.children().map(|id| document.node(&id).unwrap()).find(|node| matches!(node.kind(), HtmlNodeKind::Element(element) if element.name().local_name() == &DocumentString::from("head"))).unwrap();
    head.children()
        .filter_map(|id| match document.node(&id).unwrap().kind() {
            HtmlNodeKind::Element(element) => Some(element.name().local_name().clone()),
            _ => None,
        })
        .collect()
}
fn first_head_child_text(document: &crate::HtmlDocument) -> Vec<u16> {
    let html = document
        .node(&document.root().children().last().unwrap())
        .unwrap();
    let head = html.children().map(|id| document.node(&id).unwrap()).find(|node| matches!(node.kind(), HtmlNodeKind::Element(element) if element.name().local_name() == &DocumentString::from("head"))).unwrap();
    let element = document.node(&head.children().next().unwrap()).unwrap();
    element
        .children()
        .flat_map(|id| match document.node(&id).unwrap().kind() {
            HtmlNodeKind::Text(text) => text.code_units().to_vec(),
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn owning_head_places_prehead_comment_pi_and_head_void_children_in_source_order() {
    use super::head::{HeadBoundary, InsertionMode};
    let source = HtmlSource::utf8(
        "<!doctype html><html> <!--outside--><?x a??><head>\r\n<!--inside--><?y b?><base/><basefont><bgsound/><link/><meta charset=utf-16></head>",
    );
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    assert_eq!(output.boundary, HeadBoundary::AfterHead);
    let state = output.driver.state.as_ref().unwrap();
    assert_eq!(state.mode, InsertionMode::AfterHead);
    assert!(state.pending.is_none());
    assert!(state.diagnostics.as_slice().is_empty());
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        1
    );
    assert_eq!(
        state.before.whitespace.as_slice()[0]
            .value
            .text()
            .code_units(),
        [32]
    );
    assert!(
        state
            .lexical
            .as_slice()
            .iter()
            .any(|event| matches!(event, Event::Preprocessing { .. }))
    );
    let head_id = state.head.as_ref().unwrap();
    assert!(state.before.kernel.named(head_id, "head", &mut c).unwrap());
    let mut frozen = observe_head(output, &mut c);
    let document = frozen.document().unwrap();
    assert_eq!(
        head_names(document),
        ["base", "basefont", "bgsound", "link", "meta"].map(DocumentString::from)
    );
    let html = document
        .node(&document.root().children().last().unwrap())
        .unwrap();
    let kinds: Vec<_> = html
        .children()
        .map(|id| document.node(&id).unwrap().kind())
        .collect();
    assert!(
        matches!(kinds[0], HtmlNodeKind::Comment(value) if value.code_units()==DocumentString::from("outside").code_units())
    );
    assert!(
        matches!(kinds[1], HtmlNodeKind::ProcessingInstruction(value) if value.target().code_units()==[120] && value.data().code_units()==[97,63])
    );
    let head = html.children().map(|id| document.node(&id).unwrap()).find(|node| matches!(node.kind(), HtmlNodeKind::Element(element) if element.name().local_name() == &DocumentString::from("head"))).unwrap();
    assert!(
        matches!(document.node(&head.children().next().unwrap()).unwrap().kind(), HtmlNodeKind::Text(value) if value.code_units()==[10])
    );
    assert!(matches!(
        document.root().kind(),
        HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks)
    ));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn implicit_head_reprocesses_nonhead_tag_without_consuming_its_payload() {
    use super::head::{HeadBoundary, InsertionMode};
    let source = HtmlSource::utf8("<body x='kept'>tail");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    assert_eq!(output.boundary, HeadBoundary::AfterHead);
    let state = output.driver.state.as_ref().unwrap();
    assert_eq!(state.mode, InsertionMode::AfterHead);
    let pending = state.pending.as_ref().unwrap();
    assert_eq!(
        source.text(&pending.source).unwrap().code_units(),
        DocumentString::from("<body x='kept'>").code_units()
    );
    let super::prologue::Payload::Tag(Some(tag)) = &pending.payload else {
        panic!("actual unconsumed body");
    };
    assert_eq!(
        tag.attributes.as_slice()[0].value.value.text().code_units(),
        DocumentString::from("kept").code_units()
    );
    let mut frozen = observe_head(output, &mut c);
    assert!(head_names(frozen.document().unwrap()).is_empty());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn head_title_switches_actual_rcdata_and_recovers_mismatched_end_tag_as_text() {
    let source = HtmlSource::utf8("<title>A&amp;<b></style>X</title><meta/></head>");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let mut frozen = observe_head(output, &mut c);
    let document = frozen.document().unwrap();
    assert_eq!(
        head_names(document),
        [DocumentString::from("title"), DocumentString::from("meta")]
    );
    assert_eq!(
        first_head_child_text(document),
        DocumentString::from("A&<b></style>X").code_units()
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn head_rawtext_preserves_references_isolates_pairs_and_state_specific_null_recovery() {
    let mut units = DocumentString::from("<style>&amp;<b>")
        .code_units()
        .to_vec();
    units.extend([0xD800, 0xD83D, 0xDE00, 0, 0xFEFF]);
    units.extend_from_slice(DocumentString::from("</style></head>").code_units());
    let source = HtmlSource::utf16(DocumentString::from_utf16(units));
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let state = output.driver.state.as_ref().unwrap();
    assert!(state.lexical.as_slice().iter().any(|event| matches!(event, Event::Diagnostic(records) if records.as_slice().iter().any(|d| d.code==crate::parser::tokenizer::token::DiagnosticCode::UnexpectedNullCharacter))));
    let mut frozen = observe_head(output, &mut c);
    let mut expected = DocumentString::from("&amp;<b>").code_units().to_vec();
    expected.extend([0xD800, 0xD83D, 0xDE00, 0xFFFD, 0xFEFF]);
    assert_eq!(first_head_child_text(frozen.document().unwrap()), expected);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn disabled_head_noscript_style_returns_to_noscript_then_ignores_nested_noscript() {
    use super::head::{HeadCondition, InsertionMode};
    let source = HtmlSource::utf8(
        "<head><noscript><style>&amp;</style><noscript><!--in--><?x d?><meta/><link/></noscript><title>T</title></head>",
    );
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let diagnostics = output.driver.state.as_ref().unwrap().diagnostics.as_slice();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].condition,
        HeadCondition::UnexpectedInNoscript
    );
    assert_eq!(diagnostics[0].mode, InsertionMode::InHeadNoscript);
    let mut frozen = observe_head(output, &mut c);
    let doc = frozen.document().unwrap();
    assert_eq!(
        head_names(doc),
        [
            DocumentString::from("noscript"),
            DocumentString::from("title")
        ]
    );
    let html = doc.node(&doc.root().children().last().unwrap()).unwrap();
    let head = doc.node(&html.children().last().unwrap()).unwrap();
    let noscript = doc.node(&head.children().next().unwrap()).unwrap();
    let names: Vec<_> = noscript
        .children()
        .filter_map(|id| match doc.node(&id).unwrap().kind() {
            HtmlNodeKind::Element(e) => Some(e.name().local_name().clone()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["style", "meta", "link"].map(DocumentString::from));
    drop(names);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn head_text_eof_emits_error_restores_mode_and_retains_original_eof_for_afterhead() {
    use super::head::{HeadCondition, InsertionMode};
    let source = HtmlSource::utf8("<title>unfinished&amp;");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let state = output.driver.state.as_ref().unwrap();
    let d = &state.diagnostics.as_slice()[0];
    assert_eq!(d.condition, HeadCondition::EofInText);
    assert_eq!(d.mode, InsertionMode::Text);
    assert_eq!(source.text(&d.source).unwrap().code_units(), []);
    let pending = state.pending.as_ref().unwrap();
    assert!(matches!(pending.payload, super::prologue::Payload::Eof));
    assert_eq!(pending.source.start(), source.len());
    assert_eq!(pending.source.end(), source.len());
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        1
    );
    let mut frozen = observe_head(output, &mut c);
    assert_eq!(
        first_head_child_text(frozen.document().unwrap()),
        DocumentString::from("unfinished&").code_units()
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn script_and_template_starts_suspend_untouched_and_forbid_further_pulls() {
    use super::head::{HeadBoundary, HeadError};
    for (input, boundary) in [
        ("<script a='v'>run</script>", HeadBoundary::Script),
        (
            "<template for='x' shadowrootmode=open>inside</template>",
            HeadBoundary::Template,
        ),
    ] {
        let source = HtmlSource::utf8(input);
        let mut c = control(100);
        let mut output = head_boundary(&source, &mut c);
        assert_eq!(output.boundary, boundary);
        let state = output.driver.state.as_ref().unwrap();
        assert_eq!(
            c.usage().resource(HtmlParseResource::OpenElements).current,
            2
        );
        let super::prologue::Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload
        else {
            panic!("real suspended tag");
        };
        assert!(tag.attributes.len() > 0);
        let credits = c.usage().work(WorkCategory::InputUnitRead);
        assert!(matches!(
            output.driver.step(&mut c),
            Err(HeadError::AlreadyAtBoundary)
        ));
        assert_eq!(c.usage().work(WorkCategory::InputUnitRead), credits);
        let mut frozen = observe_head(output, &mut c);
        assert!(head_names(frozen.document().unwrap()).is_empty());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn html_head_delegation_merges_missing_root_attributes_and_retains_duplicate_recovery() {
    use super::head::HeadCondition;
    let source = HtmlSource::utf8(
        "<!doctype html><html a=first><html a=second b=before><head><html b=later c=head c=drop><noscript><html d=noscript></noscript></head>",
    );
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let state = output.driver.state.as_ref().unwrap();
    assert_eq!(
        state
            .diagnostics
            .as_slice()
            .iter()
            .filter(|d| d.condition == HeadCondition::UnexpectedHtml)
            .count(),
        3
    );
    assert!(state.lexical.as_slice().iter().any(|event|matches!(event, Event::Diagnostic(records) if records.as_slice().iter().any(|d|d.code==crate::parser::tokenizer::token::DiagnosticCode::DuplicateAttribute))));
    let mut frozen = observe_head(output, &mut c);
    let doc = frozen.document().unwrap();
    let html = doc.node(&doc.root().children().last().unwrap()).unwrap();
    let HtmlNodeKind::Element(html) = html.kind() else {
        panic!("html");
    };
    let attrs: Vec<_> = html
        .attributes()
        .iter()
        .map(|a| (a.name().local_name().clone(), a.value().clone()))
        .collect();
    assert_eq!(
        attrs,
        [
            ("a", "first"),
            ("b", "before"),
            ("c", "head"),
            ("d", "noscript")
        ]
        .map(|(a, b)| (DocumentString::from(a), DocumentString::from(b)))
    );
    drop(attrs);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn beforehead_and_inhead_ignore_erroneous_doctype_endtags_and_nested_head() {
    use super::head::HeadCondition;
    let source = HtmlSource::utf8(
        "<!doctype html><html><!doctype other></title><head><head></foo></template><meta/></head>",
    );
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let conditions: Vec<_> = output
        .driver
        .state
        .as_ref()
        .unwrap()
        .diagnostics
        .as_slice()
        .iter()
        .map(|d| d.condition)
        .collect();
    assert_eq!(
        conditions,
        [
            HeadCondition::UnexpectedDoctype,
            HeadCondition::UnexpectedEndTag,
            HeadCondition::UnexpectedHead,
            HeadCondition::UnexpectedEndTag,
            HeadCondition::UnexpectedEndTag
        ]
    );
    let mut frozen = observe_head(output, &mut c);
    assert_eq!(
        head_names(frozen.document().unwrap()),
        [DocumentString::from("meta")]
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn mixed_head_whitespace_inserts_prefix_before_reprocessing_suffix_afterhead() {
    use super::head::HeadBoundary;
    let source = HtmlSource::utf8("<html> \r\n<head> \r\nX<body>");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    assert_eq!(output.boundary, HeadBoundary::AfterHead);
    let state = output.driver.state.as_ref().unwrap();
    let ignored: Vec<_> = state
        .before
        .whitespace
        .as_slice()
        .iter()
        .flat_map(|text| text.value.text().code_units().iter().copied())
        .collect();
    assert_eq!(ignored, [32, 10]);
    let super::prologue::Payload::Text(Some(text)) = &state.pending.as_ref().unwrap().payload
    else {
        panic!("actual pending suffix");
    };
    assert_eq!(text.value.text().code_units(), [88]);
    assert_eq!(
        source
            .text(&text.origins.as_slice()[0].source)
            .unwrap()
            .code_units(),
        [88]
    );
    let mut frozen = observe_head(output, &mut c);
    let doc = frozen.document().unwrap();
    let html = doc.node(&doc.root().children().last().unwrap()).unwrap();
    let head = doc.node(&html.children().last().unwrap()).unwrap();
    assert!(
        matches!(doc.node(&head.children().next().unwrap()).unwrap().kind(),HtmlNodeKind::Text(t) if t.code_units()==[32,10])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn head_open_element_cap_stops_before_raw_state_success_and_cleans_all_actual_owners() {
    use super::head::InsertionMode;
    let source = HtmlSource::utf8("<!doctype html><html><head><title>x</title>");
    let mut c = control(2); // html + head fit; the mandatory text-element push does not.
    let mut driver = head_stage(&source, &mut c);
    driver.step(&mut c).unwrap();
    assert!(driver.step(&mut c).is_err());
    assert!(c.stop().is_some());
    assert_eq!(driver.state.as_ref().unwrap().mode, InsertionMode::InHead);
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        2
    );
    driver.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn head_cancellation_precedes_next_cursor_read_and_keeps_state_for_terminal_release() {
    let source = HtmlSource::utf8("<html><head><title>later");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(100), cancellation.clone());
    let mut driver = head_stage(&source, &mut c);
    driver.step(&mut c).unwrap();
    let open = c.usage().resource(HtmlParseResource::OpenElements).current;
    cancellation.cancel();
    assert!(driver.step(&mut c).is_err());
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        open
    );
    driver.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn head_handoff_rejects_changed_tokenizer_state_without_taking_either_owner() {
    use super::head::{HeadError, HeadStage};
    use crate::parser::tokenizer::{TokenizerHandoffError, states::Raw};
    let source = HtmlSource::utf8("<html><head>");
    let mut c = control(100);
    let (before, mut tokenizer) = paused_before_head(&source, &mut c);
    tokenizer
        .set_state(State::Raw(Raw::Rcdata), &mut c)
        .unwrap();
    let mut before = Some(before);
    let mut tokenizer = Some(tokenizer);
    assert!(matches!(
        HeadStage::new(&source, &mut before, &mut tokenizer, &mut c),
        Err(HeadError::Handoff(TokenizerHandoffError::NotData))
    ));
    assert!(before.is_some());
    assert!(tokenizer.is_some());
    let mut frozen = observe_prologue(before.take().unwrap(), &mut c);
    tokenizer.take().unwrap().release(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn head_handoff_rejects_foreign_controller_before_effects_and_preserves_owned_inputs() {
    use super::head::{HeadError, HeadStage};
    let source = HtmlSource::utf8("<html><head>");
    let mut c = control(100);
    let (before, tokenizer) = paused_before_head(&source, &mut c);
    let mut before = Some(before);
    let mut tokenizer = Some(tokenizer);
    let mut other = control(100);
    other.admit_source(&source).unwrap();
    assert!(matches!(
        HeadStage::new(&source, &mut before, &mut tokenizer, &mut other),
        Err(HeadError::Kernel(KernelError::Stop(
            ParseStop::InternalInvariant(Invariant::ForeignControl)
        )))
    ));
    assert!(before.is_some());
    assert!(tokenizer.is_some());
    let mut frozen = observe_prologue(before.take().unwrap(), &mut c);
    tokenizer.take().unwrap().release(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn selfclosing_text_and_noscript_starttags_remain_open_and_emit_unacknowledged_errors() {
    use super::head::HeadCondition;
    let source =
        HtmlSource::utf8("<title/>x</title><style/>y</style><noscript/></noscript><meta/></head>");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let diagnostics = output.driver.state.as_ref().unwrap().diagnostics.as_slice();
    assert_eq!(diagnostics.len(), 3);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.condition == HeadCondition::UnacknowledgedSelfClosing)
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(|d| source.text(&d.source).unwrap())
            .collect::<Vec<_>>(),
        ["<title/>", "<style/>", "<noscript/>"].map(DocumentString::from)
    );
    let mut frozen = observe_head(output, &mut c);
    assert_eq!(
        head_names(frozen.document().unwrap()),
        ["title", "style", "noscript", "meta"].map(DocumentString::from)
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn disabled_noscript_default_closes_then_reprocesses_br_as_afterhead_input() {
    use super::head::{HeadCondition, InsertionMode};
    let source = HtmlSource::utf8("<head><noscript></foo></br>tail");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let state = output.driver.state.as_ref().unwrap();
    assert_eq!(state.diagnostics.as_slice().len(), 2);
    assert!(
        state
            .diagnostics
            .as_slice()
            .iter()
            .all(|d| d.condition == HeadCondition::UnexpectedInNoscript
                && d.mode == InsertionMode::InHeadNoscript)
    );
    let pending = state.pending.as_ref().unwrap();
    assert_eq!(
        source.text(&pending.source).unwrap(),
        DocumentString::from("</br>")
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        1
    );
    let mut frozen = observe_head(output, &mut c);
    assert_eq!(
        head_names(frozen.document().unwrap()),
        [DocumentString::from("noscript")]
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn eof_and_permitted_beforehead_endtags_create_head_with_exact_continuation_input() {
    use super::head::HeadBoundary;
    for (input, pending) in [
        ("", Some("")),
        ("<html></head>", None),
        ("<html></body>", Some("</body>")),
        ("<html></html>", Some("</html>")),
        ("<html></br>", Some("</br>")),
    ] {
        let source = HtmlSource::utf8(input);
        let mut c = control(100);
        let output = head_boundary(&source, &mut c);
        assert_eq!(output.boundary, HeadBoundary::AfterHead);
        let state = output.driver.state.as_ref().unwrap();
        assert_eq!(
            state
                .pending
                .as_ref()
                .map(|p| source.text(&p.source).unwrap()),
            pending.map(DocumentString::from)
        );
        assert!(state.diagnostics.as_slice().is_empty());
        let mut frozen = observe_head(output, &mut c);
        assert!(head_names(frozen.document().unwrap()).is_empty());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn head_diagnostic_cap_preserves_pending_tag_first_stop_and_terminal_ownership() {
    use super::head::{HeadError, InsertionMode};
    let source = HtmlSource::utf8("<!doctype html><html><head></oops>");
    let mut c = ParseControl::new(
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
        HtmlParseCancellation::new(),
    );
    let mut driver = head_stage(&source, &mut c);
    driver.step(&mut c).unwrap();
    assert!(matches!(
        driver.step(&mut c),
        Err(HeadError::Kernel(KernelError::Stop(
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            }
        )))
    ));
    assert_eq!(driver.state.as_ref().unwrap().mode, InsertionMode::InHead);
    assert!(matches!(
        &driver
            .state
            .as_ref()
            .unwrap()
            .pending
            .as_ref()
            .unwrap()
            .payload,
        super::prologue::Payload::Tag(Some(_))
    ));
    let first = c.stop();
    let reads = c.usage().work(WorkCategory::InputUnitRead);
    let writes = c.usage().work(WorkCategory::RecordWrite);
    assert!(driver.step(&mut c).is_err());
    assert_eq!(c.stop(), first);
    assert_eq!(c.usage().work(WorkCategory::InputUnitRead), reads);
    assert_eq!(c.usage().work(WorkCategory::RecordWrite), writes);
    driver.release(&mut c).unwrap();
    assert_eq!(c.stop(), first);
    assert_released(&c);
}

#[test]
fn head_release_rejects_foreign_controller_without_consuming_driver_owners() {
    let source = HtmlSource::utf8("<html><head><style>x</style>");
    let mut c = control(100);
    let mut driver = head_stage(&source, &mut c);
    driver.step(&mut c).unwrap();
    let mut other = control(100);
    assert!(driver.release(&mut other).is_err());
    assert!(driver.state.is_some());
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        2
    );
    driver.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn actual_template_stack_predicate_distinguishes_foreign_template_from_html_template() {
    let source = HtmlSource::utf8("<html><template>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let foreign = preexisting(&mut kernel.session, Namespace::svg(), "template", &mut c);
    kernel.push(&foreign, &mut c).unwrap();
    assert!(!kernel.contains_html_template(&mut c).unwrap());
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    assert!(kernel.contains_html_template(&mut c).unwrap());
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

fn script_text_node(driver: &mut StandaloneDocumentDriver<'_>) -> ParserNode {
    for _ in 0..1_000 {
        driver.step().unwrap();
        if let Some(head) = &driver.head
            && head.state.as_ref().unwrap().mode == super::head::InsertionMode::Text
        {
            let kernel = &head.state.as_ref().unwrap().before.kernel;
            let node = kernel
                .at(kernel.stack().unwrap().len() - 1, &mut driver.control)
                .unwrap();
            assert!(kernel.named(&node, "script", &mut driver.control).unwrap());
            return node;
        }
    }
    panic!("actual script fixture must enter Text");
}
fn standalone_boundary(driver: &mut StandaloneDocumentDriver<'_>) -> super::head::HeadBoundary {
    for _ in 0..1_000 {
        if let super::head::HeadStep::Boundary(boundary) = driver.step().unwrap() {
            return boundary;
        }
    }
    panic!("finite actual source must reach an unfinished dispatch boundary");
}

#[test]
fn standalone_script_start_retains_actual_document_and_initial_metadata_before_preparation() {
    use crate::construction::parser_session::ScriptResult;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script src='x'>body</script>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    let (c, session, capability) = driver.standalone_parts().unwrap();
    capability.validate_session(c, session).unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(!metadata.force_async);
    assert!(!metadata.already_started);
    assert_eq!(metadata.script_type, None);
    assert!(!metadata.external_file);
    assert!(!metadata.ready_to_be_parser_executed);
    assert!(!metadata.delaying_load);
    assert_eq!(metadata.result, ScriptResult::Uninitialized);
    assert_eq!(metadata.steps_when_result_ready, None);
    let root = session.root(c).unwrap();
    let parser_document = session.script_parser_document(c, &node).unwrap().unwrap();
    assert!(session.same_node(c, &root, &parser_document).unwrap());
    assert!(
        session
            .script_preparation_document(c, &node)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        driver
            .head
            .as_ref()
            .unwrap()
            .script_observation(&mut driver.control)
            .unwrap(),
        (0, false, false, false, false)
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn standalone_script_end_prepares_disabled_classic_and_restores_head_and_insertion_state() {
    use crate::construction::parser_session::ScriptType;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script><meta name=m></head><body>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (c, session, _) = driver.standalone_parts().unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(metadata.already_started);
    assert_eq!(metadata.script_type, Some(ScriptType::Classic));
    assert!(!metadata.force_async);
    let root = session.root(c).unwrap();
    let parser = session.script_parser_document(c, &node).unwrap().unwrap();
    let prepared = session
        .script_preparation_document(c, &node)
        .unwrap()
        .unwrap();
    assert!(session.same_node(c, &root, &parser).unwrap());
    assert!(session.same_node(c, &root, &prepared).unwrap());
    assert_eq!(
        driver
            .head
            .as_ref()
            .unwrap()
            .script_observation(&mut driver.control)
            .unwrap(),
        (0, false, false, false, false)
    );
    let mut owning = Some(driver);
    let (mut c, mut session, capability) =
        StandaloneDocumentDriver::take_standalone_fixture(&mut owning).unwrap();
    assert!(owning.is_none());
    capability.validate_session(&mut c, &session).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    assert_eq!(frozen.script_metadata(&mut c, &node).unwrap(), metadata);
    assert_eq!(
        head_names(frozen.document().unwrap()),
        vec![DocumentString::from("script"), DocumentString::from("meta")]
    );
    assert_eq!(
        first_head_child_text(frozen.document().unwrap()),
        vec![b'x' as u16]
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn standalone_repeated_script_endings_restore_actual_nesting_pause_and_current_return_mode() {
    use crate::construction::parser_session::ScriptType;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script><script type=module>y</script></head>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let first = script_text_node(&mut driver);
    for _ in 0..100 {
        driver.step().unwrap();
        if driver.head.as_ref().unwrap().state.as_ref().unwrap().mode
            == super::head::InsertionMode::InHead
        {
            break;
        }
    }
    assert_eq!(
        driver
            .head
            .as_ref()
            .unwrap()
            .script_observation(&mut driver.control)
            .unwrap(),
        (0, false, false, false, false)
    );
    let second = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    assert_eq!(
        driver
            .head
            .as_ref()
            .unwrap()
            .script_observation(&mut driver.control)
            .unwrap(),
        (0, false, false, false, false)
    );
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(!session.same_node(c, &first, &second).unwrap());
    let first = session.script_metadata(c, &first).unwrap();
    let second = session.script_metadata(c, &second).unwrap();
    assert!(first.already_started && second.already_started);
    assert_eq!(first.script_type, Some(ScriptType::Classic));
    assert_eq!(second.script_type, Some(ScriptType::Module));
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn standalone_script_eof_marks_started_without_preparing_or_changing_initial_facts() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script type=module>x"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let head = driver.head.as_ref().unwrap();
    let state = head.state.as_ref().unwrap();
    let diagnostics = state.diagnostics.as_slice();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].condition,
        super::head::HeadCondition::EofInText
    );
    assert_eq!(
        (diagnostics[0].source.start(), diagnostics[0].source.end()),
        (21, 21)
    );
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        super::prologue::Payload::Eof
    ));
    let (c, session, _) = driver.standalone_parts().unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(metadata.already_started);
    assert!(!metadata.force_async);
    assert_eq!(metadata.script_type, None);
    assert!(session.script_parser_document(c, &node).unwrap().is_some());
    assert!(
        session
            .script_preparation_document(c, &node)
            .unwrap()
            .is_none()
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn standalone_script_preparation_preserves_empty_unknown_and_disabled_type_distinctions() {
    use crate::construction::parser_session::ScriptType;
    for (attrs, body, started, force_async, kind, has_document) in [
        ("", "", false, true, None, false),
        ("async", "", false, false, None, false),
        ("type=unknown", "x", false, true, None, false),
        ("type=' module '", "x", false, true, None, false),
        (
            "src='' type=MODULE",
            "",
            true,
            false,
            Some(ScriptType::Module),
            true,
        ),
        (
            "type=importmap",
            "x",
            true,
            false,
            Some(ScriptType::ImportMap),
            true,
        ),
        (
            "type=speculationrules",
            "x",
            true,
            false,
            Some(ScriptType::SpeculationRules),
            true,
        ),
        (
            "type=' text/JAVASCRIPT '",
            "x",
            true,
            false,
            Some(ScriptType::Classic),
            true,
        ),
        (
            "type='text/javascript; charset=utf-8'",
            "x",
            false,
            true,
            None,
            false,
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!("<script {attrs}>{body}</script></head>")),
            limits(100),
        );
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let node = script_text_node(&mut driver);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        let (c, session, _) = driver.standalone_parts().unwrap();
        let metadata = session.script_metadata(c, &node).unwrap();
        assert_eq!(
            (
                metadata.already_started,
                metadata.force_async,
                metadata.script_type
            ),
            (started, force_async, kind),
            "{attrs}"
        );
        assert_eq!(
            session.script_parser_document(c, &node).unwrap().is_some(),
            has_document,
            "{attrs}"
        );
        assert_eq!(
            session
                .script_preparation_document(c, &node)
                .unwrap()
                .is_some(),
            started,
            "{attrs}"
        );
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn standalone_scriptdata_preserves_literal_utf16_and_escaped_script_content() {
    let mut units: Vec<_> = "<script><!--<script>".encode_utf16().collect();
    units.extend([0xd800, 0xd83d, 0xde80]);
    units.extend("&amp;</script>--></script></head>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let mut owning = Some(driver);
    let (mut c, mut session, _) =
        StandaloneDocumentDriver::take_standalone_fixture(&mut owning).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    assert!(
        frozen
            .script_metadata(&mut c, &node)
            .unwrap()
            .already_started
    );
    let mut expected: Vec<_> = "<!--<script>".encode_utf16().collect();
    expected.extend([0xd800, 0xd83d, 0xde80]);
    expected.extend("&amp;</script>-->".encode_utf16());
    assert_eq!(first_head_child_text(frozen.document().unwrap()), expected);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn standalone_scriptdata_preserves_newline_origins_and_state_specific_null_recovery() {
    let mut units: Vec<_> = "<script>\r\n".encode_utf16().collect();
    units.extend([0xd800, 0, 0xdc00, 0xd83d, 0xde80]);
    units.extend("</script></head>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    let null_error = state
        .lexical
        .as_slice()
        .iter()
        .find_map(|event| match event {
            Event::Diagnostic(diagnostics) => diagnostics.as_slice().iter().find(|diagnostic| {
                diagnostic.code
                    == crate::parser::tokenizer::token::DiagnosticCode::UnexpectedNullCharacter
            }),
            _ => None,
        })
        .expect("actual ScriptData NULL diagnostic");
    assert_eq!(null_error.source.source(), request.source().identity());
    assert_eq!(
        (null_error.source.start(), null_error.source.end()),
        (11, 12)
    );
    let mut owning = Some(driver);
    let (mut c, mut session, _) =
        StandaloneDocumentDriver::take_standalone_fixture(&mut owning).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    assert_eq!(
        first_head_child_text(frozen.document().unwrap()),
        [10, 0xd800, 0xfffd, 0xdc00, 0xd83d, 0xde80]
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn standalone_script_import_revocation_stops_required_end_checkpoint_before_pop_or_prepare() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script><meta>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    driver.step().unwrap(); // actual script character token
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(!session.script_metadata(c, &node).unwrap().already_started);
    preexisting(session, Namespace::html(), "imported", c);
    let error = driver.step().unwrap_err();
    assert!(matches!(
        error,
        StandaloneError::Head(super::head::HeadError::Kernel(KernelError::Session(
            SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                StandaloneUnsupported::Provenance
            ))
        )))
    ));
    let stop = driver.control.stop().unwrap();
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::Text);
    assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        super::prologue::Payload::Tag(Some(_))
    ));
    let work = driver.control.usage().work(WorkCategory::InputUnitRead);
    assert!(matches!(driver.step(), Err(StandaloneError::Stop(reason)) if reason == stop));
    assert_eq!(
        driver.control.usage().work(WorkCategory::InputUnitRead),
        work
    );
    assert!(
        driver
            .head
            .as_mut()
            .unwrap()
            .take_continuation(&mut driver.control)
            .is_err()
    );
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(matches!(session.freeze(c), Err(SessionError::Stop(reason)) if reason == stop));
    let mut owning = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut owning).is_err());
    assert!(owning.is_some());
    let driver = owning.as_mut().unwrap();
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), Some(stop));
    assert_released(&driver.control);
}

#[test]
fn standalone_script_cancellation_is_sticky_before_next_input_and_cleanup_has_no_parse_work() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    script_text_node(&mut driver);
    request.cancellation().cancel();
    let reads = driver.control.usage().work(WorkCategory::InputUnitRead);
    assert!(matches!(
        driver.step(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert!(matches!(
        driver.step(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert_eq!(
        driver.control.usage().work(WorkCategory::InputUnitRead),
        reads
    );
    let work = driver.control.usage().clone();
    driver.release().unwrap();
    for category in [
        WorkCategory::InputUnitRead,
        WorkCategory::RecordVisit,
        WorkCategory::RecordWrite,
        WorkCategory::CopyByte,
        WorkCategory::InspectByte,
        WorkCategory::ProviderCall,
        WorkCategory::StateStep,
        WorkCategory::ReserveByte,
    ] {
        assert_eq!(driver.control.usage().work(category), work.work(category));
    }
    assert_released(&driver.control);
}

#[test]
fn standalone_template_continuation_preserves_unconsumed_token_and_has_no_script_metadata_effect() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script><template data-x=q>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::Template
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    let super::prologue::Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload else {
        panic!("actual retained template token")
    };
    assert_eq!(
        tag.name.value.text().code_units(),
        [116, 101, 109, 112, 108, 97, 116, 101]
    );
    assert_eq!(tag.attributes.as_slice().len(), 1);
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(session.script_metadata(c, &node).unwrap().already_started);
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn tokenizer_next_input_position_is_nonconsuming_and_uses_real_utf8_crlf_and_utf16_coordinates() {
    for (source, reads) in [
        (HtmlSource::utf8("<script></script>\r\n\u{1f680}"), 7),
        (
            HtmlSource::utf16(DocumentString::from("<script></script>\r\n\u{1f680}")),
            2,
        ),
    ] {
        let mut c = control(100);
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
        let mut start = Some(next_kind(&mut tokenizer, &mut c));
        crate::construction::parser_session::release_token_kind(&mut start, &mut c).unwrap();
        tokenizer
            .set_state(
                crate::parser::tokenizer::states::State::Raw(
                    crate::parser::tokenizer::states::Raw::Script,
                ),
                &mut c,
            )
            .unwrap();
        let mut end = Some(next_kind(&mut tokenizer, &mut c));
        crate::construction::parser_session::release_token_kind(&mut end, &mut c).unwrap();
        let before = c.usage().clone();
        let position = tokenizer
            .checked_next_input_position(&source, &mut c)
            .unwrap();
        assert_eq!(
            c.usage().work(WorkCategory::InputUnitRead) - before.work(WorkCategory::InputUnitRead),
            reads
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            15
        );
        let again = tokenizer
            .checked_next_input_position(&source, &mut c)
            .unwrap();
        assert_eq!((position.start(), position.end()), (17, 17));
        assert_eq!(position, again);
        let actual = next_kind(&mut tokenizer, &mut c);
        let TokenKind::Text(value) = &actual else {
            panic!("real preprocessed text")
        };
        assert_eq!(value.value.text().code_units(), [10]);
        assert_eq!(
            (
                value.origins.as_slice()[0].source.start(),
                value.origins.as_slice()[0].source.end()
            ),
            (17, 19)
        );
        crate::construction::parser_session::release_token_kind(&mut Some(actual), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn standalone_constructor_rejects_real_captured_initialization_before_source_or_tree_effects() {
    let mut builder = crate::HtmlCaptureBuilder::new(
        crate::HtmlCaptureLimits::new(10, 10, 0, 0, 10, 0, 0),
        crate::HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::Quirks,
            crate::HtmlScriptingStatus::Disabled,
            crate::HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&document).unwrap())
        .unwrap();
    let request = crate::HtmlDocumentParseRequest::captured(
        HtmlSource::utf8("<script>x</script>"),
        capture,
        document,
        limits(100)
            .into_builder()
            .source_units(0)
            .managed_nodes(0)
            .finish()
            .unwrap(),
    )
    .unwrap();
    assert!(matches!(
        StandaloneDocumentDriver::new(&request),
        Err(StandaloneError::UnsupportedDocumentInitialization)
    ));
}

#[test]
fn standalone_constructor_source_cancellation_and_atomic_layout_failures_are_typed_before_tree_success()
 {
    let cancellation_request =
        crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<script>"), limits(100));
    cancellation_request.cancellation().cancel();
    assert!(matches!(
        StandaloneDocumentDriver::new(&cancellation_request),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    let cap_request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(""),
        limits(100)
            .into_builder()
            .atomic_layout_bytes(0)
            .finish()
            .unwrap(),
    );
    assert!(matches!(
        StandaloneDocumentDriver::new(&cap_request),
        Err(StandaloneError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::AtomicLayoutBytes,
            ..
        }))
    ));
    let source_request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>"),
        limits(100).into_builder().source_units(1).finish().unwrap(),
    );
    assert!(matches!(
        StandaloneDocumentDriver::new(&source_request),
        Err(StandaloneError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::SourceUnits,
            ..
        }))
    ));
}

#[test]
fn tokenizer_next_input_position_at_eof_is_checked_empty_and_foreign_owner_is_rejected() {
    let source = HtmlSource::utf8("");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let range = tokenizer
        .checked_next_input_position(&source, &mut c)
        .unwrap();
    assert_eq!(range.source(), source.identity());
    assert_eq!((range.start(), range.end()), (0, 0));
    let mut foreign = control(100);
    foreign.admit_source(&source).unwrap();
    assert!(matches!(
        tokenizer.checked_next_input_position(&source, &mut foreign),
        Err(crate::parser::tokenizer::TokenizerHandoffError::Stop(
            ParseStop::InternalInvariant(Invariant::ForeignControl)
        ))
    ));
    tokenizer.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn standalone_script_resource_caps_retain_created_payloads_and_stop_without_partial_success() {
    for (caps, expected, node_count) in [
        (
            limits(100)
                .into_builder()
                .managed_nodes(4)
                .finish()
                .unwrap(),
            HtmlParseResource::ManagedNodes,
            4,
        ),
        (limits(2), HtmlParseResource::OpenElements, 5),
        (
            limits(100).into_builder().diagnostics(0).finish().unwrap(),
            HtmlParseResource::Diagnostics,
            5,
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><script/>x</script>"),
            caps,
        );
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let error = (0..100)
            .find_map(|_| driver.step().err())
            .expect("actual source must hit selected cap");
        let stop = driver.control.stop().unwrap();
        assert!(
            matches!(stop, ParseStop::LimitExceeded { resource, .. } if resource == expected),
            "{error:?}"
        );
        assert_eq!(
            driver
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            node_count
        );
        let work = driver.usage().work(WorkCategory::InputUnitRead);
        assert!(matches!(driver.step(), Err(StandaloneError::Stop(reason)) if reason == stop));
        assert_eq!(driver.usage().work(WorkCategory::InputUnitRead), work);
        driver.release().unwrap();
        assert_eq!(driver.control.stop(), Some(stop));
        assert_released(&driver.control);
    }
}

#[test]
fn real_standalone_fixture_failed_cleanup_keeps_driver_and_first_stop_without_issuing_another_capability()
 {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    script_text_node(&mut driver);
    request.cancellation().cancel();
    let mut input = Some(driver);
    assert!(matches!(
        StandaloneDocumentDriver::take_standalone_fixture(&mut input),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert!(input.is_some());
    let driver = input.as_mut().unwrap();
    assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn standalone_selfclosing_script_is_not_void_and_emits_exact_source_qualified_error() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><script/>x</script></head>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
    let diagnostics = state.diagnostics.as_slice();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].condition,
        super::head::HeadCondition::UnacknowledgedSelfClosing
    );
    assert_eq!(
        (diagnostics[0].source.start(), diagnostics[0].source.end()),
        (15, 24)
    );
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(session.script_metadata(c, &node).unwrap().already_started);
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn tokenizer_next_input_point_has_independent_physical_provider_and_metadata_category_deltas() {
    // Handoff: state + pending + five drafts. Then peek: reconsume, raw
    // position and range. A character adds its offset and checked range's
    // external/private metadata. EOF omits those three visits and one range.
    for (source, visits, copies, input_reads, inspected) in [
        (HtmlSource::utf8(""), 11, 2, 0, 0),
        (
            HtmlSource::utf8("ab"),
            14,
            3,
            2,
            (size_of::<usize>() + size_of::<u32>()) as u64,
        ),
        (
            HtmlSource::utf16(DocumentString::from("ab")),
            14,
            3,
            1,
            (size_of::<usize>() + size_of::<u32>()) as u64,
        ),
    ] {
        let mut c = control(100);
        c.admit_source(&source).unwrap();
        let tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
        let before = c.usage().clone();
        let point = tokenizer
            .checked_next_input_position(&source, &mut c)
            .unwrap();
        assert_eq!((point.start(), point.end()), (0, 0));
        for (category, expected) in [
            (WorkCategory::RecordVisit, visits),
            (
                WorkCategory::CopyByte,
                copies * size_of::<crate::HtmlSourceRange>() as u64,
            ),
            (WorkCategory::InputUnitRead, input_reads),
            (WorkCategory::InspectByte, inspected),
            (WorkCategory::ProviderCall, 1),
            (WorkCategory::RecordWrite, 1),
            (WorkCategory::StateStep, 0),
            (WorkCategory::ReserveByte, 0),
        ] {
            assert_eq!(
                c.usage().work(category) - before.work(category),
                expected,
                "{category:?}"
            );
        }
        tokenizer.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn tokenizer_next_input_point_uses_actual_reconsume_and_reference_lookahead_positions() {
    for (source, emitted, next, offset) in [
        (HtmlSource::utf8("<1"), b'<', b'1', 1),
        (HtmlSource::utf16(DocumentString::from("<1")), b'<', b'1', 1),
        (HtmlSource::utf8("&ltx"), b'<', b'x', 3),
    ] {
        let mut c = control(100);
        c.admit_source(&source).unwrap();
        let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
        let first = next_kind(&mut tokenizer, &mut c);
        let TokenKind::Text(value) = &first else {
            panic!("real recovered literal")
        };
        assert_eq!(value.value.text().code_units(), [u16::from(emitted)]);
        crate::construction::parser_session::release_token_kind(&mut Some(first), &mut c).unwrap();
        let point = tokenizer
            .checked_next_input_position(&source, &mut c)
            .unwrap();
        assert_eq!((point.start(), point.end()), (offset, offset));
        let second = next_kind(&mut tokenizer, &mut c);
        let TokenKind::Text(value) = &second else {
            panic!("actual unconsumed following input")
        };
        assert_eq!(value.value.text().code_units(), [u16::from(next)]);
        assert_eq!(
            (
                value.origins.as_slice()[0].source.start(),
                value.origins.as_slice()[0].source.end()
            ),
            (offset, offset + 1)
        );
        crate::construction::parser_session::release_token_kind(&mut Some(second), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn standalone_required_checkpoint_has_source_derived_category_cost_and_one_short_terminal_admission()
 {
    // Proof: three retained qualification fields, two actual session facts,
    // two checked node identities, four real session readiness reads (document
    // validation, root creation, both identity checks), one root Weak creation,
    // then StateStep1.
    let expected_work = 11 + 1 + size_of::<ParserNode>() as u64 + 1;
    for short in [false, true] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<script>x</script>"),
            limits(100),
        );
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let node = script_text_node(&mut driver);
        let (c, session, proof) = driver.standalone_parts().unwrap();
        assert!(!session.script_metadata(c, &node).unwrap().already_started);
        // Exhaust the actual controller to isolate the fixed checkpoint seam;
        // expectations above are derived from its source operations, not runs.
        let remaining = request.limits().work_credits()
            - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            WorkCategory::StateStep,
            remaining - expected_work + u64::from(short),
        )
        .unwrap();
        let before = c.usage().clone();
        let result = proof.checkpoint(c, session);
        assert_eq!(
            c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
            11
        );
        assert_eq!(
            c.usage().work(WorkCategory::ProviderCall) - before.work(WorkCategory::ProviderCall),
            1
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
            size_of::<ParserNode>() as u64
        );
        assert_eq!(
            c.usage().work(WorkCategory::StateStep) - before.work(WorkCategory::StateStep),
            u64::from(!short)
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite),
            before.work(WorkCategory::RecordWrite)
        );
        if short {
            assert!(matches!(
                result,
                Err(SessionError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    requested_delta: 1,
                    ..
                }))
            ));
        } else {
            result.unwrap();
        }
        // The exact-budget case reaches its cap too; the next real drive latches
        // a stop before parsing work and both cases retain owners for cleanup.
        assert!(driver.step().is_err());
        assert_eq!(
            driver.head.as_ref().unwrap().state.as_ref().unwrap().mode,
            super::head::InsertionMode::Text
        );
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn empty_token_disposal_charges_only_the_actual_optional_slot_move() {
    // Controls' physical-event policy permits charging the optional-slot take,
    // but an absent token creates no PendingToken record or second write.
    let mut c = control(100);
    let before = c.usage().clone();
    let mut input = None;
    dispose_token(&mut input, &mut c).unwrap();
    assert!(input.is_none());
    assert_eq!(
        c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        size_of::<Option<super::super::tokenizer::token::Token>>() as u64
    );
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        1
    );
    assert_eq!(
        c.usage().largest_atomic_layout,
        size_of::<Option<super::super::tokenizer::token::Token>>() as u64
    );
    assert!(c.stop().is_none());
    assert_released(&c);
}

#[test]
fn empty_token_disposal_requires_no_pending_record_atomic_layout() {
    let slot_bytes = size_of::<Option<super::super::tokenizer::token::Token>>() as u64;
    let caps = limits(100)
        .into_builder()
        .atomic_layout_bytes(slot_bytes)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut input = None;
    let result = dispose_token(&mut input, &mut c);
    assert!(
        result.is_ok(),
        "the only moved record fits the exact slot layout: {result:?}"
    );
    assert!(input.is_none());
    assert!(c.stop().is_none());
    assert_released(&c);
}

#[test]
fn empty_token_disposal_requires_only_its_actual_slot_work_budget() {
    // One fetched optional slot, one fixed-slot move, and one slot write.
    let actual_work = size_of::<Option<super::super::tokenizer::token::Token>>() as u64 + 2;
    let caps = limits(100)
        .into_builder()
        .work_credits(actual_work)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(caps, HtmlParseCancellation::new());
    let mut input = None;
    let result = dispose_token(&mut input, &mut c);
    assert!(
        result.is_ok(),
        "empty disposal has no pending-record work: {result:?}"
    );
    assert!(input.is_none());
    assert!(c.stop().is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        actual_work
    );
    assert_released(&c);
}

#[test]
fn present_token_disposal_accounts_for_real_pending_and_payload_moves() {
    let source = HtmlSource::utf8("");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let Event::Token(event) = tokenizer.next(&mut c, false).unwrap().unwrap() else {
        panic!("real EOF token");
    };
    let mut input = Some(event.into_token(&mut c).unwrap());
    assert!(matches!(input.as_ref().unwrap().kind, TokenKind::Eof));
    let before = c.usage().clone();
    dispose_token(&mut input, &mut c).unwrap();
    // Optional Token take + constructed PendingToken, then extracted TokenKind
    // and its optional-slot disposal. EOF carries no nested payload tickets.
    let copied = size_of::<Option<super::super::tokenizer::token::Token>>()
        + size_of::<prologue::PendingToken>()
        + size_of::<TokenKind>()
        + size_of::<Option<TokenKind>>();
    assert_eq!(
        c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
        3
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        copied as u64
    );
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        6
    );
    assert!(input.is_none());
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    assert_released(&c);
}

fn body_document(driver: StandaloneDocumentDriver<'_>) -> (ParseControl, FrozenArena) {
    let mut input = Some(driver);
    let (mut c, mut session, _) =
        StandaloneDocumentDriver::take_standalone_fixture(&mut input).unwrap();
    assert!(input.is_none());
    let frozen = session.freeze(&mut c).unwrap();
    (c, frozen)
}
fn outline(document: &crate::HtmlDocument) -> Vec<(usize, Vec<u16>)> {
    // A test observer only: expected outlines below are literal source outcomes.
    // No serialization/reparse or synthetic producer drives construction.
    let mut output = Vec::new();
    let mut stack: Vec<_> = document.root().children().map(|id| (0, id)).collect();
    stack.reverse();
    while let Some((depth, id)) = stack.pop() {
        let node = document.node(&id).unwrap();
        let value = match node.kind() {
            HtmlNodeKind::Element(element) => element.name().local_name().code_units().to_vec(),
            HtmlNodeKind::Text(text) => {
                let mut units = vec![b'#' as u16];
                units.extend_from_slice(text.code_units());
                units
            }
            HtmlNodeKind::Comment(text) => {
                let mut units = vec![b'!' as u16];
                units.extend_from_slice(text.code_units());
                units
            }
            HtmlNodeKind::ProcessingInstruction(pi) => {
                let mut units = vec![b'?' as u16];
                units.extend_from_slice(pi.target().code_units());
                units
            }
            _ => continue,
        };
        output.push((depth, value));
        let children: Vec<_> = node.children().map(|id| (depth + 1, id)).collect();
        stack.extend(children.into_iter().rev());
    }
    output
}
fn expected_outline(rows: &[(usize, &str)]) -> Vec<(usize, Vec<u16>)> {
    rows.iter()
        .map(|(depth, value)| (*depth, value.encode_utf16().collect()))
        .collect()
}

#[test]
fn ordinary_body_recovers_paragraphs_blocks_and_missing_end_p_with_actual_topology() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body><p>a<div>b</div>c</p><p>d<p>e</body></html>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::AfterAfterBody);
    assert!(!state.frameset_ok);
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        super::prologue::Payload::Eof
    ));
    let (mut c, mut frozen) = body_document(driver);
    assert_eq!(
        outline(frozen.document().unwrap()),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "#a"),
            (2, "div"),
            (3, "#b"),
            (2, "#c"),
            (2, "p"),
            (2, "p"),
            (3, "#d"),
            (2, "p"),
            (3, "#e")
        ])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_heading_list_and_ruby_implied_end_rules_preserve_literal_sibling_order() {
    for (source, rows) in [
        (
            "<body><h1>a<h2>b</h1>c",
            vec![(2, "h1"), (3, "#a"), (2, "h2"), (3, "#b"), (2, "#c")],
        ),
        (
            "<body><ul><li>a<li>b</ul><dl><dd>c<dt>d</dl>",
            vec![
                (2, "ul"),
                (3, "li"),
                (4, "#a"),
                (3, "li"),
                (4, "#b"),
                (2, "dl"),
                (3, "dd"),
                (4, "#c"),
                (3, "dt"),
                (4, "#d"),
            ],
        ),
        (
            "<body><ruby><rb>a<rtc><rt>b<rp>c</ruby>",
            vec![
                (2, "ruby"),
                (3, "rb"),
                (4, "#a"),
                (3, "rtc"),
                (4, "rt"),
                (5, "#b"),
                (4, "rp"),
                (5, "#c"),
            ],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        let (mut c, mut frozen) = body_document(driver);
        let mut expected = vec![(0, "html"), (1, "head"), (1, "body")];
        expected.extend(rows);
        assert_eq!(
            outline(frozen.document().unwrap()),
            expected_outline(&expected),
            "{source}"
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn late_head_style_and_disabled_script_remove_the_specific_head_stack_entry_and_restore_current_mode()
 {
    use crate::construction::parser_session::ScriptType;
    for (source, head_child, expected_mode) in [
        (
            "<head></head><style>x</style><body>y",
            "style",
            super::head::InsertionMode::AfterHead,
        ),
        (
            "<head></head><script>x</script><body>y",
            "script",
            super::head::InsertionMode::AfterHead,
        ),
        (
            "<body><style>x</style>y",
            "style",
            super::head::InsertionMode::InBody,
        ),
        (
            "<body><script>x</script>y",
            "script",
            super::head::InsertionMode::InBody,
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let mut raw_node = None;
        for _ in 0..100 {
            driver.step().unwrap();
            let Some(head) = &mut driver.head else {
                continue;
            };
            let state = head.state.as_mut().unwrap();
            if state.mode == super::head::InsertionMode::Text {
                assert!(state.temporary_head.is_none());
                let node = state.before.kernel.current(&mut driver.control).unwrap();
                assert!(
                    state
                        .before
                        .kernel
                        .named(&node, head_child, &mut driver.control)
                        .unwrap()
                );
                let parent = state
                    .before
                    .kernel
                    .session
                    .parent(&mut driver.control, &node)
                    .unwrap()
                    .unwrap();
                assert!(
                    state
                        .before
                        .kernel
                        .named(
                            &parent,
                            if expected_mode == super::head::InsertionMode::AfterHead {
                                "head"
                            } else {
                                "body"
                            },
                            &mut driver.control
                        )
                        .unwrap()
                );
                for index in 0..state.before.kernel.stack().unwrap().len() {
                    let entry = state.before.kernel.at(index, &mut driver.control).unwrap();
                    assert!(
                        !state
                            .before
                            .kernel
                            .named(&entry, "head", &mut driver.control)
                            .unwrap()
                    );
                }
                raw_node = Some(node);
                break;
            }
        }
        let node = raw_node.expect("actual raw/script mode");
        driver.step().unwrap(); // actual x text
        driver.step().unwrap(); // actual end token
        assert_eq!(
            driver.head.as_ref().unwrap().state.as_ref().unwrap().mode,
            expected_mode
        );
        if head_child == "script" {
            let (c, session, _) = driver.standalone_parts().unwrap();
            let metadata = session.script_metadata(c, &node).unwrap();
            assert!(metadata.already_started);
            assert_eq!(metadata.script_type, Some(ScriptType::Classic));
        }
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn body_first_lf_and_null_recovery_preserve_exact_units_and_omitted_origin_evidence() {
    let mut units: Vec<_> = "<body><pre>\r\n".encode_utf16().collect();
    units.extend_from_slice(&[0xd800, 0xd83d, 0xde00, 0, b'a' as u16]);
    units.extend("</pre><listing>&#10;b</listing>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.before.whitespace.len(), 3);
    let omitted: Vec<_> = state
        .before
        .whitespace
        .as_slice()
        .iter()
        .map(|text| text.value.text().code_units().to_vec())
        .collect();
    assert_eq!(omitted, vec![vec![10], vec![0], vec![10]]);
    let null = state
        .diagnostics
        .as_slice()
        .iter()
        .find(|d| d.condition == super::head::HeadCondition::NullInBody)
        .unwrap();
    assert_eq!((null.source.start(), null.source.end()), (16, 17));
    let (mut c, mut frozen) = body_document(driver);
    let mut expected = expected_outline(&[(0, "html"), (1, "head"), (1, "body"), (2, "pre")]);
    expected.push((3, vec![b'#' as u16, 0xd800, 0xd83d, 0xde00, b'a' as u16]));
    expected.extend(expected_outline(&[(2, "listing"), (3, "#b")]));
    assert_eq!(outline(frozen.document().unwrap()), expected);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_after_modes_place_comments_and_pi_under_html_then_document() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body>x</body><!--a--><?one data?></html><!--b--><?two data?>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    assert_eq!(
        outline(frozen.document().unwrap()),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "#x"),
            (1, "!a"),
            (1, "?one"),
            (0, "!b"),
            (0, "?two")
        ])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_family_continuations_preserve_real_tokens_before_category_effects() {
    use super::head::HeadBoundary;
    for (name, boundary) in [
        ("object", HeadBoundary::Form),
        ("fieldset", HeadBoundary::Form),
        ("input", HeadBoundary::Form),
        ("textarea", HeadBoundary::Form),
        ("select", HeadBoundary::Select),
        ("option", HeadBoundary::Select),
        ("template", HeadBoundary::Template),
        ("svg", HeadBoundary::Foreign),
        ("math", HeadBoundary::Foreign),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!("<body><p>x<{name} data-x=kept>tail")),
            limits(100),
        );
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(standalone_boundary(&mut driver), boundary, "{name}");
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
        let super::prologue::Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload
        else {
            panic!("retained actual family tag");
        };
        assert_eq!(tag.name.value.text(), &DocumentString::from(name));
        assert_eq!(tag.attributes.len(), 1);
        assert_eq!(
            tag.attributes.as_slice()[0].value.value.text(),
            &DocumentString::from("kept")
        );
        let node_count = driver
            .control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current;
        assert!(driver.step().is_err());
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            node_count
        );
        let mut retained = Some(driver);
        assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut retained).is_err());
        let driver = retained
            .as_mut()
            .expect("failed extraction retains the sole owner");
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn body_generic_names_void_recovery_rawtext_and_plaintext_follow_actual_source_rows() {
    for (source, rows) in [
        (
            "<body><widget>a<div>b</widget>c</div></widget>",
            vec![(2, "widget"), (3, "#a"), (3, "div"), (4, "#bc")],
        ),
        (
            "<body><p>a<hr><image src=x></br><source><track><wbr>",
            vec![
                (2, "p"),
                (3, "#a"),
                (2, "hr"),
                (2, "img"),
                (2, "br"),
                (2, "source"),
                (2, "track"),
                (2, "wbr"),
            ],
        ),
        (
            "<body><p>a<xmp>&amp;<b></xmp><iframe>x</iframe><noembed>y</noembed><noscript>z</noscript>",
            vec![
                (2, "p"),
                (3, "#a"),
                (2, "xmp"),
                (3, "#&amp;<b>"),
                (2, "iframe"),
                (3, "#x"),
                (2, "noembed"),
                (3, "#y"),
                (2, "noscript"),
                (3, "#z"),
            ],
        ),
        (
            "<body><p>a<plaintext><b>&amp;</b>",
            vec![(2, "p"), (3, "#a"), (2, "plaintext"), (3, "#<b>&amp;</b>")],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        let (mut c, mut frozen) = body_document(driver);
        let mut expected = vec![(0, "html"), (1, "head"), (1, "body")];
        expected.extend(rows);
        assert_eq!(
            outline(frozen.document().unwrap()),
            expected_outline(&expected),
            "{source}"
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn body_explicit_implied_and_repeated_body_merge_keep_source_frameset_state_and_attribute_order() {
    for (source, expected_flag) in [
        ("<head></head>", true),
        ("<body>", false),
        ("<head></head><widget>", true),
        ("<head></head>x", false),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        assert_eq!(
            driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .frameset_ok,
            expected_flag,
            "{source}"
        );
        driver.release().unwrap();
        assert_released(&driver.control);
    }
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<html root=a><body old=a><body old=b new=c NEW=d><html root=b extra=e>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document
        .node(&document.root().children().next().unwrap())
        .unwrap();
    let body = document.node(&html.children().last().unwrap()).unwrap();
    for (node, expected) in [
        (html, vec![("root", "a"), ("extra", "e")]),
        (body, vec![("old", "a"), ("new", "c")]),
    ] {
        let HtmlNodeKind::Element(element) = node.kind() else {
            panic!("actual HTML element");
        };
        let actual: Vec<_> = element
            .attributes()
            .iter()
            .map(|attr| (attr.name().local_name().clone(), attr.value().clone()))
            .collect();
        assert_eq!(
            actual,
            expected
                .into_iter()
                .map(|(name, value)| (DocumentString::from(name), DocumentString::from(value)))
                .collect::<Vec<_>>()
        );
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_ignored_table_parts_and_frame_are_not_generic_nodes_and_keygen_has_source_void_recovery() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<body><p>a<caption><col><colgroup><frame><head><tbody><td><tfoot><th><thead><tr><keygen>b</keygen></template></div>",
        ),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    assert_eq!(
        outline(frozen.document().unwrap()),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "#a"),
            (3, "keygen"),
            (3, "#b")
        ])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_eof_predicate_accepts_rb_rtc_and_reports_one_actual_unexpected_open_node() {
    for (source, errors) in [("<body><rb><rtc>", 0), ("<body><widget><div>", 1)] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(
            state
                .diagnostics
                .as_slice()
                .iter()
                .filter(|d| d.condition == super::head::HeadCondition::UnexpectedOpenElement)
                .count(),
            errors
        );
        assert!(matches!(
            state.pending.as_ref().unwrap().payload,
            super::prologue::Payload::Eof
        ));
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn body_first_lf_applies_to_only_the_next_real_token_and_afterbody_other_input_reprocesses() {
    for (source, expected) in [
        (
            "<body><pre><!--c-->\nb</pre>",
            vec![(2, "pre"), (3, "!c"), (3, "#\nb")],
        ),
        ("<body><pre>&#10;\nb</pre>", vec![(2, "pre"), (3, "#\nb")]),
        ("<body>a</body>b</html>c", vec![(2, "#abc")]),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        let (mut c, mut frozen) = body_document(driver);
        let mut rows = vec![(0, "html"), (1, "head"), (1, "body")];
        rows.extend(expected);
        assert_eq!(
            outline(frozen.document().unwrap()),
            expected_outline(&rows),
            "{source}"
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn body_temporary_head_open_cap_failure_retains_obligation_and_created_raw_node_for_cleanup() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<head></head><style>x</style>"),
        limits(2),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut stopped = None;
    for _ in 0..100 {
        match driver.step() {
            Ok(_) => {}
            Err(error) => {
                stopped = Some(error);
                break;
            }
        }
    }
    assert!(stopped.is_some());
    assert!(matches!(
        driver.control.stop(),
        Some(ParseStop::LimitExceeded {
            resource: HtmlParseResource::OpenElements,
            ..
        })
    ));
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::AfterHead);
    assert!(state.temporary_head.is_some());
    assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        4
    );
    let before = driver.control.usage().clone();
    assert!(driver.step().is_err());
    assert_eq!(driver.control.usage(), &before);
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn body_null_diagnostic_cap_and_cancel_stop_without_successful_partial_continuation() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body>a\0b"),
        limits(100).into_builder().diagnostics(2).finish().unwrap(),
    );
    // Missing-doctype and lexical NULL evidence consume two slots before
    // the independent tree-construction NULL error.
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut error = None;
    for _ in 0..100 {
        match driver.step() {
            Ok(_) => {}
            Err(value) => {
                error = Some(value);
                break;
            }
        }
    }
    assert!(error.is_some());
    let stop = driver.control.stop().unwrap();
    assert!(matches!(
        stop,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::Diagnostics,
            ..
        }
    ));
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.body_runs.is_some());
    assert!(state.body_run.is_some());
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), Some(stop));
    driver.release().unwrap();
    assert_released(&driver.control);
    let request =
        crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<body><p>x"), limits(100));
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    for _ in 0..100 {
        driver.step().unwrap();
        if driver
            .head
            .as_ref()
            .is_some_and(|h| h.state.as_ref().unwrap().mode == super::head::InsertionMode::InBody)
        {
            break;
        }
    }
    request.cancellation().cancel();
    let before = driver.control.usage().clone();
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
    driver.release().unwrap();
    for category in [
        WorkCategory::InputUnitRead,
        WorkCategory::StateStep,
        WorkCategory::RecordWrite,
    ] {
        assert_eq!(driver.control.usage().work(category), before.work(category));
    }
    assert_released(&driver.control);
}

#[test]
fn body_source_void_keygen_is_immediately_popped_and_sets_frameset_not_ok() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body><keygen>a<p>b"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    assert!(
        !driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .frameset_ok
    );
    let (mut c, mut frozen) = body_document(driver);
    assert_eq!(
        outline(frozen.document().unwrap()),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "keygen"),
            (2, "#a"),
            (2, "p"),
            (3, "#b")
        ])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_source_void_end_br_discards_real_end_tag_attributes_before_insertion() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body></br x=kept>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document
        .node(&document.root().children().next().unwrap())
        .unwrap();
    let body = document.node(&html.children().last().unwrap()).unwrap();
    let br = document.node(&body.children().next().unwrap()).unwrap();
    let HtmlNodeKind::Element(element) = br.kind() else {
        panic!("actual recovered br");
    };
    assert_eq!(element.name().local_name(), &DocumentString::from("br"));
    assert!(element.attributes().is_empty());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_source_void_frameset_not_ok_row_ignores_token_without_an_unsupported_transition() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body><frameset><widget>x"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    assert_eq!(
        outline(frozen.document().unwrap()),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "widget"),
            (3, "#x")
        ])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn body_source_scope_special_implied_and_eof_sets_keep_namespace_and_exact_case_distinctions() {
    use super::tag_sets::{NodeSet, in_set};
    let mut c = control(100);
    for name in [
        "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
    ] {
        let q = QualifiedName::new(Namespace::html(), None, DocumentString::from(name));
        assert!(in_set(&q, NodeSet::Implied, &mut c).unwrap(), "{name}");
        assert!(in_set(&q, NodeSet::BodyEnd, &mut c).unwrap(), "{name}");
        let other = QualifiedName::new(Namespace::svg(), None, DocumentString::from(name));
        assert!(!in_set(&other, NodeSet::Implied, &mut c).unwrap());
        assert!(!in_set(&other, NodeSet::BodyEnd, &mut c).unwrap());
    }
    for name in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        let q = QualifiedName::new(Namespace::html(), None, DocumentString::from(name));
        assert!(in_set(&q, NodeSet::Heading, &mut c).unwrap());
        assert!(!in_set(&q, NodeSet::BodyEnd, &mut c).unwrap());
    }
    for (namespace, name, special) in [
        (Namespace::mathml(), "annotation-xml", true),
        (Namespace::mathml(), "mi", true),
        (Namespace::svg(), "foreignObject", true),
        (Namespace::svg(), "foreignobject", false),
        (Namespace::svg(), "desc", true),
        (Namespace::svg(), "title", true),
        (Namespace::html(), "search", true),
        (Namespace::html(), "keygen", true),
        (Namespace::html(), "dialog", false),
        (Namespace::html(), "widget", false),
        (Namespace::html(), "image", false),
    ] {
        let q = QualifiedName::new(namespace, None, DocumentString::from(name));
        assert_eq!(
            in_set(&q, NodeSet::Special, &mut c).unwrap(),
            special,
            "{name}"
        );
    }
    let q = QualifiedName::new(
        Namespace::html(),
        None,
        DocumentString::from_utf16(vec![b'p' as u16, 0xd800]),
    );
    assert!(!in_set(&q, NodeSet::Implied, &mut c).unwrap());
    assert!(!in_set(&q, NodeSet::Special, &mut c).unwrap());
}

#[test]
fn body_supported_frameset_ignore_and_unfinished_transition_follow_actual_flag_before_any_tree_change()
 {
    for source in [
        "<head></head><frameset a=kept>",
        "<head></head><widget><frameset a=kept>",
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::Frameset
        );
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        let super::prologue::Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload
        else {
            panic!("retained transition token")
        };
        assert_eq!(tag.name.value.text(), &DocumentString::from("frameset"));
        assert_eq!(tag.attributes.len(), 1);
        assert!(state.frameset_ok);
        assert_eq!(
            state.before.kernel.stack().unwrap().len(),
            if source.contains("widget") { 3 } else { 1 }
        );
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn late_script_eof_returns_afterhead_then_creates_body_without_preparation() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<head></head><script type=module>x"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::InBody);
    assert!(state.temporary_head.is_none());
    assert!(state.frameset_ok);
    let (c, session, _) = driver.standalone_parts().unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(metadata.already_started);
    assert_eq!(metadata.script_type, None);
    assert!(
        session
            .script_preparation_document(c, &node)
            .unwrap()
            .is_none()
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn late_script_revoked_provenance_rejects_before_temporary_head_push_or_delegation_effects() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<head></head><script>x</script>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    loop {
        driver.step().unwrap();
        if driver.head.as_ref().is_some_and(|h| {
            h.state.as_ref().unwrap().mode == super::head::InsertionMode::AfterHead
        }) {
            break;
        }
    }
    let (c, session, _) = driver.standalone_parts().unwrap();
    preexisting(session, Namespace::html(), "imported", c);
    let diagnostics = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .diagnostics
        .len();
    let error = driver.step().unwrap_err();
    assert!(matches!(
        error,
        StandaloneError::Head(super::head::HeadError::Kernel(KernelError::Session(
            SessionError::Stop(ParseStop::UnsupportedStandaloneState(
                StandaloneUnsupported::Provenance
            ))
        )))
    ));
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(
        state.before.kernel.stack().unwrap().len(),
        1,
        "unsupported provenance must not temporarily push head"
    );
    assert!(state.temporary_head.is_none());
    assert_eq!(
        state.diagnostics.len(),
        diagnostics,
        "unsupported provenance precedes delegated row effects"
    );
    let super::prologue::Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload else {
        panic!("actual script token remains owned")
    };
    assert_eq!(tag.name.value.text(), &DocumentString::from("script"));
    let stop = driver.control.stop().unwrap();
    assert!(matches!(driver.step(),Err(StandaloneError::Stop(actual)) if actual==stop));
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn standalone_driver_consumes_body_and_preserves_source_paragraph_topology() {
    // HTML877d's AfterHead/body and paragraph rules require actual construction
    // from the real source, rather than returning at an unconsumed body token.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body><p>a<div>b</div>c</body></html>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let _boundary = standalone_boundary(&mut driver);
    let mut input = Some(driver);
    let (mut c, mut session, _) =
        StandaloneDocumentDriver::take_standalone_fixture(&mut input).unwrap();
    assert!(input.is_none());
    let mut frozen = session.freeze(&mut c).unwrap();
    let document = frozen.document().unwrap();
    let mut actual = Vec::new();
    let mut stack: Vec<_> = document.root().children().map(|id| (0, id)).collect();
    stack.reverse();
    while let Some((depth, id)) = stack.pop() {
        let node = document.node(&id).unwrap();
        match node.kind() {
            HtmlNodeKind::Element(element) => {
                actual.push((depth, element.name().local_name().code_units().to_vec()));
            }
            HtmlNodeKind::Text(text) => {
                let mut value = vec![b'#' as u16];
                value.extend_from_slice(text.code_units());
                actual.push((depth, value));
            }
            _ => {}
        }
        let children: Vec<_> = node.children().map(|id| (depth + 1, id)).collect();
        stack.extend(children.into_iter().rev());
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    let expected: Vec<_> = [
        (0, "html"),
        (1, "head"),
        (1, "body"),
        (2, "p"),
        (3, "#a"),
        (2, "div"),
        (3, "#b"),
        (2, "#c"),
    ]
    .into_iter()
    .map(|(depth, value)| (depth, value.encode_utf16().collect::<Vec<_>>()))
    .collect();
    assert_eq!(actual, expected);
}

// Expected trees: pinned HTML877d syntax capture §13.2.10.1/.2 and
// creation-token replay §13.2.4.3. Attribute variants do not change topology.

#[test]
fn misnested_inline_formatting_reconstructs_a_distinct_element_from_original_attributes() {
    for (source, expected_attributes) in [
        ("<p>1<b>2<i>3</b>4</i>5</p>", &[][..]),
        (
            "<p>1<b>2<i class=Keep data-x=Raw>3</b>4</i>5</p>",
            &[("class", "Keep"), ("data-x", "Raw")][..],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let boundary = standalone_boundary(&mut driver);
        if boundary != super::head::HeadBoundary::ConstructionEof {
            // Current real driver suspends at the formatting opener. Dispose
            // all actual owners before the expected runtime RED assertion.
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(boundary, super::head::HeadBoundary::ConstructionEof);
            return;
        }
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let actual_outline = outline(document);
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let p = document.node(&body).unwrap().children().next().unwrap();
        let old_b = document.node(&p).unwrap().children().nth(1).unwrap();
        let old_i = document.node(&old_b).unwrap().children().nth(1).unwrap();
        let new_i = document.node(&p).unwrap().children().nth(2).unwrap();
        let actual_attributes: Vec<_> = [&old_i, &new_i]
            .into_iter()
            .map(|id| {
                let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                    panic!("source-derived i positions must be actual elements");
                };
                element
                    .attributes()
                    .iter()
                    .map(|attribute| {
                        (
                            attribute
                                .name()
                                .namespace()
                                .uri()
                                .map(|uri| uri.code_units().to_vec()),
                            attribute.name().local_name().code_units().to_vec(),
                            attribute.value().code_units().to_vec(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(
            actual_outline,
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "p"),
                (3, "#1"),
                (3, "b"),
                (4, "#2"),
                (4, "i"),
                (5, "#3"),
                (3, "i"),
                (4, "#4"),
                (3, "#5"),
            ]),
            "{source}"
        );
        assert_ne!(
            old_i, new_i,
            "reconstruction allocates a distinct result-qualified node"
        );
        let expected_attributes: Vec<_> = expected_attributes
            .iter()
            .map(|(name, value)| {
                (
                    None::<Vec<u16>>,
                    name.encode_utf16().collect::<Vec<_>>(),
                    value.encode_utf16().collect::<Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(
            actual_attributes,
            [expected_attributes.clone(), expected_attributes]
        );
    }
}

#[test]
fn formatting_adoption_moves_furthest_block_children_into_a_distinct_element_with_original_attributes()
 {
    for (source, expected_attributes) in [
        ("<b>1<p>2</b>3</p>", &[][..]),
        (
            "<b class=Keep data-x=Raw>1<p>2</b>3</p>",
            &[("class", "Keep"), ("data-x", "Raw")][..],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let boundary = standalone_boundary(&mut driver);
        if boundary != super::head::HeadBoundary::ConstructionEof {
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(boundary, super::head::HeadBoundary::ConstructionEof);
            return;
        }
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let actual_outline = outline(document);
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let old_b = document.node(&body).unwrap().children().next().unwrap();
        let p = document.node(&body).unwrap().children().nth(1).unwrap();
        let new_b = document.node(&p).unwrap().children().next().unwrap();
        let actual_attributes: Vec<_> = [&old_b, &new_b]
            .into_iter()
            .map(|id| {
                let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                    panic!("source-derived b positions must be actual elements");
                };
                element
                    .attributes()
                    .iter()
                    .map(|attribute| {
                        (
                            attribute
                                .name()
                                .namespace()
                                .uri()
                                .map(|uri| uri.code_units().to_vec()),
                            attribute.name().local_name().code_units().to_vec(),
                            attribute.value().code_units().to_vec(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(
            actual_outline,
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "b"),
                (3, "#1"),
                (2, "p"),
                (3, "b"),
                (4, "#2"),
                (3, "#3"),
            ]),
            "{source}"
        );
        assert_ne!(
            old_b, new_b,
            "adoption creates a new formatting node instead of reusing the old one"
        );
        let expected_attributes: Vec<_> = expected_attributes
            .iter()
            .map(|(name, value)| {
                (
                    None::<Vec<u16>>,
                    name.encode_utf16().collect::<Vec<_>>(),
                    value.encode_utf16().collect::<Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(
            actual_attributes,
            [expected_attributes.clone(), expected_attributes]
        );
    }
}

fn formatting_outline(source: &str) -> Vec<(usize, Vec<u16>)> {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadBoundary::ConstructionEof {
        driver.release().unwrap();
        assert_released(&driver.control);
        panic!("selected formatting source stopped at {boundary:?}");
    }
    let (mut c, mut frozen) = body_document(driver);
    let result = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    result
}

#[test]
fn formatting_all_fourteen_start_and_end_rows_use_actual_creation() {
    for name in [
        "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong", "tt",
        "u",
    ] {
        let input = format!("<{name} data-x=Y/>1</{name}>");
        assert_eq!(
            formatting_outline(&input),
            expected_outline(&[(0, "html"), (1, "head"), (1, "body"), (2, name), (3, "#1")]),
            "{name}"
        );
    }
}

#[test]
fn formatting_nested_a_and_nobr_recovery_retains_sibling_order() {
    for (source, name) in [
        ("<p><a id=old>1<a id=new>2</a>3</p>", "a"),
        ("<p><nobr>1<nobr>2</nobr>3</p>", "nobr"),
    ] {
        assert_eq!(
            formatting_outline(source),
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "p"),
                (3, name),
                (4, "#1"),
                (3, name),
                (4, "#2"),
                (3, "#3")
            ])
        );
    }
}

#[test]
fn formatting_missing_and_not_open_entries_take_distinct_source_recovery() {
    assert_eq!(
        formatting_outline("<p>x</b>y</p>"),
        expected_outline(&[(0, "html"), (1, "head"), (1, "body"), (2, "p"), (3, "#xy")])
    );
    assert_eq!(
        formatting_outline("<p><b>1</p></b>2"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "#1"),
            (2, "#2")
        ])
    );
}

#[test]
fn formatting_reconstruction_occurs_at_whitespace_void_generic_and_xmp_rows() {
    for (source, suffix) in [
        ("<p><b>1</p> 2", vec![(2, "b"), (3, "# 2")]),
        ("<p><b>1</p><br/>2", vec![(2, "b"), (3, "br"), (3, "#2")]),
        ("<p><b>1</p><image>2", vec![(2, "b"), (3, "img"), (3, "#2")]),
        (
            "<p><b>1</p></br a=discarded>2",
            vec![(2, "b"), (3, "br"), (3, "#2")],
        ),
        ("<p><b>1</p><x>2</x>", vec![(2, "b"), (3, "x"), (4, "#2")]),
        (
            "<p><b>1</p><xmp>2</xmp>3",
            vec![(2, "b"), (3, "xmp"), (4, "#2"), (3, "#3")],
        ),
        (
            "<p><b>1</p><noscript>2</noscript>",
            vec![(2, "b"), (3, "noscript"), (4, "#2")],
        ),
    ] {
        let mut expected = expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "#1"),
        ]);
        expected.extend(expected_outline(&suffix));
        assert_eq!(formatting_outline(source), expected, "{source}");
    }
}

#[test]
fn formatting_null_comments_and_head_text_do_not_invent_reconstruction() {
    assert_eq!(
        formatting_outline("<p><b>1</p>\0<!--x--><?t data?>"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "#1"),
            (2, "!x"),
            (2, "?t")
        ])
    );
    assert_eq!(
        formatting_outline("<p><b>1</p><script>x</script>2"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "#1"),
            (2, "script"),
            (3, "#x"),
            (2, "b"),
            (3, "#2")
        ])
    );
}

#[test]
fn formatting_real_applet_and_marquee_markers_isolate_and_clear_active_entries() {
    assert_eq!(
        formatting_outline("<b>1<marquee>2</b>3</marquee>4"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "b"),
            (3, "#1"),
            (3, "marquee"),
            (4, "#23"),
            (3, "#4")
        ])
    );
    assert_eq!(
        formatting_outline("<p><b>1<applet><i>2</applet>3</p>4"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "#1"),
            (4, "applet"),
            (5, "i"),
            (6, "#2"),
            (4, "#3"),
            (2, "b"),
            (3, "#4")
        ])
    );
}

#[test]
fn formatting_noah_evicts_earliest_equal_recipe_but_keeps_original_tree_node() {
    assert_eq!(
        formatting_outline("<p><b a=1 b=2><b b=2 a=1><b a=1 b=2><b b=2 a=1>1</p>2"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "b"),
            (5, "b"),
            (6, "b"),
            (7, "#1"),
            (2, "b"),
            (3, "b"),
            (4, "b"),
            (5, "#2")
        ])
    );
    assert_eq!(
        formatting_outline("<p><b a=X><b a=x><b a=X><b a=x>1</p>2"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "b"),
            (5, "b"),
            (6, "b"),
            (7, "#1"),
            (2, "b"),
            (3, "b"),
            (4, "b"),
            (5, "b"),
            (6, "#2")
        ])
    );
}

#[test]
fn formatting_adoption_inner_loop_replaces_three_and_prunes_later_entries_without_capping() {
    assert_eq!(
        formatting_outline("<b><i><u><em><s><div>x</b>y"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "b"),
            (3, "i"),
            (4, "u"),
            (5, "em"),
            (6, "s"),
            (2, "u"),
            (3, "em"),
            (4, "s"),
            (5, "div"),
            (6, "b"),
            (7, "#x"),
            (6, "#y")
        ])
    );
}

#[test]
fn formatting_adoption_outer_loop_stops_after_exactly_eight_source_iterations() {
    assert_eq!(
        formatting_outline("<b><div><div><div><div><div><div><div><div><div>x</b>y"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "b"),
            (2, "div"),
            (3, "b"),
            (3, "div"),
            (4, "b"),
            (4, "div"),
            (5, "b"),
            (5, "div"),
            (6, "b"),
            (6, "div"),
            (7, "b"),
            (7, "div"),
            (8, "b"),
            (8, "div"),
            (9, "b"),
            (9, "div"),
            (10, "b"),
            (11, "div"),
            (12, "#xy")
        ])
    );
}

#[test]
fn formatting_real_open_slot_exhaustion_retains_pending_recipe_and_sticky_first_stop() {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<b>x"), limits(2));
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut failed = false;
    for _ in 0..100 {
        if driver.step().is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    let first = driver.control.stop();
    assert!(first.is_some());
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.formatting_pending.is_some());
    assert_eq!(state.formatting.len(), 0);
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), first);
    let mut retained = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut retained).is_err());
    let driver = retained.as_mut().unwrap();
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), first);
    assert_released(&driver.control);
}

#[test]
fn formatting_cancellation_reclaims_original_recipes_and_native_replays() {
    let request =
        crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p><b a=x>1</p>2"), limits(100));
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    request.cancellation().cancel();
    let mut retained = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut retained).is_err());
    let driver = retained.as_mut().unwrap();
    assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn formatting_replay_preserves_literal_utf16_pairs_and_isolated_surrogates() {
    let mut units = "<p><b data-v='".encode_utf16().collect::<Vec<_>>();
    units.extend([0xd800, 0xd83d, 0xde00, 0xdc00]);
    units.extend("'>1</p>2".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document.root().children().next().unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let p = document.node(&body).unwrap().children().next().unwrap();
    let old = document.node(&p).unwrap().children().next().unwrap();
    let new = document.node(&body).unwrap().children().nth(1).unwrap();
    for id in [&old, &new] {
        let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
            panic!("actual formatting element")
        };
        assert_eq!(
            element
                .attributes()
                .iter()
                .next()
                .unwrap()
                .value()
                .code_units(),
            &[0xd800, 0xd83d, 0xde00, 0xdc00]
        );
    }
    assert_ne!(old, new);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn formatting_stack_replacement_charges_actual_clone_move_and_slot_write() {
    let source = HtmlSource::utf8("<html><p><span>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer = Tokenizer::new(&source, State::Data, None, &mut c).unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let old = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let replacement = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
    let before = c.usage().clone();
    kernel.replace_stack(1, &replacement, &mut c).unwrap();
    // Kernel identity/open preflight, actual session readiness and target index,
    // then the get_mut slot.
    assert_eq!(
        c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
        4
    );
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        2 * size_of::<ParserNode>() as u64
    );
    assert_eq!(
        c.usage().work(WorkCategory::ProviderCall) - before.work(WorkCategory::ProviderCall),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::StateStep) - before.work(WorkCategory::StateStep),
        0
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::OpenElements).current,
        2
    );
    let current = kernel.current(&mut c).unwrap();
    assert!(
        kernel
            .session
            .same_node(&mut c, &current, &replacement)
            .unwrap()
    );
    let parent = kernel
        .session
        .parent(&mut c, &replacement)
        .unwrap()
        .unwrap();
    assert!(kernel.session.same_node(&mut c, &parent, &old).unwrap());
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn formatting_noah_replay_keeps_each_retained_original_attribute_order() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p><b a=1 b=2><b b=2 a=1><b a=1 b=2><b b=2 a=1>1</p>2"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document.root().children().next().unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let mut id = document.node(&body).unwrap().children().nth(1).unwrap();
    for expected in [["b", "a"], ["a", "b"], ["b", "a"]] {
        let node = document.node(&id).unwrap();
        let HtmlNodeKind::Element(element) = node.kind() else {
            panic!("actual replay")
        };
        let names = element
            .attributes()
            .iter()
            .map(|attribute| attribute.name().local_name().code_units().to_vec())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            expected
                .into_iter()
                .map(|name| name.encode_utf16().collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
        id = node.children().next().unwrap();
    }
    let HtmlNodeKind::Text(text) = document.node(&id).unwrap().kind() else {
        panic!("reconstructed descendant text")
    };
    assert_eq!(text.code_units(), &[b'2' as u16]);
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn formatting_nested_a_diagnostic_cap_stops_before_adoption_or_second_creation() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><a>1<a id=kept>2"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut stopped = false;
    for _ in 0..100 {
        if driver.step().is_err() {
            stopped = true;
            break;
        }
    }
    assert!(stopped);
    let first = driver.control.stop();
    assert!(first.is_some());
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.formatting.len(), 1);
    assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
    assert!(state.formatting_pending.is_none());
    let super::prologue::Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload else {
        panic!("original nested a input")
    };
    assert_eq!(
        tag.attributes.as_slice()[0].value.value.text().code_units(),
        "kept".encode_utf16().collect::<Vec<_>>()
    );
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), first);
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn formatting_after_body_and_after_after_body_whitespace_delegates_to_real_body_reconstruction() {
    for source in ["<p><b>1</p></body> ", "<p><b>1</p></body></html> "] {
        assert_eq!(
            formatting_outline(source),
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "p"),
                (3, "b"),
                (4, "#1"),
                (2, "b"),
                (3, "# ")
            ])
        );
    }
}

#[test]
fn ordinary_form_end_removes_only_the_pointed_stack_identity_and_allows_a_later_form() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form id=f><div>1</form>2</div><form id=g>3</form>4"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadBoundary::ConstructionEof {
        // The published driver retains the actual opener at Form. Release the
        // real tree/token/controller owners before the runtime RED assertion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadBoundary::ConstructionEof);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let old_form = document.node(&body).unwrap().children().next().unwrap();
    let later_form = document.node(&body).unwrap().children().nth(1).unwrap();
    let attributes: Vec<_> = [&old_form, &later_form]
        .into_iter()
        .map(|id| {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("source-derived form positions must be actual elements");
            };
            element
                .attributes()
                .iter()
                .map(|attribute| {
                    (
                        attribute
                            .name()
                            .namespace()
                            .uri()
                            .map(|uri| uri.code_units().to_vec()),
                        attribute.name().local_name().code_units().to_vec(),
                        attribute.value().code_units().to_vec(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // Observations own plain units, not clones of admitted string backing.
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "div"),
            (4, "#12"),
            (2, "form"),
            (3, "#3"),
            (2, "#4"),
        ])
    );
    assert_ne!(old_form, later_form);
    assert_eq!(
        attributes,
        vec![
            vec![(
                None::<Vec<u16>>,
                vec![b'i' as u16, b'd' as u16],
                vec![b'f' as u16]
            )],
            vec![(
                None::<Vec<u16>>,
                vec![b'i' as u16, b'd' as u16],
                vec![b'g' as u16]
            )],
        ]
    );
}

#[test]
fn ordinary_form_nested_start_is_ignored_before_paragraph_closure_or_creation() {
    assert_eq!(
        formatting_outline("<!doctype html><form id=f><p>1<form id=ignored>2</form>3"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "p"),
            (4, "#12"),
            (2, "#3"),
        ])
    );
}

#[test]
fn ordinary_form_null_and_out_of_scope_end_recovery_allows_the_next_form() {
    for (source, suffix) in [
        (
            "<!doctype html><body></form><form id=g>2</form>",
            vec![(2, "form"), (3, "#2")],
        ),
        (
            "<!doctype html><div><form id=f>1</div></form><form id=g>2</form>",
            vec![(2, "div"), (3, "form"), (4, "#1"), (2, "form"), (3, "#2")],
        ),
        (
            "<!doctype html><form id=f><marquee></form><form id=g>2</form></marquee>",
            vec![(2, "form"), (3, "marquee"), (4, "form"), (5, "#2")],
        ),
    ] {
        let mut expected = expected_outline(&[(0, "html"), (1, "head"), (1, "body")]);
        expected.extend(expected_outline(&suffix));
        assert_eq!(formatting_outline(source), expected, "{source}");
    }
}

#[test]
fn ordinary_form_start_closes_the_paragraph_and_self_closing_does_not_close_the_form() {
    assert_eq!(
        formatting_outline("<!doctype html><p>1<form/>2</form>3"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "#1"),
            (2, "form"),
            (3, "#2"),
            (2, "#3"),
        ])
    );
}

#[test]
fn ordinary_form_end_diagnostic_cap_clears_pointer_before_error_but_keeps_stack_identity() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form id=f><div></form>"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut failed = false;
    for _ in 0..100 {
        if driver.step().is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    let first = driver.control.stop();
    assert!(first.is_some());
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.current_form.is_none());
    assert_eq!(state.before.kernel.stack().unwrap().len(), 4);
    let Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload else {
        panic!("real end form remains owned");
    };
    assert_eq!(tag.kind, TagKind::End);
    assert_eq!(
        tag.name.value.text().code_units(),
        &[b'f' as u16, b'o' as u16, b'r' as u16, b'm' as u16]
    );
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), first);
    let mut retained = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut retained).is_err());
    let driver = retained.as_mut().unwrap();
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), first);
    assert_released(&driver.control);
}

#[test]
fn ordinary_form_nested_diagnostic_cap_preserves_pointer_paragraph_and_original_tag() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form><p>1<form id=kept>"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut failed = false;
    for _ in 0..100 {
        if driver.step().is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    let first = driver.control.stop();
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.current_form.is_some());
    assert_eq!(state.before.kernel.stack().unwrap().len(), 4);
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        8
    );
    let Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload else {
        panic!("real nested form remains owned");
    };
    assert_eq!(
        tag.attributes.as_slice()[0].value.value.text().code_units(),
        "kept".encode_utf16().collect::<Vec<_>>()
    );
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), first);
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn ordinary_form_open_slot_exhaustion_keeps_created_payload_but_cannot_set_pointer_or_publish() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form id=kept>"),
        limits(2),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut failed = false;
    for _ in 0..100 {
        if driver.step().is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    let first = driver.control.stop();
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.current_form.is_none());
    assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
    // Document, doctype, html, head, body and the actual newly created form.
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        6
    );
    assert!(matches!(
        &state.pending.as_ref().unwrap().payload,
        Payload::Tag(None)
    ));
    let mut retained = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut retained).is_err());
    let driver = retained.as_mut().unwrap();
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), first);
    assert_released(&driver.control);
}

#[test]
fn ordinary_form_cancellation_releases_the_real_pointer_and_lexical_owners() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form id=f><img>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut opened = false;
    for _ in 0..100 {
        driver.step().unwrap();
        if driver
            .head
            .as_ref()
            .and_then(|head| head.state.as_ref())
            .is_some_and(|state| state.current_form.is_some())
        {
            opened = true;
            break;
        }
    }
    assert!(opened);
    request.cancellation().cancel();
    assert!(driver.step().is_err());
    let first = driver.control.stop();
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), first);
    driver.release().unwrap();
    assert_released(&driver.control);
}

fn form_image_observations(
    source: &str,
) -> Vec<(
    Option<Vec<u16>>,
    bool,
    crate::construction::parser_session::ImageAssociationEvidence,
)> {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadBoundary::ConstructionEof {
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadBoundary::ConstructionEof);
        return Vec::new();
    }
    let (mut c, mut frozen) = body_document(driver);
    let result = {
        let document = frozen.document().unwrap();
        let mut stack: Vec<_> = document.root().children().collect();
        stack.reverse();
        let mut result = Vec::new();
        while let Some(id) = stack.pop() {
            let node = document.node(&id).unwrap();
            if let HtmlNodeKind::Element(element) = node.kind()
                && element.name().local_name().code_units()
                    == [b'i' as u16, b'm' as u16, b'g' as u16]
            {
                let state = frozen.image_form_state(&mut c, &id).unwrap();
                let owner = frozen.image_form_owner(&mut c, &id).unwrap();
                let owner_id = owner.map(|owner| {
                    let HtmlNodeKind::Element(element) = document.node(&owner).unwrap().kind()
                    else {
                        panic!("qualified form owner");
                    };
                    assert_eq!(
                        element.name().local_name().code_units(),
                        &[b'f' as u16, b'o' as u16, b'r' as u16, b'm' as u16]
                    );
                    element
                        .attributes()
                        .iter()
                        .find(|attribute| {
                            attribute.name().local_name().code_units() == [b'i' as u16, b'd' as u16]
                        })
                        .unwrap()
                        .value()
                        .code_units()
                        .to_vec()
                });
                result.push((owner_id, state.parser_inserted, state.evidence));
            }
            let children: Vec<_> = node.children().collect();
            stack.extend(children.into_iter().rev());
        }
        result
    };
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    result
}

#[test]
fn ordinary_form_images_preserve_known_null_ancestor_and_nonancestor_owners() {
    use crate::construction::parser_session::ImageAssociationEvidence::{ParserAssociated, Reset};
    for (source, owner, inserted, evidence) in [
        ("<!doctype html><img>", None, false, Reset),
        (
            "<!doctype html><form id=f><img>",
            Some("f"),
            true,
            ParserAssociated,
        ),
        (
            "<!doctype html><form id=f><img form=other>",
            Some("f"),
            true,
            ParserAssociated,
        ),
        (
            "<!doctype html><form id=f><image>",
            Some("f"),
            true,
            ParserAssociated,
        ),
        (
            "<!doctype html><form id=f><script></form></script><img>",
            Some("f"),
            true,
            ParserAssociated,
        ),
        (
            "<!doctype html><form id=f><div></form><img></div>",
            Some("f"),
            false,
            Reset,
        ),
        (
            "<!doctype html><form id=f><div></form><form id=g></div><img>",
            Some("g"),
            true,
            ParserAssociated,
        ),
    ] {
        assert_eq!(
            form_image_observations(source),
            vec![(
                owner.map(|id| id.encode_utf16().collect()),
                inserted,
                evidence
            )],
            "{source}"
        );
    }
}

#[test]
fn ordinary_form_adoption_resets_images_detached_from_their_form_owner() {
    use crate::construction::parser_session::ImageAssociationEvidence::{ParserAssociated, Reset};
    for (source, inserted, evidence) in [
        (
            "<!doctype html><form id=f><b>1<div><img></b>2</div></form>",
            false,
            Reset,
        ),
        (
            "<!doctype html><b><div><form id=f><img>",
            true,
            ParserAssociated,
        ),
        (
            "<!doctype html><b><div><form id=f><img></b></form></div>",
            // The first outer pass moves the entire form subtree. The next
            // pass makes form the furthest block and moves its img child into
            // a new b, so removal resets the flag before insertion resolves f.
            false,
            Reset,
        ),
    ] {
        assert_eq!(
            form_image_observations(source),
            vec![(Some(vec![b'f' as u16]), inserted, evidence)],
            "{source}"
        );
    }
}

#[test]
fn ordinary_form_and_image_creation_preserve_literal_utf16_values_and_qualified_owner() {
    use crate::construction::parser_session::ImageAssociationEvidence::ParserAssociated;
    let form_id = [0xD83D, 0xDE42, 0xD800];
    let alternative = [0xDC00, b'A' as u16, 0xD83D, 0xDE80];
    let text = [0xD800, 0xD83D, 0xDE42, 0xDC00];
    let mut units: Vec<u16> = "<!doctype html><form id=\"".encode_utf16().collect();
    units.extend_from_slice(&form_id);
    units.extend("\"><img alt=\"".encode_utf16());
    units.extend_from_slice(&alternative);
    units.extend("\">".encode_utf16());
    units.extend_from_slice(&text);
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    {
        let document = frozen.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let form = document.node(&body).unwrap().children().next().unwrap();
        let img = document.node(&form).unwrap().children().next().unwrap();
        let character = document.node(&form).unwrap().children().nth(1).unwrap();
        let owner = frozen.image_form_owner(&mut c, &img).unwrap().unwrap();
        assert_eq!(owner, form);
        let state = frozen.image_form_state(&mut c, &img).unwrap();
        assert!(state.parser_inserted);
        assert_eq!(state.evidence, ParserAssociated);
        let HtmlNodeKind::Element(form) = document.node(&form).unwrap().kind() else {
            panic!("actual form element")
        };
        let HtmlNodeKind::Element(img) = document.node(&img).unwrap().kind() else {
            panic!("actual image element")
        };
        assert_eq!(
            form.attributes().as_slice()[0].value().code_units(),
            &form_id
        );
        assert_eq!(
            img.attributes().as_slice()[0].value().code_units(),
            &alternative
        );
        let HtmlNodeKind::Text(character) = document.node(&character).unwrap().kind() else {
            panic!("actual exact text")
        };
        assert_eq!(character.code_units(), &text);
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ordinary_form_control_and_template_continuations_preserve_pointer_and_original_token() {
    for (name, expected) in [
        ("fieldset", super::head::HeadBoundary::Form),
        ("button", super::head::HeadBoundary::Form),
        ("input", super::head::HeadBoundary::Form),
        ("textarea", super::head::HeadBoundary::Form),
        ("output", super::head::HeadBoundary::Form),
        ("object", super::head::HeadBoundary::Form),
        ("template", super::head::HeadBoundary::Template),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!("<!doctype html><form id=f><{name} id=kept>")),
            limits(100),
        );
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(standalone_boundary(&mut driver), expected, "{name}");
        {
            let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
            assert!(state.current_form.is_some());
            assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                6
            );
            let Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload else {
                panic!("unadmitted control retains the actual tag")
            };
            assert_eq!(tag.kind, TagKind::Start);
            assert_eq!(
                tag.name.value.text().code_units(),
                &name.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                tag.attributes.as_slice()[0].value.value.text().code_units(),
                &"kept".encode_utf16().collect::<Vec<_>>()
            );
        }
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

// Governing HTML877d syntax4630–4638,5110–5114,5322–5498:
// omitted tbody and td end tags recover to two cells in one real row.

#[test]
fn ordinary_table_implied_tbody_and_cell_end_recovery_preserve_real_sibling_order() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table><tr><td>1<td>2</table>3"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadBoundary::ConstructionEof {
        // The existing driver retains the actual table opener at Table. Dispose
        // its real tree/token/controller owners before the runtime assertion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadBoundary::ConstructionEof);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_outline = outline(document);
    let actual_mode = document.mode();
    let doctype_count = document
        .root()
        .children()
        .filter(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Doctype(_)))
        .count();
    // A real doctype precedes html; select the actual element rather than
    // treating the first document child as an element.
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let table = document.node(&body).unwrap().children().next().unwrap();
    let tbody = document.node(&table).unwrap().children().next().unwrap();
    let row = document.node(&tbody).unwrap().children().next().unwrap();
    let cells: Vec<_> = document.node(&row).unwrap().children().collect();
    let element_facts: Vec<_> = [&table, &tbody, &row]
        .into_iter()
        .chain(cells.iter())
        .map(|id| {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("source-derived table positions must be actual elements");
            };
            (
                element
                    .name()
                    .namespace()
                    .uri()
                    .map(|uri| uri.code_units().to_vec()),
                element.name().local_name().code_units().to_vec(),
                element.attributes().len(),
            )
        })
        .collect();

    // Native observations own plain units and qualified weak IDs, not admitted
    // string clones. Release the actual immutable owner before assertions.
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(doctype_count, 1);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#1"),
            (5, "td"),
            (6, "#2"),
            (2, "#3"),
        ])
    );
    assert_eq!(cells.len(), 2);
    assert_ne!(cells[0], cells[1]);
    assert_ne!(table, tbody);
    assert_ne!(tbody, row);
    assert_eq!(
        element_facts,
        ["table", "tbody", "tr", "td", "td"]
            .into_iter()
            .map(|name| (
                Some(
                    "http://www.w3.org/1999/xhtml"
                        .encode_utf16()
                        .collect::<Vec<_>>()
                ),
                name.encode_utf16().collect::<Vec<_>>(),
                0,
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn structural_table_whole_pending_batch_fosters_whitespace_with_nonwhitespace() {
    for (source, expected) in [
        (
            "<!doctype html><table> a<tr><td>b</table>c",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "# a"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#b"),
                (2, "#c"),
            ],
        ),
        (
            "<!doctype html><table> \n<tr><td>b</table>c",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "table"),
                (3, "# \n"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#b"),
                (2, "#c"),
            ],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        let (mut c, mut frozen) = body_document(driver);
        let actual = outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected_outline(&expected));
    }
}

#[test]
fn structural_table_implied_colgroup_row_and_thead_transitions_are_real() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><table><col><col><thead><td>a<tbody><tr><th>b<tfoot><tr><td>c</table>d",
        ),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(
        actual,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "colgroup"),
            (4, "col"),
            (4, "col"),
            (3, "thead"),
            (4, "tr"),
            (5, "td"),
            (6, "#a"),
            (3, "tbody"),
            (4, "tr"),
            (5, "th"),
            (6, "#b"),
            (3, "tfoot"),
            (4, "tr"),
            (5, "td"),
            (6, "#c"),
            (2, "#d")
        ])
    );
}

#[test]
fn structural_table_caption_and_cell_markers_bound_formatting_reconstruction() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table><caption><b>a<tr><td><i>b<td>c</table>d"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(
        actual,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "caption"),
            (4, "b"),
            (5, "#a"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "i"),
            (7, "#b"),
            (5, "td"),
            (6, "#c"),
            (2, "#d")
        ])
    );
}

#[test]
fn structural_table_nested_table_restores_real_cell_mode_then_body() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table><tr><td>a<table><tr><td>b</table>c</table>d"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(
        actual,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#a"),
            (6, "table"),
            (7, "tbody"),
            (8, "tr"),
            (9, "td"),
            (10, "#b"),
            (6, "#c"),
            (2, "#d")
        ])
    );
}

#[test]
fn structural_table_entry_uses_actual_quirks_document_mode() {
    for (source, expected) in [
        (
            "<p>a<table><tr><td>b</table>c",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "p"),
                (3, "#a"),
                (3, "table"),
                (4, "tbody"),
                (5, "tr"),
                (6, "td"),
                (7, "#b"),
                (3, "#c"),
            ],
        ),
        (
            "<!doctype html><p>a<table><tr><td>b</table>c",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "p"),
                (3, "#a"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#b"),
                (2, "#c"),
            ],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadBoundary::ConstructionEof
        );
        let (mut c, mut frozen) = body_document(driver);
        let actual = outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected_outline(&expected));
    }
}

#[test]
fn structural_table_head_delegation_retains_actual_script_and_style_return_mode() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><table><style>s</style><script>x</script><tr><td>a</table>b",
        ),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut returns = Vec::new();
    for _ in 0..1000 {
        let step = driver.step().unwrap();
        if let Some(head) = &driver.head {
            let state = head.state.as_ref().unwrap();
            if state.mode == super::head::InsertionMode::Text {
                returns.push(state.return_mode);
            }
        }
        if matches!(
            step,
            super::head::HeadStep::Boundary(super::head::HeadBoundary::ConstructionEof)
        ) {
            break;
        }
    }
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(
        state.boundary,
        Some(super::head::HeadBoundary::ConstructionEof)
    );
    assert!(!state.before.kernel.foster_parenting);
    let (mut c, mut frozen) = body_document(driver);
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert!(!returns.is_empty());
    assert!(
        returns
            .iter()
            .all(|mode| *mode == Some(super::head::InsertionMode::InTable))
    );
    assert_eq!(
        actual,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "style"),
            (4, "#s"),
            (3, "script"),
            (4, "#x"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#a"),
            (2, "#b")
        ])
    );
}

#[test]
fn structural_table_unsupported_creation_retains_original_before_table_effects() {
    for (tag, boundary) in [
        ("<input type=hidden>", super::head::HeadBoundary::Form),
        ("<input>", super::head::HeadBoundary::Form),
        ("<select>", super::head::HeadBoundary::Select),
        ("<template>", super::head::HeadBoundary::Template),
        ("<svg>", super::head::HeadBoundary::Foreign),
    ] {
        let source = format!("<!doctype html><table>{tag}");
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        assert_eq!(standalone_boundary(&mut driver), boundary);
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.mode, super::head::InsertionMode::InTable);
        assert!(!state.before.kernel.foster_parenting);
        assert_eq!(state.diagnostics.len(), 0);
        assert!(matches!(
            state.pending.as_ref().unwrap().payload,
            Payload::Tag(Some(_))
        ));
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn structural_table_null_preprocessing_and_literal_utf16_are_retained_exactly() {
    let prefix = "<!doctype html><table>";
    let mut units: Vec<u16> = prefix.encode_utf16().collect();
    let null_start = units.len();
    units.extend([0, 0x20, 0x0d, 0x0a, 0xd83d, 0xde00, 0xd800, 0x51, 0xdc00]);
    units.extend("<tr><td>x</table>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let null_ranges: Vec<_> = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .diagnostics
        .as_slice()
        .iter()
        .filter(|diagnostic| diagnostic.condition == super::head::HeadCondition::NullInTable)
        .map(|diagnostic| (diagnostic.source.start(), diagnostic.source.end()))
        .collect();
    let (mut c, mut frozen) = body_document(driver);
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(null_ranges, vec![(null_start, null_start + 1)]);
    assert_eq!(
        actual[3],
        (
            2,
            vec![
                b'#' as u16,
                0x20,
                0x0a,
                0xd83d,
                0xde00,
                0xd800,
                0x51,
                0xdc00
            ]
        )
    );
}

#[test]
fn structural_table_buffered_collection_cancellation_releases_every_real_owner() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table> a<tr><td>b</table>"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    let mut collected = false;
    for _ in 0..100 {
        driver.step().unwrap();
        if let Some(head) = &driver.head {
            let state = head.state.as_ref().unwrap();
            if state.mode == super::head::InsertionMode::InTableText
                && state.table.entries.len() != 0
            {
                collected = true;
                break;
            }
        }
    }
    assert!(collected);
    request.cancellation().cancel();
    assert!(driver.step().is_err());
    let stop = driver.control.stop().unwrap();
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), Some(stop));
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.stop(), Some(stop));
}

#[test]
fn structural_table_work_exhaustion_retains_coherent_pending_and_flush_owners() {
    // Finite independent caps exercise tokenizer, collection, list classification,
    // direct insertion, fostered flush and clearing without using usage to derive
    // a purported exact grammar budget. Every rejected prefix must dispose fully.
    for cap in [200, 500, 1000, 2000, 5000, 10000, 20000, 50000] {
        let bounded = limits(100)
            .into_builder()
            .work_credits(cap)
            .finish()
            .unwrap();
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><table> a<tr><td><b>b</table>c"),
            bounded,
        );
        let mut driver = match StandaloneDocumentDriver::new(&request) {
            Ok(driver) => driver,
            Err(_) => continue,
        };
        for _ in 0..1000 {
            match driver.step() {
                Ok(super::head::HeadStep::Boundary(_)) => break,
                Ok(_) => {}
                Err(_) => break,
            }
        }
        let first = driver.control.stop();
        if first.is_some() {
            assert!(driver.step().is_err());
        }
        // Normal disposal can itself reach the finite work stop; retry only the
        // real terminal cleanup path, without resuming useful parser work.
        if driver.release().is_err() {
            driver.release().unwrap();
        }
        assert_released(&driver.control);
        if let Some(first) = first {
            assert_eq!(driver.control.stop(), Some(first));
        }
    }
}

#[test]
fn structural_table_disposal_stop_after_vector_pop_keeps_nested_ticket_reachable() {
    let selected = limits(100);
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table> a<tr>"),
        selected,
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    for _ in 0..100 {
        driver.step().unwrap();
        if let Some(head) = &driver.head {
            let state = head.state.as_ref().unwrap();
            if state.mode == super::head::InsertionMode::InTableText
                && state.table.entries.len() != 0
            {
                break;
            }
        }
    }
    assert!(
        driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .table
            .entries
            .len()
            != 0
    );
    // Fixture-positioning only: allow the independently known nonempty owning
    // pop cost (one visit, one write, the fixed Option<TableCharacter> move),
    // then reject further normal work. The expectation is ticket reachability,
    // not a usage-derived claimed total tokenizer cost.
    let pop_cost = 2 + std::mem::size_of::<Option<super::tables::TableCharacter>>() as u64;
    let remaining = selected.work_credits()
        - driver
            .control
            .usage()
            .resource(crate::HtmlParseResource::WorkCredits)
            .current;
    driver
        .control
        .charge(WorkCategory::StateStep, remaining - pop_cost)
        .unwrap();
    assert!(
        driver
            .head
            .as_mut()
            .unwrap()
            .state
            .as_mut()
            .unwrap()
            .release_table(&mut driver.control)
            .is_err()
    );
    let first = driver.control.stop().unwrap();
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.stop(), Some(first));
}

#[test]
fn structural_table_diagnostic_caps_preserve_whole_batch_or_coherent_flush_prefix() {
    // Document, doctype, html, head, body and table precede flush.
    // Two diagnostics admit the first fostered whitespace text before stop.
    for (cap, nodes) in [(0, 6), (1, 6), (2, 7)] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><table> a<tr>"),
            limits(100)
                .into_builder()
                .diagnostics(cap)
                .finish()
                .unwrap(),
        );
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let mut stopped = false;
        for _ in 0..100 {
            if driver.step().is_err() {
                stopped = true;
                break;
            }
        }
        assert!(stopped);
        let first = driver.control.stop().unwrap();
        assert!(matches!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            }
        ));
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::ManagedNodes)
                .current,
            nodes
        );
        assert!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current
                > 0
        );
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.mode, super::head::InsertionMode::InTableText);
        assert!(matches!(
            state.pending.as_ref().unwrap().payload,
            Payload::Tag(Some(_))
        ));
        assert!(driver.step().is_err());
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.stop(), Some(first));
    }
}

#[test]
fn structural_table_form_and_fostered_image_keep_actual_nonancestor_form_owner() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table><form id=f><img><tr><td>x</table>y"),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let image = document.node(&body).unwrap().children().next().unwrap();
    let table = document.node(&body).unwrap().children().nth(1).unwrap();
    let form = document.node(&table).unwrap().children().next().unwrap();
    let state = frozen.image_form_state(&mut c, &image).unwrap();
    let owner = frozen.image_form_owner(&mut c, &image).unwrap();
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert!(state.parser_inserted);
    assert_eq!(owner, Some(form));
    assert_eq!(
        actual,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "img"),
            (2, "table"),
            (3, "form"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#x"),
            (2, "#y")
        ])
    );
}

#[test]
fn structural_table_real_comment_pi_and_malformed_column_and_row_recovery() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><table><!--c--><?work d?><colgroup> <col></col></colgroup><thead></tbody><tr></tfoot><td>x</th></td></table>y",
        ),
        limits(100),
    );
    let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadBoundary::ConstructionEof
    );
    let table_errors = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .diagnostics
        .as_slice()
        .iter()
        .filter(|d| d.condition == super::head::HeadCondition::UnexpectedTableToken)
        .count();
    let (mut c, mut frozen) = body_document(driver);
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(table_errors, 4);
    assert_eq!(
        actual,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "!c"),
            (3, "?work"),
            (3, "colgroup"),
            (4, "# "),
            (4, "col"),
            (3, "thead"),
            (4, "tr"),
            (5, "td"),
            (6, "#x"),
            (2, "#y")
        ])
    );
}
