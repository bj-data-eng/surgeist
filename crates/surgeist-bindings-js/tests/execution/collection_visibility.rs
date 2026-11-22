use super::units;
use std::cell::Cell;
use surgeist_bindings_js::{
    Callback, Completion, Error, ObjectPrototype, Primitive, Process, ValueKind,
};

struct CollectionValue(&'static str);

pub(super) fn collection_names_follow_prototype_own_descriptors(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let admissions = Cell::new(0);
    execution.in_realm(&realm, |scope| {
        let read = scope.function(&Callback::new(|scope, arguments| {
            let holder = scope
                .native_object::<CollectionValue>(arguments.receiver())?
                .expect("collection reads must use the original native holder");
            let key = arguments.get(0).expect("collection read needs a key");
            let supported =
                scope.kind(key)? == ValueKind::String && scope.to_utf16(key)? == units("supported");
            if arguments.get(1).is_some() {
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Boolean(supported))?,
                ));
            }
            if supported {
                Ok(Completion::Return(
                    scope.string(&units(holder.payload().0))?,
                ))
            } else {
                Ok(Completion::Undefined)
            }
        }))?;
        let keys = scope.function(&Callback::new(|scope, arguments| {
            assert!(
                scope
                    .native_object::<CollectionValue>(arguments.receiver())?
                    .is_some(),
                "collection reflection must use the original native holder"
            );
            let keys = if arguments.is_empty() {
                scope.array(&[])?
            } else {
                let name = scope.string(&units("supported"))?;
                scope.array(&[name])?
            };
            Ok(Completion::Return(keys))
        }))?;
        let prototype = scope.object(ObjectPrototype::Null)?;
        let named_getter = scope.function(&Callback::new(|scope, arguments| {
            let holder = scope
                .native_object::<CollectionValue>(arguments.receiver())?
                .expect("the getter must receive the original native holder");
            let name = arguments.get(0).expect("the getter needs a name");
            assert_eq!(scope.to_utf16(name)?, units("supported"));
            assert_eq!(holder.payload().0, "native");
            Ok(Completion::Return(scope.string(&units("getter"))?))
        }))?;
        let collection = scope.project_native_object_with_collection_properties(
            ObjectPrototype::Object(&prototype),
            CollectionValue("native"),
            &read,
            &keys,
            &named_getter,
            |bytes| {
                assert!(bytes > 0);
                admissions.set(admissions.get() + 1);
                Ok::<_, Error>(())
            },
        )?;
        let check = scope.evaluate(&units(
            r#"((collection, prototype) => {
                const own = () => Object.getOwnPropertyDescriptor(collection, "supported");
                const ownKeys = () => Reflect.ownKeys(collection);
                if (collection.supported !== "native" || own()?.value !== "native"
                    || !ownKeys().includes("supported")) return "unmasked name";

                prototype.supported = "ordinary";
                if (collection.supported !== "ordinary" || own() !== undefined
                    || ownKeys().includes("supported")) return "ordinary inherited name";
                delete prototype.supported;

                Object.setPrototypeOf(prototype, new Proxy({}, {}));
                if (own()?.value !== "getter") return "live native getter after prototype lookup";

                const target = {supported: "inherited"};
                const proxy = new Proxy(target, {
                    getOwnPropertyDescriptor(object, key) {
                        return Reflect.getOwnPropertyDescriptor(object, key);
                    },
                });
                Object.setPrototypeOf(prototype, proxy);
                if (collection.supported !== "inherited") return "proxy inherited value";
                if (own() !== undefined) return "masked own descriptor";
                if (ownKeys().includes("supported")) return "masked own key";

                const sentinel = {};
                const throwing = new Proxy(target, {
                    getOwnPropertyDescriptor(object, key) {
                        if (key === "supported" || key === "unsupported") throw sentinel;
                        return Reflect.getOwnPropertyDescriptor(object, key);
                    },
                });
                Object.setPrototypeOf(prototype, throwing);
                try {
                    if (Object.getOwnPropertyDescriptor(collection, "unsupported") !== undefined) {
                        return "unsupported name was visible";
                    }
                } catch (_) {
                    return "unsupported name entered a prototype descriptor trap";
                }
                for (const [label, operation] of [
                    ["get", () => collection.supported],
                    ["descriptor", own],
                    ["own keys", ownKeys],
                ]) {
                    try {
                        operation();
                        return label + " did not throw";
                    } catch (value) {
                        if (value !== sentinel) return label + " changed the thrown value";
                    }
                }
                Object.setPrototypeOf(prototype, new Proxy(collection, {}));
                if (own() !== undefined || ownKeys().includes("supported")) {
                    return "proxy-mediated cycle exposed a name";
                }
                return "ok";
            })"#,
        ))?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        let result = scope.call(&check, &undefined, &[collection, prototype])?;
        assert_eq!(scope.to_utf16(&result)?, units("ok"));
        Ok(())
    })?;
    assert_eq!(admissions.get(), 1, "one native collection was admitted");
    Ok(())
}
