//! A commit describes exactly the record writes and copied bytes its arm
//! performs: the usage write, each record field the arm assigns, and the depth
//! cache's writes, which the depth tests own.
//!
//! Two independent checks back each command. The test names the fields the
//! command must assign, and a mechanical oracle snapshots every record before
//! the commit and compares it field by field afterwards, through exhaustive
//! destructuring of the record, so a new record field cannot go unexamined. The
//! described figures must then equal the usage write plus the size of each
//! field the oracle saw assigned, plus the depth writes, never figures read off
//! the arm or the description.
//!
//! An assignment of an identical value is invisible to a comparison, so every
//! fixture changes each field it charges for; attributes are compared by
//! backing allocation, which sees a value-equal swap. The usage cannot change
//! under a document-mode or link-flag edit, so its write there rests on the
//! arm alone.
use super::*;
use crate::{
    internal::construction::HtmlShadowRootMode, internal::construction::HtmlShadowRootProperties,
    internal::construction::HtmlSlotAssignment,
};

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
    slots::initialization(&mut initialization).unwrap().commit()
}

/// Performs `command` and returns the storage and any created node.
fn change(
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
) -> (HtmlConstructionStorage, Option<HtmlBuilderNode>) {
    let mut operation = Some(storage.prepare(command));
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        action.perform().unwrap().accept();
    }
    match slots::operation(&mut operation).unwrap().commit().unwrap() {
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

/// A record's fields as they were, copied by exhaustive destructuring.
struct RecordSnapshot {
    kind: HtmlNodeKind,
    document_focused_area_designation: Option<HtmlFocusedAreaDesignation>,
    parent: Option<usize>,
    children: VecDeque<usize>,
    children_generation: u64,
    /// The depth cache: this record's depth, the histogram count at its index
    /// and the walk level count at its index.
    depth: usize,
    population: usize,
    level_count: usize,
    template_contents: Option<TemplateContents>,
    shadow: Option<ShadowAssociation>,
    shadow_properties: Option<HtmlShadowRootProperties>,
    owner: Option<HtmlFragmentOwner>,
    publish_found: u8,
}

struct Snapshot {
    records: Vec<RecordSnapshot>,
    usage: HtmlConstructionUsage,
    attribute_revision: u64,
}

fn snapshot(storage: &HtmlConstructionStorage) -> Snapshot {
    let records = storage
        .arena
        .records
        .iter()
        .map(|record| {
            // No `..`: a field added to `Record` stops this compiling. The walk
            // cursor is scratch that a walk resets before it reads it.
            let Record {
                kind,
                document_focused_area_designation,
                parent,
                children,
                children_generation,
                depth,
                population,
                walk_cursor: _,
                level_count,
                template_contents,
                shadow,
                shadow_properties,
                owner,
                publish_found,
            } = record;
            RecordSnapshot {
                kind: kind.clone(),
                document_focused_area_designation: *document_focused_area_designation,
                parent: *parent,
                children: children.clone(),
                children_generation: *children_generation,
                depth: *depth,
                population: *population,
                level_count: *level_count,
                template_contents: *template_contents,
                shadow: shadow.clone(),
                shadow_properties: *shadow_properties,
                owner: owner.clone(),
                publish_found: *publish_found,
            }
        })
        .collect();
    Snapshot {
        records,
        usage: storage.arena.usage,
        attribute_revision: storage.arena.attribute_revision,
    }
}

/// The fields of one record a commit assigned, other than the depth cache's
/// scratch and counts, whose writes the depth plan describes.
fn assigned(before: &RecordSnapshot, after: &RecordSnapshot) -> Vec<&'static str> {
    // No `..`: a field added to the snapshot stops this compiling. The depth
    // cache's fields are set aside here and counted by `depth_cache_changes`.
    let RecordSnapshot {
        kind: kind_before,
        document_focused_area_designation: designation_before,
        parent: parent_before,
        children: children_before,
        children_generation: generation_before,
        depth: _,
        population: _,
        level_count: _,
        template_contents: contents_before,
        shadow: shadow_before,
        shadow_properties: properties_before,
        owner: owner_before,
        publish_found: found_before,
    } = before;
    let RecordSnapshot {
        kind: kind_after,
        document_focused_area_designation: designation_after,
        parent: parent_after,
        children: children_after,
        children_generation: generation_after,
        depth: _,
        population: _,
        level_count: _,
        template_contents: contents_after,
        shadow: shadow_after,
        shadow_properties: properties_after,
        owner: owner_after,
        publish_found: found_after,
    } = after;
    let mut fields = Vec::new();
    match (kind_before, kind_after) {
        (HtmlNodeKind::Element(before), HtmlNodeKind::Element(after)) => {
            let HtmlElement {
                link_flags: flags_before,
                name: name_before,
                attributes: attributes_before,
            } = before;
            let HtmlElement {
                link_flags: flags_after,
                name: name_after,
                attributes: attributes_after,
            } = after;
            if flags_before != flags_after {
                fields.push("kind.element.link_flags");
            }
            if name_before != name_after {
                fields.push("kind.element.name");
            }
            // An assignment of equal attributes still moves the allocation.
            if !Arc::ptr_eq(
                attributes_before.shared_attributes(),
                attributes_after.shared_attributes(),
            ) {
                fields.push("kind.element.attributes");
            }
        }
        (before, after) => {
            if before != after {
                fields.push("kind");
            }
        }
    }
    let changes = [
        (
            designation_before != designation_after,
            "document_focused_area_designation",
        ),
        (parent_before != parent_after, "parent"),
        (children_before != children_after, "children"),
        (generation_before != generation_after, "children_generation"),
        (contents_before != contents_after, "template_contents"),
        (shadow_before != shadow_after, "shadow"),
        (properties_before != properties_after, "shadow_properties"),
        (owner_before != owner_after, "owner"),
        (found_before != found_after, "publish_found"),
    ];
    fields.extend(
        changes
            .iter()
            .filter(|(changed, _)| *changed)
            .map(|(_, name)| *name),
    );
    fields
}

