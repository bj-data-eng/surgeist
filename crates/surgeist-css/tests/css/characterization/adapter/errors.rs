//! One representative of each error type with `Display`, produced through the
//! public API and keyed by a stable case id. Each row records the `Display`
//! text, the `source()` chain and the facts its accessors expose.

use std::error::Error as StdError;
use std::sync::Arc;

use surgeist_css::*;

use super::{Observation, Stop, error_chain};

pub fn observe(case: &str) -> Observation {
    let mut observation = Observation::shaped("error");
    let facts = facts(case, &mut observation);
    for fact in facts {
        observation.add_fact(fact);
    }
    observation
}

/// The `Display` chain, then the accessor facts of every link.
fn chain(error: &(dyn StdError + 'static)) -> Vec<String> {
    let mut facts = vec![format!("display={}", error_chain(error))];
    facts.extend(super::facts::accessor_facts(error));
    facts
}

/// The error chain of a front that must fail; `unexpected-ok` marks a
/// stimulus that no longer produces its error.
fn failed<T, E: StdError + 'static>(result: Result<T, E>) -> Vec<String> {
    match result {
        Ok(_) => vec!["unexpected-ok".to_owned()],
        Err(error) => chain(&error),
    }
}

fn with<E: StdError + 'static>(error: &E, extra: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut facts = chain(error);
    facts.extend(extra);
    facts
}

fn values(source: &str) -> CssComponentValues {
    parse_component_values(source).expect("characterization component values")
}

fn token(source: &str) -> CssComponentValue {
    CssComponentValue::try_token(source).expect("characterization token")
}

fn sheet(source: &str) -> CssSheet {
    parse_sheet(source).syntax().clone()
}

fn zero() -> CssSerializationLimits {
    CssSerializationLimits::new(0, 0, 0)
}

fn red() -> CssColor {
    CssColor::from_named(CssNamedColor::try_new("red").expect("named colour"))
}

