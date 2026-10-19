//! Native weak-slot qualification beyond the combinations currently emitted by
//! the public parser. Fixtures use the same admitted Template relation barrier
//! as parsed installation; retirement and publication use the real tree owner.
use super::*;
use crate::{DomChange, DomSnapshot, Namespace, template::TemplatePatch};
use std::sync::atomic::Ordering;

type Patch = [Option<NodeId>; 3];

struct Fixture {
    dom: Dom,
    document: DocumentId,
    other_document: DocumentId,
    root: NodeId,
    subtree: NodeId,
    target: NodeId,
    markers: [NodeId; 4],
    unrelated_target: NodeId,
    unrelated_markers: [NodeId; 2],
    unrelated_text: NodeId,
    templates: [NodeId; 4],
    patches: [Patch; 4],
}

fn limits() -> DomLimits {
    DomLimits::new(
        512,
        65536,
        512,
        256,
        128,
        8,
        131072,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}

fn patch(snapshot: &DomSnapshot, template: NodeId) -> Patch {
    let value = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .insertion_state();
    [value.target(), value.start(), value.end()]
}

fn assert_patches(snapshot: &DomSnapshot, templates: [NodeId; 4], expected: [Patch; 4]) {
    for (template, expected) in templates.into_iter().zip(expected) {
        assert_eq!(patch(snapshot, template), expected);
    }
    // Decode actual count leaves independently of the production reference
    // walker. Every applicable field contributes its own exact-generation edge.
    let references = snapshot.state.inbound.inspect_references();
    assert!(references.iter().all(|(target, owner, count)| {
        matches!(target, Key::Existing(_)) && matches!(owner, Key::Existing(_)) && *count != 0
    }));
    for (template, fields) in templates.into_iter().zip(expected) {
        for target in fields.into_iter().flatten() {
            let expected_count = fields
                .into_iter()
                .filter(|field| *field == Some(target))
                .count();
            let actual = references
                .iter()
                .filter(|(key, owner, _)| {
                    *key == Key::Existing(target) && *owner == Key::Existing(template)
                })
                .map(|(_, _, count)| *count)
                .sum::<usize>();
            assert_eq!(actual, expected_count);
        }
    }
    let graph = snapshot.state.inbound.inspect_graph().unwrap();
    assert_eq!(
        snapshot.state.inbound.retained_usage(),
        custody::Usage::new(
            graph.index_charge().bytes + graph.record_totals().bytes,
            graph.record_totals().units,
        )
    );
}

fn assert_retired(snapshot: &DomSnapshot, before: &DomSnapshot, retired: &[NodeId]) {
    let references = snapshot.state.inbound.inspect_references();
    for id in retired {
        assert!(snapshot.node(*id).is_err());
        assert!(before.node(*id).is_ok());
        assert!(references.iter().all(|(target, owner, _)| {
            *target != Key::Existing(*id) && *owner != Key::Existing(*id)
        }));
    }
}

fn assert_publication(commit: &DomCommit, before: &DomSnapshot, retired: &[NodeId]) {
    assert_eq!(commit.changes().from(), before.stamp());
    assert_ne!(commit.changes().to(), before.stamp());
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::TemplateRelations)
    );
    for id in retired {
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::Destroyed(*id))
        );
    }
}

