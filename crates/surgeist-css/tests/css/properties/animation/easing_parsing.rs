use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction, CssTokenKind};
use surgeist_css::property::CssKnownProperty;

struct InvalidEasingCase {
    source: &'static str,
    property: CssKnownProperty,
    position_marker: &'static str,
    position_offset: usize,
    encountered_kind: CssTokenKind,
    encountered_authored: &'static str,
}

fn assert_invalid_easing_is_dropped(case: &InvalidEasingCase) {
    let report = CssParser::default().parse_block_contents(case.source);
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
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "{}: expected typed property-value detail",
        case.source
    );
    assert_eq!(
        detail.known_property().unwrap(),
        case.property,
        "{}",
        case.source
    );
    assert_eq!(
        detail.expectation().unwrap().as_str(),
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

    {
        let failure = surgeist_css::CssParser::default()
            .parse_block_contents(case.source)
            .into_validation_result()
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
            position_marker: "2, -20",
            position_offset: 0,
            encountered_kind: CssTokenKind::Number,
            encountered_authored: "2",
        },
        InvalidEasingCase {
            source: "animation: fade 1s cubic-bezier(0, -20, -1, 20); color: red",
            property: CssKnownProperty::Animation,
            position_marker: "-1, 20",
            position_offset: 0,
            encountered_kind: CssTokenKind::Number,
            encountered_authored: "-1",
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
            position_marker: "jump-none",
            position_offset: 0,
            encountered_kind: CssTokenKind::Ident,
            encountered_authored: "jump-none",
        },
        InvalidEasingCase {
            source: "animation-timing-function: steps(1, JUMP-NONE); color: red",
            property: CssKnownProperty::AnimationTimingFunction,
            position_marker: "JUMP-NONE",
            position_offset: 0,
            encountered_kind: CssTokenKind::Ident,
            encountered_authored: "JUMP-NONE",
        },
        InvalidEasingCase {
            source: "transition: opacity 1s steps(1, jump-none); color: red",
            property: CssKnownProperty::Transition,
            position_marker: "jump-none",
            position_offset: 0,
            encountered_kind: CssTokenKind::Ident,
            encountered_authored: "jump-none",
        },
        InvalidEasingCase {
            source: "animation: fade 1s steps(1, jump-none); color: red",
            property: CssKnownProperty::Animation,
            position_marker: "jump-none",
            position_offset: 0,
            encountered_kind: CssTokenKind::Ident,
            encountered_authored: "jump-none",
        },
    ] {
        assert_invalid_easing_is_dropped(&case);
    }
}
