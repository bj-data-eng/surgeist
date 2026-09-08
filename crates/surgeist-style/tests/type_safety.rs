#[test]
fn public_type_safety_contract() {
    let tests = trybuild::TestCases::new();
    // Includes privacy cases such as invalid_font_*_literal.rs and invalid_text_decoration_*_literal.rs.
    tests.compile_fail("tests/compile_fail/*.rs");
    tests.pass("tests/compile_pass/*.rs");
}
