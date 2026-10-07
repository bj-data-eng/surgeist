#![forbid(unsafe_code)]
//! Original resource ownership and atomic fragment recovery under the shared 256-level ceiling.
use surgeist_css::*;

const PREFIX: &str = "/*😀*/\r\n/*🦀*/";

fn original(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 1);
    assert_eq!(
        position.column().value() as usize,
        source[..offset]
            .rsplit_once("\r\n")
            .unwrap()
            .1
            .encode_utf16()
            .count()
    );
}

fn color(declaration: &CssDeclaration, source: &str, expected: &str) {
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        expected
    );
    original(
        declaration.position().unwrap(),
        source,
        source.find(&format!("color:{expected}")).unwrap(),
    );
    let value = declaration.parsed_value().unwrap();
    assert_eq!(value.source().as_str(), source);
    assert!(
        value
            .source()
            .same_snapshot(declaration.parsed_name().unwrap().source())
    );
}

fn limit(
    diagnostic: &CssRecoveryDiagnostic,
    source: &str,
    start: usize,
    end: usize,
    opening: usize,
    production: &str,
) {
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("native owner retains the typed resource fault")
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(detail.enclosing_production().as_str(), production);
    original(diagnostic.error().position(), source, opening);
    original(diagnostic.span().start(), source, start);
    original(diagnostic.span().end(), source, end);
}

fn failed_block(prelude: &str) -> String {
    // Parent style + rejected block = two levels; function255 first enters257.
    format!("{prelude}{{{}x{}}}", "f(".repeat(255), ")".repeat(255))
}

#[test]
fn independent_failed_rule_and_declaration_components_keep_native_owners_and_partition() {
    for prelude in ["bad,", "]bad,"] {
        let failed = failed_block(prelude);
        // A declaration has only the parent style: function256 first enters257.
        let declaration = format!("--bomb:{}x{};", "g(".repeat(256), ")".repeat(256));
        let source = format!(
            "{PREFIX}.p{{color:red;{failed}color:green;{declaration}color:blue}}tail{{color:black}}"
        );
        let report = parse_sheet(&source);
        let [failed_diagnostic, declaration_diagnostic] = report.diagnostics() else {
            panic!(
                "two independent native resource units: {:?}",
                report.diagnostics()
            )
        };
        let failed_start = source.find(&failed).unwrap();
        limit(
            failed_diagnostic,
            &source,
            failed_start,
            failed_start + failed.len(),
            failed_start + prelude.len() + 1 + 2 * 254,
            "css.qualified-rule",
        );
        let declaration_start = source.find(&declaration).unwrap();
        limit(
            declaration_diagnostic,
            &source,
            declaration_start,
            declaration_start + declaration.len(),
            declaration_start + "--bomb:".len() + 2 * 255,
            "css.declaration",
        );
        let [CssRule::Style(parent), CssRule::Style(tail)] = report.syntax().rules() else {
            panic!("parent and original later root sibling survive")
        };
        let [red] = parent.declarations().as_slice() else {
            panic!("leading red slot")
        };
        color(red, &source, "red");
        let [CssRule::NestedDeclarations(run)] = parent.rules() else {
            panic!("failed complete rule partitions once; declaration loss does not partition")
        };
        let [green, blue] = run.declarations().as_slice() else {
            panic!("later valid declaration siblings")
        };
        color(green, &source, "green");
        color(blue, &source, "blue");
        original(run.position(), &source, source.find("color:green").unwrap());
        original(tail.position(), &source, source.find("tail{").unwrap());
        let [black] = tail.declarations().as_slice() else {
            panic!("tail color")
        };
        color(black, &source, "black");
        assert!(
            red.parsed_value()
                .unwrap()
                .source()
                .same_snapshot(black.parsed_value().unwrap().source())
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

fn rejects<T>(report: &CssParseReport<Option<T>>, source: &str) {
    assert!(report.syntax().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("only outer atomic rejection survives")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    assert_ne!(diagnostic.error().code(), CssErrorCode::NestingLimit);
}

#[test]
fn trailing_fragment_input_prunes_provisional_inner_component_resource_diagnostics() {
    let failed = failed_block("bad,");
    let contents = format!("color:red;{failed}color:green");
    let context = CssNamespaceContext::default();
    let rule_control = format!("{PREFIX}.p{{{contents}}}");
    let block_control = format!("{PREFIX}{{{contents}}}");
    // Real controls confirm recoverable inner syntax before testing rejection.
    let rule = parse_rule(&rule_control, &context);
    let block = parse_style_block(&block_control, &context);
    assert!(rule.syntax().is_some());
    assert!(block.syntax().is_some());
    assert_eq!(rule.diagnostics().len(), 1);
    assert_eq!(block.diagnostics().len(), 1);
    assert_eq!(
        rule.diagnostics()[0].error().code(),
        CssErrorCode::NestingLimit
    );
    assert_eq!(
        block.diagnostics()[0].error().code(),
        CssErrorCode::NestingLimit
    );
    for suffix in [" .second{}", " }"] {
        let source = format!("{rule_control}{suffix}");
        rejects(&parse_rule(&source, &context), &source);
    }
    for suffix in [" {}", " }"] {
        let source = format!("{block_control}{suffix}");
        rejects(&parse_style_block(&source, &context), &source);
    }
}
