#![forbid(unsafe_code)]

//! Positive public-consumer compilation and runtime contract.
//!
//! This harness itself compiles before the new API exists. Its RED is the
//! executed assertion that the real external example consumer must compile;
//! after implementation the same consumer must also satisfy its runtime cases.
//! Run the selected test through the repository's bounded process wrapper. The
//! nested Cargo command inherits that process group and has its own target
//! directory, while using the current product workspace and committed lockfile.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_component_value_consumer_compiles_and_preserves_tokens_and_origins() {
    let output = external_consumer::run_example("component_value_contract");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "token boundaries: ok\n",
            "checked construction: ok\n",
            "whitespace and closures: ok\n",
            "origins and limits: ok\n",
        )
    );
}
