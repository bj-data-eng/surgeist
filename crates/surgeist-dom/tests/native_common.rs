//! Source oracles: DOM3071 §1.4/createElementNS; HTML877d native common
//! initialization and §2.5.6 nonce attribute/IDL/insertion/cloning clauses.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomError,
    edit::DomChange,
    limits::{DomLimits, DomResource},
    shadow::{DomFlatRelation, DomShadowRootInit, DomSlotQueryLimits},
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress},
};
use surgeist_html::{
    construction::HtmlShadowRootMode,
    elements::native::HtmlCustomElementState,
    values::{DocumentString, Namespace},
};
use surgeist_html::{
    elements::media::{HtmlInitialMediaState, HtmlMediaResourceEffect},
    security::nonce::HtmlBrowsingContextPresence,
};

const HTML: &str = "http://www.w3.org/1999/xhtml";
const SVG: &str = "http://www.w3.org/2000/svg";
const MATH: &str = "http://www.w3.org/1998/Math/MathML";
fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            16384,
            256,
            0,
            0,
            16,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

#[test]
fn exact_expanded_names_select_common_state_without_parser_folding() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let cases = [
        (HTML, "p:div", true, true),
        (HTML, "Input", true, true),
        (HTML, "x-widget", true, true),
        (HTML, "X-widget", true, true),
        (HTML, "annotation-xml", true, true),
        (SVG, "input", false, true),
        (SVG, "Script", false, true),
        (MATH, "template", false, true),
        ("urn:example", "input", false, false),
        ("http://www.w3.org/1999/XHTML", "input", false, false),
    ];
    let tickets: Vec<_> = cases
        .iter()
        .map(|(ns, name, _, _)| batch.create_element_ns(&document, Namespace::from_uri(*ns), *name))
        .collect();
    let commit = apply(&mut dom, batch);
    let capture = dom.snapshot();
    let environment = capture
        .document(commit.document(&document).unwrap())
        .unwrap()
        .metadata()
        .document_environment();
    assert_eq!(
        environment.browsing_context(),
        HtmlBrowsingContextPresence::Absent
    );
    assert!(!environment.has_ever_had_navigable());
    assert!(
        !capture
            .document(commit.document(&document).unwrap())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    for ((ns, name, html, nonce), ticket) in cases.iter().zip(&tickets) {
        let node = capture
            .node(commit.created(ticket).unwrap().unwrap())
            .unwrap();
        let element = node.as_element().unwrap();
        assert_eq!(node.node_name(), DocumentString::from(*name));
        assert_eq!(
            element.name().namespace_uri(),
            Namespace::from_uri(*ns).uri()
        );
        assert_eq!(element.common_state().html().is_some(), *html);
        assert_eq!(
            element.nonce(),
            nonce.then(|| DocumentString::from("")).as_ref()
        );
        let common = element.common_state().element();
        assert!(common.reaction_queue_is_empty());
        assert!(!common.click_in_progress());
        if let Some(common) = element.common_state().html() {
            assert!(!common.has_previously_focused_element());
            assert!(!common.has_attached_internals());
            assert!(common.popover_is_hidden());
            assert!(!common.has_popover_trigger());
            assert!(!common.popover_is_hiding());
            assert!(!common.has_popover_toggle_task_tracker());
            assert!(!common.has_popover_close_watcher());
            assert!(!common.has_popover_opened_in_mode());
        }
    }
    let prefixed = capture
        .node(commit.created(&tickets[0]).unwrap().unwrap())
        .unwrap();
    assert_eq!(
        prefixed.as_element().unwrap().name().prefix(),
        Some(&"p".into())
    );
    assert_eq!(
        prefixed.as_element().unwrap().name().local_name(),
        &"div".into()
    );
    assert_eq!(
        capture
            .node(commit.created(&tickets[2]).unwrap().unwrap())
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_state(),
        HtmlCustomElementState::Undefined
    );
    assert_eq!(
        capture
            .node(commit.created(&tickets[5]).unwrap().unwrap())
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_state(),
        HtmlCustomElementState::Uncustomized
    );
    for index in [3, 4] {
        assert_eq!(
            capture
                .node(commit.created(&tickets[index]).unwrap().unwrap())
                .unwrap()
                .as_element()
                .unwrap()
                .custom_element_state(),
            HtmlCustomElementState::Uncustomized
        );
    }
}

#[test]
fn namespace_element_validation_uses_element_context_and_preserves_utf16() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let unusual = DocumentString::from_utf16(vec![0x70, 0x3a, 0x61, 0xd800, 0x3d]);
    let element = batch.create_element_ns(&document, Namespace::from_uri("urn:x"), unusual.clone());
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    assert_eq!(dom.snapshot().node(element).unwrap().node_name(), unusual);
    for (ns, name, error) in [
        (Namespace::none(), "p:1", DomError::InvalidCharacter),
        (Namespace::none(), "p:a", DomError::Namespace),
        (
            Namespace::from_uri("urn:x"),
            ":a",
            DomError::InvalidCharacter,
        ),
        (Namespace::from_uri("urn:x"), "xml:a", DomError::Namespace),
        (Namespace::from_uri("urn:x"), "xmlns", DomError::Namespace),
    ] {
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.create_element_ns(document, ns, name);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(element, Namespace::from_uri("urn:x"), "p:a=", "x");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidCharacter
    );
}

