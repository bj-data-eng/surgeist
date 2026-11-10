//! `ReplaceContent` and `ReplaceAttributes` replace one record's payload in
//! place: they never change topology, identity or the record count, and the
//! usage they leave is the usage of the replaced payload. The contract is the
//! construction reference's operation table
//! (`docs/reference/html-crate-contracts.md`, "Construction operation payloads
//! and failures"); the expected UTF-16 usage is derived here from the literals,
//! never read back from the owner.
use super::*;
use crate::html::support::cause;
use surgeist_html::{
    construction::{HtmlAuthoredProcessingInstruction, HtmlDoctype, HtmlProcessingInstruction},
    elements::link::HtmlLinkFlags,
    values::Attribute,
};

const HTML_URI: &str = "http://www.w3.org/1999/xhtml";
const XLINK_URI: &str = "http://www.w3.org/1999/xlink";

/// UTF-16 code units of a literal: the unit usage is charged in.
fn units(value: &str) -> usize {
    value.encode_utf16().count()
}
/// Units of an expanded-name triple: namespace URI, optional prefix, local name.
fn name_units(uri: Option<&str>, prefix: Option<&str>, local: &str) -> usize {
    uri.map_or(0, units) + prefix.map_or(0, units) + units(local)
}
fn text(value: &str) -> DocumentString {
    DocumentString::from(value)
}
fn attribute(namespace: Namespace, prefix: Option<&str>, local: &str, value: &str) -> Attribute {
    Attribute::new(
        QualifiedName::new(namespace, prefix.map(text), local),
        value,
    )
}
fn attributes(values: impl IntoIterator<Item = Attribute>) -> OrderedAttributes {
    OrderedAttributes::new(values).unwrap()
}
fn name(namespace: Namespace, prefix: Option<&str>, local: &str) -> QualifiedName {
    QualifiedName::new(namespace, prefix.map(text), local)
}
fn create_kind(
    owner: HtmlConstructionStorage,
    kind: HtmlNodeKind,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(owner, HtmlConstructionCommand::Create { kind })
}
fn element_with(
    owner: HtmlConstructionStorage,
    name: QualifiedName,
    values: OrderedAttributes,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create_kind(owner, HtmlNodeKind::Element(HtmlElement::new(name, values)))
}
fn insert(
    owner: HtmlConstructionStorage,
    node: &HtmlBuilderNode,
    parent: &HtmlBuilderNode,
) -> HtmlConstructionStorage {
    change(
        owner,
        HtmlConstructionCommand::Insert {
            node: *node,
            parent: *parent,
            before: None,
        },
    )
    .0
}
/// The five usage counters as one comparable value.
fn counters(owner: &HtmlConstructionStorage) -> [usize; 5] {
    let usage = owner.usage();
    [
        usage.nodes(),
        usage.edges(),
        usage.attributes(),
        usage.utf16_units(),
        usage.depth(),
    ]
}
fn element_of(owner: &HtmlConstructionStorage, node: &HtmlBuilderNode) -> HtmlElement {
    match owner.node(node).unwrap().kind() {
        HtmlNodeKind::Element(element) => element.clone(),
        other => panic!("expected an element, found {other:?}"),
    }
}
fn child_handles(
    owner: &HtmlConstructionStorage,
    parent: &HtmlBuilderNode,
) -> Vec<HtmlBuilderNode> {
    owner
        .node(parent)
        .unwrap()
        .children()
        .map(HtmlConstructionNode::handle)
        .collect()
}
fn pi(target: &str, data: &str) -> HtmlProcessingInstruction {
    HtmlAuthoredProcessingInstruction::new(target, data)
        .unwrap()
        .into()
}

