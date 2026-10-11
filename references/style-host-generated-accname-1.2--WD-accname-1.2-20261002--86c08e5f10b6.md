Attribution and reformatting notice added for Surgeist on 2026-10-11

This bounded reformatted excerpt accompanies Surgeist as software implementation support. The original English document remains authoritative. Added provenance notes are non-normative; this copy is not a new technical specification. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Material copied from [Accessible Name and Description Computation 1.2](https://www.w3.org/TR/accname-1.2/) under the [W3C Software and Document License, 2023 version](../licenses/w3c/software-license-2023.txt). Original copyright, liability, trademark and permissive document-license notice retained below.

# Provenance and boundary

Captured 2026-10-11. Full HTML SHA-256 `3658a43482510f5fa0083069c9dcb0321089acc903a2177a63091930da038082`; 127672 bytes. Bounded HTML SHA-256 `86c08e5f10b671ae793fb0818d794ae14b8f01f283e6208b9e6a8a445ae1571d`; 65460 bytes. Source revision metadata: `40cccbd3dd7779b10a4334cf9eba9b74aeca754e`. Retained complete sections: `terms`, `conformance`, `mapping_additional_nd`, `accessible-name-and-description-mapping`, plus source header and status. Other sections resolve through original source links. Source status and open questions remain intact; Surgeist decisions are recorded separately.

Converted with the existing HTML preparation, Pandoc 3.1.11.1 and semantic filter.

---

<!-- captured-body-start -->

[![W3C](https://www.w3.org/StyleSheets/TR/2021/logos/W3C)](https://www.w3.org/)

# <a id="title"></a>Accessible Name and Description Computation 1.2

<a id="w3c-state"></a>[W3C Working Draft](https://www.w3.org/standards/types#WD) 02 October 2026

More details about this document

<strong>This version:</strong>

<https://www.w3.org/TR/2026/WD-accname-1.2-20261002/>

<strong>Latest published version:</strong>

<https://www.w3.org/TR/accname-1.2/>

<strong>Latest editor's draft:</strong>

<https://w3c.github.io/aria/accname/>

<strong>History:</strong>

<https://www.w3.org/standards/history/accname-1.2/>

[Commit history](https://github.com/w3c/aria/commits/main/accname/)

<strong>Latest Recommendation:</strong>

<https://www.w3.org/TR/accname/>

<strong>Editors:</strong>

Bryan Garaventa ([Invited Expert](https://github.com/whatsock))

Melanie Sumner ([Invited Expert](https://github.com/melsumner))

<strong>Former editors:</strong>

Michael Cooper ([W3C](https://www.w3.org/))

Joanmarie Diggs ([Igalia, S.L.](https://www.igalia.com/)) (Editor until March 2021)

Richard Schwerdtfeger ([Knowbility](https://knowbility.org/)) (Editor until October 2017)

Joseph Scheuhammer ([Inclusive Design Research Centre, OCAD University](https://idrc.ocadu.ca/)) (Editor until May 2017)

James Craig ([Apple Inc.](https://www.apple.com/accessibility/)) (Editor until May 2016)

Andi Snow-Weaver ([IBM](https://www.ibm.com/)) (Editor until December 2012)

Aaron Leventhal ([IBM](https://www.ibm.com/)) (Editor until January 2009)

<strong>Feedback:</strong>

[GitHub w3c/accname](https://github.com/w3c/accname/) ([pull requests](https://github.com/w3c/aria/pulls?q=is%3Apr+is%3Aopen+label%3A%22spec%3Aaccname%22), [new issue](https://github.com/w3c/accname/issues/new/choose), [open issues](https://github.com/w3c/accname/issues/))

[Copyright](https://www.w3.org/policies/#copyright) © 2014-2026 [World Wide Web Consortium](https://www.w3.org/). W3C<sup>®</sup> [liability](https://www.w3.org/policies/#Legal_Disclaimer), [trademark](https://www.w3.org/policies/#W3C_Trademarks) and [permissive document license](https://www.w3.org/copyright/software-license-2023/) rules apply.

------------------------------------------------------------------------

<a id="sotd"></a>

## Status of This Document

<em>This section describes the status of this document at the time of its publication. A list of current W3C publications and the latest revision of this technical report can be found in the [W3C standards and drafts index](https://www.w3.org/TR/).</em>

This document was published by the [Accessible Rich Internet Applications Working Group](https://www.w3.org/groups/wg/aria) as a Working Draft using the [Recommendation track](https://www.w3.org/policies/process/20250818/#recs-and-notes).

Publication as a Working Draft does not imply endorsement by W3C and its Members.

This is a draft document and may be updated, replaced, or obsoleted by other documents at any time. It is inappropriate to cite this document as other than a work in progress.

This document was produced by a group operating under the [W3C Patent Policy](https://www.w3.org/policies/patent-policy/). W3C maintains a [public list of any patent disclosures](https://www.w3.org/groups/wg/aria/ipr) made in connection with the deliverables of the group; that page also includes instructions for disclosing a patent. An individual who has actual knowledge of a patent that the individual believes contains [Essential Claim(s)](https://www.w3.org/policies/patent-policy/#def-essential) must disclose the information in accordance with [section 6 of the W3C Patent Policy](https://www.w3.org/policies/patent-policy/#sec-Disclosure).

This document is governed by the <a id="w3c&#95;process&#95;revision"></a>[18 August 2025 W3C Process Document](https://www.w3.org/policies/process/20250818/).

<a id="terms"></a>

## <a id="x2-important-terms"></a>2. Important Terms

[](#terms)

<em>This section is non-normative.</em>

While some terms are defined in place, the following definitions are used throughout this document.

<strong><a id="dfn-assistive-technologies"></a><strong>Assistive Technologies</strong></strong>

Hardware and/or software that:

- relies on services provided by a [user agent](https://infra.spec.whatwg.org/#user-agent) to retrieve and render Web content
- works with a user agent or web content itself through the use of APIs, and
- provides services beyond those offered by the user agent to facilitate user interaction with web content by people with disabilities

This definition may differ from that used in other documents.

Examples of assistive technologies that are important in the context of this document include the following:

- screen magnifiers, which are used to enlarge and improve the visual readability of rendered text and images;
- screen readers, which are most-often used to convey information through synthesized speech or a refreshable Braille display;
- text-to-speech software, which is used to convert text into synthetic speech;
- speech recognition software, which is used to allow spoken control and dictation;
- alternate input technologies (including head pointers, on-screen keyboards, single switches, and sip/puff devices), which are used to simulate the keyboard;
- alternate pointing devices, which are used to simulate mouse pointing and clicking.

<strong><a id="dfn-accessible-description"></a><strong>Accessible Description</strong></strong>

An accessible description provides additional information, related to an interface element, that complements the <a id="ref-for-dfn-accessible-name-3"></a>[accessible name](#dfn-accessible-name). The accessible description might or might not be visually perceivable.

<strong><a id="dfn-accessible-name"></a><strong>Accessible Name</strong></strong>

The accessible name is the name of a user interface element. Each platform [accessibility API](https://www.w3.org/TR/wai-aria-1.2/#dfn-accessibility-api) provides the accessible name property. The value of the accessible name may be derived from a visible (e.g., the visible text on a button) or invisible (e.g., the text alternative that describes an icon) property of the user interface element. See related <a id="ref-for-dfn-accessible-description-3"></a>[accessible description](#dfn-accessible-description).

A simple use for the accessible name property may be illustrated by an "OK" button. The text "OK" is the accessible name. When the button receives focus, assistive technologies may concatenate the platform's role description with the accessible name. For example, a screen reader may speak "push-button OK" or "OK button". The order of concatenation and specifics of the role description (e.g., "button", "push-button", "clickable button") are determined by platform [accessibility API](https://www.w3.org/TR/wai-aria-1.2/#dfn-accessibility-api)s or <a id="ref-for-dfn-assistive-technologies-2"></a>[assistive technologies](#dfn-assistive-technologies).

<strong><a id="dfn-tooltip-attribute"></a><strong>Tooltip attribute</strong></strong>

Any host language attribute that would result in a user agent generating a tooltip such as in response to a mouse hover in desktop user agents.

<a id="conformance"></a>

## <a id="x3-conformance"></a>3. Conformance

[](#conformance)

As well as sections marked as non-normative, all authoring guidelines, diagrams, examples, and notes in this specification are non-normative. Everything else in this specification is normative.

The key words <em>MAY</em>, <em>MUST</em>, and <em>MUST NOT</em> in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) \[[RFC2119](https://www.w3.org/TR/accname-1.2/#bib-rfc2119)\] \[[RFC8174](https://www.w3.org/TR/accname-1.2/#bib-rfc8174)\] when, and only when, they appear in all capitals, as shown here.

Normative sections provide requirements that authors, user agents and assistive technologies <em>MUST</em> follow for an implementation to conform to this specification.

Non-normative (informative) sections provide information useful to understanding the specification. Such sections may contain examples of recommended practice, but it is not required to follow such recommendations in order to conform to this specification.

<a id="mapping&#95;additional&#95;nd"></a>

## <a id="x4-name-and-description"></a>4. Name and Description

[](#mapping_additional_nd)

The starting point of the name and description computation is a DOM [element](https://dom.spec.whatwg.org/#concept-element). The output is a flat, unstructured string that can be as simple as a single word, or a string of space-separated tokens. Examples include <code>Save</code> and <code>Reload from disk</code>.

An important factor is the [element's](https://dom.spec.whatwg.org/#concept-element) [role](https://www.w3.org/TR/wai-aria-1.3/#dfn-role), that determines which content contributes to the name string. Roles have a <code>nameFrom</code> RDF property, with three possible values:

<strong>author</strong>

name is generated from values provided by the author in explicit markup features such as the <code>aria-label</code> and <code>aria-labelledby</code> [attribute](https://dom.spec.whatwg.org/#concept-attribute), or a host language labeling mechanism, such as the <code>alt</code> or <code>title</code> [attribute](https://dom.spec.whatwg.org/#concept-attribute) in HTML, or the <code>desc</code> [element](https://dom.spec.whatwg.org/#concept-element) in SVG.

<strong>contents</strong>

name is generated from the [text nodes](https://dom.spec.whatwg.org/#exclusive-text-node) associated with the [element](https://dom.spec.whatwg.org/#concept-element). Although this may be allowed in addition to "author" in some [roles](https://www.w3.org/TR/wai-aria-1.3/#dfn-role), "content" is used only if higher priority "author" features are not provided. Priority is defined by the [text equivalent computation](#mapping_additional_nd_te) algorithm.

<strong>prohibited</strong>

the element has no name. Authors <em><em>MUST NOT</em></em> use the [<code>aria-label</code>](https://www.w3.org/TR/wai-aria-1.3/#aria-label) or [<code>aria-labelledby</code>](https://www.w3.org/TR/wai-aria-1.3/#aria-labelledby) attributes to name the element.

The [Accessible Rich Internet Applications (WAI-ARIA) 1.2](https://www.w3.org/TR/wai-aria-1.3/) \[[WAI-ARIA](https://www.w3.org/TR/accname-1.2/#bib-wai-aria)\] specification provides lists of [roles that support name from author](https://www.w3.org/TR/wai-aria-1.3/#namefromauthor), [roles that support name from content](https://www.w3.org/TR/wai-aria-1.3/#namefromcontent) and [roles that cannot be named](https://www.w3.org/TR/wai-aria-1.3/#namefromprohibited).

<a id="mapping&#95;additional&#95;nd&#95;name"></a>

### <a id="x4-1-name-computation"></a>4.1 Name Computation

[](#mapping_additional_nd_name)

[User agents](https://infra.spec.whatwg.org/#user-agent) <em>MUST</em> compute an <a id="ref-for-dfn-accessible-name-4"></a>[accessible name](#dfn-accessible-name) using the rules outlined below in the section titled [Text Equivalent Computation](#mapping_additional_nd_te).

<a id="mapping&#95;additional&#95;nd&#95;description"></a>

### <a id="x4-2-description-computation"></a>4.2 Description Computation

[](#mapping_additional_nd_description)

The following table provides the order of precedence for markup that can be applied to compute an <a id="ref-for-dfn-accessible-description-4"></a>[accessible description](#dfn-accessible-description). [User agents](https://infra.spec.whatwg.org/#user-agent) <em>MUST</em> use the first applicable entry from the table where the listed conditions are met, as described in the last column. The user agent <em>MUST NOT</em> use any markup other that the first relevant markup found, even if that markup results in an empty description:

<table>
<colgroup>
<col style="width: 25%" />
<col style="width: 25%" />
<col style="width: 25%" />
<col style="width: 25%" />
</colgroup>
<thead>
<tr class="header">
<th>Precedence</th>
<th>Attribute</th>
<th>Applicable conditions</th>
<th>How used to compute description</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<th><strong>1</strong></th>
<td><a href="https://www.w3.org/TR/wai-aria-1.3/#aria-describedby"><code>aria-describedby</code></a> attribute</td>
<td>Use on any element</td>
<td><a href="#mapping_additional_nd_name">Name computation</a> on all nodes referenced by aria-describedby on the element, concatenated, and separated by a space character</td>
</tr>
<tr class="even">
<th><strong>2</strong></th>
<td><a href="https://www.w3.org/TR/wai-aria-1.3/#aria-description"><code>aria-description</code></a> attribute</td>
<td>Use on any element</td>
<td>As a flat string</td>
</tr>
<tr class="odd">
<th><strong>3</strong></th>
<td>host language features which participate in the description calculation</td>
<td>Unique host language features <em>MAY</em> participate in the description computation for an element, only if they were not already used for the accessible name of the applicable element. See <a href="https://www.w3.org/TR/html-aam-1.0/#accdesc-computation">HTML AAM: Accessible Description Computation</a> for the HTML elements which meet this condition.</td>
<td>Either a <a href="#mapping_additional_nd_te">text equivalent computation</a> of the host language element, or the string value of the host language attribute.</td>
</tr>
<tr class="even">
<th><strong>4</strong></th>
<td>host language tooltip attribute or equivalent feature (e.g., HTML <code>title</code> attribute)</td>
<td><ul>
<li>Any element</li>
<li>Only use if not already used for the accessible name of this node</li>
</ul></td>
<td>As a flat string</td>
</tr>
</tbody>
</table>

<a id="mapping&#95;additional&#95;nd&#95;te"></a>

### <a id="x4-3-text-equivalent-computation"></a>4.3 Text Equivalent Computation

[](#mapping_additional_nd_te)

The text equivalent computation is used by both the <a id="ref-for-dfn-accessible-name-5"></a>[accessible name](#dfn-accessible-name) and <a id="ref-for-dfn-accessible-description-5"></a>[accessible description](#dfn-accessible-description). There are different rules provided for several different types of [elements](https://dom.spec.whatwg.org/#concept-element), [nodes](https://dom.spec.whatwg.org/#concept-node), and combinations of markup. Text alternatives are built up, when appropriate, from all the relevant content contained within an [element](https://dom.spec.whatwg.org/#concept-element). This is accomplished via steps 2.2 and 2.6, which are recursive, using the full set of rules to retrieve text from its own children or nodes it references.

The purpose of the computation is to create a [perceivable](https://www.w3.org/TR/wai-aria-1.2/#dfn-perceivable) label or description for alternative presentations, in the form of a flat string of space separated textual tokens.

<a id="terminology"></a>

#### <a id="x4-3-1-terminology"></a>4.3.1 Terminology

[](#terminology)

<strong>Root node</strong>

The DOM [node](https://dom.spec.whatwg.org/#concept-node) or [element](https://dom.spec.whatwg.org/#concept-element) for which the text alternative is sought.

<strong>Current node</strong>

The DOM [node](https://dom.spec.whatwg.org/#concept-node) currently traversed to compute the <code>root node</code>'s text equivalent. Initially, the <code>current node</code> is the <code>root node</code>, but at later stages is either some descendant of the <code>root node</code>, or another referenced node.

<strong>Rendered child nodes</strong>

The [nodes](https://dom.spec.whatwg.org/#concept-node) that are rendered as child nodes of a given node when taking [shadow roots](https://dom.spec.whatwg.org/#concept-shadow-root) and [slots](https://dom.spec.whatwg.org/#concept-slot) into consideration.

<strong>Flat string</strong>

A string of characters where all carriage returns, newlines, tabs, and form-feeds are replaced with a single space, and multiple spaces are reduced to a single space. The string contains only character data; it does not contain any markup.

<strong>Total accumulated text</strong>

The text equivalent computed up to, but not including the <code>current node</code>.

<strong>Accumulated text</strong>

Text accumulated at a step or sequence of steps described below. It is temporary storage for those steps.

<strong>Result</strong>

The text equivalent computed at one of the steps described below.

<strong>Append the result, without a space, to X</strong>

- If X is empty, copy the <code>result</code> to X.
- If X is non-empty, copy the <code>result</code> to the end of X.

<strong>Append the result, with a space, to X</strong>

- If X is empty, copy the <code>result</code> to X.
- If X is non-empty, add a space to the end of X and then copy the <code>result</code> to X after the space.

<strong>Prepend result, without a space, to X</strong>

- If X is empty, copy the <code>result</code> to X.
- If X is non-empty, copy the <code>result</code> to the start of X.

<strong>Prepend the result, with a space, to X</strong>

- If X is empty, copy the <code>result</code> to X.
- If X is non-empty, copy the <code>result</code> to the start of X, and add a space after the copy.

<a id="computation-steps"></a>

#### <a id="x4-3-2-computation-steps"></a>4.3.2 Computation steps

[](#computation-steps)

The text alternative for a given element is computed as follows:

1.  <a id="comp&#95;init"></a> <a id="step1"></a><em>Initialization:</em> Set the <code>root node</code> to the given element, the <code>current node</code> to the <code>root node</code>, and the <code>total accumulated text</code> to the empty string (""). If the <code>root node</code>'s role prohibits naming, return the empty string ("").
2.  <a id="comp&#95;computation"></a> <a id="step2"></a><em>Computation:</em> Compute the text alternative for the <code>current node</code>:
    1.  <a id="comp&#95;hidden&#95;not&#95;referenced"></a> <a id="step2A"></a><em>Hidden Not Referenced:</em> If the <code>current node</code> is [hidden](https://www.w3.org/TR/wai-aria/#dfn-hidden) <strong>and is</strong>:

        1.  <strong>Not</strong> part of an <code>aria-labelledby</code> or <code>aria-describedby</code> traversal, where the node directly referenced by that relation was hidden.
        2.  <strong>Nor</strong> part of a native host language text alternative [element](https://dom.spec.whatwg.org/#concept-element) (e.g. <code>label</code> in HTML) or [attribute](https://dom.spec.whatwg.org/#concept-attribute) traversal, where the root of that traversal was hidden.

        Return the empty string.

        <a id="issue-container-generatedID"></a>
        <strong>Note:</strong>

        <a id="h-note"></a>Note
        It's important to clarify the broad definition of hidden for the purposes of accessible name calculation:

        1.  Nodes with CSS properties <code>display&#58;none</code>, <code>visibility&#58;hidden</code>, <code>visibility&#58;collapse</code> or <code>content-visibility&#58;hidden</code>: They are considered hidden, as they match the guidelines "not perceivable" and "explicitly hidden".
        2.  Nodes with CSS properties <code>opacity&#58;0</code> or <code>filter&#58;opacity(0%)</code>, or similar SVG mechanisms: They are not considered hidden. Text hidden with these methods can still be selected or copied, and user agents still expose it in their accessibility trees.
        3.  Nodes with the <code>aria-hidden=&#34;true&#34;</code> property: it is considered hidden, matching the "explicitly hidden" guideline.
        4.  Nodes hidden off screen or behind another object: they are not considered hidden. They are exposed in the accessibility tree and they can even name on-screen objects.

        <a id="issue-container-generatedID-0"></a>
        <strong>Note:</strong>

        <a id="h-note-0"></a>Note
        By default, <a id="ref-for-dfn-assistive-technologies-3"></a>[assistive technologies](#dfn-assistive-technologies) do not relay hidden information, but an author can explicitly override that and include hidden text as part of the <a id="ref-for-dfn-accessible-name-6"></a>[accessible name](#dfn-accessible-name) or <a id="ref-for-dfn-accessible-description-6"></a>[accessible description](#dfn-accessible-description) by using <code>aria-labelledby</code> or <code>aria-describedby</code>.

        <a id="example-1"></a>

        <strong>Example:</strong>

        [Example 1](#example-1)
        The following examples show the meaning of the clause "Not part of an <code>aria-labelledby</code> or <code>aria-describedby</code> traversal, where the node directly referenced by that relation was hidden.".

        First, <code>element1</code>'s <a id="ref-for-dfn-accessible-name-7"></a>[accessible name](#dfn-accessible-name) is "hello" because, although <code>element3</code> is hidden, it is part of an <code>aria-labelledby</code> traversal started in <code>element2</code>, which is hidden too.

        <a id="example-1-0"></a>
        <strong>Example:</strong>

        [Example](#example-1-0)
        ``` text
        <element1 id="el1" role="button" aria-labelledby="el2" />
          <element2 id="el2" class="hidden">
            <element3 id="el3" class="hidden">hello</element3>
          </element2>
        ```

        <a id="example-2"></a>

        <strong>Example:</strong>

        [Example 2](#example-2)
        Conversely, <code>element1</code> has no <a id="ref-for-dfn-accessible-name-8"></a>[accessible name](#dfn-accessible-name) if <code>element3</code> is hidden and it is part of an <code>aria-labelledby</code> traversal started in <code>element2</code>, but <code>element2</code> is not hidden.

        ``` text
        <element1 id="el1" role="button" aria-labelledby="el2" />
        <element2 id="el2">
          <element3 id="el3" class="hidden">hello</element3>
        </element2>
        ```

    2.  <a id="comp&#95;labelledby"></a> <a id="step2B"></a><em>LabelledBy:</em> Otherwise, if the <code>current node</code> has an <code>aria-labelledby</code> [attribute](https://dom.spec.whatwg.org/#concept-attribute) that contains at least one valid IDREF, and the <code>current node</code> is not already part of an ongoing <code>aria-labelledby</code> or <code>aria-describedby</code> traversal, process its IDREFs in the order they occur:
        1.  <a id="comp&#95;labelledby&#95;reset"></a> <a id="step2B.i"></a> Set the <code>accumulated text</code> to the empty string.
        2.  <a id="comp&#95;labelledby&#95;foreach"></a> <a id="step2B.ii"></a>For each IDREF:
            1.  <a id="comp&#95;labelledby&#95;set&#95;current"></a> <a id="step2B.ii.a"></a>Set the <code>current node</code> to the node referenced by the IDREF.
            2.  <a id="comp&#95;labelledby&#95;traversal"></a> <a id="step2B.ii.b"></a><em>LabelledBy Traversal:</em> Compute the text alternative of the <code>current node</code> beginning with the overall [Computation](#comp_computation) step. Set the <code>result</code> to that text alternative.
            3.  <a id="comp&#95;labelledby&#95;append"></a> <a id="step2B.ii.c"></a>Append a space character and the <code>result</code> to the <code>accumulated text</code>.
        3.  <a id="comp&#95;labelledby&#95;return"></a> <a id="step2B.iii"></a>Return the <code>accumulated text</code> if it is not the empty string ("").

        The result of [LabelledBy Traversal](#comp_labelledby_traversal) in combination with [Hidden Not Referenced](#comp_hidden_not_referenced) means that [user agents](https://infra.spec.whatwg.org/#user-agent) <em>MUST</em> include all nodes in the subtree as part of the <a id="ref-for-dfn-accessible-name-9"></a>[accessible name](#dfn-accessible-name) or <a id="ref-for-dfn-accessible-description-7"></a>[accessible description](#dfn-accessible-description), when the node referenced by <code>aria-labelledby</code> or <code>aria-describedby</code> is hidden.

        <a id="example-3"></a>
        <strong>Example:</strong>

        [Example 3](#example-3)
        The following example shows the meaning of the clause "… and the <code>current node</code> is not already part of an <code>aria-labelledby</code> traversal …" .

        1.  <code>element1</code>'s <a id="ref-for-dfn-accessible-name-10"></a>[accessible name](#dfn-accessible-name) is "hello" because this is a first traversal of its <code>aria-labelledby</code>, leading to <code>element3</code>.
        2.  <code>element2</code> has no <a id="ref-for-dfn-accessible-name-11"></a>[accessible name](#dfn-accessible-name). The computation involves a first traversal of its <code>aria-labelledby</code> leading to <code>element1</code>, but <code>element1</code>'s <code>aria-labelledby</code> is not subsequently followed.

        ``` text
        <element1 id="el1" aria-labelledby="el3" />
        <element2 id="el2" aria-labelledby="el1" />
        <element3 id="el3"> hello </element3>
        ```

    3.  <a id="comp&#95;embedded&#95;control"></a> <a id="step2C"></a><em>Embedded Control:</em> Otherwise, if the <code>current node</code> is a control embedded within the label (e.g. any element directly referenced by <code>aria-labelledby</code>) for another [widget](https://www.w3.org/TR/wai-aria/#dfn-widget), where the user can adjust the embedded control's value, then return the embedded control as part of the text alternative in the following manner:

        1.  <a id="comp&#95;embedded&#95;control&#95;textbox"></a><em>Textbox:</em> If the embedded control has role [textbox](https://www.w3.org/TR/wai-aria-1.3/#textbox), return its value.
        2.  <a id="comp&#95;embedded&#95;control&#95;combobox&#95;or&#95;listbox"></a> <em>Combobox/Listbox:</em> If the embedded control has role [combobox](https://www.w3.org/TR/wai-aria-1.3/#combobox) or [listbox](https://www.w3.org/TR/wai-aria-1.3/#listbox), return the text alternative of the chosen [option](https://www.w3.org/TR/wai-aria-1.3/#option).
        3.  <a id="comp&#95;embedded&#95;control&#95;range"></a> <em>Range:</em> If the embedded control has role [range](https://www.w3.org/TR/wai-aria-1.3/#range) (e.g., a [spinbutton](https://www.w3.org/TR/wai-aria-1.3/#spinbutton) or [slider](https://www.w3.org/TR/wai-aria-1.3/#slider)):
            1.  <a id="comp&#95;embedded&#95;control&#95;range&#95;valuetext"></a>If the <code>aria-valuetext</code> property is present, return its value,
            2.  <a id="comp&#95;embedded&#95;control&#95;range&#95;valuenow"></a>Otherwise, if the <code>aria-valuenow</code> property is present, return its value,
            3.  <a id="comp&#95;embedded&#95;control&#95;range&#95;host&#95;language&#95;value"></a>Otherwise, use the value as specified by a host language attribute.

        <a id="example-4"></a>

        <strong>Example:</strong>

        [Example 4](#example-4)
        Consider a [check box](https://www.w3.org/TR/wai-aria-1.3/#checkbox) label that contains a text input field: "Flash the screen \[input\] times". If the input field has the value "5", the complete label is "Flash the screen 5 times", e.g.:

        ``` text
        <label for="flash">
          <input type="checkbox" id="flash">
          Flash the screen <span tabindex="0" role="textbox" aria-label="number of times" contenteditable>5</span> times.
        </label>
        ```

    4.  <a id="comp&#95;label"></a> <a id="step2D"></a><em>AriaLabel:</em> Otherwise, if the <code>current node</code> is not a [slot](https://dom.spec.whatwg.org/#concept-slot) element and has an <code>aria-label</code> [attribute](https://dom.spec.whatwg.org/#concept-attribute) whose value is not undefined, not the empty string, nor, when trimmed of [whitespace](https://infra.spec.whatwg.org/#ascii-whitespace), is not the empty string:

        1.  If traversal of the <code>current node</code> is due to [Name From Content descendant node recursion](#comp_name_from_content_for_each_child) <strong>and</strong> the <code>current node</code> is an embedded control, ignore <code>aria-label</code> and skip to rule [Embedded Control](#comp_embedded_control).
        2.  Otherwise, return the value of <code>aria-label</code>.

        <a id="example-5"></a>

        <strong>Example:</strong>

        [Example 5](#example-5)
        The following example shows the interaction of <code>aria-labelledby</code> and <code>aria-label</code> when a [node](https://dom.spec.whatwg.org/#concept-node) has an <code>aria-labelledby</code> that refers to itself. The <code>&#60;span role=&#34;button&#34;&#62;</code> elements have the <a id="ref-for-dfn-accessible-name-12"></a>[accessible names](#dfn-accessible-name) "Delete Documentation.pdf" and "Delete HolidayLetter.pdf", respectively.

        <a id="example-5-0"></a>
        <strong>Example:</strong>

        [Example](#example-5-0)
        ``` text
        <h1>Files</h1>
        <ul>
          <li>
            <a id="file_row1" href="./files/Documentation.pdf">Documentation.pdf</a>
            <span role="button" tabindex="0" id="del_row1" aria-label="Delete" aria-labelledby="del_row1 file_row1"></span>
          </li>
          <li>
            <a id="file_row2" href="./files/HolidayLetter.pdf">HolidayLetter.pdf</a>
            <span role="button" tabindex="0" id="del_row2" aria-label="Delete" aria-labelledby="del_row2 file_row2"></span>
          </li>
        </ul>
        ```

    5.  <a id="comp&#95;host&#95;language&#95;label"></a> <a id="step2E"></a><em>Host Language Label:</em> Otherwise, if the <code>current node</code>'s native markup provides an [attribute](https://dom.spec.whatwg.org/#concept-attribute) (e.g. <code>alt</code>) or [element](https://dom.spec.whatwg.org/#concept-element) (e.g. HTML <code>label</code> or SVG <code>title</code>) that defines a text alternative, return that alternative in the form of a <code>flat string</code> as defined by the host language (e.g., [HTML-AAM](https://www.w3.org/TR/html-aam-1.0/#accessible-name-and-description-computation)), unless the <code>current node</code> is exposed as presentational (<code>role=&#34;presentation&#34;</code> or <code>role=&#34;none&#34;</code>, <code>alt=&#34;&#34;</code>).
        <a id="issue-container-generatedID-1"></a>
        <strong>Note:</strong>

        <a id="h-note-1"></a>Note
        Important! The accessible name for native HTML elements may be calculated differently depending on host language. Make sure to review [HTML-AAM](https://www.w3.org/TR/html-aam-1.0/#accessible-name-and-description-computation) for more information.
        <a id="issue-container-generatedID-2"></a>
        <strong>Note:</strong>

        <a id="h-note-2"></a>Note
        For example, in HTML, the <code>img</code> element's <code>alt</code> attribute defines a text alternative string, and the <code>label</code> element provides text for the referenced form element. In SVG2, the <code>desc</code> and <code>title</code> elements provide a description of their parent element.

    6.  <a id="comp&#95;name&#95;from&#95;content"></a> <a id="step2F"></a><em>Name From Content:</em> Otherwise, if the <code>current node's</code> [role](https://www.w3.org/TR/wai-aria-1.3/#dfn-role) allows [name from content](https://www.w3.org/TR/wai-aria-1.3/#namefromcontent), or if the <code>current node</code> is referenced by <code>aria-labelledby</code>, <code>aria-describedby</code>, or is a native host language text alternative [element](https://dom.spec.whatwg.org/#concept-element) (e.g. <code>label</code> in HTML), or is a descendant of a native host language text alternative [element](https://dom.spec.whatwg.org/#concept-element):

        1.  <a id="comp&#95;name&#95;from&#95;content&#95;reset"></a> <a id="step2F.i"></a><em>Name From Content Reset:</em> Set the <code>accumulated text</code> to the empty string.
        2.  <a id="comp&#95;name&#95;from&#95;content&#95;pseudo&#95;element"></a> <a id="step2F.ii"></a><em>Name From Generated Content:</em> Check for CSS generated textual content associated with the <code>current node</code> and include it in the <code>accumulated text</code>. The CSS [<code>&#58;&#58;before</code> and <code>&#58;&#58;after</code>](https://www.w3.org/TR/selectors-3/#gen-content) pseudo elements \[[CSS2](https://www.w3.org/TR/accname-1.2/#bib-css2)\] can provide textual content for [elements](https://dom.spec.whatwg.org/#concept-element) that have a content model, in addition to the [<code>&#58;&#58;marker</code>](https://www.w3.org/TR/css-pseudo-4/#selectordef-marker) pseudo element.
            1.  <a id="comp&#95;name&#95;from&#95;content&#95;pseudo&#95;element&#95;marker"></a> For <code>&#58;&#58;marker</code> pseudo elements, [User agents](https://infra.spec.whatwg.org/#user-agent) <em>MUST</em> prepend CSS textual content, without a space, to the textual content of the <code>current node</code> if the <code>current node</code> supports <code>&#58;&#58;marker</code>.
            2.  <a id="comp&#95;name&#95;from&#95;content&#95;pseudo&#95;element&#95;before"></a> For <code>&#58;&#58;before</code> pseudo elements, [User agents](https://infra.spec.whatwg.org/#user-agent) <em>MUST</em> prepend CSS textual content, without a space, to the textual content of the <code>current node</code>.
            3.  <a id="comp&#95;name&#95;from&#95;content&#95;pseudo&#95;element&#95;after"></a> For <code>&#58;&#58;after</code> pseudo elements, [User agents](https://infra.spec.whatwg.org/#user-agent) <em>MUST</em> append CSS textual content, without a space, to the textual content of the <code>current node</code>.
        3.  <a id="comp&#95;name&#95;from&#95;content&#95;find&#95;child"></a> <em>Determine Child Nodes:</em> Determine the <code>rendered child nodes</code> of the <code>current node</code>:
            1.  <a id="comp&#95;name&#95;from&#95;content&#95;find&#95;child&#95;has&#95;shadow&#95;root"></a> If the <code>current node</code> has an attached [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root), set the <code>rendered child nodes</code> to be the child nodes of the [shadow root](https://dom.spec.whatwg.org/#concept-shadow-root).
            2.  <a id="comp&#95;name&#95;from&#95;content&#95;find&#95;child&#95;is&#95;slot"></a> Otherwise, if the <code>current node</code> is a [slot](https://dom.spec.whatwg.org/#concept-slot) with [assigned nodes](https://dom.spec.whatwg.org/#slot-assigned-nodes), set the <code>rendered child nodes</code> to be the [assigned nodes](https://dom.spec.whatwg.org/#slot-assigned-nodes) of the <code>current node</code>.
            3.  <a id="comp&#95;name&#95;from&#95;content&#95;find&#95;child&#95;otherwise"></a>Otherwise, set the <code>rendered child nodes</code> to be the child nodes of the <code>current node</code>.
        4.  <a id="comp&#95;name&#95;from&#95;content&#95;for&#95;each&#95;child"></a> <a id="step2F.iii"></a><em>Name From Each Child:</em> For each <code>rendered child node</code> of the <code>current node</code>:
            1.  <a id="comp&#95;name&#95;from&#95;content&#95;for&#95;each&#95;child&#95;set&#95;current"></a> <a id="step2F.iii.a"></a>Set the <code>current node</code> to the <code>rendered child node</code>.
            2.  <a id="comp&#95;name&#95;from&#95;content&#95;for&#95;each&#95;child&#95;recursion"></a> <a id="step2F.iii.b"></a>Compute the text alternative of the <code>current node</code> beginning with the overall [Computation](#comp_computation) step. Set the <code>result</code> to that text alternative.
            3.  <a id="comp&#95;for&#95;each&#95;child&#95;append"></a> <a id="step2F.iii.c"></a>Append the <code>result</code> to the <code>accumulated text</code>.
        5.  <a id="comp&#95;name&#95;from&#95;content&#95;return"></a> <a id="step2F.iv"></a>Return the <code>accumulated text</code> if it is not the empty string ("").

        <strong>Important</strong>: Each [node](https://dom.spec.whatwg.org/#concept-node) in the subtree is consulted only once. If text has been collected from a descendant, but is referenced by another IDREF in some descendant node, then that second, or subsequent, reference is not followed. This is done to avoid infinite loops.

        <a id="issue-container-generatedID-3"></a>
        <strong>Note:</strong>

        <a id="h-note-3"></a>Note
        This step can apply to the child nodes themselves, which means the computation is recursive and results in text collected from all the elements in the <code>current node</code>'s subtree, no matter how deep it is. However, any given descendant [node's](https://dom.spec.whatwg.org/#concept-node) text alternative can result from higher precedent markup described in steps B through D above, where "Namefrom: author" attributes provide the text alternative for the entire subtree.

        <a id="note-concatenate-inline-contents-with-space"></a>
        <strong>Note:</strong>

        <a id="h-note-4"></a>Note
        18 January 2024: The ARIA Working Group is considering the feasibility of joining text strings with and without spaces, depending on the CSS <code>display</code> value of the <code>current node</code>, and its adjacent nodes and pseudo-elements. The ongoing discussion is in [AccName \#225](https://github.com/w3c/accname/issues/225).

    7.  <a id="comp&#95;text&#95;node"></a> <a id="step2G"></a><em>Text Node:</em> Otherwise, if the <code>current node</code> is a [text node](https://dom.spec.whatwg.org/#exclusive-text-node), return its textual contents.

    8.  <a id="comp&#95;recursive&#95;name&#95;from&#95;content"></a> <a id="step2H"></a><em>Recursive Name From Content:</em> Otherwise, if the <code>current node</code> is a descendant of an element whose <a id="ref-for-dfn-accessible-name-13"></a>[Accessible Name](#dfn-accessible-name) or <a id="ref-for-dfn-accessible-description-8"></a>[Accessible Description](#dfn-accessible-description) is being computed, and contains descendants, proceed to [Name From Content Reset](#comp_name_from_content_reset).

    9.  <a id="comp&#95;tooltip"></a> <a id="step2I"></a><em>Tooltip:</em> Otherwise, if the <code>current node</code> has a <a id="ref-for-dfn-tooltip-attribute-1"></a>[Tooltip attribute](#dfn-tooltip-attribute), return its value.
        <a id="issue-container-generatedID-4"></a>
        <strong>Note:</strong>

        <a id="h-note-5"></a>Note
        Tooltip attributes are used only if nothing else, including subtree content, has provided results.

    10. <a id="comp&#95;append"></a>Append a space character and the <code>result</code> of each step above to the <code>total accumulated text</code>.
3.  <a id="comp&#95;complete"></a> After all steps are completed, the <code>total accumulated text</code> is used as the <a id="ref-for-dfn-accessible-name-14"></a>[accessible name](#dfn-accessible-name) or <a id="ref-for-dfn-accessible-description-9"></a>[accessible description](#dfn-accessible-description) of the [element](https://dom.spec.whatwg.org/#concept-element) that initiated the computation.

<a id="accessible-name-and-description-mapping"></a>

## <a id="x5-accessible-name-and-description-mapping"></a>5. Accessible Name and Description Mapping

[](#accessible-name-and-description-mapping)

Information concerning name and description accessibility API mappings, including relationships, such as labelled-by/label-for and described-by/description-for, is documented in the [Core Accessibility API Mappings](https://www.w3.org/TR/core-aam-1.2/) specification \[[CORE-AAM-1.2](https://www.w3.org/TR/accname-1.2/#bib-core-aam-1-2)\]. See the mapping table entries for [<code>aria-label</code>](https://www.w3.org/TR/core-aam-1.2/#ariaLabel), [<code>aria-labelledby</code>](https://www.w3.org/TR/core-aam-1.2/#ariaLabelledBy), and [<code>aria-describedby</code>](https://www.w3.org/TR/core-aam-1.2/#ariaDescribedBy).
