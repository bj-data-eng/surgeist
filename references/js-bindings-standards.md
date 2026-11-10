# JavaScript binding sources

The binding owner consumes the complete clauses linked below. These links pin
the source algorithms; the older bounded string capture does not stand in for
the JavaScript binding chapter. This index contains source locators and local
implementation decisions, not a transcription or replacement specification.

## Web IDL

Use Web IDL revision `0620cf171d1b97138847088748e363cae4005156`, matching the
existing [native string source](dom-webidl-string-types--editor-capture-20261010--53a256058148.md).
The complete original [`index.bs`](https://github.com/whatwg/webidl/blob/0620cf171d1b97138847088748e363cae4005156/index.bs)
has Git blob `dc7fd23d7c85adb570387a6ab513b59da96cb80b`.
The [complete JavaScript binding chapter](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#javascript-binding)
is authoritative, including its delegated definitions and abrupt completions.

| Concern | Complete applicable clauses |
| --- | --- |
| Primitive conversions | [Types and conversions](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-type-mapping), including all integer types and their abstract operations, floating types, strings, nullable values and interface types |
| Structured inputs | [Dictionaries](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-dictionary), [sequences](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-sequence), [unions](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-union), [enumerations](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-enumeration) and [overload resolution](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-overloads) |
| Installed interface objects | [Interfaces](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-interfaces), [platform objects](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-platform-objects), [extended attributes](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-extended-attributes) and [legacy extended attributes](https://webidl.spec.whatwg.org/commit-snapshots/0620cf171d1b97138847088748e363cae4005156/#js-legacy-extended-attributes) |

`ConvertToInt` operates on the exact mathematical value represented by the
JavaScript Number. Ordinary 64-bit conversion uses the full modulo `2^64`.
For `[EnforceRange]` and `[Clamp]`, this edition explicitly bounds 64-bit types
at `2^53 - 1` (and its negative for signed types). Range enforcement truncates
before testing bounds and throws **TypeError**. Clamping rounds ties to even.
The implementation decomposes binary64 to compute modulo exactly; a floating
`rem_euclid(2^64)` would round values such as `-2` incorrectly near the modulus.

Boolean conversion delegates noncoercive ToBoolean. String conversion performs
ToString once: ordinary DOMString preserves every code unit, USVString replaces
unpaired surrogates, and ByteString rejects a code unit above 255. The legacy
null-to-empty extended attribute must be applied explicitly by interface metadata;
it is not the ordinary string conversion policy.

Restricted float rejects nonfinite input and finite input that rounds outside
the finite single-precision range. Rounding follows nearest with ties to even,
including the overflow midpoint and signed underflow. Restricted double rejects
nonfinite input. Unrestricted floating conversions retain infinities and use
the exact canonical NaN bit patterns required by the selected Web IDL edition.

## Delegated ECMAScript operations

Use the existing [dated ECMAScript 2026 source selection](ecmascript-number-serialization--2026--cd65f2eaa201.md#source-identity-and-bounds),
whose publisher front matter retains its original draft status. The complete
[abstract-operations page](https://tc39.es/ecma262/2026/multipage/abstract-operations.html)
supplies ToPrimitive, ToBoolean, ToNumber, ToString and related property/call
operations; the complete [data-types page](https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html)
and [algorithm conventions](https://tc39.es/ecma262/2026/multipage/notational-conventions.html#sec-algorithm-conventions)
supply their numeric and mathematical definitions. Delegation to the external
engine preserves guest coercion and abrupt completion rather than reimplementing
JavaScript evaluation in the binding layer.

## Native interface definitions

The [DOM foundation capture](dom-foundation--editor-capture-20261010--08dff0a4827f.md)
retains the core interface definitions and mixins from the selected DOM edition.
The [source-edition index](SOURCE-EDITIONS.md) identifies the owning HTML and CSSOM
captures and their deliberate scope. A binding description must use the exact
interfaces, inheritance, member types, exposure and extended attributes from
those owners; an absent native execution contract is a producer prerequisite.
The JavaScript adapter must not implement a second native tree or silently
replace an unavailable native algorithm with a collection of replayed edits.

Web IDL adaptation and its license scope are recorded in the
[WHATWG notice](../licenses/whatwg/NOTICE.md#javascript-binding-algorithms).
The external-engine dependency retains its separate
[notice](../crates/surgeist-bindings-js/NOTICE.md).
