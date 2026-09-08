use super::*;
use crate::test_support::{
    AcceptedDeclarationCase, AcceptedValueCase, ExpectedErrorKind, RejectedDeclarationCase,
    RejectedSheetCase, assert_accepts_declarations, assert_accepts_value_cases,
    assert_rejects_declarations, assert_rejects_sheets, assert_sheet_rejected,
    parse_single_declaration,
};

fn source_position(line: u32, column: u32) -> CssSourcePosition {
    let source = format!(
        "{}{}",
        "\n".repeat(line as usize),
        " ".repeat(column as usize)
    );
    let mut input = cssparser::ParserInput::new(&source);
    let mut parser = cssparser::Parser::new(&mut input);
    while parser.next().is_ok() {}
    CssSourcePosition::from_cssparser(parser.position(), parser.current_source_location())
}

fn test_media_position() -> CssSourcePosition {
    source_position(0, 0)
}

fn style_rule(rule: &CssRule) -> &CssStyleRule {
    match rule {
        CssRule::Style(rule) => rule,
        unexpected => panic!("expected style rule, got {unexpected:?}"),
    }
}

fn declaration(input: &str, property: CssProperty) -> CssDeclaration {
    let sheet = parse_sheet(input).unwrap();
    style_rule(&sheet.rules()[0])
        .declarations()
        .iter()
        .find(|declaration| declaration.property() == property)
        .unwrap()
        .clone()
}

macro_rules! declaration_value {
    ($input:expr, $variant:ident) => {{
        let declaration = declaration($input, CssProperty::$variant);
        let value = declaration
            .known()
            .and_then(|known| known.property_value())
            .expect("ordinary declaration value");
        let CssKnownPropertyValueRef::$variant(value) = value else {
            panic!("property wrapper did not match requested property");
        };
        value.i01_subset().expect("I01 property payload").clone()
    }};
}

macro_rules! single_declaration_value {
    ($property_name:expr, $variant:ident, $authored_value:expr) => {{
        let declaration = parse_single_declaration($property_name, $authored_value);
        let value = declaration
            .known()
            .and_then(|known| known.property_value())
            .expect("ordinary declaration value");
        let CssKnownPropertyValueRef::$variant(value) = value else {
            panic!("property wrapper did not match requested property");
        };
        value.i01_subset().expect("I01 property payload").clone()
    }};
}

macro_rules! declaration_payload {
    ($declaration:expr, $variant:ident) => {{
        let declaration = &$declaration;
        let value = declaration
            .known()
            .and_then(|known| known.property_value())
            .expect("ordinary declaration value");
        let CssKnownPropertyValueRef::$variant(value) = value else {
            panic!("property wrapper did not match requested property");
        };
        value.i01_subset().expect("I01 property payload").clone()
    }};
}

macro_rules! declaration_global {
    ($input:expr, $variant:ident) => {{
        let declaration = declaration($input, CssProperty::$variant);
        declaration
            .known()
            .and_then(|known| known.global())
            .expect("global declaration value")
    }};
}

fn custom_property_value(declaration: &CssDeclaration) -> &CssCustomPropertyValue {
    declaration
        .custom()
        .expect("custom declaration")
        .value()
        .value()
        .expect("authored custom property value")
}

fn parse_media_query_list_for_test(input: &str) -> std::result::Result<CssMediaQueryList, Error> {
    crate::parser::parse_media_query_list_for_test(input)
}

fn parse_container_condition_for_test(
    input: &str,
) -> std::result::Result<CssContainerCondition, Error> {
    crate::parser::parse_container_condition_for_test(input)
}

fn media_rule(rule: &CssRule) -> &CssMediaRule {
    match rule {
        CssRule::Media(rule) => rule,
        unexpected => panic!("expected media rule, got {unexpected:?}"),
    }
}

fn container_rule(rule: &CssRule) -> &CssContainerRule {
    match rule {
        CssRule::Container(rule) => rule,
        unexpected => panic!("expected container rule, got {unexpected:?}"),
    }
}

fn import_rule(rule: &CssRule) -> &CssImportRule {
    match rule {
        CssRule::Import(rule) => rule,
        unexpected => panic!("expected import rule, got {unexpected:?}"),
    }
}

fn layer_statement_rule(rule: &CssRule) -> &CssLayerStatementRule {
    match rule {
        CssRule::LayerStatement(rule) => rule,
        unexpected => panic!("expected layer statement rule, got {unexpected:?}"),
    }
}

fn layer_block_rule(rule: &CssRule) -> &CssLayerBlockRule {
    match rule {
        CssRule::LayerBlock(rule) => rule,
        unexpected => panic!("expected layer block rule, got {unexpected:?}"),
    }
}

fn scope_rule(rule: &CssRule) -> &CssScopeRule {
    match rule {
        CssRule::Scope(rule) => rule,
        unexpected => panic!("expected scope rule, got {unexpected:?}"),
    }
}

#[test]
fn cssparser_color_dependency_is_available_for_color_parsing() {
    let mut input = cssparser::ParserInput::new("rgb(255 0 0 / 50%)");
    let mut parser = cssparser::Parser::new(&mut input);
    let parsed =
        cssparser_color::parse_color_with(&cssparser_color::DefaultColorParser, &mut parser);

    assert!(parsed.is_ok());
}

#[test]
fn parses_cssparser_color_absolute_forms() {
    let cases = [
        ("red", "rgba"),
        ("rebeccapurple", "rgba"),
        ("transparent", "rgba"),
        ("currentcolor", "currentcolor"),
        ("#abcd", "rgba"),
        ("#11223344", "rgba"),
        ("rgb(255 0 0 / 50%)", "rgba"),
        ("rgba(255, 0, 0, 0.5)", "rgba"),
        ("hsl(120deg 100% 25% / 0.75)", "hsl"),
        ("hwb(90 10% 20% / 0.8)", "hwb"),
        ("lab(50% 20 -30 / 0.9)", "lab"),
        ("lch(60% 40 120deg)", "lch"),
        ("oklab(0.6 0.1 -0.1)", "oklab"),
        ("oklch(0.7 0.2 240deg / 80%)", "oklch"),
        ("color(display-p3 0.8 0.2 0.1 / 0.9)", "color"),
        ("color(display-p3-linear 0.8 0.2 0.1 / 0.9)", "color"),
    ];

    for (value, expected_kind) in cases {
        let css = format!(".panel {{ color: {value}; }}");
        let value = declaration_value!(&css, Color);
        let color = value;
        assert_eq!(color.kind_name(), expected_kind, "{css}");
    }
}

#[test]
fn parses_css_system_colors_symbolically() {
    let color = declaration_value!(".panel { color: CanvasText; }", Color);
    assert_eq!(color, CssColor::System(CssSystemColor::CanvasText));

    let color = declaration_value!(".panel { background-color: Canvas; }", BackgroundColor);
    assert_eq!(color, CssColor::System(CssSystemColor::Canvas));

    let color = declaration_value!(".panel { border-color: AccentColor; }", BorderColor);
    assert_eq!(color, CssColor::System(CssSystemColor::AccentColor));

    let color = declaration_value!(".panel { outline-color: HighlightText; }", OutlineColor);
    assert_eq!(color, CssColor::System(CssSystemColor::HighlightText));
}

#[test]
fn rejects_unknown_system_color_like_identifiers() {
    assert!(parse_sheet(".panel { color: MadeUpSystemColor; }").is_err());
    assert!(parse_sheet(".panel { color: PlatformAccent; }").is_err());
}

#[test]
fn parses_color_mix_symbolically() {
    let CssColor::ColorMix(mix) = declaration_value!(
        ".panel { color: color-mix(in oklch, red 40%, blue); }",
        Color
    ) else {
        panic!("expected color-mix");
    };

    assert_eq!(
        mix.interpolation().space(),
        CssColorInterpolationSpace::Oklch
    );
    assert_eq!(mix.interpolation().hue(), None);
    assert_eq!(mix.left().percentage(), Some(40.0));
    assert!(matches!(mix.left().color(), CssColor::Rgba(_)));
    assert_eq!(mix.right().percentage(), None);
    assert!(matches!(mix.right().color(), CssColor::Rgba(_)));
}

#[test]
fn parses_color_mix_with_hue_interpolation() {
    let CssColor::ColorMix(mix) = declaration_value!(
        ".panel { color: color-mix(in lch longer hue, red, blue 25%); }",
        Color
    ) else {
        panic!("expected color-mix");
    };

    assert_eq!(mix.interpolation().space(), CssColorInterpolationSpace::Lch);
    assert_eq!(
        mix.interpolation().hue(),
        Some(CssHueInterpolationMethod::Longer)
    );
    assert_eq!(mix.right().percentage(), Some(25.0));
}

#[test]
fn rejects_invalid_color_mix_forms_strictly() {
    assert!(parse_sheet(".panel { color: color-mix(); }").is_err());
    assert!(parse_sheet(".panel { color: color-mix(in oklch, red); }").is_err());
    assert!(parse_sheet(".panel { color: color-mix(in unknown, red, blue); }").is_err());
    assert!(parse_sheet(".panel { color: color-mix(in oklch, red 10% 20%, blue); }").is_err());
}

#[test]
fn parses_relative_colors_symbolically() {
    let CssColor::Relative(relative) =
        declaration_value!(".panel { color: rgb(from red r g b / alpha); }", Color)
    else {
        panic!("expected relative color");
    };

    assert_eq!(relative.function(), &CssRelativeColorFunction::Rgb);
    assert!(matches!(relative.source(), CssColor::Rgba(_)));
    assert_eq!(relative.components().len(), 3);
    assert_eq!(relative.components()[0].authored().as_css(), "r");
    assert_eq!(relative.alpha().unwrap().authored().as_css(), "alpha");
}

#[test]
fn parses_relative_oklch_with_component_expressions() {
    let CssColor::Relative(relative) = declaration_value!(
        ".panel { color: oklch(from red l c calc(h + 20deg) / 80%); }",
        Color
    ) else {
        panic!("expected relative color");
    };

    assert_eq!(relative.function(), &CssRelativeColorFunction::Oklch);
    assert_eq!(relative.components().len(), 3);
    assert_eq!(
        relative.components()[2].authored().as_css(),
        "calc(h + 20deg)"
    );
}

#[test]
fn rejects_invalid_relative_color_forms_strictly() {
    assert!(parse_sheet(".panel { color: rgb(from red r g); }").is_err());
    assert!(parse_sheet(".panel { color: rgb(from red r g b /); }").is_err());
    assert!(parse_sheet(".panel { color: rgb(from red r g b extra); }").is_err());
    assert!(parse_sheet(".panel { color: hsl(from red h s); }").is_err());
    assert!(parse_sheet(".panel { color: rgb(from red foo(r) g b); }").is_err());
    assert!(parse_sheet(".panel { color: rgb(from red calc() g b); }").is_err());
    assert!(parse_sheet(".panel { color: rgb(from red calc(h +) g b); }").is_err());
}

#[test]
fn rgba_hex_alpha_preserves_channels() {
    let color = declaration_value!(".panel { color: #11223344; }", Color);
    let rgba = color.as_rgba().unwrap();
    assert_eq!((rgba.red(), rgba.green(), rgba.blue()), (0x11, 0x22, 0x33));
    assert!((rgba.alpha() - (0x44 as f32 / 255.0)).abs() < 0.0001);
}

#[test]
fn rejects_non_finite_cssparser_color_components() {
    for css in [
        ".panel { color: hsl(1e999 100% 50%); }",
        ".panel { color: hwb(1e999 10% 20%); }",
        ".panel { color: lab(50% 1e999 0); }",
        ".panel { color: lch(50% 1e999 0); }",
        ".panel { color: oklab(0.5 1e999 0); }",
        ".panel { color: oklch(0.5 1e999 0); }",
        ".panel { color: color(display-p3 1e999 0 0 / 1); }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn color_surface_accepts_expanded_strict_forms() {
    for css in [
        ".panel { color: red; }",
        ".panel { background-color: rebeccapurple; }",
        ".panel { border-color: #11223344; }",
        ".panel { outline-color: rgb(255 0 0 / 50%); }",
        ".panel { text-decoration-color: oklch(0.7 0.2 240deg / 80%); }",
        ".panel { box-shadow: 0 1px 2px color(display-p3 0.8 0.2 0.1 / 0.9); }",
        ".panel { color: CanvasText; }",
        ".panel { color: color-mix(in oklch, red 40%, blue); }",
        ".panel { color: rgb(from red r g b / alpha); }",
    ] {
        parse_sheet(css).unwrap_or_else(|error| panic!("{css} should parse: {error:?}"));
    }
}

#[test]
fn color_surface_rejects_invalid_forms_without_recovery() {
    for css in [
        ".panel { color: rgb(1, 2 3); }",
        ".panel { color: color(unknown-space 1 0 0); }",
        ".panel { color: MadeUpSystemColor; }",
        ".panel { color: color-mix(); }",
        ".panel { color: color-mix(in oklch, red); }",
        ".panel { color: rgb(from red r g); }",
        ".panel { color: rgb(from red r g b /); }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject strictly");
    }
}

fn font_face_rule(rule: &CssRule) -> &CssFontFaceRule {
    match rule {
        CssRule::FontFace(rule) => rule,
        unexpected => panic!("expected font-face rule, got {unexpected:?}"),
    }
}

fn keyframes_rule(rule: &CssRule) -> &CssKeyframesRule {
    match rule {
        CssRule::Keyframes(rule) => rule,
        unexpected => panic!("expected keyframes rule, got {unexpected:?}"),
    }
}

fn descriptor_occurrence<T>(value: T) -> CssDescriptorOccurrence<T> {
    CssDescriptorOccurrence::new(value, source_position(0, 0))
}

#[test]
fn keyframes_rule_accessors_expose_authored_structure() {
    let name = CssKeyframesName::Ident(CssCustomIdent::new("fade"));
    let selector = CssKeyframeSelectorList::try_new(vec![CssKeyframeSelector::From]).unwrap();
    let keyframes = parse_sheet("@keyframes fade { from { opacity: 0; } }").unwrap();
    let declaration = keyframes_rule(&keyframes.rules()[0]).blocks()[0].declarations()[0].clone();
    let block = CssKeyframeBlock::new(
        selector,
        CssKeyframeDeclarationList::new(vec![declaration.clone()]),
        source_position(2, 3),
    );
    let rule = CssKeyframesRule::new(name, vec![block], source_position(1, 1));

    assert_eq!(
        rule.name(),
        &CssKeyframesName::Ident(CssCustomIdent::new("fade"))
    );
    assert_eq!(rule.position(), source_position(1, 1));
    let [block] = rule.blocks() else {
        panic!("expected one keyframe block");
    };
    assert_eq!(block.position(), source_position(2, 3));
    assert_eq!(block.selectors().selectors(), &[CssKeyframeSelector::From]);
    assert_eq!(block.declarations().as_slice(), &[declaration]);
    assert_eq!(CssKeyframeSelector::From.offset().value().value(), 0.0);
    assert_eq!(CssKeyframeSelector::To.offset().value().value(), 100.0);
}

#[test]
fn keyframes_rule_parser_accepts_strict_blocks() {
    let sheet = parse_sheet(
        r#"@keyframes fade {
            from { opacity: 0; transform: translateX(0px); }
            50% { opacity: 0.5; }
            to { opacity: 1; transform: translateX(10px); }
        }"#,
    )
    .unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one keyframes rule");
    };
    let rule = keyframes_rule(rule);

    assert_eq!(
        rule.name(),
        &CssKeyframesName::Ident(CssCustomIdent::new("fade"))
    );
    assert_eq!(rule.blocks().len(), 3);
    assert_eq!(
        rule.blocks()[0].selectors().selectors(),
        &[CssKeyframeSelector::From]
    );
    assert_eq!(
        rule.blocks()[1].selectors().selectors(),
        &[CssKeyframeSelector::Percent(CssKeyframePercent::new(50.0))]
    );
    assert_eq!(
        rule.blocks()[0].declarations()[0].property(),
        &CssProperty::Opacity
    );
}

#[test]
fn keyframes_rule_parser_accepts_string_names_and_selector_lists() {
    let sheet = parse_sheet(
        r#"@keyframes "fade in" {
            0%, 100% { opacity: 1; }
        }
        .panel { animation-name: "fade in"; animation: "fade in" 120ms ease; }"#,
    )
    .unwrap();
    let [keyframes, style] = sheet.rules() else {
        panic!("expected keyframes and style rules");
    };

    assert_eq!(
        keyframes_rule(keyframes).name(),
        &CssKeyframesName::String(CssKeyframesString::new("fade in"))
    );
    assert_eq!(
        keyframes_rule(keyframes).blocks()[0]
            .selectors()
            .selectors(),
        &[
            CssKeyframeSelector::Percent(CssKeyframePercent::new(0.0)),
            CssKeyframeSelector::Percent(CssKeyframePercent::new(100.0)),
        ]
    );

    let declarations = style_rule(style).declarations();
    assert_eq!(declarations[0].property(), &CssProperty::AnimationName);
    assert_eq!(declarations[1].property(), &CssProperty::Animation);

    let names = declaration_payload!(declarations[0], AnimationName);
    assert_eq!(
        names.names(),
        &[CssAnimationName::String(CssKeyframesString::new("fade in"))]
    );

    let animations = declaration_payload!(declarations[1], Animation);
    assert_eq!(
        animations.items()[0].name(),
        Some(&CssAnimationName::String(CssKeyframesString::new(
            "fade in"
        )))
    );
}

#[test]
fn keyframes_rule_parser_accepts_keyframes_inside_conditional_groups() {
    let sheet = parse_sheet(
        r#"@media screen {
            @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
        }
        @container sidebar (inline-size > 30rem) {
            @keyframes slide { 0% { transform: translateX(0px); } 100% { transform: translateX(10px); } }
        }"#,
    )
    .unwrap();

    let [media, container] = sheet.rules() else {
        panic!("expected media and container rules");
    };
    let [media_keyframes] = media_rule(media).rules() else {
        panic!("expected keyframes inside media");
    };
    let [container_keyframes] = container_rule(container).rules() else {
        panic!("expected keyframes inside container");
    };

    assert_eq!(
        keyframes_rule(media_keyframes).name(),
        &CssKeyframesName::Ident(CssCustomIdent::new("fade"))
    );
    assert_eq!(
        keyframes_rule(container_keyframes).name(),
        &CssKeyframesName::Ident(CssCustomIdent::new("slide"))
    );
}

#[test]
fn keyframes_rule_parser_rejects_invalid_names_selectors_and_placements() {
    for css in [
        "@keyframes fade;",
        "@keyframes { from { opacity: 0; } }",
        "@keyframes none { from { opacity: 0; } }",
        r#"@keyframes "" { from { opacity: 0; } }"#,
        "@keyframes fade { 0 { opacity: 0; } }",
        "@keyframes fade { -1% { opacity: 0; } }",
        "@keyframes fade { 101% { opacity: 0; } }",
        "@keyframes fade { from { .nested { opacity: 0; } } }",
        "@keyframes fade { from { @media screen { opacity: 0; } } }",
        ".panel { @keyframes fade { from { opacity: 0; } } }",
        r#".panel { animation-name: ""; }"#,
        r#".panel { animation: "" 120ms ease; }"#,
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn keyframes_constructors_preserve_authored_empty_and_duplicate_states() {
    let location = source_position(1, 1);
    let name = CssKeyframesName::Ident(CssCustomIdent::new("fade"));
    let from = CssKeyframeSelectorList::try_new(vec![CssKeyframeSelector::From]).unwrap();

    assert_eq!(CssKeyframesString::try_new(""), None);
    assert_eq!(CssKeyframesString::try_new("   "), None);
    assert_eq!(CssKeyframePercent::try_new(-0.1), None);
    assert_eq!(CssKeyframePercent::try_new(100.1), None);
    assert_eq!(CssKeyframePercent::try_new(f32::NAN), None);
    assert_eq!(CssKeyframeSelectorList::try_new(Vec::new()), None);
    let duplicate_selectors = CssKeyframeSelectorList::try_new(vec![
        CssKeyframeSelector::From,
        CssKeyframeSelector::Percent(CssKeyframePercent::new(0.0)),
    ])
    .unwrap();
    assert_eq!(duplicate_selectors.selectors().len(), 2);

    let duplicate_a = CssKeyframeBlock::new(
        from.clone(),
        CssKeyframeDeclarationList::new(Vec::new()),
        location,
    );
    let duplicate_b =
        CssKeyframeBlock::new(from, CssKeyframeDeclarationList::new(Vec::new()), location);
    let rule = CssKeyframesRule::new(name, vec![duplicate_a, duplicate_b], location);
    assert_eq!(rule.blocks().len(), 2);
    assert!(
        rule.blocks()
            .iter()
            .all(|block| block.declarations().is_empty())
    );

    let empty = CssKeyframesRule::new(
        CssKeyframesName::Ident(CssCustomIdent::new("empty")),
        Vec::new(),
        location,
    );
    assert!(empty.blocks().is_empty());
}

#[test]
fn import_layer_name_rejects_empty_components() {
    assert!(CssLayerName::try_new(["theme"]).is_some());
    assert!(CssLayerName::try_new(["theme", "components"]).is_some());
    assert!(CssLayerName::try_new([""]).is_none());
    assert!(CssLayerName::try_new(["theme", ""]).is_none());
    assert!(CssLayerName::try_new(["theme", " \t\n "]).is_none());
}

#[test]
fn import_layer_name_rejects_non_identifier_components() {
    assert!(CssLayerName::try_new(["theme components"]).is_none());
    assert!(CssLayerName::try_new(["theme.components"]).is_none());
    assert!(CssLayerName::try_new(["theme;"]).is_none());
    assert!(CssLayerName::try_new(["1theme"]).is_none());
}

#[test]
fn import_layer_name_rejects_reserved_components() {
    assert!(CssLayerName::try_new(["inherit"]).is_none());
    assert!(CssLayerName::try_new(["theme", "initial"]).is_none());
    assert!(CssLayerName::try_new(["theme", "unset"]).is_none());
    assert!(CssLayerName::try_new(["theme", "revert"]).is_none());
    assert!(CssLayerName::try_new(["theme", "revert-layer"]).is_none());
}

#[test]
fn layer_rule_models_preserve_authored_statement_and_block_shapes() {
    let reset = CssLayerName::try_new(["reset"]).unwrap();
    let components = CssLayerName::try_new(["theme", "components"]).unwrap();
    let names = CssLayerNameList::try_new(vec![reset.clone(), components.clone()]).unwrap();
    let statement_location = source_position(2, 3);
    let statement = CssLayerStatementRule::new(names.clone(), statement_location);

    assert_eq!(CssLayerNameList::try_new(Vec::new()), None);
    assert_eq!(statement.names().names(), names.names());
    assert_eq!(statement.position(), statement_location);
    assert_eq!(
        CssRule::LayerStatement(statement.clone()),
        CssRule::LayerStatement(statement)
    );

    let nested_location = source_position(4, 1);
    let nested = CssRule::Style(CssStyleRule::new(
        CssSelector::Class("button".to_owned()),
        CssDeclarationList::new(Vec::new()),
        nested_location,
    ));
    let block_location = source_position(4, 5);
    let named_block = CssLayerBlockRule::new(
        Some(components.clone()),
        vec![nested.clone()],
        block_location,
    );
    assert_eq!(style_rule(&nested).position(), nested_location);
    assert_eq!(named_block.name(), Some(&components));
    assert_eq!(named_block.rules(), &[nested]);
    assert_eq!(named_block.position(), block_location);

    let anonymous_block = CssLayerBlockRule::new(None, Vec::new(), block_location);
    assert_eq!(anonymous_block.name(), None);
    assert!(anonymous_block.rules().is_empty());
    assert_eq!(
        CssRule::LayerBlock(anonymous_block.clone()),
        CssRule::LayerBlock(anonymous_block)
    );
}

#[test]
fn scope_rule_model_keeps_scoped_selectors_and_rules_separate() {
    assert_eq!(CssScopeSelectorList::try_new(Vec::new()), None);
    assert_eq!(CssScopedStyleSelectorList::try_new(Vec::new()), None);

    let root_selector = CssSelector::Class("card".to_owned());
    let root = CssScopeSelectorList::try_new(vec![root_selector.clone()]).unwrap();
    let limit_selector = CssSelector::Class("boundary".to_owned());
    let limit = CssScopeSelectorList::try_new(vec![limit_selector.clone()]).unwrap();

    let implicit_selector =
        CssScopedStyleSelector::Selector(CssSelector::Class("title".to_owned()));
    let relative = CssRelativeSelector::new(
        CssSelectorCombinator::Child,
        CssSelector::Class("action".to_owned()),
    );
    let relative_selector = CssScopedStyleSelector::Relative(relative.clone());
    let selectors = CssScopedStyleSelectorList::try_new(vec![
        implicit_selector.clone(),
        relative_selector.clone(),
    ])
    .unwrap();
    let declaration = parse_single_declaration("color", "rgba(0, 0, 0, 1)");
    let style = CssScopedStyleRule::new(
        selectors.clone(),
        CssDeclarationList::new(vec![declaration.clone()]),
        source_position(5, 3),
    );
    let scoped_rules = CssScopedRuleList::from_rules(vec![CssScopedRule::Style(style.clone())]);
    let location = source_position(5, 1);
    let scope = CssScopeRule::new(
        Some(root.clone()),
        Some(limit.clone()),
        scoped_rules.clone(),
        location,
    );

    assert_eq!(root.selectors(), &[root_selector]);
    assert_eq!(limit.selectors(), &[limit_selector]);
    assert_eq!(
        selectors.selectors(),
        &[implicit_selector, relative_selector]
    );
    assert_eq!(style.selectors(), &selectors);
    assert_eq!(style.declarations().as_slice(), &[declaration]);
    assert_eq!(style.position(), source_position(5, 3));
    assert_eq!(scope.root(), Some(&root));
    assert_eq!(scope.limit(), Some(&limit));
    assert_eq!(scope.rules(), &scoped_rules);
    assert_eq!(scope.position(), location);
    assert_eq!(CssRule::Scope(scope.clone()), CssRule::Scope(scope));

    let empty_rules = CssScopedRuleList::new();
    assert!(empty_rules.rules().is_empty());
}

#[test]
fn scope_selector_list_constructor_rejects_pseudo_elements() {
    let ordinary = CssSelector::Class("card".to_owned());
    let ordinary_list = CssScopeSelectorList::try_new(vec![ordinary.clone()]).unwrap();
    assert_eq!(ordinary_list.selectors(), &[ordinary]);

    let pseudo_element = CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap();
    let selector = CssSelector::Compound(
        CssCompoundSelector::new_with_scope_anchor_and_pseudo_elements(
            false,
            None,
            None,
            vec!["card".to_owned()],
            Vec::new(),
            Vec::new(),
            Some(pseudo_element),
        ),
    );

    assert_eq!(CssScopeSelectorList::try_new(vec![selector]), None);

    let first =
        CssCompoundSelector::new(None, None, vec!["card".to_owned()], Vec::new(), Vec::new());
    let part = CssComplexSelectorPart::new(
        CssSelectorCombinator::Child,
        CssCompoundSelector::new_with_scope_anchor_and_pseudo_elements(
            false,
            None,
            None,
            vec!["badge".to_owned()],
            Vec::new(),
            Vec::new(),
            Some(CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap()),
        ),
    );
    let selector = CssSelector::Complex(CssComplexSelector::try_new(first, vec![part]).unwrap());

    assert_eq!(CssScopeSelectorList::try_new(vec![selector]), None);
}

#[test]
fn scoped_group_rule_models_keep_scoped_children() {
    let child_selector =
        CssScopedStyleSelectorList::try_new(vec![CssScopedStyleSelector::Selector(
            CssSelector::Class("label".to_owned()),
        )])
        .unwrap();
    let location = source_position(8, 9);
    let child = CssScopedRule::Style(CssScopedStyleRule::new(
        child_selector,
        CssDeclarationList::new(Vec::new()),
        location,
    ));
    let scoped_children = CssScopedRuleList::from_rules(vec![child.clone()]);
    let query = CssMediaQueryList::try_new(vec![CssMediaQuery::Typed(CssTypedMediaQuery::new(
        None,
        CssMediaType::Screen,
        None,
        test_media_position(),
    ))])
    .unwrap();
    let media = CssScopedMediaRule::new(query.clone(), scoped_children.clone(), location);
    assert_eq!(media.query(), &query);
    assert_eq!(media.rules(), &scoped_children);
    assert_eq!(media.position(), location);

    let condition =
        parse_container_condition_for_test("(inline-size > 30rem)").expect("condition parses");
    let name = CssContainerName::try_new("sidebar").unwrap();
    let container = CssScopedContainerRule::new(
        Some(name.clone()),
        condition.clone(),
        scoped_children.clone(),
        location,
    );
    assert_eq!(container.name(), Some(&name));
    assert_eq!(container.condition(), &condition);
    assert_eq!(container.rules(), &scoped_children);
    assert_eq!(container.position(), location);

    let layer_name = CssLayerName::try_new(["theme"]).unwrap();
    let layer_names = CssLayerNameList::try_new(vec![layer_name.clone()]).unwrap();
    let statement = CssScopedLayerStatementRule::new(layer_names.clone(), location);
    assert_eq!(statement.names(), &layer_names);
    assert_eq!(statement.position(), location);

    let block =
        CssScopedLayerBlockRule::new(Some(layer_name.clone()), scoped_children.clone(), location);
    assert_eq!(block.name(), Some(&layer_name));
    assert_eq!(block.rules(), &scoped_children);
    assert_eq!(block.position(), location);

    assert_eq!(
        CssScopedRule::Media(media.clone()),
        CssScopedRule::Media(media)
    );
    assert_eq!(
        CssScopedRule::Container(container.clone()),
        CssScopedRule::Container(container)
    );
    assert_eq!(
        CssScopedRule::LayerStatement(statement.clone()),
        CssScopedRule::LayerStatement(statement)
    );
    assert_eq!(
        CssScopedRule::LayerBlock(block.clone()),
        CssScopedRule::LayerBlock(block)
    );
}

#[test]
fn scoped_selector_model_preserves_authored_scope_anchor_marker() {
    let anchored = CssCompoundSelector::new_with_scope_anchor(
        true,
        None,
        None,
        vec!["active".to_owned()],
        Vec::new(),
        vec![CssPseudoClass::Scope],
    );

    assert!(anchored.has_scope_anchor());
    assert_eq!(anchored.classes(), &["active".to_owned()]);
    assert_eq!(anchored.pseudo_classes(), &[CssPseudoClass::Scope]);

    let selector = CssScopedStyleSelector::Selector(CssSelector::Compound(anchored.clone()));
    let list = CssScopedStyleSelectorList::try_new(vec![selector.clone()]).unwrap();
    assert_eq!(list.selectors(), &[selector]);
}

#[test]
fn import_target_constructors_reject_empty_values() {
    assert_eq!(CssImportUrl::try_new(""), None);
    assert_eq!(CssImportUrl::try_new(" \t\n "), None);
    assert_eq!(CssImportString::try_new(""), None);
    assert_eq!(CssImportString::try_new(" \t\n "), None);
    assert_eq!(
        CssImportUrl::try_new("theme.css").unwrap().as_str(),
        "theme.css"
    );
    assert_eq!(
        CssImportString::try_new("theme.css").unwrap().as_str(),
        "theme.css"
    );
}

#[test]
fn import_rule_accessors_expose_authored_structure() {
    let target = CssImportTarget::Url(CssImportUrl::try_new("theme.css").unwrap());
    let layer = CssImportLayer::Named(CssLayerName::try_new(["theme", "components"]).unwrap());
    let media = CssMediaQueryList::try_new(vec![CssMediaQuery::Typed(CssTypedMediaQuery::new(
        None,
        CssMediaType::Screen,
        None,
        test_media_position(),
    ))])
    .unwrap();
    let location = source_position(3, 7);
    let rule = CssImportRule::new(
        target.clone(),
        Some(layer.clone()),
        None,
        Some(media.clone()),
        location,
    );

    assert_eq!(rule.target(), &target);
    assert_eq!(rule.layer(), Some(&layer));
    assert_eq!(rule.supports(), None);
    assert_eq!(rule.media(), Some(&media));
    assert_eq!(rule.position(), location);
    assert_eq!(CssRule::Import(rule.clone()), CssRule::Import(rule));
}

#[test]
fn import_rule_parser_accepts_targets_layers_and_media() {
    let sheet = parse_sheet(r#"@import "theme.css";"#).unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.target(),
        &CssImportTarget::String(CssImportString::try_new("theme.css").unwrap())
    );
    assert_eq!(rule.layer(), None);
    assert_eq!(rule.media(), None);

    let sheet = parse_sheet(r#"@import url("layout.css");"#).unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    assert_eq!(
        import_rule(rule).target(),
        &CssImportTarget::Url(CssImportUrl::try_new("layout.css").unwrap())
    );

    let sheet = parse_sheet("@import url(tokens.css) layer;").unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.target(),
        &CssImportTarget::Url(CssImportUrl::try_new("tokens.css").unwrap())
    );
    assert_eq!(rule.layer(), Some(&CssImportLayer::Anonymous));

    let sheet = parse_sheet(r#"@import url("components.css") layer(components.buttons);"#).unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.layer(),
        Some(&CssImportLayer::Named(
            CssLayerName::try_new(["components", "buttons"]).unwrap()
        ))
    );

    let sheet = parse_sheet(r#"@import url("print.css") print;"#).unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let [CssMediaQuery::Typed(print_query)] = import_rule(rule).media().unwrap().queries() else {
        panic!("expected typed print media query");
    };
    assert_eq!(print_query.media_type(), CssMediaType::Print);

    let sheet = parse_sheet(r#"@import url("wide.css") screen and (min-width: 900px);"#).unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let [CssMediaQuery::Typed(query)] = import_rule(rule).media().unwrap().queries() else {
        panic!("expected typed media query");
    };
    assert_eq!(query.media_type(), CssMediaType::Screen);
    assert!(query.condition().is_some());

    let sheet = parse_sheet(
        r#"@import url("components.css") layer(components) screen and (min-width: 900px);"#,
    )
    .unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one import rule");
    };
    let rule = import_rule(rule);
    assert_eq!(
        rule.layer(),
        Some(&CssImportLayer::Named(
            CssLayerName::try_new(["components"]).unwrap()
        ))
    );
    assert!(rule.media().is_some());
}

#[test]
fn import_rule_parser_allows_imports_before_style_rules() {
    let sheet = parse_sheet(
        r#"
            @import "theme.css";
            @import url("components.css") layer(components) screen;
            .panel { color: black; }
        "#,
    )
    .unwrap();

    assert!(matches!(
        sheet.rules(),
        [CssRule::Import(_), CssRule::Import(_), CssRule::Style(_)]
    ));
}

