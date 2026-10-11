# Referrer Policy vocabulary — pinned source clauses

Primary source: [W3C Web Application Security Referrer Policy editor source](https://github.com/w3c/webappsec-referrer-policy/blob/cc435b05ca4a94f7f1a139be5074b168d20014db/index.src.html), commit `cc435b05ca4a94f7f1a139be5074b168d20014db` (20 March 2026). Captured 11 October 2026. Complete source: 49334 bytes; SHA-256 `a277e78e06418ec60afc6f499aee79d4f3a3a067e0fd97e9886a2d9e3de95eb1`. [Rendered editor document](https://w3c.github.io/webappsec-referrer-policy/#referrer-policies) is a reading aid, not the immutable byte identity.

This bounded readable Markdown conversion retains the vocabulary paragraph and IDL enumeration (original lines 216–236), default-policy sentence (249), and the first empty-string paragraph (518–522). Other policy behavior, examples, network algorithms and sections are not captured or adopted. Source is Bikeshed HTML; selected prose is rendered below with its original meaning and literal keywords; the enumeration is exact apart from uniform indentation. The complete original and repository license are retained with the capture input. No tables, diagrams, or complex layout occur in these selected clauses.

Status: Editor's Draft, as identified by the immutable source's `Status: ED` metadata. Editors: Dominic Farolino; former editors Emily Stark and Jochen Eisinger, as retained in that source.

Attribution: W3C Web Application Security Working Group and Referrer Policy contributors. The source has no pre-existing copyright statement. The document-license notice below uses the selected source edition's 2026 year; it is license-prescribed attribution, not an upstream notice transcription.

Copyright © 2026 World Wide Web Consortium. [W3C Document License, 2023](../licenses/w3c/document-license-2023.txt).

Copyright © 2023 W3C®. This software or document includes material copied from or derived from [Referrer Policy](https://github.com/w3c/webappsec-referrer-policy/blob/cc435b05ca4a94f7f1a139be5074b168d20014db/index.src.html).

Prose is retained under the [W3C Document License, 2023](../licenses/w3c/document-license-2023.txt) implementation-support exception. The marked Web IDL Code Component is covered by the [W3C Software and Document License, 2023](../licenses/w3c/software-license-2023.txt). The [exact repository declaration](../licenses/w3c/referrer-policy/LICENSE.md) is retained locally. [Repository license at the same pin](https://github.com/w3c/webappsec-referrer-policy/blob/cc435b05ca4a94f7f1a139be5074b168d20014db/LICENSE.md) identifies the [W3C Document License](https://www.w3.org/Consortium/Legal/copyright-documents). This is an excerpt/conversion, not an unmodified complete publication.

## Referrer Policies

A **referrer policy** is the empty string, `no-referrer`, `no-referrer-when-downgrade`, `same-origin`, `origin`, `strict-origin`, `origin-when-cross-origin`, `strict-origin-when-cross-origin`, or `unsafe-url`.

```webidl
enum ReferrerPolicy {
  "",
  "no-referrer",
  "no-referrer-when-downgrade",
  "same-origin",
  "origin",
  "strict-origin",
  "origin-when-cross-origin",
  "strict-origin-when-cross-origin",
  "unsafe-url"
};
```

The **default referrer policy** is `strict-origin-when-cross-origin`.

## The empty string

The empty string `""` corresponds to no referrer policy, causing a fallback to a referrer policy defined elsewhere, or in the case where no such higher-level policy is available, falling back to the default referrer policy. This happens in Fetch's main fetch algorithm, for example.

## Surgeist selection and limits

#1133 adopts the nine-value vocabulary for HTML referrer-policy attribute interpretation. Pinned HTML `877d6146487f15b15bc8dd0e3c625063a37b456a` §2.5.5 owns its enumerated matching and missing/invalid empty-string defaults. General enumerated matching is ASCII case-insensitive, without trimming or HTTP header-list parsing. Missing, conforming empty, and invalid raw values remain distinguishable even though all select the empty semantic state. Preserve raw UTF-16 values.

The empty state must not be eagerly replaced with this draft's default policy. Network request processing, effective-policy selection, document policy containers, referrer generation and URL resolution remain host/fetch operations outside this intrinsic enumeration. This selection does not import their algorithms or claim whole Referrer Policy conformance. The older 26 January 2017 published CR has the same vocabulary but different surrounding default-policy wording; the pinned newer editor source is selected, not a mixture of their processing models.
