//! Canvas IDL dimensions retain the first Source refusal and canonical bitmap.

use super::*;
use crate::internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError};
use surgeist_dom::{DomBatch, DomError};
use surgeist_html::{elements::media::HtmlCanvasBitmapDimension, values::Namespace};

pub(in crate::document_scripting) fn canvas_dimension_write_preserves_original_source_refusal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let canvas = batch.create_element_ns(target.document(), Namespace::html(), "canvas");
    let raw_width = "1844674407370955161616";
    batch.set_attribute_ns(&canvas, Namespace::none(), "width", raw_width);
    let canvas = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&canvas)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, canvas, "canvasProbe");
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "canvasProbe.width === 300 && canvasProbe.height === 150",
        )
    })
    .unwrap();
    let result = host.in_document(&permit, |scope| {
        scope.evaluate(&units("canvasProbe.width = 17")).map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected the original Canvas Source refusal: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        ))),
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    let node = after.node(canvas).unwrap();
    assert_eq!(node.attribute(&"width".into()), Some(&raw_width.into()));
    let state = node.as_element().unwrap().canvas_none_state().unwrap();
    assert!(matches!(
        state.width(), HtmlCanvasBitmapDimension::Parsed(value)
            if value.decimal_digits() == raw_width
    ));
    assert!(matches!(
        state.height(),
        HtmlCanvasBitmapDimension::Default(150)
    ));
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
