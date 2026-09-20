use crate::numeric::NumericInputContext;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueRef as Component, CssComponentValues, CssNumericTokenKind, CssValueOrigin,
    CssValueTokenRef as ValueToken,
};
use cssparser::{BasicParseErrorKind, ParseError, Parser, Token};

use crate::error::{
    CssFeatureId, Error, basic, invalid_component_value, invalid_syntax, unexpected_token_at,
};
use crate::syntax::{
    CssAuthoredDeclarationValue, CssCustomPropertyDeclaredValue, CssCustomPropertyName,
    CssCustomPropertyValue,
};
use crate::validation::parse_global_keyword;

pub(super) static IMPLEMENTED_DECLARATIONS: &[CssFeatureId] =
    &[CssFeatureId::new("baseline.declaration.custom-property")];

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("baseline.value.substitution-dependent"),
    CssFeatureId::new("required.value.environment-substitution"),
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
    fn merge(&mut self, other: Self) {
        self.var.merge(other.var);
        self.env.merge(other.env);
        if self.invalid_var.is_none() {
            self.invalid_var = other.invalid_var;
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
    {
        if input.is_exhausted() {
            return Ok(CssCustomPropertyDeclaredValue::Global(keyword));
        }
        return Err(invalid_syntax(
            input.current_source_location(),
            "CSS global keyword must be the entire custom property value",
        ));
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
        let offset = input.position().byte_index();
        let location = input.current_source_location();
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
        if matches!(
            token,
            Token::Function(_)
                | Token::ParenthesisBlock
                | Token::SquareBracketBlock
                | Token::CurlyBracketBlock
        ) {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|error| {
                let location = numeric.error_location(&error, location, offset);
                if let Some(detail) = error.component_error() {
                    if matches!(
                        detail.kind(),
                        CssComponentValueErrorKind::BadString
                            | CssComponentValueErrorKind::BadUrl
                            | CssComponentValueErrorKind::UnmatchedClosingDelimiter
                    ) {
                        return token_error(numeric, detail.origin(), &[], false, offset, location);
                    }
                    return invalid_component_value(location, detail.clone());
                }
                invalid_syntax(location, "invalid substitution component")
            })?;
            let local = summarize(std::slice::from_ref(&component), Some(numeric), true).map_err(
                |error| {
                    invalid_component_value(
                        issue_location(numeric, error.origin(), &[], false, offset, location),
                        error,
                    )
                },
            )?;
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
                invalid_var_error = Some(match issue.reason {
                    VarMismatch::MissingName => location.new_error(BasicParseErrorKind::EndOfInput),
                    VarMismatch::InvalidName(_) => {
                        invalid_syntax(location, "invalid custom property name")
                    }
                    VarMismatch::Unexpected(_) => {
                        token_error(numeric, &issue.origin, &issue.path, false, offset, location)
                    }
                });
            }
            summary.merge(local);
        }
        end = input.position();
    }
    let qualifies = summary.var.qualifies() || summary.env.qualifies();
    if summary.var.invalid
        && (matches!(context, SubstitutionContext::CustomProperty) || !qualifies)
        && let Some(error) = invalid_var_error
    {
        return Err(error);
    }
    Ok((
        CssAuthoredDeclarationValue::new(input.slice(start..end)),
        qualifies,
    ))
}

fn issue_offset(
    numeric: &NumericInputContext<'_>,
    origin: &CssValueOrigin,
    path: &[usize],
    closing: bool,
    root_offset: usize,
) -> Option<usize> {
    match numeric {
        NumericInputContext::Parsed(source) => {
            let parsed = match origin {
                CssValueOrigin::Parsed(parsed) => parsed,
                CssValueOrigin::ImplicitClosure { at, .. } => at,
                _ => return None,
            };
            source
                .same_snapshot(parsed.source())
                .then(|| parsed.span().start().byte_offset().value())
        }
        NumericInputContext::Components(_, serialized) => {
            let base = serialized.component_path_at(root_offset)?;
            let mut complete = base.to_vec();
            complete.extend_from_slice(path.get(1..).unwrap_or_default());
            if closing {
                // The retained complete component range includes its one-byte closing token.
                serialized.component_end_for_path(&complete)?.checked_sub(1)
            } else {
                serialized.component_offset_for_path(&complete)
            }
        }
    }
}

fn input_css<'a>(numeric: &'a NumericInputContext<'_>) -> &'a str {
    match numeric {
        NumericInputContext::Parsed(source) => source.as_str(),
        NumericInputContext::Components(_, serialized) => serialized.as_css(),
    }
}

fn issue_location(
    numeric: &NumericInputContext<'_>,
    origin: &CssValueOrigin,
    path: &[usize],
    closing: bool,
    root_offset: usize,
    fallback: cssparser::SourceLocation,
) -> cssparser::SourceLocation {
    let Some(offset) = issue_offset(numeric, origin, path, closing, root_offset) else {
        return fallback;
    };
    let mut storage = cssparser::ParserInput::new(&input_css(numeric)[..offset]);
    let mut parser = Parser::new(&mut storage);
    while parser.next_including_whitespace_and_comments().is_ok() {}
    parser.current_source_location()
}