/// A fragment holding `[before, target, after]`, where `before` and `after`
/// are text siblings whose order the replacement must not disturb.
struct Row {
    owner: HtmlConstructionStorage,
    root: HtmlBuilderNode,
    siblings: [HtmlBuilderNode; 3],
}
fn row(kind: HtmlNodeKind) -> Row {
    let (owner, root) = fragment(storage());
    let (owner, before) = create_kind(owner, HtmlNodeKind::Text(text("before")));
    let (owner, target) = create_kind(owner, kind);
    let (owner, after) = create_kind(owner, HtmlNodeKind::Text(text("after")));
    let owner = insert(owner, &before, &root);
    let owner = insert(owner, &target, &root);
    let owner = insert(owner, &after, &root);
    Row {
        owner,
        root,
        siblings: [before, target, after],
    }
}
/// Performs `performed` actions of `command`, then aborts the pending
/// operation; returns whether no action remained, the returned storage and the
/// returned command.
fn aborted_pending(
    owner: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    performed: usize,
) -> (bool, HtmlConstructionStorage, HtmlConstructionCommand) {
    let mut operation = Some(owner.prepare(command));
    for _ in 0..performed {
        match operation.as_mut().unwrap().next_action().unwrap() {
            Some(action) => action.perform().unwrap().accept(),
            None => break,
        }
    }
    let finished = operation.as_mut().unwrap().next_action().unwrap().is_none();
    let (owner, command, retired) = operation.unwrap().abort().unwrap().into_parts();
    assert!(retired.is_none(), "a mutation has no publication to retire");
    (finished, owner, command)
}

/// Aborts the command `fresh` builds after every possible count of performed
/// actions, and once more as a ready operation. `unchanged` must hold after
/// every abort, and the returned command must be the original change, which
/// then commits.
fn assert_rollback_at_every_action<P>(
    fresh: impl Fn() -> (HtmlConstructionStorage, HtmlConstructionCommand, P),
    unchanged: impl Fn(&HtmlConstructionStorage, &P, &str),
    committed: impl Fn(&HtmlConstructionStorage, &P),
) {
    let mut performed = 0;
    loop {
        let (owner, command, probe) = fresh();
        let (finished, owner, returned) = aborted_pending(owner, command, performed);
        unchanged(
            &owner,
            &probe,
            &format!("aborted after {performed} actions"),
        );
        let retry = change(owner, returned).0;
        committed(&retry, &probe);
        if finished {
            break;
        }
        performed += 1;
    }
    assert!(performed > 0, "a replacement is checked before it commits");
    // A ready operation holds the complete plan and still applies nothing.
    let (owner, command, probe) = fresh();
    let (owner, returned, retired) = ready(owner, command).abort().unwrap().into_parts();
    assert!(retired.is_none());
    unchanged(&owner, &probe, "aborted when ready");
    committed(&change(owner, returned).0, &probe);
}

// ---------------------------------------------------------------- content

type KindFactory = fn() -> HtmlNodeKind;
type ContentFactory = fn() -> HtmlConstructionContent;

/// One content replacement: the initial payload, the replacement, the payload
/// the record must hold afterwards and the UTF-16 units each charges.
struct ContentCase {
    label: &'static str,
    initial: HtmlNodeKind,
    content: ContentFactory,
    expected: HtmlNodeKind,
    old_units: usize,
    new_units: usize,
}
fn content_cases() -> Vec<ContentCase> {
    vec![
        // "a😀" is three UTF-16 units: the supplementary scalar is a pair.
        ContentCase {
            label: "text",
            initial: HtmlNodeKind::Text(text("a\u{1F600}")),
            content: || HtmlConstructionContent::Text(text("hello \u{1F600}\u{1F600} world")),
            expected: HtmlNodeKind::Text(text("hello \u{1F600}\u{1F600} world")),
            old_units: 3,
            new_units: 6 + 2 + 2 + 6,
        },
        ContentCase {
            label: "comment to the empty string",
            initial: HtmlNodeKind::Comment(text("remark")),
            content: || HtmlConstructionContent::Comment(text("")),
            expected: HtmlNodeKind::Comment(text("")),
            old_units: 6,
            new_units: 0,
        },
        ContentCase {
            label: "processing instruction",
            initial: HtmlNodeKind::ProcessingInstruction(pi("tool", "href=\"a\"")),
            content: || HtmlConstructionContent::ProcessingInstruction(pi("target", "d\u{1F600}")),
            expected: HtmlNodeKind::ProcessingInstruction(pi("target", "d\u{1F600}")),
            old_units: 4 + 8,
            new_units: 6 + 3,
        },
        ContentCase {
            label: "doctype",
            initial: HtmlNodeKind::Doctype(HtmlDoctype::new("html", None, None)),
            content: || {
                HtmlConstructionContent::Doctype(HtmlDoctype::new(
                    "svg",
                    Some(text("-//W3C//DTD SVG 1.1//EN")),
                    Some(text("http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd")),
                ))
            },
            expected: HtmlNodeKind::Doctype(HtmlDoctype::new(
                "svg",
                Some(text("-//W3C//DTD SVG 1.1//EN")),
                Some(text("http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd")),
            )),
            old_units: 4,
            new_units: 3 + 23 + 48,
        },
    ]
}

