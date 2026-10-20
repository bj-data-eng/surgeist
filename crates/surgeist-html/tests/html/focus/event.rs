//! Public source-body behavior from HTML877d raw60303–60311. The small
//! adapter below owns operands and captures; it supplies no native focus proof.
use surgeist_html::{
    HtmlFocusEventAction as A, HtmlFocusEventCursor as C, HtmlFocusEventError as E,
    HtmlFocusEventInterface as Interface, HtmlFocusEventName, HtmlFocusEventReply as R,
    HtmlFocusEventTargetKind as Target, HtmlFocusEventView as View,
};

fn step(cursor: &mut C, control: &mut C, action: A, reply: R) {
    assert_eq!(cursor.action(), action);
    let wrong = match reply {
        R::Applied => R::Dispatched,
        R::Dispatched => R::Applied,
    };
    assert_eq!(cursor.reply(wrong), Err(E::WrongReply));
    assert_eq!(cursor, control);
    cursor.reply(reply).unwrap();
    control.reply(reply).unwrap();
    assert_eq!(cursor, control);
}

#[derive(Debug, Eq, PartialEq)]
struct Event<'a> {
    name: &'a str,
    target: u8,
    related: Option<u8>,
    view: u8,
    interface: Interface,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
}

fn event<'a>(
    kind: Target,
    name: &'a str,
    target: u8,
    related: Option<u8>,
    element_document_global: u8,
) -> Event<'a> {
    let mut cursor = C::new(kind);
    let mut control = C::new(kind);
    let (capture, view_role, view) = match kind {
        Target::Element => (
            A::CaptureElementDocumentGlobalView,
            View::ElementDocumentGlobal,
            element_document_global,
        ),
        Target::Window => (A::CaptureWindowView, View::TargetWindow, target),
    };
    step(&mut cursor, &mut control, capture, R::Applied);
    let A::Dispatch {
        interface,
        view: actual_view,
        bubbles,
        cancelable,
        composed,
    } = cursor.action()
    else {
        panic!("view capture must precede the actual event request")
    };
    assert_eq!(actual_view, view_role);
    let result = Event {
        name,
        target,
        related,
        view,
        interface,
        bubbles,
        cancelable,
        composed,
    };
    let action = cursor.action();
    step(&mut cursor, &mut control, action, R::Dispatched);
    assert_eq!(cursor.action(), A::Complete);
    for reply in [R::Applied, R::Dispatched] {
        assert_eq!(cursor.reply(reply), Err(E::Terminal));
        assert_eq!(cursor, control);
    }
    result
}

#[test]
fn element_event_uses_its_current_document_global_and_source_focus_event_defaults() {
    let actual = event(Target::Element, "focus", 7, Some(9), 42);
    assert_eq!(
        actual,
        Event {
            name: "focus",
            target: 7,
            related: Some(9),
            view: 42,
            interface: Interface::FocusEvent,
            bubbles: false,
            cancelable: false,
            composed: true,
        }
    );
}

#[test]
fn window_event_uses_that_same_window_as_view_without_an_element_document_capture() {
    let actual = event(Target::Window, "blur", 51, None, 99);
    assert_eq!(actual.view, 51);
    assert_eq!(actual.target, 51);
    assert_eq!(actual.related, None);
    assert_eq!(actual.interface, Interface::FocusEvent);
    assert!(!actual.bubbles);
    assert!(!actual.cancelable);
    assert!(actual.composed);
}

#[test]
fn event_name_and_nullable_related_operand_remain_the_original_callers_values() {
    assert_eq!(HtmlFocusEventName::Blur.as_str(), "blur");
    assert_eq!(HtmlFocusEventName::Focus.as_str(), "focus");
    for name in ["focus", "blur", "custom-focus-event"] {
        for related in [None, Some(13)] {
            let actual = event(Target::Element, name, 2, related, 8);
            assert_eq!(actual.name, name);
            assert_eq!(actual.related, related);
        }
    }
}
