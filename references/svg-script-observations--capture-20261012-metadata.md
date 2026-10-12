# SVG script processing source observations

These metadata records distinguish the observed SVG2 bodies from the selected
SVG Tiny 1.2 processing algorithm. They contain no specification excerpt and
select no new production edition. The original body captures remain ignored
research; they are not executable inputs or shipped browser runtime assets.

Observed on 2026-10-12:

| Source | Complete body SHA-256 | Bytes | Relevant raw lines |
| --- | --- | ---: | --- |
| [SVG2 editor-draft script chapter](https://w3c.github.io/svgwg/svg2-draft/interact.html#ScriptElement) | 261d80ba32c584ecd04079a1c770cfc2d9db8daffe246fa32b145e52a588be38 | 73496 | 955–1150 |
| [SVG2 editor-draft conformance chapter](https://w3c.github.io/svgwg/svg2-draft/conform.html#processing-modes) | aa45b2160a9509b055fae6af59900e2941d87a01d57355bdf09c8e880bbcac1c | 67346 | 91–161 |
| [SVG2 TR conformance alias cited by #1136](https://www.w3.org/TR/SVG2/conform.html#processing-modes) | d320f04b3ec9cfc531f82b641ab23ef245595c10f463c1a3968bf4c1ca61f687 | 67292 | 91–161 |

The two conformance bodies differ only at raw line 62, in publication stylesheet
selection and inclusion of a link-fixup script tag. Their observed raw 91–161
clauses are byte-identical. This is a verified bounded text comparison, not proof
of immutable edition equivalence or a changed HTML production pin. No scripts,
stylesheets, linked resources or MathJax were run or retrieved.

The observed sources separate script execution from external-resource handling.
Same-document and data references are outside that external-reference class.
Consequently script-disabled or external-reference-disabled alone does not prove
the stronger no-acquisition boundary selected for Surgeist's fresh standalone
operation. The source chapter's HTML-equivalence language does not provide a
complete SVG metadata/preparation algorithm.

The actual selected reconciliation and namespace-specific clone rule are in
[#1136](https://github.com/bj-data-eng/surgeist/issues/1136), sections “Offline
Disabled SVG script: explicit processing reconciliation” and “SVG script cloning:
preserve the processed bit by bounded correction”. The complete selected
issue-body capture, retained as active research, has SHA-256
`710d159a89adf10c742dff41123104f9a115db21710f45fe6f44c97f315cfbca`.
This observation does not replace those product determinations.

The clone decision explicitly addresses the standards gap noted in the retained
[DOM clone extension point](dom-foundation--editor-capture-20261010--08dff0a4827f.md#concept-node-clone-ext),
source lines 1667–1679. Its selected browser witness remains metadata-only:
[WebKit SVG script clone state](webkit-svg-script-clone--73aa6c89e2cb-metadata.md).
The witness revision is `73aa6c89e2cb77c46184a81aec944e4ab99d114d`;
complete original SHA-256 is
`b36a783f9ef0034c1f89b0989874356aa908f5df275bb264c633085148b49906`.
The complete original copyright and GNU Library GPL v2-or-later notice and the
actual lines 117–120 were inspected. Neither WebKit source text nor its combined
HTML/SVG preparation machinery is copied into this metadata reference. The local
original remains private research.

The complete standards bodies, capture timestamps and source URLs are recorded
in `tmp/work/html-svg-script-sources/capture-metadata.json`; `SHA256SUMS`
checks the retained original bytes.
The [source-edition mapping](SOURCE-EDITIONS.md#offline-disabled-svg-script-and-clone-sources)
records the operative boundary. This metadata is not implementation or completion
evidence.
