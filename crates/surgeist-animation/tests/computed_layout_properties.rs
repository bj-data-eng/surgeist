use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedInteger, ComputedNumeric, ComputedValue,
    DimensionKind, DimensionPercentage,
};
use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableNumber, InterpolableValue, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue, SampledProperty};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};

#[test]
fn flex_longhands_sample_factors_and_mixed_basis_independently() {
    // Style has already expanded flex into its three longhands.
    let track = track([
        ("flex-grow", computed_number(1.0), computed_number(5.0)),
        ("flex-shrink", number(6.0), number(2.0)),
        ("flex-basis", length(20.0), percentage(0.4)),
    ]);

    assert_frame(
        &track,
        0.25,
        [
            ("flex-grow", computed_number(2.0)),
            ("flex-shrink", number(5.0)),
            ("flex-basis", length_percentage(15.0, 0.1)),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("flex-grow", computed_number(3.0)),
            ("flex-shrink", number(4.0)),
            ("flex-basis", length_percentage(10.0, 0.2)),
        ],
    );
}

#[test]
fn flex_basis_keyword_and_flow_keywords_switch_at_the_midpoint() {
    let track = track([
        ("flex-basis", computed_keyword("content"), length(12.0)),
        ("flex-direction", keyword("row"), keyword("column-reverse")),
        ("flex-wrap", keyword("nowrap"), keyword("wrap-reverse")),
    ]);

    assert_frame(
        &track,
        0.5_f64.next_down(),
        [
            ("flex-basis", computed_keyword("content")),
            ("flex-direction", keyword("row")),
            ("flex-wrap", keyword("nowrap")),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("flex-basis", length(12.0)),
            ("flex-direction", keyword("column-reverse")),
            ("flex-wrap", keyword("wrap-reverse")),
        ],
    );
}

#[test]
fn alignment_content_self_and_items_longhands_switch_as_computed_keywords() {
    // Each place shorthand contributes its paired align and justify longhands.
    let endpoints = [
        ("align-content", "space-between", "space-around"),
        ("justify-content", "flex-start", "space-evenly"),
        ("align-self", "start", "self-end"),
        ("justify-self", "center", "stretch"),
        ("align-items", "baseline", "stretch"),
        ("justify-items", "start", "end"),
    ];
    let track = track(endpoints.map(|(property, from, to)| (property, keyword(from), keyword(to))));

    assert_frame(
        &track,
        0.5_f64.next_down(),
        endpoints.map(|(property, from, _)| (property, keyword(from))),
    );
    assert_frame(
        &track,
        0.5,
        endpoints.map(|(property, _, to)| (property, keyword(to))),
    );
}

#[test]
fn gap_samples_keep_unresolved_percentages_and_switch_normal_discretely() {
    let track = track([
        ("row-gap", computed_keyword("normal"), length(12.0)),
        ("column-gap", length(8.0), percentage(0.25)),
    ]);

    assert_frame(
        &track,
        0.25,
        [
            ("row-gap", computed_keyword("normal")),
            ("column-gap", length_percentage(6.0, 0.0625)),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("row-gap", length(12.0)),
            ("column-gap", length_percentage(4.0, 0.125)),
        ],
    );
}

#[test]
fn order_uses_computed_integer_rounding() {
    let track = track([("order", integer(-3.0), integer(0.0))]);

    assert_frame(&track, 0.25, [("order", integer(-2.0))]);
    assert_frame(&track, 0.5, [("order", integer(-1.0))]);
}

#[test]
fn visibility_uses_the_discrete_type_for_visible_hidden_and_collapse() {
    for (from, to) in [("visible", "collapse"), ("hidden", "visible")] {
        let track = track([("visibility", keyword(from), keyword(to))]);

        assert_frame(&track, 0.5_f64.next_down(), [("visibility", keyword(from))]);
        assert_frame(&track, 0.5, [("visibility", keyword(to))]);
    }
}

#[test]
fn position_switches_while_inset_longhands_use_their_computed_types() {
    let track = track([
        ("position", keyword("absolute"), keyword("sticky")),
        ("top", length(16.0), percentage(0.5)),
        ("right", computed_keyword("auto"), length(8.0)),
        ("bottom", percentage(0.125), percentage(0.375)),
        ("left", length(-2.0), length(6.0)),
    ]);

    assert_frame(
        &track,
        0.25,
        [
            ("position", keyword("absolute")),
            ("top", length_percentage(12.0, 0.125)),
            ("right", computed_keyword("auto")),
            ("bottom", percentage(0.1875)),
            ("left", length(0.0)),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("position", keyword("sticky")),
            ("top", length_percentage(8.0, 0.25)),
            ("right", length(8.0)),
            ("bottom", percentage(0.25)),
            ("left", length(2.0)),
        ],
    );
}

