use super::property_input::{Parser, Site};
use crate::error::property::ParseError;
use crate::numeric::NumericInputContext;
use crate::utf16_value::Control;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueRef as Component, CssComponentValues, CssNumericTokenKind, CssValueOrigin,
    CssValueTokenRef as ValueToken,
};
use cssparser::{BasicParseErrorKind, Token};

use crate::error::property::{CssFeatureId, Error, basic, invalid_component_value, invalid_syntax};
use crate::syntax::{
    CssAuthoredDeclarationValue, CssCustomPropertyDeclaredValue, CssCustomPropertyName,
    CssCustomPropertyValue,
};
use crate::validation::parse_global_keyword;

mod attr;

pub(super) static IMPLEMENTED_DECLARATIONS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.declaration.custom-property")];

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("baseline.value.substitution-dependent"),
    CssFeatureId::new("required.value.environment-substitution"),
    CssFeatureId::new("required.value.attribute-substitution"),
];

pub(crate) fn parse_custom_property_name(name: &str) -> Option<CssCustomPropertyName> {
    CssCustomPropertyName::from_ident_token(name)
}

#[derive(Clone, Copy)]
pub(super) enum SubstitutionContext {
    KnownProperty,
    CustomProperty,
    ExistingVarOnlyQuery,
    PermissiveStyleOperand,
}

#[derive(Default)]
struct Family {
    seen: bool,
    invalid: bool,
}
impl Family {
    fn qualifies(&self) -> bool {
        self.seen && !self.invalid
    }
    fn merge(&mut self, other: Self) {
        self.seen |= other.seen;
        self.invalid |= other.invalid;
    }
}
#[derive(Default)]
struct Summary {
    var: Family,
    env: Family,
    attr: Family,
    invalid_var: Option<Issue>,
}
struct Issue {
    origin: CssValueOrigin,
    path: Vec<usize>,
    reason: VarMismatch,
}
#[derive(Clone, Copy)]
enum VarMismatch {
    MissingName,
    InvalidName(usize),
    Unexpected(usize),
}
impl Summary {
    fn merge(&mut self, mut other: Self, control: Option<&Control<'_>>) {
        self.var.merge(other.var);
        self.env.merge(other.env);
        self.attr.merge(other.attr);
        if self.invalid_var.is_none() {
            self.invalid_var = other.invalid_var.take();
        }
        if let Some(issue) = other.invalid_var.take() {
            release(control, issue.path);
        }
    }
}

pub(crate) fn parse_custom_property_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssCustomPropertyDeclaredValue, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.expect_ident_cloned()
        && let Some(keyword) = parse_global_keyword(&ident)
        && input.is_exhausted()
    {
        return Ok(CssCustomPropertyDeclaredValue::Global(keyword));
    }
    input.reset(&state);
    let (authored, _) =
        collect_authored_declaration_value(input, numeric, SubstitutionContext::CustomProperty)?;
    Ok(CssCustomPropertyDeclaredValue::Value(
        CssCustomPropertyValue::new(authored),
    ))
}

