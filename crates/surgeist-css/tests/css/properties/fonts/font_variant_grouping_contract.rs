//! Fonts 4 (current editor's draft) `font-variant` grouping. The shorthand is
//! `normal | none | [ [ <common-lig-values> || <discretionary-lig-values> ||
//! <historical-lig-values> || <contextual-alt-values> ] || [ small-caps | … ] ||
//! [ stylistic(…) || historical-forms || styleset(…) || character-variant(…) ||
//! swash(…) || ornaments(…) || annotation(…) ] || [ <numeric-figure-values> ||
//! <numeric-spacing-values> || <numeric-fraction-values> || ordinal ||
//! slashed-zero ] || [ <east-asian-variant-values> || <east-asian-width-values>
//! || ruby ] || [ sub | super ] || [ text | emoji | unicode ] ]`.
//!
//! Each inner bracket is one operand of the outer `||`; Values 4 §2.2 makes
//! that grouping significant, so values of one multi-value group stay
//! contiguous while their internal order stays free. Browsers also accept the
//! split form; that is a witness divergence, not a grammar basis. The `font`
//! shorthand reaches only `<font-variant-css2>` (`normal | small-caps`).

use crate::css::support::authored_property::{ParserFront, invalid};
use surgeist_css::*;

const PROPERTY: CssKnownProperty = CssKnownProperty::FontVariant;
const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];

#[test]
fn contiguous_group_values_are_accepted_in_any_internal_order() {
    for (input, expected) in [
        (
            "common-ligatures no-discretionary-ligatures small-caps",
            "common-ligatures no-discretionary-ligatures small-caps",
        ),
        (
            "small-caps no-contextual historical-ligatures common-ligatures",
            "common-ligatures historical-ligatures no-contextual small-caps",
        ),
        (
            "super slashed-zero tabular-nums lining-nums",
            "lining-nums tabular-nums slashed-zero super",
        ),
        (
            "emoji swash(Foo) historical-forms stylistic(Bar)",
            "stylistic(Bar) historical-forms swash(Foo) emoji",
        ),
        (
            "ruby full-width jis78 diagonal-fractions ordinal",
            "diagonal-fractions ordinal jis78 full-width ruby",
        ),
        (
            "jis04 ruby no-common-ligatures contextual oldstyle-nums",
            "no-common-ligatures contextual oldstyle-nums jis04 ruby",
        ),
    ] {
        for front in FRONTS {
            front.valid(PROPERTY, input, expected);
        }
    }
}

#[test]
fn ligature_values_split_by_another_group_are_rejected() {
    for value in [
        "common-ligatures small-caps no-discretionary-ligatures",
        "no-common-ligatures lining-nums historical-ligatures",
        "discretionary-ligatures sub contextual",
        "historical-ligatures emoji no-contextual",
        "contextual jis78 common-ligatures",
        "common-ligatures historical-forms discretionary-ligatures",
    ] {
        invalid(PROPERTY, value);
    }
}

#[test]
fn numeric_values_split_by_another_group_are_rejected() {
    for value in [
        "lining-nums sub tabular-nums",
        "ordinal common-ligatures slashed-zero",
        "diagonal-fractions small-caps oldstyle-nums",
        "proportional-nums ruby ordinal",
        "slashed-zero swash(Foo) stacked-fractions",
    ] {
        invalid(PROPERTY, value);
    }
}

#[test]
fn alternates_values_split_by_another_group_are_rejected() {
    for value in [
        "historical-forms small-caps swash(Foo)",
        "stylistic(A) ordinal styleset(B)",
        "character-variant(A) common-ligatures annotation(B)",
        "ornaments(A) super historical-forms",
    ] {
        invalid(PROPERTY, value);
    }
}

#[test]
fn east_asian_values_split_by_another_group_are_rejected() {
    for value in [
        "jis78 ordinal full-width",
        "ruby small-caps simplified",
        "proportional-width common-ligatures ruby",
        "traditional unicode full-width",
    ] {
        invalid(PROPERTY, value);
    }
}

#[test]
fn font_shorthand_admits_only_the_css2_variant_subset() {
    for front in FRONTS {
        front.parse(CssKnownProperty::Font, "italic small-caps bold 16px Demo");
        front.parse(CssKnownProperty::Font, "small-caps italic 16px Demo");
    }
    for value in [
        "common-ligatures 16px Demo",
        "small-caps lining-nums 16px Demo",
        "small-caps italic small-caps 16px Demo",
    ] {
        invalid(CssKnownProperty::Font, value);
    }
}
