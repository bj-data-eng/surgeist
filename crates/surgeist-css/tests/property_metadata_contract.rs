#![forbid(unsafe_code)]

//! This outer test executes against the existing public API and requires the
//! external consumer to compile and complete the independent behavioral cases.
//! Before the property metadata API exists, its failure is nested consumer
//! compilation RED; the consumer's behavioral assertions have not run yet.
//! The nested offline Cargo process inherits the outer bounded process group
//! and exclusively owns its temporary target directory.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_property_metadata_consumer_preserves_initials_and_grammar_identity() {
    let output = external_consumer::run_example("property_metadata_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "metadata recognition, inheritance and intrinsic initial values: ok\n",
            "terminal membership, omissions, reset-only values and all exclusions: ok\n",
            "legacy grammar identity and atomic reusable reentry: ok\n",
            "parsed and constructed provenance with ordered color normalization: ok\n",
        )
    );
}

// The integration target compiles the production library without cfg(test).
// Its child selects the real expansion-owned unit test under cfg(test), so a
// missing owner transition produces nested unit-compilation RED, never a claim
// that the not-yet-compiling owner assertions ran.
#[test]
fn expansion_owner_preserves_user_agent_initial_requirement() {
    let name = "expansion::metadata_initial_tests::omitted_initial_preserves_user_agent_requirement_and_fixed_value";
    let output = external_consumer::run_selected_library_test(name);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(&format!("test {name} ... ok")),
        "the exact selected owner test must execute: {stdout}"
    );
}
