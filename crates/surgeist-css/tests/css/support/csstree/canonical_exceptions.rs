//! Explicit per-case differences from upstream CSSTree canonical output.
//!
//! Every entry is an upstream-parsed case whose normalized Surgeist
//! serialization differs from CSSTree's `generate` expectation. Each carries a
//! kind, a one-line reason and the expected Surgeist form, rendered as
//! [`super::normalize`] renders it. Where that rendering would tokenize
//! differently, as `1%+1.5em` becomes a signed number after `1%`, the pin keeps
//! the whitespace or comment that separates the tokens. Deliberate reasons cite the selected
//! specification edition, and each pinned form is the result its reason
//! requires, checked against that reason rather than taken on trust from the
//! parser. The comparison requires the actual serialization to equal the pinned
//! form token by token, so a different wrong serialization of an excepted case
//! fails. Entries are sorted by case ID, and an entry that starts matching
//! upstream fails the comparison so the list cannot go stale.

use super::{CanonicalException, CanonicalExceptionKind};

const fn deliberate(
    id: &'static str,
    surgeist: &'static str,
    reason: &'static str,
) -> CanonicalException {
    CanonicalException {
        id,
        kind: CanonicalExceptionKind::Deliberate,
        reason,
        surgeist,
    }
}

// Recovery: CSSTree retains invalid input as raw nodes; Surgeist discards it.
const UNKNOWN_PROPERTY: &str = "a declaration of an unknown property is ignored; CSS 2 https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html#parsing-errors";
const UNKNOWN_PROPERTY_AND_AT_RULE: &str = "unknown properties and unknown at-rules are ignored; CSS 2 https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html#parsing-errors";
const UNKNOWN_AT_RULE: &str = "an at-rule with an unknown at-keyword is ignored; CSS 2 https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html#parsing-errors";
const UNKNOWN_DESCRIPTOR: &str = "an unknown @font-face descriptor is invalid and ignored; Fonts 4 https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-rule";
const INVALID_PROPERTY_NAME: &str = "a declaration name must be an identifier; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-declaration";
const MISSING_COLON: &str = "a declaration without a colon is a parse error and returns nothing; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-declaration";
const STYLE_ATTRIBUTE_RULES: &str = "a style attribute holds declarations only, so nested rules are not retained; https://www.w3.org/TR/2013/REC-css-style-attr-20131107/#syntax";
const LEGACY_FILTER: &str = "proprietary progid filters are outside the filter grammar and are dropped; https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#FilterFunctions";
const NEST_AT_RULE: &str = "the superseded @nest rule is an unknown at-rule under Nesting 1; https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting";
const STARTING_STYLE: &str = "Transitions 2 is not a selected edition (references/SOURCE-EDITIONS.md), so @starting-style is an unknown at-rule and is ignored; CSS 2 https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html#parsing-errors";
const NESTED_SEMICOLON: &str = "a `;` between nested rules is not a block item; Nesting 1 https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting";
const TOP_LEVEL_GROUP_DECLARATIONS: &str = "declarations in a top-level @media rule list parse as invalid qualified rules; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-list-of-rules";
const RULE_LIST_SEMICOLON: &str = "a `;` in a rule list starts an invalid qualified-rule prelude; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-qualified-rule";
const CDO_CDC: &str = "top-level CDO and CDC tokens are discarded; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-list-of-rules";
const EOF_QUALIFIED_RULE: &str = "a qualified rule ending at EOF without a block returns nothing; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-qualified-rule";
const UNMATCHED_CLOSER: &str = "a custom-property value with an unmatched closer is invalid; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#any-value";
const BAD_STRING: &str = "a newline ends a bad string, which invalidates its declaration; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#any-value";
const DEEP_COMBINATOR: &str = "/deep/ is not a combinator, so the selector is invalid; Selectors 4 <combinator> = '>' | '+' | '~' | [ '|' '|' ] https://www.w3.org/TR/2026/WD-selectors-4-20260122/#grammar";
const INVALID_SELECTOR: &str = "a rule with an invalid selector is dropped; Selectors 4 https://www.w3.org/TR/2026/WD-selectors-4-20260122/#invalid";
const UNDECLARED_NAMESPACE: &str = "an undeclared namespace prefix invalidates the unforgiving selector; Selectors 4 https://www.w3.org/TR/2026/WD-selectors-4-20260122/#attrnmsp";
const FORGIVING_RELATIVE: &str = "a forgiving selector list drops relative members; Selectors 4 https://www.w3.org/TR/2026/WD-selectors-4-20260122/#forgiving-selector";
const MEDIA_MALFORMED: &str = "a malformed media query becomes `not all`; Media Queries 4 https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/#error-handling";
const CUSTOM_MEDIA_COLON: &str = "a dashed custom-media name permits only boolean use; Media Queries 5 https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/#custom-mq";

