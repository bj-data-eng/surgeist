# Font container sources

Format behavior follows the [WOFF 1.0 Recommendation, 13 December 2012](https://www.w3.org/TR/2012/REC-WOFF-20121213/)
and [WOFF 2.0 Recommendation, 8 August 2024](https://www.w3.org/TR/2024/REC-WOFF2-20240808/).
Container admission does not establish font-table semantics or a consumer's
shaping or rendering capabilities.

The private WOFF2 reconstruction adapts the Wuff 0.2.9 library at upstream commit
`78e67f52713c0a2c36bd083fea18ba029565c9ec`:
[source tree](https://github.com/nicoburns/wuff/tree/78e67f52713c0a2c36bd083fea18ba029565c9ec/wuff/src).
The package archive SHA-256 is
`200a7b806377b65f1ff9a985874332f15357c87e5dd7855f35f06dade95b8b3d`.
The original MIT copyright and permission notice is retained in
[licenses/wuff/LICENSE](licenses/wuff/LICENSE).

The adapted boundary replaces provider allocation growth and generic failures
with caller limits and typed refusals, and reconstructs shared collection tables
using each face's actual outline and metrics context. WOFF1 is implemented from
the Recommendation using the already-owned miniz_oxide 0.8.9 core decoder.

Brotli remains the exact `brotli-decompressor` 5.0.0 dependency with default
features disabled, through a private strict decoder and a complete precharged
pool. Its only required allocation-interface package is `alloc-no-stdlib` 2.0.4.
These interfaces do not select a global allocator or exclude the standard
library. No native executable, browser, JavaScript runtime or system-installed
font is required by resource decoding.

Archive SHA-256:

| Package | SHA-256 |
| --- | --- |
| brotli-decompressor 5.0.0 | `874bb8112abecc98cbd6d81ea4fa7e94fb9449648c93cc89aa40c81c24d7de03` |
| alloc-no-stdlib 2.0.4 | `cc7bb162ec39d46ab1ca8c77bf72e890535becd1751bb45f64c597edb4c8c6b3` |

See the [notice](NOTICE.md) for accompanying dependency license material.