/// The depth cache writes a commit visibly made: each existing record whose
/// depth or walk level count changed, and each histogram count that moved. A
/// write of an unchanged value is not seen, so this is a lower bound.
fn depth_cache_changes(before: &Snapshot, after: &Snapshot) -> usize {
    let existing = before.records.iter().zip(&after.records);
    let records: usize = existing
        .map(|(before, after)| {
            usize::from(before.depth != after.depth)
                + usize::from(before.level_count != after.level_count)
        })
        .sum();
    let counts = after
        .records
        .iter()
        .enumerate()
        .filter(|(index, record)| {
            record.population != before.records.get(*index).map_or(0, |old| old.population)
        })
        .count();
    records + counts
}

/// The bytes one assignment of the named field copies.
fn field_size(field: &str) -> usize {
    match field {
        "kind" => size_of::<HtmlNodeKind>(),
        "kind.element.link_flags" => {
            size_of::<Option<crate::internal::elements::rel::HtmlLinkFlags>>()
        }
        "kind.element.name" => size_of::<QualifiedName>(),
        "kind.element.attributes" => size_of::<OrderedAttributes>(),
        "document_focused_area_designation" => size_of::<Option<HtmlFocusedAreaDesignation>>(),
        "parent" => size_of::<Option<usize>>(),
        "children" => size_of::<VecDeque<usize>>(),
        "children_generation" => size_of::<u64>(),
        "template_contents" => size_of::<Option<TemplateContents>>(),
        "shadow" => size_of::<Option<ShadowAssociation>>(),
        "shadow_properties" => size_of::<Option<HtmlShadowRootProperties>>(),
        "owner" => size_of::<Option<HtmlFragmentOwner>>(),
        "publish_found" => size_of::<u8>(),
        other => panic!("no size for the field {other}"),
    }
}

/// What one command described at its Commit action, the depth writes inside
/// those, and the state it left behind.
struct Observation {
    record_writes: usize,
    copied_bytes: usize,
    depth_writes: usize,
    before: Snapshot,
    after: Snapshot,
}

fn observe(storage: HtmlConstructionStorage, command: HtmlConstructionCommand) -> Observation {
    let before = snapshot(&storage);
    let mut operation = Some(storage.prepare(command));
    let mut commit = None;
    while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
        if let HtmlConstructionActionDescription::Commit { work, .. } = action.description() {
            commit = Some((work.record_writes, work.copied_bytes));
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
    let HtmlConstructionCommit::Changed { storage, .. } =
        slots::operation(&mut operation).unwrap().commit().unwrap()
    else {
        panic!("mutation published");
    };
    Observation {
        record_writes,
        copied_bytes,
        depth_writes,
        before,
        after: snapshot(&storage),
    }
}

/// What a commit does to the usage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UsageEffect {
    /// The commit assigns a usage that differs from the old one.
    Changes,
    /// The commit assigns the usage, but the edit cannot move it, so the
    /// assignment is not observable.
    Identical,
    /// The commit does not assign the usage.
    Untouched,
}