#[test]
fn replacing_content_commits_the_payload_and_its_utf16_usage_without_touching_topology() {
    for case in content_cases() {
        let Row {
            owner,
            root,
            siblings,
        } = row(case.initial);
        let before = counters(&owner);
        let target = siblings[1];
        let (owner, created) = change(
            owner,
            HtmlConstructionCommand::ReplaceContent {
                node: target,
                content: (case.content)(),
            },
        );
        assert!(
            created.is_none(),
            "{}: replacement creates no node",
            case.label
        );
        assert_eq!(
            owner.node(&target).unwrap().kind(),
            &case.expected,
            "{}",
            case.label
        );
        // Nodes, edges, attributes and depth are untouched; only the units move.
        assert_eq!(
            counters(&owner),
            [
                before[0],
                before[1],
                before[2],
                before[3] - case.old_units + case.new_units,
                before[4]
            ],
            "{}",
            case.label
        );
        // The same record keeps its place among the same siblings.
        assert_eq!(child_handles(&owner, &root), siblings, "{}", case.label);
        assert_eq!(
            owner.node(&target).unwrap().parent().unwrap().handle(),
            root,
            "{}",
            case.label
        );
        // The published tree carries the replacement.
        let target_id = owner.result_id(&target).unwrap();
        let (result, _) = publish(owner, root);
        assert_eq!(
            result.node(&target_id).unwrap().kind(),
            &case.expected,
            "{}",
            case.label
        );
    }
}

#[test]
fn replacing_content_with_the_current_payload_changes_nothing_observable() {
    let Row {
        owner,
        root,
        siblings,
    } = row(HtmlNodeKind::Text(text("same")));
    let before = counters(&owner);
    let target = siblings[1];
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::ReplaceContent {
            node: target,
            content: HtmlConstructionContent::Text(text("same")),
        },
    );
    assert_eq!(counters(&owner), before);
    assert_eq!(
        owner.node(&target).unwrap().kind(),
        &HtmlNodeKind::Text(text("same"))
    );
    assert_eq!(child_handles(&owner, &root), siblings);
}

#[test]
fn replacing_an_element_name_keeps_attributes_children_and_position_and_charges_only_the_name() {
    let values = attributes([
        attribute(Namespace::none(), None, "id", "main"),
        attribute(Namespace::from_uri(XLINK_URI), Some("xlink"), "href", "#a"),
    ]);
    let backing = values.shared_attributes().clone();
    let (owner, root) = fragment(storage());
    let (owner, target) = element_with(owner, name(Namespace::html(), None, "div"), values);
    let (owner, child) = create_kind(owner, HtmlNodeKind::Text(text("inside")));
    let owner = insert(owner, &child, &target);
    let owner = insert(owner, &target, &root);
    let before = counters(&owner);
    let old_name = name_units(Some(HTML_URI), None, "div");
    let svg = Namespace::svg();
    let new_uri = "http://www.w3.org/2000/svg";
    let new_name = name_units(Some(new_uri), Some("s\u{1F600}"), "section");
    let (owner, _) = change(
        owner,
        HtmlConstructionCommand::ReplaceContent {
            node: target,
            content: HtmlConstructionContent::ElementName(name(svg, Some("s\u{1F600}"), "section")),
        },
    );
    let element = element_of(&owner, &target);
    assert!(
        element
            .name()
            .namespace()
            .uri()
            .is_some_and(|uri| *uri == text(new_uri))
    );
    assert_eq!(element.name().prefix(), Some(&text("s\u{1F600}")));
    assert_eq!(element.name().local_name(), &text("section"));
    // The attributes are the original backing, not a copy.
    assert!(Arc::ptr_eq(
        element.attributes().shared_attributes(),
        &backing
    ));
    assert_eq!(element.attributes().len(), 2);
    assert_eq!(
        counters(&owner),
        [
            before[0],
            before[1],
            before[2],
            before[3] - old_name + new_name,
            before[4]
        ]
    );
    assert_eq!(child_handles(&owner, &target), std::slice::from_ref(&child));
    assert_eq!(child_handles(&owner, &root), std::slice::from_ref(&target));
    assert_eq!(
        owner.node(&child).unwrap().parent().unwrap().handle(),
        target
    );
}

