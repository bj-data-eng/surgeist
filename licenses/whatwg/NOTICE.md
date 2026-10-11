# WHATWG Fullscreen reference

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).

The checkout includes a reformatted copy of the [Fullscreen snapshot at `7c38d773117aa1e6bfa13754afe77483f40c908f`](https://fullscreen.spec.whatwg.org/commit-snapshots/7c38d773117aa1e6bfa13754afe77483f40c908f/) as software implementation reference material. Its source body SHA-256 is `7d8cd68e63c06f2ac1a8caad1804ad48e6000af1dcf20943139f06c8fbfef6d1`.

The [exact upstream LICENSE](LICENSE) from that commit is included byte-for-byte. It declares Creative Commons Attribution 4.0 International for the specification text, and BSD 3-Clause instead to the extent portions are incorporated into source code. Those are distinct scopes, not interchangeable alternatives for the Markdown text. The LICENSE includes both complete legal texts and the original copyright notice.

Changes made for this checkout on 2026-10-03 are offline format conversion and exact-snapshot local link rewriting; the [conversion report](../../references/CONVERSION-REPORT.md) states the representation checks and limitations. Original source text, authorship and notices remain in the [bundled reference](../../references/fullscreen--7c38d773117aa1e6bfa13754afe77483f40c908f--7d8cd68e63c0.md). The original upstream document remains authoritative, and no endorsement is implied.

## DOM, HTML, Web IDL and Infra bounded captures

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).

These 2026-10-10/11 bounded captures retain the complete named clauses and original headers; other chapters remain upstream. The exact LICENSE files acquired from DOM `3071e5f269448350fdea79bb284eedaf17bc431e`, HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, Web IDL `0620cf171d1b97138847088748e363cae4005156` and Infra `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a` match the already retained [upstream LICENSE](LICENSE) byte-for-byte, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. These repository commits identify legal provenance; living chapter captures remain independently identified by their full original HTML hashes. In particular, the HTML repository commit is not claimed to identify the served chapter bytes.

