//! CSS-owned checked syntax enters one live owner without text reconstruction.
use crate::parser_resources::check_resource_diagnostics;
#[cfg(test)]
use crate::state_limits::probe;
use crate::state_limits::{Census, Extension, Inserted};
use crate::{model::State, *};
use std::sync::atomic::Ordering;
use surgeist_css::component::CssComponentValueLimits;
use surgeist_css::declaration::{
    CssDeclaration, CssDeclarationBlockError, CssDeclarationList, CssSpecifiedDeclarationBlock,
};
use surgeist_css::diagnostic::CssRecoveryDiagnostic;
use surgeist_css::query::media::{CssCustomMediaBody, CssMediaQueryList};
use surgeist_css::rule::CssSheet;
use surgeist_css::rule::counter_style::CssCounterStyleRule;
use surgeist_css::rule::font_face::CssFontFaceRule;
use surgeist_css::rule::font_face::{CssFontFaceDescriptor, CssSpecifiedFontFaceDeclarationBlock};
use surgeist_css::rule::font_feature_values::CssFontFeatureValuesRule;
use surgeist_css::rule::font_feature_values::{
    CssFontFeatureValueIndex, CssFontFeatureValueKind, CssFontFeatureValueRef,
    CssFontFeatureValuesItem,
};
use surgeist_css::rule::keyframes::CssKeyframesRule;
use surgeist_css::rule::page::CssPageRule;
use surgeist_css::rule::{CssCustomMediaRule, CssRule};
use surgeist_css::{CssParser, CssParserContext};

impl State {
    pub(crate) fn allocate(&mut self, limits: &CssomLimits) -> Result<u64, CssomError> {
        let value = self
            .identity_issuer
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                if next == 0 || next > limits.max_identity {
                    None
                } else {
                    Some(next.checked_add(1).unwrap_or(0))
                }
            })
            .map_err(|_| CssomError::IdentityExhausted)?;
        self.last_allocation = value;
        Ok(value)
    }
    pub(crate) fn list(&mut self, limits: &CssomLimits) -> Result<CssomRuleListId, CssomError> {
        let id = CssomRuleListId {
            owner: self.owner.clone(),
            serial: self.allocate(limits)?,
        };
        self.lists.insert(id.clone(), Vec::new());
        Ok(id)
    }
    pub(crate) fn media_list(
        &mut self,
        value: CssMediaQueryList,
        limits: &CssomLimits,
    ) -> Result<CssomMediaListId, CssomError> {
        let id = CssomMediaListId {
            owner: self.owner.clone(),
            serial: self.allocate(limits)?,
        };
        self.media.insert(id.clone(), value);
        Ok(id)
    }
    pub(crate) fn block_record(
        &mut self,
        data: CssomBlockData,
        parent: Option<CssomRuleId>,
        owner: Option<CssomInputVersion>,
        flags: CssomBlockFlags,
        limits: &CssomLimits,
    ) -> Result<CssomBlockId, CssomError> {
        let id = CssomBlockId {
            owner: self.owner.clone(),
            serial: self.allocate(limits)?,
        };
        let selected_font_face = if let CssomBlockData::FontFace(authored) = &data {
            let entries = font_winners(authored.occurrences().cloned().collect());
            Some(
                match CssSpecifiedFontFaceDeclarationBlock::try_from_entries(&entries, limits.css) {
                    Ok(selected) => CssomProjection::Available(selected),
                    Err(error) if error.is_resource_limit() => {
                        return Err(CssomError::FontFace(error));
                    }
                    Err(error) => CssomProjection::Unavailable(error),
                },
            )
        } else {
            None
        };
        self.blocks.insert(
            id.clone(),
            CssomBlock {
                parent,
                binding: crate::inline_context::BlockBinding::Store(owner),
                flags,
                data: data.into(),
                admission_inputs: std::sync::Arc::new(Vec::new()),
                counter_edits: std::sync::Arc::new(Vec::new()),
                replacement_input: None,
                selected_font_face: selected_font_face.map(std::sync::Arc::new),
            },
        );
        Ok(id)
    }
    /// Adopts rules into this state under `limits`; see [`Adoption`].
    pub(crate) fn adoption<'a>(&'a mut self, limits: &'a CssomLimits) -> Adoption<'a> {
        Adoption {
            state: self,
            limits,
            census: None,
            inserted: Inserted::default(),
        }
    }
    /// Adopts one rule, with the rules it contains, into this state.
    pub(crate) fn adopt(
        &mut self,
        authored: CssomAuthoredRule,
        parent: CssomParent,
        depth: usize,
        limits: &CssomLimits,
    ) -> Result<CssomRuleId, CssomError> {
        self.adoption(limits).adopt(authored, parent, depth)
    }
}

