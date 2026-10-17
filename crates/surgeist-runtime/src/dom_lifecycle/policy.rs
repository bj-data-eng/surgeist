//! The selected fullscreen directive projection for an actual opaque target.
//! Runtime owns the Permissions Policy algorithm; HTML owns sandbox values and
//! canonical DOM owns every origin and retained request input.
use super::*;
use surgeist_html::DocumentString;

/// The only allowlist parts that can match the selected opaque child origin.
/// URL expressions cannot match it: allowlist matching returns false for opaque
/// origins before URL/expression matching. This is no stored general allowlist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FullscreenDirective {
    All,
    Origins { parent: bool, declared: bool },
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
) -> Result<Option<FullscreenDirective>, DomHostedRuntimeError> {
    let mut tokens = Tokens {
        units,
        index: start,
        end,
    };
    let Some(feature) = tokens.next(work, limit, cancel)? else {
        return Ok(None);
    };
    if !keyword(feature, b"fullscreen", false, work, limit, cancel)? {
        return Ok(None);
    }
    let target_start = tokens.index;
    let mut any = false;
    while let Some(target) = tokens.next(work, limit, cancel)? {
        any = true;
        if keyword(target, b"*", false, work, limit, cancel)? {
            return Ok(Some(FullscreenDirective::All));
        }
    }
    if !any {
        return Ok(Some(FullscreenDirective::Origins {
            parent: false,
            declared: true,
        }));
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
    Ok(Some(FullscreenDirective::Origins { parent, declared }))
}

pub(super) fn fullscreen_directive(
    allow: Option<&DocumentString>,
    allow_fullscreen: bool,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<Option<FullscreenDirective>, DomHostedRuntimeError> {
    spend(work, limit, cancel)?;
    let mut result = None;
    if let Some(allow) = allow {
        let units = allow.code_units();
        let mut start = 0;
        for index in 0..=units.len() {
            if index == units.len() || unit(units, index, work, limit, cancel)? == u16::from(b';') {
                if let Some(value) = declaration(units, start, index, work, limit, cancel)? {
                    // The source map assignment replaces an earlier directive.
                    result = Some(value);
                }
                start = index + 1;
            }
        }
    }
    spend(work, limit, cancel)?;
    if result.is_none() && allow_fullscreen {
        result = Some(FullscreenDirective::All);
    }
    Ok(result)
}

/// The reached declared-origin work precedes directive parsing, even when a
/// later wildcard would make the actual declared origin irrelevant to matching.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Decision {
    Complete(surgeist_dom::DomFullscreenInherited),
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
    use surgeist_dom::{DomFullscreenInherited as Value, DomHostedEmbedderKind};
    use surgeist_html::HtmlSandboxFlag;
    spend(work, limit, cancel)?;
    let Some(inputs) = request.embedder_policy() else {
        if request.parent_document().is_some() || request.embedder().is_some() {
            return Err(DomHostedRuntimeError::Protocol);
        }
        return Ok(Decision::Complete(Value::Enabled));
    };
    // Both Get feature value calls inspect the same actual parent inherited
    // value. Its declared map is the source-created empty map in this profile;
    // Get feature value does not apply the default self allowlist here.
    spend(work, limit, cancel)?;
    if inputs.parent_fullscreen() == Value::Disabled {
        return Ok(Decision::Complete(Value::Disabled));
    }
    spend(work, limit, cancel)?;
    let parent_matches = inputs.parent_origin() == request.target_origin();
    let enabled = if inputs.kind() == DomHostedEmbedderKind::Frame {
        // A Frame's attribute algorithm returns an empty directive before the
        // declared-origin algorithm or Iframe-only raw attributes are reached.
        parent_matches
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
        match fullscreen_directive(
            inputs.allow(),
            inputs.allow_fullscreen(),
            work,
            limit,
            cancel,
        )? {
            Some(FullscreenDirective::All) => true,
            Some(FullscreenDirective::Origins {
                parent,
                declared: src,
            }) => (parent && parent_matches) || (src && declared),
            None => parent_matches,
        }
    };
    Ok(Decision::Complete(if enabled {
        Value::Enabled
    } else {
        Value::Disabled
    }))
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
                Some(FullscreenDirective::Origins {
                    parent: false,
                    declared: true,
                }),
            ),
            (
                "fullscreen 'none'; fullscreen 'SELF' 'sRc'",
                false,
                Some(FullscreenDirective::Origins {
                    parent: true,
                    declared: true,
                }),
            ),
            (
                "fullscreen 'src'; fullscreen 'none'",
                true,
                Some(FullscreenDirective::Origins {
                    parent: false,
                    declared: false,
                }),
            ),
            ("geolocation *", false, None),
            (
                "fullscreen unobserved-expression * 'self'",
                false,
                Some(FullscreenDirective::All),
            ),
            ("", true, Some(FullscreenDirective::All)),
        ] {
            let mut work = 0;
            assert_eq!(
                fullscreen_directive(Some(&raw.into()), fallback, &mut work, 10_000, &cancel)
                    .unwrap(),
                expected
            );
        }
    }
    #[test]
    fn actual_raw_reads_are_cancelled_and_one_short_without_an_output() {
        let cancel = AtomicBool::new(false);
        let raw = DocumentString::from("fullscreen 'SELF' 'src'; fullscreen 'none'");
        let mut exact = 0;
        let expected =
            fullscreen_directive(Some(&raw), false, &mut exact, 10_000, &cancel).unwrap();
        let mut short = 0;
        assert_eq!(
            fullscreen_directive(Some(&raw), false, &mut short, exact - 1, &cancel),
            Err(DomHostedRuntimeError::Admission)
        );
        assert_eq!(short, exact - 1);
        let mut work = 0;
        assert_eq!(
            fullscreen_directive(Some(&raw), false, &mut work, 10_000, &AtomicBool::new(true)),
            Err(DomHostedRuntimeError::Cancelled)
        );
        assert_eq!(work, 0);
        assert_eq!(
            expected,
            Some(FullscreenDirective::Origins {
                parent: false,
                declared: false
            })
        );
    }
}
