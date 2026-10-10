#![forbid(unsafe_code)]

use surgeist_css::{
    CssKnownProperty, CssPropertyNameRef, CssSpecifiedDeclarationBlock, parse_declaration_list_text,
};

#[test]
fn complete_margin_getter_crosses_logical_interference_without_reordering_css_text() {
    let report = parse_declaration_list_text(
        "margin-top:1px; margin-inline-start:7px; margin-right:2px; margin-bottom:3px; margin-left:4px",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "margin-top: 1px; margin-inline-start: 7px; margin-right: 2px; margin-bottom: 3px; margin-left: 4px;",
    );
    assert_eq!(
        block
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::Margin))
            .unwrap(),
        Some("1px 2px 3px 4px".to_owned()),
    );
}
