//! One representative of each error type with `Display`, produced through the
//! public API and keyed by a stable case id. Each row records the `Display`
//! text, the `source()` chain and the facts its accessors expose.

use std::error::Error as StdError;
use std::sync::Arc;

use surgeist_css::CssParser;
use surgeist_css::color::{
    CssColor, CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInput,
    CssColorInterpolationMethod, CssColorMix, CssColorMixPercentage, CssColorOrdinaryLimits,
    CssColorProfileName, CssColorPropertyFacts, CssColorResolveError, CssColorSchemeFact,
    CssCustomColor, CssDeviceColorFact, CssNamedColor, parse_used_color,
};
use surgeist_css::component::{CssComponentValue, CssComponentValues};
use surgeist_css::declaration::{CssDeclaration, CssImportance, CssSpecifiedDeclarationBlock};
use surgeist_css::normalize::CssNormalizationLimits;
use surgeist_css::numeric::{
    CssInteger, CssLength, CssLengthPercentage, CssNonNegative, CssNumeric, CssNumericLiteral,
};
use surgeist_css::property::{
    CssExpansion, CssKnownProperty, CssPropertyNameRef, CssPropertyTarget,
};
use surgeist_css::query::container::CssContainerLeaf;
use surgeist_css::query::media::CssMediaQuery;
use surgeist_css::query::supports::{CssSupportsConditionName, CssSupportsLeaf};
use surgeist_css::query::{CssCondition, CssGeneralEnclosed, CssWhenLeaf};
use surgeist_css::rule::color_profile::{
    CssColorProfileDescriptorKind, CssColorProfileDescriptorValue,
};
use surgeist_css::rule::counter_style::{
    CssCounterStyleDescriptorKind, CssCounterStyleDescriptorValue,
};
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind,
    CssSpecifiedFontFaceDeclarationBlock,
};
use surgeist_css::rule::font_feature_values::{
    CssFontFeatureValue, CssFontFeatureValueIndex, CssFontFeatureValueKind,
};
use surgeist_css::rule::font_palette_values::{
    CssFontPaletteDescriptorKind, CssFontPaletteDescriptorValue,
};
use surgeist_css::rule::keyframes::{
    CssKeyframeRuleView, CssParsedKeyframeRule, CssParsedKeyframeSelectors,
};
use surgeist_css::rule::page::CssMarginDeclarationBlock;
use surgeist_css::rule::{
    CssCustomMediaRule, CssImportRule, CssNamespacePrefix, CssRule, CssSheet,
};
use surgeist_css::selector::{
    CssAttributeName, CssLanguageRangeList, CssNamespaceContext, CssQualifiedAttributeName,
    CssQualifiedNamePrefix, CssSelectorList,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::source::{
    CssDecodedStylesheetInput, CssStylesheetDecodeHints, CssStylesheetDecodeLimits,
    CssUtf16InputLimits, CssUtf16PropertyValueLimits,
};
use surgeist_css::values::background::CssBackground;
use surgeist_css::values::color_adjust::CssColorScheme;
use surgeist_css::values::common::CssIdent;
use surgeist_css::values::font::CssFontPaletteMix;
use surgeist_css::values::image::{
    CssAngularColorStop, CssAngularColorStopPosition, CssImage, CssImage1D, CssImageValue,
};
use surgeist_css::values::layout::CssGridTemplateAreas;
use surgeist_css::values::position::{
    CssHorizontalPosition, CssPhysicalPosition, CssVerticalPosition,
};
use surgeist_css::values::shape::{CssPathData, CssShapeCommandList};
use surgeist_css::values::text::CssCharacterAlignment;
use surgeist_css::values::transform::CssOffsetRotate;

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
    CssComponentValues::parse(&CssParser::default(), source)
        .expect("characterization component values")
}

fn token(source: &str) -> CssComponentValue {
    CssComponentValue::try_token(source).expect("characterization token")
}

