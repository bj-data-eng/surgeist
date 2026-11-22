use super::units;
use std::rc::Rc;
use surgeist_bindings_js::{
    Error, Execution, ObjectPrototype, Primitive, Process, PropertyKey, Realm, ValueKind,
};

pub(super) fn native_references_follow_only_guest_reachability() -> Result<(), Error> {
    const SLOT: u32 = 3;
    let mut process = Process::bootstrap()?;
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    initial_reference_precedes_publication(&mut execution, &realm)?;
    let payload = Rc::new(17_u32);
    let funding = Rc::new(());
    let observed_payload = Rc::downgrade(&payload);
    let observed_funding = Rc::downgrade(&funding);
    let key = units("traceNative");

    let token = execution.in_realm(&realm, |scope| {
        let wrapper = scope.project_native_object(ObjectPrototype::Default, payload, |_| {
            Ok::<_, Error>(funding)
        })?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&key), &wrapper)?;
        let closure = scope.evaluate(&units(
            "(() => { const wrapper = Object.freeze(traceNative); return () => wrapper; })()",
        ))?;
        assert!(
            scope
                .native_object_reference::<Rc<u32>>(&wrapper, SLOT)?
                .is_none()
        );
        scope.set_native_object_reference::<Rc<u32>>(&wrapper, SLOT, Some(&closure))?;
        let stored = scope
            .native_object_reference::<Rc<u32>>(&wrapper, SLOT)?
            .expect("the frozen wrapper accepts its private reference");
        assert!(scope.strict_equals(&stored, &closure)?);

        let proxy = scope.evaluate(&units("new Proxy(traceNative, {})"))?;
        assert!(matches!(
            scope.native_object_reference::<Rc<u32>>(&proxy, SLOT),
            Err(Error::ExpectedNativeObject)
        ));
        assert!(matches!(
            scope.native_object_reference::<u32>(&wrapper, SLOT),
            Err(Error::ExpectedNativeObject)
        ));
        let public_slot = scope.evaluate(&units("traceNative[3]"))?;
        assert_eq!(scope.kind(&public_slot)?, ValueKind::Undefined);
        let frozen = scope.evaluate(&units(
            "Object.isFrozen(traceNative) && !Object.hasOwn(traceNative, '3')",
        ))?;
        assert!(scope.boolean(&frozen)?);

        scope.set_native_object_reference::<Rc<u32>>(&wrapper, SLOT, None)?;
        assert!(
            scope
                .native_object_reference::<Rc<u32>>(&wrapper, SLOT)?
                .is_none()
        );
        scope.set_native_object_reference::<Rc<u32>>(&wrapper, SLOT, Some(&closure))?;
        let token = scope
            .native_object_token::<Rc<u32>>(&wrapper)?
            .expect("the exact projection has a weak token");
        let reopened = scope
            .open_native_object(&token)?
            .expect("the live projection can be reopened");
        assert!(scope.strict_equals(&wrapper, &reopened)?);
        Ok::<_, Error>(token)
    })?;

    execution.memory_pressure();
    let lease = execution.in_realm(&realm, |scope| {
        let global = scope.global()?;
        let wrapper = scope.get(&global, PropertyKey::String(&key))?;
        let closure = scope
            .native_object_reference::<Rc<u32>>(&wrapper, SLOT)?
            .expect("the guest reference keeps the traced closure alive");
        let returned = scope.call(&closure, &wrapper, &[])?;
        assert!(scope.strict_equals(&returned, &wrapper)?);
        let lease = scope
            .native_object::<Rc<u32>>(&wrapper)?
            .expect("the live wrapper carries its native payload");
        assert_eq!(**lease.payload(), 17);
        Ok::<_, Error>(lease)
    })?;

    execution.in_realm(&realm, |scope| {
        let deleted = scope.evaluate(&units("delete globalThis.traceNative"))?;
        assert!(scope.boolean(&deleted)?);
        Ok::<_, Error>(())
    })?;
    execution.memory_pressure();
    execution.in_realm(&realm, |scope| {
        assert!(scope.open_native_object(&token)?.is_none());
        Ok::<_, Error>(())
    })?;
    assert!(observed_payload.upgrade().is_some());
    assert!(observed_funding.upgrade().is_some());
    drop(lease);
    assert!(observed_payload.upgrade().is_none());
    assert!(observed_funding.upgrade().is_none());
    Ok(())
}

fn initial_reference_precedes_publication(
    execution: &mut Execution<'_>,
    realm: &Realm,
) -> Result<(), Error> {
    let funding = Rc::new(());
    let refused_funding = Rc::downgrade(&funding);
    let token = execution.in_realm(realm, |scope| {
        let invalid = scope.primitive(Primitive::Null)?;
        let refused = scope.project_native_object_with_reference(
            ObjectPrototype::Default,
            17_u32,
            1,
            &invalid,
            |_| Ok::<_, Error>(funding),
        );
        assert!(matches!(refused, Err(Error::ExpectedObject)));
        assert!(refused_funding.upgrade().is_none());

        let source = scope.object(ObjectPrototype::Null)?;
        let projected = scope.project_native_object_with_reference(
            ObjectPrototype::Default,
            19_u32,
            1,
            &source,
            |_| Ok::<_, Error>(()),
        )?;
        let stored = scope
            .native_object_reference::<u32>(&projected, 1)?
            .unwrap();
        assert!(scope.strict_equals(&stored, &source)?);
        Ok(scope.native_object_token::<u32>(&projected)?.unwrap())
    })?;
    execution.memory_pressure();
    execution.in_realm(realm, |scope| {
        assert!(scope.open_native_object(&token)?.is_none());
        Ok(())
    })
}
