//! Immutable Panic diagnostics. External roots never enter stored payloads.

use super::*;
use crate::{Runtime, SpannedSourceText, value::Type, vm::SourcePath};
use std::{collections::HashMap, slice};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, ObjectId, Trace};

#[derive(Clone, Copy)]
pub(super) enum Mask {
    Any,
    Heap(ObjectId),
    NoHeap,
}
impl Mask {
    pub(super) fn intersect(self, other: Self) -> Self {
        match (self, other) {
            (Self::Any, mask) | (mask, Self::Any) => mask,
            (Self::Heap(a), Self::Heap(b)) if a.same_heap(b) => Self::Heap(a),
            _ => Self::NoHeap,
        }
    }
    pub(super) fn allows(self, id: ObjectId) -> bool {
        match self {
            Self::Any => true,
            Self::Heap(heap) => heap.same_heap(id),
            Self::NoHeap => false,
        }
    }
    fn same(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Any, Self::Any) | (Self::NoHeap, Self::NoHeap)
        ) || matches!((self, other), (Self::Heap(a), Self::Heap(b)) if a.same_heap(b))
    }
}

#[derive(Clone)]
enum RootLeaf {
    Interruption(ModuleInterruption),
    Object(JsObject),
    Realm(Realm),
}
impl RootLeaf {
    fn belongs_to(&self, runtime: &Runtime) -> bool {
        match self {
            Self::Interruption(value) => value.root_local(runtime),
            Self::Object(object) => object.runtime().same_heap(runtime),
            Self::Realm(realm) => realm.runtime().same_heap(runtime),
        }
    }
}
type RootBundle = HashMap<ObjectId, RootLeaf>;

// Reserve only for the next distinct visible leaf. Acquiring a root is deferred
// until capacity exists, and failure drops the caller's already-built bundle.
fn restore_root(
    roots: &mut RootBundle,
    id: ObjectId,
    acquire: impl FnOnce() -> Result<RootLeaf, RootingError>,
) -> Result<(), RootingError> {
    if roots.contains_key(&id) {
        return Ok(());
    }
    roots
        .len()
        .checked_add(1)
        .and_then(|len| len.checked_mul(std::mem::size_of::<(ObjectId, RootLeaf)>()))
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(RootingError::SizeOverflow)?;
    roots.try_reserve(1).map_err(|_| RootingError::Allocation)?;
    let root = acquire()?;
    roots.insert(id, root);
    Ok(())
}

/// A Panic's immutable source and external root custody.
///
/// This is the concrete [`std::error::Error::source`] of [`PanicError`]. Its own
/// standard source is empty; [`Self::view`] exposes the complete typed topology.
/// Cloning shares prepared data and roots without reconstructing an error tree.
#[derive(Clone)]
pub struct PanicDiagnostic {
    stored: StoredDiagnostic,
    roots: Rc<RootBundle>,
}

/// Only edge data and visibility; never an external root bundle or heap owner.
#[derive(Clone)]
pub(super) struct StoredDiagnostic {
    data: Rc<StoredJsError>,
    mask: Mask,
}
impl StoredDiagnostic {
    fn view(&self) -> Diagnostic<'_> {
        self.view_with(Mask::Any)
    }
    fn view_with(&self, mask: Mask) -> Diagnostic<'_> {
        Diagnostic {
            node: &self.data,
            mask: self.mask.intersect(mask),
        }
    }
    pub(super) fn to_external(&self) -> Result<PanicDiagnostic, RootingError> {
        let mut roots = RootBundle::new();
        self.view().restore_roots(&mut roots)?;
        Ok(PanicDiagnostic {
            stored: self.clone(),
            roots: Rc::new(roots),
        })
    }
}
impl Trace for StoredDiagnostic {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.view().trace(edges)
    }
}
impl fmt::Debug for StoredDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.view(), f)
    }
}

