#![forbid(unsafe_code)]
//! Current import-owned media: CSSOM's selected plain-import URL writer and
//! #836's adopted WebKit rule-owned media, independent of child-sheet presence.
//! Modern imports retain the existing CSS qualification, not a new wrapper policy.

use std::error::Error;
use surgeist_css::{
    CssEditedGroupPreludeRef as Prelude, CssEditedRuleView as Edited, CssImportRule,
    CssImportRuleView as ImportView, CssMediaCssomSerializationError, CssMediaQueryList, CssRule,
    CssRuleCssomFormat, CssRuleCssomSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationErrorKind as Resource,
    CssSpecifiedValueSerializationLimits as Limits, CssValueOrigin, parse_media_query_list,
    parse_sheet,
};

fn import(source: &str) -> CssImportRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [CssRule::Import(rule)] = report.syntax().rules() else {
        panic!("one checked import");
    };
    rule.clone()
}

fn media(source: &str) -> CssMediaQueryList {
    let report = parse_media_query_list(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone()
}

#[test]
fn supplied_changed_and_empty_media_replace_authored_media_without_mutation() {
    let original = import("/*é*/ @IMPORT '../x.css' screen;");
    let before = original.clone();
    let current = media("PRINT, (COLOR)");
    let current_before = current.clone();
    let empty = CssMediaQueryList::new(Vec::new());
    let view = ImportView::try_new(&original, &current).unwrap();
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "@import url(\"../x.css\") print, (color);"
    );
    assert_eq!(
        ImportView::try_new(&original, &empty)
            .unwrap()
            .serialize_cssom()
            .unwrap(),
        "@import url(\"../x.css\");"
    );
    assert_eq!(
        original.serialize().unwrap().as_css(),
        "@import '../x.css' screen;"
    );
    assert_eq!(original, before);
    assert_eq!(current, current_before);
    let CssValueOrigin::Parsed(origin) = view.rule().origin() else {
        panic!("real at-keyword");
    };
    assert_eq!(origin.source().as_str(), "/*é*/ @IMPORT '../x.css' screen;");
    assert_eq!(origin.span().start().byte_offset().value(), 7);
    assert!(origin.source().same_snapshot(match before.origin() {
        CssValueOrigin::Parsed(origin) => origin.source(),
        _ => panic!("parsed"),
    }));
    let CssValueOrigin::Parsed(query) = view.media().queries()[0].origin() else {
        panic!("real current query");
    };
    assert_eq!(query.source().as_str(), "PRINT, (COLOR)");
    assert_eq!(query.span().start().byte_offset().value(), 0);
    assert!(!query.source().same_snapshot(origin.source()));
}

#[test]
fn recursive_edited_media_group_uses_each_current_list_and_original_target() {
    let original = import("@import 'x' screen;");
    let print = media("print");
    let empty = CssMediaQueryList::new(Vec::new());
    let children = [
        Edited::try_import(ImportView::try_new(&original, &print).unwrap()).unwrap(),
        Edited::try_import(ImportView::try_new(&original, &empty).unwrap()).unwrap(),
    ];
    let all = media("all");
    // Formatting views do not decide destination hierarchy; CSSOM does.
    let group = Edited::try_group(Prelude::Media(&all), &children).unwrap();
    assert_eq!(
        group.serialize_cssom().unwrap(),
        "@media all {\n  @import url(\"x\") print;\n  @import url(\"x\");\n}"
    );
    assert_eq!(
        group.to_specified_css().unwrap(),
        "@media all { @import 'x' print; @import 'x'; }"
    );
    assert!(
        children[0].parsed_rule().is_none(),
        "no reconstructed authored rule"
    );
    assert_eq!(
        original.serialize().unwrap().as_css(),
        "@import 'x' screen;"
    );
}

#[test]
fn current_opaque_media_preserves_absent_optional_clause_interpretation() {
    let original = import("@import 'x' print;");
    let current = media("layer(theme)");
    let view = ImportView::try_new(&original, &current).unwrap();
    // Parenthesize the ambiguous current media operand rather than turn it into
    // a new import layer clause. The shared import interpretation owner does this.
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "@import url(\"x\") (layer(theme));"
    );
    let reparsed = import("@import url(\"x\") (layer(theme));");
    assert!(reparsed.layer().is_none());
    assert!(reparsed.supports().is_none());
    assert_eq!(
        original
            .media()
            .unwrap()
            .serialize_cssom()
            .unwrap()
            .as_css(),
        "print"
    );
}

