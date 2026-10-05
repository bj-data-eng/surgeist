//! Independently authored expectations shared by catalog, dispatch, and metadata consumers.
//!
//! These records are test inputs, never generated from the production property schema.
//! CSS Text 4: `references/css-text-4--WD-css-text-4-20260814--106c5cd086ab.md`.
//! Preserve additional grammar/lifecycle stimuli in their owning family suites.

#![expect(
    dead_code,
    reason = "each consumer exercises a different expectation subset"
)]

use surgeist_css::*;

pub struct PropertyExpectation {
    pub property: CssKnownProperty,
    pub name: &'static str,
    pub feature_id: &'static str,
    pub source_id: &'static str,
    pub positive: &'static str,
    pub negative: &'static str,
    pub dispatch: &'static str,
    pub metadata: MetadataExpectation,
}

pub enum MetadataExpectation {
    Longhand {
        inherited: bool,
        assert_initial: fn(CssLonghandValueRef<'_>),
    },
    Shorthand {
        settable: &'static [CssKnownProperty],
        reset: &'static [CssKnownProperty],
    },
}

// The independently supplied variant token drives both the expected public
// wrapper arm and the typed initial arm. No production macro is imported.
macro_rules! property_expectations {
    (
        source: $source:literal;
        longhands: { $( $longhand:ident {
            name: $name:literal, feature: $feature:literal,
            positive: $positive:literal, negative: $negative:literal,
            $(dispatch: $dispatch:literal,)?
            inherited: $inherited:literal, initial: $initial:expr
        } )* }
        shorthands: { $( $shorthand:ident {
            name: $short_name:literal, feature: $short_feature:literal,
            positive: $short_positive:literal, negative: $short_negative:literal,
            settable: [$($member:ident),*], reset: [$($reset:ident),*]
        } )* }
    ) => {
        pub const CASES: &[PropertyExpectation] = &[
            $(PropertyExpectation {
                property: CssKnownProperty::$longhand,
                name: $name,
                feature_id: $feature,
                source_id: $source,
                positive: $positive,
                negative: $negative,
                dispatch: property_expectations!(@dispatch $positive $(, $dispatch)?),
                metadata: MetadataExpectation::Longhand {
                    inherited: $inherited,
                    assert_initial: |value| {
                        let CssLonghandValueRef::$longhand(actual) = value else {
                            panic!("{} initial has the wrong typed variant: {value:?}", $name);
                        };
                        assert_eq!(*actual, $initial, "{} intrinsic initial", $name);
                    },
                },
            },)*
            $(PropertyExpectation {
                property: CssKnownProperty::$shorthand,
                name: $short_name,
                feature_id: $short_feature,
                source_id: $source,
                positive: $short_positive,
                negative: $short_negative,
                dispatch: $short_positive,
                metadata: MetadataExpectation::Shorthand {
                    settable: &[$(CssKnownProperty::$member),*],
                    reset: &[$(CssKnownProperty::$reset),*],
                },
            },)*
        ];

        pub fn assert_wrapper(
            declaration: &CssKnownDeclaration,
            value: CssKnownPropertyValueRef<'_>,
            expected: &str,
        ) {
            match (declaration.property(), value) {
                $( (CssKnownProperty::$longhand, CssKnownPropertyValueRef::$longhand(value)) => {
                    assert_eq!(value.as_css(), expected, "{} authored wrapper", $name);
                }, )*
                $( (CssKnownProperty::$shorthand, CssKnownPropertyValueRef::$shorthand(value)) => {
                    assert_eq!(value.as_css(), expected, "{} authored wrapper", $short_name);
                }, )*
                other => panic!("property/value wrapper mismatch for `{expected}`: {other:?}"),
            }
        }
    };
    (@dispatch $positive:literal) => { $positive };
    (@dispatch $positive:literal, $dispatch:literal) => { $dispatch };
}

property_expectations! {
    source: "X-TEXT4";
    longhands: {
        TextWrapMode {
            name: "text-wrap-mode", feature: "ext.property.text-wrap-mode",
            positive: "nowrap", negative: "balance",
            inherited: true, initial: CssTextWrapMode::Wrap
        }
        TextWrapStyle {
            name: "text-wrap-style", feature: "ext.property.text-wrap-style",
            positive: "avoid-short-last-line", negative: "nowrap",
            inherited: true, initial: CssTextWrapStyle::Auto
        }
        WhiteSpaceCollapse {
            name: "white-space-collapse", feature: "ext.property.white-space-collapse",
            positive: "discard", negative: "pre",
            inherited: true, initial: CssWhiteSpaceCollapse::Collapse
        }
        WhiteSpaceTrim {
            name: "white-space-trim", feature: "ext.property.white-space-trim",
            positive: "discard-after discard-before", negative: "discard-before discard-before",
            dispatch: "discard-inner discard-before",
            inherited: false, initial: CssWhiteSpaceTrim::none()
        }
        WordBreak {
            name: "word-break", feature: "baseline.property.word-break",
            positive: "keep-all", negative: "nowrap",
            inherited: true, initial: CssWordBreak::Normal
        }
    }
    shorthands: {
        TextWrap {
            name: "text-wrap", feature: "baseline.property.text-wrap",
            positive: "balance", negative: "nowrap wrap",
            settable: [TextWrapMode, TextWrapStyle], reset: []
        }
        WhiteSpace {
            name: "white-space", feature: "baseline.property.white-space",
            positive: "pre-wrap", negative: "balance",
            settable: [WhiteSpaceCollapse, TextWrapMode, WhiteSpaceTrim], reset: []
        }
    }
}

pub fn find(property: CssKnownProperty) -> Option<&'static PropertyExpectation> {
    CASES.iter().find(|case| case.property == property)
}