pub(crate) fn collect_authored_declaration_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    context: SubstitutionContext,
) -> Result<(CssAuthoredDeclarationValue, bool), ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.position();
    let mut end = start;
    let mut summary = Summary::default();
    let mut invalid_var_error = None;
    loop {
        input.skip_whitespace();
        let state = input.state();
        let offset = input.position();
        let location = input.current_source_location();
        let group = if input.control().is_some() {
            match input.next_kind(false) {
                Ok(
                    crate::syntax_consumption::TokenKind::Function(_)
                    | crate::syntax_consumption::TokenKind::Opening(_),
                ) => true,
                Ok(_) => false,
                Err(error) => match error.kind {
                    BasicParseErrorKind::EndOfInput => break,
                    _ => return Err(basic(error)),
                },
            }
        } else {
            let token = match input.next() {
                Ok(token) => token.clone(),
                Err(error) => match error.kind {
                    BasicParseErrorKind::EndOfInput => break,
                    _ => return Err(basic(error)),
                },
            };
            if token.is_parse_error() {
                return Err(location.new_unexpected_token_error(token));
            }
            matches!(
                token,
                Token::Function(_)
                    | Token::ParenthesisBlock
                    | Token::SquareBracketBlock
                    | Token::CurlyBracketBlock
            )
        };
        if group {
            input.reset(&state);
            let collected = input.collect_component();
            if input
                .control()
                .is_some_and(|control| control.failure().is_some())
            {
                return Err(location.new_error(BasicParseErrorKind::EndOfInput));
            }
            let component = collected
                .map_err(crate::CssNumericConstructionError::component)
                .map_err(|error| {
                    let location = numeric.property_error_location(&error, location, offset);
                    if let Some(detail) = error.component_error() {
                        if matches!(
                            detail.kind,
                            CssComponentValueErrorKind::BadString
                                | CssComponentValueErrorKind::BadUrl
                                | CssComponentValueErrorKind::UnmatchedClosingDelimiter
                        ) {
                            return token_error(
                                numeric,
                                detail.origin(),
                                &[],
                                false,
                                offset,
                                location,
                            );
                        }
                        return invalid_component_value(location, detail.clone());
                    }
                    invalid_syntax(location)
                })?;
            let local = summarize_with_control(
                std::slice::from_ref(component.as_ref()),
                Some(numeric),
                true,
                matches!(context, SubstitutionContext::KnownProperty),
                input.control(),
            )
            .map_err(|error| {
                if let Some(control) = input.control() {
                    let _ = control.component_resource(error.kind);
                }
                invalid_component_value(
                    issue_location(numeric, error.origin(), &[], false, offset, location),
                    error,
                )
            })?;
            if invalid_var_error.is_none()
                && let Some(issue) = &local.invalid_var
            {
                let closing = matches!(issue.reason, VarMismatch::MissingName);
                let location = issue_location(
                    numeric,
                    &issue.origin,
                    &issue.path,
                    closing,
                    offset,
                    location,
                );
                invalid_var_error = Some((issue.reason, location));
            }
            summary.merge(local, input.control());
        }
        end = input.position();
    }
    let qualifies = summary.var.qualifies() || summary.env.qualifies() || summary.attr.qualifies();
    if summary.var.invalid
        && (matches!(context, SubstitutionContext::CustomProperty) || !qualifies)
        && let Some((reason, location)) = invalid_var_error
    {
        if let Some(issue) = summary.invalid_var.take() {
            release(input.control(), issue.path);
        }
        return Err(match reason {
            VarMismatch::MissingName => location.new_error(BasicParseErrorKind::EndOfInput),
            VarMismatch::InvalidName(_) => invalid_syntax(location),
            VarMismatch::Unexpected(_) if input.control().is_some() => {
                location.new_error(BasicParseErrorKind::EndOfInput)
            }
            VarMismatch::Unexpected(_) => match location.token() {
                Some(token) => location.new_unexpected_token_error(token),
                None => location.new_error(BasicParseErrorKind::EndOfInput),
            },
        });
    }
    if let Some(issue) = summary.invalid_var.take() {
        release(input.control(), issue.path);
    }
    Ok((
        CssAuthoredDeclarationValue::new(input.slice(start..end)),
        qualifies,
    ))
}

fn issue_location<'i>(
    _numeric: &NumericInputContext<'_>,
    origin: &CssValueOrigin,
    path: &[usize],
    closing: bool,
    root: Site<'i>,
    fallback: Site<'i>,
) -> Site<'i> {
    if path.is_empty() {
        return root.source_origin(origin).unwrap_or(fallback);
    }
    root.descendant(path, closing).unwrap_or(fallback)
}

fn token_error<'i>(
    numeric: &NumericInputContext<'_>,
    origin: &CssValueOrigin,
    path: &[usize],
    closing: bool,
    root: Site<'i>,
    location: Site<'i>,
) -> ParseError<'i, Error> {
    let site = issue_location(numeric, origin, path, closing, root, location);
    if site.control().is_some() {
        return site.new_error(BasicParseErrorKind::EndOfInput);
    }
    if let Some(token) = site.token() {
        site.new_unexpected_token_error(token)
    } else {
        site.new_error(BasicParseErrorKind::EndOfInput)
    }
}

