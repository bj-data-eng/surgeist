//! Characterization snapshot of the public parser over the CSSTree corpus.
//!
//! `csstree/oracle.json` is captured from Surgeist's own parser, so matching it
//! proves only that observed behavior is unchanged; it is not a correctness
//! oracle, and re-capturing it is never evidence that a behavior change is
//! right. Correctness evidence comes from the independently established
//! `csstree/expected-classes.json` and reconciliation records and from the
//! upstream canonical comparison in `csstree_canonical.rs`.

use crate::css::support;

use support::csstree::{
    capture_csstree_oracle_from_public_parser as capture_oracle, regenerate_csstree_oracle,
};

#[test]
#[ignore = "writes the reviewed CSS oracle only when explicitly invoked with its exact gate"]
fn capture_csstree_oracle_from_public_parser() {
    capture_oracle()
        .expect("explicit public-parser capture should write and reload the CSS oracle");
}

#[test]
fn csstree_baseline_matches_committed_oracle() {
    let regenerated = regenerate_csstree_oracle()
        .expect("public CSS baseline should regenerate from the validated corpus");
    assert_eq!(regenerated, include_bytes!("../../../csstree/oracle.json"));
}
