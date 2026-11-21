//! A Form reflection refusal leaves the original canonical attribute state.

use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::DomSourceDriveError,
};

pub(super) fn form_setters_preserve_native_attributes_on_source_refusal() {
    for script in [
        "limitedForm.acceptCharset = 'changed'",
        "limitedForm.encoding = 'text/plain'",
        "limitedForm.noValidate = false",
    ] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(target.document(), Namespace::html(), "form");
        batch.set_attribute_ns(&subject, Namespace::none(), "accept-charset", "UTF-8");
        batch.set_attribute_ns(
            &subject,
            Namespace::none(),
            "enctype",
            "MuLtipart/Form-Data",
        );
        batch.set_attribute_ns(&subject, Namespace::none(), "novalidate", "authored");
        let subject = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&subject)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, subject, "limitedForm");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| evaluate(scope, script).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!(
                "native Form must return the original Source work refusal for {script}: {result:?}"
            );
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::SourceWorkLimit { limit: 0 }
            )))
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        let form = after.node(subject).unwrap();
        assert_eq!(
            form.attribute(&"accept-charset".into()),
            Some(&"UTF-8".into())
        );
        assert_eq!(
            form.attribute(&"enctype".into()),
            Some(&"MuLtipart/Form-Data".into())
        );
        assert_eq!(
            form.attribute(&"novalidate".into()),
            Some(&"authored".into())
        );
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}
