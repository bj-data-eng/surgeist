//! Cascade 5 optional clauses compose with MQ4/MQ5 general-enclosed grammar.
//! The adopted profile prefers present clauses only among complete derivations.
//! Sources: pinned Cascade 5 #at-import/#layer-names and MQ5 #mq-syntax.
use surgeist_css::{
    CssErrorCode, CssImportLayer, CssImportRule, CssImportTarget, CssMediaConditionKind,
    CssMediaQuery, CssMediaType, CssRecoveryAction, CssRule, CssSupportsConditionKind, parse_sheet,
    validate_sheet,
};

fn source(tail: &str) -> String {
    format!("@import url(test) {tail}; .after {{ color: red; }}")
}

fn clean_import(tail: &str) -> CssImportRule {
    let source = source(tail);
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{tail}: {:?}", report.diagnostics());
    assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
    let [CssRule::Import(import), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("import and following style rule: {tail}");
    };
    assert!(matches!(import.target(), CssImportTarget::Url(url) if url.as_str() == "test"));
    import.clone()
}

fn opaque_media(import: &CssImportRule, authored: &str) {
    let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
        panic!("one condition-only media query");
    };
    let CssMediaConditionKind::GeneralEnclosed(enclosed) = condition.kind() else {
        panic!("one general-enclosed media operand");
    };
    assert_eq!(enclosed.authored(), Some(authored));
}

fn named_layer(import: &CssImportRule, expected: &str) {
    assert!(
        matches!(import.layer(), Some(CssImportLayer::Named(name)) if name.components() == [expected])
    );
}

fn declaration_supports(import: &CssImportRule) {
    let CssSupportsConditionKind::Declaration(declaration) =
        import.supports().unwrap().condition().kind()
    else {
        panic!("authored declaration test");
    };
    assert_eq!(declaration.property(), "display");
    assert_eq!(declaration.authored(), "display:grid");
}

#[test]
fn empty_optional_functions_have_complete_absent_clause_media_derivations() {
    for tail in ["layer()", "supports()"] {
        let import = clean_import(tail);
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        opaque_media(&import, tail);
    }
}

#[test]
fn dimension_only_optional_functions_are_opaque_media() {
    for tail in ["layer( 2px )", "supports( 2px )"] {
        let import = clean_import(tail);
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        opaque_media(&import, tail);
    }
}

#[test]
fn invalid_specific_clause_bodies_do_not_forbid_other_complete_derivations() {
    for tail in ["layer(initial)", "layer(inherit)", "supports(not)"] {
        let import = clean_import(tail);
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        opaque_media(&import, tail);
    }
    // The same keyword is still prohibited when used as an actual layer name.
    let report = parse_sheet("@layer initial {} .after {color:red}");
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}

#[test]
fn specific_clauses_win_when_their_entire_derivation_matches() {
    let layer = clean_import("layer(theme)");
    named_layer(&layer, "theme");
    assert!(layer.supports().is_none());
    assert!(layer.media().is_none());

    let supports = clean_import("supports(display:grid)");
    assert!(supports.layer().is_none());
    declaration_supports(&supports);
    assert!(supports.media().is_none());

    let both = clean_import("layer(theme) supports(display:grid) print");
    named_layer(&both, "theme");
    declaration_supports(&both);
    assert!(
        matches!(both.media().unwrap().queries(), [CssMediaQuery::Typed(query)] if query.media_type() == CssMediaType::Print)
    );
}

#[test]
fn a_valid_prefix_clause_is_omitted_when_only_full_media_grammar_matches() {
    for prefix in ["layer(theme)", "supports(display:grid)"] {
        let import = clean_import(&format!("{prefix} and (color)"));
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
            panic!("condition-only query");
        };
        let CssMediaConditionKind::And(children) = condition.kind() else {
            panic!("complete media conjunction");
        };
        let [opaque, color] = children.conditions() else {
            panic!("two operands");
        };
        assert!(
            matches!(opaque.kind(), CssMediaConditionKind::GeneralEnclosed(value) if value.authored() == Some(prefix))
        );
        assert!(matches!(color.kind(), CssMediaConditionKind::Feature(_)));
    }
}

#[test]
fn later_clause_shaped_functions_are_media_after_specific_clauses() {
    let supports = clean_import("supports(display:grid) layer(theme)");
    assert!(supports.layer().is_none());
    declaration_supports(&supports);
    opaque_media(&supports, "layer(theme)");

    let repeated = clean_import("layer(theme) layer(other)");
    named_layer(&repeated, "theme");
    assert!(repeated.supports().is_none());
    opaque_media(&repeated, "layer(other)");

    let repeated = clean_import("supports(display:grid) supports(color:red)");
    assert!(repeated.layer().is_none());
    declaration_supports(&repeated);
    opaque_media(&repeated, "supports(color:red)");
}

