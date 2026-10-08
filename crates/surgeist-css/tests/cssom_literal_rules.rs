#![forbid(unsafe_code)]
//! Independent literal CSSOM composition expectations; compact output stays separate.
use std::error::Error;
use surgeist_css::{
    CssDeclarationBlockError, CssDeclarationBlockErrorKind, CssImportance, CssKnownProperty,
    CssNamespaceContext, CssParserContext, CssParserMode, CssRecoveryAction, CssRule,
    CssRuleCssomFormat, CssRuleCssomKind, CssRuleCssomSerializationError,
    CssRuleCssomSerializationErrorKind as RuleError,
    CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, parse_rule, parse_selector, parse_sheet,
};

fn rule(source: &str) -> CssRule {
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone().expect("one admitted rule")
}

fn exact(source: &str, expected: &str) {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let before = report.clone();
    assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    assert_eq!(report, before);
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        expected,
        "unchanged retry"
    );
}

fn limits(source: &str, expected: &str, inputs: usize, projections: usize) {
    let value = rule(source);
    let before = value.clone();
    let adequate = Limits::new(inputs, projections, expected.len());
    assert_eq!(
        value.serialize_cssom_with_limits(adequate).unwrap(),
        expected
    );
    for (bound, kind) in [
        (
            Limits::new(inputs - 1, projections, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(inputs, projections - 1, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (
            Limits::new(inputs, projections, expected.len() - 1),
            Resource::ByteLimit,
        ),
        (
            Limits::new(0, projections, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(inputs, 0, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(inputs, projections, 0), Resource::ByteLimit),
    ] {
        let failure = value.serialize_cssom_with_limits(bound).unwrap_err();
        assert_eq!(failure.kind(), RuleError::Resource(kind));
        assert_eq!(failure.rule_path(), &[]);
        assert_eq!(value, before);
        assert_eq!(
            value.serialize_cssom_with_limits(adequate).unwrap(),
            expected
        );
    }
}

#[test]
fn one_rule_boundary_rejects_extra_input_and_whole_invalid_selectors() {
    for source in ["", "a{} b{}", "a{} ;", "a{} }", "@unknown x;", ".a, > b{}"] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
    }
    assert!(
        parse_selector(".a, > b", &CssNamespaceContext::default())
            .syntax()
            .is_none()
    );
    let report = parse_rule("a{} b{}", &CssNamespaceContext::default());
    let [diagnostic] = report.diagnostics() else {
        panic!("one outer rejection")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 7);
}

#[test]
fn local_declaration_recovery_preserves_rule_diagnostics_and_real_eof() {
    let report = parse_rule(".a{width:red;opacity:0}", &CssNamespaceContext::default());
    assert!(!report.is_clean());
    let [diagnostic] = report.diagnostics() else {
        panic!("local declaration rejection")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let value = report.syntax().as_ref().unwrap();
    let before = report.clone();
    assert_eq!(value.serialize_cssom().unwrap(), ".a { opacity: 0; }");
    assert_eq!(report, before);
    let source = ".a{opacity:0";
    let eof = parse_rule(source, &CssNamespaceContext::default());
    assert!(!eof.is_clean());
    assert!(
        eof.diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    assert!(
        eof.diagnostics()
            .iter()
            .any(|d| d.error().position().byte_offset().value() == source.len())
    );
    assert_eq!(
        eof.syntax().as_ref().unwrap().serialize_cssom().unwrap(),
        ".a { opacity: 0; }"
    );
}

#[test]
fn parser_context_and_original_unicode_coordinates_survive_literal_output() {
    let source = "/*😀*/\n.a{opacity:0}";
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.is_clean());
    let CssRule::Style(style) = report.syntax().as_ref().unwrap() else {
        panic!("style")
    };
    let original = &style.declarations()[0];
    let name = original.parsed_name().unwrap();
    let value = original.parsed_value().unwrap();
    assert_eq!(
        (
            name.span().start().byte_offset().value(),
            name.span().end().byte_offset().value()
        ),
        (12, 19)
    );
    assert_eq!(
        (
            value.span().start().byte_offset().value(),
            value.span().end().byte_offset().value()
        ),
        (20, 21)
    );
    assert_eq!(
        (
            name.span().start().line().value(),
            name.span().start().column().value()
        ),
        (1, 3)
    );
    assert_eq!(name.source().as_str(), source);
    let saved = original.clone();
    assert_eq!(
        report.syntax().as_ref().unwrap().serialize_cssom().unwrap(),
        ".a { opacity: 0; }"
    );
    assert!(original.same_occurrence(&saved));
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let report = quirks.parse_rule(".a{width:1}", &CssNamespaceContext::default());
    assert!(report.is_clean(), "{report:?}");
    let CssRule::Style(style) = report.syntax().as_ref().unwrap() else {
        panic!("style")
    };
    assert_eq!(style.declarations()[0].parser_context(), quirks);
    assert_eq!(
        report.syntax().as_ref().unwrap().serialize_cssom().unwrap(),
        ".a { width: 1px; }"
    );
    let standards = parse_rule(".a{width:1}", &CssNamespaceContext::default());
    assert!(!standards.is_clean());
    assert_eq!(
        standards
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_cssom()
            .unwrap(),
        ".a { }"
    );
}

#[test]
fn selector_provider_composes_groups_namespaces_pseudos_anb_and_escapes() {
    for (source, expected) in [
        (
            r#"a#X.c[data-x='v' i]>b+c~d e{}"#,
            r#"a#X.c[data-x="v" i] > b + c ~ d e { }"#,
        ),
        (
            ":is(.a,#b):not(.c):where(.d):has(>.x,+.y){}",
            ":is(.a, #b):not(.c):where(.d):has(> .x, + .y) { }",
        ),
        (
            r#":lang(en,'fr-CA'):dir(rtl):nth-child(odd of .a,#b):nth-of-type(-n + 3)::before{}"#,
            r#":lang("en", "fr-CA"):dir(rtl):nth-child(2n+1 of .a, #b):nth-of-type(-n+3)::before { }"#,
        ),
        (
            r#".日本\ x,#\31 23,a\:b{}"#,
            r#".日本\ x, #\31 23, a\:b { }"#,
        ),
        (
            r#"@namespace n 'urn:x';n|leaf[n|key='v']{}*|*[|key]{}|leaf{}"#,
            "@namespace n url(\"urn:x\");\nn|leaf[n|key=\"v\"] { }\n*|*[|key] { }\n|leaf { }",
        ),
    ] {
        exact(source, expected);
    }
}

#[test]
fn style_block_winners_shorthand_priority_and_custom_case_have_exact_literals() {
    for (source, expected, compact) in [
        (".a{}", ".a { }", ".a { }"),
        (
            ".a{color:red;color:blue}",
            ".a { color: blue; }",
            ".a { color: red; color: blue; }",
        ),
        (
            ".a{margin:1px 2px 3px 4px;margin-left:7px}",
            ".a { margin: 1px 2px 3px 7px; }",
            ".a { margin: 1px 2px 3px 4px; margin-left: 7px; }",
        ),
        (
            ".a{opacity:0;opacity:1!important;opacity:.5}",
            ".a { opacity: 1 !important; }",
            ".a { opacity: 0; opacity: 1 !important; opacity: 0.5; }",
        ),
        (
            ".a{--Case:A/**/ B;--case:C;--Case:D}",
            ".a { --case: C; --Case: D; }",
            ".a { --Case: A/**/ B; --case: C; --Case: D; }",
        ),
    ] {
        let value = rule(source);
        let before = value.clone();
        assert_eq!(value.serialize_cssom().unwrap(), expected);
        assert_eq!(value.to_specified_css().unwrap(), compact);
        assert_eq!(value, before);
    }
}

#[test]
fn nested_style_declaration_runs_normalize_only_at_their_actual_child_positions() {
    let value = rule(".a{color:red;& .b{margin:1px}color:blue;color:green;& .c{}color:yellow}");
    let before = value.clone();
    let expected = ".a {\n  color: red;\n  & .b { margin: 1px; }\n  color: green;\n  & .c { }\n  color: yellow;\n}";
    assert_eq!(value.serialize_cssom().unwrap(), expected);
    let CssRule::Style(style) = &value else {
        panic!("style")
    };
    let [
        CssRule::Style(_),
        run @ CssRule::NestedDeclarations(_),
        CssRule::Style(_),
        CssRule::NestedDeclarations(_),
    ] = style.rules()
    else {
        panic!("separate runs")
    };
    assert_eq!(run.serialize_cssom().unwrap(), "color: green;");
    assert_eq!(
        run.to_specified_css().unwrap(),
        "color: blue; color: green;"
    );
    assert_eq!(
        value.to_specified_css().unwrap(),
        ".a { color: red; & .b { margin: 1px; } color: blue; color: green; & .c { } color: yellow; }"
    );
    assert_eq!(value, before);
}

#[test]
fn media_uses_literal_lfs_empty_two_lfs_and_prefix_only_multilevel_indentation() {
    for (source, expected, compact) in [
        (
            "@media screen{}",
            "@media screen {\n\n}",
            "@media screen { }",
        ),
        ("@media{}", "@media  {\n\n}", "@media { }"),
        (
            "@media PRINT,ONLY SCREEN{.a{opacity:0}.b{}}",
            "@media print, only screen {\n  .a { opacity: 0; }\n  .b { }\n}",
            "@media print, only screen { .a { opacity: 0; } .b { } }",
        ),
        (
            "@media screen{@media print{.a{}}}",
            "@media screen {\n  @media print {\n  .a { }\n}\n}",
            "@media screen { @media print { .a { } } }",
        ),
        (
            "@media screen{.a{& .b{}}}",
            "@media screen {\n  .a {\n  & .b { }\n}\n}",
            "@media screen { .a { & .b { } } }",
        ),
    ] {
        let value = rule(source);
        assert_eq!(value.serialize_cssom().unwrap(), expected);
        assert_eq!(value.to_specified_css().unwrap(), compact);
    }
}

#[test]
fn import_retains_selected_url_layer_supports_media_and_namespace_escape() {
    for (source, expected) in [
        ("@import 'theme.css';", "@import 'theme.css';"),
        (
            "@import url(theme.css) layer(theme) supports(display:grid) PRINT;",
            "@import url(theme.css) layer(theme) supports(display:grid) print;",
        ),
        (
            "@import 'x' layer supports(display:grid) print;",
            "@import 'x' layer supports(display:grid) print;",
        ),
        (
            r#"@namespace \31  'urn:x';"#,
            r#"@namespace \31  url("urn:x");"#,
        ),
    ] {
        let value = rule(source);
        let before = value.clone();
        assert_eq!(value.serialize_cssom().unwrap(), expected);
        assert_eq!(value.to_specified_css().unwrap(), expected);
        assert_eq!(value, before);
    }
}

#[test]
fn font_face_uses_effective_modern_order_alias_ranges_and_present_only_values() {
    exact("@font-face{}", "@font-face { }");
    exact(
        "@font-face{font-weight:500}",
        "@font-face { font-weight: 500; }",
    );
    exact(
        "@font-face{font-weight:300;font-stretch:condensed;font-width:expanded;font-weight:400 400;font-display:swap;font-family:Demo}",
        "@font-face { font-family: Demo; font-width: expanded; font-weight: 400; font-display: swap; }",
    );
    exact(
        "@font-face{font-style:oblique -10deg 20deg;font-weight:300 600;font-stretch:75% 125%}",
        "@font-face { font-width: 75% 125%; font-weight: 300 600; font-style: oblique -10deg 20deg; }",
    );
    exact(
        r#"@font-face{line-gap-override:normal;descent-override:normal;ascent-override:normal;font-language-override:normal;font-named-instance:auto;font-variation-settings:normal;font-display:auto;font-style:italic;font-weight:400;font-stretch:100%;font-feature-settings:normal;unicode-range:U+0-7F;src:local('Demo');font-family:Demo}"#,
        r#"@font-face { font-family: Demo; src: local("Demo"); unicode-range: U+0-7F; font-feature-settings: normal; font-width: 100%; font-weight: 400; font-style: italic; font-display: auto; font-variation-settings: normal; font-named-instance: auto; font-language-override: normal; ascent-override: normal; descent-override: normal; line-gap-override: normal; }"#,
    );
    let report = parse_rule(
        "@font-face{font-family:Demo;font-variant:small-caps;font-weight:500!important}",
        &CssNamespaceContext::default(),
    );
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics().len(), 2);
    assert_eq!(
        report.syntax().as_ref().unwrap().serialize_cssom().unwrap(),
        "@font-face { font-family: Demo; }"
    );
}

#[test]
fn repaired_keyframes_wrapper_keytext_and_normal_blocks_use_shared_inverse() {
    for (source, expected, compact) in [
        ("@keyframes k{}", "@keyframes k {\n\n}", "@keyframes k { }"),
        (
            "@keyframes fade{from{opacity:0}to{opacity:1}}",
            "@keyframes fade {\n  0% { opacity: 0; }\n  100% { opacity: 1; }\n}",
            "@keyframes fade { 0% { opacity: 0; } 100% { opacity: 1; } }",
        ),
        (
            "@keyframes k{from,0%,to{}from{opacity:0;opacity:1;margin:1px 2px 3px 4px;margin-left:7px}}",
            "@keyframes k {\n  0%, 0%, 100% { }\n  0% { opacity: 1; margin: 1px 2px 3px 7px; }\n}",
            "@keyframes k { 0%, 0%, 100% { } 0% { opacity: 0; opacity: 1; margin: 1px 2px 3px 4px; margin-left: 7px; } }",
        ),
        (
            "@keyframes k{from{pause:var(--P)}}",
            "@keyframes k {\n  0% { pause: var(--P); }\n}",
            "@keyframes k { 0% { pause: var(--P); } }",
        ),
    ] {
        let value = rule(source);
        let before = value.clone();
        assert_eq!(value.serialize_cssom().unwrap(), expected);
        assert_eq!(value.to_specified_css().unwrap(), compact);
        assert_eq!(value, before);
    }
    let report = parse_rule(
        "@keyframes k{from{opacity:0!important;--X:ready!important;animation-duration:1s;animation-timing-function:linear;opacity:1}}",
        &CssNamespaceContext::default(),
    );
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics().len(), 3);
    let CssRule::Keyframes(frames) = report.syntax().as_ref().unwrap() else {
        panic!("keyframes")
    };
    assert!(
        frames.blocks()[0]
            .declarations()
            .iter()
            .all(|d| d.source().importance() == CssImportance::Normal)
    );
    assert_eq!(
        report.syntax().as_ref().unwrap().serialize_cssom().unwrap(),
        "@keyframes k {\n  0% { animation-timing-function: linear; opacity: 1; }\n}"
    );
}

fn pending(
    error: &CssRuleCssomSerializationError,
    original: &surgeist_css::CssDeclaration,
    ordinal: usize,
) {
    assert_eq!(error.kind(), RuleError::DeclarationBlock);
    let cause = error
        .source()
        .unwrap()
        .downcast_ref::<CssDeclarationBlockError>()
        .unwrap();
    assert!(matches!(
        cause.kind(),
        CssDeclarationBlockErrorKind::PendingFootprintUndetermined {
            property: CssKnownProperty::Margin
        }
    ));
    assert!(cause.declaration().unwrap().same_occurrence(original));
    assert_eq!(cause.authored_ordinal(), Some(ordinal));
    assert_eq!(cause.member_ordinal(), None);
}

#[test]
fn pending_nested_run_and_keyframe_errors_retain_original_source_and_actual_paths() {
    let report = parse_sheet(".ok{}.a{.b{}opacity:0;margin:var(--M)}");
    assert!(report.is_clean());
    let before = report.clone();
    let CssRule::Style(style) = &report.syntax().rules()[1] else {
        panic!("style")
    };
    let [CssRule::Style(_), CssRule::NestedDeclarations(run)] = style.rules() else {
        panic!("nested run")
    };
    let original = &run.declarations()[1];
    let error = report.syntax().serialize_cssom().unwrap_err();
    assert_eq!(error.rule_path(), &[1, 1]);
    assert_eq!(error.keyframe_block_index(), None);
    pending(&error, original, 1);
    let error = report.syntax().rules()[1].serialize_cssom().unwrap_err();
    assert_eq!(error.rule_path(), &[1]);
    pending(&error, original, 1);
    assert_eq!(original.to_specified_css().unwrap(), "margin: var(--M);");
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".ok { }\n.a { .b { } opacity: 0; margin: var(--M); }"
    );
    assert_eq!(report, before);

    let report = parse_sheet(
        ".ok{}@media screen{@keyframes k{from{opacity:0}to{opacity:1;margin:var(--M)}}}",
    );
    assert!(report.is_clean());
    let before = report.clone();
    let CssRule::Media(media) = &report.syntax().rules()[1] else {
        panic!("media")
    };
    let CssRule::Keyframes(frames) = &media.rules()[0] else {
        panic!("frames")
    };
    let original = frames.blocks()[1].declarations()[1].source();
    let error = report.syntax().serialize_cssom().unwrap_err();
    assert_eq!(error.rule_path(), &[1, 0]);
    assert_eq!(error.keyframe_block_index(), Some(1));
    pending(&error, original, 1);
    assert_eq!(original.parser_context(), CssParserContext::default());
    assert_eq!(original.to_specified_css().unwrap(), "margin: var(--M);");
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".ok { }\n@media screen { @keyframes k { 0% { opacity: 0; } 100% { opacity: 1; margin: var(--M); } } }"
    );
    assert_eq!(report, before);
    pending(&report.syntax().serialize_cssom().unwrap_err(), original, 1);
}

#[test]
fn page_source_undefined_is_typed_at_ordinary_nested_and_sheet_paths() {
    for (source, expected_path, compact) in [
        (
            "@page :recto{margin-left:1px}",
            vec![],
            "@page :recto { margin-left: 1px; }",
        ),
        (
            "@media print{.a{}@page :left{margin:1px}}",
            vec![1],
            "@media print { .a { } @page :left { margin: 1px; } }",
        ),
    ] {
        let value = rule(source);
        let before = value.clone();
        let error = value.serialize_cssom().unwrap_err();
        assert_eq!(
            error.kind(),
            RuleError::SourceUndefined(CssRuleCssomKind::Page)
        );
        assert_eq!(error.rule_path(), expected_path);
        assert!(error.source().is_none());
        assert_eq!(value.to_specified_css().unwrap(), compact);
        assert_eq!(value, before);
    }
    let report = parse_sheet(".a{}@page{} .b{}");
    assert!(report.is_clean());
    let before = report.clone();
    let error = report.syntax().serialize_cssom().unwrap_err();
    assert_eq!(
        error.kind(),
        RuleError::SourceUndefined(CssRuleCssomKind::Page)
    );
    assert_eq!(error.rule_path(), &[1]);
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".a { }\n@page { }\n.b { }"
    );
    assert_eq!(report, before);
}

