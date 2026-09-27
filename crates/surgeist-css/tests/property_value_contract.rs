#![forbid(unsafe_code)]

//! The outer test compiles before the checked property-value API exists. Its
//! executed assertion requires the separate public consumer to compile and run.
//! The nested offline Cargo command shares the outer bounded process group and
//! owns an isolated target directory, following component_value_contract.rs.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_property_value_consumer_compiles_and_preserves_declaration_contracts() {
    let output = external_consumer::run_example("property_value_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "checked construction: ok\n",
            "declared value branches: ok\n",
            "value boundaries and origins: ok\n",
            "parsed declaration coordinates: ok\n",
            "occurrence identity and value equality: ok\n",
            "original snapshot after structural recovery: ok\n",
        )
    );
}
