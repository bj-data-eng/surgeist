#[path = "../crates/surgeist-layout/tests/support/mod.rs"]
mod support;

use support::oracle::grid::{
    AlignmentSafety, AutoPlacer, ContributionSize, DefiniteTracks, EqualShareIntrinsicTracks, Flow,
    GridArea, GridAxis, GridItemRect, GridTrack, GrowthLimit, ItemContributionFacts,
    ItemContributions, ItemPlacement, LineNameOrigin, LinePlacement, NamedGridError,
    NamedGridLines, NamedLineOccurrence, PlacementError, Track, TrackAlignment, TrackMax, TrackMin,
    TrackSize, TrackSizingError, TrackSizingSlice, align_tracks, align_tracks_report,
    compose_grid_scenario,
};
use support::oracle::inline;
use support::oracle_tree::{OracleMeasurement, OracleTree};
use surgeist::layout::{
    Available, ComputeInput, ComputeOutput, Dimension, Display, Length, NodeInput, RequestedAxis,
    RunMode, Size, SizingMode, TrackComponent,
};

fn oracle_lane_span(value: usize) -> support::oracle::grid::LaneTrackSpanLength {
    support::oracle::grid::LaneTrackSpanLength::new(value).expect("valid oracle lane span length")
}

#[test]
fn oracle_atomic_inline_item_metrics_include_margins_and_baseline() {
    let item = inline::AtomicInlineItemFacts {
        id: "a",
        size: inline::InlineSize::new(20.0, 10.0),
        margin: inline::InlineEdges {
            top: 2.0,
            right: 3.0,
            bottom: 4.0,
            left: 5.0,
        },
        first_baseline: Some(7.0),
    };

    let metrics = inline::AtomicInlineMetrics::from_item(item);

    assert_eq!(metrics.advance, 28.0);
    assert_eq!(metrics.baseline, 9.0);
    assert_eq!(metrics.descent, 7.0);
    assert_eq!(metrics.margin_box_size, inline::InlineSize::new(28.0, 16.0));
}

#[test]
fn oracle_atomic_inline_item_synthesizes_missing_baseline_from_bottom_edge() {
    let item = inline::AtomicInlineItemFacts {
        id: "a",
        size: inline::InlineSize::new(20.0, 10.0),
        margin: inline::InlineEdges::ZERO,
        first_baseline: None,
    };

    let metrics = inline::AtomicInlineMetrics::from_item(item);

    assert_eq!(metrics.baseline, 10.0);
    assert_eq!(metrics.descent, 0.0);
    assert!(metrics.synthesized_baseline);
}

#[test]
fn oracle_atomic_inline_item_rejects_baseline_before_top_edge() {
    let item = inline::AtomicInlineItemFacts {
        id: "a",
        size: inline::InlineSize::new(20.0, 10.0),
        margin: inline::InlineEdges::ZERO,
        first_baseline: Some(-1.0),
    };

    assert_eq!(
        inline::AtomicInlineMetrics::try_from_item(item),
        Err(inline::AtomicInlineError::BaselineOutOfRange {
            id: "a",
            first_baseline: -1.0,
            height: 10.0,
        })
    );
}

#[test]
fn oracle_atomic_inline_item_rejects_baseline_after_bottom_edge() {
    let item = inline::AtomicInlineItemFacts {
        id: "a",
        size: inline::InlineSize::new(20.0, 10.0),
        margin: inline::InlineEdges::ZERO,
        first_baseline: Some(11.0),
    };

    assert_eq!(
        inline::AtomicInlineMetrics::try_from_item(item),
        Err(inline::AtomicInlineError::BaselineOutOfRange {
            id: "a",
            first_baseline: 11.0,
            height: 10.0,
        })
    );
}

#[test]
fn oracle_atomic_inline_line_aligns_items_to_max_baseline() {
    let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
        available_width: inline::InlineAvailable::Definite(200.0),
        items: vec![
            inline::AtomicInlineItemFacts {
                id: "short",
                size: inline::InlineSize::new(20.0, 10.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(7.0),
            },
            inline::AtomicInlineItemFacts {
                id: "tall",
                size: inline::InlineSize::new(10.0, 20.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(12.0),
            },
        ],
    });

    assert_eq!(report.size, inline::InlineSize::new(30.0, 20.0));
    assert_eq!(report.first_baseline, Some(12.0));
    assert_eq!(report.last_baseline, Some(12.0));
    assert_eq!(report.lines.len(), 1);
    assert_eq!(
        report.lines[0],
        inline::AtomicInlineLine {
            start_item: 0,
            end_item: 2,
            y: 0.0,
            width: 30.0,
            height: 20.0,
            baseline: 12.0,
            descent: 8.0,
        }
    );
    assert_eq!(report.items[0].id, "short");
    assert_eq!(report.items[0].location, inline::InlinePoint::new(0.0, 5.0));
    assert_eq!(
        report.items[1].location,
        inline::InlinePoint::new(20.0, 0.0)
    );
}

#[test]
fn oracle_atomic_inline_line_positions_margin_boxes_and_border_boxes() {
    let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
        available_width: inline::InlineAvailable::Definite(200.0),
        items: vec![
            inline::AtomicInlineItemFacts {
                id: "a",
                size: inline::InlineSize::new(20.0, 10.0),
                margin: inline::InlineEdges {
                    top: 2.0,
                    right: 3.0,
                    bottom: 4.0,
                    left: 5.0,
                },
                first_baseline: Some(7.0),
            },
            inline::AtomicInlineItemFacts {
                id: "b",
                size: inline::InlineSize::new(10.0, 20.0),
                margin: inline::InlineEdges {
                    top: 1.0,
                    right: 2.0,
                    bottom: 3.0,
                    left: 4.0,
                },
                first_baseline: Some(12.0),
            },
        ],
    });

    assert_eq!(report.size, inline::InlineSize::new(44.0, 24.0));
    assert_eq!(
        report.lines[0],
        inline::AtomicInlineLine {
            start_item: 0,
            end_item: 2,
            y: 0.0,
            width: 44.0,
            height: 24.0,
            baseline: 13.0,
            descent: 11.0,
        }
    );
    assert_eq!(report.items[0].location, inline::InlinePoint::new(5.0, 6.0));
    assert_eq!(
        report.items[1].location,
        inline::InlinePoint::new(32.0, 1.0)
    );
}

#[test]
fn oracle_atomic_inline_wraps_between_items_for_definite_width() {
    let item = |id| inline::AtomicInlineItemFacts {
        id,
        size: inline::InlineSize::new(20.0, 10.0),
        margin: inline::InlineEdges::ZERO,
        first_baseline: Some(10.0),
    };

    let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
        available_width: inline::InlineAvailable::Definite(25.0),
        items: vec![item("a"), item("b")],
    });

    assert_eq!(report.size, inline::InlineSize::new(20.0, 20.0));
    assert_eq!(report.lines.len(), 2);
    assert_eq!(
        report.lines[0],
        inline::AtomicInlineLine {
            start_item: 0,
            end_item: 1,
            y: 0.0,
            width: 20.0,
            height: 10.0,
            baseline: 10.0,
            descent: 0.0,
        }
    );
    assert_eq!(
        report.lines[1],
        inline::AtomicInlineLine {
            start_item: 1,
            end_item: 2,
            y: 10.0,
            width: 20.0,
            height: 10.0,
            baseline: 10.0,
            descent: 0.0,
        }
    );
    assert_eq!(report.items[0].location, inline::InlinePoint::new(0.0, 0.0));
    assert_eq!(
        report.items[1].location,
        inline::InlinePoint::new(0.0, 10.0)
    );
    assert_eq!(report.first_baseline, Some(10.0));
    assert_eq!(report.last_baseline, Some(20.0));
}

#[test]
fn oracle_atomic_inline_intrinsic_widths_use_max_item_and_sum() {
    let items = vec![
        inline::AtomicInlineItemFacts {
            id: "a",
            size: inline::InlineSize::new(25.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        },
        inline::AtomicInlineItemFacts {
            id: "b",
            size: inline::InlineSize::new(100.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        },
        inline::AtomicInlineItemFacts {
            id: "c",
            size: inline::InlineSize::new(95.0, 10.0),
            margin: inline::InlineEdges {
                left: 10.0,
                right: 7.0,
                ..inline::InlineEdges::ZERO
            },
            first_baseline: Some(10.0),
        },
    ];

    assert_eq!(inline::atomic_inline_min_content_width(&items), 112.0);
    assert_eq!(inline::atomic_inline_max_content_width(&items), 237.0);
}

#[test]
fn oracle_atomic_inline_min_content_wraps_at_max_item_advance() {
    let items = vec![
        inline::AtomicInlineItemFacts {
            id: "wide",
            size: inline::InlineSize::new(95.0, 10.0),
            margin: inline::InlineEdges {
                left: 10.0,
                right: 7.0,
                ..inline::InlineEdges::ZERO
            },
            first_baseline: Some(10.0),
        },
        inline::AtomicInlineItemFacts {
            id: "next",
            size: inline::InlineSize::new(50.0, 10.0),
            margin: inline::InlineEdges::ZERO,
            first_baseline: Some(10.0),
        },
    ];

    let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
        available_width: inline::InlineAvailable::MinContent,
        items,
    });

    assert_eq!(report.size, inline::InlineSize::new(112.0, 20.0));
    assert_eq!(report.lines.len(), 2);
    assert_eq!(report.lines[0].width, 112.0);
    assert_eq!(report.lines[1].width, 50.0);
}

#[test]
fn oracle_atomic_inline_too_wide_item_overflows_without_empty_line() {
    let item = |id, width| inline::AtomicInlineItemFacts {
        id,
        size: inline::InlineSize::new(width, 10.0),
        margin: inline::InlineEdges::ZERO,
        first_baseline: Some(10.0),
    };

    let report = inline::layout_atomic_inline(inline::AtomicInlineInput {
        available_width: inline::InlineAvailable::Definite(25.0),
        items: vec![item("wide", 40.0), item("next", 10.0)],
    });

    assert_eq!(report.size, inline::InlineSize::new(40.0, 20.0));
    assert_eq!(report.lines.len(), 2);
    assert_eq!(report.lines[0].start_item, 0);
    assert_eq!(report.lines[0].end_item, 1);
    assert_eq!(report.lines[0].width, 40.0);
    assert_eq!(report.lines[1].start_item, 1);
    assert_eq!(report.lines[1].end_item, 2);
    assert_eq!(report.lines[1].width, 10.0);
}

#[test]
fn oracle_atomic_inline_wrapper_preserves_outer_and_inner_display_roles() {
    let cases = [
        (
            inline::InlineOuterDisplay::Block,
            inline::InnerFormattingContext::Block,
        ),
        (
            inline::InlineOuterDisplay::Grid,
            inline::InnerFormattingContext::Grid,
        ),
        (
            inline::InlineOuterDisplay::GridLanes,
            inline::InnerFormattingContext::GridLanes,
        ),
    ];

    for (outer_display, inner_context) in cases {
        let wrapper = inline::AtomicInlineWrapperFacts::new(
            "wrapper",
            outer_display,
            inline::InlineSize::new(40.0, 20.0),
            inline::InlineEdges::ZERO,
            Some(15.0),
        );

        assert_eq!(wrapper.outer_display, outer_display);
        assert_eq!(wrapper.inner_context(), inner_context);
        assert_eq!(
            wrapper.as_item(),
            inline::AtomicInlineItemFacts {
                id: "wrapper",
                size: inline::InlineSize::new(40.0, 20.0),
                margin: inline::InlineEdges::ZERO,
                first_baseline: Some(15.0),
            }
        );
    }
}

#[test]
fn oracle_atomic_inline_wrapper_metrics_use_outer_box_and_margins() {
    let cases = [
        inline::InlineOuterDisplay::Block,
        inline::InlineOuterDisplay::Grid,
        inline::InlineOuterDisplay::GridLanes,
    ];

    for outer_display in cases {
        let wrapper = inline::AtomicInlineWrapperFacts::new(
            "wrapper",
            outer_display,
            inline::InlineSize::new(40.0, 20.0),
            inline::InlineEdges {
                top: 2.0,
                right: 3.0,
                bottom: 4.0,
                left: 5.0,
            },
            Some(15.0),
        );

        let metrics = inline::AtomicInlineMetrics::from_item(wrapper.as_item());

        assert_eq!(metrics.advance, 48.0);
        assert_eq!(metrics.baseline, 17.0);
        assert_eq!(metrics.descent, 9.0);
        assert_eq!(metrics.margin_box_size, inline::InlineSize::new(48.0, 26.0));
    }
}

#[test]
fn oracle_atomic_inline_wrapper_produces_grid_contribution_facts() {
    let wrapper = inline::AtomicInlineWrapperFacts::new(
        "inline-grid",
        inline::InlineOuterDisplay::Grid,
        inline::InlineSize::new(80.0, 30.0),
        inline::InlineEdges::ZERO,
        Some(24.0),
    );

    let contribution = inline::atomic_inline_grid_item_facts(
        wrapper,
        support::oracle::grid::GridArea::new(1, 1, 1, 1),
        80.0,
        80.0,
    );

    assert_eq!(contribution.id, "inline-grid");
    assert_eq!(contribution.outer_display, inline::InlineOuterDisplay::Grid);
    assert_eq!(
        contribution.inner_context(),
        inline::InnerFormattingContext::Grid
    );
    assert_eq!(
        contribution.item.area,
        support::oracle::grid::GridArea::new(1, 1, 1, 1)
    );
    assert_eq!(contribution.item.min_content, 80.0);
    assert_eq!(contribution.item.max_content, 80.0);
    assert_eq!(
        contribution.item.preferred,
        support::oracle::grid::ContributionSize::Definite(80.0)
    );
    assert_eq!(contribution.item.margin_before, 0.0);
    assert_eq!(contribution.item.margin_after, 0.0);
    assert_eq!(contribution.item.contributions().max_content, 80.0);
}

#[test]
fn oracle_atomic_inline_grid_lanes_contribution_preserves_margins() {
    let wrapper = inline::AtomicInlineWrapperFacts::new(
        "inline-grid-lanes",
        inline::InlineOuterDisplay::GridLanes,
        inline::InlineSize::new(80.0, 30.0),
        inline::InlineEdges {
            left: 5.0,
            right: 7.0,
            ..inline::InlineEdges::ZERO
        },
        Some(24.0),
    );

    let contribution = inline::atomic_inline_grid_item_facts(
        wrapper,
        support::oracle::grid::GridArea::new(1, 1, 1, 1),
        60.0,
        80.0,
    );

    assert_eq!(contribution.id, "inline-grid-lanes");
    assert_eq!(
        contribution.outer_display,
        inline::InlineOuterDisplay::GridLanes
    );
    assert_eq!(
        contribution.inner_context(),
        inline::InnerFormattingContext::GridLanes
    );
    assert_eq!(contribution.item.margin_before, 5.0);
    assert_eq!(contribution.item.margin_after, 7.0);
    assert_eq!(contribution.item.contributions().min_content, 72.0);
    assert_eq!(contribution.item.contributions().max_content, 92.0);
}

fn participating_baseline_item() -> support::oracle::grid::BaselineItemFacts {
    support::oracle::grid::BaselineItemFacts {
        id: "item",
        area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
        block_size: 30.0,
        margin_before: 3.0,
        margin_after: 5.0,
        first_baseline: Some(8.0),
        last_baseline: Some(24.0),
        synthesized_first: false,
        synthesized_last: false,
        alignment: support::oracle::grid::BaselineAlignment::First,
        out_of_flow: false,
        baseline_axis_auto_margins: false,
        spans_intrinsic_track: false,
        baseline_requires_unavailable_subgrid_layout: false,
    }
}

fn oracle_baseline_test_item(
    id: &'static str,
    alignment: support::oracle::grid::BaselineAlignment,
) -> support::oracle::grid::BaselineItemFacts {
    support::oracle::grid::BaselineItemFacts {
        id,
        area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
        block_size: 20.0,
        margin_before: 0.0,
        margin_after: 0.0,
        first_baseline: Some(8.0),
        last_baseline: Some(16.0),
        synthesized_first: false,
        synthesized_last: false,
        alignment,
        out_of_flow: false,
        baseline_axis_auto_margins: false,
        spans_intrinsic_track: false,
        baseline_requires_unavailable_subgrid_layout: false,
    }
}

#[allow(clippy::too_many_arguments)]
fn oracle_baseline_item(
    id: &'static str,
    row_start: usize,
    row_span: usize,
    alignment: support::oracle::grid::BaselineAlignment,
    block_size: f32,
    margin_before: f32,
    margin_after: f32,
    first_baseline: Option<f32>,
    last_baseline: Option<f32>,
) -> support::oracle::grid::BaselineItemFacts {
    support::oracle::grid::BaselineItemFacts {
        id,
        area: support::oracle::grid::GridArea::new(1, row_start, 1, row_span),
        block_size,
        margin_before,
        margin_after,
        first_baseline,
        last_baseline,
        synthesized_first: first_baseline.is_none(),
        synthesized_last: last_baseline.is_none(),
        alignment,
        out_of_flow: false,
        baseline_axis_auto_margins: false,
        spans_intrinsic_track: false,
        baseline_requires_unavailable_subgrid_layout: false,
    }
}

#[test]
fn oracle_baseline_geometry_uses_margin_box_contributions() {
    let geometry =
        support::oracle::grid::BaselineGeometry::from_item(participating_baseline_item(), 40.0)
            .unwrap();

    assert_eq!(geometry.margin_box_size, 38.0);
    assert_eq!(geometry.major_baseline, 11.0);
    assert_eq!(geometry.minor_baseline, 11.0);
}

#[test]
fn oracle_baseline_geometry_rejects_non_participating_facts() {
    let unsupported = support::oracle::grid::OracleGridError::BaselineInferenceUnsupported;
    let cases = [
        support::oracle::grid::BaselineItemFacts {
            alignment: support::oracle::grid::BaselineAlignment::None,
            ..participating_baseline_item()
        },
        support::oracle::grid::BaselineItemFacts {
            out_of_flow: true,
            ..participating_baseline_item()
        },
        support::oracle::grid::BaselineItemFacts {
            baseline_axis_auto_margins: true,
            ..participating_baseline_item()
        },
        support::oracle::grid::BaselineItemFacts {
            synthesized_first: true,
            first_baseline: None,
            spans_intrinsic_track: true,
            ..participating_baseline_item()
        },
        support::oracle::grid::BaselineItemFacts {
            alignment: support::oracle::grid::BaselineAlignment::Last,
            synthesized_last: true,
            last_baseline: None,
            baseline_requires_unavailable_subgrid_layout: true,
            ..participating_baseline_item()
        },
    ];

    for item in cases {
        assert_eq!(
            support::oracle::grid::BaselineGeometry::from_item(item, 40.0),
            Err(unsupported)
        );
    }
}

