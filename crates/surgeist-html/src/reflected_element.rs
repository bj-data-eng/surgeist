//! Nullable Element reflection decisions, HTML 877d §2.6.1.

use crate::{KnownHtmlAttribute, QualifiedName};

/// Whether an actual attribute change clears this property's explicit target.
///
/// The caller establishes that `reflected` names the content attribute of an
/// applicable nullable Element reflected property on the actual element. This
/// decision does not classify property applicability. It applies the general
/// element-reflected-target rule: exact local name and null namespace clear the
/// explicit weak target, including when the old and new values are equal.
/// Prefix metadata has no effect. An absent removal that runs no attribute
/// change steps must not invoke this decision as a new change.
///
/// No allocation or value inspection occurs. Length rejection precedes unit
/// comparison, which reads at most the selected canonical spelling's length.
pub fn reflected_element_attribute_change(
    reflected: KnownHtmlAttribute,
    changed: &QualifiedName,
) -> bool {
    let expected = reflected.local_name().as_bytes();
    changed.namespace().uri().is_none()
        && changed.local_name().len() == expected.len()
        && changed
            .local_name()
            .code_units()
            .iter()
            .zip(expected)
            .all(|(&unit, &byte)| unit == u16::from(byte))
}

/// Source-ordered effects of setting a nullable Element reflected property.
///
/// The consumer owns the actual weak target, content attribute, identity checks,
/// admission, cancellation and atomic publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlReflectedElementSetterEffect {
    /// Set this property's explicit weak target to null.
    ClearExplicitTarget,
    /// Delete this property's content attribute, running its actual change steps.
    RemoveContentAttribute,
    /// Set the content attribute to the empty string, running its change steps.
    SetEmptyContentAttribute,
    /// Store a weak reference to the actual Element supplied to the setter.
    SetExplicitTargetFromSuppliedElement,
}

/// Allocation-free setter sequence; it stores no Element or identity stand-in.
///
/// Complete each effect, including nested attribute-change effects, before
/// requesting the next one. A null setter clears the explicit target before
/// deleting content. A non-null setter writes empty content before installing
/// the supplied target, so that write's clearing hook completes first.
#[derive(Clone, Copy, Debug)]
pub struct HtmlReflectedElementSetter {
    next: Option<HtmlReflectedElementSetterEffect>,
}

impl HtmlReflectedElementSetter {
    /// Begin the setter whose supplied value is null.
    pub const fn null() -> Self {
        Self {
            next: Some(HtmlReflectedElementSetterEffect::ClearExplicitTarget),
        }
    }

    /// Begin the setter whose supplied value is an actual non-null Element.
    ///
    /// The consumer retains and qualifies that Element; this cursor does not
    /// establish its identity, lifetime, tree scope or command support.
    pub const fn element() -> Self {
        Self {
            next: Some(HtmlReflectedElementSetterEffect::SetEmptyContentAttribute),
        }
    }

    /// Return the next effect, or `None` after the two effects are exhausted.
    pub fn next_effect(&mut self) -> Option<HtmlReflectedElementSetterEffect> {
        use HtmlReflectedElementSetterEffect::{
            ClearExplicitTarget, RemoveContentAttribute, SetEmptyContentAttribute,
            SetExplicitTargetFromSuppliedElement,
        };
        let effect = self.next?;
        self.next = match effect {
            ClearExplicitTarget => Some(RemoveContentAttribute),
            SetEmptyContentAttribute => Some(SetExplicitTargetFromSuppliedElement),
            RemoveContentAttribute | SetExplicitTargetFromSuppliedElement => None,
        };
        Some(effect)
    }
}
