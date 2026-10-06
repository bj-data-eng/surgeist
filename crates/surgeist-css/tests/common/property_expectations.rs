//! Independently authored common property contracts, consumed through public CSS APIs.
//! See `property_expectations/README.md` for ownership and adding a record.

#![expect(
    dead_code,
    reason = "each consumer exercises a different contract subset"
)]

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

pub type WrapperAssertion = fn(&CssKnownDeclaration, CssKnownPropertyValueRef<'_>, &str);

pub struct PropertyExpectation {
    pub property: P,
    pub name: &'static str,
    pub metadata: MetadataExpectation,
    pub catalog: Option<CatalogExpectation>,
    pub source_id: Option<&'static str>,
    pub aliases: &'static [&'static str],
    pub dispatch: Option<DispatchExpectation>,
    pub wrapper: Option<WrapperAssertion>,
}

pub enum MetadataExpectation {
    Longhand {
        inherited: bool,
        initial: InitialExpectation,
    },
    Shorthand {
        settable: &'static [P],
        reset: &'static [P],
    },
    FourSide,
    UniversalReset,
    Unavailable,
}

pub enum InitialExpectation {
    Fixed(fn(CssLonghandValueRef<'_>)),
    UserAgent(CssUserAgentInitial),
}

pub enum CatalogExpectation {
    Grammar {
        feature_id: &'static str,
        positive: &'static str,
        boundary: BoundaryExpectation,
        production: Option<&'static str>,
    },
    Complete {
        feature_id: &'static str,
        authored: &'static str,
        production: &'static str,
    },
}

pub struct BoundaryExpectation {
    pub authored: &'static str,
    pub outcome: BoundaryOutcome,
}

pub enum BoundaryOutcome {
    Rejected,
    Accepted(fn(&CssKnownDeclaration)),
}

pub struct DispatchExpectation {
    pub ordinary: &'static str,
    pub important: &'static str,
}

impl PropertyExpectation {
    pub fn ordinary_stimulus(&self) -> Option<&'static str> {
        self.dispatch
            .as_ref()
            .map(|dispatch| dispatch.ordinary)
            .or_else(|| {
                self.catalog.as_ref().map(|catalog| match catalog {
                    CatalogExpectation::Grammar { positive, .. } => *positive,
                    CatalogExpectation::Complete { authored, .. } => *authored,
                })
            })
    }

    pub fn feature_id(&self) -> Option<&'static str> {
        self.catalog.as_ref().map(|catalog| match catalog {
            CatalogExpectation::Grammar { feature_id, .. }
            | CatalogExpectation::Complete { feature_id, .. } => *feature_id,
        })
    }

    #[track_caller]
    pub fn assert_wrapper(
        &self,
        declaration: &CssKnownDeclaration,
        value: CssKnownPropertyValueRef<'_>,
        authored: &str,
    ) {
        assert_eq!(
            declaration.property(),
            self.property,
            "{} parsed identity",
            self.name
        );
        self.wrapper
            .expect("ordinary stimulus requires a typed wrapper")(
            declaration, value, authored
        );
    }
}

macro_rules! optional {
    () => {
        None
    };
    ($value:expr) => {
        Some($value)
    };
}

const fn rejected(authored: &'static str) -> BoundaryExpectation {
    BoundaryExpectation {
        authored,
        outcome: BoundaryOutcome::Rejected,
    }
}

const fn accepted(
    authored: &'static str,
    assertion: fn(&CssKnownDeclaration),
) -> BoundaryExpectation {
    BoundaryExpectation {
        authored,
        outcome: BoundaryOutcome::Accepted(assertion),
    }
}

macro_rules! grammar_catalog {
    ($id:literal, $positive:literal, $boundary:expr $(, $production:literal)?) => {
        CatalogExpectation::Grammar { feature_id: $id, positive: $positive, boundary: $boundary, production: optional!($($production)?) }
    };
}

