//! Font-free supplied membership components, parent fragments and body roots.
//! Row ordinals and owner keys are normalized input incidences, not Source IDs.
use crate as t;
use std::ops::Range;

fn reorder(levels: &[u8]) -> Vec<usize> {
    let rows = levels
        .iter()
        .map(|&level| t::SuppliedInlineParticipant::Text {
            level: t::BidiLevel::try_new(level).unwrap(),
            edge: t::InlineWhitespaceEdge::Preserve,
        })
        .collect::<Vec<_>>();
    let mut scratch = vec![usize::MAX; rows.len()];
    t::SuppliedLine::new(&rows)
        .reorder_into(&mut scratch)
        .unwrap()
        .iter()
        .map(|(row, _)| row)
        .collect()
}

fn rows(count: usize) -> Vec<t::SuppliedInlineBoxComponentRow> {
    (0..count)
        .map(|row| t::SuppliedInlineBoxComponentRow {
            owners: row..row + 1,
            participation: t::InlineBoxComponentParticipation::Participating,
        })
        .collect()
}

fn admit(
    root: usize,
    owners: &[usize],
    parents: &[Option<usize>],
    rows: &[t::SuppliedInlineBoxComponentRow],
    memberships: &[usize],
    order: &[usize],
    bodies: &[Range<usize>],
) -> t::Result<t::SuppliedInlineBoxComponents> {
    t::SuppliedInlineBoxComponents::try_new(t::SuppliedInlineBoxComponentsInput {
        root,
        owners,
        parents,
        rows,
        memberships,
        order,
        bodies,
    })
}

type Description = (usize, usize, Range<usize>);
type Component = (usize, usize, Range<usize>, Option<Description>);

fn description(data: &t::SuppliedInlineBoxComponents, index: usize) -> Description {
    let component = data
        .component(index)
        .expect("the retained component index exists");
    (
        component.owner(),
        component.body(),
        component.visual_range(),
    )
}

fn snapshot(data: &t::SuppliedInlineBoxComponents) -> Vec<Component> {
    let mut components = data
        .components()
        .map(|component| {
            (
                component.owner(),
                component.body(),
                component.visual_range(),
                component
                    .parent_index()
                    .map(|parent| description(data, parent)),
            )
        })
        .collect::<Vec<_>>();
    components.sort_by_key(|c| (c.1, c.0, c.2.start, c.2.end));
    components
}

fn association(data: &t::SuppliedInlineBoxComponents, row: usize, owner: usize) -> Option<usize> {
    let mut found = data.row_components(row).filter(|&index| {
        data.component(index)
            .expect("the retained component index exists")
            .owner()
            == owner
    });
    let component = found.next();
    assert!(
        found.next().is_none(),
        "one actual row has at most one component of this owner"
    );
    component
}

fn invalid(result: t::Result<t::SuppliedInlineBoxComponents>) {
    let Err(error) = result else {
        panic!("malformed supplied incidence must refuse without publishing components")
    };
    assert_eq!(error.code, t::ErrorCode::InvalidTextProtocol);
    assert_eq!(
        error.detail(),
        Some(&t::ErrorDetail::InvalidTextProtocol {
            reason: t::ProtocolError::InlineBoxFragmentation(
                t::InlineBoxFragmentationError::InvalidIncidence
            ),
        })
    );
}

#[test]
fn genuine_supplied_l2_order_splits_one_child_membership_without_splitting_the_root() {
    let order = reorder(&[0, 0, 0, 1, 1, 1]);
    assert_eq!(order, [0, 1, 2, 5, 4, 3]);
    // The two C rows are logical2/3. The actual supplied visual permutation
    // puts foreign participating root rows5/4 between them.
    let data = admit(
        0,
        &[0, 1],
        &[None, Some(0)],
        &rows(6),
        &[0, 0, 1, 1, 0, 0],
        &order,
        std::slice::from_ref(&(0..6)),
    )
    .unwrap();
    assert_eq!(
        snapshot(&data),
        [
            (0, 0, 0..6, None),
            (1, 0, 2..3, Some((0, 0, 0..6))),
            (1, 0, 5..6, Some((0, 0, 0..6))),
        ]
    );
    let root = association(&data, 0, 0).unwrap();
    assert_eq!(association(&data, 1, 0), Some(root));
    assert_eq!(association(&data, 4, 0), Some(root));
    assert_eq!(association(&data, 5, 0), Some(root));
    let c = association(&data, 2, 1).unwrap();
    let a = association(&data, 3, 1).unwrap();
    assert_ne!(c, a);
    assert_eq!(description(&data, c), (1, 0, 2..3));
    assert_eq!(description(&data, a), (1, 0, 5..6));
    assert_eq!(
        data.component(c)
            .expect("the retained component index exists")
            .parent_index(),
        Some(root)
    );
    assert_eq!(
        data.component(a)
            .expect("the retained component index exists")
            .parent_index(),
        Some(root)
    );
    assert!(association(&data, 0, 1).is_none() && association(&data, 4, 1).is_none());
}

