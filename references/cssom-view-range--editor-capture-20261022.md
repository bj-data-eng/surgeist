# CSSOM View: current Range interface source excerpt

Source: [CSSOM View Editor’s Draft](https://drafts.csswg.org/cssom-view/), displayed date 8 October 2026; captured 22 October 2026. This is a mutable editor capture, not a dated W3C Recommendation.

Complete captured HTML: 1087595 bytes; SHA-256 `71d9ce94f21ec00a5a333ee5a20d2b0f999a862d3543fbdb85cd4d1d3dcac176`.

Copyright © 2026 World Wide Web Consortium. [W3C document license](https://www.w3.org/copyright/document-license/) applies. [Original copyright notice](https://drafts.csswg.org/cssom-view/#copyright).

Bounded Markdown conversion of complete §9, including its IDL, algorithms, note and test links. A task-local Python standard-library HTML parser preserved link destinations and text, normalized presentation whitespace, and preserved ordered versus unordered lists; the algorithm order is unchanged. There are no tables in this section. The surrounding specification is outside this extract. This capture supplements the older dated edition for the current Range source check; it does not claim a complete specification refresh. See the [resolution and test evidence](range-geometry-csswg-tests--20261022.md).

---

## 9. Extensions to the [Range](https://dom.spec.whatwg.org/#range) Interface

```webidl
partial interface Range {
DOMRectList getClientRects();
[NewObject] DOMRect getBoundingClientRect();
};
```

The `getClientRects()` method, when invoked, must return an empty [DOMRectList](https://drafts.csswg.org/geometry-1/#domrectlist) object if the range is not in the document and otherwise a [DOMRectList](https://drafts.csswg.org/geometry-1/#domrectlist) object containing a list of [DOMRect](https://drafts.csswg.org/geometry-1/#domrect) objects in content order that matches the following constraints:

- For each element selected by the range, whose parent is not selected by the range, include the border areas returned by invoking [getClientRects()](https://drafts.csswg.org/cssom-view/#dom-element-getclientrects) on the element.

- For each [Text](https://dom.spec.whatwg.org/#text) node selected or partially selected by the range (including when the boundary-points are identical), include [scaled](https://drafts.csswg.org/css-viewport/#scaled) [DOMRect](https://drafts.csswg.org/geometry-1/#domrect) object (for the part that is selected, not the whole line box). The bounds of these [DOMRect](https://drafts.csswg.org/geometry-1/#domrect) objects are computed using font metrics; thus, for horizontal writing, the vertical dimension of each box is determined by the font ascent and descent, and the horizontal dimension by the text advance width. If the range covers a partial [typographic character unit](https://drafts.csswg.org/css-text-4/#typographic-character-unit) (e.g. half a surrogate pair or part of a grapheme cluster), the full typographic character unit must be included for the purpose of computing the bounds of the relevant [DOMRect](https://drafts.csswg.org/geometry-1/#domrect). [[CSS-TEXT-3]](https://drafts.csswg.org/cssom-view/#biblio-css-text-3) The [transforms](https://drafts.csswg.org/cssom-view/#transforms) that apply to the ancestors are applied.

Note: The [DOMRect](https://drafts.csswg.org/geometry-1/#domrect) objects returned by [getClientRects()](https://drafts.csswg.org/cssom-view/#dom-range-getclientrects) are not [live](https://html.spec.whatwg.org/multipage/infrastructure.html#live).

Tests

- [range-bounding-client-rect-with-nested-text.html](https://wpt.fyi/results/css/cssom-view/range-bounding-client-rect-with-nested-text.html) [(live test)](http://wpt.live/css/cssom-view/range-bounding-client-rect-with-nested-text.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom-view/range-bounding-client-rect-with-nested-text.html)
- [range-client-rects-surrogate-indexing.html](https://wpt.fyi/results/css/cssom-view/range-client-rects-surrogate-indexing.html) [(live test)](http://wpt.live/css/cssom-view/range-client-rects-surrogate-indexing.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom-view/range-client-rects-surrogate-indexing.html)

The `getBoundingClientRect()` method, when invoked, must return the result of the following algorithm:

1. Let list be the result of invoking [getClientRects()](https://drafts.csswg.org/cssom-view/#dom-range-getclientrects) on the same range this method was invoked on.

- If list is empty return a [DOMRect](https://drafts.csswg.org/geometry-1/#domrect) object whose [x](https://drafts.csswg.org/geometry-1/#dom-domrect-x), [y](https://drafts.csswg.org/geometry-1/#dom-domrect-y), [width](https://drafts.csswg.org/geometry-1/#dom-domrect-width) and [height](https://drafts.csswg.org/geometry-1/#dom-domrect-height) members are zero.

- If all rectangles in list have zero width or height, return the first rectangle in list.

- Otherwise, return a [DOMRect](https://drafts.csswg.org/geometry-1/#domrect) object describing the smallest rectangle that includes all of the rectangles in list of which the height or width is not zero.

Note: The [DOMRect](https://drafts.csswg.org/geometry-1/#domrect) object returned by [getBoundingClientRect()](https://drafts.csswg.org/cssom-view/#dom-range-getboundingclientrect) is not [live](https://html.spec.whatwg.org/multipage/infrastructure.html#live).

Tests

- [range-bounding-client-rect-with-display-contents.html](https://wpt.fyi/results/css/cssom-view/range-bounding-client-rect-with-display-contents.html) [(live test)](http://wpt.live/css/cssom-view/range-bounding-client-rect-with-display-contents.html) [(source)](https://github.com/web-platform-tests/wpt/blob/master/css/cssom-view/range-bounding-client-rect-with-display-contents.html)
