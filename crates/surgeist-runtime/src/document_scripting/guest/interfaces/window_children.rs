//! WindowProxy references backed by the attachment's live browsing contexts.

use super::*;
use crate::document_scripting::window_relations::{self, Member, Relation};
use surgeist_bindings_js::{ChildWindow, IntegerKind, IntegerMode, IntegerValue, ValueKind};
use surgeist_dom::{
    DomSnapshot,
    hosted::{DomHostedBrowsingContextId, DomHostedError, DomHostedNavigableId},
    identity::DomStamp,
    tree::DomNodeKind,
};

#[derive(Default)]
pub(super) struct Children {
    live: RefCell<Option<Box<Entry>>>,
    reconciled: Cell<Option<DomStamp>>,
}

struct Entry {
    child: Child,
    proxy: ChildWindow,
    next: Option<Box<Entry>>,
    _backing: GuestBacking,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Child {
    context: DomHostedBrowsingContextId,
    navigable: DomHostedNavigableId,
    global: DomHostedGlobalId,
    document: DocumentId,
    parent: Target,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Target {
    context: Option<DomHostedBrowsingContextId>,
    pub(super) global: DomHostedGlobalId,
    document: DocumentId,
}

#[derive(Clone)]
pub(super) enum WindowSite {
    Attached,
    Child(Rc<ChildRelations>),
}

impl WindowSite {
    pub(super) fn qualify(
        &self,
        capture: &Capture,
        read: &NativeRead<'_>,
        attached: DomHostedGlobalId,
    ) -> Result<Option<Target>, DocumentGuestEventError> {
        match self {
            Self::Attached => Ok(Some(Target {
                context: None,
                global: attached,
                document: capture.callbacks.permit.document,
            })),
            Self::Child(relations) => {
                let child = relations.child;
                Ok(child.is_live(read)?.then_some(Target {
                    context: Some(child.context),
                    global: child.global,
                    document: child.document,
                }))
            }
        }
    }
}

impl Child {
    fn is_retired(self, snapshot: &DomSnapshot) -> Result<bool, DomHostedError> {
        let mut cursor = Some(self.navigable);
        while let Some(id) = cursor {
            let navigable = match snapshot.hosted_navigable(id) {
                Ok(navigable) => navigable,
                Err(DomHostedError::StaleIdentity) => return Ok(true),
                Err(error) => return Err(error),
            };
            if navigable.is_retiring() || navigable.is_retired() {
                return Ok(true);
            }
            cursor = navigable.parent();
        }
        Ok(false)
    }

    fn is_live(mut self, read: &NativeRead<'_>) -> Result<bool, DocumentGuestEventError> {
        let public = read.events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        loop {
            if !self.in_parent(read)? {
                return Ok(false);
            }
            let Some(parent) = self.parent.context else {
                return Ok(true);
            };
            let Some(ancestor) = public
                .window_children
                .lookup(parent, read.events, |entry| Ok(entry.child))?
            else {
                return Ok(false);
            };
            if (ancestor.global, ancestor.document) != (self.parent.global, self.parent.document) {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            }
            self = ancestor;
        }
    }

    fn in_parent(self, read: &NativeRead<'_>) -> Result<bool, DocumentGuestEventError> {
        let dom = read
            .dom
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut live = false;
        let result = dom.raw().visit_document_tree_child_navigables(
            self.parent.document,
            read.events.limits.work,
            || read.events.budget.charge_work(read.account_scope, 1),
            read.cancel,
            |candidate| {
                if candidate.browsing_context() == self.context {
                    if candidate.id() != self.navigable
                        || candidate.active_document() != Some(self.document)
                        || candidate.global() != self.global
                    {
                        return Err(DocumentScriptNativeError::ProjectionCustody.into());
                    }
                    live = true;
                }
                Ok(())
            },
        );
        match result {
            Ok(_) => Ok(live),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Dom(
                    DomHostedError::StaleIdentity,
                )),
            ))) if self.parent.context.is_some() => Ok(false),
            Err(error) => Err(error),
        }
    }
}

pub(super) struct ChildRelations {
    callbacks: Rc<Callbacks>,
    child: Child,
    _backing: GuestBacking,
}

struct ChildAccess {
    relations: Rc<ChildRelations>,
    member: &'static Member,
    setter: bool,
}

