use super::units;
use std::{cell::Cell, rc::Rc};
use surgeist_bindings_js::{
    Callback, Completion, Error, NativeNamedPropertyCallbacks, NativeProjectionPrototype,
    ObjectPrototype, Primitive, Process, PropertyKey,
};

pub(super) fn retained_named_properties_stay_on_the_canonical_object(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let other = execution.create_realm()?;
    let foreign = execution.in_realm(&other, |scope| {
        let function = scope.evaluate(&units("() => undefined"))?;
        Ok::<_, Error>(scope.retain(&function))
    })?;

    execution.in_realm(&realm, |scope| {
        let read = scope.function(&Callback::new(|scope, args| {
            let payload = scope
                .native_object::<u32>(args.receiver())?
                .ok_or(Error::ExpectedNativeObject)?;
            let key = args.get(0).ok_or(Error::BackendUnavailable)?;
            let selected = scope.to_utf16(key)? == units("selected");
            if args.get(1).is_some() {
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Boolean(selected))?,
                ));
            }
            if selected {
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Number(f64::from(*payload.payload())))?,
                ));
            }
            Ok(Completion::Undefined)
        }))?;
        let keys = scope.function(&Callback::new(|scope, _| {
            let key = scope.string(&units("selected"))?;
            Ok(Completion::Return(scope.array(&[key])?))
        }))?;
        let wrong = scope.import(&foreign)?;
        let attempted = Cell::new(false);
        let refused = scope.try_project_retained_native_named_object(
            NativeProjectionPrototype::Current(ObjectPrototype::Default),
            31_u32,
            NativeNamedPropertyCallbacks {
                read: &wrong,
                keys: &keys,
                unforgeable_names: &[],
            },
            |_, _| {
                attempted.set(true);
                Ok::<_, Error>(((), ()))
            },
            |_| Ok(()),
        );
        assert!(matches!(refused, Err(Error::CallbackRealmUnavailable)));
        assert!(!attempted.get());

        let backing = Rc::new(());
        let weak = Rc::downgrade(&backing);
        let refused = scope.try_project_retained_native_named_object(
            NativeProjectionPrototype::Current(ObjectPrototype::Default),
            31_u32,
            NativeNamedPropertyCallbacks {
                read: &read,
                keys: &keys,
                unforgeable_names: &[],
            },
            |_, _| Ok((backing.clone(), backing)),
            |_| Err::<(), _>(Error::PropertyRejected),
        );
        assert!(matches!(refused, Err(Error::PropertyRejected)));
        assert!(weak.upgrade().is_none());

        let (wrapper, holder, ()) = scope.try_project_retained_native_named_object(
            NativeProjectionPrototype::Current(ObjectPrototype::Default),
            42_u32,
            NativeNamedPropertyCallbacks {
                read: &read,
                keys: &keys,
                unforgeable_names: &[],
            },
            |_, _| Ok::<_, Error>(((), ())),
            |_| Ok(()),
        )?;
        let reopened = scope.open_callback_object(&holder)?;
        assert!(scope.strict_equals(&wrapper, &reopened)?);
        let selected = scope.get(&wrapper, PropertyKey::String(&units("selected")))?;
        assert_eq!(scope.to_utf16(&selected)?, units("42"));
        Ok(())
    })
}
