# Controlled surgeist-fuzz author fixture

This standalone test workspace is repository-authored verification material.
It consumes the shared framework and its
[direct dependency attribution](../../../NOTICE.md). Its optional coverage
binaries additionally depend on libfuzzer-sys 0.4.13; its ordinary worker
binaries do not link that dependency.

* License metadata: ([MIT](licenses/libfuzzer-sys/LICENSE-MIT) OR [Apache-2.0](licenses/libfuzzer-sys/LICENSE-APACHE)) AND NCSA
* Homepage: [libfuzzer-sys](https://github.com/rust-fuzz/libfuzzer)
* Source revision: [`719e4efb9b8857ebaa782ae59376c8cbb78fed0f`](https://github.com/rust-fuzz/libfuzzer/tree/719e4efb9b8857ebaa782ae59376c8cbb78fed0f)
* Bundled LLVM source license: [full LLVM license, including exception and historical NCSA text](licenses/libfuzzer-sys/LLVM-LICENSE.txt)

The crate release's metadata names NCSA, while its bundled libFuzzer source
headers name Apache-2.0 WITH LLVM-exception. Both source statements are retained
here. Its update script pins LLVM to
[`a47b42eb9f9b302167b4fc413e6c92798d65dd0b`](https://github.com/llvm/llvm-project/tree/a47b42eb9f9b302167b4fc413e6c92798d65dd0b).
The included full license comes from that exact revision and preserves both
current and historical sections. The MIT/Apache files are exact crate-release
copies. This fixture vendors no dependency implementation source; a distributed
coverage executable needs an inventory of its actual linked dependencies.