// Canonical serialization: equal values with a different specified spelling.
const ATTRIBUTE_STRING: &str = "attribute values serialize as strings, with ` i` before a flag; CSSOM https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-simple-selector";
const LEGACY_PSEUDO_ELEMENT: &str = "legacy single-colon pseudo-elements serialize as lowercase `::name`; CSSOM https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-simple-selector";
const PSEUDO_CASE: &str = "pseudo-class and pseudo-element names are ASCII case-insensitive, Selectors 4 https://www.w3.org/TR/2026/WD-selectors-4-20260122/#case-sensitive, and serialize as their defined lowercase name; CSSOM https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-simple-selector";
const LANG_STRING: &str = "ident and string language ranges are equivalent, and :lang() serializes each argument as a string; Selectors 4 https://www.w3.org/TR/2026/WD-selectors-4-20260122/#lang-pseudo and CSSOM editor capture 2026-10-09 serialize a simple selector (references/cssom-1--editor-capture-20261009--d42e145ec395.md)";
const ANB: &str = "`even` and `odd` serialize as `2n` and `2n+1`; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#serialize-an-anb-value";
const AT_RULE_CASE: &str = "at-rule names are ASCII case-insensitive and serialize lowercase; CSSOM https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-rule";
const MEDIA_ALL: &str = "a media query omits a redundant `all and`; CSSOM https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-media-query";
const CONTAINER_CONDITION_TRIVIA: &str = "conditionText returns the specified condition, where token stream simplifications such as optional whitespace are allowed; Contain 3 CSSContainerRule (references/css-contain-3--WD-css-contain-3-20220818--7e6b410172bf.md)";
const INVALID_NESTED_RULE: &str = "a block item that does not begin with an identifier is consumed as a nested qualified rule, which its invalid selector drops; Nesting 1 https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting";
const BOM_IN_STRING: &str = "a byte order mark is removed only when decoding bytes; in string input U+FEFF or U+FFFE is a non-ASCII ident code point, so the text is an invalid qualified rule; Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#input-byte-stream";
const CONDITION_TRIVIA: &str = "condition text retains authored trivia, permitted but not required; Conditional 3 https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#the-csssupportsrule-interface";
const FAMILY_NAME: &str = "a family name serializes unquoted when it is an identifier sequence; Fonts 4 https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-family-name-syntax";
const NUMBER_FORM: &str = "numbers serialize in shortest base-ten form; CSSOM https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-component-value";
const HEX_COLOR: &str = "sRGB hex colors serialize as rgb() or rgba(); Color 4 https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#serializing-sRGB-values";
const CALC_SIMPLIFICATION: &str = "specified calculations are simplified before serialization; Values 4 https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-simplification";
const UNICODE_RANGE: &str = "a user agent may normalize a unicode-range list into a different list of the same code points, so the interpreted range serializes as uppercase U+start[-end] without leading zeros; Fonts 4 https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#unicode-range-desc, whose interpretation is Syntax 3 https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#urange; no selected clause fixes the spelling, as CSSOM editor capture 2026-10-09 leaves serialize a <'unicode-range'> undefined (references/cssom-1--editor-capture-20261009--d42e145ec395.md)";

