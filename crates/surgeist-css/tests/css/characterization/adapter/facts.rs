//! Error facts read through public accessors, for every link of a `source()`
//! chain. Facts carry no type names, so a renamed or merged error keeps its
//! facts while its `Display` text may change.

use std::error::Error as StdError;

use surgeist_css::color::{CssColorConstructionError, CssColorError, CssColorResolveError};
use surgeist_css::component::{CssComponentValueError, CssSerializedOrigin, CssValueOrigin};
use surgeist_css::declaration::CssDeclarationBlockError;
use surgeist_css::diagnostic::CssError;
use surgeist_css::normalize::CssNormalizationError;
use surgeist_css::numeric::CssNumericConstructionError;
use surgeist_css::property::{CssExpansionError, CssPropertyValueError};
use surgeist_css::query::CssConditionConstructionError;
use surgeist_css::query::container::CssContainerConstructionError;
use surgeist_css::query::media::CssMediaConstructionError;
use surgeist_css::query::supports::CssSupportsConstructionError;
use surgeist_css::rule::CssRuleConstructionError;
use surgeist_css::rule::color_profile::CssColorProfileValueError;
use surgeist_css::rule::counter_style::CssCounterStyleValueError;
use surgeist_css::rule::font_face::CssFontFaceValueError;
use surgeist_css::rule::font_feature_values::CssFontFeatureValuesError;
use surgeist_css::rule::font_palette_values::CssFontPaletteValuesError;
use surgeist_css::rule::keyframes::{
    CssKeyframeSelectorComparisonError, CssKeyframeSelectorComparisonStage,
};
use surgeist_css::rule::page::CssPageBlockError;
use surgeist_css::selector::CssSelectorConstructionError;
use surgeist_css::serialization::{CssRuleSerializationError, CssSerializationError};
use surgeist_css::source::{CssParsedOrigin, CssSourcePosition};
use surgeist_css::values::image::CssImageConstructionError;
use surgeist_css::values::shape::CssShapeConstructionError;
use surgeist_css::values::text::CssTextConstructionError;

use super::report::{code_name, component_error_kind, kind_facts};
use crate::css::support::failure_classes::{
    NumericFailure, SerializationFailure, component_failure, numeric_failure, serialization_failure,
};

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

