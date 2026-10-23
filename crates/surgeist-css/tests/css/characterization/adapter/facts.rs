//! Error facts read through public accessors, for every link of a `source()`
//! chain. Facts carry no type names, so a renamed or merged error keeps its
//! facts while its `Display` text may change.

use std::error::Error as StdError;

use surgeist_css::*;

use super::report::{code_name, component_error_kind, kind_facts};

/// Accessor facts of `error` and of each error in its `source()` chain.
pub fn accessor_facts(error: &(dyn StdError + 'static)) -> Vec<String> {
    let mut facts = Vec::new();
    let mut link = Some(error);
    let mut depth = 0;
    while let Some(error) = link {
        for fact in link_facts(error) {
            facts.push(if depth == 0 {
                fact
            } else {
                format!("source{depth}.{fact}")
            });
        }
        link = error.source();
        depth += 1;
    }
    facts
}

/// `name` for a unit or payload variant of a construction or serialization
/// enum, read through a braced pattern so payload shape does not matter.
macro_rules! variant {
    ($error:expr, $type:ident, [$($variant:ident),* $(,)?]) => {
        match $error {
            $($type::$variant { .. } => stringify!($variant),)*
            #[allow(unreachable_patterns)]
            _ => "Future",
        }
    };
}

/// The kind of a descriptor-value error: its component kind or grammar
/// diagnostic kind, or the unit kind's name.
macro_rules! descriptor_kind {
    ($kind:expr, $type:ident, [$($unit:ident),* $(,)?]) => {
        match $kind {
            $type::Component(kind) => format!("component:{}", component_error_kind(*kind)),
            $type::Grammar(kind) => format!("grammar:{}", kind_facts(kind)),
            $($type::$unit => stringify!($unit).to_owned(),)*
            #[allow(unreachable_patterns)]
            _ => "Future".to_owned(),
        }
    };
}

fn link_facts(error: &(dyn StdError + 'static)) -> Vec<String> {
    if let Some(error) = error.downcast_ref::<Error>() {
        return vec![
            format!("code={}", code_name(error.code())),
            format!("kind={}", kind_facts(error.kind())),
            format!("position={}", position(error.position())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssColorOrdinaryError>() {
        return vec![match error {
            CssColorOrdinaryError::Operation(_) => "variant=Operation".to_owned(),
            CssColorOrdinaryError::Limit {
                resource,
                required,
                allowed,
            } => format!(
                "variant=Limit resource={} required={required} allowed={allowed}",
                super::color::resource_name(*resource)
            ),
            CssColorOrdinaryError::Allocation { resource } => format!(
                "variant=Allocation resource={}",
                super::color::resource_name(*resource)
            ),
        }];
    }
    if let Some(error) = error.downcast_ref::<CssColorError>() {
        return vec![match error {
            CssColorError::Unavailable(unavailable) => format!(
                "variant=Unavailable fact={}",
                super::color::unavailable_name(unavailable)
            ),
            CssColorError::Unsupported(unsupported) => format!(
                "variant=Unsupported feature={}",
                super::color::unsupported_name(unsupported)
            ),
            other => format!(
                "variant={}",
                variant!(other, CssColorError, [Facts, MalformedInput])
            ),
        }];
    }
    if let Some(error) = error.downcast_ref::<CssFontPaletteConstructionError>() {
        return vec![format!(
            "variant={}",
            variant!(
                error,
                CssFontPaletteConstructionError,
                [
                    ContextualColor,
                    InvalidName,
                    MissingFontFamily,
                    NegativeIndex
                ]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssComponentValueError>() {
        return vec![
            format!("kind={}", component_error_kind(error.kind())),
            format!("origin={}", origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssPropertyValueParseError>() {
        return vec![
            format!(
                "kind={}",
                match error.kind() {
                    CssPropertyValueErrorKind::Component(kind) => {
                        format!("component:{}", component_error_kind(*kind))
                    }
                    CssPropertyValueErrorKind::Grammar(kind) => {
                        format!("grammar:{}", kind_facts(kind))
                    }
                    _ => "Future".to_owned(),
                }
            ),
            format!("origin={}", serialized_origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssPropertyValueSyntaxError>() {
        let variant = variant!(error, CssPropertyValueSyntaxError, [Component, Property]);
        let mut facts = vec![format!("variant={variant}")];
        match error {
            CssPropertyValueSyntaxError::Component(inner) => {
                facts.extend(prefixed("component", link_facts(inner)))
            }
            CssPropertyValueSyntaxError::Property(inner) => {
                facts.extend(prefixed("property", link_facts(inner)))
            }
        }
        return facts;
    }
    if let Some(error) = error.downcast_ref::<CssExpansionError>() {
        return match error.kind() {
            CssExpansionErrorKind::InvalidReplacement(inner) => {
                let mut facts = vec!["kind=InvalidReplacement".to_owned()];
                facts.extend(prefixed("replacement", link_facts(inner)));
                facts
            }
            CssExpansionErrorKind::ResidualSubstitution => {
                vec!["kind=ResidualSubstitution".to_owned()]
            }
            CssExpansionErrorKind::UnsupportedProperty(property) => {
                vec![format!("kind=UnsupportedProperty:{}", property.stable_id())]
            }
            _ => vec!["kind=Future".to_owned()],
        };
    }
    if let Some(error) = error.downcast_ref::<CssColorProfileValueError>() {
        return vec![
            format!(
                "kind={}",
                descriptor_kind!(
                    error.kind(),
                    CssColorProfileValueErrorKind,
                    [NotPending, RecoveredComponent, ResidualSubstitution]
                )
            ),
            format!("origin={}", serialized_origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssCounterStyleValueError>() {
        return vec![
            format!(
                "kind={}",
                descriptor_kind!(
                    error.kind(),
                    CssCounterStyleValueErrorKind,
                    [NotPending, RecoveredComponent, ResidualSubstitution]
                )
            ),
            format!("origin={}", serialized_origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssFontFaceValueError>() {
        return vec![
            format!(
                "kind={}",
                descriptor_kind!(
                    error.kind(),
                    CssFontFaceValueErrorKind,
                    [RecoveredComponent, ResidualSubstitution]
                )
            ),
            format!("origin={}", serialized_origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssFontFeatureValueError>() {
        return vec![
            format!(
                "kind={}",
                descriptor_kind!(
                    error.kind(),
                    CssFontFeatureValueErrorKind,
                    [NotPending, RecoveredComponent, ResidualSubstitution]
                )
            ),
            format!("origin={}", serialized_origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssFontPaletteValueError>() {
        return vec![
            format!(
                "kind={}",
                descriptor_kind!(
                    error.kind(),
                    CssFontPaletteValueErrorKind,
                    [NotPending, ResidualSubstitution]
                )
            ),
            format!("origin={}", serialized_origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssMediaConstructionError>() {
        return construction(
            variant!(
                error,
                CssMediaConstructionError,
                [
                    Component,
                    InvalidConditionGrammar,
                    InvalidQueryGrammar,
                    RecoveredInput
                ]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssImportConstructionError>() {
        return construction(
            variant!(
                error,
                CssImportConstructionError,
                [
                    Component,
                    InvalidRuleGrammar,
                    Media,
                    RecoveredInput,
                    Serialization,
                    Supports,
                    WorkerUnavailable
                ]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssSupportsConstructionError>() {
        return construction(
            variant!(
                error,
                CssSupportsConstructionError,
                [
                    Component,
                    InvalidConditionGrammar,
                    InvalidDeclarationGrammar,
                    RecoveredInput,
                    WorkerUnavailable
                ]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssWhenConstructionError>() {
        return construction(
            variant!(
                error,
                CssWhenConstructionError,
                [
                    Component,
                    InvalidConditionGrammar,
                    InvalidMediaFeatureGrammar,
                    RecoveredInput,
                    WorkerUnavailable
                ]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssContainerConstructionError>() {
        return construction(
            variant!(
                error,
                CssContainerConstructionError,
                [
                    Component,
                    InvalidConditionGrammar,
                    InvalidPreludeGrammar,
                    WorkerUnavailable
                ]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssCustomMediaConstructionError>() {
        return construction(
            variant!(
                error,
                CssCustomMediaConstructionError,
                [
                    AmbiguousMediaBody,
                    Component,
                    InvalidName,
                    InvalidPrelude,
                    Media,
                    RecoveredInput
                ]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssNamedSupportsConstructionError>() {
        return construction(
            variant!(
                error,
                CssNamedSupportsConstructionError,
                [Component, InvalidBodyGrammar, InvalidName, RecoveredInput]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssGeneralEnclosedError>() {
        return construction(
            variant!(
                error,
                CssGeneralEnclosedError,
                [Component, WrongOuterComponent]
            ),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssImage1DConstructionError>() {
        let mut facts = vec![format!(
            "variant={}",
            variant!(
                error,
                CssImage1DConstructionError,
                [
                    Component,
                    EmptyStripes,
                    Grammar,
                    NestingLimit,
                    Numeric,
                    RecoveredComponent
                ]
            )
        )];
        match error {
            CssImage1DConstructionError::Component(inner) => {
                facts.extend(prefixed("component", link_facts(inner)))
            }
            CssImage1DConstructionError::Grammar(inner) => {
                facts.extend(prefixed("grammar", link_facts(inner)))
            }
            CssImage1DConstructionError::Numeric(inner) => {
                facts.extend(prefixed("numeric", link_facts(inner)))
            }
            CssImage1DConstructionError::RecoveredComponent(value) => {
                facts.push(format!("origin={}", origin(value)))
            }
            _ => {}
        }
        return facts;
    }
    if let Some(error) = error.downcast_ref::<CssNumericConstructionError>() {
        return vec![
            format!(
                "kind={}",
                match error.kind() {
                    CssNumericConstructionErrorKind::Component(kind) => {
                        format!("component:{}", component_error_kind(*kind))
                    }
                    other => variant!(
                        other,
                        CssNumericConstructionErrorKind,
                        [
                            Arity,
                            EmptyValue,
                            IncompatibleTypes,
                            InvalidArgumentType,
                            MalformedExpression,
                            MultipleValues,
                            OutOfRange,
                            RecoveredComponent,
                            ResourceLimit,
                            RootDomainMismatch,
                            SubstitutionRequired,
                            UnknownFunction
                        ]
                    )
                    .to_owned(),
                }
            ),
            format!(
                "origin={}",
                error.origin().map_or_else(|| "-".to_owned(), origin)
            ),
            format!("path={}", error.path().map_or_else(|| "-".to_owned(), path)),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssColorScalarError>() {
        return vec![
            format!(
                "kind={}",
                variant!(
                    error.kind(),
                    CssColorScalarErrorKind,
                    [InvalidComponent, OutOfRange]
                )
            ),
            format!("origin={}", origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssCharacterAlignmentError>() {
        return vec![
            format!(
                "kind={}",
                match error.kind() {
                    CssCharacterAlignmentErrorKind::Component(kind) => {
                        format!("component:{}", component_error_kind(kind))
                    }
                    CssCharacterAlignmentErrorKind::InvalidCharacter => {
                        "InvalidCharacter".to_owned()
                    }
                    _ => "Future".to_owned(),
                }
            ),
            format!("origin={}", origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssSelectorConstructionError>() {
        return vec![format!(
            "kind={}",
            variant!(
                error.kind(),
                CssSelectorConstructionErrorKind,
                [
                    EmptyComplexRest,
                    EmptyCompound,
                    EmptyList,
                    InvalidAttributeValue,
                    InvalidTypePosition,
                    InvalidUnknownWebkitPseudoElementName,
                    ModifierWithoutValue,
                    NonTerminalPseudoElement,
                    Specified
                ]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssSerializationError>() {
        return vec![
            format!("kind={}", serialization_kind(error.kind())),
            format!(
                "origin={}",
                error.origin().map_or_else(|| "-".to_owned(), origin)
            ),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssRuleSerializationError>() {
        return vec![format!(
            "kind={}",
            match error.kind() {
                CssRuleCssomSerializationErrorKind::Resource(kind) => {
                    format!("resource:{}", serialization_kind(kind))
                }
                CssRuleCssomSerializationErrorKind::Value(kind) => {
                    format!("value:{}", serialization_kind(kind))
                }
                other => variant!(
                    other,
                    CssRuleCssomSerializationErrorKind,
                    [DeclarationBlock, FormatUnavailable, SourceUndefined]
                )
                .to_owned(),
            }
        )];
    }
    if let Some(error) = error.downcast_ref::<CssDeclarationBlockError>() {
        return vec![format!(
            "kind={}",
            variant!(
                error.kind(),
                CssDeclarationBlockErrorKind,
                [
                    DuplicateTerminal,
                    Expansion,
                    InvalidKeyframeSource,
                    InvalidPageSource,
                    PendingFootprintUndetermined,
                    Serialization,
                    UnresolvedEnvironment
                ]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssPageBlockError>() {
        return vec![format!(
            "kind={}",
            variant!(
                error.kind(),
                CssPageBlockErrorKind,
                [InvalidPropertyValue, PropertyNotApplicable, Resource]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssNormalizationError>() {
        return vec![format!(
            "kind={}",
            variant!(
                error.kind(),
                CssNormalizationErrorKind,
                [LimitExceeded, UnsupportedDeclaration]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssKeyframeSelectorComparisonError>() {
        return vec![format!(
            "stage={}",
            variant!(
                error.stage(),
                CssKeyframeSelectorComparisonStage,
                [Comparison, LeftNormalization, RightNormalization]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssFontFeatureValuesError>() {
        return vec![format!(
            "kind={}",
            variant!(
                error.kind(),
                CssFontFeatureValuesErrorKind,
                [
                    EmptyFamilies,
                    EmptyIndexes,
                    InvalidIdentifier,
                    InvalidIndexCount,
                    InvalidIntegerSyntax,
                    KindMismatch,
                    NegativeIndex
                ]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssRuleConstructionError>() {
        return vec![format!(
            "kind={}",
            variant!(
                error.kind(),
                CssRuleConstructionErrorKind,
                [
                    CapacityOverflow,
                    DuplicateNamespace,
                    InvalidBoundaryAnchor,
                    InvalidPlacement,
                    InvalidPreludeOrder,
                    InvalidSelectorIdentifier,
                    NamespaceMismatch,
                    NestingLimit,
                    SelectorNestingLimit
                ]
            )
        )];
    }
    if let Some(error) = error.downcast_ref::<CssPathDataConstructionError>() {
        return vec![
            format!(
                "kind={}",
                variant!(
                    error.kind(),
                    CssPathDataConstructionErrorKind,
                    [EmptyPath, ExpectedString, InvalidPathData, RecoveredInput]
                )
            ),
            format!("origin={}", origin(error.origin())),
        ];
    }
    Vec::new()
}

fn construction(variant: &str, value: &CssValueOrigin) -> Vec<String> {
    vec![
        format!("variant={variant}"),
        format!("origin={}", origin(value)),
    ]
}

fn prefixed(prefix: &str, facts: Vec<String>) -> Vec<String> {
    facts
        .into_iter()
        .map(|fact| format!("{prefix}.{fact}"))
        .collect()
}

fn serialization_kind(kind: CssSpecifiedValueSerializationErrorKind) -> &'static str {
    variant!(
        kind,
        CssSpecifiedValueSerializationErrorKind,
        [
            ByteLimit,
            CapacityOverflow,
            DepthLimit,
            InputNodeLimit,
            NamespaceBindingUnavailable,
            ProjectionNodeLimit,
            UnrepresentableValue,
            UnserializableBoundary
        ]
    )
}

fn position(position: CssSourcePosition) -> String {
    format!(
        "{}:{}:{}",
        position.byte_offset().value(),
        position.line().value(),
        position.column().value()
    )
}

fn span(origin: &CssParsedOrigin) -> String {
    let span = origin.span();
    format!(
        "{}-{}",
        span.start().byte_offset().value(),
        span.end().byte_offset().value()
    )
}

/// Where a value came from, by its source byte span.
pub fn origin(origin: &CssValueOrigin) -> String {
    match origin {
        CssValueOrigin::Parsed(parsed) => format!("parsed:{}", span(parsed)),
        CssValueOrigin::Programmatic => "programmatic".to_owned(),
        CssValueOrigin::UnretainedInput { byte_length } => format!("unretained:{byte_length}"),
        CssValueOrigin::ImplicitClosure { opening, at } => {
            format!("implicit-closure:{}@{}", span(opening), span(at))
        }
        _ => "future".to_owned(),
    }
}

fn serialized_origin(origin_value: &CssSerializedOrigin) -> String {
    match origin_value {
        CssSerializedOrigin::Token(value) => format!("token:{}", origin(value)),
        CssSerializedOrigin::End(value) => {
            format!(
                "end:{}",
                value.as_ref().map_or_else(|| "-".to_owned(), origin)
            )
        }
        CssSerializedOrigin::Separator { before, after } => {
            format!("separator:{}|{}", origin(before), origin(after))
        }
        _ => "future".to_owned(),
    }
}

fn path(values: &[usize]) -> String {
    values
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".")
}