#[test]
fn nested_components_parent_to_corresponding_fragments_and_share_only_the_root_ancestor() {
    let order = reorder(&[0, 0, 0, 1, 1, 1]);
    assert_eq!(order, [0, 1, 2, 5, 4, 3]);
    let data = admit(
        0,
        &[0, 1, 2],
        &[None, Some(0), Some(1)],
        &rows(6),
        &[0, 1, 2, 2, 0, 0],
        &order,
        std::slice::from_ref(&(0..6)),
    )
    .unwrap();
    assert_eq!(
        snapshot(&data),
        [
            (0, 0, 0..6, None),
            (1, 0, 1..3, Some((0, 0, 0..6))),
            (1, 0, 5..6, Some((0, 0, 0..6))),
            (2, 0, 2..3, Some((1, 0, 1..3))),
            (2, 0, 5..6, Some((1, 0, 5..6))),
        ]
    );
    let cc = association(&data, 2, 2).unwrap();
    let ca = association(&data, 3, 2).unwrap();
    let pc = data
        .component(cc)
        .expect("the retained component index exists")
        .parent_index()
        .unwrap();
    let pa = data
        .component(ca)
        .expect("the retained component index exists")
        .parent_index()
        .unwrap();
    assert_ne!(cc, ca);
    assert_ne!(pc, pa);
    assert_eq!(description(&data, pc), (1, 0, 1..3));
    assert_eq!(description(&data, pa), (1, 0, 5..6));
    assert_eq!(association(&data, 1, 1), Some(pc));
    let root = data
        .component(pc)
        .expect("the retained component index exists")
        .parent_index()
        .unwrap();
    assert_eq!(
        data.component(pa)
            .expect("the retained component index exists")
            .parent_index(),
        Some(root)
    );
    assert_eq!(description(&data, root), (0, 0, 0..6));
    assert!(
        data.component(root)
            .expect("the retained component index exists")
            .parent_index()
            .is_none()
    );
    // Both complete child parent chains are child -> distinct P fragment ->
    // this root. Hence their nearest common ancestor is root, not owner key P.
}

#[test]
fn metadata_is_transparent_but_the_same_foreign_row_when_participating_splits() {
    let order = reorder(&[0, 0, 0]);
    assert_eq!(order, [0, 1, 2]);
    let mut input_rows = rows(3);
    input_rows[1].participation = t::InlineBoxComponentParticipation::Metadata;
    let transparent = admit(
        0,
        &[0, 1],
        &[None, Some(0)],
        &input_rows,
        &[1, 0, 1],
        &order,
        std::slice::from_ref(&(0..3)),
    )
    .unwrap();
    input_rows[1].participation = t::InlineBoxComponentParticipation::Participating;
    let separated = admit(
        0,
        &[0, 1],
        &[None, Some(0)],
        &input_rows,
        &[1, 0, 1],
        &order,
        std::slice::from_ref(&(0..3)),
    )
    .unwrap();
    assert_eq!(
        snapshot(&transparent),
        [(0, 0, 0..3, None), (1, 0, 0..3, Some((0, 0, 0..3))),]
    );
    assert_eq!(
        association(&transparent, 0, 1),
        association(&transparent, 2, 1)
    );
    assert!(association(&transparent, 0, 1).is_some());
    assert!(
        association(&transparent, 1, 1).is_none(),
        "a span across neutral metadata does not donate membership"
    );
    assert_eq!(
        snapshot(&separated),
        [
            (0, 0, 0..3, None),
            (1, 0, 0..1, Some((0, 0, 0..3))),
            (1, 0, 2..3, Some((0, 0, 0..3))),
        ]
    );
    assert_ne!(association(&separated, 0, 1), association(&separated, 2, 1));
    assert!(association(&separated, 1, 1).is_none());
    // No advance/width enters this front: participation, not a guessed pixel
    // extent, distinguishes these two otherwise identical supplied inputs.
}

