//! Test for the class proc-macro.
#![allow(unused_crate_dependencies)]

use surgeist_js::gc::heap::Trace as HeapTrace;
use surgeist_js::{
    Context, ContextAccess, JsData, JsObject, JsResult, JsString, JsValue, Source, boa_class,
    js_string,
};

#[derive(Clone, JsData, HeapTrace)]
enum AnimalType {
    Cat,
    Dog,
    Other,
}

#[derive(Clone, JsData, HeapTrace)]
struct Animal {
    ty: AnimalType,
    age: i32,
}

#[boa_class]
impl Animal {
    #[boa(constructor)]
    #[allow(clippy::needless_pass_by_value)]
    fn new(name: String, age: i32) -> Self {
        let ty = match name.as_str() {
            "cat" => AnimalType::Cat,
            "dog" => AnimalType::Dog,
            _ => AnimalType::Other,
        };

        Self { ty, age }
    }

    #[boa(static)]
    fn marked_static_method() -> i32 {
        123
    }

    fn static_method() -> i32 {
        42
    }

    // Force this being a method (instead of a static function) by declaring it
    // as a method.
    #[boa(method)]
    #[boa(length = 11)]
    fn method(mut context: ContextAccess<'_>) -> JsObject {
        let obj = JsObject::with_null_proto(context.runtime()).unwrap();
        obj.set(js_string!("key"), 43, false, context.reborrow())
            .unwrap();
        obj
    }

    #[boa(getter)]
    fn age(&self) -> i32 {
        self.age
    }

    #[boa(setter)]
    #[boa(method)]
    #[boa(rename = "age")]
    fn set_age(&mut self, age: i32) {
        self.age = age;
    }

    fn speak(#[boa(error = "`this` was not an animal")] &self) -> JsString {
        match self.ty {
            AnimalType::Cat => js_string!("meow"),
            AnimalType::Dog => js_string!("woof"),
            AnimalType::Other => js_string!(r"¯\_(ツ)_/¯"),
        }
    }
}

#[derive(Clone, Default, JsData, HeapTrace)]
struct Pair {
    value: i32,
}

#[boa_class]
impl Pair {
    #[boa(constructor)]
    fn new(value: i32) -> Self {
        Self { value }
    }

    #[boa(symbol = "toPrimitive")]
    fn to_primitive(&self) -> i32 {
        self.value
    }

    #[boa(symbol = "iterator")]
    fn iterator(&self, mut context: ContextAccess<'_>) -> JsResult<JsValue> {
        use surgeist_js::object::builtins::JsArray;
        let arr = JsArray::from_iter(
            [JsValue::from(self.value), JsValue::from(self.value * 10)],
            context.reborrow(),
        )?;
        let iter_fn = arr.get(surgeist_js::JsSymbol::iterator(), context.reborrow())?;
        iter_fn
            .as_callable()
            .expect("@@iterator should be callable")
            .call(&arr.into(), &[], context.reborrow())
    }
}

const ASSERT_DECL: &str = r"
    function assertEq(lhs, rhs, message) {
      if (lhs !== rhs) {
        throw `AssertionError: ${message ? message + ',' : ''} expected ${JSON.stringify(rhs)}, actual ${JSON.stringify(lhs)}`;
      }
    }
";

#[test]
fn boa_class() {
    let mut context = Context::builder().build().unwrap();

    context.register_global_class::<Animal>().unwrap();

    context
        .eval(Source::from_bytes(ASSERT_DECL))
        .expect("Unreachable.");

    context
        .eval(Source::from_bytes(
            r#"
            let pet = new Animal("dog", 3);
            assertEq(pet.age, 3, "Age should be the age passed to constructor");

            assertEq(Animal.staticMethod(), 42, "Static method");
            assertEq(Animal.markedStaticMethod(), 123, "Marked static method");

            v = pet.method();
            assertEq(v.key, 43, "Method returned");

            pet.age = 4;
            assertEq(pet.age, 4, "Pet setter");

            pet.setAge(5);
            assertEq(pet.age, 5, "Pet.setAge");

            assertEq(Animal.prototype.method.length, 11, "Method.length");
            assertEq(Animal.prototype.speak.length, 0, "speak.length");
            assertEq(Animal.prototype.setAge.length, 1, "setAge.length");
     "#,
        ))
        .expect("Could not evaluate script");

    let pet = context
        .eval(Source::from_bytes("pet"))
        .unwrap()
        .as_object()
        .unwrap();
    let borrow = pet.borrow().unwrap();
    let age = context.eval(Source::from_bytes("pet.age")).unwrap();
    drop(borrow);
    assert_eq!(age, JsValue::from(5));
}