#[test]
fn fresh_native_embed_retains_inactive_attributes_through_clone_and_retirement() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let embed = batch.create_element_ns(&document, Namespace::html(), "embed");
    batch.set_attribute(&embed, "src", "relative.svg");
    batch.set_attribute(&embed, "type", "image/svg+xml");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let embed = commit.created(&embed).unwrap().unwrap();
    let captured = dom.snapshot();
    let node = captured.node(embed).unwrap();
    assert_eq!(node.node_name(), "embed".into());
    assert!(node.as_element().unwrap().is_inactive_embed());
    assert_eq!(node.owner_document(), Some(document));
    assert!(!node.is_connected());
    assert_eq!(node.attribute(&"src".into()), Some(&"relative.svg".into()));

    let mut batch = DomBatch::new();
    let copy = batch.clone_node(embed, false);
    batch.destroy(embed);
    let copy = apply(&mut dom, batch).created(&copy).unwrap().unwrap();
    assert_ne!(copy, embed);
    let current = dom.snapshot();
    assert!(matches!(current.node(embed), Err(DomError::StaleNode)));
    assert_eq!(
        current.node(copy).unwrap().attribute(&"type".into()),
        Some(&"image/svg+xml".into())
    );
    assert_eq!(
        captured.node(embed).unwrap().attribute(&"src".into()),
        Some(&"relative.svg".into())
    );
}

#[test]
fn fresh_native_track_retains_its_node_across_adoption_and_import_gets_a_new_node() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let destination = batch.create_document();
    let track = batch.create_element_ns(&source, Namespace::html(), "track");
    batch.set_attribute(&track, "kind", "CaPtIoNs");
    batch.set_attribute(
        &track,
        "label",
        DocumentString::from_utf16(vec![0x43, 0xd800]),
    );
    batch.set_attribute(&track, "srclang", "es");
    batch.set_attribute(&track, "id", "captions");
    batch.set_attribute_ns(
        &track,
        Namespace::from_uri("urn:other"),
        "p:kind",
        "chapters",
    );
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let destination = commit.document(&destination).unwrap();
    let track = commit.created(&track).unwrap().unwrap();
    let captured = dom.snapshot();
    assert_eq!(captured.node(track).unwrap().owner_document(), Some(source));
    assert!(!captured.node(track).unwrap().is_connected());
    let original = captured
        .node(track)
        .unwrap()
        .out_of_band_text_track()
        .unwrap();
    let admit = || Ok::<(), DomError>(());

    let mut batch = DomBatch::new();
    batch.adopt(track, destination);
    let copy = batch.import_node(&captured, track, destination, false);
    let copy = apply(&mut dom, batch).created(&copy).unwrap().unwrap();
    assert_ne!(copy, track);
    let current = dom.snapshot();
    for id in [track, copy] {
        let node = current.node(id).unwrap();
        assert_eq!(node.owner_document(), Some(destination));
        assert_eq!(node.attribute(&"kind".into()), Some(&"CaPtIoNs".into()));
        assert_eq!(
            node.attribute(&"label".into()),
            Some(&DocumentString::from_utf16(vec![0x43, 0xd800]))
        );
        let text = node.out_of_band_text_track().unwrap();
        assert_eq!(
            text.kind_with(admit).unwrap(),
            surgeist_html::attributes::HtmlTrackKind::Captions
        );
        assert_eq!(text.label_with(admit).unwrap(), &[0x43, 0xd800]);
        assert_eq!(text.language_with(admit).unwrap(), &[0x65, 0x73]);
        assert_eq!(
            text.identifier_with(admit).unwrap(),
            "captions".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            text.mode(),
            surgeist_html::elements::media::HtmlTextTrackMode::Disabled
        );
        assert_eq!(
            text.readiness(),
            surgeist_html::elements::media::HtmlTextTrackReadiness::NotLoaded
        );
        assert_eq!(text.cue_count(), 0);
        assert!(text.url().is_empty());
        assert_eq!(text.id() == original.id(), id == track);
    }
    assert_eq!(captured.node(track).unwrap().owner_document(), Some(source));
    let mut batch = DomBatch::new();
    batch.set_attribute(track, "kind", "unknown");
    batch.remove_attribute(track, "srclang");
    batch.destroy(copy);
    apply(&mut dom, batch);
    let next = dom.snapshot();
    let text = next.node(track).unwrap().out_of_band_text_track().unwrap();
    assert_eq!(
        text.kind_with(admit).unwrap(),
        surgeist_html::attributes::HtmlTrackKind::Metadata
    );
    assert!(text.language_with(admit).unwrap().is_empty());
    assert_eq!(text.id(), original.id());
    assert!(matches!(next.node(copy), Err(DomError::StaleNode)));
    assert_eq!(original.language_with(admit).unwrap(), &[0x65, 0x73]);
}

