#![forbid(unsafe_code)]

//! This outer test executes against the existing public API and requires the
//! external consumer to compile and complete the independent behavioral cases.
//! Before the selector/query fragment API exists, its failure is nested consumer
//! compilation RED; the consumer's behavioral assertions have not run yet.
//! The nested offline Cargo process inherits the outer bounded process group
//! and exclusively owns its temporary target directory.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_selector_query_fragment_consumer_preserves_admission_and_recovery() {
    let output = external_consumer::run_example("selector_query_fragment_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        concat!(
            "fragment signatures and immutable namespace context: ok\n",
            "selector exact admission and authored semantics: ok\n",
            "selector forgiving recovery and actual EOF provenance: ok\n",
            "media exact admission and member recovery: ok\n",
            "media actual EOF and source coordinates: ok\n",
            "fragment 128/129 and 255/256/257 depth plus EOF: ok\n",
        )
    );
}
