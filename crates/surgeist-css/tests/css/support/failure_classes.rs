//! Failure classes read from the public error predicates.
//!
//! Table-driven tests name the class each case must report; these functions
//! derive it from the error's `is_*` predicates and accessors and check that
//! exactly one class holds.

use std::fmt::Debug;

use surgeist_css::{CssComponentValueError, CssNumericConstructionError, CssSerializationError};

/// The class of a [`CssComponentValueError`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentFailure {
    BadString,
    BadUrl,
    UnmatchedClosingDelimiter,
    EmptyInput,
    TrailingInput,
    InvalidToken,
    InvalidIdentifier,
    InvalidNumber,
    InvalidDimension,
    InvalidFunction,
    InvalidString,
    UnserializableBoundary,
    NestingLimit,
    ComponentLimit,
    ByteLimit,
    CapacityOverflow,
    ParserUnavailable,
}

/// The one class whose predicate holds for `error`.
///
/// # Panics
///
/// Panics unless exactly one predicate holds.
pub fn component_failure(error: &CssComponentValueError) -> ComponentFailure {
    sole(
        error,
        [
            (error.is_bad_string(), ComponentFailure::BadString),
            (error.is_bad_url(), ComponentFailure::BadUrl),
            (
                error.is_unmatched_closing_delimiter(),
                ComponentFailure::UnmatchedClosingDelimiter,
            ),
            (error.is_empty_input(), ComponentFailure::EmptyInput),
            (error.is_trailing_input(), ComponentFailure::TrailingInput),
            (error.is_invalid_token(), ComponentFailure::InvalidToken),
            (
                error.is_invalid_identifier(),
                ComponentFailure::InvalidIdentifier,
            ),
            (error.is_invalid_number(), ComponentFailure::InvalidNumber),
            (
                error.is_invalid_dimension(),
                ComponentFailure::InvalidDimension,
            ),
            (
                error.is_invalid_function(),
                ComponentFailure::InvalidFunction,
            ),
            (error.is_invalid_string(), ComponentFailure::InvalidString),
            (
                error.is_unserializable_boundary(),
                ComponentFailure::UnserializableBoundary,
            ),
            (error.is_nesting_limit(), ComponentFailure::NestingLimit),
            (error.is_component_limit(), ComponentFailure::ComponentLimit),
            (error.is_byte_limit(), ComponentFailure::ByteLimit),
            (
                error.is_capacity_overflow(),
                ComponentFailure::CapacityOverflow,
            ),
            (
                error.is_parser_unavailable(),
                ComponentFailure::ParserUnavailable,
            ),
        ],
    )
}

/// The class of a [`CssNumericConstructionError`].
///
/// A rejected component that is not a resource limit reports its own class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericFailure {
    EmptyValue,
    MultipleValues,
    RecoveredComponent,
    SubstitutionRequired,
    MalformedExpression,
    UnknownFunction,
    Arity,
    InvalidArgumentType,
    IncompatibleTypes,
    RootDomainMismatch,
    OutOfRange,
    ResourceLimit,
    Component(ComponentFailure),
}

/// The one class whose predicate holds for `error`.
///
/// # Panics
///
/// Panics unless exactly one predicate or a non-resource component failure holds.
pub fn numeric_failure(error: &CssNumericConstructionError) -> NumericFailure {
    let unit = holding([
        (error.is_empty_value(), NumericFailure::EmptyValue),
        (error.is_multiple_values(), NumericFailure::MultipleValues),
        (
            error.is_recovered_component(),
            NumericFailure::RecoveredComponent,
        ),
        (
            error.is_substitution_required(),
            NumericFailure::SubstitutionRequired,
        ),
        (
            error.is_malformed_expression(),
            NumericFailure::MalformedExpression,
        ),
        (error.is_unknown_function(), NumericFailure::UnknownFunction),
        (error.is_arity(), NumericFailure::Arity),
        (
            error.is_invalid_argument_type(),
            NumericFailure::InvalidArgumentType,
        ),
        (
            error.is_incompatible_types(),
            NumericFailure::IncompatibleTypes,
        ),
        (
            error.is_root_domain_mismatch(),
            NumericFailure::RootDomainMismatch,
        ),
        (error.is_out_of_range(), NumericFailure::OutOfRange),
        (error.is_resource_limit(), NumericFailure::ResourceLimit),
    ]);
    let component = error
        .component_error()
        .filter(|_| !error.is_resource_limit());
    match (component, unit.as_slice()) {
        (Some(component), []) => NumericFailure::Component(component_failure(component)),
        (None, [class]) => *class,
        _ => panic!("expected exactly one failure class for {error:?}"),
    }
}

/// The class of a [`CssSerializationError`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationFailure {
    InputNodeLimit,
    ProjectionNodeLimit,
    ByteLimit,
    DepthLimit,
    CapacityOverflow,
    UnserializableBoundary,
    UnrepresentableValue,
    NamespaceBindingUnavailable,
}

/// The one class whose predicate holds for `error`.
///
/// # Panics
///
/// Panics unless exactly one predicate holds.
pub fn serialization_failure(error: &CssSerializationError) -> SerializationFailure {
    sole(
        error,
        [
            (
                error.is_input_node_limit(),
                SerializationFailure::InputNodeLimit,
            ),
            (
                error.is_projection_node_limit(),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (error.is_byte_limit(), SerializationFailure::ByteLimit),
            (error.is_depth_limit(), SerializationFailure::DepthLimit),
            (
                error.is_capacity_overflow(),
                SerializationFailure::CapacityOverflow,
            ),
            (
                error.is_unserializable_boundary(),
                SerializationFailure::UnserializableBoundary,
            ),
            (
                error.is_unrepresentable_value(),
                SerializationFailure::UnrepresentableValue,
            ),
            (
                error.is_namespace_binding_unavailable(),
                SerializationFailure::NamespaceBindingUnavailable,
            ),
        ],
    )
}

fn holding<C: Copy, const N: usize>(classes: [(bool, C); N]) -> Vec<C> {
    classes
        .into_iter()
        .filter_map(|(holds, class)| holds.then_some(class))
        .collect()
}

fn sole<E: Debug, C: Copy + Debug, const N: usize>(error: &E, classes: [(bool, C); N]) -> C {
    match holding(classes).as_slice() {
        [class] => *class,
        classes => panic!("expected exactly one failure class for {error:?}, found {classes:?}"),
    }
}
