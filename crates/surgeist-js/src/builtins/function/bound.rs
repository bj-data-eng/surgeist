use crate::ContextAccess;
use surgeist_js_gc::heap::Trace as HeapTrace;

use crate::{
    JsObject, JsResult, JsValue,
    object::{
        JsData, ObjectEdge,
        internal_methods::{
            CallValue, InternalMethodCallContext, InternalObjectMethods, ORDINARY_INTERNAL_METHODS,
        },
    },
    value::StoredValue,
};

/// Binds a `Function Object` when `bind` is called.
#[derive(Debug, HeapTrace)]
pub struct BoundFunction {
    target_function: ObjectEdge,
    this: StoredValue,
    args: Vec<StoredValue>,
}

impl JsData for BoundFunction {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        static CONSTRUCTOR_METHODS: InternalObjectMethods = InternalObjectMethods {
            __call__: bound_function_exotic_call,
            __construct__: bound_function_exotic_construct,
            ..ORDINARY_INTERNAL_METHODS
        };

        static FUNCTION_METHODS: InternalObjectMethods = InternalObjectMethods {
            __call__: bound_function_exotic_call,
            ..ORDINARY_INTERNAL_METHODS
        };

        if self.target_function().is_constructor() {
            &CONSTRUCTOR_METHODS
        } else {
            &FUNCTION_METHODS
        }
    }
}

impl BoundFunction {
    /// Abstract operation `BoundFunctionCreate`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boundfunctioncreate
    pub fn create(
        target_function: JsObject,
        this: JsValue,
        args: Vec<JsValue>,
        context: ContextAccess<'_>,
    ) -> JsResult<JsObject> {
        let context = context.into_owner();
        // 1. Let proto be ? targetFunction.[[GetPrototypeOf]]().
        let proto = target_function.__get_prototype_of__(context)?;

        // 2. Let internalSlotsList be the internal slots listed in Table 35, plus [[Prototype]] and [[Extensible]].
        // 3. Let obj be ! MakeBasicObject(internalSlotsList).
        // 4. Set obj.[[Prototype]] to proto.
        // 5. Set obj.[[Call]] as described in 10.4.1.1.
        // 6. If IsConstructor(targetFunction) is true, then
        // a. Set obj.[[Construct]] as described in 10.4.1.2.
        // 7. Set obj.[[BoundTargetFunction]] to targetFunction.
        // 8. Set obj.[[BoundThis]] to boundThis.
        // 9. Set obj.[[BoundArguments]] to boundArgs.
        // 10. Return obj.
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            Self {
                target_function: target_function.edge(),
                this: this.clone().into_stored(),
                args: args.iter().cloned().map(JsValue::into_stored).collect(),
            },
        )
        .upcast())
    }

    /// Returns the bound function's receiver.
    #[must_use]
    pub fn this(&self) -> JsValue {
        self.this.to_value()
    }

    /// Returns the rooted target function.
    #[must_use]
    pub fn target_function(&self) -> JsObject {
        JsObject::from_inner(self.target_function.clone())
    }

    /// Returns the bound arguments as rooted values.
    #[must_use]
    pub fn args(&self) -> Vec<JsValue> {
        self.args.iter().map(StoredValue::to_value).collect()
    }

    pub(crate) fn call_parts(&self) -> (JsObject, JsValue, Vec<JsValue>) {
        (self.target_function(), self.this(), self.args())
    }
}

/// Internal method `[[Call]]` for Bound Function Exotic Objects
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-bound-function-exotic-objects-call-thisargument-argumentslist
#[allow(clippy::unnecessary_wraps)]
fn bound_function_exotic_call(
    obj: &JsObject,
    argument_count: usize,
    context: &mut InternalMethodCallContext<'_>,
) -> JsResult<CallValue> {
    let bound_function = obj
        .downcast_ref::<BoundFunction>()?
        .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?;

    // 1. Let target be F.[[BoundTargetFunction]].
    let (target, bound_this, bound_args) = bound_function.call_parts();
    context
        .vm
        .stack
        .calling_convention_set_function(argument_count, target.clone().into());

    // 2. Let boundThis be F.[[BoundThis]].
    context
        .vm
        .stack
        .calling_convention_set_this(argument_count, bound_this);

    // 3. Let boundArgs be F.[[BoundArguments]].

    // 4. Let args be the list-concatenation of boundArgs and argumentsList.
    context
        .vm
        .stack
        .calling_convention_insert_arguments(argument_count, &bound_args);

    // 5. Return ? Call(target, boundThis, args).
    Ok(target.__call__(bound_args.len() + argument_count))
}

/// Internal method `[[Construct]]` for Bound Function Exotic Objects
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-bound-function-exotic-objects-construct-argumentslist-newtarget
#[allow(clippy::unnecessary_wraps)]
fn bound_function_exotic_construct(
    function_object: &JsObject,
    argument_count: usize,
    context: &mut InternalMethodCallContext<'_>,
) -> JsResult<CallValue> {
    let new_target = context.vm.stack.pop();

    debug_assert!(new_target.is_object(), "new.target should be an object");

    let bound_function = function_object
        .downcast_ref::<BoundFunction>()?
        .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?;

    // 1. Let target be F.[[BoundTargetFunction]].
    let target = bound_function.target_function();

    // 2. Assert: IsConstructor(target) is true.

    // 3. Let boundArgs be F.[[BoundArguments]].
    let bound_args = bound_function.args();

    // 4. Let args be the list-concatenation of boundArgs and argumentsList.
    context
        .vm
        .stack
        .calling_convention_insert_arguments(argument_count, &bound_args);

    // 5. If SameValue(F, newTarget) is true, set newTarget to target.
    let function_object: JsValue = function_object.clone().into();
    let new_target = if JsValue::same_value(&function_object, &new_target) {
        target.clone().into()
    } else {
        new_target
    };

    // 6. Return ? Construct(target, args, newTarget).
    context.vm.stack.push(new_target);
    Ok(target.__construct__(bound_args.len() + argument_count))
}
