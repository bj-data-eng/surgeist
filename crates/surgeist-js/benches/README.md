# JavaScript engine benchmarks

The `full` benchmark has a realm creation case and three workloads per
[fixture](bench_scripts/).

- **Parser** lexes and parses the source into a script.
- **Compiler** looks up a script's bytecode. The first call compiles it and
  later calls use the cached code block. The label matches the Boa v0.22
  reference suite.
- **Execution** evaluates the already-compiled script in the VM.

Use the `compile` benchmark to measure fresh bytecode compilation. It parses
a new script outside the timer for each timed compilation.

See the [performance baseline procedure](../../../tools/surgeist-js-test262/baseline/README.md#performance)
for the comparison commands, sample settings, and separate script suite.
