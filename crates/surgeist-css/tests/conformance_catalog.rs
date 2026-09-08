use surgeist_css::{
    CssAngleCalculation, CssAngleUnit, CssBasicShapeValue, CssBlendMode, CssBoxEdgeKeyword,
    CssBoxShadow, CssCalculationType, CssClipPathValue, CssDelayLiteral, CssEasingValue,
    CssErrorCode, CssExclusionReason, CssFeatureKind, CssFilterFunctionValue, CssFilterValue,
    CssFrequencyCalculation, CssFrequencyUnit, CssHorizontalPosition, CssIntegerCalculation,
    CssKnownProperty, CssKnownPropertyValueRef, CssLength, CssLengthCalculation,
    CssLengthDimension, CssLengthUnit, CssNumberCalculation, CssPercentageCalculation,
    CssRecoveryAction, CssResolution, CssResolutionUnit, CssRule, CssSpecificationTier,
    CssSupportStatus, CssSupportsConditionKind, CssTimeCalculation, CssTimeUnit,
    CssTransformFunctionValue, CssTransformPerspective, CssTransformScaleComponent,
    CssTransformValue, CssVerticalPosition, ErrorKind, conformance_exclusion,
    conformance_exclusions, feature_metadata, parse_sheet, parse_style_attribute,
    property_metadata, specification_source, specification_sources,
};

#[derive(Clone, Copy)]
enum ExpectedSource {
    Id(&'static str),
}

#[derive(Clone, Copy)]
enum Input {
    Sheet(&'static str),
    Style(&'static str),
}

struct ExpectedFeature {
    id: &'static str,
    kind: CssFeatureKind,
    spelling: &'static str,
    source: ExpectedSource,
    production: &'static str,
    status: CssSupportStatus,
    supported_subset: Option<&'static str>,
    unsupported_remainder: Option<&'static str>,
    recognized_code: Option<CssErrorCode>,
    positive: Option<Input>,
    negative: Option<(Input, CssErrorCode)>,
}

const BASELINE_RULE_SUBSET: &str =
    "The baseline parser spelling and the I01 recovery extensions are supported.";
const BASELINE_RULE_REMAINDER: &str =
    "Other valid forms of the cited rule production are outside the I01 subset.";
const SELECTOR_REMAINDER: &str =
    "Other valid forms of the cited Selectors production are outside the I01 subset.";
const SUPPORTS_SELECTOR_SUBSET: &str = "selector() accepts complete Selectors 3 plus the selected I01 extensions: i and s attribute modifiers; :scope, :focus-visible, :focus-within, :required, :optional, :valid, :invalid, :placeholder-shown, :modal, :fullscreen, :popover-open, :default, :indeterminate, :read-only, :read-write, :in-range, and :out-of-range; :is(), :where(), :has(), selector-list :not(), and nth-child of lists; and ::marker, ::selection, ::backdrop, and generated-marker sequences.";
const SUPPORTS_SELECTOR_REMAINDER: &str = "The || combinator, unselected Selectors 4 pseudo-classes and pseudo-elements, and syntax outside those atomic extension rows remain outside the typed subset; balanced content is preserved as general-enclosed authored syntax.";
const QUERY_REMAINDER: &str =
    "Other valid forms of the cited query production are outside the I01 subset.";
const TIMING_SUBSET: &str = "The I01 shorthand components plus C03 duration, signed delay, iteration, and typed calculation syntax and C05 easing functions are supported.";
const TIMING_REMAINDER: &str =
    "Other valid forms of the cited shorthand production remain unsupported.";
const BASIC_SHAPE_SUBSET: &str =
    "Typed inset(), circle(), ellipse(), and polygon() functions are supported.";
const BASIC_SHAPE_REMAINDER: &str = "path(), shape(), rect(), and xywh() remain unsupported.";
const BACKDROP_FILTER_SUBSET: &str = "The exact I01 filter-function-list subset preserved at bc5394f:src/parser/effects.rs is supported with typed current values.";
const BACKDROP_FILTER_REMAINDER: &str = "Every Filter Effects 2 behavior absent from that preserved baseline subset remains unsupported.";
const CLIP_PATH_SUBSET: &str =
    "none, URL, and typed inset(), circle(), ellipse(), and polygon() functions are supported.";
const CLIP_PATH_REMAINDER: &str =
    "Reference-box combinations and path(), shape(), rect(), and xywh() remain unsupported.";

const COLOR5_RELATIVE_SUBSET: &str = "The eight preserved relative-color families are supported: rgb()/rgba(), hsl()/hsla(), hwb(), lab(), lch(), oklab(), oklch(), and color() in a predefined RGB or XYZ space.";
const COLOR5_RELATIVE_REMAINDER: &str = "alpha(), custom-profile parameters, and other unselected CSS Color 5 color functions remain unsupported.";
const COLOR5_MIX_SUBSET: &str = "The preserved color-mix() subset requires an interpolation method, exactly two colors, optional trailing percentages, and a predefined or polar color space.";
const COLOR5_MIX_REMAINDER: &str =
    "Other valid forms of the dated CSS Color 5 color-mix() production remain unsupported.";
const GRID_REPEAT_SUBSET: &str = "Non-recursive integer track and fixed repeats, plus one fixed-size automatic repeat where the consumer permits it, are supported.";
const GRID_REPEAT_REMAINDER: &str =
    "Subgrid name-repeat and other unselected Grid 2 forms remain unsupported.";
const GRID_PROPERTY_SUBSET: &str = "The C07 structural grammar supports non-recursive integer track and fixed repeats, one fixed-size automatic repeat where permitted, and repeat-free automatic track-size lists.";
const GRID_PROPERTY_REMAINDER: &str =
    "Subgrid name-repeat and other unselected Grid 2 property grammar remain unsupported.";
const KEYFRAMES_SUBSET: &str = "Keyframe names, literal selectors, empty rules and blocks, duplicate selectors and blocks in authored order, and supported declarations with recovery are supported.";
const KEYFRAMES_REMAINDER: &str = "Calculation selectors, string names, and declaration-processing grammar not selected by C07 remain unsupported.";
const FONT_WEIGHT_RANGE_SUBSET: &str =
    "Integer font-weight values from 1 through 1000 are supported.";
const FONT_WEIGHT_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-weight property grammar remains unsupported.";
const FONT_FACE_WEIGHT_RANGE_SUBSET: &str = "Font-face font-weight numbers from 1 through 1000 and increasing two-value ranges are supported.";
const FONT_FACE_WEIGHT_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-weight descriptor grammar remains unsupported.";
const FONT_FACE_STYLE_RANGE_SUBSET: &str =
    "Font-face oblique style with one or two increasing -90deg through 90deg angles is supported.";
const FONT_FACE_STYLE_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-style descriptor grammar remains unsupported.";
const FONT_FACE_STRETCH_RANGE_SUBSET: &str = "Font-face non-negative percentage stretch values and increasing two-value ranges are supported.";
const FONT_FACE_STRETCH_RANGE_REMAINDER: &str =
    "Other unselected Fonts 4 font-stretch descriptor grammar remains unsupported.";
const FONT_SOURCE_HINTS_SUBSET: &str = "The woff, woff2, truetype, opentype, collection, embedded-opentype, and svg format() hints and the variations, color-colrv0, color-colrv1, color-svg, color-sbix, color-cbdt, features-opentype, features-aat, features-graphite, and incremental tech() hints are supported.";
const FONT_SOURCE_HINTS_REMAINDER: &str =
    "Other unselected Fonts 4 font source format and technology hints remain unsupported.";

fn assert_complete_fonts3_feature(
    id: &str,
    kind: CssFeatureKind,
    spelling: &str,
    production: &str,
) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
    assert_eq!(metadata.kind(), kind, "{id}");
    assert_eq!(metadata.spelling(), spelling, "{id}");
    assert_eq!(metadata.source().id().as_str(), "O-FONTS3", "{id}");
    assert_eq!(metadata.production(), production, "{id}");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
    assert_eq!(metadata.supported_subset(), None, "{id}");
    assert_eq!(metadata.unsupported_remainder(), None, "{id}");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id}");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id}");
}

fn assert_partial_fonts4_feature(
    id: &str,
    kind: CssFeatureKind,
    spelling: &str,
    production: &str,
    subset: &str,
    remainder: &str,
) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
    assert_eq!(metadata.kind(), kind, "{id}");
    assert_eq!(metadata.spelling(), spelling, "{id}");
    assert_eq!(metadata.source().id().as_str(), "I-FONTS4", "{id}");
    assert_eq!(metadata.production(), production, "{id}");
    assert_eq!(metadata.status(), CssSupportStatus::Partial, "{id}");
    assert_eq!(metadata.supported_subset(), Some(subset), "{id}");
    assert_eq!(metadata.unsupported_remainder(), Some(remainder), "{id}");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id}");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id}");
}

#[test]
fn fonts3_and_preserved_fonts4_metadata_are_truthful() {
    let fonts3_properties = [
        ("baseline.property.font", "font", "#propdef-font", "menu"),
        (
            "baseline.property.font-family",
            "font-family",
            "#propdef-font-family",
            "\"Avenir Next\", sans-serif",
        ),
        (
            "baseline.property.font-feature-settings",
            "font-feature-settings",
            "#propdef-font-feature-settings",
            "\"kern\" on, \"liga\" 0",
        ),
        (
            "official.property.font-kerning",
            "font-kerning",
            "#propdef-font-kerning",
            "normal",
        ),
        (
            "baseline.property.font-size",
            "font-size",
            "#propdef-font-size",
            "medium",
        ),
        (
            "official.property.font-size-adjust",
            "font-size-adjust",
            "#propdef-font-size-adjust",
            "0.5",
        ),
        (
            "baseline.property.font-stretch",
            "font-stretch",
            "#propdef-font-stretch",
            "condensed",
        ),
        (
            "baseline.property.font-style",
            "font-style",
            "#propdef-font-style",
            "oblique",
        ),
        (
            "official.property.font-synthesis",
            "font-synthesis",
            "#propdef-font-synthesis",
            "weight style",
        ),
        (
            "baseline.property.font-variant",
            "font-variant",
            "#propdef-font-variant",
            "small-caps oldstyle-nums",
        ),
        (
            "official.property.font-variant-caps",
            "font-variant-caps",
            "#propdef-font-variant-caps",
            "all-small-caps",
        ),
        (
            "official.property.font-variant-east-asian",
            "font-variant-east-asian",
            "#propdef-font-variant-east-asian",
            "jis04 ruby",
        ),
        (
            "official.property.font-variant-ligatures",
            "font-variant-ligatures",
            "#propdef-font-variant-ligatures",
            "common-ligatures no-discretionary-ligatures",
        ),
        (
            "official.property.font-variant-numeric",
            "font-variant-numeric",
            "#propdef-font-variant-numeric",
            "lining-nums tabular-nums slashed-zero",
        ),
        (
            "official.property.font-variant-position",
            "font-variant-position",
            "#propdef-font-variant-position",
            "super",
        ),
        (
            "baseline.property.font-weight",
            "font-weight",
            "#propdef-font-weight",
            "700",
        ),
    ];
    for (id, spelling, production, authored) in fonts3_properties {
        let report = parse_style_attribute(&format!("{spelling}: {authored}"));
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{id}");
        assert_complete_fonts3_feature(id, CssFeatureKind::Property, spelling, production);
    }

    let font_face = parse_sheet(concat!(
        "@font-face { font-family: Demo Sans; ",
        "src: local(\"Demo Sans\"), url(demo.woff2) format(\"woff2\"); ",
        "font-style: oblique; font-weight: 700; font-stretch: condensed; ",
        "unicode-range: U+0-7F; font-feature-settings: \"kern\" on; }"
    ));
    assert!(font_face.is_clean(), "{:?}", font_face.diagnostics());
    assert_eq!(font_face.syntax().rules().len(), 1);

    for (id, kind, spelling, production) in [
        (
            "baseline.rule.font-face",
            CssFeatureKind::Rule,
            "@font-face",
            "#font-face-rule",
        ),
        (
            "baseline.descriptor.font-family",
            CssFeatureKind::Descriptor,
            "font-family in @font-face",
            "#font-family-desc",
        ),
        (
            "baseline.descriptor.src",
            CssFeatureKind::Descriptor,
            "src in @font-face",
            "#src-desc",
        ),
        (
            "baseline.descriptor.font-style",
            CssFeatureKind::Descriptor,
            "font-style in @font-face",
            "#font-prop-desc",
        ),
        (
            "baseline.descriptor.font-weight",
            CssFeatureKind::Descriptor,
            "font-weight in @font-face",
            "#font-prop-desc",
        ),
        (
            "baseline.descriptor.font-stretch",
            CssFeatureKind::Descriptor,
            "font-stretch in @font-face",
            "#font-prop-desc",
        ),
        (
            "baseline.descriptor.unicode-range",
            CssFeatureKind::Descriptor,
            "unicode-range in @font-face",
            "#unicode-range-desc",
        ),
        (
            "official.descriptor.font-feature-settings",
            CssFeatureKind::Descriptor,
            "font-feature-settings in @font-face",
            "#font-rend-desc",
        ),
        (
            "official.value.font-source",
            CssFeatureKind::Value,
            "@font-face source list",
            "#src-desc",
        ),
        (
            "official.value.opentype-tag",
            CssFeatureKind::Value,
            "OpenType feature tag",
            "#font-rend-desc",
        ),
    ] {
        assert_complete_fonts3_feature(id, kind, spelling, production);
    }

    let display =
        parse_sheet("@font-face { font-family: Demo; src: url(demo.woff2); font-display: swap; }");
    assert!(display.is_clean(), "{:?}", display.diagnostics());
    let display_metadata =
        feature_metadata("baseline.descriptor.font-display").expect("font-display metadata");
    assert_eq!(display_metadata.kind(), CssFeatureKind::Descriptor);
    assert_eq!(display_metadata.spelling(), "font-display in @font-face");
    assert_eq!(display_metadata.source().id().as_str(), "I-FONTS4");
    assert_eq!(display_metadata.production(), "#font-display-desc");
    assert_eq!(display_metadata.status(), CssSupportStatus::Complete);
    assert_eq!(display_metadata.supported_subset(), None);
    assert_eq!(display_metadata.unsupported_remainder(), None);
    assert_eq!(display_metadata.recognized_unsupported_code(), None);

    let unsupported = parse_sheet("@font-feature-values Demo { @styleset { nice: 1; } }");
    assert!(unsupported.syntax().rules().is_empty());
    assert_eq!(unsupported.diagnostics().len(), 1);
    assert_eq!(
        unsupported.diagnostics()[0].error().code(),
        CssErrorCode::UnsupportedAtRule
    );
    let unsupported_metadata =
        feature_metadata("later.rule.font-feature-values").expect("font-feature-values metadata");
    assert_eq!(unsupported_metadata.kind(), CssFeatureKind::Rule);
    assert_eq!(unsupported_metadata.spelling(), "@font-feature-values");
    assert_eq!(unsupported_metadata.source().id().as_str(), "I-FONTS4");
    assert_eq!(
        unsupported_metadata.production(),
        "#font-feature-values-rule"
    );
    assert_eq!(
        unsupported_metadata.status(),
        CssSupportStatus::RecognizedUnsupported
    );
    assert_eq!(
        unsupported_metadata.recognized_unsupported_code(),
        Some(CssErrorCode::UnsupportedAtRule)
    );

    let numeric_weight = parse_style_attribute("font-weight: 725");
    assert!(
        numeric_weight.is_clean(),
        "{:?}",
        numeric_weight.diagnostics()
    );
    assert_partial_fonts4_feature(
        "ext.property.font-weight-range",
        CssFeatureKind::Property,
        "font-weight numeric range",
        "#font-weight-prop",
        FONT_WEIGHT_RANGE_SUBSET,
        FONT_WEIGHT_RANGE_REMAINDER,
    );
    let invalid_numeric_weight = parse_style_attribute("font-weight: 1001");
    assert!(invalid_numeric_weight.syntax().is_empty());
    assert_eq!(invalid_numeric_weight.diagnostics().len(), 1);
    assert_eq!(
        invalid_numeric_weight.diagnostics()[0].error().code(),
        CssErrorCode::InvalidPropertyValue
    );

    for (id, spelling, production, subset, remainder, authored, rejected) in [
        (
            "ext.descriptor.font-weight-range",
            "font-weight ranges in @font-face",
            "#font-weight-desc",
            FONT_FACE_WEIGHT_RANGE_SUBSET,
            FONT_FACE_WEIGHT_RANGE_REMAINDER,
            "font-weight: 300 700",
            "font-weight: 700 300",
        ),
        (
            "ext.descriptor.font-style-oblique-range",
            "font-style oblique ranges in @font-face",
            "#font-style-desc",
            FONT_FACE_STYLE_RANGE_SUBSET,
            FONT_FACE_STYLE_RANGE_REMAINDER,
            "font-style: oblique -10deg 20deg",
            "font-style: oblique 91deg",
        ),
        (
            "ext.descriptor.font-stretch-range",
            "font-stretch percentage ranges in @font-face",
            "#font-stretch-desc",
            FONT_FACE_STRETCH_RANGE_SUBSET,
            FONT_FACE_STRETCH_RANGE_REMAINDER,
            "font-stretch: 75% 125%",
            "font-stretch: -1%",
        ),
    ] {
        let report = parse_sheet(&format!(
            "@font-face {{ font-family: Demo; src: url(demo.woff2); {authored}; }}"
        ));
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_partial_fonts4_feature(
            id,
            CssFeatureKind::Descriptor,
            spelling,
            production,
            subset,
            remainder,
        );

        let rejected_report = parse_sheet(&format!(
            "@font-face {{ font-family: Demo; src: url(demo.woff2); {rejected}; }}"
        ));
        assert_eq!(rejected_report.diagnostics().len(), 1, "{id}");
        assert_eq!(
            rejected_report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidDescriptorValue,
            "{id}"
        );
    }

    let modern_hints = parse_sheet(concat!(
        "@font-face { font-family: Demo; ",
        "src: url(demo.woff2) format(woff2) tech(variations, color-colrv1); }"
    ));
    assert!(modern_hints.is_clean(), "{:?}", modern_hints.diagnostics());
    assert_partial_fonts4_feature(
        "ext.value.font-source-modern-hints",
        CssFeatureKind::Value,
        "format() keyword and tech() font-source hints",
        "#font-face-src-parsing",
        FONT_SOURCE_HINTS_SUBSET,
        FONT_SOURCE_HINTS_REMAINDER,
    );
    let unknown_hint = parse_sheet(
        "@font-face { font-family: Demo; src: url(demo.woff2) format(woff3) tech(unknown); }",
    );
    assert!(unknown_hint.syntax().rules().is_empty());
    assert_eq!(unknown_hint.diagnostics().len(), 2);
    assert_eq!(
        unknown_hint.diagnostics()[0].error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(
        unknown_hint.diagnostics()[1].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );
}

fn assert_clean_color(authored: &str) {
    let source = format!("color: {authored}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color),
        "{source}",
    );
}

fn assert_rejected_color(authored: &str) {
    let source = format!("color: {authored}");
    let report = parse_style_attribute(&source);
    assert!(report.syntax().is_empty(), "{source}");
    assert_eq!(report.diagnostics().len(), 1, "{source}");
    assert_eq!(
        report.diagnostics()[0].error().code(),
        CssErrorCode::InvalidColorSyntax,
        "{source}",
    );
}

fn assert_complete_color4_value(id: &str, spelling: &str, production: &str, authored: &str) {
    assert_clean_color(authored);
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
    assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id}");
    assert_eq!(metadata.spelling(), spelling, "{id}");
    assert_eq!(metadata.source().id().as_str(), "O-COLOR4", "{id}");
    assert_eq!(metadata.production(), production, "{id}");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
    assert_eq!(metadata.supported_subset(), None, "{id}");
    assert_eq!(metadata.unsupported_remainder(), None, "{id}");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id}");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id}");
}

