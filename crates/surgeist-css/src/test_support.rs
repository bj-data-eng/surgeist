use crate::{CssParseReport, Error};

/// Clean-report assertions for crate-private tests.
pub(crate) trait CssParseReportTestExt<T> {
    fn unwrap(self) -> T;
    fn expect(self, message: &str) -> T;
    fn expect_err(self, message: &str) -> Error;
}

impl<T> CssParseReportTestExt<T> for CssParseReport<T> {
    fn unwrap(self) -> T {
        self.expect("stylesheet report contained recovery diagnostics")
    }

    fn expect(self, message: &str) -> T {
        let (syntax, diagnostics) = self.into_parts();
        assert!(diagnostics.is_empty(), "{message}: {diagnostics:?}");
        syntax
    }

    fn expect_err(self, message: &str) -> Error {
        let (_, diagnostics) = self.into_parts();
        diagnostics
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{message}"))
            .error()
            .clone()
    }
}

/// Runs `operation` alone in a re-executed child of this test binary, so its
/// process-global allocation counters see no other test.
///
/// libtest exits successfully when `--exact` selects no test, so the parent
/// also requires the `completed` line that the child prints only after
/// `operation` returns. A renamed or misspelled `test` therefore fails.
#[track_caller]
pub(crate) fn run_in_isolated_child(
    test: &str,
    child_environment: &str,
    completed: &str,
    operation: impl FnOnce(),
) {
    if std::env::var(child_environment).as_deref() == Ok(test) {
        operation();
        println!("{completed}");
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env(child_environment, test)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run and reap the isolated child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "isolated child failed: {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status,
    );
    assert!(
        stdout.contains(completed),
        "the isolated child must select `{test}` and complete it\nstdout:\n{stdout}\nstderr:\n{stderr}",
    );
}