impl ChildAccess {
    fn call<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let proxy = scope.global()?;
        if !scope.strict_equals(arguments.receiver(), &proxy)? {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Window required"))?,
            ));
        }
        let callbacks = &self.relations.callbacks;
        let owner = callbacks.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        let events = callbacks
            .access
            .get()
            .and_then(|access| access.events.upgrade())
            .ok_or(Error::ExpiredIdentity)?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let child = self.relations.child;
        let result = (|| {
            let (live, count) = callbacks.with_read(&owner, &events, |read, _| {
                let live = child.is_live(read)?;
                let count =
                    if live && !self.setter && matches!(self.member.relation, Relation::Length) {
                        child_count(read, child.document)?
                    } else {
                        0
                    };
                Ok((live, count))
            })?;
            if self.setter {
                let undefined;
                let value = if let Some(value) = arguments.get(0) {
                    value
                } else {
                    undefined = scope.primitive(Primitive::Undefined)?;
                    &undefined
                };
                if matches!(self.member.relation, Relation::Opener)
                    && scope.kind(value)? == ValueKind::Null
                {
                    return Ok(Completion::Undefined);
                }
                events.charge_work(self.member.key.len() as u64)?;
                return Ok(window_relations::replace_attribute(
                    scope,
                    &proxy,
                    self.member,
                    value,
                )?);
            }
            let value = match self.member.relation {
                Relation::WindowProxy => proxy,
                Relation::Parent if live => match child.parent.context {
                    Some(context) => public
                        .window_children
                        .lookup(context, &events, |entry| Ok(entry.proxy.open(scope)?))?
                        .ok_or(DocumentScriptNativeError::ProjectionCustody)?,
                    None => scope.open_callback_object(&public.window)?,
                },
                Relation::Parent | Relation::Opener => scope.primitive(Primitive::Null)?,
                Relation::Closed => scope.primitive(Primitive::Boolean(!live))?,
                Relation::Length => scope.primitive(Primitive::Number(f64::from(count)))?,
            };
            Ok(Completion::Return(value))
        })();
        document::tree::tree_completion(scope, &events, result)
    }
}

fn child_count(
    read: &NativeRead<'_>,
    document: DocumentId,
) -> Result<u32, DocumentGuestEventError> {
    let dom = read
        .dom
        .try_borrow()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let count = dom.raw().visit_document_tree_child_navigables(
        document,
        read.events.limits.work,
        || read.events.budget.charge_work(read.account_scope, 1),
        read.cancel,
        |_| Ok(()),
    )?;
    Ok(u32::try_from(count).map_err(|_| Error::ArrayTooLong)?)
}

impl Drop for Children {
    fn drop(&mut self) {
        release_entries(self.live.get_mut().take());
    }
}

fn release_entries(mut entries: Option<Box<Entry>>) {
    while let Some(mut entry) = entries {
        entries = entry.next.take();
    }
}

impl Children {
    /// Release registry custody at a published native revision. Existing guest
    /// references and the separate creation-provenance graph retain their
    /// original realms. Cleanup needs no new guest admission.
    pub(super) fn reap(&self, snapshot: &DomSnapshot) -> bool {
        if self.reconciled.get() == Some(snapshot.stamp()) {
            return false;
        }
        let Ok(mut live) = self.live.try_borrow_mut() else {
            return false;
        };
        let mut retired = None;
        let mut complete = true;
        let mut cursor = &mut *live;
        while let Some(entry) = cursor.as_ref() {
            match entry.child.is_retired(snapshot) {
                Ok(true) => {
                    if let Some(mut entry) = cursor.take() {
                        *cursor = entry.next.take();
                        entry.next = retired.take();
                        retired = Some(entry);
                    }
                    continue;
                }
                Ok(false) => (),
                Err(_) => complete = false,
            }
            let Some(entry) = cursor.as_mut() else {
                break;
            };
            cursor = &mut entry.next;
        }
        if complete {
            self.reconciled.set(Some(snapshot.stamp()));
        }
        drop(live);
        let released = retired.is_some();
        release_entries(retired);
        released
    }

    fn lookup<T>(
        &self,
        context: DomHostedBrowsingContextId,
        events: &GuestEvents,
        project: impl FnOnce(&Entry) -> Result<T, DocumentGuestEventError>,
    ) -> Result<Option<T>, DocumentGuestEventError> {
        let children = self
            .live
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut next = children.as_deref();
        while let Some(entry) = next {
            events.step()?;
            if entry.child.context == context {
                return project(entry).map(Some);
            }
            next = entry.next.as_deref();
        }
        Ok(None)
    }
}

pub(super) fn install(
    scope: &mut Scope<'_>,
    capture: CaptureFunding<'_>,
    site: WindowSite,
) -> Result<(), DocumentGuestEventError> {
    let read = capture(size_of::<(Capture, WindowSite)>())?;
    scope.install_window_index_reader(&Callback::new(move |scope, arguments| {
        read.read_window_index(scope, arguments, &site)
    }))?;
    Ok(())
}

