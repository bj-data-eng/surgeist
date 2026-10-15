# HTML hosted Document activity and browsing-context clauses

Source: [immutable WHATWG HTML snapshot](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/), revision
`877d6146487f15b15bc8dd0e3c625063a37b456a` (9 October 2026).
Complete original HTML: 14930614 bytes, SHA-256
`877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`. Extracted offline from the already retained original on
2026-10-15; no new retrieval or edition is selected.

This bounded representation retains the complete active-document definition
(original lines70877–70880, plus its opening algorithm wrapper at70876),
Document browsing-context definition and complete createDocument/removed-iframe
note (71636–71642), and fully-active heading/definition plus complete
child-navigable interaction paragraph (71919–71932). The surrounding browsing
context/navigation chapters and the subsequent example are not included.
The exact selected original fragments, joined by one extra LF between fragments,
have SHA-256 `f8564ea4758e8af4bdd7ee661325dfe6565013d25796121e84e924efe8e225eb`; the filename suffix identifies this bounded
input, not a new upstream revision or the final Markdown-file hash.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is
[Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/);
the existing [complete WHATWG license](../licenses/whatwg/LICENSE) and
[pinned HTML attribution](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11)
apply. The upstream BSD 3-Clause provision applies to portions incorporated into
source code, not as an alternative license for this Markdown prose.

Changes by Surgeist on 2026-10-15: bounded offline extraction, explanatory headings
and provenance, and absolute same-edition fragment links. Selected source bodies
remain literal semantic HTML inside Markdown, preserving every word, original
ID, inline code/variable/definition, algorithm wrapper and informative-note class.
No source script executed, operative branch paraphrased or browser implementation
ingested. Original text and normative/informative distinctions remain authoritative;
no endorsement or hosted/browser implementation claim is implied.

## Navigable active Document definition

<!-- selected-source-fragment:70876-70880 -->
  <div data-algorithm="">
  <p>A <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=navigables:navigable-2>navigable</a>'s <dfn data-dfn-for=navigable id=nav-document data-export="">active
  document</dfn> is its <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-active-history-entry" id=navigables:nav-active-history-entry-6>active session history
  entry</a>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#she-document" id=navigables:she-document-2>document</a>.</p>
  </div>

## Document browsing context and createDocument note

<!-- selected-source-fragment:71636-71642 -->
  <p>A <dfn data-export="" data-lt="browsing context" id=concept-document-bc data-dfn-for=Document><code>Document</code>'s browsing context</dfn> is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#browsing-context" id=windows:browsing-context-7>browsing context</a> or null,
  initially null.</p>

  <p class=note>A <code id=windows:document-3><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> does not necessarily have a non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=windows:concept-document-bc>browsing context</a>. In particular, data mining tools are likely
  to never instantiate browsing contexts. A <code id=windows:document-4><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> created using an API such as <code id=windows:dom-domimplementation-createdocument><a data-x-internal=dom-domimplementation-createdocument href=https://dom.spec.whatwg.org/#dom-domimplementation-createdocument>createDocument()</a></code> never has a non-null <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#concept-document-bc" id=windows:concept-document-bc-2>browsing context</a>. And the <code id=windows:document-5><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> originally
  created for an <code id=windows:the-iframe-element><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-iframe-element">iframe</a></code> element, which has since been <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#remove-an-element-from-a-document" id=windows:remove-an-element-from-a-document>removed from the document</a>, has no associated browsing context, since that
  browsing context was <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#destroy-a-document" id=windows:destroy-a-document>nulled out</a>.</p>

## Fully active Documents

<!-- selected-source-fragment:71919-71932 -->
  <h4 id=fully-active-documents><span class=secno>7.3.3</span> Fully active documents<a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active-documents" class=self-link></a></h4>

  <div data-algorithm="">
  <p>A <code id=fully-active-documents:document><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code> <var>d</var> is said to be <dfn data-dfn-for=Document id=fully-active data-export="">fully
  active</dfn> when <var>d</var> is the <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-document" id=fully-active-documents:nav-document>active document</a> of a
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#navigable" id=fully-active-documents:navigable>navigable</a> <var>navigable</var>, and either <var>navigable</var> is a <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#top-level-traversable" id=fully-active-documents:top-level-traversable>top-level
  traversable</a> or <var>navigable</var>'s <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container-document" id=fully-active-documents:nav-container-document>container
  document</a> is <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=fully-active-documents:fully-active>fully active</a>.</p>
  </div>

  <p>Because they are associated with an element, <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable" id=fully-active-documents:child-navigable>child
  navigables</a> are always tied to a specific <code id=fully-active-documents:document-2><a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#document">Document</a></code>, their <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container-document" id=fully-active-documents:nav-container-document-2>container document</a>, in their <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-parent" id=fully-active-documents:nav-parent>parent navigable</a>. User agents must not allow the user to interact with
  <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#child-navigable" id=fully-active-documents:child-navigable-2>child navigables</a> whose <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nav-container-document" id=fully-active-documents:nav-container-document-3>container documents</a> are not themselves <a href="https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#fully-active" id=fully-active-documents:fully-active-2>fully
  active</a>.</p>
