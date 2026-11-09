//! Deferred arguments retain Rust-owned recipes through callable resolution and entry gates.
use crate::{
    Context, JsNativeError, JsResult, JsValue,
    builtins::{
        Proxy,
        array::Array,
        function::{BoundFunction, OrdinaryFunction, reserve_prepared_call_entry},
    },
    object::{
        JsObject,
        internal_methods::{
            CallEntryPermit, HostEntryPermit, InternalMethodPropertyContext,
            is_compatible_property_descriptor, validate_and_apply_property_descriptor,
        },
    },
    property::{PropertyDescriptor, PropertyKey},
};

/// A fresh callable recipe. The closure commits/activates preallocated state
/// without admission, program callbacks, or allocation after publication.
pub(crate) type PreparedFunction<'a> = Box<dyn FnOnce(&mut Context) -> JsResult<JsObject> + 'a>;

/// Supplies the deferred data value while preserving absent attribute fields.
/// A value-only recipe starts generic; descriptor filling cannot change its kind.
pub(crate) fn function_value_descriptor(
    descriptor: &PropertyDescriptor,
    value: JsValue,
) -> PropertyDescriptor {
    debug_assert!(!descriptor.is_accessor_descriptor());
    PropertyDescriptor::builder()
        .value(value)
        .maybe_writable(descriptor.writable())
        .maybe_enumerable(descriptor.enumerable())
        .maybe_configurable(descriptor.configurable())
        .build()
}

type ArgumentMaterializer<'a> = Box<dyn FnOnce(&mut [JsValue], &mut Context) -> JsResult<()> + 'a>;

pub(crate) struct PreparedArguments<'a> {
    values: Vec<JsValue>,
    materialize: ArgumentMaterializer<'a>,
}

impl<'a> PreparedArguments<'a> {
    pub(crate) fn new(
        values: Vec<JsValue>,
        materialize: impl FnOnce(&mut [JsValue], &mut Context) -> JsResult<()> + 'a,
    ) -> Self {
        Self {
            values,
            materialize: Box::new(materialize),
        }
    }
}

enum ArgumentTransform {
    Bound {
        values: Vec<JsValue>,
        prefix: usize,
    },
    Proxy {
        array: JsObject,
        values: Vec<JsValue>,
    },
}

