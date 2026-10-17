//! Ordinary owned projection of the same finite expression grammar.
//!
//! Parent component admission and bottom-up nested-node construction remain in
//! numeric.rs. This adapter supplies those real leaves, drives each action and
//! projects the admitted flat records; it defines no arithmetic grammar/type
//! policy. Its ordinary buffers make no controlled-allocation promise.

use super::{bounded_expression as engine, *};

pub(super) fn parse<'a>(
    items: &'a [CssComponentValue],
    parser: &mut NumericParser<'a>,
    function: Option<(CssMathFunction, &'a CssComponentValue)>,
) -> Result<CssCalculationExpression> {
    let entry = function.map_or(engine::Entry::Sum, |(function, _)| {
        engine::Entry::Function {
            function,
            site: items.len(),
        }
    });
    let mut cursor = engine::ExpressionCursor::new(entry, 0);
    let mut records = Vec::new();
    let mut leaves: Vec<Option<CssCalculationExpression>> = Vec::new();
    let root = loop {
        match cursor
            .request()
            .copied()
            .expect("ordinary expression has a pending action")
        {
            engine::Request::ReadInput { ordinal } => {
                let kind = match items.get(ordinal) {
                    None => engine::InputKind::End,
                    Some(component) => match component.view() {
                        CssComponentValueRef::Comment(_) => engine::InputKind::Comment,
                        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_)) => {
                            engine::InputKind::Whitespace
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Comma) => {
                            engine::InputKind::Comma
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('+')) => {
                            engine::InputKind::Plus
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('-')) => {
                            engine::InputKind::Minus
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('*')) => {
                            engine::InputKind::Multiply
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('/')) => {
                            engine::InputKind::Divide
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Ident(name))
                            if function.is_some_and(|(f, _)| f == CssMathFunction::Clamp)
                                && name.eq_ignore_ascii_case("none") =>
                        {
                            engine::InputKind::None
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Ident(name))
                            if function.is_some_and(|(f, _)| f == CssMathFunction::Round)
                                && rounding_strategy(name).is_some() =>
                        {
                            engine::InputKind::Strategy(rounding_strategy(name).unwrap())
                        }
                        _ if !cursor
                            .admits_operand(ordinal)
                            .expect("actual pending input") =>
                        {
                            engine::InputKind::Invalid
                        }
                        _ => {
                            let value = parser.take(component)?;
                            let ty = value.ty;
                            let leaf = leaves.len();
                            leaves.push(Some(value));
                            engine::InputKind::Operand { leaf, ty }
                        }
                    },
                };
                cursor
                    .acknowledge_input(ordinal, Some(engine::Input::new(ordinal, kind)))
                    .expect("actual pending ordinary input");
            }
            engine::Request::AppendRecord { index, record } => {
                assert_eq!(index, records.len(), "ordinary expression record sequence");
                records.push(record);
                cursor
                    .acknowledge_append(index, true)
                    .expect("actual appended ordinary record");
            }
            engine::Request::ReadRecord { index, kind } => {
                let record = records.get(index).copied();
                assert!(
                    record.is_some_and(|record| record.kind() == kind),
                    "actual immutable ordinary record kind"
                );
                if let Some(engine::Record::Frame(frame)) = record {
                    assert!(
                        frame.parent().is_none_or(|parent| parent < index),
                        "actual immutable ordinary frame points backward"
                    );
                }
                cursor
                    .acknowledge_record(index, record)
                    .expect("actual immutable ordinary frame");
            }
            engine::Request::Compute => cursor
                .acknowledge_compute()
                .expect("actual ordinary computation"),
            engine::Request::Done { node, .. } => break node,
            engine::Request::Failed { kind, site } => {
                let component = if site == items.len() {
                    if kind == engine::FailureKind::MalformedExpression {
                        items
                            .iter()
                            .rev()
                            .find(|c| !trivia(c))
                            .or_else(|| function.map(|(_, component)| component))
                    } else {
                        function
                            .map(|(_, component)| component)
                            .or_else(|| items.iter().rev().find(|c| !trivia(c)))
                    }
                } else {
                    items.get(site)
                };
                let kind = match kind {
                    engine::FailureKind::MalformedExpression => {
                        CssNumericConstructionErrorKind::MalformedExpression
                    }
                    engine::FailureKind::Arity => CssNumericConstructionErrorKind::Arity,
                    engine::FailureKind::InvalidArgumentType => {
                        CssNumericConstructionErrorKind::InvalidArgumentType
                    }
                    engine::FailureKind::IncompatibleTypes => {
                        CssNumericConstructionErrorKind::IncompatibleTypes
                    }
                    engine::FailureKind::CapacityOverflow => {
                        CssNumericConstructionErrorKind::ResourceLimit
                    }
                };
                return Err(parser.error(kind, component));
            }
        }
    };
    project(
        &records,
        &mut leaves,
        root,
        items,
        function.map(|(_, component)| component),
    )
}

