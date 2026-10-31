//! Fallible operations over the actual borrowed or transaction-owned rows.

use std::cmp::Ordering;

use crate::Result;
use crate::processing::contextual::alignment::ScopeAlignmentBudget;

fn work(budget: &mut Option<&mut ScopeAlignmentBudget>, count: usize) -> Result<()> {
    if let Some(budget) = budget {
        budget.work(count)?;
    }
    Ok(())
}

/// Orders private scratch without allocating. Equal keys require no stable
/// identity; callers include any semantically required tie in their comparator.
/// Refusal may leave scratch permuted, but never performs an unadmitted move.
pub(crate) fn order_by<T>(
    values: &mut [T],
    mut budget: Option<&mut ScopeAlignmentBudget>,
    mut compare: impl FnMut(&T, &T) -> Ordering,
) -> Result<()> {
    fn sift<T>(
        values: &mut [T],
        mut root: usize,
        end: usize,
        budget: &mut Option<&mut ScopeAlignmentBudget>,
        compare: &mut impl FnMut(&T, &T) -> Ordering,
    ) -> Result<()> {
        while root < end / 2 {
            // root < end / 2 proves these child calculations cannot overflow.
            let mut child = root * 2 + 1;
            if child + 1 < end {
                work(budget, 1)?;
                if compare(&values[child], &values[child + 1]) == Ordering::Less {
                    child += 1;
                }
            }
            work(budget, 1)?;
            if compare(&values[root], &values[child]) != Ordering::Less {
                break;
            }
            work(budget, 2)?;
            values.swap(root, child);
            root = child;
        }
        Ok(())
    }
    let len = values.len();
    for root in (0..len / 2).rev() {
        sift(values, root, len, &mut budget, &mut compare)?;
    }
    for end in (1..len).rev() {
        work(&mut budget, 2)?;
        values.swap(0, end);
        sift(values, 0, end, &mut budget, &mut compare)?;
    }
    Ok(())
}

pub(crate) fn partition_point<T>(
    values: &[T],
    mut budget: Option<&mut ScopeAlignmentBudget>,
    mut predicate: impl FnMut(&T) -> bool,
) -> Result<usize> {
    let mut start = 0;
    let mut end = values.len();
    while start < end {
        let middle = start + (end - start) / 2;
        work(&mut budget, 1)?;
        if predicate(&values[middle]) {
            start = middle + 1;
        } else {
            end = middle;
        }
    }
    Ok(start)
}

pub(crate) fn binary_search_by<T>(
    values: &[T],
    mut budget: Option<&mut ScopeAlignmentBudget>,
    mut compare: impl FnMut(&T) -> Ordering,
) -> Result<std::result::Result<usize, usize>> {
    let mut start = 0;
    let mut end = values.len();
    while start < end {
        let middle = start + (end - start) / 2;
        work(&mut budget, 1)?;
        match compare(&values[middle]) {
            Ordering::Less => start = middle + 1,
            Ordering::Greater => end = middle,
            Ordering::Equal => return Ok(Ok(middle)),
        }
    }
    Ok(Err(start))
}