/// A run of rule adoptions into one staged state.
///
/// After each adopted rule the state is checked against the limits. The first
/// check is the census of the whole state; it is kept as `census`. Each later
/// check extends `census` by what the adoption inserted since the previous
/// check, recorded in `inserted`, instead of walking the whole state again.
///
/// The census is exact because this value owns the only access to the state
/// for its whole lifetime: the state is borrowed mutably and reachable only
/// for reading, and every change adoption makes (a rule, block, media list or
/// feature map inserted, a member appended to a rule list) goes through a
/// method that records it. No other change to the state can happen between
/// two checks, so `census` plus `inserted` always equals the whole state. A
/// refusal drops `census`, so any later check starts from the whole state
/// again. The value is not part of the state, is not cloned with it and is
/// not reachable from a snapshot; it ends with the adoption.
pub(crate) struct Adoption<'a> {
    state: &'a mut State,
    limits: &'a CssomLimits,
    census: Option<Census>,
    inserted: Inserted,
}

impl std::ops::Deref for Adoption<'_> {
    type Target = State;
    fn deref(&self) -> &State {
        self.state
    }
}

impl<'a> Adoption<'a> {
    pub(crate) fn limits(&self) -> &'a CssomLimits {
        self.limits
    }
    fn allocate(&mut self) -> Result<u64, CssomError> {
        self.state.allocate(self.limits)
    }
    /// A new empty rule list; it has no entries to account for until a member is appended.
    pub(crate) fn list(&mut self) -> Result<CssomRuleListId, CssomError> {
        self.state.list(self.limits)
    }
    fn media_list(&mut self, value: CssMediaQueryList) -> Result<CssomMediaListId, CssomError> {
        let id = self.state.media_list(value, self.limits)?;
        self.inserted.media(id.clone());
        Ok(id)
    }
    fn block_record(
        &mut self,
        data: CssomBlockData,
        parent: Option<CssomRuleId>,
        owner: Option<CssomInputVersion>,
        flags: CssomBlockFlags,
    ) -> Result<CssomBlockId, CssomError> {
        let id = self
            .state
            .block_record(data, parent, owner, flags, self.limits)?;
        self.inserted.block(id.clone());
        Ok(id)
    }
    fn insert_rule(&mut self, id: CssomRuleId, rule: CssomRule) {
        self.state.rules.insert(id.clone(), rule);
        self.inserted.rule(id);
    }
    fn insert_map(&mut self, id: CssomFeatureMapId, map: CssomFeatureMap) {
        self.state.maps.insert(id.clone(), map);
        self.inserted.map(id);
    }
    /// Appends an adopted rule to the list of its parent.
    pub(crate) fn push_child(&mut self, list: &CssomRuleListId, child: CssomRuleId) {
        self.state
            .lists
            .get_mut(list)
            .expect("new list")
            .push(child);
        self.inserted.member();
    }
    /// The census of the whole state as the adoption has left it.
    fn whole_census(&self) -> Result<Census, CssomError> {
        #[cfg(test)]
        probe::whole_run();
        Census::of(self.state, self.limits, std::iter::empty())
    }
    /// Refuses the state as the adoption has left it when it exceeds the limits.
    fn check(&mut self) -> Result<(), CssomError> {
        #[cfg(test)]
        if probe::whole_state() {
            return self.state.check_limits(self.limits);
        }
        let inserted = std::mem::take(&mut self.inserted);
        let census = match self.census.take() {
            Some(mut census) => {
                let extended = census.extend(self.state, &inserted, self.limits);
                #[cfg(test)]
                if probe::cross_check() {
                    census.assert_matches_whole_state(&extended, self.state, self.limits);
                }
                match extended? {
                    Extension::Applied => census,
                    Extension::WholeStateNeeded => self.whole_census()?,
                }
            }
            None => self.whole_census()?,
        };
        self.census = Some(census);
        Ok(())
    }
}