impl PanicDiagnostic {
    pub(super) fn new(source: JsError) -> Self {
        let mut roots = RootBundle::new();
        collect_live_roots(&source, &mut roots);
        let stored = StoredDiagnostic {
            data: Rc::new(source.into_stored()),
            mask: Mask::Any,
        };
        Self {
            stored,
            roots: Rc::new(roots),
        }
    }
    /// Borrows this observation's visible, typed diagnostic topology.
    #[must_use]
    pub fn view(&self) -> Diagnostic<'_> {
        self.stored.view()
    }
    pub(super) fn into_stored(self) -> StoredDiagnostic {
        self.stored
    }
    pub(super) fn admit(&mut self, runtime: &Runtime) {
        let receiver = self
            .roots
            .iter()
            .find_map(|(id, root)| root.belongs_to(runtime).then_some(*id));
        let mask = self
            .stored
            .mask
            .intersect(receiver.map_or(Mask::NoHeap, Mask::Heap));
        if mask.same(self.stored.mask) {
            return;
        }
        let stored = StoredDiagnostic {
            data: self.stored.data.clone(),
            mask,
        };
        let mut roots = RootBundle::new();
        // Walk the pruned view, not all leaves in the receiving runtime. A
        // foreign parent also removes receiver-local descendants.
        stored.view().visit_roots(&mut |id| {
            roots.entry(id).or_insert_with(|| {
                self.roots
                    .get(&id)
                    .expect("visible diagnostic root is retained")
                    .clone()
            });
        });
        let previous = std::mem::replace(
            self,
            Self {
                stored,
                roots: Rc::new(roots),
            },
        );
        // Final foreign disposal can reenter: the new view and all its roots
        // must already be published, with no RefCell or heap borrow held.
        drop(previous);
    }
}
fn collect_live_roots(error: &JsError, roots: &mut RootBundle) {
    match &error.inner {
        Repr::Opaque(value) => {
            if let Some(object) = value.as_object() {
                roots
                    .entry(object.identity())
                    .or_insert(RootLeaf::Object(object));
            }
        }
        Repr::Native(native) => {
            if let Some(realm) = &native.realm {
                roots
                    .entry(realm.identity())
                    .or_insert_with(|| RootLeaf::Realm(realm.clone()));
            }
            if let Some(cause) = &native.cause {
                collect_live_roots(cause, roots);
            }
            if let JsNativeErrorKind::Aggregate(members) = &native.kind {
                for member in members {
                    collect_live_roots(member, roots);
                }
            }
        }
        Repr::Engine(EngineError::ModuleInterrupted(value)) => {
            if value.has_root() {
                roots
                    .entry(value.identity())
                    .or_insert_with(|| RootLeaf::Interruption(value.clone()));
            }
        }
        Repr::Engine(EngineError::ForeignSelectedRealm(value)) => {
            for (id, root) in value.original.roots.iter() {
                roots.entry(*id).or_insert_with(|| root.clone());
            }
        }
        Repr::Engine(EngineError::Panic(panic)) => {
            if let Some(source) = &panic.source {
                for (id, root) in source.roots.iter() {
                    roots.entry(*id).or_insert_with(|| root.clone());
                }
            }
        }
        Repr::Engine(_) => {}
    }
}
impl fmt::Debug for PanicDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PanicDiagnostic")
            .field(&self.view())
            .finish()
    }
}
impl fmt::Display for PanicDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.view(), f)
    }
}
impl error::Error for PanicDiagnostic {}
impl PartialEq for PanicDiagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.view() == other.view()
    }
}
impl Eq for PanicDiagnostic {}

#[derive(Clone, Copy)]
enum Replacement {
    Object,
    Realm,
}
const OBJECT_REFUSAL_SOURCE: NativeSourceInfo = NativeSourceInfo::caller();
const REALM_REFUSAL_SOURCE: NativeSourceInfo = NativeSourceInfo::caller();
impl Replacement {
    fn message(self) -> &'static str {
        match self {
            Self::Object => "object belongs to a different runtime",
            Self::Realm => "error realm belongs to a different runtime",
        }
    }
    fn source_info(self) -> NativeSourceInfo {
        match self {
            Self::Object => OBJECT_REFUSAL_SOURCE,
            Self::Realm => REALM_REFUSAL_SOURCE,
        }
    }
}