#[test]
fn changing_track_to_a_media_parent_refuses_the_entire_native_edit() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let track = batch.create_element_ns(&document, Namespace::html(), "track");
    let div = batch.create_element_ns(&document, Namespace::html(), "div");
    let audio = batch.create_element_ns(&document, Namespace::html(), "audio");
    batch.insert_before(&track, &div, None);
    let commit = apply(&mut dom, batch);
    let track = commit.created(&track).unwrap().unwrap();
    let audio = commit.created(&audio).unwrap().unwrap();
    let div = commit.created(&div).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(track, "label", "private prefix");
    batch.insert_before(track, audio, None);
    let cancelled = AtomicBool::new(false);
    assert_eq!(
        dom.apply(batch, &cancelled).unwrap_err(),
        DomError::TrackMembershipUnavailable
    );
    let current = dom.snapshot();
    assert_eq!(current.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let node = current.node(track).unwrap();
    assert_eq!(node.parent(), Some(div));
    let text = node.out_of_band_text_track().unwrap();
    assert!(
        text.label_with(|| Ok::<(), DomError>(()))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        text.kind_with(|| Ok::<(), DomError>(())).unwrap(),
        surgeist_html::attributes::HtmlTrackKind::Subtitles
    );
    assert_eq!(
        text.kind_with(|| Err(DomError::Cancelled)),
        Err(DomError::Cancelled)
    );
}

#[test]
fn fresh_native_audio_and_video_preserve_exact_namespaces() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let cases = [
        (HTML, "audio", "audio", Some(HtmlInitialMediaState::Audio)),
        (HTML, "p:video", "video", Some(HtmlInitialMediaState::Video)),
        (HTML, "Audio", "Audio", None),
        (SVG, "video", "video", None),
        ("urn:example", "audio", "audio", None),
    ];
    let tickets: Vec<_> = cases
        .iter()
        .map(|(namespace, qualified, _, _)| {
            batch.create_element_ns(&document, Namespace::from_uri(*namespace), *qualified)
        })
        .collect();
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let snapshot = dom.snapshot();
    for ((namespace, qualified, local, state), ticket) in cases.iter().zip(&tickets) {
        let id = commit.created(ticket).unwrap().unwrap();
        let node = snapshot.node(id).unwrap();
        let element = node.as_element().unwrap();
        assert_eq!(node.owner_document(), Some(document));
        assert!(node.parent().is_none());
        assert_eq!(node.tag_name(), Some(DocumentString::from(*qualified)));
        assert_eq!(
            element.name().namespace_uri(),
            Namespace::from_uri(*namespace).uri()
        );
        assert_eq!(element.name().local_name(), &DocumentString::from(*local));
        assert!(node.attributes().unwrap().is_empty());
        assert_eq!(element.initial_media_state(), *state);
        assert_eq!(
            element.media_automatic_track_selection_performed(),
            state.map(|_| false)
        );
        if let Some(state) = state {
            assert!(state.network_is_empty());
            assert!(!state.has_media_data());
            assert!(state.paused());
            assert!(state.can_autoplay());
            assert!(!state.delays_load_event());
            assert!(state.show_poster());
            assert_eq!(state.current_time(), 0.0);
            assert!(state.current_src().is_empty());
        }
    }
}