impl Adoption<'_> {
    fn ordinary_block(
        &mut self,
        declarations: &CssDeclarationList,
        parent: &CssomRuleId,
    ) -> Result<CssomBlockId, CssomError> {
        check_declarations(declarations.iter(), self.limits)?;
        let selected = projection(CssSpecifiedDeclarationBlock::try_from_declarations(
            declarations,
            self.limits.css,
        ))?;
        self.block_record(
            CssomBlockData::Properties {
                domain: CssomPropertyDomain::Ordinary,
                authored: CssomPropertyOccurrences::Ordinary(declarations.clone()),
                selected,
            },
            Some(parent.clone()),
            None,
            CssomBlockFlags::default(),
        )
    }
    fn rule_children(
        &mut self,
        rules: &[CssRule],
        parent: &CssomRuleId,
        depth: usize,
    ) -> Result<CssomRuleListId, CssomError> {
        let list = self.list()?;
        for rule in rules {
            let id = self.adopt(
                CssomAuthoredRule::Rule(rule.clone()),
                CssomParent::Rule(parent.clone()),
                depth + 1,
            )?;
            self.push_child(&list, id);
        }
        Ok(list)
    }
    pub(crate) fn adopt(
        &mut self,
        authored: CssomAuthoredRule,
        parent: CssomParent,
        depth: usize,
    ) -> Result<CssomRuleId, CssomError> {
        if depth > self.limits.max_depth {
            return Err(CssomError::Limit {
                resource: "rule depth",
                maximum: self.limits.max_depth,
            });
        }
        let id = CssomRuleId {
            owner: self.owner.clone(),
            serial: self.allocate()?,
        };
        let data = match &authored {
            CssomAuthoredRule::Rule(v) => self.rule_data(v, &id, depth)?,
            CssomAuthoredRule::Keyframe(v) => {
                check_declarations(v.declarations().iter().map(|v| v.source()), self.limits)?;
                let selected = projection(
                    CssSpecifiedDeclarationBlock::try_from_keyframe_declarations(
                        v.declarations(),
                        self.limits.css,
                    ),
                )?;
                let block = self.block_record(
                    CssomBlockData::Properties {
                        domain: CssomPropertyDomain::Keyframe,
                        authored: CssomPropertyOccurrences::Keyframe(v.declarations().clone()),
                        selected,
                    },
                    Some(id.clone()),
                    None,
                    CssomBlockFlags::default(),
                )?;
                CssomRuleData::Keyframe {
                    selectors: v.selectors().clone(),
                    block,
                }
            }
            CssomAuthoredRule::RawNestedDeclarations(v) => {
                let block = self.ordinary_block(v, &id)?;
                CssomRuleData::NestedDeclarations { block }
            }
            CssomAuthoredRule::SupportsTestChild(v) => CssomRuleData::SupportsTestChild(v.clone()),
            CssomAuthoredRule::Margin(v) => {
                check_declarations(v.declarations().properties().iter(), self.limits)?;
                let selected =
                    projection(v.declarations().try_specified_properties(self.limits.css))?;
                let block = self.block_record(
                    CssomBlockData::Properties {
                        domain: CssomPropertyDomain::Margin,
                        authored: CssomPropertyOccurrences::Margin(v.declarations().clone()),
                        selected,
                    },
                    Some(id.clone()),
                    None,
                    CssomBlockFlags::default(),
                )?;
                CssomRuleData::Margin {
                    name: v.name(),
                    block,
                }
            }
        };
        self.insert_rule(
            id.clone(),
            CssomRule {
                parent,
                data: data.into(),
                authored: authored.into(),
                insertion_origin: None,
                selector_input: None,
                selector_inputs: None,
                selector_namespaces: None,
                admission_diagnostics: std::sync::Arc::new(Vec::new()),
            },
        );
        self.check()?;
        Ok(id)
    }
    fn page_data(
        &mut self,
        v: &CssPageRule,
        id: &CssomRuleId,
        depth: usize,
    ) -> Result<CssomRuleData, CssomError> {
        check_declarations(v.declarations().properties().iter(), self.limits)?;
        let selected = match v.declarations().try_specified(self.limits.css) {
            Ok(v) => CssomProjection::Available(v),
            Err(e) if e.is_resource_limit() => return Err(CssomError::Page(e)),
            Err(e) => CssomProjection::Unavailable(e),
        };
        let block = self.block_record(
            CssomBlockData::Page {
                authored: v.declarations().clone(),
                selected,
            },
            Some(id.clone()),
            None,
            CssomBlockFlags::default(),
        )?;
        let children = self.list()?;
        for child in v.margin_rules() {
            let child = self.adopt(
                CssomAuthoredRule::Margin(child.clone()),
                CssomParent::Rule(id.clone()),
                depth + 1,
            )?;
            self.push_child(&children, child);
        }
        Ok(CssomRuleData::Page {
            selectors: v.selectors().clone(),
            block,
            children,
        })
    }
    fn keyframes_data(
        &mut self,
        v: &CssKeyframesRule,
        id: &CssomRuleId,
        depth: usize,
    ) -> Result<CssomRuleData, CssomError> {
        let children = self.list()?;
        for child in v.blocks() {
            let child = self.adopt(
                CssomAuthoredRule::Keyframe(child.clone()),
                CssomParent::Rule(id.clone()),
                depth + 1,
            )?;
            self.push_child(&children, child);
        }
        Ok(CssomRuleData::Keyframes {
            name: v.name().clone(),
            children,
        })
    }
    fn counter_data(
        &mut self,
        v: &CssCounterStyleRule,
        id: &CssomRuleId,
    ) -> Result<CssomRuleData, CssomError> {
        let block = self.block_record(
            CssomBlockData::CounterStyle(Box::new(v.descriptors().clone())),
            Some(id.clone()),
            None,
            CssomBlockFlags::default(),
        )?;
        Ok(CssomRuleData::CounterStyle {
            name: v.name().as_str().to_owned(),
            block,
        })
    }
    fn font_data(
        &mut self,
        v: &CssFontFaceRule,
        id: &CssomRuleId,
    ) -> Result<CssomRuleData, CssomError> {
        let block = self.block_record(
            CssomBlockData::FontFace(v.descriptors().clone()),
            Some(id.clone()),
            None,
            CssomBlockFlags::default(),
        )?;
        Ok(CssomRuleData::FontFace { block })
    }
    fn features_data(&mut self, v: &CssFontFeatureValuesRule) -> Result<CssomRuleData, CssomError> {
        let mut maps = Vec::new();
        for kind in [
            CssFontFeatureValueKind::Stylistic,
            CssFontFeatureValueKind::HistoricalForms,
            CssFontFeatureValueKind::Styleset,
            CssFontFeatureValueKind::CharacterVariant,
            CssFontFeatureValueKind::Swash,
            CssFontFeatureValueKind::Ornaments,
            CssFontFeatureValueKind::Annotation,
        ] {
            let id = CssomFeatureMapId {
                owner: self.owner.clone(),
                serial: self.allocate()?,
            };
            let mut map = CssomFeatureMap {
                kind,
                entries: Vec::new(),
                authored: std::sync::Arc::new(Vec::new()),
            };
            for item in v.items() {
                if let CssFontFeatureValuesItem::Block(block) = item {
                    if block.kind() != kind {
                        continue;
                    }
                    for definition in block.definitions() {
                        std::sync::Arc::make_mut(&mut map.authored).push(definition.clone());
                        let value = match definition.value().view() {
                            CssFontFeatureValueRef::Indexes(indexes) => indexes
                                .iter()
                                .map(CssFontFeatureValueIndex::to_u32)
                                .collect::<Option<Vec<_>>>()
                                .map_or_else(
                                    || CssomFeatureValue::Authored(definition.value().clone()),
                                    CssomFeatureValue::Unsigned,
                                ),
                            _ => CssomFeatureValue::Authored(definition.value().clone()),
                        };
                        let entry = CssomFeatureEntry {
                            key: definition.name().as_str().to_owned(),
                            value,
                        };
                        if let Some(old) = map.entries.iter_mut().find(|v| v.key == entry.key) {
                            *old = entry;
                        } else {
                            map.entries.push(entry);
                        }
                    }
                }
            }
            self.insert_map(id.clone(), map);
            maps.push(id);
        }
        Ok(CssomRuleData::FontFeatureValues {
            families: v.families().to_vec(),
            family_input: None,
            maps,
            font_display: v
                .items()
                .iter()
                .filter_map(|v| match v {
                    CssFontFeatureValuesItem::FontDisplay(v) => Some(v.clone()),
                    _ => None,
                })
                .collect(),
        })
    }
    fn custom_media_data(
        &mut self,
        authored: &CssRule,
        rule: &CssCustomMediaRule,
    ) -> Result<CssomRuleData, CssomError> {
        let query = match rule.body() {
            CssCustomMediaBody::True => CssomCustomMediaQuery::Boolean(true),
            CssCustomMediaBody::False => CssomCustomMediaQuery::Boolean(false),
            CssCustomMediaBody::Media(list) => {
                CssomCustomMediaQuery::MediaList(self.media_list(list.clone())?)
            }
            // A body this CSSOM does not know stays an opaque authored rule, as
            // any other rule it does not interpret does.
            _ => {
                return Ok(CssomRuleData::Leaf {
                    current: CssomAuthoredRule::Rule(authored.clone()),
                });
            }
        };
        Ok(CssomRuleData::CustomMedia {
            name: rule.name().clone(),
            query,
        })
    }
    fn rule_data(
        &mut self,
        v: &CssRule,
        id: &CssomRuleId,
        depth: usize,
    ) -> Result<CssomRuleData, CssomError> {
        Ok(match v {
            CssRule::SupportsCondition(v) => {
                self.named_supports_data(v.name(), v.body(), id, depth)?
            }
            CssRule::Style(v) => CssomRuleData::Style {
                selectors: v.selectors().clone(),
                block: self.ordinary_block(v.declarations(), id)?,
                children: self.rule_children(v.rules(), id, depth)?,
            },
            CssRule::NestedDeclarations(v) => CssomRuleData::NestedDeclarations {
                block: self.ordinary_block(v.declarations(), id)?,
            },
            CssRule::Media(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Media(self.media_list(v.query().clone())?),
                children: self.rule_children(v.rules(), id, depth)?,
            },
            CssRule::Supports(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Supports(v.condition().clone()),
                children: self.rule_children(v.rules(), id, depth)?,
            },
            CssRule::Container(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Container(v.prelude().clone()),
                children: self.rule_children(v.rules(), id, depth)?,
            },
            CssRule::LayerBlock(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Layer(v.name().cloned()),
                children: self.rule_children(v.rules(), id, depth)?,
            },
            CssRule::When(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::When(v.condition().clone()),
                children: self.rule_children(v.rules(), id, depth)?,
            },
            CssRule::Else(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Else(v.condition().cloned()),
                children: self.rule_children(v.rules(), id, depth)?,
            },
            CssRule::Scope(v) => CssomRuleData::Group {
                prelude: CssomGroupPrelude::Scope {
                    start: v.root().cloned(),
                    end: v.limit().cloned(),
                },
                children: self.rule_children(v.rules().rules(), id, depth)?,
            },
            CssRule::Page(v) => self.page_data(v, id, depth)?,
            CssRule::Keyframes(v) => self.keyframes_data(v, id, depth)?,
            CssRule::CounterStyle(v) => self.counter_data(v, id)?,
            CssRule::FontFace(v) => self.font_data(v, id)?,
            CssRule::FontFeatureValues(v) => self.features_data(v)?,
            CssRule::CustomMedia(rule) => self.custom_media_data(v, rule)?,
            CssRule::Import(v) => CssomRuleData::Import {
                target: v.target().clone(),
                layer: v.layer().cloned(),
                supports: v.supports().cloned(),
                origin: v.origin().clone(),
                media: self.media_list(
                    v.media()
                        .cloned()
                        .unwrap_or_else(|| CssMediaQueryList::new(Vec::new())),
                )?,
                child_sheet: None,
                resolved_location: None,
                input: None,
            },
            _ => CssomRuleData::Leaf {
                current: CssomAuthoredRule::Rule(v.clone()),
            },
        })
    }
}