#[test]
fn oracle_baseline_offset_uses_whole_spanned_area_for_major_group() {
    let offset = support::oracle::grid::baseline_offset(
        support::oracle::grid::BaselineGroupKind::Major,
        20.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 11.0,
        },
    );

    assert_eq!(offset, 9.0);
}

#[test]
fn oracle_baseline_offset_uses_whole_spanned_area_for_minor_group() {
    let offset = support::oracle::grid::baseline_offset(
        support::oracle::grid::BaselineGroupKind::Minor,
        12.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 9.0,
        },
    );

    assert_eq!(offset, 34.0);
}

#[test]
fn oracle_baseline_shim_grows_before_for_major_group() {
    let shim = support::oracle::grid::baseline_intrinsic_shim(
        support::oracle::grid::BaselineGroupKind::Major,
        20.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 11.0,
        },
    );

    assert_eq!(
        shim,
        support::oracle::grid::BaselineShim {
            before: 9.0,
            after: 0.0,
        }
    );
}

#[test]
fn oracle_baseline_shim_grows_after_for_minor_group() {
    let shim = support::oracle::grid::baseline_intrinsic_shim(
        support::oracle::grid::BaselineGroupKind::Minor,
        14.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 9.0,
        },
    );

    assert_eq!(
        shim,
        support::oracle::grid::BaselineShim {
            before: 0.0,
            after: 5.0,
        }
    );
}

#[test]
fn oracle_baseline_shim_clamps_negative_major_growth_to_zero() {
    let shim = support::oracle::grid::baseline_intrinsic_shim(
        support::oracle::grid::BaselineGroupKind::Major,
        8.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 9.0,
        },
    );

    assert_eq!(shim, support::oracle::grid::BaselineShim::default());
}

#[test]
fn oracle_baseline_shim_clamps_negative_minor_growth_to_zero() {
    let shim = support::oracle::grid::baseline_intrinsic_shim(
        support::oracle::grid::BaselineGroupKind::Minor,
        7.0,
        support::oracle::grid::BaselineGeometry {
            available_span_size: 75.0,
            margin_box_size: 38.0,
            major_baseline: 11.0,
            minor_baseline: 9.0,
        },
    );

    assert_eq!(shim, support::oracle::grid::BaselineShim::default());
}

#[test]
fn oracle_baseline_participation_rejects_out_of_flow_items() {
    let mut item =
        oracle_baseline_test_item("abspos", support::oracle::grid::BaselineAlignment::First);
    item.out_of_flow = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert!(!report.participates);
    assert_eq!(
        report.fallback,
        Some(support::oracle::grid::BaselineFallback::Start)
    );
}

#[test]
fn oracle_baseline_participation_rejects_auto_margins() {
    let mut item = oracle_baseline_test_item(
        "auto-margin",
        support::oracle::grid::BaselineAlignment::Last,
    );
    item.baseline_axis_auto_margins = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert!(!report.participates);
    assert_eq!(
        report.fallback,
        Some(support::oracle::grid::BaselineFallback::End)
    );
}

#[test]
fn oracle_baseline_participation_falls_back_for_synthesized_intrinsic_cycles() {
    let mut item =
        oracle_baseline_test_item("synth", support::oracle::grid::BaselineAlignment::First);
    item.first_baseline = None;
    item.synthesized_first = true;
    item.spans_intrinsic_track = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert!(!report.participates);
    assert_eq!(
        report.fallback,
        Some(support::oracle::grid::BaselineFallback::Start)
    );
}

#[test]
fn oracle_baseline_participation_falls_back_for_unavailable_subgrid_layout() {
    let mut item = oracle_baseline_test_item(
        "subgrid-synth",
        support::oracle::grid::BaselineAlignment::First,
    );
    item.first_baseline = None;
    item.synthesized_first = true;
    item.baseline_requires_unavailable_subgrid_layout = true;
    let report = support::oracle::grid::baseline_participation(item);

    assert!(!report.participates);
    assert_eq!(
        report.fallback,
        Some(support::oracle::grid::BaselineFallback::Start)
    );
}

#[test]
fn oracle_baseline_participation_none_alignment_does_not_panic() {
    let item = oracle_baseline_test_item("none", support::oracle::grid::BaselineAlignment::None);
    let report = support::oracle::grid::baseline_participation(item);

    assert!(!report.participates);
    assert_eq!(report.group, None);
    assert_eq!(report.fallback, None);
    assert!(!report.used_synthesized_baseline);
}

#[test]
fn oracle_baseline_predicates_ignore_unaligned_synthesized_cycle() {
    let mut item = oracle_baseline_test_item(
        "first-explicit",
        support::oracle::grid::BaselineAlignment::First,
    );
    item.synthesized_last = true;
    item.spans_intrinsic_track = true;

    let report = support::oracle::grid::baseline_participation(item);
    assert!(report.participates);
    assert_eq!(report.fallback, None);
    assert!(!report.used_synthesized_baseline);
    assert!(support::oracle::grid::BaselineGeometry::from_item(item, 40.0).is_ok());
}

#[test]
fn oracle_baseline_predicates_reject_missing_aligned_first_intrinsic_cycle() {
    let mut item = oracle_baseline_test_item(
        "first-missing",
        support::oracle::grid::BaselineAlignment::First,
    );
    item.first_baseline = None;
    item.spans_intrinsic_track = true;

    let report = support::oracle::grid::baseline_participation(item);
    assert!(!report.participates);
    assert_eq!(
        report.fallback,
        Some(support::oracle::grid::BaselineFallback::Start)
    );
    assert_eq!(
        support::oracle::grid::BaselineGeometry::from_item(item, 40.0),
        Err(support::oracle::grid::OracleGridError::BaselineInferenceUnsupported)
    );
}

#[test]
fn oracle_baseline_predicates_reject_missing_aligned_last_intrinsic_cycle() {
    let mut item = oracle_baseline_test_item(
        "last-missing",
        support::oracle::grid::BaselineAlignment::Last,
    );
    item.last_baseline = None;
    item.spans_intrinsic_track = true;

    let report = support::oracle::grid::baseline_participation(item);
    assert!(!report.participates);
    assert_eq!(
        report.fallback,
        Some(support::oracle::grid::BaselineFallback::End)
    );
    assert_eq!(
        support::oracle::grid::BaselineGeometry::from_item(item, 40.0),
        Err(support::oracle::grid::OracleGridError::BaselineInferenceUnsupported)
    );
}

#[test]
fn oracle_baseline_groups_collect_major_group_on_start_track() {
    let report =
        support::oracle::grid::baseline_groups(support::oracle::grid::BaselineGroupInput {
            track_count: 3,
            track_sizes: vec![30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![
                oracle_baseline_item(
                    "a",
                    1,
                    1,
                    support::oracle::grid::BaselineAlignment::First,
                    20.0,
                    3.0,
                    2.0,
                    Some(8.0),
                    Some(16.0),
                ),
                oracle_baseline_item(
                    "b",
                    1,
                    1,
                    support::oracle::grid::BaselineAlignment::First,
                    24.0,
                    1.0,
                    1.0,
                    Some(12.0),
                    Some(18.0),
                ),
            ],
        })
        .unwrap();

    assert_eq!(report.major[0], Some(13.0));
    assert_eq!(report.minor, vec![None, None, None]);
}

#[test]
fn oracle_baseline_groups_collect_minor_group_on_end_track_for_spanning_item() {
    let report =
        support::oracle::grid::baseline_groups(support::oracle::grid::BaselineGroupInput {
            track_count: 3,
            track_sizes: vec![30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![oracle_baseline_item(
                "span",
                1,
                2,
                support::oracle::grid::BaselineAlignment::Last,
                30.0,
                2.0,
                4.0,
                Some(8.0),
                Some(22.0),
            )],
        })
        .unwrap();

    assert_eq!(report.minor[1], Some(12.0));
}

#[test]
fn oracle_baseline_groups_preserve_nonparticipants_without_updating_group() {
    let mut nonparticipant = oracle_baseline_item(
        "absolute",
        1,
        1,
        support::oracle::grid::BaselineAlignment::First,
        80.0,
        20.0,
        0.0,
        Some(40.0),
        Some(60.0),
    );
    nonparticipant.out_of_flow = true;
    let mut empty_row_nonparticipant = oracle_baseline_item(
        "empty-row-absolute",
        2,
        1,
        support::oracle::grid::BaselineAlignment::First,
        80.0,
        20.0,
        0.0,
        Some(40.0),
        Some(60.0),
    );
    empty_row_nonparticipant.out_of_flow = true;

    let report =
        support::oracle::grid::baseline_groups(support::oracle::grid::BaselineGroupInput {
            track_count: 2,
            track_sizes: vec![30.0, 40.0],
            gap: 5.0,
            items: vec![
                oracle_baseline_item(
                    "participant",
                    1,
                    1,
                    support::oracle::grid::BaselineAlignment::First,
                    20.0,
                    1.0,
                    0.0,
                    Some(6.0),
                    Some(14.0),
                ),
                nonparticipant,
                empty_row_nonparticipant,
            ],
        })
        .unwrap();

    assert_eq!(report.participation.len(), 3);
    assert_eq!(report.participation[0].id, "participant");
    assert_eq!(report.participation[1].id, "absolute");
    assert_eq!(report.participation[2].id, "empty-row-absolute");
    assert!(!report.participation[1].participates);
    assert!(!report.participation[2].participates);
    assert_eq!(report.major[0], Some(7.0));
    assert_eq!(report.major[1], None);
}

#[test]
fn oracle_baseline_groups_reject_invalid_track_and_row_spans() {
    let valid_item = oracle_baseline_item(
        "item",
        1,
        1,
        support::oracle::grid::BaselineAlignment::First,
        20.0,
        0.0,
        0.0,
        Some(6.0),
        Some(14.0),
    );
    let invalid_start = support::oracle::grid::BaselineItemFacts {
        area: support::oracle::grid::GridArea::new(1, 0, 1, 1),
        ..valid_item
    };
    let invalid_span = support::oracle::grid::BaselineItemFacts {
        area: support::oracle::grid::GridArea::new(1, 1, 1, 0),
        ..valid_item
    };
    let beyond_tracks = support::oracle::grid::BaselineItemFacts {
        area: support::oracle::grid::GridArea::new(1, 2, 1, 2),
        ..valid_item
    };

    let cases = [
        support::oracle::grid::BaselineGroupInput {
            track_count: 0,
            track_sizes: vec![],
            gap: 0.0,
            items: vec![],
        },
        support::oracle::grid::BaselineGroupInput {
            track_count: 2,
            track_sizes: vec![30.0],
            gap: 0.0,
            items: vec![valid_item],
        },
        support::oracle::grid::BaselineGroupInput {
            track_count: 2,
            track_sizes: vec![30.0, 40.0],
            gap: 0.0,
            items: vec![invalid_start],
        },
        support::oracle::grid::BaselineGroupInput {
            track_count: 2,
            track_sizes: vec![30.0, 40.0],
            gap: 0.0,
            items: vec![invalid_span],
        },
        support::oracle::grid::BaselineGroupInput {
            track_count: 2,
            track_sizes: vec![30.0, 40.0],
            gap: 0.0,
            items: vec![beyond_tracks],
        },
    ];

    for input in cases {
        assert!(support::oracle::grid::baseline_groups(input).is_err());
    }
}

#[test]
fn oracle_baseline_groups_collect_spanning_major_group_on_start_track() {
    let report =
        support::oracle::grid::baseline_groups(support::oracle::grid::BaselineGroupInput {
            track_count: 4,
            track_sizes: vec![20.0, 30.0, 40.0, 50.0],
            gap: 5.0,
            items: vec![oracle_baseline_item(
                "span-major",
                2,
                2,
                support::oracle::grid::BaselineAlignment::First,
                30.0,
                2.0,
                3.0,
                Some(9.0),
                Some(21.0),
            )],
        })
        .unwrap();

    assert_eq!(report.major[1], Some(11.0));
    assert_eq!(report.major[2], None);
}

#[test]
fn oracle_container_baselines_prefer_major_and_minor_groups() {
    let report =
        support::oracle::grid::container_baselines(support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![Some(14.0), None],
                minor: vec![None, Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: vec![
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "first",
                    area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 20.0,
                },
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "last",
                    area: support::oracle::grid::GridArea::new(2, 1, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 10.0,
                    last_baseline: 24.0,
                },
            ],
        })
        .unwrap();

    assert_eq!(report.first, Some(14.0));
    assert_eq!(report.last, Some(64.0));
}

#[test]
fn oracle_container_baselines_use_minor_group_for_first_when_major_missing() {
    let report =
        support::oracle::grid::container_baselines(support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0],
            track_sizes: vec![30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![None],
                minor: vec![Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        })
        .unwrap();

    assert_eq!(report.first, Some(24.0));
    assert_eq!(report.last, Some(24.0));
}

#[test]
fn oracle_container_baselines_use_major_group_for_last_when_minor_missing() {
    let report =
        support::oracle::grid::container_baselines(support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![40.0],
            track_sizes: vec![30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![Some(12.0)],
                minor: vec![None],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        })
        .unwrap();

    assert_eq!(report.first, Some(52.0));
    assert_eq!(report.last, Some(52.0));
}

#[test]
fn oracle_container_baselines_fallback_by_grid_order_and_synthesis() {
    let report =
        support::oracle::grid::container_baselines(support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![None, None],
                minor: vec![None, None],
                participation: Vec::new(),
            },
            fallback_items: vec![
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "row-2-col-1",
                    area: support::oracle::grid::GridArea::new(1, 2, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 70.0,
                    last_baseline: 40.0,
                },
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "row-1-col-2-synth-first",
                    area: support::oracle::grid::GridArea::new(2, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 30.0,
                    last_baseline: 6.0,
                },
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "row-1-col-1",
                    area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 22.0,
                },
            ],
        })
        .unwrap();

    assert_eq!(report.first, Some(8.0));
    assert_eq!(report.last, Some(40.0));
}

#[test]
fn oracle_container_baselines_last_fallback_uses_spanned_end_edge() {
    let report =
        support::oracle::grid::container_baselines(support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0, 80.0],
            track_sizes: vec![30.0, 30.0, 30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![None, None, None],
                minor: vec![None, None, None],
                participation: Vec::new(),
            },
            fallback_items: vec![
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "starts-later",
                    area: support::oracle::grid::GridArea::new(1, 2, 1, 1),
                    block_offset: 40.0,
                    first_baseline: 11.0,
                    last_baseline: 55.0,
                },
                support::oracle::grid::ContainerBaselineFallbackItem {
                    id: "spans-to-last-row",
                    area: support::oracle::grid::GridArea::new(2, 1, 1, 3),
                    block_offset: 0.0,
                    first_baseline: 8.0,
                    last_baseline: 92.0,
                },
            ],
        })
        .unwrap();

    assert_eq!(report.first, Some(8.0));
    assert_eq!(report.last, Some(92.0));
}

