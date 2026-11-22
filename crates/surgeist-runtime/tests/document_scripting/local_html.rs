use super::standard_window::{guest_limits, new_registry};
use super::*;
use std::num::NonZeroUsize;
use surgeist_html_parser::{HtmlParseLimits, HtmlSource, live::HtmlLiveParseProgress};

fn parse_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(128)
        .buffered_utf16_units(1024)
        .token_attributes(8)
        .auxiliary_entries(256)
        .managed_nodes(64)
        .managed_edges(64)
        .managed_attributes(16)
        .managed_utf16_units(1024)
        .source_map_entries(0)
        .source_map_payload_units(0)
        .diagnostics(0)
        .diagnostic_payload_units(0)
        .context_effects(0)
        .open_elements(16)
        .active_formatting_entries(0)
        .tree_depth(16)
        .managed_capacity_bytes(1_048_576)
        .peak_transient_bytes(1_048_576)
        .atomic_layout_bytes(65_536)
        .work_credits(1_048_576)
        .finish()
        .unwrap()
}

pub(super) fn window_listener_attaches_to_the_visible_prefix_before_eof() {
    let source = HtmlSource::utf16(
        "<!doctype html><html><head></head><body><div id=p>A😀&amp;β</div></body></html>".into(),
    );
    let cancellation = Arc::new(cancel());
    let mut session = session(limits());
    let mut hosted = session.hosted_execution();
    let mut registry = Some(new_registry());
    let (top, mut parser) = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancellation).unwrap();
        let top = hosted
            .create_local_html_top_level(&mut dom, host, &cancellation)
            .unwrap();
        let mut parser = hosted
            .begin_local_html_parser(&mut dom, top, &source, parse_limits(), &cancellation)
            .unwrap();
        assert_eq!(
            parser
                .advance(
                    &hosted,
                    &mut dom,
                    NonZeroUsize::new(7).unwrap(),
                    &cancellation
                )
                .unwrap(),
            HtmlLiveParseProgress::Paused
        );
        registry
            .as_mut()
            .unwrap()
            .bind(&mut dom, &cancellation)
            .unwrap();
        (top, parser)
    };
    let mut host = prepare(session, cancellation.clone())
        .enable_with_guest_event_registry(top, ALLOW, None, &mut registry, guest_limits())
        .unwrap();
    let permit = host.permit().unwrap();
    assert_eq!(permit.document(), top.document());
    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            globalThis.prefix = document.getElementById("p");
            globalThis.prefixText = prefix.firstChild;
            globalThis.observed = 0;
            window.addEventListener("parser-observation", function(event) {
                if (this === window && event.currentTarget === window) observed++;
            }, { once: true });
            return prefixText.data === "A" && prefix.ownerDocument === document;
        })())"#,
        1.0,
    );
    let before_eof = host
        .with_native(|dom, hosted, _| {
            parser
                .advance(&hosted, dom, NonZeroUsize::new(32).unwrap(), &cancellation)
                .unwrap()
        })
        .unwrap();
    assert_eq!(before_eof, HtmlLiveParseProgress::AwaitingEof);
    let handoff = host
        .with_native(|dom, hosted, _| {
            assert!(
                parser
                    .take_stop_parsing(&hosted, dom, &cancellation)
                    .unwrap()
                    .is_none()
            );
            parser.accept_eof();
            assert_eq!(
                parser
                    .advance(&hosted, dom, NonZeroUsize::MIN, &cancellation)
                    .unwrap(),
                HtmlLiveParseProgress::StopParsing
            );
            let handoff = parser
                .take_stop_parsing(&hosted, dom, &cancellation)
                .unwrap()
                .expect("the original EOF supplies one handoff");
            assert!(
                parser
                    .take_stop_parsing(&hosted, dom, &cancellation)
                    .unwrap()
                    .is_none()
            );
            handoff
        })
        .unwrap();
    assert_eq!(handoff.top_level(), top);
    assert_eq!(
        handoff.parser_continuation().document(),
        top.document().node()
    );
    assert_eq!(
        handoff.parser_continuation().eof().source(),
        source.identity()
    );
    assert_eq!(handoff.parser_continuation().eof().start(), source.len());
    assert!(handoff.parser_continuation().parser_scripts_are_absent());

    eval(
        &mut host,
        &permit,
        r#"Number((() => {
            window.dispatchEvent(new Event("parser-observation"));
            return observed === 1 && prefix === document.getElementById("p")
                && prefixText === prefix.firstChild && prefixText.data === "A😀&β";
        })())"#,
        1.0,
    );
    assert!(host.stop().into_session().is_ok());
}