/// Checks that `command` assigns exactly the `fields` (a field once per record
/// it is assigned in) and the usage as given, and that its description charges
/// the usage write, the size of each field seen assigned and the depth writes.
fn assert_assigned(
    label: &str,
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    fields: &[&str],
    usage: UsageEffect,
    moves_depth: bool,
) {
    let seen = observe(storage, command);
    assert_eq!(
        seen.before.records.len(),
        seen.after.records.len(),
        "{label}: creates no record"
    );
    let mut observed: Vec<&str> = seen
        .before
        .records
        .iter()
        .zip(&seen.after.records)
        .flat_map(|(before, after)| assigned(before, after))
        .collect();
    observed.sort_unstable();
    let mut expected = fields.to_vec();
    expected.sort_unstable();
    assert_eq!(observed, expected, "{label}: fields assigned");
    match usage {
        UsageEffect::Changes => assert_ne!(
            seen.before.usage, seen.after.usage,
            "{label}: usage changes"
        ),
        UsageEffect::Identical | UsageEffect::Untouched => {
            assert_eq!(seen.before.usage, seen.after.usage, "{label}: usage stays")
        }
    }
    // The depth plan describes the cache writes; the commit may not make more
    // than it describes.
    let cache = depth_cache_changes(&seen.before, &seen.after);
    assert_eq!(
        cache > 0,
        moves_depth,
        "{label}: the fixture moves the depth"
    );
    assert!(
        seen.depth_writes >= cache,
        "{label}: {cache} depth cache changes seen against {} writes described",
        seen.depth_writes
    );
    // The storage's attribute revision advances with every assignment of an element's attribute
    // list, and with nothing else: a table of attribute names trusts it to see every change.
    let revision_writes =
        usize::from(seen.after.attribute_revision != seen.before.attribute_revision);
    assert_eq!(
        revision_writes == 1,
        observed.contains(&"kind.element.attributes"),
        "{label}: the attribute revision advances exactly when an attribute list is assigned"
    );
    let usage_writes = usize::from(usage != UsageEffect::Untouched);
    let usage_bytes = usage_writes * size_of::<HtmlConstructionUsage>();
    let field_bytes: usize = observed.iter().map(|field| field_size(field)).sum();
    assert_eq!(
        (seen.record_writes, seen.copied_bytes),
        (
            usage_writes + observed.len() + revision_writes + seen.depth_writes,
            usage_bytes
                + field_bytes
                + revision_writes * size_of::<u64>()
                + seen.depth_writes * size_of::<usize>(),
        ),
        "{label}: described (writes, bytes) against the assignments seen"
    );
}

/// Checks that `command` appends `records` records and assigns nothing in any
/// older one, and that its description charges the usage write, one write and
/// one record's bytes for each appended record, and one word for each
/// depth-count increment seen.
fn assert_created(
    label: &str,
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    records: usize,
) {
    let seen = observe(storage, command);
    let old = seen.before.records.len();
    assert_eq!(seen.after.records.len(), old + records, "{label}: records");
    for (index, (before, after)) in seen
        .before
        .records
        .iter()
        .zip(&seen.after.records)
        .enumerate()
    {
        assert_eq!(
            assigned(before, after),
            Vec::<&str>::new(),
            "{label}: record {index}"
        );
        // Only histogram counts change, and they are the increments counted
        // below; no existing record moves.
        assert_eq!(
            (before.depth, before.level_count),
            (after.depth, after.level_count),
            "{label}: depth cache of record {index}"
        );
    }
    assert_ne!(
        seen.before.usage, seen.after.usage,
        "{label}: usage changes"
    );
    let counted = |snapshot: &Snapshot| -> usize {
        snapshot
            .records
            .iter()
            .map(|record| record.population)
            .sum()
    };
    let increments = counted(&seen.after) - counted(&seen.before);
    assert_eq!(
        (seen.record_writes, seen.copied_bytes),
        (
            1 + records + increments + seen.depth_writes,
            size_of::<HtmlConstructionUsage>()
                + records * size_of::<Record>()
                + (increments + seen.depth_writes) * size_of::<usize>(),
        ),
        "{label}: described (writes, bytes) against the records and counts seen"
    );
}

/// Appends a fresh element child under `parent`.
fn with_child(
    storage: HtmlConstructionStorage,
    parent: &HtmlBuilderNode,
) -> HtmlConstructionStorage {
    let (storage, child) = element(storage, "child");
    change(
        storage,
        HtmlConstructionCommand::Insert {
            node: child,
            parent: parent.clone(),
            before: None,
        },
    )
    .0
}

fn name(namespace: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(namespace, None, local)
}

