//! Document tree selection and forwarding through the original Source owner.

use super::*;
use surgeist_bindings_js::{
    ValueKind,
    conversion::{
        declarations::{
            document_obsolete_alink_color_setter, document_obsolete_bg_color_setter,
            document_obsolete_fg_color_setter, document_obsolete_link_color_setter,
            document_obsolete_vlink_color_setter,
        },
        types::{DOM_STRING, USV_STRING},
    },
    declarations::DocumentReadyState,
    dom_exception::DomExceptionName,
};
use surgeist_dom::document::DomDocumentReadiness;
use surgeist_dom::source::DomSourceEdit;
use surgeist_html::values::Namespace;
use surgeist_html::{
    bounded::attributes::dir_keyword_with,
    elements::style::{HtmlDocumentBodyRole, document_body_role_with},
    vocabulary::KnownHtmlElement,
};

#[derive(Clone, Copy)]
enum Member {
    Dir,
    Body,
    Head,
    Cookie,
    FgColor,
    LinkColor,
    VlinkColor,
    AlinkColor,
    BgColor,
}

#[derive(Clone, Copy)]
enum Target {
    Html,
    Head,
    Body,
    BodyElement,
}

struct Selection {
    document_element: Option<NodeId>,
    target: Option<NodeId>,
}

impl Member {
    fn text_target(self) -> Option<(Target, &'static [u16])> {
        Some(match self {
            Self::Dir => (Target::Html, const { &ascii(b"dir") }),
            Self::FgColor => (Target::BodyElement, const { &ascii(b"text") }),
            Self::LinkColor => (Target::BodyElement, const { &ascii(b"link") }),
            Self::VlinkColor => (Target::BodyElement, const { &ascii(b"vlink") }),
            Self::AlinkColor => (Target::BodyElement, const { &ascii(b"alink") }),
            Self::BgColor => (Target::BodyElement, const { &ascii(b"bgcolor") }),
            Self::Head | Self::Body | Self::Cookie => return None,
        })
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (member, name, get_name, set_name) in [
        (
            Member::Cookie,
            &ascii(b"cookie")[..],
            &ascii(b"get cookie")[..],
            Some(&ascii(b"set cookie")[..]),
        ),
        (
            Member::Dir,
            &ascii(b"dir")[..],
            &ascii(b"get dir")[..],
            Some(&ascii(b"set dir")[..]),
        ),
        (
            Member::Body,
            &ascii(b"body")[..],
            &ascii(b"get body")[..],
            Some(&ascii(b"set body")[..]),
        ),
        (
            Member::Head,
            &ascii(b"head")[..],
            &ascii(b"get head")[..],
            None,
        ),
        (
            Member::FgColor,
            &ascii(b"fgColor")[..],
            &ascii(b"get fgColor")[..],
            Some(&ascii(b"set fgColor")[..]),
        ),
        (
            Member::LinkColor,
            &ascii(b"linkColor")[..],
            &ascii(b"get linkColor")[..],
            Some(&ascii(b"set linkColor")[..]),
        ),
        (
            Member::VlinkColor,
            &ascii(b"vlinkColor")[..],
            &ascii(b"get vlinkColor")[..],
            Some(&ascii(b"set vlinkColor")[..]),
        ),
        (
            Member::AlinkColor,
            &ascii(b"alinkColor")[..],
            &ascii(b"get alinkColor")[..],
            Some(&ascii(b"set alinkColor")[..]),
        ),
        (
            Member::BgColor,
            &ascii(b"bgColor")[..],
            &ascii(b"get bgColor")[..],
            Some(&ascii(b"set bgColor")[..]),
        ),
    ] {
        let handler = capture(size_of::<(Capture, Member)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.document_tree_get(scope, arguments, member)
            }),
            FunctionMetadata {
                name: get_name,
                length: 0,
            },
        )?;
        let setter = if let Some(set_name) = set_name {
            let handler = capture(size_of::<(Capture, Member)>())?;
            Some(scope.fixed_arity_function(
                &Callback::new(move |scope, arguments| {
                    handler.document_tree_set(scope, arguments, member)
                }),
                FunctionMetadata {
                    name: set_name,
                    length: 1,
                },
            )?)
        } else {
            None
        };
        scope.define_new_property(
            prototype,
            PropertyKey::String(name),
            PropertyDescriptor::Accessor {
                get: Some(&getter),
                set: setter.as_ref(),
                enumerable: true,
                configurable: true,
            },
        )?;
    }
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"referrer"),
        &ascii(b"get referrer"),
        true,
        Callback::new(move |scope, arguments| handler.document_referrer(scope, arguments)),
    )?;
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"readyState"),
        &ascii(b"get readyState"),
        true,
        Callback::new(move |scope, arguments| handler.document_readiness(scope, arguments)),
    )?;
    Ok(())
}

