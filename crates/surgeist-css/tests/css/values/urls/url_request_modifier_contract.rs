//! Values 5 WD 2024-11-11 §4.1.1 defines the three request-modifier
//! productions, independently of Values 4's open identifier/function carrier.
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#request-url-modifiers
//! These assertions concern authored admission, provenance and immutable output;
//! Fetch execution, CORS policy and resource resolution belong downstream.

use crate::css::support::check_property_text as checked;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::value_assertions::background_image_url as parsed_url;
use crate::css::support::value_assertions::url_consumer_text as consumer_value;
use crate::css::support::value_assertions::{
    assert_first_property_diagnostic, assert_property_rejected,
};
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssSerializedOrigin,
    CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef,
    CssPropertyTarget,
};
use surgeist_css::rule::color_profile::{
    CssColorProfileDescriptorKind, CssColorProfileDescriptorValue,
    CssColorProfileDescriptorValueRef,
};
use surgeist_css::rule::counter_style::{
    CssCounterStyleDescriptorKind, CssCounterStyleDescriptorValue,
};
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceDescriptorValue,
    CssFontFaceSource, CssFontTechHint,
};
use surgeist_css::rule::{CssImportRule, CssImportTarget, CssRule};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::common::{
    CssIdent, CssUrl, CssUrlFunction, CssUrlModifier, CssUrlModifierFunction,
};
use surgeist_css::values::image::CssImageValue;

const REFERRER_POLICIES: [&str; 8] = [
    "no-referrer",
    "no-referrer-when-downgrade",
    "same-origin",
    "origin",
    "strict-origin",
    "origin-when-cross-origin",
    "strict-origin-when-cross-origin",
    "unsafe-url",
];

const INVALID_MODIFIERS: [&str; 27] = [
    "crossorigin()",
    "crossorigin(credentialless)",
    "crossorigin(anonymous use-credentials)",
    "crossorigin(anonymous, use-credentials)",
    "crossorigin(\"anonymous\")",
    "crossorigin(1)",
    "crossorigin([anonymous])",
    "crossorigin(f(anonymous))",
    "crossorigin(anonymous /)",
    "integrity()",
    "integrity(sha256)",
    "integrity(1)",
    "integrity(url(hash))",
    "integrity(\"one\" \"two\")",
    "integrity(\"one\",)",
    "integrity([\"hash\"])",
    "integrity(f(\"hash\"))",
    "integrity(\"hash\" /)",
    "referrerpolicy()",
    "referrerpolicy(default)",
    "referrerpolicy(\"origin\")",
    "referrerpolicy(1)",
    "referrerpolicy(origin same-origin)",
    "referrerpolicy(origin,)",
    "referrerpolicy([origin])",
    "referrerpolicy(f(origin))",
    "referrerpolicy(origin /)",
];

const CONSUMERS: [CssKnownProperty; 17] = [
    CssKnownProperty::BackgroundImage,
    CssKnownProperty::MaskImage,
    CssKnownProperty::BorderImageSource,
    CssKnownProperty::ListStyleImage,
    CssKnownProperty::Cursor,
    CssKnownProperty::Content,
    CssKnownProperty::Filter,
    CssKnownProperty::BackdropFilter,
    CssKnownProperty::ClipPath,
    CssKnownProperty::OffsetPath,
    CssKnownProperty::Background,
    CssKnownProperty::Mask,
    CssKnownProperty::BorderImage,
    CssKnownProperty::ListStyle,
    CssKnownProperty::CueBefore,
    CssKnownProperty::CueAfter,
    CssKnownProperty::Cue,
];

fn background_url(declaration: &CssDeclaration) -> &CssUrl {
    let CssKnownPropertyValueRef::BackgroundImage(images) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary background image")
    };
    let [CssImageValue::Url(url)] = images.value().images() else {
        panic!("one URL")
    };
    url
}

fn one_function(url: &CssUrl) -> &CssUrlModifierFunction {
    let [CssUrlModifier::Function(function)] = url.modifiers() else {
        panic!("one function")
    };
    function
}