/// A borrowed visible diagnostic. The owning Panic or rooted stored owner must
/// outlive this view. Hidden subtrees are replaced before any observation.
#[derive(Clone, Copy)]
pub struct Diagnostic<'a> {
    node: &'a StoredJsError,
    mask: Mask,
}
impl<'a> Diagnostic<'a> {
    pub(super) fn stored(node: &'a StoredJsError, mask: Mask) -> Self {
        Self { node, mask }
    }
    fn replacement(self) -> Option<Replacement> {
        match &self.node.inner {
            StoredErrorRepr::Opaque(StoredValue::Object(object))
                if !self.mask.allows(object.identity()) =>
            {
                Some(Replacement::Object)
            }
            StoredErrorRepr::Native(native)
                if native
                    .realm_identity
                    .is_some_and(|id| !self.mask.allows(id)) =>
            {
                Some(Replacement::Realm)
            }
            _ => None,
        }
    }
    /// The visible opaque, native or typed engine diagnostic.
    #[must_use]
    pub fn kind(self) -> DiagnosticKind<'a> {
        if let Some(replacement) = self.replacement() {
            return DiagnosticKind::Native(DiagnosticNative {
                data: NativeData::Replacement(replacement),
                mask: self.mask,
            });
        }
        match &self.node.inner {
            StoredErrorRepr::ForeignSelectedRealm {
                selected,
                receiving,
                original,
            } => DiagnosticKind::Engine(DiagnosticEngine::ForeignSelectedRealm(
                DiagnosticForeignSelectedRealm {
                    selected: *selected,
                    receiving: *receiving,
                    diagnostic: original.view_with(self.mask),
                },
            )),
            StoredErrorRepr::JobDeadlineOverflow((now, delay)) => {
                DiagnosticKind::Engine(DiagnosticEngine::JobDeadlineOverflow {
                    now: *now,
                    delay: *delay,
                })
            }
            StoredErrorRepr::ModuleInterrupted(value) => DiagnosticKind::Engine(
                DiagnosticEngine::ModuleInterrupted(DiagnosticModuleInterruption(value, self.mask)),
            ),
            StoredErrorRepr::Opaque(value) => DiagnosticKind::Opaque(DiagnosticOpaque { value }),
            StoredErrorRepr::Native(native) => DiagnosticKind::Native(DiagnosticNative {
                data: NativeData::Original(native),
                mask: self.mask,
            }),
            StoredErrorRepr::Rooting(value) => {
                DiagnosticKind::Engine(DiagnosticEngine::Rooting(*value))
            }
            StoredErrorRepr::Resource(value) => {
                DiagnosticKind::Engine(DiagnosticEngine::Resource(*value))
            }
            StoredErrorRepr::InvalidEntryPermit => {
                DiagnosticKind::Engine(DiagnosticEngine::InvalidEntryPermit)
            }
            StoredErrorRepr::ContextAccessBusy => {
                DiagnosticKind::Engine(DiagnosticEngine::ContextAccessBusy)
            }
            StoredErrorRepr::ObjectAccess(error) => {
                DiagnosticKind::Engine(DiagnosticEngine::ObjectAccess(*error))
            }
            StoredErrorRepr::ValueRender(error) => {
                DiagnosticKind::Engine(DiagnosticEngine::ValueRender(*error))
            }
            StoredErrorRepr::JobContextRequired(required) => {
                DiagnosticKind::Engine(DiagnosticEngine::JobContextRequired { required })
            }
            StoredErrorRepr::JobRetired(reason) => {
                DiagnosticKind::Engine(DiagnosticEngine::JobRetired(*reason))
            }
            StoredErrorRepr::Job(error) => DiagnosticKind::Engine(DiagnosticEngine::Job(*error)),
            StoredErrorRepr::InvocationIdExhausted => {
                DiagnosticKind::Engine(DiagnosticEngine::InvocationIdExhausted)
            }
            StoredErrorRepr::ScopeAnalysis(value) => {
                DiagnosticKind::Engine(DiagnosticEngine::ScopeAnalysis(*value))
            }
            StoredErrorRepr::RuntimeLimit(value) => {
                DiagnosticKind::Engine(DiagnosticEngine::RuntimeLimit(*value))
            }
            StoredErrorRepr::Panic { message, source } => {
                DiagnosticKind::Engine(DiagnosticEngine::Panic {
                    message,
                    diagnostic: source.as_ref().map(|source| source.view_with(self.mask)),
                })
            }
            #[cfg(feature = "fuzz")]
            StoredErrorRepr::NoInstructionsRemain => {
                DiagnosticKind::Engine(DiagnosticEngine::NoInstructionsRemain)
            }
        }
    }
    /// The original captured trace, distinguishing absent and empty captures.
    #[must_use]
    pub fn backtrace(self) -> Option<DiagnosticBacktrace<'a>> {
        if self.replacement().is_some() {
            return None;
        }
        self.node
            .backtrace
            .as_ref()
            .map(|trace| DiagnosticBacktrace {
                frames: trace.frames(),
            })
    }
    fn visit_roots(self, visit: &mut impl FnMut(ObjectId)) {
        match self.kind() {
            DiagnosticKind::Opaque(value) => {
                if let Some(id) = value.object_identity() {
                    visit(id);
                }
            }
            DiagnosticKind::Native(native) => {
                if let Some(id) = native.realm_identity() {
                    visit(id);
                }
                if let Some(cause) = native.cause() {
                    cause.visit_roots(visit);
                }
                for member in native.members() {
                    member.visit_roots(visit);
                }
            }
            DiagnosticKind::Engine(DiagnosticEngine::ForeignSelectedRealm(value)) => {
                value.diagnostic().visit_roots(visit)
            }
            DiagnosticKind::Engine(DiagnosticEngine::Panic {
                diagnostic: Some(source),
                ..
            }) => source.visit_roots(visit),
            DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(value)) => {
                if value.0.has_anchor(value.1) {
                    visit(value.identity());
                }
            }
            DiagnosticKind::Engine(_) => {}
        }
    }
    pub(super) fn trace(self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        match self.kind() {
            DiagnosticKind::Opaque(value) => value.value.trace(edges)?,
            DiagnosticKind::Native(native) => {
                if let NativeData::Original(data) = native.data {
                    data.realm.trace(edges)?;
                }
                if let Some(cause) = native.cause() {
                    cause.trace(edges)?;
                }
                for member in native.members() {
                    member.trace(edges)?;
                }
            }
            DiagnosticKind::Engine(DiagnosticEngine::ForeignSelectedRealm(value)) => {
                value.diagnostic().trace(edges)?
            }
            DiagnosticKind::Engine(DiagnosticEngine::Panic {
                diagnostic: Some(source),
                ..
            }) => source.trace(edges)?,
            DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(value)) => {
                value.0.trace_with(value.1, edges)?
            }
            DiagnosticKind::Engine(_) => {}
        }
        Ok(())
    }
    fn restore_roots(self, roots: &mut RootBundle) -> Result<(), RootingError> {
        match self.kind() {
            DiagnosticKind::Opaque(value) => {
                if let StoredValue::Object(edge) = value.value {
                    restore_root(roots, edge.identity(), || {
                        Ok(RootLeaf::Object(JsObject::restore(&CapturedObject(
                            edge.clone(),
                        ))?))
                    })?;
                }
            }
            DiagnosticKind::Native(native) => {
                if let NativeData::Original(data) = native.data
                    && let (Some(id), Some(realm)) = (data.realm_identity, &data.realm)
                {
                    restore_root(roots, id, || {
                        Ok(RootLeaf::Realm(Realm::restore(&CapturedRealm(
                            realm.clone(),
                        ))?))
                    })?;
                }
                if let Some(cause) = native.cause() {
                    cause.restore_roots(roots)?;
                }
                for member in native.members() {
                    member.restore_roots(roots)?;
                }
            }
            DiagnosticKind::Engine(DiagnosticEngine::ForeignSelectedRealm(value)) => {
                value.diagnostic().restore_roots(roots)?;
            }
            DiagnosticKind::Engine(DiagnosticEngine::Panic {
                diagnostic: Some(source),
                ..
            }) => source.restore_roots(roots)?,
            DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(value)) => {
                if value.0.has_anchor(value.1) {
                    restore_root(roots, value.identity(), || {
                        Ok(RootLeaf::Interruption(value.0.external()?))
                    })?;
                }
            }
            DiagnosticKind::Engine(_) => {}
        }
        Ok(())
    }
    pub(super) fn erase(self, context: &mut Context) -> JsErasedError {
        let inner = match self.kind() {
            DiagnosticKind::Opaque(value) => {
                // Keep the real opaque conversion's property reads, first
                // failure and fallback; do not stringify a diagnostic view.
                return JsError {
                    inner: Repr::Opaque(
                        value.value().expect("rooted diagnostic value is available"),
                    ),
                    backtrace: self.node.backtrace.clone(),
                }
                .into_erased(context.access());
            }
            DiagnosticKind::Native(native) => {
                // Ordinary erased conversion processes cause BEFORE members.
                let cause = native.cause().map(|cause| Box::new(cause.erase(context)));
                let kind = match native.kind() {
                    DiagnosticNativeKind::Error => JsErasedNativeErrorKind::Error,
                    DiagnosticNativeKind::Eval => JsErasedNativeErrorKind::Eval,
                    DiagnosticNativeKind::Range => JsErasedNativeErrorKind::Range,
                    DiagnosticNativeKind::Reference => JsErasedNativeErrorKind::Reference,
                    DiagnosticNativeKind::Syntax => JsErasedNativeErrorKind::Syntax,
                    DiagnosticNativeKind::Type => JsErasedNativeErrorKind::Type,
                    DiagnosticNativeKind::Uri => JsErasedNativeErrorKind::Uri,
                    DiagnosticNativeKind::Aggregate => JsErasedNativeErrorKind::Aggregate(
                        native
                            .members()
                            .map(|member| member.erase(context))
                            .collect(),
                    ),
                };
                ErasedRepr::Native(JsErasedNativeError {
                    kind,
                    message: Cow::Owned(native.message().to_owned()),
                    cause,
                })
            }
            DiagnosticKind::Engine(engine) => ErasedRepr::Engine(match engine {
                DiagnosticEngine::ModuleInterrupted(value) => {
                    ErasedEngineError::ModuleInterrupted {
                        identity: value.identity(),
                        cause: Box::new(value.cause().erase(context)),
                    }
                }
                DiagnosticEngine::ForeignSelectedRealm(value) => {
                    ErasedEngineError::ForeignSelectedRealm(ErasedForeignSelectedRealm {
                        selected: value.selected_realm(),
                        receiving: value.receiving_realm(),
                        diagnostic: Box::new(value.diagnostic().erase(context)),
                    })
                }
                DiagnosticEngine::JobDeadlineOverflow { now, delay } => {
                    ErasedEngineError::JobDeadlineOverflow { now, delay }
                }
                DiagnosticEngine::Rooting(value) => ErasedEngineError::Rooting(value),
                DiagnosticEngine::Resource(value) => ErasedEngineError::Resource(value),
                DiagnosticEngine::InvalidEntryPermit => ErasedEngineError::InvalidEntryPermit,
                DiagnosticEngine::ContextAccessBusy => ErasedEngineError::ContextAccessBusy,
                DiagnosticEngine::ObjectAccess(error) => ErasedEngineError::ObjectAccess(error),
                DiagnosticEngine::ValueRender(error) => ErasedEngineError::ValueRender(error),
                DiagnosticEngine::JobContextRequired { required } => {
                    ErasedEngineError::JobContextRequired {
                        required: required.clone(),
                    }
                }
                DiagnosticEngine::JobRetired(reason) => ErasedEngineError::JobRetired(reason),
                DiagnosticEngine::Job(error) => ErasedEngineError::Job(error),
                DiagnosticEngine::InvocationIdExhausted => ErasedEngineError::InvocationIdExhausted,
                DiagnosticEngine::ScopeAnalysis(value) => ErasedEngineError::ScopeAnalysis(value),
                DiagnosticEngine::RuntimeLimit(value) => ErasedEngineError::RuntimeLimit(value),
                DiagnosticEngine::Panic {
                    message,
                    diagnostic,
                } => ErasedEngineError::Panic(ErasedPanicError {
                    message: Box::from(message),
                    source: diagnostic.map(|source| Box::new(source.erase(context))),
                }),
                #[cfg(feature = "fuzz")]
                DiagnosticEngine::NoInstructionsRemain => ErasedEngineError::NoInstructionsRemain,
            }),
        };
        JsErasedError { inner }
    }
}

