//! Temporary row storage used by the existing disposable child factory.
use crate::LayoutScalar;
use crate::text_layout::scope_alignment::{EntryBacking, EntryFunding};

/// Data drops before its backing lease. Moving a completed copy into a session
/// transfers both pieces; scratch work/materialization is never refunded.
pub(super) struct Rows<T> {
    pub(super) values: Vec<T>,
    backing: EntryBacking,
}
impl<T> Rows<T> {
    pub(super) fn new() -> Self {
        Self {
            values: Vec::new(),
            backing: EntryBacking::default(),
        }
    }
    pub(super) fn push<S: LayoutScalar>(
        &mut self,
        value: T,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<()> {
        self.backing.reserve_push(&mut self.values, funding)?;
        self.values.push(value);
        Ok(())
    }
    pub(super) fn into_parts(self) -> (Vec<T>, EntryBacking) {
        (self.values, self.backing)
    }
}

pub(super) fn copy<T: Copy, S: LayoutScalar>(
    source: &[T],
    funding: Option<&EntryFunding<S>>,
) -> surgeist_text::Result<Rows<T>> {
    let mut rows = Rows::new();
    for row in source {
        inspect(funding, 1)?;
        rows.push(*row, funding)?;
    }
    Ok(rows)
}

pub(super) fn contains<T: Eq, S: LayoutScalar>(
    source: &[T],
    target: &T,
    funding: Option<&EntryFunding<S>>,
) -> surgeist_text::Result<bool> {
    for row in source {
        inspect(funding, 1)?;
        if row == target {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn inspect<S: LayoutScalar>(
    funding: Option<&EntryFunding<S>>,
    count: usize,
) -> surgeist_text::Result<()> {
    funding.map_or(Ok(()), |funding| funding.visit(count))
}
