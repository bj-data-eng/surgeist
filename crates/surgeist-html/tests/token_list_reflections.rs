//! HTML877d association and supported-token requirements.

use std::cell::Cell;

use surgeist_html::{
    attributes::sandbox::HtmlSandboxKeyword,
    elements::{
        link::HtmlRelKeyword,
        token_list::{HtmlTokenListReflection, HtmlTokenListSupport},
    },
    vocabulary::KnownHtmlElement,
};

#[test]
fn each_reflection_names_the_exact_html_element_and_attribute() {
    use HtmlTokenListReflection::*;
    use KnownHtmlElement as Element;

    for (reflection, element, attribute) in [
        (ScriptBlocking, Element::Script, "blocking"),
        (LinkBlocking, Element::Link, "blocking"),
        (StyleBlocking, Element::Style, "blocking"),
        (AnchorRel, Element::A, "rel"),
        (AreaRel, Element::Area, "rel"),
        (FormRel, Element::Form, "rel"),
        (LinkRel, Element::Link, "rel"),
        (LinkSizes, Element::Link, "sizes"),
        (IframeSandbox, Element::Iframe, "sandbox"),
        (OutputHtmlFor, Element::Output, "for"),
    ] {
        assert_eq!(reflection.element(), element);
        assert_eq!(reflection.attribute(), attribute);
        assert!(
            reflection
                .attribute_units()
                .iter()
                .copied()
                .eq(attribute.encode_utf16())
        );
    }
}

#[test]
fn supported_token_policies_distinguish_host_candidates_from_defined_membership() {
    use HtmlTokenListReflection::*;
    use HtmlTokenListSupport as Support;

    let cases = [
        (ScriptBlocking, "RENDER", Support::Supported),
        (LinkBlocking, "rEnDeR", Support::Supported),
        (StyleBlocking, "render", Support::Supported),
        (StyleBlocking, " render", Support::Unsupported),
        (LinkBlocking, "render script", Support::Unsupported),
        (ScriptBlocking, "", Support::Unsupported),
        (
            AnchorRel,
            "NoOpEnEr",
            Support::RelCandidate(HtmlRelKeyword::NoOpener),
        ),
        (
            AreaRel,
            "noreferrer",
            Support::RelCandidate(HtmlRelKeyword::NoReferrer),
        ),
        (
            FormRel,
            "opener",
            Support::RelCandidate(HtmlRelKeyword::Opener),
        ),
        (FormRel, "external", Support::Unsupported),
        (AnchorRel, "stylesheet", Support::Unsupported),
        (AreaRel, "noreferrer opener", Support::Unsupported),
        (
            LinkRel,
            "stylesheet",
            Support::RelCandidate(HtmlRelKeyword::Stylesheet),
        ),
        (
            LinkRel,
            "expect",
            Support::RelCandidate(HtmlRelKeyword::Expect),
        ),
        (LinkRel, "noopener", Support::Unsupported),
        (LinkRel, "canonical", Support::Unsupported),
        (LinkRel, "previous", Support::Unsupported),
        (LinkRel, "shortcut", Support::Unsupported),
        (
            IframeSandbox,
            "ALLOW-SCRIPTS",
            Support::SandboxCandidate(HtmlSandboxKeyword::AllowScripts),
        ),
        (
            IframeSandbox,
            "allow-same-origin",
            Support::SandboxCandidate(HtmlSandboxKeyword::AllowSameOrigin),
        ),
        (
            IframeSandbox,
            "allow-pointer-lock",
            Support::SandboxCandidate(HtmlSandboxKeyword::AllowPointerLock),
        ),
        (IframeSandbox, "allow-unknown", Support::Unsupported),
        (LinkSizes, "any", Support::Undefined),
        (OutputHtmlFor, "control", Support::Undefined),
    ];
    for (reflection, token, expected) in cases {
        let units = token.encode_utf16().collect::<Vec<_>>();
        let spent = Cell::new(0);
        let observe = |work| {
            spent.set(0);
            reflection.support_with(&units, || {
                if spent.get() == work {
                    return Err(());
                }
                spent.set(spent.get() + 1);
                Ok(())
            })
        };
        assert_eq!(observe(usize::MAX), Ok(expected), "{reflection:?} {token}");
        let exact = spent.get();
        assert!(exact > 0);
        assert_eq!(observe(exact - 1), Err(()));
        assert_eq!(spent.get(), exact - 1);
        assert_eq!(observe(exact), Ok(expected));
    }
    assert_eq!(
        LinkRel.support_with(&[0xd800, 0x0065, 0x006c], || Ok::<_, ()>(())),
        Ok(Support::Unsupported)
    );
}
