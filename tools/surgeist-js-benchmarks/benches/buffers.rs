//! Matched workloads for ordinary and shared buffer operations.

use criterion::{Criterion, criterion_group, criterion_main};
use surgeist_js::{Context, JsValue, Source, js_string, optimizer::OptimizerOptions};

struct Workload {
    name: &'static str,
    source: &'static str,
    expected: i32,
}

const WORKLOADS: &[Workload] = &[
    Workload {
        name: "Ordinary overlapping typed array copy",
        source: r#"
            const values = new Uint8Array(4096);
            values.fill(7);
            function run() {
                values[4095] = 0;
                values.copyWithin(1024, 0, 3072);
                return values[4095];
            }
        "#,
        expected: 7,
    },
    Workload {
        name: "Shared unaligned DataView access",
        source: r#"
            const view = new DataView(new SharedArrayBuffer(64));
            function run() {
                let total = 0;
                for (let offset = 5; offset < 48; offset += 8) {
                    view.setUint32(offset, 0x12345678, true);
                    total += view.getUint32(offset, true);
                }
                return total;
            }
        "#,
        expected: 6 * 0x1234_5678,
    },
    Workload {
        name: "Shared adjacent atomic lanes",
        source: r#"
            const lanes = new Uint16Array(new SharedArrayBuffer(8));
            function run() {
                Atomics.store(lanes, 0, 0);
                Atomics.store(lanes, 1, 0);
                for (let i = 0; i < 128; i++) {
                    Atomics.add(lanes, i & 1, 1);
                }
                return Atomics.load(lanes, 0) * 65536 + Atomics.load(lanes, 1);
            }
        "#,
        expected: 64 * 65_536 + 64,
    },
];

fn buffers(criterion: &mut Criterion) {
    for workload in WORKLOADS {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(OptimizerOptions::empty());
        context
            .eval(Source::from_bytes(workload.source))
            .expect("prepare buffer workload");
        let function = context
            .global_object()
            .unwrap()
            .get(js_string!("run"), context.access())
            .expect("load buffer workload")
            .as_callable()
            .expect("buffer workload is callable");

        let mut run = || {
            function
                .call(&JsValue::undefined(), &[], context.access())
                .expect("execute buffer workload")
        };
        let expected = JsValue::from(workload.expected);
        assert_eq!(run(), expected, "{}", workload.name);
        criterion.bench_function(workload.name, |bencher| bencher.iter(&mut run));
        assert_eq!(run(), expected, "{}", workload.name);
    }
}

criterion_group!(benches, buffers);
criterion_main!(benches);
