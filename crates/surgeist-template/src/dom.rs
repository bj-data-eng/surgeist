//! Static HTML composition into a caller's DOM batch.
//!
//! [`append_static_html`] queues construction requests for the concrete part of
//! a validated template (native elements, their static attributes and text)
//! into the caller's ordinary [`DomBatch`]. The whole
//! template is preflighted first, so a [`TemplateDomError`] means no command
//! was queued. Symbolic components, interpolations, conditionals, iteration and
//! attribute expressions remain unresolved and are reported through
//! [`TemplateDomUnresolvedKind`]. The caller applies the batch; DOM owns
//! document and parent validity, native-family admission, limits and atomic
//! publication. This module evaluates nothing.

use surgeist_dom::{
    DomBatch,
    edit::{CreatedNode, NodeOperand},
};
use surgeist_html::{
    values::{DocumentString, Namespace},
    vocabulary::KnownHtmlElement,
};

use crate::span::SourceSpan;
use crate::validation::{ValidatedAttrValue, ValidatedNode, ValidatedTemplate};

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

impl core::fmt::Display for TemplateDomUnresolvedKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Component => "component element",
            Self::Interpolation => "interpolation",
            Self::Conditional => "conditional block",
            Self::Iteration => "foreach block",
            Self::AttributeExpression => "attribute expression",
            Self::AttributeInterpolation => "interpolated attribute",
        })
    }
}

impl core::fmt::Display for TemplateDomError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unresolved { span, kind } => {
                write!(f, "{kind} at ")?;
                span.write_start(f)?;
                f.write_str(" is symbolic and cannot become a DOM node")
            }
            Self::UnsupportedTemplateContentsTarget { span } => {
                f.write_str("template element at ")?;
                span.write_start(f)?;
                f.write_str(" needs a template-contents association that is not supported")
            }
            Self::Allocation => f.write_str("allocation failed while preparing the DOM batch"),
            Self::RepresentationExhausted => {
                f.write_str("the DOM command count exceeds the representable range")
            }
        }
    }
}

impl core::error::Error for TemplateDomError {}

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
/// use surgeist_dom::{Dom, DomBatch, limits::DomLimits};
/// use surgeist_html::values::Namespace;
/// use surgeist_template::dom::append_static_html;
/// use surgeist_template::registry::{AttributeKind, AttributeRule, ComponentRegistry,
///     NativeElementRegistry};
/// use surgeist_template::{parse_template, validate_template};
/// let native = NativeElementRegistry::from_html_metadata(
///     AttributeRule::one(AttributeKind::Static)).unwrap();
/// let authored = parse_template("<div title=example>hello</div>").unwrap();
/// let template = validate_template(&authored, &native, &ComponentRegistry::default()).unwrap();
/// let names = surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
/// let mut dom = Dom::new(DomLimits::new(16, 4096, 32, 0, 0, 4, 8192, names).unwrap()).unwrap();
/// let mut batch = DomBatch::new();
/// let document = batch.create_document();
/// let parent = batch.create_fragment(&document);
/// let roots = append_static_html(&template, &mut batch,
///     (&document).into(), (&parent).into()).unwrap();
/// let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
/// let snapshot = dom.snapshot();
/// let root = snapshot.node(commit.created(&roots[0]).unwrap().unwrap()).unwrap();
/// assert_eq!(root.as_element().unwrap().name().namespace_uri(), Namespace::html().uri());
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
