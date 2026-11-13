//! The publication of the storage refuses every class of invalid reference and association with
//! its typed cause, hands the storage back as it was and grows no bank. A result a storage
//! publishes therefore holds only references that resolve, which is why a row that does not
//! resolve reads as an unreadable node only in a tree that no storage built.
//!
//! Each test corrupts one record of [`rich_tree`] through the arena, which the operations of the
//! storage never do, and publishes the tree.

use super::*;
use crate::internal::construction::{HtmlFragmentOwner, HtmlNodeId};
use HtmlConstructionCause as Cause;

/// The position of the first record of `storage` that `which` accepts.
fn first(storage: &HtmlConstructionStorage, which: impl Fn(&Record) -> bool) -> usize {
    storage.arena.records.iter().position(which).unwrap()
}
/// The identity of the record at `position`.
fn id(storage: &HtmlConstructionStorage, position: usize) -> HtmlNodeId {
    HtmlNodeId {
        tree: storage.arena.serial,
        index: NodeIx::at(position),
    }
}
/// The record of the first element that has children and no contents.
fn parent_element(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |record| {
        matches!(record.kind, NodeContent::Element(_))
            && !record.children.is_empty()
            && record.template_contents.is_none()
    })
}
/// The template that owns its contents.
fn owning_template(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |record| {
        matches!(record.template_contents, Some(TemplateContents::Owned(_)))
    })
}
/// The template that refers to shadow contents.
fn referring_template(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |record| {
        matches!(
            record.template_contents,
            Some(TemplateContents::ShadowReference(_))
        )
    })
}
/// The shadow host that has a shadow association.
fn shadow_host(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |record| record.shadow.is_some())
}
/// The first text record, which has no children.
fn text_record(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |record| {
        matches!(record.kind, NodeContent::Text(_))
    })
}
/// The element that nothing holds and that has no association.
fn loose_element(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |record| {
        matches!(record.kind, NodeContent::Element(_))
            && record.parent.is_none()
            && record.template_contents.is_none()
            && record.shadow.is_none()
    })
}

/// Corrupts [`rich_tree`] with `corrupt`, publishes it with a consumer that refuses nothing, and
/// checks the refusal: its cause is `expected`, the storage is as it was before the call, and the
/// consumer was asked to admit no bank and holds no record of one.
#[track_caller]
fn assert_refused(
    corrupt: impl FnOnce(&mut HtmlConstructionStorage, &HtmlBuilderNode),
    expected: Cause,
) {
    let (mut storage, root) = rich_tree();
    corrupt(&mut storage, &root);
    let before = snapshot(&storage);
    let mut meter = Meter::default();
    let (storage, error) = match publish_tree(storage, &root, &mut meter) {
        Err(refusal) => *refusal,
        Ok(_) => panic!("the publication was not refused, expected {expected:?}"),
    };
    let HtmlChargedError::Faulted(error) = error else {
        panic!("the consumer refused");
    };
    assert_eq!(error.cause(), expected);
    assert_eq!(snapshot(&storage), before, "the storage changed");
    assert!(
        meter.growths.is_empty() && meter.records.is_empty(),
        "a bank was admitted: {:?}",
        meter.growths
    );
}

#[test]
fn the_unmodified_rich_tree_publishes() {
    let (storage, root) = rich_tree();
    assert!(publish_tree(storage, &root, &mut Meter::default()).is_ok());
}

#[test]
fn a_child_entry_naming_a_record_past_the_records_is_refused_as_an_unknown_node() {
    assert_refused(
        |storage, _| {
            let parent = parent_element(storage);
            storage.arena.records[parent]
                .children
                .push_back(NodeIx::at(1000));
        },
        Cause::UnknownNode,
    );
    assert_refused(
        |storage, root| {
            let root = position(storage, root);
            storage.arena.records[root]
                .children
                .push_front(NodeIx::at(1000));
        },
        Cause::UnknownNode,
    );
}

#[test]
fn a_parent_naming_a_record_past_the_records_is_refused() {
    // A record that no list holds names a parent past the records.
    assert_refused(
        |storage, _| {
            let loose = loose_element(storage);
            storage.arena.records[loose].parent = Some(NodeIx::at(1000));
        },
        Cause::UnknownNode,
    );
    // A record in a list whose owner it does not name as its parent is met first, by the list.
    // Publication checks records in index order, and `rich_tree` creates each parent before its
    // children, so the parent's list is checked before the child's own parent.
    assert_refused(
        |storage, _| {
            let text = text_record(storage);
            storage.arena.records[text].parent = Some(NodeIx::at(1000));
        },
        Cause::ParentChildConflict,
    );
}

