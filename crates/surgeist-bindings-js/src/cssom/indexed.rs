//! Closed indexed legacy-platform-object behavior for the two native lists.

use super::*;

pub(super) trait List: Sized + 'static {
    type Host: StylesheetHost;
    const INTERFACE: Interface;

    fn index() -> Argument<IntegerType>;

    fn sheet(&self) -> &Sheet<Self::Host>;
    fn length(&self, scope: &mut Scope<'_>) -> Result<u32, Error>;
    fn item<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        index: u32,
    ) -> Result<Option<Value<'scope>>, Error>;
}

fn extent(length: usize) -> Result<u32, Error> {
    u32::try_from(length).map_err(|_| Error::ArrayTooLong)
}

const ITEM_READER: u32 = 0;

pub(super) fn state<'scope, L: List>(
    scope: &mut Scope<'scope>,
    value: &Value<'scope>,
) -> Result<Option<Rc<L>>, Error> {
    Ok(scope
        .native_object::<Rc<L>>(value)?
        .map(|native| native.payload().clone()))
}

pub(super) fn brand<'scope, L: List>(
    scope: &mut Scope<'scope>,
    value: &Value<'scope>,
) -> Result<Rc<L>, Error> {
    state(scope, value)?.ok_or_else(|| type_error(scope))
}

fn indexed<'scope, L: List>(
    scope: &mut Scope<'scope>,
    state: Rc<L>,
) -> Result<Value<'scope>, Error> {
    let prototype = L::INTERFACE.prototype(scope)?;
    let read = scope.bounded_function(
        &Callback::new(read_index::<L>),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
        2,
    )?;
    let keys = scope.fixed_arity_function(
        &Callback::new(keys::<L>),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
    )?;
    let bytes = scope.native_projection_bytes::<Rc<L>, <L::Host as StylesheetHost>::Storage>()?;
    let backing = state.sheet().realm.host.retain(scope, bytes)?;
    let object = scope.project_native_object_with_collection_properties(
        ObjectPrototype::Object(&prototype),
        state,
        &read,
        &keys,
        |_| Ok::<_, Error>(backing),
    )?;
    scope.set_native_object_reference::<Rc<L>>(&object, ITEM_READER, Some(&read))?;
    Ok(object)
}

fn read_index<'scope, L: List>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<L>(scope, arguments.receiver())?;
    state.sheet().realm.host.work(scope, 1)?;
    let key = arguments.get(0).ok_or(Error::BackendUnavailable)?;
    let presence_only = arguments.get(1).is_some();
    let index = if scope.kind(key)? == ValueKind::Number {
        let IntegerValue::UnsignedLong(index) =
            scope.integer(key, IntegerKind::UnsignedLong, IntegerMode::Wrap)?
        else {
            return Err(Error::BackendUnavailable);
        };
        Some(index)
    } else {
        None
    };
    if presence_only {
        let present = match index {
            Some(index) => index < state.length(scope)?,
            None => false,
        };
        return Ok(Completion::Return(
            scope.primitive(Primitive::Boolean(present))?,
        ));
    }
    match index {
        Some(index) => Ok(state
            .item(scope, index)?
            .map_or(Completion::Undefined, Completion::Return)),
        None => Ok(Completion::Undefined),
    }
}

fn keys<'scope, L: List>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<L>(scope, arguments.receiver())?;
    let host = &state.sheet().realm.host;
    host.work(scope, 1)?;
    if arguments.get(0).is_some() {
        return Ok(Completion::Return(scope.array(&[])?));
    }
    let length = state.length(scope)?;
    let bytes = (length as usize)
        .checked_mul(size_of::<Value<'scope>>())
        .ok_or(Error::Allocation)?;
    let _backing = host.retain(scope, bytes)?;
    host.work(scope, length as usize)?;
    let mut indices = Vec::new();
    indices
        .try_reserve_exact(length as usize)
        .map_err(|_| Error::Allocation)?;
    for index in 0..length {
        indices.push(scope.primitive(Primitive::Number(f64::from(index)))?);
    }
    Ok(Completion::Return(scope.array(&indices)?))
}

fn backing<L: List>(
    scope: &mut Scope<'_>,
    host: &L::Host,
) -> Result<<L::Host as StylesheetHost>::Storage, Error> {
    host.retain(scope, size_of::<L>())
}

pub(super) fn length<'scope, L: List>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<L>(scope, arguments.receiver())?;
    let value = state.length(scope)?;
    number(scope, value)
}

pub(super) fn item<'scope, L: List>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let state = brand::<L>(scope, arguments.receiver())?;
    let index = integer_argument(&state.sheet().realm.account, scope, &arguments, &L::index())?;
    let resolver = scope
        .native_object_reference::<Rc<L>>(arguments.receiver(), ITEM_READER)?
        .ok_or(Error::ExpectedNativeObject)?;
    let index = scope.primitive(Primitive::Number(f64::from(index)))?;
    // The private reader belongs to the list's creation realm. A borrowed
    // prototype method still creates any returned wrapper in that realm.
    let value = scope.call(&resolver, arguments.receiver(), &[index])?;
    if scope.kind(&value)? == ValueKind::Undefined {
        nullable(scope)
    } else {
        Ok(Completion::Return(value))
    }
}

