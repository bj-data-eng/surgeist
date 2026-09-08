use surgeist_style::{Resolved, ResolvedWithDiagnostics};

fn main() {
    let _resolved = ResolvedWithDiagnostics {
        resolved: Resolved::new(),
        diagnostics: Vec::new(),
    };
}
