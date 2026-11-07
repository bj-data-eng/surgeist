//! Borrowed admission shared by a compilation and its real producers.

use std::{cell::Cell, fmt, mem::size_of};

use surgeist_js_ast::{
    operations::{NameAdmission, NameWork},
    scope::{ScopeAdmission, ScopeWork},
};
use surgeist_js_string::StringWork;

use crate::{
    JsError, JsResult,
    error::{EngineError, EngineResourceError},
    value::primitive::PrimitiveWork,
};
use surgeist_js_string::StringError;

/// Native logical work, not processor instructions or physical allocation size.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CompilerWork {
    Visit,
    Search,
    RegisterProbe,
    RegisterWrite,
    IndexProbe,
    IndexWrite,
    MetadataVisit,
    MetadataWrite,
    Opcode,
    EmitBytes {
        bytes: usize,
    },
    PatchBytes {
        bytes: usize,
    },
    Read {
        bytes: usize,
    },
    Copy {
        bytes: usize,
    },
    Capacity {
        bytes: usize,
    },
    Relocate {
        bytes: usize,
    },
    /// The prospective structural depth, checked before entering recursion.
    Depth {
        depth: usize,
    },
    /// Native request price for iterating an AST-owned standard name set.
    /// Capacity describes the opaque traversal request, not observed bucket reads.
    NameSetIteration {
        capacity: usize,
    },
    EnvironmentProbe,
    Scope(ScopeWork),
    Name(NameWork),
    String(StringWork),
    Primitive(PrimitiveWork),
}

pub(crate) type CompilerAdmission<'a> = Option<&'a mut dyn FnMut(CompilerWork) -> JsResult<()>>;

/// Phase-local depth, borrowed by every nested compiler. It allocates nothing.
#[derive(Debug, Default)]
pub(crate) struct CompilerDepth {
    current: Cell<usize>,
    peak: Cell<usize>,
}

impl CompilerDepth {
    pub(crate) const fn new() -> Self {
        Self {
            current: Cell::new(0),
            peak: Cell::new(0),
        }
    }

    pub(crate) fn peak(&self) -> usize {
        self.peak.get()
    }
}

/// Drop releases structural depth without admission or an observation callback.
#[derive(Debug)]
pub(crate) struct CompilerDepthGuard<'a>(Option<&'a CompilerDepth>);

impl Drop for CompilerDepthGuard<'_> {
    fn drop(&mut self) {
        if let Some(depth) = self.0 {
            depth.current.set(
                depth
                    .current
                    .get()
                    .checked_sub(1)
                    .expect("an admitted compiler depth is released once"),
            );
        }
    }
}

/// Borrows only the phase admission and depth state; no Context or owned budget.
pub(crate) struct CompilerControl<'a> {
    admission: CompilerAdmission<'a>,
    depth: &'a CompilerDepth,
}

impl fmt::Debug for CompilerControl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompilerControl")
            .field("controlled", &self.admission.is_some())
            .field("depth", &self.depth.current.get())
            .finish()
    }
}

impl<'a> CompilerControl<'a> {
    pub(crate) fn new(admission: CompilerAdmission<'a>, depth: &'a CompilerDepth) -> Self {
        Self { admission, depth }
    }

