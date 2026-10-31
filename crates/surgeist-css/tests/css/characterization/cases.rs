//! The characterization case table.
//!
//! Every case names one front by its stable front id, an argument (`-` when
//! the front takes none) and an input. Case ids are stable: a later API step
//! keeps every id and edits only the adapter. Inputs are handwritten stimuli,
//! not expected results; the golden file records what the code does with them.

use crate::css::support::property_expectations::{self, CatalogExpectation};

/// The colour whose `1e308` literal needs the most actions of the corpus. The
/// fixed colour action ceiling of the corpus is derived from its requirement,
/// so that it is refused with an action limit under the fixed limits and
/// reaches its numerical outcome under the large allowance.
pub const NUMERICAL_OVERFLOW_INPUT: &str = "color-mix(in srgb, lab(0 1e308 0), red)";

/// One observation request.
pub struct Case {
    pub id: String,
    pub front: String,
    pub feature: String,
    pub argument: String,
    pub input: String,
}

impl Case {
    fn new(id: String, front: &str, argument: &str, input: &str) -> Self {
        Self {
            id,
            front: front.to_owned(),
            feature: "-".to_owned(),
            argument: argument.to_owned(),
            input: input.to_owned(),
        }
    }
}

/// Every case, in table order. The golden file sorts them by id.
pub fn all() -> Vec<Case> {
    let mut cases = report_cases();
    cases.extend(FRONT_CASES.iter().map(|(front, argument, slug, input)| {
        Case::new(format!("front.{front}.{slug}"), front, argument, input)
    }));
    cases.extend(property_cases());
    cases.extend(color_cases());
    for front in ["color.used.large-actions", "color.authored.large-actions"] {
        let suffix = front.strip_prefix("color.").unwrap();
        cases.push(Case::new(
            format!("color.numerical-mix-overflow.absent.{suffix}"),
            front,
            "absent",
            NUMERICAL_OVERFLOW_INPUT,
        ));
    }
    cases.extend(COLOR_LIMIT_INPUTS.iter().map(|(slug, input)| {
        Case::new(
            format!("color-limits.{slug}"),
            "color.limits",
            "absent",
            input,
        )
    }));
    cases.extend(
        ERROR_CASES
            .iter()
            .map(|(slug, front)| Case::new(format!("error.{slug}"), front, slug, "-")),
    );
    cases
}

/// The sheet and style-attribute report cases. Their inputs keep their capture
/// provenance; `feature` is the historical feature profile of each case.
fn report_cases() -> Vec<Case> {
    let mut lines = include_str!("report_cases.tsv").lines();
    assert_eq!(lines.next(), Some("case_id\tentry\tfeature\tinput"));
    lines
        .map(|line| {
            let [id, front, feature, input] = line
                .split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .unwrap_or_else(|_| panic!("report case has four columns: {line}"));
            Case {
                id: id.to_owned(),
                front: front.to_owned(),
                feature: feature.to_owned(),
                argument: "-".to_owned(),
                input: unescape(input),
            }
        })
        .collect()
}

fn unescape(field: &str) -> String {
    let mut output = String::with_capacity(field.len());
    let mut characters = field.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match characters.next() {
            Some('\\') => output.push('\\'),
            Some('t') => output.push('\t'),
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            other => panic!("malformed report case escape {other:?} in {field}"),
        }
    }
    output
}

/// Each inventory property's accepted and rejected values through the text
/// front, and its first accepted value through the components and grammar
/// fronts. The important dispatch stimulus runs through the text front with
/// `!important`.
fn property_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut push = |cases: &mut Vec<Case>, id: String, front: &str, argument: &str, input: &str| {
        assert!(ids.insert(id.clone()), "property case id collides: {id}");
        cases.push(Case::new(id, front, argument, input));
    };
    for record in property_expectations::CASES {
        let name = record.name;
        let positive = record.catalog.as_ref().map(|catalog| match catalog {
            CatalogExpectation::Grammar { positive, .. } => *positive,
            CatalogExpectation::Complete { authored, .. } => *authored,
        });
        // The grammar catalog's positive sample, including the samples whose
        // serialization is a pinned known defect, then the extra accepts.
        let accepted = positive
            .into_iter()
            .chain(
                record
                    .accepted_rows()
                    .into_iter()
                    .map(|(authored, _)| authored),
            )
            .fold(Vec::new(), |mut values, value| {
                // The catalog sample is also an accepted row unless it is a
                // known serialization defect; one value is one case.
                if !values.contains(&value) {
                    values.push(value);
                }
                values
            });
        for authored in &accepted {
            push(
                &mut cases,
                format!("property.{name}.accept.{}", value_key(authored)),
                "property-value.text",
                name,
                authored,
            );
        }
        // A record may list one rejected value twice; one value is one case.
        let mut seen = Vec::new();
        let mut rejected = record.rejected_rows();
        rejected.retain(|value| {
            let first = !seen.contains(value);
            seen.push(*value);
            first
        });
        for authored in rejected {
            push(
                &mut cases,
                format!("property.{name}.reject.{}", value_key(authored)),
                "property-value.text",
                name,
                authored,
            );
        }
        if let Some(first) = accepted.first() {
            for front in [
                "property-value.components",
                "property-value.grammar.text",
                "property-value.grammar.components",
            ] {
                let suffix = front.strip_prefix("property-value.").unwrap();
                push(
                    &mut cases,
                    format!("property.{name}.{suffix}"),
                    front,
                    name,
                    first,
                );
            }
        }
        if let Some(dispatch) = &record.dispatch {
            push(
                &mut cases,
                format!("property.{name}.important"),
                "property-value.text",
                &format!("{name}!important"),
                dispatch.important,
            );
        }
    }
    cases
}

/// A stable key for an authored value: a readable slug and a hash of the
/// exact text, so neither table order nor a near-identical value renames it.
fn value_key(value: &str) -> String {
    let mut slug = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    let slug = if slug.is_empty() { "empty" } else { slug };
    let slug = &slug[..slug.len().min(32)];
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{}-{:08x}", slug.trim_end_matches('-'), hash & 0xffff_ffff)
}

