//! Data structures that contain intrinsic objects and constructors.

use surgeist_js_gc::heap::Trace as HeapTrace;

use crate::{
    JsSymbol, Runtime,
    builtins::{Array, iterable::IteratorPrototypes, uri::UriFunctions},
    js_string,
    object::{
        CONSTRUCTOR, JsFunction, JsObject, ObjectEdge, PROTOTYPE,
        internal_methods::immutable_prototype::IMMUTABLE_PROTOTYPE_EXOTIC_INTERNAL_METHODS,
        shape::{
            RootShape,
            shared_shape::template::{ObjectTemplate, ObjectTemplateBuilder},
        },
    },
    property::{Attribute, PropertyKey},
    realm::BootstrapRoots,
};

#[cfg(feature = "intl")]
use crate::builtins::intl::Intl;

/// The intrinsic objects and constructors owned by a realm.
#[derive(Debug, HeapTrace)]
pub struct Intrinsics {
    /// Cached standard constructors
    pub(super) constructors: StandardConstructors,
    /// Cached intrinsic objects
    pub(super) objects: IntrinsicObjects,
    /// Cached object templates.
    pub(super) templates: ObjectTemplates,
}

impl Intrinsics {
    /// Creates a new set of uninitialized intrinsics.
    ///
    /// Creates all the required empty objects for every intrinsic in this realm.
    ///
    /// To initialize all the intrinsics with their spec properties, see [`Realm::initialize`].
    ///
    /// [`Realm::initialize`]: crate::realm::Realm::initialize
    pub(crate) fn uninit(root_shape: &RootShape, roots: &mut BootstrapRoots) -> Option<Self> {
        let constructors = StandardConstructors::uninit(root_shape.runtime(), roots);
        let templates = ObjectTemplates::new(root_shape, &constructors, roots);

        Some(Self {
            constructors,
            objects: IntrinsicObjects::uninit(root_shape.runtime(), roots)?,
            templates,
        })
    }

    /// Returns the runtime owning these intrinsics.
    #[must_use]
    pub fn runtime(&self) -> Runtime {
        self.constructors.object().prototype().runtime()
    }

    /// Return the cached intrinsic objects.
    #[inline]
    #[must_use]
    pub const fn objects(&self) -> &IntrinsicObjects {
        &self.objects
    }

    /// Return the cached standard constructors.
    #[inline]
    #[must_use]
    pub const fn constructors(&self) -> &StandardConstructors {
        &self.constructors
    }

    pub(crate) const fn templates(&self) -> &ObjectTemplates {
        &self.templates
    }
}

/// Stores a constructor (such as `Object`) and its corresponding prototype.
#[derive(Debug, HeapTrace)]
pub struct StandardConstructor {
    constructor: ObjectEdge,
    prototype: ObjectEdge,
}

impl StandardConstructor {
    fn uninit(runtime: &Runtime, roots: &mut BootstrapRoots) -> Self {
        Self::retain(
            roots,
            JsFunction::empty_intrinsic_function(runtime, true),
            JsObject::with_null_proto(runtime),
        )
    }
}

impl StandardConstructor {
    /// Creates a new `StandardConstructor` from the constructor and the prototype.
    pub(crate) fn new(constructor: JsFunction, prototype: JsObject) -> Self {
        Self {
            constructor: constructor.into_record(),
            prototype: prototype.into_record(),
        }
    }

    /// Build a constructor with a defined prototype.
    fn with_prototype(runtime: &Runtime, roots: &mut BootstrapRoots, prototype: JsObject) -> Self {
        Self::retain(
            roots,
            JsFunction::empty_intrinsic_function(runtime, true),
            prototype,
        )
    }

    fn retain(roots: &mut BootstrapRoots, constructor: JsFunction, prototype: JsObject) -> Self {
        Self {
            constructor: roots.retain(constructor.into()),
            prototype: roots.retain(prototype),
        }
    }

    /// Return the prototype of the constructor object.
    ///
    /// This is the same as `Object.prototype`, `Array.prototype`, etc.
    #[inline]
    #[must_use]
    pub fn prototype(&self) -> JsObject {
        JsObject::from_inner(self.prototype.clone())
    }

    /// Return the constructor object.
    ///
    /// This is the same as `Object`, `Array`, etc.
    #[inline]
    #[must_use]
    pub fn constructor(&self) -> JsObject {
        JsObject::from_inner(self.constructor.clone())
    }
}

/// Cached core standard constructors.
#[derive(Debug, HeapTrace)]
pub struct StandardConstructors {
    object: StandardConstructor,
    proxy: StandardConstructor,
    date: StandardConstructor,
    function: StandardConstructor,
    async_function: StandardConstructor,
    generator_function: StandardConstructor,
    async_generator_function: StandardConstructor,
    array: StandardConstructor,
    bigint: StandardConstructor,
    number: StandardConstructor,
    boolean: StandardConstructor,
    string: StandardConstructor,
    regexp: StandardConstructor,
    symbol: StandardConstructor,
    error: StandardConstructor,
    type_error: StandardConstructor,
    reference_error: StandardConstructor,
    range_error: StandardConstructor,
    syntax_error: StandardConstructor,
    eval_error: StandardConstructor,
    uri_error: StandardConstructor,
    aggregate_error: StandardConstructor,
    map: StandardConstructor,
    set: StandardConstructor,
    typed_array: StandardConstructor,
    typed_int8_array: StandardConstructor,
    typed_uint8_array: StandardConstructor,
    typed_uint8clamped_array: StandardConstructor,
    typed_int16_array: StandardConstructor,
    typed_uint16_array: StandardConstructor,
    typed_int32_array: StandardConstructor,
    typed_uint32_array: StandardConstructor,
    typed_bigint64_array: StandardConstructor,
    typed_biguint64_array: StandardConstructor,
    #[cfg(feature = "float16")]
    typed_float16_array: StandardConstructor,
    typed_float32_array: StandardConstructor,
    typed_float64_array: StandardConstructor,
    array_buffer: StandardConstructor,
    shared_array_buffer: StandardConstructor,
    data_view: StandardConstructor,
    date_time_format: StandardConstructor,
    promise: StandardConstructor,
    weak_ref: StandardConstructor,
    weak_map: StandardConstructor,
    weak_set: StandardConstructor,
    iterator: StandardConstructor,
    finalization_registry: StandardConstructor,
    #[cfg(feature = "intl")]
    collator: StandardConstructor,
    #[cfg(feature = "intl")]
    list_format: StandardConstructor,
    #[cfg(feature = "intl")]
    locale: StandardConstructor,
    #[cfg(feature = "intl")]
    segmenter: StandardConstructor,
    #[cfg(feature = "intl")]
    plural_rules: StandardConstructor,
    #[cfg(feature = "intl")]
    number_format: StandardConstructor,
    #[cfg(feature = "temporal")]
    instant: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_date_time: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_date: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_time: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_year_month: StandardConstructor,
    #[cfg(feature = "temporal")]
    plain_month_day: StandardConstructor,
    #[cfg(feature = "temporal")]
    time_zone: StandardConstructor,
    #[cfg(feature = "temporal")]
    duration: StandardConstructor,
    #[cfg(feature = "temporal")]
    zoned_date_time: StandardConstructor,
    #[cfg(feature = "temporal")]
    calendar: StandardConstructor,
}

