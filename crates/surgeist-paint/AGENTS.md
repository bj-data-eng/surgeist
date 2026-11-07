# surgeist-paint Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy.

## Boundary and discovery

Paint currently owns resolved stacking-context membership and child-context
ordering. The [public front door](src/lib.rs), [models](src/model.rs),
[algorithm](src/algorithm.rs) and [accounting](src/accounting.rs) own their
contracts. [README.md](README.md) links the capability and custody documentation.
Full primitive emission, computed Style and Render scene lowering are unfinished.

[Unit tests](src/algorithm/tests.rs) cover independent ordering, validation and
Ledger boundaries. [Layout composition](tests/layout_composition.rs) retains
real completed geometry with explicit native resolved paint facts.
[Focused checks](docs/how-to.md#check-the-crate) run under the root process policy.