#[test]
fn modern_unselected_whole_formats_keep_typed_limits_and_meaningful_compact_providers() {
    for (source, format, compact) in [
        (
            r#"@counter-style Tick{system:cyclic;symbols:"x"}"#,
            CssRuleCssomFormat::CounterStyle,
            r#"@counter-style Tick { system: cyclic; symbols: "x"; }"#,
        ),
        (
            "@font-feature-values Demo{}",
            CssRuleCssomFormat::FontFeatureValues,
            "@font-feature-values Demo { }",
        ),
        (
            "@font-palette-values --p{font-family:Demo}",
            CssRuleCssomFormat::FontPaletteValues,
            "@font-palette-values --p { font-family: Demo; }",
        ),
        (
            "@color-profile --x{}",
            CssRuleCssomFormat::ColorProfile,
            "@color-profile --x { }",
        ),
        (
            "@supports-condition --f{}",
            CssRuleCssomFormat::SupportsCondition,
            "@supports-condition --f { }",
        ),
        (
            "@custom-media --x TRUE;",
            CssRuleCssomFormat::CustomMedia,
            "@custom-media --x true;",
        ),
        (
            "@layer a,b;",
            CssRuleCssomFormat::LayerStatement,
            "@layer a, b;",
        ),
        ("@layer a{}", CssRuleCssomFormat::LayerBlock, "@layer a { }"),
        (
            "@supports(display:grid){}",
            CssRuleCssomFormat::Supports,
            "@supports (display:grid) { }",
        ),
        (
            "@container(width > 1px){}",
            CssRuleCssomFormat::Container,
            "@container (width > 1px) { }",
        ),
        (
            "@scope(.root)to (.stop){}",
            CssRuleCssomFormat::Scope,
            "@scope (.root) to (.stop) { }",
        ),
    ] {
        let value = rule(source);
        let before = value.clone();
        let error = value.serialize_cssom().unwrap_err();
        assert_eq!(error.kind(), RuleError::FormatUnavailable(format));
        assert_eq!(error.rule_path(), &[]);
        assert!(error.source().is_none());
        assert_eq!(value.to_specified_css().unwrap(), compact);
        assert_eq!(value, before);
        let report = parse_sheet(&format!(".ok{{}}{source}"));
        assert!(report.is_clean(), "{report:?}");
        assert_eq!(
            report.syntax().serialize_cssom().unwrap_err().rule_path(),
            &[1]
        );
    }
}

