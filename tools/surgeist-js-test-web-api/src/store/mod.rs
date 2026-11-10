//! Module containing the types related to the [`JsValueStore`].
use rustc_hash::FxHashMap;
use std::sync::Arc;
use surgeist_js::bigint::RawBigInt;
use surgeist_js::builtins::array_buffer::SharedArrayBuffer;
use surgeist_js::builtins::error::ErrorKind;
use surgeist_js::builtins::typed_array::TypedArrayKind;
use surgeist_js::value::TryIntoJs;
use surgeist_js::{ContextAccess, JsError, JsResult, JsString, JsValue, JsVariant, js_error};

mod from;
mod to;

/// Convenience method to avoid copy-pasting the same message.
#[inline]
fn unsupported_type() -> JsError {
    js_error!(Error: "DataCloneError: unsupported type for structured data")
}

#[inline]
fn unsupported_transfer() -> JsError {
    js_error!(TypeError: "Found an invalid value in transferList")
}

/// A type to help store [`JsString`]. Because [`JsString`] relies on [`std::rc::Rc`],
/// it cannot be `Send`, which is a necessary contract for the Store. The [`StringStore`]
/// can be transformed from and into `JsString`, but owns its data. It is _not_ copy-on-
/// write.
#[derive(Debug, Eq, PartialEq, Hash)]
struct StringStore(Vec<u16>);

impl StringStore {
    fn to_js_string(&self) -> JsString {
        JsString::from(self.0.as_slice())
    }
}

impl From<JsString> for StringStore {
    fn from(value: JsString) -> Self {
        Self(value.to_vec())
    }
}

impl From<StringStore> for JsString {
    fn from(value: StringStore) -> Self {
        JsString::from(value.0.as_slice())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug)]
struct StoredGraph {
    root: NodeId,
    nodes: Box<[StoredValue]>,
}

/// A snapshot node. Edges are indices within its owning graph.
#[derive(Debug)]
enum StoredValue {
    /// Primitive values - `null`.
    Null,

    /// Primitive values - `undefined`.
    Undefined,

    /// Primitive values - `Boolean`.
    Boolean(bool),

    /// Primitive values - `float64`. No need to store integers separately,
    /// they'll be checked when recreating the `JsValue`.
    Float(f64),

    /// [`JsString`]s are context-free, but not `Send`. Since we want to be
    /// `Send`, we'll have to make a copy of the data.
    String(StringStore),

    /// [`surgeist_js::JsBigInt`]s are context-free but not `Send`. The Raw version
    /// of it is, though.
    BigInt(RawBigInt),

    /// A dictionary of strings to values which should be reconstructed into
    /// a `JsObject`. Note: the prototype and constructor are not maintained,
    /// and during reconstruction the default `Object` prototype will be used.
    Object(Vec<(StringStore, NodeId)>),

    /// A `Map()` object in JavaScript.
    Map(Vec<(NodeId, NodeId)>),

    /// A `Set()` object in JavaScript. The elements are already unique at
    /// construction.
    Set(Vec<NodeId>),

    /// An `Array` object in JavaScript.
    Array(Vec<Option<NodeId>>),

    /// A `Date` object in JavaScript. Although this can be marshaled, it uses
    /// the system's datetime library to be reconstructed and may diverge.
    Date(f64),

    /// Allowed error types (see the structured clone algorithm page).
    #[expect(unused)]
    Error {
        kind: ErrorKind,
        name: StringStore,
        message: StringStore,
        stack: StringStore,
        cause: StringStore,
    },

    /// Regular expression. We store the expression and its flags. Everything else
    /// is reset. These are extracted as `String`, so we don't need to use the
    /// [`StringStore`] type.
    RegExp { source: String, flags: String },

    /// Array Buffer.
    ArrayBuffer(Vec<u8>),

    /// Shared Array Buffer.
    SharedArrayBuffer(SharedArrayBuffer),

    /// Dataview.
    #[expect(unused)]
    DataView {
        buffer: NodeId,
        byte_length: u64,
        byte_offset: u64,
    },

    /// Typed Array, including its kind and data.
    TypedArray {
        kind: TypedArrayKind,
        buffer: NodeId,
    },
}

/// A [`JsValue`]-like structure that can rebuild its value given any [`Context`].
/// It essentially stores the value itself and its original type. During
/// reconstruction, the constructors of the new [`Context`] will be used.
///
/// This follows the rules of the [structured clone algorithm][sca], but does not
/// require a [`Context`] to copy/move, and can be [`Send`] between threads.
///
/// It is not serializable as it allows recursive values.
///
/// To transform a [`JsValue`] into a [`JsValueStore`], the application MUST
/// pass in the context of the initial value. To transform it back to a
/// [`JsValue`], the application MUST pass the context that will contain
/// all prototypes for the new types (e.g. Object).
///
/// [sca]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm
#[derive(Debug, Clone)]
pub struct JsValueStore(Arc<StoredGraph>);

