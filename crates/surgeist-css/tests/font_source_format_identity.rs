#![forbid(unsafe_code)]
//! Authored equality distinguishes the keyword and string format productions.
//! This is Surgeist's authored-model contract, separate from resource recognition.
//! Selected grammar: https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-format-values

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceDescriptorValue,
    CssFontFaceSource, CssFontFaceUrlSource, CssRule, parse_sheet,
};

fn sources(hints: &[&str]) -> Vec<CssFontFaceUrlSource> {
    let values = hints
        .iter()
        .map(|hint| format!("url(face) {hint}"))
        .collect::<Vec<_>>()
        .join(", ");
    let text = format!("@font-face {{ src: {values}; }}");
    let report = parse_sheet(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("one font-face rule");
    };
    let descriptor = face
        .descriptors()
        .effective(CssFontFaceDescriptorKind::Src)
        .expect("retained source descriptor");
    let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(list)) =
        descriptor.value()
    else {
        panic!("ordinary sources");
    };
    assert_eq!(list.sources().len(), hints.len());
    list.sources()
        .iter()
        .map(|source| match source {
            CssFontFaceSource::Url(url) => url.clone(),
            _ => panic!("URL source"),
        })
        .collect()
}

#[test]
fn same_spelling_keyword_and_string_formats_remain_distinct_authored_sources() {
    for spelling in [
        "woff",
        "woff2",
        "truetype",
        "opentype",
        "collection",
        "embedded-opentype",
        "svg",
    ] {
        let keyword = format!("format({spelling})");
        let string = format!("format(\"{spelling}\")");
        let values = sources(&[&keyword, &string]);
        assert_ne!(values[0], values[1], "{spelling}");
    }
}

#[test]
fn keyword_case_folding_keeps_string_case_and_production_distinctions() {
    let values = sources(&[
        "format(woff2)",
        "format(WOFF2)",
        "format(\"woff2\")",
        "format(\"WOFF2\")",
    ]);
    assert_eq!(values[0], values[1]);
    assert_ne!(values[2], values[3]);
    assert_ne!(values[0], values[2]);
    assert_ne!(values[1], values[3]);
}

#[test]
fn equal_decoded_strings_ignore_source_positions_and_keep_presence() {
    let values = sources(&[
        "format(\"woff2\")",
        r#"format("\77 off2")"#,
        "format(\"\")",
        "",
    ]);
    assert_eq!(values[0], values[1]);
    assert_ne!(values[2], values[3]);
    assert_ne!(values[0], values[2]);
}
