//! Literal oracles from HTML877d's normative definitions/interface dispatch and
//! common initial slots. DOM validity, live identity and family hooks are later.
use std::convert::Infallible;
use surgeist_html::bounded::{
    HtmlElementNameRead as Read, element_classification_with, is_nonce_element,
    native_element_initialization_with,
};
use surgeist_html::{
    DocumentString, HtmlCommonScope, HtmlCommonState, HtmlElementClass as R,
    HtmlNativeFormAssociationKind as F, HtmlNativeFreshNonce, HtmlNativeInitialization,
    KnownHtmlElement as K, Namespace, QualifiedName,
};

fn initialize(name: &QualifiedName) -> HtmlNativeInitialization {
    native_element_initialization_with(name, |_| Ok::<_, Infallible>(())).unwrap()
}

fn html(local: &str) -> HtmlNativeInitialization {
    initialize(&QualifiedName::new(Namespace::html(), None, local))
}

fn html_class(local: &str) -> R {
    classify(&QualifiedName::new(Namespace::html(), None, local))
}

fn classify(name: &QualifiedName) -> R {
    element_classification_with(name, |_| Ok::<_, Infallible>(())).unwrap()
}

// Each of these normative definitions uses HTMLElement, plus div/span's actual
// reflected-only interfaces. This literal set is independent of production.
const COMMON: &str = "article section nav aside hgroup header footer address
dt dd figure figcaption main search em strong small s cite dfn abbr ruby rt rp
code var samp kbd sub sup i b u mark bdi bdo wbr summary noscript div span";

// These interfaces add reflection, child-text operations or current-tree
// queries, rather than another mutable native initialization slot. Heading and
// modification interfaces are shared definitions; body event execution remains
// a separately admitted capability. CommonHtml describes fulfilled native
// initialization, not identical WebIDL interfaces.
const REFLECTED_QUERY_COMMON: &str = "html head title body p hr pre blockquote
ol ul menu li dl q data time br ins del map datalist label legend progress meter
table caption colgroup col tbody thead tfoot tr td th h1 h2 h3 h4 h5 h6";

#[test]
fn reflected_child_text_and_tree_query_elements_need_only_common_initialization() {
    assert_eq!(REFLECTED_QUERY_COMMON.split_whitespace().count(), 41);
    for local in REFLECTED_QUERY_COMMON.split_whitespace() {
        let initial = html(local);
        assert_eq!(initial.requirement(), R::CommonHtml, "{local}");
        assert_common(initial, true, true);
    }
}

#[test]
fn all_113_normative_modern_names_preserve_their_exact_requirement() {
    // Complete concrete names from normative DOM-interface definition blocks,
    // not the nonnormative index or the classifier's expected output.
    let modern = "a abbr address area article aside audio b base bdi bdo blockquote body br
button canvas caption cite code col colgroup data datalist dd del details dfn
dialog div dl dt em embed fieldset figcaption figure footer form h1 h2 h3 h4
h5 h6 head header hgroup hr html i iframe img input ins kbd label legend li
link main map mark menu meta meter nav noscript object ol optgroup option
output p picture pre progress q rp rt ruby s samp script search section select
selectedcontent slot small source span strong style sub summary sup table
tbody td template textarea tfoot th thead time title tr track u ul var video wbr";
    assert_eq!(modern.split_whitespace().count(), 113);
    assert_eq!(K::ALL.len(), 113);
    assert_eq!(COMMON.split_whitespace().count(), 41);
    assert_eq!(REFLECTED_QUERY_COMMON.split_whitespace().count(), 41);
    for local in modern.split_whitespace() {
        let requirement = html_class(local);
        let form = match local {
            "img" => Some(F::Image),
            "button" => Some(F::Button),
            "fieldset" => Some(F::Fieldset),
            "object" => Some(F::Object),
            "textarea" => Some(F::Textarea),
            "output" => Some(F::Output),
            "input" => Some(F::Input),
            "select" => Some(F::Select),
            _ => None,
        };
        if let Some(form) = form {
            assert_eq!(requirement, R::FormAssociated(form), "{local}");
        } else if COMMON
            .split_whitespace()
            .chain(REFLECTED_QUERY_COMMON.split_whitespace())
            .any(|expected| expected == local)
        {
            assert_eq!(requirement, R::CommonHtml, "{local}");
        } else {
            let R::KnownHtml(element) = requirement else {
                panic!("{local}: built-in must retain an unmet identity, got {requirement:?}")
            };
            assert_eq!(element.local_name(), local);
        }
    }
}

