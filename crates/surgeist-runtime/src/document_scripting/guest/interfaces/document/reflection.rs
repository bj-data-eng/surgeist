//! Reflected attributes over authenticated live elements and Source edits.

use super::{
    element::{HTML_NAMESPACE, MATHML_NAMESPACE, SVG_NAMESPACE},
    table_legacy::TableFamily,
    text::TextBuffer,
    *,
};
use std::fmt::Write;
use surgeist_bindings_js::conversion::types::{BOOLEAN, DOM_STRING, DOM_STRING_NULL_TO_EMPTY};
use surgeist_bindings_js::conversion::{HiddenAttributeValue, IntegerType, declarations};
use surgeist_bindings_js::{
    IntegerKind, IntegerMode, IntegerValue, dom_exception::DomExceptionName,
};
use surgeist_dom::{source::DomSourceEdit, tree::DomNode};
use surgeist_html::attributes::command::HtmlCommandState;
use surgeist_html::attributes::{
    HtmlClosedByState, HtmlContentEditableState, HtmlHidden, HtmlHiddenState, HtmlTranslate,
};
use surgeist_html::bounded::attributes::{
    HtmlAutocompleteControl, HtmlControlDimensionRule, HtmlHiddenSetterAction,
    HtmlHiddenSetterInput, HtmlTableSpanRule, button_command_readback_with,
    closedby_processing_with, contenteditable_processing_with, contenteditable_setter_state_with,
    control_autocomplete_idl_with, control_length_reflection_with, control_length_setter_value,
    dir_keyword_with, enterkeyhint_keyword_with, form_autocomplete_keyword_with,
    form_enctype_keyword_with, form_enctype_override_keyword_with, form_method_keyword_with,
    form_method_override_keyword_with, heading_offset_reflection_with, heading_offset_setter_value,
    hidden_processing_with, hidden_setter_action_with, inputmode_keyword_with,
    popover_keyword_with, table_cell_scope_keyword_with, template_shadow_root_mode_keyword_with,
    template_shadow_root_slot_assignment_keyword_with,
};
use surgeist_html::bounded::input::input_colorspace_with;
use surgeist_html::bounded::marquee::{
    HtmlMarqueeUnsignedReflection, marquee_loop_setter_value_with, marquee_loop_value_with,
};
use surgeist_html::bounded::numeric::{
    HtmlNumericParseError, integer_long_work_bound, parse_integer_long,
};
use surgeist_html::bounded::select::{
    select_size_reflection_setter_value, select_size_reflection_with,
};
use surgeist_html::elements::dialog::computed_dialog_closed_by;
use surgeist_html::elements::media::HtmlCanvasDimension;
use surgeist_html::values::Namespace;
use surgeist_html::vocabulary::KnownHtmlElement;
use surgeist_html_semantics::HtmlTranslationMode;

#[derive(Clone, Copy)]
pub(super) enum Reflection {
    String,
    StringNullToEmpty,
    Boolean,
    Mode,
    SlotAssignment,
    Direction,
    Translate,
    Hidden,
    DialogClosedBy,
    ButtonCommand,
    ButtonType,
    FormAutocomplete,
    ControlAutocomplete,
    FormEnctype,
    FormMethod,
    FormEnctypeOverride,
    FormMethodOverride,
    InputType,
    InputColorSpace,
    Popover,
    HeadingOffset,
    TabIndex,
    ContentEditable,
    EnterKeyHint,
    InputMode,
    // Unit policies keep frequently captured reflection metadata compact.
    SignedLongZero,
    SignedLongOne,
    ControlLength,
    ControlDimension(HtmlControlDimensionRule),
    SelectSize,
    CanvasDimension(HtmlCanvasDimension),
    MarqueeUnsigned(HtmlMarqueeUnsignedReflection),
    MarqueeLoop,
    TableSpan(TableSpan),
    TableScope,
}

#[derive(Clone, Copy)]
pub(super) enum TableSpan {
    CellColumns,
    CellRows,
    Column,
}

impl TableSpan {
    fn rule(
        self,
        node: DomNode<'_>,
        read: &NativeRead<'_>,
    ) -> Result<HtmlTableSpanRule, DocumentGuestEventError> {
        Ok(match self {
            Self::CellColumns => HtmlTableSpanRule::CellColspan,
            Self::CellRows => HtmlTableSpanRule::CellRowspan,
            Self::Column => match super::table_cell_column::html_kind(read, node)? {
                Some(KnownHtmlElement::Col) => HtmlTableSpanRule::ColSpan,
                Some(KnownHtmlElement::Colgroup) => HtmlTableSpanRule::ColgroupSpan,
                _ => return Err(DomError::WrongNodeKind.into()),
            },
        })
    }
}