impl StandardConstructors {
    fn uninit(runtime: &Runtime, roots: &mut BootstrapRoots) -> Self {
        Self {
            object: StandardConstructor::with_prototype(
                runtime,
                roots,
                JsObject::ordinary_prototype(runtime, &IMMUTABLE_PROTOTYPE_EXOTIC_INTERNAL_METHODS),
            ),
            async_generator_function: StandardConstructor::uninit(runtime, roots),
            proxy: StandardConstructor::uninit(runtime, roots),
            date: StandardConstructor::uninit(runtime, roots),
            function: StandardConstructor::retain(
                roots,
                JsFunction::empty_intrinsic_function(runtime, true),
                JsFunction::empty_intrinsic_function(runtime, false).into(),
            ),
            async_function: StandardConstructor::uninit(runtime, roots),
            generator_function: StandardConstructor::uninit(runtime, roots),
            array: StandardConstructor::with_prototype(
                runtime,
                roots,
                JsObject::from_proto_and_data(runtime, None, Array)
                    .expect("intrinsic prototype has no external heap edges"),
            ),
            bigint: StandardConstructor::uninit(runtime, roots),
            number: StandardConstructor::with_prototype(
                runtime,
                roots,
                JsObject::from_proto_and_data(runtime, None, 0.0)
                    .expect("intrinsic prototype has no external heap edges"),
            ),
            boolean: StandardConstructor::with_prototype(
                runtime,
                roots,
                JsObject::from_proto_and_data(runtime, None, false)
                    .expect("intrinsic prototype has no external heap edges"),
            ),
            string: StandardConstructor::with_prototype(
                runtime,
                roots,
                JsObject::from_proto_and_data(runtime, None, js_string!())
                    .expect("intrinsic prototype has no external heap edges"),
            ),
            regexp: StandardConstructor::uninit(runtime, roots),
            symbol: StandardConstructor::uninit(runtime, roots),
            error: StandardConstructor::uninit(runtime, roots),
            type_error: StandardConstructor::uninit(runtime, roots),
            reference_error: StandardConstructor::uninit(runtime, roots),
            range_error: StandardConstructor::uninit(runtime, roots),
            syntax_error: StandardConstructor::uninit(runtime, roots),
            eval_error: StandardConstructor::uninit(runtime, roots),
            uri_error: StandardConstructor::uninit(runtime, roots),
            aggregate_error: StandardConstructor::uninit(runtime, roots),
            map: StandardConstructor::uninit(runtime, roots),
            set: StandardConstructor::uninit(runtime, roots),
            typed_array: StandardConstructor::uninit(runtime, roots),
            typed_int8_array: StandardConstructor::uninit(runtime, roots),
            typed_uint8_array: StandardConstructor::uninit(runtime, roots),
            typed_uint8clamped_array: StandardConstructor::uninit(runtime, roots),
            typed_int16_array: StandardConstructor::uninit(runtime, roots),
            typed_uint16_array: StandardConstructor::uninit(runtime, roots),
            typed_int32_array: StandardConstructor::uninit(runtime, roots),
            typed_uint32_array: StandardConstructor::uninit(runtime, roots),
            typed_bigint64_array: StandardConstructor::uninit(runtime, roots),
            typed_biguint64_array: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "float16")]
            typed_float16_array: StandardConstructor::uninit(runtime, roots),
            typed_float32_array: StandardConstructor::uninit(runtime, roots),
            typed_float64_array: StandardConstructor::uninit(runtime, roots),
            array_buffer: StandardConstructor::uninit(runtime, roots),
            shared_array_buffer: StandardConstructor::uninit(runtime, roots),
            data_view: StandardConstructor::uninit(runtime, roots),
            date_time_format: StandardConstructor::uninit(runtime, roots),
            promise: StandardConstructor::uninit(runtime, roots),
            weak_ref: StandardConstructor::uninit(runtime, roots),
            weak_map: StandardConstructor::uninit(runtime, roots),
            weak_set: StandardConstructor::uninit(runtime, roots),
            iterator: StandardConstructor::uninit(runtime, roots),
            finalization_registry: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "intl")]
            collator: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "intl")]
            list_format: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "intl")]
            locale: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "intl")]
            segmenter: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "intl")]
            plural_rules: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "intl")]
            number_format: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            instant: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            plain_date_time: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            plain_date: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            plain_time: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            plain_year_month: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            plain_month_day: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            time_zone: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            duration: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            zoned_date_time: StandardConstructor::uninit(runtime, roots),
            #[cfg(feature = "temporal")]
            calendar: StandardConstructor::uninit(runtime, roots),
        }
    }
}

impl StandardConstructors {
    /// Returns the `AsyncGeneratorFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorfunction-constructor
    #[inline]
    #[must_use]
    pub const fn async_generator_function(&self) -> &StandardConstructor {
        &self.async_generator_function
    }

    /// Returns the `Object` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-constructor
    #[inline]
    #[must_use]
    pub const fn object(&self) -> &StandardConstructor {
        &self.object
    }

    /// Returns the `Proxy` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-proxy-constructor
    #[inline]
    #[must_use]
    pub const fn proxy(&self) -> &StandardConstructor {
        &self.proxy
    }

    /// Returns the `Date` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    #[inline]
    #[must_use]
    pub const fn date(&self) -> &StandardConstructor {
        &self.date
    }