#[test]
fn all_published_arguments_are_admitted_in_both_quoted_url_forms() {
    let mut modifiers = vec![
        "crossorigin(anonymous)".to_owned(),
        "crossorigin(use-credentials)".to_owned(),
        "integrity(\"\")".to_owned(),
        "integrity(\"sha256-not-evaluated\")".to_owned(),
    ];
    modifiers.extend(REFERRER_POLICIES.map(|policy| format!("referrerpolicy({policy})")));
    for function in ["url", "src"] {
        for modifier in &modifiers {
            let text = format!("{function}(\"#asset\" {modifier})");
            let url = parsed_url(&text);
            assert_eq!(url.as_str(), "#asset");
            assert!(url.is_local_url());
            assert_eq!(
                url.function(),
                if function == "url" {
                    CssUrlFunction::Url
                } else {
                    CssUrlFunction::Src
                }
            );
            assert_eq!(url.serialize_specified(Default::default()).unwrap(), text);
            let checked = checked(CssKnownProperty::BackgroundImage, &text).unwrap();
            assert_eq!(background_url(&checked), &url);
        }
    }
}

#[test]
fn known_request_modifiers_share_admission_across_all_property_consumers() {
    for property in CONSUMERS {
        for function in ["url", "src"] {
            let url = format!(
                "{function}(\"asset\" crossorigin(anonymous) integrity(\"hash\") referrerpolicy(origin))"
            );
            let value = consumer_value(property, &url);
            let source = format!("{}:{value}!important", property.canonical_name());
            let report = CssParser::default().parse_block_contents(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 1, "{source}");
            assert_eq!(report.syntax()[0].known().unwrap().property(), property);
            assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
            assert!(
                CssParser::default()
                    .parse_block_contents(&source)
                    .into_validation_result()
                    .is_ok(),
                "{source}"
            );
            assert!(checked(property, &value).is_ok(), "{source}");
        }
    }
}

#[test]
fn missing_extra_wrong_category_and_malformed_known_arguments_drop_whole_declarations() {
    for modifier in INVALID_MODIFIERS {
        for function in ["url", "src"] {
            let source = format!(
                "color:red;background-image:{function}(\"asset\" {modifier})!important;width:2px"
            );
            let report = CssParser::default().parse_block_contents(&source);
            assert_eq!(
                report.syntax().len(),
                2,
                "recognized modifier must fail: {source}"
            );
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            assert_eq!(
                report.syntax()[1].known().unwrap().property(),
                CssKnownProperty::Width
            );
            assert_eq!(report.diagnostics().len(), 1, "{source}");
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropDeclaration
            );
            assert_eq!(
                report.diagnostics()[0].error().code(),
                CssErrorCode::InvalidPropertyValue
            );
            assert_eq!(
                CssParser::default()
                    .parse_block_contents(&source)
                    .into_validation_result()
                    .unwrap_err()
                    .diagnostics(),
                report.diagnostics(),
                "{source}"
            );
        }
    }
}

#[test]
fn every_property_consumer_rejects_invalid_recognized_modifier_arguments() {
    for property in CONSUMERS {
        for modifier in [
            "crossorigin(credentialless)",
            "integrity(hash)",
            "referrerpolicy(default)",
        ] {
            let value = consumer_value(property, &format!("src(\"asset\" {modifier})"));
            let source = format!("color:red;{}:{value};width:2px", property.canonical_name());
            let report = CssParser::default().parse_block_contents(&source);
            assert_eq!(report.syntax().len(), 2, "{source}");
            // Values 5 request URL modifiers: the invalid modifier function is
            // responsible in every consumer.
            crate::css::support::value_assertions::assert_first_property_rejection(
                report.diagnostics(),
                property,
            );
            let function = &modifier[..=modifier.find('(').unwrap()];
            assert_property_rejected(checked(property, &value), property, Some(function));
        }
    }
}

#[test]
fn checked_input_rejects_every_invalid_known_argument_without_losing_origin() {
    for modifier in INVALID_MODIFIERS {
        let text = format!("src(\"asset\" {modifier})");
        let error = checked(CssKnownProperty::BackgroundImage, &text).unwrap_err();
        assert!(error.code().is_some(), "{text}: {error:?}");
        let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
            panic!("original parsed token origin: {text}: {error:?}")
        };
        assert_eq!(origin.source().as_str(), text);
    }
}

