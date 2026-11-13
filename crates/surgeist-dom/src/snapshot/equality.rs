//! Structural Node equality over two ordinary subtrees in one snapshot.

use super::{DomNode, DomNodeKind};
use crate::{DocumentString, DomError, NodeId, names::same_code_units};

#[derive(Clone, Copy)]
struct Children<'a> {
    left: &'a [NodeId],
    right: &'a [NodeId],
}

impl<'a> DomNode<'a> {
    /// Compare the pinned DOM equality fields, attributes, and ordinary
    /// children with an exact node in this snapshot. Each visited edge and
    /// canonical lookup, compared code unit, and copied scratch frame is
    /// metered. New scratch is admitted before allocation and old scratch
    /// stays funded through replacement.
    pub fn is_equal_node_with<E: From<DomError>, Backing>(
        self,
        other: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
        mut reserve: impl FnMut(usize) -> Result<Backing, E>,
    ) -> Result<bool, E> {
        let mut left = self;
        let mut right = self.snapshot.node_with(other, &mut step)?;
        let mut siblings = Children {
            left: &[],
            right: &[],
        };
        // Scratch storage drops before its admission.
        let mut pending = (Vec::<Children<'a>>::new(), None);
        loop {
            if left.id() != right.id() {
                let children = Children {
                    left: left.children(),
                    right: right.children(),
                };
                if children.left.len() != children.right.len()
                    || !fields_equal(left, right, &mut step)?
                {
                    return Ok(false);
                }
                if !children.left.is_empty() {
                    if !siblings.left.is_empty() {
                        push_frame(&mut pending, siblings, &mut step, &mut reserve)?;
                    }
                    siblings = children;
                }
            }
            if siblings.left.is_empty() {
                let Some(frame) = pending.0.pop() else {
                    return Ok(true);
                };
                step()?;
                siblings = frame;
            }
            let ((left_id, left_rest), (right_id, right_rest)) = siblings
                .left
                .split_first()
                .zip(siblings.right.split_first())
                .ok_or(DomError::NotFound)?;
            step()?;
            left = self.snapshot.node_with(*left_id, &mut step)?;
            right = self.snapshot.node_with(*right_id, &mut step)?;
            siblings = Children {
                left: left_rest,
                right: right_rest,
            };
        }
    }
}

fn push_frame<'a, E: From<DomError>, Backing>(
    pending: &mut (Vec<Children<'a>>, Option<Backing>),
    frame: Children<'a>,
    mut step: impl FnMut() -> Result<(), E>,
    reserve: impl FnOnce(usize) -> Result<Backing, E>,
) -> Result<(), E> {
    step()?;
    if pending.0.len() == pending.0.capacity() {
        let capacity = pending
            .0
            .capacity()
            .checked_mul(2)
            .ok_or(DomError::RepresentationExhausted)?
            .max(1);
        let bytes = capacity
            .checked_mul(size_of::<Children<'_>>())
            .ok_or(DomError::RepresentationExhausted)?;
        let backing = reserve(bytes)?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        if replacement.capacity() != capacity {
            return Err(DomError::AllocationFailed.into());
        }
        for &frame in &pending.0 {
            step()?;
            replacement.push(frame);
        }
        *pending = (replacement, Some(backing));
    }
    pending.0.push(frame);
    Ok(())
}

fn fields_equal<E: From<DomError>>(
    left: DomNode<'_>,
    right: DomNode<'_>,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    use DomNodeKind as Kind;
    match (left.kind(), right.kind()) {
        (Kind::Document(left), Kind::Document(right)) => Ok(left.interface() == right.interface()),
        (Kind::DocumentType(left), Kind::DocumentType(right)) => {
            Ok(equal_text(left.name(), right.name(), &mut step)?
                && equal_text(left.public_id(), right.public_id(), &mut step)?
                && equal_text(left.system_id(), right.system_id(), step)?)
        }
        (Kind::ProcessingInstruction(left), Kind::ProcessingInstruction(right)) => {
            Ok(equal_text(left.target(), right.target(), &mut step)?
                && equal_text(left.data(), right.data(), step)?)
        }
        (Kind::Text(left), Kind::Text(right)) | (Kind::Comment(left), Kind::Comment(right)) => {
            equal_text(left, right, step)
        }
        (Kind::Attr(left), Kind::Attr(right)) => {
            Ok(left.name().same_expanded_identity(right.name())?
                && equal_text(left.value(), right.value(), step)?)
        }
        (Kind::Element(a), Kind::Element(b)) => {
            if a.has_failed_construction_interface() != b.has_failed_construction_interface()
                || !a.name().same_expanded_identity(b.name())?
            {
                return Ok(false);
            }
            match (a.name().prefix(), b.name().prefix()) {
                (Some(a), Some(b)) if equal_text(a, b, &mut step)? => (),
                (None, None) => (),
                _ => return Ok(false),
            }
            let left = left.attributes().ok_or(DomError::WrongNodeKind)?;
            let right = right.attributes().ok_or(DomError::WrongNodeKind)?;
            if left.len() != right.len() {
                return Ok(false);
            }
            let mut equal = true;
            left.find_with(&mut step, |attribute, step| {
                let matching = right.find_with(&mut *step, |candidate, _| {
                    Ok(attribute.name().same_expanded_identity(candidate.name())?)
                })?;
                equal = match matching {
                    Some(matching) => equal_text(attribute.value(), matching.value(), step)?,
                    None => false,
                };
                Ok(!equal)
            })?;
            Ok(equal)
        }
        (Kind::DocumentFragment, Kind::DocumentFragment)
        | (Kind::ShadowRoot(_), Kind::ShadowRoot(_)) => Ok(true),
        _ => Ok(false),
    }
}

fn equal_text<E>(
    left: &DocumentString,
    right: &DocumentString,
    step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    same_code_units(left.code_units(), right.code_units(), step)
}