    /// Returns the `Function` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-constructor
    #[inline]
    #[must_use]
    pub const fn function(&self) -> &StandardConstructor {
        &self.function
    }

    /// Returns the `AsyncFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-async-function-constructor
    #[inline]
    #[must_use]
    pub const fn async_function(&self) -> &StandardConstructor {
        &self.async_function
    }

    /// Returns the `GeneratorFunction` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorfunction-constructor
    #[inline]
    #[must_use]
    pub const fn generator_function(&self) -> &StandardConstructor {
        &self.generator_function
    }

    /// Returns the `Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array-constructor
    #[inline]
    #[must_use]
    pub const fn array(&self) -> &StandardConstructor {
        &self.array
    }

    /// Returns the `BigInt` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint-constructor
    #[inline]
    #[must_use]
    pub const fn bigint(&self) -> &StandardConstructor {
        &self.bigint
    }

    /// Returns the `Number` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number-constructor
    #[inline]
    #[must_use]
    pub const fn number(&self) -> &StandardConstructor {
        &self.number
    }

    /// Returns the `Boolean` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean-constructor
    #[inline]
    #[must_use]
    pub const fn boolean(&self) -> &StandardConstructor {
        &self.boolean
    }

    /// Returns the `String` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string-constructor
    #[inline]
    #[must_use]
    pub const fn string(&self) -> &StandardConstructor {
        &self.string
    }

    /// Returns the `RegExp` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-regexp-constructor
    #[inline]
    #[must_use]
    pub const fn regexp(&self) -> &StandardConstructor {
        &self.regexp
    }

    /// Returns the `Symbol` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol-constructor
    #[inline]
    #[must_use]
    pub const fn symbol(&self) -> &StandardConstructor {
        &self.symbol
    }

    /// Returns the `Error` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-error-constructor
    #[inline]
    #[must_use]
    pub const fn error(&self) -> &StandardConstructor {
        &self.error
    }

    /// Returns the `ReferenceError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-referenceerror
    #[inline]
    #[must_use]
    pub const fn reference_error(&self) -> &StandardConstructor {
        &self.reference_error
    }

    /// Returns the `TypeError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
    #[inline]
    #[must_use]
    pub const fn type_error(&self) -> &StandardConstructor {
        &self.type_error
    }

    /// Returns the `RangeError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-rangeerror
    #[inline]
    #[must_use]
    pub const fn range_error(&self) -> &StandardConstructor {
        &self.range_error
    }

    /// Returns the `SyntaxError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
    #[inline]
    #[must_use]
    pub const fn syntax_error(&self) -> &StandardConstructor {
        &self.syntax_error
    }

    /// Returns the `EvalError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-evalerror
    #[inline]
    #[must_use]
    pub const fn eval_error(&self) -> &StandardConstructor {
        &self.eval_error
    }

    /// Returns the `URIError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-urierror
    #[inline]
    #[must_use]
    pub const fn uri_error(&self) -> &StandardConstructor {
        &self.uri_error
    }

    /// Returns the `AggregateError` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-aggregate-error-constructor
    #[inline]
    #[must_use]
    pub const fn aggregate_error(&self) -> &StandardConstructor {
        &self.aggregate_error
    }

    /// Returns the `Map` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-map-constructor
    #[inline]
    #[must_use]
    pub const fn map(&self) -> &StandardConstructor {
        &self.map
    }

    /// Returns the `Set` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-constructor
    #[inline]
    #[must_use]
    pub const fn set(&self) -> &StandardConstructor {
        &self.set
    }

