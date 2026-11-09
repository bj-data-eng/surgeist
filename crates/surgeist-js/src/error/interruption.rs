//! Prepared immutable source-module interruption causes and observation roots.
use super::{
    Context, Diagnostic, EngineError, JsError, RootingError, StoredJsError, diagnostic::Mask,
};
use crate::{Runtime, builtins::promise::native_settlement::Association, module::Module};
use std::{cell::OnceCell, fmt, rc::Rc};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, ObjectId, RecordRef, Rooted, Trace};

pub(crate) struct CauseData {
    original: OnceCell<StoredJsError>,
}
pub(crate) struct CauseAnchor(Rc<CauseData>);
impl Trace for CauseAnchor {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.0
            .original
            .get()
            .map_or(Ok(()), |error| error.trace(edges))
    }
}

/// Shared rootless identity and visibility; every stored owner traces this value.
pub(crate) struct RetainedInterruption {
    data: Rc<CauseData>,
    anchor: Option<RecordRef<CauseAnchor>>,
    identity: ObjectId,
    mask: Mask,
}
impl RetainedInterruption {
    pub(crate) const fn preparation_layout() -> [usize; 3] {
        [
            std::mem::size_of::<CauseData>(),
            std::mem::size_of::<CauseAnchor>(),
            std::mem::size_of::<Self>(),
        ]
    }
    pub(crate) fn prepare(context: &Context) -> crate::JsResult<(Rc<Self>, Rooted<CauseAnchor>)> {
        let data = Rc::new(CauseData {
            original: OnceCell::new(),
        });
        let anchor = context
            .runtime()
            .heap
            .allocate(CauseAnchor(data.clone()))
            .map_err(|_| super::EngineResourceError::Allocation)?;
        let retained = Rc::new(Self {
            data,
            identity: anchor.handle().erase(),
            anchor: Some(anchor.record_ref()),
            mask: Mask::Any,
        });
        Ok((retained, anchor))
    }
    pub(crate) fn seal(self: &Rc<Self>, error: &JsError) -> Rc<Self> {
        if let Some(EngineError::ModuleInterrupted(interruption)) = error.as_engine() {
            return interruption.retained.clone();
        }
        assert!(
            error.as_engine().is_some(),
            "only typed engine failures interrupt modules"
        );
        if self.data.original.get().is_none() {
            let _ = self.data.original.set(error.clone().into_stored());
        }
        self.clone()
    }
    pub(super) fn view_with(&self, mask: Mask) -> Diagnostic<'_> {
        Diagnostic::stored(
            self.data.original.get().expect("interruption is sealed"),
            self.mask.intersect(mask),
        )
    }
    pub(crate) fn view(&self) -> Diagnostic<'_> {
        Diagnostic::stored(
            self.data.original.get().expect("interruption is sealed"),
            self.mask,
        )
    }
    pub(super) fn has_anchor(&self, mask: Mask) -> bool {
        self.anchor.is_some() && mask.allows(self.identity)
    }
    pub(super) fn trace_with(
        &self,
        mask: Mask,
        edges: &mut EdgeVisitor<'_>,
    ) -> Result<(), AccessError> {
        if self.has_anchor(mask) {
            self.anchor.trace(edges)?;
        }
        if let Some(original) = self.data.original.get() {
            Diagnostic::stored(original, self.mask.intersect(mask)).trace(edges)?;
        }
        Ok(())
    }
    pub(crate) fn external(self: &Rc<Self>) -> Result<ModuleInterruption, RootingError> {
        let root = self
            .anchor
            .as_ref()
            .map(|anchor| {
                anchor
                    .upgrade()?
                    .ok_or(RootingError::Expired)
                    .map(ObservationRoot::Cause)
            })
            .transpose()?;
        Ok(ModuleInterruption {
            retained: self.clone(),
            root,
        })
    }
}
impl Trace for RetainedInterruption {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.trace_with(self.mask, edges)
    }
}
impl fmt::Debug for RetainedInterruption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Interruption").field(&self.identity).finish()
    }
}
#[derive(Clone)]
enum ObservationRoot {
    Module(Module),
    Association(Rooted<Association>),
    Cause(Rooted<CauseAnchor>),
}
impl ObservationRoot {
    fn local(&self, runtime: &Runtime) -> bool {
        match self {
            Self::Module(module) => module.realm().runtime().same_heap(runtime),
            Self::Association(owner) => owner.runtime().same_heap(&runtime.heap),
            Self::Cause(owner) => owner.runtime().same_heap(&runtime.heap),
        }
    }
}