fn link(
    storage: HtmlConstructionStorage,
    flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
) -> (HtmlConstructionStorage, HtmlBuilderNode) {
    create(
        storage,
        HtmlConstructionCommand::CreateElement {
            name: name(Namespace::html(), "link"),
            attributes: OrderedAttributes::default(),
            link_flags: flags,
        },
    )
}

#[test]
fn attaching_a_shadow_assigns_the_contents_owner_and_the_host_association() {
    let (storage, host) = element(begin(), "host");
    let (storage, contents) = shadow(storage);
    assert_assigned(
        "AttachShadow",
        storage,
        HtmlConstructionCommand::AttachShadow { host, contents },
        &["owner", "shadow"],
        UsageEffect::Changes,
        true,
    );
}

#[test]
fn attaching_a_template_shadow_assigns_both_owners_the_contents_and_the_association() {
    let (storage, template) = element(begin(), "template");
    let (storage, host) = element(storage, "host");
    let (storage, contents) = shadow(storage);
    // A child below the shadow root makes the deepest record move.
    let storage = with_child(storage, &contents);
    assert_assigned(
        "AttachTemplateShadow",
        storage,
        HtmlConstructionCommand::AttachTemplateShadow {
            template,
            host,
            contents,
        },
        &["owner", "owner", "template_contents", "shadow"],
        UsageEffect::Changes,
        true,
    );
}

#[test]
fn setting_template_shadow_contents_assigns_the_old_owner_and_the_contents() {
    let (storage, template) = element(begin(), "template");
    let (storage, contents) = shadow(storage);
    assert_assigned(
        "SetTemplateShadowContents",
        storage,
        HtmlConstructionCommand::SetTemplateShadowContents { template, contents },
        &["owner", "template_contents"],
        UsageEffect::Changes,
        true,
    );
}

#[test]
fn reassigning_owned_contents_assigns_both_owners_and_the_contents() {
    let (storage, template) = element(begin(), "template");
    let (storage, contents) = create(storage, HtmlConstructionCommand::CreateFragment);
    let storage = with_child(storage, &contents);
    assert_assigned(
        "ReassignTemplateContents over owned contents",
        storage,
        HtmlConstructionCommand::ReassignTemplateContents { template, contents },
        &["owner", "owner", "template_contents"],
        UsageEffect::Changes,
        true,
    );
}

#[test]
fn reassigning_over_a_shadow_reference_assigns_the_new_owner_and_the_contents() {
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
    assert_assigned(
        "ReassignTemplateContents over a shadow reference",
        storage,
        HtmlConstructionCommand::ReassignTemplateContents {
            template,
            contents: ordinary,
        },
        &["owner", "template_contents"],
        UsageEffect::Changes,
        true,
    );
}

#[test]
fn replacing_text_like_content_assigns_the_kind() {
    // Each replacement differs in length, so the units in the usage move too.
    let replacements: [(HtmlNodeKind, HtmlConstructionContent); 4] = [
        (
            HtmlNodeKind::Text("a".into()),
            HtmlConstructionContent::Text("bcd".into()),
        ),
        (
            HtmlNodeKind::Comment("a".into()),
            HtmlConstructionContent::Comment("bcd".into()),
        ),
        (
            HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction::new("t", "a")),
            HtmlConstructionContent::ProcessingInstruction(HtmlProcessingInstruction::new(
                "t", "bc",
            )),
        ),
        (
            HtmlNodeKind::Doctype(HtmlDoctype::new("a", None, None)),
            HtmlConstructionContent::Doctype(HtmlDoctype::new("bcd", None, None)),
        ),
    ];
    for (kind, content) in replacements {
        let label = format!("ReplaceContent over {kind:?}");
        let (storage, node) = create(begin(), HtmlConstructionCommand::Create { kind });
        assert_assigned(
            &label,
            storage,
            HtmlConstructionCommand::ReplaceContent { node, content },
            &["kind"],
            UsageEffect::Changes,
            false,
        );
    }
}