fn link_element(
    owner: HtmlConstructionStorage,
    flags: Option<HtmlLinkFlags>,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(
        owner,
        HtmlConstructionCommand::CreateElement {
            name: name(Namespace::html(), None, "link"),
            attributes: OrderedAttributes::default(),
            link_flags: flags,
        },
    )
}
fn rename(node: &HtmlBuilderNode, to: QualifiedName) -> HtmlConstructionCommand {
    HtmlConstructionCommand::ReplaceContent {
        node: *node,
        content: HtmlConstructionContent::ElementName(to),
    }
}

#[test]
fn renaming_a_link_to_the_same_expanded_name_keeps_its_link_history() {
    let (owner, target) = link_element(storage(), Some(HtmlLinkFlags::fresh()));
    // A prefix leaves the expanded name, so the history still applies.
    let (owner, _) = change(
        owner,
        rename(&target, name(Namespace::html(), Some("p"), "link")),
    );
    let element = element_of(&owner, &target);
    assert_eq!(element.name().prefix(), Some(&text("p")));
    assert_eq!(element.link_flags(), Some(HtmlLinkFlags::fresh()));
}

#[test]
fn renaming_a_link_away_from_the_html_link_expanded_name_drops_its_link_history() {
    // Contract: `docs/reference/html-crate-contracts.md`, operation table,
    // `ReplaceContent` row (line 492): link history survives only while the
    // new name is an HTML `link` expanded name, and is otherwise cleared. It
    // follows `CreateElement::link_flags`, which no other element may carry.
    for (namespace, local) in [
        (Namespace::html(), "div"),
        (Namespace::svg(), "link"),
        (Namespace::html(), "LINK"),
    ] {
        let (owner, target) = link_element(storage(), Some(HtmlLinkFlags::fresh()));
        let (owner, _) = change(owner, rename(&target, name(namespace, None, local)));
        assert_eq!(
            element_of(&owner, &target).link_flags(),
            None,
            "{local} cannot hold link history"
        );
    }
}

#[test]
fn renaming_a_non_link_to_a_link_starts_without_link_history() {
    let (owner, plain) = element_with(
        storage(),
        name(Namespace::html(), None, "div"),
        OrderedAttributes::default(),
    );
    let (owner, _) = change(owner, rename(&plain, name(Namespace::html(), None, "link")));
    assert_eq!(element_of(&owner, &plain).link_flags(), None);
}

#[test]
fn renaming_a_template_keeps_its_contents_only_while_it_stays_a_template() {
    // Contract: `docs/reference/html-crate-contracts.md`, operation table,
    // `ReplaceContent` row (line 492): a new element name keeps the record's
    // template-ness or the operation is refused with
    // `InvalidTemplateAssociation`. The rule is the pinned HTML one that each
    // template element, and no other, has template contents
    // (references/html-language-elements-interactive-scripting--877d6146487f
    // --05bb1d85cc89.md, "template contents").
    let (owner, template) = create(storage(), element("template"));
    let contents = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let usage = counters(&owner);
    let (owner, _) = change(
        owner,
        rename(&template, name(Namespace::html(), Some("p"), "template")),
    );
    assert_eq!(
        owner
            .node(&template)
            .unwrap()
            .template_contents()
            .unwrap()
            .handle(),
        contents
    );
    assert_eq!(
        owner.node(&template).unwrap().template_contents_kind(),
        Some(HtmlTemplateContentsKind::Owned)
    );
    // Only the prefix is new.
    assert_eq!(counters(&owner)[3], usage[3] + units("p"));
    // Renaming it to a non-template would strand its contents.
    let owner = refuse(
        owner,
        rename(&template, name(Namespace::html(), None, "div")),
        cause::INVALID_TEMPLATE_ASSOCIATION,
    );
    assert_eq!(counters(&owner)[3], usage[3] + units("p"));
    assert!(element_of(&owner, &template).name().local_name() == &text("template"));
    assert_eq!(
        owner.node(&template).unwrap().template_contents_kind(),
        Some(HtmlTemplateContentsKind::Owned)
    );
}