#[test]
fn unloaded_media_rejects_reached_resource_effects_atomically() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let audio = batch.create_element_ns(&document, Namespace::html(), "audio");
    let video = batch.create_element_ns(&document, Namespace::html(), "video");
    let source = batch.create_element_ns(&document, Namespace::html(), "source");
    batch.set_attribute(&audio, "poster", "plain audio content attribute");
    batch.set_attribute(&video, "poster", "");
    batch.set_attribute_ns(
        &video,
        Namespace::from_uri("urn:foreign"),
        "p:src",
        "unrelated",
    );
    let commit = apply(&mut dom, batch);
    let audio = commit.created(&audio).unwrap().unwrap();
    let video = commit.created(&video).unwrap().unwrap();
    let source = commit.created(&source).unwrap().unwrap();
    let mut unset = DomBatch::new();
    unset.remove_attribute(video, "poster");
    apply(&mut dom, unset);

    let before = dom.snapshot();
    let usage = dom.usage();
    let mut load = DomBatch::new();
    load.set_attribute(audio, "src", "");
    let mut poster = DomBatch::new();
    poster.set_attribute(video, "poster", "poster.png");
    let mut select = DomBatch::new();
    select.insert_before(source, video, None);
    let cancel = AtomicBool::new(false);
    for (batch, expected) in [
        (load, HtmlMediaResourceEffect::Load),
        (poster, HtmlMediaResourceEffect::DeterminePoster),
        (select, HtmlMediaResourceEffect::SelectResource),
    ] {
        assert_eq!(
            dom.apply(batch, &cancel).unwrap_err(),
            DomError::MediaResourceEffectUnavailable(expected)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
    assert!(dom.snapshot().node(source).unwrap().parent().is_none());
    assert_eq!(
        dom.snapshot()
            .node(video)
            .unwrap()
            .as_element()
            .unwrap()
            .initial_media_state(),
        Some(HtmlInitialMediaState::Video)
    );
    let mut clone = DomBatch::new();
    let cloned = clone.clone_node(video, false);
    let cloned = apply(&mut dom, clone).created(&cloned).unwrap().unwrap();
    assert_ne!(cloned, video);
    assert_eq!(
        dom.snapshot()
            .node(cloned)
            .unwrap()
            .as_element()
            .unwrap()
            .initial_media_state(),
        Some(HtmlInitialMediaState::Video)
    );
}

#[test]
fn media_fallback_is_ordinary_content_outside_the_flat_tree() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let cases = [(HTML, "audio"), (HTML, "video"), (SVG, "video")].map(|(namespace, name)| {
        let host = batch.create_element_ns(&document, Namespace::from_uri(namespace), name);
        let fallback = batch.create_element_ns(&document, Namespace::html(), "span");
        let text = batch.create_text(&document, "fallback");
        batch.insert_before(&host, &root, None);
        batch.insert_before(&fallback, &host, None);
        batch.insert_before(&text, &fallback, None);
        (host, fallback, text, namespace == HTML)
    });
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let flat = snapshot
        .flat_tree(document, DomSlotQueryLimits::new(16, 16).unwrap(), &cancel)
        .unwrap();
    for (host, fallback, text, media) in cases {
        let host = commit.created(&host).unwrap().unwrap();
        let fallback = commit.created(&fallback).unwrap().unwrap();
        let text = commit.created(&text).unwrap().unwrap();
        assert_eq!(snapshot.node(fallback).unwrap().parent(), Some(host));
        assert_eq!(snapshot.node(text).unwrap().parent(), Some(fallback));
        assert_eq!(snapshot.node(host).unwrap().shadow_root(), None);
        assert_eq!(flat.relation(host).unwrap(), DomFlatRelation::Parent(root));
        assert_eq!(
            flat.relation(fallback).unwrap(),
            if media {
                DomFlatRelation::Excluded
            } else {
                DomFlatRelation::Parent(host)
            }
        );
        assert_eq!(
            flat.relation(text).unwrap(),
            if media {
                DomFlatRelation::Excluded
            } else {
                DomFlatRelation::Parent(fallback)
            }
        );
    }
}

#[test]
fn removing_media_from_a_document_requires_its_stable_state_steps() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let container = batch.create_element_ns(&document, Namespace::html(), "div");
    let audio = batch.create_element_ns(&document, Namespace::html(), "audio");
    let video = batch.create_element_ns(&document, Namespace::html(), "video");
    let fragment = batch.create_fragment(&document);
    let detached = batch.create_element_ns(&document, Namespace::html(), "audio");
    let shadow_host = batch.create_element_ns(&document, Namespace::html(), "section");
    let shadow = batch.attach_shadow(
        &shadow_host,
        DomShadowRootInit::new(HtmlShadowRootMode::Open),
    );
    let hosted = batch.create_element_ns(&document, Namespace::html(), "video");
    batch.insert_before(&container, &document, None);
    batch.insert_before(&audio, &container, None);
    batch.insert_before(&video, &container, None);
    batch.insert_before(&detached, &fragment, None);
    batch.insert_before(&shadow_host, &container, None);
    batch.insert_before(&hosted, &shadow, None);
    let commit = apply(&mut dom, batch);
    let container = commit.created(&container).unwrap().unwrap();
    let audio = commit.created(&audio).unwrap().unwrap();
    let video = commit.created(&video).unwrap().unwrap();
    let detached = commit.created(&detached).unwrap().unwrap();
    let shadow_host = commit.created(&shadow_host).unwrap().unwrap();
    let hosted = commit.created(&hosted).unwrap().unwrap();
    let cancel = AtomicBool::new(false);

    let mut remove_detached = DomBatch::new();
    remove_detached.detach(detached);
    remove_detached.detach(shadow_host);
    apply(&mut dom, remove_detached);
    assert!(dom.snapshot().node(detached).unwrap().parent().is_none());
    assert!(dom.snapshot().node(shadow_host).unwrap().parent().is_none());
    assert!(dom.snapshot().node(hosted).unwrap().parent().is_some());

    let before = dom.snapshot().stamp();
    let usage = dom.usage();
    for node in [audio, container] {
        let mut batch = DomBatch::new();
        batch.detach(node);
        assert_eq!(
            dom.apply(batch, &cancel).unwrap_err(),
            DomError::MediaRemovalStableStateUnavailable
        );
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(dom.usage(), usage);
    }
    let limits = DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2);
    let result = match dom.begin_source_edit(DomSourceEdit::detach(video), limits, &cancel) {
        Ok(mut lease) => lease.advance(&cancel).map(drop),
        Err(error) => Err(error),
    };
    assert_eq!(result, Err(DomError::MediaRemovalStableStateUnavailable));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().node(audio).unwrap().parent(),
        Some(container)
    );
    assert_eq!(
        dom.snapshot().node(video).unwrap().parent(),
        Some(container)
    );
}

