use std::ops::ControlFlow;

use super::{
    collection_read::{CollectionRead, Meter},
    node_iterator::traversal::OrdinaryTree,
    read_source::HostedReadError,
    *,
};
use crate::internal::{
    document::{DomDocumentClass, DomDocumentMetadata},
    form::FormNamedScope,
    hosted::{DomHostedBrowsingContextId, DomHostedEntry, DomHostedError, DomHostedGlobalId},
    names::{ascii_lower, in_html_namespace_with, qualified_equals_with, same_code_units},
    snapshot::{DomNode, DomNodeKind, DomSiblingDirection},
    token_list::{CLASS, scan_space_tokens},
};

use surgeist_html::{
    construction::HtmlDocumentMode,
    values::HtmlNameInput,
    vocabulary::{
        HtmlDocumentCollection, HtmlDocumentCollectionMatch, HtmlDocumentNamedId, KnownHtmlElement,
        is_document_all_named_local_name,
    },
};

const ID: &[u16] = &[0x69, 0x64];
const NAME: &[u16] = &[0x6e, 0x61, 0x6d, 0x65];
const WILDCARD: &[u16] = &[0x2a];

mod form_controls;
pub use form_controls::{
    DomFormNamedValue, DomHtmlFormControlsCollection, DomHtmlFormControlsCollectionRead,
    DomRadioNodeList, DomRadioNodeListRead,
};

fn document_metadata<'a, E: From<DomError>>(
    snapshot: &'a DomSnapshot,
    root: DomNode<'a>,
    step: impl FnMut() -> Result<(), E>,
) -> Result<&'a DomDocumentMetadata, E> {
    let document = match root.owner_document() {
        Some(document) => snapshot.node_with(document.node(), step)?,
        None => root,
    };
    match document.kind() {
        DomNodeKind::Document(metadata) => Ok(metadata),
        _ => Err(DomError::WrongNodeKind.into()),
    }
}

fn matches_expanded_name<E>(
    name: &(impl HtmlNameInput + ?Sized),
    namespace: Option<&DocumentString>,
    local_name: &DocumentString,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    let namespace_matches = match (name.namespace_uri(), namespace) {
        (_, Some(query)) if query.code_units() == WILDCARD => true,
        (Some(stored), Some(query)) => {
            same_code_units(stored.code_units(), query.code_units(), &mut step)?
        }
        (None, None) => true,
        _ => false,
    };
    Ok(namespace_matches
        && (local_name.code_units() == WILDCARD
            || same_code_units(
                name.local_name().code_units(),
                local_name.code_units(),
                step,
            )?))
}

fn window_name_element<E>(
    element: &(impl HtmlNameInput + ?Sized),
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    if !in_html_namespace_with(element, &mut step)? {
        return Ok(false);
    }
    step()?;
    // HTML's exact recognition is bounded by its longest spelling.
    Ok(matches!(
        KnownHtmlElement::from_units(element.local_name().code_units()),
        Some(
            KnownHtmlElement::Embed
                | KnownHtmlElement::Form
                | KnownHtmlElement::Img
                | KnownHtmlElement::Object
        )
    ))
}

