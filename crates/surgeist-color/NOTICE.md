# surgeist-color

This crate is authored in the Surgeist repository under the project's
[MIT License](../../LICENSE). Its numerical and formatting implementation was
relocated from `surgeist-css`; this change adds no third-party Cargo dependency
or bundled upstream implementation.

## Sources

Numerical conversion and resolved serialization implement the selected CSS
Color requirements. The `named` table transcribes the opaque named colors of
Color 4 section 6.1 (CRD 2026-09-08) and is verified against the retained
specification text. Source comments retain the original edition, coefficient
derivations and numerical-policy qualifications. These references are
implementation evidence, not a claim that this crate incorporates a browser
implementation or implements every clause in those standards.

* Selected editions and product policies: [source-edition reference](../../references/SOURCE-EDITIONS.md#html-color-delegation-and-intrinsic-contrast-policy)
* W3C references and original copyright notices: [W3C attribution](../../licenses/w3c/NOTICE.md#css-color-module-level-4)
* Applicable reference license: [W3C Software and Document License, 2023](../../licenses/w3c/software-license-2023.txt)
* Homepage: [W3C technical reports](https://www.w3.org/TR/)

The repository's [root notice](../../NOTICE.md) covers the accompanying reference
collection and its legal material. Preserve that material with redistributed
reference documents. The crate's dependency-free manifest does not change the
licenses of those documents.