#[test]
fn import_rule_parser_rejects_late_nested_unsupported_and_malformed_imports() {
    for css in [
        r#".panel { color: black; } @import "late.css";"#,
        r#"@media screen { @import "nested.css"; }"#,
        r#"@scope { @import "nested.css"; }"#,
        r#"@import url("theme.css") supports(display: grid) supports(color: red);"#,
        r#"@import url("theme.css") screen layer(components);"#,
        "@import;",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn layer_rule_parser_accepts_statement_and_block_forms() {
    let sheet = parse_sheet(
        r#"
            @layer reset, theme.components;
            @layer theme { .button { color: black; } }
            @layer { .utility { color: red; } }
        "#,
    )
    .unwrap();

    assert_eq!(sheet.rules().len(), 3);
    let statement = layer_statement_rule(&sheet.rules()[0]);
    assert_eq!(
        statement.names().names(),
        &[
            CssLayerName::try_new(["reset"]).unwrap(),
            CssLayerName::try_new(["theme", "components"]).unwrap(),
        ]
    );

    let named = layer_block_rule(&sheet.rules()[1]);
    assert_eq!(
        named.name(),
        Some(&CssLayerName::try_new(["theme"]).unwrap())
    );
    assert!(matches!(named.rules(), [CssRule::Style(_)]));

    let anonymous = layer_block_rule(&sheet.rules()[2]);
    assert_eq!(anonymous.name(), None);
    assert!(matches!(anonymous.rules(), [CssRule::Style(_)]));
}

#[test]
fn layer_rule_parser_accepts_nested_group_rule_blocks() {
    let sheet = parse_sheet(
        r#"
            @media screen {
                @layer theme.components {
                    @container (inline-size > 30rem) {
                        @layer utilities;
                        .button { color: black; }
                    }
                }
            }
        "#,
    )
    .unwrap();

    let media = media_rule(&sheet.rules()[0]);
    let [CssRule::LayerBlock(layer)] = media.rules() else {
        panic!("expected layer block inside media");
    };
    assert_eq!(
        layer.name(),
        Some(&CssLayerName::try_new(["theme", "components"]).unwrap())
    );
    let [CssRule::Container(container)] = layer.rules() else {
        panic!("expected container inside layer");
    };
    assert!(matches!(
        container.rules(),
        [CssRule::LayerStatement(_), CssRule::Style(_)]
    ));
}

#[test]
fn layer_rule_parser_rejects_malformed_layer_syntax() {
    for css in [
        "@layer;",
        "@layer , theme;",
        "@layer theme,;",
        "@layer theme..components;",
        "@layer theme, { .x { color: black; } }",
        "@layer theme.components extra { .x { color: black; } }",
        "@layer initial;",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn scope_rule_parser_accepts_roots_limits_and_scoped_style_rules() {
    let sheet = parse_sheet(
        r#"
            @scope (.card, [data-scope]) to (.stop, .boundary) {
                .title, > .action { color: black; }
                &:hover { color: red; }
                :scope .note { color: blue; }
            }
        "#,
    )
    .unwrap();

    let scope = scope_rule(&sheet.rules()[0]);
    assert_eq!(scope.root().unwrap().selectors().len(), 2);
    assert_eq!(scope.limit().unwrap().selectors().len(), 2);

    let [
        CssScopedRule::Style(first),
        CssScopedRule::Style(anchored),
        CssScopedRule::Style(scope_pseudo),
    ] = scope.rules().rules()
    else {
        panic!("expected three scoped style rules");
    };
    let selectors = first.selectors().selectors();
    assert!(matches!(selectors[0], CssScopedStyleSelector::Selector(_)));
    assert!(matches!(selectors[1], CssScopedStyleSelector::Relative(_)));
    let CssScopedStyleSelector::Relative(relative) = &selectors[1] else {
        panic!("expected relative selector");
    };
    assert_eq!(relative.combinator(), CssSelectorCombinator::Child);

    let [CssScopedStyleSelector::Selector(CssSelector::Compound(selector))] =
        anchored.selectors().selectors()
    else {
        panic!("expected authored scope anchor selector");
    };
    assert!(selector.has_scope_anchor());
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Hover]);

    let [CssScopedStyleSelector::Selector(CssSelector::Complex(selector))] =
        scope_pseudo.selectors().selectors()
    else {
        panic!("expected authored :scope complex selector");
    };
    assert_eq!(selector.first().pseudo_classes(), &[CssPseudoClass::Scope]);
}

#[test]
fn scope_rule_parser_accepts_pseudo_elements_in_scoped_style_rules() {
    let sheet = parse_sheet("@scope (.card) { .label::before { color: red; } }").unwrap();
    let scope = scope_rule(&sheet.rules()[0]);
    let [CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("expected scoped style rule");
    };
    let [CssScopedStyleSelector::Selector(CssSelector::Compound(selector))] =
        style.selectors().selectors()
    else {
        panic!("expected compound scoped style selector");
    };

    assert_eq!(selector.classes(), &["label".to_owned()]);
    assert_eq!(
        selector.pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::Before]
    );
}

#[test]
fn scope_rule_parser_accepts_limit_only_and_empty_prelude_scope() {
    let sheet = parse_sheet(
        r#"
            @scope to (.stop) { .title { color: black; } }
            @scope { > .item { color: red; } }
        "#,
    )
    .unwrap();

    let limit_only = scope_rule(&sheet.rules()[0]);
    assert!(limit_only.root().is_none());
    assert_eq!(limit_only.limit().unwrap().selectors().len(), 1);

    let anonymous = scope_rule(&sheet.rules()[1]);
    assert!(anonymous.root().is_none());
    assert!(anonymous.limit().is_none());
    let [CssScopedRule::Style(rule)] = anonymous.rules().rules() else {
        panic!("expected scoped style rule");
    };
    assert!(matches!(
        rule.selectors().selectors(),
        [CssScopedStyleSelector::Relative(_)]
    ));
}

#[test]
fn scope_rule_parser_keeps_nested_scoped_group_rules_scoped() {
    let sheet = parse_sheet(
        r#"
            @scope (.card) {
                @media screen {
                    @container (inline-size > 30rem) {
                        @layer theme {
                            @scope (.inner) {
                                > .label { color: black; }
                            }
                        }
                    }
                }
                @layer reset;
            }
        "#,
    )
    .unwrap();

    let scope = scope_rule(&sheet.rules()[0]);
    let [
        CssScopedRule::Media(media),
        CssScopedRule::LayerStatement(_),
    ] = scope.rules().rules()
    else {
        panic!("expected scoped media and layer statement");
    };
    let [CssScopedRule::Container(container)] = media.rules().rules() else {
        panic!("expected scoped container");
    };
    let [CssScopedRule::LayerBlock(layer)] = container.rules().rules() else {
        panic!("expected scoped layer block");
    };
    let [CssScopedRule::Scope(nested_scope)] = layer.rules().rules() else {
        panic!("expected nested scope rule");
    };
    let [CssScopedRule::Style(style)] = nested_scope.rules().rules() else {
        panic!("expected scoped relative style rule");
    };
    assert!(matches!(
        style.selectors().selectors(),
        [CssScopedStyleSelector::Relative(_)]
    ));
}

#[test]
fn public_api_exposes_layer_scope_and_scoped_rule_structure() {
    let sheet = parse_sheet(
        r#"
            @layer reset, theme.components;
            @layer theme {
                .button:hover::before { content: "x"; }
                .panel > dialog::backdrop { color: black; }
            }
            @scope (.card, [data-scope]) to (.stop) {
                .title, > .action { color: black; }
                @media screen {
                    @layer components {
                        > .icon::after { content: counter(section); }
                    }
                }
                @layer scoped.order;
            }
        "#,
    )
    .unwrap();

    let [
        CssRule::LayerStatement(statement),
        CssRule::LayerBlock(layer),
        CssRule::Scope(scope),
    ] = sheet.rules()
    else {
        panic!("expected layer statement, layer block, and scope rule");
    };

    let layer_names = statement.names().names();
    assert_eq!(layer_names.len(), 2);
    assert_eq!(layer_names[0].components(), &["reset".to_owned()]);
    assert_eq!(
        layer_names[1].components(),
        &["theme".to_owned(), "components".to_owned()]
    );
    assert_eq!(layer.name().unwrap().components(), &["theme".to_owned()]);

    let [CssRule::Style(before_rule), CssRule::Style(backdrop_rule)] = layer.rules() else {
        panic!("expected ordinary style rules inside layer block");
    };
    let CssSelector::Compound(before_selector) = before_rule.selector() else {
        panic!("expected compound pseudo-element selector");
    };
    assert_eq!(before_selector.pseudo_classes(), &[CssPseudoClass::Hover]);
    assert_eq!(
        before_selector.pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::Before]
    );

    let CssSelector::Complex(backdrop_selector) = backdrop_rule.selector() else {
        panic!("expected complex pseudo-element selector");
    };
    let [backdrop_part] = backdrop_selector.rest() else {
        panic!("expected one complex selector part");
    };
    assert_eq!(backdrop_part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(
        backdrop_part
            .selector()
            .pseudo_elements()
            .unwrap()
            .pseudo_elements(),
        &[CssPseudoElement::Backdrop]
    );

    assert_eq!(scope.root().unwrap().selectors().len(), 2);
    assert_eq!(scope.limit().unwrap().selectors().len(), 1);
    let [
        CssScopedRule::Style(scoped_style),
        CssScopedRule::Media(scoped_media),
        CssScopedRule::LayerStatement(scoped_layer_statement),
    ] = scope.rules().rules()
    else {
        panic!("expected scoped style, scoped media, and scoped layer statement");
    };

    let scoped_selectors = scoped_style.selectors().selectors();
    assert!(matches!(
        scoped_selectors[0],
        CssScopedStyleSelector::Selector(_)
    ));
    let CssScopedStyleSelector::Relative(relative_action) = &scoped_selectors[1] else {
        panic!("expected relative scoped selector");
    };
    assert_eq!(relative_action.combinator(), CssSelectorCombinator::Child);

    assert_eq!(
        scoped_layer_statement.names().names()[0].components(),
        &["scoped".to_owned(), "order".to_owned()]
    );
    let [CssScopedRule::LayerBlock(scoped_layer)] = scoped_media.rules().rules() else {
        panic!("expected scoped layer nested inside scoped media");
    };
    let [CssScopedRule::Style(nested_scoped_style)] = scoped_layer.rules().rules() else {
        panic!("expected scoped style rule inside scoped layer");
    };
    let [CssScopedStyleSelector::Relative(relative_icon)] =
        nested_scoped_style.selectors().selectors()
    else {
        panic!("expected nested scoped-only relative selector");
    };
    assert_eq!(relative_icon.combinator(), CssSelectorCombinator::Child);
    let CssSelector::Compound(icon_selector) = relative_icon.selector() else {
        panic!("expected compound selector after relative combinator");
    };
    assert_eq!(icon_selector.classes(), &["icon".to_owned()]);
    assert_eq!(
        icon_selector.pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::After]
    );
}

#[test]
fn nested_style_rule_parser_accepts_layer_and_scope_groups() {
    let sheet = parse_sheet(
        r#"
            .card {
                color: black;
                @layer components { & > .title { color: red; } }
                @scope (.title) { > .icon { color: blue; } }
            }
        "#,
    )
    .unwrap();

    assert!(matches!(
        sheet.rules(),
        [CssRule::Style(_), CssRule::LayerBlock(_), CssRule::Scope(_)]
    ));
    let layer = layer_block_rule(&sheet.rules()[1]);
    assert!(matches!(layer.rules(), [CssRule::Style(_)]));
    let scope = scope_rule(&sheet.rules()[2]);
    assert!(matches!(scope.rules().rules(), [CssScopedRule::Style(_)]));
}

#[test]
fn scope_rule_parser_rejects_malformed_scope_syntax() {
    for css in [
        "@scope .card { .title { color: black; } }",
        "@scope (.card) to { .title { color: black; } }",
        "@scope to { .title { color: black; } }",
        "@scope () { .title { color: black; } }",
        "@scope (.card,) { .title { color: black; } }",
        "@scope (.card) to (.stop) extra { .title { color: black; } }",
        "@scope (.card::before) { .title { color: black; } }",
        "@scope (.card) to (.stop::after) { .title { color: black; } }",
        "@scope (.card) { @font-face { font-family: Test; src: local(Test); } }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn relative_selectors_stay_rejected_outside_scoped_blocks() {
    assert!(parse_sheet("> .label { color: black; }").is_err());
    assert!(parse_sheet("@media screen { > .label { color: black; } }").is_err());
}

#[test]
fn parsed_style_rule_is_explicit_rule_variant() {
    let sheet = parse_sheet(".panel { width: 10px; }").unwrap();
    let [rule] = sheet.rules() else {
        panic!("style sheet should parse exactly one rule");
    };
    let style_rule = style_rule(rule);

    assert_eq!(
        style_rule.selector(),
        &CssSelector::Class("panel".to_owned())
    );
    assert_eq!(style_rule.declarations().len(), 1);
}

fn single_declaration(input: &str) -> CssDeclaration {
    let sheet = parse_sheet(input).unwrap();
    let [rule] = sheet.rules() else {
        panic!("{input} should parse exactly one rule");
    };
    let rule = style_rule(rule);
    let [declaration] = rule.declarations().as_slice() else {
        panic!("{input} should parse exactly one declaration");
    };
    declaration.clone()
}

fn filter_arguments(css: &str) -> CssFilterArguments {
    CssFilterArguments::new(CssAuthoredFunctionArguments::new(css))
}

fn basic_shape_arguments(css: &str) -> CssBasicShapeArguments {
    CssBasicShapeArguments::new(CssAuthoredFunctionArguments::new(css))
}

fn easing_arguments(css: &str) -> CssEasingArguments {
    CssEasingArguments::new(CssAuthoredFunctionArguments::new(css))
}

#[test]
fn background_color_preserves_authored_property_identity() {
    let declaration = single_declaration(".panel { background-color: black; }");
    assert_eq!(declaration.property(), &CssProperty::BackgroundColor);
    assert_eq!(
        declaration_payload!(declaration, BackgroundColor),
        CssColor::BLACK
    );
}

#[test]
fn custom_property_name_constructor_preserves_case_and_rejects_non_custom_names() {
    let name = CssCustomPropertyName::try_new("--BrandColor").unwrap();
    assert_eq!(name.as_str(), "--BrandColor");
    assert_eq!(
        CssCustomPropertyName::try_new("--brand_color-1")
            .unwrap()
            .as_str(),
        "--brand_color-1",
    );
    assert_eq!(CssCustomPropertyName::try_new("color"), None);
    assert_eq!(CssCustomPropertyName::try_new("-gap"), None);
    assert_eq!(CssCustomPropertyName::try_new("--"), None);
    assert_eq!(CssCustomPropertyName::try_new("-- bad"), None);
    assert_eq!(CssCustomPropertyName::try_new("--;"), None);
    assert_eq!(CssCustomPropertyName::try_new("--gap;"), None);
    assert_eq!(CssCustomPropertyName::try_new("--gap\n"), None);
    assert_eq!(CssCustomPropertyName::try_new("--gap\u{7f}"), None);
}

#[test]
fn list_counter_and_content_models_preserve_authored_shapes() {
    let counter_name = CssCounterName::try_new("section").unwrap();
    let counter_style =
        CssCounterStyle::Named(CssCounterStyleName::try_new("chapter-style").unwrap());
    let counter = CssCounterFunction::new(counter_name.clone(), Some(counter_style.clone()));
    assert_eq!(counter.name(), &counter_name);
    assert_eq!(counter.style(), Some(&counter_style));

    let separator = CssContentString::try_new(".").unwrap();
    let counters =
        CssCountersFunction::new(counter_name.clone(), separator.clone(), Some(counter_style));
    assert_eq!(counters.name(), &counter_name);
    assert_eq!(counters.separator(), &separator);
    assert!(counters.style().is_some());

    let attr = CssAttributeName::try_new("data-label").unwrap();
    let content_list = CssContentList::try_new(vec![
        CssContentItem::String(CssContentString::try_new("Chapter ").unwrap()),
        CssContentItem::Counter(counter),
        CssContentItem::Counters(counters),
        CssContentItem::Attr(attr.clone()),
        CssContentItem::Url(CssUrl::try_new("marker.svg").unwrap()),
        CssContentItem::OpenQuote,
        CssContentItem::CloseQuote,
        CssContentItem::NoOpenQuote,
        CssContentItem::NoCloseQuote,
    ])
    .unwrap();
    assert_eq!(content_list.items().len(), 9);
    let list_style = CssListStyle::try_new(
        Some(CssListStyleType::String(
            CssContentString::try_new("*").unwrap(),
        )),
        Some(CssListStylePosition::Inside),
        Some(CssListStyleImage::Url(
            CssUrl::try_new("bullet.svg").unwrap(),
        )),
    )
    .unwrap();
    assert!(matches!(
        list_style.style_type(),
        Some(CssListStyleType::String(_))
    ));
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));
    assert!(matches!(
        list_style.image(),
        Some(CssListStyleImage::Url(_))
    ));
    let counter_change = CssCounterChange::new(counter_name.clone(), Some(4));
    assert_eq!(counter_change.name(), &counter_name);
    assert_eq!(counter_change.value(), Some(4));
    let changes = CssCounterChangeList::try_new(vec![counter_change]).unwrap();
    assert_eq!(changes.changes().len(), 1);
    assert_eq!(changes.changes()[0].name(), &counter_name);
}

#[test]
fn list_counter_and_content_constructors_reject_invalid_states() {
    assert_eq!(CssContentString::try_new("bad\0string"), None);
    assert_eq!(CssContentList::try_new(Vec::new()), None);
    assert_eq!(CssCounterChangeList::try_new(Vec::new()), None);
    assert_eq!(CssCounterChanges::try_changes(Vec::new()), None);
    assert_eq!(CssListStyle::try_new(None, None, None), None);

    for name in [
        "",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "none",
    ] {
        assert_eq!(CssCounterName::try_new(name), None, "{name:?} rejected");
    }
    assert_eq!(
        CssCounterName::try_new("list-item").unwrap().as_str(),
        "list-item"
    );
}

#[test]
fn counter_style_name_constructor_uses_counter_style_ident_rules() {
    assert_eq!(
        CssCounterStyleName::try_new("chapter-style")
            .unwrap()
            .as_str(),
        "chapter-style"
    );
    assert_eq!(
        CssCounterStyleName::try_new("auto").unwrap().as_str(),
        "auto"
    );
    assert_eq!(
        CssCounterStyleName::try_new("span").unwrap().as_str(),
        "span"
    );

    for name in [
        "none",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ] {
        assert_eq!(
            CssCounterStyleName::try_new(name),
            None,
            "{name:?} rejected"
        );
    }

    for name in [
        "",
        "1chapter",
        "-1chapter",
        "chapter style",
        "chapter;",
        ".chapter",
    ] {
        assert_eq!(
            CssCounterStyleName::try_new(name),
            None,
            "{name:?} rejected"
        );
    }
}

#[test]
fn parses_generated_content_values_symbolically() {
    let cases = [
        ("content normal", "normal", CssContent::Normal),
        ("content none", "none", CssContent::None),
        (
            "content string",
            "\"Chapter \"",
            CssContent::Items(
                CssContentList::try_new(vec![CssContentItem::String(
                    CssContentString::try_new("Chapter ").unwrap(),
                )])
                .unwrap(),
            ),
        ),
        (
            "content url",
            "url(marker.svg)",
            CssContent::Items(
                CssContentList::try_new(vec![CssContentItem::Url(
                    CssUrl::try_new("marker.svg").unwrap(),
                )])
                .unwrap(),
            ),
        ),
        (
            "content counter",
            "counter(section)",
            CssContent::Items(
                CssContentList::try_new(vec![CssContentItem::Counter(CssCounterFunction::new(
                    CssCounterName::try_new("section").unwrap(),
                    None,
                ))])
                .unwrap(),
            ),
        ),
        (
            "content counter with style",
            "counter(section, upper-roman)",
            CssContent::Items(
                CssContentList::try_new(vec![CssContentItem::Counter(CssCounterFunction::new(
                    CssCounterName::try_new("section").unwrap(),
                    Some(CssCounterStyle::BuiltIn(CssBuiltInCounterStyle::UpperRoman)),
                ))])
                .unwrap(),
            ),
        ),
        (
            "content counters",
            "counters(section, \".\")",
            CssContent::Items(
                CssContentList::try_new(vec![CssContentItem::Counters(CssCountersFunction::new(
                    CssCounterName::try_new("section").unwrap(),
                    CssContentString::try_new(".").unwrap(),
                    None,
                ))])
                .unwrap(),
            ),
        ),
        (
            "content counters with style",
            "counters(section, \".\", lower-alpha)",
            CssContent::Items(
                CssContentList::try_new(vec![CssContentItem::Counters(CssCountersFunction::new(
                    CssCounterName::try_new("section").unwrap(),
                    CssContentString::try_new(".").unwrap(),
                    Some(CssCounterStyle::BuiltIn(CssBuiltInCounterStyle::LowerAlpha)),
                ))])
                .unwrap(),
            ),
        ),
        (
            "content attr",
            "attr(data-label)",
            CssContent::Items(
                CssContentList::try_new(vec![CssContentItem::Attr(
                    CssAttributeName::try_new("data-label").unwrap(),
                )])
                .unwrap(),
            ),
        ),
        (
            "content quote keywords",
            "open-quote close-quote no-open-quote no-close-quote",
            CssContent::Items(
                CssContentList::try_new(vec![
                    CssContentItem::OpenQuote,
                    CssContentItem::CloseQuote,
                    CssContentItem::NoOpenQuote,
                    CssContentItem::NoCloseQuote,
                ])
                .unwrap(),
            ),
        ),
    ];

    for (label, authored_value, expected_value) in cases {
        let actual = single_declaration_value!("content", Content, authored_value);
        assert_eq!(actual, expected_value, "{label}");
    }
}

#[test]
fn parses_list_style_longhands_and_shorthand_symbolically() {
    assert_eq!(
        single_declaration_value!("list-style-type", ListStyleType, "square"),
        CssListStyleType::CounterStyle(CssCounterStyle::BuiltIn(CssBuiltInCounterStyle::Square,))
    );
    assert_eq!(
        single_declaration_value!("list-style-type", ListStyleType, "custom-counter"),
        CssListStyleType::CounterStyle(CssCounterStyle::Named(
            CssCounterStyleName::try_new("custom-counter").unwrap(),
        ))
    );
    assert_eq!(
        single_declaration_value!("list-style-type", ListStyleType, "\"*\""),
        CssListStyleType::String(CssContentString::try_new("*").unwrap(),)
    );
    assert_eq!(
        single_declaration_value!("list-style-position", ListStylePosition, "inside"),
        CssListStylePosition::Inside
    );
    assert_eq!(
        single_declaration_value!("list-style-image", ListStyleImage, "url(marker.svg)"),
        CssListStyleImage::Url(CssUrl::try_new("marker.svg").unwrap(),)
    );

    let list_style =
        single_declaration_value!("list-style", ListStyle, "url(marker.svg) inside square");
    assert_eq!(
        list_style.style_type(),
        Some(&CssListStyleType::CounterStyle(CssCounterStyle::BuiltIn(
            CssBuiltInCounterStyle::Square,
        )))
    );
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));
    assert_eq!(
        list_style.image(),
        Some(&CssListStyleImage::Url(
            CssUrl::try_new("marker.svg").unwrap()
        ))
    );

    let list_style = single_declaration_value!("list-style", ListStyle, "none inside");
    assert_eq!(list_style.style_type(), Some(&CssListStyleType::None));
    assert_eq!(list_style.image(), Some(&CssListStyleImage::None));
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));

    let list_style = single_declaration_value!("list-style", ListStyle, "none");
    assert_eq!(list_style.style_type(), Some(&CssListStyleType::None));
    assert_eq!(list_style.image(), Some(&CssListStyleImage::None));
    assert_eq!(list_style.position(), None);

    for authored_value in ["square none", "none square"] {
        let list_style = single_declaration_value!("list-style", ListStyle, authored_value);
        assert_eq!(
            list_style.style_type(),
            Some(&CssListStyleType::CounterStyle(CssCounterStyle::BuiltIn(
                CssBuiltInCounterStyle::Square,
            ))),
            "{authored_value}"
        );
        assert_eq!(
            list_style.image(),
            Some(&CssListStyleImage::None),
            "{authored_value}"
        );
        assert_eq!(list_style.position(), None, "{authored_value}");
    }

    for authored_value in ["url(marker.svg) none", "none url(marker.svg)"] {
        let list_style = single_declaration_value!("list-style", ListStyle, authored_value);
        assert_eq!(
            list_style.style_type(),
            Some(&CssListStyleType::None),
            "{authored_value}"
        );
        assert_eq!(
            list_style.image(),
            Some(&CssListStyleImage::Url(
                CssUrl::try_new("marker.svg").unwrap()
            )),
            "{authored_value}"
        );
        assert_eq!(list_style.position(), None, "{authored_value}");
    }
}

#[test]
fn parses_counter_change_values_symbolically() {
    assert_eq!(
        single_declaration_value!("counter-reset", CounterReset, "none"),
        CssCounterChanges::None
    );

    for property in ["counter-reset", "counter-increment", "counter-set"] {
        let value = match property {
            "counter-reset" => {
                single_declaration_value!("counter-reset", CounterReset, "section 2 page -1 item")
            }
            "counter-increment" => single_declaration_value!(
                "counter-increment",
                CounterIncrement,
                "section 2 page -1 item"
            ),
            "counter-set" => {
                single_declaration_value!("counter-set", CounterSet, "section 2 page -1 item")
            }
            _ => unreachable!(),
        };
        let CssCounterChanges::Changes(changes) = value else {
            panic!("{property} should parse counter changes");
        };
        assert_eq!(changes.changes().len(), 3, "{property}");
        assert_eq!(changes.changes()[0].name().as_str(), "section");
        assert_eq!(changes.changes()[0].value(), Some(2));
        assert_eq!(changes.changes()[1].name().as_str(), "page");
        assert_eq!(changes.changes()[1].value(), Some(-1));
        assert_eq!(changes.changes()[2].name().as_str(), "item");
        assert_eq!(changes.changes()[2].value(), None);
    }
}

#[test]
fn public_api_exposes_generated_content_list_style_and_counter_values() {
    let sheet = parse_sheet(
        r#"
            .chapter::before {
                content: "Chapter " counter(section, upper-roman) counters(item, ".", lower-alpha) attr(data-label) open-quote;
                list-style: url(marker.svg) inside square;
                counter-reset: section 1 page -1 item;
            }
        "#,
    )
    .unwrap();

    let style = style_rule(&sheet.rules()[0]);
    let content = style
        .declarations()
        .iter()
        .find(|declaration| declaration.property() == CssProperty::Content)
        .unwrap();
    let CssContent::Items(content_list) = declaration_payload!(*content, Content) else {
        panic!("expected content item list");
    };
    let [
        CssContentItem::String(prefix),
        CssContentItem::Counter(counter),
        CssContentItem::Counters(counters),
        CssContentItem::Attr(attribute),
        CssContentItem::OpenQuote,
    ] = content_list.items()
    else {
        panic!("expected inspectable generated content items");
    };
    assert_eq!(prefix.as_str(), "Chapter ");
    assert_eq!(counter.name().as_str(), "section");
    assert!(matches!(
        counter.style(),
        Some(CssCounterStyle::BuiltIn(CssBuiltInCounterStyle::UpperRoman))
    ));
    assert_eq!(counters.name().as_str(), "item");
    assert_eq!(counters.separator().as_str(), ".");
    assert!(matches!(
        counters.style(),
        Some(CssCounterStyle::BuiltIn(CssBuiltInCounterStyle::LowerAlpha))
    ));
    assert_eq!(attribute.as_str(), "data-label");

    let CssSelector::Compound(selector) = style.selector() else {
        panic!("expected compound pseudo-element selector");
    };
    assert_eq!(
        selector.pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::Before]
    );

    let list_style = style
        .declarations()
        .iter()
        .find(|declaration| declaration.property() == CssProperty::ListStyle)
        .unwrap();
    let list_style = declaration_payload!(*list_style, ListStyle);
    assert_eq!(
        list_style.style_type(),
        Some(&CssListStyleType::CounterStyle(CssCounterStyle::BuiltIn(
            CssBuiltInCounterStyle::Square,
        )))
    );
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));
    let Some(CssListStyleImage::Url(marker)) = list_style.image() else {
        panic!("expected marker URL slot");
    };
    assert_eq!(marker.as_str(), "marker.svg");

    let counter_reset = style
        .declarations()
        .iter()
        .find(|declaration| declaration.property() == CssProperty::CounterReset)
        .unwrap();
    let CssCounterChanges::Changes(changes) = declaration_payload!(*counter_reset, CounterReset)
    else {
        panic!("expected counter change list");
    };
    let [section, page, item] = changes.changes() else {
        panic!("expected three counter changes");
    };
    assert_eq!(section.name().as_str(), "section");
    assert_eq!(section.value(), Some(1));
    assert_eq!(page.name().as_str(), "page");
    assert_eq!(page.value(), Some(-1));
    assert_eq!(item.name().as_str(), "item");
    assert_eq!(item.value(), None);
}

#[test]
fn rejects_unsupported_generated_content_list_and_counter_forms() {
    for (property_name, authored_value) in [
        ("content", "normal \"x\""),
        ("content", "counter()"),
        ("content", "counters(item)"),
        ("content", "attr()"),
        ("content", "\"x\" / \"alt\""),
        ("content", "contents"),
        ("content", "linear-gradient(red, blue)"),
        ("content", "target-counter(attr(href), page)"),
        ("content", "counter(item, symbols(cyclic \"*\" \"+\"))"),
        ("list-style-position", "center"),
        ("list-style-image", "red"),
        ("list-style-image", "linear-gradient(red, blue)"),
        ("list-style", "inside outside"),
        ("list-style", "none none"),
        ("list-style", "none none inside"),
        ("list-style", "symbols(cyclic \"*\" \"+\") inside"),
        ("counter-reset", "none item"),
        ("counter-increment", "1"),
        ("counter-set", "inherit 1"),
    ] {
        RejectedDeclarationCase {
            label: authored_value,
            property_name,
            authored_value,
            expected_error: ExpectedErrorKind::InvalidSyntaxOrUnsupportedValueForProperty {
                property: property_name,
            },
            property_name_should_be_recognized: true,
        }
        .assert_rejects();
    }
}

#[test]
fn authored_declaration_value_constructor_rejects_empty_css() {
    let value = CssAuthoredDeclarationValue::try_new("  8px  ").unwrap();
    assert_eq!(value.as_css(), "  8px  ");
    assert_eq!(CssAuthoredDeclarationValue::try_new(""), None);
    assert_eq!(CssAuthoredDeclarationValue::try_new(" \t\n "), None);
}

#[test]
fn variable_reference_and_fallback_accessors_preserve_authored_css() {
    let fallback_reference =
        CssVariableReference::new(CssCustomPropertyName::try_new("--fallback").unwrap(), None);
    let fallback = CssVariableFallback::new(
        CssAuthoredDeclarationValue::try_new("calc(1px + var(--fallback))").unwrap(),
        vec![fallback_reference.clone()],
    );
    let reference = CssVariableReference::new(
        CssCustomPropertyName::try_new("--space").unwrap(),
        Some(fallback),
    );
    assert_eq!(reference.name().as_str(), "--space");
    let fallback = reference.fallback().unwrap();
    assert_eq!(fallback.as_css(), "calc(1px + var(--fallback))");
    assert_eq!(fallback.references(), &[fallback_reference]);
}

#[test]
fn custom_property_value_accessors_preserve_authored_css() {
    let value = CssCustomPropertyValue::new(
        CssAuthoredDeclarationValue::try_new("calc(var(--space) * 2)").unwrap(),
    );
    assert_eq!(value.as_css(), "calc(var(--space) * 2)");
    assert!(!value.is_empty());
}

#[test]
fn variable_dependent_value_preserves_authored_css() {
    let authored = CssAuthoredDeclarationValue::try_new("var(--space)").unwrap();
    let value = CssSubstitutionDependentValue::new(authored);
    assert_eq!(value.as_css(), "var(--space)");
}

#[test]
fn parses_custom_property_declarations_as_authored_syntax() {
    let declaration = single_declaration(".theme { --BrandColor: #fff; }");
    assert_eq!(
        declaration.property(),
        &CssProperty::Custom(CssCustomPropertyName::try_new("--BrandColor").unwrap())
    );
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "#fff");
    assert!(!value.is_empty());
}

#[test]
fn custom_value_preserves_nested_variable_fallback_authored_css() {
    let declaration =
        single_declaration(".theme { --gap: var(--space, calc(1px + var(--fallback))); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--space, calc(1px + var(--fallback)))");
}

#[test]
fn custom_value_preserves_variable_fallback_authored_css() {
    let declaration = single_declaration(".theme { --gap: var(--space, 8px); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--space, 8px)");
}

#[test]
fn custom_values_accept_plain_and_empty_variable_fallback_forms() {
    let declaration = single_declaration(".theme { --gap: var(--space); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--space)");

    let declaration = single_declaration(".theme { --gap: var(--empty,); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "var(--empty,)");
}

#[test]
fn supported_properties_accept_variable_dependent_values_symbolically() {
    let declaration = single_declaration(".panel { gap: var(--space, 8px); }");
    assert_eq!(declaration.property(), &CssProperty::Gap);
    let value = declaration
        .known()
        .unwrap()
        .substitution_dependent()
        .expect("expected variable dependent value");
    assert_eq!(value.as_css(), "var(--space, 8px)");
}

#[test]
fn supported_properties_accept_embedded_variable_dependent_values_symbolically() {
    let declaration = single_declaration(".panel { width: calc(var(--w) + 1px); }");
    assert_eq!(declaration.property(), &CssProperty::Width);
    let value = declaration
        .known()
        .unwrap()
        .substitution_dependent()
        .expect("expected variable dependent value");
    assert_eq!(value.as_css(), "calc(var(--w) + 1px)");
}

#[test]
fn variable_dependent_values_skip_post_substitution_validation() {
    let declaration = single_declaration(".panel { color: var(--brand, 8px); }");
    assert_eq!(declaration.property(), &CssProperty::Color);
    let value = declaration
        .known()
        .unwrap()
        .substitution_dependent()
        .expect("expected variable dependent value");
    assert_eq!(value.as_css(), "var(--brand, 8px)");
}

#[test]
fn malformed_var_in_supported_property_rejects_whole_sheet() {
    assert!(parse_sheet(".panel { gap: var(color); }").is_err());
    assert!(parse_sheet(".panel { color: var(--brand); bogus: 1; }").is_err());
}

#[test]
fn no_var_invalid_supported_values_still_reject_strictly() {
    assert!(parse_sheet(".panel { gap: auto; }").is_err());
}

#[test]
fn parses_root_selector_for_custom_property_declarations() {
    let sheet = parse_sheet(":root { --space: 8px; }").unwrap();

    assert_eq!(sheet.rules().len(), 1);
    assert_eq!(style_rule(&sheet.rules()[0]).declarations().len(), 1);
    assert_eq!(
        style_rule(&sheet.rules()[0]).declarations()[0].property(),
        &CssProperty::Custom(CssCustomPropertyName::try_new("--space").unwrap())
    );
}

#[test]
fn root_selector_carries_root_pseudo_class_structurally() {
    let sheet = parse_sheet(":root { --space: 8px; }").unwrap();

    assert_eq!(
        style_rule(&sheet.rules()[0]).selector(),
        &CssSelector::PseudoClass(CssPseudoClass::Root)
    );
}

#[test]
fn compound_root_selector_carries_root_pseudo_class_structurally() {
    let sheet = parse_sheet("html:root { --space: 8px; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };

    assert_eq!(selector.tag().map(String::as_str), Some("html"));
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Root]);
}

#[test]
fn parses_tier_1_state_pseudo_classes_as_authored_selectors() {
    let cases = [
        (":hover { color: black; }", CssPseudoClass::Hover),
        (":active { color: black; }", CssPseudoClass::Active),
        (":focus { color: black; }", CssPseudoClass::Focus),
        (
            ":focus-visible { color: black; }",
            CssPseudoClass::FocusVisible,
        ),
        (
            ":focus-within { color: black; }",
            CssPseudoClass::FocusWithin,
        ),
        (":disabled { color: black; }", CssPseudoClass::Disabled),
        (":enabled { color: black; }", CssPseudoClass::Enabled),
        (":checked { color: black; }", CssPseudoClass::Checked),
        (":required { color: black; }", CssPseudoClass::Required),
        (":optional { color: black; }", CssPseudoClass::Optional),
        (":valid { color: black; }", CssPseudoClass::Valid),
        (":invalid { color: black; }", CssPseudoClass::Invalid),
        (
            ":placeholder-shown { color: black; }",
            CssPseudoClass::PlaceholderShown,
        ),
    ];

    for (css, expected) in cases {
        let sheet = parse_sheet(css).unwrap();
        assert_eq!(
            style_rule(&sheet.rules()[0]).selector(),
            &CssSelector::PseudoClass(expected)
        );
    }
}

