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
    pub inverse: Option<InverseExpectation>,
    pub mapping: MappingExpectation,
    /// Additional ordinary `(authored, specified serialization)` rows, run by
    /// the record harness through every front with re-parse idempotence.
    pub accepts: &'static [(&'static str, &'static str)],
    /// Additional values the property grammar rejects at every front.
    pub rejects: &'static [&'static str],
}

pub struct InverseExpectation {
    pub authored: &'static str,
    pub value: &'static str,
}

pub struct IndependentTerminalExpectation {
    pub property: CssPropertyNameRef<'static>,
    pub name: &'static str,
    pub mapping: MappingExpectation,
    pub keyframe_admitted: bool,
}

#[derive(Clone, Copy)]
pub enum MappingExpectation {
    Neutral,
    Logical(&'static str),
    Physical(&'static str),
}
impl MappingExpectation {
    pub const fn bucket(self) -> u8 {
        match self {
            Self::Logical(_) => 0,
            Self::Neutral => 1,
            Self::Physical(_) => 2,
        }
    }
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
        /// The specified serialization of `positive` when it differs from the
        /// authored text; `None` records that `positive` is already written
        /// in its specified serialization. Each stated form follows CSSOM 1
        /// (WD 2021-08-26) §6.7.2 (canonical grammar order for `||`/`&&`
        /// components, omitted defaults, lowercase keywords, `url("…")`,
        /// double-quoted strings, `<time>` in seconds, `, ` separators), Values
        /// 4 (WD 2024-03-12) §8.3.2 two-component `<position>`, §5.7 both-term
        /// `<ratio>` and §10.13 calculation simplification and term order,
        /// Color 4 (CRD 2026-09-08) §16 `rgb()` and number-alpha forms, Box
        /// Alignment 3 (WD 2026-01-30) §4.2 `first baseline` as `baseline`, and
        /// Transforms 2 (WD 2021-11-09) §5 numeric scale percentages.
        serialization: Option<&'static str>,
        boundary: BoundaryExpectation,
        production: Option<&'static str>,
        status: Option<CssSupportStatus>,
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

    /// Every ordinary `(authored, specified serialization)` row: the grammar
    /// catalog's positive sample, unless its serialization is a pinned known
    /// defect, followed by the record's additional `accepts` rows. `all`
    /// admits no ordinary value.
    pub fn accepted_rows(&self) -> Vec<(&'static str, &'static str)> {
        let mut rows = Vec::new();
        if let Some(CatalogExpectation::Grammar {
            feature_id,
            positive,
            serialization,
            ..
        }) = &self.catalog
            && !matches!(self.metadata, MetadataExpectation::UniversalReset)
            && !KNOWN_SERIALIZATION_DEFECTS
                .iter()
                .any(|(id, _)| id == feature_id)
        {
            rows.push((*positive, serialization.unwrap_or(positive)));
        }
        rows.extend_from_slice(self.accepts);
        rows
    }

    /// Every value the grammar rejects: the catalog's rejected boundary
    /// sample followed by the record's additional `rejects` rows.
    pub fn rejected_rows(&self) -> Vec<&'static str> {
        let mut rows = Vec::new();
        if let Some(CatalogExpectation::Grammar {
            boundary:
                BoundaryExpectation {
                    authored,
                    outcome: BoundaryOutcome::Rejected,
                },
            ..
        }) = &self.catalog
        {
            rows.push(*authored);
        }
        rows.extend_from_slice(self.rejects);
        rows
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

/// Positive vectors whose current specified serialization keeps a component
/// that CSSOM 1 (WD 2021-08-26) §6.7.2 omits because omitting it does not
/// change the meaning, with that current serialization. Each record states
/// the spec form and its basis:
///
/// - `place-items`: Box Alignment 3 (WD 2026-01-30) §7.3 copies an omitted
///   `<'justify-items'>` from `<'align-items'>`, giving `stretch`.
/// - `background-size`: Backgrounds 3 §2.9, omitted second size is `auto`.
/// - `counter-increment`: Lists 3 §4.2, omitted integer is 1.
/// - `font-feature-settings`: Fonts 4 §6.12, `on` and an omitted value are 1.
/// - `shape-outside`: Shapes 1 §6.1, omitted box is margin-box.
/// - `mask-border`: Masking 1 §8.7, initial `auto` width and `alpha` mode.
pub const KNOWN_SERIALIZATION_DEFECTS: &[(&str, &str)] = &[
    (
        "baseline.property.place-items",
        "place-items: stretch stretch;",
    ),
    (
        "baseline.property.background-size",
        "background-size: cover, 10px auto;",
    ),
    (
        "baseline.property.counter-increment",
        "counter-increment: section 1;",
    ),
    (
        "baseline.property.font-feature-settings",
        "font-feature-settings: \"kern\" on, \"liga\" 0;",
    ),
    (
        "official.property.shape-outside",
        "shape-outside: circle(50%) margin-box;",
    ),
    (
        "official.property.mask-border",
        "mask-border: url(\"a\") 1 2 fill / auto / 2 round alpha;",
    ),
];

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
    ($id:literal, $positive:literal $(=> $serialization:literal)?, $boundary:expr $(, $production:literal)?) => {
        CatalogExpectation::Grammar { feature_id: $id, positive: $positive, serialization: optional!($($serialization)?), boundary: $boundary, production: optional!($($production)?), status: None }
    };
}

macro_rules! complete_grammar_catalog {
    ($id:literal, $positive:literal $(=> $serialization:literal)?, $boundary:expr) => {
        CatalogExpectation::Grammar {
            feature_id: $id,
            positive: $positive,
            serialization: optional!($($serialization)?),
            boundary: $boundary,
            production: None,
            status: Some(CssSupportStatus::Complete),
        }
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
        $(inverse: ($inverse_input:literal, $inverse_value:literal),)?
        $(mapping: $mapping:ident($group:literal),)?
        $(accepts: [$($accepted:literal $(=> $canonical:literal)?),* $(,)?],)?
        $(rejects: [$($rejected:literal),* $(,)?],)?
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
                inverse: optional!($(InverseExpectation { authored: $inverse_input, value: $inverse_value })?),
                mapping: property_records!(@mapping $($mapping($group))?),
                accepts: &[$($(property_records!(@accept $accepted $(=> $canonical)?)),*)?],
                rejects: &[$($($rejected),*)?],
            },)*
        ];
    };
    (@accept $authored:literal) => { ($authored, $authored) };
    (@accept $authored:literal => $canonical:literal) => { ($authored, $canonical) };
    (@dispatch) => { None };
    (@dispatch $ordinary:literal) => { Some(DispatchExpectation { ordinary: $ordinary, important: $ordinary }) };
    (@dispatch $ordinary:literal => $important:literal) => { Some(DispatchExpectation { ordinary: $ordinary, important: $important }) };
    (@wrapper $variant:ident) => { None };
    (@mapping) => { MappingExpectation::Neutral };
    (@mapping logical($group:literal)) => { MappingExpectation::Logical($group) };
    (@mapping physical($group:literal)) => { MappingExpectation::Physical($group) };
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
