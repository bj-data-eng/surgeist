//! The fixed two-feature directive projection for an actual opaque target.
//! Runtime owns the Permissions Policy algorithm; HTML owns sandbox values and
//! canonical DOM owns every origin and retained request input.
use super::*;
use surgeist_html::DocumentString;

/// The only allowlist parts that can match the selected opaque child origin.
/// URL expressions cannot match it: allowlist matching returns false for opaque
/// origins before URL/expression matching. This is no stored general allowlist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpaqueDirective {
    All,
    Origins { parent: bool, declared: bool },
}
#[derive(Clone, Copy)]
enum Feature {
    Fullscreen,
    FocusWithoutUserActivation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Directives {
    fullscreen: Option<OpaqueDirective>,
    focus_without_user_activation: Option<OpaqueDirective>,
}

fn unit(
    units: &[u16],
    index: usize,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<u16, DomHostedRuntimeError> {
    spend(work, limit, cancel)?;
    Ok(units[index])
}
fn space(value: u16) -> bool {
    matches!(value, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
}
struct Tokens<'a> {
    units: &'a [u16],
    index: usize,
    end: usize,
}
impl<'a> Tokens<'a> {
    fn next(
        &mut self,
        work: &mut u64,
        limit: u64,
        cancel: &AtomicBool,
    ) -> Result<Option<&'a [u16]>, DomHostedRuntimeError> {
        while self.index < self.end {
            if !space(unit(self.units, self.index, work, limit, cancel)?) {
                break;
            }
            self.index += 1;
        }
        if self.index == self.end {
            return Ok(None);
        }
        let start = self.index;
        while self.index < self.end {
            if space(unit(self.units, self.index, work, limit, cancel)?) {
                break;
            }
            self.index += 1;
        }
        Ok(Some(&self.units[start..self.index]))
    }
}
fn keyword(
    token: &[u16],
    expected: &[u8],
    insensitive: bool,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<bool, DomHostedRuntimeError> {
    spend(work, limit, cancel)?;
    if token.len() != expected.len() {
        return Ok(false);
    }
    for (index, expected) in expected.iter().enumerate() {
        let mut value = unit(token, index, work, limit, cancel)?;
        if insensitive && (u16::from(b'A')..=u16::from(b'Z')).contains(&value) {
            value += u16::from(b'a' - b'A');
        }
        if value != u16::from(*expected) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn declaration(
    units: &[u16],
    start: usize,
    end: usize,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<Option<(Feature, OpaqueDirective)>, DomHostedRuntimeError> {
    let mut tokens = Tokens {
        units,
        index: start,
        end,
    };
    let Some(feature) = tokens.next(work, limit, cancel)? else {
        return Ok(None);
    };
    let feature = if keyword(feature, b"fullscreen", false, work, limit, cancel)? {
        Feature::Fullscreen
    } else if keyword(
        feature,
        b"focus-without-user-activation",
        false,
        work,
        limit,
        cancel,
    )? {
        Feature::FocusWithoutUserActivation
    } else {
        return Ok(None);
    };
    let target_start = tokens.index;
    let mut any = false;
    while let Some(target) = tokens.next(work, limit, cancel)? {
        any = true;
        if keyword(target, b"*", false, work, limit, cancel)? {
            return Ok(Some((feature, OpaqueDirective::All)));
        }
    }
    if !any {
        return Ok(Some((
            feature,
            OpaqueDirective::Origins {
                parent: false,
                declared: true,
            },
        )));
    }
    tokens.index = target_start;
    let mut parent = false;
    let mut declared = false;
    while let Some(target) = tokens.next(work, limit, cancel)? {
        if keyword(target, b"'self'", true, work, limit, cancel)? {
            parent = true;
        } else if keyword(target, b"'src'", true, work, limit, cancel)? {
            declared = true;
        }
        // Other expressions cannot match the actual opaque target. No tuple
        // origin, general URL parser or expression matcher is claimed here.
    }
    Ok(Some((
        feature,
        OpaqueDirective::Origins { parent, declared },
    )))
}

fn directives(
    allow: Option<&DocumentString>,
    allow_fullscreen: bool,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<Directives, DomHostedRuntimeError> {
    spend(work, limit, cancel)?;
    let mut result = Directives {
        fullscreen: None,
        focus_without_user_activation: None,
    };
    if let Some(allow) = allow {
        let units = allow.code_units();
        let mut start = 0;
        for index in 0..=units.len() {
            if index == units.len() || unit(units, index, work, limit, cancel)? == u16::from(b';') {
                if let Some((feature, value)) =
                    declaration(units, start, index, work, limit, cancel)?
                {
                    // Each actual feature's source map assignment replaces only
                    // its own earlier declaration.
                    match feature {
                        Feature::Fullscreen => result.fullscreen = Some(value),
                        Feature::FocusWithoutUserActivation => {
                            result.focus_without_user_activation = Some(value)
                        }
                    }
                }
                start = index + 1;
            }
        }
    }
    spend(work, limit, cancel)?;
    if result.fullscreen.is_none() && allow_fullscreen {
        result.fullscreen = Some(OpaqueDirective::All);
    }
    Ok(result)
}

/// The reached declared-origin work precedes directive parsing, even when a
/// later wildcard would make the actual declared origin irrelevant to matching.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Decision {
    Complete(surgeist_dom::DomInitialPermissionsPolicy),
    SandboxOrigin,
    Url,
}

pub(super) fn inherited(
    request: &surgeist_dom::DomHostedPolicyRequest,
    declared_matches: Option<bool>,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<Decision, DomHostedRuntimeError> {
    use surgeist_dom::{
        DomHostedEmbedderKind, DomInitialPermissionsPolicy as Policy,
        DomPermissionsPolicyValue as Value,
    };
    use surgeist_html::HtmlSandboxFlag;
    spend(work, limit, cancel)?;
    let Some(inputs) = request.embedder_policy() else {
        if request.parent_document().is_some() || request.embedder().is_some() {
            return Err(DomHostedRuntimeError::Protocol);
        }
        return Ok(Decision::Complete(Policy::new(
            Value::Enabled,
            Value::Enabled,
        )));
    };
    // Read each feature's own inherited parent value. The selected profile's
    // declared map is actually empty; neither value defaults from the other.
    spend(work, limit, cancel)?;
    let parent_fullscreen = inputs.parent_policy().inherited_fullscreen();
    spend(work, limit, cancel)?;
    let parent_focus = inputs
        .parent_policy()
        .inherited_focus_without_user_activation();
    if parent_fullscreen == Value::Disabled && parent_focus == Value::Disabled {
        return Ok(Decision::Complete(Policy::new(
            Value::Disabled,
            Value::Disabled,
        )));
    }
    spend(work, limit, cancel)?;
    let parent_matches = inputs.parent_origin() == request.target_origin();
    let parsed = if inputs.kind() == DomHostedEmbedderKind::Frame {
        // A Frame's attribute algorithm returns an empty directive before the
        // declared-origin algorithm or Iframe-only raw attributes are reached.
        Directives {
            fullscreen: None,
            focus_without_user_activation: None,
        }
    } else {
        let declared = if let Some(matches) = declared_matches {
            matches
        } else {
            spend(work, limit, cancel)?;
            if inputs
                .parent_sandbox_flags()
                .contains(HtmlSandboxFlag::Origin)
            {
                return Ok(Decision::SandboxOrigin);
            }
            // Declared origin reads the present raw attribute. The retained
            // Iframe flags slot separately owns child-creation sandbox state;
            // it can differ after an attribute change while content is null.
            spend(work, limit, cancel)?;
            if let Some(raw) = inputs.sandbox() {
                let flags = surgeist_html::HtmlSandboxFlags::parse_with(raw, || {
                    spend(work, limit, cancel)
                })?;
                if flags.contains(HtmlSandboxFlag::Origin) {
                    return Ok(Decision::SandboxOrigin);
                }
            }
            spend(work, limit, cancel)?;
            if inputs.srcdoc().is_some() {
                parent_matches
            } else if inputs.src().is_some() {
                return Ok(Decision::Url);
            } else {
                parent_matches
            }
        };
        let parsed = directives(
            inputs.allow(),
            inputs.allow_fullscreen(),
            work,
            limit,
            cancel,
        )?;
        return Ok(Decision::Complete(Policy::new(
            inherited_feature(
                parent_fullscreen,
                parsed.fullscreen,
                parent_matches,
                declared,
            ),
            inherited_feature(
                parent_focus,
                parsed.focus_without_user_activation,
                parent_matches,
                declared,
            ),
        )));
    };
    Ok(Decision::Complete(Policy::new(
        inherited_feature(parent_fullscreen, parsed.fullscreen, parent_matches, false),
        inherited_feature(
            parent_focus,
            parsed.focus_without_user_activation,
            parent_matches,
            false,
        ),
    )))
}

fn inherited_feature(
    parent: surgeist_dom::DomPermissionsPolicyValue,
    directive: Option<OpaqueDirective>,
    parent_matches: bool,
    declared_matches: bool,
) -> surgeist_dom::DomPermissionsPolicyValue {
    use surgeist_dom::DomPermissionsPolicyValue::{Disabled, Enabled};
    if parent == Disabled {
        return Disabled;
    }
    let enabled = match directive {
        Some(OpaqueDirective::All) => true,
        Some(OpaqueDirective::Origins { parent, declared }) => {
            (parent && parent_matches) || (declared && declared_matches)
        }
        None => parent_matches,
    };
    if enabled { Enabled } else { Disabled }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opaque_fullscreen_directives_follow_literal_defaults_and_last_assignment() {
        let cancel = AtomicBool::new(false);
        for (raw, fallback, expected) in [
            ("", false, None),
            (
                "fullscreen",
                false,
                Some(OpaqueDirective::Origins {
                    parent: false,
                    declared: true,
                }),
            ),
            (
                "fullscreen 'none'; fullscreen 'SELF' 'sRc'",
                false,
                Some(OpaqueDirective::Origins {
                    parent: true,
                    declared: true,
                }),
            ),
            (
                "fullscreen 'src'; fullscreen 'none'",
                true,
                Some(OpaqueDirective::Origins {
                    parent: false,
                    declared: false,
                }),
            ),
            ("geolocation *", false, None),
            (
                "fullscreen unobserved-expression * 'self'",
                false,
                Some(OpaqueDirective::All),
            ),
            ("", true, Some(OpaqueDirective::All)),
        ] {
            let mut work = 0;
            assert_eq!(
                directives(Some(&raw.into()), fallback, &mut work, 10_000, &cancel)
                    .unwrap()
                    .fullscreen,
                expected
            );
        }
    }
    #[test]
    fn actual_raw_reads_are_cancelled_and_one_short_without_an_output() {
        let cancel = AtomicBool::new(false);
        let raw = DocumentString::from("fullscreen 'SELF' 'src'; fullscreen 'none'");
        let mut exact = 0;
        let expected = directives(Some(&raw), false, &mut exact, 10_000, &cancel).unwrap();
        let mut short = 0;
        assert_eq!(
            directives(Some(&raw), false, &mut short, exact - 1, &cancel),
            Err(DomHostedRuntimeError::Admission)
        );
        assert_eq!(short, exact - 1);
        let mut work = 0;
        assert_eq!(
            directives(Some(&raw), false, &mut work, 10_000, &AtomicBool::new(true)),
            Err(DomHostedRuntimeError::Cancelled)
        );
        assert_eq!(work, 0);
        assert_eq!(
            expected.fullscreen,
            Some(OpaqueDirective::Origins {
                parent: false,
                declared: false
            })
        );
    }
    #[test]
    fn exact_two_features_replace_independently_and_legacy_fallback_only_sets_fullscreen() {
        let none = Some(OpaqueDirective::Origins {
            parent: false,
            declared: false,
        });
        let own = Some(OpaqueDirective::Origins {
            parent: true,
            declared: false,
        });
        for (raw, fallback, fullscreen, focus) in [
            ("focus-without-user-activation 'none'", false, None, none),
            (
                "focus-without-user-activation 'none'",
                true,
                Some(OpaqueDirective::All),
                none,
            ),
            ("fullscreen 'none'", true, none, None),
            (
                "focus-without-user-activation *; fullscreen 'none'; focus-without-user-activation 'self'",
                false,
                none,
                own,
            ),
            (
                "fullscreen 'self'; focus-without-user-activation 'none'; fullscreen *",
                false,
                Some(OpaqueDirective::All),
                none,
            ),
            ("FOCUS-WITHOUT-USER-ACTIVATION 'none'", false, None, None),
        ] {
            let mut work = 0;
            assert_eq!(
                directives(
                    Some(&raw.into()),
                    fallback,
                    &mut work,
                    10_000,
                    &AtomicBool::new(false)
                )
                .unwrap(),
                Directives {
                    fullscreen,
                    focus_without_user_activation: focus
                }
            );
        }
    }
}
