//! Progress and Meter bindings over current canonical content attributes.

use super::{reflection::Receiver, *};
use surgeist_bindings_js::conversion::types::DOUBLE;
use surgeist_dom::{source::DomSourceEdit, tree::DomNamedNodeMap};
use surgeist_html::{
    bounded::numeric::{
        HtmlFloatingPointAllocationError, floating_point_number_work_bound,
        floating_point_recovery_token,
    },
    elements::range_indicator::{HtmlMeterRange, HtmlProgressMaximum},
    microsyntax::numeric::HtmlFloatingPointNumber,
    values::Namespace,
};

const VALUE: &[u16] = &ascii(b"value");
const MIN: &[u16] = &ascii(b"min");
const MAX: &[u16] = &ascii(b"max");
const LOW: &[u16] = &ascii(b"low");
const HIGH: &[u16] = &ascii(b"high");
const OPTIMUM: &[u16] = &ascii(b"optimum");

#[derive(Clone, Copy)]
enum ProgressMember {
    Value,
    Max,
    Position,
}

#[derive(Clone, Copy)]
enum MeterMember {
    Value,
    Min,
    Max,
    Low,
    High,
    Optimum,
}

#[derive(Clone, Copy)]
enum Member {
    Progress(ProgressMember),
    Meter(MeterMember),
}

struct Accessor {
    name: &'static [u16],
    getter: &'static [u16],
    setter: Option<&'static [u16]>,
    member: Member,
}

const PROGRESS: &[Accessor] = &[
    Accessor {
        name: VALUE,
        getter: &ascii(b"get value"),
        setter: Some(&ascii(b"set value")),
        member: Member::Progress(ProgressMember::Value),
    },
    Accessor {
        name: MAX,
        getter: &ascii(b"get max"),
        setter: Some(&ascii(b"set max")),
        member: Member::Progress(ProgressMember::Max),
    },
    Accessor {
        name: &ascii(b"position"),
        getter: &ascii(b"get position"),
        setter: None,
        member: Member::Progress(ProgressMember::Position),
    },
];

const METER: &[Accessor] = &[
    Accessor {
        name: VALUE,
        getter: &ascii(b"get value"),
        setter: Some(&ascii(b"set value")),
        member: Member::Meter(MeterMember::Value),
    },
    Accessor {
        name: MIN,
        getter: &ascii(b"get min"),
        setter: Some(&ascii(b"set min")),
        member: Member::Meter(MeterMember::Min),
    },
    Accessor {
        name: MAX,
        getter: &ascii(b"get max"),
        setter: Some(&ascii(b"set max")),
        member: Member::Meter(MeterMember::Max),
    },
    Accessor {
        name: LOW,
        getter: &ascii(b"get low"),
        setter: Some(&ascii(b"set low")),
        member: Member::Meter(MeterMember::Low),
    },
    Accessor {
        name: HIGH,
        getter: &ascii(b"get high"),
        setter: Some(&ascii(b"set high")),
        member: Member::Meter(MeterMember::High),
    },
    Accessor {
        name: OPTIMUM,
        getter: &ascii(b"get optimum"),
        setter: Some(&ascii(b"set optimum")),
        member: Member::Meter(MeterMember::Optimum),
    },
];

pub(super) fn install_progress<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    install(scope, prototype, capture, PROGRESS)
}

pub(super) fn install_meter<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    install(scope, prototype, capture, METER)
}

fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    accessors: &[Accessor],
) -> Result<(), DocumentGuestEventError> {
    for &Accessor {
        name,
        getter,
        setter,
        member,
    } in accessors
    {
        let get = capture(size_of::<(Capture, Member)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                get.read_range_indicator(scope, arguments, member)
            }),
            FunctionMetadata {
                name: getter,
                length: 0,
            },
        )?;
        let setter = match setter {
            Some(setter) => {
                let set = capture(size_of::<(Capture, Member, &'static [u16])>())?;
                Some(scope.fixed_arity_function(
                    &Callback::new(move |scope, arguments| {
                        set.write_range_indicator(scope, arguments, member, name)
                    }),
                    FunctionMetadata {
                        name: setter,
                        length: 1,
                    },
                )?)
            }
            None => None,
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
    Ok(())
}

impl Member {
    fn receiver(self) -> Receiver {
        match self {
            Self::Progress(_) => Receiver::Progress,
            Self::Meter(_) => Receiver::Meter,
        }
    }

    fn read(
        self,
        read: &NativeRead<'_>,
        attributes: DomNamedNodeMap<'_>,
    ) -> Result<f64, DocumentGuestEventError> {
        let attribute = |name| attributes.get_null_namespace_attribute_with(name, || read.step());
        let numeric = |name| {
            attribute(name)?
                .map(|raw| recover_number(read, raw.value()))
                .transpose()
                .map(Option::flatten)
        };
        let value = match self {
            Self::Progress(ProgressMember::Max) => {
                HtmlProgressMaximum::new(numeric(MAX)?).maximum()
            }
            Self::Progress(member) => {
                let Some(value) = attribute(VALUE)? else {
                    return Ok(if matches!(member, ProgressMember::Position) {
                        -1.0
                    } else {
                        0.0
                    });
                };
                let bound = HtmlProgressMaximum::new(numeric(MAX)?);
                let value = recover_number(read, value.value())?;
                if matches!(member, ProgressMember::Position) {
                    bound.determinate_position(value)
                } else {
                    bound.determinate_value(value)
                }
            }
            Self::Meter(member) => {
                let minimum = numeric(MIN)?;
                if matches!(member, MeterMember::Min) {
                    return Ok(HtmlMeterRange::minimum_from(minimum).value());
                }
                let range = HtmlMeterRange::new(minimum, numeric(MAX)?);
                match member {
                    MeterMember::Value => range.value(numeric(VALUE)?),
                    MeterMember::Min => range.minimum(),
                    MeterMember::Max => range.maximum(),
                    MeterMember::Low => range.low(numeric(LOW)?),
                    MeterMember::High => range.high(numeric(LOW)?, numeric(HIGH)?),
                    MeterMember::Optimum => range.optimum(numeric(OPTIMUM)?),
                }
            }
        };
        Ok(value.value())
    }
}

fn recover_number(
    read: &NativeRead<'_>,
    raw: &DocumentString,
) -> Result<Option<HtmlFloatingPointNumber>, DocumentGuestEventError> {
    read.source_step()?;
    let work = floating_point_number_work_bound(raw.len())
        .and_then(|work| u64::try_from(work).ok())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    read.events.budget.charge_work(read.account_scope, work)?;
    let Ok(token) = floating_point_recovery_token(raw.code_units()) else {
        read.source_step()?;
        return Ok(None);
    };
    let bytes = token.required_bytes();
    let _scratch = read.reserve(bytes)?;
    let parsed = token.try_parse(bytes).map_err(|error| match error {
        HtmlFloatingPointAllocationError::RepresentationExhausted => {
            DocumentScriptNativeError::RepresentationExhausted.into()
        }
        HtmlFloatingPointAllocationError::AllocationFailed { .. } => {
            DocumentGuestEventError::from(Error::Allocation)
        }
        _ => DocumentScriptNativeError::StorageLimit.into(),
    })?;
    read.source_step()?;
    Ok(parsed.into_parts().0.ok())
}

impl Capture {
    fn read_range_indicator<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = member.receiver();
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
                let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
                member.read(read, attributes).map(Some)
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(
                scope.primitive(Primitive::Number(value))?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_range_indicator<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: Member,
        attribute: &[u16],
    ) -> Result<Completion<'scope>, Error> {
        let receiver = member.receiver();
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let matches_receiver = || {
                self.callbacks.with_read(&owner, &events, |read, _| {
                    receiver.matches(read.node(id)?, || read.step())
                })
            };
            if !matches_receiver()? {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
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
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.value(&DOUBLE, input))
                .map_err(Error::from)?;
            scope.check_active()?;
            if !matches_receiver()? {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let value = *converted.get();
            if matches!(member, Member::Progress(ProgressMember::Max)) && value <= 0.0 {
                return Ok(Completion::Undefined);
            }
            // The finite primitive's ToString cannot run guest conversion code.
            let primitive = scope.primitive(Primitive::Number(value))?;
            let text = scope.to_utf16(&primitive)?;
            let (value, _value_backing) = copy_native_data(&owner, &events, &text)?;
            let (name, _name_backing) = copy_native_data(&owner, &events, attribute)?;
            self.apply_attribute(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_attribute_ns(id, Namespace::none(), name, value),
                None,
                element::AttributeResult::Value,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