fn facts(case: &str, observation: &mut Observation) -> Vec<String> {
    match case {
        "background-construction" => failed(CssBackground::try_new(vec![])),
        "character-alignment" => failed(CssCharacterAlignment::try_new("", None)),
        "color-construction" => failed(CssCustomColor::try_new(
            CssColorProfileName::try_new("--p").expect("profile name"),
            vec![],
            None,
        )),
        "color-interpolation-method" => failed(CssColorInterpolationMethod::try_new(
            surgeist_color::ColorSpace::Predefined(surgeist_color::PredefinedColorSpace::Srgb),
            Some(surgeist_color::HueInterpolation::Shorter),
        )),
        "color-mix-construction" => failed(CssColorMix::try_new(None, Vec::new())),
        "color-operation" => match unavailable_current_color() {
            CssColorOrdinaryError::Operation(error) => chain(&error),
            other => chain(&other),
        },
        "color-ordinary-limit" => {
            let facts = absent_facts();
            let error = parse_used_color(
                CssColorInput::Utf8(b"red"),
                CssColorGrammarRequest::WholeColor,
                &facts,
                facts.identity(),
                CssColorOrdinaryLimits {
                    input_units: 1,
                    algorithm_records: 1,
                    arithmetic_records: 1,
                    evaluation_records: 1,
                    actions: 1,
                },
            )
            .err()
            .expect("input-unit refusal");
            chain(&error)
        }
        "color-ordinary-operation" => chain(&unavailable_current_color()),
        "color-profile-value" => failed(CssColorProfileDescriptorValue::try_new(
            CssColorProfileDescriptorKind::RenderingIntent,
            values("banana"),
            Default::default(),
        )),
        "color-scalar" => failed(CssColorMixPercentage::try_from_component(token("150%"))),
        "color-scheme-construction" => failed(CssColorScheme::try_new(vec![], false)),
        "component-value" => failed(CssIdent::try_new("")),
        "container-construction" => failed(CssContainerCondition::try_from_components(
            values("width > "),
            Default::default(),
        )),
        "counter-style-value" => failed(CssCounterStyleDescriptorValue::try_from_components(
            CssCounterStyleDescriptorKind::System,
            values("banana"),
            Default::default(),
        )),
        "custom-media-construction" => failed(CssCustomMediaRule::try_from_components(
            values("x (color)"),
            Default::default(),
        )),
        "custom-media-serialization" => {
            let sheet = sheet("@custom-media --x (color), ???, screen;");
            let [CssRule::CustomMedia(rule)] = sheet.rules() else {
                panic!("one custom media rule")
            };
            failed(rule.serialize_tokens(Default::default()))
        }
        "declaration-block" => {
            let sheet = sheet("@keyframes k { from { opacity: 0 } }");
            let [CssRule::Keyframes(rule)] = sheet.rules() else {
                panic!("one keyframes rule")
            };
            let error = match rule.blocks()[0].declarations().serialize_cssom(zero()) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(
                &error,
                [format!(
                    "authored-ordinal={} member-ordinal={}",
                    index(error.authored_ordinal()),
                    index(error.member_ordinal())
                )],
            )
        }
        "empty-language-range-list" => failed(CssLanguageRangeList::try_new(vec![])),
        "expansion" => {
            let report = parse_style_attribute("margin: var(--x)");
            let declaration = &report.syntax().as_slice()[0];
            let CssExpansion::Pending(handle) = expand_declaration(declaration).unwrap() else {
                panic!("pending substitution")
            };
            failed(handle.reenter(values("red")))
        }
        "font-face-declaration-block" => {
            let sheet = sheet("@font-face { font-family: X; font-family: Y }");
            let [CssRule::FontFace(rule)] = sheet.rules() else {
                panic!("one font-face rule")
            };
            let entries = rule
                .descriptors()
                .occurrences()
                .cloned()
                .collect::<Vec<_>>();
            failed(CssSpecifiedFontFaceDeclarationBlock::try_from_entries(
                &entries,
                zero(),
            ))
        }
        "font-face-value" => failed(CssAuthoredFontFaceDescriptorValue::try_from_components(
            CssFontFaceDescriptorKind::FontDisplay,
            values("never"),
            Default::default(),
        )),
        "font-feature-value" => failed(CssFontFeatureValue::try_from_components(
            CssFontFeatureValueKind::Swash,
            values("1 2"),
            Default::default(),
        )),
        "font-feature-values" => {
            let error = match CssFontFeatureValueIndex::try_from_decimal("-1") {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(
                &error,
                [format!(
                    "definition-index={}",
                    index(error.definition_index())
                )],
            )
        }
        "font-palette-construction" => failed(CssFontPaletteIndex::try_new(
            CssIntegerValue::Literal(CssIntegerLiteral::from_i32(-1)),
        )),
        "font-palette-mix-construction" => failed(CssFontPaletteMix::try_new(None, Vec::new())),
        "font-palette-value" => failed(CssFontPaletteDescriptorValue::try_new(
            CssFontPaletteDescriptorKind::BasePalette,
            values("-1"),
            Default::default(),
        )),
        "general-enclosed" => failed(CssGeneralEnclosed::try_from_component(token("a"))),
        "gradient-stop-construction" => {
            let stop = || {
                CssAngularColorStopPosition::try_from_component(token("10deg"))
                    .expect("angular stop position")
            };
            failed(CssAngularColorStop::try_new(
                red(),
                vec![stop(), stop(), stop()],
            ))
        }
        "image-1d-construction" => failed(CssImage1D::try_from_components(values("stripes()"))),
        "image-construction" => failed(CssImage::try_new(CssImageValue::None)),
        "import-construction" => failed(CssImportRule::try_from_components(
            values("1px"),
            &CssNamespaceContext::default(),
            Default::default(),
        )),
        "import-serialization" => {
            let sheet = sheet("@import 'x' screen,???;");
            let [CssRule::Import(rule)] = sheet.rules() else {
                panic!("one import rule")
            };
            failed(rule.serialize_tokens(Default::default()))
        }
        "input-decode" => failed(decode_stylesheet_bytes_with_limits(
            b".a { color: red }",
            CssStylesheetDecodeHints::default(),
            CssStylesheetDecodeLimits::new(1, 1),
        )),
        "keyframe-rule-view" => {
            let report = parse_keyframe_rule("from { opacity: 0 }");
            let rule = report.syntax().as_ref().expect("keyframe rule");
            let block = CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
                rule.block().declarations(),
                Default::default(),
            )
            .expect("keyframe declarations");
            let view = CssKeyframeRuleView::try_new(rule.block().selectors(), &block)
                .expect("keyframe view");
            failed(view.serialize_cssom(zero()))
        }
        "keyframe-selector-comparison" => {
            let list = |source: &str| {
                parse_keyframe_selector_list(source)
                    .syntax()
                    .as_ref()
                    .unwrap()
                    .selectors()
                    .clone()
            };
            let error = match list("from, to").matches_normalized(&list("0%,100%"), zero()) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(
                &error,
                [format!("selector-index={}", index(error.selector_index()))],
            )
        }
        "media-construction" => failed(CssMediaQuery::try_from_components(
            values("screen and"),
            Default::default(),
        )),
        "media-cssom-serialization" => {
            let report = parse_media_query("screen and (min-width: 1px)");
            failed(report.syntax().serialize_cssom(zero()))
        }
        "media-serialization" => {
            let report = parse_media_query("screen and");
            failed(report.syntax().serialize_tokens(Default::default()))
        }
        "motion-construction" => failed(CssOffsetRotate::try_new(None, None)),
        "named-supports-construction" => {
            failed(CssSupportsConditionName::try_from_component(token("1px")))
        }
        "normalization" => {
            let error = match normalize_sheet_with_limits(
                &sheet(".a { color: red } .b { color: blue }"),
                CssNormalizationLimits::try_new(1, 1, 1, 1).expect("normalization limits"),
            ) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(
                &error,
                [format!(
                    "declaration-order={}",
                    index(error.declaration_order())
                )],
            )
        }
        "numeric-construction" => failed(CssSpecifiedLength::try_from_component(token("red"))),
        "page-block" => {
            let report = parse_declaration_block_contents("display: block");
            let error = match CssMarginDeclarationBlock::try_new(
                report.syntax().as_slice().to_vec(),
                Default::default(),
            ) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(&error, [format!("ordinal={}", index(error.ordinal()))])
        }
        "page-projection" => {
            let report = parse_page_declaration_block_contents("size: a4; size: a5; margin: 1px");
            let block = report.syntax().as_ref().expect("page declarations");
            failed(block.try_specified(zero()))
        }
        "parse-diagnostic" => {
            let report = parse_style_attribute("width: red");
            chain(report.diagnostics()[0].error())
        }
        "path-data-construction" => {
            let error = match CssPathData::try_new("M 0 0 Q") {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(
                &error,
                [format!(
                    "decoded-byte-offset={}",
                    index(error.decoded_byte_offset())
                )],
            )
        }
        "position-construction" => failed(CssPhysicalPosition::try_new(
            CssHorizontalPosition::RightOffset(
                CssSpecifiedLengthPercentage::try_from_component(token("10px")).expect("offset"),
            ),
            CssVerticalPosition::Top,
        )),
        "property-metadata" => {
            // The first grammar, canonical or legacy, whose metadata is
            // unavailable; none means the error is currently unreachable.
            let first = CssKnownProperty::all()
                .iter()
                .flat_map(|property| {
                    std::iter::once(property.grammar())
                        .chain(property.legacy_shorthands().iter().copied())
                })
                .find_map(|grammar| grammar.metadata().err());
            match first {
                Some(error) => chain(&error),
                None => vec!["unreached=every grammar has metadata".to_owned()],
            }
        }
        "property-value-parse" => failed(parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            values("red"),
            CssImportance::Normal,
        )),
        "property-value-syntax" => {
            let limits = CssUtf16PropertyValueLimits {
                input: CssUtf16InputLimits::default(),
                syntax: CssComponentValueLimits::default(),
                max_work: 4_000_000,
                max_live_bytes: 16 * 1024 * 1024,
                max_retained_bytes: 16 * 1024 * 1024,
            };
            let report = CssParserContext::default()
                .parse_color_scheme_utf16_with_limits(
                    Arc::new("light inherit".encode_utf16().collect()),
                    limits,
                    |_| Ok::<(), Stop>(()),
                    |_| {},
                )
                .expect("prepared colour scheme");
            match report.result() {
                Ok(_) => vec!["unexpected-ok".to_owned()],
                Err(error) => chain(error),
            }
        }
        "qualified-name" => failed(CssQualifiedAttributeName::try_new(
            CssQualifiedNamePrefix::Named(
                CssNamespacePrefix::try_new("svg").expect("namespace prefix"),
            ),
            CssAttributeName::try_new("href").expect("attribute name"),
            &CssNamespaceContext::default(),
        )),
        "retained-payload" => {
            let limits = CssUtf16PropertyValueLimits {
                input: CssUtf16InputLimits::default(),
                syntax: CssComponentValueLimits::default(),
                max_work: 4_000_000,
                max_live_bytes: 16 * 1024 * 1024,
                max_retained_bytes: 16 * 1024 * 1024,
            };
            let parse = || {
                CssParserContext::default()
                    .parse_color_scheme_utf16_with_limits(
                        Arc::new("light dark".encode_utf16().collect()),
                        limits,
                        |_| Ok::<(), Stop>(()),
                        |_| {},
                    )
                    .expect("prepared colour scheme")
            };
            let error = match parse().equivalent_with(&parse(), || Err::<(), Stop>(Stop)) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            chain(&error)
        }
        "rule-construction" => {
            let rules = sheet(".a {} @import url(a.css);").rules().to_vec();
            let late = sheet("@import url(a.css);").rules().to_vec();
            let error = match CssSheet::try_from_rules(rules.into_iter().chain(late).collect()) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(&error, [format!("path={}", path(error.path()))])
        }
        "rule-cssom-serialization" => {
            let error = match sheet(".a { color: red }").serialize_cssom(zero()) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(
                &error,
                [format!(
                    "rule-path={} keyframe-block-index={}",
                    path(error.rule_path()),
                    index(error.keyframe_block_index())
                )],
            )
        }
        "selector-construction" => failed(CssSelectorList::try_new(Vec::new(), Default::default())),
        "shape-construction" => failed(CssShapeCommandList::try_new(Vec::new())),
        "specified-rule-serialization" => {
            let error = match sheet(".a { color: red }").serialize_specified(zero()) {
                Ok(_) => return vec!["unexpected-ok".to_owned()],
                Err(error) => error,
            };
            with(
                &error,
                [format!("rule-index={}", index(error.rule_index()))],
            )
        }
        "specified-value-serialization" => {
            let report = parse_style_attribute("color: red");
            failed(report.syntax().as_slice()[0].serialize_specified(zero()))
        }
        "supports-construction" => failed(parse_css_supports_condition("display: grid (")),
        "utf16-input-interrupted" => {
            failed(CssParserContext::default().parse_sheet_utf16_with_limits(
                Arc::new(".a { color: red }".encode_utf16().collect()),
                CssUtf16InputLimits::default(),
                || Err(Stop),
            ))
        }
        "utf16-input-limit" => failed(CssParserContext::default().parse_sheet_utf16_with_limits(
            Arc::new(".a { color: red }".encode_utf16().collect()),
            CssUtf16InputLimits {
                max_input_units: 1,
                ..CssUtf16InputLimits::default()
            },
            || Ok::<(), Stop>(()),
        )),
        "when-construction" => failed(parse_when_condition(")")),
        other => {
            observation.shape = "unknown".to_owned();
            panic!("unknown error case `{other}`")
        }
    }
}

fn index(value: Option<usize>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn path(values: &[usize]) -> String {
    values
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

fn absent_facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(7, 3),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )
}

fn unavailable_current_color() -> CssColorOrdinaryError {
    let facts = absent_facts();
    parse_used_color(
        CssColorInput::Utf8(b"currentcolor"),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits {
            input_units: 1024,
            algorithm_records: 16_384,
            arithmetic_records: 4096,
            evaluation_records: 4096,
            actions: 4_000_000,
        },
    )
    .err()
    .expect("unavailable current colour")
}
