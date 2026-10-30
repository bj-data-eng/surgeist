//! Fallible owned buffers of the completed source-analysis phase.
use crate::Result;

pub(in crate::engine) fn push<T>(rows: &mut Vec<T>, value: T) -> Result<()> {
    rows.try_reserve(1).map_err(|_| super::capacity())?;
    rows.push(value);
    Ok(())
}
pub(in crate::engine) fn filled<T: Clone>(length: usize, value: T) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(length)
        .map_err(|_| super::capacity())?;
    rows.resize(length, value);
    Ok(rows)
}
pub(in crate::engine) fn collect<T>(values: impl IntoIterator<Item = T>) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    for value in values {
        push(&mut rows, value)?;
    }
    Ok(rows)
}
