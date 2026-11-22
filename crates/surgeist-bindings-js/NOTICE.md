# surgeist-bindings-js

This crate uses the [Boa JavaScript engine](https://boajs.dev/) through the
[Surgeist fork](https://github.com/bj-data-eng/surgeist-boa). Boa is available
under the upstream [MIT](../surgeist-boa/LICENSE-MIT) or
[Unlicense](../surgeist-boa/LICENSE-UNLICENSE) terms. The Git submodule records
the exact source. Surgeist's [MIT license](../../LICENSE) covers its own code.

The [bundled locale data](../surgeist-boa/core/icu_provider/README.md) is
generated from [Unicode ICU4X](https://github.com/unicode-org/icu4x) sources.
The retained [Unicode License v3 and ICU4X attribution](../../licenses/boa-icu4x/LICENSE)
preserve the notice shipped with the data-generating provider.