#[test]
fn oracle_container_baselines_return_none_for_empty_input() {
    let report =
        support::oracle::grid::container_baselines(support::oracle::grid::ContainerBaselineInput {
            track_offsets: Vec::new(),
            track_sizes: Vec::new(),
            groups: support::oracle::grid::BaselineGroupReport {
                major: Vec::new(),
                minor: Vec::new(),
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        })
        .unwrap();

    assert_eq!(report.first, None);
    assert_eq!(report.last, None);
}

#[test]
fn oracle_container_baselines_reject_vector_shape_mismatches() {
    let cases = [
        support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![Some(14.0), None],
                minor: vec![None, Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        },
        support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![Some(14.0)],
                minor: vec![None, Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        },
        support::oracle::grid::ContainerBaselineInput {
            track_offsets: vec![0.0, 40.0],
            track_sizes: vec![30.0, 30.0],
            groups: support::oracle::grid::BaselineGroupReport {
                major: vec![Some(14.0), None],
                minor: vec![Some(6.0)],
                participation: Vec::new(),
            },
            fallback_items: Vec::new(),
        },
    ];

    for input in cases {
        let error = support::oracle::grid::container_baselines(input).unwrap_err();

        assert_eq!(
            error,
            support::oracle::grid::OracleGridError::SpanOutOfRange
        );
    }
}

#[test]
fn oracle_container_baselines_reject_invalid_fallback_spans() {
    let valid_item = support::oracle::grid::ContainerBaselineFallbackItem {
        id: "fallback",
        area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
        block_offset: 0.0,
        first_baseline: 8.0,
        last_baseline: 22.0,
    };
    let cases = [
        support::oracle::grid::ContainerBaselineFallbackItem {
            area: support::oracle::grid::GridArea::new(1, 0, 1, 1),
            ..valid_item
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            area: support::oracle::grid::GridArea::new(1, 1, 1, 0),
            ..valid_item
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            area: support::oracle::grid::GridArea::new(1, 2, 1, 2),
            ..valid_item
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            area: support::oracle::grid::GridArea::new(0, 1, 1, 1),
            ..valid_item
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            area: support::oracle::grid::GridArea::new(1, 1, 0, 1),
            ..valid_item
        },
    ];

    for item in cases {
        let error = support::oracle::grid::container_baselines(
            support::oracle::grid::ContainerBaselineInput {
                track_offsets: vec![0.0, 40.0],
                track_sizes: vec![30.0, 30.0],
                groups: support::oracle::grid::BaselineGroupReport {
                    major: vec![None, None],
                    minor: vec![None, None],
                    participation: Vec::new(),
                },
                fallback_items: vec![item],
            },
        )
        .unwrap_err();

        assert_eq!(
            error,
            support::oracle::grid::OracleGridError::SpanOutOfRange
        );
    }
}

#[test]
fn grid_definite_tracks_distribute_leftover_space_to_fr_tracks() {
    let tracks = DefiniteTracks::new(300.0, 10.0)
        .track(Track::px(50.0))
        .track(Track::fr(1.0))
        .track(Track::fr(2.0))
        .solve();

    let one_fr = 230.0 / 3.0;
    assert_eq!(tracks.sizes().len(), 3);
    assert_eq!(tracks.size(0), 50.0);
    assert_eq!(tracks.offset(0), 0.0);
    assert!((tracks.size(1) - one_fr).abs() < 0.000_001);
    assert_eq!(tracks.offset(1), 60.0);
    assert!((tracks.size(2) - one_fr * 2.0).abs() < 0.000_001);
    assert!((tracks.offset(2) - (70.0 + one_fr)).abs() < 0.000_001);
}

#[test]
fn grid_explicit_tracks_resolve_percent_and_fr_after_fixed_tracks_and_gaps() {
    let tracks = DefiniteTracks::new(400.0, 20.0)
        .track(Track::px(80.0))
        .track(Track::percent(0.25))
        .track(Track::fr(1.0))
        .track(Track::fr(3.0))
        .solve();

    assert_eq!(tracks.size(0), 80.0);
    assert_eq!(tracks.size(1), 100.0);
    assert_eq!(tracks.size(2), 40.0);
    assert_eq!(tracks.size(3), 120.0);
    assert_eq!(tracks.offset(0), 0.0);
    assert_eq!(tracks.offset(1), 100.0);
    assert_eq!(tracks.offset(2), 220.0);
    assert_eq!(tracks.offset(3), 280.0);
}

#[test]
fn grid_fraction_tracks_do_not_expand_sub_one_factor_to_all_leftover_space() {
    let tracks = DefiniteTracks::new(200.0, 0.0)
        .track(Track::px(50.0))
        .track(Track::fr(0.5))
        .solve();

    assert_eq!(tracks.size(0), 50.0);
    assert_eq!(tracks.size(1), 75.0);

    let report = TrackSizingSlice::definite_columns(200.0, 0.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::flex(0.5))
        .solve();

    assert_eq!(report.flex_fraction, Some(150.0));
    assert_eq!(report.final_tracks[1].size, 75.0);
}

#[test]
fn grid_line_area_resolves_spans_across_tracks_and_gaps() {
    let tracks = DefiniteTracks::new(150.0, 5.0)
        .track(Track::px(30.0))
        .track(Track::px(40.0))
        .track(Track::px(50.0))
        .solve();

    let area = tracks.area(2, 4);

    assert_eq!(area.start, 35.0);
    assert_eq!(area.size, 95.0);
}

#[test]
fn grid_auto_placement_places_row_column_and_dense_items() {
    let mut row = AutoPlacer::try_new(3, 2, Flow::Row)
        .unwrap()
        .occupied(GridArea::new(2, 1, 1, 1));
    assert_eq!(row.place(2, 1).unwrap(), GridArea::new(1, 2, 2, 1));
    assert_eq!(row.place(1, 1).unwrap(), GridArea::new(3, 2, 1, 1));

    let mut column = AutoPlacer::try_new(2, 3, Flow::Column)
        .unwrap()
        .occupied(GridArea::new(1, 2, 1, 1));
    assert_eq!(column.place(1, 2).unwrap(), GridArea::new(2, 1, 1, 2));
    assert_eq!(column.place(1, 1).unwrap(), GridArea::new(2, 3, 1, 1));

    let mut column_dense = AutoPlacer::try_new(2, 3, Flow::ColumnDense)
        .unwrap()
        .occupied(GridArea::new(1, 1, 1, 1))
        .occupied(GridArea::new(1, 2, 1, 1));
    assert_eq!(column_dense.place(1, 1).unwrap(), GridArea::new(1, 3, 1, 1));

    let mut dense = AutoPlacer::try_new(3, 2, Flow::RowDense)
        .unwrap()
        .occupied(GridArea::new(2, 1, 1, 1))
        .occupied(GridArea::new(1, 2, 2, 1));
    assert_eq!(dense.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
}

#[test]
fn grid_auto_placement_reports_zero_explicit_tracks() {
    assert_eq!(
        AutoPlacer::try_new(0, 1, Flow::Row).unwrap_err(),
        PlacementError::NoExplicitTracks(GridAxis::Column)
    );
    assert_eq!(
        AutoPlacer::try_new(1, 0, Flow::Row).unwrap_err(),
        PlacementError::NoExplicitTracks(GridAxis::Row)
    );
}

#[test]
fn grid_auto_placement_reports_row_flow_span_wider_than_columns() {
    let mut placer = AutoPlacer::try_new(2, 1, Flow::Row).unwrap();

    assert_eq!(
        placer.place(3, 1).unwrap_err(),
        PlacementError::SpanExceedsExplicitTracks {
            axis: GridAxis::Column,
            span: 3,
            explicit_tracks: 2,
        }
    );
}

#[test]
fn grid_auto_placement_reports_column_flow_span_taller_than_rows() {
    let mut placer = AutoPlacer::try_new(1, 2, Flow::Column).unwrap();

    assert_eq!(
        placer.place(1, 3).unwrap_err(),
        PlacementError::SpanExceedsExplicitTracks {
            axis: GridAxis::Row,
            span: 3,
            explicit_tracks: 2,
        }
    );
}

#[test]
fn grid_placement_resolves_start_and_end_lines() {
    let placement = LinePlacement::Lines { start: 2, end: 5 }
        .resolve_axis(1)
        .unwrap();

    assert_eq!(placement.start_line, 2);
    assert_eq!(placement.end_line, 5);
    assert_eq!(placement.span, 3);
}

#[test]
fn grid_placement_resolves_start_line_plus_span() {
    let placement = LinePlacement::LineSpan { start: 3, span: 2 }
        .resolve_axis(1)
        .unwrap();

    assert_eq!(placement.start_line, 3);
    assert_eq!(placement.end_line, 5);
    assert_eq!(placement.span, 2);
}

#[test]
fn grid_placement_resolves_span_plus_end_line() {
    let placement = LinePlacement::SpanLine { span: 2, end: 5 }
        .resolve_axis(1)
        .unwrap();

    assert_eq!(placement.start_line, 3);
    assert_eq!(placement.end_line, 5);
    assert_eq!(placement.span, 2);
}

#[test]
fn grid_placement_defaults_auto_auto_to_one_track_span() {
    let placement = LinePlacement::Auto.resolve_axis(4).unwrap();

    assert_eq!(placement.start_line, 4);
    assert_eq!(placement.end_line, 5);
    assert_eq!(placement.span, 1);
}

#[test]
fn grid_placement_extends_implicit_tracks_after_explicit_grid() {
    let placement = LinePlacement::Line(4).resolve_axis(1).unwrap();

    assert_eq!(placement.start_line, 4);
    assert_eq!(placement.end_line, 5);
    assert_eq!(placement.span, 1);
    assert_eq!(placement.implicit_after(3), 1);
}

#[test]
fn grid_item_placement_resolves_two_axes_to_area() {
    let placement = ItemPlacement {
        column: LinePlacement::LineSpan { start: 2, span: 2 },
        row: LinePlacement::SpanLine { span: 2, end: 4 },
    }
    .resolve(1, 1)
    .unwrap();

    assert_eq!(placement.column.start_line, 2);
    assert_eq!(placement.column.end_line, 4);
    assert_eq!(placement.row.start_line, 2);
    assert_eq!(placement.row.end_line, 4);
    assert_eq!(placement.area(), GridArea::new(2, 2, 2, 2));
}

fn named_columns(explicit_track_count: usize, line_names: Vec<Vec<&str>>) -> NamedGridLines {
    NamedGridLines::new(GridAxis::Column, explicit_track_count, line_names).unwrap()
}

#[test]
fn oracle_named_grid_lines_empty_initializes_all_explicit_lines() {
    let lines = NamedGridLines::empty(GridAxis::Column, 2);

    assert_eq!(lines.explicit_track_count, 2);
    assert!(lines.line_names(1).is_empty());
    assert!(lines.line_names(2).is_empty());
    assert!(lines.line_names(3).is_empty());
}

#[test]
fn oracle_named_grid_lines_return_names_by_one_based_line() {
    let lines = named_columns(2, vec![vec!["a"], vec!["b", "c"], vec![]]);

    assert_eq!(lines.line_names(1), vec!["a"]);
    assert_eq!(lines.line_names(2), vec!["b", "c"]);
    assert!(lines.line_names(3).is_empty());
    assert!(lines.line_names(0).is_empty());
}

#[test]
fn oracle_named_grid_lines_reject_reserved_names() {
    let auto_err =
        NamedGridLines::new(GridAxis::Column, 1, vec![vec!["auto"], vec![]]).unwrap_err();
    let span_err =
        NamedGridLines::new(GridAxis::Column, 1, vec![vec!["span"], vec![]]).unwrap_err();

    assert_eq!(
        auto_err,
        NamedGridError::ReservedLineName {
            name: "auto".to_owned(),
        }
    );
    assert_eq!(
        span_err,
        NamedGridError::ReservedLineName {
            name: "span".to_owned(),
        }
    );
}

#[test]
fn oracle_named_line_occurrence_shape_is_exported() {
    let occurrence = NamedLineOccurrence {
        line: 2,
        origin: LineNameOrigin::Explicit,
    };

    assert_eq!(occurrence.line, 2);
    assert_eq!(occurrence.origin, LineNameOrigin::Explicit);
}

#[test]
fn oracle_named_grid_lines_preserve_repeated_names_in_source_order() {
    let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);

    assert_eq!(lines.named_occurrences("a"), vec![1, 2, 3]);
    assert_eq!(lines.named_occurrences("b"), vec![2, 4]);
}

#[test]
fn oracle_named_grid_lines_reject_mismatched_line_count() {
    let err = support::oracle::grid::NamedGridLines::new(
        support::oracle::grid::GridAxis::Row,
        2,
        vec![vec!["a"], vec!["b"]],
    )
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::NamedGridError::LineNameCountMismatch {
            axis: support::oracle::grid::GridAxis::Row,
            explicit_track_count: 2,
            line_count: 2,
        }
    );
}

#[test]
fn oracle_named_fixed_repeat_expands_line_names_between_tracks() {
    let expanded = support::oracle::grid::expand_named_fixed_repeat(
        support::oracle::grid::GridAxis::Column,
        2,
        [
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::NamedTrackComponent::Track,
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["b".to_owned()]),
            support::oracle::grid::NamedTrackComponent::Track,
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["c".to_owned()]),
        ],
    )
    .unwrap();

    assert_eq!(expanded.explicit_track_count, 4);
    assert_eq!(expanded.named_occurrences("a"), vec![1, 3]);
    assert_eq!(expanded.named_occurrences("b"), vec![2, 4]);
    assert_eq!(expanded.named_occurrences("c"), vec![3, 5]);
}

#[test]
fn oracle_named_fixed_repeat_merges_adjacent_line_name_lists() {
    let expanded = support::oracle::grid::expand_named_fixed_repeat(
        support::oracle::grid::GridAxis::Column,
        2,
        [
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["start".to_owned()]),
            support::oracle::grid::NamedTrackComponent::Track,
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["end".to_owned()]),
            support::oracle::grid::NamedTrackComponent::LineNames(vec!["next".to_owned()]),
            support::oracle::grid::NamedTrackComponent::Track,
        ],
    )
    .unwrap();

    assert_eq!(expanded.explicit_track_count, 4);
    assert_eq!(expanded.line_names(2), vec!["end", "next"]);
    assert_eq!(expanded.line_names(3), vec!["start"]);
}

#[test]
fn oracle_named_fixed_repeat_rejects_zero_repeat() {
    assert_eq!(
        support::oracle::grid::expand_named_fixed_repeat(
            support::oracle::grid::GridAxis::Column,
            0,
            [support::oracle::grid::NamedTrackComponent::Track],
        )
        .unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroRepeat
    );
}

#[test]
fn oracle_named_fixed_repeat_rejects_reserved_line_names() {
    assert_eq!(
        support::oracle::grid::expand_named_fixed_repeat(
            support::oracle::grid::GridAxis::Column,
            1,
            [support::oracle::grid::NamedTrackComponent::LineNames(vec![
                "span".to_owned(),
            ])],
        )
        .unwrap_err(),
        support::oracle::grid::NamedGridError::ReservedLineName {
            name: "span".to_owned(),
        }
    );
}

#[test]
fn oracle_named_line_lookup_counts_positive_occurrences_from_start() {
    let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 2);
    assert_eq!(report.explicit_matches, vec![1, 2, 3]);
    assert!(report.implicit_lines_assumed_named.is_empty());
}

#[test]
fn oracle_named_line_lookup_counts_negative_occurrences_from_end() {
    let lines = named_columns(3, vec![vec!["a"], vec!["b", "a"], vec!["a"], vec!["b"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", -1).unwrap();

    assert_eq!(report.resolved_line, 3);
    assert_eq!(report.explicit_matches, vec![1, 2, 3]);
}

#[test]
fn oracle_named_line_lookup_extends_after_for_missing_positive_occurrence() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", 4).unwrap();

    assert_eq!(report.resolved_line, 5);
    assert_eq!(report.explicit_matches, vec![1, 3]);
    assert_eq!(report.implicit_lines_assumed_named, vec![4, 5]);
}

#[test]
fn oracle_named_line_lookup_extends_before_for_missing_negative_occurrence() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
    let report = support::oracle::grid::resolve_named_line(&lines, "a", -3).unwrap();

    assert_eq!(report.resolved_line, 0);
    assert_eq!(report.explicit_matches, vec![1, 3]);
    assert_eq!(report.implicit_lines_assumed_named, vec![0]);
}

#[test]
fn oracle_named_line_lookup_rejects_zero_occurrence() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

    assert_eq!(
        support::oracle::grid::resolve_named_line(&lines, "a", 0).unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroLine
    );
}

#[test]
fn oracle_named_line_lookup_rejects_reserved_custom_ident() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

    assert_eq!(
        support::oracle::grid::resolve_named_line(&lines, "auto", 1).unwrap_err(),
        support::oracle::grid::NamedGridError::ReservedLineName {
            name: "auto".to_owned(),
        }
    );
    assert_eq!(
        support::oracle::grid::resolve_named_line(&lines, "span", 1).unwrap_err(),
        support::oracle::grid::NamedGridError::ReservedLineName {
            name: "span".to_owned(),
        }
    );
}

#[test]
fn oracle_named_numeric_positive_line_passes_through() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);

    assert_eq!(
        support::oracle::grid::resolve_numeric_line(&lines, 3).unwrap(),
        3
    );
}

#[test]
fn oracle_named_numeric_negative_line_counts_from_explicit_end() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);

    assert_eq!(
        support::oracle::grid::resolve_numeric_line(&lines, -1).unwrap(),
        5
    );
    assert_eq!(
        support::oracle::grid::resolve_numeric_line(&lines, -2).unwrap(),
        4
    );
}

#[test]
fn oracle_named_numeric_zero_line_is_invalid() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);

    assert_eq!(
        support::oracle::grid::resolve_numeric_line(&lines, 0).unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroLine,
    );
}

#[test]
fn oracle_named_span_from_start_finds_nth_named_line_forward() {
    let lines = named_columns(
        5,
        vec![vec!["a"], vec![], vec!["a"], vec![], vec!["a"], vec![]],
    );

    let report = support::oracle::grid::resolve_named_span_from_start(&lines, 1, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 5);
}

#[test]
fn oracle_named_span_from_start_skips_explicit_end_line_for_implicit_names() {
    let lines = named_columns(4, vec![vec!["a"], vec![], vec!["a"], vec![], vec![]]);

    let report = support::oracle::grid::resolve_named_span_from_start(&lines, 1, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 6);
    assert_eq!(report.explicit_matches, vec![1, 3]);
    assert_eq!(report.implicit_lines_assumed_named, vec![6]);
}

#[test]
fn oracle_named_span_from_end_finds_nth_named_line_backward() {
    let lines = named_columns(
        5,
        vec![vec!["a"], vec![], vec!["a"], vec![], vec!["a"], vec![]],
    );

    let report = support::oracle::grid::resolve_named_span_from_end(&lines, 5, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 1);
}

#[test]
fn oracle_named_span_extends_implicitly_when_name_is_missing() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

    let report = support::oracle::grid::resolve_named_span_from_start(&lines, 3, "a", 2).unwrap();

    assert_eq!(report.resolved_line, 5);
    assert_eq!(report.implicit_lines_assumed_named, vec![4, 5]);
}

#[test]
fn oracle_named_span_extends_implicitly_backward_when_name_is_missing() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

    let report = support::oracle::grid::resolve_named_span_from_end(&lines, 1, "a", 2).unwrap();

    assert_eq!(report.resolved_line, -1);
    assert_eq!(report.explicit_matches, vec![1, 3]);
    assert_eq!(report.implicit_lines_assumed_named, vec![0, -1]);
}

#[test]
fn oracle_named_span_rejects_zero_count() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

    assert_eq!(
        support::oracle::grid::resolve_named_span_from_start(&lines, 1, "a", 0).unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroSpan
    );
    assert_eq!(
        support::oracle::grid::resolve_named_span_from_end(&lines, 3, "a", 0).unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroSpan
    );
}

#[test]
fn oracle_named_span_rejects_reserved_custom_ident() {
    let lines = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);

    assert_eq!(
        support::oracle::grid::resolve_named_span_from_start(&lines, 1, "auto", 1).unwrap_err(),
        support::oracle::grid::NamedGridError::ReservedLineName {
            name: "auto".to_owned(),
        }
    );
    assert_eq!(
        support::oracle::grid::resolve_named_span_from_start(&lines, 1, "span", 1).unwrap_err(),
        support::oracle::grid::NamedGridError::ReservedLineName {
            name: "span".to_owned(),
        }
    );
    assert_eq!(
        support::oracle::grid::resolve_named_span_from_end(&lines, 3, "auto", 1).unwrap_err(),
        support::oracle::grid::NamedGridError::ReservedLineName {
            name: "auto".to_owned(),
        }
    );
    assert_eq!(
        support::oracle::grid::resolve_named_span_from_end(&lines, 3, "span", 1).unwrap_err(),
        support::oracle::grid::NamedGridError::ReservedLineName {
            name: "span".to_owned(),
        }
    );
}

#[test]
fn oracle_named_axis_resolves_named_start_and_named_end() {
    let lines = named_columns(4, vec![vec!["a"], vec![], vec!["b"], vec![], vec!["b"]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_owned(),
                occurrence: 1,
            },
            end: support::oracle::grid::NamedGridLine::Named {
                name: "b".to_owned(),
                occurrence: 2,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 5);
    assert_eq!(report.resolved.span, 4);
}

#[test]
fn oracle_named_axis_resolves_line_to_named_span() {
    let lines = named_columns(4, vec![vec!["a"], vec![], vec!["a"], vec![], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(1),
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 2,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 6);
}

#[test]
fn oracle_named_axis_resolves_required_mixed_forms() {
    let lines = named_columns(
        5,
        vec![vec!["a"], vec![], vec!["b"], vec!["a"], vec![], vec!["b"]],
    );

    let named_to_span = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_owned(),
                occurrence: 1,
            },
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("b".to_owned()),
                count: 2,
            },
        },
        None,
    )
    .unwrap();
    assert_eq!(
        (
            named_to_span.resolved.start_line,
            named_to_span.resolved.end_line,
        ),
        (1, 6)
    );

    let span_to_number = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 1,
            },
            end: support::oracle::grid::NamedGridLine::Number(6),
        },
        None,
    )
    .unwrap();
    assert_eq!(
        (
            span_to_number.resolved.start_line,
            span_to_number.resolved.end_line,
        ),
        (4, 6)
    );

    let span_to_named = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 1,
            },
            end: support::oracle::grid::NamedGridLine::Named {
                name: "b".to_owned(),
                occurrence: 2,
            },
        },
        None,
    )
    .unwrap();
    assert_eq!(
        (
            span_to_named.resolved.start_line,
            span_to_named.resolved.end_line,
        ),
        (4, 6)
    );

    let auto_to_number = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Auto,
            end: support::oracle::grid::NamedGridLine::Number(4),
        },
        Some(2),
    )
    .unwrap();
    assert_eq!(
        (
            auto_to_number.resolved.start_line,
            auto_to_number.resolved.end_line,
        ),
        (3, 4)
    );

    let number_to_auto = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(2),
            end: support::oracle::grid::NamedGridLine::Auto,
        },
        Some(4),
    )
    .unwrap();
    assert_eq!(
        (
            number_to_auto.resolved.start_line,
            number_to_auto.resolved.end_line,
        ),
        (2, 3)
    );
}