#[test]
fn obsolete_dispatch_keeps_26_common_cases_and_two_stateful_requirements() {
    let common = "applet bgsound blink isindex keygen multicol nextid spacer
acronym basefont big center nobr noembed noframes plaintext rb rtc strike tt
listing xmp dir font param frameset";
    assert_eq!(common.split_whitespace().count(), 26);
    for local in common.split_whitespace() {
        assert_eq!(html_class(local), R::CommonHtml, "{local}");
    }
    assert_eq!(html_class("marquee"), R::HtmlMarquee);
    assert_eq!(html_class("frame"), R::HtmlFrame);
}

#[test]
fn known_specials_never_fall_through_as_common() {
    for (local, element) in [
        ("a", K::A),
        ("area", K::Area),
        ("audio", K::Audio),
        ("base", K::Base),
        ("canvas", K::Canvas),
        ("details", K::Details),
        ("dialog", K::Dialog),
        ("embed", K::Embed),
        ("form", K::Form),
        ("iframe", K::Iframe),
        ("link", K::Link),
        ("meta", K::Meta),
        ("optgroup", K::Optgroup),
        ("option", K::Option),
        ("picture", K::Picture),
        ("script", K::Script),
        ("selectedcontent", K::Selectedcontent),
        ("slot", K::Slot),
        ("source", K::Source),
        ("style", K::Style),
        ("template", K::Template),
        ("track", K::Track),
        ("video", K::Video),
    ] {
        assert_eq!(html_class(local), R::KnownHtml(element));
    }
}

#[test]
fn exact_unknown_custom_and_uppercase_names_use_only_the_exhausted_fallback() {
    for local in [
        "Input",
        "SCRIPT",
        "DIV",
        "image",
        "unknown",
        "x-widget",
        "X-widget",
        "annotation-xml",
    ] {
        assert_eq!(html_class(local), R::CommonHtml, "{local}");
    }
    let prefixed = QualifiedName::new(Namespace::html(), Some("p".into()), "input");
    assert_eq!(classify(&prefixed), R::FormAssociated(F::Input));
}

#[test]
fn real_namespace_identity_separates_svg_script_mathml_and_generic_names() {
    for (namespace, local, expected) in [
        (Namespace::svg(), "script", R::SvgScript),
        (Namespace::svg(), "Script", R::CommonSvg),
        (Namespace::svg(), "input", R::CommonSvg),
        (Namespace::mathml(), "script", R::CommonMathml),
        (Namespace::mathml(), "input", R::CommonMathml),
        (Namespace::mathml(), "template", R::CommonMathml),
        (Namespace::mathml(), "annotation-xml", R::CommonMathml),
        (Namespace::none(), "input", R::GenericElement),
        (
            Namespace::from_uri("urn:example"),
            "template",
            R::GenericElement,
        ),
        (
            Namespace::from_uri("http://www.w3.org/1999/XHTML"),
            "input",
            R::GenericElement,
        ),
    ] {
        assert_eq!(
            classify(&QualifiedName::new(namespace, None, local)),
            expected
        );
    }
}

