//! `Display` and `source` of the semantic graph validation failure.

use super::*;
use std::error::Error as _;

fn resource() -> SemanticResourceId {
    SemanticResourceId::new(GraphGeneration(7), ResourceIndex(3))
}

#[test]
fn a_validation_failure_prints_the_text_that_graph_build_embeds_in_the_render_failure() {
    let failure = GraphValidationError::DeclaredReadCountMismatch {
        resource: resource(),
        declared: 2,
        recorded: 3,
    };
    assert_eq!(
        failure.to_string(),
        "resource 3 declares 2 reads but the graph records 3"
    );
    let error = graph_build::<()>(Err(failure)).unwrap_err();
    assert_eq!(
        error.message(),
        "semantic frame graph validation failed: resource 3 declares 2 reads but the graph records 3"
    );
    // The text is in the message and not repeated as a source.
    assert!(error.source().is_none());
}

#[test]
fn only_an_accounting_failure_has_a_cause_and_it_is_returned_unchanged() {
    assert!(
        GraphValidationError::UnknownResource(resource())
            .source()
            .is_none()
    );
    let accounting = GraphValidationError::Accounting(Error::invalid_value("x", 1, "must be 2"));
    assert_eq!(accounting.to_string(), "x value 1 is invalid: must be 2");
    assert_eq!(
        accounting.source().unwrap().to_string(),
        "x value 1 is invalid: must be 2"
    );
}
