//! Current import-owned media: CSSOM's selected plain-import URL writer and
//! #836's adopted WebKit rule-owned media, independent of child-sheet presence.
//! Every import, including modern clauses and `src()`, uses the CSSOM editor
//! capture's URL-first CSSImportRule text (#1327).

use crate::css::support::failure_classes::SerializationFailure as Resource;
use crate::css::support::serialization_limits::LimitFailure;
use crate::css::support::{graph_cssom, graph_specified};
use std::error::Error;
use surgeist_css::{
    CssGroupPreludeRef as Prelude, CssImportRule, CssImportRuleView as ImportView,
    CssMediaQueryList, CssParser, CssRule, CssRuleGraphInput as Input, CssSerializationError,
    CssSerializationLimits as Limits, CssValueOrigin,
};

fn import(source: &str) -> CssImportRule {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [CssRule::Import(rule)] = report.syntax().rules() else {
        panic!("one checked import");
    };
    rule.clone()
}

fn media(source: &str) -> CssMediaQueryList {
    let report = CssMediaQueryList::parse(&CssParser::default(), source);
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
    let view = ImportView::try_new(&original, &current, Default::default()).unwrap();
    assert_eq!(
        view.serialize_cssom(Default::default()).unwrap(),
        "@import url(\"../x.css\") print, (color);"
    );
    assert_eq!(
        ImportView::try_new(&original, &empty, Default::default())
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap(),
        "@import url(\"../x.css\");"
    );
    assert_eq!(
        original
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
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
        Input::import(&original, &print),
        Input::import(&original, &empty),
    ];
    let all = media("all");
    // Formatting inputs do not decide destination hierarchy; CSSOM does.
    let group = Input::group(Prelude::Media(&all), &children);
    assert_eq!(
        graph_cssom(group, Default::default()).unwrap(),
        "@media all {\n  @import url(\"x\") print;\n  @import url(\"x\");\n}"
    );
    assert_eq!(
        graph_specified(group, Default::default()).unwrap(),
        "@media all { @import 'x' print; @import 'x'; }"
    );
    assert_eq!(
        original
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "@import 'x' screen;"
    );
}

#[test]
fn current_opaque_media_preserves_absent_optional_clause_interpretation() {
    let original = import("@import 'x' print;");
    let current = media("layer(theme)");
    let view = ImportView::try_new(&original, &current, Default::default()).unwrap();
    // Parenthesize the ambiguous current media operand rather than turn it into
    // a new import layer clause. The shared import interpretation owner does this.
    assert_eq!(
        view.serialize_cssom(Default::default()).unwrap(),
        "@import url(\"x\") (layer(theme));"
    );
    let reparsed = import("@import url(\"x\") (layer(theme));");
    assert!(reparsed.layer().is_none());
    assert!(reparsed.supports().is_none());
    assert_eq!(
        original
            .media()
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "print"
    );
}

#[test]
fn modern_clause_and_extended_target_output_uses_the_cssom_url_first() {
    let current = media("print");
    for (source, expected, compact) in [
        (
            "@import 'x' layer(theme) supports(display:grid) screen;",
            "@import url(\"x\") layer(theme) supports(display:grid) print;",
            "@import 'x' layer(theme) supports(display:grid) print;",
        ),
        (
            "@import src('x') screen;",
            "@import url(\"x\") print;",
            "@import src('x') print;",
        ),
    ] {
        let original = import(source);
        let before = original.clone();
        let view = ImportView::try_new(&original, &current, Default::default()).unwrap();
        assert_eq!(view.serialize_cssom(Default::default()).unwrap(), expected);
        assert_eq!(
            graph_specified(Input::import(view.rule(), view.media()), Default::default()).unwrap(),
            compact
        );
        assert_eq!(original, before);
    }
    let original = import("@import 'x';");
    let children = [Input::import(&original, &current)];
    let condition_rule = CssParser::default().parse_sheet("@supports (display: grid) {}");
    let [CssRule::Supports(supports)] = condition_rule.syntax().rules() else {
        panic!("supports");
    };
    let group = Input::group(Prelude::Supports(supports.condition()), &children);
    assert_eq!(
        graph_cssom(group, Default::default()).unwrap(),
        "@supports (display: grid) {\n  @import url(\"x\") print;\n}"
    );
}

#[test]
fn supplied_recovered_media_keeps_current_origin_and_its_cssom_projection() {
    let original = import("@import 'x' print;");
    let report = CssMediaQueryList::parse(&CssParser::default(), "screen,???");
    assert!(!report.is_clean());
    let current = report.syntax();
    let before = report.clone();
    let error = ImportView::try_new(&original, current, Limits::new(6, 7, 128)).unwrap_err();
    assert!(
        error.is_resource_limit()
            && error
                .serialization_error()
                .is_some_and(|error| error.is_projection_node_limit()),
        "{error:?}"
    );
    let provider = error
        .source()
        .unwrap()
        .downcast_ref::<CssSerializationError>()
        .expect("media failure");
    assert_eq!(provider.origin(), Some(current.queries()[1].origin()));
    let view = ImportView::try_new(&original, current, Limits::new(6, 8, 128)).unwrap();
    assert_eq!(
        view.serialize_cssom(Default::default()).unwrap(),
        "@import url(\"x\") screen, not all;"
    );
    assert_eq!(report, before);
    assert_eq!(
        original
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "@import 'x' print;"
    );
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
            ImportView::try_new(&original, &empty, limits)
                .unwrap_err()
                .limit_kind(),
            Some(kind)
        );
        assert_eq!(original, before);
    }
    let view = ImportView::try_new(&original, &empty, Limits::new(3, 3, expected.len())).unwrap();
    assert_eq!(
        view.serialize_cssom(Limits::new(3, 3, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(original, before);
    // The compact and literal requests bound their bytes separately while
    // preserving the same provider node accounting.
    let compact = "@import 'é';";
    let edited = Input::import(view.rule(), view.media());
    assert!(
        graph_specified(edited, Limits::new(2, 3, compact.len()))
            .unwrap_err()
            .is_resource_limit()
            && graph_specified(edited, Limits::new(2, 3, compact.len()))
                .unwrap_err()
                .serialization_error()
                .is_some_and(|error| error.is_input_node_limit())
    );
    assert_eq!(
        graph_specified(edited, Limits::new(3, 3, compact.len())).unwrap(),
        compact
    );
    assert_eq!(
        graph_cssom(edited, Limits::new(3, 3, expected.len())).unwrap(),
        expected
    );
}

#[test]
fn recursive_siblings_share_node_and_output_limits_with_exact_failure_path() {
    let original = import("@import 'x' screen;");
    let print = media("print");
    let empty = CssMediaQueryList::new(Vec::new());
    let children = [
        Input::import(&original, &print),
        Input::import(&original, &empty),
    ];
    let all = media("all");
    let group = Input::group(Prelude::Media(&all), &children);
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
        let error = graph_cssom(group, limits).unwrap_err();
        assert_eq!(error.limit_kind(), Some(kind));
        assert_eq!(error.rule_path(), &[1]);
    }
    let error = graph_cssom(group, Limits::new(12, 12, expected.len() - 1)).unwrap_err();
    assert!(
        error.is_resource_limit()
            && error
                .serialization_error()
                .is_some_and(|error| error.is_byte_limit()),
        "{error:?}"
    );
    assert_eq!(
        graph_cssom(group, Limits::new(12, 12, expected.len())).unwrap(),
        expected
    );
    assert_eq!(
        original
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "@import 'x' screen;"
    );
}