fn sheet(source: &str) -> CssSheet {
    CssParser::default().parse_sheet(source).syntax().clone()
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
        "color-operation" => {
            let error = unavailable_current_color();
            match error.color_error() {
                Some(operation) => chain(operation),
                None => chain(&error),
            }
        }
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
        "container-construction" => failed(CssCondition::<CssContainerLeaf>::try_from_components(
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
            let report = CssParser::default().parse_block_contents("margin: var(--x)");
            let declaration = &report.syntax().as_slice()[0];
            let CssExpansion::Pending(handle) = declaration.expand().unwrap() else {
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
        // A palette index is `<integer [0,∞]>`; its range is the numeric
        // model's, so a negative index is a numeric range rejection.
        "font-palette-construction" => {
            failed(CssNumeric::<CssInteger, CssNonNegative>::try_from_literal(
                CssNumericLiteral::from_i32(-1),
            ))
        }
        "font-palette-mix-construction" => failed(CssFontPaletteMix::try_new(None, Vec::new())),
        "font-palette-value" => failed(CssFontPaletteDescriptorValue::try_new(
            CssFontPaletteDescriptorKind::BasePalette,
            values("-1"),
            Default::default(),
        )),
        "general-enclosed" => failed(CssGeneralEnclosed::try_from_component(token("a"))),
        "grid-template-area" => failed(CssGridTemplateAreas::try_rows(vec![])),
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
        "input-decode" => failed(CssDecodedStylesheetInput::decode(
            b".a { color: red }",
            CssStylesheetDecodeHints::default(),
            CssStylesheetDecodeLimits::new(1, 1),
        )),
        "keyframe-rule-view" => {
            let report = CssParsedKeyframeRule::parse(&CssParser::default(), "from { opacity: 0 }");
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
                CssParsedKeyframeSelectors::parse(&CssParser::default(), source)
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
            let report = CssMediaQuery::parse(&CssParser::default(), "screen and (min-width: 1px)");
            failed(report.syntax().serialize_cssom(zero()))
        }
        "media-serialization" => {
            let report = CssMediaQuery::parse(&CssParser::default(), "screen and");
            failed(report.syntax().serialize_tokens(Default::default()))
        }
        "motion-construction" => failed(CssOffsetRotate::try_new(None, None)),
        "named-supports-construction" => {
            failed(CssSupportsConditionName::try_from_component(token("1px")))
        }
        "normalization" => {
            let error = match sheet(".a { color: red } .b { color: blue }").normalize(
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
        "numeric-construction" => failed(CssNumeric::<CssLength>::try_from_component(token("red"))),
        "page-block" => {
            let report = CssParser::default().parse_block_contents("display: block");
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
            let report =
                CssParser::default().parse_page_block_contents("size: a4; size: a5; margin: 1px");
            let block = report.syntax().as_ref().expect("page declarations");
            failed(block.try_specified(zero()))
        }
        "parse-diagnostic" => {
            let report = CssParser::default().parse_block_contents("width: red");
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
                CssNumeric::<CssLengthPercentage>::try_from_component(token("10px"))
                    .expect("offset"),
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
        "property-value-parse" => failed(CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Width)),
            values("red"),
            CssImportance::Normal,
        )),
        "property-value-syntax" => {
            let limits = CssUtf16PropertyValueLimits {
                input: CssUtf16InputLimits::default(),
                max_work: 4_000_000,
                max_live_bytes: 16 * 1024 * 1024,
                max_retained_bytes: 16 * 1024 * 1024,
            };
            let report = CssColorScheme::parse_utf16(
                &CssParser::default(),
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
                max_work: 4_000_000,
                max_live_bytes: 16 * 1024 * 1024,
                max_retained_bytes: 16 * 1024 * 1024,
            };
            let parse = || {
                CssColorScheme::parse_utf16(
                    &CssParser::default(),
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
            let report = CssParser::default().parse_block_contents("color: red");
            failed(report.syntax().as_slice()[0].serialize_specified(zero()))
        }
        "supports-construction" => failed(CssCondition::<CssSupportsLeaf>::parse_condition_text(
            &CssParser::default(),
            "display: grid (",
        )),
        "utf16-input-interrupted" => failed(CssParser::default().parse_sheet_utf16(
            Arc::new(".a { color: red }".encode_utf16().collect()),
            CssUtf16InputLimits::default(),
            || Err(Stop),
        )),
        "utf16-input-limit" => failed(CssParser::default().parse_sheet_utf16(
            Arc::new(".a { color: red }".encode_utf16().collect()),
            CssUtf16InputLimits {
                max_input_units: 1,
                ..CssUtf16InputLimits::default()
            },
            || Ok::<(), Stop>(()),
        )),
        "validation-failure" => failed(
            CssParser::default()
                .parse_block_contents("width: red")
                .into_validation_result(),
        ),
        "when-construction" => failed(CssCondition::<CssWhenLeaf>::parse(
            &CssParser::default(),
            ")",
        )),
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

fn unavailable_current_color() -> CssColorResolveError {
    let facts = absent_facts();
    parse_used_color(
        CssColorInput::Utf8(b"currentcolor"),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits::default(),
    )
    .err()
    .expect("unavailable current colour")
}
