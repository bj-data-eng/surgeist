//! A declarative-shadow and template-contents commit describes exactly the
//! record writes and copied bytes its commit arm performs: the usage write,
//! each field the arm assigns, and the depth cache's writes, which the depth
//! tests own. The expected figures are derived by reading each arm in
//! `HtmlConstructionReadyOperation::commit`, never from the description.
use super::*;
use crate::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};

/// The size of one `Option<HtmlFragmentOwner>` field assignment.
const OWNER: usize = size_of::<Option<HtmlFragmentOwner>>();
/// The size of one `Option<TemplateContents>` field assignment.
const CONTENTS: usize = size_of::<Option<TemplateContents>>();
/// The size of one `Option<ShadowAssociation>` field assignment.
const ASSOCIATION: usize = size_of::<Option<ShadowAssociation>>();

fn begin() -> HtmlConstructionStorage {
    let mut initialization = Some(HtmlConstructionStorage::begin());
    initialization
        .as_mut()
        .unwrap()
        .next_action()
        .unwrap()
        .unwrap()
        .perform()
        .accept();
    HtmlConstructionInitialization::try_ready(&mut initialization)
        .unwrap()
        .commit()
}

/// Performs `command` and returns the storage and any created node.
fn change(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, Option<HtmlBuilderNode>) {
    let mut operation = Some(
        storage
            .prepare(command)
            .unwrap_or_else(|_| panic!("prepare")),
    );
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        action.perform().unwrap().accept();
    }
    match HtmlConstructionOperation::try_ready(&mut operation)
        .unwrap()
        .commit()
    {
        HtmlConstructionCommit::Changed { storage, created } => (storage, created),
        HtmlConstructionCommit::Published { .. } => panic!("mutation published"),
    }
}

fn create(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    let (storage, created) = change(storage, command);
    (storage, created.unwrap())
}

fn element(
    storage: HtmlConstructionStorage,
    local: &str,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(
        storage,
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Element(HtmlElement::new(
                QualifiedName::new(Namespace::html(), None, local),
                OrderedAttributes::default(),
            )),
        },
    )
}

fn shadow(storage: HtmlConstructionStorage) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(
        storage,
        HtmlConstructionCommand::CreateShadowContents {
            properties: HtmlShadowRootProperties::new(
                HtmlShadowRootMode::Closed,
                HtmlSlotAssignment::Manual,
                true,
                true,
                true,
            ),
        },
    )
}

fn contents_of(storage: &HtmlConstructionStorage, template: &HtmlBuilderNode) -> HtmlBuilderNode {
    storage
        .node(template)
        .unwrap()
        .template_contents()
        .unwrap()
        .handle()
}

/// What a commit action described, and the depth writes inside its writes.
struct Described {
    record_writes: usize,
    copied_bytes: usize,
    depth_writes: usize,
}

fn described(storage: HtmlConstructionStorage, command: HtmlConstructionCommand) -> Described {
    let mut operation = Some(
        storage
            .prepare(command)
            .unwrap_or_else(|_| panic!("prepare")),
    );
    let mut commit = None;
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        if let HtmlConstructionActionDescription::Commit {
            record_writes,
            copied_bytes,
            ..
        } = action.description()
        {
            commit = Some((record_writes, copied_bytes));
        }
        action.perform().unwrap().accept();
    }
    let (_, depth_writes) = operation
        .as_ref()
        .unwrap()
        .plan
        .depth
        .commit_work()
        .unwrap();
    let (record_writes, copied_bytes) = commit.expect("every operation ends in a Commit action");
    Described {
        record_writes,
        copied_bytes,
        depth_writes,
    }
}

/// The work categories a consumer charges a commit description against.
#[derive(Debug, Eq, PartialEq)]
enum Resource {
    CopyByte,
    RecordWrite,
}

/// A consumer's admission of a commit description, in its charging order
/// (copied bytes, then record writes), against fixed limits.
fn admit(limits: (usize, usize), described: &Described) -> Result<(), Resource> {
    if described.copied_bytes > limits.1 {
        return Err(Resource::CopyByte);
    }
    if described.record_writes > limits.0 {
        return Err(Resource::RecordWrite);
    }
    Ok(())
}

/// The figures of a commit that assigns `writes` fields of `bytes` in all and
/// also writes the usage.
fn with_usage(writes: usize, bytes: usize) -> (usize, usize) {
    (1 + writes, size_of::<HtmlConstructionUsage>() + bytes)
}

