//! A sheet's statements must not each read the whole sheet again, so the nodes
//! read for one parse grow in proportion to the sheet, whatever its rule count.

use super::bounded_execution::work;

fn reads_per_byte(source: &str) -> f64 {
    let (report, built, scanned) =
        work::measure(|| crate::CssParser::default().parse_sheet(source));
    let reads = built + scanned;
    assert!(report.diagnostics().is_empty(), "{report:?}");
    assert!(reads > 0, "the parse reads a token tree");
    reads as f64 / source.len() as f64
}

fn sheet(statement: impl Fn(usize) -> String, count: usize) -> String {
    (0..count).map(statement).collect()
}

fn assert_linear(statement: impl Fn(usize) -> String) {
    let one = reads_per_byte(&sheet(&statement, 1));
    for count in [10, 100] {
        let many = reads_per_byte(&sheet(&statement, count));
        println!("1 statement {one:.2} node reads per byte; {count} statements {many:.2}");
        // Per-byte work for many statements stays within that of one
        // statement; a whole-sheet walk per statement would grow with count.
        assert!(
            many <= one + 0.25,
            "{count} statements read {many:.2} nodes per sheet byte; one reads {one:.2}"
        );
    }
}

#[test]
fn custom_media_statements_read_work_linear_in_sheet_size() {
    assert_linear(|index| format!("@custom-media --m{index} (width > {index}px);\n"));
    assert_linear(|index| format!("@custom-media --b{index} true;\n"));
}

#[test]
fn import_statements_read_work_linear_in_sheet_size() {
    assert_linear(|index| format!("@import url(\"s{index}.css\") screen;\n"));
}
