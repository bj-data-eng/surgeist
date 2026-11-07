# Render fuzz case guide

Read the [Render guide](../AGENTS.md) and shared
[Fuzz contracts](../../surgeist-fuzz/AGENTS.md).

This package owns concrete public headless cases, strict canonical codecs,
independent expectations, seeds and controlled detection tests. It consumes
Render front doors and never creates a renderer/device/window. The
[README](README.md) defines checked capabilities, exclusions, provenance and
focused checks; [fuzz.toml](fuzz.toml) registers targets and bounds.

`src/resources.rs` owns resource cases; `src/scenes.rs` owns authored structural
inspection and concrete geometry cases. `src/codec.rs` owns transport only;
semantic product rejection belongs to each evaluator. Workers use the same
case/evaluator for replay, property and optional coverage engines. Ordinary
builds do not link the coverage engine. Product code does not depend here.
