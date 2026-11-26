# surgeist-layout-parity

Browser-parity tooling for [surgeist-layout](../surgeist-layout/README.md), for
developers who change layout behavior or refresh the browser corpus. The crate
holds the checked-in corpus of browser-captured XML, the golden generator
`surgeist-layout-generate`, the `qualify-root-context` qualification driver and
the test support that compares layout output with the captured geometry. It
depends on `surgeist-layout`'s public API only. It is not published.

## Try a parity check

From this checkout, with the locked dependencies available, validate the checked-in
corpus without a browser or a source cache:

```sh
cargo run --locked --offline -j 1 -p surgeist-layout-parity --bin surgeist-layout-generate -- check-corpus
```

Expected: the command exits with status 0. The strict raw verdict over every
checked-in case is the test
`browser_parity::tests::runs_all_checked_in_browser_parity_xml`:

```sh
env -u SURGEIST_PARITY_FILTER SURGEIST_PARITY_RAW_EXACT=1 cargo test --locked --offline -j 1 -p surgeist-layout-parity --lib browser_parity::tests::runs_all_checked_in_browser_parity_xml -- --exact
```

The [reference](docs/reference.md) lists the targets, the runtime and the recipes. The
[corpus guide](tests/layout/browser_parity/README.md) documents every
generation, import and qualification command, and the
[measurement protocol](tests/layout/browser_parity/measurement-protocol.md) records
the private browser protocol. The [crate guide](AGENTS.md) lists the layout of the
sources, the corpus and the supported checks.

## License and attribution

[MIT license](LICENSE) · [Third-party notices](NOTICE.md)