#[test]
fn source_media_creation_retains_its_exact_native_result() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = apply(&mut dom, batch).document(&document).unwrap();
    let cancel = AtomicBool::new(false);
    let limits = DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2);
    for (name, expected) in [
        ("audio", HtmlInitialMediaState::Audio),
        ("p:video", HtmlInitialMediaState::Video),
    ] {
        let edit = DomSourceEdit::create_element_ns(document.node(), Namespace::html(), name);
        let mut lease = dom.begin_source_edit(edit, limits, &cancel).unwrap();
        let DomSourceProgress::Complete(result) = lease.advance(&cancel).unwrap() else {
            panic!("fresh unloaded media has no resource handoff");
        };
        let node = result.published_node().unwrap();
        drop(lease);
        assert_eq!(
            dom.snapshot()
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .initial_media_state(),
            Some(expected)
        );
        let before = dom.snapshot().stamp();
        let usage = dom.usage();
        let edit = DomSourceEdit::set_native_attribute(node, "src", "");
        let refusal = match dom.begin_source_edit(edit, limits, &cancel) {
            Ok(mut lease) => lease.advance(&cancel).unwrap_err(),
            Err(error) => error,
        };
        assert_eq!(
            refusal,
            DomError::MediaResourceEffectUnavailable(HtmlMediaResourceEffect::Load)
        );
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(dom.usage(), usage);
    }
    let before = dom.snapshot();
    let usage = dom.usage();
    let edit = DomSourceEdit::create_element_ns(document.node(), Namespace::html(), "audio");
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        dom.begin_source_edit(edit, limits, &cancelled),
        Err(DomError::Cancelled)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
}

#[test]
fn unmet_builtin_profiles_reject_the_whole_batch_with_the_actual_requirement() {
    let mut dom = dom();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    batch.create_element(&document, "not-published");
    batch.create_element_ns(&document, Namespace::html(), "object");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ObjectTaskSourceUnavailable
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage().managed_nodes(), 0);
}

#[test]
fn nonce_content_and_idl_slot_change_independently_with_precise_change_domains() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    batch.set_attribute(&element, "nonce", "content");
    let commit = apply(&mut dom, batch);
    let element = commit.created(&element).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "slot");
    let changed = apply(&mut dom, batch);
    assert_eq!(
        changed.changes().changes(),
        &[DomChange::ElementState(element)]
    );
    let capture = dom.snapshot();
    let current = capture.node(element).unwrap().as_element().unwrap();
    assert_eq!(current.nonce(), Some(&"slot".into()));
    assert_eq!(
        capture.node(element).unwrap().attribute(&"nonce".into()),
        Some(&"content".into())
    );
    assert_eq!(
        old.node(element).unwrap().as_element().unwrap().nonce(),
        Some(&"content".into())
    );
    let stamp = capture.stamp();
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "slot");
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().stamp(), stamp);
    // Setting the same content still executes nonce change steps, resetting a
    // diverged slot even though the attribute's final value does not change.
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "nonce", "content");
    let changed = apply(&mut dom, batch);
    assert_eq!(
        changed.changes().changes(),
        &[DomChange::ElementState(element)]
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "Nonce", "case");
    batch.set_attribute_ns(
        element,
        Namespace::from_uri("urn:x"),
        "p:nonce",
        "namespace",
    );
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        Some(&"content".into())
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute(element, "nonce");
    let changed = apply(&mut dom, batch);
    assert!(
        changed
            .changes()
            .changes()
            .contains(&DomChange::Attributes(element))
    );
    assert!(
        changed
            .changes()
            .changes()
            .contains(&DomChange::ElementState(element))
    );
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        Some(&"".into())
    );
}

#[test]
fn generic_nonce_content_is_ordinary_and_has_no_cryptographic_slot() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element(&document, "input");
    batch.set_attribute(&element, "nonce", "ordinary");
    let commit = apply(&mut dom, batch);
    let element = commit.created(&element).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        None
    );
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "unsupported");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NonceInapplicable
    );
}