#[test]
fn parses_compound_tier_1_state_pseudo_classes() {
    let sheet = parse_sheet(".button:hover { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    assert_eq!(selector.classes(), &["button".to_owned()]);
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn rejects_function_syntax_for_simple_state_pseudo_classes() {
    assert!(parse_sheet(":hover() { color: black; }").is_err());
    assert!(parse_sheet(":focus() { color: black; }").is_err());
}

#[test]
fn parses_tier_2_structural_simple_pseudo_classes() {
    let cases = [
        (":first-child { color: black; }", CssPseudoClass::FirstChild),
        (":last-child { color: black; }", CssPseudoClass::LastChild),
        (":only-child { color: black; }", CssPseudoClass::OnlyChild),
        (":empty { color: black; }", CssPseudoClass::Empty),
        (
            ":first-of-type { color: black; }",
            CssPseudoClass::FirstOfType,
        ),
        (
            ":last-of-type { color: black; }",
            CssPseudoClass::LastOfType,
        ),
        (
            ":only-of-type { color: black; }",
            CssPseudoClass::OnlyOfType,
        ),
    ];

    for (css, expected) in cases {
        let sheet = parse_sheet(css).unwrap();
        assert_eq!(
            style_rule(&sheet.rules()[0]).selector(),
            &CssSelector::PseudoClass(expected)
        );
    }
}

#[test]
fn parses_compound_structural_simple_pseudo_classes() {
    let sheet = parse_sheet("button:first-child { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    assert_eq!(selector.tag().map(String::as_str), Some("button"));
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::FirstChild]);
}

#[test]
fn rejects_function_syntax_for_non_functional_structural_pseudo_classes() {
    assert!(parse_sheet(":first-child() { color: black; }").is_err());
    assert!(parse_sheet(":empty() { color: black; }").is_err());
}

#[test]
fn parses_requested_double_colon_pseudo_elements() {
    let cases = [
        ("::selection { color: black; }", CssPseudoElement::Selection),
        ("::before { color: black; }", CssPseudoElement::Before),
        ("::after { color: black; }", CssPseudoElement::After),
        ("::marker { color: black; }", CssPseudoElement::Marker),
        ("::backdrop { color: black; }", CssPseudoElement::Backdrop),
    ];

    for (css, expected) in cases {
        let sheet = parse_sheet(css).unwrap_or_else(|error| panic!("{css}: {error:?}"));
        let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
            panic!("expected compound selector for {css}");
        };
        assert_eq!(
            selector.pseudo_elements().unwrap().pseudo_elements(),
            &[expected]
        );
    }
}

#[test]
fn parses_compound_and_complex_terminal_pseudo_element_selectors() {
    let sheet = parse_sheet(
        r#"
            .button.primary:hover::before { color: black; }
            li[data-kind="task"]::marker { color: red; }
            .card > dialog::backdrop { color: blue; }
        "#,
    )
    .unwrap();

    let CssSelector::Compound(button) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound button selector");
    };
    assert_eq!(
        button.classes(),
        &["button".to_owned(), "primary".to_owned()]
    );
    assert_eq!(button.pseudo_classes(), &[CssPseudoClass::Hover]);
    assert_eq!(
        button.pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::Before]
    );

    let CssSelector::Compound(marker) = style_rule(&sheet.rules()[1]).selector() else {
        panic!("expected compound marker selector");
    };
    assert_eq!(marker.tag().map(String::as_str), Some("li"));
    assert_eq!(
        marker.pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::Marker]
    );

    let CssSelector::Complex(backdrop) = style_rule(&sheet.rules()[2]).selector() else {
        panic!("expected complex backdrop selector");
    };
    assert_eq!(backdrop.first().classes(), &["card".to_owned()]);
    let [part] = backdrop.rest() else {
        panic!("expected one complex selector part");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(
        part.selector().pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::Backdrop]
    );
}

#[test]
fn parses_supported_generated_marker_pseudo_element_chains() {
    for css in [
        ".item::before::marker { color: black; }",
        ".item::after::marker { color: black; }",
    ] {
        let sheet = parse_sheet(css).unwrap_or_else(|error| panic!("{css}: {error:?}"));
        let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
            panic!("expected compound selector for {css}");
        };
        assert_eq!(
            selector.pseudo_elements().unwrap().pseudo_elements(),
            if css.contains("before") {
                &[CssPseudoElement::Before, CssPseudoElement::Marker][..]
            } else {
                &[CssPseudoElement::After, CssPseudoElement::Marker][..]
            }
        );
    }
}

#[test]
fn rejects_invalid_pseudo_element_forms_and_non_terminal_positions() {
    for css in [
        ":marker { color: black; }",
        "::part(foo) { color: black; }",
        "::unknown { color: black; }",
        ".button::before:hover { color: black; }",
        ".button::before.primary { color: black; }",
        ".button::before[data-x] { color: black; }",
        ".button::before#icon { color: black; }",
        ".button::before span { color: black; }",
        ".button::before > span { color: black; }",
        ".button::marker::before { color: black; }",
        ".button::before::after { color: black; }",
        ".button::selection::marker { color: black; }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn selector_list_constructor_rejects_empty_lists() {
    assert_eq!(CssSelectorList::try_new(Vec::new()), None);
    let list = CssSelectorList::try_new(vec![CssSelector::Class("button".to_owned())]).unwrap();
    assert_eq!(list.selectors(), &[CssSelector::Class("button".to_owned())]);
}

#[test]
fn pseudo_selector_list_constructor_accepts_complex_selectors() {
    assert_eq!(CssPseudoSelectorList::try_new(Vec::new()), None);

    let first =
        CssCompoundSelector::new(None, None, vec!["field".to_owned()], Vec::new(), Vec::new());
    let part = CssComplexSelectorPart::new(
        CssSelectorCombinator::Descendant,
        CssCompoundSelector::new(None, None, vec!["icon".to_owned()], Vec::new(), Vec::new()),
    );
    let complex = CssSelector::Complex(CssComplexSelector::try_new(first, vec![part]).unwrap());

    let list = CssPseudoSelectorList::try_new(vec![complex.clone()]).unwrap();
    assert_eq!(list.selectors(), &[complex]);
}

#[test]
fn pseudo_element_sequence_constructor_guards_supported_terminal_shapes() {
    let before = CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap();
    assert_eq!(before.pseudo_elements(), &[CssPseudoElement::Before]);

    let before_marker =
        CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before, CssPseudoElement::Marker])
            .unwrap();
    assert_eq!(
        before_marker.pseudo_elements(),
        &[CssPseudoElement::Before, CssPseudoElement::Marker]
    );

    assert_eq!(CssPseudoElementSequence::try_new(Vec::new()), None);
    assert_eq!(
        CssPseudoElementSequence::try_new(
            vec![CssPseudoElement::Marker, CssPseudoElement::Before,]
        ),
        None
    );
    assert_eq!(
        CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before, CssPseudoElement::After,]),
        None
    );
}

#[test]
fn complex_selector_constructor_rejects_parts_after_pseudo_elements() {
    let pseudo_element = CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap();
    let first = CssCompoundSelector::new_with_scope_anchor_and_pseudo_elements(
        false,
        None,
        None,
        vec!["button".to_owned()],
        Vec::new(),
        Vec::new(),
        Some(pseudo_element),
    );
    let part = CssComplexSelectorPart::new(
        CssSelectorCombinator::Descendant,
        CssCompoundSelector::new(None, None, vec!["icon".to_owned()], Vec::new(), Vec::new()),
    );

    assert_eq!(CssComplexSelector::try_new(first, vec![part]), None);
}

#[test]
fn relative_selector_list_constructor_requires_selectors() {
    assert_eq!(CssRelativeSelectorList::try_new(Vec::new()), None);
}

#[test]
fn relative_selector_preserves_combinator_and_selector() {
    let selector = CssRelativeSelector::new(
        CssSelectorCombinator::Child,
        CssSelector::Class("icon".to_owned()),
    );

    assert_eq!(selector.combinator(), CssSelectorCombinator::Child);
    assert_eq!(selector.selector(), &CssSelector::Class("icon".to_owned()));
}

#[test]
fn nth_child_pattern_preserves_optional_selector_list() {
    let list =
        CssPseudoSelectorList::try_new(vec![CssSelector::Class("important".to_owned())]).unwrap();
    let pattern =
        CssNthChildPattern::new(CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 1)), Some(list));

    assert!(
        matches!(pattern.pattern(), CssNthPattern::AnPlusB(value) if value.a() == 2 && value.b() == 1)
    );
    assert_eq!(
        pattern.selector_list().unwrap().selectors(),
        &[CssSelector::Class("important".to_owned())]
    );
}

#[test]
fn nth_pattern_model_exposes_an_plus_b_coefficients() {
    let pattern = CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 1));
    let CssNthPattern::AnPlusB(value) = pattern else {
        panic!("expected an+b pattern");
    };
    assert_eq!(value.a(), 2);
    assert_eq!(value.b(), 1);
}

#[test]
fn parses_nth_child_patterns() {
    let cases = [
        (":nth-child(odd) { color: black; }", CssNthPattern::Odd),
        (":nth-child(even) { color: black; }", CssNthPattern::Even),
        (":nth-child(3) { color: black; }", CssNthPattern::Integer(3)),
        (
            ":nth-child(-1) { color: black; }",
            CssNthPattern::Integer(-1),
        ),
        (
            ":nth-child(+3) { color: black; }",
            CssNthPattern::Integer(3),
        ),
        (
            ":nth-child(n) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(1, 0)),
        ),
        (
            ":nth-child(-n) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(-1, 0)),
        ),
        (
            ":nth-child(+n) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(1, 0)),
        ),
        (
            ":nth-child(2n+1) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 1)),
        ),
        (
            ":nth-child(2n-1) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, -1)),
        ),
        (
            ":nth-child(-n+3) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(-1, 3)),
        ),
        (
            ":nth-child(+3n-2) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(3, -2)),
        ),
    ];

    for (css, expected) in cases {
        let sheet = parse_sheet(css).unwrap_or_else(|error| panic!("{css}: {error:?}"));
        let CssSelector::PseudoClass(CssPseudoClass::NthChild(pattern)) =
            style_rule(&sheet.rules()[0]).selector()
        else {
            panic!("expected nth-child selector");
        };
        assert_eq!(pattern.pattern(), expected);
        assert!(pattern.selector_list().is_none());
    }
}

#[test]
fn parses_all_nth_structural_pseudo_classes() {
    let cases = [
        (
            ":nth-child(2n) { color: black; }",
            CssPseudoClass::NthChild(CssNthChildPattern::new(
                CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0)),
                None,
            )),
        ),
        (
            ":nth-last-child(2n) { color: black; }",
            CssPseudoClass::NthLastChild(CssNthChildPattern::new(
                CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0)),
                None,
            )),
        ),
        (
            ":nth-of-type(2n) { color: black; }",
            CssPseudoClass::NthOfType(CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0))),
        ),
        (
            ":nth-last-of-type(2n) { color: black; }",
            CssPseudoClass::NthLastOfType(CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0))),
        ),
    ];

    for (css, expected) in cases {
        let sheet = parse_sheet(css).unwrap();
        assert_eq!(
            style_rule(&sheet.rules()[0]).selector(),
            &CssSelector::PseudoClass(expected)
        );
        if let CssSelector::PseudoClass(
            CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern),
        ) = style_rule(&sheet.rules()[0]).selector()
        {
            assert!(pattern.selector_list().is_none());
        }
    }
}

#[test]
fn nth_child_accepts_strict_of_selector_lists() {
    let sheet =
        parse_sheet("li:nth-child(2n+1 of li.important, .row[hidden]) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::NthChild(pattern)] = selector.pseudo_classes() else {
        panic!("expected nth-child pseudo-class");
    };
    assert!(
        matches!(pattern.pattern(), CssNthPattern::AnPlusB(value) if value.a() == 2 && value.b() == 1)
    );
    let selector_list = pattern.selector_list().expect("expected of selector list");
    assert_eq!(selector_list.selectors().len(), 2);
    assert!(matches!(
        selector_list.selectors()[0],
        CssSelector::Compound(_)
    ));
    assert!(matches!(
        selector_list.selectors()[1],
        CssSelector::Compound(_)
    ));

    let sheet =
        parse_sheet(".item:nth-last-child(even of .selected ~ .tail) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::NthLastChild(pattern)] = selector.pseudo_classes() else {
        panic!("expected nth-last-child pseudo-class");
    };
    assert_eq!(pattern.pattern(), CssNthPattern::Even);
    assert!(matches!(
        pattern.selector_list().unwrap().selectors()[0],
        CssSelector::Complex(_)
    ));
}

#[test]
fn nth_child_of_selector_lists_reject_invalid_entries_strictly() {
    assert!(parse_sheet(":nth-child(odd of) { color: black; }").is_err());
    assert!(parse_sheet(":nth-child(odd of .valid, .bad..selector) { color: black; }").is_err());
    assert!(parse_sheet(":nth-child(odd of ::before) { color: black; }").is_err());
    assert!(parse_sheet(":nth-of-type(odd of .item) { color: black; }").is_err());
    assert!(parse_sheet(":nth-last-of-type(even of .item) { color: black; }").is_err());
}

#[test]
fn rejects_unsupported_nth_patterns_and_of_selector_forms() {
    assert!(parse_sheet(":nth-child() { color: black; }").is_err());
    assert!(parse_sheet(":nth-child(foo) { color: black; }").is_err());
    assert!(parse_sheet(":nth-child(2n +) { color: black; }").is_err());
    assert!(parse_sheet(":nth-child(2n + +1) { color: black; }").is_err());
    assert!(parse_sheet(":nth-of-type(2n of .item) { color: black; }").is_err());
}

#[test]
fn rejects_trailing_tokens_in_nth_functions() {
    assert!(parse_sheet(":nth-child(odd even) { color: black; }").is_err());
    assert!(parse_sheet(":nth-child(1 2) { color: black; }").is_err());
    assert!(parse_sheet(":nth-child(2n+1 extra) { color: black; }").is_err());
}

#[test]
fn nth_pseudo_class_arguments_are_publicly_inspectable() {
    let sheet = parse_sheet(":nth-child(2n+1) { color: black; }").unwrap();
    let CssSelector::PseudoClass(CssPseudoClass::NthChild(pattern)) =
        style_rule(&sheet.rules()[0]).selector()
    else {
        panic!("expected nth-child an+b selector");
    };
    let CssNthPattern::AnPlusB(value) = pattern.pattern() else {
        panic!("expected nth-child an+b pattern");
    };
    assert_eq!(value.a(), 2);
    assert_eq!(value.b(), 1);
    assert!(pattern.selector_list().is_none());
}

#[test]
fn parses_selector_list_functional_pseudo_classes() {
    let sheet = parse_sheet(".button:not(.disabled, .loading) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Not(list)] = selector.pseudo_classes() else {
        panic!("expected :not selector list");
    };
    assert_eq!(
        list.selectors(),
        &[
            CssSelector::Class("disabled".to_owned()),
            CssSelector::Class("loading".to_owned()),
        ]
    );

    let sheet = parse_sheet(":is(.primary, .secondary) { color: black; }").unwrap();
    assert!(matches!(
        style_rule(&sheet.rules()[0]).selector(),
        CssSelector::PseudoClass(CssPseudoClass::Is(_))
    ));

    let sheet = parse_sheet(":where(button, .link) { color: black; }").unwrap();
    assert!(matches!(
        style_rule(&sheet.rules()[0]).selector(),
        CssSelector::PseudoClass(CssPseudoClass::Where(_))
    ));

    let sheet = parse_sheet(".field:has(.error) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    assert!(matches!(
        selector.pseudo_classes(),
        [CssPseudoClass::Has(_)]
    ));
}

#[test]
fn parses_compound_selector_list_functional_pseudo_classes() {
    let sheet = parse_sheet(".field:not(:disabled, :focus) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Not(list)] = selector.pseudo_classes() else {
        panic!("expected :not selector list");
    };
    assert_eq!(
        list.selectors(),
        &[
            CssSelector::PseudoClass(CssPseudoClass::Disabled),
            CssSelector::PseudoClass(CssPseudoClass::Focus),
        ]
    );
}

#[test]
fn functional_selector_lists_accept_supported_complex_selectors() {
    let sheet = parse_sheet(
        ".scope:is(.card > .title, button.primary:hover, [data-state=\"open\"].active) { color: black; }",
    )
    .unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Is(list)] = selector.pseudo_classes() else {
        panic!("expected :is selector list");
    };
    assert_eq!(list.selectors().len(), 3);
    assert!(matches!(list.selectors()[0], CssSelector::Complex(_)));
    assert!(matches!(list.selectors()[1], CssSelector::Compound(_)));
    assert!(matches!(list.selectors()[2], CssSelector::Compound(_)));

    let sheet = parse_sheet(":not(.field .icon, button.primary:hover) { color: black; }").unwrap();
    let CssSelector::PseudoClass(CssPseudoClass::Not(list)) =
        style_rule(&sheet.rules()[0]).selector()
    else {
        panic!("expected :not selector list");
    };
    assert_eq!(list.selectors().len(), 2);
    assert!(matches!(list.selectors()[0], CssSelector::Complex(_)));
    assert!(matches!(list.selectors()[1], CssSelector::Compound(_)));

    let sheet = parse_sheet(":where(.toolbar + .panel, .stack ~ .item) { color: black; }").unwrap();
    let CssSelector::PseudoClass(CssPseudoClass::Where(list)) =
        style_rule(&sheet.rules()[0]).selector()
    else {
        panic!("expected :where selector list");
    };
    assert_eq!(list.selectors().len(), 2);
    assert!(
        list.selectors()
            .iter()
            .all(|selector| matches!(selector, CssSelector::Complex(_)))
    );
}

#[test]
fn functional_selector_lists_reject_invalid_entries_strictly() {
    assert!(parse_sheet(":is(.valid, .bad..selector) { color: black; }").is_err());
    assert!(parse_sheet(":where(.valid, .col || .cell) { color: black; }").is_err());
    assert!(parse_sheet(":not(.valid, ::before) { color: black; }").is_err());
    assert!(parse_sheet(":is(.valid,) { color: black; }").is_err());
}

#[test]
fn functional_pseudo_class_selector_arguments_reject_pseudo_elements() {
    for css in [
        ":is(::before) { color: black; }",
        ":where(.x::after) { color: black; }",
        ":not(::marker) { color: black; }",
        ":has(::backdrop) { color: black; }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn selector_argument_surface_accepts_full_supported_strict_forms() {
    for css in [
        ":not(.field .icon, button.primary:hover) { color: black; }",
        ":is(.card > .title, button.primary:hover, [data-state=\"open\"].active) { color: black; }",
        ":where(.toolbar + .panel, .stack ~ .item) { color: black; }",
        ".card:has(.field > .icon) { color: black; }",
        ".card:has(> .icon, + .error, ~ .warning) { color: black; }",
        "li:nth-child(2n+1 of li.important, .row[hidden]) { color: black; }",
        "li:nth-last-child(even of .item.selected) { color: black; }",
    ] {
        parse_sheet(css).unwrap_or_else(|error| panic!("{css} should parse: {error:?}"));
    }
}

#[test]
fn selector_argument_surface_rejects_invalid_entries_without_recovery() {
    for css in [
        ":is(.valid, .bad..selector) { color: black; }",
        ":where(.valid, .col || .cell) { color: black; }",
        ":not(.valid, ::before) { color: black; }",
        ".card:has() { color: black; }",
        ".card:has(:has(.nested)) { color: black; }",
        ".card:has(:is(:has(.nested))) { color: black; }",
        ".card:has(:nth-child(odd of :has(.nested))) { color: black; }",
        ".card:has(::before) { color: black; }",
        ".card:has(.valid, .bad..selector) { color: black; }",
        ":nth-child(odd of) { color: black; }",
        ":nth-child(odd of .valid, .bad..selector) { color: black; }",
        ":nth-of-type(odd of .item) { color: black; }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject strictly");
    }
}

#[test]
fn has_accepts_strict_relative_selector_lists() {
    let sheet = parse_sheet(".card:has(.field > .icon) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Has(list)] = selector.pseudo_classes() else {
        panic!("expected :has selector list");
    };
    assert_eq!(list.selectors().len(), 1);
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Descendant
    );
    assert!(matches!(
        list.selectors()[0].selector(),
        CssSelector::Complex(_)
    ));

    let sheet = parse_sheet(".card:has(> .icon, + .error, ~ .warning) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Has(list)] = selector.pseudo_classes() else {
        panic!("expected :has selector list");
    };
    assert_eq!(list.selectors().len(), 3);
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        list.selectors()[1].combinator(),
        CssSelectorCombinator::NextSibling
    );
    assert_eq!(
        list.selectors()[2].combinator(),
        CssSelectorCombinator::SubsequentSibling
    );
}

#[test]
fn has_rejects_invalid_relative_selector_entries_strictly() {
    assert!(parse_sheet(".card:has() { color: black; }").is_err());
    assert!(parse_sheet(".card:has(.valid, .bad..selector) { color: black; }").is_err());
    assert!(parse_sheet(".card:has(:has(.nested)) { color: black; }").is_err());
    assert!(parse_sheet(".card:has(:is(:has(.nested))) { color: black; }").is_err());
    assert!(parse_sheet(".card:has(:nth-child(odd of :has(.nested))) { color: black; }").is_err());
    assert!(parse_sheet(".card:has(::before) { color: black; }").is_err());
    assert!(parse_sheet(".card:has(| .bad) { color: black; }").is_err());
    assert!(parse_sheet(".card:has(.valid,) { color: black; }").is_err());
}

#[test]
fn functional_pseudo_class_arguments_are_publicly_inspectable() {
    let sheet = parse_sheet(".button:not(.disabled) { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Not(list)] = selector.pseudo_classes() else {
        panic!("expected :not selector list");
    };
    assert_eq!(
        list.selectors(),
        &[CssSelector::Class("disabled".to_owned())]
    );
}

#[test]
fn rejects_empty_selector_list_functional_pseudo_classes() {
    assert!(parse_sheet(":not() { color: black; }").is_err());
    assert!(parse_sheet(":is() { color: black; }").is_err());
    assert!(parse_sheet(":where() { color: black; }").is_err());
    assert!(parse_sheet(":has() { color: black; }").is_err());
}

#[test]
fn parses_tier_4_runtime_state_pseudo_classes() {
    let cases = [
        (":modal { color: black; }", CssPseudoClass::Modal),
        (":fullscreen { color: black; }", CssPseudoClass::Fullscreen),
        (
            ":popover-open { color: black; }",
            CssPseudoClass::PopoverOpen,
        ),
        (":default { color: black; }", CssPseudoClass::Default),
        (
            ":indeterminate { color: black; }",
            CssPseudoClass::Indeterminate,
        ),
        (":read-only { color: black; }", CssPseudoClass::ReadOnly),
        (":read-write { color: black; }", CssPseudoClass::ReadWrite),
        (":in-range { color: black; }", CssPseudoClass::InRange),
        (
            ":out-of-range { color: black; }",
            CssPseudoClass::OutOfRange,
        ),
    ];

    for (css, expected) in cases {
        let sheet = parse_sheet(css).unwrap();
        assert_eq!(
            style_rule(&sheet.rules()[0]).selector(),
            &CssSelector::PseudoClass(expected)
        );
    }
}

