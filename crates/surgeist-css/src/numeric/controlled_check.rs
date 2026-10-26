//! Borrowed env-index admission through the original finite math grammar.
//! Input was issued by the original checked component owner. This adapter
//! retains only types/lengths, not an expression projection or cloned syntax.

use super::{bounded_expression as engine, *};
use crate::utf16_value::Control;
use std::mem::size_of;

#[derive(Clone, Copy)]
struct Fact<'a> {
    component: &'a CssComponentValue,
    ty: CssNumericType,
    length: usize,
}
struct Scratch<'a> {
    pending: Vec<(&'a CssComponentValue, bool)>,
    ready: Vec<Fact<'a>>,
    records: Vec<engine::Record>,
    leaves: Vec<Fact<'a>>,
    lengths: Vec<Option<usize>>,
    cursor: engine::ExpressionCursor,
}
type Checked<T> = std::result::Result<Result<T>, ()>;

pub(super) fn check(
    component: &CssComponentValue,
    policy: AdmissionPolicy,
    control: &Control<'_>,
) -> Checked<()> {
    let baseline = control.live().bytes();
    let result = (|| {
        control.step()?;
        control.add(size_of::<Scratch<'_>>())?;
        let mut scratch = Scratch {
            pending: Vec::new(),
            ready: Vec::new(),
            records: Vec::new(),
            leaves: Vec::new(),
            lengths: Vec::new(),
            cursor: engine::ExpressionCursor::new(engine::Entry::Sum, 0),
        };
        // The producer already qualified count/depth/serialized input. Default
        // numeric policy adds no length/count ceiling. Strict closure rejection
        // still precedes all numeric grammar, just as in validate_components.
        control.reserve(&mut scratch.pending, 1)?;
        scratch.pending.push((component, false));
        while let Some((current, _)) = scratch.pending.pop() {
            control.step()?;
            if let Some(children) = current.child_values() {
                if policy == AdmissionPolicy::Strict
                    && matches!(
                        current.implicit_termination_origin(),
                        Some(CssValueOrigin::ImplicitClosure { .. })
                    )
                {
                    return Ok(Err(error(
                        CssNumericConstructionErrorKind::RecoveredComponent,
                        current,
                    )));
                }
                for child in children.items().iter().rev() {
                    control.step()?;
                    control.reserve(&mut scratch.pending, 1)?;
                    scratch.pending.push((child, false));
                }
            }
        }
        if !matches!(
            component.view(),
            CssComponentValueRef::Function(_)
                | CssComponentValueRef::Token(
                    CssValueTokenRef::Number(_)
                        | CssValueTokenRef::Percentage(_)
                        | CssValueTokenRef::Dimension { .. }
                )
        ) {
            return Ok(Err(error(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                component,
            )));
        }
        control.reserve(&mut scratch.pending, 1)?;
        scratch.pending.push((component, false));
        while let Some((current, visited)) = scratch.pending.pop() {
            control.step()?;
            if visited {
                let fact = match node(current, &mut scratch, control)? {
                    Ok(fact) => fact,
                    Err(error) => return Ok(Err(error)),
                };
                control.reserve(&mut scratch.ready, 1)?;
                scratch.ready.push(fact);
            } else {
                control.reserve(&mut scratch.pending, 1)?;
                scratch.pending.push((current, true));
                if let Some(children) = current.child_values() {
                    for child in children.items().iter().rev() {
                        control.step()?;
                        if child.child_values().is_some() {
                            control.reserve(&mut scratch.pending, 1)?;
                            scratch.pending.push((child, false));
                        }
                    }
                }
            }
        }
        let fact = scratch.ready.last().expect("actual root fact");
        let literal = matches!(
            component.view(),
            CssComponentValueRef::Token(
                CssValueTokenRef::Number(_)
                    | CssValueTokenRef::Dimension { .. }
                    | CssValueTokenRef::Percentage(_)
            )
        );
        if (literal
            && !matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(n)) if n.kind() == CssNumericTokenKind::Integer))
            || !CalculationRoot::Integer.accepts(fact.ty)
        {
            return Ok(Err(error(
                CssNumericConstructionErrorKind::RootDomainMismatch,
                component,
            )));
        }
        Ok(Ok(()))
    })();
    // Scratch and all actual banks dropped before infallible release. Errors
    // retain only the original component's shared origin, never an owned path.
    control.released(
        control
            .live()
            .bytes()
            .checked_sub(baseline)
            .expect("numeric caller baseline"),
    );
    result
}

