# surgeist-fuzz

## Dependencies

This notice covers the direct external dependencies of version 0.1.0 in
[Cargo.toml](Cargo.toml), resolved by the shared [Cargo.lock](../../Cargo.lock),
including optional property, coverage and execution features. The framework
also consumes the repository-owned [surgeist-generator](../surgeist-generator/NOTICE.md)
contracts. Its own [MIT license](LICENSE) remains separate.

This is a source-distribution attribution set. Dependency code is not vendored
in this crate. Transitive dependencies, downloaded executables, compiler tools,
and the standalone controlled fixture's optional libFuzzer dependency are
outside this direct framework inventory. The fixture retains its own
[notice](tests/fixtures/subject/NOTICE.md). A distributed binary needs an
inventory for its actual features and linked material. Builds do not acquire
license texts.

The files below are verbatim copies from the exact cached crate releases.
License alternatives remain available; this notice does not select one.

### arbitrary

This product optionally depends on arbitrary 1.5.0:

* License: [MIT](licenses/arbitrary/LICENSE-MIT) OR [Apache-2.0](licenses/arbitrary/LICENSE-APACHE)
* Homepage: [arbitrary](https://github.com/rust-fuzz/arbitrary/)

### clap

This product optionally depends on clap 4.6.7 for the execution CLI, distributed
by its individual contributors:

* License: [MIT](licenses/clap/LICENSE-MIT) OR [Apache-2.0](licenses/clap/LICENSE-APACHE)
* Homepage: [clap](https://github.com/clap-rs/clap)

### ctrlc

This product optionally depends on ctrlc 3.5.0:

* License: [MIT](licenses/ctrlc/LICENSE-MIT) OR [Apache-2.0](licenses/ctrlc/LICENSE-APACHE)
* Homepage: [ctrlc](https://github.com/Detegr/rust-ctrlc)

### proptest

This product optionally depends on proptest 1.11.0:

* License: [MIT](licenses/proptest/LICENSE-MIT) OR [Apache-2.0](licenses/proptest/LICENSE-APACHE)
* Homepage: [proptest](https://proptest-rs.github.io/proptest/proptest/index.html)

### proptest-state-machine

This product optionally depends on proptest-state-machine 0.8.0:

* License: [MIT](licenses/proptest-state-machine/LICENSE-MIT) OR [Apache-2.0](licenses/proptest-state-machine/LICENSE-APACHE)
* Homepage: [proptest-state-machine](https://proptest-rs.github.io/proptest/proptest/state-machine.html)

### rustix

This product depends on rustix 1.1.4:

* License: [Apache-2.0 WITH LLVM-exception](licenses/rustix/LICENSE-Apache-2.0_WITH_LLVM-exception) OR [Apache-2.0](licenses/rustix/LICENSE-APACHE) OR [MIT](licenses/rustix/LICENSE-MIT)
* Homepage: [rustix](https://github.com/bytecodealliance/rustix)
* Additional material: [COPYRIGHT](licenses/rustix/COPYRIGHT)

### serde

This product depends on serde 1.0.228:

* License: [MIT](licenses/serde/LICENSE-MIT) OR [Apache-2.0](licenses/serde/LICENSE-APACHE)
* Homepage: [serde](https://serde.rs)

### serde_json

This product depends on serde_json 1.0.145:

* License: [MIT](licenses/serde_json/LICENSE-MIT) OR [Apache-2.0](licenses/serde_json/LICENSE-APACHE)
* Homepage: [serde_json](https://github.com/serde-rs/json)

### toml

This product depends on toml 0.9.8:

* License: [MIT](licenses/toml/LICENSE-MIT) OR [Apache-2.0](licenses/toml/LICENSE-APACHE)
* Homepage: [toml](https://github.com/toml-rs/toml)

Keep this notice and its license files together. Changes to dependency versions
or the distributed feature/target set require comparison with the selected
upstream material.
