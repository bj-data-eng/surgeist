//! Shared HTML877d intrinsic Meta processing. No Document/URL/CSS/UI controller.
use super::*;
use std::ops::Range;

/// Caller admission before a shared refresh operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaRefreshAction {
    /// One source cursor read or constant processing step.
    Step,
    /// The initial digit sequence has been collected. An empty range denotes
    /// exact zero. Consumers needing a materialized integer do so here, before
    /// suffix processing, using the existing HTML numeric owner.
    DelayDigits(Range<usize>),
}
/// Source-selected target before any external URL processing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMetaRefreshSyntaxTarget {
    /// Use the actual current Document URL.
    DocumentUrl,
    /// Encoding-parse this exact original UTF-16 range relative to the Document.
    UrlString(Range<usize>),
}

/// Exact source default, independently of a fetching attribute's empty fallback.
pub const fn default_document_referrer_policy() -> HtmlReferrerPolicy {
    HtmlReferrerPolicy::StrictOriginWhenCrossOrigin
}

/// Meta's exact ASCII-insensitive vocabulary and four source legacy aliases.
/// No whitespace trimming or attribute applicability is inferred.
pub fn meta_referrer_policy_with<E>(
    raw: &DocumentString,
    mut admit: impl FnMut() -> Result<(), E>,
) -> Result<Option<HtmlReferrerPolicy>, E> {
    for (spelling, policy) in [
        ("never", HtmlReferrerPolicy::NoReferrer),
        ("default", default_document_referrer_policy()),
        ("always", HtmlReferrerPolicy::UnsafeUrl),
        (
            "origin-when-crossorigin",
            HtmlReferrerPolicy::OriginWhenCrossOrigin,
        ),
    ] {
        let keyword = Keyword {
            spelling,
            canonical: spelling,
            state: policy,
            conforming: false,
        };
        if enum_keyword_matches_with(raw, &keyword, false, &mut |_| admit())? {
            return Ok(Some(policy));
        }
    }
    let selected = enum_value_selection_with(&REFERRERPOLICY, Some(raw), |_| admit())?;
    Ok(selected
        .intrinsic
        .filter(|policy| *policy != HtmlReferrerPolicy::Empty))
}

fn whitespace(unit: u16) -> bool {
    matches!(unit, 9 | 10 | 12 | 13 | 32)
}

/// Exact first nonempty token, unless any comma occurs anywhere in the input.
/// Output borrows an original UTF-16 range; the caller owns materialization.
pub fn meta_language_token_with<E>(
    raw: &DocumentString,
    mut admit: impl FnMut() -> Result<(), E>,
) -> Result<Option<Range<usize>>, E> {
    let units = raw.code_units();
    for unit in units {
        admit()?;
        if *unit == 44 {
            return Ok(None);
        }
    }
    let mut pos = 0;
    loop {
        admit()?;
        if !units.get(pos).copied().is_some_and(whitespace) {
            break;
        }
        pos += 1;
    }
    let start = pos;
    loop {
        admit()?;
        if !units
            .get(pos)
            .copied()
            .is_some_and(|unit| !whitespace(unit))
        {
            break;
        }
        pos += 1;
    }
    if start == pos {
        Ok(None)
    } else {
        admit()?;
        Ok(Some(start..pos))
    }
}

fn refresh_unit<E>(
    units: &[u16],
    pos: usize,
    admit: &mut impl FnMut(HtmlMetaRefreshAction) -> Result<(), E>,
) -> Result<Option<u16>, E> {
    admit(HtmlMetaRefreshAction::Step)?;
    Ok(units.get(pos).copied())
}
fn refresh_space<E>(
    units: &[u16],
    pos: &mut usize,
    admit: &mut impl FnMut(HtmlMetaRefreshAction) -> Result<(), E>,
) -> Result<(), E> {
    while refresh_unit(units, *pos, admit)?.is_some_and(whitespace) {
        *pos += 1;
    }
    Ok(())
}

/// Shared declarative-refresh framing, without materializing or truncating time.
/// `None` means lexical rejection; a URL-string target is not parsed/qualified.
pub fn meta_refresh_syntax_with<E>(
    raw: &DocumentString,
    mut admit: impl FnMut(HtmlMetaRefreshAction) -> Result<(), E>,
) -> Result<Option<HtmlMetaRefreshSyntaxTarget>, E> {
    let units = raw.code_units();
    let mut pos = 0;
    refresh_space(units, &mut pos, &mut admit)?;
    let start = pos;
    while refresh_unit(units, pos, &mut admit)?.is_some_and(|u| (48..=57).contains(&u)) {
        pos += 1;
    }
    admit(HtmlMetaRefreshAction::Step)?;
    if start == pos && refresh_unit(units, pos, &mut admit)? != Some(46) {
        return Ok(None);
    }
    admit(HtmlMetaRefreshAction::DelayDigits(start..pos))?;
    while refresh_unit(units, pos, &mut admit)?.is_some_and(|u| (48..=57).contains(&u) || u == 46) {
        pos += 1;
    }
    admit(HtmlMetaRefreshAction::Step)?;
    if pos != units.len() {
        if !refresh_unit(units, pos, &mut admit)?
            .is_some_and(|u| u == 59 || u == 44 || whitespace(u))
        {
            return Ok(None);
        }
        refresh_space(units, &mut pos, &mut admit)?;
        if refresh_unit(units, pos, &mut admit)?.is_some_and(|u| u == 59 || u == 44) {
            pos += 1;
        }
        refresh_space(units, &mut pos, &mut admit)?;
        admit(HtmlMetaRefreshAction::Step)?;
    }
    let target = if pos == units.len() {
        HtmlMetaRefreshSyntaxTarget::DocumentUrl
    } else {
        let original = pos;
        let mut skip_quotes = true;
        if refresh_unit(units, pos, &mut admit)?.is_some_and(|u| u == 85 || u == 117) {
            pos += 1;
            if refresh_unit(units, pos, &mut admit)?.is_some_and(|u| u == 82 || u == 114) {
                pos += 1;
                if refresh_unit(units, pos, &mut admit)?.is_some_and(|u| u == 76 || u == 108) {
                    pos += 1;
                    refresh_space(units, &mut pos, &mut admit)?;
                    if refresh_unit(units, pos, &mut admit)? == Some(61) {
                        pos += 1;
                        refresh_space(units, &mut pos, &mut admit)?;
                    } else {
                        skip_quotes = false;
                    }
                } else {
                    skip_quotes = false;
                }
            } else {
                skip_quotes = false;
            }
        }
        let range = if skip_quotes {
            let quote = refresh_unit(units, pos, &mut admit)?.filter(|u| *u == 39 || *u == 34);
            if quote.is_some() {
                pos += 1;
            }
            let start = pos;
            let mut end = units.len();
            if let Some(quote) = quote {
                while let Some(unit) = refresh_unit(units, pos, &mut admit)? {
                    if unit == quote {
                        end = pos;
                        break;
                    }
                    pos += 1;
                }
            }
            start..end
        } else {
            original..units.len()
        };
        admit(HtmlMetaRefreshAction::Step)?;
        HtmlMetaRefreshSyntaxTarget::UrlString(range)
    };
    admit(HtmlMetaRefreshAction::Step)?;
    Ok(Some(target))
}