fn error(
    kind: CssNumericConstructionErrorKind,
    component: &CssComponentValue,
) -> CssNumericConstructionError {
    CssNumericConstructionError::at(kind, Some(component))
}
fn text(value: &str, control: &Control<'_>) -> std::result::Result<(), ()> {
    for _ in value.chars() {
        control.step()?;
    }
    Ok(())
}
fn leaf<'a>(component: &'a CssComponentValue, control: &Control<'_>) -> Checked<Fact<'a>> {
    control.step()?;
    let length = match component.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Number(number)) => {
            Some(number.representation().len())
        }
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => {
            number.representation().len().checked_add(1)
        }
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
            text(unit, control)?;
            number.representation().len().checked_add(unit.len())
        }
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => {
            text(name, control)?;
            let Some(constant) = CssNumericConstant::parse(name) else {
                return Ok(Err(error(
                    CssNumericConstructionErrorKind::InvalidArgumentType,
                    component,
                )));
            };
            Some(constant.name().len())
        }
        _ => {
            return Ok(Err(error(
                CssNumericConstructionErrorKind::MalformedExpression,
                component,
            )));
        }
    };
    let ty = match intrinsic_token_type(component).expect("classified intrinsic token") {
        Ok(ty) => ty,
        Err(error) => return Ok(Err(error)),
    };
    let Some(length) = length else {
        return Ok(Err(error(
            CssNumericConstructionErrorKind::ResourceLimit,
            component,
        )));
    };
    Ok(Ok(Fact {
        component,
        ty,
        length,
    }))
}

fn node<'a>(
    component: &'a CssComponentValue,
    scratch: &mut Scratch<'a>,
    control: &Control<'_>,
) -> Checked<Fact<'a>> {
    let (items, function, group) = match component.view() {
        CssComponentValueRef::Function(function) => {
            text(function.name(), control)?;
            if function.name().eq_ignore_ascii_case("var") {
                return Ok(Err(error(
                    CssNumericConstructionErrorKind::SubstitutionRequired,
                    component,
                )));
            }
            let Some(kind) = CssMathFunction::parse(function.name()) else {
                return Ok(Err(error(
                    CssNumericConstructionErrorKind::UnknownFunction,
                    component,
                )));
            };
            (function.values().items(), Some(kind), false)
        }
        CssComponentValueRef::Block(block) if block.kind() == CssBlockKind::Parenthesis => {
            (block.values().items(), None, true)
        }
        _ => return leaf(component, control),
    };
    scratch.records.clear();
    scratch.leaves.clear();
    scratch.lengths.clear();
    scratch.cursor = engine::ExpressionCursor::new(
        function.map_or(engine::Entry::Sum, |function| engine::Entry::Function {
            function,
            site: items.len(),
        }),
        0,
    );
    let (root, ty) = loop {
        control.step()?;
        match scratch
            .cursor
            .request()
            .copied()
            .expect("actual math request")
        {
            engine::Request::ReadInput { ordinal } => {
                let kind = match items.get(ordinal) {
                    None => engine::InputKind::End,
                    Some(current) => match current.view() {
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
                            if function == Some(CssMathFunction::Clamp)
                                && name.eq_ignore_ascii_case("none") =>
                        {
                            engine::InputKind::None
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Ident(name))
                            if function == Some(CssMathFunction::Round)
                                && ordinary_expression::rounding_strategy(name).is_some() =>
                        {
                            engine::InputKind::Strategy(
                                ordinary_expression::rounding_strategy(name).unwrap(),
                            )
                        }
                        _ if !scratch
                            .cursor
                            .admits_operand(ordinal)
                            .expect("actual ordinal") =>
                        {
                            engine::InputKind::Invalid
                        }
                        _ => {
                            let fact = if current.child_values().is_some() {
                                let mut found = None;
                                for fact in &scratch.ready {
                                    control.step()?;
                                    if std::ptr::eq(fact.component, current) {
                                        found = Some(*fact);
                                        break;
                                    }
                                }
                                found.expect("postorder nested fact")
                            } else {
                                match leaf(current, control)? {
                                    Ok(fact) => fact,
                                    Err(error) => return Ok(Err(error)),
                                }
                            };
                            let leaf = scratch.leaves.len();
                            control.reserve(&mut scratch.leaves, 1)?;
                            scratch.leaves.push(fact);
                            engine::InputKind::Operand { leaf, ty: fact.ty }
                        }
                    },
                };
                scratch
                    .cursor
                    .acknowledge_input(ordinal, Some(engine::Input::new(ordinal, kind)))
                    .expect("actual input");
            }
            engine::Request::AppendRecord { index, record } => {
                assert_eq!(index, scratch.records.len());
                control.reserve(&mut scratch.records, 1)?;
                scratch.records.push(record);
                scratch
                    .cursor
                    .acknowledge_append(index, true)
                    .expect("actual append");
            }
            engine::Request::ReadRecord { index, .. } => scratch
                .cursor
                .acknowledge_record(index, scratch.records.get(index).copied())
                .expect("actual record"),
            engine::Request::Compute => scratch
                .cursor
                .acknowledge_compute()
                .expect("actual compute"),
            engine::Request::Done { node, ty, .. } => break (node, ty),
            engine::Request::Failed { kind, site } => {
                let mut last = None;
                if site == items.len()
                    && (kind == engine::FailureKind::MalformedExpression || function.is_none())
                {
                    for current in items.iter().rev() {
                        control.step()?;
                        if !trivia(current) {
                            last = Some(current);
                            break;
                        }
                    }
                }
                let current = items.get(site).or(last).unwrap_or(component);
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
                return Ok(Err(error(kind, current)));
            }
        }
    };
    let length = match canonical_length(scratch, root, control)? {
        Some(length) => length,
        None => {
            return Ok(Err(error(
                CssNumericConstructionErrorKind::ResourceLimit,
                component,
            )));
        }
    };
    let Some(length) = length.checked_add(if group { 2 } else { 0 }) else {
        return Ok(Err(error(
            CssNumericConstructionErrorKind::ResourceLimit,
            component,
        )));
    };
    Ok(Ok(Fact {
        component,
        ty,
        length,
    }))
}