/// The visible diagnostic representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind<'a> {
    /// An original opaque value or object identity.
    Opaque(DiagnosticOpaque<'a>),
    /// A native error or a local TypeError replacement.
    Native(DiagnosticNative<'a>),
    /// An exact typed engine payload, without external rooted-error rebuilding.
    Engine(DiagnosticEngine<'a>),
}
/// A borrowed opaque value, with explicit checked rooting for object access.
#[derive(Clone, Copy)]
pub struct DiagnosticOpaque<'a> {
    value: &'a StoredValue,
}
impl DiagnosticOpaque<'_> {
    /// Its ECMAScript value type, without rooting or guest conversion.
    #[must_use]
    pub fn value_type(&self) -> Type {
        match self.value {
            StoredValue::Undefined => Type::Undefined,
            StoredValue::Null => Type::Null,
            StoredValue::Boolean(_) => Type::Boolean,
            StoredValue::Integer32(_) | StoredValue::Float64(_) => Type::Number,
            StoredValue::BigInt(_) => Type::BigInt,
            StoredValue::Symbol(_) => Type::Symbol,
            StoredValue::String(_) => Type::String,
            StoredValue::Object(_) => Type::Object,
        }
    }
    /// An inert identity; it does not keep an object or its runtime alive.
    #[must_use]
    pub fn object_identity(&self) -> Option<ObjectId> {
        match self.value {
            StoredValue::Object(object) => Some(object.identity()),
            _ => None,
        }
    }
    /// Clones one primitive or roots one existing object. This never reads
    /// guest properties. An active heap borrow is reported without panicking.
    pub fn value(&self) -> Result<JsValue, AccessError> {
        match self.value {
            StoredValue::Object(object) => object
                .upgrade()?
                .map(JsValue::from)
                .ok_or(AccessError::StaleHandle),
            _ => Ok(self.value.to_value()),
        }
    }
}
impl PartialEq for DiagnosticOpaque<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Eq for DiagnosticOpaque<'_> {}

