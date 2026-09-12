//! The public depth budget applies to complete mathematical expressions,
//! including operator nodes between nested function components.
use surgeist_css::{
    CssCalcLength, CssLengthPercentageCalculation, CssNumberCalculation, parse_component_values,
};

#[test]
fn mixed_operator_calculations_serialize_at_the_supported_depth() {
    isolated(
        "mixed_operator_calculations_serialize_at_the_supported_depth",
        || {
            let source = format!("{}1{}", "calc(1 + 2 * ".repeat(256), ")".repeat(256));
            let calculation =
                CssNumberCalculation::try_from_components(parse_component_values(&source).unwrap())
                    .unwrap();
            assert_eq!(calculation.serialize().unwrap().as_css(), source);
            drop(calculation);
        },
    );
}

#[test]
fn mixed_operator_length_fragments_serialize_at_the_supported_depth() {
    isolated(
        "mixed_operator_length_fragments_serialize_at_the_supported_depth",
        || {
            let source = format!("{}1px{}", "calc(1px + 2 * ".repeat(256), ")".repeat(256));
            let calculation = CssCalcLength::Typed(
                CssLengthPercentageCalculation::try_from_components(
                    parse_component_values(&source).unwrap(),
                )
                .unwrap(),
            );
            assert_eq!(calculation.to_css_string(), source);
            drop(calculation);
        },
    );
}

fn isolated(test: &str, operation: fn()) {
    const CHILD: &str = "SURGEIST_NUMERIC_SERIALIZATION_STACK_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(test) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(operation)
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("mixed numeric serialization and destruction completed");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(CHILD, test)
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
            .contains("mixed numeric serialization and destruction completed")
    );
}