/// Upstream-parsed cases whose normalized projection differs. Sorted by ID.
pub(crate) const EXCEPTIONS: &[CanonicalException] = &[
    deliberate(
        "atrule/atrule/container.json#/complex query",
        "@container not ((a :500px) and (not (b :500px))){}",
        CONTAINER_CONDITION_TRIVIA,
    ),
    deliberate(
        "atrule/atrule/font-face.json#/@font-face with declaration looks like a rule in block",
        "@font-face{}",
        UNKNOWN_DESCRIPTOR,
    ),
    deliberate(
        "atrule/atrule/font-face.json#/single declaration",
        "@font-face{font-family:name}",
        FAMILY_NAME,
    ),
    deliberate(
        "atrule/atrule/font-feature-values.json#/with all feature types",
        "@font-feature-values Another Font{@annotation{circled:1}@ornaments{fleuron:1}@stylistic{alt-a:1}@swash{swash1:1}@historical-forms{hist:1}}",
        FAMILY_NAME,
    ),
    deliberate(
        "atrule/atrule/import.json#/should be case insensetive",
        r#"@import"test"all"#,
        AT_RULE_CASE,
    ),
    deliberate(
        "atrule/atrule/media.json#/a list of conditions",
        "@media (min-width:500px),only screen,(200px<width) and ((foo:1/2) or (100px>width>200px)){}",
        MEDIA_ALL,
    ),
    deliberate(
        "atrule/atrule/media.json#/missed 'and' keyword",
        "@media not all{}",
        MEDIA_MALFORMED,
    ),
    deliberate(
        "atrule/atrule/media.json#/should be case insensitive",
        "@media screen{}",
        AT_RULE_CASE,
    ),
    deliberate(
        "atrule/atrule/supports.json#/base test with comments",
        "@supports not (flex :1){}",
        CONDITION_TRIVIA,
    ),
    deliberate(
        "atrule/atrule/supports.json#/base test with spaces",
        "@supports (flex :1){}",
        CONDITION_TRIVIA,
    ),
    deliberate(
        "atrule/atrule/supports.json#/should be case insensitive",
        "@supports (flex:1){}",
        AT_RULE_CASE,
    ),
    deliberate(
        "block/Block.json#/at-rule in block",
        "{}",
        UNKNOWN_PROPERTY_AND_AT_RULE,
    ),
    deliberate(
        "block/Block.json#/at-rule in the ending of block",
        "{}",
        UNKNOWN_PROPERTY_AND_AT_RULE,
    ),
    deliberate(
        "block/Block.json#/at-rule with block in block",
        "{}",
        UNKNOWN_PROPERTY_AND_AT_RULE,
    ),
    deliberate(
        "block/Block.json#/at-rule with block in the ending of block",
        "{}",
        UNKNOWN_PROPERTY_AND_AT_RULE,
    ),
    deliberate(
        "block/Block.json#/bad declaration should include semicolon/0",
        "{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "block/Block.json#/bad declaration should include semicolon/1",
        "{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "block/Block.json#/bad declaration should not include whitespaces/0",
        "{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "block/Block.json#/bad declaration should not include whitespaces/1",
        "{}",
        MISSING_COLON,
    ),
    deliberate(
        "block/Block.json#/bad declaration should not include whitespaces/2",
        "{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "block/Block.json#/bad symbol in a property name",
        "{}",
        INVALID_PROPERTY_NAME,
    ),
    deliberate("block/Block.json#/bad value", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.2", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.3", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.4", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.c.2", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.c.3", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.c.4", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.s.2", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.s.3", "{}", UNKNOWN_PROPERTY),
    deliberate("block/Block.json#/block.s.4", "{}", UNKNOWN_PROPERTY),
    deliberate(
        "block/Block.json#/hanging semicolons/3",
        "{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "block/Block.json#/uncomplete !important",
        "{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.2",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.3",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.4",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.c.2",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.c.3",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.c.4",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.s.2",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.s.3",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/DeclarationList.json#/block.s.4",
        "",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "declarationList/nesting.json#/basic",
        "color:blue",
        STYLE_ATTRIBUTE_RULES,
    ),
    deliberate(
        "declarationList/nesting.json#/don't parse nested rule when it not started with &",
        "",
        STYLE_ATTRIBUTE_RULES,
    ),
    deliberate(
        "declarationList/nesting.json#/nested @media",
        "color:blue",
        STYLE_ATTRIBUTE_RULES,
    ),
    deliberate(
        "declarationList/tolerant.json#/no colon after a property name",
        "color:red",
        MISSING_COLON,
    ),
    deliberate(
        "mediaQuery/MediaQuery.json#/dashed name feature",
        "not all",
        CUSTOM_MEDIA_COLON,
    ),
    deliberate(
        "mediaQuery/MediaQuery.json#/media feature with comments",
        "not all",
        MEDIA_MALFORMED,
    ),
    deliberate(
        "rule/Rule.json#/declaration with ~1~1",
        ".test{}",
        INVALID_NESTED_RULE,
    ),
    deliberate("rule/Rule.json#/rule.0", "s{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.1", "s{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.2", "s0,s1{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.3", "s0,s1{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.c.0", "s{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.c.1", "s{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.c.2", "s0,s1{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.c.3", "s0,s1{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.s.0", "s{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.s.1", "s{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.s.2", "s0,s1{}", UNKNOWN_PROPERTY),
    deliberate("rule/Rule.json#/rule.s.3", "s0,s1{}", UNKNOWN_PROPERTY),
    deliberate(
        "rule/Rule.json#/value.color.ident.0",
        "s{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "rule/Rule.json#/value.color.ident.1",
        "yellow{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "rule/legacy.json#/rule.4",
        ".test{color:red}",
        LEGACY_FILTER,
    ),
    deliberate(
        "rule/legacy.json#/rule.5",
        ".test{color:red}",
        LEGACY_FILTER,
    ),
    deliberate(
        "rule/legacy.json#/rule.s.4",
        ".test{color:red}",
        LEGACY_FILTER,
    ),
    deliberate(
        "rule/legacy.json#/rule.s.5",
        ".test{color:red}",
        LEGACY_FILTER,
    ),
    deliberate("rule/nested-atrule.json#/basic", ".test{}", UNKNOWN_AT_RULE),
    deliberate(
        "rule/nested-atrule.json#/recursion",
        ".test{}",
        UNKNOWN_AT_RULE,
    ),
    deliberate(
        "rule/nested-atrule.json#/with block",
        ".test{}",
        UNKNOWN_AT_RULE,
    ),
    deliberate(
        "rule/nested-atrule.json#/with prelude",
        ".test{}",
        UNKNOWN_AT_RULE,
    ),
    deliberate(
        "rule/nesting.json#/basic",
        ".foo{color:blue;&:hover{color:green}}",
        NEST_AT_RULE,
    ),
    deliberate(
        "rule/nesting.json#/don't parse nested rule when it not started with &",
        ".foo{.bar &{color:green}div:hover{color:red}}",
        NESTED_SEMICOLON,
    ),
    deliberate(
        "rule/nesting.json#/nested @starting-style",
        "selector{}",
        STARTING_STYLE,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrib.0",
        r#"[a="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrib.c.0",
        r#"[a="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrib.s.0",
        r#"[a="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrselector $=",
        r#"[a$="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrselector *=",
        r#"[a*="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrselector =",
        r#"[a="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrselector ^=",
        r#"[a^="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrselector |=",
        r#"[a|="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/AttributeSelector.json#/attrselector ~0=",
        r#"[a~="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/Combinator.json#/combinator with comments around #4",
        "",
        DEEP_COMBINATOR,
    ),
    deliberate(
        "selector/Combinator.json#/combinator with spaces around #4",
        "",
        DEEP_COMBINATOR,
    ),
    deliberate(
        "selector/Combinator.json#/legacy shadow-piercing descendant combinator",
        "",
        DEEP_COMBINATOR,
    ),
    deliberate("selector/Nth.json#/even keyword", ":nth-child(2n)", ANB),
    deliberate(
        "selector/Nth.json#/even keyword should be case insensitive",
        ":nth-child(2n)",
        ANB,
    ),
    deliberate(
        "selector/Nth.json#/nth-child case insensetive",
        ":nth-child(2n+1)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/Nth.json#/nth-last-child case insensetive",
        ":nth-last-child(2n+1)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/Nth.json#/nth-last-of-type case insensetive",
        ":nth-last-of-type(2n+1)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/Nth.json#/nth-of-type case insensetive",
        ":nth-of-type(2n+1)",
        PSEUDO_CASE,
    ),
    deliberate("selector/Nth.json#/odd keyword", ":nth-child(2n+1)", ANB),
    deliberate(
        "selector/Nth.json#/odd keyword be case insensitive",
        ":nth-child(2n+1)",
        ANB,
    ),
    deliberate(
        "selector/PseudoClassSelector.json#/pseudo element :after",
        "::after",
        LEGACY_PSEUDO_ELEMENT,
    ),
    deliberate(
        "selector/PseudoClassSelector.json#/pseudo element :before",
        "::before",
        LEGACY_PSEUDO_ELEMENT,
    ),
    deliberate(
        "selector/PseudoClassSelector.json#/pseudo element :before case insesitive",
        "::before",
        LEGACY_PSEUDO_ELEMENT,
    ),
    deliberate(
        "selector/PseudoClassSelector.json#/pseudo element :first-letter",
        "::first-letter",
        LEGACY_PSEUDO_ELEMENT,
    ),
    deliberate(
        "selector/PseudoClassSelector.json#/pseudo element :first-line",
        "::first-line",
        LEGACY_PSEUDO_ELEMENT,
    ),
    deliberate(
        "selector/Selector.json#/simpleselector.18",
        r#":lang("nl-be")"#,
        LANG_STRING,
    ),
    deliberate(
        "selector/Selector.json#/simpleselector.3",
        r#"[a="b"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/Selector.json#/simpleselector.4",
        r#"[a="b"][c="d"]"#,
        ATTRIBUTE_STRING,
    ),
    deliberate(
        "selector/functional-pseudo/-moz-any.json#/should be case insensitive",
        ":has(.a,.b)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/-webkit-any.json#/should be case insensitive",
        ":has(.a,.b)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/dir.json#/should be case insensitive",
        ":dir(ltr)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/has.json#/should be case insensitive",
        ":has(.a,.b)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/host-context.json#/should be case insensitive",
        ":host-context(.a)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/host.json#/should be case insensitive",
        ":host(.a)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/is.json#/should be case insensitive",
        ":has(.a,.b)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/is.json#/should support relative selectors",
        ":is()",
        FORGIVING_RELATIVE,
    ),
    deliberate(
        "selector/functional-pseudo/lang.json#/ident",
        r#":lang("en")"#,
        LANG_STRING,
    ),
    deliberate(
        "selector/functional-pseudo/lang.json#/list",
        r#":lang("en","de-DE","es")"#,
        LANG_STRING,
    ),
    deliberate(
        "selector/functional-pseudo/lang.json#/should be case insensitive",
        r#":lang("en-US")"#,
        LANG_STRING,
    ),
    deliberate(
        "selector/functional-pseudo/lang.json#/with spaces and comments",
        r#":lang("en")"#,
        LANG_STRING,
    ),
    deliberate(
        "selector/functional-pseudo/not.json#/should be case insensitive",
        ":not(.a,.b)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/slotted.json#/should be case insensitive",
        "::slotted(.a)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/where.json#/should be case insensitive",
        ":has(.a,.b)",
        PSEUDO_CASE,
    ),
    deliberate(
        "selector/functional-pseudo/where.json#/should support relative selectors",
        ":where()",
        FORGIVING_RELATIVE,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/BOM UTF-16BE #2",
        "",
        BOM_IN_STRING,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/BOM UTF-16LE #2",
        "",
        BOM_IN_STRING,
    ),
    deliberate("stylesheet/StyleSheet.json#/CDC", "", CDO_CDC),
    deliberate("stylesheet/StyleSheet.json#/CDO", "", CDO_CDC),
    deliberate("stylesheet/StyleSheet.json#/CDO~1CDC", "a{}", CDO_CDC),
    deliberate(
        "stylesheet/StyleSheet.json#/issue #250",
        "",
        UNKNOWN_AT_RULE,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/issue111.test1",
        "",
        UNDECLARED_NAMESPACE,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/issue90.test1",
        "@media test{@page{}}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/issue90.test2",
        "x{}@media test{a{}@page{}b{}}y{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/rule with a bad-string token (issue #93)",
        ".first{}.thisAndAllFollowingRulesGetSwallowed{color:red}",
        BAD_STRING,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.0",
        "s{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.1",
        "s0{}s1{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.2",
        "",
        UNKNOWN_AT_RULE,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.3",
        "",
        UNKNOWN_AT_RULE,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.4",
        "s{}",
        UNKNOWN_PROPERTY_AND_AT_RULE,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.c.0",
        "s{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.s.0",
        "s{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.s.1",
        "s0{}s1{}",
        UNKNOWN_PROPERTY,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.s.2",
        "",
        UNKNOWN_AT_RULE,
    ),
    deliberate(
        "stylesheet/StyleSheet.json#/stylesheet.s.3",
        "s{}",
        UNKNOWN_PROPERTY_AND_AT_RULE,
    ),
    deliberate(
        "stylesheet/tolerant.json#/bad selector/0",
        "",
        INVALID_SELECTOR,
    ),
    deliberate(
        "stylesheet/tolerant.json#/bad selector/1",
        ".a{}",
        INVALID_SELECTOR,
    ),
    deliberate(
        "stylesheet/tolerant.json#/correct fallback on error in at-rule block/0",
        "@media (max-width:1000px){}*{color:red}",
        TOP_LEVEL_GROUP_DECLARATIONS,
    ),
    deliberate(
        "stylesheet/tolerant.json#/correct fallback on error in at-rule block/1",
        "@media (max-width:1000px){a{}}",
        RULE_LIST_SEMICOLON,
    ),
    deliberate(
        "stylesheet/tolerant.json#/correct fallback on error in at-rule block/2",
        "@media (max-width:1000px){a{}}",
        RULE_LIST_SEMICOLON,
    ),
    deliberate("stylesheet/tolerant.json#/no block", "", EOF_QUALIFIED_RULE),
    deliberate(
        "stylesheet/tolerant.json#/parser should respect bracket balance over stop token",
        ".a{}",
        UNMATCHED_CLOSER,
    ),
    deliberate(
        "stylesheet/tolerant.json#/parser should respect bracket balance over stop token #2",
        ".a{width:10px}",
        UNMATCHED_CLOSER,
    ),
    deliberate(
        "stylesheet/tolerant.json#/rule with unclosed attribute selector/0",
        "",
        EOF_QUALIFIED_RULE,
    ),
    deliberate(
        "stylesheet/tolerant.json#/rule with unclosed attribute selector/1",
        "",
        EOF_QUALIFIED_RULE,
    ),
    deliberate(
        "stylesheet/tolerant.json#/rule with unclosed attribute selector/2",
        "",
        EOF_QUALIFIED_RULE,
    ),
    deliberate(
        "stylesheet/tolerant.json#/rule with unclosed pseudo selector",
        "",
        EOF_QUALIFIED_RULE,
    ),
    deliberate(
        "stylesheet/tolerant.json#/unclosed unknown functional-pseudo in selector",
        "",
        EOF_QUALIFIED_RULE,
    ),
    deliberate(
        "stylesheet/tolerant.json#/uncomplete pseudo in selector",
        "",
        INVALID_SELECTOR,
    ),
    deliberate(
        "value/Dimension.json#/dimension #1",
        "width:0.1px",
        NUMBER_FORM,
    ),
    deliberate(
        "value/Dimension.json#/dimension #3",
        "width:0px",
        NUMBER_FORM,
    ),
    deliberate(
        "value/HexColor.json#/basic/0",
        "color:rgb(17,0,0)",
        HEX_COLOR,
    ),
    deliberate(
        "value/HexColor.json#/basic/1",
        "color:rgb(18,58,188)",
        HEX_COLOR,
    ),
    deliberate(
        "value/HexColor.json#/basic/2",
        "color:rgb(171,193,35)",
        HEX_COLOR,
    ),
    deliberate(
        "value/HexColor.json#/basic/3",
        "color:rgba(161,178,195,0.83)",
        HEX_COLOR,
    ),
    deliberate(
        "value/HexColor.json#/basic/4",
        "color:rgb(170,187,204)",
        HEX_COLOR,
    ),
    deliberate(
        "value/HexColor.json#/edge cases/9",
        "color:rgb(30,87,233)",
        HEX_COLOR,
    ),
    deliberate(
        "value/Percentage.json#/percentage.1",
        "width:0.1%",
        NUMBER_FORM,
    ),
    deliberate(
        "value/UnicodeRange.json#/a number with a negative exponent",
        "U+12E-130",
        UNICODE_RANGE,
    ),
    deliberate(
        "value/UnicodeRange.json#/a single question mark",
        "U+0-F",
        UNICODE_RANGE,
    ),
    deliberate(
        "value/UnicodeRange.json#/unicode range hex pair",
        "U+F00-FFF",
        UNICODE_RANGE,
    ),
    deliberate(
        "value/UnicodeRange.json#/unicode range hex pair #2",
        "U+25-FF",
        UNICODE_RANGE,
    ),
    deliberate(
        "value/UnicodeRange.json#/unicode range hex pair number~1letters",
        "U+1234-ABCD",
        UNICODE_RANGE,
    ),
    deliberate(
        "value/UnicodeRange.json#/unicode range hex with ?",
        "U+F0000-F00FF",
        UNICODE_RANGE,
    ),
    deliberate(
        "value/UnicodeRange.json#/unicode range one hex",
        "U+F00",
        UNICODE_RANGE,
    ),
    deliberate(
        "value/function/calc.json#/nested",
        "width:calc(1% + 1.5em)",
        CALC_SIMPLIFICATION,
    ),
    deliberate(
        "value/function/calc.json#/with dimension and percentage",
        "width:calc(1% + 6em)",
        CALC_SIMPLIFICATION,
    ),
];