fn token_error<'i>(
    numeric: &NumericInputContext<'_>,
    origin: &CssValueOrigin,
    path: &[usize],
    closing: bool,
    root_offset: usize,
    location: cssparser::SourceLocation,
) -> ParseError<'i, Error> {
    if let Some(offset) = issue_offset(numeric, origin, path, closing, root_offset) {
        let source = input_css(numeric);
        let mut storage = cssparser::ParserInput::new(&source[offset..]);
        let mut parser = Parser::new(&mut storage);
        if let Ok(token) = parser.next_including_whitespace_and_comments() {
            return location.new_custom_error(unexpected_token_at(source, offset, token));
        }
    }
    location.new_error(BasicParseErrorKind::EndOfInput)
}

fn trivia(component: &CssComponentValue) -> bool {
    matches!(
        component.view(),
        Component::Comment(_) | Component::Token(ValueToken::Whitespace(_))
    )
}
fn fallback_valid(items: &[CssComponentValue]) -> bool {
    !items.iter().any(|item| {
        matches!(
            item.view(),
            Component::Token(ValueToken::Semicolon | ValueToken::Delim('!'))
        )
    })
}

// A mismatch is local to its family. Traversal always continues into every child.
fn var_mismatch(items: &[CssComponentValue]) -> Option<VarMismatch> {
    let mut args = items.iter().enumerate().filter(|(_, item)| !trivia(item));
    let Some((index, name)) = args.next() else {
        return Some(VarMismatch::MissingName);
    };
    if !matches!(name.view(), Component::Token(ValueToken::Ident(name)) if parse_custom_property_name(name).is_some())
    {
        return Some(
            if matches!(name.view(), Component::Token(ValueToken::Ident(_))) {
                VarMismatch::InvalidName(index)
            } else {
                VarMismatch::Unexpected(index)
            },
        );
    }
    if let Some((index, separator)) = args.next() {
        if !matches!(separator.view(), Component::Token(ValueToken::Comma)) {
            return Some(VarMismatch::Unexpected(index));
        }
        if !fallback_valid(&items[index + 1..]) {
            return items
                .iter()
                .enumerate()
                .skip(index + 1)
                .find_map(|(i, item)| {
                    matches!(
                        item.view(),
                        Component::Token(ValueToken::Semicolon | ValueToken::Delim('!'))
                    )
                    .then_some(VarMismatch::Unexpected(i))
                });
        }
    }
    None
}

fn env_valid(
    items: &[CssComponentValue],
    numeric: Option<&NumericInputContext<'_>>,
) -> Result<bool, CssComponentValueError> {
    let mut args = items.iter().enumerate().filter(|(_, item)| !trivia(item));
    let Some((_, name)) = args.next() else {
        return Ok(false);
    };
    if !matches!(name.view(), Component::Token(ValueToken::Ident(name))
        if parse_global_keyword(name).is_none() && !name.eq_ignore_ascii_case("default"))
    {
        return Ok(false);
    }
    for (index, argument) in args {
        match argument.view() {
            Component::Token(ValueToken::Comma) => {
                let fallback = &items[index + 1..];
                return Ok(fallback
                    .iter()
                    .any(|item| !matches!(item.view(), Component::Comment(_)))
                    && fallback_valid(fallback));
            }
            Component::Token(ValueToken::Number(number))
                if number.kind() == CssNumericTokenKind::Integer =>
            {
                let nonnegative =
                    match crate::integer_value::admit_integer_literal(argument.clone()) {
                        Ok(crate::CssIntegerValue::Literal(value)) => value >= 0,
                        // The shared exact owner already reduces every spelling of
                        // signed zero to Literal(0). A negative ExactLiteral is nonzero.
                        Ok(crate::CssIntegerValue::ExactLiteral(value)) => {
                            !value.numeric().representation().starts_with('-')
                        }
                        _ => false,
                    };
                if !nonnegative {
                    return Ok(false);
                }
            }
            Component::Function(function) if crate::numeric::is_math_function(function.name()) => {
                let values = CssComponentValues::try_new(vec![argument.clone()])?;
                let result = if let Some(numeric) = numeric {
                    numeric.admit(values, crate::numeric::CalculationRoot::Integer)
                } else {
                    crate::numeric::construct(
                        values,
                        crate::numeric::CalculationRoot::Integer,
                        crate::CssComponentValueLimits::default(),
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
    Ok(true)
}

fn summarize(
    items: &[CssComponentValue],
    numeric: Option<&NumericInputContext<'_>>,
    classify_env: bool,
) -> Result<Summary, CssComponentValueError> {
    let mut summary = Summary::default();
    let mut stack = vec![(items, 0_usize)];
    let mut path = Vec::new();
    while let Some((values, index)) = stack.last_mut() {
        let Some(component) = values.get(*index) else {
            stack.pop();
            path.pop();
            continue;
        };
        let current_index = *index;
        *index += 1;
        path.push(current_index);
        let children = match component.view() {
            Component::Function(function) => {
                match substitution_kind(function.name()) {
                    Some(SubstitutionKind::Var) => {
                        summary.var.seen = true;
                        if let Some(reason) = var_mismatch(function.values().items()) {
                            summary.var.invalid = true;
                            if summary.invalid_var.is_none() {
                                let mut issue_path = path.clone();
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
                        summary.env.invalid |= !env_valid(function.values().items(), numeric)?;
                    }
                    _ => {}
                }
                Some(function.values().items())
            }
            Component::Block(block) => Some(block.values().items()),
            _ => None,
        };
        if let Some(children) = children {
            stack.push((children, 0));
        } else {
            path.pop();
        }
    }
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
            let summary = summarize(items, None, false).ok()?;
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
                    if substitution_kind(function.name()).is_some() {
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
