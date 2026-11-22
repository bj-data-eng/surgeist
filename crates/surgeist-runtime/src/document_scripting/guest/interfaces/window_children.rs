//! WindowProxy references backed by the attachment's live browsing contexts.

use super::*;
use crate::document_scripting::window_relations::{self, Member, Relation};
use surgeist_bindings_js::{ChildWindow, IntegerKind, IntegerMode, IntegerValue, ValueKind};
use surgeist_dom::{
    DomSnapshot,
    hosted::history::DomHostedNavigable,
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
    relations: Rc<ChildRelations>,
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
                Ok(relations.is_live(read)?.then_some(Target {
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

    fn is_live(self, read: &NativeRead<'_>) -> Result<bool, DocumentGuestEventError> {
        let record = read
            .snapshot
            .captured_hosted_navigable_with(self.navigable, &mut || read.step())?;
        if (record.global(), record.browsing_context()) != (self.global, self.context) {
            return Err(DocumentScriptNativeError::ProjectionCustody.into());
        }
        if record.is_retiring()
            || record.is_retired()
            || record.active_document() != Some(self.document)
        {
            return Ok(false);
        }
        read.snapshot
            .document(self.document)?
            .captured_fully_active_with(&mut || read.step())
    }
}

pub(in crate::document_scripting::guest) struct ChildRelations {
    callbacks: Rc<Callbacks>,
    child: Child,
    retired: Cell<bool>,
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
        if let Some(result) =
            call_child_window_relation(scope, &arguments, self.member, self.setter)?
        {
            return Ok(result);
        }
        let callbacks = &self.relations.callbacks;
        let owner = callbacks.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        let events = callbacks
            .access
            .get()
            .and_then(|access| access.events.upgrade())
            .ok_or(Error::ExpiredIdentity)?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let attached = public.window(scope)?;
        if !scope.strict_equals(arguments.receiver(), &attached)? {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Window required"))?,
            ));
        }
        window_relations::access_attached(
            scope,
            &owner,
            &callbacks.permit,
            &arguments,
            self.member,
            self.setter,
        )
    }
}

pub(in crate::document_scripting) fn call_child_window_relation<'scope>(
    scope: &mut Scope<'scope>,
    arguments: &Arguments<'scope>,
    member: &Member,
    setter: bool,
) -> Result<Option<Completion<'scope>>, Error> {
    let Some(relations) = scope.child_window_payload::<ChildRelations>(arguments.receiver())?
    else {
        return Ok(None);
    };
    relations.call(scope, arguments, member, setter).map(Some)
}

impl ChildRelations {
    pub(in crate::document_scripting::guest) fn is_retired(&self) -> bool {
        self.retired.get()
    }

    pub(in crate::document_scripting::guest) fn global(&self) -> DomHostedGlobalId {
        self.child.global
    }

    pub(super) fn document(&self) -> DocumentId {
        self.child.document
    }

    fn is_live(&self, read: &NativeRead<'_>) -> Result<bool, DocumentGuestEventError> {
        Ok(!self.is_retired() && self.child.is_live(read)?)
    }

