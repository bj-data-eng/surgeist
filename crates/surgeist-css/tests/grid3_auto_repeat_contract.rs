#![forbid(unsafe_code)]

//! The outer test compiles against the existing public API and launches the
//! external consumer. Before the auto-repeat content return type is widened,
//! its failure is nested consumer compilation RED, not an executed behavioral
//! failure inside that consumer. The separate grammar target executes runtime
//! assertions against the existing API.
//! The nested offline Cargo process inherits the outer bounded process group
//! and exclusively owns its temporary target directory.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_auto_repeat_consumer_preserves_general_bodies_and_fixed_surroundings() {
    let output = external_consumer::run_example("grid3_auto_repeat_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "general automatic body and fixed surroundings: ok\n",
            "symbolic automatic calculation and line names: ok\n",
            "checked component grammar and preserved origins: ok\n",
            "explicit axes and implicit track roles: ok\n",
        )
    );
}
