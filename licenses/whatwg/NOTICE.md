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

## HTML common microsyntax incorporation — 2026-10-11

The HTML crate's intrinsic numeric, temporal and token values and common
interpretation algorithms also implement the selected edition's §2.3 common
microsyntax clauses. Their Rust implementation separates author grammar,
standard recovery and raw storage. The defining clauses remain in the
[foundations capture](../../references/html-language-foundations--877d6146487f--b55e73c31978.md).
The edition and original-source hash below apply to these algorithms as well.
No full browser or attribute-conformance claim follows from this incorporation.

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

The [being-rendered definition and notes](../../references/html-being-rendered--877d6146487f.md)
add a bounded prose excerpt from original lines 99611–99622 of that same
snapshot. Markdown conversion removes inline markup and folds layout whitespace;
the three paragraphs retain the original words. The existing exact license and
Creative Commons Attribution 4.0 prose terms apply. This excerpt adds no distinct
source identity or rendering implementation.

## HTML character-reference table incorporation — 2026-10-11

The HTML crate incorporates the complete named-character-reference and C1
replacement tables from the pinned HTML edition into ordinary Rust data under
the upstream BSD 3-Clause source-code provision. The [offline generator](../../crates/surgeist-html/tools/generate_character_references.py)
reads the retained [syntax capture](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md),
SHA-256 `db1eaacc388d494736080301dd18e9147feb39f2651e2bc7fbeed36ee2cfb214`,
and writes [private reference data](../../crates/surgeist-html/src/parser/character_references.rs).
Adaptations omit the leading ampersand from lookup keys, sort all complete
names, retain semicolon-less spellings and two-scalar values, and represent
unlisted C1 overrides as absence. The generation headers identify the selected
HTML revision, original-source hashes and independently compared html5ever
revision. No html5ever atom implementation or generation machinery is copied.
The unchanged [upstream LICENSE](LICENSE) supplies the original copyright,
conditions and disclaimer. Lookup data alone does not assert full tokenization
or browser conformance.

## HTML editability, design mode and spellcheck — 2026-10-11

The [bounded complete-section capture](../../references/html-editability--877d6146487f.md)
adds §§6.8.1, 6.8.2, 6.8.4 and 6.8.5 from the same immutable HTML snapshot
`877d6146487f15b15bc8dd0e3c625063a37b456a`. Copyright © WHATWG
(Apple, Google, Mozilla, Microsoft). The exact [upstream license](LICENSE)
continues to cover specification prose under CC BY 4.0 and portions incorporated
into source code under BSD 3-Clause. Changes by Surgeist on 2026-10-11:
selected sections converted to Markdown, source links made absolute and
script/style/MDN annotations removed without executing source scripts.
The capture retains the complete selected algorithms and interface text; no
editing-command runtime or spelling service is incorporated.

## HTML iframe sandbox directive — 2026-10-11

The [complete sandboxing capture](../../references/html-sandboxing--877d6146487f.md)
retains §7.1.5 from HTML revision `877d6146487f15b15bc8dd0e3c625063a37b456a`,
complete original SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). The exact
[upstream license](LICENSE) covers prose under CC BY 4.0 and portions
incorporated into source code under BSD 3-Clause. Changes by Surgeist on
2026-10-11 comprise offline bounded extraction, Pandoc Markdown conversion,
absolute source links and inert ID/note preservation; source scripts do not run.
The Rust intrinsic attribute operation adapts thirteen keywords and sixteen
directive restrictions, separately retaining author-only errors. It incorporates
no active-policy composition, navigation, origin change or enforcement algorithm.

## JavaScript MIME essence and HTML checkpoint — 2026-10-11

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).