#[test]
fn exact_rule_tariffs_include_shared_projection_and_retained_losing_occurrences() {
    // Style: rule + selector list + simple class =3. Empty build/output=2.
    // All fallback probe: attempt1 + all368 admitted member presence checks=P369.
    // One opacity build I5/P3, output I4/P373 => rule I12/P379.
    // Duplicate opacity build I9/P5, same output => rule I16/P381.
    limits(".a{}", ".a { }", 5, 5);
    limits(".日本{}", ".日本 { }", 5, 5);
    limits(".a{opacity:0}", ".a { opacity: 0; }", 12, 379);
    limits(".a{opacity:0;opacity:1}", ".a { opacity: 1; }", 16, 381);
    // Keyframes: rule/name2 + block/selector-list/From3. Empty build/output2.
    // Keyframe All probe: attempt1 +361 admitted member presence checks=P362.
    // Keyframe opacity build I5/P5 + output I4/P366 => I14/P376; duplicate I18/P380.
    limits("@keyframes k{}", "@keyframes k {\n\n}", 2, 2);
    limits("@keyframes k{from{}}", "@keyframes k {\n  0% { }\n}", 7, 7);
    limits(
        "@keyframes k{from{opacity:0}}",
        "@keyframes k {\n  0% { opacity: 0; }\n}",
        14,
        376,
    );
    limits(
        "@keyframes k{from{opacity:0;opacity:1}}",
        "@keyframes k {\n  0% { opacity: 1; }\n}",
        18,
        380,
    );
    // Empty query Media: rule1 + query-list1; no child slice fee.
    limits("@media{}", "@media  {\n\n}", 2, 2);
}