fn document_named_id_policy<E>(
    element: &(impl HtmlNameInput + ?Sized),
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<Option<HtmlDocumentNamedId>, E> {
    if !in_html_namespace_with(element, &mut step)? {
        return Ok(None);
    }
    step()?;
    Ok(HtmlDocumentNamedId::for_local_name(
        element.local_name().code_units(),
    ))
}

/// The exact Document or Element that owns a descendant query.
#[derive(Clone, Copy, Debug)]
pub enum DomDescendantQueryRoot {
    /// Query the descendants of a Document.
    Document(DocumentId),
    /// Query the descendants of an Element, including a detached Element.
    Element(NodeId),
}

impl DomDescendantQueryRoot {
    fn node(self) -> NodeId {
        match self {
            Self::Document(document) => document.node(),
            Self::Element(element) => element,
        }
    }

    fn validate(self, kind: DomNodeKind<'_>) -> Result<(), DomError> {
        match (self, kind) {
            (Self::Document(_), DomNodeKind::Document(_))
            | (Self::Element(_), DomNodeKind::Element(_)) => Ok(()),
            _ => Err(DomError::WrongNodeKind),
        }
    }
}

/// A live collection of ordinary elements rooted at one exact node.
#[derive(Debug)]
pub struct DomHtmlCollection {
    root: NodeId,
    filter: Filter,
}

#[derive(Debug)]
enum Filter {
    Children,
    FormControls,
    FieldsetControls,
    NamedForm {
        name: DocumentString,
        scope: FormNamedScope,
    },
    FormNamedProperties,
    TagName {
        qualified_name: DocumentString,
        class: DomDocumentClass,
    },
    TagNameNs {
        namespace: Option<DocumentString>,
        local_name: DocumentString,
    },
    ClassName(DocumentString),
    WindowName(DocumentString),
    DocumentName(DocumentString),
    All,
    AllName(DocumentString),
    Document(HtmlDocumentCollection),
}

#[derive(Clone, Copy)]
enum ElementNameScope {
    Collection,
    Window,
    All,
}

/// A live HTMLAllCollection rooted at one exact canonical Document.
#[derive(Debug)]
pub struct DomHtmlAllCollection(DomHtmlCollection);

/// The native result of HTMLAllCollection named lookup.
#[derive(Debug)]
pub enum DomHtmlAllNamedValue {
    /// No Element currently matches.
    Absent,
    /// One Element currently matches.
    Element(NodeId),
    /// Multiple Elements match. The returned HTMLCollection remains live.
    Collection(DomHtmlCollection),
}

/// The current native element candidates for an exact Window property name.
#[derive(Debug)]
pub enum DomWindowNamedElements {
    /// The name has no matching Element in the Document's ordinary tree.
    Absent,
    /// Exactly one Element matches the name.
    Element(NodeId),
    /// At least two Elements match. The collection remains live across edits.
    Collection(DomHtmlCollection),
}

/// The current native value of an exact Window property name.
#[derive(Debug)]
pub enum DomWindowNamedValue {
    /// The current Window has no native value for the name.
    Unsupported,
    /// The first matching child's native browsing context.
    Child(DomHostedBrowsingContextId),
    /// The one matching Element.
    Element(NodeId),
    /// A live collection of the matching Elements.
    Collection(DomHtmlCollection),
}

/// The current native value of an exact Document property name.
#[derive(Debug)]
pub enum DomDocumentNamedValue {
    /// The Document has no native value for the name.
    Absent,
    /// Exactly one Element matches the name and returns as an Element.
    Element(NodeId),
    /// The active Global of the sole matching iframe's actual content navigable.
    Child(DomHostedGlobalId),
    /// Multiple Elements match. The collection remains live across edits.
    Collection(DomHtmlCollection),
}

enum ElementMatches {
    Absent,
    One(NodeId),
    Multiple,
}

/// Finite work and supported-name output for one collection read.
#[derive(Clone, Copy, Debug)]
pub struct DomHtmlCollectionLimits {
    work: u64,
    names: usize,
    name_units: usize,
}

impl DomHtmlCollectionLimits {
    /// Bound the read's cumulative work and one supported-name result.
    pub const fn new(work: u64, names: usize, name_units: usize) -> Self {
        Self {
            work,
            names,
            name_units,
        }
    }
}

/// A qualified read of the collection in the current canonical view.
///
/// Reopen the view after mutation to observe the new tree. Its allowance is
/// cumulative across its operations, and the caller's admission runs at
/// every reached step.
pub struct DomHtmlCollectionRead<'dom, A> {
    view: CollectionRead<'dom, A>,
    collection: &'dom DomHtmlCollection,
    class_quirks: bool,
    limits: DomHtmlCollectionLimits,
}

/// One qualified current view of a native HTMLAllCollection.
pub struct DomHtmlAllCollectionRead<'dom, A>(DomHtmlCollectionRead<'dom, A>);

impl<A> std::fmt::Debug for DomHtmlAllCollectionRead<'_, A> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("DomHtmlAllCollectionRead")
            .field(&self.0)
            .finish()
    }
}

/// Owned ordered names with the backing admitted for this read.
pub struct DomHtmlCollectionNames {
    names: Vec<DocumentString>,
    vector_backing: Option<custody::Debit>,
    text_backing: Option<custody::Debit>,
    retained_units: usize,
}

impl DomHtmlCollectionNames {
    /// Borrow the exact names in their first-occurrence order.
    pub fn as_slice(&self) -> &[DocumentString] {
        &self.names
    }
}

impl std::fmt::Debug for DomHtmlCollectionNames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomHtmlCollectionNames")
            .field("len", &self.names.len())
            .finish_non_exhaustive()
    }
}

impl<A> std::fmt::Debug for DomHtmlCollectionRead<'_, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomHtmlCollectionRead")
            .field("root", &self.collection.root)
            .field("stamp", &self.view.snapshot.stamp())
            .finish_non_exhaustive()
    }
}

