# surgeist-paint

Paint plans stacking-context membership and immediate child-context order from
explicitly resolved rendering-box facts. It owns iterative topology validation,
flex/grid order-modified traversal and shared context classification. Ledger
owns work admission, scratch/result custody and optional phase profiling.

This is a partial paint stage. A context tree does not emit the complete CSS2
Appendix E primitive sequence, execute effects, construct computed Style or
produce a Render scene. Callers retain their real box identities and geometry.

## Start

See [getting started](docs/getting-started.md) for a two-box example.
Run the focused public composition test from the repository root:

```sh
cargo test --offline --locked -j 1 -p surgeist-paint --test layout_composition
```

The test runs real Layout and associates its completed geometry with explicitly
supplied native paint facts; it does not authenticate computed Style.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Construct and inspect a context plan. |
| [How-to](docs/how-to.md) | Set budgets, inspect failure reports and run focused checks. |
| [Reference](docs/reference.md) | Find types, dimensions and source boundaries. |
| [Explanation](docs/explanation.md) | Understand ordering, identity and storage custody. |

## License and attribution

Authored in this repository under the [MIT License](../../LICENSE).
The [repository notice](../../NOTICE.md) records dependency provenance.
