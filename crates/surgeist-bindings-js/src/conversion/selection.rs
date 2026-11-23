//! Ordered Web IDL selection over declaration-validated alternatives.

use crate::{Value, ValueKind};

use super::sequence::IteratorSource;
use super::{Conversion, Failure, Rule, Work};

#[cfg(test)]
pub(super) mod tests;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Category {
    Undefined,
    Interface(&'static str),
    Callable,
    Sequence,
    Dictionary,
    Object,
    Boolean,
    Number,
    BigInt,
    String,
    Any,
}

/// A candidate's tag identifies the generated conversion that consumes the value.
/// Nested union members contribute their categories to a single overload candidate.
pub(super) struct Candidate<Tag> {
    tag: Tag,
    categories: &'static [Category],
    optional: bool,
    nullable: bool,
}

impl<Tag> Candidate<Tag> {
    pub(super) const fn new(
        tag: Tag,
        categories: &'static [Category],
        optional: bool,
        nullable: bool,
    ) -> Self {
        Self {
            tag,
            categories,
            optional,
            nullable,
        }
    }

    fn has(&self, category: Category) -> bool {
        self.categories.contains(&category)
    }
}

#[derive(Clone, Copy)]
pub(super) enum SelectionKind {
    Union,
    Overload,
}

impl SelectionKind {
    fn rule(self, declaration: &'static str) -> Rule {
        match self {
            Self::Union => Rule::Union(declaration),
            Self::Overload => Rule::Overload(declaration),
        }
    }
}

pub enum SelectedInput<'input, 'scope> {
    Value(&'input Value<'scope>),
    Iterable(IteratorSource<'input, 'scope>),
}

impl<'scope> Conversion<'_, 'scope> {
    pub(super) fn effective_arity(
        &mut self,
        declaration: &'static str,
        supplied: usize,
        maximum: usize,
        arities: &[usize],
    ) -> Result<usize, Failure> {
        self.work(Work::Selection, 1).map_err(Failure)?;
        let count = supplied.min(maximum);
        if arities.contains(&count) {
            Ok(count)
        } else {
            Err(self.validation(SelectionKind::Overload.rule(declaration)))
        }
    }

    pub(super) fn select<'input, Tag: Copy>(
        &mut self,
        kind: SelectionKind,
        declaration: &'static str,
        input: &'input Value<'scope>,
        candidates: &[Candidate<Tag>],
        mut has_interface: impl FnMut(&mut Self, &'static str, &Value<'scope>) -> Result<bool, Failure>,
    ) -> Result<(Tag, SelectedInput<'input, 'scope>), Failure> {
        self.work(Work::Selection, 1).map_err(Failure)?;
        let inspected = self.scope.kind(input);
        let value = self.core(inspected)?;
        let find = |category| candidates.iter().find(|candidate| candidate.has(category));
        let selected = |candidate: &Candidate<Tag>| (candidate.tag, SelectedInput::Value(input));

        if matches!(value, ValueKind::Undefined) {
            let explicitly_undefined = match kind {
                SelectionKind::Union => find(Category::Undefined),
                SelectionKind::Overload => candidates.iter().find(|candidate| candidate.optional),
            };
            if let Some(candidate) = explicitly_undefined {
                return Ok(selected(candidate));
            }
        }
        if matches!(value, ValueKind::Null | ValueKind::Undefined)
            && let Some(candidate) = candidates
                .iter()
                .find(|candidate| candidate.nullable || candidate.has(Category::Dictionary))
        {
            return Ok(selected(candidate));
        }

        if value == ValueKind::Object {
            for candidate in candidates {
                for category in candidate.categories {
                    if let Category::Interface(name) = category {
                        self.work(Work::Selection, 1).map_err(Failure)?;
                        if has_interface(self, name, input)? {
                            return Ok(selected(candidate));
                        }
                    }
                }
            }
            // A plain object candidate is distinguishable only from non-object
            // categories. A declaration cannot also offer a callable,
            // dictionary, iterable or interface alternative at this slot.
            if let Some(candidate) = find(Category::Object) {
                return Ok(selected(candidate));
            }
            if let Some(candidate) = find(Category::Callable) {
                let callable = self.scope.is_callable(input);
                if self.core(callable)? {
                    return Ok(selected(candidate));
                }
            }
            if let Some(candidate) = find(Category::Sequence)
                && let Some(source) = self.probe_iterator(input)?
            {
                return Ok((candidate.tag, SelectedInput::Iterable(source)));
            }
            if let Some(candidate) = find(Category::Dictionary) {
                return Ok(selected(candidate));
            }
        }

        let matching_primitive = match value {
            ValueKind::Boolean => find(Category::Boolean),
            ValueKind::Number => find(Category::Number),
            ValueKind::BigInt => find(Category::BigInt),
            _ => None,
        };
        if let Some(candidate) = matching_primitive {
            return Ok(selected(candidate));
        }
        let fallback = [
            Category::String,
            Category::Number,
            Category::Boolean,
            Category::BigInt,
        ]
        .into_iter()
        .find_map(find);
        if let Some(candidate) = fallback {
            return Ok(selected(candidate));
        }
        if matches!(kind, SelectionKind::Overload)
            && let Some(candidate) = find(Category::Any)
        {
            return Ok(selected(candidate));
        }
        Err(self.validation(kind.rule(declaration)))
    }
}