#[test]
fn expanded_logical_inset_longhands_keep_their_own_values() {
    let track = track([
        ("inset-block-start", length(16.0), percentage(0.5)),
        ("inset-block-end", length(8.0), computed_keyword("auto")),
        ("inset-inline-start", length(-16.0), length(-8.0)),
        ("inset-inline-end", percentage(0.125), percentage(0.375)),
    ]);

    assert_frame(
        &track,
        0.5,
        [
            ("inset-block-start", length_percentage(8.0, 0.25)),
            ("inset-block-end", computed_keyword("auto")),
            ("inset-inline-start", length(-12.0)),
            ("inset-inline-end", percentage(0.25)),
        ],
    );
}

#[test]
fn break_and_decoration_keywords_switch_discretely() {
    let endpoints = [
        ("break-before", "auto", "page"),
        ("break-after", "avoid-page", "column"),
        ("break-inside", "auto", "avoid"),
        ("box-decoration-break", "slice", "clone"),
    ];
    let track = track(endpoints.map(|(property, from, to)| (property, keyword(from), keyword(to))));

    assert_frame(
        &track,
        0.5_f64.next_down(),
        endpoints.map(|(property, from, _)| (property, keyword(from))),
    );
    assert_frame(
        &track,
        0.5,
        endpoints.map(|(property, _, to)| (property, keyword(to))),
    );
}

#[test]
fn widows_and_orphans_use_computed_integer_interpolation() {
    let track = track([
        ("widows", integer(2.0), integer(6.0)),
        ("orphans", integer(3.0), integer(4.0)),
    ]);

    assert_frame(
        &track,
        0.25,
        [("widows", integer(3.0)), ("orphans", integer(3.0))],
    );
    assert_frame(
        &track,
        0.5,
        [("widows", integer(4.0)), ("orphans", integer(4.0))],
    );
}