#[test]
fn oracle_named_axis_drops_end_span_when_both_sides_are_spans() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 3);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Span {
                name: None,
                count: 1,
            },
            end: support::oracle::grid::NamedGridLine::Span {
                name: None,
                count: 1,
            },
        },
        Some(2),
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DroppedEndSpan)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 3);
}

#[test]
fn oracle_named_axis_records_ordered_span_span_normalizations() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 4,
            },
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("b".to_owned()),
                count: 2,
            },
        },
        Some(2),
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolutions,
        vec![
            support::oracle::grid::NamedPlacementConflictResolution::DroppedEndSpan,
            support::oracle::grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne,
        ]
    );
    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DroppedEndSpan)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 3);
}

#[test]
fn oracle_named_axis_swaps_reversed_resolved_lines() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(4),
            end: support::oracle::grid::NamedGridLine::Number(2),
        },
        None,
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::SwappedResolvedLines)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 4);
}

#[test]
fn oracle_named_axis_drops_equal_end_line_to_span_one() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(3),
            end: support::oracle::grid::NamedGridLine::Number(3),
        },
        None,
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DroppedEqualEndLine)
    );
    assert_eq!(report.resolved.start_line, 3);
    assert_eq!(report.resolved.end_line, 4);
}

#[test]
fn oracle_named_axis_clears_end_lookup_when_equal_line_drops_end() {
    let lines = named_columns(4, vec![vec![], vec![], vec!["mark"], vec![], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(3),
            end: support::oracle::grid::NamedGridLine::Named {
                name: "mark".to_owned(),
                occurrence: 1,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(
        report.normalized_end,
        support::oracle::grid::NamedGridLine::Span {
            name: None,
            count: 1,
        }
    );
    assert!(report.end_lookup.is_none());
    assert_eq!(report.resolved.start_line, 3);
    assert_eq!(report.resolved.end_line, 4);
}

#[test]
fn oracle_named_axis_defaults_lone_start_named_span_to_one() {
    let lines = named_columns(3, vec![vec!["a"], vec![], vec!["a"], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 4,
            },
            end: support::oracle::grid::NamedGridLine::Auto,
        },
        Some(2),
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 3);
}

#[test]
fn oracle_named_axis_defaults_lone_end_named_span_to_one() {
    let lines = named_columns(3, vec![vec!["a"], vec![], vec!["a"], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Auto,
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("a".to_owned()),
                count: 4,
            },
        },
        Some(2),
    )
    .unwrap();

    assert_eq!(
        report.conflict_resolution,
        Some(support::oracle::grid::NamedPlacementConflictResolution::DefaultedLoneNamedSpanToOne)
    );
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 3);
}

#[test]
fn oracle_named_axis_bare_ident_prefers_side_generated_line_name() {
    let lines = named_columns(
        3,
        vec![vec!["main-start"], vec![], vec![], vec!["main-end"]],
    );
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
            end: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
        },
        None,
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 4);
}

#[test]
fn oracle_named_axis_bare_ident_falls_back_to_raw_name_without_side_names() {
    let lines = named_columns(4, vec![vec![], vec!["foo"], vec![], vec!["foo"], vec![]]);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::BareIdent("foo".to_owned()),
            end: support::oracle::grid::NamedGridLine::Number(5),
        },
        None,
    )
    .unwrap();

    assert_eq!(report.start_lookup.as_ref().unwrap().name, "foo");
    assert_eq!(report.resolved.start_line, 2);
    assert_eq!(report.resolved.end_line, 5);
}

#[test]
fn oracle_template_areas_generate_row_and_column_line_names() {
    let areas = support::oracle::grid::TemplateAreas::new([
        vec!["head", "head"],
        vec!["nav", "main"],
        vec!["nav", "main"],
    ])
    .unwrap();

    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 2),
    )
    .unwrap();
    let rows = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Row,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 3),
    )
    .unwrap();

    assert_eq!(columns.line_names(1), vec!["head-start", "nav-start"]);
    assert_eq!(columns.line_names(2), vec!["nav-end", "main-start"]);
    assert_eq!(columns.line_names(3), vec!["head-end", "main-end"]);
    assert_eq!(rows.line_names(1), vec!["head-start"]);
    assert_eq!(
        rows.line_names(2),
        vec!["head-end", "nav-start", "main-start"]
    );
    assert_eq!(rows.line_names(4), vec!["nav-end", "main-end"]);
}

#[test]
fn oracle_template_areas_reject_non_rectangular_area() {
    let err =
        support::oracle::grid::TemplateAreas::new([vec!["a", "a"], vec!["a", "b"]]).unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::NamedGridError::AreaNotRectangular {
            area: "a".to_owned(),
        }
    );
}

#[test]
fn oracle_template_areas_reject_empty_matrix() {
    assert_eq!(
        support::oracle::grid::TemplateAreas::new(Vec::<Vec<&str>>::new()).unwrap_err(),
        support::oracle::grid::NamedGridError::EmptyTemplateAreas,
    );
}

#[test]
fn oracle_template_areas_reject_mismatched_row_lengths() {
    let err = support::oracle::grid::TemplateAreas::new([vec!["a", "a"], vec!["a"]]).unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::NamedGridError::TemplateAreaRowLengthMismatch {
            expected: 2,
            actual: 1,
            row: 2,
        }
    );
}

#[test]
fn oracle_template_areas_treat_dot_runs_as_null_cells() {
    let areas = support::oracle::grid::TemplateAreas::new([vec!["....", "main"]]).unwrap();

    assert!(!areas.contains_area("...."));
    assert!(areas.contains_area("main"));
}

#[test]
fn oracle_template_areas_expand_base_line_map_to_template_size() {
    let areas = support::oracle::grid::TemplateAreas::new([vec!["a", "a", "a"]]).unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 1),
    )
    .unwrap();

    assert_eq!(columns.explicit_track_count, 3);
    assert_eq!(columns.line_names(1), vec!["a-start"]);
    assert_eq!(columns.line_names(4), vec!["a-end"]);
}

#[test]
fn oracle_template_areas_preserve_larger_base_line_map() {
    let areas = support::oracle::grid::TemplateAreas::new([vec!["a"]]).unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 3),
    )
    .unwrap();

    assert_eq!(columns.explicit_track_count, 3);
    assert_eq!(columns.line_names(1), vec!["a-start"]);
    assert_eq!(columns.line_names(2), vec!["a-end"]);
}

#[test]
fn oracle_template_areas_preserve_explicit_names_before_generated_names() {
    let areas = support::oracle::grid::TemplateAreas::new([vec!["a"]]).unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        named_columns(1, vec![vec!["explicit"], vec![]]),
    )
    .unwrap();

    assert_eq!(columns.line_names(1), vec!["explicit", "a-start"]);
    assert_eq!(
        columns.line_names[0][1].origin,
        support::oracle::grid::LineNameOrigin::AreaGenerated
    );
}

#[test]
fn oracle_template_areas_generate_facts_for_both_axes() {
    let areas = support::oracle::grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
    let facts = support::oracle::grid::area_generated_facts(
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 2),
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 1),
    )
    .unwrap();

    assert_eq!(facts.columns.line_names(1), vec!["a-start"]);
    assert_eq!(facts.columns.line_names(3), vec!["a-end"]);
    assert_eq!(facts.rows.line_names(1), vec!["a-start"]);
    assert_eq!(facts.rows.line_names(2), vec!["a-end"]);
    assert_eq!(facts.areas.area_rectangle("a").unwrap().column_end, 3);
}

#[test]
fn oracle_template_areas_resolve_area_to_generated_named_lines() {
    let areas = support::oracle::grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
    let placement = support::oracle::grid::resolve_named_area(&areas, "a").unwrap();

    assert_eq!(
        placement.column.start,
        support::oracle::grid::NamedGridLine::Named {
            name: "a-start".to_owned(),
            occurrence: 1,
        }
    );
    assert_eq!(
        placement.row.end,
        support::oracle::grid::NamedGridLine::Named {
            name: "a-end".to_owned(),
            occurrence: 1,
        }
    );
}

#[test]
fn oracle_template_areas_reject_missing_area_resolution() {
    let areas = support::oracle::grid::TemplateAreas::new([vec!["a"]]).unwrap();

    assert_eq!(
        support::oracle::grid::resolve_named_area(&areas, "b").unwrap_err(),
        support::oracle::grid::NamedGridError::AreaNotFound {
            area: "b".to_owned(),
        }
    );
}

#[test]
fn oracle_named_grid_resolves_area_generated_names_to_grid_area() {
    let areas =
        support::oracle::grid::TemplateAreas::new([vec!["head", "head"], vec!["nav", "main"]])
            .unwrap();
    let columns = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Column,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 2),
    )
    .unwrap();
    let rows = support::oracle::grid::area_generated_lines(
        support::oracle::grid::GridAxis::Row,
        &areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 2),
    )
    .unwrap();

    assert_eq!(columns.named_occurrences("main-start"), vec![2]);
    assert_eq!(rows.named_occurrences("main-start"), vec![2]);

    let report =
        support::oracle::grid::resolve_named_grid_area_report(&columns, &rows, "main").unwrap();

    assert_eq!(
        report.area,
        support::oracle::grid::GridArea::new(2, 2, 1, 1)
    );
    assert_eq!(
        report.column.start_lookup.as_ref().unwrap().name,
        "main-start"
    );
    assert_eq!(report.column.end_lookup.as_ref().unwrap().name, "main-end");
    assert_eq!(report.row.start_lookup.as_ref().unwrap().name, "main-start");
    assert_eq!(report.row.end_lookup.as_ref().unwrap().name, "main-end");
    assert!(
        report
            .column
            .start_lookup
            .as_ref()
            .unwrap()
            .implicit_lines_assumed_named
            .is_empty()
    );
}

#[test]
fn oracle_axis_shorthand_repeats_omitted_custom_ident() {
    let expanded = support::oracle::grid::expand_axis_shorthand(
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
        None,
    );

    assert_eq!(
        expanded,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
            end: support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
        }
    );
}

#[test]
fn oracle_axis_shorthand_defaults_omitted_non_ident_to_auto() {
    let expanded = support::oracle::grid::expand_axis_shorthand(
        support::oracle::grid::NamedGridLine::Number(2),
        None,
    );

    assert_eq!(
        expanded,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(2),
            end: support::oracle::grid::NamedGridLine::Auto,
        }
    );
}

#[test]
fn oracle_grid_area_shorthand_repeats_single_custom_ident_to_all_sides() {
    let expanded = support::oracle::grid::expand_grid_area_shorthand(vec![
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned()),
    ])
    .unwrap();

    assert_eq!(
        expanded.row.start,
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned())
    );
    assert_eq!(
        expanded.row.end,
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned())
    );
    assert_eq!(
        expanded.column.start,
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned())
    );
    assert_eq!(
        expanded.column.end,
        support::oracle::grid::NamedGridLine::BareIdent("main".to_owned())
    );
}

#[test]
fn oracle_grid_area_shorthand_expands_two_and_four_values() {
    let two = support::oracle::grid::expand_grid_area_shorthand(vec![
        support::oracle::grid::NamedGridLine::BareIdent("row".to_owned()),
        support::oracle::grid::NamedGridLine::BareIdent("col".to_owned()),
    ])
    .unwrap();
    assert_eq!(
        two.row.end,
        support::oracle::grid::NamedGridLine::BareIdent("row".to_owned())
    );
    assert_eq!(
        two.column.end,
        support::oracle::grid::NamedGridLine::BareIdent("col".to_owned())
    );

    let four = support::oracle::grid::expand_grid_area_shorthand(vec![
        support::oracle::grid::NamedGridLine::Number(1),
        support::oracle::grid::NamedGridLine::Number(2),
        support::oracle::grid::NamedGridLine::Number(3),
        support::oracle::grid::NamedGridLine::Number(4),
    ])
    .unwrap();
    assert_eq!(
        four.row.start,
        support::oracle::grid::NamedGridLine::Number(1)
    );
    assert_eq!(
        four.column.start,
        support::oracle::grid::NamedGridLine::Number(2)
    );
    assert_eq!(
        four.row.end,
        support::oracle::grid::NamedGridLine::Number(3)
    );
    assert_eq!(
        four.column.end,
        support::oracle::grid::NamedGridLine::Number(4)
    );
}

#[test]
fn oracle_grid_area_shorthand_defaults_omitted_non_idents_to_auto() {
    let expanded = support::oracle::grid::expand_grid_area_shorthand(vec![
        support::oracle::grid::NamedGridLine::Number(2),
        support::oracle::grid::NamedGridLine::Number(3),
        support::oracle::grid::NamedGridLine::Number(4),
    ])
    .unwrap();

    assert_eq!(
        expanded.row.start,
        support::oracle::grid::NamedGridLine::Number(2)
    );
    assert_eq!(
        expanded.row.end,
        support::oracle::grid::NamedGridLine::Number(4)
    );
    assert_eq!(
        expanded.column.end,
        support::oracle::grid::NamedGridLine::Auto
    );
}

#[test]
fn oracle_named_grid_resolves_subgrid_named_span_into_parent_space() {
    let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["b"], vec!["c"]]);
    let subgrid = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![vec![], vec![], vec![], vec![]],
        None,
    )
    .unwrap();

    assert_eq!(subgrid.lines.line_names(1), vec!["b"]);
    assert_eq!(subgrid.lines.line_names(4), vec!["c"]);

    let report = support::oracle::grid::resolve_named_axis_placement(
        &subgrid.lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "b".to_owned(),
                occurrence: 1,
            },
            end: support::oracle::grid::NamedGridLine::Span {
                name: Some("c".to_owned()),
                count: 1,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.start_lookup.as_ref().unwrap().resolved_line, 1);
    assert_eq!(report.end_lookup.as_ref().unwrap().resolved_line, 4);
    assert_eq!(report.resolved.start_line, 1);
    assert_eq!(report.resolved.end_line, 4);
}

#[test]
fn oracle_named_axis_auto_auto_with_cursor_resolves_one_track_span() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);
    let report = support::oracle::grid::resolve_named_axis_placement(
        &lines,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Auto,
            end: support::oracle::grid::NamedGridLine::Auto,
        },
        Some(3),
    )
    .unwrap();

    assert_eq!(report.resolved.start_line, 3);
    assert_eq!(report.resolved.end_line, 4);
    assert_eq!(report.resolved.span, 1);
}

#[test]
fn oracle_named_axis_unresolved_auto_without_cursor_returns_error() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);

    assert_eq!(
        support::oracle::grid::resolve_named_axis_placement(
            &lines,
            support::oracle::grid::NamedAxisPlacement {
                start: support::oracle::grid::NamedGridLine::Auto,
                end: support::oracle::grid::NamedGridLine::Auto,
            },
            None,
        )
        .unwrap_err(),
        support::oracle::grid::NamedGridError::AutoWithoutCursor
    );
}

#[test]
fn oracle_named_axis_maps_line_before_first_error() {
    let lines =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4);

    assert_eq!(
        support::oracle::grid::resolve_named_axis_placement(
            &lines,
            support::oracle::grid::NamedAxisPlacement {
                start: support::oracle::grid::NamedGridLine::Number(-10),
                end: support::oracle::grid::NamedGridLine::Number(2),
            },
            None,
        )
        .unwrap_err(),
        support::oracle::grid::NamedGridError::LineBeforeFirst {
            axis: support::oracle::grid::GridAxis::Column,
            start_line: -4,
            end_line: 2,
        }
    );
}

#[test]
fn oracle_anonymous_span_offsets_from_known_edge() {
    assert_eq!(
        support::oracle::grid::resolve_anonymous_span_from_start(2, 3).unwrap(),
        5
    );
    assert_eq!(
        support::oracle::grid::resolve_anonymous_span_from_end(5, 3).unwrap(),
        2
    );
}

#[test]
fn oracle_anonymous_span_rejects_zero_count() {
    assert_eq!(
        support::oracle::grid::resolve_anonymous_span_from_start(2, 0).unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroSpan
    );
    assert_eq!(
        support::oracle::grid::resolve_anonymous_span_from_end(5, 0).unwrap_err(),
        support::oracle::grid::NamedGridError::ZeroSpan
    );
}

#[test]
fn grid_track_report_initializes_fixed_percent_and_flex_tracks() {
    let report = TrackSizingSlice::definite_columns(400.0, 10.0)
        .track(GridTrack::fixed(80.0))
        .track(GridTrack::percent(0.25))
        .track(GridTrack::flex(1.0))
        .solve();

    assert_eq!(
        report.initialized.tracks,
        vec![
            TrackSize::new(80.0, GrowthLimit::Definite(80.0)),
            TrackSize::new(100.0, GrowthLimit::Definite(100.0)),
            TrackSize::new(0.0, GrowthLimit::Infinite),
        ]
    );
    assert_eq!(report.after_intrinsic_minimums, report.initialized);
    assert_eq!(report.after_content_based_minimums, report.initialized);
    assert_eq!(report.after_spanning_items, report.initialized);
    assert_eq!(report.after_maximize_tracks, report.initialized);
    assert_eq!(report.flex_fraction, Some(200.0));
    assert_eq!(report.final_tracks[0].size, 80.0);
    assert_eq!(report.final_tracks[0].offset, 0.0);
    assert_eq!(report.final_tracks[1].size, 100.0);
    assert_eq!(report.final_tracks[1].offset, 90.0);
    assert_eq!(report.final_tracks[2].size, 200.0);
    assert_eq!(report.final_tracks[2].offset, 200.0);
}

#[test]
fn grid_track_report_initializes_auto_and_intrinsic_keywords() {
    let report = TrackSizingSlice::indefinite_columns(5.0)
        .track(GridTrack::auto())
        .track(GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent))
        .track(GridTrack::new(
            TrackMin::MaxContent,
            TrackMax::FitContent(120.0),
        ))
        .solve();

    assert_eq!(
        report.initialized.tracks,
        vec![
            TrackSize::new(0.0, GrowthLimit::Infinite),
            TrackSize::new(0.0, GrowthLimit::Infinite),
            TrackSize::new(0.0, GrowthLimit::Definite(120.0)),
        ]
    );
    assert_eq!(report.flex_fraction, None);
    assert_eq!(report.final_tracks[0].offset, 0.0);
    assert_eq!(report.final_tracks[1].offset, 5.0);
    assert_eq!(report.final_tracks[2].offset, 10.0);
}