fn assert_complete_color5_value(
    id: &str,
    spelling: &str,
    production: &str,
    accepted: &str,
    rejected: &str,
) {
    assert_clean_color(accepted);
    assert_rejected_color(rejected);
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
    assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id}");
    assert_eq!(metadata.spelling(), spelling, "{id}");
    assert_eq!(metadata.source().id().as_str(), "I-COLOR5", "{id}");
    assert_eq!(metadata.production(), production, "{id}");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
    assert_eq!(metadata.supported_subset(), None, "{id}");
    assert_eq!(metadata.unsupported_remainder(), None, "{id}");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id}");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id}");
}

fn assert_partial_color5_value(
    id: &str,
    spelling: &str,
    production: &str,
    subset: &str,
    remainder: &str,
    accepted: &str,
    rejected: &str,
) {
    assert_clean_color(accepted);
    assert_rejected_color(rejected);
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
    assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id}");
    assert_eq!(metadata.spelling(), spelling, "{id}");
    assert_eq!(metadata.source().id().as_str(), "I-COLOR5", "{id}");
    assert_eq!(metadata.production(), production, "{id}");
    assert_eq!(metadata.status(), CssSupportStatus::Partial, "{id}");
    assert_eq!(metadata.supported_subset(), Some(subset), "{id}");
    assert_eq!(metadata.unsupported_remainder(), Some(remainder), "{id}");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id}");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id}");
}

#[test]
fn color4_value_and_property_metadata_match_public_authored_behavior() {
    assert_complete_color4_value("official.value.color", "<color>", "#color-type", "red");
    assert_complete_color4_value(
        "official.value.alpha",
        "<alpha-value>",
        "#alpha-syntax",
        "rgb(0 0 0 / 150%)",
    );
    assert_complete_color4_value(
        "official.value.hue",
        "<hue>",
        "#hue-syntax",
        "hsl(calc(1turn - 90deg) 50% 50%)",
    );
    assert_complete_color4_value(
        "official.value.rgb",
        "rgb()/rgba()",
        "#rgb-functions",
        "rgb(calc(1 + 2) 20% none / 150%)",
    );
    assert_complete_color4_value(
        "official.value.hex-color",
        "<hex-color>",
        "#hex-notation",
        "#00ff88cc",
    );
    assert_complete_color4_value(
        "official.value.named-color",
        "<named-color>",
        "#named-colors",
        "rebeccapurple",
    );
    assert_complete_color4_value(
        "official.value.system-color",
        "<system-color>",
        "#css-system-colors",
        "CanvasText",
    );
    assert_complete_color4_value(
        "official.value.deprecated-system-color",
        "<deprecated-system-color>",
        "#css-system-colors",
        "ActiveBorder",
    );
    assert_complete_color4_value(
        "official.value.transparent",
        "transparent",
        "#transparent-color",
        "transparent",
    );
    assert_complete_color4_value(
        "official.value.currentcolor",
        "currentColor",
        "#currentcolor-color",
        "currentColor",
    );
    assert_complete_color4_value(
        "official.value.hsl",
        "hsl()/hsla()",
        "#the-hsl-notation",
        "hsl(30deg 120% -20% / none)",
    );
    assert_complete_color4_value(
        "official.value.hwb",
        "hwb()",
        "#the-hwb-notation",
        "hwb(none 20% 120% / -10%)",
    );
    assert_complete_color4_value(
        "official.value.lab",
        "lab()",
        "#specifying-lab-lch",
        "lab(50% 20 -30 / 120%)",
    );
    assert_complete_color4_value(
        "official.value.lch",
        "lch()",
        "#specifying-lab-lch",
        "lch(50% 20 30deg / none)",
    );
    assert_complete_color4_value(
        "official.value.oklab",
        "oklab()",
        "#specifying-oklab-oklch",
        "oklab(50% 0.1 -0.1)",
    );
    assert_complete_color4_value(
        "official.value.oklch",
        "oklch()",
        "#specifying-oklab-oklch",
        "oklch(50% 0.2 30deg)",
    );
    assert_complete_color4_value(
        "official.value.predefined-color",
        "color()",
        "#color-function",
        "color(display-p3 1 0 0 / 120%)",
    );

    let color = property_metadata("color").expect("color metadata");
    assert_eq!(color.feature().status(), CssSupportStatus::Complete);
    assert_eq!(color.feature().source().id().as_str(), "O-COLOR4");
    assert_eq!(color.feature().production(), "#propdef-color");
    assert_eq!(color.feature().supported_subset(), None);
    assert_eq!(color.feature().unsupported_remainder(), None);

    let opacity_report = parse_style_attribute("opacity: 150%");
    assert!(
        opacity_report.is_clean(),
        "{:?}",
        opacity_report.diagnostics()
    );
    let opacity = property_metadata("opacity").expect("opacity metadata");
    assert_eq!(opacity.feature().status(), CssSupportStatus::Complete);
    assert_eq!(opacity.feature().source().id().as_str(), "O-COLOR4");
    assert_eq!(opacity.feature().production(), "#propdef-opacity");
    assert_eq!(opacity.feature().supported_subset(), None);
    assert_eq!(opacity.feature().unsupported_remainder(), None);
}

#[test]
fn relative_color_selected_families_and_deferred_remainder_are_distinct() {
    assert_partial_color5_value(
        "ext.value.relative-color",
        "relative color syntax",
        "#relative-colors,#relative-syntax",
        COLOR5_RELATIVE_SUBSET,
        COLOR5_RELATIVE_REMAINDER,
        "rgb(from red r g b / alpha)",
        "alpha(from red r g b)",
    );
}

#[test]
fn relative_rgb_channels_reject_foreign_identifiers_and_dimensions() {
    assert_complete_color5_value(
        "ext.value.relative-color.rgb",
        "relative rgb()/rgba()",
        "#relative-RGB",
        "rgb(from red r g b / alpha)",
        "rgb(from red h g b)",
    );
}

#[test]
fn relative_hsl_channels_keep_hue_and_percentage_domains_distinct() {
    assert_complete_color5_value(
        "ext.value.relative-color.hsl",
        "relative hsl()/hsla()",
        "#relative-HSL",
        "hsl(from red h s l / alpha)",
        "hsl(from red 10% s l)",
    );
}

#[test]
fn relative_hwb_channels_use_only_hwb_environment() {
    assert_complete_color5_value(
        "ext.value.relative-color.hwb",
        "relative hwb()",
        "#relative-HWB",
        "hwb(from red h w b / alpha)",
        "hwb(from red h s b)",
    );
}

#[test]
fn relative_lab_channels_use_only_lab_environment() {
    assert_complete_color5_value(
        "ext.value.relative-color.lab",
        "relative lab()",
        "#relative-Lab",
        "lab(from red l a b / alpha)",
        "lab(from red l c b)",
    );
}

#[test]
fn relative_oklab_channels_use_only_oklab_environment() {
    assert_complete_color5_value(
        "ext.value.relative-color.oklab",
        "relative oklab()",
        "#relative-Oklab",
        "oklab(from red l a b / alpha)",
        "oklab(from red l c b)",
    );
}

#[test]
fn relative_lch_channels_keep_hue_domain_distinct() {
    assert_complete_color5_value(
        "ext.value.relative-color.lch",
        "relative lch()",
        "#relative-LCH",
        "lch(from red l c h / alpha)",
        "lch(from red l c calc(h + 10%))",
    );
}

#[test]
fn relative_oklch_channels_keep_hue_domain_distinct() {
    assert_complete_color5_value(
        "ext.value.relative-color.oklch",
        "relative oklch()",
        "#relative-OkLCh",
        "oklch(from red l c h / alpha)",
        "oklch(from red l c calc(h + 10%))",
    );
}

#[test]
fn relative_predefined_color_channels_follow_space_environment() {
    assert_complete_color5_value(
        "ext.value.relative-color.predefined",
        "relative color()",
        "#relative-color-function",
        "color(from red display-p3 r g b / alpha)",
        "color(from red srgb x g b)",
    );
}

#[test]
fn color_mix_preserved_subset_rejects_cross_space_hue_methods() {
    assert_partial_color5_value(
        "ext.value.color-mix",
        "color-mix()",
        "#funcdef-color-mix",
        COLOR5_MIX_SUBSET,
        COLOR5_MIX_REMAINDER,
        "color-mix(in oklch longer hue, red 25%, blue 75%)",
        "color-mix(in srgb longer hue, red, blue)",
    );
}

fn record_partial_metadata_mismatch(
    mismatches: &mut Vec<String>,
    identity: (&str, CssFeatureKind, &str),
    provenance: (&str, &str),
    boundary: (&str, &str),
) {
    let (id, kind, spelling) = identity;
    let (source, production) = provenance;
    let (subset, remainder) = boundary;
    let Some(metadata) = feature_metadata(id) else {
        mismatches.push(format!("{id} is absent"));
        return;
    };
    if metadata.kind() != kind {
        mismatches.push(format!("{id} has stale kind {:?}", metadata.kind()));
    }
    if metadata.spelling() != spelling {
        mismatches.push(format!("{id} has stale spelling {:?}", metadata.spelling()));
    }
    if metadata.source().id().as_str() != source {
        mismatches.push(format!(
            "{id} has stale source {:?}",
            metadata.source().id().as_str()
        ));
    }
    if metadata.production() != production {
        mismatches.push(format!(
            "{id} has stale production {:?}",
            metadata.production()
        ));
    }
    if metadata.status() != CssSupportStatus::Partial {
        mismatches.push(format!("{id} has stale status {:?}", metadata.status()));
    }
    if metadata.supported_subset() != Some(subset) {
        mismatches.push(format!(
            "{id} has stale supported subset {:?}",
            metadata.supported_subset()
        ));
    }
    if metadata.unsupported_remainder() != Some(remainder) {
        mismatches.push(format!(
            "{id} has stale unsupported remainder {:?}",
            metadata.unsupported_remainder()
        ));
    }
}

#[test]
fn grid_and_keyframe_metadata_matches_preserved_boundaries() {
    let grid = parse_style_attribute(concat!(
        "grid-template-columns: repeat(auto-fit, 10px); ",
        "grid-auto-rows: minmax(10px, auto)",
    ));
    assert!(grid.is_clean(), "{:?}", grid.diagnostics());
    assert_eq!(grid.syntax().len(), 2);
    assert_eq!(
        grid.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::GridTemplateColumns),
    );
    assert_eq!(
        grid.syntax()[1].known().map(|known| known.property()),
        Some(CssKnownProperty::GridAutoRows),
    );

    let invalid_grid =
        parse_style_attribute("grid-template-columns: repeat(auto-fit, 1fr); color: red");
    assert_eq!(invalid_grid.syntax().len(), 1);
    assert_eq!(
        invalid_grid.syntax()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::Color),
    );
    assert_eq!(
        invalid_grid.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );

    let keyframes = parse_sheet(concat!(
        "@keyframes fade { ",
        "from, 0%, from { } ",
        "from { opacity: 0; } ",
        "0% { opacity: 1; } ",
        "} ",
        "@keyframes empty {}",
    ));
    assert!(keyframes.is_clean(), "{:?}", keyframes.diagnostics());
    let [CssRule::Keyframes(fade), CssRule::Keyframes(empty)] = keyframes.syntax().rules() else {
        panic!("expected authored duplicate and empty keyframe rules");
    };
    assert_eq!(fade.blocks().len(), 3);
    assert!(fade.blocks()[0].declarations().is_empty());
    assert!(empty.blocks().is_empty());

    let mut mismatches = Vec::new();
    record_partial_metadata_mismatch(
        &mut mismatches,
        ("ext.value.grid-repeat", CssFeatureKind::Value, "repeat()"),
        ("R-GRID2", "#repeat-notation"),
        (GRID_REPEAT_SUBSET, GRID_REPEAT_REMAINDER),
    );
    record_partial_metadata_mismatch(
        &mut mismatches,
        (
            "baseline.property.grid-template-rows",
            CssFeatureKind::Property,
            "grid-template-rows",
        ),
        ("R-GRID2", "#propdef-grid-template-rows"),
        (GRID_PROPERTY_SUBSET, GRID_PROPERTY_REMAINDER),
    );
    record_partial_metadata_mismatch(
        &mut mismatches,
        (
            "baseline.property.grid-template-columns",
            CssFeatureKind::Property,
            "grid-template-columns",
        ),
        ("R-GRID2", "#propdef-grid-template-columns"),
        (GRID_PROPERTY_SUBSET, GRID_PROPERTY_REMAINDER),
    );
    record_partial_metadata_mismatch(
        &mut mismatches,
        (
            "baseline.property.grid-template",
            CssFeatureKind::Property,
            "grid-template",
        ),
        ("R-GRID2", "#propdef-grid-template"),
        (GRID_PROPERTY_SUBSET, GRID_PROPERTY_REMAINDER),
    );
    record_partial_metadata_mismatch(
        &mut mismatches,
        (
            "baseline.property.grid-auto-rows",
            CssFeatureKind::Property,
            "grid-auto-rows",
        ),
        ("R-GRID2", "#propdef-grid-auto-rows"),
        (GRID_PROPERTY_SUBSET, GRID_PROPERTY_REMAINDER),
    );
    record_partial_metadata_mismatch(
        &mut mismatches,
        (
            "baseline.property.grid-auto-columns",
            CssFeatureKind::Property,
            "grid-auto-columns",
        ),
        ("R-GRID2", "#propdef-grid-auto-columns"),
        (GRID_PROPERTY_SUBSET, GRID_PROPERTY_REMAINDER),
    );
    record_partial_metadata_mismatch(
        &mut mismatches,
        ("baseline.property.grid", CssFeatureKind::Property, "grid"),
        ("R-GRID2", "#propdef-grid"),
        (GRID_PROPERTY_SUBSET, GRID_PROPERTY_REMAINDER),
    );
    record_partial_metadata_mismatch(
        &mut mismatches,
        (
            "baseline.rule.keyframes",
            CssFeatureKind::Rule,
            "@keyframes",
        ),
        ("I-ANIMATIONS1", "#keyframes"),
        (KEYFRAMES_SUBSET, KEYFRAMES_REMAINDER),
    );

    if !mismatches.is_empty() {
        panic!("stale Grid/keyframe metadata:\n{}", mismatches.join("\n"));
    }
}