fn canonical_length(
    scratch: &mut Scratch<'_>,
    root: usize,
    control: &Control<'_>,
) -> std::result::Result<Option<usize>, ()> {
    for record in &scratch.records {
        control.step()?;
        let length = match *record {
            engine::Record::Node(node) => match node.kind() {
                engine::NodeKind::Leaf { leaf } => Some(scratch.leaves[leaf].length),
                engine::NodeKind::Group { child } => {
                    scratch.lengths[child].and_then(|length| length.checked_add(2))
                }
                engine::NodeKind::Sum { last, count }
                | engine::NodeKind::Product { last, count } => {
                    let mut total = count.checked_sub(1).and_then(|count| count.checked_mul(3));
                    let mut next = Some(last);
                    while let Some(index) = next {
                        control.step()?;
                        let (previous, operand) = match scratch.records[index] {
                            engine::Record::Term(term) => {
                                let (previous, _, operand, _, _) = term.parts();
                                (previous, operand)
                            }
                            engine::Record::Factor(factor) => {
                                let (previous, _, operand, _, _) = factor.parts();
                                (previous, operand)
                            }
                            _ => unreachable!("actual math list"),
                        };
                        total = total.and_then(|total| {
                            scratch.lengths[operand].and_then(|length| total.checked_add(length))
                        });
                        next = previous;
                    }
                    total
                }
                engine::NodeKind::Function {
                    function,
                    last,
                    count,
                    strategy,
                } => {
                    let mut total = function
                        .as_str()
                        .len()
                        .checked_add(2)
                        .and_then(|length| {
                            count
                                .checked_sub(1)
                                .and_then(|count| count.checked_mul(2))
                                .and_then(|separators| length.checked_add(separators))
                        })
                        .and_then(|length| {
                            length.checked_add(
                                strategy.map_or(0, |strategy| strategy.name().len() + 2),
                            )
                        });
                    let mut next = Some(last);
                    while let Some(index) = next {
                        control.step()?;
                        let engine::Record::Argument(argument) = scratch.records[index] else {
                            unreachable!("actual argument")
                        };
                        let (previous, _, operand) = argument.parts();
                        let added = operand.map_or(Some(4), |operand| scratch.lengths[operand]);
                        total = total
                            .and_then(|total| added.and_then(|length| total.checked_add(length)));
                        next = previous;
                    }
                    total
                }
            },
            _ => None,
        };
        control.reserve(&mut scratch.lengths, 1)?;
        scratch.lengths.push(length);
    }
    Ok(scratch.lengths[root])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CssUtf16InputLimits, CssUtf16PropertyValueLimits, CssUtf16ValueStorage};
    use std::cell::Cell;

    fn limits() -> CssUtf16PropertyValueLimits {
        CssUtf16PropertyValueLimits {
            input: CssUtf16InputLimits::default(),
            syntax: CssComponentValueLimits::default(),
            max_work: 1_000_000,
            max_live_bytes: 16_000_000,
            max_retained_bytes: 16_000_000,
        }
    }

    #[test]
    fn symbolic_integer_domain_uses_original_math_grammar_without_evaluating_indices() {
        // Integer calculations are symbolic number-typed expressions. Their
        // spelling need not already be integral, finite, or nonnegative.
        for source in [
            "calc(2.5)",
            "calc(-2.5)",
            "calc(NaN)",
            "calc(infinity)",
            "calc(1 / 0)",
            "calc((2px / 1px) + max(1, 2))",
            "clamp(none, 2.5, none)",
            "round(up, 2.5, 1)",
            "sin(30deg)",
        ] {
            let values = crate::parse_component_values(source).unwrap();
            let component = &values.items()[0];
            let original = component.origin().clone();
            let admit = |_| Ok(());
            let released = Cell::new(usize::MAX);
            let release = |usage: CssUtf16ValueStorage| released.set(usage.bytes());
            let control = Control::new(limits(), &admit, &release);
            assert_eq!(
                NumericInputContext::Checked.check_component_controlled(
                    component,
                    CalculationRoot::Integer,
                    &control
                ),
                Ok(Ok(())),
                "{source}"
            );
            assert_eq!(control.live().bytes(), 0, "{source}");
            assert_eq!(released.get(), 0, "{source}");
            assert_eq!(component.origin(), &original);
            NumericInputContext::Checked
                .check_component(component, CalculationRoot::Integer)
                .unwrap();
        }
    }

    #[test]
    fn invalid_integer_math_preserves_actual_error_kind_and_original_component_origin() {
        for (source, expected) in [
            (
                "calc(1px)",
                CssNumericConstructionErrorKind::RootDomainMismatch,
            ),
            (
                "calc(1 + 2px)",
                CssNumericConstructionErrorKind::IncompatibleTypes,
            ),
            (
                "calc(1 + )",
                CssNumericConstructionErrorKind::MalformedExpression,
            ),
            ("min()", CssNumericConstructionErrorKind::Arity),
            (
                "future(1)",
                CssNumericConstructionErrorKind::UnknownFunction,
            ),
            (
                "calc(var(--x))",
                CssNumericConstructionErrorKind::SubstitutionRequired,
            ),
            ("2.5", CssNumericConstructionErrorKind::RootDomainMismatch),
            ("NaN", CssNumericConstructionErrorKind::RootDomainMismatch),
            (
                "calc(1",
                CssNumericConstructionErrorKind::RecoveredComponent,
            ),
        ] {
            let values = crate::parse_component_values(source).unwrap();
            let component = &values.items()[0];
            let admit = |_| Ok(());
            let release = |_| {};
            let control = Control::new(limits(), &admit, &release);
            let error = NumericInputContext::Checked
                .check_component_controlled(component, CalculationRoot::Integer, &control)
                .unwrap()
                .unwrap_err();
            assert_eq!(error.kind(), &expected, "{source}");
            let ordinary = NumericInputContext::Checked
                .check_component(component, CalculationRoot::Integer)
                .unwrap_err();
            assert_eq!(error.origin(), ordinary.origin(), "{source}");
            assert_eq!(control.failure(), None, "{source}");
            assert_eq!(control.live().bytes(), 0, "{source}");
        }
    }

    #[test]
    fn every_reached_math_admission_can_refuse_without_retaining_scratch_or_changing_input() {
        let values = crate::parse_component_values("calc(2.5 + max(1, min(3, 4)) / 2)").unwrap();
        let component = &values.items()[0];
        let before = values.clone();
        let count = Cell::new(0);
        let admit = |_| {
            count.set(count.get() + 1);
            Ok(())
        };
        let release = |_| {};
        let control = Control::new(limits(), &admit, &release);
        NumericInputContext::Checked
            .check_component_controlled(component, CalculationRoot::Integer, &control)
            .unwrap()
            .unwrap();
        assert_eq!(control.live().bytes(), 0);
        // Enumerate reached real admissions, not synthetic padding/work. Each
        // cut invokes the same owner from its original unchanged input.
        for cut in 1..=count.get() {
            let seen = Cell::new(0);
            let admit = |_| {
                seen.set(seen.get() + 1);
                if seen.get() == cut { Err(()) } else { Ok(()) }
            };
            let released = Cell::new(usize::MAX);
            let release = |usage: CssUtf16ValueStorage| released.set(usage.bytes());
            let control = Control::new(limits(), &admit, &release);
            assert!(
                NumericInputContext::Checked
                    .check_component_controlled(component, CalculationRoot::Integer, &control)
                    .is_err(),
                "cut {cut}"
            );
            assert_eq!(
                control.failure(),
                Some(crate::utf16_value::Failure::Admission)
            );
            assert_eq!(control.live().bytes(), 0, "cut {cut}");
            assert_eq!(released.get(), 0, "cut {cut}");
            assert_eq!(values, before);
        }
    }
}
