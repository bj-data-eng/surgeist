//! Values 4 §2.2 reorderable component combinators.
//!
//! `a || b` admits one or more operands and `a && b` every operand not marked
//! optional, each at most once and in any order. Combinators are not
//! associative (`a || b || c` and `a || [ b || c ]` differ), so an operand is a
//! whole sub-grammar parser, possibly itself a group, that consumes its
//! complete contiguous match. A bracketed group therefore can never be
//! interleaved with its siblings, and a matched operand is never re-entered,
//! which rejects duplicates uniformly.
//!
//! Every operand shares one failure policy: a committed operand failure (for
//! example a malformed nested function) propagates unchanged, and [`fill`]
//! treats an ordinary rejection as "no match" while nesting and
//! component-resource failures propagate from any operand.
use super::property_input::Stream;
use crate::diagnostic::error::property::{
    Error, SyntaxError, is_resource_parse_error, unexpected_at,
};

/// The reorderable combinator joining one operand table.
#[derive(Clone, Copy)]
pub(super) enum Combinator<const N: usize> {
    /// `||`: one or more operands.
    AnyOf,
    /// `&&`: every operand whose `optional` entry is false (`?` operands may
    /// be absent).
    AllOf { optional: [bool; N] },
}

impl<const N: usize> Combinator<N> {
    fn is_complete(self, matched: &[bool; N]) -> bool {
        match self {
            Self::AnyOf => matched.contains(&true),
            Self::AllOf { optional } => matched
                .iter()
                .zip(optional)
                .all(|(matched, optional)| *matched || optional),
        }
    }
}

/// What a group does at a component that no remaining operand admits.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Tail {
    /// The group extends to the end or the caller's boundary: reject it.
    Reject,
    /// Juxtaposed grammar follows the group: stop and leave it to the caller.
    Leave,
}

/// One operand attempt: `Ok(true)` consumed a complete match, `Ok(false)`
/// did not match (the combinator undoes any consumption), and `Err` is a
/// committed failure that ends the whole value.
pub(super) type Operand<'i> = Result<bool, SyntaxError<'i, Error>>;

/// Parses one reorderable group from an operand table indexed `0..N`, tried
/// in table order, and returns which operands matched.
///
/// Iteration stops at exhaustion, at `boundary`, or at a component no
/// unmatched operand admits; [`Tail`] decides whether that last case is an
/// error. An empty `||` result or a missing required `&&` operand is rejected
/// at the current location. A boundary that does not hold must leave the
/// input unchanged; one that holds may keep the component it recognized for
/// its caller (the `font` prefix keeps the font size it probed).
///
/// A rejection reports the next component as the encountered token, or the
/// end of the value when none remains.
pub(super) fn parse_group<'i, const N: usize>(
    input: &mut Stream<'i>,
    combinator: Combinator<N>,
    tail: Tail,
    boundary: impl FnMut(&mut Stream<'i>) -> bool,
    operand: impl FnMut(usize, &mut Stream<'i>) -> Operand<'i>,
) -> Result<[bool; N], SyntaxError<'i, Error>> {
    let matched = collect(input, tail, boundary, operand)?;
    if combinator.is_complete(&matched) {
        Ok(matched)
    } else {
        Err(rejected_here(input))
    }
}

/// One bracketed `||` group used as an operand of an enclosing group. An
/// empty group does not match; its operands' committed failures still
/// propagate. Only `||` is admitted: an unmatched `||` group has filled no
/// operand slot, so the enclosing group never sees partial state.
pub(super) fn group_operand<'i, const N: usize>(
    input: &mut Stream<'i>,
    boundary: impl FnMut(&mut Stream<'i>) -> bool,
    operand: impl FnMut(usize, &mut Stream<'i>) -> Operand<'i>,
) -> Operand<'i> {
    let matched = collect::<N>(input, Tail::Leave, boundary, operand)?;
    Ok(Combinator::AnyOf.is_complete(&matched))
}

/// The rejection at the current location: the next component is the
/// encountered token, or the value ended.
fn rejected_here<'i>(input: &mut Stream<'i>) -> SyntaxError<'i, Error> {
    let location = input.site();
    let state = input.mark();
    let error = match input.consume_after_whitespace() {
        Ok(token) => location.unexpected(token.clone()),
        Err(_) => unexpected_at(location),
    };
    input.restore(state);
    error
}

fn collect<'i, const N: usize>(
    input: &mut Stream<'i>,
    tail: Tail,
    mut boundary: impl FnMut(&mut Stream<'i>) -> bool,
    mut operand: impl FnMut(usize, &mut Stream<'i>) -> Operand<'i>,
) -> Result<[bool; N], SyntaxError<'i, Error>> {
    let mut matched = [false; N];
    'group: while !input.only_whitespace_remains() && !boundary(input) {
        for (index, matched) in matched.iter_mut().enumerate() {
            if *matched {
                continue;
            }
            let state = input.mark();
            if operand(index, input)? {
                *matched = true;
                continue 'group;
            }
            input.restore(state);
        }
        if tail == Tail::Reject {
            return Err(rejected_here(input));
        }
        break;
    }
    Ok(matched)
}

