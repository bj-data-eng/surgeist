#![forbid(unsafe_code)]

//! This outer test executes against the existing public API and requires the
//! external consumer to compile and complete the independent behavioral cases.
//! Before the media-grouping semantic API exists, its failure is nested consumer
//! compilation RED; the consumer's behavioral assertions have not run yet.
//! The nested offline Cargo process inherits the outer bounded process group
//! and exclusively owns its temporary target directory.

#[path = "support/external_consumer.rs"]
mod external_consumer;

#[test]
fn public_media_grouping_semantic_consumer_preserves_operators_groups_and_feature_grammar() {
    let output = external_consumer::run_example("media_grouping_semantic_consumer");
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 consumer output"),
        "media grouping semantic contract: ok\n"
    );
}