pub(crate) fn projection(
    result: Result<CssSpecifiedDeclarationBlock, CssDeclarationBlockError>,
) -> Result<CssomProjection<CssSpecifiedDeclarationBlock, CssDeclarationBlockError>, CssomError> {
    match result {
        Ok(v) => Ok(CssomProjection::Available(v)),
        Err(e) if e.is_resource_limit() => Err(CssomError::Declaration(e)),
        Err(e) => Ok(CssomProjection::Unavailable(e)),
    }
}
/// Component limits that bound only the output bytes of `serialize_tokens`;
/// the structural limits keep their defaults.
pub(crate) fn token_byte_limits(max_bytes: usize) -> CssComponentValueLimits {
    let defaults = CssComponentValueLimits::default();
    CssComponentValueLimits::try_new(
        defaults.max_nesting_depth(),
        defaults.max_components(),
        max_bytes,
    )
    .expect("the default nesting depth is valid")
}
pub(crate) fn check_declarations<'a>(
    declarations: impl IntoIterator<Item = &'a CssDeclaration>,
    limits: &CssomLimits,
) -> Result<(), CssomError> {
    let mut nodes = 0usize;
    for declaration in declarations {
        let values = declaration.value_components();
        nodes = nodes
            .checked_add(values.component_count())
            .filter(|v| *v <= limits.max_entries)
            .ok_or(CssomError::Limit {
                resource: "authored component entries",
                maximum: limits.max_entries,
            })?;
        if declaration
            .parsed_value()
            .is_some_and(|v| v.source().as_str().len() > limits.max_input_bytes)
        {
            return Err(CssomError::Limit {
                resource: "retained parse input bytes",
                maximum: limits.max_input_bytes,
            });
        }
        // Charge raw component output even when symbolic expansion cannot select
        // a footprint. No semantic wrapper or parse round trip is involved.
        if let Err(e) = values.serialize_tokens(token_byte_limits(limits.max_input_bytes))
            && e.is_resource_limit()
        {
            return Err(CssomError::Component(e));
        }
    }
    Ok(())
}
pub(crate) fn parse(
    source: &str,
    limits: &CssomLimits,
    context: CssParserContext,
) -> Result<(CssSheet, Vec<CssRecoveryDiagnostic>), CssomError> {
    if source.len() > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    let report = CssParser::new(context, Default::default()).parse_sheet(source);
    check_resource_diagnostics(report.diagnostics())?;
    Ok(report.into_parts())
}