#[test]
fn nested_image_and_generated_content_consumers_admit_valid_known_modifiers() {
    for (property, pattern) in [
        (CssKnownProperty::BackgroundImage, "light-dark(VALUE, none)"),
        (CssKnownProperty::Cursor, "image-set(VALUE 1x), auto"),
        (CssKnownProperty::Content, "target-text(VALUE)"),
        (CssKnownProperty::Content, "target-counter(VALUE, page)"),
        (
            CssKnownProperty::Content,
            "target-counters(VALUE, page, \".\")",
        ),
        (CssKnownProperty::ListStyleType, "symbols(VALUE)"),
    ] {
        let valid = pattern.replace("VALUE", "src(\"asset\" integrity(\"hash\"))");
        assert!(
            checked(property, &valid).is_ok(),
            "valid importing grammar: {valid}"
        );
    }
}

#[test]
fn nested_image_and_generated_content_consumers_reject_invalid_known_modifiers() {
    for (property, pattern) in [
        (CssKnownProperty::BackgroundImage, "light-dark(VALUE, none)"),
        (CssKnownProperty::Cursor, "image-set(VALUE 1x), auto"),
        (CssKnownProperty::Content, "target-text(VALUE)"),
        (CssKnownProperty::Content, "target-counter(VALUE, page)"),
        (
            CssKnownProperty::Content,
            "target-counters(VALUE, page, \".\")",
        ),
        (CssKnownProperty::ListStyleType, "symbols(VALUE)"),
    ] {
        let invalid = pattern.replace("VALUE", "src(\"asset\" integrity(hash))");
        assert_property_rejected(checked(property, &invalid), property, Some("integrity("));
    }
}

#[test]
fn imports_reject_invalid_known_modifiers_atomically_and_keep_following_rules() {
    for modifier in INVALID_MODIFIERS {
        let source = format!(
            "@import src(\"theme.css\" {modifier}) layer(theme) screen;.after{{color:red}}"
        );
        let report = CssParser::default().parse_sheet(&source);
        assert!(
            matches!(report.syntax().rules(), [CssRule::Style(_)]),
            "{source}: {:?}",
            report.syntax()
        );
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        assert_eq!(
            CssParser::default()
                .parse_sheet(&source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
        assert!(
            CssImportRule::try_from_components(
                CssComponentValues::parse(
                    &CssParser::default(),
                    &source[..source.find(".after").unwrap()]
                )
                .unwrap(),
                &CssNamespaceContext::default(),
                Default::default(),
            )
            .as_ref()
            .is_err_and(|error| error.is_invalid_rule_grammar())
        );
    }
}

#[test]
fn font_sources_reject_invalid_known_modifiers_at_the_source_item_boundary() {
    for modifier in INVALID_MODIFIERS {
        let value = format!("src(\"bad.woff\" {modifier}),url(\"good.woff\" integrity(\"hash\"))");
        let report = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            &value,
            CssFontFaceDescriptorKind::Src,
        );
        assert!(!report.is_clean(), "{value}");
        let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(sources)) =
            report.syntax().as_ref().unwrap()
        else {
            panic!("retained good source: {value}")
        };
        let [CssFontFaceSource::Url(good)] = sources.sources() else {
            panic!("only valid source: {value}")
        };
        assert_eq!(good.url().as_str(), "good.woff");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropFontSourceListItem
        );
        let error = CssAuthoredFontFaceDescriptorValue::try_from_components(
            CssFontFaceDescriptorKind::Src,
            CssComponentValues::parse(&CssParser::default(), &value).unwrap(),
            Default::default(),
        )
        .unwrap_err();
        assert!(error.diagnostic().is_some(), "{value}: {error:?}");
    }
}

#[test]
fn color_profile_src_uses_the_shared_known_modifier_grammar() {
    let valid = "src(\"profile.icc\" crossorigin(use-credentials) integrity(\"hash\") referrerpolicy(same-origin))";
    assert!(
        CssColorProfileDescriptorValue::parse(
            &CssParser::default(),
            valid,
            CssColorProfileDescriptorKind::Src
        )
        .is_clean()
    );
    for modifier in [
        "crossorigin(credentialless)",
        "integrity(hash)",
        "referrerpolicy(default)",
    ] {
        let text = format!("src(\"profile.icc\" {modifier})");
        let report = CssColorProfileDescriptorValue::parse(
            &CssParser::default(),
            &text,
            CssColorProfileDescriptorKind::Src,
        );
        assert!(report.syntax().is_none(), "{text}");
        assert!(!report.is_clean());
    }
}

