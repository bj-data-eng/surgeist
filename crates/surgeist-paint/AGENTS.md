# surgeist-paint Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy.

## Boundary and discovery

Paint owns resolved stacking-context membership/child order and non-table CSS
primitive sequencing. The [public front door](src/lib.rs), [stacking models](src/model.rs),
[context algorithm](src/algorithm.rs), [paint-order module](src/paint_order/mod.rs)
and [accounting](src/accounting.rs) own contracts. [README.md](README.md) links
capability and custody documentation. Native resolved inputs do not certify
computed Style; table painting and complete Render scene lowering remain unfinished.

[Stacking tests](src/algorithm/tests.rs) and [paint-order tests](src/paint_order/tests.rs)
cover independent ordering and typed input failures. [Accounting tests](src/paint_order/accounting_tests.rs)
cover granular work and actual-capacity custody. [Layout composition](tests/paint_order_layout.rs)
and [native inline composition](tests/paint_order_native_inline.rs) retain real
producer associations with explicit native CSS facts. [Focused checks](docs/how-to.md#check-the-crate)
run under the root process policy.
