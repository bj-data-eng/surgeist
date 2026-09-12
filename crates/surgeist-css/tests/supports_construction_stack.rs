#![forbid(unsafe_code)]
//! Public construction, projection and owned-tree operations at supported depth.
use surgeist_css::{
    CssNamespaceContext, CssSupportsCondition, CssSupportsConditionKind, parse_component_values,
};

fn construct(source: &str) -> CssSupportsCondition {
    CssSupportsCondition::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

#[test]
fn supported_supports_depth_and_owned_child_lifetimes_fit_an_ordinary_caller_stack() {
    const CHILD: &str = "SURGEIST_SUPPORTS_CONSTRUCTION_STACK_CHILD";
    const TEST: &str =
        "supported_supports_depth_and_owned_child_lifetimes_fit_an_ordinary_caller_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                // The public component limit permits 256 nested enclosures.
                let grouped = format!("{}(width:1px){}", "(".repeat(255), ")".repeat(255));
                let negated = format!("{}(width:1px){}", "not (".repeat(255), ")".repeat(255));
                let opaque = format!("{}1{}", "Future(".repeat(256), ")".repeat(256));
                let numeric = format!("(opacity:{}1{})", "calc(".repeat(255), ")".repeat(255));
                let mut conjunction = "(width:1px)".to_owned();
                for _ in 0..255 {
                    conjunction = format!("({conjunction} and future())");
                }
                for source in [grouped, negated, opaque, numeric, conjunction] {
                    let condition = construct(&source);
                    let cloned = condition.clone();
                    assert_eq!(condition, cloned);
                    assert_eq!(condition.serialize().unwrap().as_css(), source);
                    assert_eq!(cloned.serialize().unwrap().as_css(), source);
                    drop(condition);
                    assert_eq!(cloned.serialize().unwrap().as_css(), source);
                    drop(cloned);
                }
                let child_source = format!("{}(width:1px){}", "not (".repeat(253), ")".repeat(253));
                let parent = construct(&format!("({child_source}) and future()"));
                let CssSupportsConditionKind::And(children) = parent.kind() else {
                    panic!("conjunction")
                };
                let child = children.conditions()[0].clone();
                let expected = format!("({child_source})");
                assert_eq!(child.serialize().unwrap().as_css(), expected);
                drop(parent);
                let copy = child.clone();
                assert_eq!(copy, child);
                assert_eq!(copy.serialize().unwrap().as_css(), expected);
                drop(child);
                drop(copy);
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("supports construction, serialization and destruction completed");
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
        String::from_utf8_lossy(&output.stdout)
            .contains("supports construction, serialization and destruction completed")
    );
}
