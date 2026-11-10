//! Shared engine-independent records for binding declarations and plans.
//!
//! These records do not implement platform objects. The eventual installer must
//! follow the pinned semantic sources, including receiver/security checks before
//! conversion, abrupt completion, and actual native continuation custody. A
//! `AccessMapping` distinguishes native work, derived projection, callback
//! invocation, forwarding, and external host authority. It is never evidence
//! that an adapter is implemented or an interface can already be installed.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DefinitionKind {
    Interface,
    Mixin,
    CallbackInterface,
    Dictionary,
    Enumeration,
    Typedef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NamedKind {
    Primitive,
    Interface,
    CallbackInterface,
    Dictionary,
    Enumeration,
    Typedef,
    ExternalInterface,
    ExternalMixin,
    ExternalCallback,
    ExternalTypedef,
    ExternalDictionary,
    ExternalType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum IdlType {
    Named {
        name: &'static str,
        kind: NamedKind,
    },
    Nullable(&'static IdlType),
    Sequence(&'static IdlType),
    Promise(&'static IdlType),
    Union(&'static [IdlType]),
    Annotated {
        inner: &'static IdlType,
        attributes: &'static [ExtendedAttribute],
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AttributeKind {
    Exposed,
    SameObject,
    NewObject,
    CEReactions,
    Unscopable,
    PutForwards,
    LegacyNullToEmptyString,
    LegacyUnenumerableNamedProperties,
    LegacyUnforgeable,
    LegacyOverrideBuiltIns,
    LegacyLenientThis,
    HTMLConstructor,
    Reflect,
    ReflectSetter,
    ReflectRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AttributeValue {
    None,
    Identifier(&'static str),
    Integers(&'static [i64]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ExtendedAttribute {
    pub kind: AttributeKind,
    pub value: AttributeValue,
}

/// Absence is distinct from a declared null or empty-dictionary default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DefaultValue {
    Absent,
    Null,
    EmptyDictionary,
    Boolean(bool),
    String(&'static str),
    Integer(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Presence {
    Required,
    Optional,
    Variadic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Argument {
    pub name: &'static str,
    pub ty: IdlType,
    pub presence: Presence,
    pub default: DefaultValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Special {
    None,
    Getter,
    Setter,
    Deleter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MemberKind {
    Attribute {
        ty: IdlType,
        readonly: bool,
    },
    Operation {
        result: IdlType,
        arguments: &'static [Argument],
        special: Special,
        is_static: bool,
    },
    Constructor {
        arguments: &'static [Argument],
    },
    Constant {
        ty: IdlType,
        value: i64,
    },
    Iterable {
        value: IdlType,
    },
    DictionaryMember {
        ty: IdlType,
        required: bool,
        default: DefaultValue,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Member {
    pub id: &'static str,
    pub name: Option<&'static str>,
    pub attributes: &'static [ExtendedAttribute],
    pub kind: MemberKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Definition {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: DefinitionKind,
    pub parent: Option<&'static str>,
    pub partial: bool,
    pub attributes: &'static [ExtendedAttribute],
    pub members: &'static [Member],
    pub values: &'static [&'static str],
    pub alias: Option<IdlType>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Includes {
    pub interface: &'static str,
    pub mixin: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct External {
    pub name: &'static str,
    pub kind: NamedKind,
    pub owner: &'static str,
    pub source: &'static str,
    pub requirement: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SourceUnit {
    pub id: &'static str,
    pub path: &'static str,
    pub anchor: &'static str,
    pub block: usize,
    pub sha256: &'static str,
    pub revision: &'static str,
    pub url: &'static str,
    pub entries: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Producer {
    pub owner: &'static str,
    pub path: &'static str,
    pub symbol: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Requirement {
    pub owner: &'static str,
    pub contract: &'static str,
    pub source: &'static str,
    pub issue: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AccessMapping<A, D> {
    /// Unique declaration-member plus constructor/getter/setter/call identity.
    pub access: &'static str,
    pub operation: AccessOperation<A, D>,
    pub result_view: ResultView,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AccessOperation<A, D> {
    /// One actual canonical-domain operation or its precise producer contract.
    Native {
        adapter: A,
        producers: &'static [Producer],
        requirements: &'static [Requirement],
    },
    /// An SDK-independent value/wrapper projection over real producer inputs.
    /// `owner` retains the semantic algorithm's authority; this record grants
    /// no permission to copy language algorithms into an SDK backend.
    Derived {
        algorithm: D,
        owner: &'static str,
        contract: &'static str,
        source: &'static str,
        inputs: &'static [Producer],
        requirements: &'static [Requirement],
    },
    /// Web IDL call-a-user-objects-operation, not a native handleEvent method:
    /// callable object or guest Get(operation), correct receiver/settings,
    /// conversion, abrupt completion and callback cleanup.
    CallbackInvocation {
        interface: &'static str,
        operation: &'static str,
        source: &'static str,
        requirements: &'static [Requirement],
    },
    /// Web IDL PutForwards setter: guest Get(receiver, attribute), object check,
    /// then guest Set(returned object, target, original value, false). Native
    /// authority is owned by the getter/target, not a fabricated native setter.
    PutForwards {
        getter: &'static str,
        target: &'static str,
        source: &'static str,
    },
    /// A specific external authority/service; no native method is invented.
    ExternalHost { requirement: Requirement },
}

/// Projection semantics belong to the returned operation, not its class: a
/// NodeList may be live childNodes or a static query result. SameObject/NewObject
/// identity remains independently declared by the standard extended attributes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResultView {
    Value,
    Live,
    Static,
    Sequence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Exposure {
    Window,
    All,
}

/// Requires the pinned legacy internal methods, not copied own data properties.
///
/// The installer must preserve named visibility/prototype collisions, numerical
/// index precedence, descriptor queries, special define/delete/preventExtensions,
/// key order, and distinct holder/Receiver semantics. A method receiver is brand
/// checked; an indexed own data-property observation is not an accessor call on
/// that Receiver. Native observations must remain current across guest reentry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LegacyProperties {
    pub indexed_getter: Option<&'static str>,
    pub named_getter: Option<&'static str>,
    pub override_builtins: bool,
    pub unenumerable_names: bool,
}

/// A declaration-derived plan. It installs nothing and fabricates no handlers.
///
/// Ordinary inherited members stay on the inherited prototype; unforgeable
/// descriptors from inclusive ancestor interfaces are copied onto each instance.
/// Included mixin members are own members of each including interface. External
/// mixins are explicit unresolved declaration requirements. Empty `constructors`
/// requires calls and construction attempts to throw, not an absent interface
/// object. The pinned Web IDL leaves its IsConstructor status unresolved; an
/// absent-[[Construct]] choice is a separate backend policy. Indexed
/// getter interfaces get the realm's Array.values @@iterator; only an explicit
/// value iterable adds entries/keys/values/forEach.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct InterfacePlan {
    pub name: &'static str,
    pub parent: Option<&'static str>,
    pub exposure: Exposure,
    pub own_members: &'static [&'static str],
    pub external_mixins: &'static [&'static str],
    pub constructors: &'static [&'static str],
    pub constructor_length: usize,
    pub properties: &'static [PropertyPlan],
    pub legacy: LegacyProperties,
    pub value_iterable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Placement {
    Interface,
    Prototype,
    Instance,
    InterfaceAndPrototype,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PropertyForm {
    Method { length: usize },
    Accessor { setter: bool },
    Constant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Descriptor {
    pub writable: bool,
    pub enumerable: bool,
    pub configurable: bool,
}

/// Method groups retain every overload; accessor functions have source-defined
/// `get name`/`set name` and lengths 0/1. An accessor descriptor has no writable
/// slot; `writable` here is used only for data descriptors. Methods are built-in
/// call-only functions. Legacy indexed/named properties are not these entries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PropertyPlan {
    pub name: &'static str,
    pub declarations: &'static [&'static str],
    pub placement: Placement,
    pub form: PropertyForm,
    pub descriptor: Descriptor,
    pub unscopable: bool,
}

/// Shared mandatory interface-object properties, not per-family invented flags.
/// Each interface function/prototype and these references belong to its realm;
/// inheritance selects the parent interface function and prototype respectively.
pub(super) const INTERFACE_OBJECT_PROPERTIES: InterfaceObjectProperties =
    InterfaceObjectProperties {
        global: Descriptor {
            writable: true,
            enumerable: false,
            configurable: true,
        },
        prototype: Descriptor {
            writable: false,
            enumerable: false,
            configurable: false,
        },
        prototype_constructor: Descriptor {
            writable: true,
            enumerable: false,
            configurable: true,
        },
        to_string_tag: Descriptor {
            writable: false,
            enumerable: false,
            configurable: true,
        },
        unscopables: Descriptor {
            writable: false,
            enumerable: false,
            configurable: true,
        },
    };

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct InterfaceObjectProperties {
    pub global: Descriptor,
    pub prototype: Descriptor,
    pub prototype_constructor: Descriptor,
    pub to_string_tag: Descriptor,
    pub unscopables: Descriptor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SemanticKind {
    InterfaceObjects,
    PropertyDescriptors,
    Brands,
    ArgumentConversion,
    LegacyPlatformObjects,
    Iterable,
    SameObject,
    NewObject,
    Unscopables,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SemanticSource {
    pub kind: SemanticKind,
    pub source: &'static str,
}
