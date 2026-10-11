# surgeist-html

This crate owns the pinned HTML language foundation. Its initial public surface
provides immutable exact UTF-16 strings, namespace identity, qualified names and
ordered raw attributes. The [HTML contribution](https://github.com/bj-data-eng/surgeist/issues/1132)
tracks the remaining typed authoring, immutable document/fragment, parser,
semantic interpretation, provenance and resource-admission capabilities.

The [selected WHATWG edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline)
defines behavior; the separate html5ever revision identifies implementation
provenance. Shared values apply no language-specific normalization. A Rust `str`
converts losslessly to `DocumentString`; conversion back to UTF-8 rejects isolated
surrogates. Raw names and attributes preserve supplied values for their eventual
owning language or DOM validation boundary.

The crate is independently usable and has no DOM or rendering dependency. DOM
will consume these shared values directly and own canonical live identity,
mutation, captures and HTML integration. Parser construction values do not form
a second live DOM. Template syntax and symbolic expressions remain in
`surgeist-template`.
