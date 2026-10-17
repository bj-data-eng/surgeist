# Selection API: Range association source excerpts

Source: [https://www.w3.org/TR/2026/WD-selection-api-20260611/](https://www.w3.org/TR/2026/WD-selection-api-20260611/), W3C Working Draft, 11 June 2026.
Retrieved 2026-10-17. Complete downloaded HTML SHA256: `616183cecb1e25d14ceca692b0279e2c6404b88973c2e94c80757eee40cd0243`.

Copyright © 2026 World Wide Web Consortium. [W3C Software and Document Notice and License](https://www.w3.org/copyright/software-license-2023/) applies.

Bounded Markdown conversion using Pandoc 3.1.11.1, retaining complete Definition, addRange method (including notes), and Document interface extension sections. Presentation spans were removed and source links made explicit. These excerpts contain no tables. Other Selection methods and event algorithms are outside this extract, not waived. This is source evidence; #1243 owns the selected bounded native contract, including its distinction from browser getSelection and its explicit retirement policy.

---

## 2. Definition

Every [document](https://dom.spec.whatwg.org/#document) with a [browsing context](https://html.spec.whatwg.org/multipage/document-sequences.html#concept-document-bc) has a unique selection associated with it.

Note

This is a requirement of the HTML spec. IE9 and Opera Next 12.00 alpha seem to follow it, while Firefox 12.0a1 and Chrome 17 dev seem not to. See [Mozilla bug](https://bugzilla.mozilla.org/show_bug.cgi?id=717339), [WebKit bug](https://bugs.webkit.org/show_bug.cgi?id=76114).

This one [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) must be shared by all the content of the [document](https://dom.spec.whatwg.org/#document) (though not by nested [documents](https://dom.spec.whatwg.org/#document)), including any [editing hosts](https://html.spec.whatwg.org/multipage/interaction.html#editing-host) in the [document](https://dom.spec.whatwg.org/#document).

Each [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) can be associated with a single [range](https://dom.spec.whatwg.org/#concept-range). When there is no [range](https://dom.spec.whatwg.org/#concept-range) associated with the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection), the selection is empty. The selection must be initially [empty](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-empty).

Note

A [document](https://dom.spec.whatwg.org/#document)'s [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) is a singleton object associated with that [document](https://dom.spec.whatwg.org/#document), so it gets replaced with a new object when `Document.open()` is called. See [bug 15470](https://www.w3.org/Bugs/Public/show_bug.cgi?id=15470). IE9 and Opera Next 12.00 alpha allow the user to reset the range to null after the fact by clicking somewhere; Firefox 12.0a1 and Chrome 17 dev do not. We follow Gecko/WebKit, because it lessens the chance of getRangeAt(0) throwing.

Once a [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) is associated with a given [range](https://dom.spec.whatwg.org/#concept-range), it must continue to be associated with that same [range](https://dom.spec.whatwg.org/#concept-range) until this specification requires otherwise.

Note

For instance, if the DOM changes in a way that changes the range's boundary points, or a script modifies the boundary points of the range, the same range object must continue to be associated with the selection. However, if the user changes the selection or a script calls [`addRange`](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dom-selection-addrange)`()`, the selection must be associated with a new range object, as required elsewhere in this specification.

If the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection)'s [range](https://dom.spec.whatwg.org/#concept-range) is not null and is [collapsed](https://dom.spec.whatwg.org/#range-collapsed), then the caret position must be at that [range](https://dom.spec.whatwg.org/#concept-range)'s [boundary point](https://dom.spec.whatwg.org/#concept-range-bp). When the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) is not [collapsed](https://dom.spec.whatwg.org/#range-collapsed), this specification does not define the caret position; user agents should follow platform conventions in deciding whether the caret is at the start of the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection), the end of the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection), or somewhere else.

Each [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) has a direction: forwards, backwards, or directionless. If the user creates a [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) by indicating first one [boundary point](https://dom.spec.whatwg.org/#concept-range-bp) of the [range](https://dom.spec.whatwg.org/#concept-range) and then the other (such as by clicking on one point and dragging to another), and the first indicated [boundary point](https://dom.spec.whatwg.org/#concept-range-bp) is [after](https://dom.spec.whatwg.org/#concept-range-bp-after) the second, then the corresponding [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) must initially be [backwards](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-backwards). If the first indicated [boundary point](https://dom.spec.whatwg.org/#concept-range-bp) is [before](https://dom.spec.whatwg.org/#concept-range-bp-before) the second, then the corresponding [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) must initially be [forwards](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-forwards). Otherwise, it must be [directionless](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-directionless).

When the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection)'s [range](https://dom.spec.whatwg.org/#concept-range) is mutated by scripts, e.g. via [`selectNode`](https://dom.spec.whatwg.org/#dom-range-selectnode)`(node)`, [direction](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-direction) of the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) must be preserved.

Each [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection)s also have an anchor and a focus. If the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection)'s [range](https://dom.spec.whatwg.org/#concept-range) is null, its [anchor](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-anchor) and [focus](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-focus) are both null. If the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection)'s [range](https://dom.spec.whatwg.org/#concept-range) is not null and its [direction](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-direction) is [forwards](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-forwards), its [anchor](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-anchor) is the [range](https://dom.spec.whatwg.org/#concept-range)'s [start](https://dom.spec.whatwg.org/#concept-range-start), and its [focus](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-focus) is the [end](https://dom.spec.whatwg.org/#concept-range-end). Otherwise, its [focus](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-focus) is the [start](https://dom.spec.whatwg.org/#concept-range-start) and its [anchor](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-anchor) is the [end](https://dom.spec.whatwg.org/#concept-range-end).

Note

[anchor](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-anchor) and [focus](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-focus) of [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) need not to be in the [document tree](https://dom.spec.whatwg.org/#concept-document-tree). It could be in a [shadow tree](https://dom.spec.whatwg.org/#concept-shadow-tree) of the same [document](https://dom.spec.whatwg.org/#document).

Each [document](https://dom.spec.whatwg.org/#document), [input](https://html.spec.whatwg.org/multipage/input.html#the-input-element) element, and [textarea](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element) element has a boolean has scheduled selectionchange event, which is initially false.

`addRange()` method
The method must follow these steps:

1.  If the [root](https://dom.spec.whatwg.org/#concept-tree-root) of the `range`'s boundary points are not the [document](https://dom.spec.whatwg.org/#document) associated with [this](https://webidl.spec.whatwg.org/#this), abort these steps.
2.  If `rangeCount` is not `0`, abort these steps.
3.  Set [this](https://webidl.spec.whatwg.org/#this)'s range to `range` by a strong reference (not by making a copy).

Note

Since [range](https://dom.spec.whatwg.org/#concept-range) is added by reference, subsequent calls to `getRangeAt(0)` returns the same object, and any changes that a script makes to [range](https://dom.spec.whatwg.org/#concept-range) after it is added must be reflected in the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection), until something else removes or replaces [this](https://webidl.spec.whatwg.org/#this)'s [range](https://dom.spec.whatwg.org/#concept-range). In particular, the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) will contain `b` as opposed to `a` after running the following code: `var r = document.createRange(); r.selectNode(a); getSelection().addRange(r); r.selectNode(b);`

Note

At Step 2, Chrome 58 and Edge 25 do nothing. Firefox 51 gives you a multi-range selection. At least they keep the exisiting [range](https://dom.spec.whatwg.org/#concept-range).

At Step 3, Chrome 58 and Firefox 51 store a reference, as described here. Edge 25 stores a copy. Firefox 51 changes its selection if the [range](https://dom.spec.whatwg.org/#concept-range) is modified.

## 4.1 Extensions to `Document` interface

The [`Document`](https://dom.spec.whatwg.org/#document) interface is defined in \[[HTML](https://www.w3.org/TR/2026/WD-selection-api-20260611/#bib-html)\].

    partial interface Document {
      Selection? getSelection();
    };

`getSelection()` method
The method must return the [selection](https://www.w3.org/TR/2026/WD-selection-api-20260611/#dfn-selection) associated with [this](https://webidl.spec.whatwg.org/#this) if [this](https://webidl.spec.whatwg.org/#this) has an associated [browsing context](https://html.spec.whatwg.org/multipage/document-sequences.html#concept-document-bc), and it must return `null` otherwise.