#[derive(Clone, Copy)]
pub(super) enum Receiver {
    Element,
    Html,
    Svg,
    Mathml,
    Template,
    Body,
    Button,
    Canvas,
    Br,
    Hr,
    Data,
    Details,
    Dialog,
    OptGroup,
    Option,
    Select,
    Time,
    Div,
    Paragraph,
    Heading,
    DList,
    Directory,
    Fieldset,
    Font,
    Form,
    Frameset,
    Input,
    Label,
    Legend,
    Marquee,
    Media,
    Meter,
    Li,
    OList,
    UList,
    Menu,
    Meta,
    Output,
    Param,
    Pre,
    Progress,
    Slot,
    TableCell,
    TableColumn,
    TableLegacy(TableFamily),
    Textarea,
}

impl Receiver {
    pub(super) fn matches(
        self,
        node: DomNode<'_>,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
    ) -> Result<bool, DocumentGuestEventError> {
        Ok(match self {
            Self::Element => node.as_element().is_some(),
            Self::Html => node.as_element().is_some_and(|element| {
                element
                    .name()
                    .namespace_uri()
                    .is_some_and(|uri| uri.code_units() == HTML_NAMESPACE)
            }),
            Self::Svg | Self::Mathml => {
                step()?;
                let namespace = if matches!(self, Self::Svg) {
                    SVG_NAMESPACE
                } else {
                    MATHML_NAMESPACE
                };
                node.as_element().is_some_and(|element| {
                    element
                        .name()
                        .namespace_uri()
                        .is_some_and(|uri| uri.code_units() == namespace)
                })
            }
            Self::Template => node.as_template().is_some(),
            Self::Body => {
                step()?;
                html_element_is(node, "HTMLBodyElement")
            }
            Self::Button => {
                step()?;
                node.as_element().is_some_and(|element| {
                    !element.has_failed_construction_interface() && element.button_state().is_some()
                })
            }
            Self::Br => html_element_is(node, "HTMLBRElement"),
            Self::Hr => html_element_is(node, "HTMLHRElement"),
            Self::Data => html_element_is(node, "HTMLDataElement"),
            Self::Details => html_element_is(node, "HTMLDetailsElement"),
            Self::Dialog => {
                step()?;
                node.as_dialog().is_some()
                    && node
                        .as_element()
                        .is_some_and(|element| !element.has_failed_construction_interface())
            }
            Self::OptGroup => html_element_is(node, "HTMLOptGroupElement"),
            Self::Option => html_element_is(node, "HTMLOptionElement"),
            Self::Select => html_element_is(node, "HTMLSelectElement"),
            Self::Time => html_element_is(node, "HTMLTimeElement"),
            Self::Div => html_element_is(node, "HTMLDivElement"),
            Self::Paragraph => html_element_is(node, "HTMLParagraphElement"),
            Self::Heading => html_element_is(node, "HTMLHeadingElement"),
            Self::DList => html_element_is(node, "HTMLDListElement"),
            Self::Directory => super::list_compact::is_directory(node),
            Self::Fieldset => {
                step()?;
                node.as_fieldset().is_some()
                    && node
                        .as_element()
                        .is_some_and(|element| !element.has_failed_construction_interface())
            }
            Self::Font => super::font::is_font(node),
            Self::Form => {
                step()?;
                node.as_form().is_some()
                    && node
                        .as_element()
                        .is_some_and(|element| !element.has_failed_construction_interface())
            }
            Self::Frameset => {
                step()?;
                super::frameset::is_frameset(node)
            }
            Self::Input => {
                step()?;
                node.as_element().is_some_and(|element| {
                    !element.has_failed_construction_interface() && element.input_state().is_some()
                })
            }
            Self::Label => {
                step()?;
                html_element_is(node, "HTMLLabelElement")
            }
            Self::Legend => {
                step()?;
                html_element_is(node, "HTMLLegendElement")
            }
            Self::Marquee => {
                step()?;
                node.as_element().is_some_and(|element| {
                    !element.has_failed_construction_interface()
                        && element.marquee_state().is_some()
                })
            }
            Self::Li => html_element_is(node, "HTMLLIElement"),
            Self::OList => html_element_is(node, "HTMLOListElement"),
            Self::UList => html_element_is(node, "HTMLUListElement"),
            Self::Menu => html_element_is(node, "HTMLMenuElement"),
            Self::Meta => html_element_is(node, "HTMLMetaElement"),
            Self::Output => {
                step()?;
                node.as_element().is_some_and(|element| {
                    !element.has_failed_construction_interface() && element.output_state().is_some()
                })
            }
            Self::Param => super::param::is_param(node),
            Self::Pre => return super::pre::is_pre(node, step),
            Self::Slot => super::slot::is_slot(node),
            Self::TableCell => {
                step()?;
                html_element_is(node, "HTMLTableCellElement")
            }
            Self::Textarea => {
                step()?;
                node.as_element().is_some_and(|element| {
                    !element.has_failed_construction_interface()
                        && element.textarea_state().is_some()
                })
            }
            Self::TableColumn => {
                step()?;
                html_element_is(node, "HTMLTableColElement")
            }
            Self::TableLegacy(family) => {
                step()?;
                html_element_is(node, family.interface())
            }
            Self::Progress => {
                step()?;
                html_element_is(node, "HTMLProgressElement")
            }
            Self::Media => {
                step()?;
                node.as_element().is_some_and(|element| {
                    !element.has_failed_construction_interface()
                        && matches!(
                            KnownHtmlElement::from_name(element.name()),
                            Some(KnownHtmlElement::Audio | KnownHtmlElement::Video)
                        )
                })
            }
            Self::Meter => {
                step()?;
                html_element_is(node, "HTMLMeterElement")
            }
            Self::Canvas => {
                step()?;
                html_element_is(node, "HTMLCanvasElement")
            }
        })
    }

