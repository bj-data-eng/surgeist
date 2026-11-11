//! Exact source relation for the authenticated Grid Lanes axis fixtures.

use std::fmt;

const COLUMN_MARKER: &str = "grid-auto-flow: column; grid-template-columns: 30px 40px;";
const ROW_MARKER: &str =
    "grid-auto-flow: row; grid-template-columns: 30px 40px; grid-template-rows: 50px 60px;";

#[derive(Debug, PartialEq, Eq)]
pub(super) enum SourceDeltaError {
    MissingMarker(&'static str),
    DuplicateMarker(&'static str),
    ReversedMarkers,
    ControlBytesChanged,
}

impl fmt::Display for SourceDeltaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMarker(marker) => {
                write!(f, "original is missing template marker {marker:?}")
            }
            Self::DuplicateMarker(marker) => {
                write!(f, "original repeats template marker {marker:?}")
            }
            Self::ReversedMarkers => f.write_str("original template markers are reversed"),
            Self::ControlBytesChanged => {
                f.write_str("control bytes differ from the two allowed template replacements")
            }
        }
    }
}

impl std::error::Error for SourceDeltaError {}

fn unique_marker(original: &[u8], marker: &'static str) -> Result<usize, SourceDeltaError> {
    let mut offsets = original
        .windows(marker.len())
        .enumerate()
        .filter_map(|(offset, bytes)| (bytes == marker.as_bytes()).then_some(offset));
    let first = offsets
        .next()
        .ok_or(SourceDeltaError::MissingMarker(marker))?;
    if offsets.next().is_some() {
        return Err(SourceDeltaError::DuplicateMarker(marker));
    }
    Ok(first)
}

pub(super) fn verify(original: &[u8], control: &[u8]) -> Result<(), SourceDeltaError> {
    let column = unique_marker(original, COLUMN_MARKER)?;
    let row = unique_marker(original, ROW_MARKER)?;
    if column >= row {
        return Err(SourceDeltaError::ReversedMarkers);
    }

    // Each final semicolon and every byte outside the two values is retained.
    let column_end = column + COLUMN_MARKER.len() - 1;
    let row_end = row + ROW_MARKER.len() - 1;
    let column_value = column_end - "30px 40px".len();
    let row_value = row_end - "50px 60px".len();
    let expected = [
        &original[..column_value],
        b"none",
        &original[column_end..row_value],
        b"none",
        &original[row_end..],
    ]
    .concat();
    if expected != control {
        return Err(SourceDeltaError::ControlBytesChanged);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGINAL: &str =
        include_str!("html/grid-lanes/grid_lanes_axes_vertical_rl_parallel.html");
    const CONTROL: &str = include_str!(
        "../../../examples/qualify-root-context/html/qualification/grid_lanes_axes_vertical_rl_parallel_control.html"
    );

    #[test]
    fn source_pair_accepts_exact_template_replacements() {
        assert_eq!(verify(ORIGINAL.as_bytes(), CONTROL.as_bytes()), Ok(()));
    }

    #[test]
    fn source_pair_refuses_an_unrelated_control_edit() {
        let edited = CONTROL.replace("<title>Grid lanes", "<title>Other grid lanes");
        assert_ne!(edited, CONTROL);
        assert_eq!(
            verify(ORIGINAL.as_bytes(), edited.as_bytes()),
            Err(SourceDeltaError::ControlBytesChanged)
        );
    }

    #[test]
    fn source_pair_refuses_replacing_the_grid_axis_template() {
        for (allowed, wrong) in [
            (
                "grid-auto-flow: column; grid-template-columns: none; grid-template-rows: 50px 60px;",
                "grid-auto-flow: column; grid-template-columns: 30px 40px; grid-template-rows: none;",
            ),
            (
                "grid-auto-flow: row; grid-template-columns: 30px 40px; grid-template-rows: none;",
                "grid-auto-flow: row; grid-template-columns: none; grid-template-rows: 50px 60px;",
            ),
        ] {
            let edited = CONTROL.replace(allowed, wrong);
            assert_ne!(edited, CONTROL);
            assert_eq!(
                verify(ORIGINAL.as_bytes(), edited.as_bytes()),
                Err(SourceDeltaError::ControlBytesChanged)
            );
        }
    }

    #[test]
    fn source_pair_requires_both_markers_once() {
        for marker in [COLUMN_MARKER, ROW_MARKER] {
            let missing = ORIGINAL.replace(marker, "");
            assert_eq!(
                verify(missing.as_bytes(), CONTROL.as_bytes()),
                Err(SourceDeltaError::MissingMarker(marker))
            );
            let original = [ORIGINAL, marker].concat();
            let control = [CONTROL, marker].concat();
            assert_eq!(
                verify(original.as_bytes(), control.as_bytes()),
                Err(SourceDeltaError::DuplicateMarker(marker))
            );
        }
    }

    #[test]
    fn source_pair_requires_column_marker_before_row_marker() {
        fn reverse(source: &str, column: &str, row: &str) -> String {
            let (before, rest) = source.split_once(column).unwrap();
            let (between, after) = rest.split_once(row).unwrap();
            [before, row, between, column, after].concat()
        }

        let original = reverse(ORIGINAL, COLUMN_MARKER, ROW_MARKER);
        let control = reverse(
            CONTROL,
            "grid-auto-flow: column; grid-template-columns: none;",
            "grid-auto-flow: row; grid-template-columns: 30px 40px; grid-template-rows: none;",
        );
        assert_eq!(
            verify(original.as_bytes(), control.as_bytes()),
            Err(SourceDeltaError::ReversedMarkers)
        );
    }
}