#[test]
fn sheet_budget_is_cumulative_at_later_sibling_and_retry_preserves_the_tree() {
    let report = parse_sheet(".a{}.b{}");
    assert!(report.is_clean());
    let before = report.clone();
    let expected = ".a { }\n.b { }";
    let exact = Limits::new(11, 11, expected.len()); // sheet1 + two Style5.
    assert_eq!(
        report.syntax().serialize_cssom_with_limits(exact).unwrap(),
        expected
    );
    for (bound, kind) in [
        (Limits::new(6, 11, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(11, 6, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(11, 11, 7), Resource::ByteLimit),
        (
            Limits::new(10, 11, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(11, 10, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(11, 11, expected.len() - 1), Resource::ByteLimit),
    ] {
        let failure = report
            .syntax()
            .serialize_cssom_with_limits(bound)
            .unwrap_err();
        assert_eq!(failure.kind(), RuleError::Resource(kind));
        assert_eq!(failure.rule_path(), &[1]);
        assert_eq!(report, before);
        assert_eq!(
            report.syntax().serialize_cssom_with_limits(exact).unwrap(),
            expected
        );
    }
    let empty = parse_sheet("");
    assert_eq!(
        empty
            .syntax()
            .serialize_cssom_with_limits(Limits::new(1, 1, 0))
            .unwrap(),
        ""
    );
    for (bound, kind) in [
        (Limits::new(0, 1, 0), Resource::InputNodeLimit),
        (Limits::new(1, 0, 0), Resource::ProjectionNodeLimit),
    ] {
        let failure = empty
            .syntax()
            .serialize_cssom_with_limits(bound)
            .unwrap_err();
        assert_eq!(failure.kind(), RuleError::Resource(kind));
        assert_eq!(failure.rule_path(), &[]);
    }
}
#[test]
fn block_resource_failure_keeps_the_actual_generated_member_and_source_occurrence() {
    // Both retained opacity sources are visited before winner scans. Style prefix3
    // + block aggregate1 + two generated2 + first winner1 =P7; the next winner
    // is the original second occurrence, generated member0, before any block text.
    let value = rule(".a{opacity:0;opacity:1}");
    let before = value.clone();
    let CssRule::Style(style) = &value else {
        panic!("style")
    };
    let original = &style.declarations()[1];
    let failure = value
        .serialize_cssom_with_limits(Limits::new(16, 7, 18))
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        RuleError::Resource(Resource::ProjectionNodeLimit)
    );
    assert_eq!(failure.rule_path(), &[]);
    assert_eq!(failure.keyframe_block_index(), None);
    let cause = failure
        .source()
        .unwrap()
        .downcast_ref::<CssDeclarationBlockError>()
        .unwrap();
    assert!(
        matches!(cause.kind(), CssDeclarationBlockErrorKind::Serialization(error)
        if error.kind() == Resource::ProjectionNodeLimit)
    );
    assert!(cause.declaration().unwrap().same_occurrence(original));
    assert_eq!(cause.authored_ordinal(), Some(1));
    assert_eq!(cause.member_ordinal(), Some(0));
    assert_eq!(value, before);
    assert_eq!(
        value
            .serialize_cssom_with_limits(Limits::new(16, 381, 18))
            .unwrap(),
        ".a { opacity: 1; }"
    );

    // Keyframes prefix5 + aggregate1 + source admission2 + generated2
    // + member admission2 + first winner1 =P13. The same losing/winning
    // source schedule has a genuine keyframe block index and Normal source.
    let value = rule("@keyframes k{from{opacity:0;opacity:1}}");
    let before = value.clone();
    let CssRule::Keyframes(frames) = &value else {
        panic!("keyframes")
    };
    let original = frames.blocks()[0].declarations()[1].source();
    let expected = "@keyframes k {\n  0% { opacity: 1; }\n}";
    let failure = value
        .serialize_cssom_with_limits(Limits::new(18, 13, expected.len()))
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        RuleError::Resource(Resource::ProjectionNodeLimit)
    );
    assert_eq!(failure.rule_path(), &[]);
    assert_eq!(failure.keyframe_block_index(), Some(0));
    let cause = failure
        .source()
        .unwrap()
        .downcast_ref::<CssDeclarationBlockError>()
        .unwrap();
    assert!(cause.declaration().unwrap().same_occurrence(original));
    assert_eq!(cause.authored_ordinal(), Some(1));
    assert_eq!(cause.member_ordinal(), Some(0));
    assert_eq!(original.importance(), CssImportance::Normal);
    assert_eq!(value, before);
    assert_eq!(
        value
            .serialize_cssom_with_limits(Limits::new(18, 380, expected.len()))
            .unwrap(),
        expected
    );
}
#[test]
fn nested_media_sibling_resource_failure_keeps_child_path_and_atomic_retry() {
    let value = rule("@media{.a{}.b{}}");
    let before = value.clone();
    let expected = "@media  {\n  .a { }\n  .b { }\n}";
    // Media rule + empty query-list2; each empty Style5. Prefix bytes through
    // the first child and the second child's two spaces are9+1+2+6+1+2=21.
    let adequate = Limits::new(12, 12, expected.len());
    assert_eq!(
        value.serialize_cssom_with_limits(adequate).unwrap(),
        expected
    );
    for (bound, kind) in [
        (Limits::new(7, 12, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(12, 7, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(12, 12, 21), Resource::ByteLimit),
    ] {
        let failure = value.serialize_cssom_with_limits(bound).unwrap_err();
        assert_eq!(failure.kind(), RuleError::Resource(kind));
        assert_eq!(failure.rule_path(), &[1]);
        assert_eq!(failure.keyframe_block_index(), None);
        assert_eq!(value, before);
        assert_eq!(
            value.serialize_cssom_with_limits(adequate).unwrap(),
            expected
        );
    }
}

// Literal-only safe universal omission. The repaired original17 prefix above
// remains frozen; compact and standalone providers retain authored universals.
fn universal_case(
    authored: &str,
    compact_selector: &str,
    literal_selector: &str,
    namespaces: &CssNamespaceContext,
) {
    let source = format!("{authored}{{}}");
    let report = parse_rule(&source, namespaces);
    assert!(report.is_clean(), "{source}: {report:?}");
    let before = report.clone();
    let value = report.syntax().as_ref().unwrap();
    assert_eq!(
        value.serialize_cssom().unwrap(),
        format!("{literal_selector} {{ }}")
    );
    assert_eq!(
        value.to_specified_css().unwrap(),
        format!("{compact_selector} {{ }}")
    );
    assert_eq!(report, before);
    let selector_report = parse_selector(authored, namespaces);
    assert!(
        selector_report.is_clean(),
        "{authored}: {selector_report:?}"
    );
    let selector_before = selector_report.clone();
    assert_eq!(
        selector_report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        compact_selector
    );
    assert_eq!(selector_report, selector_before);
    assert_eq!(
        value.serialize_cssom().unwrap(),
        format!("{literal_selector} {{ }}")
    );
}

fn universal_default_namespaces() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([
        (None, surgeist_css::CssNamespaceName::new("urn:default")),
        (
            Some(surgeist_css::CssNamespacePrefix::try_new("n").unwrap()),
            surgeist_css::CssNamespaceName::new("urn:default"),
        ),
        (
            Some(surgeist_css::CssNamespacePrefix::try_new("svg").unwrap()),
            surgeist_css::CssNamespaceName::new("urn:svg"),
        ),
    ])
}

#[test]
fn literal_universal_omission_keeps_real_class_id_attribute_members_and_pe_tail() {
    for (authored, compact, literal) in [
        ("*.card", "*.card", ".card"),
        ("*#card", "*#card", "#card"),
        ("*[data-x]", "*[data-x]", "[data-x]"),
        ("*[data-x='v' i]", "*[data-x=\"v\" i]", "[data-x=\"v\" i]"),
        ("*#id.card[data-x]", "*#id.card[data-x]", "#id.card[data-x]"),
        ("*.card:hover", "*.card:hover", ".card:hover"),
        ("*.card::before", "*.card::before", ".card::before"),
        ("*#card::after", "*#card::after", "#card::after"),
        ("*[data-x]::before", "*[data-x]::before", "[data-x]::before"),
        (
            "*.card::part(foo bar):hover::before",
            "*.card::part(foo bar):hover::before",
            ".card::part(foo bar):hover::before",
        ),
        (
            "*.a>*.b+.c~*[data-x] *.d",
            "*.a > *.b + .c ~ *[data-x] *.d",
            ".a > .b + .c ~ [data-x] .d",
        ),
        ("* > *.card", "* > *.card", "* > .card"),
    ] {
        universal_case(authored, compact, literal, &CssNamespaceContext::default());
        universal_case(authored, compact, literal, &universal_default_namespaces());
    }
    let value = rule("*.card{}");
    let CssRule::Style(style) = &value else {
        panic!("style")
    };
    let [surgeist_css::CssStyleSelector::Selector(surgeist_css::CssSelector::Compound(compound))] =
        style.selectors().selectors()
    else {
        panic!("retained explicit universal")
    };
    let name = compound.type_selector().unwrap().clone();
    assert_eq!(name.local_name(), None);
    assert_eq!(name.namespace(), &surgeist_css::CssNamespaceConstraint::Any);
    assert_eq!(value.serialize_cssom().unwrap(), ".card { }");
    assert_eq!(compound.type_selector(), Some(&name));
    exact(
        "@namespace 'urn:default';*.card{}*#id{}*[data-x]{}",
        "@namespace url(\"urn:default\");\n.card { }\n#id { }\n[data-x] { }",
    );
}

#[test]
fn literal_universal_omission_preserves_explicit_namespace_prefixes_and_real_types() {
    for (authored, expected) in [
        ("n|*.card", "n|*.card"),
        ("svg|*.card", "svg|*.card"),
        ("|*.card", "|*.card"),
        ("*|*.card", "*|*.card"),
        ("n|*[data-x]", "n|*[data-x]"),
        ("|*#id", "|*#id"),
        ("*|*[|data-x]", "*|*[|data-x]"),
        ("n|*", "n|*"),
        ("|*", "|*"),
        ("*|*", "*|*"),
        ("leaf.card", "leaf.card"),
        ("n|leaf.card", "n|leaf.card"),
    ] {
        // n deliberately binds the same URI as Default: a symbolic authored
        // prefix still does not become an unqualified literal output name.
        universal_case(
            authored,
            expected,
            expected,
            &universal_default_namespaces(),
        );
    }
    for selector in ["|*.card", "*|*.card"] {
        universal_case(
            selector,
            selector,
            selector,
            &CssNamespaceContext::default(),
        );
    }
}

#[test]
fn literal_universal_omission_preserves_sole_pe_only_pseudo_only_and_featureless_forms() {
    for selector in [
        "*",
        "*::before",
        "*::after",
        "*::part(foo)",
        "*:hover",
        "*:host",
        "*:host(*.inner)",
        "*:host-context(*.inner)",
        "*:is(*.inner)",
        "*:is(:host)",
        "*:is(*:host)",
        "*:where(*.inner)",
        "*:not(*.inner)",
    ] {
        universal_case(
            selector,
            selector,
            selector,
            &CssNamespaceContext::default(),
        );
        universal_case(
            selector,
            selector,
            selector,
            &universal_default_namespaces(),
        );
    }
}

#[test]
fn literal_universal_omission_preserves_every_selector_function_argument_role() {
    for (authored, compact, literal) in [
        (
            "*.outer:is(*.inner)",
            "*.outer:is(*.inner)",
            ".outer:is(*.inner)",
        ),
        (
            "*.outer:where(*.inner)",
            "*.outer:where(*.inner)",
            ".outer:where(*.inner)",
        ),
        (
            "*.outer:not(*.inner)",
            "*.outer:not(*.inner)",
            ".outer:not(*.inner)",
        ),
        (
            "*.outer:has(>*.inner)",
            "*.outer:has(> *.inner)",
            ".outer:has(> *.inner)",
        ),
        (
            "*.outer:host(*.inner)",
            "*.outer:host(*.inner)",
            ".outer:host(*.inner)",
        ),
        (
            "*.outer:host-context(*.inner)",
            "*.outer:host-context(*.inner)",
            ".outer:host-context(*.inner)",
        ),
        (
            "*.outer:nth-child(2n of *.inner)",
            "*.outer:nth-child(2n of *.inner)",
            ".outer:nth-child(2n of *.inner)",
        ),
        (
            "*.outer:nth-last-child(2n of *.inner)",
            "*.outer:nth-last-child(2n of *.inner)",
            ".outer:nth-last-child(2n of *.inner)",
        ),
        (
            "*.outer::slotted(*.inner)",
            "*.outer::slotted(*.inner)",
            ".outer::slotted(*.inner)",
        ),
    ] {
        universal_case(authored, compact, literal, &CssNamespaceContext::default());
        universal_case(authored, compact, literal, &universal_default_namespaces());
    }
    // Argument-wide protection includes non-subject compounds and nested
    // argument routes. Later ordinary compounds regain literal eligibility.
    for (authored, compact, literal) in [
        (
            "*.outer:is(*.ancestor>*.subject)>*.tail",
            "*.outer:is(*.ancestor > *.subject) > *.tail",
            ".outer:is(*.ancestor > *.subject) > .tail",
        ),
        (
            "*.outer:is(*.inner:where(*.deep))>*.tail",
            "*.outer:is(*.inner:where(*.deep)) > *.tail",
            ".outer:is(*.inner:where(*.deep)) > .tail",
        ),
        (
            "*.outer:has(>*.inner:not(*.deep))>*.tail",
            "*.outer:has(> *.inner:not(*.deep)) > *.tail",
            ".outer:has(> *.inner:not(*.deep)) > .tail",
        ),
        (
            "*.outer:is(:host(*.inner))>*.tail",
            "*.outer:is(:host(*.inner)) > *.tail",
            ".outer:is(:host(*.inner)) > .tail",
        ),
    ] {
        universal_case(authored, compact, literal, &CssNamespaceContext::default());
        universal_case(authored, compact, literal, &universal_default_namespaces());
    }
}

#[test]
fn literal_universal_omission_protects_actual_nesting_and_scoped_anchor_payloads() {
    for selector in ["*&.card", "*&&.card"] {
        universal_case(
            selector,
            selector,
            selector,
            &CssNamespaceContext::default(),
        );
        universal_case(
            selector,
            selector,
            selector,
            &universal_default_namespaces(),
        );
    }
    universal_case(
        "*.ancestor &",
        "*.ancestor &",
        ".ancestor &",
        &CssNamespaceContext::default(),
    );
    exact(
        ".p{*&.card{}*.ancestor &{}}",
        ".p {\n  *&.card { }\n  .ancestor & { }\n}",
    );
    let value = rule("@scope{*&.card{}}");
    let before = value.clone();
    let CssRule::Scope(scope) = &value else {
        panic!("scope")
    };
    let [surgeist_css::CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("scoped style")
    };
    let [
        surgeist_css::CssScopedStyleSelector::Selector(surgeist_css::CssSelector::Compound(
            compound,
        )),
    ] = style.selectors().selectors()
    else {
        panic!("scope anchor compound")
    };
    assert_eq!(compound.nesting_selectors(), 0);
    assert_eq!(compound.scope_anchors(), 1);
    assert_eq!(compound.type_selector().unwrap().local_name(), None);
    assert_eq!(value.to_specified_css().unwrap(), "@scope { *&.card { } }");
    let selector = surgeist_css::CssSelector::Compound(compound.clone());
    assert_eq!(selector.to_specified_css().unwrap(), "*&.card");
    // Whole Scope literal format is unselected, so no public literal path
    // reaches its scoped Style payload. This is genuine parsed retention,
    // rather than an artificial conversion into an ordinary rule.
    let error = value.serialize_cssom().unwrap_err();
    assert_eq!(
        error.kind(),
        RuleError::FormatUnavailable(CssRuleCssomFormat::Scope)
    );
    assert_eq!(error.rule_path(), &[]);
    assert_eq!(value, before);
}

#[test]
fn literal_universal_omission_retains_input_projection_visits_and_saves_only_star_bytes() {
    // Rule/list/empty-build/output4 + Compound1/qualified-name1/member1=7.
    // A complex chain adds Complex1 and each actual three-node compound.
    for (source, expected, count) in [
        ("*.a{}", ".a { }", 7),
        ("*#a{}", "#a { }", 7),
        ("*[data-x]{}", "[data-x] { }", 7),
        ("*.a>*.b{}", ".a > .b { }", 11),
        ("*.a::before{}", ".a::before { }", 8),
        ("*{}", "* { }", 6),
        ("*::before{}", "*::before { }", 7),
        ("*:hover{}", "*:hover { }", 7),
        ("*&.a{}", "*&.a { }", 8),
        // Argument slices and output-role carriers have no invented node.
        // Outer Compound/name/class/pseudo4 + inner Compound/name/class3.
        ("*.a:is(*.x){}", ".a:is(*.x) { }", 11),
        // Relative1 is an actual owner visit in Has; its inner compound3.
        ("*.a:has(>*.x){}", ".a:has(> *.x) { }", 12),
        // Outer4 + argument Complex1 + two protected compounds6.
        ("*.a:is(*.x>*.y){}", ".a:is(*.x > *.y) { }", 15),
    ] {
        limits(source, expected, count, count);
    }
    let value = rule("*.a{}");
    let before = value.clone();
    assert_eq!(
        value
            .serialize_cssom_with_limits(Limits::new(7, 7, 6))
            .unwrap(),
        ".a { }"
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(7, 7, 7))
            .unwrap(),
        "*.a { }"
    );
    let compact_failure = value
        .to_specified_css_with_limits(Limits::new(7, 7, 6))
        .unwrap_err();
    assert_eq!(
        compact_failure.kind(),
        surgeist_css::CssSpecifiedRuleSerializationErrorKind::Resource(Resource::ByteLimit)
    );
    let selector_report = parse_selector("*.a", &CssNamespaceContext::default());
    let selector = selector_report.syntax().as_ref().unwrap();
    assert_eq!(
        selector
            .to_specified_css_with_limits(Limits::new(3, 3, 3))
            .unwrap(),
        "*.a"
    );
    assert_eq!(
        selector
            .to_specified_css_with_limits(Limits::new(3, 3, 2))
            .unwrap_err()
            .kind(),
        Resource::ByteLimit
    );
    assert_eq!(value, before);
}

#[test]
fn literal_universal_omission_sheet_budget_remains_cumulative_at_late_sibling() {
    let report = parse_sheet("*.a{}*.b{}");
    assert!(report.is_clean());
    let before = report.clone();
    let expected = ".a { }\n.b { }";
    let adequate = Limits::new(15, 15, 13); // sheet1 + each retained Style7.
    assert_eq!(expected.len(), 13);
    assert_eq!(
        report
            .syntax()
            .serialize_cssom_with_limits(adequate)
            .unwrap(),
        expected
    );
    assert_eq!(
        report
            .syntax()
            .to_specified_css_with_limits(Limits::new(15, 15, 15))
            .unwrap(),
        "*.a { }\n*.b { }"
    );
    for (bound, kind) in [
        (Limits::new(14, 15, 13), Resource::InputNodeLimit),
        (Limits::new(15, 14, 13), Resource::ProjectionNodeLimit),
        (Limits::new(15, 15, 12), Resource::ByteLimit),
        (Limits::new(8, 15, 13), Resource::InputNodeLimit),
        (Limits::new(15, 8, 13), Resource::ProjectionNodeLimit),
        (Limits::new(15, 15, 7), Resource::ByteLimit),
    ] {
        let error = report
            .syntax()
            .serialize_cssom_with_limits(bound)
            .unwrap_err();
        assert_eq!(error.kind(), RuleError::Resource(kind));
        assert_eq!(error.rule_path(), &[1]);
        assert_eq!(report, before);
        assert_eq!(
            report
                .syntax()
                .serialize_cssom_with_limits(adequate)
                .unwrap(),
            expected
        );
    }
}
