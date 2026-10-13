# surgeist-fuzz

Follow the [repository guide](../../AGENTS.md). This crate owns shared fuzz
contracts, bounded generation mechanics, safe process execution, immutable
evidence, same-failure reduction and local campaign control. Semantic cases,
expectations, codecs, domain reductions and requirement references stay in the
consuming crate's `fuzz/` package. Generic forests here are fixture topology,
not a DOM or a layout model. Benchmarking is outside the current boundary.

Production crates do not depend on this crate. Its optional property and
coverage features provide authoring adapters; ordinary builds do not link
libFuzzer. The controlled author fixture under `tests/fixtures/subject/` is a
standalone test workspace, with its own lockfile and no production membership.
It verifies the implementor workflow without adding product requirements.

## Focused checks

Use one Cargo job and the repository process discipline. Framework checks are:

```sh
cargo test --offline --locked -j 1 -p surgeist-fuzz
cargo test --offline --locked -j 1 -p surgeist-fuzz --all-features
cargo clippy --offline --locked -j 1 -p surgeist-fuzz --all-targets --all-features -- -F unsafe-code -D warnings
```

Select campaign integration and real sanitizer coverage separately as described
in the [contributor guide](README.md). They compile isolated source bases and
control descendants; never point their output at another worker's checkout or
target directory. Use a clean source revision and explicit finite budgets.
Continuous coverage is an explicit single-worker mode composed of finite engine
rounds; tests must arrange bounded cancellation. Mutable session retention is
separate from immutable failure evidence.

Failure identity is the declared property, class and stable discriminator.
Unqualified crashes/panics cannot establish same-bug minimization. Retain exact
canonical bytes before evaluation; never call a seed or a libFuzzer entropy
input a concrete structured case. Missing evidence remains explicit.
