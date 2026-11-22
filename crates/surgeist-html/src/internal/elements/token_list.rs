//! HTML's Element and content-attribute associations for live DOMTokenList views.

use crate::{
    internal::{
        attributes::sandbox::{HtmlSandboxKeyword, sandbox_keyword_with},
        elements::{
            rel::{HtmlRelKeyword, HtmlRelRole, standard_keyword_with},
            style::blocking_token_is_render_with,
        },
    },
    vocabulary::{KnownHtmlAttribute, KnownHtmlElement},
};

/// One HTML DOMTokenList reflection and its exact associated Element family.
///
/// The DOM retains association identity and the current content attribute.
/// This declaration supplies the HTML-specific name and support policy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlTokenListReflection {
    /// The `HTMLScriptElement.blocking` reflection.
    ScriptBlocking,
    /// The `HTMLLinkElement.blocking` reflection.
    LinkBlocking,
    /// The `HTMLStyleElement.blocking` reflection.
    StyleBlocking,
    /// The `HTMLAnchorElement.relList` reflection.
    AnchorRel,
    /// The `HTMLAreaElement.relList` reflection.
    AreaRel,
    /// The `HTMLFormElement.relList` reflection.
    FormRel,
    /// The `HTMLLinkElement.relList` reflection.
    LinkRel,
    /// The `HTMLLinkElement.sizes` reflection.
    LinkSizes,
    /// The `HTMLIFrameElement.sandbox` reflection.
    IframeSandbox,
    /// The `HTMLOutputElement.htmlFor` reflection.
    OutputHtmlFor,
}

/// HTML's result of classifying one DOMTokenList `supports()` argument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTokenListSupport {
    /// HTML defines no supported-token set for this association.
    Undefined,
    /// The token belongs to the supported set defined by HTML itself.
    Supported,
    /// The defined set excludes this token without consulting the host.
    Unsupported,
    /// The possible link token requires the host's actual processing capability.
    RelCandidate(HtmlRelKeyword),
    /// The allowed sandbox token requires the host's actual directive support.
    SandboxCandidate(HtmlSandboxKeyword),
}

impl HtmlTokenListReflection {
    /// The exact HTML Element family to which the association belongs.
    pub const fn element(self) -> KnownHtmlElement {
        match self {
            Self::ScriptBlocking => KnownHtmlElement::Script,
            Self::LinkBlocking | Self::LinkRel | Self::LinkSizes => KnownHtmlElement::Link,
            Self::StyleBlocking => KnownHtmlElement::Style,
            Self::AnchorRel => KnownHtmlElement::A,
            Self::AreaRel => KnownHtmlElement::Area,
            Self::FormRel => KnownHtmlElement::Form,
            Self::IframeSandbox => KnownHtmlElement::Iframe,
            Self::OutputHtmlFor => KnownHtmlElement::Output,
        }
    }

    /// The exact associated local name in the null namespace.
    pub const fn attribute(self) -> &'static str {
        self.content_attribute().0.local_name()
    }

    /// The exact associated local name as static UTF-16 units.
    pub const fn attribute_units(self) -> &'static [u16] {
        self.content_attribute().1
    }

    const fn content_attribute(self) -> (KnownHtmlAttribute, &'static [u16]) {
        use KnownHtmlAttribute as Attribute;
        match self {
            Self::ScriptBlocking | Self::LinkBlocking | Self::StyleBlocking => (
                Attribute::Blocking,
                &const { attribute_units::<8>(Attribute::Blocking) },
            ),
            Self::AnchorRel | Self::AreaRel | Self::FormRel | Self::LinkRel => (
                Attribute::Rel,
                &const { attribute_units::<3>(Attribute::Rel) },
            ),
            Self::LinkSizes => (
                Attribute::Sizes,
                &const { attribute_units::<5>(Attribute::Sizes) },
            ),
            Self::IframeSandbox => (
                Attribute::Sandbox,
                &const { attribute_units::<7>(Attribute::Sandbox) },
            ),
            Self::OutputHtmlFor => (
                Attribute::For,
                &const { attribute_units::<3>(Attribute::For) },
            ),
        }
    }

    /// Classify the whole argument under HTML877d and DOM's ASCII-lowercase
    /// comparison. Charge branch/table access and each reached comparison
    /// before inspecting it. The classification allocates no storage.
    pub fn support_with<E>(
        self,
        token: &[u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlTokenListSupport, E> {
        use HtmlTokenListSupport as Support;
        step()?;
        let role = match self {
            Self::ScriptBlocking | Self::LinkBlocking | Self::StyleBlocking => {
                return Ok(if blocking_token_is_render_with(token, step)? {
                    Support::Supported
                } else {
                    Support::Unsupported
                });
            }
            Self::LinkSizes | Self::OutputHtmlFor => return Ok(Support::Undefined),
            Self::IframeSandbox => {
                return Ok(sandbox_keyword_with(token, &mut step)?
                    .map_or(Support::Unsupported, Support::SandboxCandidate));
            }
            Self::AnchorRel => HtmlRelRole::Anchor,
            Self::AreaRel => HtmlRelRole::Area,
            Self::FormRel => HtmlRelRole::Form,
            Self::LinkRel => HtmlRelRole::Link,
        };
        let keyword = standard_keyword_with(token, |work| {
            for _ in 0..work {
                step()?;
            }
            Ok::<_, E>(())
        })?;
        step()?;
        Ok(keyword
            .filter(|keyword| keyword.possible_supported_token(role))
            .map_or(Support::Unsupported, Support::RelCandidate))
    }
}

const fn attribute_units<const N: usize>(attribute: KnownHtmlAttribute) -> [u16; N] {
    let bytes = attribute.local_name().as_bytes();
    assert!(bytes.len() == N);
    let mut units = [0; N];
    let mut index = 0;
    while index < N {
        assert!(bytes[index].is_ascii());
        units[index] = bytes[index] as u16;
        index += 1;
    }
    units
}