    /// Returns the `TypedArray` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_array(&self) -> &StandardConstructor {
        &self.typed_array
    }

    /// Returns the `Int8Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_int8_array(&self) -> &StandardConstructor {
        &self.typed_int8_array
    }

    /// Returns the `Uint8Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint8_array(&self) -> &StandardConstructor {
        &self.typed_uint8_array
    }

    /// Returns the `Uint8ClampedArray` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint8clamped_array(&self) -> &StandardConstructor {
        &self.typed_uint8clamped_array
    }

    /// Returns the `Int16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_int16_array(&self) -> &StandardConstructor {
        &self.typed_int16_array
    }

    /// Returns the `Uint16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint16_array(&self) -> &StandardConstructor {
        &self.typed_uint16_array
    }

    /// Returns the `Uint32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_uint32_array(&self) -> &StandardConstructor {
        &self.typed_uint32_array
    }

    /// Returns the `Int32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_int32_array(&self) -> &StandardConstructor {
        &self.typed_int32_array
    }

    /// Returns the `BigInt64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_bigint64_array(&self) -> &StandardConstructor {
        &self.typed_bigint64_array
    }

    /// Returns the `BigUint64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_biguint64_array(&self) -> &StandardConstructor {
        &self.typed_biguint64_array
    }

    /// Returns the `Float16Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[cfg(feature = "float16")]
    #[inline]
    #[must_use]
    pub const fn typed_float16_array(&self) -> &StandardConstructor {
        &self.typed_float16_array
    }

    /// Returns the `Float32Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_float32_array(&self) -> &StandardConstructor {
        &self.typed_float32_array
    }

    /// Returns the `Float64Array` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typedarray-constructors
    #[inline]
    #[must_use]
    pub const fn typed_float64_array(&self) -> &StandardConstructor {
        &self.typed_float64_array
    }

    /// Returns the `ArrayBuffer` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-constructor
    #[inline]
    #[must_use]
    pub const fn array_buffer(&self) -> &StandardConstructor {
        &self.array_buffer
    }

    /// Returns the `SharedArrayBuffer` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-sharedarraybuffer-constructor
    #[inline]
    #[must_use]
    pub const fn shared_array_buffer(&self) -> &StandardConstructor {
        &self.shared_array_buffer
    }

    /// Returns the `DataView` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview-constructor
    #[inline]
    #[must_use]
    pub const fn data_view(&self) -> &StandardConstructor {
        &self.data_view
    }

    /// Returns the `Intl.DateTimeFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl-datetimeformat-constructor
    #[inline]
    #[must_use]
    pub const fn date_time_format(&self) -> &StandardConstructor {
        &self.date_time_format
    }

    /// Returns the `Promise` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise-constructor
    #[inline]
    #[must_use]
    pub const fn promise(&self) -> &StandardConstructor {
        &self.promise
    }

    /// Returns the `WeakRef` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weak-ref-constructor
    #[inline]
    #[must_use]
    pub const fn weak_ref(&self) -> &StandardConstructor {
        &self.weak_ref
    }

    /// Returns the `WeakMap` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakmap-constructor
    #[inline]
    #[must_use]
    pub const fn weak_map(&self) -> &StandardConstructor {
        &self.weak_map
    }

    /// Returns the `WeakSet` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-weakset-constructor
    #[inline]
    #[must_use]
    pub const fn weak_set(&self) -> &StandardConstructor {
        &self.weak_set
    }

    /// Returns the `Iterator` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-iterator-helpers/#sec-iterator-constructor
    #[inline]
    #[must_use]
    pub const fn iterator(&self) -> &StandardConstructor {
        &self.iterator
    }

    /// Returns the `FinalizationRegistry` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-finalization-registry-constructor
    #[inline]
    #[must_use]
    pub const fn finalization_registry(&self) -> &StandardConstructor {
        &self.finalization_registry
    }

    /// Returns the `Intl.Collator` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.collator
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn collator(&self) -> &StandardConstructor {
        &self.collator
    }

    /// Returns the `Intl.ListFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.ListFormat
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn list_format(&self) -> &StandardConstructor {
        &self.list_format
    }

    /// Returns the `Intl.Locale` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-Intl.Locale
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn locale(&self) -> &StandardConstructor {
        &self.locale
    }

    /// Returns the `Intl.Segmenter` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.segmenter
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn segmenter(&self) -> &StandardConstructor {
        &self.segmenter
    }

    /// Returns the `Intl.PluralRules` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.pluralrules
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn plural_rules(&self) -> &StandardConstructor {
        &self.plural_rules
    }

    /// Returns the `Intl.NumberFormat` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat
    #[inline]
    #[must_use]
    #[cfg(feature = "intl")]
    pub const fn number_format(&self) -> &StandardConstructor {
        &self.number_format
    }

    /// Returns the `Temporal.Instant` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-instant-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn instant(&self) -> &StandardConstructor {
        &self.instant
    }

    /// Returns the `Temporal.PlainDateTime` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plaindatetime-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_date_time(&self) -> &StandardConstructor {
        &self.plain_date_time
    }

    /// Returns the `Temporal.PlainDate` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plaindate-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_date(&self) -> &StandardConstructor {
        &self.plain_date
    }

    /// Returns the `Temporal.PlainTime` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plaintime-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_time(&self) -> &StandardConstructor {
        &self.plain_time
    }

    /// Returns the `Temporal.PlainYearMonth` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plainyearmonth-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_year_month(&self) -> &StandardConstructor {
        &self.plain_year_month
    }

    /// Returns the `Temporal.PlainMonthDay` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-plainmonthday-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn plain_month_day(&self) -> &StandardConstructor {
        &self.plain_month_day
    }

    /// Returns the `Temporal.TimeZone` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-timezone-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn time_zone(&self) -> &StandardConstructor {
        &self.time_zone
    }

    /// Returns the `Temporal.Duration` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-duration-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn duration(&self) -> &StandardConstructor {
        &self.duration
    }

    /// Returns the `Temporal.ZonedDateTime` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-zoneddatetime-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn zoned_date_time(&self) -> &StandardConstructor {
        &self.zoned_date_time
    }

    /// Returns the `Temporal.Calendar` constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-calendar-constructor
    #[inline]
    #[must_use]
    #[cfg(feature = "temporal")]
    pub const fn calendar(&self) -> &StandardConstructor {
        &self.calendar
    }
}

/// Cached intrinsic objects
#[derive(Debug, HeapTrace)]
pub struct IntrinsicObjects {
    /// [`%Reflect%`](https://tc39.es/ecma262/#sec-reflect)
    reflect: ObjectEdge,

    /// [`%Math%`](https://tc39.es/ecma262/#sec-math)
    math: ObjectEdge,

    /// [`%JSON%`](https://tc39.es/ecma262/#sec-json)
    json: ObjectEdge,

    /// [`%ThrowTypeError%`](https://tc39.es/ecma262/#sec-%throwtypeerror%)
    throw_type_error: ObjectEdge,

    /// [`%Array.prototype.values%`](https://tc39.es/ecma262/#sec-array.prototype.values)
    array_prototype_values: ObjectEdge,

    /// [`%Array.prototype.toString%`](https://tc39.es/ecma262/#sec-array.prototype.tostring)
    array_prototype_to_string: ObjectEdge,

    /// Cached iterator prototypes.
    iterator_prototypes: IteratorPrototypes,

    /// [`%GeneratorFunction.prototype.prototype%`](https://tc39.es/ecma262/#sec-properties-of-generator-prototype)
    generator: ObjectEdge,

    /// [`%AsyncGeneratorFunction.prototype.prototype%`](https://tc39.es/ecma262/#sec-properties-of-asyncgenerator-prototype)
    async_generator: ObjectEdge,

    /// [`%Atomics%`](https://tc39.es/ecma262/#sec-atomics)
    atomics: ObjectEdge,

    /// [`%eval%`](https://tc39.es/ecma262/#sec-eval-x)
    eval: ObjectEdge,

    /// URI related functions
    uri_functions: UriFunctions,

    /// [`%isFinite%`](https://tc39.es/ecma262/#sec-isfinite-number)
    is_finite: ObjectEdge,

    /// [`%isNaN%`](https://tc39.es/ecma262/#sec-isnan-number)
    is_nan: ObjectEdge,

    /// [`%parseFloat%`](https://tc39.es/ecma262/#sec-parsefloat-string)
    parse_float: ObjectEdge,

    /// [`%parseInt%`](https://tc39.es/ecma262/#sec-parseint-string-radix)
    parse_int: ObjectEdge,

    /// [`%escape%`](https://tc39.es/ecma262/#sec-escape-string)
    #[cfg(feature = "annex-b")]
    escape: ObjectEdge,

    /// [`%unescape%`](https://tc39.es/ecma262/#sec-unescape-string)
    #[cfg(feature = "annex-b")]
    unescape: ObjectEdge,

    /// [`%Intl%`](https://tc39.es/ecma402/#intl-object)
    #[cfg(feature = "intl")]
    intl: ObjectEdge,

