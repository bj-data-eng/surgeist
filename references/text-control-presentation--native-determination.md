# Visible control-character source determination

The [Text/Layout control contract](../docs/text-layout-contract.md#native-control-character-presentation)
selects the native presentation policy for #64. This reference records decisive
source evidence; it does not certify implementation or browser equivalence.

The pinned [Text 4 §4](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#white-space-processing)
and [Text 3 §4](css-text-3--CRD-css-text-3-20260814--17c26f1b4145.md#white-space-processing)
require visible nonexempt Cc and permit a UA-generated code-point representation.
They prescribe So/Common treatment but do not enumerate the complete bidi,
segmentation, line-breaking and joining property tuple. The contract labels its
chosen ordinary-symbol profile as native policy completing that underspecification.

Source inspection on 24 October 2026 found the
[CSSWG #7249 resolution](https://github.com/w3c/csswg-drafts/issues/7249#issuecomment-1137545872)
retains the existing visible requirement for U+0000. Its discussion rejects
depending solely on whether the selected font happens to paint the control.
[#8261](https://github.com/w3c/csswg-drafts/issues/8261) tracks implementation
shipping coordination; it does not replace the normative behavior or select a
more detailed Unicode-property profile.

Static inspection of WPT
[`control-chars-00B.html`](https://github.com/web-platform-tests/wpt/blob/69f2bba61ce7e9469ccbce53de4644a89f6b7dff/css/css-text/white-space/control-chars-00B.html)
at revision `69f2bba61ce7e9469ccbce53de4644a89f6b7dff` found an assertion that
U+000B is visible and a mismatch reference to an empty result. The test injects
the scalar through CSS to avoid HTML parser alteration. It does not settle the
profile's bidi, grapheme or line-break choices. Git blob identity is
`6cdf0b35cf142cac701d54e7dceb635fdd9be24e`, not a raw SHA-256 capture.
No test code was imported and no browser was executed for this determination.
