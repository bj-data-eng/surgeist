use surgeist_dom::{CreatedNode, DomBatch, NodeOperand};
use surgeist_html::{DocumentString, KnownHtmlElement, Namespace};

use crate::{SourceSpan, ValidatedAttrValue, ValidatedNode, ValidatedTemplate};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateDomUnresolvedKind {
    Component,
    Interpolation,
    Conditional,
    Iteration,
    AttributeExpression,
    AttributeInterpolation,
}

/// A preflight rejection. No commands have been appended when this is returned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateDomError {
    Unresolved {
        span: SourceSpan,
        kind: TemplateDomUnresolvedKind,
    },
    /// HTML template children require an actual template-contents association.
    UnsupportedTemplateContentsTarget {
        span: SourceSpan,
    },
    Allocation,
    RepresentationExhausted,
}

enum PlannedKind {
    Element {
        name: DocumentString,
        attributes: Vec<(DocumentString, DocumentString)>,
    },
    Text(DocumentString),
}

struct PlannedNode {
    // Index into this private plan, never a canonical node identity.
    parent: Option<usize>,
    kind: PlannedKind,
}

/// Appends concrete HTML construction requests to the caller's ordinary batch.
///
/// Both operands accept published nodes or creations in this same batch. The
/// entire template is preflighted before commands are queued; symbolic values
/// remain unresolved, and HTML template contents require an association that this
/// adapter does not yet support.
/// Returned root tickets resolve only after the caller successfully applies the
/// batch. DOM owns document/parent validity, native-family admission, limits and
/// atomic publication; this function neither applies nor evaluates anything.
///
/// Temporary storage and string preparation errors are returned before queuing.
/// Like DomBatch's queue operations, infallible shared-backing and command-vector
/// allocations retain Rust's ordinary allocation-failure behavior.
///
/// ```
/// use std::sync::atomic::AtomicBool;
/// use surgeist_dom::{Dom, DomBatch, DomLimits, Namespace};
/// use surgeist_template::{AttributeKind, AttributeRule, ComponentRegistry,
///     NativeElementRegistry, append_static_html, parse_template, validate_template};
/// let native = NativeElementRegistry::from_html_metadata(
///     AttributeRule::one(AttributeKind::Static)).unwrap();
/// let authored = parse_template("<div title=example>hello</div>").unwrap();
/// let template = validate_template(&authored, &native, &ComponentRegistry::default()).unwrap();
/// let mut dom = Dom::new(DomLimits::new(16, 4096, 32, 0, 0, 4, 8192).unwrap()).unwrap();
/// let mut batch = DomBatch::new();
/// let document = batch.create_document();
/// let parent = batch.create_fragment(&document);
/// let roots = append_static_html(&template, &mut batch,
///     (&document).into(), (&parent).into()).unwrap();
/// let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
/// let snapshot = dom.snapshot();
/// let root = snapshot.node(commit.created(&roots[0]).unwrap().unwrap()).unwrap();
/// assert_eq!(root.as_element().unwrap().name().namespace(), &Namespace::html());
/// assert_eq!(root.children().len(), 1);
/// ```
pub fn append_static_html(
    template: &ValidatedTemplate,
    batch: &mut DomBatch,
    document: NodeOperand,
    parent: NodeOperand,
) -> Result<Vec<CreatedNode>, TemplateDomError> {
    let mut plan = Vec::new();
    let mut pending = Vec::new();
    pending
        .try_reserve(1)
        .map_err(|_| TemplateDomError::Allocation)?;
    pending.push((template.nodes().iter(), None));
    let mut commands = batch.len();
    while let Some((nodes, owner)) = pending.last_mut() {
        let Some(node) = nodes.next() else {
            pending.pop();
            continue;
        };
        let owner = *owner;
        let (kind, children) = match node {
            ValidatedNode::NativeElement(element) => {
                if element.name().as_str() == KnownHtmlElement::Template.local_name() {
                    return Err(TemplateDomError::UnsupportedTemplateContentsTarget {
                        span: element.span(),
                    });
                }
                let mut attributes = Vec::new();
                attributes
                    .try_reserve_exact(element.attributes().len())
                    .map_err(|_| TemplateDomError::Allocation)?;
                for attribute in element.attributes() {
                    let value = match attribute.value() {
                        ValidatedAttrValue::Bool => "",
                        ValidatedAttrValue::Static(value) => value,
                        ValidatedAttrValue::Expression(_) => {
                            return Err(unresolved(
                                attribute.span(),
                                TemplateDomUnresolvedKind::AttributeExpression,
                            ));
                        }
                        ValidatedAttrValue::Interpolated(_) => {
                            return Err(unresolved(
                                attribute.span(),
                                TemplateDomUnresolvedKind::AttributeInterpolation,
                            ));
                        }
                    };
                    attributes.push((
                        document_string(attribute.name().as_str())?,
                        document_string(value)?,
                    ));
                }
                commands = commands
                    .checked_add(attributes.len())
                    .ok_or(TemplateDomError::RepresentationExhausted)?;
                (
                    PlannedKind::Element {
                        name: document_string(element.name().as_str())?,
                        attributes,
                    },
                    element.children(),
                )
            }
            ValidatedNode::Text(text) => {
                (PlannedKind::Text(document_string(text.value())?), &[][..])
            }
            ValidatedNode::ComponentElement(element) => {
                return Err(unresolved(
                    element.span(),
                    TemplateDomUnresolvedKind::Component,
                ));
            }
            ValidatedNode::Interpolation(value) => {
                return Err(unresolved(
                    value.span(),
                    TemplateDomUnresolvedKind::Interpolation,
                ));
            }
            ValidatedNode::If(value) => {
                return Err(unresolved(
                    value.span(),
                    TemplateDomUnresolvedKind::Conditional,
                ));
            }
            ValidatedNode::ForEach(value) => {
                return Err(unresolved(
                    value.span(),
                    TemplateDomUnresolvedKind::Iteration,
                ));
            }
        };
        commands = commands
            .checked_add(2)
            .ok_or(TemplateDomError::RepresentationExhausted)?;
        let index = plan.len();
        plan.try_reserve(1)
            .map_err(|_| TemplateDomError::Allocation)?;
        plan.push(PlannedNode {
            parent: owner,
            kind,
        });
        if !children.is_empty() {
            pending
                .try_reserve(1)
                .map_err(|_| TemplateDomError::Allocation)?;
            pending.push((children.iter(), Some(index)));
        }
    }

    let mut created: Vec<CreatedNode> = Vec::new();
    created
        .try_reserve_exact(plan.len())
        .map_err(|_| TemplateDomError::Allocation)?;
    let mut roots = Vec::new();
    roots
        .try_reserve_exact(template.nodes().len())
        .map_err(|_| TemplateDomError::Allocation)?;
    let namespace = Namespace::html();
    for node in plan {
        let target = node
            .parent
            .map_or_else(|| parent.clone(), |index| (&created[index]).into());
        let ticket = match node.kind {
            PlannedKind::Element { name, attributes } => {
                let ticket = batch.create_element_ns(document.clone(), namespace.clone(), name);
                for (name, value) in attributes {
                    batch.set_attribute(&ticket, name, value);
                }
                ticket
            }
            PlannedKind::Text(text) => batch.create_text(document.clone(), text),
        };
        batch.insert_before(&ticket, target, None);
        if node.parent.is_none() {
            roots.push(ticket.clone());
        }
        created.push(ticket);
    }
    Ok(roots)
}

fn unresolved(span: SourceSpan, kind: TemplateDomUnresolvedKind) -> TemplateDomError {
    TemplateDomError::Unresolved { span, kind }
}

fn document_string(value: &str) -> Result<DocumentString, TemplateDomError> {
    let mut units = Vec::new();
    units
        .try_reserve_exact(value.encode_utf16().count())
        .map_err(|_| TemplateDomError::Allocation)?;
    units.extend(value.encode_utf16());
    Ok(DocumentString::from_utf16(units))
}