#[test]
fn parses_compound_runtime_state_pseudo_classes() {
    let sheet = parse_sheet(".dialog:modal:fullscreen { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    assert_eq!(selector.classes(), &["dialog".to_owned()]);
    assert_eq!(
        selector.pseudo_classes(),
        &[CssPseudoClass::Modal, CssPseudoClass::Fullscreen]
    );
}

#[test]
fn rejects_function_syntax_for_runtime_state_pseudo_classes() {
    assert!(parse_sheet(":modal() { color: black; }").is_err());
    assert!(parse_sheet(":fullscreen() { color: black; }").is_err());
    assert!(parse_sheet(":read-only() { color: black; }").is_err());
}

#[test]
fn rejects_unsupported_relative_or_combinator_selector_forms() {
    assert!(parse_sheet(".field:has(.col || .cell) { color: black; }").is_err());
    assert!(parse_sheet(".field:has(|| .icon) { color: black; }").is_err());
    assert!(parse_sheet(".field:has(::before) { color: black; }").is_err());
    assert!(parse_sheet(".field:has(| .icon) { color: black; }").is_err());
}

#[test]
fn parses_combinator_selectors() {
    for css in [
        ".stack .item { color: black; }",
        ".toolbar > button { color: black; }",
        "label + input { color: black; }",
        "h2 ~ p { color: black; }",
        ".card[data-state=open] > .title:hover { color: black; }",
    ] {
        let sheet = parse_sheet(css).unwrap_or_else(|error| panic!("{css}: {error:?}"));
        assert!(matches!(
            style_rule(&sheet.rules()[0]).selector(),
            CssSelector::Complex(_)
        ));
    }
}

#[test]
fn combinator_selectors_are_structurally_inspectable() {
    let sheet = parse_sheet(".toolbar > button { color: black; }").unwrap();
    let rule = style_rule(&sheet.rules()[0]);
    let CssSelector::Complex(selector) = rule.selector() else {
        panic!("expected complex selector");
    };
    assert_eq!(
        selector.rest()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        selector.rest()[0].selector().tag().map(String::as_str),
        Some("button")
    );
}

#[test]
fn nesting_selector_composition_preserves_parent_and_child_structure() {
    let parent = CssSelector::Class("card".to_owned());
    let child = CssSelector::Class("title".to_owned());

    let descendant = CssSelector::combine_descendant(parent.clone(), child.clone()).unwrap();
    let CssSelector::Complex(descendant) = descendant else {
        panic!("expected descendant complex selector");
    };
    assert_eq!(descendant.first().classes(), &["card".to_owned()]);
    let [part] = descendant.rest() else {
        panic!("expected one descendant part");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Descendant);
    assert_eq!(part.selector().classes(), &["title".to_owned()]);

    let appended = CssSelector::append_to_subject(
        parent,
        CssCompoundSelector::new(
            None,
            None,
            vec!["active".to_owned()],
            Vec::new(),
            vec![CssPseudoClass::Hover],
        ),
    )
    .unwrap();
    let CssSelector::Compound(appended) = appended else {
        panic!("expected compound selector");
    };
    assert_eq!(
        appended.classes(),
        &["card".to_owned(), "active".to_owned()]
    );
    assert_eq!(appended.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_selector_composition_preserves_child_complex_chain() {
    let parent = CssSelector::Class("card".to_owned());
    let child = CssSelector::Complex(
        CssComplexSelector::try_new(
            CssCompoundSelector::new(None, None, vec!["title".to_owned()], Vec::new(), Vec::new()),
            vec![CssComplexSelectorPart::new(
                CssSelectorCombinator::Child,
                CssCompoundSelector::new(
                    None,
                    None,
                    vec!["icon".to_owned()],
                    Vec::new(),
                    Vec::new(),
                ),
            )],
        )
        .unwrap(),
    );

    let combined = CssSelector::combine_descendant(parent, child).unwrap();
    let CssSelector::Complex(combined) = combined else {
        panic!("expected complex selector");
    };
    assert_eq!(combined.first().classes(), &["card".to_owned()]);
    let [title, icon] = combined.rest() else {
        panic!("expected parent descendant link followed by child chain");
    };
    assert_eq!(title.combinator(), CssSelectorCombinator::Descendant);
    assert_eq!(title.selector().classes(), &["title".to_owned()]);
    assert_eq!(icon.combinator(), CssSelectorCombinator::Child);
    assert_eq!(icon.selector().classes(), &["icon".to_owned()]);
}

#[test]
fn nesting_selector_composition_preserves_complex_chains() {
    let parent = CssSelector::Complex(
        CssComplexSelector::try_new(
            CssCompoundSelector::new(None, None, vec!["card".to_owned()], Vec::new(), Vec::new()),
            vec![CssComplexSelectorPart::new(
                CssSelectorCombinator::Child,
                CssCompoundSelector::new(
                    Some("button".to_owned()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            )],
        )
        .unwrap(),
    );
    let child = CssCompoundSelector::new(
        None,
        None,
        vec!["icon".to_owned()],
        Vec::new(),
        vec![CssPseudoClass::Hover],
    );

    let combined = CssSelector::combine_with_combinator(
        parent.clone(),
        CssSelectorCombinator::NextSibling,
        child,
    )
    .unwrap();
    let CssSelector::Complex(combined) = combined else {
        panic!("expected complex selector");
    };
    assert_eq!(combined.first().classes(), &["card".to_owned()]);
    let [button, icon] = combined.rest() else {
        panic!("expected preserved parent part and appended child part");
    };
    assert_eq!(button.combinator(), CssSelectorCombinator::Child);
    assert_eq!(button.selector().tag().map(String::as_str), Some("button"));
    assert_eq!(icon.combinator(), CssSelectorCombinator::NextSibling);
    assert_eq!(icon.selector().classes(), &["icon".to_owned()]);
    assert_eq!(icon.selector().pseudo_classes(), &[CssPseudoClass::Hover]);

    let appended = CssSelector::append_to_subject(
        parent,
        CssCompoundSelector::new(
            None,
            None,
            vec!["primary".to_owned()],
            vec![CssAttributeSelector::new_qualified(
                CssNamespaceConstraint::ExplicitNone,
                CssAttributeName::new("aria-current"),
                CssAttributeMatcher::Equals("true".to_owned()),
                CssAttributeCaseSensitivity::DocumentDefault,
            )],
            vec![CssPseudoClass::Focus],
        ),
    )
    .unwrap();
    let CssSelector::Complex(appended) = appended else {
        panic!("expected complex selector");
    };
    let [part] = appended.rest() else {
        panic!("expected preserved complex selector part");
    };
    assert_eq!(part.selector().tag().map(String::as_str), Some("button"));
    assert_eq!(part.selector().classes(), &["primary".to_owned()]);
    assert_eq!(part.selector().pseudo_classes(), &[CssPseudoClass::Focus]);
    let [attribute] = part.selector().attributes() else {
        panic!("expected appended attribute selector");
    };
    assert_eq!(attribute.name().as_str(), "aria-current");

    assert!(
        CssSelector::append_to_subject(
            CssSelector::Class("card".to_owned()),
            CssCompoundSelector::new(
                Some("button".to_owned()),
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        )
        .is_none()
    );
    assert!(
        CssSelector::append_to_subject(
            CssSelector::Class("card".to_owned()),
            CssCompoundSelector::new(
                None,
                Some("submit".to_owned()),
                Vec::new(),
                Vec::new(),
                Vec::new()
            ),
        )
        .is_none()
    );
}

#[test]
fn nesting_flattens_descendant_and_parent_selectors_in_source_order() {
    let sheet = parse_sheet(
        r#".card {
            color: black;
            .title { color: white; }
            background-color: white;
            &:hover { opacity: 0.8; }
        }"#,
    )
    .unwrap();

    let [base_before, title, base_after, hover] = sheet.rules() else {
        panic!("expected four flattened rules");
    };

    assert_eq!(
        style_rule(base_before).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(base_before).declarations()[0].property(),
        &CssProperty::Color
    );

    let CssSelector::Complex(title_selector) = style_rule(title).selector() else {
        panic!("expected descendant selector");
    };
    assert_eq!(title_selector.first().classes(), &["card".to_owned()]);
    assert_eq!(
        title_selector.rest()[0].combinator(),
        CssSelectorCombinator::Descendant
    );
    assert_eq!(
        title_selector.rest()[0].selector().classes(),
        &["title".to_owned()]
    );

    assert_eq!(
        style_rule(base_after).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(base_after).declarations()[0].property(),
        &CssProperty::BackgroundColor
    );

    let CssSelector::Compound(hover_selector) = style_rule(hover).selector() else {
        panic!("expected compound hover selector");
    };
    assert_eq!(hover_selector.classes(), &["card".to_owned()]);
    assert_eq!(hover_selector.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_flattens_selector_lists_and_relative_combinators() {
    let sheet = parse_sheet(
        r#".button, .link {
            &.active[aria-current=true] { color: black; }
            > .icon { opacity: 1; }
        }"#,
    )
    .unwrap();

    let [button_active, link_active, button_icon, link_icon] = sheet.rules() else {
        panic!("expected four flattened rules");
    };
    for rule in sheet.rules() {
        assert!(matches!(rule, CssRule::Style(_)));
    }

    for (rule, parent_class) in [(button_active, "button"), (link_active, "link")] {
        let CssSelector::Compound(selector) = style_rule(rule).selector() else {
            panic!("expected appended compound selector");
        };
        assert_eq!(
            selector.classes(),
            &[parent_class.to_owned(), "active".to_owned()]
        );
        let [attribute] = selector.attributes() else {
            panic!("expected appended attribute selector");
        };
        assert_eq!(attribute.name().as_str(), "aria-current");
        assert_eq!(
            attribute.matcher(),
            &CssAttributeMatcher::Equals("true".to_owned())
        );
    }

    for (rule, parent_class) in [(button_icon, "button"), (link_icon, "link")] {
        let CssSelector::Complex(selector) = style_rule(rule).selector() else {
            panic!("expected relative child selector");
        };
        assert_eq!(selector.first().classes(), &[parent_class.to_owned()]);
        let [part] = selector.rest() else {
            panic!("expected one child selector part");
        };
        assert_eq!(part.combinator(), CssSelectorCombinator::Child);
        assert_eq!(part.selector().classes(), &["icon".to_owned()]);
    }
}

#[test]
fn nesting_flattens_ampersand_combinator_chain_and_suffixes() {
    let sheet = parse_sheet(
        r#".tabs {
            & > .tab { color: black; }
        }
        .button {
            &.active[aria-current=true]:hover { opacity: 1; }
        }"#,
    )
    .unwrap();

    let [tab, button] = sheet.rules() else {
        panic!("expected two flattened rules");
    };

    let CssSelector::Complex(tab_selector) = style_rule(tab).selector() else {
        panic!("expected ampersand child selector");
    };
    assert_eq!(tab_selector.first().classes(), &["tabs".to_owned()]);
    let [part] = tab_selector.rest() else {
        panic!("expected one child selector part");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(part.selector().classes(), &["tab".to_owned()]);

    let CssSelector::Compound(button_selector) = style_rule(button).selector() else {
        panic!("expected appended compound selector");
    };
    assert_eq!(
        button_selector.classes(),
        &["button".to_owned(), "active".to_owned()]
    );
    let [attribute] = button_selector.attributes() else {
        panic!("expected appended attribute selector");
    };
    assert_eq!(attribute.name().as_str(), "aria-current");
    assert_eq!(button_selector.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_flattens_style_rules_inside_conditional_groups() {
    let sheet = parse_sheet(
        r#"@media screen {
            .card { .title { color: white; } }
        }
        @container sidebar (inline-size > 30rem) {
            .tabs { & > .tab { color: black; } }
        }"#,
    )
    .unwrap();

    let [media, container] = sheet.rules() else {
        panic!("expected media and container rules");
    };

    let [media_title] = media_rule(media).rules() else {
        panic!("expected one flattened media rule");
    };
    let CssSelector::Complex(title_selector) = style_rule(media_title).selector() else {
        panic!("expected descendant selector inside media");
    };
    assert_eq!(title_selector.first().classes(), &["card".to_owned()]);
    assert_eq!(
        title_selector.rest()[0].combinator(),
        CssSelectorCombinator::Descendant
    );
    assert_eq!(
        title_selector.rest()[0].selector().classes(),
        &["title".to_owned()]
    );

    let [container_tab] = container_rule(container).rules() else {
        panic!("expected one flattened container rule");
    };
    let CssSelector::Complex(tab_selector) = style_rule(container_tab).selector() else {
        panic!("expected child selector inside container");
    };
    assert_eq!(tab_selector.first().classes(), &["tabs".to_owned()]);
    assert_eq!(
        tab_selector.rest()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        tab_selector.rest()[0].selector().classes(),
        &["tab".to_owned()]
    );
}

#[test]
fn nesting_flattens_media_and_container_inside_style_rules() {
    let sheet = parse_sheet(
        r#".card {
            color: black;
            @media (min-width: 600px) {
                background-color: white;
                > .title { color: black; }
            }
            @container sidebar (inline-size > 30rem) {
                &:hover { opacity: 0.9; }
            }
        }"#,
    )
    .unwrap();

    let [base, media, container] = sheet.rules() else {
        panic!("expected base, media, and container rules");
    };
    assert_eq!(
        style_rule(base).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(base).declarations()[0].property(),
        &CssProperty::Color
    );

    let media = media_rule(media);
    let [media_base, media_title] = media.rules() else {
        panic!("expected two flattened media rules");
    };
    assert_eq!(
        style_rule(media_base).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(media_base).declarations()[0].property(),
        &CssProperty::BackgroundColor
    );
    let CssSelector::Complex(title_selector) = style_rule(media_title).selector() else {
        panic!("expected complex title selector");
    };
    assert_eq!(title_selector.first().classes(), &["card".to_owned()]);
    assert_eq!(
        title_selector.rest()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        title_selector.rest()[0].selector().classes(),
        &["title".to_owned()]
    );

    let container = container_rule(container);
    let [container_hover] = container.rules() else {
        panic!("expected one flattened container rule");
    };
    let CssSelector::Compound(hover_selector) = style_rule(container_hover).selector() else {
        panic!("expected hover compound selector");
    };
    assert_eq!(hover_selector.classes(), &["card".to_owned()]);
    assert_eq!(hover_selector.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_preserves_terminal_pseudo_elements_on_appended_selectors() {
    let sheet = parse_sheet(".card { &::before { color: red; } }").unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected flattened pseudo-element rule");
    };
    let CssSelector::Compound(selector) = style_rule(rule).selector() else {
        panic!("expected compound selector");
    };

    assert_eq!(selector.classes(), &["card".to_owned()]);
    assert_eq!(
        selector.pseudo_elements().unwrap().pseudo_elements(),
        &[CssPseudoElement::Before]
    );
}

#[test]
fn nesting_rejects_selector_parts_after_pseudo_elements() {
    for css in [
        ".card::before { .icon { color: red; } }",
        ".card { &::before .icon { color: red; } }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn nesting_flattens_media_before_later_declaration_run_in_source_order() {
    let sheet = parse_sheet(
        r#".card {
            color: black;
            @media (min-width: 600px) { opacity: 0.8; }
            background-color: white;
        }"#,
    )
    .unwrap();

    let [base_before, media, base_after] = sheet.rules() else {
        panic!("expected parent declaration, media, parent declaration");
    };

    assert_eq!(
        style_rule(base_before).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(base_before).declarations()[0].property(),
        &CssProperty::Color
    );

    let [media_base] = media_rule(media).rules() else {
        panic!("expected one flattened media rule");
    };
    assert_eq!(
        style_rule(media_base).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(media_base).declarations()[0].property(),
        &CssProperty::Opacity
    );

    assert_eq!(
        style_rule(base_after).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(base_after).declarations()[0].property(),
        &CssProperty::BackgroundColor
    );
}

#[test]
fn nesting_inside_media_and_container_stays_inside_group() {
    let sheet = parse_sheet(
        r#"@media (prefers-color-scheme: dark) {
            .card { .title { color: white; } }
        }
        @container sidebar (inline-size > 30rem) {
            .card { &:hover { opacity: 0.9; } }
        }"#,
    )
    .unwrap();

    let [media, container] = sheet.rules() else {
        panic!("expected media and container");
    };
    assert!(matches!(media, CssRule::Media(_)));
    assert!(matches!(container, CssRule::Container(_)));

    let [media_title] = media_rule(media).rules() else {
        panic!("expected nested media style rule");
    };
    let CssSelector::Complex(title_selector) = style_rule(media_title).selector() else {
        panic!("expected descendant selector inside media");
    };
    assert_eq!(title_selector.first().classes(), &["card".to_owned()]);
    assert_eq!(
        title_selector.rest()[0].selector().classes(),
        &["title".to_owned()]
    );

    let [container_hover] = container_rule(container).rules() else {
        panic!("expected nested container style rule");
    };
    let CssSelector::Compound(hover_selector) = style_rule(container_hover).selector() else {
        panic!("expected hover selector inside container");
    };
    assert_eq!(hover_selector.classes(), &["card".to_owned()]);
    assert_eq!(hover_selector.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn nesting_rejects_unsupported_at_rules_inside_style_blocks() {
    for css in [
        r#".card { @import "x.css"; }"#,
        r#".card { @font-face { font-family: Inter; src: url("inter.woff2"); } }"#,
        ".card { @keyframes fade { from { opacity: 0; } } }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn nesting_rejects_unsupported_nested_selector_forms() {
    for (css, expects_selector_error) in [
        (".card { .theme & { color: black; } }", true),
        (".card { && { color: black; } }", true),
        (".card { & & { color: black; } }", true),
        (".card { svg|a { color: black; } }", false),
        (".card { [svg|href] { color: black; } }", true),
        (".card { .col || .cell { color: black; } }", true),
    ] {
        let error = parse_sheet(css).expect_err(css);
        if expects_selector_error {
            assert!(
                matches!(error.kind(), ErrorKind::InvalidSelector { .. }),
                "{css} should reject as an invalid selector, got {error:?}"
            );
        }
    }
}

#[test]
fn keyframes_and_nesting_reject_browser_recovery_forms() {
    let rejected = [
        "@-webkit-keyframes fade { from { opacity: 0; } }",
        "@keyframes fade { 0 { opacity: 0; } }",
        "@keyframes fade { from { @media screen { opacity: 0; } } }",
        "@keyframes fade { from { .nested { opacity: 0; } } }",
        ".card { & & { color: black; } }",
        ".card { .theme & { color: black; } }",
        ".card { && { color: black; } }",
        ".card { svg|a { color: black; } }",
        ".card { .col || .cell { color: black; } }",
        r#".card { @import url("theme.css"); }"#,
        r#".card { @font-face { font-family: Inter; src: url("inter.woff2"); } }"#,
        ".card { @keyframes fade { from { opacity: 0; } } }",
    ];

    for css in rejected {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn keyframes_and_nesting_accept_practical_surface_matrix() {
    let accepted = [
        r#"@keyframes fade { from { opacity: 0; } to { opacity: 1; } }"#,
        r#"@keyframes "fade in" { 0%, 100% { opacity: 1; } }"#,
        r#"@keyframes duplicate { from, 0% { } from { opacity: 1; } }"#,
        ".card { color: black; .title { color: white; } }",
        ".card { &:hover { opacity: 0.9; } }",
        ".card { > .title[aria-current=true] { color: white; } }",
        ".card { @media (min-width: 600px) { &:hover { opacity: 0.9; } } }",
        "@media screen { .card { .title { color: black; } } }",
        "@container sidebar (inline-size > 30rem) { .card { &:hover { opacity: 1; } } }",
        ".card { &:not(.field .icon) { color: black; } }",
    ];

    for css in accepted {
        assert!(parse_sheet(css).is_ok(), "{css} should parse");
    }
}

#[test]
fn rejects_invalid_combinator_selectors() {
    assert!(parse_sheet("> .item { color: black; }").is_err());
    assert!(parse_sheet(".a > > .b { color: black; }").is_err());
    assert!(parse_sheet(".a > { color: black; }").is_err());
    assert!(parse_sheet(".col || .cell { color: black; }").is_err());
    assert!(parse_sheet(".field:has(> > .icon) { color: black; }").is_err());
    assert!(parse_sheet(".field:has(.field > > .icon) { color: black; }").is_err());
}

#[test]
fn parses_attribute_selector_matcher_forms() {
    let cases = [
        ("[disabled] { color: black; }", CssAttributeMatcher::Exists),
        (
            "[data-state=open] { color: black; }",
            CssAttributeMatcher::Equals("open".to_owned()),
        ),
        (
            r#"[data-role~="button"] { color: black; }"#,
            CssAttributeMatcher::Includes("button".to_owned()),
        ),
        (
            "[lang|=en] { color: black; }",
            CssAttributeMatcher::DashMatch("en".to_owned()),
        ),
        (
            r#"[href^="https"] { color: black; }"#,
            CssAttributeMatcher::Prefix("https".to_owned()),
        ),
        (
            r#"[src$=".svg"] { color: black; }"#,
            CssAttributeMatcher::Suffix(".svg".to_owned()),
        ),
        (
            r#"[data-id*="card"] { color: black; }"#,
            CssAttributeMatcher::Substring("card".to_owned()),
        ),
    ];

    for (css, expected) in cases {
        let sheet = parse_sheet(css).unwrap();
        let CssSelector::Compound(selector) = style_rule(&sheet.rules()[0]).selector() else {
            panic!("{css} should parse as a compound selector");
        };
        let [attribute] = selector.attributes() else {
            panic!("{css} should have one attribute selector");
        };
        assert_eq!(attribute.matcher(), &expected);
        assert_eq!(
            attribute.case_sensitivity(),
            CssAttributeCaseSensitivity::DocumentDefault
        );
    }
}

#[test]
fn attribute_selectors_are_structurally_inspectable() {
    let sheet = parse_sheet(r#"[data-state="open" i] { color: black; }"#).unwrap();
    let rule = style_rule(&sheet.rules()[0]);
    let CssSelector::Compound(selector) = rule.selector() else {
        panic!("expected compound selector");
    };
    let [attribute] = selector.attributes() else {
        panic!("expected one attribute selector");
    };
    assert_eq!(attribute.name().as_str(), "data-state");
    assert_eq!(
        attribute.matcher(),
        &CssAttributeMatcher::Equals("open".to_owned())
    );
    assert_eq!(
        attribute.case_sensitivity(),
        CssAttributeCaseSensitivity::AsciiCaseInsensitive
    );
}

#[test]
fn parses_attribute_selector_case_modifiers_and_compound_position() {
    let insensitive = parse_sheet(r#"[data-state="OPEN" i] { color: black; }"#).unwrap();
    let CssSelector::Compound(selector) = style_rule(&insensitive.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    assert_eq!(
        selector.attributes()[0].case_sensitivity(),
        CssAttributeCaseSensitivity::AsciiCaseInsensitive
    );

    let sensitive = parse_sheet(r#"[data-state="open" s] { color: black; }"#).unwrap();
    let CssSelector::Compound(selector) = style_rule(&sensitive.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    assert_eq!(
        selector.attributes()[0].case_sensitivity(),
        CssAttributeCaseSensitivity::ExplicitSensitive
    );

    let mixed = parse_sheet("button.primary[aria-expanded=true]:hover { color: black; }").unwrap();
    let CssSelector::Compound(selector) = style_rule(&mixed.rules()[0]).selector() else {
        panic!("expected compound selector");
    };
    assert_eq!(selector.tag().map(String::as_str), Some("button"));
    assert_eq!(selector.classes(), &["primary".to_owned()]);
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Hover]);
    let [attribute] = selector.attributes() else {
        panic!("expected one attribute selector");
    };
    assert_eq!(attribute.name().as_str(), "aria-expanded");
    assert_eq!(
        attribute.matcher(),
        &CssAttributeMatcher::Equals("true".to_owned())
    );
}

#[test]
fn rejects_invalid_attribute_selectors() {
    assert!(parse_sheet("[svg|href] { color: black; }").is_err());
    assert!(parse_sheet("[data-state=] { color: black; }").is_err());
    assert!(parse_sheet("[data-state=open q] { color: black; }").is_err());
    assert!(parse_sheet("[] { color: black; }").is_err());
    assert!(parse_sheet("[data-state=open extra] { color: black; }").is_err());
}

#[test]
fn attribute_name_constructor_matches_parser_identifier_invariants() {
    assert_eq!(
        CssAttributeName::try_new("data-state").unwrap().as_str(),
        "data-state"
    );
    assert_eq!(CssAttributeName::try_new(""), None);
    assert_eq!(CssAttributeName::try_new(" \t\n "), None);
    assert_eq!(CssAttributeName::try_new("data state"), None);
    assert_eq!(CssAttributeName::try_new("svg|href"), None);
    assert_eq!(CssAttributeName::try_new("data-state extra"), None);
    assert_eq!(CssAttributeName::try_new("data-state;"), None);
}

#[test]
fn practical_pseudo_class_matrix_accepts_supported_and_rejects_unsupported_forms() {
    let accepted = [
        ":hover { color: black; }",
        ":focus-visible { color: black; }",
        ":disabled { color: black; }",
        ":first-child { color: black; }",
        ":nth-child(2n+1) { color: black; }",
        ":nth-child(2n of .item) { color: black; }",
        ":not(.disabled) { color: black; }",
        ".field:not(.field .icon) { color: black; }",
        ":is(.primary, .secondary) { color: black; }",
        ":where(button, .link) { color: black; }",
        ".field:has(.error) { color: black; }",
        ".field:has(> .icon) { color: black; }",
        ".field:has(.field > .icon) { color: black; }",
        ":modal { color: black; }",
        ":read-only { color: black; }",
        ":visited { color: black; }",
        ":target { color: black; }",
        ":lang(en) { color: black; }",
    ];

    for css in accepted {
        assert!(parse_sheet(css).is_ok(), "{css} should parse");
    }

    let rejected = [
        ":host { color: black; }",
        ":state(open) { color: black; }",
        ":hover() { color: black; }",
        ":not() { color: black; }",
        ":nth-of-type(2n of .item) { color: black; }",
        ".field:has(.col || .cell) { color: black; }",
        ".field:has(:has(.nested)) { color: black; }",
        ".field:has(.valid, .bad..selector) { color: black; }",
        ".field:not(::before) { color: black; }",
    ];

    for css in rejected {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn custom_property_with_var_remains_custom_property_value() {
    let declaration = single_declaration(".theme { --gap: var(--space, 8px); }");
    assert!(matches!(
        custom_property_value(&declaration),
        value
            if value.as_css() == "var(--space, 8px)"
    ));
}

#[test]
fn unknown_property_with_var_rejects() {
    assert!(parse_sheet(".panel { made-up-property: var(--space); }").is_err());
}

#[test]
fn custom_property_values_validate_nested_variable_syntax() {
    let declaration = single_declaration(".theme { --gap: calc(1px + var(--space)); }");
    let value = custom_property_value(&declaration);
    assert_eq!(value.as_css(), "calc(1px + var(--space))");
}

#[test]
fn rejects_malformed_variable_references() {
    assert!(parse_sheet(".theme { --gap: var(); }").is_err());
    assert!(parse_sheet(".theme { --gap: var(color); }").is_err());
    assert!(parse_sheet(".theme { --gap: var(--gap --other); }").is_err());
    assert!(parse_sheet(".theme { --gap: var(--gap, }").is_err());
}

#[test]
fn custom_property_global_keyword_must_be_whole_value() {
    assert_eq!(
        single_declaration(".theme { --gap: inherit; }")
            .custom()
            .unwrap()
            .value()
            .global(),
        Some(CssGlobalKeyword::Inherit)
    );
    assert!(parse_sheet(".theme { --gap: inherit 1px; }").is_err());
}

#[test]
fn custom_property_names_are_case_sensitive_when_parsed() {
    let declaration = single_declaration(".theme { --BrandColor: 1px; }");
    assert_eq!(
        declaration.property(),
        &CssProperty::Custom(CssCustomPropertyName::try_new("--BrandColor").unwrap())
    );
    assert_ne!(
        declaration.property(),
        &CssProperty::Custom(CssCustomPropertyName::try_new("--brandcolor").unwrap())
    );
}

#[test]
fn parser_accepts_escaped_custom_property_names_from_cssparser_ident_tokens() {
    assert_eq!(CssCustomPropertyName::try_new("--bad name"), None);

    let declaration = single_declaration(".theme { --bad\\ name: 1px; }");
    let CssProperty::Custom(name) = declaration.property() else {
        panic!("expected custom property");
    };
    assert_eq!(name.as_str(), "--bad name");
}

#[test]
fn rejects_malformed_custom_property_names() {
    assert!(parse_sheet(".theme { --: 1px; }").is_err());
    assert!(parse_sheet(".theme { --bad name: 1px; }").is_err());
}

fn assert_display_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Display(_)));
}

fn assert_box_sizing_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::BoxSizing(_)));
}

fn assert_position_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Position(_)));
}

fn assert_direction_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Direction(_)));
}

fn assert_overflow_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::Overflow(_)
            | CssKnownPropertyValueRef::OverflowX(_)
            | CssKnownPropertyValueRef::OverflowY(_)
    ));
}

fn assert_flex_direction_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FlexDirection(_)));
}

fn assert_flex_wrap_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FlexWrap(_)));
}

fn assert_float_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Float(_)));
}

fn assert_clear_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Clear(_)));
}

fn assert_alignment_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::AlignContent(_)
            | CssKnownPropertyValueRef::JustifyContent(_)
            | CssKnownPropertyValueRef::JustifyTracks(_)
            | CssKnownPropertyValueRef::AlignTracks(_)
    ));
}

fn assert_align_items_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::AlignItems(_)
            | CssKnownPropertyValueRef::AlignSelf(_)
            | CssKnownPropertyValueRef::JustifyItems(_)
            | CssKnownPropertyValueRef::JustifySelf(_)
    ));
}

fn assert_place_alignment_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::PlaceContent(_)
            | CssKnownPropertyValueRef::PlaceItems(_)
            | CssKnownPropertyValueRef::PlaceSelf(_)
    ));
}

fn assert_visibility_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Visibility(_)));
}

fn assert_content_visibility_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::ContentVisibility(_)
    ));
}

fn assert_length_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::Width(_)
            | CssKnownPropertyValueRef::Height(_)
            | CssKnownPropertyValueRef::MinWidth(_)
            | CssKnownPropertyValueRef::MinHeight(_)
            | CssKnownPropertyValueRef::MaxWidth(_)
            | CssKnownPropertyValueRef::MaxHeight(_)
            | CssKnownPropertyValueRef::FlexBasis(_)
            | CssKnownPropertyValueRef::Gap(_)
            | CssKnownPropertyValueRef::RowGap(_)
            | CssKnownPropertyValueRef::ColumnGap(_)
            | CssKnownPropertyValueRef::FontSize(_)
            | CssKnownPropertyValueRef::LineHeight(_)
            | CssKnownPropertyValueRef::Top(_)
            | CssKnownPropertyValueRef::Right(_)
            | CssKnownPropertyValueRef::Bottom(_)
            | CssKnownPropertyValueRef::Left(_)
            | CssKnownPropertyValueRef::MarginTop(_)
            | CssKnownPropertyValueRef::MarginRight(_)
            | CssKnownPropertyValueRef::MarginBottom(_)
            | CssKnownPropertyValueRef::MarginLeft(_)
            | CssKnownPropertyValueRef::PaddingTop(_)
            | CssKnownPropertyValueRef::PaddingRight(_)
            | CssKnownPropertyValueRef::PaddingBottom(_)
            | CssKnownPropertyValueRef::PaddingLeft(_)
            | CssKnownPropertyValueRef::BorderTopWidth(_)
            | CssKnownPropertyValueRef::BorderRightWidth(_)
            | CssKnownPropertyValueRef::BorderBottomWidth(_)
            | CssKnownPropertyValueRef::BorderLeftWidth(_)
    ));
}

fn assert_edges_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::Inset(_)
            | CssKnownPropertyValueRef::Margin(_)
            | CssKnownPropertyValueRef::Padding(_)
            | CssKnownPropertyValueRef::BorderWidth(_)
    ));
}

fn assert_color_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::Color(_)
            | CssKnownPropertyValueRef::Background(_)
            | CssKnownPropertyValueRef::BackgroundColor(_)
            | CssKnownPropertyValueRef::BorderColor(_)
            | CssKnownPropertyValueRef::BorderTopColor(_)
            | CssKnownPropertyValueRef::BorderRightColor(_)
            | CssKnownPropertyValueRef::BorderBottomColor(_)
            | CssKnownPropertyValueRef::BorderLeftColor(_)
    ));
}

fn assert_border_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::Border(_)
            | CssKnownPropertyValueRef::BorderTop(_)
            | CssKnownPropertyValueRef::BorderRight(_)
            | CssKnownPropertyValueRef::BorderBottom(_)
            | CssKnownPropertyValueRef::BorderLeft(_)
    ));
}

fn assert_border_style_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BorderTopStyle(_)
            | CssKnownPropertyValueRef::BorderRightStyle(_)
            | CssKnownPropertyValueRef::BorderBottomStyle(_)
            | CssKnownPropertyValueRef::BorderLeftStyle(_)
    ));
}

fn assert_border_styles_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::BorderStyle(_)));
}

fn assert_border_radius_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::BorderRadius(_)));
}

fn assert_corner_radius_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BorderTopLeftRadius(_)
            | CssKnownPropertyValueRef::BorderTopRightRadius(_)
            | CssKnownPropertyValueRef::BorderBottomRightRadius(_)
            | CssKnownPropertyValueRef::BorderBottomLeftRadius(_)
    ));
}

fn assert_box_shadow_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::BoxShadow(_)));
}

fn assert_opacity_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Opacity(_)));
}

fn assert_flex_grow_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FlexGrow(_)));
}

fn assert_flex_shrink_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FlexShrink(_)));
}

fn assert_aspect_ratio_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::AspectRatio(_)));
}

fn assert_scrollbar_width_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::ScrollbarWidth(_)));
}

fn assert_order_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Order(_)));
}

fn assert_flex_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Flex(_)));
}

fn assert_z_index_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::ZIndex(_)));
}

fn assert_box_decoration_break_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BoxDecorationBreak(_)
    ));
}

fn assert_grid_flow_tolerance_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::GridFlowTolerance(_)
    ));
}

fn assert_grid_track_list_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::GridTemplateRows(_)
            | CssKnownPropertyValueRef::GridTemplateColumns(_)
            | CssKnownPropertyValueRef::GridAutoRows(_)
            | CssKnownPropertyValueRef::GridAutoColumns(_)
    ));
}

fn assert_grid_template_areas_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::GridTemplateAreas(_)
    ));
}

fn assert_grid_template_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::GridTemplate(_)));
}

fn assert_grid_auto_flow_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::GridAutoFlow(_)));
}

fn assert_grid_line_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::GridRowStart(_)
            | CssKnownPropertyValueRef::GridRowEnd(_)
            | CssKnownPropertyValueRef::GridColumnStart(_)
            | CssKnownPropertyValueRef::GridColumnEnd(_)
    ));
}

fn assert_grid_line_range_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::GridRow(_) | CssKnownPropertyValueRef::GridColumn(_)
    ));
}

fn assert_grid_area_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::GridArea(_)));
}

fn assert_grid_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Grid(_)));
}

fn assert_writing_mode_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::WritingMode(_)));
}

fn assert_text_align_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::TextAlign(_)));
}

fn assert_text_align_last_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::TextAlignLast(_)));
}

fn assert_text_indent_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::TextIndent(_)));
}

fn assert_vertical_align_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::VerticalAlign(_)));
}

fn assert_font_family_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FontFamily(_)));
}

fn assert_font_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Font(_)));
}

fn assert_font_weight_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FontWeight(_)));
}

fn assert_font_style_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FontStyle(_)));
}

fn assert_font_stretch_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FontStretch(_)));
}

fn assert_font_variant_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::FontVariant(_)));
}

fn assert_font_feature_settings_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::FontFeatureSettings(_)
    ));
}

fn assert_letter_spacing_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::LetterSpacing(_)));
}

fn assert_text_wrap_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::TextWrap(_)));
}

fn assert_white_space_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::WhiteSpace(_)));
}

fn assert_word_break_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::WordBreak(_)));
}

fn assert_overflow_wrap_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::OverflowWrap(_)));
}

fn assert_text_overflow_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::TextOverflow(_)));
}

fn assert_text_decoration_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::TextDecoration(_)));
}

fn assert_text_decoration_line_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TextDecorationLine(_)
    ));
}

fn assert_text_decoration_color_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TextDecorationColor(_)
    ));
}

fn assert_text_decoration_style_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TextDecorationStyle(_)
    ));
}

fn assert_text_decoration_thickness_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TextDecorationThickness(_)
    ));
}

fn assert_text_transform_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::TextTransform(_)));
}

fn assert_background_image_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BackgroundImage(_)
    ));
}

fn assert_background_position_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BackgroundPosition(_)
    ));
}

fn assert_background_size_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::BackgroundSize(_)));
}

fn assert_background_repeat_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BackgroundRepeat(_)
    ));
}

fn assert_background_box_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BackgroundOrigin(_) | CssKnownPropertyValueRef::BackgroundClip(_)
    ));
}

fn assert_background_attachment_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::BackgroundAttachment(_)
    ));
}

fn assert_cursor_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Cursor(_)));
}

fn assert_pointer_events_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::PointerEvents(_)));
}

fn assert_user_select_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::UserSelect(_)));
}

fn assert_outline_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Outline(_)));
}

fn assert_outline_color_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::OutlineColor(_)));
}

fn assert_outline_style_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::OutlineStyle(_)));
}

fn assert_outline_width_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::OutlineWidth(_)));
}

fn assert_transform_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Transform(_)));
}

fn assert_transform_origin_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TransformOrigin(_)
    ));
}

fn assert_translate_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Translate(_)));
}

fn assert_rotate_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Rotate(_)));
}

fn assert_scale_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Scale(_)));
}

fn assert_filter_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::Filter(_) | CssKnownPropertyValueRef::BackdropFilter(_)
    ));
}

fn assert_clip_path_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::ClipPath(_)));
}

fn assert_mask_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Mask(_)));
}

fn assert_mask_image_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::MaskImage(_)));
}

fn assert_mask_size_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::MaskSize(_)));
}

fn assert_mask_position_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::MaskPosition(_)));
}

fn assert_mask_repeat_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::MaskRepeat(_)));
}

fn assert_transition_property_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TransitionProperty(_)
    ));
}

fn assert_time_list_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TransitionDuration(_)
            | CssKnownPropertyValueRef::TransitionDelay(_)
            | CssKnownPropertyValueRef::AnimationDuration(_)
            | CssKnownPropertyValueRef::AnimationDelay(_)
    ));
}

fn assert_easing_list_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::TransitionTimingFunction(_)
            | CssKnownPropertyValueRef::AnimationTimingFunction(_)
    ));
}

fn assert_transition_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Transition(_)));
}

fn assert_animation_name_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::AnimationName(_)));
}

fn assert_animation_iteration_count_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::AnimationIterationCount(_)
    ));
}

fn assert_animation_direction_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::AnimationDirection(_)
    ));
}

fn assert_animation_fill_mode_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::AnimationFillMode(_)
    ));
}

fn assert_animation_play_state_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(
        value,
        CssKnownPropertyValueRef::AnimationPlayState(_)
    ));
}

fn assert_animation_value(value: CssKnownPropertyValueRef<'_>) {
    assert!(matches!(value, CssKnownPropertyValueRef::Animation(_)));
}

macro_rules! value_case {
    ($label:literal, $property_name:literal, $authored_value:literal, $property:expr, $assertion:path) => {
        AcceptedValueCase {
            label: $label,
            property_name: $property_name,
            authored_value: $authored_value,
            expected_property: $property,
            assert_value: $assertion,
        }
    };
}

#[test]
fn invalid_mixed_declarations_emit_expected_typed_diagnostics() {
    assert_sheet_rejected(
        ".panel { width: 10px; display: inline; }",
        &ExpectedErrorKind::UnsupportedValue {
            property: Some("display"),
        },
    );
    assert_sheet_rejected(
        ".panel { width: inherit 10px; height: 20px; }",
        &ExpectedErrorKind::InvalidSyntax,
    );
}

#[test]
fn malformed_sheet_surfaces_emit_expected_typed_diagnostics() {
    assert_rejects_sheets(&[
        RejectedSheetCase {
            label: "valid declaration before invalid declaration fails the whole sheet",
            input: ".panel { width: 10px; display: inline; height: 20px; }",
            expected_error: ExpectedErrorKind::UnsupportedValue {
                property: Some("display"),
            },
        },
        RejectedSheetCase {
            label: "invalid declaration before valid declaration fails the whole sheet",
            input: ".panel { display: inline; width: 10px; }",
            expected_error: ExpectedErrorKind::UnsupportedValue {
                property: Some("display"),
            },
        },
        RejectedSheetCase {
            label: "unknown property fails the whole sheet",
            input: ".panel { widht: 10px; width: 20px; }",
            expected_error: ExpectedErrorKind::UnknownProperty { name: "widht" },
        },
        RejectedSheetCase {
            label: "unsupported at-rule fails the whole sheet",
            input: "@unknown screen { .panel { width: 10px; } }",
            expected_error: ExpectedErrorKind::UnsupportedAtRule { name: "unknown" },
        },
        RejectedSheetCase {
            label: "invalid selector fails the whole sheet",
            input: "??? { width: 10px; }",
            expected_error: ExpectedErrorKind::InvalidSelector,
        },
        RejectedSheetCase {
            label: "malformed declaration block fails the whole sheet",
            input: ".panel { width 10px; height: 20px; }",
            expected_error: ExpectedErrorKind::InvalidSyntax,
        },
        RejectedSheetCase {
            label: "trailing junk after a value fails the whole sheet",
            input: ".panel { width: 10px solid; }",
            expected_error: ExpectedErrorKind::InvalidSyntax,
        },
        RejectedSheetCase {
            label: "invalid comma-list item fails the whole sheet",
            input: ".panel { transition-duration: 150ms, solid; }",
            expected_error: ExpectedErrorKind::InvalidSyntaxOrUnsupportedValueForProperty {
                property: "transition-duration",
            },
        },
        RejectedSheetCase {
            label: "invalid shorthand component fails the whole sheet",
            input: ".panel { border: 1px solid dotted; }",
            expected_error: ExpectedErrorKind::InvalidSyntaxOrUnsupportedValueForProperty {
                property: "border",
            },
        },
    ]);
}

#[test]
fn malformed_authored_surfaces_emit_recovery_diagnostics() {
    for css in [
        "@layer reset; @layer , theme; .ok { color: black; }",
        "@scope (.card) { .ok { color: black; } @font-face { font-family: Test; src: local(Test); } } .after { color: blue; }",
        ".ok { color: black; } .bad::before .later { color: red; } .after { color: blue; }",
        ".ok { content: \"good\"; content: normal \"bad\"; color: red; }",
        ".ok { list-style: square inside; counter-reset: none item; color: red; }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn leakage_wrong_keyword_and_unit_matrix_rejects_property_family_crossovers() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "display rejects unsupported inline keyword",
            property_name: "display",
            authored_value: "inline",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "display",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "overflow rejects auto keyword",
            property_name: "overflow",
            authored_value: "auto",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "overflow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "align-items rejects content distribution keyword",
            property_name: "align-items",
            authored_value: "space-between",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "align-items",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "padding rejects auto keyword",
            property_name: "padding",
            authored_value: "auto",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "padding",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "border-width rejects percentage",
            property_name: "border-width",
            authored_value: "10%",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "border-width",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "border-color rejects border style keyword",
            property_name: "border-color",
            authored_value: "solid",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "border-color",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "font-size rejects auto keyword",
            property_name: "font-size",
            authored_value: "auto",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "font-size",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "grid-auto-flow rejects position keyword",
            property_name: "grid-auto-flow",
            authored_value: "left",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "grid-auto-flow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "cursor rejects length",
            property_name: "cursor",
            authored_value: "10px",
            expected_error: ExpectedErrorKind::InvalidSyntaxOrUnsupportedValueForProperty {
                property: "cursor",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "transition-duration rejects length unit",
            property_name: "transition-duration",
            authored_value: "10px",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "transition-duration",
            },
            property_name_should_be_recognized: true,
        },
    ]);
}

#[test]
fn rejection_malformed_functions_lists_and_shorthands_matrix() {
    for input in [
        ".panel { width: calc(10px + ); }",
        ".panel { width: calc(10px * 2px); }",
        ".panel { transform: translate(red); }",
        ".panel { filter: opacity(red); }",
        ".panel { clip-path: polygon(0 0, ); }",
        ".panel { transition-timing-function: cubic-bezier(0.1, red, 0.3, 1); }",
        ".panel { font-family: sans-serif,; }",
        ".panel { background-image: none,; }",
        ".panel { transition-property: opacity,; }",
        ".panel { animation-name: fade,; }",
        ".panel { border: 1px 2px solid; }",
        ".panel { box-shadow: inset inset 1px 2px; }",
        ".panel { text-decoration: underline underline; }",
        ".panel { transition: opacity 1s 2s 3s; }",
        ".panel { animation: fade 1s 2s 3s; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(
            matches!(
                error.kind(),
                ErrorKind::InvalidPropertyValue(_)
                    | ErrorKind::UnexpectedEnd(_)
                    | ErrorKind::UnexpectedToken(_)
                    | ErrorKind::InvalidQualifiedRule(_)
            ),
            "{input} rejected with unexpected error kind: {:?}",
            error.kind(),
        );
    }
}

#[test]
fn rejection_negative_numbers_and_public_constructor_invariants_matrix() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "flex-grow rejects negative numbers",
            property_name: "flex-grow",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "flex-grow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "flex-shrink rejects negative numbers",
            property_name: "flex-shrink",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "flex-shrink",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "padding rejects negative lengths",
            property_name: "padding",
            authored_value: "-1px",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "padding",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "border-radius rejects negative lengths",
            property_name: "border-radius",
            authored_value: "-1px",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "border-radius",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "animation-iteration-count rejects negative numbers",
            property_name: "animation-iteration-count",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "animation-iteration-count",
            },
            property_name_should_be_recognized: true,
        },
    ]);

    assert_eq!(CssFontFamilyList::try_new(Vec::new()), None);
    assert_eq!(CssGridTrackList::try_new(Vec::new()), None);
    assert_eq!(
        CssPosition::try_new(vec![
            CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Left),
            CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Right),
        ]),
        None
    );
    assert_eq!(CssTransitionList::try_new(Vec::new()), None);
    assert_eq!(
        CssAnimation::try_new(CssAnimationComponents::default()),
        None
    );
}

#[test]
fn numeric_properties_use_property_specific_authored_models() {
    assert_eq!(
        declaration_payload!(single_declaration(".panel { opacity: 0.5; }"), Opacity),
        CssOpacity::try_new(0.5).unwrap()
    );
    assert_eq!(
        declaration_payload!(single_declaration(".panel { flex-grow: 2; }"), FlexGrow),
        CssFlexFactor::try_new(2.0).unwrap()
    );
    assert_eq!(
        declaration_payload!(single_declaration(".panel { flex-shrink: 0; }"), FlexShrink),
        CssFlexFactor::try_new(0.0).unwrap()
    );
    assert_eq!(
        declaration_payload!(
            single_declaration(".panel { aspect-ratio: 1.5; }"),
            AspectRatio
        ),
        CssAspectRatio::try_new(1.5).unwrap()
    );
    assert_eq!(
        declaration_payload!(
            single_declaration(".panel { scrollbar-width: thin; }"),
            ScrollbarWidth
        ),
        CssScrollbarWidth::Thin
    );
    assert_eq!(CssOpacity::try_new(0.5).unwrap().value(), 0.5);
    assert_eq!(CssFlexFactor::try_new(2.0).unwrap().value(), 2.0);
    assert_eq!(CssAspectRatio::try_new(1.5).unwrap().value(), 1.5);
}

#[test]
fn numeric_property_models_reject_invalid_authored_values() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "flex-grow rejects negative values",
            property_name: "flex-grow",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "flex-grow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "flex-shrink rejects negative values",
            property_name: "flex-shrink",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "flex-shrink",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "aspect-ratio rejects zero",
            property_name: "aspect-ratio",
            authored_value: "0",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "aspect-ratio",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "aspect-ratio rejects negative values",
            property_name: "aspect-ratio",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "aspect-ratio",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "scrollbar-width rejects numbers",
            property_name: "scrollbar-width",
            authored_value: "8",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "scrollbar-width",
            },
            property_name_should_be_recognized: true,
        },
    ]);

    assert_eq!(CssOpacity::try_new(-0.1), None);
    assert_eq!(CssOpacity::try_new(1.1), None);
    assert_eq!(CssOpacity::try_new(f32::NAN), None);
    assert_eq!(CssFlexFactor::try_new(-1.0), None);
    assert_eq!(CssFlexFactor::try_new(f32::INFINITY), None);
    assert_eq!(CssAspectRatio::try_new(0.0), None);
    assert_eq!(CssAspectRatio::try_new(f32::NEG_INFINITY), None);
}

