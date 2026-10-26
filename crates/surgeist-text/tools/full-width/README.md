# Raw full-width mapping data

Text owns the raw Unicode mapping required by
[CSS Text4 §2.1.1](../../../../references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#text-transform-mapping).
The [checked-in input](../../data/UnicodeData-width-18.0.0.txt) retains every
`<wide>` and `<narrow>` record from UnicodeData18.0.0, byte-for-byte and in
source order. For `<wide>`, reverse the decomposition; for `<narrow>`, use it
forward. This is neither NFKC/NFKD nor an EastAsianWidth property lookup.
Width execution, preserved-SPACE policy and casing/kana order belong to the
[transformation contract](../../../../docs/text-layout-contract.md#text-transformations-and-correspondence).

## Regenerate and check

From the repository root, using installed Python3 and no network/dependencies:

```sh
pisct run --timeout-seconds 30 -- python3 crates/surgeist-text/tools/full-width/generate.py
pisct run --timeout-seconds 30 -- python3 crates/surgeist-text/tools/full-width/generate.py --check
```

The stdlib-only generator owns [width_data.rs](../../src/transform/width_data.rs).
`--check` compares exact bytes and never rewrites either file. Paths resolve
from the script, so the working directory does not affect the result.
It validates raw-record and pair hashes, tag direction, singleton BMP scalar
values, source order, input/destination uniqueness and 104wide+122narrow counts.
The 226 pairs are sorted by input and exposed only as private scalar constants.
Do not hand-edit generated output or fetch data during a build.

## Edition and provenance

- Original: [UnicodeData.txt, Unicode18.0.0](https://www.unicode.org/Public/18.0.0/ucd/UnicodeData.txt).
- Complete file: 2,243,593 bytes; SHA256
  `0736451de439ae7baf1425136617da495e09ee5afbe6e394374db7009ea08950`.
- Untouched record payload (excluding excerpt comments): 226 lines,16,216bytes;
  SHA256 `d202b05d03a4247fbe365bb5159a643639876415fcbab4c2765773f5f1f8ff40`.
- Directed pairs, sorted `INPUT;OUTPUT\n` uppercase four-digit hexadecimal:
  SHA256 `523324ae4afd9cde0d257db25c803b85048adf4f699fe6ae4cda96e47b127a4d`.

The excerpt was taken from the already-retained complete source, not provider
normalization output. This is a bounded width-data edition; it does not relabel
ICU2.2's casing/analysis data as Unicode18 or claim exact17 qualification.

Copyright © 1991-2026 Unicode, Inc. Distributed under the
[Unicode License V3](../../../../licenses/unicode/18.0.0/LICENSE)
([Unicode data homepage](https://www.unicode.org/ucd/)). The retained source
license is byte-identical to that linked file: SHA256
`e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96`.
It is distinct from Text's existing ICU4X notice with 2020–2024 copyright years.
No separately copyrighted Unicode report excerpt is incorporated.
