# Font resource controls

These are data inputs, not fonts installed on the host. Tests package unchanged
SFNT tables in independently authored WOFF1/WOFF2 wire fixtures in memory; no
provider encoder, download or native program runs during tests. Original bytes
and their independent table contents remain the expectations.

## Original Ahem 1.1

`ahem-original/Ahem.woff2` is extracted without modification from the embedded
data URL in Layout's [base CSS](../../../surgeist-layout/tests/layout/browser_parity/scripts/gentest/test_base_style.css).
It is the 1,496-byte Ahem 1.1 resource retained from the existing browser corpus,
not the distinct Ahem 1.50 TTF. License and authorship are preserved by the
repository's [Ahem notice](../../../../NOTICE.md#ahem),
[public-domain/CC0 declaration](../../../../licenses/ahem/COPYING), and
[upstream README](../../../../licenses/ahem/UPSTREAM-README).

SHA-256: `e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c`.
Independent retained source facts are units-per-em 1000, hhea ascender 800,
descender -200 and line gap zero. They do not establish a selected text baseline.
The later TTF is referenced from Render's existing fixture, without duplication.

## Source Code Pro CFF

`source-code-pro/SourceCodePro-Regular.otf` is unchanged from
[Adobe Source Code Pro](https://github.com/adobe-fonts/source-code-pro/blob/d3f1a5962cde503f9409c21e58527611d4a19ef1/OTF/SourceCodePro-Regular.otf),
release `2.042R-u/1.062R-i/1.026R-vf`.
The original [SIL Open Font License 1.1 and copyright](source-code-pro/LICENSE.md)
are retained unchanged, including their CRLF line endings. The local Git
attribute records that byte-preservation requirement. This is a CFF/OpenType
and shaping-table control.

- Font: 131,128 bytes; SHA-256 `9f9664e2edf6f045c11e774f9bd0be6993971f2544a39061a5ce478b96b051f8`.
- License: SHA-256 `7c940e28a5388e9bba866cf0e408edda45fe0899ba98665b8f6ab31dc5e4b8ff`.

## Variable COLRv1 control

`color-fonts/test_glyphs-glyf_colr_1_variable.ttf` is unchanged from
[Google Fonts' COLRv1 variable test font](https://github.com/googlefonts/color-fonts/blob/0046ea4c3b69e9fbbe464c2594816894e3aa5e4b/fonts/test_glyphs-glyf_colr_1_variable.ttf).
The original [Apache License 2.0](color-fonts/LICENSE) is retained unchanged.
The upstream [description](https://github.com/googlefonts/color-fonts/blob/0046ea4c3b69e9fbbe464c2594816894e3aa5e4b/README.md#colrv1-test-font)
identifies its static/variable color-test purpose. Tests preserve every actual
table, including `COLR`, `CPAL`, `fvar` and `gvar`; they do not claim a consumer
can render every color operation or variation.

- Font: 53,096 bytes; SHA-256 `ad575a09d6748aebcb3b90ffd384c64d5c64b5fd9927967e7cd7cc0d70c98d34`.
- License: SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.
