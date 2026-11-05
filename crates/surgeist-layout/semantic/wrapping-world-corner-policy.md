# Collapsed Flex world-corner height

Policy `flexbox-collapse-two-round-struts`, edition
`css-flexbox-1-20251014-native-2`, covers the four variants of the
[original wrapping fixture](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/tests/layout/browser_parity/html/flex/fri07_collapsed_strut_wrapping.html).
The [strut derivation](https://github.com/bj-data-eng/surgeist/blob/main/crates/surgeist-layout/semantic/collapsed-flex-strut-policy.md)
defines its complete Layout packets and root-local comparisons.

The authenticated current browser capture records the root's world-corner
height as the difference between its separately rounded bottom and top
coordinates. This height is bound to the original V2 root-height observation.
The root-local height remains a separate comparison. Both browser values are
157px for these four captures.

The unrounded Layout root height is 68px from cross struts of 35px and 25px
and one 8px row gap. The final Layout packet also has height 68px. In this
fixture's CSS coordinate range, an integral length `h` gives
`round(top + h) - round(top) = h` for every rounding phase. The comparator
requires that integral length and agreement between the two Layout packets.
The two height comparisons each retain the exact 157px and 68px pair.
