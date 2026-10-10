#![forbid(unsafe_code)]
//! Supplied CSSOM custom names use exact semantic strings, independently of CSS spelling.

use std::collections::HashSet;

use surgeist_css::*;

#[test]
fn semantic_names_preserve_supplied_characters_without_trimming_or_tokenization() {
    for supplied in [
        "--Theme",
        "--1",
        "---",
        "-- ",
        "--bad name",
        "--a:b;c{}()[]",
        "--x/**/y",
        "--x\\y",
        "--\n\r\t\u{c}",
        "--\u{1}\u{7f}",
        "--😀",
        "--é",
    ] {
        let name = CssCustomPropertyName::try_from_decoded(supplied).unwrap();
        assert_eq!(name.as_str(), supplied);
    }
}

#[test]
fn semantic_names_require_actual_leading_dashes_and_nonempty_suffix() {
    for supplied in [
        "",
        "-",
        "--",
        "name",
        "-name",
        " --name",
        "\n--name",
        "—-name",
        r"\2d \2d name",
    ] {
        assert!(
            CssCustomPropertyName::try_from_decoded(supplied).is_none(),
            "{supplied:?}"
        );
    }
}

#[test]
fn authored_escapes_and_semantic_backslashes_have_distinct_identities() {
    let authored = CssCustomPropertyName::try_new(r"--\78").unwrap();
    let supplied = CssCustomPropertyName::try_from_decoded(r"--\78").unwrap();
    assert_eq!(authored.as_str(), "--x");
    assert_eq!(supplied.as_str(), r"--\78");
    assert_ne!(authored, supplied);
    let escaped_prefix = CssCustomPropertyName::try_new(r"\2d \2d Theme").unwrap();
    assert_eq!(escaped_prefix.as_str(), "--Theme");
    assert!(CssCustomPropertyName::try_from_decoded(r"\2d \2d Theme").is_none());
    let semantic_space = CssCustomPropertyName::try_from_decoded("--bad name").unwrap();
    assert_eq!(
        CssCustomPropertyName::try_new(r"--bad\ name").unwrap(),
        semantic_space
    );
    assert!(CssCustomPropertyName::try_new("--bad name").is_none());
}

#[test]
fn case_and_unicode_codepoint_distinctions_are_preserved_in_equality_and_hashing() {
    let supplied = ["--Theme", "--theme", "--foó", "--foo\u{301}", "--𝓧", "--X"];
    let names: HashSet<_> = supplied
        .into_iter()
        .map(|s| CssCustomPropertyName::try_from_decoded(s).unwrap())
        .collect();
    assert_eq!(names.len(), 6);
    for name in supplied {
        assert!(names.contains(&CssCustomPropertyName::try_from_decoded(name).unwrap()));
    }
}

#[test]
fn nul_is_retained_in_identity_and_existing_identifier_output_replaces_it_only_on_output() {
    let name = CssCustomPropertyName::try_from_decoded("--a\0Z").unwrap();
    assert_eq!(name.as_str().as_bytes(), b"--a\0Z");
    let replaced = CssCustomPropertyName::try_from_decoded("--a�Z").unwrap();
    assert_ne!(name, replaced);
    let report = parse_property_value_text(
        "RAW",
        CssPropertyNameRef::Custom(&name),
        CssImportance::Normal,
    );
    assert!(report.is_clean());
    let declaration = report.syntax().as_ref().unwrap();
    assert_eq!(declaration.custom().unwrap().name(), &name);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert_eq!(declaration.to_specified_css().unwrap(), "--a�Z: RAW;");
    assert_eq!(
        declaration.custom().unwrap().name().as_str().as_bytes(),
        b"--a\0Z"
    );
}

#[test]
fn owned_semantic_names_compose_real_value_origins_and_parsed_name_equality_after_input_drop() {
    let name = {
        let supplied = "--bad name".to_owned();
        CssCustomPropertyName::try_from_decoded(supplied).unwrap()
    };
    let report = {
        let input = "MiXeD".to_owned();
        parse_property_value_text(
            &input,
            CssPropertyNameRef::Custom(&name),
            CssImportance::Important,
        )
    };
    let declaration = report.syntax().as_ref().unwrap();
    assert!(declaration.parsed_name().is_none());
    assert_eq!(
        declaration.parsed_value().unwrap().source().as_str(),
        "MiXeD"
    );
    assert_eq!(
        declaration.to_specified_css().unwrap(),
        "--bad\\ name: MiXeD !important;"
    );
    let parsed = {
        let input = "--bad\\ name:MiXeD".to_owned();
        parse_declaration_block_contents(&input)
    };
    let parsed = &parsed.syntax().as_slice()[0];
    assert_eq!(parsed.custom().unwrap().name(), &name);
    assert_eq!(
        parsed.parsed_name().unwrap().source().as_str(),
        "--bad\\ name:MiXeD"
    );
}