/// Checks the description of `command` against the figures its commit arm
/// derives (`writes` record writes of `bytes` in all, besides the depth cache's
/// writes), and that exactly those figures are the limit that admits it.
fn assert_exact_fit(
    label: &str,
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    (writes, bytes): (usize, usize),
) {
    let described = described(storage, command);
    let expected = (
        writes + described.depth_writes,
        bytes + described.depth_writes * size_of::<usize>(),
    );
    assert_eq!(
        (described.record_writes, described.copied_bytes),
        expected,
        "{label}: described (writes, bytes) against the commit arm's"
    );
    assert_eq!(admit(expected, &described), Ok(()), "{label}: exact limit");
    if expected.0 > 0 {
        assert_eq!(
            admit((expected.0 - 1, expected.1), &described),
            Err(Resource::RecordWrite),
            "{label}: one write fewer"
        );
    }
    if expected.1 > 0 {
        assert_eq!(
            admit((expected.0, expected.1 - 1), &described),
            Err(Resource::CopyByte),
            "{label}: one byte fewer"
        );
    }
}

#[test]
fn attaching_a_shadow_describes_the_two_fields_it_assigns() {
    // The arm assigns the contents' owner and the host's shadow association.
    let (storage, host) = element(begin(), "host");
    let (storage, contents) = shadow(storage);
    assert_exact_fit(
        "AttachShadow",
        storage,
        HtmlConstructionCommand::AttachShadow { host, contents },
        with_usage(2, OWNER + ASSOCIATION),
    );
}

#[test]
fn attaching_a_template_shadow_describes_the_four_fields_it_assigns() {
    // The arm assigns the old contents' owner, the template's contents, the
    // new contents' owner and the host's shadow association.
    let (storage, template) = element(begin(), "template");
    let (storage, host) = element(storage, "host");
    let (storage, contents) = shadow(storage);
    assert_exact_fit(
        "AttachTemplateShadow",
        storage,
        HtmlConstructionCommand::AttachTemplateShadow {
            template,
            host,
            contents,
        },
        with_usage(4, 2 * OWNER + CONTENTS + ASSOCIATION),
    );
}

#[test]
fn setting_template_shadow_contents_describes_the_two_fields_it_assigns() {
    // The arm assigns the old contents' owner and the template's contents.
    let (storage, template) = element(begin(), "template");
    let (storage, contents) = shadow(storage);
    assert_exact_fit(
        "SetTemplateShadowContents",
        storage,
        HtmlConstructionCommand::SetTemplateShadowContents { template, contents },
        with_usage(2, OWNER + CONTENTS),
    );
}

#[test]
fn reassigning_owned_contents_describes_the_three_fields_it_assigns() {
    // The arm assigns the old contents' owner, the template's contents and the
    // new contents' owner.
    let (storage, template) = element(begin(), "template");
    let (storage, contents) = create(storage, HtmlConstructionCommand::CreateFragment);
    assert_exact_fit(
        "ReassignTemplateContents over owned contents",
        storage,
        HtmlConstructionCommand::ReassignTemplateContents { template, contents },
        with_usage(3, 2 * OWNER + CONTENTS),
    );
}

#[test]
fn reassigning_over_a_shadow_reference_describes_the_two_fields_it_assigns() {
    // A shadow reference owns no old contents, so the arm assigns only the
    // template's contents and the new contents' owner.
    let (storage, template) = element(begin(), "template");
    let ordinary = contents_of(&storage, &template);
    let (storage, shadow) = shadow(storage);
    let (storage, _) = change(
        storage,
        HtmlConstructionCommand::SetTemplateShadowContents {
            template: template.clone(),
            contents: shadow,
        },
    );
    assert_exact_fit(
        "ReassignTemplateContents over a shadow reference",
        storage,
        HtmlConstructionCommand::ReassignTemplateContents {
            template,
            contents: ordinary,
        },
        with_usage(2, OWNER + CONTENTS),
    );
}

fn name(namespace: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(namespace, None, local)
}