fn fixture() -> Fixture {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let other_document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let subtree = batch.create_element_ns(&document, Namespace::html(), "section");
    batch.insert_before(&subtree, &root, None);
    let target = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&target, &subtree, None);
    let markers = [
        "patch-start",
        "patch-end",
        "alternate-start",
        "alternate-end",
    ]
    .map(|name| {
        let node = batch.create_processing_instruction(&document, name, "");
        batch.insert_before(&node, &target, None);
        node
    });
    let unrelated_target = batch.create_element_ns(&document, Namespace::html(), "aside");
    batch.insert_before(&unrelated_target, &root, None);
    let unrelated_markers = ["other-start", "other-end"].map(|name| {
        let node = batch.create_processing_instruction(&document, name, "");
        batch.insert_before(&node, &unrelated_target, None);
        node
    });
    let unrelated_text = batch.create_text(&document, "unrelated literal survives");
    batch.insert_before(&unrelated_text, &unrelated_target, None);
    let templates = std::array::from_fn::<_, 4, _>(|_| {
        let template = batch.create_element_ns(&document, Namespace::html(), "template");
        batch.insert_before(&template, &root, None);
        template
    });
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let other_document = commit.document(&other_document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let subtree = commit.created(&subtree).unwrap().unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let markers = markers.map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let unrelated_target = commit.created(&unrelated_target).unwrap().unwrap();
    let unrelated_markers =
        unrelated_markers.map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let unrelated_text = commit.created(&unrelated_text).unwrap().unwrap();
    let templates = templates.map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let patches = [
        [Some(target), Some(markers[0]), Some(markers[1])],
        [Some(target), Some(markers[0]), Some(markers[3])],
        [Some(target), Some(markers[2]), Some(markers[1])],
        [
            Some(unrelated_target),
            Some(unrelated_markers[0]),
            Some(unrelated_markers[1]),
        ],
    ];
    let before = dom.snapshot();
    let batch = DomBatch::new();
    let mut draft = Draft::new(&dom, batch.owner.clone(), &cancel).unwrap();
    for (template, fields) in templates.into_iter().zip(patches) {
        let mut relation = draft
            .template_mut(Key::Existing(template), &cancel)
            .unwrap();
        relation.template_patch = Some(TemplatePatch {
            target: fields[0].map(Key::Existing),
            start: fields[1].map(Key::Existing),
            end: fields[2].map(Key::Existing),
        });
        relation.commit(&cancel).unwrap();
    }
    let prepared = draft.finish(&dom, &cancel).unwrap();
    let commit = dom.publish(prepared, &cancel).unwrap();
    assert_publication(&commit, &before, &[]);
    assert_patches(&dom.snapshot(), templates, patches);
    Fixture {
        dom,
        document,
        other_document,
        root,
        subtree,
        target,
        markers,
        unrelated_target,
        unrelated_markers,
        unrelated_text,
        templates,
        patches,
    }
}

fn assert_unrelated(fixture: &Fixture) {
    let snapshot = fixture.dom.snapshot();
    assert_eq!(patch(&snapshot, fixture.templates[3]), fixture.patches[3]);
    for id in [
        fixture.root,
        fixture.unrelated_target,
        fixture.unrelated_markers[0],
        fixture.unrelated_markers[1],
    ] {
        assert!(snapshot.node(id).is_ok());
    }
    assert_eq!(
        snapshot
            .node(fixture.unrelated_text)
            .unwrap()
            .data()
            .unwrap(),
        &DocumentString::from("unrelated literal survives")
    );
}

#[test]
fn retiring_start_clears_only_shared_start_slots_and_reused_generation_stays_unbound() {
    let mut fixture = fixture();
    let before = fixture.dom.snapshot();
    let retired = fixture.markers[0];
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.detach(retired);
    batch.destroy(retired);
    let commit = fixture.dom.apply(batch, &cancel).unwrap();
    let mut expected = fixture.patches;
    expected[0][1] = None;
    expected[1][1] = None;
    assert_publication(&commit, &before, &[retired]);
    assert_patches(&fixture.dom.snapshot(), fixture.templates, expected);
    assert_retired(&fixture.dom.snapshot(), &before, &[retired]);
    assert_patches(&before, fixture.templates, fixture.patches);
    assert_unrelated(&fixture);

    let mut batch = DomBatch::new();
    let replacement =
        batch.create_processing_instruction(fixture.document, "replacement-start", "");
    let commit = fixture.dom.apply(batch, &cancel).unwrap();
    let replacement = commit.created(&replacement).unwrap().unwrap();
    assert_eq!(replacement.slot, retired.slot);
    assert_ne!(replacement.generation, retired.generation);
    assert_patches(&fixture.dom.snapshot(), fixture.templates, expected);
    assert!(fixture.dom.snapshot().node(retired).is_err());
    assert!(before.node(retired).is_ok());
}

#[test]
fn retiring_end_preserves_distinct_live_starts_and_unrelated_end() {
    let mut fixture = fixture();
    let before = fixture.dom.snapshot();
    let retired = fixture.markers[1];
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.detach(retired);
    batch.destroy(retired);
    let commit = fixture.dom.apply(batch, &cancel).unwrap();
    let mut expected = fixture.patches;
    expected[0][2] = None;
    expected[2][2] = None;
    assert_publication(&commit, &before, &[retired]);
    assert_patches(&fixture.dom.snapshot(), fixture.templates, expected);
    assert_retired(&fixture.dom.snapshot(), &before, &[retired]);
    assert_patches(&before, fixture.templates, fixture.patches);
    assert_unrelated(&fixture);
}