/// Native error classification without an owned aggregate member vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticNativeKind {
    /// Error.
    Error,
    /// EvalError.
    Eval,
    /// RangeError.
    Range,
    /// ReferenceError.
    Reference,
    /// SyntaxError.
    Syntax,
    /// TypeError.
    Type,
    /// URIError.
    Uri,
    /// AggregateError; inspect members in authored order.
    Aggregate,
}
#[derive(Clone, Copy)]
enum NativeData<'a> {
    Original(&'a StoredNativeError),
    Replacement(Replacement),
}
/// A native diagnostic with all masking already applied.
#[derive(Clone, Copy)]
pub struct DiagnosticNative<'a> {
    data: NativeData<'a>,
    mask: Mask,
}
impl<'a> DiagnosticNative<'a> {
    /// Its native error kind.
    #[must_use]
    pub fn kind(&self) -> DiagnosticNativeKind {
        let NativeData::Original(data) = self.data else {
            return DiagnosticNativeKind::Type;
        };
        match data.kind {
            StoredNativeErrorKind::Error => DiagnosticNativeKind::Error,
            StoredNativeErrorKind::Eval => DiagnosticNativeKind::Eval,
            StoredNativeErrorKind::Range => DiagnosticNativeKind::Range,
            StoredNativeErrorKind::Reference => DiagnosticNativeKind::Reference,
            StoredNativeErrorKind::Syntax => DiagnosticNativeKind::Syntax,
            StoredNativeErrorKind::Type => DiagnosticNativeKind::Type,
            StoredNativeErrorKind::Uri => DiagnosticNativeKind::Uri,
            StoredNativeErrorKind::Aggregate(_) => DiagnosticNativeKind::Aggregate,
        }
    }
    /// Borrows the exact visible native message.
    #[must_use]
    pub fn message(&self) -> &'a str {
        match self.data {
            NativeData::Original(data) => &data.message,
            NativeData::Replacement(replacement) => replacement.message(),
        }
    }
    /// Borrows its visible cause, if present.
    #[must_use]
    pub fn cause(&self) -> Option<Diagnostic<'a>> {
        match self.data {
            NativeData::Original(data) => data.cause.as_deref().map(|node| Diagnostic {
                node,
                mask: self.mask,
            }),
            NativeData::Replacement(_) => None,
        }
    }
    /// Iterates aggregate members in authored order; empty for other kinds.
    #[must_use]
    pub fn members(&self) -> DiagnosticMembers<'a> {
        let members = match self.data {
            NativeData::Original(StoredNativeError {
                kind: StoredNativeErrorKind::Aggregate(members),
                ..
            }) => members.as_slice(),
            _ => &[],
        };
        DiagnosticMembers {
            members: members.iter(),
            mask: self.mask,
        }
    }
    /// Its visible native realm identity without creating a realm root.
    #[must_use]
    pub fn realm_identity(&self) -> Option<ObjectId> {
        match self.data {
            NativeData::Original(data) => data.realm_identity,
            NativeData::Replacement(_) => None,
        }
    }
    /// Borrows the original stack or the local replacement's native position.
    #[must_use]
    pub fn stack(&self) -> DiagnosticStack<'a> {
        match self.data {
            NativeData::Original(data) => match &data.stack.0 {
                ErrorStack::Position(frame) => {
                    DiagnosticStack::Position(DiagnosticFrame::from_entry(frame))
                }
                ErrorStack::Backtrace(trace) => DiagnosticStack::Backtrace(DiagnosticBacktrace {
                    frames: trace.frames(),
                }),
            },
            NativeData::Replacement(replacement) => {
                DiagnosticStack::Position(DiagnosticFrame::Native {
                    function_name: None,
                    source_info: replacement.source_info(),
                })
            }
        }
    }
}
impl PartialEq for DiagnosticNative<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
            && self.message() == other.message()
            && self.realm_identity() == other.realm_identity()
            && self.cause() == other.cause()
            && self.members().eq(other.members())
    }
}
impl Eq for DiagnosticNative<'_> {}