    pub(super) fn error_message(self) -> &'static [u16] {
        match self {
            Self::Element => const { &ascii(b"Element required") },
            Self::Html => const { &ascii(b"HTMLElement required") },
            Self::Svg => const { &ascii(b"SVGElement required") },
            Self::Mathml => const { &ascii(b"MathMLElement required") },
            Self::Template => const { &ascii(b"HTMLTemplateElement required") },
            Self::Body => const { &ascii(b"HTMLBodyElement required") },
            Self::Button => const { &ascii(b"HTMLButtonElement required") },
            Self::Canvas => const { &ascii(b"HTMLCanvasElement required") },
            Self::Br => const { &ascii(b"HTMLBRElement required") },
            Self::Hr => const { &ascii(b"HTMLHRElement required") },
            Self::Data => const { &ascii(b"HTMLDataElement required") },
            Self::Details => const { &ascii(b"HTMLDetailsElement required") },
            Self::Dialog => const { &ascii(b"HTMLDialogElement required") },
            Self::OptGroup => const { &ascii(b"HTMLOptGroupElement required") },
            Self::Option => const { &ascii(b"HTMLOptionElement required") },
            Self::Select => const { &ascii(b"HTMLSelectElement required") },
            Self::Time => const { &ascii(b"HTMLTimeElement required") },
            Self::Div => const { &ascii(b"HTMLDivElement required") },
            Self::Paragraph => const { &ascii(b"HTMLParagraphElement required") },
            Self::Heading => const { &ascii(b"HTMLHeadingElement required") },
            Self::DList => const { &ascii(b"HTMLDListElement required") },
            Self::Directory => const { &ascii(b"HTMLDirectoryElement required") },
            Self::Fieldset => const { &ascii(b"HTMLFieldSetElement required") },
            Self::Font => const { &ascii(b"HTMLFontElement required") },
            Self::Form => const { &ascii(b"HTMLFormElement required") },
            Self::Frameset => const { &ascii(b"HTMLFrameSetElement required") },
            Self::Input => const { &ascii(b"HTMLInputElement required") },
            Self::Label => const { &ascii(b"HTMLLabelElement required") },
            Self::Legend => const { &ascii(b"HTMLLegendElement required") },
            Self::Marquee => const { &ascii(b"HTMLMarqueeElement required") },
            Self::Media => const { &ascii(b"HTMLMediaElement required") },
            Self::Meter => const { &ascii(b"HTMLMeterElement required") },
            Self::Li => const { &ascii(b"HTMLLIElement required") },
            Self::OList => const { &ascii(b"HTMLOListElement required") },
            Self::UList => const { &ascii(b"HTMLUListElement required") },
            Self::Menu => const { &ascii(b"HTMLMenuElement required") },
            Self::Meta => const { &ascii(b"HTMLMetaElement required") },
            Self::Output => const { &ascii(b"HTMLOutputElement required") },
            Self::Param => const { &ascii(b"HTMLParamElement required") },
            Self::Pre => const { &ascii(b"HTMLPreElement required") },
            Self::Progress => const { &ascii(b"HTMLProgressElement required") },
            Self::Slot => const { &ascii(b"HTMLSlotElement required") },
            Self::TableCell => const { &ascii(b"HTMLTableCellElement required") },
            Self::TableColumn => const { &ascii(b"HTMLTableColElement required") },
            Self::TableLegacy(family) => family.error_message(),
            Self::Textarea => const { &ascii(b"HTMLTextAreaElement required") },
        }
    }
}

fn html_element_is(node: DomNode<'_>, interface: &str) -> bool {
    node.as_element().is_some_and(|element| {
        !element.has_failed_construction_interface()
            && KnownHtmlElement::from_name(element.name())
                .is_some_and(|known| known.inventory_row().interface() == interface)
    })
}

pub(super) type Member = (
    &'static [u16],
    &'static [u16],
    &'static [u16],
    &'static [u16],
    Reflection,
);

