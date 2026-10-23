# Hyphenation resources

Text uses the exact registry dependency Hypher 0.1.8 with default features off
and only `alloc` and `english` enabled. It has no transitive dependencies. This
is the bounded en-US resource operation in the
[hyphenation contract](../../../docs/text-layout-contract.md#automatic-resources-word-custody-and-initial-limits),
not general English or multilingual support.

## Source and attribution

The retained [American-English source](../licenses/hypher/hyph-en-us.tex) is
version 2005-05-30 from hyph-utf8 revision
`d81cc506416bffef2a75ee2dc969d4b41bc36613`, path
`patterns/hyph-en-us.tex`. Its SHA256 is
`f4ffcd96c5cbc886bdad23f95dcae8edc3cd3620eae62f7946eceda97c4e68f8`.
The provider's packed data remains unchanged. Text retains the source for
provenance and independent finite qualification, without another runtime copy of
that packed data. Copyright, permission and provider licenses are linked from
[the crate notice](../NOTICE.md#hypher-018).

The private [resource adapter](../src/engine/analysis/hyphenation/en_us.rs)
combines the provider's pattern observations with the source's fourteen exact
whole-word exceptions. `present`, `presents`, `project` and `projects` explicitly
forbid cuts; a zero-cut override differs from no exception. Exception matching
ignores ASCII case only. Text separately adopts cuts 3 and 4 for the exact word
`democrat`, identifying this as `TextCorrection`. It changes neither the retained
source nor derivative words. Other cuts retain `Pattern` or `SourceException`
provenance.

## Source ownership

A resource result supplies checked UTF-8 offsets in its actual borrowed query;
it establishes no lexical word, style, wrapping, UTF-16 or selection authority.
Text maps that query to the retained processed word. Authentic SHY alone is
omitted; other formatting controls and marks remain exact. Isolated UTF-16 units
do not become replacement characters in an English query. Source gaps and real
EGC boundaries remain part of admission.

The same complete-word count owner serves manual and automatic native
admission. Every genuine language contribution must be exactly en-US, compared
without ASCII case distinctions. The logical-preceding text contribution owns
each cut's style and indication. Ancestor wrapping permission is separate.
Unavailable resources retain manual behavior and an explicit word observation.
The raw standalone operation keeps its distinct unconstrained count policy.

Prepared word and cut observations retain genuine source pieces and resource
provenance. They cannot be supplied by callers as a fabricated SHY or control.
Selected automatic indication text remains computed presentation; plaintext and
authored source are unchanged. Actual chooser priority and fit remain consuming
operations, described with the [selected-line protocol](composed-lines.md).