#[test]
fn grid_track_report_initializes_minmax_growth_limits() {
    let report = TrackSizingSlice::definite_columns(200.0, 0.0)
        .track(GridTrack::new(TrackMin::Fixed(40.0), TrackMax::Fixed(90.0)))
        .track(GridTrack::new(TrackMin::Percent(0.25), TrackMax::Auto))
        .solve();

    assert_eq!(
        report.initialized.tracks,
        vec![
            TrackSize::new(40.0, GrowthLimit::Definite(90.0)),
            TrackSize::new(50.0, GrowthLimit::Infinite),
        ]
    );
    assert_eq!(
        report.after_maximize_tracks.tracks,
        vec![
            TrackSize::new(90.0, GrowthLimit::Definite(90.0)),
            TrackSize::new(50.0, GrowthLimit::Infinite),
        ]
    );
    assert_eq!(report.final_tracks[0].size, 90.0);
    assert_eq!(report.final_tracks[1].size, 50.0);
}

#[test]
fn grid_contributions_use_supplied_intrinsic_facts_and_margins() {
    let contributions = ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content: 40.0,
        max_content: 90.0,
        preferred: ContributionSize::Auto,
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Auto,
        margin_before: 5.0,
        margin_after: 7.0,
        automatic_minimum_applies: false,
    }
    .contributions();

    assert_eq!(
        contributions,
        ItemContributions {
            minimum: 12.0,
            min_content: 52.0,
            max_content: 102.0,
            limited_min_content: 52.0,
            limited_max_content: 102.0,
        }
    );
}

#[test]
fn grid_contributions_apply_min_max_and_preferred_limits() {
    let contributions = ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content: 40.0,
        max_content: 100.0,
        preferred: ContributionSize::Definite(65.0),
        min_size: ContributionSize::Definite(50.0),
        max_size: ContributionSize::Auto,
        margin_before: 2.0,
        margin_after: 3.0,
        automatic_minimum_applies: true,
    }
    .contributions();

    assert_eq!(
        contributions,
        ItemContributions {
            minimum: 55.0,
            min_content: 45.0,
            max_content: 105.0,
            limited_min_content: 55.0,
            limited_max_content: 70.0,
        }
    );
}

#[test]
fn grid_contributions_treat_explicit_infinite_max_as_unlimited() {
    let contributions = ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content: 20.0,
        max_content: 80.0,
        preferred: ContributionSize::Definite(50.0),
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Infinite,
        margin_before: 0.0,
        margin_after: 0.0,
        automatic_minimum_applies: true,
    }
    .contributions();

    assert_eq!(contributions.minimum, 20.0);
    assert_eq!(contributions.limited_max_content, 50.0);
}

#[test]
fn grid_intrinsic_single_span_grows_minimum_and_content_phases() {
    let report = TrackSizingSlice::indefinite_columns(0.0)
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 80.0,
            max_content: 120.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Definite(30.0),
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: false,
        })
        .solve();

    assert_eq!(
        report.after_intrinsic_minimums.tracks,
        vec![TrackSize::new(30.0, GrowthLimit::Infinite)]
    );
    assert_eq!(
        report.after_content_based_minimums.tracks,
        vec![TrackSize::new(80.0, GrowthLimit::Infinite)]
    );
    assert_eq!(report.final_tracks[0].size, 80.0);
}

#[test]
fn grid_intrinsic_single_span_clamps_to_growth_limit() {
    let report = TrackSizingSlice::indefinite_columns(0.0)
        .track(GridTrack::new(TrackMin::Auto, TrackMax::FitContent(40.0)))
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 90.0,
            max_content: 120.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    assert_eq!(
        report.after_content_based_minimums.tracks,
        vec![TrackSize::new(40.0, GrowthLimit::Definite(40.0))]
    );
    assert_eq!(report.final_tracks[0].size, 40.0);
}

#[test]
fn grid_intrinsic_spanning_items_distribute_deficits_across_auto_tracks() {
    let report = TrackSizingSlice::indefinite_columns(10.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 2, 1),
            min_content: 110.0,
            max_content: 140.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    assert_eq!(
        report.after_spanning_items.tracks,
        vec![
            TrackSize::new(50.0, GrowthLimit::Infinite),
            TrackSize::new(50.0, GrowthLimit::Infinite),
        ]
    );
    assert_eq!(report.final_tracks[0].offset, 0.0);
    assert_eq!(report.final_tracks[1].offset, 60.0);
}

#[test]
fn grid_intrinsic_row_spanning_items_use_row_axis() {
    let report = TrackSizingSlice::indefinite_rows(10.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 2),
            min_content: 110.0,
            max_content: 140.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    assert_eq!(
        report.after_spanning_items.tracks,
        vec![
            TrackSize::new(50.0, GrowthLimit::Infinite),
            TrackSize::new(50.0, GrowthLimit::Infinite),
        ]
    );
    assert_eq!(report.final_tracks[0].offset, 0.0);
    assert_eq!(report.final_tracks[1].offset, 60.0);
}

#[test]
fn grid_intrinsic_spanning_items_report_unsupported_mixed_track_categories() {
    let error = TrackSizingSlice::indefinite_columns(10.0)
        .track(GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent))
        .track(GridTrack::auto())
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 2, 1),
            min_content: 110.0,
            max_content: 140.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .try_solve()
        .unwrap_err();

    assert_eq!(
        error,
        TrackSizingError::UnsupportedSpanningTrackMix {
            axis: GridAxis::Column,
            start: 1,
            span: 2,
        }
    );
}

#[test]
fn grid_maximize_tracks_distributes_free_space_to_finite_growth_limits() {
    let report = TrackSizingSlice::definite_columns(180.0, 0.0)
        .track(GridTrack::new(
            TrackMin::Fixed(50.0),
            TrackMax::Fixed(100.0),
        ))
        .track(GridTrack::new(TrackMin::Fixed(50.0), TrackMax::Fixed(80.0)))
        .solve();

    assert_eq!(
        report.after_maximize_tracks.tracks,
        vec![
            TrackSize::new(100.0, GrowthLimit::Definite(100.0)),
            TrackSize::new(80.0, GrowthLimit::Definite(80.0)),
        ]
    );
    assert_eq!(report.final_tracks[0].size, 100.0);
    assert_eq!(report.final_tracks[1].size, 80.0);
}

#[test]
fn grid_flex_tracks_share_leftover_space_by_factor() {
    let report = TrackSizingSlice::definite_columns(300.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::flex(1.0))
        .track(GridTrack::flex(2.0))
        .solve();

    assert_eq!(report.flex_fraction, Some(230.0 / 3.0));
    assert_eq!(
        report.after_flexing.tracks,
        vec![
            TrackSize::new(50.0, GrowthLimit::Definite(50.0)),
            TrackSize::new(230.0 / 3.0, GrowthLimit::Infinite),
            TrackSize::new(460.0 / 3.0, GrowthLimit::Infinite),
        ]
    );
}

#[test]
fn grid_flex_tracks_recompute_fraction_after_oversized_base_tracks() {
    let report = TrackSizingSlice::definite_columns(300.0, 0.0)
        .track(GridTrack::flex(1.0))
        .track(GridTrack::flex(1.0))
        .item(ItemContributionFacts {
            area: GridArea::new(1, 1, 1, 1),
            min_content: 200.0,
            max_content: 200.0,
            preferred: ContributionSize::Auto,
            min_size: ContributionSize::Auto,
            max_size: ContributionSize::Auto,
            margin_before: 0.0,
            margin_after: 0.0,
            automatic_minimum_applies: true,
        })
        .solve();

    assert_eq!(report.flex_fraction, Some(100.0));
    assert_eq!(report.final_tracks[0].size, 200.0);
    assert_eq!(report.final_tracks[1].size, 100.0);
}

#[test]
fn grid_flex_tracks_report_zero_fraction_when_no_space_remains() {
    let report = TrackSizingSlice::definite_columns(80.0, 0.0)
        .track(GridTrack::fixed(100.0))
        .track(GridTrack::flex(1.0))
        .solve();

    assert_eq!(report.flex_fraction, Some(0.0));
    assert_eq!(report.final_tracks[0].size, 100.0);
    assert_eq!(report.final_tracks[1].size, 0.0);
}

#[test]
fn grid_stretch_grows_auto_tracks_after_flexing() {
    let report = TrackSizingSlice::definite_columns(120.0, 20.0)
        .track(GridTrack::auto())
        .track(GridTrack::auto())
        .stretch_auto_tracks()
        .solve();

    assert_eq!(report.after_maximize_tracks, report.after_spanning_items);
    assert_eq!(
        report.after_stretch.tracks,
        vec![
            TrackSize::new(50.0, GrowthLimit::Infinite),
            TrackSize::new(50.0, GrowthLimit::Infinite),
        ]
    );
    assert_eq!(report.final_tracks[1].offset, 70.0);
}

#[test]
fn grid_auto_placement_reports_placed_areas_cursor_and_implicit_growth() {
    let mut row = AutoPlacer::try_new(2, 1, Flow::Row).unwrap();
    assert_eq!(row.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
    assert_eq!(row.place(2, 1).unwrap(), GridArea::new(1, 2, 2, 1));

    let row_report = row.report();
    assert_eq!(
        row_report.areas,
        vec![GridArea::new(1, 1, 1, 1), GridArea::new(1, 2, 2, 1)]
    );
    assert_eq!(row_report.implicit_columns_after, 0);
    assert_eq!(row_report.implicit_rows_after, 1);
    assert_eq!(row_report.cursor.column, 1);
    assert_eq!(row_report.cursor.row, 3);

    let mut column = AutoPlacer::try_new(1, 2, Flow::Column).unwrap();
    assert_eq!(column.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
    assert_eq!(column.place(1, 2).unwrap(), GridArea::new(2, 1, 1, 2));

    let column_report = column.report();
    assert_eq!(
        column_report.areas,
        vec![GridArea::new(1, 1, 1, 1), GridArea::new(2, 1, 1, 2)]
    );
    assert_eq!(column_report.implicit_columns_after, 1);
    assert_eq!(column_report.implicit_rows_after, 0);
    assert_eq!(column_report.cursor.column, 3);
    assert_eq!(column_report.cursor.row, 1);
}

#[test]
fn grid_equal_share_intrinsic_tracks_distribute_unbounded_spanning_deficits() {
    let tracks = EqualShareIntrinsicTracks::new(3)
        .base(0, 20.0)
        .item(1, 1, 50.0)
        .item(0, 3, 100.0)
        .solve(10.0);

    assert_eq!(tracks.size(0), 30.0);
    assert_eq!(tracks.size(1), 60.0);
    assert_eq!(tracks.size(2), 10.0);
    assert_eq!(tracks.offset(0), 0.0);
    assert_eq!(tracks.offset(1), 40.0);
    assert_eq!(tracks.offset(2), 110.0);
}

#[test]
fn grid_auto_track_uses_stubbed_intrinsic_contribution_for_track_size() {
    let expected = EqualShareIntrinsicTracks::new(1)
        .item(0, 1, 80.0)
        .solve(0.0);
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackComponent::AUTO],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(80.0, 10.0),
                Size::new(80.0, 10.0),
            ))
            .run_mode(RunMode::ComputeSize),
        )
        .measure_when(
            2,
            OracleMeasurement::new(ComputeOutput::from_sizes(
                Size::new(80.0, 10.0),
                Size::new(80.0, 10.0),
            ))
            .run_mode(RunMode::PerformLayout)
            .known(Size::new(Some(80.0), Some(20.0))),
        );

    let output = surgeist::layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(expected.size(0), 20.0));
    assert_eq!(
        tree.inputs(2).last().unwrap().known,
        Size::new(Some(expected.size(0)), Some(20.0))
    );
    assert_eq!(
        tree.layout(2).unwrap().size,
        Size::new(expected.size(0), 10.0)
    );
}

#[test]
fn grid_alignment_distributes_free_space_after_track_sizing() {
    let start = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::Start);
    assert_eq!(start.offset(0), 0.0);
    assert_eq!(start.offset(1), 60.0);

    let end = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::End);
    assert_eq!(end.offset(0), 90.0);
    assert_eq!(end.offset(1), 150.0);

    let center = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::Center);
    assert_eq!(center.offset(0), 45.0);
    assert_eq!(center.offset(1), 105.0);

    let between = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::SpaceBetween);
    assert_eq!(between.offset(0), 0.0);
    assert_eq!(between.offset(1), 150.0);

    let around = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::SpaceAround);
    assert_eq!(around.offset(0), 22.5);
    assert_eq!(around.offset(1), 127.5);

    let evenly = align_tracks(200.0, vec![50.0, 50.0], 10.0, TrackAlignment::SpaceEvenly);
    assert!((evenly.offset(0) - 30.0).abs() < 0.000_001);
    assert!((evenly.offset(1) - 120.0).abs() < 0.000_001);
}

#[test]
fn grid_alignment_report_exposes_distribution_and_safe_fallback() {
    let center = align_tracks_report(
        200.0,
        vec![50.0, 50.0],
        10.0,
        TrackAlignment::Center,
        AlignmentSafety::Unsafe,
    );
    assert_eq!(center.leading_offset, 45.0);
    assert_eq!(center.distributed_gap, 10.0);
    assert_eq!(center.offsets, vec![45.0, 105.0]);
    assert!(!center.safe_fallback_used);

    let between = align_tracks_report(
        200.0,
        vec![50.0, 50.0],
        10.0,
        TrackAlignment::SpaceBetween,
        AlignmentSafety::Unsafe,
    );
    assert_eq!(between.leading_offset, 0.0);
    assert_eq!(between.distributed_gap, 100.0);
    assert_eq!(between.offsets, vec![0.0, 150.0]);

    let safe = align_tracks_report(
        80.0,
        vec![50.0, 50.0],
        10.0,
        TrackAlignment::Center,
        AlignmentSafety::Safe,
    );
    assert_eq!(safe.leading_offset, 0.0);
    assert_eq!(safe.distributed_gap, 10.0);
    assert_eq!(safe.offsets, vec![0.0, 60.0]);
    assert!(safe.safe_fallback_used);
}

#[test]
fn grid_scenario_composes_phase_reports_into_item_rects() {
    let mut placer = AutoPlacer::try_new(3, 1, Flow::Row).unwrap();
    assert_eq!(placer.place(1, 1).unwrap(), GridArea::new(1, 1, 1, 1));
    assert_eq!(placer.place(2, 1).unwrap(), GridArea::new(2, 1, 2, 1));
    let placement = placer.report();
    let columns = TrackSizingSlice::definite_columns(300.0, 10.0)
        .track(GridTrack::fixed(50.0))
        .track(GridTrack::flex(1.0))
        .track(GridTrack::flex(1.0))
        .solve();
    let rows = TrackSizingSlice::definite_rows(20.0, 0.0)
        .track(GridTrack::fixed(20.0))
        .solve();
    let column_alignment = align_tracks_report(
        300.0,
        columns
            .final_tracks
            .iter()
            .map(|track| track.size)
            .collect(),
        10.0,
        TrackAlignment::Start,
        AlignmentSafety::Unsafe,
    );
    let row_alignment = align_tracks_report(
        20.0,
        rows.final_tracks.iter().map(|track| track.size).collect(),
        0.0,
        TrackAlignment::Start,
        AlignmentSafety::Unsafe,
    );

    let scenario = compose_grid_scenario(placement, columns, rows, column_alignment, row_alignment);

    assert_eq!(
        scenario.item_rects,
        vec![
            GridItemRect::new(0.0, 0.0, 50.0, 20.0),
            GridItemRect::new(60.0, 0.0, 240.0, 20.0),
        ]
    );
}

#[test]
fn oracle_tree_stubs_child_measurements_and_records_layout_inputs() {
    let mut tree = OracleTree::new()
        .children(1, [2])
        .children(2, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(Dimension::px(120.0), Dimension::px(20.0)),
                grid_template_columns: vec![TrackComponent::px(120.0)],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(8.0), Length::ZERO),
                ..NodeInput::default()
            },
        )
        .style(2, NodeInput::default())
        .measure(
            2,
            ComputeOutput::from_sizes(Size::new(40.0, 10.0), Size::new(80.0, 10.0)),
        );

    let output = surgeist::layout::compute_grid(
        &mut tree,
        1,
        ComputeInput {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: Size::NONE,
            parent: Size::new(Some(300.0), Some(200.0)),
            available: Size::new(Available::MAX_CONTENT, Available::MAX_CONTENT),
        },
    );

    assert_eq!(output.size, Size::new(120.0, 20.0));
    assert_eq!(
        tree.inputs(2).last().unwrap().run_mode,
        RunMode::PerformLayout
    );
    assert_eq!(
        tree.inputs(2).last().unwrap().known,
        Size::new(Some(120.0), Some(20.0))
    );
    assert_eq!(tree.layout(2).unwrap().size, Size::new(40.0, 10.0));
}

#[test]
fn oracle_axis_mapping_preserves_parallel_horizontal_axes() {
    let report = support::oracle::grid::map_axis(support::oracle::grid::AxisMappingInput {
        queried_axis: GridAxis::Column,
        parent_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        child_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        parent_direction: support::oracle::grid::OracleDirection::Ltr,
        child_direction: support::oracle::grid::OracleDirection::Ltr,
        parent_flipped_in_resolved_axis: false,
        child_flipped_in_resolved_axis: false,
    })
    .unwrap();

    assert_eq!(report.parent_axis, GridAxis::Column);
    assert_eq!(report.child_axis, GridAxis::Column);
    assert!(!report.reversed);
}

#[test]
fn oracle_axis_mapping_rejects_vertical_mapping_without_explicit_support() {
    let err = support::oracle::grid::map_axis(support::oracle::grid::AxisMappingInput {
        queried_axis: GridAxis::Column,
        parent_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        child_writing_mode: support::oracle::grid::OracleWritingMode::VerticalRl,
        parent_direction: support::oracle::grid::OracleDirection::Ltr,
        child_direction: support::oracle::grid::OracleDirection::Ltr,
        parent_flipped_in_resolved_axis: false,
        child_flipped_in_resolved_axis: false,
    })
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::AxisMappingError::VerticalWritingModeUnsupported
    );
}

#[test]
fn oracle_axis_mapping_reports_reversed_when_flipped_states_differ() {
    let report = support::oracle::grid::map_axis(support::oracle::grid::AxisMappingInput {
        queried_axis: GridAxis::Row,
        parent_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        child_writing_mode: support::oracle::grid::OracleWritingMode::HorizontalTb,
        parent_direction: support::oracle::grid::OracleDirection::Rtl,
        child_direction: support::oracle::grid::OracleDirection::Ltr,
        parent_flipped_in_resolved_axis: true,
        child_flipped_in_resolved_axis: false,
    })
    .unwrap();

    assert_eq!(report.parent_axis, GridAxis::Row);
    assert_eq!(report.child_axis, GridAxis::Row);
    assert!(report.reversed);
}