#[test]
fn constructor_invariants_reject_invalid_public_numeric_values() {
    assert_eq!(CssFiniteNumber::try_new(1.25).unwrap().value(), 1.25);
    assert_eq!(CssFiniteNumber::try_new(f32::NAN), None);
    assert_eq!(CssFiniteNumber::try_new(f32::INFINITY), None);

    assert_eq!(CssNonNegativeNumber::try_new(1.25).unwrap().value(), 1.25);
    assert_eq!(CssNonNegativeNumber::try_new(-0.1), None);
    assert_eq!(CssNonNegativeNumber::try_new(f32::NEG_INFINITY), None);

    assert_eq!(
        CssLengthDimension::try_new(2.0, CssLengthUnit::Rem)
            .unwrap()
            .value(),
        2.0
    );
    assert_eq!(
        CssLengthDimension::try_new(f32::NAN, CssLengthUnit::Rem),
        None
    );

    assert_eq!(CssLength::try_px(f32::NAN), None);
    assert_eq!(CssLength::try_percent(f32::INFINITY), None);
    assert_eq!(
        CssLength::try_dimension(f32::NEG_INFINITY, CssLengthUnit::Rem),
        None
    );
    assert_eq!(CssLength::try_px(3.0).unwrap(), CssLength::px(3.0));
    assert_eq!(
        CssLength::try_dimension(4.0, CssLengthUnit::Px).unwrap(),
        CssLength::px(4.0)
    );

    assert_eq!(CssGridTrackBreadth::try_fraction(f32::NAN), None);
    assert_eq!(CssGridTrackBreadth::try_fraction(-0.1), None);
    assert_eq!(
        CssGridTrackBreadth::try_fraction(1.0).unwrap(),
        CssGridTrackBreadth::try_fraction(1.0).unwrap()
    );

    assert_eq!(CssScaleValues::try_new(vec![1.0, f32::NAN]), None);
    assert_eq!(CssScaleValues::try_new(vec![f32::INFINITY]), None);

    assert_eq!(CssCalcLength::try_px(f32::NAN), None);
    assert_eq!(CssCalcLength::try_percent(f32::INFINITY), None);
    assert_eq!(
        CssCalcLength::try_dimension(f32::NEG_INFINITY, CssLengthUnit::Rem),
        None
    );
    assert_eq!(
        CssCalcLengthTerm::add(CssCalcLength::try_px(1.0).unwrap()),
        CssCalcLengthTerm::add(CssCalcLength::px(1.0))
    );

    assert_eq!(CssFlexFactor::try_new(f32::NAN), None);
    assert_eq!(CssFlexFactor::try_new(-1.0), None);
    assert_eq!(
        CssFlex::components(
            CssFlexFactor::try_new(1.0).unwrap(),
            Some(CssFlexFactor::try_new(0.0).unwrap()),
            Some(CssLength::px(2.0)),
        ),
        CssFlex::Components {
            grow: CssFlexFactor::try_new(1.0).unwrap(),
            shrink: Some(CssFlexFactor::try_new(0.0).unwrap()),
            basis: Some(CssLength::px(2.0)),
        }
    );
}

#[test]
fn media_query_list_constructor_requires_queries() {
    assert_eq!(CssMediaQueryList::try_new(Vec::new()), None);
    assert!(
        CssMediaQueryList::try_new(vec![CssMediaQuery::Typed(CssTypedMediaQuery::new(
            None,
            CssMediaType::Screen,
            None,
            test_media_position(),
        ))])
        .is_some()
    );
}

#[test]
fn media_condition_list_constructor_requires_at_least_two_conditions() {
    let width = CssMediaCondition::new(
        CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(CssRangeFeature::new(
            Some(CssQueryComparison::GreaterThanOrEqual),
            CssQueryLength::try_new(600.0, CssLengthUnit::Px).unwrap(),
        ))),
        test_media_position(),
    );
    assert_eq!(CssMediaConditionList::try_new(Vec::new()), None);
    assert_eq!(CssMediaConditionList::try_new(vec![width.clone()]), None);
    assert!(CssMediaConditionList::try_new(vec![width.clone(), width]).is_some());
}

#[test]
fn media_feature_numeric_constructors_enforce_query_invariants() {
    assert_eq!(
        CssQueryLength::try_new(12.0, CssLengthUnit::Rem)
            .unwrap()
            .value()
            .value(),
        12.0
    );
    assert_eq!(
        CssQueryLength::try_new(12.0, CssLengthUnit::Rem)
            .unwrap()
            .unit(),
        CssLengthUnit::Rem
    );
    assert_eq!(CssQueryLength::try_new(-0.1, CssLengthUnit::Px), None);
    assert_eq!(CssQueryLength::try_new(f32::NAN, CssLengthUnit::Px), None);
    assert_eq!(CssQueryLength::unitless_zero().value().value(), 0.0);
    assert_eq!(CssQueryLength::unitless_zero().authored_unit(), None);
    assert_eq!(CssQueryLength::unitless_zero().unit(), CssLengthUnit::Px);

    assert_eq!(
        CssRatio::try_new(16.0, 9.0).unwrap().numerator().value(),
        16.0
    );
    assert_eq!(
        CssRatio::try_new(16.0, 9.0).unwrap().denominator().value(),
        9.0
    );
    assert_eq!(CssRatio::try_new(-1.0, 1.0), None);
    assert_eq!(CssRatio::try_new(1.0, 0.0), None);
    assert_eq!(CssRatio::try_new(f32::INFINITY, 1.0), None);

    assert_eq!(
        CssResolution::try_new(2.0, CssResolutionUnit::Dppx)
            .unwrap()
            .unit(),
        CssResolutionUnit::Dppx
    );
    assert_eq!(CssResolution::try_new(0.0, CssResolutionUnit::Dpi), None);
    assert_eq!(
        CssResolution::try_new(f32::NAN, CssResolutionUnit::Dpi),
        None
    );
}

#[test]
fn media_feature_names_are_canonical() {
    assert_eq!(
        CssMediaFeatureQuery::Width(CssRangeFeature::new(
            None,
            CssQueryLength::try_new(1.0, CssLengthUnit::Px).unwrap(),
        ))
        .name(),
        "width"
    );
    assert_eq!(
        CssMediaFeatureQuery::PrefersColorScheme(CssColorSchemePreference::Dark).name(),
        "prefers-color-scheme"
    );
    assert_eq!(
        CssMediaFeatureQuery::AnyPointer(CssPointerCapability::Fine).name(),
        "any-pointer"
    );
}

#[test]
fn media_query_parser_accepts_supported_types_ranges_and_conditions() {
    for css in [
        "screen",
        "print",
        "speech",
        "tv",
        "screen and (min-width: 600px)",
        "(device-width: 800px)",
        "(aspect-ratio: 16/9)",
        "(color-index)",
        "(scan: progressive)",
        "(grid: 1)",
        "(width >= 600px)",
        "(orientation: landscape)",
        "(prefers-color-scheme: dark)",
        "(hover: hover) and (pointer: fine)",
        "not screen and (max-width: 400px)",
        "screen, print",
    ] {
        parse_media_query_list_for_test(css).unwrap_or_else(|error| {
            panic!("{css} should parse as a media query list: {error}");
        });
    }
}

#[test]
fn media_query_parser_preserves_defined_false_and_rejects_malformed_conditions() {
    for css in [
        "future-screen",
        "(unknown-feature: yes)",
        "(width: auto)",
        "(width: min-content)",
    ] {
        let query = parse_media_query_list_for_test(css)
            .unwrap_or_else(|error| panic!("{css} should be valid defined-false syntax: {error}"));
        assert!(!matches!(query.queries(), [CssMediaQuery::Never(_)]));
    }

    for css in ["(width >= )", "screen and", "screen or print"] {
        assert!(
            parse_media_query_list_for_test(css).is_err(),
            "{css} should reject"
        );
    }
}

#[test]
fn media_query_parser_preserves_typed_query_structure() {
    let query_list = parse_media_query_list_for_test("not screen and (max-width: 400px)").unwrap();
    let [CssMediaQuery::Typed(query)] = query_list.queries() else {
        panic!("expected one typed media query");
    };

    assert_eq!(query.modifier(), Some(CssMediaQueryModifier::Not));
    assert_eq!(query.media_type(), CssMediaType::Screen);
    let Some(condition) = query.condition() else {
        panic!("expected width condition");
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(width)) = condition.kind()
    else {
        panic!("expected width condition");
    };
    assert_eq!(
        width.comparison(),
        Some(CssQueryComparison::LessThanOrEqual)
    );
    assert_eq!(width.value().value().value(), 400.0);
    assert_eq!(width.value().unit(), CssLengthUnit::Px);
}

#[test]
fn media_query_parser_preserves_condition_only_range_structure() {
    let query_list = parse_media_query_list_for_test("(width >= 600px)").unwrap();
    let [CssMediaQuery::Condition(condition)] = query_list.queries() else {
        panic!("expected one condition-only width query");
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(width)) = condition.kind()
    else {
        panic!("expected one condition-only width query");
    };

    assert_eq!(
        width.comparison(),
        Some(CssQueryComparison::GreaterThanOrEqual)
    );
    assert_eq!(width.value().value().value(), 600.0);
    assert_eq!(width.value().unit(), CssLengthUnit::Px);
}

#[test]
fn media_query_parser_preserves_discrete_and_condition_list_structure() {
    let query_list = parse_media_query_list_for_test("(hover: hover) and (pointer: fine)").unwrap();
    let [CssMediaQuery::Condition(condition)] = query_list.queries() else {
        panic!("expected one condition-only and query");
    };
    let CssMediaConditionKind::And(list) = condition.kind() else {
        panic!("expected one condition-only and query");
    };
    let [hover, pointer] = list.conditions() else {
        panic!("expected two conditions");
    };

    assert!(matches!(
        hover.kind(),
        CssMediaConditionKind::Feature(CssMediaFeatureQuery::Hover(CssHoverCapability::Hover))
    ));
    assert!(matches!(
        pointer.kind(),
        CssMediaConditionKind::Feature(CssMediaFeatureQuery::Pointer(CssPointerCapability::Fine))
    ));
}

#[test]
fn media_query_parser_preserves_comma_separated_queries() {
    let query_list = parse_media_query_list_for_test("screen, print").unwrap();
    let [screen, print] = query_list.queries() else {
        panic!("expected two media queries");
    };

    let CssMediaQuery::Typed(screen) = screen else {
        panic!("expected screen query")
    };
    let CssMediaQuery::Typed(print) = print else {
        panic!("expected print query")
    };
    assert_eq!(screen.media_type(), CssMediaType::Screen);
    assert_eq!(print.media_type(), CssMediaType::Print);
}

#[test]
fn container_name_constructor_rejects_invalid_and_reserved_names() {
    assert_eq!(
        CssContainerName::try_new("sidebar").unwrap().as_str(),
        "sidebar"
    );
    assert_eq!(
        CssContainerName::try_new("layout-pane").unwrap().as_str(),
        "layout-pane"
    );

    for name in ["", " \t\n ", "two names", "1pane", "pane;", "none"] {
        assert_eq!(
            CssContainerName::try_new(name),
            None,
            "{name} should reject"
        );
    }
    for reserved in [
        "and",
        "or",
        "not",
        "style",
        "NoNe",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "InItIaL",
    ] {
        assert_eq!(
            CssContainerName::try_new(reserved),
            None,
            "{reserved} should reject"
        );
    }
}

#[test]
fn container_condition_list_constructor_requires_at_least_two_conditions() {
    let width =
        CssContainerCondition::Feature(CssContainerFeatureQuery::Width(CssRangeFeature::new(
            Some(CssQueryComparison::GreaterThan),
            CssQueryLength::try_new(600.0, CssLengthUnit::Px).unwrap(),
        )));
    assert_eq!(CssContainerConditionList::try_new(Vec::new()), None);
    assert_eq!(
        CssContainerConditionList::try_new(vec![width.clone()]),
        None
    );
    assert!(CssContainerConditionList::try_new(vec![width.clone(), width]).is_some());
}

#[test]
fn font_face_descriptor_collection_requires_family_and_src() {
    assert!(CssFontFaceDescriptors::try_new(None, None, None, None, None, None, None).is_none());
    assert!(CssFontFaceUrlSource::try_new("", None, Vec::new()).is_none());
    assert!(CssFontFaceUrlSource::try_new("   ", None, Vec::new()).is_none());

    let family = CssFontFaceFamily::try_new("Avenir Next").unwrap();
    let src = CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Local(
        CssFontLocalName::try_new("Avenir Next").unwrap(),
    )])
    .unwrap();

    assert!(
        CssFontFaceDescriptors::try_new(
            Some(descriptor_occurrence(family.clone())),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .is_none()
    );
    assert!(
        CssFontFaceDescriptors::try_new(
            None,
            Some(descriptor_occurrence(src.clone())),
            None,
            None,
            None,
            None,
            None,
        )
        .is_none()
    );

    let descriptors = CssFontFaceDescriptors::try_new(
        Some(descriptor_occurrence(family.clone())),
        Some(descriptor_occurrence(src.clone())),
        Some(descriptor_occurrence(
            CssFontFaceWeight::try_range(400.0, 700.0).unwrap(),
        )),
        Some(descriptor_occurrence(CssFontFaceStyle::Oblique(Some(
            CssFontFaceObliqueRange::try_new(-10.0, Some(20.0)).unwrap(),
        )))),
        Some(descriptor_occurrence(
            CssFontFaceStretch::try_range_percent(75.0, 125.0).unwrap(),
        )),
        Some(descriptor_occurrence(CssFontDisplay::Swap)),
        Some(descriptor_occurrence(
            CssUnicodeRangeList::try_new(vec![CssUnicodeRange::try_new(0, 0x7f).unwrap()]).unwrap(),
        )),
    )
    .unwrap();

    assert_eq!(descriptors.font_family().value(), &family);
    assert_eq!(descriptors.src().value(), &src);
    assert_eq!(
        descriptors.font_weight().unwrap().start().value().value(),
        400.0
    );
    assert!(matches!(
        descriptors.font_style(),
        Some(occurrence)
            if matches!(occurrence.value(), CssFontFaceStyle::Oblique(Some(_)))
    ));
    assert_eq!(
        descriptors
            .font_stretch()
            .unwrap()
            .end()
            .unwrap()
            .percent()
            .value(),
        125.0
    );
    assert_eq!(
        descriptors
            .font_display()
            .map(CssDescriptorOccurrence::value),
        Some(&CssFontDisplay::Swap)
    );
    assert_eq!(
        descriptors.unicode_range().unwrap().ranges(),
        &[CssUnicodeRange::try_new(0, 0x7f).unwrap()]
    );
}

#[test]
fn font_face_string_constructors_reject_empty_values() {
    assert_eq!(
        CssFontFaceFamily::try_new("Avenir Next").unwrap().as_str(),
        "Avenir Next"
    );
    assert_eq!(
        CssFontLocalName::try_new("Avenir Next").unwrap().as_str(),
        "Avenir Next"
    );
    assert_eq!(
        CssFontFaceUrlSource::try_new(
            "fonts/avenir.woff2",
            Some(CssFontFormatHint::Woff2),
            vec![CssFontTechHint::Variations],
        )
        .unwrap()
        .url(),
        "fonts/avenir.woff2"
    );

    for value in ["", " \t\n "] {
        assert_eq!(CssFontFaceFamily::try_new(value), None);
        assert_eq!(CssFontLocalName::try_new(value), None);
        assert_eq!(CssFontFaceUrlSource::try_new(value, None, Vec::new()), None);
    }
}

#[test]
fn font_face_source_and_unicode_lists_reject_empty_values() {
    assert_eq!(CssFontFaceSourceList::try_new(Vec::new()), None);
    assert_eq!(CssUnicodeRangeList::try_new(Vec::new()), None);

    let url = CssFontFaceUrlSource::try_new(
        "fonts/avenir.woff2",
        Some(CssFontFormatHint::Woff2),
        vec![CssFontTechHint::ColorCOLRv1],
    )
    .unwrap();
    assert_eq!(url.format(), Some(&CssFontFormatHint::Woff2));
    assert_eq!(url.tech(), &[CssFontTechHint::ColorCOLRv1]);

    let sources = CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Url(url)]).unwrap();
    assert!(matches!(sources.sources(), [CssFontFaceSource::Url(_)]));

    let ranges =
        CssUnicodeRangeList::try_new(vec![CssUnicodeRange::try_new(0x20, 0x7e).unwrap()]).unwrap();
    assert_eq!(ranges.ranges()[0].start(), 0x20);
    assert_eq!(ranges.ranges()[0].end(), 0x7e);
}

#[test]
fn font_face_numeric_descriptors_enforce_invariants() {
    assert_eq!(
        CssFontFaceWeightValue::try_new(1.0)
            .unwrap()
            .value()
            .value(),
        1.0
    );
    assert_eq!(
        CssFontFaceWeightValue::try_new(1000.0)
            .unwrap()
            .value()
            .value(),
        1000.0
    );
    assert_eq!(CssFontFaceWeightValue::try_new(0.999), None);
    assert_eq!(CssFontFaceWeightValue::try_new(1000.001), None);
    assert_eq!(CssFontFaceWeightValue::try_new(f32::NAN), None);
    assert_eq!(CssFontFaceWeight::try_range(700.0, 400.0), None);
    assert_eq!(
        CssFontFaceWeight::try_single(400.0)
            .unwrap()
            .start()
            .value()
            .value(),
        400.0
    );

    assert_eq!(
        CssFontFaceStretchValue::try_new_percent(0.0)
            .unwrap()
            .percent()
            .value(),
        0.0
    );
    assert_eq!(CssFontFaceStretchValue::try_new_percent(-0.1), None);
    assert_eq!(
        CssFontFaceStretchValue::try_new_percent(f32::INFINITY),
        None
    );
    assert_eq!(CssFontFaceStretch::try_range_percent(125.0, 75.0), None);
    assert_eq!(
        CssFontFaceStretch::try_single_percent(100.0)
            .unwrap()
            .start()
            .percent()
            .value(),
        100.0
    );

    assert_eq!(
        CssFontFaceObliqueRange::try_new(-90.0, Some(90.0))
            .unwrap()
            .end_degrees()
            .unwrap()
            .value(),
        90.0
    );
    assert_eq!(CssFontFaceObliqueRange::try_new(-90.1, None), None);
    assert_eq!(CssFontFaceObliqueRange::try_new(90.1, None), None);
    assert_eq!(CssFontFaceObliqueRange::try_new(f32::NAN, None), None);
    assert_eq!(CssFontFaceObliqueRange::try_new(10.0, Some(0.0)), None);

    assert_eq!(
        CssUnicodeRange::try_new(0x10ffff, 0x10ffff).unwrap().end(),
        0x10ffff
    );
    assert_eq!(CssUnicodeRange::try_new(2, 1), None);
    assert_eq!(CssUnicodeRange::try_new(0, 0x110000), None);
}

#[test]
fn font_face_rule_accessors_expose_authored_structure() {
    let descriptors = CssFontFaceDescriptors::try_new(
        Some(descriptor_occurrence(
            CssFontFaceFamily::try_new("Avenir Next").unwrap(),
        )),
        Some(descriptor_occurrence(
            CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Url(
                CssFontFaceUrlSource::try_new("fonts/avenir.woff2", None, Vec::new()).unwrap(),
            )])
            .unwrap(),
        )),
        None,
        Some(descriptor_occurrence(CssFontFaceStyle::Normal)),
        None,
        Some(descriptor_occurrence(CssFontDisplay::Auto)),
        None,
    )
    .unwrap();
    let location = source_position(9, 5);
    let rule = CssFontFaceRule::new(descriptors.clone(), location);

    assert_eq!(rule.descriptors(), &descriptors);
    assert_eq!(rule.position(), location);
    assert_eq!(CssRule::FontFace(rule.clone()), CssRule::FontFace(rule));
}

#[test]
fn font_face_rule_parser_accepts_descriptor_block() {
    let sheet = parse_sheet(
        r#"@font-face {
            font-family: "Inter";
            src: url("inter.woff2") format("woff2");
            font-weight: 400 700;
            font-style: normal;
            font-display: swap;
            unicode-range: U+0000-00FF, U+0100-017F;
        }"#,
    )
    .unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one font-face rule");
    };

    let descriptors = font_face_rule(rule).descriptors();
    assert_eq!(descriptors.font_family().as_str(), "Inter");
    let [CssFontFaceSource::Url(source)] = descriptors.src().sources() else {
        panic!("expected one URL font source");
    };
    assert_eq!(source.url(), "inter.woff2");
    assert_eq!(source.format(), Some(&CssFontFormatHint::Woff2));
    assert!(source.tech().is_empty());
    assert_eq!(
        descriptors.font_weight().unwrap().start().value().value(),
        400.0
    );
    assert_eq!(
        descriptors
            .font_weight()
            .unwrap()
            .end()
            .unwrap()
            .value()
            .value(),
        700.0
    );
    assert_eq!(
        descriptors.font_style().map(CssDescriptorOccurrence::value),
        Some(&CssFontFaceStyle::Normal)
    );
    assert_eq!(
        descriptors
            .font_display()
            .map(CssDescriptorOccurrence::value),
        Some(&CssFontDisplay::Swap)
    );
    assert_eq!(
        descriptors.unicode_range().unwrap().ranges(),
        &[
            CssUnicodeRange::try_new(0x0000, 0x00ff).unwrap(),
            CssUnicodeRange::try_new(0x0100, 0x017f).unwrap()
        ]
    );
}

#[test]
fn font_face_rule_parser_accepts_source_list_forms() {
    let sheet = parse_sheet(
        r#"@font-face {
            font-family: Avenir Next;
            src: local("Inter"), url("inter.woff2") format("woff2"), url("inter-var.woff2") tech(variations);
        }"#,
    )
    .unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one font-face rule");
    };

    let descriptors = font_face_rule(rule).descriptors();
    assert_eq!(descriptors.font_family().as_str(), "Avenir Next");
    let [
        CssFontFaceSource::Local(local),
        CssFontFaceSource::Url(woff2),
        CssFontFaceSource::Url(variable),
    ] = descriptors.src().sources()
    else {
        panic!("expected local source and two URL sources");
    };
    assert_eq!(local.as_str(), "Inter");
    assert_eq!(woff2.url(), "inter.woff2");
    assert_eq!(woff2.format(), Some(&CssFontFormatHint::Woff2));
    assert!(woff2.tech().is_empty());
    assert_eq!(variable.url(), "inter-var.woff2");
    assert_eq!(variable.format(), None);
    assert_eq!(variable.tech(), &[CssFontTechHint::Variations]);
}

#[test]
fn font_face_rule_parser_accepts_strict_numeric_ranges() {
    let sheet = parse_sheet(
        r#"@font-face {
            font-family: Inter;
            src: url(inter.woff2);
            font-style: oblique -10deg 20deg;
            font-stretch: 75% 125%;
        }"#,
    )
    .unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one font-face rule");
    };

    let descriptors = font_face_rule(rule).descriptors();
    let Some(CssFontFaceStyle::Oblique(Some(oblique))) =
        descriptors.font_style().map(CssDescriptorOccurrence::value)
    else {
        panic!("expected oblique range");
    };
    assert_eq!(oblique.start_degrees().value(), -10.0);
    assert_eq!(oblique.end_degrees().unwrap().value(), 20.0);
    assert_eq!(
        descriptors
            .font_stretch()
            .unwrap()
            .start()
            .percent()
            .value(),
        75.0
    );
    assert_eq!(
        descriptors
            .font_stretch()
            .unwrap()
            .end()
            .unwrap()
            .percent()
            .value(),
        125.0
    );
}

#[test]
fn font_face_rule_parser_rejects_invalid_descriptor_blocks() {
    for css in [
        "@font-face { font-family: Inter; }",
        "@font-face { src: url(a.woff2); }",
        "@font-face { font-family: Inter; src: url(a.woff2); unknown: x; }",
        "@font-face { font-family: Inter; src: url(a.woff2); @media screen {} }",
        "@font-face { font-family: Inter; src: url(a.woff2); .nested {} }",
        "@font-face { font-family: Inter; src: url(a.woff2); font-weight: bolder; }",
        "@font-face { font-family: Inter; src: url(a.woff2) format(woff3); }",
        "@font-face { font-family: Inter; src: url(a.woff2) tech(color-paint); }",
        r#"@font-face { font-family: Inter; src: url("a.woff2") tech(variations) format(woff2); }"#,
        r#"@font-face { font-family: Inter; src: url("a.woff2") tech(variations color-svg); }"#,
        "@font-face { font-family: Inter; src: url(a.woff2); unicode-range: U+110000-110001; }",
        ".panel { src: url(a.woff2); }",
    ] {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn container_condition_parser_accepts_supported_size_style_and_boolean_conditions() {
    for css in [
        "(width > 600px)",
        "(inline-size >= 30rem)",
        "(aspect-ratio > 1 / 1)",
        "(orientation: landscape)",
        "not (width < 300px)",
        "(width > 600px) and (orientation: landscape)",
        "(width > 600px) or (orientation: portrait)",
        "style(--theme)",
        "style(--theme: dark)",
    ] {
        parse_container_condition_for_test(css).unwrap_or_else(|error| {
            panic!("{css} should parse as a container condition: {error}");
        });
    }
}

#[test]
fn container_condition_parser_rejects_unknown_features_and_malformed_conditions() {
    for css in [
        "(unknown > 1px)",
        "(width: auto)",
        "(width: min-content)",
        "(aspect-ratio: -1 / 1)",
        "(aspect-ratio: 1 / 0)",
        "style(color: red)",
        "scroll-state(stuck: top)",
        "(width > )",
    ] {
        assert!(
            parse_container_condition_for_test(css).is_err(),
            "{css} should reject"
        );
    }
}

#[test]
fn container_condition_parser_preserves_size_feature_structure() {
    let condition = parse_container_condition_for_test("(inline-size >= 30rem)").unwrap();
    let CssContainerCondition::Feature(CssContainerFeatureQuery::InlineSize(inline_size)) =
        condition
    else {
        panic!("expected inline-size feature");
    };

    assert_eq!(
        inline_size.comparison(),
        Some(CssQueryComparison::GreaterThanOrEqual)
    );
    assert_eq!(inline_size.value().value().value(), 30.0);
    assert_eq!(inline_size.value().unit(), CssLengthUnit::Rem);
}

#[test]
fn container_condition_parser_preserves_ratio_and_logic_structure() {
    let condition =
        parse_container_condition_for_test("(aspect-ratio > 1 / 1) and (orientation: landscape)")
            .unwrap();
    let CssContainerCondition::And(list) = condition else {
        panic!("expected and condition list");
    };
    let [ratio, orientation] = list.conditions() else {
        panic!("expected two conditions");
    };

    let CssContainerCondition::Feature(CssContainerFeatureQuery::AspectRatio(ratio)) = ratio else {
        panic!("expected aspect-ratio feature");
    };
    assert_eq!(ratio.comparison(), Some(CssQueryComparison::GreaterThan));
    assert_eq!(ratio.value().numerator().value(), 1.0);
    assert_eq!(ratio.value().denominator().value(), 1.0);

    assert_eq!(
        orientation,
        &CssContainerCondition::Feature(CssContainerFeatureQuery::Orientation(
            CssOrientation::Landscape
        ))
    );
}

#[test]
fn container_style_query_preserves_custom_property_presence() {
    let condition = parse_container_condition_for_test("style(--theme)").unwrap();
    assert_eq!(
        condition,
        CssContainerCondition::Style(CssContainerStyleQuery::CustomPropertyPresence(
            CssCustomPropertyName::try_new("--theme").unwrap()
        ))
    );
}

#[test]
fn container_style_query_preserves_custom_property_authored_value() {
    let condition = parse_container_condition_for_test("style(--theme: dark)").unwrap();
    assert_eq!(
        condition,
        CssContainerCondition::Style(CssContainerStyleQuery::CustomPropertyValue {
            name: CssCustomPropertyName::try_new("--theme").unwrap(),
            value: CssAuthoredDeclarationValue::try_new("dark").unwrap(),
        })
    );
}

#[test]
fn container_rule_accessors_expose_authored_structure() {
    let name = CssContainerName::try_new("sidebar").unwrap();
    let condition =
        parse_container_condition_for_test("(inline-size > 30rem)").expect("condition parses");
    let location = source_position(4, 9);
    let nested = CssRule::Style(CssStyleRule::new(
        CssSelector::Class("card".to_owned()),
        CssDeclarationList::new(Vec::new()),
        location,
    ));
    let rule = CssContainerRule::new(
        Some(name.clone()),
        condition.clone(),
        vec![nested.clone()],
        location,
    );

    assert_eq!(style_rule(&nested).position(), location);
    assert_eq!(rule.name(), Some(&name));
    assert_eq!(rule.condition(), &condition);
    assert_eq!(rule.rules(), &[nested]);
    assert_eq!(rule.position(), location);
    assert_eq!(CssRule::Container(rule.clone()), CssRule::Container(rule));
}

#[test]
fn container_rule_parser_accepts_unnamed_named_and_style_conditions() {
    let sheet = parse_sheet("@container (inline-size > 30rem) { .card { color: black; } }")
        .expect("unnamed container rule parses");
    let [rule] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let rule = container_rule(rule);
    assert_eq!(rule.name(), None);
    assert_eq!(rule.position().byte_offset().value(), 0);
    assert_eq!(rule.position().line().value(), 0);
    assert_eq!(rule.position().column().value(), 0);
    assert!(matches!(
        rule.condition(),
        CssContainerCondition::Feature(CssContainerFeatureQuery::InlineSize(_))
    ));
    let [nested] = rule.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(nested).selector(),
        &CssSelector::Class("card".to_owned())
    );

    let sheet = parse_sheet("@container sidebar (width >= 300px) { .title { color: black; } }")
        .expect("named container rule parses");
    let [rule] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let rule = container_rule(rule);
    assert_eq!(
        rule.name(),
        Some(&CssContainerName::try_new("sidebar").unwrap())
    );
    assert!(matches!(
        rule.condition(),
        CssContainerCondition::Feature(CssContainerFeatureQuery::Width(_))
    ));

    let sheet = parse_sheet("@container style(--theme: dark) { .title { color: black; } }")
        .expect("style query container rule parses");
    let [rule] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let rule = container_rule(rule);
    assert_eq!(rule.name(), None);
    assert_eq!(
        rule.condition(),
        &CssContainerCondition::Style(CssContainerStyleQuery::CustomPropertyValue {
            name: CssCustomPropertyName::try_new("--theme").unwrap(),
            value: CssAuthoredDeclarationValue::try_new("dark").unwrap(),
        })
    );
}

#[test]
fn nested_conditional_rules_allow_media_and_container_in_either_direction() {
    let sheet =
        parse_sheet("@media screen { @container (width > 300px) { .panel { color: black; } } }")
            .expect("media rule can contain a container rule");
    let [media] = sheet.rules() else {
        panic!("expected one media rule");
    };
    let media = media_rule(media);
    let [container] = media.rules() else {
        panic!("expected one nested container rule");
    };
    let container = container_rule(container);
    let [style] = container.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(style).selector(),
        &CssSelector::Class("panel".to_owned())
    );

    let sheet =
        parse_sheet("@container (width > 300px) { @media screen { .panel { color: black; } } }")
            .expect("container rule can contain a media rule");
    let [container] = sheet.rules() else {
        panic!("expected one container rule");
    };
    let container = container_rule(container);
    let [media] = container.rules() else {
        panic!("expected one nested media rule");
    };
    let media = media_rule(media);
    let [style] = media.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(style).selector(),
        &CssSelector::Class("panel".to_owned())
    );
}

#[test]
fn container_rule_parser_rejects_unknown_features_imports_and_invalid_declarations() {
    assert!(parse_sheet("@container (unknown > 1px) { .card { color: black; } }").is_err());
    assert!(parse_sheet("@container (width > 300px) { @import \"x.css\"; }").is_err());
    assert!(parse_sheet("@container (width > 300px) { .card { made-up: 1; } }").is_err());
}

#[test]
fn media_rule_parser_accepts_style_rule_body() {
    let sheet =
        parse_sheet("@media screen and (min-width: 600px) { .panel { color: black; } }").unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected one media rule");
    };
    let rule = media_rule(rule);

    let [CssMediaQuery::Typed(query)] = rule.query().queries() else {
        panic!("expected typed media query");
    };
    assert_eq!(query.media_type(), CssMediaType::Screen);
    assert!(query.condition().is_some());
    assert_eq!(rule.position().byte_offset().value(), 0);
    assert_eq!(rule.position().line().value(), 0);
    assert_eq!(rule.position().column().value(), 0);

    let [nested] = rule.rules() else {
        panic!("expected one nested style rule");
    };
    assert_eq!(
        style_rule(nested).selector(),
        &CssSelector::Class("panel".to_owned())
    );
}

#[test]
fn media_rule_parser_accepts_nested_media_rule() {
    let sheet =
        parse_sheet("@media screen { @media (min-width: 600px) { .panel { color: black; } } }")
            .unwrap();
    let [outer] = sheet.rules() else {
        panic!("expected one outer media rule");
    };
    let outer = media_rule(outer);
    let [inner] = outer.rules() else {
        panic!("expected one inner media rule");
    };
    let inner = media_rule(inner);
    let [nested] = inner.rules() else {
        panic!("expected one nested style rule");
    };

    assert_eq!(
        style_rule(nested).selector(),
        &CssSelector::Class("panel".to_owned())
    );
}

#[test]
fn media_rule_parser_retains_defined_false_features_and_rejects_invalid_bodies() {
    let sheet = parse_sheet("@media (unknown: yes) { .panel { color: black; } }").unwrap();
    let [rule] = sheet.rules() else {
        panic!("expected retained media rule")
    };
    assert!(matches!(
        media_rule(rule).query().queries(),
        [CssMediaQuery::Condition(condition)]
            if matches!(condition.kind(), CssMediaConditionKind::DefinedFalse(_))
    ));
    assert!(parse_sheet("@media screen { .panel { made-up: value; } }").is_err());
}

#[test]
fn advanced_css_surface_matrix_accepts_supported_forms() {
    let accepted = [
        ".toolbar > button[aria-expanded=true] { color: black; }",
        ".stack .item:hover { color: black; }",
        r#"@import url("theme.css") screen and (min-width: 600px);"#,
        "@media (prefers-color-scheme: dark) { .panel { color: black; } }",
        "@container sidebar (inline-size > 30rem) { .panel { color: black; } }",
        r#"@font-face { font-family: Inter; src: url("inter.woff2") format("woff2"); }"#,
        "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }",
        ".field:has(> .icon) { color: black; }",
    ];

    for css in accepted {
        assert!(parse_sheet(css).is_ok(), "{css} should parse");
    }
}

#[test]
fn advanced_css_surface_matrix_rejects_unsupported_forms() {
    let rejected = [
        r#"@import url("late.css"); .panel { color: black; } @import url("later.css");"#,
        r#"@import url("theme.css") supports(display: grid) layer(theme);"#,
        "@font-face { font-family: Inter; }",
        ".field:has(::before) { color: black; }",
        "[svg|href] { color: black; }",
        ".col || .cell { color: black; }",
        "@container scroll-state(stuck: top) { .panel { color: black; } }",
    ];

    for css in rejected {
        assert!(parse_sheet(css).is_err(), "{css} should reject");
    }
}