#[test]
fn differing_levels_inside_contiguous_membership_do_not_create_an_extra_component() {
    let order = reorder(&[0, 0, 0, 1, 0, 0]);
    assert_eq!(order, [0, 1, 2, 3, 4, 5]);
    let data = admit(
        0,
        &[0, 1],
        &[None, Some(0)],
        &rows(6),
        &[0, 0, 1, 1, 0, 0],
        &order,
        std::slice::from_ref(&(0..6)),
    )
    .unwrap();
    assert_eq!(
        snapshot(&data),
        [(0, 0, 0..6, None), (1, 0, 2..4, Some((0, 0, 0..6)))]
    );
    assert_eq!(association(&data, 2, 1), association(&data, 3, 1));
    assert!(association(&data, 2, 1).is_some());
}

#[test]
fn an_empty_body_has_its_own_root_and_no_child_component_without_a_participating_row() {
    let order = reorder(&[0, 0]);
    assert_eq!(order, [0, 1]);
    let data = admit(
        0,
        &[0, 1],
        &[None, Some(0)],
        &rows(2),
        &[1, 1],
        &order,
        &[0..2, 2..2],
    )
    .unwrap();
    assert_eq!(
        snapshot(&data),
        [
            (0, 0, 0..2, None),
            (1, 0, 0..2, Some((0, 0, 0..2))),
            (0, 1, 2..2, None),
        ]
    );
    let roots = data
        .components()
        .enumerate()
        .filter(|(_, c)| c.owner() == 0)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 2);
    assert_ne!(roots[0], roots[1]);
    assert!(data.components().all(|c| c.body() != 1 || c.owner() == 0));
    for row in 0..2 {
        let child = association(&data, row, 1).unwrap();
        assert_eq!(
            data.component(child)
                .expect("the retained component index exists")
                .body(),
            0
        );
        assert!(data.row_components(row).all(|i| {
            data.component(i)
                .expect("the retained component index exists")
                .body()
                == 0
        }));
    }
}

#[test]
fn sparse_active_owner_keys_stop_parentage_at_the_designated_family_root() {
    let order = reorder(&[0, 0]);
    assert_eq!(order, [0, 1]);
    // Original parent0 is exterior to this active family. It must not become
    // an owner/NCA or invalidate root2's local formatter boundary.
    let parents = [None, None, Some(0), None, None, Some(2)];
    let data = admit(
        2,
        &[2, 5],
        &parents,
        &rows(2),
        &[2, 5],
        &order,
        std::slice::from_ref(&(0..2)),
    )
    .unwrap();
    assert_eq!(
        snapshot(&data),
        [(2, 0, 0..2, None), (5, 0, 1..2, Some((2, 0, 0..2)))]
    );
    let root = association(&data, 0, 2).unwrap();
    let child = association(&data, 1, 5).unwrap();
    assert!(
        data.component(root)
            .expect("the retained component index exists")
            .parent_index()
            .is_none()
    );
    assert_eq!(
        data.component(child)
            .expect("the retained component index exists")
            .parent_index(),
        Some(root)
    );
    assert!(data.components().all(|c| c.owner() == 2 || c.owner() == 5));
}

#[test]
fn a_non_root_parent_cycle_refuses_as_invalid_incidence() {
    // Root0 is a stop boundary; the invalid cycle is solely non-root1<->2.
    invalid(admit(
        0,
        &[0, 1, 2],
        &[None, Some(2), Some(1)],
        &rows(2),
        &[1, 2],
        &[0, 1],
        std::slice::from_ref(&(0..2)),
    ));
}

#[test]
fn out_of_bounds_and_inactive_foreign_memberships_refuse_precisely() {
    invalid(admit(
        0,
        &[0, 1],
        &[None, Some(0)],
        &rows(2),
        &[1, 99],
        &[0, 1],
        std::slice::from_ref(&(0..2)),
    ));
    let parents = [None, None, Some(0), None, None, Some(2)];
    invalid(admit(
        2,
        &[2, 5],
        &parents,
        &rows(2),
        &[5, 0],
        &[0, 1],
        std::slice::from_ref(&(0..2)),
    ));
}

