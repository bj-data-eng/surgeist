//! Genuine inner-CssStops resource qualification, using the width child's
//! resources() and the core's real captured source builder. No dummy provider
//! or Cell is constructed.
//!
//! Named storage expectations use real capacities/lengths/type sizes, never
//! production visitor totals. Selected readback is only a named-subset lower
//! bound; clone live-byte deltas are a complete census of the actual deep-copy
//! fields below. Public union tests qualify identity/lifetime deduplication,
//! not exact transient peak, provider CPU, fallback fonts or trial counts.
//! Low preparation caps bound the whole operation, including existing container
//! metrics; they do NOT isolate new hypothetical SPACE/ch probe metering.

use super::*;
use crate::engine::layout::data::{CellPayload, ClusterContent, ShapedData};
use crate::engine::scope_alignment::{accounting, census, visit_prepared_allocations};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use std::mem::{size_of, size_of_val};
use std::sync::Arc;

fn default_capture(text: &str) -> Captured {
    captured(
        text,
        TabSize::spaces(1.0).unwrap(),
        20.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::All,
        LineOrientation::Vertical,
        COMBINED_TAB_WIDTH_FAMILY,
    )
}
fn limited_capture(text: &str, limits: ScopeAlignmentLimits) -> Captured {
    captured_with_limits(
        Case {
            text,
            size: TabSize::spaces(1.0).unwrap(),
            root_size: 20.0,
            orientation: LineOrientation::Vertical,
            inner: ParagraphBaseDirection::LeftToRight,
        },
        TextCombineUpright::All,
        COMBINED_TAB_WIDTH_FAMILY,
        limits,
    )
}
fn limits(capture: &Captured) -> ScopeAlignmentLimits {
    capture
        .processed
        .captured_scope(0)
        .unwrap()
        .scope_alignment_limits()
        .unwrap()
}
fn allowance(capture: &Captured) -> ScopeAlignmentAllowance {
    let caps = limits(capture);
    ScopeAlignmentAllowance::new(caps.records(), caps.payload_bytes())
}
fn limit(error: Error, expected: ScopeAlignmentLimitKind) {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment {
        reason:
            ScopeAlignmentError::Limit {
                kind,
                required,
                limit,
            },
    }) = error.detail()
    else {
        panic!("actual scoped admission refusal: {error:?}");
    };
    assert_eq!(*kind, expected);
    assert!(required > limit);
}
fn premises(prepared: &PreparedText, capture: &Captured, text: &str) {
    source_premises(
        capture,
        text,
        ParagraphBaseDirection::LeftToRight,
        &vec![0; text.len()],
    );
    let views = prepared.cell_views().collect::<Vec<_>>();
    let [view] = views.as_slice() else {
        panic!("one genuine prepared Cell");
    };
    assert_eq!(view.extent(), 10.0);
    assert_eq!(view.has_provider_input(), text == "B\t");
    let shaped = &prepared.inner.shaped;
    let [cell] = shaped.cells.as_slice() else {
        panic!("one actual Cell row");
    };
    let CellPayload::LaidOut { layout, provider } = &cell.payload else {
        panic!("nonempty CSS TAB layout");
    };
    assert_eq!(provider.is_some(), text == "B\t");
    assert_eq!(layout.advance, 0.0..10.0);
    assert_eq!(
        (layout.transform.x_scale, layout.transform.x_offset),
        (1.0, 0.0)
    );
    assert!(!layout.positions.is_empty());
    let [span] = layout.tabs.as_slice() else {
        panic!("one executed TAB receipt");
    };
    let before = if text == "B\t" { 5.0 } else { 0.0 };
    assert_eq!(
        (span.atom, span.before, span.after, span.amount),
        (text.len() - 1, before, 10.0, 10.0 - before)
    );
    assert!(layout.clusters.contains(&span.cluster));
    assert!(matches!(
        shaped.clusters[span.cluster].content,
        ClusterContent::CssTab(_)
    ));
    assert!(shaped.empty_run.is_none());
    if text == "\t" {
        assert!(shaped.runs.is_empty());
        assert!(shaped.glyphs.is_empty());
        assert!(shaped.provider_groups.is_empty());
    } else {
        assert_eq!(shaped.glyphs.len(), 1);
    }
}

