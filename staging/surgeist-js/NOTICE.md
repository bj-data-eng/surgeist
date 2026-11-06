# JavaScript staging attribution

## Boa

The ECMAScript engine, its supporting crates, and the selected test and
benchmark workloads are adapted from [Boa v0.22](https://github.com/boa-dev/boa/releases/tag/v0.22).
The source is dual licensed under the [MIT License](LICENSE-MIT) or
[Unlicense](LICENSE-UNLICENSE).

## ICU4X data

The optional Intl provider embeds the 12 unmodified Boa v0.22 postcard
files in [data/](crates/surgeist-js-icu-provider/data/). Boa's
[ICU4X 2.3 data update](https://github.com/boa-dev/boa/pull/5490) introduced the
retained files. Its pinned generator uses ICU4X SourceDataProvider defaults,
with Modern CLDR locale coverage plus en-US and Manx and the recommended
segmenter models. The upstream commit identifies the exact retained bytes.
The default input versions come from the pinned generator's source. The
published blobs have not been independently regenerated.

* [ICU4X Unicode License 3.0](licenses/icu-data/ICU4X-LICENSE)
* [CLDR 48.2.1 license](licenses/icu-data/CLDR-LICENSE)
* [ICU export release-78.1rc notices](licenses/icu-data/ICU-LICENSE)
* [LSTM Word Segmentation v0.1.0 license](licenses/icu-data/LSTM-LICENSE)
* [TZDB 2026c license](licenses/icu-data/TZDB-LICENSE)

## V8 benchmarks

The [V8 benchmark subset](tools/surgeist-js-benchmarks/scripts/v8-benches/)
retains copyright and redistribution conditions in each script. Boa sourced
these workloads from [V8 revision 52ab610bd13](https://chromium.googlesource.com/v8/v8/+/52ab610bd13/benchmarks/).
The scripts are test material.

## Test262

The external [Test262 suite](https://github.com/tc39/test262/tree/d86b2294eb0a17eaa281ff12c73c473ec864c72f)
is run from its pinned checkout under its [own license](https://github.com/tc39/test262/blob/d86b2294eb0a17eaa281ff12c73c473ec864c72f/LICENSE).
The staging baseline records run outcomes and case identifiers.
