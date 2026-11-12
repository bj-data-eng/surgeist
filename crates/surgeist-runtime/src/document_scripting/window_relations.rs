//! HTML Window relations for the original native no-opener top-level.

use std::{
    mem::size_of,
    rc::{Rc, Weak},
};

use surgeist_bindings_js::{
    Arguments, Callback, Completion, Error, FunctionMetadata, Primitive, PropertyDescriptor,
    PropertyKey, Scope, ValueKind,
};
use surgeist_dom::{DomHostedError, DomHostedRuntimeRetention};
use surgeist_html_semantics::HtmlScriptingStatus;

use super::{DocumentScriptNativeError, DocumentScriptPermit, NativeOwner, refuse};
use crate::{DomHostedRuntimeError, DomSession};

const fn ascii<const N: usize>(bytes: &[u8; N]) -> [u16; N] {
    let mut units = [0; N];
    let mut source: &[u8] = bytes;
    let mut target: &mut [u16] = &mut units;
    while let ([byte, remaining @ ..], [unit, pending @ ..]) = (source, target) {
        *unit = *byte as u16;
        source = remaining;
        target = pending;
    }
    units
}

enum Relation {
    WindowProxy,
    Opener,
}

struct Member {
    key: &'static [u16],
    get: &'static [u16],
    set: Option<&'static [u16]>,
    relation: Relation,
}

const MEMBERS: &[Member] = &[
    Member {
        key: &ascii(b"window"),
        get: &ascii(b"get window"),
        set: None,
        relation: Relation::WindowProxy,
    },
    Member {
        key: &ascii(b"self"),
        get: &ascii(b"get self"),
        set: Some(&ascii(b"set self")),
        relation: Relation::WindowProxy,
    },
    Member {
        key: &ascii(b"opener"),
        get: &ascii(b"get opener"),
        set: Some(&ascii(b"set opener")),
        relation: Relation::Opener,
    },
    Member {
        key: &ascii(b"parent"),
        get: &ascii(b"get parent"),
        set: Some(&ascii(b"set parent")),
        relation: Relation::WindowProxy,
    },
];

struct WindowRelations {
    native: Weak<NativeOwner>,
    permit: DocumentScriptPermit,
    _backing: DomHostedRuntimeRetention,
}

struct Access {
    host: Rc<WindowRelations>,
    member: &'static Member,
}

pub(super) fn storage_bytes() -> usize {
    let callbacks = MEMBERS
        .iter()
        .map(|member| 1 + usize::from(member.set.is_some()))
        .sum::<usize>();
    size_of::<WindowRelations>() + callbacks * size_of::<Access>()
}

pub(super) fn installation_work() -> usize {
    MEMBERS
        .iter()
        .map(|member| member.key.len() + member.get.len() + member.set.map_or(0, <[u16]>::len))
        .sum()
}

pub(super) fn install(
    scope: &mut Scope<'_>,
    native: Weak<NativeOwner>,
    permit: DocumentScriptPermit,
    backing: DomHostedRuntimeRetention,
) -> Result<(), Error> {
    let host = Rc::new(WindowRelations {
        native,
        permit,
        _backing: backing,
    });
    let global = scope.global()?;
    for member in MEMBERS {
        let getter = Access {
            host: host.clone(),
            member,
        };
        let get = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| getter.call(scope, arguments, false)),
            FunctionMetadata {
                name: member.get,
                length: 0,
            },
        )?;
        let set = member
            .set
            .map(|name| {
                let setter = Access {
                    host: host.clone(),
                    member,
                };
                scope.fixed_arity_function(
                    &Callback::new(move |scope, arguments| setter.call(scope, arguments, true)),
                    FunctionMetadata { name, length: 1 },
                )
            })
            .transpose()?;
        scope.define_new_property(
            &global,
            PropertyKey::String(member.key),
            PropertyDescriptor::Accessor {
                get: Some(&get),
                set: set.as_ref(),
                enumerable: true,
                configurable: member.set.is_some(),
            },
        )?;
    }
    Ok(())
}

impl Access {
    fn call<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        setter: bool,
    ) -> Result<Completion<'scope>, Error> {
        // V8 gives API callbacks their creation realm's global proxy for a
        // null or undefined this value, matching the Web IDL accessor default.
        let global = scope.global()?;
        if !scope.strict_equals(arguments.receiver(), &global)? {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Not a Window object"))?,
            ));
        }

        let native = self.host.native.upgrade().ok_or(Error::ExpiredIdentity)?;
        let attachment = native.attachment.get().ok_or(Error::ExpiredIdentity)?;
        let permit = &self.host.permit;
        if scope.realm_authority()?.identity() != permit.realm
            || attachment.realm.identity() != permit.realm
            || attachment.target.document() != permit.document
        {
            return refuse(
                scope,
                &attachment.failures,
                DocumentScriptNativeError::ForeignPermit,
            );
        }

        let undefined;
        let mut replacement = None;
        if setter {
            let value = match arguments.get(0) {
                Some(value) => value,
                None => {
                    undefined = scope.primitive(Primitive::Undefined)?;
                    &undefined
                }
            };
            if !matches!(self.member.relation, Relation::Opener)
                || scope.kind(value)? != ValueKind::Null
            {
                replacement = Some(value);
            }
        }

        let result = (|| {
            let mut session = native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            qualify(&native, attachment, &mut session)?;
            if replacement.is_some() {
                session.hosted_execution().admit_script_work(
                    &session.dom(),
                    self.member.key.len(),
                    &native.cancellation,
                )?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            return refuse(scope, &attachment.failures, error);
        }

        if let Some(value) = replacement {
            return match scope.define_property(
                &global,
                PropertyKey::String(self.member.key),
                PropertyDescriptor::Data {
                    value,
                    writable: true,
                    enumerable: true,
                    configurable: true,
                },
            ) {
                Ok(()) => Ok(Completion::Undefined),
                Err(Error::PropertyRejected) => Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Cannot replace Window attribute"))?,
                )),
                Err(error) => Err(error),
            };
        }
        if setter {
            // Disowning this live no-opener context leaves its opener absent
            // and preserves the accessor.
            return Ok(Completion::Undefined);
        }

        // The canonical attached navigable is this parentless top-level.
        // Its actual browsing context was produced without an opener.
        match self.member.relation {
            Relation::WindowProxy => Ok(Completion::Return(global)),
            Relation::Opener => Ok(Completion::Return(scope.primitive(Primitive::Null)?)),
        }
    }
}

fn qualify(
    native: &NativeOwner,
    attachment: &super::Attachment,
    session: &mut DomSession,
) -> Result<(), DocumentScriptNativeError> {
    let status = match native.active_event_request() {
        Some(request) => native.qualify_event(attachment, session, &request)?,
        None => native.observe(attachment, session)?.status(),
    };
    if status != HtmlScriptingStatus::Enabled {
        return Err(DomHostedRuntimeError::Dom(DomHostedError::ScriptingDisabled).into());
    }
    Ok(())
}