macro_rules! metadata_expectation {
    ($variant:ident, longhand($inherited:literal, ua($requirement:expr))) => {
        MetadataExpectation::Longhand { inherited: $inherited, initial: InitialExpectation::UserAgent($requirement) }
    };
    ($variant:ident, longhand($inherited:literal, |$value:ident| $body:expr)) => {
        MetadataExpectation::Longhand {
            inherited: $inherited,
            initial: InitialExpectation::Fixed(|view| {
                let CssLonghandValueRef::$variant($value) = view else {
                    panic!("{} initial has the wrong typed variant: {view:?}", stringify!($variant));
                };
                $body
            }),
        }
    };
    ($variant:ident, shorthand([$($settable:ident),*], [$($reset:ident),*])) => {
        MetadataExpectation::Shorthand { settable: &[$(P::$settable),*], reset: &[$(P::$reset),*] }
    };
    ($variant:ident, four_side()) => { MetadataExpectation::FourSide };
    ($variant:ident, universal()) => { MetadataExpectation::UniversalReset };
    ($variant:ident, unavailable()) => { MetadataExpectation::Unavailable };
}

macro_rules! property_records {
    ($( $variant:ident, $name:literal {
        metadata: $kind:ident $arguments:tt,
        $(catalog: $catalog:expr,)?
        $(source: $source:literal,)?
        $(aliases: [$($alias:literal),*],)?
        $(dispatch: $dispatch:literal $(=> $important:literal)?,)?
        $(wrapper: $wrapper:ident,)?
    } )*) => {
        pub const CASES: &[PropertyExpectation] = &[
            $(PropertyExpectation {
                property: P::$variant,
                name: $name,
                metadata: metadata_expectation!($variant, $kind $arguments),
                catalog: optional!($($catalog)?),
                source_id: optional!($($source)?),
                aliases: &[$($($alias),*)?],
                dispatch: property_records!(@dispatch $($dispatch $(=> $important)?)?),
                wrapper: property_records!(@wrapper $variant $($wrapper)?),
            },)*
        ];
    };
    (@dispatch) => { None };
    (@dispatch $ordinary:literal) => { Some(DispatchExpectation { ordinary: $ordinary, important: $ordinary }) };
    (@dispatch $ordinary:literal => $important:literal) => { Some(DispatchExpectation { ordinary: $ordinary, important: $important }) };
    (@wrapper $variant:ident) => { None };
    (@wrapper $variant:ident yes) => {
        Some(|declaration, value, expected| {
            let (P::$variant, CssKnownPropertyValueRef::$variant(value)) = (declaration.property(), value) else {
                panic!("{} property/value wrapper mismatch for `{expected}`: {value:?}", stringify!($variant));
            };
            assert_eq!(value.as_css(), expected, "{} authored wrapper", stringify!($variant));
        })
    };
}

pub fn find(property: P) -> Option<&'static PropertyExpectation> {
    CASES.iter().find(|case| case.property == property)
}

include!("property_expectations/records.rs");

fn exact_literal(component: Option<&surgeist_css::CssComponentValue>, css: &str) -> bool {
    use surgeist_css::{CssComponentValueRef as Component, CssValueTokenRef as Token};
    let expected = surgeist_css::CssComponentValue::try_token(css).unwrap();
    match (
        component.map(surgeist_css::CssComponentValue::view),
        expected.view(),
    ) {
        (
            Some(Component::Token(Token::Number(actual))),
            Component::Token(Token::Number(expected)),
        )
        | (
            Some(Component::Token(Token::Percentage(actual))),
            Component::Token(Token::Percentage(expected)),
        ) => actual.representation() == expected.representation(),
        (
            Some(Component::Token(Token::Dimension {
                number: actual,
                unit: actual_unit,
            })),
            Component::Token(Token::Dimension {
                number: expected,
                unit: expected_unit,
            }),
        ) => actual.representation() == expected.representation() && actual_unit == expected_unit,
        _ => false,
    }
}
