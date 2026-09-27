#![forbid(unsafe_code)]
//! Deep generic test bodies retain their authored payload on an ordinary caller stack.

use surgeist_css::{
    CssComponentValueErrorKind, CssComponentValueLimits, CssNamedSupportsConstructionError,
    CssNormalizedItem, CssRule, CssSupportsConditionName, CssSupportsConditionRule,
    CssSupportsTestBody, CssSupportsTestItem, ErrorKind, normalize_sheet, parse_component_values,
    parse_sheet,
};

const CHILD: &str = "SURGEIST_NAMED_SUPPORTS_STACK_CHILD";
const TEST: &str = "deep_named_supports_lifecycle_fits_two_mib_caller_stack";
const ADMITTED_BODY_DEPTH: usize = 255;

#[derive(Clone, Copy)]
enum Chain {
    Qualified,
    AtRule,
}

impl Chain {
    fn opening(self) -> &'static str {
        match self {
            Self::Qualified => "&{",
            Self::AtRule => "@probe{",
        }
    }

    fn specified_opening(self) -> &'static str {
        match self {
            Self::Qualified => "& { ",
            Self::AtRule => "@probe { ",
        }
    }
}

fn body_source(chain: Chain, depth: usize) -> String {
    format!(
        "{}future:yes;{}",
        chain.opening().repeat(depth),
        "}".repeat(depth)
    )
}

fn specified_rule(chain: Chain, depth: usize) -> String {
    format!(
        "@supports-condition --deep {{ {}future: yes;{} }}",
        chain.specified_opening().repeat(depth),
        " }".repeat(depth)
    )
}

fn assert_chain(body: &CssSupportsTestBody, chain: Chain, depth: usize) {
    let mut current = body;
    for level in 0..depth {
        let [item] = current.items() else {
            panic!("expected one generic test at level {level}");
        };
        current = match (chain, item) {
            (Chain::Qualified, CssSupportsTestItem::QualifiedRule(rule)) => rule.body(),
            (Chain::AtRule, CssSupportsTestItem::AtRule(rule)) => {
                assert_eq!(rule.name(), "probe");
                rule.body().expect("nested at-rule block")
            }
            _ => panic!("unexpected generic test at level {level}"),
        };
    }
    let [CssSupportsTestItem::Declarations(run)] = current.items() else {
        panic!("expected one retained leaf declaration");
    };
    let [declaration] = run.declarations() else {
        panic!("expected one retained leaf declaration");
    };
    assert_eq!(declaration.property(), "future");
}

fn assert_nested_lifecycle(chain: Chain) {
    // The outer definition consumes one structural level, leaving 255 generic blocks.
    let body = body_source(chain, ADMITTED_BODY_DEPTH);
    let source = format!("@supports-condition --deep{{{body}}}");
    let expected_specified = specified_rule(chain, ADMITTED_BODY_DEPTH);
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("expected one retained definition");
    };
    assert_chain(rule.body(), chain, ADMITTED_BODY_DEPTH);
    assert_eq!(rule.serialize().unwrap().as_css(), source);
    assert_eq!(rule.to_specified_css().unwrap(), expected_specified);
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        expected_specified
    );

    let cloned = rule.clone();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(normalized.items().len(), 1);
    assert!(
        normalized
            .items()
            .iter()
            .all(|item| matches!(item, CssNormalizedItem::Rule(_)))
    );
    drop(report);
    assert_chain(cloned.body(), chain, ADMITTED_BODY_DEPTH);
    assert_eq!(cloned.serialize().unwrap().as_css(), source);
    assert_eq!(cloned.to_specified_css().unwrap(), expected_specified);
    drop(cloned);
    drop(normalized);

    // Checked construction must apply the same aggregate structural ceiling.
    let components = parse_component_values(&body).unwrap();
    let constructed_body = CssSupportsTestBody::try_from_components(components).unwrap();
    assert_chain(&constructed_body, chain, ADMITTED_BODY_DEPTH);
    let constructed = CssSupportsConditionRule::try_new(
        CssSupportsConditionName::try_new("--deep").unwrap(),
        constructed_body,
    )
    .unwrap();
    assert_eq!(constructed.to_specified_css().unwrap(), expected_specified);
    drop(constructed);

    // A body at depth 256 is independently valid, but the enclosing definition
    // would put the complete rule over the 256-level structural ceiling.
    let too_deep_body = body_source(chain, ADMITTED_BODY_DEPTH + 1);
    let overflow_source =
        format!("@supports-condition --overflow{{{too_deep_body}}}@supports-condition --after{{}}");
    let overflow_report = parse_sheet(&overflow_source);
    assert!(!overflow_report.is_clean());
    assert!(
        overflow_report.diagnostics().iter().any(|diagnostic| {
            matches!(
                diagnostic.error().kind(),
                ErrorKind::InvalidComponentValue(detail)
                    if detail.kind() == CssComponentValueErrorKind::NestingLimit
            ) || matches!(diagnostic.error().kind(), ErrorKind::NestingLimit(_))
        }),
        "expected typed nesting diagnostic: {:?}",
        overflow_report.diagnostics()
    );
    let [CssRule::SupportsCondition(after)] = overflow_report.syntax().rules() else {
        panic!("overdeep definition must be dropped while following rule survives");
    };
    assert_eq!(after.name().as_str(), "--after");
    let components = parse_component_values(&too_deep_body).unwrap();
    let tighter = CssComponentValueLimits::try_new(255, usize::MAX, usize::MAX).unwrap();
    let error = CssSupportsTestBody::try_from_components_with_limits(components.clone(), tighter)
        .unwrap_err();
    assert!(
        matches!(error, CssNamedSupportsConstructionError::Component(ref detail) if detail.kind() == CssComponentValueErrorKind::NestingLimit)
    );
    let body_at_ceiling = CssSupportsTestBody::try_from_components(components).unwrap();
    let error = CssSupportsConditionRule::try_new(
        CssSupportsConditionName::try_new("--deep").unwrap(),
        body_at_ceiling,
    )
    .unwrap_err();
    assert!(
        matches!(error, CssNamedSupportsConstructionError::Component(ref detail) if detail.kind() == CssComponentValueErrorKind::NestingLimit)
    );
}

#[test]
fn deep_named_supports_lifecycle_fits_two_mib_caller_stack() {
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                assert_nested_lifecycle(Chain::Qualified);
                assert_nested_lifecycle(Chain::AtRule);
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("named supports deep lifecycle completed");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD, TEST)
        .env_remove("RUST_MIN_STACK")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("named supports deep lifecycle completed")
    );
}