/// Colour inputs through the four ordinary used-colour requests, authored
/// resolution and the controlled protocol, under one or more fact sets.
fn color_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (slug, facts, input) in COLOR_INPUTS {
        for front in [
            "color.used.utf8.whole",
            "color.used.utf16.whole",
            "color.used.utf8.prefix",
            "color.used.utf16.prefix",
            "color.authored",
            "color.protocol.html-srgb",
            "color.protocol.css-p3",
            "color.protocol.quantize-p3",
        ] {
            let suffix = front.strip_prefix("color.").unwrap();
            cases.push(Case::new(
                format!("color.{slug}.{facts}.{suffix}"),
                front,
                facts,
                input,
            ));
        }
    }
    cases
}

/// Inputs whose ordinary colour limits are searched (`(slug, input)`).
const COLOR_LIMIT_INPUTS: &[(&str, &str)] = &[("named", "red"), ("hex", "#336699")];

/// `(slug, facts, input)`. Fact sets: `absent` has no palette, scheme or
/// current colour; `light` has initial properties, a light scheme and a
/// palette; `current` has a current colour and a dark scheme.
const COLOR_INPUTS: &[(&str, &str, &str)] = &[
    ("named", "absent", "red"),
    ("named-upper", "absent", "ReBeccaPurple"),
    ("transparent", "absent", "transparent"),
    ("hex-3", "absent", "#0f8"),
    ("hex-4", "absent", "#0f88"),
    ("hex-6", "absent", "#336699"),
    ("hex-8", "absent", "#33669980"),
    ("hex-bad", "absent", "#12345"),
    ("rgb-legacy", "absent", "rgb(255, 0, 0)"),
    ("rgb-modern-alpha", "absent", "rgb(10% 20% 30% / 0.5)"),
    ("rgba-legacy", "absent", "rgba(1, 2, 3, 40%)"),
    ("rgb-mixed-legacy", "absent", "rgb(1, 2%, 3)"),
    ("rgb-overflow", "absent", "rgb(300 -20 1e40)"),
    ("rgb-calc", "absent", "rgb(calc(255 / 2) calc(10 * 3) 0)"),
    ("rgb-none", "absent", "rgb(none 0 0 / none)"),
    ("hsl", "absent", "hsl(120deg 50% 25%)"),
    ("hsl-turn", "absent", "hsl(0.5turn 100% 50% / 25%)"),
    ("hwb", "absent", "hwb(200 10% 20%)"),
    ("lab", "absent", "lab(50% 40 -20)"),
    ("lch", "absent", "lch(60 30 270)"),
    ("oklab", "absent", "oklab(0.5 0.1 -0.1)"),
    ("oklch", "absent", "oklch(70% 0.2 30 / 0.8)"),
    ("color-srgb", "absent", "color(srgb 0.1 0.2 0.3)"),
    ("color-p3", "absent", "color(display-p3 1 0 0)"),
    ("color-xyz", "absent", "color(xyz-d65 0.2 0.3 0.4 / 50%)"),
    (
        "color-rec2020-overflow",
        "absent",
        "color(rec2020 2 -1 0.5)",
    ),
    ("mix", "absent", "color-mix(in srgb, red, blue)"),
    (
        "mix-weighted",
        "absent",
        "color-mix(in oklch longer hue, red 30%, blue)",
    ),
    ("mix-zero", "absent", "color-mix(in srgb, red 0%, blue 0%)"),
    ("relative", "absent", "rgb(from red r g calc(b + 10))"),
    (
        "relative-hsl",
        "absent",
        "hsl(from #336699 calc(h + 30) s l / alpha)",
    ),
    ("light-dark-absent", "absent", "light-dark(red, blue)"),
    ("light-dark", "light", "light-dark(red, blue)"),
    ("light-dark-current", "current", "light-dark(red, blue)"),
    ("current-absent", "absent", "currentcolor"),
    ("current", "current", "currentColor"),
    ("current-initial", "light", "currentcolor"),
    ("system-absent", "absent", "Canvas"),
    ("system", "light", "CanvasText"),
    ("system-missing", "light", "Highlight"),
    ("contrast", "absent", "contrast-color(white)"),
    ("device-cmyk", "absent", "device-cmyk(0 0.5 1 0)"),
    ("prefix-tail", "absent", "red garbage"),
    ("prefix-function-tail", "absent", "rgb(1 2 3) , blue"),
    ("whitespace", "absent", "  blue  "),
    ("empty", "absent", ""),
    ("unknown-name", "absent", "notacolor"),
    ("bad-function", "absent", "rgb(1 2)"),
    ("bad-alpha", "absent", "rgb(1 2 3 / red)"),
    ("non-ascii", "absent", "r\u{e9}d"),
    ("comment", "absent", "/* c */ red /* d */"),
    ("escape", "absent", "r\\65 d"),
    ("calc-unit", "absent", "rgb(calc(1px / 1px) 0 0)"),
    ("calc-contextual-unit", "absent", "rgb(calc(1em / 1px) 0 0)"),
    (
        "numerical-lab-infinity",
        "absent",
        "lab(50 calc(infinity) 0)",
    ),
    ("numerical-mix-overflow", "absent", NUMERICAL_OVERFLOW_INPUT),
];

