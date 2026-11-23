//! HTML Script's native async state and direct content reflections.

use super::{
    element::install_html_token_list,
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_bindings_js::conversion::types::BOOLEAN;
use surgeist_dom::source::DomSourceEdit;
use surgeist_html::elements::token_list::HtmlTokenListReflection;

const BEFORE_ASYNC: &[reflection::Member] = &[
    (
        &ascii(b"type"),
        &ascii(b"get type"),
        &ascii(b"set type"),
        &ascii(b"type"),
        Reflection::String,
    ),
    (
        &ascii(b"noModule"),
        &ascii(b"get noModule"),
        &ascii(b"set noModule"),
        &ascii(b"nomodule"),
        Reflection::Boolean,
    ),
];

const DEFER: reflection::Member = (
    &ascii(b"defer"),
    &ascii(b"get defer"),
    &ascii(b"set defer"),
    &ascii(b"defer"),
    Reflection::Boolean,
);

const AFTER_BLOCKING: &[reflection::Member] = &[
    (
        &ascii(b"crossOrigin"),
        &ascii(b"get crossOrigin"),
        &ascii(b"set crossOrigin"),
        &ascii(b"crossorigin"),
        Reflection::CrossOrigin,
    ),
    (
        &ascii(b"referrerPolicy"),
        &ascii(b"get referrerPolicy"),
        &ascii(b"set referrerPolicy"),
        &ascii(b"referrerpolicy"),
        Reflection::ReferrerPolicy,
    ),
    (
        &ascii(b"integrity"),
        &ascii(b"get integrity"),
        &ascii(b"set integrity"),
        &ascii(b"integrity"),
        Reflection::String,
    ),
    (
        &ascii(b"fetchPriority"),
        &ascii(b"get fetchPriority"),
        &ascii(b"set fetchPriority"),
        &ascii(b"fetchpriority"),
        Reflection::FetchPriority,
    ),
    (
        &ascii(b"charset"),
        &ascii(b"get charset"),
        &ascii(b"set charset"),
        &ascii(b"charset"),
        Reflection::String,
    ),
    (
        &ascii(b"event"),
        &ascii(b"get event"),
        &ascii(b"set event"),
        &ascii(b"event"),
        Reflection::String,
    ),
    (
        &ascii(b"htmlFor"),
        &ascii(b"get htmlFor"),
        &ascii(b"set htmlFor"),
        &ascii(b"for"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Script, BEFORE_ASYNC)?;
    let get = capture(size_of::<Capture>())?;
    let set = capture(size_of::<Capture>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| get.script_async(scope, arguments, false)),
        FunctionMetadata {
            name: &ascii(b"get async"),
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| set.script_async(scope, arguments, true)),
        FunctionMetadata {
            name: &ascii(b"set async"),
            length: 1,
        },
    )?;
    define_accessor(
        scope,
        prototype,
        &ascii(b"async"),
        &getter,
        Some(&setter),
        true,
    )?;
    reflection::install(scope, prototype, capture, Receiver::Script, &[DEFER])?;
    install_html_token_list(
        scope,
        prototype,
        capture,
        HtmlTokenListReflection::ScriptBlocking,
    )?;
    reflection::install(scope, prototype, capture, Receiver::Script, AFTER_BLOCKING)
}

enum AsyncAccess {
    Read(bool),
    Write,
}

impl Capture {
    fn script_async<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        write: bool,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Script;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                if write {
                    return Ok(Some(AsyncAccess::Write));
                }
                let script = node.as_html_script().ok_or(DomError::WrongNodeKind)?;
                script
                    .effective_async_with(|| read.step())
                    .map(|value| Some(AsyncAccess::Read(value)))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            if let AsyncAccess::Read(value) = value {
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Boolean(value))?,
                ));
            }
            let absent;
            let input = match arguments.get(0) {
                Some(input) => input,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            let assigned = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.value(&BOOLEAN, input))
                .map_err(Error::from)?;
            scope.check_active()?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_html_script_async(id, *assigned.get()),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
