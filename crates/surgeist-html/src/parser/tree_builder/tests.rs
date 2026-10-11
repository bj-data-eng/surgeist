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
            token::{Event, TokenKind},
        },
    },
};

fn limits(open: u64) -> HtmlParseLimits {
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
fn assert_released(control: &ParseControl) {
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::Diagnostics,
        HtmlParseResource::OpenElements,
        HtmlParseResource::ManagedCapacityBytes,
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
    let head = document.node(&html.children().last().unwrap()).unwrap();
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
    let head = document.node(&html.children().last().unwrap()).unwrap();
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
    use super::head::{HeadBoundary, HeadMode};
    let source = HtmlSource::utf8(
        "<!doctype html><html> <!--outside--><?x a??><head>\r\n<!--inside--><?y b?><base/><basefont><bgsound/><link/><meta charset=utf-16></head>",
    );
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    assert_eq!(output.boundary, HeadBoundary::AfterHead);
    let state = output.driver.state.as_ref().unwrap();
    assert_eq!(state.mode, HeadMode::AfterHead);
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
    let head = document.node(&html.children().last().unwrap()).unwrap();
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
    use super::head::{HeadBoundary, HeadMode};
    let source = HtmlSource::utf8("<body x='kept'>tail");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    assert_eq!(output.boundary, HeadBoundary::AfterHead);
    let state = output.driver.state.as_ref().unwrap();
    assert_eq!(state.mode, HeadMode::AfterHead);
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
    use super::head::{HeadCondition, HeadMode};
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
    assert_eq!(diagnostics[0].mode, HeadMode::InHeadNoscript);
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
    use super::head::{HeadCondition, HeadMode};
    let source = HtmlSource::utf8("<title>unfinished&amp;");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let state = output.driver.state.as_ref().unwrap();
    let d = &state.diagnostics.as_slice()[0];
    assert_eq!(d.condition, HeadCondition::EofInText);
    assert_eq!(d.mode, HeadMode::Text);
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
    use super::head::HeadMode;
    let source = HtmlSource::utf8("<!doctype html><html><head><title>x</title>");
    let mut c = control(2); // html + head fit; the mandatory text-element push does not.
    let mut driver = head_stage(&source, &mut c);
    driver.step(&mut c).unwrap();
    assert!(driver.step(&mut c).is_err());
    assert!(c.stop().is_some());
    assert_eq!(driver.state.as_ref().unwrap().mode, HeadMode::InHead);
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
    use super::head::{HeadCondition, HeadMode};
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
                && d.mode == HeadMode::InHeadNoscript)
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
    use super::head::{HeadError, HeadMode};
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
    assert_eq!(driver.state.as_ref().unwrap().mode, HeadMode::InHead);
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