/// `(front, argument, slug, input)`; the case id is `front.{front}.{slug}`.
const FRONT_CASES: &[(&str, &str, &str, &str)] = &[
    // Stylesheets from bytes and from UTF-16.
    ("sheet.bytes", "-", "utf8", ".a { color: red }"),
    (
        "sheet.bytes",
        "-",
        "charset-rule",
        "@charset \"windows-1252\"; .a { content: \"\u{e9}\" }",
    ),
    (
        "sheet.bytes",
        "windows-1252",
        "protocol-label",
        ".a { color: red }",
    ),
    ("sheet.bytes", "-", "bom", "\u{feff}.a { color: red }"),
    (
        "sheet.bytes",
        "-",
        "recovered",
        ".a { color: red; width: } @foo; .b {",
    ),
    (
        "sheet.utf16",
        "-",
        "clean",
        ".a { color: red } @media print { .b { width: 1px } }",
    ),
    ("sheet.utf16", "-", "recovered", ".a { color: ; } }"),
    (
        "sheet.utf16",
        "-",
        "non-ascii",
        ".\u{e9} { content: \"\u{1f600}\" }",
    ),
    ("sheet.utf16", "-", "empty", ""),
    // Sheets for the rule families later steps reshape.
    (
        "sheet",
        "-",
        "scope-body",
        "@scope (.a) to (.b) { color: red; .c { width: 1px } @media print { .d { height: 2px } } }",
    ),
    (
        "sheet",
        "-",
        "scope-nested",
        ".x { @scope (.a) { :scope { color: red } } }",
    ),
    (
        "sheet",
        "-",
        "scope-implicit",
        "@scope { .a { color: blue } }",
    ),
    (
        "sheet",
        "-",
        "supports-parenthesized",
        "@supports ((display: grid)) and (not (display: inline-grid)) { .a { color: red } }",
    ),
    (
        "sheet",
        "-",
        "supports-selector",
        "@supports selector(a > b) or font-tech(color-COLRv1) { .a { color: red } }",
    ),
    (
        "sheet",
        "-",
        "media-nested-condition",
        "@media ((min-width: 1px) and (not (hover))) or print { .a { color: red } }",
    ),
    (
        "sheet",
        "-",
        "container-style",
        "@container card (inline-size > 30em) and style(--x: 1) { .a { color: red } }",
    ),
    (
        "sheet",
        "-",
        "when-else",
        "@when media(width > 1px) { .a { color: red } } @else supports(display: grid) { .b { color: blue } } @else { .c { color: green } }",
    ),
    (
        "sheet",
        "-",
        "four-sides",
        ".a { margin: 1px 2px 1px 2px; padding: 1px; inset: 1px 2px 3px; scroll-margin: 0 0 0 0; scroll-padding: 1px 1px }",
    ),
    (
        "sheet",
        "-",
        "border-sides",
        ".a { border-color: red red; border-style: solid none solid none; border-width: thin medium thick 1px; border-radius: 1px 2px / 3px }",
    ),
    (
        "sheet",
        "-",
        "logical-pairs",
        ".a { margin-block: 1px 1px; padding-inline: 2px; inset-block: auto 1px; border-block-color: red blue; scroll-margin-inline: 1px 2px }",
    ),
    (
        "sheet",
        "-",
        "positions",
        ".a { object-position: right 10px top; perspective-origin: left; mask-position: center, 1px 2px; background-position: bottom 1px right 2px }",
    ),
    (
        "sheet",
        "-",
        "numerics",
        ".a { width: calc(100% - 2 * 3px); opacity: clamp(0, 1.5, 1); rotate: 0.25turn; line-height: 1.5e0; z-index: -0; transition-duration: 250ms }",
    ),
    (
        "sheet",
        "-",
        "integers-and-indices",
        ".a { hyphenate-limit-chars: 6 3 calc(2); font-palette: palette-mix(in lch, --a 40%, --b); order: calc(1 + 1) }",
    ),
    (
        "sheet",
        "-",
        "page-margin",
        "@page :first { margin: 1in; size: a4 landscape; @top-left { content: \"x\" } }",
    ),
    (
        "sheet",
        "-",
        "font-face",
        "@font-face { font-family: \"X\"; src: url(x.woff2) format(\"woff2\"); font-display: swap; unicode-range: U+0-7F }",
    ),
    (
        "sheet",
        "-",
        "keyframes",
        "@keyframes spin { from { rotate: 0deg } 50%, 75% { opacity: 0.5 } to { rotate: 1turn } }",
    ),
    (
        "sheet",
        "-",
        "layers-and-import",
        "@import url(a.css) layer(base) supports(display: grid) screen; @layer a, b; @layer a { .x { color: red } }",
    ),
    (
        "sheet",
        "-",
        "namespace",
        "@namespace svg url(http://www.w3.org/2000/svg); svg|rect { fill: red }",
    ),
    (
        "sheet",
        "-",
        "nesting",
        ".a { color: red; & .b { color: blue } .c & { width: 1px } width: 2px }",
    ),
    // Style attribute and declaration lists: the inputs whose handling
    // differs between the three declaration-list readings.
    ("style", "-", "qualified-rule-start", "a {} color: red"),
    ("style", "-", "root-close-brace", "color: red } width: 1px"),
    ("style", "-", "nested-rule", "color: red; .x { width: 1px }"),
    (
        "style",
        "-",
        "important-spacing",
        "color: red ! IMPORTANT ; width: 1px!important",
    ),
    (
        "block-contents",
        "-",
        "qualified-rule-start",
        "a {} color: red",
    ),
    (
        "block-contents",
        "-",
        "root-close-brace",
        "color: red } width: 1px",
    ),
    (
        "block-contents",
        "-",
        "nested-rule",
        "color: red; .x { width: 1px }",
    ),
    (
        "block-contents",
        "-",
        "at-rule",
        "color: red; @media print { width: 1px }",
    ),
    (
        "block-contents",
        "-",
        "clean",
        "color: red; margin: 1px 2px; --x: { a: b }",
    ),
    (
        "block-contents",
        "-",
        "deep",
        "--x: ((((((((((((((((((((a))))))))))))))))))))",
    ),
    ("block-contents", "-", "empty", ""),
    (
        "block-contents.utf16",
        "-",
        "qualified-rule-start",
        "a {} color: red",
    ),
    (
        "block-contents.utf16",
        "-",
        "root-close-brace",
        "color: red } width: 1px",
    ),
    (
        "block-contents.utf16",
        "-",
        "clean",
        "color: red; margin: 1px 2px",
    ),
    (
        "block-contents.utf16",
        "-",
        "non-ascii",
        "content: \"\u{e9}\u{1f600}\"; color: blue",
    ),
    (
        "block-contents.page",
        "-",
        "clean",
        "size: a4; margin: 1in; @top-center { content: \"t\" }",
    ),
    (
        "block-contents.page",
        "-",
        "recovered",
        "size: banana; color: red; marks: crop cross",
    ),
    (
        "block-contents.page",
        "-",
        "root-close-brace",
        "margin: 1px } size: a5",
    ),
    (
        "block-contents.font-face",
        "-",
        "clean",
        "font-family: X; src: local(X), url(x.woff); font-weight: 100 900",
    ),
    (
        "block-contents.font-face",
        "-",
        "recovered",
        "font-family: ; color: red; font-display: never",
    ),
    (
        "declaration-list",
        "-",
        "qualified-rule-start",
        "a {} color: red",
    ),
    (
        "declaration-list",
        "-",
        "root-close-brace",
        "color: red } width: 1px",
    ),
    (
        "declaration-list",
        "-",
        "nested-rule",
        "color: red; .x { width: 1px }",
    ),
    (
        "declaration-list",
        "-",
        "clean",
        "color: red; width: 1px !important",
    ),
    ("sheet.validate", "-", "clean", ".a { color: red }"),
    (
        "sheet.validate",
        "-",
        "recovered",
        ".a { color: red; width: }",
    ),
    ("style.validate", "-", "clean", "color: red"),
    ("style.validate", "-", "recovered", "color: red; width:"),
    ("declaration-list.validate", "-", "clean", "color: red"),
    (
        "declaration-list.validate",
        "-",
        "recovered",
        "a {} color: red",
    ),
    // Property values in each context.
    (
        "property-value.text",
        "margin",
        "margin-four-equal",
        "1px 1px 1px 1px",
    ),
    (
        "property-value.text",
        "margin",
        "margin-pair",
        "1px 2px 1px 2px",
    ),
    (
        "property-value.text",
        "padding",
        "padding-three",
        "1px 2px 1px",
    ),
    (
        "property-value.text",
        "inset",
        "inset-auto",
        "auto auto auto auto",
    ),
    (
        "property-value.text",
        "scroll-margin",
        "scroll-margin-two",
        "1px 1px",
    ),
    (
        "property-value.text",
        "scroll-padding",
        "scroll-padding-four",
        "auto 1px auto 1px",
    ),
    (
        "property-value.text",
        "border-color",
        "border-color-two",
        "red red",
    ),
    (
        "property-value.text",
        "border-width",
        "border-width-calc",
        "calc(1px + 1px) thin",
    ),
    (
        "property-value.text",
        "border-radius",
        "border-radius-slash",
        "1px 1px 1px 1px / 2px 2px",
    ),
    (
        "property-value.text",
        "margin-inline",
        "margin-inline-pair",
        "1px 1px",
    ),
    (
        "property-value.text",
        "padding-block",
        "padding-block-pair",
        "1px 2px",
    ),
    (
        "property-value.text",
        "scroll-snap-align",
        "scroll-snap-align-one",
        "start",
    ),
    (
        "property-value.text",
        "scroll-snap-align",
        "scroll-snap-align-equal",
        "start start",
    ),
    (
        "property-value.text",
        "scroll-snap-align",
        "scroll-snap-align-two",
        "start end",
    ),
    (
        "property-value.text",
        "contain-intrinsic-size",
        "contain-intrinsic-size-equal",
        "10px 10px",
    ),
    (
        "property-value.text",
        "contain-intrinsic-size",
        "contain-intrinsic-size-unequal",
        "auto 10px none",
    ),
    (
        "property-value.text",
        "border-spacing",
        "border-spacing-one",
        "2px",
    ),
    (
        "property-value.text",
        "border-top-left-radius",
        "border-corner-radius-equal",
        "1px 1px",
    ),
    (
        "property-value.text",
        "interest-delay",
        "interest-delay-one",
        "1s",
    ),
    (
        "property-value.text",
        "interest-delay",
        "interest-delay-equal",
        "1s 1s",
    ),
    (
        "property-value.text",
        "interest-delay",
        "interest-delay-two",
        "1s 2s",
    ),
    (
        "property-value.text",
        "clip-path",
        "clip-path-inset-round",
        "inset(1px round 2px 2px 2px 2px / 3px)",
    ),
    (
        "property-value.text",
        "object-position",
        "object-position-flow-relative",
        "start end",
    ),
    (
        "property-value.text",
        "perspective-origin",
        "perspective-origin-flow-relative",
        "x-start y-end",
    ),
    (
        "property-value.text",
        "mask-position",
        "mask-position-flow-relative",
        "block-start inline-end",
    ),
    (
        "property-value.text",
        "object-position",
        "position-four",
        "right 10px bottom 5%",
    ),
    (
        "property-value.text",
        "width",
        "calc-nested",
        "calc(calc(100% / 3) - min(1px, 2em))",
    ),
    (
        "property-value.text",
        "width",
        "calc-recovered-math",
        "calc(1px + )",
    ),
    (
        "property-value.text",
        "width",
        "calc-infinity",
        "calc(infinity * 1px)",
    ),
    (
        "property-value.text",
        "width",
        "calc-nan",
        "calc(NaN * 1px)",
    ),
    ("property-value.text", "opacity", "number-exponent", "5e-1"),
    ("property-value.text", "opacity", "percentage", "50%"),
    ("property-value.text", "rotate", "angle-units", "1rad"),
    ("property-value.text", "line-height", "unitless-zero", "0"),
    (
        "property-value.text",
        "z-index",
        "z-index-integer-calc",
        "calc(1.5)",
    ),
    (
        "property-value.text",
        "transition-duration",
        "time-negative",
        "-1s",
    ),
    (
        "property-value.text",
        "font-weight",
        "number-range",
        "calc(1000 + 1)",
    ),
    (
        "property-value.text",
        "hyphenate-limit-chars",
        "hyphenate-integer-calc",
        "calc(2) auto 3",
    ),
    (
        "property-value.text",
        "font-palette",
        "palette-mix",
        "palette-mix(in oklab, light 20%, dark)",
    ),
    (
        "property-value.text",
        "aspect-ratio",
        "ratio",
        "16 / calc(9)",
    ),
    ("property-value.text", "color", "var", "var(--x, red)"),
    ("property-value.text", "color", "global", "revert-layer"),
    ("property-value.text", "--x", "custom", " { a } [b] (c) "),
    ("property-value.text", "width", "standards-unitless", "10"),
    (
        "property-value.text",
        "quirks:width",
        "quirks-unitless",
        "10",
    ),
    (
        "property-value.text",
        "quirks:color",
        "quirks-hashless",
        "ff0000",
    ),
    (
        "property-value.text",
        "glyph-orientation-vertical",
        "svg-only",
        "90deg",
    ),
    (
        "property-value.components",
        "margin",
        "margin-pair",
        "1px 2px 1px 2px",
    ),
    ("property-value.components", "--x", "custom", "a b"),
    ("property-value.components", "width", "rejected", "red"),
    ("property-value.page", "margin", "margin", "1in 2in"),
    ("property-value.page", "color", "not-applicable", "red"),
    ("property-value.page", "size", "descriptor", "a4"),
    (
        "property-value.margin",
        "content",
        "content",
        "\"x\" counter(page)",
    ),
    ("property-value.margin", "width", "width", "1px"),
    (
        "property-value.margin",
        "display",
        "not-applicable",
        "block",
    ),
    (
        "property-value.grammar.text",
        "page-break-before",
        "legacy-grammar",
        "always",
    ),
    (
        "property-value.grammar.components",
        "page-break-after",
        "legacy-grammar",
        "avoid",
    ),
    ("svg-glyph-orientation-vertical.text", "-", "angle", "90deg"),
    ("svg-glyph-orientation-vertical.text", "-", "auto", "auto"),
    (
        "svg-glyph-orientation-vertical.text",
        "-",
        "rejected",
        "45deg",
    ),
    (
        "svg-glyph-orientation-vertical.components",
        "-",
        "number",
        "0",
    ),
    (
        "svg-glyph-orientation-vertical.components",
        "-",
        "rejected",
        "red",
    ),
    // Single declarations and rules.
    ("declaration", "-", "clean", "color: red !important"),
    ("declaration", "-", "custom", "--x: { a }"),
    ("declaration", "-", "invalid-value", "width: red"),
    ("declaration", "-", "trailing", "color: red; width: 1px"),
    ("declaration", "-", "unknown", "colr: red"),
    ("declaration", "-", "empty", ""),
    ("rule", "-", "style", ".a > .b { color: red }"),
    (
        "rule",
        "-",
        "media",
        "@media screen and (min-width: 1px) { .a { color: red } }",
    ),
    ("rule", "-", "namespaced", "svg|rect { fill: red }"),
    ("rule", "-", "two-rules", ".a {} .b {}"),
    ("rule", "-", "invalid", "@media { .a { color: red }"),
    ("rule", "-", "import-late", "@import url(a.css);"),
    ("rule", "-", "empty", ""),
    ("rule-syntax", "-", "qualified", ".a { color: red }"),
    (
        "rule-syntax",
        "-",
        "at-rule-block",
        "@media print { .a {} }",
    ),
    (
        "rule-syntax",
        "-",
        "at-rule-statement",
        "@import url(a.css) screen;",
    ),
    ("rule-syntax", "-", "unknown-at-rule", "@foo bar { baz }"),
    ("rule-syntax", "-", "two-rules", ".a {} .b {}"),
    ("rule-syntax", "-", "empty", ""),
    // Selectors.
    (
        "selector",
        "-",
        "compound",
        "a.b#c[d=\"e\" i]:hover::before",
    ),
    ("selector", "svg", "namespaced", "svg|rect"),
    ("selector", "-", "undeclared-prefix", "svg|rect"),
    ("selector", "-", "list", "a, b"),
    ("selector", "-", "invalid", "a >"),
    ("selector-list", "-", "list", "a, .b > c, :is(d, e) + f"),
    (
        "selector-list",
        "-",
        "forgiving",
        ":is(a, :unknown, b), :where()",
    ),
    ("selector-list", "-", "invalid-item", "a, ::, b"),
    ("selector-list", "-", "nesting-selector", "& .a"),
    ("style-selector-list", "ordinary", "ordinary", "a, b"),
    ("style-selector-list", "nested", "relative", "> a, & b, c"),
    (
        "style-selector-list",
        "scoped-present",
        "scope-present",
        ":scope > a, b",
    ),
    (
        "style-selector-list",
        "scoped-absent",
        "scope-absent",
        ":scope > a, b",
    ),
    ("style-selector-list", "ordinary", "invalid", "a, )"),
    (
        "relative-selector-list",
        "-",
        "combinators",
        "> a, + b, ~ c, d",
    ),
    ("relative-selector-list", "-", "invalid", ">> a"),
    ("relative-selector-list", "-", "empty", ""),
    // Media queries.
    (
        "media-query",
        "-",
        "type-and-feature",
        "screen and (min-width: 100px)",
    ),
    ("media-query", "-", "range", "(400px <= width < 700px)"),
    ("media-query", "-", "not-only", "not print"),
    ("media-query", "-", "invalid", "screen and"),
    ("media-query", "-", "list", "a, b"),
    (
        "media-query.cssom",
        "-",
        "type-and-feature",
        "SCREEN AND (MIN-WIDTH: 100PX)",
    ),
    ("media-query.cssom", "-", "invalid", "(min-width: )"),
    ("media-query.cssom", "-", "list", "a, b"),
    (
        "media-query-list",
        "-",
        "list",
        "screen, (hover: hover) and (pointer: fine), print",
    ),
    (
        "media-query-list",
        "-",
        "recovered-item",
        "screen, (min-width: red), print",
    ),
    ("media-query-list", "-", "empty", ""),
    ("media-query-list", "-", "unknown-feature", "(foo: bar)"),
    (
        "media-query-list.utf16",
        "-",
        "list",
        "screen, (hover: hover)",
    ),
    (
        "media-query-list.utf16",
        "-",
        "recovered-item",
        "screen, (min-width: red)",
    ),
    ("media-query-list.utf16", "-", "non-ascii", "(\u{e9}: 1)"),
    // Supports.
    ("supports-condition", "-", "declaration", "(display: grid)"),
    (
        "supports-condition",
        "-",
        "nested",
        "((display: grid)) and (not (color: red))",
    ),
    ("supports-condition", "-", "selector", "selector(a > b)"),
    (
        "supports-condition",
        "svg",
        "namespaced-selector",
        "selector(svg|rect)",
    ),
    ("supports-condition", "-", "general-enclosed", "foo(bar)"),
    ("supports-condition", "-", "invalid", "display: grid"),
    ("supports.css-text", "-", "declaration", "(display: grid)"),
    (
        "supports.css-text",
        "-",
        "bare-declaration",
        "display: grid",
    ),
    ("supports.css-text", "-", "invalid", "(display: grid"),
    ("supports.declaration", "display", "valid", "grid"),
    (
        "supports.declaration",
        "color",
        "important",
        "red !important",
    ),
    ("supports.declaration", "--x", "custom", "anything"),
    ("supports.declaration", "width", "invalid", "red"),
    ("supports.test-child", "-", "declaration", "display: grid"),
    (
        "supports.test-child",
        "-",
        "condition",
        "(display: grid) or (color: red)",
    ),
    ("supports.test-child", "-", "invalid", "display grid"),
    (
        "supports.test-body",
        "-",
        "two-children",
        "display: grid; color: red",
    ),
    (
        "supports.test-body",
        "-",
        "recovered",
        "display: grid; ???; color: red",
    ),
    ("supports.test-body", "-", "empty", ""),
    ("when-condition", "-", "media", "media(width > 1px)"),
    (
        "when-condition",
        "-",
        "supports",
        "supports(display: grid) and media(print)",
    ),
    (
        "when-condition",
        "-",
        "not",
        "not (media(print) or supports(color: red))",
    ),
    ("when-condition", "-", "invalid", "media(width >)"),
    // Keyframes, page and font-feature fragments.
    (
        "keyframe-rule",
        "-",
        "clean",
        "from, 50% { opacity: 0; color: red }",
    ),
    (
        "keyframe-rule",
        "-",
        "important",
        "to { opacity: 1 !important; width: 1px }",
    ),
    (
        "keyframe-rule",
        "-",
        "invalid-selector",
        "51 { opacity: 0 }",
    ),
    ("keyframe-selectors", "-", "list", "from, 25.5%, TO"),
    (
        "keyframe-selectors",
        "-",
        "timeline-range",
        "entry 10%, exit 90%",
    ),
    ("keyframe-selectors", "-", "invalid", "101%, from"),
    (
        "page-selectors",
        "-",
        "list",
        ":first, named:left, :blank:right",
    ),
    ("page-selectors", "-", "invalid", ":middle"),
    ("page-selectors", "-", "empty", ""),
    (
        "page-margin-rule",
        "-",
        "clean",
        "@top-left { content: \"a\"; width: 1px }",
    ),
    (
        "page-margin-rule",
        "-",
        "recovered",
        "@bottom-center { color: ; content: \"b\" }",
    ),
    (
        "page-margin-rule",
        "-",
        "invalid",
        "@middle { content: \"c\" }",
    ),
    (
        "font-feature-family-list",
        "-",
        "list",
        "Font One, \"Font Two\"",
    ),
    ("font-feature-family-list", "-", "invalid", "serif, 1"),
    ("font-feature-family-list", "-", "empty", ""),
    ("font-feature-value", "styleset", "styleset", "1 2 20"),
    ("font-feature-value", "swash", "swash-two", "1 2"),
    (
        "font-feature-value",
        "character-variant",
        "character-variant",
        "2 3",
    ),
    ("font-feature-value", "annotation", "negative", "-1"),
    ("font-feature-display-value", "-", "keyword", "swap"),
    ("font-feature-display-value", "-", "invalid", "never"),
    // The five descriptor-value fronts.
    ("descriptor.color-profile", "src", "src", "url(profile.icc)"),
    (
        "descriptor.color-profile",
        "rendering-intent",
        "rendering-intent",
        "perceptual",
    ),
    (
        "descriptor.color-profile",
        "components",
        "components",
        "cyan magenta",
    ),
    (
        "descriptor.color-profile",
        "rendering-intent",
        "invalid",
        "banana",
    ),
    ("descriptor.counter-style", "system", "system", "fixed 3"),
    (
        "descriptor.counter-style",
        "symbols",
        "symbols",
        "\"*\" url(a.svg) b",
    ),
    (
        "descriptor.counter-style",
        "range",
        "range",
        "1 10, infinite -1",
    ),
    ("descriptor.counter-style", "pad", "invalid", "-1 \"0\""),
    (
        "descriptor.font-face",
        "src",
        "src",
        "local(X), url(x.woff2) format(woff2) tech(variations)",
    ),
    (
        "descriptor.font-face",
        "font-weight",
        "weight-range",
        "900 100",
    ),
    (
        "descriptor.font-face",
        "unicode-range",
        "unicode-range",
        "U+0025-00FF, U+4??",
    ),
    ("descriptor.font-face", "font-display", "invalid", "never"),
    ("descriptor.font-palette", "base-palette", "index", "1"),
    (
        "descriptor.font-palette",
        "override-colors",
        "colors",
        "0 red, 1 #00f",
    ),
    (
        "descriptor.font-palette",
        "font-family",
        "family",
        "Bixa, \"X\"",
    ),
    ("descriptor.font-palette", "base-palette", "invalid", "-1"),
    ("descriptor.page", "size", "size", "a4 landscape"),
    ("descriptor.page", "marks", "marks", "crop cross"),
    ("descriptor.page", "bleed", "bleed", "6pt"),
    ("descriptor.page", "page-orientation", "invalid", "sideways"),
    // The fifteen real-brace block fragments.
    (
        "block.color-profile",
        "-",
        "clean",
        "{ src: url(a.icc); rendering-intent: relative-colorimetric }",
    ),
    (
        "block.counter-style",
        "-",
        "clean",
        "{ system: cyclic; symbols: a b; suffix: \".\" }",
    ),
    (
        "block.counter-style",
        "-",
        "recovered",
        "{ system: banana; symbols: a }",
    ),
    (
        "block.font-face",
        "-",
        "clean",
        "{ font-family: X; src: url(x.woff) }",
    ),
    (
        "block.font-feature-value",
        "styleset",
        "clean",
        "{ nice: 1 2; fancy: 3 }",
    ),
    (
        "block.font-feature-values",
        "-",
        "clean",
        "{ font-display: swap; @styleset { nice: 1 } @swash { flowing: 1 } }",
    ),
    (
        "block.font-palette-values",
        "-",
        "clean",
        "{ font-family: Bixa; base-palette: 1; override-colors: 0 red }",
    ),
    (
        "block.group",
        "-",
        "clean",
        "{ .a { color: red } @media print { .b { width: 1px } } }",
    ),
    (
        "block.group",
        "svg",
        "namespaced",
        "{ svg|rect { fill: red } }",
    ),
    (
        "block.keyframe-declarations",
        "-",
        "clean",
        "{ opacity: 0; color: red; animation-name: x }",
    ),
    (
        "block.keyframes",
        "-",
        "clean",
        "{ from { opacity: 0 } to { opacity: 1 } }",
    ),
    (
        "block.margin",
        "-",
        "clean",
        "{ content: \"x\"; width: 1px; display: block }",
    ),
    (
        "block.page",
        "-",
        "clean",
        "{ size: a4; margin: 1in; @top-left { content: \"x\" } }",
    ),
    (
        "block.scope",
        "present",
        "present",
        "{ color: red; .a { width: 1px } @media print { .b { color: blue } } }",
    ),
    (
        "block.scope",
        "absent",
        "absent",
        "{ :scope { color: red } }",
    ),
    (
        "block.scoped-group",
        "present",
        "clean",
        "{ .a { color: red } }",
    ),
    (
        "block.scoped-group",
        "absent",
        "declarations",
        "{ color: red; .a { width: 1px } }",
    ),
    (
        "block.supports-test",
        "-",
        "clean",
        "{ display: grid; color: red }",
    ),
    (
        "block.style",
        "-",
        "clean",
        "{ color: red; & .a { width: 1px } }",
    ),
    ("block.style", "-", "unterminated", "{ color: red"),
    ("block.style", "-", "no-brace", "color: red"),
    // The three component-value fronts.
    ("component-value", "-", "function", "rgb(1 2 3)"),
    ("component-value", "-", "block", "[a { b }]"),
    ("component-value", "-", "two-values", "a b"),
    ("component-value", "-", "bad-url", "url(a b)"),
    ("component-value", "-", "empty", ""),
    (
        "component-values",
        "-",
        "mixed",
        "a 1px \"s\" url(x) #id f(1, 2) { }",
    ),
    ("component-values", "-", "unclosed", "f(a [b"),
    ("component-values", "-", "unmatched-close", "a ) b"),
    ("component-values", "-", "bad-string", "\"a\nb\""),
    (
        "component-values",
        "-",
        "deep",
        "((((((((((((((((((((((((((((((((a))))))))))))))))))))))))))))))))",
    ),
    ("component-values.comma", "-", "three", "a, b c, d"),
    ("component-values.comma", "-", "empty-item", "a, , b"),
    ("component-values.comma", "-", "trailing", "a,"),
    // Lexical serializers.
    ("serialize.identifier", "-", "plain", "foo"),
    ("serialize.identifier", "-", "digit-start", "1a"),
    ("serialize.identifier", "-", "dash-digit", "-1"),
    ("serialize.identifier", "-", "double-dash", "--x"),
    ("serialize.identifier", "-", "escapes", "a b\u{0}c\u{7f}"),
    ("serialize.identifier", "-", "non-ascii", "\u{e9}\u{1d7d8}"),
    ("serialize.identifier", "-", "empty", ""),
    ("serialize.string", "-", "plain", "abc"),
    ("serialize.string", "-", "quotes", "a\"b\\c"),
    ("serialize.string", "-", "controls", "a\nb\u{0}"),
    ("serialize.string", "-", "empty", ""),
    ("serialize.comma-list", "-", "three", "a|b c|d"),
    ("serialize.comma-list", "-", "single", "a"),
    ("serialize.comma-list", "-", "empty-item", "a||b"),
    ("serialize.whitespace-list", "-", "three", "a|b|c"),
    ("serialize.whitespace-list", "-", "single", "a"),
    (
        "serialize.font-face-family-list",
        "-",
        "families",
        "Inter|Open Sans|serif",
    ),
    (
        "serialize.font-face-family-list",
        "-",
        "quoted",
        "Font \"X\"|monospace",
    ),
    (
        "serialize.font-face-family-list",
        "-",
        "generic-keyword",
        "sans-serif",
    ),
    // Normalization, expansion and stylesheet options.
    (
        "sheet.normalize",
        "sheet",
        "nested",
        ".a { margin: 1px; .b { color: red } } @media print { .c { padding: 2px 3px } }",
    ),
    (
        "sheet.normalize",
        "report",
        "recovered",
        ".a { margin: 1px; width: } @foo; .b { all: initial }",
    ),
    (
        "sheet.normalize",
        "sheet",
        "pending",
        ".a { margin: var(--x) 1px; --x: 2px }",
    ),
    (
        "sheet.normalize",
        "report",
        "layers-and-scope",
        "@layer a { .x { color: red } } @scope (.a) { .b { inset: 1px } }",
    ),
    (
        "declaration.expand",
        "-",
        "longhands",
        "margin: 1px 2px; border: 1px solid red",
    ),
    (
        "declaration.expand",
        "-",
        "custom-and-global",
        "--x: 1; color: inherit; all: revert",
    ),
    (
        "declaration.expand",
        "-",
        "pending",
        "margin: var(--x); font: var(--f)",
    ),
    (
        "sheet.with-options",
        "https://example.test/a.css",
        "location",
        ".a { color: red }",
    ),
    ("sheet.with-options", "-", "no-location", ".a {"),
    // Every parser-context method, under quirks mode.
    (
        "sheet",
        "quirks",
        "quirks",
        ".a { color: ff0000; width: 10; line-height: 1 }",
    ),
    ("sheet.utf16", "quirks", "quirks", ".a { width: 10 }"),
    ("style", "quirks", "quirks", "color: ff0000; height: 10"),
    ("declaration-list", "quirks", "quirks", "width: 10"),
    (
        "block-contents",
        "quirks",
        "quirks",
        "width: 10; color: f00",
    ),
    ("block-contents.utf16", "quirks", "quirks", "width: 10"),
    (
        "block-contents.page",
        "quirks",
        "quirks",
        "margin: 10; size: a4",
    ),
    (
        "property-value.text",
        "standards:width",
        "standards-context",
        "10px",
    ),
    ("property-value.components", "quirks:width", "quirks", "10"),
    ("property-value.page", "quirks:margin", "quirks", "10"),
    (
        "property-value.grammar.text",
        "quirks:width",
        "quirks",
        "10",
    ),
    (
        "property-value.grammar.components",
        "quirks:height",
        "quirks",
        "10",
    ),
    (
        "svg-glyph-orientation-vertical.text",
        "quirks",
        "quirks",
        "90",
    ),
    (
        "svg-glyph-orientation-vertical.components",
        "quirks",
        "quirks",
        "90deg",
    ),
    ("declaration", "quirks", "quirks", "width: 10"),
    ("rule", "quirks", "quirks", ".a { width: 10 }"),
    ("block.group", "quirks", "quirks", "{ .a { width: 10 } }"),
    (
        "block.scope",
        "quirks:present",
        "quirks",
        "{ width: 10; .a { height: 10 } }",
    ),
    (
        "block.scoped-group",
        "quirks:absent",
        "quirks",
        "{ .a { width: 10 } }",
    ),
    ("block.page", "quirks", "quirks", "{ margin: 10 }"),
    ("block.margin", "quirks", "quirks", "{ width: 10 }"),
    (
        "block.keyframes",
        "quirks",
        "quirks",
        "{ from { width: 10 } }",
    ),
    (
        "block.keyframe-declarations",
        "quirks",
        "quirks",
        "{ width: 10 }",
    ),
    ("block.style", "quirks", "quirks", "{ width: 10 }"),
    (
        "page-margin-rule",
        "quirks",
        "quirks",
        "@top-left { width: 10 }",
    ),
    ("keyframe-rule", "quirks", "quirks", "from { width: 10 }"),
    ("font-feature-family-list", "quirks", "quirks", "A, B"),
    ("media-query-list", "quirks", "quirks", "(min-width: 10)"),
    (
        "media-query-list.utf16",
        "quirks",
        "quirks",
        "(min-width: 10)",
    ),
    ("color-scheme.utf16", "quirks", "quirks", "dark"),
    ("supports-condition", "quirks", "quirks", "(width: 10)"),
    ("supports.css-text", "quirks", "quirks", "(width: 10)"),
    ("supports.declaration", "quirks:width", "quirks", "10"),
    ("when-condition", "quirks", "quirks", "supports(width: 10)"),
    // Rule graphs and edited-rule views.
    (
        "rule-graph",
        "-",
        "style-and-groups",
        ".a { color: red; .b { width: 1px } } @media print { .c { margin: 1px 2px } } @scope (.x) { .y { color: blue } }",
    ),
    (
        "rule-graph",
        "-",
        "page-and-keyframes",
        "@page :first { margin: 1in } @keyframes k { from { opacity: 0 } }",
    ),
    (
        "edited-rule",
        "-",
        "style-and-groups",
        ".a { color: red; .b { width: 1px } } @media print { .c { margin: 1px 2px } } @scope (.x) { .y { color: blue } }",
    ),
    (
        "edited-rule",
        "-",
        "import-and-font-face",
        "@import url(a.css) print; @font-face { font-family: X; src: url(x.woff) }",
    ),
    // Conformance metadata.
    ("conformance.feature-catalog", "-", "all", "-"),
    (
        "conformance.feature",
        "baseline.property.color",
        "property",
        "-",
    ),
    ("conformance.feature", "baseline.rule.media", "rule", "-"),
    ("conformance.feature", "unknown.feature", "unknown", "-"),
    ("conformance.specification-sources", "-", "all", "-"),
    (
        "conformance.specification-source",
        "O-SYNTAX3",
        "syntax-3",
        "-",
    ),
    (
        "conformance.specification-source",
        "unknown",
        "unknown",
        "-",
    ),
    ("conformance.exclusions", "-", "all", "-"),
    (
        "conformance.exclusion",
        "excluded.X-UI4.property.ime-mode",
        "ime-mode",
        "-",
    ),
    ("conformance.exclusion", "unknown", "unknown", "-"),
    ("conformance.property-support", "color", "color", "-"),
    (
        "conformance.property-support",
        "-webkit-appearance",
        "alias",
        "-",
    ),
    ("conformance.property-support", "ime-mode", "excluded", "-"),
    // `color-scheme` from UTF-16 input.
    ("color-scheme.utf16", "-", "list", "light dark"),
    ("color-scheme.utf16", "-", "only", "only light"),
    ("color-scheme.utf16", "-", "normal", "normal"),
    ("color-scheme.utf16", "-", "reserved", "light inherit"),
    ("color-scheme.utf16", "-", "unclosed", "light \"dark"),
];

