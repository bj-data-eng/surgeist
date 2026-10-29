//! Authored boundary means and raw shaping preserved by the shared owner.
use super::fixtures::{allocations, assert_cells, ligature_font, select_method};
use super::*;

#[test]
fn opposite_full_tracking_values_are_neutral_and_unequal_values_share_one_cell_edge() {
    font_premises(AHEM, false);
    let units = [Range::new(0, 1), Range::new(1, 2)];
    let mut control_source = Source::new("AB");
    control_source.span(units[1], own(0.0));
    let control = whole(&prepare(AHEM, &control_source, own(0.0), &units));
    let expected_raw = [
        (units[0], 35, 0.0, 0.0, 10.0),
        (units[1], 36, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&control, AHEM), expected_raw);
    assert_eq!(control.feedback().advance, 20.0);
    assert_eq!(paint_offsets(&control), [(0.0, 0.0), (10.0, 0.0)]);
    for (right, edge, total) in [(-4.0, 10.0, 20.0), (8.0, 16.0, 26.0)] {
        let mut source = Source::identified("AB", Id::from_u64(1566020), 1);
        source.span(units[1], own(right));
        assert_eq!(source.spans()[0].range(), units[1]);
        assert_eq!(source.spans()[0].style().letter_spacing, right);
        let prepared = prepare(AHEM, &source, own(4.0), &units);
        let line = whole(&prepared);
        assert_eq!(raw(&line, AHEM), expected_raw);
        assert_eq!(line.feedback().advance, total);
        assert_eq!(line.measurements().alignment().advance(), total);
        assert_eq!(paint_offsets(&line), [(0.0, 0.0), (edge, 0.0)]);
        assert_cells(&prepared, line, &source, &[0.0, edge, total]);
        assert_eq!(prepared.analyzed_source().source(), &source);
    }
}

#[test]
fn material_authored_tracking_suppresses_real_optional_defaults_without_inter_character_selection()
{
    let units = [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)];
    for tag in [*b"liga", *b"clig"] {
        let bytes = ligature_font(tag);
        let source = Source::identified("fiA", Id::from_u64(1566021), 1);
        let control = whole(&prepare(&bytes, &source, own(0.0), &units));
        assert_eq!(
            raw(&control, &bytes),
            [
                (Range::new(0, 2), 58, 0.0, 0.0, 10.0),
                (units[2], 35, 0.0, 0.0, 10.0),
            ]
        );
        assert_eq!(control.feedback().advance, 20.0);
        let prepared = prepare(&bytes, &source, own(4.0), &units);
        let line = whole(&prepared);
        assert_eq!(
            line.selected_line().request().context.spacing,
            SpacingPolicy::None
        );
        assert_eq!(
            raw(&line, &bytes),
            [
                (units[0], 35, 0.0, 0.0, 10.0),
                (units[1], 36, 0.0, 0.0, 10.0),
                (units[2], 35, 0.0, 0.0, 10.0),
            ]
        );
        assert_eq!(line.feedback().advance, 38.0);
        assert_eq!(paint_offsets(&line), [(0.0, 0.0), (14.0, 0.0), (28.0, 0.0)]);
        assert_cells(&prepared, line, &source, &[0.0, 14.0, 28.0, 38.0]);
    }
}

#[test]
fn actual_kern_pair_keeps_raw_adjustment_and_an_independently_expandable_boundary() {
    let bytes = crate::shaping_test_fonts::pair_adjusted_ahem();
    font_premises(&bytes, false);
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert!(parsed.gpos().unwrap().lookup_list().is_ok());
    let gpos = sfnt::table(&bytes, *b"GPOS");
    let features = usize::from(word(gpos, 6));
    assert_eq!(word(gpos, features), 1);
    assert_eq!(&gpos[features + 2..features + 6], b"kern");
    let list = usize::from(word(gpos, 8));
    assert_eq!(word(gpos, list), 1);
    let lookup = list + usize::from(word(gpos, list + 2));
    assert_eq!((word(gpos, lookup), word(gpos, lookup + 4)), (2, 1));
    let sub = lookup + usize::from(word(gpos, lookup + 6));
    assert_eq!(
        (word(gpos, sub), word(gpos, sub + 4), word(gpos, sub + 6)),
        (1, 4, 0)
    );
    assert_eq!(word(gpos, sub + 8), 1);
    let coverage = sub + usize::from(word(gpos, sub + 2));
    assert_eq!(
        (
            word(gpos, coverage),
            word(gpos, coverage + 2),
            word(gpos, coverage + 4)
        ),
        (1, 1, 35)
    );
    let pair = sub + usize::from(word(gpos, sub + 10));
    assert_eq!((word(gpos, pair), word(gpos, pair + 2)), (1, 36));
    assert_eq!(i16::from_be_bytes(word(gpos, pair + 4).to_be_bytes()), -500);
    let source = Source::identified("AB", Id::from_u64(1566022), 1);
    let units = [Range::new(0, 1), Range::new(1, 2)];
    let control = whole(&prepare(&bytes, &source, own(0.0), &units));
    let expected_raw = [
        (units[0], 35, 0.0, 0.0, 5.0),
        (units[1], 36, 0.0, 0.0, 10.0),
    ];
    assert_eq!(raw(&control, &bytes), expected_raw);
    assert_eq!(control.feedback().advance, 15.0);
    assert_eq!(paint_offsets(&control), [(0.0, 0.0), (5.0, 0.0)]);
    let prepared = prepare(&bytes, &source, own(4.0), &units);
    let ordinary = whole(&prepared);
    assert_eq!(raw(&ordinary, &bytes), expected_raw);
    assert_eq!(ordinary.feedback().advance, 19.0);
    assert_eq!(paint_offsets(&ordinary), [(0.0, 0.0), (9.0, 0.0)]);
    let selected = select_method(&prepared, SpacingPolicy::InterCharacter);
    let natural = selected.finalize(SpacingAllocations::None).unwrap();
    assert_eq!(raw(&natural, &bytes), expected_raw);
    let row = natural.spacing_slots().next().unwrap();
    assert_eq!(
        row.geometry,
        crate::SpacingSlotGeometry::Boundary(crate::SpacingBoundaryGeometry::Independent)
    );
    assert_eq!(row.authored_letter_spacing, 4.0);
    assert!(row.can_expand());
    let allocated = natural
        .with_spacing(allocations(&selected, &[2.0]))
        .unwrap();
    assert_eq!(raw(&allocated, &bytes), expected_raw);
    assert_eq!(allocated.feedback().advance, 21.0);
    assert_eq!(paint_offsets(&allocated), [(0.0, 0.0), (11.0, 0.0)]);
    assert_cells(&prepared, allocated, &source, &[0.0, 11.0, 21.0]);
    assert_eq!(prepared.analyzed_source().source(), &source);
}
