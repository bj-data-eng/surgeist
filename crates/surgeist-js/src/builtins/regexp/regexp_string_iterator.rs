//! This module implements the global `RegExp String Iterator` object.
//!
//! A `RegExp` String Iterator is an object, that represents a specific iteration over some
//! specific String instance object, matching against some specific `RegExp` instance object.
//! There is not a named constructor for `RegExp` String Iterator objects. Instead, `RegExp`
//! String Iterator objects are created by calling certain methods of `RegExp` instance objects.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-regexp-string-iterator-objects

use crate::{
    Context, JsData, JsResult, JsString, JsValue,
    builtins::{
        BuiltInBuilder, IntrinsicObject, iterable::create_iter_result_object_trusted, regexp,
    },
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_string,
    object::{JsObject, ObjectEdge},
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
};
use regexp::{RegExp, advance_string_index};
use surgeist_js_gc::heap::Trace as HeapTrace;

/// The `RegExp String Iterator` object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-regexp-string-iterator-objects
#[derive(Debug, Clone, HeapTrace, JsData)]
pub(crate) struct RegExpStringIterator {
    matcher: Option<ObjectEdge>,
    #[heap_trace(skip)]
    string: JsString,
    global: bool,
    unicode: bool,
    executing: bool,
}

impl IntrinsicObject for RegExpStringIterator {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
            .static_method(Self::next, js_string!("next"), 0)?
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("RegExp String Iterator"),
                Attribute::CONFIGURABLE,
            )?
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().regexp_string()
    }
}

// TODO: See todos in create_regexp_string_iterator and next.
impl RegExpStringIterator {
    fn new(matcher: &JsObject, string: JsString, global: bool, unicode: bool) -> Self {
        Self {
            matcher: Some(matcher.edge()),
            string,
            global,
            unicode,
            executing: false,
        }
    }

    /// `22.2.7.1 CreateRegExpStringIterator ( R, S, global, fullUnicode )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createregexpstringiterator
    pub(crate) fn create_regexp_string_iterator(
        matcher: JsObject,
        string: JsString,
        global: bool,
        unicode: bool,
        context: &mut Context,
    ) -> JsValue {
        // TODO: Implement this with closures and generators.
        //       For now all values of the closure are stored in RegExpStringIterator and the actual closure execution is in `.next()`.

        // 1. Assert: Type(S) is String.
        // 2. Assert: Type(global) is Boolean.
        // 3. Assert: Type(fullUnicode) is Boolean.

        // 4. Let closure be a new Abstract Closure with no parameters that captures R, S, global,
        //    and fullUnicode and performs the following steps when called:

        // 5. Return ! CreateIteratorFromClosure(closure, "%RegExpStringIteratorPrototype%", %RegExpStringIteratorPrototype%).

        let regexp_string_iterator = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .regexp_string(),
            Self::new(&matcher, string, global, unicode),
        );

        regexp_string_iterator.into()
    }

    /// `%RegExpStringIteratorPrototype%.next ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%regexpstringiteratorprototype%.next
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let iterator = this
            .as_object()
            .and_then(|object| object.downcast::<Self>().ok())
            .ok_or_else(|| {
                JsNativeError::typ().with_message("`this` is not a RegExpStringIterator")
            })?;
        let (matcher, string, global, unicode) = {
            let mut object = iterator.borrow_mut()?;
            let state = object.data_mut();
            if state.executing {
                return Err(JsNativeError::typ()
                    .with_message("RegExp String Iterator is already executing")
                    .into());
            }
            let Some(matcher) = &state.matcher else {
                drop(object);
                return Ok(create_iter_result_object_trusted(
                    JsValue::undefined(),
                    true,
                    context,
                ));
            };
            let matcher = JsObject::from_inner(matcher.clone());
            state.executing = true;
            (matcher, state.string.clone(), state.global, state.unicode)
        };

        let result = Self::advance(&matcher, &string, global, unicode, context);
        let mut object = iterator.borrow_mut()?;
        let state = object.data_mut();
        state.executing = false;
        if matches!(&result, Ok(None)) || (!global && result.is_ok()) {
            state.matcher = None;
        }
        drop(object);

        Ok(match result? {
            Some(m) => create_iter_result_object_trusted(m.into(), false, context),
            None => create_iter_result_object_trusted(JsValue::undefined(), true, context),
        })
    }

    fn advance(
        matcher: &JsObject,
        string: &JsString,
        global: bool,
        unicode: bool,
        context: &mut Context,
    ) -> JsResult<Option<JsObject>> {
        // TODO: This is the code that should be created as a closure in create_regexp_string_iterator.

        // i. Let match be ? RegExpExec(R, S).
        let m = RegExp::abstract_exec(matcher, string.clone(), context)?;

        if let Some(m) = m {
            // iii. If global is false, then
            if !global {
                // 1. Perform ? Yield(match).
                // 2. Return undefined.
                return Ok(Some(m));
            }

            // iv. Let matchStr be ? ToString(? Get(match, "0")).
            let m_str = m.get(0, context.access())?.to_string(context.access())?;

            // v. If matchStr is the empty String, then
            if m_str.is_empty() {
                // 1. Let thisIndex be ℝ(? ToLength(? Get(R, "lastIndex"))).
                let this_index = matcher
                    .get(js_string!("lastIndex"), context.access())?
                    .to_length(context.access())?;

                // 2. Let nextIndex be ! AdvanceStringIndex(S, thisIndex, fullUnicode).
                let next_index = advance_string_index(string, this_index, unicode);

                // 3. Perform ? Set(R, "lastIndex", 𝔽(nextIndex), true).
                matcher.set(js_string!("lastIndex"), next_index, true, context.access())?;
            }

            // vi. Perform ? Yield(match).
            Ok(Some(m))
        } else {
            // ii. If match is null, return undefined.
            Ok(None)
        }
    }
}