/// An immutable, instance-terminal source-module interruption.
///
/// Increasing a later operation's budget cannot resume the interrupted instance.
/// Cloning shares the cause and existing observation roots; inspecting it never
/// reconstructs an owned guest-error graph.
#[derive(Clone)]
pub struct ModuleInterruption {
    pub(crate) retained: Rc<RetainedInterruption>,
    root: Option<ObservationRoot>,
}
impl ModuleInterruption {
    /// Stable cause identity, including its heap and allocation generation.
    #[must_use]
    pub fn identity(&self) -> ObjectId {
        self.retained.identity
    }
    /// Borrows the original typed failure and available diagnostic provenance.
    #[must_use]
    pub fn cause(&self) -> Diagnostic<'_> {
        self.retained.view()
    }
    // The cause has already crossed receiving-runtime admission. An observation
    // root may be local even when that admitted cause has no visible heap leaves.
    pub(crate) fn module(retained: Rc<RetainedInterruption>, module: Module) -> Self {
        Self {
            retained,
            root: Some(ObservationRoot::Module(module)),
        }
    }
    pub(crate) fn association(
        retained: Rc<RetainedInterruption>,
        owner: Rooted<Association>,
    ) -> Self {
        Self {
            retained,
            root: Some(ObservationRoot::Association(owner)),
        }
    }
    pub(super) fn root_local(&self, runtime: &Runtime) -> bool {
        self.root.as_ref().is_some_and(|root| root.local(runtime))
    }
    pub(super) fn has_root(&self) -> bool {
        self.root.is_some() && self.retained.has_anchor(self.retained.mask)
    }
    pub(crate) fn admit(&mut self, runtime: &Runtime) {
        if self.root.as_ref().is_some_and(|root| !root.local(runtime)) {
            let narrowed = Rc::new(RetainedInterruption {
                data: self.retained.data.clone(),
                anchor: None,
                identity: self.retained.identity,
                mask: Mask::NoHeap,
            });
            let old = std::mem::replace(
                self,
                Self {
                    retained: narrowed,
                    root: None,
                },
            );
            drop(old);
        }
    }
}
impl PartialEq for ModuleInterruption {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}
impl Eq for ModuleInterruption {}
impl fmt::Debug for ModuleInterruption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleInterruption")
            .field("identity", &self.identity())
            .field("cause", &self.cause())
            .finish()
    }
}
impl fmt::Display for ModuleInterruption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source module interrupted: {}", self.cause())
    }
}
impl std::error::Error for ModuleInterruption {}

/// Borrowed immutable interruption nested in a diagnostic.
#[derive(Clone, Copy)]
pub struct DiagnosticModuleInterruption<'a>(
    pub(crate) &'a Rc<RetainedInterruption>,
    pub(super) Mask,
);
impl<'a> DiagnosticModuleInterruption<'a> {
    /// Stable interruption identity.
    #[must_use]
    pub fn identity(self) -> ObjectId {
        self.0.identity
    }
    /// Borrows the retained original failure.
    #[must_use]
    pub fn cause(self) -> Diagnostic<'a> {
        self.0.view_with(self.1)
    }
}
impl fmt::Debug for DiagnosticModuleInterruption<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ModuleInterrupted")
            .field(&self.identity())
            .finish()
    }
}
impl PartialEq for DiagnosticModuleInterruption<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}
impl Eq for DiagnosticModuleInterruption<'_> {}

#[cfg(test)]
pub(crate) mod metadata_layout_tests;