#[test]
fn oracle_subgrid_name_repeat_expands_to_used_span() {
    let expanded = support::oracle::grid::expand_subgrid_name_list(
        support::oracle::grid::GridAxis::Column,
        4,
        vec![
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::Repeat {
                count: support::oracle::grid::SubgridNameRepeatCount::Number(2),
                line_name_sets: vec![vec!["b".to_owned()]],
            },
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["c".to_owned()]),
        ],
    )
    .unwrap();

    assert_eq!(
        expanded.local_line_names,
        vec![vec!["a"], vec!["b"], vec!["b"], vec!["c"], vec![],]
    );
}

#[test]
fn oracle_subgrid_auto_fill_name_repeat_pads_to_used_span() {
    let expanded = support::oracle::grid::expand_subgrid_name_list(
        support::oracle::grid::GridAxis::Column,
        3,
        vec![support::oracle::grid::SubgridNameComponent::Repeat {
            count: support::oracle::grid::SubgridNameRepeatCount::AutoFill,
            line_name_sets: vec![vec!["b".to_owned()]],
        }],
    )
    .unwrap();

    assert_eq!(
        expanded.local_line_names,
        vec![vec!["b"], vec!["b"], vec!["b"], vec!["b"]]
    );
}

#[test]
fn oracle_subgrid_auto_fill_name_repeat_reserves_trailing_fixed_names() {
    let expanded = support::oracle::grid::expand_subgrid_name_list(
        support::oracle::grid::GridAxis::Column,
        4,
        vec![
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["a".to_owned()]),
            support::oracle::grid::SubgridNameComponent::Repeat {
                count: support::oracle::grid::SubgridNameRepeatCount::AutoFill,
                line_name_sets: vec![vec!["b".to_owned()]],
            },
            support::oracle::grid::SubgridNameComponent::LineNames(vec!["c".to_owned()]),
        ],
    )
    .unwrap();

    assert_eq!(
        expanded.local_line_names,
        vec![vec!["a"], vec!["a"], vec!["a"], vec!["a"], vec!["c"]]
    );
}

#[test]
fn oracle_subgrid_name_repeat_rejects_multiple_auto_fill_repeats() {
    assert_eq!(
        support::oracle::grid::expand_subgrid_name_list(
            support::oracle::grid::GridAxis::Column,
            3,
            vec![
                support::oracle::grid::SubgridNameComponent::Repeat {
                    count: support::oracle::grid::SubgridNameRepeatCount::AutoFill,
                    line_name_sets: vec![vec!["a".to_owned()]],
                },
                support::oracle::grid::SubgridNameComponent::Repeat {
                    count: support::oracle::grid::SubgridNameRepeatCount::AutoFill,
                    line_name_sets: vec![vec!["b".to_owned()]],
                },
            ],
        )
        .unwrap_err(),
        support::oracle::grid::NamedGridError::MultipleAutoFillRepeats
    );
}

#[test]
fn oracle_subgrid_line_names_merge_parent_and_local_names() {
    let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["c"], vec!["d"]]);
    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 5),
        false,
        vec![
            vec!["local-start".to_owned()],
            vec![],
            vec!["middle".to_owned()],
            vec!["local-end".to_owned()],
        ],
        None,
    )
    .unwrap();

    assert_eq!(report.lines.line_names(1), vec!["b", "local-start"]);
    assert_eq!(report.lines.line_names(3), vec!["c", "middle"]);
    assert_eq!(report.lines.line_names(4), vec!["d", "local-end"]);
    assert_eq!(
        report.local_line_names.line_names[0][0].origin,
        support::oracle::grid::LineNameOrigin::LocalSubgrid
    );
}

#[test]
fn oracle_subgrid_line_names_reverse_parent_line_order_when_axis_is_reversed() {
    let parent = named_columns(4, vec![vec!["a"], vec!["b"], vec![], vec!["c"], vec!["d"]]);
    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 5),
        true,
        vec![vec![], vec![], vec![], vec![]],
        None,
    )
    .unwrap();

    assert_eq!(report.lines.line_names(1), vec!["d"]);
    assert_eq!(report.lines.line_names(2), vec!["c"]);
    assert_eq!(report.lines.line_names(4), vec!["b"]);
}

#[test]
fn oracle_subgrid_recomputes_area_generated_names_from_clipped_parent_areas() {
    let parent_areas =
        support::oracle::grid::TemplateAreas::new([vec!["a", "a", "a", "a"]]).unwrap();
    let parent_facts = support::oracle::grid::area_generated_facts(
        &parent_areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 4),
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 1),
    )
    .unwrap();
    let parent = parent_facts.columns.clone();

    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent,
        support::oracle::grid::TrackSpan::new(2, 4),
        false,
        vec![vec![], vec![], vec![]],
        Some(&parent_facts),
    )
    .unwrap();

    assert_eq!(
        report.clipped_area_sources["a"].parent_span,
        support::oracle::grid::TrackSpan::new(2, 4)
    );
    assert_eq!(report.lines.line_names(1), vec!["a-start"]);
    assert_eq!(report.lines.line_names(3), vec!["a-end"]);
}

#[test]
fn oracle_subgrid_reversed_area_generated_names_follow_parent_boundaries() {
    let parent_areas = support::oracle::grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
    let parent_facts = support::oracle::grid::area_generated_facts(
        &parent_areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 2),
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 1),
    )
    .unwrap();

    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent_facts.columns,
        support::oracle::grid::TrackSpan::new(1, 3),
        true,
        vec![vec![], vec![], vec![]],
        Some(&parent_facts),
    )
    .unwrap();

    assert_eq!(report.lines.line_names(1), vec!["a-end"]);
    assert_eq!(report.lines.line_names(3), vec!["a-start"]);
}

#[test]
fn oracle_subgrid_line_names_ignore_parent_area_generated_names_until_recomputed() {
    let parent_areas = support::oracle::grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
    let parent_facts = support::oracle::grid::area_generated_facts(
        &parent_areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 2),
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 1),
    )
    .unwrap();

    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent_facts.columns,
        support::oracle::grid::TrackSpan::new(1, 3),
        false,
        vec![vec![], vec![], vec![]],
        None,
    )
    .unwrap();

    assert!(report.lines.line_names(1).is_empty());
    assert!(report.lines.line_names(3).is_empty());
}

#[test]
fn oracle_subgrid_line_names_order_area_generated_before_local_names() {
    let parent_areas = support::oracle::grid::TemplateAreas::new([vec!["a", "a"]]).unwrap();
    let parent_facts = support::oracle::grid::area_generated_facts(
        &parent_areas,
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 2),
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Row, 1),
    )
    .unwrap();

    let report = support::oracle::grid::inherit_named_subgrid_lines(
        &parent_facts.columns,
        support::oracle::grid::TrackSpan::new(1, 3),
        false,
        vec![
            vec!["local-start".to_owned()],
            vec![],
            vec!["local-end".to_owned()],
        ],
        Some(&parent_facts),
    )
    .unwrap();

    assert_eq!(report.lines.line_names(1), vec!["a-start", "local-start"]);
    assert_eq!(report.lines.line_names(3), vec!["a-end", "local-end"]);
}

#[test]
fn oracle_subgrid_named_placement_clamps_to_subgrid_explicit_lines() {
    let subgrid = named_columns(2, vec![vec!["a"], vec![], vec!["a"]]);
    let report = support::oracle::grid::resolve_named_subgrid_axis_placement(
        &subgrid,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_owned(),
                occurrence: -3,
            },
            end: support::oracle::grid::NamedGridLine::Named {
                name: "a".to_owned(),
                occurrence: 4,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.unclamped_start_line, 0);
    assert_eq!(report.unclamped_end_line, 5);
    assert_eq!(report.clamped.resolved.start_line, 1);
    assert_eq!(report.clamped.resolved.end_line, 3);
}

#[test]
fn oracle_subgrid_named_placement_expands_collapsed_clamp_to_edge_track() {
    let subgrid =
        support::oracle::grid::NamedGridLines::empty(support::oracle::grid::GridAxis::Column, 1);
    let report = support::oracle::grid::resolve_named_subgrid_axis_placement(
        &subgrid,
        support::oracle::grid::NamedAxisPlacement {
            start: support::oracle::grid::NamedGridLine::Number(2),
            end: support::oracle::grid::NamedGridLine::Span {
                name: None,
                count: 3,
            },
        },
        None,
    )
    .unwrap();

    assert_eq!(report.unclamped_start_line, 2);
    assert_eq!(report.unclamped_end_line, 5);
    assert_eq!(report.clamped.resolved.start_line, 1);
    assert_eq!(report.clamped.resolved.end_line, 2);
}

#[test]
fn oracle_subgrid_eligibility_accepts_requested_axis_with_parent_grid() {
    let report = support::oracle::grid::subgrid_eligibility(
        support::oracle::grid::SubgridEligibilityInput {
            requested: true,
            has_parent_grid: true,
            independent_formatting_context: false,
            excluded_from_normal_layout: false,
            parent_is_lanes_in_resolved_axis: false,
        },
    );

    assert!(report.eligible);
    assert_eq!(report.reason, None);
}

#[test]
fn oracle_subgrid_eligibility_rejects_lanes_parent_in_resolved_axis() {
    let report = support::oracle::grid::subgrid_eligibility(
        support::oracle::grid::SubgridEligibilityInput {
            requested: true,
            has_parent_grid: true,
            independent_formatting_context: false,
            excluded_from_normal_layout: false,
            parent_is_lanes_in_resolved_axis: true,
        },
    );

    assert!(!report.eligible);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::SubgridIneligibleReason::ParentIsLanesInResolvedAxis)
    );
}

#[test]
fn oracle_subgrid_eligibility_reports_first_blocking_reason() {
    let report = support::oracle::grid::subgrid_eligibility(
        support::oracle::grid::SubgridEligibilityInput {
            requested: false,
            has_parent_grid: false,
            independent_formatting_context: true,
            excluded_from_normal_layout: true,
            parent_is_lanes_in_resolved_axis: true,
        },
    );

    assert!(!report.eligible);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::SubgridIneligibleReason::NotRequested)
    );
}

#[test]
fn oracle_subgrid_eligibility_reports_each_blocking_reason() {
    let cases = [
        (
            support::oracle::grid::SubgridEligibilityInput {
                requested: true,
                has_parent_grid: false,
                independent_formatting_context: false,
                excluded_from_normal_layout: false,
                parent_is_lanes_in_resolved_axis: false,
            },
            support::oracle::grid::SubgridIneligibleReason::NoParentGrid,
        ),
        (
            support::oracle::grid::SubgridEligibilityInput {
                requested: true,
                has_parent_grid: true,
                independent_formatting_context: true,
                excluded_from_normal_layout: false,
                parent_is_lanes_in_resolved_axis: false,
            },
            support::oracle::grid::SubgridIneligibleReason::IndependentFormattingContext,
        ),
        (
            support::oracle::grid::SubgridEligibilityInput {
                requested: true,
                has_parent_grid: true,
                independent_formatting_context: false,
                excluded_from_normal_layout: true,
                parent_is_lanes_in_resolved_axis: false,
            },
            support::oracle::grid::SubgridIneligibleReason::ExcludedFromNormalLayout,
        ),
    ];

    for (input, reason) in cases {
        assert_eq!(
            support::oracle::grid::subgrid_eligibility(input).reason,
            Some(reason)
        );
    }
}

#[test]
fn oracle_subgrid_copies_parent_tracks_for_span() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 60.0, 90.0],
            parent_span: support::oracle::grid::TrackSpan::new(2, 4),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
        },
    )
    .unwrap();

    assert_eq!(report.copied_parent_tracks, vec![60.0, 90.0]);
    assert_eq!(report.final_tracks, vec![60.0, 90.0]);
}

#[test]
fn oracle_subgrid_reverses_copied_tracks_before_mbp_removal() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 60.0, 90.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 4),
            reversed: true,
            start_mbp: 10.0,
            end_mbp: 20.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
        },
    )
    .unwrap();

    assert_eq!(report.after_reversal, vec![90.0, 60.0, 40.0]);
    assert_eq!(report.final_tracks, vec![80.0, 60.0, 20.0]);
}

#[test]
fn oracle_subgrid_resolves_normal_gap_to_parent_gap() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![50.0, 50.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(20.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(20.0),
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, 0.0);
    assert_eq!(report.final_tracks, vec![50.0, 50.0]);
}

#[test]
fn oracle_subgrid_baselines_slice_parent_groups_for_span() {
    let report = support::oracle::grid::inherit_subgrid_baselines(
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(2, 4),
            reversed: false,
            parent_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_major: vec![Some(4.0), Some(8.0), None, Some(6.0)],
            parent_minor: vec![None, Some(5.0), Some(7.0), None],
        },
    )
    .unwrap();

    assert_eq!(report.sliced_major, vec![Some(8.0), None]);
    assert_eq!(report.sliced_minor, vec![Some(5.0), Some(7.0)]);
    assert_eq!(report.after_reversal_major, vec![Some(8.0), None]);
    assert_eq!(report.after_reversal_minor, vec![Some(5.0), Some(7.0)]);
    assert_eq!(report.after_mbp_major, vec![Some(8.0), None]);
    assert_eq!(report.after_mbp_minor, vec![Some(5.0), Some(7.0)]);
    assert_eq!(report.final_major, vec![Some(8.0), None]);
    assert_eq!(report.final_minor, vec![Some(5.0), Some(7.0)]);
}

#[test]
fn oracle_subgrid_baselines_reverse_and_adjust_edges() {
    let report = support::oracle::grid::inherit_subgrid_baselines(
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: true,
            parent_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
            start_mbp: 3.0,
            end_mbp: 5.0,
            parent_major: vec![Some(10.0), Some(14.0)],
            parent_minor: vec![Some(4.0), Some(8.0)],
        },
    )
    .unwrap();

    assert_eq!(report.final_major.len(), 2);
    assert_eq!(report.final_minor.len(), 2);
    assert!(report.reversed);
    assert_eq!(report.start_mbp, 3.0);
    assert_eq!(report.end_mbp, 5.0);
    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.sliced_major, vec![Some(10.0), Some(14.0)]);
    assert_eq!(report.sliced_minor, vec![Some(4.0), Some(8.0)]);
    assert_eq!(report.after_reversal_major, vec![Some(14.0), Some(10.0)]);
    assert_eq!(report.after_reversal_minor, vec![Some(8.0), Some(4.0)]);
    assert_eq!(report.after_mbp_major, vec![Some(17.0), Some(10.0)]);
    assert_eq!(report.after_mbp_minor, vec![Some(8.0), Some(9.0)]);
    assert_eq!(report.final_major, vec![Some(12.0), Some(5.0)]);
    assert_eq!(report.final_minor, vec![Some(3.0), Some(4.0)]);
}

#[test]
fn oracle_subgrid_baselines_reject_invalid_spans_and_group_shapes() {
    let base = support::oracle::grid::SubgridBaselineInheritanceInput {
        parent_span: support::oracle::grid::TrackSpan::new(1, 3),
        reversed: false,
        parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
        subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
        start_mbp: 0.0,
        end_mbp: 0.0,
        parent_major: vec![Some(1.0), Some(2.0)],
        parent_minor: vec![Some(3.0), Some(4.0)],
    };

    let cases = [
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(0, 1),
            ..base.clone()
        },
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(2, 2),
            ..base.clone()
        },
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(1, 4),
            ..base.clone()
        },
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_minor: vec![Some(3.0)],
            ..base
        },
    ];

    for input in cases {
        assert!(support::oracle::grid::inherit_subgrid_baselines(input).is_err());
    }
}

#[test]
fn oracle_subgrid_baselines_preserve_none_through_mbp_and_gap_adjustment() {
    let report = support::oracle::grid::inherit_subgrid_baselines(
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
            start_mbp: 3.0,
            end_mbp: 5.0,
            parent_major: vec![None, Some(14.0)],
            parent_minor: vec![Some(12.0), None],
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.after_mbp_major, vec![None, Some(14.0)]);
    assert_eq!(report.after_mbp_minor, vec![Some(12.0), None]);
    assert_eq!(report.final_major, vec![None, Some(9.0)]);
    assert_eq!(report.final_minor, vec![Some(7.0), None]);
}

#[test]
fn oracle_subgrid_baselines_adjust_each_internal_gap_edge() {
    let report = support::oracle::grid::inherit_subgrid_baselines(
        support::oracle::grid::SubgridBaselineInheritanceInput {
            parent_span: support::oracle::grid::TrackSpan::new(1, 4),
            reversed: false,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_major: vec![Some(20.0), Some(30.0), Some(40.0)],
            parent_minor: vec![Some(10.0), Some(20.0), Some(30.0)],
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.final_major, vec![Some(15.0), Some(20.0), Some(35.0)]);
    assert_eq!(report.final_minor, vec![Some(5.0), Some(10.0), Some(25.0)]);
}

#[test]
fn oracle_subgrid_baselines_apply_signed_gap_differences() {
    let cases = [
        (
            support::oracle::grid::OracleGapReport::length(10.0),
            -5.0,
            vec![Some(18.0), Some(25.0)],
            vec![Some(10.0), Some(25.0)],
        ),
        (
            support::oracle::grid::OracleGapReport::length(20.0),
            0.0,
            vec![Some(13.0), Some(20.0)],
            vec![Some(5.0), Some(20.0)],
        ),
    ];

    for (subgrid_gap, gap_difference, final_major, final_minor) in cases {
        let report = support::oracle::grid::inherit_subgrid_baselines(
            support::oracle::grid::SubgridBaselineInheritanceInput {
                parent_span: support::oracle::grid::TrackSpan::new(1, 3),
                reversed: false,
                parent_gap: support::oracle::grid::OracleGapReport::length(20.0),
                subgrid_gap,
                start_mbp: 3.0,
                end_mbp: 5.0,
                parent_major: vec![Some(10.0), Some(20.0)],
                parent_minor: vec![Some(5.0), Some(15.0)],
            },
        )
        .unwrap();

        assert_eq!(report.gap_difference, gap_difference);
        assert_eq!(report.after_mbp_major, vec![Some(13.0), Some(20.0)]);
        assert_eq!(report.after_mbp_minor, vec![Some(5.0), Some(20.0)]);
        assert_eq!(report.final_major, final_major);
        assert_eq!(report.final_minor, final_minor);
    }
}

#[test]
fn oracle_subgrid_publishes_descendant_baseline_to_ancestor_track() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 4),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap();

    assert_eq!(report.ancestor_track, Some(2));
    assert_eq!(
        report.group,
        Some(support::oracle::grid::BaselineGroupKind::Major)
    );
    assert_eq!(report.baseline, Some(75.0));
}