/// Exact engine diagnostic payloads. Nested Panic sources remain borrowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticEngine<'a> {
    /// Exact refusal encountered while acquiring roots.
    Rooting(RootingError),
    /// Publication refused a selected realm from another runtime.
    ForeignSelectedRealm(DiagnosticForeignSelectedRealm<'a>),
    /// Exact timer operands whose sum cannot be represented.
    JobDeadlineOverflow {
        /// Sampled publication clock instant.
        now: JsInstant,
        /// Exact requested delay.
        delay: JsDuration,
    },
    /// A retained source-module interruption and its original cause.
    ModuleInterrupted(DiagnosticModuleInterruption<'a>),
    /// Engine resource refusal.
    Resource(EngineResourceError),
    /// Entry authority did not match.
    InvalidEntryPermit,
    /// Context access is currently leased.
    ContextAccessBusy,
    /// A live object refused the required access or native projection.
    ObjectAccess(ObjectAccessError),
    /// Exact value rendering refusal.
    ValueRender(crate::value::ValueRenderError),
    /// Work requires the same exact Context identity.
    JobContextRequired {
        /// Context-free diagnostic identity, without liveness/access authority.
        required: &'a crate::job::ContextId,
    },
    /// Owned work was retired.
    JobRetired(crate::job::JobRetirement),
    /// A dynamic job operation is invalid for its current state.
    Job(JobError),
    /// Invocation identities are exhausted.
    InvocationIdExhausted,
    /// Scope publication refusal.
    ScopeAnalysis(surgeist_js_ast::scope::ScopeStateError),
    /// Exact configured limit refusal.
    RuntimeLimit(RuntimeLimitError),
    /// Fuzz instruction availability was exhausted.
    #[cfg(feature = "fuzz")]
    NoInstructionsRemain,
    /// An enclosing engine Panic and its diagnostic source.
    Panic {
        /// Original Panic message.
        message: &'a str,
        /// Its visible diagnostic source, if present.
        diagnostic: Option<Diagnostic<'a>>,
    },
}
/// Borrowed authored aggregate member iterator.
#[derive(Clone)]
pub struct DiagnosticMembers<'a> {
    members: slice::Iter<'a, StoredJsError>,
    mask: Mask,
}
impl<'a> Iterator for DiagnosticMembers<'a> {
    type Item = Diagnostic<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.members.next().map(|node| Diagnostic {
            node,
            mask: self.mask,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.members.size_hint()
    }
}
impl DoubleEndedIterator for DiagnosticMembers<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.members.next_back().map(|node| Diagnostic {
            node,
            mask: self.mask,
        })
    }
}
impl ExactSizeIterator for DiagnosticMembers<'_> {}
impl std::iter::FusedIterator for DiagnosticMembers<'_> {}
impl fmt::Debug for DiagnosticMembers<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
impl PartialEq for Diagnostic<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}
impl Eq for Diagnostic<'_> {}