#[test]
fn inner_tab_preparation_refuses_insufficient_source_admitted_work_rows_and_payload() {
    for text in ["\t", "B\t"] {
        let (mut system, _bytes) = resources();
        let capture = default_capture(text);
        let seed = capture
            .processed
            .captured_scope(0)
            .unwrap()
            .scope_alignment_usage()
            .unwrap();
        let full = limits(&capture);
        let prepared = system
            .prepare(&capture.analyzed, options(LineOrientation::Vertical))
            .unwrap();
        premises(&prepared, &capture, text);
        // Use actual completed capture accounting solely to admit SOURCE.
        // These caps leave no room for the real preparation's new material.
        // They do not assert a particular font query count or refusal site.
        for (kind, caps) in [
            (
                ScopeAlignmentLimitKind::Records,
                ScopeAlignmentLimits::new(
                    seed.records(),
                    full.font_queries(),
                    full.structural_work(),
                    full.payload_bytes(),
                ),
            ),
            (
                ScopeAlignmentLimitKind::StructuralWork,
                ScopeAlignmentLimits::new(
                    full.records(),
                    full.font_queries(),
                    seed.structural_work(),
                    full.payload_bytes(),
                ),
            ),
            (
                ScopeAlignmentLimitKind::PayloadBytes,
                ScopeAlignmentLimits::new(
                    full.records(),
                    full.font_queries(),
                    full.structural_work(),
                    seed.peak_payload_bytes(),
                ),
            ),
        ] {
            // A Source construction failure is a setup failure, not evidence
            // that System::prepare enforced the requested operation limit.
            let small = limited_capture(text, caps);
            source_premises(
                &small,
                text,
                ParagraphBaseDirection::LeftToRight,
                &vec![0; text.len()],
            );
            let error = system
                .prepare(&small.analyzed, options(LineOrientation::Vertical))
                .unwrap_err();
            limit(error, kind);
        }
        // Failed independent preparations cannot invalidate a completed input.
        assert_eq!(
            complete(&prepared, TabContext::NoTabs).feedback().advance,
            10.0
        );
    }
}

#[derive(Default, Clone, Copy)]
struct Footprint {
    rows: usize,
    bytes: usize,
}
impl Footprint {
    fn buffer<T>(&mut self, value: &Vec<T>) {
        self.rows += value.len();
        self.bytes += value.capacity() * size_of::<T>();
    }
}
struct Expected {
    identity: census::AllocationIdentity,
    rows: usize,
    bytes: usize,
}
fn object<T>(values: &T) -> Expected {
    Expected {
        identity: census::AllocationWitness::object(values, 0).identity(),
        rows: 0,
        bytes: size_of_val(values),
    }
}
fn buffer<T>(values: &Vec<T>) -> Option<Expected> {
    (values.capacity() != 0).then(|| Expected {
        identity: census::AllocationWitness::buffer(values, 0)
            .unwrap()
            .identity(),
        rows: values.len(),
        bytes: values.capacity() * size_of::<T>(),
    })
}
fn owned<T>(out: &mut Vec<Expected>, values: &Vec<T>) {
    out.push(object(values));
    out.extend(buffer(values));
}
fn named(shaped: &ShapedData) -> Vec<Expected> {
    let mut out = Vec::new();
    // These are actual WHOLE adopted arenas, not fictional per-Cell slices.
    owned(&mut out, &shaped.cells);
    owned(&mut out, &shaped.items);
    owned(&mut out, &shaped.runs);
    owned(&mut out, &shaped.clusters);
    owned(&mut out, &shaped.glyphs);
    owned(&mut out, &shaped.provider_groups);
    for run in &shaped.runs {
        out.extend(buffer(&run.coords));
    }
    for cell in &shaped.cells {
        if let CellPayload::LaidOut { layout, .. } = &cell.payload {
            // Both Vec headers are already embedded in the actual Cell row.
            out.extend(buffer(&layout.positions));
            out.extend(buffer(&layout.tabs));
        }
    }
    out
}
fn named_prepared(prepared: &PreparedText, expected: &[Expected]) {
    let data = &prepared.inner;
    let mut seen = vec![0; expected.len()];
    visit_prepared_allocations(
        &data.scope_alignment,
        &data.analyzed,
        &data.shaped,
        &data.tables,
        &mut |actual| {
            for (index, row) in expected.iter().enumerate() {
                if row.identity == actual.identity() {
                    seen[index] += 1;
                    assert_eq!(actual.records(), row.rows);
                    assert_eq!(actual.payload_bytes(), row.bytes);
                }
            }
            Ok(())
        },
    )
    .unwrap();
    assert!(seen.iter().all(|count| *count == 1));
}