/// `(slug, front)`: one representative of each error type with `Display`,
/// keyed by a stable case id rather than the type name.
const ERROR_CASES: &[(&str, &str)] = &[
    ("background-construction", "error"),
    ("character-alignment", "error"),
    ("color-construction", "error"),
    ("color-interpolation-method", "error"),
    ("color-mix-construction", "error"),
    ("color-operation", "error"),
    ("color-ordinary-limit", "error"),
    ("color-ordinary-operation", "error"),
    ("color-profile-value", "error"),
    ("color-scalar", "error"),
    ("color-scheme-construction", "error"),
    ("component-value", "error"),
    ("container-construction", "error"),
    ("counter-style-value", "error"),
    ("custom-media-construction", "error"),
    ("custom-media-serialization", "error"),
    ("declaration-block", "error"),
    ("empty-language-range-list", "error"),
    ("expansion", "error"),
    ("font-face-declaration-block", "error"),
    ("font-face-value", "error"),
    ("font-feature-value", "error"),
    ("font-feature-values", "error"),
    ("font-palette-construction", "error"),
    ("font-palette-mix-construction", "error"),
    ("font-palette-value", "error"),
    ("general-enclosed", "error"),
    ("gradient-stop-construction", "error"),
    ("grid-template-area", "error"),
    ("image-1d-construction", "error"),
    ("image-construction", "error"),
    ("import-construction", "error"),
    ("import-serialization", "error"),
    ("input-decode", "error"),
    ("keyframe-rule-view", "error"),
    ("keyframe-selector-comparison", "error"),
    ("media-construction", "error"),
    ("media-cssom-serialization", "error"),
    ("media-serialization", "error"),
    ("motion-construction", "error"),
    ("named-supports-construction", "error"),
    ("normalization", "error"),
    ("numeric-construction", "error"),
    ("page-block", "error"),
    ("page-projection", "error"),
    ("parse-diagnostic", "error"),
    ("path-data-construction", "error"),
    ("position-construction", "error"),
    ("property-metadata", "error"),
    ("property-value-parse", "error"),
    ("property-value-syntax", "error"),
    ("qualified-name", "error"),
    ("retained-payload", "error"),
    ("rule-construction", "error"),
    ("rule-cssom-serialization", "error"),
    ("selector-construction", "error"),
    ("shape-construction", "error"),
    ("specified-rule-serialization", "error"),
    ("specified-value-serialization", "error"),
    ("supports-construction", "error"),
    ("utf16-input-interrupted", "error"),
    ("utf16-input-limit", "error"),
    ("validation-failure", "error"),
    ("when-construction", "error"),
];