fn select_target(
    read: &NativeRead<'_>,
    document: DocumentId,
    target: Target,
) -> Result<Selection, DocumentGuestEventError> {
    let root = read
        .snapshot
        .document_element_with(document, || read.step())?;
    let mut selected = Selection {
        document_element: root,
        target: None,
    };
    let Some(root) = root else {
        return Ok(selected);
    };
    let root_node = read.node(root)?;
    let element = root_node.as_element().ok_or(DomError::WrongNodeKind)?;
    if document_body_role_with(element.name(), |_| read.step())? != HtmlDocumentBodyRole::Html {
        return Ok(selected);
    }
    if matches!(target, Target::Html) {
        selected.target = Some(root);
        return Ok(selected);
    }
    for &child in root_node.children() {
        let node = read.node(child)?;
        let Some(element) = node.as_element() else {
            continue;
        };
        if matches!(target, Target::Head) {
            read.step()?;
            if KnownHtmlElement::from_name(element.name()) == Some(KnownHtmlElement::Head) {
                selected.target = Some(child);
                return Ok(selected);
            }
        } else if document_body_role_with(element.name(), |_| read.step())?
            == HtmlDocumentBodyRole::Body
        {
            if matches!(target, Target::BodyElement) {
                read.step()?;
                if KnownHtmlElement::from_name(element.name()) != Some(KnownHtmlElement::Body) {
                    return Ok(selected);
                }
            }
            selected.target = Some(child);
            return Ok(selected);
        }
    }
    Ok(selected)
}

fn require_cookie_averse(
    read: &NativeRead<'_>,
    document: DocumentId,
) -> Result<(), DocumentGuestEventError> {
    let DomNodeKind::Document(metadata) = read.node(document.node())?.kind() else {
        return Err(DomError::WrongNodeKind.into());
    };
    metadata.require_cookie_averse().map_err(Into::into)
}