The [bounded MIME Sniffing capture](../../references/mimesniff-javascript-essence--39aa53511b13.md)
retains §4.6 JavaScript MIME type and essence-match definitions, all 16 essence
strings and the attached HTML hook note from
[immutable revision `39aa53511b13953d84fef8d4131d6f61d0ccbde6`](https://mimesniff.spec.whatwg.org/commit-snapshots/39aa53511b13953d84fef8d4131d6f61d0ccbde6/),
displayed 17 July 2026; complete original HTML SHA-256
`11caac49505995bcef51153a827c6fe96464111c8914e2c399d9e8fe9a467cd7`.
Its original footer declares CC BY 4.0 for specification prose and BSD 3-Clause
instead for portions incorporated into source code. The existing complete
[WHATWG license](LICENSE) supplies both legal texts, the copyright and disclaimer.
No MIME repository LICENSE acquisition or byte-identity comparison is claimed.
Changes by Surgeist on 2026-10-11 are bounded extraction, Markdown conversion
with Pandoc 3.1.11.1 and absolute-link rewriting, without executing source scripts.
Original source text remains authoritative; no endorsement is implied.

The private Disabled-script classification adapts the exact 16-string matching
vocabulary into Rust source under the BSD code-incorporation provision. HTML
owns the surrounding preparation branches. This incorporation adds no MIME
record parser, parameter stripping, sniffing operation or runtime dependency.

The [complete checkpoint capture](../../references/html-microtask-checkpoint--877d6146487f.md)
retains queue/reentrancy definitions, all eight algorithm steps and explanatory
notes from the existing HTML snapshot
`877d6146487f15b15bc8dd0e3c625063a37b456a`, complete original SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
The same CC BY prose terms and unchanged exact HTML license above apply.
Changes comprise bounded extraction, Pandoc conversion and absolute-link
rewriting on 2026-10-11, with no source-script execution. This is another
representation of that HTML identity, not a separate edition. Promise,
IndexedDB, ClearKeptObjects and external timing hooks remain in the capture.

The selected private standalone absent-runtime-domain reduction is an explicit
architecture capability specialization, not a browser event loop, execution of
the external timing algorithm or a normative null-script-return rule. No timing
source pin, retained snapshot or algorithm incorporation is claimed. The
[edition selection](../../references/SOURCE-EDITIONS.md#disabled-script-preparation-and-standalone-checkpoint-sources)
and [conversion record](../../references/CONVERSION-REPORT.md#javascript-mime-essence-and-html-checkpoint-sources--2026-10-11)
bound this source use without certifying a complete parser.

## HTML-delegated Fetch destinations — 2026-10-12

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).

The [bounded Fetch definition](../../references/fetch-script-like-destinations--e9460d1b22ea.md)
retains the script-like destination definition and adjacent warning from
[immutable Fetch revision `e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d`](https://fetch.spec.whatwg.org/commit-snapshots/e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d/),
original rendered lines 2820–2828, complete original HTML SHA-256
`639d437c000a870bf161804a92f9e0ce1b22dae61f39d8975a7b8429302534eb`.
The exact upstream LICENSE from that revision is byte-identical to the retained
[complete local LICENSE](LICENSE), SHA-256
`85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`.
Specification prose remains CC BY 4.0; BSD 3-Clause applies instead only to
portions incorporated into source code, not this Markdown document.

Changes by Surgeist on 2026-10-12 comprise bounded extraction, hand conversion
to Markdown, typography and link formatting, plus explicitly labelled source
selection commentary. No normative destination is added or removed; the warning
retains `xslt` outside the finite definition. Original text remains authoritative
and no endorsement is implied. This is source reference retention; it does not
claim a Fetch implementation or completed HTML link-destination operation.

## Ordinary-template and PI-map incorporation — 2026-10-12

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft).

The private fresh ordinary-template construction implements the selected
[HTML syntax clauses](../../references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md)
and [template document/state/adoption/cloning/patching clauses](../../references/html-language-elements-interactive-scripting--877d6146487f--05bb1d85cc89.md#the-template-element)
of revision 877d6146487f15b15bc8dd0e3c625063a37b456a, complete original SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`.
The retained [DOM foundation capture](../../references/dom-foundation--editor-capture-20261010--08dff0a4827f.md)
supplies node-document, cloning/adoption and PI initialization/map behavior;
complete original SHA-256
`034b308db8482bddb37070849e9d4ab40bcb2d10e6494cffad14678a252a58b1`.
The existing legal-pin/capture distinction for that DOM witness remains unchanged.

The source algorithms are adapted into Surgeist-owned controlled registry,
creation, effect/clone and map operations. The existing complete [LICENSE](LICENSE)
retains BSD 3-Clause conditions for portions incorporated into source code;
the original specification prose and converted references remain CC BY 4.0.
No new edition, full DOM, public complete parser or execution runtime is claimed.

The delegated XML-stylesheet/XML grammar is a separate W3C origin.
Its [source-specific attribution](../w3c/NOTICE.md#dom-delegated-xml-pseudo-attribute-grammar)
and complete historical/2015 document terms, original notices and prescribed
derivative notices apply independently. WHATWG terms do not replace them.
