use super::units;
use surgeist_bindings_js::{
    Error, Execution, ObjectPrototype, Primitive, Process, PropertyKey, Realm, Root,
};

pub(super) fn guest_objects_keep_their_creation_realm(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let (plain, null, array, function_root, a_prototype) = execution.in_realm(&a, |scope| {
        let plain = scope.evaluate(&units("({})"))?;
        let null = scope.evaluate(&units("Object.create(null)"))?;
        let array = scope.evaluate(&units("[]"))?;
        let function = scope.evaluate(&units("(function() {})"))?;
        let prototype = scope.evaluate(&units("Object.prototype"))?;
        Ok((
            scope.retain(&plain),
            scope.retain(&null),
            scope.retain(&array),
            scope.retain(&function),
            scope.retain(&prototype),
        ))
    })?;

    check_associated_realm(&mut execution, &b, &a, &plain, "ordinary object")?;
    check_associated_realm(&mut execution, &b, &a, &null, "null-prototype object")?;
    check_associated_realm(&mut execution, &b, &a, &array, "array")?;
    check_associated_realm(&mut execution, &b, &a, &function_root, "ordinary function")?;

    let (proxy, bound) = execution.in_realm(&b, |scope| {
        let object = scope.import(&plain)?;
        let function = scope.import(&function_root)?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        let replace = scope.evaluate(&units(
            "object => { const proto = {}; Object.setPrototypeOf(object, proto);\
             return Object.getPrototypeOf(object) === proto; }",
        ))?;
        let replaced = scope.call(&replace, &undefined, &[object])?;
        assert!(scope.boolean(&replaced)?);

        let proxy = scope.evaluate(&units(
            "fn => new Proxy(fn, { getPrototypeOf() { throw Error('unexpected trap'); } })",
        ))?;
        let proxy_target = scope.import(&function_root)?;
        let proxy = scope.call(&proxy, &undefined, &[proxy_target])?;
        let bind = scope.evaluate(&units("fn => Function.prototype.bind.call(fn, null)"))?;
        let bound = scope.call(&bind, &undefined, &[function])?;
        Ok((scope.retain(&proxy), scope.retain(&bound)))
    })?;

    check_associated_realm(
        &mut execution,
        &b,
        &a,
        &plain,
        "after prototype replacement",
    )?;
    check_associated_realm(&mut execution, &a, &b, &proxy, "callable Proxy")?;
    check_associated_realm(&mut execution, &a, &b, &bound, "bound function")?;

    execution.in_realm(&b, |scope| {
        let set_null = scope.evaluate(&units("fn => { fn.prototype = null; }"))?;
        let function = scope.import(&function_root)?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        scope.call(&set_null, &undefined, &[function])?;
        let uses_function_realm = scope.evaluate(&units(
            "(ctor, proto) => Object.getPrototypeOf(Reflect.construct(Object, [], ctor)) === proto",
        ))?;
        for (label, wrapper) in [("callable Proxy", &proxy), ("bound function", &bound)] {
            let wrapper = scope.import(wrapper)?;
            let a_prototype = scope.import(&a_prototype)?;
            let uses_a = scope.call(&uses_function_realm, &undefined, &[wrapper, a_prototype])?;
            assert!(
                scope.boolean(&uses_a)?,
                "{label} lost its target's function realm"
            );
        }
        Ok(())
    })
}

fn check_associated_realm(
    execution: &mut Execution<'_>,
    observer: &Realm,
    creator: &Realm,
    original: &Root,
    label: &str,
) -> Result<(), Error> {
    let (creator, creator_global) = execution.in_realm(creator, |scope| {
        let global = scope.global()?;
        Ok((scope.realm_authority()?.identity(), scope.retain(&global)))
    })?;
    execution.in_realm(observer, |scope| {
        let observer = scope.realm_authority()?.identity();
        let observer_global = scope.global()?;
        assert_ne!(creator, observer);

        let original_value = scope.import(original)?;
        let holder = scope.object(ObjectPrototype::Null)?;
        let key_units = [u16::from(b'x')];
        let key = PropertyKey::String(&key_units);
        scope.set(&holder, key, &original_value)?;
        let value = scope.get(&holder, key)?;
        assert!(scope.strict_equals(&value, &original_value)?);

        let callback = scope.callback_object(&value).unwrap_or_else(|error| {
            panic!("{label} creation realm must be observable from the other realm: {error:?}")
        });
        assert_eq!(callback.associated_realm().identity(), creator, "{label}");

        scope.in_callback_object_realm(&callback, |entered, object| {
            assert_eq!(entered.realm_authority()?.identity(), creator, "{label}");
            let global = entered.global()?;
            let expected_global = entered.import(&creator_global)?;
            assert!(entered.strict_equals(&global, &expected_global)?, "{label}");
            let expected = entered.import(original)?;
            assert!(entered.strict_equals(object, &expected)?, "{label}");
            Ok::<(), Error>(())
        })?;

        assert_eq!(scope.realm_authority()?.identity(), observer, "{label}");
        let restored = scope.global()?;
        assert!(scope.strict_equals(&restored, &observer_global)?, "{label}");
        Ok(())
    })
}
