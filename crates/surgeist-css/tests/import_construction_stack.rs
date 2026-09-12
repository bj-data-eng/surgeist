#![forbid(unsafe_code)]
//! Checked imports retain usable owned children at the public component depth.
use surgeist_css::{CssImportRule, CssNamespaceContext, parse_component_values};

fn construct(source: &str) -> CssImportRule {
    CssImportRule::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

#[test]
fn supported_import_construction_and_child_lifetimes_fit_an_ordinary_caller_stack() {
    const CHILD: &str = "SURGEIST_IMPORT_CONSTRUCTION_STACK_CHILD";
    const TEST: &str =
        "supported_import_construction_and_child_lifetimes_fit_an_ordinary_caller_stack";
    if std::env::var(CHILD).as_deref() == Ok(TEST) {
        let worker = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let opaque = format!("{}1{}", "Future(".repeat(256), ")".repeat(256));
                let supports_numeric = format!(
                    "supports(opacity:{}1{})",
                    "calc(".repeat(255),
                    ")".repeat(255)
                );
                let supports_not = format!(
                    "supports({}(width:1px){})",
                    "not (".repeat(254),
                    ")".repeat(254)
                );
                let grouped_media = format!("{}(width: 1px){}", "(".repeat(255), ")".repeat(255));
                for tail in [opaque, supports_numeric, supports_not, grouped_media] {
                    let source = format!("@import 'x' {tail};");
                    let import = construct(&source);
                    let copy = import.clone();
                    assert_eq!(import, copy);
                    assert_eq!(import.serialize().unwrap().as_css(), source);
                    assert_eq!(copy.serialize().unwrap().as_css(), source);
                    drop(import);
                    assert_eq!(copy.serialize().unwrap().as_css(), source);
                    drop(copy);
                }
                let expression = format!("{}1{}", "calc(".repeat(255), ")".repeat(255));
                let source = format!("@import 'x' supports(opacity:{expression}) print;");
                let import = construct(&source);
                let supports = import.supports().unwrap().condition().clone();
                let media = import.media().unwrap().clone();
                drop(import);
                let copy = supports.clone();
                assert_eq!(copy, supports);
                assert_eq!(
                    supports.serialize().unwrap().as_css(),
                    format!("(opacity:{expression})")
                );
                assert_eq!(media.serialize().unwrap().as_css(), "print");
                drop(copy);
                drop(supports);
                drop(media);
            })
            .unwrap();
        if let Err(panic) = worker.join() {
            std::panic::resume_unwind(panic);
        }
        println!("import construction, serialization and destruction completed");
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
            .contains("import construction, serialization and destruction completed")
    );
}
