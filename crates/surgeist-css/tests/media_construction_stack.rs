use surgeist_css::{CssMediaCondition, CssMediaQuery, parse_component_values};

#[test]
fn deepest_supported_media_construction_and_serialization_fit_an_ordinary_caller_stack() {
    const CHILD: &str = "SURGEIST_MEDIA_CONSTRUCTION_STACK_CHILD";
    const TEST: &str =
        "deepest_supported_media_construction_and_serialization_fit_an_ordinary_caller_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let source = format!("{}(width: 1px){}", "(".repeat(255), ")".repeat(255));
                let components = parse_component_values(&source).unwrap();
                let condition = CssMediaCondition::try_from_components(components.clone()).unwrap();
                assert_eq!(condition.serialize().unwrap().as_css(), source);
                let query = CssMediaQuery::try_from_components(components).unwrap();
                let cloned = query.clone();
                assert_eq!(query, cloned);
                assert_eq!(cloned.serialize().unwrap().as_css(), source);
                drop(cloned);
                drop(query);
                drop(condition);
                let opaque = format!("(future: {}1{})", "calc(".repeat(255), ")".repeat(255));
                let condition = CssMediaCondition::try_from_components(
                    parse_component_values(&opaque).unwrap(),
                )
                .unwrap();
                let surgeist_css::CssMediaConditionKind::UnknownFeature(unknown) = condition.kind()
                else {
                    panic!("unknown feature with valid numeric value");
                };
                assert_eq!(
                    unknown.reason(),
                    surgeist_css::CssUnknownMediaFeatureReason::UnknownName
                );
                assert_eq!(unknown.serialize().unwrap().as_css(), opaque);
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("media construction, serialization and destruction completed");
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
            .contains("media construction, serialization and destruction completed")
    );
}
