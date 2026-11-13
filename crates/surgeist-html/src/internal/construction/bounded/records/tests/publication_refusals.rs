//! The publication of the storage refuses every class of invalid reference and association with
//! its typed cause, hands the storage back as it was and grows no bank. A result a storage
//! publishes therefore holds only references that resolve, which is why a row that does not
//! resolve reads as an unreadable node only in a tree that no storage built.
//!
//! Each test corrupts one row or one entry of the association table of [`rich_tree`] through the
//! arena, which the operations of the storage never do, and publishes the tree. A row holds the
//! shadow properties of a fragment and the owner of a fragment in words that no other kind
//! reads, and it names an association by an index, so a record of another storage, a host that
//! the owner of its entry does not name and shadow properties on a record that is not a fragment
//! are not states a row can hold.

use super::*;
use crate::internal::construction::rows::{
    BuildAssoc, BuildAssocKind, NodeFlags, NodeRow, RowKind,
};
use HtmlConstructionCause as Cause;

/// The position of the first row of `storage` that `which` accepts.
fn first(storage: &HtmlConstructionStorage, which: impl Fn(&NodeRow) -> bool) -> usize {
    storage.arena.rows.iter().position(which).unwrap()
}
/// The position of the owner of the first entry of the association table that `which` accepts.
fn owner_of(storage: &HtmlConstructionStorage, which: impl Fn(&BuildAssoc) -> bool) -> usize {
    storage
        .arena
        .associations
        .iter()
        .find(|entry| which(entry))
        .unwrap()
        .owner
        .position()
}
/// The position of the entry of the association table that `which` accepts.
fn entry_of(storage: &HtmlConstructionStorage, which: impl Fn(&BuildAssoc) -> bool) -> usize {
    storage.arena.associations.iter().position(which).unwrap()
}
/// The row of the first element that has children and no contents.
fn parent_element(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |row| {
        row.kind == RowKind::Element
            && row.first().is_some()
            && !row.flags.has(NodeFlags::HAS_ASSOC)
    })
}
/// The template that refers to shadow contents.
fn referring_template(storage: &HtmlConstructionStorage) -> usize {
    owner_of(storage, |entry| {
        entry.kind == BuildAssocKind::ShadowReference
    })
}
/// The shadow host that has a shadow association.
fn shadow_host(storage: &HtmlConstructionStorage) -> usize {
    owner_of(storage, |entry| entry.kind == BuildAssocKind::Shadow)
}
/// The first text row, which has no children.
fn text_record(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |row| row.kind == RowKind::Text)
}
/// The element that nothing holds and that has no association.
fn loose_element(storage: &HtmlConstructionStorage) -> usize {
    first(storage, |row| {
        row.kind == RowKind::Element && row.parent.is_none() && !row.flags.has(NodeFlags::HAS_ASSOC)
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
fn a_link_naming_a_record_past_the_records_is_refused_as_an_unknown_node() {
    let past = Some(NodeIx::at(1000));
    // The first child of a parent.
    assert_refused(
        |storage, _| {
            let parent = parent_element(storage);
            storage.arena.rows[parent].first_child = past;
        },
        Cause::UnknownNode,
    );
    assert_refused(
        |storage, root| {
            let root = position(storage, root);
            storage.arena.rows[root].first_child = past;
        },
        Cause::UnknownNode,
    );
    // The next sibling of a child.
    assert_refused(
        |storage, _| {
            let text = text_record(storage);
            storage.arena.rows[text].next = past;
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
            storage.arena.rows[loose].parent = Some(NodeIx::at(1000));
        },
        Cause::UnknownNode,
    );
    // A record in a list whose owner it does not name as its parent is met first, by the list.
    // Publication checks records in index order, and `rich_tree` creates each parent before its
    // children, so the parent's list is checked before the child's own parent.
    assert_refused(
        |storage, _| {
            let text = text_record(storage);
            storage.arena.rows[text].parent = Some(NodeIx::at(1000));
        },
        Cause::ParentChildConflict,
    );
}

#[test]
fn a_template_contents_that_the_template_and_the_fragment_disagree_about_is_refused() {
    // Contents past the records.
    assert_refused(
        |storage, _| {
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Owned);
            storage.arena.associations[entry].contents = NodeIx::at(1000);
        },
        Cause::UnknownNode,
    );
    // Contents on a record that is not an element.
    assert_refused(
        |storage, _| {
            let text = text_record(storage);
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Owned);
            storage.arena.associations[entry].owner = NodeIx::at(text);
        },
        Cause::InvalidTemplateAssociation,
    );
    // Contents that are an element, not a fragment.
    assert_refused(
        |storage, _| {
            let loose = loose_element(storage);
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Owned);
            storage.arena.associations[entry].contents = NodeIx::at(loose);
        },
        Cause::InvalidTemplateAssociation,
    );
    // Contents that carry shadow properties.
    assert_refused(
        |storage, _| {
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Owned);
            let owned = storage.arena.associations[entry].contents.position();
            storage.arena.rows[owned].a = NodeRow::fragment(Some(shadow_properties())).a;
        },
        Cause::InvalidTemplateAssociation,
    );
    // Contents that another template owns.
    assert_refused(
        |storage, _| {
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Owned);
            let owned = storage.arena.associations[entry].contents.position();
            let other = NodeIx::at(referring_template(storage));
            storage.arena.rows[owned].set_owner(Some(other));
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
            let owned = entry_of(storage, |entry| entry.kind == BuildAssocKind::Owned);
            let owned = storage.arena.associations[owned].contents;
            let referring = entry_of(storage, |entry| {
                entry.kind == BuildAssocKind::ShadowReference
            });
            storage.arena.associations[referring].contents = owned;
        },
        Cause::InvalidContentsReference,
    );
    // The shadow contents lose their properties.
    assert_refused(
        |storage, _| {
            let referring = entry_of(storage, |entry| {
                entry.kind == BuildAssocKind::ShadowReference
            });
            let shadow = storage.arena.associations[referring].contents.position();
            storage.arena.rows[shadow].a = 0;
        },
        Cause::InvalidContentsReference,
    );
}

