#![forbid(unsafe_code)]

//! The outer test compiles before the expansion API exists. Its executed
//! assertion requires the real public consumer to compile and complete all
//! independent behavioral cases. The nested offline Cargo process inherits the
//! outer bounded process group and owns an isolated target directory.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_expansion_consumer_compiles_and_preserves_normalized_contribution_contracts() {
    let output = external_consumer::run_example("declaration_expansion_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "four-sided authored values and reset boundary: ok\n",
            "four-sided styles and current colors: ok\n",
            "side border defaults: ok\n",
            "full border image resets: ok\n",
            "ordinary longhands: ok\n",
            "global and universal resets: ok\n",
            "source and occurrence identity: ok\n",
            "strict reentry success and origins: ok\n",
            "strict rejection and unsupported identity: ok\n",
            "custom symbolic contributions: ok\n",
        )
    );
}
