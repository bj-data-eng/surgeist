use super::Golden;

#[test]
fn geometry_replay_refuses_a_replaced_image_witness_without_capture_authority() {
    let xml = include_str!(
        "captured/flex-composition-differences/xml/flex/fri07_flex_composition__border_box_rtl.xml"
    );
    let error = Golden::parse(xml).expect_err("geometry replay must refuse unbound image evidence");
    assert!(
        error
            .to_string()
            .contains("Flex composition image requires capture authority"),
        "{error}"
    );
}
