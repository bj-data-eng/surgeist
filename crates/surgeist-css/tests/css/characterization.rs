//! Characterization baseline of the public CSS surface.
//!
//! `fixtures/public-report-characterization.tsv` is captured from the crate's
//! own output. It records every public front by a stable front id, the clean
//! flag, retained syntax, specified and CSSOM serialization, diagnostics,
//! charged work, colour outcomes, colour protocol traces, and error `Display`
//! and `source()` chains. A matching file shows only that observed behaviour
//! is unchanged; it is not a correctness oracle, and re-capturing it is never
//! evidence that a behaviour change is right. Independent expectations live in
//! the domain modules and in `corpora::public_report_characterization`, which
//! checks typed values against the same file.
//!
//! `adapter` is the only code here that calls the CSS API, so an API change
//! edits the adapter and leaves the case table and the golden file alone. The
//! case table is `cases`: the report cases in `report_cases.tsv`, handwritten
//! front, colour and error cases, and the shared property inventory.
//!
//! Capture is a writing operation, not a routine check:
//!
//! ```sh
//! SURGEIST_CSS_WRITE_CHARACTERIZATION=1 cargo test --offline --locked -j 1 \
//!     -p surgeist-css --test css characterization::capture -- --ignored
//! ```

mod adapter;
mod cases;

use std::fmt::Write as _;
use std::path::Path;

const GOLDEN_PATH: &str = "tests/fixtures/public-report-characterization.tsv";
const GOLDEN: &str = include_str!("../fixtures/public-report-characterization.tsv");
const CAPTURE_GATE: &str = "SURGEIST_CSS_WRITE_CHARACTERIZATION";

/// The golden header. The first eight columns keep the report-observable
/// schema that `corpora::public_report_characterization` also reads.
pub const HEADER: &str = "case_id\tentry\tfeature\tinput\tclean\tretained\tauthored_declarations\tdiagnostics\targument\tshape\tspecified\tcssom\tcharges\tfacts";

/// Every case observed through the adapter, rendered in canonical case order.
fn render_golden() -> String {
    let mut rows = cases::all()
        .into_iter()
        .map(|case| {
            let observation = adapter::observe(&case);
            let fields = [
                case.id.as_str(),
                case.front.as_str(),
                case.feature.as_str(),
                case.input.as_str(),
                &observation.clean,
                &observation.retained,
                &observation.authored,
                &observation.diagnostics,
                case.argument.as_str(),
                &observation.shape,
                &observation.specified,
                &observation.cssom,
                &observation.charges,
                &observation.facts,
            ];
            for (index, field) in fields.iter().enumerate() {
                assert!(
                    index == 3 || !field.is_empty(),
                    "{}: empty observation in column {index}",
                    case.id
                );
            }
            (case.id.clone(), fields.map(escape).join("\t"))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    for pair in rows.windows(2) {
        assert_ne!(
            pair[0].0, pair[1].0,
            "duplicate characterization case id {}",
            pair[0].0
        );
    }
    let mut output = String::with_capacity(GOLDEN.len());
    output.push_str(HEADER);
    output.push('\n');
    for (_, row) in rows {
        output.push_str(&row);
        output.push('\n');
    }
    output
}

/// Tabs, line breaks and backslashes are escaped so one case is one line.
fn escape(field: &str) -> String {
    let mut output = String::with_capacity(field.len());
    for character in field.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            other => output.push(other),
        }
    }
    output
}

/// The first differing line of two renderings, with its case id.
fn first_difference(expected: &str, actual: &str) -> String {
    let mut expected_lines = expected.lines();
    let mut actual_lines = actual.lines();
    let mut line = 1;
    loop {
        match (expected_lines.next(), actual_lines.next()) {
            (None, None) => return "renderings differ only in trailing bytes".to_owned(),
            (left, right) if left == right => line += 1,
            (left, right) => {
                let mut message = format!("first difference at golden line {line}\n");
                let _ = writeln!(message, "golden:   {}", left.unwrap_or("<end of file>"));
                let _ = writeln!(message, "observed: {}", right.unwrap_or("<end of file>"));
                return message;
            }
        }
    }
}

#[test]
fn characterization_golden_matches_observed_public_behaviour() {
    let observed = render_golden();
    assert!(
        observed == GOLDEN,
        "{}\nre-capture only for a recorded behaviour change: {CAPTURE_GATE}=1 cargo test \
         -p surgeist-css --test css characterization::capture -- --ignored",
        first_difference(GOLDEN, &observed)
    );
}

#[test]
#[ignore = "writes the characterization golden file only when explicitly invoked with its exact gate"]
fn capture() {
    assert_eq!(
        std::env::var_os(CAPTURE_GATE).as_deref(),
        Some(std::ffi::OsStr::new("1")),
        "refusing to write the characterization golden file without {CAPTURE_GATE}=1"
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_PATH);
    let temporary = path.with_extension("tsv.capture.tmp");
    std::fs::write(&temporary, render_golden()).expect("write the captured golden file");
    std::fs::rename(&temporary, &path).expect("replace the golden file");
}
