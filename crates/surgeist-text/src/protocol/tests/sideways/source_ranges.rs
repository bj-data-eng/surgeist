//! Source-range pieces observe the actual physical placed basis, while their
//! font instance, source intersections and provider cluster ordinals stay owned.
use super::*;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/ahem/Ahem.ttf"
));

struct Case {
    orientation: LineOrientation,
    inline: f64,
    baseline: f64,
    cell: Rect,
    run: Point,
    carets: [Rect; 2],
}

fn check(case: Case) {
    let mut s = system();
    let p = prepare(
        &mut s,
        input(source("A"), own_style(), false),
        case.orientation,
    );
    assert_eq!(p.orientation(), case.orientation);
    assert_eq!(p.analyzed_source().source().text().code_units(), [65]);
    assert_eq!(
        p.analyzed_source().source().id(),
        Some(Id::from_u64(1515002))
    );
    assert_eq!(p.analyzed_source().source().revision(), 11);
    assert_eq!(
        p.unit_views()
            .map(|unit| (unit.source_range(), unit.level().get(), unit.advance()))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 0, 10.0)]
    );
    assert_eq!(p.atomic_views().count(), 0);
    assert_eq!(p.control_views().count(), 0);
    let mut origins = p.origin_views();
    let origin = origins.next().unwrap().handle();
    assert!(origins.next().is_none());
    let line = whole(&p);
    assert_eq!(line.prepared_text(), p);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().ascent,
            line.feedback().descent,
            line.feedback().line_height
        ),
        (10.0, 8.0, 8.0, 2.0, 10.0)
    );
    assert_eq!(glyphs(&line), [(35, 0.0, 0.0, 0.0, 10.0)]);
    let original_run = line.run_views().next().unwrap();
    assert_eq!(original_run.face().bytes(), AHEM);
    assert_eq!(original_run.face().index(), 0);
    assert_eq!(
        original_run.face().catalogue_identity(),
        p.font_catalogue_identity()
    );
    let original_instance = original_run.effective_instance().unwrap();
    let metrics = original_instance
        .metrics(p.font_catalogue_identity())
        .unwrap();
    assert_eq!(
        (metrics.units_per_em(), metrics.ascent(), metrics.descent()),
        (1000, 8.0, -2.0)
    );

    let layout = placed(&p, vec![(line.clone(), case.inline, case.baseline)]);
    assert_eq!(layout.orientation(), case.orientation);
    assert_eq!(
        layout.analyzed_source().identity(),
        p.analyzed_source().identity()
    );
    assert_eq!(layout.source(), p.analyzed_source().source());
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), Range::new(0, 1));
    assert_eq!(layout.lines()[0].bounds(), case.cell);
    assert_eq!(layout.lines()[0].baseline(), case.baseline);
    assert_eq!(layout.run_views().next().unwrap().origin(), case.run);
    assert_eq!(
        layout
            .cursor(Cursor::new(0, Affinity::After))
            .unwrap()
            .rect(),
        case.carets[0]
    );
    assert_eq!(
        layout
            .cursor(Cursor::new(1, Affinity::Before))
            .unwrap()
            .rect(),
        case.carets[1]
    );
    let selection = layout
        .selection(Selection::new(
            Cursor::new(0, Affinity::After),
            Cursor::new(1, Affinity::Before),
        ))
        .unwrap();
    assert_eq!(selection.rects().len(), 1);
    assert_eq!(selection.rects()[0].rect(), case.cell);

    // Only now query the separate original-source observation owner. Ahem A
    // occupies local inline0..10 and font over/under8/2; physical CW keeps its
    // inline edges on y, baseline on x, and cell baselineX−2 .. baselineX+8.
    let query = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert!(std::ptr::eq(query.layout(), &layout));
    assert_eq!(query.layout().orientation(), case.orientation);
    assert_eq!(query.origin(), &origin);
    assert_eq!(query.local_range(), Range::new(0, 1));
    assert_eq!(query.units().len(), 1);
    let unit = query.units().next().unwrap();
    assert_eq!(unit.line_index(), 0);
    assert_eq!(unit.finalized_line(), line);
    assert_eq!(
        unit.selected_intersections()
            .map(|part| (part.origin(), part.local_range(), part.aggregate_range()))
            .collect::<Vec<_>>(),
        [(origin.clone(), Range::new(0, 1), Range::new(0, 1))]
    );
    assert_eq!(
        unit.expanded_intersections()
            .map(|part| (part.origin(), part.local_range(), part.aggregate_range()))
            .collect::<Vec<_>>(),
        [(origin.clone(), Range::new(0, 1), Range::new(0, 1))]
    );
    assert_eq!(unit.fragments().count(), 1);
    assert_eq!(unit.pieces().len(), 1);
    let crate::SourceRangeGeometry::Line(piece) = unit.pieces().next().unwrap() else {
        panic!("ordinary Range fixture must retain Line geometry");
    };
    assert_eq!(piece.cluster_indices().collect::<Vec<_>>(), [0]);
    assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
    let instance = piece.effective_instance();
    assert_eq!(instance.face().bytes(), original_instance.face().bytes());
    assert_eq!(instance.face().index(), original_instance.face().index());
    assert_eq!(
        instance.face().catalogue_identity(),
        original_instance.face().catalogue_identity()
    );
    assert_eq!(instance.font_size(), original_instance.font_size());
    assert_eq!(
        instance.normalized_coords().collect::<Vec<_>>(),
        original_instance.normalized_coords().collect::<Vec<_>>()
    );
    assert_eq!(instance.synthesis(), original_instance.synthesis());
    assert_eq!(
        piece.rect(),
        case.cell,
        "range-piece rectangle must use the retained physical placement basis"
    );
    assert_eq!(
        piece.advance_edges(),
        (case.inline, case.inline + 10.0),
        "directed edges are physical inline-axis coordinates"
    );
    assert_eq!(
        piece.baseline(),
        case.baseline,
        "baseline is the physical cross-axis coordinate"
    );
}