#[test]
fn generated_content_and_bookmark_payloads_are_discrete() {
    let endpoints = [
        ("content", "none", r#""Heading""#),
        ("quotes", "none", r#""«" "»""#),
        ("string-set", "none", r#"chapter "Heading""#),
        ("bookmark-label", "content(text)", r#""Overview""#),
        ("bookmark-state", "open", "closed"),
    ];
    let track = track(endpoints.map(|(property, from, to)| (property, keyword(from), keyword(to))));

    assert_frame(
        &track,
        0.5_f64.next_down(),
        endpoints.map(|(property, from, _)| (property, keyword(from))),
    );
    assert_frame(
        &track,
        0.5,
        endpoints.map(|(property, _, to)| (property, keyword(to))),
    );
}

#[test]
fn bookmark_levels_interpolate_integers_and_switch_none_discretely() {
    let numbered = track([("bookmark-level", integer(1.0), integer(4.0))]);
    assert_frame(&numbered, 0.25, [("bookmark-level", integer(2.0))]);
    assert_frame(&numbered, 0.5, [("bookmark-level", integer(3.0))]);

    let from_none = track([("bookmark-level", computed_keyword("none"), integer(4.0))]);
    assert_frame(
        &from_none,
        0.5_f64.next_down(),
        [("bookmark-level", computed_keyword("none"))],
    );
    assert_frame(&from_none, 0.5, [("bookmark-level", integer(4.0))]);
}

#[test]
fn discrete_text_longhands_keep_their_effective_keywords() {
    // The caller supplies effective longhands from the text-align, text-wrap,
    // white-space and text-spacing shorthands.
    let endpoints = [
        ("hanging-punctuation", "none", "first last"),
        ("hyphenate-character", "auto", r#""~""#),
        ("hyphenate-limit-last", "none", "page"),
        ("hyphens", "manual", "auto"),
        ("line-break", "loose", "strict"),
        ("overflow-wrap", "normal", "anywhere"),
        ("text-align-all", "start", "justify"),
        ("text-align-last", "auto", "center"),
        ("text-autospace", "normal", "auto"),
        ("text-group-align", "none", "end"),
        ("text-justify", "auto", "inter-word"),
        ("text-spacing-trim", "auto", "space-all"),
        ("text-transform", "none", "uppercase full-width"),
        ("text-wrap-mode", "wrap", "nowrap"),
        ("text-wrap-style", "auto", "balance"),
        ("white-space-collapse", "collapse", "preserve"),
        ("white-space-trim", "none", "discard-before"),
        ("word-break", "normal", "keep-all"),
        ("word-space-transform", "none", "ideographic-space"),
        ("wrap-after", "auto", "avoid-line"),
        ("wrap-before", "auto", "line"),
        ("wrap-inside", "auto", "avoid"),
    ];
    let track = track(endpoints.map(|(property, from, to)| (property, keyword(from), keyword(to))));

    assert_frame(
        &track,
        0.5_f64.next_down(),
        endpoints.map(|(property, from, _)| (property, keyword(from))),
    );
    assert_frame(
        &track,
        0.5,
        endpoints.map(|(property, _, to)| (property, keyword(to))),
    );
}

#[test]
fn computed_text_components_sample_lengths_percentages_numbers_and_integers() {
    let hyphenate_chars = |word, after| {
        record(vec![
            integer_component(word),
            token_component("auto"),
            integer_component(after),
        ])
    };
    let indent = |value, flags| {
        record(vec![
            ComputedComponent::Numeric(value),
            token_component(flags),
        ])
    };
    let track = track([
        (
            "hyphenate-limit-chars",
            hyphenate_chars(5.0, 2.0),
            hyphenate_chars(9.0, 4.0),
        ),
        ("hyphenate-limit-lines", integer(2.0), integer(6.0)),
        ("hyphenate-limit-zone", length(6.0), percentage(0.25)),
        ("letter-spacing", length(-2.0), percentage(0.125)),
        ("line-padding", length(2.0), length(6.0)),
        ("tab-size", computed_number(2.0), computed_number(5.0)),
        (
            "text-indent",
            indent(mixed_length(20.0, 0.0), "hanging"),
            indent(mixed_length(0.0, 0.5), "hanging"),
        ),
        ("word-spacing", length(-4.0), percentage(0.5)),
    ]);

    assert_frame(
        &track,
        0.5,
        [
            ("hyphenate-limit-chars", hyphenate_chars(7.0, 3.0)),
            ("hyphenate-limit-lines", integer(4.0)),
            ("hyphenate-limit-zone", length_percentage(3.0, 0.125)),
            ("letter-spacing", length_percentage(-1.0, 0.0625)),
            ("line-padding", length(4.0)),
            ("tab-size", computed_number(3.5)),
            ("text-indent", indent(mixed_length(10.0, 0.25), "hanging")),
            ("word-spacing", length_percentage(-2.0, 0.25)),
        ],
    );
}

#[test]
fn incomparable_text_components_switch_the_whole_value() {
    let indent = |pixels, flags| {
        record(vec![
            ComputedComponent::Numeric(mixed_length(pixels, 0.0)),
            token_component(flags),
        ])
    };
    let track = track([
        (
            "hyphenate-limit-lines",
            computed_keyword("no-limit"),
            integer(5.0),
        ),
        ("tab-size", computed_number(4.0), length(20.0)),
        (
            "text-indent",
            indent(10.0, "hanging"),
            indent(30.0, "each-line"),
        ),
    ]);
    assert_frame(
        &track,
        0.5_f64.next_down(),
        [
            ("hyphenate-limit-lines", computed_keyword("no-limit")),
            ("tab-size", computed_number(4.0)),
            ("text-indent", indent(10.0, "hanging")),
        ],
    );
    assert_frame(
        &track,
        0.5,
        [
            ("hyphenate-limit-lines", integer(5.0)),
            ("tab-size", length(20.0)),
            ("text-indent", indent(30.0, "each-line")),
        ],
    );
}

#[test]
fn corresponding_font_variation_axes_interpolate_with_finite_extrapolation() {
    let from = variation_axes([("GRAD", 5.0), ("wdth", 80.0), ("wght", 200.0)]);
    let to = variation_axes([("GRAD", -15.0), ("wdth", 100.0), ("wght", 350.0)]);
    let property = "font-variation-settings";
    let track = track([(property, from.clone(), to.clone())]);
    assert_frame(
        &track,
        0.5,
        [(
            property,
            variation_axes([("GRAD", -5.0), ("wdth", 90.0), ("wght", 275.0)]),
        )],
    );
    let pair = InterpolationPair::new(PropertyKey::from_text(property).unwrap(), from, to).unwrap();
    assert_eq!(
        pair.sample(InterpolationProgress::new(-0.5).unwrap()),
        InterpolationOutcome::Value(variation_axes([
            ("GRAD", 15.0),
            ("wdth", 70.0),
            ("wght", 125.0)
        ])),
    );
    assert_eq!(
        pair.sample(InterpolationProgress::new(1.5).unwrap()),
        InterpolationOutcome::Value(variation_axes([
            ("GRAD", -25.0),
            ("wdth", 110.0),
            ("wght", 425.0)
        ])),
    );
}

#[test]
fn incompatible_variation_maps_and_feature_settings_switch_discretely() {
    let property = "font-variation-settings";
    let from = variation_axes([("wght", 300.0)]);
    for to in [
        variation_axes([("WGHT", 700.0)]),
        variation_axes([("wdth", 100.0), ("wght", 700.0)]),
        computed_keyword("normal"),
    ] {
        let track = track([
            (property, from.clone(), to.clone()),
            (
                "font-feature-settings",
                keyword(r#""liga" 0"#),
                keyword(r#""liga" 1"#),
            ),
        ]);
        assert_frame(
            &track,
            0.5_f64.next_down(),
            [
                (property, from.clone()),
                ("font-feature-settings", keyword(r#""liga" 0"#)),
            ],
        );
        assert_frame(
            &track,
            0.5,
            [
                (property, to),
                ("font-feature-settings", keyword(r#""liga" 1"#)),
            ],
        );
    }
}

#[test]
fn comparable_font_style_angles_and_fractional_weights_are_preserved() {
    let angle = |degrees| {
        numeric(ComputedNumeric::dimension(
            ComputedDimension::new(DimensionKind::Angle, degrees).unwrap(),
        ))
    };
    // Style supplies normal as the comparable oblique zero-degree input.
    let track = track([
        ("font-style", angle(0.0), angle(20.0)),
        (
            "font-weight",
            computed_number(380.0),
            computed_number(465.0),
        ),
    ]);
    assert_frame(
        &track,
        0.5,
        [
            ("font-style", angle(10.0)),
            ("font-weight", computed_number(422.5)),
        ],
    );
}

fn variation_axes<const N: usize>(axes: [(&str, f64); N]) -> InterpolableValue {
    // The computed map is already deduplicated and sorted by the caller.
    record(
        axes.into_iter()
            .flat_map(|(tag, value)| {
                [
                    token_component(tag),
                    ComputedComponent::Numeric(ComputedNumeric::number(value).unwrap()),
                ]
            })
            .collect(),
    )
}

fn record(components: Vec<ComputedComponent>) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::new(components).unwrap())
}

fn token_component(value: &str) -> ComputedComponent {
    ComputedComponent::Discrete(DiscreteValue::from_text(value).unwrap())
}

fn track<const N: usize>(
    endpoints: [(&str, InterpolableValue, InterpolableValue); N],
) -> KeyframeTrack {
    let timing = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::None,
    )
    .unwrap();
    let segments = endpoints
        .into_iter()
        .map(|(property, from, to)| {
            KeyframeSegment::new(
                PropertyKey::new(property).unwrap(),
                UnitRatio::new(0.0).unwrap(),
                UnitRatio::new(1.0).unwrap(),
                from,
                to,
                Easing::linear(),
            )
            .unwrap()
        })
        .collect();
    KeyframeTrack::new(
        KeyframeAnimationId::new("computed-layout-properties").unwrap(),
        timing,
        segments,
    )
    .unwrap()
}

fn assert_frame<const N: usize>(
    track: &KeyframeTrack,
    elapsed: f64,
    expected: [(&str, InterpolableValue); N],
) {
    let frame = track.sample(
        ElapsedTime::from_secs(elapsed).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(frame.len(), expected.len());
    for (sample, (property, value)) in frame.iter().zip(expected) {
        assert_eq!(sample.classification(), SampleClassification::Active);
        assert_eq!(
            sample.value(),
            &SampleValue::Value(SampledProperty::new(
                PropertyKey::from_text(property).unwrap(),
                value
            )),
            "{property} at {elapsed}"
        );
    }
}

fn integer(value: f64) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(integer_component(value)))
}

fn integer_component(value: f64) -> ComputedComponent {
    ComputedComponent::Numeric(ComputedNumeric::integer(
        ComputedInteger::new(value).unwrap(),
    ))
}

fn number(value: f64) -> InterpolableValue {
    InterpolableValue::number(InterpolableNumber::new(value).unwrap())
}

fn computed_number(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::number(value).unwrap())
}

fn length(value: f64) -> InterpolableValue {
    numeric(ComputedNumeric::dimension(
        ComputedDimension::new(DimensionKind::Length, value).unwrap(),
    ))
}

fn percentage(ratio: f64) -> InterpolableValue {
    numeric(ComputedNumeric::percentage(ratio).unwrap())
}

fn length_percentage(pixels: f64, ratio: f64) -> InterpolableValue {
    numeric(mixed_length(pixels, ratio))
}

fn mixed_length(pixels: f64, ratio: f64) -> ComputedNumeric {
    ComputedNumeric::mix(
        DimensionPercentage::new(
            ComputedDimension::new(DimensionKind::Length, pixels).unwrap(),
            ratio,
        )
        .unwrap(),
    )
}

fn numeric(value: ComputedNumeric) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Numeric(value)))
}

fn keyword(value: &str) -> InterpolableValue {
    InterpolableValue::discrete(DiscreteValue::from_text(value).unwrap())
}

fn computed_keyword(value: &str) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::single(ComputedComponent::Discrete(
        DiscreteValue::from_text(value).unwrap(),
    )))
}
