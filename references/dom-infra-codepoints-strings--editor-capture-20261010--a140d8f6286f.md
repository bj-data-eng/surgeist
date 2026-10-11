# Infra Standard: bounded DOM foundation source capture

Attribution and reformatting notice added for Surgeist on 2026-10-10. Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Original English standard and normative/informative labels remain authoritative; this partial format conversion is implementation support, not a replacement specification or an implementation-conformance claim. No endorsement is implied.

Source: [Infra Standard](https://infra.spec.whatwg.org/). Retrieved 2026-10-10. Displayed status: Infra Living Standard — Last Updated 5 October 2026.

The exact upstream repository LICENSE files from DOM `3071e5f269448350fdea79bb284eedaf17bc431e`, HTML `877d6146487f15b15bc8dd0e3c625063a37b456a`, Web IDL `0620cf171d1b97138847088748e363cae4005156` and Infra `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a` all match the already retained [WHATWG LICENSE](../licenses/whatwg/LICENSE) byte-for-byte, SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`. Specification prose is Creative Commons Attribution 4.0 International; the BSD 3-Clause scope applies when portions are incorporated into code, not as an alternative license for this Markdown.

Full retrieved HTML: 383,471 bytes, SHA-256 `aa22601a985cb01c51678e531e6a131174a128881027ef98d37de8ad150757cb`. Complete-section extraction input: 56,104 bytes, SHA-256 `a140d8f6286f6f39d5ac10e06849cac58c293f23942acecafac31f53aa2b1a56`. Source revision metadata: `c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a`.

Retained complete heading sections, including descendant clauses and bounded at the next equal-or-higher-level heading: `code-points`, `strings`, `ipr`. Original source header is also retained. Every other clause and all cross-chapter references remain upstream. These are explicitly partial captures. The living page explicitly identifies its source revision above; exact capture hashes preserve that edition as implementation evidence. String types and algorithms are selected independently of JavaScript bindings.

Conversion uses existing repository HTML preparation/table-span models, approved Pandoc 3.1.11.1 and the semantic Lua filter. Source scripts/styles are omitted without execution. Source headings, IDs, links, prose, inline code, variables, literal blocks, images and every selected table cell/span are compared against parsed GFM. Selected tables remain literal semantic HTML, preserving every original table cell, code/link/variable, row/column span and anchor; table flattening is not used. Source examples and Web IDL remain evidence even where the corresponding JavaScript API is outside Surgeist's native Rust contract.

---

<!-- captured-body-start -->

[![WHATWG](https://resources.whatwg.org/logo-infra.svg)](https://whatwg.org/)

# <a id="title"></a>Infra

<a id="subtitle"></a>Living Standard — Last Updated 5 October 2026

<strong>Participate:</strong>

[GitHub whatwg/infra](https://github.com/whatwg/infra) ([new issue](https://github.com/whatwg/infra/issues/new/choose), [open issues](https://github.com/whatwg/infra/issues))

[Chat on Matrix](https://whatwg.org/chat)

<strong>Commits:</strong>

[GitHub whatwg/infra/commits](https://github.com/whatwg/infra/commits)

<a id="commit-snapshot-link"></a>[Snapshot as of this commit](https://infra.spec.whatwg.org/commit-snapshots/c67ae1fb1d161b23d0f8c9bf9917ea1993f7b19a/)

<strong>Tests:</strong>

[web-platform-tests infra/](https://github.com/web-platform-tests/wpt/tree/master/infra) ([ongoing work](https://github.com/web-platform-tests/wpt/labels/infra))

<strong>Translations (non-normative):</strong>

[日本語](https://triple-underscore.github.io/infra-ja.html)

[简体中文](https://htmlspecs.com/infra/)

[한국어](https://ko.htmlspecs.com/infra/)

### <a id="code-points"></a>4.6. Code points[](#code-points)

A <a id="code-point"></a><strong>code point</strong> is a Unicode code point and is represented as "U+" followed by four-to-six <a id="ref-for-ascii-upper-hex-digit①"></a>[ASCII upper hex digits](#ascii-upper-hex-digit), in the range U+0000 to U+10FFFF, inclusive. A <a id="ref-for-code-point②"></a>[code point](#code-point)’s <a id="code-point-value"></a><strong>value</strong> is its underlying number.

A <a id="ref-for-code-point③"></a>[code point](#code-point) may be followed by its name, by its rendered form between parentheses when it is not U+0028 or U+0029, or by both. Documents using the Infra Standard are encouraged to follow <a id="ref-for-code-point④"></a>[code points](#code-point) by their name when they cannot be rendered or are U+0028 or U+0029; otherwise, follow them by their rendered form between parentheses, for legibility.

A <a id="ref-for-code-point⑤"></a>[code point](#code-point)’s name is defined in Unicode and represented in <a id="ref-for-ascii-uppercase①"></a>[ASCII uppercase](#ascii-uppercase). [\[UNICODE\]](https://infra.spec.whatwg.org/#biblio-unicode)

<a id="example-code-point-notation"></a>

<strong>Example:</strong>

[](#example-code-point-notation)

The <a id="ref-for-code-point⑥"></a>[code point](#code-point) rendered as 🤔 is represented as U+1F914.

When referring to that <a id="ref-for-code-point⑦"></a>[code point](#code-point), we might say "U+1F914 (🤔)", to provide extra context. Documents are allowed to use "U+1F914 THINKING FACE (🤔)" as well, though this is somewhat verbose.

<a id="example-code-point-notation-hard-to-render"></a>

<strong>Example:</strong>

[](#example-code-point-notation-hard-to-render)<a id="ref-for-code-point⑧"></a>[Code points](#code-point) that are difficult to render unambigiously, such as U+000A, can be referred to as "U+000A LF". U+0029 can be referred to as "U+0029 RIGHT PARENTHESIS", because even though it renders, this avoids unmatched parentheses.

<a id="ref-for-code-point⑨"></a>[Code points](#code-point) are sometimes referred to as <a id="ref-for-code-point①⓪"></a>[characters](#code-point) and in certain contexts are prefixed with "0x" rather than "U+".

A <a id="leading-surrogate"></a><strong>leading surrogate</strong> is a <a id="ref-for-code-point①①"></a>[code point](#code-point) that is in the range U+D800 to U+DBFF, inclusive.

A <a id="trailing-surrogate"></a><strong>trailing surrogate</strong> is a <a id="ref-for-code-point①②"></a>[code point](#code-point) that is in the range U+DC00 to U+DFFF, inclusive.

A <a id="surrogate"></a><strong>surrogate</strong> is a <a id="ref-for-leading-surrogate"></a>[leading surrogate](#leading-surrogate) or a <a id="ref-for-trailing-surrogate"></a>[trailing surrogate](#trailing-surrogate).

A <a id="scalar-value"></a><strong>scalar value</strong> is a <a id="ref-for-code-point①③"></a>[code point](#code-point) that is not a <a id="ref-for-surrogate"></a>[surrogate](#surrogate).

A <a id="noncharacter"></a><strong>noncharacter</strong> is a <a id="ref-for-code-point①④"></a>[code point](#code-point) that is in the range U+FDD0 to U+FDEF, inclusive, or U+FFFE, U+FFFF, U+1FFFE, U+1FFFF, U+2FFFE, U+2FFFF, U+3FFFE, U+3FFFF, U+4FFFE, U+4FFFF, U+5FFFE, U+5FFFF, U+6FFFE, U+6FFFF, U+7FFFE, U+7FFFF, U+8FFFE, U+8FFFF, U+9FFFE, U+9FFFF, U+AFFFE, U+AFFFF, U+BFFFE, U+BFFFF, U+CFFFE, U+CFFFF, U+DFFFE, U+DFFFF, U+EFFFE, U+EFFFF, U+FFFFE, U+FFFFF, U+10FFFE, or U+10FFFF.

An <a id="ascii-code-point"></a><strong>ASCII code point</strong> is a <a id="ref-for-code-point①⑤"></a>[code point](#code-point) in the range U+0000 NULL to U+007F DELETE, inclusive.

An <a id="ascii-tab-or-newline"></a><strong>ASCII tab or newline</strong> is U+0009 TAB, U+000A LF, or U+000D CR.

<a id="ascii-whitespace"></a><strong>ASCII whitespace</strong> is U+0009 TAB, U+000A LF, U+000C FF, U+000D CR, or U+0020 SPACE.

<strong>Note:</strong>

"Whitespace" is a mass noun.

<strong>Note:</strong>

The XML, JSON, and parts of the HTTP specifications exclude U+000C FF in their definition of whitespace:

- [XML’s S production](https://www.w3.org/TR/xml/#NT-S)
- [JSON’s ws production](https://www.rfc-editor.org/info/rfc8259/#section-2)
- <a id="ref-for-http-whitespace"></a>[HTTP whitespace](https://fetch.spec.whatwg.org/#http-whitespace)

Prefer using Infra’s <a id="ref-for-ascii-whitespace"></a>[ASCII whitespace](#ascii-whitespace) definition for new features, unless your specification deals exclusively with XML/JSON/HTTP.

A <a id="c0-control"></a><strong>C0 control</strong> is a <a id="ref-for-code-point①⑥"></a>[code point](#code-point) in the range U+0000 NULL to U+001F INFORMATION SEPARATOR ONE, inclusive.

A <a id="c0-control-or-space"></a><strong>C0 control or space</strong> is a <a id="ref-for-c0-control"></a>[C0 control](#c0-control) or U+0020 SPACE.

A <a id="control"></a><strong>control</strong> is a <a id="ref-for-c0-control①"></a>[C0 control](#c0-control) or a <a id="ref-for-code-point①⑦"></a>[code point](#code-point) in the range U+007F DELETE to U+009F APPLICATION PROGRAM COMMAND, inclusive.

An <a id="ascii-digit"></a><strong>ASCII digit</strong> is a <a id="ref-for-code-point①⑧"></a>[code point](#code-point) in the range U+0030 (0) to U+0039 (9), inclusive.

An <a id="ascii-upper-hex-digit"></a><strong>ASCII upper hex digit</strong> is an <a id="ref-for-ascii-digit"></a>[ASCII digit](#ascii-digit) or a <a id="ref-for-code-point①⑨"></a>[code point](#code-point) in the range U+0041 (A) to U+0046 (F), inclusive.

An <a id="ascii-lower-hex-digit"></a><strong>ASCII lower hex digit</strong> is an <a id="ref-for-ascii-digit①"></a>[ASCII digit](#ascii-digit) or a <a id="ref-for-code-point②⓪"></a>[code point](#code-point) in the range U+0061 (a) to U+0066 (f), inclusive.

An <a id="ascii-hex-digit"></a><strong>ASCII hex digit</strong> is an <a id="ref-for-ascii-upper-hex-digit②"></a>[ASCII upper hex digit](#ascii-upper-hex-digit) or <a id="ref-for-ascii-lower-hex-digit"></a>[ASCII lower hex digit](#ascii-lower-hex-digit).

An <a id="ascii-upper-alpha"></a><strong>ASCII upper alpha</strong> is a <a id="ref-for-code-point②①"></a>[code point](#code-point) in the range U+0041 (A) to U+005A (Z), inclusive.

An <a id="ascii-lower-alpha"></a><strong>ASCII lower alpha</strong> is a <a id="ref-for-code-point②②"></a>[code point](#code-point) in the range U+0061 (a) to U+007A (z), inclusive.

An <a id="ascii-alpha"></a><strong>ASCII alpha</strong> is an <a id="ref-for-ascii-upper-alpha"></a>[ASCII upper alpha](#ascii-upper-alpha) or <a id="ref-for-ascii-lower-alpha"></a>[ASCII lower alpha](#ascii-lower-alpha).

An <a id="ascii-alphanumeric"></a><strong>ASCII alphanumeric</strong> is an <a id="ref-for-ascii-digit②"></a>[ASCII digit](#ascii-digit) or <a id="ref-for-ascii-alpha"></a>[ASCII alpha](#ascii-alpha).

### <a id="strings"></a>4.7. Strings[](#strings)

A <a id="string"></a><strong>string</strong> is a sequence of <a id="ref-for-16-bit-unsigned-integer"></a>[16-bit unsigned integers](https://infra.spec.whatwg.org/#16-bit-unsigned-integer), also known as <a id="code-unit"></a><strong>code units</strong>. A <a id="ref-for-string①②"></a>[string](#string) is also known as a <a id="javascript-string"></a>[JavaScript string](#string). <a id="ref-for-string①③"></a>[Strings](#string) are denoted by double quotes and monospace font.

<a id="example-string-notation"></a>

<strong>Example:</strong>

[](#example-string-notation)"<code>Hello, world!</code>" is a string.

<strong>Note:</strong>

This is different from how Unicode defines "code unit". In particular it refers exclusively to how Unicode defines it for Unicode 16-bit strings. [\[UNICODE\]](https://infra.spec.whatwg.org/#biblio-unicode)

A <a id="ref-for-string①④"></a>[string](#string) can also be interpreted as containing <a id="ref-for-code-point②③"></a>[code points](#code-point), per the conversion defined in <a id="ref-for-sec-ecmascript-language-types-string-type"></a>[The String Type](https://tc39.github.io/ecma262/#sec-ecmascript-language-types-string-type) section of the JavaScript specification. [\[ECMA-262\]](https://infra.spec.whatwg.org/#biblio-ecma-262)

<strong>Note:</strong>

This conversion process converts surrogate pairs into their corresponding <a id="ref-for-scalar-value"></a>[scalar value](#scalar-value) and maps any remaining surrogates to their corresponding <a id="ref-for-code-point②④"></a>[code point](#code-point), leaving them effectively as-is.

<a id="example-javascript-string-in-code-points"></a>

<strong>Example:</strong>

[](#example-javascript-string-in-code-points)A <a id="ref-for-string①⑤"></a>[string](#string) consisting of the <a id="ref-for-code-unit①"></a>[code units](#code-unit) 0xD83D, 0xDCA9, and 0xD800, when interpreted as containing <a id="ref-for-code-point②⑤"></a>[code points](#code-point), would consist of the <a id="ref-for-code-point②⑥"></a>[code points](#code-point) U+1F4A9 and U+D800.

A <a id="ref-for-string①⑥"></a>[string](#string)’s <a id="string-length"></a><strong><a id="javascript-string-length"></a>length</strong> is the number of <a id="ref-for-code-unit②"></a>[code units](#code-unit) it contains.

A <a id="ref-for-string①⑦"></a>[string](#string)’s <a id="string-code-point-length"></a><strong>code point length</strong> is the number of <a id="ref-for-code-point②⑦"></a>[code points](#code-point) it contains.

------------------------------------------------------------------------

To signify <a id="ref-for-string①⑧"></a>[strings](#string) with additional restrictions on the <a id="ref-for-code-point②⑧"></a>[code points](#code-point) they can contain this specification defines <a id="ref-for-ascii-string"></a>[ASCII strings](#ascii-string), <a id="ref-for-isomorphic-string"></a>[isomorphic strings](#isomorphic-string), and <a id="ref-for-scalar-value-string"></a>[scalar value strings](#scalar-value-string). Using these improves clarity in specifications.

An <a id="ascii-string"></a><strong>ASCII string</strong> is a <a id="ref-for-string①⑨"></a>[string](#string) whose <a id="ref-for-code-point②⑨"></a>[code points](#code-point) are all <a id="ref-for-ascii-code-point"></a>[ASCII code points](#ascii-code-point).

An <a id="isomorphic-string"></a><strong>isomorphic string</strong> is a <a id="ref-for-string②⓪"></a>[string](#string) whose <a id="ref-for-code-point③⓪"></a>[code points](#code-point) are all in the range U+0000 NULL to U+00FF (ÿ), inclusive.

A <a id="scalar-value-string"></a><strong>scalar value string</strong> is a <a id="ref-for-string②①"></a>[string](#string) whose <a id="ref-for-code-point③①"></a>[code points](#code-point) are all <a id="ref-for-scalar-value①"></a>[scalar values](#scalar-value).

<strong>Note:</strong>

A <a id="ref-for-scalar-value-string①"></a>[scalar value string](#scalar-value-string) is useful for any kind of I/O or other kind of operation where <a id="ref-for-utf-8-encode①"></a>[UTF-8 encode](https://encoding.spec.whatwg.org/#utf-8-encode) comes into play.

------------------------------------------------------------------------

To <a id="javascript-string-convert"></a><strong>convert</strong> a <a id="ref-for-string②②"></a>[string](#string) into a <a id="ref-for-scalar-value-string②"></a>[scalar value string](#scalar-value-string), replace any <a id="ref-for-surrogate①"></a>[surrogates](#surrogate) with U+FFFD (�).

<strong>Note:</strong>

The replaced surrogates are never part of surrogate pairs, since the process of interpreting the string as containing <a id="ref-for-code-point③②"></a>[code points](#code-point) will have converted surrogate pairs into <a id="ref-for-scalar-value②"></a>[scalar values](#scalar-value).

A <a id="ref-for-scalar-value-string③"></a>[scalar value string](#scalar-value-string) can always be used as a <a id="ref-for-string②③"></a>[string](#string) implicitly since every <a id="ref-for-scalar-value-string④"></a>[scalar value string](#scalar-value-string) is a <a id="ref-for-string②④"></a>[string](#string). On the other hand, a <a id="ref-for-string②⑤"></a>[string](#string) can only be implicitly used as a <a id="ref-for-scalar-value-string⑤"></a>[scalar value string](#scalar-value-string) if it is known to not contain <a id="ref-for-surrogate②"></a>[surrogates](#surrogate); otherwise a <a id="ref-for-javascript-string-convert"></a>[conversion](#javascript-string-convert) is to be performed.

An implementation likely has to perform explicit conversion, depending on how it actually ends up representing <a id="ref-for-string②⑥"></a>[strings](#string) and <a id="ref-for-scalar-value-string⑥"></a>[scalar value strings](#scalar-value-string). It is fairly typical for implementations to have multiple implementations of <a id="ref-for-string②⑦"></a>[strings](#string) alone for performance and memory reasons.

------------------------------------------------------------------------

A <a id="ref-for-string②⑧"></a>[string](#string) <var>a</var> <a id="string-is"></a><strong>is</strong> or is <a id="ref-for-string-is"></a>[identical to](#string-is) a <a id="ref-for-string②⑨"></a>[string](#string) <var>b</var> if it consists of the same sequence of <a id="ref-for-code-unit③"></a>[code units](#code-unit).

Except where otherwise stated, all string comparisons use <a id="ref-for-string-is①"></a>[is](#string-is).

<strong>Note:</strong>

This type of <a id="ref-for-string③⓪"></a>[string](#string) comparison was formerly known as a "case-sensitive" comparison in HTML. Strings that compare as <a id="ref-for-string-is②"></a>[identical to](#string-is) one another are not only sensitive to case variation (such as UPPER and lower case), but also to other code point encoding choices, such as normalization form or the order of combining marks. Two strings that are visually or even canonically equivalent according to Unicode might still not be <a id="ref-for-string-is③"></a>[identical to](#string-is) each other. [\[HTML\]](https://infra.spec.whatwg.org/#biblio-html) [\[UNICODE\]](https://infra.spec.whatwg.org/#biblio-unicode)

A <a id="ref-for-string③①"></a>[string](#string) <var>potentialPrefix</var> is a <a id="code-unit-prefix"></a><strong>code unit prefix</strong> of a <a id="ref-for-string③②"></a>[string](#string) <var>input</var> if the following steps return true:

1.  Let <var>i</var> be 0.

2.  <a id="ref-for-iteration-while①"></a>[While](https://infra.spec.whatwg.org/#iteration-while) true:

    1.  If <var>i</var> is greater than or equal to <var>potentialPrefix</var>’s <a id="ref-for-string-length"></a>[length](#string-length), then return true.

    2.  If <var>i</var> is greater than or equal to <var>input</var>’s <a id="ref-for-string-length①"></a>[length](#string-length), then return false.

    3.  Let <var>potentialPrefixCodeUnit</var> be the <var>i</var>th <a id="ref-for-code-unit④"></a>[code unit](#code-unit) of <var>potentialPrefix</var>.

    4.  Let <var>inputCodeUnit</var> be the <var>i</var>th <a id="ref-for-code-unit⑤"></a>[code unit](#code-unit) of <var>input</var>.

    5.  Return false if <var>potentialPrefixCodeUnit</var> is not <var>inputCodeUnit</var>.

    6.  Set <var>i</var> to <var>i</var> + 1.

When it is clear from context that <a id="ref-for-code-unit⑥"></a>[code units](#code-unit) are in play, e.g., because one of the strings is a literal containing only characters that are in the range U+0020 SPACE to U+007E (~), "<var>input</var> <a id="string-starts-with"></a><strong>starts with</strong> <var>potentialPrefix</var>" can be used as a synonym for "<var>potentialPrefix</var> is a <a id="ref-for-code-unit-prefix"></a>[code unit prefix](#code-unit-prefix) of <var>input</var>".

<a id="code-unit-prefix-example"></a>

<strong>Example:</strong>

[](#code-unit-prefix-example)With unknown values, it is good to be explicit: <var>targetString</var> is a <a id="ref-for-code-unit-prefix①"></a>[code unit prefix](#code-unit-prefix) of <var>userInput</var>. But with a literal, we can use plainer language: <var>userInput</var> <a id="ref-for-string-starts-with"></a>[starts with](#string-starts-with) "<code>!</code>".

A <a id="ref-for-string③③"></a>[string](#string) <var>potentialSuffix</var> is a <a id="code-unit-suffix"></a><strong>code unit suffix</strong> of a <a id="ref-for-string③④"></a>[string](#string) <var>input</var> if the following steps return true:

1.  Let <var>i</var> be 1.

2.  <a id="ref-for-iteration-while②"></a>[While](https://infra.spec.whatwg.org/#iteration-while) true:

    1.  Let <var>potentialSuffixIndex</var> be <var>potentialSuffix</var>’s <a id="ref-for-string-length②"></a>[length](#string-length) − <var>i</var>.

    2.  Let <var>inputIndex</var> be <var>input</var>’s <a id="ref-for-string-length③"></a>[length](#string-length) − <var>i</var>.

    3.  If <var>potentialSuffixIndex</var> is less than 0, then return true.

    4.  If <var>inputIndex</var> is less than 0, then return false.

    5.  Let <var>potentialSuffixCodeUnit</var> be the <var>potentialSuffixIndex</var>th <a id="ref-for-code-unit⑦"></a>[code unit](#code-unit) of <var>potentialSuffix</var>.

    6.  Let <var>inputCodeUnit</var> be the <var>inputIndex</var>th <a id="ref-for-code-unit⑧"></a>[code unit](#code-unit) of <var>input</var>.

    7.  Return false if <var>potentialSuffixCodeUnit</var> is not <var>inputCodeUnit</var>.

    8.  Set <var>i</var> to <var>i</var> + 1.

When it is clear from context that <a id="ref-for-code-unit⑨"></a>[code units](#code-unit) are in play, e.g., because one of the strings is a literal containing only characters that are in the range U+0020 SPACE to U+007E (~), "<var>input</var> <a id="string-ends-with"></a><strong>ends with</strong> <var>potentialSuffix</var>" can be used as a synonym for "<var>potentialSuffix</var> is a <a id="ref-for-code-unit-suffix"></a>[code unit suffix](#code-unit-suffix) of <var>input</var>".

<a id="code-unit-suffix-example"></a>

<strong>Example:</strong>

[](#code-unit-suffix-example)With unknown values, it is good to be explicit: <var>targetString</var> is a <a id="ref-for-code-unit-suffix①"></a>[code unit suffix](#code-unit-suffix) of <var>domain</var>. But with a literal, we can use plainer language: <var>domain</var> <a id="ref-for-string-ends-with"></a>[ends with](#string-ends-with) "<code>.</code>".

------------------------------------------------------------------------

A <a id="ref-for-string③⑤"></a>[string](#string) <var>a</var> is <a id="code-unit-less-than"></a><strong>code unit less than</strong> a <a id="ref-for-string③⑥"></a>[string](#string) <var>b</var> if the following steps return true:

1.  If <var>b</var> is a <a id="ref-for-code-unit-prefix②"></a>[code unit prefix](#code-unit-prefix) of <var>a</var>, then return false.

2.  If <var>a</var> is a <a id="ref-for-code-unit-prefix③"></a>[code unit prefix](#code-unit-prefix) of <var>b</var>, then return true.

3.  Let <var>n</var> be the smallest index such that the <var>n</var>th <a id="ref-for-code-unit①⓪"></a>[code unit](#code-unit) of <var>a</var> is different from the <var>n</var>th code unit of <var>b</var>. (There has to be such an index, since neither string is a prefix of the other.)

4.  If the <var>n</var>th code unit of <var>a</var> is less than the <var>n</var>th code unit of <var>b</var>, then return true.

5.  Return false.

<strong>Note:</strong>

This matches the ordering used by JavaScript’s <code>&#60;</code> operator, and its <code><a id="ref-for-sec-array.prototype.sort"></a>[sort()](https://tc39.github.io/ecma262/#sec-array.prototype.sort)</code> method on an array of strings. This ordering compares the 16-bit code units in each string, producing a highly efficient, consistent, and deterministic sort order. The resulting ordering will not match any particular alphabet or lexicographic order, particularly for <a id="ref-for-code-point③③"></a>[code points](#code-point) represented by a surrogate pair. [\[ECMA-262\]](https://infra.spec.whatwg.org/#biblio-ecma-262)

<a id="example-code-unit-less-than"></a>

<strong>Example:</strong>

[](#example-code-unit-less-than)For example, the code point U+FF5E FULLWIDTH TILDE (～) is obviously less than the code point U+1F600 (😀), but the tilde is composed of a single code unit 0xFF5E, while the smiley is composed of two code units 0xD83D and 0XDE00, so the smiley is <a id="ref-for-code-unit-less-than"></a>[code unit less than](#code-unit-less-than) the tilde.

------------------------------------------------------------------------

The <a id="code-unit-substring"></a><strong>code unit substring</strong> from <var>start</var> with length <var>length</var> within a <a id="ref-for-string③⑦"></a>[string](#string) <var>string</var> is determined as follows:

1.  <a id="ref-for-assert⑤"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>start</var> and <var>length</var> are nonnegative.

2.  <a id="ref-for-assert⑥"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>start</var> + <var>length</var> is less than or equal to <var>string</var>’s <a id="ref-for-string-length④"></a>[length](#string-length).

3.  Let <var>result</var> be the empty string.

4.  <a id="ref-for-list-iterate④"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>i</var> of <a id="ref-for-the-exclusive-range"></a>[the range](https://infra.spec.whatwg.org/#the-exclusive-range) from <var>start</var> to <var>start</var> + <var>length</var>, exclusive: append the <var>i</var>th <a id="ref-for-code-unit①①"></a>[code unit](#code-unit) of <var>string</var> to <var>result</var>.

5.  Return <var>result</var>.

The <a id="code-unit-substring-by-positions"></a><strong>code unit substring</strong> from <var>start</var> to <var>end</var> within a <a id="ref-for-string③⑧"></a>[string](#string) <var>string</var> is the <a id="ref-for-code-unit-substring"></a>[code unit substring](#code-unit-substring) from <var>start</var> with length <var>end</var> − <var>start</var> within <var>string</var>.

The <a id="code-unit-substring-to-the-end-of-the-string"></a><strong>code unit substring</strong> from <var>start</var> to the end of a <a id="ref-for-string③⑨"></a>[string](#string) <var>string</var> is the <a id="ref-for-code-unit-substring-by-positions"></a>[code unit substring](#code-unit-substring-by-positions) from <var>start</var> to <var>string</var>’s <a id="ref-for-string-length⑤"></a>[length](#string-length) within <var>string</var>.

<a id="example-code-unit-substring"></a>

<strong>Example:</strong>

[](#example-code-unit-substring)The <a id="ref-for-code-unit-substring①"></a>[code unit substring](#code-unit-substring) from 1 with length 3 within "<code>Hello world</code>" is "<code>ell</code>". This can also be expressed as the <a id="ref-for-code-unit-substring-by-positions①"></a>[code unit substring](#code-unit-substring-by-positions) from 1 to 4.

<strong>Note:</strong>

The numbers given to these algorithms are best thought of as positions <em>between</em> <a id="ref-for-code-unit①②"></a>[code units](#code-unit), not indices of the code units themselves. The substring returned is then formed by the code units between these positions. That explains why, for example, the <a id="ref-for-code-unit-substring-by-positions②"></a>[code unit substring](#code-unit-substring-by-positions) from 0 to 0 within the empty string is the empty string, even though there is no code unit at index 0 within the empty string.

The <a id="code-point-substring"></a><strong>code point substring</strong> within a <a id="ref-for-string④⓪"></a>[string](#string) <var>string</var> from <var>start</var> with length <var>length</var> is determined as follows:

1.  <a id="ref-for-assert⑦"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>start</var> and <var>length</var> are nonnegative.

2.  <a id="ref-for-assert⑧"></a>[Assert](https://infra.spec.whatwg.org/#assert): <var>start</var> + <var>length</var> is less than or equal to <var>string</var>’s <a id="ref-for-string-code-point-length①"></a>[code point length](#string-code-point-length).

3.  Let <var>result</var> be the empty string.

4.  <a id="ref-for-list-iterate⑤"></a>[For each](https://infra.spec.whatwg.org/#list-iterate) <var>i</var> of <a id="ref-for-the-exclusive-range①"></a>[the range](https://infra.spec.whatwg.org/#the-exclusive-range) from <var>start</var> to <var>start</var> + <var>length</var>, exclusive: append the <var>i</var>th <a id="ref-for-code-point③④"></a>[code point](#code-point) of <var>string</var> to <var>result</var>.

5.  Return <var>result</var>.

The <a id="code-point-substring-by-positions"></a><strong>code point substring</strong> from <var>start</var> to <var>end</var> within a <a id="ref-for-string④①"></a>[string](#string) <var>string</var> is the <a id="ref-for-code-point-substring"></a>[code point substring](#code-point-substring) within <var>string</var> from <var>start</var> with length <var>end</var> − <var>start</var>.

The <a id="code-point-substring-to-the-end-of-the-string"></a><strong>code point substring</strong> from <var>start</var> to the end of a <a id="ref-for-string④②"></a>[string](#string) <var>string</var> is the <a id="ref-for-code-point-substring-by-positions"></a>[code point substring](#code-point-substring-by-positions) from <var>start</var> to <var>string</var>’s <a id="ref-for-string-code-point-length②"></a>[code point length](#string-code-point-length) within <var>string</var>.

<a id="example-code-unit-vs-point-substring"></a>

<strong>Example:</strong>

[](#example-code-unit-vs-point-substring)

Generally, <a id="ref-for-code-unit-substring②"></a>[code unit substring](#code-unit-substring) is used when given developer-supplied positions or lengths, since that is how string indexing works in JavaScript. See, for example, the methods of the <code><a id="ref-for-characterdata"></a>[CharacterData](https://dom.spec.whatwg.org/#characterdata)</code> class. [\[DOM\]](https://infra.spec.whatwg.org/#biblio-dom)

Otherwise, <a id="ref-for-code-point-substring①"></a>[code point substring](#code-point-substring) is likely to be better. For example, the <a id="ref-for-code-point-substring②"></a>[code point substring](#code-point-substring) from 0 with length 1 within "<code>👽</code>" is "<code>👽</code>", whereas the <a id="ref-for-code-unit-substring③"></a>[code unit substring](#code-unit-substring) from 0 with length 1 within "<code>👽</code>" is the <a id="ref-for-string④③"></a>[string](#string) containing the single <a id="ref-for-surrogate③"></a>[surrogate](#surrogate) U+D83B.

------------------------------------------------------------------------

To <a id="isomorphic-encode"></a><strong>isomorphic encode</strong> an <a id="ref-for-isomorphic-string①"></a>[isomorphic string](#isomorphic-string) <var>input</var>: return a <a id="ref-for-byte-sequence①⑥"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence) whose <a id="ref-for-byte-sequence-length③"></a>[length](https://infra.spec.whatwg.org/#byte-sequence-length) is equal to <var>input</var>’s <a id="ref-for-string-code-point-length③"></a>[code point length](#string-code-point-length) and whose <a id="ref-for-byte①②"></a>[bytes](https://infra.spec.whatwg.org/#byte) have the same <a id="ref-for-byte-value②"></a>[values](https://infra.spec.whatwg.org/#byte-value) as the <a id="ref-for-code-point-value①"></a>[values](#code-point-value) of <var>input</var>’s <a id="ref-for-code-point③⑤"></a>[code points](#code-point), in the same order.

------------------------------------------------------------------------

To <a id="ascii-lowercase"></a><strong>ASCII lowercase</strong> a <a id="ref-for-string④④"></a>[string](#string), replace all <a id="ref-for-ascii-upper-alpha①"></a>[ASCII upper alphas](#ascii-upper-alpha) in the <a id="ref-for-string④⑤"></a>[string](#string) with their corresponding <a id="ref-for-code-point③⑥"></a>[code point](#code-point) in <a id="ref-for-ascii-lower-alpha①"></a>[ASCII lower alpha](#ascii-lower-alpha).

To <a id="ascii-uppercase"></a><strong>ASCII uppercase</strong> a <a id="ref-for-string④⑥"></a>[string](#string), replace all <a id="ref-for-ascii-lower-alpha②"></a>[ASCII lower alphas](#ascii-lower-alpha) in the <a id="ref-for-string④⑦"></a>[string](#string) with their corresponding <a id="ref-for-code-point③⑦"></a>[code point](#code-point) in <a id="ref-for-ascii-upper-alpha②"></a>[ASCII upper alpha](#ascii-upper-alpha).

A <a id="ref-for-string④⑧"></a>[string](#string) <var>A</var> is an <a id="ascii-case-insensitive"></a><strong>ASCII case-insensitive</strong> match for a <a id="ref-for-string④⑨"></a>[string](#string) <var>B</var>, if the <a id="ref-for-ascii-lowercase"></a>[ASCII lowercase](#ascii-lowercase) of <var>A</var> is the <a id="ref-for-ascii-lowercase①"></a>[ASCII lowercase](#ascii-lowercase) of <var>B</var>.

To <a id="ascii-encode"></a><strong>ASCII encode</strong> an <a id="ref-for-ascii-string①"></a>[ASCII string](#ascii-string) <var>input</var>: return the <a id="ref-for-isomorphic-encode①"></a>[isomorphic encoding](#isomorphic-encode) of <var>input</var>.

<strong>Note:</strong>

<a id="ref-for-isomorphic-encode②"></a>[Isomorphic encode](#isomorphic-encode) and <a id="ref-for-utf-8-encode②"></a>[UTF-8 encode](https://encoding.spec.whatwg.org/#utf-8-encode) return the same <a id="ref-for-byte-sequence①⑦"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence) for <var>input</var>.

To <a id="ascii-decode"></a><strong>ASCII decode</strong> a <a id="ref-for-byte-sequence①⑧"></a>[byte sequence](https://infra.spec.whatwg.org/#byte-sequence) <var>input</var>, run these steps:

1.  <a id="ref-for-assert⑨"></a>[Assert](https://infra.spec.whatwg.org/#assert): all bytes in <var>input</var> are <a id="ref-for-ascii-byte①"></a>[ASCII bytes](https://infra.spec.whatwg.org/#ascii-byte).

    Note: This precondition ensures that <a id="ref-for-isomorphic-decode①"></a>[isomorphic decode](https://infra.spec.whatwg.org/#isomorphic-decode) and <a id="ref-for-utf-8-decode②"></a>[UTF-8 decode](https://encoding.spec.whatwg.org/#utf-8-decode) return the same <a id="ref-for-string⑤⓪"></a>[string](#string) for this input.

2.  Return the <a id="ref-for-isomorphic-decode②"></a>[isomorphic decoding](https://infra.spec.whatwg.org/#isomorphic-decode) of <var>input</var>.

------------------------------------------------------------------------

To <a id="strip-newlines"></a><strong>strip newlines</strong> from a <a id="ref-for-string⑤①"></a>[string](#string), remove any U+000A LF and U+000D CR <a id="ref-for-code-point③⑧"></a>[code points](#code-point) from the <a id="ref-for-string⑤②"></a>[string](#string).

To <a id="normalize-newlines"></a><strong>normalize newlines</strong> in a <a id="ref-for-string⑤③"></a>[string](#string), replace every U+000D CR U+000A LF <a id="ref-for-code-point③⑨"></a>[code point](#code-point) pair with a single U+000A LF <a id="ref-for-code-point④⓪"></a>[code point](#code-point), and then replace every remaining U+000D CR <a id="ref-for-code-point④①"></a>[code point](#code-point) with a U+000A LF <a id="ref-for-code-point④②"></a>[code point](#code-point).

To <a id="strip-leading-and-trailing-ascii-whitespace"></a><strong>strip leading and trailing ASCII whitespace</strong> from a <a id="ref-for-string⑤④"></a>[string](#string), remove all <a id="ref-for-ascii-whitespace①"></a>[ASCII whitespace](#ascii-whitespace) that are at the start or the end of the <a id="ref-for-string⑤⑤"></a>[string](#string).

To <a id="strip-and-collapse-ascii-whitespace"></a><strong>strip and collapse ASCII whitespace</strong> in a <a id="ref-for-string⑤⑥"></a>[string](#string), replace any sequence of one or more consecutive <a id="ref-for-code-point④③"></a>[code points](#code-point) that are <a id="ref-for-ascii-whitespace②"></a>[ASCII whitespace](#ascii-whitespace) in the <a id="ref-for-string⑤⑦"></a>[string](#string) with a single U+0020 SPACE <a id="ref-for-code-point④④"></a>[code point](#code-point), and then remove any leading and trailing <a id="ref-for-ascii-whitespace③"></a>[ASCII whitespace](#ascii-whitespace) from that string.

------------------------------------------------------------------------

To <a id="collect-a-sequence-of-code-points"></a><strong>collect a sequence of <a id="ref-for-code-point④⑤"></a>[code points](#code-point)</strong> meeting a condition <var>condition</var> from a <a id="ref-for-string⑤⑧"></a>[string](#string) <var>input</var>, given a <a id="string-position-variable"></a><strong>position variable</strong> <var>position</var> tracking the position of the calling algorithm within <var>input</var>:

1.  Let <var>result</var> be the empty <a id="ref-for-string⑤⑨"></a>[string](#string).

2.  While <var>position</var> doesn’t point past the end of <var>input</var> and the <a id="ref-for-code-point④⑥"></a>[code point](#code-point) at <var>position</var> within <var>input</var> meets the condition <var>condition</var>:

    1.  Append that <a id="ref-for-code-point④⑦"></a>[code point](#code-point) to the end of <var>result</var>.

    2.  Advance <var>position</var> by 1.

3.  Return <var>result</var>.

<strong>Note:</strong>

In addition to returning the collected <a id="ref-for-code-point④⑧"></a>[code points](#code-point), this algorithm updates the <a id="ref-for-string-position-variable"></a>[position variable](#string-position-variable) in the calling algorithm.

To <a id="skip-ascii-whitespace"></a><strong>skip ASCII whitespace</strong> within a <a id="ref-for-string⑥⓪"></a>[string](#string) <var>input</var> given a <a id="ref-for-string-position-variable①"></a>[position variable](#string-position-variable) <var>position</var>, <a id="ref-for-collect-a-sequence-of-code-points"></a>[collect a sequence of code points](#collect-a-sequence-of-code-points) that are <a id="ref-for-ascii-whitespace④"></a>[ASCII whitespace](#ascii-whitespace) from <var>input</var> given <var>position</var>. The collected <a id="ref-for-code-point④⑨"></a>[code points](#code-point) are not used, but <var>position</var> is still updated.

------------------------------------------------------------------------

To <a id="strictly-split"></a><strong>strictly split a <a id="ref-for-string⑥①"></a>[string](#string)</strong> <var>input</var> on a particular delimiter <a id="ref-for-code-point⑤⓪"></a>[code point](#code-point) <var>delimiter</var>:

1.  Let <var>position</var> be a <a id="ref-for-string-position-variable②"></a>[position variable](#string-position-variable) for <var>input</var>, initially pointing at the start of <var>input</var>.

2.  Let <var>tokens</var> be a <a id="ref-for-list⑨"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-string⑥②"></a>[strings](#string), initially empty.

3.  Let <var>token</var> be the result of <a id="ref-for-collect-a-sequence-of-code-points①"></a>[collecting a sequence of code points](#collect-a-sequence-of-code-points) that are not equal to <var>delimiter</var> from <var>input</var>, given <var>position</var>.

4.  <a id="ref-for-list-append①⓪"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>token</var> to <var>tokens</var>.

5.  While <var>position</var> is not past the end of <var>input</var>:

    1.  <a id="ref-for-assert①⓪"></a>[Assert](https://infra.spec.whatwg.org/#assert): the <a id="ref-for-code-point⑤①"></a>[code point](#code-point) at <var>position</var> within <var>input</var> is <var>delimiter</var>.

    2.  Advance <var>position</var> by 1.

    3.  Let <var>token</var> be the result of <a id="ref-for-collect-a-sequence-of-code-points②"></a>[collecting a sequence of code points](#collect-a-sequence-of-code-points) that are not equal to <var>delimiter</var> from <var>input</var>, given <var>position</var>.

    4.  <a id="ref-for-list-append①①"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>token</var> to <var>tokens</var>.

6.  Return <var>tokens</var>.

<strong>Note:</strong>

This algorithm is a "strict" split, as opposed to the commonly-used variants <a id="ref-for-split-on-ascii-whitespace"></a>[for ASCII whitespace](#split-on-ascii-whitespace) and <a id="ref-for-split-on-commas"></a>[for commas](#split-on-commas) below, which are both more lenient in various ways involving interspersed <a id="ref-for-ascii-whitespace⑤"></a>[ASCII whitespace](#ascii-whitespace).

To <a id="split-on-ascii-whitespace"></a><strong>split a <a id="ref-for-string⑥③"></a>[string](#string) <var>input</var> on ASCII whitespace</strong>:

1.  Let <var>position</var> be a <a id="ref-for-string-position-variable③"></a>[position variable](#string-position-variable) for <var>input</var>, initially pointing at the start of <var>input</var>.

2.  Let <var>tokens</var> be a <a id="ref-for-list①⓪"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-string⑥④"></a>[strings](#string), initially empty.

3.  <a id="ref-for-skip-ascii-whitespace"></a>[Skip ASCII whitespace](#skip-ascii-whitespace) within <var>input</var> given <var>position</var>.

4.  While <var>position</var> is not past the end of <var>input</var>:

    1.  Let <var>token</var> be the result of <a id="ref-for-collect-a-sequence-of-code-points③"></a>[collecting a sequence of code points](#collect-a-sequence-of-code-points) that are not <a id="ref-for-ascii-whitespace⑥"></a>[ASCII whitespace](#ascii-whitespace) from <var>input</var>, given <var>position</var>.

    2.  <a id="ref-for-list-append①②"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>token</var> to <var>tokens</var>.

    3.  <a id="ref-for-skip-ascii-whitespace①"></a>[Skip ASCII whitespace](#skip-ascii-whitespace) within <var>input</var> given <var>position</var>.

5.  Return <var>tokens</var>.

To <a id="split-on-commas"></a><strong>split a <a id="ref-for-string⑥⑤"></a>[string](#string) <var>input</var> on commas</strong>:

1.  Let <var>position</var> be a <a id="ref-for-string-position-variable④"></a>[position variable](#string-position-variable) for <var>input</var>, initially pointing at the start of <var>input</var>.

2.  Let <var>tokens</var> be a <a id="ref-for-list①①"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-string⑥⑥"></a>[strings](#string), initially empty.

3.  While <var>position</var> is not past the end of <var>input</var>:

    1.  Let <var>token</var> be the result of <a id="ref-for-collect-a-sequence-of-code-points④"></a>[collecting a sequence of code points](#collect-a-sequence-of-code-points) that are not U+002C (,) from <var>input</var>, given <var>position</var>.

        <strong>Note:</strong>

        <var>token</var> might be the empty string.

    2.  <a id="ref-for-strip-leading-and-trailing-ascii-whitespace"></a>[Strip leading and trailing ASCII whitespace](#strip-leading-and-trailing-ascii-whitespace) from <var>token</var>.

    3.  <a id="ref-for-list-append①③"></a>[Append](https://infra.spec.whatwg.org/#list-append) <var>token</var> to <var>tokens</var>.

    4.  If <var>position</var> is not past the end of <var>input</var>:

        1.  <a id="ref-for-assert①①"></a>[Assert](https://infra.spec.whatwg.org/#assert): the <a id="ref-for-code-point⑤②"></a>[code point](#code-point) at <var>position</var> within <var>input</var> is U+002C (,).

        2.  Advance <var>position</var> by 1.

4.  Return <var>tokens</var>.

To <a id="string-concatenate"></a><strong>concatenate</strong> a <a id="ref-for-list①②"></a>[list](https://infra.spec.whatwg.org/#list) of <a id="ref-for-string⑥⑦"></a>[strings](#string) <var>list</var>, using an optional separator string <var>separator</var>, run these steps:

1.  If <var>list</var> <a id="ref-for-list-is-empty"></a>[is empty](https://infra.spec.whatwg.org/#list-is-empty), then return the empty string.

2.  If <var>separator</var> is not given, then set <var>separator</var> to the empty string.

3.  Return a <a id="ref-for-string⑥⑧"></a>[string](#string) whose contents are <var>list</var>’s <a id="ref-for-list-item"></a>[items](https://infra.spec.whatwg.org/#list-item), in order, separated from each other by <var>separator</var>.

<a id="example-string-concatenate"></a>

<strong>Example:</strong>

[](#example-string-concatenate)To serialize a set <var>set</var>, return the <a id="ref-for-string-concatenate"></a>[concatenation](#string-concatenate) of <var>set</var> using U+0020 SPACE.

## <a id="ipr"></a>Intellectual property rights[](#ipr)

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). This work is licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/). To the extent portions of it are incorporated into source code, such portions in the source code are licensed under the [BSD 3-Clause License](https://opensource.org/licenses/BSD-3-Clause) instead.

This is the Living Standard. Those interested in the patent-review version should view the [Living Standard Review Draft](https://infra.spec.whatwg.org/review-drafts/2026-07/).