fn trivia(component: &CssComponentValue) -> bool {
    matches!(
        component.view(),
        Component::Comment(_) | Component::Token(ValueToken::Whitespace(_))
    )
}
fn fallback_valid_with_control(
    items: &[CssComponentValue],
    control: Option<&Control<'_>>,
) -> Result<bool, CssComponentValueError> {
    for item in items {
        step(control)?;
        if matches!(
            item.view(),
            Component::Token(ValueToken::Semicolon | ValueToken::Delim('!'))
        ) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn next_argument<'a>(
    items: &'a [CssComponentValue],
    cursor: &mut usize,
    control: Option<&Control<'_>>,
) -> Result<Option<(usize, &'a CssComponentValue)>, CssComponentValueError> {
    while *cursor < items.len() {
        step(control)?;
        let index = *cursor;
        *cursor += 1;
        if !trivia(&items[index]) {
            return Ok(Some((index, &items[index])));
        }
    }
    Ok(None)
}

// A mismatch is local to its family. Traversal always continues into every child.
fn var_mismatch_with_control(
    items: &[CssComponentValue],
    control: Option<&Control<'_>>,
) -> Result<Option<VarMismatch>, CssComponentValueError> {
    let mut cursor = 0;
    let Some((index, name)) = next_argument(items, &mut cursor, control)? else {
        return Ok(Some(VarMismatch::MissingName));
    };
    step(control)?;
    if !matches!(name.view(), Component::Token(ValueToken::Ident(name)) if crate::CssCustomPropertyName::accepts_ident_token(name))
    {
        return Ok(Some(
            if matches!(name.view(), Component::Token(ValueToken::Ident(_))) {
                VarMismatch::InvalidName(index)
            } else {
                VarMismatch::Unexpected(index)
            },
        ));
    }
    if let Some((index, separator)) = next_argument(items, &mut cursor, control)? {
        if !matches!(separator.view(), Component::Token(ValueToken::Comma)) {
            return Ok(Some(VarMismatch::Unexpected(index)));
        }
        for (index, item) in items.iter().enumerate().skip(index + 1) {
            step(control)?;
            if matches!(
                item.view(),
                Component::Token(ValueToken::Semicolon | ValueToken::Delim('!'))
            ) {
                return Ok(Some(VarMismatch::Unexpected(index)));
            }
        }
    }
    Ok(None)
}

fn env_valid_with_control(
    items: &[CssComponentValue],
    numeric: Option<&NumericInputContext<'_>>,
    control: Option<&Control<'_>>,
) -> Result<bool, CssComponentValueError> {
    let mut cursor = 0;
    let Some((_, name)) = next_argument(items, &mut cursor, control)? else {
        step(control)?;
        return Ok(false);
    };
    if !matches!(name.view(), Component::Token(ValueToken::Ident(name))
        if parse_global_keyword(name).is_none() && !name.eq_ignore_ascii_case("default"))
    {
        return Ok(false);
    }
    while let Some((index, argument)) = next_argument(items, &mut cursor, control)? {
        step(control)?;
        match argument.view() {
            Component::Token(ValueToken::Comma) => {
                let fallback = &items[index + 1..];
                let mut non_comment = false;
                for item in fallback {
                    step(control)?;
                    non_comment |= !matches!(item.view(), Component::Comment(_));
                }
                return Ok(non_comment && fallback_valid_with_control(fallback, control)?);
            }
            Component::Token(ValueToken::Number(number))
                if number.kind() == CssNumericTokenKind::Integer =>
            {
                let nonnegative = if let Some(control) = control {
                    crate::integer_value::nonnegative_integer_literal_controlled(argument, control)
                        .map_err(|_| {
                            CssComponentValueError::new(
                                CssComponentValueErrorKind::CapacityOverflow,
                                argument.origin().clone(),
                            )
                        })?
                } else {
                    crate::CssNumericLiteral::<crate::CssInteger>::try_from_component(
                        argument.clone(),
                    )
                    .is_ok_and(|value| !value.is_negative())
                };
                if !nonnegative {
                    return Ok(false);
                }
            }
            Component::Function(function) if crate::numeric::is_math_function(function.name()) => {
                let result = if let Some(control) = control {
                    numeric
                        .unwrap_or(&NumericInputContext::Checked)
                        .check_component_controlled(
                            argument,
                            crate::numeric::CalculationRoot::Integer,
                            control,
                        )
                        .map_err(|_| {
                            CssComponentValueError::new(
                                CssComponentValueErrorKind::CapacityOverflow,
                                argument.origin().clone(),
                            )
                        })?
                } else if let Some(numeric) = numeric {
                    numeric.check_component(argument, crate::numeric::CalculationRoot::Integer)
                } else {
                    crate::numeric::check_component(
                        argument,
                        crate::numeric::CalculationRoot::Integer,
                    )
                };
                if let Err(error) = result {
                    if matches!(
                        error.kind(),
                        crate::CssNumericConstructionErrorKind::ResourceLimit
                    ) {
                        return Err(error.component_error().cloned().unwrap_or_else(|| {
                            CssComponentValueError::new(
                                CssComponentValueErrorKind::ComponentLimit,
                                argument.origin().clone(),
                            )
                        }));
                    }
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }
    step(control)?;
    Ok(true)
}

fn summarize(
    items: &[CssComponentValue],
    numeric: Option<&NumericInputContext<'_>>,
    classify_env: bool,
    classify_attr: bool,
) -> Result<Summary, CssComponentValueError> {
    summarize_with_control(items, numeric, classify_env, classify_attr, None)
}

fn summarize_with_control(
    items: &[CssComponentValue],
    numeric: Option<&NumericInputContext<'_>>,
    classify_env: bool,
    classify_attr: bool,
    control: Option<&Control<'_>>,
) -> Result<Summary, CssComponentValueError> {
    let mut summary = Summary::default();
    let mut stack = Vec::new();
    reserve(control, &mut stack, 1)?;
    stack.push((items, 0_usize));
    let mut path = Vec::new();
    while !stack.is_empty() {
        step(control)?;
        let Some((values, index)) = stack.last_mut() else {
            break;
        };
        let Some(component) = values.get(*index) else {
            stack.pop();
            path.pop();
            continue;
        };
        let current_index = *index;
        *index += 1;
        reserve(control, &mut path, 1)?;
        path.push(current_index);
        let children = match component.view() {
            Component::Function(function) => {
                match substitution_kind(function.name()) {
                    Some(SubstitutionKind::Var) => {
                        summary.var.seen = true;
                        if let Some(reason) =
                            var_mismatch_with_control(function.values().items(), control)?
                        {
                            summary.var.invalid = true;
                            if summary.invalid_var.is_none() {
                                let mut issue_path = Vec::new();
                                reserve(control, &mut issue_path, path.len() + 1)?;
                                for index in &path {
                                    step(control)?;
                                    issue_path.push(*index);
                                }
                                let origin = match reason {
                                    VarMismatch::MissingName => function.closing_origin(),
                                    VarMismatch::InvalidName(index)
                                    | VarMismatch::Unexpected(index) => {
                                        issue_path.push(index);
                                        function.values().items()[index].origin()
                                    }
                                };
                                summary.invalid_var = Some(Issue {
                                    origin: origin.clone(),
                                    path: issue_path,
                                    reason,
                                });
                            }
                        }
                    }
                    Some(SubstitutionKind::Env) if classify_env => {
                        summary.env.seen = true;
                        summary.env.invalid |=
                            !env_valid_with_control(function.values().items(), numeric, control)?;
                    }
                    _ => {}
                }
                if classify_attr && function.name().eq_ignore_ascii_case("attr") {
                    summary.attr.seen = true;
                    summary.attr.invalid |= !match control {
                        Some(_) => attr::valid_with_control(function.values().items(), control)?,
                        None => attr::valid(function.values().items())?,
                    };
                }
                Some(function.values().items())
            }
            Component::Block(block) => Some(block.values().items()),
            _ => None,
        };
        if let Some(children) = children {
            reserve(control, &mut stack, 1)?;
            stack.push((children, 0));
        } else {
            path.pop();
        }
    }
    release(control, stack);
    release(control, path);
    Ok(summary)
}

/// Query callers deliberately retain their existing var-only restrictions.
/// Permissive style values retain malformed env as ordinary declaration tokens.
pub(super) fn checked_variable_components(
    items: &[CssComponentValue],
    context: SubstitutionContext,
) -> Option<bool> {
    match context {
        SubstitutionContext::ExistingVarOnlyQuery | SubstitutionContext::PermissiveStyleOperand => {
            let summary = summarize(items, None, false, false).ok()?;
            if summary.var.invalid {
                None
            } else {
                Some(summary.var.qualifies())
            }
        }
        SubstitutionContext::KnownProperty | SubstitutionContext::CustomProperty => {
            unreachable!("source adapter owns property contexts")
        }
    }
}

/// An ordinary descriptor defers as a whole when either independent
/// substitution family qualifies. An invalid var() does not veto a valid env().
pub(crate) fn descriptor_substitution_qualifies(
    items: &[CssComponentValue],
    numeric: &NumericInputContext<'_>,
) -> Result<bool, CssComponentValueError> {
    let summary = summarize(items, Some(numeric), true, false)?;
    Ok(summary.var.qualifies() || summary.env.qualifies())
}

/// Descriptors without a separate var() exception defer only for valid env().
pub(crate) fn descriptor_environment_qualifies(
    items: &[CssComponentValue],
    numeric: &NumericInputContext<'_>,
) -> Result<bool, CssComponentValueError> {
    Ok(summarize(items, Some(numeric), true, false)?
        .env
        .qualifies())
}

#[derive(Clone, Copy)]
enum SubstitutionKind {
    Var,
    Env,
}
fn substitution_kind(name: &str) -> Option<SubstitutionKind> {
    if name.eq_ignore_ascii_case("var") {
        Some(SubstitutionKind::Var)
    } else if name.eq_ignore_ascii_case("env") {
        Some(SubstitutionKind::Env)
    } else {
        None
    }
}

pub(crate) fn contains_substitution(values: &CssComponentValues) -> bool {
    let mut stack = vec![values.items()];
    while let Some(items) = stack.pop() {
        for item in items {
            match item.view() {
                Component::Function(function) => {
                    if substitution_kind(function.name()).is_some()
                        || function.name().eq_ignore_ascii_case("attr")
                    {
                        return true;
                    }
                    stack.push(function.values().items());
                }
                Component::Block(block) => stack.push(block.values().items()),
                _ => {}
            }
        }
    }
    false
}

pub(crate) fn first_substitution_origin(values: &CssComponentValues) -> Option<CssValueOrigin> {
    let mut stack = vec![values.items().iter()];
    while let Some(items) = stack.last_mut() {
        if let Some(item) = items.next() {
            match item.view() {
                Component::Function(function) => {
                    if function.name().eq_ignore_ascii_case("var")
                        || function.name().eq_ignore_ascii_case("env")
                    {
                        return Some(item.origin().clone());
                    }
                    stack.push(function.values().items().iter());
                }
                Component::Block(block) => stack.push(block.values().items().iter()),
                _ => {}
            }
        } else {
            stack.pop();
        }
    }
    None
}

fn step(control: Option<&Control<'_>>) -> Result<(), CssComponentValueError> {
    match control {
        Some(control) => control.step().map_err(|_| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::CapacityOverflow,
                CssValueOrigin::Programmatic,
            )
        }),
        None => Ok(()),
    }
}
fn reserve<T>(
    control: Option<&Control<'_>>,
    values: &mut Vec<T>,
    additional: usize,
) -> Result<(), CssComponentValueError> {
    if let Some(control) = control {
        control.reserve(values, additional).map_err(|_| {
            CssComponentValueError::new(
                CssComponentValueErrorKind::CapacityOverflow,
                CssValueOrigin::Programmatic,
            )
        })
    } else {
        values.reserve(additional);
        Ok(())
    }
}
fn release<T>(control: Option<&Control<'_>>, values: Vec<T>) {
    let bytes = values.capacity().checked_mul(std::mem::size_of::<T>());
    drop(values);
    if let Some(control) = control {
        let Some(bytes) = bytes else {
            unreachable!("admitted symbolic bank")
        };
        control.released(bytes);
    }
}
