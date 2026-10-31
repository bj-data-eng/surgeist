//! Cell-inner font geometry and immutable unit identity.
use super::*;
use crate::engine::layout::data::FinalizedRunFrame;
use crate::{SourceRangeCellUnitKind, SourceRangeGeometry, SourceRangeUnit};

fn actual_inner_clusters(layout: &Layout) -> [usize; 2] {
    // Independent finalized producer incidence, read BEFORE any Range result.
    // The public fixture already qualified two real A glyphs at raw0/10 with
    // advances10 and physical5-wide cluster rows. Do not derive these indices
    // from EGC/glyph cardinality or a payload-local numbering convention.
    let line = &layout.inner.lines[0].finalized;
    assert_eq!(line.data.cells.len(), 1);
    assert!(!line.data.font_runs.is_empty());
    for run in &line.data.font_runs {
        // Retained actual font-design origin offset b, BEFORE Range. Only
        // this qualified b0 fixture makes metric and paint-origin Y coincide.
        assert_eq!(run.metrics.font_origin_offset(), 0.0);
    }
    let mut actual = [0; 2];
    for (ordinal, index) in actual.iter_mut().enumerate() {
        let source = Range::new(ordinal, ordinal + 1);
        let candidates = line
            .data
            .clusters
            .iter()
            .enumerate()
            .filter(|(_, cluster)| {
                matches!(cluster.frame, FinalizedRunFrame::Cell(0)) && cluster.source == source
            })
            .collect::<Vec<_>>();
        let [(found, cluster)] = candidates.as_slice() else {
            panic!("one real finer finalized A metric row");
        };
        assert!(cluster.provider_atoms.contains(&ordinal));
        assert_eq!(cluster.x, f64::from(u32::try_from(ordinal).unwrap()) * 10.0);
        assert_eq!((cluster.shaped_advance, cluster.advance), (10.0, 10.0));
        assert_eq!(cluster.logical_adjustments, [0.0; 3]);
        *index = *found;
    }
    assert_ne!(actual[0], actual[1]);
    actual
}

fn inspect(
    unit: SourceRangeUnit<'_>,
    layout: &Layout,
    origin: &PreparedOrigin,
    ordinal: usize,
    cluster: usize,
) {
    let owner = unit
        .cell_unit()
        .expect("a genuine Cell EGC has an actual Cell unit owner");
    assert_eq!(owner.kind(), SourceRangeCellUnitKind::Semantic { ordinal });
    assert_eq!((owner.line_index(), owner.cell_index()), (0, 0));
    assert_eq!(owner.presentation_range(), unit.presentation_range());
    let source_cell = owner.source_cell();
    let scope = layout
        .analyzed_source()
        .processed_source()
        .captured_scope(1)
        .unwrap();
    assert_eq!(source_cell.scope(), scope);
    assert_eq!(source_cell.mode(), TextCombineUpright::All);
    assert_eq!(
        source_cell.inner_direction(),
        ParagraphBaseDirection::LeftToRight
    );
    assert_eq!(source_cell.used_unit_count(), 2);
    assert_eq!(
        source_cell.used_code_points().collect::<Vec<_>>(),
        [0x41, 0x41]
    );
    let expected_source = Range::new(ordinal, ordinal + 1);
    for intersections in [
        owner.selected_intersections().collect::<Vec<_>>(),
        owner.expanded_intersections().collect::<Vec<_>>(),
    ] {
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].origin(), *origin);
        assert_eq!(intersections[0].local_range(), expected_source);
        assert_eq!(intersections[0].aggregate_range(), expected_source);
    }
    let geometries = unit.pieces().collect::<Vec<_>>();
    let [geometry] = geometries.as_slice() else {
        panic!("one real font piece for this A EGC");
    };
    let SourceRangeGeometry::CellFont(piece) = geometry else {
        panic!("an inner A font metric is neither ordinary Line nor native CellTab geometry");
    };
    assert!(piece.cell_unit().same_unit(&owner));
    assert_eq!(
        piece.cell_unit().kind(),
        SourceRangeCellUnitKind::Semantic { ordinal }
    );
    let edges = if ordinal == 0 {
        (0.0, 10.0)
    } else {
        (10.0, 20.0)
    };
    assert_eq!(piece.inner_advance_edges(), edges);
    // This exact singleton follows the independently qualified finer finalized
    // source row above. It imposes no singleton rule on ligature/share fallback.
    assert_eq!(piece.inner_cluster_indices().collect::<Vec<_>>(), [cluster]);
    let packets = layout.glyph_runs();
    let [actual_run] = packets.as_slice() else {
        panic!("one actual retained AA font packet");
    };
    let instance = piece.effective_instance();
    assert!(instance.face().is_same_face(&actual_run.font().face()));
    assert_eq!(instance.face().bytes(), actual_run.font().bytes());
    assert_eq!(instance.font_size(), 10.0);
    assert_eq!(instance.axis_count(), 0);
    let font_metrics = instance
        .metrics(instance.face().catalogue_identity())
        .unwrap();
    assert_eq!((font_metrics.ascent(), font_metrics.descent()), (8.0, -2.0));
    // Piece extents use the existing Native under-distance convention. They
    // are not the same signed descent observation returned by FontMetrics.
    assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
    // This is the common METRIC reference, not a GlyphFrame/design origin.
    // actual_inner_clusters qualified retained b0 before any Range query;
    // parent fixture independently qualifies the real paint GlyphFrame.
    let frame = piece.metric_frame();
    assert_eq!(frame.origin(), Point::new(95.0, 28.0));
    assert_eq!(frame.x_scale(), 0.5);
    // The real uncompressed edges0/10/20 and font band-8/+2 map through
    // common metric reference (95,28)+(.5*h,v): these are metric boxes,
    // not ink or the whole square. No raw metrics are reconstructed.
    let expected = if ordinal == 0 { FIRST } else { SECOND };
    assert_eq!(piece.rect(), expected);
    assert_eq!(geometry.rect(), expected);
}

