use crate::bytecompiler::control::{CompilerControl, CompilerDepth};
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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(10, &mut control).unwrap();

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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(0, &mut control).unwrap();

    assert_eq!(entries(builder), Vec::<EntryRange>::new());
}

#[test]
fn multiple_source_non_overlapping() {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(2, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(4, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(0, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(0, &mut control).unwrap();
    }
    builder.pop_source_position(0, &mut control).unwrap();

    assert_eq!(entries(builder), Vec::<EntryRange>::new(),);
}

#[test]
fn multiple_source_inner_overlapping() {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(3, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(3, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(3, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(5, &mut control).unwrap();

        builder
            .push_source_position(5, Some((5, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(7, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(3, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(4, &mut control).unwrap();

        builder
            .push_source_position(5, Some((5, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(7, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(0, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(10, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(10, &mut control).unwrap();

    assert_eq!(
        builder.build(10, &mut control).unwrap(),
        vec![Entry {
            pc: 0,
            position: Some((1, 1).into()),
        }]
        .into()
    );
}

#[test]
fn finish_inserts_none_position_if_len_is_not_equal() {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(10, &mut control).unwrap();

    assert_eq!(
        builder.build(20, &mut control).unwrap(),
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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(2, Some((1, 1).into()), &mut control)
            .unwrap();
        {
            builder
                .push_source_position(4, Some((1, 1).into()), &mut control)
                .unwrap();
            builder.pop_source_position(6, &mut control).unwrap();
        }
        builder.pop_source_position(8, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

    assert_eq!(
        builder.build(20, &mut control).unwrap(),
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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(2, Some((1, 1).into()), &mut control)
            .unwrap();
        {
            builder
                .push_source_position(4, Some((2, 1).into()), &mut control)
                .unwrap();
            builder.pop_source_position(6, &mut control).unwrap();
        }
        builder.pop_source_position(8, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

    assert_eq!(
        builder.build(20, &mut control).unwrap(),
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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((2, 1).into()), &mut control)
        .unwrap();
    {
        builder
            .push_source_position(2, Some((1, 1).into()), &mut control)
            .unwrap();
        {
            builder
                .push_source_position(4, Some((1, 1).into()), &mut control)
                .unwrap();
            builder.pop_source_position(6, &mut control).unwrap();
        }
        builder.pop_source_position(8, &mut control).unwrap();
    }
    builder.pop_source_position(10, &mut control).unwrap();

    assert_eq!(
        builder.build(20, &mut control).unwrap(),
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
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    builder
        .push_source_position(2, Some((2, 1).into()), &mut control)
        .unwrap();
    builder
        .push_source_position(4, Some((3, 1).into()), &mut control)
        .unwrap();
    builder.abort_source_position();
    builder.abort_source_position();
    builder.abort_source_position();

    // A completed build also verifies that all three live scopes were removed.
    assert_eq!(builder.build(10, &mut control).unwrap().as_ref(), &[]);
}

#[test]
fn aborted_child_restores_the_parent_source_range() {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    for child_start in [0, 3] {
        let mut builder = SourceMapBuilder::default();
        builder
            .push_source_position(0, Some((1, 1).into()), &mut control)
            .unwrap();
        builder
            .push_source_position(child_start, Some((2, 1).into()), &mut control)
            .unwrap();
        builder.abort_source_position();
        builder.pop_source_position(10, &mut control).unwrap();

        assert_eq!(
            builder.build(10, &mut control).unwrap().as_ref(),
            &[Entry {
                pc: 0,
                position: Some((1, 1).into()),
            }]
        );
    }
}

#[test]
fn aborted_parent_discards_completed_descendants_and_restores_outer_range() {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    // A completed descendant can either swap its parent's entry or split it.
    // Aborting that parent must undo both forms of flattening.
    for child_start in [2, 4] {
        let mut builder = SourceMapBuilder::default();
        builder
            .push_source_position(0, Some((1, 1).into()), &mut control)
            .unwrap();
        builder
            .push_source_position(2, Some((2, 1).into()), &mut control)
            .unwrap();
        builder
            .push_source_position(child_start, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(6, &mut control).unwrap();
        builder.abort_source_position();
        builder.pop_source_position(10, &mut control).unwrap();

        assert_eq!(
            builder.build(10, &mut control).unwrap().as_ref(),
            &[Entry {
                pc: 0,
                position: Some((1, 1).into()),
            }]
        );
    }
}

#[test]
fn aborted_scope_preserves_completed_siblings_and_allows_later_scopes() {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    builder
        .push_source_position(2, Some((2, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(4, &mut control).unwrap();

    builder
        .push_source_position(5, Some((3, 1).into()), &mut control)
        .unwrap();
    builder
        .push_source_position(5, Some((4, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(6, &mut control).unwrap();
    builder.abort_source_position();

    builder
        .push_source_position(7, Some((5, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(8, &mut control).unwrap();
    builder.pop_source_position(10, &mut control).unwrap();

    assert_eq!(
        builder.build(10, &mut control).unwrap().as_ref(),
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

fn limited<T>(
    limit: usize,
    run: impl FnOnce(&mut CompilerControl<'_>) -> crate::JsResult<T>,
) -> (crate::JsResult<T>, usize) {
    let mut accepted = 0;
    let mut admit = |_| {
        if accepted == limit {
            return Err(crate::bytecompiler::control::size_overflow());
        }
        accepted += 1;
        Ok(())
    };
    let depth = CompilerDepth::new();
    let result = run(&mut CompilerControl::new(Some(&mut admit), &depth));
    (result, accepted)
}

fn assert_refused<T>(result: crate::JsResult<T>) {
    let Err(error) = result else {
        panic!("source-map work should have been refused");
    };
    assert_eq!(
        error.as_engine(),
        crate::bytecompiler::control::size_overflow().as_engine()
    );
}

fn outer_scope() -> SourceMapBuilder {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = SourceMapBuilder::default();
    builder
        .push_source_position(0, Some((1, 1).into()), &mut control)
        .unwrap();
    builder
}

fn nested_scope(start: u32) -> SourceMapBuilder {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = outer_scope();
    builder
        .push_source_position(start, Some((2, 1).into()), &mut control)
        .unwrap();
    builder
}

fn completed_map(mut builder: SourceMapBuilder) -> Box<[Entry]> {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    builder.pop_source_position(10, &mut control).unwrap();
    builder.build(10, &mut control).unwrap()
}

#[test]
fn every_refused_child_pop_remains_abortable_and_a_retained_pop_can_retry() {
    for (start, end, expected) in [
        (
            0,
            6,
            vec![
                Entry {
                    pc: 0,
                    position: Some((2, 1).into()),
                },
                Entry {
                    pc: 6,
                    position: Some((1, 1).into()),
                },
            ],
        ),
        (
            2,
            6,
            vec![
                Entry {
                    pc: 0,
                    position: Some((1, 1).into()),
                },
                Entry {
                    pc: 2,
                    position: Some((2, 1).into()),
                },
                Entry {
                    pc: 6,
                    position: Some((1, 1).into()),
                },
            ],
        ),
        (
            3,
            3,
            // Preserve the historical tail rule: finalization starts its None
            // sentinel at the last stored range's end, including an empty child.
            vec![
                Entry {
                    pc: 0,
                    position: Some((1, 1).into()),
                },
                Entry {
                    pc: 3,
                    position: None,
                },
            ],
        ),
    ] {
        let mut builder = nested_scope(start);
        let (result, requests) = limited(usize::MAX, |control| {
            builder.pop_source_position(end, control)
        });
        result.unwrap();
        assert_eq!(completed_map(builder).as_ref(), expected);
        assert!(requests > 0);

        let mut exact = nested_scope(start);
        let (result, accepted) =
            limited(requests, |control| exact.pop_source_position(end, control));
        result.unwrap();
        assert_eq!(accepted, requests);
        assert_eq!(completed_map(exact).as_ref(), expected);

        for limit in 0..requests {
            let mut refused = nested_scope(start);
            assert_refused(limited(limit, |control| refused.pop_source_position(end, control)).0);
            // This is the same callback-free recovery required by the compiler's
            // source-position guard when its fallible success-pop refuses.
            refused.abort_source_position();
            assert_eq!(
                completed_map(refused).as_ref(),
                &[Entry {
                    pc: 0,
                    position: Some((1, 1).into()),
                }]
            );
        }

        let mut retry = nested_scope(start);
        assert_refused(
            limited(requests - 1, |control| {
                retry.pop_source_position(end, control)
            })
            .0,
        );
        limited(usize::MAX, |control| {
            retry.pop_source_position(end, control)
        })
        .0
        .unwrap();
        assert_eq!(completed_map(retry).as_ref(), expected);
    }
}

#[test]
fn every_refused_outer_pop_remains_abortable() {
    let mut builder = outer_scope();
    let (result, requests) = limited(usize::MAX, |control| {
        builder.pop_source_position(10, control)
    });
    result.unwrap();
    for limit in 0..requests {
        let mut refused = outer_scope();
        assert_refused(limited(limit, |control| refused.pop_source_position(10, control)).0);
        refused.abort_source_position();
        let result = limited(usize::MAX, |control| refused.build(10, control))
            .0
            .unwrap();
        assert!(result.is_empty());
    }
}

#[test]
fn refused_pop_after_completed_descendants_restores_the_outer_scope() {
    fn setup(descendant_start: u32) -> SourceMapBuilder {
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(None, &depth);
        let mut builder = nested_scope(2);
        builder
            .push_source_position(descendant_start, Some((3, 1).into()), &mut control)
            .unwrap();
        builder.pop_source_position(6, &mut control).unwrap();
        builder
    }
    for descendant_start in [2, 4] {
        let mut builder = setup(descendant_start);
        let (result, requests) = limited(usize::MAX, |control| {
            builder.pop_source_position(8, control)
        });
        result.unwrap();
        for limit in 0..requests {
            let mut refused = setup(descendant_start);
            assert_refused(limited(limit, |control| refused.pop_source_position(8, control)).0);
            refused.abort_source_position();
            assert_eq!(
                completed_map(refused).as_ref(),
                &[Entry {
                    pc: 0,
                    position: Some((1, 1).into()),
                }]
            );
        }
    }
}

#[test]
fn refused_first_push_leaves_an_empty_builder() {
    let mut builder = SourceMapBuilder::default();
    let (result, requests) = limited(usize::MAX, |control| {
        builder.push_source_position(0, Some((1, 1).into()), control)
    });
    result.unwrap();
    let mut exact = SourceMapBuilder::default();
    limited(requests, |control| {
        exact.push_source_position(0, Some((1, 1).into()), control)
    })
    .0
    .unwrap();
    assert_eq!(
        completed_map(exact).as_ref(),
        &[Entry {
            pc: 0,
            position: Some((1, 1).into()),
        }]
    );
    for limit in 0..requests {
        let mut refused = SourceMapBuilder::default();
        assert_refused(
            limited(limit, |control| {
                refused.push_source_position(0, Some((1, 1).into()), control)
            })
            .0,
        );
        assert!(
            limited(usize::MAX, |control| refused.build(10, control))
                .0
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn refused_push_preserves_the_parent_and_allows_a_fresh_scope() {
    let mut builder = outer_scope();
    let (result, requests) = limited(usize::MAX, |control| {
        builder.push_source_position(2, Some((2, 1).into()), control)
    });
    result.unwrap();
    let mut exact = outer_scope();
    let (result, accepted) = limited(requests, |control| {
        exact.push_source_position(2, Some((2, 1).into()), control)
    });
    result.unwrap();
    assert_eq!(accepted, requests);
    limited(usize::MAX, |control| exact.pop_source_position(6, control))
        .0
        .unwrap();
    assert_eq!(
        completed_map(exact).as_ref(),
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
                pc: 6,
                position: Some((1, 1).into())
            },
        ]
    );
    for limit in 0..requests {
        let mut refused = outer_scope();
        assert_refused(
            limited(limit, |control| {
                refused.push_source_position(2, Some((2, 1).into()), control)
            })
            .0,
        );
        assert_eq!(
            completed_map(refused).as_ref(),
            &[Entry {
                pc: 0,
                position: Some((1, 1).into()),
            }]
        );
    }
    let mut retry = outer_scope();
    assert_refused(
        limited(requests - 1, |control| {
            retry.push_source_position(2, Some((2, 1).into()), control)
        })
        .0,
    );
    limited(usize::MAX, |control| {
        retry.push_source_position(2, Some((2, 1).into()), control)?;
        retry.pop_source_position(6, control)
    })
    .0
    .unwrap();
    assert_eq!(
        completed_map(retry).as_ref(),
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
                pc: 6,
                position: Some((1, 1).into())
            },
        ]
    );
}

fn ranges_for_finalization() -> SourceMapBuilder {
    let depth = CompilerDepth::new();
    let mut control = CompilerControl::new(None, &depth);
    let mut builder = outer_scope();
    builder
        .push_source_position(2, Some((1, 1).into()), &mut control)
        .unwrap();
    builder
        .push_source_position(4, Some((2, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(6, &mut control).unwrap();
    builder.pop_source_position(8, &mut control).unwrap();
    builder.pop_source_position(10, &mut control).unwrap();
    builder
        .push_source_position(10, Some((3, 1).into()), &mut control)
        .unwrap();
    builder.pop_source_position(10, &mut control).unwrap();
    builder
}

#[test]
fn finalization_admits_filter_dedup_output_and_exact_or_one_short_storage() {
    let expected = vec![
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
        },
    ];
    let (result, requests) = limited(usize::MAX, |control| {
        ranges_for_finalization().build(20, control)
    });
    assert_eq!(result.unwrap().as_ref(), expected);
    let (exact, accepted) = limited(requests, |control| {
        ranges_for_finalization().build(20, control)
    });
    assert_eq!(accepted, requests);
    assert_eq!(exact.unwrap().as_ref(), expected);
    for limit in 0..requests {
        assert_refused(
            limited(limit, |control| {
                ranges_for_finalization().build(20, control)
            })
            .0,
        );
    }
    // A refused finalization consumes only private scratch. Rebuilding from the
    // same source ranges produces the complete original map.
    let retried = limited(usize::MAX, |control| {
        ranges_for_finalization().build(20, control)
    })
    .0
    .unwrap();
    assert_eq!(retried.as_ref(), expected);
}
