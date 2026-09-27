#![forbid(unsafe_code)]

//! The outer test compiles before the source-hint projection API exists. Its
//! executed assertion requires the public consumer to compile and run its
//! independent Fonts4 compatibility and authored-value preservation cases.
//! The nested offline Cargo process inherits the outer bounded process group
//! and owns an isolated target directory, following property_value_contract.rs.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_font_source_consumer_compiles_and_preserves_authored_and_semantic_hints() {
    let output = external_consumer::run_example("font_source_compatibility_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "legacy compatibility and construction: ok\n",
            "conjunctive technology requirements: ok\n",
            "absent and unknown format hints: ok\n",
            "format equivalence and authored equality: ok\n",
        )
    );
}
