#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedNumeric, ComputedValue, DimensionKind,
    DimensionPercentage,
};
use surgeist_animation::interpolation::{
    ColorInterpolationSpace, DiscreteValue, InterpolableColor, InterpolableValue,
    InterpolationOutcome, InterpolationPair, InterpolationProgress, PropertyKey,
};
use surgeist_animation::progress::UnitRatio;

#[test]
fn decoration_and_emphasis_keyword_sets_switch_whole() {
    // Style supplies the effective longhands of the decoration, emphasis and skip shorthands.
    let endpoints = [
        ("text-decoration-line", "underline overline", "line-through"),
        ("text-decoration-style", "dotted", "wavy"),
        ("text-underline-position", "from-font left", "under right"),
        ("text-decoration-skip-self", "none", "objects"),
        ("text-decoration-skip-box", "none", "all"),
        ("text-decoration-skip-inset", "none", "auto"),
        ("text-decoration-skip-spaces", "start end", "none"),
        ("text-decoration-skip-ink", "auto", "all"),
        ("text-emphasis-style", "open sesame", r#""※""#),
        ("text-emphasis-position", "over right", "under left"),
        ("text-emphasis-skip", "spaces punctuation", "symbols narrow"),
    ];
    for (property, from, to) in endpoints {
        switches(property, discrete(from), discrete(to));
    }
}

#[test]
fn decoration_lengths_sample_computed_values_and_switch_keywords() {
    for (property, from, to, midpoint, extrapolated) in [
        ("text-decoration-thickness", 2.0, 6.0, 4.0, 8.0),
        ("text-underline-offset", -4.0, 8.0, 2.0, 14.0),
    ] {
        samples(
            property,
            computed([length(from)]),
            computed([length(to)]),
            [
                (0.5, computed([length(midpoint)])),
                (1.5, computed([length(extrapolated)])),
            ],
        );
        switches(property, computed([tag("auto")]), computed([length(to)]));
    }
    switches(
        "text-decoration-thickness",
        computed([tag("from-font")]),
        computed([tag("auto")]),
    );
}

#[test]
fn decoration_background_and_border_colors_use_prepared_premultiplied_endpoints() {
    let blue = |alpha| {
        InterpolableValue::color(
            InterpolableColor::from_straight_components(
                ColorInterpolationSpace::Srgb,
                0.0,
                0.0,
                1.0,
                UnitRatio::new(alpha).unwrap(),
            )
            .unwrap(),
        )
    };
    for property in [
        "text-decoration-color",
        "text-emphasis-color",
        "background-color",
        "border-top-color",
        "border-right-color",
        "border-bottom-color",
        "border-left-color",
    ] {
        samples(property, blue(0.0), blue(1.0), [(0.5, blue(0.5))]);
    }
}

#[test]
fn background_layer_lists_and_border_keywords_use_complete_endpoints() {
    for (property, from, to) in [
        (
            "background-image",
            "url(paper.svg), linear-gradient(red, blue)",
            "none",
        ),
        (
            "background-repeat",
            "repeat no-repeat, no-repeat no-repeat",
            "space round",
        ),
        ("background-attachment", "fixed, local", "scroll"),
        ("border-top-style", "dotted", "solid"),
        ("border-right-style", "double", "groove"),
        ("border-bottom-style", "ridge", "inset"),
        ("border-left-style", "none", "outset"),
        (
            "border-image-source",
            "url(edge.svg)",
            "linear-gradient(red, blue)",
        ),
        ("border-image-repeat", "stretch round", "space repeat"),
    ] {
        switches(property, discrete(from), discrete(to));
    }
}

#[test]
fn border_widths_and_radii_keep_computed_components() {
    for property in [
        "border-top-width",
        "border-right-width",
        "border-bottom-width",
        "border-left-width",
    ] {
        samples(
            property,
            computed([length(2.0)]),
            computed([length(10.0)]),
            [(0.25, computed([length(4.0)]))],
        );
    }
    for property in [
        "border-top-left-radius",
        "border-top-right-radius",
        "border-bottom-right-radius",
        "border-bottom-left-radius",
    ] {
        samples(
            property,
            computed([length(10.0), percentage(0.25)]),
            computed([percentage(0.5), length(26.0)]),
            [(0.5, computed([mixed(5.0, 0.25), mixed(13.0, 0.125)]))],
        );
    }
}

#[test]
fn border_image_records_combine_sides_without_repeating_them() {
    // The four side values and slice fill are already expanded computed operands.
    samples(
        "border-image-slice",
        computed([
            number(4.0),
            percentage(0.25),
            number(8.0),
            percentage(0.5),
            tag("fill"),
        ]),
        computed([
            number(12.0),
            percentage(0.5),
            number(16.0),
            percentage(0.75),
            tag("fill"),
        ]),
        [(
            0.5,
            computed([
                number(8.0),
                percentage(0.375),
                number(12.0),
                percentage(0.625),
                tag("fill"),
            ]),
        )],
    );
    samples(
        "border-image-width",
        computed([tag("auto"), length(8.0), number(1.0), length(4.0)]),
        computed([tag("auto"), percentage(0.5), number(3.0), length(6.0)]),
        [(
            0.5,
            computed([tag("auto"), mixed(4.0, 0.25), number(2.0), length(5.0)]),
        )],
    );
    samples(
        "border-image-outset",
        computed([number(0.0), length(2.0), number(1.0), length(4.0)]),
        computed([number(2.0), length(8.0), number(5.0), length(6.0)]),
        [(
            0.5,
            computed([number(1.0), length(5.0), number(3.0), length(5.0)]),
        )],
    );
}

#[test]
fn incompatible_border_image_components_switch_the_entire_record() {
    let slice = computed([
        number(2.0),
        number(4.0),
        number(6.0),
        number(8.0),
        tag("fill"),
    ]);
    for to in [
        computed([
            number(4.0),
            number(6.0),
            number(8.0),
            number(10.0),
            tag("no-fill"),
        ]),
        computed([
            number(4.0),
            number(6.0),
            number(8.0),
            percentage(0.5),
            tag("fill"),
        ]),
    ] {
        switches("border-image-slice", slice.clone(), to);
    }
    let sides = computed([number(1.0), number(2.0), number(3.0), number(4.0)]);
    for (property, incompatible) in [
        ("border-image-width", tag("auto")),
        ("border-image-width", length(8.0)),
        ("border-image-outset", length(8.0)),
    ] {
        switches(
            property,
            sides.clone(),
            computed([number(5.0), number(6.0), number(7.0), incompatible]),
        );
    }
}

fn switches(property: &str, from: InterpolableValue, to: InterpolableValue) {
    samples(
        property,
        from.clone(),
        to.clone(),
        [(0.5_f64.next_down(), from), (0.5, to)],
    );
}

fn samples(
    property: &str,
    from: InterpolableValue,
    to: InterpolableValue,
    expected: impl IntoIterator<Item = (f64, InterpolableValue)>,
) {
    let pair = InterpolationPair::new(PropertyKey::new(property).unwrap(), from, to).unwrap();
    for (progress, value) in expected {
        assert_eq!(
            pair.sample(InterpolationProgress::new(progress).unwrap()),
            InterpolationOutcome::Value(value),
            "{property} at progress {progress}",
        );
    }
}

fn discrete(value: &str) -> InterpolableValue {
    InterpolableValue::discrete(DiscreteValue::from_text(value).unwrap())
}

fn computed<const N: usize>(components: [ComputedComponent; N]) -> InterpolableValue {
    InterpolableValue::computed(ComputedValue::new(Vec::from(components)).unwrap())
}

fn tag(value: &str) -> ComputedComponent {
    ComputedComponent::Discrete(DiscreteValue::from_text(value).unwrap())
}

fn number(value: f64) -> ComputedComponent {
    ComputedComponent::Numeric(ComputedNumeric::number(value).unwrap())
}

fn pixels(value: f64) -> ComputedDimension {
    ComputedDimension::new(DimensionKind::Length, value).unwrap()
}

fn length(value: f64) -> ComputedComponent {
    ComputedComponent::Numeric(ComputedNumeric::dimension(pixels(value)))
}

fn percentage(value: f64) -> ComputedComponent {
    ComputedComponent::Numeric(ComputedNumeric::percentage(value).unwrap())
}

fn mixed(px: f64, ratio: f64) -> ComputedComponent {
    ComputedComponent::Numeric(ComputedNumeric::mix(
        DimensionPercentage::new(pixels(px), ratio).unwrap(),
    ))
}