#[test]
fn advanced_css_rule_surface_is_structurally_accessible() {
    let sheet = parse_sheet(
        r#"
            @import url("theme.css") screen and (min-width: 600px);
            @font-face { font-family: Inter; src: url("inter.woff2") format("woff2"); }
            @media (prefers-color-scheme: dark) { .panel { color: black; } }
            @container sidebar (inline-size > 30rem) {
                .toolbar > button[aria-expanded=true] { color: black; }
            }
        "#,
    )
    .unwrap();

    let [
        CssRule::Import(import),
        CssRule::FontFace(font_face),
        CssRule::Media(media),
        CssRule::Container(container),
    ] = sheet.rules()
    else {
        panic!("expected import, font-face, media, and container rules");
    };

    let CssImportTarget::Url(import_url) = import.target() else {
        panic!("expected URL import target");
    };
    assert_eq!(import_url.as_str(), "theme.css");
    let [CssMediaQuery::Typed(import_query)] = import.media().unwrap().queries() else {
        panic!("expected typed import media query");
    };
    assert_eq!(import_query.media_type(), CssMediaType::Screen);
    let Some(import_condition) = import_query.condition() else {
        panic!("expected import width condition");
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(width)) =
        import_condition.kind()
    else {
        panic!("expected import width condition");
    };
    assert_eq!(
        width.comparison(),
        Some(CssQueryComparison::GreaterThanOrEqual)
    );
    assert_eq!(width.value().value().value(), 600.0);
    assert_eq!(width.value().unit(), CssLengthUnit::Px);

    let descriptors = font_face.descriptors();
    assert_eq!(descriptors.font_family().as_str(), "Inter");
    let [CssFontFaceSource::Url(source)] = descriptors.src().sources() else {
        panic!("expected one font-face URL source");
    };
    assert_eq!(source.url(), "inter.woff2");
    assert_eq!(source.format(), Some(&CssFontFormatHint::Woff2));

    let [CssMediaQuery::Condition(media_condition)] = media.query().queries() else {
        panic!("expected prefers-color-scheme media condition");
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::PrefersColorScheme(color_scheme)) =
        media_condition.kind()
    else {
        panic!("expected prefers-color-scheme media condition");
    };
    assert_eq!(color_scheme, &CssColorSchemePreference::Dark);
    let [media_nested] = media.rules() else {
        panic!("expected one nested media rule");
    };
    assert_eq!(
        style_rule(media_nested).selector(),
        &CssSelector::Class("panel".to_owned())
    );

    assert_eq!(
        container.name(),
        Some(&CssContainerName::try_new("sidebar").unwrap())
    );
    let CssContainerCondition::Feature(CssContainerFeatureQuery::InlineSize(inline_size)) =
        container.condition()
    else {
        panic!("expected inline-size container condition");
    };
    assert_eq!(
        inline_size.comparison(),
        Some(CssQueryComparison::GreaterThan)
    );
    assert_eq!(inline_size.value().value().value(), 30.0);
    assert_eq!(inline_size.value().unit(), CssLengthUnit::Rem);
    let [container_nested] = container.rules() else {
        panic!("expected one nested container rule");
    };
    let CssSelector::Complex(selector) = style_rule(container_nested).selector() else {
        panic!("expected complex selector");
    };
    assert_eq!(selector.first().classes(), &["toolbar".to_owned()]);
    let [part] = selector.rest() else {
        panic!("expected one complex selector part");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(part.selector().tag().map(String::as_str), Some("button"));
    let [attribute] = part.selector().attributes() else {
        panic!("expected one attribute selector");
    };
    assert_eq!(attribute.name().as_str(), "aria-expanded");
    assert_eq!(
        attribute.matcher(),
        &CssAttributeMatcher::Equals("true".to_owned())
    );
    assert_eq!(
        attribute.case_sensitivity(),
        CssAttributeCaseSensitivity::DocumentDefault
    );
}

#[test]
fn keyframes_and_flattened_nesting_are_structurally_accessible() {
    let sheet = parse_sheet(
        r#"@keyframes fade {
            from { opacity: 0; }
            to { opacity: 1; }
        }
        .card {
            color: black;
            &:hover { opacity: 0.9; }
            @media (min-width: 600px) {
                > .title { color: white; }
            }
        }"#,
    )
    .unwrap();

    let [keyframes, base, hover, media] = sheet.rules() else {
        panic!("expected keyframes and flattened style output");
    };

    let CssRule::Keyframes(keyframes) = keyframes else {
        panic!("expected keyframes rule");
    };
    assert_eq!(
        keyframes.name(),
        &CssKeyframesName::Ident(CssCustomIdent::new("fade"))
    );
    let [from, to] = keyframes.blocks() else {
        panic!("expected two keyframe blocks");
    };
    assert_eq!(from.selectors().selectors(), &[CssKeyframeSelector::From]);
    assert_eq!(from.declarations()[0].property(), &CssProperty::Opacity);
    assert_eq!(to.selectors().selectors(), &[CssKeyframeSelector::To]);
    assert_eq!(to.declarations()[0].property(), &CssProperty::Opacity);

    assert_eq!(
        style_rule(base).selector(),
        &CssSelector::Class("card".to_owned())
    );
    assert_eq!(
        style_rule(base).declarations()[0].property(),
        &CssProperty::Color
    );

    let CssSelector::Compound(hover_selector) = style_rule(hover).selector() else {
        panic!("expected flattened hover selector");
    };
    assert_eq!(hover_selector.classes(), &["card".to_owned()]);
    assert_eq!(hover_selector.pseudo_classes(), &[CssPseudoClass::Hover]);

    let CssRule::Media(media) = media else {
        panic!("expected media rule");
    };
    let [nested] = media.rules() else {
        panic!("expected one nested flattened rule");
    };
    let CssSelector::Complex(selector) = style_rule(nested).selector() else {
        panic!("expected complex nested title selector");
    };
    assert_eq!(selector.first().classes(), &["card".to_owned()]);
    let [part] = selector.rest() else {
        panic!("expected one child selector part");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(part.selector().classes(), &["title".to_owned()]);
    assert_eq!(
        style_rule(nested).declarations()[0].property(),
        &CssProperty::Color
    );
}

#[test]
fn color_model_preserves_rgba_and_currentcolor() {
    let rgba = CssRgbaColor::try_new(255, 128, 0, 0.5).unwrap();
    assert_eq!(rgba.red(), 255);
    assert_eq!(rgba.green(), 128);
    assert_eq!(rgba.blue(), 0);
    assert_eq!(rgba.alpha(), 0.5);
    assert_eq!(CssColor::BLACK.as_rgba().unwrap().red(), 0);
}

#[test]
fn color_model_rejects_invalid_rgba_alpha() {
    assert_eq!(CssRgbaColor::try_new(0, 0, 0, -0.1), None);
    assert_eq!(CssRgbaColor::try_new(0, 0, 0, 1.1), None);
    assert_eq!(CssRgbaColor::try_new(0, 0, 0, f32::NAN), None);
}

#[test]
fn color_function_model_preserves_color_space_and_components() {
    let color = CssColorFunction::try_new(
        CssPredefinedColorSpace::DisplayP3,
        [Some(0.8), Some(0.2), Some(0.1)],
        Some(0.9),
    )
    .unwrap();

    assert_eq!(color.color_space(), CssPredefinedColorSpace::DisplayP3);
    assert_eq!(color.components(), &[Some(0.8), Some(0.2), Some(0.1)]);
    assert_eq!(color.alpha(), Some(0.9));
}

#[test]
fn color_model_rejects_non_finite_components_and_invalid_alpha() {
    assert_eq!(
        CssHslColor::try_new(Some(f32::NAN), Some(1.0), Some(0.5), Some(1.0)),
        None
    );
    assert_eq!(
        CssHwbColor::try_new(Some(30.0), Some(f32::INFINITY), Some(0.2), Some(1.0)),
        None
    );
    assert_eq!(
        CssLabColor::try_new(Some(0.5), Some(0.1), Some(f32::NEG_INFINITY), Some(1.0)),
        None
    );
    assert_eq!(
        CssLchColor::try_new(Some(0.5), Some(0.1), Some(30.0), Some(1.1)),
        None
    );
    assert_eq!(
        CssColorFunction::try_new(
            CssPredefinedColorSpace::DisplayP3,
            [Some(0.8), Some(f32::NAN), Some(0.1)],
            Some(0.9),
        ),
        None
    );
    assert_eq!(
        CssColorFunction::try_new(
            CssPredefinedColorSpace::DisplayP3,
            [Some(0.8), Some(0.2), Some(0.1)],
            Some(-0.1),
        ),
        None
    );
}

#[test]
fn symbolic_color_model_rejects_invalid_percentages_and_component_counts() {
    assert_eq!(
        CssColorMixComponent::try_new(CssColor::BLACK, Some(-0.1)),
        None
    );
    assert_eq!(
        CssColorMixComponent::try_new(CssColor::BLACK, Some(100.1)),
        None
    );
    assert_eq!(
        CssColorMixComponent::try_new(CssColor::BLACK, Some(f32::NAN)),
        None
    );

    let component = || {
        CssColorComponentExpression::new(
            CssAuthoredDeclarationValue::try_new("r").unwrap(),
            Vec::new(),
        )
    };

    assert_eq!(
        CssRelativeColor::try_new(
            CssRelativeColorFunction::Rgb,
            CssColor::BLACK,
            vec![component(), component()],
            None,
        ),
        None
    );
    assert_eq!(
        CssRelativeColor::try_new(
            CssRelativeColorFunction::Color(CssPredefinedColorSpace::Srgb),
            CssColor::BLACK,
            vec![component(), component(), component(), component()],
            None,
        ),
        None
    );
}

#[test]
fn rejection_unsupported_but_syntactically_valid_css_keywords_stay_rejected() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "display inline remains unsupported",
            property_name: "display",
            authored_value: "inline",
            expected_error: ExpectedErrorKind::UnsupportedValue {
                property: Some("display"),
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "overflow auto remains unsupported",
            property_name: "overflow",
            authored_value: "auto",
            expected_error: ExpectedErrorKind::UnsupportedValue {
                property: Some("overflow"),
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "text-align-last match-parent remains unsupported",
            property_name: "text-align-last",
            authored_value: "match-parent",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "text-align-last",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "background-origin margin-box remains unsupported",
            property_name: "background-origin",
            authored_value: "margin-box",
            expected_error: ExpectedErrorKind::UnsupportedValueForProperty {
                property: "background-origin",
            },
            property_name_should_be_recognized: true,
        },
    ]);
}

#[test]
fn acceptance_css_wide_global_keyword_matrix_accepts_supported_globals() {
    let cases = [
        AcceptedDeclarationCase {
            label: "all inherit",
            property_name: "all",
            authored_value: "inherit",
            expected_property: CssProperty::All,
            expected_global: CssGlobalKeyword::Inherit,
        },
        AcceptedDeclarationCase {
            label: "all initial",
            property_name: "all",
            authored_value: "initial",
            expected_property: CssProperty::All,
            expected_global: CssGlobalKeyword::Initial,
        },
        AcceptedDeclarationCase {
            label: "all unset",
            property_name: "all",
            authored_value: "unset",
            expected_property: CssProperty::All,
            expected_global: CssGlobalKeyword::Unset,
        },
        AcceptedDeclarationCase {
            label: "all revert",
            property_name: "all",
            authored_value: "revert",
            expected_property: CssProperty::All,
            expected_global: CssGlobalKeyword::Revert,
        },
        AcceptedDeclarationCase {
            label: "all revert-layer",
            property_name: "all",
            authored_value: "revert-layer",
            expected_property: CssProperty::All,
            expected_global: CssGlobalKeyword::RevertLayer,
        },
        AcceptedDeclarationCase {
            label: "display global initial",
            property_name: "display",
            authored_value: "initial",
            expected_property: CssProperty::Display,
            expected_global: CssGlobalKeyword::Initial,
        },
        AcceptedDeclarationCase {
            label: "width global unset",
            property_name: "width",
            authored_value: "unset",
            expected_property: CssProperty::Width,
            expected_global: CssGlobalKeyword::Unset,
        },
        AcceptedDeclarationCase {
            label: "color global revert",
            property_name: "color",
            authored_value: "revert",
            expected_property: CssProperty::Color,
            expected_global: CssGlobalKeyword::Revert,
        },
        AcceptedDeclarationCase {
            label: "animation global revert-layer",
            property_name: "animation",
            authored_value: "revert-layer",
            expected_property: CssProperty::Animation,
            expected_global: CssGlobalKeyword::RevertLayer,
        },
        AcceptedDeclarationCase {
            label: "background-color global inherit preserves authored property",
            property_name: "background-color",
            authored_value: "inherit",
            expected_property: CssProperty::BackgroundColor,
            expected_global: CssGlobalKeyword::Inherit,
        },
        AcceptedDeclarationCase {
            label: "mask global unset",
            property_name: "mask",
            authored_value: "unset",
            expected_property: CssProperty::Mask,
            expected_global: CssGlobalKeyword::Unset,
        },
    ];

    assert_accepts_declarations(&cases);
}

#[test]
fn acceptance_box_layout_and_spacing_family_matrix_accepts_supported_values() {
    let cases = [
        value_case!(
            "display block",
            "display",
            "block",
            CssProperty::Display,
            assert_display_value
        ),
        value_case!(
            "display inline-grid-lanes",
            "display",
            "inline-grid-lanes",
            CssProperty::Display,
            assert_display_value
        ),
        value_case!(
            "box-sizing border-box",
            "box-sizing",
            "border-box",
            CssProperty::BoxSizing,
            assert_box_sizing_value
        ),
        value_case!(
            "position sticky",
            "position",
            "sticky",
            CssProperty::Position,
            assert_position_value
        ),
        value_case!(
            "direction rtl",
            "direction",
            "rtl",
            CssProperty::Direction,
            assert_direction_value
        ),
        value_case!(
            "overflow axes",
            "overflow",
            "hidden scroll",
            CssProperty::Overflow,
            assert_overflow_value
        ),
        value_case!(
            "overflow-x clip",
            "overflow-x",
            "clip",
            CssProperty::OverflowX,
            assert_overflow_value
        ),
        value_case!(
            "overflow-y visible",
            "overflow-y",
            "visible",
            CssProperty::OverflowY,
            assert_overflow_value
        ),
        value_case!(
            "flex-direction column-reverse",
            "flex-direction",
            "column-reverse",
            CssProperty::FlexDirection,
            assert_flex_direction_value
        ),
        value_case!(
            "flex-wrap wrap-reverse",
            "flex-wrap",
            "wrap-reverse",
            CssProperty::FlexWrap,
            assert_flex_wrap_value
        ),
        value_case!(
            "width calc",
            "width",
            "calc(100% - 12px)",
            CssProperty::Width,
            assert_length_value
        ),
        value_case!(
            "height auto",
            "height",
            "auto",
            CssProperty::Height,
            assert_length_value
        ),
        value_case!(
            "min-width zero",
            "min-width",
            "0",
            CssProperty::MinWidth,
            assert_length_value
        ),
        value_case!(
            "min-height min-content",
            "min-height",
            "min-content",
            CssProperty::MinHeight,
            assert_length_value
        ),
        value_case!(
            "max-width max-content",
            "max-width",
            "max-content",
            CssProperty::MaxWidth,
            assert_length_value
        ),
        value_case!(
            "max-height fit-content",
            "max-height",
            "fit-content",
            CssProperty::MaxHeight,
            assert_length_value
        ),
        value_case!(
            "flex-basis rem",
            "flex-basis",
            "10rem",
            CssProperty::FlexBasis,
            assert_length_value
        ),
        value_case!(
            "gap two lengths",
            "gap",
            "12px",
            CssProperty::Gap,
            assert_length_value
        ),
        value_case!(
            "row-gap normal",
            "row-gap",
            "normal",
            CssProperty::RowGap,
            assert_length_value
        ),
        value_case!(
            "column-gap percent",
            "column-gap",
            "5%",
            CssProperty::ColumnGap,
            assert_length_value
        ),
        value_case!(
            "inset shorthand",
            "inset",
            "auto 10px 5%",
            CssProperty::Inset,
            assert_edges_value
        ),
        value_case!(
            "top auto",
            "top",
            "auto",
            CssProperty::Top,
            assert_length_value
        ),
        value_case!(
            "right length",
            "right",
            "10px",
            CssProperty::Right,
            assert_length_value
        ),
        value_case!(
            "bottom percent",
            "bottom",
            "5%",
            CssProperty::Bottom,
            assert_length_value
        ),
        value_case!(
            "left calc",
            "left",
            "calc(3px + 4%)",
            CssProperty::Left,
            assert_length_value
        ),
        value_case!(
            "z-index integer",
            "z-index",
            "-2",
            CssProperty::ZIndex,
            assert_z_index_value
        ),
        value_case!(
            "box-decoration-break clone",
            "box-decoration-break",
            "clone",
            CssProperty::BoxDecorationBreak,
            assert_box_decoration_break_value
        ),
        value_case!(
            "margin shorthand",
            "margin",
            "auto 10px 5%",
            CssProperty::Margin,
            assert_edges_value
        ),
        value_case!(
            "margin-top auto",
            "margin-top",
            "auto",
            CssProperty::MarginTop,
            assert_length_value
        ),
        value_case!(
            "margin-right length",
            "margin-right",
            "10px",
            CssProperty::MarginRight,
            assert_length_value
        ),
        value_case!(
            "margin-bottom percent",
            "margin-bottom",
            "5%",
            CssProperty::MarginBottom,
            assert_length_value
        ),
        value_case!(
            "margin-left calc",
            "margin-left",
            "calc(3px + 4%)",
            CssProperty::MarginLeft,
            assert_length_value
        ),
        value_case!(
            "padding shorthand",
            "padding",
            "1px 2% calc(3px + 4%) 0",
            CssProperty::Padding,
            assert_edges_value
        ),
        value_case!(
            "padding-top length",
            "padding-top",
            "12px",
            CssProperty::PaddingTop,
            assert_length_value
        ),
        value_case!(
            "padding-right percent",
            "padding-right",
            "2%",
            CssProperty::PaddingRight,
            assert_length_value
        ),
        value_case!(
            "padding-bottom calc",
            "padding-bottom",
            "calc(3px + 4%)",
            CssProperty::PaddingBottom,
            assert_length_value
        ),
        value_case!(
            "padding-left zero",
            "padding-left",
            "0",
            CssProperty::PaddingLeft,
            assert_length_value
        ),
        value_case!(
            "border-width shorthand",
            "border-width",
            "1px 2px 3px 4px",
            CssProperty::BorderWidth,
            assert_edges_value
        ),
        value_case!(
            "border-top-width length",
            "border-top-width",
            "1px",
            CssProperty::BorderTopWidth,
            assert_length_value
        ),
        value_case!(
            "border-right-width length",
            "border-right-width",
            "2px",
            CssProperty::BorderRightWidth,
            assert_length_value
        ),
        value_case!(
            "border-bottom-width length",
            "border-bottom-width",
            "3px",
            CssProperty::BorderBottomWidth,
            assert_length_value
        ),
        value_case!(
            "border-left-width length",
            "border-left-width",
            "4px",
            CssProperty::BorderLeftWidth,
            assert_length_value
        ),
        value_case!(
            "border-radius shorthand",
            "border-radius",
            "1px 2px 3px / 4px 5px",
            CssProperty::BorderRadius,
            assert_border_radius_value
        ),
        value_case!(
            "border-top-left-radius pair",
            "border-top-left-radius",
            "4px 10%",
            CssProperty::BorderTopLeftRadius,
            assert_corner_radius_value
        ),
        value_case!(
            "border-top-right-radius length",
            "border-top-right-radius",
            "1px",
            CssProperty::BorderTopRightRadius,
            assert_corner_radius_value
        ),
        value_case!(
            "border-bottom-right-radius percent",
            "border-bottom-right-radius",
            "10%",
            CssProperty::BorderBottomRightRadius,
            assert_corner_radius_value
        ),
        value_case!(
            "border-bottom-left-radius calc",
            "border-bottom-left-radius",
            "calc(1px + 2%)",
            CssProperty::BorderBottomLeftRadius,
            assert_corner_radius_value
        ),
    ];

    assert_accepts_value_cases(&cases);
}

#[test]
fn acceptance_color_background_border_outline_and_shadow_matrix_accepts_supported_values() {
    let cases = [
        value_case!(
            "color named",
            "color",
            "black",
            CssProperty::Color,
            assert_color_value
        ),
        value_case!(
            "background color",
            "background",
            "#fff",
            CssProperty::Background,
            assert_color_value
        ),
        value_case!(
            "background-color authored property",
            "background-color",
            "transparent",
            CssProperty::BackgroundColor,
            assert_color_value
        ),
        value_case!(
            "border shorthand",
            "border",
            "solid 2px #fff",
            CssProperty::Border,
            assert_border_value
        ),
        value_case!(
            "border-top shorthand",
            "border-top",
            "black dotted",
            CssProperty::BorderTop,
            assert_border_value
        ),
        value_case!(
            "border-right width-only",
            "border-right",
            "1px",
            CssProperty::BorderRight,
            assert_border_value
        ),
        value_case!(
            "border-bottom color-only",
            "border-bottom",
            "#fff",
            CssProperty::BorderBottom,
            assert_border_value
        ),
        value_case!(
            "border-left style-color",
            "border-left",
            "dashed black",
            CssProperty::BorderLeft,
            assert_border_value
        ),
        value_case!(
            "border-color named",
            "border-color",
            "black",
            CssProperty::BorderColor,
            assert_color_value
        ),
        value_case!(
            "border-top-color named",
            "border-top-color",
            "black",
            CssProperty::BorderTopColor,
            assert_color_value
        ),
        value_case!(
            "border-right-color named",
            "border-right-color",
            "white",
            CssProperty::BorderRightColor,
            assert_color_value
        ),
        value_case!(
            "border-bottom-color transparent",
            "border-bottom-color",
            "transparent",
            CssProperty::BorderBottomColor,
            assert_color_value
        ),
        value_case!(
            "border-left-color hex",
            "border-left-color",
            "#fff",
            CssProperty::BorderLeftColor,
            assert_color_value
        ),
        value_case!(
            "border-style shorthand",
            "border-style",
            "none hidden dotted dashed",
            CssProperty::BorderStyle,
            assert_border_styles_value
        ),
        value_case!(
            "border-top-style solid",
            "border-top-style",
            "solid",
            CssProperty::BorderTopStyle,
            assert_border_style_value
        ),
        value_case!(
            "border-right-style double",
            "border-right-style",
            "double",
            CssProperty::BorderRightStyle,
            assert_border_style_value
        ),
        value_case!(
            "border-bottom-style ridge",
            "border-bottom-style",
            "ridge",
            CssProperty::BorderBottomStyle,
            assert_border_style_value
        ),
        value_case!(
            "border-left-style outset",
            "border-left-style",
            "outset",
            CssProperty::BorderLeftStyle,
            assert_border_style_value
        ),
        value_case!(
            "background-image list",
            "background-image",
            "url(\"hero.png\"), none",
            CssProperty::BackgroundImage,
            assert_background_image_value
        ),
        value_case!(
            "background-position offset",
            "background-position",
            "left 10px top 20%",
            CssProperty::BackgroundPosition,
            assert_background_position_value
        ),
        value_case!(
            "background-size list",
            "background-size",
            "cover, 10px auto",
            CssProperty::BackgroundSize,
            assert_background_size_value
        ),
        value_case!(
            "background-repeat list",
            "background-repeat",
            "repeat-x, no-repeat round",
            CssProperty::BackgroundRepeat,
            assert_background_repeat_value
        ),
        value_case!(
            "background-origin box",
            "background-origin",
            "content-box",
            CssProperty::BackgroundOrigin,
            assert_background_box_value
        ),
        value_case!(
            "background-clip box",
            "background-clip",
            "padding-box",
            CssProperty::BackgroundClip,
            assert_background_box_value
        ),
        value_case!(
            "background-attachment list",
            "background-attachment",
            "fixed, local",
            CssProperty::BackgroundAttachment,
            assert_background_attachment_value
        ),
        value_case!(
            "outline shorthand",
            "outline",
            "thick dotted white",
            CssProperty::Outline,
            assert_outline_value
        ),
        value_case!(
            "outline-color",
            "outline-color",
            "black",
            CssProperty::OutlineColor,
            assert_outline_color_value
        ),
        value_case!(
            "outline-style auto",
            "outline-style",
            "auto",
            CssProperty::OutlineStyle,
            assert_outline_style_value
        ),
        value_case!(
            "outline-width length",
            "outline-width",
            "2px",
            CssProperty::OutlineWidth,
            assert_outline_width_value
        ),
        value_case!(
            "box-shadow none",
            "box-shadow",
            "none",
            CssProperty::BoxShadow,
            assert_box_shadow_value
        ),
        value_case!(
            "box-shadow list",
            "box-shadow",
            "inset 1px 2px 3px 4px black, 0 1px #fff",
            CssProperty::BoxShadow,
            assert_box_shadow_value
        ),
        value_case!(
            "opacity number",
            "opacity",
            "0.5",
            CssProperty::Opacity,
            assert_opacity_value
        ),
    ];

    assert_accepts_value_cases(&cases);
}

#[test]
fn acceptance_position_alignment_flex_and_grid_matrix_accepts_supported_values() {
    let cases = [
        value_case!(
            "float left",
            "float",
            "left",
            CssProperty::Float,
            assert_float_value
        ),
        value_case!(
            "clear both",
            "clear",
            "both",
            CssProperty::Clear,
            assert_clear_value
        ),
        value_case!(
            "align-content distribution",
            "align-content",
            "space-between",
            CssProperty::AlignContent,
            assert_alignment_value
        ),
        value_case!(
            "justify-content safe center",
            "justify-content",
            "safe center",
            CssProperty::JustifyContent,
            assert_alignment_value
        ),
        value_case!(
            "align-items baseline",
            "align-items",
            "first baseline",
            CssProperty::AlignItems,
            assert_align_items_value
        ),
        value_case!(
            "align-self safe flex-end",
            "align-self",
            "safe flex-end",
            CssProperty::AlignSelf,
            assert_align_items_value
        ),
        value_case!(
            "justify-items stretch",
            "justify-items",
            "stretch",
            CssProperty::JustifyItems,
            assert_align_items_value
        ),
        value_case!(
            "justify-self center",
            "justify-self",
            "center",
            CssProperty::JustifySelf,
            assert_align_items_value
        ),
        value_case!(
            "place-content pair",
            "place-content",
            "center end",
            CssProperty::PlaceContent,
            assert_place_alignment_value
        ),
        value_case!(
            "place-items single",
            "place-items",
            "stretch",
            CssProperty::PlaceItems,
            assert_place_alignment_value
        ),
        value_case!(
            "place-self pair",
            "place-self",
            "end center",
            CssProperty::PlaceSelf,
            assert_place_alignment_value
        ),
        value_case!(
            "visibility collapse",
            "visibility",
            "collapse",
            CssProperty::Visibility,
            assert_visibility_value
        ),
        value_case!(
            "content-visibility auto",
            "content-visibility",
            "auto",
            CssProperty::ContentVisibility,
            assert_content_visibility_value
        ),
        value_case!(
            "flex-grow number",
            "flex-grow",
            "2",
            CssProperty::FlexGrow,
            assert_flex_grow_value
        ),
        value_case!(
            "flex-shrink number",
            "flex-shrink",
            "0",
            CssProperty::FlexShrink,
            assert_flex_shrink_value
        ),
        value_case!(
            "order negative integer",
            "order",
            "-2",
            CssProperty::Order,
            assert_order_value
        ),
        value_case!(
            "flex components",
            "flex",
            "2 0 10rem",
            CssProperty::Flex,
            assert_flex_value
        ),
        value_case!(
            "flex keyword none",
            "flex",
            "none",
            CssProperty::Flex,
            assert_flex_value
        ),
        value_case!(
            "justify-tracks distribution",
            "justify-tracks",
            "space-evenly",
            CssProperty::JustifyTracks,
            assert_alignment_value
        ),
        value_case!(
            "align-tracks center",
            "align-tracks",
            "center",
            CssProperty::AlignTracks,
            assert_alignment_value
        ),
        value_case!(
            "aspect-ratio number",
            "aspect-ratio",
            "1.5",
            CssProperty::AspectRatio,
            assert_aspect_ratio_value
        ),
        value_case!(
            "scrollbar-width keyword",
            "scrollbar-width",
            "thin",
            CssProperty::ScrollbarWidth,
            assert_scrollbar_width_value
        ),
        value_case!(
            "grid-flow-tolerance infinite",
            "grid-flow-tolerance",
            "infinite",
            CssProperty::GridFlowTolerance,
            assert_grid_flow_tolerance_value
        ),
        value_case!(
            "grid-template-rows tracks",
            "grid-template-rows",
            "[top] 100px 1fr",
            CssProperty::GridTemplateRows,
            assert_grid_track_list_value
        ),
        value_case!(
            "grid-template-columns repeat",
            "grid-template-columns",
            "repeat(2, minmax(10px, 1fr))",
            CssProperty::GridTemplateColumns,
            assert_grid_track_list_value
        ),
        value_case!(
            "grid-template-areas rows",
            "grid-template-areas",
            "\"header header\" \"nav main\"",
            CssProperty::GridTemplateAreas,
            assert_grid_template_areas_value
        ),
        value_case!(
            "grid-template shorthand",
            "grid-template",
            "100px 1fr / repeat(2, minmax(10px, 1fr))",
            CssProperty::GridTemplate,
            assert_grid_template_value
        ),
        value_case!(
            "grid-auto-rows minmax",
            "grid-auto-rows",
            "minmax(10px, auto)",
            CssProperty::GridAutoRows,
            assert_grid_track_list_value
        ),
        value_case!(
            "grid-auto-columns fit-content",
            "grid-auto-columns",
            "fit-content(20%)",
            CssProperty::GridAutoColumns,
            assert_grid_track_list_value
        ),
        value_case!(
            "grid-auto-flow dense",
            "grid-auto-flow",
            "column dense",
            CssProperty::GridAutoFlow,
            assert_grid_auto_flow_value
        ),
        value_case!(
            "grid-row-start span",
            "grid-row-start",
            "span 2 main",
            CssProperty::GridRowStart,
            assert_grid_line_value
        ),
        value_case!(
            "grid-row-end auto",
            "grid-row-end",
            "auto",
            CssProperty::GridRowEnd,
            assert_grid_line_value
        ),
        value_case!(
            "grid-column-start ident",
            "grid-column-start",
            "nav",
            CssProperty::GridColumnStart,
            assert_grid_line_value
        ),
        value_case!(
            "grid-column-end integer",
            "grid-column-end",
            "4",
            CssProperty::GridColumnEnd,
            assert_grid_line_value
        ),
        value_case!(
            "grid-row range",
            "grid-row",
            "1 / span 2",
            CssProperty::GridRow,
            assert_grid_line_range_value
        ),
        value_case!(
            "grid-column range",
            "grid-column",
            "nav / main",
            CssProperty::GridColumn,
            assert_grid_line_range_value
        ),
        value_case!(
            "grid-area shorthand",
            "grid-area",
            "header / 1 / span 2 / main",
            CssProperty::GridArea,
            assert_grid_area_value
        ),
        value_case!(
            "grid auto-flow shorthand",
            "grid",
            "auto-flow dense 12px / repeat(auto-fit, 10px)",
            CssProperty::Grid,
            assert_grid_value
        ),
    ];

    assert_accepts_value_cases(&cases);
}

#[test]
fn acceptance_typography_and_text_family_matrix_accepts_supported_values() {
    let cases = [
        value_case!(
            "font-size length",
            "font-size",
            "16px",
            CssProperty::FontSize,
            assert_length_value
        ),
        value_case!(
            "line-height normal",
            "line-height",
            "normal",
            CssProperty::LineHeight,
            assert_length_value
        ),
        value_case!(
            "writing-mode vertical",
            "writing-mode",
            "vertical-rl",
            CssProperty::WritingMode,
            assert_writing_mode_value
        ),
        value_case!(
            "text-align start",
            "text-align",
            "start",
            CssProperty::TextAlign,
            assert_text_align_value
        ),
        value_case!(
            "text-align-last justify",
            "text-align-last",
            "justify",
            CssProperty::TextAlignLast,
            assert_text_align_last_value
        ),
        value_case!(
            "text-indent flags",
            "text-indent",
            "1rem hanging each-line",
            CssProperty::TextIndent,
            assert_text_indent_value
        ),
        value_case!(
            "vertical-align keyword",
            "vertical-align",
            "super",
            CssProperty::VerticalAlign,
            assert_vertical_align_value
        ),
        value_case!(
            "vertical-align length",
            "vertical-align",
            "4px",
            CssProperty::VerticalAlign,
            assert_vertical_align_value
        ),
        value_case!(
            "font-family list",
            "font-family",
            "\"Avenir Next\", Gill Sans, sans-serif",
            CssProperty::FontFamily,
            assert_font_family_value
        ),
        value_case!(
            "font shorthand",
            "font",
            "italic small-caps 700 condensed 16px/normal \"Avenir Next\", sans-serif",
            CssProperty::Font,
            assert_font_value
        ),
        value_case!(
            "font-weight number",
            "font-weight",
            "725",
            CssProperty::FontWeight,
            assert_font_weight_value
        ),
        value_case!(
            "font-style italic",
            "font-style",
            "italic",
            CssProperty::FontStyle,
            assert_font_style_value
        ),
        value_case!(
            "font-stretch semi-condensed",
            "font-stretch",
            "semi-condensed",
            CssProperty::FontStretch,
            assert_font_stretch_value
        ),
        value_case!(
            "font-variant small-caps",
            "font-variant",
            "small-caps",
            CssProperty::FontVariant,
            assert_font_variant_value
        ),
        value_case!(
            "font-feature-settings list",
            "font-feature-settings",
            "\"kern\" on, \"liga\" 0",
            CssProperty::FontFeatureSettings,
            assert_font_feature_settings_value
        ),
        value_case!(
            "letter-spacing normal",
            "letter-spacing",
            "normal",
            CssProperty::LetterSpacing,
            assert_letter_spacing_value
        ),
        value_case!(
            "letter-spacing length",
            "letter-spacing",
            "0.1em",
            CssProperty::LetterSpacing,
            assert_letter_spacing_value
        ),
        value_case!(
            "text-wrap balance",
            "text-wrap",
            "balance",
            CssProperty::TextWrap,
            assert_text_wrap_value
        ),
        value_case!(
            "white-space pre-wrap",
            "white-space",
            "pre-wrap",
            CssProperty::WhiteSpace,
            assert_white_space_value
        ),
        value_case!(
            "word-break keep-all",
            "word-break",
            "keep-all",
            CssProperty::WordBreak,
            assert_word_break_value
        ),
        value_case!(
            "overflow-wrap anywhere",
            "overflow-wrap",
            "anywhere",
            CssProperty::OverflowWrap,
            assert_overflow_wrap_value
        ),
        value_case!(
            "text-overflow ellipsis",
            "text-overflow",
            "ellipsis",
            CssProperty::TextOverflow,
            assert_text_overflow_value
        ),
        value_case!(
            "text-decoration shorthand",
            "text-decoration",
            "underline dotted white 3px",
            CssProperty::TextDecoration,
            assert_text_decoration_value
        ),
        value_case!(
            "text-decoration-line list",
            "text-decoration-line",
            "underline overline",
            CssProperty::TextDecorationLine,
            assert_text_decoration_line_value
        ),
        value_case!(
            "text-decoration-color",
            "text-decoration-color",
            "black",
            CssProperty::TextDecorationColor,
            assert_text_decoration_color_value
        ),
        value_case!(
            "text-decoration-style",
            "text-decoration-style",
            "wavy",
            CssProperty::TextDecorationStyle,
            assert_text_decoration_style_value
        ),
        value_case!(
            "text-decoration-thickness length",
            "text-decoration-thickness",
            "2px",
            CssProperty::TextDecorationThickness,
            assert_text_decoration_thickness_value
        ),
        value_case!(
            "text-transform uppercase",
            "text-transform",
            "uppercase",
            CssProperty::TextTransform,
            assert_text_transform_value
        ),
    ];

    assert_accepts_value_cases(&cases);
}

