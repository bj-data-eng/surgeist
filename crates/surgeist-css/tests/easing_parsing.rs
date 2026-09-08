use surgeist_css::{
    CssErrorCode, CssKnownProperty, CssRecoveryAction, CssTokenKind, ErrorKind,
    parse_style_attribute,
};

struct InvalidEasingCase {
    source: &'static str,
    property: CssKnownProperty,
    position_marker: &'static str,
    position_offset: usize,
    encountered_kind: CssTokenKind,
    encountered_authored: &'static str,
}

fn assert_invalid_easing_is_dropped(case: &InvalidEasingCase) {
    let report = parse_style_attribute(case.source);
    let [declaration] = report.syntax().as_slice() else {
        panic!(
            "{}: invalid easing must be dropped while the valid sibling is retained; syntax={:?}",
            case.source,
            report.syntax(),
        );
    };
    assert_eq!(
        declaration
            .known()
            .expect("retained color declaration")
            .property(),
        CssKnownProperty::Color,
        "{}",
        case.source,
    );

    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{}: expected one exact invalid-easing diagnostic; diagnostics={:?}",
            case.source,
            report.diagnostics(),
        );
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue,
        "{}",
        case.source,
    );
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{}",
        case.source,
    );
    let responsible = case
        .source
        .find(case.position_marker)
        .expect("responsible easing token")
        + case.position_offset;
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        case.source.find(';').expect("declaration end") + 1,
        "{}",
        case.source,
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("{}: expected typed property-value detail", case.source);
    };
    assert_eq!(detail.property(), case.property, "{}", case.source);
    assert_eq!(
        detail.expectation().as_str(),
        "a value accepted by the property's grammar",
        "{}",
        case.source,
    );
    let encountered = detail
        .encountered()
        .expect("responsible easing argument token");
    assert_eq!(encountered.kind(), case.encountered_kind, "{}", case.source);
    assert_eq!(
        encountered.authored(),
        case.encountered_authored,
        "{}",
        case.source
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(case.source)
            .expect_err("strict validation must reject the recovered easing declaration");
        assert_eq!(
            failure.diagnostics(),
            report.diagnostics(),
            "{}",
            case.source
        );
    }
}

#[test]
fn cubic_bezier_rejects_x_coordinates_outside_the_closed_unit_interval() {
    for case in [
        InvalidEasingCase {
            source: "transition-timing-function: cubic-bezier(-0.01, 0, 0.5, 1); color: red",
            property: CssKnownProperty::TransitionTimingFunction,
            position_marker: "-0.01",
            position_offset: 0,
            encountered_kind: CssTokenKind::Number,
            encountered_authored: "-0.01",
        },
        InvalidEasingCase {
            source: "animation-timing-function: cubic-bezier(0.5, -20, 1.01, 20); color: red",
            property: CssKnownProperty::AnimationTimingFunction,
            position_marker: "1.01",
            position_offset: 0,
            encountered_kind: CssTokenKind::Number,
            encountered_authored: "1.01",
        },
        InvalidEasingCase {
            source: "transition: opacity 1s cubic-bezier(2, -20, 0, 20); color: red",
            property: CssKnownProperty::Transition,
            position_marker: "cubic-bezier",
            position_offset: 0,
            encountered_kind: CssTokenKind::Function,
            encountered_authored: "cubic-bezier(",
        },
        InvalidEasingCase {
            source: "animation: fade 1s cubic-bezier(0, -20, -1, 20); color: red",
            property: CssKnownProperty::Animation,
            position_marker: "cubic-bezier",
            position_offset: 0,
            encountered_kind: CssTokenKind::Function,
            encountered_authored: "cubic-bezier(",
        },
    ] {
        assert_invalid_easing_is_dropped(&case);
    }
}

#[test]
fn steps_jump_none_rejects_a_single_interval() {
    for case in [
        InvalidEasingCase {
            source: "transition-timing-function: steps(1, jump-none); color: red",
            property: CssKnownProperty::TransitionTimingFunction,
            position_marker: "steps(1",
            position_offset: "steps(".len(),
            encountered_kind: CssTokenKind::Number,
            encountered_authored: "1",
        },
        InvalidEasingCase {
            source: "animation-timing-function: steps(1, JUMP-NONE); color: red",
            property: CssKnownProperty::AnimationTimingFunction,
            position_marker: "steps(1",
            position_offset: "steps(".len(),
            encountered_kind: CssTokenKind::Number,
            encountered_authored: "1",
        },
        InvalidEasingCase {
            source: "transition: opacity 1s steps(1, jump-none); color: red",
            property: CssKnownProperty::Transition,
            position_marker: "steps",
            position_offset: 0,
            encountered_kind: CssTokenKind::Function,
            encountered_authored: "steps(",
        },
        InvalidEasingCase {
            source: "animation: fade 1s steps(1, jump-none); color: red",
            property: CssKnownProperty::Animation,
            position_marker: "steps",
            position_offset: 0,
            encountered_kind: CssTokenKind::Function,
            encountered_authored: "steps(",
        },
    ] {
        assert_invalid_easing_is_dropped(&case);
    }
}
