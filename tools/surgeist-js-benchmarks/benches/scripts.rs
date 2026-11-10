#![allow(unused_crate_dependencies, missing_docs)]
use criterion::{Criterion, criterion_group, criterion_main};
use std::{path::Path, time::Duration};
use surgeist_js::{
    Context, JsValue, Source, js_string, optimizer::OptimizerOptions, script::Script,
};

fn bench_scripts(c: &mut Criterion) {
    let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts");

    let scripts: Vec<_> = walkdir::WalkDir::new(&scripts_dir)
        .sort_by_file_name()
        .into_iter()
        .map(|entry| entry.expect("read benchmark fixture entry"))
        .filter(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "js")
                && path
                    .file_name()
                    .is_some_and(|base| !base.display().to_string().starts_with("_"))
        })
        .collect();

    for entry in scripts {
        let path = entry.path();
        let code = std::fs::read_to_string(path).unwrap();

        // Create a nice benchmark name from the relative path
        let rel_path = path.strip_prefix(&scripts_dir).unwrap().with_extension("");
        let name = rel_path
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");

        let mut group = c.benchmark_group(&name);
        let is_v8_suite = rel_path.starts_with("v8-benches");
        // Use reduced sample size for slow benchmarks (e.g., v8-benches)
        if is_v8_suite {
            group.sample_size(10);
            group.measurement_time(Duration::from_secs(5));
        }

        let context = &mut Context::builder().build().unwrap();

        // Disable optimizations
        context.set_optimizer_options(OptimizerOptions::empty());

        // Register runtime for console.log support
        surgeist_js_test_host::register(
            surgeist_js_test_host::extensions::ConsoleExtension(surgeist_js_test_host::NullLogger),
            None,
            context,
        )
        .expect("Runtime registration failed");

        // Parse and compile once, outside the benchmark loop
        let script = Script::parse(Source::from_bytes(&code), None, context.access()).unwrap();
        script.codeblock(context.access()).unwrap();

        // Evaluate once to define the main function
        script.evaluate(context.access()).unwrap();

        // Get the main function
        let function = context
            .global_object()
            .unwrap()
            .get(js_string!("main"), context.access())
            .unwrap_or_else(|_| panic!("No main function defined in script: {}", path.display()))
            .as_callable()
            .unwrap_or_else(|| panic!("'main' is not a function in script: {}", path.display()))
            .clone();

        let mut call = || function.call(&JsValue::undefined(), &[], context.access());
        if name == "intl/numberformat-different-options" {
            group.bench_function("Unsupported currency format error", |b| {
                b.iter(|| {
                    let error = call().expect_err("Boa v0.22 has no Intl currency format");
                    let native = error.as_native().expect("native currency format error");
                    assert!(native.is_type() && native.message() == "unimplemented");
                    error
                });
            });
        } else {
            group.bench_function("Execution", |b| {
                b.iter(|| call().expect("execute benchmark script"));
            });
        }
        group.finish();

        if is_v8_suite {
            let failures = context
                .global_object()
                .unwrap()
                .get(js_string!("harnessErrorCount"), context.access())
                .expect("read the V8 harness error count");
            assert_eq!(
                failures.as_number(),
                Some(0.0),
                "{name} failed inside its harness"
            );
        }
    }
}

criterion_group!(benches, bench_scripts);
criterion_main!(benches);