    fn call<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        member: &Member,
        setter: bool,
    ) -> Result<Completion<'scope>, Error> {
        let callbacks = &self.callbacks;
        let owner = callbacks.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        let events = callbacks
            .access
            .get()
            .and_then(|access| access.events.upgrade())
            .ok_or(Error::ExpiredIdentity)?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let child = self.child;
        let proxy = arguments.receiver();
        let result = (|| {
            let (live, count) = callbacks.with_read(&owner, &events, |read, _| {
                let live = self.is_live(read)?;
                let count = if live && !setter && matches!(member.relation, Relation::Length) {
                    child_count(read, child.document)?
                } else {
                    0
                };
                Ok((live, count))
            })?;
            if setter {
                let undefined;
                let value = if let Some(value) = arguments.get(0) {
                    value
                } else {
                    undefined = scope.primitive(Primitive::Undefined)?;
                    &undefined
                };
                if matches!(member.relation, Relation::Opener)
                    && scope.kind(value)? == ValueKind::Null
                {
                    return Ok(Completion::Undefined);
                }
                events.charge_work(member.key.len() as u64)?;
                return Ok(window_relations::replace_attribute(
                    scope, proxy, member, value,
                )?);
            }
            let value = match member.relation {
                Relation::WindowProxy => scope.import(&scope.retain(proxy))?,
                Relation::Parent if live => match child.parent.context {
                    Some(_) => public
                        .child_window(scope, &owner, &events, child.parent.global)?
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

fn describe_child(
    read: &NativeRead<'_>,
    record: DomHostedNavigable<'_>,
    document: DocumentId,
) -> Result<Child, DocumentGuestEventError> {
    let parent_id = record
        .parent()
        .or(record.former_parent())
        .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
    let parent = read
        .snapshot
        .captured_hosted_navigable_with(parent_id, &mut || read.step())?;
    let parent_context = (parent.parent().is_some() || parent.former_parent().is_some())
        .then(|| parent.browsing_context());
    let parent_document = read
        .snapshot
        .global_document_with(parent.global(), &mut || read.step())?;
    Ok(Child {
        context: record.browsing_context(),
        navigable: record.id(),
        global: record.global(),
        document,
        parent: Target {
            context: parent_context,
            global: parent.global(),
            document: parent_document,
        },
    })
}

impl PublicEventBindings {
    /// Admit the original profile for a same-origin native child before any
    /// guest operation uses that child's prototypes. Retired children keep
    /// creation provenance without regaining a live Window registry entry.
    pub(super) fn establish_child_origin<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        origin: DomHostedGlobalId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        let Some((child, live)) = self.child_by_origin(owner, events, origin)? else {
            return Ok(None);
        };
        self.establish_child_from_attached(scope, owner, events, child, live)
    }

    pub(in crate::document_scripting::guest) fn child_window<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        global: DomHostedGlobalId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        if let Some(window) = self.window_children.open_global(scope, events, global)? {
            return Ok(Some(window));
        }
        let Some((child, true)) = self.child_by_origin(owner, events, global)? else {
            return Ok(None);
        };
        self.establish_child_from_attached(scope, owner, events, child, true)
    }

    fn child_by_origin(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        origin: DomHostedGlobalId,
    ) -> Result<Option<(Child, bool)>, DocumentGuestEventError> {
        let attachment = owner.attachment.get().ok_or(Error::ExpiredIdentity)?;
        self.callbacks.with_read(owner, events, |read, attached| {
            let document = read
                .snapshot
                .global_document_with(origin, &mut || read.step())?;
            let Some(record) = read
                .snapshot
                .hosted_creation_navigable_with(document, &mut || read.step())?
            else {
                return Ok(None);
            };
            if record.traversable() != attachment.target.navigable() || record.container().is_none()
            {
                return Ok(None);
            }
            let child = describe_child(read, record, document)?;
            let top = read.node(self.callbacks.permit.document.node())?;
            let DomNodeKind::Document(top) = top.kind() else {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            };
            let mut cursor = record;
            let mut document = document;
            while cursor.id() != attachment.target.navigable() {
                let node = read.node(document.node())?;
                let DomNodeKind::Document(metadata) = node.kind() else {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                };
                if metadata.origin() != top.origin() {
                    return Ok(None);
                }
                let parent = cursor
                    .parent()
                    .or(cursor.former_parent())
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                cursor = read
                    .snapshot
                    .captured_hosted_navigable_with(parent, &mut || read.step())?;
                document = read
                    .snapshot
                    .global_document_with(cursor.global(), &mut || read.step())?;
            }
            if cursor.global() != attached || document != self.callbacks.permit.document {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            }
            Ok(Some((child, child.is_live(read)?)))
        })
    }

    fn establish_child_from_attached<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        child: Child,
        live: bool,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        if scope.realm_authority()?.identity() == self.callbacks.permit.realm {
            return self.establish_child(scope, owner, events, child, live);
        }
        let window = self.window(scope)?;
        let window = scope.callback_object(&window)?;
        let proxy = scope.in_callback_object_realm(&window, |scope, _| {
            let proxy = self.establish_child(scope, owner, events, child, live)?;
            Ok::<_, DocumentGuestEventError>(proxy.as_ref().map(|value| scope.retain(value)))
        })?;
        proxy
            .as_ref()
            .map(|root| scope.import(root).map_err(Into::into))
            .transpose()
    }
}

pub(in crate::document_scripting::guest) fn child_window_event_receiver<'scope>(
    scope: &Scope<'scope>,
    receiver: &Value<'scope>,
    owner: &NativeOwner,
    events: &GuestEvents,
) -> Result<Option<Rc<ChildRelations>>, DocumentGuestEventError> {
    let Some(relations) = scope.child_window_payload::<ChildRelations>(receiver)? else {
        return Ok(None);
    };
    let live = relations
        .callbacks
        .with_read(owner, events, |read, _| relations.is_live(read))?;
    Ok(live.then_some(relations))
}

