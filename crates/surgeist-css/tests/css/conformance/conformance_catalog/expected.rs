//! Declared metadata and public parser vectors for named catalog records.
//!
//! Each row states the record's kind, cited source, production anchor and
//! status. A spelling is stated only when it is an authored token or grammar
//! name; descriptive spellings and subset/remainder prose are not re-asserted,
//! because the catalog status invariants cover their presence. `accepts`
//! inputs must parse without diagnostics; `rejects` inputs must report the
//! given root code first.

use surgeist_css::conformance::CssFeatureKind as K;
use surgeist_css::conformance::CssSupportStatus;
use surgeist_css::diagnostic::CssErrorCode;

#[derive(Clone, Copy, Debug)]
pub(super) enum Input {
    Sheet(&'static str),
    Style(&'static str),
}

use Input::{Sheet, Style};

#[derive(Clone, Copy, Debug)]
pub(super) struct ExpectedFeature {
    pub(super) id: &'static str,
    pub(super) kind: K,
    pub(super) spelling: Option<&'static str>,
    pub(super) source: &'static str,
    pub(super) production: &'static str,
    pub(super) status: CssSupportStatus,
    pub(super) positive: Option<Input>,
    pub(super) negative: Option<(Input, CssErrorCode)>,
}

const fn record(
    id: &'static str,
    kind: K,
    source: &'static str,
    production: &'static str,
    status: CssSupportStatus,
) -> ExpectedFeature {
    ExpectedFeature {
        id,
        kind,
        spelling: None,
        source,
        production,
        status,
        positive: None,
        negative: None,
    }
}

const fn complete(
    id: &'static str,
    kind: K,
    source: &'static str,
    production: &'static str,
) -> ExpectedFeature {
    record(id, kind, source, production, CssSupportStatus::Complete)
}

const fn partial(
    id: &'static str,
    kind: K,
    source: &'static str,
    production: &'static str,
) -> ExpectedFeature {
    record(id, kind, source, production, CssSupportStatus::Partial)
}

impl ExpectedFeature {
    const fn spelled(mut self, spelling: &'static str) -> Self {
        self.spelling = Some(spelling);
        self
    }

    const fn accepts(mut self, input: Input) -> Self {
        self.positive = Some(input);
        self
    }

    const fn rejects(mut self, input: Input, code: CssErrorCode) -> Self {
        self.negative = Some((input, code));
        self
    }
}

pub(super) const EXPECTED: &[ExpectedFeature] = &[
    complete("baseline.rule.import", K::Rule, "O-CASCADE4", "#at-import")
        .spelled("@import")
        .accepts(Sheet("@import url(theme.css) supports(display: grid) print;")),
    complete("ext.import.layer", K::Rule, "R-CASCADE5", "#at-import")
        .accepts(Sheet("@import url(theme.css) layer(theme) supports(display: grid) print;")),
    complete("ext.stylesheet.prelude-order", K::Rule, "R-CASCADE5", "#at-import")
        .accepts(Sheet("@layer reset; @import url(theme.css) layer(theme) supports(display: grid) print;")),
    partial("baseline.rule.layer-statement", K::Rule, "R-CASCADE5", "#layering")
        .accepts(Sheet("@layer reset, theme;"))
        .rejects(Sheet("@layer initial;"), CssErrorCode::InvalidAtRulePrelude),
    partial("baseline.rule.layer-block", K::Rule, "R-CASCADE5", "#layering")
        .accepts(Sheet("@layer theme { .x { color: red; } }"))
        .rejects(Sheet("@layer first, second { .x { color: red; } }"), CssErrorCode::InvalidAtRuleBody),
    complete("baseline.rule.font-face", K::Rule, "I-FONTS4-20260907", "#font-face-rule")
        .spelled("@font-face")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2); }")),
    complete("baseline.rule.keyframes", K::Rule, "I-ANIMATIONS1", "#keyframes")
        .spelled("@keyframes")
        .accepts(Sheet("@keyframes fade { from { opacity: 0; } to { opacity: 1; } }")),
    partial("baseline.rule.style", K::Rule, "O-SYNTAX3", "#style-rules")
        .accepts(Sheet(".x { color: red; }"))
        .rejects(Sheet("??? { color: red; }"), CssErrorCode::InvalidSelector),
    complete("official.rule.at-rule", K::Rule, "O-SYNTAX3", "#at-rules,#consume-at-rule")
        .accepts(Sheet("@media print { @font-feature-values Demo { @styleset { nice: 1; } } }")),
    complete("official.qualified-rule.generic", K::Rule, "O-SYNTAX3", "#consume-qualified-rule")
        .accepts(Sheet(".x { color: red; }")),
    complete("baseline.rule.media", K::Rule, "O-CONDITIONAL3", "#at-media")
        .spelled("@media")
        .accepts(Sheet("@media screen { .x { color: red; } }")),
    complete("official.rule.conditional-group-context", K::Rule, "O-CONDITIONAL3", "#contents-of,#placement")
        .accepts(Sheet("@media print { @supports (display: grid) { .x { color: red; } } }")),
    partial("baseline.rule.container", K::Rule, "X-CONDITIONAL5", "#container-rule")
        .spelled("@container")
        .accepts(Sheet("@container (width > 1px) { .x { color: red; } }"))
        .rejects(Sheet("@container scroll-state(stuck: top) and { .x { color: red; } }"), CssErrorCode::InvalidAtRulePrelude),
    partial("baseline.rule.scope", K::Rule, "X-CASCADE6", "#scope-atrule")
        .spelled("@scope")
        .accepts(Sheet("@scope (.card) { .title { color: red; } }"))
        .rejects(Sheet("@scope .card { .title { color: red; } }"), CssErrorCode::InvalidAtRulePrelude),
    complete("foundation.declaration-list.style-attribute", K::Declaration, "O-STYLE-ATTR", "#syntax")
        .accepts(Style("color: red; width: 1px")),
    complete("foundation.declaration.importance", K::Declaration, "O-CASCADE4", "#importance")
        .accepts(Style("color: red !important")),
    complete("official.declaration.generic", K::Declaration, "O-SYNTAX3", "#consume-declaration")
        .accepts(Style("color: red")),
    complete("official.value.syntax-token-stream", K::Value, "O-SYNTAX3", "#tokenization")
        .accepts(Style("--x: [ { ( 1px ) } ]")),
    complete("official.value.component-value", K::Value, "O-SYNTAX3", "#consume-component-value")
        .spelled("<component-value>")
        .accepts(Style("--x: fn(1, 2) [a] {b}")),
    complete("official.value.simple-block", K::Value, "O-SYNTAX3", "#consume-simple-block")
        .accepts(Style("--x: (1px) [2px] {3px}")),
    complete("official.value.function", K::Value, "O-SYNTAX3", "#consume-function")
        .spelled("function")
        .accepts(Style("--x: f(a, b)")),
    complete("official.value.stylesheet", K::Value, "O-SYNTAX3", "#parser-entry-points")
        .spelled("<stylesheet>")
        .accepts(Sheet("@layer a; .x { color: red; }")),
    complete("official.value.rule-list", K::Value, "O-SYNTAX3", "#typedef-rule-list")
        .spelled("<rule-list>")
        .accepts(Sheet("@media print { .x { color: red; } .y { color: blue; } }")),
    complete("official.value.declaration-list", K::Value, "O-SYNTAX3", "#typedef-declaration-list")
        .spelled("<declaration-list>")
        .accepts(Sheet("@font-face { font-family: Demo; src: url(demo.woff2); }")),
    complete("official.value.style-block", K::Value, "O-SYNTAX3", "#typedef-block-contents")
        .spelled("<style-block>")
        .accepts(Sheet(".x { color: red; & .y { color: blue; } }")),
    complete("official.value.declaration-value", K::Value, "O-SYNTAX3", "#any-value")
        .spelled("<declaration-value>")
        .accepts(Style("--x: 1px solid red")),
    complete("official.value.any-value", K::Value, "O-SYNTAX3", "#any-value")
        .spelled("<any-value>")
        .accepts(Sheet("@supports (--x: !) { .x { color: red; } }")),
    complete("official.value.an-plus-b", K::Value, "O-SYNTAX3", "#the-anb-type")
        .spelled("<an+b>")
        .accepts(Sheet(":nth-child(-n+3) { color: red; }"))
        .rejects(Sheet(":nth-child(n-) { color: red; }"), CssErrorCode::InvalidSelector),
    complete("official.value.unicode-range", K::Value, "O-SYNTAX3", "#consume-unicode-range-value")
        .spelled("<urange>")
        .accepts(Sheet("@font-face { font-family: Demo; src: url(demo.woff2); unicode-range: U+00??, U+0-7F; }"))
        .rejects(Sheet("@font-face { font-family: Demo; src: url(demo.woff2); unicode-range: U+110000; }"), CssErrorCode::InvalidDescriptorValue),
    partial("baseline.declaration.custom-property", K::Declaration, "O-VARIABLES1", "#defining-variables,#syntax")
        .accepts(Style("--x: inherit 1px"))
        .rejects(Style("--: 1px"), CssErrorCode::UnknownProperty),
    partial("baseline.value.substitution-dependent", K::Value, "O-VARIABLES1", "#using-variables")
        .accepts(Style("width: var(--width, 1px)"))
        .rejects(Style("width: var(color)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.css-wide-keyword", K::Value, "O-CASCADE4", "#defaulting-keywords")
        .accepts(Style("color: revert-layer")),
    complete("official.value.custom-ident", K::Value, "O-VALUES3", "#custom-idents")
        .spelled("<custom-ident>")
        .accepts(Style("animation-name: slide-in"))
        .rejects(Style("animation-name: default"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.ident", K::Value, "O-VALUES3", "#custom-idents")
        .spelled("<ident>")
        .accepts(Style("display: block")),
    complete("official.value.string", K::Value, "O-VALUES3", "#strings")
        .spelled("<string>")
        .accepts(Style("content: \"label\"")),
    complete("official.value.url", K::Value, "I-VALUES4-20240312", "#urls")
        .spelled("<url>")
        .accepts(Style("background-image: url(hero.png)")),
    complete("official.value.url-modifier", K::Value, "I-VALUES4-20240312", "#url-modifiers")
        .spelled("<url-modifier>")
        .accepts(Sheet("@import url(\"theme.css\" format-like(x)) screen;")),
    complete("official.value.integer", K::Value, "O-VALUES3", "#integers")
        .spelled("<integer>")
        .accepts(Style("z-index: -3"))
        .rejects(Style("z-index: 1.5"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.number", K::Value, "O-VALUES3", "#numbers")
        .spelled("<number>")
        .accepts(Style("flex-grow: 1.5"))
        .rejects(Style("flex-grow: 1px"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.dimension", K::Value, "O-VALUES3", "#dimensions")
        .spelled("<dimension>")
        .accepts(Style("width: 1.5rem"))
        .rejects(Style("width: 1.5qux"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.percentage", K::Value, "O-VALUES3", "#percentages")
        .spelled("<percentage>")
        .accepts(Style("text-indent: -12.5%"))
        .rejects(Style("text-indent: 12.5 %"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.length", K::Value, "O-VALUES3", "#lengths")
        .spelled("<length>")
        .accepts(Style("width: 2cqw"))
        .rejects(Style("width: 2deg"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.length-percentage", K::Value, "O-VALUES3", "#mixed-percentages")
        .spelled("<length-percentage>")
        .accepts(Style("width: calc((1px + 2%) * 3)"))
        .rejects(Style("width: calc(1px + 2deg)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.angle", K::Value, "O-VALUES3", "#angles")
        .spelled("<angle>")
        .accepts(Style("rotate: -0.5turn"))
        .rejects(Style("rotate: 1hz"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.angle-percentage", K::Value, "O-VALUES3", "#mixed-percentages")
        .spelled("<angle-percentage>")
        .accepts(Style("background-image: conic-gradient(red 10%, blue 90deg)"))
        .rejects(Style("background-image: conic-gradient(red 10px, blue)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.time", K::Value, "O-VALUES3", "#time")
        .spelled("<time>")
        .accepts(Style("transition-delay: -250ms"))
        .rejects(Style("transition-delay: 1px"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.time-percentage", K::Value, "O-VALUES3", "#mixed-percentages")
        .spelled("<time-percentage>")
        .accepts(Style("transition-delay: calc(-1s + 250ms)"))
        .rejects(Style("transition-delay: calc(1s + 10%)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.frequency", K::Value, "O-VALUES3", "#frequency")
        .spelled("<frequency>")
        .accepts(Style("voice-pitch: 200Hz absolute"))
        .rejects(Style("voice-pitch: 200px absolute"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.frequency-percentage", K::Value, "O-VALUES3", "#mixed-percentages")
        .spelled("<frequency-percentage>")
        .accepts(Style("voice-pitch: calc(100Hz + 10%)"))
        .rejects(Style("voice-pitch: calc(100Hz + 10deg)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.resolution", K::Value, "O-VALUES3", "#resolution")
        .spelled("<resolution>")
        .accepts(Sheet("@media (resolution: 2dppx) { .x { color: red; } }")),
    complete("official.value.color", K::Value, "O-COLOR4", "#color-type")
        .spelled("<color>")
        .accepts(Style(concat!("color: ", "red"))),
    complete("official.value.alpha", K::Value, "O-COLOR4", "#alpha-syntax")
        .spelled("<alpha-value>")
        .accepts(Style(concat!("color: ", "rgb(0 0 0 / 150%)"))),
    complete("official.value.hue", K::Value, "O-COLOR4", "#hue-syntax")
        .spelled("<hue>")
        .accepts(Style(concat!("color: ", "hsl(calc(1turn - 90deg) 50% 50%)"))),
    complete("official.value.rgb", K::Value, "O-COLOR4", "#rgb-functions")
        .spelled("rgb()/rgba()")
        .accepts(Style(concat!("color: ", "rgb(calc(1 + 2) 20% none / 150%)"))),
    complete("official.value.hex-color", K::Value, "O-COLOR4", "#hex-notation")
        .spelled("<hex-color>")
        .accepts(Style(concat!("color: ", "#00ff88cc"))),
    complete("official.value.named-color", K::Value, "O-COLOR4", "#named-colors")
        .spelled("<named-color>")
        .accepts(Style(concat!("color: ", "rebeccapurple"))),
    complete("official.value.system-color", K::Value, "O-COLOR4", "#css-system-colors")
        .spelled("<system-color>")
        .accepts(Style(concat!("color: ", "CanvasText"))),
    complete("official.value.deprecated-system-color", K::Value, "O-COLOR4", "#css-system-colors")
        .spelled("<deprecated-system-color>")
        .accepts(Style(concat!("color: ", "ActiveBorder"))),
    complete("official.value.transparent", K::Value, "O-COLOR4", "#transparent-color")
        .spelled("transparent")
        .accepts(Style(concat!("color: ", "transparent"))),
    complete("official.value.currentcolor", K::Value, "O-COLOR4", "#currentcolor-color")
        .spelled("currentColor")
        .accepts(Style(concat!("color: ", "currentColor"))),
    complete("official.value.hsl", K::Value, "O-COLOR4", "#the-hsl-notation")
        .spelled("hsl()/hsla()")
        .accepts(Style(concat!("color: ", "hsl(30deg 120 -20% / none)"))),
    complete("official.value.hwb", K::Value, "O-COLOR4", "#the-hwb-notation")
        .spelled("hwb()")
        .accepts(Style(concat!("color: ", "hwb(none 20 120% / -10%)"))),
    complete("official.value.lab", K::Value, "O-COLOR4", "#specifying-lab-lch")
        .spelled("lab()")
        .accepts(Style(concat!("color: ", "lab(50% 20 -30 / 120%)"))),
    complete("official.value.lch", K::Value, "O-COLOR4", "#specifying-lab-lch")
        .spelled("lch()")
        .accepts(Style(concat!("color: ", "lch(50% 20 30deg / none)"))),
    complete("official.value.oklab", K::Value, "O-COLOR4", "#specifying-oklab-oklch")
        .spelled("oklab()")
        .accepts(Style(concat!("color: ", "oklab(50% 0.1 -0.1)"))),
    complete("official.value.oklch", K::Value, "O-COLOR4", "#specifying-oklab-oklch")
        .spelled("oklch()")
        .accepts(Style(concat!("color: ", "oklch(50% 0.2 30deg)"))),
    complete("official.value.predefined-color", K::Value, "O-COLOR4", "#color-function")
        .spelled("color()")
        .accepts(Style(concat!("color: ", "color(display-p3 1 0 0 / 120%)"))),
    partial("ext.value.relative-color", K::Value, "I-COLOR5", "#relative-colors,#relative-syntax")
        .accepts(Style(concat!("color: ", "rgb(from red r g b / alpha)")))
        .rejects(Style(concat!("color: ", "alpha(from red r g b)")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.rgb", K::Value, "I-COLOR5", "#relative-RGB")
        .accepts(Style(concat!("color: ", "rgb(from red r g b / alpha)")))
        .rejects(Style(concat!("color: ", "rgb(from red h g b)")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.hsl", K::Value, "I-COLOR5", "#relative-HSL")
        .accepts(Style(concat!("color: ", "hsl(from red h s l / alpha)")))
        .rejects(Style(concat!("color: ", "hsl(from red 10% s l)")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.hwb", K::Value, "I-COLOR5", "#relative-HWB")
        .accepts(Style(concat!("color: ", "hwb(from red h w b / alpha)")))
        .rejects(Style(concat!("color: ", "hwb(from red h s b)")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.lab", K::Value, "I-COLOR5", "#relative-Lab")
        .accepts(Style(concat!("color: ", "lab(from red l a b / alpha)")))
        .rejects(Style(concat!("color: ", "lab(from red l c b)")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.oklab", K::Value, "I-COLOR5", "#relative-Oklab")
        .accepts(Style(concat!("color: ", "oklab(from red l a b / alpha)")))
        .rejects(Style(concat!("color: ", "oklab(from red l c b)")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.lch", K::Value, "I-COLOR5", "#relative-LCH")
        .accepts(Style(concat!("color: ", "lch(from red l c h / alpha)")))
        .rejects(Style(concat!("color: ", "lch(from red l c calc(h + 10%))")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.oklch", K::Value, "I-COLOR5", "#relative-OkLCh")
        .accepts(Style(concat!("color: ", "oklch(from red l c h / alpha)")))
        .rejects(Style(concat!("color: ", "oklch(from red l c calc(h + 10%))")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.relative-color.predefined", K::Value, "I-COLOR5", "#relative-color-function")
        .accepts(Style(concat!("color: ", "color(from red display-p3 r g b / alpha)")))
        .rejects(Style(concat!("color: ", "color(from red srgb x g b)")), CssErrorCode::InvalidColorSyntax),
    partial("ext.value.color-mix", K::Value, "I-COLOR5", "#funcdef-color-mix")
        .spelled("color-mix()")
        .accepts(Style(concat!("color: ", "color-mix(in oklch longer hue, red 25%, blue 75%)")))
        .rejects(Style(concat!("color: ", "color-mix(in srgb longer hue, red, blue)")), CssErrorCode::InvalidColorSyntax),
    complete("ext.value.grid-repeat", K::Value, "X-GRID3-20260121", "#intrinsic-auto-repeat")
        .spelled("repeat()")
        .accepts(Style("grid-template-columns: repeat(auto-fit, 10px)"))
        .rejects(Style("grid-template-columns: repeat(auto-fit, minmax(1fr, 10px))"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.position", K::Value, "O-VALUES3", "#position")
        .spelled("<position>")
        .accepts(Style("object-position: right 5% bottom 2px"))
        .rejects(Style("object-position: left right"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.background-position", K::Value, "O-BACKGROUNDS3", "#background-position")
        .spelled("<bg-position>#")
        .accepts(Style("background-position: left 10px top"))
        .rejects(Style("background-position: top 10px 20px"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter-image", K::Value, "I-FILTER1", "#FilterCSSImageValue")
        .spelled("filter()")
        .accepts(Style("background-image: filter(url(hero.png), blur(2px))"))
        .rejects(Style("background-image: filter(url(hero.png))"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform-list", K::Value, "O-TRANSFORMS1", "#transform-function-lists")
        .spelled("<transform-list>")
        .accepts(Style("transform: translate(1px) rotate(45deg)"))
        .rejects(Style("transform: translate(1px), rotate(45deg)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform-function", K::Value, "O-TRANSFORMS1", "#transform-functions")
        .spelled("<transform-function>")
        .accepts(Style("transform: skewX(5deg)"))
        .rejects(Style("transform: unknown(1px)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.matrix", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("matrix()")
        .accepts(Style("transform: matrix(1, 0, 0, 1, 10, 20)"))
        .rejects(Style("transform: matrix(1, 0, 0, 1, 10)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.translate", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("translate()")
        .accepts(Style("transform: translate(1px, 2%)"))
        .rejects(Style("transform: translate(1deg)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.translate-x", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("translateX()")
        .accepts(Style("transform: translateX(3px)"))
        .rejects(Style("transform: translateX(3px, 4px)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.translate-y", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("translateY()")
        .accepts(Style("transform: translateY(4%)"))
        .rejects(Style("transform: translateY(4deg)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.scale", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("scale()")
        .accepts(Style("transform: scale(1.5, 2)"))
        .rejects(Style("transform: scale(1px)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.scale-x", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("scaleX()")
        .accepts(Style("transform: scaleX(.5)"))
        .rejects(Style("transform: scaleX(.5px)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.scale-y", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("scaleY()")
        .accepts(Style("transform: scaleY(2)"))
        .rejects(Style("transform: scaleY(2deg)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.rotate", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("rotate()")
        .accepts(Style("transform: rotate(45deg)"))
        .rejects(Style("transform: rotate(45px)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.skew", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("skew()")
        .accepts(Style("transform: skew(10deg, 20deg)"))
        .rejects(Style("transform: skew(10px)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.skew-x", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("skewX()")
        .accepts(Style("transform: skewX(5deg)"))
        .rejects(Style("transform: skewX(5deg, 6deg)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.transform.skew-y", K::Value, "O-TRANSFORMS1", "#two-d-transform-functions")
        .spelled("skewY()")
        .accepts(Style("transform: skewY(6deg)"))
        .rejects(Style("transform: skewY(6%)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.easing-function", K::Value, "D-EASING2", "#easing-functions")
        .spelled("<easing-function>")
        .accepts(Style("transition-timing-function: ease, linear"))
        .rejects(Style("transition-timing-function: ease-sideways"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.linear-easing", K::Value, "D-EASING2", "#linear-easing-function-syntax,#create-a-linear-easing-function")
        .spelled("linear()")
        .accepts(Style("transition-timing-function: linear(0, 0.25 25% 75%, 1)"))
        .rejects(Style("transition-timing-function: linear(0)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.cubic-bezier-easing", K::Value, "O-EASING1", "#cubic-bezier-easing-functions")
        .spelled("cubic-bezier()")
        .accepts(Style("transition-timing-function: cubic-bezier(.25, 0, .75, 1)"))
        .rejects(Style("transition-timing-function: cubic-bezier(2, 0, .75, 1)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.step-easing", K::Value, "O-EASING1", "#step-easing-functions")
        .spelled("steps()")
        .accepts(Style("transition-timing-function: steps(2, jump-none)"))
        .rejects(Style("transition-timing-function: steps(1, jump-none)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.step-position", K::Value, "O-EASING1", "#step-easing-functions")
        .spelled("<step-position>")
        .accepts(Style("animation-timing-function: steps(3, start), step-end"))
        .rejects(Style("animation-timing-function: steps(3, middle)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.shadow", K::Value, "O-BACKGROUNDS3", "#box-shadow")
        .spelled("<shadow>")
        .accepts(Style("box-shadow: red inset -1px 2px 3px -4px"))
        .rejects(Style("box-shadow: 1px"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.matrix3d", K::Value, "I-TRANSFORMS2", "#funcdef-matrix3d")
        .spelled("matrix3d()")
        .accepts(Style("transform: matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)"))
        .rejects(Style("transform: matrix3d(1, 0, 0, 1)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.perspective", K::Value, "I-TRANSFORMS2", "#funcdef-perspective")
        .spelled("perspective()")
        .accepts(Style("transform: perspective(none) perspective(0)"))
        .rejects(Style("transform: perspective(-1px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.rotate3d", K::Value, "I-TRANSFORMS2", "#funcdef-rotate3d")
        .spelled("rotate3d()")
        .accepts(Style("transform: rotate3d(1, 0, -1, 45deg)"))
        .rejects(Style("transform: rotate3d(1, 0, 45deg)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.rotate-x", K::Value, "I-TRANSFORMS2", "#funcdef-rotatex")
        .spelled("rotateX()")
        .accepts(Style("transform: rotateX(10deg)"))
        .rejects(Style("transform: rotateX(10px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.rotate-y", K::Value, "I-TRANSFORMS2", "#funcdef-rotatey")
        .spelled("rotateY()")
        .accepts(Style("transform: rotateY(20deg)"))
        .rejects(Style("transform: rotateY(20%)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.rotate-z", K::Value, "I-TRANSFORMS2", "#funcdef-rotatez")
        .spelled("rotateZ()")
        .accepts(Style("transform: rotateZ(30deg)"))
        .rejects(Style("transform: rotateZ(30)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.scale3d", K::Value, "I-TRANSFORMS2", "#funcdef-scale3d")
        .spelled("scale3d()")
        .accepts(Style("transform: scale3d(1, 50%, 2)"))
        .rejects(Style("transform: scale3d(1, 2)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.scale-z", K::Value, "I-TRANSFORMS2", "#funcdef-scalez")
        .spelled("scaleZ()")
        .accepts(Style("transform: scaleZ(125%)"))
        .rejects(Style("transform: scaleZ(1px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.translate3d", K::Value, "I-TRANSFORMS2", "#funcdef-translate3d")
        .spelled("translate3d()")
        .accepts(Style("transform: translate3d(10%, 2px, 4em)"))
        .rejects(Style("transform: translate3d(1px, 2px, 3%)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.transform.translate-z", K::Value, "I-TRANSFORMS2", "#funcdef-translatez")
        .spelled("translateZ()")
        .accepts(Style("transform: translateZ(3px)"))
        .rejects(Style("transform: translateZ(3%)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter-function-list", K::Value, "I-FILTER1", "#FilterProperty")
        .spelled("<filter-function-list>")
        .accepts(Style("filter: url(\"filters.svg#rough\") blur(2px) drop-shadow(1px 2px red) opacity(50%)"))
        .rejects(Style("filter: blur(2px), opacity(50%)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.blur", K::Value, "I-FILTER1", "#funcdef-filter-blur")
        .spelled("blur()")
        .accepts(Style("filter: blur(2px)"))
        .rejects(Style("filter: blur(-2px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.brightness", K::Value, "I-FILTER1", "#funcdef-filter-brightness")
        .spelled("brightness()")
        .accepts(Style("filter: brightness()"))
        .rejects(Style("filter: brightness(-1)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.contrast", K::Value, "I-FILTER1", "#funcdef-filter-contrast")
        .spelled("contrast()")
        .accepts(Style("filter: contrast(25%)"))
        .rejects(Style("filter: contrast(25px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.grayscale", K::Value, "I-FILTER1", "#funcdef-filter-grayscale")
        .spelled("grayscale()")
        .accepts(Style("filter: grayscale(.5)"))
        .rejects(Style("filter: grayscale(-.5)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.hue-rotate", K::Value, "I-FILTER1", "#funcdef-filter-hue-rotate")
        .spelled("hue-rotate()")
        .accepts(Style("filter: hue-rotate(45deg)"))
        .rejects(Style("filter: hue-rotate(45%)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.invert", K::Value, "I-FILTER1", "#funcdef-filter-invert")
        .spelled("invert()")
        .accepts(Style("filter: invert(10%)"))
        .rejects(Style("filter: invert(10px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.opacity", K::Value, "I-FILTER1", "#funcdef-filter-opacity")
        .spelled("opacity()")
        .accepts(Style("filter: opacity(.75)"))
        .rejects(Style("filter: opacity(-1)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.saturate", K::Value, "I-FILTER1", "#funcdef-filter-saturate")
        .spelled("saturate()")
        .accepts(Style("filter: saturate(2)"))
        .rejects(Style("filter: saturate(2deg)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.sepia", K::Value, "I-FILTER1", "#funcdef-filter-sepia")
        .spelled("sepia()")
        .accepts(Style("filter: sepia(30%)"))
        .rejects(Style("filter: sepia(-30%)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.filter.drop-shadow", K::Value, "I-FILTER1", "#funcdef-filter-drop-shadow")
        .spelled("drop-shadow()")
        .accepts(Style("filter: drop-shadow(red 1px 2px 3px)"))
        .rejects(Style("filter: drop-shadow(inset 1px 2px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape", K::Value, "S-SHAPES1", "#typedef-basic-shape")
        .spelled("<basic-shape>")
        .accepts(Style("clip-path: polygon(0 0, 100% 0)"))
        .rejects(Style("clip-path: triangle(0 0)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.shape", K::Value, "S-SHAPES1", "#funcdef-basic-shape-shape")
        .spelled("shape()")
        .accepts(Style("clip-path: shape(from 0 0, line to 1px 1px)"))
        .rejects(Style("clip-path: shape(line to 1px 1px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.inset", K::Value, "S-SHAPES1", "#funcdef-basic-shape-inset")
        .spelled("inset()")
        .accepts(Style("clip-path: inset(1px round 2px)"))
        .rejects(Style("clip-path: inset(1deg)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.circle", K::Value, "S-SHAPES1", "#funcdef-basic-shape-circle")
        .spelled("circle()")
        .accepts(Style("clip-path: circle(10px at center)"))
        .rejects(Style("clip-path: circle(10px 20px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.ellipse", K::Value, "S-SHAPES1", "#funcdef-basic-shape-ellipse")
        .spelled("ellipse()")
        .accepts(Style("clip-path: ellipse(10px 20% at left top)"))
        .rejects(Style("clip-path: ellipse(10px 20px 30px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.polygon", K::Value, "S-SHAPES1", "#funcdef-basic-shape-polygon")
        .spelled("polygon()")
        .accepts(Style("clip-path: polygon(evenodd round 2px, 0 0, 100% 0)"))
        .rejects(Style("clip-path: polygon(0 0 0)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.path", K::Value, "S-SHAPES1", "#funcdef-basic-shape-path")
        .spelled("path()")
        .accepts(Style("clip-path: path('M 0 0 L 1 1')"))
        .rejects(Style("clip-path: path(M 0 0)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.rect", K::Value, "S-SHAPES1", "#funcdef-basic-shape-rect")
        .spelled("rect()")
        .accepts(Style("clip-path: rect(0 1px 1px 0)"))
        .rejects(Style("clip-path: rect(0 1px 1px)"), CssErrorCode::InvalidPropertyValue),
    complete("ext.value.basic-shape.xywh", K::Value, "S-SHAPES1", "#funcdef-basic-shape-xywh")
        .spelled("xywh()")
        .accepts(Style("clip-path: xywh(0 0 1px 1px)"))
        .rejects(Style("clip-path: xywh(0 0 1px)"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.calc", K::Value, "O-VALUES3", "#calc-notation,#calc-syntax,#calc-type-checking")
        .spelled("calc()")
        .accepts(Style("width: calc((1px + 2%) * 3)"))
        .rejects(Style("width: calc(1px +)"), CssErrorCode::InvalidPropertyValue),
    complete("later.rule.namespace", K::Rule, "O-NAMESPACES3", "#terminology,#declaration,#syntax,#scope,#prefixes")
        .spelled("@namespace")
        .accepts(Sheet("@namespace svg url(https://example.test/svg);")),
    complete("later.rule.supports", K::Rule, "O-CONDITIONAL3", "#at-supports")
        .accepts(Sheet("@supports (display: grid) and future-layout(mode) { .x { color: red; } }")),
    complete("ext.rule.supports-condition", K::Rule, "X-CONDITIONAL5", "#at-ruledef-supports-condition")
        .spelled("@supports-condition")
        .accepts(Sheet("@supports-condition --feature { future: value; }")),
    complete("later.rule.counter-style", K::Rule, "O-COUNTERSTYLES3", "#the-counter-style-rule")
        .spelled("@counter-style")
        .accepts(Sheet("@counter-style thumbs { system: cyclic; symbols: 👍; suffix: \" \"; }")),
    complete("official.descriptor.counter-style.system", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-system")
        .accepts(Sheet("@counter-style dots { system: fixed 3; symbols: a b; }"))
        .rejects(Sheet("@counter-style dots { system: fixed 3 4; symbols: a b; }"), CssErrorCode::InvalidDescriptorValue),
    complete("official.descriptor.counter-style.negative", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-negative")
        .accepts(Sheet("@counter-style dots { system: numeric; symbols: \"0\" \"1\"; negative: \"(\" \")\"; }")),
    complete("official.descriptor.counter-style.prefix", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-prefix")
        .accepts(Sheet("@counter-style dots { system: cyclic; symbols: a; prefix: \"[\"; }")),
    complete("official.descriptor.counter-style.suffix", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-suffix")
        .accepts(Sheet("@counter-style dots { system: cyclic; symbols: a; suffix: \"] \"; }")),
    complete("official.descriptor.counter-style.range", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-range")
        .accepts(Sheet("@counter-style dots { system: cyclic; symbols: a; range: 1 10, 20 infinite; }"))
        .rejects(Sheet("@counter-style dots { system: cyclic; symbols: a; range: 10 1; }"), CssErrorCode::InvalidDescriptorValue),
    complete("official.descriptor.counter-style.pad", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-pad")
        .accepts(Sheet("@counter-style dots { system: numeric; symbols: \"0\" \"1\"; pad: 3 \"0\"; }")),
    complete("official.descriptor.counter-style.fallback", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-fallback")
        .accepts(Sheet("@counter-style dots { system: cyclic; symbols: a; fallback: decimal; }")),
    complete("official.descriptor.counter-style.symbols", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-symbols")
        .accepts(Sheet("@counter-style dots { system: alphabetic; symbols: a b c; }")),
    complete("official.descriptor.counter-style.additive-symbols", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-symbols")
        .accepts(Sheet("@counter-style roman { system: additive; additive-symbols: 10 X, 5 V, 1 I; }")),
    complete("official.descriptor.counter-style.speak-as", K::Descriptor, "O-COUNTERSTYLES3", "#counter-style-speak-as")
        .accepts(Sheet("@counter-style dots { system: cyclic; symbols: a; speak-as: bullets; }")),
    complete("official.value.counter-style", K::Value, "O-COUNTERSTYLES3", "#the-counter-style-rule")
        .spelled("<counter-style>")
        .accepts(Style("list-style-type: symbols(cyclic \"*\")")),
    complete("official.value.counter-style-name", K::Value, "O-COUNTERSTYLES3", "#the-counter-style-rule")
        .spelled("<counter-style-name>")
        .accepts(Style("list-style-type: lower-roman"))
        .rejects(Style("list-style-type: none none"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.symbol", K::Value, "O-COUNTERSTYLES3", "#counter-style-symbols")
        .spelled("<symbol>")
        .accepts(Sheet("@counter-style thumbs { system: cyclic; symbols: \"👍\" url(thumb.png); }")),
    complete("official.value.symbols-function", K::Value, "O-COUNTERSTYLES3", "#symbols-function")
        .spelled("symbols()")
        .accepts(Style("list-style-type: symbols(numeric \"0\" \"1\")"))
        .rejects(Style("list-style-type: symbols(numeric \"0\")"), CssErrorCode::InvalidPropertyValue),
    complete("official.value.symbols-type", K::Value, "O-COUNTERSTYLES3", "#symbols-function")
        .spelled("cyclic|numeric|alphabetic|symbolic|fixed")
        .accepts(Style("list-style-type: symbols(alphabetic \"a\" \"b\")")),
    complete("later.rule.page", K::Rule, "L-PAGE3-20181018", "#syntax-page-selector")
        .spelled("@page")
        .accepts(Sheet("@page { margin: 1cm; }")),
    complete("official.selector.page-pseudo", K::Selector, "L-PAGE3-20181018", "#page-selectors")
        .spelled(":left|:right|:first|:blank")
        .accepts(Sheet("@page :left { margin: 1cm; }")),
    complete("official.selector.logical-page-pseudo", K::Selector, "I-LOGICAL1-20251204", "#page")
        .spelled(":recto|:verso")
        .accepts(Sheet("@page :recto { margin: 1cm; }")),
    // Empty definitions remain invalid under the selected authored grammar.
    complete("later.rule.font-feature-values", K::Rule, "I-FONTS4-20260907", "#font-feature-values-syntax")
        .spelled("@font-feature-values")
        .accepts(Sheet("@font-feature-values Font One { @styleset { nice: 1; } }")),
    complete("later.rule.font-palette-values", K::Rule, "I-FONTS4-20260907", "#font-palette-values")
        .spelled("@font-palette-values")
        .accepts(Sheet("@font-palette-values --brand { font-family: Demo; }")),
    complete("later.descriptor.font-palette-values.font-family", K::Descriptor, "I-FONTS4-20260907", "#font-family-2-desc")
        .accepts(Sheet("@font-palette-values --brand { font-family: Demo, \"serif\"; }")),
    complete("later.descriptor.font-palette-values.base-palette", K::Descriptor, "I-FONTS4-20260907", "#base-palette-desc")
        .accepts(Sheet("@font-palette-values --brand { font-family: Demo; base-palette: dark; }")),
    complete("later.descriptor.font-palette-values.override-colors", K::Descriptor, "I-FONTS4-20260907", "#override-color")
        .accepts(Sheet("@font-palette-values --brand { font-family: Demo; override-colors: 0 red; }")),
    complete("baseline.descriptor.font-family", K::Descriptor, "I-FONTS4-20260907", "#font-family-desc")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2); }")),
    complete("baseline.descriptor.src", K::Descriptor, "I-FONTS4-20260907", "#font-face-src-parsing")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2) format(\"woff2\"); }")),
    complete("baseline.descriptor.font-weight", K::Descriptor, "I-FONTS4-20260907", "#font-prop-desc")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2); font-weight: 700; }")),
    complete("baseline.descriptor.font-style", K::Descriptor, "I-FONTS4-20260907", "#font-prop-desc")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2); font-style: italic; }")),
    complete("baseline.descriptor.font-stretch", K::Descriptor, "I-FONTS4-20260907", "#font-prop-desc")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2); font-width: condensed; }")),
    complete("baseline.descriptor.font-display", K::Descriptor, "I-FONTS4", "#font-display-desc")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2); font-display: swap; }")),
    complete("baseline.descriptor.unicode-range", K::Descriptor, "O-FONTS3", "#unicode-range-desc")
        .accepts(Sheet("@font-face { font-family: Inter; src: url(inter.woff2); unicode-range: U+0000-00FF; }")),
    complete("official.descriptor.font-feature-settings", K::Descriptor, "I-FONTS4-20260907", "#font-rend-desc")
        .accepts(Sheet("@font-face { font-family: Demo; src: url(demo.woff2); font-feature-settings: \"kern\" on; }")),
    complete("official.descriptor.font-named-instance", K::Descriptor, "I-FONTS4-20260907", "#font-named-instance")
        .accepts(Sheet("@font-face { font-named-instance: \"Grotesque\"; }")),
    complete("official.descriptor.font-language-override", K::Descriptor, "I-FONTS4-20260907", "#font-language-override-desc")
        .accepts(Sheet("@font-face { font-language-override: \"TRK\"; }")),
    complete("official.descriptor.ascent-override", K::Descriptor, "I-FONTS4-20260907", "#descdef-font-face-ascent-override")
        .accepts(Sheet("@font-face { ascent-override: 125%; }")),
    complete("official.descriptor.descent-override", K::Descriptor, "I-FONTS4-20260907", "#descdef-font-face-descent-override")
        .accepts(Sheet("@font-face { descent-override: 25%; }")),
    complete("official.descriptor.line-gap-override", K::Descriptor, "I-FONTS4-20260907", "#descdef-font-face-line-gap-override")
        .accepts(Sheet("@font-face { line-gap-override: 0%; }")),
    complete("official.value.opentype-tag", K::Value, "I-FONTS4-20260907", "#font-rend-desc")
        .accepts(Style("font-feature-settings: \"liga\" 0"))
        .rejects(Style("font-feature-settings: \"lig\" 0"), CssErrorCode::InvalidPropertyValue),
    complete("ext.descriptor.font-weight-range", K::Descriptor, "I-FONTS4-20260907", "#font-prop-desc")
        .accepts(Sheet("@font-face { font-weight: bold 300; }")),
    complete("ext.descriptor.font-style-oblique-range", K::Descriptor, "I-FONTS4-20260907", "#font-prop-desc")
        .accepts(Sheet("@font-face { font-family: Demo; src: url(demo.woff2); font-style: oblique -10deg 20deg; }"))
        .rejects(Sheet("@font-face { font-family: Demo; src: url(demo.woff2); font-style: oblique 91deg; }"), CssErrorCode::InvalidDescriptorValue),
    complete("ext.value.font-source-modern-hints", K::Value, "I-FONTS4-20260907", "#font-face-src-parsing")
        .accepts(Sheet("@font-face { font-family: Demo; src: url(demo.woff2) format(woff2) tech(variations, color-colrv1); }"))
        .rejects(Sheet("@font-face { font-family: Demo; src: url(demo.woff2) format(woff3) tech(unknown); }"), CssErrorCode::InvalidDescriptorValue),
    partial("baseline.selector.complex", K::Selector, "O-SELECTORS3", "#type-selectors,#universal-selector,#attribute-representation,#attribute-substrings,#class-html,#id-selectors,#descendant-combinators,#child-combinators,#adjacent-sibling-combinators,#general-sibling-combinators")
        .accepts(Sheet("article#main.card[data-ready][lang|=\"en\"] > span + a ~ b { color: red; }"))
        .rejects(Sheet("svg|a { color: red; }"), CssErrorCode::InvalidSelector),
    partial("baseline.selector.pseudo-class", K::Selector, "O-SELECTORS3", "#dynamic-pseudos,#UIstates,#structural-pseudos")
        .accepts(Sheet(".button:hover { color: red; }"))
        .rejects(Sheet(".host:host() { color: red; }"), CssErrorCode::InvalidSelector),
    partial("baseline.selector.functional", K::Selector, "O-SELECTORS3", "#structural-pseudos,#negation")
        .accepts(Sheet(".item:nth-child(2n+1) { color: red; }"))
        .rejects(Sheet(".item:dir(1) { color: red; }"), CssErrorCode::InvalidSelector),
    partial("baseline.selector.extension-state", K::Selector, "I-SELECTORS4", "#useraction-pseudos,#input-pseudos,#resource-pseudos,#display-state-pseudos,#the-defined-pseudo,#the-any-link-pseudo,#video-state,#media-loading-state,#sound-state,#open-state,#pip-state,#checked,#user-pseudos")
        .accepts(Sheet(".button:focus-visible { color: red; }"))
        .rejects(Sheet(".link:local-link { color: red; }"), CssErrorCode::InvalidSelector),
    partial("baseline.selector.extension-functional", K::Selector, "I-SELECTORS4", "#matches,#zero-matches,#relational,#negation,#the-nth-child-pseudo")
        .accepts(Sheet(".item:is(.primary, .secondary) { color: red; }"))
        .rejects(Sheet(".item:has(:has(.nested)) { color: red; }"), CssErrorCode::InvalidSelector),
    partial("baseline.selector.attribute-case", K::Selector, "I-SELECTORS4", "#attribute-case")
        .accepts(Sheet("[data-kind=\"primary\" i] { color: red; }"))
        .rejects(Sheet("[data-kind=\"primary\" q] { color: red; }"), CssErrorCode::InvalidSelector),
    partial("baseline.selector.nesting", K::Selector, "I-NESTING1", "#nest-selector,#syntax-examples,#nesting-at-scope")
        .accepts(Sheet(".card { & > .title { color: red; } }"))
        .rejects(Sheet(".card { & || ! { color: red; } }"), CssErrorCode::InvalidSelector),
    complete("baseline.media.type", K::MediaQuery, "X-MEDIA5", "#media-types")
        .accepts(Sheet("@media speech { .x { color: red; } }")),
    partial("baseline.container.condition", K::ContainerQuery, "X-CONDITIONAL5", "#container-rule")
        .accepts(Sheet("@container (width > 1px) and style(--theme) { .x { color: red; } }"))
        .rejects(Sheet("@container style(color: red) and { .x { color: red; } }"), CssErrorCode::InvalidAtRulePrelude),
    partial("baseline.container.size-feature", K::ContainerQuery, "X-CONDITIONAL5", "#size-container")
        .accepts(Sheet("@container (inline-size > 1px) { .x { color: red; } }"))
        .rejects(Sheet("@container (unknown-size > 1px) or { .x { color: red; } }"), CssErrorCode::InvalidAtRulePrelude),
    complete("official.property.flex-flow", K::Property, "O-FLEXBOX1", "#propdef-flex-flow")
        .spelled("flex-flow")
        .accepts(Style("flex-flow: column wrap")),
    complete("official.property.column-count", K::Property, "O-MULTICOL1", "#propdef-column-count")
        .spelled("column-count")
        .accepts(Style("column-count: 3"))
        .rejects(Style("column-count: 0"), CssErrorCode::InvalidPropertyValue),
    complete("official.property.column-fill", K::Property, "O-MULTICOL1", "#propdef-column-fill")
        .spelled("column-fill")
        .accepts(Style("column-fill: balance")),
    complete("official.property.column-rule", K::Property, "O-MULTICOL1", "#propdef-column-rule")
        .spelled("column-rule")
        .accepts(Style("column-rule: thin dotted red")),
    complete("official.property.column-rule-color", K::Property, "O-MULTICOL1", "#propdef-column-rule-color")
        .spelled("column-rule-color")
        .accepts(Style("column-rule-color: red")),
    complete("official.property.column-rule-style", K::Property, "O-MULTICOL1", "#propdef-column-rule-style")
        .spelled("column-rule-style")
        .accepts(Style("column-rule-style: dashed")),
    complete("official.property.column-rule-width", K::Property, "O-MULTICOL1", "#propdef-column-rule-width")
        .spelled("column-rule-width")
        .accepts(Style("column-rule-width: 2px")),
    complete("official.property.column-span", K::Property, "O-MULTICOL1", "#propdef-column-span")
        .spelled("column-span")
        .accepts(Style("column-span: all")),
    complete("official.property.column-width", K::Property, "O-MULTICOL1", "#propdef-column-width")
        .spelled("column-width")
        .accepts(Style("column-width: 12em")),
    complete("official.property.columns", K::Property, "O-MULTICOL1", "#propdef-columns")
        .spelled("columns")
        .accepts(Style("columns: 3 12em")),
    complete("baseline.property.grid-template-rows", K::Property, "R-GRID2", "#propdef-grid-template-rows")
        .spelled("grid-template-rows")
        .accepts(Style("grid-template-rows: 10px auto")),
    complete("baseline.property.grid-template-columns", K::Property, "R-GRID2", "#propdef-grid-template-columns")
        .spelled("grid-template-columns")
        .accepts(Style("grid-template-columns: repeat(auto-fit, 10px)")),
    complete("baseline.property.grid-template", K::Property, "R-GRID2", "#propdef-grid-template")
        .spelled("grid-template")
        .accepts(Style("grid-template: \"a b\" 10px / 1fr 2fr")),
    complete("baseline.property.grid-auto-rows", K::Property, "R-GRID2", "#propdef-grid-auto-rows")
        .spelled("grid-auto-rows")
        .accepts(Style("grid-auto-rows: minmax(10px, auto)")),
    complete("baseline.property.grid-auto-columns", K::Property, "R-GRID2", "#propdef-grid-auto-columns")
        .spelled("grid-auto-columns")
        .accepts(Style("grid-auto-columns: 1fr")),
    complete("baseline.property.grid", K::Property, "R-GRID2", "#propdef-grid")
        .spelled("grid")
        .accepts(Style("grid: auto-flow dense / 10px")),
    complete("baseline.property.font-weight", K::Property, "I-FONTS4-20260907", "#propdef-font-weight")
        .spelled("font-weight")
        .accepts(Style("font-weight: 725.25"))
        .rejects(Style("font-weight: 1001"), CssErrorCode::InvalidPropertyValue),
    complete("baseline.property.font-style", K::Property, "I-FONTS4-20260907", "#propdef-font-style")
        .spelled("font-style")
        .accepts(Style("font-style: oblique 10deg")),
    complete("baseline.property.font-variant", K::Property, "I-FONTS4-20260907", "#propdef-font-variant")
        .spelled("font-variant")
        .accepts(Style("font-variant: small-caps oldstyle-nums")),
    complete("official.property.font-variant-alternates", K::Property, "I-FONTS4-20260907", "#propdef-font-variant-alternates")
        .spelled("font-variant-alternates")
        .accepts(Style("font-variant-alternates: styleset(Alpha, beta)")),
    complete("official.property.font-variant-caps", K::Property, "I-FONTS4-20260907", "#propdef-font-variant-caps")
        .spelled("font-variant-caps")
        .accepts(Style("font-variant-caps: all-small-caps")),
    complete("official.property.font-variant-east-asian", K::Property, "I-FONTS4-20260907", "#propdef-font-variant-east-asian")
        .spelled("font-variant-east-asian")
        .accepts(Style("font-variant-east-asian: jis04 ruby")),
    complete("official.property.font-variant-emoji", K::Property, "I-FONTS4-20260907", "#propdef-font-variant-emoji")
        .spelled("font-variant-emoji")
        .accepts(Style("font-variant-emoji: emoji")),
    complete("official.property.font-variant-ligatures", K::Property, "I-FONTS4-20260907", "#propdef-font-variant-ligatures")
        .spelled("font-variant-ligatures")
        .accepts(Style("font-variant-ligatures: common-ligatures no-discretionary-ligatures")),
    complete("official.property.font-variant-numeric", K::Property, "I-FONTS4-20260907", "#propdef-font-variant-numeric")
        .spelled("font-variant-numeric")
        .accepts(Style("font-variant-numeric: lining-nums tabular-nums slashed-zero")),
    complete("official.property.font-variant-position", K::Property, "I-FONTS4-20260907", "#propdef-font-variant-position")
        .spelled("font-variant-position")
        .accepts(Style("font-variant-position: super")),
    complete("baseline.property.font-feature-settings", K::Property, "I-FONTS4-20260907", "#propdef-font-feature-settings")
        .spelled("font-feature-settings")
        .accepts(Style("font-feature-settings: \"kern\" on, \"liga\" 0")),
    complete("official.property.font-kerning", K::Property, "I-FONTS4-20260907", "#propdef-font-kerning")
        .spelled("font-kerning")
        .accepts(Style("font-kerning: normal")),
    complete("official.property.font-size-adjust", K::Property, "I-FONTS4-20260907", "#propdef-font-size-adjust")
        .spelled("font-size-adjust")
        .accepts(Style("font-size-adjust: 0.5")),
    complete("official.property.font-synthesis", K::Property, "I-FONTS4-20260907", "#propdef-font-synthesis")
        .spelled("font-synthesis")
        .accepts(Style("font-synthesis: weight style")),
    complete("official.property.font-palette", K::Property, "I-FONTS4-20260907", "#propdef-font-palette")
        .spelled("font-palette")
        .accepts(Style("font-palette: palette-mix(light, dark)")),
    complete("ext.property.font-weight-range", K::Property, "I-FONTS4-20260907", "#font-weight-absolute-values")
        .accepts(Style("font-weight: 1"))
        .rejects(Style("font-weight: 0"), CssErrorCode::InvalidPropertyValue),
    complete("baseline.property.z-index", K::Property, "O-CSS2", "visuren.html#propdef-z-index")
        .spelled("z-index")
        .accepts(Style("z-index: auto")),
    complete("baseline.property.background-position", K::Property, "O-BACKGROUNDS3", "#propdef-background-position")
        .spelled("background-position")
        .accepts(Style("background-position: left top, right 4px bottom 8%")),
    complete("official.property.object-position", K::Property, "O-IMAGES3", "#propdef-object-position")
        .spelled("object-position")
        .accepts(Style("object-position: center 25%")),
    complete("baseline.property.box-shadow", K::Property, "O-BACKGROUNDS3", "#propdef-box-shadow")
        .spelled("box-shadow")
        .accepts(Style("box-shadow: 1px 2px 3px red")),
    complete("baseline.property.transform", K::Property, "O-TRANSFORMS1", "#propdef-transform")
        .spelled("transform")
        .accepts(Style("transform: translate(1px, 2%)")),
    complete("baseline.property.transform-origin", K::Property, "O-TRANSFORMS1", "#propdef-transform-origin")
        .spelled("transform-origin")
        .accepts(Style("transform-origin: left top 50px"))
        .rejects(Style("transform-origin: left top 50%"), CssErrorCode::InvalidPropertyValue),
    complete("baseline.property.filter", K::Property, "I-FILTER1", "#propdef-filter")
        .spelled("filter")
        .accepts(Style("filter: blur(2px)")),
    complete("baseline.property.backdrop-filter", K::Property, "X-BACKDROP-FILTER", "#BackdropFilterProperty")
        .spelled("backdrop-filter")
        .accepts(Style("backdrop-filter: blur(2px) opacity(50%)")),
    complete("baseline.property.clip-path", K::Property, "S-MASKING1", "#propdef-clip-path")
        .spelled("clip-path")
        .accepts(Style("clip-path: border-box circle(10px)")),
    complete("baseline.property.mask-position", K::Property, "S-MASKING1", "#propdef-mask-position")
        .spelled("mask-position")
        .accepts(Style("mask-position: left 10px bottom 20%, center 25%")),
    complete("baseline.property.transition-duration", K::Property, "I-TRANSITIONS1", "#propdef-transition-duration")
        .accepts(Style("transition-duration: calc((1s + 250ms) * 2)")),
    complete("baseline.property.transition-delay", K::Property, "I-TRANSITIONS1", "#propdef-transition-delay")
        .accepts(Style("transition-delay: -250ms")),
    complete("baseline.property.transition-timing-function", K::Property, "I-TRANSITIONS1", "#propdef-transition-timing-function")
        .spelled("transition-timing-function")
        .accepts(Style("transition-timing-function: cubic-bezier(.25, 0, .75, 1)")),
    complete("baseline.property.transition", K::Property, "I-TRANSITIONS1", "#propdef-transition")
        .accepts(Style("transition: opacity calc(1s + 250ms) linear -200ms")),
    complete("baseline.property.animation-duration", K::Property, "I-ANIMATIONS1", "#propdef-animation-duration")
        .accepts(Style("animation-duration: calc(1s + 250ms)")),
    complete("baseline.property.animation-delay", K::Property, "I-ANIMATIONS1", "#propdef-animation-delay")
        .accepts(Style("animation-delay: -1s")),
    complete("baseline.property.animation-timing-function", K::Property, "I-ANIMATIONS1", "#propdef-animation-timing-function")
        .spelled("animation-timing-function")
        .accepts(Style("animation-timing-function: steps(2, jump-none)")),
    complete("baseline.property.animation-iteration-count", K::Property, "I-ANIMATIONS1", "#propdef-animation-iteration-count")
        .accepts(Style("animation-iteration-count: calc((1 + 2) * 3)")),
    complete("baseline.property.animation", K::Property, "I-ANIMATIONS1", "#propdef-animation")
        .accepts(Style("animation: fade calc(1s + 250ms) linear calc((1 + 2) * 3) -200ms both running")),
    complete("official.selector.group", K::Selector, "O-SELECTORS3", "#grouping")
        .accepts(Sheet("h1, h2 { color: red; }")),
    complete("official.selector.type", K::Selector, "O-SELECTORS3", "#type-selectors")
        .accepts(Sheet("article { color: red; }")),
    complete("official.selector.universal", K::Selector, "O-SELECTORS3", "#universal-selector")
        .accepts(Sheet("* { color: red; }")),
    complete("official.selector.attribute-presence-value", K::Selector, "O-SELECTORS3", "#attribute-representation")
        .accepts(Sheet("[data-ready][lang|=\"en\"] { color: red; }")),
    complete("official.selector.attribute-substring", K::Selector, "O-SELECTORS3", "#attribute-substrings")
        .accepts(Sheet("[title^=\"lead\"][href$=\".pdf\"][class*=\"x\"] { color: red; }")),
    complete("official.selector.class", K::Selector, "O-SELECTORS3", "#class-html")
        .accepts(Sheet(".card { color: red; }")),
    complete("official.selector.id", K::Selector, "O-SELECTORS3", "#id-selectors")
        .accepts(Sheet("#first#second { color: red; }")),
    complete("official.selector.dynamic", K::Selector, "O-SELECTORS3", "#dynamic-pseudos")
        .accepts(Sheet(":link, :visited, :hover, :active, :focus { color: red; }")),
    complete("official.selector.target", K::Selector, "O-SELECTORS3", "#target-pseudo")
        .accepts(Sheet(":target { color: red; }")),
    complete("official.selector.lang", K::Selector, "I-SELECTORS4", "#the-lang-pseudo")
        .accepts(Sheet(":lang(en) { color: red; }"))
        .rejects(Sheet(":lang(en en) { color: red; }"), CssErrorCode::InvalidSelector),
    complete("official.selector.ui-state", K::Selector, "O-SELECTORS3", "#UIstates")
        .accepts(Sheet(":enabled, :disabled, :checked { color: red; }")),
    complete("official.selector.structural", K::Selector, "O-SELECTORS3", "#structural-pseudos")
        .accepts(Sheet(":root, :nth-last-of-type(even), :empty { color: red; }")),
    complete("official.selector.negation", K::Selector, "O-SELECTORS3", "#negation")
        .accepts(Sheet(":not(.excluded) { color: red; }")),
    complete("official.selector.first-line", K::Selector, "O-SELECTORS3", "#first-line")
        .accepts(Sheet(".line::first-line { color: red; }")),
    complete("official.selector.first-letter", K::Selector, "O-SELECTORS3", "#first-letter")
        .accepts(Sheet(".letter:first-letter { color: red; }")),
    complete("official.selector.generated", K::Selector, "O-SELECTORS3", "#gen-content")
        .accepts(Sheet(".x::after, .y:before { color: red; }")),
    complete("official.selector.combinator.descendant", K::Selector, "O-SELECTORS3", "#descendant-combinators")
        .accepts(Sheet("section a { color: red; }")),
    complete("official.selector.combinator.child", K::Selector, "O-SELECTORS3", "#child-combinators")
        .accepts(Sheet("section > a { color: red; }")),
    complete("official.selector.combinator.next-sibling", K::Selector, "O-SELECTORS3", "#adjacent-sibling-combinators")
        .accepts(Sheet("section + a { color: red; }")),
    complete("official.selector.combinator.subsequent-sibling", K::Selector, "O-SELECTORS3", "#general-sibling-combinators")
        .accepts(Sheet("section ~ p { color: red; }")),
    complete("official.selector.namespace-qualified-name", K::Selector, "O-NAMESPACES3", "#scope,#prefixes,#css-qnames")
        .accepts(Sheet("@namespace svg \"urn:svg\"; svg|a, *|a, |a, [svg|href] { color: red; }")),
    complete("ext.pseudo-element.prefix", K::Selector, "X-PSEUDO4", "#selectordef-first-letter-prefix")
        .accepts(Sheet("p::first-letter::prefix { color: red; }")),
    complete("ext.pseudo-element.suffix", K::Selector, "X-PSEUDO4", "#selectordef-first-letter-suffix")
        .accepts(Sheet("p::first-letter::suffix { color: red; }")),
    complete("ext.pseudo-element.search-text", K::Selector, "X-PSEUDO4", "#selectordef-search-text")
        .accepts(Sheet(".x::search-text { color: red; }")),
    complete("ext.pseudo-element.target-text", K::Selector, "X-PSEUDO4", "#selectordef-target-text")
        .spelled("::target-text")
        .accepts(Sheet(".x::target-text { color: red; }")),
    complete("ext.pseudo-element.spelling-error", K::Selector, "X-PSEUDO4", "#selectordef-spelling-error")
        .spelled("::spelling-error")
        .accepts(Sheet(".x::spelling-error { color: red; }")),
    complete("ext.pseudo-element.grammar-error", K::Selector, "X-PSEUDO4", "#selectordef-grammar-error")
        .spelled("::grammar-error")
        .accepts(Sheet(".x::grammar-error { color: red; }")),
    complete("ext.pseudo-element.highlight", K::Selector, "X-PSEUDO4", "#selectordef-highlight-custom-ident")
        .accepts(Sheet(".x::highlight(MyName) { color: red; }")),
    complete("ext.pseudo-element.placeholder", K::Selector, "X-PSEUDO4", "#selectordef-placeholder")
        .accepts(Sheet("input::placeholder { color: red; }")),
    complete("ext.pseudo-element.file-selector-button", K::Selector, "X-PSEUDO4", "#selectordef-file-selector-button,#element-backed")
        .accepts(Sheet("input::file-selector-button:first-child { color: red; }")),
    complete("ext.pseudo-element.details-content", K::Selector, "X-PSEUDO4", "#selectordef-details-content,#element-backed")
        .accepts(Sheet("details::details-content::before { color: red; }")),
    complete("ext.pseudo-element.search-text-current", K::Selector, "X-PSEUDO4", "#selectordef-search-text,#element-backed")
        .accepts(Sheet(".x::search-text:current { color: red; }")),
    complete("official.media.query-list-core", K::MediaQuery, "X-MEDIA5", "#mq-syntax")
        .accepts(Sheet("@media screen and (min-width: 1px), print { .x { color: red; } }"))
        .rejects(Sheet("@media screen, ??? { .x { color: red; } }"), CssErrorCode::InvalidMediaQuery),
    complete("official.media.feature.width", K::MediaQuery, "X-MEDIA5", "#width")
        .accepts(Sheet("@media (min-width: 1px) { .x { color: red; } }")),
    complete("official.media.feature.height", K::MediaQuery, "X-MEDIA5", "#height")
        .accepts(Sheet("@media (max-height: 2px) { .x { color: red; } }")),
    complete("official.media.feature.device-width", K::MediaQuery, "X-MEDIA5", "#device-width")
        .accepts(Sheet("@media (device-width: 800px) { .x { color: red; } }")),
    complete("official.media.feature.device-height", K::MediaQuery, "X-MEDIA5", "#device-height")
        .accepts(Sheet("@media (max-device-height: 800px) { .x { color: red; } }")),
    complete("official.media.feature.aspect-ratio", K::MediaQuery, "X-MEDIA5", "#aspect-ratio")
        .accepts(Sheet("@media (min-aspect-ratio: 16/9) { .x { color: red; } }")),
    complete("official.media.feature.device-aspect-ratio", K::MediaQuery, "X-MEDIA5", "#device-aspect-ratio")
        .accepts(Sheet("@media (device-aspect-ratio: 16/9) { .x { color: red; } }")),
    complete("official.media.feature.resolution", K::MediaQuery, "X-MEDIA5", "#resolution")
        .accepts(Sheet("@media (resolution: 192dpi) { .x { color: red; } }")),
    complete("official.media.feature.color", K::MediaQuery, "X-MEDIA5", "#color")
        .accepts(Sheet("@media (color: 8) { .x { color: red; } }")),
    complete("official.media.feature.color-index", K::MediaQuery, "X-MEDIA5", "#color-index")
        .accepts(Sheet("@media (min-color-index: 256) { .x { color: red; } }")),
    complete("official.media.feature.monochrome", K::MediaQuery, "X-MEDIA5", "#monochrome")
        .accepts(Sheet("@media (monochrome: 1) { .x { color: red; } }")),
    complete("official.media.feature.scan", K::MediaQuery, "X-MEDIA5", "#scan")
        .accepts(Sheet("@media (scan: progressive) { .x { color: red; } }")),
    complete("official.media.feature.grid", K::MediaQuery, "X-MEDIA5", "#grid")
        .accepts(Sheet("@media (grid: 0) { .x { color: red; } }")),
    complete("ext.media.resolution.dppx", K::MediaQuery, "X-MEDIA5", "#resolution")
        .accepts(Sheet("@media (resolution: 2dppx) { .x { color: red; } }")),
    complete("ext.supports.general-enclosed", K::Value, "X-VALUES4", "css-values-4/Overview.bs#typedef-general-enclosed")
        .accepts(Sheet("@supports future-layout(mode) { .x { color: red; } }")),
    complete("ext.supports.named", K::Value, "X-CONDITIONAL5", "#typedef-supports-decl,#typedef-supports-condition-name")
        .accepts(Sheet("@supports (--feature) { .x { color: red; } }")),
    partial("ext.supports.selector", K::Selector, "R-CONDITIONAL4", "#typedef-supports-selector-fn")
        .accepts(Sheet("@supports selector(.card:is(.primary, .secondary)) {}"))
        .rejects(Sheet("@supports selector(.x) and { .y { color: red; } }"), CssErrorCode::InvalidAtRulePrelude),
    complete("official.media.feature.orientation", K::MediaQuery, "X-MEDIA5", "#orientation")
        .accepts(Sheet("@media (orientation: landscape) { .x { color: red; } }")),
];
