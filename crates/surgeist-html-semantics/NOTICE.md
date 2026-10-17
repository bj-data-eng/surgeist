# surgeist-html-semantics

Surgeist-authored code uses the repository [MIT License](../../LICENSE).

The selected HTML algorithms and source adaptations retain the [WHATWG notice](../../licenses/whatwg/NOTICE.md).

## URL value comparison

The record-owned fragment-excluding value comparison adapts the URL serializer
and equality algorithms from WHATWG URL Standard revision
`fde3f74f063341a28d437216f75735b8b40128f5`. The
[bounded source](../../references/url-equality-serialization--fde3f74f0633.md)
and [WHATWG attribution and full license](../../licenses/whatwg/NOTICE.md#url-equality-and-serialization--2026-10-16)
retain the copyright, conditions and disclaimer. Source-code incorporation uses
the license's BSD 3-Clause provision; the reference prose remains CC BY 4.0.
Surgeist uses the original producer's fragment boundary and bounded borrowed
prefix comparison, without copying a URL parser or normalizing serialization.

## Unicode bidirectional character data

The private first-strong lookup and its offline generator use complete Unicode
18.0.0 DerivedBidiClass data from Unicode, Inc. The source data and generated
table are distributed under the exact
[Unicode License v3](../../licenses/unicode/18.0.0/LICENSE), with
[source attribution](../../licenses/unicode/18.0.0/NOTICE.md). The selected
UAX #9 revision 52 remains an external normative citation; its report text is
not bundled. This lookup supplies HTML directionality rather than a full
bidirectional text-layout implementation.

## Delegated editable definition

The immutable editable predicate adapts the HTML-delegated definition from
[w3c/editing](https://github.com/w3c/editing), revision
`85de9ff4a9816139d06fc2dc39d5dacdc456fddc`. The source's original CC0 and
contributor-agreement notice and its separately declared W3C Software and
Document License are retained in the
[source notice](../../licenses/w3c/NOTICE.md#html-delegated-editability-definitions),
[repository license declaration](../../licenses/w3c/editing/LICENSE.md) and
[complete license text](../../licenses/w3c/software-license-2023.txt).
Surgeist adapts the definition to bounded immutable captures and explicit
document-state facts. It does not incorporate editing-command execution.
HTML itself remains the owner of editing-host classification, under the
separate WHATWG attribution above.

## Dependencies

This crate depends on repository-owned [HTML values](../surgeist-html/NOTICE.md), whose notice retains the third-party dependency license texts and source identities. It has no parser, CSS or Color dependency.
