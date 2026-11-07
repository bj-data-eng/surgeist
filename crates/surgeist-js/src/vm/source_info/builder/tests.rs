use crate::vm::source_info::{
    Entry,
    builder::{EntryRange, SourceMapBuilder},
};

fn entries(mut builder: SourceMapBuilder) -> Vec<EntryRange> {
    builder.entries.retain(|entry| !entry.is_empty());
    // println!("\n{:#?}", builder.entries);
    builder.entries.clone()
}

#[test]
fn empty() {
    let builder = SourceMapBuilder::default();

    assert_eq!(entries(builder), Vec::<EntryRange>::new());
}

#[test]
fn single_source_non_overlapping() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    builder.pop_source_position(10);

    assert_eq!(
        entries(builder),
        vec![EntryRange {
            start: 0,
            end: 10,
            position: Some((1, 1).into())
        }]
    );
}

#[test]
fn single_source_overlapping() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    builder.pop_source_position(0);

    assert_eq!(entries(builder), Vec::<EntryRange>::new());
}

#[test]
fn multiple_source_non_overlapping() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(2, Some((3, 1).into()));
        builder.pop_source_position(4);
    }
    builder.pop_source_position(10);

    assert_eq!(
        entries(builder),
        vec![
            EntryRange {
                start: 0,
                end: 2,
                position: Some((1, 1).into())
            },
            EntryRange {
                start: 2,
                end: 4,
                position: Some((3, 1).into())
            },
            EntryRange {
                start: 4,
                end: 10,
                position: Some((1, 1).into())
            },
        ]
    );
}

#[test]
fn multiple_source_full_overlapping() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(0, Some((3, 1).into()));
        builder.pop_source_position(0);
    }
    builder.pop_source_position(0);

    assert_eq!(entries(builder), Vec::<EntryRange>::new(),);
}

#[test]
fn multiple_source_inner_overlapping() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(3, Some((3, 1).into()));
        builder.pop_source_position(3);
    }
    builder.pop_source_position(10);

    assert_eq!(
        entries(builder),
        vec![EntryRange {
            start: 0,
            end: 10,
            position: Some((1, 1).into())
        }]
    );
}

#[test]
fn multiple_source_multiple_inner_non_overlapping() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(3, Some((3, 1).into()));
        builder.pop_source_position(5);

        builder.push_source_position(5, Some((5, 1).into()));
        builder.pop_source_position(7);
    }
    builder.pop_source_position(10);

    assert_eq!(
        entries(builder),
        vec![
            EntryRange {
                start: 0,
                end: 3,
                position: Some((1, 1).into())
            },
            EntryRange {
                start: 3,
                end: 5,
                position: Some((3, 1).into())
            },
            EntryRange {
                start: 5,
                end: 7,
                position: Some((5, 1).into())
            },
            EntryRange {
                start: 7,
                end: 10,
                position: Some((1, 1).into())
            }
        ]
    );
}

#[test]
fn multiple_source_multiple_inner_non_overlapping_with_gap() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(3, Some((3, 1).into()));
        builder.pop_source_position(4);

        builder.push_source_position(5, Some((5, 1).into()));
        builder.pop_source_position(7);
    }
    builder.pop_source_position(10);

    assert_eq!(
        entries(builder),
        vec![
            EntryRange {
                start: 0,
                end: 3,
                position: Some((1, 1).into())
            },
            EntryRange {
                start: 3,
                end: 4,
                position: Some((3, 1).into())
            },
            EntryRange {
                start: 4,
                end: 5,
                position: Some((1, 1).into())
            },
            EntryRange {
                start: 5,
                end: 7,
                position: Some((5, 1).into())
            },
            EntryRange {
                start: 7,
                end: 10,
                position: Some((1, 1).into())
            }
        ]
    );
}

#[test]
fn multiple_source_outer_overlapping() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(0, Some((3, 1).into()));
        builder.pop_source_position(10);
    }
    builder.pop_source_position(10);

    assert_eq!(
        entries(builder),
        vec![EntryRange {
            start: 0,
            end: 10,
            position: Some((3, 1).into())
        }]
    );
}

#[test]
fn finish_does_not_insert_none_position_if_len_is_equal() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    builder.pop_source_position(10);

    assert_eq!(
        builder.build(10),
        vec![Entry {
            pc: 0,
            position: Some((1, 1).into()),
        }]
        .into()
    );
}

#[test]
fn finish_inserts_none_position_if_len_is_not_equal() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    builder.pop_source_position(10);

    assert_eq!(
        builder.build(20),
        vec![
            Entry {
                pc: 0,
                position: Some((1, 1).into()),
            },
            Entry {
                pc: 10,
                position: None,
            }
        ]
        .into()
    );
}

