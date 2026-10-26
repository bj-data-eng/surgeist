//! Grid line indices compare integer values, ignoring token spelling and
//! diagnostic origins, as specified by `CssGridLineIndex`'s contract.

use surgeist_css::*;

fn line(source: &str) -> CssGridLine {
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one Grid line declaration")
    };
    let CssKnownPropertyValueRef::GridColumnStart(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("grid-column-start")
    };
    value.value().clone()
}

#[test]
fn grid_line_indices_compare_values_across_spellings_at_every_integer_magnitude() {
    for (left, right, canonical) in [
        ("+0007", "7", "7"),
        ("-0007", "-7", "-7"),
        ("+02147483648", "2147483648", "2147483648"),
    ] {
        let left = line(&format!("grid-column-start:{left}"));
        let right = line(&format!("grid-column-start:{right}"));
        assert_eq!(
            left.serialize_specified(Default::default()).unwrap(),
            canonical
        );
        assert_eq!(
            right.serialize_specified(Default::default()).unwrap(),
            canonical
        );
        assert_eq!(left, right, "equal integers compare equal across spellings");
    }
}

#[test]
fn equal_grid_line_spellings_ignore_source_snapshot_and_coordinates() {
    for spelling in ["7", "+0007", "2147483648", "+02147483648", "-0007"] {
        let left = line(&format!("grid-column-start:{spelling}"));
        let right = line(&format!("  grid-column-start: {spelling}"));
        assert_eq!(left, right, "diagnostic origins do not alter equality");
    }
}