#[test]
fn renaming_an_element_assigns_the_name_and_clears_the_link_history_unless_a_link() {
    // A link with history renamed to a non-link loses the history.
    let (storage, node) = link(
        begin(),
        Some(crate::internal::elements::rel::HtmlLinkFlags::fresh()),
    );
    assert_assigned(
        "ReplaceContent renaming a link to a non-link",
        storage,
        HtmlConstructionCommand::ReplaceContent {
            node,
            content: HtmlConstructionContent::ElementName(name(Namespace::html(), "section")),
        },
        &["kind.element.name", "kind.element.link_flags"],
        UsageEffect::Changes,
        false,
    );
    // A rename to a link leaves the history alone: the element has history to
    // lose, and a prefix changes the name but keeps the HTML link expanded name.
    let (storage, node) = link(
        begin(),
        Some(crate::internal::elements::rel::HtmlLinkFlags::fresh()),
    );
    assert_assigned(
        "ReplaceContent renaming a link to a link",
        storage,
        HtmlConstructionCommand::ReplaceContent {
            node,
            content: HtmlConstructionContent::ElementName(QualifiedName::new(
                Namespace::html(),
                Some("p".into()),
                "link",
            )),
        },
        &["kind.element.name"],
        UsageEffect::Changes,
        false,
    );
}

fn attributes_of(pairs: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(pairs.iter().map(|(local, value)| {
        crate::internal::values::Attribute::new(
            QualifiedName::new(Namespace::none(), None, *local),
            *value,
        )
    }))
    .unwrap()
}

#[test]
fn replacing_attributes_assigns_the_collection() {
    let (storage, element) = create(
        begin(),
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Element(HtmlElement::new(
                name(Namespace::html(), "div"),
                OrderedAttributes::default(),
            )),
        },
    );
    assert_assigned(
        "ReplaceAttributes",
        storage,
        HtmlConstructionCommand::ReplaceAttributes {
            element,
            attributes: attributes_of(&[("id", "x")]).into(),
        },
        &["kind.element.attributes"],
        UsageEffect::Changes,
        false,
    );
}

#[test]
fn replacing_attributes_with_equal_values_still_assigns_the_collection() {
    // The value-sharing swap changes only the backing allocation, and the usage
    // it leaves is the one it found.
    let (storage, element) = create(
        begin(),
        HtmlConstructionCommand::Create {
            kind: HtmlNodeKind::Element(HtmlElement::new(
                name(Namespace::html(), "div"),
                attributes_of(&[("id", "x")]),
            )),
        },
    );
    assert_assigned(
        "ReplaceAttributes with equal values",
        storage,
        HtmlConstructionCommand::ReplaceAttributes {
            element,
            attributes: attributes_of(&[("id", "x")]).into(),
        },
        &["kind.element.attributes"],
        UsageEffect::Identical,
        false,
    );
}

#[test]
fn setting_the_document_mode_assigns_the_kind() {
    let (storage, document) = create(
        begin(),
        HtmlConstructionCommand::CreateDocument {
            mode: HtmlDocumentMode::NoQuirks,
        },
    );
    assert_assigned(
        "SetDocumentMode",
        storage,
        HtmlConstructionCommand::SetDocumentMode {
            document,
            mode: HtmlDocumentMode::Quirks,
        },
        &["kind"],
        UsageEffect::Identical,
        false,
    );
}

#[test]
fn setting_link_flags_assigns_the_link_history() {
    let (storage, element) = link(begin(), None);
    assert_assigned(
        "SetLinkFlags",
        storage,
        HtmlConstructionCommand::SetLinkFlags {
            element,
            flags: crate::internal::elements::rel::HtmlLinkFlags::fresh(),
        },
        &["kind.element.link_flags"],
        UsageEffect::Identical,
        false,
    );
}

#[test]
fn creating_records_appends_them_and_counts_their_depths() {
    let command = HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Element(HtmlElement::new(
            name(Namespace::html(), "p"),
            OrderedAttributes::default(),
        )),
    };
    assert_created("Create of an element", begin(), command, 1);
    // A template creates its contents record too.
    let command = HtmlConstructionCommand::Create {
        kind: HtmlNodeKind::Element(HtmlElement::new(
            name(Namespace::html(), "template"),
            OrderedAttributes::default(),
        )),
    };
    assert_created("Create of a template", begin(), command, 2);
}

#[test]
fn commands_that_change_no_record_assign_nothing_and_describe_no_write() {
    // The reservation grows the record bank and the transfer check only reads,
    // so not even the usage is assigned.
    assert_assigned(
        "ReserveRecords",
        begin(),
        HtmlConstructionCommand::ReserveRecords { additional: 2 },
        &[],
        UsageEffect::Untouched,
        false,
    );
    let (storage, from) = create(begin(), HtmlConstructionCommand::CreateFragment);
    let (storage, to) = create(storage, HtmlConstructionCommand::CreateFragment);
    assert_assigned(
        "CheckChildTransfer",
        storage,
        HtmlConstructionCommand::CheckChildTransfer { from, to },
        &[],
        UsageEffect::Untouched,
        false,
    );
}