pub(super) const CONTROL_AUTOCOMPLETE: Member = (
    &ascii(b"autocomplete"),
    &ascii(b"get autocomplete"),
    &ascii(b"set autocomplete"),
    &ascii(b"autocomplete"),
    Reflection::ControlAutocomplete,
);

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: Receiver,
    members: &[Member],
) -> Result<(), DocumentGuestEventError> {
    for &(member, get_name, set_name, attribute, reflection) in members {
        let get = capture(size_of::<(Capture, Receiver, &[u16], Reflection)>())?;
        let set = capture(size_of::<(Capture, Receiver, &[u16], Reflection)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                get.reflect_attribute(scope, arguments, receiver, attribute, reflection, false)
            }),
            FunctionMetadata {
                name: get_name,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                set.reflect_attribute(scope, arguments, receiver, attribute, reflection, true)
            }),
            FunctionMetadata {
                name: set_name,
                length: 1,
            },
        )?;
        scope.define_new_property(
            prototype,
            PropertyKey::String(member),
            PropertyDescriptor::Accessor {
                get: Some(&getter),
                set: Some(&setter),
                enumerable: true,
                configurable: true,
            },
        )?;
    }
    Ok(())
}

pub(super) fn install_is_content_editable<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let getter = capture(size_of::<Capture>())?;
    super::install_getter(
        scope,
        prototype,
        &ascii(b"isContentEditable"),
        &ascii(b"get isContentEditable"),
        true,
        Callback::new(move |scope, arguments| getter.is_content_editable(scope, arguments)),
    )?;
    Ok(())
}