pub(super) fn rounding_strategy(name: &str) -> Option<CssRoundingStrategy> {
    if name.eq_ignore_ascii_case("nearest") {
        Some(CssRoundingStrategy::Nearest)
    } else if name.eq_ignore_ascii_case("up") {
        Some(CssRoundingStrategy::Up)
    } else if name.eq_ignore_ascii_case("down") {
        Some(CssRoundingStrategy::Down)
    } else if name.eq_ignore_ascii_case("to-zero") {
        Some(CssRoundingStrategy::ToZero)
    } else {
        None
    }
}

struct Projected {
    value: Option<CssCalculationExpression>,
    start: usize,
    end: usize,
}
fn take(values: &mut [Option<Projected>], index: usize) -> Projected {
    values[index].take().expect("grammar node has one parent")
}
fn project(
    records: &[engine::Record],
    leaves: &mut [Option<CssCalculationExpression>],
    root: usize,
    items: &[CssComponentValue],
    outer: Option<&CssComponentValue>,
) -> Result<CssCalculationExpression> {
    let mut values: Vec<Option<Projected>> = Vec::with_capacity(records.len());
    for record in records {
        let engine::Record::Node(node) = *record else {
            values.push(None);
            continue;
        };
        let (kind, origin, start, end) = match node.kind() {
            engine::NodeKind::Leaf { leaf } => {
                let value = leaves[leaf].take().expect("actual ordinary leaf used once");
                values.push(Some(Projected {
                    value: Some(value),
                    start: node.site(),
                    end: node.site() + 1,
                }));
                continue;
            }
            engine::NodeKind::Sum { last, count } => {
                let mut terms = Vec::with_capacity(count);
                let mut next = Some(last);
                while let Some(index) = next {
                    let engine::Record::Term(term) = records[index] else {
                        unreachable!("actual term link");
                    };
                    let (previous, _, operand, operator, _) = term.parts();
                    let mut value = take(&mut values, operand);
                    terms.push((
                        operator,
                        value.value.take().unwrap(),
                        value.start,
                        value.end,
                    ));
                    next = previous;
                }
                terms.reverse();
                let origin = terms[0].1.origin.clone();
                let start = terms[0].2;
                let end = terms.last().unwrap().3;
                (
                    NodeKind::Sum(
                        terms
                            .into_iter()
                            .map(|(op, value, _, _)| (op, value))
                            .collect(),
                    ),
                    origin,
                    start,
                    end,
                )
            }
            engine::NodeKind::Product { last, count } => {
                let mut factors = Vec::with_capacity(count);
                let mut next = Some(last);
                while let Some(index) = next {
                    let engine::Record::Factor(factor) = records[index] else {
                        unreachable!("actual factor link");
                    };
                    let (previous, _, operand, operator, _) = factor.parts();
                    let mut value = take(&mut values, operand);
                    factors.push((
                        operator,
                        value.value.take().unwrap(),
                        value.start,
                        value.end,
                    ));
                    next = previous;
                }
                factors.reverse();
                let origin = factors[0].1.origin.clone();
                let start = factors[0].2;
                let end = factors.last().unwrap().3;
                (
                    NodeKind::Product(
                        factors
                            .into_iter()
                            .map(|(op, value, _, _)| (op, value))
                            .collect(),
                    ),
                    origin,
                    start,
                    end,
                )
            }
            engine::NodeKind::Function {
                function,
                last,
                count,
                strategy,
            } => {
                let mut arguments = Vec::with_capacity(count);
                let mut next = Some(last);
                while let Some(index) = next {
                    let engine::Record::Argument(argument) = records[index] else {
                        unreachable!("actual argument link");
                    };
                    let (previous, _, operand) = argument.parts();
                    arguments.push(operand.map(|index| take(&mut values, index).value.unwrap()));
                    next = previous;
                }
                arguments.reverse();
                let component = outer.expect("ordinary root function");
                (
                    NodeKind::Function {
                        function,
                        args: arguments,
                        strategy,
                    },
                    component.origin().clone(),
                    0,
                    items.len(),
                )
            }
            engine::NodeKind::Group { .. } => {
                unreachable!("ordinary nested components are real prebuilt operands")
            }
        };
        values.push(Some(Projected {
            value: Some(CssCalculationExpression {
                admits_pure_context: kind.admits_pure_context(),
                kind,
                ty: node.numeric_type(),
                origin,
                components: None,
                syntax: direct_syntax(&items[start..end]),
                closing: None,
            }),
            start,
            end,
        }));
    }
    Ok(take(&mut values, root).value.unwrap())
}
