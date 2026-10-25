//! CSS strictness adaptation around the pinned ICU line-break provider.
//!
//! Policies share complete semantic components, never truncated style runs.
//! Anywhere's separate text/TCU opportunity owner lives in protocol::prepared.

use std::ops::Range;

use icu_properties::{
    CodePointMapData,
    props::{EastAsianWidth, GeneralCategory, LineBreak as Class},
};
use icu_segmenter::{
    LineSegmenter,
    options::{LineBreakOptions, LineBreakStrictness, LineBreakWordOption},
};

use super::{Boundary, is_hard_separator, style_at};
use crate::{
    LineBreak, engine::resolve::AnalyzedStyleRun, projection::SourceProjection,
    style::OrdinaryWordBreak,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct Policy {
    strictness: LineBreak,
    word: OrdinaryWordBreak,
    cjk: bool,
}

impl Policy {
    const BASELINE: Self = Self {
        strictness: LineBreak::Strict,
        word: OrdinaryWordBreak::Normal,
        cjk: false,
    };

    fn normal_word(self) -> Self {
        Self {
            word: OrdinaryWordBreak::Normal,
            ..self
        }
    }
}

pub(super) fn policies(styles: &[AnalyzedStyleRun]) -> crate::Result<Vec<Policy>> {
    styles
        .iter()
        .map(|row| {
            let style = row.style.authored();
            let strictness = match style.line_break {
                LineBreak::Auto | LineBreak::Anywhere => LineBreak::Strict,
                other => other,
            };
            let cjk = if let Some(locale) = row.style.font_locale() {
                // Parse only the already admitted canonical language/script/region
                // prefix. Extensions remain untouched in the authored Style.
                let mut id: icu_locale::LanguageIdentifier =
                    locale.as_str().parse().map_err(|_| {
                        crate::Error::new(
                            crate::ErrorCode::InvalidStyle,
                            "canonical content locale cannot be represented for line breaking",
                        )
                    })?;
                if id.script.is_none() && id.language.as_str() != "und" {
                    icu_locale::LocaleExpander::new_extended().maximize(&mut id);
                }
                id.script.is_some_and(|script| {
                    matches!(
                        script.as_str(),
                        "Hant"
                            | "Hans"
                            | "Hani"
                            | "Hanb"
                            | "Bopo"
                            | "Jpan"
                            | "Hrkt"
                            | "Hira"
                            | "Kana"
                    )
                })
            } else {
                false
            };
            Ok(Policy {
                strictness,
                word: style.effective_breaking().word,
                cjk: cjk && strictness != LineBreak::Strict,
            })
        })
        .collect()
}

fn provider(text: &str, policy: Policy) -> Vec<bool> {
    let mut options = LineBreakOptions::default();
    options.strictness = Some(match policy.strictness {
        LineBreak::Loose => LineBreakStrictness::Loose,
        LineBreak::Normal => LineBreakStrictness::Normal,
        _ => LineBreakStrictness::Strict,
    });
    options.word_option = Some(match policy.word {
        OrdinaryWordBreak::Normal => LineBreakWordOption::Normal,
        OrdinaryWordBreak::BreakAll => LineBreakWordOption::BreakAll,
        OrdinaryWordBreak::KeepAll => LineBreakWordOption::KeepAll,
    });
    // ICU's only content-locale distinction is ja/zh. This identifier conveys
    // the proven Appendix-F classification, never an invented authored locale.
    let cjk = icu_locale::langid!("ja");
    options.content_locale = policy.cjk.then_some(&cjk);
    let mut cuts = vec![false; text.len() + 1];
    for byte in LineSegmenter::new_for_non_complex_scripts(options).segment_str(text) {
        cuts[byte] = true;
    }
    cuts
}

pub(super) fn apply(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    policies: &[Policy],
    offsets: &[usize],
    component: Range<usize>,
    boundaries: &mut [Boundary],
) {
    let mut needed = Vec::new();
    for atom in component.clone() {
        let policy = policies[style_at(projection, styles, atom)];
        if !needed.contains(&policy) {
            needed.push(policy);
        }
    }
    // At most 3 strictnesses * 3 word modes * 2 CJK classifications.
    let start = offsets[component.start];
    let text = &projection.segmentation[start..offsets[component.end]];
    let baseline = provider(text, Policy::BASELINE);
    let classes = CodePointMapData::<Class>::new();
    let mut effective = Vec::with_capacity(component.len());
    let mut owners = Vec::with_capacity(component.len());
    let mut opening = Vec::with_capacity(component.len());
    let mut blocks_opening = false;
    for atom in component.clone() {
        let class = classes.get32(projection.atoms[atom].code_point);
        let inherited = matches!(class, Class::CombiningMark | Class::ZWJ)
            .then(|| {
                effective.last().copied().filter(|previous| {
                    !matches!(
                        *previous,
                        Class::MandatoryBreak
                            | Class::CarriageReturn
                            | Class::LineFeed
                            | Class::NextLine
                            | Class::Space
                            | Class::ZWSpace
                    )
                })
            })
            .flatten();
        let local = atom - component.start;
        let owner = if inherited.is_some() {
            owners[local - 1]
        } else {
            local
        };
        let class = inherited.unwrap_or({
            if matches!(class, Class::CombiningMark | Class::ZWJ) {
                Class::Alphabetic
            } else {
                class
            }
        });
        owners.push(owner);
        effective.push(class);
        if class != Class::Space {
            let cp = projection.atoms[component.start + owner].code_point;
            let prior = owner.checked_sub(1).map(|i| effective[i]);
            blocks_opening = class == Class::OpenPunctuation
                || class == Class::Quotation
                    && CodePointMapData::<GeneralCategory>::new().get32(cp)
                        == GeneralCategory::InitialPunctuation
                    && prior.is_none_or(|value| {
                        matches!(
                            value,
                            Class::MandatoryBreak
                                | Class::CarriageReturn
                                | Class::LineFeed
                                | Class::NextLine
                                | Class::OpenPunctuation
                                | Class::Quotation
                                | Class::Glue
                                | Class::Space
                                | Class::ZWSpace
                        )
                    });
        }
        opening.push(blocks_opening);
    }
    // Provider local0/eot are sentinels, never text/text permission. Actual
    // terminal ZW/SP causes retain their independently qualified permissions.
    if independent_ending(projection, styles, &component, &effective, &opening) {
        boundaries[component.end] = Boundary::Line;
    }
    let mut groups = Vec::new();
    for policy in &needed {
        if !groups.contains(&policy.normal_word()) {
            groups.push(policy.normal_word());
        }
    }
    // Retain only baseline, one strictness/locale Normal-word pass, and one
    // actual word-mode pass at a time. Each finite provider tuple runs once.
    for group in groups {
        let normal_storage = (group != Policy::BASELINE).then(|| provider(text, group));
        let normal = normal_storage.as_ref().unwrap_or(&baseline);
        for policy in needed
            .iter()
            .copied()
            .filter(|policy| policy.normal_word() == group)
        {
            let actual_storage =
                (policy.word != OrdinaryWordBreak::Normal).then(|| provider(text, policy));
            let actual = actual_storage.as_ref().unwrap_or(normal);
            for atom in component.start + 1..component.end {
                if policies[style_at(projection, styles, atom)] != policy {
                    continue;
                }
                let byte = offsets[atom] - start;
                let independent = baseline[byte];
                let before = effective[atom - component.start - 1];
                let after = classes.get32(projection.atoms[atom].code_point);
                let required = required_pair(
                    projection.atoms[atom - 1].code_point,
                    projection.atoms[atom].code_point,
                    before,
                    after,
                    policy,
                );
                let allowed = match required {
                    // Strict Normal-word output preserves independently caused SPACE
                    // and ZWSP permissions without retaining the provider's extra CJ.
                    Some(false) => independent,
                    Some(true) => actual[byte] || normal[byte],
                    None => actual[byte],
                };
                if !allowed {
                    continue;
                }
                if !independent {
                    let forbidden =
                        |class| matches!(class, Class::Glue | Class::WordJoiner | Class::ZWJ);
                    if forbidden(before)
                        || forbidden(after)
                        || classes.get32(projection.atoms[atom - 1].code_point) == Class::ZWJ
                        || is_hard_separator(projection.atoms[atom - 1].code_point)
                        || is_hard_separator(projection.atoms[atom].code_point)
                    {
                        continue;
                    }
                    // Preserve UAX55 LB14/LB15a and quote rules bypassed by ICU's
                    // early CSS return. Opening state is precomputed, not rescanned.
                    if opening[atom - component.start - 1]
                        || quote_blocks(projection, &component, &effective, &owners, atom)
                    {
                        continue;
                    }
                }
                boundaries[atom] = Boundary::Line;
            }
        }
    }
    for (atom, boundary) in boundaries
        .iter_mut()
        .enumerate()
        .take(component.end)
        .skip(component.start + 1)
    {
        if [atom - 1, atom].into_iter().any(|at| {
            styles[style_at(projection, styles, at)]
                .style
                .authored()
                .line_break
                == LineBreak::Anywhere
        }) && !is_hard_separator(projection.atoms[atom - 1].code_point)
            && !is_hard_separator(projection.atoms[atom].code_point)
            && projection
                .graphemes
                .binary_search(&projection.atoms[atom].presentation.start)
                .is_ok()
            && *boundary < Boundary::Line
        {
            *boundary = Boundary::Anywhere;
        }
    }
}

fn independent_ending(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    component: &Range<usize>,
    effective: &[Class],
    opening: &[bool],
) -> bool {
    let end = component.end;
    let Some(after) = projection.atoms.get(end) else {
        return false;
    };
    let classes = CodePointMapData::<Class>::new();
    let right = classes.get32(after.code_point);
    // UAX55 LB6/LB7 precede LB8. Inspect only the actual retained suffix of
    // this component; another component can never supply its ZW authority.
    if is_hard_separator(after.code_point) || matches!(right, Class::Space | Class::ZWSpace) {
        return false;
    }
    let prior = effective
        .iter()
        .rev()
        .find(|class| **class != Class::Space)
        .copied();
    // LB8's ZW SP* permission precedes the WJ/glue/punctuation prohibitions
    // below. Prepared TCU, mandatory and wrapping guards still own admission.
    if prior == Some(Class::ZWSpace) {
        return true;
    }
    let before = &projection.atoms[end - 1];
    let style = styles[style_at(projection, styles, end - 1)]
        .style
        .authored();
    if !matches!(before.code_point, 0x20 | 0x09)
        || !matches!(
            style.white_space,
            crate::WhiteSpace::Preserve | crate::WhiteSpace::PreserveSpaces
        )
        || matches!(after.code_point, 0x20 | 0x09)
        || opening.last() == Some(&true)
    {
        return false;
    }
    // Incident glue/hard rules and pre-SPACE opening context remain relevant
    // to this whitespace owner; no Unicode text pair crosses the component.
    if matches!(
        right,
        Class::WordJoiner
            | Class::ZWJ
            | Class::ZWSpace
            | Class::ClosePunctuation
            | Class::CloseParenthesis
            | Class::Exclamation
            | Class::BreakSymbols
    ) {
        return false;
    }
    if matches!(
        prior,
        Some(Class::ClosePunctuation | Class::CloseParenthesis)
    ) && matches!(right, Class::Nonstarter | Class::ConditionalJapaneseStarter)
        || prior == Some(Class::BreakBoth) && right == Class::BreakBoth
    {
        return false;
    }
    let next = projection
        .atoms
        .get(end + 1)
        .map(|atom| classes.get32(atom.code_point));
    if right == Class::InfixNumeric && next != Some(Class::Numeric) {
        return false;
    }
    if right == Class::Quotation
        && CodePointMapData::<GeneralCategory>::new().get32(after.code_point)
            == GeneralCategory::FinalPunctuation
        && next.is_none_or(|class| {
            matches!(
                class,
                Class::Space
                    | Class::Glue
                    | Class::WordJoiner
                    | Class::ClosePunctuation
                    | Class::Quotation
                    | Class::CloseParenthesis
                    | Class::Exclamation
                    | Class::InfixNumeric
                    | Class::BreakSymbols
                    | Class::MandatoryBreak
                    | Class::CarriageReturn
                    | Class::LineFeed
                    | Class::NextLine
                    | Class::ZWSpace
            )
        })
    {
        return false;
    }
    true
}

fn quote_blocks(
    projection: &SourceProjection,
    component: &Range<usize>,
    effective: &[Class],
    owners: &[usize],
    atom: usize,
) -> bool {
    let local = atom - component.start;
    let before = owners[local - 1];
    let cp = |i: usize| projection.atoms[component.start + i].code_point;
    let east_asian = |i| {
        matches!(
            CodePointMapData::<EastAsianWidth>::new().get32(cp(i)),
            EastAsianWidth::Fullwidth | EastAsianWidth::Wide | EastAsianWidth::Halfwidth
        )
    };
    let category = |i| CodePointMapData::<GeneralCategory>::new().get32(cp(i));
    if effective[local - 1] == Class::Quotation
        && (category(before) != GeneralCategory::FinalPunctuation
            || before.checked_sub(1).is_none_or(|i| !east_asian(i))
            || !east_asian(local))
    {
        return true;
    }
    if effective[local] != Class::Quotation {
        return false;
    }
    let mut next = local + 1;
    while next < owners.len() && owners[next] == local {
        next += 1;
    }
    // LB15b, LB19 and LB19a. These are contextual quote restrictions, not a
    // blanket veto of all quotation marks or all East Asian punctuation.
    category(local) != GeneralCategory::InitialPunctuation
        || !east_asian(local - 1)
        || next == owners.len()
        || !east_asian(next)
        || category(local) == GeneralCategory::FinalPunctuation
            && matches!(
                effective[next],
                Class::Space
                    | Class::Glue
                    | Class::WordJoiner
                    | Class::ClosePunctuation
                    | Class::Quotation
                    | Class::CloseParenthesis
                    | Class::Exclamation
                    | Class::InfixNumeric
                    | Class::BreakSymbols
                    | Class::MandatoryBreak
                    | Class::CarriageReturn
                    | Class::LineFeed
                    | Class::NextLine
                    | Class::ZWSpace
            )
}

/// The bounded mandatory CSS distinctions; all other UAX behavior stays ICU's.
fn required_pair(
    before_cp: u32,
    after_cp: u32,
    before: Class,
    after: Class,
    policy: Policy,
) -> Option<bool> {
    let loose = policy.strictness == LineBreak::Loose;
    if matches!(after_cp, 0x301c | 0x30a0) {
        return Some(policy.strictness != LineBreak::Strict && policy.cjk);
    }
    if matches!(after_cp, 0x2010 | 0x2013) {
        let id = before == Class::Ideographic
            || before == Class::ConditionalJapaneseStarter && loose
            || policy.word == OrdinaryWordBreak::BreakAll
                && matches!(
                    before,
                    Class::Alphabetic | Class::Numeric | Class::ComplexContext | Class::Ambiguous
                );
        return Some(loose && id);
    }
    if matches!(
        after_cp,
        0x3005 | 0x303b | 0x309d | 0x309e | 0x30fd | 0x30fe
    ) || after == Class::Inseparable
    {
        return Some(loose);
    }
    // BreakAll converts processed CJ letter units to effective ID before this
    // class-based restriction. Named iteration/punctuation rules above retain
    // their independent precedence (see the adopted small-kana determination).
    if after == Class::ConditionalJapaneseStarter {
        return (policy.word != OrdinaryWordBreak::BreakAll).then_some(loose);
    }
    if matches!(
        after_cp,
        0x30fb | 0xff1a | 0xff1b | 0xff65 | 0x203c | 0x2047..=0x2049 | 0xff01 | 0xff1f
    ) {
        return Some(loose && policy.cjk);
    }
    let east_asian = |cp| {
        matches!(
            CodePointMapData::<EastAsianWidth>::new().get32(cp),
            EastAsianWidth::Ambiguous | EastAsianWidth::Fullwidth | EastAsianWidth::Wide
        )
    };
    if after == Class::PostfixNumeric && east_asian(after_cp)
        || before == Class::PrefixNumeric && east_asian(before_cp)
    {
        return Some(loose && policy.cjk);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_preserved_separator_retains_context_and_incident_prohibitions() {
        // CSS's maximal preserved separator cause retains UAX55 LB11/13/14/
        // 15a. The SPACE-before-GL exception remains distinct from WJ/ZWJ.
        for (text, seam, allowed) in [
            ("a b", 2, true),
            ("(  A", 3, false),
            ("“  A", 3, false),
            ("a \u{2060}b", 2, false),
            ("a \u{200d}b", 2, false),
            ("a \u{a0}b", 2, true),
            ("a )", 2, false),
            ("a  b", 2, false),
        ] {
            let source = crate::Source::new(text);
            let projection = SourceProjection::new(&source).unwrap();
            let styles = vec![AnalyzedStyleRun {
                owner: crate::engine::resolve::AnalyzedStyleOwner::Source(crate::Range::new(
                    0,
                    source.text().len(),
                )),
                style: crate::ValidatedStyle::try_from(crate::Style::default()).unwrap(),
            }];
            let offsets: Vec<_> = projection
                .segmentation
                .char_indices()
                .map(|(byte, _)| byte)
                .chain(std::iter::once(projection.segmentation.len()))
                .collect();
            let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
            apply(
                &projection,
                &styles,
                &policies(&styles).unwrap(),
                &offsets,
                0..seam,
                &mut boundaries,
            );
            assert_eq!(boundaries[seam] == Boundary::Line, allowed, "{text:?}");
        }
    }
}