#[test]
fn target_retirement_preserves_live_detached_and_adopted_markers() {
    let mut fixture = fixture();
    let cancel = AtomicBool::new(false);
    let original = fixture.dom.snapshot();
    let mut batch = DomBatch::new();
    for marker in fixture.markers {
        batch.detach(marker);
    }
    batch.adopt(fixture.markers[0], fixture.other_document);
    batch.detach(fixture.target);
    let commit = fixture.dom.apply(batch, &cancel).unwrap();
    // Structural edits conservatively invalidate TemplateRelations. The weak
    // slots themselves, rather than absence of that domain, prove no cleanup.
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::Parent(fixture.target))
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::NodeDocument(fixture.markers[0]))
    );
    assert_patches(&fixture.dom.snapshot(), fixture.templates, fixture.patches);
    assert_eq!(
        fixture
            .dom
            .snapshot()
            .node(fixture.markers[0])
            .unwrap()
            .owner_document()
            .unwrap(),
        fixture.other_document
    );
    let before = fixture.dom.snapshot();
    let mut batch = DomBatch::new();
    batch.destroy(fixture.target);
    let commit = fixture.dom.apply(batch, &cancel).unwrap();
    let mut expected = fixture.patches;
    for fields in &mut expected[..3] {
        fields[0] = None;
    }
    assert_publication(&commit, &before, &[fixture.target]);
    assert_patches(&fixture.dom.snapshot(), fixture.templates, expected);
    assert_retired(&fixture.dom.snapshot(), &original, &[fixture.target]);
    for marker in fixture.markers {
        assert!(fixture.dom.snapshot().node(marker).is_ok());
    }
    assert_patches(&original, fixture.templates, fixture.patches);
    assert_unrelated(&fixture);
}

#[test]
fn destroying_owning_subtree_clears_each_surviving_template_and_preserves_old_snapshot() {
    let mut fixture = fixture();
    let before = fixture.dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.detach(fixture.subtree);
    batch.destroy(fixture.subtree);
    let commit = fixture.dom.apply(batch, &cancel).unwrap();
    let retired = [
        fixture.subtree,
        fixture.target,
        fixture.markers[0],
        fixture.markers[1],
        fixture.markers[2],
        fixture.markers[3],
    ];
    let mut expected = fixture.patches;
    expected[..3].fill([None; 3]);
    assert_publication(&commit, &before, &retired);
    assert_patches(&fixture.dom.snapshot(), fixture.templates, expected);
    assert_retired(&fixture.dom.snapshot(), &before, &retired);
    assert_patches(&before, fixture.templates, fixture.patches);
    assert_unrelated(&fixture);
}

#[test]
fn created_then_destroyed_patch_target_has_no_published_identity_or_dangling_index() {
    let mut fixture = fixture();
    let before = fixture.dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let target = batch.create_element_ns(fixture.document, Namespace::html(), "article");
    batch.destroy(&target);
    let mut draft = Draft::new(&fixture.dom, batch.owner.clone(), &cancel).unwrap();
    draft.apply(&batch.commands[0], limits(), &cancel).unwrap();
    let created = draft.resolve((&target).into()).unwrap();
    assert!(matches!(created, Key::Created(_)));
    let mut relation = draft
        .template_mut(Key::Existing(fixture.templates[0]), &cancel)
        .unwrap();
    relation.template_patch.as_mut().unwrap().target = Some(created);
    relation.commit(&cancel).unwrap();
    assert_eq!(
        draft
            .node(Key::Existing(fixture.templates[0]))
            .unwrap()
            .template_patch()
            .unwrap()
            .target,
        Some(created)
    );
    assert!(
        draft
            .inbound_keys(created, &cancel)
            .unwrap()
            .contains(&Key::Existing(fixture.templates[0]))
    );
    draft.apply(&batch.commands[1], limits(), &cancel).unwrap();
    assert_eq!(
        draft
            .node(Key::Existing(fixture.templates[0]))
            .unwrap()
            .template_patch()
            .unwrap()
            .target,
        None
    );
    assert!(draft.inbound_keys(created, &cancel).unwrap().is_empty());
    let prepared = draft.finish(&fixture.dom, &cancel).unwrap();
    let commit = fixture.dom.publish(prepared, &cancel).unwrap();
    assert_eq!(commit.created(&target).unwrap(), None);
    assert_publication(&commit, &before, &[]);
    assert!(
        !commit
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::Created(_) | DomChange::Destroyed(_)))
    );
    let mut expected = fixture.patches;
    expected[0][0] = None;
    assert_patches(&fixture.dom.snapshot(), fixture.templates, expected);
    assert_patches(&before, fixture.templates, fixture.patches);
    assert_unrelated(&fixture);
}