impl TryIntoJs for JsValueStore {
    fn try_into_js(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        to::decode(&self.0, context)
    }
}

impl JsValueStore {
    /// Create a context-free [`JsValue`] equivalent from an existing `JsValue` and the
    /// [`Context`] that was used to create it. The `transfer` argument allows for
    /// transferring ownership of the inner data to the context-free value instead of
    /// cloning it. By default, if a value isn't in the transfer vector, it is cloned.
    ///
    /// # Errors
    /// Any errors related to transferring or cloning a value's inner data.
    pub fn try_from_js(
        value: &JsValue,
        context: ContextAccess<'_>,
        transfer: Vec<JsValue>,
    ) -> JsResult<Self> {
        // Verify transfer entries and retain their roots throughout traversal.
        let transfer = transfer
            .into_iter()
            .map(|v| match v.variant() {
                JsVariant::Object(o) if from::is_transferable(&o) => Ok((o.identity(), o)),
                _ => Err(unsupported_transfer()),
            })
            .collect::<JsResult<FxHashMap<_, _>>>()?;

        Ok(Self(Arc::new(from::encode(value, &transfer, context)?)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_js::{Context, JsObject, Source, js_string};

    fn publish_clone(source: &str, context: &mut Context) {
        let original = context.eval(Source::from_bytes(source)).unwrap();
        let snapshot = JsValueStore::try_from_js(&original, context.access(), Vec::new()).unwrap();
        let cloned = snapshot.try_into_js(context.access()).unwrap();
        context
            .register_global_property(
                js_string!("cloned"),
                cloned,
                surgeist_js::property::Attribute::all(),
            )
            .unwrap();
    }

    #[test]
    fn reconstruction_preserves_self_mutual_cycles_and_shared_object_identity() {
        let mut context = Context::builder().build().unwrap();
        publish_clone(
            "const shared = { value: 42 }; const root = { left: shared, right: shared }; \
             const peer = { root, shared }; root.self = root; root.peer = peer; root",
            &mut context,
        );
        assert_eq!(
            context
                .eval(Source::from_bytes(
                    "cloned !== root && cloned.self === cloned && \
                     cloned.peer.root === cloned && cloned.left === cloned.right && \
                     cloned.left === cloned.peer.shared && cloned.left !== shared && \
                     cloned.left.value === 42",
                ))
                .unwrap(),
            JsValue::from(true)
        );
    }

    #[test]
    fn reconstruction_publishes_map_set_and_array_nodes_before_their_cycle_edges() {
        let mut context = Context::builder().build().unwrap();
        publish_clone(
            "const root = []; const map = new Map(); const set = new Set(); \
             root.push(map, set, root); map.set(root, set); set.add(root); set.add(map); root",
            &mut context,
        );
        assert_eq!(
            context
                .eval(Source::from_bytes(
                    "cloned !== root && cloned[2] === cloned && \
                     cloned[0].get(cloned) === cloned[1] && \
                     cloned[1].has(cloned) && cloned[1].has(cloned[0]) && \
                     cloned[0] !== map && cloned[1] !== set",
                ))
                .unwrap(),
            JsValue::from(true)
        );
    }

    #[test]
    fn reconstruction_sets_date_time_and_preserves_shared_date_identity() {
        let mut context = Context::builder().build().unwrap();
        publish_clone(
            "const date = new Date(12345); ({ left: date, right: date, invalid: new Date(NaN) })",
            &mut context,
        );
        assert_eq!(
            context
                .eval(Source::from_bytes(
                    "cloned.left === cloned.right && cloned.left !== date && \
                     cloned.left.getTime() === 12345 && Number.isNaN(cloned.invalid.getTime())",
                ))
                .unwrap(),
            JsValue::from(true)
        );
    }

    #[test]
    fn dropping_the_last_snapshot_releases_a_cycle() {
        let mut context = Context::builder().build().unwrap();
        let original = context
            .eval(Source::from_bytes(
                "const node = {}; node.self = node; node",
            ))
            .unwrap();
        let snapshot = JsValueStore::try_from_js(&original, context.access(), Vec::new()).unwrap();
        let weak = Arc::downgrade(&snapshot.0);
        let cloned = snapshot
            .try_into_js(context.access())
            .unwrap()
            .as_object()
            .unwrap();
        let edge = cloned
            .get(js_string!("self"), context.access())
            .unwrap()
            .as_object()
            .unwrap();
        assert!(JsObject::equals(&edge, &cloned));

        drop(snapshot);
        assert!(
            weak.upgrade().is_none(),
            "the cyclic snapshot is still owned"
        );
    }
}
