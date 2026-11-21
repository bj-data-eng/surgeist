//! HTMLMarqueeElement reflection and commands over the canonical native element.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::source::DomSourceEdit;
use surgeist_html::bounded::marquee::HtmlMarqueeUnsignedReflection;

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"behavior"),
        &ascii(b"get behavior"),
        &ascii(b"set behavior"),
        &ascii(b"behavior"),
        Reflection::String,
    ),
    (
        &ascii(b"bgColor"),
        &ascii(b"get bgColor"),
        &ascii(b"set bgColor"),
        &ascii(b"bgcolor"),
        Reflection::String,
    ),
    (
        &ascii(b"direction"),
        &ascii(b"get direction"),
        &ascii(b"set direction"),
        &ascii(b"direction"),
        Reflection::String,
    ),
    (
        &ascii(b"height"),
        &ascii(b"get height"),
        &ascii(b"set height"),
        &ascii(b"height"),
        Reflection::String,
    ),
    (
        &ascii(b"hspace"),
        &ascii(b"get hspace"),
        &ascii(b"set hspace"),
        &ascii(b"hspace"),
        Reflection::MarqueeUnsigned(HtmlMarqueeUnsignedReflection::Hspace),
    ),
    (
        &ascii(b"loop"),
        &ascii(b"get loop"),
        &ascii(b"set loop"),
        &ascii(b"loop"),
        Reflection::MarqueeLoop,
    ),
    (
        &ascii(b"scrollAmount"),
        &ascii(b"get scrollAmount"),
        &ascii(b"set scrollAmount"),
        &ascii(b"scrollamount"),
        Reflection::MarqueeUnsigned(HtmlMarqueeUnsignedReflection::ScrollAmount),
    ),
    (
        &ascii(b"scrollDelay"),
        &ascii(b"get scrollDelay"),
        &ascii(b"set scrollDelay"),
        &ascii(b"scrolldelay"),
        Reflection::MarqueeUnsigned(HtmlMarqueeUnsignedReflection::ScrollDelay),
    ),
    (
        &ascii(b"trueSpeed"),
        &ascii(b"get trueSpeed"),
        &ascii(b"set trueSpeed"),
        &ascii(b"truespeed"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"vspace"),
        &ascii(b"get vspace"),
        &ascii(b"set vspace"),
        &ascii(b"vspace"),
        Reflection::MarqueeUnsigned(HtmlMarqueeUnsignedReflection::Vspace),
    ),
    (
        &ascii(b"width"),
        &ascii(b"get width"),
        &ascii(b"set width"),
        &ascii(b"width"),
        Reflection::String,
    ),
];

const METHODS: &[(&[u16], bool)] = &[(&ascii(b"start"), true), (&ascii(b"stop"), false)];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Marquee, MEMBERS)?;
    for &(name, running) in METHODS {
        let handler = capture(size_of::<(Capture, bool)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.set_marquee_running(scope, arguments, running)
            }),
            FunctionMetadata { name, length: 0 },
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn set_marquee_running<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        running: bool,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Marquee;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let matches = self.callbacks.with_read(&owner, &events, |read, _| {
                receiver.matches(read.node(id)?, || read.step())
            })?;
            if !matches {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let edit = if running {
                DomSourceEdit::start_marquee(id)
            } else {
                DomSourceEdit::stop_marquee(id)
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}
