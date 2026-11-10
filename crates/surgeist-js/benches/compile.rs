//! Measures compilation with a fresh parsed script for each sample iteration.

use std::{cell::RefCell, fs, hint::black_box, path::Path};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use surgeist_js::{Context, Source, optimizer::OptimizerOptions, script::Script};

fn fresh_compilation(criterion: &mut Criterion) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/bench_scripts");
    let mut scripts: Vec<_> = fs::read_dir(fixtures)
        .expect("read compiler benchmark fixtures")
        .map(|entry| entry.expect("read fixture directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "js"))
        .collect();
    scripts.sort_unstable();

    let mut group = criterion.benchmark_group("Fresh compilation");
    for script in scripts {
        let source = fs::read(&script).expect("read compiler benchmark fixture");
        let name = script
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("UTF-8 fixture name");
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(OptimizerOptions::empty());
        let context = RefCell::new(context);

        group.bench_function(name, |bencher| {
            bencher.iter_batched_ref(
                || {
                    Script::parse(
                        Source::from_bytes(&source),
                        None,
                        context.borrow_mut().access(),
                    )
                    .expect("parse compiler benchmark fixture")
                },
                |script| {
                    black_box(
                        script
                            .codeblock(context.borrow_mut().access())
                            .expect("compile fixture"),
                    )
                },
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, fresh_compilation);
criterion_main!(benches);
