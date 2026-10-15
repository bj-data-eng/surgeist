#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "fixtures/template-generated-consumer/src/contract.rs"]
mod contract;

static NEXT_INVOCATION: AtomicU64 = AtomicU64::new(0);

struct Invocation(PathBuf);

impl Invocation {
    fn new() -> Self {
        let base = std::env::temp_dir();
        loop {
            let serial = NEXT_INVOCATION.fetch_add(1, Ordering::Relaxed);
            let path = base.join(format!(
                "surgeist-template-consumer-{}-{serial}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("cannot create {}: {error}", path.display()),
            }
        }
    }
}

impl Drop for Invocation {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0)
            .unwrap_or_else(|error| panic!("cannot clean {}: {error}", self.0.display()));
    }
}

fn consume(input: &Path) -> Output {
    // Run the owning test through `pisct run`: this ordinary child and its rustc
    // descendants stay in that bounded process group. Its target is independent
    // of the outer Cargo invocation, so no parent/child target lock is shared.
    Command::new(env!("CARGO"))
        .args(["run", "--offline", "--locked", "-j", "1", "--quiet"])
        .arg("--manifest-path")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/template-generated-consumer/Cargo.toml"
        ))
        .arg("--target-dir")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/template-generated-consumer"
        ))
        .env("SURGEIST_TEMPLATE_GENERATED_INPUT", input)
        .output()
        .expect("execute the already installed Cargo for the compiled consumer")
}

fn diagnostics(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn generated_rust_constructs_revalidatable_symbolic_template_values() {
    let authored = surgeist::template::parse_template(contract::SOURCE).expect("strict template");
    contract::verify(&authored);
    let (native, components) = contract::registries();
    let validated = surgeist::template::validate_template(&authored, &native, &components)
        .expect("explicit registries validate the authored template");
    let generated = surgeist::template::render_to_rust(&validated);
    let invocation = Invocation::new();

    // Establish that the fixed scaffold, published APIs and independent literal
    // assertions actually compile and execute before consuming the product artifact.
    let control_path = invocation.0.join("control.rs");
    fs::write(
        &control_path,
        format!(
            "::surgeist::template::parse_template({:?}).expect(\"control template\")",
            contract::SOURCE
        ),
    )
    .expect("write parser control expression");
    let control = consume(&control_path);
    assert!(
        control.status.success(),
        "compiled consumer control failed before product-artifact testing:\n{}",
        diagnostics(&control)
    );
    println!(
        "compiled consumer control passed: {}",
        diagnostics(&control)
    );

    let generated_path = invocation.0.join("generated.rs");
    fs::write(&generated_path, &generated).expect("write exact renderer output");
    let actual = consume(&generated_path);
    assert!(
        actual.status.success(),
        "renderer artifact must construct the authored TemplateDocument and pass its real consumer\n{}\nexact generated artifact:\n{generated}",
        diagnostics(&actual)
    );
    println!(
        "compiled renderer artifact passed: {}",
        diagnostics(&actual)
    );
}