impl Capture {
    fn read_window_index<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        site: &WindowSite,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let Some(value) = arguments.get(0) else {
                let count =
                    self.callbacks
                        .with_read(&owner, &events, |read, attached| {
                            match site.qualify(self, read, attached)? {
                                Some(target) => child_count(read, target.document),
                                None => Ok(0),
                            }
                        })?;
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Number(f64::from(count)))?,
                ));
            };
            let IntegerValue::UnsignedLong(index) =
                scope.integer(value, IntegerKind::UnsignedLong, IntegerMode::EnforceRange)?
            else {
                return Err(Error::ExpectedPropertyKey.into());
            };
            let context = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    let Some(target) = site.qualify(self, read, attached)? else {
                        return Ok(None);
                    };
                    let dom = read
                        .dom
                        .try_borrow()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    dom.raw().window_indexed_child(
                        target.global,
                        index as usize,
                        events.limits.work,
                        || events.budget.charge_work(read.account_scope, 1),
                        read.cancel,
                    )
                })?;
            if arguments.get(1).is_some() {
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Boolean(context.is_some()))?,
                ));
            }
            match context {
                Some(context) => self.project_window_child(scope, &owner, &events, site, context),
                None => Ok(Completion::Undefined),
            }
        })();
        document::tree::tree_completion(scope, &events, result)
    }

    pub(super) fn project_window_child<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        site: &WindowSite,
        context: DomHostedBrowsingContextId,
    ) -> Result<Completion<'scope>, DocumentGuestEventError> {
        let (child, same_origin) = self.callbacks.with_read(owner, events, |read, attached| {
            let parent_target = site
                .qualify(self, read, attached)?
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let dom = read
                .dom
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let document = parent_target.document;
            let root = read.node(document.node())?;
            let DomNodeKind::Document(parent) = root.kind() else {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            };
            let mut selected = None;
            dom.raw().visit_document_tree_child_navigables(
                document,
                events.limits.work,
                || events.budget.charge_work(read.account_scope, 1),
                read.cancel,
                |child| {
                    if child.browsing_context() == context {
                        let document = child
                            .active_document()
                            .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                        let node = read.node(document.node())?;
                        let DomNodeKind::Document(child_document) = node.kind() else {
                            return Err(DocumentScriptNativeError::ProjectionCustody.into());
                        };
                        selected = Some((
                            Child {
                                context,
                                navigable: child.id(),
                                global: child.global(),
                                document,
                                parent: parent_target,
                            },
                            child_document.origin() == parent.origin(),
                        ));
                    }
                    Ok(())
                },
            )?;
            selected.ok_or_else(|| DocumentScriptNativeError::ProjectionCustody.into())
        })?;
        if !same_origin {
            return Err(DocumentScriptNativeError::ProjectionCustody.into());
        }

        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let registry = &public.window_children;
        if let Some(proxy) = registry.lookup(context, events, |candidate| {
            if candidate.child != child {
                Err(DocumentScriptNativeError::ProjectionCustody.into())
            } else {
                Ok(candidate.proxy.open(scope)?)
            }
        })? {
            return Ok(Completion::Return(proxy));
        }

        let backing = events.reserve(owner, size_of::<Entry>())?;
        let bytes = window_relations::child_callback_count()
            .checked_mul(size_of::<ChildAccess>())
            .and_then(|bytes| {
                bytes.checked_add(size_of::<ChildRelations>() + 2 * size_of::<usize>())
            })
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let source = Rc::new(ChildRelations {
            callbacks: self.callbacks.clone(),
            child,
            _backing: events.reserve(owner, bytes)?,
        });
        let document_registry = self.callbacks.with_read(owner, events, |read, _| {
            read.step()?;
            Ok(custom_elements::document_registry(
                &read.snapshot,
                child.document,
                true,
            )?)
        })?;
        let admission = events
            .prepare_origin(owner)?
            .ok_or(Error::RegistryConflict)?;
        let (proxy, origin) = scope.create_same_origin_window(|scope, target| {
            window_relations::install_child_members(scope, |member, setter| {
                let access = ChildAccess {
                    relations: source.clone(),
                    member,
                    setter,
                };
                Callback::new(move |scope, arguments| access.call(scope, arguments))
            })?;
            public.build_origin(
                scope,
                events,
                OriginInstall {
                    global: child.global,
                    document: child.document,
                    registry: document_registry,
                    dom_exception_limits: owner.dom_exception_limits.get(),
                    admission,
                    conversion_host: public
                        .conversion_host
                        .for_origin(ConversionOrigin::DomException),
                    window: Some(WindowOriginInstall {
                        target,
                        relations: source.clone(),
                        owner,
                    }),
                },
            )
        })?;
        let value = proxy.open(scope)?;
        let mut children = registry
            .live
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        public.publish_origin(origin)?;
        let next = children.take();
        *children = Some(Box::new(Entry {
            child,
            proxy,
            next,
            _backing: backing,
        }));
        Ok(Completion::Return(value))
    }
}
