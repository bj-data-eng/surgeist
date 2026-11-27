//! HTMLFormElement reflections and reset over the canonical Form.

use super::{
    element::install_html_token_list,
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::source::DomSourceEdit;
use surgeist_html::elements::token_list::HtmlTokenListReflection;

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"acceptCharset"),
        &ascii(b"get acceptCharset"),
        &ascii(b"set acceptCharset"),
        &ascii(b"accept-charset"),
        Reflection::String,
    ),
    (
        &ascii(b"autocomplete"),
        &ascii(b"get autocomplete"),
        &ascii(b"set autocomplete"),
        &ascii(b"autocomplete"),
        Reflection::FormAutocomplete,
    ),
    (
        &ascii(b"enctype"),
        &ascii(b"get enctype"),
        &ascii(b"set enctype"),
        &ascii(b"enctype"),
        Reflection::FormEnctype,
    ),
    (
        &ascii(b"encoding"),
        &ascii(b"get encoding"),
        &ascii(b"set encoding"),
        &ascii(b"enctype"),
        Reflection::FormEnctype,
    ),
    (
        &ascii(b"method"),
        &ascii(b"get method"),
        &ascii(b"set method"),
        &ascii(b"method"),
        Reflection::FormMethod,
    ),
    (
        &ascii(b"name"),
        &ascii(b"get name"),
        &ascii(b"set name"),
        &ascii(b"name"),
        Reflection::String,
    ),
    (
        &ascii(b"noValidate"),
        &ascii(b"get noValidate"),
        &ascii(b"set noValidate"),
        &ascii(b"novalidate"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"target"),
        &ascii(b"get target"),
        &ascii(b"set target"),
        &ascii(b"target"),
        Reflection::String,
    ),
    (
        &ascii(b"rel"),
        &ascii(b"get rel"),
        &ascii(b"set rel"),
        &ascii(b"rel"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Form, MEMBERS)?;
    install_html_token_list(scope, prototype, capture, HtmlTokenListReflection::FormRel)?;
    let handler = capture(size_of::<Capture>())?;
    let name = &ascii(b"reset");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.reset_form(scope, arguments)),
        FunctionMetadata { name, length: 0 },
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn reset_form<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Form;
        let Some(GuestNode::Element(form)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let matches = self.callbacks.with_read(&owner, &events, |read, _| {
                receiver.matches(read.node(form)?, || read.step())
            })?;
            if !matches {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::reset_form(form),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
