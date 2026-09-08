mod support;

use support::csstree::{
    capture_csstree_oracle_from_public_parser as capture_oracle, regenerate_csstree_oracle,
    validate_shared_support_surface,
};

#[test]
#[ignore = "writes the reviewed CSS oracle only when explicitly invoked with its exact gate"]
fn capture_csstree_oracle_from_public_parser() {
    capture_oracle()
        .expect("explicit public-parser capture should write and reload the CSS oracle");
}

#[test]
fn csstree_baseline_matches_committed_oracle() {
    assert!(validate_shared_support_surface());
    let regenerated = regenerate_csstree_oracle()
        .expect("public CSS baseline should regenerate from the validated corpus");
    assert_eq!(regenerated, include_bytes!("csstree/oracle.json"));
}