impl Capture {
    fn is_content_editable<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Html;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let editable = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                read.snapshot
                    .html_is_content_editable_with(id, || read.step())
                    .map(Some)
            })?;
            match editable {
                Some(value) => Ok(Completion::Return(
                    scope.primitive(Primitive::Boolean(value))?,
                )),
                None => Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                )),
            }
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn reflect_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
        attribute: &[u16],
        reflection: Reflection,
        write: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                if write {
                    return Ok(Some(Reflected::Write));
                }
                let stored = if matches!(
                    reflection,
                    Reflection::Translate
                        | Reflection::TabIndex
                        | Reflection::InputType
                        | Reflection::CanvasDimension(_)
                ) {
                    None
                } else {
                    let attrs = node.attributes().ok_or(DomError::WrongNodeKind)?;
                    attrs.get_null_namespace_attribute_with(attribute, || read.step())?
                };
                let units = stored.map(|value| value.value().code_units());
                Ok(Some(match reflection {
                    Reflection::Boolean => Reflected::Boolean(stored.is_some()),
                    Reflection::String | Reflection::StringNullToEmpty => {
                        Reflected::String(stored.map(|value| value.value().clone()))
                    }
                    Reflection::Mode => Reflected::Keyword(Some(
                        template_shadow_root_mode_keyword_with(units, |_| read.step())?,
                    )),
                    Reflection::SlotAssignment => Reflected::Keyword(Some(
                        template_shadow_root_slot_assignment_keyword_with(units, |_| read.step())?,
                    )),
                    Reflection::Direction => {
                        Reflected::Keyword(Some(dir_keyword_with(units, |_| read.step())?))
                    }
                    Reflection::Translate => Reflected::Boolean(
                        read.snapshot
                            .html_translation_mode_with(id, || read.step())?
                            == HtmlTranslationMode::Enabled,
                    ),
                    Reflection::Hidden => match hidden_processing_with(units, |_| read.step())? {
                        HtmlHiddenState::NotHidden => Reflected::Boolean(false),
                        HtmlHiddenState::Keyword(HtmlHidden::Hidden) => Reflected::Boolean(true),
                        HtmlHiddenState::Keyword(keyword @ HtmlHidden::UntilFound) => {
                            Reflected::Keyword(Some(keyword.canonical_keyword()))
                        }
                    },
                    Reflection::TabIndex => Reflected::Signed(node.tab_index_with(|| read.step())?),
                    Reflection::DialogClosedBy => {
                        let state = closedby_processing_with(units, |_| read.step())?;
                        let value = match state {
                            HtmlClosedByState::Keyword(keyword) => keyword,
                            HtmlClosedByState::Auto => {
                                read.step()?;
                                let dialog = node.as_dialog().ok_or(DomError::WrongNodeKind)?;
                                computed_dialog_closed_by(state, dialog.is_modal())
                            }
                        };
                        Reflected::Keyword(Some(value.canonical_keyword()))
                    }
                    Reflection::ButtonCommand => {
                        let command = button_command_readback_with(
                            stored.map(|value| value.value()),
                            |_| read.step(),
                        )?;
                        match command.state() {
                            HtmlCommandState::Standard(keyword) => {
                                Reflected::Keyword(Some(keyword.canonical_keyword()))
                            }
                            HtmlCommandState::Custom | HtmlCommandState::Unknown => {
                                Reflected::String(command.custom_value().cloned())
                            }
                        }
                    }
                    Reflection::ButtonType => {
                        Reflected::Keyword(Some(super::button::type_keyword(read, node, units)?))
                    }
                    Reflection::FormAutocomplete => {
                        Reflected::Keyword(Some(form_autocomplete_keyword_with(units, |_| {
                            read.step()
                        })?))
                    }
                    Reflection::ControlAutocomplete => {
                        let control = match receiver {
                            Receiver::Input => HtmlAutocompleteControl::Input(
                                node.as_input()
                                    .ok_or(DomError::WrongNodeKind)?
                                    .input_type_with(|| read.step())?,
                            ),
                            Receiver::Select => HtmlAutocompleteControl::Select,
                            Receiver::Textarea => HtmlAutocompleteControl::Textarea,
                            _ => return Err(DomError::WrongNodeKind.into()),
                        };
                        let plan = control_autocomplete_idl_with(units, control, |work| {
                            for _ in 0..work {
                                read.step()?;
                            }
                            Ok::<_, DocumentGuestEventError>(())
                        })?;
                        let length = plan
                            .code_units_len()
                            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                        Reflected::Autocomplete(TextBuffer::build_exact(read, length, |emit| {
                            plan.visit_code_units_with(|| read.step(), emit)
                        })?)
                    }
                    Reflection::FormEnctype => {
                        Reflected::Keyword(Some(form_enctype_keyword_with(units, |_| read.step())?))
                    }
                    Reflection::FormMethod => {
                        Reflected::Keyword(Some(form_method_keyword_with(units, |_| read.step())?))
                    }
                    Reflection::FormEnctypeOverride => {
                        Reflected::Keyword(Some(form_enctype_override_keyword_with(units, |_| {
                            read.step()
                        })?))
                    }
                    Reflection::FormMethodOverride => {
                        Reflected::Keyword(Some(form_method_override_keyword_with(units, |_| {
                            read.step()
                        })?))
                    }
                    Reflection::InputType => {
                        let input = node.as_input().ok_or(DomError::WrongNodeKind)?;
                        let state = input.input_type_with(|| read.step())?;
                        Reflected::Keyword(Some(state.canonical_keyword()))
                    }
                    Reflection::InputColorSpace => {
                        let space = input_colorspace_with(units, || read.step())?;
                        Reflected::Keyword(Some(space.canonical_keyword()))
                    }
                    Reflection::Popover => {
                        Reflected::Keyword(popover_keyword_with(units, |_| read.step())?)
                    }
                    Reflection::HeadingOffset => Reflected::Unsigned(u32::from(
                        heading_offset_reflection_with(units, |_| read.step())?,
                    )),
                    Reflection::ContentEditable => Reflected::Keyword(Some(
                        contenteditable_processing_with(units, |_| read.step())?
                            .reflection_keyword(),
                    )),
                    Reflection::EnterKeyHint => {
                        Reflected::Keyword(Some(enterkeyhint_keyword_with(units, |_| read.step())?))
                    }
                    Reflection::InputMode => {
                        Reflected::Keyword(Some(inputmode_keyword_with(units, |_| read.step())?))
                    }
                    Reflection::TableSpan(span) => {
                        Reflected::Unsigned(u32::from(table_span_value(
                            span.rule(node, read)?,
                            stored.map(|value| value.value()),
                            || read.step(),
                        )?))
                    }
                    Reflection::TableScope => Reflected::Keyword(Some(
                        table_cell_scope_keyword_with(stored.map(|value| value.value()), |_| {
                            read.step()
                        })?,
                    )),
                    Reflection::SignedLongZero => Reflected::Signed(signed_long_value(
                        stored.map(|value| value.value()),
                        0,
                        || read.step(),
                    )?),
                    Reflection::SignedLongOne => Reflected::Signed(signed_long_value(
                        stored.map(|value| value.value()),
                        1,
                        || read.step(),
                    )?),
                    Reflection::ControlLength => {
                        Reflected::Signed(control_length_reflection_with(units, |_| read.step())?)
                    }
                    Reflection::ControlDimension(rule) => {
                        Reflected::Unsigned(rule.read_with(units, |_| read.step())?)
                    }
                    Reflection::SelectSize => Reflected::Unsigned(select_size_reflection_with(
                        stored.map(|value| value.value()),
                        || read.step(),
                    )?),
                    Reflection::MarqueeUnsigned(rule) => Reflected::Unsigned(
                        rule.read_with(stored.map(|value| value.value()), || read.step())?,
                    ),
                    Reflection::CanvasDimension(dimension) => {
                        let state = node
                            .as_element()
                            .and_then(|element| element.canvas_none_state())
                            .ok_or(DomError::WrongNodeKind)?;
                        Reflected::Unsigned(
                            state.reflected_dimension_with(dimension, || read.step())?,
                        )
                    }
                    Reflection::MarqueeLoop => Reflected::Signed(marquee_loop_value_with(
                        stored.map(|value| value.value()),
                        || read.step(),
                    )?),
                }))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            let output = match value {
                Reflected::Write => None,
                Reflected::Boolean(value) => Some(scope.primitive(Primitive::Boolean(value))?),
                Reflected::Unsigned(value) => {
                    Some(scope.primitive(Primitive::Number(f64::from(value)))?)
                }
                Reflected::Signed(value) => {
                    Some(scope.primitive(Primitive::Number(f64::from(value)))?)
                }
                Reflected::String(value) => Some(self.copy_dom_units(
                    scope,
                    value.as_ref().map_or(&[], DocumentString::code_units),
                )?),
                Reflected::Autocomplete(value) => {
                    Some(self.copy_dom_units(scope, value.code_units())?)
                }
                Reflected::Keyword(None) => Some(scope.primitive(Primitive::Null)?),
                Reflected::Keyword(Some(keyword)) => {
                    let mut units = [0; MAX_KEYWORD_UNITS];
                    Some(self.copy_dom_units(scope, keyword_units(keyword, &mut units)?)?)
                }
            };
            if let Some(output) = output {
                return Ok(Completion::Return(output));
            }
            let absent;
            let input = match arguments.get(0) {
                Some(input) => input,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            let value = if matches!(reflection, Reflection::Hidden) {
                self.prepare_hidden_attribute(scope, &owner, &events, id, input)?
            } else if matches!(reflection, Reflection::Boolean | Reflection::Translate) {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.value(&BOOLEAN, input))
                    .map_err(Error::from)?;
                if matches!(reflection, Reflection::Translate) {
                    scope.check_active()?;
                    self.callbacks
                        .with_read(&owner, &events, |read, _| read.node(id).map(drop))?;
                    let keyword = if *converted.get() {
                        HtmlTranslate::Yes
                    } else {
                        HtmlTranslate::No
                    };
                    let mut units = [0; MAX_KEYWORD_UNITS];
                    let units = keyword_units(keyword.canonical_keyword(), &mut units)?;
                    Some(copy_native_data(&owner, &events, units)?)
                } else if *converted.get() {
                    Some(copy_native_data(&owner, &events, &[])?)
                } else {
                    None
                }
            } else if matches!(reflection, Reflection::Popover) {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.value(&DOM_STRING.nullable(), input))
                    .map_err(Error::from)?;
                scope.check_active()?;
                converted
                    .get()
                    .as_ref()
                    .map(|value| copy_native_data(&owner, &events, value.as_code_units()))
                    .transpose()?
            } else if matches!(
                reflection,
                Reflection::HeadingOffset
                    | Reflection::SelectSize
                    | Reflection::ControlDimension(_)
                    | Reflection::TableSpan(_)
                    | Reflection::MarqueeUnsigned(_)
                    | Reflection::CanvasDimension(_)
            ) {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| {
                        cx.value(
                            &IntegerType::new(IntegerKind::UnsignedLong, IntegerMode::Wrap),
                            input,
                        )
                    })
                    .map_err(Error::from)?;
                scope.check_active()?;
                let IntegerValue::UnsignedLong(value) = *converted.get() else {
                    return Err(Error::BackendUnavailable.into());
                };
                events.step()?;
                if matches!(
                    reflection,
                    Reflection::ControlDimension(_)
                        | Reflection::MarqueeUnsigned(_)
                        | Reflection::CanvasDimension(_)
                ) {
                    let current = self.callbacks.with_read(&owner, &events, |read, _| {
                        receiver.matches(read.node(id)?, || read.step())
                    })?;
                    if !current {
                        return Ok(Completion::Throw(
                            scope.type_error(receiver.error_message())?,
                        ));
                    }
                }
                let value = match reflection {
                    Reflection::HeadingOffset => heading_offset_setter_value(value),
                    Reflection::SelectSize => select_size_reflection_setter_value(value),
                    Reflection::ControlDimension(rule) => {
                        let Ok(value) = rule.setter_value(value) else {
                            return Ok(Completion::Throw(
                                scope.dom_exception(DomExceptionName::IndexSizeError, &[])?,
                            ));
                        };
                        value
                    }
                    Reflection::MarqueeUnsigned(rule) => rule.setter_value(value),
                    Reflection::CanvasDimension(dimension) => dimension.reflected_value(value),
                    _ => HtmlTableSpanRule::reflected_setter_value(value),
                };
                Some(copy_numeric_attribute(
                    &owner,
                    &events,
                    NumericAttribute::Unsigned(value),
                )?)
            } else if matches!(
                reflection,
                Reflection::SignedLongZero
                    | Reflection::SignedLongOne
                    | Reflection::ControlLength
                    | Reflection::MarqueeLoop
                    | Reflection::TabIndex
            ) {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| {
                        cx.value(
                            &IntegerType::new(IntegerKind::Long, IntegerMode::Wrap),
                            input,
                        )
                    })
                    .map_err(Error::from)?;
                scope.check_active()?;
                let IntegerValue::Long(value) = *converted.get() else {
                    return Err(Error::BackendUnavailable.into());
                };
                if matches!(reflection, Reflection::TabIndex | Reflection::ControlLength) {
                    let current = self.callbacks.with_read(&owner, &events, |read, _| {
                        receiver.matches(read.node(id)?, || read.step())
                    })?;
                    if !current {
                        return Ok(Completion::Throw(
                            scope.type_error(receiver.error_message())?,
                        ));
                    }
                }
                let value = if matches!(reflection, Reflection::ControlLength) {
                    events.step()?;
                    let Ok(value) = control_length_setter_value(value) else {
                        return Ok(Completion::Throw(
                            scope.dom_exception(DomExceptionName::IndexSizeError, &[])?,
                        ));
                    };
                    value
                } else if matches!(reflection, Reflection::MarqueeLoop) {
                    let Some(change) = self.callbacks.with_read(&owner, &events, |read, _| {
                        let node = read.node(id)?;
                        if !receiver.matches(node, || read.step())? {
                            return Ok(None);
                        }
                        let attrs = node.attributes().ok_or(DomError::WrongNodeKind)?;
                        let stored =
                            attrs.get_null_namespace_attribute_with(attribute, || read.step())?;
                        marquee_loop_setter_value_with(
                            value,
                            stored.map(|value| value.value()),
                            || read.step(),
                        )
                        .map(Some)
                    })?
                    else {
                        return Ok(Completion::Throw(
                            scope.type_error(receiver.error_message())?,
                        ));
                    };
                    let Some(change) = change else {
                        return Ok(Completion::Undefined);
                    };
                    change
                } else {
                    value
                };
                Some(copy_numeric_attribute(
                    &owner,
                    &events,
                    NumericAttribute::SignedLong(value),
                )?)
            } else {
                let descriptor = if matches!(reflection, Reflection::StringNullToEmpty) {
                    &DOM_STRING_NULL_TO_EMPTY
                } else {
                    &DOM_STRING
                };
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.value(descriptor, input))
                    .map_err(Error::from)?;
                scope.check_active()?;
                let raw = converted.get().as_code_units();
                if matches!(reflection, Reflection::ContentEditable) {
                    let state = self.callbacks.with_read(&owner, &events, |read, _| {
                        read.node(id)?;
                        contenteditable_setter_state_with(raw, |_| read.step())
                    })?;
                    match state {
                        Some(HtmlContentEditableState::Inherit) => None,
                        Some(HtmlContentEditableState::Keyword(keyword)) => {
                            let mut units = [0; MAX_KEYWORD_UNITS];
                            let units = keyword_units(keyword.canonical_keyword(), &mut units)?;
                            Some(copy_native_data(&owner, &events, units)?)
                        }
                        None => {
                            return Ok(Completion::Throw(
                                scope.dom_exception(DomExceptionName::SyntaxError, &[])?,
                            ));
                        }
                    }
                } else {
                    if matches!(
                        receiver,
                        Receiver::Body
                            | Receiver::Button
                            | Receiver::Dialog
                            | Receiver::Fieldset
                            | Receiver::Form
                            | Receiver::Frameset
                            | Receiver::Input
                            | Receiver::Label
                            | Receiver::Legend
                            | Receiver::Marquee
                            | Receiver::Output
                            | Receiver::TableLegacy(_)
                            | Receiver::Textarea
                    ) || matches!(reflection, Reflection::ControlAutocomplete)
                    {
                        let current = self.callbacks.with_read(&owner, &events, |read, _| {
                            receiver.matches(read.node(id)?, || read.step())
                        })?;
                        if !current {
                            return Ok(Completion::Throw(
                                scope.type_error(receiver.error_message())?,
                            ));
                        }
                    } else if matches!(reflection, Reflection::EnterKeyHint | Reflection::InputMode)
                    {
                        self.callbacks
                            .with_read(&owner, &events, |read, _| read.node(id).map(drop))?;
                    }
                    Some(copy_native_data(&owner, &events, raw)?)
                }
            };
            let (name, _name_backing) = copy_native_data(&owner, &events, attribute)?;
            match value {
                Some((value, _backing)) => self.apply_attribute(
                    scope,
                    &owner,
                    &events,
                    DomSourceEdit::set_attribute_ns(id, Namespace::none(), name, value),
                    None,
                    element::AttributeResult::Value,
                ),
                None => self.apply_tree_edit(
                    scope,
                    &owner,
                    &events,
                    DomSourceEdit::remove_attribute_ns(id, Namespace::none(), name),
                    Completion::Undefined,
                ),
            }
        })();
        tree::tree_completion(scope, &events, result)
    }
}