#[test]
fn a_template_contents_that_the_template_and_the_fragment_disagree_about_is_refused() {
    // Contents past the records.
    assert_refused(
        |storage, _| {
            let template = owning_template(storage);
            storage.arena.records[template].template_contents =
                Some(TemplateContents::Owned(NodeIx::at(1000)));
        },
        Cause::UnknownNode,
    );
    // Contents on a record that is not an element.
    assert_refused(
        |storage, _| {
            let text = text_record(storage);
            let contents = owning_template(storage);
            let Some(TemplateContents::Owned(owned)) =
                storage.arena.records[contents].template_contents
            else {
                panic!("an owning template");
            };
            storage.arena.records[text].template_contents = Some(TemplateContents::Owned(owned));
        },
        Cause::InvalidTemplateAssociation,
    );
    // Contents that are an element, not a fragment.
    assert_refused(
        |storage, _| {
            let template = owning_template(storage);
            let loose = loose_element(storage);
            storage.arena.records[template].template_contents =
                Some(TemplateContents::Owned(NodeIx::at(loose)));
        },
        Cause::InvalidTemplateAssociation,
    );
    // Contents that carry shadow properties.
    assert_refused(
        |storage, _| {
            let template = owning_template(storage);
            let Some(TemplateContents::Owned(owned)) =
                storage.arena.records[template].template_contents
            else {
                panic!("an owning template");
            };
            storage.arena.records[owned.position()].shadow_properties = Some(shadow_properties());
        },
        Cause::InvalidTemplateAssociation,
    );
    // Contents that another template owns.
    assert_refused(
        |storage, _| {
            let template = owning_template(storage);
            let Some(TemplateContents::Owned(owned)) =
                storage.arena.records[template].template_contents
            else {
                panic!("an owning template");
            };
            let other = id(storage, referring_template(storage));
            storage.arena.records[owned.position()].owner =
                Some(HtmlFragmentOwner::Template(other));
        },
        Cause::InvalidTemplateAssociation,
    );
}

#[test]
fn a_template_that_refers_to_contents_without_shadow_properties_is_refused_as_an_invalid_contents_reference()
 {
    // The referring template names the contents of the first template, which are ordinary.
    assert_refused(
        |storage, _| {
            let referring = referring_template(storage);
            let template = owning_template(storage);
            let Some(TemplateContents::Owned(owned)) =
                storage.arena.records[template].template_contents
            else {
                panic!("an owning template");
            };
            storage.arena.records[referring].template_contents =
                Some(TemplateContents::ShadowReference(owned));
        },
        Cause::InvalidContentsReference,
    );
    // The shadow contents lose their properties.
    assert_refused(
        |storage, _| {
            let referring = referring_template(storage);
            let Some(TemplateContents::ShadowReference(shadow)) =
                storage.arena.records[referring].template_contents
            else {
                panic!("a referring template");
            };
            storage.arena.records[shadow.position()].shadow_properties = None;
        },
        Cause::InvalidContentsReference,
    );
}

