use surgeist_html::{
    Attribute, DocumentString, HtmlCanvasBitmapDimension, HtmlCanvasNoneState, HtmlMarqueeState,
    HtmlNonNegativeInteger, HtmlValueInterpreter, HtmlValueRecoveryState, KnownHtmlAttribute,
    KnownHtmlElement,
};

fn integer(value: &str) -> HtmlNonNegativeInteger {
    HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(value)).unwrap()
}

fn dimensions(state: &HtmlCanvasNoneState) -> (String, String) {
    fn digits(value: HtmlCanvasBitmapDimension<'_>) -> String {
        match value {
            HtmlCanvasBitmapDimension::Default(value) => value.to_string(),
            HtmlCanvasBitmapDimension::Parsed(value) => value.decimal_digits().to_owned(),
        }
    }
    (digits(state.width()), digits(state.height()))
}

fn assert_none_bitmap(state: &HtmlCanvasNoneState) {
    assert!(state.context_is_none());
    assert!(!state.has_bound_context());
    assert!(state.origin_clean());
    assert!(state.bitmap_is_transparent_black());
}

// Literal values come from HTML #the-canvas-element, not the shared rule table.
#[test]
fn native_canvas_has_no_context_and_a_clean_transparent_300_by_150_bitmap() {
    let state = HtmlCanvasNoneState::fresh();
    assert_none_bitmap(&state);
    assert_eq!(dimensions(&state), ("300".into(), "150".into()));
    assert_eq!(state.width(), HtmlCanvasBitmapDimension::Default(300));
    assert_eq!(state.height(), HtmlCanvasBitmapDimension::Default(150));
}

#[test]
fn copied_attributes_initialize_zero_and_exact_unbounded_bitmap_dimensions() {
    let large = "99999999999999999999999999999999999999999999999999999999";
    let width = integer(large);
    let allocation = width.decimal_digits().as_ptr();
    let state = HtmlCanvasNoneState::from_parsed_attributes(Some(width), Some(integer("0")));
    assert_eq!(dimensions(&state), (large.into(), "0".into()));
    let HtmlCanvasBitmapDimension::Parsed(width) = state.width() else {
        panic!("actual large dimension must be retained");
    };
    assert_eq!(width.decimal_digits().as_ptr(), allocation);
    assert_none_bitmap(&state);
}

#[test]
fn bitmap_dimensions_are_not_unsigned_long_reflected_values() {
    // #reflect unsigned-long getters default above 2147483647. Bitmap sizing
    // instead follows the uncapped content-attribute integer in the canvas rule.
    let state = HtmlCanvasNoneState::from_parsed_attributes(
        Some(integer("2147483648")),
        Some(integer("4294967296")),
    );
    assert_eq!(
        dimensions(&state),
        ("2147483648".into(), "4294967296".into())
    );
    assert_none_bitmap(&state);
}

#[test]
fn explicit_defaults_have_the_same_dimension_state_as_absent_attributes() {
    let explicit = HtmlCanvasNoneState::from_parsed_attributes(
        Some(integer("000300")),
        Some(integer("000150")),
    );
    assert_eq!(explicit, HtmlCanvasNoneState::fresh());
    let unequal =
        HtmlCanvasNoneState::from_parsed_attributes(Some(integer("150")), Some(integer("300")));
    assert_eq!(dimensions(&unequal), ("150".into(), "300".into()));
}

#[test]
fn setting_changing_removing_and_redundantly_setting_dimensions_preserves_none_state() {
    let mut state = HtmlCanvasNoneState::fresh();
    state.width_attribute_changed(Some(integer("0")));
    assert_eq!(dimensions(&state), ("0".into(), "150".into()));
    state.height_attribute_changed(Some(integer("42")));
    assert_eq!(dimensions(&state), ("0".into(), "42".into()));
    assert_none_bitmap(&state);
    let same = state.clone();
    state.height_attribute_changed(Some(integer("42")));
    assert_eq!(state, same);
    state.width_attribute_changed(Some(integer("640")));
    assert_eq!(dimensions(&state), ("640".into(), "42".into()));
    state.width_attribute_changed(None);
    assert_eq!(dimensions(&state), ("300".into(), "42".into()));
    state.height_attribute_changed(None);
    assert_eq!(state, HtmlCanvasNoneState::fresh());
    assert_none_bitmap(&state);
}

#[test]
fn actual_canvas_attribute_recovery_supplies_prefix_values_or_dimension_defaults() {
    let interpreter = HtmlValueInterpreter::new(64);
    let canvas = KnownHtmlElement::Canvas.qualified_name();
    // Independent expectations exercise the existing scalar kernel used by an
    // adapter, including author-invalid recovery rather than author validation.
    for (width, height, expected) in [
        (None, None, ("300", "150")),
        (Some(""), Some("-1"), ("300", "150")),
        (Some("+0080tail"), Some(" 12.5"), ("80", "12")),
        (Some("0"), Some("0000"), ("0", "0")),
        (
            Some("2147483648"),
            Some("4294967296"),
            ("2147483648", "4294967296"),
        ),
    ] {
        let width =
            width.map(|value| Attribute::new(KnownHtmlAttribute::Width.qualified_name(), value));
        let height =
            height.map(|value| Attribute::new(KnownHtmlAttribute::Height.qualified_name(), value));
        let width = interpreter.canvas_width(&canvas, width.as_ref()).unwrap();
        let height = interpreter.canvas_height(&canvas, height.as_ref()).unwrap();
        let recovered = |state: &HtmlValueRecoveryState<HtmlNonNegativeInteger>| match state {
            HtmlValueRecoveryState::Value(value) => Some(value.clone()),
            HtmlValueRecoveryState::Missing | HtmlValueRecoveryState::Failed(_) => None,
        };
        let state = HtmlCanvasNoneState::from_parsed_attributes(
            recovered(width.common().unwrap().recovery()),
            recovered(height.common().unwrap().recovery()),
        );
        assert_eq!(dimensions(&state), (expected.0.into(), expected.1.into()));
        assert_none_bitmap(&state);
    }
}

#[test]
fn marquee_starts_on_and_start_stop_are_idempotent_independent_local_writes() {
    let mut first = HtmlMarqueeState::fresh();
    let mut second = HtmlMarqueeState::fresh();
    assert!(first.turned_on());
    first.stop();
    first.stop();
    assert!(!first.turned_on());
    assert!(second.turned_on());
    first.start();
    first.start();
    assert!(first.turned_on());
    second.stop();
    assert!(!second.turned_on());
    assert!(first.turned_on());
    assert!(HtmlMarqueeState::fresh().turned_on());
}