#[test]
fn real_tab_only_and_normal_fit_mixed_layout_buffers_have_retained_capacity() {
    for text in ["\t", "B\t"] {
        let (mut system, _bytes) = resources();
        let capture = default_capture(text);
        let prepared = system
            .prepare(&capture.analyzed, options(LineOrientation::Vertical))
            .unwrap();
        premises(&prepared, &capture, text);
        named_prepared(&prepared, &named(&prepared.inner.shaped));
        let selected = prepared
            .select(request(&prepared, TabContext::NoTabs))
            .unwrap();
        let rows = named(&selected.inner.basis.shaped);
        let own = rows.iter().fold(Footprint::default(), |mut total, row| {
            total.rows += row.rows;
            total.bytes += row.bytes;
            total
        });
        let input = prepared.scope_alignment_retention().unwrap();
        let retained = selected.scope_alignment_retention().unwrap();
        // Selected own buffers are distinct live allocations; its strong
        // Prepared ancestor remains retained. This is explicitly a lower bound,
        // not a claim of complete selected Graph/scopes/reference accounting.
        assert!(retained.records() >= input.records() + own.rows);
        assert!(retained.payload_bytes() >= input.payload_bytes() + own.bytes);
        assert_eq!(
            selected
                .finalize(LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: ScopeAllocations::None
                })
                .unwrap()
                .feedback()
                .advance,
            10.0
        );
    }
}

// Complete owned live storage of an ACTUAL clone_shaped output. Shared FontData,
// requirements/exclusion Arcs and Prepared tab references remain input custody.
// No nested Vec header is added twice: it is embedded in its outer row/struct.
fn cloned_storage(shaped: &ShapedData) -> Footprint {
    let mut out = Footprint {
        rows: 0,
        bytes: size_of::<ShapedData>(),
    };
    out.buffer(&shaped.runs);
    for run in &shaped.runs {
        out.buffer(&run.coords);
    }
    out.buffer(&shaped.clusters);
    out.buffer(&shaped.glyphs);
    out.buffer(&shaped.provider_groups);
    out.buffer(&shaped.scope_glyph_custody);
    out.buffer(&shaped.scope_relations);
    out.buffer(&shaped.font_queries);
    out.buffer(&shaped.character_gaps);
    out.buffer(&shaped.character_invocations);
    for invocation in &shaped.character_invocations {
        out.buffer(&invocation.components);
        for row in &invocation.components {
            out.buffer(row);
        }
        out.buffer(&invocation.generated);
        for row in &invocation.generated {
            out.buffer(row);
        }
        out.buffer(&invocation.constraints);
        for row in &invocation.constraints {
            out.buffer(&row.components);
            out.buffer(&row.provenance);
        }
    }
    out.buffer(&shaped.atomic_gaps.rows);
    out.buffer(&shaped.word_spacing);
    for row in &shaped.word_spacing {
        out.buffer(&row.components);
    }
    out.buffer(&shaped.items);
    out.buffer(&shaped.cells);
    for cell in &shaped.cells {
        if let CellPayload::LaidOut { layout, .. } = &cell.payload {
            out.buffer(&layout.positions);
            out.buffer(&layout.tabs);
        }
    }
    if let Some(run) = &shaped.empty_run {
        out.buffer(&run.coords);
    }
    out
}