#[test]
fn generated_shared_getter_refuses_exclusive_alias_and_retries_same_receiver() {
    use surgeist_js::error::{EngineError, ObjectAccessError};

    let mut context = Context::builder().build().unwrap();
    context.register_global_class::<Animal>().unwrap();
    let pet = context
        .eval(Source::from_bytes("new Animal('dog', 3)"))
        .unwrap();
    let object = pet.as_object().unwrap();
    let alias = object.clone();
    let getter = context
        .eval(Source::from_bytes(
            "Object.getOwnPropertyDescriptor(Animal.prototype, 'age').get",
        ))
        .unwrap()
        .as_callable()
        .unwrap();

    let guard = alias.borrow_mut().unwrap();
    let error = getter.call(&pet, &[], context.access()).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
    );
    drop(guard);

    assert_eq!(object.downcast_ref::<Animal>().unwrap().unwrap().age, 3);
    assert_eq!(
        getter.call(&pet, &[], context.access()).unwrap(),
        JsValue::new(3)
    );
}

#[test]
fn generated_mutable_method_refuses_aliases_before_write_and_retries_same_receiver() {
    use surgeist_js::error::{EngineError, ObjectAccessError};

    for exclusive in [false, true] {
        let mut context = Context::builder().build().unwrap();
        context.register_global_class::<Animal>().unwrap();
        let pet = context
            .eval(Source::from_bytes("new Animal('dog', 3)"))
            .unwrap();
        let object = pet.as_object().unwrap();
        let alias = object.clone();
        let method = context
            .eval(Source::from_bytes("Animal.prototype.setAge"))
            .unwrap()
            .as_callable()
            .unwrap();
        let args = [JsValue::new(7)];

        let error = if exclusive {
            let guard = alias.borrow_mut().unwrap();
            let error = method.call(&pet, &args, context.access()).unwrap_err();
            drop(guard);
            error
        } else {
            let guard = alias.borrow().unwrap();
            let error = method.call(&pet, &args, context.access()).unwrap_err();
            drop(guard);
            error
        };
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
        );
        assert_eq!(object.downcast_ref::<Animal>().unwrap().unwrap().age, 3);

        assert_eq!(
            method.call(&pet, &args, context.access()).unwrap(),
            JsValue::null()
        );
        assert_eq!(object.downcast_ref::<Animal>().unwrap().unwrap().age, 7);
    }
}

#[test]
fn boa_class_symbol_methods() {
    let mut context = Context::builder().build().unwrap();

    context.register_global_class::<Pair>().unwrap();

    context
        .eval(Source::from_bytes(ASSERT_DECL))
        .expect("Unreachable.");

    context
        .eval(Source::from_bytes(
            r#"
            let p = new Pair(7);

            // Symbol.toPrimitive should be defined on the prototype.
            assertEq(typeof p[Symbol.toPrimitive], "function", "@@toPrimitive is a function");
            assertEq(+p, 7, "toPrimitive numeric coercion");
            assertEq(p + 3, 10, "toPrimitive addition");

            // Symbol.iterator should be defined on the prototype.
            assertEq(typeof p[Symbol.iterator], "function", "@@iterator is a function");

            // Spread should work via [Symbol.iterator].
            let spread = [...p];
            assertEq(spread.length, 2, "spread length");
            assertEq(spread[0], 7, "spread[0]");
            assertEq(spread[1], 70, "spread[1]");

            // for-of should also work.
            let sum = 0;
            for (let v of p) { sum += v; }
            assertEq(sum, 77, "for-of sum");
     "#,
        ))
        .expect("Could not evaluate script");
}