#[test]
fn oracle_subgrid_publishes_reversed_descendant_baseline_to_ancestor_track() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: true,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Minor,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap();

    assert_eq!(report.ancestor_track, Some(4));
    assert_eq!(
        report.group,
        Some(support::oracle::grid::BaselineGroupKind::Minor)
    );
    assert_eq!(report.baseline, Some(75.0));
}

#[test]
fn oracle_subgrid_publishes_last_local_track_to_ancestor_track() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 3,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap();

    assert_eq!(report.ancestor_track, Some(4));
    assert_eq!(
        report.group,
        Some(support::oracle::grid::BaselineGroupKind::Major)
    );
    assert_eq!(report.baseline, Some(75.0));
}

#[test]
fn oracle_subgrid_publishes_reversed_last_local_track_to_ancestor_track() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: true,
            descendant_local_track: 3,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Minor,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap();

    assert_eq!(report.ancestor_track, Some(2));
    assert_eq!(
        report.group,
        Some(support::oracle::grid::BaselineGroupKind::Minor)
    );
    assert_eq!(report.baseline, Some(75.0));
}

#[test]
fn oracle_subgrid_does_not_publish_synthesized_cycle_fallback() {
    let report = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 4),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 1,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: true,
        },
    )
    .unwrap();

    assert!(!report.published);
    assert_eq!(report.ancestor_track, None);
    assert_eq!(report.group, None);
    assert_eq!(report.baseline, None);
}

#[test]
fn oracle_subgrid_publish_rejects_zero_local_track() {
    let error = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 0,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        support::oracle::grid::OracleGridError::SpanOutOfRange
    );
}

#[test]
fn oracle_subgrid_publish_rejects_local_track_beyond_span() {
    let error = support::oracle::grid::publish_subgrid_baseline(
        support::oracle::grid::SubgridBaselinePublicationInput {
            subgrid_span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
            subgrid_offset_in_parent: 40.0,
            reversed: false,
            descendant_local_track: 4,
            descendant_track_offset_in_subgrid: 20.0,
            descendant_group: support::oracle::grid::BaselineGroupKind::Major,
            descendant_baseline_in_track: 12.0,
            inherited_axis_offset: 3.0,
            synthesized_cycle_fallback: false,
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        support::oracle::grid::OracleGridError::SpanOutOfRange
    );
}

#[test]
fn oracle_subgrid_applies_gap_difference_to_internal_edges() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![50.0, 50.0, 50.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 4),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, 5.0);
    assert_eq!(report.final_tracks, vec![45.0, 40.0, 45.0]);
}

#[test]
fn oracle_subgrid_adds_negative_gap_difference_to_internal_edges() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![40.0, 40.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(20.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
        },
    )
    .unwrap();

    assert_eq!(report.gap_difference, -5.0);
    assert_eq!(report.final_tracks, vec![45.0, 45.0]);
}

#[test]
fn oracle_subgrid_mbp_removal_clamps_tracks_to_zero() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![5.0, 10.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 20.0,
            end_mbp: 20.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
        },
    )
    .unwrap();

    assert_eq!(report.final_tracks, vec![0.0, 0.0]);
}

#[test]
fn oracle_subgrid_mbp_removal_consumes_across_tracks() {
    let report = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![5.0, 10.0, 20.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 4),
            reversed: false,
            start_mbp: 12.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
        },
    )
    .unwrap();

    assert_eq!(report.start_mbp_removed, vec![0.0, 3.0, 20.0]);
    assert_eq!(report.end_mbp_removed, vec![0.0, 3.0, 20.0]);
    assert_eq!(report.final_tracks, vec![0.0, 3.0, 20.0]);
}

fn oracle_subgrid_leaf(
    id: &'static str,
    start: usize,
    end: usize,
) -> support::oracle::grid::SubgridChild {
    support::oracle::grid::SubgridChild::Leaf(support::oracle::grid::SubgridLeaf {
        id,
        span_in_parent: support::oracle::grid::TrackSpan::new(start, end),
        contribution: oracle_lane_facts(20.0, 40.0),
    })
}

fn oracle_subgrid_node(
    id: &'static str,
    start: usize,
    end: usize,
    children: Vec<support::oracle::grid::SubgridChild>,
) -> support::oracle::grid::SubgridChild {
    support::oracle::grid::SubgridChild::Subgrid(support::oracle::grid::SubgridNode {
        id,
        axis: support::oracle::grid::SubgridAxisKind::Inherited,
        reversed: false,
        span_in_parent: support::oracle::grid::TrackSpan::new(start, end),
        margins: support::oracle::grid::AxisEdges::default(),
        border: support::oracle::grid::AxisEdges::default(),
        padding: support::oracle::grid::AxisEdges::default(),
        parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
        subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
        children,
    })
}

#[test]
fn oracle_subgrid_traversal_collects_direct_leaf() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
        },
    )
    .unwrap();

    assert_eq!(report.leaves.len(), 1);
    assert_eq!(
        report.leaves[0].ancestor_span,
        support::oracle::grid::TrackSpan::new(1, 2)
    );
}

#[test]
fn oracle_subgrid_traversal_accumulates_edge_mbp_for_intrinsic_tracks() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
                    margins: support::oracle::grid::AxisEdges {
                        start: 3.0,
                        end: 4.0,
                    },
                    border: support::oracle::grid::AxisEdges {
                        start: 5.0,
                        end: 6.0,
                    },
                    padding: support::oracle::grid::AxisEdges {
                        start: 7.0,
                        end: 8.0,
                    },
                    parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(10.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![15.0, 18.0]);
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![15.0, 18.0]
    );
}

#[test]
fn oracle_subgrid_traversal_swaps_edge_mbp_for_reversed_subgrid() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: true,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
                    margins: support::oracle::grid::AxisEdges {
                        start: 3.0,
                        end: 4.0,
                    },
                    border: support::oracle::grid::AxisEdges {
                        start: 5.0,
                        end: 6.0,
                    },
                    padding: support::oracle::grid::AxisEdges {
                        start: 7.0,
                        end: 8.0,
                    },
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![18.0, 15.0]);
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![18.0, 15.0]
    );
}

#[test]
fn oracle_subgrid_traversal_accumulates_interior_edge_mbp_by_track() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(2, 4),
                    margins: support::oracle::grid::AxisEdges {
                        start: 2.0,
                        end: 3.0,
                    },
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 2.0, 3.0, 0.0]);
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![0.0, 2.0, 3.0, 0.0]
    );
}

#[test]
fn oracle_subgrid_traversal_translates_leaf_span_through_child_subgrid() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![oracle_subgrid_node(
                "sub",
                2,
                4,
                vec![oracle_subgrid_leaf("leaf", 2, 3)],
            )],
        },
    )
    .unwrap();

    assert_eq!(
        report.leaves[0].ancestor_span,
        support::oracle::grid::TrackSpan::new(3, 4)
    );
}

#[test]
fn oracle_subgrid_traversal_translates_reversed_leaf_span_from_end_edge() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: true,
                    span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
                    margins: support::oracle::grid::AxisEdges::default(),
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(
        report.leaves[0].ancestor_span,
        support::oracle::grid::TrackSpan::new(4, 5)
    );
}

#[test]
fn oracle_subgrid_traversal_preserves_reversed_orientation_through_nested_subgrid() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "outer",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: true,
                    span_in_parent: support::oracle::grid::TrackSpan::new(2, 6),
                    margins: support::oracle::grid::AxisEdges::default(),
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: vec![support::oracle::grid::SubgridChild::Subgrid(
                        support::oracle::grid::SubgridNode {
                            id: "inner",
                            axis: support::oracle::grid::SubgridAxisKind::Inherited,
                            reversed: false,
                            span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
                            margins: support::oracle::grid::AxisEdges::default(),
                            border: support::oracle::grid::AxisEdges::default(),
                            padding: support::oracle::grid::AxisEdges::default(),
                            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                            children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                        },
                    )],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(
        report.leaves[0].ancestor_span,
        support::oracle::grid::TrackSpan::new(5, 6)
    );
}

#[test]
fn oracle_subgrid_traversal_accumulates_gap_differences() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
                    margins: support::oracle::grid::AxisEdges::default(),
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
                    children: vec![oracle_subgrid_leaf("leaf", 2, 3)],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(
        report.leaves[0].accumulated_gap_adjustment,
        vec![5.0, 5.0, 0.0]
    );
}

#[test]
fn oracle_subgrid_traversal_skips_edge_mbp_for_non_intrinsic_tracks() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![false, false],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
                    margins: support::oracle::grid::AxisEdges {
                        start: 10.0,
                        end: 10.0,
                    },
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: Vec::new(),
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0]);
}

#[test]
fn oracle_subgrid_traversal_requires_intrinsic_facts_for_edge_mbp() {
    let err = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 2),
                    margins: support::oracle::grid::AxisEdges {
                        start: 1.0,
                        end: 1.0,
                    },
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: Vec::new(),
                },
            )],
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::OracleGridError::MissingIntrinsicMinTrackFacts
    );
}

#[test]
fn oracle_subgrid_traversal_rejects_standalone_axis() {
    let err = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "root",
                    axis: support::oracle::grid::SubgridAxisKind::Standalone,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 2),
                    margins: support::oracle::grid::AxisEdges::default(),
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: Vec::new(),
                },
            )],
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::OracleGridError::StandaloneSubgridTraversalUnsupported
    );
}

#[test]
fn oracle_subgrid_traversal_rejects_invalid_leaf_span() {
    let err = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true],
            root_children: vec![oracle_subgrid_leaf("bad", 2, 2)],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}

#[test]
fn oracle_subgrid_traversal_supports_mixed_root_children() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![
                oracle_subgrid_leaf("direct", 1, 2),
                oracle_subgrid_node("sub", 2, 4, vec![oracle_subgrid_leaf("nested", 1, 2)]),
            ],
        },
    )
    .unwrap();

    assert_eq!(
        report
            .leaves
            .iter()
            .map(|leaf| (leaf.id, leaf.ancestor_span))
            .collect::<Vec<_>>(),
        vec![
            ("direct", support::oracle::grid::TrackSpan::new(1, 2)),
            ("nested", support::oracle::grid::TrackSpan::new(2, 3)),
        ]
    );
}

#[test]
fn oracle_subgrid_traversal_accumulates_nested_edge_adjustments() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "outer",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 4),
                    margins: support::oracle::grid::AxisEdges {
                        start: 2.0,
                        end: 4.0,
                    },
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: vec![support::oracle::grid::SubgridChild::Subgrid(
                        support::oracle::grid::SubgridNode {
                            id: "inner",
                            axis: support::oracle::grid::SubgridAxisKind::Inherited,
                            reversed: false,
                            span_in_parent: support::oracle::grid::TrackSpan::new(2, 3),
                            margins: support::oracle::grid::AxisEdges {
                                start: 3.0,
                                end: 5.0,
                            },
                            border: support::oracle::grid::AxisEdges::default(),
                            padding: support::oracle::grid::AxisEdges::default(),
                            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                            children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                        },
                    )],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![2.0, 8.0, 4.0]);
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![2.0, 8.0, 4.0]
    );
}

#[test]
fn oracle_subgrid_traversal_translates_nested_edge_adjustments_to_ancestor_tracks() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true, true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "outer",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(2, 5),
                    margins: support::oracle::grid::AxisEdges::default(),
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                    children: vec![support::oracle::grid::SubgridChild::Subgrid(
                        support::oracle::grid::SubgridNode {
                            id: "inner",
                            axis: support::oracle::grid::SubgridAxisKind::Inherited,
                            reversed: false,
                            span_in_parent: support::oracle::grid::TrackSpan::new(2, 3),
                            margins: support::oracle::grid::AxisEdges {
                                start: 3.0,
                                end: 5.0,
                            },
                            border: support::oracle::grid::AxisEdges::default(),
                            padding: support::oracle::grid::AxisEdges::default(),
                            parent_gap: support::oracle::grid::OracleGapReport::length(0.0),
                            subgrid_gap: support::oracle::grid::OracleGapReport::length(0.0),
                            children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                        },
                    )],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.edge_lower_bounds, vec![0.0, 0.0, 8.0, 0.0]);
    assert_eq!(
        report.leaves[0].accumulated_edge_adjustment,
        vec![0.0, 0.0, 8.0, 0.0]
    );
}

#[test]
fn oracle_subgrid_traversal_applies_full_span_internal_gap() {
    let report = support::oracle::grid::traverse_subgrid_intrinsic(
        support::oracle::grid::SubgridTraversalInput {
            ancestor_track_intrinsic_min_eligibility: vec![true, true],
            root_children: vec![support::oracle::grid::SubgridChild::Subgrid(
                support::oracle::grid::SubgridNode {
                    id: "sub",
                    axis: support::oracle::grid::SubgridAxisKind::Inherited,
                    reversed: false,
                    span_in_parent: support::oracle::grid::TrackSpan::new(1, 3),
                    margins: support::oracle::grid::AxisEdges::default(),
                    border: support::oracle::grid::AxisEdges::default(),
                    padding: support::oracle::grid::AxisEdges::default(),
                    parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
                    subgrid_gap: support::oracle::grid::OracleGapReport::length(20.0),
                    children: vec![oracle_subgrid_leaf("leaf", 1, 2)],
                },
            )],
        },
    )
    .unwrap();

    assert_eq!(report.leaves[0].accumulated_gap_adjustment, vec![5.0, 5.0]);
}

#[test]
fn oracle_grid_lanes_disables_row_axis_item_baseline_offsets() {
    let report = support::oracle::grid::grid_lanes_baseline_policy(
        support::oracle::grid::GridLanesBaselineInput {
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            queried_axis: support::oracle::grid::GridAxis::Row,
            requested_alignment: support::oracle::grid::BaselineAlignment::First,
            has_items: true,
        },
    );

    assert!(!report.applies_item_offsets);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::GridLanesBaselineReason::WebKitMasonryFallback)
    );
}

#[test]
fn oracle_grid_lanes_disables_column_axis_item_baseline_offsets() {
    let report = support::oracle::grid::grid_lanes_baseline_policy(
        support::oracle::grid::GridLanesBaselineInput {
            auto_flow: support::oracle::grid::LaneAutoFlow::Column,
            queried_axis: support::oracle::grid::GridAxis::Column,
            requested_alignment: support::oracle::grid::BaselineAlignment::Last,
            has_items: true,
        },
    );

    assert!(!report.applies_item_offsets);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::GridLanesBaselineReason::WebKitMasonryFallback)
    );
}

#[test]
fn oracle_grid_lanes_disables_item_baseline_offsets_for_all_axis_combinations() {
    let cases = [
        (
            support::oracle::grid::LaneAutoFlow::Row,
            support::oracle::grid::GridAxis::Row,
        ),
        (
            support::oracle::grid::LaneAutoFlow::Row,
            support::oracle::grid::GridAxis::Column,
        ),
        (
            support::oracle::grid::LaneAutoFlow::Column,
            support::oracle::grid::GridAxis::Row,
        ),
        (
            support::oracle::grid::LaneAutoFlow::Column,
            support::oracle::grid::GridAxis::Column,
        ),
    ];

    for (auto_flow, queried_axis) in cases {
        let report = support::oracle::grid::grid_lanes_baseline_policy(
            support::oracle::grid::GridLanesBaselineInput {
                auto_flow,
                queried_axis,
                requested_alignment: support::oracle::grid::BaselineAlignment::First,
                has_items: true,
            },
        );

        assert!(!report.applies_item_offsets);
        assert_eq!(
            report.reason,
            Some(support::oracle::grid::GridLanesBaselineReason::WebKitMasonryFallback)
        );
    }
}

#[test]
fn oracle_grid_lanes_can_synthesize_container_baselines_from_geometry() {
    let report = support::oracle::grid::grid_lanes_container_baselines(vec![
        support::oracle::grid::ContainerBaselineFallbackItem {
            id: "a",
            area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
            block_offset: 0.0,
            first_baseline: 20.0,
            last_baseline: 0.0,
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            id: "b",
            area: support::oracle::grid::GridArea::new(2, 1, 1, 1),
            block_offset: 30.0,
            first_baseline: 30.0,
            last_baseline: 0.0,
        },
    ]);

    assert_eq!(report.first, Some(20.0));
    assert_eq!(report.last, Some(30.0));
}

#[test]
fn oracle_grid_lanes_container_baselines_use_final_geometry_offsets() {
    let report = support::oracle::grid::grid_lanes_container_baselines(vec![
        support::oracle::grid::ContainerBaselineFallbackItem {
            id: "first",
            area: support::oracle::grid::GridArea::new(1, 1, 1, 1),
            block_offset: 12.0,
            first_baseline: 7.0,
            last_baseline: 0.0,
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            id: "middle",
            area: support::oracle::grid::GridArea::new(2, 1, 1, 1),
            block_offset: 30.0,
            first_baseline: 5.0,
            last_baseline: 8.0,
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            id: "last",
            area: support::oracle::grid::GridArea::new(3, 1, 1, 1),
            block_offset: 44.0,
            first_baseline: 3.0,
            last_baseline: 11.0,
        },
    ]);

    assert_eq!(report.first, Some(19.0));
    assert_eq!(report.last, Some(55.0));
}

#[test]
fn oracle_grid_lanes_container_baselines_last_uses_spanned_end_edge() {
    let report = support::oracle::grid::grid_lanes_container_baselines(vec![
        support::oracle::grid::ContainerBaselineFallbackItem {
            id: "starts-later",
            area: support::oracle::grid::GridArea::new(1, 2, 1, 1),
            block_offset: 40.0,
            first_baseline: 8.0,
            last_baseline: 14.0,
        },
        support::oracle::grid::ContainerBaselineFallbackItem {
            id: "spans-to-last-row",
            area: support::oracle::grid::GridArea::new(2, 1, 1, 3),
            block_offset: 5.0,
            first_baseline: 3.0,
            last_baseline: 91.0,
        },
    ]);

    assert_eq!(report.first, Some(8.0));
    assert_eq!(report.last, Some(96.0));
}

#[test]
fn oracle_grid_lanes_container_baselines_return_none_for_empty_input() {
    let report = support::oracle::grid::grid_lanes_container_baselines(Vec::new());

    assert_eq!(report.first, None);
    assert_eq!(report.last, None);
}

#[test]
fn oracle_grid_lanes_baseline_policy_reports_no_items() {
    let report = support::oracle::grid::grid_lanes_baseline_policy(
        support::oracle::grid::GridLanesBaselineInput {
            auto_flow: support::oracle::grid::LaneAutoFlow::Row,
            queried_axis: support::oracle::grid::GridAxis::Row,
            requested_alignment: support::oracle::grid::BaselineAlignment::First,
            has_items: false,
        },
    );

    assert!(!report.applies_item_offsets);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::GridLanesBaselineReason::NoItems)
    );
}

