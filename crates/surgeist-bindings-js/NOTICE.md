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

## Standards adaptations

Primitive conversions adapt the pinned [Web IDL algorithms](../../references/js-bindings-standards.md),
Copyright © WHATWG (Apple, Google, Mozilla, Microsoft), under the
[WHATWG license](../../licenses/whatwg/LICENSE) and
[adaptation notice](../../licenses/whatwg/NOTICE.md#javascript-binding-algorithms).
The [retained DOMException declaration and names](../../references/webidl-domexception--0620cf171d1b.md)
supply interface metadata and native exception names.

The CSS namespace and stylesheet, rule-list and media-list declarations adapt
the [retained CSSOM editor draft](../../references/cssom-1--editor-capture-20261009--d42e145ec395.md#source-provenance),
Copyright © 2026 World Wide Web Consortium, under the
[W3C Software and Document License](../../licenses/w3c/software-license-2023.txt).
The authored description and generated Rust transpose the selected declarations
into binding metadata.
