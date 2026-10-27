//! Lexical preparation is a linear walk of its input. A sheet's statements
//! must not each walk the whole sheet again, so the bytes prepared for one
//! parse grow in proportion to the sheet, whatever its rule count.

use crate::tokenization::PREPARED_BYTES;

fn prepared_per_byte(source: &str) -> f64 {
    PREPARED_BYTES.with(|prepared| prepared.set(0));
    let report = crate::CssParser::default().parse_sheet(source);
    let prepared = PREPARED_BYTES.with(std::cell::Cell::get);
    assert!(report.diagnostics().is_empty(), "{report:?}");
    prepared as f64 / source.len() as f64
}

fn sheet(statement: impl Fn(usize) -> String, count: usize) -> String {
    (0..count).map(statement).collect()
}

fn assert_linear(statement: impl Fn(usize) -> String) {
    let one = prepared_per_byte(&sheet(&statement, 1));
    for count in [10, 100] {
        let many = prepared_per_byte(&sheet(&statement, count));
        println!("1 statement {one:.2} prepared bytes per byte; {count} statements {many:.2}");
        // Per-byte work for many statements stays within that of one
        // statement; a whole-sheet walk per statement would grow with count.
        assert!(
            many <= one + 1.0,
            "{count} statements prepare {many:.2} bytes per sheet byte; one prepares {one:.2}"
        );
    }
}

#[test]
fn custom_media_statements_prepare_work_linear_in_sheet_size() {
    assert_linear(|index| format!("@custom-media --m{index} (width > {index}px);\n"));
    assert_linear(|index| format!("@custom-media --b{index} true;\n"));
}

#[test]
fn import_statements_prepare_work_linear_in_sheet_size() {
    assert_linear(|index| format!("@import url(\"s{index}.css\") screen;\n"));
}