pub(in crate::document_scripting::guest) fn was_child_window<'scope>(
    scope: &Scope<'scope>,
    value: &Value<'scope>,
    global: DomHostedGlobalId,
) -> Result<bool, Error> {
    Ok(scope
        .child_window_payload::<ChildRelations>(value)?
        .is_some_and(|relations| relations.child.global == global))
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
    /// Release live registry custody at a published native revision. DOM's
    /// retired Documents can still create nodes with their original global, so
    /// Runtime retains the corresponding creation prototypes in OriginBindings.
    /// Cleanup needs no new guest admission.
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
            match entry.relations.child.is_retired(snapshot) {
                Ok(true) => {
                    if let Some(mut entry) = cursor.take() {
                        entry.relations.retired.set(true);
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
        self.lookup_where(|child| child.context == context, events, project)
    }

    pub(in crate::document_scripting::guest) fn open_global<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        global: DomHostedGlobalId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        self.lookup_where(
            |child| child.global == global,
            events,
            |entry| Ok(entry.proxy.open(scope)?),
        )
    }

    fn lookup_where<T>(
        &self,
        matches: impl Fn(Child) -> bool,
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
            if matches(entry.relations.child) {
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
        let proxy = public
            .establish_child(scope, owner, events, child, true)?
            .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
        Ok(Completion::Return(proxy))
    }
}

impl PublicEventBindings {
    fn establish_child<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        child: Child,
        live: bool,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        let registry = &self.window_children;
        if let Some(proxy) = registry.lookup(child.context, events, |candidate| {
            if candidate.relations.child != child {
                Err(DocumentScriptNativeError::ProjectionCustody.into())
            } else {
                Ok(candidate.proxy.open(scope)?)
            }
        })? {
            return Ok(Some(proxy));
        }

        let backing = live
            .then(|| events.reserve(owner, size_of::<Entry>()))
            .transpose()?;
        let bytes = window_relations::child_callback_count()
            .checked_mul(size_of::<ChildAccess>())
            .and_then(|bytes| {
                bytes.checked_add(size_of::<ChildRelations>() + 2 * size_of::<usize>())
            })
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let source = Rc::new(ChildRelations {
            callbacks: self.callbacks.clone(),
            child,
            retired: Cell::new(!live),
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
        let (proxy, origin) = scope.create_same_origin_window(&source, |scope, target| {
            window_relations::install_child_members(scope, |member, setter| {
                let access = ChildAccess {
                    relations: source.clone(),
                    member,
                    setter,
                };
                Callback::new(move |scope, arguments| access.call(scope, arguments))
            })?;
            self.build_origin(
                scope,
                events,
                OriginInstall {
                    global: child.global,
                    document: child.document,
                    registry: document_registry,
                    dom_exception_limits: owner.dom_exception_limits.get(),
                    admission,
                    conversion_host: self
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
        let Some(backing) = backing else {
            self.publish_origin(origin)?;
            return Ok(None);
        };
        let value = proxy.open(scope)?;
        let mut children = registry
            .live
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        self.publish_origin(origin)?;
        let next = children.take();
        *children = Some(Box::new(Entry {
            relations: source,
            proxy,
            next,
            _backing: backing,
        }));
        Ok(Some(value))
    }
}
