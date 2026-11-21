//! Textarea value edits preserve the first Source refusal and canonical state.

use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::DomSourceDriveError,
};
use surgeist_html::values::DocumentString;

pub(super) fn textarea_values_preserve_raw_default_and_caret_on_source_refusal() {
    let raw = DocumentString::from_utf16(vec![65, 13, 10, 0xd800, 0, 0xdc00]);
    let default = DocumentString::from("initial\r\ndefault");
    for script in [
        "limitedTextarea.value = 'next'",
        "limitedTextarea.defaultValue = 'next'",
    ] {
        let cancellation = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancellation, 0);
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(target.document(), Namespace::html(), "textarea");
        batch.set_textarea_default_value(&subject, default.clone());
        batch.set_textarea_value(&subject, raw.clone());
        let subject = session
            .dom()
            .apply(batch, &cancellation)
            .unwrap()
            .created(&subject)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancellation, 200_000, 0);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, subject, "limitedTextarea");
        let before = host.snapshot().unwrap();
        let original = before.node(subject).unwrap();
        let state = original.as_textarea().unwrap().state();
        assert!(state.flags().dirty_value());
        assert_eq!(state.raw_value(), &raw);
        assert_eq!(state.selection().cursor_position(), Some(5));
        let [child] = original.children() else {
            panic!("expected a single native default Text child");
        };
        assert_eq!(before.node(*child).unwrap().data(), Some(&default));

        let first = host.in_document(&permit, |scope| evaluate(scope, script).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = first else {
            panic!("expected the original Textarea Source refusal for {script}: {first:?}");
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::SourceWorkLimit { limit: 0 }
            )))
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        let node = after.node(subject).unwrap();
        assert_eq!(node.as_textarea().unwrap().state(), state);
        assert_eq!(node.children(), &[*child]);
        assert_eq!(after.node(*child).unwrap().data(), Some(&default));
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}