const EXPECTED: &[ExpectedFeature] = &[
    ExpectedFeature {
        id: "baseline.rule.import",
        kind: CssFeatureKind::Rule,
        spelling: "@import",
        source: ExpectedSource::Id("O-CASCADE4"),
        production: "#at-import",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@import url(theme.css) supports(display: grid) print;",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.rule.layer-statement",
        kind: CssFeatureKind::Rule,
        spelling: "@layer ...;",
        source: ExpectedSource::Id("R-CASCADE5"),
        production: "#layering",
        status: CssSupportStatus::Partial,
        supported_subset: Some(BASELINE_RULE_SUBSET),
        unsupported_remainder: Some(BASELINE_RULE_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet("@layer reset, theme;")),
        negative: Some((
            Input::Sheet("@layer initial;"),
            CssErrorCode::InvalidAtRulePrelude,
        )),
    },
    ExpectedFeature {
        id: "baseline.rule.layer-block",
        kind: CssFeatureKind::Rule,
        spelling: "@layer {...}",
        source: ExpectedSource::Id("R-CASCADE5"),
        production: "#layering",
        status: CssSupportStatus::Partial,
        supported_subset: Some(BASELINE_RULE_SUBSET),
        unsupported_remainder: Some(BASELINE_RULE_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet("@layer theme { .x { color: red; } }")),
        negative: Some((
            Input::Sheet("@layer first, second { .x { color: red; } }"),
            CssErrorCode::InvalidAtRuleBody,
        )),
    },
    ExpectedFeature {
        id: "baseline.rule.font-face",
        kind: CssFeatureKind::Rule,
        spelling: "@font-face",
        source: ExpectedSource::Id("O-FONTS3"),
        production: "#font-face-rule",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2); }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.rule.keyframes",
        kind: CssFeatureKind::Rule,
        spelling: "@keyframes",
        source: ExpectedSource::Id("I-ANIMATIONS1"),
        production: "#keyframes",
        status: CssSupportStatus::Partial,
        supported_subset: Some(KEYFRAMES_SUBSET),
        unsupported_remainder: Some(KEYFRAMES_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }",
        )),
        negative: Some((
            Input::Sheet("@keyframes none { from { opacity: 0; } }"),
            CssErrorCode::InvalidAtRulePrelude,
        )),
    },
    ExpectedFeature {
        id: "baseline.rule.style",
        kind: CssFeatureKind::Rule,
        spelling: "style and nested qualified rules",
        source: ExpectedSource::Id("O-SYNTAX3"),
        production: "#style-rules",
        status: CssSupportStatus::Partial,
        supported_subset: Some(BASELINE_RULE_SUBSET),
        unsupported_remainder: Some(BASELINE_RULE_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(".x { color: red; }")),
        negative: Some((
            Input::Sheet("??? { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.rule.media",
        kind: CssFeatureKind::Rule,
        spelling: "@media",
        source: ExpectedSource::Id("O-CONDITIONAL3"),
        production: "#at-media",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet("@media screen { .x { color: red; } }")),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.rule.container",
        kind: CssFeatureKind::Rule,
        spelling: "@container",
        source: ExpectedSource::Id("X-CONTAIN3"),
        production: "#container-rule",
        status: CssSupportStatus::Partial,
        supported_subset: Some(BASELINE_RULE_SUBSET),
        unsupported_remainder: Some(BASELINE_RULE_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@container (width > 1px) { .x { color: red; } }",
        )),
        negative: Some((
            Input::Sheet("@container scroll-state(stuck: top) { .x { color: red; } }"),
            CssErrorCode::InvalidAtRulePrelude,
        )),
    },
    ExpectedFeature {
        id: "baseline.rule.scope",
        kind: CssFeatureKind::Rule,
        spelling: "@scope",
        source: ExpectedSource::Id("X-CASCADE6"),
        production: "#scope-atrule",
        status: CssSupportStatus::Partial,
        supported_subset: Some(BASELINE_RULE_SUBSET),
        unsupported_remainder: Some(BASELINE_RULE_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet("@scope (.card) { .title { color: red; } }")),
        negative: Some((
            Input::Sheet("@scope .card { .title { color: red; } }"),
            CssErrorCode::InvalidAtRulePrelude,
        )),
    },
    ExpectedFeature {
        id: "foundation.encoding.charset",
        kind: CssFeatureKind::Rule,
        spelling: "optional leading legacy @charset metadata",
        source: ExpectedSource::Id("O-SYNTAX3"),
        production: "#charset-rule",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet("@charset \"UTF-8\"; .x { color: red; }")),
        negative: None,
    },
    ExpectedFeature {
        id: "foundation.declaration-list.style-attribute",
        kind: CssFeatureKind::Declaration,
        spelling: "style-attribute declaration-list structure",
        source: ExpectedSource::Id("O-STYLE-ATTR"),
        production: "#syntax",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Style("color: red; width: 1px")),
        negative: None,
    },
    ExpectedFeature {
        id: "foundation.declaration.importance",
        kind: CssFeatureKind::Declaration,
        spelling: "terminal declaration !important annotation",
        source: ExpectedSource::Id("O-CASCADE4"),
        production: "#importance",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Style("color: red !important")),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.declaration.custom-property",
        kind: CssFeatureKind::Declaration,
        spelling: "custom-property names and authored token streams",
        source: ExpectedSource::Id("O-VARIABLES1"),
        production: "#defining-variables,#syntax",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "Baseline custom-property names and authored token streams, including I01 recovery behavior, are supported.",
        ),
        unsupported_remainder: Some(
            "Other valid CSS Variables custom-property declaration forms are outside the I01 subset.",
        ),
        recognized_code: None,
        positive: Some(Input::Style("--theme: dark")),
        negative: Some((
            Input::Style("--x: inherit 1px"),
            CssErrorCode::InvalidQualifiedRule,
        )),
    },
    ExpectedFeature {
        id: "baseline.value.substitution-dependent",
        kind: CssFeatureKind::Value,
        spelling: "preserved known-property values containing substitution functions",
        source: ExpectedSource::Id("O-VARIABLES1"),
        production: "#using-variables",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "Known-property values with syntactically admissible var() references remain authored and symbolic.",
        ),
        unsupported_remainder: Some(
            "Other valid CSS Variables substitution functions and post-substitution forms are outside the I01 subset.",
        ),
        recognized_code: None,
        positive: Some(Input::Style("width: var(--width, 1px)")),
        negative: Some((
            Input::Style("width: var(color)"),
            CssErrorCode::InvalidPropertyValue,
        )),
    },
    ExpectedFeature {
        id: "later.rule.namespace",
        kind: CssFeatureKind::Rule,
        spelling: "@namespace",
        source: ExpectedSource::Id("O-NAMESPACES3"),
        production: "#declaration,#syntax",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@namespace svg url(https://example.test/svg);",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "later.rule.counter-style",
        kind: CssFeatureKind::Rule,
        spelling: "@counter-style",
        source: ExpectedSource::Id("O-COUNTERSTYLES3"),
        production: "#the-counter-style-rule",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@counter-style thumbs { system: cyclic; symbols: 👍; suffix: \" \"; }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "later.rule.page",
        kind: CssFeatureKind::Rule,
        spelling: "@page",
        source: ExpectedSource::Id("O-CSS2"),
        production: "page.html#page-box",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet("@page { margin: 1cm; }")),
        negative: None,
    },
    ExpectedFeature {
        id: "official.selector.page-pseudo",
        kind: CssFeatureKind::Selector,
        spelling: ":left|:right|:first",
        source: ExpectedSource::Id("O-CSS2"),
        production: "page.html#page-selectors",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet("@page :left { margin: 1cm; }")),
        negative: None,
    },
    ExpectedFeature {
        id: "later.rule.font-feature-values",
        kind: CssFeatureKind::Rule,
        spelling: "@font-feature-values",
        source: ExpectedSource::Id("I-FONTS4"),
        production: "#font-feature-values-rule",
        status: CssSupportStatus::RecognizedUnsupported,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: Some(CssErrorCode::UnsupportedAtRule),
        positive: None,
        negative: Some((
            Input::Sheet("@font-feature-values Font One { @styleset { nice: 1; } }"),
            CssErrorCode::UnsupportedAtRule,
        )),
    },
    ExpectedFeature {
        id: "baseline.descriptor.font-family",
        kind: CssFeatureKind::Descriptor,
        spelling: "font-family in @font-face",
        source: ExpectedSource::Id("O-FONTS3"),
        production: "#font-family-desc",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2); }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.descriptor.src",
        kind: CssFeatureKind::Descriptor,
        spelling: "src in @font-face",
        source: ExpectedSource::Id("O-FONTS3"),
        production: "#src-desc",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2) format(\"woff2\"); }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.descriptor.font-weight",
        kind: CssFeatureKind::Descriptor,
        spelling: "font-weight in @font-face",
        source: ExpectedSource::Id("O-FONTS3"),
        production: "#font-prop-desc",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2); font-weight: 700; }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.descriptor.font-style",
        kind: CssFeatureKind::Descriptor,
        spelling: "font-style in @font-face",
        source: ExpectedSource::Id("O-FONTS3"),
        production: "#font-prop-desc",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2); font-style: italic; }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.descriptor.font-stretch",
        kind: CssFeatureKind::Descriptor,
        spelling: "font-stretch in @font-face",
        source: ExpectedSource::Id("O-FONTS3"),
        production: "#font-prop-desc",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2); font-stretch: condensed; }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.descriptor.font-display",
        kind: CssFeatureKind::Descriptor,
        spelling: "font-display in @font-face",
        source: ExpectedSource::Id("I-FONTS4"),
        production: "#font-display-desc",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2); font-display: swap; }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.descriptor.unicode-range",
        kind: CssFeatureKind::Descriptor,
        spelling: "unicode-range in @font-face",
        source: ExpectedSource::Id("O-FONTS3"),
        production: "#unicode-range-desc",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@font-face { font-family: Inter; src: url(inter.woff2); unicode-range: U+0000-00FF; }",
        )),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.selector.complex",
        kind: CssFeatureKind::Selector,
        spelling: "type, universal, ID, class; presence and six valued attribute matchers; descendant, child, next-sibling, subsequent-sibling combinators",
        source: ExpectedSource::Id("O-SELECTORS3"),
        production: "#type-selectors,#universal-selector,#attribute-representation,#attribute-substrings,#class-html,#id-selectors,#descendant-combinators,#child-combinators,#adjacent-sibling-combinators,#general-sibling-combinators",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized complex-selector spelling group is supported.",
        ),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            "article#main.card[data-ready][lang|=\"en\"] > span + a ~ b { color: red; }",
        )),
        negative: Some((
            Input::Sheet("svg|a { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.selector.pseudo-class",
        kind: CssFeatureKind::Selector,
        spelling: ":root, :hover, :active, :focus, :disabled, :enabled, :checked, :first-child, :last-child, :only-child, :empty, :first-of-type, :last-of-type, :only-of-type",
        source: ExpectedSource::Id("O-SELECTORS3"),
        production: "#dynamic-pseudos,#UIstates,#structural-pseudos",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized pseudo-class spelling group is supported.",
        ),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(".button:hover { color: red; }")),
        negative: Some((
            Input::Sheet(".host:host { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.selector.functional",
        kind: CssFeatureKind::Selector,
        spelling: ":nth-child(), :nth-last-child(), :nth-of-type(), :nth-last-of-type(), :not()",
        source: ExpectedSource::Id("O-SELECTORS3"),
        production: "#structural-pseudos,#negation",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized functional pseudo-class spelling group is supported.",
        ),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(".item:nth-child(2n+1) { color: red; }")),
        negative: Some((
            Input::Sheet(".item:dir(ltr) { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.selector.extension-state",
        kind: CssFeatureKind::Selector,
        spelling: ":scope, :focus-visible, :focus-within, :required, :optional, :valid, :invalid, :placeholder-shown, :default, :indeterminate, :read-only, :read-write, :in-range, :out-of-range, :modal, :fullscreen, :popover-open",
        source: ExpectedSource::Id("I-SELECTORS4"),
        production: "#useraction-pseudos,#input-pseudos,#resource-pseudos,#display-state-pseudos",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact I01 extension-state pseudo-class spelling group is supported.",
        ),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(".button:focus-visible { color: red; }")),
        negative: Some((
            Input::Sheet(".link:local-link { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.selector.extension-functional",
        kind: CssFeatureKind::Selector,
        spelling: ":is(), :where(), complex :not(), :has(), and nth-child of lists",
        source: ExpectedSource::Id("I-SELECTORS4"),
        production: "#matches,#zero-matches,#relational,#negation,#the-nth-child-pseudo",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact I01 extension-functional pseudo-class spelling group is supported.",
        ),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            ".item:is(.primary, .secondary) { color: red; }",
        )),
        negative: Some((
            Input::Sheet(".item:has(:has(.nested)) { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.selector.attribute-case",
        kind: CssFeatureKind::Selector,
        spelling: "i and s attribute-selector modifiers",
        source: ExpectedSource::Id("I-SELECTORS4"),
        production: "#attribute-case",
        status: CssSupportStatus::Partial,
        supported_subset: Some("The i and s attribute-selector case modifiers are supported."),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet("[data-kind=\"primary\" i] { color: red; }")),
        negative: Some((
            Input::Sheet("[data-kind=\"primary\" q] { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.selector.pseudo-element",
        kind: CssFeatureKind::Selector,
        spelling: "::before, ::after, ::marker, ::selection, ::backdrop, and generated ::marker sequences",
        source: ExpectedSource::Id("I01-BASE-SELECTORS"),
        production: "pseudo-element selector",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized pseudo-element spelling group is supported.",
        ),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(".item::before { content: \"x\"; }")),
        negative: Some((
            Input::Sheet(".item::part(label) { color: red; }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.selector.nesting",
        kind: CssFeatureKind::Selector,
        spelling: "nesting &, scoped selector anchors, and scoped relative selectors",
        source: ExpectedSource::Id("I-NESTING1"),
        production: "#nest-selector",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "Nesting &, scoped selector anchors, and scoped relative selectors are supported.",
        ),
        unsupported_remainder: Some(SELECTOR_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(".card { & > .title { color: red; } }")),
        negative: Some((
            Input::Sheet(".card { & || .title { color: red; } }"),
            CssErrorCode::InvalidSelector,
        )),
    },
    ExpectedFeature {
        id: "baseline.media.query-list",
        kind: CssFeatureKind::MediaQuery,
        spelling: "typed/condition query lists, not/only, and/or/not, range and colon forms, and malformed-member Never recovery",
        source: ExpectedSource::Id("I01-BASE-QUERIES"),
        production: "media query list",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized media query-list spelling group and malformed-member Never recovery are supported.",
        ),
        unsupported_remainder: Some(QUERY_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@media screen and (min-width: 1px), print { .x { color: red; } }",
        )),
        negative: Some((
            Input::Sheet("@media screen, ??? { .x { color: red; } }"),
            CssErrorCode::InvalidMediaQuery,
        )),
    },
    ExpectedFeature {
        id: "baseline.media.type",
        kind: CssFeatureKind::MediaQuery,
        spelling: "all, aural, braille, embossed, handheld, print, projection, screen, speech, tty, tv",
        source: ExpectedSource::Id("O-MEDIA3"),
        production: "#media1",
        status: CssSupportStatus::Complete,
        supported_subset: None,
        unsupported_remainder: None,
        recognized_code: None,
        positive: Some(Input::Sheet("@media speech { .x { color: red; } }")),
        negative: None,
    },
    ExpectedFeature {
        id: "baseline.media.range-feature",
        kind: CssFeatureKind::MediaQuery,
        spelling: "width, height, resolution, color, monochrome and their min-/max- names",
        source: ExpectedSource::Id("I01-BASE-QUERIES"),
        production: "media range feature",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized media range-feature spelling group is supported.",
        ),
        unsupported_remainder: Some(QUERY_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet("@media (width >= 1px) { .x { color: red; } }")),
        negative: Some((
            Input::Sheet("@media (min-width) { .x { color: red; } }"),
            CssErrorCode::InvalidMediaQuery,
        )),
    },
    ExpectedFeature {
        id: "baseline.media.discrete-feature",
        kind: CssFeatureKind::MediaQuery,
        spelling: "orientation, prefers-color-scheme, prefers-reduced-motion, prefers-reduced-transparency, prefers-contrast, forced-colors, hover, any-hover, pointer, any-pointer, display-mode",
        source: ExpectedSource::Id("I01-BASE-QUERIES"),
        production: "media discrete feature",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized media discrete-feature spelling group is supported.",
        ),
        unsupported_remainder: Some(QUERY_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@media (orientation: landscape) { .x { color: red; } }",
        )),
        negative: Some((
            Input::Sheet("@media (scripting: enabled) { .x { color: red; } }"),
            CssErrorCode::InvalidMediaQuery,
        )),
    },
    ExpectedFeature {
        id: "baseline.container.condition",
        kind: CssFeatureKind::ContainerQuery,
        spelling: "and/or/not, size features, and custom-property style existence/equality",
        source: ExpectedSource::Id("X-CONTAIN3"),
        production: "#container-rule",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized container-condition spelling group is supported.",
        ),
        unsupported_remainder: Some(QUERY_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@container (width > 1px) and style(--theme) { .x { color: red; } }",
        )),
        negative: Some((
            Input::Sheet("@container style(color: red) { .x { color: red; } }"),
            CssErrorCode::InvalidAtRulePrelude,
        )),
    },
    ExpectedFeature {
        id: "baseline.container.size-feature",
        kind: CssFeatureKind::ContainerQuery,
        spelling: "width, height, inline-size, block-size, aspect-ratio, orientation and applicable min-/max- names",
        source: ExpectedSource::Id("X-CONTAIN3"),
        production: "#size-container",
        status: CssSupportStatus::Partial,
        supported_subset: Some(
            "The exact baseline-recognized container size-feature spelling group is supported.",
        ),
        unsupported_remainder: Some(QUERY_REMAINDER),
        recognized_code: None,
        positive: Some(Input::Sheet(
            "@container (inline-size > 1px) { .x { color: red; } }",
        )),
        negative: Some((
            Input::Sheet("@container (unknown-size > 1px) { .x { color: red; } }"),
            CssErrorCode::InvalidAtRulePrelude,
        )),
    },
];

fn assert_c03_value_metadata(
    id: &str,
    spelling: &str,
    production: &str,
    status: CssSupportStatus,
    subset: Option<&str>,
    remainder: Option<&str>,
) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
    assert_eq!(metadata.id().as_str(), id);
    assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id} kind");
    assert_eq!(metadata.spelling(), spelling, "{id} spelling");
    assert_eq!(metadata.source().id().as_str(), "O-VALUES3", "{id} source");
    assert_eq!(metadata.production(), production, "{id} production");
    assert_eq!(metadata.status(), status, "{id} status");
    assert_eq!(metadata.supported_subset(), subset, "{id} subset");
    assert_eq!(
        metadata.unsupported_remainder(),
        remainder,
        "{id} remainder"
    );
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
}

