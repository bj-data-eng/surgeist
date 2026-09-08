# Getting started

This guide connects an existing Rust application to a local `surgeist-css`
checkout and demonstrates retained CSS syntax with a recovery diagnostic.

## Prerequisites

Use an existing Cargo project, a local Surgeist checkout, and a Rust
toolchain supporting the crate's edition 2024. The [manifest](../Cargo.toml)
owns the package and feature declarations. Offline commands require their
dependencies to be available in your local Cargo cache.

## Parse a stylesheet

1. In the consuming project's `Cargo.toml`, add a path dependency. This example
   assumes the consumer and `surgeist-css` are sibling directories; use the
   actual relative path for your checkout.

   ```toml
   [dependencies]
   surgeist-css = { path = "../surgeist/crates/surgeist-css" }
   ```

2. Put this program in the consumer's `src/main.rs`:

   ```rust
   use surgeist_css::{CssRecoveryAction, parse_sheet};

   fn main() {
       let report = parse_sheet(
           ".before { color: red; } @unknown value; .after { color: blue; }",
       );
       assert_eq!(report.syntax().rules().len(), 2);
       assert_eq!(report.diagnostics().len(), 1);
       assert_eq!(report.diagnostics()[0].action(), CssRecoveryAction::DropAtRule);
   }
   ```

3. Run `cargo run --offline` from the consumer project. The program exits
   successfully when both valid rules remain and the unknown at-rule produces
   exactly one `DropAtRule` diagnostic. The report is recovered, so `is_clean()`
   is false even though every retained rule is valid.

The example follows the public [stylesheet recovery documentation](../src/lib.rs).
It does not evaluate or apply the retained styles.

## Verify an existing checkout

From the `surgeist-css` crate directory, this focused public-API test checks
stylesheet and style-attribute recovery without running the corpus capture:

```sh
cargo test --offline -p surgeist-css --no-default-features --test initiative_i01_audit recovery_front_doors_retain_valid_siblings -- --exact
```

The expected result is one passing test. Its assertions are in
[tests/initiative_i01_audit.rs](../tests/initiative_i01_audit.rs).

Continue with [declaration inspection and clean-input validation](how-to.md).
