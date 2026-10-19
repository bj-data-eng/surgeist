//! HTML §13.4 decisions over supplied values. Capture qualification and parser
//! execution/accounting belong to their adapters, never to this local owner.

use crate::DocumentString;
#[cfg(test)]
use crate::QualifiedName;
use std::convert::Infallible;

/// All four pinned parser modes. Fragment admission accepts requested Inert/Fragment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParserScriptingMode {
    Normal,
    Disabled,
    Inert,
    Fragment,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFragmentTokenizerState {
    Data,
    Rcdata,
    Rawtext,
    ScriptData,
    Plaintext,
}
/// Initial §13.4 insertion-mode selection; no complete parser state is claimed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFragmentInsertionMode {
    InBody,
    InRow,
    InTableBody,
    InCaption,
    InColumnGroup,
    InTable,
    InTemplate,
    InFrameset,
    BeforeHead,
}

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// The finite comparisons needed by fragment initialization. The consumer
/// admits and executes a comparison before the policy can inspect its result.
pub enum HtmlFragmentNameTest<'a> {
    /// Compare the supplied name's namespace with the HTML namespace.
    HtmlNamespace,
    /// Compare its local name with this ordered group of exact ASCII names.
    Local(&'a [&'static str]),
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum FixedInsertionScope {
    Fragment,
    StackReset,
}