#[test]
fn captured_import_copies_nonce_after_attributes_and_native_insertion_keeps_content() {
    let mut source = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let child = batch.create_element_ns(&document, Namespace::from_uri(SVG), "g");
    batch.set_attribute(&child, "nonce", "public");
    batch.set_nonce(&child, "private");
    batch.insert_before(&child, &parent, None);
    batch.insert_before(&parent, &document, None);
    let commit = apply(&mut source, batch);
    let parent = commit.created(&parent).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let captured = source.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(parent);
    batch.destroy(parent);
    apply(&mut source, batch);
    drop(source);
    let mut target = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let imported = batch.import_node(&captured, parent, &document, true);
    batch.insert_before(&imported, &document, None);
    let commit = apply(&mut target, batch);
    let document = commit.document(&document).unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    let capture = target.snapshot();
    let copied_child = capture.node(imported).unwrap().children()[0];
    assert_ne!(copied_child, child);
    let element = capture.node(copied_child).unwrap().as_element().unwrap();
    assert_eq!(element.nonce(), Some(&"private".into()));
    assert_eq!(
        capture
            .node(copied_child)
            .unwrap()
            .attribute(&"nonce".into()),
        Some(&"public".into())
    );
    assert_eq!(
        capture.node(copied_child).unwrap().owner_document(),
        Some(document)
    );
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    batch.adopt(imported, &other);
    batch.insert_before(imported, &other, None);
    let commit = apply(&mut target, batch);
    let capture = target.snapshot();
    assert_eq!(
        capture.node(copied_child).unwrap().owner_document(),
        Some(commit.document(&other).unwrap())
    );
    assert_eq!(
        capture
            .node(copied_child)
            .unwrap()
            .as_element()
            .unwrap()
            .nonce(),
        Some(&"private".into())
    );
}

#[test]
fn exact_nonce_string_capacity_counts_both_content_and_slot_and_rolls_back_late_failure() {
    // 31 document + 28 HTML URI + 3 div + 5 nonce + 2 content + 2 slot = 71.
    for capacity in [70, 71] {
        let mut dom = Dom::new(
            DomLimits::new(
                3,
                capacity,
                16,
                0,
                0,
                0,
                0,
                surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let element = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
        batch.set_attribute(&element, "nonce", "xy");
        let result = dom.apply(batch, &AtomicBool::new(false));
        if capacity == 70 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 70
                }
            );
            assert_eq!(dom.usage().managed_nodes(), 0);
        } else {
            let commit = result.unwrap();
            let document = commit.document(&document).unwrap();
            let element = commit.created(&element).unwrap().unwrap();
            assert_eq!(dom.usage().string_code_units(), 71);
            let before = dom.snapshot();
            let mut batch = DomBatch::new();
            batch.set_nonce(element, "z");
            batch.insert_before(document, element, None);
            assert_eq!(
                dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                DomError::HierarchyRequest
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage().string_code_units(), 71);
            assert_eq!(
                dom.snapshot()
                    .node(element)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .nonce(),
                Some(&"xy".into())
            );
            let mut batch = DomBatch::new();
            batch.set_nonce(element, "long");
            assert_eq!(
                dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 71
                }
            );
        }
    }
}

