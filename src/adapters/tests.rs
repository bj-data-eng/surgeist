use super::{AdapterBoundary, AdapterError, AdapterErrorKind};

#[test]
fn adapter_error_exposes_kind_and_boundary() {
    let error = AdapterError::new(AdapterErrorKind::StyleLayoutValue {
        property: "width".to_owned(),
        reason: "bad width".to_owned(),
    });

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(matches!(
        error.kind(),
        AdapterErrorKind::StyleLayoutValue { property, .. } if property == "width"
    ));
}
