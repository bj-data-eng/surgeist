//! Math left open at the end of input. Syntax 3 CRD 2026-10-01 §5.5.10
//! "consume a function" returns the function at `<eof-token>`, so a declaration
//! ending in `calc(1px` is valid and its value equals the explicitly closed
//! function (§5.5.10 is a parse error only for diagnostics). Public checked
//! constructors still reject such recovered math (#1374); parsing and
//! serialization must not.

use crate::css::support::sheet_cases::style_rule;
use crate::css::support::value_assertions::assert_first_property_diagnostic;
use surgeist_css::*;

/// The source with each function left open at the end explicitly closed.
fn closed(value: &str) -> String {
    let open = value.matches('(').count() - value.matches(')').count();
    format!("{value}{}", ")".repeat(open))
}

fn css_text(declarations: &CssDeclarationList) -> String {
    CssSpecifiedDeclarationBlock::try_from_declarations(declarations, Default::default())
        .unwrap()
        .serialize_cssom(Default::default())
        .unwrap()
}

/// Asserts that `property: value`, whose math is open at the end of input, is
/// kept in a style attribute and as the last declaration of a sheet, and that
/// both serialize as the explicitly closed source does.
#[track_caller]
fn assert_recovered_math_kept(property: &str, value: &str) {
    let source = format!("{property}: {value}");
    let closed_source = format!("color: red; {property}: {}", closed(value));
    let expected = CssParser::default().parse_block_contents(&closed_source);
    assert!(
        expected.is_clean(),
        "{closed_source}: {:?}",
        expected.diagnostics()
    );
    let expected = css_text(expected.syntax());

    let attribute = CssParser::default().parse_block_contents(format!("color: red; {source}"));
    assert_eq!(
        attribute.syntax().as_slice().len(),
        2,
        "{source} in a style attribute: {:?}",
        attribute.diagnostics()
    );
    assert_eq!(
        css_text(attribute.syntax()),
        expected,
        "{source} in a style attribute"
    );

    let sheet = CssParser::default().parse_sheet(format!(".a {{ color: red; {source}"));
    let declarations = style_rule(&sheet.syntax().rules()[0]).declarations();
    assert_eq!(
        declarations.as_slice().len(),
        2,
        "{source} in a sheet: {:?}",
        sheet.diagnostics()
    );
    assert_eq!(css_text(declarations), expected, "{source} in a sheet");
}

#[test]
fn absolute_voice_pitch_keeps_recovered_frequency_math() {
    assert_recovered_math_kept("voice-pitch", "absolute calc(100Hz");
    assert_recovered_math_kept("voice-pitch", "absolute calc(100Hz + 1kHz");
    // Caller-supplied components keep the strict checked policy.
    let context = CssParserContext::new(CssParserMode::Standards);
    let name = CssPropertyNameRef::Known(CssKnownProperty::VoicePitch);
    for (value, accepted) in [
        ("absolute calc(100Hz)", true),
        ("absolute calc(100Hz", false),
    ] {
        let values = CssComponentValues::parse(&CssParser::default(), value).unwrap();
        assert_eq!(
            CssDeclaration::try_from_components(
                &CssParser::new(context, Default::default()),
                CssPropertyTarget::Property(name),
                values,
                CssImportance::Normal
            )
            .is_ok(),
            accepted,
            "{value}"
        );
    }
}

#[test]
fn numeric_property_domains_keep_recovered_math() {
    for (property, value) in [
        ("width", "calc(1px + 2em"),
        ("opacity", "calc(0.5"),
        ("z-index", "calc(2"),
        ("rotate", "calc(10deg"),
        ("transition-duration", "calc(1s"),
        ("transition-delay", "calc(-1s"),
        ("font-weight", "calc(400"),
        ("font-style", "oblique calc(10deg"),
        ("font-feature-settings", "\"liga\" calc(1"),
        ("aspect-ratio", "1 / calc(2"),
        ("orphans", "calc(2"),
        ("line-height", "calc(1.5"),
        ("animation-iteration-count", "calc(2"),
        ("animation-timing-function", "steps(calc(2"),
        ("pause-before", "calc(1s"),
        ("voice-rate", "calc(50%"),
        ("voice-duration", "calc(1s"),
        ("voice-family", "male calc(2"),
        ("image-orientation", "calc(90deg"),
        ("column-count", "calc(2"),
        ("bookmark-level", "calc(2"),
        ("grid-column-start", "calc(2"),
        ("counter-increment", "a calc(2"),
        ("filter", "blur(calc(1px"),
        ("transform", "rotate(calc(10deg"),
    ] {
        assert_recovered_math_kept(property, value);
    }
}