#[test]
fn real_tab_layout_deep_clones_fund_distinct_buffers_and_retire_their_actual_bytes() {
    for text in ["\t", "B\t"] {
        let (mut system, _bytes) = resources();
        let capture = default_capture(text);
        let prepared = system
            .prepare(&capture.analyzed, options(LineOrientation::Vertical))
            .unwrap();
        premises(&prepared, &capture, text);
        let selected = prepared
            .select(request(&prepared, TabContext::NoTabs))
            .unwrap();
        let source = &selected.inner.basis.shaped;
        let retained = selected.scope_alignment_retention().unwrap();
        let mut ledger = Some(
            ScopeAlignmentBudget::new(
                limits(&capture),
                retained.records(),
                retained.payload_bytes(),
            )
            .unwrap(),
        );
        let first = accounting::clone_shaped(&mut ledger, source).unwrap();
        let first_size = cloned_storage(&first);
        assert_eq!(
            ledger.as_ref().unwrap().usage().payload_bytes(),
            retained.payload_bytes() + first_size.bytes
        );
        let second = accounting::clone_shaped(&mut ledger, source).unwrap();
        let second_size = cloned_storage(&second);
        assert_eq!(
            ledger.as_ref().unwrap().usage().payload_bytes(),
            retained.payload_bytes() + first_size.bytes + second_size.bytes
        );
        assert!(
            ledger.as_ref().unwrap().usage().records()
                >= retained.records() + first_size.rows + second_size.rows
        );
        for clone in [&first, &second] {
            assert!(Arc::ptr_eq(
                &source.character_exclusions,
                &clone.character_exclusions
            ));
            match (
                &source.character_requirements,
                &clone.character_requirements,
            ) {
                (Some(a), Some(b)) => assert!(Arc::ptr_eq(a, b)),
                (None, None) => (),
                _ => panic!("clone must preserve the actual shared requirement owner"),
            }
            let CellPayload::LaidOut {
                layout: original, ..
            } = &source.cells[0].payload
            else {
                panic!("real source layout");
            };
            let CellPayload::LaidOut { layout: copied, .. } = &clone.cells[0].payload else {
                panic!("real cloned layout");
            };
            assert_ne!(original.positions.as_ptr(), copied.positions.as_ptr());
            assert_ne!(original.tabs.as_ptr(), copied.tabs.as_ptr());
            assert_eq!(copied.tabs.len(), 1);
            assert_eq!(
                (
                    copied.tabs[0].before,
                    copied.tabs[0].after,
                    copied.tabs[0].amount
                ),
                (
                    original.tabs[0].before,
                    original.tabs[0].after,
                    original.tabs[0].amount
                )
            );
        }
        accounting::drop_shaped(&mut ledger, first).unwrap();
        assert_eq!(
            ledger.as_ref().unwrap().usage().payload_bytes(),
            retained.payload_bytes() + second_size.bytes
        );
        accounting::drop_shaped(&mut ledger, second).unwrap();
        assert_eq!(
            ledger.as_ref().unwrap().usage().payload_bytes(),
            retained.payload_bytes()
        );
        let full = limits(&capture);
        let tight = ScopeAlignmentLimits::new(
            full.records(),
            full.font_queries(),
            full.structural_work(),
            retained.payload_bytes(),
        );
        let mut tight = Some(
            ScopeAlignmentBudget::new(tight, retained.records(), retained.payload_bytes()).unwrap(),
        );
        limit(
            accounting::clone_shaped(&mut tight, source).unwrap_err(),
            ScopeAlignmentLimitKind::PayloadBytes,
        );
        assert_eq!(
            selected
                .finalize(LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: ScopeAllocations::None
                })
                .unwrap()
                .feedback()
                .advance,
            10.0
        );
    }
}

