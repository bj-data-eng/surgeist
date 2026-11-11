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

fn indexed<'scope, L: List>(
    scope: &mut Scope<'scope>,
    state: Rc<L>,
) -> Result<Value<'scope>, Error> {
    let prototype = L::INTERFACE.prototype(scope)?;
    let handlers = [
        handler(scope, &state, IndexedTrap::Get)?,
        handler(scope, &state, IndexedTrap::Has)?,
        handler(scope, &state, IndexedTrap::Descriptor)?,
        handler(scope, &state, IndexedTrap::Define)?,
        handler(scope, &state, IndexedTrap::Set)?,
        handler(scope, &state, IndexedTrap::Delete)?,
        handler(scope, &state, IndexedTrap::Keys)?,
        handler(scope, &state, IndexedTrap::PreventExtensions)?,
        handler(scope, &state, IndexedTrap::SetPrototype)?,
    ];
    let roots = handlers.each_ref().map(|value| &value.root);
    let root = scope.operations.indexed_proxy(&prototype.root, &roots)?;
    let owner = state.clone();
    let read = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            let index = arguments.get(0).ok_or(Error::BackendUnavailable)?;
            let IntegerValue::UnsignedLong(index) =
                scope.integer(index, IntegerKind::UnsignedLong, IntegerMode::Wrap)?
            else {
                return Err(Error::BackendUnavailable);
            };
            match owner.item(scope, index)? {
                Some(value) => Ok(Completion::Return(value)),
                None => nullable(scope),
            }
        }),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
    )?;
    scope.operations.project(&root, state, &[&read.root])?;
    Ok(scope.value(root))
}

fn handler<'scope, L: List>(
    scope: &mut Scope<'scope>,
    state: &Rc<L>,
    trap: IndexedTrap,
) -> Result<Value<'scope>, Error> {
    let state = state.clone();
    scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| dispatch(scope, state.as_ref(), trap, arguments)),
        FunctionMetadata {
            name: &[],
            length: trap.arity(),
        },
    )
}

fn dispatch<'scope, L: List>(
    scope: &mut Scope<'scope>,
    state: &L,
    trap: IndexedTrap,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    let host = &state.sheet().realm.host;
    host.work(scope, 1)?;
    let target = arguments.get(0).ok_or(Error::BackendUnavailable)?;
    match trap {
        IndexedTrap::PreventExtensions => return boolean(scope, false),
        IndexedTrap::Keys => {
            let length = state.length(scope)?;
            let (own_keys, own_length) = scope.operations.indexed_own_keys(&target.root)?;
            let count = length.checked_add(own_length).ok_or(Error::ArrayTooLong)?;
            host.work(scope, count as usize)?;
            let keys = scope.operations.indexed_keys(&own_keys, length)?;
            return Ok(Completion::Return(scope.value(keys)));
        }
        IndexedTrap::SetPrototype => {
            let candidate = arguments.get(1).ok_or(Error::BackendUnavailable)?;
            let mut current = candidate.root.clone();
            loop {
                use crate::backend::PrototypeStep;
                host.work(scope, 1)?;
                match scope
                    .operations
                    .indexed_prototype_step(&target.root, &current)?
                {
                    PrototypeStep::Complete => break,
                    PrototypeStep::Cycle => return boolean(scope, false),
                    PrototypeStep::Next(next) => current = next,
                }
            }
        }
        _ => {
            let key = arguments.get(1).ok_or(Error::BackendUnavailable)?;
            if let Some(index) = scope.operations.indexed_key(&key.root)? {
                match trap {
                    IndexedTrap::Define => return boolean(scope, false),
                    IndexedTrap::Get | IndexedTrap::Descriptor => {
                        if let Some(value) = state.item(scope, index)? {
                            return if matches!(trap, IndexedTrap::Descriptor) {
                                let descriptor =
                                    scope.operations.indexed_descriptor(&value.root)?;
                                Ok(Completion::Return(scope.value(descriptor)))
                            } else {
                                Ok(Completion::Return(value))
                            };
                        }
                    }
                    IndexedTrap::Has => {
                        if index < state.length(scope)? {
                            return boolean(scope, true);
                        }
                    }
                    IndexedTrap::Set => {
                        if index < state.length(scope)? {
                            return boolean(scope, false);
                        }
                    }
                    IndexedTrap::Delete => {
                        let supported = index < state.length(scope)?;
                        return boolean(scope, !supported);
                    }
                    _ => (),
                }
            }
        }
    }
    let mut roots = [&target.root; 4];
    let roots = roots
        .get_mut(..trap.arity() as usize)
        .ok_or(Error::BackendUnavailable)?;
    for (index, root) in roots.iter_mut().enumerate() {
        *root = &arguments.get(index).ok_or(Error::BackendUnavailable)?.root;
    }
    let root = scope.operations.indexed_fallback(trap, roots)?;
    Ok(Completion::Return(scope.value(root)))
}

fn boolean<'scope>(scope: &mut Scope<'scope>, value: bool) -> Result<Completion<'scope>, Error> {
    Ok(Completion::Return(
        scope.primitive(Primitive::Boolean(value))?,
    ))
}

// The list owns nine private trap closure payloads and one read closure. These
// targets share the list state. Their generic SDK/function bookkeeping is not
// attributed as a native CSSOM payload.
fn backing<L: List>(
    scope: &mut Scope<'_>,
    host: &L::Host,
) -> Result<<L::Host as StylesheetHost>::Storage, Error> {
    let bytes = scope
        .operations
        .projection_storage_bytes()
        .checked_add(size_of::<L>())
        .and_then(|bytes| bytes.checked_add(9 * size_of::<(Rc<L>, IndexedTrap)>()))
        .and_then(|bytes| bytes.checked_add(size_of::<Rc<L>>()))
        .ok_or(Error::Allocation)?;
    host.retain(scope, bytes)
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
    let root = scope
        .operations
        .projection_link(&arguments.receiver().root, 0)?;
    let resolver = scope.value(root);
    let index = scope.primitive(Primitive::Number(f64::from(index)))?;
    // The private function belongs to the list's creation realm. Borrowing a
    // prototype method across realms cannot change the returned wrapper realm.
    Ok(Completion::Return(scope.call(
        &resolver,
        arguments.receiver(),
        &[index],
    )?))
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
