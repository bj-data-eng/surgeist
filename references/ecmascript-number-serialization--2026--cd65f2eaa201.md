# ECMAScript 2026 — bounded Number serialization

This document includes material copied from or derived from ECMAScript® 2026
Language Specification, https://tc39.es/ecma262/2026/.
Copyright © 2026 Ecma International.

This modified representation extracts the complete ToString operation and
Number::toString operation with all three notes, together with the Number
representation/rounding definitions and selected mathematical definitions
needed to interpret them. It converts the original HTML into Markdown,
preserving mathematical variables/scripts, list nesting and source anchors;
links resolve against their original dated pages. Original source text begins
under the edition front matter below. The full
[copyright notice, copyright license, disclaimers and software terms](../licenses/ecma262/LICENSE-2026.txt)
and [attribution/modification notice](../licenses/ecma262/NOTICE.md) accompany
this conversion. Ecma retains copyright; Surgeist's license is unchanged.

## Source identity and bounds

| Complete original page | Bytes | Original SHA-256 | Retained scope |
| --- | --- | --- | --- |
| [Abstract operations](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-tostring) | 845895 | cd65f2eaa201aee5c37c2c6045ca9d7097644599cc14d3f4805fe2c55205b5d4 | Complete §7.1.17, including its Number branch |
| [Data types and values](https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring) | 894158 | 1d18bdb598f7b6c3c0093926dc98a42d79fb667ebdaaf030258371fccc27d3f3 | §6.1.6.1 introductory definitions through the Number value for infinities; complete §6.1.6.1.20 and Notes 1–3 |
| [Notation](https://tc39.es/ecma262/2026/multipage/notational-conventions.html#sec-mathematical-operations) | 677415 | 8b1a5e473eb59d285a2873ad428ce745013fb85384a80ed924644f81083c6d09 | §5.2.6 numeric kinds, type/operator/conversion definitions, abs and complete interval definitions/example; unrelated logarithm, min/max, modulo, clamp, floor and truncate definitions omitted |

The original [front matter](https://tc39.es/ecma262/2026/multipage/) is
582025 bytes, SHA-256 f148c5bc76ea743218abaff5e7d3283fd44b53771b09eb0c213d185b52ec5e2d.
Its title/date and complete About/Contributing block are retained with the
introduction's edition paragraph. The publisher's literal heading is
“Draft ECMA-262 / March 31, 2026”; the introduction calls this the seventeenth
edition. The dated 2026 URLs identify the captured source, rather than the
rolling edition; this conversion does not upgrade that draft header into a
different publication claim. The complete original legal page is 568815 bytes,
SHA-256 cf7102c0e9d8f78a7d3549e989574fe96c7ecce55ff964cb665e44cad9dab854.
These are original-page hashes, not hashes of excerpts or this Markdown.

## Selection boundary

The [owning source determination](SOURCE-EDITIONS.md#html-best-number-serialization)
selects only ToString's Number branch and Number::toString at radix 10 for
HTML877d's best-number representation. Other branches/radices remain in the
complete source excerpt for fidelity but are not selected behavior.

Note 2 below is an informative recommendation in ECMA. The owning #1134
determination deliberately makes its shortest/nearest/even decimal-significand
refinement mandatory for Surgeist. Number-value conversion's binary
roundTiesToEven, Range's separately selected nearest admissible lattice
candidate with larger-number ties, and serialization's decimal significand tie
are distinct domains. This reference does not make ECMA normative for Range.

The HTML finite-number boundary rejects nonfinite input; retained NaN/Infinity
source branches do not become valid HTML numeric output. Number preserves valid
authored strings; Range requests serialization only on source-required
correction/default branches. Source retention is not an implementation,
roundtrip proof, Range acceptance, JavaScript runtime or full ECMA selection.
Note 3's paper/code links are original citations; no linked code was acquired.

## Original selected source

<a id="ecma-edition-front-matter"></a>

## Draft ECMA-262 / March 31, 2026

## ECMAScript® 2026 Language Specification

<a id="metadata-block"></a>

## About this Specification

The document at <a href="https://tc39.es/ecma262/">https&#58;//tc39.es/ecma262/</a> is the most accurate and up-to-date ECMAScript specification. It contains the content of the most recent yearly snapshot plus any <a href="https://github.com/tc39/proposals/blob/HEAD/finished-proposals.md">finished proposals</a> — those that have reached Stage 4 in the <a href="https://tc39.es/process-document/">proposal process</a> and thus are implemented in several implementations and will be included in the next yearly snapshot. Historical snapshots are available at <a href="https://ecma-international.org/publications-and-standards/standards/ecma-262/">https&#58;//ecma-international.org/publications-and-standards/standards/ecma-262/</a>.

This document is available as <a href="https://tc39.es/ecma262/2026/">a single page</a> and as <a href="https://tc39.es/ecma262/2026/multipage/index.html">multiple pages</a>.

## Contributing to this Specification

This specification is developed on GitHub with the help of the ECMAScript community. There are a number of ways to contribute to the development of this specification:

- GitHub Repository: <a href="https://github.com/tc39/ecma262">https&#58;//github.com/tc39/ecma262</a>
- Issues: <a href="https://github.com/tc39/ecma262/issues">All Issues</a>, <a href="https://github.com/tc39/ecma262/issues/new">File a New Issue</a>
- Pull Requests: <a href="https://github.com/tc39/ecma262/pulls">All Pull Requests</a>, <a href="https://github.com/tc39/ecma262/pulls/new">Create a New Pull Request</a>
- Test Suite: <a href="https://github.com/tc39/test262">Test262</a>
- Editors:
  - <a href="mailto:shu%20at%20rfrn%20dot%20org">Shu-yu Guo</a>
  - <a href="mailto:ecma262-editor-list%20at%20michael%20dot%20ficarra%20dot%20me">Michael Ficarra</a> (<a href="https://bsky.app/profile/michael.ficarra.me">&#64;michael.ficarra.me</a>)
  - <a href="mailto:bakkot%20at%20gmail%20dot%20com">Kevin Gibbons</a>
- Community:
  - Discourse: <a href="https://es.discourse.group">https&#58;//es.discourse.group/</a>
  - Chat: <a href="https://github.com/tc39/how-we-work/blob/HEAD/matrix-guide.md">Matrix</a>
  - Mailing List Archives: <a href="https://esdiscuss.org">https&#58;//esdiscuss.org/</a>

Refer to the <a id="&#95;ref&#95;0"></a><a href="https://tc39.es/ecma262/2026/multipage/colophon.html#sec-colophon">colophon</a> for more information on how this document is created.

This Ecma Standard defines the ECMAScript 2026 Language. It is the seventeenth edition of the ECMAScript Language Specification. ECMAScript is based on several originating technologies, the most well-known being JavaScript (Netscape) and JScript (Microsoft). The language was invented by Brendan Eich at Netscape and first appeared in that company's Navigator 2.0 browser. Though best known as the language embedded in web browsers, it has also been widely adopted for use outside the browser, including server and embedded applications, and has grown to be one of the world's most widely used general-purpose programming languages.

<a id="sec-mathematical-operations"></a>

## 5.2.6 Mathematical Operations

This specification makes reference to these kinds of numeric values:

- <a id="mathematical-value"></a>Mathematical values: Arbitrary real numbers, used as the default numeric type.
- <a id="extended-mathematical-value"></a>Extended mathematical values: <a id="&#95;ref&#95;1178"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">Mathematical values</a> together with +∞ and -∞.
- <em>Numbers</em>: <a id="&#95;ref&#95;1179"></a><a href="https://tc39.es/ecma262/2026/multipage/bibliography.html#sec-bibliography">IEEE 754-2019</a> binary64 (double-precision floating point) values.
- <em>BigInts</em>: <a id="&#95;ref&#95;1180"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types">ECMAScript language values</a> representing arbitrary <a id="&#95;ref&#95;1181"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integers</a> in a one-to-one correspondence.

In the language of this specification, numerical values are distinguished among different numeric kinds using subscript suffixes. The subscript <sub>𝔽</sub> refers to Numbers, and the subscript <sub>ℤ</sub> refers to BigInts. Numeric values without a subscript suffix refer to <a id="&#95;ref&#95;1182"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical values</a>. This specification denotes most numeric values in base 10; it also uses numeric values of the form 0x followed by digits 0-9 or A-F as base-16 values.

In general, when this specification refers to a numerical value, such as in the phrase, "the length of <var>y</var>" or "the <a id="&#95;ref&#95;1183"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> represented by the four hexadecimal digits ...", without explicitly specifying a numeric kind, the phrase refers to a <a id="&#95;ref&#95;1184"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical value</a>. Phrases which refer to a Number or a BigInt value are explicitly annotated as such; for example, "the <a id="&#95;ref&#95;1185"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#number-value-for">Number value for</a> the number of code points in …" or "the <a id="&#95;ref&#95;1186"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#bigint-value-for">BigInt value for</a> …".

When the term <a id="integer"></a><a id="mathematical integer"></a>integer is used in this specification, it refers to a <a id="&#95;ref&#95;1187"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical value</a> which is in the set of <a id="&#95;ref&#95;1188"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integers</a>, unless otherwise stated. When the term <a id="integral-number"></a><a id="sec-isinteger"></a><a id="sec-isintegralnumber"></a>integral Number is used in this specification, it refers to a <a id="&#95;ref&#95;1189"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#finite">finite</a> Number value whose <a id="&#95;ref&#95;1190"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical value</a> is in the set of <a id="&#95;ref&#95;1191"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integers</a>.

Numeric operators such as +, ×, =, and ≥ refer to those operations as determined by the type of the operands. When applied to <a id="&#95;ref&#95;1192"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical values</a>, the operators refer to the usual mathematical operations. When applied to <a id="&#95;ref&#95;1193"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#extended-mathematical-value">extended mathematical values</a>, the operators refer to the usual mathematical operations over the extended real numbers; indeterminate forms are not defined and their use in this specification should be considered an editorial error. When applied to Numbers, the operators refer to the relevant operations within <a id="&#95;ref&#95;1194"></a><a href="https://tc39.es/ecma262/2026/multipage/bibliography.html#sec-bibliography">IEEE 754-2019</a>. When applied to BigInts, the operators refer to the usual mathematical operations applied to the <a id="&#95;ref&#95;1195"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value-of">mathematical value of</a> the BigInt. Numeric operators applied to mixed-type operands (such as a Number and a <a id="&#95;ref&#95;1196"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical value</a>) are not defined and should be considered an editorial error in this specification.

Conversions between <a id="&#95;ref&#95;1197"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical values</a> and Numbers or BigInts are always explicit in this document. A conversion from a <a id="&#95;ref&#95;1198"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical value</a> or <a id="&#95;ref&#95;1199"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#extended-mathematical-value">extended mathematical value</a> <var>x</var> to a Number is denoted as "the <a id="&#95;ref&#95;1200"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#number-value-for">Number value for</a> <var>x</var>" or <a id="𝔽"></a>𝔽(<var>x</var>), and is defined in <a id="&#95;ref&#95;44"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-number-type">6.1.6.1</a>. A conversion from an <a id="&#95;ref&#95;1201"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> <var>x</var> to a BigInt is denoted as "the <a id="bigint-value-for"></a>BigInt value for <var>x</var>" or <a id="ℤ"></a>ℤ(<var>x</var>). A conversion from a Number or BigInt <var>x</var> to a <a id="&#95;ref&#95;1202"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical value</a> is denoted as "the <a id="mathematical-value-of"></a>mathematical value of <var>x</var>", or <a id="ℝ"></a>ℝ(<var>x</var>). The <a id="&#95;ref&#95;1203"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value-of">mathematical value of</a> +0<sub>𝔽</sub> and -0<sub>𝔽</sub> is the <a id="&#95;ref&#95;1204"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical value</a> 0. The <a id="&#95;ref&#95;1205"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value-of">mathematical value of</a> non-<a id="&#95;ref&#95;1206"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#finite">finite</a> values is not defined. The <a id="extended-mathematical-value-of"></a>extended mathematical value of <var>x</var> is the <a id="&#95;ref&#95;1207"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value-of">mathematical value of</a> <var>x</var> for <a id="&#95;ref&#95;1208"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#finite">finite</a> values, and is +∞ and -∞ for +∞<sub>𝔽</sub> and -∞<sub>𝔽</sub> respectively; it is not defined for NaN.

The mathematical function <a id="eqn-abs"></a>abs(<var>x</var>) produces the absolute value of <var>x</var>, which is -<var>x</var> if <var>x</var> \< 0 and otherwise is <var>x</var> itself.

An <a id="interval"></a>interval from lower bound <var>a</var> to upper bound <var>b</var> is a possibly-infinite, possibly-empty set of numeric values of the same numeric type. Each bound will be described as either inclusive or exclusive, but not both. There are four kinds of intervals, as follows:

- An <a id="&#95;ref&#95;1227"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a> from <var>a</var> (inclusive) to <var>b</var> (inclusive), also called an <a id="inclusive-interval"></a>inclusive interval from <var>a</var> to <var>b</var>, includes all values <var>x</var> of the same numeric type such that <var>a</var> ≤ <var>x</var> ≤ <var>b</var>, and no others.
- An <a id="&#95;ref&#95;1228"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a> from <var>a</var> (inclusive) to <var>b</var> (exclusive) includes all values <var>x</var> of the same numeric type such that <var>a</var> ≤ <var>x</var> \< <var>b</var>, and no others.
- An <a id="&#95;ref&#95;1229"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a> from <var>a</var> (exclusive) to <var>b</var> (inclusive) includes all values <var>x</var> of the same numeric type such that <var>a</var> \< <var>x</var> ≤ <var>b</var>, and no others.
- An <a id="&#95;ref&#95;1230"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a> from <var>a</var> (exclusive) to <var>b</var> (exclusive) includes all values <var>x</var> of the same numeric type such that <var>a</var> \< <var>x</var> \< <var>b</var>, and no others.

For example, the <a id="&#95;ref&#95;1231"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a> from 1 (inclusive) to 2 (exclusive) consists of all <a id="&#95;ref&#95;1232"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#mathematical-value">mathematical values</a> between 1 and 2, including 1 and not including 2. For the purpose of defining intervals, -0<sub>𝔽</sub> \< +0<sub>𝔽</sub>, so, for example, an <a id="&#95;ref&#95;1233"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#inclusive-interval">inclusive interval</a> with a lower bound of +0<sub>𝔽</sub> includes +0<sub>𝔽</sub> but not -0<sub>𝔽</sub>. NaN is never included in an <a id="&#95;ref&#95;1234"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a>.

<a id="sec-ecmascript-language-types-number-type"></a>

## 6.1.6.1 The Number Type

The Number type has exactly 18,437,736,874,454,810,627 (that is, 2<sup>64</sup> - 2<sup>53</sup> + 3) values, representing the double-precision floating point <a id="&#95;ref&#95;1349"></a><a href="https://tc39.es/ecma262/2026/multipage/bibliography.html#sec-bibliography">IEEE 754-2019</a> binary64 values as specified in the IEEE Standard for Binary Floating-Point Arithmetic, except that the 9,007,199,254,740,990 (that is, 2<sup>53</sup> - 2) distinct NaN values of the IEEE Standard are represented in ECMAScript as a single special NaN value. (Note that the NaN value is produced by the program expression <code>NaN</code>.) In some implementations, external code might be able to detect a difference between various NaN values, but such behaviour is <a id="&#95;ref&#95;1350"></a><a href="https://tc39.es/ecma262/2026/multipage/overview.html#implementation-defined">implementation-defined</a>; to ECMAScript code, all NaN values are indistinguishable from each other.

Note

The bit pattern that might be observed in an ArrayBuffer (see <a id="&#95;ref&#95;95"></a><a href="https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-arraybuffer-objects">25.1</a>) or a SharedArrayBuffer (see <a id="&#95;ref&#95;96"></a><a href="https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-sharedarraybuffer-objects">25.2</a>) after a Number value has been stored into it is not necessarily the same as the internal representation of that Number value used by the ECMAScript implementation.

There are two other special values, called positive Infinity and negative Infinity. For brevity, these values are also referred to for expository purposes by the symbols +∞<sub>𝔽</sub> and -∞<sub>𝔽</sub>, respectively. (Note that these two infinite Number values are produced by the program expressions <code>+Infinity</code> (or simply <code>Infinity</code>) and <code>-Infinity</code>.)

The other 18,437,736,874,454,810,624 (that is, 2<sup>64</sup> - 2<sup>53</sup>) values are called the <a id="finite"></a>finite numbers. Half of these are positive numbers and half are negative numbers; for every <a id="&#95;ref&#95;1351"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#finite">finite</a> positive Number value there is a corresponding negative value having the same magnitude.

Note that there is both a positive zero and a negative zero. For brevity, these values are also referred to for expository purposes by the symbols +0<sub>𝔽</sub> and -0<sub>𝔽</sub>, respectively. (Note that these two different zero Number values are produced by the program expressions <code>+0</code> (or simply <code>0</code>) and <code>-0</code>.)

The 18,437,736,874,454,810,622 (that is, 2<sup>64</sup> - 2<sup>53</sup> - 2) <a id="&#95;ref&#95;1352"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#finite">finite</a> non-zero values are of two kinds:

18,428,729,675,200,069,632 (that is, 2<sup>64</sup> - 2<sup>54</sup>) of them are normalized, having the form

<var>s</var> × <var>m</var> × 2<sup><var>e</var></sup>

where <var>s</var> is 1 or -1, <var>m</var> is an <a id="&#95;ref&#95;1353"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> in the <a id="&#95;ref&#95;1354"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a> from 2<sup>52</sup> (inclusive) to 2<sup>53</sup> (exclusive), and <var>e</var> is an <a id="&#95;ref&#95;1355"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> in the <a id="&#95;ref&#95;1356"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#inclusive-interval">inclusive interval</a> from -1074 to 971.

The remaining 9,007,199,254,740,990 (that is, 2<sup>53</sup> - 2) values are denormalized, having the form

<var>s</var> × <var>m</var> × 2<sup><var>e</var></sup>

where <var>s</var> is 1 or -1, <var>m</var> is an <a id="&#95;ref&#95;1357"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> in the <a id="&#95;ref&#95;1358"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#interval">interval</a> from 0 (exclusive) to 2<sup>52</sup> (exclusive), and <var>e</var> is -1074.

Note that all the positive and negative <a id="&#95;ref&#95;1359"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integers</a> whose magnitude is no greater than 2<sup>53</sup> are representable in the Number type. The <a id="&#95;ref&#95;1360"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> 0 has two representations in the Number type: +0<sub>𝔽</sub> and -0<sub>𝔽</sub>.

A <a id="&#95;ref&#95;1361"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#finite">finite</a> number has an <em>odd significand</em> if it is non-zero and the <a id="&#95;ref&#95;1362"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> <var>m</var> used to express it (in one of the two forms shown above) is odd. Otherwise, it has an <em>even significand</em>.

In this specification, the phrase “the <a id="number-value-for"></a><a id="number-value"></a>Number value for <var>x</var>” where <var>x</var> represents an exact real mathematical quantity (which might even be an irrational number such as π) means a Number value chosen in the following manner. Consider the set of all <a id="&#95;ref&#95;1363"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#finite">finite</a> values of the Number type, with -0<sub>𝔽</sub> removed and with two additional values added to it that are not representable in the Number type, namely 2<sup>1024</sup> (which is +1 × 2<sup>53</sup> × 2<sup>971</sup>) and -2<sup>1024</sup> (which is -1 × 2<sup>53</sup> × 2<sup>971</sup>). Choose the member of this set that is closest in value to <var>x</var>. If two values of the set are equally close, then the one with an even significand is chosen; for this purpose, the two extra values 2<sup>1024</sup> and -2<sup>1024</sup> are considered to have even significands. Finally, if 2<sup>1024</sup> was chosen, replace it with +∞<sub>𝔽</sub>; if -2<sup>1024</sup> was chosen, replace it with -∞<sub>𝔽</sub>; if +0<sub>𝔽</sub> was chosen, replace it with -0<sub>𝔽</sub> if and only if <var>x</var> \< 0; any other chosen value is used unchanged. The result is the <a id="&#95;ref&#95;1364"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#number-value-for">Number value for</a> <var>x</var>. (This procedure corresponds exactly to the behaviour of the <a id="&#95;ref&#95;1365"></a><a href="https://tc39.es/ecma262/2026/multipage/bibliography.html#sec-bibliography">IEEE 754-2019</a> roundTiesToEven mode.)

The <a id="&#95;ref&#95;1366"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#number-value-for">Number value for</a> +∞ is +∞<sub>𝔽</sub>, and the <a id="&#95;ref&#95;1367"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#number-value-for">Number value for</a> -∞ is -∞<sub>𝔽</sub>.

<a id="sec-tostring"></a>
<a id="table-tostring-conversions"></a>

## 7.1.17 ToString ( <var>argument</var> )

The abstract operation ToString takes argument <var>argument</var> (an <a id="&#95;ref&#95;2257"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types">ECMAScript language value</a>) and returns either a <a id="&#95;ref&#95;2258"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-completion-record-specification-type">normal completion containing</a> a String or a <a id="&#95;ref&#95;2259"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-completion-record-specification-type">throw completion</a>. It converts <var>argument</var> to a value of type String. It performs the following steps when called:

1.  If <var>argument</var> <a id="&#95;ref&#95;2260"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-string-type">is a String</a>, return <var>argument</var>.
2.  If <var>argument</var> <a id="&#95;ref&#95;2261"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-symbol-type">is a Symbol</a>, throw a TypeError exception.
3.  If <var>argument</var> is undefined, return "undefined".
4.  If <var>argument</var> is null, return "null".
5.  If <var>argument</var> is true, return "true".
6.  If <var>argument</var> is false, return "false".
7.  If <var>argument</var> <a id="&#95;ref&#95;2262"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-number-type">is a Number</a>, return <a id="&#95;ref&#95;2263"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring">Number&#58;&#58;toString</a>(<var>argument</var>, 10).
8.  If <var>argument</var> <a id="&#95;ref&#95;2264"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-bigint-type">is a BigInt</a>, return <a id="&#95;ref&#95;2265"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-bigint-tostring">BigInt&#58;&#58;toString</a>(<var>argument</var>, 10).
9.  <a id="&#95;ref&#95;2266"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#assert">Assert</a>: <var>argument</var> <a id="&#95;ref&#95;2267"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-object-type">is an Object</a>.
10. Let <var>primValue</var> be ? <a id="&#95;ref&#95;2268"></a><a href="https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-toprimitive">ToPrimitive</a>(<var>argument</var>, string).
11. <a id="&#95;ref&#95;2269"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#assert">Assert</a>: <var>primValue</var> <a id="&#95;ref&#95;2270"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-object-type">is not an Object</a>.
12. Return ? <a id="&#95;ref&#95;2271"></a><a href="https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-tostring">ToString</a>(<var>primValue</var>).

<a id="sec-numeric-types-number-tostring"></a>
<a id="sec-tostring-applied-to-the-number-type"></a>

## 6.1.6.1.20 Number::toString ( <var>x</var>, <var>radix</var> )

The abstract operation Number::toString takes arguments <var>x</var> (a Number) and <var>radix</var> (an <a id="&#95;ref&#95;1461"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integer</a> in the <a id="&#95;ref&#95;1462"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#inclusive-interval">inclusive interval</a> from 2 to 36) and returns a String. It represents <var>x</var> as a String using a positional numeral system with radix <var>radix</var>. The digits used in the representation of a number using radix <var>r</var> are taken from the first <var>r</var> code units of "0123456789abcdefghijklmnopqrstuvwxyz" in order. The representation of numbers with magnitude greater than or equal to 1<sub>𝔽</sub> never includes leading zeroes. It performs the following steps when called:

1.  If <var>x</var> is NaN, return "NaN".
2.  If <var>x</var> is either +0<sub>𝔽</sub> or -0<sub>𝔽</sub>, return "0".
3.  If <var>x</var> \< -0<sub>𝔽</sub>, return the <a id="&#95;ref&#95;1463"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#string-concatenation">string-concatenation</a> of "-" and <a id="&#95;ref&#95;1464"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring">Number&#58;&#58;toString</a>(-<var>x</var>, <var>radix</var>).
4.  If <var>x</var> is +∞<sub>𝔽</sub>, return "Infinity".
5.  <a id="step-number-tostring-intermediate-values"></a>Let <var>n</var>, <var>k</var>, and <var>s</var> be <a id="&#95;ref&#95;1465"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integers</a> such that <var>k</var> ≥ 1, <var>radix</var><sup><var>k</var> - 1</sup> ≤ <var>s</var> \< <var>radix</var><sup><var>k</var></sup>, <a id="&#95;ref&#95;1466"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#%F0%9D%94%BD">𝔽</a>(<var>s</var> × <var>radix</var><sup><var>n</var> - <var>k</var></sup>) is <var>x</var>, and <var>k</var> is as small as possible. Note that <var>k</var> is the number of digits in the representation of <var>s</var> using radix <var>radix</var>, that <var>s</var> is not divisible by <var>radix</var>, and that the least significant digit of <var>s</var> is not necessarily uniquely determined by these criteria.
6.  If <var>radix</var> ≠ 10 or <var>n</var> is in the <a id="&#95;ref&#95;1467"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#inclusive-interval">inclusive interval</a> from -5 to 21, then
    1.  If <var>n</var> ≥ <var>k</var>, then
        1.  Return the <a id="&#95;ref&#95;1468"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#string-concatenation">string-concatenation</a> of:
            - the code units of the <var>k</var> digits of the representation of <var>s</var> using radix <var>radix</var>
            - <var>n</var> - <var>k</var> occurrences of the code unit 0x0030 (DIGIT ZERO)
    2.  If <var>n</var> \> 0, then
        1.  Return the <a id="&#95;ref&#95;1469"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#string-concatenation">string-concatenation</a> of:
            - the code units of the most significant <var>n</var> digits of the representation of <var>s</var> using radix <var>radix</var>
            - the code unit 0x002E (FULL STOP)
            - the code units of the remaining <var>k</var> - <var>n</var> digits of the representation of <var>s</var> using radix <var>radix</var>
    3.  <a id="&#95;ref&#95;1470"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#assert">Assert</a>: <var>n</var> ≤ 0.
    4.  Return the <a id="&#95;ref&#95;1471"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#string-concatenation">string-concatenation</a> of:
        - the code unit 0x0030 (DIGIT ZERO)
        - the code unit 0x002E (FULL STOP)
        - \-<var>n</var> occurrences of the code unit 0x0030 (DIGIT ZERO)
        - the code units of the <var>k</var> digits of the representation of <var>s</var> using radix <var>radix</var>
7.  NOTE: In this case, the input will be represented using scientific E notation, such as <code>1.2e+3</code>.
8.  <a id="&#95;ref&#95;1472"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#assert">Assert</a>: <var>radix</var> is 10.
9.  If <var>n</var> \< 0, then
    1.  Let <var>exponentSign</var> be the code unit 0x002D (HYPHEN-MINUS).
10. Else,
    1.  Let <var>exponentSign</var> be the code unit 0x002B (PLUS SIGN).
11. If <var>k</var> = 1, then
    1.  Return the <a id="&#95;ref&#95;1473"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#string-concatenation">string-concatenation</a> of:
        - the code unit of the single digit of <var>s</var>
        - the code unit 0x0065 (LATIN SMALL LETTER E)
        - <var>exponentSign</var>
        - the code units of the decimal representation of <a id="&#95;ref&#95;1474"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#eqn-abs">abs</a>(<var>n</var> - 1)
12. Return the <a id="&#95;ref&#95;1475"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#string-concatenation">string-concatenation</a> of:
    - the code unit of the most significant digit of the decimal representation of <var>s</var>
    - the code unit 0x002E (FULL STOP)
    - the code units of the remaining <var>k</var> - 1 digits of the decimal representation of <var>s</var>
    - the code unit 0x0065 (LATIN SMALL LETTER E)
    - <var>exponentSign</var>
    - the code units of the decimal representation of <a id="&#95;ref&#95;1476"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#eqn-abs">abs</a>(<var>n</var> - 1)

Note 1

The following observations may be useful as guidelines for implementations, but are not part of the normative requirements of this Standard:

- If x is any Number value other than -0<sub>𝔽</sub>, then <a id="&#95;ref&#95;1477"></a><a href="https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-tonumber">ToNumber</a>(<a id="&#95;ref&#95;1478"></a><a href="https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-tostring">ToString</a>(x)) is x.
- The least significant digit of s is not always uniquely determined by the requirements listed in step <a id="&#95;ref&#95;98"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#step-number-tostring-intermediate-values">5</a>.

Note 2

For implementations that provide more accurate conversions than required by the rules above, it is recommended that the following alternative version of step <a id="&#95;ref&#95;99"></a><a href="https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#step-number-tostring-intermediate-values">5</a> be used as a guideline:

5.  Let <var>n</var>, <var>k</var>, and <var>s</var> be <a id="&#95;ref&#95;1479"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#integer">integers</a> such that <var>k</var> ≥ 1, <var>radix</var><sup><var>k</var> - 1</sup> ≤ <var>s</var> \< <var>radix</var><sup><var>k</var></sup>, <a id="&#95;ref&#95;1480"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#%F0%9D%94%BD">𝔽</a>(<var>s</var> × <var>radix</var><sup><var>n</var> - <var>k</var></sup>) is <var>x</var>, and <var>k</var> is as small as possible. If there are multiple possibilities for <var>s</var>, choose the value of <var>s</var> for which <var>s</var> × <var>radix</var><sup><var>n</var> - <var>k</var></sup> is closest in value to <a id="&#95;ref&#95;1481"></a><a href="https://tc39.es/ecma262/2026/multipage/notational-conventions.html#%E2%84%9D">ℝ</a>(<var>x</var>). If there are two such possible values of <var>s</var>, choose the one that is even. Note that <var>k</var> is the number of digits in the representation of <var>s</var> using radix <var>radix</var> and that <var>s</var> is not divisible by <var>radix</var>.

Note 3

Implementers of ECMAScript may find useful the paper and code written by David M. Gay for binary-to-decimal conversion of floating-point numbers:

Gay, David M. Correctly Rounded Binary-Decimal and Decimal-Binary Conversions. Numerical Analysis, Manuscript 90-10. AT&T Bell Laboratories (Murray Hill, New Jersey). 30 November 1990. Available as<br>
<a href="https://ampl.com/_archive/first-website/REFS/rounding.pdf">https&#58;//ampl.com/_archive/first-website/REFS/rounding.pdf</a>. Associated code available as<br>
<a href="http://netlib.sandia.gov/fp/dtoa.c">http&#58;//netlib.sandia.gov/fp/dtoa.c</a> and as<br>
<a href="http://netlib.sandia.gov/fp/g_fmt.c">http&#58;//netlib.sandia.gov/fp/g_fmt.c</a> and may also be found at the various <code>netlib</code> mirror sites.
