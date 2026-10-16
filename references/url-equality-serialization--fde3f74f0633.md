# URL equality and serialization

Bounded reference for HTML's ancestor-URL comparison. The source algorithms remain
authoritative; this does not adopt a URL parser, a general URL engine, JavaScript
bindings, or a new HTML edition.

## Source and attribution

WHATWG, *URL Standard*, snapshot `fde3f74f063341a28d437216f75735b8b40128f5`
(7 October 2026), inspected 16 October 2026. This is the current URL snapshot at
inspection and precedes the selected HTML877d edition.

- [Immutable rendered snapshot](https://url.spec.whatwg.org/commit-snapshots/fde3f74f063341a28d437216f75735b8b40128f5/).
- [Complete original url.bs](https://github.com/whatwg/url/blob/fde3f74f063341a28d437216f75735b8b40128f5/url.bs):
  165,812 bytes; SHA-256 `7f45ea5b6167f08a098619167254a99a3c8b5fd6514fff4963ede4a7c8e05818`;
  verified Git blob `2aab6786dfb97f1b4ad168a4e35c7691b9b70d28`.
- [Upstream LICENSE](https://github.com/whatwg/url/blob/fde3f74f063341a28d437216f75735b8b40128f5/LICENSE)
  is byte-identical to the retained [WHATWG license](../licenses/whatwg/LICENSE):
  16,315 bytes; SHA-256 `85dc6f5ccb57a6fe8c33d158f9fc8fc7ee5655a5d3db2cdd131c6a3d0f48a864`;
  Git blob `f2dcda46deccefd245749202a88a7837e35c6daa`.

Copyright © WHATWG (Apple, Google, Mozilla, Microsoft). Specification prose is
licensed under Creative Commons Attribution 4.0 International. BSD 3-Clause
applies instead only to portions incorporated into source code, not as an
alternative license for this reference. Original notices, conditions and
disclaimers are retained in the linked complete license. No endorsement is implied.

Changes by Surgeist: bounded offline extraction, Markdown headings and source
links, plus the separately marked reading guide below. The complete selected
Bikeshed source units are retained byte for byte inside inert fenced blocks;
no source markup, notes, conditions or step order is silently rewritten. This
follows the existing bounded HR-Time/UUID reference representation. No renderer,
script or conversion tool was downloaded or executed; there are no tables in
these selected units requiring table repair. The original source remains
available through immutable links.

## Reading guide — explanatory, not replacement source

URL equality serializes both actual URL records with the requested fragment
exclusion and compares the resulting strings exactly. Excluding the fragment
omits only the serializer's final fragment-delimiter/value append. The query
append remains, including an empty query's question mark. The path is neither
trimmed nor reparsed. URL-path equality is a separate operation and is not
selected here.

Parser-produced records protect the fragment boundary through encoding and
delimiter handling. The generic record model is broader: URL paths are ASCII
strings, and the standard expressly recognizes paths produced outside its URL
parser. The serializer appends component contents rather than re-encoding them.
Consequently, a full serialization alone does not establish that its first
literal number sign was appended as the fragment delimiter. An observed opaque
path containing a literal number sign with a null fragment must remain complete
when excluding fragments. The original record/serializer can attest the exact
final fragment-append boundary without a parser or duplicate serialized string.

A percent-encoded number sign remains ordinary serialized text. HTML's
matches-about:blank predicate and native record identity are separate from URL
equality. None of these observations validates arbitrary author input or
establishes missing URL facts.

The implementation boundary and bounded comparison requirements belong to
[#1124](https://github.com/bj-data-eng/surgeist/issues/1124), rather than to this
source reference. HTML-Semantics retains actual externally attested URL records;
this reference does not replace the original provider or establish a missing
parse result, Document association, source revision or origin.

## Exact retained source units

### Percent-encode sets

[Original url.bs lines 469–495](https://github.com/whatwg/url/blob/fde3f74f063341a28d437216f75735b8b40128f5/url.bs#L469-L495).

```text
<p>A <dfn>percent-encode set</dfn> is a <a for=/>set</a> of <a for=/>code points</a>.

<p>The <dfn oldids=simple-encode-set>C0 control percent-encode set</dfn> is a
<a>percent-encode set</a> consisting of <a>C0 controls</a> and all <a>code points</a> greater than
U+007E (~).

<p>The <dfn>fragment percent-encode set</dfn> is a <a>percent-encode set</a> consisting of the
<a>C0 control percent-encode set</a> and U+0020 SPACE, U+0022 ("), U+003C (&lt;), U+003E (>), and
U+0060 (`).

<p>The <dfn>query percent-encode set</dfn> is a <a>percent-encode set</a> consisting of the
<a>C0 control percent-encode set</a> and U+0020 SPACE, U+0022 ("), U+0023 (#), U+003C (&lt;), and
U+003E (>).

<p class=note>The <a>query percent-encode set</a> cannot be defined in terms of the
<a>fragment percent-encode set</a> due to the omission of U+0060 (`).

<p>The <dfn>special-query percent-encode set</dfn> is a <a>percent-encode set</a> consisting of
the <a>query percent-encode set</a> and U+0027 (').

<p>The <dfn oldids=default-encode-set>path percent-encode set</dfn> is a <a>percent-encode set</a>
consisting of the <a>query percent-encode set</a> and U+003F (?), U+005E (^), U+0060 (`),
U+007B ({), and U+007D (}).

<p>The <dfn oldids=userinfo-encode-set>userinfo percent-encode set</dfn> is a
<a>percent-encode set</a> consisting of the <a>path percent-encode set</a> and U+002F (/),
U+003A (:), U+003B (;), U+003D (=), U+0040 (@), U+005B ([) to U+005D (]), inclusive, and U+007C (|).
```

### Forbidden host and domain code points

[Original url.bs lines 807–812](https://github.com/whatwg/url/blob/fde3f74f063341a28d437216f75735b8b40128f5/url.bs#L807-L812).

```text
<p>A <dfn export>forbidden host code point</dfn> is U+0000 NULL, U+0009 TAB, U+000A LF, U+000D CR,
U+0020 SPACE, U+0023 (#), U+002F (/), U+003A (:), U+003C (&lt;), U+003E (>), U+003F (?), U+0040 (@),
U+005B ([), U+005C (\), U+005D (]), U+005E (^), or U+007C (|).

<p>A <dfn export>forbidden domain code point</dfn> is a <a>forbidden host code point</a>,
a <a>C0 control</a>, U+0025 (%), or U+007F DELETE.
```

### Opaque path parsing

[Original url.bs lines 3034–3069](https://github.com/whatwg/url/blob/fde3f74f063341a28d437216f75735b8b40128f5/url.bs#L3034-L3069).

```text
   <dt><dfn export for="basic URL parser" id=cannot-be-a-base-url-path-state>opaque path state</dfn>
   <dd>
    <ol>
     <li><p>If <a>c</a> is U+003F (?), then set <var>url</var>'s <a for=url>query</a> to the empty
     string and <var>state</var> to <a>query state</a>.

     <li><p>Otherwise, if <a>c</a> is U+0023 (#), then set <var>url</var>'s <a for=url>fragment</a>
     to the empty string and <var>state</var> to <a>fragment state</a>.

     <li>
      <p>Otherwise, if <a>c</a> is U+0020 SPACE:

      <ol>
       <li><p><a>Invalid-URL-unit</a> <a>validation error</a>.

       <li><p>If <a>remaining</a> starts with U+003F (?) or U+0023 (#), then append
       "<code>%20</code>" to <var>url</var>'s <a for=url>path</a>.

       <li><p>Otherwise, append U+0020 SPACE to <var>url</var>'s <a for=url>path</a>.
      </ol>

     <li>
      <p>Otherwise, if <a>c</a> is not the <a>EOF code point</a>:

      <ol>
       <li><p>If <a>c</a> is not a <a>URL code point</a> and not U+0025 (%), <a>invalid-URL-unit</a>
       <a>validation error</a>.

       <li><p>If <a>c</a> is U+0025 (%) and <a>remaining</a> does not start with two
       <a>ASCII hex digits</a>, <a>invalid-URL-unit</a> <a>validation error</a>.

       <li><p><a for="code point">UTF-8 percent-encode</a> <a>c</a> using the
       <a>C0 control percent-encode set</a> and append the result to <var>url</var>'s
       <a for=url>path</a>.
      </ol>
    </ol>
```

### URL serialization, path serialization and URL equality

[Original url.bs lines 3168–3271](https://github.com/whatwg/url/blob/fde3f74f063341a28d437216f75735b8b40128f5/url.bs#L3168-L3271).

```text
<h3 id=url-serializing>URL serializing</h3>

<div algorithm="URL serializer">
<p>The <dfn export id=concept-url-serializer lt="URL serializer">URL serializer</dfn> takes a
<a for=/>URL</a> <var>url</var>, with an optional boolean
<dfn export for="URL serializer"><var>exclude fragment</var></dfn> (default false), and then runs
these steps. They return an <a>ASCII string</a>.

<ol>
 <li><p>Let <var>output</var> be <var>url</var>'s <a for=url>scheme</a> and U+003A (:) concatenated.

 <li>
  <p>If <var>url</var>'s <a for=url>host</a> is non-null:

  <ol>
   <li><p>Append "<code>//</code>" to <var>output</var>.

   <li>
    <p>If <var>url</var> <a>includes credentials</a>, then:

    <ol>
     <li><p>Append <var>url</var>'s <a for=url>username</a> to
     <var>output</var>.

     <li><p>If <var>url</var>'s <a for=url>password</a> is not the empty string, then append
     U+003A (:), followed by <var>url</var>'s <a for=url>password</a>, to <var>output</var>.

     <li><p>Append U+0040 (@) to <var>output</var>.
    </ol>

   <li><p>Append <var>url</var>'s <a for=url>host</a>,
   <a lt="host serializer">serialized</a>, to <var>output</var>.

   <li><p>If <var>url</var>'s <a for=url>port</a> is non-null, append U+003A (:) followed by
   <var>url</var>'s <a for=url>port</a>, <a lt="serialize an integer">serialized</a>, to
   <var>output</var>.
  </ol>

 <li>
  <p>If <var>url</var>'s <a for=url>host</a> is null, <var>url</var> does not have an
  <a for=url>opaque path</a>, <var>url</var>'s <a for=url>path</a>'s <a for=list>size</a> is greater
  than 1, and <var>url</var>'s <a for=url>path</a>[0] is the empty string, then append U+002F (/)
  followed by U+002E (.) to <var>output</var>.

  <p class=note>This prevents <code>web+demo:/.//not-a-host/</code> or
  <code>web+demo:/path/..//not-a-host/</code>, when <a lt="URL parser">parsed</a> and then
  <a lt="URL serializer">serialized</a>, from ending up as <code>web+demo://not-a-host/</code> (they
  end up as <code>web+demo:/.//not-a-host/</code>).

 <li><p>Append the result of <a>URL path serializing</a> <var>url</var> to <var>output</var>.

 <li><p>If <var>url</var>'s <a for=url>query</a> is non-null, append
 U+003F (?), followed by <var>url</var>'s <a for=url>query</a>, to
 <var>output</var>.

 <li><p>If <var>exclude fragment</var> is false and <var>url</var>'s <a for=url>fragment</a> is
 non-null, then append U+0023 (#), followed by <var>url</var>'s <a for=url>fragment</a>, to
 <var>output</var>.

 <li><p>Return <var>output</var>.
</ol>
</div>

<div algorithm>
<p>The <dfn export lt="URL path serializer|URL path serializing">URL path serializer</dfn> takes a
<a for=/>URL</a> or <a for=/>URL path</a> <var>urlOrPath</var> and then runs these steps. They
return an <a>ASCII string</a>.

<ol>
 <li><p>Let <var>path</var> be <var>urlOrPath</var>'s <a for=url>path</a> if <var>urlOrPath</var> is
 a <a for=/>URL</a>; otherwise <var>urlOrPath</var>.

 <li><p>If <var>path</var> is a <a for=/>URL path segment</a>, then return <var>path</var>.

 <li><p>Let <var>output</var> be the empty string.

 <li><p><a for=list>For each</a> <var>segment</var> of <var>path</var>: append U+002F (/) followed
 by <var>segment</var> to <var>output</var>.

 <li><p>Return <var>output</var>.
</ol>
</div>


<h3 id=url-equivalence>URL equivalence</h3>

<div algorithm="equal">
<p>To determine whether a <a for=/>URL</a> <var>A</var>
<dfn export for=url id=concept-url-equals lt=equal>equals</dfn> <a for=/>URL</a> <var>B</var>, with
an optional boolean <dfn export for="url/equals"><var>exclude fragments</var></dfn> (default false),
run these steps:

<ol>
 <li><p>Let <var>serializedA</var> be the result of <a lt="URL serializer">serializing</a>
 <var>A</var>, with <a for="URL serializer"><i>exclude fragment</i></a> set to
 <var>exclude fragments</var>.

 <li><p>Let <var>serializedB</var> be the result of <a lt="URL serializer">serializing</a>
 <var>B</var>, with <a for="URL serializer"><i>exclude fragment</i></a> set to
 <var>exclude fragments</var>.

 <li><p>Return true if <var>serializedA</var> is <var>serializedB</var>; otherwise false.
</ol>
</div>
```

### URL path representation and non-parser producers

[Original url.bs lines 1952–1963](https://github.com/whatwg/url/blob/fde3f74f063341a28d437216f75735b8b40128f5/url.bs#L1952-L1963).

```text
<p>A <dfn export>URL path</dfn> is either a <a>URL path segment</a> or a <a for=/>list</a> of zero
or more <a>URL path segments</a>. In the latter case the <a>URL path segments</a> never contain
U+002F (/).

<p>A <dfn export>URL path segment</dfn> is an <a for=/>ASCII string</a>. It commonly refers to a
directory or a file, but has no predefined meaning.

<p class=note>A <a for=/>URL path</a> that is a <a>URL path segment</a> is
<a for=url lt="opaque path">opaque</a> and can contain U+002F (/), as with
<code>web+demo:a/b</code>. The <a>URL parser</a> never produces one that starts with U+002F (/), but
<cite>URL Pattern</cite> does while canonicalizing patterns, so that is not an invariant.
[[URLPATTERN]]
```