fn assert_common(initial: HtmlNativeInitialization, html: bool, nonce: bool) {
    let common = initial.common();
    assert!(common.element().reaction_queue_is_empty());
    assert!(!common.element().click_in_progress());
    assert_eq!(
        initial.nonce(),
        nonce.then_some(HtmlNativeFreshNonce::Empty)
    );
    assert_eq!(common.html().is_some(), html);
    if let Some(state) = common.html() {
        assert!(!state.has_previously_focused_element());
        assert!(!state.has_attached_internals());
        assert!(state.popover_is_hidden());
        assert!(!state.has_popover_trigger());
        assert!(!state.popover_is_hiding());
        assert!(!state.has_popover_toggle_task_tracker());
        assert!(!state.has_popover_close_watcher());
        assert!(!state.has_popover_opened_in_mode());
    }
}

#[test]
fn actual_fresh_common_tuple_covers_every_requirement_and_exact_scope() {
    for (namespace, local, html, nonce) in [
        (Namespace::none(), "input", false, false),
        (Namespace::from_uri("urn:example"), "script", false, false),
        (Namespace::html(), "article", true, true),
        (Namespace::html(), "x-widget", true, true),
        (Namespace::html(), "input", true, true),
        (Namespace::html(), "script", true, true),
        (Namespace::html(), "marquee", true, true),
        (Namespace::html(), "frame", true, true),
        (Namespace::svg(), "script", false, true),
        (Namespace::svg(), "input", false, true),
        (Namespace::mathml(), "template", false, true),
    ] {
        let name = QualifiedName::new(namespace, None, local);
        let initial = initialize(&name);
        assert_common(initial, html, nonce);
        assert_eq!(initial.nonce().is_some(), is_nonce_element(&name));
    }
}

#[test]
fn foreign_and_mathml_branches_never_read_local_names() {
    for (namespace, expected) in [
        (Namespace::none(), R::GenericElement),
        (Namespace::from_uri("urn:example"), R::GenericElement),
        (Namespace::mathml(), R::CommonMathml),
    ] {
        let name = QualifiedName::new(namespace, None, DocumentString::from_utf16([0xd800]));
        let initial = native_element_initialization_with(&name, |read| {
            assert!(matches!(read, Read::NamespaceHeader | Read::NamespacePair));
            Ok::<_, Infallible>(())
        })
        .unwrap();
        assert_eq!(initial.requirement(), expected);
    }
}