/// Each domain below accepts a value whose math is left open, at the end of a
/// style attribute and of a sheet, and serializes it as the closed source.
#[test]
fn every_numeric_domain_keeps_recovered_math() {
    for (property, value) in [
        ("voice-balance", "calc(1"),
        ("voice-pitch", "calc(10Hz"),
        ("voice-pitch", "high calc(10Hz"),
        ("voice-pitch", "calc(10%"),
        ("voice-range", "calc(10Hz"),
        ("voice-range", "absolute calc(100Hz"),
        ("pause", "calc(1s"),
        ("pause", "1s calc(1s"),
        ("pause-after", "calc(1s"),
        ("rest", "calc(1s"),
        ("rest", "1s calc(1s"),
        ("rest-before", "calc(1s"),
        ("rest-after", "calc(1s"),
        ("interest-delay", "calc(1s"),
        ("interest-delay", "1s calc(1s"),
        ("interest-delay-start", "calc(1s"),
        ("text-underline-offset", "calc(1px"),
        ("text-decoration-thickness", "calc(1px"),
        ("text-shadow", "1px calc(2px"),
        ("box-shadow", "1px 2px calc(3px"),
        ("box-shadow", "1px 2px 3px calc(4px"),
        ("letter-spacing", "calc(1px"),
        ("word-spacing", "calc(1px"),
        ("tab-size", "calc(4"),
        ("tab-size", "calc(4px"),
        ("line-padding", "calc(1px"),
        ("text-indent", "calc(1px"),
        ("vertical-align", "calc(1px"),
        ("scroll-margin-top", "calc(1px"),
        ("scroll-padding-top", "calc(1px"),
        ("margin", "calc(1px"),
        ("margin", "1px calc(1px"),
        ("padding", "calc(1px"),
        ("inset", "calc(1px"),
        ("border-radius", "calc(1px"),
        ("border-radius", "1px / calc(1px"),
        ("border-width", "calc(1px"),
        ("border-image-slice", "calc(1"),
        ("border-image-width", "calc(1"),
        ("border-image-outset", "calc(1"),
        ("border-spacing", "calc(1px"),
        ("outline-offset", "calc(1px"),
        ("shape-margin", "calc(1px"),
        ("offset-distance", "calc(1px"),
        ("shape-image-threshold", "calc(0.5"),
        ("perspective", "calc(1px"),
        ("scale", "calc(1"),
        ("scale", "1 calc(2"),
        ("translate", "calc(1px"),
        ("translate", "1px calc(2px"),
        ("rotate", "x calc(10deg"),
        ("rotate", "1 2 3 calc(10deg"),
        ("transform", "translate(calc(1px"),
        ("transform", "scale(calc(1"),
        ("transform", "perspective(calc(1px"),
        ("transform", "skew(calc(1deg"),
        ("transform", "rotate3d(1, 0, 0, calc(10deg"),
        ("transform-origin", "calc(1px"),
        ("perspective-origin", "calc(1px"),
        ("object-position", "calc(1px"),
        ("background-position", "calc(1px"),
        ("background-position", "left calc(1px"),
        ("background-size", "calc(1px"),
        ("background-size", "1px calc(2px"),
        ("background-image", "linear-gradient(red calc(10%"),
        ("background-image", "linear-gradient(red, blue calc(10%"),
        ("background-image", "conic-gradient(red calc(10deg"),
        ("mask-border-slice", "calc(1"),
        ("gap", "calc(1px"),
        ("gap", "1px calc(1px"),
        ("column-gap", "calc(1px"),
        ("column-width", "calc(1px"),
        ("column-rule-width", "calc(1px"),
        ("columns", "calc(2"),
        ("columns", "calc(1px"),
        ("grid-template-columns", "repeat(2, calc(1px"),
        ("grid-template-columns", "fit-content(calc(1px"),
        ("grid-auto-rows", "calc(1fr"),
        ("grid-row", "calc(2"),
        ("grid-row", "1 / calc(2"),
        ("grid-column-end", "span calc(2"),
        ("flex", "calc(1"),
        ("flex", "1 1 calc(1px"),
        ("flex-basis", "calc(1px"),
        ("flex-grow", "calc(1"),
        ("order", "calc(1"),
        ("aspect-ratio", "calc(1"),
        ("font-size", "calc(1px"),
        ("font-size-adjust", "calc(1"),
        ("font-stretch", "calc(100%"),
        ("font-variation-settings", "\"wght\" calc(400"),
        ("hyphenate-limit-chars", "calc(1"),
        ("hyphenate-limit-chars", "5 calc(1"),
        ("line-height", "calc(1"),
        ("widows", "calc(2"),
        ("z-index", "calc(1"),
        ("counter-reset", "a calc(1"),
        ("counter-set", "a calc(1"),
        ("animation-iteration-count", "calc(1"),
        ("animation-duration", "calc(1s"),
        ("animation-delay", "calc(1s"),
        ("animation-timing-function", "linear(0, calc(0.5"),
        ("animation-timing-function", "linear(0, 0.5 calc(10%"),
        ("transition", "calc(1s"),
        ("transition", "all calc(1s"),
        ("animation", "calc(1s"),
        ("animation", "fade calc(1"),
        ("transition-delay", "calc(1s"),
        ("filter", "brightness(calc(1"),
        ("filter", "hue-rotate(calc(10deg"),
        ("filter", "drop-shadow(1px 2px calc(3px"),
        ("backdrop-filter", "opacity(calc(50%"),
        ("clip-path", "circle(calc(10px"),
        ("clip-path", "inset(calc(10px"),
        ("shape-outside", "circle(calc(10px"),
        ("offset-path", "ray(calc(10deg"),
        ("contain-intrinsic-size", "calc(1px"),
        ("contain-intrinsic-size", "auto calc(1px"),
        ("width", "fit-content(calc(1px"),
        ("width", "min(1px"),
        ("width", "max(1px, calc(2px"),
        ("width", "clamp(1px, 2px, calc(3px"),
        ("width", "round(1px, calc(2px"),
        ("width", "mod(1px, calc(2px"),
        ("width", "abs(calc(2px"),
        ("color", "rgb(0 0 calc(2"),
        ("color", "color(srgb 0 0 calc(1"),
        ("color", "color-mix(in srgb, red calc(10%"),
        ("background-color", "rgb(0 0 0 / calc(0.5"),
        ("border-top-left-radius", "calc(1px"),
        ("text-shadow", "1px 2px calc(3px"),
        ("text-shadow", "red 1px 2px calc(3px"),
        ("clip", "rect(1px, 2px, 3px, calc(4px"),
        ("transform", "matrix(1, 0, 0, 1, 0, calc(1"),
        (
            "transform",
            "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, calc(1",
        ),
        (
            "animation-timing-function",
            "cubic-bezier(0.1, 0.2, 0.3, calc(0.5",
        ),
        ("clip-path", "polygon(0 0, 1px calc(10px"),
        ("background-image", "conic-gradient(red, blue calc(10deg"),
        ("background-image", "radial-gradient(red, blue calc(10%"),
        ("grid-template-columns", "minmax(1px, calc(2px"),
        ("color", "rgb(1 2 calc(3"),
        ("color", "rgb(1, 2, calc(3"),
        ("color", "hsl(10 20% calc(30%"),
        ("color", "hsla(10, 20%, 30%, calc(0.5"),
        ("color", "oklch(0.5 0.1 calc(10"),
        ("color", "lab(50% 10 calc(10"),
        ("color", "rgb(from red r g calc(1"),
        ("color", "color-mix(in oklab, red 10%, blue calc(20%"),
        ("color", "color-mix(in hsl shorter hue, red, blue calc(20%"),
        (
            "background-image",
            "linear-gradient(to right, red, blue calc(10%",
        ),
        (
            "background-image",
            "linear-gradient(red 0%, blue calc(10% + 5%",
        ),
        (
            "background-image",
            "repeating-linear-gradient(red, blue calc(10px",
        ),
        (
            "background-image",
            "linear-gradient(in oklab, red, blue calc(10%",
        ),
        ("filter", "drop-shadow(red 1px 2px calc(3px"),
        ("filter", "sepia(calc(50%"),
        ("transform", "translate3d(1px, 2px, calc(3px"),
        ("transform", "scale3d(1, 2, calc(3"),
        ("transform", "rotate(calc(10deg + 1deg"),
        ("transform", "perspective(calc(100px"),
        ("transform", "translateX(calc(1px"),
        ("width", "calc-size(auto, size + calc(1px"),
        ("animation", "1s calc(1s"),
        ("transition", "1s calc(1s"),
        ("border", "calc(1px"),
        ("outline", "calc(1px"),
        ("border-image", "url(a.png) 1 / calc(1"),
        ("border-image", "url(a.png) 1 / 1 / calc(1"),
        ("mask-border", "url(a.png) 1 / calc(1"),
        ("mask-position", "calc(1px"),
        ("mask-size", "calc(1px"),
        ("column-rule", "calc(1px"),
        ("columns", "2 calc(1px"),
        ("flex-flow", "row"),
        ("grid-area", "1 / 1 / 1 / calc(1"),
        ("text-decoration", "underline calc(1px"),
        ("hyphenate-limit-chars", "auto auto calc(1"),
        ("overflow-clip-margin", "calc(1px"),
        ("scroll-padding", "calc(1px"),
        ("scroll-margin", "1px calc(1px"),
        ("background", "calc(1px"),
        ("background", "url(a.png) calc(1px"),
        ("background", "url(a.png) 1px calc(2px"),
        ("background", "url(a.png) 1px 2px / calc(3px"),
        ("mask", "url(a.png) 1px calc(2px"),
        ("shape-outside", "inset(1px round calc(10px"),
        ("clip-path", "inset(1px round calc(10px"),
        ("offset", "calc(1px"),
        ("offset-position", "calc(1px"),
        ("offset-anchor", "calc(1px"),
        ("offset-rotate", "calc(10deg"),
        ("offset-rotate", "auto calc(10deg"),
        ("text-decoration-thickness", "calc(1px + 2%"),
        ("font-size", "calc(1px + 2%"),
        ("font-size", "calc(1em"),
        ("font-style", "oblique calc(10deg + 2deg"),
        ("font-width", "calc(100%"),
        ("font-stretch", "calc(50%"),
        ("font-variation-settings", "\"wght\" calc(1"),
        ("voice-family", "male calc(1"),
        ("voice-family", "young female calc(1"),
        ("voice-family", "\"a\", male calc(2"),
        ("voice-pitch", "x-high calc(5Hz"),
        ("voice-pitch", "calc(5Hz + 1%"),
        ("voice-range", "calc(5Hz + 1%"),
        ("pause", "1s calc(2s"),
        ("rest", "1s calc(2s"),
        ("rest-after", "calc(1s + 2s"),
        ("pause-before", "calc(1s + 2s"),
    ] {
        assert_recovered_math_kept(property, value);
    }
}

/// `cursor` requires a trailing keyword, so a value ending in open math is
/// invalid; the resolution itself is valid and is not the responsible token.
#[test]
fn cursor_rejection_admits_recovered_resolution_math_like_a_bare_resolution() {
    for source in [
        "cursor: image-set(\"a.png\" 2dppx",
        "cursor: image-set(\"a.png\" calc(2dppx",
    ] {
        let report = CssParser::default().parse_block_contents(source);
        assert!(report.syntax().as_slice().is_empty(), "{source}");
        assert_first_property_diagnostic(
            report.diagnostics(),
            source,
            CssKnownProperty::Cursor,
            "2dppx",
        );
    }
}