#[test]
fn replacing_nonce_slot_can_make_room_for_the_content_attribute() {
    // The document and element use 62 units. With an empty content attribute
    // and a private 16-unit slot, the replacement case starts at the limit.
    // Both edits end at 62 + 5 (name) + 7 (content) + 7 (slot).
    let cancel = AtomicBool::new(false);
    for existing in [false, true] {
        let mut dom = Dom::new(
            DomLimits::new(
                3,
                83,
                16,
                0,
                0,
                0,
                0,
                surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut initial = DomBatch::new();
        let document = initial.create_document();
        let element = initial.create_element_ns(&document, Namespace::from_uri(HTML), "div");
        if existing {
            initial.set_attribute(&element, "nonce", "");
        }
        initial.set_nonce(&element, "0123456789abcdef");
        let element = dom
            .apply(initial, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap();
        assert_eq!(
            dom.usage().string_code_units(),
            if existing { 83 } else { 78 }
        );

        let before = dom.snapshot();
        let old = before.node(element).unwrap().attributes().unwrap().item(0);
        let mut edit = DomBatch::new();
        edit.set_attribute(element, "nonce", "1234567");
        dom.apply(edit, &cancel).unwrap();
        assert_eq!(dom.usage().string_code_units(), 81);
        let after = dom.snapshot();
        let node = after.node(element).unwrap();
        let attr = node.attributes().unwrap().item(0).unwrap();
        assert_eq!(attr.value(), &"1234567".into());
        assert_eq!(node.as_element().unwrap().nonce(), Some(attr.value()));
        if let Some(old) = old {
            assert_eq!(attr.id(), old.id());
            assert_eq!(old.value(), &"".into());
        }
    }
}

#[test]
fn common_state_survives_character_data_replacement_fragment_moves_and_child_replacement() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let old_child = batch.create_element_ns(&document, Namespace::from_uri(MATH), "input");
    let new_child = batch.create_element_ns(&document, Namespace::from_uri(HTML), "span");
    let text = batch.create_text(&document, "old");
    let fragment = batch.create_fragment(&document);
    let nonce = DocumentString::from_utf16(vec![0xd800, 0x41, 0xdc00]);
    batch.set_attribute(&new_child, "nonce", nonce.clone());
    batch.set_attribute(&new_child, "popover", "auto");
    batch.set_attribute(&new_child, "autofocus", "");
    batch.set_attribute(&new_child, "is", "x-later");
    batch.insert_before(&text, &new_child, None);
    batch.insert_before(&old_child, &root, None);
    batch.insert_before(&new_child, &fragment, None);
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    let root = commit.created(&root).unwrap().unwrap();
    let old_child = commit.created(&old_child).unwrap().unwrap();
    let new_child = commit.created(&new_child).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_data(text, 0, 3, "new");
    batch.replace_child(fragment, root, old_child);
    batch.remove_attribute(new_child, "popover");
    apply(&mut dom, batch);
    let capture = dom.snapshot();
    assert_eq!(capture.node(root).unwrap().children(), &[new_child]);
    assert!(capture.node(fragment).unwrap().children().is_empty());
    assert_eq!(capture.node(old_child).unwrap().parent(), None);
    let element = capture.node(new_child).unwrap().as_element().unwrap();
    assert_eq!(element.nonce(), Some(&nonce));
    assert_eq!(
        capture.node(new_child).unwrap().attribute(&"nonce".into()),
        Some(&nonce)
    );
    assert_eq!(element.saved_is(), None);
    assert_eq!(
        element.custom_element_state(),
        HtmlCustomElementState::Uncustomized
    );
    assert!(element.common_state().html().unwrap().popover_is_hidden());
    assert_eq!(old.node(root).unwrap().children(), &[old_child]);
    assert_eq!(old.node(new_child).unwrap().parent(), Some(fragment));
}

#[test]
fn state_only_edits_have_dom_history_while_equal_host_facts_keep_their_revision() {
    use surgeist_dom::capture::{DomHostDomains, DomHostObservations, DomHostPublisher, Observed};
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let mut host = DomHostPublisher::new(DomHostDomains::Focus);
    let view = before.document(document).unwrap();
    let first = host
        .publish(
            view,
            view.stamp(),
            DomHostObservations::Focus(Observed::Supported(None)),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_nonce(element, "state");
    let commit = apply(&mut dom, batch);
    assert_eq!(
        dom.changes_since(before.stamp()).unwrap().changes(),
        &[DomChange::ElementState(element)]
    );
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::ElementState(element)]
    );
    let capture = dom.snapshot();
    let view = capture.document(document).unwrap();
    let next = host
        .publish(
            view,
            view.stamp(),
            DomHostObservations::Focus(Observed::Supported(None)),
        )
        .unwrap();
    assert_eq!(
        first.capture().stamp().host_revision(),
        next.capture().stamp().host_revision()
    );
    assert_ne!(
        first.capture().stamp().document(),
        next.capture().stamp().document()
    );
}

#[test]
fn media_stable_state_admits_before_publication_and_uses_current_ordinary_membership() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let owner = dom.bind_media_stable_state(1, &cancel).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let second_document = batch.create_document();
    let reinsert_parent = batch.create_element_ns(&second_document, Namespace::html(), "div");
    batch.insert_before(&reinsert_parent, &second_document, None);
    let container = batch.create_element_ns(&document, Namespace::html(), "div");
    let audio = batch.create_element_ns(&document, Namespace::html(), "audio");
    let video = batch.create_element_ns(&document, Namespace::html(), "video");
    batch.insert_before(&container, &document, None);
    batch.insert_before(&audio, &container, None);
    batch.insert_before(&video, &container, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let second_document = commit.document(&second_document).unwrap();
    let reinsert_parent = commit.created(&reinsert_parent).unwrap().unwrap();
    let container = commit.created(&container).unwrap().unwrap();
    let audio = commit.created(&audio).unwrap().unwrap();
    let video = commit.created(&video).unwrap().unwrap();
    let original = dom.snapshot();

    let mut batch = DomBatch::new();
    batch.detach(container);
    assert!(matches!(
        dom.apply(batch, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::MediaRemovalContinuations,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), original.stamp());
    assert_eq!(owner.run_native(&mut dom, &cancel).unwrap(), None);

    let limits = DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2);
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::detach(audio), limits, &cancel)
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(media_autoplay(&dom, audio), Some(true));
    assert_eq!(
        owner.run_native(&mut dom, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    );
    let mut batch = DomBatch::new();
    batch.insert_before(audio, reinsert_parent, None);
    apply(&mut dom, batch);
    let completed = owner.run_native(&mut dom, &cancel).unwrap().unwrap();
    assert_eq!(completed.target(), audio);
    assert_eq!(completed.former_document(), document);
    assert_eq!(media_autoplay(&dom, audio), Some(true));

    let mut batch = DomBatch::new();
    batch.insert_before(audio, reinsert_parent, None);
    let before_same_position = dom.snapshot().stamp();
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().stamp(), before_same_position);
    let reinsertion = owner.run_native(&mut dom, &cancel).unwrap().unwrap();
    assert_ne!(reinsertion, completed);
    assert_eq!(reinsertion.target(), audio);
    assert_eq!(reinsertion.former_document(), second_document);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::move_before(audio, reinsert_parent, None),
            limits,
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert_eq!(owner.run_native(&mut dom, &cancel).unwrap(), None);

    let mut batch = DomBatch::new();
    batch.detach(container);
    apply(&mut dom, batch);
    assert_eq!(media_autoplay(&dom, video), Some(true));
    let completed = owner.run_native(&mut dom, &cancel).unwrap().unwrap();
    assert_eq!(completed.target(), video);
    assert_eq!(completed.former_document(), document);
    assert_eq!(media_autoplay(&dom, video), Some(false));
    assert_eq!(
        dom.snapshot().node(video).unwrap().parent(),
        Some(container)
    );
    assert_eq!(owner.run_native(&mut dom, &cancel).unwrap(), None);
}