impl Capture {
    fn prepare_hidden_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        id: NodeId,
        input: &Value<'scope>,
    ) -> Result<Option<(DocumentString, GuestBacking)>, DocumentGuestEventError> {
        let converted = self
            .callbacks
            .conversion
            .run(scope, |cx| {
                cx.value(&declarations::html_element_hidden_setter(), input)
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let value = match converted.get() {
            None => HtmlHiddenSetterInput::Null,
            Some(HiddenAttributeValue::Boolean(value)) => HtmlHiddenSetterInput::Boolean(*value),
            Some(HiddenAttributeValue::Number(value)) => HtmlHiddenSetterInput::Number(*value),
            Some(HiddenAttributeValue::String(value)) => {
                HtmlHiddenSetterInput::String(value.as_code_units())
            }
        };
        let action = self.callbacks.with_read(owner, events, |read, _| {
            read.node(id)?;
            hidden_setter_action_with(value, |_| read.step())
        })?;
        match action {
            HtmlHiddenSetterAction::Remove => Ok(None),
            HtmlHiddenSetterAction::SetEmpty => copy_native_data(owner, events, &[]).map(Some),
            HtmlHiddenSetterAction::SetUntilFound => {
                let mut units = [0; MAX_KEYWORD_UNITS];
                let units = keyword_units(HtmlHidden::UntilFound.canonical_keyword(), &mut units)?;
                copy_native_data(owner, events, units).map(Some)
            }
        }
    }
}

// HTML's longest canonical keyword among these reflected attributes.
const MAX_KEYWORD_UNITS: usize = b"application/x-www-form-urlencoded".len();

fn keyword_units<'a>(
    keyword: &str,
    units: &'a mut [u16; MAX_KEYWORD_UNITS],
) -> Result<&'a [u16], DocumentScriptNativeError> {
    let units = units
        .get_mut(..keyword.len())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    for (unit, byte) in units.iter_mut().zip(keyword.bytes()) {
        *unit = u16::from(byte);
    }
    Ok(units)
}

