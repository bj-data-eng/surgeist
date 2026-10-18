# surgeist-html-fuzz Package Guide

Read the [repository](../../../AGENTS.md), [HTML](../AGENTS.md) and
[shared fuzz](../../surgeist-fuzz/AGENTS.md) guides.

This verification package owns Unicode cases, codecs, captured fixtures, expectations and reductions. Use public
HTML APIs; no private parser copy or fuzz-only production export. Framework owns
engines, worker supervision, artifacts and same-failure reduction.

Read [Cargo.toml](Cargo.toml), [README.md](README.md) and [fuzz.toml](fuzz.toml)
for package features and exact revisioned scope/limits. Preserve
exact UTF16 and chronological typed diagnostics; source offsets need not increase.
New native behavior is not authorized by a fuzz counterexample: retain exact case,
context/limits/failure and return it to the coordinator before a production fix.

## Command Inventory

Run focused tests in default/property/coverage/all-feature modes, strict all-target
all-feature Clippy and formatting through bounded pisct run with offline/locked
Cargo and one build job. No live campaign against dirty source or tool acquisition.
