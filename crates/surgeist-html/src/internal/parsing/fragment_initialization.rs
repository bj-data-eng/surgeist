//! HTML §13.4 decisions over supplied values. Capture qualification and parser
//! execution/accounting belong to their adapters, never to this local owner.

use crate::internal::values::DocumentString;
#[cfg(test)]
use crate::internal::values::QualifiedName;
use std::convert::Infallible;

/// All four pinned parser modes. Fragment admission accepts requested Inert/Fragment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParserScriptingMode {
    /// Scripts run when inserted, respecting `async` and `defer` (#scripting-mode-normal).
    Normal,
    /// Scripts are disabled, so `noscript` represents fallback content (#scripting-mode-disabled).
    Disabled,
    /// Scripts are enabled but marked already started so they never execute; the fragment
    /// parsing algorithm's default (#scripting-mode-inert).
    Inert,
    /// Scripts execute as soon as the fragment parsing algorithm inserts them
    /// (#scripting-mode-fragment).
    Fragment,
}
/// The tokenizer state a fragment parse starts in, chosen from its context element.
///
/// See [`fragment_tokenizer_state`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFragmentTokenizerState {
    /// The data state: the context is an HTML element that is not text-like, a `noscript` while
    /// the scripting mode is Disabled, or any context outside the HTML namespace.
    Data,
    /// The RCDATA state, for `title` and `textarea` contexts.
    Rcdata,
    /// The RAWTEXT state, for `style`, `xmp`, `iframe`, `noembed` and `noframes` contexts, and
    /// for `noscript` unless the scripting mode is Disabled.
    Rawtext,
    /// The script data state, for a `script` context.
    ScriptData,
    /// The PLAINTEXT state, for a `plaintext` context.
    Plaintext,
}
/// Initial §13.4 insertion-mode selection; no complete parser state is claimed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFragmentInsertionMode {
    /// The "in body" mode: the fallback for any context without a more specific mode, and the
    /// result for every non-HTML-namespace context.
    InBody,
    /// The "in row" mode, for a `tr` context.
    InRow,
    /// The "in table body" mode, for `tbody`, `thead` and `tfoot` contexts.
    InTableBody,
    /// The "in caption" mode, for a `caption` context.
    InCaption,
    /// The "in column group" mode, for a `colgroup` context.
    InColumnGroup,
    /// The "in table" mode, for a `table` context.
    InTable,
    /// The "in template" mode, for a `template` context in fragment initialization.
    InTemplate,
    /// The "in frameset" mode, for a `frameset` context.
    InFrameset,
    /// The "before head" mode, for an `html` context in fragment initialization.
    BeforeHead,
}

/// The finite comparisons needed by fragment initialization. The consumer
/// admits and executes a comparison before the policy can inspect its result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
fn html_namespace(
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
) -> bool {
    crate::internal::values::is_html_namespace_uri(name.namespace_uri())
}

/// Selects only the tokenizer state from an exact name and already-resolved mode.
/// No capture facts or execution capability are read or inferred.
pub fn fragment_tokenizer_state(
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
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
    name: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
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
        let name = QualifiedName::new(
            crate::internal::values::Namespace::html(),
            Some("prefix".into()),
            "noscript",
        );
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
