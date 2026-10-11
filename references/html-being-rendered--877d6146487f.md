# HTML being-rendered definition and notes

Bounded Markdown conversion of [pinned HTML rendering introduction](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/rendering.html#being-rendered), revision `877d6146487f15b15bc8dd0e3c625063a37b456a`. Original complete source SHA-256 `877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f`; 14930614 bytes. Selected original lines 99611–99622: the definition and two notes, not the whole rendering chapter. Captured from the already retained original on 11 October 2026.

Attribution: WHATWG HTML contributors; specification prose under CC BY 4.0 as recorded in the [existing WHATWG notice](../licenses/whatwg/NOTICE.md#pinned-html-language-snapshot--2026-10-11). This excerpt converts markup to Markdown prose and folds layout whitespace; no behavior or words are repaired. Original inline link identities remain available at the immutable source above. No tables/code blocks occur in this selection.

## Definition and notes

An element is being rendered if it has any associated CSS layout boxes, SVG layout boxes, or some equivalent in other styling languages.

Just being off-screen does not mean the element is not being rendered. The presence of the hidden attribute normally means the element is not being rendered, though this might be overridden by the style sheets.

The fully active state does not affect whether an element is being rendered or not. Even if a document is not fully active and not shown at all to the user, elements within it can still qualify as "being rendered".

## Selected scope

For #1134/#543, this defines the required supplied participation fact: associated CSS layout boxes, SVG layout boxes, or an equivalent in another styling language. It is not viewport intersection, painted pixel visibility, an element-name test, or an inference from hidden alone. CSS-box existence is a separate boolean used by the list-owner ancestor search. Existing retained HTML grouping definitions own that search and ordinal numbering.
