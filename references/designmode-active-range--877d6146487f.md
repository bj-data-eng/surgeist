# designMode: nullable active range and minimum range ownership

Bounded source excerpts and evidence summary prepared on 16 October 2026. These sources support the designMode determination in [#1124](https://github.com/bj-data-eng/surgeist/issues/1124); they do not provide a complete Selection API or editor specification.

## Source basis and attribution

- HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`: the complete selected setter is already retained in [HTML editability](html-editability--877d6146487f.md). Its focus algorithm is retained in [HTML host facts](html-language-host-facts--877d6146487f--2dd822df9d5e.md), corresponding to full-source raw lines 60164–60184.
- Editing `85de9ff4a9816139d06fc2dc39d5dacdc456fddc`: [original execCommand source](https://github.com/w3c/editing/blob/85de9ff4a9816139d06fc2dc39d5dacdc456fddc/docs/execCommand/index.html), retained complete HTML SHA-256 `da222a4ea70349eba9d9036a20708f1fa63690118f6c657da3f7f06c56fa78cf`, raw lines 1329–1335. The [existing editing reference](execcommand-editability--85de9ff4a981.md) retains the original 2015 W3C Editing APIs Community Group notice, CC0/CLA statement and repository W3C Software and Document License notice; those notices also apply to the excerpt below. No editing-command execution is selected.
- DOM `3071e5f269448350fdea79bb284eedaf17bc431e`: [immutable DOM snapshot](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/), retained complete snapshot HTML SHA-256 `9221bf5c72d0399e11749f8c8d5c01b60a45fa045076b02935ac298119ea7be1`, raw lines 12330–12337, 12402–12424 and 12586–12596. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft); prose is [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), with the [exact WHATWG license](../licenses/whatwg/LICENSE).

The excerpts below are manually converted to Markdown with whitespace and emphasis normalized. Original source terminology is preserved. The editing draft's unresolved range token and missing selection/getSelection fragment targets are not silently repaired. No semantic change or endorsement is implied.

## Editing: active range

The **active range** is the `[=range=]` of the [selection](https://github.com/w3c/editing/blob/85de9ff4a9816139d06fc2dc39d5dacdc456fddc/docs/execCommand/index.html#concept-selection) given by calling [getSelection()](https://github.com/w3c/editing/blob/85de9ff4a9816139d06fc2dc39d5dacdc456fddc/docs/execCommand/index.html#dom-document-getselection) on the [context object](http://dom.spec.whatwg.org/#context-object). (Thus the active range may be null.)

The retained source does not define those two local selection/getSelection targets. This excerpt establishes the explicit nullable active-range statement, not the missing algorithms.

## DOM: boundary points and ranges

A **boundary point** is a tuple consisting of a node (a node) and an offset (a non-negative integer).

A correct boundary point's offset will be between 0 and the boundary point's node's length, inclusive.

Objects implementing the AbstractRange interface are known as **ranges**.

A range has two associated boundary points — a **start** and **end**.

For convenience, a range's **start node** is its start's node, its **start offset** is its start's offset, its **end node** is its end's node, and its **end offset** is its end's offset.

A range is **collapsed** if its start node is its end node and its start offset is its end offset.

Objects implementing the Range interface are known as **live ranges**.

Algorithms that modify a tree (in particular the insert, remove, move, replace data, and split algorithms) modify live ranges associated with that tree.

The **root** of a live range is the root of its start node.

Original anchors: [boundary point](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#concept-range-bp), [range](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#concept-range), [live range](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#concept-live-range). This excerpt establishes the data and live-update obligation; it does not reproduce all range mutation algorithms.

## Pinned WebKit witness and limits

Inspected [WebKit Document.cpp](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/dom/Document.cpp) at `73aa6c89e2cb77c46184a81aec944e4ab99d114d`; exact original SHA-256 `c3f64c3d98f23601ffa74b2e63b82f30aaeb4035bb537c657475a8c00095c55c`. This section is behavioral metadata, not redistributed WebKit source code. Original LGPL source remains in ignored research material, as with the [existing WebKit Document reference](webkit-document-title--73aa6c89e2cb-metadata.md).

At original-file lines 8077–8084, setDesignMode assigns the mode, schedules a style rebuild and notifies an existing accessibility cache. It does not implement HTML's explicit range-reset/focus sequence. It also selects Off for inputs other than ASCII-insensitive On, so its invalid-input behavior is not the selected HTML setter behavior. At lines 10793–10796, getSelection returns null when the Document has no DOMWindow; otherwise it delegates to that window.

This witness does not establish a standards-defined null-range branch. It offers no basis to import omission of range reset or focusing for non-null state. The selected Surgeist null-range behavior is recorded explicitly in #1124, separately from the source evidence.