#[test]
fn clause_shaped_functions_are_valid_operands_inside_typed_media_queries() {
    for operand in ["layer(theme)", "supports(x)"] {
        let import = clean_import(&format!("screen and {operand}"));
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        let [CssMediaQuery::Typed(query)] = import.media().unwrap().queries() else {
            panic!("typed media query");
        };
        assert_eq!(query.media_type(), CssMediaType::Screen);
        assert!(
            matches!(query.condition().unwrap().kind(), CssMediaConditionKind::GeneralEnclosed(value) if value.authored() == Some(operand))
        );
    }
}

#[test]
fn exhausted_alternatives_recover_media_once_after_the_first_valid_clauses() {
    let report = parse_sheet(&source("layer(theme) screen and, print"));
    let [CssRule::Import(import), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("retained import and following rule");
    };
    named_layer(import, "theme");
    assert!(
        matches!(import.media().unwrap().queries(), [CssMediaQuery::Never(_), CssMediaQuery::Typed(query)] if query.media_type() == CssMediaType::Print)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("only the chosen media member recovers");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
}

#[test]
fn reserved_bare_media_identifiers_still_recover_after_an_anonymous_layer_clause() {
    let report = parse_sheet(&source("layer layer"));
    let [CssRule::Import(import), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("import with media recovery");
    };
    assert!(matches!(import.layer(), Some(CssImportLayer::Anonymous)));
    assert!(matches!(
        import.media().unwrap().queries(),
        [CssMediaQuery::Never(_)]
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("one malformed media member");
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
}

#[test]
fn escaped_and_cased_optional_names_keep_original_media_origins() {
    for tail in [r"l\61yer()", "SUPPORTS(2px)"] {
        let import = clean_import(tail);
        opaque_media(&import, tail);
        let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
            panic!("one media condition");
        };
        assert_eq!(condition.position().unwrap().byte_offset().value(), 18);
    }
}

#[test]
fn failed_clause_probes_do_not_duplicate_retained_eof_closures() {
    for tail in ["layer(", "supports("] {
        let report = parse_sheet(&format!("@import url(test) {tail}"));
        let [CssRule::Import(import)] = report.syntax().rules() else {
            panic!("retained import at EOF");
        };
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        opaque_media(import, tail);
        let [diagnostic] = report.diagnostics() else {
            panic!("one real implicit closure");
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert!(validate_sheet(&format!("@import url(test) {tail}")).is_err());
    }
}

#[test]
fn eof_closures_do_not_prefer_a_grammatically_incomplete_prefix_interpretation() {
    let report = parse_sheet("@import url(test) layer(theme) and (color");
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("retained import with complete media grammar");
    };
    assert!(import.layer().is_none());
    assert!(import.supports().is_none());
    let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
        panic!("complete media condition, not recovery");
    };
    let CssMediaConditionKind::And(operands) = condition.kind() else {
        panic!("complete media conjunction");
    };
    let [layer, color] = operands.conditions() else {
        panic!("both original operands");
    };
    assert!(
        matches!(layer.kind(), CssMediaConditionKind::GeneralEnclosed(value) if value.authored() == Some("layer(theme)"))
    );
    assert!(matches!(
        color.kind(),
        CssMediaConditionKind::Feature(surgeist_css::CssMediaFeatureQuery::Boolean(
            surgeist_css::CssMediaFeatureKind::Color
        ))
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("one actual EOF closure, no failed-probe diagnostic");
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
}

#[test]
fn lexical_errors_remain_terminal_across_optional_clause_probes() {
    for (tail, token) in [("layer(url(a b))", "url(a b)"), ("supports(])", "]")] {
        let source = source(tail);
        let report = parse_sheet(&source);
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
        let [diagnostic] = report.diagnostics() else {
            panic!("one terminal lexical failure");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidComponentValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(token).unwrap()
        );
        let surgeist_css::ErrorKind::InvalidComponentValue(error) = diagnostic.error().kind()
        else {
            panic!("original typed component error");
        };
        let surgeist_css::CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("original parsed origin");
        };
        assert_eq!(origin.span().start(), diagnostic.error().position());
    }
}

#[test]
fn excessive_nesting_stops_the_import_without_erasing_its_resource_error() {
    let tail = format!("{}x{}", "future(".repeat(257), ")".repeat(257));
    let report = parse_sheet(&source(&tail));
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    let [diagnostic] = report.diagnostics() else {
        panic!("one terminal resource failure");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        18 + 256 * 7
    );
}