    /// [`%SegmentsPrototype%`](https://tc39.es/ecma402/#sec-%segmentsprototype%-object)
    #[cfg(feature = "intl")]
    segments_prototype: ObjectEdge,

    /// [`%Temporal%`](https://tc39.es/proposal-temporal/#sec-temporal-objects)
    #[cfg(feature = "temporal")]
    temporal: ObjectEdge,

    /// [`%Temporal.Now%`](https://tc39.es/proposal-temporal/#sec-temporal-now-object)
    #[cfg(feature = "temporal")]
    now: ObjectEdge,
}

impl IntrinsicObjects {
    /// Creates a new set of uninitialized intrinsic objects.
    ///
    /// Creates all the required empty objects for every intrinsic object in this realm.
    ///
    /// To initialize all the intrinsic objects with their spec properties, see [`Realm::initialize`].
    ///
    /// [`Realm::initialize`]: crate::realm::Realm::initialize
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn uninit(runtime: &Runtime, roots: &mut BootstrapRoots) -> Option<Self> {
        Some(Self {
            reflect: roots.retain(JsObject::with_null_proto(runtime)),
            math: roots.retain(JsObject::with_null_proto(runtime)),
            json: roots.retain(JsObject::with_null_proto(runtime)),
            throw_type_error: roots
                .retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            array_prototype_values: roots
                .retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            array_prototype_to_string: roots
                .retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            iterator_prototypes: IteratorPrototypes::uninit(runtime, roots),
            generator: roots.retain(JsObject::with_null_proto(runtime)),
            async_generator: roots.retain(JsObject::with_null_proto(runtime)),
            atomics: roots.retain(JsObject::with_null_proto(runtime)),
            eval: roots.retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            uri_functions: UriFunctions::uninit(runtime, roots),
            is_finite: roots.retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            is_nan: roots.retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            parse_float: roots.retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            parse_int: roots.retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            #[cfg(feature = "annex-b")]
            escape: roots.retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            #[cfg(feature = "annex-b")]
            unescape: roots.retain(JsFunction::empty_intrinsic_function(runtime, false).into()),
            #[cfg(feature = "intl")]
            intl: roots.retain(
                JsObject::new_unique(runtime, None, Intl::new()?)
                    .expect("Intl has no JavaScript heap edges")
                    .upcast(),
            ),
            #[cfg(feature = "intl")]
            segments_prototype: roots.retain(JsObject::with_null_proto(runtime)),
            #[cfg(feature = "temporal")]
            temporal: roots.retain(JsObject::with_null_proto(runtime)),
            #[cfg(feature = "temporal")]
            now: roots.retain(JsObject::with_null_proto(runtime)),
        })
    }

    /// Gets the [`%ThrowTypeError%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%throwtypeerror%
    #[inline]
    #[must_use]
    pub fn throw_type_error(&self) -> JsFunction {
        JsFunction::from_record(self.throw_type_error.clone())
    }

    /// Gets the [`%Array.prototype.values%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.values
    #[inline]
    #[must_use]
    pub fn array_prototype_values(&self) -> JsFunction {
        JsFunction::from_record(self.array_prototype_values.clone())
    }

    /// Gets the [`%Array.prototype.toString%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.tostring
    #[inline]
    #[must_use]
    pub fn array_prototype_to_string(&self) -> JsFunction {
        JsFunction::from_record(self.array_prototype_to_string.clone())
    }

    /// Gets the cached iterator prototypes.
    #[inline]
    #[must_use]
    pub const fn iterator_prototypes(&self) -> &IteratorPrototypes {
        &self.iterator_prototypes
    }

    /// Gets the [`%GeneratorFunction.prototype.prototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generator-objects
    #[inline]
    #[must_use]
    pub fn generator(&self) -> JsObject {
        JsObject::from_inner(self.generator.clone())
    }

    /// Gets the [`%AsyncGeneratorFunction.prototype.prototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgenerator-objects
    #[inline]
    #[must_use]
    pub fn async_generator(&self) -> JsObject {
        JsObject::from_inner(self.async_generator.clone())
    }

    /// Gets the [`%Atomics%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-atomics
    #[inline]
    #[must_use]
    pub fn atomics(&self) -> JsObject {
        JsObject::from_inner(self.atomics.clone())
    }

    /// Gets the [`%eval%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-eval-x
    #[inline]
    #[must_use]
    pub fn eval(&self) -> JsFunction {
        JsFunction::from_record(self.eval.clone())
    }

    /// Gets the URI intrinsic functions.
    #[inline]
    #[must_use]
    pub const fn uri_functions(&self) -> &UriFunctions {
        &self.uri_functions
    }

    /// Gets the [`%Reflect%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-reflect
    #[inline]
    #[must_use]
    pub fn reflect(&self) -> JsObject {
        JsObject::from_inner(self.reflect.clone())
    }

    /// Gets the [`%Math%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math
    #[inline]
    #[must_use]
    pub fn math(&self) -> JsObject {
        JsObject::from_inner(self.math.clone())
    }

    /// Gets the [`%JSON%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-json
    #[inline]
    #[must_use]
    pub fn json(&self) -> JsObject {
        JsObject::from_inner(self.json.clone())
    }

    /// Gets the [`%isFinite%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isfinite-number
    #[inline]
    #[must_use]
    pub fn is_finite(&self) -> JsFunction {
        JsFunction::from_record(self.is_finite.clone())
    }

    /// Gets the [`%isNaN%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isnan-number
    #[inline]
    #[must_use]
    pub fn is_nan(&self) -> JsFunction {
        JsFunction::from_record(self.is_nan.clone())
    }

    /// Gets the [`%parseFloat%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parsefloat-string
    #[inline]
    #[must_use]
    pub fn parse_float(&self) -> JsFunction {
        JsFunction::from_record(self.parse_float.clone())
    }

    /// Gets the [`%parseInt%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parseint-string-radix
    #[inline]
    #[must_use]
    pub fn parse_int(&self) -> JsFunction {
        JsFunction::from_record(self.parse_int.clone())
    }

    /// Gets the [`%escape%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-escape-string
    #[must_use]
    #[cfg(feature = "annex-b")]
    #[inline]
    pub fn escape(&self) -> JsFunction {
        JsFunction::from_record(self.escape.clone())
    }

