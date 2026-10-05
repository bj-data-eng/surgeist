#![forbid(unsafe_code)]
//! Independent exact fixtures for the 19 ordinary / 17 scoped variants at
//! 31dd36b7. Scope boundary grammar is covered by scope_complete_lifecycle.
use surgeist_css::{CssRule, parse_sheet};

fn exact(source: &str, expected: &str) {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let before = report.clone();
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(report, before);
    let emitted = parse_sheet(expected);
    assert!(
        emitted.is_clean(),
        "{expected}: {:?}",
        emitted.diagnostics()
    );
    assert_eq!(emitted.syntax().to_specified_css().unwrap(), expected);
}

macro_rules! ordinary {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            exact($source, $expected);
        }
    };
}
macro_rules! scoped {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            exact(
                concat!("@scope(.root){", $source, "}"),
                concat!("@scope (.root) { ", $expected, " }"),
            );
        }
    };
}

ordinary!(
    custom_media_emits_boolean_body,
    "@custom-media --x TRUE;",
    "@custom-media --x true;"
);
ordinary!(
    import_emits_admitted_target,
    "@import \"x\";",
    "@import \"x\";"
);
ordinary!(
    namespace_emits_escaped_literal_control,
    "@namespace n 'u';",
    "@namespace n url(\"u\");"
);
ordinary!(
    counter_style_emits_typed_descriptors,
    "@counter-style Tick{system:cyclic;symbols:\"x\"}",
    "@counter-style Tick { system: cyclic; symbols: \"x\"; }"
);
ordinary!(
    page_emits_margin_importance,
    "@page :left{margin:1px!important}",
    "@page :left { margin: 1px !important; }"
);
ordinary!(
    layer_statement_emits_ordered_names,
    "@layer a,b;",
    "@layer a, b;"
);
ordinary!(layer_block_emits_empty_block, "@layer a{}", "@layer a { }");
ordinary!(
    font_face_reuses_empty_provider_control,
    "@font-face{}",
    "@font-face { }"
);
ordinary!(
    feature_values_reuses_empty_provider_control,
    "@font-feature-values Demo{}",
    "@font-feature-values Demo { }"
);
ordinary!(
    palette_reuses_family_provider_control,
    "@font-palette-values --p{font-family:Demo}",
    "@font-palette-values --p { font-family: Demo; }"
);
ordinary!(
    profile_reuses_empty_provider_control,
    "@color-profile --x{}",
    "@color-profile --x { }"
);
ordinary!(
    keyframes_emits_blocks_and_typed_numeric_values,
    "@keyframes k{from{opacity:.25}to{opacity:1}}",
    "@keyframes k { 0% { opacity: 0.25; } 100% { opacity: 1; } }"
);
ordinary!(
    style_emits_ordered_selector_list_and_typed_value,
    ".a,#b{opacity:.5}",
    ".a, #b { opacity: 0.5; }"
);
ordinary!(
    media_emits_query_list_and_empty_block,
    "@media PRINT,ONLY SCREEN{}",
    "@media print, only screen { }"
);
ordinary!(
    supports_emits_symbolic_condition,
    "@supports (display: grid){}",
    "@supports (display: grid) { }"
);
ordinary!(
    named_supports_reuses_empty_provider_control,
    "@supports-condition --f{}",
    "@supports-condition --f { }"
);
ordinary!(
    container_emits_symbolic_condition,
    "@container (width > 1px){}",
    "@container (width > 1px) { }"
);
ordinary!(
    scope_emits_existing_ordinary_boundary,
    "@scope(.root)to (.stop){}",
    "@scope (.root) to (.stop) { }"
);

#[test]
fn detached_nested_declarations_emits_the_authored_run() {
    let report = parse_sheet(".parent{.child{}opacity:.5!important;opacity:.25}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    let [CssRule::Style(_), run @ CssRule::NestedDeclarations(_)] = parent.rules() else {
        panic!("child then run")
    };
    let before = run.clone();
    assert_eq!(
        run.to_specified_css().unwrap(),
        "opacity: 0.5 !important; opacity: 0.25;"
    );
    assert_eq!(run, &before);
}

scoped!(
    scoped_page_emits_inside_an_ordinary_group,
    "@media print{@page :left{margin:1px!important}}",
    "@media print { @page :left { margin: 1px !important; } }"
);
scoped!(
    scoped_counter_style_reuses_descriptor_boundary,
    "@counter-style Tick{system:cyclic;symbols:\"x\"}",
    "@counter-style Tick { system: cyclic; symbols: \"x\"; }"
);
scoped!(
    scoped_font_face_reuses_provider,
    "@font-face{}",
    "@font-face { }"
);
scoped!(
    scoped_keyframes_reuses_block_boundary,
    "@keyframes k{from{opacity:0}}",
    "@keyframes k { 0% { opacity: 0; } }"
);
scoped!(
    scoped_custom_media_preserves_boolean,
    "@custom-media --x true;",
    "@custom-media --x true;"
);
scoped!(
    scoped_feature_values_reuses_provider,
    "@font-feature-values Demo{}",
    "@font-feature-values Demo { }"
);
scoped!(
    scoped_palette_reuses_provider,
    "@font-palette-values --p{font-family:Demo}",
    "@font-palette-values --p { font-family: Demo; }"
);
scoped!(
    scoped_profile_reuses_provider,
    "@color-profile --x{}",
    "@color-profile --x { }"
);
scoped!(
    scoped_style_preserves_selector_order,
    ".a,.b{opacity:.5}",
    ".a, .b { opacity: 0.5; }"
);
scoped!(
    scoped_media_preserves_query_order,
    "@media print,screen{}",
    "@media print, screen { }"
);
scoped!(
    scoped_supports_preserves_condition,
    "@supports (display: grid){}",
    "@supports (display: grid) { }"
);
scoped!(
    scoped_named_supports_reuses_provider,
    "@supports-condition --f{}",
    "@supports-condition --f { }"
);
scoped!(
    scoped_container_preserves_condition,
    "@container (width > 1px){}",
    "@container (width > 1px) { }"
);
scoped!(
    scoped_layer_statement_preserves_names,
    "@layer a,b;",
    "@layer a, b;"
);
scoped!(
    scoped_layer_block_preserves_empty_block,
    "@layer a{}",
    "@layer a { }"
);
scoped!(
    scoped_scope_reuses_existing_ordinary_boundary,
    "@scope(.inner){}",
    "@scope (.inner) { }"
);

#[test]
fn scoped_nested_declarations_preserve_separate_runs_around_a_child() {
    exact(
        ".parent{@scope{opacity:.25;.child{}opacity:.5}}",
        ".parent { @scope { opacity: 0.25; .child { } opacity: 0.5; } }",
    );
}
