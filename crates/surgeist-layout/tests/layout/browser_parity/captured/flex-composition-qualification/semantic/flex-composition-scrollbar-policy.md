# Sideways Flex classic scrollbar edge qualification

Layout policy `classic-scrollbar-logical-end`, edition
`css-overflow-3-20251007-native-1`, applies to the visible scroller
in [the sideways composition fixture](../tests/layout/browser_parity/html/flex/fri07_flex_composition.html).
[CSS Overflow 3 §4.2](../../../references/css-overflow-3--WD-css-overflow-3-20251007--1b59ae503fd8.md#scrollbar-gutter-property)
leaves the scrollbar edge to the user agent. The Native choice is owned
by [#207](https://github.com/bj-data-eng/surgeist/issues/207) and
implemented in [scroll geometry](../src/scroll/box_geometry.rs).

Native places the block-axis gutter at logical inline-end. The 15 px
gutter in this capture lies at physical bottom in LTR and physical top
in RTL. The browser places it at physical bottom in both directions.
The visible scroller retains its 70×60 border box, 70×45 client size and
50 px leftward scroll span. Its child starts at parent-relative y=0 in
both LTR results and at y=0 in the browser versus y=15 in Native RTL.

The exact comparison qualifies that one y pair in each RTL variant.
The [collapse-disabled control](../tests/layout/browser_parity/captured/flex-composition-differences/gutter-control.json)
preserves the scroller's size, content,
writing mode and scrollbar conditions. It verifies the same local
offsets with collapse enabled and disabled.
