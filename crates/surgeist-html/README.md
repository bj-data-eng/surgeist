# surgeist-html

This crate owns the pinned HTML language foundation. Its initial public surface
provides immutable exact UTF-16 strings, namespace identity, qualified names,
ordered raw attributes and native immutable document/fragment construction.
The [HTML contribution](https://github.com/bj-data-eng/surgeist/issues/1132)
tracks the remaining typed vocabulary/value interpretation, parser, semantic
interpretation, provenance and full parser resource-admission capabilities.

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

`HtmlSource` identifies one supplied already-decoded Unicode input. Its validated
half-open ranges use supplied UTF-8 bytes or supplied UTF-16 units. Equal text
does not imply equal source identity; cloning a source retains its identity.
UTF-8 ranges require scalar boundaries; UTF-16 ranges can select individual
surrogate units. Source identity supplies no document URL, base URL, origin or
permission. This source contract performs no parsing or preprocessing, and
preserves literal U+FEFF and CR/LF units.

`HtmlDocument::builder` and `HtmlFragment::builder` create bounded native
authoring sessions. The caller supplies every construction capacity explicitly:
nodes, edges, attributes, stored UTF-16 units and depth. Roots and separate
template/shadow fragments participate in admission. A rejected append leaves
content and admitted usage unchanged; consuming `finish` freezes one immutable
flat arena. The storage does not recursively own child nodes.

Traversal preserves mixed sibling order and exact native values without
serialization or reparsing. Node IDs are qualified by their immutable result;
foreign-result lookup returns a typed error. Template contents and native shadow
contents have separate fragment roots and reverse owner associations. Native
processing instructions validate the selected HTML target/data syntax; raw
shared names remain available without generic DOM name validation.

These native construction artifacts perform no parsing. They do not claim parser
metadata, context effects, diagnostics, source mapping, full content-conformance
checking or script execution. Their capacities count logical retained content;
they do not establish a universal process-memory or CPU bound. General shadow-host
conformance and live installation remain separate admission concerns.