fn assert_private_start_cleanup(draft: &mut Draft, fixture: &Fixture, cancel: &AtomicBool) {
    for template in fixture.templates[..2].iter().copied() {
        let value = draft
            .node(Key::Existing(template))
            .unwrap()
            .template_patch()
            .unwrap();
        assert_eq!(value.start, None);
        assert_eq!(value.target, Some(Key::Existing(fixture.target)));
    }
    assert!(
        draft
            .inbound_keys(Key::Existing(fixture.markers[0]), cancel)
            .unwrap()
            .is_empty()
    );
    assert!(draft.node(Key::Existing(fixture.markers[0])).is_err());
}

#[test]
fn cancellation_after_nonnull_patch_cleanup_keeps_canonical_relations_and_index_unchanged() {
    let mut fixture = fixture();
    let before = fixture.dom.snapshot();
    let usage = fixture.dom.usage();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.detach(fixture.markers[0]);
    batch.destroy(fixture.markers[0]);
    let mut draft = Draft::new(&fixture.dom, batch.owner.clone(), &cancel).unwrap();
    for command in &batch.commands {
        draft.apply(command, limits(), &cancel).unwrap();
    }
    assert_private_start_cleanup(&mut draft, &fixture, &cancel);
    let prepared = draft.finish(&fixture.dom, &cancel).unwrap();
    cancel.store(true, Ordering::Release);
    assert_eq!(
        fixture.dom.publish(prepared, &cancel).unwrap_err(),
        DomError::Cancelled
    );
    drop(draft);
    assert_eq!(*fixture.dom.state, *before.state);
    assert_eq!(fixture.dom.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.dom.usage(), usage);
    assert_patches(&fixture.dom.snapshot(), fixture.templates, fixture.patches);
    cancel.store(false, Ordering::Release);
    let commit = fixture.dom.apply(batch, &cancel).unwrap();
    assert_publication(&commit, &before, &[fixture.markers[0]]);
}

#[test]
fn finite_refusal_after_nonnull_patch_cleanup_rolls_back_retirement_and_can_retry() {
    let mut fixture = fixture();
    let before = fixture.dom.snapshot();
    let usage = fixture.dom.usage();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    batch.detach(fixture.markers[0]);
    batch.destroy(fixture.markers[0]);
    let failed_ticket = batch.create_text(
        fixture.document,
        "x".repeat(limits().string_code_units() + 1),
    );
    let mut draft = Draft::new(&fixture.dom, batch.owner.clone(), &cancel).unwrap();
    for command in &batch.commands[..2] {
        draft.apply(command, limits(), &cancel).unwrap();
    }
    assert_private_start_cleanup(&mut draft, &fixture, &cancel);
    assert_eq!(
        draft
            .apply(&batch.commands[2], limits(), &cancel)
            .unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: limits().string_code_units(),
        }
    );
    assert!(draft.resolve((&failed_ticket).into()).is_err());
    drop(draft);
    assert_eq!(
        fixture.dom.apply(batch, &cancel).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: limits().string_code_units(),
        }
    );
    assert_eq!(*fixture.dom.state, *before.state);
    assert_eq!(fixture.dom.snapshot().stamp(), before.stamp());
    assert_eq!(fixture.dom.usage(), usage);
    assert_patches(&fixture.dom.snapshot(), fixture.templates, fixture.patches);
    let mut retry = DomBatch::new();
    retry.detach(fixture.markers[0]);
    retry.destroy(fixture.markers[0]);
    let commit = fixture.dom.apply(retry, &cancel).unwrap();
    assert_publication(&commit, &before, &[fixture.markers[0]]);
}
