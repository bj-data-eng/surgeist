#![forbid(unsafe_code)]
use surgeist_css::{
    CssImportLayer, CssLayerName, CssNamespaceContext, CssRule, CssValueOrigin, parse_rule,
    parse_sheet,
};

// Cascade 5 CR 2022-01-13 §6.4.2 takes <ident> components. CSS Syntax consumes an escaped
// numeric-start spelling as one decoded identifier, not an authored number.
#[test]
fn published_css_layer_ingress_preserves_decoded_escaped_identifiers() {
    for source in [r"@layer \31 edge;", r"@layer outer.\31 edge {}"] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let components = match report.syntax() {
            Some(CssRule::LayerStatement(rule)) => rule.names().names()[0].components(),
            Some(CssRule::LayerBlock(rule)) => rule.name().unwrap().components(),
            other => panic!("expected actual checked layer occurrence for {source}, got {other:?}"),
        };
        assert_eq!(components.last().unwrap(), "1edge");
    }
}

#[test]
fn published_css_sheet_keeps_escaped_layer_statement_between_actual_siblings() {
    let report = parse_sheet(
        r"@layer outer { @layer inner {} @layer {} } @layer outer.\31 edge, safe; .after { color: red; }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::LayerBlock(outer),
        CssRule::LayerStatement(statement),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!(
            "all three actual source occurrences must remain, got {:?}",
            report.syntax().rules()
        );
    };
    assert_eq!(outer.rules().len(), 2);
    assert_eq!(
        statement.names().names()[0].components(),
        &["outer", "1edge"]
    );
    assert_eq!(statement.names().names()[1].components(), &["safe"]);
}

#[test]
fn published_layer_name_constructor_accepts_decoded_identifier_values() {
    for (component, spelling) in [("1edge", r"\31 edge"), ("a b", r"a\ b"), ("a.b", r"a\.b")] {
        let name = CssLayerName::try_new([component])
            .expect("decoded identifier is not authored spelling");
        assert_eq!(name.components(), &[component]);
        assert_eq!(name.serialize_specified().unwrap(), spelling);
        let source = format!("@layer {spelling};");
        let report = parse_rule(&source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let Some(CssRule::LayerStatement(rule)) = report.syntax() else {
            panic!("one named layer statement")
        };
        assert_eq!(rule.names().names()[0], name);
    }
    assert!(CssLayerName::try_new(["inherit"]).is_none());
}

#[test]
fn escaped_import_layer_is_named_and_keeps_original_input_after_input_drop() {
    let source = String::from(r"@import 'x.css' layer(theme.\31 x);");
    let report = parse_sheet(&source);
    drop(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("one actual import")
    };
    let Some(CssImportLayer::Named(name)) = import.layer() else {
        panic!("escaped layer name must not become media: {import:?}")
    };
    assert_eq!(name.components(), &["theme", "1x"]);
    assert!(import.media().is_none());
    assert_eq!(name.serialize_specified().unwrap(), r"theme.\31 x");
    assert_eq!(
        import.serialize().unwrap().as_css(),
        r"@import 'x.css' layer(theme.\31 x);"
    );
    let CssValueOrigin::Parsed(origin) = import.origin() else {
        panic!("original parsed provenance")
    };
    assert_eq!(
        origin.source().as_str(),
        r"@import 'x.css' layer(theme.\31 x);"
    );
    assert_eq!(origin.span().start().byte_offset().value(), 0);
}

#[test]
fn decoded_admission_retains_reserved_keywords_and_actual_authored_grammar() {
    for component in [
        "",
        "a\0b",
        "inherit",
        "INITIAL",
        "unset",
        "ReVeRt",
        "revert-layer",
    ] {
        assert!(
            CssLayerName::try_new([component]).is_none(),
            "{component:?}"
        );
    }
    assert!(CssLayerName::try_new(Vec::<String>::new()).is_none());
    // <ident>, rather than <custom-ident>: default is not a reserved layer name.
    let name = CssLayerName::try_new(["default", "Case"]).unwrap();
    assert_eq!(name.serialize_specified().unwrap(), "default.Case");
    let report = parse_rule("@layer default.Case;", &CssNamespaceContext::default());
    assert!(report.is_clean());
    for invalid in [
        "@layer 1edge;",
        "@layer a b;",
        "@layer a..b;",
        "@layer a. b;",
        "@layer a .b;",
        r"@layer \69 nherit;",
        "@layer safe.INITIAL;",
    ] {
        let report = parse_rule(invalid, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{invalid}");
        assert!(!report.is_clean(), "{invalid}");
    }
}