impl JsObject {
    pub(crate) fn define_declaration_property(
        &self,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
        context: &mut Context,
    ) -> JsResult<()> {
        let success = self.__define_own_property__(
            &key,
            descriptor,
            &mut InternalMethodPropertyContext::tracking_effects(context),
        )?;
        if !success {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot redefine property: {key}"))
                .into());
        }
        Ok(())
    }

    /// Defines a source-declared function while keeping its fresh value private
    /// until the real property mutation or ultimate trap entry is permitted.
    pub(crate) fn define_prepared_function(
        &self,
        key: &PropertyKey,
        mut descriptor: PropertyDescriptor,
        materialize: PreparedFunction<'_>,
        context: &mut Context,
    ) -> JsResult<Option<JsObject>> {
        self.check_runtime(context.runtime())?;
        // This entry is for syntactic function identifiers after CanDeclare.
        // These keys would select different exotic algorithms and cannot be
        // admitted through the identifier-only declaration protocol.
        if matches!(key, PropertyKey::Index(_))
            || self.is::<Array>() && matches!(key, PropertyKey::String(name) if name == "length")
        {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        if self.is::<Proxy>() {
            return crate::builtins::proxy::proxy_define_prepared_function(
                self,
                key,
                descriptor,
                materialize,
                context,
            );
        }
        // Namespace definitions with a fresh object cannot satisfy SameValue.
        // Retain its repeated GetOwnProperty first, including an uninitialized
        // export's error, rather than materializing an impossible definition.
        if self.is::<crate::module::ModuleNamespace>() && !matches!(key, PropertyKey::Symbol(_)) {
            self.__get_own_property__(key, &mut InternalMethodPropertyContext::new(context))?;
            return Ok(None);
        }
        // The only canonical numeric strings which are IdentifierNames are
        // NaN/Infinity. Integer-indexed definitions reject these before reading
        // the value. Run that real exotic precheck with the absent value.
        if self.is::<crate::builtins::typed_array::TypedArray>()
            && matches!(key, PropertyKey::String(name) if name == "NaN" || name == "Infinity")
        {
            let accepted = self.__define_own_property__(
                key,
                descriptor,
                &mut InternalMethodPropertyContext::new(context),
            )?;
            debug_assert!(!accepted, "NaN/Infinity cannot be valid integer indices");
            return Ok(None);
        }
        let ordinary_branch = core::ptr::fn_addr_eq(
            self.vtable().__define_own_property__,
            crate::object::internal_methods::ORDINARY_INTERNAL_METHODS.__define_own_property__,
        ) || self.is::<Array>()
            || self.is::<crate::JsString>()
            || self.is::<crate::builtins::function::arguments::MappedArguments>()
            || self.is::<crate::builtins::typed_array::TypedArray>()
            || self.is::<crate::module::ModuleNamespace>();
        if !ordinary_branch {
            // An embedding-defined internal operation is itself a host callback.
            // Preserve its real dispatch rather than treating it as ordinary.
            context.check_runtime_limits()?;
            let function = materialize(context)?;
            descriptor = function_value_descriptor(&descriptor, function.clone().into());
            context.note_execution_effect();
            return self
                .__define_own_property__(
                    key,
                    descriptor,
                    &mut InternalMethodPropertyContext::new(context),
                )
                .map(|accepted| accepted.then_some(function));
        }
        // Source function names are identifiers. String/arguments virtual
        // indices are therefore absent; array length is rejected by the
        // required CanDeclareGlobalFunction preflight. Their remaining branches
        // are precisely OrdinaryDefineOwnProperty, using their real own lookup.
        let mut property = InternalMethodPropertyContext::new(context);
        let current = self.__get_own_property__(key, &mut property)?;
        let extensible = self.__is_extensible__(&mut property)?;
        // A fresh callable is a new data value. It cannot match any existing
        // non-writable value or turn a non-configurable accessor into data.
        if current.as_ref().is_some_and(|current| {
            !current.expect_configurable()
                && (current.is_accessor_descriptor() || !current.expect_writable())
        }) || !is_compatible_property_descriptor(extensible, descriptor.clone(), current.clone())
        {
            return Ok(None);
        }
        property.check_runtime_limits()?;
        let function = materialize(&mut property)?;
        descriptor = function_value_descriptor(&descriptor, function.clone().into());
        property.note_execution_effect();
        let accepted = validate_and_apply_property_descriptor(
            Some((self, key)),
            extensible,
            descriptor,
            current,
            property.slot(),
        )?;
        debug_assert!(
            accepted,
            "fresh-function restrictions were checked before activation"
        );
        Ok(Some(function))
    }

    /// Resolves bound/proxy semantics before permitting the actual callback target.
    /// The argument recipe must finish its private work before exposing any value.
    pub(crate) fn call_prepared<'a>(
        &self,
        mut this: JsValue,
        arguments: PreparedArguments<'a>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let mut target = self.clone();
        let mut argument_count = arguments.values.len();
        let mut transforms = Vec::new();
        loop {
            target.check_runtime(context.runtime())?;
            context.runtime().check_value(&this)?;
            let bound_parts = target
                .downcast_ref::<BoundFunction>()?
                .map(|bound| bound.call_parts());
            if let Some((next, bound_this, mut values)) = bound_parts {
                let prefix = values.len();
                values.resize(prefix + argument_count, JsValue::undefined());
                argument_count = values.len();
                transforms.push(ArgumentTransform::Bound { values, prefix });
                target = next;
                this = bound_this;
                continue;
            }
            if target.is::<Proxy>() {
                let (proxy_target, handler, trap) = Proxy::resolve_call(&target, context)?;
                if let Some(trap) = trap {
                    let array = Array::create_array_from_list(
                        std::iter::repeat_with(JsValue::undefined).take(argument_count),
                        context,
                    );
                    let values = vec![proxy_target.into(), this, array.clone().into()];
                    transforms.push(ArgumentTransform::Proxy { array, values });
                    argument_count = 3;
                    this = handler.into();
                    target = trap;
                } else {
                    target = proxy_target;
                }
                continue;
            }
            if !target.is_callable() {
                return Err(JsNativeError::typ()
                    .with_message("value is not callable")
                    .into());
            }
            let permit = if target.is::<OrdinaryFunction>() {
                CallEntryPermit::Guest(reserve_prepared_call_entry(
                    &target,
                    argument_count,
                    context,
                )?)
            } else {
                CallEntryPermit::Host(HostEntryPermit::reserve(target.clone(), context)?)
            };
            let PreparedArguments {
                mut values,
                materialize,
            } = arguments;
            materialize(&mut values, context)?;
            for transform in transforms {
                match transform {
                    ArgumentTransform::Bound {
                        values: mut output,
                        prefix,
                    } => {
                        for (slot, value) in output[prefix..].iter_mut().zip(values) {
                            *slot = value;
                        }
                        values = output;
                    }
                    ArgumentTransform::Proxy {
                        array,
                        values: output,
                    } => {
                        if !values.is_empty() {
                            let mut array = array.borrow_mut()?;
                            let slots = array
                                .properties_mut()
                                .dense_indexed_properties_mut()
                                .expect("private argument array retains preallocated dense slots");
                            for (slot, value) in slots.iter_mut().zip(values) {
                                *slot = value.into_stored();
                            }
                        }
                        values = output;
                    }
                }
            }
            return target.call_with_entry(&this, &values, context, Some(permit));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Source,
        execution::{Execution, ExecutionOptions},
        js_string,
        native_function::NativeFunction,
    };
    use std::cell::Cell;

    fn native_answer(context: &mut Context) -> JsObject {
        NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::new(42)))
            .to_js_function(context.realm())
            .unwrap()
            .into()
    }

    #[test]
    fn immutable_property_rejection_keeps_the_function_recipe_unmaterialized() {
        let context = &mut Context::default();
        let object = JsObject::with_object_proto(context.intrinsics());
        object
            .define_property_or_throw(
                js_string!("kept"),
                PropertyDescriptor::builder()
                    .value(7)
                    .writable(false)
                    .configurable(false),
                context.access(),
            )
            .unwrap();
        let materialized = Cell::new(false);
        let answer = native_answer(context);
        let result = object
            .define_prepared_function(
                &js_string!("kept").into(),
                PropertyDescriptor::builder().build(),
                Box::new(|_| {
                    materialized.set(true);
                    Ok(answer)
                }),
                context,
            )
            .unwrap();
        assert!(result.is_none());
        assert!(!materialized.get());
        assert_eq!(
            object.get(js_string!("kept"), context.access()).unwrap(),
            JsValue::new(7)
        );
    }

    #[test]
    fn ordinary_definition_exposes_the_materialized_callable_once() {
        let context = &mut Context::default();
        let object = JsObject::with_object_proto(context.intrinsics());
        let calls = Cell::new(0);
        let answer = native_answer(context);
        let function = object
            .define_prepared_function(
                &js_string!("answer").into(),
                PropertyDescriptor::builder()
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                Box::new(|_| {
                    calls.set(calls.get() + 1);
                    Ok(answer)
                }),
                context,
            )
            .unwrap()
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(
            object
                .get(js_string!("answer"), context.access())
                .unwrap()
                .as_object(),
            Some(function.clone())
        );
        assert_eq!(
            function
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap(),
            JsValue::new(42)
        );
    }

    #[test]
    fn value_only_function_replacement_preserves_existing_property_attributes() {
        let context = &mut Context::default();
        let object = JsObject::with_object_proto(context.intrinsics());
        let old_function = native_answer(context);
        object
            .define_property_or_throw(
                js_string!("answer"),
                PropertyDescriptor::builder()
                    .value(old_function.clone())
                    .writable(true)
                    .enumerable(true)
                    .configurable(false),
                context.access(),
            )
            .unwrap();
        let replacement = native_answer(context);
        let function = object
            .define_prepared_function(
                &js_string!("answer").into(),
                PropertyDescriptor::builder().build(),
                Box::new(|_| Ok(replacement)),
                context,
            )
            .unwrap()
            .unwrap();
        assert!(!JsObject::equals(&function, &old_function));
        let descriptor = object
            .__get_own_property__(
                &js_string!("answer").into(),
                &mut InternalMethodPropertyContext::new(context),
            )
            .unwrap()
            .unwrap();
        assert_eq!(descriptor.value(), Some(&function.clone().into()));
        assert_eq!(descriptor.writable(), Some(true));
        assert_eq!(descriptor.enumerable(), Some(true));
        assert_eq!(descriptor.configurable(), Some(false));
        assert_eq!(
            function
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap(),
            JsValue::new(42)
        );
    }

    #[test]
    fn guest_zero_instruction_refusal_happens_before_argument_materialization() {
        let context = &mut Context::default();
        let function = context
            .eval(Source::from_bytes("(function(value) { return value; })"))
            .unwrap()
            .as_object()
            .unwrap();
        context.execution = Some(Execution::new(ExecutionOptions::new(0)).unwrap());
        let materialized = Cell::new(false);
        let result = function.call_prepared(
            JsValue::undefined(),
            PreparedArguments::new(vec![JsValue::undefined()], |values, _| {
                materialized.set(true);
                values[0] = JsValue::new(42);
                Ok(())
            }),
            context,
        );
        assert_eq!(
            result.unwrap_err().as_engine(),
            Some(&crate::error::EngineError::RuntimeLimit(
                crate::error::RuntimeLimitError::Instruction { limit: 0 },
            ))
        );
        assert!(!materialized.get());
        context.execution = None;
    }

    #[test]
    fn host_runtime_refusal_happens_before_argument_materialization() {
        let context = &mut Context::default();
        let function = native_answer(context);
        context.runtime_limits_mut().set_recursion_limit(0);
        let materialized = Cell::new(false);
        let result = function.call_prepared(
            JsValue::undefined(),
            PreparedArguments::new(vec![], |_, _| {
                materialized.set(true);
                Ok(())
            }),
            context,
        );
        assert_eq!(
            result.unwrap_err().as_engine(),
            Some(&crate::error::EngineError::RuntimeLimit(
                crate::error::RuntimeLimitError::Recursion,
            ))
        );
        assert!(!materialized.get());
    }

    #[test]
    fn prospective_guest_stack_refusal_happens_before_argument_materialization() {
        let context = &mut Context::default();
        let function = context
            .eval(Source::from_bytes("(function(value) { return value; })"))
            .unwrap()
            .as_object()
            .unwrap();
        // The completed evaluation has restored the empty host stack. One
        // argument plus the call's this/function prologue reaches this limit.
        context.runtime_limits_mut().set_stack_size_limit(3);
        let materialized = Cell::new(false);
        let result = function.call_prepared(
            JsValue::undefined(),
            PreparedArguments::new(vec![JsValue::undefined()], |values, _| {
                materialized.set(true);
                values[0] = JsValue::new(42);
                Ok(())
            }),
            context,
        );
        assert_eq!(
            result.unwrap_err().as_engine(),
            Some(&crate::error::EngineError::RuntimeLimit(
                crate::error::RuntimeLimitError::StackSize,
            ))
        );
        assert!(!materialized.get());
    }

    #[test]
    fn only_the_actual_declaration_write_marks_an_execution_effect() {
        let context = &mut Context::default();
        let object = JsObject::with_object_proto(context.intrinsics());
        object
            .define_property_or_throw(
                js_string!("kept"),
                PropertyDescriptor::builder()
                    .value(7)
                    .writable(false)
                    .configurable(false),
                context.access(),
            )
            .unwrap();
        context.execution = Some(Execution::new(ExecutionOptions::new(64)).unwrap());
        crate::builtins::object::OrdinaryObject::from_property_descriptor(
            Some(
                PropertyDescriptor::builder()
                    .value(JsValue::undefined())
                    .build(),
            ),
            context,
        )
        .unwrap();
        object
            .define_declaration_property(
                js_string!("kept").into(),
                PropertyDescriptor::builder().value(9).build(),
                context,
            )
            .unwrap_err();
        let no_effect = context
            .execution
            .take()
            .unwrap()
            .finish(Ok(JsValue::undefined()));
        assert!(!no_effect.report.effects_started());

        context.execution = Some(Execution::new(ExecutionOptions::new(64)).unwrap());
        object
            .define_declaration_property(
                js_string!("declared").into(),
                PropertyDescriptor::builder()
                    .value(JsValue::undefined())
                    .writable(true)
                    .build(),
                context,
            )
            .unwrap();
        let effect = context
            .execution
            .take()
            .unwrap()
            .finish(Ok(JsValue::undefined()));
        assert!(effect.report.effects_started());
    }

    #[test]
    fn bound_proxy_resolution_preserves_get_method_and_argument_order() {
        let context = &mut Context::default();
        let function = context
            .eval(Source::from_bytes(
                r#"
            globalThis.preparedLookups = 0;
            new Proxy((function(first, second) { return first + second; }).bind(null, 40), {
                get apply() { preparedLookups++; return undefined; }
            })
        "#,
            ))
            .unwrap()
            .as_object()
            .unwrap();
        let materialized = Cell::new(false);
        let result = function
            .call_prepared(
                JsValue::undefined(),
                PreparedArguments::new(vec![JsValue::undefined()], |values, context| {
                    assert_eq!(
                        context
                            .global_object()
                            .get(js_string!("preparedLookups"), context.access())?,
                        JsValue::new(1)
                    );
                    materialized.set(true);
                    values[0] = JsValue::new(2);
                    Ok(())
                }),
                context,
            )
            .unwrap();
        assert!(materialized.get());
        assert_eq!(result, JsValue::new(42));
    }

    #[test]
    fn proxy_trap_wrappers_receive_an_immediately_callable_prepared_function() {
        let context = &mut Context::default();
        let object = context.eval(Source::from_bytes(r#"
            globalThis.preparedOrder = [];
            globalThis.preparedGetterRan = false;
            const trap = new Proxy(function(target, key, descriptor) {
                preparedOrder.push('define');
                return descriptor.value() === 42;
            }, {
                apply(target, receiver, args) {
                    preparedOrder.push('apply');
                    return Reflect.apply(target, receiver, args);
                }
            }).bind(null);
            new Proxy({}, {
                get defineProperty() { preparedGetterRan = true; preparedOrder.push('get'); return trap; }
            })
        "#)).unwrap().as_object().unwrap();
        let answer = native_answer(context);
        let function = object
            .define_prepared_function(
                &js_string!("answer").into(),
                PropertyDescriptor::builder()
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                Box::new(|context| {
                    assert_eq!(
                        context
                            .global_object()
                            .get(js_string!("preparedGetterRan"), context.access())?,
                        JsValue::new(true)
                    );
                    Ok(answer)
                }),
                context,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            function
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap(),
            JsValue::new(42)
        );
        assert_eq!(
            context
                .eval(Source::from_bytes("preparedOrder.join(',')"))
                .unwrap(),
            JsValue::from(js_string!("get,apply,define"))
        );
    }
}