    /// Gets the [`%unescape%`][spec] intrinsic function.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-unescape-string
    #[must_use]
    #[cfg(feature = "annex-b")]
    #[inline]
    pub fn unescape(&self) -> JsFunction {
        JsFunction::from_record(self.unescape.clone())
    }

    /// Gets the [`%Intl%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma402/#intl-object
    #[must_use]
    #[cfg(feature = "intl")]
    #[inline]
    pub fn intl(&self) -> JsObject<Intl> {
        JsObject::from_inner(self.intl.clone())
            .downcast::<Intl>()
            .expect("Intl intrinsic has its native brand")
    }

    /// Gets the [`%SegmentsPrototype%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-%segmentsprototype%-object
    #[must_use]
    #[cfg(feature = "intl")]
    pub fn segments_prototype(&self) -> JsObject {
        JsObject::from_inner(self.segments_prototype.clone())
    }

    /// Gets the [`%Temporal%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-objects
    #[cfg(feature = "temporal")]
    #[must_use]
    #[inline]
    pub fn temporal(&self) -> JsObject {
        JsObject::from_inner(self.temporal.clone())
    }

    /// Gets the [`%Temporal.Now%`][spec] intrinsic object.
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-now-object
    #[cfg(feature = "temporal")]
    #[must_use]
    #[inline]
    pub fn now(&self) -> JsObject {
        JsObject::from_inner(self.now.clone())
    }
}

/// Contains commonly used [`ObjectTemplate`]s.
#[derive(Debug, HeapTrace)]
pub(crate) struct ObjectTemplates {
    iterator_result: ObjectTemplate,
    ordinary_object: ObjectTemplate,
    array: ObjectTemplate,
    number: ObjectTemplate,
    string: ObjectTemplate,
    symbol: ObjectTemplate,
    bigint: ObjectTemplate,
    boolean: ObjectTemplate,

    regexp: ObjectTemplate,
    regexp_without_proto: ObjectTemplate,

    unmapped_arguments: ObjectTemplate,
    mapped_arguments: ObjectTemplate,

    function_with_prototype: ObjectTemplate,
    function_prototype: ObjectTemplate,

    function: ObjectTemplate,
    async_function: ObjectTemplate,
    generator_function: ObjectTemplate,
    async_generator_function: ObjectTemplate,

    function_without_proto: ObjectTemplate,
    function_with_prototype_without_proto: ObjectTemplate,

    namespace: ObjectTemplate,

    with_resolvers: ObjectTemplate,

    wait_async: ObjectTemplate,
}

