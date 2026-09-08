use surgeist_style::{
    InvalidAtComputedValueReason, Property, StyleDiagnostic, StyleDiagnosticKind,
    StyleDiagnosticSubject,
};

fn main() {
    let _diagnostic = StyleDiagnostic {
        kind: StyleDiagnosticKind::InvalidAtComputedValue,
        subject: StyleDiagnosticSubject::Property(Property::Color),
        source: None,
        reason: InvalidAtComputedValueReason::MissingTypedCustomPropertyValue(
            surgeist_style::CustomPropertyName::try_new("--brand").unwrap(),
            Property::Color,
        ),
    };
}