#[test]
fn modern_clause_and_extended_target_output_retains_existing_qualification() {
    let current = media("print");
    for (source, expected) in [
        (
            "@import 'x' layer(theme) supports(display:grid) screen;",
            "@import 'x' layer(theme) supports(display:grid) print;",
        ),
        ("@import src('x') screen;", "@import src('x') print;"),
    ] {
        let original = import(source);
        let before = original.clone();
        let view = ImportView::try_new(&original, &current).unwrap();
        assert_eq!(view.serialize_cssom().unwrap(), expected);
        assert_eq!(
            Edited::try_import(view)
                .unwrap()
                .to_specified_css()
                .unwrap(),
            expected
        );
        assert_eq!(original, before);
    }
    let original = import("@import 'x';");
    let children = [Edited::try_import(ImportView::try_new(&original, &current).unwrap()).unwrap()];
    let condition_rule = parse_sheet("@supports (display: grid) {}");
    let [CssRule::Supports(supports)] = condition_rule.syntax().rules() else {
        panic!("supports");
    };
    let group = Edited::try_group(Prelude::Supports(supports.condition()), &children).unwrap();
    assert_eq!(
        group.serialize_cssom().unwrap_err().kind(),
        Kind::FormatUnavailable(CssRuleCssomFormat::Supports)
    );
}

#[test]
fn supplied_recovered_media_keeps_current_origin_and_existing_cssom_projection() {
    let original = import("@import 'x' print;");
    let report = parse_media_query_list("screen,???");
    assert!(!report.is_clean());
    let current = report.syntax();
    let before = report.clone();
    let error =
        ImportView::try_new_with_limits(&original, current, Limits::new(6, 7, 128)).unwrap_err();
    assert_eq!(error.kind(), Kind::Resource(Resource::ProjectionNodeLimit));
    let provider = error
        .source()
        .unwrap()
        .downcast_ref::<CssMediaCssomSerializationError>()
        .expect("media failure");
    assert_eq!(provider.origin(), current.queries()[1].origin());
    let view = ImportView::try_new_with_limits(&original, current, Limits::new(6, 8, 128)).unwrap();
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "@import url(\"x\") screen, not all;"
    );
    assert_eq!(report, before);
    assert_eq!(original.serialize().unwrap().as_css(), "@import 'x' print;");
}

#[test]
fn empty_current_media_charges_current_payload_and_url_bytes_atomically() {
    let original = import("@import 'é' screen, print, speech;");
    let before = original.clone();
    let empty = CssMediaQueryList::new(Vec::new());
    let expected = "@import url(\"é\");";
    // Rule1 + original target1 + supplied empty media-list aggregate1. Retired
    // authored media is not visited, and the two-byte é counts as output bytes.
    for (limits, kind) in [
        (Limits::new(2, 3, expected.len()), Resource::InputNodeLimit),
        (
            Limits::new(3, 2, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
        (Limits::new(3, 3, expected.len() - 1), Resource::ByteLimit),
    ] {
        assert_eq!(
            ImportView::try_new_with_limits(&original, &empty, limits)
                .unwrap_err()
                .kind(),
            Kind::Resource(kind)
        );
        assert_eq!(original, before);
    }
    let view =
        ImportView::try_new_with_limits(&original, &empty, Limits::new(3, 3, expected.len()))
            .unwrap();
    assert_eq!(
        view.serialize_cssom_with_limits(Limits::new(3, 3, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(original, before);
    // Embedding checks compact output; the final literal request separately
    // bounds its bytes while preserving the same provider node accounting.
    let compact = "@import 'é';";
    assert_eq!(
        Edited::try_import_with_limits(view, Limits::new(2, 3, compact.len()))
            .unwrap_err()
            .kind(),
        Kind::Resource(Resource::InputNodeLimit)
    );
    let edited = Edited::try_import_with_limits(view, Limits::new(3, 3, compact.len())).unwrap();
    assert_eq!(edited.to_specified_css().unwrap(), compact);
    assert_eq!(
        edited
            .serialize_cssom_with_limits(Limits::new(3, 3, expected.len()))
            .unwrap(),
        expected
    );
}

#[test]
fn recursive_siblings_share_node_and_output_limits_with_exact_failure_path() {
    let original = import("@import 'x' screen;");
    let print = media("print");
    let empty = CssMediaQueryList::new(Vec::new());
    let children = [
        Edited::try_import(ImportView::try_new(&original, &print).unwrap()).unwrap(),
        Edited::try_import(ImportView::try_new(&original, &empty).unwrap()).unwrap(),
    ];
    let all = media("all");
    let group = Edited::try_group(Prelude::Media(&all), &children).unwrap();
    let expected = "@media all {\n  @import url(\"x\") print;\n  @import url(\"x\");\n}";
    // Group1 + outer list/type/query3 + first rule/target/list/type/query5
    // + second rule/target/empty-list3 = 12, charged once cumulatively.
    for (limits, kind) in [
        (
            Limits::new(11, 12, expected.len()),
            Resource::InputNodeLimit,
        ),
        (
            Limits::new(12, 11, expected.len()),
            Resource::ProjectionNodeLimit,
        ),
    ] {
        let error = group.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(error.kind(), Kind::Resource(kind));
        assert_eq!(error.rule_path(), &[1]);
    }
    let error = group
        .serialize_cssom_with_limits(Limits::new(12, 12, expected.len() - 1))
        .unwrap_err();
    assert_eq!(error.kind(), Kind::Resource(Resource::ByteLimit));
    assert_eq!(
        group
            .serialize_cssom_with_limits(Limits::new(12, 12, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        original.serialize().unwrap().as_css(),
        "@import 'x' screen;"
    );
}