impl ObjectTemplates {
    pub(crate) fn new(
        root_shape: &RootShape,
        constructors: &StandardConstructors,
        roots: &mut BootstrapRoots,
    ) -> Self {
        // pre-initialize used shapes.
        let ordinary_object =
            ObjectTemplateBuilder::with_prototype(root_shape, constructors.object().prototype());
        let mut array = ObjectTemplateBuilder::new(root_shape);
        let length_property_key: PropertyKey = js_string!("length").into();
        array.property(
            length_property_key.clone(),
            Attribute::WRITABLE | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        array.set_prototype(constructors.array().prototype());

        let number =
            ObjectTemplateBuilder::with_prototype(root_shape, constructors.number().prototype());
        let symbol =
            ObjectTemplateBuilder::with_prototype(root_shape, constructors.symbol().prototype());
        let bigint =
            ObjectTemplateBuilder::with_prototype(root_shape, constructors.bigint().prototype());
        let boolean =
            ObjectTemplateBuilder::with_prototype(root_shape, constructors.boolean().prototype());
        let mut string = ObjectTemplateBuilder::new(root_shape);
        string.property(
            length_property_key.clone(),
            Attribute::READONLY | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        string.set_prototype(constructors.string().prototype());

        let mut regexp_without_proto = ObjectTemplateBuilder::new(root_shape);
        regexp_without_proto.property(js_string!("lastIndex").into(), Attribute::WRITABLE);

        let mut regexp = regexp_without_proto.clone();
        regexp.set_prototype(constructors.regexp().prototype());

        let name_property_key: PropertyKey = js_string!("name").into();
        let mut function = ObjectTemplateBuilder::new(root_shape);
        function.property(
            length_property_key.clone(),
            Attribute::READONLY | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );
        function.property(
            name_property_key,
            Attribute::READONLY | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );

        let function_without_proto = function.clone();
        let mut async_function = function.clone();
        let mut function_with_prototype = function.clone();

        function_with_prototype.property(
            PROTOTYPE.into(),
            Attribute::WRITABLE | Attribute::PERMANENT | Attribute::NON_ENUMERABLE,
        );
        let mut generator_function = function_with_prototype.clone();
        let mut async_generator_function = function_with_prototype.clone();

        let function_with_prototype_without_proto = function_with_prototype.clone();

        function.set_prototype(constructors.function().prototype());
        function_with_prototype.set_prototype(constructors.function().prototype());
        async_function.set_prototype(constructors.async_function().prototype());
        generator_function.set_prototype(constructors.generator_function().prototype());
        async_generator_function.set_prototype(constructors.async_generator_function().prototype());

        let mut function_prototype = ordinary_object.clone();
        function_prototype.property(
            CONSTRUCTOR.into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        );

        let mut unmapped_arguments = ordinary_object.clone();

        // 4. Perform DefinePropertyOrThrow(obj, "length", PropertyDescriptor { [[Value]]: 𝔽(len),
        // [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        unmapped_arguments.property(
            length_property_key,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        // 7. Perform ! DefinePropertyOrThrow(obj, @@iterator, PropertyDescriptor {
        // [[Value]]: %Array.prototype.values%, [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }).
        unmapped_arguments.property(
            JsSymbol::iterator().into(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let mut mapped_arguments = unmapped_arguments.clone();

        // 8. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Get]]: %ThrowTypeError%, [[Set]]: %ThrowTypeError%, [[Enumerable]]: false,
        // [[Configurable]]: false }).
        unmapped_arguments.accessor(
            js_string!("callee").into(),
            true,
            true,
            Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );

        // 21. Perform ! DefinePropertyOrThrow(obj, "callee", PropertyDescriptor {
        // [[Value]]: func, [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }).
        mapped_arguments.property(
            js_string!("callee").into(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let mut iterator_result = ordinary_object.clone();
        iterator_result.property(
            js_string!("value").into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        );
        iterator_result.property(
            js_string!("done").into(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        );

        let mut namespace = ObjectTemplateBuilder::new(root_shape);
        namespace.property(JsSymbol::to_string_tag().into(), Attribute::empty());

        let with_resolvers = {
            let mut with_resolvers = ordinary_object.clone();

            with_resolvers
                // 4. Perform ! CreateDataPropertyOrThrow(obj, "promise", promiseCapability.[[Promise]]).
                .property(js_string!("promise").into(), Attribute::all())
                // 5. Perform ! CreateDataPropertyOrThrow(obj, "resolve", promiseCapability.[[Resolve]]).
                .property(js_string!("resolve").into(), Attribute::all())
                // 6. Perform ! CreateDataPropertyOrThrow(obj, "reject", promiseCapability.[[Reject]]).
                .property(js_string!("reject").into(), Attribute::all());

            with_resolvers
        };

        let wait_async = {
            let mut obj = ordinary_object.clone();

            obj.property(js_string!("async").into(), Attribute::all())
                .property(js_string!("value").into(), Attribute::all());

            obj
        };

        Self {
            iterator_result: iterator_result.build(roots),
            ordinary_object: ordinary_object.build(roots),
            array: array.build(roots),
            number: number.build(roots),
            string: string.build(roots),
            symbol: symbol.build(roots),
            bigint: bigint.build(roots),
            boolean: boolean.build(roots),
            regexp: regexp.build(roots),
            regexp_without_proto: regexp_without_proto.build(roots),
            unmapped_arguments: unmapped_arguments.build(roots),
            mapped_arguments: mapped_arguments.build(roots),
            function_with_prototype: function_with_prototype.build(roots),
            function_prototype: function_prototype.build(roots),
            function: function.build(roots),
            async_function: async_function.build(roots),
            generator_function: generator_function.build(roots),
            async_generator_function: async_generator_function.build(roots),
            function_without_proto: function_without_proto.build(roots),
            function_with_prototype_without_proto: function_with_prototype_without_proto
                .build(roots),
            namespace: namespace.build(roots),
            with_resolvers: with_resolvers.build(roots),
            wait_async: wait_async.build(roots),
        }
    }

    /// Cached iterator result template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"done"`: (`WRITABLE`, `CONFIGURABLE`, `ENUMERABLE`)
    /// 3. `"value"`: (`WRITABLE`, `CONFIGURABLE`, `ENUMERABLE`)
    pub(crate) const fn iterator_result(&self) -> &ObjectTemplate {
        &self.iterator_result
    }

    /// Cached ordinary object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    pub(crate) const fn ordinary_object(&self) -> &ObjectTemplate {
        &self.ordinary_object
    }

    /// Cached array object template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`WRITABLE`, `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `Array.prototype`
    pub(crate) const fn array(&self) -> &ObjectTemplate {
        &self.array
    }

    /// Cached number object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Number.prototype`
    pub(crate) const fn number(&self) -> &ObjectTemplate {
        &self.number
    }

    /// Cached string object template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `String.prototype`
    pub(crate) const fn string(&self) -> &ObjectTemplate {
        &self.string
    }

    /// Cached symbol object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Symbol.prototype`
    pub(crate) const fn symbol(&self) -> &ObjectTemplate {
        &self.symbol
    }

    /// Cached bigint object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `BigInt.prototype`
    pub(crate) const fn bigint(&self) -> &ObjectTemplate {
        &self.bigint
    }

    /// Cached boolean object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Boolean.prototype`
    pub(crate) const fn boolean(&self) -> &ObjectTemplate {
        &self.boolean
    }

    /// Cached regexp object template.
    ///
    /// Transitions:
    ///
    /// 1. `"lastIndex"`: (`WRITABLE` , `PERMANENT`,`NON_ENUMERABLE`)
    pub(crate) const fn regexp(&self) -> &ObjectTemplate {
        &self.regexp
    }

    /// Cached regexp object template without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"lastIndex"`: (`WRITABLE` , `PERMANENT`,`NON_ENUMERABLE`)
    /// 2. `__proto__`: `RegExp.prototype`
    pub(crate) const fn regexp_without_proto(&self) -> &ObjectTemplate {
        &self.regexp_without_proto
    }

    /// Cached unmapped arguments object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"length"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `@@iterator`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `get/set` `"callee"`: (`NON_ENUMERABLE`, `PERMANENT`)
    pub(crate) const fn unmapped_arguments(&self) -> &ObjectTemplate {
        &self.unmapped_arguments
    }

    /// Cached mapped arguments object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"length"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `@@iterator`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `"callee"`: (`WRITABLE`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn mapped_arguments(&self) -> &ObjectTemplate {
        &self.mapped_arguments
    }

    /// Cached function object with `"prototype"` property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    /// 4. `__proto__`: `Function.prototype`
    pub(crate) const fn function_with_prototype(&self) -> &ObjectTemplate {
        &self.function_with_prototype
    }

    /// Cached constructor function object template.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"constructor"`: (`WRITABLE`, `CONFIGURABLE`, `NON_ENUMERABLE`)
    pub(crate) const fn function_prototype(&self) -> &ObjectTemplate {
        &self.function_prototype
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `__proto__`: `Function.prototype`
    pub(crate) const fn function(&self) -> &ObjectTemplate {
        &self.function
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `__proto__`: `AsyncFunction.prototype`
    pub(crate) const fn async_function(&self) -> &ObjectTemplate {
        &self.async_function
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    /// 4. `__proto__`: `GeneratorFunction.prototype`
    pub(crate) const fn generator_function(&self) -> &ObjectTemplate {
        &self.generator_function
    }

    /// Cached function object property template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    /// 4. `__proto__`: `AsyncGeneratorFunction.prototype`
    pub(crate) const fn async_generator_function(&self) -> &ObjectTemplate {
        &self.async_generator_function
    }

    /// Cached function object without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn function_without_proto(&self) -> &ObjectTemplate {
        &self.function_without_proto
    }

    /// Cached function object with `"prototype"` and without `__proto__` template.
    ///
    /// Transitions:
    ///
    /// 1. `"length"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 2. `"name"`: (`READONLY`, `NON_ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"prototype"`: (`WRITABLE`, `PERMANENT`, `NON_ENUMERABLE`)
    pub(crate) const fn function_with_prototype_without_proto(&self) -> &ObjectTemplate {
        &self.function_with_prototype_without_proto
    }

    /// Cached namespace object template.
    ///
    /// Transitions:
    ///
    /// 1. `@@toStringTag`: (`READONLY`, `NON_ENUMERABLE`, `PERMANENT`)
    pub(crate) const fn namespace(&self) -> &ObjectTemplate {
        &self.namespace
    }

    /// Cached object from the `Promise.withResolvers` method.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"promise"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"resolve"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    /// 4. `"reject"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn with_resolvers(&self) -> &ObjectTemplate {
        &self.with_resolvers
    }

    /// Cached object from the `Atomics.waitAsync` method.
    ///
    /// Transitions:
    ///
    /// 1. `__proto__`: `Object.prototype`
    /// 2. `"async"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    /// 3. `"value"`: (`WRITABLE`, `ENUMERABLE`, `CONFIGURABLE`)
    pub(crate) const fn wait_async(&self) -> &ObjectTemplate {
        &self.wait_async
    }
}

#[cfg(test)]
mod projection_tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crate::{
        Context, JsObject, JsValue, Source, builtins::promise::PromiseState,
        execution::ExecutionOptions, object::builtins::JsPromise,
    };
    use surgeist_js_gc::heap::{AccessError, RuntimeHeap};

    fn context() -> Context {
        let builder = Context::builder();
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(100_000);
        builder.build().unwrap()
    }

    fn guest_object(context: &mut Context, source: &str) -> JsObject {
        context
            .with_execution(ExecutionOptions::new(100_000), |mut access| {
                access.eval(Source::from_bytes(source))
            })
            .unwrap()
            .result
            .unwrap()
            .as_object()
            .unwrap()
    }

    fn under_heap_read<T>(
        heap: &RuntimeHeap,
        operation: impl FnOnce() -> T,
    ) -> std::thread::Result<T> {
        heap.read(|_| {
            assert_eq!(
                heap.allocate(()).unwrap_err().reason,
                AccessError::HeapBorrowed
            );
            catch_unwind(AssertUnwindSafe(operation))
        })
        .unwrap()
    }

    #[test]
    fn standard_prototype_projection_under_heap_read_does_not_unwind_and_retries_identity() {
        let mut context = context();
        let expected = guest_object(&mut context, "Promise.prototype");
        let heap = context.runtime().heap.clone();
        let attempt = under_heap_read(&heap, || {
            context.intrinsics().constructors().promise().prototype()
        });
        assert!(
            attempt.is_ok(),
            "intrinsic prototype acquisition must refuse without a Rust unwind"
        );
        drop(attempt);
        let retried = context.intrinsics().constructors().promise().prototype();
        assert_eq!(retried.identity(), expected.identity());
        assert_eq!(retried, expected);
    }

    #[test]
    fn standard_constructor_projection_under_heap_read_does_not_unwind_and_retries_identity() {
        let mut context = context();
        let expected = guest_object(&mut context, "Promise");
        let heap = context.runtime().heap.clone();
        let attempt = under_heap_read(&heap, || {
            context.intrinsics().constructors().promise().constructor()
        });
        assert!(
            attempt.is_ok(),
            "intrinsic constructor acquisition must refuse without a Rust unwind"
        );
        drop(attempt);
        let retried = context.intrinsics().constructors().promise().constructor();
        assert_eq!(retried.identity(), expected.identity());
        assert_eq!(retried, expected);
    }

    #[test]
    fn intrinsic_runtime_association_under_heap_read_does_not_root_or_unwind() {
        let context = context();
        let heap = context.runtime().heap.clone();
        let attempt = under_heap_read(&heap, || context.intrinsics().runtime());
        assert!(
            attempt.is_ok(),
            "runtime association must not acquire an intrinsic object root"
        );
        let associated = attempt.unwrap();
        assert!(associated.same_heap(context.runtime()));
        let retried = context.intrinsics().runtime();
        assert!(retried.same_heap(context.runtime()));
    }

    #[test]
    fn pending_factory_under_heap_read_does_not_unwind_and_allows_fresh_resolution() {
        let mut context = context();
        let heap = context.runtime().heap.clone();
        let attempt = under_heap_read(&heap, || JsPromise::new_pending(context.access()));
        assert!(
            attempt.is_ok(),
            "actual heap access refusal must not become a Rust unwind"
        );
        drop(attempt);
        context.runtime().collect().unwrap();

        let (promise, functions) = JsPromise::new_pending(context.access());
        assert_eq!(promise.state().unwrap(), PromiseState::Pending);
        functions
            .resolve
            .call(&JsValue::undefined(), &[71.into()], context.access())
            .unwrap();
        assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(71.into()));
    }

    #[test]
    fn unborrowed_intrinsic_identity_and_pending_factory_preserve_actual_guest_objects() {
        let mut context = context();
        let expected_constructor = guest_object(&mut context, "Promise");
        let expected_prototype = guest_object(&mut context, "Promise.prototype");
        assert_eq!(
            context.intrinsics().constructors().promise().constructor(),
            expected_constructor
        );
        assert_eq!(
            context.intrinsics().constructors().promise().prototype(),
            expected_prototype
        );
        assert!(context.intrinsics().runtime().same_heap(context.runtime()));

        let value = JsValue::from(JsObject::with_null_proto(context.runtime()));
        let (promise, functions) = JsPromise::new_pending(context.access());
        assert_eq!(promise.state().unwrap(), PromiseState::Pending);
        functions
            .resolve
            .call(&JsValue::undefined(), &[value.clone()], context.access())
            .unwrap();
        assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(value));
    }

    #[test]
    fn runtime_association_keeps_the_heap_without_rooting_intrinsic_objects_after_context_drop() {
        let context = context();
        let weak_constructor = context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .edge();
        let weak_prototype = context
            .intrinsics()
            .constructors()
            .promise()
            .prototype()
            .edge();
        let runtime = context.intrinsics().runtime();
        assert!(runtime.same_heap(context.runtime()));
        drop(context);

        runtime.collect().unwrap();
        assert!(!weak_constructor.is_live().unwrap());
        assert!(!weak_prototype.is_live().unwrap());
    }
}