enum Reflected {
    Write,
    Boolean(bool),
    Unsigned(u32),
    Signed(i32),
    String(Option<DocumentString>),
    Autocomplete(TextBuffer),
    Keyword(Option<&'static str>),
}

enum NumericAttribute {
    Unsigned(u32),
    SignedLong(i32),
}

fn signed_long_value(
    raw: Option<&DocumentString>,
    fallback: i32,
    mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
) -> Result<i32, DocumentGuestEventError> {
    let Some(raw) = raw else {
        return Ok(fallback);
    };
    let work = integer_long_work_bound(raw.len())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    for _ in 0..work {
        step()?;
    }
    match parse_integer_long(raw) {
        Ok(Some(value)) => Ok(value),
        Ok(None) | Err(HtmlNumericParseError::InvalidSyntax) => Ok(fallback),
        Err(_) => Err(Error::BackendUnavailable.into()),
    }
}

fn table_span_value(
    rule: HtmlTableSpanRule,
    raw: Option<&DocumentString>,
    mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
) -> Result<u16, DocumentGuestEventError> {
    let work = match raw {
        Some(raw) => rule
            .work_bound(raw.len())
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?,
        None => 1,
    };
    for _ in 0..work {
        step()?;
    }
    Ok(rule.content_value(raw))
}

fn copy_numeric_attribute(
    owner: &NativeOwner,
    events: &GuestEvents,
    value: NumericAttribute,
) -> Result<(DocumentString, GuestBacking), DocumentGuestEventError> {
    // Typed decimal cases fix the maximum capacity before formatting: a u32
    // has ten digits, and an i32 has at most ten digits plus its minus sign.
    let capacity = match value {
        NumericAttribute::Unsigned(_) => 10,
        NumericAttribute::SignedLong(_) => 11,
    };
    let _format_backing = events.reserve(owner, capacity)?;
    let mut text = String::new();
    text.try_reserve_exact(capacity)
        .map_err(|_| Error::Allocation)?;
    if text.capacity() > capacity {
        return Err(DocumentScriptNativeError::StorageLimit.into());
    }
    events.step()?;
    match value {
        NumericAttribute::Unsigned(value) => write!(&mut text, "{value}"),
        NumericAttribute::SignedLong(value) => write!(&mut text, "{value}"),
    }
    .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
    let mut units = [0_u16; 11];
    for (byte, unit) in text.as_bytes().iter().zip(&mut units) {
        events.step()?;
        *unit = u16::from(*byte);
    }
    let units = units
        .get(..text.len())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    copy_native_data(owner, events, units)
}