    pub(crate) fn reborrow(&mut self) -> CompilerControl<'_> {
        CompilerControl {
            admission: self.admission.as_mut().map(|callback| &mut **callback as _),
            depth: self.depth,
        }
    }

    #[inline]
    pub(crate) fn admit(&mut self, work: CompilerWork) -> JsResult<()> {
        if let Some(admission) = self.admission.as_mut() {
            admission(work)?;
        }
        Ok(())
    }

    /// The guard borrows the external depth state, not this mutable control.
    pub(crate) fn enter(&mut self) -> JsResult<CompilerDepthGuard<'a>> {
        if self.admission.is_none() {
            return Ok(CompilerDepthGuard(None));
        }
        let depth = self.depth;
        let next = depth
            .current
            .get()
            .checked_add(1)
            .ok_or_else(size_overflow)?;
        self.admit(CompilerWork::Depth { depth: next })?;
        depth.current.set(next);
        depth.peak.set(depth.peak.get().max(next));
        Ok(CompilerDepthGuard(Some(depth)))
    }

    /// Forward the scope producer's own requests without a second work tariff.
    pub(crate) fn with_scope_admission<T>(
        &mut self,
        run: impl FnOnce(ScopeAdmission<'_, JsError>) -> JsResult<T>,
    ) -> JsResult<T> {
        if self.admission.is_none() {
            return run(None);
        }
        run(Some(&mut |work| self.admit(CompilerWork::Scope(work))))
    }

    /// Forward AST name/search work to its existing operation dimensions.
    pub(crate) fn with_name_admission<T>(
        &mut self,
        run: impl FnOnce(NameAdmission<'_, JsError>) -> JsResult<T>,
    ) -> JsResult<T> {
        if self.admission.is_none() {
            return run(None);
        }
        run(Some(&mut |work| self.admit(CompilerWork::Name(work))))
    }
}

pub(crate) fn size_overflow() -> JsError {
    EngineError::Resource(EngineResourceError::SizeOverflow).into()
}

pub(crate) fn allocation_error() -> JsError {
    EngineError::Resource(EngineResourceError::Allocation).into()
}

pub(crate) fn string_error(error: StringError<JsError>) -> JsError {
    match error {
        StringError::Refused(error) => error,
        StringError::SizeOverflow => size_overflow(),
        StringError::Allocation(_) => allocation_error(),
    }
}

pub(crate) fn primitive_error(error: crate::value::primitive::PrimitiveError<JsError>) -> JsError {
    use crate::value::primitive::PrimitiveError;
    match error {
        PrimitiveError::Abrupt(error) | PrimitiveError::Refused(error) => error,
        PrimitiveError::WorkOverflow => size_overflow(),
        PrimitiveError::Allocation(_) => allocation_error(),
    }
}

pub(crate) fn payload_bytes<T>(elements: usize) -> JsResult<usize> {
    elements
        .checked_mul(size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or_else(size_overflow)
}

/// Admit geometric Vec growth and possible existing-payload relocation.
/// Allocator over-allocation and descriptor allocations remain native boundaries.
pub(crate) fn reserve<T>(
    values: &mut Vec<T>,
    additional: usize,
    control: &mut CompilerControl<'_>,
) -> JsResult<()> {
    let needed = values
        .len()
        .checked_add(additional)
        .ok_or_else(size_overflow)?;
    payload_bytes::<T>(needed)?;
    if needed > values.capacity() {
        let maximum = if size_of::<T>() == 0 {
            usize::MAX
        } else {
            isize::MAX as usize / size_of::<T>()
        };
        // Clamping optional growth does not relax the checked required extent.
        // Exact growth for every byte/record would make repeated appends quadratic.
        let target = values
            .capacity()
            .checked_mul(2)
            .unwrap_or(maximum)
            .min(maximum)
            .max(needed);
        let bytes = payload_bytes::<T>(target)?;
        control.admit(CompilerWork::Capacity { bytes })?;
        control.admit(CompilerWork::Relocate {
            bytes: payload_bytes::<T>(values.len())?,
        })?;
        values
            .try_reserve_exact(target - values.len())
            .map_err(|_| allocation_error())?;
    }
    Ok(())
}

/// ThinVec's pinned producer has no fallible reservation API. Admit the known
/// payload before native entry; this does not guarantee allocation success.
pub(crate) fn thin_with_capacity<T>(
    elements: usize,
    control: &mut CompilerControl<'_>,
) -> JsResult<thin_vec::ThinVec<T>> {
    u32::try_from(elements).map_err(|_| size_overflow())?;
    let bytes = payload_bytes::<T>(elements)?;
    bytes
        .checked_add(2 * size_of::<usize>())
        .filter(|size| *size <= isize::MAX as usize)
        .ok_or_else(size_overflow)?;
    if elements != 0 {
        control.admit(CompilerWork::Capacity { bytes })?;
    }
    control.admit(CompilerWork::MetadataWrite)?;
    Ok(thin_vec::ThinVec::with_capacity(elements))
}

pub(crate) fn thin_reserve<T>(
    values: &mut thin_vec::ThinVec<T>,
    additional: usize,
    control: &mut CompilerControl<'_>,
) -> JsResult<()> {
    let needed = values
        .len()
        .checked_add(additional)
        .ok_or_else(size_overflow)?;
    u32::try_from(needed).map_err(|_| size_overflow())?;
    payload_bytes::<T>(needed)?;
    if needed > values.capacity() {
        let maximum = if size_of::<T>() == 0 {
            u32::MAX as usize
        } else {
            (u32::MAX as usize).min((isize::MAX as usize - 2 * size_of::<usize>()) / size_of::<T>())
        };
        if needed > maximum {
            return Err(size_overflow());
        }
        let target = values
            .capacity()
            .checked_mul(2)
            .unwrap_or(maximum)
            .min(maximum)
            .max(needed);
        control.admit(CompilerWork::Capacity {
            bytes: payload_bytes::<T>(target)?,
        })?;
        control.admit(CompilerWork::Relocate {
            bytes: payload_bytes::<T>(values.len())?,
        })?;
        values.reserve_exact(target - values.len());
    }
    Ok(())
}

pub(crate) fn thin_push<T>(
    values: &mut thin_vec::ThinVec<T>,
    value: T,
    control: &mut CompilerControl<'_>,
) -> JsResult<()> {
    thin_reserve(values, 1, control)?;
    control.admit(CompilerWork::MetadataWrite)?;
    control.admit(CompilerWork::Copy {
        bytes: size_of::<T>(),
    })?;
    values.push(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_controls_share_depth_and_release_without_callbacks_after_refusal() {
        let marker = size_overflow();
        let mut attempts = Vec::new();
        let mut callback = |work| {
            let CompilerWork::Depth { depth } = work else {
                panic!("only structural entries are requested");
            };
            attempts.push(depth);
            if depth > 2 {
                Err(marker.clone())
            } else {
                Ok(())
            }
        };
        let depth = CompilerDepth::new();
        {
            let mut parent = CompilerControl::new(Some(&mut callback), &depth);
            let outer = parent.enter().unwrap();
            {
                let mut child = parent.reborrow();
                let inner = child.enter().unwrap();
                assert_eq!(child.enter().unwrap_err().as_engine(), marker.as_engine());
                assert_eq!(depth.current.get(), 2);
                drop(inner);
                assert_eq!(depth.current.get(), 1);
            }
            drop(outer);
        }
        assert_eq!(attempts, [1, 2, 3]);
        assert_eq!(depth.current.get(), 0);
        assert_eq!(depth.peak(), 2);
    }

    #[test]
    fn reserve_refusal_preserves_values_and_reports_the_requested_extent() {
        let mut values = vec![41_u32, 42];
        let mut requested = Vec::new();
        let mut callback = |work| {
            requested.push(work);
            Err(size_overflow())
        };
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(Some(&mut callback), &depth);
        let additional = values.capacity() + 1;
        assert!(reserve(&mut values, additional, &mut control).is_err());
        assert_eq!(values, [41, 42]);
        assert!(
            matches!(requested.as_slice(), [CompilerWork::Capacity { bytes }] if *bytes == (2 + additional) * size_of::<u32>())
        );
    }
}