#[test]
fn duplicate_missing_and_out_of_bounds_visual_order_incidence_refuse_precisely() {
    for order in [&[0, 0][..], &[0][..], &[0, 2][..]] {
        invalid(admit(
            0,
            &[0],
            &[None],
            &rows(2),
            &[0, 0],
            order,
            std::slice::from_ref(&(0..2)),
        ));
    }
}

#[test]
fn overlapping_body_ranges_refuse_instead_of_duplicating_row_incidence() {
    invalid(admit(
        0,
        &[0],
        &[None],
        &rows(3),
        &[0, 0, 0],
        &[0, 1, 2],
        &[0..2, 1..3],
    ));
}

#[test]
fn one_participating_row_retains_both_sibling_components_and_their_shared_root() {
    let order = reorder(&[0, 0]);
    assert_eq!(order, [0, 1]);
    let input_rows = [
        t::SuppliedInlineBoxComponentRow {
            owners: 0..2,
            participation: t::InlineBoxComponentParticipation::Participating,
        },
        t::SuppliedInlineBoxComponentRow {
            owners: 2..3,
            participation: t::InlineBoxComponentParticipation::Participating,
        },
    ];
    // One actual normalized row belongs to siblings1/2. Unrelated sibling3
    // has its own participating row, never a borrowed membership in row0.
    let data = admit(
        0,
        &[0, 1, 2, 3],
        &[None, Some(0), Some(0), Some(0)],
        &input_rows,
        &[1, 2, 3],
        &order,
        std::slice::from_ref(&(0..2)),
    )
    .unwrap();
    assert_eq!(
        snapshot(&data),
        [
            (0, 0, 0..2, None),
            (1, 0, 0..1, Some((0, 0, 0..2))),
            (2, 0, 0..1, Some((0, 0, 0..2))),
            (3, 0, 1..2, Some((0, 0, 0..2))),
        ]
    );
    let root = association(&data, 0, 0).unwrap();
    let first = association(&data, 0, 1).unwrap();
    let second = association(&data, 0, 2).unwrap();
    assert_ne!(first, second);
    assert_ne!(first, root);
    assert_ne!(second, root);
    assert_eq!(
        data.component(first)
            .expect("the retained component index exists")
            .parent_index(),
        Some(root)
    );
    assert_eq!(
        data.component(second)
            .expect("the retained component index exists")
            .parent_index(),
        Some(root)
    );
    let mut shared = data
        .row_components(0)
        .map(|index| description(&data, index))
        .collect::<Vec<_>>();
    shared.sort_by_key(|component| (component.0, component.1, component.2.start, component.2.end));
    assert_eq!(shared, [(0, 0, 0..2), (1, 0, 0..1), (2, 0, 0..1)]);
    assert!(association(&data, 0, 3).is_none());
    assert_eq!(association(&data, 1, 0), Some(root));
    assert!(association(&data, 1, 1).is_none() && association(&data, 1, 2).is_none());
    let separate = association(&data, 1, 3).unwrap();
    assert_eq!(description(&data, separate), (3, 0, 1..2));
    assert_eq!(
        data.component(separate)
            .expect("the retained component index exists")
            .parent_index(),
        Some(root)
    );
}

#[test]
fn participating_rows_require_real_membership_while_unowned_metadata_remains_valid() {
    let mut input_rows = [t::SuppliedInlineBoxComponentRow {
        owners: 0..0,
        participation: t::InlineBoxComponentParticipation::Metadata,
    }];
    let metadata = admit(
        0,
        &[0],
        &[None],
        &input_rows,
        &[],
        &[0],
        std::slice::from_ref(&(0..1)),
    )
    .unwrap();
    assert_eq!(snapshot(&metadata), [(0, 0, 0..1, None)]);
    assert!(metadata.row_components(0).next().is_none());

    input_rows[0].participation = t::InlineBoxComponentParticipation::Participating;
    input_rows[0].owners = 0..1;
    let rooted = admit(
        0,
        &[0],
        &[None],
        &input_rows,
        &[0],
        &[0],
        std::slice::from_ref(&(0..1)),
    )
    .unwrap();
    assert_eq!(snapshot(&rooted), [(0, 0, 0..1, None)]);
    assert_eq!(rooted.row_components(0).collect::<Vec<_>>(), [0]);

    input_rows[0].owners = 0..0;
    invalid(admit(
        0,
        &[0],
        &[None],
        &input_rows,
        &[],
        &[0],
        std::slice::from_ref(&(0..1)),
    ));
}
