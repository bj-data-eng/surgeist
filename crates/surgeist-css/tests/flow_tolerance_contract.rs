#![forbid(unsafe_code)]

//! This outer test executes against the existing public API and requires the
//! external consumer to compile and complete the independent behavioral cases.
//! Before the checked flow-tolerance API exists, its failure is nested consumer
//! compilation RED; the consumer's behavioral assertions have not run yet.
//! The nested offline Cargo process inherits the outer bounded process group
//! and exclusively owns its temporary target directory.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_flow_tolerance_consumer_preserves_checked_values_and_longhand_contributions() {
    let output = external_consumer::run_example("flow_tolerance_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "checked symbolic and signed values: ok\n",
            "checked calculation construction: ok\n",
            "independent parsed and constructed semantics: ok\n",
            "component construction and provenance: ok\n",
            "ordered symbolic longhand normalization: ok\n",
            "global values remain symbolic: ok\n",
            "strict pending reentry and preserved origins: ok\n",
            "canonical symbolic specified serialization: ok\n",
        )
    );
}
