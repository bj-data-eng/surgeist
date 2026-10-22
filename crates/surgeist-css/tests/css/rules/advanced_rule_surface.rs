//! Supported and unsupported advanced rule forms, and the public structure of
//! the retained rule graph.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn advanced_css_surface_matrix_accepts_supported_forms() {
    let accepted = [
        ".toolbar > button[aria-expanded=true] { color: black; }",
        ".stack .item:hover { color: black; }",
        r#"@import url("theme.css") screen and (min-width: 600px);"#,
        "@media (prefers-color-scheme: dark) { .panel { color: black; } }",
        "@container sidebar (inline-size > 30rem) { .panel { color: black; } }",
        "@container scroll-state(stuck: top) { .panel { color: black; } }",
        r#"@font-face { font-family: Inter; src: url("inter.woff2") format("woff2"); }"#,
        "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }",
        ".field:has(> .icon) { color: black; }",
    ];

    for css in accepted {
        assert!(parse_sheet(css).is_clean(), "{css} should parse");
    }

    let sheet = parse_sheet(r#"@import url("theme.css") supports(display: grid) layer(theme);"#)
        .expect("clean parse report");
    let [CssRule::Import(import)] = sheet.rules() else {
        panic!("expected retained conditional import")
    };
    assert!(matches!(import.target(), CssImportTarget::Url(url) if url.as_str() == "theme.css"));
    assert!(import.layer().is_none());
    let CssSupportsConditionKind::Declaration(declaration) = import
        .supports()
        .expect("selected supports clause")
        .condition()
        .kind()
    else {
        panic!("expected declaration supports clause")
    };
    assert_eq!(declaration.authored(), Some("display: grid"));
    let [CssMediaQuery::Condition(condition)] = import.media().expect("media suffix").queries()
    else {
        panic!("expected condition-only media")
    };
    let CssMediaConditionKind::GeneralEnclosed(enclosed) = condition.kind() else {
        panic!("expected layer function as opaque media")
    };
    assert_eq!(enclosed.authored(), Some("layer(theme)"));
}

#[test]
fn advanced_css_surface_matrix_rejects_unsupported_forms() {
    let rejected = [
        (
            r#"@import url("late.css"); .panel { color: black; } @import url("later.css");"#,
            Rejection::placement("import"),
        ),
        (
            ".field:has(::before) { color: black; }",
            Rejection::selector(Some("before")),
        ),
        (
            "[svg|href] { color: black; }",
            Rejection::selector(Some("svg")),
        ),
        (".col || { color: black; }", Rejection::selector(Some("|"))),
        (
            "@container scroll-state(stuck: top) trailing { .panel { color: black; } }",
            Rejection::prelude("container", Some("trailing")),
        ),
    ];

    for (css, rejection) in rejected {
        assert_rejection(css, rejection);
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
    .expect("clean parse report");

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
    let CssMediaRangeRef::Min { value } = width.view() else {
        panic!("expected authored min-width prefix");
    };
    assert_media_length_literal(value, "600", "px");

    let descriptors = font_face.descriptors();
    assert_eq!(face_value!(descriptors, FontFamily).as_str(), "Inter");
    let [CssFontFaceSource::Url(source)] = face_value!(descriptors, Src).sources() else {
        panic!("expected one font-face URL source");
    };
    assert_eq!(source.url().as_str(), "inter.woff2");
    assert_eq!(
        source.format().and_then(CssFontFormat::recognized_format),
        Some(CssFontFormatHint::Woff2)
    );

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
        style_rule(media_nested).selectors().selectors()[0].selector(),
        &CssSelector::Class("panel".to_owned())
    );

    assert_eq!(
        container.prelude().entries()[0].name(),
        Some(&CssContainerName::try_new("sidebar").unwrap())
    );
    let CssContainerConditionKind::Feature(CssContainerFeatureQuery::InlineSize(inline_size)) =
        container.prelude().entries()[0].query().unwrap().kind()
    else {
        panic!("expected inline-size container condition");
    };
    let CssMediaRangeRef::FeatureFirst { comparison, value } = inline_size.view() else {
        panic!("inline-size range")
    };
    assert_eq!(comparison, CssQueryComparison::GreaterThan);
    let CssContainerLengthRef::Numeric(value) = value.view() else {
        panic!("typed length")
    };
    assert_eq!(value.components().serialize().unwrap().as_css(), "30rem");
    let [container_nested] = container.rules() else {
        panic!("expected one nested container rule");
    };
    let CssSelector::Complex(selector) =
        style_rule(container_nested).selectors().selectors()[0].selector()
    else {
        panic!("expected complex selector");
    };
    assert_eq!(selector.first().classes(), &["toolbar".to_owned()]);
    let [part] = selector.rest() else {
        panic!("expected one complex selector part");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_eq!(
        part.selector()
            .type_selector()
            .and_then(CssQualifiedSelectorName::local_name),
        Some("button")
    );
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
fn keyframes_and_authored_nesting_are_structurally_accessible() {
    let sheet = parse_sheet("@keyframes fade { from { opacity: 0; } to { opacity: 1; } } .card { color: black; &:hover { opacity: 0.9; } @media (min-width: 600px) { > .title { color: white; } } }").expect("clean parse report");
    let [CssRule::Keyframes(keyframes), card] = sheet.rules() else {
        panic!("expected keyframes and authored card");
    };
    assert_eq!(
        keyframes.name(),
        &CssKeyframesName::Ident(
            CssKeyframesIdent::try_new(CssCustomIdent::try_new("fade").unwrap()).unwrap()
        )
    );
    let [from, to] = keyframes.blocks() else {
        panic!("expected two keyframe blocks");
    };
    assert_eq!(from.selectors().selectors(), &[CssKeyframeSelector::From]);
    assert_eq!(
        from.declarations()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity)
    );
    assert_eq!(to.selectors().selectors(), &[CssKeyframeSelector::To]);
    assert_eq!(
        to.declarations()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Opacity)
    );
    let card = style_rule(card);
    assert_eq!(
        card.declarations()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::Color)
    );
    let [hover, media] = card.rules() else {
        panic!("expected hover then media");
    };
    let CssSelector::Compound(hover) = style_rule(hover).selectors().selectors()[0].selector()
    else {
        panic!("expected anchored hover");
    };
    assert_eq!(hover.nesting_selectors(), 1);
    assert_eq!(hover.pseudo_classes(), &[CssPseudoClass::Hover]);
    let [title] = media_rule(media).rules() else {
        panic!("expected nested title");
    };
    let [CssStyleSelector::Relative(relative)] = style_rule(title).selectors().selectors() else {
        panic!("expected relative child");
    };
    assert_eq!(relative.combinator(), CssSelectorCombinator::Child);
    assert_eq!(relative.selector(), &CssSelector::Class("title".to_owned()));
    assert_eq!(
        style_rule(title).declarations()[0]
            .known()
            .map(|known| known.property()),
        Some(CssKnownProperty::Color)
    );
}
