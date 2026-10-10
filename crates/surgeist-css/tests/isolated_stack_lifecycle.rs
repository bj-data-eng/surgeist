#![forbid(unsafe_code)]
//! Real subprocess characterization of the shared stack runner. The worker's
//! output and destruction, rather than elapsed time, establish completion.

#[path = "support/isolated_stack.rs"]
mod isolated_stack;

const CHILD: &str = "SURGEIST_STACK_HELPER_CONTRACT_CHILD";
const COMPLETED: &str = "helper worker joined and resources destroyed";

#[test]
fn selected_child_runs_once_and_completes_after_resource_destruction() {
    const TEST: &str = "selected_child_runs_once_and_completes_after_resource_destruction";
    let output = isolated_stack::run(TEST, CHILD, COMPLETED, Some("helper-contract"), || {
        assert!(std::env::var_os("RUST_MIN_STACK").is_none());
        assert_eq!(std::thread::current().name(), Some("helper-contract"));
        let mut input = String::new();
        assert_eq!(std::io::stdin().read_line(&mut input).unwrap(), 0);
        struct Resource;
        impl Drop for Resource {
            fn drop(&mut self) {
                println!("owned resource destroyed");
            }
        }
        let _resource = Resource;
        println!("selected operation executed");
    });
    if let Some(output) = output {
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.matches("selected operation executed").count(), 1);
        assert_eq!(stdout.matches("owned resource destroyed").count(), 1);
        assert_eq!(stdout.matches(COMPLETED).count(), 1);
        assert!(stdout.find("owned resource destroyed").unwrap() < stdout.find(COMPLETED).unwrap());
        assert!(stdout.contains("1 passed; 0 failed"), "{stdout}");
    }
}

#[test]
fn worker_panic_is_propagated_by_selected_child() {
    const TEST: &str = "worker_panic_is_propagated_by_selected_child";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        isolated_stack::run(TEST, CHILD, COMPLETED, None, || {
            println!("failing operation stdout");
            panic!("worker panic diagnostic");
        });
    }
}

#[test]
fn failing_child_is_reaped_and_exposes_its_stdout_and_panic_diagnostic() {
    let panic = std::panic::catch_unwind(|| {
        isolated_stack::run(
            "worker_panic_is_propagated_by_selected_child",
            CHILD,
            COMPLETED,
            None,
            || panic!("parent must reexecute selected test"),
        );
    })
    .expect_err("nonzero selected child must fail the parent");
    let message = panic
        .downcast_ref::<String>()
        .expect("runner status diagnostic");
    assert!(
        message.contains("isolated stack child failed:"),
        "{message}"
    );
    assert!(message.contains("failing operation stdout"), "{message}");
    assert!(message.contains("worker panic diagnostic"), "{message}");
    assert!(message.contains("0 passed; 1 failed"), "{message}");
    assert!(!message.contains(COMPLETED), "{message}");
}
