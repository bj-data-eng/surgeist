#![forbid(unsafe_code)]

//! A host may inspect authored Grid placement without resolving layout lines.

use surgeist_css::{CssGridLine, CssKnownPropertyValueRef, CssValueOrigin, parse_style_attribute};

fn main() {
    let report =
        parse_style_attribute("grid-area: main 2147483648 / nav / span calc(-1) !important");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = &report.syntax().as_slice()[0];
    let CssKnownPropertyValueRef::GridArea(value) = declaration
        .known()
        .expect("known placement property")
        .property_value()
        .expect("ordinary placement value")
    else {
        panic!("grid-area value")
    };
    let area = value.value();
    assert!(area.authored_column_end().is_none());
    assert_eq!(area.effective_column_end(), area.effective_column_start());
    assert_eq!(
        area.effective_row_end().serialize_specified().unwrap(),
        "span calc(-1)"
    );
    assert_eq!(
        area.serialize_specified().unwrap(),
        "2147483648 main / nav / span calc(-1)"
    );
    let CssGridLine::Indexed(index) = area.row_start() else {
        panic!("indexed row start")
    };
    let surgeist_css::CssIntegerValue::ExactLiteral(literal) = index.value() else {
        panic!("exact ordinary integer")
    };
    assert!(matches!(literal.origin(), CssValueOrigin::Parsed(_)));
}