#[test]
fn renaming_a_non_template_to_a_template_is_refused_because_it_has_no_contents() {
    // Contract: the `ReplaceContent` row of the operation table (line 492); a
    // template element must have template contents and this record has none.
    let (owner, plain) = create(storage(), element("div"));
    let usage = counters(&owner);
    let owner = refuse(
        owner,
        rename(&plain, name(Namespace::html(), None, "template")),
        cause::INVALID_TEMPLATE_ASSOCIATION,
    );
    assert_eq!(counters(&owner), usage);
    assert!(element_of(&owner, &plain).name().local_name() == &text("div"));
    assert!(owner.node(&plain).unwrap().template_contents().is_none());
}

#[test]
fn replacing_content_of_the_wrong_kind_is_refused_and_leaves_the_record_unchanged() {
    // Every payload kind refuses every other kind of replacement.
    let initial: Vec<(&str, KindFactory)> = vec![
        ("text", || HtmlNodeKind::Text(text("t"))),
        ("comment", || HtmlNodeKind::Comment(text("c"))),
        ("processing instruction", || {
            HtmlNodeKind::ProcessingInstruction(pi("target", "data"))
        }),
        ("doctype", || {
            HtmlNodeKind::Doctype(HtmlDoctype::new("html", None, None))
        }),
        ("element", || {
            HtmlNodeKind::Element(HtmlElement::new(
                name(Namespace::html(), None, "span"),
                OrderedAttributes::default(),
            ))
        }),
    ];
    let replacements: Vec<(&str, ContentFactory)> = vec![
        ("text", || HtmlConstructionContent::Text(text("x"))),
        ("comment", || HtmlConstructionContent::Comment(text("x"))),
        ("processing instruction", || {
            HtmlConstructionContent::ProcessingInstruction(pi("x", "y"))
        }),
        ("doctype", || {
            HtmlConstructionContent::Doctype(HtmlDoctype::new("x", None, None))
        }),
        ("element", || {
            HtmlConstructionContent::ElementName(name(Namespace::html(), None, "x"))
        }),
    ];
    for (held, kind) in &initial {
        for (offered, content) in &replacements {
            if held == offered {
                continue;
            }
            let Row {
                owner,
                root,
                siblings,
            } = row(kind());
            let before = counters(&owner);
            let target = siblings[1];
            let owner = refuse(
                owner,
                HtmlConstructionCommand::ReplaceContent {
                    node: target,
                    content: content(),
                },
                cause::WRONG_NODE_KIND,
            );
            assert_eq!(counters(&owner), before, "{held} refused {offered}");
            assert_eq!(
                owner.node(&target).unwrap().kind(),
                &kind(),
                "{held} refused {offered}"
            );
            assert_eq!(child_handles(&owner, &root), siblings);
        }
    }
}

#[test]
fn replacing_the_content_of_a_root_or_foreign_record_is_refused() {
    // Roots hold no replaceable payload: "roots, topology and associations
    // cannot be replaced by this operation".
    let (owner, root) = fragment(storage());
    let (owner, document) = create(
        owner,
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::NoQuirks,
        },
    );
    let before = counters(&owner);
    let mut owner = owner;
    for node in [&root, &document] {
        owner = refuse(
            owner,
            HtmlConstructionCommand::ReplaceContent {
                node: *node,
                content: HtmlConstructionContent::Text(text("x")),
            },
            cause::WRONG_NODE_KIND,
        );
    }
    assert_eq!(counters(&owner), before);
    assert!(matches!(
        owner.node(&root).unwrap().kind(),
        HtmlNodeKind::Fragment
    ));
    assert!(matches!(
        owner.node(&document).unwrap().kind(),
        HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks)
    ));
    // A handle of another storage is not an operand of this one.
    let (other, foreign) = create_kind(storage(), HtmlNodeKind::Text(text("foreign")));
    let owner = refuse(
        owner,
        HtmlConstructionCommand::ReplaceContent {
            node: foreign,
            content: HtmlConstructionContent::Text(text("x")),
        },
        cause::FOREIGN_BUILDER,
    );
    assert_eq!(counters(&owner), before);
    assert_eq!(
        other.node(&foreign).unwrap().kind(),
        &HtmlNodeKind::Text(text("foreign"))
    );
}