#[test]
fn a_shadow_association_that_the_host_and_the_contents_disagree_about_is_refused() {
    // The association names another record as its host.
    assert_refused(
        |storage, _| {
            let host = shadow_host(storage);
            let other = id(storage, parent_element(storage));
            if let Some(shadow) = storage.arena.records[host].shadow.as_mut() {
                shadow.host = other;
            }
        },
        Cause::InvalidShadowAssociation,
    );
    // The host and then the contents are of another storage.
    for host_side in [true, false] {
        assert_refused(
            move |storage, _| {
                let host = shadow_host(storage);
                if let Some(shadow) = storage.arena.records[host].shadow.as_mut() {
                    if host_side {
                        shadow.host.tree += 1;
                    } else {
                        shadow.contents.tree += 1;
                    }
                }
            },
            Cause::InvalidShadowAssociation,
        );
    }
    // The association is on a record that is not an element.
    assert_refused(
        |storage, _| {
            let host = shadow_host(storage);
            let text = text_record(storage);
            storage.arena.records[text].shadow = storage.arena.records[host].shadow;
        },
        Cause::InvalidShadowAssociation,
    );
    // The contents are not shadow contents.
    assert_refused(
        |storage, _| {
            let host = shadow_host(storage);
            let element = id(storage, parent_element(storage));
            if let Some(shadow) = storage.arena.records[host].shadow.as_mut() {
                shadow.contents = element;
            }
        },
        Cause::InvalidShadowAssociation,
    );
    // The contents belong to another host.
    assert_refused(
        |storage, _| {
            let host = shadow_host(storage);
            let Some(shadow) = storage.arena.records[host].shadow else {
                panic!("a shadow host");
            };
            let other = id(storage, parent_element(storage));
            storage.arena.records[shadow.contents.index.position()].owner =
                Some(HtmlFragmentOwner::ShadowHost(other));
        },
        Cause::InvalidShadowAssociation,
    );
    // A record that is not a fragment carries shadow properties.
    assert_refused(
        |storage, _| {
            let element = parent_element(storage);
            storage.arena.records[element].shadow_properties = Some(shadow_properties());
        },
        Cause::InvalidShadowAssociation,
    );
}

/// A fragment that nothing owns, in `storage`, which a test gives an owner.
fn loose_fragment(storage: &mut HtmlConstructionStorage) -> usize {
    let fragment = ok(storage.create_fragment(&mut Meter::default()));
    position(storage, &fragment)
}

#[test]
fn a_contents_fragment_whose_owner_does_not_name_it_is_refused() {
    // The owner is a record whose template contents are not this fragment.
    assert_refused(
        |storage, _| {
            let fragment = loose_fragment(storage);
            let loose = id(storage, loose_element(storage));
            storage.arena.records[fragment].owner = Some(HtmlFragmentOwner::Template(loose));
        },
        Cause::InvalidTemplateAssociation,
    );
    // The owner is a record whose shadow association is not this fragment.
    assert_refused(
        |storage, _| {
            let fragment = loose_fragment(storage);
            let loose = id(storage, loose_element(storage));
            storage.arena.records[fragment].owner = Some(HtmlFragmentOwner::ShadowHost(loose));
        },
        Cause::InvalidShadowAssociation,
    );
    // The owner is a record of another storage.
    assert_refused(
        |storage, _| {
            let fragment = loose_fragment(storage);
            let mut foreign = id(storage, loose_element(storage));
            foreign.tree += 1;
            storage.arena.records[fragment].owner = Some(HtmlFragmentOwner::Template(foreign));
        },
        Cause::ForeignBuilder,
    );
    // The owner is a record past the records.
    assert_refused(
        |storage, _| {
            let fragment = loose_fragment(storage);
            let mut past = id(storage, 0);
            past.index = NodeIx::at(1000);
            storage.arena.records[fragment].owner = Some(HtmlFragmentOwner::Template(past));
        },
        Cause::UnknownNode,
    );
}

#[test]
fn an_owner_on_a_detached_record_that_is_not_a_fragment_is_refused_as_already_owned() {
    type Make = fn(&mut HtmlConstructionStorage) -> usize;
    let loose_text: Make = |storage| {
        let text = ok(storage.create_text(&utf16("x"), &mut Meter::default()));
        position(storage, &text)
    };
    let loose_comment: Make = |storage| {
        let comment = ok(storage.create_comment(&utf16("x"), &mut Meter::default()));
        position(storage, &comment)
    };
    let existing: Make = |storage| loose_element(storage);
    for which in [loose_text, loose_comment, existing] {
        assert_refused(
            move |storage, _| {
                let record = which(storage);
                let owner = id(storage, owning_template(storage));
                storage.arena.records[record].owner = Some(HtmlFragmentOwner::Template(owner));
            },
            Cause::AssociationAlreadyOwned,
        );
    }
}

#[test]
fn an_owner_on_a_record_that_a_list_holds_is_refused_as_a_parent_child_conflict() {
    // Publication checks records in index order, and `rich_tree` creates each parent before its
    // children, so the parent's list meets the record before its own owner is checked.
    for which in [text_record, parent_element] {
        assert_refused(
            move |storage, _| {
                let record = which(storage);
                let owner = id(storage, owning_template(storage));
                storage.arena.records[record].owner = Some(HtmlFragmentOwner::Template(owner));
            },
            Cause::ParentChildConflict,
        );
    }
}