| Original standard and bundled representation | Full original HTML SHA-256 and complete retained clauses |
| --- | --- |
| [DOM Standard: bounded DOM foundation source capture](https://dom.spec.whatwg.org/); [dom-foundation--editor-capture-20261010--08dff0a4827f.md](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md) | `034b308db8482bddb37070849e9d4ab40bcb2d10e6494cffad14678a252a58b1`; `trees`, `namespaces`, `node-trees`, `interface-node`, `interface-document`, `interface-documenttype`, `interface-documentfragment`, `interface-shadowroot`, `interface-element`, `interface-characterdata`, `interface-text`, `interface-cdatasection`, `interface-processinginstruction`, `interface-comment`, `ipr` |
| [HTML Standard: bounded DOM foundation source capture](https://html.spec.whatwg.org/multipage/dom.html); [dom-html-language-direction--editor-capture-20261010--494ac341fe2b.md](../../references/dom-html-language-direction--editor-capture-20261010--494ac341fe2b.md) | `2540c3e6a061fef814c0e7e53c1c64c892a6807a63396199a7783f868d4b6ea8`; `the-documentorshadowroot-interface`, `elements-in-the-dom`, `the-lang-and-xml:lang-attributes`, `the-dir-attribute` |
| [HTML Standard: bounded DOM foundation source capture](https://html.spec.whatwg.org/multipage/semantics-other.html); [dom-html-host-selector-facts--editor-capture-20261010--07d421b207b3.md](../../references/dom-html-host-selector-facts--editor-capture-20261010--07d421b207b3.md) | `8ea165c78170419eb4e18d91ba3fdc5264361b77de4d619f10829868f1addee1`; `disabled-elements`, `selectors` |
| [Web IDL Standard: bounded DOM foundation source capture](https://webidl.spec.whatwg.org/); [dom-webidl-string-types--editor-capture-20261010--53a256058148.md](../../references/dom-webidl-string-types--editor-capture-20261010--53a256058148.md) | `802c98f87b3bb8d353a5e071041fb5091188d11669aaed6201627574ff18202c`; `idl-DOMString`, `idl-USVString`, `ipr` |
| [HTML Standard: bounded DOM foundation source capture](https://html.spec.whatwg.org/multipage/interaction.html); [dom-html-host-interaction--editor-capture-20261010--8a01a8043e7c.md](../../references/dom-html-host-interaction--editor-capture-20261010--8a01a8043e7c.md) | `87d5bedffbeb165e8c5c2b5765de86db90277fcd0fd2f47829d99fe90f64a3c4`; `the-hidden-attribute`, `inert-subtrees`, `user-activation-data-model`, `focus` |
| [Infra Standard: bounded DOM foundation source capture](https://infra.spec.whatwg.org/); [dom-infra-codepoints-strings--editor-capture-20261010--a140d8f6286f.md](../../references/dom-infra-codepoints-strings--editor-capture-20261010--a140d8f6286f.md) | `aa22601a985cb01c51678e531e6a131174a128881027ef98d37de8ad150757cb`; `code-points`, `strings`, `ipr` |
| [html-tables bounded style determination](https://html.spec.whatwg.org/multipage/tables.html); [style-specialized-determinations-html-tables--editor-capture-20261011--7e90e654e94f.md](../../references/style-specialized-determinations-html-tables--editor-capture-20261011--7e90e654e94f.md) | `769e9afab8720b2c762fbe060ae2b5a15bfe84e202f7a043414e732cf9ff910a`; `table-processing-model` |

Changes comprise offline section extraction, format conversion, provenance labels and link rewriting; complex tables preserve semantic HTML. Original text, code examples, normative/informative labels, status, copyright and notices remain in each capture. Specification prose remains Creative Commons Attribution 4.0 International; BSD 3-Clause applies only to portions incorporated into source code, not as an alternative license for these Markdown documents. Original sources remain authoritative and no endorsement or full DOM/HTML implementation conformance is implied.

## HTML vocabulary incorporation — 2026-10-11

The HTML crate's [vocabulary source](../../crates/surgeist-html/src/vocabulary.rs)
incorporates portions of the selected edition's non-normative element and
attribute indexes into ordinary Rust source under the upstream BSD 3-Clause
provision. The [offline generator](../../crates/surgeist-html/tools/generate_vocabulary.py)
reads the retained [index capture](../../references/html-language-elements-attributes--877d6146487f--2ca83b5ac984.md).
Adaptations expand the grouped heading names, collapse printed whitespace,
preserve contextual attribute rows, and resolve local references to the exact
HTML edition. Source revision and hashes are embedded in the generated records.
The unchanged [upstream LICENSE](LICENSE) supplies the copyright, conditions
and disclaimer. These inventories do not replace the defining HTML clauses or
assert complete native HTML or browser conformance.

## Pinned HTML language snapshot — 2026-10-11

The [selected HTML edition](../../references/SOURCE-EDITIONS.md#pinned-html-language-baseline) retains ten bounded complete-section format conversions from [immutable HTML snapshot `877d6146487f15b15bc8dd0e3c625063a37b456a`](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/), complete original HTML SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. This snapshot directly identifies the retained edition, unlike the historical live chapter/legal-pin association above. All §4 element definitions are included; other browser runtime chapters are explicitly bounded. The [exact upstream LICENSE](LICENSE) at this commit remains unchanged, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is Creative Commons Attribution 4.0 International; BSD 3-Clause applies only to portions incorporated into source code. Offline extraction, Markdown reformatting, semantic tables/SVG preservation, inert embedded-example retention and same-edition link rewriting are disclosed in the [conversion report](../../references/CONVERSION-REPORT.md#pinned-html-language-snapshot--2026-10-11). Source text and original classifications remain authoritative; no endorsement is implied.