#[test]
fn tab_results_replay_with_real_old_new_coexistence_alias_dedup_and_tight_refusal() {
    for text in ["\t", "B\t"] {
        let (mut system, _bytes) = resources();
        let capture = default_capture(text);
        let prepared = system
            .prepare(&capture.analyzed, options(LineOrientation::Vertical))
            .unwrap();
        premises(&prepared, &capture, text);
        let maximum = allowance(&capture);
        let input = prepared.scope_alignment_retention().unwrap();
        limit(
            prepared
                .select_with_scope_allowance(
                    request(&prepared, TabContext::NoTabs),
                    ScopeAlignmentAllowance::new(input.records() - 1, maximum.payload_bytes()),
                )
                .unwrap_err(),
            ScopeAlignmentLimitKind::Records,
        );
        let selected = prepared
            .select_with_scope_allowance(request(&prepared, TabContext::NoTabs), maximum)
            .unwrap();
        let line = selected
            .finalize_with_scope_allowance(
                LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: ScopeAllocations::None,
                },
                maximum,
            )
            .unwrap();
        let retained = line.scope_alignment_retention().unwrap();
        limit(
            line.with_spacing_with_scope_allowance(
                SpacingAllocations::None,
                ScopeAlignmentAllowance::new(maximum.records(), retained.payload_bytes() - 1),
            )
            .unwrap_err(),
            ScopeAlignmentLimitKind::PayloadBytes,
        );
        let replay = line
            .with_spacing_with_scope_allowance(SpacingAllocations::None, maximum)
            .unwrap();
        assert_eq!(replay.selected_line(), selected);
        assert_eq!(replay.feedback().advance, 10.0);
        assert_eq!(line.feedback().advance, 10.0);
        let input = ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&replay));
        let ancestors = [
            ScopeAlignmentProduct::Prepared(&prepared),
            ScopeAlignmentProduct::Selected(&selected),
        ];
        let shared =
            scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&ancestors), maximum)
                .unwrap();
        assert_eq!(shared.external_retention().records(), 0);
        assert_eq!(shared.external_retention().payload_bytes(), 0);
        let old = [ScopeAlignmentProduct::Finalized(&line)];
        let both =
            scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&old), maximum).unwrap();
        assert!(both.external_retention().records() > 0);
        assert!(both.external_retention().payload_bytes() > 0);
        let alias = line.clone();
        let aliases = [
            ScopeAlignmentProduct::Finalized(&line),
            ScopeAlignmentProduct::Finalized(&alias),
        ];
        let dedup = scope_alignment_census(input, ScopeAlignmentProducts::Mixed(&aliases), maximum)
            .unwrap();
        assert_eq!(dedup.input_retention(), both.input_retention());
        assert_eq!(dedup.external_retention(), both.external_retention());
        let live =
            both.input_retention().payload_bytes() + both.external_retention().payload_bytes();
        assert_eq!(both.usage().payload_bytes(), live);
        limit(
            scope_alignment_census(
                input,
                ScopeAlignmentProducts::Mixed(&old),
                ScopeAlignmentAllowance::new(maximum.records(), live - 1),
            )
            .unwrap_err(),
            ScopeAlignmentLimitKind::PayloadBytes,
        );
        drop(system);
        assert_eq!(
            line.with_spacing(SpacingAllocations::None)
                .unwrap()
                .feedback()
                .advance,
            10.0
        );
    }
}
