# surgeist-bindings-js

This crate depends on the external `v8` 152.2.0 Rust SDK and its separately
built native library. The SDK is MIT-licensed; that description does not replace
V8's BSD license or the licenses of its bundled libraries. Surgeist's own
[MIT license](../../LICENSE) remains separate.

This notice covers that SDK, the selected native macOS ARM64 release build with
default `use_custom_libcxx`, and the source-qualified components below. The
native build graph establishes inclusion; this is not an inspection of every
object in a downstream executable or a complete binary-release inventory.
Other targets, feature choices and final distributions need their own inventory.

## Dependencies

### Rust SDK and native V8

This product depends on [rusty_v8](https://github.com/denoland/rusty_v8),
distributed by the Deno authors, and [V8](https://v8.dev/), distributed by the
V8 project authors:

| Material | Local license and attribution | Exact source |
| --- | --- | --- |
| `v8` SDK 152.2.0 | [MIT](../../licenses/rusty-v8/LICENSE) | [SDK LICENSE](https://github.com/denoland/rusty_v8/blob/v152.2.0/LICENSE), Git blob `5a671244de937cfc449c42d48da7867e4f0a7852` |
| V8-owned implementation | [Original V8 LICENSE](../../licenses/rusty-v8/v8/LICENSE), BSD 3-Clause with its external-library qualifications | [V8 LICENSE at `c4ca1eccb90c5464d826b7713dc27178a53b0cfe`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/LICENSE) |

The SDK's [pinned submodules](https://github.com/denoland/rusty_v8/blob/v152.2.0/.gitmodules),
[native target](https://github.com/denoland/rusty_v8/blob/v152.2.0/BUILD.gn)
and [build defaults](https://github.com/denoland/rusty_v8/blob/v152.2.0/.gn)
select the sources below. Its [release workflow](https://github.com/denoland/rusty_v8/blob/v152.2.0/.github/workflows/ci.yml)
publishes `librusty_v8_release_aarch64-apple-darwin.a.gz`; the release has no
separate notice payload. The library is a complete static archive of the native
target's dependencies. ICU data and Temporal support are enabled. Custom libc++
also includes libc++abi on this target.

### Native libraries

These components retain their own terms. License alternatives and combined
upstream notice files are preserved without selecting a new project license.

| Component and homepage | Local legal material | Pinned upstream source |
| --- | --- | --- |
| [ICU](https://icu.unicode.org/) code and embedded data | [Complete LICENSE](../../licenses/rusty-v8/icu/LICENSE), including Unicode 3.0, older ICU and third-party/data notices | [ICU LICENSE](https://chromium.googlesource.com/chromium/deps/icu/+/d578f2e8b7bd5938e21cfb6bf15c079e0aa5b738/LICENSE) |
| [Abseil](https://abseil.io/) | [Apache 2.0](../../licenses/rusty-v8/abseil/LICENSE), [AUTHORS](../../licenses/rusty-v8/abseil/AUTHORS) | [Abseil at `d16e32215c3ab90ba57c2e904a5344d85c7353e4`](https://chromium.googlesource.com/chromium/src/third_party/abseil-cpp/+/d16e32215c3ab90ba57c2e904a5344d85c7353e4/) |
| [simdutf](https://github.com/simdutf/simdutf) 7.7.0 Chromium single-header copy | [MIT](../../licenses/rusty-v8/simdutf/LICENSE) | [simdutf LICENSE](https://chromium.googlesource.com/chromium/src/third_party/simdutf/+/f7356eed293f8208c40b3c1b344a50bd70971983/LICENSE) |
| [libc++](https://libcxx.llvm.org/) | [Complete LICENSE.TXT](../../licenses/rusty-v8/llvm/libcxx/LICENSE.TXT), [CREDITS.TXT](../../licenses/rusty-v8/llvm/libcxx/CREDITS.TXT) | [libc++ at `b16984ce99c702355a5b2b4c52574e82cec41fb9`](https://chromium.googlesource.com/external/github.com/llvm/llvm-project/libcxx/+/b16984ce99c702355a5b2b4c52574e82cec41fb9/) |
| [libc++abi](https://libcxxabi.llvm.org/) | [Complete LICENSE.TXT](../../licenses/rusty-v8/llvm/libcxxabi/LICENSE.TXT), [CREDITS.TXT](../../licenses/rusty-v8/llvm/libcxxabi/CREDITS.TXT) | [libc++abi at `8f11bb1d4438d0239d0dfc1bd9456a9f31629dda`](https://chromium.googlesource.com/external/github.com/llvm/llvm-project/libcxxabi/+/8f11bb1d4438d0239d0dfc1bd9456a9f31629dda/) |
| [LLVM libc](https://libc.llvm.org/) headers | [Complete LICENSE.TXT](../../licenses/rusty-v8/llvm/libc/LICENSE.TXT) | [libc LICENSE.TXT](https://chromium.googlesource.com/external/github.com/llvm/llvm-project/libc/+/75ae50cd48c8a9d70f552ccff040029968778da9/LICENSE.TXT) |
| [FP16](https://github.com/Maratyszcza/FP16) | [MIT](../../licenses/rusty-v8/fp16/LICENSE) | [FP16 LICENSE](https://github.com/Maratyszcza/FP16/blob/782eea126dc5c755827be751a099eb01826175cf/LICENSE) |
| [fast_float](https://github.com/fastfloat/fast_float) | [Apache 2.0](../../licenses/rusty-v8/fast-float/LICENSE-APACHE), [Boost 1.0](../../licenses/rusty-v8/fast-float/LICENSE-BOOST), [MIT](../../licenses/rusty-v8/fast-float/LICENSE-MIT), [AUTHORS](../../licenses/rusty-v8/fast-float/AUTHORS) | [fast_float at `34164f547b7df3f5d794ff67e9f885c36819ebfc`](https://github.com/fastfloat/fast_float/tree/34164f547b7df3f5d794ff67e9f885c36819ebfc) |
| [Dragonbox](https://github.com/jk-jeon/dragonbox) | [Apache 2.0 with LLVM exceptions](../../licenses/rusty-v8/dragonbox/LICENSE-Apache2-LLVM), [Boost 1.0](../../licenses/rusty-v8/dragonbox/LICENSE-Boost), [original copyright header](../../licenses/rusty-v8/dragonbox/copyright-header.txt) | [Dragonbox at `beeeef91cf6fef89a4d4ba5e95d47ca64ccb3a44`](https://github.com/jk-jeon/dragonbox/tree/beeeef91cf6fef89a4d4ba5e95d47ca64ccb3a44); header copied from `include/dragonbox/dragonbox.h`, lines 1–16 |
| [Highway](https://github.com/google/highway) | [Complete LICENSE](../../licenses/rusty-v8/highway/LICENSE), retaining Apache/BSD alternatives and identified exceptions | [Highway LICENSE](https://github.com/google/highway/blob/2607d3b5b0113992fe84d3848859eae13b3b52c1/LICENSE) |

The full ICU and LLVM files deliberately retain their original scope and
qualifications. Their build-file or legacy sections do not imply that every
listed component is present in the selected runtime. A generic Unicode or
Apache license would lose those original notices. Dragonbox's unchanged header
names its Apache text `LICENSE-Apache`; the local link preserves the upstream
file's actual name, `LICENSE-Apache2-LLVM`.

### Additional notices in V8 runtime sources

The following material is inside the pinned V8 source above. Each copied header
is an unchanged byte excerpt of the identified source, with no implementation
code retained here.

| Material | Local legal material | Exact V8 source path |
| --- | --- | --- |
| Chromium inspector protocol and CRDTP | [BSD 3-Clause](../../licenses/rusty-v8/v8/inspector_protocol/LICENSE) | [`third_party/inspector_protocol/LICENSE`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/third_party/inspector_protocol/LICENSE) |
| rapidhash | [BSD 2-Clause](../../licenses/rusty-v8/v8/rapidhash-v8/LICENSE) | [`third_party/rapidhash-v8/LICENSE`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/third_party/rapidhash-v8/LICENSE) |
| Bjoern Hoehrmann UTF-8 decoder | [MIT](../../licenses/rusty-v8/v8/utf8-decoder/LICENSE) | [`third_party/utf8-decoder/LICENSE`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/third_party/utf8-decoder/LICENSE) |
| SipHash reference implementation | [CC0](../../licenses/rusty-v8/v8/siphash/LICENSE) | [`third_party/siphash/LICENSE`](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/third_party/siphash/LICENSE) |
| Sun Strongtalk assembler origin and V8 modifications | [Original copyright, BSD conditions and modification notice](../../licenses/rusty-v8/v8/strongtalk-license-header.txt) | [`src/codegen/assembler.cc`, lines 1–34](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/codegen/assembler.cc#L1-L34) |
| Sun fdlibm origin and V8 modifications | [Original copyright, permission grant and modification notice](../../licenses/rusty-v8/v8/fdlibm-license-header.txt) | [`src/base/ieee754.cc`, lines 1–14](https://github.com/denoland/v8/blob/c4ca1eccb90c5464d826b7713dc27178a53b0cfe/src/base/ieee754.cc#L1-L14) |

### Native Temporal and Rust libraries

The native library's Temporal implementation is **0.2.4**, selected through
[`third_party/rust` at `afbc96607d0e659715d803cc099607dd1737fc41`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/).
Its `temporal_capi/v0_2/BUILD.gn` runtime dependency graph contains the 31 Rust
libraries grouped below. The pinned `<crate>/<epoch>/README.chromium` records
each version and its selected legal file under `chromium_crates_io/vendor/`.
The source links below point to those original legal files. Shared local files
were compared byte for byte across every component they cover.

| Native component versions | Local legal material | Pinned original legal file |
| --- | --- | --- |
| [`calendrical_calculations`](https://github.com/unicode-org/icu4x) 0.2.4 | [Apache 2.0](../../licenses/rusty-v8/native-rust/calendrical-calculations/LICENSE) | [`calendrical_calculations-v0_2/LICENSE`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/calendrical_calculations-v0_2/LICENSE) |
| [`core_maths`](https://github.com/robertbastian/core_maths) 0.1.1 | [MIT](../../licenses/rusty-v8/native-rust/core-maths/LICENSE) | [`core_maths-v0_1/LICENSE`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/core_maths-v0_1/LICENSE) |
| [`diplomat-runtime`](https://github.com/rust-diplomat/diplomat) 0.15.1 | [Apache 2.0](../../licenses/rusty-v8/native-rust/diplomat-runtime/LICENSE-APACHE) | [`diplomat-runtime-v0_15/LICENSE-APACHE`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/diplomat-runtime-v0_15/LICENSE-APACHE) |
| [`either`](https://github.com/rayon-rs/either) 1.16.0, [`num-traits`](https://github.com/rust-num/num-traits) 0.2.19, [`stable_deref_trait`](https://github.com/storyyeller/stable_deref_trait) 1.2.1 | [Identical Apache 2.0 text](../../licenses/rusty-v8/native-rust/either/LICENSE-APACHE) | [`either-v1/LICENSE-APACHE`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/either-v1/LICENSE-APACHE); also `num-traits-v0_2/LICENSE-APACHE` and `stable_deref_trait-v1/LICENSE-APACHE` |
| [ICU4X](https://github.com/unicode-org/icu4x) and associated libraries listed below | [Identical Unicode 3.0 text](../../licenses/rusty-v8/native-rust/icu4x/LICENSE) | [`icu_calendar-v2/LICENSE`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/icu_calendar-v2/LICENSE); each covered crate retains `LICENSE` |
| [`libm`](https://github.com/rust-lang/compiler-builtins) 0.2.16 | [Complete original LICENSE.txt](../../licenses/rusty-v8/native-rust/libm/LICENSE.txt), including its additional notices | [`libm-v0_2/LICENSE.txt`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/libm-v0_2/LICENSE.txt) |
| [`serde` and `serde_core`](https://github.com/serde-rs/serde) 1.0.228 | [Identical Apache 2.0 text](../../licenses/rusty-v8/native-rust/serde/LICENSE-APACHE) | [`serde-v1/LICENSE-APACHE`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/serde-v1/LICENSE-APACHE); also `serde_core-v1/LICENSE-APACHE` |
| [Temporal](https://github.com/boa-dev/temporal): `temporal_capi`, `temporal_rs`, `timezone_provider` 0.2.4 | [Identical Apache 2.0 text](../../licenses/rusty-v8/temporal/LICENSE-Apache), the Chromium inventory's selected license | [`temporal_capi-v0_2/LICENSE-Apache`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/temporal_capi-v0_2/LICENSE-Apache); also `temporal_rs-v0_2/LICENSE-Apache` and `timezone_provider-v0_2/LICENSE-Apache` |
| [`utf8_iter`](https://github.com/hsivonen/utf8_iter) 1.0.4 | [Apache 2.0](../../licenses/rusty-v8/native-rust/utf8-iter/LICENSE-APACHE) | [`utf8_iter-v1/LICENSE-APACHE`](https://chromium.googlesource.com/chromium/src/third_party/rust/+/afbc96607d0e659715d803cc099607dd1737fc41/chromium_crates_io/vendor/utf8_iter-v1/LICENSE-APACHE) |

The shared Unicode file covers `icu_calendar` 2.2.1; `icu_calendar_data`,
`icu_collections`, `icu_locale`, `icu_locale_core`, `icu_locale_data` and
`icu_provider` 2.2.0; `ixdtf` 0.6.5; `litemap` 0.8.2; `potential_utf` 0.1.5;
`resb` 0.1.2; `tinystr` 0.8.3; `writeable` 0.6.3; `yoke` 0.8.3; `zerofrom`
0.1.8; `zerotrie` 0.2.4; `zerovec` 0.11.6; and `zoneinfo64` 0.3.0. These are
native-build versions, not an assertion about Cargo's separately resolved graph.
Their upstream homepages are [ICU4X](https://github.com/unicode-org/icu4x) and
[Temporal](https://github.com/boa-dev/temporal); the pinned Chromium metadata
identifies each individual package.

Separately, the consuming SDK's Cargo dependencies resolve `temporal_capi`,
`temporal_rs` and `timezone_provider` **0.2.6** in the root lockfile. Those source
packages identify [Temporal revision `6a5d18375acd62ba778726df68cdc9dd3d09f285`](https://github.com/boa-dev/temporal/tree/6a5d18375acd62ba778726df68cdc9dd3d09f285).
Their original [Apache 2.0](../../licenses/rusty-v8/temporal/LICENSE-Apache)
and [MIT alternative](../../licenses/rusty-v8/temporal/LICENSE-MIT) are retained;
all three 0.2.6 copies were compared, and their Apache text is also identical to
the native 0.2.4 copies. Resolving 0.2.6 in Cargo does not replace the Temporal
objects inside the prebuilt native archive.

## Coverage

Legal texts and the identified copyright headers are retained verbatim. No
upstream implementation is vendored by this attribution directory. It does not
add upstream test suites, toolchain binaries or source-build dependencies to the
runtime inventory. In particular, the selected defaults do not establish bundled
zlib, custom libunwind, PartitionAlloc or the separate Wasm C API target merely
because those sources occur in the SDK checkout.

This notice is a scoped dependency and native-source attribution record. It does
not enumerate every transitive Cargo dependency, build tool, compiler runtime or
object retained by a final linker. No downstream release binary is distributed
by this source change; a binary distribution must carry its applicable complete
notices and verify its actual target, features and bundled material.

## Web IDL algorithm basis

The primitive conversion implementation adapts the pinned Web IDL algorithms,
Copyright © WHATWG (Apple, Google, Mozilla, Microsoft), under the existing
[complete license](../../licenses/whatwg/LICENSE) and
[adaptation notice](../../licenses/whatwg/NOTICE.md#javascript-binding-algorithms).
The [binding source index](../../references/js-bindings-standards.md) records
the complete applicable source clauses. These standards terms are separate from
the external engine dependency above.
