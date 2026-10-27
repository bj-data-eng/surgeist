//! Deep generic test bodies retain their authored payload on an ordinary caller stack.

use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValueLimits, CssComponentValues};
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::normalize::CssNormalizedItem;
use surgeist_css::query::supports::{
    CssSupportsConditionName, CssSupportsTestBody, CssSupportsTestItem,
};
use surgeist_css::rule::{CssRule, CssSupportsConditionRule};

const CHILD: &str = "SURGEIST_NAMED_SUPPORTS_STACK_CHILD";
const TEST: &str = "css::supports::named::named_supports_stack::deep_named_supports_lifecycle_fits_two_mib_caller_stack";
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
    let report = CssParser::default().parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("expected one retained definition");
    };
    assert_chain(rule.body(), chain, ADMITTED_BODY_DEPTH);
    assert_eq!(
        rule.serialize_tokens(Default::default()).unwrap().as_css(),
        source
    );
    assert_eq!(
        rule.serialize_specified(Default::default()).unwrap(),
        expected_specified
    );
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        expected_specified
    );

    let cloned = rule.clone();
    let normalized = report.syntax().normalize(Default::default()).unwrap();
    assert_eq!(normalized.items().len(), 1);
    assert!(
        normalized
            .items()
            .iter()
            .all(|item| matches!(item, CssNormalizedItem::Rule(_)))
    );
    drop(report);
    assert_chain(cloned.body(), chain, ADMITTED_BODY_DEPTH);
    assert_eq!(
        cloned
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    assert_eq!(
        cloned.serialize_specified(Default::default()).unwrap(),
        expected_specified
    );
    drop(cloned);
    drop(normalized);

    // Checked construction must apply the same aggregate structural ceiling.
    let components = CssComponentValues::parse(&CssParser::default(), &body).unwrap();
    let constructed_body =
        CssSupportsTestBody::try_from_components(components, Default::default()).unwrap();
    assert_chain(&constructed_body, chain, ADMITTED_BODY_DEPTH);
    let constructed = CssSupportsConditionRule::try_new(
        CssSupportsConditionName::try_new("--deep").unwrap(),
        constructed_body,
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        constructed.serialize_specified(Default::default()).unwrap(),
        expected_specified
    );
    drop(constructed);

    // A body at depth 256 is independently valid, but the enclosing definition
    // would put the complete rule over the 256-level structural ceiling.
    let too_deep_body = body_source(chain, ADMITTED_BODY_DEPTH + 1);
    let overflow_source =
        format!("@supports-condition --overflow{{{too_deep_body}}}@supports-condition --after{{}}");
    let overflow_report = CssParser::default().parse_sheet(&overflow_source);
    assert!(!overflow_report.is_clean());
    assert!(
        overflow_report.diagnostics().iter().any(|diagnostic| {
            diagnostic
                .error()
                .component_error()
                .is_some_and(|detail| detail.is_nesting_limit())
                || diagnostic.error().code() == CssErrorCode::NestingLimit
        }),
        "expected typed nesting diagnostic: {:?}",
        overflow_report.diagnostics()
    );
    let [CssRule::SupportsCondition(after)] = overflow_report.syntax().rules() else {
        panic!("overdeep definition must be dropped while following rule survives");
    };
    assert_eq!(after.name().as_str(), "--after");
    let components = CssComponentValues::parse(&CssParser::default(), &too_deep_body).unwrap();
    let tighter = CssComponentValueLimits::try_new(255, usize::MAX, usize::MAX).unwrap();
    let error = CssSupportsTestBody::try_from_components(components.clone(), tighter).unwrap_err();
    assert!(
        error
            .component_error()
            .is_some_and(|detail| detail.is_nesting_limit())
    );
    let body_at_ceiling =
        CssSupportsTestBody::try_from_components(components, Default::default()).unwrap();
    let error = CssSupportsConditionRule::try_new(
        CssSupportsConditionName::try_new("--deep").unwrap(),
        body_at_ceiling,
        Default::default(),
    )
    .unwrap_err();
    assert!(
        error
            .component_error()
            .is_some_and(|detail| detail.is_nesting_limit())
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
