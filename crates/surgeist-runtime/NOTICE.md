# surgeist-runtime

This notice covers the `surgeist-runtime` 0.1.0 source distribution described by
[Cargo.toml](Cargo.toml). The project's own license is [MIT](LICENSE).

## Dependencies

Sibling Surgeist dependencies are source owned in this repository. This notice
also covers the direct native entropy dependency below; it does not claim a
complete transitive or compiled-release inventory. The default feature set is
empty. This source distribution contains no vendored implementation of that
dependency; its accompanying license files are retained for attribution.

### getrandom 0.3.4

The native hosted lifecycle uses getrandom, distributed by The Rand Project
Developers, to request cryptographically secure random bytes through its safe,
fallible interface. The dependency is selected for macOS, Linux, Windows,
Android and iOS with default features disabled. Other targets retain the
typed unavailable branch; no JavaScript entropy backend is selected here.

* Licenses: [MIT](licenses/getrandom/LICENSE-MIT) OR [Apache-2.0](licenses/getrandom/LICENSE-APACHE)
* Homepage: [getrandom](https://github.com/rust-random/getrandom)

Both license files are exact copies from the already installed 0.3.4 source
package. Its MIT notice retains the 2018–2025 rust-random and 2014 Rust Project
copyrights. No dependency source is copied into Surgeist implementation files.

## Native history UUID generation

Copyright © 2023 W3C®. This software or document includes material copied from or
derived from [Web Cryptography API](https://github.com/w3c/webcrypto/blob/811c24c69eb22d477af5f1678cb70dbc611c7c40/spec/Overview.html).
The original document retains Copyright © 2026 World Wide Web Consortium and
its source status. The complete [W3C Document License 2023](../../licenses/w3c/document-license-2023.txt)
and [source attribution and modification notice](../../licenses/w3c/NOTICE.md#native-hosted-random-uuid-clauses)
accompany the [bounded UUID algorithm reference](../../references/webcrypto-random-uuid--811c24c69eb2--1c697c6b1d48.md).

Surgeist's Rust implementation applies the selected internal generation and
hexadecimal algorithms to actual fallible native entropy, with fixed owned
buffers, finite admission, cancellation polls and retained pending ownership.
The exposed Crypto method, Web IDL, JavaScript bindings and general cryptographic
operations are not incorporated into this crate. This software is not a
replacement technical specification; no W3C endorsement is implied.

This inventory covers this crate's source distribution. It does not cover the
Rust toolchain or standard library, or dependencies and assets selected by root
adapters and consuming applications.
