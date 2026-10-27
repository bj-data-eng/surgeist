# Syntax 3 conformance coverage

This page maps the clauses of the pinned CSS Syntax Module Level 3 edition to
the tests of this crate. It records what the tests establish and what they do
not, so a reader can tell a covered clause from an unexamined one.

**Nothing is claimed yet.** The crate has no implementation and no tests. Every
row below has the status "Not claimed". A row changes only in the commit that
adds the implementation and the tests that exercise its clause, and that commit
cites the tests by name.

## Edition and evidence classes

The [CSS Syntax 3 edition](../../../references/SOURCE-EDITIONS.md#css-syntax-3-edition)
selected for Surgeist is the 1 October 2026 Candidate Recommendation Draft. Its
[local copy][syntax] is the only source of expected behavior for these rows.
The conventions of other libraries are not an authority for a clause.

Each covered row will name its evidence by class:

- **Specification-derived**: a literal input and expected output written from
  the clause text or one of its examples.
- **Differential**: a comparison with another implementation's output on the
  same input. It detects disagreement; it never establishes a clause, and a
  disagreement is resolved from the specification text.
- **Property**: an invariant checked over generated inputs.
- **External vectors**: a retained corpus of third-party conformance tests,
  with its provenance recorded with the corpus.

## Clause map

| Clause | Subject | Status | Tests |
| --- | --- | --- | --- |
| [3.2][input-byte-stream] | The input byte stream: decoding and fallback encoding | Not claimed | None |
| [3.3][input-preprocessing] | Preprocessing the input stream | Not claimed | None |
| [4.1][token-diagrams] and [4.2][tokenizer-definitions] | Token set and lexical definitions | Not claimed | None |
| [4.3][tokenizer-algorithms] | Tokenizer algorithms, including `unicode-range` tokens | Not claimed | None |
| [5.2][css-tree] and [5.3][parser-definitions] | Parsing results and token streams | Not claimed | None |
| [5.4][parser-entry-points] | Parser entry points | Not claimed | None |
| [5.5][parser-algorithms] | Parser algorithms | Not claimed | None |
| [6][anb-microsyntax] | The `An+B` microsyntax | Not claimed | None |
| [9][serialization] | Serialization and the round-trip requirement | Not claimed | None |

Sections 7 and 8 define how grammars and style sheets use the syntax. They give
the syntax a CSS meaning and are outside this crate; `surgeist-css` records
their coverage.

[syntax]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md
[input-byte-stream]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#input-byte-stream
[input-preprocessing]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#input-preprocessing
[token-diagrams]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#token-diagrams
[tokenizer-definitions]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#tokenizer-definitions
[tokenizer-algorithms]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#tokenizer-algorithms
[css-tree]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#css-tree
[parser-definitions]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-definitions
[parser-entry-points]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-entry-points
[parser-algorithms]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#parser-algorithms
[anb-microsyntax]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#anb-microsyntax
[serialization]: ../../../references/css-syntax-3--CRD-css-syntax-3-20261001--8dc3adcc2b4e.md#serialization