#[test]
fn finish_full_consecutive_duplicates() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(2, Some((1, 1).into()));
        {
            builder.push_source_position(4, Some((1, 1).into()));
            builder.pop_source_position(6);
        }
        builder.pop_source_position(8);
    }
    builder.pop_source_position(10);

    assert_eq!(
        builder.build(20),
        vec![
            Entry {
                pc: 0,
                position: Some((1, 1).into()),
            },
            Entry {
                pc: 10,
                position: None,
            }
        ]
        .into()
    );
}

#[test]
fn finish_outer_consecutive_duplicates() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    {
        builder.push_source_position(2, Some((1, 1).into()));
        {
            builder.push_source_position(4, Some((2, 1).into()));
            builder.pop_source_position(6);
        }
        builder.pop_source_position(8);
    }
    builder.pop_source_position(10);

    assert_eq!(
        builder.build(20),
        vec![
            Entry {
                pc: 0,
                position: Some((1, 1).into()),
            },
            Entry {
                pc: 4,
                position: Some((2, 1).into()),
            },
            Entry {
                pc: 6,
                position: Some((1, 1).into()),
            },
            Entry {
                pc: 10,
                position: None,
            }
        ]
        .into()
    );
}

#[test]
fn finish_inner_consecutive_duplicates() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((2, 1).into()));
    {
        builder.push_source_position(2, Some((1, 1).into()));
        {
            builder.push_source_position(4, Some((1, 1).into()));
            builder.pop_source_position(6);
        }
        builder.pop_source_position(8);
    }
    builder.pop_source_position(10);

    assert_eq!(
        builder.build(20),
        vec![
            Entry {
                pc: 0,
                position: Some((2, 1).into()),
            },
            Entry {
                pc: 2,
                position: Some((1, 1).into()),
            },
            Entry {
                pc: 8,
                position: Some((2, 1).into()),
            },
            Entry {
                pc: 10,
                position: None,
            }
        ]
        .into()
    );
}

#[test]
fn aborted_nested_scopes_leave_no_successful_source_entries() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    builder.push_source_position(2, Some((2, 1).into()));
    builder.push_source_position(4, Some((3, 1).into()));
    builder.abort_source_position();
    builder.abort_source_position();
    builder.abort_source_position();

    // A completed build also verifies that all three live scopes were removed.
    assert_eq!(builder.build(10).as_ref(), &[]);
}

#[test]
fn aborted_child_restores_the_parent_source_range() {
    for child_start in [0, 3] {
        let mut builder = SourceMapBuilder::default();
        builder.push_source_position(0, Some((1, 1).into()));
        builder.push_source_position(child_start, Some((2, 1).into()));
        builder.abort_source_position();
        builder.pop_source_position(10);

        assert_eq!(
            builder.build(10).as_ref(),
            &[Entry {
                pc: 0,
                position: Some((1, 1).into()),
            }]
        );
    }
}

#[test]
fn aborted_parent_discards_completed_descendants_and_restores_outer_range() {
    // A completed descendant can either swap its parent's entry or split it.
    // Aborting that parent must undo both forms of flattening.
    for child_start in [2, 4] {
        let mut builder = SourceMapBuilder::default();
        builder.push_source_position(0, Some((1, 1).into()));
        builder.push_source_position(2, Some((2, 1).into()));
        builder.push_source_position(child_start, Some((3, 1).into()));
        builder.pop_source_position(6);
        builder.abort_source_position();
        builder.pop_source_position(10);

        assert_eq!(
            builder.build(10).as_ref(),
            &[Entry {
                pc: 0,
                position: Some((1, 1).into()),
            }]
        );
    }
}

#[test]
fn aborted_scope_preserves_completed_siblings_and_allows_later_scopes() {
    let mut builder = SourceMapBuilder::default();
    builder.push_source_position(0, Some((1, 1).into()));
    builder.push_source_position(2, Some((2, 1).into()));
    builder.pop_source_position(4);

    builder.push_source_position(5, Some((3, 1).into()));
    builder.push_source_position(5, Some((4, 1).into()));
    builder.pop_source_position(6);
    builder.abort_source_position();

    builder.push_source_position(7, Some((5, 1).into()));
    builder.pop_source_position(8);
    builder.pop_source_position(10);

    assert_eq!(
        builder.build(10).as_ref(),
        &[
            Entry {
                pc: 0,
                position: Some((1, 1).into())
            },
            Entry {
                pc: 2,
                position: Some((2, 1).into())
            },
            Entry {
                pc: 4,
                position: Some((1, 1).into())
            },
            Entry {
                pc: 7,
                position: Some((5, 1).into())
            },
            Entry {
                pc: 8,
                position: Some((1, 1).into())
            },
        ]
    );
}