#[test]
fn acceptance_interaction_effect_mask_transition_animation_matrix_accepts_supported_values() {
    let cases = [
        value_case!(
            "cursor keyword",
            "cursor",
            "grab",
            CssProperty::Cursor,
            assert_cursor_value
        ),
        value_case!(
            "cursor url fallback",
            "cursor",
            "url(cursor.png), pointer",
            CssProperty::Cursor,
            assert_cursor_value
        ),
        value_case!(
            "pointer-events none",
            "pointer-events",
            "none",
            CssProperty::PointerEvents,
            assert_pointer_events_value
        ),
        value_case!(
            "user-select text",
            "user-select",
            "text",
            CssProperty::UserSelect,
            assert_user_select_value
        ),
        value_case!(
            "transform functions",
            "transform",
            "translate(10px, 20px) rotate(45deg) scale(1.5)",
            CssProperty::Transform,
            assert_transform_value
        ),
        value_case!(
            "transform none",
            "transform",
            "none",
            CssProperty::Transform,
            assert_transform_value
        ),
        value_case!(
            "transform-origin position",
            "transform-origin",
            "center top",
            CssProperty::TransformOrigin,
            assert_transform_origin_value
        ),
        value_case!(
            "translate values",
            "translate",
            "10px 20px",
            CssProperty::Translate,
            assert_translate_value
        ),
        value_case!(
            "rotate angle",
            "rotate",
            "45deg",
            CssProperty::Rotate,
            assert_rotate_value
        ),
        value_case!(
            "scale values",
            "scale",
            "1.5 2",
            CssProperty::Scale,
            assert_scale_value
        ),
        value_case!(
            "filter functions",
            "filter",
            "blur(4px) opacity(50%)",
            CssProperty::Filter,
            assert_filter_value
        ),
        value_case!(
            "backdrop-filter none",
            "backdrop-filter",
            "none",
            CssProperty::BackdropFilter,
            assert_filter_value
        ),
        value_case!(
            "clip-path shape",
            "clip-path",
            "circle(50% at center)",
            CssProperty::ClipPath,
            assert_clip_path_value
        ),
        value_case!(
            "mask shorthand",
            "mask",
            "url(mask.png) center / contain no-repeat",
            CssProperty::Mask,
            assert_mask_value
        ),
        value_case!(
            "mask-image list",
            "mask-image",
            "url(mask.png), none",
            CssProperty::MaskImage,
            assert_mask_image_value
        ),
        value_case!(
            "mask-size contain",
            "mask-size",
            "contain",
            CssProperty::MaskSize,
            assert_mask_size_value
        ),
        value_case!(
            "mask-position center",
            "mask-position",
            "center",
            CssProperty::MaskPosition,
            assert_mask_position_value
        ),
        value_case!(
            "mask-repeat repeat",
            "mask-repeat",
            "repeat",
            CssProperty::MaskRepeat,
            assert_mask_repeat_value
        ),
        value_case!(
            "transition-property list",
            "transition-property",
            "opacity, transform",
            CssProperty::TransitionProperty,
            assert_transition_property_value
        ),
        value_case!(
            "transition-duration list",
            "transition-duration",
            "150ms, 2s",
            CssProperty::TransitionDuration,
            assert_time_list_value
        ),
        value_case!(
            "transition-delay time",
            "transition-delay",
            "20ms",
            CssProperty::TransitionDelay,
            assert_time_list_value
        ),
        value_case!(
            "transition-timing-function list",
            "transition-timing-function",
            "ease-in, cubic-bezier(0.1, 0.2, 0.3, 1)",
            CssProperty::TransitionTimingFunction,
            assert_easing_list_value
        ),
        value_case!(
            "transition shorthand list",
            "transition",
            "opacity 150ms ease-in 20ms, transform 2s linear",
            CssProperty::Transition,
            assert_transition_value
        ),
        value_case!(
            "animation-name list",
            "animation-name",
            "fade, none",
            CssProperty::AnimationName,
            assert_animation_name_value
        ),
        value_case!(
            "animation-duration time",
            "animation-duration",
            "1s",
            CssProperty::AnimationDuration,
            assert_time_list_value
        ),
        value_case!(
            "animation-delay time",
            "animation-delay",
            "200ms",
            CssProperty::AnimationDelay,
            assert_time_list_value
        ),
        value_case!(
            "animation-timing-function easing",
            "animation-timing-function",
            "ease-out",
            CssProperty::AnimationTimingFunction,
            assert_easing_list_value
        ),
        value_case!(
            "animation-iteration-count list",
            "animation-iteration-count",
            "2, infinite",
            CssProperty::AnimationIterationCount,
            assert_animation_iteration_count_value
        ),
        value_case!(
            "animation-direction",
            "animation-direction",
            "alternate",
            CssProperty::AnimationDirection,
            assert_animation_direction_value
        ),
        value_case!(
            "animation-fill-mode",
            "animation-fill-mode",
            "both",
            CssProperty::AnimationFillMode,
            assert_animation_fill_mode_value
        ),
        value_case!(
            "animation-play-state list",
            "animation-play-state",
            "running, paused",
            CssProperty::AnimationPlayState,
            assert_animation_play_state_value
        ),
        value_case!(
            "animation shorthand list",
            "animation",
            "fade 1s ease-in 200ms 3 alternate both running, slide 2s linear",
            CssProperty::Animation,
            assert_animation_value
        ),
    ];

    assert_accepts_value_cases(&cases);
}

#[test]
fn parses_calc_width_as_css_calc_length() {
    let value = declaration_value!(".panel { width: calc(20px + 10%); }", Width);

    match value {
        CssLength::Calc(calc) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(20px + 10%)");
        }
        other => panic!("expected calc length, got {other:?}"),
    }
}

#[test]
fn parses_nested_calc_width_with_subtraction_as_css_syntax() {
    let value = declaration_value!(".panel { width: calc(100% - calc(12px + 3%)); }", Width);

    match value {
        CssLength::Calc(calc) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(100% - calc(12px + 3%))");
        }
        other => panic!("expected nested calc length, got {other:?}"),
    }
}

#[test]
fn exposes_nested_calc_terms_structurally() {
    let value = declaration_value!(".panel { width: calc(100% - calc(12px + 3%)); }", Width);

    let calc = match value {
        CssLength::Calc(calc) => calc,
        other => panic!("expected nested calc length, got {other:?}"),
    };

    let terms = match calc {
        CssCalcLength::Sum(terms) => terms,
        other => panic!("expected calc sum, got {other:?}"),
    };
    assert_eq!(terms.len(), 2);
    assert_eq!(terms[0].operator(), CssCalcOperator::Add);
    assert_eq!(terms[0].value(), &CssCalcLength::percent(100.0));
    assert_eq!(terms[1].operator(), CssCalcOperator::Subtract);

    let nested_terms = match terms[1].value() {
        CssCalcLength::Sum(terms) => terms,
        other => panic!("expected nested calc sum, got {other:?}"),
    };
    assert_eq!(nested_terms.len(), 2);
    assert_eq!(nested_terms[0].operator(), CssCalcOperator::Add);
    assert_eq!(nested_terms[0].value(), &CssCalcLength::px(12.0));
    assert_eq!(nested_terms[1].operator(), CssCalcOperator::Add);
    assert_eq!(nested_terms[1].value(), &CssCalcLength::percent(3.0));
}

#[test]
fn successful_declarations_expose_authored_source_position() {
    let input = ".panel {\n  height: 20px;\n  width: calc(100% - 4px);\n}\n";
    let height = declaration(input, CssProperty::Height);
    let width = declaration(input, CssProperty::Width);

    assert_eq!(height.position().byte_offset().value(), 11);
    assert_eq!(height.position().line().value(), 1);
    assert_eq!(height.position().column().value(), 2);
    assert_eq!(width.position().byte_offset().value(), 27);
    assert_eq!(width.position().line().value(), 2);
    assert_eq!(width.position().column().value(), 2);
}

#[test]
fn parses_supported_length_units_as_authored_dimensions() {
    let cases = [
        ("1em", 1.0, CssLengthUnit::Em),
        ("2rem", 2.0, CssLengthUnit::Rem),
        ("3vw", 3.0, CssLengthUnit::Vw),
        ("4svh", 4.0, CssLengthUnit::Svh),
        ("5lvw", 5.0, CssLengthUnit::Lvw),
        ("6dvb", 6.0, CssLengthUnit::Dvb),
        ("7cqi", 7.0, CssLengthUnit::Cqi),
        ("8cm", 8.0, CssLengthUnit::Cm),
        ("9pt", 9.0, CssLengthUnit::Pt),
    ];

    for (authored, expected_value, expected_unit) in cases {
        let value = declaration_value!(&format!(".panel {{ width: {authored}; }}"), Width);

        match value {
            CssLength::Dimension(length) => {
                assert_eq!(length.value(), expected_value);
                assert_eq!(length.unit(), expected_unit);
                assert_eq!(length.to_css_string(), authored);
            }
            other => panic!("expected authored dimension for {authored}, got {other:?}"),
        }
    }
}

#[test]
fn parses_supported_calc_length_units_as_authored_dimensions() {
    let cases = [
        ("1em", 1.0, CssLengthUnit::Em),
        ("2rem", 2.0, CssLengthUnit::Rem),
        ("3vw", 3.0, CssLengthUnit::Vw),
        ("4svh", 4.0, CssLengthUnit::Svh),
        ("5lvw", 5.0, CssLengthUnit::Lvw),
        ("6dvb", 6.0, CssLengthUnit::Dvb),
        ("7cqi", 7.0, CssLengthUnit::Cqi),
        ("8cm", 8.0, CssLengthUnit::Cm),
        ("9pt", 9.0, CssLengthUnit::Pt),
    ];

    for (authored, expected_value, expected_unit) in cases {
        let value = declaration_value!(
            &format!(".panel {{ width: calc({authored} + 2px); }}"),
            Width
        );

        let CssLength::Calc(CssCalcLength::Sum(terms)) = value else {
            panic!("expected calc length for {authored}");
        };
        assert_eq!(terms.len(), 2);
        match terms[0].value() {
            CssCalcLength::Dimension(length) => {
                assert_eq!(length.value(), expected_value);
                assert_eq!(length.unit(), expected_unit);
                assert_eq!(length.to_css_string(), authored);
            }
            other => panic!("expected authored calc dimension for {authored}, got {other:?}"),
        }
        assert_eq!(terms[1].value(), &CssCalcLength::px(2.0));
    }
}

#[test]
fn unit_matrix_accepts_every_supported_length_unit_in_ordinary_length_contexts() {
    for unit in supported_length_units() {
        let authored = format!("1{}", unit.as_css_str());
        let declaration = parse_single_declaration("width", &authored);

        assert_eq!(declaration.property(), &CssProperty::Width);
        assert_eq!(
            declaration_payload!(declaration, Width),
            CssLength::dimension(1.0, unit),
            "{authored} should preserve its supported length unit",
        );
    }
}

#[test]
fn unit_matrix_accepts_every_supported_length_unit_in_calc_contexts() {
    for unit in supported_length_units() {
        let authored = format!("calc(1{} + 2px)", unit.as_css_str());
        let declaration = parse_single_declaration("width", &authored);

        assert_eq!(declaration.property(), &CssProperty::Width);
        let CssLength::Calc(CssCalcLength::Sum(terms)) = declaration_payload!(declaration, Width)
        else {
            panic!("{authored} should parse as a calc length");
        };
        assert_eq!(terms.len(), 2);
        assert_eq!(
            terms[0].value(),
            &CssCalcLength::dimension(1.0, unit),
            "{authored} should preserve its supported calc length unit",
        );
        assert_eq!(terms[1].value(), &CssCalcLength::px(2.0));
    }
}

#[test]
fn unit_matrix_rejects_unknown_length_units_in_ordinary_and_calc_contexts() {
    assert_sheet_rejected(
        ".panel { width: 1quux; }",
        &ExpectedErrorKind::UnsupportedValue {
            property: Some("width"),
        },
    );
    assert_sheet_rejected(
        ".panel { width: calc(1quux + 2px); }",
        &ExpectedErrorKind::UnsupportedValue {
            property: Some("width"),
        },
    );
}

fn supported_length_units() -> [CssLengthUnit; 49] {
    [
        CssLengthUnit::Px,
        CssLengthUnit::Em,
        CssLengthUnit::Rem,
        CssLengthUnit::Ex,
        CssLengthUnit::Rex,
        CssLengthUnit::Cap,
        CssLengthUnit::Rcap,
        CssLengthUnit::Ch,
        CssLengthUnit::Rch,
        CssLengthUnit::Ic,
        CssLengthUnit::Ric,
        CssLengthUnit::Lh,
        CssLengthUnit::Rlh,
        CssLengthUnit::Vw,
        CssLengthUnit::Vh,
        CssLengthUnit::Vi,
        CssLengthUnit::Vb,
        CssLengthUnit::Vmin,
        CssLengthUnit::Vmax,
        CssLengthUnit::Svw,
        CssLengthUnit::Svh,
        CssLengthUnit::Svi,
        CssLengthUnit::Svb,
        CssLengthUnit::Svmin,
        CssLengthUnit::Svmax,
        CssLengthUnit::Lvw,
        CssLengthUnit::Lvh,
        CssLengthUnit::Lvi,
        CssLengthUnit::Lvb,
        CssLengthUnit::Lvmin,
        CssLengthUnit::Lvmax,
        CssLengthUnit::Dvw,
        CssLengthUnit::Dvh,
        CssLengthUnit::Dvi,
        CssLengthUnit::Dvb,
        CssLengthUnit::Dvmin,
        CssLengthUnit::Dvmax,
        CssLengthUnit::Cqw,
        CssLengthUnit::Cqh,
        CssLengthUnit::Cqi,
        CssLengthUnit::Cqb,
        CssLengthUnit::Cqmin,
        CssLengthUnit::Cqmax,
        CssLengthUnit::Cm,
        CssLengthUnit::Mm,
        CssLengthUnit::Q,
        CssLengthUnit::In,
        CssLengthUnit::Pc,
        CssLengthUnit::Pt,
    ]
}

#[test]
fn typo_property_has_unknown_property_error_kind() {
    let error = parse_sheet(".panel { widht: 10px; }").unwrap_err();

    let ErrorKind::UnknownProperty(detail) = error.kind() else {
        panic!("unexpected error kind: {:?}", error.kind());
    };
    assert_eq!(detail.name().as_str(), "widht");
}

#[test]
fn parses_global_keywords_for_different_value_domains() {
    assert_eq!(
        declaration_global!(".panel { width: inherit; }", Width),
        CssGlobalKeyword::Inherit
    );
    assert_eq!(
        declaration_global!(".panel { display: initial; }", Display),
        CssGlobalKeyword::Initial
    );
    assert_eq!(
        declaration_global!(".panel { color: unset; }", Color),
        CssGlobalKeyword::Unset
    );
}

#[test]
fn parses_newer_global_keywords_as_authored_syntax() {
    assert_eq!(
        declaration_global!(".panel { padding: revert; }", Padding),
        CssGlobalKeyword::Revert
    );
    assert_eq!(
        declaration_global!(".panel { margin: revert-layer; }", Margin),
        CssGlobalKeyword::RevertLayer
    );
}

#[test]
fn parses_all_property_global_keywords_as_authored_syntax() {
    let cases = [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ];

    for (authored, expected) in cases {
        assert_eq!(
            declaration_global!(&format!(".panel {{ all: {authored}; }}"), All),
            expected
        );
    }
}

#[test]
fn rejects_non_global_all_values_with_typed_unsupported_value() {
    for input in [".panel { all: block; }", ".panel { all: 1px; }"] {
        let error = parse_sheet(input).expect_err(input);

        let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
            panic!("unexpected error kind: {:?}", error.kind());
        };
        assert_eq!(detail.property(), CssKnownProperty::All);
    }
}

#[test]
fn global_keyword_must_be_the_whole_value() {
    let error = parse_sheet(".panel { width: inherit 10px; }").unwrap_err();

    assert!(matches!(
        error.kind(),
        ErrorKind::InvalidPropertyValue(_) | ErrorKind::UnexpectedToken(_)
    ));
}

#[test]
fn unsupported_display_keyword_is_typed_with_property_context() {
    let error = parse_sheet(".panel { display: inline; }").unwrap_err();

    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("unexpected error kind: {:?}", error.kind());
    };
    assert_eq!(detail.property(), CssKnownProperty::Display);
}

#[test]
fn unsupported_overflow_keyword_is_typed_with_property_context() {
    let error = parse_sheet(".panel { overflow: auto; }").unwrap_err();

    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("unexpected error kind: {:?}", error.kind());
    };
    assert_eq!(detail.property(), CssKnownProperty::Overflow);
}

#[test]
fn unsupported_position_keyword_is_typed_with_property_context() {
    let error = parse_sheet(".panel { position: running; }").unwrap_err();

    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("unexpected error kind: {:?}", error.kind());
    };
    assert_eq!(detail.property(), CssKnownProperty::Position);
}

#[test]
fn unsupported_alignment_keyword_is_typed_with_property_context() {
    let error = parse_sheet(".panel { align-items: unsafe center; }").unwrap_err();

    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("unexpected error kind: {:?}", error.kind());
    };
    assert_eq!(detail.property(), CssKnownProperty::AlignItems);
}

#[test]
fn parses_position_float_clear_visibility_values() {
    assert_eq!(
        declaration_value!(".panel { position: static; }", Position),
        CssLayoutPosition::Static
    );
    assert_eq!(
        declaration_value!(".panel { position: fixed; }", Position),
        CssLayoutPosition::Fixed
    );
    assert_eq!(
        declaration_value!(".panel { position: sticky; }", Position),
        CssLayoutPosition::Sticky
    );
    assert_eq!(
        declaration_value!(".panel { float: left; }", Float),
        CssFloat::Left
    );
    assert_eq!(
        declaration_value!(".panel { clear: both; }", Clear),
        CssClear::Both
    );
    assert_eq!(
        declaration_value!(".panel { visibility: collapse; }", Visibility),
        CssVisibility::Collapse
    );
    assert_eq!(
        declaration_value!(".panel { content-visibility: auto; }", ContentVisibility),
        CssContentVisibility::Auto
    );
}

#[test]
fn parses_content_alignment_and_place_shorthands() {
    assert_eq!(
        declaration_value!(".panel { align-content: space-between; }", AlignContent),
        CssAlignment::SpaceBetween
    );
    assert_eq!(
        declaration_value!(".panel { justify-content: safe center; }", JustifyContent),
        CssAlignment::SafeCenter
    );
    assert_eq!(
        declaration_value!(".panel { align-items: first baseline; }", AlignItems),
        CssAlignItems::FirstBaseline
    );
    assert_eq!(
        declaration_value!(".panel { place-content: center end; }", PlaceContent),
        CssPlaceAlignment::content(CssAlignment::Center, CssAlignment::End)
    );
    assert_eq!(
        declaration_value!(".panel { place-items: stretch; }", PlaceItems),
        CssPlaceAlignment::items_all(CssAlignItems::Stretch)
    );
    assert_eq!(
        declaration_value!(".panel { place-self: end center; }", PlaceSelf),
        CssPlaceAlignment::items(CssAlignItems::End, CssAlignItems::Center)
    );
}

#[test]
fn preserves_explicit_safe_alignment_values() {
    assert_eq!(
        declaration_value!(".panel { align-items: safe end; }", AlignItems),
        CssAlignItems::SafeEnd
    );
    assert_eq!(
        declaration_value!(".panel { align-self: safe flex-end; }", AlignSelf),
        CssAlignItems::SafeFlexEnd
    );
    assert_eq!(
        declaration_value!(".panel { justify-content: safe center; }", JustifyContent),
        CssAlignment::SafeCenter
    );
}

#[test]
fn rejects_positioning_alignment_and_visibility_leakage_values() {
    let cases = [
        ".panel { float: center; }",
        ".panel { clear: start; }",
        ".panel { align-content: left; }",
        ".panel { justify-content: auto; }",
        ".panel { place-items: auto; }",
        ".panel { place-items: space-between; }",
        ".panel { visibility: auto; }",
        ".panel { content-visibility: collapse; }",
    ];

    for case in cases {
        assert!(parse_sheet(case).is_err(), "{case} should be rejected");
    }
}

#[test]
fn rejects_unmodeled_safe_prefixed_alignment_values() {
    let cases = [
        ".panel { align-items: safe start; }",
        ".panel { align-items: safe flex-start; }",
        ".panel { align-items: safe stretch; }",
        ".panel { align-content: safe start; }",
        ".panel { align-content: safe flex-start; }",
        ".panel { align-content: safe stretch; }",
        ".panel { place-content: safe start; }",
        ".panel { place-content: safe flex-start; }",
        ".panel { place-content: safe stretch; }",
    ];

    for case in cases {
        assert!(parse_sheet(case).is_err(), "{case} should be rejected");
    }
}

#[test]
fn unknown_dimension_units_are_reported_as_unknown_units() {
    let error = parse_sheet(".panel { width: 1quux; }").unwrap_err();

    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("unexpected error kind: {:?}", error.kind());
    };
    assert_eq!(detail.property(), CssKnownProperty::Width);
}

#[test]
fn unknown_calc_dimension_units_are_reported_as_unknown_units() {
    let error = parse_sheet(".panel { width: calc(1quux + 2px); }").unwrap_err();

    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("unexpected error kind: {:?}", error.kind());
    };
    assert_eq!(detail.property(), CssKnownProperty::Width);
}

#[test]
fn selector_parse_failure_has_typed_error_kind() {
    let error = parse_sheet("??? { width: 10px; }").unwrap_err();

    assert!(matches!(error.kind(), ErrorKind::InvalidSelector(_)));
}

#[test]
fn selector_missing_class_name_has_typed_error_kind() {
    let error = parse_sheet(". { width: 10px; }").unwrap_err();

    assert!(matches!(error.kind(), ErrorKind::InvalidSelector(_)));
}

#[test]
fn grid_flow_tolerance_calc_is_preserved_as_css_syntax() {
    let value = declaration_value!(
        ".panel { grid-flow-tolerance: calc(8px + 2%); }",
        GridFlowTolerance
    );

    match value {
        CssGridFlowTolerance::Length(CssLength::Calc(calc)) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(8px + 2%)");
        }
        other => panic!("expected calc grid-flow-tolerance, got {other:?}"),
    }
}

#[test]
fn rejects_unknown_calc_functions() {
    let error = parse_sheet(".panel { width: min(10px, 20px); }").unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn parses_calc_in_edge_shorthands() {
    let sheet = parse_sheet(".panel { margin: calc(4px + 1%) 2px; }").unwrap();
    let edges = declaration_value!(".panel { margin: calc(4px + 1%) 2px; }", Margin);

    match &edges.top {
        CssLength::Calc(calc) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(4px + 1%)");
        }
        other => panic!("expected calc top edge, got {other:?}"),
    }
    assert_eq!(edges.right, CssLength::px(2.0));
    match &edges.bottom {
        CssLength::Calc(calc) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(4px + 1%)");
        }
        other => panic!("expected calc bottom edge, got {other:?}"),
    }
    assert_eq!(edges.left, CssLength::px(2.0));

    assert_eq!(style_rule(&sheet.rules()[0]).declarations().len(), 1);
}

#[test]
fn parses_authored_normal_gap_without_canonicalizing_it() {
    let value = declaration_value!(".panel { gap: normal; }", Gap);
    assert_eq!(value, CssLength::Normal);
}

#[test]
fn parses_authored_calc_gap_without_canonicalizing_it() {
    let value = declaration_value!(".panel { gap: calc(8px + 2%); }", Gap);
    match value {
        CssLength::Calc(calc) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(8px + 2%)");
        }
        other => panic!("expected calc gap, got {other:?}"),
    }
}

#[test]
fn rejects_line_height_auto() {
    let error = parse_sheet(".panel { line-height: auto; }").unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn rejects_line_height_min_content() {
    let error = parse_sheet(".panel { line-height: min-content; }").unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn rejects_font_size_auto() {
    let error = parse_sheet(".panel { font-size: auto; }").unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn parses_typography_and_text_keyword_families() {
    assert_eq!(
        declaration_value!(".panel { writing-mode: vertical-rl; }", WritingMode),
        CssWritingMode::VerticalRl
    );
    assert_eq!(
        declaration_value!(".panel { text-align: start; }", TextAlign),
        CssTextAlign::Start
    );
    assert_eq!(
        declaration_value!(".panel { text-align-last: justify; }", TextAlignLast),
        CssTextAlignLast::Justify
    );
    assert_eq!(
        declaration_value!(".panel { text-wrap: balance; }", TextWrap),
        CssTextWrap::Balance
    );
    assert_eq!(
        declaration_value!(".panel { white-space: pre-wrap; }", WhiteSpace),
        CssWhiteSpace::PreWrap
    );
    assert_eq!(
        declaration_value!(".panel { word-break: keep-all; }", WordBreak),
        CssWordBreak::KeepAll
    );
    assert_eq!(
        declaration_value!(".panel { overflow-wrap: anywhere; }", OverflowWrap),
        CssOverflowWrap::Anywhere
    );
    assert_eq!(
        declaration_value!(".panel { text-overflow: ellipsis; }", TextOverflow),
        CssTextOverflow::Ellipsis
    );
    assert_eq!(
        declaration_value!(".panel { text-transform: uppercase; }", TextTransform),
        CssTextTransform::Uppercase
    );
}

#[test]
fn parses_typography_and_text_length_families() {
    assert_eq!(
        declaration_value!(".panel { text-indent: 2em; }", TextIndent),
        CssTextIndent::new(CssLength::dimension(2.0, CssLengthUnit::Em), false, false,)
    );
    assert_eq!(
        declaration_value!(".panel { vertical-align: 4px; }", VerticalAlign),
        CssVerticalAlign::Length(CssVerticalAlignLength::new(CssLength::px(4.0)))
    );
    assert_eq!(
        declaration_value!(".panel { letter-spacing: normal; }", LetterSpacing),
        CssLetterSpacing::Normal
    );
    assert_eq!(
        declaration_value!(".panel { letter-spacing: 0.1em; }", LetterSpacing),
        CssLetterSpacing::Length(CssLetterSpacingLength::new(CssLength::dimension(
            0.1,
            CssLengthUnit::Em
        )))
    );
    assert_eq!(
        declaration_value!(
            ".panel { text-decoration-thickness: from-font; }",
            TextDecorationThickness
        ),
        CssTextDecorationThickness::FromFont
    );
    assert_eq!(
        declaration_value!(
            ".panel { text-decoration-thickness: 2px; }",
            TextDecorationThickness
        ),
        CssTextDecorationThickness::Length(CssTextDecorationThicknessLength::new(CssLength::px(
            2.0
        )))
    );
}

#[test]
fn parses_font_families_and_font_shorthand_as_authored_syntax() {
    let family = declaration_value!(
        ".panel { font-family: \"Avenir Next\", Gill Sans, sans-serif; }",
        FontFamily
    );
    assert_eq!(
        family.families(),
        [
            CssFontFamilyName::try_quoted("Avenir Next").unwrap(),
            CssFontFamilyName::try_ident_sequence("Gill Sans").unwrap(),
            CssFontFamilyName::try_ident_sequence("sans-serif").unwrap(),
        ]
    );

    assert_eq!(
        declaration_value!(".panel { font-weight: 725; }", FontWeight),
        CssFontWeight::Number(CssFontWeightNumber::new(725))
    );
    assert_eq!(
        declaration_value!(".panel { font-style: italic; }", FontStyle),
        CssFontStyle::Italic
    );
    assert_eq!(
        declaration_value!(".panel { font-stretch: semi-condensed; }", FontStretch),
        CssFontStretch::SemiCondensed
    );
    assert_eq!(
        declaration_value!(".panel { font-variant: small-caps; }", FontVariant),
        CssFontVariant::SmallCaps
    );
    assert_eq!(
        declaration_value!(
            ".panel { font-feature-settings: \"kern\" on, \"liga\" 0; }",
            FontFeatureSettings
        ),
        CssFontFeatureSettings::Features(CssFontFeatureList::new(vec![
            CssFontFeature::new("kern", Some(CssFontFeatureValue::On)),
            CssFontFeature::new("liga", Some(CssFontFeatureValue::Integer(0))),
        ]))
    );

    let font = declaration_value!(
        ".panel { font: italic small-caps 700 condensed 16px/normal \"Avenir Next\", sans-serif; }",
        Font
    );
    assert_eq!(font.style(), Some(CssFontStyle::Italic));
    assert_eq!(font.variant(), Some(CssFontVariant::SmallCaps));
    assert_eq!(
        font.weight(),
        Some(CssFontWeight::Number(CssFontWeightNumber::new(700)))
    );
    assert_eq!(font.stretch(), Some(CssFontStretch::Condensed));
    assert_eq!(font.size(), &CssLength::px(16.0));
    assert_eq!(font.line_height(), Some(&CssLength::Normal));
    assert_eq!(
        font.families().families(),
        [
            CssFontFamilyName::try_quoted("Avenir Next").unwrap(),
            CssFontFamilyName::try_ident_sequence("sans-serif").unwrap(),
        ]
    );
}

#[test]
fn parses_text_decoration_family() {
    assert_eq!(
        declaration_value!(
            ".panel { text-decoration-line: underline overline; }",
            TextDecorationLine
        ),
        CssTextDecorationLine::new(vec![
            CssTextDecorationLineComponent::Underline,
            CssTextDecorationLineComponent::Overline,
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { text-decoration-color: black; }",
            TextDecorationColor
        ),
        CssColor::BLACK
    );
    assert_eq!(
        declaration_value!(
            ".panel { text-decoration-style: wavy; }",
            TextDecorationStyle
        ),
        CssTextDecorationStyle::Wavy
    );

    let value = declaration_value!(
        ".panel { text-decoration: underline dotted white 3px; }",
        TextDecoration
    );
    assert_eq!(
        value,
        CssTextDecoration::new(
            Some(CssTextDecorationLine::new(vec![
                CssTextDecorationLineComponent::Underline
            ])),
            Some(CssColor::WHITE),
            Some(CssTextDecorationStyle::Dotted),
            Some(CssTextDecorationThickness::Length(
                CssTextDecorationThicknessLength::new(CssLength::px(3.0))
            )),
        )
    );
}

#[test]
fn checked_typography_constructors_reject_invalid_states() {
    assert_eq!(CssFontFamilyList::try_new(Vec::new()), None);
    assert_eq!(CssFontWeightNumber::try_new(0), None);
    assert_eq!(CssFontWeightNumber::try_new(1001), None);
    assert_eq!(
        CssFontWeightNumber::try_new(500),
        Some(CssFontWeightNumber::new(500))
    );
    assert_eq!(CssFontFeatureList::try_new(Vec::new()), None);
    assert_eq!(CssTextDecorationLine::try_new(Vec::new()), None);
    assert!(
        CssFont::try_new(
            None,
            None,
            None,
            None,
            CssLength::px(12.0),
            None,
            CssFontFamilyList::new(vec![CssFontFamilyName::ident_sequence("sans-serif")]),
        )
        .is_some(),
    );
    assert_eq!(
        CssFont::try_new(
            None,
            None,
            None,
            None,
            CssLength::Auto,
            None,
            CssFontFamilyList::new(vec![CssFontFamilyName::ident_sequence("sans-serif")]),
        ),
        None
    );
    assert_eq!(CssFontFamilyName::try_quoted(""), None);
    assert_eq!(CssFontFamilyName::try_ident_sequence(""), None);
    assert_eq!(
        CssFontFamilyList::try_new(vec![CssFontFamilyName::ident_sequence("")]),
        None
    );
    assert_eq!(CssFontFeature::try_new("abc", None), None);
    assert_eq!(CssFontFeature::try_new("abcde", None), None);
    assert_eq!(
        CssFontFeature::try_new("kern", Some(CssFontFeatureValue::On)),
        Some(CssFontFeature::new("kern", Some(CssFontFeatureValue::On)))
    );
    assert_eq!(CssVerticalAlignLength::try_new(CssLength::Auto), None);
    assert_eq!(
        CssLetterSpacingLength::try_new(CssLength::percent(10.0)),
        None
    );
    assert_eq!(
        CssTextDecorationThicknessLength::try_new(CssLength::px(-1.0)),
        None
    );
    assert_eq!(
        CssTextDecorationLine::try_new(vec![
            CssTextDecorationLineComponent::Underline,
            CssTextDecorationLineComponent::Underline,
        ]),
        None
    );
}

#[test]
fn typography_and_text_property_families_accept_supported_values() {
    let sheet = parse_sheet(
        ".panel {
            writing-mode: horizontal-tb;
            text-align: center;
            text-align-last: auto;
            text-indent: 1rem hanging each-line;
            vertical-align: super;
            font-family: \"Avenir Next\", sans-serif;
            font: italic 700 16px/normal \"Avenir Next\", sans-serif;
            font-weight: bold;
            font-style: oblique;
            font-stretch: expanded;
            font-variant: normal;
            font-feature-settings: normal;
            letter-spacing: 1px;
            text-wrap: wrap;
            white-space: nowrap;
            word-break: break-word;
            overflow-wrap: break-word;
            text-overflow: clip;
            text-decoration: underline solid black 1px;
            text-decoration-line: none;
            text-decoration-color: transparent;
            text-decoration-style: solid;
            text-decoration-thickness: auto;
            text-transform: capitalize;
        }",
    )
    .unwrap();
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssProperty::WritingMode,
        CssProperty::TextAlign,
        CssProperty::TextAlignLast,
        CssProperty::TextIndent,
        CssProperty::VerticalAlign,
        CssProperty::FontFamily,
        CssProperty::Font,
        CssProperty::FontWeight,
        CssProperty::FontStyle,
        CssProperty::FontStretch,
        CssProperty::FontVariant,
        CssProperty::FontFeatureSettings,
        CssProperty::LetterSpacing,
        CssProperty::TextWrap,
        CssProperty::WhiteSpace,
        CssProperty::WordBreak,
        CssProperty::OverflowWrap,
        CssProperty::TextOverflow,
        CssProperty::TextDecoration,
        CssProperty::TextDecorationLine,
        CssProperty::TextDecorationColor,
        CssProperty::TextDecorationStyle,
        CssProperty::TextDecorationThickness,
        CssProperty::TextTransform,
    ] {
        assert!(
            declarations
                .iter()
                .any(|declaration| declaration.property() == property),
            "missing parsed declaration for {property:?}",
        );
    }
}

#[test]
fn typography_and_text_properties_reject_cross_family_values() {
    for input in [
        ".panel { font-size: auto; }",
        ".panel { font-weight: 1001; }",
        ".panel { font-style: bold; }",
        ".panel { font-family:; }",
        ".panel { letter-spacing: auto; }",
        ".panel { text-decoration-style: 2px; }",
        ".panel { text-transform: wrap; }",
        ".panel { font-feature-settings: \"abc\" on; }",
        ".panel { font-feature-settings: \"abcde\" on; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(
            error.kind(),
            ErrorKind::InvalidPropertyValue(_)
                | ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidQualifiedRule(_)
                | ErrorKind::InvalidColorSyntax(_)
        ));
    }
}