#[test]
fn a_shadow_association_that_the_host_and_the_contents_disagree_about_is_refused() {
    // The entry names another record as its host.
    assert_refused(
        |storage, _| {
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Shadow);
            storage.arena.associations[entry].owner = NodeIx::at(parent_element(storage));
        },
        Cause::InvalidShadowAssociation,
    );
    // The association is on a record that is not an element.
    assert_refused(
        |storage, _| {
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Shadow);
            storage.arena.associations[entry].owner = NodeIx::at(text_record(storage));
        },
        Cause::InvalidShadowAssociation,
    );
    // The contents are not shadow contents.
    assert_refused(
        |storage, _| {
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Shadow);
            storage.arena.associations[entry].contents = NodeIx::at(parent_element(storage));
        },
        Cause::InvalidShadowAssociation,
    );
    // The contents belong to another host.
    assert_refused(
        |storage, _| {
            let entry = entry_of(storage, |entry| entry.kind == BuildAssocKind::Shadow);
            let shadow = storage.arena.associations[entry].contents.position();
            let other = NodeIx::at(parent_element(storage));
            storage.arena.rows[shadow].set_owner(Some(other));
        },
        Cause::InvalidShadowAssociation,
    );
    // A host row that lost its flag, so that a lookup by owner is never made.
    assert_refused(
        |storage, _| {
            let host = shadow_host(storage);
            storage.arena.rows[host]
                .flags
                .set(NodeFlags::HAS_ASSOC, false);
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
            let loose = NodeIx::at(loose_element(storage));
            storage.arena.rows[fragment].set_owner(Some(loose));
        },
        Cause::InvalidTemplateAssociation,
    );
    // The owner is a record whose shadow association is not this fragment, which carries shadow
    // properties and so names a host.
    assert_refused(
        |storage, _| {
            let fragment =
                ok(storage.create_shadow_contents(shadow_properties(), &mut Meter::default()));
            let fragment = position(storage, &fragment);
            let loose = NodeIx::at(loose_element(storage));
            storage.arena.rows[fragment].set_owner(Some(loose));
        },
        Cause::InvalidShadowAssociation,
    );
    // The owner is a record past the records.
    assert_refused(
        |storage, _| {
            let fragment = loose_fragment(storage);
            storage.arena.rows[fragment].set_owner(Some(NodeIx::at(1000)));
        },
        Cause::UnknownNode,
    );
    // The fragment is marked owned and names no owner.
    assert_refused(
        |storage, _| {
            let fragment = loose_fragment(storage);
            storage.arena.rows[fragment]
                .flags
                .set(NodeFlags::OWNED, true);
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
                storage.arena.rows[record].flags.set(NodeFlags::OWNED, true);
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
                storage.arena.rows[record].flags.set(NodeFlags::OWNED, true);
            },
            Cause::ParentChildConflict,
        );
    }
}