/// The name of the first class whose predicate holds, or `Future`.
fn first(classes: &[(bool, &'static str)]) -> &'static str {
    classes
        .iter()
        .find_map(|(holds, name)| holds.then_some(*name))
        .unwrap_or("Future")
}

/// The kind of a descriptor-value failure: its component kind or grammar
/// diagnostic kind, or the name of the refusal.
fn descriptor_kind(
    component: Option<&CssComponentValueError>,
    diagnostic: Option<&CssError>,
    units: &[(bool, &'static str)],
) -> String {
    if let Some(component) = component {
        return format!(
            "component:{}",
            component_error_kind(component_failure(component))
        );
    }
    if let Some(diagnostic) = diagnostic {
        return format!("grammar:{}", kind_facts(diagnostic));
    }
    first(units).to_owned()
}

/// The facts of a descriptor-value failure, which every descriptor family reports alike.
macro_rules! descriptor_facts {
    ($error:expr, $origin:expr, [$($predicate:ident => $name:literal),* $(,)?]) => {
        vec![
            format!(
                "kind={}",
                descriptor_kind(
                    $error.component_error(),
                    $error.diagnostic(),
                    &[$(($error.$predicate(), $name)),*],
                )
            ),
            format!("origin={}", serialized_origin($origin)),
        ]
    };
}

fn link_facts(error: &(dyn StdError + 'static)) -> Vec<String> {
    if let Some(error) = error.downcast_ref::<CssError>() {
        return vec![
            format!("code={}", code_name(error.code())),
            format!("kind={}", kind_facts(error)),
            format!("position={}", position(error.position())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssColorResolveError>() {
        if error.color_error().is_some() {
            return vec!["variant=Operation".to_owned()];
        }
        if let Some((resource, allowed, required)) = error.exhausted_limit() {
            return vec![format!(
                "variant=Limit resource={} required={required} allowed={allowed}",
                super::color::resource_name(resource)
            )];
        }
        if let Some(resource) = error.failed_allocation() {
            return vec![format!(
                "variant=Allocation resource={}",
                super::color::resource_name(resource)
            )];
        }
        return vec!["variant=Future".to_owned()];
    }
    if let Some(error) = error.downcast_ref::<CssColorError>() {
        if let Some(unavailable) = error.unavailable() {
            return vec![format!(
                "variant=Unavailable fact={}",
                super::color::unavailable_name(&unavailable)
            )];
        }
        if let Some(unsupported) = error.unsupported() {
            return vec![format!(
                "variant=Unsupported feature={}",
                super::color::unsupported_name(&unsupported)
            )];
        }
        return vec![format!(
            "variant={}",
            first(&[
                (error.fact_defect().is_some(), "Facts"),
                (error.malformed_input_offset().is_some(), "MalformedInput"),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssFontPaletteValuesError>() {
        // A descriptor-value failure carries its origin; a palette
        // construction failure has none.
        if let Some(origin) = error.origin() {
            return descriptor_facts!(error, origin, [
                is_not_pending => "NotPending",
                is_residual_substitution => "ResidualSubstitution",
            ]);
        }
        return vec![format!(
            "variant={}",
            first(&[
                (error.contextual_color().is_some(), "ContextualColor"),
                (error.is_invalid_name(), "InvalidName"),
                (error.is_missing_font_family(), "MissingFontFamily"),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssComponentValueError>() {
        return vec![
            format!("kind={}", component_error_kind(component_failure(error))),
            format!("origin={}", origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssPropertyValueError>() {
        let kind = match error.component_error() {
            Some(component) if error.code().is_none() => {
                format!(
                    "component:{}",
                    component_error_kind(component_failure(component))
                )
            }
            _ if error.code().is_some() => format!("grammar:{}", kind_facts(error)),
            _ => "Future".to_owned(),
        };
        return vec![
            format!("kind={kind}"),
            format!("origin={}", serialized_origin(&error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssExpansionError>() {
        if let Some(inner) = error.replacement_error() {
            let mut facts = vec!["kind=InvalidReplacement".to_owned()];
            facts.extend(prefixed("replacement", link_facts(inner)));
            return facts;
        }
        if error.is_residual_substitution() {
            return vec!["kind=ResidualSubstitution".to_owned()];
        }
        return vec!["kind=Future".to_owned()];
    }
    if let Some(error) = error.downcast_ref::<CssColorProfileValueError>() {
        return descriptor_facts!(error, error.origin(), [
            is_not_pending => "NotPending",
            is_recovered_component => "RecoveredComponent",
            is_residual_substitution => "ResidualSubstitution",
        ]);
    }
    if let Some(error) = error.downcast_ref::<CssCounterStyleValueError>() {
        return descriptor_facts!(error, error.origin(), [
            is_not_pending => "NotPending",
            is_recovered_component => "RecoveredComponent",
            is_residual_substitution => "ResidualSubstitution",
        ]);
    }
    if let Some(error) = error.downcast_ref::<CssFontFaceValueError>() {
        return descriptor_facts!(error, error.origin(), [
            is_recovered_component => "RecoveredComponent",
            is_residual_substitution => "ResidualSubstitution",
        ]);
    }
    if let Some(error) = error.downcast_ref::<CssFontFeatureValuesError>() {
        // A descriptor-value failure carries its origin; a structural
        // failure of the rule has none.
        if let Some(origin) = error.origin() {
            return descriptor_facts!(error, origin, [
                is_not_pending => "NotPending",
                is_recovered_component => "RecoveredComponent",
                is_residual_substitution => "ResidualSubstitution",
            ]);
        }
        return vec![format!(
            "kind={}",
            first(&[
                (error.is_empty_families(), "EmptyFamilies"),
                (error.is_invalid_identifier(), "InvalidIdentifier"),
                (error.is_invalid_index_count(), "InvalidIndexCount"),
                (error.is_invalid_integer_syntax(), "InvalidIntegerSyntax"),
                (error.is_kind_mismatch(), "KindMismatch"),
                (error.is_negative_index(), "NegativeIndex"),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssMediaConstructionError>() {
        return construction(
            first(&[
                (error.is_ambiguous_media_body(), "AmbiguousMediaBody"),
                (error.component_error().is_some(), "Component"),
                (
                    error.is_invalid_condition_grammar(),
                    "InvalidConditionGrammar",
                ),
                (error.is_invalid_name(), "InvalidName"),
                (error.is_invalid_prelude(), "InvalidPrelude"),
                (error.is_invalid_query_grammar(), "InvalidQueryGrammar"),
                (error.is_recovered_input(), "RecoveredInput"),
            ]),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssRuleConstructionError>() {
        // An `@import` construction failure carries its origin; a rule-list
        // construction failure has none.
        if let Some(value) = error.origin() {
            return construction(
                first(&[
                    (error.component_error().is_some(), "Component"),
                    (error.is_invalid_rule_grammar(), "InvalidRuleGrammar"),
                    (error.media_error().is_some(), "Media"),
                    (error.is_recovered_input(), "RecoveredInput"),
                    (error.serialization_error().is_some(), "Serialization"),
                    (error.supports_error().is_some(), "Supports"),
                    (error.is_worker_unavailable(), "WorkerUnavailable"),
                ]),
                value,
            );
        }
        return vec![format!(
            "kind={}",
            first(&[
                (error.is_capacity_overflow(), "CapacityOverflow"),
                (error.is_duplicate_namespace(), "DuplicateNamespace"),
                (error.is_invalid_boundary_anchor(), "InvalidBoundaryAnchor"),
                (error.is_invalid_placement(), "InvalidPlacement"),
                (error.is_invalid_prelude_order(), "InvalidPreludeOrder"),
                (
                    error.is_invalid_selector_identifier(),
                    "InvalidSelectorIdentifier"
                ),
                (error.is_namespace_mismatch(), "NamespaceMismatch"),
                (error.is_nesting_limit(), "NestingLimit"),
                (error.is_selector_nesting_limit(), "SelectorNestingLimit"),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssSupportsConstructionError>() {
        return construction(
            first(&[
                (error.component_error().is_some(), "Component"),
                (error.is_invalid_body_grammar(), "InvalidBodyGrammar"),
                (
                    error.is_invalid_condition_grammar(),
                    "InvalidConditionGrammar",
                ),
                (
                    error.is_invalid_declaration_grammar(),
                    "InvalidDeclarationGrammar",
                ),
                (error.is_invalid_name(), "InvalidName"),
                (error.is_recovered_input(), "RecoveredInput"),
                (error.is_worker_unavailable(), "WorkerUnavailable"),
            ]),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssConditionConstructionError>() {
        return construction(
            first(&[
                (error.component_error().is_some(), "Component"),
                (
                    error.is_invalid_condition_grammar(),
                    "InvalidConditionGrammar",
                ),
                (
                    error.is_invalid_media_feature_grammar(),
                    "InvalidMediaFeatureGrammar",
                ),
                (error.is_recovered_input(), "RecoveredInput"),
                (error.is_worker_unavailable(), "WorkerUnavailable"),
                (error.is_wrong_outer_component(), "WrongOuterComponent"),
            ]),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssContainerConstructionError>() {
        return construction(
            first(&[
                (error.component_error().is_some(), "Component"),
                (
                    error.is_invalid_condition_grammar(),
                    "InvalidConditionGrammar",
                ),
                (error.is_invalid_prelude_grammar(), "InvalidPreludeGrammar"),
                (error.is_worker_unavailable(), "WorkerUnavailable"),
            ]),
            error.origin(),
        );
    }
    if let Some(error) = error.downcast_ref::<CssImageConstructionError>() {
        // `<image-1d>` failures report their variant; the image and
        // gradient-stop construction failures have no accessor facts.
        let variant = first(&[
            (error.component_error().is_some(), "Component"),
            (error.is_empty_stripes(), "EmptyStripes"),
            (error.property_value_error().is_some(), "Grammar"),
            (error.recovered_origin().is_some(), "RecoveredComponent"),
        ]);
        if variant == "Future" {
            return Vec::new();
        }
        let mut facts = vec![format!("variant={variant}")];
        if let Some(inner) = error.component_error() {
            facts.extend(prefixed("component", link_facts(inner)));
        } else if let Some(inner) = error.property_value_error() {
            facts.extend(prefixed("grammar", link_facts(inner)));
        } else if let Some(value) = error.recovered_origin() {
            facts.push(format!("origin={}", origin(value)));
        }
        return facts;
    }
    if let Some(error) = error.downcast_ref::<CssNumericConstructionError>() {
        let kind = match numeric_failure(error) {
            NumericFailure::Component(kind) => format!("component:{}", component_error_kind(kind)),
            other => numeric_name(other).to_owned(),
        };
        return vec![
            format!("kind={kind}"),
            format!(
                "origin={}",
                error.origin().map_or_else(|| "-".to_owned(), origin)
            ),
            format!("path={}", error.path().map_or_else(|| "-".to_owned(), path)),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssColorConstructionError>() {
        // A colour scalar failure carries its origin; the other colour
        // construction failures have no accessor facts.
        let Some(value) = error.origin() else {
            return Vec::new();
        };
        return vec![
            format!(
                "kind={}",
                if error.component_error().is_some() {
                    "InvalidComponent"
                } else {
                    "OutOfRange"
                }
            ),
            format!("origin={}", origin(value)),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssTextConstructionError>() {
        let kind = match error.component_error() {
            Some(component) => format!(
                "component:{}",
                component_error_kind(component_failure(component))
            ),
            None if error.is_invalid_character() => "InvalidCharacter".to_owned(),
            None => "Future".to_owned(),
        };
        return vec![
            format!("kind={kind}"),
            format!("origin={}", origin(error.origin())),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssSelectorConstructionError>() {
        // A qualified-name failure has no accessor facts.
        if error.undeclared_namespace_prefix().is_some() {
            return Vec::new();
        }
        return vec![format!(
            "kind={}",
            first(&[
                (error.is_empty_complex_rest(), "EmptyComplexRest"),
                (error.is_empty_compound(), "EmptyCompound"),
                (error.is_empty_list(), "EmptyList"),
                (
                    error.attribute_value_error().is_some(),
                    "InvalidAttributeValue"
                ),
                (
                    error.invalid_type_position().is_some(),
                    "InvalidTypePosition"
                ),
                (
                    error.is_invalid_unknown_webkit_pseudo_element_name(),
                    "InvalidUnknownWebkitPseudoElementName",
                ),
                (error.is_modifier_without_value(), "ModifierWithoutValue"),
                (
                    error.non_terminal_pseudo_element().is_some(),
                    "NonTerminalPseudoElement"
                ),
                (error.serialization_error().is_some(), "Specified"),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssSerializationError>() {
        return vec![
            format!("kind={}", serialization_kind(serialization_failure(error))),
            format!(
                "origin={}",
                error.origin().map_or_else(|| "-".to_owned(), origin)
            ),
        ];
    }
    if let Some(error) = error.downcast_ref::<CssRuleSerializationError>() {
        let kind = match error.serialization_error() {
            Some(cause) if error.is_resource_limit() => {
                format!(
                    "resource:{}",
                    serialization_kind(serialization_failure(cause))
                )
            }
            Some(cause) => format!("value:{}", serialization_kind(serialization_failure(cause))),
            None => first(&[
                (
                    error.declaration_block_error().is_some(),
                    "DeclarationBlock",
                ),
                (error.unavailable_format().is_some(), "FormatUnavailable"),
                (error.undefined_source().is_some(), "SourceUndefined"),
            ])
            .to_owned(),
        };
        return vec![format!("kind={kind}")];
    }
    if let Some(error) = error.downcast_ref::<CssDeclarationBlockError>() {
        return vec![format!(
            "kind={}",
            first(&[
                (error.is_duplicate_terminal(), "DuplicateTerminal"),
                (error.expansion().is_some(), "Expansion"),
                (
                    error.invalid_keyframe_source().is_some(),
                    "InvalidKeyframeSource"
                ),
                (error.is_invalid_page_source(), "InvalidPageSource"),
                (
                    error.pending_footprint_property().is_some(),
                    "PendingFootprintUndetermined",
                ),
                (error.serialization().is_some(), "Serialization"),
                (
                    error.unresolved_environment_property().is_some(),
                    "UnresolvedEnvironment"
                ),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssPageBlockError>() {
        return vec![format!(
            "kind={}",
            first(&[
                (error.is_invalid_property_value(), "InvalidPropertyValue"),
                (error.is_property_not_applicable(), "PropertyNotApplicable"),
                (error.resource().is_some(), "Resource"),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssNormalizationError>() {
        return vec![format!(
            "kind={}",
            first(&[
                (error.exhausted_limit().is_some(), "LimitExceeded"),
                (error.expansion().is_some(), "UnsupportedDeclaration"),
            ])
        )];
    }
    if let Some(error) = error.downcast_ref::<CssKeyframeSelectorComparisonError>() {
        return vec![format!(
            "stage={}",
            match error.stage() {
                CssKeyframeSelectorComparisonStage::Comparison => "Comparison",
                CssKeyframeSelectorComparisonStage::LeftNormalization => "LeftNormalization",
                CssKeyframeSelectorComparisonStage::RightNormalization => "RightNormalization",
                _ => "Future",
            }
        )];
    }
    if let Some(error) = error.downcast_ref::<CssShapeConstructionError>() {
        // A path-data failure carries its origin; a command-list failure has
        // no accessor facts.
        let Some(value) = error.origin() else {
            return Vec::new();
        };
        return vec![
            format!(
                "kind={}",
                first(&[
                    (error.is_empty_path(), "EmptyPath"),
                    (error.is_expected_string(), "ExpectedString"),
                    (error.is_invalid_path_data(), "InvalidPathData"),
                    (error.is_recovered_input(), "RecoveredInput"),
                ])
            ),
            format!("origin={}", origin(value)),
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

fn numeric_name(kind: NumericFailure) -> &'static str {
    match kind {
        NumericFailure::Arity => "Arity",
        NumericFailure::EmptyValue => "EmptyValue",
        NumericFailure::IncompatibleTypes => "IncompatibleTypes",
        NumericFailure::InvalidArgumentType => "InvalidArgumentType",
        NumericFailure::MalformedExpression => "MalformedExpression",
        NumericFailure::MultipleValues => "MultipleValues",
        NumericFailure::OutOfRange => "OutOfRange",
        NumericFailure::RecoveredComponent => "RecoveredComponent",
        NumericFailure::ResourceLimit => "ResourceLimit",
        NumericFailure::RootDomainMismatch => "RootDomainMismatch",
        NumericFailure::SubstitutionRequired => "SubstitutionRequired",
        NumericFailure::UnknownFunction => "UnknownFunction",
        NumericFailure::Component(_) => "Component",
    }
}

fn serialization_kind(kind: SerializationFailure) -> &'static str {
    match kind {
        SerializationFailure::ByteLimit => "ByteLimit",
        SerializationFailure::CapacityOverflow => "CapacityOverflow",
        SerializationFailure::DepthLimit => "DepthLimit",
        SerializationFailure::InputNodeLimit => "InputNodeLimit",
        SerializationFailure::NamespaceBindingUnavailable => "NamespaceBindingUnavailable",
        SerializationFailure::ProjectionNodeLimit => "ProjectionNodeLimit",
        SerializationFailure::UnrepresentableValue => "UnrepresentableValue",
        SerializationFailure::UnserializableBoundary => "UnserializableBoundary",
    }
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
