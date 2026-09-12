#![forbid(unsafe_code)]

//! The public 256-level component limit must fit an ordinary 2 MiB stack.
//! Run through the repository's bounded process wrapper: an abort stays in the
//! child, and the wrapper's deadline covers its inherited process group.

use std::process::{Command, Stdio};

use surgeist_css::{
    CssComponentValueErrorKind, CssComponentValueRef, CssValueOrigin, parse_component_values,
};

const CHILD_CASE: &str = "SURGEIST_CSS_COMPONENT_STACK_LIMIT_CHILD";
const COMPLETED: &str = "component stack assertions and resource destruction completed";

fn isolated(test_name: &str, check: fn()) {
    if std::env::var(CHILD_CASE).as_deref() == Ok(test_name) {
        let worker = std::thread::Builder::new()
            .name("component-stack-contract".into())
            .stack_size(2 * 1024 * 1024)
            .spawn(check)
            .expect("start the explicitly sized component parser thread");
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("{COMPLETED}");
        return;
    }

    let output = Command::new(std::env::current_exe().expect("locate this integration test"))
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(CHILD_CASE, test_name)
        .env_remove("RUST_MIN_STACK")
        .stdin(Stdio::null())
        .output()
        .expect("run and reap the isolated component parser child");
    assert!(
        output.status.success(),
        "component parser child failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(COMPLETED),
        "the selected child must execute its assertions and join its worker"
    );
}

#[test]
fn depth_256_parses_serializes_and_drops_on_an_ordinary_stack() {
    isolated(
        "depth_256_parses_serializes_and_drops_on_an_ordinary_stack",
        || {
            let source = format!("{}x{}", "(".repeat(256), ")".repeat(256));
            let values = parse_component_values(&source).expect("256 levels are admitted");
            assert_eq!(values.component_count(), 257);
            assert_eq!(values.nesting_depth(), 256);
            let serialized = values.serialize().expect("serialize the admitted value");
            assert_eq!(serialized.as_css(), source);
            drop(serialized);
            drop(values);
            drop(source);
        },
    );
}

#[test]
fn depth_257_reports_the_first_excess_opening_on_an_ordinary_stack() {
    isolated(
        "depth_257_reports_the_first_excess_opening_on_an_ordinary_stack",
        || {
            let source = format!("{}x{}", "(".repeat(257), ")".repeat(257));
            let error = parse_component_values(&source).expect_err("257 levels exceed the limit");
            assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
            let CssValueOrigin::Parsed(origin) = error.origin() else {
                panic!("the excess opening must retain its source origin");
            };
            assert_eq!(origin.source().as_str(), source);
            assert_eq!(origin.span().start().byte_offset().value(), 256);
            assert_eq!(origin.span().end().byte_offset().value(), 257);
            drop(error);
            drop(source);
        },
    );
}

#[test]
fn mixed_blocks_preserve_eof_closures_and_reject_mismatched_delimiters() {
    let source = "f([{x";
    let values = parse_component_values(source).expect("EOF closes each open block");
    assert_eq!(values.component_count(), 4);
    assert_eq!(values.nesting_depth(), 3);
    assert_eq!(values.serialize().unwrap().as_css(), "f([{x}])");
    let mut children = &values;
    for (start, end) in [(0, 2), (2, 3), (3, 4)] {
        let (closing, nested) = match children.items()[0].view() {
            CssComponentValueRef::Function(value) => (value.closing_origin(), value.values()),
            CssComponentValueRef::Block(value) => (value.closing_origin(), value.values()),
            _ => panic!("expected the next enclosing function or block"),
        };
        let CssValueOrigin::ImplicitClosure { opening, at } = closing else {
            panic!("missing authored delimiters must have implicit origins");
        };
        assert_eq!(opening.span().start().byte_offset().value(), start);
        assert_eq!(opening.span().end().byte_offset().value(), end);
        assert_eq!(at.span().start().byte_offset().value(), source.len());
        assert_eq!(at.span().end().byte_offset().value(), source.len());
        assert!(opening.source().same_snapshot(at.source()));
        children = nested;
    }

    let error = parse_component_values("f([x)").expect_err("a parenthesis cannot close a bracket");
    assert_eq!(
        error.kind(),
        CssComponentValueErrorKind::UnmatchedClosingDelimiter
    );
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("the mismatched delimiter must retain its source origin");
    };
    assert_eq!(origin.span().start().byte_offset().value(), 4);
    assert_eq!(origin.span().end().byte_offset().value(), 5);
}