/// One fixed mapping, with the fragment-only last-node decisions separated
/// from reset's state-sensitive template/head/html and non-last td/th rules.
/// The optional scope also preserves each algorithm's comparison order.
fn fixed_insertion_mode<E>(
    scope: FixedInsertionScope,
    mut compare: impl FnMut(&'static str) -> Result<bool, E>,
) -> Result<Option<HtmlFragmentInsertionMode>, E> {
    use FixedInsertionScope::{Fragment, StackReset};
    use HtmlFragmentInsertionMode as Mode;
    for (local, mode, only) in [
        ("tr", Mode::InRow, None),
        ("tbody", Mode::InTableBody, None),
        ("thead", Mode::InTableBody, None),
        ("tfoot", Mode::InTableBody, None),
        ("caption", Mode::InCaption, None),
        ("colgroup", Mode::InColumnGroup, None),
        ("table", Mode::InTable, None),
        ("template", Mode::InTemplate, Some(Fragment)),
        ("body", Mode::InBody, Some(StackReset)),
        ("frameset", Mode::InFrameset, None),
        ("html", Mode::BeforeHead, Some(Fragment)),
    ] {
        if only.is_none_or(|selected| selected == scope) && compare(local)? {
            return Ok(Some(mode));
        }
    }
    Ok(None)
}

fn html_tokenizer_state<E>(
    effective: HtmlParserScriptingMode,
    mut compare: impl FnMut(&[&'static str]) -> Result<bool, E>,
) -> Result<HtmlFragmentTokenizerState, E> {
    use HtmlFragmentTokenizerState as State;
    Ok(if compare(&["title", "textarea"])? {
        State::Rcdata
    } else if compare(&["style", "xmp", "iframe", "noembed", "noframes"])?
        || (compare(&["noscript"])? && effective != HtmlParserScriptingMode::Disabled)
    {
        State::Rawtext
    } else if compare(&["script"])? {
        State::ScriptData
    } else if compare(&["plaintext"])? {
        State::Plaintext
    } else {
        State::Data
    })
}

/// The supplied scripting mode has already been resolved by the caller. This
/// local decision neither reads captured facts nor promises execution support.
pub fn fragment_initial_modes<E>(
    effective: HtmlParserScriptingMode,
    mut compare: impl FnMut(HtmlFragmentNameTest<'_>) -> Result<bool, E>,
) -> Result<(HtmlFragmentTokenizerState, HtmlFragmentInsertionMode), E> {
    if !compare(HtmlFragmentNameTest::HtmlNamespace)? {
        return Ok((
            HtmlFragmentTokenizerState::Data,
            HtmlFragmentInsertionMode::InBody,
        ));
    }
    let tokenizer = html_tokenizer_state(effective, |choices| {
        compare(HtmlFragmentNameTest::Local(choices))
    })?;
    let insertion = fixed_insertion_mode(FixedInsertionScope::Fragment, |local| {
        compare(HtmlFragmentNameTest::Local(&[local]))
    })?
    .unwrap_or(HtmlFragmentInsertionMode::InBody);
    Ok((tokenizer, insertion))
}

/// The ordinary value adapter compares exact borrowed UTF-16. It does not
/// normalize, allocate, or take a parser controller.
fn exact(value: &DocumentString, literal: &str) -> bool {
    value.len() == literal.len()
        && value
            .code_units()
            .iter()
            .copied()
            .eq(literal.bytes().map(u16::from))
}
fn html_namespace(name: &(impl crate::HtmlNameInput + ?Sized)) -> bool {
    name.namespace_uri()
        .is_some_and(|uri| exact(uri, HTML_NAMESPACE))
}

/// Selects only the tokenizer state from an exact name and already-resolved mode.
/// No capture facts or execution capability are read or inferred.
pub fn fragment_tokenizer_state(
    name: &(impl crate::HtmlNameInput + ?Sized),
    effective: HtmlParserScriptingMode,
) -> HtmlFragmentTokenizerState {
    if !html_namespace(name) {
        return HtmlFragmentTokenizerState::Data;
    }
    html_tokenizer_state(effective, |choices| {
        Ok::<_, Infallible>(choices.iter().any(|local| exact(name.local_name(), local)))
    })
    .unwrap()
}

/// Selects only the fragment's initial insertion mode from its exact context name.
/// Scripting status and tokenizer state do not influence this operation.
pub fn fragment_insertion_mode(
    name: &(impl crate::HtmlNameInput + ?Sized),
) -> HtmlFragmentInsertionMode {
    if !html_namespace(name) {
        return HtmlFragmentInsertionMode::InBody;
    }
    fixed_insertion_mode(FixedInsertionScope::Fragment, |local| {
        Ok::<_, Infallible>(exact(name.local_name(), local))
    })
    .unwrap()
    .unwrap_or(HtmlFragmentInsertionMode::InBody)
}

/// Selects the fixed rows of a parser stack reset in their specified order.
/// The caller retains state-sensitive template, cell, head and HTML decisions.
pub fn stack_reset_insertion_mode<E>(
    compare: impl FnMut(&'static str) -> Result<bool, E>,
) -> Result<Option<HtmlFragmentInsertionMode>, E> {
    fixed_insertion_mode(FixedInsertionScope::StackReset, compare)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplied_effective_scripting_selects_noscript_without_execution_validation() {
        // The local HTML algorithm consumes a resolved mode. Qualification and
        // capability rejection remain the capture/request owner's job.
        let name = QualifiedName::new(crate::Namespace::html(), Some("prefix".into()), "noscript");
        for effective in [
            HtmlParserScriptingMode::Normal,
            HtmlParserScriptingMode::Disabled,
            HtmlParserScriptingMode::Inert,
            HtmlParserScriptingMode::Fragment,
        ] {
            let expected = if effective == HtmlParserScriptingMode::Disabled {
                HtmlFragmentTokenizerState::Data
            } else {
                HtmlFragmentTokenizerState::Rawtext
            };
            assert_eq!(fragment_tokenizer_state(&name, effective), expected);
            let modes = fragment_initial_modes(effective, |test| {
                Ok::<_, Infallible>(match test {
                    HtmlFragmentNameTest::HtmlNamespace => html_namespace(&name),
                    HtmlFragmentNameTest::Local(choices) => {
                        choices.iter().any(|local| exact(name.local_name(), local))
                    }
                })
            })
            .unwrap();
            assert_eq!(modes, (expected, HtmlFragmentInsertionMode::InBody));
        }
    }
}