impl<H: StylesheetHost> Rules<H> {
    pub(super) fn create<'scope>(
        scope: &mut Scope<'scope>,
        sheet: Rc<Sheet<H>>,
    ) -> Result<Value<'scope>, Error> {
        let backing = backing::<Self>(scope, &sheet.realm.host)?;
        indexed(
            scope,
            Rc::new(Self {
                sheet,
                native: OnceCell::new(),
                cache: RefCell::new(RuleCache {
                    entries: Vec::new(),
                    backing: None,
                }),
                _backing: backing,
            }),
        )
    }

    pub(super) fn bind(&self, native: &CssRuleList) {
        self.native.get_or_init(|| native.clone());
    }

    fn capture(&self, scope: &mut Scope<'_>) -> Result<CssomCapture<CssRuleList>, Error> {
        let sheet = self.sheet.capture(scope)?;
        let native = self.native.get().ok_or(Error::BackendUnavailable)?;
        self.sheet.result(scope, native.capture(sheet.snapshot()))
    }

    fn wrapper<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        capture: CssomCapture<CssRule>,
    ) -> Result<Value<'scope>, Error> {
        let id = capture.handle().id();
        let known = {
            let cache = self
                .cache
                .try_borrow()
                .map_err(|_| Error::RegistryConflict)?;
            let mut found = None;
            for (known, identity) in &cache.entries {
                self.sheet.realm.host.work(scope, 1)?;
                if known == id {
                    found = Some(*identity);
                    break;
                }
            }
            found
        };
        if let Some(identity) = known
            && let Some(root) = scope.operations.reopen_projection(identity)?
        {
            return Ok(scope.value(root));
        }
        let backing = project_backing::<Rule<H>, H>(scope, &self.sheet.realm.host)?;
        let prototype = Interface::Rule.prototype(scope)?;
        let object = scope.object(ObjectPrototype::Object(&prototype))?;
        let identity = scope.operations.project(
            &object.root,
            Rc::new(Rule {
                sheet: self.sheet.clone(),
                rule: capture.handle().clone(),
                _backing: backing,
            }),
            &[],
        )?;
        self.cache_identity(scope, id, identity)?;
        Ok(object)
    }

    fn cache_identity(
        &self,
        scope: &mut Scope<'_>,
        id: &CssomRuleId,
        identity: Projection,
    ) -> Result<(), Error> {
        let mut cache = self
            .cache
            .try_borrow_mut()
            .map_err(|_| Error::RegistryConflict)?;
        let host = &self.sheet.realm.host;
        // Reclaim collected entries before growing the live list's cache.
        // A detached but reachable wrapper still owns its identity.
        let mut cursor = 0;
        while cursor < cache.entries.len() {
            host.work(scope, 1)?;
            let (_, identity) = cache.entries.get(cursor).ok_or(Error::RegistryConflict)?;
            if scope.operations.projection_is_live(*identity)? {
                cursor += 1;
            } else {
                host.work(scope, 1)?;
                cache.entries.swap_remove(cursor);
            }
        }
        if cache.entries.len() == cache.entries.capacity() {
            let capacity = cache.entries.capacity().saturating_mul(2).max(4);
            let bytes = capacity
                .checked_mul(size_of::<(CssomRuleId, Projection)>())
                .ok_or(Error::Allocation)?;
            let backing = host.retain(scope, bytes)?;
            host.work(scope, cache.entries.len())?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(capacity)
                .map_err(|_| Error::Allocation)?;
            replacement.append(&mut cache.entries);
            cache.entries = replacement;
            cache.backing = Some(backing);
        }
        cache.entries.push((id.clone(), identity));
        Ok(())
    }
}

impl<H: StylesheetHost> List for Rules<H> {
    type Host = H;
    const INTERFACE: Interface = Interface::Rules;

    fn index() -> Argument<IntegerType> {
        crate::conversion::declarations::css_rule_list_item_0_index()
    }

    fn sheet(&self) -> &Sheet<H> {
        &self.sheet
    }

    fn length(&self, scope: &mut Scope<'_>) -> Result<u32, Error> {
        let rules = self.capture(scope)?;
        extent(self.sheet.result(scope, rules.length())?)
    }

    fn item<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        index: u32,
    ) -> Result<Option<Value<'scope>>, Error> {
        let rules = self.capture(scope)?;
        self.sheet
            .result(scope, rules.item(index as usize))?
            .map(|capture| self.wrapper(scope, capture))
            .transpose()
    }
}

impl<H: StylesheetHost> Media<H> {
    pub(super) fn create<'scope>(
        scope: &mut Scope<'scope>,
        sheet: Rc<Sheet<H>>,
    ) -> Result<Value<'scope>, Error> {
        let backing = backing::<Self>(scope, &sheet.realm.host)?;
        indexed(
            scope,
            Rc::new(Self {
                sheet,
                native: OnceCell::new(),
                _backing: backing,
            }),
        )
    }

    pub(super) fn bind(&self, native: &MediaList) {
        self.native.get_or_init(|| native.clone());
    }

    pub(super) fn capture(&self, scope: &mut Scope<'_>) -> Result<CssomCapture<MediaList>, Error> {
        let sheet = self.sheet.capture(scope)?;
        let native = self.native.get().ok_or(Error::BackendUnavailable)?;
        self.sheet.result(scope, native.capture(sheet.snapshot()))
    }
}

impl<H: StylesheetHost> List for Media<H> {
    type Host = H;
    const INTERFACE: Interface = Interface::Media;

    fn index() -> Argument<IntegerType> {
        crate::conversion::declarations::media_list_item_0_index()
    }

    fn sheet(&self) -> &Sheet<H> {
        &self.sheet
    }

    fn length(&self, scope: &mut Scope<'_>) -> Result<u32, Error> {
        let media = self.capture(scope)?;
        extent(self.sheet.result(scope, media.length())?)
    }

    fn item<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        index: u32,
    ) -> Result<Option<Value<'scope>>, Error> {
        let media = self.capture(scope)?;
        self.sheet
            .result(
                scope,
                media.item(index as usize, CssSerializationLimits::default()),
            )?
            .map(|text| js_text(scope, &self.sheet.realm, &text))
            .transpose()
    }
}
