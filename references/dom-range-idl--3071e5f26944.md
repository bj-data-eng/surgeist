# DOM live Range Web IDL source excerpts

Copyright WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is CC BY 4.0. Source incorporation follows the retained [WHATWG license](../licenses/whatwg/LICENSE).

Exact DOM revision `3071e5f269448350fdea79bb284eedaf17bc431e`. Both complete IDL blocks are copied from that revision.

## <a id="interface-abstractrange"></a>AbstractRange

[AbstractRange](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#interface-abstractrange)

```webidl
[Exposed=Window]
interface AbstractRange {
  readonly attribute Node startContainer;
  readonly attribute unsigned long startOffset;
  readonly attribute Node endContainer;
  readonly attribute unsigned long endOffset;
  readonly attribute boolean collapsed;
};
```

## <a id="interface-range"></a>Range

[Range](https://dom.spec.whatwg.org/commit-snapshots/3071e5f269448350fdea79bb284eedaf17bc431e/#interface-range)

```webidl
[Exposed=Window]
interface Range : AbstractRange {
  constructor();

  readonly attribute Node commonAncestorContainer;

  undefined setStart(Node node, unsigned long offset);
  undefined setEnd(Node node, unsigned long offset);
  undefined setStartBefore(Node node);
  undefined setStartAfter(Node node);
  undefined setEndBefore(Node node);
  undefined setEndAfter(Node node);
  undefined collapse(optional boolean toStart = false);
  undefined selectNode(Node node);
  undefined selectNodeContents(Node node);

  const unsigned short START_TO_START = 0;
  const unsigned short START_TO_END = 1;
  const unsigned short END_TO_END = 2;
  const unsigned short END_TO_START = 3;
  short compareBoundaryPoints(unsigned short how, Range sourceRange);

  [CEReactions] undefined deleteContents();
  [CEReactions, NewObject] DocumentFragment extractContents();
  [CEReactions, NewObject] DocumentFragment cloneContents();
  [CEReactions] undefined insertNode(Node node);
  [CEReactions] undefined surroundContents(Node newParent);

  [NewObject] Range cloneRange();
  undefined detach();

  boolean isPointInRange(Node node, unsigned long offset);
  short comparePoint(Node node, unsigned long offset);

  boolean intersectsNode(Node node);

  stringifier;
};
```
