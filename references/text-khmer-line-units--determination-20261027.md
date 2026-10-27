# Khmer line-breaking units

Surgeist's Khmer line-breaking profile preserves the complete unit around a
post-vowel COENG sequence. It derives line units from the retained Unicode 17
extended grapheme clusters (EGCs); it does not change editing, cursor or
selection EGCs. The [adopted determination](https://github.com/bj-data-eng/surgeist/issues/1504#issuecomment-6440213742)
owns this bounded contribution.

## Rule and operation boundary

Within one connected semantic component, suppress an EGC boundary when its
immediate preceding suffix is U+17D2 COENG followed by zero or more characters
whose retained Indic_Conjunct_Break property is Extend, and the following
character has Khmer script and Indic_Conjunct_Break Consonant. Merge the complete
adjacent EGC ranges, including their marks; repeated matches merge transitively.
Do not skip an arbitrary mark or control to find a COENG witness.

This is the Khmer restriction of revised GB9c. Semantic barriers bound the
suffix; connected source-origin, inline-style, empty and out-of-flow seams do
not create new barriers. Ordinary and missing-capability opportunities,
BreakAll, line-break Anywhere, emergency overflow wrapping and their intrinsic
consumers share the resulting boundary protection. Explicit and required
controls retain their own genuine occurrence and authority.

For UTF-16 source `[1780,17B6,17D2,1781]`, the line unit is `[0,4)`, so no
automatic line operation cuts at3. Appending U+1782 supplies a separate unit
`[4,5)`; eligible missing-capability fallback may cut at4. The default retained
EGCs still split the original sequence before the final consonant. This is an
intentional distinction between line-unit and editing-unit operations.

Line-unit protection does not make a sequence a detected word, a valid
orthographic syllable or an eligible lexical letter unit. The actual
missing-capability witness and Appendix-E classification still govern Khmer
fallback. Manual suppresses its lexical cause; actual wrapping permission,
GL/WJ/ZWJ relations, original source correspondence and independent controls
retain their existing owners. The [lexical-fallback determination](text-lexical-fallback--determination-20261025.md)
supplies the Normal/KeepAll and missing-capability distinction.

## Source identities and edition limits

- Foundation: [UAX29 revision 47, Unicode 17](https://www.unicode.org/reports/tr29/tr29-47.html),
  represented by the existing [metadata record](uax29-revision-47-extended-grapheme-clusters--tr29-47.html--130d4e0d3f9d.md)
  and [licensed implementation companion](unicode-17-grapheme-implementation-companion.md).
- Selected later rule: [UAX29 revision 49, Unicode 18, GB9c](https://www.unicode.org/reports/tr29/tr29-49.html#GB9c),
  dated 1 September 2026. Its [modifications](https://www.unicode.org/reports/tr29/tr29-49.html#Modifications)
  identify the change as UTC 187-C47. Complete privately retained HTML SHA-256:
  `60fb49ee640a154d9e40468306c6b283edcef73be3af5ede778107b4787bdec8`.
- Property cross-check: [DerivedCoreProperties 18.0.0](https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt),
  dated 7 August 2026. COENG remains Linker and U+1780..U+17B3 remain Consonant;
  AA need not become Extend. Complete private reference SHA-256:
  `09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9`.
- CSS operation basis: the [pinned Text4 character-unit clauses](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#characters)
  and [missing-analysis fallback](css-text-4--WD-css-text-4-20260814--106c5cd086ab.md#fallback-breaking).

The later report and property source were checked 27 October 2026. This record is
handwritten policy and bibliographic metadata; it reproduces no report paragraphs
or tables. Report-specific redistribution restrictions still apply. No Unicode 18
property file is imported into production by this profile: runtime properties
remain those of the retained ICU provider. This adoption does not upgrade the
global Unicode baseline, other scripts, word segmentation, editing, or shaping.
Harfrust's experimental Khmer shaping grammar is not the line-unit authority.