#[test]
fn c14_amended_ledger_public_metadata_is_reconciled() {
    for id in [
        "official.value.syntax-token-stream",
        "official.value.component-value",
        "official.value.simple-block",
        "official.value.function",
        "official.value.declaration-value",
        "official.value.any-value",
        "official.value.an-plus-b",
        "official.value.unicode-range",
        "official.value.css-wide-keyword",
        "official.value.custom-ident",
        "official.value.ident",
        "official.value.string",
        "official.value.url",
        "official.value.url-modifier",
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    }

    for id in [
        "official.value.dimension",
        "official.value.angle",
        "official.value.angle-percentage",
        "official.value.time-percentage",
        "official.value.frequency",
        "official.value.frequency-percentage",
        "official.value.calc",
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    }

    for id in [
        "ext.value.relative-color",
        "ext.value.color-mix",
        "ext.value.grid-repeat",
        "ext.value.basic-shape",
        "ext.descriptor.font-weight-range",
        "ext.descriptor.font-style-oblique-range",
        "ext.descriptor.font-stretch-range",
        "ext.value.font-source-modern-hints",
        "ext.property.font-weight-range",
        "ext.supports.selector",
        "ext.media.range.width",
        "ext.media.range.height",
        "ext.media.range.resolution",
        "ext.media.range.color",
        "ext.media.range.monochrome",
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.status(), CssSupportStatus::Partial, "{id} status");
        assert!(metadata.supported_subset().is_some(), "{id} subset");
        assert!(metadata.unsupported_remainder().is_some(), "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    }

    let metadata =
        feature_metadata("later.rule.font-feature-values").expect("font-feature-values metadata");
    assert_eq!(metadata.status(), CssSupportStatus::RecognizedUnsupported);
    assert_eq!(
        metadata.recognized_unsupported_code(),
        Some(CssErrorCode::UnsupportedAtRule)
    );
}

#[test]
fn c14_closure_metadata_and_docs_are_truthful() {
    for (id, kind, spelling, source, production) in [
        (
            "official.property.flex-flow",
            CssFeatureKind::Property,
            "flex-flow",
            "O-FLEXBOX1",
            "#propdef-flex-flow",
        ),
        (
            "official.property.column-count",
            CssFeatureKind::Property,
            "column-count",
            "O-MULTICOL1",
            "#propdef-column-count",
        ),
        (
            "official.property.column-fill",
            CssFeatureKind::Property,
            "column-fill",
            "O-MULTICOL1",
            "#propdef-column-fill",
        ),
        (
            "official.property.column-rule",
            CssFeatureKind::Property,
            "column-rule",
            "O-MULTICOL1",
            "#propdef-column-rule",
        ),
        (
            "official.property.column-rule-color",
            CssFeatureKind::Property,
            "column-rule-color",
            "O-MULTICOL1",
            "#propdef-column-rule-color",
        ),
        (
            "official.property.column-rule-style",
            CssFeatureKind::Property,
            "column-rule-style",
            "O-MULTICOL1",
            "#propdef-column-rule-style",
        ),
        (
            "official.property.column-rule-width",
            CssFeatureKind::Property,
            "column-rule-width",
            "O-MULTICOL1",
            "#propdef-column-rule-width",
        ),
        (
            "official.property.column-span",
            CssFeatureKind::Property,
            "column-span",
            "O-MULTICOL1",
            "#propdef-column-span",
        ),
        (
            "official.property.column-width",
            CssFeatureKind::Property,
            "column-width",
            "O-MULTICOL1",
            "#propdef-column-width",
        ),
        (
            "official.property.columns",
            CssFeatureKind::Property,
            "columns",
            "O-MULTICOL1",
            "#propdef-columns",
        ),
        (
            "official.rule.at-rule",
            CssFeatureKind::Rule,
            "generic at-rule",
            "O-SYNTAX3",
            "#at-rules,#consume-at-rule",
        ),
        (
            "official.qualified-rule.generic",
            CssFeatureKind::Rule,
            "generic qualified rule",
            "O-SYNTAX3",
            "#consume-qualified-rule",
        ),
        (
            "official.declaration.generic",
            CssFeatureKind::Declaration,
            "generic declaration",
            "O-SYNTAX3",
            "#consume-declaration",
        ),
        (
            "official.value.stylesheet",
            CssFeatureKind::Value,
            "<stylesheet>",
            "O-SYNTAX3",
            "#parser-entry-points",
        ),
        (
            "official.value.rule-list",
            CssFeatureKind::Value,
            "<rule-list>",
            "O-SYNTAX3",
            "#declaration-rule-list",
        ),
        (
            "official.value.declaration-list",
            CssFeatureKind::Value,
            "<declaration-list>",
            "O-SYNTAX3",
            "#declaration-rule-list",
        ),
        (
            "official.value.style-block",
            CssFeatureKind::Value,
            "<style-block>",
            "O-SYNTAX3",
            "#declaration-rule-list",
        ),
        (
            "official.value.syntax-token-stream",
            CssFeatureKind::Value,
            "syntax token stream",
            "O-SYNTAX3",
            "#tokenization",
        ),
        (
            "official.value.component-value",
            CssFeatureKind::Value,
            "<component-value>",
            "O-SYNTAX3",
            "#consume-component-value",
        ),
        (
            "official.value.simple-block",
            CssFeatureKind::Value,
            "simple block",
            "O-SYNTAX3",
            "#consume-simple-block",
        ),
        (
            "official.value.function",
            CssFeatureKind::Value,
            "function",
            "O-SYNTAX3",
            "#consume-function",
        ),
        (
            "official.value.declaration-value",
            CssFeatureKind::Value,
            "<declaration-value>",
            "O-SYNTAX3",
            "#any-value",
        ),
        (
            "official.value.any-value",
            CssFeatureKind::Value,
            "<any-value>",
            "O-SYNTAX3",
            "#any-value",
        ),
        (
            "official.value.an-plus-b",
            CssFeatureKind::Value,
            "<an+b>",
            "O-SYNTAX3",
            "#the-anb-type",
        ),
        (
            "official.value.unicode-range",
            CssFeatureKind::Value,
            "<urange>",
            "O-SYNTAX3",
            "#urange-syntax",
        ),
        (
            "official.value.css-wide-keyword",
            CssFeatureKind::Value,
            "CSS-wide keyword",
            "O-CASCADE4",
            "#defaulting-keywords",
        ),
        (
            "official.value.custom-ident",
            CssFeatureKind::Value,
            "<custom-ident>",
            "O-VALUES3",
            "#custom-idents",
        ),
        (
            "official.value.ident",
            CssFeatureKind::Value,
            "<ident>",
            "O-VALUES3",
            "#custom-idents",
        ),
        (
            "official.value.string",
            CssFeatureKind::Value,
            "<string>",
            "O-VALUES3",
            "#strings",
        ),
        (
            "official.value.url",
            CssFeatureKind::Value,
            "<url>",
            "O-VALUES3",
            "#urls",
        ),
        (
            "official.value.url-modifier",
            CssFeatureKind::Value,
            "<url-modifier>",
            "O-VALUES3",
            "#url-modifiers",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
    }

    for (id, spelling, production) in [
        ("official.value.dimension", "<dimension>", "#dimensions"),
        ("official.value.angle", "<angle>", "#angles"),
        (
            "official.value.angle-percentage",
            "<angle-percentage>",
            "#mixed-percentages",
        ),
        (
            "official.value.time-percentage",
            "<time-percentage>",
            "#mixed-percentages",
        ),
        ("official.value.frequency", "<frequency>", "#frequency"),
        (
            "official.value.frequency-percentage",
            "<frequency-percentage>",
            "#mixed-percentages",
        ),
        (
            "official.value.calc",
            "calc()",
            "#calc-notation,#calc-syntax,#calc-type-checking",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
        assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), "O-VALUES3", "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
    }
}

#[test]
fn c14_remaining_official_values_are_typed() {
    let dimension =
        CssLengthDimension::try_new(-1.5, CssLengthUnit::Cqw).expect("finite dimension");
    assert_eq!(dimension.value(), -1.5);
    assert_eq!(dimension.unit(), CssLengthUnit::Cqw);
    assert!(CssLengthDimension::try_new(f32::INFINITY, CssLengthUnit::Px).is_none());

    let angle = CssAngleCalculation::try_literal(-0.5, CssAngleUnit::Turns).expect("finite angle");
    let percentage = CssPercentageCalculation::try_literal(25.0).expect("finite percentage");
    let time =
        CssTimeCalculation::try_literal(-250.0, CssTimeUnit::Milliseconds).expect("finite time");
    let frequency = CssFrequencyCalculation::try_literal(1.5, CssFrequencyUnit::Kilohertz)
        .expect("finite frequency");
    assert_eq!(angle.result_type(), CssCalculationType::Angle);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    assert_eq!(time.result_type(), CssCalculationType::Time);
    assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
    assert!(CssAngleCalculation::try_literal(f32::NAN, CssAngleUnit::Degrees).is_none());
    assert!(CssTimeCalculation::try_literal(f32::INFINITY, CssTimeUnit::Seconds).is_none());
    assert!(
        CssFrequencyCalculation::try_literal(f32::NEG_INFINITY, CssFrequencyUnit::Hertz).is_none()
    );

    let parsed = parse_style_attribute(concat!(
        "width: calc((1cqw + 2%) * 3); ",
        "transform: rotate(calc((1turn + 180deg) / 2)); ",
        "transition-delay: calc((-1s + 250ms) * 2); ",
        "filter: hue-rotate(calc((1turn - 90deg) / 3))",
    ));
    assert!(parsed.is_clean(), "{:?}", parsed.diagnostics());
    assert_eq!(
        parsed
            .syntax()
            .iter()
            .map(|declaration| declaration.known().expect("known value").property())
            .collect::<Vec<_>>(),
        [
            CssKnownProperty::Width,
            CssKnownProperty::Transform,
            CssKnownProperty::TransitionDelay,
            CssKnownProperty::Filter,
        ],
    );

    let recovered = parse_style_attribute(concat!(
        "width: 1deg; ",
        "rotate: 1hz; ",
        "transition-delay: 1px; ",
        "color: red",
    ));
    assert_eq!(recovered.syntax().len(), 1);
    assert_eq!(
        recovered.syntax()[0]
            .known()
            .expect("retained sibling")
            .property(),
        CssKnownProperty::Color,
    );
    assert_eq!(recovered.diagnostics().len(), 3);
    assert!(
        recovered
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );

    for (id, spelling, production) in [
        ("official.value.dimension", "<dimension>", "#dimensions"),
        ("official.value.angle", "<angle>", "#angles"),
        (
            "official.value.angle-percentage",
            "<angle-percentage>",
            "#mixed-percentages",
        ),
        (
            "official.value.time-percentage",
            "<time-percentage>",
            "#mixed-percentages",
        ),
        ("official.value.frequency", "<frequency>", "#frequency"),
        (
            "official.value.frequency-percentage",
            "<frequency-percentage>",
            "#mixed-percentages",
        ),
        (
            "official.value.calc",
            "calc()",
            "#calc-notation,#calc-syntax,#calc-type-checking",
        ),
    ] {
        assert_c03_value_metadata(
            id,
            spelling,
            production,
            CssSupportStatus::Complete,
            None,
            None,
        );
    }
}

#[test]
fn c14_retained_partial_extensions_have_direct_public_evidence() {
    for (id, kind, input) in [
        (
            "ext.value.relative-color",
            CssFeatureKind::Value,
            Input::Style("color: rgb(from red r g b / alpha)"),
        ),
        (
            "ext.value.color-mix",
            CssFeatureKind::Value,
            Input::Style("color: color-mix(in oklch longer hue, red 25%, blue 75%)"),
        ),
        (
            "ext.value.grid-repeat",
            CssFeatureKind::Value,
            Input::Style("grid-template-columns: repeat(auto-fit, 10px)"),
        ),
        (
            "ext.value.basic-shape",
            CssFeatureKind::Value,
            Input::Style("clip-path: circle(50% at center)"),
        ),
        (
            "ext.descriptor.font-weight-range",
            CssFeatureKind::Descriptor,
            Input::Sheet(
                "@font-face { font-family: Demo; src: url(demo.woff2); font-weight: 300 700; }",
            ),
        ),
        (
            "ext.descriptor.font-style-oblique-range",
            CssFeatureKind::Descriptor,
            Input::Sheet(
                "@font-face { font-family: Demo; src: url(demo.woff2); font-style: oblique -10deg 20deg; }",
            ),
        ),
        (
            "ext.descriptor.font-stretch-range",
            CssFeatureKind::Descriptor,
            Input::Sheet(
                "@font-face { font-family: Demo; src: url(demo.woff2); font-stretch: 75% 125%; }",
            ),
        ),
        (
            "ext.value.font-source-modern-hints",
            CssFeatureKind::Value,
            Input::Sheet(
                "@font-face { font-family: Demo; src: url(demo.woff2) format(woff2) tech(variations, color-colrv1); }",
            ),
        ),
        (
            "ext.property.font-weight-range",
            CssFeatureKind::Property,
            Input::Style("font-weight: 725"),
        ),
        (
            "ext.supports.selector",
            CssFeatureKind::Selector,
            Input::Sheet("@supports selector(.card:is(.primary, .secondary)) {}"),
        ),
        (
            "ext.media.range.width",
            CssFeatureKind::MediaQuery,
            Input::Sheet("@media (width >= 1px) {}"),
        ),
        (
            "ext.media.range.height",
            CssFeatureKind::MediaQuery,
            Input::Sheet("@media (height < 100vh) {}"),
        ),
        (
            "ext.media.range.resolution",
            CssFeatureKind::MediaQuery,
            Input::Sheet("@media (resolution >= 2dppx) {}"),
        ),
        (
            "ext.media.range.color",
            CssFeatureKind::MediaQuery,
            Input::Sheet("@media (color > 0) {}"),
        ),
        (
            "ext.media.range.monochrome",
            CssFeatureKind::MediaQuery,
            Input::Sheet("@media (monochrome = 1) {}"),
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.status(), CssSupportStatus::Partial, "{id} status");
        assert!(metadata.supported_subset().is_some(), "{id} subset");
        assert!(metadata.unsupported_remainder().is_some(), "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
        assert!(diagnostics(input).is_empty(), "{id} public positive vector");
    }

    let unsupported = parse_sheet(concat!(
        "@font-feature-values Demo { @styleset { nice: 1; } } ",
        ".tail { color: red; }",
    ));
    let [CssRule::Style(_)] = unsupported.syntax().rules() else {
        panic!("later sibling must survive the unsupported at-rule");
    };
    let [diagnostic] = unsupported.diagnostics() else {
        panic!("expected one unsupported-at-rule diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnsupportedAtRule);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    let ErrorKind::UnsupportedAtRule(detail) = diagnostic.error().kind() else {
        panic!("expected typed unsupported-at-rule detail");
    };
    assert_eq!(detail.name().as_str(), "font-feature-values");
    assert_eq!(detail.feature().as_str(), "later.rule.font-feature-values");

    let metadata =
        feature_metadata("later.rule.font-feature-values").expect("font-feature-values metadata");
    assert_eq!(metadata.kind(), CssFeatureKind::Rule);
    assert_eq!(metadata.spelling(), "@font-feature-values");
    assert_eq!(metadata.source().id().as_str(), "I-FONTS4");
    assert_eq!(metadata.production(), "#font-feature-values-rule");
    assert_eq!(metadata.status(), CssSupportStatus::RecognizedUnsupported);
    assert_eq!(
        metadata.recognized_unsupported_code(),
        Some(CssErrorCode::UnsupportedAtRule),
    );
}

fn assert_c03_timing_metadata(
    id: &str,
    property: &str,
    source: &str,
    production: &str,
    status: CssSupportStatus,
    subset: Option<&str>,
    remainder: Option<&str>,
) {
    let report = parse_style_attribute(property);
    assert!(report.is_clean(), "{property}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("{property}: expected one retained declaration");
    };
    let known = declaration.known().expect("known timing declaration");
    assert_eq!(known.property().stable_id(), id, "{property} identity");

    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
    assert_eq!(metadata.id().as_str(), id);
    assert_eq!(metadata.kind(), CssFeatureKind::Property, "{id} kind");
    assert_eq!(
        metadata.spelling(),
        known.property().canonical_name(),
        "{id} spelling"
    );
    assert_eq!(metadata.source().id().as_str(), source, "{id} source");
    assert_eq!(metadata.production(), production, "{id} production");
    assert_eq!(metadata.status(), status, "{id} status");
    assert_eq!(metadata.supported_subset(), subset, "{id} subset");
    assert_eq!(
        metadata.unsupported_remainder(),
        remainder,
        "{id} remainder"
    );
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
}

fn assert_complete_position_value_metadata(
    id: &str,
    spelling: &str,
    source: &str,
    production: &str,
) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
    assert_eq!(metadata.id().as_str(), id);
    assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id} kind");
    assert_eq!(metadata.spelling(), spelling, "{id} spelling");
    assert_eq!(metadata.source().id().as_str(), source, "{id} source");
    assert_eq!(metadata.production(), production, "{id} production");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
    assert_eq!(metadata.supported_subset(), None, "{id} subset");
    assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
}

fn assert_complete_position_property_metadata(
    id: &str,
    canonical_name: &str,
    property: CssKnownProperty,
    source: &str,
    production: &str,
) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
    let property_owner = property_metadata(canonical_name)
        .unwrap_or_else(|| panic!("missing `{canonical_name}` property metadata"));
    assert!(
        std::ptr::eq(metadata, property_owner.feature()),
        "{id} owner"
    );
    assert_eq!(property_owner.property(), property, "{id} property owner");
    assert_eq!(property_owner.canonical_name(), canonical_name, "{id} name");
    assert_eq!(metadata.id().as_str(), id);
    assert_eq!(metadata.kind(), CssFeatureKind::Property, "{id} kind");
    assert_eq!(metadata.spelling(), canonical_name, "{id} spelling");
    assert_eq!(metadata.source().id().as_str(), source, "{id} source");
    assert_eq!(metadata.production(), production, "{id} production");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
    assert_eq!(metadata.supported_subset(), None, "{id} subset");
    assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
}

#[test]
fn official_position_metadata_matches_generic_position_behavior() {
    let report = parse_style_attribute("object-position: right 5% bottom 2px");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ObjectPosition(value) = report.syntax()[0]
        .known()
        .expect("known object-position")
        .property_value()
        .expect("ordinary object-position")
    else {
        panic!("expected object-position value");
    };
    assert!(matches!(
        value.position().value().horizontal(),
        CssHorizontalPosition::RightOffset(offset)
            if matches!(offset.value(), CssLength::Percent(value) if value.value() == 5.0)
    ));
    assert!(matches!(
        value.position().value().vertical(),
        CssVerticalPosition::BottomOffset(offset)
            if matches!(offset.value(), CssLength::Px(value) if value.value() == 2.0)
    ));

    assert_complete_position_value_metadata(
        "official.value.position",
        "<position>",
        "O-VALUES3",
        "#position",
    );
}

#[test]
fn official_background_position_metadata_matches_three_component_layer_behavior() {
    let report = parse_style_attribute("background-position: left 10px top");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundPosition(value) = report.syntax()[0]
        .known()
        .expect("known background-position")
        .property_value()
        .expect("ordinary background-position")
    else {
        panic!("expected background-position value");
    };
    let [layer] = value.positions().positions() else {
        panic!("expected one background-position layer");
    };
    assert!(matches!(
        layer.horizontal(),
        CssHorizontalPosition::LeftOffset(offset)
            if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)
    ));
    assert!(matches!(layer.vertical(), CssVerticalPosition::Top));

    assert_complete_position_value_metadata(
        "official.value.background-position",
        "<bg-position>#",
        "O-BACKGROUNDS3",
        "#background-position",
    );
}