/// An operand committed by lookahead: once `starts` admits the next
/// component, a parse failure is the whole value's failure.
pub(super) fn commit<'i, T>(
    slot: &mut Option<T>,
    input: &mut Stream<'i>,
    starts: impl FnOnce(&mut Stream<'i>) -> bool,
    parse: impl FnOnce(&mut Stream<'i>) -> Result<T, SyntaxError<'i, Error>>,
) -> Operand<'i> {
    if !starts(input) {
        return Ok(false);
    }
    *slot = Some(parse(input)?);
    Ok(true)
}

/// One keyword operand from `table`, matched ASCII case-insensitively.
pub(super) fn keyword<'i, T: Copy>(
    slot: &mut Option<T>,
    input: &mut Stream<'i>,
    table: &[(&str, T)],
) -> Operand<'i> {
    let Ok(ident) = input.consume_ident() else {
        return Ok(false);
    };
    let Some((_, value)) = table
        .iter()
        .find(|(name, _)| ident.eq_ignore_ascii_case(name))
    else {
        return Ok(false);
    };
    *slot = Some(*value);
    Ok(true)
}

/// One optional component under the shared failure policy: a rejection is
/// `None` with its consumption undone; resource failures propagate.
pub(super) fn attempt<'i, T>(
    input: &mut Stream<'i>,
    parse: impl FnOnce(&mut Stream<'i>) -> Result<T, SyntaxError<'i, Error>>,
) -> Result<Option<T>, SyntaxError<'i, Error>> {
    let mut value = None;
    fill(&mut value, input, parse)?;
    Ok(value)
}

/// Attempts one recoverable operand into its slot under the shared failure
/// policy: a rejection is "no match"; resource failures propagate.
pub(super) fn fill<'i, T>(
    slot: &mut Option<T>,
    input: &mut Stream<'i>,
    parse: impl FnOnce(&mut Stream<'i>) -> Result<T, SyntaxError<'i, Error>>,
) -> Operand<'i> {
    match input.attempt(parse) {
        Ok(value) => {
            *slot = Some(value);
            Ok(true)
        }
        Err(error) if is_resource_parse_error(&error) => Err(error),
        Err(_) => Ok(false),
    }
}

