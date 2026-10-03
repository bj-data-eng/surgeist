Attribution and reformatting notice added for Surgeist on 2026-10-03

This reformatted document accompanies Surgeist as software implementation support. The original English document at the source URL remains authoritative; this copy is not a new technical specification. Added labels and representation notes are non-normative. W3C is not responsible for content absent from the original, and this copy may contain formatting or hypertext errors.

Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang). This software or document includes material copied from or derived from [Elaborate description of Stacking Contexts](https://www.w3.org/TR/2011/REC-CSS2-20110607/zindex.html).

Original copyright notice (from the CSS 2.1 edition title page): Copyright © 2011 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C liability, trademark and document use rules apply.

License: [W3C Document License, 2015 version](../licenses/w3c/document-license-2015.txt). Changes are format conversion, visible semantic labels, local exact-snapshot links, and passive SVG/TeX representations as detailed in the [conversion report](CONVERSION-REPORT.md). Original source text, status, authorship, and legal notices remain below.

# Source provenance

Title: Elaborate description of Stacking Contexts

Source snapshot: https://www.w3.org/TR/2011/REC-CSS2-20110607/zindex.html

Snapshot SHA-256: b486aef70d4d44e7f9cbf4376da33b5f31e8d0dcb15d400aa496bf4cb2d04f13

Conversion: offline format conversion of the exact stored HTML; not a new specification or summary. Publication versions remain distinct. Source fragment identifiers are preserved as short HTML anchors. Original copyright and licensing text/links are retained where present in the source.

Representation notes:
- Small semantic emphasis/subscript/superscript HTML is retained to avoid GFM intraword-delimiter and subscript rendering defects; website layout HTML is not retained.

---

<a id="q23.0"></a>

# Appendix E. Elaborate description of Stacking Contexts

(hide)

<strong>Note:</strong> Several sections of this specification have been updated by other specifications. Please, see ["Cascading Style Sheets (CSS) — The Official Definition"](https://www.w3.org/TR/CSS/#css) in the latest CSS Snapshot for a list of specifications and the sections they replace.

The CSS Working Group is also developing [CSS level 2 revision 2 (CSS 2.2).](https://www.w3.org/TR/CSS22/)

This chapter defines the CSS 2.1 painting order in more detail than described in the rest of the specification.

<a id="stacking-defs"></a>

## E.1 Definitions

Tree Order  
Preorder depth-first traversal of the <em>rendering</em> tree, in logical (not visual) order for bidirectional content, after taking into account properties that move boxes around.

Element  
In this description, "element" refers to actual elements, pseudo-elements, and anonymous boxes. Pseudo-elements and anonymous boxes are treated as descendants in the appropriate places. For example, an outside list marker comes before an adjoining ':before' box in the line box, which comes before the content of the box, and so forth.

<a id="painting-order"></a>

## E.2 Painting order

The bottom of the stack is the furthest from the user, the top of the stack is the nearest to the user:

```text

	     |	   |	     |	  |
	     |		|    |	  |	⇦ ☻
	     |		|	  |	user
z-index:  canvas  -1	0    1	  2
```
The stacking context background and most negative positioned stacking contexts are at the bottom of the stack, while the most positive positioned stacking contexts are at the top of the stack.

The canvas is transparent if contained within another, and given a UA-defined color if it is not. It is infinite in extent and contains the root element. Initially, the viewport is anchored with its top left corner at the canvas origin.

The painting order for the descendants of an element generating a stacking context (see the ['z-index'](css2--visuren.html--3f334c530cf4.md#propdef-z-index) property) is:

1.  If the element is a root element:

    1.  background color of element over the entire canvas.
    2.  background image of element, over the entire canvas, anchored at the origin that would be used if it was painted for the root element.

2.  If the element is a block, list-item, or other block equivalent:

    1.  background color of element unless it is the root element.
    2.  background image of element unless it is the root element.
    3.  border of element.

    Otherwise, if the element is a block level table:

    1.  table backgrounds (color then image) unless it is the root element.
    2.  column group backgrounds (color then image).
    3.  column backgrounds (color then image).
    4.  row group backgrounds (color then image).
    5.  row backgrounds (color then image).
    6.  cell backgrounds (color then image).
    7.  all table borders (in tree order for separated borders).

3.  Stacking contexts formed by positioned descendants with negative z-indices (excluding 0) in z-index order (most negative first) then tree order.

4.  For all its in-flow, non-positioned, block-level descendants in tree order: If the element is a block, list-item, or other block equivalent:

    1.  background color of element.
    2.  background image of element.
    3.  border of element.

    Otherwise, the element is a table:

    1.  table backgrounds (color then image).
    2.  column group backgrounds (color then image).
    3.  column backgrounds (color then image).
    4.  row group backgrounds (color then image).
    5.  row backgrounds (color then image).
    6.  cell backgrounds (color then image).
    7.  all table borders (in tree order for separated borders).

5.  All non-positioned floating descendants, in tree order. For each one of these, treat the element as if it created a new stacking context, but any positioned descendants and descendants which actually create a new stacking context should be considered part of the parent stacking context, not this new one.

6.  If the element is an inline element that generates a stacking context, then:

    1.  For each line box that the element is in:

        1.  Jump to [7.2.1](#each-box) for the box(es) of the element in that line box (in tree order).

7.  Otherwise: first for the element, then for all its in-flow, non-positioned, block-level descendants in tree order:

    1.  If the element is a block-level replaced element, then: the replaced content, atomically.

    2.  Otherwise, for each line box of that element:

        1.  <a id="each-box"></a>

            For each box that is a child of that element, in that line box, in tree order:

            1.  background color of element.

            2.  background image of element.

            3.  border of element.

            4.  For inline elements:

                1.  For all the element's in-flow, non-positioned, inline-level children that are in this line box, and all runs of text inside the element that is on this line box, in tree order:

                    1.  If this is a run of text, then:

                        1.  any underlining affecting the text of the element, in tree order of the elements applying the underlining (such that the deepest element's underlining, if any, is painted topmost and the root element's underlining, if any, is drawn bottommost).
                        2.  any overlining affecting the text of the element, in tree order of the elements applying the overlining (such that the deepest element's overlining, if any, is painted topmost and the root element's overlining, if any, is drawn bottommost).
                        3.  the text.
                        4.  any line-through affecting the text of the element, in tree order of the elements applying the line-through (such that the deepest element's line-through, if any, is painted topmost and the root element's line-through, if any, is drawn bottommost).

                    2.  Otherwise, jump to [7.2.1](#each-box) for that element.

                For inline-block and inline-table elements:

                1.  For each one of these, treat the element as if it created a new stacking context, but any positioned descendants and descendants which actually create a new stacking context should be considered part of the parent stacking context, not this new one.

                For inline-level replaced elements:

                1.  the replaced content, atomically.

            > <strong data-conversion-semantic="note">Note</strong>
            >
            > Some of the boxes may have been generated by line splitting or the Unicode bidirectional algorithm.

        2.  Optionally, the outline of the element (see [10 below](#outlines)).

    3.  Optionally, if the element is block-level, the outline of the element (see [10 below](#outlines)).

8.  All positioned descendants with 'z-index: auto' or 'z-index: 0', in tree order. For those with 'z-index: auto', treat the element as if it created a new stacking context, but any positioned descendants and descendants which actually create a new stacking context should be considered part of the parent stacking context, not this new one. For those with 'z-index: 0', treat the stacking context generated atomically.

9.  Stacking contexts formed by positioned descendants with z-indices greater than or equal to 1 in z-index order (smallest first) then tree order.

10. <a id="outlines"></a>

    Finally, implementations that do not draw outlines in steps above must draw outlines from this stacking context at this stage. (It is recommended to draw outlines in this step and not in the steps above.)

<a id="stacking-notes"></a>

## E.3 Notes

The background of the root element is only painted once, over the whole canvas.

While the backgrounds of bidirectional inlines are painted in tree order, they are positioned in visual order. Since the positioning of inline backgrounds is unspecified in CSS 2.1, the exact result of these two requirements is UA-defined. CSS3 may define this in more detail.