#[test]
fn background_position_property_metadata_matches_layer_list_behavior() {
    let report = parse_style_attribute("background-position: left top, right 4px bottom 8%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0]
        .known()
        .expect("known background-position");
    assert_eq!(declaration.property(), CssKnownProperty::BackgroundPosition);
    let CssKnownPropertyValueRef::BackgroundPosition(value) = declaration
        .property_value()
        .expect("ordinary background-position")
    else {
        panic!("expected background-position value");
    };
    assert_eq!(value.positions().positions().len(), 2);

    assert_complete_position_property_metadata(
        "baseline.property.background-position",
        "background-position",
        CssKnownProperty::BackgroundPosition,
        "O-BACKGROUNDS3",
        "#propdef-background-position",
    );
}

#[test]
fn object_position_property_metadata_matches_current_accessor_behavior() {
    let report = parse_style_attribute("object-position: center 25%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0].known().expect("known object-position");
    assert_eq!(declaration.property(), CssKnownProperty::ObjectPosition);
    let CssKnownPropertyValueRef::ObjectPosition(value) = declaration
        .property_value()
        .expect("ordinary object-position")
    else {
        panic!("expected object-position value");
    };
    assert!(matches!(
        value.position().value().horizontal(),
        CssHorizontalPosition::Center
    ));
    assert!(matches!(
        value.position().value().vertical(),
        CssVerticalPosition::Offset(offset)
            if matches!(offset.value(), CssLength::Percent(value) if value.value() == 25.0)
    ));

    assert_complete_position_property_metadata(
        "official.property.object-position",
        "object-position",
        CssKnownProperty::ObjectPosition,
        "O-IMAGES3",
        "#propdef-object-position",
    );
}

#[test]
fn transform_origin_property_metadata_matches_directed_z_behavior() {
    let report = parse_style_attribute("transform-origin: top 50px");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0].known().expect("known transform-origin");
    assert_eq!(declaration.property(), CssKnownProperty::TransformOrigin);
    let CssKnownPropertyValueRef::TransformOrigin(value) = declaration
        .property_value()
        .expect("ordinary transform-origin")
    else {
        panic!("expected transform-origin value");
    };
    assert!(matches!(
        value.origin().horizontal(),
        CssHorizontalPosition::Center
    ));
    assert!(matches!(
        value.origin().vertical(),
        CssVerticalPosition::Top
    ));
    assert!(matches!(
        value.origin().z().map(|z| z.value()),
        Some(CssLength::Px(value)) if value.value() == 50.0
    ));

    assert_complete_position_property_metadata(
        "baseline.property.transform-origin",
        "transform-origin",
        CssKnownProperty::TransformOrigin,
        "O-TRANSFORMS1",
        "#propdef-transform-origin",
    );
}

#[test]
fn mask_position_property_metadata_matches_generic_layer_behavior() {
    let report = parse_style_attribute("mask-position: left 10px bottom 20%, center 25%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax()[0].known().expect("known mask-position");
    assert_eq!(declaration.property(), CssKnownProperty::MaskPosition);
    let CssKnownPropertyValueRef::MaskPosition(value) = declaration
        .property_value()
        .expect("ordinary mask-position")
    else {
        panic!("expected mask-position value");
    };
    let [first, second] = value.positions().positions() else {
        panic!("expected two mask-position layers");
    };
    assert!(matches!(
        first.value().horizontal(),
        CssHorizontalPosition::LeftOffset(offset)
            if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)
    ));
    assert!(matches!(
        second.value().horizontal(),
        CssHorizontalPosition::Center
    ));

    assert_complete_position_property_metadata(
        "baseline.property.mask-position",
        "mask-position",
        CssKnownProperty::MaskPosition,
        "S-MASKING1",
        "#propdef-mask-position",
    );
}

#[test]
fn official_integer_metadata_matches_checked_integer_behavior() {
    let value = CssIntegerCalculation::literal(-3);
    assert_eq!(value.result_type(), CssCalculationType::Integer);
    assert_c03_value_metadata(
        "official.value.integer",
        "<integer>",
        "#integers",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_number_metadata_matches_checked_number_behavior() {
    let value = CssNumberCalculation::try_literal(-3.5).expect("finite number");
    assert_eq!(value.result_type(), CssCalculationType::Number);
    assert_c03_value_metadata(
        "official.value.number",
        "<number>",
        "#numbers",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_dimension_metadata_matches_checked_dimension_behavior() {
    let value = CssLengthDimension::try_new(1.5, CssLengthUnit::Rem).expect("finite dimension");
    assert_eq!(value.value(), 1.5);
    assert_eq!(value.unit(), CssLengthUnit::Rem);
    assert_c03_value_metadata(
        "official.value.dimension",
        "<dimension>",
        "#dimensions",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_percentage_metadata_matches_checked_percentage_behavior() {
    let value = CssPercentageCalculation::try_literal(-12.5).expect("finite percentage");
    assert_eq!(value.result_type(), CssCalculationType::Percentage);
    assert_c03_value_metadata(
        "official.value.percentage",
        "<percentage>",
        "#percentages",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_length_metadata_matches_checked_length_behavior() {
    let value = CssLengthCalculation::try_dimension(2.0, CssLengthUnit::Cqw)
        .expect("finite length dimension");
    assert_eq!(value.result_type(), CssCalculationType::Length);
    assert_c03_value_metadata(
        "official.value.length",
        "<length>",
        "#lengths",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_length_percentage_metadata_matches_mixed_length_parser_behavior() {
    let report = parse_style_attribute("width: calc((1px + 2%) * 3)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Width
    );
    assert_c03_value_metadata(
        "official.value.length-percentage",
        "<length-percentage>",
        "#mixed-percentages",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_angle_metadata_matches_checked_angle_behavior() {
    let value = CssAngleCalculation::try_literal(-0.5, CssAngleUnit::Turns).expect("finite angle");
    assert_eq!(value.result_type(), CssCalculationType::Angle);
    assert_c03_value_metadata(
        "official.value.angle",
        "<angle>",
        "#angles",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_angle_percentage_metadata_matches_checked_mixed_models() {
    let angle =
        CssAngleCalculation::try_literal(45.0, CssAngleUnit::Degrees).expect("finite angle");
    let percentage = CssPercentageCalculation::try_literal(25.0).expect("finite percentage");
    assert_eq!(angle.result_type(), CssCalculationType::Angle);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    assert_c03_value_metadata(
        "official.value.angle-percentage",
        "<angle-percentage>",
        "#mixed-percentages",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_time_metadata_matches_checked_time_behavior() {
    let value =
        CssDelayLiteral::try_new(-250.0, CssTimeUnit::Milliseconds).expect("finite signed time");
    assert_eq!(value.value(), -250.0);
    assert_eq!(value.unit(), CssTimeUnit::Milliseconds);
    assert_c03_value_metadata(
        "official.value.time",
        "<time>",
        "#time",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_time_percentage_metadata_matches_checked_mixed_models() {
    let time = CssTimeCalculation::try_literal(-1.0, CssTimeUnit::Seconds).expect("finite time");
    let percentage = CssPercentageCalculation::try_literal(50.0).expect("finite percentage");
    assert_eq!(time.result_type(), CssCalculationType::Time);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    assert_c03_value_metadata(
        "official.value.time-percentage",
        "<time-percentage>",
        "#mixed-percentages",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_frequency_metadata_matches_checked_frequency_behavior() {
    let value = CssFrequencyCalculation::try_literal(440.0, CssFrequencyUnit::Hertz)
        .expect("finite frequency");
    assert_eq!(value.result_type(), CssCalculationType::Frequency);
    assert_c03_value_metadata(
        "official.value.frequency",
        "<frequency>",
        "#frequency",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_frequency_percentage_metadata_matches_checked_mixed_models() {
    let frequency = CssFrequencyCalculation::try_literal(1.5, CssFrequencyUnit::Kilohertz)
        .expect("finite frequency");
    let percentage = CssPercentageCalculation::try_literal(75.0).expect("finite percentage");
    assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
    assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
    assert_c03_value_metadata(
        "official.value.frequency-percentage",
        "<frequency-percentage>",
        "#mixed-percentages",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_resolution_metadata_matches_checked_resolution_behavior() {
    let value = CssResolution::try_new(2.0, CssResolutionUnit::Dppx).expect("finite resolution");
    assert_eq!(value.value().value(), 2.0);
    assert_eq!(value.unit(), CssResolutionUnit::Dppx);
    assert_c03_value_metadata(
        "official.value.resolution",
        "<resolution>",
        "#resolution",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn official_calc_metadata_matches_typed_calculation_parser_behavior() {
    let report = parse_style_attribute("width: calc((1px + 2%) * 3)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Width
    );
    assert_c03_value_metadata(
        "official.value.calc",
        "calc()",
        "#calc-notation,#calc-syntax,#calc-type-checking",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn transition_duration_metadata_matches_typed_duration_behavior() {
    assert_c03_timing_metadata(
        "baseline.property.transition-duration",
        "transition-duration: calc((1s + 250ms) * 2)",
        "I-TRANSITIONS1",
        "#propdef-transition-duration",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn transition_delay_metadata_matches_signed_delay_behavior() {
    assert_c03_timing_metadata(
        "baseline.property.transition-delay",
        "transition-delay: -250ms",
        "I-TRANSITIONS1",
        "#propdef-transition-delay",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn animation_duration_metadata_matches_typed_duration_behavior() {
    assert_c03_timing_metadata(
        "baseline.property.animation-duration",
        "animation-duration: calc(1s + 250ms)",
        "I-ANIMATIONS1",
        "#propdef-animation-duration",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn animation_delay_metadata_matches_signed_delay_behavior() {
    assert_c03_timing_metadata(
        "baseline.property.animation-delay",
        "animation-delay: -1s",
        "I-ANIMATIONS1",
        "#propdef-animation-delay",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn animation_iteration_count_metadata_matches_typed_iteration_behavior() {
    assert_c03_timing_metadata(
        "baseline.property.animation-iteration-count",
        "animation-iteration-count: calc((1 + 2) * 3)",
        "I-ANIMATIONS1",
        "#propdef-animation-iteration-count",
        CssSupportStatus::Complete,
        None,
        None,
    );
}

#[test]
fn transition_metadata_matches_c03_shorthand_behavior_and_c05_remainder() {
    assert_c03_timing_metadata(
        "baseline.property.transition",
        "transition: opacity calc(1s + 250ms) linear -200ms",
        "I-TRANSITIONS1",
        "#propdef-transition",
        CssSupportStatus::Partial,
        Some(TIMING_SUBSET),
        Some(TIMING_REMAINDER),
    );
}

#[test]
fn animation_metadata_matches_c03_shorthand_behavior_and_c05_remainder() {
    assert_c03_timing_metadata(
        "baseline.property.animation",
        "animation: fade calc(1s + 250ms) linear calc((1 + 2) * 3) -200ms both running",
        "I-ANIMATIONS1",
        "#propdef-animation",
        CssSupportStatus::Partial,
        Some(TIMING_SUBSET),
        Some(TIMING_REMAINDER),
    );
}

#[test]
fn media_conditional_and_import_metadata_are_truthful() {
    for source in [
        "@media speech and (device-width: 800px) and (resolution: 2dpcm) { .x { color: red; } }",
        "@supports (display: grid) and future-layout(mode) { .x { color: red; } }",
        "@supports selector(.card > .item:hover) { .x { color: red; } }",
        "@layer reset; @import url(theme.css) layer(theme) supports(display: grid) print;",
    ] {
        let report = parse_sheet(source);
        assert!(
            report.is_clean(),
            "paired grammar behavior must be clean for {source:?}: {:?}",
            report.diagnostics()
        );
    }

    let assert_complete = |id: &str, kind: CssFeatureKind, source_id: &str, production: &str| {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.source().id().as_str(), source_id, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(
            metadata.recognized_unsupported_code(),
            None,
            "{id} recognized code"
        );
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} alias");
    };

    assert_complete(
        "official.media.query-list-core",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#syntax",
    );
    assert_complete(
        "baseline.media.type",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#media1",
    );
    assert_complete(
        "official.media.feature.width",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#width",
    );
    assert_complete(
        "official.media.feature.height",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#height",
    );
    assert_complete(
        "official.media.feature.device-width",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#device-width",
    );
    assert_complete(
        "official.media.feature.device-height",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#device-height",
    );
    assert_complete(
        "official.media.feature.orientation",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#orientation",
    );
    assert_complete(
        "official.media.feature.aspect-ratio",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#aspect-ratio",
    );
    assert_complete(
        "official.media.feature.device-aspect-ratio",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#device-aspect-ratio",
    );
    assert_complete(
        "official.media.feature.color",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#color",
    );
    assert_complete(
        "official.media.feature.color-index",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#color-index",
    );
    assert_complete(
        "official.media.feature.monochrome",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#monochrome",
    );
    assert_complete(
        "official.media.feature.resolution",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#resolution",
    );
    assert_complete(
        "official.media.feature.scan",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#scan",
    );
    assert_complete(
        "official.media.feature.grid",
        CssFeatureKind::MediaQuery,
        "O-MEDIA3",
        "#grid",
    );

    assert_complete(
        "baseline.rule.media",
        CssFeatureKind::Rule,
        "O-CONDITIONAL3",
        "#at-media",
    );
    assert_complete(
        "later.rule.supports",
        CssFeatureKind::Rule,
        "O-CONDITIONAL3",
        "#at-supports",
    );
    assert_complete(
        "official.rule.conditional-group-context",
        CssFeatureKind::Rule,
        "O-CONDITIONAL3",
        "#contents,#placement",
    );
    assert_complete(
        "baseline.rule.import",
        CssFeatureKind::Rule,
        "O-CASCADE4",
        "#at-import",
    );

    assert_complete(
        "ext.media.resolution.dppx",
        CssFeatureKind::MediaQuery,
        "R-MEDIA4",
        "#resolution",
    );
    assert_complete(
        "ext.supports.general-enclosed",
        CssFeatureKind::Value,
        "X-VALUES4",
        "css-values-4/Overview.bs#general-enclosed",
    );
    let general_enclosed_source =
        specification_source("X-VALUES4").expect("immutable general-enclosed source");
    assert_eq!(
        general_enclosed_source.tier(),
        CssSpecificationTier::SurgeistExtension
    );
    assert_eq!(general_enclosed_source.url(), None);
    assert_eq!(
        general_enclosed_source.repository_provenance(),
        Some("720ea2863696971ea6a6744e0f23acbb3e6936bd:css-values-4/Overview.bs")
    );
    assert_complete(
        "ext.import.layer",
        CssFeatureKind::Rule,
        "R-CASCADE5",
        "#at-import",
    );
    assert_complete(
        "ext.stylesheet.prelude-order",
        CssFeatureKind::Rule,
        "R-CASCADE5",
        "#at-import",
    );

    let selector =
        feature_metadata("ext.supports.selector").expect("Conditional 4 selector-test metadata");
    assert_eq!(selector.kind(), CssFeatureKind::Selector);
    assert_eq!(selector.source().id().as_str(), "R-CONDITIONAL4");
    assert_eq!(selector.production(), "#at-supports");
    assert_eq!(selector.status(), CssSupportStatus::Partial);
    assert_eq!(selector.supported_subset(), Some(SUPPORTS_SELECTOR_SUBSET));
    assert_eq!(
        selector.unsupported_remainder(),
        Some(SUPPORTS_SELECTOR_REMAINDER)
    );
    assert_eq!(selector.recognized_unsupported_code(), None);
    assert!(selector.baseline_alias_targets().is_empty());
}

#[test]
fn selectors3_and_namespace_metadata_are_truthful() {
    let selectors = parse_sheet(concat!(
        ":root,:link,:visited,:target,:lang(en),:hover,:active,:focus,",
        ":enabled,:disabled,:checked,:indeterminate,:first-child,:last-child,",
        ":only-child,:empty,:nth-child(2n+1),:nth-last-child(2),",
        ":first-of-type,:last-of-type,:only-of-type,:nth-of-type(odd),",
        ":nth-last-of-type(even),:not(.excluded) { color: red; }",
        "article#first#second.card[data-ready][lang|=\"en\"][title^=\"lead\"] ",
        "> section + a ~ p { color: red; }",
        ".line::first-line { color: red; }",
        ".letter:first-letter { color: red; }",
        ".generated:before { color: red; }",
    ));
    assert!(
        selectors.is_clean(),
        "paired Selectors 3 grammar behavior: {:?}",
        selectors.diagnostics()
    );

    let qualified = parse_sheet(concat!(
        "@namespace svg \"urn:svg\";",
        "svg|a,svg|*,*|a,|a,[svg|href],[*|title],[|lang],[plain] { color: red; }",
        "@supports selector(svg|a) {}",
    ));
    assert!(
        qualified.is_clean(),
        "paired namespace grammar behavior: {:?}",
        qualified.diagnostics()
    );
    assert!(matches!(
        qualified.syntax().rules().first(),
        Some(CssRule::Namespace(_))
    ));
    let Some(CssRule::Supports(typed)) = qualified.syntax().rules().last() else {
        panic!("expected a final typed supports rule")
    };
    assert!(matches!(
        typed.condition().kind(),
        CssSupportsConditionKind::Selector(_)
    ));

    let general_enclosed = parse_sheet("@supports selector(.x || .y) {}");
    assert!(
        general_enclosed.is_clean(),
        "balanced selector remainder: {:?}",
        general_enclosed.diagnostics()
    );
    let [CssRule::Supports(general_enclosed)] = general_enclosed.syntax().rules() else {
        panic!("expected general-enclosed supports rule")
    };
    assert!(matches!(
        general_enclosed.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(value)
            if value.authored() == "selector(.x || .y)"
    ));

    let assert_complete = |id: &str, kind: CssFeatureKind, source: &str, production: &str| {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(
            metadata.recognized_unsupported_code(),
            None,
            "{id} recognized code"
        );
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
    };

    assert_complete(
        "official.selector.group",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#grouping",
    );
    assert_complete(
        "official.selector.type",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#type-selectors",
    );
    assert_complete(
        "official.selector.universal",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#universal-selector",
    );
    assert_complete(
        "official.selector.attribute-presence-value",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#attribute-representation",
    );
    assert_complete(
        "official.selector.attribute-substring",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#attribute-substrings",
    );
    assert_complete(
        "official.selector.class",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#class-html",
    );
    assert_complete(
        "official.selector.id",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#id-selectors",
    );
    assert_complete(
        "official.selector.dynamic",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#dynamic-pseudos",
    );
    assert_complete(
        "official.selector.target",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#target-pseudo",
    );
    assert_complete(
        "official.selector.lang",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#lang-pseudo",
    );
    assert_complete(
        "official.selector.ui-state",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#UIstates",
    );
    assert_complete(
        "official.selector.structural",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#structural-pseudos",
    );
    assert_complete(
        "official.selector.negation",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#negation",
    );
    assert_complete(
        "official.selector.first-line",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#first-line",
    );
    assert_complete(
        "official.selector.first-letter",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#first-letter",
    );
    assert_complete(
        "official.selector.generated",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#gen-content",
    );
    assert_complete(
        "official.selector.combinator.descendant",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#descendant-combinators",
    );
    assert_complete(
        "official.selector.combinator.child",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#child-combinators",
    );
    assert_complete(
        "official.selector.combinator.next-sibling",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#adjacent-sibling-combinators",
    );
    assert_complete(
        "official.selector.combinator.subsequent-sibling",
        CssFeatureKind::Selector,
        "O-SELECTORS3",
        "#general-sibling-combinators",
    );
    assert_complete(
        "later.rule.namespace",
        CssFeatureKind::Rule,
        "O-NAMESPACES3",
        "#declaration,#syntax",
    );
    assert_complete(
        "official.selector.namespace-qualified-name",
        CssFeatureKind::Selector,
        "O-NAMESPACES3",
        "#scope,#prefixes,#css-qnames",
    );

    let supports_selector =
        feature_metadata("ext.supports.selector").expect("selector() extension metadata");
    assert_eq!(supports_selector.id().as_str(), "ext.supports.selector");
    assert_eq!(supports_selector.kind(), CssFeatureKind::Selector);
    assert_eq!(supports_selector.source().id().as_str(), "R-CONDITIONAL4");
    assert_eq!(supports_selector.production(), "#at-supports");
    assert_eq!(supports_selector.status(), CssSupportStatus::Partial);
    assert_eq!(
        supports_selector.supported_subset(),
        Some(SUPPORTS_SELECTOR_SUBSET)
    );
    assert_eq!(
        supports_selector.unsupported_remainder(),
        Some(SUPPORTS_SELECTOR_REMAINDER)
    );
    assert_eq!(supports_selector.recognized_unsupported_code(), None);
    assert!(supports_selector.baseline_alias_targets().is_empty());
}

#[test]
fn counter_styles_and_page_metadata_are_truthful() {
    let counter_styles = [
        (
            "later.rule.counter-style",
            CssFeatureKind::Rule,
            "@counter-style",
            "#the-counter-style-rule",
        ),
        (
            "official.descriptor.counter-style.system",
            CssFeatureKind::Descriptor,
            "system in @counter-style",
            "#counter-style-system",
        ),
        (
            "official.descriptor.counter-style.negative",
            CssFeatureKind::Descriptor,
            "negative in @counter-style",
            "#counter-style-negative",
        ),
        (
            "official.descriptor.counter-style.prefix",
            CssFeatureKind::Descriptor,
            "prefix in @counter-style",
            "#counter-style-prefix",
        ),
        (
            "official.descriptor.counter-style.suffix",
            CssFeatureKind::Descriptor,
            "suffix in @counter-style",
            "#counter-style-suffix",
        ),
        (
            "official.descriptor.counter-style.range",
            CssFeatureKind::Descriptor,
            "range in @counter-style",
            "#counter-style-range",
        ),
        (
            "official.descriptor.counter-style.pad",
            CssFeatureKind::Descriptor,
            "pad in @counter-style",
            "#counter-style-pad",
        ),
        (
            "official.descriptor.counter-style.fallback",
            CssFeatureKind::Descriptor,
            "fallback in @counter-style",
            "#counter-style-fallback",
        ),
        (
            "official.descriptor.counter-style.symbols",
            CssFeatureKind::Descriptor,
            "symbols in @counter-style",
            "#counter-style-symbols",
        ),
        (
            "official.descriptor.counter-style.additive-symbols",
            CssFeatureKind::Descriptor,
            "additive-symbols in @counter-style",
            "#counter-style-symbols",
        ),
        (
            "official.descriptor.counter-style.speak-as",
            CssFeatureKind::Descriptor,
            "speak-as in @counter-style",
            "#counter-style-speak-as",
        ),
        (
            "official.value.counter-style",
            CssFeatureKind::Value,
            "<counter-style>",
            "#the-counter-style-rule",
        ),
        (
            "official.value.counter-style-name",
            CssFeatureKind::Value,
            "<counter-style-name>",
            "#the-counter-style-rule",
        ),
        (
            "official.value.symbol",
            CssFeatureKind::Value,
            "<symbol>",
            "#counter-style-symbols",
        ),
        (
            "official.value.symbols-function",
            CssFeatureKind::Value,
            "symbols()",
            "#symbols-function",
        ),
        (
            "official.value.symbols-type",
            CssFeatureKind::Value,
            "cyclic|numeric|alphabetic|symbolic|fixed",
            "#symbols-function",
        ),
    ];

    let page = [
        (
            "later.rule.page",
            CssFeatureKind::Rule,
            "@page",
            "page.html#page-box",
        ),
        (
            "official.selector.page-pseudo",
            CssFeatureKind::Selector,
            ":left|:right|:first",
            "page.html#page-selectors",
        ),
    ];

    for (id, kind, spelling, production) in counter_styles {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(
            metadata.source().id().as_str(),
            "O-COUNTERSTYLES3",
            "{id} source"
        );
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(
            metadata.recognized_unsupported_code(),
            None,
            "{id} recognized code"
        );
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
    }

    for (id, kind, spelling, production) in page {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), "O-CSS2", "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(
            metadata.recognized_unsupported_code(),
            None,
            "{id} recognized code"
        );
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
    }
}

#[test]
fn c12_property_metadata_is_truthful() {
    let properties = [
        (
            "official.property.border-collapse",
            "border-collapse",
            "O-CSS2",
            "tables.html#propdef-border-collapse",
            "collapse",
        ),
        (
            "official.property.border-spacing",
            "border-spacing",
            "O-CSS2",
            "tables.html#propdef-border-spacing",
            "2px 3px",
        ),
        (
            "official.property.caption-side",
            "caption-side",
            "O-CSS2",
            "tables.html#propdef-caption-side",
            "bottom",
        ),
        (
            "official.property.clip",
            "clip",
            "O-CSS2",
            "visufx.html#propdef-clip",
            "rect(auto, 10px, 20px, -1px)",
        ),
        (
            "official.property.empty-cells",
            "empty-cells",
            "O-CSS2",
            "tables.html#propdef-empty-cells",
            "hide",
        ),
        (
            "official.property.orphans",
            "orphans",
            "O-CSS2",
            "page.html#propdef-orphans",
            "3",
        ),
        (
            "official.property.page-break-after",
            "page-break-after",
            "O-CSS2",
            "page.html#propdef-page-break-after",
            "right",
        ),
        (
            "official.property.page-break-before",
            "page-break-before",
            "O-CSS2",
            "page.html#propdef-page-break-before",
            "always",
        ),
        (
            "official.property.page-break-inside",
            "page-break-inside",
            "O-CSS2",
            "page.html#propdef-page-break-inside",
            "avoid",
        ),
        (
            "official.property.quotes",
            "quotes",
            "O-CSS2",
            "generate.html#propdef-quotes",
            "\"open\" \"close\"",
        ),
        (
            "official.property.table-layout",
            "table-layout",
            "O-CSS2",
            "tables.html#propdef-table-layout",
            "fixed",
        ),
        (
            "official.property.widows",
            "widows",
            "O-CSS2",
            "page.html#propdef-widows",
            "4",
        ),
        (
            "official.property.word-spacing",
            "word-spacing",
            "O-CSS2",
            "text.html#propdef-word-spacing",
            "-0.25em",
        ),
        (
            "official.property.text-combine-upright",
            "text-combine-upright",
            "O-WRITING3",
            "#propdef-text-combine-upright",
            "all",
        ),
        (
            "official.property.text-orientation",
            "text-orientation",
            "O-WRITING3",
            "#propdef-text-orientation",
            "sideways",
        ),
        (
            "official.property.unicode-bidi",
            "unicode-bidi",
            "O-WRITING3",
            "#propdef-unicode-bidi",
            "isolate-override",
        ),
        (
            "official.property.caret-color",
            "caret-color",
            "O-UI3",
            "#propdef-caret-color",
            "rebeccapurple",
        ),
        (
            "official.property.outline-offset",
            "outline-offset",
            "O-UI3",
            "#propdef-outline-offset",
            "-2px",
        ),
        (
            "official.property.resize",
            "resize",
            "O-UI3",
            "#propdef-resize",
            "horizontal",
        ),
        (
            "official.property.contain",
            "contain",
            "O-CONTAIN1",
            "#propdef-contain",
            "paint size",
        ),
        (
            "official.property.transform-box",
            "transform-box",
            "O-TRANSFORMS1",
            "#propdef-transform-box",
            "view-box",
        ),
        (
            "official.property.background-blend-mode",
            "background-blend-mode",
            "O-COMPOSITING1",
            "#propdef-background-blend-mode",
            "multiply, luminosity",
        ),
        (
            "official.property.isolation",
            "isolation",
            "O-COMPOSITING1",
            "#propdef-isolation",
            "isolate",
        ),
        (
            "official.property.mix-blend-mode",
            "mix-blend-mode",
            "O-COMPOSITING1",
            "#propdef-mix-blend-mode",
            "soft-light",
        ),
    ];

    for (id, name, source, production, authored_value) in properties {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), CssFeatureKind::Property, "{id} kind");
        assert_eq!(metadata.spelling(), name, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");

        let property = property_metadata(name).unwrap_or_else(|| panic!("missing property {name}"));
        assert_eq!(property.feature(), metadata, "{id} property metadata");
        assert_eq!(property.canonical_name(), name, "{id} canonical name");

        let source = format!("{name}: {authored_value}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        let [declaration] = report.syntax().as_slice() else {
            panic!("{id}: expected one retained declaration");
        };
        assert_eq!(
            declaration
                .known()
                .expect("known C12 property")
                .property()
                .stable_id(),
            id,
            "{id} parser vector",
        );
    }

    let alias = feature_metadata("official.property-alias.glyph-orientation-vertical")
        .expect("legacy glyph-orientation-vertical metadata");
    assert_eq!(alias.kind(), CssFeatureKind::PropertyAlias);
    assert_eq!(alias.spelling(), "glyph-orientation-vertical");
    assert_eq!(alias.source().id().as_str(), "O-WRITING3");
    assert_eq!(alias.production(), "#propdef-glyph-orientation-vertical");
    assert_eq!(alias.status(), CssSupportStatus::Complete);
    assert_eq!(alias.supported_subset(), None);
    assert_eq!(alias.unsupported_remainder(), None);
    assert_eq!(alias.recognized_unsupported_code(), None);
    assert!(alias.baseline_alias_targets().is_empty());
    let alias_report = parse_style_attribute("glyph-orientation-vertical: 90");
    assert!(alias_report.is_clean(), "{:?}", alias_report.diagnostics());
    assert_eq!(
        alias_report.syntax()[0]
            .known()
            .expect("known legacy alias")
            .property(),
        CssKnownProperty::TextOrientation,
    );
    assert!(CssKnownProperty::TextOrientation.aliases().is_empty());
    assert_eq!(
        CssKnownProperty::from_name("glyph-orientation-vertical"),
        None
    );

    for (id, spelling, source, production, vector) in [
        (
            "official.value.box-edge-keywords",
            "content-box|padding-box|border-box|margin-box|fill-box|stroke-box|view-box",
            "O-BOX3",
            "#keywords",
            "transform-box: view-box",
        ),
        (
            "official.value.blend-mode",
            "normal|multiply|screen|overlay|darken|lighten|color-dodge|color-burn|hard-light|soft-light|difference|exclusion|hue|saturation|color|luminosity",
            "O-COMPOSITING1",
            "#blending,#blendingseparable,#blendingnonseparable",
            "background-blend-mode: multiply, luminosity",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing metadata for {id}"));
        assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
        let report = parse_style_attribute(vector);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{id} parser vector");
    }

    for keyword in [
        "content-box",
        "padding-box",
        "border-box",
        "margin-box",
        "fill-box",
        "stroke-box",
        "view-box",
    ] {
        assert!(
            CssBoxEdgeKeyword::from_keyword(keyword).is_some(),
            "{keyword}"
        );
    }
    assert_eq!(CssBoxEdgeKeyword::from_keyword("content"), None);

    for keyword in [
        "normal",
        "multiply",
        "screen",
        "overlay",
        "darken",
        "lighten",
        "color-dodge",
        "color-burn",
        "hard-light",
        "soft-light",
        "difference",
        "exclusion",
        "hue",
        "saturation",
        "color",
        "luminosity",
    ] {
        assert!(CssBlendMode::from_keyword(keyword).is_some(), "{keyword}");
    }
    assert_eq!(CssBlendMode::from_keyword("plus-lighter"), None);
}

#[test]
fn named_conformance_records_expose_declared_metadata() {
    for expected in EXPECTED {
        let actual = feature_metadata(expected.id).expect("expected catalog record");
        assert_eq!(actual.id().as_str(), expected.id);
        assert_eq!(actual.kind(), expected.kind, "{} kind", expected.id);
        assert_eq!(
            actual.spelling(),
            expected.spelling,
            "{} spelling",
            expected.id
        );
        assert_eq!(
            actual.production(),
            expected.production,
            "{} production",
            expected.id
        );
        assert_eq!(actual.status(), expected.status, "{} status", expected.id);
        assert_eq!(
            actual.supported_subset(),
            expected.supported_subset,
            "{} supported subset",
            expected.id
        );
        assert_eq!(
            actual.unsupported_remainder(),
            expected.unsupported_remainder,
            "{} unsupported remainder",
            expected.id
        );
        assert_eq!(
            actual.recognized_unsupported_code(),
            expected.recognized_code,
            "{} recognized code",
            expected.id
        );
        let ExpectedSource::Id(source_id) = expected.source;
        assert_eq!(
            actual.source().id().as_str(),
            source_id,
            "{} source",
            expected.id
        );

        let partial = expected.status == CssSupportStatus::Partial;
        assert_eq!(
            actual.supported_subset().is_some(),
            partial,
            "{} subset status invariant",
            expected.id
        );
        assert_eq!(
            actual.unsupported_remainder().is_some(),
            partial,
            "{} remainder status invariant",
            expected.id
        );
        assert_eq!(
            actual.recognized_unsupported_code().is_some(),
            expected.status == CssSupportStatus::RecognizedUnsupported,
            "{} recognized status invariant",
            expected.id
        );
    }

    assert!(
        feature_metadata("BASELINE.RULE.IMPORT").is_none(),
        "lookup is exact"
    );
    assert!(
        feature_metadata("baseline.rule.import ").is_none(),
        "lookup does not trim"
    );
    assert!(feature_metadata("baseline.property.display").is_some());
    assert!(feature_metadata("").is_none());
}

#[test]
fn conformance_catalog_vectors_cover_each_supported_and_unsupported_boundary() {
    for expected in EXPECTED {
        match expected.status {
            CssSupportStatus::Complete | CssSupportStatus::Partial => {
                let input = expected
                    .positive
                    .expect("supported record needs a positive vector");
                let diagnostics = diagnostics(input);
                assert!(
                    diagnostics.is_empty(),
                    "{} positive vector produced {diagnostics:?}",
                    expected.id
                );
            }
            CssSupportStatus::RecognizedUnsupported => {
                assert!(
                    expected.positive.is_none(),
                    "recognized unsupported has no positive vector"
                );
            }
        }

        match expected.status {
            CssSupportStatus::Complete => assert!(expected.negative.is_none()),
            CssSupportStatus::Partial | CssSupportStatus::RecognizedUnsupported => {
                let (input, code) = expected
                    .negative
                    .expect("unsupported boundary needs a vector");
                let diagnostics = diagnostics(input);
                assert!(
                    !diagnostics.is_empty(),
                    "{} negative vector was accepted",
                    expected.id
                );
                assert_eq!(
                    diagnostics[0].0, code,
                    "{} negative vector root",
                    expected.id
                );
                if expected.status == CssSupportStatus::RecognizedUnsupported {
                    assert_eq!(
                        diagnostics[0].1,
                        Some(expected.id),
                        "{} diagnostic feature identity",
                        expected.id
                    );
                }
            }
        }
    }
}

#[test]
fn source_registry_lookups_are_exact_and_preserve_provenance_xor() {
    let filter = specification_source("X-FILTER2-BASE").expect("filter baseline source");
    assert_eq!(filter.module(), "Filter Effects");
    assert_eq!(filter.level(), "2 baseline subset");
    assert_eq!(filter.tier(), CssSpecificationTier::SurgeistExtension);
    assert_eq!(filter.url(), None);
    assert_eq!(
        filter.repository_provenance(),
        Some("bc5394f:src/parser/effects.rs")
    );

    for source in specification_sources() {
        assert_ne!(
            source.url().is_some(),
            source.repository_provenance().is_some()
        );
    }
    assert!(specification_source("o-color4").is_none());
    assert!(specification_source(" O-COLOR4").is_none());
    assert!(specification_source("O-COLOR4 ").is_none());
    assert!(specification_source("").is_none());
}

#[test]
fn source_registry_exposes_scrollbars_level_one_reliable_provenance() {
    let source = specification_source("R-SCROLLBARS1").expect("Scrollbars 1 source");
    assert_eq!(source.id().as_str(), "R-SCROLLBARS1");
    assert_eq!(source.module(), "CSS Scrollbars Styling");
    assert_eq!(source.level(), "1");
    assert_eq!(source.tier(), CssSpecificationTier::Snapshot2026Reliable);
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/")
    );
    assert_eq!(source.repository_provenance(), None);

    assert!(specification_source("r-scrollbars1").is_none());
    assert!(specification_source("R-SCROLLBARS1 ").is_none());
}

#[test]
fn source_registry_exposes_containment_level_three_extension_provenance() {
    let source = specification_source("X-CONTAIN3").expect("Containment 3 source");
    assert_eq!(source.id().as_str(), "X-CONTAIN3");
    assert_eq!(source.module(), "CSS Containment");
    assert_eq!(source.level(), "3");
    assert_eq!(source.tier(), CssSpecificationTier::SurgeistExtension);
    assert_eq!(
        source.url(),
        Some("https://www.w3.org/TR/2022/WD-css-contain-3-20220818/")
    );
    assert_eq!(source.repository_provenance(), None);

    assert!(specification_source("x-contain3").is_none());
    assert!(specification_source(" X-CONTAIN3").is_none());
}

#[test]
fn preserved_i01_catalog_exposes_dated_atomic_provenance_and_alias_targets() {
    let scrollbar =
        feature_metadata("baseline.property.scrollbar-width").expect("scrollbar-width record");
    assert_eq!(scrollbar.source().id().as_str(), "R-SCROLLBARS1");
    assert_eq!(
        scrollbar.source().tier(),
        CssSpecificationTier::Snapshot2026Reliable
    );

    for id in [
        "baseline.rule.container",
        "baseline.container.condition",
        "baseline.container.size-feature",
    ] {
        let feature = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}`"));
        assert_eq!(feature.source().id().as_str(), "X-CONTAIN3", "{id}");
        assert_eq!(
            feature.source().tier(),
            CssSpecificationTier::SurgeistExtension
        );
    }

    let alias = feature_metadata("baseline.selector.pseudo-element")
        .expect("preserved pseudo-element alias");
    let targets: Vec<_> = alias
        .baseline_alias_targets()
        .iter()
        .map(|id| id.as_str())
        .collect();
    assert_eq!(
        targets,
        [
            "official.selector.generated",
            "ext.pseudo-element.marker",
            "ext.pseudo-element.selection",
            "ext.pseudo-element.backdrop",
            "ext.pseudo-element.generated-marker",
        ]
    );

    let generated =
        feature_metadata("official.selector.generated").expect("atomic generated pseudo-element");
    assert_eq!(generated.source().id().as_str(), "O-SELECTORS3");
    assert!(generated.baseline_alias_targets().is_empty());

    let report = parse_sheet(".note::before { color: red; }");
    assert!(
        report.is_clean(),
        "atomic parser case: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().rules().len(), 1);
}

#[test]
fn every_alias_atomic_target_has_declared_metadata_and_public_parser_evidence() {
    let aliases: &[(&str, &[&str])] = &[
        (
            "baseline.selector.pseudo-element",
            &[
                "official.selector.generated",
                "ext.pseudo-element.marker",
                "ext.pseudo-element.selection",
                "ext.pseudo-element.backdrop",
                "ext.pseudo-element.generated-marker",
            ],
        ),
        (
            "baseline.media.query-list",
            &[
                "official.media.query-list-core",
                "ext.media.condition-syntax",
                "ext.media.malformed-member-never",
            ],
        ),
        (
            "baseline.media.range-feature",
            &[
                "official.media.feature.width",
                "official.media.feature.height",
                "official.media.feature.resolution",
                "official.media.feature.color",
                "official.media.feature.monochrome",
                "ext.media.resolution.dppx",
                "ext.media.range.width",
                "ext.media.range.height",
                "ext.media.range.resolution",
                "ext.media.range.color",
                "ext.media.range.monochrome",
            ],
        ),
        (
            "baseline.media.discrete-feature",
            &[
                "official.media.feature.orientation",
                "ext.media.hover",
                "ext.media.any-hover",
                "ext.media.pointer",
                "ext.media.any-pointer",
                "ext.media.prefers-color-scheme",
                "ext.media.prefers-reduced-motion",
                "ext.media.prefers-reduced-transparency",
                "ext.media.prefers-contrast",
                "ext.media.forced-colors",
                "ext.media.display-mode",
            ],
        ),
    ];
    for (alias_id, expected_targets) in aliases {
        let alias = feature_metadata(alias_id).unwrap_or_else(|| panic!("missing `{alias_id}`"));
        let actual_targets: Vec<_> = alias
            .baseline_alias_targets()
            .iter()
            .map(|target| target.as_str())
            .collect();
        assert_eq!(actual_targets, *expected_targets, "{alias_id}");
    }

    let clean_cases = [
        (
            "official.selector.generated",
            "O-SELECTORS3",
            ".x::after { color: red; }",
        ),
        (
            "ext.pseudo-element.marker",
            "X-PSEUDO4",
            "li::marker { color: red; }",
        ),
        (
            "ext.pseudo-element.selection",
            "X-PSEUDO4",
            "::selection { color: red; }",
        ),
        (
            "ext.pseudo-element.backdrop",
            "X-PSEUDO4",
            ".dialog::backdrop { color: red; }",
        ),
        (
            "ext.pseudo-element.generated-marker",
            "X-PSEUDO4",
            ".item::before::marker { color: red; }",
        ),
        (
            "official.media.query-list-core",
            "O-MEDIA3",
            "@media screen, print { .x { color: red; } }",
        ),
        (
            "ext.media.condition-syntax",
            "R-MEDIA4",
            "@media not screen and (width: 1px) { .x { color: red; } }",
        ),
        (
            "official.media.feature.width",
            "O-MEDIA3",
            "@media (min-width: 1px) { .x { color: red; } }",
        ),
        (
            "official.media.feature.height",
            "O-MEDIA3",
            "@media (max-height: 2px) { .x { color: red; } }",
        ),
        (
            "official.media.feature.resolution",
            "O-MEDIA3",
            "@media (resolution: 192dpi) { .x { color: red; } }",
        ),
        (
            "ext.media.resolution.dppx",
            "R-MEDIA4",
            "@media (resolution: 2dppx) { .x { color: red; } }",
        ),
        (
            "official.media.feature.color",
            "O-MEDIA3",
            "@media (color: 8) { .x { color: red; } }",
        ),
        (
            "official.media.feature.monochrome",
            "O-MEDIA3",
            "@media (monochrome: 1) { .x { color: red; } }",
        ),
        (
            "ext.media.range.width",
            "R-MEDIA4",
            "@media (width >= 1px) { .x { color: red; } }",
        ),
        (
            "ext.media.range.height",
            "R-MEDIA4",
            "@media (height < 2px) { .x { color: red; } }",
        ),
        (
            "ext.media.range.resolution",
            "R-MEDIA4",
            "@media (resolution > 1dppx) { .x { color: red; } }",
        ),
        (
            "ext.media.range.color",
            "R-MEDIA4",
            "@media (color >= 8) { .x { color: red; } }",
        ),
        (
            "ext.media.range.monochrome",
            "R-MEDIA4",
            "@media (monochrome = 1) { .x { color: red; } }",
        ),
        (
            "official.media.feature.orientation",
            "O-MEDIA3",
            "@media (orientation: portrait) { .x { color: red; } }",
        ),
        (
            "ext.media.hover",
            "R-MEDIA4",
            "@media (hover: hover) { .x { color: red; } }",
        ),
        (
            "ext.media.any-hover",
            "R-MEDIA4",
            "@media (any-hover: none) { .x { color: red; } }",
        ),
        (
            "ext.media.pointer",
            "R-MEDIA4",
            "@media (pointer: fine) { .x { color: red; } }",
        ),
        (
            "ext.media.any-pointer",
            "R-MEDIA4",
            "@media (any-pointer: coarse) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-color-scheme",
            "X-MEDIA5",
            "@media (prefers-color-scheme: dark) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-reduced-motion",
            "X-MEDIA5",
            "@media (prefers-reduced-motion: reduce) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-reduced-transparency",
            "X-MEDIA5",
            "@media (prefers-reduced-transparency: reduce) { .x { color: red; } }",
        ),
        (
            "ext.media.prefers-contrast",
            "X-MEDIA5",
            "@media (prefers-contrast: more) { .x { color: red; } }",
        ),
        (
            "ext.media.forced-colors",
            "X-MEDIA5",
            "@media (forced-colors: active) { .x { color: red; } }",
        ),
        (
            "ext.media.display-mode",
            "X-DISPLAY-MODE-BASE",
            "@media (display-mode: picture-in-picture) { .x { color: red; } }",
        ),
    ];
    for (id, source_id, css) in clean_cases {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}`"));
        assert_eq!(metadata.source().id().as_str(), source_id, "{id}");
        assert!(metadata.baseline_alias_targets().is_empty(), "{id}");
        let report = parse_sheet(css);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 1, "{id}");
    }

    let malformed = feature_metadata("ext.media.malformed-member-never")
        .expect("malformed-member recovery target");
    assert_eq!(malformed.source().id().as_str(), "R-MEDIA4");
    assert!(malformed.baseline_alias_targets().is_empty());
    let source = "@media screen, ??? { .x { color: red; } }";
    let report = parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one malformed-member diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("???").expect("responsible malformed member")
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find(" ???").expect("malformed recovery unit")
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find('{').expect("end of media query list")
    );
}

fn assert_complete_function_metadata(id: &str, spelling: &str, source: &str, production: &str) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
    assert_eq!(metadata.id().as_str(), id, "{id} identity");
    assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id} kind");
    assert_eq!(metadata.spelling(), spelling, "{id} spelling");
    assert_eq!(metadata.source().id().as_str(), source, "{id} source");
    assert_eq!(metadata.production(), production, "{id} production");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
    assert_eq!(metadata.supported_subset(), None, "{id} subset");
    assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
}

fn assert_partial_function_metadata(
    id: &str,
    spelling: &str,
    source: &str,
    production: &str,
    subset: &str,
    remainder: &str,
) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
    assert_eq!(metadata.id().as_str(), id, "{id} identity");
    assert_eq!(metadata.kind(), CssFeatureKind::Value, "{id} kind");
    assert_eq!(metadata.spelling(), spelling, "{id} spelling");
    assert_eq!(metadata.source().id().as_str(), source, "{id} source");
    assert_eq!(metadata.production(), production, "{id} production");
    assert_eq!(metadata.status(), CssSupportStatus::Partial, "{id} status");
    assert_eq!(metadata.supported_subset(), Some(subset), "{id} subset");
    assert_eq!(
        metadata.unsupported_remainder(),
        Some(remainder),
        "{id} remainder"
    );
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
}

fn assert_complete_function_property_metadata(
    id: &str,
    canonical_name: &str,
    source: &str,
    production: &str,
) {
    let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing `{id}` metadata"));
    let property = property_metadata(canonical_name)
        .unwrap_or_else(|| panic!("missing `{canonical_name}` property metadata"));
    assert!(std::ptr::eq(metadata, property.feature()), "{id} owner");
    assert_eq!(metadata.kind(), CssFeatureKind::Property, "{id} kind");
    assert_eq!(metadata.spelling(), canonical_name, "{id} spelling");
    assert_eq!(metadata.source().id().as_str(), source, "{id} source");
    assert_eq!(metadata.production(), production, "{id} production");
    assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
    assert_eq!(metadata.supported_subset(), None, "{id} subset");
    assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
    assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
    assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");
}

#[test]
fn official_two_dimensional_transform_metadata_matches_typed_functions() {
    let report = parse_style_attribute(concat!(
        "transform: matrix(1, 0, 0, 1, 10, 20) translate(1px, 2%) ",
        "translateX(3px) translateY(4%) scale(1.5, 2) scaleX(.5) scaleY(2) ",
        "rotate(45deg) skew(10deg, 20deg) skewX(5deg) skewY(6deg)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .expect("known transform")
        .property_value()
        .expect("ordinary transform")
    else {
        panic!("expected transform value");
    };
    let CssTransformValue::Functions(functions) = transform.current() else {
        panic!("expected typed transform functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssTransformFunctionValue::Matrix(_),
            CssTransformFunctionValue::Translate(_),
            CssTransformFunctionValue::TranslateX(_),
            CssTransformFunctionValue::TranslateY(_),
            CssTransformFunctionValue::Scale(_),
            CssTransformFunctionValue::ScaleX(_),
            CssTransformFunctionValue::ScaleY(_),
            CssTransformFunctionValue::Rotate(_),
            CssTransformFunctionValue::Skew(_),
            CssTransformFunctionValue::SkewX(_),
            CssTransformFunctionValue::SkewY(_),
        ]
    ));

    assert_complete_function_metadata(
        "official.value.transform-list",
        "<transform-list>",
        "O-TRANSFORMS1",
        "#transform-function-lists",
    );
    assert_complete_function_metadata(
        "official.value.transform-function",
        "<transform-function>",
        "O-TRANSFORMS1",
        "#transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.matrix",
        "matrix()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.translate",
        "translate()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.translate-x",
        "translateX()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.translate-y",
        "translateY()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.scale",
        "scale()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.scale-x",
        "scaleX()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.scale-y",
        "scaleY()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.rotate",
        "rotate()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.skew",
        "skew()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.skew-x",
        "skewX()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
    assert_complete_function_metadata(
        "official.value.transform.skew-y",
        "skewY()",
        "O-TRANSFORMS1",
        "#two-d-transform-functions",
    );
}

#[test]
fn transform_matrix3d_exposes_sixteen_finite_components() {
    let report = parse_style_attribute(
        "transform: matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransformValue::Functions(functions) = transform.current() else {
        panic!("expected transform functions");
    };
    let CssTransformFunctionValue::Matrix3d(matrix) = &functions.functions()[0] else {
        panic!("expected matrix3d");
    };
    assert_eq!(matrix.components().len(), 16);
    assert_complete_function_metadata(
        "ext.value.transform.matrix3d",
        "matrix3d()",
        "I-TRANSFORMS2",
        "#funcdef-matrix3d",
    );
}

#[test]
fn transform_perspective_accepts_none_and_zero_and_rejects_invalid_dimensions() {
    let report = parse_style_attribute("transform: perspective(none) perspective(0)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransformValue::Functions(functions) = transform.current() else {
        panic!("expected transform functions");
    };
    assert!(matches!(
        functions.functions()[0],
        CssTransformFunctionValue::Perspective(CssTransformPerspective::None)
    ));
    assert!(matches!(
        &functions.functions()[1],
        CssTransformFunctionValue::Perspective(CssTransformPerspective::Length(length))
            if matches!(length.value(), CssLength::Zero)
    ));
    assert!(!parse_style_attribute("transform: perspective(10%)").is_clean());
    assert!(!parse_style_attribute("transform: perspective(-1px)").is_clean());
    assert_complete_function_metadata(
        "ext.value.transform.perspective",
        "perspective()",
        "I-TRANSFORMS2",
        "#funcdef-perspective",
    );
}

#[test]
fn transform_three_dimensional_rotations_are_typed() {
    let report = parse_style_attribute(
        "transform: rotate3d(1, 0, -1, 45deg) rotateX(10deg) rotateY(20deg) rotateZ(30deg)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransformValue::Functions(functions) = transform.current() else {
        panic!("expected transform functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssTransformFunctionValue::Rotate3d(_),
            CssTransformFunctionValue::RotateX(_),
            CssTransformFunctionValue::RotateY(_),
            CssTransformFunctionValue::RotateZ(_),
        ]
    ));
    assert_complete_function_metadata(
        "ext.value.transform.rotate3d",
        "rotate3d()",
        "I-TRANSFORMS2",
        "#funcdef-rotate3d",
    );
    assert_complete_function_metadata(
        "ext.value.transform.rotate-x",
        "rotateX()",
        "I-TRANSFORMS2",
        "#funcdef-rotatex",
    );
    assert_complete_function_metadata(
        "ext.value.transform.rotate-y",
        "rotateY()",
        "I-TRANSFORMS2",
        "#funcdef-rotatey",
    );
    assert_complete_function_metadata(
        "ext.value.transform.rotate-z",
        "rotateZ()",
        "I-TRANSFORMS2",
        "#funcdef-rotatez",
    );
}

#[test]
fn transform_three_dimensional_scales_preserve_number_and_percentage_operands() {
    let report = parse_style_attribute("transform: scale3d(1, 50%, 2) scaleZ(125%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransformValue::Functions(functions) = transform.current() else {
        panic!("expected transform functions");
    };
    let CssTransformFunctionValue::Scale3d(scale) = &functions.functions()[0] else {
        panic!("expected scale3d");
    };
    assert!(matches!(scale.x(), CssTransformScaleComponent::Number(_)));
    assert!(matches!(
        scale.y(),
        CssTransformScaleComponent::Percentage(_)
    ));
    assert!(matches!(
        functions.functions()[1],
        CssTransformFunctionValue::ScaleZ(CssTransformScaleComponent::Percentage(_))
    ));
    assert_complete_function_metadata(
        "ext.value.transform.scale3d",
        "scale3d()",
        "I-TRANSFORMS2",
        "#funcdef-scale3d",
    );
    assert_complete_function_metadata(
        "ext.value.transform.scale-z",
        "scaleZ()",
        "I-TRANSFORMS2",
        "#funcdef-scalez",
    );
}

#[test]
fn transform_three_dimensional_translations_keep_z_length_only() {
    let report = parse_style_attribute("transform: translate3d(10%, 2px, 4em) translateZ(3px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransformValue::Functions(functions) = transform.current() else {
        panic!("expected transform functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssTransformFunctionValue::Translate3d(_),
            CssTransformFunctionValue::TranslateZ(_),
        ]
    ));
    assert!(!parse_style_attribute("transform: translate3d(1px, 2px, 3%)").is_clean());
    assert_complete_function_metadata(
        "ext.value.transform.translate3d",
        "translate3d()",
        "I-TRANSFORMS2",
        "#funcdef-translate3d",
    );
    assert_complete_function_metadata(
        "ext.value.transform.translate-z",
        "translateZ()",
        "I-TRANSFORMS2",
        "#funcdef-translatez",
    );
}

#[test]
fn official_easing_metadata_matches_typed_functions() {
    let report = parse_style_attribute(
        "transition-timing-function: ease, cubic-bezier(.25, 0, .75, 1), steps(2, jump-none)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::TransitionTimingFunction(value) = report.syntax()[0]
        .known()
        .expect("known transition timing")
        .property_value()
        .expect("ordinary transition timing")
    else {
        panic!("expected transition timing value");
    };
    assert!(matches!(
        value.current().values(),
        [
            CssEasingValue::Keyword(_),
            CssEasingValue::CubicBezier(_),
            CssEasingValue::Steps(_),
        ]
    ));
    assert_complete_function_metadata(
        "official.value.easing-function",
        "<easing-function>",
        "O-EASING1",
        "#easing-functions",
    );
    assert_complete_function_metadata(
        "official.value.cubic-bezier-easing",
        "cubic-bezier()",
        "O-EASING1",
        "#cubic-bezier-easing-functions",
    );
    assert_complete_function_metadata(
        "official.value.step-easing",
        "steps()",
        "O-EASING1",
        "#step-easing-functions",
    );
    assert_complete_function_metadata(
        "official.value.step-position",
        "<step-position>",
        "O-EASING1",
        "#step-easing-functions",
    );
}

#[test]
fn official_shadow_metadata_matches_typed_box_shadow() {
    let report = parse_style_attribute("box-shadow: red inset -1px 2px 3px -4px");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BoxShadow(value) = report.syntax()[0]
        .known()
        .expect("known box shadow")
        .property_value()
        .expect("ordinary box shadow")
    else {
        panic!("expected box-shadow value");
    };
    let CssBoxShadow::Shadows(shadows) = value.current() else {
        panic!("expected typed shadows");
    };
    assert!(shadows.shadows()[0].inset());
    assert!(shadows.shadows()[0].color().is_some());
    assert_complete_function_metadata(
        "official.value.shadow",
        "<shadow>",
        "O-BACKGROUNDS3",
        "#box-shadow",
    );
}

#[test]
fn filter_function_list_preserves_typed_authored_order() {
    let report = parse_style_attribute(
        "filter: url(\"filters.svg#rough\") blur(2px) drop-shadow(1px 2px red) opacity(50%)",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .expect("known filter")
        .property_value()
        .expect("ordinary filter")
    else {
        panic!("expected filter value");
    };
    let CssFilterValue::Functions(functions) = value.current() else {
        panic!("expected typed filter functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssFilterFunctionValue::Url(_),
            CssFilterFunctionValue::Blur(_),
            CssFilterFunctionValue::DropShadow(_),
            CssFilterFunctionValue::Opacity(_),
        ]
    ));
    assert_complete_function_metadata(
        "ext.value.filter-function-list",
        "<filter-function-list>",
        "I-FILTER1",
        "#FilterProperty",
    );
}

#[test]
fn every_filter_amount_function_has_exact_typed_domain() {
    let report = parse_style_attribute(concat!(
        "filter: blur(2px) brightness() contrast(25%) grayscale(.5) ",
        "hue-rotate(45deg) invert(10%) opacity(.75) saturate(2) sepia(30%)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected filter value");
    };
    let CssFilterValue::Functions(functions) = value.current() else {
        panic!("expected typed filter functions");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssFilterFunctionValue::Blur(_),
            CssFilterFunctionValue::Brightness(_),
            CssFilterFunctionValue::Contrast(_),
            CssFilterFunctionValue::Grayscale(_),
            CssFilterFunctionValue::HueRotate(_),
            CssFilterFunctionValue::Invert(_),
            CssFilterFunctionValue::Opacity(_),
            CssFilterFunctionValue::Saturate(_),
            CssFilterFunctionValue::Sepia(_),
        ]
    ));
    assert_complete_function_metadata(
        "ext.value.filter.blur",
        "blur()",
        "I-FILTER1",
        "#funcdef-filter-blur",
    );
    assert_complete_function_metadata(
        "ext.value.filter.brightness",
        "brightness()",
        "I-FILTER1",
        "#funcdef-filter-brightness",
    );
    assert_complete_function_metadata(
        "ext.value.filter.contrast",
        "contrast()",
        "I-FILTER1",
        "#funcdef-filter-contrast",
    );
    assert_complete_function_metadata(
        "ext.value.filter.grayscale",
        "grayscale()",
        "I-FILTER1",
        "#funcdef-filter-grayscale",
    );
    assert_complete_function_metadata(
        "ext.value.filter.hue-rotate",
        "hue-rotate()",
        "I-FILTER1",
        "#funcdef-filter-hue-rotate",
    );
    assert_complete_function_metadata(
        "ext.value.filter.invert",
        "invert()",
        "I-FILTER1",
        "#funcdef-filter-invert",
    );
    assert_complete_function_metadata(
        "ext.value.filter.opacity",
        "opacity()",
        "I-FILTER1",
        "#funcdef-filter-opacity",
    );
    assert_complete_function_metadata(
        "ext.value.filter.saturate",
        "saturate()",
        "I-FILTER1",
        "#funcdef-filter-saturate",
    );
    assert_complete_function_metadata(
        "ext.value.filter.sepia",
        "sepia()",
        "I-FILTER1",
        "#funcdef-filter-sepia",
    );
}

#[test]
fn drop_shadow_rejects_box_shadow_only_components() {
    let report = parse_style_attribute("filter: drop-shadow(red 1px 2px 3px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected filter value");
    };
    let CssFilterValue::Functions(functions) = value.current() else {
        panic!("expected typed filter functions");
    };
    assert!(matches!(
        functions.functions(),
        [CssFilterFunctionValue::DropShadow(_)]
    ));
    assert!(!parse_style_attribute("filter: drop-shadow(inset 1px 2px)").is_clean());
    assert!(!parse_style_attribute("filter: drop-shadow(1px 2px 3px 4px)").is_clean());
    assert_complete_function_metadata(
        "ext.value.filter.drop-shadow",
        "drop-shadow()",
        "I-FILTER1",
        "#funcdef-filter-drop-shadow",
    );
}

#[test]
fn clip_path_distinguishes_selected_and_deferred_shape_functions() {
    for value in [
        "inset(1px)",
        "circle(10px)",
        "ellipse(10px 20%)",
        "polygon(, 0 0, 100% 0)",
    ] {
        let report = parse_style_attribute(&format!("clip-path: {value}"));
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    }
    for value in [
        "path('M 0 0 L 1 1')",
        "shape(from 0 0, line to 1px 1px)",
        "rect(0 1px 1px 0)",
        "xywh(0 0 1px 1px)",
    ] {
        assert!(
            !parse_style_attribute(&format!("clip-path: {value}")).is_clean(),
            "deferred {value}"
        );
    }
    assert_partial_function_metadata(
        "ext.value.basic-shape",
        "<basic-shape>",
        "S-SHAPES1",
        "#typedef-basic-shape",
        BASIC_SHAPE_SUBSET,
        BASIC_SHAPE_REMAINDER,
    );
}

#[test]
fn every_selected_basic_shape_has_typed_public_components() {
    let report = parse_style_attribute(concat!(
        "clip-path: inset(1px round 2px); ",
        "clip-path: circle(10px at center); ",
        "clip-path: ellipse(10px 20% at left top); ",
        "clip-path: polygon(evenodd round 2px, 0 0, 100% 0)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let expected = ["inset", "circle", "ellipse", "polygon"];
    for (declaration, expected) in report.syntax().iter().zip(expected) {
        let CssKnownPropertyValueRef::ClipPath(value) = declaration
            .known()
            .expect("known clip path")
            .property_value()
            .expect("ordinary clip path")
        else {
            panic!("expected clip-path value");
        };
        let Some(CssClipPathValue::BasicShape(shape)) = value.current() else {
            panic!("expected current {expected} shape");
        };
        assert!(
            matches!(
                (expected, shape),
                ("inset", CssBasicShapeValue::Inset(_))
                    | ("circle", CssBasicShapeValue::Circle(_))
                    | ("ellipse", CssBasicShapeValue::Ellipse(_))
                    | ("polygon", CssBasicShapeValue::Polygon(_))
            ),
            "expected {expected}"
        );
        if let CssBasicShapeValue::Polygon(polygon) = shape {
            assert!(polygon.round().is_some(), "polygon round component");
        }
    }
    assert_complete_function_metadata(
        "ext.value.basic-shape.inset",
        "inset()",
        "S-SHAPES1",
        "#funcdef-basic-shape-inset",
    );
    assert_complete_function_metadata(
        "ext.value.basic-shape.circle",
        "circle()",
        "S-SHAPES1",
        "#funcdef-basic-shape-circle",
    );
    assert_complete_function_metadata(
        "ext.value.basic-shape.ellipse",
        "ellipse()",
        "S-SHAPES1",
        "#funcdef-basic-shape-ellipse",
    );
    assert_complete_function_metadata(
        "ext.value.basic-shape.polygon",
        "polygon()",
        "S-SHAPES1",
        "#funcdef-basic-shape-polygon",
    );
}

#[test]
fn backdrop_filter_preserves_exact_typed_baseline_subset() {
    let report = parse_style_attribute("backdrop-filter: blur(2px) opacity(50%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackdropFilter(value) = report.syntax()[0]
        .known()
        .expect("known backdrop filter")
        .property_value()
        .expect("ordinary backdrop filter")
    else {
        panic!("expected backdrop-filter value");
    };
    assert!(matches!(
        value.current(),
        CssFilterValue::Functions(functions)
            if matches!(
                functions.functions(),
                [CssFilterFunctionValue::Blur(_), CssFilterFunctionValue::Opacity(_)]
            )
    ));
    let metadata =
        feature_metadata("baseline.property.backdrop-filter").expect("backdrop-filter metadata");
    assert_eq!(metadata.kind(), CssFeatureKind::Property);
    assert_eq!(metadata.spelling(), "backdrop-filter");
    assert_eq!(metadata.source().id().as_str(), "X-FILTER2-BASE");
    assert_eq!(metadata.production(), "#propdef-backdrop-filter");
    assert_eq!(metadata.status(), CssSupportStatus::Partial);
    assert_eq!(metadata.supported_subset(), Some(BACKDROP_FILTER_SUBSET));
    assert_eq!(
        metadata.unsupported_remainder(),
        Some(BACKDROP_FILTER_REMAINDER)
    );
    assert!(metadata.baseline_alias_targets().is_empty());
}

#[test]
fn clip_path_selected_subset_and_remainder_are_distinct() {
    for value in [
        "none",
        "url(\"shapes.svg#clip\")",
        "inset(1px)",
        "circle(10px)",
        "ellipse(10px 20%)",
        "polygon(round 2px, 0 0, 100% 0)",
    ] {
        let report = parse_style_attribute(&format!("clip-path: {value}"));
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    }
    for value in [
        "border-box circle(10px)",
        "path('M 0 0 L 1 1')",
        "shape(from 0 0, line to 1px 1px)",
        "rect(0 1px 1px 0)",
        "xywh(0 0 1px 1px)",
    ] {
        assert!(
            !parse_style_attribute(&format!("clip-path: {value}")).is_clean(),
            "unsupported {value}"
        );
    }
    let metadata = feature_metadata("baseline.property.clip-path").expect("clip-path metadata");
    assert_eq!(metadata.kind(), CssFeatureKind::Property);
    assert_eq!(metadata.spelling(), "clip-path");
    assert_eq!(metadata.source().id().as_str(), "S-MASKING1");
    assert_eq!(metadata.production(), "#propdef-clip-path");
    assert_eq!(metadata.status(), CssSupportStatus::Partial);
    assert_eq!(metadata.supported_subset(), Some(CLIP_PATH_SUBSET));
    assert_eq!(metadata.unsupported_remainder(), Some(CLIP_PATH_REMAINDER));
    assert!(metadata.baseline_alias_targets().is_empty());
}

#[test]
fn completed_function_property_metadata_matches_public_current_accessors() {
    let report = parse_style_attribute(concat!(
        "transform: translate(1px, 2%); ",
        "box-shadow: 1px 2px 3px red; ",
        "filter: blur(2px); ",
        "transition-timing-function: cubic-bezier(.25, 0, .75, 1); ",
        "animation-timing-function: steps(2, jump-none)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::Transform(value)
            if matches!(value.current(), CssTransformValue::Functions(_))
    ));
    assert!(matches!(
        report.syntax()[1]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::BoxShadow(value)
            if matches!(value.current(), CssBoxShadow::Shadows(_))
    ));
    assert!(matches!(
        report.syntax()[2]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::Filter(value)
            if matches!(value.current(), CssFilterValue::Functions(_))
    ));
    assert!(matches!(
        report.syntax()[3]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::TransitionTimingFunction(value)
            if matches!(value.current().values(), [CssEasingValue::CubicBezier(_)])
    ));
    assert!(matches!(
        report.syntax()[4]
            .known()
            .unwrap()
            .property_value()
            .unwrap(),
        CssKnownPropertyValueRef::AnimationTimingFunction(value)
            if matches!(value.current().values(), [CssEasingValue::Steps(_)])
    ));

    assert_complete_function_property_metadata(
        "baseline.property.transform",
        "transform",
        "O-TRANSFORMS1",
        "#propdef-transform",
    );
    assert_complete_function_property_metadata(
        "baseline.property.box-shadow",
        "box-shadow",
        "O-BACKGROUNDS3",
        "#propdef-box-shadow",
    );
    assert_complete_function_property_metadata(
        "baseline.property.filter",
        "filter",
        "I-FILTER1",
        "#propdef-filter",
    );
    assert_complete_function_property_metadata(
        "baseline.property.transition-timing-function",
        "transition-timing-function",
        "I-TRANSITIONS1",
        "#propdef-transition-timing-function",
    );
    assert_complete_function_property_metadata(
        "baseline.property.animation-timing-function",
        "animation-timing-function",
        "I-ANIMATIONS1",
        "#propdef-animation-timing-function",
    );
}

#[test]
fn exclusion_registry_exposes_named_official_audit_facts() {
    let predecessor = conformance_exclusion("excluded.O-CSS2.property.margin")
        .expect("superseded CSS2 margin definition");
    assert_eq!(predecessor.source().id().as_str(), "O-CSS2");
    assert_eq!(predecessor.production(), "box.html#propdef-margin");
    assert_eq!(
        predecessor.reason(),
        CssExclusionReason::SupersededWithoutCurrentProduction
    );
    assert_eq!(
        predecessor.superseding_ids().map(|ids| ids[0].as_str()),
        Some("baseline.property.margin")
    );

    let informative = conformance_exclusion("excluded.O-CSS2.informative-property.azimuth")
        .expect("informative CSS2 Appendix A property row");
    assert_eq!(informative.production(), "aural.html#propdef-azimuth");
    assert_eq!(informative.reason(), CssExclusionReason::InformativeOnly);
    assert_eq!(informative.superseding_ids(), None);

    let processing = conformance_exclusion("excluded.O-IMAGES3.processing")
        .expect("out-of-boundary Images processing row");
    assert_eq!(
        processing.reason(),
        CssExclusionReason::OutsideAuthoredSyntaxBoundary
    );

    let audit = conformance_exclusion("excluded.O-COLOR4.informative-audit")
        .expect("Color 4 informative source audit row");
    assert_eq!(audit.source().id().as_str(), "O-COLOR4");
    assert_eq!(audit.reason(), CssExclusionReason::InformativeOnly);
    assert!(audit.superseding_ids().is_none());

    assert!(
        conformance_exclusions()
            .iter()
            .any(|row| row.id() == audit.id())
    );
    assert!(conformance_exclusion("EXCLUDED.O-COLOR4.INFORMATIVE-AUDIT").is_none());
    assert!(conformance_exclusion(" excluded.O-COLOR4.informative-audit").is_none());
    assert!(conformance_exclusion("excluded.O-COLOR4.informative-audit ").is_none());
}

fn diagnostics(input: Input) -> Vec<(CssErrorCode, Option<&'static str>)> {
    let diagnostics = match input {
        Input::Sheet(source) => parse_sheet(source).diagnostics().to_vec(),
        Input::Style(source) => parse_style_attribute(source).diagnostics().to_vec(),
    };
    diagnostics
        .iter()
        .map(|diagnostic| {
            let feature = match diagnostic.error().kind() {
                ErrorKind::UnsupportedAtRule(detail) => Some(detail.feature().as_str()),
                _ => None,
            };
            (diagnostic.error().code(), feature)
        })
        .collect()
}

#[test]
fn images_three_properties_are_publicly_recognized() {
    let object = parse_style_attribute("object-position: left top");
    assert!(object.is_clean(), "{:?}", object.diagnostics());
    assert_eq!(
        object.syntax()[0]
            .known()
            .expect("known object position")
            .property(),
        CssKnownProperty::ObjectPosition,
    );

    for (property, value) in [
        ("object-fit", "cover"),
        ("image-rendering", "pixelated"),
        ("image-orientation", "from-image"),
    ] {
        let source = format!("{property}: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained Images 3 declaration")
                .property()
                .canonical_name(),
            property,
            "{source}",
        );
        assert_eq!(
            report.syntax()[1]
                .known()
                .expect("retained color declaration")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );
    }
}

#[test]
fn c13_background_image_metadata_is_truthful() {
    let c13_records = [
        (
            "official.property.border-image",
            CssFeatureKind::Property,
            "border-image",
            "O-BACKGROUNDS3",
            "#propdef-border-image",
            "border-image: url(frame.png) 10 fill / 2 / 1 round",
        ),
        (
            "official.property.border-image-outset",
            CssFeatureKind::Property,
            "border-image-outset",
            "O-BACKGROUNDS3",
            "#propdef-border-image-outset",
            "border-image-outset: 1 2px 3 4px",
        ),
        (
            "official.property.border-image-repeat",
            CssFeatureKind::Property,
            "border-image-repeat",
            "O-BACKGROUNDS3",
            "#propdef-border-image-repeat",
            "border-image-repeat: round space",
        ),
        (
            "official.property.border-image-slice",
            CssFeatureKind::Property,
            "border-image-slice",
            "O-BACKGROUNDS3",
            "#propdef-border-image-slice",
            "border-image-slice: 10 fill",
        ),
        (
            "official.property.border-image-source",
            CssFeatureKind::Property,
            "border-image-source",
            "O-BACKGROUNDS3",
            "#propdef-border-image-source",
            "border-image-source: linear-gradient(red, blue)",
        ),
        (
            "official.property.border-image-width",
            CssFeatureKind::Property,
            "border-image-width",
            "O-BACKGROUNDS3",
            "#propdef-border-image-width",
            "border-image-width: 1 auto 25% 4px",
        ),
        (
            "official.property.image-orientation",
            CssFeatureKind::Property,
            "image-orientation",
            "O-IMAGES3",
            "#propdef-image-orientation",
            "image-orientation: 90deg flip",
        ),
        (
            "official.property.image-rendering",
            CssFeatureKind::Property,
            "image-rendering",
            "O-IMAGES3",
            "#propdef-image-rendering",
            "image-rendering: crisp-edges",
        ),
        (
            "official.property.object-fit",
            CssFeatureKind::Property,
            "object-fit",
            "O-IMAGES3",
            "#propdef-object-fit",
            "object-fit: scale-down",
        ),
        (
            "official.value.background-layer",
            CssFeatureKind::Value,
            "<bg-layer>",
            "O-BACKGROUNDS3",
            "#layering",
            "background: url(hero.png) left top / cover no-repeat fixed border-box",
        ),
        (
            "official.value.background-image",
            CssFeatureKind::Value,
            "<bg-image>",
            "O-BACKGROUNDS3",
            "#background-image",
            "background-image: url(hero.png), none",
        ),
        (
            "official.value.repeat-style",
            CssFeatureKind::Value,
            "<repeat-style>",
            "O-BACKGROUNDS3",
            "#background-repeat",
            "background-repeat: repeat-x, no-repeat round",
        ),
        (
            "official.value.background-attachment",
            CssFeatureKind::Value,
            "<attachment>",
            "O-BACKGROUNDS3",
            "#background-attachment",
            "background-attachment: fixed, local",
        ),
        (
            "official.value.background-size",
            CssFeatureKind::Value,
            "<bg-size>",
            "O-BACKGROUNDS3",
            "#background-size",
            "background-size: cover, 10px auto",
        ),
        (
            "official.value.line-style",
            CssFeatureKind::Value,
            "<line-style>",
            "O-BACKGROUNDS3",
            "#border-style",
            "border-style: none hidden dotted dashed",
        ),
        (
            "official.value.line-width",
            CssFeatureKind::Value,
            "<line-width>",
            "O-BACKGROUNDS3",
            "#border-width",
            "border-width: thin 2px medium thick",
        ),
        (
            "official.value.image",
            CssFeatureKind::Value,
            "<image>",
            "O-IMAGES3",
            "#image-values",
            "background-image: url(hero.png)",
        ),
        (
            "official.value.gradient",
            CssFeatureKind::Value,
            "<gradient>",
            "O-IMAGES3",
            "#gradients",
            "background-image: linear-gradient(red, blue)",
        ),
        (
            "official.value.linear-gradient",
            CssFeatureKind::Value,
            "linear-gradient()",
            "O-IMAGES3",
            "#linear-gradients",
            "background-image: linear-gradient(to right, red, blue)",
        ),
        (
            "official.value.radial-gradient",
            CssFeatureKind::Value,
            "radial-gradient()",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(circle, red, blue)",
        ),
        (
            "official.value.repeating-linear-gradient",
            CssFeatureKind::Value,
            "repeating-linear-gradient()",
            "O-IMAGES3",
            "#repeating-gradients",
            "background-image: repeating-linear-gradient(45deg, #000 10px, #fff 30px)",
        ),
        (
            "official.value.repeating-radial-gradient",
            CssFeatureKind::Value,
            "repeating-radial-gradient()",
            "O-IMAGES3",
            "#repeating-gradients",
            "background-image: repeating-radial-gradient(ellipse 20% 30% at center, red, blue 40px)",
        ),
        (
            "official.value.color-stop-list",
            CssFeatureKind::Value,
            "<color-stop-list>",
            "O-IMAGES3",
            "#color-stop-syntax",
            "background-image: linear-gradient(red 10%, 20%, blue 90%)",
        ),
        (
            "official.value.side-or-corner",
            CssFeatureKind::Value,
            "<side-or-corner>",
            "O-IMAGES3",
            "#linear-gradients",
            "background-image: linear-gradient(to right bottom, red, blue)",
        ),
        (
            "official.value.radial-shape",
            CssFeatureKind::Value,
            "<radial-shape>",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(circle, red, blue)",
        ),
        (
            "official.value.radial-size",
            CssFeatureKind::Value,
            "<radial-size>",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(ellipse 20% 30%, red, blue)",
        ),
        (
            "official.value.radial-extent",
            CssFeatureKind::Value,
            "<radial-extent>",
            "O-IMAGES3",
            "#radial-gradients",
            "background-image: radial-gradient(closest-side, red, blue)",
        ),
    ];
    assert_eq!(c13_records.len(), 27);

    for (id, kind, spelling, source, production, vector) in c13_records {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(metadata.id().as_str(), id, "{id} identity");
        assert_eq!(metadata.kind(), kind, "{id} kind");
        assert_eq!(metadata.spelling(), spelling, "{id} spelling");
        assert_eq!(metadata.source().id().as_str(), source, "{id} source");
        assert_eq!(metadata.production(), production, "{id} production");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id} status");
        assert_eq!(metadata.supported_subset(), None, "{id} subset");
        assert_eq!(metadata.unsupported_remainder(), None, "{id} remainder");
        assert_eq!(metadata.recognized_unsupported_code(), None, "{id} code");
        assert!(metadata.baseline_alias_targets().is_empty(), "{id} atomic");

        if kind == CssFeatureKind::Property {
            let property = property_metadata(spelling)
                .unwrap_or_else(|| panic!("missing property metadata for {spelling}"));
            assert!(std::ptr::eq(metadata, property.feature()), "{id} owner");
        }

        let report = parse_style_attribute(vector);
        assert!(report.is_clean(), "{id}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{id} parser vector");
    }

    let promoted_properties = [
        "background",
        "background-color",
        "background-image",
        "background-size",
        "background-repeat",
        "background-origin",
        "background-clip",
        "background-attachment",
        "border",
        "border-top",
        "border-right",
        "border-bottom",
        "border-left",
        "border-width",
        "border-top-width",
        "border-right-width",
        "border-bottom-width",
        "border-left-width",
        "border-color",
        "border-top-color",
        "border-right-color",
        "border-bottom-color",
        "border-left-color",
        "border-style",
        "border-top-style",
        "border-right-style",
        "border-bottom-style",
        "border-left-style",
        "border-radius",
        "border-top-left-radius",
        "border-top-right-radius",
        "border-bottom-right-radius",
        "border-bottom-left-radius",
    ];
    assert_eq!(promoted_properties.len(), 33);
    for name in promoted_properties {
        let metadata = property_metadata(name)
            .unwrap_or_else(|| panic!("missing promoted property metadata for {name}"))
            .feature();
        assert_eq!(metadata.source().id().as_str(), "O-BACKGROUNDS3", "{name}");
        assert_eq!(metadata.production(), format!("#propdef-{name}"), "{name}");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{name}");
        assert_eq!(metadata.supported_subset(), None, "{name}");
        assert_eq!(metadata.unsupported_remainder(), None, "{name}");
    }

    for (id, source) in [
        ("baseline.property.background-position", "O-BACKGROUNDS3"),
        ("official.property.object-position", "O-IMAGES3"),
        ("baseline.property.box-shadow", "O-BACKGROUNDS3"),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing preserved {id}"));
        assert_eq!(metadata.source().id().as_str(), source, "{id}");
        assert_eq!(metadata.status(), CssSupportStatus::Complete, "{id}");
        assert_eq!(metadata.supported_subset(), None, "{id}");
        assert_eq!(metadata.unsupported_remainder(), None, "{id}");
    }
}