/// A borrowed immutable capture, preserving absent-versus-empty distinctions.
#[derive(Clone, Copy)]
pub struct DiagnosticBacktrace<'a> {
    frames: &'a [ShadowEntry],
}
impl<'a> DiagnosticBacktrace<'a> {
    /// Captured frames in oldest-to-newest order.
    #[must_use]
    pub fn frames(self) -> DiagnosticFrames<'a> {
        DiagnosticFrames {
            frames: self.frames.iter(),
        }
    }
}
/// Iterator over captured frame provenance.
#[derive(Clone)]
pub struct DiagnosticFrames<'a> {
    frames: slice::Iter<'a, ShadowEntry>,
}
impl<'a> Iterator for DiagnosticFrames<'a> {
    type Item = DiagnosticFrame<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.frames.next().map(DiagnosticFrame::from_entry)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.frames.size_hint()
    }
}
impl DoubleEndedIterator for DiagnosticFrames<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.frames.next_back().map(DiagnosticFrame::from_entry)
    }
}
impl ExactSizeIterator for DiagnosticFrames<'_> {}
impl std::iter::FusedIterator for DiagnosticFrames<'_> {}
impl fmt::Debug for DiagnosticFrames<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
/// The distinct position/captured-stack forms of a native diagnostic.
#[derive(Debug, Clone, Copy)]
pub enum DiagnosticStack<'a> {
    /// A single construction/source position.
    Position(DiagnosticFrame<'a>),
    /// A captured call stack.
    Backtrace(DiagnosticBacktrace<'a>),
}
/// Borrowed exact origin information for one captured frame.
#[derive(Debug, Clone, Copy)]
pub enum DiagnosticFrame<'a> {
    /// A native frame and its optional compiler-feature source location.
    Native {
        /// Optional native function name.
        function_name: Option<&'a JsString>,
        /// Native construction/call site metadata.
        source_info: NativeSourceInfo,
    },
    /// A bytecode frame and its original source.
    Bytecode {
        /// Captured bytecode offset.
        pc: u32,
        /// Original function name.
        function_name: &'a JsString,
        /// Original source path/category.
        path: &'a SourcePath,
        /// Source-map position for the captured offset.
        position: Option<surgeist_js_ast::Position>,
        /// Original spanned source text.
        source_text: &'a SpannedSourceText,
    },
}
impl<'a> DiagnosticFrame<'a> {
    fn from_entry(entry: &'a ShadowEntry) -> Self {
        match entry {
            ShadowEntry::Native {
                function_name,
                source_info,
            } => Self::Native {
                function_name: function_name.as_ref(),
                source_info: *source_info,
            },
            ShadowEntry::Bytecode { pc, source_info } => Self::Bytecode {
                pc: *pc,
                function_name: source_info.function_name(),
                path: source_info.map().path(),
                position: source_info.map().find(*pc),
                source_text: source_info.text_spanned(),
            },
        }
    }
    fn display(self, f: &mut fmt::Formatter<'_>, show_name: bool) -> fmt::Result {
        match self {
            Self::Native {
                function_name,
                source_info,
            } => {
                if show_name {
                    if let Some(name) = function_name {
                        write!(f, "{}", name.to_std_string_escaped())?;
                    } else {
                        f.write_str("<anonymous>")?;
                    }
                }
                if let Some(location) = source_info.as_location() {
                    write!(
                        f,
                        " (native at {}:{}:{})",
                        location.file(),
                        location.line(),
                        location.column()
                    )
                } else {
                    f.write_str(" (native)")
                }
            }
            Self::Bytecode {
                function_name,
                path,
                position,
                ..
            } => {
                if show_name {
                    if function_name.is_empty() {
                        f.write_str("<main>")?;
                    } else {
                        write!(f, "{}", function_name.to_std_string_escaped())?;
                    }
                }
                write!(f, " ({path}")?;
                if let Some(position) = position {
                    write!(
                        f,
                        ":{}:{}",
                        position.line_number(),
                        position.column_number()
                    )?;
                } else {
                    f.write_str(":?:?")?;
                }
                f.write_str(")")
            }
        }
    }
}
impl fmt::Display for DiagnosticFrame<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, true)
    }
}
impl fmt::Debug for DiagnosticBacktrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.frames()).finish()
    }
}
impl fmt::Display for DiagnosticBacktrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for frame in self.frames().rev() {
            write!(f, "\n    at {frame}")?;
        }
        Ok(())
    }
}
impl fmt::Display for DiagnosticStack<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Position(frame) => frame.fmt(f),
            Self::Backtrace(trace) => trace.fmt(f),
        }
    }
}
impl fmt::Debug for DiagnosticOpaque<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            StoredValue::Object(object) => {
                f.debug_tuple("Object").field(&object.identity()).finish()
            }
            value => value.fmt(f),
        }
    }
}
impl fmt::Debug for DiagnosticNative<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiagnosticNative")
            .field("kind", &self.kind())
            .field("message", &self.message())
            .field("cause", &self.cause())
            .field("members", &self.members())
            .field("realm", &self.realm_identity())
            .finish()
    }
}
impl fmt::Debug for Diagnostic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diagnostic")
            .field("kind", &self.kind())
            .field("backtrace", &self.backtrace())
            .finish()
    }
}
impl fmt::Display for DiagnosticOpaque<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value() {
            Ok(value) => value.display().fmt(f),
            Err(_) => f.write_str("<unavailable diagnostic object>"),
        }
    }
}
impl fmt::Display for DiagnosticNative<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            DiagnosticNativeKind::Aggregate => f.write_str("AggregateError")?,
            kind => f.write_str(match kind {
                DiagnosticNativeKind::Error => "Error",
                DiagnosticNativeKind::Eval => "EvalError",
                DiagnosticNativeKind::Range => "RangeError",
                DiagnosticNativeKind::Reference => "ReferenceError",
                DiagnosticNativeKind::Syntax => "SyntaxError",
                DiagnosticNativeKind::Type => "TypeError",
                DiagnosticNativeKind::Uri => "UriError",
                DiagnosticNativeKind::Aggregate => unreachable!(),
            })?,
        }
        let message = self.message().trim();
        if !message.is_empty() {
            write!(f, ": {message}")?;
        }
        let position = match self.stack() {
            DiagnosticStack::Position(frame) => Some(frame),
            DiagnosticStack::Backtrace(trace) => trace.frames().next(),
        };
        if let Some(position) = position {
            position.display(f, false)?;
        }
        Ok(())
    }
}
impl fmt::Display for DiagnosticEngine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignSelectedRealm(_) => {
                f.write_str("ForeignSelectedRealm: selected realm belongs to a different runtime")
            }
            Self::JobDeadlineOverflow { now, delay } => write!(
                f,
                "JobDeadlineOverflow: {now:?} + {delay:?} is unrepresentable"
            ),
            Self::Rooting(value) => write!(f, "RootingError: {value}"),
            Self::Resource(value) => write!(f, "EngineResourceError: {value}"),
            Self::InvalidEntryPermit => {
                f.write_str("InvalidEntryPermit: execution entry authority does not match")
            }
            Self::ContextAccessBusy => {
                f.write_str("ContextAccessBusy: host context access is already leased")
            }
            Self::ObjectAccess(error) => write!(f, "ObjectAccessError: {error}"),
            Self::ValueRender(error) => write!(f, "ValueRenderError: {error}"),
            Self::JobContextRequired { required } => {
                write!(f, "JobContextRequired: task requires Context {required}")
            }
            Self::JobRetired(reason) => write!(f, "JobRetired: {reason:?}"),
            Self::Job(error) => write!(f, "JobError: {error}"),
            Self::InvocationIdExhausted => {
                f.write_str("InvocationIdExhausted: invocation identity space exhausted")
            }
            Self::ScopeAnalysis(value) => write!(f, "ScopeAnalysisError: {value}"),
            Self::RuntimeLimit(value) => write!(f, "RuntimeLimitError: {value}"),
            Self::Panic { message, .. } => write!(f, "EnginePanic: {message}"),
            Self::ModuleInterrupted(value) => write!(f, "ModuleInterrupted: {}", value.cause()),
            #[cfg(feature = "fuzz")]
            Self::NoInstructionsRemain => {
                f.write_str("NoInstructionsRemainError: instruction budget was exhausted")
            }
        }
    }
}
impl fmt::Display for DiagnosticKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opaque(value) => value.fmt(f),
            Self::Native(native) => native.fmt(f),
            Self::Engine(engine) => engine.fmt(f),
        }
    }
}
impl fmt::Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind().fmt(f)?;
        if let Some(trace) = self.backtrace() {
            trace.fmt(f)?;
        }
        Ok(())
    }
}