#[test]
fn aborting_a_content_replacement_after_any_action_changes_nothing_and_returns_the_command() {
    for case in content_cases() {
        let label = case.label;
        let initial = case.initial.clone();
        let expected = case.expected.clone();
        let content = case.content;
        assert_rollback_at_every_action(
            || {
                let Row {
                    owner, siblings, ..
                } = row(initial.clone());
                let before = counters(&owner);
                let target = siblings[1];
                (
                    owner,
                    HtmlConstructionCommand::ReplaceContent {
                        node: target,
                        content: content(),
                    },
                    (target, before),
                )
            },
            |owner, (target, before), when| {
                assert_eq!(counters(owner), *before, "{label} {when}");
                assert_eq!(
                    owner.node(target).unwrap().kind(),
                    &initial,
                    "{label} {when}"
                );
            },
            |owner, (target, before)| {
                assert_eq!(owner.node(target).unwrap().kind(), &expected, "{label}");
                assert_eq!(
                    owner.usage().utf16_units(),
                    before[3] - case.old_units + case.new_units,
                    "{label}"
                );
            },
        );
    }
}

// ------------------------------------------------------------- attributes

/// UTF-16 units one attribute charges: its expanded name and its value.
fn attribute_units(uri: Option<&str>, prefix: Option<&str>, local: &str, value: &str) -> usize {
    name_units(uri, prefix, local) + units(value)
}
fn replace_attributes(
    element: &HtmlBuilderNode,
    values: OrderedAttributes,
) -> HtmlConstructionCommand {
    HtmlConstructionCommand::ReplaceAttributes {
        element: *element,
        attributes: values.into(),
    }
}
fn observed(values: &OrderedAttributes) -> Vec<(Option<String>, Option<String>, String, String)> {
    values
        .iter()
        .map(|attribute| {
            let name = attribute.name();
            (
                name.namespace().uri().map(|uri| uri.to_utf8().unwrap()),
                name.prefix().map(|prefix| prefix.to_utf8().unwrap()),
                name.local_name().to_utf8().unwrap(),
                attribute.value().to_utf8().unwrap(),
            )
        })
        .collect()
}
fn expected(
    rows: &[(Option<&str>, Option<&str>, &str, &str)],
) -> Vec<(Option<String>, Option<String>, String, String)> {
    rows.iter()
        .map(|(uri, prefix, local, value)| {
            (
                uri.map(str::to_owned),
                prefix.map(str::to_owned),
                (*local).to_owned(),
                (*value).to_owned(),
            )
        })
        .collect()
}

/// An element holding `old` attributes between two siblings, with one child.
struct Host {
    owner: HtmlConstructionStorage,
    root: HtmlBuilderNode,
    element: HtmlBuilderNode,
    child: HtmlBuilderNode,
}
fn host(name: QualifiedName, old: OrderedAttributes) -> Host {
    let (owner, root) = fragment(storage());
    let (owner, element) = element_with(owner, name, old);
    let (owner, child) = create_kind(owner, HtmlNodeKind::Text(text("inside")));
    let owner = insert(owner, &child, &element);
    let owner = insert(owner, &element, &root);
    Host {
        owner,
        root,
        element,
        child,
    }
}
fn old_attributes() -> OrderedAttributes {
    attributes([
        attribute(Namespace::none(), None, "id", "main"),
        attribute(Namespace::none(), None, "class", "a b"),
    ])
}
fn new_attributes() -> OrderedAttributes {
    attributes([
        attribute(
            Namespace::from_uri(XLINK_URI),
            Some("xlink"),
            "href",
            "#\u{1F600}",
        ),
        attribute(Namespace::none(), None, "id", "next"),
        attribute(Namespace::none(), None, "title", ""),
    ])
}
const OLD_UNITS: usize = 2 + 4 + 5 + 3;
// xlink namespace (28) + "xlink" (5) + "href" (4) + "#" and a pair (3);
// "id" (2) + "next" (4); "title" (5) + "".
const NEW_UNITS: usize = 28 + 5 + 4 + 3 + 2 + 4 + 5;