#[test]
fn replacing_text_like_content_describes_the_kind_it_assigns() {
    // The arm assigns the record's whole kind for every payload but a name.
    let replacements: [(HtmlNodeKind, HtmlConstructionContent); 4] = [
        (
            HtmlNodeKind::Text("a".into()),
            HtmlConstructionContent::Text("b".into()),
        ),
        (
            HtmlNodeKind::Comment("a".into()),
            HtmlConstructionContent::Comment("b".into()),
        ),
        (
            HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction::new("t", "a")),
            HtmlConstructionContent::ProcessingInstruction(HtmlProcessingInstruction::new(
                "t", "b",
            )),
        ),
        (
            HtmlNodeKind::Doctype(HtmlDoctype::new("a", None, None)),
            HtmlConstructionContent::Doctype(HtmlDoctype::new("b", None, None)),
        ),
    ];
    for (kind, content) in replacements {
        let label = format!("ReplaceContent over {kind:?}");
        let (storage, node) = create(begin(), HtmlConstructionCommand::Create { kind });
        assert_exact_fit(
            &label,
            storage,
            HtmlConstructionCommand::ReplaceContent { node, content },
            with_usage(1, size_of::<HtmlNodeKind>()),
        );
    }
}

#[test]
fn renaming_an_element_describes_the_name_and_the_link_history_it_assigns() {
    // The arm assigns the name, and clears the link history unless the new
    // name is an HTML link.
    let (storage, node) = element(begin(), "div");
    assert_exact_fit(
        "ReplaceContent renaming to a non-link",
        storage,
        HtmlConstructionCommand::ReplaceContent {
            node,
            content: HtmlConstructionContent::ElementName(name(Namespace::html(), "span")),
        },
        with_usage(
            2,
            size_of::<QualifiedName>() + size_of::<Option<crate::HtmlLinkFlags>>(),
        ),
    );
    let (storage, node) = element(begin(), "div");
    assert_exact_fit(
        "ReplaceContent renaming to a link",
        storage,
        HtmlConstructionCommand::ReplaceContent {
            node,
            content: HtmlConstructionContent::ElementName(name(Namespace::html(), "link")),
        },
        with_usage(1, size_of::<QualifiedName>()),
    );
}

#[test]
fn replacing_attributes_describes_the_collection_it_assigns() {
    let (storage, element) = element(begin(), "div");
    assert_exact_fit(
        "ReplaceAttributes",
        storage,
        HtmlConstructionCommand::ReplaceAttributes {
            element,
            attributes: OrderedAttributes::default().into(),
        },
        with_usage(1, size_of::<OrderedAttributes>()),
    );
}

#[test]
fn setting_the_document_mode_describes_the_kind_it_assigns() {
    let (storage, document) = create(
        begin(),
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::NoQuirks,
        },
    );
    assert_exact_fit(
        "SetDocumentMode",
        storage,
        HtmlConstructionCommand::SetDocumentMode {
            document,
            mode: HtmlDocumentMode::Quirks,
        },
        with_usage(1, size_of::<HtmlNodeKind>()),
    );
}

#[test]
fn setting_link_flags_describes_the_history_it_assigns() {
    let (storage, element) = create(
        begin(),
        HtmlConstructionCommand::CreateElement {
            name: name(Namespace::html(), "link"),
            attributes: OrderedAttributes::default(),
            link_flags: None,
        },
    );
    assert_exact_fit(
        "SetLinkFlags",
        storage,
        HtmlConstructionCommand::SetLinkFlags {
            element,
            flags: crate::HtmlLinkFlags::fresh(),
        },
        with_usage(1, size_of::<Option<crate::HtmlLinkFlags>>()),
    );
}

#[test]
fn creating_records_describes_each_record_and_its_depth_count() {
    // Each record is one write, and so is the count of its depth; the usage is
    // the one other write.
    let command = HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Element(HtmlElement::new(
            name(Namespace::html(), "p"),
            OrderedAttributes::default(),
        )),
    };
    assert_exact_fit(
        "Create of an element",
        begin(),
        command,
        with_usage(2, size_of::<Record>()),
    );
    // A template creates its contents record too.
    let command = HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Element(HtmlElement::new(
            name(Namespace::html(), "template"),
            OrderedAttributes::default(),
        )),
    };
    assert_exact_fit(
        "Create of a template",
        begin(),
        command,
        with_usage(4, 2 * size_of::<Record>()),
    );
}

#[test]
fn commands_that_change_no_record_describe_no_write() {
    // Neither arm assigns anything: the reservation grows the record bank and
    // the transfer check only reads, so even the usage is not written.
    assert_exact_fit(
        "ReserveRecords",
        begin(),
        HtmlConstructionCommand::ReserveRecords { additional: 2 },
        (0, 0),
    );
    let (storage, from) = create(begin(), HtmlConstructionCommand::CreateFragment);
    let (storage, to) = create(storage, HtmlConstructionCommand::CreateFragment);
    assert_exact_fit(
        "CheckChildTransfer",
        storage,
        HtmlConstructionCommand::CheckChildTransfer { from, to },
        (0, 0),
    );
}
