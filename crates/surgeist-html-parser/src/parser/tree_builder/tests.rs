use super::prologue::Payload;
use super::*;
use crate::{
    DocumentString, HtmlConstructionLimits, HtmlDocumentMode, HtmlFragment, HtmlNodeKind,
    HtmlParseCancellation, HtmlParseLimits, HtmlScriptKind, HtmlSource, Namespace,
    OrderedAttributes, QualifiedName,
    construction::{FrozenArena, RootKind},
    parser::{
        control::Invariant,
        tokenizer::{
            Tokenizer,
            states::State,
            token::{Event, TagKind, TokenKind},
        },
    },
};

pub(crate) fn limits(open: u64) -> HtmlParseLimits {
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
fn next_kind(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    control: &mut ParseControl,
) -> TokenKind {
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
fn tag(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    control: &mut ParseControl,
) -> Option<Tag> {
    let TokenKind::Tag(tag) = next_kind(tokenizer, control) else {
        panic!("expected tag")
    };
    Some(tag)
}
fn text(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    control: &mut ParseControl,
) -> Option<Text> {
    let TokenKind::Text(text) = next_kind(tokenizer, control) else {
        panic!("expected text")
    };
    Some(text)
}
fn begin(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    kind: RootKind,
    control: &mut ParseControl,
) -> (LocationKernel, ParserNode) {
    let document = matches!(kind, RootKind::Document(_));
    let mut session = ParserArenaSession::new(control, kind).unwrap();
    let mut html = tag(tokenizer, control);
    let html = session.allocate_html_tag(control, &mut html).unwrap();
    if document {
        let root = session.root(control).unwrap();
        session.insert(control, &root, &html, None, None).unwrap();
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
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    push: StackPush,
    control: &mut ParseControl,
) -> ParserNode {
    kernel
        .insert_html_tag(&mut tag(tokenizer, control), push, None, control, None)
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
        mut session,
        open,
        templates,
        ..
    } = kernel;
    open.unwrap().release(control).unwrap();
    templates.unwrap().release(control).unwrap();
    session.freeze(control).unwrap()
}
pub(crate) fn assert_released(control: &ParseControl) {
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
                    .insert_text(&mut input, None, &mut c, None)
                    .unwrap()
                    .unwrap();
                assert_eq!(result._offset, offset);
                assert!(input.is_none());
                offset += length;
            }
            TokenKind::Eof => break,
            _ => panic!("expected remaining text"),
        }
    }
    let popped = kernel.pop(&mut c, None).unwrap().unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let mut a = text(&mut tokenizer, &mut c);
    let first = kernel
        .insert_text(&mut a, None, &mut c, None)
        .unwrap()
        .unwrap();
    let table = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    kernel.set_foster_parenting(true, &mut c).unwrap();
    let location = kernel.adjusted_location(None, &mut c).unwrap();
    same(&kernel, &location.parent, &body, &mut c);
    same(&kernel, location.before.as_ref().unwrap(), &table, &mut c);
    let mut b = text(&mut tokenizer, &mut c);
    let second = kernel
        .insert_text(&mut b, None, &mut c, None)
        .unwrap()
        .unwrap();
    same(&kernel, &first._node, &second._node, &mut c);
    assert_eq!(second._offset, 1);
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let table = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    kernel.session.remove(&mut c, &table, None).unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, html) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let table = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    kernel
        .session
        .insert(&mut c, &html, &table, None, None)
        .unwrap();
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
        let mut tokenizer =
            crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
                .unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    kernel.insert_text(&mut input, None, &mut c, None).unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
        .insert_text(&mut text(&mut tokenizer, &mut c), None, &mut c, None)
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
        kernel.pop(&mut c, None).unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let mut first_node = None;
    loop {
        match next_kind(&mut tokenizer, &mut c) {
            TokenKind::Text(input) => {
                let result = kernel
                    .insert_text(&mut Some(input), None, &mut c, None)
                    .unwrap()
                    .unwrap();
                if let Some(first) = &first_node {
                    same(&kernel, &result._node, first, &mut c);
                } else {
                    first_node = Some(result._node);
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
fn pushing_a_node_created_after_every_pushed_node_examines_no_stack_entry() {
    let source = HtmlSource::utf8(format!("<html>{}", "<div>".repeat(40)));
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let mut steps = Vec::new();
    for _ in 0..40 {
        let node = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
        let before = c.usage().work(WorkCategory::StateStep);
        kernel.push(&node, &mut c).unwrap();
        steps.push(c.usage().work(WorkCategory::StateStep) - before);
    }
    assert_eq!(steps, vec![0; 40]);
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn a_node_already_on_the_stack_is_a_duplicate_wherever_it_sits_and_however_it_got_there() {
    let source = HtmlSource::utf8("<html><body><p><span><b>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, html) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let p = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    for on_stack in [&html, &body, &p] {
        assert!(matches!(
            kernel.push(on_stack, &mut c),
            Err(KernelError::DuplicateOpenElement)
        ));
    }
    // A popped node may be pushed again, and then it is a duplicate again.
    kernel.pop(&mut c, None).unwrap();
    kernel.push(&p, &mut c).unwrap();
    assert!(matches!(
        kernel.push(&p, &mut c),
        Err(KernelError::DuplicateOpenElement)
    ));
    // A node written into the stack by replacement is on the stack too.
    let span = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
    kernel.replace_stack(2, &span, &mut c, None).unwrap();
    assert!(matches!(
        kernel.push(&span, &mut c),
        Err(KernelError::DuplicateOpenElement)
    ));
    // And so is one written by insertion at a position below the top.
    let b = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
    kernel.insert_stack(1, &b, &mut c).unwrap();
    assert!(matches!(
        kernel.push(&b, &mut c),
        Err(KernelError::DuplicateOpenElement)
    ));
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

/// Applies a long deterministic sequence of every stack operation to a kernel
/// and to a plain vector of the same nodes, and requires the kernel's answer to
/// "which template is last" to be the vector's after each one. The vector's
/// answer is a scan from the top, so it is independent of how the kernel
/// derives its own.
#[test]
fn the_last_template_is_the_topmost_open_template_after_every_stack_operation() {
    const POOL: [&str; 12] = [
        "<template>",
        "<div>",
        "<template>",
        "<div>",
        "<div>",
        "<template>",
        "<div>",
        "<template>",
        "<div>",
        "<div>",
        "<template>",
        "<div>",
    ];
    let source = HtmlSource::utf8(format!("<html>{}", POOL.concat()));
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let pool: Vec<(ParserNode, bool)> = POOL
        .iter()
        .map(|tag| {
            let node = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
            (node, *tag == "<template>")
        })
        .collect();
    // `open` holds pool indices; the html element below them is not a template.
    let mut open: Vec<usize> = Vec::new();
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut pick = |bound: usize| {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((seed >> 33) as usize) % bound
    };
    let mut seen_template_last = false;
    let mut seen_no_template = false;
    let mut ran = [0_u32; 5];
    for step in 0..600 {
        let unused: Vec<usize> = (0..pool.len()).filter(|id| !open.contains(id)).collect();
        let operation = pick(5);
        match operation {
            0 if !unused.is_empty() => {
                let id = unused[pick(unused.len())];
                kernel.push(&pool[id].0, &mut c).unwrap();
                open.push(id);
            }
            1 if !open.is_empty() => {
                kernel.pop(&mut c, None).unwrap();
                open.pop();
            }
            2 if !open.is_empty() && !unused.is_empty() => {
                let at = pick(open.len());
                let id = unused[pick(unused.len())];
                kernel
                    .replace_stack(at + 1, &pool[id].0, &mut c, None)
                    .unwrap();
                open[at] = id;
            }
            3 if !unused.is_empty() => {
                let at = pick(open.len() + 1);
                let id = unused[pick(unused.len())];
                kernel.insert_stack(at + 1, &pool[id].0, &mut c).unwrap();
                open.insert(at, id);
            }
            4 if !open.is_empty() => {
                let at = pick(open.len());
                kernel
                    .remove_stack_identity(&pool[open[at]].0, &mut c, None)
                    .unwrap();
                open.remove(at);
            }
            _ => continue,
        }
        ran[operation] += 1;
        assert_eq!(kernel.stack().unwrap().len(), open.len() + 1, "step {step}");
        for (index, id) in open.iter().enumerate() {
            let node = kernel.at(index + 1, &mut c).unwrap();
            same(&kernel, &node, &pool[*id].0, &mut c);
        }
        let listed: Vec<usize> = open
            .iter()
            .enumerate()
            .filter(|(_, id)| pool[**id].1)
            .map(|(at, _)| at + 1)
            .collect();
        assert_eq!(kernel.template_indices(), listed, "step {step}");
        let expected = open.iter().rposition(|id| pool[*id].1);
        let actual = kernel.last_template(&mut c).unwrap();
        match (expected, actual) {
            (None, None) => seen_no_template = true,
            (Some(at), Some(node)) => {
                same(&kernel, &node, &pool[open[at]].0, &mut c);
                seen_template_last = true;
            }
            (expected, actual) => panic!(
                "step {step}: expected stack index {expected:?}, found {}",
                actual.is_some()
            ),
        }
        assert_eq!(
            kernel.contains_html_template(&mut c).unwrap(),
            expected.is_some(),
            "step {step}"
        );
    }
    // The sequence visited both answers, so the comparison was not vacuous.
    assert!(seen_template_last && seen_no_template);
    assert!(ran.iter().all(|count| *count > 20), "{ran:?}");
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn checking_a_new_element_for_insertion_costs_the_same_at_every_depth() {
    // A new element has no children, so it can be a host-including ancestor of
    // the target only by being the target: the check visits no ancestor.
    let chunks = [5_usize, 15, 60];
    let tags: String = chunks
        .iter()
        .map(|divs| format!("{}<p>", "<div>".repeat(*divs)))
        .collect();
    let source = HtmlSource::utf8(format!("<html>{tags}"));
    let mut c = control(200);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let mut visits = Vec::new();
    for divs in chunks {
        for _ in 0..divs {
            add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
        }
        let mut p = tag(&mut tokenizer, &mut c);
        let node = kernel.session.allocate_html_tag(&mut c, &mut p).unwrap();
        let top = kernel.current(&mut c).unwrap();
        let before = c.usage().work(WorkCategory::RecordVisit);
        let ignored = kernel
            .session
            .element_insertion_ignored(&mut c, &node, &top)
            .unwrap();
        assert!(ignored.is_none());
        visits.push(c.usage().work(WorkCategory::RecordVisit) - before);
    }
    assert!(
        visits.windows(2).all(|pair| pair[0] == pair[1]),
        "visits at depths {chunks:?}: {visits:?}"
    );
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn popping_the_last_template_costs_the_same_at_every_depth() {
    // Each chunk opens more elements and a template on top of the previous
    // template; popping it makes the previous one last again, wherever it lies.
    let chunks = [5_usize, 15, 60];
    let tags: String = chunks
        .iter()
        .map(|divs| format!("{}<template>", "<div>".repeat(*divs)))
        .collect();
    let source = HtmlSource::utf8(format!("<html><template>{tags}"));
    let mut c = control(200);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let mut costs = Vec::new();
    for divs in chunks {
        for _ in 0..divs {
            add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
        }
        let template = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
        let last = kernel.last_template(&mut c).unwrap().unwrap();
        same(&kernel, &last, &template, &mut c);
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        kernel.pop(&mut c, None).unwrap();
        costs.push(c.usage().resource(HtmlParseResource::WorkCredits).current - before);
        // The template below is last again.
        assert!(kernel.last_template(&mut c).unwrap().is_some());
    }
    assert!(
        costs.windows(2).all(|pair| pair[0] == pair[1]),
        "work of popping a template at depths {chunks:?}: {costs:?}"
    );
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn finding_a_listed_template_charges_each_probe_of_the_search() {
    // Sixteen nested templates are listed at stack indices 1 through 16, and a
    // div sits on top at 17. The search probes the middle of the range still
    // open, `low + (high - low) / 2` over the listing positions, and stops at
    // the first probe that finds the entry; each probe is one state step.
    // Replacing or removing a listed entry searches the list once, to learn
    // that the entry is listed and where, and unlists it at that position.
    //
    // 1. Replace the template at stack index 9 (position 8 of 16): the first
    //    probe is position 8, which holds 9, so the search takes 1 probe: 1.
    //    Unlisting leaves 15 listings: 1-8 and 10-16.
    // 2. Remove the template at index 8 (position 7 of 15): the first probe is
    //    position 7, which holds 8, so the search takes 1 probe: 1. The
    //    entries above move down, leaving 14 listings: 1-7 and 9-15.
    // 3. Replace the div, now at index 16, which is not listed. The range
    //    0..14 probes position 7 (holds 9, less than 16), then 8..14 probes
    //    position 11 (holds 13), then 12..14 probes position 13 (holds 15),
    //    and 14..14 is empty: 3 probes and not found. Nothing is unlisted: 3.
    let source = HtmlSource::utf8(format!("<html>{}<div><div><div>", "<template>".repeat(16)));
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    for _ in 0..17 {
        add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    }
    assert_eq!(kernel.template_indices(), (1..=16).collect::<Vec<_>>());
    let steps = |c: &ParseControl| c.usage().work(WorkCategory::StateStep);

    let first = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
    let before = steps(&c);
    kernel.replace_stack(9, &first, &mut c, None).unwrap();
    assert_eq!(steps(&c) - before, 1);
    assert_eq!(
        kernel.template_indices(),
        (1..=8).chain(10..=16).collect::<Vec<_>>()
    );

    let eighth = kernel.at(8, &mut c).unwrap();
    let before = steps(&c);
    kernel.remove_stack_identity(&eighth, &mut c, None).unwrap();
    assert_eq!(steps(&c) - before, 1);
    assert_eq!(
        kernel.template_indices(),
        (1..=7).chain(9..=15).collect::<Vec<_>>()
    );

    let second = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
    let before = steps(&c);
    kernel.replace_stack(16, &second, &mut c, None).unwrap();
    assert_eq!(steps(&c) - before, 3);
    assert_eq!(kernel.template_indices().len(), 14);
    tokenizer.release(&mut c).unwrap();
    kernel.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn the_listing_search_agrees_with_the_standard_search_and_counts_its_probes() {
    // Odd listings, so every query is found, between two listings, or beyond
    // either end. The result must be the standard search's; the probes must be
    // at least one and at most one more than the halvings of the range, and
    // none for an empty list.
    for len in 0..=40_usize {
        let listed: Vec<usize> = (0..len).map(|at| 2 * at + 1).collect();
        let halvings = (usize::BITS - len.leading_zeros()) as usize;
        for query in 0..=2 * len + 1 {
            let mut probes = 0;
            let found = search_listing(&listed, query, || {
                probes += 1;
                Ok(())
            })
            .unwrap();
            assert_eq!(
                found,
                listed.binary_search(&query),
                "{len} listings, {query}"
            );
            if len == 0 {
                assert_eq!(probes, 0);
            } else {
                assert!(
                    (1..=halvings).contains(&probes),
                    "{len} listings, {query}: {probes}"
                );
            }
        }
    }
    // A refused probe stops the search before it reads a listing.
    let mut allowed = 2;
    let refused = search_listing(&[1, 3, 5, 7, 9, 11, 13, 15], 15, || {
        if allowed == 0 {
            return Err(ParseStop::Cancelled);
        }
        allowed -= 1;
        Ok(())
    });
    assert_eq!(refused, Err(ParseStop::Cancelled));
}

#[test]
fn a_failed_stack_release_takes_the_templates_and_clears_the_root_target() {
    let source = HtmlSource::utf8("<html><template>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    assert_eq!(kernel.template_indices(), [1]);
    assert!(kernel.root_target.is_some());
    // A foreign controller cannot release the stack, which is the first failure.
    let mut foreign = control(100);
    assert!(kernel.release_stack(&mut foreign).is_err());
    // The refusal handed back neither owner: both were taken and the target
    // was dropped, rather than the first refusal leaving the others behind.
    assert!(kernel.open.is_none());
    assert!(kernel.templates.is_none());
    assert!(kernel.root_target.is_none());
}

#[test]
fn a_frozen_stack_release_without_the_open_elements_reports_consumed_and_takes_the_rest() {
    let source = HtmlSource::utf8("<html><template>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    assert_eq!(kernel.template_indices(), [1]);
    // Private-state fault injection: the open-element storage is already gone,
    // released through its own owner, while the listings are still retained.
    kernel.open.take().unwrap().release(&mut c).unwrap();
    assert!(matches!(
        kernel.release_frozen_stack(&mut c),
        Err(KernelError::Consumed)
    ));
    assert!(kernel.templates.is_none());
    assert!(kernel.root_target.is_none());
    tokenizer.release(&mut c).unwrap();
    kernel.session.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn empty_wrong_kind_duplicate_and_foreign_handles_preserve_stack() {
    let source = HtmlSource::utf8("<html><body>");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    assert!(matches!(
        kernel.pop(&mut c, None),
        Err(KernelError::Consumed)
    ));
    assert_released(&c);
}

#[test]
fn exact_stack_cap_rejection_keeps_permanent_nodes_and_cleans_all_live_storage() {
    let source = HtmlSource::utf8("<html><body>");
    let mut c = control(1);
    c.admit_source(&source).unwrap();
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
            .insert_text(&mut input, Some(&document), &mut c, None)
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let document = kernel.session.root(&mut c).unwrap();
    let mut input = tag(&mut tokenizer, &mut c);
    let created = kernel
        .insert_html_tag(&mut input, StackPush::NoPush, Some(&document), &mut c, None)
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
            None,
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
        .insert_text(&mut text(&mut tokenizer, &mut c), None, &mut c, None)
        .unwrap()
        .unwrap();
    let parent = kernel
        .session
        .parent(&mut c, &inserted._node)
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, html) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    kernel.session.remove(&mut c, &html, None).unwrap();
    let document = kernel.session.root(&mut c).unwrap();
    let created = kernel
        .insert_html_tag(
            &mut tag(&mut tokenizer, &mut c),
            StackPush::NoPush,
            Some(&document),
            &mut c,
            None,
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, html) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let body = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let p = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    assert_eq!(
        kernel
            .insert_element(&p, Some(&html), &mut c, None)
            .unwrap(),
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
        kernel.insert_element(&node, None, &mut c, None).unwrap(),
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
        kernel
            .insert_element(&template, None, &mut c, None)
            .unwrap(),
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(
        &mut tokenizer,
        RootKind::Document(HtmlDocumentMode::NoQuirks),
        &mut c,
    );
    let document = kernel.session.root(&mut c).unwrap();
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    let mut input = tag(&mut tokenizer, &mut c);
    assert!(matches!(
        kernel.insert_html_tag(&mut input, StackPush::Push, Some(&document), &mut c, None),
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
        kernel.insert_html_tag(&mut input, StackPush::Push, Some(&document), &mut c, None),
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
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
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

fn next_owned_token(
    tokenizer: &mut Tokenizer<'_>,
    evidence: &mut crate::construction::evidence::ParseEvidence,
    c: &mut ParseControl,
) -> crate::parser::tokenizer::token::Token {
    loop {
        let crate::parser::tokenizer::TokenizerPull::Event(event) =
            tokenizer.next(c, false, evidence).unwrap()
        else {
            panic!("actual token");
        };
        match event {
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
        let mut input = Some(next_owned_token(
            &mut tokenizer,
            engine.kernel.as_mut().unwrap().session.evidence(c).unwrap(),
            c,
        ));
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
        crate::construction::release_token_kind(&mut kind, c).unwrap();
        assert!(kind.is_none());
    }
    while let Some(text) = output.whitespace.try_pop(c).unwrap() {
        text.release(c).unwrap();
    }
    output.whitespace.release(c).unwrap();
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
        assert!(output.diagnostics().is_empty());
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
    assert_eq!(output.diagnostics().len(), 1);
    assert_eq!(
        output.diagnostics()[0].condition,
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
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].condition,
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
    assert_eq!(output.diagnostics().len(), 1);
    let diagnostic = &output.diagnostics()[0];
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
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].condition,
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
    assert_eq!(output.diagnostics().len(), 2);
    for (diagnostic, condition, literal) in output
        .diagnostics()
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    crate::construction::release_token_kind(&mut kind, &mut c).unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let mut engine = DocumentPrologue::new(&source, &mut c).unwrap();
    let mut first = Some({
        let evidence = engine.kernel().unwrap().session.evidence(&mut c).unwrap();
        let event = tokenizer.next_in(&mut c, false, evidence).unwrap().unwrap();
        match event {
            crate::parser::tokenizer::token::Event::Token(value) => {
                value.into_token(&mut c).unwrap()
            }
            _ => panic!("literal prologue token"),
        }
    });
    engine.process(&mut first, &mut c).unwrap();
    let mut input = Some({
        let evidence = engine.kernel().unwrap().session.evidence(&mut c).unwrap();
        let event = tokenizer.next_in(&mut c, false, evidence).unwrap().unwrap();
        match event {
            crate::parser::tokenizer::token::Event::Token(value) => {
                value.into_token(&mut c).unwrap()
            }
            _ => panic!("literal prologue token"),
        }
    });
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
        assert_eq!(output.diagnostics().len(), 1);
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
        assert_eq!(output.diagnostics().len(), usize::from(error));
        if error {
            assert_eq!(
                output.diagnostics()[0].condition,
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
    assert_eq!(output.diagnostics().len(), 1);
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
    let mut tokenizer = crate::parser::tokenizer::tests::LexerFixture::new(
        &second,
        State::Data,
        None,
        &mut foreign,
    )
    .unwrap();
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
    crate::construction::release_token_kind(&mut kind, &mut foreign).unwrap();
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
    let mut tokenizer = crate::parser::tokenizer::tests::LexerFixture::new(
        &source,
        State::Data,
        None,
        &mut foreign,
    )
    .unwrap();
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
    crate::construction::release_token_kind(&mut kind, &mut foreign).unwrap();
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
        let mut token = Some(next_owned_token(
            &mut tokenizer,
            prologue
                .kernel
                .as_mut()
                .unwrap()
                .session
                .evidence(c)
                .unwrap(),
            c,
        ));
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
    assert!(state.diagnostics().is_empty());
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
    assert!(state.lexical.as_slice().iter().any(|event| matches!(event, Event::Diagnostic(records) if state.before.kernel.session.observed_evidence().tokenizer_observation(records).code == crate::parser::tokenizer::token::DiagnosticCode::UnexpectedNullCharacter)));
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
    let diagnostics = output.driver.state.as_ref().unwrap().diagnostics();
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
    let d = &state.diagnostics()[0];
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
            .diagnostics()
            .iter()
            .filter(|d| d.condition == HeadCondition::UnexpectedHtml)
            .count(),
        3
    );
    assert!(state.lexical.as_slice().iter().any(|event|matches!(event, Event::Diagnostic(records) if state.before.kernel.session.observed_evidence().tokenizer_observation(records).code == crate::parser::tokenizer::token::DiagnosticCode::DuplicateAttribute)));
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
        .diagnostics()
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
    let diagnostics = output.driver.state.as_ref().unwrap().diagnostics();
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
    assert_eq!(state.diagnostics().len(), 2);
    assert!(
        state
            .diagnostics()
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
        assert!(state.diagnostics().is_empty());
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
fn standalone_boundary(driver: &mut StandaloneDocumentDriver<'_>) -> super::head::HeadStep {
    for _ in 0..1_000 {
        let step = driver.step().unwrap();
        if step != super::head::HeadStep::Consumed {
            return step;
        }
    }
    panic!("finite actual source must reach completion or an unsupported boundary");
}

fn before_ordinary_end(driver: &mut StandaloneDocumentDriver<'_>) {
    for _ in 0..1_000 {
        driver.step().unwrap();
        if driver
            .head
            .as_ref()
            .and_then(|h| h.state.as_ref())
            .is_some_and(|s| s.end == Some(super::end::EndPhase::InsertionPoint))
        {
            return;
        }
    }
    panic!("actual source EOF must begin its owning end phase");
}

#[test]
fn standalone_script_start_retains_actual_document_and_initial_metadata_before_preparation() {
    use surgeist_html::HtmlScriptResult;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script src='x'>body</script>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = script_text_node(&mut driver);
    let (c, session, capability) = driver.standalone_parts().unwrap();
    capability.validate_session(c, session).unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(!metadata.flags.force_async());
    assert!(!metadata.flags.already_started());
    assert_eq!(metadata.script_type, None);
    assert!(!metadata.flags.external_file());
    assert!(!metadata.flags.ready_to_be_parser_executed());
    assert!(!metadata.flags.delaying_load());
    assert_eq!(metadata.result, HtmlScriptResult::Uninitialized);
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
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script><meta name=m></head><body>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (c, session, _) = driver.standalone_parts().unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(metadata.flags.already_started());
    assert_eq!(metadata.script_type, Some(HtmlScriptKind::Classic));
    assert!(!metadata.flags.force_async());
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
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script><script type=module>y</script></head>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
        super::head::HeadStep::Complete
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
    assert!(first.flags.already_started() && second.flags.already_started());
    assert_eq!(first.script_type, Some(HtmlScriptKind::Classic));
    assert_eq!(second.script_type, Some(HtmlScriptKind::Module));
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn standalone_script_eof_marks_started_without_preparing_or_changing_initial_facts() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script type=module>x"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let head = driver.head.as_ref().unwrap();
    let state = head.state.as_ref().unwrap();
    let diagnostics = state.diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].condition,
        super::head::HeadCondition::EofInText
    );
    assert_eq!(
        (diagnostics[0].source.start(), diagnostics[0].source.end()),
        (21, 21)
    );
    assert!(state.pending.is_none());
    assert_eq!(state.end, Some(super::end::EndPhase::Done));
    let (c, session, _) = driver.standalone_parts().unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(metadata.flags.already_started());
    assert!(!metadata.flags.force_async());
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
            Some(HtmlScriptKind::Module),
            true,
        ),
        (
            "type=importmap",
            "x",
            true,
            false,
            Some(HtmlScriptKind::ImportMap),
            true,
        ),
        (
            "type=speculationrules",
            "x",
            true,
            false,
            Some(HtmlScriptKind::SpeculationRules),
            true,
        ),
        (
            "type=' text/JAVASCRIPT '",
            "x",
            true,
            false,
            Some(HtmlScriptKind::Classic),
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let node = script_text_node(&mut driver);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (c, session, _) = driver.standalone_parts().unwrap();
        let metadata = session.script_metadata(c, &node).unwrap();
        assert_eq!(
            (
                metadata.flags.already_started(),
                metadata.flags.force_async(),
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let mut owning = Some(driver);
    let (mut c, mut session, _) =
        StandaloneDocumentDriver::take_standalone_fixture(&mut owning).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    assert!(
        frozen
            .script_metadata(&mut c, &node)
            .unwrap()
            .flags
            .already_started()
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    let null_error = state
        .lexical
        .as_slice()
        .iter()
        .find_map(|event| match event {
            Event::Diagnostic(id) => {
                let diagnostic = state
                    .before
                    .kernel
                    .session
                    .observed_evidence()
                    .tokenizer_observation(id);
                (diagnostic.code
                    == crate::parser::tokenizer::token::DiagnosticCode::UnexpectedNullCharacter)
                    .then_some(diagnostic)
            }
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = script_text_node(&mut driver);
    driver.step().unwrap(); // actual script character token
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(
        !session
            .script_metadata(c, &node)
            .unwrap()
            .flags
            .already_started()
    );
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
fn standalone_template_creation_and_eof_preserve_prior_script_metadata() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>x</script><template data-x=q>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let script = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.pending.is_none());
    assert_eq!(state.end, Some(super::end::EndPhase::Done));
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(
        session
            .script_metadata(c, &script)
            .unwrap()
            .flags
            .already_started()
    );
    drop(script);
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document.root().children().next().unwrap();
    let head = document.node(&html).unwrap().children().next().unwrap();
    let template = document.node(&head).unwrap().children().nth(1).unwrap();
    let node = document.node(&template).unwrap();
    let HtmlNodeKind::Element(element) = node.kind() else {
        panic!("template")
    };
    assert_eq!(
        element.name().local_name().code_units(),
        [116, 101, 109, 112, 108, 97, 116, 101]
    );
    assert_eq!(element.attributes().len(), 1);
    assert_eq!(
        element.attributes().as_slice()[0]
            .name()
            .local_name()
            .code_units(),
        [100, 97, 116, 97, 45, 120]
    );
    assert_eq!(
        element.attributes().as_slice()[0].value().code_units(),
        [113]
    );
    assert!(node.children().next().is_none());
    let contents = node.template_contents().unwrap();
    assert!(
        document
            .node(&contents)
            .unwrap()
            .children()
            .next()
            .is_none()
    );
    drop((html, head, template, contents));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn tokenizer_next_input_position_is_nonconsuming_and_uses_real_utf8_crlf_and_utf16_coordinates() {
    for (source, reads) in [
        (HtmlSource::utf8("<script></script>\r\n\u{1f680}"), 3),
        (
            HtmlSource::utf16(DocumentString::from("<script></script>\r\n\u{1f680}")),
            0,
        ),
    ] {
        let mut c = control(100);
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
                .unwrap();
        let mut start = Some(next_kind(&mut tokenizer, &mut c));
        crate::construction::release_token_kind(&mut start, &mut c).unwrap();
        tokenizer
            .set_state(
                crate::parser::tokenizer::states::State::Raw(
                    crate::parser::tokenizer::states::Raw::Script,
                ),
                &mut c,
            )
            .unwrap();
        let mut end = Some(next_kind(&mut tokenizer, &mut c));
        crate::construction::release_token_kind(&mut end, &mut c).unwrap();
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
            12 // No next-item decoding; actual cursor/range metadata only.
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
        crate::construction::release_token_kind(&mut Some(actual), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn standalone_constructor_rejects_real_captured_initialization_before_source_or_tree_effects() {
    let mut builder = crate::HtmlCaptureBuilder::new(
        crate::HtmlCaptureLimits::new(10, 10, 0, 0, 10, 0, 0, 16_777_216),
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
    let mut entry = super::entry::OrdinaryDocumentEntry::new(&request);
    assert!(std::ptr::eq(entry.source(), request.source()));
    assert_eq!(
        entry.failure(),
        Some(super::entry::InitializationFailure::Rejected(
            super::entry::EntryRejection::DocumentInitialization,
        ))
    );
    assert!(entry.at_stop().is_none());
    assert!(entry.retained_usage().is_none());
    assert!(matches!(
        entry.take_driver(),
        Err(StandaloneError::UnsupportedDocumentInitialization)
    ));
    entry.release_failure().unwrap();
}

#[test]
fn standalone_constructor_source_cancellation_and_atomic_layout_failures_are_typed_before_tree_success()
 {
    let cancellation_request =
        crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<script>"), limits(100));
    cancellation_request.cancellation().cancel();
    let mut entry = super::entry::OrdinaryDocumentEntry::new(&cancellation_request);
    assert!(matches!(
        entry.take_driver(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    entry.release_failure().unwrap();
    let cap_request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(""),
        limits(100)
            .into_builder()
            .atomic_layout_bytes(0)
            .finish()
            .unwrap(),
    );
    let mut entry = super::entry::OrdinaryDocumentEntry::new(&cap_request);
    assert!(matches!(
        entry.take_driver(),
        Err(StandaloneError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::AtomicLayoutBytes,
            ..
        }))
    ));
    entry.release_failure().unwrap();
    let source_request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<script>"),
        limits(100).into_builder().source_units(1).finish().unwrap(),
    );
    let mut entry = super::entry::OrdinaryDocumentEntry::new(&source_request);
    assert!(matches!(
        entry.take_driver(),
        Err(StandaloneError::Stop(ParseStop::LimitExceeded {
            resource: HtmlParseResource::SourceUnits,
            ..
        }))
    ));
    entry.release_failure().unwrap();
}

#[test]
fn tokenizer_next_input_position_at_eof_is_checked_empty_and_foreign_owner_is_rejected() {
    let source = HtmlSource::utf8("");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = script_text_node(&mut driver);
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
    let diagnostics = state.diagnostics();
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
        super::head::HeadStep::Complete
    );
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(
        session
            .script_metadata(c, &node)
            .unwrap()
            .flags
            .already_started()
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn tokenizer_next_input_point_has_independent_physical_provider_and_metadata_category_deltas() {
    // State + pending + six actual drafts, outer fixed position record,
    // cursor metadata and existing checked range. No next-item decoding,
    // codepoint/offset inspection or preprocessing occurs at this handoff.
    for source in [
        HtmlSource::utf8(""),
        HtmlSource::utf8("ab"),
        HtmlSource::utf16(DocumentString::from("ab")),
    ] {
        let mut c = control(100);
        c.admit_source(&source).unwrap();
        let tokenizer =
            crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
                .unwrap();
        let before = c.usage().clone();
        let point = tokenizer
            .checked_next_input_position(&source, &mut c)
            .unwrap();
        assert_eq!((point.start(), point.end()), (0, 0));
        for (category, expected) in [
            (WorkCategory::RecordVisit, 12),
            (
                WorkCategory::CopyByte,
                2 * size_of::<crate::HtmlSourceRange>() as u64,
            ),
            (WorkCategory::InputUnitRead, 0),
            (WorkCategory::InspectByte, 0),
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
        let mut tokenizer =
            crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
                .unwrap();
        let first = next_kind(&mut tokenizer, &mut c);
        let TokenKind::Text(value) = &first else {
            panic!("real recovered literal")
        };
        assert_eq!(value.value.text().code_units(), [u16::from(emitted)]);
        crate::construction::release_token_kind(&mut Some(first), &mut c).unwrap();
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
        crate::construction::release_token_kind(&mut Some(second), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn standalone_required_checkpoint_has_source_derived_category_cost_and_one_short_terminal_admission()
 {
    // Two actual proof checks each read eight qualification facts, including
    // the explicit images-disabled construction capability. The queue
    // owner reads checkpoint1, readiness8 (form plus the seven effect fields,
    // including Range and Radio) and empty cursor/length2. Its actual
    // bool set/clear contributes CopyByte2/RecordWrite2, with StateStep1.
    // One-short fails the final clear write, retaining the flag under first-stop.
    // The same queue now retains/restores its actual triggering cause. Its
    // reference adds one ProviderCall/write; restoration adds one visit/write.
    let cause_bytes = std::mem::size_of::<crate::construction::provenance::ConstructionCauseRef>()
        as u64
        + std::mem::size_of::<Option<usize>>() as u64;
    let expected_work = 28 + 1 + (2 + cause_bytes) + 4 + 1;
    for short in [false, true] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<script>x</script>"),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let node = script_text_node(&mut driver);
        let (c, session, proof) = driver.standalone_parts().unwrap();
        assert!(
            !session
                .script_metadata(c, &node)
                .unwrap()
                .flags
                .already_started()
        );
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
            28
        );
        assert_eq!(
            c.usage().work(WorkCategory::ProviderCall) - before.work(WorkCategory::ProviderCall),
            1
        );
        assert_eq!(
            c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
            2 + cause_bytes
        );
        assert_eq!(
            c.usage().work(WorkCategory::StateStep) - before.work(WorkCategory::StateStep),
            1
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
            4 - u64::from(short)
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
            let first = c.stop().unwrap();
            assert_eq!(proof.checkpoint(c, session), Err(SessionError::Stop(first)));
            assert_eq!(session.freeze(c).err(), Some(SessionError::Stop(first)));
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let Event::Token(event) = tokenizer.next(&mut c, false).unwrap().unwrap() else {
        panic!("real EOF token");
    };
    let mut input = Some(event.into_token(&mut c).unwrap());
    assert!(matches!(input.as_ref().unwrap().kind, TokenKind::Eof));
    let before = c.usage().clone();
    dispose_token(&mut input, &mut c).unwrap();
    // Optional Token take + PendingToken with actual family descriptor, then TokenKind
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
        7
    );
    assert!(input.is_none());
    assert!(c.stop().is_none());
    tokenizer.release(&mut c).unwrap();
    assert_released(&c);
}

pub(super) fn body_document(driver: StandaloneDocumentDriver<'_>) -> (ParseControl, FrozenArena) {
    let mut input = Some(driver);
    let (mut c, mut session, _) =
        StandaloneDocumentDriver::take_standalone_fixture(&mut input).unwrap();
    assert!(input.is_none());
    let frozen = session.freeze(&mut c).unwrap();
    (c, frozen)
}
pub(super) fn outline(document: &crate::HtmlDocument) -> Vec<(usize, Vec<u16>)> {
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
pub(super) fn expected_outline(rows: &[(usize, &str)]) -> Vec<(usize, Vec<u16>)> {
    rows.iter()
        .map(|(depth, value)| (*depth, value.encode_utf16().collect()))
        .collect()
}

fn assert_pre_listing_self_closing_rule(
    source: &str,
    start_end: usize,
    br_start: usize,
    br_end: usize,
    rows: &[(usize, &str)],
) {
    use super::head::{HeadCondition, HeadStep, InsertionMode};
    use crate::construction::evidence::{
        DiagnosticContext, DiagnosticKind, DiagnosticLocationRole, DiagnosticSeverity,
        DiagnosticToken,
    };

    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(standalone_boundary(&mut driver), HeadStep::Complete);
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(!state.frameset_ok);
    let evidence = state.before.kernel.session.retained_evidence().unwrap();
    // HTML877d's pre/listing row does not acknowledge the start-token solidus.
    // That error precedes the later end-br recovery in the same InBody phase.
    // Neither the rule identity nor the offsets are derived from the producer.
    assert_eq!(evidence.len(), 2);
    for (index, (condition, branch, start, end)) in [
        (
            HeadCondition::UnacknowledgedSelfClosing,
            "pre-or-listing-start/self-closing",
            21,
            start_end,
        ),
        (
            HeadCondition::UnexpectedEndTag,
            "br-end/reprocess-as-start",
            br_start,
            br_end,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (record, location) = evidence.retained_record(index).unwrap();
        assert_eq!(record.kind, DiagnosticKind::Tree(condition));
        assert_eq!(
            record.context,
            DiagnosticContext::Tree(InsertionMode::InBody)
        );
        assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
        assert_eq!(record.rule.anchor, "parsing-main-inbody");
        assert_eq!(record.rule.branch, branch);
        assert_eq!(location.source.source(), request.source().identity());
        assert_eq!((location.start(), location.end()), (start, end));
        assert_eq!(location.role, DiagnosticLocationRole::TokenEnvelope);
        assert_eq!(location.token, Some(DiagnosticToken::Tag));
    }
    let (mut c, mut frozen) = body_document(driver);
    // The self-closing flag leaves the actual element open; its initial LF is
    // ignored, A remains inside it, and the recovered br precedes a sibling p.
    assert_eq!(outline(frozen.document().unwrap()), expected_outline(rows));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn pre_self_closing_pre_listing_rule_keeps_literal_evidence_and_recovery() {
    assert_pre_listing_self_closing_rule(
        "<!doctype html><body><pre/>\nA</pre></br><p>Z</p>",
        27,
        35,
        40,
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "pre"),
            (3, "#A"),
            (2, "br"),
            (2, "p"),
            (3, "#Z"),
        ],
    );
}

#[test]
fn listing_self_closing_pre_listing_rule_keeps_literal_evidence_and_recovery() {
    assert_pre_listing_self_closing_rule(
        "<!doctype html><body><listing/>\nA</listing></br><p>Z</p>",
        31,
        43,
        48,
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "listing"),
            (3, "#A"),
            (2, "br"),
            (2, "p"),
            (3, "#Z"),
        ],
    );
}

// Literal source-rule observations through the actual driver and native arena.
struct ExpectedTreeRule {
    condition: super::head::HeadCondition,
    mode: super::head::InsertionMode,
    anchor: &'static str,
    branch: &'static str,
    span: (usize, usize),
}
fn assert_caller_rule_evidence(
    source: &str,
    expected: &[ExpectedTreeRule],
    rows: &[(usize, &str)],
) {
    use super::head::HeadStep;
    use crate::construction::evidence::{
        DiagnosticContext, DiagnosticKind, DiagnosticLocationRole, DiagnosticSeverity,
        DiagnosticToken,
    };
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(standalone_boundary(&mut driver), HeadStep::Complete);
    let evidence = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .session
        .retained_evidence()
        .unwrap();
    assert_eq!(evidence.len(), expected.len(), "{source}");
    for (index, expected) in expected.iter().enumerate() {
        let (record, location) = evidence.retained_record(index).unwrap();
        assert_eq!(
            record.kind,
            DiagnosticKind::Tree(expected.condition),
            "{source} record {index}"
        );
        assert_eq!(
            record.context,
            DiagnosticContext::Tree(expected.mode),
            "{source} record {index}"
        );
        assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
        assert_eq!(
            record.rule.anchor, expected.anchor,
            "{source} record {index}"
        );
        assert_eq!(
            record.rule.branch, expected.branch,
            "{source} record {index}"
        );
        assert_eq!(location.source.source(), request.source().identity());
        assert_eq!((location.start(), location.end()), expected.span);
        assert_eq!(location.role, DiagnosticLocationRole::TokenEnvelope);
        assert_eq!(location.token, Some(DiagnosticToken::Tag));
    }
    let (mut c, mut frozen) = body_document(driver);
    assert_eq!(outline(frozen.document().unwrap()), expected_outline(rows));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn in_table_body_tr_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><tbody><tr/></tr></tbody></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InRow,
            anchor: "parsing-main-intbody",
            branch: "table-structure-start/self-closing",
            span: (29, 34),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
        ],
    );
}

#[test]
fn in_row_td_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><tbody><tr><td/></td></tr></tbody></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InCell,
            anchor: "parsing-main-intr",
            branch: "cell-start/self-closing",
            span: (33, 38),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
        ],
    );
}

#[test]
fn in_row_th_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><tbody><tr><th/></th></tr></tbody></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InCell,
            anchor: "parsing-main-intr",
            branch: "cell-start/self-closing",
            span: (33, 38),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "th"),
        ],
    );
}

#[test]
fn in_table_caption_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><caption/></caption></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InCaption,
            anchor: "parsing-main-intable",
            branch: "table-structure-start/self-closing",
            span: (22, 32),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "caption"),
        ],
    );
}

#[test]
fn in_table_colgroup_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><colgroup/></colgroup></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InColumnGroup,
            anchor: "parsing-main-intable",
            branch: "table-structure-start/self-closing",
            span: (22, 33),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "colgroup"),
        ],
    );
}

#[test]
fn in_table_tbody_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><tbody/></tbody></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InTableBody,
            anchor: "parsing-main-intable",
            branch: "table-structure-start/self-closing",
            span: (22, 30),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tbody"),
        ],
    );
}

#[test]
fn in_table_thead_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><thead/></thead></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InTableBody,
            anchor: "parsing-main-intable",
            branch: "table-structure-start/self-closing",
            span: (22, 30),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "thead"),
        ],
    );
}

#[test]
fn in_table_tfoot_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><tfoot/></tfoot></table>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InTableBody,
            anchor: "parsing-main-intable",
            branch: "table-structure-start/self-closing",
            span: (22, 30),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tfoot"),
        ],
    );
}

#[test]
fn after_head_frameset_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head></head><frameset/></frameset>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InFrameset,
            anchor: "the-after-head-insertion-mode",
            branch: "frameset-start/self-closing",
            span: (28, 39),
        }],
        &[(0, "html"), (1, "head"), (1, "frameset")],
    );
}

#[test]
fn in_body_frameset_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><input type=hidden><frameset/></frameset>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedBodyToken,
                mode: InsertionMode::InBody,
                anchor: "parsing-main-inbody",
                branch: "frameset-start/in-body",
                span: (34, 45),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::InFrameset,
                anchor: "parsing-main-inbody",
                branch: "frameset-start/self-closing",
                span: (34, 45),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "frameset")],
    );
}

#[test]
fn in_frameset_nested_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head></head><frameset><frameset/></frameset></frameset>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnacknowledgedSelfClosing,
            mode: InsertionMode::InFrameset,
            anchor: "parsing-main-inframeset",
            branch: "frameset-start/self-closing",
            span: (38, 49),
        }],
        &[(0, "html"), (1, "head"), (1, "frameset"), (2, "frameset")],
    );
}

#[test]
fn before_head_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><html><html/><head></head><body>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::BeforeHead,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (21, 28),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::BeforeHead,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (21, 28),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "body")],
    );
}

#[test]
fn in_head_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head><html/></head><body>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::InHead,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (21, 28),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::InHead,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (21, 28),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "body")],
    );
}

#[test]
fn after_head_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head></head><html/><body>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::AfterHead,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (28, 35),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::AfterHead,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (28, 35),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "body")],
    );
}

#[test]
fn in_body_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><body><html/></body>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::InBody,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (21, 28),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::InBody,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (21, 28),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "body")],
    );
}

#[test]
fn after_body_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><body></body><html/>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::AfterBody,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (28, 35),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::AfterBody,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (28, 35),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "body")],
    );
}

#[test]
fn after_after_body_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><body></body></html><html/>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::AfterAfterBody,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (35, 42),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::AfterAfterBody,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (35, 42),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "body")],
    );
}

#[test]
fn in_frameset_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head></head><frameset><html/></frameset>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::InFrameset,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (38, 45),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::InFrameset,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (38, 45),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "frameset")],
    );
}

#[test]
fn after_frameset_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head></head><frameset></frameset><html/>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::AfterFrameset,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (49, 56),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::AfterFrameset,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (49, 56),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "frameset")],
    );
}

#[test]
fn after_after_frameset_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head></head><frameset></frameset></html><html/>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::AfterAfterFrameset,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (56, 63),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::AfterAfterFrameset,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (56, 63),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "frameset")],
    );
}

#[test]
fn in_head_noscript_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><head><noscript><html/></noscript></head><body>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::InHeadNoscript,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (31, 38),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::InHeadNoscript,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (31, 38),
            },
        ],
        &[(0, "html"), (1, "head"), (2, "noscript"), (1, "body")],
    );
}

#[test]
fn in_table_html_self_closing_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><table><html/></table>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedTableToken,
                mode: InsertionMode::InTable,
                anchor: "parsing-main-intable",
                branch: "anything-else/foster",
                span: (22, 29),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedHtml,
                mode: InsertionMode::InTable,
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
                span: (22, 29),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnacknowledgedSelfClosing,
                mode: InsertionMode::InTable,
                anchor: "parsing-main-inbody",
                branch: "html-start/self-closing",
                span: (22, 29),
            },
        ],
        &[(0, "html"), (1, "head"), (1, "body"), (2, "table")],
    );
}

#[test]
fn foreign_end_br_breakout_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><svg><g></br>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedForeignToken,
                mode: InsertionMode::InBody,
                anchor: "parsing-main-inforeign",
                branch: "foreign-end/html-breakout",
                span: (23, 28),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedEndTag,
                mode: InsertionMode::InBody,
                anchor: "parsing-main-inbody",
                branch: "br-end/reprocess-as-start",
                span: (23, 28),
            },
        ],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "svg"),
            (3, "g"),
            (2, "br"),
        ],
    );
}

#[test]
fn foreign_end_p_breakout_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><svg><g></p>",
        &[
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedForeignToken,
                mode: InsertionMode::InBody,
                anchor: "parsing-main-inforeign",
                branch: "foreign-end/html-breakout",
                span: (23, 27),
            },
            ExpectedTreeRule {
                condition: HeadCondition::UnexpectedEndTag,
                mode: InsertionMode::InBody,
                anchor: "parsing-main-inbody",
                branch: "p-end/p-not-in-button-scope",
                span: (23, 27),
            },
        ],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "svg"),
            (3, "g"),
            (2, "p"),
        ],
    );
}

#[test]
fn foreign_start_breakout_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><svg><g><br/>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnexpectedForeignToken,
            mode: InsertionMode::InBody,
            anchor: "parsing-main-inforeign",
            branch: "foreign-start/html-breakout",
            span: (23, 28),
        }],
        &[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "svg"),
            (3, "g"),
            (2, "br"),
        ],
    );
}

#[test]
fn explicit_body_frameset_guard_retains_caller_rule() {
    use super::head::{HeadCondition, InsertionMode};
    assert_caller_rule_evidence(
        "<!doctype html><body><frameset/>",
        &[ExpectedTreeRule {
            condition: HeadCondition::UnexpectedBodyToken,
            mode: InsertionMode::InBody,
            anchor: "parsing-main-inbody",
            branch: "frameset-start/in-body",
            span: (21, 32),
        }],
        &[(0, "html"), (1, "head"), (1, "body")],
    );
}

#[test]
fn ordinary_body_recovers_paragraphs_blocks_and_missing_end_p_with_actual_topology() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<body><p>a<div>b</div>c</p><p>d<p>e</body></html>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::AfterAfterBody);
    assert!(!state.frameset_ok);
    assert!(state.pending.is_none());
    assert_eq!(state.end, Some(super::end::EndPhase::Done));
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
            assert!(metadata.flags.already_started());
            assert_eq!(metadata.script_type, Some(HtmlScriptKind::Classic));
        }
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
        .diagnostics()
        .into_iter()
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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

// HTML877d Color creation consumes the original tag after allocating the
// native input. Reached unavailable CanvasText does not certify completion or
// association, and it never becomes the standard-invalid black fallback.
fn assert_color_palette_refusal(
    literal: &str,
    mode: super::head::InsertionMode,
    stack: usize,
    nodes: u64,
    form_pointer: bool,
    foster: bool,
) {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    for _ in 0..1_000 {
        match driver.step() {
            Ok(super::head::HeadStep::Consumed) => {}
            Err(_) => break,
            Ok(other) => panic!("expected reached Color fact refusal, got {other:?}"),
        }
    }
    let first = ParseStop::ColorProcessing(crate::HtmlColorProcessingError::Unavailable(
        surgeist_css::color::CssColorUnavailable::Palette(
            surgeist_css::color::CssSystemColor::CanvasText,
        ),
    ));
    assert_eq!(driver.control.stop(), Some(first));
    assert_eq!(
        driver.control.stopped_source_binding().unwrap().unwrap().0,
        *request.source().identity()
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, mode);
    assert_eq!(state.current_form.is_some(), form_pointer);
    assert_eq!(state.before.kernel.stack().unwrap().len(), stack);
    assert_eq!(state.before.kernel.foster_parenting, foster);
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        Payload::Tag(None)
    ));
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        nodes
    );
    if foster {
        let diagnostics = state.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].condition,
            super::head::HeadCondition::UnexpectedTableToken
        );
        assert_eq!(diagnostics[0].mode, super::head::InsertionMode::InTable);
    }
    let work = driver
        .control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    request.cancellation().cancel();
    assert!(driver.step().is_err());
    assert_eq!(driver.control.stop(), Some(first));
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current,
        work
    );
    let mut retained = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut retained).is_err());
    let driver = retained
        .as_mut()
        .expect("failed extraction retains the real owner");
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.stop(), Some(first));
}

#[test]
fn body_color_fact_refusal_preserves_prefix_and_consumed_creation() {
    // Document/html/head/body/p/Text(x)/input: seven actual records.
    assert_color_palette_refusal(
        "<body><p>x<input data-x=kept type=color value=currentcolor>tail",
        super::head::InsertionMode::InBody,
        3,
        7,
        false,
        false,
    );
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(
            state
                .diagnostics()
                .iter()
                .filter(|d| d.condition == super::head::HeadCondition::UnexpectedOpenElement)
                .count(),
            errors
        );
        assert!(state.pending.is_none());
        assert_eq!(state.end, Some(super::end::EndPhase::Done));
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
fn body_frameset_entry_uses_actual_flag_and_replaces_the_implied_body() {
    for source in [
        "<head></head><frameset a=kept>",
        "<head></head><widget><frameset a=kept>",
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.mode, super::head::InsertionMode::InFrameset);
        assert!(state.frameset_ok);
        assert_eq!(state.before.kernel.stack().unwrap().len(), 0);
        let (mut c, mut frozen) = body_document(driver);
        let actual = outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(
            actual,
            expected_outline(&[(0, "html"), (1, "head"), (1, "frameset")])
        );
    }
}

#[test]
fn late_script_eof_returns_afterhead_then_creates_body_without_preparation() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<head></head><script type=module>x"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = script_text_node(&mut driver);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::InBody);
    assert!(state.temporary_head.is_none());
    assert!(state.frameset_ok);
    let (c, session, _) = driver.standalone_parts().unwrap();
    let metadata = session.script_metadata(c, &node).unwrap();
    assert!(metadata.flags.already_started());
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
        .diagnostics()
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
        state.diagnostics().len(),
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let boundary = standalone_boundary(&mut driver);
        if boundary != super::head::HeadStep::Complete {
            // Current real driver suspends at the formatting opener. Dispose
            // all actual owners before the expected runtime RED assertion.
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(boundary, super::head::HeadStep::Complete);
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let boundary = standalone_boundary(&mut driver);
        if boundary != super::head::HeadStep::Complete {
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(boundary, super::head::HeadStep::Complete);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut tokenizer =
        crate::parser::tokenizer::tests::LexerFixture::new(&source, State::Data, None, &mut c)
            .unwrap();
    let (mut kernel, _) = begin(&mut tokenizer, RootKind::Fragment, &mut c);
    let old = add(&mut kernel, &mut tokenizer, StackPush::Push, &mut c);
    let replacement = add(&mut kernel, &mut tokenizer, StackPush::NoPush, &mut c);
    let before = c.usage().clone();
    kernel.replace_stack(1, &replacement, &mut c, None).unwrap();
    // Kernel readiness1, target index10, at1, pre-removal qualification15,
    // get_mut1, post-removal qualification15, removed index10, option5 and
    // textarea5. Each of the four indices includes the form field plus seven
    // actual effect-readiness fields, including Range and Radio, and its shared
    // handle identity/range qualification reads one additional header. Four
    // HTML namespace comparisons inspect28 units at three credits each; local
    // p length differs from option, textarea and style, so no local units read.
    // Actual Style-pop qualification adds html_local's3 record reads plus the
    // two comparison headers =5 visits, and one28-unit namespace comparison
    // at3 bytes per pair. It mutates no Style row for this p element.
    // The replacement is also asked whether it is a template, so the kernel
    // can keep the last template exact: its name read (index10 plus the
    // record1) and html_named's namespace comparison (two plus the header =3,
    // inspecting28 units) and local comparison (one, one choice and the
    // header =3; the length differs from template, so no local units are
    // read) =17 visits, and one more asking whether the replaced entry is
    // listed as a template.
    assert_eq!(
        c.usage().work(WorkCategory::RecordVisit) - before.work(WorkCategory::RecordVisit),
        63 + 3 + 2 + 17 + 1
    );
    assert_eq!(
        c.usage().work(WorkCategory::RecordWrite) - before.work(WorkCategory::RecordWrite),
        1
    );
    assert_eq!(
        c.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
        3 * size_of::<ParserNode>() as u64
    );
    assert_eq!(
        c.usage().work(WorkCategory::ProviderCall) - before.work(WorkCategory::ProviderCall),
        2
    );
    assert_eq!(
        c.usage().work(WorkCategory::InspectByte) - before.work(WorkCategory::InspectByte),
        (4 + 1 + 1) * 28 * 3
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // The published driver retains the actual opener at Form. Release the
        // real tree/token/controller owners before the runtime RED assertion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    crate::construction::FormAssociationEvidence,
)> {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
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
    use crate::construction::FormAssociationEvidence::{ParserAssociated, Reset};
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
    use crate::construction::FormAssociationEvidence::{ParserAssociated, Reset};
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
    use crate::construction::FormAssociationEvidence::ParserAssociated;
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
fn ordinary_form_color_fact_refusal_retains_actual_form_pointer() {
    // Document/doctype/html/head/body/form/input: seven actual records.
    assert_color_palette_refusal(
        "<!doctype html><form id=f><input id=kept type=color value=currentcolor>",
        super::head::InsertionMode::InBody,
        3,
        7,
        true,
        false,
    );
}

// Governing HTML877d syntax4630–4638,5110–5114,5322–5498:
// omitted tbody and td end tags recover to two cells in one real row.

#[test]
fn ordinary_table_implied_tbody_and_cell_end_recovery_preserve_real_sibling_order() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table><tr><td>1<td>2</table>3"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // The existing driver retains the actual table opener at Table. Dispose
        // its real tree/token/controller owners before the runtime assertion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let mut returns = Vec::new();
    for _ in 0..1000 {
        let step = driver.step().unwrap();
        if let Some(head) = &driver.head {
            let state = head.state.as_ref().unwrap();
            if state.mode == super::head::InsertionMode::Text {
                returns.push(state.return_mode);
            }
        }
        if matches!(step, super::head::HeadStep::Complete) {
            break;
        }
    }
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.boundary, None);
    assert_eq!(state.end, Some(super::end::EndPhase::Done));
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
fn structural_table_color_fact_refusal_preserves_reached_foster_effect() {
    // Document/doctype/html/head/body/table/input: seven actual records.
    // The Color refusal follows the InTable diagnostic and foster=true write.
    assert_color_palette_refusal(
        "<!doctype html><table><input type=color value=currentcolor>",
        super::head::InsertionMode::InTable,
        3,
        7,
        false,
        true,
    );
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let null_ranges: Vec<_> = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .diagnostics()
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
        let mut entry = super::entry::OrdinaryDocumentEntry::new(&request);
        let mut driver = match entry.take_driver() {
            Ok(driver) => driver,
            Err(_) => {
                entry.release_failure().unwrap();
                continue;
            }
        };
        for _ in 0..1000 {
            match driver.step() {
                Ok(super::head::HeadStep::Boundary(_) | super::head::HeadStep::Complete) => break,
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let table_errors = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .diagnostics()
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

#[test]
fn ordinary_listed_controls_preserve_nested_button_recovery_and_object_marker() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><fieldset><button id=b>A<button>C</fieldset><object name=o><b>D</object></form>E",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // Dispose actual tree, lexical and controller owners before checking completion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
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
    let form = document.node(&body).unwrap().children().next().unwrap();
    let fieldset = document.node(&form).unwrap().children().next().unwrap();
    let first_button = document.node(&fieldset).unwrap().children().next().unwrap();
    let second_button = document.node(&fieldset).unwrap().children().nth(1).unwrap();
    let object = document.node(&form).unwrap().children().nth(1).unwrap();
    let attributes: Vec<_> = [&form, &fieldset, &first_button, &second_button, &object]
        .into_iter()
        .map(|id| {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("source-derived control positions must be actual elements");
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

    // Keep only plain copied observer units and result-qualified identities;
    // no clones of controlled string/attribute backings outlive disposal.
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_ne!(first_button, second_button);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "fieldset"),
            (4, "button"),
            (5, "#A"),
            (4, "button"),
            (5, "#C"),
            (3, "object"),
            (4, "b"),
            (5, "#D"),
            (2, "#E"),
        ])
    );
    assert_eq!(
        attributes,
        vec![
            vec![(
                None::<Vec<u16>>,
                vec![b'i' as u16, b'd' as u16],
                vec![b'f' as u16],
            )],
            vec![],
            vec![(
                None::<Vec<u16>>,
                vec![b'i' as u16, b'd' as u16],
                vec![b'b' as u16],
            )],
            vec![],
            vec![(
                None::<Vec<u16>>,
                vec![b'n' as u16, b'a' as u16, b'm' as u16, b'e' as u16],
                vec![b'o' as u16],
            )],
        ]
    );
}

#[test]
fn listed_controls_fieldset_closes_p_and_button_scope_preserves_outer_ancestors() {
    assert_eq!(
        formatting_outline(
            "<!doctype html><p>A<fieldset>B</fieldset>C<button><span>D<button>E</button>F"
        ),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "#A"),
            (2, "fieldset"),
            (3, "#B"),
            (2, "#C"),
            (2, "button"),
            (3, "span"),
            (4, "#D"),
            (2, "button"),
            (3, "#E"),
            (2, "#F")
        ])
    );
}

#[test]
fn listed_controls_nested_button_diagnostic_cap_retains_old_stack_and_original_tag() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><button><p>A<button id=kept>"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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
    assert_eq!(state.before.kernel.stack().unwrap().len(), 4);
    let Payload::Tag(Some(tag)) = &state.pending.as_ref().unwrap().payload else {
        panic!("original nested button token");
    };
    assert_eq!(
        tag.name.value.text().code_units(),
        "button".encode_utf16().collect::<Vec<_>>()
    );
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
fn listed_controls_table_cell_delegation_and_marker_clear_keep_real_modes() {
    assert_eq!(
        formatting_outline(
            "<!doctype html><table><tr><td><fieldset><button>A</button></fieldset><object><b>B</object>C</table>D"
        ),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "fieldset"),
            (7, "button"),
            (8, "#A"),
            (6, "object"),
            (7, "b"),
            (8, "#B"),
            (6, "#C"),
            (2, "#D")
        ])
    );
}

#[test]
fn listed_controls_cancellation_releases_canonical_node_and_object_marker_owners() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><object><button>A"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let mut opened = false;
    for _ in 0..100 {
        driver.step().unwrap();
        if driver
            .head
            .as_ref()
            .and_then(|head| head.state.as_ref())
            .is_some_and(|state| state.before.kernel.stack().unwrap().len() == 4)
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

#[test]
fn listed_controls_exact_utf16_text_preserves_pairs_and_isolates() {
    let mut units: Vec<_> = "<!doctype html><button>".encode_utf16().collect();
    units.extend([0xd83d, 0xde00, 0xd800, 13, 10, 0xdc00]);
    units.extend("</button>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let actual = {
        let document = frozen.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let button = document.node(&body).unwrap().children().next().unwrap();
        let text = document.node(&button).unwrap().children().next().unwrap();
        let HtmlNodeKind::Text(text) = document.node(&text).unwrap().kind() else {
            panic!("exact text");
        };
        text.code_units().to_vec()
    };
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual, [0xd83d, 0xde00, 0xd800, 10, 0xdc00]);
}

#[test]
fn listed_controls_open_slot_exact_and_one_short_retains_actual_owners() {
    for name in ["button", "fieldset", "object"] {
        let source = format!("<!doctype html><{name}></{name}>");
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source.as_str()), limits(2));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let mut failed = false;
        for _ in 0..100 {
            if driver.step().is_err() {
                failed = true;
                break;
            }
        }
        assert!(failed, "{name}");
        let first = driver.control.stop();
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
        assert!(matches!(
            &state.pending.as_ref().unwrap().payload,
            Payload::Tag(None)
        ));
        assert!(driver.step().is_err());
        assert_eq!(driver.control.stop(), first);
        driver.release().unwrap();
        assert_released(&driver.control);

        let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(3));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn listed_controls_table_foster_delegation_restores_structural_mode_before_rows() {
    assert_eq!(
        formatting_outline(
            "<!doctype html><table><button>A</button><object>B</object><fieldset>C</fieldset><tr><td>D</table>E"
        ),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "button"),
            (3, "#A"),
            (2, "object"),
            (3, "#B"),
            (2, "fieldset"),
            (3, "#C"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#D"),
            (2, "#E")
        ])
    );
}

fn listed_control_owner_observations(source: &str) -> Vec<(Vec<u16>, Option<Vec<u16>>)> {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return Vec::new();
    }
    let (mut c, mut frozen) = body_document(driver);
    let observations = {
        let document = frozen.document().unwrap();
        let mut stack: Vec<_> = document.root().children().collect();
        stack.reverse();
        let mut observations = Vec::new();
        while let Some(id) = stack.pop() {
            let node = document.node(&id).unwrap();
            if let HtmlNodeKind::Element(element) = node.kind() {
                let name = element.name().local_name().code_units();
                if ["button", "fieldset", "object"]
                    .iter()
                    .any(|candidate| name.iter().copied().eq(candidate.encode_utf16()))
                {
                    let state = frozen.form_association_state(&mut c, &id).unwrap();
                    use crate::HtmlNativeFormAssociationKind;
                    let expected = match name {
                        [98, 117, 116, 116, 111, 110] => HtmlNativeFormAssociationKind::Button,
                        [102, 105, 101, 108, 100, 115, 101, 116] => {
                            HtmlNativeFormAssociationKind::Fieldset
                        }
                        [111, 98, 106, 101, 99, 116] => HtmlNativeFormAssociationKind::Object,
                        _ => unreachable!(),
                    };
                    assert_eq!(state.kind, expected);
                    let owner = frozen
                        .form_association_owner(&mut c, &id)
                        .unwrap()
                        .map(|owner| {
                            let HtmlNodeKind::Element(element) =
                                document.node(&owner).unwrap().kind()
                            else {
                                panic!("qualified actual form owner");
                            };
                            assert_eq!(
                                element.name().local_name().code_units(),
                                [102, 111, 114, 109]
                            );
                            element
                                .attributes()
                                .iter()
                                .find(|attribute| {
                                    attribute.name().local_name().code_units() == [105, 100]
                                })
                                .unwrap()
                                .value()
                                .code_units()
                                .to_vec()
                        });
                    observations.push((name.to_vec(), owner));
                }
            }
            let mut children: Vec<_> = node.children().collect();
            children.reverse();
            stack.extend(children);
        }
        observations
    };
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    observations
}

#[test]
fn listed_controls_real_driver_resolves_forward_form_and_first_nonform_id() {
    for (source, expected) in [
        (
            "<!doctype html><button form=f>A</button><fieldset form=f>B</fieldset><object form=f>C</object><form id=f></form>",
            Some("f"),
        ),
        (
            "<!doctype html><button form=f>A</button><fieldset form=f>B</fieldset><object form=f>C</object><span id=f></span><form id=f></form>",
            None,
        ),
        (
            "<!doctype html><form id=f><button form=''>A</button><fieldset form=''>B</fieldset><object form=''>C</object></form>",
            None,
        ),
        (
            "<!doctype html><form id=f><button>A</button><fieldset>B</fieldset><object>C</object></form>",
            Some("f"),
        ),
        (
            "<!doctype html><button form=F>A</button><fieldset form=F>B</fieldset><object form=F>C</object><form id=f></form>",
            None,
        ),
    ] {
        assert_eq!(
            listed_control_owner_observations(source),
            ["button", "fieldset", "object"]
                .into_iter()
                .map(|name| (
                    name.encode_utf16().collect(),
                    expected.map(|id| id.encode_utf16().collect())
                ))
                .collect::<Vec<_>>(),
            "{source}"
        );
    }
}

#[test]
fn listed_controls_late_body_or_html_id_merge_reconsiders_real_forward_owners() {
    for target in ["body", "html"] {
        let source = format!(
            "<!doctype html><button form=f>A</button><fieldset form=f>B</fieldset><object form=f>C</object><form id=f></form><{target} id=f>"
        );
        assert_eq!(
            listed_control_owner_observations(&source),
            ["button", "fieldset", "object"]
                .into_iter()
                .map(|name| (name.encode_utf16().collect(), None))
                .collect::<Vec<_>>(),
            "{target}"
        );
    }
}

#[test]
fn listed_controls_object_scope_blocks_outer_button_recovery() {
    assert_eq!(
        formatting_outline("<!doctype html><button>A<object><button>B</button></object>C</button>"),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "button"),
            (3, "#A"),
            (3, "object"),
            (4, "button"),
            (5, "#B"),
            (3, "#C")
        ])
    );
}

#[test]
fn listed_controls_frameset_flag_follows_each_actual_start_row() {
    for (name, expected) in [("button", false), ("fieldset", true), ("object", false)] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!("<!doctype html><{name}>")),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let mut opened = false;
        for _ in 0..100 {
            driver.step().unwrap();
            if let Some(state) = driver.head.as_ref().and_then(|head| head.state.as_ref())
                && state.before.kernel.stack().unwrap().len() == 3
            {
                assert_eq!(state.frameset_ok, expected, "{name}");
                opened = true;
                break;
            }
        }
        assert!(opened, "{name}");
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}

#[test]
fn ordinary_frameset_nested_frames_and_noframes_preserve_trailing_nodes() {
    // Governing HTML877d syntax4218–4222,5618–5776: frame insertion pops
    // immediately; noframes uses RAWTEXT; trailing comments/PI belong to Document.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><frameset cols='*'><frame name='a'/><frameset><frame name='b'></frameset><noframes>&amp;<b>x</noframes></frameset></html><!--tail--><?x p?>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // Dispose the actual retained opener, tree, tokenizer and controller
        // owners before asserting the existing callable's missing behavior.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_outline = outline(document);
    let actual_mode = document.mode();
    // The real doctype is a preceding Document child, not the html element.
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let frameset = document.node(&html).unwrap().children().nth(1).unwrap();
    let children: Vec<_> = document.node(&frameset).unwrap().children().collect();
    let nested_frame = document
        .node(&children[1])
        .unwrap()
        .children()
        .next()
        .unwrap();
    let element_facts: Vec<_> = [
        &frameset,
        &children[0],
        &children[1],
        &nested_frame,
        &children[2],
    ]
    .into_iter()
    .map(|id| {
        let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
            panic!("source-derived frameset positions must be actual elements");
        };
        (
            element.name().local_name().code_units().to_vec(),
            element
                .attributes()
                .iter()
                .map(|attribute| {
                    (
                        attribute.name().local_name().code_units().to_vec(),
                        attribute.value().code_units().to_vec(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    })
    .collect();

    // Observations retain only plain units and qualified weak identities.
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "frameset"),
            (2, "frame"),
            (2, "frameset"),
            (3, "frame"),
            (2, "noframes"),
            (3, "#&amp;<b>x"),
            (0, "!tail"),
            (0, "?x"),
        ])
    );
    assert_ne!(frameset, children[1]);
    assert_ne!(children[0], nested_frame);
    assert_eq!(
        element_facts,
        [
            ("frameset", vec![("cols", "*")]),
            ("frame", vec![("name", "a")]),
            ("frameset", vec![]),
            ("frame", vec![("name", "b")]),
            ("noframes", vec![]),
        ]
        .into_iter()
        .map(|(name, attributes)| (
            name.encode_utf16().collect::<Vec<_>>(),
            attributes
                .into_iter()
                .map(|(name, value)| (
                    name.encode_utf16().collect::<Vec<_>>(),
                    value.encode_utf16().collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>()
        ))
        .collect::<Vec<_>>()
    );
}

#[test]
fn frameset_body_removal_preserves_form_facts_and_reconstructs_surviving_formatting() {
    use super::head::InsertionMode;
    use crate::construction::FormAssociationEvidence;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form><fieldset><b><frameset></frameset></html> "),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let (old_body, old_form, fieldset, old_b) = loop {
        driver.step().unwrap();
        if let Some(state) = driver.head.as_mut().and_then(|head| head.state.as_mut())
            && state.mode == InsertionMode::InBody
            && state.before.kernel.stack().unwrap().len() == 5
        {
            break (
                state.before.kernel.at(1, &mut driver.control).unwrap(),
                state.before.kernel.at(2, &mut driver.control).unwrap(),
                state.before.kernel.at(3, &mut driver.control).unwrap(),
                state.before.kernel.at(4, &mut driver.control).unwrap(),
            );
        }
    };
    before_ordinary_end(&mut driver);
    let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
    assert_eq!(state.mode, InsertionMode::AfterAfterFrameset);
    assert_eq!(state.formatting.len(), 1);
    assert!(
        state
            .before
            .kernel
            .session
            .same_node(
                &mut driver.control,
                state.current_form.as_ref().unwrap().owned().unwrap(),
                &old_form
            )
            .unwrap()
    );
    assert!(
        state
            .before
            .kernel
            .session
            .parent(&mut driver.control, &old_body)
            .unwrap()
            .is_none()
    );
    let facts = state
        .before
        .kernel
        .session
        .form_association_state(&mut driver.control, &fieldset)
        .unwrap();
    assert!(facts.parser_inserted);
    assert_eq!(facts.evidence, FormAssociationEvidence::ParserAssociated);
    let owner = state
        .before
        .kernel
        .session
        .form_association_owner(&mut driver.control, &fieldset)
        .unwrap()
        .unwrap();
    assert!(
        state
            .before
            .kernel
            .session
            .same_node(&mut driver.control, &owner, &old_form)
            .unwrap()
    );
    let new_b = state.before.kernel.current(&mut driver.control).unwrap();
    assert!(
        !state
            .before
            .kernel
            .session
            .same_node(&mut driver.control, &new_b, &old_b)
            .unwrap()
    );
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
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
            (1, "frameset"),
            (1, "b"),
            (2, "# ")
        ])
    );
}

#[test]
fn frameset_mixed_text_keeps_every_whitespace_run_and_exact_omitted_origins() {
    use super::head::{HeadCondition, InsertionMode};
    let literal = "<!doctype html><frameset>A \0\tB\r\nC</frameset>D \u{c}E</html>F \tG";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    let errors: Vec<_> = state
        .diagnostics()
        .iter()
        .filter(|d| {
            matches!(
                d.condition,
                HeadCondition::NullInFrameset | HeadCondition::UnexpectedFramesetToken
            )
        })
        .map(|d| (d.mode, d.condition, d.source.start(), d.source.end()))
        .collect();
    let mut expected = Vec::new();
    for (index, byte) in literal.bytes().enumerate() {
        if matches!(byte, b'A'..=b'G' | 0) {
            expected.push((
                match byte {
                    b'A'..=b'C' | 0 => InsertionMode::InFrameset,
                    b'D'..=b'E' => InsertionMode::AfterFrameset,
                    _ => InsertionMode::AfterAfterFrameset,
                },
                if byte == 0 {
                    HeadCondition::NullInFrameset
                } else {
                    HeadCondition::UnexpectedFramesetToken
                },
                index,
                index + 1,
            ));
        }
    }
    assert_eq!(errors, expected);
    let omitted: Vec<_> = state
        .before
        .whitespace
        .as_slice()
        .iter()
        .flat_map(|t| t.value.text().code_units())
        .copied()
        .collect();
    assert_eq!(omitted, [65, 0, 66, 67, 68, 69, 70, 71]);
    let (mut c, mut frozen) = body_document(driver);
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(
        actual,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "frameset"),
            (2, "# \t\n"),
            (1, "# \u{c} \t")
        ])
    );
}

#[test]
fn frameset_noframes_restores_each_actual_return_mode_on_end_and_eof() {
    use super::head::{HeadCondition, InsertionMode};
    for (prefix, mode, depth) in [
        ("<!doctype html><frameset>", InsertionMode::InFrameset, 2),
        (
            "<!doctype html><frameset></frameset>",
            InsertionMode::AfterFrameset,
            1,
        ),
        (
            "<!doctype html><frameset></frameset></html>",
            InsertionMode::AfterAfterFrameset,
            1,
        ),
    ] {
        for ending in ["</noframes>", ""] {
            let literal = format!("{prefix}<noframes>&amp;<b>x{ending}");
            let request = crate::HtmlDocumentParseRequest::offline(
                HtmlSource::utf8(literal.as_str()),
                limits(100),
            );
            let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
            let mut entered = false;
            for _ in 0..100 {
                driver.step().unwrap();
                if let Some(state) = driver.head.as_ref().and_then(|h| h.state.as_ref())
                    && state.mode == InsertionMode::Text
                {
                    assert_eq!(state.return_mode, Some(mode));
                    entered = true;
                    break;
                }
            }
            assert!(entered);
            assert_eq!(
                standalone_boundary(&mut driver),
                super::head::HeadStep::Complete
            );
            let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
            assert_eq!(state.mode, mode);
            assert_eq!(state.return_mode, None);
            assert_eq!(
                state
                    .diagnostics()
                    .iter()
                    .filter(|d| d.condition == HeadCondition::EofInText)
                    .count(),
                usize::from(ending.is_empty())
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
                    (1, "frameset"),
                    (depth, "noframes"),
                    (depth + 1, "#&amp;<b>x")
                ])
            );
        }
    }
}

#[test]
fn frameset_rawtext_preserves_utf16_isolates_pairs_and_recovers_null() {
    let mut units: Vec<_> = "<!doctype html><frameset><noframes>"
        .encode_utf16()
        .collect();
    units.extend_from_slice(&[0xd800, 0xd83d, 0xde00, 0, 13, 10]);
    units.extend("</noframes></frameset>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let mut expected =
        expected_outline(&[(0, "html"), (1, "head"), (1, "frameset"), (2, "noframes")]);
    expected.push((3, vec![35, 0xd800, 0xd83d, 0xde00, 0xfffd, 10]));
    let actual = outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual, expected);
}

#[test]
fn frameset_mode_local_ignored_tokens_do_not_enter_body_family_continuations() {
    let source = "<!doctype html><frameset><html id=f><input><template></template><svg><select>x</select><form><button>y</button><widget></widget><frame/></frameset><frame><form></html><svg><input><?tail p?>";
    assert_eq!(
        formatting_outline(source),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "frameset"),
            (2, "frame"),
            (0, "?tail")
        ])
    );
}

#[test]
fn frameset_comment_and_pi_placement_tracks_current_node_then_document() {
    let source = "<!doctype html><frameset><!--in--><?i a?><frame></frameset><!--after--><?a b?></html><!--doc--><?d c?>";
    assert_eq!(
        formatting_outline(source),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "frameset"),
            (2, "!in"),
            (2, "?i"),
            (2, "frame"),
            (1, "!after"),
            (1, "?a"),
            (0, "!doc"),
            (0, "?d")
        ])
    );
}

#[test]
fn frameset_selfclosing_acknowledgement_and_open_eof_follow_actual_stack() {
    use super::head::HeadCondition;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><frameset/><frame/><frameset></frameset>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.before.kernel.stack().unwrap().len(), 0);
    let conditions: Vec<_> = state.diagnostics().iter().map(|d| d.condition).collect();
    assert_eq!(
        conditions,
        [
            HeadCondition::UnacknowledgedSelfClosing,
            HeadCondition::UnexpectedOpenElement
        ]
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn frameset_body_diagnostic_cap_stops_before_removal_or_stack_changes() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form><frameset>"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let error = (0..100)
        .find_map(|_| driver.step().err())
        .expect("frameset source error consumes diagnostic slot");
    let stop = driver.control.stop().unwrap();
    assert!(
        matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            }
        ),
        "{error:?}"
    );
    let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::InBody);
    assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
    assert!(state.current_form.is_some());
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        Payload::Tag(Some(_))
    ));
    // Document->doctype/html, html->head/body, body->form: removal has not
    // committed the source's next operation after the rejected diagnostic.
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .current,
        5
    );
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), Some(stop));
    assert_released(&driver.control);
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form><frameset></frameset>"),
        limits(100).into_builder().diagnostics(1).finish().unwrap(),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    assert_eq!(
        driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .diagnostics()
            .len(),
        1
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn frameset_afterafter_whitespace_reconstruction_stop_retains_exact_original_ranges() {
    let literal = "<!doctype html><b><frameset></frameset></html>\r\n ";
    // Eight permanent nodes: Document/doctype/html/head/body/original b,
    // frameset and reconstructed b. The ninth would be its new text node.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(literal),
        limits(100)
            .into_builder()
            .managed_nodes(8)
            .finish()
            .unwrap(),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    // Reach the real return mode, then retain the next actual event from the
    // SAME tokenizer/control in the ordinary driver's pending owning slot.
    let mut reached = false;
    for _ in 0..100 {
        driver.step().unwrap();
        if driver
            .head
            .as_ref()
            .and_then(|head| head.state.as_ref())
            .is_some_and(|state| state.mode == super::head::InsertionMode::AfterAfterFrameset)
        {
            reached = true;
            break;
        }
    }
    assert!(reached);
    let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
    assert!(state.pending.is_none());
    for _ in 0..100 {
        let evidence = state
            .before
            .kernel
            .session
            .evidence(&mut driver.control)
            .unwrap();
        let crate::parser::tokenizer::TokenizerPull::Event(event) = state
            .tokenizer
            .as_mut()
            .unwrap()
            .next(&mut driver.control, false, evidence)
            .unwrap()
        else {
            panic!("complete fixture requires actual event");
        };
        match event {
            Event::Token(event) => {
                let token = event.into_token(&mut driver.control).unwrap();
                write(
                    std::mem::size_of::<Option<super::prologue::PendingToken>>()
                        + std::mem::size_of::<super::prologue::PendingToken>(),
                    3,
                    &mut driver.control,
                )
                .unwrap();
                state.pending = Some(super::prologue::PendingToken::from_token(token));
                break;
            }
            event => state.lexical.try_push(&mut driver.control, event).unwrap(),
        }
    }
    let Payload::Text(Some(text)) = &state.pending.as_ref().unwrap().payload else {
        panic!("real trailing whitespace token")
    };
    assert_eq!(text.value.text().code_units(), [10]);
    let origins: Vec<_> = text
        .origins
        .as_slice()
        .iter()
        .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
        .collect();
    assert_eq!(origins, [(0, 1, literal.len() - 3, literal.len() - 1)]);
    let origin_slots = driver
        .control
        .usage()
        .resource(HtmlParseResource::SourceMapEntries)
        .current;
    assert!(driver.step().is_err());
    let stop = driver.control.stop().unwrap();
    assert!(matches!(
        stop,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedNodes,
            ..
        }
    ));
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, super::head::InsertionMode::AfterAfterFrameset);
    // The actual ninth-node/document-binding preflight now precedes lexical
    // transfer. Reconstruction still installed b; the exact CRLF owner stays
    // in the pending slot, and replay added precisely its one b-name origin.
    let retained = state.whitespace_pending.as_ref().unwrap();
    assert_eq!(retained.value.text().code_units(), [10]);
    assert_eq!(retained.origins.as_slice().len(), 1);
    assert_eq!(
        retained.origins.as_slice()[0].source.start(),
        literal.len() - 3
    );
    assert_eq!(
        retained.origins.as_slice()[0].source.end(),
        literal.len() - 1
    );
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current,
        // Replayed name origin, its group, replay cause, namespace group and
        // the actual next-rule trigger plus its InBody delegation, both retained
        // before the refused allocation.
        origin_slots + 7
    );
    assert_eq!(state.formatting.len(), 1);
    assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
    assert!(driver.step().is_err());
    let mut owner = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut owner).is_err());
    let driver = owner.as_mut().unwrap();
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), Some(stop));
    assert_released(&driver.control);
}

#[test]
fn frameset_exact_node_and_temporary_open_element_caps_preserve_terminal_owners() {
    for (caps, expected) in [
        (
            limits(100)
                .into_builder()
                .managed_nodes(5)
                .finish()
                .unwrap(),
            Some(HtmlParseResource::ManagedNodes),
        ),
        (
            limits(100)
                .into_builder()
                .managed_nodes(6)
                .finish()
                .unwrap(),
            None,
        ),
        (limits(2), Some(HtmlParseResource::OpenElements)),
        (limits(3), None),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><frameset><frame></frameset>"),
            caps,
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let result = (0..100)
            .find_map(|_| match driver.step() {
                Ok(step) if step != super::head::HeadStep::Consumed => Some(Ok(step)),
                Err(e) => Some(Err(e)),
                _ => None,
            })
            .unwrap();
        if let Some(resource) = expected {
            assert!(result.is_err());
            let stop = driver.control.stop().unwrap();
            assert!(matches!(stop,ParseStop::LimitExceeded{resource:r,..}if r==resource));
            assert!(matches!(driver.step(),Err(StandaloneError::Stop(first))if first==stop));
            let mut owning = Some(driver);
            assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut owning).is_err());
            let driver = owning.as_mut().unwrap();
            driver.release().unwrap();
            assert_released(&driver.control);
        } else {
            assert!(matches!(result, Ok(super::head::HeadStep::Complete)));
            driver.release().unwrap();
            assert_released(&driver.control);
        }
    }
}

#[test]
fn frameset_partition_diagnostic_failure_retains_popped_run_and_all_origin_owners() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><frameset>A \0\tB</frameset>"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let error = (0..100).find_map(|_| driver.step().err()).unwrap();
    let stop = driver.control.stop().unwrap();
    assert!(
        matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            }
        ),
        "{error:?}"
    );
    // Lexical NULL can exhaust this zero cap before the Text token is delivered.
    // The ordinary nonwhite fixture below targets actual partition emission.
    driver.release().unwrap();
    assert_released(&driver.control);
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><frameset>A \tB</frameset>"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert!((0..100).find_map(|_| driver.step().err()).is_some());
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.body_runs.is_some());
    assert!(state.body_run.is_some());
    let stop = driver.control.stop().unwrap();
    let before = driver.control.usage().clone();
    assert!(driver.step().is_err());
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), Some(stop));
    for category in [
        WorkCategory::InputUnitRead,
        WorkCategory::StateStep,
        WorkCategory::RecordWrite,
        WorkCategory::CopyByte,
    ] {
        assert_eq!(driver.control.usage().work(category), before.work(category));
    }
    assert_released(&driver.control);
}

#[test]
fn frameset_each_mode_cancellation_is_sticky_and_cleanup_performs_no_parse_work() {
    use super::head::InsertionMode;
    for (prefix, mode) in [
        ("<!doctype html><frameset>", InsertionMode::InFrameset),
        (
            "<!doctype html><frameset></frameset>",
            InsertionMode::AfterFrameset,
        ),
        (
            "<!doctype html><frameset></frameset></html>",
            InsertionMode::AfterAfterFrameset,
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!("{prefix}<noframes>x")),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let mut reached = false;
        for _ in 0..100 {
            driver.step().unwrap();
            if driver
                .head
                .as_ref()
                .and_then(|h| h.state.as_ref())
                .is_some_and(|state| state.mode == mode)
            {
                reached = true;
                break;
            }
        }
        assert!(reached);
        request.cancellation().cancel();
        let before = driver.control.usage().clone();
        assert!(matches!(
            driver.step(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        assert!(driver.step().is_err());
        driver.release().unwrap();
        assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
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
            assert_eq!(driver.control.usage().work(category), before.work(category));
        }
        assert_released(&driver.control);
    }
}

#[test]
fn frameset_proofless_head_rejects_before_any_frame_family_effect() {
    let source = HtmlSource::utf8("<!doctype html><head></head><frameset><frame>");
    let mut c = control(100);
    let output = head_boundary(&source, &mut c);
    let mut stage = output.driver;
    write(
        std::mem::size_of::<Option<super::head::HeadBoundary>>(),
        1,
        &mut c,
    )
    .unwrap();
    stage.state.as_mut().unwrap().boundary = None;
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    assert!(stage.step(&mut c).is_err());
    let stop = c.stop().unwrap();
    assert_eq!(
        stop,
        ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance)
    );
    let state = stage.state.as_ref().unwrap();
    assert_eq!(state.before.kernel.stack().unwrap().len(), 1);
    assert!(state.diagnostics().is_empty());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    assert!(stage.take_continuation(&mut c).is_err());
    assert!(stage.step(&mut c).is_err());
    stage.release(&mut c).unwrap();
    assert_eq!(c.stop(), Some(stop));
    assert_released(&c);
}

#[test]
fn frameset_import_taint_rejects_frame_before_diagnostic_stack_and_creation_effects() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><frameset><frame>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    for _ in 0..100 {
        driver.step().unwrap();
        if driver
            .head
            .as_ref()
            .and_then(|h| h.state.as_ref())
            .is_some_and(|state| state.mode == super::head::InsertionMode::InFrameset)
        {
            break;
        }
    }
    let (c, session, _) = driver.standalone_parts().unwrap();
    preexisting(session, Namespace::html(), "imported", c);
    let nodes = driver
        .control
        .usage()
        .resource(HtmlParseResource::ManagedNodes)
        .current;
    assert!(driver.step().is_err());
    let stop = driver.control.stop().unwrap();
    assert_eq!(
        stop,
        ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance)
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
    assert!(state.diagnostics().is_empty());
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        Payload::Tag(Some(_))
    ));
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        nodes
    );
    assert!(driver.step().is_err());
    let mut owner = Some(driver);
    assert!(StandaloneDocumentDriver::take_standalone_fixture(&mut owner).is_err());
    let driver = owner.as_mut().unwrap();
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), Some(stop));
    assert_released(&driver.control);
}

#[test]
fn frameset_finite_work_exhaustion_disposes_every_retained_transition_owner() {
    for cap in [200, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000] {
        let caps = limits(100)
            .into_builder()
            .work_credits(cap)
            .finish()
            .unwrap();
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><form><fieldset><b><frameset><frame>A \tB<noframes>x</noframes></frameset></html> ",
            ),
            caps,
        );
        let mut entry = super::entry::OrdinaryDocumentEntry::new(&request);
        let mut driver = match entry.take_driver() {
            Ok(d) => d,
            Err(_) => {
                entry.release_failure().unwrap();
                continue;
            }
        };
        for _ in 0..1000 {
            match driver.step() {
                Ok(super::head::HeadStep::Boundary(_) | super::head::HeadStep::Complete)
                | Err(_) => break,
                _ => {}
            }
        }
        let stop = driver.control.stop();
        if let Some(first) = stop {
            assert!(matches!(driver.step(),Err(StandaloneError::Stop(s))if s==first));
        }
        // Active cleanup may itself reach the finite work cap; terminal retry
        // disposes the same retained owner, never progresses the parser.
        if driver.release().is_err() {
            driver.release().unwrap();
        }
        assert_released(&driver.control);
        if let Some(first) = stop {
            assert_eq!(driver.control.stop(), Some(first));
        }
    }
}

#[test]
fn ordinary_textarea_rcdata_first_lf_and_form_sibling_order_preserve_exact_native_text() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><textarea name=t>\nA&amp;B&#13;C</textarea>Z</form>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // Dispose real tree, lexical and controller owners before the RED assertion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let textarea = document.node(&form).unwrap().children().next().unwrap();
    let following_text = document.node(&form).unwrap().children().nth(1).unwrap();
    let textarea_child_count = document.node(&textarea).unwrap().children().count();
    let form_child_count = document.node(&form).unwrap().children().count();
    let textarea_text = document.node(&textarea).unwrap().children().next().unwrap();
    let HtmlNodeKind::Text(text) = document.node(&textarea_text).unwrap().kind() else {
        panic!("textarea child must be the actual RCDATA text node");
    };
    let actual_text_units = text.code_units().to_vec();
    let HtmlNodeKind::Text(text) = document.node(&following_text).unwrap().kind() else {
        panic!("the character following textarea must be an actual form text sibling");
    };
    let actual_following_units = text.code_units().to_vec();
    let attributes: Vec<_> = [&form, &textarea]
        .into_iter()
        .map(|id| {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("source-derived form and textarea positions must be actual elements");
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

    // Only plain copied units and result-qualified identities survive disposal.
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_ne!(textarea_text, following_text);
    assert_eq!(textarea_child_count, 1);
    assert_eq!(form_child_count, 2);
    // Physical initial LF is omitted; reference-produced CR is retained as unit 13.
    assert_eq!(actual_text_units, vec![65, 38, 66, 13, 67]);
    assert_eq!(actual_following_units, vec![90]);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "textarea"),
            (4, "#A&B\rC"),
            (3, "#Z"),
        ])
    );
    assert_eq!(
        attributes,
        vec![
            vec![(None::<Vec<u16>>, vec![105, 100], vec![102])],
            vec![(None::<Vec<u16>>, vec![110, 97, 109, 101], vec![116])],
        ]
    );
}

struct TextareaNativeObservation {
    shape: Vec<(usize, Vec<u16>)>,
    raw: Vec<Vec<u16>>,
}

fn textarea_native_observation(source: HtmlSource) -> TextareaNativeObservation {
    let request = crate::HtmlDocumentParseRequest::offline(source, limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let shape = outline(document);
    let mut stack: Vec<_> = document.root().children().collect();
    let mut textareas = Vec::new();
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        stack.extend(node.children());
        if matches!(node.kind(),HtmlNodeKind::Element(e) if e.name().namespace()==&Namespace::html() && e.name().local_name().code_units()==[116,101,120,116,97,114,101,97])
        {
            textareas.push(id);
        }
    }
    let mut raw = Vec::new();
    for id in &textareas {
        let state = frozen.textarea_state(&mut c, id).unwrap();
        assert!(!state.flags.dirty_value() && !state.flags.user_validity());
        raw.push(
            frozen
                .textarea_raw_value(&mut c, id)
                .unwrap()
                .code_units()
                .to_vec(),
        );
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    TextareaNativeObservation { shape, raw }
}

#[test]
fn ordinary_table_textarea_delegation_preserves_foster_tree_and_returns_to_table() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table><textarea>\nA</textarea><tr><td>B</table>C"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }
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
            (2, "textarea"),
            (3, "#A"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#B"),
            (2, "#C")
        ])
    );
}

#[test]
fn textarea_rcdata_first_lf_end_like_and_null_preserve_exact_raw_units() {
    for (input, expected) in [
        ("<textarea>A</textarea>", vec![65]),
        ("<textarea>\r\nA</textarea>", vec![65]),
        ("<textarea>&#10;A</textarea>", vec![65]),
        ("<textarea>\n\nA</textarea>", vec![10, 65]),
        ("<textarea>&amp;B&#13;C</textarea>Z", vec![38, 66, 13, 67]),
        (
            "<textarea>A</textareax>&lt;b></textarea>",
            "A</textareax><b>".encode_utf16().collect(),
        ),
        ("<textarea>A\0B</textarea>", vec![65, 0xfffd, 66]),
        ("<textarea/>A</textarea>", vec![65]),
        ("<textarea>A", vec![65]),
    ] {
        let TextareaNativeObservation { raw, .. } =
            textarea_native_observation(HtmlSource::utf8(input));
        assert_eq!(raw, vec![expected], "{input:?}");
    }
}

#[test]
fn textarea_exact_utf16_and_eof_pop_reset_keep_raw_distinct_from_api_value() {
    let mut units: Vec<u16> = "<textarea>\n".encode_utf16().collect();
    units.extend([0xd800, 0xd83d, 0xde00, 0xdc00, 13, 10]);
    units.extend("&#13;C".encode_utf16());
    let TextareaNativeObservation { raw, .. } =
        textarea_native_observation(HtmlSource::utf16(DocumentString::from_utf16(units)));
    // Physical CRLF is one LF, numeric CR is not API-normalized.
    assert_eq!(raw, vec![vec![0xd800, 0xd83d, 0xde00, 0xdc00, 10, 13, 67]]);
}

#[test]
fn textarea_table_delegation_returns_to_actual_in_table_and_keeps_foster_location() {
    let TextareaNativeObservation { shape, raw } = textarea_native_observation(HtmlSource::utf8(
        "<!doctype html><table><textarea>\nA</textarea><tr><td>B</table>C",
    ));
    assert_eq!(raw, vec![vec![65]]);
    assert_eq!(
        shape,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "textarea"),
            (3, "#A"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#B"),
            (2, "#C")
        ])
    );
}

#[test]
fn textarea_form_association_is_actual_known_state_with_or_without_explicit_form() {
    for (attrs, associated) in [
        ("name=t", true),
        ("name=t form=f", true),
        ("form=missing", false),
        ("form=''", false),
    ] {
        let input = format!("<!doctype html><form id=f><textarea {attrs}>A</textarea></form>");
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(input), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let doc = frozen.document().unwrap();
        let html = doc
            .root()
            .children()
            .find(|id| matches!(doc.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = doc.node(&html).unwrap().children().nth(1).unwrap();
        let form = doc.node(&body).unwrap().children().next().unwrap();
        let textarea = doc.node(&form).unwrap().children().next().unwrap();
        let state = frozen.form_association_state(&mut c, &textarea).unwrap();
        assert_eq!(state.kind, crate::HtmlNativeFormAssociationKind::Textarea);
        let owner = frozen.form_association_owner(&mut c, &textarea).unwrap();
        assert_eq!(owner.as_ref(), associated.then_some(&form));
        assert_eq!(
            frozen
                .textarea_raw_value(&mut c, &textarea)
                .unwrap()
                .code_units(),
            [65]
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn textarea_eof_reset_failure_occurs_after_real_pop_before_return_and_stays_terminal() {
    // html/head/body local names+URIs96, textarea36, doctype4, native/raw A2:
    // before pop current138. Reset's independent new raw A requires peak139.
    for cap in [138, 139] {
        let caps = limits(100)
            .into_builder()
            .managed_utf16_units(cap)
            .finish()
            .unwrap();
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><textarea>A"),
            caps,
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let mut failure = None;
        for _ in 0..1000 {
            match driver.step() {
                Ok(super::head::HeadStep::Complete) => {
                    break;
                }
                Ok(_) => {}
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
        }
        if cap == 138 {
            assert!(failure.is_some());
            let stop = driver.control.stop().unwrap();
            let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
            assert_eq!(state.mode, super::head::InsertionMode::Text);
            assert_eq!(state.return_mode, Some(super::head::InsertionMode::InBody));
            assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
            assert!(matches!(driver.step(),Err(StandaloneError::Stop(s)) if s==stop));
            let work = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            driver.release().unwrap();
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                work
            );
            assert_eq!(driver.control.stop(), Some(stop));
            assert_released(&driver.control);
        } else {
            assert!(failure.is_none());
            let (mut c, mut frozen) = body_document(driver);
            let doc = frozen.document().unwrap();
            let html = doc
                .root()
                .children()
                .find(|id| matches!(doc.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
                .unwrap();
            let body = doc.node(&html).unwrap().children().nth(1).unwrap();
            let textarea = doc.node(&body).unwrap().children().next().unwrap();
            assert_eq!(
                frozen
                    .textarea_raw_value(&mut c, &textarea)
                    .unwrap()
                    .code_units(),
                [65]
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .high_water,
                139
            );
            frozen.release(&mut c).unwrap();
            assert_released(&c);
        }
    }
}

#[test]
fn textarea_text_cancellation_preserves_raw_owner_and_cleanup_runs_no_reset() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><textarea>A&amp;B</textarea>Z"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let mut reached = false;
    for _ in 0..1000 {
        driver.step().unwrap();
        let Some(stage) = driver.head.as_mut() else {
            continue;
        };
        let state = stage.state.as_mut().unwrap();
        if state.mode == super::head::InsertionMode::Text {
            let node = state.before.kernel.current(&mut driver.control).unwrap();
            if state
                .before
                .kernel
                .session
                .textarea_raw_value(&mut driver.control, &node)
                .unwrap()
                .code_units()
                == [65]
            {
                reached = true;
                break;
            }
        }
    }
    assert!(reached);
    request.cancellation().cancel();
    let before = driver.control.usage().clone();
    assert!(matches!(
        driver.step(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert!(driver.step().is_err());
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
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
        assert_eq!(driver.control.usage().work(category), before.work(category));
    }
    assert_released(&driver.control);
}

#[test]
fn textarea_finite_work_stops_retain_all_lexical_native_and_raw_owners_for_disposal() {
    for cap in [200, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000] {
        let caps = limits(100)
            .into_builder()
            .work_credits(cap)
            .finish()
            .unwrap();
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><form id=f><table><textarea name=t>\nA&amp;B&#13;C</textarea><tr><td>Z</table></form>",
            ),
            caps,
        );
        let mut entry = super::entry::OrdinaryDocumentEntry::new(&request);
        let mut driver = match entry.take_driver() {
            Ok(d) => d,
            Err(_) => {
                entry.release_failure().unwrap();
                continue;
            }
        };
        for _ in 0..1000 {
            match driver.step() {
                Ok(super::head::HeadStep::Boundary(_) | super::head::HeadStep::Complete)
                | Err(_) => break,
                _ => {}
            }
        }
        let first = driver.control.stop();
        if let Some(stop) = first {
            assert!(matches!(driver.step(),Err(StandaloneError::Stop(s)) if s==stop));
        }
        if driver.release().is_err() {
            driver.release().unwrap();
        }
        assert_released(&driver.control);
        if let Some(stop) = first {
            assert_eq!(driver.control.stop(), Some(stop));
        }
    }
}

#[test]
fn textarea_proofless_head_rejects_before_diagnostic_stack_and_creation_effects() {
    let source = HtmlSource::utf8("<!doctype html><head></head><body><textarea>A</textarea>");
    let mut c = control(100);
    let mut stage = head_boundary(&source, &mut c).driver;
    write(size_of::<Option<super::head::HeadBoundary>>(), 1, &mut c).unwrap();
    stage.state.as_mut().unwrap().boundary = None;
    stage.step(&mut c).unwrap(); // Actual explicit-body row precedes textarea.
    assert_eq!(
        stage.state.as_ref().unwrap().mode,
        super::head::InsertionMode::InBody
    );
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    assert!(stage.step(&mut c).is_err());
    let first = c.stop().unwrap();
    assert_eq!(
        first,
        ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance)
    );
    let state = stage.state.as_ref().unwrap();
    assert!(state.diagnostics().is_empty());
    assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        Payload::Tag(Some(_))
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    assert!(stage.take_continuation(&mut c).is_err());
    assert!(stage.step(&mut c).is_err());
    stage.release(&mut c).unwrap();
    assert_eq!(c.stop(), Some(first));
    assert_released(&c);
}

#[test]
fn textarea_text_returns_to_actual_table_body_row_cell_and_caption_modes() {
    for (input, expected) in [
        (
            "<!doctype html><table><tbody><textarea>A</textarea><tr><td>B</table>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "textarea"),
                (3, "#A"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#B"),
            ]),
        ),
        (
            "<!doctype html><table><tr><textarea>A</textarea><td>B</table>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "textarea"),
                (3, "#A"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#B"),
            ]),
        ),
        (
            "<!doctype html><table><tr><td><textarea>A</textarea>B</table>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "textarea"),
                (7, "#A"),
                (6, "#B"),
            ]),
        ),
        (
            "<!doctype html><table><caption><textarea>A</textarea>B</caption></table>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "table"),
                (3, "caption"),
                (4, "textarea"),
                (5, "#A"),
                (4, "#B"),
            ]),
        ),
    ] {
        let TextareaNativeObservation { shape, raw } =
            textarea_native_observation(HtmlSource::utf8(input));
        assert_eq!(raw, vec![vec![65]], "{input}");
        assert_eq!(shape, expected, "{input}");
    }
}

#[test]
fn textarea_late_form_id_reset_observes_first_element_order_without_changing_raw() {
    for (prefix, associated) in [("", true), ("<div id=f></div>", false)] {
        let source =
            format!("<!doctype html>{prefix}<textarea form=f>A</textarea><form id=f></form>");
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let doc = frozen.document().unwrap();
        let html = doc
            .root()
            .children()
            .find(|id| matches!(doc.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = doc.node(&html).unwrap().children().nth(1).unwrap();
        let children: Vec<_> = doc.node(&body).unwrap().children().collect();
        let textarea = &children[usize::from(!prefix.is_empty())];
        let form = children.last().unwrap();
        let owner = frozen.form_association_owner(&mut c, textarea).unwrap();
        assert_eq!(owner.as_ref(), associated.then_some(form));
        let state = frozen.form_association_state(&mut c, textarea).unwrap();
        assert!(!state.parser_inserted);
        assert_eq!(
            state.evidence,
            crate::construction::FormAssociationEvidence::Reset
        );
        assert_eq!(
            frozen
                .textarea_raw_value(&mut c, textarea)
                .unwrap()
                .code_units(),
            [65]
        );
        let state = frozen.textarea_state(&mut c, textarea).unwrap();
        assert!(!state.flags.dirty_value() && !state.flags.user_validity());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn ordinary_output_descendant_text_and_form_sibling_order_preserve_native_structure() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><output name=o>A<span>B&amp;C</span>D</output>Z</form>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // Dispose real tree, lexical and controller owners before the RED assertion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let output = document.node(&form).unwrap().children().next().unwrap();
    let following_text = document.node(&form).unwrap().children().nth(1).unwrap();
    let output_children: Vec<_> = document.node(&output).unwrap().children().collect();
    let form_child_count = document.node(&form).unwrap().children().count();
    let span = output_children[1].clone();
    let nested_text = document.node(&span).unwrap().children().next().unwrap();
    let text_units: Vec<_> = [
        &output_children[0],
        &nested_text,
        &output_children[2],
        &following_text,
    ]
    .into_iter()
    .map(|id| {
        let HtmlNodeKind::Text(text) = document.node(id).unwrap().kind() else {
            panic!("source-derived output and following positions must be actual text nodes");
        };
        text.code_units().to_vec()
    })
    .collect();
    let elements: Vec<_> = [&form, &output, &span]
        .into_iter()
        .map(|id| {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("source-derived form, output and span positions must be actual elements");
            };
            (
                element
                    .name()
                    .namespace()
                    .uri()
                    .unwrap()
                    .code_units()
                    .to_vec(),
                element.name().local_name().code_units().to_vec(),
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
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    // Only copied units and result-qualified identities survive owner disposal.
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_ne!(output, span);
    assert_ne!(form, output);
    assert_ne!(nested_text, output_children[0]);
    assert_ne!(nested_text, output_children[2]);
    assert_ne!(following_text, output_children[2]);
    assert_eq!(output_children.len(), 3);
    assert_eq!(form_child_count, 2);
    assert_eq!(
        text_units,
        vec![vec![65], vec![66, 38, 67], vec![68], vec![90]]
    );
    // Descendant output value is AB&CD; direct-child text alone would be AD.
    assert_eq!(text_units[..3].concat(), vec![65, 66, 38, 67, 68]);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "output"),
            (4, "#A"),
            (4, "span"),
            (5, "#B&C"),
            (4, "#D"),
            (3, "#Z"),
        ])
    );
    let html_namespace: Vec<u16> = "http://www.w3.org/1999/xhtml".encode_utf16().collect();
    assert_eq!(
        elements,
        vec![
            (
                html_namespace.clone(),
                vec![102, 111, 114, 109],
                vec![(None::<Vec<u16>>, vec![105, 100], vec![102])]
            ),
            (
                html_namespace.clone(),
                vec![111, 117, 116, 112, 117, 116],
                vec![(None::<Vec<u16>>, vec![110, 97, 109, 101], vec![111])]
            ),
            (html_namespace, vec![115, 112, 97, 110], vec![]),
        ]
    );
}

struct OutputNativeObservation {
    shape: Vec<(usize, Vec<u16>)>,
    values: Vec<Vec<u16>>,
    owners: Vec<bool>,
}
fn output_native_observation(source: HtmlSource) -> OutputNativeObservation {
    use crate::HtmlNativeFormAssociationKind;
    use crate::construction::OutputDefaultValueOverride;
    let request = crate::HtmlDocumentParseRequest::offline(source, limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let shape = outline(document);
    let mut stack: Vec<_> = document.root().children().collect();
    let mut outputs = Vec::new();
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        stack.extend(node.children());
        if matches!(node.kind(),HtmlNodeKind::Element(e) if e.name().namespace()==&Namespace::html()
                    && e.name().local_name().code_units()==[111,117,116,112,117,116])
        {
            outputs.push(id);
        }
    }
    let mut values = Vec::new();
    let mut owners = Vec::new();
    for id in &outputs {
        assert_eq!(
            frozen
                .output_state(&mut c, id)
                .unwrap()
                .default_value_override,
            OutputDefaultValueOverride::Null
        );
        assert_eq!(
            frozen.form_association_state(&mut c, id).unwrap().kind,
            HtmlNativeFormAssociationKind::Output
        );
        owners.push(frozen.form_association_owner(&mut c, id).unwrap().is_some());
        let mut buffer = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
        frozen.append_output_value(&mut c, id, &mut buffer).unwrap();
        let units = buffer.units().to_vec();
        buffer.clear(&mut c).unwrap();
        frozen
            .append_output_default_value(&mut c, id, &mut buffer)
            .unwrap();
        assert_eq!(buffer.units(), units);
        buffer.release(&mut c).unwrap();
        values.push(units);
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    OutputNativeObservation {
        shape,
        values,
        owners,
    }
}

#[test]
fn ordinary_output_values_retain_utf16_and_entity_cr_in_nested_text_but_omit_data_null() {
    let mut units: Vec<u16> = "<!doctype html><form id=f><output>A\r\nB&#13;C<span>"
        .encode_utf16()
        .collect();
    units.extend_from_slice(&[0xD800, 0x20, 0xD83D, 0xDE42, 0xDC00, 0]);
    units.extend("</span><!--ignored--><?ok ignored?></output>Z</form>".encode_utf16());
    let observed =
        output_native_observation(HtmlSource::utf16(crate::DocumentString::from_utf16(units)));
    assert_eq!(
        observed.values,
        vec![vec![
            65, 10, 66, 13, 67, 0xD800, 0x20, 0xD83D, 0xDE42, 0xDC00
        ]]
    );
    assert_eq!(observed.owners, vec![true]);
    assert_eq!(observed.shape[0], (0, vec![104, 116, 109, 108]));
}

#[test]
fn ordinary_table_output_fosters_actual_container_and_restores_cell_dispatch() {
    let observed = output_native_observation(HtmlSource::utf8(
        "<!doctype html><table><output id=o>A<span>B</span>C</output><tr><td>D</table>E",
    ));
    assert_eq!(observed.values, vec![vec![65, 66, 67]]);
    assert_eq!(observed.owners, vec![false]);
    assert_eq!(
        observed.shape,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "output"),
            (3, "#A"),
            (3, "span"),
            (4, "#B"),
            (3, "#C"),
            (2, "table"),
            (3, "tbody"),
            (4, "tr"),
            (5, "td"),
            (6, "#D"),
            (2, "#E"),
        ])
    );
}

#[test]
fn ordinary_output_explicit_form_resolves_late_id_and_formatting_reconstruction_keeps_descendants()
{
    let observed = output_native_observation(HtmlSource::utf8(
        "<!doctype html><b><output form=f>A<i>B</output>C</i></b><form id=f></form>",
    ));
    assert_eq!(observed.values, vec![vec![65, 66]]);
    assert_eq!(observed.owners, vec![true]);
    assert_eq!(
        observed.shape,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "b"),
            (3, "output"),
            (4, "#A"),
            (4, "i"),
            (5, "#B"),
            (3, "i"),
            (4, "#C"),
            (2, "form"),
        ])
    );
}

#[test]
fn ordinary_hidden_inputs_preserve_ordered_value_rewrite_and_form_sibling_order() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><input value='A&#13;B&#10;C' type=hidden name=before><input type=HiDdEn value='A&#13;B&#10;C' name=after><input value='&#13;&#10;' type=hidden name=empty>Z</form>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // Dispose every actual tree/lexer/controller owner before the RED assertion.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    // A real doctype precedes the html element in the Document.
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let input_child_counts: Vec<_> = children[..3]
        .iter()
        .map(|id| document.node(id).unwrap().children().count())
        .collect();
    let HtmlNodeKind::Text(following_text) = document.node(&children[3]).unwrap().kind() else {
        panic!("the literal Z following three void inputs is actual form child text");
    };
    let following_units = following_text.code_units().to_vec();
    let elements: Vec<_> = [&form, &children[0], &children[1], &children[2]]
        .into_iter()
        .map(|id| {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("source-derived form/input positions must be actual elements");
            };
            (
                element
                    .name()
                    .namespace()
                    .uri()
                    .unwrap()
                    .code_units()
                    .to_vec(),
                element.name().local_name().code_units().to_vec(),
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
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    frozen.release(&mut c).unwrap();
    assert_released(&c);

    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(children.len(), 4);
    assert_ne!(children[0], children[1]);
    assert_ne!(children[0], children[2]);
    assert_ne!(children[1], children[2]);
    assert_eq!(input_child_counts, vec![0, 0, 0]);
    assert_eq!(following_units, vec![90]);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "input"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    let html_namespace: Vec<u16> = "http://www.w3.org/1999/xhtml".encode_utf16().collect();
    // Before type: Text strips CR/LF; nonempty Text→Hidden writes ABC back.
    // Type first: Hidden preserves CR/LF. Newline-only before type sanitizes to
    // empty, so the nonempty transition guard leaves the original attribute.
    assert_eq!(
        elements,
        vec![
            (
                html_namespace.clone(),
                vec![102, 111, 114, 109],
                vec![(None::<Vec<u16>>, vec![105, 100], vec![102])]
            ),
            (
                html_namespace.clone(),
                vec![105, 110, 112, 117, 116],
                vec![
                    (None, vec![118, 97, 108, 117, 101], vec![65, 66, 67]),
                    (
                        None,
                        vec![116, 121, 112, 101],
                        vec![104, 105, 100, 100, 101, 110]
                    ),
                    (
                        None,
                        vec![110, 97, 109, 101],
                        vec![98, 101, 102, 111, 114, 101]
                    ),
                ]
            ),
            (
                html_namespace.clone(),
                vec![105, 110, 112, 117, 116],
                vec![
                    (
                        None,
                        vec![116, 121, 112, 101],
                        vec![72, 105, 68, 100, 69, 110]
                    ),
                    (None, vec![118, 97, 108, 117, 101], vec![65, 13, 66, 10, 67]),
                    (None, vec![110, 97, 109, 101], vec![97, 102, 116, 101, 114]),
                ]
            ),
            (
                html_namespace,
                vec![105, 110, 112, 117, 116],
                vec![
                    (None, vec![118, 97, 108, 117, 101], vec![13, 10]),
                    (
                        None,
                        vec![116, 121, 112, 101],
                        vec![104, 105, 100, 100, 101, 110]
                    ),
                    (None, vec![110, 97, 109, 101], vec![101, 109, 112, 116, 121]),
                ]
            ),
        ]
    );
}

#[test]
fn hidden_input_table_direct_placement_and_body_formatting_reconstruction_are_source_ordered() {
    for (source, expected) in [
        (
            "<!doctype html><table><input type=hidden value='A&#13;B'><tr><td>C</table>Z",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "table"),
                (3, "input"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#C"),
                (2, "#Z"),
            ]),
        ),
        (
            "<!doctype html><form><p><b>A</p><input value='A&#10;B' type=hidden></form>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "form"),
                (3, "p"),
                (4, "b"),
                (5, "#A"),
                (3, "b"),
                (4, "input"),
            ]),
        ),
        (
            "<!doctype html><table><caption><input type=hidden></caption><tr><td><input type=hidden></table>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "table"),
                (3, "caption"),
                (4, "input"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "input"),
            ]),
        ),
        (
            "<!doctype html><input type=hidden><frameset><frame name='a'/></frameset>",
            expected_outline(&[(0, "html"), (1, "head"), (1, "frameset"), (2, "frame")]),
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let actual = outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected, "{source}");
    }
}
#[test]
fn hidden_input_explicit_form_late_id_uses_first_element_and_keeps_default_value_exact() {
    for (prefix, associated) in [("", true), ("<div id=f></div>", false)] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!(
                "<!doctype html><input form=f type=hidden value='A&#13;B'>{prefix}<form id=f></form>"
            )),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let input = document.node(&body).unwrap().children().next().unwrap();
        let form = document.node(&body).unwrap().children().last().unwrap();
        let owner = frozen.form_association_owner(&mut c, &input).unwrap();
        assert_eq!(owner, associated.then(|| form.clone()));
        assert_eq!(
            frozen.form_association_state(&mut c, &input).unwrap().kind,
            crate::HtmlNativeFormAssociationKind::Input
        );
        let mut value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
        frozen
            .append_input_default_value(&mut c, &input, &mut value)
            .unwrap();
        let actual = value.units().to_vec();
        value.release(&mut c).unwrap();
        drop(owner);
        drop(input);
        drop(form);
        drop(body);
        drop(html);
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, [65, 13, 66]);
    }
}
#[test]
fn hidden_input_proofless_head_rejects_before_diagnostic_stack_or_creation() {
    let source = HtmlSource::utf8("<!doctype html><head></head><body><input type='hidden'/>");
    let mut c = control(100);
    let mut stage = head_boundary(&source, &mut c).driver;
    write(size_of::<Option<super::head::HeadBoundary>>(), 1, &mut c).unwrap();
    stage.state.as_mut().unwrap().boundary = None;
    stage.step(&mut c).unwrap();
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    assert!(stage.step(&mut c).is_err());
    let first = c.stop().unwrap();
    assert_eq!(
        first,
        ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance)
    );
    let state = stage.state.as_ref().unwrap();
    assert!(state.diagnostics().is_empty());
    assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
    assert!(matches!(
        state.pending.as_ref().unwrap().payload,
        Payload::Tag(Some(_))
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    assert!(stage.step(&mut c).is_err());
    assert!(stage.take_continuation(&mut c).is_err());
    stage.release(&mut c).unwrap();
    assert_eq!(c.stop(), Some(first));
    assert_released(&c);
}
#[test]
fn nonhidden_color_reached_fact_refusal_retains_partial_creation() {
    // Document/doctype/html/head/body/input: six actual records.
    assert_color_palette_refusal(
        "<!doctype html><input type=color value=currentcolor>",
        super::head::InsertionMode::InBody,
        2,
        6,
        false,
        false,
    );
}

#[test]
fn ordinary_select_option_pop_clones_selected_content_with_distinct_native_children() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><select name=s><button><selectedcontent></selectedcontent></button><option value=a><b>A&amp;B</b></option><optgroup label=g><option selected value=b><i>C</i></option></optgroup></select>Z</form>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        // Dispose the actual tree, lexer and payload owners before asserting RED.
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    // Snapshot the real native tree without assuming the expected structure.
    // The doctype is observed separately by mode and is skipped as an element.
    let mut stack: Vec<_> = document.root().children().collect();
    stack.reverse();
    let mut elements = Vec::new();
    let mut texts = Vec::new();
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        let children: Vec<_> = node.children().collect();
        stack.extend(children.iter().rev().cloned());
        match node.kind() {
            HtmlNodeKind::Element(element) => elements.push((
                id,
                children,
                element
                    .name()
                    .namespace()
                    .uri()
                    .map(|uri| uri.code_units().to_vec()),
                element.name().local_name().code_units().to_vec(),
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
                    .collect::<Vec<_>>(),
            )),
            HtmlNodeKind::Text(text) => texts.push((id, text.code_units().to_vec())),
            _ => {}
        }
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);

    // HTML selectedness keeps the later selected option. Its parser pop clones
    // the option's children into selectedcontent, not the option element itself.
    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "select"),
            (4, "button"),
            (5, "selectedcontent"),
            (6, "i"),
            (7, "#C"),
            (4, "option"),
            (5, "b"),
            (6, "#A&B"),
            (4, "optgroup"),
            (5, "option"),
            (6, "i"),
            (7, "#C"),
            (3, "#Z"),
        ])
    );
    let units = |value: &str| value.encode_utf16().collect::<Vec<_>>();
    let html_namespace = units("http://www.w3.org/1999/xhtml");
    let actual_elements: Vec<_> = elements
        .iter()
        .map(|(_, _, namespace, name, attributes)| {
            (namespace.clone(), name.clone(), attributes.clone())
        })
        .collect();
    let expected_elements: Vec<_> = [
        ("html", vec![]),
        ("head", vec![]),
        ("body", vec![]),
        ("form", vec![("id", "f")]),
        ("select", vec![("name", "s")]),
        ("button", vec![]),
        ("selectedcontent", vec![]),
        ("i", vec![]),
        ("option", vec![("value", "a")]),
        ("b", vec![]),
        ("optgroup", vec![("label", "g")]),
        ("option", vec![("selected", ""), ("value", "b")]),
        ("i", vec![]),
    ]
    .into_iter()
    .map(|(name, attributes)| {
        (
            Some(html_namespace.clone()),
            units(name),
            attributes
                .into_iter()
                .map(|(name, value)| (None::<Vec<u16>>, units(name), units(value)))
                .collect::<Vec<_>>(),
        )
    })
    .collect();
    assert_eq!(actual_elements, expected_elements);
    assert_eq!(
        texts
            .iter()
            .map(|(_, text)| text.clone())
            .collect::<Vec<_>>(),
        vec![units("C"), units("A&B"), units("C"), units("Z")]
    );
    assert_ne!(elements[7].0, elements[12].0);
    assert_ne!(texts[0].0, texts[2].0);
    assert_eq!(elements[6].1, vec![elements[7].0.clone()]);
    assert_eq!(elements[7].1, vec![texts[0].0.clone()]);
    assert_eq!(elements[11].1, vec![elements[12].0.clone()]);
    assert_eq!(elements[12].1, vec![texts[2].0.clone()]);
}

#[test]
fn select_family_native_states_follow_size_multiple_disabled_and_last_selected_rules() {
    for (attributes, options, expected) in [
        ("", "<option>A<option>B", vec![true, false]),
        ("size=0", "<option>A<option>B", vec![false, false]),
        ("size=-0", "<option>A<option>B", vec![false, false]),
        ("size=+1junk", "<option>A<option>B", vec![true, false]),
        ("size=-1", "<option>A<option>B", vec![true, false]),
        (
            "size=999999999999999999999999",
            "<option>A<option>B",
            vec![false, false],
        ),
        ("multiple", "<option>A<option>B", vec![false, false]),
        (
            "multiple",
            "<option selected>A<option selected>B",
            vec![true, true],
        ),
        (
            "",
            "<option selected>A<option selected>B",
            vec![false, true],
        ),
        (
            "",
            "<optgroup disabled><option>A</optgroup><option>B",
            vec![false, true],
        ),
        ("", "<option disabled>A<option>B", vec![false, true]),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!(
                "<!doctype html><select {attributes}>{options}</select>"
            )),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
        let mut pending = vec![document.root().id()];
        let mut selected = vec![];
        let mut select = None;
        let mut option_ids = vec![];
        while let Some(id) = pending.pop() {
            let node = document.node(&id).unwrap();
            if let HtmlNodeKind::Element(element) = node.kind() {
                if element.name().local_name() == &DocumentString::from("select") {
                    assert!(
                        !frozen
                            .select_state(&mut c, &id)
                            .unwrap()
                            .flags
                            .user_validity()
                    );
                    select = Some(id.clone());
                } else if element.name().local_name() == &DocumentString::from("option") {
                    let state = frozen.option_state(&mut c, &id).unwrap();
                    selected.push(state.flags.selectedness());
                    assert!(!state.flags.dirtiness());
                    option_ids.push(id.clone());
                }
            }
            pending.extend(node.children().collect::<Vec<_>>().into_iter().rev());
        }
        assert_eq!(selected, expected, "{attributes} {options}");
        for option in option_ids {
            assert_eq!(
                frozen.option_nearest_select(&mut c, &option).unwrap(),
                select
            );
        }
        drop(select);
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn select_option_appends_agree_with_the_full_selectedness_algorithm_over_every_short_sequence() {
    // Every option appended in test builds is compared with the full algorithm
    // run on the options before it, so parsing each sequence is the check.
    let options = [
        "<option>A",
        "<option selected>A",
        "<option disabled>A",
        "<optgroup disabled><option>A</optgroup>",
        "<optgroup><option selected>A</optgroup>",
    ];
    let mut sequences = vec![String::new()];
    let mut layer = vec![String::new()];
    for _ in 0..3 {
        layer = layer
            .iter()
            .flat_map(|prefix| {
                options
                    .iter()
                    .map(move |option| format!("{prefix}{option}"))
            })
            .collect();
        sequences.extend(layer.iter().cloned());
    }
    // The second wrapper has the adoption agency move a div of options out of a
    // datalist into the select, connecting every option in one insertion.
    for (open, close) in [("", ""), ("<b><datalist><div>", "</b>")] {
        for attributes in ["", "size=0", "size=2", "multiple"] {
            for sequence in &sequences {
                let request = crate::HtmlDocumentParseRequest::offline(
                    HtmlSource::utf8(format!(
                        "<!doctype html><select {attributes}>{open}{sequence}{close}</select>"
                    )),
                    limits(100),
                );
                let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
                assert_eq!(
                    standalone_boundary(&mut driver),
                    super::head::HeadStep::Complete
                );
                let (mut c, mut frozen) = body_document(driver);
                frozen.release(&mut c).unwrap();
                assert_released(&c);
            }
        }
    }
}

#[test]
fn style_order_extensions_agree_with_the_tree_walk_over_varied_documents() {
    // Every style trigger on the main tree is compared in test builds with a
    // walk of the tree, so parsing each document is the check. The documents
    // insert styles at the end, mid-tree, through foster parenting and the
    // adoption agency, in templates and foreign content, and with text between.
    for body in [
        "<head><style>a{}</style><style>b{}</style><body><style>c{}</style>",
        "<style>a{}</style><!--c--><style>b{}</style>t<style>c{}</style>",
        "<table><style>a{}</style><tr><td><style>b{}</style></table><style>c{}",
        "<table><b><style>a{}</style></b><tr><td>x</table><style>c{}",
        "<b><style>a{}</style><p>x</b><style>b{}</style>",
        "<b><p><style>a{}</style></b>x</p><style>b{}</style>",
        "<a>1<style>a{}</style><a>2<style>b{}</style></a></a><style>c{}",
        "<template><style>a{}</style></template><style>b{}</style>\
         <template><template><style>c{}</style></template></template><style>d{}",
        "<svg><style>a{}</style></svg><style>b{}</style><math><style>c{}</style></math>",
        "<select><style>a{}</style><option>x</select><style>b{}",
        "<div><div><style>a{}</style></div><style>b{}</style></div><style>c{}",
        "<p><style>a{}</style><p><style>b{}</style><ul><li><style>c{}",
        "<form><style>a{}</style></form><form><style>b{}</style><style>c{}</style>",
        "<style>a{}</style><frameset><style>b{}</style></frameset><style>c{}",
        "<style type=text/css media=print title=t nonce=n>a{}</style><style>b{}",
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!("<!doctype html>{body}")),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn select_family_modern_body_dispatch_keeps_generic_children_and_hidden_guard() {
    for (source, expected) in [
        (
            "<select><div>x<option>A</option></div></select>Z",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "div"),
                (4, "#x"),
                (4, "option"),
                (5, "#A"),
                (2, "#Z"),
            ],
        ),
        (
            "<select><option>A<select><option>B",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "option"),
                (4, "#A"),
                (2, "option"),
                (3, "#B"),
            ],
        ),
        (
            "<select><option>A<input type=hidden value=v>Z",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "option"),
                (4, "#A"),
                (2, "input"),
                (2, "#Z"),
            ],
        ),
        (
            "<select><option>A<hr><option>B",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "option"),
                (4, "#A"),
                (3, "hr"),
                (3, "option"),
                (4, "#B"),
            ],
        ),
        (
            "<!doctype html><table><select><option>A</select><tr><td>B",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "option"),
                (4, "#A"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#B"),
            ],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete,
            "{source}"
        );
        let (mut c, mut frozen) = body_document(driver);
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
fn select_family_selectedcontent_disabled_and_multiple_are_intrinsic_not_empty_callbacks() {
    for (attributes, target, has_clone, disabled) in [
        (
            "",
            "<button><selectedcontent></selectedcontent></button>",
            true,
            false,
        ),
        (
            "multiple",
            "<button><selectedcontent></selectedcontent></button>",
            false,
            false,
        ),
        (
            "",
            "<button><selectedcontent><selectedcontent></selectedcontent></selectedcontent></button>",
            true,
            false,
        ),
        (
            "",
            "<option><selectedcontent></selectedcontent></option>",
            false,
            true,
        ),
    ] {
        let source = format!(
            "<!doctype html><select {attributes}>{target}<option selected><i>C</i></option></select>"
        );
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let doc = frozen.document().unwrap();
        let mut pending = vec![doc.root().id()];
        let mut targets = vec![];
        let mut italic = vec![];
        while let Some(id) = pending.pop() {
            let node = doc.node(&id).unwrap();
            if let HtmlNodeKind::Element(e) = node.kind() {
                if e.name().local_name() == &DocumentString::from("selectedcontent") {
                    targets.push(id.clone());
                }
                if e.name().local_name() == &DocumentString::from("i") {
                    italic.push(id.clone());
                }
            }
            pending.extend(node.children().collect::<Vec<_>>().into_iter().rev());
        }
        assert_eq!(italic.len(), 1 + usize::from(has_clone));
        assert_eq!(
            frozen
                .selectedcontent_state(&mut c, &targets[0])
                .unwrap()
                .disabled,
            disabled
        );
        if has_clone {
            assert_ne!(italic[0], italic[1]);
        }
        drop((italic, targets));
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn select_fragment_siblings_finish_insertion_before_post_connection_cloning() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><select><option><div><option selected>A</option></div><div><option selected>B</option></div></option><selectedcontent></selectedcontent></select>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    before_ordinary_end(&mut driver);
    // Four implicit structural records + doctype; nine source element/text
    // records; one outer-clone fragment; six sibling-subtree clones; the final
    // B update's fragment and text clone.
    // No intermediate A update is permitted before both insertion hooks finish.
    let before_checkpoint = driver
        .control
        .usage()
        .resource(HtmlParseResource::ManagedNodes)
        .current;
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(matches!(
        session.freeze(c),
        Err(SessionError::PendingIntrinsicWork)
    ));
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(before_checkpoint, 23);
}
#[test]
fn ordinary_svg_adjusted_names_and_foreign_integration_preserve_native_namespaces_and_order() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><body><svg viewbox=\"0 0 1 1\"><lineargradient id=g xlink:href=\"#x\"/><foreignobject><p>x</p></foreignobject></svg><p>tail</p>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    // Observe actual expanded names and ordered attributes, independently of
    // the outline observer and the literal expected tree below.
    let mut stack: Vec<_> = document.root().children().collect();
    stack.reverse();
    let mut elements = Vec::new();
    let mut texts = Vec::new();
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        let children: Vec<_> = node.children().collect();
        stack.extend(children.iter().rev().cloned());
        match node.kind() {
            HtmlNodeKind::Element(element) => elements.push((
                id,
                children,
                element
                    .name()
                    .namespace()
                    .uri()
                    .map(|uri| uri.code_units().to_vec()),
                element
                    .name()
                    .prefix()
                    .map(|prefix| prefix.code_units().to_vec()),
                element.name().local_name().code_units().to_vec(),
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
                            attribute
                                .name()
                                .prefix()
                                .map(|prefix| prefix.code_units().to_vec()),
                            attribute.name().local_name().code_units().to_vec(),
                            attribute.value().code_units().to_vec(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )),
            HtmlNodeKind::Text(text) => texts.push((id, text.code_units().to_vec())),
            _ => {}
        }
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);

    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "svg"),
            (3, "linearGradient"),
            (3, "foreignObject"),
            (4, "p"),
            (5, "#x"),
            (2, "p"),
            (3, "#tail"),
        ])
    );
    let units = |value: &str| value.encode_utf16().collect::<Vec<_>>();
    let actual_elements: Vec<_> = elements
        .iter()
        .map(|(_, _, namespace, prefix, name, attributes)| {
            (
                namespace.clone(),
                prefix.clone(),
                name.clone(),
                attributes.clone(),
            )
        })
        .collect();
    let html = "http://www.w3.org/1999/xhtml";
    let svg = "http://www.w3.org/2000/svg";
    let expected_elements: Vec<_> = [
        (html, "html", vec![]),
        (html, "head", vec![]),
        (html, "body", vec![]),
        (svg, "svg", vec![(None, None, "viewBox", "0 0 1 1")]),
        (
            svg,
            "linearGradient",
            vec![
                (None, None, "id", "g"),
                (
                    Some("http://www.w3.org/1999/xlink"),
                    Some("xlink"),
                    "href",
                    "#x",
                ),
            ],
        ),
        (svg, "foreignObject", vec![]),
        (html, "p", vec![]),
        (html, "p", vec![]),
    ]
    .into_iter()
    .map(|(namespace, name, attributes)| {
        (
            Some(units(namespace)),
            None::<Vec<u16>>,
            units(name),
            attributes
                .into_iter()
                .map(|(namespace, prefix, name, value)| {
                    (
                        namespace.map(units),
                        prefix.map(units),
                        units(name),
                        units(value),
                    )
                })
                .collect::<Vec<_>>(),
        )
    })
    .collect();
    assert_eq!(actual_elements, expected_elements);
    assert_eq!(
        texts
            .iter()
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>(),
        vec![units("x"), units("tail")]
    );
    assert_eq!(
        elements[2].1,
        vec![elements[3].0.clone(), elements[7].0.clone()]
    );
    assert_eq!(
        elements[3].1,
        vec![elements[4].0.clone(), elements[5].0.clone()]
    );
    assert!(elements[4].1.is_empty());
    assert_eq!(elements[5].1, vec![elements[6].0.clone()]);
    assert_eq!(elements[6].1, vec![texts[0].0.clone()]);
    assert_eq!(elements[7].1, vec![texts[1].0.clone()]);
    assert_ne!(elements[6].0, elements[7].0);
    assert_ne!(texts[0].0, texts[1].0);
}

#[test]
fn foreign_cdata_null_pi_comment_doctype_and_surrogates_keep_real_native_payloads() {
    let mut units: Vec<u16> = "<!doctype html><svg><![CDATA[A<]]>\0"
        .encode_utf16()
        .collect();
    units.extend_from_slice(&[0xd800, 0xd83d, 0xde00, 0xdc00]);
    units.extend("<!--c--><?work data?><!doctype wrong></svg><p>Z</p>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(!state.frameset_ok);
    assert_eq!(
        state
            .diagnostics()
            .iter()
            .filter(|d| d.condition == super::head::HeadCondition::NullInForeign)
            .count(),
        1
    );
    assert_eq!(
        state
            .diagnostics()
            .iter()
            .filter(|d| d.condition == super::head::HeadCondition::UnexpectedDoctype)
            .count(),
        1
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual = outline(document);
    let mut expected = expected_outline(&[(0, "html"), (1, "head"), (1, "body"), (2, "svg")]);
    let mut text: Vec<u16> = "#A<\u{fffd}".encode_utf16().collect();
    text.extend_from_slice(&[0xd800, 0xd83d, 0xde00, 0xdc00]);
    expected.push((3, text));
    expected.extend(expected_outline(&[
        (3, "!c"),
        (3, "?work"),
        (2, "p"),
        (3, "#Z"),
    ]));
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let svg = document.node(&body).unwrap().children().next().unwrap();
    let pi = document.node(&svg).unwrap().children().nth(2).unwrap();
    let HtmlNodeKind::ProcessingInstruction(pi) = document.node(&pi).unwrap().kind() else {
        panic!("real PI");
    };
    let data = pi.data().code_units().to_vec();
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual, expected);
    assert_eq!(data, "data".encode_utf16().collect::<Vec<_>>());
}

#[test]
fn foreign_encoding_integration_math_text_exceptions_and_annotation_breakout_are_qualified() {
    for (source, rows) in [
        (
            "<!doctype html><math><annotation-xml encoding='TeXt/HtMl'><svg><g><p>X</p></svg></annotation-xml></math><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "math"),
                (3, "annotation-xml"),
                (4, "svg"),
                (5, "g"),
                (4, "p"),
                (5, "#X"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
        (
            "<!doctype html><math><annotation-xml encoding=' text/html'><g><p>X</p></math>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "math"),
                (3, "annotation-xml"),
                (4, "g"),
                (2, "p"),
                (3, "#X"),
            ],
        ),
        (
            "<!doctype html><math><mtext><mglyph/><malignmark/><b>X</b></mtext></math>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "math"),
                (3, "mtext"),
                (4, "mglyph"),
                (4, "malignmark"),
                (4, "b"),
                (5, "#X"),
            ],
        ),
        (
            "<!doctype html><svg><desc><![CDATA[X<]]><p>Y</p></desc><title><b>Z</b></title></svg>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "svg"),
                (3, "desc"),
                (4, "#X<"),
                (4, "p"),
                (5, "#Y"),
                (3, "title"),
                (4, "b"),
                (5, "#Z"),
            ],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete,
            "{source}"
        );
        let (mut c, mut frozen) = body_document(driver);
        let actual = outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected_outline(&rows), "{source}");
    }
}

#[test]
fn foreign_font_breakout_mismatched_end_and_table_foster_keep_actual_order() {
    for (source, rows) in [
        (
            "<!doctype html><svg><g><font FACE=x>X</font></svg><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "svg"),
                (3, "g"),
                (2, "font"),
                (3, "#X"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
        (
            "<!doctype html><svg><g><font foo=x>X</font></svg><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "svg"),
                (3, "g"),
                (4, "font"),
                (5, "#X"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
        (
            "<!doctype html><svg><g><path/></SVG><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "svg"),
                (3, "g"),
                (4, "path"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
        (
            "<!doctype html><table><svg><g>X</g></svg><tr><td>Y</td></tr></table>Z",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "svg"),
                (3, "g"),
                (4, "#X"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#Y"),
                (2, "#Z"),
            ],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete,
            "{source}"
        );
        let (mut c, mut frozen) = body_document(driver);
        let actual = outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected_outline(&rows), "{source}");
    }
}

#[test]
fn foreign_svg_script_fresh_start_and_self_closing_preserve_native_attributes_and_processed_state()
{
    for (source, processed) in [
        ("<!doctype html><svg><script data-x=kept>", false),
        ("<!doctype html><svg><script data-x='kept'/>", true),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let doc = frozen.document().unwrap();
        let mut pending: Vec<_> = doc.root().children().collect();
        let mut found = None;
        while let Some(id) = pending.pop() {
            let node = doc.node(&id).unwrap();
            pending.extend(node.children());
            if matches!(node.kind(), HtmlNodeKind::Element(e)
                if e.name().namespace()==&Namespace::svg() && e.name().local_name()==&DocumentString::from("script"))
            {
                let HtmlNodeKind::Element(e) = node.kind() else {
                    unreachable!()
                };
                assert_eq!(e.attributes().len(), 1);
                assert_eq!(
                    e.attributes().as_slice()[0].name().local_name(),
                    &DocumentString::from("data-x")
                );
                assert_eq!(
                    e.attributes().as_slice()[0].value(),
                    &DocumentString::from("kept")
                );
                found = Some(id);
            }
        }
        let id = found.expect("actual native SVG script");
        assert_eq!(
            frozen
                .svg_script_state(&mut c, &id)
                .unwrap()
                .flags
                .already_processed(),
            processed
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn foreign_all_svg_name_and_attribute_adjustments_follow_literal_selected_tables() {
    // Defining HTML877d tables, independent of the implementation's constants.
    let names = "altGlyph altGlyphDef altGlyphItem animateColor animateMotion animateTransform clipPath feBlend feColorMatrix feComponentTransfer feComposite feConvolveMatrix feDiffuseLighting feDisplacementMap feDistantLight feDropShadow feFlood feFuncA feFuncB feFuncG feFuncR feGaussianBlur feImage feMerge feMergeNode feMorphology feOffset fePointLight feSpecularLighting feSpotLight feTile feTurbulence foreignObject glyphRef linearGradient radialGradient textPath";
    let attributes = "attributeName attributeType baseFrequency baseProfile calcMode clipPathUnits diffuseConstant edgeMode filterUnits glyphRef gradientTransform gradientUnits kernelMatrix kernelUnitLength keyPoints keySplines keyTimes lengthAdjust limitingConeAngle markerHeight markerUnits markerWidth maskContentUnits maskUnits numOctaves pathLength patternContentUnits patternTransform patternUnits pointsAtX pointsAtY pointsAtZ preserveAlpha preserveAspectRatio primitiveUnits refX refY repeatCount repeatDur requiredExtensions requiredFeatures specularConstant specularExponent spreadMethod startOffset stdDeviation stitchTiles surfaceScale systemLanguage tableValues targetX targetY textLength viewBox viewTarget xChannelSelector yChannelSelector zoomAndPan";
    // Each independent literal uses the same bounded request rather than one
    // growing tokenizer name trie obscuring this table-mapping observation.
    for name in names.split_whitespace() {
        let source = format!("<!doctype html><svg><{}/></svg>", name.to_ascii_lowercase());
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let svg = document.node(&body).unwrap().children().next().unwrap();
        let id = document.node(&svg).unwrap().children().next().unwrap();
        let node = document.node(&id).unwrap();
        let HtmlNodeKind::Element(element) = node.kind() else {
            panic!("adjusted element");
        };
        let actual = (
            element.name().namespace() == &Namespace::svg(),
            element.name().local_name().code_units().to_vec(),
            node.children().count(),
        );
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, (true, name.encode_utf16().collect::<Vec<_>>(), 0));
    }
    for (index, name) in attributes.split_whitespace().enumerate() {
        let source = format!(
            "<!doctype html><svg {}='v{index}'/>",
            name.to_ascii_lowercase()
        );
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let svg = document.node(&body).unwrap().children().next().unwrap();
        let HtmlNodeKind::Element(element) = document.node(&svg).unwrap().kind() else {
            panic!("svg");
        };
        let actual: Vec<_> = element
            .attributes()
            .iter()
            .map(|a| {
                (
                    a.name().namespace().uri().is_none(),
                    a.name().prefix().is_none(),
                    a.name().local_name().code_units().to_vec(),
                    a.value().code_units().to_vec(),
                )
            })
            .collect();
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(
            actual,
            vec![(
                true,
                true,
                name.encode_utf16().collect::<Vec<_>>(),
                format!("v{index}").encode_utf16().collect::<Vec<_>>()
            )]
        );
    }
}

#[test]
fn foreign_all_namespace_adjustments_unknown_colons_and_xmlns_errors_keep_original_values() {
    let source = "<!doctype html><svg xlink:actuate=0 xlink:arcrole=1 xlink:href=2 xlink:role=3 xlink:show=4 xlink:title=5 xlink:type=6 xml:lang=7 xml:space=8 xmlns=wrong xmlns:xlink=also-wrong xlink:unknown=U foo:bar=V viewboxes='N'/>";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(
        state
            .diagnostics()
            .iter()
            .filter(|d| d.condition == super::head::HeadCondition::ForeignNamespaceMismatch)
            .count(),
        2
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let svg = document.node(&body).unwrap().children().next().unwrap();
    let HtmlNodeKind::Element(element) = document.node(&svg).unwrap().kind() else {
        panic!("real svg");
    };
    let namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let actual: Vec<_> = element
        .attributes()
        .iter()
        .map(|a| {
            (
                a.name().namespace().uri().map(|s| s.code_units().to_vec()),
                a.name().prefix().map(|s| s.code_units().to_vec()),
                a.name().local_name().code_units().to_vec(),
                a.value().code_units().to_vec(),
            )
        })
        .collect();
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    let xlink = "http://www.w3.org/1999/xlink";
    let xml = "http://www.w3.org/XML/1998/namespace";
    let xmlns = "http://www.w3.org/2000/xmlns/";
    let expected = [
        (Some(xlink), Some("xlink"), "actuate", "0"),
        (Some(xlink), Some("xlink"), "arcrole", "1"),
        (Some(xlink), Some("xlink"), "href", "2"),
        (Some(xlink), Some("xlink"), "role", "3"),
        (Some(xlink), Some("xlink"), "show", "4"),
        (Some(xlink), Some("xlink"), "title", "5"),
        (Some(xlink), Some("xlink"), "type", "6"),
        (Some(xml), Some("xml"), "lang", "7"),
        (Some(xml), Some("xml"), "space", "8"),
        (Some(xmlns), None, "xmlns", "wrong"),
        (Some(xmlns), Some("xmlns"), "xlink", "also-wrong"),
        (None, None, "xlink:unknown", "U"),
        (None, None, "foo:bar", "V"),
        (None, None, "viewboxes", "N"),
    ];
    let units = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert_eq!(namespace, units("http://www.w3.org/2000/svg"));
    assert_eq!(
        actual,
        expected
            .into_iter()
            .map(|(ns, prefix, name, value)| (
                ns.map(units),
                prefix.map(units),
                units(name),
                units(value)
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn foreign_local_control_spellings_have_no_html_metadata_and_keep_actual_namespace() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><math><input/><textarea/><script/><form/><select/><option/></math><p>Z</p>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.current_form.is_none());
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let math = document.node(&body).unwrap().children().next().unwrap();
    for (id, literal) in document
        .node(&math)
        .unwrap()
        .children()
        .zip(["input", "textarea", "script", "form", "select", "option"])
    {
        let HtmlNodeKind::Element(element) = document.node(&id).unwrap().kind() else {
            panic!("foreign child");
        };
        assert_eq!(element.name().namespace(), &Namespace::mathml());
        assert_eq!(element.name().local_name(), &DocumentString::from(literal));
        assert!(matches!(
            frozen.form_association_state(&mut c, &id),
            Err(SessionError::FormAssociationMetadataUnavailable)
        ));
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn foreign_null_diagnostic_limit_preserves_recovery_owner_and_terminal_cleanup() {
    let source = HtmlSource::utf16(DocumentString::from_utf16(
        "<!doctype html><svg>"
            .encode_utf16()
            .chain([0, 0])
            .chain("</svg>".encode_utf16())
            .collect::<Vec<_>>(),
    ));
    let request = crate::HtmlDocumentParseRequest::offline(
        source,
        limits(100).into_builder().diagnostics(1).finish().unwrap(),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let _first = loop {
        match driver.step() {
            Err(error) => break error,
            Ok(super::head::HeadStep::Boundary(_)) => panic!("diagnostic cap must stop"),
            _ => {}
        }
    };
    let stop = driver.control.stop().unwrap();
    assert!(matches!(
        stop,
        super::super::control::ParseStop::LimitExceeded {
            resource: HtmlParseResource::Diagnostics,
            ..
        }
    ));
    assert!(matches!(driver.step(), Err(StandaloneError::Stop(replay)) if replay == stop));
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert!(state.body_run.is_some());
    assert!(state.whitespace_pending.is_none());
    assert!(state.frameset_ok);
    driver.release().unwrap();
    assert_eq!(driver.control.stop(), Some(stop));
    assert_released(&driver.control);
}

#[test]
fn foreign_proofless_head_preserves_start_token_before_reconstruction_and_creation() {
    for tag in ["svg", "math"] {
        let source = HtmlSource::utf8(format!(
            "<!doctype html><head></head><body><{tag} data-x=kept>"
        ));
        let mut c = control(100);
        let mut stage = head_boundary(&source, &mut c).driver;
        write(size_of::<Option<super::head::HeadBoundary>>(), 1, &mut c).unwrap();
        stage.state.as_mut().unwrap().boundary = None;
        stage.step(&mut c).unwrap();
        let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        assert!(stage.step(&mut c).is_err());
        let stop = ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance);
        assert_eq!(c.stop(), Some(stop));
        let state = stage.state.as_ref().unwrap();
        assert!(state.diagnostics().is_empty());
        assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
        let Payload::Tag(Some(input)) = &state.pending.as_ref().unwrap().payload else {
            panic!("untouched tag");
        };
        assert_eq!(input.name.value.text(), &DocumentString::from(tag));
        assert_eq!(
            input.attributes.as_slice()[0].value.value.text(),
            &DocumentString::from("kept")
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            nodes
        );
        assert!(stage.step(&mut c).is_err());
        stage.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(stop));
        assert_released(&c);
    }
}

#[test]
fn foreign_attributes_keep_literal_non_scalar_utf16_and_unknown_name_units() {
    let mut units: Vec<u16> = "<!doctype html><svg data-".encode_utf16().collect();
    units.push(0xd800);
    units.extend("='".encode_utf16());
    let value = [0xd800, 0xd83d, 0xde00, 0xdc00];
    units.extend(value);
    units.extend("'/>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let svg = document.node(&body).unwrap().children().next().unwrap();
    let HtmlNodeKind::Element(element) = document.node(&svg).unwrap().kind() else {
        panic!("svg");
    };
    let attr = &element.attributes().as_slice()[0];
    let observed = (
        attr.name().namespace().uri().is_none(),
        attr.name().prefix().is_none(),
        attr.name().local_name().code_units().to_vec(),
        attr.value().code_units().to_vec(),
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    let mut name: Vec<u16> = "data-".encode_utf16().collect();
    name.push(0xd800);
    assert_eq!(observed, (true, true, name, value.to_vec()));
}

#[test]
fn ordinary_selectedcontent_clones_foreign_children_with_distinct_qualified_native_records() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><select name=s><button><selectedcontent></selectedcontent></button><option selected><svg viewbox='0 0 1 1'><lineargradient id=g xlink:href='#x'/></svg><math definitionurl=u><mtext xml:lang=en>M</mtext></math></option></select>Z",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    let mut stack: Vec<_> = document.root().children().collect();
    stack.reverse();
    let mut elements = Vec::new();
    let mut texts = Vec::new();
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        let children: Vec<_> = node.children().collect();
        stack.extend(children.iter().rev().cloned());
        match node.kind() {
            HtmlNodeKind::Element(element) => elements.push((
                id,
                children,
                element
                    .name()
                    .namespace()
                    .uri()
                    .map(|s| s.code_units().to_vec()),
                element.name().prefix().map(|s| s.code_units().to_vec()),
                element.name().local_name().code_units().to_vec(),
                element
                    .attributes()
                    .iter()
                    .map(|attribute| {
                        (
                            attribute
                                .name()
                                .namespace()
                                .uri()
                                .map(|s| s.code_units().to_vec()),
                            attribute.name().prefix().map(|s| s.code_units().to_vec()),
                            attribute.name().local_name().code_units().to_vec(),
                            attribute.value().code_units().to_vec(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )),
            HtmlNodeKind::Text(text) => texts.push((id, text.code_units().to_vec())),
            _ => {}
        }
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "select"),
            (3, "button"),
            (4, "selectedcontent"),
            (5, "svg"),
            (6, "linearGradient"),
            (5, "math"),
            (6, "mtext"),
            (7, "#M"),
            (3, "option"),
            (4, "svg"),
            (5, "linearGradient"),
            (4, "math"),
            (5, "mtext"),
            (6, "#M"),
            (2, "#Z"),
        ])
    );
    let units = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    let html = "http://www.w3.org/1999/xhtml";
    let svg = "http://www.w3.org/2000/svg";
    let math = "http://www.w3.org/1998/Math/MathML";
    let xlink = "http://www.w3.org/1999/xlink";
    let xml = "http://www.w3.org/XML/1998/namespace";
    let rows = [
        (html, "html", vec![]),
        (html, "head", vec![]),
        (html, "body", vec![]),
        (html, "select", vec![(None, None, "name", "s")]),
        (html, "button", vec![]),
        (html, "selectedcontent", vec![]),
        (svg, "svg", vec![(None, None, "viewBox", "0 0 1 1")]),
        (
            svg,
            "linearGradient",
            vec![
                (None, None, "id", "g"),
                (Some(xlink), Some("xlink"), "href", "#x"),
            ],
        ),
        (math, "math", vec![(None, None, "definitionURL", "u")]),
        (math, "mtext", vec![(Some(xml), Some("xml"), "lang", "en")]),
        (html, "option", vec![(None, None, "selected", "")]),
        (svg, "svg", vec![(None, None, "viewBox", "0 0 1 1")]),
        (
            svg,
            "linearGradient",
            vec![
                (None, None, "id", "g"),
                (Some(xlink), Some("xlink"), "href", "#x"),
            ],
        ),
        (math, "math", vec![(None, None, "definitionURL", "u")]),
        (math, "mtext", vec![(Some(xml), Some("xml"), "lang", "en")]),
    ];
    let expected: Vec<_> = rows
        .into_iter()
        .map(|(namespace, name, attrs)| {
            (
                Some(units(namespace)),
                None::<Vec<u16>>,
                units(name),
                attrs
                    .into_iter()
                    .map(|(ns, prefix, name, value)| {
                        (ns.map(units), prefix.map(units), units(name), units(value))
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let actual: Vec<_> = elements
        .iter()
        .map(|(_, _, ns, prefix, name, attrs)| {
            (ns.clone(), prefix.clone(), name.clone(), attrs.clone())
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(
        texts
            .iter()
            .map(|(_, text)| text.clone())
            .collect::<Vec<_>>(),
        vec![units("M"), units("M"), units("Z")]
    );
    assert_eq!(
        elements[5].1,
        vec![elements[6].0.clone(), elements[8].0.clone()]
    );
    assert_eq!(
        elements[10].1,
        vec![elements[11].0.clone(), elements[13].0.clone()]
    );
    for (cloned, original) in [(6, 11), (7, 12), (8, 13), (9, 14)] {
        assert_ne!(elements[cloned].0, elements[original].0);
    }
    assert!(elements[7].1.is_empty());
    assert!(elements[12].1.is_empty());
    assert_eq!(elements[9].1, vec![texts[0].0.clone()]);
    assert_eq!(elements[14].1, vec![texts[1].0.clone()]);
    assert_ne!(texts[0].0, texts[1].0);
}

#[test]
fn ordinary_template_routes_rows_into_separate_hosted_contents_and_keeps_body_sibling_order() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><template id=t><tr><td>A</td><td>B</td></tr></template><p>Z</p>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let ordinary_outline = outline(document);
    let mut elements = Vec::new();
    let mut ordinary_texts = Vec::new();
    let mut template = None;
    let mut stack: Vec<_> = document.root().children().collect();
    stack.reverse();
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        let children: Vec<_> = node.children().collect();
        stack.extend(children.iter().rev().cloned());
        match node.kind() {
            HtmlNodeKind::Element(element) => {
                let name = element.name();
                elements.push((
                    id.clone(),
                    node.parent(),
                    children,
                    name.namespace()
                        .uri()
                        .map(|value| value.code_units().to_vec()),
                    name.prefix().map(|value| value.code_units().to_vec()),
                    name.local_name().code_units().to_vec(),
                    element
                        .attributes()
                        .iter()
                        .map(|attribute| {
                            (
                                attribute
                                    .name()
                                    .namespace()
                                    .uri()
                                    .map(|value| value.code_units().to_vec()),
                                attribute
                                    .name()
                                    .prefix()
                                    .map(|value| value.code_units().to_vec()),
                                attribute.name().local_name().code_units().to_vec(),
                                attribute.value().code_units().to_vec(),
                            )
                        })
                        .collect::<Vec<_>>(),
                ));
                if name.local_name() == &DocumentString::from("template") {
                    template = Some((id, node.template_contents()));
                }
            }
            HtmlNodeKind::Text(value) => ordinary_texts.push((id, value.code_units().to_vec())),
            _ => {}
        }
    }
    let mut contents = None;
    let mut contents_outline = Vec::new();
    let mut content_elements = Vec::new();
    let mut content_texts = Vec::new();
    if let Some((_, Some(fragment_id))) = &template {
        let fragment = document.node(fragment_id).unwrap();
        let host = match fragment.fragment_owner() {
            Some(crate::HtmlFragmentOwner::Template(id)) => Some(id.clone()),
            _ => None,
        };
        let children: Vec<_> = fragment.children().collect();
        contents = Some((
            fragment_id.clone(),
            matches!(fragment.kind(), HtmlNodeKind::Fragment),
            fragment.parent(),
            host,
            children.clone(),
        ));
        let mut stack: Vec<_> = children.into_iter().map(|id| (0, id)).collect();
        stack.reverse();
        while let Some((depth, id)) = stack.pop() {
            let node = document.node(&id).unwrap();
            let children: Vec<_> = node.children().collect();
            stack.extend(children.iter().rev().cloned().map(|id| (depth + 1, id)));
            match node.kind() {
                HtmlNodeKind::Element(element) => {
                    let name = element.name();
                    contents_outline.push((depth, name.local_name().code_units().to_vec()));
                    content_elements.push((
                        id,
                        node.parent(),
                        children,
                        name.namespace()
                            .uri()
                            .map(|value| value.code_units().to_vec()),
                        name.prefix().map(|value| value.code_units().to_vec()),
                        name.local_name().code_units().to_vec(),
                        element.attributes().len(),
                    ));
                }
                HtmlNodeKind::Text(value) => {
                    let mut label = vec![b'#' as u16];
                    label.extend_from_slice(value.code_units());
                    contents_outline.push((depth, label));
                    content_texts.push((id, node.parent(), value.code_units().to_vec()));
                }
                _ => {}
            }
        }
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);

    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        ordinary_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (2, "template"),
            (1, "body"),
            (2, "p"),
            (3, "#Z"),
        ])
    );
    assert_eq!(
        contents_outline,
        expected_outline(&[(0, "tr"), (1, "td"), (2, "#A"), (1, "td"), (2, "#B"),])
    );
    let units = |value: &str| value.encode_utf16().collect::<Vec<_>>();
    assert_eq!(elements.len(), 5);
    let expected_elements: Vec<_> = ["html", "head", "template", "body", "p"]
        .into_iter()
        .map(|name| {
            (
                Some(units("http://www.w3.org/1999/xhtml")),
                None::<Vec<u16>>,
                units(name),
                if name == "template" {
                    vec![(None, None, units("id"), units("t"))]
                } else {
                    vec![]
                },
            )
        })
        .collect();
    assert_eq!(
        elements
            .iter()
            .map(|entry| (
                entry.3.clone(),
                entry.4.clone(),
                entry.5.clone(),
                entry.6.clone()
            ))
            .collect::<Vec<_>>(),
        expected_elements
    );
    let template = template.unwrap();
    let contents = contents.unwrap();
    assert_eq!(
        elements[0].2,
        vec![elements[1].0.clone(), elements[3].0.clone()]
    );
    assert_eq!(elements[1].2, vec![template.0.clone()]);
    assert_eq!(elements[2].0, template.0);
    assert!(elements[2].2.is_empty());
    assert_eq!(elements[2].1, Some(elements[1].0.clone()));
    assert_eq!(template.1, Some(contents.0.clone()));
    assert!(contents.1);
    assert!(contents.2.is_none());
    assert_eq!(contents.3, Some(template.0.clone()));
    assert_eq!(content_elements.len(), 3);
    assert_eq!(contents.4, vec![content_elements[0].0.clone()]);
    assert_eq!(content_elements[0].1, Some(contents.0.clone()));
    assert_eq!(
        content_elements[0].2,
        vec![content_elements[1].0.clone(), content_elements[2].0.clone()]
    );
    assert_eq!(content_elements[1].1, Some(content_elements[0].0.clone()));
    assert_eq!(content_elements[2].1, Some(content_elements[0].0.clone()));
    assert_eq!(
        content_elements
            .iter()
            .map(|entry| (entry.3.clone(), entry.4.clone(), entry.5.clone(), entry.6))
            .collect::<Vec<_>>(),
        ["tr", "td", "td"]
            .into_iter()
            .map(|name| (
                Some(units("http://www.w3.org/1999/xhtml")),
                None::<Vec<u16>>,
                units(name),
                0
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(content_texts.len(), 2);
    assert_eq!(content_elements[1].2, vec![content_texts[0].0.clone()]);
    assert_eq!(content_elements[2].2, vec![content_texts[1].0.clone()]);
    assert_eq!(content_texts[0].1, Some(content_elements[1].0.clone()));
    assert_eq!(content_texts[1].1, Some(content_elements[2].0.clone()));
    assert_eq!(content_texts[0].2, units("A"));
    assert_eq!(content_texts[1].2, units("B"));
    assert_eq!(ordinary_texts.len(), 1);
    assert_eq!(ordinary_texts[0].1, units("Z"));
    assert_eq!(elements[3].2, vec![elements[4].0.clone()]);
    assert_eq!(elements[4].2, vec![ordinary_texts[0].0.clone()]);
    let ids: Vec<_> = elements
        .iter()
        .map(|entry| &entry.0)
        .chain(std::iter::once(&contents.0))
        .chain(content_elements.iter().map(|entry| &entry.0))
        .chain(content_texts.iter().map(|entry| &entry.0))
        .chain(ordinary_texts.iter().map(|entry| &entry.0))
        .collect();
    for (index, id) in ids.iter().enumerate() {
        for other in &ids[index + 1..] {
            assert_ne!(id, other);
        }
    }
}

#[test]
fn template_nested_contents_reuse_one_real_inert_document_and_preserve_ordinary_documents() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<template id=a><div>A<template id=b><p>B</p></template>C</div></template><p>Z</p>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    // Obtain the qualified active subjects from the actual open stack.
    let outer = loop {
        driver.step().unwrap();
        if let Some(head) = &driver.head {
            let kernel = &head.state.as_ref().unwrap().before.kernel;
            let node = kernel
                .at(kernel.stack().unwrap().len() - 1, &mut driver.control)
                .unwrap();
            if kernel
                .named(&node, "template", &mut driver.control)
                .unwrap()
            {
                break node;
            }
        }
    };
    let (c, session, _) = driver.standalone_parts().unwrap();
    let state: crate::construction::TemplateState = session.template_state(c, &outer).unwrap();
    assert!(state.target.is_none() && state._start.is_none() && state.end.is_none());
    let ordinary: crate::construction::ParserDocument = session.node_document(c, &outer).unwrap();
    let ordinary = ordinary.node(c, session).unwrap();
    let inert = session
        .node_document(c, &state.contents)
        .unwrap()
        .node(c, session)
        .unwrap();
    assert!(!session.same_node(c, &ordinary, &inert).unwrap());
    let root = session.root(c).unwrap();
    assert!(session.same_node(c, &ordinary, &root).unwrap());
    drop(root);
    drop((outer, state, ordinary, inert));
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    assert_eq!(
        outline(document),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (2, "template"),
            (1, "body"),
            (2, "p"),
            (3, "#Z")
        ])
    );
    let root = document.root().id();
    let mut pending: Vec<_> = document.root().children().collect();
    let mut templates = Vec::new();
    let mut texts = Vec::new();
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        pending.extend(node.children());
        if let Some(contents) = node.template_contents() {
            assert!(node.children().next().is_none());
            templates.push((
                id.clone(),
                contents.clone(),
                frozen.node_document(&mut c, &id).unwrap(),
                frozen.node_document(&mut c, &contents).unwrap(),
            ));
            pending.extend(document.node(&contents).unwrap().children());
        }
        if let crate::HtmlNodeKind::Text(text) = node.kind() {
            texts.push(text.code_units().to_vec());
        }
    }
    assert_eq!(templates.len(), 2);
    assert_eq!(templates[0].2, root);
    assert_eq!(templates[1].2, templates[0].3);
    assert_eq!(templates[1].3, templates[0].3);
    assert_ne!(templates[0].0, templates[1].0);
    assert_ne!(templates[0].1, templates[1].1);
    assert_eq!(
        document.node(&templates[0].3).unwrap().kind(),
        &crate::HtmlNodeKind::Document(crate::HtmlDocumentMode::NoQuirks)
    );
    texts.sort();
    assert_eq!(texts, vec![vec![65], vec![66], vec![67], vec![90]]);
    drop((pending, templates, texts, root));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn template_pi_lookup_uses_whole_xml_grammar_without_html_character_reference_recovery() {
    for (data, expected) in [
        ("name='A&amp;&#x80;&#13;'", Some(vec![65, 38, 128, 13])),
        ("name='ok' x='bad&unknown;'", None),
        ("name='a' name='b'", None),
        ("name='a'\u{c}x='b'", None),
        ("Name='a'", None),
        ("name='&#x0;'", None),
        ("name='&lt;&gt;&quot;&apos;'", Some(vec![60, 62, 34, 39])),
    ] {
        let source = format!("<body><?marker {data}?><p>Z</p>");
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let pi = document.node(&body).unwrap().children().next().unwrap();
        let actual = frozen
            .processing_instruction_attribute(&mut c, &pi, &DocumentString::from("name"))
            .unwrap()
            .map(|s| s.code_units().to_vec());
        drop((pi, body, html));
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected, "{data}");
    }
}

#[test]
fn template_patch_removes_original_range_and_inserts_before_attached_end_marker() {
    for (source, expected) in [
        (
            "<body><div><?start name='x'?><p>old</p><?end?><span>K</span></div><template for=x><p>N</p></template><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "div"),
                (3, "p"),
                (4, "#N"),
                (3, "span"),
                (4, "#K"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
        (
            "<body><div><?marker name='x'?><span>K</span></div><template for=x><p>N</p></template><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "div"),
                (3, "p"),
                (4, "#N"),
                (3, "span"),
                (4, "#K"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
        (
            "<body><div><?start name='x'?><p>old</p></div><template for=x><p>N</p></template><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "div"),
                (3, "p"),
                (4, "#N"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
        (
            "<body><div><?start name='x'?><?start name='other'?><p>A</p><?end?><p>B</p><?end?><span>K</span></div><template for=x><p>N</p></template><p>Z</p>",
            vec![
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "div"),
                (3, "p"),
                (4, "#N"),
                (3, "span"),
                (4, "#K"),
                (2, "p"),
                (3, "#Z"),
            ],
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let actual = outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected_outline(&expected), "{source}");
    }
}

#[test]
fn template_form_pointer_survives_nested_form_recovery_and_template_parser_association_is_suppressed()
 {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<form id=f><template><form id=g><img></form></template><img></form>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    assert_eq!(
        outline(document),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "template"),
            (3, "img")
        ])
    );
    let html = document.root().children().next().unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let template = document.node(&form).unwrap().children().next().unwrap();
    let img = document.node(&form).unwrap().children().nth(1).unwrap();
    let contents = document
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap();
    let inner = document.node(&contents).unwrap().children().next().unwrap();
    let inner_img = document.node(&inner).unwrap().children().next().unwrap();
    assert_eq!(
        frozen.form_association_owner(&mut c, &img).unwrap(),
        Some(form.clone())
    );
    assert!(
        frozen
            .form_association_state(&mut c, &img)
            .unwrap()
            .parser_inserted
    );
    assert_eq!(
        frozen.form_association_owner(&mut c, &inner_img).unwrap(),
        Some(inner.clone())
    );
    assert!(
        !frozen
            .form_association_state(&mut c, &inner_img)
            .unwrap()
            .parser_inserted
    );
    drop((html, body, form, template, img, contents, inner, inner_img));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn template_selectedcontent_deep_clone_keeps_distinct_contents_and_adopts_real_documents() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<select><button><selectedcontent></selectedcontent></button><option><template id=t><i>A</i><template id=n><svg><desc>B</desc></svg></template><?marker name='x'?></template></option></select>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    assert_eq!(
        outline(document),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "select"),
            (3, "button"),
            (4, "selectedcontent"),
            (5, "template"),
            (3, "option"),
            (4, "template")
        ])
    );
    let mut pending: Vec<_> = document.root().children().collect();
    pending.reverse();
    let mut templates = Vec::new();
    let mut text = Vec::new();
    let mut italic = Vec::new();
    let mut pi = Vec::new();
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        if let HtmlNodeKind::Element(element) = node.kind()
            && element.name().local_name() == &DocumentString::from("i")
        {
            italic.push(id.clone());
        }
        if let HtmlNodeKind::Text(value) = node.kind() {
            text.push((id.clone(), value.code_units().to_vec()));
        }
        if matches!(node.kind(), HtmlNodeKind::ProcessingInstruction(_)) {
            pi.push(id.clone());
        }
        let mut children: Vec<_> = node.children().collect();
        if let Some(contents) = node.template_contents() {
            assert!(children.is_empty());
            templates.push((
                id.clone(),
                contents.clone(),
                frozen.node_document(&mut c, &id).unwrap(),
                frozen.node_document(&mut c, &contents).unwrap(),
            ));
            children.extend(document.node(&contents).unwrap().children());
        }
        pending.extend(children.into_iter().rev());
    }
    assert_eq!(templates.len(), 4);
    assert_eq!(italic.len(), 2);
    assert_eq!(text.len(), 4);
    assert_eq!(pi.len(), 2);
    assert_eq!(
        text.iter().map(|entry| entry.1.clone()).collect::<Vec<_>>(),
        vec![vec![65], vec![66], vec![65], vec![66]]
    );
    for (left, right) in [(0, 2), (1, 3)] {
        assert_ne!(templates[left].0, templates[right].0);
        assert_ne!(templates[left].1, templates[right].1);
        assert_eq!(templates[left].2, templates[right].2);
        assert_eq!(templates[left].3, templates[right].3);
    }
    assert_ne!(italic[0], italic[1]);
    assert_ne!(text[0].0, text[2].0);
    assert_ne!(text[1].0, text[3].0);
    assert_ne!(pi[0], pi[1]);
    for id in &pi {
        assert_eq!(
            frozen
                .processing_instruction_attribute(&mut c, id, &DocumentString::from("name"))
                .unwrap()
                .unwrap()
                .code_units(),
            [120]
        );
    }
    drop((pending, templates, text, italic, pi));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn template_pi_xml_unicode_names_values_and_invalid_whole_maps_preserve_native_utf16() {
    for (data, key, expected) in [
        ("名='&#x1F600;'", "名", Some(vec![0xd83d, 0xde00])),
        ("\u{10000}='A'", "\u{10000}", Some(vec![65])),
        ("name='&#X41;'", "name", None),
        ("name='&#xD800;'", "name", None),
        ("name='A' x='<'", "name", None),
        ("name='A' x='\u{ffff}'", "name", None),
        ("name='A' x='&AMP;'", "name", None),
        ("name='A'\r\nx='B'", "name", Some(vec![65])),
    ] {
        let source = format!("<body><?marker {data}?><p>Z</p>");
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(DocumentString::from_utf16(
                source.encode_utf16().collect::<Vec<_>>(),
            )),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let pi = document.node(&body).unwrap().children().next().unwrap();
        let actual = frozen
            .processing_instruction_attribute(&mut c, &pi, &DocumentString::from(key))
            .unwrap()
            .map(|s| s.code_units().to_vec());
        assert!(matches!(
            document.node(&pi).unwrap().kind(),
            HtmlNodeKind::ProcessingInstruction(_)
        ));
        drop((pi, body, html));
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected, "{data}");
    }
}

#[test]
fn template_valid_shadow_mode_returns_to_plain_contents_before_for_patching() {
    for mode in ["open", "ClOsEd"] {
        let source = format!(
            "<body><div><?marker name='x'?></div><template for=x shadowrootmode={mode}><p>N</p></template><p>Z</p>"
        );
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let children: Vec<_> = document.node(&body).unwrap().children().collect();
        assert_eq!(children.len(), 3);
        assert!(matches!(
            document
                .node(&children[0])
                .unwrap()
                .children()
                .next()
                .map(|id| document.node(&id).unwrap().kind().clone()),
            Some(HtmlNodeKind::ProcessingInstruction(_))
        ));
        let template = document.node(&children[1]).unwrap();
        assert!(template.children().next().is_none());
        assert!(template.declarative_shadow_root().is_none());
        let contents = template.template_contents().unwrap();
        let p = document.node(&contents).unwrap().children().next().unwrap();
        let text = document.node(&p).unwrap().children().next().unwrap();
        assert_eq!(
            document.node(&text).unwrap().kind(),
            &HtmlNodeKind::Text(DocumentString::from("N"))
        );
        drop((html, body, children, contents, p, text));
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn template_modes_reprocess_table_families_and_head_text_into_real_contents() {
    for (source, expected, markers) in [
        ("<template><col></template>", vec![(0, "col")], 0),
        (
            "<template><td>A<th>B</template>",
            vec![(0, "td"), (1, "#A"), (0, "th"), (1, "#B")],
            1,
        ),
        (
            "<template><tbody><tr><td>A<th>B</template>",
            vec![
                (0, "tbody"),
                (1, "tr"),
                (2, "td"),
                (3, "#A"),
                (2, "th"),
                (3, "#B"),
            ],
            1,
        ),
        (
            "<template><caption>A</caption></template>",
            vec![(0, "caption"), (1, "#A")],
            0,
        ),
        (
            "<template><style>x</style><!--C--><?marker name='x'?><p>A</p></template>",
            vec![
                (0, "style"),
                (1, "#x"),
                (0, "!C"),
                (0, "?marker"),
                (0, "p"),
                (1, "#A"),
            ],
            0,
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.template_modes.len(), 0);
        // Thorough implied pops do not execute cell-end-marker cleanup;
        // end-template clears only through the last actual marker.
        assert_eq!(state.formatting.len(), markers);
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document.root().children().next().unwrap();
        let head = document.node(&html).unwrap().children().next().unwrap();
        let template = document.node(&head).unwrap().children().next().unwrap();
        let contents = document
            .node(&template)
            .unwrap()
            .template_contents()
            .unwrap();
        let mut pending: Vec<_> = document
            .node(&contents)
            .unwrap()
            .children()
            .map(|id| (0, id))
            .collect();
        pending.reverse();
        let mut actual = Vec::new();
        while let Some((depth, id)) = pending.pop() {
            let node = document.node(&id).unwrap();
            let value = match node.kind() {
                HtmlNodeKind::Element(e) => e.name().local_name().code_units().to_vec(),
                HtmlNodeKind::Text(t) => [vec![35], t.code_units().to_vec()].concat(),
                HtmlNodeKind::Comment(t) => [vec![33], t.code_units().to_vec()].concat(),
                HtmlNodeKind::ProcessingInstruction(p) => {
                    [vec![63], p.target().code_units().to_vec()].concat()
                }
                _ => panic!("literal child kinds"),
            };
            actual.push((depth, value));
            let children: Vec<_> = node.children().map(|id| (depth + 1, id)).collect();
            pending.extend(children.into_iter().rev());
        }
        drop((html, head, template, contents, pending));
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(actual, expected_outline(&expected), "{source}");
    }
}

#[test]
fn template_eof_unwinds_both_mode_markers_but_does_not_remove_patch_markers() {
    let source = "<body><div><?marker name='x'?></div><template for=x><p>N";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.template_modes.len(), 0);
    assert_eq!(state.formatting.len(), 0);
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
            (2, "div"),
            (3, "p"),
            (4, "#N"),
            (3, "?marker")
        ])
    );
}

#[test]
fn ordinary_svg_script_foreign_text_and_self_closing_preserve_native_siblings() {
    // HTML877d foreign insertion/character/end rules and the selected ordinary
    // offline SVG processing specialization; this observes real native results.
    for (source, script_text) in [
        (
            "<!doctype html><svg><script xlink:href='offline.js'><![CDATA[A<&]]></script><g/></svg><p>after</p>",
            Some("A<&"),
        ),
        (
            "<!doctype html><svg><script xlink:href='offline.js'/><g/></svg><p>after</p>",
            None,
        ),
        (
            "<!doctype html><svg><script xlink:href='offline.js'>A&amp;B</script><g/></svg><p>after</p>",
            Some("A&B"),
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let boundary = standalone_boundary(&mut driver);
        if boundary != super::head::HeadStep::Complete {
            // Dispose the actual parser/tree/controller owners before asserting
            // the source-required continuation beyond the unsupported branch.
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(boundary, super::head::HeadStep::Complete, "{source}");
            return;
        }

        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let actual_mode = document.mode();
        let actual_outline = outline(document);
        let root_child_count = document.root().children().count();
        let mut stack: Vec<_> = document.root().children().collect();
        stack.reverse();
        let mut elements = Vec::new();
        let mut texts = Vec::new();
        while let Some(id) = stack.pop() {
            let node = document.node(&id).unwrap();
            let children: Vec<_> = node.children().collect();
            stack.extend(children.iter().rev().cloned());
            match node.kind() {
                HtmlNodeKind::Element(element) => elements.push((
                    id,
                    children,
                    element
                        .name()
                        .namespace()
                        .uri()
                        .map(|uri| uri.code_units().to_vec()),
                    element
                        .name()
                        .prefix()
                        .map(|prefix| prefix.code_units().to_vec()),
                    element.name().local_name().code_units().to_vec(),
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
                                attribute
                                    .name()
                                    .prefix()
                                    .map(|prefix| prefix.code_units().to_vec()),
                                attribute.name().local_name().code_units().to_vec(),
                                attribute.value().code_units().to_vec(),
                            )
                        })
                        .collect::<Vec<_>>(),
                )),
                HtmlNodeKind::Text(text) => texts.push((id, text.code_units().to_vec())),
                _ => {}
            }
        }
        // Only copied units and result-qualified identities outlive native
        // backing. Release all actual storage before the remaining assertions.
        frozen.release(&mut c).unwrap();
        assert_released(&c);

        assert_eq!(boundary, super::head::HeadStep::Complete, "{source}");
        assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks, "{source}");
        assert_eq!(root_child_count, 2, "{source}");
        let mut rows = vec![
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "svg"),
            (3, "script"),
        ];
        if let Some(text) = script_text {
            rows.push((
                4,
                match text {
                    "A<&" => "#A<&",
                    "A&B" => "#A&B",
                    _ => unreachable!("only independent literal inputs above"),
                },
            ));
        }
        rows.extend([(3, "g"), (2, "p"), (3, "#after")]);
        assert_eq!(actual_outline, expected_outline(&rows), "{source}");

        let units = |value: &str| value.encode_utf16().collect::<Vec<_>>();
        let actual_elements: Vec<_> = elements
            .iter()
            .map(|(_, _, namespace, prefix, name, attributes)| {
                (
                    namespace.clone(),
                    prefix.clone(),
                    name.clone(),
                    attributes.clone(),
                )
            })
            .collect();
        let html = "http://www.w3.org/1999/xhtml";
        let svg = "http://www.w3.org/2000/svg";
        let expected_elements: Vec<_> = [
            (html, "html", vec![]),
            (html, "head", vec![]),
            (html, "body", vec![]),
            (svg, "svg", vec![]),
            (
                svg,
                "script",
                vec![(
                    Some("http://www.w3.org/1999/xlink"),
                    Some("xlink"),
                    "href",
                    "offline.js",
                )],
            ),
            (svg, "g", vec![]),
            (html, "p", vec![]),
        ]
        .into_iter()
        .map(|(namespace, name, attributes)| {
            (
                Some(units(namespace)),
                None::<Vec<u16>>,
                units(name),
                attributes
                    .into_iter()
                    .map(|(namespace, prefix, name, value)| {
                        (
                            namespace.map(units),
                            prefix.map(units),
                            units(name),
                            units(value),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
        assert_eq!(actual_elements, expected_elements, "{source}");

        let expected_texts: Vec<_> = script_text
            .into_iter()
            .chain(std::iter::once("after"))
            .map(units)
            .collect();
        assert_eq!(
            texts
                .iter()
                .map(|(_, text)| text.clone())
                .collect::<Vec<_>>(),
            expected_texts,
            "{source}"
        );
        assert_eq!(
            elements
                .iter()
                .map(|(_, children, ..)| children.len())
                .collect::<Vec<_>>(),
            vec![2, 0, 2, 2, usize::from(script_text.is_some()), 0, 1],
            "{source}"
        );
        assert_eq!(
            elements[2].1,
            vec![elements[3].0.clone(), elements[6].0.clone()],
            "{source}"
        );
        assert_eq!(
            elements[3].1,
            vec![elements[4].0.clone(), elements[5].0.clone()],
            "{source}"
        );
        if script_text.is_some() {
            assert_eq!(elements[4].1, vec![texts[0].0.clone()], "{source}");
            assert_ne!(texts[0].0, texts[1].0, "{source}");
        } else {
            assert!(elements[4].1.is_empty(), "{source}");
        }
        assert_eq!(
            elements[6].1,
            vec![texts.last().unwrap().0.clone()],
            "{source}"
        );
        for (position, (id, ..)) in elements.iter().enumerate() {
            for (other, ..) in elements.iter().skip(position + 1) {
                assert_ne!(id, other, "{source}");
            }
        }
    }
}

pub(super) fn svg_open_node(driver: &mut StandaloneDocumentDriver<'_>) -> ParserNode {
    for _ in 0..1_000 {
        driver.step().unwrap();
        let Some(head) = &driver.head else { continue };
        let kernel = &head.state.as_ref().unwrap().before.kernel;
        let Some(last) = kernel.stack().unwrap().len().checked_sub(1) else {
            continue;
        };
        let node = kernel.at(last, &mut driver.control).unwrap();
        if kernel
            .session
            .element_name(&mut driver.control, &node)
            .unwrap()
            .is_some_and(|name| {
                name.namespace() == &Namespace::svg()
                    && name.local_name() == &DocumentString::from("script")
            })
        {
            assert_ne!(
                head.state.as_ref().unwrap().mode,
                super::head::InsertionMode::Text
            );
            return node;
        }
    }
    panic!("real foreign SVG script start");
}
#[test]
fn svg_script_parser_end_defers_then_restores_wrapper_without_html_preparation() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><svg><script xlink:href='offline.js'></script><g/></svg><p>Z</p>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let node = svg_open_node(&mut driver);
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(
        !session
            .svg_script_state(c, &node)
            .unwrap()
            .flags
            .already_processed()
    );
    assert_eq!(
        session.script_metadata(c, &node).err(),
        Some(SessionError::ScriptMetadataUnavailable)
    );
    driver.step().unwrap();
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(
        session
            .svg_script_state(c, &node)
            .unwrap()
            .flags
            .already_processed()
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
    assert!(
        driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .svg_pending
            .is_none()
    );
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}
fn svg_end_cost() -> u64 {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><svg><script></script>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let _node = svg_open_node(&mut driver);
    let before = driver
        .control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    driver.step().unwrap();
    let cost = driver
        .control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    driver.release().unwrap();
    assert_released(&driver.control);
    cost
}
#[test]
fn svg_script_every_foreign_end_work_cut_including_flag_and_restore_has_terminal_cleanup() {
    let cost = svg_end_cost();
    assert!(cost > 100);
    for allowance in 0..cost {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><svg><script></script>"),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let _node = svg_open_node(&mut driver);
        let used = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        driver
            .control
            .charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - allowance,
            )
            .unwrap();
        assert!(driver.step().is_err(), "cut {allowance}/{cost}");
        let stop = driver.control.stop().unwrap();
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        let reads = driver.control.usage().work(WorkCategory::InputUnitRead);
        assert!(matches!(driver.step(),Err(StandaloneError::Stop(s)) if s==stop));
        assert_eq!(
            driver.control.usage().work(WorkCategory::InputUnitRead),
            reads
        );
        let work = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.stop(), Some(stop));
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
    }
}
#[test]
fn svg_script_cancelled_foreign_end_keeps_false_and_original_first_stop() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<svg><script></script>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let _node = svg_open_node(&mut driver);
    request.cancellation().cancel();
    assert!(matches!(
        driver.step(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
}
#[test]
fn svg_script_inert_template_and_actual_patch_connection_have_distinct_processed_states() {
    for (source, processed) in [
        (
            "<body><template><svg><script>A</script></svg></template><p>Z</p>",
            false,
        ),
        (
            "<body><div><?marker name='x'?></div><template for=x><svg><script>A</script></svg></template><p>Z</p>",
            true,
        ),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let mut pending = vec![document.root().id()];
        let mut scripts = vec![];
        while let Some(id) = pending.pop() {
            let node = document.node(&id).unwrap();
            if matches!(node.kind(),HtmlNodeKind::Element(e) if e.name().namespace()==&Namespace::svg()
                && e.name().local_name()==&DocumentString::from("script"))
            {
                scripts.push(id.clone());
            }
            pending.extend(node.children());
            if let Some(contents) = node.template_contents() {
                pending.push(contents);
            }
        }
        assert_eq!(scripts.len(), 1);
        assert_eq!(
            frozen
                .svg_script_state(&mut c, &scripts[0])
                .unwrap()
                .flags
                .already_processed(),
            processed
        );
        drop((pending, scripts));
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}
#[test]
fn svg_script_foreign_end_does_not_drain_pending_select_tasks_as_an_html_checkpoint() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><select><option><div><option selected>A</option></div><div><option selected>B</option></div></option><selectedcontent></selectedcontent></select><svg><script></script></svg>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let script = svg_open_node(&mut driver);
    assert_eq!(driver.step().unwrap(), super::head::HeadStep::Consumed);
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(
        session
            .svg_script_state(c, &script)
            .unwrap()
            .flags
            .already_processed()
    );
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert_eq!(
        session.freeze(c).err(),
        Some(SessionError::PendingIntrinsicWork)
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn svg_script_selectedcontent_deep_clone_preserves_true_bits_and_inert_nested_false_bits() {
    use crate::construction::SvgScriptOrigin;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected><svg><script>A</script><script xlink:href='#x'/><foreignObject><template><svg><script>B</script></svg></template></foreignObject></svg></option></select>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mut pending = vec![(document.root().id(), false)];
    let mut states = vec![];
    let mut ids = vec![];
    while let Some((id, inert)) = pending.pop() {
        let node = document.node(&id).unwrap();
        if matches!(node.kind(),HtmlNodeKind::Element(e) if e.name().namespace()==&Namespace::svg()
            && e.name().local_name()==&DocumentString::from("script"))
        {
            let state = frozen.svg_script_state(&mut c, &id).unwrap();
            assert_eq!(state.flags.already_processed(), !inert);
            states.push((state.origin, inert));
            ids.push(id.clone());
        }
        pending.extend(node.children().map(|id| (id, inert)));
        if let Some(contents) = node.template_contents() {
            pending.push((contents, true));
        }
    }
    assert_eq!(states.len(), 6);
    for origin in [SvgScriptOrigin::Parser, SvgScriptOrigin::Clone] {
        assert_eq!(
            states
                .iter()
                .filter(|&&(o, inert)| o == origin && !inert)
                .count(),
            2
        );
        assert_eq!(
            states
                .iter()
                .filter(|&&(o, inert)| o == origin && inert)
                .count(),
            1
        );
    }
    for (i, id) in ids.iter().enumerate() {
        assert!(!ids[i + 1..].contains(id));
    }
    drop((states, ids, pending));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn svg_script_actual_slot_layout_and_stack_caps_have_exact_and_one_short_terminal_boundaries() {
    let source = "<!doctype html><svg><script xlink:href='#saved'/></svg>";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    // Native allocator/layout observations qualify physical boundaries, not a
    // semantic oracle. Each subsequent cap admits the same real producer sequence.
    let auxiliary = driver
        .usage()
        .resource(HtmlParseResource::AuxiliaryEntries)
        .high_water;
    let atomic = driver.usage().largest_atomic_layout;
    driver.release().unwrap();
    assert_released(&driver.control);
    for (resource, exact) in [
        (HtmlParseResource::AuxiliaryEntries, auxiliary),
        (HtmlParseResource::AtomicLayoutBytes, atomic),
        (HtmlParseResource::OpenElements, 4),
    ] {
        for short in [false, true] {
            let cap = exact - u64::from(short);
            let builder = limits(100).into_builder();
            let caps = match resource {
                HtmlParseResource::AuxiliaryEntries => builder.auxiliary_entries(cap),
                HtmlParseResource::AtomicLayoutBytes => builder.atomic_layout_bytes(cap),
                HtmlParseResource::OpenElements => builder.open_elements(cap),
                _ => unreachable!(),
            }
            .finish()
            .unwrap();
            let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), caps);
            let mut entry = super::entry::OrdinaryDocumentEntry::new(&request);
            let mut driver = match entry.take_driver() {
                Ok(driver) => driver,
                Err(error) => {
                    // Initial fixed layouts can dominate the same aggregate cap;
                    // no fictitious later flag-only layout boundary is asserted.
                    assert!(
                        short && resource == HtmlParseResource::AtomicLayoutBytes,
                        "{error:?}"
                    );
                    entry.release_failure().unwrap();
                    continue;
                }
            };
            let mut reached = false;
            for _ in 0..1_000 {
                match driver.step() {
                    Ok(super::head::HeadStep::Complete) => {
                        reached = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
            assert_eq!(reached, !short, "{resource:?}, cap {cap}");
            if short {
                assert!(
                    matches!(driver.control.stop(),Some(ParseStop::LimitExceeded{resource:r,..}) if r==resource)
                );
            }
            driver.release().unwrap();
            assert_released(&driver.control);
        }
    }
}

#[test]
fn ordinary_eof_pops_open_option_and_clones_selected_content_before_completion() {
    // HTML §13.2.7 pops every open element. The option pop hook updates
    // descendant selectedcontent elements using distinct clones of its children.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let mut reached_endpoint = false;
    for _ in 0..1_000 {
        match driver.step() {
            Ok(super::head::HeadStep::Consumed) => {}
            Ok(_) => {
                reached_endpoint = true;
                break;
            }
            Err(error) => {
                driver.release().unwrap();
                assert_released(&driver.control);
                panic!("ordinary source processing failed: {error:?}");
            }
        }
    }
    if !reached_endpoint {
        driver.release().unwrap();
        assert_released(&driver.control);
        panic!("finite ordinary source did not reach its actual endpoint");
    }

    // This existing fixture observer only transfers/finalizes native storage;
    // the actual driver steps above must perform the source-defined pop effect.
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    let mut stack: Vec<_> = document.root().children().collect();
    stack.reverse();
    let mut elements = Vec::new();
    let mut texts = Vec::new();
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        let children: Vec<_> = node.children().collect();
        stack.extend(children.iter().rev().cloned());
        match node.kind() {
            HtmlNodeKind::Element(element) => elements.push((
                id,
                element
                    .name()
                    .namespace()
                    .uri()
                    .map(|uri| uri.code_units().to_vec()),
                element.name().local_name().code_units().to_vec(),
                children,
            )),
            HtmlNodeKind::Text(text) => texts.push((id, text.code_units().to_vec())),
            _ => {}
        }
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);

    // Release every actual parser/native owner before the behavior assertions.
    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    let units = |value: &str| value.encode_utf16().collect::<Vec<_>>();
    assert_eq!(
        elements
            .iter()
            .map(|(_, namespace, name, _)| (namespace.clone(), name.clone()))
            .collect::<Vec<_>>(),
        [
            "html",
            "head",
            "body",
            "select",
            "button",
            "selectedcontent",
            "option"
        ]
        .map(|name| (Some(units("http://www.w3.org/1999/xhtml")), units(name)))
    );
    let original_children = &elements[6].3;
    assert_eq!(original_children.len(), 1);
    let original_text = texts
        .iter()
        .find(|(id, _)| id == &original_children[0])
        .expect("actual original option text");
    assert_eq!(original_text.1, units("A"));

    let cloned_children = &elements[5].3;
    assert_eq!(
        cloned_children.len(),
        1,
        "EOF must pop the open selected option and clone its text into selectedcontent"
    );
    let cloned_text = texts
        .iter()
        .find(|(id, _)| id == &cloned_children[0])
        .expect("actual cloned selectedcontent text");
    assert_eq!(cloned_text.1, units("A"));
    assert_ne!(cloned_text.0, original_text.0);
    assert_eq!(texts.len(), 2);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "select"),
            (3, "button"),
            (4, "selectedcontent"),
            (5, "#A"),
            (3, "option"),
            (4, "#A"),
        ])
    );
}

#[test]
fn in_table_form_pointer_copy_fee_covers_the_actual_retained_receiver() {
    use super::head::{FormPointer, HeadCondition, HeadStep, InsertionMode};
    use crate::construction::{evidence::DiagnosticRule, provenance::ParserRule};

    fn ready<'a>(request: &'a crate::HtmlDocumentParseRequest) -> StandaloneDocumentDriver<'a> {
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(request);
        for _ in 0..100 {
            driver.step().unwrap();
            if driver
                .head
                .as_ref()
                .is_some_and(|head| head.state.as_ref().unwrap().mode == InsertionMode::InTable)
            {
                break;
            }
        }
        let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
        assert_eq!(state.mode, InsertionMode::InTable);
        assert!(state.current_form.is_none());
        assert!(state.pending.is_none());
        let evidence = state
            .before
            .kernel
            .session
            .evidence(&mut driver.control)
            .unwrap();
        let crate::parser::tokenizer::TokenizerPull::Event(event) = state
            .tokenizer
            .as_mut()
            .unwrap()
            .next(&mut driver.control, false, evidence)
            .unwrap()
        else {
            panic!("complete fixture requires actual event");
        };
        let Event::Token(event) = event else {
            panic!("actual form token");
        };
        let token = event.into_token(&mut driver.control).unwrap();
        assert!(matches!(token.kind, TokenKind::Tag(_)));
        write(
            size_of::<Option<super::prologue::PendingToken>>(),
            1,
            &mut driver.control,
        )
        .unwrap();
        state.pending = Some(super::prologue::PendingToken::from_token(token));
        assert_eq!(
            state.token_in(&["form"], &mut driver.control).unwrap(),
            Some("form")
        );
        driver
    }

    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><table><form>"),
        limits(100),
    );
    let mut actual = ready(&request);
    let before = actual.control.usage().clone();
    let state = actual.head.as_mut().unwrap().state.as_mut().unwrap();
    let outcome = state
        .table_rules(
            InsertionMode::InTable,
            request.source(),
            &mut actual.control,
        )
        .unwrap();
    let actual_copy =
        actual.control.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte);
    let actual_writes = actual.control.usage().work(WorkCategory::RecordWrite)
        - before.work(WorkCategory::RecordWrite);
    let pointer_installed = matches!(state.current_form, Some(FormPointer::Owned(_)));
    let actual_stack = state.before.kernel.stack().unwrap().len();

    // The independent common envelope is real trigger retention, one diagnostic,
    // native form creation/insertion, stack removal and pending-token disposal.
    // Tag comparisons add no CopyByte/RecordWrite. The no-template predicate
    // reads the kernel's tracked last template and clones no stack handle.
    // This envelope does not
    // call table_rules or install a form pointer; the two extra source actions
    // are one ParserNode clone and one actual Option<FormPointer> receiver write.
    let mut common = ready(&request);
    let before = common.control.usage().clone();
    let state = common.head.as_mut().unwrap().state.as_mut().unwrap();
    state
        .record_trigger(
            ParserRule::Tree(InsertionMode::InTable),
            &mut common.control,
        )
        .unwrap();
    state
        .emit(
            HeadCondition::UnexpectedTableToken,
            DiagnosticRule {
                anchor: "parsing-main-intable",
                branch: "form-start",
            },
            &mut common.control,
        )
        .unwrap();
    let node = state.insert_tag(&mut common.control).unwrap();
    state
        .before
        .kernel
        .pop(
            &mut common.control,
            state.script.as_ref().map(|script| &script.capability),
        )
        .unwrap();
    let common_outcome = state.finish(&mut common.control).unwrap();
    let common_copy =
        common.control.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte);
    let common_writes = common.control.usage().work(WorkCategory::RecordWrite)
        - before.work(WorkCategory::RecordWrite);
    let common_stack = state.before.kernel.stack().unwrap().len();
    drop(node);

    // Release both actual lexical/native/controller owners before the fee
    // assertion. An underadmitted write must not hide a cleanup failure.
    actual.release().unwrap();
    common.release().unwrap();
    assert_released(&actual.control);
    assert_released(&common.control);
    assert_eq!(outcome, Some(HeadStep::Consumed));
    assert_eq!(common_outcome, HeadStep::Consumed);
    assert!(pointer_installed);
    assert_eq!(actual_stack, common_stack);
    assert_eq!(actual_writes, common_writes + 1);
    assert_eq!(
        actual_copy,
        common_copy + (size_of::<ParserNode>() + size_of::<Option<FormPointer>>()) as u64,
        "the installed receiver needs its actual compiled byte fee: Option<FormPointer>={}, Option<ParserNode>={} bytes",
        size_of::<Option<FormPointer>>(),
        size_of::<Option<ParserNode>>()
    );
}

#[test]
fn ordinary_text_input_keeps_exact_default_value_and_form_siblings_through_eof() {
    // HTML877d: create/append/reset/associate (syntax3511–3539), then
    // InBody input insert/immediate-pop (4654–4676). Value-mode reset strips
    // CR/LF from internal value, not from the native default value attribute.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><form id=f><input value='A&#13;B&#10;C'>Z</form>"),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.stop(), first_stop);
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "ordinary default Text input must be constructed before source EOF"
        );
        return;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_mode = document.mode();
    let actual_outline = outline(document);
    // Select the unique real document element, after the doctype.
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let input = children[0].clone();
    let input_children = document.node(&input).unwrap().children().count();
    let HtmlNodeKind::Element(element) = document.node(&input).unwrap().kind() else {
        panic!("the source input is a real native Element");
    };
    let input_namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let input_name = element.name().local_name().code_units().to_vec();
    let actual_attributes: Vec<_> = element
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
        .collect();
    let HtmlNodeKind::Text(text) = document.node(&children[1]).unwrap().kind() else {
        panic!("Z follows the void input as a sibling in the form");
    };
    let following_text = text.code_units().to_vec();
    let owner = frozen.form_association_owner(&mut control, &input).unwrap();
    let owner_is_form = owner.as_ref() == Some(&form);
    let child_count = children.len();
    drop(owner);
    drop(input);
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.stop(), None);

    assert_eq!(actual_mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 2);
    assert_eq!(input_children, 0);
    assert_eq!(
        input_namespace,
        "http://www.w3.org/1999/xhtml"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(input_name, vec![105, 110, 112, 117, 116]);
    assert_eq!(
        actual_attributes,
        vec![(None, vec![118, 97, 108, 117, 101], vec![65, 13, 66, 10, 67])]
    );
    assert_eq!(following_text, vec![90]);
    assert!(owner_is_form);
}

#[test]
fn ordinary_text_input_selected_types_foster_in_table_keep_form_and_exact_values() {
    for (kind, expected_type) in [
        ("text", surgeist_html::HtmlInputType::Text),
        ("SEARCH", surgeist_html::HtmlInputType::Search),
        ("tel", surgeist_html::HtmlInputType::Telephone),
        ("password", surgeist_html::HtmlInputType::Password),
        ("not-a-type", surgeist_html::HtmlInputType::Text),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!(
                "<!doctype html><form id=f><table><input type={kind} value='A&#13;B&#10;C' checked><tr><td>T</table>Z</form>"
            )),
            limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut c, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        assert_eq!(
            outline(document),
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "form"),
                (3, "input"),
                (3, "table"),
                (4, "tbody"),
                (5, "tr"),
                (6, "td"),
                (7, "#T"),
                (3, "#Z")
            ])
        );
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().last().unwrap();
        let form = document.node(&body).unwrap().children().next().unwrap();
        let input = document.node(&form).unwrap().children().next().unwrap();
        assert_eq!(
            frozen.form_association_owner(&mut c, &input).unwrap(),
            Some(form.clone())
        );
        let state = frozen.input_state(&mut c, &input).unwrap();
        assert_eq!(state.flags.input_type(), expected_type);
        assert_eq!(state.cursor, 0);
        assert!(state.flags.checkedness() && state.selected_files_empty);
        assert!(!state.value_rewritten_during_creation);
        let mut value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
        frozen
            .append_input_internal_value(&mut c, &input, &mut value)
            .unwrap();
        assert_eq!(value.units(), [65, 66, 67]);
        value.clear(&mut c).unwrap();
        frozen
            .append_input_default_value(&mut c, &input, &mut value)
            .unwrap();
        assert_eq!(value.units(), [65, 13, 66, 10, 67]);
        value.release(&mut c).unwrap();
        frozen.release(&mut c).unwrap();
        assert_released(&c);
    }
}

#[test]
fn ordinary_text_input_makes_frameset_not_ok_and_is_void_inside_reconstructed_formatting() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><p><b>A</p><input type=search value=x/>Z<frameset><frame></frameset>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    assert_eq!(
        outline(frozen.document().unwrap()),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "p"),
            (3, "b"),
            (4, "#A"),
            (2, "b"),
            (3, "input"),
            (3, "#Z")
        ])
    );
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ordinary_color_fact_refusal_retains_native_default_after_table_dispatch() {
    assert_color_palette_refusal(
        "<!doctype html><table><input type=color value=currentcolor>",
        super::head::InsertionMode::InTable,
        3,
        7,
        false,
        true,
    );
}

#[test]
fn ordinary_checkbox_keeps_unsanitized_default_and_form_sibling_through_eof() {
    // HTML877d Checkbox defines no value sanitizer. Create/append/reset/associate
    // precedes InBody input insertion/immediate pop; the entity-derived CR/LF
    // therefore remain in this type-first native value attribute.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><input type=checkbox value='A&#13;B&#10;C' checked>Z</form>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.stop(), first_stop);
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "ordinary Checkbox must reach EOF completion with its actual native input and sibling"
        );
        return;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let input = children[0].clone();
    let input_children = document.node(&input).unwrap().children().count();
    let HtmlNodeKind::Element(element) = document.node(&input).unwrap().kind() else {
        panic!("the Checkbox token creates a real native Element");
    };
    let namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let actual_attributes: Vec<_> = element
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
        .collect();
    let owner = frozen.form_association_owner(&mut control, &input).unwrap();
    let owner_is_form = owner.as_ref() == Some(&form);
    let child_count = children.len();
    drop(owner);
    drop(input);
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 2);
    assert_eq!(input_children, 0);
    assert_eq!(
        namespace,
        "http://www.w3.org/1999/xhtml"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        actual_attributes,
        vec![
            (
                None,
                vec![116, 121, 112, 101],
                vec![99, 104, 101, 99, 107, 98, 111, 120]
            ),
            (None, vec![118, 97, 108, 117, 101], vec![65, 13, 66, 10, 67]),
            (None, vec![99, 104, 101, 99, 107, 101, 100], vec![]),
        ]
    );
    assert!(owner_is_form);
}

#[test]
fn ordinary_checkbox_same_name_preserves_both_checked_and_external_explicit_form_owner() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><input type=checkbox name=g checked><input type=CHECKBOX name=g checked=false></form><input type=checkbox form=f value=''>",
        ),
        limits(100),
    );
    let mut driver = super::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let doc = frozen.document().unwrap();
    assert_eq!(
        outline(doc),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "input"),
            (2, "input")
        ])
    );
    let html = doc
        .root()
        .children()
        .find(|id| matches!(doc.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = doc.node(&html).unwrap().children().nth(1).unwrap();
    let form = doc.node(&body).unwrap().children().next().unwrap();
    let outside = doc.node(&body).unwrap().children().nth(1).unwrap();
    let members: Vec<_> = doc.node(&form).unwrap().children().collect();
    for input in members.iter().chain(std::iter::once(&outside)) {
        assert_eq!(
            frozen.form_association_owner(&mut c, input).unwrap(),
            Some(form.clone())
        );
        let state = frozen.input_state(&mut c, input).unwrap();
        assert_eq!(state.flags.input_type(), crate::HtmlInputType::Checkbox);
        assert_eq!(state.flags.checkedness(), input != &outside);
        assert!(
            !state.flags.dirty_checkedness()
                && !state.flags.dirty_value()
                && !state.flags.indeterminate()
        );
        let mut value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
        frozen
            .append_input_value(&mut c, input, &mut value)
            .unwrap();
        assert_eq!(
            value.units(),
            if input == &outside {
                &[][..]
            } else {
                &[111, 110][..]
            }
        );
        value.release(&mut c).unwrap();
        let HtmlNodeKind::Element(element) = doc.node(input).unwrap().kind() else {
            panic!("input");
        };
        assert_eq!(
            element
                .attributes()
                .iter()
                .filter(|a| a.name().local_name().code_units() == [118, 97, 108, 117, 101])
                .count(),
            usize::from(input == &outside)
        );
    }
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ordinary_checkbox_actual_work_and_storage_boundaries_stop_once_and_release_every_prefix() {
    use crate::HtmlParseResource as R;
    let source = "<!doctype html><form><input value='A&#13;B&#10;C' type=checkbox checked><input type=checkbox>Z</form>";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
    let mut driver = super::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    driver.release().unwrap();
    assert_released(&driver.control);
    let usage = driver.control.usage().clone();
    eprintln!(
        "Checkbox full transaction: work={} units={} managed_bytes={} auxiliary={} source_maps={}; kind={} flags={}",
        usage.resource(R::WorkCredits).high_water,
        usage.resource(R::ManagedUtf16Units).high_water,
        usage.resource(R::ManagedCapacityBytes).high_water,
        usage.resource(R::AuxiliaryEntries).high_water,
        usage.resource(R::SourceMapEntries).high_water,
        std::mem::size_of::<crate::HtmlInputType>(),
        std::mem::size_of::<crate::HtmlInputFlags>()
    );
    for resource in [
        R::WorkCredits,
        R::ManagedUtf16Units,
        R::ManagedCapacityBytes,
        R::AuxiliaryEntries,
        R::SourceMapEntries,
    ] {
        let peak = usage.resource(resource).high_water;
        assert!(peak > 1);
        for short in [false, true] {
            let cap = peak - u64::from(short);
            let builder = limits(100).into_builder();
            let limited = match resource {
                R::WorkCredits => builder.work_credits(cap),
                R::ManagedUtf16Units => builder.managed_utf16_units(cap),
                R::ManagedCapacityBytes => builder.managed_capacity_bytes(cap),
                R::AuxiliaryEntries => builder.auxiliary_entries(cap),
                R::SourceMapEntries => builder.source_map_entries(cap),
                _ => unreachable!(),
            }
            .finish()
            .unwrap();
            let request =
                crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limited);
            let mut driver = super::entry::initialized_driver(&request);
            let result = (|| {
                for _ in 0..1000 {
                    let step = driver.step()?;
                    if step != super::head::HeadStep::Consumed {
                        return Ok::<_, super::StandaloneError>(step);
                    }
                }
                panic!("finite actual source");
            })();
            let disposal = driver.release();
            if short {
                // The complete transaction includes terminal-safe disposal.
                // A one-short Work cap can reach Complete then refuse its
                // final cleanup admission; the same original owner is retried.
                assert!(result.is_err() || disposal.is_err());
                assert!(
                    matches!(driver.control.stop(), Some(ParseStop::LimitExceeded { resource: actual, .. }) if actual == resource)
                );
                let first = driver.control.stop().unwrap();
                let at_stop = driver.control.usage().clone();
                assert!(driver.step().is_err());
                assert_eq!(driver.control.stop(), Some(first));
                assert_eq!(driver.control.usage(), &at_stop);
                driver.release().unwrap();
                assert_released(&driver.control);
                assert_eq!(driver.control.stop(), Some(first));
                assert_eq!(
                    driver.control.usage().resource(R::WorkCredits).current,
                    at_stop.resource(R::WorkCredits).current
                );
            } else {
                assert_eq!(result.ok(), Some(super::head::HeadStep::Complete));
                disposal.unwrap();
                assert_released(&driver.control);
                assert_eq!(driver.control.usage().resource(resource).high_water, peak);
                assert_eq!(driver.control.stop(), None);
            }
        }
    }
}

#[test]
fn ordinary_checkbox_fostered_void_node_retains_current_form_and_unsanitized_value() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><table><input type=checkbox value='A&#13;B&#10;C' checked><tr><td>T</table>Z</form>",
        ),
        limits(100),
    );
    let mut driver = super::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let doc = frozen.document().unwrap();
    assert_eq!(
        outline(doc),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "table"),
            (4, "tbody"),
            (5, "tr"),
            (6, "td"),
            (7, "#T"),
            (3, "#Z")
        ])
    );
    let html = doc
        .root()
        .children()
        .find(|id| matches!(doc.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = doc.node(&html).unwrap().children().nth(1).unwrap();
    let form = doc.node(&body).unwrap().children().next().unwrap();
    let input = doc.node(&form).unwrap().children().next().unwrap();
    assert_eq!(doc.node(&input).unwrap().children().count(), 0);
    assert_eq!(
        frozen.form_association_owner(&mut c, &input).unwrap(),
        Some(form)
    );
    let state = frozen.input_state(&mut c, &input).unwrap();
    assert_eq!(state.flags.input_type(), crate::HtmlInputType::Checkbox);
    assert!(state.flags.checkedness());
    assert!(!state.flags.dirty_checkedness() && !state.value_rewritten_during_creation);
    let mut value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
    frozen
        .append_input_value(&mut c, &input, &mut value)
        .unwrap();
    assert_eq!(value.units(), [65, 13, 66, 10, 67]);
    value.clear(&mut c).unwrap();
    frozen
        .append_input_internal_value(&mut c, &input, &mut value)
        .unwrap();
    assert_eq!(value.units(), [65, 13, 66, 10, 67]);
    value.release(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ordinary_details_name_group_closes_newly_inserted_open_member() {
    // HTML877d details insertion steps close the given element when another
    // open element belongs to its exact nonempty name group in the same tree.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><details name=g open><summary>A</summary></details><details name=g open><summary>B</summary></details>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.stop(), first_stop);
        assert_eq!(boundary, super::head::HeadStep::Complete);
        return;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let members: Vec<_> = document.node(&body).unwrap().children().collect();
    let actual_attributes: Vec<_> = members
        .iter()
        .map(|id| {
            let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() else {
                panic!("actual native details element");
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
    drop(members);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "details"),
            (3, "summary"),
            (4, "#A"),
            (2, "details"),
            (3, "summary"),
            (4, "#B"),
        ])
    );
    assert_eq!(
        actual_attributes,
        vec![
            vec![
                (None, vec![110, 97, 109, 101], vec![103]),
                (None, vec![111, 112, 101, 110], vec![]),
            ],
            vec![(None, vec![110, 97, 109, 101], vec![103])],
        ],
        "insertion must preserve the existing open peer and close the given member"
    );
}

#[test]
fn ordinary_comment_diagnostics_retain_bounded_event_layouts() {
    use crate::construction::{
        StringLocation,
        evidence::{
            DiagnosticContext, DiagnosticKind, DiagnosticLocation, DiagnosticLocationRole,
            DiagnosticRecord, DiagnosticSeverity, ParseEvidence,
        },
        native_view::{NativeKind, NativeString, NativeView},
        provenance::{
            AssociationRelation, ConstructionCause, NativeAssociation, NativeDestination,
            OriginGroup,
        },
        provenance::{
            LexicalDisposition, LexicalKind, LexicalOccurrence, LexicalPart, LexicalRole,
        },
    };
    use crate::parser::tokenizer::token::{
        DiagnosticCode, DiagnosticPhase, OriginKind, TextOrigin,
    };
    use std::mem::size_of;

    fn units(view: &mut NativeView<'_>, value: NativeString<'_>, expected: &[u16]) {
        assert_eq!(value.len(), expected.len());
        for (index, &unit) in expected.iter().enumerate() {
            assert_eq!(view.unit(&value, index).unwrap(), Some(unit));
        }
        assert_eq!(view.unit(&value, expected.len()).unwrap(), None);
    }

    // HTML 877d comment state: each literal NULL reports one tokenizer error
    // and appends U+FFFD. The proper doctype and explicit skeleton require no
    // implied-node or missing-doctype recovery. All NULLs precede head handoff.
    // The source has N+61 UTF-8 bytes, six nodes, five edges and no attributes.
    // Before any observation, allow all exact-growth submissions by these
    // three N-record writers: Event, DiagnosticRecord and DiagnosticLocation.
    // 2*(N+64)^2*(their sizes + u16) also covers old/new comment-unit layouts;
    // the independent linear allowance covers lexer/origin/lexical/native/D1
    // operations and their compiled records. These are roomy fixture fees,
    // not a universal work bound or the archived 100M diagnostic threshold.
    let per_record = size_of::<Event>()
        + size_of::<DiagnosticRecord>()
        + size_of::<DiagnosticLocation>()
        + size_of::<TextOrigin>()
        + size_of::<LexicalPart>()
        + size_of::<LexicalOccurrence>()
        + size_of::<ConstructionCause>()
        + size_of::<OriginGroup>()
        + size_of::<NativeAssociation>()
        + size_of::<crate::HtmlSourceRange>()
        + size_of::<ParserNode>()
        + size_of::<u16>();
    let fixed = size_of::<StandaloneDocumentDriver<'_>>()
        + size_of::<head::HeadStage<'_>>()
        + size_of::<head::HeadState<'_>>()
        + size_of::<Tokenizer<'_>>()
        + size_of::<ParserArenaSession>()
        + size_of::<ParseEvidence>()
        + size_of::<ParseControl>()
        + size_of::<terminal::CompletedStandaloneDocument<'_>>()
        + size_of::<terminal::FrozenStandaloneParts>();
    let exact_growth_record = size_of::<Event>()
        + size_of::<DiagnosticRecord>()
        + size_of::<DiagnosticLocation>()
        + size_of::<u16>();
    let mut observations = Vec::new(); // Caller-owned test observations, not parser storage.
    for n in [0_usize, 1, 8, 32, 128, 512] {
        let room = (n + 64) as u64;
        let physical = 64 * room * per_record as u64 + 16 * fixed as u64;
        let work = 2 * room * room * exact_growth_record as u64
            + 1024 * room * per_record as u64
            + 32 * fixed as u64;
        // N origins, N diagnostic locations, N lexical data parts and fixed
        // native fields/causes, plus their final source-qualified D1 rows.
        // The 128-record/unit allowance also admits transient cursor/cache
        // slots without making observed row counts select the limit.
        let caps = HtmlParseLimits::builder()
            .source_units((n + 61) as u64)
            .buffered_utf16_units(64 * room)
            .token_attributes(0)
            .auxiliary_entries(64 * room)
            .managed_nodes(8)
            .managed_edges(8)
            .managed_attributes(0)
            .managed_utf16_units(64 * room)
            .source_map_entries(128 * room)
            .source_map_payload_units(64 * room)
            .diagnostics((n + 8) as u64)
            .diagnostic_payload_units(0)
            .context_effects(0)
            .open_elements(8)
            .tree_depth(8)
            .managed_capacity_bytes(physical)
            .peak_transient_bytes(2 * physical)
            .atomic_layout_bytes(physical)
            .work_credits(work)
            .finish()
            .unwrap();
        let raw = format!(
            "<!doctype html><!--{}--><html><head></head><body></body></html>",
            "\0".repeat(n)
        );
        assert_eq!(raw.len(), n + 61);
        let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), caps);
        let source = request.source();
        let mut driver = entry::initialized_driver(&request);
        let mut submitted_slots = 0_usize;
        let mut reservations = 0_usize;
        let mut retained_capacity = 0_usize;
        let mut handed_off = false;
        let mut complete = false;
        for _ in 0..(2 * source.len() + 128) {
            let before_head = driver.head.is_none();
            let old_capacity = driver.lexical.as_ref().unwrap().capacity();
            let step = driver.step().unwrap();
            if before_head && driver.head.is_none() {
                let events = driver.lexical.as_ref().unwrap();
                let capacity = events.capacity();
                if capacity != old_capacity {
                    // Actual old+new submitted layouts at this writer, never
                    // total ReserveByte attributed to one vector or memcpy.
                    submitted_slots += old_capacity + capacity;
                    reservations += 1;
                }
                assert!(events.len() <= n);
                retained_capacity = capacity;
            } else if before_head {
                let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
                assert_eq!(state.lexical.len(), n);
                assert_eq!(state.lexical.capacity(), retained_capacity);
                handed_off = true; // Same owning vector moved, not copied/reissued.
            }
            match step {
                head::HeadStep::Consumed => {}
                head::HeadStep::Complete => {
                    complete = true;
                    break;
                }
                boundary => panic!("unexpected ordinary boundary for N={n}: {boundary:?}"),
            }
        }
        assert!(complete && handed_off, "actual EOF completion for N={n}");
        assert_eq!(driver.control.stop(), None);
        let parse_reserve = driver.control.usage().work(WorkCategory::ReserveByte);
        assert!(parse_reserve >= (submitted_slots * size_of::<Event>()) as u64);
        {
            let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
            let evidence = state.before.kernel.session.observed_evidence();
            let events = state.lexical.as_slice();
            assert_eq!(events.len(), n);
            for (i, event) in events.iter().enumerate() {
                let Event::Diagnostic(id) = event else {
                    panic!("only NULL diagnostics retained");
                };
                let (record, location) = evidence.get(&mut driver.control, id).unwrap();
                assert_eq!(
                    record.kind,
                    DiagnosticKind::Tokenizer(DiagnosticCode::UnexpectedNullCharacter)
                );
                assert_eq!(
                    record.context,
                    DiagnosticContext::Tokenizer {
                        phase: DiagnosticPhase::Tokenizer,
                        state: State::Comment
                    }
                );
                assert_eq!(location.source.source(), source.identity());
                assert_eq!(
                    (location.source.start(), location.source.end()),
                    (19 + i, 20 + i)
                );
            }
            let (occurrences, parts, omissions) = evidence.lexical_observations();
            assert!(omissions.is_empty());
            let comments: Vec<_> = occurrences
                .iter()
                .enumerate()
                .filter(|(_, item)| item.kind == LexicalKind::Comment)
                .collect();
            assert_eq!(comments.len(), 1);
            let (comment, occurrence) = comments[0];
            assert_eq!(occurrence.disposition, LexicalDisposition::Applied);
            assert_eq!(occurrence.parent, None);
            assert_eq!(occurrence.source.source(), source.identity());
            assert_eq!(
                (occurrence.source.start(), occurrence.source.end()),
                (15, n + 22)
            );
            let data: Vec<_> = parts
                .iter()
                .filter(|part| part.occurrence == comment && part.role == LexicalRole::Data)
                .collect();
            assert_eq!(data.len(), n.max(1));
            for (i, part) in data.iter().enumerate() {
                if n == 0 {
                    assert_eq!(part.kind, None);
                    assert_eq!((part.start, part.end), (0, 0));
                    assert_eq!(part.source.source(), source.identity());
                    assert_eq!((part.source.start(), part.source.end()), (21, 21));
                    continue;
                }
                assert_eq!(part.kind, Some(OriginKind::NullReplacement));
                assert_eq!((part.start, part.end), (i, i + 1));
                assert_eq!(part.source.source(), source.identity());
                assert_eq!((part.source.start(), part.source.end()), (19 + i, 20 + i));
            }
        }
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        assert!(input.is_none());
        {
            let diagnostics = completed.diagnostics().unwrap();
            assert_eq!(diagnostics.len(), n);
            for i in 0..n {
                let (record, location) = diagnostics.retained_record(i).unwrap().unwrap();
                assert_eq!(
                    record.kind,
                    DiagnosticKind::Tokenizer(DiagnosticCode::UnexpectedNullCharacter)
                );
                assert_eq!(
                    record.context,
                    DiagnosticContext::Tokenizer {
                        phase: DiagnosticPhase::Tokenizer,
                        state: State::Comment
                    }
                );
                assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
                assert_eq!(record.rule.anchor, "parse-error-unexpected-null-character");
                assert_eq!(record.rule.branch, "unexpected-null-character");
                assert_eq!(location.role, DiagnosticLocationRole::InputFrontier);
                assert_eq!(location.token, None);
                assert_eq!(location.source.source(), source.identity());
                assert_eq!(
                    (location.source.start(), location.source.end()),
                    (19 + i, 20 + i)
                );
            }
        }
        let comment;
        {
            let mut view = completed.native_view().unwrap();
            let root = view.root().unwrap();
            assert!(matches!(
                view.kind(&root).unwrap(),
                NativeKind::Document(HtmlDocumentMode::NoQuirks)
            ));
            assert!(view.parent(&root).unwrap().is_none());
            assert_eq!(view.child_count(&root).unwrap(), 3);
            let doctype = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Doctype {
                name,
                public_id,
                system_id,
            } = view.kind(&doctype).unwrap()
            else {
                panic!("doctype");
            };
            units(&mut view, name, &[104, 116, 109, 108]);
            // DOM creation defaults are real empty strings, not absent IDs.
            units(&mut view, public_id.unwrap(), &[]);
            units(&mut view, system_id.unwrap(), &[]);
            assert_eq!(view.child_count(&doctype).unwrap(), 0);
            comment = view.child_at(&root, 1).unwrap().unwrap();
            let NativeKind::Comment(value) = view.kind(&comment).unwrap() else {
                panic!("comment");
            };
            units(&mut view, value, &vec![0xfffd; n]);
            assert_eq!(view.child_count(&comment).unwrap(), 0);
            let html = view.child_at(&root, 2).unwrap().unwrap();
            assert_eq!(view.child_count(&html).unwrap(), 2);
            for (node, expected) in [
                (&html, "html"),
                (&view.child_at(&html, 0).unwrap().unwrap(), "head"),
                (&view.child_at(&html, 1).unwrap().unwrap(), "body"),
            ] {
                let NativeKind::Element(name) = view.kind(node).unwrap() else {
                    panic!("HTML element");
                };
                assert!(name.prefix.is_none());
                units(
                    &mut view,
                    name.namespace.unwrap(),
                    &"http://www.w3.org/1999/xhtml"
                        .encode_utf16()
                        .collect::<Vec<_>>(),
                );
                units(
                    &mut view,
                    name.local,
                    &expected.encode_utf16().collect::<Vec<_>>(),
                );
                assert_eq!(view.attribute_count(node).unwrap(), 0);
                if expected != "html" {
                    assert_eq!(view.child_count(node).unwrap(), 0);
                }
                let parent = view.parent(node).unwrap().unwrap();
                assert!(
                    view.same_node(&parent, if expected == "html" { &root } else { &html })
                        .unwrap()
                );
            }
            for node in [&doctype, &comment] {
                let parent = view.parent(node).unwrap().unwrap();
                assert!(view.same_node(&parent, &root).unwrap());
            }
        }
        {
            let mut rows = completed.native_associations().unwrap();
            let mut node_index = None;
            for index in 0..rows.len() {
                if rows.node_for(index, &comment).unwrap() {
                    let (row, _) = rows.row(index).unwrap().unwrap();
                    if let NativeDestination::Node(node) = row.destination {
                        node_index = Some(node);
                    }
                }
            }
            let node = node_index.unwrap();
            let mut count = 0;
            for index in 0..rows.len() {
                let (row, _) = rows.row(index).unwrap().unwrap();
                if row.destination == NativeDestination::Field(StringLocation::Comment(node))
                    && row.relation == AssociationRelation::Output
                {
                    assert!(row.group.is_some());
                    if n == 0 {
                        assert_eq!((row.start, row.end), (0, 0));
                        assert!(row.source.is_none() && row.kind.is_none());
                    } else {
                        assert_eq!((row.start, row.end), (count, count + 1));
                        assert_eq!(row.kind, Some(OriginKind::NullReplacement));
                        let range = row.source.as_ref().unwrap();
                        assert_eq!(range.source(), source.identity());
                        assert_eq!((range.start(), range.end()), (19 + count, 20 + count));
                    }
                    count += 1;
                }
            }
            assert_eq!(count, n.max(1));
        }
        drop(comment);
        assert!(completed.at_stop().is_none());
        let before_cleanup = completed.usage().unwrap().clone();
        let mut frozen = completed.freeze_parts().unwrap();
        assert_eq!(frozen.source_binding().0, source.identity());
        assert_eq!(frozen.source_binding().2, n + 61);
        assert!(frozen.at_stop().is_none());
        frozen.release().unwrap();
        let after = frozen.retained_usage();
        assert_eq!(after.actual_capacity_bytes, 0);
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
            assert_eq!(after.resource(resource).current, 0, "N={n} {resource:?}");
            assert!(
                after.resource(resource).high_water >= before_cleanup.resource(resource).high_water
            );
        }
        assert!(after.work(WorkCategory::ReserveByte) >= parse_reserve);
        assert!(frozen.after_cleanup().is_some());
        assert!(frozen.at_stop().is_none());
        eprintln!(
            "Event witness N={n} reservations={reservations} submitted_slots={submitted_slots} retained_capacity={retained_capacity} compiled_event_bytes={} parse_all_writer_reserve_bytes={parse_reserve} cleanup_actual_bytes={}",
            size_of::<Event>(),
            after.actual_capacity_bytes
        );
        observations.push((n, submitted_slots));
    }
    // All six actual completed trees, diagnostics, lexical/native ranges and
    // zero-storage disposal were checked above. A generous 8N old+new slot
    // envelope tests this actual writer's avoidable repeated exact growth;
    // it is not an HTML constant or a blanket reservation-policy requirement.
    for (n, submitted_slots) in observations {
        assert!(
            submitted_slots <= 8 * n,
            "Event layout submissions for N={n}: {submitted_slots} > {}",
            8 * n
        );
    }
}

#[test]
fn ordinary_radio_same_group_keeps_only_later_checked_member_through_eof() {
    // HTML877d Radio group membership requires the same ordinary tree and form
    // owner, with equal nonempty name values. Inserting the later checked member
    // clears only the earlier member's current checkedness, not its attribute.
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><input type=radio name=g checked><input type=radio name=g checked>Z</form>",
        ),
        limits(100),
    );
    let mut driver = super::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.stop(), first_stop);
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "ordinary Radio must complete before its actual group state can be observed"
        );
        return;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let child_count = children.len();
    let mut namespaces = Vec::new();
    let mut names = Vec::new();
    let mut child_counts = Vec::new();
    let mut attributes = Vec::new();
    let mut owned_by_form = Vec::new();
    let mut checkedness = Vec::new();
    let mut dirty_checkedness = Vec::new();
    let mut values = Vec::new();
    for input in &children[..2] {
        let HtmlNodeKind::Element(element) = document.node(input).unwrap().kind() else {
            panic!("the Radio tokens must create actual native Elements");
        };
        namespaces.push(
            element
                .name()
                .namespace()
                .uri()
                .unwrap()
                .code_units()
                .to_vec(),
        );
        names.push(element.name().local_name().code_units().to_vec());
        child_counts.push(document.node(input).unwrap().children().count());
        attributes.push(
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
                .collect::<Vec<_>>(),
        );
        let owner = frozen.form_association_owner(&mut control, input).unwrap();
        owned_by_form.push(owner.as_ref() == Some(&form));
        drop(owner);
        let state = frozen.input_state(&mut control, input).unwrap();
        checkedness.push(state.flags.checkedness());
        dirty_checkedness.push(state.flags.dirty_checkedness());
        let mut value = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
        frozen
            .append_input_value(&mut control, input, &mut value)
            .unwrap();
        values.push(value.units().to_vec());
        value.release(&mut control).unwrap();
    }
    let HtmlNodeKind::Text(text) = document.node(&children[2]).unwrap().kind() else {
        panic!("Z must remain a real sibling after both void Radio inputs");
    };
    let following_text = text.code_units().to_vec();
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 3);
    assert_eq!(
        namespaces,
        vec![
            "http://www.w3.org/1999/xhtml"
                .encode_utf16()
                .collect::<Vec<_>>(),
            "http://www.w3.org/1999/xhtml"
                .encode_utf16()
                .collect::<Vec<_>>(),
        ]
    );
    assert_eq!(names, vec![vec![105, 110, 112, 117, 116]; 2]);
    assert_eq!(child_counts, vec![0, 0]);
    let expected_attributes = vec![
        (None, vec![116, 121, 112, 101], vec![114, 97, 100, 105, 111]),
        (None, vec![110, 97, 109, 101], vec![103]),
        (None, vec![99, 104, 101, 99, 107, 101, 100], vec![]),
    ];
    assert_eq!(
        attributes,
        vec![expected_attributes.clone(), expected_attributes]
    );
    assert_eq!(following_text, vec![90]);
    assert_eq!(owned_by_form, vec![true, true]);
    assert_eq!(values, vec![vec![111, 110]; 2]);
    assert_eq!(dirty_checkedness, vec![false, false]);
    assert_eq!(checkedness, vec![false, true]);
}

fn ordinary_input_sanitizer_observes_completed_value(
    raw: &'static str,
    expected_type: crate::HtmlInputType,
    expected_default: &[u16],
    expected_current: &[u16],
    expected_multiple: bool,
    expected_reference_origins: &[(usize, usize, usize, usize)],
) {
    use crate::construction::provenance::LexicalRole;
    use crate::parser::tokenizer::token::OriginKind;

    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let source = request.source();
    let mut driver = super::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.stop(), first_stop);
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "ordinary Email/URL must complete before actual sanitized state is observed"
        );
        return;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let child_count = children.len();
    let input = children[0].clone();
    let input_children = document.node(&input).unwrap().children().count();
    let HtmlNodeKind::Element(element) = document.node(&input).unwrap().kind() else {
        panic!("the actual input token must create a native Element");
    };
    let namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let local_name = element.name().local_name().code_units().to_vec();
    let actual_attributes: Vec<_> = element
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
        .collect();
    let HtmlNodeKind::Text(text) = document.node(&children[1]).unwrap().kind() else {
        panic!("Z remains an actual sibling after the void input");
    };
    let following_text = text.code_units().to_vec();
    let owner = frozen.form_association_owner(&mut control, &input).unwrap();
    let owner_is_form = owner.as_ref() == Some(&form);
    drop(owner);
    let state = frozen.input_state(&mut control, &input).unwrap();
    let actual_type = state.flags.input_type();
    let mode_of_value = state.flags.input_type().value_mode();
    let flags = (
        state.flags.dirty_value(),
        state.flags.dirty_checkedness(),
        state.flags.checkedness(),
        state.flags.indeterminate(),
        state.flags.user_validity(),
    );
    let writeback_absent = frozen
        .input_writeback_sources(&mut control, &input)
        .unwrap()
        .is_none();
    let mut value = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
    frozen
        .append_input_internal_value(&mut control, &input, &mut value)
        .unwrap();
    let internal_value = value.units().to_vec();
    value.clear(&mut control).unwrap();
    frozen
        .append_input_value(&mut control, &input, &mut value)
        .unwrap();
    let current_value = value.units().to_vec();
    value.clear(&mut control).unwrap();
    frozen
        .append_input_default_value(&mut control, &input, &mut value)
        .unwrap();
    let default_value = value.units().to_vec();
    value.release(&mut control).unwrap();
    let (reference_origins, original_source) = {
        let evidence = frozen.retained_evidence().unwrap();
        let (_, parts, _) = evidence.lexical_observations();
        let references: Vec<_> = parts
            .iter()
            .filter(|part| {
                part.role == LexicalRole::Value && part.kind == Some(OriginKind::CharacterReference)
            })
            .collect();
        let original_source = references
            .iter()
            .all(|part| part.source.source() == source.identity());
        (
            references
                .iter()
                .map(|part| (part.start, part.end, part.source.start(), part.source.end()))
                .collect::<Vec<_>>(),
            original_source,
        )
    };
    drop(input);
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 2);
    assert_eq!(input_children, 0);
    assert_eq!(
        namespace,
        "http://www.w3.org/1999/xhtml"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(local_name, vec![105, 110, 112, 117, 116]);
    let type_units = match expected_type {
        crate::HtmlInputType::Url => vec![117, 114, 108],
        crate::HtmlInputType::Email => vec![101, 109, 97, 105, 108],
        _ => panic!("the literal sanitizer fixtures use only URL or Email"),
    };
    let mut expected_attributes = vec![(None, vec![116, 121, 112, 101], type_units)];
    if expected_multiple {
        // Presence selects multiple even though its authored spelling is false.
        expected_attributes.push((
            None,
            vec![109, 117, 108, 116, 105, 112, 108, 101],
            vec![102, 97, 108, 115, 101],
        ));
    }
    expected_attributes.push((
        None,
        vec![118, 97, 108, 117, 101],
        expected_default.to_vec(),
    ));
    assert_eq!(actual_attributes, expected_attributes);
    assert_eq!(following_text, vec![90]);
    assert!(owner_is_form && writeback_absent && original_source);
    assert_eq!(actual_type, expected_type);
    assert_eq!(mode_of_value, crate::HtmlInputValueMode::Value);
    assert_eq!(flags, (false, false, false, false, false));
    assert_eq!(default_value, expected_default);
    assert_eq!(reference_origins, expected_reference_origins);
    assert_eq!(internal_value, expected_current);
    assert_eq!(current_value, expected_current);
}

#[test]
fn ordinary_url_strips_newlines_then_boundary_ascii_whitespace_through_eof() {
    // HTML877d #url-state-(type=url): newline removal precedes ASCII trim.
    ordinary_input_sanitizer_observes_completed_value(
        "<!doctype html><form id=f><input type=url value=' &#9;https://e.test/a&#13;b&#10;c&#12; '>Z</form>",
        crate::HtmlInputType::Url,
        &[
            32, 9, 104, 116, 116, 112, 115, 58, 47, 47, 101, 46, 116, 101, 115, 116, 47, 97, 13,
            98, 10, 99, 12, 32,
        ],
        &[
            104, 116, 116, 112, 115, 58, 47, 47, 101, 46, 116, 101, 115, 116, 47, 97, 98, 99,
        ],
        false,
        &[
            (1, 2, 50, 54),
            (18, 19, 70, 75),
            (20, 21, 76, 81),
            (22, 23, 82, 87),
        ],
    );
}

#[test]
fn ordinary_email_single_strips_newlines_then_boundary_ascii_whitespace_through_eof() {
    // HTML877d #email-state-(type=email), when multiple is absent.
    ordinary_input_sanitizer_observes_completed_value(
        "<!doctype html><form id=f><input type=email value=' &#9;a&#13;@b.test&#10;&#12; '>Z</form>",
        crate::HtmlInputType::Email,
        &[32, 9, 97, 13, 64, 98, 46, 116, 101, 115, 116, 10, 12, 32],
        &[97, 64, 98, 46, 116, 101, 115, 116],
        false,
        &[
            (1, 2, 52, 56),
            (3, 4, 57, 62),
            (11, 12, 69, 74),
            (12, 13, 74, 79),
        ],
    );
}

#[test]
fn ordinary_email_multiple_trims_tokens_but_preserves_internal_newline_through_eof() {
    // HTML877d multiple branch trims tokens and joins with commas; no global
    // newline stripping is specified. The two actual values are a\n@b.test
    // and c@d.test; the invalid address's internal LF is still retained.
    ordinary_input_sanitizer_observes_completed_value(
        "<!doctype html><form id=f><input type=email multiple=false value=' a&#10;@b.test , &#9;c@d.test '>Z</form>",
        crate::HtmlInputType::Email,
        &[
            32, 97, 10, 64, 98, 46, 116, 101, 115, 116, 32, 44, 32, 9, 99, 64, 100, 46, 116, 101,
            115, 116, 32,
        ],
        &[
            97, 10, 64, 98, 46, 116, 101, 115, 116, 44, 99, 64, 100, 46, 116, 101, 115, 116,
        ],
        true,
        &[(2, 3, 68, 73), (13, 14, 83, 87)],
    );
}

fn progressive_driver_document(raw: &str, cuts: &[usize]) -> Vec<(usize, Vec<u16>)> {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
    let mut attempt = owner.begin_progressive().unwrap();
    let mut driver = attempt.take_driver().unwrap();
    for cut in cuts.iter().copied().chain(std::iter::once(raw.len())) {
        driver.make_available(cut).unwrap();
        let mut starved = false;
        for _ in 0..(8 * raw.len() + 256) {
            match driver.step().unwrap() {
                head::HeadStep::Consumed => {}
                head::HeadStep::NeedInput => {
                    starved = true;
                    break;
                }
                other => panic!("unsealed source completed or crossed boundary: {other:?}"),
            }
        }
        assert!(starved);
        assert!(driver.control.stop().is_none());
        // NeedInput does not consume the driver or create completion proof.
        let mut slot = Some(driver);
        assert!(StandaloneDocumentDriver::take_completed(&mut slot).is_err());
        driver = slot.take().unwrap();
    }
    driver.seal_input().unwrap();
    let mut complete = false;
    for _ in 0..(8 * raw.len() + 256) {
        match driver.step().unwrap() {
            head::HeadStep::Consumed => {}
            head::HeadStep::Complete => {
                complete = true;
                break;
            }
            other => panic!("sealed source failed completion: {other:?}"),
        }
    }
    assert!(complete);
    let mut slot = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut slot).unwrap();
    completed.prepare_native_associations().unwrap();
    let parts = completed.freeze_parts().unwrap();
    drop(completed);
    drop(attempt);
    let mut original = Some(owner.take_request().unwrap());
    let mut parts = Some(parts);
    let mut sealed =
        super::seal::SealedOrdinaryConstruction::take(&mut original, &mut parts).unwrap();
    assert_eq!(sealed.source().as_utf8(), Some(raw));
    assert!(sealed.is_offline());
    assert_eq!(
        sealed
            .final_usage()
            .resource(HtmlParseResource::SourceUnits)
            .current,
        raw.len() as u64
    );
    let observed = outline(sealed.document().unwrap());
    sealed.release().unwrap();
    observed
}
#[test]
fn progressive_ordinary_every_split_retains_native_tree_until_explicit_seal() {
    let raw = "<!doctype html><p a='&amp;'>A&notin;B</p><i>C</i>";
    let expected = expected_outline(&[
        (0, "html"),
        (1, "head"),
        (1, "body"),
        (2, "p"),
        (3, "#A∉B"),
        (2, "i"),
        (3, "#C"),
    ]);
    for cut in 0..=raw.len() {
        assert_eq!(
            progressive_driver_document(raw, &[cut]),
            expected,
            "cut={cut}"
        );
    }
    assert_eq!(
        progressive_driver_document(raw, &(0..=raw.len()).collect::<Vec<_>>()),
        expected
    );
}
#[test]
fn progressive_script_end_handoff_at_unavailable_frontier_is_zero_width_and_not_replayed() {
    for (raw, expected) in [
        (
            "<script>x</script><p>y</p>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (2, "script"),
                (3, "#x"),
                (1, "body"),
                (2, "p"),
                (3, "#y"),
            ]),
        ),
        (
            "<svg><script>x</script><path/></svg><p>y</p>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "svg"),
                (3, "script"),
                (4, "#x"),
                (3, "path"),
                (2, "p"),
                (3, "#y"),
            ]),
        ),
    ] {
        let end = raw.find("</script>").unwrap() + 9;
        assert_eq!(progressive_driver_document(raw, &[end]), expected);
    }
}
#[test]
fn progressive_retained_reference_cancellation_is_first_stop_and_has_no_retry_reads() {
    let raw = "<!doctype html><p>&NotEqualTilde;</p>";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let cancel = request.cancellation().clone();
    let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
    let mut attempt = owner.begin_progressive().unwrap();
    let mut driver = attempt.take_driver().unwrap();
    driver.make_available(raw.find("Tilde").unwrap()).unwrap();
    while driver.step().unwrap() != head::HeadStep::NeedInput {}
    let before = driver.control.usage().clone();
    cancel.cancel();
    assert!(matches!(
        driver.step(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    let first = driver.control.first_stop_usage().unwrap().clone();
    assert!(matches!(
        driver.make_available(raw.len()),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert!(matches!(
        driver.seal_input(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert!(matches!(
        driver.step(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert_eq!(
        driver.control.usage().work(WorkCategory::InputUnitRead),
        before.work(WorkCategory::InputUnitRead)
    );
    assert_eq!(driver.control.first_stop_usage(), Some(&first));
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.first_stop_usage(), Some(&first));
}

#[test]
fn progressive_table_template_select_clone_and_checkbox_partitions_use_the_same_tree() {
    for (raw, expected) in [
        (
            "<table>X<tr><td>A</table><p>B",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "#X"),
                (2, "table"),
                (3, "tbody"),
                (4, "tr"),
                (5, "td"),
                (6, "#A"),
                (2, "p"),
                (3, "#B"),
            ]),
        ),
        (
            "<template><tr><td>A</template><p>B",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (2, "template"),
                (1, "body"),
                (2, "p"),
                (3, "#B"),
            ]),
        ),
        (
            "<form><input type=checkbox value='A\r\nB' checked><p>X</form>",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "form"),
                (3, "input"),
                (3, "p"),
                (4, "#X"),
            ]),
        ),
        (
            "<select><button><selectedcontent></selectedcontent></button><option selected>A<b>B</b></option></select><p>C",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "button"),
                (4, "selectedcontent"),
                (5, "#A"),
                (5, "b"),
                (6, "#B"),
                (3, "option"),
                (4, "#A"),
                (4, "b"),
                (5, "#B"),
                (2, "p"),
                (3, "#C"),
            ]),
        ),
        (
            "",
            expected_outline(&[(0, "html"), (1, "head"), (1, "body")]),
        ),
    ] {
        let cuts = (0..=raw.len()).collect::<Vec<_>>();
        assert_eq!(progressive_driver_document(raw, &cuts), expected, "{raw:?}");
    }
}
#[test]
fn progressive_advance_and_seal_refuse_invalid_frontiers_and_consumed_owners() {
    for operation in 0..4 {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>X"), limits(100));
        let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
        let mut attempt = owner.begin_progressive().unwrap();
        let mut driver = attempt.take_driver().unwrap();
        driver.make_available(2).unwrap();
        let error = match operation {
            0 => driver.make_available(1),
            1 => driver.make_available(5),
            2 => driver.seal_input(),
            _ => {
                driver.make_available(4).unwrap();
                driver.seal_input().unwrap();
                driver.seal_input()
            }
        }
        .unwrap_err();
        assert!(matches!(
            error,
            StandaloneError::Stop(ParseStop::InternalInvariant(
                crate::parser::control::Invariant::InputCursorState
            ))
        ));
        let first = driver.control.first_stop_usage().unwrap().clone();
        let reads = driver.control.usage().work(WorkCategory::InputUnitRead);
        assert!(driver.step().is_err());
        assert!(driver.make_available(4).is_err());
        assert!(driver.seal_input().is_err());
        assert_eq!(
            driver.control.usage().work(WorkCategory::InputUnitRead),
            reads
        );
        assert_eq!(driver.control.first_stop_usage(), Some(&first));
        driver.release().unwrap();
        assert_released(&driver.control);
        assert!(driver.step().is_err());
    }
}

fn progressive_successfully_released_driver_refuses_feed_once(seal: bool) {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>X"), limits(100));
    let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
    let mut attempt = owner.begin_progressive().unwrap();
    let mut driver = attempt.take_driver().unwrap();
    driver.release().unwrap();
    assert_released(&driver.control);
    assert!(driver.control.stop().is_none());
    assert!(driver.control.first_stop_usage().is_none());
    let error = if seal {
        driver.seal_input()
    } else {
        driver.make_available(0)
    }
    .unwrap_err();
    assert!(matches!(error, StandaloneError::Consumed));
    let expected = ParseStop::InternalInvariant(Invariant::RegistryRow);
    assert_eq!(driver.control.stop(), Some(expected));
    let first = driver.control.first_stop_usage().unwrap().clone();
    assert_eq!(driver.control.usage(), &first);
    for _ in 0..2 {
        assert!(matches!(
            driver.make_available(4),
            Err(StandaloneError::Stop(stop)) if stop == expected
        ));
        assert!(matches!(
            driver.seal_input(),
            Err(StandaloneError::Stop(stop)) if stop == expected
        ));
        assert!(matches!(
            driver.step(),
            Err(StandaloneError::Stop(stop)) if stop == expected
        ));
        assert_eq!(driver.control.usage(), &first);
        assert_eq!(driver.control.first_stop_usage(), Some(&first));
    }
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.first_stop_usage(), Some(&first));
}

#[test]
fn progressive_successful_release_then_make_available_latches_consumed_first_stop() {
    progressive_successfully_released_driver_refuses_feed_once(false);
}

#[test]
fn progressive_successful_release_then_seal_input_latches_consumed_first_stop() {
    progressive_successfully_released_driver_refuses_feed_once(true);
}

#[derive(Clone, Copy)]
enum ProgressiveNativeField {
    Text,
    Comment,
    AttributeName(usize),
    AttributeValue(usize),
}
struct ProgressiveFieldExpectation<'a> {
    path: &'a [usize],
    field: ProgressiveNativeField,
    value: &'a str,
    // Full ordered output interval, source interval and origin kind, never an
    // opaque group or payload-container index masquerading as a native node.
    origins: &'a [(
        usize,
        usize,
        usize,
        usize,
        crate::parser::tokenizer::token::OriginKind,
    )],
    operation: crate::construction::provenance::ConstructionOperation,
    rule: crate::construction::provenance::ParserRule,
    family: crate::construction::evidence::DiagnosticToken,
}
struct ProgressiveAttributeExpectation<'a> {
    path: &'a [usize],
    // Actual retained order, unlike the corpus dump's alphabetic presentation.
    attributes: &'a [(Option<&'a str>, Option<&'a str>, &'a str, &'a str)],
}
struct ProgressiveDiagnosticExpectation {
    kind: crate::construction::evidence::DiagnosticKind,
    context: crate::construction::evidence::DiagnosticContext,
    role: crate::construction::evidence::DiagnosticLocationRole,
    token: Option<crate::construction::evidence::DiagnosticToken>,
    source: (usize, usize),
    anchor: &'static str,
    branch: &'static str,
}

fn progressive_native_node(
    view: &mut crate::construction::native_view::NativeView<'_>,
    path: &[usize],
) -> ParserNode {
    let mut node = view.root().unwrap();
    for &position in path {
        node = if position == usize::MAX {
            view.template_contents(&node).unwrap().unwrap()
        } else {
            view.child_at(&node, position).unwrap().unwrap()
        };
    }
    node
}

fn progressive_native_units(
    view: &mut crate::construction::native_view::NativeView<'_>,
    value: crate::construction::native_view::NativeString<'_>,
    expected: &str,
) {
    let expected: Vec<_> = expected.encode_utf16().collect();
    assert_eq!(value.len(), expected.len());
    for (index, unit) in expected.into_iter().enumerate() {
        assert_eq!(view.unit(&value, index).unwrap(), Some(unit));
    }
    assert_eq!(view.unit(&value, value.len()).unwrap(), None);
}

fn progressive_completed_native_evidence(
    raw: &str,
    cuts: &[usize],
    dump: &str,
    attributes: &[ProgressiveAttributeExpectation<'_>],
    fields: &[ProgressiveFieldExpectation<'_>],
    diagnostics: &[ProgressiveDiagnosticExpectation],
) {
    use crate::construction::{
        StringLocation,
        evidence::DiagnosticSeverity,
        native_view::NativeKind,
        provenance::{
            AssociationRelation, ConstructionOperation, GroupBody, NativeAttributeField,
            NativeDestination,
        },
    };
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let source = request.source().clone();
    let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
    let mut attempt = owner.begin_progressive().unwrap();
    let mut driver = attempt.take_driver().unwrap();
    for cut in cuts.iter().copied().chain(std::iter::once(raw.len())) {
        driver.make_available(cut).unwrap();
        let mut starved = false;
        for _ in 0..(8 * raw.len() + 256) {
            match driver.step().unwrap() {
                head::HeadStep::Consumed => {}
                head::HeadStep::NeedInput => {
                    starved = true;
                    break;
                }
                other => panic!("unsealed completion/boundary: {other:?}"),
            }
        }
        assert!(starved && driver.control.stop().is_none());
    }
    driver.seal_input().unwrap();
    let mut complete = false;
    for _ in 0..(8 * raw.len() + 256) {
        match driver.step().unwrap() {
            head::HeadStep::Consumed => {}
            head::HeadStep::Complete => {
                complete = true;
                break;
            }
            other => panic!("sealed completion: {other:?}"),
        }
    }
    assert!(complete);
    let mut slot = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut slot).unwrap();
    assert!(slot.is_none());
    // This is the existing admitted full native observer/comparator, including
    // namespaces, prefixes, attributes, ordinary and template edges/backlinks.
    conformance::assert_completed_literal(&mut completed, dump, HtmlDocumentMode::NoQuirks);
    {
        let retained = completed.diagnostics().unwrap();
        assert_eq!(retained.len(), diagnostics.len());
        for (index, expected) in diagnostics.iter().enumerate() {
            let (record, location) = retained.retained_record(index).unwrap().unwrap();
            assert_eq!(record.kind, expected.kind);
            assert_eq!(record.context, expected.context);
            assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
            assert_eq!(
                (record.rule.anchor, record.rule.branch),
                (expected.anchor, expected.branch)
            );
            assert_eq!(location.role, expected.role);
            assert_eq!(location.token, expected.token);
            assert_eq!(location.source.source(), source.identity());
            assert_eq!(location.source.coordinates(), source.coordinates());
            assert_eq!(
                (location.source.start(), location.source.end()),
                expected.source
            );
        }
    }
    {
        let mut view = completed.native_view().unwrap();
        for expected in attributes {
            let node = progressive_native_node(&mut view, expected.path);
            assert_eq!(
                view.attribute_count(&node).unwrap(),
                expected.attributes.len()
            );
            for (slot, &(namespace, prefix, local, value)) in expected.attributes.iter().enumerate()
            {
                let attr = view.attribute_at(&node, slot).unwrap().unwrap();
                assert_eq!(attr.name.namespace.is_some(), namespace.is_some());
                assert_eq!(attr.name.prefix.is_some(), prefix.is_some());
                if let Some(namespace) = namespace {
                    progressive_native_units(&mut view, attr.name.namespace.unwrap(), namespace);
                }
                if let Some(prefix) = prefix {
                    progressive_native_units(&mut view, attr.name.prefix.unwrap(), prefix);
                }
                progressive_native_units(&mut view, attr.name.local, local);
                progressive_native_units(&mut view, attr.value, value);
            }
            assert!(
                view.attribute_at(&node, expected.attributes.len())
                    .unwrap()
                    .is_none()
            );
        }
    }
    completed.prepare_native_associations().unwrap();
    for expected in fields {
        let node = {
            let mut view = completed.native_view().unwrap();
            let node = progressive_native_node(&mut view, expected.path);
            let value = match expected.field {
                ProgressiveNativeField::Text => match view.kind(&node).unwrap() {
                    NativeKind::Text(value) => value,
                    _ => panic!("literal path must be native Text"),
                },
                ProgressiveNativeField::Comment => match view.kind(&node).unwrap() {
                    NativeKind::Comment(value) => value,
                    _ => panic!("literal path must be native Comment"),
                },
                ProgressiveNativeField::AttributeName(slot) => {
                    view.attribute_at(&node, slot).unwrap().unwrap().name.local
                }
                ProgressiveNativeField::AttributeValue(slot) => {
                    view.attribute_at(&node, slot).unwrap().unwrap().value
                }
            };
            progressive_native_units(&mut view, value, expected.value);
            node
        };
        let mut rows = completed.native_associations().unwrap();
        // Comment field locations use native node identities. Attribute fields
        // use retained container identities and must use the published checked
        // mapping instead of comparing their raw indices with native nodes.
        let mut comment_index = None;
        if matches!(expected.field, ProgressiveNativeField::Comment) {
            for index in 0..rows.len() {
                if rows.node_for(index, &node).unwrap() {
                    let (row, _) = rows.row(index).unwrap().unwrap();
                    if let NativeDestination::Node(index) = row.destination {
                        comment_index = Some(index);
                    }
                }
            }
            assert!(comment_index.is_some());
        }
        let mut count = 0;
        let mut group_index = None;
        for index in 0..rows.len() {
            let matched = match expected.field {
                ProgressiveNativeField::Text => rows.text_for(index, &node).unwrap(),
                ProgressiveNativeField::Comment => {
                    rows.row(index).unwrap().unwrap().0.destination
                        == NativeDestination::Field(StringLocation::Comment(comment_index.unwrap()))
                }
                ProgressiveNativeField::AttributeName(slot) => rows
                    .attribute_for(index, &node, slot, NativeAttributeField::Name)
                    .unwrap(),
                ProgressiveNativeField::AttributeValue(slot) => rows
                    .attribute_for(index, &node, slot, NativeAttributeField::Value)
                    .unwrap(),
            };
            if !matched {
                continue;
            }
            let (row, cause) = rows.row(index).unwrap().unwrap();
            assert_eq!(row.relation, AssociationRelation::Output);
            let &(start, end, source_start, source_end, kind) = expected
                .origins
                .get(count)
                .expect("no extra output origin or uncovered interval");
            assert_eq!((row.start, row.end), (start, end));
            assert_eq!(row.kind, Some(kind));
            let range = row.source.as_ref().unwrap();
            assert_eq!(range.source(), source.identity());
            assert_eq!(range.coordinates(), source.coordinates());
            assert_eq!((range.start(), range.end()), (source_start, source_end));
            let group = row.group.unwrap();
            if let Some(previous) = group_index {
                assert_eq!(group, previous);
            }
            group_index = Some(group);
            let cause_index = row.cause;
            assert_eq!(cause.operation, expected.operation);
            if expected.operation == ConstructionOperation::Copy {
                let original = cause.original.unwrap();
                assert!(cause.source.is_none());
                let group = rows.group(group).unwrap().unwrap();
                assert!(
                    matches!(group.body, GroupBody::Alias { original: actual } if actual == original)
                );
            } else if expected.operation == ConstructionOperation::Replay {
                let original = cause.original_cause.unwrap();
                assert!(cause.source.is_none());
                assert!(cause.rule.is_none() && cause.family.is_none());
                let original = rows.cause_at(original).unwrap().unwrap();
                assert_eq!(original.operation, ConstructionOperation::Token);
                assert_eq!(original.rule, Some(expected.rule));
                assert_eq!(original.family, Some(expected.family));
                let cause_source = original.source.as_ref().unwrap();
                assert_eq!(cause_source.source(), source.identity());
                assert_eq!(
                    (cause_source.start(), cause_source.end()),
                    (source_start, source_end)
                );
            } else {
                assert_eq!(cause.rule, Some(expected.rule));
                assert_eq!(cause.family, Some(expected.family));
                let cause_source = cause.source.as_ref().unwrap();
                assert_eq!(cause_source.source(), source.identity());
                assert!(cause_source.start() <= source_start && cause_source.end() >= source_end);
            }
            let group = rows.group(group).unwrap().unwrap();
            assert_eq!(group.cause, cause_index);
            assert_eq!(group.units, expected.value.encode_utf16().count());
            if expected.operation == ConstructionOperation::Copy {
                assert!(matches!(group.body, GroupBody::Alias { .. }));
            }
            count += 1;
        }
        assert_eq!(count, expected.origins.len());
        let group = group_index.unwrap();
        let mut cursor = Some(rows.group_origin_cursor(group).unwrap());
        for &(start, end, source_start, source_end, kind) in expected.origins {
            let origin = rows
                .next_group_origin(cursor.as_mut().unwrap())
                .unwrap()
                .unwrap();
            assert_eq!((origin.start, origin.end, origin.kind), (start, end, kind));
            assert_eq!(origin.source.source(), source.identity());
            assert_eq!(
                (origin.source.start(), origin.source.end()),
                (source_start, source_end)
            );
        }
        assert!(
            rows.next_group_origin(cursor.as_mut().unwrap())
                .unwrap()
                .is_none()
        );
        rows.release_group_origin_cursor(&mut cursor).unwrap();
        assert!(cursor.is_none());
        let mut value = Some(rows.materialize_group(group).unwrap());
        assert_eq!(
            value.as_ref().unwrap().text().code_units(),
            expected.value.encode_utf16().collect::<Vec<_>>()
        );
        rows.release_materialized_group(&mut value).unwrap();
        assert!(value.is_none());
    }
    assert!(completed.at_stop().is_none());
    let parts = completed.freeze_parts().unwrap();
    drop(completed);
    drop(attempt);
    let mut original = Some(owner.take_request().unwrap());
    let mut parts = Some(parts);
    let mut sealed =
        super::seal::SealedOrdinaryConstruction::take(&mut original, &mut parts).unwrap();
    assert_eq!(sealed.source().identity(), source.identity());
    assert_eq!(sealed.source().as_utf8(), Some(raw));
    assert!(sealed.is_offline());
    assert_eq!(
        sealed
            .final_usage()
            .resource(HtmlParseResource::SourceUnits)
            .current,
        raw.len() as u64
    );
    assert!(sealed.document().is_some());
    sealed.release().unwrap();
    assert_eq!(sealed.retained_usage().actual_capacity_bytes, 0);
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
    ] {
        assert_eq!(sealed.retained_usage().resource(resource).current, 0);
    }
}

#[test]
fn progressive_partitions_retain_qualified_ordered_attributes_d1_and_diagnostic_ranges() {
    use crate::construction::{
        evidence::{DiagnosticContext, DiagnosticKind, DiagnosticLocationRole, DiagnosticToken},
        provenance::ParserRule,
    };
    use crate::parser::tokenizer::token::{DiagnosticCode, DiagnosticPhase, OriginKind};
    // HTML877d comment NULL replacement, first duplicate attribute retention,
    // character references and foreign attribute/name adjustment. All offsets
    // below are literal UTF-8 source positions, not values from another run.
    let raw = "<!doctype html><!--\0--><p z='&amp;' a=X z=Y>&notin;</p><svg viewbox='0' xlink:href='&amp;'/>";
    let dump = "| <!DOCTYPE html>\n| <!-- � -->\n| <html>\n|   <head>\n|   <body>\n|     <p>\n|       a=\"X\"\n|       z=\"&\"\n|       \"∉\"\n|     <svg svg>\n|       viewBox=\"0\"\n|       xlink href=\"&\"\n";
    let attributes = [
        ProgressiveAttributeExpectation {
            path: &[2, 1, 0],
            attributes: &[(None, None, "z", "&"), (None, None, "a", "X")],
        },
        ProgressiveAttributeExpectation {
            path: &[2, 1, 1],
            attributes: &[
                (None, None, "viewBox", "0"),
                (
                    Some("http://www.w3.org/1999/xlink"),
                    Some("xlink"),
                    "href",
                    "&",
                ),
            ],
        },
    ];
    let fields = [
        ProgressiveFieldExpectation {
            path: &[1],
            field: ProgressiveNativeField::Comment,
            value: "�",
            origins: &[(0, 1, 19, 20, OriginKind::NullReplacement)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Prologue(prologue::PrologueMode::BeforeHtml),
            family: DiagnosticToken::Comment,
        },
        ProgressiveFieldExpectation {
            path: &[2, 1, 0],
            field: ProgressiveNativeField::AttributeName(0),
            value: "z",
            origins: &[(0, 1, 26, 27, OriginKind::Literal)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Tag,
        },
        ProgressiveFieldExpectation {
            path: &[2, 1, 0],
            field: ProgressiveNativeField::AttributeValue(0),
            value: "&",
            origins: &[(0, 1, 29, 34, OriginKind::CharacterReference)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Tag,
        },
        ProgressiveFieldExpectation {
            path: &[2, 1, 0, 0],
            field: ProgressiveNativeField::Text,
            value: "∉",
            origins: &[(0, 1, 44, 51, OriginKind::CharacterReference)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Text,
        },
        ProgressiveFieldExpectation {
            path: &[2, 1, 1],
            field: ProgressiveNativeField::AttributeValue(1),
            value: "&",
            origins: &[(0, 1, 84, 89, OriginKind::CharacterReference)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Tag,
        },
    ];
    let diagnostics = [
        ProgressiveDiagnosticExpectation {
            kind: DiagnosticKind::Tokenizer(DiagnosticCode::UnexpectedNullCharacter),
            context: DiagnosticContext::Tokenizer {
                phase: DiagnosticPhase::Tokenizer,
                state: State::Comment,
            },
            role: DiagnosticLocationRole::InputFrontier,
            token: None,
            source: (19, 20),
            anchor: "parse-error-unexpected-null-character",
            branch: "unexpected-null-character",
        },
        ProgressiveDiagnosticExpectation {
            kind: DiagnosticKind::Tokenizer(DiagnosticCode::DuplicateAttribute),
            context: DiagnosticContext::Tokenizer {
                phase: DiagnosticPhase::Tokenizer,
                state: State::AttributeName,
            },
            role: DiagnosticLocationRole::InputFrontier,
            token: None,
            source: (41, 42),
            anchor: "parse-error-duplicate-attribute",
            branch: "duplicate-attribute",
        },
    ];
    for cut in 0..=raw.len() {
        progressive_completed_native_evidence(
            raw,
            &[cut],
            dump,
            &attributes,
            &fields,
            &diagnostics,
        );
    }
    progressive_completed_native_evidence(
        raw,
        &(0..=raw.len()).collect::<Vec<_>>(),
        dump,
        &attributes,
        &fields,
        &diagnostics,
    );
}

#[test]
fn progressive_partitions_keep_fostered_native_destination_and_buffered_diagnostic_roles() {
    use crate::construction::{
        evidence::{DiagnosticContext, DiagnosticKind, DiagnosticLocationRole, DiagnosticToken},
        provenance::ParserRule,
    };
    use crate::parser::tokenizer::token::OriginKind;
    let raw = "<!doctype html><table>X<tr><td>A</table><p>B";
    let dump = "| <!DOCTYPE html>\n| <html>\n|   <head>\n|   <body>\n|     \"X\"\n|     <table>\n|       <tbody>\n|         <tr>\n|           <td>\n|             \"A\"\n|     <p>\n|       \"B\"\n";
    let fields = [
        ProgressiveFieldExpectation {
            path: &[1, 1, 0],
            field: ProgressiveNativeField::Text,
            value: "X",
            origins: &[(0, 1, 22, 23, OriginKind::Literal)],
            operation: crate::construction::provenance::ConstructionOperation::Replay,
            rule: ParserRule::Tree(head::InsertionMode::InTableText),
            family: DiagnosticToken::Text,
        },
        ProgressiveFieldExpectation {
            path: &[1, 1, 1, 0, 0, 0, 0],
            field: ProgressiveNativeField::Text,
            value: "A",
            origins: &[(0, 1, 31, 32, OriginKind::Literal)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Text,
        },
    ];
    let diagnostics = [
        ProgressiveDiagnosticExpectation {
            kind: DiagnosticKind::Tree(head::HeadCondition::NonWhitespaceInTable),
            context: DiagnosticContext::Tree(head::InsertionMode::InTableText),
            role: DiagnosticLocationRole::BufferedTableCharacter,
            token: Some(DiagnosticToken::Text),
            source: (22, 23),
            anchor: "parsing-main-intabletext",
            branch: "pending-characters/non-whitespace",
        },
        ProgressiveDiagnosticExpectation {
            kind: DiagnosticKind::Tree(head::HeadCondition::UnexpectedTableToken),
            context: DiagnosticContext::Tree(head::InsertionMode::InTableText),
            role: DiagnosticLocationRole::BufferedTableCharacter,
            token: Some(DiagnosticToken::Text),
            source: (22, 23),
            anchor: "parsing-main-intable",
            branch: "anything-else/pending-character",
        },
    ];
    for cut in 0..=raw.len() {
        progressive_completed_native_evidence(raw, &[cut], dump, &[], &fields, &diagnostics);
    }
    progressive_completed_native_evidence(
        raw,
        &(0..=raw.len()).collect::<Vec<_>>(),
        dump,
        &[],
        &fields,
        &diagnostics,
    );
}

#[test]
fn progressive_partitions_keep_template_fragment_native_and_source_destinations() {
    use crate::construction::{evidence::DiagnosticToken, provenance::ParserRule};
    use crate::parser::tokenizer::token::OriginKind;
    let raw = "<!doctype html><template><tr><td>A</template><p>B";
    let dump = "| <!DOCTYPE html>\n| <html>\n|   <head>\n|     <template>\n|       content\n|         <tr>\n|           <td>\n|             \"A\"\n|   <body>\n|     <p>\n|       \"B\"\n";
    let fields = [ProgressiveFieldExpectation {
        path: &[1, 0, 0, usize::MAX, 0, 0, 0],
        field: ProgressiveNativeField::Text,
        value: "A",
        origins: &[(0, 1, 33, 34, OriginKind::Literal)],
        operation: crate::construction::provenance::ConstructionOperation::Token,
        rule: ParserRule::Tree(head::InsertionMode::InBody),
        family: DiagnosticToken::Text,
    }];
    for cut in 0..=raw.len() {
        progressive_completed_native_evidence(raw, &[cut], dump, &[], &fields, &[]);
    }
    progressive_completed_native_evidence(
        raw,
        &(0..=raw.len()).collect::<Vec<_>>(),
        dump,
        &[],
        &fields,
        &[],
    );
}

#[test]
fn progressive_partitions_keep_selectedcontent_copy_groups_and_distinct_native_destinations() {
    use crate::construction::{evidence::DiagnosticToken, provenance::ParserRule};
    use crate::parser::tokenizer::token::OriginKind;
    let raw = "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A<b>B</b></option></select><p>C";
    let dump = "| <!DOCTYPE html>\n| <html>\n|   <head>\n|   <body>\n|     <select>\n|       <button>\n|         <selectedcontent>\n|           \"A\"\n|           <b>\n|             \"B\"\n|       <option>\n|         selected=\"\"\n|         \"A\"\n|         <b>\n|           \"B\"\n|     <p>\n|       \"C\"\n";
    let fields = [
        ProgressiveFieldExpectation {
            path: &[1, 1, 0, 0, 0, 0],
            field: ProgressiveNativeField::Text,
            value: "A",
            origins: &[(0, 1, 92, 93, OriginKind::Literal)],
            operation: crate::construction::provenance::ConstructionOperation::Copy,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Text,
        },
        ProgressiveFieldExpectation {
            path: &[1, 1, 0, 0, 0, 1, 0],
            field: ProgressiveNativeField::Text,
            value: "B",
            origins: &[(0, 1, 96, 97, OriginKind::Literal)],
            operation: crate::construction::provenance::ConstructionOperation::Copy,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Text,
        },
        ProgressiveFieldExpectation {
            path: &[1, 1, 0, 1, 0],
            field: ProgressiveNativeField::Text,
            value: "A",
            origins: &[(0, 1, 92, 93, OriginKind::Literal)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Text,
        },
        ProgressiveFieldExpectation {
            path: &[1, 1, 0, 1, 1, 0],
            field: ProgressiveNativeField::Text,
            value: "B",
            origins: &[(0, 1, 96, 97, OriginKind::Literal)],
            operation: crate::construction::provenance::ConstructionOperation::Token,
            rule: ParserRule::Tree(head::InsertionMode::InBody),
            family: DiagnosticToken::Text,
        },
    ];
    for cut in 0..=raw.len() {
        progressive_completed_native_evidence(raw, &[cut], dump, &[], &fields, &[]);
    }
    progressive_completed_native_evidence(
        raw,
        &(0..=raw.len()).collect::<Vec<_>>(),
        dump,
        &[],
        &fields,
        &[],
    );
}

#[test]
fn progressive_rawtext_rcdata_script_escaped_endings_and_crlf_handoffs_preserve_native_text() {
    for (raw, expected) in [
        (
            "<title>A&amp;</TiTlEextra!></TiTlE><p>B",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (2, "title"),
                (3, "#A&</TiTlEextra!>"),
                (1, "body"),
                (2, "p"),
                (3, "#B"),
            ]),
        ),
        (
            "<style>A&amp;</styLe><p>B",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (2, "style"),
                (3, "#A&amp;"),
                (1, "body"),
                (2, "p"),
                (3, "#B"),
            ]),
        ),
        (
            "<script><!--<script>x</script>--></script>\r\n<p>B",
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (2, "script"),
                (3, "#<!--<script>x</script>-->"),
                (2, "#\n"),
                (1, "body"),
                (2, "p"),
                (3, "#B"),
            ]),
        ),
    ] {
        for cut in 0..=raw.len() {
            assert_eq!(
                progressive_driver_document(raw, &[cut]),
                expected,
                "{raw:?} cut={cut}"
            );
        }
    }
}

fn progressive_driver_limits(
    caps: HtmlParseLimits,
) -> (Option<ParseStop>, crate::parser::control::ParseUsage) {
    let raw = "<!doctype html><p>&unknown;&#x41;</p>";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), caps);
    let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
    let mut attempt = owner.begin_progressive().unwrap();
    let mut driver = match attempt.take_driver() {
        Ok(driver) => driver,
        Err(_) => {
            let at = attempt.at_stop().unwrap().clone();
            attempt.retain_failure().unwrap();
            drop(attempt);
            owner.release_failure().unwrap();
            let usage = owner.retained_usage().unwrap();
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::ManagedCapacityBytes,
                HtmlParseResource::AuxiliaryEntries,
                HtmlParseResource::OpenElements,
            ] {
                assert_eq!(usage.resource(resource).current, 0);
            }
            return (
                Some(match owner.failure().unwrap() {
                    super::entry::InitializationFailure::Stopped(stop) => stop,
                    _ => panic!("execution refusal"),
                }),
                at,
            );
        }
    };
    let result = (|| -> Result<(), StandaloneError> {
        for cut in [raw.len() / 2, raw.len()] {
            driver.make_available(cut)?;
            loop {
                match driver.step()? {
                    head::HeadStep::Consumed => {}
                    head::HeadStep::NeedInput => break,
                    other => panic!("unsealed {other:?}"),
                }
            }
        }
        driver.seal_input()?;
        loop {
            match driver.step()? {
                head::HeadStep::Consumed => {}
                head::HeadStep::Complete => break,
                other => panic!("sealed {other:?}"),
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let first = driver.control.stop().unwrap();
        let at = driver.control.first_stop_usage().unwrap().clone();
        assert!(driver.step().is_err());
        assert!(driver.make_available(raw.len()).is_err());
        assert!(driver.seal_input().is_err());
        assert_eq!(driver.control.first_stop_usage(), Some(&at));
        assert_eq!(driver.control.stop(), Some(first));
        let mut slot = Some(driver);
        assert!(StandaloneDocumentDriver::take_completed(&mut slot).is_err());
        driver = slot.take().unwrap();
    }
    if driver.release().is_err() {
        let first = driver.control.first_stop_usage().unwrap().clone();
        driver.release().unwrap();
        assert_eq!(driver.control.first_stop_usage(), Some(&first));
    }
    assert_released(&driver.control);
    (driver.control.stop(), driver.control.usage().clone())
}
#[test]
fn progressive_ordinary_completion_has_exact_work_storage_and_fixed_layout_refusals() {
    let (stop, usage) = progressive_driver_limits(limits(100));
    assert_eq!(stop, None);
    let work = usage.resource(HtmlParseResource::WorkCredits).current;
    assert_eq!(
        progressive_driver_limits(
            limits(100)
                .into_builder()
                .work_credits(work)
                .finish()
                .unwrap()
        )
        .0,
        None
    );
    assert!(matches!(
        progressive_driver_limits(
            limits(100)
                .into_builder()
                .work_credits(work - 1)
                .finish()
                .unwrap()
        )
        .0,
        Some(ParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    for limit in [0, 1, 512, 2048, 8192, 16384] {
        assert!(matches!(
            progressive_driver_limits(
                limits(100)
                    .into_builder()
                    .work_credits(limit)
                    .finish()
                    .unwrap()
            )
            .0,
            Some(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
    }
    for (resource, peak) in [
        (
            HtmlParseResource::ManagedCapacityBytes,
            usage
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water,
        ),
        (
            HtmlParseResource::OpenElements,
            usage.resource(HtmlParseResource::OpenElements).high_water,
        ),
        (
            HtmlParseResource::AtomicLayoutBytes,
            usage.largest_atomic_layout,
        ),
    ] {
        let caps = |limit| {
            let b = limits(100).into_builder();
            match resource {
                HtmlParseResource::ManagedCapacityBytes => b.managed_capacity_bytes(limit),
                HtmlParseResource::OpenElements => b.open_elements(limit),
                HtmlParseResource::AtomicLayoutBytes => b.atomic_layout_bytes(limit),
                _ => unreachable!(),
            }
            .finish()
            .unwrap()
        };
        assert_eq!(
            progressive_driver_limits(caps(peak)).0,
            None,
            "{resource:?}"
        );
        assert!(
            matches!(progressive_driver_limits(caps(peak-1)).0,Some(ParseStop::LimitExceeded {resource:actual,..}) if actual==resource),
            "{resource:?}"
        );
    }
}

#[test]
fn progressive_original_cancellation_before_initialization_and_immediately_before_seal_is_terminal()
{
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), limits(100));
    request.cancellation().cancel();
    let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
    let mut attempt = owner.begin_progressive().unwrap();
    assert!(attempt.take_driver().is_err());
    assert_eq!(
        attempt.at_stop().unwrap().work(WorkCategory::InputUnitRead),
        0
    );
    attempt.retain_failure().unwrap();
    drop(attempt);
    let first = owner.at_stop().unwrap().clone();
    owner.release_failure().unwrap();
    assert_eq!(owner.at_stop(), Some(&first));
    for resource in [
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::OpenElements,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::Diagnostics,
    ] {
        assert_eq!(
            owner.after_cleanup().unwrap().resource(resource).current,
            0,
            "{resource:?}"
        );
    }

    let raw = "<p>&notin;";
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let cancellation = request.cancellation().clone();
    let mut owner = super::seal::OrdinaryInitializationOwner::new(request);
    let mut attempt = owner.begin_progressive().unwrap();
    let mut driver = attempt.take_driver().unwrap();
    driver.make_available(raw.len()).unwrap();
    while driver.step().unwrap() != head::HeadStep::NeedInput {}
    let reads = driver.control.usage().work(WorkCategory::InputUnitRead);
    cancellation.cancel();
    assert!(matches!(
        driver.seal_input(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    let first = driver.control.first_stop_usage().unwrap().clone();
    assert!(driver.step().is_err());
    assert!(driver.seal_input().is_err());
    assert_eq!(
        driver.control.usage().work(WorkCategory::InputUnitRead),
        reads
    );
    assert_eq!(driver.control.first_stop_usage(), Some(&first));
    let mut slot = Some(driver);
    assert!(StandaloneDocumentDriver::take_completed(&mut slot).is_err());
    let mut driver = slot.take().unwrap();
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.first_stop_usage(), Some(&first));
}

#[test]
fn radio_table_delegation_fosters_real_void_inputs_and_preserves_reset_default_on() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><table><input type=radio name=g checked value='A&#13;B'><input type=radio name=g checked></table>Z",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    assert_eq!(
        outline(document),
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "input"),
            (2, "input"),
            (2, "table"),
            (2, "#Z")
        ])
    );
    let html = document.root().children().nth(1).unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let inputs: Vec<_> = document.node(&body).unwrap().children().take(2).collect();
    assert!(
        !frozen
            .input_state(&mut c, &inputs[0])
            .unwrap()
            .flags
            .checkedness()
    );
    assert!(
        frozen
            .input_state(&mut c, &inputs[1])
            .unwrap()
            .flags
            .checkedness()
    );
    let mut value = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
    frozen
        .append_input_value(&mut c, &inputs[0], &mut value)
        .unwrap();
    assert_eq!(value.units(), [65, 13, 66]);
    value.release(&mut c).unwrap();
    drop(inputs);
    drop((body, html));
    frozen.release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn ordinary_multiple_email_retains_produced_counts_distinct_from_joined_current() {
    for (value, expected, count) in [
        ("", "", 0),
        (" ", "", 1),
        ("a@b.test,", "a@b.test", 1),
        ("a@b.test, ", "a@b.test,", 2),
        (",,", ",", 2),
        (", , ", ",,", 3),
    ] {
        let raw = format!(
            "<!doctype html><form id=f><input multiple=false type=email value='{value}'>Z</form>"
        );
        let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
        let mut driver = super::entry::initialized_driver(&request);
        assert_eq!(
            standalone_boundary(&mut driver),
            super::head::HeadStep::Complete
        );
        let (mut control, mut frozen) = body_document(driver);
        let document = frozen.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let form = document.node(&body).unwrap().children().next().unwrap();
        let input = document.node(&form).unwrap().children().next().unwrap();
        let state = frozen.input_state(&mut control, &input).unwrap();
        let owner = frozen.form_association_owner(&mut control, &input).unwrap();
        let owned = owner.as_ref() == Some(&form);
        drop(owner);
        let mut buffer = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
        frozen
            .append_input_value(&mut control, &input, &mut buffer)
            .unwrap();
        let current = buffer.units().to_vec();
        buffer.clear(&mut control).unwrap();
        frozen
            .append_input_default_value(&mut control, &input, &mut buffer)
            .unwrap();
        let default = buffer.units().to_vec();
        buffer.release(&mut control).unwrap();
        drop(input);
        drop(form);
        drop(body);
        drop(html);
        frozen.release(&mut control).unwrap();
        assert_released(&control);
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(control.stop(), None);
        assert!(owned);
        assert_eq!(state.email_value_count, Some(count), "{value:?}");
        assert_eq!(state.flags.input_type(), crate::HtmlInputType::Email);
        assert!(!state.flags.input_type().is_selectable());
        assert!(!state.flags.dirty_value());
        assert_eq!(current, expected.encode_utf16().collect::<Vec<_>>());
        assert_eq!(default, value.encode_utf16().collect::<Vec<_>>());
    }
}
fn ordinary_default_button_input_observes_ordered_values(
    raw: &'static str,
    expected_type: crate::HtmlInputType,
    type_units: &[u16],
) {
    // HTML877d Default mode returns the actual value attribute or empty.
    // Value-before-type first uses Text sanitization, then Value→Default
    // writes the nonempty current value back; type-before-value preserves CR/LF.
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let mut driver = super::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.stop(), first_stop);
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "ordinary Button/Reset/Submit must complete before native Default values are observed"
        );
        return;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let child_count = children.len();
    let mut attributes = Vec::new();
    let mut names = Vec::new();
    let mut states = Vec::new();
    let mut owners = Vec::new();
    let mut input_child_counts = Vec::new();
    let mut current_values = Vec::new();
    let mut internal_values = Vec::new();
    let mut default_values = Vec::new();
    for input in &children[..4] {
        let node = document.node(input).unwrap();
        input_child_counts.push(node.children().count());
        let HtmlNodeKind::Element(element) = node.kind() else {
            panic!("the actual void input must remain a native element");
        };
        names.push((
            element.name().local_name().code_units().to_vec(),
            element
                .name()
                .namespace()
                .uri()
                .unwrap()
                .code_units()
                .to_vec(),
        ));
        attributes.push(
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
                .collect::<Vec<_>>(),
        );
        let owner = frozen.form_association_owner(&mut control, input).unwrap();
        owners.push(owner.as_ref() == Some(&form));
        drop(owner);
        states.push(frozen.input_state(&mut control, input).unwrap());
        let mut value = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
        frozen
            .append_input_value(&mut control, input, &mut value)
            .unwrap();
        current_values.push(value.units().to_vec());
        value.clear(&mut control).unwrap();
        frozen
            .append_input_internal_value(&mut control, input, &mut value)
            .unwrap();
        internal_values.push(value.units().to_vec());
        value.clear(&mut control).unwrap();
        frozen
            .append_input_default_value(&mut control, input, &mut value)
            .unwrap();
        default_values.push(value.units().to_vec());
        value.release(&mut control).unwrap();
    }
    let HtmlNodeKind::Text(text) = document.node(&children[4]).unwrap().kind() else {
        panic!("Z must follow all four void inputs as an actual form child");
    };
    let following_text = text.code_units().to_vec();
    let original_source = {
        let evidence = frozen.retained_evidence().unwrap();
        let (_, parts, _) = evidence.lexical_observations();
        !parts.is_empty()
            && parts
                .iter()
                .all(|part| part.source.source() == request.source().identity())
    };
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), None);

    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "input"),
            (3, "input"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 5);
    assert_eq!(input_child_counts, vec![0; 4]);
    assert_eq!(
        names,
        vec![
            (
                vec![105, 110, 112, 117, 116],
                "http://www.w3.org/1999/xhtml"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            );
            4
        ]
    );
    let type_attribute = (None, vec![116, 121, 112, 101], type_units.to_vec());
    let value_name = vec![118, 97, 108, 117, 101];
    assert_eq!(
        attributes,
        vec![
            vec![
                type_attribute.clone(),
                (None, value_name.clone(), vec![13, 10, 88])
            ],
            vec![(None, value_name.clone(), vec![88]), type_attribute.clone()],
            vec![type_attribute.clone()],
            vec![type_attribute, (None, value_name, vec![])],
        ]
    );
    let expected_values = vec![vec![13, 10, 88], vec![88], vec![], vec![]];
    assert_eq!(current_values, expected_values);
    assert_eq!(internal_values, expected_values);
    assert_eq!(default_values, expected_values);
    assert_eq!(owners, vec![true; 4]);
    assert_eq!(following_text, vec![90]);
    assert!(original_source);
    for state in states {
        assert_eq!(state.flags.input_type(), expected_type);
        assert_eq!(
            state.flags.input_type().value_mode(),
            crate::HtmlInputValueMode::Default
        );
        assert_eq!(
            state.flags.input_type().sanitization(),
            crate::HtmlInputValueSanitization::String(crate::HtmlInputSanitization::Preserve)
        );
        assert!(!state.flags.dirty_value());
        assert!(!state.flags.dirty_checkedness());
        assert!(!state.flags.checkedness());
        assert!(!state.flags.indeterminate());
        assert!(!state.flags.user_validity());
        assert!(state.selected_files_empty);
        assert_eq!(state.cursor, 0);
    }
}

#[test]
fn ordinary_default_button_input_button_preserves_order_and_empty_default_through_eof() {
    ordinary_default_button_input_observes_ordered_values(
        "<!doctype html><form id=f><input type=button value='&#13;&#10;X'><input value='&#13;&#10;X' type=button><input type=button><input type=button value=''>Z</form>",
        crate::HtmlInputType::Button,
        &[98, 117, 116, 116, 111, 110],
    );
}

#[test]
fn ordinary_default_button_input_reset_preserves_order_and_empty_default_through_eof() {
    ordinary_default_button_input_observes_ordered_values(
        "<!doctype html><form id=f><input type=reset value='&#13;&#10;X'><input value='&#13;&#10;X' type=reset><input type=reset><input type=reset value=''>Z</form>",
        crate::HtmlInputType::Reset,
        &[114, 101, 115, 101, 116],
    );
}

#[test]
fn ordinary_default_button_input_submit_preserves_order_and_empty_default_through_eof() {
    ordinary_default_button_input_observes_ordered_values(
        "<!doctype html><form id=f><input type=submit value='&#13;&#10;X'><input value='&#13;&#10;X' type=submit><input type=submit><input type=submit value=''>Z</form>",
        crate::HtmlInputType::Submit,
        &[115, 117, 98, 109, 105, 116],
    );
}

fn ordinary_temporal_input_observes_completed_value(
    raw: &str,
    expected_type: crate::HtmlInputType,
    expected_default: &[u16],
    expected_current: &[u16],
    expected_keyword: &str,
    expected_reference_origins: &[(usize, usize, usize, usize)],
) -> super::head::HeadStep {
    use crate::construction::provenance::LexicalRole;
    use crate::parser::tokenizer::token::OriginKind;

    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let source = request.source();
    let mut driver = super::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.stop(), first_stop);
        assert!(first_stop.is_none());
        assert!(driver.control.first_stop_usage().is_none());
        return boundary;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let child_count = children.len();
    let input = children[0].clone();
    let input_children = document.node(&input).unwrap().children().count();
    let HtmlNodeKind::Element(element) = document.node(&input).unwrap().kind() else {
        panic!("the actual input token must create a native Element");
    };
    let namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let local_name = element.name().local_name().code_units().to_vec();
    let actual_attributes: Vec<_> = element
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
        .collect();
    let HtmlNodeKind::Text(text) = document.node(&children[1]).unwrap().kind() else {
        panic!("Z remains an actual sibling after the void input");
    };
    let following_text = text.code_units().to_vec();
    let owner = frozen.form_association_owner(&mut control, &input).unwrap();
    let owner_is_form = owner.as_ref() == Some(&form);
    drop(owner);
    let state = frozen.input_state(&mut control, &input).unwrap();
    let actual_type = state.flags.input_type();
    let mode_of_value = state.flags.input_type().value_mode();
    let selectable = state.flags.input_type().is_selectable();
    let flags = (
        state.flags.dirty_value(),
        state.flags.dirty_checkedness(),
        state.flags.checkedness(),
        state.flags.indeterminate(),
        state.flags.user_validity(),
    );
    let writeback_absent = frozen
        .input_writeback_sources(&mut control, &input)
        .unwrap()
        .is_none();
    let mut value = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
    frozen
        .append_input_internal_value(&mut control, &input, &mut value)
        .unwrap();
    let internal_value = value.units().to_vec();
    value.clear(&mut control).unwrap();
    frozen
        .append_input_value(&mut control, &input, &mut value)
        .unwrap();
    let current_value = value.units().to_vec();
    value.clear(&mut control).unwrap();
    frozen
        .append_input_default_value(&mut control, &input, &mut value)
        .unwrap();
    let default_value = value.units().to_vec();
    value.release(&mut control).unwrap();
    let (reference_origins, original_source) = {
        let evidence = frozen.retained_evidence().unwrap();
        let (_, parts, _) = evidence.lexical_observations();
        let references: Vec<_> = parts
            .iter()
            .filter(|part| {
                part.role == LexicalRole::Value && part.kind == Some(OriginKind::CharacterReference)
            })
            .collect();
        let original_source = references
            .iter()
            .all(|part| part.source.source() == source.identity());
        (
            references
                .iter()
                .map(|part| (part.start, part.end, part.source.start(), part.source.end()))
                .collect::<Vec<_>>(),
            original_source,
        )
    };
    drop(input);
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 2);
    assert_eq!(input_children, 0);
    assert_eq!(
        namespace,
        "http://www.w3.org/1999/xhtml"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(local_name, vec![105, 110, 112, 117, 116]);
    let mut expected_attributes = vec![(
        None,
        vec![116, 121, 112, 101],
        expected_keyword.encode_utf16().collect::<Vec<_>>(),
    )];
    expected_attributes.push((
        None,
        vec![118, 97, 108, 117, 101],
        expected_default.to_vec(),
    ));
    assert_eq!(actual_attributes, expected_attributes);
    assert_eq!(following_text, vec![90]);
    assert!(owner_is_form && writeback_absent && original_source);
    assert_eq!(actual_type, expected_type);
    assert_eq!(mode_of_value, crate::HtmlInputValueMode::Value);
    assert_eq!(flags, (false, false, false, false, false));
    assert!(!selectable);
    assert_eq!(default_value, expected_default);
    assert_eq!(reference_origins, expected_reference_origins);
    assert_eq!(internal_value, expected_current);
    assert_eq!(current_value, expected_current);
    boundary
}

// HTML877d strict input sanitizers, rather than common microsyntax recovery:
// Date35975–77, Month36146–48, Week36300–02, Time36451–53,
// LocalDateTime36607–09. Each attempted form is released before the
// completion oracle. A Boundary(Form) leaves later native/value assertions
// unexecuted; the full matrix still exercises and cleans up every fixture.
type TemporalInputLiteral<'a> = (
    &'a str,
    &'a str,
    &'a str,
    &'a [(usize, usize, usize, usize)],
);

fn ordinary_temporal_input_literal_matrix(
    keyword: &str,
    input_type: crate::HtmlInputType,
    cases: &[TemporalInputLiteral<'_>],
) {
    let mut boundaries = Vec::new();
    for &(authored, native_default, current, reference_origins) in cases {
        let raw =
            format!("<!doctype html><form id=f><input type={keyword} value='{authored}'>Z</form>");
        let boundary = ordinary_temporal_input_observes_completed_value(
            &raw,
            input_type,
            &native_default.encode_utf16().collect::<Vec<_>>(),
            &current.encode_utf16().collect::<Vec<_>>(),
            keyword,
            reference_origins,
        );
        boundaries.push((authored, boundary));
    }
    for (authored, boundary) in boundaries {
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "{keyword} value={authored:?} must complete before its sanitized state is observed"
        );
    }
}

#[test]
fn ordinary_temporal_input_date_uses_strict_gregorian_validity_and_keeps_year_spelling() {
    ordinary_temporal_input_literal_matrix(
        "date",
        crate::HtmlInputType::Date,
        &[
            ("2024-02-29", "2024-02-29", "2024-02-29", &[]),
            ("2023-02-29", "2023-02-29", "", &[]),
            ("002024-02-29", "002024-02-29", "002024-02-29", &[]),
            (
                "10000000000000000000000000000000000000000000400-02-29",
                "10000000000000000000000000000000000000000000400-02-29",
                "10000000000000000000000000000000000000000000400-02-29",
                &[],
            ),
            ("0000-01-01", "0000-01-01", "", &[]),
        ],
    );
}

#[test]
fn ordinary_temporal_input_month_preserves_valid_values_and_empties_invalid_months() {
    ordinary_temporal_input_literal_matrix(
        "month",
        crate::HtmlInputType::Month,
        &[
            ("2024-12", "2024-12", "2024-12", &[]),
            ("2024-13", "2024-13", "", &[]),
            ("", "", "", &[]),
        ],
    );
}

#[test]
fn ordinary_temporal_input_week_checks_year_specific_week53_and_uppercase_w() {
    ordinary_temporal_input_literal_matrix(
        "week",
        crate::HtmlInputType::Week,
        &[
            ("2020-W53", "2020-W53", "2020-W53", &[]),
            ("2021-W53", "2021-W53", "", &[]),
            ("2020-w53", "2020-w53", "", &[]),
            ("2020-W00", "2020-W00", "", &[]),
        ],
    );
}

#[test]
fn ordinary_temporal_input_time_requires_strict_fraction_precision_without_normalizing() {
    ordinary_temporal_input_literal_matrix(
        "time",
        crate::HtmlInputType::Time,
        &[
            ("12:34:00.000", "12:34:00.000", "12:34:00.000", &[]),
            ("12:34:56.123", "12:34:56.123", "12:34:56.123", &[]),
            ("12:34:56.1234", "12:34:56.1234", "", &[]),
            ("24:00", "24:00", "", &[]),
            ("23:59:60", "23:59:60", "", &[]),
        ],
    );
}

#[test]
fn ordinary_temporal_input_local_datetime_normalizes_separator_and_shortest_time_only() {
    ordinary_temporal_input_literal_matrix(
        "datetime-local",
        crate::HtmlInputType::LocalDateTime,
        &[
            (
                "2024-02-29 12:34:00.000",
                "2024-02-29 12:34:00.000",
                "2024-02-29T12:34",
                &[],
            ),
            (
                "2024-02-29&#32;12:34:00.000",
                "2024-02-29 12:34:00.000",
                "2024-02-29T12:34",
                &[(10, 11, 70, 75)],
            ),
            (
                "002024-02-29T12:34:01.120",
                "002024-02-29T12:34:01.120",
                "002024-02-29T12:34:01.12",
                &[],
            ),
            (
                "2024-02-29T12:34:00.001",
                "2024-02-29T12:34:00.001",
                "2024-02-29T12:34:00.001",
                &[],
            ),
            ("2024-02-29t12:34", "2024-02-29t12:34", "", &[]),
            ("2024-02-29T12:34Z", "2024-02-29T12:34Z", "", &[]),
        ],
    );
}

#[test]
fn default_button_input_table_delegation_fosters_void_nodes_with_preserved_values_and_owner() {
    for (keyword, kind) in [
        ("button", crate::HtmlInputType::Button),
        ("reset", crate::HtmlInputType::Reset),
        ("submit", crate::HtmlInputType::Submit),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(format!(
                "<!doctype html><form id=f><table><input type={keyword} value='A&#13;B&#10;C'></table>Z</form>"
            )),
            limits(100),
        );
        let mut driver = super::entry::initialized_driver(&request);
        let boundary = standalone_boundary(&mut driver);
        if boundary != super::head::HeadStep::Complete {
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(boundary, super::head::HeadStep::Complete);
            continue;
        }
        let (mut c, mut frozen) = body_document(driver);
        let doc = frozen.document().unwrap();
        let actual_outline = outline(doc);
        let html = doc
            .root()
            .children()
            .find(|id| matches!(doc.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = doc.node(&html).unwrap().children().nth(1).unwrap();
        let form = doc.node(&body).unwrap().children().next().unwrap();
        let children: Vec<_> = doc.node(&form).unwrap().children().collect();
        let child_count = children.len();
        let input_children = doc.node(&children[0]).unwrap().children().count();
        let state = frozen.input_state(&mut c, &children[0]).unwrap();
        let owner = frozen.form_association_owner(&mut c, &children[0]).unwrap();
        let owns = owner.as_ref() == Some(&form);
        drop(owner);
        let mut buffer = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
        frozen
            .append_input_value(&mut c, &children[0], &mut buffer)
            .unwrap();
        let value = buffer.units().to_vec();
        buffer.release(&mut c).unwrap();
        let HtmlNodeKind::Text(text) = doc.node(&children[2]).unwrap().kind() else {
            panic!("following actual text")
        };
        let following_text = text.code_units().to_vec();
        drop(children);
        drop(form);
        drop(body);
        drop(html);
        frozen.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        assert_eq!(c.stop(), None);
        assert_eq!(
            actual_outline,
            expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "form"),
                (3, "input"),
                (3, "table"),
                (3, "#Z")
            ])
        );
        assert_eq!(child_count, 3);
        assert_eq!(input_children, 0);
        assert_eq!(state.flags.input_type(), kind);
        assert_eq!(value, [65, 13, 66, 10, 67]);
        assert!(owns);
        assert_eq!(following_text, [90]);
    }
}

fn ordinary_number_input_observes_complete(
    source: HtmlSource,
    expected_attributes: &[(&str, &[u16])],
    expected_default: &[u16],
    expected_current: &[u16],
    expected_references: &[(usize, usize, usize, usize)],
) -> super::head::HeadStep {
    use crate::construction::provenance::LexicalRole;
    use crate::parser::tokenizer::token::OriginKind;

    let request = crate::HtmlDocumentParseRequest::offline(source, limits(100));
    let mut driver = super::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.stop(), first_stop);
        assert!(first_stop.is_none());
        assert!(driver.control.first_stop_usage().is_none());
        return boundary;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let child_count = children.len();
    let input = children[0].clone();
    let input_children = document.node(&input).unwrap().children().count();
    let HtmlNodeKind::Element(element) = document.node(&input).unwrap().kind() else {
        panic!("input must be an actual native element");
    };
    let namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let local = element.name().local_name().code_units().to_vec();
    let attributes: Vec<_> = element
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
        .collect();
    let HtmlNodeKind::Text(text) = document.node(&children[1]).unwrap().kind() else {
        panic!("Z must remain an actual sibling after the void input");
    };
    let following_text = text.code_units().to_vec();
    let owner = frozen.form_association_owner(&mut control, &input).unwrap();
    let owner_is_form = owner.as_ref() == Some(&form);
    drop(owner);
    let state = frozen.input_state(&mut control, &input).unwrap();
    let actual_type = state.flags.input_type();
    let value_mode = state.flags.input_type().value_mode();
    let selectable = state.flags.input_type().is_selectable();
    let flags = (
        state.flags.dirty_value(),
        state.flags.dirty_checkedness(),
        state.flags.checkedness(),
        state.flags.indeterminate(),
        state.flags.user_validity(),
    );
    let files_empty = state.selected_files_empty;
    let writeback_absent = frozen
        .input_writeback_sources(&mut control, &input)
        .unwrap()
        .is_none();
    let mut buffer = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
    frozen
        .append_input_internal_value(&mut control, &input, &mut buffer)
        .unwrap();
    let internal = buffer.units().to_vec();
    buffer.clear(&mut control).unwrap();
    frozen
        .append_input_value(&mut control, &input, &mut buffer)
        .unwrap();
    let current = buffer.units().to_vec();
    buffer.clear(&mut control).unwrap();
    frozen
        .append_input_default_value(&mut control, &input, &mut buffer)
        .unwrap();
    let default = buffer.units().to_vec();
    buffer.release(&mut control).unwrap();
    let (references, source_qualified) = {
        let evidence = frozen.retained_evidence().unwrap();
        let (_, parts, _) = evidence.lexical_observations();
        let source_qualified = parts
            .iter()
            .all(|part| part.source.source() == request.source().identity());
        let references = parts
            .iter()
            .filter(|part| {
                part.role == LexicalRole::Value && part.kind == Some(OriginKind::CharacterReference)
            })
            .map(|part| (part.start, part.end, part.source.start(), part.source.end()))
            .collect::<Vec<_>>();
        (references, source_qualified)
    };
    drop(input);
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 2);
    assert_eq!(input_children, 0);
    assert_eq!(
        namespace,
        "http://www.w3.org/1999/xhtml"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(local, [105, 110, 112, 117, 116]);
    assert_eq!(
        attributes,
        expected_attributes
            .iter()
            .map(|(name, value)| (
                None,
                name.encode_utf16().collect::<Vec<_>>(),
                value.to_vec()
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(following_text, [90]);
    assert!(owner_is_form && files_empty && writeback_absent && source_qualified);
    assert_eq!(actual_type, crate::HtmlInputType::Number);
    assert_eq!(value_mode, crate::HtmlInputValueMode::Value);
    assert!(!selectable);
    assert_eq!(flags, (false, false, false, false, false));
    assert_eq!(default, expected_default);
    assert_eq!(internal, expected_current);
    assert_eq!(current, expected_current);
    assert_eq!(references, expected_references);
    boundary
}

// HTML877d36776 uses complete valid-floating-point grammar (3209+), not
// prefix recovery or binary64 representability. Each real fixture disposes
// before the final completion oracle, so the whole matrix runs at RED.
fn ordinary_number_input_literal_matrix(cases: &[(&str, &str)]) {
    let keyword: Vec<_> = "number".encode_utf16().collect();
    let mut boundaries = Vec::new();
    for &(raw_value, expected_current) in cases {
        let value: Vec<_> = raw_value.encode_utf16().collect();
        let current: Vec<_> = expected_current.encode_utf16().collect();
        for value_first in [false, true] {
            let attrs = if value_first {
                format!("value='{raw_value}' type=number")
            } else {
                format!("type=number value='{raw_value}'")
            };
            let raw = format!("<!doctype html><form id=f><input {attrs}>Z</form>");
            let expected = if value_first {
                vec![("value", value.as_slice()), ("type", keyword.as_slice())]
            } else {
                vec![("type", keyword.as_slice()), ("value", value.as_slice())]
            };
            let boundary = ordinary_number_input_observes_complete(
                HtmlSource::utf8(raw),
                &expected,
                &value,
                &current,
                &[],
            );
            boundaries.push((raw_value, value_first, boundary));
        }
    }
    for (value, value_first, boundary) in boundaries {
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "Number value={value:?}, value-first={value_first}"
        );
    }
}

#[test]
fn ordinary_number_input_complete_author_numbers_preserve_exact_units_even_on_float_overflow() {
    ordinary_number_input_literal_matrix(&[
        (".5", ".5"),
        ("-0", "-0"),
        ("1e+2", "1e+2"),
        ("1e999", "1e999"),
        ("001.00", "001.00"),
    ]);
}

#[test]
fn ordinary_number_input_invalid_complete_forms_empty_without_prefix_recovery_or_trim() {
    ordinary_number_input_literal_matrix(&[
        ("+1", ""),
        ("1.", ""),
        ("12junk", ""),
        (" 1", ""),
        ("1 ", ""),
        ("1e", ""),
        ("1e+", ""),
        ("NaN", ""),
        ("Infinity", ""),
    ]);
}

#[test]
fn ordinary_number_input_absent_and_present_empty_defaults_remain_distinct_native_attributes() {
    let keyword: Vec<_> = "number".encode_utf16().collect();
    let absent = ordinary_number_input_observes_complete(
        HtmlSource::utf8("<!doctype html><form id=f><input type=number>Z</form>"),
        &[("type", &keyword)],
        &[],
        &[],
        &[],
    );
    let empty = ordinary_number_input_observes_complete(
        HtmlSource::utf8("<!doctype html><form id=f><input type=number value=''>Z</form>"),
        &[("type", &keyword), ("value", &[])],
        &[],
        &[],
        &[],
    );
    assert_eq!(absent, super::head::HeadStep::Complete);
    assert_eq!(empty, super::head::HeadStep::Complete);
}

#[test]
fn ordinary_number_input_entity_crlf_stays_default_but_invalidates_current_in_both_orders() {
    let keyword: Vec<_> = "number".encode_utf16().collect();
    let default = [49, 13, 10, 50];
    let mut boundaries = Vec::new();
    for value_first in [false, true] {
        let attrs = if value_first {
            "value='1&#13;&#10;2' type=number"
        } else {
            "type=number value='1&#13;&#10;2'"
        };
        let raw = format!("<!doctype html><form id=f><input {attrs}>Z</form>");
        let cr = raw.find("&#13;").unwrap();
        let lf = raw.find("&#10;").unwrap();
        let expected = if value_first {
            vec![("value", default.as_slice()), ("type", keyword.as_slice())]
        } else {
            vec![("type", keyword.as_slice()), ("value", default.as_slice())]
        };
        boundaries.push(ordinary_number_input_observes_complete(
            HtmlSource::utf8(raw),
            &expected,
            &default,
            &[],
            &[(1, 2, cr, cr + 5), (2, 3, lf, lf + 5)],
        ));
    }
    for boundary in boundaries {
        assert_eq!(boundary, super::head::HeadStep::Complete);
    }
}

#[test]
fn ordinary_number_input_isolated_utf16_surrogate_stays_native_default_and_empties_current() {
    let mut raw: Vec<_> = "<!doctype html><form id=f><input type=number value='"
        .encode_utf16()
        .collect();
    raw.extend([49, 0xd800, 50]);
    raw.extend("'>Z</form>".encode_utf16());
    let keyword: Vec<_> = "number".encode_utf16().collect();
    let boundary = ordinary_number_input_observes_complete(
        HtmlSource::utf16(crate::DocumentString::from_utf16(raw)),
        &[("type", &keyword), ("value", &[49, 0xd800, 50])],
        &[49, 0xd800, 50],
        &[],
        &[],
    );
    assert_eq!(boundary, super::head::HeadStep::Complete);
}

#[test]
fn ordinary_number_input_initial_value_is_not_mandatorily_clamped_or_step_rounded() {
    let keyword: Vec<_> = "number".encode_utf16().collect();
    let boundary = ordinary_number_input_observes_complete(
        HtmlSource::utf8(
            "<!doctype html><form id=f><input type=number min=0 max=10 step=2 value=13>Z</form>",
        ),
        &[
            ("type", &keyword),
            ("min", &[48]),
            ("max", &[49, 48]),
            ("step", &[50]),
            ("value", &[49, 51]),
        ],
        &[49, 51],
        &[49, 51],
        &[],
    );
    assert_eq!(boundary, super::head::HeadStep::Complete);
}

// HTML877d reset34985–96, type-change35087–102, Filename39211–31,
// Image Default38055–57, and Image src guards37907–25. Fresh offline image
// support is affirmatively disabled by the selected #1136 construction policy;
// these cases do not infer that premise from scripting or a missing loader.
fn ordinary_file_image_input_observes_completed_state(
    raw: &str,
    expected_type: crate::HtmlInputType,
    expected_attributes: &[(&str, &str)],
    expected_default: &str,
    expected_internal: &str,
    expected_current: &str,
    expected_checked: bool,
) -> super::head::HeadStep {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw), limits(100));
    let mut driver = super::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.stop(), stop);
        assert!(stop.is_none());
        assert!(driver.control.first_stop_usage().is_none());
        return boundary;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let child_count = children.len();
    let input = children[0].clone();
    let node = document.node(&input).unwrap();
    let input_child_count = node.children().count();
    let HtmlNodeKind::Element(element) = node.kind() else {
        panic!("input must be an actual native Element");
    };
    let name = element.name().local_name().code_units().to_vec();
    let namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let attributes: Vec<_> = element
        .attributes()
        .iter()
        .map(|a| {
            (
                a.name()
                    .namespace()
                    .uri()
                    .map(|uri| uri.code_units().to_vec()),
                a.name().local_name().code_units().to_vec(),
                a.value().code_units().to_vec(),
            )
        })
        .collect();
    let HtmlNodeKind::Text(text) = document.node(&children[1]).unwrap().kind() else {
        panic!("Z must be the actual following sibling, not an input descendant");
    };
    let following_text = text.code_units().to_vec();
    let owner = frozen.form_association_owner(&mut control, &input).unwrap();
    let owner_is_form = owner.as_ref() == Some(&form);
    drop(owner);
    let association_kind = frozen
        .form_association_state(&mut control, &input)
        .unwrap()
        .kind;
    let state = frozen.input_state(&mut control, &input).unwrap();
    let actual_type = state.flags.input_type();
    let flags = (
        state.flags.dirty_value(),
        state.flags.dirty_checkedness(),
        state.flags.checkedness(),
        state.flags.indeterminate(),
        state.flags.user_validity(),
    );
    let mut buffer = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
    frozen
        .append_input_default_value(&mut control, &input, &mut buffer)
        .unwrap();
    let default = buffer.units().to_vec();
    buffer.clear(&mut control).unwrap();
    frozen
        .append_input_internal_value(&mut control, &input, &mut buffer)
        .unwrap();
    let internal = buffer.units().to_vec();
    buffer.clear(&mut control).unwrap();
    frozen
        .append_input_value(&mut control, &input, &mut buffer)
        .unwrap();
    let current = buffer.units().to_vec();
    buffer.release(&mut control).unwrap();
    let original_source = {
        let (_, parts, _) = frozen.retained_evidence().unwrap().lexical_observations();
        !parts.is_empty()
            && parts
                .iter()
                .all(|part| part.source.source() == request.source().identity())
    };
    drop(input);
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 2);
    assert_eq!(input_child_count, 0);
    assert_eq!(name, vec![105, 110, 112, 117, 116]);
    assert_eq!(
        namespace,
        "http://www.w3.org/1999/xhtml"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        attributes,
        expected_attributes
            .iter()
            .map(|(name, value)| (
                None,
                name.encode_utf16().collect::<Vec<_>>(),
                value.encode_utf16().collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(following_text, vec![90]);
    assert!(owner_is_form && original_source);
    assert_eq!(
        association_kind,
        crate::HtmlNativeFormAssociationKind::Input
    );
    assert_eq!(actual_type, expected_type);
    assert_eq!(flags, (false, false, expected_checked, false, false));
    assert!(state.selected_files_empty);
    assert_eq!(default, expected_default.encode_utf16().collect::<Vec<_>>());
    assert_eq!(
        internal,
        expected_internal.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(current, expected_current.encode_utf16().collect::<Vec<_>>());
    boundary
}

type FileImageInputLiteral<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a str, bool);

#[test]
fn ordinary_file_input_empty_selection_keeps_raw_default_and_ordered_internal_value() {
    // The malformed value/checked content remains stored. Reset installs that
    // default internally but actually empties selected files; Filename getter
    // therefore returns empty, never a synthetic path or the raw value.
    let cases: &[FileImageInputLiteral<'_>] = &[
        ("type=file", &[("type", "file")], "", false),
        (
            "type=file value=''",
            &[("type", "file"), ("value", "")],
            "",
            false,
        ),
        (
            "type=file value='not-a-file' checked=false",
            &[
                ("type", "file"),
                ("value", "not-a-file"),
                ("checked", "false"),
            ],
            "not-a-file",
            true,
        ),
        (
            "type=file value='&#13;&#10;X'",
            &[("type", "file"), ("value", "\r\nX")],
            "\r\nX",
            false,
        ),
        (
            "value='&#13;&#10;X' type=file",
            &[("value", "\r\nX"), ("type", "file")],
            "\r\nX",
            false,
        ),
        (
            "checked value='raw' type=file",
            &[("checked", ""), ("value", "raw"), ("type", "file")],
            "raw",
            true,
        ),
    ];
    let mut outcomes = Vec::new();
    for &(attributes, native_attributes, default, checked) in cases {
        let raw = format!("<!doctype html><form id=f><input {attributes}>Z</form>");
        outcomes.push((
            attributes,
            ordinary_file_image_input_observes_completed_state(
                &raw,
                crate::HtmlInputType::File,
                native_attributes,
                default,
                default,
                "",
                checked,
            ),
        ));
    }
    // Every literal takes the real driver path and is released before any
    // discriminatory completion assertion, including all currently refused cases.
    assert!(
        outcomes
            .iter()
            .all(|(_, boundary)| *boundary == super::head::HeadStep::Complete),
        "all File literals must complete with established empty selection: {outcomes:?}"
    );
}

#[test]
fn ordinary_offline_image_input_disabled_policy_keeps_default_and_all_src_spellings() {
    // Source guards skip URL/fetch before parsing src under the selected disabled
    // image policy. Whitespace/relative/absolute/data/fragment are not empty-src
    // or guessed URL-failure cases, and input Image remains form-associated.
    let cases: &[FileImageInputLiteral<'_>] = &[
        ("type=image", &[("type", "image")], "", false),
        (
            "type=image src='' value=''",
            &[("type", "image"), ("src", ""), ("value", "")],
            "",
            false,
        ),
        (
            "type=image src='photo.png' value='raw' checked",
            &[
                ("type", "image"),
                ("src", "photo.png"),
                ("value", "raw"),
                ("checked", ""),
            ],
            "raw",
            true,
        ),
        (
            "src=' \t ' type=image value='&#13;&#10;X'",
            &[("src", " \t "), ("type", "image"), ("value", "\r\nX")],
            "\r\nX",
            false,
        ),
        (
            "type=image src='https://example.invalid/p.png'",
            &[("type", "image"), ("src", "https://example.invalid/p.png")],
            "",
            false,
        ),
        (
            "src='data:image/png;base64,AA==' type=image",
            &[("src", "data:image/png;base64,AA=="), ("type", "image")],
            "",
            false,
        ),
        (
            "type=image src='#fragment'",
            &[("type", "image"), ("src", "#fragment")],
            "",
            false,
        ),
        (
            "value='&#13;&#10;X' src='photo.png' type=image",
            &[("value", "X"), ("src", "photo.png"), ("type", "image")],
            "X",
            false,
        ),
    ];
    let mut outcomes = Vec::new();
    for &(attributes, native_attributes, value, checked) in cases {
        let raw = format!("<!doctype html><form id=f><input {attributes}>Z</form>");
        outcomes.push((
            attributes,
            ordinary_file_image_input_observes_completed_state(
                &raw,
                crate::HtmlInputType::Image,
                native_attributes,
                value,
                value,
                value,
                checked,
            ),
        ));
    }
    assert!(
        outcomes
            .iter()
            .all(|(_, boundary)| *boundary == super::head::HeadStep::Complete),
        "all offline Image literals must complete under actual images-disabled policy: {outcomes:?}"
    );
}

#[test]
fn number_input_table_fostering_uses_value_state_and_preserves_form_and_native_default() {
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><form id=f><table><input type=number value=13 min=0 max=10 step=2 checked><tr><td>T</table>Z</form>",
        ),
        limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    assert_eq!(
        standalone_boundary(&mut driver),
        super::head::HeadStep::Complete
    );
    let (mut c, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let shape = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().last().unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let input = document.node(&form).unwrap().children().next().unwrap();
    let owner = frozen.form_association_owner(&mut c, &input).unwrap();
    let state = frozen.input_state(&mut c, &input).unwrap();
    let mut output = crate::parser::buffer::Utf16Buffer::new(&mut c).unwrap();
    frozen
        .append_input_internal_value(&mut c, &input, &mut output)
        .unwrap();
    let current = output.units().to_vec();
    output.clear(&mut c).unwrap();
    frozen
        .append_input_default_value(&mut c, &input, &mut output)
        .unwrap();
    let default = output.units().to_vec();
    output.release(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    assert_released(&c);
    assert_eq!(
        shape,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "table"),
            (4, "tbody"),
            (5, "tr"),
            (6, "td"),
            (7, "#T"),
            (3, "#Z"),
        ])
    );
    assert_eq!(owner, Some(form));
    assert_eq!(state.flags.input_type(), crate::HtmlInputType::Number);
    assert!(!state.flags.input_type().is_selectable());
    assert!(!state.flags.dirty_value() && !state.flags.dirty_checkedness());
    assert!(state.flags.checkedness() && state.selected_files_empty);
    assert!(!state.value_rewritten_during_creation);
    assert_eq!(state.cursor, 0);
    assert_eq!(current, [49, 51]);
    assert_eq!(default, [49, 51]);
}

// HTML877d Range36939-66, constraints38694+/38808+, ordered input
// initialization34947+/35088+, and the selected #1134 exact finite-number join.
fn ordinary_range_input_observes_complete(
    source: HtmlSource,
    expected_attributes: &[(&str, &[u16])],
    expected_default: &[u16],
    expected_current: &[u16],
    expected_references: &[(usize, usize, usize, usize)],
) -> super::head::HeadStep {
    use crate::construction::provenance::LexicalRole;
    use crate::parser::tokenizer::token::OriginKind;

    let request = crate::HtmlDocumentParseRequest::offline(source, limits(100));
    let mut driver = super::entry::initialized_driver(&request);
    let boundary = standalone_boundary(&mut driver);
    if boundary != super::head::HeadStep::Complete {
        let first_stop = driver.control.stop();
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.stop(), first_stop);
        assert!(first_stop.is_none());
        assert!(driver.control.first_stop_usage().is_none());
        return boundary;
    }

    let (mut control, mut frozen) = body_document(driver);
    let document = frozen.document().unwrap();
    let mode = document.mode();
    let actual_outline = outline(document);
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let form = document.node(&body).unwrap().children().next().unwrap();
    let children: Vec<_> = document.node(&form).unwrap().children().collect();
    let child_count = children.len();
    let input = children[0].clone();
    let input_children = document.node(&input).unwrap().children().count();
    let HtmlNodeKind::Element(element) = document.node(&input).unwrap().kind() else {
        panic!("input must be an actual native element");
    };
    let namespace = element
        .name()
        .namespace()
        .uri()
        .unwrap()
        .code_units()
        .to_vec();
    let local = element.name().local_name().code_units().to_vec();
    let attributes: Vec<_> = element
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
        .collect();
    let HtmlNodeKind::Text(text) = document.node(&children[1]).unwrap().kind() else {
        panic!("Z must remain an actual sibling after the void input");
    };
    let following_text = text.code_units().to_vec();
    let owner = frozen.form_association_owner(&mut control, &input).unwrap();
    let owner_is_form = owner.as_ref() == Some(&form);
    drop(owner);
    let state = frozen.input_state(&mut control, &input).unwrap();
    let actual_type = state.flags.input_type();
    let value_mode = state.flags.input_type().value_mode();
    let selectable = state.flags.input_type().is_selectable();
    let flags = (
        state.flags.dirty_value(),
        state.flags.dirty_checkedness(),
        state.flags.checkedness(),
        state.flags.indeterminate(),
        state.flags.user_validity(),
    );
    let files_empty = state.selected_files_empty;
    let cursor = state.cursor;
    let writeback_absent = frozen
        .input_writeback_sources(&mut control, &input)
        .unwrap()
        .is_none();
    let mut buffer = crate::parser::buffer::Utf16Buffer::new(&mut control).unwrap();
    frozen
        .append_input_internal_value(&mut control, &input, &mut buffer)
        .unwrap();
    let internal = buffer.units().to_vec();
    buffer.clear(&mut control).unwrap();
    frozen
        .append_input_value(&mut control, &input, &mut buffer)
        .unwrap();
    let current = buffer.units().to_vec();
    buffer.clear(&mut control).unwrap();
    frozen
        .append_input_default_value(&mut control, &input, &mut buffer)
        .unwrap();
    let default = buffer.units().to_vec();
    buffer.release(&mut control).unwrap();
    let (references, source_qualified) = {
        let evidence = frozen.retained_evidence().unwrap();
        let (_, parts, _) = evidence.lexical_observations();
        let source_qualified = !parts.is_empty()
            && parts
                .iter()
                .all(|part| part.source.source() == request.source().identity());
        let references = parts
            .iter()
            .filter(|part| {
                part.role == LexicalRole::Value && part.kind == Some(OriginKind::CharacterReference)
            })
            .map(|part| (part.start, part.end, part.source.start(), part.source.end()))
            .collect::<Vec<_>>();
        (references, source_qualified)
    };
    drop(input);
    drop(children);
    drop(form);
    drop(body);
    drop(html);
    frozen.release(&mut control).unwrap();
    assert_released(&control);
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), None);

    assert_eq!(mode, HtmlDocumentMode::NoQuirks);
    assert_eq!(
        actual_outline,
        expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "form"),
            (3, "input"),
            (3, "#Z"),
        ])
    );
    assert_eq!(child_count, 2);
    assert_eq!(input_children, 0);
    assert_eq!(
        namespace,
        "http://www.w3.org/1999/xhtml"
            .encode_utf16()
            .collect::<Vec<_>>()
    );
    assert_eq!(local, [105, 110, 112, 117, 116]);
    assert_eq!(
        attributes,
        expected_attributes
            .iter()
            .map(|(name, value)| (
                None,
                name.encode_utf16().collect::<Vec<_>>(),
                value.to_vec()
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(following_text, [90]);
    assert!(owner_is_form && files_empty && writeback_absent && source_qualified);
    assert_eq!(actual_type, crate::HtmlInputType::Range);
    assert_eq!(value_mode, crate::HtmlInputValueMode::Value);
    assert!(!selectable);
    assert_eq!(cursor, 0);
    assert_eq!(flags, (false, false, false, false, false));
    assert_eq!(default, expected_default);
    assert_eq!(internal, expected_current);
    assert_eq!(current, expected_current);
    assert_eq!(references, expected_references);
    boundary
}

type RangeInputLiteral<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a str, &'a str);

// Each source drives the actual ordinary constructor and releases all of its
// owners before any discriminatory Complete assertion. The expected strings
// below are literal source-rule results, never computed by the Range producer.
fn ordinary_range_input_literal_matrix(cases: &[RangeInputLiteral<'_>]) {
    let mut boundaries = Vec::new();
    for &(attributes, expected_native, default, current) in cases {
        let raw = format!("<!doctype html><form id=f><input {attributes}>Z</form>");
        let native_units: Vec<_> = expected_native
            .iter()
            .map(|(name, value)| (*name, value.encode_utf16().collect::<Vec<_>>()))
            .collect();
        let native: Vec<_> = native_units
            .iter()
            .map(|(name, value)| (*name, value.as_slice()))
            .collect();
        let default: Vec<_> = default.encode_utf16().collect();
        let current: Vec<_> = current.encode_utf16().collect();
        let boundary = ordinary_range_input_observes_complete(
            HtmlSource::utf8(raw),
            &native,
            &default,
            &current,
            &[],
        );
        boundaries.push((attributes, boundary));
    }
    for (attributes, boundary) in boundaries {
        assert_eq!(
            boundary,
            super::head::HeadStep::Complete,
            "Range attributes={attributes:?}"
        );
    }
}

#[test]
fn ordinary_range_input_default_and_invalid_value_use_source_midpoint_without_rewriting_native_default()
 {
    // HTML877d36939-66: invalid/empty/absent current gets midpoint(0,100)=50.
    // The separate defaultValue getter still reflects actual value content.
    ordinary_range_input_literal_matrix(&[
        ("type=range", &[("type", "range")], "", "50"),
        (
            "type=range value=''",
            &[("type", "range"), ("value", "")],
            "",
            "50",
        ),
        (
            "type=range value=bad",
            &[("type", "range"), ("value", "bad")],
            "bad",
            "50",
        ),
        (
            "type=range value=+1",
            &[("type", "range"), ("value", "+1")],
            "+1",
            "50",
        ),
        (
            "type=range value=1.",
            &[("type", "range"), ("value", "1.")],
            "1.",
            "50",
        ),
        (
            "type=range value=12junk",
            &[("type", "range"), ("value", "12junk")],
            "12junk",
            "50",
        ),
        (
            "type=range value=' 1 '",
            &[("type", "range"), ("value", " 1 ")],
            " 1 ",
            "50",
        ),
        (
            "type=range value=1e+",
            &[("type", "range"), ("value", "1e+")],
            "1e+",
            "50",
        ),
    ]);
}

#[test]
fn ordinary_range_input_bounds_recover_prefixes_clamp_and_disable_upper_correction_when_reversed() {
    // Recovery applies to constraints; whole Author grammar applies to current.
    // max<min changes the default and disables overflow, not underflow.
    ordinary_range_input_literal_matrix(&[
        (
            "type=range min=80 max=20",
            &[("type", "range"), ("min", "80"), ("max", "20")],
            "",
            "80",
        ),
        (
            "type=range min=80 max=20 step=any value=90",
            &[
                ("type", "range"),
                ("min", "80"),
                ("max", "20"),
                ("step", "any"),
                ("value", "90"),
            ],
            "90",
            "90",
        ),
        (
            "type=range min=80 max=20 step=any value=0",
            &[
                ("type", "range"),
                ("min", "80"),
                ("max", "20"),
                ("step", "any"),
                ("value", "0"),
            ],
            "0",
            "80",
        ),
        (
            "type=range min=0 max=100 step=any value=-2",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "any"),
                ("value", "-2"),
            ],
            "-2",
            "0",
        ),
        (
            "type=range min=0 max=100 step=any value=101",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "any"),
                ("value", "101"),
            ],
            "101",
            "100",
        ),
        (
            "type=range min='+2junk' max=10 step=any value=1",
            &[
                ("type", "range"),
                ("min", "+2junk"),
                ("max", "10"),
                ("step", "any"),
                ("value", "1"),
            ],
            "1",
            "2",
        ),
        (
            "type=range min='+2junk' max=10 step=any value='+2junk'",
            &[
                ("type", "range"),
                ("min", "+2junk"),
                ("max", "10"),
                ("step", "any"),
                ("value", "+2junk"),
            ],
            "+2junk",
            "6",
        ),
        (
            "type=range min=bad max=1 step=4 value=3",
            &[
                ("type", "range"),
                ("min", "bad"),
                ("max", "1"),
                ("step", "4"),
                ("value", "3"),
            ],
            "3",
            "1",
        ),
    ]);
}

#[test]
fn ordinary_range_input_step_ties_and_base_precedence_use_actual_content_not_default_minimum() {
    // min0/base0: 50 is tied between40 and60, so choose60. With unusable
    // min, usable value content50 becomes the base and50 has no mismatch.
    ordinary_range_input_literal_matrix(&[
        (
            "type=range min=0 max=100 step=20 value=50",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "20"),
                ("value", "50"),
            ],
            "50",
            "60",
        ),
        (
            "type=range min=0 max=100 step=20 value=bad",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "20"),
                ("value", "bad"),
            ],
            "bad",
            "60",
        ),
        (
            "type=range max=100 step=20 value=50",
            &[
                ("type", "range"),
                ("max", "100"),
                ("step", "20"),
                ("value", "50"),
            ],
            "50",
            "50",
        ),
        (
            "type=range min=bad max=100 step=20 value=50",
            &[
                ("type", "range"),
                ("min", "bad"),
                ("max", "100"),
                ("step", "20"),
                ("value", "50"),
            ],
            "50",
            "50",
        ),
        (
            "type=range min=bad max=100 step=20 value=bad",
            &[
                ("type", "range"),
                ("min", "bad"),
                ("max", "100"),
                ("step", "20"),
                ("value", "bad"),
            ],
            "bad",
            "60",
        ),
        (
            "type=range min=-30 max=10 step=20 value=0",
            &[
                ("type", "range"),
                ("min", "-30"),
                ("max", "10"),
                ("step", "20"),
                ("value", "0"),
            ],
            "0",
            "10",
        ),
        (
            "type=range min=0 max=10 step=4 value=10",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "10"),
                ("step", "4"),
                ("value", "10"),
            ],
            "10",
            "8",
        ),
        (
            "type=range min=0 max=100 step=0 value=1.5",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "0"),
                ("value", "1.5"),
            ],
            "1.5",
            "2",
        ),
        (
            "type=range min=0 max=100 step=' any ' value=1.5",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", " any "),
                ("value", "1.5"),
            ],
            "1.5",
            "2",
        ),
    ]);
}

#[test]
fn ordinary_range_input_any_preserves_valid_spelling_and_grammatical_conversion_overflow() {
    // any is a whole ASCII-insensitive keyword. 1e999 is grammatical; failed
    // conversion is not invalid syntax, underflow, overflow or step mismatch.
    ordinary_range_input_literal_matrix(&[
        (
            "type=range step=ANY value=.5",
            &[("type", "range"), ("step", "ANY"), ("value", ".5")],
            ".5",
            ".5",
        ),
        (
            "type=range step=any value=1e+2",
            &[("type", "range"), ("step", "any"), ("value", "1e+2")],
            "1e+2",
            "1e+2",
        ),
        (
            "type=range step=any value=-0",
            &[("type", "range"), ("step", "any"), ("value", "-0")],
            "-0",
            "-0",
        ),
        (
            "type=range min=0 max=100 step=any value=1e999",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "any"),
                ("value", "1e999"),
            ],
            "1e999",
            "1e999",
        ),
        (
            "value=1e999 type=range min=0 max=100 step=any",
            &[
                ("value", "1e999"),
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "any"),
            ],
            "1e999",
            "1e999",
        ),
    ]);
}

#[test]
fn ordinary_range_input_allowed_step_preserves_authored_signed_zero_without_replacement() {
    // HTML877d36939-66 and 38839-43: grammatical -0 is within these bounds
    // and already lies on the allowed-step lattice. No replacement is requested,
    // so its authored spelling and content default remain unchanged.
    ordinary_range_input_literal_matrix(&[
        (
            "type=range value=-0",
            &[("type", "range"), ("value", "-0")],
            "-0",
            "-0",
        ),
        (
            "type=range min=0 max=100 step=1 value=-0",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "1"),
                ("value", "-0"),
            ],
            "-0",
            "-0",
        ),
        (
            "value=-0 min=0 max=100 step=1 type=range",
            &[
                ("value", "-0"),
                ("min", "0"),
                ("max", "100"),
                ("step", "1"),
                ("type", "range"),
            ],
            "-0",
            "-0",
        ),
    ]);
}

#[test]
fn ordinary_range_input_exact_finite_lattice_and_creation_reset_use_final_constraints_in_both_orders()
 {
    // Selected #1134 exact-binary64 join: explicit min/base0, step0.1 and
    // current0.3 choose already-representable0.2. No browser tolerance claim.
    // Initial prefix corrections do not change value content. The required
    // fresh-creation reset reloads that default and uses the final constraints
    // in either order; intermediate causal effects require separate owner tests.
    ordinary_range_input_literal_matrix(&[
        (
            "type=range min=0 max=100 step=0.1 value=0.3",
            &[
                ("type", "range"),
                ("min", "0"),
                ("max", "100"),
                ("step", "0.1"),
                ("value", "0.3"),
            ],
            "0.3",
            "0.2",
        ),
        (
            "value=0.3 min=0 max=100 step=0.1 type=range",
            &[
                ("value", "0.3"),
                ("min", "0"),
                ("max", "100"),
                ("step", "0.1"),
                ("type", "range"),
            ],
            "0.3",
            "0.2",
        ),
        (
            "type=range value=0.3 min=0 max=100 step=0.1",
            &[
                ("type", "range"),
                ("value", "0.3"),
                ("min", "0"),
                ("max", "100"),
                ("step", "0.1"),
            ],
            "0.3",
            "0.2",
        ),
        (
            "type=range value=50 step=20 min=0",
            &[
                ("type", "range"),
                ("value", "50"),
                ("step", "20"),
                ("min", "0"),
            ],
            "50",
            "60",
        ),
    ]);
}

#[test]
fn ordinary_range_input_entity_crlf_stays_native_default_but_generates_midpoint_current_in_both_orders()
 {
    let keyword: Vec<_> = "range".encode_utf16().collect();
    let default = [49, 13, 10, 50];
    let mut boundaries = Vec::new();
    for value_first in [false, true] {
        let attributes = if value_first {
            "value='1&#13;&#10;2' type=range"
        } else {
            "type=range value='1&#13;&#10;2'"
        };
        let raw = format!("<!doctype html><form id=f><input {attributes}>Z</form>");
        let cr = raw.find("&#13;").unwrap();
        let lf = raw.find("&#10;").unwrap();
        let native = if value_first {
            vec![("value", default.as_slice()), ("type", keyword.as_slice())]
        } else {
            vec![("type", keyword.as_slice()), ("value", default.as_slice())]
        };
        boundaries.push(ordinary_range_input_observes_complete(
            HtmlSource::utf8(raw),
            &native,
            &default,
            &[53, 48],
            &[(1, 2, cr, cr + 5), (2, 3, lf, lf + 5)],
        ));
    }
    for boundary in boundaries {
        assert_eq!(boundary, super::head::HeadStep::Complete);
    }
}

#[test]
fn ordinary_range_input_isolated_utf16_surrogate_stays_native_default_and_generates_midpoint_current()
 {
    let mut raw: Vec<_> = "<!doctype html><form id=f><input type=range value='"
        .encode_utf16()
        .collect();
    raw.extend([49, 0xd800, 50]);
    raw.extend("'>Z</form>".encode_utf16());
    let keyword: Vec<_> = "range".encode_utf16().collect();
    let boundary = ordinary_range_input_observes_complete(
        HtmlSource::utf16(crate::DocumentString::from_utf16(raw)),
        &[("type", &keyword), ("value", &[49, 0xd800, 50])],
        &[49, 0xd800, 50],
        &[53, 48],
        &[],
    );
    assert_eq!(boundary, super::head::HeadStep::Complete);
}

// HTML877d AfterHead raw96773–96784 requires the real head pointer even
// when InHead processing makes template the new current stack identity.
#[test]
fn afterhead_template_removes_exact_head_identity_and_retains_source_diagnostic() {
    use super::head::{HeadCondition, HeadStep, InsertionMode};
    use crate::construction::evidence::{
        DiagnosticContext, DiagnosticKind, DiagnosticLocationRole, DiagnosticSeverity,
        DiagnosticToken,
    };
    for (source, late) in [
        (
            "<!doctype html><head></head><template>x</template><body>z",
            true,
        ),
        ("<!doctype html><body><template>x</template>z", false),
    ] {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits(100));
        let mut driver = super::entry::initialized_driver(&request);
        let mut reached = false;
        for _ in 0..100 {
            driver.step().unwrap();
            let Some(stage) = &mut driver.head else {
                continue;
            };
            let state = stage.state.as_mut().unwrap();
            if state.mode != InsertionMode::InTemplate {
                continue;
            }
            assert!(state.temporary_head.is_none());
            assert_eq!(state.template_modes.len(), 1);
            assert_eq!(state.formatting.len(), 1);
            let current = state.before.kernel.current(&mut driver.control).unwrap();
            assert!(
                state
                    .before
                    .kernel
                    .named(&current, "template", &mut driver.control)
                    .unwrap()
            );
            let parent = state
                .before
                .kernel
                .session
                .parent(&mut driver.control, &current)
                .unwrap()
                .unwrap();
            assert!(
                state
                    .before
                    .kernel
                    .named(
                        &parent,
                        if late { "head" } else { "body" },
                        &mut driver.control
                    )
                    .unwrap()
            );
            if late {
                let head = state.head.as_ref().unwrap();
                assert!(
                    state
                        .before
                        .kernel
                        .session
                        .same_node(&mut driver.control, &parent, head)
                        .unwrap()
                );
                assert_eq!(state.before.kernel.stack().unwrap().len(), 2);
                for index in 0..2 {
                    let entry = state.before.kernel.at(index, &mut driver.control).unwrap();
                    assert!(
                        !state
                            .before
                            .kernel
                            .session
                            .same_node(&mut driver.control, &entry, head)
                            .unwrap()
                    );
                }
                // Removing head must leave template current, rather than popping it.
                let still_current = state.before.kernel.current(&mut driver.control).unwrap();
                assert!(
                    state
                        .before
                        .kernel
                        .session
                        .same_node(&mut driver.control, &still_current, &current)
                        .unwrap()
                );
            }
            reached = true;
            break;
        }
        assert!(reached);
        assert_eq!(standalone_boundary(&mut driver), HeadStep::Complete);
        let (mut control, mut frozen) = body_document(driver);
        let diagnostics = frozen.retained_evidence().unwrap().diagnostics();
        assert_eq!(diagnostics.len(), usize::from(late));
        if late {
            let (record, location) = diagnostics.retained_record(0).unwrap().unwrap();
            assert_eq!(
                record.kind,
                DiagnosticKind::Tree(HeadCondition::UnexpectedBodyToken)
            );
            assert_eq!(
                record.context,
                DiagnosticContext::Tree(InsertionMode::AfterHead)
            );
            assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
            assert_eq!(record.rule.anchor, "the-after-head-insertion-mode");
            assert_eq!(record.rule.branch, "head-start/delegate-using-head-pointer");
            assert_eq!(location.role, DiagnosticLocationRole::TokenEnvelope);
            assert_eq!(location.token, Some(DiagnosticToken::Tag));
            assert_eq!(location.source.source(), request.source().identity());
            assert_eq!((location.source.start(), location.source.end()), (28, 38));
        }
        frozen.release(&mut control).unwrap();
        assert_released(&control);
    }
}

#[test]
fn afterhead_template_open_slot_refusal_retains_actual_temporary_head_and_first_stop() {
    use super::head::InsertionMode;
    // Initial head creation requires two slots. After it closes, html is
    // open; the temporary head consumes slot two and template needs slot three.
    for open in [2, 3] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><head></head><template>x</template>"),
            limits(open),
        );
        let mut driver = super::entry::initialized_driver(&request);
        let mut failed = false;
        for _ in 0..100 {
            if driver.step().is_err() {
                failed = true;
                break;
            }
        }
        if open == 3 {
            assert!(!failed);
            assert_eq!(
                standalone_boundary(&mut driver),
                super::head::HeadStep::Complete
            );
            driver.release().unwrap();
            assert_released(&driver.control);
            continue;
        }
        assert!(failed);
        assert!(matches!(
            driver.control.stop(),
            Some(ParseStop::LimitExceeded {
                resource: HtmlParseResource::OpenElements,
                requested_delta: 1,
                ..
            })
        ));
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(
            state.mode,
            if open == 1 {
                InsertionMode::AfterHead
            } else {
                InsertionMode::InTemplate
            }
        );
        assert!(state.temporary_head.is_some());
        assert_eq!(state.before.kernel.stack().unwrap().len(), open as usize);
        assert_eq!(state.diagnostics().len(), 1);
        let first = driver.control.stop();
        let at = driver.control.first_stop_usage().unwrap().clone();
        let usage = driver.control.usage().clone();
        assert!(driver.step().is_err());
        assert_eq!(driver.control.usage(), &usage);
        assert_eq!(driver.control.first_stop_usage(), Some(&at));
        driver.release().unwrap();
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            usage.resource(HtmlParseResource::WorkCredits).current,
            "terminal cleanup performs no useful work or capacity reservation"
        );
        assert_eq!(driver.control.stop(), first);
        assert_eq!(driver.control.first_stop_usage(), Some(&at));
        assert_released(&driver.control);
    }
}

#[test]
fn afterhead_template_cancel_and_one_short_work_precede_input_and_cleanup_do_no_work() {
    use super::head::InsertionMode;
    for cancel in [false, true] {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><head></head><template>x</template>"),
            limits(100),
        );
        let mut driver = super::entry::initialized_driver(&request);
        let mut reached = false;
        for _ in 0..100 {
            driver.step().unwrap();
            if driver
                .head
                .as_ref()
                .and_then(|stage| stage.state.as_ref())
                .is_some_and(|state| state.mode == InsertionMode::AfterHead)
            {
                reached = true;
                break;
            }
        }
        assert!(reached);
        if cancel {
            request.cancellation().cancel();
        } else {
            // The next original-controller qualification requires a single
            // credit. Leave zero (one short), before pulling the template token.
            let remaining = request.limits().work_credits()
                - driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current;
            driver
                .control
                .charge(WorkCategory::StateStep, remaining)
                .unwrap();
        }
        let reads = driver.control.usage().work(WorkCategory::InputUnitRead);
        let writes = driver.control.usage().work(WorkCategory::RecordWrite);
        assert!(driver.step().is_err());
        if cancel {
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
        } else {
            assert!(matches!(
                driver.control.stop(),
                Some(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    requested_delta: 1,
                    ..
                })
            ));
        }
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.mode, InsertionMode::AfterHead);
        assert!(state.temporary_head.is_none());
        assert_eq!(state.before.kernel.stack().unwrap().len(), 1);
        assert!(state.diagnostics().is_empty());
        let first = driver.control.stop();
        let at = driver.control.first_stop_usage().unwrap().clone();
        let usage = driver.control.usage().clone();
        assert!(driver.step().is_err());
        assert_eq!(driver.control.usage(), &usage);
        assert_eq!(
            driver.control.usage().work(WorkCategory::InputUnitRead),
            reads
        );
        assert_eq!(
            driver.control.usage().work(WorkCategory::RecordWrite),
            writes
        );
        driver.release().unwrap();
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            usage.resource(HtmlParseResource::WorkCredits).current,
            "terminal cleanup performs no useful work or capacity reservation"
        );
        assert_eq!(driver.control.stop(), first);
        assert_eq!(driver.control.first_stop_usage(), Some(&at));
        assert_eq!(
            driver.control.usage().work(WorkCategory::InputUnitRead),
            reads
        );
        assert_eq!(
            driver.control.usage().work(WorkCategory::RecordWrite),
            writes
        );
        assert_released(&driver.control);
    }
}

#[test]
fn afterhead_template_diagnostic_refusal_precedes_temporary_head_and_template_effects() {
    use super::head::InsertionMode;
    let request = crate::HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><head></head><template>x</template>"),
        limits(100).into_builder().diagnostics(0).finish().unwrap(),
    );
    let mut driver = super::entry::initialized_driver(&request);
    let mut failed = false;
    for _ in 0..100 {
        if driver.step().is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    assert!(matches!(
        driver.control.stop(),
        Some(ParseStop::LimitExceeded {
            resource: HtmlParseResource::Diagnostics,
            requested_delta: 1,
            ..
        })
    ));
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    assert_eq!(state.mode, InsertionMode::AfterHead);
    assert!(state.temporary_head.is_none());
    assert_eq!(state.template_modes.len(), 0);
    assert_eq!(state.formatting.len(), 0);
    assert_eq!(state.before.kernel.stack().unwrap().len(), 1);
    assert!(state.diagnostics().is_empty());
    let pending = state.pending.as_ref().unwrap();
    assert_eq!((pending.source.start(), pending.source.end()), (28, 38));
    assert!(matches!(pending.payload, Payload::Tag(Some(_))));
    // No successful pair has been published; the original tag remains owned.
    let first = driver.control.stop();
    let at = driver.control.first_stop_usage().unwrap().clone();
    let usage = driver.control.usage().clone();
    assert!(driver.step().is_err());
    assert_eq!(driver.control.usage(), &usage);
    driver.release().unwrap();
    assert_eq!(
        driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current,
        usage.resource(HtmlParseResource::WorkCredits).current,
        "terminal cleanup performs no useful work or capacity reservation"
    );
    assert_eq!(driver.control.stop(), first);
    assert_eq!(driver.control.first_stop_usage(), Some(&at));
    assert_released(&driver.control);
}

mod complete_hostile_scan {
    use super::*;
    use crate::construction::{
        evidence::{
            DiagnosticContext, DiagnosticEvidenceView, DiagnosticKind, DiagnosticLocationRole,
            DiagnosticSeverity,
        },
        native_view::{NativeKind, NativeView},
    };
    use crate::parser::tokenizer::{
        states::Quote,
        token::{DiagnosticCode, DiagnosticPhase},
    };

    #[derive(Clone, Copy)]
    enum Scan {
        ClosedValue,
        UnterminatedValue,
        Pi,
        LongPi,
        Duplicates,
    }

    fn input(scan: Scan) -> (String, usize) {
        match scan {
            Scan::ClosedValue => (
                format!(
                    "<!doctype html><p>P</p><div x=\"{}\">Q</div>",
                    "a".repeat(4096)
                ),
                2079,
            ),
            Scan::UnterminatedValue => (
                format!("<!doctype html><p>P</p><div x=\"{}", "a".repeat(4096)),
                2079,
            ),
            Scan::Pi => (
                format!("<!doctype html><p>P</p><?q {}?>", "b".repeat(512)),
                283,
            ),
            Scan::LongPi => (
                format!("<!doctype html><p>P</p><?q {}?>", "b".repeat(4096)),
                2075,
            ),
            Scan::Duplicates => {
                let mut raw = String::from("<!doctype html><p>P</p><div");
                for index in 0..64 {
                    raw.push_str(&format!(
                        " n{index:03}='v{index:03}' N{index:03}='discard{index:03}'"
                    ));
                }
                raw.push_str(">Q</div>");
                (raw, 996)
            }
        }
    }

    fn diagnostics(
        view: DiagnosticEvidenceView<'_>,
        source: &HtmlSource,
        scan: Scan,
        stopped: bool,
    ) {
        let count = match scan {
            Scan::Duplicates => {
                if stopped {
                    32
                } else {
                    64
                }
            }
            Scan::UnterminatedValue if !stopped => 1,
            _ => 0,
        };
        assert_eq!(view.len(), count);
        for index in 0..count {
            let (record, location) = view.retained_record(index).unwrap().unwrap();
            let duplicate = matches!(scan, Scan::Duplicates);
            assert_eq!(
                record.kind,
                DiagnosticKind::Tokenizer(if duplicate {
                    DiagnosticCode::DuplicateAttribute
                } else {
                    DiagnosticCode::EofInTag
                })
            );
            assert_eq!(
                record.context,
                DiagnosticContext::Tokenizer {
                    phase: DiagnosticPhase::Tokenizer,
                    state: if duplicate {
                        State::AttributeName
                    } else {
                        State::AttributeValue(Quote::Double)
                    },
                }
            );
            assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
            assert_eq!(
                (record.rule.anchor, record.rule.branch),
                if duplicate {
                    ("parse-error-duplicate-attribute", "duplicate-attribute")
                } else {
                    ("parse-error-eof-in-tag", "eof-in-tag")
                }
            );
            assert_eq!(location.role, DiagnosticLocationRole::InputFrontier);
            assert_eq!(location.token, None);
            assert_eq!(location.source.source(), source.identity());
            assert_eq!(location.source.coordinates(), source.coordinates());
            // Each literal pair has 30 ASCII units. The second name's '='
            // is at pair offset 17; EOF has the exact zero-width source end.
            let expected = if duplicate {
                (27 + index * 30 + 17, 27 + index * 30 + 18)
            } else {
                (source.len(), source.len())
            };
            assert_eq!((location.source.start(), location.source.end()), expected);
        }
        assert!(view.retained_record(count).unwrap().is_none());
    }

    fn element(view: &mut NativeView<'_>, path: &[usize], local: &str) -> ParserNode {
        let node = progressive_native_node(view, path);
        let NativeKind::Element(name) = view.kind(&node).unwrap() else {
            panic!("literal element")
        };
        progressive_native_units(
            view,
            name.namespace.unwrap(),
            "http://www.w3.org/1999/xhtml",
        );
        assert!(name.prefix.is_none());
        progressive_native_units(view, name.local, local);
        node
    }

    fn native(view: &mut NativeView<'_>, scan: Scan) {
        let root = view.root().unwrap();
        assert!(matches!(
            view.kind(&root).unwrap(),
            NativeKind::Document(HtmlDocumentMode::NoQuirks)
        ));
        assert_eq!(view.child_count(&root).unwrap(), 2);
        let doctype = progressive_native_node(view, &[0]);
        let NativeKind::Doctype {
            name,
            public_id,
            system_id,
        } = view.kind(&doctype).unwrap()
        else {
            panic!("literal doctype")
        };
        progressive_native_units(view, name, "html");
        // Canonical DOM creation defaults absent lexical identifiers to the
        // real empty strings, as in the existing native doctype fixtures.
        progressive_native_units(view, public_id.unwrap(), "");
        progressive_native_units(view, system_id.unwrap(), "");
        let html = element(view, &[1], "html");
        assert_eq!(view.child_count(&html).unwrap(), 2);
        let head = element(view, &[1, 0], "head");
        assert_eq!(view.child_count(&head).unwrap(), 0);
        let body = element(view, &[1, 1], "body");
        assert_eq!(
            view.child_count(&body).unwrap(),
            if matches!(scan, Scan::UnterminatedValue) {
                1
            } else {
                2
            }
        );
        let p = element(view, &[1, 1, 0], "p");
        assert_eq!(view.child_count(&p).unwrap(), 1);
        let text = progressive_native_node(view, &[1, 1, 0, 0]);
        let NativeKind::Text(value) = view.kind(&text).unwrap() else {
            panic!("literal P text")
        };
        progressive_native_units(view, value, "P");
        if matches!(scan, Scan::UnterminatedValue) {
            return;
        }
        if matches!(scan, Scan::Pi | Scan::LongPi) {
            let pi = progressive_native_node(view, &[1, 1, 1]);
            let NativeKind::ProcessingInstruction { target, data } = view.kind(&pi).unwrap() else {
                panic!("actual PI, not comment")
            };
            progressive_native_units(view, target, "q");
            progressive_native_units(
                view,
                data,
                &"b".repeat(if matches!(scan, Scan::Pi) { 512 } else { 4096 }),
            );
            assert_eq!(view.child_count(&pi).unwrap(), 0);
            return;
        }
        let div = element(view, &[1, 1, 1], "div");
        let count = if matches!(scan, Scan::Duplicates) {
            64
        } else {
            1
        };
        assert_eq!(view.attribute_count(&div).unwrap(), count);
        for index in 0..count {
            let attribute = view.attribute_at(&div, index).unwrap().unwrap();
            assert!(attribute.name.namespace.is_none() && attribute.name.prefix.is_none());
            if matches!(scan, Scan::Duplicates) {
                progressive_native_units(view, attribute.name.local, &format!("n{index:03}"));
                progressive_native_units(view, attribute.value, &format!("v{index:03}"));
            } else {
                progressive_native_units(view, attribute.name.local, "x");
                progressive_native_units(view, attribute.value, &"a".repeat(4096));
            }
        }
        assert!(view.attribute_at(&div, count).unwrap().is_none());
        assert_eq!(view.child_count(&div).unwrap(), 1);
        let text = progressive_native_node(view, &[1, 1, 1, 0]);
        let NativeKind::Text(value) = view.kind(&text).unwrap() else {
            panic!("literal Q text")
        };
        progressive_native_units(view, value, "Q");
    }

    fn partner(scan: Scan) {
        let (raw, _) = input(scan);
        for utf16 in [false, true] {
            let source = if utf16 {
                HtmlSource::utf16(DocumentString::from(raw.as_str()))
            } else {
                HtmlSource::utf8(raw.clone())
            };
            let request = crate::HtmlDocumentParseRequest::offline(source.clone(), limits(100));
            let mut driver = entry::initialized_driver(&request);
            let result = (0..1000)
                .find_map(|_| match driver.step() {
                    Ok(head::HeadStep::Consumed) => None,
                    other => Some(other),
                })
                .expect("finite literal driver steps");
            if !matches!(result, Ok(head::HeadStep::Complete)) {
                let stop = driver.control.stop();
                let at = driver.control.first_stop_usage().cloned();
                driver.release().unwrap();
                assert_released(&driver.control);
                assert_eq!(driver.control.first_stop_usage(), at.as_ref());
                panic!("uninstrumented literal must complete: {result:?}, {stop:?}");
            }
            let mut slot = Some(driver);
            let mut completed = StandaloneDocumentDriver::take_completed(&mut slot).unwrap();
            assert!(slot.is_none());
            diagnostics(completed.diagnostics().unwrap(), &source, scan, false);
            native(&mut completed.native_view().unwrap(), scan);
            let mut parts = completed.freeze_parts().unwrap();
            parts.release().unwrap();
            let released = parts.retained_usage();
            assert_eq!(released.actual_capacity_bytes, 0);
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::ManagedCapacityBytes,
                HtmlParseResource::Diagnostics,
                HtmlParseResource::SourceMapEntries,
            ] {
                assert_eq!(released.resource(resource).current, 0);
            }
        }
    }

    fn cancelled(scan: Scan) {
        let (raw, frontier) = input(scan);
        for utf16 in [false, true] {
            let source = if utf16 {
                HtmlSource::utf16(DocumentString::from(raw.as_str()))
            } else {
                HtmlSource::utf8(raw.clone())
            };
            let request = crate::HtmlDocumentParseRequest::offline(source.clone(), limits(100));
            let cancellation = request.cancellation().clone();
            let mut driver = entry::initialized_driver(&request);
            let before = driver.control.usage().clone();
            crate::parser::tokenizer::tests::cancel_after_consumed(
                driver.tokenizer.as_mut().unwrap(),
                frontier,
                cancellation.clone(),
                &mut driver.control,
            )
            .unwrap();
            assert_eq!(
                driver.control.usage().work(WorkCategory::CopyByte)
                    - before.work(WorkCategory::CopyByte),
                std::mem::size_of::<Option<(usize, HtmlParseCancellation)>>() as u64
            );
            assert_eq!(
                driver.control.usage().work(WorkCategory::RecordWrite)
                    - before.work(WorkCategory::RecordWrite),
                1
            );
            assert!(driver.control.owns_request(&cancellation));
            if !utf16 && matches!(scan, Scan::ClosedValue) {
                println!(
                    "complete scan compiled bytes: InputCursor={} Tokenizer={} Driver={} hook={}",
                    std::mem::size_of::<crate::parser::input::InputCursor<'_>>(),
                    std::mem::size_of::<Tokenizer<'_>>(),
                    std::mem::size_of::<StandaloneDocumentDriver<'_>>(),
                    std::mem::size_of::<Option<(usize, HtmlParseCancellation)>>()
                );
            }
            let result = (0..1000).find_map(|_| match driver.step() {
                Ok(head::HeadStep::Consumed) => None,
                other => Some(other),
            });
            assert!(result.unwrap().is_err());
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
            assert!(cancellation.is_cancelled());
            let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
            let range = crate::parser::tokenizer::tests::current_scan_range(
                state.tokenizer.as_ref().unwrap(),
            );
            assert_eq!(range.source(), source.identity());
            assert_eq!(range.coordinates(), source.coordinates());
            assert_eq!(range.end(), frontier);
            assert!(range.end() < source.len());
            diagnostics(
                state
                    .before
                    .kernel
                    .session
                    .observed_evidence()
                    .diagnostics(),
                &source,
                scan,
                true,
            );
            let first = driver.control.first_stop_usage().unwrap().clone();
            let usage = driver.control.usage().clone();
            assert_eq!(first, usage);
            let reads = usage.work(WorkCategory::InputUnitRead);
            assert!((frontier as u64..20 * source.len() as u64).contains(&reads));
            assert!(usage.work(WorkCategory::StateStep) > 0);
            assert!(usage.actual_capacity_bytes > 0);
            assert!(driver.step().is_err());
            assert!(driver.make_available(source.len()).is_err());
            assert!(driver.seal_input().is_err());
            assert_eq!(driver.control.usage(), &usage);
            let mut slot = Some(driver);
            assert!(matches!(
                StandaloneDocumentDriver::take_completed(&mut slot),
                Err(StandaloneError::Stop(ParseStop::Cancelled))
            ));
            let driver = slot
                .as_mut()
                .expect("terminal take cannot publish or consume a partial document");
            assert_eq!(driver.control.usage(), &usage);
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
            assert_eq!(
                driver.control.usage().work(WorkCategory::InputUnitRead),
                reads
            );
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                usage.resource(HtmlParseResource::WorkCredits).current
            );
            assert_eq!(driver.control.first_stop_usage(), Some(&first));
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
            let released = driver.control.usage().clone();
            assert!(driver.step().is_err());
            assert_eq!(driver.control.usage(), &released);
        }
    }

    // HTML877d quoted-value, EOF-in-tag, PI data and duplicate-attribute
    // states supply independent literals; both physical source encodings run.
    #[test]
    fn closed_value_partner() {
        partner(Scan::ClosedValue);
    }
    #[test]
    fn unterminated_value_partner() {
        partner(Scan::UnterminatedValue);
    }
    #[test]
    fn pi_partner() {
        partner(Scan::Pi);
    }
    #[test]
    fn duplicates_partner() {
        partner(Scan::Duplicates);
    }
    #[test]
    fn closed_value_cancelled() {
        cancelled(Scan::ClosedValue);
    }
    #[test]
    fn unterminated_value_cancelled() {
        cancelled(Scan::UnterminatedValue);
    }
    #[test]
    fn pi_cancelled() {
        cancelled(Scan::Pi);
    }
    #[test]
    fn long_pi_cancelled() {
        cancelled(Scan::LongPi);
    }
    #[test]
    fn duplicates_cancelled() {
        cancelled(Scan::Duplicates);
    }

    #[test]
    fn long_pi_has_typed_finite_work_refusal_and_real_cleanup() {
        let (raw, _) = input(Scan::LongPi);
        for utf16 in [false, true] {
            let source = if utf16 {
                HtmlSource::utf16(DocumentString::from(raw.as_str()))
            } else {
                HtmlSource::utf8(raw.clone())
            };
            let request = crate::HtmlDocumentParseRequest::offline(source.clone(), limits(100));
            let mut driver = entry::initialized_driver(&request);
            let result = (0..1000)
                .find_map(|_| match driver.step() {
                    Ok(head::HeadStep::Consumed) => None,
                    other => Some(other),
                })
                .expect("finite literal steps");
            let stop = driver.control.stop();
            let first = driver.control.first_stop_usage().cloned();
            let usage = driver.control.usage().clone();
            println!(
                "4K PI utf16={utf16} result={result:?} reads={} inspect={} copy={} capacity={}",
                usage.work(WorkCategory::InputUnitRead),
                usage.work(WorkCategory::InspectByte),
                usage.work(WorkCategory::CopyByte),
                usage.actual_capacity_bytes
            );
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(driver.control.first_stop_usage(), first.as_ref());
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                usage.resource(HtmlParseResource::WorkCredits).current
            );
            assert!(result.is_err());
            let Some(ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit: 100_000_000,
                current,
                requested_delta,
            }) = stop
            else {
                panic!("real typed finite work refusal: {stop:?}")
            };
            assert!(current + requested_delta > 100_000_000);
            assert!(usage.work(WorkCategory::InputUnitRead) > 4096);
            assert!(!request.cancellation().is_cancelled());
            assert!(driver.step().is_err());
            assert_eq!(driver.control.first_stop_usage(), first.as_ref());
        }
    }

    mod long_numeric_reference {
        use super::*;

        const PREFIX: &str = "<!doctype html><p>é&#0;P</p><p>";

        fn source(hex: bool, digits: usize, utf16: bool) -> HtmlSource {
            let raw = format!(
                "{PREFIX}{}{};Z</p>",
                if hex { "&#x" } else { "&#" },
                if hex { "F" } else { "9" }.repeat(digits)
            );
            if utf16 {
                HtmlSource::utf16(DocumentString::from(raw.as_str()))
            } else {
                HtmlSource::utf8(raw)
            }
        }

        fn reference_start(utf16: bool) -> usize {
            // Fifteen doctype units, p/é/&#0;/P/end-p, then start-p.
            // é occupies one UTF16 unit and two UTF8 bytes.
            if utf16 { 31 } else { 32 }
        }

        fn records(
            view: DiagnosticEvidenceView<'_>,
            source: &HtmlSource,
            hex: bool,
            digits: usize,
            utf16: bool,
            complete: bool,
        ) {
            let count = if complete { 2 } else { 1 };
            assert_eq!(view.len(), count);
            for index in 0..count {
                let (record, location) = view.retained_record(index).unwrap().unwrap();
                let (code, anchor, branch, range) = if index == 0 {
                    (
                        DiagnosticCode::NullCharacterReference,
                        "parse-error-null-character-reference",
                        "null-character-reference",
                        if utf16 { (19, 23) } else { (20, 24) },
                    )
                } else {
                    let start = reference_start(utf16);
                    (
                        DiagnosticCode::CharacterReferenceOutsideUnicodeRange,
                        "parse-error-character-reference-outside-unicode-range",
                        "character-reference-outside-unicode-range",
                        (start, start + digits + if hex { 4 } else { 3 }),
                    )
                };
                assert_eq!(record.kind, DiagnosticKind::Tokenizer(code));
                assert_eq!(
                    record.context,
                    DiagnosticContext::Tokenizer {
                        phase: DiagnosticPhase::Tokenizer,
                        state: State::Data,
                    }
                );
                assert_eq!(record.severity, DiagnosticSeverity::Recoverable);
                assert_eq!((record.rule.anchor, record.rule.branch), (anchor, branch));
                assert_eq!(location.role, DiagnosticLocationRole::InputFrontier);
                assert_eq!(location.token, None);
                assert_eq!(location.source.source(), source.identity());
                assert_eq!(location.source.coordinates(), source.coordinates());
                assert_eq!((location.source.start(), location.source.end()), range);
            }
            assert!(view.retained_record(count).unwrap().is_none());
        }

        fn run(
            driver: &mut StandaloneDocumentDriver<'_>,
        ) -> Result<head::HeadStep, StandaloneError> {
            (0..1000)
                .find_map(|_| match driver.step() {
                    Ok(head::HeadStep::Consumed) => None,
                    other => Some(other),
                })
                .expect("finite literal driver steps")
        }

        fn stopped(
            driver: StandaloneDocumentDriver<'_>,
            source: &HtmlSource,
            hex: bool,
            digits: usize,
            utf16: bool,
            expected: ParseStop,
        ) {
            let mut driver = driver;
            assert_eq!(driver.control.stop(), Some(expected));
            let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
            records(
                state
                    .before
                    .kernel
                    .session
                    .observed_evidence()
                    .diagnostics(),
                source,
                hex,
                digits,
                utf16,
                false,
            );
            let first = driver.control.first_stop_usage().unwrap().clone();
            let usage = driver.control.usage().clone();
            assert_eq!(first, usage);
            assert_eq!(
                usage.resource(HtmlParseResource::SourceUnits).current,
                source.len() as u64
            );
            assert!(usage.actual_capacity_bytes > 0);
            assert!(driver.step().is_err());
            assert!(driver.make_available(source.len()).is_err());
            assert!(driver.seal_input().is_err());
            assert_eq!(driver.control.usage(), &usage);
            let mut slot = Some(driver);
            assert!(
                matches!(StandaloneDocumentDriver::take_completed(&mut slot), Err(StandaloneError::Stop(stop)) if stop == expected)
            );
            let driver = slot
                .as_mut()
                .expect("failed take retains actual partial owner");
            assert_eq!(driver.control.usage(), &usage);
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
            assert_eq!(driver.control.first_stop_usage(), Some(&first));
            assert_eq!(driver.control.stop(), Some(expected));
            for category in [
                WorkCategory::InputUnitRead,
                WorkCategory::StateStep,
                WorkCategory::RecordVisit,
                WorkCategory::RecordWrite,
                WorkCategory::InspectByte,
                WorkCategory::CopyByte,
                WorkCategory::ProviderCall,
            ] {
                assert_eq!(driver.control.usage().work(category), usage.work(category));
            }
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                usage.resource(HtmlParseResource::WorkCredits).current
            );
            let released = driver.control.usage().clone();
            assert!(driver.step().is_err());
            assert_eq!(driver.control.usage(), &released);
        }

        #[test]
        fn decimal_and_hex_complete_partners_have_literal_replacement_and_retained_diagnostics() {
            for hex in [false, true] {
                for utf16 in [false, true] {
                    let source = source(hex, 512, utf16);
                    let request =
                        crate::HtmlDocumentParseRequest::offline(source.clone(), limits(100));
                    let mut driver = entry::initialized_driver(&request);
                    let result = run(&mut driver);
                    if !matches!(result, Ok(head::HeadStep::Complete)) {
                        let stop = driver.control.stop();
                        driver.release().unwrap();
                        assert_released(&driver.control);
                        panic!("uninstrumented literal complete partner: {result:?} {stop:?}");
                    }
                    assert_eq!(
                        driver
                            .control
                            .usage()
                            .resource(HtmlParseResource::SourceUnits)
                            .current,
                        source.len() as u64
                    );
                    let mut slot = Some(driver);
                    let mut completed =
                        StandaloneDocumentDriver::take_completed(&mut slot).unwrap();
                    assert!(slot.is_none());
                    records(
                        completed.diagnostics().unwrap(),
                        &source,
                        hex,
                        512,
                        utf16,
                        true,
                    );
                    {
                        let mut view = completed.native_view().unwrap();
                        let root = view.root().unwrap();
                        assert!(matches!(
                            view.kind(&root).unwrap(),
                            NativeKind::Document(HtmlDocumentMode::NoQuirks)
                        ));
                        assert_eq!(view.child_count(&root).unwrap(), 2);
                        let doctype = progressive_native_node(&mut view, &[0]);
                        let NativeKind::Doctype {
                            name,
                            public_id,
                            system_id,
                        } = view.kind(&doctype).unwrap()
                        else {
                            panic!("literal doctype")
                        };
                        progressive_native_units(&mut view, name, "html");
                        progressive_native_units(&mut view, public_id.unwrap(), "");
                        progressive_native_units(&mut view, system_id.unwrap(), "");
                        let html = element(&mut view, &[1], "html");
                        assert_eq!(view.child_count(&html).unwrap(), 2);
                        let head = element(&mut view, &[1, 0], "head");
                        assert_eq!(view.child_count(&head).unwrap(), 0);
                        let body = element(&mut view, &[1, 1], "body");
                        assert_eq!(view.child_count(&body).unwrap(), 2);
                        let mut previous = None;
                        for (index, expected) in ["é\u{fffd}P", "\u{fffd}Z"].into_iter().enumerate()
                        {
                            let p = element(&mut view, &[1, 1, index], "p");
                            assert_eq!(view.attribute_count(&p).unwrap(), 0);
                            assert_eq!(view.child_count(&p).unwrap(), 1);
                            let text = progressive_native_node(&mut view, &[1, 1, index, 0]);
                            if let Some(previous) = previous.replace(text.clone()) {
                                assert!(!view.same_node(&previous, &text).unwrap());
                            }
                            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                                panic!("literal text")
                            };
                            progressive_native_units(&mut view, value, expected);
                        }
                    }
                    let mut parts = completed.freeze_parts().unwrap();
                    parts.release().unwrap();
                    assert_eq!(parts.retained_usage().actual_capacity_bytes, 0);
                    for resource in [
                        HtmlParseResource::BufferedUtf16Units,
                        HtmlParseResource::TokenAttributes,
                        HtmlParseResource::AuxiliaryEntries,
                        HtmlParseResource::ManagedEdges,
                        HtmlParseResource::ManagedAttributes,
                        HtmlParseResource::ManagedUtf16Units,
                        HtmlParseResource::SourceMapEntries,
                        HtmlParseResource::Diagnostics,
                        HtmlParseResource::ContextEffects,
                        HtmlParseResource::OpenElements,
                        HtmlParseResource::ManagedCapacityBytes,
                        HtmlParseResource::PeakTransientBytes,
                    ] {
                        assert_eq!(parts.retained_usage().resource(resource).current, 0);
                    }
                }
            }
        }

        #[test]
        fn decimal_and_hex_complete_cancel_at_exact_original_digit_frontier() {
            for hex in [false, true] {
                for utf16 in [false, true] {
                    let source = source(hex, 8192, utf16);
                    let request =
                        crate::HtmlDocumentParseRequest::offline(source.clone(), limits(100));
                    let cancellation = request.cancellation().clone();
                    let mut driver = entry::initialized_driver(&request);
                    let frontier = reference_start(utf16) + if hex { 3 } else { 2 } + 1024;
                    let before = driver.control.usage().clone();
                    crate::parser::tokenizer::tests::cancel_after_consumed(
                        driver.tokenizer.as_mut().unwrap(),
                        frontier,
                        cancellation.clone(),
                        &mut driver.control,
                    )
                    .unwrap();
                    assert_eq!(
                        driver.control.usage().work(WorkCategory::CopyByte)
                            - before.work(WorkCategory::CopyByte),
                        std::mem::size_of::<Option<(usize, HtmlParseCancellation)>>() as u64
                    );
                    assert_eq!(
                        driver.control.usage().work(WorkCategory::RecordWrite)
                            - before.work(WorkCategory::RecordWrite),
                        1
                    );
                    assert!(driver.control.owns_request(&cancellation));
                    assert!(run(&mut driver).is_err());
                    assert!(cancellation.is_cancelled());
                    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
                    let range = crate::parser::tokenizer::tests::current_scan_range(
                        state.tokenizer.as_ref().unwrap(),
                    );
                    assert_eq!(range.source(), source.identity());
                    assert_eq!(range.coordinates(), source.coordinates());
                    assert_eq!(range.end(), frontier);
                    assert!(range.end() < source.len());
                    stopped(driver, &source, hex, 8192, utf16, ParseStop::Cancelled);
                }
            }
        }

        #[test]
        fn decimal_and_hex_complete_digit_action_floor_forces_finite_work_refusal() {
            const DIGITS: usize = 90_000;
            // char_ref NumericDigits charges at least StateStep1, two ASCII
            // InputUnitRead (peek/read), CopyByte(sizeof(bool)+sizeof(u32))
            // and two actual accumulator writes per consumed digit. Other
            // inspection/range/receiver/prefix fees are positive. The digit-only
            // floor therefore cannot finish under this request-local budget.
            let per_digit = 1 + 2 + std::mem::size_of::<bool>() + std::mem::size_of::<u32>() + 2;
            let budget = (DIGITS * per_digit) as u64;
            for hex in [false, true] {
                for utf16 in [false, true] {
                    let source = source(hex, DIGITS, utf16);
                    let selected = limits(100)
                        .into_builder()
                        .work_credits(budget)
                        .finish()
                        .unwrap();
                    let request =
                        crate::HtmlDocumentParseRequest::offline(source.clone(), selected);
                    let mut driver = entry::initialized_driver(&request);
                    assert!(run(&mut driver).is_err());
                    let stop = driver.control.stop().unwrap();
                    let ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        limit,
                        current,
                        requested_delta,
                    } = stop
                    else {
                        panic!("digit action work refusal: {stop:?}")
                    };
                    assert_eq!(limit, budget);
                    assert!(current + requested_delta > limit);
                    assert!(!request.cancellation().is_cancelled());
                    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
                    let range = crate::parser::tokenizer::tests::current_scan_range(
                        state.tokenizer.as_ref().unwrap(),
                    );
                    assert_eq!(range.source(), source.identity());
                    assert_eq!(range.coordinates(), source.coordinates());
                    assert!(range.end() > reference_start(utf16) + if hex { 3 } else { 2 });
                    assert!(
                        range.end() < reference_start(utf16) + DIGITS + if hex { 4 } else { 3 }
                    );
                    stopped(driver, &source, hex, DIGITS, utf16, stop);
                }
            }
        }
    }
}

#[test]
fn the_fresh_head_transition_without_retained_lexical_evidence_reports_consumed() {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<html>"), limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    // Private-state fault injection: the driver's lexical evidence owner is gone
    // before the one step that hands the owners to the head stage.
    driver
        .lexical
        .take()
        .unwrap()
        .release(&mut driver.control)
        .unwrap();
    let error = driver.step().err().unwrap();
    assert!(matches!(error, StandaloneError::Consumed), "{error:?}");
    // A lost owner is an impossible state, not an unsupported document: the
    // driver latches it as an invariant stop, and that first stop is the one
    // cleanup runs under.
    assert_eq!(
        driver.control.stop(),
        Some(ParseStop::InternalInvariant(Invariant::RegistryRow))
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}

#[test]
fn a_token_event_is_taken_only_from_a_token_slot_and_other_content_stays_for_release() {
    use crate::parser::input::PreprocessingRecord;
    let source = HtmlSource::utf8("a\r\nb");
    let mut c = control(100);
    c.admit_source(&source).unwrap();
    let mut empty = None;
    let stop = take_token_event(&mut empty, &mut c).err().unwrap();
    assert_eq!(
        stop,
        ParseStop::InternalInvariant(Invariant::InputCursorState)
    );
    assert_eq!(c.stop(), Some(stop));
    assert!(empty.is_none());

    let mut other = Some(Event::Preprocessing {
        _record: PreprocessingRecord::Newline { was_crlf: true },
        _source: source.range(1, 3).unwrap(),
        retention: OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap(),
    });
    // The first stop stays the first stop; the content is still in its slot.
    let again = take_token_event(&mut other, &mut c).err().unwrap();
    assert_eq!(again, stop);
    other.take().unwrap().release(&mut c).unwrap();
    assert_released(&c);
}

#[test]
fn only_invariant_faults_are_classified_as_invariant_faults() {
    use super::head::HeadError;
    use super::prologue::PrologueError;
    let lost = [
        StandaloneError::Consumed,
        StandaloneError::Head(HeadError::Consumed),
        StandaloneError::Head(HeadError::MissingInput),
        StandaloneError::Head(HeadError::MissingTokenizer),
        StandaloneError::Head(HeadError::InvalidFormattingState),
        StandaloneError::Head(HeadError::InvalidTextToken),
        StandaloneError::Head(HeadError::MissingReturnMode),
        StandaloneError::Head(HeadError::InvalidScriptNesting),
        StandaloneError::Head(HeadError::TokenizerExhausted),
        StandaloneError::Head(HeadError::AlreadyAtBoundary),
        StandaloneError::TokenizerExhausted,
        StandaloneError::Head(HeadError::Kernel(KernelError::Consumed)),
        StandaloneError::Head(HeadError::Prologue(PrologueError::Consumed)),
        StandaloneError::Head(HeadError::Prologue(PrologueError::MissingInput)),
        StandaloneError::Head(HeadError::Prologue(PrologueError::AlreadyAtBeforeHead)),
        StandaloneError::Head(HeadError::Prologue(PrologueError::PendingInput)),
        StandaloneError::Prologue(PrologueError::Consumed),
        StandaloneError::Prologue(PrologueError::MissingInput),
        StandaloneError::Prologue(PrologueError::AlreadyAtBeforeHead),
        StandaloneError::Prologue(PrologueError::PendingInput),
        StandaloneError::Prologue(PrologueError::Kernel(KernelError::Consumed)),
    ];
    for error in &lost {
        assert!(error.invariant_fault(), "{error:?}");
    }
    let named = [
        StandaloneError::UnsupportedDocumentInitialization,
        StandaloneError::UnsupportedExecutionProfile,
        StandaloneError::Stop(ParseStop::Cancelled),
        StandaloneError::Head(HeadError::UnsupportedScriptPause),
        StandaloneError::Head(HeadError::UnsupportedPendingScript),
        StandaloneError::Head(HeadError::IncompleteCompletion),
        StandaloneError::Head(HeadError::ConstructionCapabilityUnavailable),
        StandaloneError::Head(HeadError::Handoff(
            crate::parser::tokenizer::TokenizerHandoffError::PendingEvents,
        )),
        StandaloneError::Head(HeadError::Kernel(KernelError::Stop(ParseStop::Cancelled))),
        StandaloneError::Head(HeadError::Kernel(KernelError::EmptyStack)),
        StandaloneError::Head(HeadError::Kernel(KernelError::NotElement)),
        StandaloneError::Head(HeadError::Kernel(KernelError::FirstOpenElementNotHtml)),
        StandaloneError::Head(HeadError::Kernel(KernelError::DuplicateOpenElement)),
        StandaloneError::Head(HeadError::Kernel(KernelError::Session(
            SessionError::ScriptMetadataUnavailable,
        ))),
        StandaloneError::Head(HeadError::Kernel(KernelError::Session(
            SessionError::WrongToken,
        ))),
        StandaloneError::Prologue(PrologueError::Kernel(KernelError::Stop(
            ParseStop::Cancelled,
        ))),
    ];
    for error in &named {
        assert!(!error.invariant_fault(), "{error:?}");
    }
}

#[test]
fn an_ordinary_completion_without_its_stage_state_latches_the_registry_row() {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), limits(100));
    let mut driver = entry::initialized_driver(&request);
    for _ in 0..100 {
        if driver.head.is_some() {
            break;
        }
        driver.step().unwrap();
    }
    // Private-state fault injection: the stage's state is not where completion
    // expects it, and the refusal is not a document or a limit.
    let state = driver.head.as_mut().unwrap().state.take().unwrap();
    let mut slot = Some(driver);
    let error = StandaloneDocumentDriver::take_completed(&mut slot)
        .err()
        .unwrap();
    assert!(
        matches!(error, StandaloneError::Head(head::HeadError::Consumed)),
        "{error:?}"
    );
    let mut driver = slot.unwrap();
    let stop = ParseStop::InternalInvariant(Invariant::RegistryRow);
    assert_eq!(driver.control.stop(), Some(stop));
    driver.head.as_mut().unwrap().state = Some(state);
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.stop(), Some(stop));
}

#[test]
fn enabling_meta_csp_without_the_prologue_latches_the_registry_row() {
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), limits(100));
    let mut driver = entry::initialized_driver(&request);
    // Private-state fault injection: the prologue the processor attaches to is
    // gone, which is not an unsupported document.
    let prologue = driver.prologue.take().unwrap();
    let error = driver.enable_meta_csp(false).err().unwrap();
    assert!(matches!(error, StandaloneError::Consumed), "{error:?}");
    let stop = ParseStop::InternalInvariant(Invariant::RegistryRow);
    assert_eq!(driver.control.stop(), Some(stop));
    driver.prologue = Some(prologue);
    driver.release().unwrap();
    assert_released(&driver.control);
    assert_eq!(driver.control.stop(), Some(stop));
}

#[test]
fn stepping_a_driver_whose_stage_waits_at_a_boundary_latches_the_registry_row() {
    // The parse functions stop at the first boundary a stage returns, and the
    // driver itself consumes the after-head one, so a driver stepped while its
    // stage waits at a boundary has lost its place; no document puts it there.
    let request = crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), limits(100));
    let mut driver = entry::initialized_driver(&request);
    for _ in 0..100 {
        if driver.head.is_some() {
            break;
        }
        driver.step().unwrap();
    }
    // Private-state fault injection: the stage is left at a boundary.
    driver
        .head
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .boundary = Some(super::head::HeadBoundary::Script);
    assert_eq!(driver.control.stop(), None);
    let error = driver.step().err().unwrap();
    assert!(
        matches!(
            error,
            StandaloneError::Head(super::head::HeadError::AlreadyAtBoundary)
        ),
        "{error:?}"
    );
    assert_eq!(
        driver.control.stop(),
        Some(ParseStop::InternalInvariant(Invariant::RegistryRow))
    );
    driver.release().unwrap();
    assert_released(&driver.control);
}