#[test]
fn source_range_piece_geometry_matches_authentic_horizontal_and_clockwise_placement() {
    check(Case {
        orientation: H,
        inline: 20.0,
        baseline: 100.0,
        cell: Rect::new(20.0, 92.0, 10.0, 10.0),
        run: Point::new(20.0, 100.0),
        carets: [
            Rect::new(20.0, 92.0, 1.0, 10.0),
            Rect::new(30.0, 92.0, 1.0, 10.0),
        ],
    });
    check(Case {
        orientation: CW,
        inline: 20.0,
        baseline: 100.0,
        cell: Rect::new(98.0, 20.0, 10.0, 10.0),
        run: Point::new(100.0, 20.0),
        carets: [
            Rect::new(98.0, 20.0, 10.0, 1.0),
            Rect::new(98.0, 30.0, 10.0, 1.0),
        ],
    });
}

#[test]
fn source_range_piece_geometry_retains_fractional_negative_clockwise_anchor() {
    check(Case {
        orientation: H,
        inline: -3.25,
        baseline: -5.5,
        cell: Rect::new(-3.25, -13.5, 10.0, 10.0),
        run: Point::new(-3.25, -5.5),
        carets: [
            Rect::new(-3.25, -13.5, 1.0, 10.0),
            Rect::new(6.75, -13.5, 1.0, 10.0),
        ],
    });
    check(Case {
        orientation: CW,
        inline: -3.25,
        baseline: -5.5,
        cell: Rect::new(-7.5, -3.25, 10.0, 10.0),
        run: Point::new(-5.5, -3.25),
        carets: [
            Rect::new(-7.5, -3.25, 10.0, 1.0),
            Rect::new(-7.5, 6.75, 10.0, 1.0),
        ],
    });
}