#[test]
fn oracle_grid_lanes_baseline_policy_reports_no_baseline_alignment_requested() {
    let report = support::oracle::grid::grid_lanes_baseline_policy(
        support::oracle::grid::GridLanesBaselineInput {
            auto_flow: support::oracle::grid::LaneAutoFlow::Column,
            queried_axis: support::oracle::grid::GridAxis::Column,
            requested_alignment: support::oracle::grid::BaselineAlignment::None,
            has_items: true,
        },
    );

    assert!(!report.applies_item_offsets);
    assert_eq!(
        report.reason,
        Some(support::oracle::grid::GridLanesBaselineReason::NoBaselineAlignmentRequested)
    );
}

#[test]
fn oracle_lanes_row_auto_flow_makes_rows_the_lane_axis() {
    assert_eq!(
        support::oracle::grid::lane_axis(support::oracle::grid::LaneAutoFlow::Row),
        GridAxis::Row
    );
    assert_eq!(
        support::oracle::grid::grid_axis_for_lanes(support::oracle::grid::LaneAutoFlow::Row),
        GridAxis::Column
    );
}

#[test]
fn oracle_lanes_place_definite_and_indefinite_items_with_fixed_tolerance() {
    let report = support::oracle::grid::place_lanes(support::oracle::grid::LanePlacementInput {
        grid_axis_tracks: 3,
        auto_flow: support::oracle::grid::LaneAutoFlow::Row,
        lane_gap: 10.0,
        tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
        tolerance_basis: 0.0,
        items: vec![
            support::oracle::grid::LaneItemInput::definite("a", 1, 2, 40.0),
            support::oracle::grid::LaneItemInput::auto("b", 1, 20.0),
            support::oracle::grid::LaneItemInput::auto("c", 2, 30.0),
        ],
    })
    .unwrap();

    assert_eq!(report.item_offsets[0].offset, 0.0);
    assert_eq!(report.item_offsets[1].offset, 0.0);
    assert_eq!(report.item_offsets[2].offset, 50.0);
    assert_eq!(report.content_size, 80.0);
}

#[test]
fn oracle_lanes_finite_search_does_not_wrap_candidate_span() {
    let report = support::oracle::grid::place_lanes(support::oracle::grid::LanePlacementInput {
        grid_axis_tracks: 3,
        auto_flow: support::oracle::grid::LaneAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
        tolerance_basis: 0.0,
        items: vec![
            support::oracle::grid::LaneItemInput::auto("a", 2, 10.0),
            support::oracle::grid::LaneItemInput::auto("b", 2, 10.0),
        ],
    })
    .unwrap();

    assert!(
        report
            .item_offsets
            .iter()
            .all(|item| item.grid_axis_start + item.grid_axis_span <= 4)
    );
}

#[test]
fn oracle_lanes_reject_definite_item_that_exceeds_grid_axis() {
    let err = support::oracle::grid::place_lanes(support::oracle::grid::LanePlacementInput {
        grid_axis_tracks: 3,
        auto_flow: support::oracle::grid::LaneAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
        tolerance_basis: 0.0,
        items: vec![support::oracle::grid::LaneItemInput::definite(
            "a", 3, 2, 10.0,
        )],
    })
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}

#[test]
fn oracle_lanes_infinite_tolerance_uses_round_robin_cursor() {
    let report = support::oracle::grid::place_lanes(support::oracle::grid::LanePlacementInput {
        grid_axis_tracks: 2,
        auto_flow: support::oracle::grid::LaneAutoFlow::Column,
        lane_gap: 0.0,
        tolerance: support::oracle::grid::LaneFlowTolerance::Infinite,
        tolerance_basis: 0.0,
        items: vec![
            support::oracle::grid::LaneItemInput::auto("a", 1, 10.0),
            support::oracle::grid::LaneItemInput::auto("b", 1, 10.0),
            support::oracle::grid::LaneItemInput::auto("c", 1, 10.0),
        ],
    })
    .unwrap();

    assert_eq!(
        report
            .item_offsets
            .iter()
            .map(|item| item.grid_axis_start)
            .collect::<Vec<_>>(),
        vec![1, 2, 1]
    );
}

#[test]
fn oracle_lanes_percentage_tolerance_resolves_against_basis() {
    let report = support::oracle::grid::place_lanes(support::oracle::grid::LanePlacementInput {
        grid_axis_tracks: 2,
        auto_flow: support::oracle::grid::LaneAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: support::oracle::grid::LaneFlowTolerance::Percent(0.25),
        tolerance_basis: 40.0,
        items: vec![
            support::oracle::grid::LaneItemInput::definite("a", 1, 1, 10.0),
            support::oracle::grid::LaneItemInput::auto("b", 1, 10.0),
        ],
    })
    .unwrap();

    assert_eq!(report.item_offsets[1].grid_axis_start, 2);
}

#[test]
fn oracle_lanes_finite_tolerance_chooses_first_candidate_within_tolerance() {
    let report = support::oracle::grid::place_lanes(support::oracle::grid::LanePlacementInput {
        grid_axis_tracks: 3,
        auto_flow: support::oracle::grid::LaneAutoFlow::Row,
        lane_gap: 0.0,
        tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(10.0),
        tolerance_basis: 0.0,
        items: vec![
            support::oracle::grid::LaneItemInput::definite("a", 1, 1, 10.0),
            support::oracle::grid::LaneItemInput::definite("b", 2, 1, 20.0),
            support::oracle::grid::LaneItemInput::auto("c", 1, 10.0),
        ],
    })
    .unwrap();

    assert_eq!(report.item_offsets[2].grid_axis_start, 3);
}

fn oracle_lane_facts(min_content: f32, max_content: f32) -> ItemContributionFacts {
    ItemContributionFacts {
        area: GridArea::new(1, 1, 1, 1),
        min_content,
        max_content,
        preferred: ContributionSize::Auto,
        min_size: ContributionSize::Auto,
        max_size: ContributionSize::Infinite,
        margin_before: 0.0,
        margin_after: 0.0,
        automatic_minimum_applies: true,
    }
}

#[test]
fn oracle_lanes_intrinsic_keeps_definite_items_by_span() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::definite(
                    "a",
                    support::oracle::grid::TrackSpan::new(1, 2),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.definite_items.len(), 1);
    assert!(report.indefinite_groups.is_empty());
    assert_eq!(
        report.definite_items[0].contribution.area,
        GridArea::new(1, 1, 1, 1)
    );
}

#[test]
fn oracle_lanes_intrinsic_rewrites_definite_item_area_from_span() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::definite(
                    "a",
                    support::oracle::grid::TrackSpan::new(2, 3),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        },
    )
    .unwrap();

    assert_eq!(
        report.definite_items[0].contribution.area,
        GridArea::new(2, 1, 1, 1)
    );
}

#[test]
fn oracle_lanes_intrinsic_rewrites_row_axis_areas_from_spans() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Row,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::definite(
                    "a",
                    support::oracle::grid::TrackSpan::new(2, 3),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        },
    )
    .unwrap();

    assert_eq!(
        report.definite_items[0].contribution.area,
        GridArea::new(1, 2, 1, 1)
    );
}

#[test]
fn oracle_lanes_intrinsic_groups_indefinite_items_by_span_length() {
    let facts = oracle_lane_facts(20.0, 50.0);
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::indefinite(
                    "a",
                    oracle_lane_span(2),
                    facts,
                ),
                support::oracle::grid::LaneIntrinsicItem::indefinite(
                    "b",
                    oracle_lane_span(2),
                    ItemContributionFacts {
                        min_content: 30.0,
                        max_content: 60.0,
                        ..facts
                    },
                ),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.indefinite_groups.len(), 1);
    assert_eq!(report.indefinite_groups[0].span, 2);
    assert_eq!(report.indefinite_groups[0].max_min_content, 30.0);
    assert_eq!(report.indefinite_groups[0].max_max_content, 60.0);
    assert_eq!(report.indefinite_groups[0].max_min_size, 30.0);
    assert_eq!(report.converted_indefinite_items.len(), 2);
    assert_eq!(report.final_track_report.final_tracks.len(), 3);
}

#[test]
fn oracle_lanes_intrinsic_groups_indefinite_items_by_min_size() {
    let facts = ItemContributionFacts {
        automatic_minimum_applies: false,
        min_size: ContributionSize::Definite(12.0),
        ..oracle_lane_facts(100.0, 120.0)
    };
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(1),
                facts,
            )],
        },
    )
    .unwrap();

    assert_eq!(report.indefinite_groups[0].max_min_size, 12.0);
    assert_eq!(
        report.converted_indefinite_items[0]
            .contribution
            .min_content,
        100.0
    );
    assert_eq!(
        report.converted_indefinite_items[0].contribution.min_size,
        ContributionSize::Definite(12.0)
    );
    assert!(
        !report.converted_indefinite_items[0]
            .contribution
            .automatic_minimum_applies
    );
    assert_eq!(report.final_track_report.final_tracks[0].size, 12.0);
}

#[test]
fn oracle_lanes_intrinsic_uses_min_content_for_min_content_tracks() {
    let facts = ItemContributionFacts {
        automatic_minimum_applies: false,
        min_size: ContributionSize::Definite(12.0),
        ..oracle_lane_facts(100.0, 120.0)
    };
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::new(TrackMin::MinContent, TrackMax::MaxContent)],
            content_sized_tracks: vec![0],
            items: vec![support::oracle::grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(1),
                facts,
            )],
        },
    )
    .unwrap();

    assert_eq!(report.indefinite_groups[0].max_min_size, 12.0);
    assert_eq!(report.indefinite_groups[0].max_min_content, 100.0);
    assert_eq!(report.final_track_report.final_tracks[0].size, 100.0);
}

#[test]
fn oracle_lanes_intrinsic_converts_all_spans_that_overlap_content_tracks() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![
                GridTrack::fixed(20.0),
                GridTrack::auto(),
                GridTrack::fixed(20.0),
            ],
            content_sized_tracks: vec![1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(2),
                oracle_lane_facts(30.0, 60.0),
            )],
        },
    )
    .unwrap();

    assert_eq!(
        report
            .converted_indefinite_items
            .iter()
            .map(|item| item.span)
            .collect::<Vec<_>>(),
        vec![
            support::oracle::grid::TrackSpan::new(1, 3),
            support::oracle::grid::TrackSpan::new(2, 4),
        ]
    );
    assert_eq!(report.final_track_report.final_tracks[1].size, 0.0);
}

#[test]
fn oracle_lanes_intrinsic_distributes_converted_spanning_items() {
    let facts = ItemContributionFacts {
        automatic_minimum_applies: false,
        min_size: ContributionSize::Definite(70.0),
        ..oracle_lane_facts(90.0, 120.0)
    };
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(2),
                facts,
            )],
        },
    )
    .unwrap();

    assert_eq!(report.converted_indefinite_items.len(), 1);
    assert_eq!(
        report.converted_indefinite_items[0].span,
        support::oracle::grid::TrackSpan::new(1, 3)
    );
    assert_eq!(report.final_track_report.final_tracks[0].size, 30.0);
    assert_eq!(report.final_track_report.final_tracks[1].size, 30.0);
}

#[test]
fn oracle_lanes_intrinsic_splits_full_span_deficit_across_disjoint_content_tracks() {
    let facts = ItemContributionFacts {
        automatic_minimum_applies: false,
        min_size: ContributionSize::Definite(100.0),
        ..oracle_lane_facts(120.0, 160.0)
    };
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::fixed(20.0), GridTrack::auto()],
            content_sized_tracks: vec![0, 2],
            items: vec![support::oracle::grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(3),
                facts,
            )],
        },
    )
    .unwrap();

    assert_eq!(report.final_track_report.final_tracks[0].size, 30.0);
    assert_eq!(report.final_track_report.final_tracks[1].size, 20.0);
    assert_eq!(report.final_track_report.final_tracks[2].size, 30.0);
}

#[test]
fn oracle_lanes_intrinsic_clamps_oversized_indefinite_spans_before_reporting() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::indefinite(
                "a",
                oracle_lane_span(5),
                oracle_lane_facts(30.0, 60.0),
            )],
        },
    )
    .unwrap();

    assert_eq!(report.indefinite_groups[0].span, 2);
    assert_eq!(
        report
            .converted_indefinite_items
            .iter()
            .map(|item| item.span)
            .collect::<Vec<_>>(),
        vec![support::oracle::grid::TrackSpan::new(1, 3)]
    );
}

#[test]
fn oracle_lanes_intrinsic_skips_definite_items_outside_content_tracks_for_sizing() {
    let report = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(200.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![1],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::definite(
                    "a",
                    support::oracle::grid::TrackSpan::new(1, 2),
                    oracle_lane_facts(80.0, 120.0),
                )
                .expect("valid oracle lane item"),
            ],
        },
    )
    .unwrap();

    assert_eq!(report.definite_items.len(), 1);
    assert_eq!(report.final_track_report.final_tracks[0].size, 0.0);
    assert_eq!(report.final_track_report.final_tracks[1].size, 0.0);
}

#[test]
fn oracle_lanes_intrinsic_reports_nested_indefinite_subgrid_unsupported() {
    let err = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto(), GridTrack::auto(), GridTrack::auto()],
            content_sized_tracks: vec![0, 1, 2],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::nested_indefinite_subgrid(
                    "subgrid-child",
                    oracle_lane_span(2),
                    oracle_lane_facts(20.0, 50.0),
                ),
            ],
        },
    )
    .unwrap_err();

    assert_eq!(
        err,
        support::oracle::grid::OracleGridError::NestedGridLanesSubgridIndefiniteUnsupported
    );
}

#[test]
fn oracle_lanes_intrinsic_rejects_invalid_definite_span() {
    let err = support::oracle::grid::LaneIntrinsicItem::definite(
        "bad",
        support::oracle::grid::TrackSpan::new(2, 2),
        oracle_lane_facts(20.0, 50.0),
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}

#[test]
fn oracle_lanes_intrinsic_rejects_definite_span_outside_tracks() {
    let err = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto()],
            content_sized_tracks: vec![0],
            items: vec![
                support::oracle::grid::LaneIntrinsicItem::definite(
                    "bad",
                    support::oracle::grid::TrackSpan::new(2, 3),
                    oracle_lane_facts(20.0, 50.0),
                )
                .expect("valid oracle lane item"),
            ],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}

#[test]
fn oracle_lanes_intrinsic_rejects_invalid_content_sized_track() {
    let err = support::oracle::grid::lane_intrinsic_sizing(
        support::oracle::grid::LaneIntrinsicSizingInput {
            axis: GridAxis::Column,
            available: Some(300.0),
            gap: 10.0,
            tracks: vec![GridTrack::auto()],
            content_sized_tracks: vec![1],
            items: vec![support::oracle::grid::LaneIntrinsicItem::indefinite(
                "bad",
                oracle_lane_span(1),
                oracle_lane_facts(20.0, 50.0),
            )],
        },
    )
    .unwrap_err();

    assert_eq!(err, support::oracle::grid::OracleGridError::SpanOutOfRange);
}

#[test]
fn oracle_scenario_composes_subgrid_rect_from_explicit_tracks_and_offsets() {
    let report = support::oracle::grid::compose_subgrid_item_rect(
        support::oracle::grid::SubgridItemRectInput {
            inherited_axis: GridAxis::Column,
            inherited_axis_offset: 20.0,
            standalone_axis_offset: 5.0,
            inherited_axis_size: 80.0,
            standalone_axis_size: 30.0,
            container_mbp_offset: support::oracle::grid::AxisEdges {
                start: 3.0,
                end: 0.0,
            },
            item_inline_offset: 7.0,
            item_block_offset: 11.0,
        },
    );

    assert_eq!(report.inherited_axis_offset, 30.0);
    assert_eq!(report.standalone_axis_offset, 16.0);
    assert_eq!(report.rect, GridItemRect::new(30.0, 16.0, 80.0, 30.0));
}

#[test]
fn oracle_scenario_composes_lane_rect_from_lane_offset_and_grid_axis_area() {
    let rect =
        support::oracle::grid::compose_lane_item_rect(support::oracle::grid::LaneItemRectInput {
            grid_axis_start: 12.0,
            grid_axis_size: 50.0,
            lane_axis_offset: 27.0,
            lane_axis_size: 40.0,
            grid_axis_is_column: true,
        });

    assert_eq!(rect, GridItemRect::new(12.0, 27.0, 50.0, 40.0));
}

#[test]
fn oracle_scenario_offsets_grid_items_by_baseline_report() {
    let baseline_rect = support::oracle::grid::compose_baseline_aligned_item_rect(
        support::oracle::grid::BaselineAlignedItemRectInput {
            area_x: 10.0,
            area_y: 4.0,
            area_width: 50.0,
            area_height: 40.0,
            item_width: 20.0,
            item_height: 30.0,
            normal_x_offset: 3.0,
            normal_y_offset: 8.0,
            baseline_y_offset: Some(6.0),
        },
    );

    assert_eq!(baseline_rect, GridItemRect::new(13.0, 10.0, 20.0, 30.0));

    let normal_rect = support::oracle::grid::compose_baseline_aligned_item_rect(
        support::oracle::grid::BaselineAlignedItemRectInput {
            area_x: 10.0,
            area_y: 4.0,
            area_width: 50.0,
            area_height: 40.0,
            item_width: 20.0,
            item_height: 30.0,
            normal_x_offset: 3.0,
            normal_y_offset: 8.0,
            baseline_y_offset: None,
        },
    );

    assert_eq!(normal_rect, GridItemRect::new(13.0, 12.0, 20.0, 30.0));
}

#[test]
fn oracle_direct_subgrid_inherited_columns_shape() {
    let inherited = support::oracle::grid::inherit_subgrid_tracks(
        support::oracle::grid::SubgridTrackInheritanceInput {
            parent_tracks: vec![80.0, 120.0],
            parent_span: support::oracle::grid::TrackSpan::new(1, 3),
            reversed: false,
            start_mbp: 0.0,
            end_mbp: 0.0,
            parent_gap: support::oracle::grid::OracleGapReport::length(10.0),
            subgrid_gap: support::oracle::grid::OracleGapReport::normal_resolved_to(10.0),
        },
    )
    .unwrap();

    assert_eq!(inherited.final_tracks, vec![80.0, 120.0]);
}

#[test]
fn oracle_grid_lanes_three_item_shape() {
    let report = support::oracle::grid::place_lanes(support::oracle::grid::LanePlacementInput {
        grid_axis_tracks: 2,
        auto_flow: support::oracle::grid::LaneAutoFlow::Row,
        lane_gap: 5.0,
        tolerance: support::oracle::grid::LaneFlowTolerance::Fixed(0.0),
        tolerance_basis: 0.0,
        items: vec![
            support::oracle::grid::LaneItemInput::auto("a", 1, 20.0),
            support::oracle::grid::LaneItemInput::auto("b", 1, 30.0),
            support::oracle::grid::LaneItemInput::auto("c", 2, 10.0),
        ],
    })
    .unwrap();

    assert_eq!(report.item_offsets.len(), 3);
    assert_eq!(report.content_size, 45.0);
}