#[test]
fn replacing_attributes_commits_the_ordered_collection_backing_and_usage() {
    // The literals above are the oracle for the units.
    assert_eq!(OLD_UNITS, 14);
    assert_eq!(
        NEW_UNITS,
        attribute_units(Some(XLINK_URI), Some("xlink"), "href", "#\u{1F600}")
            + attribute_units(None, None, "id", "next")
            + attribute_units(None, None, "title", "")
    );
    assert_eq!(
        OLD_UNITS,
        attribute_units(None, None, "id", "main") + attribute_units(None, None, "class", "a b")
    );
    let Host {
        owner,
        root,
        element,
        child,
    } = host(name(Namespace::html(), None, "div"), old_attributes());
    let before = counters(&owner);
    assert_eq!(before[2], 2);
    let replacement = new_attributes();
    let backing = replacement.shared_attributes().clone();
    let (owner, created) = change(owner, replace_attributes(&element, replacement));
    assert!(created.is_none());
    let held = element_of(&owner, &element);
    assert_eq!(
        observed(held.attributes()),
        expected(&[
            (Some(XLINK_URI), Some("xlink"), "href", "#\u{1F600}"),
            (None, None, "id", "next"),
            (None, None, "title", ""),
        ])
    );
    // The validated collection moved in with its original shared backing.
    assert!(Arc::ptr_eq(held.attributes().shared_attributes(), &backing));
    // Name, topology and node count are untouched; attributes and units move.
    assert_eq!(held.name(), &name(Namespace::html(), None, "div"));
    assert_eq!(
        counters(&owner),
        [
            before[0],
            before[1],
            3,
            before[3] - OLD_UNITS + NEW_UNITS,
            before[4]
        ]
    );
    assert_eq!(
        child_handles(&owner, &element),
        std::slice::from_ref(&child)
    );
    assert_eq!(child_handles(&owner, &root), std::slice::from_ref(&element));
    let element_id = owner.result_id(&element).unwrap();
    let (result, _) = publish(owner, root);
    let HtmlNodeKind::Element(published) = result.node(&element_id).unwrap().kind() else {
        panic!("published element");
    };
    assert_eq!(published.attributes().len(), 3);
    assert_eq!(result.usage().attributes(), 3);
}

#[test]
fn replacing_attributes_with_none_and_from_none_keeps_the_count_exact() {
    let Host { owner, element, .. } = host(name(Namespace::html(), None, "div"), old_attributes());
    let before = counters(&owner);
    let (owner, _) = change(
        owner,
        replace_attributes(&element, OrderedAttributes::default()),
    );
    assert!(element_of(&owner, &element).attributes().is_empty());
    assert_eq!(
        counters(&owner),
        [before[0], before[1], 0, before[3] - OLD_UNITS, before[4]]
    );
    let (owner, _) = change(owner, replace_attributes(&element, new_attributes()));
    assert_eq!(element_of(&owner, &element).attributes().len(), 3);
    assert_eq!(
        counters(&owner),
        [
            before[0],
            before[1],
            3,
            before[3] - OLD_UNITS + NEW_UNITS,
            before[4]
        ]
    );
}

#[test]
fn replacing_attributes_with_a_value_equal_collection_moves_only_the_backing() {
    // The value-sharing consumer swaps in an equal collection over a shared
    // allocation; nothing observable but the allocation may change.
    let Host { owner, element, .. } = host(name(Namespace::html(), None, "div"), old_attributes());
    let before = counters(&owner);
    let original = element_of(&owner, &element)
        .attributes()
        .shared_attributes()
        .clone();
    let twin = old_attributes();
    let twin_backing = twin.shared_attributes().clone();
    assert!(!Arc::ptr_eq(&original, &twin_backing));
    let (owner, _) = change(owner, replace_attributes(&element, twin));
    let held = element_of(&owner, &element);
    assert_eq!(counters(&owner), before);
    assert_eq!(
        observed(held.attributes()),
        expected(&[(None, None, "id", "main"), (None, None, "class", "a b")])
    );
    assert!(Arc::ptr_eq(
        held.attributes().shared_attributes(),
        &twin_backing
    ));
}