/// The existing FontFace inventory defines a last occurrence winner, including pending values.
/// Keep those actual winning occurrences in their authored order, without a kind-order table.
pub(crate) fn font_winners(entries: Vec<CssFontFaceDescriptor>) -> Vec<CssFontFaceDescriptor> {
    let mut seen = Vec::new();
    let mut winners = entries
        .into_iter()
        .rev()
        .filter(|entry| {
            let kind = entry.value().kind();
            if seen.contains(&kind) {
                false
            } else {
                seen.push(kind);
                true
            }
        })
        .collect::<Vec<_>>();
    winners.reverse();
    winners
}

#[cfg(test)]
mod adoption_tests {
    use super::*;

    fn rule(source: &str) -> CssomAuthoredRule {
        let (sheet, _) = CssParser::default().parse_sheet(source).into_parts();
        CssomAuthoredRule::Rule(sheet.rules()[0].clone())
    }

    #[test]
    fn a_refusal_drops_the_census_so_the_next_check_starts_from_the_whole_state() {
        let store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let mut state = (*store.state).clone();
        // The state, then a rule, its block and its children list fit; a second rule does not.
        let limits = CssomLimits {
            max_objects: 4,
            ..Default::default()
        };
        let mut adoption = state.adoption(&limits);
        adoption
            .adopt(rule("a { color: red }"), CssomParent::Detached, 1)
            .unwrap();
        assert!(adoption.census.is_some());
        let refusal = adoption
            .adopt(rule("b { color: red }"), CssomParent::Detached, 1)
            .unwrap_err();
        assert!(matches!(
            refusal,
            CssomError::Limit {
                resource: "objects",
                maximum: 4
            }
        ));
        assert!(adoption.census.is_none());
        assert!(adoption.inserted.is_empty());
        // Checking again measures the whole state, which still exceeds the limit.
        assert!(matches!(
            adoption.check(),
            Err(CssomError::Limit {
                resource: "objects",
                maximum: 4
            })
        ));
        assert!(adoption.census.is_none());
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn final_identity_is_issued_once_then_exhaustion_never_wraps() {
        let store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let mut state = (*store.state).clone();
        state.identity_issuer.store(u64::MAX, Ordering::Relaxed);
        let mut sibling = state.clone();
        assert_eq!(state.allocate(&store.limits).unwrap(), u64::MAX);
        assert!(matches!(
            sibling.allocate(&store.limits),
            Err(CssomError::IdentityExhausted)
        ));
        assert!(matches!(
            state.allocate(&store.limits),
            Err(CssomError::IdentityExhausted)
        ));
        assert_eq!(state.owner, sibling.owner);
        assert_eq!(state.last_allocation, u64::MAX);
    }

    #[test]
    fn a_finite_refusal_does_not_advance_the_shared_issuer() {
        let store = CssomStore::new(Default::default(), Default::default()).unwrap();
        let mut state = (*store.state).clone();
        let mut limits = store.limits.clone();
        limits.max_identity = 1;
        assert!(matches!(
            state.allocate(&limits),
            Err(CssomError::IdentityExhausted)
        ));
        limits.max_identity = 2;
        assert_eq!(state.allocate(&limits).unwrap(), 2);
        assert!(matches!(
            state.allocate(&limits),
            Err(CssomError::IdentityExhausted)
        ));
        limits.max_identity = 3;
        assert_eq!(state.allocate(&limits).unwrap(), 3);
    }
}