/// The Values 4 §2.3 `#` multiplier over a native rule-level parser, each
/// item bounded at its comma (cssparser's `parse_until_before`). An empty
/// item keeps the list's own
/// post-conditions: at the end of the list it is diagnosed by `trailing`
/// (given the exhausted input and the empty item's start); elsewhere `item`
/// re-enters at the stray separator and reports it as it would any
/// unexpected component.
pub(super) fn parse_native_comma_list<'i, 't, T>(
    input: &mut cssparser::Parser<'i, 't>,
    mut item: impl FnMut(
        &mut cssparser::Parser<'i, '_>,
    ) -> Result<T, cssparser::ParseError<'i, crate::diagnostic::CssError>>,
    trailing: impl FnOnce(
        &mut cssparser::Parser<'i, 't>,
        cssparser::SourceLocation,
    ) -> cssparser::ParseError<'i, crate::diagnostic::CssError>,
) -> Result<Vec<T>, cssparser::ParseError<'i, crate::diagnostic::CssError>> {
    let mut values = Vec::with_capacity(1);
    loop {
        // The item starts directly after its separator.
        let start = input.current_source_location();
        let mut empty = false;
        let value = input.parse_until_before(cssparser::Delimiter::Comma, |input| {
            if input.is_exhausted() {
                empty = true;
                return Err(input.new_error(cssparser::BasicParseErrorKind::EndOfInput));
            }
            item(input)
        });
        match value {
            Ok(value) => values.push(value),
            Err(_) if empty && input.is_exhausted() => return Err(trailing(input, start)),
            Err(error) if empty => return Err(item(input).err().unwrap_or(error)),
            Err(error) => return Err(error),
        }
        match input.next() {
            Err(_) => return Ok(values),
            Ok(cssparser::Token::Comma) => {}
            Ok(_) => unreachable!("an item ends at a comma or the end of the list"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::values::CssComponentValueErrorKind;
    use crate::component::{CssComponentValueError, CssValueOrigin};
    use crate::syntax_consumption::{SyntaxInput, SyntaxInputLimits, normalize};

    fn with_input<R>(css: &str, run: impl FnOnce(&mut Stream<'_>) -> R) -> R {
        let values = crate::component::values::parse_component_values(css).unwrap();
        let document = normalize(
            SyntaxInput::CheckedComponents(values.items()),
            SyntaxInputLimits::default(),
            0,
        )
        .unwrap();
        run(&mut Stream::new(&document))
    }

    fn word<'i>(input: &mut Stream<'i>, expected: &str) -> Operand<'i> {
        Ok(input
            .attempt(|input| input.consume_keyword(expected))
            .is_ok())
    }

    fn resource<'i>(input: &Stream<'i>) -> SyntaxError<'i, Error> {
        crate::diagnostic::error::property::invalid_component_value(
            input.site(),
            CssComponentValueError::new(
                CssComponentValueErrorKind::NestingLimit,
                CssValueOrigin::Programmatic,
            ),
        )
    }

    /// `a || b || c`, each operand its same-named keyword.
    fn flat<'i>(input: &mut Stream<'i>, tail: Tail) -> Result<[bool; 3], SyntaxError<'i, Error>> {
        parse_group::<3>(
            input,
            Combinator::AnyOf,
            tail,
            |_| false,
            |operand, input| word(input, ["a", "b", "c"][operand]),
        )
    }

    /// `a || [ b || c ]`.
    fn nested<'i>(input: &mut Stream<'i>) -> Result<[bool; 2], SyntaxError<'i, Error>> {
        parse_group::<2>(
            input,
            Combinator::AnyOf,
            Tail::Reject,
            |_| false,
            |operand, input| match operand {
                0 => word(input, "a"),
                _ => group_operand::<2>(
                    input,
                    |_| false,
                    |operand, input| word(input, ["b", "c"][operand]),
                ),
            },
        )
    }

    #[test]
    fn any_order_admits_each_operand_once_in_any_order() {
        let matched = with_input("c a", |input| flat(input, Tail::Reject).unwrap());
        assert_eq!(matched, [true, false, true]);
        assert!(with_input("a c a", |input| flat(input, Tail::Reject).is_err()));
        assert!(with_input("x", |input| flat(input, Tail::Reject).is_err()));
    }

    #[test]
    fn leave_stops_before_an_unadmitted_component() {
        with_input("b x", |input| {
            assert_eq!(flat(input, Tail::Leave).unwrap(), [false, true, false]);
            assert!(input.consume_keyword("x").is_ok());
        });
        assert!(with_input("x", |input| flat(input, Tail::Leave).is_err()));
    }

    #[test]
    fn a_bracketed_operand_is_never_interleaved_with_its_siblings() {
        // Values 4 §2.2: `a || [ b || c ]` admits `c b a` but not `b a c`.
        assert_eq!(
            with_input("c b a", |input| nested(input).unwrap()),
            [true, true]
        );
        assert_eq!(
            with_input("a b", |input| nested(input).unwrap()),
            [true, true]
        );
        assert!(with_input("b a c", |input| nested(input).is_err()));
        assert!(with_input("b a b", |input| nested(input).is_err()));
    }

    #[test]
    fn all_order_requires_every_non_optional_operand() {
        let all = |css: &str, optional: [bool; 2]| {
            with_input(css, |input| {
                parse_group::<2>(
                    input,
                    Combinator::AllOf { optional },
                    Tail::Reject,
                    |_| false,
                    |operand, input| word(input, ["a", "b"][operand]),
                )
                .is_ok()
            })
        };
        assert!(all("b a", [false, false]));
        assert!(!all("a", [false, false]));
        assert!(all("a", [false, true]));
        assert!(!all("b", [false, true]));
    }

    #[test]
    fn boundary_ends_the_group_without_consuming_it() {
        with_input("a , b", |input| {
            let matched = parse_group::<3>(
                input,
                Combinator::AnyOf,
                Tail::Reject,
                super::super::values::next_is_comma,
                |operand, input| word(input, ["a", "b", "c"][operand]),
            )
            .unwrap();
            assert_eq!(matched, [true, false, false]);
            assert!(input.consume_comma().is_ok());
        });
    }

    #[test]
    fn fill_treats_rejection_as_no_match_but_propagates_resource_failures() {
        with_input("a", |input| {
            let mut slot: Option<()> = None;
            let rejected = fill(&mut slot, input, |input| {
                Err::<(), _>(unexpected_at(input.site()))
            });
            assert!(!rejected.unwrap() && slot.is_none());
            let error = fill(&mut slot, input, |input| Err::<(), _>(resource(input)));
            assert!(is_resource_parse_error(&error.unwrap_err()));
            let committed = commit(
                &mut slot,
                input,
                |_| true,
                |input| Err::<(), _>(unexpected_at(input.site())),
            );
            assert!(committed.is_err());
            assert!(!commit(&mut slot, input, |_| false, |_| Ok(())).unwrap());
        });
    }

    #[test]
    fn group_propagates_a_resource_failure_from_any_operand() {
        let resource_failed = with_input("a b", |input| {
            let mut slot: Option<()> = None;
            parse_group::<2>(
                input,
                Combinator::AnyOf,
                Tail::Reject,
                |_| false,
                |operand, input| match operand {
                    0 => word(input, "a"),
                    _ => fill(&mut slot, input, |input| Err::<(), _>(resource(input))),
                },
            )
            .map_err(|error| is_resource_parse_error(&error))
        });
        assert_eq!(resource_failed, Err(true));
    }
}