#[test]
fn every_admission_refusal_preserves_first_failure_and_original_operands() {
    for (namespace, local) in [
        (Namespace::none(), "input"),
        (Namespace::html(), "article"),
        (Namespace::html(), "input"),
        (Namespace::html(), "frame"),
        (Namespace::html(), "marquee"),
        (Namespace::html(), "X-widget"),
        (Namespace::svg(), "script"),
        (Namespace::mathml(), "input"),
    ] {
        let name = QualifiedName::new(namespace, Some(DocumentString::from_utf16([0xd800])), local);
        let before = name.clone();
        let mut reads = Vec::new();
        let expected = native_element_initialization_with(&name, |read| {
            reads.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap();
        assert_eq!(expected, initialize(&name));
        for (cut, &denied) in reads.iter().enumerate() {
            let mut observed = 0;
            let result = native_element_initialization_with(&name, |read| {
                assert!(observed <= cut, "read after first refusal");
                let position = observed;
                observed += 1;
                if position == cut {
                    Err((position, read))
                } else {
                    Ok(())
                }
            });
            assert_eq!(result, Err((cut, denied)));
            assert_eq!(observed, cut + 1);
            assert_eq!(name, before);
        }
    }
}

#[test]
fn arbitrary_long_and_isolated_units_are_preserved_with_bounded_inspection() {
    for local in [
        DocumentString::default(),
        DocumentString::from_utf16([0xd800]),
        DocumentString::from_utf16(vec![65; 100_000]),
    ] {
        let name = QualifiedName::new(Namespace::html(), None, local);
        let before = name.clone();
        let mut local_pairs = 0;
        let initial = native_element_initialization_with(&name, |read| {
            if read == Read::LocalNamePair {
                local_pairs += 1;
            }
            Ok::<_, Infallible>(())
        })
        .unwrap();
        assert_eq!(initial.requirement(), R::CommonHtml);
        if name.local_name().len() != 1 {
            assert_eq!(local_pairs, 0);
        }
        assert_eq!(name, before);
    }
}

#[test]
fn classification_separates_common_scope_and_nonce_applicability() {
    for (namespace, local, scope, nonce) in [
        (Namespace::none(), "div", HtmlCommonScope::Element, false),
        (
            Namespace::from_uri("urn:other"),
            "script",
            HtmlCommonScope::Element,
            false,
        ),
        (Namespace::html(), "x-example", HtmlCommonScope::Html, true),
        (Namespace::html(), "input", HtmlCommonScope::Html, true),
        (Namespace::html(), "marquee", HtmlCommonScope::Html, true),
        (Namespace::html(), "frame", HtmlCommonScope::Html, true),
        (Namespace::svg(), "script", HtmlCommonScope::Element, true),
        (Namespace::svg(), "div", HtmlCommonScope::Element, true),
        (Namespace::mathml(), "div", HtmlCommonScope::Element, true),
        (
            Namespace::from_uri("http://www.w3.org/1999/XHTML"),
            "div",
            HtmlCommonScope::Element,
            false,
        ),
    ] {
        let class = classify(&QualifiedName::new(namespace, None, local));
        assert_eq!(class.common_scope(), scope, "{local}");
        assert_eq!(class.has_nonce(), nonce, "{local}");
    }
}

#[test]
fn initial_common_values_preserve_element_and_html_applicability() {
    for scope in [HtmlCommonScope::Element, HtmlCommonScope::Html] {
        let current = HtmlCommonState::<Infallible>::new(scope);
        assert_eq!(current.scope(), scope);
        assert!(current.element().reaction_queue_is_empty());
        assert!(!current.element().click_in_progress());
        assert_eq!(current.html().is_some(), scope == HtmlCommonScope::Html);
        if let Some(html) = current.html() {
            assert_eq!(html.previously_focused_element(), None);
            assert!(!html.has_previously_focused_element());
            assert!(!html.has_attached_internals());
            assert!(html.popover_is_hidden());
            assert!(!html.has_popover_trigger());
            assert!(!html.popover_is_hiding());
            assert!(!html.has_popover_toggle_task_tracker());
            assert!(!html.has_popover_close_watcher());
            assert!(!html.has_popover_opened_in_mode());
        }
    }
}

#[test]
fn remembering_and_clearing_focus_retains_the_actual_nullable_reference() {
    // HTML877d #previously-focused-element: initially null, subsequently the
    // actual remembered element. References here are value-owner inputs only.
    let mut current = HtmlCommonState::<u32>::new(HtmlCommonScope::Html);
    let html = current.html_mut().unwrap();
    assert_eq!(html.previously_focused_element(), None);
    assert!(!html.has_previously_focused_element());
    html.remember_focused_element(Some(17));
    assert_eq!(html.previously_focused_element(), Some(&17));
    assert!(html.has_previously_focused_element());
    html.remember_focused_element(Some(29));
    assert_eq!(html.previously_focused_element(), Some(&29));
    html.remember_focused_element(None);
    assert_eq!(html.previously_focused_element(), None);
    assert!(!html.has_previously_focused_element());
    html.remember_focused_element(Some(41));
    html.clear_previously_focused_element();
    assert_eq!(html.previously_focused_element(), None);
    assert!(!html.has_previously_focused_element());
    assert!(!html.has_attached_internals());
    assert!(html.popover_is_hidden());
    assert!(!html.has_popover_trigger());
    assert!(!html.popover_is_hiding());
    assert!(!html.has_popover_toggle_task_tracker());
    assert!(!html.has_popover_close_watcher());
    assert!(!html.has_popover_opened_in_mode());
    assert!(current.element().reaction_queue_is_empty());
    assert!(!current.element().click_in_progress());
}

#[test]
fn common_reference_mapping_moves_the_actual_noncopy_reference_once() {
    let mut current = HtmlCommonState::<String>::new(HtmlCommonScope::Html);
    current
        .html_mut()
        .unwrap()
        .remember_focused_element(Some("remembered element".to_owned()));
    let mut calls = 0;
    let mapped = current.map(|reference| {
        calls += 1;
        (7_u8, reference)
    });
    assert_eq!(calls, 1);
    assert_eq!(mapped.scope(), HtmlCommonScope::Html);
    let html = mapped.html().unwrap();
    assert!(html.has_previously_focused_element());
    let (domain, reference) = html.previously_focused_element().unwrap();
    assert_eq!(*domain, 7);
    assert_eq!(reference, "remembered element");
    assert!(html.popover_is_hidden());
    assert!(!html.has_attached_internals());
    assert!(mapped.element().reaction_queue_is_empty());
    assert!(!mapped.element().click_in_progress());
}

#[test]
fn null_only_profile_mapping_preserves_nullness_and_nonhtml_inapplicability() {
    for scope in [HtmlCommonScope::Element, HtmlCommonScope::Html] {
        // Native and admitted offline-parser profiles cannot supply a positive
        // reference; the consuming owner changes reference domains losslessly.
        let mapped: HtmlCommonState<u32> =
            HtmlCommonState::<Infallible>::new(scope).map(|reference| match reference {});
        assert_eq!(mapped.scope(), scope);
        if let Some(html) = mapped.html() {
            assert_eq!(html.previously_focused_element(), None);
            assert!(!html.has_previously_focused_element());
        } else {
            assert_eq!(scope, HtmlCommonScope::Element);
        }
    }
    let mut nonhtml = HtmlCommonState::<u32>::new(HtmlCommonScope::Element);
    assert!(nonhtml.html_mut().is_none());
    let mapped: HtmlCommonState<String> = nonhtml.map(|_| {
        panic!("non-HTML state cannot contain an HTML reference");
    });
    assert_eq!(mapped.scope(), HtmlCommonScope::Element);
    assert!(mapped.html().is_none());
}

#[test]
fn native_freshness_uses_one_classification_read_sequence_and_preserves_scopes() {
    for (namespace, local, scope, nonce) in [
        (Namespace::none(), "input", HtmlCommonScope::Element, None),
        (
            Namespace::html(),
            "input",
            HtmlCommonScope::Html,
            Some(HtmlNativeFreshNonce::Empty),
        ),
        (
            Namespace::html(),
            "template",
            HtmlCommonScope::Html,
            Some(HtmlNativeFreshNonce::Empty),
        ),
        (
            Namespace::svg(),
            "script",
            HtmlCommonScope::Element,
            Some(HtmlNativeFreshNonce::Empty),
        ),
        (
            Namespace::mathml(),
            "input",
            HtmlCommonScope::Element,
            Some(HtmlNativeFreshNonce::Empty),
        ),
    ] {
        let name = QualifiedName::new(namespace, None, local);
        let mut class_reads = Vec::new();
        let class = element_classification_with(&name, |read| {
            class_reads.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap();
        let mut native_reads = Vec::new();
        let initial = native_element_initialization_with(&name, |read| {
            native_reads.push(read);
            Ok::<_, Infallible>(())
        })
        .unwrap();
        assert_eq!(native_reads, class_reads);
        assert_eq!(initial.requirement(), class);
        assert_eq!(initial.common().scope(), scope);
        assert_eq!(initial.nonce(), nonce);
        for (cut, &denied) in class_reads.iter().enumerate() {
            let mut observed = 0;
            let result = element_classification_with(&name, |read| {
                assert!(observed <= cut, "read after first refusal");
                let position = observed;
                observed += 1;
                if position == cut {
                    Err((position, read))
                } else {
                    Ok(())
                }
            });
            assert_eq!(result, Err((cut, denied)));
            assert_eq!(observed, cut + 1);
        }
    }
}