impl Dom {
    /// Form a live tag-name collection and admit the query's complete backing.
    pub fn get_elements_by_tag_name(
        &self,
        root: DomDescendantQueryRoot,
        qualified_name: DocumentString,
        max_retained_query_units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlCollection, DomError> {
        let class = self.with_collection_root(root.node(), cancel, |snapshot, node, step| {
            root.validate(node.kind())?;
            Ok(document_metadata(snapshot, node, step)?.class())
        })?;
        admit(
            qualified_name.retained_units(),
            max_retained_query_units,
            DomResource::HtmlCollectionQueryUnits,
        )?;
        Ok(DomHtmlCollection {
            root: root.node(),
            filter: Filter::TagName {
                qualified_name,
                class,
            },
        })
    }

    /// Form a live namespace and local-name collection and admit both strings.
    pub fn get_elements_by_tag_name_ns(
        &self,
        root: DomDescendantQueryRoot,
        namespace: Option<DocumentString>,
        local_name: DocumentString,
        max_retained_query_units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlCollection, DomError> {
        self.with_collection_root(root.node(), cancel, |_, node, _| root.validate(node.kind()))?;
        let namespace = namespace.filter(|namespace| !namespace.is_empty());
        let retained = local_name
            .retained_units()
            .checked_add(namespace.as_ref().map_or(0, DocumentString::retained_units))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            retained,
            max_retained_query_units,
            DomResource::HtmlCollectionQueryUnits,
        )?;
        Ok(DomHtmlCollection {
            root: root.node(),
            filter: Filter::TagNameNs {
                namespace,
                local_name,
            },
        })
    }

    /// Form a live collection of descendants whose classes contain the query.
    pub fn get_elements_by_class_name(
        &self,
        root: DomDescendantQueryRoot,
        classes: DocumentString,
        max_retained_query_units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlCollection, DomError> {
        self.with_collection_root(root.node(), cancel, |_, node, _| root.validate(node.kind()))?;
        admit(
            classes.retained_units(),
            max_retained_query_units,
            DomResource::HtmlCollectionQueryUnits,
        )?;
        Ok(DomHtmlCollection {
            root: root.node(),
            filter: Filter::ClassName(classes),
        })
    }

    /// Form a live collection of a Document, Element or Fragment's direct elements.
    pub fn child_elements(
        &self,
        parent: NodeId,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlCollection, DomError> {
        self.with_collection_root(parent, cancel, |_, node, _| match node.kind() {
            DomNodeKind::Document(_)
            | DomNodeKind::Element(_)
            | DomNodeKind::DocumentFragment
            | DomNodeKind::ShadowRoot(_) => Ok(()),
            _ => Err(DomError::WrongNodeKind),
        })?;
        Ok(DomHtmlCollection {
            root: parent,
            filter: Filter::Children,
        })
    }

    /// Read the closest ordinary Element sibling in the selected direction.
    /// Each native lookup uses the current Source scope and caller admission.
    pub fn read_element_sibling<E: From<DomError>>(
        &self,
        node: NodeId,
        direction: DomSiblingDirection,
        work_limit: u64,
        admission: impl Fn() -> Result<(), E>,
        cancel: &AtomicBool,
    ) -> Result<Option<NodeId>, E> {
        let view = self.open_collection_read(
            node,
            DomResource::HtmlCollectionWork,
            work_limit,
            admission,
            cancel,
        )?;
        if !matches!(
            view.node(node)?.kind(),
            DomNodeKind::Element(_)
                | DomNodeKind::Text(_)
                | DomNodeKind::CdataSection(_)
                | DomNodeKind::Comment(_)
                | DomNodeKind::ProcessingInstruction(_)
        ) {
            return Err(DomError::WrongNodeKind.into());
        }
        let Some((parent, ordinal)) = view.position(node)? else {
            return Ok(None);
        };
        let parent = view.node(parent)?;
        let (before, from_node) = parent
            .children()
            .split_at_checked(ordinal)
            .ok_or(DomError::NotFound)?;
        let first = |sibling: DomNode<'_>| Ok(ControlFlow::Break(sibling.id()));
        match direction {
            DomSiblingDirection::Previous => {
                view.visit_elements(before.iter().rev().copied(), first)
            }
            DomSiblingDirection::Next => {
                view.visit_elements(from_node.iter().skip(1).copied(), first)
            }
        }
    }

    /// Form a live collection of every ordinary Element descendant.
    pub fn html_all_collection(
        &self,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlAllCollection, DomError> {
        self.validate_collection_document(document, cancel)?;
        Ok(DomHtmlAllCollection(DomHtmlCollection {
            root: document.node(),
            filter: Filter::All,
        }))
    }

    /// Form a live collection for a standard Document element filter.
    pub fn document_html_collection(
        &self,
        document: DocumentId,
        filter: HtmlDocumentCollection,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlCollection, DomError> {
        self.validate_collection_document(document, cancel)?;
        Ok(DomHtmlCollection {
            root: document.node(),
            filter: Filter::Document(filter),
        })
    }

    /// Read one exact Document named property in the current ordinary tree.
    /// The collection and any sole iframe's content edge share one admission.
    pub fn document_named_value<E: From<DomHostedError>>(
        &self,
        document: DocumentId,
        name: DocumentString,
        max_retained_query_units: usize,
        work_limit: u64,
        admission: impl Fn() -> Result<(), E>,
        cancel: &AtomicBool,
    ) -> Result<DomDocumentNamedValue, E> {
        let empty = name.is_empty();
        let retained = name.retained_units();
        let collection = DomHtmlCollection {
            root: document.node(),
            filter: Filter::DocumentName(name),
        };
        let read = || -> Result<DomDocumentNamedValue, HostedReadError<E>> {
            let read = self.read_html_collection(
                &collection,
                DomHtmlCollectionLimits::new(work_limit, 0, 0),
                || admission().map_err(HostedReadError::Caller),
                cancel,
            )?;
            admit(
                retained,
                max_retained_query_units,
                DomResource::HtmlCollectionQueryUnits,
            )?;
            if empty {
                return Ok(DomDocumentNamedValue::Absent);
            }
            match read.first_two_matches(|_| Ok(true))? {
                ElementMatches::Absent => Ok(DomDocumentNamedValue::Absent),
                ElementMatches::Multiple => Ok(DomDocumentNamedValue::Collection(collection)),
                ElementMatches::One(node) => {
                    let candidate = read.view.node(node)?;
                    let element = candidate.as_element().ok_or(DomError::WrongNodeKind)?;
                    if element.iframe_initial_state().is_some()
                        && let Some(Some(content)) = element.child_navigable()
                    {
                        let child = self.document_named_child_global(
                            &read.view.snapshot,
                            document,
                            candidate,
                            content,
                            &mut || read.view.meter.step(),
                        )?;
                        return Ok(DomDocumentNamedValue::Child(child));
                    }
                    Ok(DomDocumentNamedValue::Element(node))
                }
            }
        };
        read().map_err(HostedReadError::into_caller)
    }

    /// Check support for an exact Document name without retaining a query or result.
    pub fn document_has_named_property<E: From<DomError>>(
        &self,
        document: DocumentId,
        name: &DocumentString,
        work_limit: u64,
        admission: impl Fn() -> Result<(), E>,
        cancel: &AtomicBool,
    ) -> Result<bool, E> {
        let collection = DomHtmlCollection {
            root: document.node(),
            filter: Filter::All,
        };
        let read = self.read_html_collection(
            &collection,
            DomHtmlCollectionLimits::new(work_limit, 0, 0),
            admission,
            cancel,
        )?;
        if name.is_empty() {
            return Ok(false);
        }
        Ok(read
            .walk(|node| {
                read.document_names(node, |candidate| read.name_match(candidate, name, ()))
            })?
            .is_some())
    }

    /// Read each Document property name once, in ordinary tree order.
    pub fn document_supported_names<E: From<DomError>>(
        &self,
        document: DocumentId,
        limits: DomHtmlCollectionLimits,
        admission: impl Fn() -> Result<(), E>,
        cancel: &AtomicBool,
    ) -> Result<DomHtmlCollectionNames, E> {
        let collection = DomHtmlCollection {
            root: document.node(),
            filter: Filter::All,
        };
        let read = self.read_html_collection(&collection, limits, admission, cancel)?;
        read.collect_names(|node, output| {
            read.document_names(node, |value| {
                read.append_supported_name(output, value)?;
                Ok(ControlFlow::Continue(()))
            })
        })
    }

    /// Resolve the current ordinary-tree Element candidates for a Window name.
    ///
    /// The read admits the query's complete retained UTF-16 backing and stops
    /// at the second matching Element. The returned live collection retains the
    /// exact Document and name to read again after edits.
    ///
    /// # Errors
    ///
    /// Returns an error for an unavailable Document or Source scope,
    /// cancellation, exhausted work or backing, or the caller's refusal.
    pub fn window_named_elements<A, E>(
        &self,
        document: DocumentId,
        name: DocumentString,
        max_retained_query_units: usize,
        work_limit: u64,
        admission: A,
        cancel: &AtomicBool,
    ) -> Result<DomWindowNamedElements, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        let empty = name.is_empty();
        let retained = name.retained_units();
        let collection = DomHtmlCollection {
            root: document.node(),
            filter: Filter::WindowName(name),
        };
        let read = self.read_html_collection(
            &collection,
            DomHtmlCollectionLimits::new(work_limit, 0, 0),
            admission,
            cancel,
        )?;
        admit(
            retained,
            max_retained_query_units,
            DomResource::HtmlCollectionQueryUnits,
        )?;
        if empty {
            return Ok(DomWindowNamedElements::Absent);
        }

        Ok(match read.first_two_matches(|_| Ok(true))? {
            ElementMatches::Absent => DomWindowNamedElements::Absent,
            ElementMatches::One(node) => DomWindowNamedElements::Element(node),
            ElementMatches::Multiple => DomWindowNamedElements::Collection(collection),
        })
    }

    /// Resolve one exact name against the actual current hosted Window.
    ///
    /// The first ordinary child with the exact active target name takes
    /// precedence. Its origin or an independent Element contributor must
    /// support the name. When only Elements match, multiple matches return
    /// a live collection rooted at the Window's associated Document.
    ///
    /// # Errors
    ///
    /// Returns the first invalid hosted identity, Source or caller refusal,
    /// cancellation, or exhausted work or query backing. Both searches share
    /// one work allowance and one active Source scope.
    pub fn window_named_value<E: From<DomHostedError>>(
        &self,
        global: DomHostedGlobalId,
        name: DocumentString,
        max_retained_query_units: usize,
        work_limit: u64,
        admission: impl Fn() -> Result<(), E>,
        cancel: &AtomicBool,
    ) -> Result<DomWindowNamedValue, E> {
        if global.source.owner != self.owner {
            return Err(DomHostedError::ForeignSource.into());
        }
        let read = || -> Result<DomWindowNamedValue, HostedReadError<E>> {
            let source = self.qualify_read_source(cancel)?;
            let meter = Meter::new(
                cancel,
                source,
                DomResource::HtmlCollectionWork,
                work_limit,
                || admission().map_err(HostedReadError::Caller),
            );
            let (document, metadata, _) =
                self.dispatch_global_document_metadata_with(global, &mut || meter.step())?;
            admit(
                name.retained_units(),
                max_retained_query_units,
                DomResource::HtmlCollectionQueryUnits,
            )?;
            let snapshot = self.snapshot();
            let mut first = None;
            self.find_document_tree_child(
                &snapshot,
                document,
                metadata,
                &mut || meter.step(),
                !name.is_empty(),
                |child, child_document, step| {
                    step()?;
                    let entry = DomHostedEntry {
                        record: child.active,
                    };
                    let Some(target) = entry.target_name().filter(|target| !target.is_empty())
                    else {
                        return Ok(false);
                    };
                    if !same_code_units(target.code_units(), name.code_units(), &mut *step)? {
                        return Ok(false);
                    }
                    step()?;
                    first = Some((
                        child.browsing_context(),
                        child_document.origin() == metadata.origin(),
                    ));
                    Ok(true)
                },
            )?;
            if name.is_empty() {
                return Ok(DomWindowNamedValue::Unsupported);
            }
            if let Some((context, true)) = first {
                return Ok(DomWindowNamedValue::Child(context));
            }

            let collection = DomHtmlCollection {
                root: document.node(),
                filter: Filter::WindowName(name),
            };
            let elements = DomHtmlCollectionRead {
                view: CollectionRead {
                    dom: self,
                    snapshot,
                    meter,
                },
                collection: &collection,
                class_quirks: false,
                limits: DomHtmlCollectionLimits::new(work_limit, 0, 0),
            };
            let matches = if first.is_some() {
                elements
                    .item(0)?
                    .map_or(ElementMatches::Absent, ElementMatches::One)
            } else {
                elements.first_two_matches(|_| Ok(true))?
            };
            // The caller can expire the hosted issuer at any admission step.
            elements.view.meter.step()?;
            metadata
                .document_environment()
                .hosted
                .ok_or(DomHostedError::StaleIdentity)?
                .validate_live()?;
            Ok(match (first, matches) {
                (_, ElementMatches::Absent) => DomWindowNamedValue::Unsupported,
                (Some((context, _)), _) => DomWindowNamedValue::Child(context),
                (None, ElementMatches::One(node)) => DomWindowNamedValue::Element(node),
                (None, ElementMatches::Multiple) => DomWindowNamedValue::Collection(collection),
            })
        };
        read().map_err(|error| match error {
            HostedReadError::Hosted(DomHostedError::Dom(DomError::WorkLimit {
                resource: DomResource::HtmlCollectionWork,
                limit,
            })) => DomHostedError::WorkLimit { limit }.into(),
            other => other.into_caller(),
        })
    }

    /// Revalidate the collection's root against the current read authority.
    pub fn read_html_collection<'dom, A, E>(
        &'dom self,
        collection: &'dom DomHtmlCollection,
        limits: DomHtmlCollectionLimits,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<DomHtmlCollectionRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        let view = self.open_collection_read(
            collection.root,
            DomResource::HtmlCollectionWork,
            limits.work,
            admission,
            cancel,
        )?;
        let root = view.node(collection.root)?;
        let class_quirks = matches!(collection.filter, Filter::ClassName(_))
            && document_metadata(&view.snapshot, root, || view.meter.step())?.mode()
                == HtmlDocumentMode::Quirks;
        Ok(DomHtmlCollectionRead {
            view,
            collection,
            class_quirks,
            limits,
        })
    }

    /// Open a fresh HTMLAllCollection read against the current Document.
    pub fn read_html_all_collection<'dom, A, E>(
        &'dom self,
        collection: &'dom DomHtmlAllCollection,
        limits: DomHtmlCollectionLimits,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<DomHtmlAllCollectionRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        self.read_html_collection(&collection.0, limits, admission, cancel)
            .map(DomHtmlAllCollectionRead)
    }
}

impl<A, E> DomHtmlCollectionRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    fn walk<'read, T>(
        &'read self,
        visit: impl FnMut(DomNode<'read>) -> Result<ControlFlow<T>, E>,
    ) -> Result<Option<T>, E> {
        self.walk_where(|_| true, visit)
    }

    fn walk_where<'read, T>(
        &'read self,
        mut can_match: impl FnMut(DomNode<'read>) -> bool,
        mut visit: impl FnMut(DomNode<'read>) -> Result<ControlFlow<T>, E>,
    ) -> Result<Option<T>, E> {
        if matches!(&self.collection.filter, Filter::Children) {
            let root = self.view.node(self.collection.root)?;
            return self
                .view
                .visit_elements(root.children().iter().copied(), |node| {
                    if can_match(node) {
                        visit(node)
                    } else {
                        Ok(ControlFlow::Continue(()))
                    }
                });
        }
        if matches!(
            &self.collection.filter,
            Filter::Document(HtmlDocumentCollection::Applets)
        ) {
            self.view.meter.step()?;
            return Ok(None);
        }
        if let Filter::ClassName(query) = &self.collection.filter
            && scan_space_tokens(
                query.code_units(),
                || self.view.meter.step(),
                |_, _| Ok(ControlFlow::Break(())),
            )?
            .is_none()
        {
            return Ok(None);
        }
        let visit_included = |node: DomNode<'read>| {
            if let Some(element) = node.as_element()
                && can_match(node)
            {
                let matches = match &self.collection.filter {
                    Filter::TagName {
                        qualified_name,
                        class,
                    } => {
                        qualified_name.code_units() == WILDCARD
                            || qualified_equals_with(
                                element.name(),
                                qualified_name,
                                *class == DomDocumentClass::Html
                                    && in_html_namespace_with(element.name(), || {
                                        self.view.meter.step()
                                    })?,
                                || self.view.meter.step(),
                            )?
                    }
                    Filter::TagNameNs {
                        namespace,
                        local_name,
                    } => matches_expanded_name(
                        element.name(),
                        namespace.as_ref(),
                        local_name,
                        || self.view.meter.step(),
                    )?,
                    Filter::ClassName(classes) => self.matches_classes(node, classes)?,
                    Filter::WindowName(name) => {
                        self.element_matches_name(node, name, ElementNameScope::Window)?
                    }
                    Filter::DocumentName(name) => self
                        .document_names(node, |candidate| self.name_match(candidate, name, ()))?
                        .is_break(),
                    Filter::Children | Filter::All | Filter::FormNamedProperties => true,
                    Filter::FormControls => node
                        .is_listed_by_form_with(self.collection.root, || self.view.meter.step())?,
                    Filter::FieldsetControls => {
                        node.is_listed_control_with(|| self.view.meter.step())?
                    }
                    Filter::NamedForm { name, scope } => {
                        let step = || self.view.meter.step();
                        self.element_matches_name(node, name, ElementNameScope::Collection)?
                            && match scope {
                                FormNamedScope::Controls => {
                                    node.is_listed_by_form_with(self.collection.root, step)?
                                }
                                FormNamedScope::LegacyImages => {
                                    node.is_legacy_image_of_form_with(self.collection.root, step)?
                                }
                            }
                    }
                    Filter::AllName(name) => {
                        self.element_matches_name(node, name, ElementNameScope::All)?
                    }
                    Filter::Document(filter) => self.matches_document_filter(node, *filter)?,
                };
                if matches {
                    return visit(node);
                }
            }
            Ok(ControlFlow::Continue(()))
        };
        if matches!(
            &self.collection.filter,
            Filter::FormControls | Filter::NamedForm { .. } | Filter::FormNamedProperties
        ) {
            let root = self
                .view
                .node(self.collection.root)?
                .root_with(false, || self.view.meter.step())?;
            self.view.inclusive_tree(root, visit_included)
        } else {
            self.view.descendants(self.collection.root, visit_included)
        }
    }

    fn matches_classes(&self, node: DomNode<'_>, query: &DocumentString) -> Result<bool, E> {
        let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
        let step = || self.view.meter.step();
        let Some(classes) = attributes.get_null_namespace_attribute_with(CLASS, step)? else {
            return Ok(false);
        };
        let missing = scan_space_tokens(query.code_units(), step, |requested, _| {
            let found = scan_space_tokens(classes.value().code_units(), step, |candidate, _| {
                let matches = if self.class_quirks {
                    if candidate.len() != requested.len() {
                        return Ok(ControlFlow::Continue(()));
                    }
                    for (&candidate, &requested) in candidate.iter().zip(requested) {
                        step()?;
                        if ascii_lower(candidate) != ascii_lower(requested) {
                            return Ok(ControlFlow::Continue(()));
                        }
                    }
                    true
                } else {
                    same_code_units(candidate, requested, step)?
                };
                Ok(if matches {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                })
            })?;
            Ok(if found.is_some() {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            })
        })?;
        Ok(missing.is_none())
    }

    fn first_two_matches(
        &self,
        mut matches: impl FnMut(DomNode<'_>) -> Result<bool, E>,
    ) -> Result<ElementMatches, E> {
        let mut first = None;
        let second = self.walk(|node| {
            if !matches(node)? {
                return Ok(ControlFlow::Continue(()));
            }
            if first.is_some() {
                return Ok(ControlFlow::Break(()));
            }
            first = Some(node.id());
            Ok(ControlFlow::Continue(()))
        })?;
        if second.is_some() {
            Ok(ElementMatches::Multiple)
        } else {
            Ok(first.map_or(ElementMatches::Absent, ElementMatches::One))
        }
    }

    fn matches_document_filter(
        &self,
        node: DomNode<'_>,
        filter: HtmlDocumentCollection,
    ) -> Result<bool, E> {
        let element = node.as_element().ok_or(DomError::WrongNodeKind)?;
        if !in_html_namespace_with(element.name(), || self.view.meter.step())? {
            return Ok(false);
        }
        self.view.meter.step()?;
        match filter.match_local_name(element.name().local_name().code_units()) {
            HtmlDocumentCollectionMatch::Excluded => Ok(false),
            HtmlDocumentCollectionMatch::Included => Ok(true),
            HtmlDocumentCollectionMatch::AttributePresent(name) => {
                let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
                Ok(attributes
                    .get_null_namespace_attribute_with(name, || self.view.meter.step())?
                    .is_some())
            }
        }
    }

    /// Count every represented element in the current ordinary tree.
    pub fn length(&self) -> Result<usize, E> {
        let mut length = 0usize;
        self.walk::<()>(|_| {
            length = length
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(ControlFlow::Continue(()))
        })?;
        Ok(length)
    }

    /// Read the current element at the given zero-based index.
    pub fn item(&self, mut index: usize) -> Result<Option<NodeId>, E> {
        self.walk(|node| {
            if index == 0 {
                return Ok(ControlFlow::Break(node.id()));
            }
            index -= 1;
            Ok(ControlFlow::Continue(()))
        })
    }

    /// Read the last current element in collection order.
    pub fn last_item(&self) -> Result<Option<NodeId>, E> {
        if matches!(&self.collection.filter, Filter::Children) {
            let root = self.view.node(self.collection.root)?;
            return self
                .view
                .visit_elements(root.children().iter().rev().copied(), |node| {
                    Ok(ControlFlow::Break(node.id()))
                });
        }
        let mut last = None;
        self.walk::<()>(|node| {
            last = Some(node.id());
            Ok(ControlFlow::Continue(()))
        })?;
        Ok(last)
    }

    /// Read the first current element with the exact ID or eligible name.
    pub fn named_item(&self, name: &DocumentString) -> Result<Option<NodeId>, E> {
        self.first_named_match(name, ElementNameScope::Collection)
    }

    fn first_named_match(
        &self,
        name: &DocumentString,
        scope: ElementNameScope,
    ) -> Result<Option<NodeId>, E> {
        self.view.meter.step()?;
        if name.is_empty() {
            return Ok(None);
        }
        self.walk(|node| {
            self.names(node, scope, |candidate| {
                self.name_match(candidate, name, node.id())
            })
        })
    }

    fn name_match<T>(
        &self,
        candidate: &DocumentString,
        query: &DocumentString,
        matched: T,
    ) -> Result<ControlFlow<T>, E> {
        if same_code_units(candidate.code_units(), query.code_units(), || {
            self.view.meter.step()
        })? {
            Ok(ControlFlow::Break(matched))
        } else {
            Ok(ControlFlow::Continue(()))
        }
    }

    fn element_matches_name(
        &self,
        node: DomNode<'_>,
        name: &DocumentString,
        scope: ElementNameScope,
    ) -> Result<bool, E> {
        Ok(matches!(
            self.names(node, scope, |value| self.name_match(value, name, ()))?,
            ControlFlow::Break(())
        ))
    }

    fn document_names<T>(
        &self,
        node: DomNode<'_>,
        mut visit: impl FnMut(&DocumentString) -> Result<ControlFlow<T>, E>,
    ) -> Result<ControlFlow<T>, E> {
        let step = || self.view.meter.step();
        let element = node.as_element().ok_or(DomError::WrongNodeKind)?;
        let Some(id_policy) = document_named_id_policy(element.name(), step)? else {
            return Ok(ControlFlow::Continue(()));
        };
        let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
        let read_name = || -> Result<_, E> {
            Ok(attributes
                .get_null_namespace_attribute_with(NAME, step)?
                .filter(|name| !name.value().is_empty()))
        };
        // An image's name determines whether its ID contributes. Emit the ID first.
        let image_name = (id_policy == HtmlDocumentNamedId::WithName)
            .then(read_name)
            .transpose()?;
        if (id_policy == HtmlDocumentNamedId::Always || matches!(image_name, Some(Some(_))))
            && let Some(id) = attributes.get_null_namespace_attribute_with(ID, step)?
            && !id.value().is_empty()
            && let ControlFlow::Break(value) = visit(id.value())?
        {
            return Ok(ControlFlow::Break(value));
        }
        let name = match image_name {
            Some(name) => name,
            None => read_name()?,
        };
        match name {
            Some(name) => visit(name.value()),
            None => Ok(ControlFlow::Continue(())),
        }
    }

    // Borrow the exact ID and eligible name contributions from one Element.
    fn names<T>(
        &self,
        node: DomNode<'_>,
        scope: ElementNameScope,
        mut visit: impl FnMut(&DocumentString) -> Result<ControlFlow<T>, E>,
    ) -> Result<ControlFlow<T>, E> {
        let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
        if let Some(id) =
            attributes.get_null_namespace_attribute_with(ID, || self.view.meter.step())?
            && !id.value().is_empty()
            && let ControlFlow::Break(value) = visit(id.value())?
        {
            return Ok(ControlFlow::Break(value));
        }
        let element = node.as_element().ok_or(DomError::WrongNodeKind)?;
        let eligible = match scope {
            ElementNameScope::Collection => {
                in_html_namespace_with(element.name(), || self.view.meter.step())?
            }
            ElementNameScope::Window => {
                window_name_element(element.name(), || self.view.meter.step())?
            }
            ElementNameScope::All => {
                if in_html_namespace_with(element.name(), || self.view.meter.step())? {
                    self.view.meter.step()?;
                    is_document_all_named_local_name(element.name().local_name().code_units())
                } else {
                    false
                }
            }
        };
        if !eligible {
            return Ok(ControlFlow::Continue(()));
        }
        if let Some(name) =
            attributes.get_null_namespace_attribute_with(NAME, || self.view.meter.step())?
            && !name.value().is_empty()
        {
            return visit(name.value());
        }
        Ok(ControlFlow::Continue(()))
    }

    /// Return each nonempty supported name once, in first occurrence order.
    pub fn supported_names(&self) -> Result<DomHtmlCollectionNames, E> {
        self.names_for(ElementNameScope::Collection)
    }

    fn names_for(&self, scope: ElementNameScope) -> Result<DomHtmlCollectionNames, E> {
        self.collect_names(|node, output| {
            self.names(node, scope, |value| {
                self.append_supported_name(output, value)?;
                Ok(ControlFlow::Continue(()))
            })
        })
    }

    fn collect_names(
        &self,
        mut collect: impl FnMut(DomNode<'_>, &mut DomHtmlCollectionNames) -> Result<ControlFlow<()>, E>,
    ) -> Result<DomHtmlCollectionNames, E> {
        let mut output = DomHtmlCollectionNames {
            names: Vec::new(),
            vector_backing: None,
            text_backing: None,
            retained_units: 0,
        };
        self.walk(|node| collect(node, &mut output))?;
        Ok(output)
    }

    fn append_supported_name(
        &self,
        output: &mut DomHtmlCollectionNames,
        value: &DocumentString,
    ) -> Result<(), E> {
        for present in &output.names {
            self.view.meter.step()?;
            if same_code_units(present.code_units(), value.code_units(), || {
                self.view.meter.step()
            })? {
                return Ok(());
            }
        }
        let count = output
            .names
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let next_units = output
            .retained_units
            .checked_add(value.retained_units())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(count, self.limits.names, DomResource::HtmlCollectionNames)?;
        admit(
            next_units,
            self.limits.name_units,
            DomResource::HtmlCollectionNameUnits,
        )?;
        if output.names.capacity() < count {
            self.grow_names(output, count)?;
        }
        self.view.meter.step()?;
        if self.view.meter.source.is_some() {
            let mut payload = source::storage::Payload::default();
            payload.string(value)?;
            let usage = custody::Usage::new(payload.bytes, payload.units);
            let admit = |usage| {
                self.view
                    .dom
                    .admit_collection_output_usage(usage, self.view.meter.cancellation)
            };
            if let Some(backing) = &mut output.text_backing {
                backing.grow(usage, admit)?;
            } else {
                output.text_backing = Some(
                    self.view
                        .dom
                        .custody_registry
                        .ledger()
                        .retain(usage, admit)?,
                );
            }
        }
        output.names.push(value.clone());
        output.retained_units = next_units;
        Ok(())
    }

    fn grow_names(&self, output: &mut DomHtmlCollectionNames, count: usize) -> Result<(), E> {
        self.view.meter.step()?;
        let capacity = count.max(
            output
                .names
                .capacity()
                .saturating_mul(2)
                .min(self.limits.names),
        );
        let usage = |slots: usize| {
            slots
                .checked_mul(size_of::<DocumentString>())
                .map(|bytes| custody::Usage::new(bytes, 0))
                .ok_or(DomError::RepresentationExhausted)
        };
        let admit = |usage| {
            self.view
                .dom
                .admit_collection_output_usage(usage, self.view.meter.cancellation)
        };
        let mut backing = self
            .view
            .meter
            .source
            .as_ref()
            .map(|_| {
                self.view
                    .dom
                    .custody_registry
                    .ledger()
                    .retain(usage(capacity)?, admit)
            })
            .transpose()?;
        let mut names = Vec::new();
        names
            .try_reserve_exact(capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        super::admit(
            names.capacity(),
            self.limits.names,
            DomResource::HtmlCollectionNames,
        )?;
        if let Some(backing) = &mut backing {
            transaction::nodes::resize_debit(backing, usage(names.capacity())?, admit)?;
        }
        // Source holds both old and replacement vector storage until the move.
        // The text occurrence receipts stay with the whole output.
        for value in output.names.drain(..) {
            self.view.meter.step()?;
            names.push(value);
        }
        output.names = names;
        output.vector_backing = backing;
        Ok(())
    }
}

impl<A, E> DomHtmlAllCollectionRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    /// Count represented elements in the current ordinary tree.
    pub fn length(&self) -> Result<usize, E> {
        self.0.length()
    }

    /// Read the current element at the given zero-based index.
    pub fn item(&self, index: usize) -> Result<Option<NodeId>, E> {
        self.0.item(index)
    }

    /// Check current support without retaining a named subcollection.
    pub fn has_named_item(&self, name: &DocumentString) -> Result<bool, E> {
        self.0
            .first_named_match(name, ElementNameScope::All)
            .map(|item| item.is_some())
    }

    /// Read the native named result. A returned subcollection retains the
    /// exact query backing and uses the same Document for later reads.
    pub fn named_item(
        &self,
        name: &DocumentString,
        max_retained_query_units: usize,
    ) -> Result<DomHtmlAllNamedValue, E> {
        self.0.view.meter.step()?;
        if name.is_empty() {
            return Ok(DomHtmlAllNamedValue::Absent);
        }
        match self.0.first_two_matches(|node| {
            self.0
                .element_matches_name(node, name, ElementNameScope::All)
        })? {
            ElementMatches::Absent => Ok(DomHtmlAllNamedValue::Absent),
            ElementMatches::One(node) => Ok(DomHtmlAllNamedValue::Element(node)),
            ElementMatches::Multiple => {
                admit(
                    name.retained_units(),
                    max_retained_query_units,
                    DomResource::HtmlCollectionQueryUnits,
                )?;
                Ok(DomHtmlAllNamedValue::Collection(DomHtmlCollection {
                    root: self.0.collection.root,
                    filter: Filter::AllName(name.clone()),
                }))
            }
        }
    }

    /// Return each eligible nonempty ID or name in tree order.
    pub fn supported_names(&self) -> Result<DomHtmlCollectionNames, E> {
        self.0.names_for(ElementNameScope::All)
    }
}

#[cfg(test)]
mod tests {
    use super::{HtmlDocumentNamedId, document_named_id_policy, window_name_element};
    use surgeist_html::values::{Namespace, QualifiedName};

    #[test]
    fn window_name_eligibility_uses_the_four_exact_html_expanded_names() {
        for (namespace, local, expected) in [
            (Namespace::html(), "embed", true),
            (Namespace::html(), "form", true),
            (Namespace::html(), "img", true),
            (Namespace::html(), "object", true),
            (Namespace::html(), "iframe", false),
            (Namespace::html(), "IMG", false),
            (Namespace::svg(), "embed", false),
        ] {
            let name = QualifiedName::new(namespace, Some("p".into()), local);
            assert_eq!(
                window_name_element(&name, || Ok::<_, ()>(())),
                Ok(expected),
                "{name:?}"
            );
        }
    }

    #[test]
    fn document_named_eligibility_applies_the_exact_html_expanded_name_and_id_rule() {
        use HtmlDocumentNamedId::{Always, Never, WithName};
        for (namespace, local, expected) in [
            (Namespace::html(), "embed", Some(Never)),
            (Namespace::html(), "form", Some(Never)),
            (Namespace::html(), "iframe", Some(Never)),
            (Namespace::html(), "img", Some(WithName)),
            (Namespace::html(), "object", Some(Always)),
            (Namespace::html(), "frame", None),
            (Namespace::html(), "OBJECT", None),
            (Namespace::none(), "object", None),
            (Namespace::svg(), "iframe", None),
        ] {
            let name = QualifiedName::new(namespace, Some("p".into()), local);
            assert_eq!(
                document_named_id_policy(&name, || Ok::<_, ()>(())),
                Ok(expected),
                "{name:?}"
            );
        }
    }
}
