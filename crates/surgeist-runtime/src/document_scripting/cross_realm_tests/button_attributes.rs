//! Button setters preserve the first Source refusal and the original attributes.

use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::DomSourceDriveError,
};
use surgeist_html::values::DocumentString;

pub(super) fn button_setters_preserve_native_attributes_on_source_refusal() {
    let attributes = [
        ("command", "ClOsE"),
        ("type", "unrecognized"),
        ("formmethod", "PoSt"),
        ("formenctype", "MuLtipart/Form-Data"),
        ("disabled", "authored"),
    ];
    for script in [
        "limitedButton.command = '--next'",
        "limitedButton.type = 'reset'",
        "limitedButton.formMethod = 'get'",
        "limitedButton.formEnctype = 'text/plain'",
        "limitedButton.disabled = false",
    ] {
        let cancellation = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancellation, 0);
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(target.document(), Namespace::html(), "button");
        for (name, value) in attributes {
            batch.set_attribute_ns(&subject, Namespace::none(), name, value);
        }
        let subject = session
            .dom()
            .apply(batch, &cancellation)
            .unwrap()
            .created(&subject)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancellation, 200_000, 0);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, subject, "limitedButton");
        let stamp = host.snapshot().unwrap().stamp();
        let first = host.in_document(&permit, |scope| evaluate(scope, script).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = first else {
            panic!("expected the original Button Source refusal for {script}: {first:?}");
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::SourceWorkLimit { limit: 0 }
            )))
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), stamp);
        let node = after.node(subject).unwrap();
        for (name, value) in attributes {
            assert_eq!(
                node.attribute_ns(&Namespace::none(), &name.into()),
                Some(&DocumentString::from(value))
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}
