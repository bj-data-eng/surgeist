# Unicode 17 extended grapheme implementation companion

## Source provenance

Classification: informative implementation rule-data companion

Unicode version: 17.0.0

Source release tag: [final-17.0-20250910](https://github.com/unicode-org/unicodetools/tree/final-17.0-20250910)

Immutable source commit: ecd26be2e82be37609090942c14e518e57e905d8

Source file: [SegmenterDefault.txt](https://github.com/unicode-org/unicodetools/blob/ecd26be2e82be37609090942c14e518e57e905d8/unicodetools/src/main/resources/org/unicode/tools/SegmenterDefault.txt)

Source snapshot SHA-256: 591c9e49f9bd744e795689fed2cb54904a1d446ec28535c321717f01dbc44df8

Source Git blob SHA-1: c108c0144d75d7e149091fab9596321b301dd2df

Included excerpt: the complete @GraphemeClusterBreak section, source lines 1–54, ending before @LineBreak

Included excerpt SHA-256: df5cd5a1bbda23d91a66413d86420d68651b51ce07c3d6a2e3c4de0651909756

License: [Unicode License v3 (Unicode-3.0)](https://github.com/unicode-org/unicodetools/blob/ecd26be2e82be37609090942c14e518e57e905d8/LICENSE), reproduced in full below

License snapshot SHA-256: fe5c62b543e287981db198f2acfa0ca732d12591a1024536dc8fd85dacd77104

## Status and scope

This is a concrete implementation reference built from Unicode's licensed segmentation rule data. The rule-data excerpt below is verbatim, including original comments and inactive double-hash lines. No rule text has been added to the excerpt.

The normative reference remains [Unicode Standard Annex #29, revision 47, Unicode 17.0.0](https://www.unicode.org/reports/tr29/tr29-47.html). This companion does not reproduce the report or replace that authority. Consult the linked report for its full conformance requirements, explanatory material, property derivations, implementation guidance, and other segmentation algorithms.

The data source uses internal variables and regular-expression-like notation. Its grapheme rule numbers 9.1, 9.2, and 9.3 correspond to GB9a, GB9b, and GB9c. Lines starting with # are comments; double-hash lines retain inactive alternatives from the source.

Important completeness caveat: this final-17 snapshot leaves start-of-text, end-of-text, and the default fallback implicit in the segmenter engine. The [same-commit Segmenter.java](https://github.com/unicode-org/unicodetools/blob/ecd26be2e82be37609090942c14e518e57e905d8/unicodetools/src/main/java/org/unicode/tools/Segmenter.java#L132-L136) assigns the artificial numbers 0.2, 0.3, and 999, and its [builder](https://github.com/unicode-org/unicodetools/blob/ecd26be2e82be37609090942c14e518e57e905d8/unicodetools/src/main/java/org/unicode/tools/Segmenter.java#L645-L651) adds the corresponding display rules. These supply the start/end behavior associated with GB1 and GB2 and the fallback associated with GB999. The excerpt is therefore rule data interpreted by that engine, not a self-contained executable algorithm.

## Exact grapheme rule-data excerpt

```text
@GraphemeClusterBreak
## double ## at the start of a line doesn't show up

# VARIABLES

$CR=\p{Grapheme_Cluster_Break=CR}
$LF=\p{Grapheme_Cluster_Break=LF}
$Control=\p{Grapheme_Cluster_Break=Control}
$Extend=\p{Grapheme_Cluster_Break=Extend}
$ZWJ=\p{Grapheme_Cluster_Break=ZWJ}
$RI=\p{Grapheme_Cluster_Break=Regional_Indicator}
$Prepend=\p{Grapheme_Cluster_Break=Prepend}
$SpacingMark=\p{Grapheme_Cluster_Break=SpacingMark}
$L=\p{Grapheme_Cluster_Break=L}
$V=\p{Grapheme_Cluster_Break=V}
$T=\p{Grapheme_Cluster_Break=T}
$LV=\p{Grapheme_Cluster_Break=LV}
$LVT=\p{Grapheme_Cluster_Break=LVT}
$ConjunctLinker=\p{Indic_Conjunct_Break=Linker}
$LinkingConsonant=\p{Indic_Conjunct_Break=Consonant}
##	$E_Base=\p{Grapheme_Cluster_Break=E_Base}
##	$E_Modifier=\p{Grapheme_Cluster_Break=E_Modifier}
$ExtPict=\p{Extended_Pictographic=True}
$ConjunctExtender=[\p{Indic_Conjunct_Break=Linker}\p{Indic_Conjunct_Break=Extend}]
##	$EBG=\p{Grapheme_Cluster_Break=E_Base_GAZ}
##	$Glue_After_Zwj=\p{Grapheme_Cluster_Break=Glue_After_Zwj}
$XX = \p{Grapheme_Cluster_Break=Other}

# RULES

# Break at the start and end of text, unless the text is empty.
# Do not break between a CR and LF. Otherwise, break before and after controls.
3) $CR  	×  	$LF
4) ( $Control | $CR | $LF ) 	÷
5) ÷ 	( $Control | $CR | $LF )
# Do not break Hangul syllable sequences.
6) $L 	× 	( $L | $V | $LV | $LVT )
7) ( $LV | $V ) 	× 	( $V | $T )
8) ( $LVT | $T)    ×  $T
## Do not break before extending characters or ZWJ.
##	9) × 	($Extend | $ZWJ | $ConjunctLinker)
9) × 	($Extend | $ZWJ)
# Only for extended grapheme clusters: Do not break before SpacingMarks, or after Prepend characters.
9.1) × 	$SpacingMark
9.2) $Prepend  ×
9.3) $LinkingConsonant $ConjunctExtender* $ConjunctLinker $ConjunctExtender*  × $LinkingConsonant
## Do not break within emoji modifier sequences or emoji zwj sequences.
##	10) $E_Base $Extend* × $E_Modifier
11) $ExtPict $Extend* $ZWJ × $ExtPict
# Do not break within emoji flag sequences. That is, do not break between regional indicator (RI) symbols if there is an odd number of RI characters before the break point.
12) ^ ($RI $RI)* $RI × $RI
13) [^$RI] ($RI $RI)* $RI × $RI
# Otherwise, break everywhere.

```

## Same-version properties, tests, and readable examples

Use Unicode 17.0.0 inputs when working with this snapshot:

- [GraphemeBreakProperty-17.0.0.txt](https://www.unicode.org/Public/17.0.0/ucd/auxiliary/GraphemeBreakProperty.txt): Grapheme_Cluster_Break assignments
- [DerivedCoreProperties-17.0.0.txt](https://www.unicode.org/Public/17.0.0/ucd/DerivedCoreProperties.txt): Indic_Conjunct_Break assignments
- [emoji-data.txt, Emoji 17.0](https://www.unicode.org/Public/17.0.0/ucd/emoji/emoji-data.txt): Extended_Pictographic assignments
- [GraphemeBreakTest-17.0.0.txt](https://www.unicode.org/Public/17.0.0/ucd/auxiliary/GraphemeBreakTest.txt): official default grapheme boundary test samples
- [Grapheme Break Chart, Unicode 17.0.0](https://www.unicode.org/Public/17.0.0/ucd/auxiliary/GraphemeBreakTest.html#rules): readable rule table, pairwise chart, and sample strings; explicitly informative, with mechanically modified rules

The test samples are useful checks of behavior. They do not replace the full normative specification or establish correctness for every possible input.

## Copyright and permission notice

The following is the complete LICENSE file from the immutable source commit cited above, with its original copyright notice, permission terms, disclaimer, and SPDX identifier preserved.

```text
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 2001-2024 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.

SPDX-License-Identifier: Unicode-3.0
```