#[test]
fn aa_cell_font_variants_expose_the_actual_font_frame_edges_and_semantic_identity() {
    let (layout, origin) = fixture();
    let clusters = actual_inner_clusters(&layout);
    let full = layout
        .source_range_fragments(&origin, Range::new(0, 2))
        .unwrap();
    exact(
        &full,
        &origin,
        Range::new(0, 2),
        &[(Range::new(0, 1), FIRST), (Range::new(1, 2), SECOND)],
    );
    let first = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    let second = layout
        .source_range_fragments(&origin, Range::new(1, 2))
        .unwrap();
    let units = full.units().collect::<Vec<_>>();
    for ordinal in 0..2 {
        inspect(units[ordinal], &layout, &origin, ordinal, clusters[ordinal]);
    }
    let partials = [
        first.units().next().unwrap(),
        second.units().next().unwrap(),
    ];
    for ordinal in 0..2 {
        inspect(
            partials[ordinal],
            &layout,
            &origin,
            ordinal,
            clusters[ordinal],
        );
        assert!(units[ordinal].same_unit(&partials[ordinal]));
        assert!(
            units[ordinal]
                .cell_unit()
                .unwrap()
                .same_unit(&partials[ordinal].cell_unit().unwrap())
        );
    }
    // The actual original occurrence and containing Cell are shared; neither
    // licenses collapsing these two genuine semantic-unit identities.
    assert_eq!(
        units[0].selected_intersections().next().unwrap().origin(),
        units[1].selected_intersections().next().unwrap().origin()
    );
    assert_eq!(
        units[0].cell_unit().unwrap().source_cell().scope(),
        units[1].cell_unit().unwrap().source_cell().scope()
    );
    assert!(!units[0].same_unit(&units[1]));
    assert!(
        !units[0]
            .cell_unit()
            .unwrap()
            .same_unit(&units[1].cell_unit().unwrap())
    );
}

#[test]
fn equal_geometry_and_same_original_do_not_alias_different_actual_placements() {
    let (layout, origin) = fixture();
    let original = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    let unit = original.units().next().unwrap();
    let cell = unit.cell_unit().unwrap();
    let clone = layout.clone();
    let aliased = clone
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    let aliased_unit = aliased.units().next().unwrap();
    assert!(unit.same_unit(&aliased_unit));
    assert!(cell.same_unit(&aliased_unit.cell_unit().unwrap()));
    // A fresh real placement of the SAME retained finalized line, at the SAME
    // anchor, shares its PreparedOrigin and literal rectangles, not Layout identity.
    let line = unit.finalized_line();
    let prepared = line.prepared_text();
    let packets = layout.glyph_runs();
    let [packet] = packets.as_slice() else {
        panic!("actual retained AA packet");
    };
    let separate = placed_cell(&prepared, line, packet.font().bytes(), 0.5);
    let other = separate
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    let other_unit = other.units().next().unwrap();
    let other_cell = other_unit.cell_unit().unwrap();
    assert_eq!(rectangles(&original), [FIRST]);
    assert_eq!(rectangles(&other), [FIRST]);
    assert_eq!(original.origin(), other.origin());
    assert_eq!(cell.kind(), other_cell.kind());
    assert_eq!(
        (cell.line_index(), cell.cell_index()),
        (other_cell.line_index(), other_cell.cell_index())
    );
    assert!(!unit.same_unit(&other_unit));
    assert!(!cell.same_unit(&other_cell));
}