#[test]
fn known_names_and_keywords_decode_css_escapes_and_fold_ascii_case() {
    for text in [
        r##"SRC("\23 asset" CrOsSoRiGiN(AnOnYmOuS))"##,
        r##"url("#asset" c\72 ossorigin(\61 nonymous))"##,
        r##"src("#asset" REFERRERPOLICY(STRICT-ORIGIN))"##,
        r##"url("#asset" INTEGRITY("sha\32 56"))"##,
    ] {
        let url = parsed_url(text);
        assert_eq!(url.as_str(), "#asset");
        assert!(url.is_local_url());
        assert!(checked(CssKnownProperty::BackgroundImage, text).is_ok());
    }
    for text in [
        r##"src("asset" CrOsSoRiGiN(wrong))"##,
        r##"url("asset" integ\72 ity(hash))"##,
    ] {
        let opening = &text[text.find("\" ").unwrap() + 2..=text.rfind("(").unwrap()];
        assert_property_rejected(
            checked(CssKnownProperty::BackgroundImage, text),
            CssKnownProperty::BackgroundImage,
            Some(opening),
        );
    }
}

#[test]
fn unknown_extensions_bare_identifiers_and_repetition_remain_ordered_authored_syntax() {
    let text = r##"src("asset" crossorigin integrity referrerpolicy future() future([x] f(1,"s")) crossorigin(anonymous) crossorigin(use-credentials) integrity("one") integrity("two"))"##;
    let url = parsed_url(text);
    assert_eq!(url.modifiers().len(), 9);
    assert_eq!(url.serialize_specified(Default::default()).unwrap(), text);
    assert_eq!(
        one_function(&parsed_url(r##"url("asset" integrity-extra(hash))"##)).name(),
        "integrity-extra"
    );
    assert_eq!(
        parsed_url(&url.serialize_specified(Default::default()).unwrap()),
        url
    );
}

#[test]
fn modifier_arguments_keep_decoded_values_order_and_original_token_spans() {
    let source = r##"background-image:SRC("\23 asset" integrity("sha\32 56") crossorigin(anonymous) referrerpolicy(origin))!important"##;
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    assert_eq!(declaration.importance(), CssImportance::Important);
    let url = background_url(declaration);
    assert_eq!(url.function(), CssUrlFunction::Src);
    assert_eq!(url.as_str(), "#asset");
    assert!(url.is_local_url());
    for (modifier, name, argument, decoded) in url
        .modifiers()
        .iter()
        .zip([
            ("integrity", r##""sha\32 56""##, "sha256"),
            ("crossorigin", "anonymous", "anonymous"),
            ("referrerpolicy", "origin", "origin"),
        ])
        .map(|(modifier, (name, argument, decoded))| (modifier, name, argument, decoded))
    {
        let CssUrlModifier::Function(function) = modifier else {
            panic!("functional modifier")
        };
        assert_eq!(function.name(), name);
        assert_eq!(function.arguments().as_css(), argument);
        let [child] = function.argument_components().items() else {
            panic!("one argument")
        };
        match child.view() {
            CssComponentValueRef::Token(
                CssValueTokenRef::String(value) | CssValueTokenRef::Ident(value),
            ) => assert_eq!(value, decoded),
            other => panic!("argument token: {other:?}"),
        }
        let CssValueOrigin::Parsed(origin) = child.origin() else {
            panic!("authored origin")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            &source[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
            argument
        );
    }
}

#[test]
fn checked_known_arguments_preserve_programmatic_origins() {
    let arguments = CssComponentValues::try_new(
        vec![CssComponentValue::try_string("hash").unwrap()],
        Default::default(),
    )
    .unwrap();
    let modifier =
        CssUrlModifierFunction::try_new(CssIdent::try_new("integrity").unwrap(), arguments)
            .unwrap();
    assert!(matches!(
        modifier.argument_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let url = CssUrl::from_parts(
        CssUrlFunction::Src,
        "#asset",
        vec![CssUrlModifier::Function(modifier)],
    );
    assert_eq!(
        url.serialize_specified(Default::default()).unwrap(),
        "src(\"#asset\" integrity(\"hash\"))"
    );
}

#[test]
fn strict_pending_reentry_rejects_bad_known_arguments_and_is_retryable() {
    for property in [
        CssKnownProperty::BackgroundImage,
        CssKnownProperty::Background,
        CssKnownProperty::MaskImage,
    ] {
        let authored = checked(property, "src(var(--asset))").unwrap();
        let CssExpansion::Pending(pending) = authored.expand().unwrap() else {
            panic!("pending URL")
        };
        for modifier in INVALID_MODIFIERS {
            let replacement = CssComponentValues::parse(
                &CssParser::default(),
                format!("src(\"asset\" {modifier})"),
            )
            .unwrap();
            assert!(
                pending
                    .reenter(replacement)
                    .unwrap_err()
                    .replacement_error()
                    .is_some(),
                "{modifier}"
            );
            assert!(pending.source().same_occurrence(&authored));
        }
        assert!(
            pending
                .reenter(
                    CssComponentValues::parse(
                        &CssParser::default(),
                        "src(\"asset\" integrity(var(--hash)))"
                    )
                    .unwrap()
                )
                .unwrap_err()
                .is_residual_substitution()
        );
        let replacement =
            CssComponentValues::parse(&CssParser::default(), "src(\"asset\" integrity(\"hash\"))")
                .unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed contribution")
        };
        for contribution in values.items() {
            assert!(contribution.source().same_occurrence(&authored));
            assert_eq!(contribution.source().importance(), CssImportance::Important);
            assert_eq!(contribution.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn known_argument_comments_and_whitespace_preserve_the_single_significant_token() {
    for modifier in [
        "crossorigin( /**/ anonymous /**/ )",
        "integrity( /**/ \"hash\" /**/ )",
        "referrerpolicy( /**/ origin /**/ )",
    ] {
        let text = format!("url(\"asset\" {modifier})");
        let url = parsed_url(&text);
        assert!(checked(CssKnownProperty::BackgroundImage, &text).is_ok());
        assert_eq!(url.serialize_specified(Default::default()).unwrap(), text);
    }
    for modifier in [
        "crossorigin(anony/**/mous)",
        "integrity(\"a\"/**/\"b\")",
        "referrerpolicy(ori/**/gin)",
    ] {
        assert_property_rejected(
            checked(
                CssKnownProperty::BackgroundImage,
                &format!("url(\"asset\" {modifier})"),
            ),
            CssKnownProperty::BackgroundImage,
            Some(&modifier[..=modifier.find('(').unwrap()]),
        );
    }
}

#[test]
fn known_modifiers_share_cumulative_specified_output_budgets_atomically() {
    let text = "src(\"asset\" crossorigin(anonymous) integrity(\"hash\") referrerpolicy(origin))";
    let url = parsed_url(text);
    let before = url.clone();
    // One URL, one target, and three pairs of modifier/function-argument nodes.
    assert_eq!(
        url.serialize_specified(CssSerializationLimits::new(8, 8, text.len()))
            .unwrap(),
        text
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(7, 8, text.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(8, 7, text.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(8, 8, text.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&url.serialize_specified(limits).unwrap_err()),
            kind
        );
        assert_eq!(url, before);
    }
    assert_eq!(url.serialize_specified(Default::default()).unwrap(), text);
    assert_eq!(parsed_url(text), url);
}

#[test]
fn normalization_preserves_request_modifiers_and_has_atomic_cumulative_limits() {
    let source = ".x{background-image:src(\"#asset\" integrity(\"hash\"));mask-image:url(\"asset\" referrerpolicy(origin))}";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean());
    let normalized = report
        .syntax()
        .normalize(CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap())
        .unwrap();
    let [
        CssNormalizedItem::Rule(_),
        CssNormalizedItem::Declaration(first),
        CssNormalizedItem::Declaration(second),
    ] = normalized.items()
    else {
        panic!("two ordered declarations")
    };
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style")
    };
    assert!(first.source().same_occurrence(&style.declarations()[0]));
    assert!(second.source().same_occurrence(&style.declarations()[1]));
    assert_eq!(
        background_url(first.source())
            .serialize_specified(Default::default())
            .unwrap(),
        "src(\"#asset\" integrity(\"hash\"))"
    );
    let error = report
        .syntax()
        .normalize(CssNormalizationLimits::try_new(0, 1, 2, 1).unwrap())
        .unwrap_err();
    assert!(error.exhausted_limit() == Some((CssNormalizationResource::Contributions, 1)));
    assert_eq!(report.syntax().rules().len(), 1);
    assert!(report.syntax().normalize(Default::default()).is_ok());
}

#[test]
fn structural_recovery_and_checked_component_failures_remain_typed() {
    for modifier in ["integrity([)", "crossorigin([)", "referrerpolicy({)"] {
        let source = format!("background-image:src(\"asset\" {modifier});width:2px");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(
            !report.syntax().iter().any(|d| d
                .known()
                .is_some_and(|k| k.property() == CssKnownProperty::BackgroundImage)),
            "{source}"
        );
    }
    assert!(
        CssComponentValue::try_token("a b")
            .unwrap_err()
            .is_invalid_token()
    );
    let recovered = CssComponentValues::parse(
        &CssParser::default(),
        "src(\"asset\" integrity(\"hash\")) f(",
    )
    .unwrap();
    let error = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage)),
        recovered,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert!(error.code().is_some());
}

#[test]
fn integrity_requires_a_string_argument() {
    for modifier in [
        "integrity(hash)",
        "integrity(1)",
        "integrity(\"one\" \"two\")",
    ] {
        let source = format!("background-image:url(\"asset\" {modifier})");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(
            report.syntax().is_empty(),
            "known grammar must reject {modifier}"
        );
        assert_first_property_diagnostic(
            report.diagnostics(),
            &source,
            CssKnownProperty::BackgroundImage,
            &modifier[..=modifier.find('(').unwrap()],
        );
    }
}

#[test]
fn checked_programmatic_url_components_reject_invalid_known_function_arguments() {
    for (name, argument) in [
        ("crossorigin", "wrong"),
        ("integrity", "hash"),
        ("referrerpolicy", "default"),
    ] {
        let arguments = CssComponentValues::try_new(
            vec![CssComponentValue::try_ident(argument).unwrap()],
            Default::default(),
        )
        .unwrap();
        let modifier = CssComponentValue::try_function(name, arguments).unwrap();
        let url = CssComponentValue::try_function(
            "src",
            CssComponentValues::try_new(
                vec![
                    CssComponentValue::try_string("asset").unwrap(),
                    CssComponentValue::try_token(" ").unwrap(),
                    modifier,
                ],
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let error = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(
                CssKnownProperty::BackgroundImage,
            )),
            CssComponentValues::try_new(vec![url], Default::default()).unwrap(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(error.code().is_some());
        assert!(matches!(
            error.origin(),
            CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
        ));
    }
}

#[test]
fn valid_import_font_and_profile_fronts_preserve_the_ordered_shared_url() {
    for function in ["url", "src"] {
        let text = format!(
            "{function}(\"#asset\" referrerpolicy(origin) integrity(\"hash\") crossorigin(use-credentials))"
        );
        let expected = parsed_url(&text);
        let source = format!("@import {text} layer(theme) screen;");
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Import(import)] = report.syntax().rules() else {
            panic!("one import")
        };
        let CssImportTarget::Url(target) = import.target() else {
            panic!("URL import")
        };
        assert_eq!(target.url(), &expected);
        assert!(import.layer().is_some());
        assert!(import.media().is_some());
        let checked_import = CssImportRule::try_from_components(
            CssComponentValues::parse(&CssParser::default(), &source).unwrap(),
            &CssNamespaceContext::default(),
            Default::default(),
        )
        .unwrap();
        let CssImportTarget::Url(target) = checked_import.target() else {
            panic!("checked URL import")
        };
        assert_eq!(target.url(), &expected);
        let font = CssAuthoredFontFaceDescriptorValue::try_from_components(
            CssFontFaceDescriptorKind::Src,
            CssComponentValues::parse(
                &CssParser::default(),
                format!("{text} format(\"woff2\") tech(variations)"),
            )
            .unwrap(),
            Default::default(),
        )
        .unwrap();
        let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(sources)) =
            font
        else {
            panic!("font sources")
        };
        let [CssFontFaceSource::Url(font)] = sources.sources() else {
            panic!("one font URL")
        };
        assert_eq!(font.url(), &expected);
        assert_eq!(font.tech(), &[CssFontTechHint::Variations]);
        let profile = CssColorProfileDescriptorValue::try_new(
            CssColorProfileDescriptorKind::Src,
            CssComponentValues::parse(&CssParser::default(), &text).unwrap(),
            Default::default(),
        )
        .unwrap();
        let CssColorProfileDescriptorValueRef::Src(url) = profile.view() else {
            panic!("profile URL")
        };
        assert_eq!(url, &expected);
        for modifier in url.modifiers() {
            let CssUrlModifier::Function(function) = modifier else {
                panic!("function")
            };
            let CssValueOrigin::Parsed(origin) = function.argument_components().items()[0].origin()
            else {
                panic!("original token")
            };
            assert_eq!(origin.source().as_str(), text);
        }
    }
}

#[test]
fn profile_pending_reentry_rejects_invalid_known_modifiers_and_allows_retry() {
    let pending = CssColorProfileDescriptorValue::try_new(
        CssColorProfileDescriptorKind::Src,
        CssComponentValues::parse(&CssParser::default(), "env(profile)").unwrap(),
        Default::default(),
    )
    .unwrap();
    for modifier in INVALID_MODIFIERS {
        let error = pending
            .reparse_after_substitution(
                CssComponentValues::parse(
                    &CssParser::default(),
                    format!("url(\"profile\" {modifier})"),
                )
                .unwrap(),
                Default::default(),
            )
            .unwrap_err();
        assert!(error.diagnostic().is_some(), "{modifier}: {error:?}");
        assert!(matches!(
            pending.view(),
            CssColorProfileDescriptorValueRef::Pending(_)
        ));
    }
    assert!(
        pending
            .reparse_after_substitution(
                CssComponentValues::parse(
                    &CssParser::default(),
                    "url(\"profile\" integrity(\"hash\"))"
                )
                .unwrap(),
                Default::default()
            )
            .is_ok()
    );
}

#[test]
fn counter_symbol_descriptor_consumers_use_shared_known_argument_validation() {
    for (kind, prefix) in [
        (CssCounterStyleDescriptorKind::Negative, ""),
        (CssCounterStyleDescriptorKind::Prefix, ""),
        (CssCounterStyleDescriptorKind::Suffix, ""),
        (CssCounterStyleDescriptorKind::Pad, "2 "),
        (CssCounterStyleDescriptorKind::Symbols, ""),
        (CssCounterStyleDescriptorKind::AdditiveSymbols, "1 "),
    ] {
        let valid = format!("{prefix}src(\"asset\" integrity(\"hash\"))");
        assert!(
            CssCounterStyleDescriptorValue::parse(&CssParser::default(), &valid, kind).is_clean(),
            "{}:{valid}",
            kind.as_str()
        );
        assert!(
            CssCounterStyleDescriptorValue::try_from_components(
                kind,
                CssComponentValues::parse(&CssParser::default(), &valid).unwrap(),
                Default::default(),
            )
            .is_ok()
        );
        let invalid = format!("{prefix}src(\"asset\" integrity(hash))");
        let report = CssCounterStyleDescriptorValue::parse(&CssParser::default(), &invalid, kind);
        assert!(!report.is_clean(), "{}:{invalid}", kind.as_str());
        assert!(report.syntax().is_none());
        let error = CssCounterStyleDescriptorValue::try_from_components(
            kind,
            CssComponentValues::parse(&CssParser::default(), &invalid).unwrap(),
            Default::default(),
        )
        .unwrap_err();
        assert!(
            error.diagnostic().is_some(),
            "{}:{invalid}: {error:?}",
            kind.as_str()
        );
    }
}

#[test]
fn referrerpolicy_rejects_keywords_outside_the_eight_selected_alternatives() {
    let report = CssParser::default()
        .parse_block_contents("background-image:url(\"asset\" referrerpolicy(default))");
    assert!(
        report.syntax().is_empty(),
        "unknown referrer policy must reject"
    );
    assert_first_property_diagnostic(
        report.diagnostics(),
        "background-image:url(\"asset\" referrerpolicy(default))",
        CssKnownProperty::BackgroundImage,
        "referrerpolicy(",
    );
}