enum TextValue {
    Raw(Option<DocumentString>),
    Direction(&'static str),
}

enum BodyInput {
    Null,
    Element(NodeId),
    TypeMismatch,
}

enum BodyPlan {
    Unchanged,
    Hierarchy,
    Append {
        node: NodeId,
        parent: NodeId,
    },
    Replace {
        node: NodeId,
        parent: NodeId,
        current: NodeId,
    },
}

impl Capture {
    fn document_readiness<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let DomNodeKind::Document(metadata) = read.node(document.node())?.kind() else {
                    return Err(DomError::WrongNodeKind.into());
                };
                Ok(match metadata.readiness() {
                    DomDocumentReadiness::Loading => DocumentReadyState::Loading,
                    DomDocumentReadiness::Interactive => DocumentReadyState::Interactive,
                    DomDocumentReadiness::Complete => DocumentReadyState::Complete,
                })
            })?;
            Ok(Completion::Return(
                self.copy_dom_units(scope, value.as_code_units())?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn document_referrer<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let DomNodeKind::Document(metadata) = read.node(document.node())?.kind() else {
                    return Err(DomError::WrongNodeKind.into());
                };
                Ok(metadata.referrer()?.cloned())
            })?;
            let units = value.as_ref().map_or(&[][..], DocumentString::code_units);
            Ok(Completion::Return(self.copy_dom_units(scope, units)?))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn document_tree_get<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let output = match member {
                Member::Cookie => {
                    self.callbacks.with_read(&owner, &events, |read, _| {
                        require_cookie_averse(read, document)
                    })?;
                    self.copy_dom_units(scope, &[])?
                }
                Member::Head | Member::Body => {
                    let target = if matches!(member, Member::Head) {
                        Target::Head
                    } else {
                        Target::Body
                    };
                    let selected = self.callbacks.with_read(&owner, &events, |read, _| {
                        Ok(select_target(read, document, target)?.target)
                    })?;
                    match selected {
                        Some(node) => events
                            .public
                            .as_ref()
                            .ok_or(Error::ExpiredIdentity)?
                            .project_node(scope, &owner, &events, node)?,
                        None => scope.primitive(Primitive::Null)?,
                    }
                }
                _ => {
                    let (target, attribute) =
                        member.text_target().ok_or(Error::BackendUnavailable)?;
                    let value = self.callbacks.with_read(&owner, &events, |read, _| {
                        let Some(selected) = select_target(read, document, target)?.target else {
                            return Ok(if matches!(member, Member::Dir) {
                                TextValue::Direction("")
                            } else {
                                TextValue::Raw(None)
                            });
                        };
                        let raw = read
                            .node(selected)?
                            .attributes()
                            .ok_or(DomError::WrongNodeKind)?
                            .get_null_namespace_attribute_with(attribute, || read.step())?;
                        Ok(if matches!(member, Member::Dir) {
                            TextValue::Direction(dir_keyword_with(
                                raw.map(|value| value.value().code_units()),
                                |_| read.step(),
                            )?)
                        } else {
                            TextValue::Raw(raw.map(|value| value.value().clone()))
                        })
                    })?;
                    match value {
                        TextValue::Raw(value) => self.copy_dom_units(
                            scope,
                            value.as_ref().map_or(&[], DocumentString::code_units),
                        )?,
                        TextValue::Direction(keyword) => {
                            let mut units = [0_u16; 4];
                            let units = units
                                .get_mut(..keyword.len())
                                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                            for (unit, byte) in units.iter_mut().zip(keyword.bytes()) {
                                *unit = u16::from(byte);
                            }
                            self.copy_dom_units(scope, units)?
                        }
                    }
                }
            };
            Ok(Completion::Return(output))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn document_tree_set<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Document(document)) =
            tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            // Cookie qualifies the current Document facts after coercion.
            if !matches!(member, Member::Cookie) {
                self.callbacks.with_read(&owner, &events, |read, _| {
                    read.node(document.node())?;
                    Ok(())
                })?;
            }
            let absent;
            let input = match arguments.get(0) {
                Some(input) => input,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            if matches!(member, Member::Cookie) {
                self.callbacks
                    .conversion
                    .run(scope, |conversion| conversion.value(&USV_STRING, input))
                    .map_err(Error::from)?;
                scope.check_active()?;
                self.callbacks.with_read(&owner, &events, |read, _| {
                    require_cookie_averse(read, document)
                })?;
                return Ok(Completion::Undefined);
            }
            if matches!(member, Member::Body) {
                let converted = match scope.kind(input)? {
                    ValueKind::Null | ValueKind::Undefined => BodyInput::Null,
                    _ => match tree::projected_node(scope, input)? {
                        Some(GuestNode::Element(node)) => {
                            self.callbacks.with_read(&owner, &events, |read, _| {
                                Ok(
                                    if element::is_html_element(read.node(node)?, || read.step())? {
                                        BodyInput::Element(node)
                                    } else {
                                        BodyInput::TypeMismatch
                                    },
                                )
                            })?
                        }
                        _ => BodyInput::TypeMismatch,
                    },
                };
                scope.check_active()?;
                let node = match converted {
                    BodyInput::Null => {
                        return Ok(Completion::Throw(
                            scope.dom_exception(DomExceptionName::HierarchyRequestError, &[])?,
                        ));
                    }
                    BodyInput::TypeMismatch => {
                        return Ok(Completion::Throw(
                            scope.type_error(&ascii(b"HTMLElement required"))?,
                        ));
                    }
                    BodyInput::Element(node) => node,
                };
                let plan = self.callbacks.with_read(&owner, &events, |read, _| {
                    let node_view = read.node(node)?;
                    let element = node_view.as_element().ok_or(DomError::WrongNodeKind)?;
                    if document_body_role_with(element.name(), |_| read.step())?
                        != HtmlDocumentBodyRole::Body
                    {
                        return Ok(BodyPlan::Hierarchy);
                    }
                    let selected = select_target(read, document, Target::Body)?;
                    if selected.target == Some(node) {
                        return Ok(BodyPlan::Unchanged);
                    }
                    let Some(parent) = selected.document_element else {
                        return Ok(BodyPlan::Hierarchy);
                    };
                    Ok(match selected.target {
                        Some(current) => BodyPlan::Replace {
                            node,
                            parent,
                            current,
                        },
                        None => BodyPlan::Append { node, parent },
                    })
                })?;
                let edit = match plan {
                    BodyPlan::Unchanged => return Ok(Completion::Undefined),
                    BodyPlan::Hierarchy => {
                        return Ok(Completion::Throw(
                            scope.dom_exception(DomExceptionName::HierarchyRequestError, &[])?,
                        ));
                    }
                    BodyPlan::Append { node, parent } => {
                        DomSourceEdit::insert_before(node, parent, None)
                    }
                    BodyPlan::Replace {
                        node,
                        parent,
                        current,
                    } => DomSourceEdit::replace_child(node, parent, current),
                };
                return self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined);
            }
            let descriptor = match member {
                Member::Dir => DOM_STRING,
                Member::FgColor => document_obsolete_fg_color_setter(),
                Member::LinkColor => document_obsolete_link_color_setter(),
                Member::VlinkColor => document_obsolete_vlink_color_setter(),
                Member::AlinkColor => document_obsolete_alink_color_setter(),
                Member::BgColor => document_obsolete_bg_color_setter(),
                Member::Head | Member::Body | Member::Cookie => {
                    return Err(Error::BackendUnavailable.into());
                }
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |cx| cx.value(&descriptor, input))
                .map_err(Error::from)?;
            scope.check_active()?;
            let (target, attribute) = member.text_target().ok_or(Error::BackendUnavailable)?;
            let selected = self.callbacks.with_read(&owner, &events, |read, _| {
                Ok(select_target(read, document, target)?.target)
            })?;
            let Some(target) = selected else {
                return Ok(Completion::Undefined);
            };
            let (value, _value_backing) =
                copy_native_data(&owner, &events, converted.get().as_code_units())?;
            let (name, _name_backing) = copy_native_data(&owner, &events, attribute)?;
            self.apply_attribute(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_attribute_ns(target, Namespace::none(), name, value),
                None,
                element::AttributeResult::Value,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