#[test]
fn parses_background_properties_as_authored_syntax() {
    assert_eq!(
        declaration_value!(
            ".panel { background-image: url(\"hero.png\"), none; }",
            BackgroundImage
        ),
        CssImageLayerList::new(vec![
            CssImageLayer::Url(CssUrl::new("hero.png")),
            CssImageLayer::None,
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { background-position: left 10px top 20%; }",
            BackgroundPosition
        ),
        CssPositionList::new(vec![CssPosition::new(vec![
            CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Left),
            CssPositionComponent::Length(CssLength::px(10.0)),
            CssPositionComponent::Vertical(CssVerticalPositionKeyword::Top),
            CssPositionComponent::Length(CssLength::percent(20.0)),
        ])])
    );
    assert_eq!(
        declaration_value!(
            ".panel { background-size: cover, 10px auto; }",
            BackgroundSize
        ),
        CssBackgroundSizeList::new(vec![
            CssBackgroundSize::Cover,
            CssBackgroundSize::Explicit {
                width: CssBackgroundSizeComponent::Length(CssLength::px(10.0)),
                height: Some(CssBackgroundSizeComponent::Auto),
            },
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { background-repeat: repeat-x, no-repeat round; }",
            BackgroundRepeat
        ),
        CssBackgroundRepeatList::new(vec![
            CssBackgroundRepeat::RepeatX,
            CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::NoRepeat,
                y: CssBackgroundRepeatStyle::Round,
            },
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { background-origin: content-box; }",
            BackgroundOrigin
        ),
        CssBackgroundBox::ContentBox
    );
    assert_eq!(
        declaration_value!(".panel { background-clip: padding-box; }", BackgroundClip),
        CssBackgroundBox::PaddingBox
    );
    assert_eq!(
        declaration_value!(
            ".panel { background-attachment: fixed, local; }",
            BackgroundAttachment
        ),
        CssBackgroundAttachmentList::new(vec![
            CssBackgroundAttachment::Fixed,
            CssBackgroundAttachment::Local,
        ])
    );
}

#[test]
fn parses_interaction_and_outline_properties_as_authored_syntax() {
    assert_eq!(
        declaration_value!(".panel { cursor: grab; }", Cursor),
        CssCursor::Keyword(CssCursorKeyword::Grab)
    );
    assert_eq!(
        declaration_value!(".panel { pointer-events: none; }", PointerEvents),
        CssPointerEvents::None
    );
    assert_eq!(
        declaration_value!(".panel { user-select: text; }", UserSelect),
        CssUserSelect::Text
    );
    assert_eq!(
        declaration_value!(".panel { outline: thick dotted white; }", Outline),
        CssOutline::new(
            Some(CssOutlineWidth::Thick),
            Some(CssOutlineStyle::Border(CssBorderStyle::Dotted)),
            Some(CssColor::WHITE),
        )
    );
    assert_eq!(
        declaration_value!(".panel { outline-width: 2px; }", OutlineWidth),
        CssOutlineWidth::Length(CssLength::px(2.0))
    );
}

#[test]
fn parses_transform_effect_and_mask_properties_as_authored_syntax() {
    let transform = declaration_value!(
        ".panel { transform: translate(10px, 20px) rotate(45deg) scale(1.5); }",
        Transform
    );
    let CssTransform::Functions(functions) = transform else {
        panic!("expected transform functions");
    };
    assert_eq!(functions.functions().len(), 3);
    assert_eq!(
        functions.functions()[0].kind(),
        CssTransformFunctionKind::Translate
    );

    assert_eq!(
        declaration_value!(".panel { transform-origin: center top; }", TransformOrigin),
        CssPosition::new(vec![
            CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
            CssPositionComponent::Vertical(CssVerticalPositionKeyword::Top),
        ])
    );
    assert_eq!(
        declaration_value!(".panel { filter: blur(4px) opacity(50%); }", Filter),
        CssFilter::Functions(CssFilterFunctionList::new(vec![
            CssFilterFunction::Blur(filter_arguments("4px")),
            CssFilterFunction::Opacity(filter_arguments("50%")),
        ]))
    );
    assert_eq!(
        declaration_value!(".panel { backdrop-filter: none; }", BackdropFilter),
        CssFilter::None
    );
    assert_eq!(
        declaration_value!(".panel { clip-path: circle(50% at center); }", ClipPath),
        CssClipPath::BasicShape(CssBasicShape::Circle(basic_shape_arguments(
            "50% at center"
        ),))
    );
    assert_eq!(
        declaration_value!(".panel { mask-image: url(mask.png), none; }", MaskImage),
        CssImageLayerList::new(vec![
            CssImageLayer::Url(CssUrl::new("mask.png")),
            CssImageLayer::None,
        ])
    );
    let mask_layers = declaration_value!(
        ".panel { mask: url(mask.png) center / contain no-repeat; }",
        Mask
    );
    assert_eq!(mask_layers.layers().len(), 1);
}

#[test]
fn authored_transform_filter_easing_and_basic_shape_arguments_preserve_css_with_family_context() {
    fn transform_css(arguments: &CssTransformArguments) -> &str {
        arguments.as_css()
    }
    fn filter_css(arguments: &CssFilterArguments) -> &str {
        arguments.as_css()
    }
    fn basic_shape_css(arguments: &CssBasicShapeArguments) -> &str {
        arguments.as_css()
    }
    fn easing_css(arguments: &CssEasingArguments) -> &str {
        arguments.as_css()
    }

    let CssTransform::Functions(functions) = declaration_value!(
        ".panel { transform: translate(10px, 20px) rotate(45deg); }",
        Transform
    ) else {
        panic!("expected transform functions");
    };
    assert_eq!(
        transform_css(functions.functions()[0].arguments()),
        "10px, 20px"
    );

    let CssFilter::Functions(functions) =
        declaration_value!(".panel { filter: blur(4px) opacity(50%); }", Filter)
    else {
        panic!("expected filter functions");
    };
    let CssFilterFunction::Opacity(arguments) = &functions.functions()[1] else {
        panic!("expected opacity filter");
    };
    assert_eq!(filter_css(arguments), "50%");

    let CssClipPath::BasicShape(CssBasicShape::Circle(arguments)) =
        declaration_value!(".panel { clip-path: circle(50% at center); }", ClipPath)
    else {
        panic!("expected basic shape clip-path");
    };
    assert_eq!(basic_shape_css(&arguments), "50% at center");

    let easings = declaration_value!(
        ".panel { transition-timing-function: cubic-bezier(0.1, 0.2, 0.3, 1); }",
        TransitionTimingFunction
    );
    let CssEasing::CubicBezier(arguments) = &easings.easings()[0] else {
        panic!("expected cubic-bezier easing");
    };
    assert_eq!(easing_css(arguments), "0.1, 0.2, 0.3, 1");
}

#[test]
fn parses_transition_properties_and_preserves_comma_lists() {
    assert_eq!(
        declaration_value!(
            ".panel { transition-property: opacity, transform; }",
            TransitionProperty
        ),
        CssTransitionPropertyList::new(vec![
            CssTransitionProperty::Custom(CssCustomIdent::new("opacity")),
            CssTransitionProperty::Custom(CssCustomIdent::new("transform")),
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { transition-duration: 150ms, 2s; }",
            TransitionDuration
        ),
        CssTimeList::new(vec![
            CssTime::try_milliseconds(150.0).unwrap(),
            CssTime::try_seconds(2.0).unwrap(),
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { transition-timing-function: ease-in, cubic-bezier(0.1, 0.2, 0.3, 1); }",
            TransitionTimingFunction
        ),
        CssEasingList::new(vec![
            CssEasing::EaseIn,
            CssEasing::CubicBezier(easing_arguments("0.1, 0.2, 0.3, 1")),
        ])
    );

    let transitions = declaration_value!(
        ".panel { transition: opacity 150ms ease-in 20ms, transform 2s linear; }",
        Transition
    );
    assert_eq!(transitions.items().len(), 2);
}

#[test]
fn parses_animation_properties_and_preserves_comma_lists() {
    assert_eq!(
        declaration_value!(".panel { animation-name: fade, none; }", AnimationName),
        CssAnimationNameList::new(vec![
            CssAnimationName::Custom(CssCustomIdent::new("fade")),
            CssAnimationName::None,
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { animation-iteration-count: 2, infinite; }",
            AnimationIterationCount
        ),
        CssAnimationIterationCountList::new(vec![
            CssAnimationIterationCount::try_number(2.0).unwrap(),
            CssAnimationIterationCount::Infinite,
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { animation-play-state: running, paused; }",
            AnimationPlayState
        ),
        CssAnimationPlayStateList::new(vec![
            CssAnimationPlayState::Running,
            CssAnimationPlayState::Paused,
        ])
    );

    let animations = declaration_value!(
        ".panel { animation: fade 1s ease-in 200ms 3 alternate both running, slide 2s linear; }",
        Animation
    );
    assert_eq!(animations.items().len(), 2);
}

#[test]
fn background_effect_and_animation_constructors_reject_invalid_states() {
    assert_eq!(CssImageLayerList::try_new(Vec::new()), None);
    assert_eq!(CssCursorUrlList::try_new(Vec::new()), None);
    assert!(CssCursor::try_urls(Vec::new(), CssCursorKeyword::Pointer).is_none());
    assert_eq!(
        CssPosition::try_new(vec![
            CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Left),
            CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Right),
        ]),
        None
    );
    assert_eq!(
        CssPosition::try_new(vec![
            CssPositionComponent::Vertical(CssVerticalPositionKeyword::Top),
            CssPositionComponent::Vertical(CssVerticalPositionKeyword::Bottom),
        ]),
        None
    );
    assert_eq!(CssTranslateValues::try_new(Vec::new()), None);
    assert_eq!(
        CssTranslateValues::try_new(vec![
            CssLength::px(1.0),
            CssLength::px(2.0),
            CssLength::px(3.0),
            CssLength::px(4.0),
        ]),
        None
    );
    assert_eq!(CssScaleValues::try_new(Vec::new()), None);
    assert_eq!(CssScaleValues::try_new(vec![1.0, 2.0, 3.0, 4.0]), None);
    assert_eq!(CssMaskList::try_new(Vec::new()), None);
    assert_eq!(CssTransitionList::try_new(Vec::new()), None);
    assert_eq!(CssTransition::try_new(None, None, None, None), None);
    assert_eq!(CssAnimationList::try_new(Vec::new()), None);
    assert_eq!(
        CssAnimation::try_new(CssAnimationComponents::default()),
        None
    );
    assert_eq!(CssTime::try_seconds(-1.0), None);
    assert_eq!(CssAnimationIterationCount::try_number(-1.0), None);
    assert_eq!(CssOutline::try_new(None, None, None), None);
}

#[test]
fn background_effect_and_animation_property_families_accept_supported_values() {
    let sheet = parse_sheet(
        ".panel {
            background-image: none;
            background-position: center;
            background-size: contain;
            background-repeat: no-repeat;
            background-origin: border-box;
            background-clip: content-box;
            background-attachment: scroll;
            cursor: pointer;
            pointer-events: auto;
            user-select: all;
            outline: 1px solid black;
            outline-color: white;
            outline-style: dashed;
            outline-width: thin;
            transform: none;
            transform-origin: left top;
            translate: 10px 20px;
            rotate: 45deg;
            scale: 1.5 2;
            filter: none;
            backdrop-filter: blur(4px);
            clip-path: none;
            mask: none;
            mask-image: none;
            mask-size: auto;
            mask-position: center;
            mask-repeat: repeat;
            transition-property: opacity;
            transition-duration: 1s;
            transition-delay: 20ms;
            transition-timing-function: ease;
            transition: opacity 1s ease;
            animation-name: fade;
            animation-duration: 1s;
            animation-delay: 20ms;
            animation-timing-function: ease-out;
            animation-iteration-count: infinite;
            animation-direction: alternate;
            animation-fill-mode: both;
            animation-play-state: paused;
            animation: fade 1s ease-in-out infinite alternate both running;
        }",
    )
    .unwrap();
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssProperty::BackgroundImage,
        CssProperty::BackgroundPosition,
        CssProperty::BackgroundSize,
        CssProperty::BackgroundRepeat,
        CssProperty::BackgroundOrigin,
        CssProperty::BackgroundClip,
        CssProperty::BackgroundAttachment,
        CssProperty::Cursor,
        CssProperty::PointerEvents,
        CssProperty::UserSelect,
        CssProperty::Outline,
        CssProperty::OutlineColor,
        CssProperty::OutlineStyle,
        CssProperty::OutlineWidth,
        CssProperty::Transform,
        CssProperty::TransformOrigin,
        CssProperty::Translate,
        CssProperty::Rotate,
        CssProperty::Scale,
        CssProperty::Filter,
        CssProperty::BackdropFilter,
        CssProperty::ClipPath,
        CssProperty::Mask,
        CssProperty::MaskImage,
        CssProperty::MaskSize,
        CssProperty::MaskPosition,
        CssProperty::MaskRepeat,
        CssProperty::TransitionProperty,
        CssProperty::TransitionDuration,
        CssProperty::TransitionDelay,
        CssProperty::TransitionTimingFunction,
        CssProperty::Transition,
        CssProperty::AnimationName,
        CssProperty::AnimationDuration,
        CssProperty::AnimationDelay,
        CssProperty::AnimationTimingFunction,
        CssProperty::AnimationIterationCount,
        CssProperty::AnimationDirection,
        CssProperty::AnimationFillMode,
        CssProperty::AnimationPlayState,
        CssProperty::Animation,
    ] {
        assert!(
            declarations
                .iter()
                .any(|declaration| declaration.property() == property),
            "missing parsed declaration for {property:?}",
        );
    }
}

#[test]
fn background_effect_and_animation_properties_reject_cross_family_values_and_empty_lists() {
    for input in [
        ".panel { background-size: solid; }",
        ".panel { cursor: 10px; }",
        ".panel { pointer-events: grab; }",
        ".panel { outline-width: 10%; }",
        ".panel { transform: red; }",
        ".panel { filter: 10px; }",
        ".panel { transition-duration: 10px; }",
        ".panel { animation-iteration-count: -1; }",
        ".panel { animation-play-state: visible; }",
        ".panel { transition: opacity 1s, ; }",
        ".panel { animation: fade 1s, ; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(
            error.kind(),
            ErrorKind::InvalidPropertyValue(_)
                | ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidQualifiedRule(_)
        ));
    }
}

#[test]
fn rejects_duplicate_axis_position_keywords_across_shared_position_properties() {
    for input in [
        ".panel { background-position: left right; }",
        ".panel { background-position: right left; }",
        ".panel { background-position: top bottom; }",
        ".panel { background-position: bottom top; }",
        ".panel { mask-position: left right; }",
        ".panel { mask-position: top bottom; }",
        ".panel { transform-origin: left right; }",
        ".panel { transform-origin: top bottom; }",
        ".panel { mask: url(mask.png) left right / contain no-repeat; }",
        ".panel { mask: url(mask.png) top bottom / contain no-repeat; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(
            error.kind(),
            ErrorKind::InvalidPropertyValue(_)
                | ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidQualifiedRule(_)
        ));
    }
}

#[test]
fn preserves_valid_position_keyword_forms_after_duplicate_axis_rejection() {
    for input in [
        ".panel { background-position: left top; }",
        ".panel { background-position: right bottom; }",
        ".panel { background-position: center center; }",
        ".panel { background-position: left 10px top 20%; }",
        ".panel { mask-position: center center; }",
        ".panel { transform-origin: right bottom; }",
        ".panel { mask: url(mask.png) left top / contain no-repeat; }",
    ] {
        parse_sheet(input).unwrap_or_else(|error| panic!("{input} should parse: {error}"));
    }
}

#[test]
fn transform_filter_shape_and_easing_functions_reject_invalid_arguments() {
    for input in [
        ".panel { transform: translate(red); }",
        ".panel { filter: opacity(red); }",
        ".panel { clip-path: circle(red); }",
        ".panel { transition-timing-function: cubic-bezier(red); }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(
            error.kind(),
            ErrorKind::InvalidPropertyValue(_)
                | ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidQualifiedRule(_)
        ));
    }
}

#[test]
fn rejects_padding_auto() {
    let error = parse_sheet(".panel { padding: auto; }").unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn rejects_border_width_percent() {
    let error = parse_sheet(".panel { border-width: 10%; }").unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn rejects_gap_auto() {
    let error = parse_sheet(".panel { gap: auto; }").unwrap_err();
    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}

#[test]
fn accepts_margin_auto() {
    assert_eq!(
        declaration_value!(".panel { margin: auto; }", Margin),
        CssEdges::all(CssLength::Auto)
    );
}

#[test]
fn parses_spacing_inset_and_z_index_values() {
    assert_eq!(
        declaration_value!(".panel { inset: auto 10px 5%; }", Inset),
        CssEdges::new(
            CssLength::Auto,
            CssLength::px(10.0),
            CssLength::percent(5.0),
            CssLength::px(10.0),
        )
    );
    assert_eq!(
        declaration_value!(".panel { top: calc(10px + 5%); }", Top),
        CssLength::Calc(CssCalcLength::sum(
            CssCalcLengthTerm::add(CssCalcLength::px(10.0)),
            [CssCalcLengthTerm::add(CssCalcLength::percent(5.0))]
        ))
    );
    assert_eq!(
        declaration_value!(".panel { z-index: -2; }", ZIndex),
        CssZIndex::Integer(-2)
    );
    assert_eq!(
        declaration_value!(
            ".panel { box-decoration-break: clone; }",
            BoxDecorationBreak
        ),
        CssBoxDecorationBreak::Clone
    );
}

#[test]
fn parses_spacing_longhands_with_existing_component_rules() {
    assert_eq!(
        declaration_value!(".panel { margin-left: auto; }", MarginLeft),
        CssLength::Auto
    );
    assert_eq!(
        declaration_value!(".panel { padding-top: 12px; }", PaddingTop),
        CssLength::px(12.0)
    );
    assert_eq!(
        declaration_value!(".panel { border-right-width: 2px; }", BorderRightWidth),
        CssLength::px(2.0)
    );
}

#[test]
fn parses_border_style_and_border_shorthand_values() {
    assert_eq!(
        declaration_value!(".panel { border-style: solid dashed; }", BorderStyle),
        CssBorderStyles::new(
            CssBorderStyle::Solid,
            CssBorderStyle::Dashed,
            CssBorderStyle::Solid,
            CssBorderStyle::Dashed,
        )
    );
    assert_eq!(
        declaration_value!(".panel { border-left-style: groove; }", BorderLeftStyle),
        CssBorderStyle::Groove
    );
    assert_eq!(
        declaration_value!(".panel { border: solid 2px #fff; }", Border),
        CssBorder::new(
            Some(CssLength::px(2.0)),
            Some(CssBorderStyle::Solid),
            Some(CssColor::WHITE),
        )
    );
    assert_eq!(
        declaration_value!(".panel { border-top: black dotted; }", BorderTop),
        CssBorder::new(None, Some(CssBorderStyle::Dotted), Some(CssColor::BLACK),)
    );
}

#[test]
fn parses_border_radius_shorthand_and_longhands() {
    assert_eq!(
        declaration_value!(
            ".panel { border-top-left-radius: 4px 10%; }",
            BorderTopLeftRadius
        ),
        CssCornerRadius::new(CssLength::px(4.0), CssLength::percent(10.0),)
    );
    assert_eq!(
        declaration_value!(
            ".panel { border-radius: 1px 2px 3px / 4px 5px; }",
            BorderRadius
        ),
        CssBorderRadii::new(
            CssCornerRadius::new(CssLength::px(1.0), CssLength::px(4.0)),
            CssCornerRadius::new(CssLength::px(2.0), CssLength::px(5.0)),
            CssCornerRadius::new(CssLength::px(3.0), CssLength::px(4.0)),
            CssCornerRadius::new(CssLength::px(2.0), CssLength::px(5.0)),
        )
    );
}

#[test]
fn parses_box_shadow_none_and_shadow_lists() {
    assert_eq!(
        declaration_value!(".panel { box-shadow: none; }", BoxShadow),
        CssBoxShadow::None
    );

    let value = declaration_value!(
        ".panel { box-shadow: inset 1px 2px 3px 4px black, 0 1px #fff; }",
        BoxShadow
    );

    let CssBoxShadow::Shadows(shadows) = value else {
        panic!("expected box-shadow list");
    };
    assert_eq!(shadows.shadows().len(), 2);
    assert_eq!(
        shadows.shadows()[0],
        CssShadow::new(
            true,
            CssLength::px(1.0),
            CssLength::px(2.0),
            Some(CssLength::px(3.0)),
            Some(CssLength::px(4.0)),
            Some(CssColor::BLACK),
        )
    );
    assert_eq!(
        shadows.shadows()[1],
        CssShadow::new(
            false,
            CssLength::Zero,
            CssLength::px(1.0),
            None,
            None,
            Some(CssColor::WHITE),
        )
    );
}

#[test]
fn checked_border_constructor_rejects_empty_shorthands() {
    assert_eq!(CssBorder::try_new(None, None, None), None);
    assert_eq!(
        CssBorder::try_new(None, Some(CssBorderStyle::Solid), None),
        Some(CssBorder::new(None, Some(CssBorderStyle::Solid), None))
    );
}

#[test]
fn checked_border_constructor_rejects_parser_invalid_widths() {
    for width in [
        CssLength::Auto,
        CssLength::percent(10.0),
        CssLength::px(-1.0),
        CssLength::MinContent,
        CssLength::Normal,
        CssLength::Calc(CssCalcLength::percent(10.0)),
        CssLength::Calc(CssCalcLength::px(-1.0)),
    ] {
        assert_eq!(
            CssBorder::try_new(Some(width), Some(CssBorderStyle::Solid), None),
            None
        );
    }

    assert_eq!(
        CssBorder::try_new(
            Some(CssLength::Calc(CssCalcLength::px(1.0))),
            Some(CssBorderStyle::Solid),
            None,
        ),
        Some(CssBorder::new(
            Some(CssLength::Calc(CssCalcLength::px(1.0))),
            Some(CssBorderStyle::Solid),
            None,
        ))
    );
}

#[test]
fn checked_corner_radius_constructor_rejects_parser_invalid_values() {
    for value in [
        CssLength::Auto,
        CssLength::MinContent,
        CssLength::MaxContent,
        CssLength::FitContent,
        CssLength::Normal,
        CssLength::px(-1.0),
        CssLength::percent(-1.0),
        CssLength::Calc(CssCalcLength::px(-1.0)),
        CssLength::Calc(CssCalcLength::percent(-1.0)),
    ] {
        assert_eq!(
            CssCornerRadius::try_new(value.clone(), CssLength::px(1.0)),
            None
        );
        assert_eq!(CssCornerRadius::try_new(CssLength::px(1.0), value), None);
    }

    assert_eq!(
        CssCornerRadius::try_new(CssLength::px(1.0), CssLength::percent(25.0)),
        Some(CssCornerRadius::new(
            CssLength::px(1.0),
            CssLength::percent(25.0)
        ))
    );
}

#[test]
fn checked_shadow_constructor_rejects_invalid_pairings_and_lengths() {
    assert_eq!(
        CssShadow::try_new(false, CssLength::Auto, CssLength::px(2.0), None, None, None,),
        None
    );
    assert_eq!(
        CssShadow::try_new(
            false,
            CssLength::px(1.0),
            CssLength::px(2.0),
            None,
            Some(CssLength::px(4.0)),
            None,
        ),
        None
    );
    assert_eq!(
        CssShadow::try_new(
            false,
            CssLength::px(1.0),
            CssLength::px(2.0),
            Some(CssLength::px(-3.0)),
            None,
            None,
        ),
        None
    );
    assert_eq!(
        CssShadow::try_new(
            false,
            CssLength::px(-1.0),
            CssLength::px(2.0),
            Some(CssLength::px(3.0)),
            Some(CssLength::px(-4.0)),
            None,
        ),
        Some(CssShadow::new(
            false,
            CssLength::px(-1.0),
            CssLength::px(2.0),
            Some(CssLength::px(3.0)),
            Some(CssLength::px(-4.0)),
            None,
        ))
    );
}

#[test]
fn box_model_property_families_accept_supported_values() {
    let sheet = parse_sheet(
        ".panel {
            inset: auto 1px 2%;
            top: auto;
            right: 1px;
            bottom: 2%;
            left: calc(3px + 4%);
            z-index: 7;
            box-decoration-break: slice;
            margin-top: auto;
            margin-right: 1px;
            margin-bottom: 2%;
            margin-left: calc(3px + 4%);
            padding-top: 1px;
            padding-right: 2%;
            padding-bottom: calc(3px + 4%);
            padding-left: 0;
            border: 1px solid black;
            border-top: solid;
            border-right: 1px;
            border-bottom: #fff;
            border-left: dashed black;
            border-top-width: 1px;
            border-right-width: 2px;
            border-bottom-width: 3px;
            border-left-width: 4px;
            border-top-color: black;
            border-right-color: white;
            border-bottom-color: transparent;
            border-left-color: #fff;
            border-style: none hidden dotted dashed;
            border-top-style: solid;
            border-right-style: double;
            border-bottom-style: ridge;
            border-left-style: outset;
            border-radius: 1px 2px / 3px 4px;
            border-top-left-radius: 1px;
            border-top-right-radius: 1px 2px;
            border-bottom-right-radius: 10%;
            border-bottom-left-radius: calc(1px + 2%);
            box-shadow: 1px 2px;
        }",
    )
    .unwrap();
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssProperty::Inset,
        CssProperty::Top,
        CssProperty::Right,
        CssProperty::Bottom,
        CssProperty::Left,
        CssProperty::ZIndex,
        CssProperty::BoxDecorationBreak,
        CssProperty::MarginTop,
        CssProperty::MarginRight,
        CssProperty::MarginBottom,
        CssProperty::MarginLeft,
        CssProperty::PaddingTop,
        CssProperty::PaddingRight,
        CssProperty::PaddingBottom,
        CssProperty::PaddingLeft,
        CssProperty::Border,
        CssProperty::BorderTop,
        CssProperty::BorderRight,
        CssProperty::BorderBottom,
        CssProperty::BorderLeft,
        CssProperty::BorderTopWidth,
        CssProperty::BorderRightWidth,
        CssProperty::BorderBottomWidth,
        CssProperty::BorderLeftWidth,
        CssProperty::BorderTopColor,
        CssProperty::BorderRightColor,
        CssProperty::BorderBottomColor,
        CssProperty::BorderLeftColor,
        CssProperty::BorderStyle,
        CssProperty::BorderTopStyle,
        CssProperty::BorderRightStyle,
        CssProperty::BorderBottomStyle,
        CssProperty::BorderLeftStyle,
        CssProperty::BorderRadius,
        CssProperty::BorderTopLeftRadius,
        CssProperty::BorderTopRightRadius,
        CssProperty::BorderBottomRightRadius,
        CssProperty::BorderBottomLeftRadius,
        CssProperty::BoxShadow,
    ] {
        assert!(
            declarations
                .iter()
                .any(|declaration| declaration.property() == property),
            "missing parsed declaration for {property:?}",
        );
    }
}

#[test]
fn non_negative_box_model_properties_reject_negative_lengths() {
    for input in [
        ".panel { border-radius: -1px; }",
        ".panel { padding-top: -1px; }",
        ".panel { border-width: -1px; }",
        ".panel { box-shadow: 1px 2px -3px; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
    }
}

#[test]
fn box_model_properties_reject_cross_family_values() {
    for input in [
        ".panel { padding-top: auto; }",
        ".panel { border-width: 10%; }",
        ".panel { border-style: 10px; }",
        ".panel { border-color: solid; }",
        ".panel { border-radius: auto; }",
        ".panel { box-shadow: auto; }",
        ".panel { z-index: 1.5; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(
            error.kind(),
            ErrorKind::InvalidPropertyValue(_)
                | ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidQualifiedRule(_)
                | ErrorKind::InvalidColorSyntax(_)
        ));
    }
}

#[test]
fn parses_grid_track_lists_and_template_areas() {
    assert_eq!(
        declaration_value!(
            ".panel { grid-template-columns: [main] repeat(2, minmax(10px, 1fr)) fit-content(20%); }",
            GridTemplateColumns
        ),
        CssGridTrackList::new(vec![
            CssGridTrackComponent::LineNames(CssGridLineNames::new(vec![CssCustomIdent::new(
                "main"
            )])),
            CssGridTrackComponent::Repeat(CssGridRepeat::new(
                CssGridRepeatCount::integer(2),
                CssGridTrackList::new(vec![CssGridTrackComponent::TrackSize(
                    CssGridTrackSize::minmax(
                        CssGridTrackBreadth::length(CssLength::px(10.0)),
                        CssGridTrackBreadth::try_fraction(1.0).unwrap(),
                    )
                )]),
            )),
            CssGridTrackComponent::TrackSize(CssGridTrackSize::fit_content(CssLength::percent(
                20.0
            ))),
        ])
    );
    assert_eq!(
        declaration_value!(
            ".panel { grid-template-areas: \"header header\" \"nav main\"; }",
            GridTemplateAreas
        ),
        CssGridTemplateAreas::rows(vec![
            CssGridTemplateAreaRow::new(vec![
                CssGridTemplateAreaCell::Named(CssCustomIdent::new("header")),
                CssGridTemplateAreaCell::Named(CssCustomIdent::new("header")),
            ]),
            CssGridTemplateAreaRow::new(vec![
                CssGridTemplateAreaCell::Named(CssCustomIdent::new("nav")),
                CssGridTemplateAreaCell::Named(CssCustomIdent::new("main")),
            ]),
        ])
    );
    assert_eq!(
        declaration_value!(".panel { grid-template-areas: none; }", GridTemplateAreas),
        CssGridTemplateAreas::None
    );
}

#[test]
fn parses_grid_flow_lines_and_shorthands() {
    assert_eq!(
        declaration_value!(".panel { grid-auto-flow: column dense; }", GridAutoFlow),
        CssGridAutoFlow::new(CssGridAutoFlowAxis::Column, true)
    );
    assert_eq!(
        declaration_value!(".panel { grid-row-start: span 2 main; }", GridRowStart),
        CssGridLine::span(Some(2), Some(CssCustomIdent::new("main")))
    );
    assert_eq!(
        declaration_value!(".panel { grid-column: nav / span 3; }", GridColumn),
        CssGridLineRange::new(
            CssGridLine::CustomIdent(CssCustomIdent::new("nav")),
            Some(CssGridLine::span(Some(3), None)),
        )
    );
    assert_eq!(
        declaration_value!(
            ".panel { grid-area: header / 1 / span 2 / main; }",
            GridArea
        ),
        CssGridArea::new(
            CssGridLine::CustomIdent(CssCustomIdent::new("header")),
            Some(CssGridLine::integer(1)),
            Some(CssGridLine::span(Some(2), None)),
            Some(CssGridLine::CustomIdent(CssCustomIdent::new("main"))),
        )
    );
}

#[test]
fn parses_grid_template_and_grid_shorthands() {
    assert_eq!(
        declaration_value!(
            ".panel { grid-template: 100px 1fr / repeat(2, minmax(10px, 1fr)); }",
            GridTemplate
        ),
        CssGridTemplate::RowsColumns {
            rows: CssGridTrackList::new(vec![
                CssGridTrackComponent::TrackSize(CssGridTrackSize::breadth(
                    CssGridTrackBreadth::length(CssLength::px(100.0))
                )),
                CssGridTrackComponent::TrackSize(CssGridTrackSize::breadth(
                    CssGridTrackBreadth::try_fraction(1.0).unwrap()
                )),
            ]),
            columns: Some(CssGridTrackList::new(vec![CssGridTrackComponent::Repeat(
                CssGridRepeat::new(
                    CssGridRepeatCount::integer(2),
                    CssGridTrackList::new(vec![CssGridTrackComponent::TrackSize(
                        CssGridTrackSize::minmax(
                            CssGridTrackBreadth::length(CssLength::px(10.0)),
                            CssGridTrackBreadth::try_fraction(1.0).unwrap(),
                        )
                    )]),
                )
            )])),
        }
    );
    assert_eq!(
        declaration_value!(
            ".panel { grid: auto-flow dense 12px / repeat(auto-fit, 10px); }",
            Grid
        ),
        CssGrid::AutoFlow {
            flow: CssGridAutoFlow::new(CssGridAutoFlowAxis::Row, true),
            auto_tracks: Some(CssGridTrackList::new(vec![
                CssGridTrackComponent::TrackSize(CssGridTrackSize::breadth(
                    CssGridTrackBreadth::length(CssLength::px(12.0))
                ),)
            ])),
            explicit_tracks: CssGridTrackList::new(vec![CssGridTrackComponent::Repeat(
                CssGridRepeat::new(
                    CssGridRepeatCount::AutoFit,
                    CssGridTrackList::new(vec![CssGridTrackComponent::TrackSize(
                        CssGridTrackSize::breadth(CssGridTrackBreadth::length(CssLength::px(10.0)))
                    )]),
                )
            )]),
        }
    );
}

#[test]
fn parses_order_flex_and_track_alignment() {
    assert_eq!(
        declaration_value!(".panel { order: -2; }", Order),
        CssOrder::Integer(-2)
    );
    assert_eq!(
        declaration_value!(".panel { flex: 2 0 10rem; }", Flex),
        CssFlex::Components {
            grow: CssFlexFactor::try_new(2.0).unwrap(),
            shrink: Some(CssFlexFactor::try_new(0.0).unwrap()),
            basis: Some(CssLength::dimension(10.0, CssLengthUnit::Rem)),
        }
    );
    assert_eq!(
        declaration_value!(".panel { flex: none; }", Flex),
        CssFlex::None
    );
    assert_eq!(
        declaration_value!(".panel { flex: auto; }", Flex),
        CssFlex::Auto
    );
    assert_eq!(
        declaration_value!(".panel { justify-tracks: space-evenly; }", JustifyTracks),
        CssAlignment::SpaceEvenly
    );
    assert_eq!(
        declaration_value!(".panel { align-tracks: center; }", AlignTracks),
        CssAlignment::Center
    );
}

#[test]
fn grid_and_flex_property_families_accept_supported_values() {
    let sheet = parse_sheet(
        ".panel {
            grid-template-rows: [top] 100px 1fr;
            grid-template-columns: repeat(2, minmax(10px, 1fr));
            grid-template-areas: \"header header\" \"nav main\";
            grid-template: 100px / 1fr 2fr;
            grid-auto-rows: minmax(10px, auto);
            grid-auto-columns: fit-content(20%);
            grid-auto-flow: row dense;
            grid-row-start: 1;
            grid-row-end: span 2;
            grid-column-start: nav;
            grid-column-end: auto;
            grid-row: 1 / span 2;
            grid-column: nav / main;
            grid-area: header / nav / main / 4;
            grid: auto-flow 12px / repeat(auto-fill, 10px);
            order: 2;
            flex: 1 1 auto;
            justify-tracks: space-between;
            align-tracks: stretch;
        }",
    )
    .unwrap();
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssProperty::GridTemplateRows,
        CssProperty::GridTemplateColumns,
        CssProperty::GridTemplateAreas,
        CssProperty::GridTemplate,
        CssProperty::GridAutoRows,
        CssProperty::GridAutoColumns,
        CssProperty::GridAutoFlow,
        CssProperty::GridRowStart,
        CssProperty::GridRowEnd,
        CssProperty::GridColumnStart,
        CssProperty::GridColumnEnd,
        CssProperty::GridRow,
        CssProperty::GridColumn,
        CssProperty::GridArea,
        CssProperty::Grid,
        CssProperty::Order,
        CssProperty::Flex,
        CssProperty::JustifyTracks,
        CssProperty::AlignTracks,
    ] {
        assert!(
            declarations
                .iter()
                .any(|declaration| declaration.property() == property),
            "missing parsed declaration for {property:?}",
        );
    }
}

#[test]
fn grid_and_flex_properties_reject_cross_family_values() {
    for input in [
        ".panel { order: 1.2; }",
        ".panel { grid-auto-flow: left; }",
        ".panel { grid-template-areas: \"a a\" \"a .\"; }",
        ".panel { grid-row: 1 / / 2; }",
        ".panel { flex: solid; }",
        ".panel { justify-tracks: auto; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(
            error.kind(),
            ErrorKind::InvalidPropertyValue(_)
                | ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidQualifiedRule(_)
        ));
    }
}

#[test]
fn checked_grid_constructors_reject_parser_invalid_states() {
    assert_eq!(CssCustomIdent::try_new(""), None);
    assert_eq!(CssCustomIdent::try_new("auto"), None);
    assert_eq!(
        CssCustomIdent::try_new("main"),
        Some(CssCustomIdent::new("main"))
    );
    assert_eq!(CssGridLineNames::try_new(Vec::new()), None);
    assert_eq!(CssGridTrackList::try_new(Vec::new()), None);
    assert_eq!(CssGridRepeatCount::try_integer(0), None);
    assert_eq!(
        CssGridRepeat::try_new(
            CssGridRepeatCount::integer(1),
            CssGridTrackList::new(vec![CssGridTrackComponent::TrackSize(
                CssGridTrackSize::breadth(CssGridTrackBreadth::try_fraction(1.0).unwrap())
            )])
        ),
        Some(CssGridRepeat::new(
            CssGridRepeatCount::integer(1),
            CssGridTrackList::new(vec![CssGridTrackComponent::TrackSize(
                CssGridTrackSize::breadth(CssGridTrackBreadth::try_fraction(1.0).unwrap())
            )])
        ))
    );
    assert_eq!(
        CssGridRepeat::try_new(
            CssGridRepeatCount::integer(1),
            CssGridTrackList::new(vec![])
        ),
        None
    );
    assert_eq!(CssGridTemplateAreaRow::try_new(Vec::new()), None);
    assert_eq!(CssGridTemplateAreas::try_rows(Vec::new()), None);
    assert_eq!(
        CssGridTemplateAreas::try_rows(vec![
            CssGridTemplateAreaRow::new(vec![
                CssGridTemplateAreaCell::Named(CssCustomIdent::new("a")),
                CssGridTemplateAreaCell::Named(CssCustomIdent::new("a")),
            ]),
            CssGridTemplateAreaRow::new(vec![
                CssGridTemplateAreaCell::Named(CssCustomIdent::new("a")),
                CssGridTemplateAreaCell::Empty,
            ]),
        ]),
        None
    );
    assert_eq!(CssGridLine::try_integer(0), None);
    assert_eq!(CssGridLineSpan::try_new(None, None), None);
    assert_eq!(CssGridLineSpan::try_new(Some(0), None), None);
}

#[test]
fn rejects_grid_auto_flow_shorthand_without_explicit_tracks() {
    for input in [
        ".panel { grid: auto-flow; }",
        ".panel { grid: auto-flow dense; }",
        ".panel { grid: auto-flow 12px; }",
    ] {
        let error = parse_sheet(input).expect_err(input);
        assert!(matches!(
            error.kind(),
            ErrorKind::InvalidPropertyValue(_)
                | ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidQualifiedRule(_)
        ));
    }
}

#[test]
fn invalid_parser_custom_ident_errors_keep_source_location() {
    let error = parse_sheet(".panel {\n  grid-template-columns: [auto] 1fr;\n}").unwrap_err();

    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
    assert_eq!(error.position().line().value(), 1);
    assert_ne!(error.position().column().value(), 0);
}

#[test]
fn rejects_inconsistent_grid_template_area_row_widths() {
    let error = parse_sheet(".panel { grid-template-areas: \"a a\" \"b\"; }").unwrap_err();

    assert!(matches!(error.kind(), ErrorKind::InvalidPropertyValue(_)));
}