#[test]
fn media_stable_state_retires_the_original_generation_and_skips_other_trees() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let owner = dom.bind_media_stable_state(4, &cancel).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let fragment = batch.create_fragment(&document);
    let retired = batch.create_element_ns(&document, Namespace::html(), "audio");
    let light = batch.create_element_ns(&document, Namespace::html(), "audio");
    let hosted = batch.create_element_ns(&document, Namespace::html(), "video");
    let detached = batch.create_element_ns(&document, Namespace::html(), "audio");
    batch.insert_before(&host, &document, None);
    batch.insert_before(&retired, &host, None);
    batch.insert_before(&light, &host, None);
    batch.insert_before(&hosted, &shadow, None);
    batch.insert_before(&detached, &fragment, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let retired = commit.created(&retired).unwrap().unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let light = commit.created(&light).unwrap().unwrap();
    let hosted = commit.created(&hosted).unwrap().unwrap();
    let detached = commit.created(&detached).unwrap().unwrap();

    let mut batch = DomBatch::new();
    batch.detach(retired);
    batch.destroy(retired);
    batch.detach(detached);
    batch.detach(host);
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    let replacement = batch.create_element_ns(document.node(), Namespace::html(), "video");
    let replacement = apply(&mut dom, batch)
        .created(&replacement)
        .unwrap()
        .unwrap();
    assert_eq!(
        dom.snapshot().node(retired).unwrap_err(),
        DomError::StaleNode
    );
    assert_ne!(retired, replacement);
    let retired_step = owner.run_native(&mut dom, &cancel).unwrap().unwrap();
    let descendant_step = owner.run_native(&mut dom, &cancel).unwrap().unwrap();
    assert_eq!(retired_step.target(), retired);
    assert_eq!(descendant_step.target(), light);
    assert_eq!(owner.run_native(&mut dom, &cancel).unwrap(), None);
    assert_eq!(media_autoplay(&dom, light), Some(false));
    for node in [hosted, detached, replacement] {
        assert_eq!(media_autoplay(&dom, node), Some(true));
    }
}

fn media_autoplay(dom: &Dom, node: surgeist_dom::identity::NodeId) -> Option<bool> {
    let snapshot = dom.snapshot();
    let surgeist_dom::tree::DomNodeKind::Element(element) = snapshot.node(node).unwrap().kind()
    else {
        panic!("the test created an exact media element");
    };
    element.media_can_autoplay()
}

#[test]
fn media_removal_admission_keeps_first_refusal_before_later_resource_work() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let audio = batch.create_element_ns(&document, Namespace::html(), "audio");
    batch.insert_before(&audio, &document, None);
    let audio = apply(&mut dom, batch).created(&audio).unwrap().unwrap();
    let before = dom.snapshot();

    let conflicting_edit = || {
        let mut batch = DomBatch::new();
        batch.detach(audio);
        batch.set_attribute(audio, "src", "");
        batch
    };
    assert_eq!(
        dom.apply(conflicting_edit(), &cancel).unwrap_err(),
        DomError::MediaRemovalStableStateUnavailable,
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());

    let stable_state = dom.bind_media_stable_state(0, &cancel).unwrap();
    assert_eq!(
        dom.apply(conflicting_edit(), &cancel).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::MediaRemovalContinuations,
            limit: 0,
        },
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(stable_state.run_native(&mut dom, &cancel).unwrap(), None);
}

#[test]
fn raised_node_limits_cover_the_live_checkpoint_path() {
    let with_nodes = |nodes| {
        DomLimits::new(
            nodes,
            16384,
            256,
            0,
            0,
            16,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap()
    };
    let mut dom = Dom::new(with_nodes(4)).unwrap();
    let cancel = AtomicBool::new(false);
    let owner = dom.bind_media_stable_state(1, &cancel).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let audio = batch.create_element_ns(&document, Namespace::html(), "audio");
    batch.insert_before(&audio, &document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let audio = commit.created(&audio).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.detach(audio);
    apply(&mut dom, batch);

    dom.set_limits(with_nodes(32)).unwrap();
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(document.node(), Namespace::html(), "div");
    let mut parent = root;
    for _ in 0..10 {
        let child = batch.create_element_ns(document.node(), Namespace::html(), "div");
        batch.insert_before(&child, &parent, None);
        parent = child;
    }
    batch.insert_before(audio, &parent, None);
    apply(&mut dom, batch);
    assert_eq!(
        owner
            .run_native(&mut dom, &cancel)
            .unwrap()
            .unwrap()
            .target(),
        audio
    );
    assert_eq!(media_autoplay(&dom, audio), Some(false));
}