#[test]
fn replacing_attributes_keeps_link_history_and_template_contents() {
    // Link history and template contents are not attributes.
    let (owner, link) = link_element(storage(), Some(HtmlLinkFlags::fresh()));
    let (owner, _) = change(owner, replace_attributes(&link, new_attributes()));
    assert_eq!(
        element_of(&owner, &link).link_flags(),
        Some(HtmlLinkFlags::fresh())
    );
    let (owner, template) = create(owner, element("template"));
    let contents = owner
        .node(&template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle();
    let (owner, _) = change(owner, replace_attributes(&template, new_attributes()));
    assert_eq!(
        owner
            .node(&template)
            .unwrap()
            .template_contents()
            .unwrap()
            .handle(),
        contents
    );
    assert_eq!(element_of(&owner, &template).attributes().len(), 3);
}

#[test]
fn replacing_attributes_of_a_record_that_is_not_an_element_is_refused() {
    for kind in [
        HtmlNodeKind::Text(text("t")),
        HtmlNodeKind::Comment(text("c")),
        HtmlNodeKind::ProcessingInstruction(pi("target", "data")),
        HtmlNodeKind::Doctype(HtmlDoctype::new("html", None, None)),
    ] {
        let Row {
            owner, siblings, ..
        } = row(kind.clone());
        let before = counters(&owner);
        let target = siblings[1];
        let owner = refuse(
            owner,
            replace_attributes(&target, new_attributes()),
            cause::WRONG_NODE_KIND,
        );
        assert_eq!(counters(&owner), before, "{kind:?}");
        assert_eq!(owner.node(&target).unwrap().kind(), &kind);
    }
    let (owner, root) = fragment(storage());
    let before = counters(&owner);
    let owner = refuse(
        owner,
        replace_attributes(&root, new_attributes()),
        cause::WRONG_NODE_KIND,
    );
    assert_eq!(counters(&owner), before);
}

#[test]
fn replacing_attributes_of_a_foreign_handle_is_refused_and_keeps_the_collection() {
    let Host { owner, element, .. } = host(name(Namespace::html(), None, "div"), old_attributes());
    let before = counters(&owner);
    let (_, foreign) = element_with(
        storage(),
        name(Namespace::html(), None, "div"),
        OrderedAttributes::default(),
    );
    let owner = refuse(
        owner,
        replace_attributes(&foreign, new_attributes()),
        cause::FOREIGN_BUILDER,
    );
    assert_eq!(counters(&owner), before);
    assert_eq!(
        observed(element_of(&owner, &element).attributes()),
        expected(&[(None, None, "id", "main"), (None, None, "class", "a b")])
    );
}

#[test]
fn aborting_an_attribute_replacement_after_any_action_changes_nothing_and_returns_the_command() {
    assert_rollback_at_every_action(
        || {
            let Host { owner, element, .. } =
                host(name(Namespace::html(), None, "div"), old_attributes());
            let before = counters(&owner);
            let backing = element_of(&owner, &element)
                .attributes()
                .shared_attributes()
                .clone();
            (
                owner,
                replace_attributes(&element, new_attributes()),
                (element, before, backing),
            )
        },
        |owner, (element, before, backing), when| {
            assert_eq!(counters(owner), *before, "{when}");
            let held = element_of(owner, element);
            assert!(
                Arc::ptr_eq(held.attributes().shared_attributes(), backing),
                "{when}"
            );
            assert_eq!(
                observed(held.attributes()),
                expected(&[(None, None, "id", "main"), (None, None, "class", "a b")]),
                "{when}"
            );
        },
        |owner, (element, before, _)| {
            assert_eq!(
                observed(element_of(owner, element).attributes()),
                expected(&[
                    (Some(XLINK_URI), Some("xlink"), "href", "#\u{1F600}"),
                    (None, None, "id", "next"),
                    (None, None, "title", ""),
                ])
            );
            assert_eq!(owner.usage().attributes(), 3);
            assert_eq!(
                owner.usage().utf16_units(),
                before[3] - OLD_UNITS + NEW_UNITS
            );
        },
    );
}
